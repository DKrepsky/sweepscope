//! Sample scaling and window crop: raw codes to divisions from the top edge (ADR 0011 item 2).
use sweepscope_kernel::contracts::{ChannelData, DerivedTrace, TriggerOutcome};

/// Code of the screen centre; a channel's ground sits at `zero_code`, which already holds its position.
const SCREEN_CENTRE_CODE: f64 = 128.0;
const SCREEN_CENTRE_DIVS: f64 = 4.0;

/// The part of a record that is on screen, cut to what the record really holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Window {
    pub start: usize,
    pub len: usize,
}

impl Window {
    pub fn within(record_len: usize, start: usize, len: usize) -> Self {
        let start = start.min(record_len);
        Self {
            start,
            len: len.min(record_len - start),
        }
    }

    /// Index of the trigger sample inside the window, `-1` when there is none or it is cropped away.
    pub fn trigger_index(&self, trigger: TriggerOutcome) -> i32 {
        let TriggerOutcome::Triggered { sample_index } = trigger else {
            return -1;
        };
        match sample_index.checked_sub(self.start) {
            Some(inside) if inside < self.len => i32::try_from(inside).unwrap_or(-1),
            _ => -1,
        }
    }
}

/// The math trace (ADR 0019): ground at `position_divs`, never inverted, one value per record sample.
pub(crate) fn scale_derived(trace: &DerivedTrace) -> Vec<f32> {
    trace
        .volts
        .iter()
        .map(|volts| (SCREEN_CENTRE_DIVS - (trace.position_divs + volts / trace.volts_per_div)) as f32)
        .collect()
}

/// Divisions from the top edge; not clamped, the graph marks off-screen samples.
pub(crate) fn scale_window(data: &ChannelData, counts_per_div: f64, window: Window, invert: bool) -> Vec<f32> {
    let sign = if invert { -1.0 } else { 1.0 };
    let ground_up = (data.zero_code - SCREEN_CENTRE_CODE) / counts_per_div;
    data.codes[window.start..window.start + window.len]
        .iter()
        .map(|&code| {
            let up = ground_up + sign * (f64::from(code) - data.zero_code) / counts_per_div;
            (SCREEN_CENTRE_DIVS - up) as f32
        })
        .collect()
}
