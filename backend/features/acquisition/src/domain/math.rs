//! The math channel (ADR 0019): what the user chose, the published slice and the trace of one frame.
use engineering_format::Quantity;
use math_channel::Operation;
use sweepscope_kernel::contracts::{Capabilities, DerivedTrace, DerivedUnit, Frame};
use sweepscope_kernel::KernelError;

use super::model::{Model, CHANNELS};
use super::view::{formatted, VIEWS_IN_ROLL};
use crate::{Availability, MathOperation, MathPatch, MathState, MathStatus, ScaleChoice, SettingTarget};

const MANTISSAS: [f64; 3] = [1.0, 2.0, 5.0];
const FIRST_DECADE: i32 = -3;
const DECADES: i32 = 6;
const DEFAULT_SCALE_INDEX: usize = 9;
const POSITION_LIMIT_DIVS: f64 = 4.0;
const POSITION_STEP_DIVS: f64 = 0.2;
const SAME_SCALE_TOLERANCE: f64 = 1e-9;

/// What the user chose for the one math channel; sources are channel numbers, 1 to 4.
#[derive(Debug, Clone, PartialEq)]
pub struct MathSetting {
    pub enabled: bool,
    pub operation: MathOperation,
    pub source_a: u8,
    pub source_b: u8,
    pub scale_index: usize,
    pub position_divs: f64,
}

impl Default for MathSetting {
    fn default() -> Self {
        Self {
            enabled: false,
            operation: MathOperation::Subtract,
            source_a: 1,
            source_b: 2,
            scale_index: DEFAULT_SCALE_INDEX,
            position_divs: 0.0,
        }
    }
}

impl MathSetting {
    /// The setting a fresh model starts with: scale fitted to the sources.
    pub fn fitted(caps: &Capabilities, model: &Model) -> Self {
        let defaults = Self::default();
        Self {
            scale_index: fitted_scale_index(caps, model, &defaults),
            ..defaults
        }
    }
}

/// 1-2-5 from 1 mV to 500, per division.
fn scale_table() -> Vec<f64> {
    (0..DECADES)
        .flat_map(|decade| MANTISSAS.map(|mantissa| (mantissa, FIRST_DECADE + decade)))
        .map(|(mantissa, exponent)| {
            if exponent < 0 {
                mantissa / 10f64.powi(-exponent)
            } else {
                mantissa * 10f64.powi(exponent)
            }
        })
        .collect()
}

fn needs_b(operation: MathOperation) -> bool {
    operation != MathOperation::InvertA
}

fn unit_quantity(operation: MathOperation) -> Quantity {
    match operation {
        MathOperation::Multiply => Quantity::VoltsSquaredPerDiv,
        _ => Quantity::VoltsPerDiv,
    }
}

fn unit_text(operation: MathOperation) -> &'static str {
    match operation {
        MathOperation::Multiply => "V\u{b2}",
        _ => "V",
    }
}

fn label(operation: MathOperation) -> &'static str {
    match operation {
        MathOperation::Add => "A+B",
        MathOperation::Subtract => "A\u{2212}B",
        MathOperation::Multiply => "A\u{d7}B",
        MathOperation::InvertA => "Invert A",
    }
}

fn source_index(number: u8) -> usize {
    usize::from(number) - 1
}

/// Volts per division of a source as the user reads it (probe included).
fn source_scale(caps: &Capabilities, model: &Model, number: u8) -> f64 {
    let index = source_index(number);
    model.range(caps, index).map_or(1.0, |r| r.volts_per_div) * model.probes[index]
}

fn scale_text(volts_per_div: f64) -> String {
    let f = formatted(volts_per_div, Quantity::VoltsPerDiv);
    format!("{} {}", f.text, f.unit)
}

fn fitted_scale_index(caps: &Capabilities, model: &Model, setting: &MathSetting) -> usize {
    let a = source_scale(caps, model, setting.source_a);
    let target = match setting.operation {
        MathOperation::InvertA => a,
        MathOperation::Multiply => a * source_scale(caps, model, setting.source_b),
        _ => a.max(source_scale(caps, model, setting.source_b)),
    };
    let table = scale_table();
    table
        .iter()
        .position(|&value| value >= target * (1.0 - SAME_SCALE_TOLERANCE))
        .unwrap_or(table.len() - 1)
}

