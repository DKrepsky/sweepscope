#![allow(dead_code)]
use sweepscope_acquisition::{ChannelPatch, Defaults, Instrument};
use sweepscope_kernel::contracts::{Capabilities, TimeBaseStep, VerticalRange};

const FIRST_VERIFIED_SECONDS: f64 = 100e-9;
const FIRST_ROLL_SECONDS: f64 = 0.5;

pub fn capabilities() -> Capabilities {
    let nanos = [2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0].map(|v| v * 1e-9);
    let micros = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0].map(|v| v * 1e-6);
    let millis = [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0].map(|v| v * 1e-3);
    let seconds = [0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1000.0];
    let time_bases = nanos
        .iter()
        .chain(&micros)
        .chain(&millis)
        .chain(&seconds)
        .map(|&seconds_per_div| TimeBaseStep {
            seconds_per_div,
            streaming: seconds_per_div >= FIRST_ROLL_SECONDS,
            verified: seconds_per_div >= FIRST_VERIFIED_SECONDS,
        })
        .collect();
    let volts = [0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0];
    let vertical_ranges = volts
        .iter()
        .map(|&volts_per_div| VerticalRange {
            volts_per_div,
            verified: (0.05..=1.0).contains(&volts_per_div),
            max_position_divs: if [0.05, 0.5, 5.0].contains(&volts_per_div) {
                1.6
            } else {
                4.0
            },
        })
        .collect();
    Capabilities {
        channel_count: 4,
        record_len: 4096,
        time_bases,
        vertical_ranges,
        adc_interval_ns: vec![1.0, 2.0, 4.0, 4.0],
        samples_per_div: 250.0,
    }
}

pub fn time_base_index(caps: &Capabilities, seconds_per_div: f64) -> u8 {
    caps.time_bases
        .iter()
        .position(|t| (t.seconds_per_div / seconds_per_div - 1.0).abs() < 1e-9)
        .unwrap() as u8
}

pub fn range_index(caps: &Capabilities, volts_per_div: f64) -> u8 {
    caps.vertical_ranges
        .iter()
        .position(|r| (r.volts_per_div / volts_per_div - 1.0).abs() < 1e-9)
        .unwrap() as u8
}

pub fn ready_instrument() -> Instrument {
    let mut instrument = Instrument::new(Defaults::default());
    instrument.on_ready(capabilities());
    instrument
}

pub fn enable_all(instrument: &mut Instrument) {
    for channel in 1..=4 {
        instrument.set_channel(channel, patch_enabled(true)).unwrap();
    }
}

pub fn patch_enabled(enabled: bool) -> ChannelPatch {
    ChannelPatch {
        enabled: Some(enabled),
        ..ChannelPatch::default()
    }
}

pub fn patch_position(divs: f64) -> ChannelPatch {
    ChannelPatch {
        position_divs: Some(divs),
        ..ChannelPatch::default()
    }
}

pub fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}
