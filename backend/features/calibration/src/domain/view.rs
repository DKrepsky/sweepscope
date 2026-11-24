//! Builds the shown unit state and keeps the slice revision.
use engineering_format::{format, Quantity};
use sweepscope_kernel::contracts::Availability;

use super::offsets::StoredUnit;
use super::texts;
use crate::{TooltipRow, TooltipText, UnitCalibration, ZeroCalibrationState, ZeroSession};

const NO_OFFSET: &str = "-";
const TRUE_MINUS: &str = "\u{2212}";

pub fn unit_calibration(stored: Option<&StoredUnit>, ranges: &[f64]) -> UnitCalibration {
    let rows = match stored {
        Some(unit) => largest_rows(unit, ranges),
        None => texts::typical_rows(),
    };
    UnitCalibration {
        calibrated: stored.is_some(),
        stored_at_utc: stored.map(|unit| unit.stored_at_utc.clone()),
        tooltip: TooltipText {
            rows,
            notes: texts::notes(stored.is_some()),
        },
    }
}

/// One row per channel: the offset that is largest relative to its range.
fn largest_rows(unit: &StoredUnit, ranges: &[f64]) -> Vec<TooltipRow> {
    unit.volts
        .iter()
        .enumerate()
        .map(|(channel, volts)| TooltipRow {
            label: format!("CH{}", channel + 1),
            value: largest(volts, ranges).map_or_else(|| NO_OFFSET.to_owned(), |(v, range)| offset_text(v, range)),
        })
        .collect()
}

fn largest(volts: &[f64], ranges: &[f64]) -> Option<(f64, f64)> {
    // Reversed so that ties (an all-zero channel) resolve to the most sensitive range, as `max_by` keeps the last.
    volts
        .iter()
        .zip(ranges)
        .rev()
        .filter(|(v, range)| v.is_finite() && **range > 0.0)
        .max_by(|a, b| (a.0.abs() / a.1).total_cmp(&(b.0.abs() / b.1)))
        .map(|(&v, &range)| (v, range))
}

/// Millivolts with two to zero decimals: offsets are small and `+0.66 mV` reads better than `660 µV`.
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
    let shown = format(range, Quantity::VoltsPerDiv);
    format!("{sign}{magnitude} mV at {} {}", shown.text, shown.unit)
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
