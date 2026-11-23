mod domain_support;

use domain_support::assert_close;
use sweepscope_acquisition::{geometry, record_percent, trigger_index_in_window};

const RECORD: usize = 4096;
const INTERVAL_AT_250_PER_DIV: f64 = 800e-9;
const SECONDS_PER_DIV: f64 = 200e-6;

#[test]
fn given_a_record_of_4096_and_250_samples_per_div_when_the_window_is_computed_then_it_spans_10_divisions() {
    let g = geometry(RECORD, INTERVAL_AT_250_PER_DIV, SECONDS_PER_DIV);

    assert_close(g.samples_per_div, 250.0);
    assert_eq!(g.window_len, 2501);
    assert_eq!(g.window_start, 798);
    assert_close(g.record_divs, 16.384);
}

#[test]
fn given_a_centred_window_when_the_screen_position_is_half_then_the_record_percent_is_50() {
    let g = geometry(RECORD, INTERVAL_AT_250_PER_DIV, SECONDS_PER_DIV);

    assert_close(record_percent(&g, RECORD, 0.5), 50.0);
}

#[test]
fn given_a_centred_window_when_the_screen_position_is_an_edge_then_the_record_percent_is_symmetric() {
    let g = geometry(RECORD, INTERVAL_AT_250_PER_DIV, SECONDS_PER_DIV);

    let left = record_percent(&g, RECORD, 0.0);
    let right = record_percent(&g, RECORD, 1.0);

    assert!((left - 19.48).abs() < 0.01, "{left}");
    assert_close(left + right, 100.0);
}

#[test]
fn given_four_channels_at_2_ns_per_div_when_the_window_is_computed_then_it_holds_6_samples() {
    let g = geometry(RECORD, 4e-9, 2e-9);

    assert_close(g.samples_per_div, 0.5);
    assert_eq!(g.window_len, 6);
}

#[test]
fn given_a_window_longer_than_the_record_when_computed_then_it_is_clamped_to_the_record() {
    let g = geometry(100, 1e-6, 1e-3);

    assert_eq!(g.window_len, 100);
    assert_eq!(g.window_start, 0);
}

#[test]
fn given_a_trigger_sample_inside_the_window_when_mapped_then_the_index_is_relative_to_the_window_start() {
    let g = geometry(RECORD, INTERVAL_AT_250_PER_DIV, SECONDS_PER_DIV);

    assert_eq!(trigger_index_in_window(&g, g.window_start + 10), Some(10));
}

#[test]
fn given_a_trigger_sample_outside_the_window_when_mapped_then_there_is_no_index() {
    let g = geometry(RECORD, INTERVAL_AT_250_PER_DIV, SECONDS_PER_DIV);

    assert_eq!(trigger_index_in_window(&g, g.window_start - 1), None);
    assert_eq!(trigger_index_in_window(&g, g.window_start + g.window_len), None);
}
