mod common;

use common::*;
use sweepscope_display::{encode, encode_in_mode, stamp_sent, InterpolationMode};
use sweepscope_kernel::contracts::{DerivedTrace, DerivedUnit, TriggerOutcome};

const HEADER_LEN: usize = 48;

#[test]
fn given_a_frame_when_encoded_then_header_fields_match_the_layout() {
    let f = frame(
        [
            Some(channel(vec![128; 8], 0.0)),
            None,
            Some(channel(vec![128; 8], 0.0)),
            None,
        ],
        TriggerOutcome::Triggered { sample_index: 4 },
    );

    let bytes = encode(&f, &context(1, 5), 9, 1234.5);

    assert_eq!(&bytes[0..4], b"SSF1");
    assert_eq!(u32_at(&bytes, 4), 9);
    assert_eq!(f64_at(&bytes, 8), 1234.5);
    assert_eq!(f64_at(&bytes, 16), 0.0);
    assert_eq!(bytes[24], 0b0101);
    assert_eq!(bytes[25], 0b01);
    assert_eq!(&bytes[26..28], &[0, 0]);
    assert_eq!(u32_at(&bytes, 28), 5);
    assert_eq!(f64_at(&bytes, 32), 4e-6);
    assert_eq!(i32_at(&bytes, 40), 3);
    assert_eq!(u32_at(&bytes, 44), 7);
    assert_eq!(bytes.len(), HEADER_LEN + 4 * 5 * 2);
}

#[test]
fn given_an_auto_frame_when_encoded_then_only_the_auto_flag_is_set_and_no_trigger_index() {
    let f = one_channel(vec![128; 4], TriggerOutcome::Auto);

    let bytes = encode(&f, &context(0, 4), 0, 0.0);

    assert_eq!(bytes[25], 0b10);
    assert_eq!(i32_at(&bytes, 40), -1);
}

#[test]
fn given_a_roll_chunk_when_encoded_then_no_flags_are_set() {
    let f = one_channel(vec![128; 4], TriggerOutcome::None);

    let bytes = encode(&f, &context(0, 4), 0, 0.0);

    assert_eq!(bytes[25], 0);
    assert_eq!(i32_at(&bytes, 40), -1);
}

#[test]
fn given_codes_when_encoded_then_values_are_divisions_from_the_top_edge() {
    let f = one_channel(vec![128, 228, 28, 153], TriggerOutcome::None);

    let values = f32s(&encode(&f, &context(0, 4), 0, 0.0));

    assert_eq!(values, vec![4.0, 0.0, 8.0, 3.0]);
}

#[test]
fn given_a_channel_position_when_encoded_then_the_ground_level_moves_with_it() {
    let f = frame([Some(channel(vec![178], 2.0)), None, None, None], TriggerOutcome::None);

    let values = f32s(&encode(&f, &context(0, 1), 0, 0.0));

    assert_eq!(values, vec![2.0]);
}

#[test]
fn given_invert_when_encoded_then_the_trace_mirrors_around_the_ground_level() {
    let f = frame(
        [Some(channel(vec![178, 203, 153], 2.0)), None, None, None],
        TriggerOutcome::None,
    );
    let mut ctx = context(0, 3);
    ctx.invert[0] = true;

    let values = f32s(&encode(&f, &context(0, 3), 0, 0.0));
    let inverted = f32s(&encode(&f, &ctx, 0, 0.0));

    assert_eq!(values, vec![2.0, 1.0, 3.0]);
    assert_eq!(inverted, vec![2.0, 3.0, 1.0]);
}

#[test]
fn given_invert_on_one_channel_when_encoded_then_the_other_channel_is_untouched() {
    let f = frame(
        [Some(channel(vec![153], 0.0)), Some(channel(vec![153], 0.0)), None, None],
        TriggerOutcome::None,
    );
    let mut ctx = context(0, 1);
    ctx.invert[1] = true;

    let values = f32s(&encode(&f, &ctx, 0, 0.0));

    assert_eq!(values, vec![3.0, 5.0]);
}

#[test]
fn given_an_off_screen_sample_when_encoded_then_the_value_is_not_clamped() {
    let f = one_channel(vec![255, 0], TriggerOutcome::None);

    let values = f32s(&encode(&f, &context(0, 2), 0, 0.0));

    assert!(values[0] < 0.0, "{values:?}");
    assert!(values[1] > 8.0, "{values:?}");
}

#[test]
fn given_a_window_when_encoded_then_only_its_samples_are_sent() {
    let f = one_channel(vec![28, 128, 228, 128, 28], TriggerOutcome::None);

    let bytes = encode(&f, &context(1, 3), 0, 0.0);

    assert_eq!(u32_at(&bytes, 28), 3);
    assert_eq!(f32s(&bytes), vec![4.0, 0.0, 4.0]);
}

#[test]
fn given_a_window_that_excludes_the_trigger_sample_when_encoded_then_the_index_is_minus_one() {
    let before = one_channel(vec![128; 8], TriggerOutcome::Triggered { sample_index: 1 });
    let after = one_channel(vec![128; 8], TriggerOutcome::Triggered { sample_index: 6 });

    assert_eq!(i32_at(&encode(&before, &context(2, 4), 0, 0.0), 40), -1);
    assert_eq!(i32_at(&encode(&after, &context(2, 4), 0, 0.0), 40), -1);
}

