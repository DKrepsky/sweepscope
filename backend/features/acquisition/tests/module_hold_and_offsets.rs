#[path = "domain_support/mod.rs"]
mod domain_support;
mod module_support;

use std::sync::Arc;

use domain_support::{capabilities, range_index};
use module_support::{expect_recoverable, frame, ready_rig, Rig, UNIT};
use sweepscope_acquisition::{ArmSingle, SetRunning, SettingTarget, StartAutoset, StepSetting};
use sweepscope_kernel::contracts::{
    AcquisitionSetup, CaptureOutcome, CaptureProbe, Frame, HoldInstrument, ProbeCaptured, RollItem, TriggerOutcome,
    ZeroOffsets, ZeroOffsetsChanged,
};
use sweepscope_kernel::testing::EventRecorder;

const REASON: &str = "Zero calibration is open";
const OFFSET_V: f64 = 0.5;

fn hold(rig: &Rig, on: bool) -> Result<(), sweepscope_kernel::KernelError> {
    rig.bus.request(HoldInstrument {
        on,
        reason: REASON.into(),
    })
}

fn run(rig: &Rig) {
    rig.bus.request(SetRunning { running: true }).unwrap();
}

fn offsets_for(unit: &str, channel_one_volts: f64) -> ZeroOffsets {
    let range = usize::from(range_index(&capabilities(), 1.0));
    let mut per_range = vec![0.0; range + 1];
    per_range[range] = channel_one_volts;
    ZeroOffsets {
        unit_id: unit.into(),
        volts: [per_range, vec![], vec![], vec![]],
    }
}

/// A nonzero offset on the trigger source changes the setup, so the new application proves the event was handled.
fn publish_offsets(rig: &Rig, offsets: Option<ZeroOffsets>) {
    let before = rig.device.applies().len();
    rig.bus.publish(ZeroOffsetsChanged { offsets });
    rig.wait(|| rig.device.applies().len() > before);
}

fn probe_setup(rig: &Rig) -> AcquisitionSetup {
    rig.device.applies().last().unwrap().1.clone()
}

fn probe(rig: &Rig, id: u64, discard: u8, keep: u8) -> Result<(), sweepscope_kernel::KernelError> {
    rig.bus.request(CaptureProbe {
        id,
        setup: probe_setup(rig),
        discard,
        keep,
    })
}

fn probe_frame(marker: u8) -> CaptureOutcome {
    let mut f = frame(TriggerOutcome::Auto);
    f.channels[0].as_mut().unwrap().codes = vec![marker; 4];
    CaptureOutcome::Frame(Arc::new(f))
}

fn zero_code_of(frame: &Frame) -> f64 {
    frame.channels[0].as_ref().unwrap().zero_code
}

fn last_zero_code(rig: &Rig) -> f64 {
    zero_code_of(&rig.frames.events().last().unwrap().frame)
}

fn two_frames(rig: &Rig) {
    let seen = rig.frames.events().len();
    rig.complete_auto(rig.setup_revision());
    rig.complete_auto(rig.setup_revision());
    rig.wait(|| rig.frames.events().len() >= seen + 2);
}

fn probes(rig: &Rig) -> EventRecorder<ProbeCaptured> {
    EventRecorder::new(&rig.bus)
}

#[test]
fn given_a_running_scope_when_held_then_setup_and_run_commands_fail_with_the_reason() {
    let rig = ready_rig();
    run(&rig);

    hold(&rig, true).unwrap();

    let step = rig.bus.request(StepSetting {
        target: SettingTarget::TimeBase,
        direction: 1,
    });
    assert_eq!(expect_recoverable(step), REASON);
    assert_eq!(
        expect_recoverable(rig.bus.request(SetRunning { running: true })),
        REASON
    );
    assert_eq!(expect_recoverable(rig.bus.request(ArmSingle)), REASON);
    assert_eq!(expect_recoverable(rig.bus.request(StartAutoset)), REASON);
}

#[test]
fn given_a_held_scope_when_the_state_is_read_then_the_controls_carry_the_reason() {
    let rig = ready_rig();

    hold(&rig, true).unwrap();

    let controls = rig.snapshot().instrument.unwrap().controls;
    assert_eq!(controls.run.reason.as_deref(), Some(REASON));
    assert_eq!(controls.vertical.reason.as_deref(), Some(REASON));
    assert!(!controls.autoset.enabled);
}

