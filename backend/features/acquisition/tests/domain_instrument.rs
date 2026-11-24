mod domain_support;

use domain_support::{
    assert_close, capabilities, enable_all, patch_enabled, patch_position, range_index, ready_instrument,
    time_base_index,
};
use sweepscope_acquisition::{ChannelPatch, Defaults, Instrument, SettingTarget, TriggerPatch};
use sweepscope_kernel::contracts::{Coupling, Slope, Sweep};
use sweepscope_kernel::ErrorCategory;

const CH1: SettingTarget = SettingTarget::Range { channel: 1 };

fn state(instrument: &Instrument) -> sweepscope_acquisition::InstrumentState {
    instrument.state(1).unwrap()
}

#[test]
fn given_no_device_ever_ready_when_the_state_is_read_then_there_is_none() {
    let instrument = Instrument::new(Defaults::default());

    assert!(instrument.state(1).is_none());
    assert!(instrument.setup().is_none());
}

#[test]
fn given_the_first_ready_device_when_the_state_is_read_then_it_holds_the_first_run_defaults() {
    let caps = capabilities();
    let instrument = ready_instrument();

    let s = state(&instrument);

    assert_eq!(s.channels.len(), 4);
    assert_eq!(
        s.channels.iter().map(|c| c.enabled).collect::<Vec<_>>(),
        [true, false, false, false]
    );
    assert_eq!(s.channels[0].range.index, range_index(&caps, 1.0));
    assert_eq!(s.channels[0].coupling, Coupling::Dc);
    assert_eq!(s.channels[0].probe, 1.0);
    assert_eq!(s.channels[0].position_divs, 0.0);
    assert_eq!(s.horizontal.time_base.index, time_base_index(&caps, 200e-6));
    assert_eq!(
        (s.trigger.source, s.trigger.slope, s.trigger.sweep),
        (1, Slope::Rising, Sweep::Auto)
    );
    assert_eq!(s.trigger.level.value, 0.0);
    assert_close(s.horizontal.trigger_position_fraction, 0.5);
    assert!(!s.running);
}

#[test]
fn given_the_defaults_when_the_setup_is_read_then_the_trigger_sits_at_50_percent_of_the_record() {
    let instrument = ready_instrument();

    let setup = instrument.setup().unwrap();

    assert_close(setup.trigger.horizontal_position_percent, 50.0);
}

#[test]
fn given_the_first_range_when_stepping_to_a_smaller_range_then_the_step_is_impossible_with_a_reason() {
    let mut instrument = ready_instrument();
    for _ in 0..20 {
        instrument.step(CH1, -1).unwrap();
    }

    let range = &state(&instrument).channels[0].range;

    assert_eq!(range.index, 0);
    assert!(!range.faster.enabled);
    assert!(range.faster.reason.is_some());
    assert!(range.slower.enabled);
}

#[test]
fn given_the_last_range_when_stepping_to_a_larger_range_then_the_step_is_impossible_with_a_reason() {
    let mut instrument = ready_instrument();
    for _ in 0..20 {
        instrument.step(CH1, 1).unwrap();
    }

    let range = &state(&instrument).channels[0].range;

    assert_eq!(usize::from(range.index), capabilities().vertical_ranges.len() - 1);
    assert!(!range.slower.enabled);
    assert!(range.slower.reason.is_some());
}

#[test]
fn given_the_last_range_when_stepping_further_then_nothing_changes_and_there_is_no_error() {
    let mut instrument = ready_instrument();
    for _ in 0..20 {
        instrument.step(CH1, 1).unwrap();
    }

    let change = instrument.step(CH1, 1).unwrap();

    assert!(!change.setup_changed && !change.state_changed);
}

#[test]
fn given_a_range_step_when_applied_then_the_setup_changes() {
    let mut instrument = ready_instrument();
    let before = instrument.setup().unwrap().channels[0].range_index;

    let change = instrument.step(CH1, 1).unwrap();

    assert!(change.setup_changed && change.state_changed);
    assert_eq!(instrument.setup().unwrap().channels[0].range_index, before + 1);
}

#[test]
fn given_the_time_base_table_when_stepping_to_the_fastest_then_the_first_step_is_impossible_with_a_reason() {
    let mut instrument = ready_instrument();
    for _ in 0..40 {
        instrument.step(SettingTarget::TimeBase, -1).unwrap();
    }

    let time_base = &state(&instrument).horizontal.time_base;

    assert_eq!(time_base.index, 0);
    assert!(!time_base.faster.enabled && time_base.faster.reason.is_some());
}

#[test]
fn given_the_time_base_table_when_stepping_to_the_slowest_then_roll_steps_are_reached_and_the_end_has_a_reason() {
    let caps = capabilities();
    let mut instrument = ready_instrument();
    for _ in 0..40 {
        instrument.step(SettingTarget::TimeBase, 1).unwrap();
    }

    let horizontal = state(&instrument).horizontal;

    assert_eq!(usize::from(horizontal.time_base.index), caps.time_bases.len() - 1);
    assert!(horizontal.roll);
    assert_eq!(horizontal.time_base.slower.reason.as_deref(), Some("Slowest time base"));
}

