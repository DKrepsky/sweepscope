#![allow(dead_code)]
use std::sync::Arc;

use sweepscope_kernel::contracts::{ChannelData, Frame, FrameContext, TriggerOutcome};

pub const COUNTS_PER_DIV: f64 = 25.0;

pub fn channel(codes: Vec<u8>, position_divs: f64) -> ChannelData {
    ChannelData {
        codes,
        volts_per_div: 1.0,
        zero_code: 128.0 + position_divs * COUNTS_PER_DIV,
    }
}

pub fn frame(channels: [Option<ChannelData>; 4], trigger: TriggerOutcome) -> Frame {
    Frame {
        channels,
        counts_per_div: COUNTS_PER_DIV,
        sample_interval_s: 4e-6,
        trigger,
    }
}

pub fn context(window_start: usize, window_len: usize) -> FrameContext {
    FrameContext {
        setup_revision: 7,
        window_start,
        window_len,
        invert: [false; 4],
        probe: [1.0; 4],
        calibrated: [true; 4],
        seconds_per_div: 1e-3,
    }
}

pub fn one_channel(codes: Vec<u8>, trigger: TriggerOutcome) -> Arc<Frame> {
    Arc::new(frame([Some(channel(codes, 0.0)), None, None, None], trigger))
}

pub fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

pub fn i32_at(bytes: &[u8], at: usize) -> i32 {
    i32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

pub fn f64_at(bytes: &[u8], at: usize) -> f64 {
    f64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

pub fn f32s(bytes: &[u8]) -> Vec<f32> {
    let (chunks, _) = bytes[48..].as_chunks::<4>();
    chunks.iter().map(|c| f32::from_le_bytes(*c)).collect()
}
