//! Pulse trigger rules (#25): the width text, its limits and what the slice shows.
use engineering_format::{parse_seconds, Quantity};
use sweepscope_kernel::contracts::TriggerKind;
use sweepscope_kernel::KernelError;

use super::model::Model;
use super::view::formatted;
use crate::{PulseState, TriggerKindName, TriggerPatch};

/// The driver's range for a pulse width.
const MIN_WIDTH_S: f64 = 10e-9;
const MAX_WIDTH_S: f64 = 10.0;
/// Widths checked on hardware (R6).
const VERIFIED_MIN_NS: f64 = 50_000.0;
const VERIFIED_MAX_NS: f64 = 5_000_000.0;

/// Edits the pulse fields of the model; any bad width rejects the whole patch.
pub fn apply(model: &mut Model, patch: &TriggerPatch) -> Result<(), KernelError> {
    if let Some(text) = &patch.width_text {
        model.pulse_width_s = parse_width(text, model.pulse_width_s)?;
    }
    model.pulse_condition = patch.condition.unwrap_or(model.pulse_condition);
    let is_pulse = patch
        .kind
        .map_or(model.is_pulse(), |kind| kind == TriggerKindName::Pulse);
    model.set_pulse(is_pulse);
    Ok(())
}

pub fn width_in_range(width_s: f64) -> bool {
    (MIN_WIDTH_S..=MAX_WIDTH_S).contains(&width_s)
}

fn parse_width(text: &str, applied_s: f64) -> Result<f64, KernelError> {
    match parse_seconds(text) {
        Ok(width) if width_in_range(width) => Ok(width),
        _ => {
            let applied = formatted(applied_s, Quantity::Seconds);
            let unit = applied.unit.replace("us", "\u{b5}s");
            Err(KernelError::recoverable(format!(
                "Invalid width. Enter a time such as 500 \u{b5}s. Applied: {} {unit}.",
                applied.text
            )))
        }
    }
}

pub fn kind_name(model: &Model) -> TriggerKindName {
    if model.is_pulse() {
        TriggerKindName::Pulse
    } else {
        TriggerKindName::Edge
    }
}

pub fn state(model: &Model) -> Option<PulseState> {
    let TriggerKind::Pulse { width_s, condition } = model.setup.trigger.kind else {
        return None;
    };
    Some(PulseState {
        condition,
        width: formatted(width_s, Quantity::Seconds),
        verified: (VERIFIED_MIN_NS..=VERIFIED_MAX_NS).contains(&(width_s * 1e9).round()),
    })
}
