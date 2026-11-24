use sweepscope_acquisition::{
    ArmSingle, ChannelPatch, ForceTriggerNow, LoadSetup, MathPatch, ResetSetting, SaveSetup, SetChannel, SetMath,
    SetRunning, SetTrigger, SetTriggerLevel, SetTriggerPosition, SettingTarget, SetupLoaded, StartAutoset, StepSetting,
    TriggerPatch,
};
use sweepscope_calibration::{CloseZeroCalibration, MeasureZeroChannel, OpenZeroCalibration, StartZeroCalibration};
use sweepscope_device::{RetryConnection, UseSimulator};
use sweepscope_display::{InterpolationMode, Persistence, SetInterpolation, SetPersistence};
use sweepscope_export::{ExportFormat, ExportFrame, SaveScreenshot};
use sweepscope_kernel::contracts::SavedFile;
use sweepscope_kernel::{Bus, KernelError, Request};
use sweepscope_recording::{StartRecording, StopRecording};
use sweepscope_settings::{LoadUiState, SaveUiState, UiStateEntry};
use sweepscope_spectrum::{SetSpectrum, SpectrumPatch};
use tauri::ipc::{Channel, InvokeBody, InvokeResponseBody};
use tauri::State;

use super::snapshot::{snapshot_with, StateSnapshot};
use super::streams::{FrameStreams, RollStreams, SpectrumStreams};
use crate::composition::{unix_time_us, Backend};

/// Every command passed to `handler`; compared in tests with the manifest, `build.rs` and the capability.
#[cfg(test)]
pub const REGISTERED_COMMANDS: &[&str] = &[
    "load_ui_state",
    "save_ui_state",
    "get_snapshot",
    "set_interpolation",
    "export_frame",
    "save_screenshot",
    "save_setup",
    "load_setup",
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
    "arm_single",
    "start_autoset",
    "set_math",
    "set_persistence",
    "set_spectrum",
    "start_recording",
    "stop_recording",
    "open_roll_stream",
    "close_roll_stream",
    "open_spectrum_stream",
    "close_spectrum_stream",
    "open_zero_calibration",
    "start_zero_calibration",
    "measure_zero_calibration",
    "close_zero_calibration",
];

pub fn load_ui_state_with(bus: &Bus, namespace: String) -> Result<Option<UiStateEntry>, KernelError> {
    bus.request(LoadUiState { namespace })
}

pub fn save_ui_state_with(bus: &Bus, entry: UiStateEntry) -> Result<(), KernelError> {
    bus.request(SaveUiState(entry))
}

/// The raw body of `save_screenshot`; a JSON body is refused so the command never parses untrusted text as bytes.
pub fn screenshot_bytes(body: &InvokeBody) -> Result<Vec<u8>, KernelError> {
    match body {
        InvokeBody::Raw(bytes) => Ok(bytes.clone()),
        InvokeBody::Json(_) => Err(KernelError::recoverable("the screenshot must be sent as binary data")),
    }
}

/// Dialogs block, so file commands run on a blocking thread and never on the main thread or the bus.
async fn blocking_request<R>(bus: Bus, request: R) -> Result<R::Response, KernelError>
where
    R: Request + Send + 'static,
    R::Response: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || bus.request(request))
        .await
        .map_err(|error| KernelError::fatal(format!("the file command stopped unexpectedly: {error}")))?
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
fn set_interpolation(backend: State<'_, Backend>, mode: InterpolationMode) -> Result<(), KernelError> {
    backend.bus.request(SetInterpolation { mode })
}

#[tauri::command]
async fn export_frame(backend: State<'_, Backend>, format: ExportFormat) -> Result<Option<SavedFile>, KernelError> {
    blocking_request(backend.bus.clone(), ExportFrame { format }).await
}

#[tauri::command]
async fn save_screenshot(
    backend: State<'_, Backend>,
    request: tauri::ipc::Request<'_>,
) -> Result<Option<SavedFile>, KernelError> {
    let png = screenshot_bytes(request.body())?;
    blocking_request(backend.bus.clone(), SaveScreenshot { png }).await
}

