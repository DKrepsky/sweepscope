//! The math channel (ADR 0019). Inert until its own package fills it in: the slice reports a disabled `A-B`.
use engineering_format::Quantity;

use super::model::Model;
use super::view::formatted;
use crate::{Availability, MathOperation, MathState, MathStatus, ScaleChoice, SettingTarget};

const LABEL: &str = "A\u{2212}B";
const UNAVAILABLE: &str = "Math is not available yet";

pub fn state(_model: &Model) -> MathState {
    MathState {
        enabled: false,
        operation: MathOperation::Subtract,
        source_a: 1,
        source_b: Some(2),
        scale: ScaleChoice {
            index: 0,
            label: formatted(1.0, Quantity::VoltsPerDiv),
            verified: true,
            faster: closed(),
            slower: closed(),
        },
        unit: "V".to_owned(),
        position: formatted(0.0, Quantity::Divisions),
        position_divs: 0.0,
        status: MathStatus::Unavailable {
            reason: UNAVAILABLE.to_owned(),
        },
        label: LABEL.to_owned(),
    }
}

/// Steps the scale or position of the math channel; `target` is `MathScale` or `MathPosition`.
pub fn step(_model: &mut Model, _target: SettingTarget, _direction: i8) {}

pub fn reset(_model: &mut Model, _target: SettingTarget) {}

fn closed() -> Availability {
    Availability {
        enabled: false,
        reason: Some(UNAVAILABLE.to_owned()),
    }
}
