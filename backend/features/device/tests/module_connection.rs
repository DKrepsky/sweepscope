//! Connection lifecycle of the device module, on simulator devices and real threads.
mod module_support;

use std::sync::atomic::Ordering;

use sweepscope_device::{DeviceError, RetryConnection, UseSimulator};
use sweepscope_kernel::contracts::{ApplySetup, ConnectionIssueKind, ConnectionState, DeviceSource, SetCapturing};
use sweepscope_kernel::{ErrorCategory, KernelError};

use module_support::*;

fn apply(revision: u64) -> ApplySetup {
    ApplySetup {
        setup_revision: revision,
        setup: setup(sweepscope_device::Sweep::Auto, 0.0),
    }
}

#[test]
fn given_a_simulator_plugged_in_when_the_module_starts_then_it_goes_disconnected_connecting_ready_with_identity_and_capabilities(
) {
    let mut simulator = Source::plugged();

    let rig = rig(None, Some(&mut simulator));

    rig.wait_for_state(ConnectionState::Ready);
    let events = rig.connection.events();
    let states: Vec<_> = events.iter().map(|e| e.snapshot.state).collect();
    assert_eq!(
        states,
        [
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Ready
        ]
    );
    let ready = events.last().unwrap();
    assert!(ready.snapshot.identity.as_ref().unwrap().simulated);
    assert_eq!(ready.snapshot.source, Some(DeviceSource::Simulator));
    assert!(ready.snapshot.simulator_active);
    assert_eq!(ready.capabilities.as_ref().unwrap().channel_count, 4);
    let revisions: Vec<_> = events.iter().map(|e| e.snapshot.revision).collect();
    assert!(revisions.windows(2).all(|pair| pair[1] > pair[0]), "{revisions:?}");
}

#[test]
fn given_a_ready_device_when_it_is_unplugged_then_disconnected_is_published_and_the_worker_has_ended() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);
    assert_eq!(simulator.hooks.live_devices.load(Ordering::SeqCst), 1);

    simulator.control.unplug();

    rig.wait_for_state(ConnectionState::Disconnected);
    assert!(wait_until(|| simulator.hooks.live_devices.load(Ordering::SeqCst) == 0));
    assert!(!rig.snapshot().simulator_active);
}

#[test]
fn given_an_unplugged_simulator_when_it_is_plugged_in_again_then_a_new_session_is_ready_and_accepts_a_setup() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);
    simulator.control.unplug();
    rig.wait_for_state(ConnectionState::Disconnected);

    simulator.control.plug_in();

    rig.wait_for_state(ConnectionState::Ready);
    assert_eq!(simulator.hooks.opened.load(Ordering::SeqCst), 2);
    assert_eq!(rig.bus.request(apply(1)), Ok(()));
}

#[test]
fn given_a_catalog_whose_open_says_not_found_when_the_module_handles_it_then_it_is_disconnected_without_an_issue() {
    let mut hardware = Source::plugged();
    *hardware.hooks.open_error.lock().unwrap() = Some(DeviceError::NotFound);

    let rig = rig(Some(&mut hardware), None);

    assert!(wait_until(|| hardware.hooks.open_error.lock().unwrap().is_some()));
    rig.wait_for_state(ConnectionState::Disconnected);
    let events = rig.connection.events();
    assert!(events.iter().all(|e| e.snapshot.issue.is_none()));
    assert!(events.iter().all(|e| e.snapshot.state != ConnectionState::Error));
}

#[test]
fn given_a_catalog_whose_open_is_denied_when_the_module_handles_it_then_the_error_names_permission_and_the_command() {
    let mut hardware = Source::plugged();
    *hardware.hooks.open_error.lock().unwrap() = Some(DeviceError::PermissionDenied);

    let rig = rig(Some(&mut hardware), None);

    rig.wait_for_state(ConnectionState::Error);
    let snapshot = rig.snapshot();
    let issue = snapshot.issue.unwrap();
    assert_eq!(issue.kind, ConnectionIssueKind::PermissionDenied);
    assert!(issue.help.unwrap().install_command.contains("udevadm"));
    assert_eq!(snapshot.source, Some(DeviceSource::Hardware));
}

#[test]
fn given_a_catalog_whose_open_fails_with_io_when_the_module_handles_it_then_the_kind_is_communication() {
    let mut hardware = Source::plugged();
    *hardware.hooks.open_error.lock().unwrap() = Some(DeviceError::Io("usb stalled".into()));

    let rig = rig(Some(&mut hardware), None);

    rig.wait_for_state(ConnectionState::Error);
    assert_eq!(rig.snapshot().issue.unwrap().kind, ConnectionIssueKind::Communication);
}

#[test]
fn given_an_error_state_when_retry_arrives_and_the_open_now_succeeds_then_the_state_is_ready() {
    let mut hardware = Source::plugged();
    *hardware.hooks.open_error.lock().unwrap() = Some(DeviceError::PermissionDenied);
    let rig = rig(Some(&mut hardware), None);
    rig.wait_for_state(ConnectionState::Error);
    *hardware.hooks.open_error.lock().unwrap() = None;

    rig.bus.request(RetryConnection).unwrap();

    rig.wait_for_state(ConnectionState::Ready);
    assert_eq!(rig.snapshot().source, Some(DeviceSource::Hardware));
    assert!(rig.snapshot().issue.is_none());
}

