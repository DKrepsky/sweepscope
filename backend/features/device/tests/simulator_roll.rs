mod common;

use std::time::Duration;

use common::*;
use sweepscope_device::{DeviceError, ScopeDevice, TriggerOutcome};

const ALL: [bool; 4] = [true; 4];

fn rolling_rig() -> Rig {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_500MS));
    rig.device.start_roll().unwrap();
    rig
}

fn samples(frame: &sweepscope_device::Frame) -> usize {
    frame.channels[0].as_ref().unwrap().codes.len()
}

#[test]
fn given_a_roll_at_500_ms_per_div_when_one_second_passes_then_about_499_samples_per_channel_are_returned() {
    let mut rig = rolling_rig();
    rig.clock.advance(Duration::from_secs(1));

    let frame = rig.device.poll_roll().unwrap();

    assert!((499..=500).contains(&samples(&frame)), "{}", samples(&frame));
    assert_eq!(frame.trigger, TriggerOutcome::None);
    assert!(frame
        .channels
        .iter()
        .all(|c| c.as_ref().is_some_and(|c| c.codes.len() == samples(&frame))));
}

#[test]
fn given_a_roll_that_was_just_polled_when_polled_again_at_once_then_the_frame_is_empty() {
    let mut rig = rolling_rig();
    rig.clock.advance(Duration::from_secs(1));
    rig.device.poll_roll().unwrap();

    let frame = rig.device.poll_roll().unwrap();

    assert_eq!(samples(&frame), 0);
}

#[test]
fn given_two_polls_when_the_clock_advanced_between_them_then_no_sample_is_lost_or_repeated() {
    let mut rig = rolling_rig();
    rig.clock.advance(Duration::from_millis(1001));
    let first = samples(&rig.device.poll_roll().unwrap());
    rig.clock.advance(Duration::from_millis(1001));

    let second = samples(&rig.device.poll_roll().unwrap());

    assert_eq!(first + second, 1001);
}

#[test]
fn given_a_roll_when_the_clock_passes_95_percent_of_the_buffer_span_then_overrun_is_returned_once_and_polling_continues(
) {
    let mut rig = rolling_rig();
    rig.clock.advance(Duration::from_secs(32));

    let overrun = rig.device.poll_roll();
    rig.clock.advance(Duration::from_secs(1));
    let next = rig.device.poll_roll();

    assert_eq!(overrun.err(), Some(DeviceError::Overrun));
    assert!((499..=500).contains(&samples(&next.unwrap())));
}

#[test]
fn given_a_roll_just_below_95_percent_of_the_span_when_polled_then_data_is_returned() {
    let mut rig = rolling_rig();
    rig.clock.advance(Duration::from_secs(31));

    let frame = rig.device.poll_roll().unwrap();

    assert_eq!(samples(&frame), 15_500);
}

#[test]
fn given_inject_overrun_when_polled_then_overrun_once_and_then_data() {
    let mut rig = rolling_rig();
    rig.control.inject_overrun();
    rig.clock.advance(Duration::from_secs(1));

    let first = rig.device.poll_roll();
    rig.clock.advance(Duration::from_secs(1));
    let second = rig.device.poll_roll();

    assert_eq!(first.err(), Some(DeviceError::Overrun));
    assert!(samples(&second.unwrap()) > 0);
}

#[test]
fn given_a_fast_time_base_when_the_roll_starts_then_it_is_an_invalid_setting() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200MS));

    assert!(matches!(rig.device.start_roll(), Err(DeviceError::InvalidSetting(_))));
}

#[test]
fn given_a_roll_that_was_never_started_when_polled_then_it_is_an_invalid_setting() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_500MS));

    assert!(matches!(rig.device.poll_roll(), Err(DeviceError::InvalidSetting(_))));
}

#[test]
fn given_a_stopped_roll_when_polled_then_it_is_an_invalid_setting() {
    let mut rig = rolling_rig();

    rig.device.stop().unwrap();

    assert!(matches!(rig.device.poll_roll(), Err(DeviceError::InvalidSetting(_))));
}

#[test]
fn given_a_disabled_channel_when_a_roll_chunk_is_polled_then_its_entry_is_none() {
    let mut rig = rig();
    configured(&mut rig, &setup([true, false, false, false], TB_500MS));
    rig.device.start_roll().unwrap();
    rig.clock.advance(Duration::from_secs(1));

    let frame = rig.device.poll_roll().unwrap();

    assert!(frame.channels[0].is_some());
    assert!(frame.channels[1..].iter().all(Option::is_none));
}