#[tauri::command]
async fn save_setup(backend: State<'_, Backend>) -> Result<Option<SavedFile>, KernelError> {
    blocking_request(backend.bus.clone(), SaveSetup).await
}

#[tauri::command]
async fn load_setup(backend: State<'_, Backend>) -> Result<Option<SetupLoaded>, KernelError> {
    blocking_request(backend.bus.clone(), LoadSetup).await
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
fn arm_single(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(ArmSingle)
}

#[tauri::command]
fn start_autoset(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(StartAutoset)
}

#[tauri::command]
fn set_math(backend: State<'_, Backend>, patch: MathPatch) -> Result<(), KernelError> {
    backend.bus.request(SetMath { patch })
}

#[tauri::command]
fn set_persistence(backend: State<'_, Backend>, value: Persistence) -> Result<(), KernelError> {
    backend.bus.request(SetPersistence { value })
}

#[tauri::command]
fn set_spectrum(backend: State<'_, Backend>, patch: SpectrumPatch) -> Result<(), KernelError> {
    backend.bus.request(SetSpectrum { patch })
}

#[tauri::command]
async fn start_recording(backend: State<'_, Backend>) -> Result<Option<SavedFile>, KernelError> {
    blocking_request(backend.bus.clone(), StartRecording).await
}

#[tauri::command]
fn stop_recording(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(StopRecording)
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
fn open_zero_calibration(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(OpenZeroCalibration)
}

#[tauri::command]
fn start_zero_calibration(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(StartZeroCalibration)
}

#[tauri::command]
fn measure_zero_calibration(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(MeasureZeroChannel)
}

#[tauri::command]
fn close_zero_calibration(backend: State<'_, Backend>) -> Result<(), KernelError> {
    backend.bus.request(CloseZeroCalibration)
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

#[tauri::command]
fn open_roll_stream(
    backend: State<'_, Backend>,
    streams: State<'_, RollStreams>,
    on_frame: Channel<InvokeResponseBody>,
) -> Result<(), KernelError> {
    streams.open(&backend.bus, on_frame, unix_time_us)
}

#[tauri::command]
fn close_roll_stream(streams: State<'_, RollStreams>) {
    streams.close();
}

#[tauri::command]
fn open_spectrum_stream(
    backend: State<'_, Backend>,
    streams: State<'_, SpectrumStreams>,
    on_frame: Channel<InvokeResponseBody>,
) -> Result<(), KernelError> {
    streams.open(&backend.bus, on_frame, unix_time_us)
}

#[tauri::command]
fn close_spectrum_stream(streams: State<'_, SpectrumStreams>) {
    streams.close();
}

pub fn install(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        load_ui_state,
        save_ui_state,
        get_snapshot,
        set_interpolation,
        export_frame,
        save_screenshot,
        save_setup,
        load_setup,
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
        arm_single,
        start_autoset,
        set_math,
        set_persistence,
        set_spectrum,
        start_recording,
        stop_recording,
        open_roll_stream,
        close_roll_stream,
        open_spectrum_stream,
        close_spectrum_stream,
        open_zero_calibration,
        start_zero_calibration,
        measure_zero_calibration,
        close_zero_calibration,
    ])
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sweepscope_kernel::testing::MemoryStorage;
    use sweepscope_kernel::ErrorCategory;
    use sweepscope_settings::{SettingsDeps, SettingsModule};

    use super::*;

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
    fn given_a_json_body_when_the_screenshot_bytes_are_read_then_it_is_refused() {
        let error = screenshot_bytes(&InvokeBody::Json(serde_json::json!([137, 80]))).expect_err("json body");

        assert_eq!(error.category, ErrorCategory::Recoverable);
        assert!(error.message.contains("binary"));
    }

    #[test]
    fn given_a_raw_body_when_the_screenshot_bytes_are_read_then_they_are_returned_unchanged() {
        let bytes = screenshot_bytes(&InvokeBody::Raw(vec![137, 80, 78, 71])).expect("raw body");

        assert_eq!(bytes, [137, 80, 78, 71]);
    }
}
