mod zero_module_support;

use std::time::Duration;

use sweepscope_calibration::domain::offsets::OffsetFile;
use sweepscope_calibration::ZeroStep;
use zero_module_support::{Rig, COUNTS, RANGES, UNIT};

const GATE_RANGE_INDEX: u8 = 5;
const WRONG_STEP: &str = "That is not possible at this step of the zero calibration";

fn stored_volts(rig: &Rig) -> Option<[Vec<f64>; 4]> {
    OffsetFile::load(&rig.storage)
        .unwrap()
        .unit(UNIT)
        .map(|unit| unit.volts.clone())
}

fn connected_rig() -> Rig {
    let rig = Rig::new();
    rig.connect(Some(UNIT));
    rig
}

fn probed_ranges(rig: &Rig) -> Vec<u8> {
    rig.acquisition
        .probe_requests()
        .iter()
        .map(|probe| probe.setup.channels[usize::from(probe.setup.trigger.source)].range_index)
        .collect()
}

/// 17 codes (1.41 mV) on CH4 at 2 mV/div, 1 code elsewhere: the measured short of the hardware run.
fn ch4_error_of_1_45_millivolts(channel: usize, range_index: usize) -> Vec<u8> {
    let bias = match (channel, range_index) {
        (3, 0) => 17,
        _ => 1,
    };
    vec![128 + bias; 300]
}

/// 36 codes (2.98 mV) at 2 mV/div on every channel.
fn three_millivolts_at_two_millivolts(_channel: usize, range_index: usize) -> Vec<u8> {
    vec![if range_index == 0 { 128 + 36 } else { 129 }; 300]
}

fn clipped_at_the_gate(_channel: usize, range_index: usize) -> Vec<u8> {
    let mut codes = vec![128; 300];
    if range_index == usize::from(GATE_RANGE_INDEX) {
        codes[7] = 255;
    }
    codes
}

/// 2 mV/div is off zero (continuable), 5 mV/div clips.
fn off_zero_then_clipped(channel: usize, range_index: usize) -> Vec<u8> {
    match range_index {
        1 => clipped_at_the_gate(channel, usize::from(GATE_RANGE_INDEX)),
        other => three_millivolts_at_two_millivolts(channel, other),
    }
}

fn rejected_at_channel_one(rig: &Rig) {
    rig.open().unwrap();
    rig.start().unwrap();
    rig.measure().unwrap();
    rig.until_step(ZeroStep::NotShorted);
}

#[test]
fn given_a_ch4_short_with_1_45_millivolts_at_two_millivolts_when_measured_then_it_is_accepted_and_stored() {
    let rig = connected_rig();
    rig.set_signal(ch4_error_of_1_45_millivolts);

    rig.calibrate_channels(4);

    let volts = stored_volts(&rig).unwrap();
    assert!((volts[3][0] - 17.0 * RANGES[0] / COUNTS).abs() < 1e-12);
}

#[test]
fn given_three_millivolts_at_two_millivolts_when_measured_then_it_is_rejected_with_the_new_limit_and_continue() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);

    rejected_at_channel_one(&rig);

    let session = rig.state().session.unwrap();
    assert_eq!(
        session.not_shorted,
        Some(vec!["Level 2.98 mV from zero at 2 mV/div, limit 2.50 mV.".to_owned()])
    );
    assert!(session.can_continue);
}

#[test]
fn given_a_rejection_at_two_millivolts_when_continued_then_no_probe_is_repeated_and_the_channel_is_accepted() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);
    rejected_at_channel_one(&rig);
    assert_eq!(probed_ranges(&rig), vec![GATE_RANGE_INDEX, 0]);

    rig.continue_channel().unwrap();
    let session = rig.until_step(ZeroStep::Channel).session.unwrap();

    assert_eq!(probed_ranges(&rig), vec![5, 0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11]);
    assert_eq!(session.done_channels, [true, false, false, false]);
}

#[test]
fn given_a_continued_rejection_when_the_channel_is_accepted_then_it_is_overridden_and_the_rejected_reading_is_kept() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);
    rejected_at_channel_one(&rig);

    rig.continue_channel().unwrap();
    let session = rig.until_step(ZeroStep::Channel).session.unwrap();

    assert_eq!(session.overridden, [true, false, false, false]);
    assert_eq!(session.not_shorted, None);
    assert!(!session.can_continue);
}