#[test]
fn given_a_running_scope_when_held_then_capturing_stops() {
    let rig = ready_rig();
    run(&rig);

    hold(&rig, true).unwrap();

    assert_eq!(rig.device.capturing_calls(), vec![true, false]);
    assert!(!rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_running_scope_when_held_and_released_then_the_model_returns_with_a_new_revision_and_capturing_resumes() {
    let rig = ready_rig();
    run(&rig);
    let applied_before = rig.device.applies();

    hold(&rig, true).unwrap();
    hold(&rig, false).unwrap();

    let applied = rig.device.applies();
    assert_eq!(applied.len(), applied_before.len() + 1);
    assert!(applied.last().unwrap().0 > applied_before.last().unwrap().0);
    assert_eq!(applied.last().unwrap().1, applied_before.last().unwrap().1);
    assert_eq!(rig.device.capturing_calls(), vec![true, false, true]);
    assert!(rig.snapshot().instrument.unwrap().running);
}

#[test]
fn given_a_stopped_scope_when_held_and_released_then_it_stays_stopped() {
    let rig = ready_rig();

    hold(&rig, true).unwrap();
    hold(&rig, false).unwrap();

    assert!(rig.device.capturing_calls().is_empty());
    let state = rig.snapshot().instrument.unwrap();
    assert!(!state.running);
    assert!(state.controls.run.enabled);
}

#[test]
fn given_an_armed_single_when_held_and_released_then_the_single_is_cancelled_and_not_rearmed() {
    let rig = ready_rig();
    rig.bus.request(ArmSingle).unwrap();

    hold(&rig, true).unwrap();
    hold(&rig, false).unwrap();

    assert_eq!(rig.device.capturing_calls(), vec![true, false]);
    let state = rig.snapshot().instrument.unwrap();
    assert!(!state.running);
    let last = rig.device.applies().pop().unwrap().1;
    assert_ne!(last.trigger.sweep, sweepscope_kernel::contracts::Sweep::Normal);
}

#[test]
fn given_a_running_autoset_when_held_then_the_request_is_refused_with_its_reason() {
    let rig = ready_rig();
    rig.bus.request(StartAutoset).unwrap();

    let refused = hold(&rig, true);

    assert_eq!(expect_recoverable(refused), "Auto-setup is running");
}

#[test]
fn given_no_scope_when_held_then_the_request_is_refused() {
    let rig = module_support::rig();

    assert!(hold(&rig, true).is_err());
}

#[test]
fn given_a_scope_that_is_not_held_when_a_probe_is_requested_then_it_is_refused() {
    let rig = ready_rig();

    let refused = probe(&rig, 1, 0, 1);

    assert_eq!(expect_recoverable(refused), "The instrument is not held");
}

#[test]
fn given_a_held_scope_when_a_probe_is_requested_then_its_setup_is_applied_with_a_fresh_revision_and_capturing_starts() {
    let rig = ready_rig();
    hold(&rig, true).unwrap();
    let before = rig.device.applies().last().unwrap().0;

    probe(&rig, 7, 1, 3).unwrap();

    assert!(rig.device.applies().last().unwrap().0 > before);
    assert_eq!(rig.device.capturing_calls(), vec![true]);
}

#[test]
fn given_a_probe_with_one_discard_and_two_kept_when_four_frames_arrive_then_the_last_two_are_answered_once() {
    let rig = ready_rig();
    let answers = probes(&rig);
    hold(&rig, true).unwrap();
    probe(&rig, 7, 1, 2).unwrap();
    let revision = rig.setup_revision();

    for marker in [1, 2, 3] {
        rig.complete(revision, probe_frame(marker));
    }
    rig.wait(|| !answers.events().is_empty());

    let answer = &answers.events()[0];
    assert_eq!(answer.id, 7);
    let codes: Vec<u8> = answer
        .frames
        .iter()
        .map(|f| f.channels[0].as_ref().unwrap().codes[0])
        .collect();
    assert_eq!(codes, vec![2, 3]);
}

#[test]
fn given_a_probe_when_its_frames_arrive_then_none_is_published_to_the_display() {
    let rig = ready_rig();
    let answers = probes(&rig);
    hold(&rig, true).unwrap();
    probe(&rig, 1, 0, 1).unwrap();

    rig.complete(rig.setup_revision(), probe_frame(9));
    rig.wait(|| !answers.events().is_empty());

    assert!(rig.frames.events().is_empty());
}

#[test]
fn given_a_probe_when_the_capture_times_out_then_the_answer_has_no_frames() {
    let rig = ready_rig();
    let answers = probes(&rig);
    hold(&rig, true).unwrap();
    probe(&rig, 3, 0, 2).unwrap();

    rig.complete(rig.setup_revision(), CaptureOutcome::TimedOut);
    rig.wait(|| !answers.events().is_empty());

    assert_eq!((answers.events()[0].id, answers.events()[0].frames.len()), (3, 0));
}

#[test]
fn given_a_probe_when_a_frame_of_an_older_revision_arrives_then_it_is_ignored() {
    let rig = ready_rig();
    let answers = probes(&rig);
    hold(&rig, true).unwrap();
    probe(&rig, 1, 0, 1).unwrap();
    let current = rig.setup_revision();

    rig.complete(current - 1, probe_frame(1));
    rig.complete(current, probe_frame(2));
    rig.wait(|| !answers.events().is_empty());

    let codes = answers.events()[0].frames[0].channels[0].as_ref().unwrap().codes[0];
    assert_eq!(codes, 2);
}

#[test]
fn given_a_pending_probe_when_the_hold_is_released_then_the_probe_fails() {
    let rig = ready_rig();
    let answers = probes(&rig);
    hold(&rig, true).unwrap();
    probe(&rig, 5, 0, 1).unwrap();

    hold(&rig, false).unwrap();

    assert_eq!((answers.events()[0].id, answers.events()[0].frames.len()), (5, 0));
}

#[test]
fn given_offsets_of_the_connected_unit_when_a_frame_is_published_then_its_zero_code_moves_by_the_offset_in_codes() {
    let rig = ready_rig();
    run(&rig);
    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));

    rig.complete_auto(rig.setup_revision());
    rig.wait(|| !rig.frames.events().is_empty());

    assert_eq!(last_zero_code(&rig), 128.0 + OFFSET_V / 1.0 * 25.0);
    assert_eq!(rig.frames.events()[0].context.calibrated, [true, false, false, false]);
}

