use serde::{Deserialize, Serialize};
use sweepscope_kernel::contracts::{ConnectionSnapshot, Coupling, Slope, Sweep};
use sweepscope_kernel::{Event, Request};
use ts_rs::TS;

/// A number for geometry (`value`, SI) next to what the UI shows: the number part and the unit with prefix.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct FormattedValue {
    pub value: f64,
    pub text: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct Availability {
    pub enabled: bool,
    pub reason: Option<String>,
}

/// The current entry of a scale table. `faster` is a smaller volts or time per division.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ScaleChoice {
    pub index: u8,
    pub label: FormattedValue,
    pub verified: bool,
    pub faster: Availability,
    pub slower: Availability,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ChannelState {
    pub number: u8,
    pub enabled: bool,
    /// Label already multiplied by the probe factor.
    pub range: ScaleChoice,
    pub position: FormattedValue,
    pub position_divs: f64,
    pub max_position_divs: f64,
    pub coupling: Coupling,
    pub bandwidth_limit: bool,
    /// 1, 10 or 100.
    pub probe: f64,
    pub probe_text: String,
    pub invert: bool,
    /// Reason text when the range is outside 50 mV to 1 V per division.
    pub unverified: Option<String>,
    pub enable: Availability,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct HorizontalState {
    pub time_base: ScaleChoice,
    /// Percent of the screen.
    pub trigger_position: FormattedValue,
    pub trigger_position_fraction: f64,
    pub window_text: String,
    pub window_start_fraction: f64,
    pub window_size_fraction: f64,
    pub trigger_tick_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct TriggerState {
    pub source: u8,
    pub slope: Slope,
    pub sweep: Sweep,
    /// Probe-scaled volts.
    pub level: FormattedValue,
    /// From the screen centre, up is positive.
    pub level_divisions: f64,
    pub level_tag: String,
    pub position_tag: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ControlsAvailability {
    pub run: Availability,
    pub force: Availability,
    pub vertical: Availability,
    pub horizontal: Availability,
    pub trigger: Availability,
}

/// Instrument slice. `channels` always holds four entries.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct InstrumentState {
    #[ts(type = "number")]
    pub revision: u64,
    pub running: bool,
    pub channels: Vec<ChannelState>,
    pub horizontal: HorizontalState,
    pub trigger: TriggerState,
    pub controls: ControlsAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum Phase {
    Stopped,
    Armed,
    WaitingForTrigger,
    Triggered,
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct AcquisitionStatus {
    #[ts(type = "number")]
    pub revision: u64,
    pub phase: Phase,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct InstrumentFacts {
    #[ts(type = "number")]
    pub revision: u64,
    pub sample_rate: Option<FormattedValue>,
    pub sample_interval: Option<FormattedValue>,
    pub record_points: Option<FormattedValue>,
    pub capture_rate: Option<FormattedValue>,
    pub dead_time: Option<FormattedValue>,
}

/// What `get_snapshot` returns; the shell assembles it from `GetConnection` and `GetInstrument`.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct StateSnapshot {
    pub connection: ConnectionSnapshot,
    pub instrument: Option<InstrumentState>,
    pub status: AcquisitionStatus,
    pub facts: InstrumentFacts,
}

/// Optional fields: a missing field leaves the setting alone.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ChannelPatch {
    #[ts(optional)]
    pub enabled: Option<bool>,
    #[ts(optional)]
    pub position_divs: Option<f64>,
    #[ts(optional)]
    pub coupling: Option<Coupling>,
    #[ts(optional)]
    pub bandwidth_limit: Option<bool>,
    #[ts(optional)]
    pub probe: Option<f64>,
    #[ts(optional)]
    pub invert: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct TriggerPatch {
    #[ts(optional)]
    pub source: Option<u8>,
    #[ts(optional)]
    pub slope: Option<Slope>,
    #[ts(optional)]
    pub sweep: Option<Sweep>,
}

/// What `step_setting` and `reset_setting` act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum SettingTarget {
    Range { channel: u8 },
    TimeBase,
    TriggerPosition,
    TriggerLevel,
}

/// Events: each carries one whole slice; the shell forwards them as `instrument_changed`, `status_changed`, `facts_changed`.
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentChanged(pub InstrumentState);
impl Event for InstrumentChanged {}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusChanged(pub AcquisitionStatus);
impl Event for StatusChanged {}

#[derive(Debug, Clone, PartialEq)]
pub struct FactsChanged(pub InstrumentFacts);
impl Event for FactsChanged {}

#[derive(Debug, Clone, PartialEq)]
pub struct SetRunning {
    pub running: bool,
}
impl Request for SetRunning {
    type Response = ();
}

/// The shell's `force_trigger` command; `acquisition` forwards it to `device` as the kernel `ForceTrigger`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForceTriggerNow;
impl Request for ForceTriggerNow {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetChannel {
    pub channel: u8,
    pub patch: ChannelPatch,
}
impl Request for SetChannel {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepSetting {
    pub target: SettingTarget,
    pub direction: i8,
}
impl Request for StepSetting {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResetSetting {
    pub target: SettingTarget,
}
impl Request for ResetSetting {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTriggerPosition {
    pub fraction: f64,
}
impl Request for SetTriggerPosition {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTriggerLevel {
    pub divisions: f64,
}
impl Request for SetTriggerLevel {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTrigger {
    pub patch: TriggerPatch,
}
impl Request for SetTrigger {
    type Response = ();
}

/// Everything of `StateSnapshot` that `acquisition` owns.
#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentSnapshot {
    pub instrument: Option<InstrumentState>,
    pub status: AcquisitionStatus,
    pub facts: InstrumentFacts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetInstrument;
impl Request for GetInstrument {
    type Response = InstrumentSnapshot;
}
