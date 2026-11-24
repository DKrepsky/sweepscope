mod domain_support;

use domain_support::{assert_close, enable_all, patch_enabled, range_index, ready_instrument, time_base_index};
use sweepscope_acquisition::{
    ChannelPatch, Instrument, MathOperation, MathPatch, MathState, MathStatus, SettingTarget,
};
use sweepscope_kernel::contracts::{ChannelData, DerivedUnit, Frame, TriggerOutcome};
use sweepscope_kernel::ErrorCategory;

const CH2_OFF: &str = "CH2 is off";
const VIEWS_IN_ROLL: &str =
    "XY, FFT, persistence, math and cursors need a captured record; not available in roll mode.";

fn math(instrument: &Instrument) -> MathState {
    instrument.state(1).unwrap().math
}

fn enabled() -> MathPatch {
    MathPatch {
        enabled: Some(true),
        ..MathPatch::default()
    }
}

fn operation(operation: MathOperation) -> MathPatch {
    MathPatch {
        operation: Some(operation),
        ..MathPatch::default()
    }
}

fn two_channels() -> Instrument {
    let mut instrument = ready_instrument();
    instrument.set_channel(2, patch_enabled(true)).unwrap();
    instrument
}

fn set_range(instrument: &mut Instrument, channel: u8, volts_per_div: f64) {
    let wanted = range_index(&domain_support::capabilities(), volts_per_div);
    while current_range(instrument, channel) != wanted {
        let direction = if current_range(instrument, channel) < wanted {
            1
        } else {
            -1
        };
        instrument.step(SettingTarget::Range { channel }, direction).unwrap();
    }
}

fn current_range(instrument: &Instrument, channel: u8) -> u8 {
    instrument.state(1).unwrap().channels[usize::from(channel) - 1]
        .range
        .index
}

fn scale_text(state: &MathState) -> String {
    format!("{} {}", state.scale.label.text, state.scale.label.unit)
}

fn data(codes: Vec<u8>, volts_per_div: f64) -> Option<ChannelData> {
    Some(ChannelData {
        codes,
        volts_per_div,
        zero_code: 128.0,
    })
}

fn frame(ch1: Option<ChannelData>, ch2: Option<ChannelData>) -> Frame {
    Frame {
        channels: [ch1, ch2, None, None],
        counts_per_div: 25.0,
        sample_interval_s: 1e-6,
        trigger: TriggerOutcome::Auto,
    }
}

fn two_channel_frame() -> Frame {
    frame(data(vec![153, 128, 103], 1.0), data(vec![128, 153, 178], 1.0))
}

#[test]
fn given_a_fresh_instrument_when_the_math_state_is_read_then_it_is_off_with_the_default_subtraction_of_ch1_and_ch2() {
    let state = math(&ready_instrument());

    assert!(!state.enabled);
    assert_eq!(state.operation, MathOperation::Subtract);
    assert_eq!((state.source_a, state.source_b), (1, Some(2)));
    assert_eq!(state.label, "A\u{2212}B");
    assert_eq!(state.unit, "V");
    assert_eq!(state.status, MathStatus::Ok);
}

#[test]
fn given_ch1_at_default_range_when_math_is_enabled_then_the_scale_is_the_nearest_table_value_at_or_above() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 1, 0.2);
    set_range(&mut instrument, 2, 0.05);

    instrument.set_math(enabled()).unwrap();

    assert_eq!(scale_text(&math(&instrument)), "200 mV/div");
}

#[test]
fn given_a_source_range_off_the_table_when_math_is_enabled_then_the_scale_rounds_up() {
    let mut instrument = two_channels();
    instrument
        .set_channel(
            1,
            ChannelPatch {
                probe: Some(10.0),
                ..ChannelPatch::default()
            },
        )
        .unwrap();
    set_range(&mut instrument, 1, 0.2);

    instrument.set_math(enabled()).unwrap();

    assert_eq!(scale_text(&math(&instrument)), "2 V/div");
}

#[test]
fn given_multiply_when_it_is_chosen_then_the_unit_is_volts_squared_and_the_scale_is_the_product() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 1, 0.5);
    set_range(&mut instrument, 2, 0.2);
    instrument.set_math(enabled()).unwrap();

    instrument.set_math(operation(MathOperation::Multiply)).unwrap();

    let state = math(&instrument);
    assert_eq!(state.unit, "V\u{b2}");
    assert_eq!(state.label, "A\u{d7}B");
    assert_eq!(scale_text(&state), "0.1 V\u{b2}/div");
}

