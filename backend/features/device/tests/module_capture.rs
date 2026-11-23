//! Worker loop of the device module: capture publishing, stop, timeouts and failures.
mod module_support;

use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::time::Duration;

use sweepscope_device::{DeviceError, Sweep};
use sweepscope_kernel::contracts::{
    ApplySetup, CaptureCompleted, CaptureOutcome, ConnectionIssueKind, ConnectionState, ForceTrigger, SetCapturing,
};

use module_support::*;

fn ready_rig(source: &mut Source) -> Rig {
    let rig = rig(None, Some(source));
    rig.wait_for_state(ConnectionState::Ready);
    rig
}

fn apply(rig: &Rig, revision: u64, sweep: Sweep, level_volts: f64) {
    rig.bus
        .request(ApplySetup {
            setup_revision: revision,
            setup: setup(sweep, level_volts),
        })
        .unwrap();
}

fn capturing(rig: &Rig, on: bool) {
    rig.bus.request(SetCapturing { on }).unwrap();
}

fn next_completed(rig: &Rig) -> CaptureCompleted {
    rig.completed.recv_timeout(WAIT).expect("a capture completes")
}

#[test]
fn given_a_burst_of_five_setups_when_capturing_starts_then_the_first_capture_carries_the_last_revision() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    for revision in 1..=5 {
        apply(&rig, revision, Sweep::Auto, 0.0);
    }

    capturing(&rig, true);

    let completed = next_completed(&rig);
    assert_eq!(completed.setup_revision, 5);
    assert!(matches!(completed.outcome, CaptureOutcome::Frame(_)));
}

#[test]
fn given_capturing_on_when_frames_arrive_then_the_loop_keeps_publishing() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    apply(&rig, 1, Sweep::Auto, 0.0);

    capturing(&rig, true);

    next_completed(&rig);
    next_completed(&rig);
}

#[test]
fn given_a_setup_without_capturing_when_applied_then_the_device_is_configured_and_no_frame_is_published() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);

    apply(&rig, 1, Sweep::Auto, 0.0);

    assert!(wait_until(|| simulator.hooks.configures.load(Ordering::SeqCst) == 1));
    assert!(rig.completed.recv_timeout(Duration::from_millis(100)).is_none());
}

#[test]
fn given_capturing_on_and_a_capture_in_flight_when_capturing_is_switched_off_then_the_finished_frame_is_not_published()
{
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    let (entered, entered_rx) = channel();
    let (release_tx, release) = channel();
    *simulator.hooks.gate.lock().unwrap() = Some(Gate { entered, release });
    apply(&rig, 1, Sweep::Auto, 0.0);
    capturing(&rig, true);
    entered_rx.recv_timeout(WAIT).expect("the worker is inside capture");

    capturing(&rig, false);
    release_tx.send(()).unwrap();
    apply(&rig, 2, Sweep::Auto, 0.0);

    assert!(wait_until(|| simulator.hooks.configures.load(Ordering::SeqCst) == 2));
    assert!(rig.completed.try_next().is_none());
}

#[test]
fn given_normal_sweep_and_no_trigger_when_a_capture_times_out_then_timed_out_is_published_and_the_loop_continues() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    apply(&rig, 1, Sweep::Normal, 50.0);

    capturing(&rig, true);

    assert_eq!(next_completed(&rig).outcome, CaptureOutcome::TimedOut);
    assert_eq!(next_completed(&rig).outcome, CaptureOutcome::TimedOut);
    assert_eq!(rig.snapshot().state, ConnectionState::Ready);
}

#[test]
fn given_capturing_when_the_device_is_lost_mid_capture_then_the_state_is_disconnected() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    *simulator.hooks.capture_error.lock().unwrap() = Some(DeviceError::Disconnected);
    apply(&rig, 1, Sweep::Auto, 0.0);

    capturing(&rig, true);

    rig.wait_for_state(ConnectionState::Disconnected);
    assert!(wait_until(|| simulator.hooks.live_devices.load(Ordering::SeqCst) == 0));
    assert!(rig.snapshot().issue.is_none());
}

#[test]
fn given_capturing_when_another_error_happens_then_the_state_is_error_and_capturing_stops() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    *simulator.hooks.capture_error.lock().unwrap() = Some(DeviceError::Io("short read".into()));
    apply(&rig, 1, Sweep::Auto, 0.0);

    capturing(&rig, true);

    rig.wait_for_state(ConnectionState::Error);
    assert_eq!(rig.snapshot().issue.unwrap().kind, ConnectionIssueKind::Communication);
    while rig.completed.try_next().is_some() {}
    assert!(rig.completed.recv_timeout(Duration::from_millis(150)).is_none());
}

#[test]
fn given_a_session_that_failed_when_retry_arrives_then_the_device_is_opened_again_and_ready() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    *simulator.hooks.capture_error.lock().unwrap() = Some(DeviceError::Io("short read".into()));
    apply(&rig, 1, Sweep::Auto, 0.0);
    capturing(&rig, true);
    rig.wait_for_state(ConnectionState::Error);

    rig.bus.request(sweepscope_device::RetryConnection).unwrap();

    rig.wait_for_state(ConnectionState::Ready);
    assert_eq!(simulator.hooks.opened.load(Ordering::SeqCst), 2);
    assert!(wait_until(|| simulator.hooks.live_devices.load(Ordering::SeqCst) == 1));
}

#[test]
fn given_capturing_in_normal_sweep_without_trigger_when_forced_then_a_frame_is_published() {
    let mut simulator = Source::plugged();
    let rig = ready_rig(&mut simulator);
    apply(&rig, 1, Sweep::Normal, 50.0);
    capturing(&rig, true);
    assert_eq!(next_completed(&rig).outcome, CaptureOutcome::TimedOut);

    rig.bus.request(ForceTrigger).unwrap();

    let frames = std::iter::from_fn(|| rig.completed.recv_timeout(WAIT))
        .take(3)
        .any(|completed| matches!(completed.outcome, CaptureOutcome::Frame(_)));
    assert!(frames);
}
