//! Builds the shown unit state and keeps the slice revision.
use engineering_format::{format, Quantity};
use sweepscope_kernel::contracts::Availability;

use super::offsets::StoredUnit;
use super::texts;
use crate::{ChannelTooltip, TooltipText, UnitCalibration, ZeroCalibrationState, ZeroSession};

const NO_OFFSET: &str = "-";
const TRUE_MINUS: &str = "\u{2212}";

/// Probe attenuation factors, in the slot order of a tooltip cell.
const PROBE_FACTORS: [f64; 3] = [1.0, 10.0, 100.0];
/// Typical zero error in millivolts of the four most sensitive ranges (protocol A.3).
const TYPICAL_MILLIVOLTS: [f64; 4] = [0.66, 1.04, 0.83, 0.83];
const COUNTS_PER_DIV: f64 = 24.15;
const TYPICAL_COUNTS_ABOVE: f64 = 2.0;

pub fn unit_calibration(stored: Option<&StoredUnit>, ranges: &[f64]) -> UnitCalibration {
    let channels = match stored {
        Some(unit) => std::array::from_fn(|channel| calibrated_channel(unit.volts.get(channel), ranges)),
        None => std::array::from_fn(|_| typical_channel(ranges)),
    };
    UnitCalibration {
        calibrated: stored.is_some(),
        stored_at_utc: stored.map(|unit| unit.stored_at_utc.clone()),
        tooltip: TooltipText {
            heading: texts::heading(stored.is_some()).to_owned(),
            channels,
        },
    }
}

fn calibrated_channel(volts: Option<&Vec<f64>>, ranges: &[f64]) -> ChannelTooltip {
    cells(ranges, |index, range, factor| {
        match volts.and_then(|v| v.get(index)).filter(|v| v.is_finite()) {
            Some(offset) => offset_text(offset * factor, range * factor),
            None => NO_OFFSET.to_owned(),
        }
    })
}

fn typical_channel(ranges: &[f64]) -> ChannelTooltip {
    cells(ranges, |index, range, factor| match TYPICAL_MILLIVOLTS.get(index) {
        Some(millivolts) => trimmed(offset_text(millivolts * 1e-3 * factor, range * factor)),
        None => {
            let two_counts = TYPICAL_COUNTS_ABOVE * range / COUNTS_PER_DIV * factor;
            format!("up to {} at {}", volts_text(two_counts), range_text(range * factor))
        }
    })
}

fn cells(ranges: &[f64], text: impl Fn(usize, f64, f64) -> String) -> ChannelTooltip {
    ChannelTooltip {
        cells: ranges
            .iter()
            .enumerate()
            .map(|(index, &range)| PROBE_FACTORS.map(|factor| text(index, range, factor)))
            .collect(),
    }
}

/// `+6.60 mV at 20 mV/div` becomes `+6.6 mV at 20 mV/div`: the typical figures carry two or three digits only.
fn trimmed(text: String) -> String {
    let Some((number, rest)) = text.split_once(' ') else {
        return text;
    };
    if !number.contains('.') {
        return text;
    }
    format!("{} {rest}", number.trim_end_matches('0').trim_end_matches('.'))
}

/// Millivolts with two to zero decimals: offsets are small and `+0.66 mV` reads better than `660 uV`.
fn offset_text(volts: f64, range: f64) -> String {
    let millivolts = volts * 1e3;
    let decimals = match millivolts.abs() {
        m if m < 10.0 => 2,
        m if m < 100.0 => 1,
        _ => 0,
    };
    let magnitude = format!("{:.decimals$}", millivolts.abs());
    let rounds_to_zero = magnitude.parse::<f64>() == Ok(0.0);
    let sign = if millivolts < 0.0 && !rounds_to_zero {
        TRUE_MINUS
    } else {
        "+"
    };
    format!("{sign}{magnitude} mV at {}", range_text(range))
}

fn range_text(range: f64) -> String {
    let shown = format(range, Quantity::VoltsPerDiv);
    format!("{} {}", shown.text, shown.unit)
}

fn volts_text(volts: f64) -> String {
    let shown = format(volts, Quantity::Volts);
    format!("{} {}", shown.text, shown.unit)
}

/// The slice: a revision that moves only when something shown changed.
pub struct Slice {
    state: ZeroCalibrationState,
}

impl Slice {
    pub fn new(availability: Availability) -> Self {
        Self {
            state: ZeroCalibrationState {
                revision: 0,
                availability,
                unit: None,
                session: None,
            },
        }
    }

    pub fn state(&self) -> ZeroCalibrationState {
        self.state.clone()
    }

    /// Returns the new state when it differs from the current one.
    pub fn update(
        &mut self,
        availability: Availability,
        unit: Option<UnitCalibration>,
        session: Option<ZeroSession>,
    ) -> Option<ZeroCalibrationState> {
        let unchanged =
            self.state.availability == availability && self.state.unit == unit && self.state.session == session;
        if unchanged {
            return None;
        }
        self.state = ZeroCalibrationState {
            revision: self.state.revision + 1,
            availability,
            unit,
            session,
        };
        Some(self.state.clone())
    }
}
