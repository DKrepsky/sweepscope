mod zero_module_support;

use std::sync::Arc;
use std::time::Duration;

use sweepscope_calibration::domain::offsets::FILE_KEY;
use sweepscope_calibration::domain::rules::{CHANNEL_TIMEOUT, PROBE_TIMEOUT};
use sweepscope_calibration::ZeroStep;
use sweepscope_kernel::testing::MemoryStorage;
use sweepscope_kernel::Storage;
use zero_module_support::{ReadOnlyStorage, Rig, UNIT};

fn connected_rig() -> Rig {
    let rig = Rig::new();
    rig.connect(Some(UNIT));
    rig
}

fn unanswered_rig_measuring() -> Rig {
    let rig = connected_rig();
    rig.acquisition.behaviour.lock().unwrap().answers = false;
    rig.open().unwrap();
    rig.start().unwrap();
    rig.measure().unwrap();
    rig
}

#[test]
fn given_a_channel_being_measured_when_the_device_is_lost_then_the_session_fails_and_stores_nothing() {
    let rig = unanswered_rig_measuring();

    rig.disconnect();
    let session = rig.until_step(ZeroStep::Failed).session.unwrap();

    assert!(session.failure.unwrap().contains("disconnected"));
    assert_eq!(rig.storage.read(FILE_KEY).unwrap(), None);
}

#[test]
fn given_a_failed_session_when_it_is_closed_then_the_hold_is_released() {
    let rig = unanswered_rig_measuring();
    rig.disconnect();
    rig.until_step(ZeroStep::Failed);

    rig.close().unwrap();

    assert_eq!(rig.acquisition.hold_requests(), vec![true, false]);
}

#[test]
fn given_a_done_session_when_the_device_is_lost_then_the_result_stays_done() {
    let rig = connected_rig();
    rig.calibrate_channels(4);

    rig.disconnect();
    rig.until(|s| !s.availability.enabled);

    assert_eq!(rig.step(), Some(ZeroStep::Done));
}

#[test]
fn given_a_probe_that_is_never_answered_when_the_probe_timeout_passes_then_the_session_fails() {
    let rig = unanswered_rig_measuring();

    rig.clock.advance(PROBE_TIMEOUT + Duration::from_millis(1));
    let session = rig.until_step(ZeroStep::Failed).session.unwrap();

    assert!(session.failure.unwrap().contains("did not answer"));
}

#[test]
fn given_a_probe_answered_empty_when_it_arrives_then_the_session_fails() {
    let rig = unanswered_rig_measuring();
    let id = rig.acquisition.probe_requests()[0].id;

    rig.bus
        .publish(sweepscope_kernel::contracts::ProbeCaptured { id, frames: vec![] });

    assert!(rig.until_step(ZeroStep::Failed).session.unwrap().failure.is_some());
}

#[test]
fn given_a_probe_for_another_request_when_it_arrives_then_the_wait_goes_on() {
    let rig = unanswered_rig_measuring();
    let id = rig.acquisition.probe_requests()[0].id;

    rig.bus.publish(sweepscope_kernel::contracts::ProbeCaptured {
        id: id + 100,
        frames: vec![],
    });
    rig.clock.advance(PROBE_TIMEOUT + Duration::from_millis(1));

    let session = rig.until_step(ZeroStep::Failed).session.unwrap();
    assert!(session.failure.unwrap().contains("did not answer"));
}

#[test]
fn given_a_channel_in_progress_when_the_channel_timeout_passes_then_the_session_fails_with_that_reason() {
    let rig = unanswered_rig_measuring();

    rig.clock.advance(CHANNEL_TIMEOUT + Duration::from_millis(1));
    let session = rig.until_step(ZeroStep::Failed).session.unwrap();

    assert!(session.failure.unwrap().contains("too long"));
}

#[test]
fn given_a_failing_store_when_the_fourth_channel_is_accepted_then_the_session_fails_and_nothing_is_announced() {
    let memory = MemoryStorage::default();
    let rig = Rig::build(memory.clone(), Arc::new(ReadOnlyStorage(memory)));
    rig.connect(Some(UNIT));

    rig.open().unwrap();
    rig.start().unwrap();
    for _ in 0..4 {
        rig.measure().unwrap();
        rig.until(|s| {
            let step = s.session.as_ref().map(|x| x.step);
            matches!(step, Some(ZeroStep::Channel | ZeroStep::Failed))
        });
    }

    let session = rig.state().session.unwrap();
    assert_eq!(session.step, ZeroStep::Failed);
    assert!(session.failure.unwrap().contains("could not be saved"));
    assert!(rig.offsets.events().iter().all(|e| e.offsets.is_none()));
    assert!(!rig.state().unit.unwrap().calibrated);
}

#[test]
fn given_a_scope_without_a_serial_when_connected_then_calibration_is_unavailable_and_open_is_refused() {
    let rig = Rig::new();
    rig.connect(None);

    let refused = rig.open().unwrap_err();

    assert_eq!(
        rig.state().availability.reason.as_deref(),
        Some("This scope reports no serial number")
    );
    assert_eq!(refused.message, "This scope reports no serial number");
    assert!(rig.acquisition.hold_requests().is_empty());
}

#[test]
fn given_no_device_when_the_dialog_is_opened_then_it_is_refused_with_no_device() {
    let rig = Rig::new();

    let refused = rig.open().unwrap_err();

    assert_eq!(refused.message, "No device");
}

#[test]
fn given_a_refused_hold_when_the_dialog_is_opened_then_the_reason_is_returned_and_shown_and_no_session_starts() {
    let rig = connected_rig();
    rig.acquisition.behaviour.lock().unwrap().refuse_hold = Some("Auto-setup is running".into());

    let refused = rig.open().unwrap_err();

    assert_eq!(refused.message, "Auto-setup is running");
    let state = rig.state();
    assert_eq!(state.availability.reason.as_deref(), Some("Auto-setup is running"));
    assert_eq!(state.session, None);
}

#[test]
fn given_a_shown_refusal_when_a_few_seconds_pass_then_calibration_is_available_again() {
    let rig = connected_rig();
    rig.acquisition.behaviour.lock().unwrap().refuse_hold = Some("Auto-setup is running".into());
    rig.open().unwrap_err();

    rig.clock.advance(Duration::from_secs(4));
    let state = rig.until(|s| s.availability.enabled);

    assert_eq!(state.availability.reason, None);
}

#[test]
fn given_no_session_when_a_step_command_arrives_then_it_is_refused() {
    let rig = connected_rig();

    assert!(rig.start().is_err());
    assert!(rig.measure().is_err());
}

#[test]
fn given_the_intro_when_a_channel_is_measured_then_the_step_is_refused() {
    let rig = connected_rig();
    rig.open().unwrap();

    assert!(rig.measure().is_err());
    assert_eq!(rig.step(), Some(ZeroStep::Intro));
}

#[test]
fn given_no_session_when_close_arrives_then_nothing_happens() {
    let rig = connected_rig();

    rig.close().unwrap();

    assert!(rig.acquisition.hold_requests().is_empty());
}

#[test]
fn given_an_open_dialog_when_it_is_opened_again_then_the_instrument_is_held_once() {
    let rig = connected_rig();
    rig.open().unwrap();

    rig.open().unwrap();

    assert_eq!(rig.acquisition.hold_requests(), vec![true]);
}
