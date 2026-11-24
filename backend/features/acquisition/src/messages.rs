use serde::{Deserialize, Serialize};
use sweepscope_kernel::contracts::{Availability, Coupling, FormattedValue, PulseCondition, SavedFile, Slope, Sweep};
use sweepscope_kernel::{Event, Request};
use ts_rs::TS;

/// The current entry of a scale table. `faster` is a smaller volts or time per division.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScaleChoice {
    pub index: u8,
    pub label: FormattedValue,
    pub faster: Availability,
    pub slower: Availability,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
    pub enable: Availability,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HorizontalState {
    pub time_base: ScaleChoice,
    /// Percent of the screen.
    pub trigger_position: FormattedValue,
    pub trigger_position_fraction: f64,
    pub window_text: String,
    pub window_start_fraction: f64,
    pub window_size_fraction: f64,
    pub trigger_tick_fraction: f64,
    /// Roll mode (500 ms/div and slower): no trigger, no frames, items stream instead (ADR 0017).
    pub roll: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
    pub kind: TriggerKindName,
    /// Set while `kind` is `pulse`.
    pub pulse: Option<PulseState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TriggerKindName {
    Edge,
    Pulse,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PulseState {
    pub condition: PulseCondition,
    pub width: FormattedValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ControlsAvailability {
    pub run: Availability,
    pub force: Availability,
    pub vertical: Availability,
    pub horizontal: Availability,
    pub trigger: Availability,
    /// Save and load of setup files.
    pub files: Availability,
    pub single: Availability,
    pub autoset: Availability,
    pub record: Availability,
    /// XY, FFT, persistence, math, cursors and measurements need a captured record; not available in roll.
    pub frame_views: Availability,
}

/// Instrument slice. `channels` always holds four entries.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InstrumentState {
    #[ts(type = "number")]
    pub revision: u64,
    pub running: bool,
    pub channels: Vec<ChannelState>,
    pub horizontal: HorizontalState,
    pub trigger: TriggerState,
    pub controls: ControlsAvailability,
    /// Set once when the last setup could not be restored completely (ADR 0016 item 6).
    pub setup_notice: Option<SetupNotice>,
    pub math: MathState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    InvertA,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum MathStatus {
    Ok,
    /// The sources have very different scales; `detail` says which.
    Mixed {
        detail: String,
    },
    Unavailable {
        reason: String,
    },
}

/// The one math channel (ADR 0019); `label` reads like `A−B`.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MathState {
    pub enabled: bool,
    pub operation: MathOperation,
    /// Channel numbers, 1 to 4.
    pub source_a: u8,
    pub source_b: Option<u8>,
    pub scale: ScaleChoice,
    pub unit: String,
    pub position: FormattedValue,
    pub position_divs: f64,
    pub status: MathStatus,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MathPatch {
    #[ts(optional)]
    pub enabled: Option<bool>,
    #[ts(optional)]
    pub operation: Option<MathOperation>,
    #[ts(optional)]
    pub source_a: Option<u8>,
    #[ts(optional)]
    pub source_b: Option<u8>,
    #[ts(optional)]
    pub position_divs: Option<f64>,
}

/// A message the UI shows once per `id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupNotice {
    #[ts(type = "number")]
    pub id: u64,
    pub text: String,
}

/// Result of `load_setup`: values the device cannot do are listed in `warnings` and keep their current setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupLoaded {
    pub file_name: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Phase {
    Stopped,
    Armed,
    WaitingForTrigger,
    Triggered,
    Auto,
    /// Single finished.
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcquisitionStatus {
    #[ts(type = "number")]
    pub revision: u64,
    pub phase: Phase,
    pub autoset: AutosetStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AutosetStatus {
    pub running: bool,
    /// Set by the last auto-setup that saw a flat input, until a frame with signal arrives.
    pub no_signal: bool,
    pub outcome: Option<AutosetOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AutosetOutcome {
    /// The UI shows each outcome once per `id`.
    #[ts(type = "number")]
    pub id: u64,
    pub kind: AutosetOutcomeKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AutosetOutcomeKind {
    Done,
    NoSignal,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InstrumentFacts {
    #[ts(type = "number")]
    pub revision: u64,
    pub sample_rate: Option<FormattedValue>,
    pub sample_interval: Option<FormattedValue>,
    pub record_points: Option<FormattedValue>,
    pub capture_rate: Option<FormattedValue>,
    pub dead_time: Option<FormattedValue>,
}

/// Optional fields: a missing field leaves the setting alone.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
#[ts(export)]
pub struct TriggerPatch {
    #[ts(optional)]
    pub source: Option<u8>,
    #[ts(optional)]
    pub slope: Option<Slope>,
    #[ts(optional)]
    pub sweep: Option<Sweep>,
    #[ts(optional)]
    pub kind: Option<TriggerKindName>,
    #[ts(optional)]
    pub condition: Option<PulseCondition>,
    /// Text such as `500 us`; text that does not parse rejects the whole patch.
    #[ts(optional)]
    pub width_text: Option<String>,
}

/// What `step_setting` and `reset_setting` act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum SettingTarget {
    Range { channel: u8 },
    TimeBase,
    TriggerPosition,
    TriggerLevel,
    MathScale,
    MathPosition,
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

/// Arms one capture; the first triggered frame ends it with phase `Done`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmSingle;
impl Request for ArmSingle {
    type Response = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartAutoset;
impl Request for StartAutoset {
    type Response = ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetMath {
    pub patch: MathPatch,
}
impl Request for SetMath {
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

/// Asks for a file name and writes the current setup; `None` when the dialog is cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveSetup;
impl Request for SaveSetup {
    type Response = Option<SavedFile>;
}

/// Asks for a file and applies it to the connected device; `None` when the dialog is cancelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadSetup;
impl Request for LoadSetup {
    type Response = Option<SetupLoaded>;
}
