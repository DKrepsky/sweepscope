mod domain_support;

use domain_support::{ready_instrument, time_base_index};
use sweepscope_acquisition::{Instrument, InstrumentState, SettingTarget, TriggerKindName, TriggerPatch};
use sweepscope_kernel::contracts::{PulseCondition, TriggerKind};
use sweepscope_kernel::ErrorCategory;

const TRIGGER_IN_ROLL: &str =
    "Trigger is not available in roll mode (500 ms/div and slower). Use a faster time base to trigger.";
const SINGLE_IN_ROLL: &str = "Single needs a captured record; not available in roll mode.";
const VIEWS_IN_ROLL: &str =
    "XY, FFT, persistence, math and cursors need a captured record; not available in roll mode.";
const RECORD_OUTSIDE_ROLL: &str = "Recording needs roll mode (500 ms/div or slower)";
const BAD_WIDTH: &str = "Invalid width. Enter a time such as 500 \u{b5}s. Applied: 500 \u{b5}s.";

fn state(instrument: &Instrument) -> InstrumentState {
    instrument.state(1).unwrap()
}

fn at_time_base(seconds_per_div: f64) -> Instrument {
    let wanted = time_base_index(&domain_support::capabilities(), seconds_per_div);
    let mut instrument = ready_instrument();
    while instrument.setup().unwrap().time_base_index != wanted {
        let direction = if instrument.setup().unwrap().time_base_index < wanted {
            1
        } else {
            -1
        };
        instrument.step(SettingTarget::TimeBase, direction).unwrap();
    }
    instrument
}

fn pulse_patch(text: &str) -> TriggerPatch {
    TriggerPatch {
        kind: Some(TriggerKindName::Pulse),
        condition: Some(PulseCondition::Greater),
        width_text: Some(text.to_owned()),
        ..TriggerPatch::default()
    }
}

fn pulse_width(instrument: &Instrument) -> String {
    let pulse = state(instrument).trigger.pulse.unwrap();
    format!("{} {}", pulse.width.text, pulse.width.unit)
}

#[test]
fn given_a_500_ms_per_div_setup_when_the_state_is_read_then_roll_is_on_and_the_trigger_controls_say_why_not() {
    let controls = state(&at_time_base(0.5)).controls;

    assert!(state(&at_time_base(0.5)).horizontal.roll);
    assert_eq!(controls.trigger.reason.as_deref(), Some(TRIGGER_IN_ROLL));
    assert_eq!(controls.force.reason.as_deref(), Some(TRIGGER_IN_ROLL));
}

#[test]
fn given_roll_when_the_state_is_read_then_single_autoset_and_frame_views_are_unavailable_with_their_texts() {
    let controls = state(&at_time_base(0.5)).controls;

    assert_eq!(controls.single.reason.as_deref(), Some(SINGLE_IN_ROLL));
    assert!(!controls.autoset.enabled);
    assert_eq!(controls.frame_views.reason.as_deref(), Some(VIEWS_IN_ROLL));
    assert!(!controls.single.enabled && !controls.frame_views.enabled);
}

#[test]
fn given_roll_stopped_when_the_state_is_read_then_record_waits_for_a_run() {
    let controls = state(&at_time_base(0.5)).controls;

    assert!(!controls.record.enabled);
    assert_eq!(controls.record.reason.as_deref(), Some("Start acquisition first"));
}

#[test]
fn given_roll_running_when_the_state_is_read_then_record_is_available() {
    let mut instrument = at_time_base(0.5);
    instrument.set_running(true).unwrap();

    assert!(state(&instrument).controls.record.enabled);
}

#[test]
fn given_a_frame_time_base_when_the_state_is_read_then_record_needs_roll_and_the_other_controls_are_available() {
    let mut instrument = at_time_base(0.2);
    instrument.set_running(true).unwrap();

    let controls = state(&instrument).controls;

    assert_eq!(controls.record.reason.as_deref(), Some(RECORD_OUTSIDE_ROLL));
    assert!(controls.trigger.enabled && controls.single.enabled && controls.autoset.enabled);
    assert!(controls.force.enabled && controls.frame_views.enabled);
    assert!(!state(&instrument).horizontal.roll);
}