#[test]
fn given_an_error_state_when_the_device_is_replugged_then_the_module_tries_again_without_a_retry() {
    let mut hardware = Source::plugged();
    *hardware.hooks.open_error.lock().unwrap() = Some(DeviceError::PermissionDenied);
    let rig = rig(Some(&mut hardware), None);
    rig.wait_for_state(ConnectionState::Error);
    *hardware.hooks.open_error.lock().unwrap() = None;

    hardware.control.unplug();
    rig.wait_for_state(ConnectionState::Disconnected);
    hardware.control.plug_in();

    rig.wait_for_state(ConnectionState::Ready);
}

#[test]
fn given_a_ready_device_when_retry_arrives_then_the_session_is_kept() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);

    rig.bus.request(RetryConnection).unwrap();
    rig.bus.request(apply(1)).unwrap();

    assert_eq!(simulator.hooks.opened.load(Ordering::SeqCst), 1);
    assert_eq!(rig.snapshot().state, ConnectionState::Ready);
}

#[test]
fn given_an_unplugged_simulator_and_no_hardware_when_use_simulator_is_enabled_then_a_simulator_session_opens() {
    let mut simulator = Source::unplugged();
    let rig = rig(None, Some(&mut simulator));
    assert_eq!(rig.snapshot().state, ConnectionState::Disconnected);

    rig.bus.request(UseSimulator { enabled: true }).unwrap();

    rig.wait_for_state(ConnectionState::Ready);
    assert_eq!(rig.snapshot().source, Some(DeviceSource::Simulator));
    assert!(rig.snapshot().simulator_active);
}

#[test]
fn given_a_running_simulator_session_when_use_simulator_is_disabled_then_the_state_is_disconnected() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);

    rig.bus.request(UseSimulator { enabled: false }).unwrap();

    rig.wait_for_state(ConnectionState::Disconnected);
    assert!(!rig.snapshot().simulator_active);
}

#[test]
fn given_a_build_without_a_simulator_when_use_simulator_arrives_then_it_is_a_recoverable_error() {
    let mut hardware = Source::unplugged();
    let rig = rig(Some(&mut hardware), None);

    let result = rig.bus.request(UseSimulator { enabled: true });

    assert_eq!(result.unwrap_err().category, ErrorCategory::Recoverable);
}

#[test]
fn given_a_simulator_session_when_hardware_is_attached_then_the_simulator_ends_and_the_hardware_opens() {
    let mut hardware = Source::unplugged();
    let mut simulator = Source::plugged();
    let rig = rig(Some(&mut hardware), Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);
    assert_eq!(rig.snapshot().source, Some(DeviceSource::Simulator));

    hardware.control.plug_in();

    assert!(wait_until(|| {
        let snapshot = rig.snapshot();
        snapshot.state == ConnectionState::Ready && snapshot.source == Some(DeviceSource::Hardware)
    }));
    assert!(wait_until(|| simulator.hooks.live_devices.load(Ordering::SeqCst) == 0));
    assert!(rig.snapshot().simulator_active);
}

#[test]
fn given_hardware_and_simulator_both_attached_when_the_module_starts_then_the_hardware_wins() {
    let mut hardware = Source::plugged();
    let mut simulator = Source::plugged();

    let rig = rig(Some(&mut hardware), Some(&mut simulator));

    assert!(wait_until(|| {
        let snapshot = rig.snapshot();
        snapshot.state == ConnectionState::Ready && snapshot.source == Some(DeviceSource::Hardware)
    }));
    assert!(wait_until(|| simulator.hooks.live_devices.load(Ordering::SeqCst) == 0));
}

#[test]
fn given_no_session_when_a_setup_arrives_then_it_returns_a_recoverable_error_naming_the_reason() {
    let mut simulator = Source::unplugged();
    let rig = rig(None, Some(&mut simulator));

    let error: KernelError = rig.bus.request(apply(1)).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert!(error.message.contains("connected"), "{}", error.message);
}

#[test]
fn given_no_session_when_capturing_is_switched_off_then_it_succeeds_and_when_switched_on_it_fails() {
    let mut simulator = Source::unplugged();
    let rig = rig(None, Some(&mut simulator));

    assert_eq!(rig.bus.request(SetCapturing { on: false }), Ok(()));
    assert!(rig.bus.request(SetCapturing { on: true }).is_err());
}

#[test]
fn given_a_started_module_when_it_is_dropped_then_the_worker_is_gone() {
    let mut simulator = Source::plugged();
    let rig = rig(None, Some(&mut simulator));
    rig.wait_for_state(ConnectionState::Ready);

    drop(rig.module);

    assert_eq!(simulator.hooks.live_devices.load(Ordering::SeqCst), 0);
}

#[test]
fn given_a_second_module_on_the_same_bus_when_created_then_the_duplicate_handler_is_refused() {
    let mut first = Source::unplugged();
    let rig = rig(None, Some(&mut first));
    let second = Source::unplugged();

    let result = sweepscope_device::DeviceModule::new(sweepscope_device::DeviceDeps {
        bus: rig.bus.clone(),
        clock: std::sync::Arc::new(sweepscope_kernel::SystemClock),
        hardware: None,
        simulator: Some(sweepscope_device::SimulatorSource {
            catalog: Box::new(sweepscope_device::SimulatedCatalog::new(
                std::sync::Arc::new(sweepscope_kernel::SystemClock),
                second.control.clone(),
            )),
            control: second.control.clone(),
        }),
    });

    assert!(result.is_err());
}
