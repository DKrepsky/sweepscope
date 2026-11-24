//! Auto-setup decisions (ADR 0020). Pure: plain numbers in and out, one probe at a time, no device and no clock.
#![forbid(unsafe_code)]

mod machine;
mod tables;

pub use machine::Autoset;

/// Numbers the lib needs about the instrument; the caller derives them from its capability tables.
/// Ranges and time bases are sorted from the most sensitive and the fastest.
#[derive(Debug, Clone, PartialEq)]
pub struct Tables {
    pub range_volts_per_div: Vec<f64>,
    pub range_verified: Vec<bool>,
    pub max_position_divs: Vec<f64>,
    pub time_base_s_per_div: Vec<f64>,
    pub roll_from_index: usize,
    pub counts_per_div: f64,
    pub record_len: usize,
    pub samples_per_div: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelObservation {
    pub min_code: u8,
    pub max_code: u8,
    pub clipped: bool,
    pub period_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub channels: [Option<ChannelObservation>; 4],
    pub sample_interval_s: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub range_index: [u8; 4],
    pub position_divs: [f64; 4],
    pub time_base_index: u8,
    pub trigger_source: u8,
    pub trigger_level_volts: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailReason {
    /// Signal present on an enabled channel but no period measurable up to the slowest rung.
    NoPeriod,
    /// Still clipped after the vertical corrections.
    Clipped,
    /// The period stayed outside 1.5 to 7 periods per screen after one correction.
    UnstablePeriod,
    ProbeBudget,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Apply the plan, capture, and call `next` with what was seen.
    Probe(Plan),
    Done(Plan),
    NoSignal(Plan),
    Failed(FailReason),
}