#[test]
fn given_the_slowest_frame_step_when_stepping_slower_then_roll_mode_starts() {
    let caps = capabilities();
    let mut instrument = ready_instrument();
    instrument.reset(SettingTarget::TimeBase).unwrap();
    while instrument.setup().unwrap().time_base_index < time_base_index(&caps, 200e-3) {
        instrument.step(SettingTarget::TimeBase, 1).unwrap();
    }
    assert!(!state(&instrument).horizontal.roll);

    instrument.step(SettingTarget::TimeBase, 1).unwrap();

    assert!(state(&instrument).horizontal.roll);
}

#[test]
fn given_the_time_base_table_when_stepping_slower_from_the_fastest_then_every_step_is_reachable() {
    let mut instrument = ready_instrument();
    for _ in 0..40 {
        instrument.step(SettingTarget::TimeBase, -1).unwrap();
    }

    let reached = (0..40)
        .filter(|_| instrument.step(SettingTarget::TimeBase, 1).unwrap().setup_changed)
        .count();

    assert_eq!(reached, capabilities().time_bases.len() - 1);
}

#[test]
fn given_a_time_base_step_when_applied_then_the_geometry_and_trigger_percent_follow() {
    let mut instrument = ready_instrument();
    instrument.set_trigger_position(0.0).unwrap();
    let before = instrument.setup().unwrap().trigger.horizontal_position_percent;

    instrument.step(SettingTarget::TimeBase, -1).unwrap();

    let after = instrument.setup().unwrap().trigger.horizontal_position_percent;
    assert_close(after, before);
    assert!(state(&instrument).horizontal.window_text.contains("of"));
}

#[test]
fn given_a_single_enabled_channel_when_disabling_it_then_the_edit_is_rejected_with_a_reason() {
    let mut instrument = ready_instrument();

    let error = instrument.set_channel(1, patch_enabled(false)).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(error.message, "At least one channel must stay on");
    assert!(state(&instrument).channels[0].enabled);
}

#[test]
fn given_a_single_enabled_channel_when_the_state_is_read_then_its_enable_is_unavailable_with_a_reason() {
    let instrument = ready_instrument();

    let enable = &state(&instrument).channels[0].enable;

    assert!(!enable.enabled);
    assert_eq!(enable.reason.as_deref(), Some("At least one channel must stay on"));
}

#[test]
fn given_the_trigger_source_when_it_is_disabled_then_the_source_moves_to_the_first_enabled_channel() {
    let mut instrument = ready_instrument();
    instrument.set_channel(3, patch_enabled(true)).unwrap();
    instrument.set_channel(2, patch_enabled(true)).unwrap();

    instrument.set_channel(1, patch_enabled(false)).unwrap();

    assert_eq!(state(&instrument).trigger.source, 2);
    assert_eq!(instrument.setup().unwrap().trigger.source, 1);
}

#[test]
fn given_a_disabled_channel_when_it_is_chosen_as_trigger_source_then_the_edit_is_rejected() {
    let mut instrument = ready_instrument();

    let result = instrument.set_trigger(TriggerPatch {
        source: Some(2),
        ..TriggerPatch::default()
    });

    assert_eq!(result.unwrap_err().category, ErrorCategory::Recoverable);
}

#[test]
fn given_another_channel_when_enabled_then_the_sample_interval_and_window_are_recomputed() {
    let mut instrument = ready_instrument();
    for _ in 0..40 {
        instrument.step(SettingTarget::TimeBase, -1).unwrap();
    }
    let one_channel = instrument.frame_context(1).window_len;

    enable_all(&mut instrument);

    assert_eq!(one_channel, 21);
    assert_eq!(instrument.frame_context(1).window_len, 6);
}

#[test]
fn given_a_position_beyond_the_limit_when_set_then_it_clamps_to_the_range_limit() {
    let mut instrument = ready_instrument();

    instrument.set_channel(1, patch_position(9.0)).unwrap();

    let channel = &state(&instrument).channels[0];
    assert_close(channel.position_divs, 4.0);
    assert_close(channel.position.value, 4.0);
}

#[test]
fn given_a_position_beyond_a_narrow_limit_when_the_range_changes_then_it_is_clamped_again() {
    let mut instrument = ready_instrument();
    instrument.set_channel(1, patch_position(3.5)).unwrap();

    instrument.step(CH1, -1).unwrap();

    let channel = &state(&instrument).channels[0];
    assert_close(channel.position_divs, 1.6);
    assert_close(channel.max_position_divs, 1.6);
}

