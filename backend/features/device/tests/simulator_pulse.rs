mod common;

use std::time::Duration;

use common::*;
use sweepscope_device::{
    DeviceError, PulseCondition, ScopeDevice, SignalKind, SignalSpec, Slope, Sweep, TriggerKind, TriggerOutcome,
};

const TIMEOUT: Duration = Duration::from_millis(250);

fn pulse_rig(condition: PulseCondition, width_s: f64) -> Rig {
    let mut rig = rig();
    rig.control.set_signal(0, SignalKind::Calibrator);
    let mut config = setup([true, false, false, false], TB_200US);
    config.trigger.level_volts = 0.95;
    config.trigger.sweep = Sweep::Normal;
    config.trigger.kind = TriggerKind::Pulse { width_s, condition };
    configured(&mut rig, &config);
    rig
}

fn triggers(condition: PulseCondition, width_s: f64) -> bool {
    match pulse_rig(condition, width_s).device.capture(TIMEOUT) {
        Ok(frame) => matches!(frame.trigger, TriggerOutcome::Triggered { .. }),
        Err(DeviceError::Timeout) => false,
        Err(other) => panic!("{other:?}"),
    }
}

fn capture_inverted_train(slope: Slope, condition: PulseCondition, width_s: f64) -> Result<(), DeviceError> {
    let mut rig = rig();
    rig.control.set_signal(
        0,
        SignalSpec::new(SignalKind::PulseTrain { width_s: 300e-6 }).with_amplitude(-1.91),
    );
    let mut config = setup([true, false, false, false], TB_200US);
    config.trigger.level_volts = -0.95;
    config.trigger.slope = slope;
    config.trigger.sweep = Sweep::Normal;
    config.trigger.kind = TriggerKind::Pulse { width_s, condition };
    configured(&mut rig, &config);
    rig.device.capture(TIMEOUT).map(|_| ())
}

#[test]
fn given_the_calibrator_when_greater_than_400_us_then_every_frame_is_triggered() {
    let mut rig = pulse_rig(PulseCondition::Greater, 400e-6);

    for _ in 0..3 {
        let frame = rig.device.capture(TIMEOUT).unwrap();
        assert!(matches!(frame.trigger, TriggerOutcome::Triggered { .. }));
    }
}

#[test]
fn given_the_calibrator_when_greater_than_600_us_then_the_capture_times_out() {
    let mut rig = pulse_rig(PulseCondition::Greater, 600e-6);

    assert!(matches!(rig.device.capture(TIMEOUT), Err(DeviceError::Timeout)));
}

#[test]
fn given_the_calibrator_when_less_than_600_us_then_it_triggers() {
    assert!(triggers(PulseCondition::Less, 600e-6));
}

#[test]
fn given_the_calibrator_when_less_than_400_us_then_it_does_not_trigger() {
    assert!(!triggers(PulseCondition::Less, 400e-6));
}

#[test]
fn given_the_calibrator_when_equal_to_480_us_then_it_triggers() {
    assert!(triggers(PulseCondition::Equal, 480e-6));
}

#[test]
fn given_the_calibrator_when_equal_to_520_us_then_it_triggers() {
    assert!(triggers(PulseCondition::Equal, 520e-6));
}

#[test]
fn given_the_calibrator_when_equal_to_450_us_then_it_does_not_trigger() {
    assert!(!triggers(PulseCondition::Equal, 450e-6));
}

#[test]
fn given_the_calibrator_when_not_equal_to_200_us_then_it_triggers() {
    assert!(triggers(PulseCondition::NotEqual, 200e-6));
}

#[test]
fn given_the_calibrator_when_not_equal_to_500_us_then_it_does_not_trigger() {
    assert!(!triggers(PulseCondition::NotEqual, 500e-6));
}

#[test]
fn given_a_pulse_trigger_when_captured_then_the_trigger_sample_sits_on_the_leading_edge() {
    let mut rig = pulse_rig(PulseCondition::Greater, 400e-6);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let TriggerOutcome::Triggered { sample_index } = frame.trigger else {
        panic!("expected a trigger, got {:?}", frame.trigger)
    };
    let data = frame.channels[0].as_ref().unwrap();
    let zero = data.zero_code.round() as i32;
    assert!(i32::from(data.codes[sample_index - 20]) <= zero + 1);
    assert!(i32::from(data.codes[sample_index + 20]) > zero + 10);
}

#[test]
fn given_a_pulse_trigger_and_auto_sweep_when_no_pulse_matches_then_the_frame_is_untriggered() {
    let mut rig = rig();
    rig.control.set_signal(0, SignalKind::Calibrator);
    let mut config = setup([true, false, false, false], TB_200US);
    config.trigger.level_volts = 0.95;
    config.trigger.kind = TriggerKind::Pulse {
        width_s: 600e-6,
        condition: PulseCondition::Greater,
    };
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert_eq!(frame.trigger, TriggerOutcome::Auto);
}

#[test]
fn given_an_inverted_pulse_train_when_the_polarity_is_negative_then_its_pulse_width_triggers() {
    assert!(capture_inverted_train(Slope::Falling, PulseCondition::Equal, 300e-6).is_ok());
}

#[test]
fn given_an_inverted_pulse_train_when_the_width_differs_then_it_times_out() {
    let result = capture_inverted_train(Slope::Falling, PulseCondition::Equal, 500e-6);

    assert!(matches!(result, Err(DeviceError::Timeout)));
}

#[test]
fn given_an_inverted_pulse_train_when_the_polarity_is_positive_then_the_pulse_is_not_found() {
    let result = capture_inverted_train(Slope::Rising, PulseCondition::Equal, 300e-6);

    assert!(matches!(result, Err(DeviceError::Timeout)));
}

#[test]
fn given_a_changed_frequency_when_an_edge_trigger_captures_then_it_triggers_on_the_new_period() {
    let mut rig = rig();
    rig.control
        .set_signal(0, SignalSpec::new(SignalKind::Square).with_frequency(250.0));
    let mut config = setup([true, false, false, false], TB_200US);
    config.trigger.sweep = Sweep::Normal;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    assert!(matches!(frame.trigger, TriggerOutcome::Triggered { .. }));
}

#[test]
fn given_a_non_positive_pulse_width_when_configured_then_the_setting_is_invalid() {
    let mut rig = rig();
    let mut config = setup([true, false, false, false], TB_200US);
    config.trigger.kind = TriggerKind::Pulse {
        width_s: 0.0,
        condition: PulseCondition::Equal,
    };

    assert!(matches!(
        rig.device.configure(&config),
        Err(DeviceError::InvalidSetting(_))
    ));
}