#[test]
fn given_invert_a_when_it_is_chosen_then_source_b_is_none_and_the_scale_follows_a() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 1, 0.1);
    set_range(&mut instrument, 2, 5.0);
    instrument.set_math(enabled()).unwrap();

    instrument.set_math(operation(MathOperation::InvertA)).unwrap();

    let state = math(&instrument);
    assert_eq!(state.source_b, None);
    assert_eq!(scale_text(&state), "100 mV/div");
}

#[test]
fn given_only_ch1_on_when_math_is_enabled_then_it_is_unavailable_because_ch2_is_off() {
    let mut instrument = ready_instrument();

    instrument.set_math(enabled()).unwrap();

    assert_eq!(
        math(&instrument).status,
        MathStatus::Unavailable {
            reason: CH2_OFF.to_owned()
        }
    );
}

#[test]
fn given_ch2_and_ch3_on_without_ch1_when_math_is_enabled_then_the_first_two_enabled_channels_are_the_sources() {
    let mut instrument = ready_instrument();
    instrument.set_channel(2, patch_enabled(true)).unwrap();
    instrument.set_channel(3, patch_enabled(true)).unwrap();
    instrument.set_channel(1, patch_enabled(false)).unwrap();

    instrument.set_math(enabled()).unwrap();

    let state = math(&instrument);
    assert_eq!((state.source_a, state.source_b), (2, Some(3)));
    assert_eq!(state.status, MathStatus::Ok);
}

#[test]
fn given_a_chosen_source_pair_that_is_on_when_math_is_enabled_then_the_choice_is_kept() {
    let mut instrument = ready_instrument();
    enable_all(&mut instrument);
    instrument
        .set_math(MathPatch {
            source_a: Some(3),
            source_b: Some(4),
            ..MathPatch::default()
        })
        .unwrap();

    instrument.set_math(enabled()).unwrap();

    assert_eq!((math(&instrument).source_a, math(&instrument).source_b), (3, Some(4)));
}

#[test]
fn given_math_on_when_ch2_goes_off_then_the_status_becomes_unavailable_with_the_reason() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();

    instrument.set_channel(2, patch_enabled(false)).unwrap();

    assert_eq!(
        math(&instrument).status,
        MathStatus::Unavailable {
            reason: CH2_OFF.to_owned()
        }
    );
}

#[test]
fn given_invert_a_and_ch2_off_when_the_status_is_read_then_math_is_still_available() {
    let mut instrument = ready_instrument();
    instrument.set_math(enabled()).unwrap();

    instrument.set_math(operation(MathOperation::InvertA)).unwrap();

    assert_eq!(math(&instrument).status, MathStatus::Ok);
}

#[test]
fn given_sources_at_500_mv_and_20_mv_when_math_is_on_then_the_status_is_mixed_with_the_scales_in_the_detail() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 1, 0.5);
    set_range(&mut instrument, 2, 0.02);

    instrument.set_math(enabled()).unwrap();

    let state = math(&instrument);
    assert_eq!(
        state.status,
        MathStatus::Mixed {
            detail: "CH1 500 mV/div, CH2 20 mV/div. The math trace has its own scale, 500 mV/div.".to_owned()
        }
    );
    assert_eq!(state.unit, "V");
}

#[test]
fn given_different_scales_when_the_operation_is_multiply_then_the_status_is_not_mixed() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 1, 0.5);
    set_range(&mut instrument, 2, 0.02);

    instrument.set_math(enabled()).unwrap();
    instrument.set_math(operation(MathOperation::Multiply)).unwrap();

    assert_eq!(math(&instrument).status, MathStatus::Ok);
}

#[test]
fn given_a_500_ms_per_div_time_base_when_math_is_enabled_then_it_is_refused_with_the_roll_reason() {
    let mut instrument = two_channels();
    let wanted = time_base_index(&domain_support::capabilities(), 0.5);
    while instrument.setup().unwrap().time_base_index < wanted {
        instrument.step(SettingTarget::TimeBase, 1).unwrap();
    }

    let error = instrument.set_math(enabled()).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_eq!(error.message, VIEWS_IN_ROLL);
}

