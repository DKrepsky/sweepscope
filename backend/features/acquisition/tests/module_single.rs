#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use module_support::{expect_recoverable, ready_rig, DeviceCall, Rig};
use sweepscope_acquisition::{ArmSingle, ForceTriggerNow, Phase, SetRunning};
use sweepscope_kernel::contracts::{CaptureOutcome, Sweep};

fn phases(rig: &Rig) -> Vec<Phase> {
    rig.status.events().iter().map(|e| e.0.phase).collect()
}

fn armed() -> Rig {
    let rig = ready_rig();
    rig.bus.request(ArmSingle).unwrap();
    rig
}

fn published_sweep(rig: &Rig) -> Sweep {
    rig.snapshot().instrument.unwrap().trigger.sweep
}

fn last_sent_sweep(rig: &Rig) -> Sweep {
    rig.device.applies().last().unwrap().1.trigger.sweep
}

#[test]
fn given_a_ready_scope_when_a_single_is_armed_then_the_phase_waits_for_a_trigger_and_capturing_is_on() {
    let rig = armed();

    assert_eq!(phases(&rig), [Phase::WaitingForTrigger]);
    assert_eq!(rig.device.capturing_calls(), [true]);
    assert!(rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_an_armed_single_when_the_setup_is_sent_then_the_sweep_is_normal_and_the_published_sweep_is_unchanged() {
    let rig = armed();

    assert_eq!(last_sent_sweep(&rig), Sweep::Normal);
    assert_eq!(published_sweep(&rig), Sweep::Auto);
}

#[test]
fn given_an_armed_single_when_the_first_triggered_frame_arrives_then_one_frame_is_published_and_the_run_is_done() {
    let rig = armed();

    rig.complete_triggered(rig.setup_revision());
    rig.wait(|| phases(&rig).contains(&Phase::Done));

    assert_eq!(rig.frames.events().len(), 1);
    assert_eq!(phases(&rig), [Phase::WaitingForTrigger, Phase::Done]);
    assert_eq!(rig.device.capturing_calls(), [true, false]);
    assert!(!rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_finished_single_when_the_setup_is_sent_again_then_the_sweep_is_restored() {
    let rig = armed();

    rig.complete_triggered(rig.setup_revision());
    rig.wait(|| phases(&rig).contains(&Phase::Done));

    assert_eq!(last_sent_sweep(&rig), Sweep::Auto);
}

#[test]
fn given_a_finished_single_when_more_frames_of_the_old_revision_arrive_then_none_is_published() {
    let rig = armed();
    let revision = rig.setup_revision();
    rig.complete_triggered(revision);
    rig.wait(|| phases(&rig).contains(&Phase::Done));

    rig.complete_triggered(revision);
    rig.complete_triggered(rig.setup_revision());
    rig.unplug();

    assert_eq!(rig.frames.events().len(), 1);
}

#[test]
fn given_an_armed_single_when_a_capture_times_out_then_it_keeps_waiting_and_no_frame_is_published() {
    let rig = armed();

    rig.complete(rig.setup_revision(), CaptureOutcome::TimedOut);
    rig.unplug();

    assert!(rig.frames.events().is_empty());
    assert_eq!(phases(&rig), [Phase::WaitingForTrigger, Phase::Stopped]);
}

#[test]
fn given_an_armed_single_when_forced_then_the_free_run_frame_ends_it_with_done() {
    let rig = armed();

    rig.bus.request(ForceTriggerNow).unwrap();
    rig.complete_auto(rig.setup_revision());
    rig.wait(|| phases(&rig).contains(&Phase::Done));

    assert_eq!(rig.device.forces(), 1);
    assert_eq!(rig.frames.events().len(), 1);
}

#[test]
fn given_a_done_single_when_run_is_pressed_then_continuous_acquisition_resumes() {
    let rig = armed();
    rig.complete_triggered(rig.setup_revision());
    rig.wait(|| phases(&rig).contains(&Phase::Done));

    rig.bus.request(SetRunning { running: true }).unwrap();

    assert_eq!(phases(&rig).last(), Some(&Phase::Armed));
    assert_eq!(rig.device.capturing_calls(), [true, false, true]);
    assert_eq!(last_sent_sweep(&rig), Sweep::Auto);
}

#[test]
fn given_an_armed_single_when_stopped_then_the_sweep_is_restored_and_capturing_is_off() {
    let rig = armed();

    rig.bus.request(SetRunning { running: false }).unwrap();

    assert_eq!(last_sent_sweep(&rig), Sweep::Auto);
    assert_eq!(rig.device.capturing_calls(), [true, false]);
    assert_eq!(phases(&rig).last(), Some(&Phase::Stopped));
}

#[test]
fn given_an_armed_single_when_run_is_pressed_then_it_becomes_continuous_with_the_sweep_restored() {
    let rig = armed();

    rig.bus.request(SetRunning { running: true }).unwrap();

    assert_eq!(last_sent_sweep(&rig), Sweep::Auto);
    assert_eq!(phases(&rig).last(), Some(&Phase::Armed));
    assert!(rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_roll_time_base_when_a_single_is_armed_then_it_is_rejected_with_the_roll_reason() {
    let rig = ready_rig();
    rig.step_time_base_until_roll(true);

    let reason = expect_recoverable(rig.bus.request(ArmSingle));

    assert_eq!(reason, "Single needs a captured record; not available in roll mode.");
    assert!(!rig.device.calls().contains(&DeviceCall::Capturing(true)));
}

#[test]
fn given_no_scope_when_a_single_is_armed_then_it_is_rejected_with_the_connection_text() {
    let rig = module_support::rig();

    let reason = expect_recoverable(rig.bus.request(ArmSingle));

    assert_eq!(reason, "No scope connected");
}
