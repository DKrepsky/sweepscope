mod common;

use std::time::Duration;

use common::*;
use sweepscope_device::{ScopeDevice, SignalKind, SignalSpec};

const TIMEOUT: Duration = Duration::from_secs(1);
const RANGE_2MV: u8 = 0;
const RANGE_10MV: u8 = 2;
const RANGE_100MV: u8 = 5;

struct Stats {
    mean_offset_codes: f64,
    sd_codes: f64,
    counts_per_div: f64,
    volts_per_div: f64,
}

fn shorted_stats(rig: &mut Rig, channel: usize, range: u8) -> Stats {
    rig.control.set_signal(channel, SignalKind::Shorted);
    let mut config = setup([true; 4], TB_200US);
    config.channels[channel].range_index = range;
    configured(rig, &config);
    let frame = rig.device.capture(TIMEOUT).unwrap();
    let data = frame.channels[channel].as_ref().unwrap();
    let codes: Vec<f64> = data.codes.iter().map(|c| f64::from(*c)).collect();
    let mean = codes.iter().sum::<f64>() / codes.len() as f64;
    let variance = codes.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / codes.len() as f64;
    Stats {
        mean_offset_codes: mean - data.zero_code,
        sd_codes: variance.sqrt(),
        counts_per_div: frame.counts_per_div,
        volts_per_div: data.volts_per_div,
    }
}

#[test]
fn given_a_shorted_channel_on_the_10_mv_range_when_captured_then_it_centres_on_zero_code_plus_the_zero_error() {
    let mut rig = rig();
    rig.control.set_zero_error_volts(0, RANGE_10MV.into(), 0.0083);

    let stats = shorted_stats(&mut rig, 0, RANGE_10MV);

    let expected = 0.0083 / stats.volts_per_div * stats.counts_per_div;
    assert!(
        (stats.mean_offset_codes - expected).abs() < 0.3,
        "{}",
        stats.mean_offset_codes
    );
}

#[test]
fn given_a_shorted_channel_on_the_2_mv_range_when_captured_then_noise_is_about_24_codes() {
    let mut rig = rig();

    let stats = shorted_stats(&mut rig, 0, RANGE_2MV);

    assert!((stats.sd_codes - 24.0).abs() < 2.0, "{}", stats.sd_codes);
}

#[test]
fn given_a_shorted_channel_on_the_100_mv_range_when_captured_then_the_input_is_quiet_and_on_zero() {
    let mut rig = rig();

    let stats = shorted_stats(&mut rig, 1, RANGE_100MV);

    assert!(stats.sd_codes < 1.0, "{}", stats.sd_codes);
    assert!(stats.mean_offset_codes.abs() < 0.3, "{}", stats.mean_offset_codes);
}

#[test]
fn given_a_zero_error_on_one_channel_when_another_channel_is_shorted_then_it_keeps_its_own_error() {
    let mut rig = rig();
    rig.control.set_zero_error_volts(0, RANGE_10MV.into(), 0.02);
    rig.control.set_zero_error_volts(1, RANGE_10MV.into(), 0.0);

    let stats = shorted_stats(&mut rig, 1, RANGE_10MV);

    assert!(stats.mean_offset_codes.abs() < 0.3, "{}", stats.mean_offset_codes);
}

#[test]
fn given_a_zero_error_when_a_small_sine_is_captured_then_the_error_shifts_its_mean() {
    let mut rig = rig();
    rig.control
        .set_signal(0, SignalSpec::new(SignalKind::Sine).with_amplitude(0.02));
    rig.control.set_zero_error_volts(0, RANGE_10MV.into(), 0.01);
    let mut config = setup([true; 4], TB_200US);
    config.channels[0].range_index = RANGE_10MV;
    configured(&mut rig, &config);

    let frame = rig.device.capture(TIMEOUT).unwrap();

    let data = frame.channels[0].as_ref().unwrap();
    let mean = data.codes.iter().map(|c| f64::from(*c)).sum::<f64>() / data.codes.len() as f64;
    let expected = 0.01 / data.volts_per_div * frame.counts_per_div;
    assert!((mean - data.zero_code - expected).abs() < 1.5, "{mean}");
}

#[test]
fn given_the_same_shorted_setup_when_captured_by_two_devices_then_the_codes_are_identical() {
    assert_eq!(shorted_codes(), shorted_codes());
}

fn shorted_codes() -> Vec<u8> {
    let mut rig = rig();
    rig.control.set_signal(0, SignalKind::Shorted);
    configured(&mut rig, &setup([true; 4], TB_200US));
    rig.device.capture(TIMEOUT).unwrap().channels[0].clone().unwrap().codes
}
