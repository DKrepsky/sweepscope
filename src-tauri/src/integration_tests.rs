//! The real device, acquisition and display modules on the simulator, wired by `compose`.
use std::sync::Arc;
use std::time::Duration;

use sweepscope_acquisition::{Phase, SetRunning};
use sweepscope_device::SimulatorControl;
use sweepscope_display::DisplayFrameReady;
use sweepscope_kernel::contracts::ConnectionState;
use sweepscope_kernel::testing::{eventually, MemoryStorage};
use sweepscope_kernel::{Bus, SystemClock};

use crate::composition::{compose, simulator_source, unix_time_us, Backend, Wiring};
use crate::ipc::snapshot_with;

const WAIT: Duration = Duration::from_secs(10);

fn simulated_backend() -> (Backend, SimulatorControl) {
    let clock = Arc::new(SystemClock);
    let source = simulator_source(clock.clone(), true);
    let control = source.control.clone();
    let backend = compose(Wiring {
        storage: Arc::new(MemoryStorage::default()),
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
