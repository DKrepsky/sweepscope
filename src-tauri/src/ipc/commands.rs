use sweepscope_acquisition::{
    ChannelPatch, ForceTriggerNow, GetInstrument, InstrumentSnapshot, ResetSetting, SetChannel, SetRunning, SetTrigger,
    SetTriggerLevel, SetTriggerPosition, SettingTarget, StateSnapshot, StepSetting, TriggerPatch,
};
use sweepscope_device::{GetConnection, RetryConnection, UseSimulator};
use sweepscope_kernel::{Bus, KernelError};
use sweepscope_settings::{LoadUiState, SaveUiState, UiStateEntry};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::State;

use super::frames::FrameStreams;
use crate::composition::{unix_time_us, Backend};

/// Every command passed to `handler`; compared in tests with the manifest, `build.rs` and the capability.
#[cfg(test)]
pub const REGISTERED_COMMANDS: &[&str] = &[
    "load_ui_state",
    "save_ui_state",
    "get_snapshot",
    "set_running",
    "force_trigger",
    "set_channel",
    "step_setting",
    "reset_setting",
    "set_trigger_position",
    "set_trigger_level",
    "set_trigger",
    "use_simulator",
    "retry_connection",
    "open_frame_stream",
    "close_frame_stream",
];

pub fn load_ui_state_with(bus: &Bus, namespace: String) -> Result<Option<UiStateEntry>, KernelError> {
    bus.request(LoadUiState { namespace })
}

pub fn save_ui_state_with(bus: &Bus, entry: UiStateEntry) -> Result<(), KernelError> {
    bus.request(SaveUiState(entry))
}

/// Assembles the four slices; the shell holds no state of its own (ADR 0013 item 1).
pub fn snapshot_with(bus: &Bus) -> Result<StateSnapshot, KernelError> {
    let connection = bus.request(GetConnection)?;
    let InstrumentSnapshot {
        instrument,
        status,
        facts,
    } = bus.request(GetInstrument)?;
    Ok(StateSnapshot {
        connection,
        instrument,
        status,
        facts,
    })
}

#[tauri::command]
fn load_ui_state(backend: State<'_, Backend>, namespace: String) -> Result<Option<UiStateEntry>, KernelError> {
    load_ui_state_with(&backend.bus, namespace)
}

#[tauri::command]
fn save_ui_state(backend: State<'_, Backend>, entry: UiStateEntry) -> Result<(), KernelError> {
    save_ui_state_with(&backend.bus, entry)
}

#[tauri::command]
fn get_snapshot(backend: State<'_, Backend>) -> Result<StateSnapshot, KernelError> {
    snapshot_with(&backend.bus)
}

#[tauri::command]
fn set_running(backend: State<'_, Backend>, running: bool) -> Result<(), KernelError> {
    backend.bus.request(SetRunning { running })
}

#[tauri::command]
fn force_trigger(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(ForceTriggerNow)
}

#[tauri::command]
fn set_channel(backend: State<'_, Backend>, channel: u8, patch: ChannelPatch) -> Result<(), KernelError> {
    backend.bus.request(SetChannel { channel, patch })
}

#[tauri::command]
fn step_setting(backend: State<'_, Backend>, target: SettingTarget, direction: i8) -> Result<(), KernelError> {
    backend.bus.request(StepSetting { target, direction })
}

#[tauri::command]
fn reset_setting(backend: State<'_, Backend>, target: SettingTarget) -> Result<(), KernelError> {
    backend.bus.request(ResetSetting { target })
}

#[tauri::command]
fn set_trigger_position(backend: State<'_, Backend>, fraction: f64) -> Result<(), KernelError> {
    backend.bus.request(SetTriggerPosition { fraction })
}

#[tauri::command]
fn set_trigger_level(backend: State<'_, Backend>, divisions: f64) -> Result<(), KernelError> {
    backend.bus.request(SetTriggerLevel { divisions })
}

#[tauri::command]
fn set_trigger(backend: State<'_, Backend>, patch: TriggerPatch) -> Result<(), KernelError> {
    backend.bus.request(SetTrigger { patch })
}

#[tauri::command]
fn use_simulator(backend: State<'_, Backend>, enabled: bool) -> Result<(), KernelError> {
    backend.bus.request(UseSimulator { enabled })
}

#[tauri::command]
fn retry_connection(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(RetryConnection)
}

#[tauri::command]
fn open_frame_stream(
    backend: State<'_, Backend>,
    streams: State<'_, FrameStreams>,
    on_frame: Channel<InvokeResponseBody>,
) -> Result<(), KernelError> {
    streams.open(&backend.bus, on_frame, unix_time_us)
}

#[tauri::command]
fn close_frame_stream(streams: State<'_, FrameStreams>) {
    streams.close();
}