#[test]
fn given_math_on_when_the_time_base_enters_roll_then_the_status_is_unavailable_with_the_roll_reason() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();
    let wanted = time_base_index(&domain_support::capabilities(), 0.5);
    while instrument.setup().unwrap().time_base_index < wanted {
        instrument.step(SettingTarget::TimeBase, 1).unwrap();
    }

    assert_eq!(
        math(&instrument).status,
        MathStatus::Unavailable {
            reason: VIEWS_IN_ROLL.to_owned()
        }
    );
}

#[test]
fn given_the_scale_when_it_is_stepped_up_then_it_follows_the_1_2_5_table() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();
    let mut seen = vec![scale_text(&math(&instrument))];

    for _ in 0..3 {
        instrument.step(SettingTarget::MathScale, 1).unwrap();
        seen.push(scale_text(&math(&instrument)));
    }

    assert_eq!(seen, ["1 V/div", "2 V/div", "5 V/div", "10 V/div"]);
}

#[test]
fn given_the_scale_at_the_ends_when_it_is_stepped_beyond_then_it_stays_and_the_step_is_unavailable() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();
    for _ in 0..30 {
        instrument.step(SettingTarget::MathScale, 1).unwrap();
    }
    assert_eq!(scale_text(&math(&instrument)), "500 V/div");
    assert!(!math(&instrument).scale.slower.enabled);

    for _ in 0..30 {
        instrument.step(SettingTarget::MathScale, -1).unwrap();
    }

    assert_eq!(scale_text(&math(&instrument)), "1 mV/div");
    assert!(!math(&instrument).scale.faster.enabled);
}

#[test]
fn given_a_stepped_scale_when_it_is_reset_then_it_returns_to_the_fit_for_the_sources() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 1, 0.2);
    set_range(&mut instrument, 2, 0.05);
    instrument.set_math(enabled()).unwrap();
    instrument.step(SettingTarget::MathScale, 1).unwrap();
    instrument.step(SettingTarget::MathScale, 1).unwrap();

    instrument.reset(SettingTarget::MathScale).unwrap();

    assert_eq!(scale_text(&math(&instrument)), "200 mV/div");
}

#[test]
fn given_the_position_when_it_is_stepped_then_it_moves_by_a_fifth_of_a_division_and_stops_at_the_screen_edge() {
    let mut instrument = two_channels();
    instrument.step(SettingTarget::MathPosition, 1).unwrap();
    instrument.step(SettingTarget::MathPosition, 1).unwrap();
    assert_close(math(&instrument).position_divs, 0.4);

    for _ in 0..40 {
        instrument.step(SettingTarget::MathPosition, -1).unwrap();
    }

    assert_close(math(&instrument).position_divs, -4.0);
}

#[test]
fn given_a_moved_position_when_it_is_reset_then_it_returns_to_the_centre() {
    let mut instrument = two_channels();
    instrument.step(SettingTarget::MathPosition, 1).unwrap();

    instrument.reset(SettingTarget::MathPosition).unwrap();

    assert_close(math(&instrument).position_divs, 0.0);
    assert_eq!(math(&instrument).position.text, "0.00");
}

#[test]
fn given_a_position_patch_beyond_the_screen_when_it_is_set_then_it_is_clamped_and_a_nan_is_refused() {
    let mut instrument = two_channels();
    let set = |divs: f64| MathPatch {
        position_divs: Some(divs),
        ..MathPatch::default()
    };

    instrument.set_math(set(9.0)).unwrap();
    assert_close(math(&instrument).position_divs, 4.0);
    let error = instrument.set_math(set(f64::NAN)).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert_close(math(&instrument).position_divs, 4.0);
}

#[test]
fn given_a_source_number_outside_one_to_four_when_math_is_patched_then_it_is_refused_and_nothing_changes() {
    let mut instrument = two_channels();
    let before = math(&instrument);

    let result = instrument.set_math(MathPatch {
        source_a: Some(5),
        ..MathPatch::default()
    });

    assert!(result.is_err());
    assert_eq!(math(&instrument), before);
}

#[test]
fn given_a_math_edit_when_it_is_applied_then_the_device_setup_is_not_changed() {
    let mut instrument = two_channels();

    let change = instrument.set_math(enabled()).unwrap();

    assert!(change.state_changed);
    assert!(!change.setup_changed);
}