fn source_off(model: &Model, number: u8) -> bool {
    !model.setup.channels[source_index(number)].enabled
}

fn status(caps: &Capabilities, model: &Model, scale: f64) -> MathStatus {
    let setting = &model.math;
    if !setting.enabled {
        return MathStatus::Ok;
    }
    if model.in_roll(caps) {
        return MathStatus::Unavailable {
            reason: VIEWS_IN_ROLL.to_owned(),
        };
    }
    let sources = [
        Some(setting.source_a),
        needs_b(setting.operation).then_some(setting.source_b),
    ];
    if let Some(number) = sources.into_iter().flatten().find(|&n| source_off(model, n)) {
        return MathStatus::Unavailable {
            reason: format!("CH{number} is off"),
        };
    }
    mixed(caps, model, scale).unwrap_or(MathStatus::Ok)
}

fn mixed(caps: &Capabilities, model: &Model, scale: f64) -> Option<MathStatus> {
    let setting = &model.math;
    if !matches!(setting.operation, MathOperation::Add | MathOperation::Subtract) {
        return None;
    }
    let a = source_scale(caps, model, setting.source_a);
    let b = source_scale(caps, model, setting.source_b);
    if (a / b - 1.0).abs() < SAME_SCALE_TOLERANCE {
        return None;
    }
    Some(MathStatus::Mixed {
        detail: format!(
            "CH{} {}, CH{} {}. The math trace has its own scale, {}.",
            setting.source_a,
            scale_text(a),
            setting.source_b,
            scale_text(b),
            scale_text(scale),
        ),
    })
}

fn step_availability(open: bool, reason: &str, gate: &impl Fn(Availability) -> Availability) -> Availability {
    gate(Availability {
        enabled: open,
        reason: (!open).then(|| reason.to_owned()),
    })
}

fn scale_of(setting: &MathSetting) -> (usize, f64) {
    let table = scale_table();
    let index = setting.scale_index.min(table.len() - 1);
    (index, table[index])
}

pub fn state(caps: &Capabilities, model: &Model, gate: &impl Fn(Availability) -> Availability) -> MathState {
    let setting = &model.math;
    let (index, scale) = scale_of(setting);
    let last = scale_table().len() - 1;
    MathState {
        enabled: setting.enabled,
        operation: setting.operation,
        source_a: setting.source_a,
        source_b: needs_b(setting.operation).then_some(setting.source_b),
        scale: ScaleChoice {
            index: index as u8,
            label: formatted(scale, unit_quantity(setting.operation)),
            faster: step_availability(index > 0, "Smallest scale", gate),
            slower: step_availability(index < last, "Largest scale", gate),
        },
        unit: unit_text(setting.operation).to_owned(),
        position: formatted(setting.position_divs, Quantity::Divisions),
        position_divs: setting.position_divs,
        status: status(caps, model, scale),
        label: label(setting.operation).to_owned(),
    }
}

/// Applies a patch. A first enable picks the default sources when the chosen ones are not both on; a first
/// enable or another operation fits the scale to the sources.
pub fn apply(caps: &Capabilities, model: &mut Model, patch: &MathPatch) -> Result<(), KernelError> {
    validate(patch)?;
    if patch.enabled == Some(true) && model.in_roll(caps) {
        return Err(KernelError::recoverable(VIEWS_IN_ROLL));
    }
    let was_enabled = model.math.enabled;
    let operation = model.math.operation;
    let setting = &mut model.math;
    setting.enabled = patch.enabled.unwrap_or(setting.enabled);
    setting.operation = patch.operation.unwrap_or(setting.operation);
    setting.source_a = patch.source_a.unwrap_or(setting.source_a);
    setting.source_b = patch.source_b.unwrap_or(setting.source_b);
    if let Some(position) = patch.position_divs {
        setting.position_divs = clamp_position(position);
    }
    let enabling = !was_enabled && model.math.enabled;
    if enabling {
        choose_default_sources(model);
    }
    if enabling || model.math.operation != operation {
        model.math.scale_index = fitted_scale_index(caps, model, &model.math);
    }
    Ok(())
}

