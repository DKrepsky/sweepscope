use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sweepscope_acquisition::{AcquisitionDeps, AcquisitionModule};
use sweepscope_calibration::{CalibrationDeps, CalibrationModule};
use sweepscope_device::{
    DeviceCatalog, DeviceDeps, DeviceModule, HantekCatalog, SignalKind, SignalSpec, SimulatedCatalog, SimulatorControl,
    SimulatorSource,
};
use sweepscope_display::{DisplayDeps, DisplayModule};
use sweepscope_export::{ExportDeps, ExportModule};
use sweepscope_kernel::{Bus, Clock, FilePicker, FileStorage, KernelError, Storage, SystemClock};
use sweepscope_measurement::{MeasurementDeps, MeasurementModule};
use sweepscope_recording::{OsRecordFiles, RecordFiles, RecordingDeps, RecordingModule};
use sweepscope_settings::{SettingsDeps, SettingsModule};
use sweepscope_spectrum::{SpectrumDeps, SpectrumModule};

const SIMULATOR_VARIABLE: &str = "SWEEPSCOPE_SIMULATOR";
const NO_HARDWARE_VARIABLE: &str = "SWEEPSCOPE_NO_HARDWARE";
const SIMULATOR_SIGNAL_VARIABLE: &str = "SWEEPSCOPE_SIMULATOR_SIGNAL";
const CHANNELS: usize = 4;

/// The wired backend. Modules are kept alive here; the bus is the only way in.
/// Fields drop in declaration order, so consumers stop before the producers they read from.
pub struct Backend {
    pub bus: Bus,
    _display: DisplayModule,
    _measurement: MeasurementModule,
    _calibration: CalibrationModule,
    _spectrum: SpectrumModule,
    _recording: RecordingModule,
    _export: ExportModule,
    _acquisition: AcquisitionModule,
    _device: DeviceModule,
    _settings: SettingsModule,
}

/// Everything `compose` needs from the outside, so tests can swap the world for fakes.
pub struct Wiring {
    pub storage: Arc<dyn Storage>,
    pub picker: Arc<dyn FilePicker>,
    pub record_files: Arc<dyn RecordFiles>,
    pub clock: Arc<dyn Clock>,
    pub hardware: Option<Box<dyn DeviceCatalog>>,
    pub simulator: Option<SimulatorSource>,
    pub unix_time_us: fn() -> f64,
}

/// Consumers of frames and instrument state; built before the producers publish.
struct Consumers {
    recording: RecordingModule,
    spectrum: SpectrumModule,
    measurement: MeasurementModule,
    export: ExportModule,
    calibration: CalibrationModule,
}

fn consumers(bus: &Bus, wiring: &Wiring) -> Result<Consumers, KernelError> {
    Ok(Consumers {
        recording: RecordingModule::new(RecordingDeps {
            bus: bus.clone(),
            clock: Arc::clone(&wiring.clock),
            picker: Arc::clone(&wiring.picker),
            files: Arc::clone(&wiring.record_files),
            unix_time_us: wiring.unix_time_us,
        })?,
        spectrum: SpectrumModule::new(SpectrumDeps {
            bus: bus.clone(),
            clock: Arc::clone(&wiring.clock),
            unix_time_us: wiring.unix_time_us,
        })?,
        measurement: MeasurementModule::new(MeasurementDeps {
            bus: bus.clone(),
            clock: Arc::clone(&wiring.clock),
        })?,
        export: ExportModule::new(ExportDeps {
            bus: bus.clone(),
            picker: Arc::clone(&wiring.picker),
            unix_time_us: wiring.unix_time_us,
        })?,
        calibration: CalibrationModule::new(CalibrationDeps {
            bus: bus.clone(),
            clock: Arc::clone(&wiring.clock),
            storage: Arc::clone(&wiring.storage),
            unix_time_us: wiring.unix_time_us,
        })?,
    })
}

pub fn compose(wiring: Wiring) -> Result<Backend, KernelError> {
    let bus = Bus::new();
    let settings = SettingsModule::new(SettingsDeps {
        bus: bus.clone(),
        storage: Arc::clone(&wiring.storage),
    })?;
    let consumers = consumers(&bus, &wiring)?;
    let acquisition = AcquisitionModule::new(AcquisitionDeps {
        bus: bus.clone(),
        clock: Arc::clone(&wiring.clock),
        storage: wiring.storage,
        picker: wiring.picker,
    })?;
    let device = DeviceModule::new(DeviceDeps {
        bus: bus.clone(),
        clock: wiring.clock,
        hardware: wiring.hardware,
        simulator: wiring.simulator,
    })?;
    let display = DisplayModule::new(DisplayDeps {
        bus: bus.clone(),
        unix_time_us: wiring.unix_time_us,
    })?;
    Ok(Backend {
        bus,
        _display: display,
        _measurement: consumers.measurement,
        _calibration: consumers.calibration,
        _spectrum: consumers.spectrum,
        _recording: consumers.recording,
        _export: consumers.export,
        _acquisition: acquisition,
        _device: device,
        _settings: settings,
    })
}