#[test]
fn given_math_on_with_two_channels_when_a_frame_is_derived_then_it_is_a_minus_b_in_volts_with_the_math_scale() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();

    let trace = instrument.derive_math(&two_channel_frame()).unwrap();

    assert_eq!(trace.unit, DerivedUnit::Volts);
    assert_eq!(trace.volts.len(), 3);
    for (actual, expected) in trace.volts.iter().zip([1.0, -1.0, -3.0]) {
        assert_close(*actual, expected);
    }
    assert_close(trace.volts_per_div, 1.0);
    assert_close(trace.position_divs, 0.0);
}

#[test]
fn given_probes_when_a_frame_is_derived_then_the_trace_is_in_probe_scaled_volts() {
    let mut instrument = two_channels();
    instrument
        .set_channel(
            1,
            ChannelPatch {
                probe: Some(10.0),
                ..ChannelPatch::default()
            },
        )
        .unwrap();
    instrument.set_math(enabled()).unwrap();

    let trace = instrument.derive_math(&two_channel_frame()).unwrap();

    assert_close(trace.volts[0], 10.0);
}

#[test]
fn given_inverted_sources_when_a_frame_is_derived_then_the_physical_signal_is_used() {
    let mut instrument = two_channels();
    instrument
        .set_channel(
            1,
            ChannelPatch {
                invert: Some(true),
                ..ChannelPatch::default()
            },
        )
        .unwrap();
    instrument.set_math(enabled()).unwrap();

    let trace = instrument.derive_math(&two_channel_frame()).unwrap();

    assert_close(trace.volts[0], 1.0);
}

#[test]
fn given_multiply_when_a_frame_is_derived_then_the_unit_is_volts_squared() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();
    instrument.set_math(operation(MathOperation::Multiply)).unwrap();

    let trace = instrument.derive_math(&two_channel_frame()).unwrap();

    assert_eq!(trace.unit, DerivedUnit::VoltsSquared);
    assert_close(trace.volts[1], 0.0);
    assert_close(trace.volts[2], -2.0);
}

#[test]
fn given_invert_a_when_a_frame_with_only_ch1_is_derived_then_the_trace_is_minus_a() {
    let mut instrument = ready_instrument();
    instrument.set_math(enabled()).unwrap();
    instrument.set_math(operation(MathOperation::InvertA)).unwrap();

    let trace = instrument.derive_math(&frame(data(vec![153, 103], 1.0), None)).unwrap();

    assert_close(trace.volts[0], -1.0);
    assert_close(trace.volts[1], 1.0);
}

#[test]
fn given_math_off_when_a_frame_is_derived_then_there_is_no_trace() {
    let instrument = two_channels();

    assert!(instrument.derive_math(&two_channel_frame()).is_none());
}

#[test]
fn given_ch2_off_when_a_frame_is_derived_then_there_is_no_trace() {
    let mut instrument = ready_instrument();
    instrument.set_math(enabled()).unwrap();

    assert!(instrument
        .derive_math(&frame(data(vec![153, 128], 1.0), None))
        .is_none());
}

#[test]
fn given_mixed_scales_when_a_frame_is_derived_then_the_trace_is_still_computed() {
    let mut instrument = two_channels();
    set_range(&mut instrument, 2, 0.02);
    instrument.set_math(enabled()).unwrap();

    let trace = instrument
        .derive_math(&frame(data(vec![153], 1.0), data(vec![153], 0.02)))
        .unwrap();

    assert_close(trace.volts[0], 1.0 - 0.02);
}

#[test]
fn given_sources_with_different_record_lengths_when_a_frame_is_derived_then_there_is_no_trace() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();

    let uneven = frame(data(vec![153, 128], 1.0), data(vec![128], 1.0));

    assert!(instrument.derive_math(&uneven).is_none());
}

#[test]
fn given_math_on_when_it_is_disabled_again_then_there_is_no_trace_and_the_status_is_ok() {
    let mut instrument = two_channels();
    instrument.set_math(enabled()).unwrap();

    instrument
        .set_math(MathPatch {
            enabled: Some(false),
            ..MathPatch::default()
        })
        .unwrap();

    assert!(instrument.derive_math(&two_channel_frame()).is_none());
    assert_eq!(math(&instrument).status, MathStatus::Ok);
}