#[test]
fn given_a_frame_with_one_count_per_division_scale_when_offsets_apply_then_its_own_counts_per_division_are_used() {
    let rig = ready_rig();
    run(&rig);
    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));
    let mut single_channel = frame(TriggerOutcome::Auto);
    single_channel.counts_per_div = 21.1;

    rig.complete(rig.setup_revision(), CaptureOutcome::Frame(Arc::new(single_channel)));
    rig.wait(|| !rig.frames.events().is_empty());

    assert_eq!(last_zero_code(&rig), 128.0 + OFFSET_V * 21.1);
}

#[test]
fn given_offsets_of_another_unit_when_a_frame_is_published_then_it_is_left_as_captured() {
    let rig = ready_rig();
    run(&rig);
    rig.bus.publish(ZeroOffsetsChanged {
        offsets: Some(offsets_for("other/unit/9", OFFSET_V)),
    });

    two_frames(&rig);

    assert_eq!(last_zero_code(&rig), 128.0);
    assert_eq!(rig.frames.events().last().unwrap().context.calibrated, [false; 4]);
}

#[test]
fn given_offsets_when_they_are_withdrawn_then_frames_are_left_as_captured_again() {
    let rig = ready_rig();
    run(&rig);
    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));
    publish_offsets(&rig, None);

    rig.complete_auto(rig.setup_revision());
    rig.wait(|| !rig.frames.events().is_empty());

    assert_eq!(last_zero_code(&rig), 128.0);
}

#[test]
fn given_a_published_frame_when_offsets_arrive_then_the_held_frame_is_published_again_corrected() {
    let rig = ready_rig();
    run(&rig);
    rig.complete_auto(rig.setup_revision());
    rig.wait(|| rig.frames.events().len() == 1);

    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));
    rig.wait(|| rig.frames.events().len() == 2);

    assert_eq!(zero_code_of(&rig.frames.events()[0].frame), 128.0);
    assert_eq!(last_zero_code(&rig), 128.0 + OFFSET_V * 25.0);
    assert!(rig.frames.events()[1].context.calibrated[0]);
}

#[test]
fn given_offsets_when_a_roll_item_is_published_then_its_samples_are_corrected() {
    let rig = ready_rig();
    rig.step_time_base_until_roll(true);
    run(&rig);
    rig.wait(|| !rig.roll_items().is_empty());
    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));
    rig.wait(|| rig.roll_items().len() >= 3);
    let revision = rig.setup_revision();

    rig.poll_samples(revision, 0);
    rig.wait(|| {
        rig.roll_items()
            .iter()
            .any(|c| matches!(c.item, RollItem::Samples { .. }))
    });

    let samples = rig
        .roll_items()
        .into_iter()
        .find_map(|c| match c.item {
            RollItem::Samples { frame, .. } => Some(frame),
            _ => None,
        })
        .unwrap();
    assert_eq!(zero_code_of(&samples), 128.0 + OFFSET_V * 25.0);
}

#[test]
fn given_offsets_when_they_arrive_then_the_trigger_level_sent_to_the_device_moves_by_the_offset_of_source_and_range() {
    let rig = ready_rig();
    let before = rig.device.applies().last().unwrap().1.trigger.level_volts;

    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));

    let after = rig.device.applies().last().unwrap().1.trigger.level_volts;
    assert_eq!(after, before + OFFSET_V);
}

#[test]
fn given_offsets_when_the_user_level_is_read_then_it_is_not_shifted() {
    let rig = ready_rig();

    publish_offsets(&rig, Some(offsets_for(UNIT, OFFSET_V)));

    let level = rig.snapshot().instrument.unwrap().trigger.level_divisions;
    assert_eq!(level, 0.0);
}
