#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use module_support::{expect_recoverable, ready_rig, Rig};
use sweepscope_acquisition::{Phase, SetChannel, SetRunning, SettingTarget, StepSetting};

fn run(rig: &Rig) {
    rig.bus.request(SetRunning { running: true }).unwrap();
}

fn step_time_base(rig: &Rig) {
    rig.bus
        .request(StepSetting {
            target: SettingTarget::TimeBase,
            direction: 1,
        })
        .unwrap();
}

#[test]
fn given_a_running_device_when_it_is_unplugged_then_running_is_false_and_the_controls_are_unavailable() {
    let rig = ready_rig();
    run(&rig);

    rig.unplug();

    let instrument = rig.snapshot().instrument.unwrap();
    assert!(!instrument.running);
    assert!(!instrument.controls.run.enabled);
    assert!(!instrument.controls.vertical.enabled);
    assert_eq!(rig.snapshot().status.phase, Phase::Stopped);
}

#[test]
fn given_an_unplugged_device_when_a_setting_is_edited_then_it_is_rejected_and_the_last_setup_stays_visible() {
    let rig = ready_rig();
    step_time_base(&rig);
    let shown = rig.snapshot().instrument.unwrap().horizontal.time_base.index;
    rig.unplug();

    let reason = expect_recoverable(rig.bus.request(StepSetting {
        target: SettingTarget::TimeBase,
        direction: 1,
    }));

    assert_eq!(reason, "No scope connected");
    assert_eq!(rig.snapshot().instrument.unwrap().horizontal.time_base.index, shown);
}

#[test]
fn given_a_device_that_is_connecting_when_the_state_is_read_then_the_controls_say_connecting() {
    let rig = ready_rig();

    rig.connecting();

    let controls = rig.snapshot().instrument.unwrap().controls;
    assert_eq!(controls.trigger.reason.as_deref(), Some("Connecting"));
}

#[test]
fn given_a_device_in_error_when_the_state_is_read_then_the_controls_carry_the_error_text() {
    let rig = ready_rig();

    rig.fail("The scope stopped answering");

    let controls = rig.snapshot().instrument.unwrap().controls;
    assert_eq!(
        controls.horizontal.reason.as_deref(),
        Some("The scope stopped answering")
    );
}

#[test]
fn given_a_changed_setup_and_a_running_device_when_it_is_replugged_then_the_same_setup_is_applied_with_a_new_revision_and_running_resumes(
) {
    let rig = ready_rig();
    step_time_base(&rig);
    rig.bus
        .request(SetChannel {
            channel: 2,
            patch: domain_support::patch_enabled(true),
        })
        .unwrap();
    run(&rig);
    let (old_revision, old_setup) = rig.device.applies().pop().unwrap();
    rig.unplug();

    rig.plug();

    let (new_revision, new_setup) = rig.device.applies().pop().unwrap();
    assert_eq!(new_setup, old_setup);
    assert!(new_revision > old_revision);
    rig.wait(|| rig.device.capturing_calls() == vec![true, true]);
    rig.wait(|| rig.snapshot().instrument.is_some_and(|i| i.running));
    assert_eq!(rig.snapshot().status.phase, Phase::Armed);
}

#[test]
fn given_a_stopped_device_when_it_is_replugged_then_the_setup_is_restored_and_acquisition_stays_stopped() {
    let rig = ready_rig();
    step_time_base(&rig);
    rig.unplug();

    rig.plug();

    assert_eq!(rig.device.applies().len(), 3);
    assert!(rig.device.capturing_calls().is_empty());
    assert!(!rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_device_that_failed_mid_capture_when_the_error_arrives_then_running_is_false_and_the_state_stays_readable() {
    let rig = ready_rig();
    run(&rig);

    rig.fail("USB transfer failed");

    let instrument = rig.snapshot().instrument.unwrap();
    assert!(!instrument.running);
    assert_eq!(rig.snapshot().status.phase, Phase::Stopped);
    assert_eq!(instrument.channels.len(), 4);
}

#[test]
fn given_a_frame_of_the_session_before_the_unplug_when_it_arrives_after_the_replug_then_it_is_dropped() {
    let rig = ready_rig();
    run(&rig);
    let before_unplug = rig.setup_revision();
    rig.unplug();
    rig.plug();
    rig.wait(|| rig.snapshot().instrument.is_some_and(|i| i.running));

    rig.complete_triggered(before_unplug);
    rig.settle_after_capture_events();

    let revisions: Vec<u64> = rig.frames.events().iter().map(|f| f.context.setup_revision).collect();
    assert_eq!(revisions, vec![rig.setup_revision()]);
}

#[test]
fn given_a_ready_device_when_the_same_ready_state_is_announced_again_then_the_setup_is_not_applied_again() {
    let rig = ready_rig();
    let applied = rig.device.applies().len();

    rig.announce_ready_again();
    rig.unplug();

    assert_eq!(rig.device.applies().len(), applied);
}

#[test]
fn given_twenty_unplug_and_replug_cycles_when_they_finish_then_the_setup_is_restored_and_every_cycle_resumed() {
    let rig = ready_rig();
    step_time_base(&rig);
    run(&rig);
    let setup = rig.device.applies().pop().unwrap().1;

    for _ in 0..20 {
        rig.unplug();
        rig.plug();
    }

    assert_eq!(rig.device.applies().pop().unwrap().1, setup);
    rig.wait(|| rig.device.capturing_calls().len() == 21);
    rig.wait(|| rig.snapshot().instrument.is_some_and(|i| i.running));
}