#[test]
fn given_four_continued_channels_when_the_last_is_accepted_then_one_write_stores_them_and_all_are_overridden() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);
    rig.open().unwrap();
    rig.start().unwrap();

    for channel in 0..4 {
        rig.measure().unwrap();
        rig.until_step(ZeroStep::NotShorted);
        assert_eq!(stored_volts(&rig), None, "channel {channel}");
        rig.continue_channel().unwrap();
        rig.until_step(if channel == 3 {
            ZeroStep::Done
        } else {
            ZeroStep::Channel
        });
    }

    let volts = stored_volts(&rig).unwrap();
    assert!(volts.iter().all(|v| (v[0] - 36.0 * RANGES[0] / COUNTS).abs() < 1e-12));
    assert_eq!(rig.state().session.unwrap().overridden, [true; 4]);
}

#[test]
fn given_a_clipped_gate_reading_when_rejected_then_continue_is_not_offered_and_the_request_fails() {
    let rig = connected_rig();
    rig.set_signal(clipped_at_the_gate);
    rejected_at_channel_one(&rig);
    let before = rig.state();

    let refused = rig.continue_channel().unwrap_err();

    assert!(!before.session.as_ref().unwrap().can_continue);
    assert_eq!(refused.message, WRONG_STEP);
    assert_eq!(rig.state(), before);
}

#[test]
fn given_a_clipped_later_range_when_continued_then_the_channel_is_still_accepted_as_overridden() {
    let rig = connected_rig();
    rig.set_signal(off_zero_then_clipped);
    rejected_at_channel_one(&rig);

    rig.continue_channel().unwrap();
    let session = rig.until_step(ZeroStep::Channel).session.unwrap();

    assert_eq!(session.overridden, [true, false, false, false]);
    assert_eq!(session.done_channels, [true, false, false, false]);
}

#[test]
fn given_a_rejection_when_try_again_is_pressed_then_the_sweep_restarts_at_the_gate() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);
    rejected_at_channel_one(&rig);

    rig.measure().unwrap();
    rig.until_step(ZeroStep::NotShorted);

    assert_eq!(probed_ranges(&rig), vec![GATE_RANGE_INDEX, 0, GATE_RANGE_INDEX, 0]);
}

#[test]
fn given_a_continued_channel_when_the_dialog_is_cancelled_then_nothing_is_stored_and_the_hold_is_released() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);
    rejected_at_channel_one(&rig);
    rig.continue_channel().unwrap();
    rig.until_step(ZeroStep::Channel);

    rig.close().unwrap();

    assert_eq!(stored_volts(&rig), None);
    assert_eq!(rig.acquisition.hold_requests(), vec![true, false]);
    assert!(rig.offsets.events().iter().all(|e| e.offsets.is_none()));
}

#[test]
fn given_a_rejection_waiting_longer_than_the_channel_timeout_when_continued_then_the_channel_is_still_accepted() {
    let rig = connected_rig();
    rig.set_signal(three_millivolts_at_two_millivolts);
    rejected_at_channel_one(&rig);
    rig.clock.advance(Duration::from_secs(120));

    rig.continue_channel().unwrap();
    let session = rig.until_step(ZeroStep::Channel).session.unwrap();

    assert_eq!(session.done_channels, [true, false, false, false]);
}

#[test]
fn given_no_session_when_continued_then_it_fails_and_the_state_is_unchanged() {
    let rig = connected_rig();
    let before = rig.state();

    assert!(rig.continue_channel().is_err());

    assert_eq!(rig.state(), before);
}

#[test]
fn given_a_session_outside_not_shorted_when_continued_then_it_fails_with_the_wrong_step_text() {
    let rig = connected_rig();
    rig.open().unwrap();
    rig.start().unwrap();
    let before = rig.state();

    let refused = rig.continue_channel().unwrap_err();

    assert_eq!(refused.message, WRONG_STEP);
    assert_eq!(rig.state(), before);
}
