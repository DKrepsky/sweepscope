//! The setup file (ADR 0016 items 4 and 5): values in volts and seconds, never device indexes, so a file stays
//! meaningful on another device. Parsing validates the whole file before anything is applied.
use engineering_format::{format, Quantity};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sweepscope_kernel::contracts::{Capabilities, Coupling, Slope, Sweep};
use sweepscope_kernel::KernelError;

use super::instrument::{PROBES, SCREEN_HALF_HEIGHT_DIVS};
use super::model::{Model, CHANNELS};

pub const FORMAT: &str = "sweepscope-setup";
pub const VERSION: u64 = 1;
const MATCH_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupFile {
    format: String,
    version: u64,
    channels: Vec<ChannelEntry>,
    time_base_s_per_div: f64,
    trigger: TriggerEntry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ChannelEntry {
    channel: u8,
    enabled: bool,
    volts_per_div: f64,
    position_divs: f64,
    coupling: Coupling,
    bandwidth_limit: bool,
    probe: f64,
    invert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TriggerEntry {
    /// Channel number, 1 to 4.
    source: u8,
    slope: Slope,
    sweep: Sweep,
    level_volts: f64,
    position_percent: f64,
}

impl SetupFile {
    /// `None` when the model points at a range or time base the capabilities do not have.
    pub fn from_model(caps: &Capabilities, model: &Model) -> Option<Self> {
        let channels = (0..CHANNELS)
            .map(|n| channel_entry(caps, model, n))
            .collect::<Option<Vec<_>>>()?;
        let time_base = caps.time_bases.get(usize::from(model.setup.time_base_index))?;
        let trigger = &model.setup.trigger;
        Some(Self {
            format: FORMAT.to_owned(),
            version: VERSION,
            channels,
            time_base_s_per_div: time_base.seconds_per_div,
            trigger: TriggerEntry {
                source: trigger.source + 1,
                slope: trigger.slope,
                sweep: trigger.sweep,
                level_volts: trigger.level_volts,
                position_percent: round6(model.trigger_fraction * 100.0),
            },
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = serde_json::to_vec_pretty(self).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, KernelError> {
        let value: Value = serde_json::from_slice(bytes)
            .map_err(|_| KernelError::recoverable("The file is not valid JSON, so it is not a setup file"))?;
        if value.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(KernelError::recoverable(
                "The file is not a SweepScope setup (its format is missing or different)",
            ));
        }
        match value.get("version").and_then(Value::as_u64) {
            Some(VERSION) => {}
            Some(newer) if newer > VERSION => {
                return Err(KernelError::recoverable(format!(
                    "The setup file was written by a newer SweepScope (version {newer})"
                )))
            }
            _ => return Err(KernelError::recoverable("The setup file has an unsupported version")),
        }
        let file: Self = serde_json::from_value(value).map_err(|error| {
            KernelError::recoverable(format!("The setup file is incomplete or has bad values: {error}"))
        })?;
        file.check_structure()?;
        Ok(file)
    }

    fn check_structure(&self) -> Result<(), KernelError> {
        let in_order = self
            .channels
            .iter()
            .zip(1u8..)
            .all(|(entry, number)| entry.channel == number);
        if self.channels.len() != CHANNELS || !in_order {
            return Err(KernelError::recoverable(
                "The setup file must list channels 1 to 4 in order",
            ));
        }
        if !(1..=CHANNELS as u8).contains(&self.trigger.source) {
            return Err(KernelError::recoverable(
                "The trigger source in the setup file must be 1 to 4",
            ));
        }
        if !self.channels.iter().any(|c| c.enabled) {
            return Err(KernelError::recoverable("The setup file enables no channel"));
        }
        if !self.numbers_are_finite() {
            return Err(KernelError::recoverable(
                "The setup file holds a value that is not a number",
            ));
        }
        Ok(())
    }

    fn numbers_are_finite(&self) -> bool {
        let channels = self
            .channels
            .iter()
            .all(|c| c.volts_per_div.is_finite() && c.position_divs.is_finite() && c.probe.is_finite());
        channels
            && self.time_base_s_per_div.is_finite()
            && self.trigger.level_volts.is_finite()
            && self.trigger.position_percent.is_finite()
    }
}

fn channel_entry(caps: &Capabilities, model: &Model, number: usize) -> Option<ChannelEntry> {
    let channel = &model.setup.channels[number];
    Some(ChannelEntry {
        channel: number as u8 + 1,
        enabled: channel.enabled,
        volts_per_div: model.range(caps, number)?.volts_per_div,
        position_divs: channel.position_divs,
        coupling: channel.coupling,
        bandwidth_limit: channel.bandwidth_limit,
        probe: model.probes[number],
        invert: model.invert[number],
    })
}

/// Applies what the device can do and returns one warning for every value it cannot (those keep their current setting).
pub fn apply(file: &SetupFile, caps: &Capabilities, model: &mut Model) -> Vec<String> {
    let mut warnings = Vec::new();
    for entry in &file.channels {
        apply_channel(entry, caps, model, &mut warnings);
    }
    apply_time_base(file.time_base_s_per_div, caps, model, &mut warnings);
    apply_trigger(&file.trigger, caps, model, &mut warnings);
    warnings
}

fn apply_channel(entry: &ChannelEntry, caps: &Capabilities, model: &mut Model, warnings: &mut Vec<String>) {
    let number = usize::from(entry.channel) - 1;
    let label = format!("CH{}", entry.channel);
    let setup = &mut model.setup.channels[number];
    setup.enabled = entry.enabled;
    setup.coupling = entry.coupling;
    setup.bandwidth_limit = entry.bandwidth_limit;
    setup.position_divs = entry.position_divs;
    model.invert[number] = entry.invert;
    apply_range(entry, caps, model, warnings);
    if PROBES.contains(&entry.probe) {
        model.probes[number] = entry.probe;
    } else {
        let kept = model.probes[number];
        warnings.push(format!("{label} probe {}x is not available; kept {kept}x", entry.probe));
    }
    let wanted = entry.position_divs;
    model.clamp_position(caps, number);
    let limited = model.setup.channels[number].position_divs;
    if limited != wanted {
        warnings.push(format!(
            "{label} position {wanted:.2} div is out of range; limited to {limited:.2} div"
        ));
    }
}

fn apply_range(entry: &ChannelEntry, caps: &Capabilities, model: &mut Model, warnings: &mut Vec<String>) {
    let number = usize::from(entry.channel) - 1;
    let ranges = caps.vertical_ranges.iter().map(|r| r.volts_per_div);
    match matching_index(ranges, entry.volts_per_div) {
        Some(index) => model.setup.channels[number].range_index = index,
        None => {
            let kept = model.range(caps, number).map_or(0.0, |r| r.volts_per_div);
            let what = format!("CH{} volts/div", entry.channel);
            warnings.push(unavailable(&what, entry.volts_per_div, Quantity::Volts, kept));
        }
    }
}

fn apply_time_base(seconds_per_div: f64, caps: &Capabilities, model: &mut Model, warnings: &mut Vec<String>) {
    let usable = caps
        .time_bases
        .iter()
        .map(|t| if t.streaming { f64::NAN } else { t.seconds_per_div });
    match matching_index(usable, seconds_per_div) {
        Some(index) => model.setup.time_base_index = index,
        None => {
            let kept = caps
                .time_bases
                .get(usize::from(model.setup.time_base_index))
                .map_or(0.0, |t| t.seconds_per_div);
            warnings.push(unavailable("Time/div", seconds_per_div, Quantity::Seconds, kept));
        }
    }
}

fn apply_trigger(entry: &TriggerEntry, caps: &Capabilities, model: &mut Model, warnings: &mut Vec<String>) {
    let trigger = &mut model.setup.trigger;
    trigger.slope = entry.slope;
    trigger.sweep = entry.sweep;
    trigger.level_volts = entry.level_volts;
    trigger.source = entry.source - 1;
    model.trigger_fraction = round6(entry.position_percent / 100.0).clamp(0.0, 1.0);
    let asked = model.setup.trigger.source;
    model.keep_source_enabled();
    if model.setup.trigger.source != asked {
        warnings.push(format!(
            "Trigger source CH{} is not enabled; kept CH{}",
            asked + 1,
            model.setup.trigger.source + 1
        ));
    }
    let divisions = model.level_divisions(caps);
    if divisions.abs() > SCREEN_HALF_HEIGHT_DIVS {
        let limited = divisions.clamp(-SCREEN_HALF_HEIGHT_DIVS, SCREEN_HALF_HEIGHT_DIVS);
        model.set_level_divisions(caps, limited);
        warnings.push("Trigger level is off the screen; limited to the screen edge".to_owned());
    }
    model.settle(caps);
}

fn matching_index(values: impl Iterator<Item = f64>, wanted: f64) -> Option<u8> {
    let position = values
        .into_iter()
        .position(|v| (v / wanted - 1.0).abs() <= MATCH_TOLERANCE);
    position.map(|index| index as u8)
}

fn unavailable(what: &str, wanted: f64, quantity: Quantity, kept: f64) -> String {
    format!(
        "{what} {} is not available; kept {}",
        shown(wanted, quantity),
        shown(kept, quantity)
    )
}

fn shown(value: f64, quantity: Quantity) -> String {
    let formatted = format(value, quantity);
    format!("{} {}", formatted.text, formatted.unit)
}

fn round6(value: f64) -> f64 {
    (value * 1e6).round() / 1e6
}