fn validate(patch: &MathPatch) -> Result<(), KernelError> {
    for source in [patch.source_a, patch.source_b].into_iter().flatten() {
        if !(1..=CHANNELS as u8).contains(&source) {
            return Err(KernelError::recoverable("The channel must be 1 to 4"));
        }
    }
    match patch.position_divs {
        Some(position) if !position.is_finite() => Err(KernelError::recoverable("The value must be a number")),
        _ => Ok(()),
    }
}

fn choose_default_sources(model: &mut Model) {
    let setting = &model.math;
    let a_on = !source_off(model, setting.source_a);
    let b_on = !needs_b(setting.operation) || !source_off(model, setting.source_b);
    if a_on && b_on {
        return;
    }
    let enabled: Vec<u8> = (1..=CHANNELS as u8).filter(|&n| !source_off(model, n)).collect();
    if let [first, second, ..] = enabled[..] {
        model.math.source_a = first;
        model.math.source_b = second;
    }
}

fn clamp_position(divisions: f64) -> f64 {
    round(divisions.clamp(-POSITION_LIMIT_DIVS, POSITION_LIMIT_DIVS))
}

fn round(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}

/// Steps the scale or position of the math channel; `target` is `MathScale` or `MathPosition`.
pub fn step(model: &mut Model, target: SettingTarget, direction: i8) {
    let setting = &mut model.math;
    match target {
        SettingTarget::MathScale => {
            let last = scale_table().len() - 1;
            let next = setting.scale_index.saturating_add_signed(isize::from(direction));
            setting.scale_index = next.min(last);
        }
        SettingTarget::MathPosition => {
            setting.position_divs = clamp_position(setting.position_divs + f64::from(direction) * POSITION_STEP_DIVS);
        }
        _ => {}
    }
}

/// The scale returns to the fit for the current sources, the position to the centre.
pub fn reset(caps: &Capabilities, model: &mut Model, target: SettingTarget) {
    match target {
        SettingTarget::MathScale => model.math.scale_index = fitted_scale_index(caps, model, &model.math),
        SettingTarget::MathPosition => model.math.position_divs = 0.0,
        _ => {}
    }
}

fn operation_of(operation: MathOperation) -> Operation {
    match operation {
        MathOperation::Add => Operation::Add,
        MathOperation::Subtract => Operation::Subtract,
        MathOperation::Multiply => Operation::Multiply,
        MathOperation::InvertA => Operation::InvertA,
    }
}

fn probe_scaled_volts(frame: &Frame, model: &Model, number: u8) -> Option<Vec<f64>> {
    let index = source_index(number);
    let data = frame.channels[index].as_ref()?;
    let volts_per_count = data.volts_per_div * model.probes[index] / frame.counts_per_div;
    Some(
        data.codes
            .iter()
            .map(|&code| (f64::from(code) - data.zero_code) * volts_per_count)
            .collect(),
    )
}

/// `None` while math is off or unavailable, or when a source is missing from the frame.
pub fn derive(caps: &Capabilities, model: &Model, frame: &Frame) -> Option<DerivedTrace> {
    let setting = &model.math;
    let (_, scale) = scale_of(setting);
    let usable = setting.enabled && !matches!(status(caps, model, scale), MathStatus::Unavailable { .. });
    if !usable {
        return None;
    }
    let a = probe_scaled_volts(frame, model, setting.source_a)?;
    let b = if needs_b(setting.operation) {
        Some(probe_scaled_volts(frame, model, setting.source_b)?)
    } else {
        None
    };
    let volts = math_channel::apply(operation_of(setting.operation), &a, b.as_deref())?;
    let unit = match setting.operation {
        MathOperation::Multiply => DerivedUnit::VoltsSquared,
        _ => DerivedUnit::Volts,
    };
    Some(DerivedTrace {
        volts,
        unit,
        volts_per_div: scale,
        position_divs: setting.position_divs,
    })
}