#[test]
fn given_a_trigger_on_the_window_edges_when_encoded_then_the_index_is_inside_the_window() {
    let first = one_channel(vec![128; 8], TriggerOutcome::Triggered { sample_index: 2 });
    let last = one_channel(vec![128; 8], TriggerOutcome::Triggered { sample_index: 5 });

    assert_eq!(i32_at(&encode(&first, &context(2, 4), 0, 0.0), 40), 0);
    assert_eq!(i32_at(&encode(&last, &context(2, 4), 0, 0.0), 40), 3);
}

#[test]
fn given_a_window_longer_than_the_record_when_encoded_then_it_is_cut_to_the_record() {
    let f = one_channel(vec![128; 4], TriggerOutcome::None);

    let bytes = encode(&f, &context(2, 10), 0, 0.0);

    assert_eq!(u32_at(&bytes, 28), 2);
    assert_eq!(bytes.len(), HEADER_LEN + 8);
}

#[test]
fn given_an_encoded_frame_when_stamp_sent_is_called_then_bytes_16_to_24_change_and_the_rest_not() {
    let f = one_channel(vec![128; 4], TriggerOutcome::None);
    let original = encode(&f, &context(0, 4), 3, 10.0);
    let mut stamped = original.clone();

    stamp_sent(&mut stamped, 99.25);

    assert_eq!(f64_at(&stamped, 16), 99.25);
    assert_eq!(stamped[..16], original[..16]);
    assert_eq!(stamped[24..], original[24..]);
}

fn math(volts: Vec<f64>) -> DerivedTrace {
    DerivedTrace {
        volts,
        unit: DerivedUnit::Volts,
        volts_per_div: 0.5,
        position_divs: 1.0,
    }
}

#[test]
fn given_a_derived_trace_when_encoded_then_channel_5_follows_the_position_and_scale_and_the_mask_has_bit_4() {
    let f = one_channel(vec![128; 3], TriggerOutcome::None);
    let trace = math(vec![0.0, 1.0, -0.5]);

    let bytes = encode_in_mode(&f, &context(0, 3), Some(&trace), 0, 0.0, InterpolationMode::Linear);

    assert_eq!(bytes[24], 0b1_0001);
    let values = f32s(&bytes);
    assert_eq!(values.len(), 6);
    assert_eq!(&values[3..], &[3.0, 1.0, 4.0]);
}

#[test]
fn given_a_derived_trace_and_a_window_when_encoded_then_it_is_cropped_like_the_channels() {
    let f = one_channel(vec![128; 5], TriggerOutcome::None);
    let trace = math(vec![0.0, 0.5, 1.0, 1.5, 2.0]);

    let values = f32s(&encode_in_mode(
        &f,
        &context(1, 3),
        Some(&trace),
        0,
        0.0,
        InterpolationMode::Linear,
    ));

    assert_eq!(&values[3..], &[2.0, 1.0, 0.0]);
}

#[test]
fn given_a_derived_trace_of_the_wrong_length_when_encoded_then_channel_5_is_left_out() {
    let f = one_channel(vec![128; 3], TriggerOutcome::None);

    let bytes = encode_in_mode(
        &f,
        &context(0, 3),
        Some(&math(vec![0.0; 2])),
        0,
        0.0,
        InterpolationMode::Linear,
    );

    assert_eq!(bytes[24], 0b0001);
    assert_eq!(f32s(&bytes).len(), 3);
}

#[test]
fn given_a_derived_trace_and_sinc_when_encoded_then_channel_5_has_as_many_points_as_the_others() {
    let f = one_channel(vec![128; 51], TriggerOutcome::None);
    let trace = math(vec![0.0; 51]);

    let bytes = encode_in_mode(&f, &context(0, 51), Some(&trace), 0, 0.0, InterpolationMode::Sinc);

    let points = u32_at(&bytes, 28) as usize;
    assert_eq!(f32s(&bytes).len(), 2 * points);
}

#[test]
fn given_the_golden_frame_when_encoded_then_it_equals_the_committed_fixture() {
    let f = frame(
        [
            Some(channel(vec![28, 78, 128, 178, 228], 0.0)),
            None,
            Some(channel(vec![178, 203, 153, 178, 178], 2.0)),
            None,
        ],
        TriggerOutcome::Triggered { sample_index: 3 },
    );
    let mut ctx = context(0, 5);
    ctx.invert[2] = true;

    let bytes = encode(&f, &ctx, 42, 1_700_000_000_000_000.0);

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/frame-v1.bin");
    if std::env::var_os("SWEEPSCOPE_UPDATE_FIXTURES").is_some() {
        std::fs::write(&path, &bytes).unwrap();
    }
    let committed = std::fs::read(&path).expect("fixture missing; run with SWEEPSCOPE_UPDATE_FIXTURES=1");
    assert_eq!(bytes, committed);
}
