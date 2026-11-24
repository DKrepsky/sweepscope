//! Accuracy texts and availability reasons. Numbers derive from protocol A.2 and A.3 measurements on one unit.
use sweepscope_kernel::contracts::Availability;

use crate::TooltipRow;

pub const NO_DEVICE: &str = "No device";
pub const NEEDS_READY_SCOPE: &str = "Zero calibration needs a ready scope";
pub const NO_SERIAL: &str = "This scope reports no serial number";
pub const AUTO_SETUP_RUNNING: &str = "Auto-setup is running";

pub const TYPICAL_HEADING: &str = "Typical zero error of one measured unit:";
pub const ZERO_REMOVED_NOTE: &str =
    "Zero error removed per channel and range, measured with the input shorted (about 12 000 samples averaged).";
pub const GAIN_NOTE: &str =
    "Gain: about 2 % from 50 mV to 1 V/div (one unit measured), not verified outside that range.";
pub const POSITION_NOTE: &str =
    "At extreme vertical positions readings can still be off by 2 to 3 counts (about 0.1 division).";
pub const RESOLUTION_NOTE: &str = "Resolution is one count, 1/24 of a division.";

const TYPICAL_ROWS: [(&str, &str); 5] = [
    ("2 mV/div", "+0.66 mV (8 counts)"),
    ("5 mV/div", "+1.04 mV (5 counts)"),
    ("10 mV/div", "+0.83 mV (2 counts)"),
    ("20 mV/div", "+0.83 mV (1 count)"),
    ("50 mV/div and above", "up to 2 counts between channels"),
];

pub fn available() -> Availability {
    Availability {
        enabled: true,
        reason: None,
    }
}

pub fn unavailable(reason: &str) -> Availability {
    Availability {
        enabled: false,
        reason: Some(reason.to_owned()),
    }
}

pub fn typical_rows() -> Vec<TooltipRow> {
    TYPICAL_ROWS
        .iter()
        .map(|(label, value)| TooltipRow {
            label: (*label).to_owned(),
            value: (*value).to_owned(),
        })
        .collect()
}

/// Notes under the rows. Not calibrated: the heading first; calibrated: what the calibration removed first.
pub fn notes(calibrated: bool) -> Vec<String> {
    let lead = if calibrated { ZERO_REMOVED_NOTE } else { TYPICAL_HEADING };
    [lead, GAIN_NOTE, POSITION_NOTE, RESOLUTION_NOTE]
        .map(str::to_owned)
        .to_vec()
}
