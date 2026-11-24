#![allow(dead_code)]
use std::sync::Arc;

use sweepscope_calibration::domain::statistics::Reading;
use sweepscope_kernel::contracts::{ChannelData, Frame, TriggerOutcome};

pub const RANGES: [f64; 12] = [0.002, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0];
pub const GATE_INDEX: usize = 5;
pub const ZERO: f64 = 128.0;
pub const COUNTS: f64 = 24.15;

pub fn flat(center: u8) -> Vec<u8> {
    vec![center; 300]
}

/// Alternates around the centre: standard deviation `amplitude`, peak to peak `2 x amplitude`.
pub fn alternating(center: u8, amplitude: u8) -> Vec<u8> {
    (0..300)
        .map(|i| {
            if i % 2 == 0 {
                center + amplitude
            } else {
                center - amplitude
            }
        })
        .collect()
}

pub fn reading(codes: &[u8]) -> Reading {
    Reading::new(codes, ZERO, COUNTS).expect("codes are not empty")
}

pub fn frame_with(channel: usize, codes: Vec<u8>, zero_code: f64) -> Arc<Frame> {
    let mut channels: [Option<ChannelData>; 4] = Default::default();
    channels[channel] = Some(ChannelData {
        codes,
        volts_per_div: 0.1,
        zero_code,
    });
    Arc::new(Frame {
        channels,
        counts_per_div: COUNTS,
        sample_interval_s: 4e-8,
        trigger: TriggerOutcome::Auto,
    })
}