#[test]
fn given_an_autoset_run_when_the_state_is_read_then_run_single_force_and_autoset_say_it_is_running() {
    let mut instrument = ready_instrument();
    instrument.set_running(true).unwrap();

    instrument.set_autoset_running(true);

    let controls = state(&instrument).controls;
    for control in [&controls.run, &controls.single, &controls.force, &controls.autoset] {
        assert_eq!(control.reason.as_deref(), Some("Auto-setup is running"));
    }
}

#[test]
fn given_a_new_instrument_when_the_state_is_read_then_the_trigger_is_an_edge_without_pulse_state() {
    let trigger = state(&ready_instrument()).trigger;

    assert_eq!(trigger.kind, TriggerKindName::Edge);
    assert!(trigger.pulse.is_none());
}

#[test]
fn given_a_pulse_width_text_when_set_then_the_state_shows_it_and_the_setup_carries_seconds() {
    let mut instrument = ready_instrument();

    instrument.set_trigger(pulse_patch("500 us")).unwrap();

    assert_eq!(pulse_width(&instrument), "500 us");
    assert_eq!(
        instrument.setup().unwrap().trigger.kind,
        TriggerKind::Pulse {
            width_s: 0.0005,
            condition: PulseCondition::Greater
        }
    );
}

#[test]
fn given_text_that_is_not_a_time_when_set_then_the_patch_is_rejected_and_nothing_changes() {
    let mut instrument = ready_instrument();
    instrument.set_trigger(pulse_patch("500 us")).unwrap();
    let before = state(&instrument);

    let error = instrument
        .set_trigger(TriggerPatch {
            condition: Some(PulseCondition::Less),
            ..pulse_patch("abc")
        })
        .unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(error.message, BAD_WIDTH);
    assert_eq!(state(&instrument), before);
}

#[test]
fn given_a_width_outside_the_driver_range_when_set_then_it_is_rejected_with_the_same_text() {
    let mut instrument = ready_instrument();
    instrument.set_trigger(pulse_patch("500 us")).unwrap();

    for text in ["1 ns", "11 s", "0", "-5 us"] {
        let error = instrument.set_trigger(pulse_patch(text)).unwrap_err();
        assert_eq!(error.message, BAD_WIDTH, "{text}");
    }
}

#[test]
fn given_widths_outside_and_inside_the_checked_range_when_set_then_all_are_accepted() {
    let mut instrument = ready_instrument();

    for text in ["5 us", "50 us", "500 us", "5 ms", "50 ms"] {
        instrument.set_trigger(pulse_patch(text)).unwrap();
        assert!(state(&instrument).trigger.pulse.is_some(), "{text}");
    }
}

#[test]
fn given_a_pulse_trigger_when_switched_to_edge_and_back_then_the_width_and_condition_return() {
    let mut instrument = ready_instrument();
    instrument.set_trigger(pulse_patch("2 ms")).unwrap();
    let edge = TriggerPatch {
        kind: Some(TriggerKindName::Edge),
        ..TriggerPatch::default()
    };
    let pulse = TriggerPatch {
        kind: Some(TriggerKindName::Pulse),
        ..TriggerPatch::default()
    };

    instrument.set_trigger(edge).unwrap();
    assert!(state(&instrument).trigger.pulse.is_none());
    instrument.set_trigger(pulse).unwrap();

    assert_eq!(pulse_width(&instrument), "2.00 ms");
}

#[test]
fn given_an_edge_trigger_when_only_a_width_is_given_then_the_trigger_stays_an_edge() {
    let mut instrument = ready_instrument();

    instrument
        .set_trigger(TriggerPatch {
            width_text: Some("1 ms".into()),
            ..TriggerPatch::default()
        })
        .unwrap();

    assert_eq!(instrument.setup().unwrap().trigger.kind, TriggerKind::Edge);
}

#[test]
fn given_a_pulse_trigger_when_the_slope_changes_then_the_polarity_is_the_slope_and_the_pulse_stays() {
    let mut instrument = ready_instrument();
    instrument.set_trigger(pulse_patch("500 us")).unwrap();

    instrument
        .set_trigger(TriggerPatch {
            slope: Some(sweepscope_kernel::contracts::Slope::Falling),
            ..TriggerPatch::default()
        })
        .unwrap();

    assert_eq!(state(&instrument).trigger.kind, TriggerKindName::Pulse);
    assert_eq!(
        instrument.setup().unwrap().trigger.slope,
        sweepscope_kernel::contracts::Slope::Falling
    );
}