#[test]
fn given_a_trigger_level_when_the_range_changes_then_the_volts_stay_and_the_divisions_change() {
    let mut instrument = ready_instrument();
    instrument.set_trigger_level(2.0).unwrap();
    let before = state(&instrument).trigger;

    instrument.step(CH1, 1).unwrap();

    let after = state(&instrument).trigger;
    assert_close(after.level.value, before.level.value);
    assert_close(before.level_divisions, 2.0);
    assert_close(after.level_divisions, 1.0);
}

#[test]
fn given_a_source_with_an_offset_when_the_level_is_set_then_it_is_stored_relative_to_the_ground() {
    let mut instrument = ready_instrument();
    instrument.set_channel(1, patch_position(1.0)).unwrap();

    instrument.set_trigger_level(3.0).unwrap();

    assert_close(instrument.setup().unwrap().trigger.level_volts, 2.0);
    assert_close(state(&instrument).trigger.level_divisions, 3.0);
}

#[test]
fn given_a_level_beyond_the_screen_when_set_then_it_clamps_to_the_screen_edge() {
    let mut instrument = ready_instrument();

    instrument.set_trigger_level(99.0).unwrap();

    assert_close(state(&instrument).trigger.level_divisions, 4.0);
}

#[test]
fn given_a_trigger_position_when_stepped_then_it_moves_5_percent_of_the_screen_and_stops_at_the_edges() {
    let mut instrument = ready_instrument();
    let target = SettingTarget::TriggerPosition;

    instrument.step(target, 1).unwrap();
    assert_close(state(&instrument).horizontal.trigger_position_fraction, 0.55);
    for _ in 0..30 {
        instrument.step(target, 1).unwrap();
    }

    assert_close(state(&instrument).horizontal.trigger_position_fraction, 1.0);
}

#[test]
fn given_a_trigger_level_when_stepped_then_it_moves_0_2_division() {
    let mut instrument = ready_instrument();

    instrument.step(SettingTarget::TriggerLevel, -1).unwrap();

    assert_close(state(&instrument).trigger.level_divisions, -0.2);
}

#[test]
fn given_changed_settings_when_reset_then_the_defaults_return() {
    let caps = capabilities();
    let mut instrument = ready_instrument();
    instrument.step(CH1, 3).unwrap();
    instrument.step(CH1, 1).unwrap();
    instrument.step(SettingTarget::TimeBase, 1).unwrap();
    instrument.set_trigger_position(0.2).unwrap();
    instrument.set_trigger_level(1.0).unwrap();

    for target in [
        CH1,
        SettingTarget::TimeBase,
        SettingTarget::TriggerPosition,
        SettingTarget::TriggerLevel,
    ] {
        instrument.reset(target).unwrap();
    }

    let s = state(&instrument);
    assert_eq!(s.channels[0].range.index, range_index(&caps, 1.0));
    assert_eq!(s.horizontal.time_base.index, time_base_index(&caps, 200e-6));
    assert_close(s.horizontal.trigger_position_fraction, 0.5);
    assert_close(s.trigger.level.value, 0.0);
}

#[test]
fn given_an_unknown_channel_or_a_value_that_is_not_finite_when_edited_then_the_edit_is_rejected() {
    let mut instrument = ready_instrument();

    assert!(instrument.set_channel(0, ChannelPatch::default()).is_err());
    assert!(instrument.set_channel(5, ChannelPatch::default()).is_err());
    assert!(instrument.set_channel(1, patch_position(f64::NAN)).is_err());
    assert!(instrument.step(SettingTarget::Range { channel: 7 }, 1).is_err());
    assert!(instrument.set_trigger_position(f64::INFINITY).is_err());
    assert!(instrument.set_trigger_level(f64::NAN).is_err());
}

#[test]
fn given_a_probe_that_is_not_1_10_or_100_when_set_then_the_edit_is_rejected() {
    let mut instrument = ready_instrument();

    let patch = ChannelPatch {
        probe: Some(7.0),
        ..ChannelPatch::default()
    };

    assert!(instrument.set_channel(1, patch).is_err());
}

#[test]
fn given_a_rejected_patch_when_part_of_it_was_valid_then_nothing_is_applied() {
    let mut instrument = ready_instrument();
    let patch = ChannelPatch {
        bandwidth_limit: Some(true),
        enabled: Some(false),
        ..ChannelPatch::default()
    };

    assert!(instrument.set_channel(1, patch).is_err());

    assert!(!state(&instrument).channels[0].bandwidth_limit);
}

#[test]
fn given_a_trigger_patch_when_applied_then_slope_and_sweep_change() {
    let mut instrument = ready_instrument();

    let change = instrument
        .set_trigger(TriggerPatch {
            slope: Some(Slope::Falling),
            sweep: Some(Sweep::Normal),
            ..TriggerPatch::default()
        })
        .unwrap();

    let trigger = state(&instrument).trigger;
    assert!(change.setup_changed);
    assert_eq!((trigger.slope, trigger.sweep), (Slope::Falling, Sweep::Normal));
}
