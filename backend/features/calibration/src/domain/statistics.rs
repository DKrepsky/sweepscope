use std::sync::Arc;

use sweepscope_kernel::contracts::Frame;

use super::rules::{CLIPPED_HIGH_CODE, CLIPPED_LOW_CODE};

/// Statistics of raw converter codes, in codes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeStats {
    pub mean: f64,
    pub std_dev: f64,
    pub peak_to_peak: f64,
    pub clipped: bool,
}

impl CodeStats {
    pub fn of(codes: &[u8]) -> Option<Self> {
        let &first = codes.first()?;
        let count = codes.len() as f64;
        let mean = codes.iter().map(|&c| f64::from(c)).sum::<f64>() / count;
        let variance = codes.iter().map(|&c| (f64::from(c) - mean).powi(2)).sum::<f64>() / count;
        let (low, high) = codes
            .iter()
            .fold((first, first), |(low, high), &c| (low.min(c), high.max(c)));
        Some(Self {
            mean,
            std_dev: variance.sqrt(),
            peak_to_peak: f64::from(high - low),
            clipped: low == CLIPPED_LOW_CODE || high == CLIPPED_HIGH_CODE,
        })
    }
}

/// What one probe gave for the channel under test: the kept frames pooled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reading {
    pub stats: CodeStats,
    pub zero_code: f64,
    pub counts_per_div: f64,
}

impl Reading {
    pub fn new(codes: &[u8], zero_code: f64, counts_per_div: f64) -> Option<Self> {
        Some(Self {
            stats: CodeStats::of(codes)?,
            zero_code,
            counts_per_div,
        })
    }

    /// `None` when there are no frames or the channel carries no data.
    pub fn from_frames(frames: &[Arc<Frame>], channel: usize) -> Option<Self> {
        let first = frames.first()?;
        let zero_code = first.channels.get(channel)?.as_ref()?.zero_code;
        let codes: Vec<u8> = frames
            .iter()
            .filter_map(|frame| frame.channels.get(channel)?.as_ref())
            .flat_map(|data| data.codes.iter().copied())
            .collect();
        Self::new(&codes, zero_code, first.counts_per_div)
    }

    pub fn offset_codes(&self) -> f64 {
        self.stats.mean - self.zero_code
    }

    pub fn codes_to_volts(&self, codes: f64, volts_per_div: f64) -> f64 {
        codes * volts_per_div / self.counts_per_div
    }
}
