mod common;

use common::*;
use sweepscope_calibration::domain::statistics::{CodeStats, Reading};

#[test]
fn given_no_codes_when_statistics_are_asked_then_none_is_returned() {
    assert_eq!(CodeStats::of(&[]), None);
}

#[test]
fn given_alternating_codes_when_statistics_are_asked_then_mean_deviation_and_span_are_exact() {
    let stats = CodeStats::of(&alternating(100, 3)).unwrap();

    assert_eq!(stats.mean, 100.0);
    assert_eq!(stats.std_dev, 3.0);
    assert_eq!(stats.peak_to_peak, 6.0);
    assert!(!stats.clipped);
}

#[test]
fn given_a_code_on_either_rail_when_statistics_are_asked_then_the_record_is_clipped() {
    assert!(CodeStats::of(&[0, 10]).unwrap().clipped);
    assert!(CodeStats::of(&[10, 255]).unwrap().clipped);
}

#[test]
fn given_three_kept_frames_when_a_reading_is_made_then_the_codes_are_pooled_with_the_zero_code_of_the_channel() {
    let frames = [
        frame_with(2, flat(130), 128.5),
        frame_with(2, flat(131), 128.5),
        frame_with(2, flat(132), 128.5),
    ];

    let reading = Reading::from_frames(&frames, 2).unwrap();

    assert_eq!(reading.stats.mean, 131.0);
    assert_eq!(reading.zero_code, 128.5);
    assert_eq!(reading.counts_per_div, COUNTS);
    assert_eq!(reading.offset_codes(), 2.5);
}

#[test]
fn given_a_channel_without_data_when_a_reading_is_made_then_none_is_returned() {
    let frames = [frame_with(2, flat(130), 128.0)];

    assert_eq!(Reading::from_frames(&frames, 0), None);
    assert_eq!(Reading::from_frames(&[], 2), None);
}