pub fn install(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        load_ui_state,
        save_ui_state,
        get_snapshot,
        set_running,
        force_trigger,
        set_channel,
        step_setting,
        reset_setting,
        set_trigger_position,
        set_trigger_level,
        set_trigger,
        use_simulator,
        retry_connection,
        open_frame_stream,
        close_frame_stream,
    ])
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use sweepscope_acquisition::{AcquisitionStatus, InstrumentFacts, Phase};
    use sweepscope_device::{DeviceDeps, DeviceModule};
    use sweepscope_kernel::contracts::ConnectionState;
    use sweepscope_kernel::testing::{eventually, FakeClock, MemoryStorage};
    use sweepscope_kernel::ErrorCategory;
    use sweepscope_settings::{SettingsDeps, SettingsModule};

    use super::*;
    use crate::composition::{compose, simulator_source, Wiring};

    fn entry(namespace: &str) -> UiStateEntry {
        UiStateEntry {
            namespace: namespace.into(),
            version: 1,
            json: "{\"percent\":100}".into(),
        }
    }

    fn settings_bus() -> (Bus, SettingsModule) {
        let bus = Bus::new();
        let module = SettingsModule::new(SettingsDeps {
            bus: bus.clone(),
            storage: Arc::new(MemoryStorage::default()),
        })
        .expect("settings module");
        (bus, module)
    }

    fn quiet_instrument() -> InstrumentSnapshot {
        InstrumentSnapshot {
            instrument: None,
            status: AcquisitionStatus {
                revision: 1,
                phase: Phase::Stopped,
            },
            facts: InstrumentFacts {
                revision: 1,
                sample_rate: None,
                sample_interval: None,
                record_points: None,
                capture_rate: None,
                dead_time: None,
            },
        }
    }

    #[test]
    fn given_a_composed_app_when_an_unknown_namespace_is_loaded_then_it_returns_none() {
        let (bus, _module) = settings_bus();
        let loaded = load_ui_state_with(&bus, "app.unknown".into());
        assert_eq!(loaded, Ok(None));
    }

    #[test]
    fn given_a_saved_entry_when_loaded_through_the_handlers_then_it_round_trips() {
        let (bus, _module) = settings_bus();
        save_ui_state_with(&bus, entry("app.text-scale")).expect("save");
        let loaded = load_ui_state_with(&bus, "app.text-scale".into());
        assert_eq!(loaded, Ok(Some(entry("app.text-scale"))));
    }

    #[test]
    fn given_a_handler_error_when_serialised_for_the_webview_then_it_has_category_and_message_without_paths() {
        let error = load_ui_state_with(&Bus::new(), "app.text-scale".into()).expect_err("no handler registered");

        let json = serde_json::to_value(&error).expect("serialise");
        assert_eq!(error.category, ErrorCategory::Fatal);
        assert_eq!(json["category"], "fatal");
        assert!(json["message"].is_string());
        assert!(!error.message.contains('/') && !error.message.contains('\\'));
    }

    #[test]
    fn given_no_slice_handlers_when_a_snapshot_is_requested_then_the_error_has_category_and_message_without_paths() {
        let error = snapshot_with(&Bus::new()).expect_err("no handler registered");

        let json = serde_json::to_value(&error).expect("serialise");
        assert!(json["category"].is_string());
        assert!(json["message"].is_string());
        assert!(!error.message.contains('/') && !error.message.contains('\\'));
    }

    #[test]
    fn given_a_ready_simulated_device_when_a_snapshot_is_requested_then_it_carries_the_connection_and_the_instrument_slices(
    ) {
        let bus = Bus::new();
        let _device = DeviceModule::new(DeviceDeps {
            bus: bus.clone(),
            clock: Arc::new(FakeClock::default()),
            hardware: None,
            simulator: Some(simulator_source(Arc::new(FakeClock::default()), true)),
        })
        .expect("device module");
        bus.handle(|_: GetInstrument| Ok(quiet_instrument()))
            .expect("fake acquisition");
        assert!(eventually(Duration::from_secs(5), || {
            snapshot_with(&bus).is_ok_and(|s| s.connection.state == ConnectionState::Ready)
        }));

        let snapshot = snapshot_with(&bus).expect("snapshot");

        assert!(snapshot.connection.simulator_active);
        assert_eq!(snapshot.status.phase, Phase::Stopped);
        assert_eq!(snapshot.instrument, None);
    }

    #[test]
    fn given_the_composed_app_with_the_simulator_when_a_snapshot_is_requested_then_the_device_becomes_ready() {
        let clock = Arc::new(FakeClock::default());
        let backend = compose(Wiring {
            storage: Arc::new(MemoryStorage::default()),
            clock: clock.clone(),
            hardware: None,
            simulator: Some(simulator_source(clock, true)),
            unix_time_us: crate::composition::unix_time_us,
        })
        .expect("compose");

        assert!(eventually(Duration::from_secs(5), || {
            snapshot_with(&backend.bus).is_ok_and(|s| s.connection.state == ConnectionState::Ready)
        }));
    }
}
