//! Every tunable number of the sweep in one place; tuned on one unit, check on hardware (ADR 0021, PRD 5.2).
use std::time::Duration;

pub const GATE_VOLTS_PER_DIV: f64 = 0.1;
pub const GATE_MAX_PEAK_TO_PEAK_CODES: f64 = 6.0;
pub const GATE_MAX_STD_DEV_CODES: f64 = 1.5;
pub const GATE_MAX_LEVEL_CODES: f64 = 8.0;
pub const OTHER_MAX_LEVEL_CODES: f64 = 12.0;
pub const NOISE_FLOOR_CODES: f64 = 2.0;
pub const NOISE_FACTOR: f64 = 3.0;
pub const CLIPPED_LOW_CODE: u8 = 0;
pub const CLIPPED_HIGH_CODE: u8 = 255;

/// Standard deviation in codes of a shorted input per range, 2 mV/div first (protocol A.3).
pub const REFERENCE_STD_DEV_CODES: [f64; 12] = [24.0, 8.6, 4.1, 2.1, 0.9, 0.5, 0.5, 0.05, 0.0, 0.4, 0.0, 0.0];

pub const PROBE_DISCARD_FRAMES: u8 = 1;
pub const PROBE_KEEP_FRAMES: u8 = 3;
pub const PROBE_TIME_BASE_SECONDS_PER_DIV: f64 = 2e-3;
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
pub const CHANNEL_TIMEOUT: Duration = Duration::from_secs(40);
