#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use std::time::Duration;

use module_support::{ready_rig, Rig};
use sweepscope_acquisition::{Phase, SetChannel, SetRunning, SettingTarget, StepSetting};
use sweepscope_kernel::contracts::CaptureOutcome;
use sweepscope_kernel::testing::eventually;

fn run(rig: &Rig) -> u64 {
    rig.bus.request(SetRunning { running: true }).unwrap();
    rig.setup_revision()
}

fn phases(rig: &Rig) -> Vec<Phase> {
    rig.status.events().iter().map(|e| e.0.phase).collect()
}

fn wait_for_frames(rig: &Rig, count: usize) {
    rig.wait(|| rig.frames.events().len() >= count);
}

#[test]
fn given_a_ready_device_when_run_and_a_triggered_frame_arrives_then_the_status_goes_armed_then_triggered() {
    let rig = ready_rig();
    let revision = run(&rig);

    rig.complete_triggered(revision);
    wait_for_frames(&rig, 1);

    assert_eq!(phases(&rig), vec![Phase::Armed, Phase::Triggered]);
}

#[test]
fn given_a_running_device_when_an_auto_frame_arrives_then_the_status_is_auto() {
    let rig = ready_rig();
    let revision = run(&rig);

    rig.complete_auto(revision);
    wait_for_frames(&rig, 1);

    assert_eq!(phases(&rig), vec![Phase::Armed, Phase::Auto]);
}

#[test]
fn given_a_running_device_when_a_capture_times_out_then_the_status_waits_for_a_trigger_and_no_frame_is_published() {
    let rig = ready_rig();
    let revision = run(&rig);

    rig.complete(revision, CaptureOutcome::TimedOut);
    rig.wait(|| phases(&rig).contains(&Phase::WaitingForTrigger));

    assert!(rig.frames.events().is_empty());
}

#[test]
fn given_two_frames_with_the_same_outcome_when_they_arrive_then_the_status_is_published_once() {
    let rig = ready_rig();
    let revision = run(&rig);

    rig.complete_triggered(revision);
    rig.complete_triggered(revision);
    wait_for_frames(&rig, 2);

    assert_eq!(phases(&rig), vec![Phase::Armed, Phase::Triggered]);
}

#[test]
fn given_a_frame_when_it_is_published_then_it_carries_the_context_of_the_current_setup() {
    let rig = ready_rig();
    let patch = sweepscope_acquisition::ChannelPatch {
        invert: Some(true),
        ..Default::default()
    };
    rig.bus.request(SetChannel { channel: 1, patch }).unwrap();
    let revision = run(&rig);

    rig.complete_triggered(revision);
    wait_for_frames(&rig, 1);

    let context = rig.frames.events()[0].context;
    assert_eq!(context.setup_revision, revision);
    assert_eq!(context.window_len, 2501);
    assert_eq!(context.window_start, 798);
    assert_eq!(context.invert, [true, false, false, false]);
}

#[test]
fn given_a_setup_change_when_a_frame_of_the_older_revision_arrives_then_it_is_not_published() {
    let rig = ready_rig();
    let old = run(&rig);
    rig.bus
        .request(SetChannel {
            channel: 2,
            patch: domain_support::patch_enabled(true),
        })
        .unwrap();

    rig.complete_triggered(old);
    rig.settle_after_capture_events();

    let revisions: Vec<u64> = rig.frames.events().iter().map(|f| f.context.setup_revision).collect();
    assert_eq!(revisions, vec![rig.setup_revision()]);
    assert!(rig.setup_revision() > old);
}

#[test]
fn given_a_stopped_device_when_a_late_frame_arrives_then_it_is_not_published_and_the_status_stays_stopped() {
    let rig = ready_rig();
    let revision = run(&rig);
    rig.bus.request(SetRunning { running: false }).unwrap();

    rig.complete_triggered(revision);

    assert!(
        !eventually(Duration::from_millis(150), || !rig.frames.events().is_empty()),
        "a frame was published while stopped"
    );
    assert_eq!(rig.snapshot().status.phase, Phase::Stopped);
}

#[test]
fn given_a_run_when_frames_arrive_then_the_first_facts_come_with_the_run_and_the_rate_is_held_back_for_250_ms() {
    let rig = ready_rig();
    let revision = run(&rig);
    rig.complete_triggered(revision);
    rig.clock.advance(Duration::from_millis(10));
    rig.complete_triggered(revision);
    wait_for_frames(&rig, 2);

    let held_back: Vec<bool> = rig.facts.events().iter().map(|f| f.0.capture_rate.is_some()).collect();

    assert!(held_back.iter().all(|has_rate| !has_rate), "{held_back:?}");
}

#[test]
fn given_frames_every_50_ms_when_250_ms_have_passed_then_the_published_capture_rate_is_20_per_second() {
    let rig = ready_rig();
    let revision = run(&rig);
    for frame in 1..=8 {
        rig.complete_triggered(revision);
        wait_for_frames(&rig, frame);
        rig.clock.advance(Duration::from_millis(50));
    }

    rig.wait(|| {
        rig.facts.events().iter().any(|f| {
            f.0.capture_rate
                .as_ref()
                .is_some_and(|rate| (rate.value - 20.0).abs() < 0.5)
        })
    });
}

#[test]
fn given_a_setup_change_when_the_facts_follow_then_they_are_published_at_once_with_the_new_sample_rate() {
    let rig = ready_rig();
    let before = rig.snapshot().facts.sample_rate.unwrap().value;

    rig.bus
        .request(StepSetting {
            target: SettingTarget::TimeBase,
            direction: -1,
        })
        .unwrap();

    let after = rig.snapshot().facts.sample_rate.unwrap().value;
    assert!(after > before, "{after} should be above {before}");
    assert_eq!(
        rig.facts.events().last().unwrap().0.sample_rate.as_ref().unwrap().value,
        after
    );
}

#[test]
fn given_facts_events_when_they_are_read_then_their_revisions_grow_and_the_snapshot_equals_the_last_event() {
    let rig = ready_rig();
    rig.bus
        .request(SetChannel {
            channel: 2,
            patch: domain_support::patch_enabled(true),
        })
        .unwrap();

    let events = rig.facts.events();

    let revisions: Vec<u64> = events.iter().map(|e| e.0.revision).collect();
    assert!(revisions.windows(2).all(|w| w[0] < w[1]), "{revisions:?}");
    assert_eq!(rig.snapshot().facts, events.last().unwrap().0);
}