pub fn compose_on_disk(config_dir: PathBuf, picker: Arc<dyn FilePicker>) -> Result<Backend, KernelError> {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let start_plugged = simulator_requested(std::env::var(SIMULATOR_VARIABLE).ok().as_deref());
    let simulator = simulator_source(Arc::clone(&clock), start_plugged);
    apply_simulator_signal(
        &simulator.control,
        simulator_signal(std::env::var(SIMULATOR_SIGNAL_VARIABLE).ok().as_deref()),
    );
    compose(Wiring {
        storage: Arc::new(FileStorage::new(config_dir)),
        picker,
        record_files: Arc::new(OsRecordFiles),
        hardware: choose_hardware(
            hardware_disabled(std::env::var(NO_HARDWARE_VARIABLE).ok().as_deref()),
            hardware_catalog,
        ),
        simulator: Some(simulator),
        clock,
        unix_time_us,
    })
}

pub fn simulator_requested(value: Option<&str>) -> bool {
    value == Some("1")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulatorSignal {
    /// The default: the usual waveform on every input.
    Open,
    /// Every input is shorted to ground, so the zero calibration can be tried without a scope.
    Shorted,
}

pub fn simulator_signal(value: Option<&str>) -> SimulatorSignal {
    match value {
        None | Some("open") => SimulatorSignal::Open,
        Some("shorted") => SimulatorSignal::Shorted,
        Some(other) => {
            log::warn!("{SIMULATOR_SIGNAL_VARIABLE}={other} is not shorted or open, using open");
            SimulatorSignal::Open
        }
    }
}

fn apply_simulator_signal(control: &SimulatorControl, signal: SimulatorSignal) {
    if signal == SimulatorSignal::Shorted {
        for channel in 0..CHANNELS {
            control.set_signal(channel, SignalSpec::new(SignalKind::Shorted));
        }
    }
}

pub fn hardware_disabled(value: Option<&str>) -> bool {
    value == Some("1")
}

/// Takes the discovery as a function so a disabled run never touches USB, and a test can prove it.
fn choose_hardware(
    disabled: bool,
    start: impl FnOnce() -> Option<Box<dyn DeviceCatalog>>,
) -> Option<Box<dyn DeviceCatalog>> {
    if disabled {
        log::info!("hardware discovery is off ({NO_HARDWARE_VARIABLE}=1)");
        return None;
    }
    start()
}

/// The simulator is always available to `use_simulator`; it starts plugged in only when asked for.
/// Unplugging before the module is built means the manager never sees an `Attached` event.
pub fn simulator_source(clock: Arc<dyn Clock>, start_plugged: bool) -> SimulatorSource {
    let control = SimulatorControl::new();
    if !start_plugged {
        control.unplug();
    }
    SimulatorSource {
        catalog: Box::new(SimulatedCatalog::new(clock, control.clone())),
        control,
    }
}

fn hardware_catalog() -> Option<Box<dyn DeviceCatalog>> {
    match HantekCatalog::start() {
        Ok(catalog) => Some(Box::new(catalog)),
        Err(error) => {
            log::warn!("hardware discovery is unavailable: {error}");
            None
        }
    }
}

pub fn unix_time_us() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64() * 1_000_000.0)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sweepscope_acquisition::SetRunning;
    use sweepscope_device::{GetConnection, UseSimulator};
    use sweepscope_kernel::contracts::{ConnectionState, FrameCaptured};
    use sweepscope_kernel::testing::{eventually, FakeClock, MemoryStorage, ScriptedPicker};
    use sweepscope_kernel::SystemClock;
    use sweepscope_recording::OsRecordFiles;

    use super::*;

    const WAIT: Duration = Duration::from_secs(5);

    fn device_with(source: SimulatorSource) -> (Bus, DeviceModule) {
        let bus = Bus::new();
        let module = DeviceModule::new(DeviceDeps {
            bus: bus.clone(),
            clock: Arc::new(FakeClock::default()),
            hardware: None,
            simulator: Some(source),
        })
        .expect("device module");
        (bus, module)
    }

    fn connection_state(bus: &Bus) -> ConnectionState {
        bus.request(GetConnection).expect("connection").state
    }

    #[test]
    fn given_the_variable_is_one_when_read_then_the_simulator_is_requested() {
        assert!(simulator_requested(Some("1")));
    }

    #[test]
    fn given_the_variable_is_missing_or_anything_else_when_read_then_the_simulator_is_not_requested() {
        assert!(!simulator_requested(None));
        assert!(!simulator_requested(Some("0")));
        assert!(!simulator_requested(Some("true")));
    }

    #[test]
    fn given_the_no_hardware_variable_is_one_when_read_then_hardware_is_disabled() {
        assert!(hardware_disabled(Some("1")));
        assert!(!hardware_disabled(None));
        assert!(!hardware_disabled(Some("0")));
    }

    #[test]
    fn given_hardware_is_disabled_when_the_catalog_is_chosen_then_discovery_is_never_started() {
        let mut started = false;

        let catalog = choose_hardware(true, || {
            started = true;
            None
        });

        assert!(catalog.is_none());
        assert!(!started);
    }

    #[test]
    fn given_hardware_is_enabled_when_the_catalog_is_chosen_then_discovery_is_started() {
        let mut started = false;

        choose_hardware(false, || {
            started = true;
            None
        });

        assert!(started);
    }

    #[test]
    fn given_the_simulator_is_requested_when_the_device_starts_then_it_becomes_ready() {
        let (bus, _module) = device_with(simulator_source(Arc::new(FakeClock::default()), true));

        assert!(eventually(WAIT, || connection_state(&bus) == ConnectionState::Ready));
    }

    #[test]
    fn given_the_simulator_is_not_requested_when_the_device_starts_then_it_stays_disconnected_until_enabled() {
        let (bus, _module) = device_with(simulator_source(Arc::new(FakeClock::default()), false));
        assert_eq!(connection_state(&bus), ConnectionState::Disconnected);

        bus.request(UseSimulator { enabled: true }).expect("enable");

        assert!(eventually(WAIT, || connection_state(&bus) == ConnectionState::Ready));
    }

    #[test]
    fn given_the_signal_variable_is_shorted_when_read_then_the_inputs_are_shorted() {
        assert_eq!(simulator_signal(Some("shorted")), SimulatorSignal::Shorted);
    }

    #[test]
    fn given_the_signal_variable_is_missing_open_or_unknown_when_read_then_the_inputs_are_open() {
        assert_eq!(simulator_signal(None), SimulatorSignal::Open);
        assert_eq!(simulator_signal(Some("open")), SimulatorSignal::Open);
        assert_eq!(simulator_signal(Some("1")), SimulatorSignal::Open);
    }

    fn peak_to_peak_of_first_frame(signal: SimulatorSignal) -> u8 {
        let clock = Arc::new(SystemClock);
        let source = simulator_source(clock.clone(), true);
        apply_simulator_signal(&source.control, signal);
        let backend = compose(Wiring {
            storage: Arc::new(MemoryStorage::default()),
            picker: Arc::new(ScriptedPicker::default()),
            record_files: Arc::new(OsRecordFiles),
            clock,
            hardware: None,
            simulator: Some(source),
            unix_time_us,
        })
        .expect("compose");
        let frames = backend.bus.subscribe_latest::<FrameCaptured>();
        assert!(eventually(WAIT, || connection_state(&backend.bus) == ConnectionState::Ready));
        assert!(eventually(WAIT, || backend
            .bus
            .request(SetRunning { running: true })
            .is_ok()));
        let mut first = None;
        assert!(eventually(WAIT, || {
            first = frames.try_next();
            first.is_some()
        }));
        let captured = first.expect("frame");
        let codes = &captured.frame.channels[0].as_ref().expect("channel 1").codes;
        codes.iter().max().expect("codes") - codes.iter().min().expect("codes")
    }

    #[test]
    fn given_shorted_inputs_when_a_frame_is_captured_then_channel_one_is_nearly_flat() {
        assert!(peak_to_peak_of_first_frame(SimulatorSignal::Shorted) < 30);
    }

    #[test]
    fn given_open_inputs_when_a_frame_is_captured_then_channel_one_shows_the_waveform() {
        assert!(peak_to_peak_of_first_frame(SimulatorSignal::Open) > 30);
    }

    #[test]
    fn given_the_clock_when_read_then_the_unix_time_is_in_microseconds() {
        assert!(unix_time_us() > 1.0e15);
    }
}
