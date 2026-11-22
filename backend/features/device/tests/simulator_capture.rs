mod common;

use std::time::Duration;

use common::*;
use sweepscope_device::{DeviceError, ScopeDevice, SignalKind, Slope, Sweep, TriggerOutcome};

const ALL: [bool; 4] = [true; 4];
const TIMEOUT: Duration = Duration::from_millis(250);

#[test]
fn given_four_channels_and_an_edge_trigger_on_a_sine_at_200_us_when_captured_then_every_channel_has_4096_codes() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200US));

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert!(frame
        .channels
        .iter()
        .all(|c| c.as_ref().is_some_and(|c| c.codes.len() == 4096)));
}

#[test]
fn given_a_trigger_at_30_percent_when_captured_then_the_trigger_sample_is_at_that_position() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.trigger.horizontal_position_percent = 30.0;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert_eq!(frame.trigger, TriggerOutcome::Triggered { sample_index: 1229 });
}

#[test]
fn given_a_rising_trigger_when_captured_then_the_signal_rises_through_the_level_at_the_trigger_sample() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200US));

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let TriggerOutcome::Triggered { sample_index } = frame.trigger else {
        panic!("expected a trigger, got {:?}", frame.trigger)
    };
    let data = frame.channels[0].as_ref().unwrap();
    let zero = data.zero_code.round() as i32;
    let before = i32::from(data.codes[sample_index - 20]);
    let after = i32::from(data.codes[sample_index + 20]);
    assert!(before < zero && after > zero, "{before} {zero} {after}");
}

#[test]
fn given_a_falling_trigger_when_captured_then_the_signal_falls_through_the_level_at_the_trigger_sample() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.trigger.slope = Slope::Falling;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let TriggerOutcome::Triggered { sample_index } = frame.trigger else {
        panic!("expected a trigger, got {:?}", frame.trigger)
    };
    let data = frame.channels[0].as_ref().unwrap();
    let zero = data.zero_code.round() as i32;
    assert!(i32::from(data.codes[sample_index - 20]) > zero);
    assert!(i32::from(data.codes[sample_index + 20]) < zero);
}

#[test]
fn given_a_capture_when_the_clock_is_read_then_it_advanced_by_one_frame_cycle() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200US));
    let before = rig.clock_now();

    rig.device.capture(TIMEOUT).unwrap();

    // 4 x 4096 samples x 800 ns = 3.3 ms rounds to one 20 ms poll, plus 15 ms readback and 5 ms re-arm.
    assert_eq!(rig.clock_now() - before, Duration::from_millis(40));
}

#[test]
fn given_two_captures_when_the_time_between_returns_is_read_then_it_equals_the_cycle() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200US));
    rig.device.capture(TIMEOUT).unwrap();
    let first_return = rig.clock_now();

    rig.device.capture(TIMEOUT).unwrap();

    assert_eq!(rig.clock_now() - first_return, Duration::from_millis(40));
}

#[test]
fn given_a_slow_time_base_when_captured_then_the_cycle_is_the_acquisition_rounded_to_20_ms_plus_dead_time() {
    let mut rig = rig();
    configured(&mut rig, &setup([true, false, false, false], TB_200MS));
    let before = rig.clock_now();

    rig.device.capture(Duration::from_secs(60)).unwrap();

    // 4096 x 0.8 ms = 3.2768 s -> 3.28 s, plus 20 ms.
    assert_eq!(rig.clock_now() - before, Duration::from_millis(3300));
}

#[test]
fn given_normal_sweep_and_a_level_above_the_signal_when_captured_then_it_times_out_after_the_timeout_in_clock_time() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.trigger.sweep = Sweep::Normal;
    config.trigger.level_volts = 3.0;
    configured(&mut rig, &config);
    let before = rig.clock_now();

    let result = rig.device.capture(TIMEOUT);

    assert_eq!(result.err(), Some(DeviceError::Timeout));
    assert_eq!(rig.clock_now() - before, TIMEOUT);
}

#[test]
fn given_normal_sweep_and_a_signal_that_crosses_when_captured_then_it_triggers() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.trigger.sweep = Sweep::Normal;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert!(matches!(frame.trigger, TriggerOutcome::Triggered { .. }));
}

#[test]
fn given_auto_sweep_and_a_level_above_the_signal_when_captured_then_the_outcome_is_auto() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.trigger.level_volts = 3.0;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert_eq!(frame.trigger, TriggerOutcome::Auto);
    assert_eq!(frame.channels[0].as_ref().unwrap().codes.len(), 4096);
}

