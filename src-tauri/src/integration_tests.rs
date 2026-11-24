//! The real device, acquisition and display modules on the simulator, wired by `compose`.
use std::sync::Arc;
use std::time::Duration;

use sweepscope_acquisition::{Phase, SetRunning};
use sweepscope_device::SimulatorControl;
use sweepscope_display::DisplayFrameReady;
use sweepscope_export::{ExportFormat, ExportFrame, SaveScreenshot};
use sweepscope_kernel::contracts::ConnectionState;
use sweepscope_kernel::testing::{eventually, MemoryStorage, ScriptedPicker};
use sweepscope_kernel::{Bus, SystemClock};
use sweepscope_recording::OsRecordFiles;

use crate::composition::{compose, simulator_source, unix_time_us, Backend, Wiring};
use crate::ipc::snapshot_with;

const WAIT: Duration = Duration::from_secs(10);

pub(crate) fn simulated_backend() -> (Backend, SimulatorControl) {
    simulated_backend_with(Arc::new(ScriptedPicker::default()))
}

fn simulated_backend_with(picker: Arc<ScriptedPicker>) -> (Backend, SimulatorControl) {
    let clock = Arc::new(SystemClock);
    let source = simulator_source(clock.clone(), true);
    let control = source.control.clone();
    let backend = compose(Wiring {
        storage: Arc::new(MemoryStorage::default()),
        picker,
        record_files: Arc::new(OsRecordFiles),
        clock,
        hardware: None,
        simulator: Some(source),
        unix_time_us,
    })
    .expect("compose");
    (backend, control)
}

fn wait_until_ready(bus: &Bus) {
    assert!(eventually(WAIT, || {
        snapshot_with(bus).is_ok_and(|s| s.connection.state == ConnectionState::Ready && s.instrument.is_some())
    }));
}

#[test]
fn given_the_composed_app_with_the_simulator_when_a_snapshot_is_requested_then_the_device_is_ready_with_an_instrument()
{
    let (backend, _control) = simulated_backend();

    wait_until_ready(&backend.bus);

    let snapshot = snapshot_with(&backend.bus).expect("snapshot");
    assert!(snapshot.connection.simulator_active);
    assert!(!snapshot.instrument.expect("instrument").running);
}

#[test]
fn given_a_ready_simulator_when_running_then_display_frames_arrive_and_the_status_leaves_stopped() {
    let (backend, _control) = simulated_backend();
    wait_until_ready(&backend.bus);
    let frames = backend.bus.subscribe_latest::<DisplayFrameReady>();

    backend.bus.request(SetRunning { running: true }).expect("run");

    assert!(eventually(WAIT, || frames.try_next().is_some()));
    assert_ne!(
        snapshot_with(&backend.bus).expect("snapshot").status.phase,
        Phase::Stopped
    );
}

#[test]
fn given_a_running_simulator_when_unplugged_then_the_connection_drops_and_running_ends() {
    let (backend, control) = simulated_backend();
    wait_until_ready(&backend.bus);
    backend.bus.request(SetRunning { running: true }).expect("run");

    control.unplug();

    assert!(eventually(WAIT, || {
        snapshot_with(&backend.bus).is_ok_and(|s| {
            s.connection.state == ConnectionState::Disconnected
                && s.status.phase == Phase::Stopped
                && s.instrument.is_some_and(|i| !i.running)
        })
    }));
}

#[test]
fn given_a_running_simulator_when_frames_arrive_then_the_snapshot_carries_measurements_for_the_enabled_channels() {
    let (backend, _control) = simulated_backend();
    wait_until_ready(&backend.bus);

    backend.bus.request(SetRunning { running: true }).expect("run");

    assert!(eventually(WAIT, || {
        snapshot_with(&backend.bus).is_ok_and(|s| !s.measurements.channels.is_empty())
    }));
}

#[test]
fn given_a_frame_and_a_cancelled_dialog_when_exported_then_nothing_is_saved() {
    let picker = Arc::new(ScriptedPicker::new(vec![None], vec![]));
    let (backend, _control) = simulated_backend_with(picker);
    wait_until_ready(&backend.bus);
    backend.bus.request(SetRunning { running: true }).expect("run");

    let saved = eventually(WAIT, || {
        backend
            .bus
            .request(ExportFrame {
                format: ExportFormat::Csv,
            })
            .is_ok_and(|saved| saved.is_none())
    });

    assert!(saved);
}

#[test]
fn given_a_body_that_is_not_a_png_when_saved_as_a_screenshot_then_it_is_refused_before_any_dialog() {
    let (backend, _control) = simulated_backend();

    let error = backend
        .bus
        .request(SaveScreenshot {
            png: b"not a png".to_vec(),
        })
        .expect_err("refused");

    assert_eq!(error.category, sweepscope_kernel::ErrorCategory::Recoverable);
}