#[test]
fn given_a_disabled_channel_when_captured_then_its_entry_is_none() {
    let mut rig = rig();
    configured(&mut rig, &setup([true, false, true, false], TB_200US));

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert!(frame.channels[0].is_some() && frame.channels[2].is_some());
    assert!(frame.channels[1].is_none() && frame.channels[3].is_none());
}

#[test]
fn given_a_sine_above_the_range_when_captured_then_codes_clip_at_0_and_255() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.channels[0].range_index = 0;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let codes = &frame.channels[0].as_ref().unwrap().codes;
    assert_eq!(codes.iter().min(), Some(&0));
    assert_eq!(codes.iter().max(), Some(&255));
}

#[test]
fn given_a_sine_within_the_range_when_captured_then_codes_scale_by_counts_per_div() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200US));

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let data = frame.channels[0].as_ref().unwrap();
    let peak = f64::from(*data.codes.iter().max().unwrap()) - data.zero_code;
    assert!(
        (peak - 1.0 / data.volts_per_div * frame.counts_per_div).abs() < 1.5,
        "{peak}"
    );
    assert_eq!(frame.counts_per_div, 24.15);
}

#[test]
fn given_a_calibrator_signal_when_captured_then_it_swings_between_zero_and_five_volts() {
    let mut rig = rig();
    rig.control.set_signal(0, SignalKind::Calibrator);
    let mut config = setup([true, false, false, false], TB_200US);
    config.trigger.level_volts = 2.5;
    config.channels[0].range_index = 10;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let data = frame.channels[0].as_ref().unwrap();
    let high = f64::from(*data.codes.iter().max().unwrap());
    let low = f64::from(*data.codes.iter().min().unwrap());
    let swing_volts = (high - low) / frame.counts_per_div * data.volts_per_div;
    assert!((swing_volts - 5.0).abs() < 0.5, "{swing_volts}");
}

#[test]
fn given_a_device_that_was_not_configured_when_captured_then_it_is_an_invalid_setting() {
    let mut rig = rig();

    let result = rig.device.capture(TIMEOUT);

    assert!(matches!(result, Err(DeviceError::InvalidSetting(_))));
}

#[test]
fn given_an_out_of_table_range_index_when_configured_then_it_is_an_invalid_setting() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.channels[1].range_index = 12;

    assert!(matches!(
        rig.device.configure(&config),
        Err(DeviceError::InvalidSetting(_))
    ));
}

#[test]
fn given_an_out_of_table_time_base_when_configured_then_it_is_an_invalid_setting() {
    let mut rig = rig();

    assert!(matches!(
        rig.device.configure(&setup(ALL, 36)),
        Err(DeviceError::InvalidSetting(_))
    ));
}

#[test]
fn given_no_enabled_channel_when_configured_then_it_is_an_invalid_setting() {
    let mut rig = rig();

    let result = rig.device.configure(&setup([false; 4], TB_200US));

    assert!(matches!(result, Err(DeviceError::InvalidSetting(_))));
}

#[test]
fn given_a_trigger_source_on_a_disabled_channel_when_configured_then_it_is_an_invalid_setting() {
    let mut rig = rig();
    let mut config = setup([false, true, false, false], TB_200US);
    config.trigger.source = 0;

    assert!(matches!(
        rig.device.configure(&config),
        Err(DeviceError::InvalidSetting(_))
    ));
}

#[test]
fn given_a_non_finite_channel_position_when_configured_then_it_is_an_invalid_setting() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.channels[0].position_divs = f64::NAN;

    assert!(matches!(
        rig.device.configure(&config),
        Err(DeviceError::InvalidSetting(_))
    ));
}

#[test]
fn given_a_non_finite_trigger_level_when_configured_then_it_is_an_invalid_setting() {
    let mut rig = rig();
    let mut config = setup(ALL, TB_200US);
    config.trigger.level_volts = f64::INFINITY;

    assert!(matches!(
        rig.device.configure(&config),
        Err(DeviceError::InvalidSetting(_))
    ));
}

#[test]
fn given_auto_sweep_and_a_timeout_shorter_than_the_cycle_when_captured_then_the_capture_still_completes() {
    let mut rig = rig();
    configured(&mut rig, &setup(ALL, TB_200US));

    let frame = rig.device.capture(Duration::from_millis(1));

    assert!(frame.is_ok());
}
