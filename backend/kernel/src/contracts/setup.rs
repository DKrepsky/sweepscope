use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Requested instrument state. Time base and range are indexes into `Capabilities`; the device converts volts to its own codes.
#[derive(Debug, Clone, PartialEq)]
pub struct AcquisitionSetup {
    pub channels: [ChannelSetup; 4],
    pub time_base_index: u8,
    pub trigger: TriggerSetup,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSetup {
    pub enabled: bool,
    pub range_index: u8,
    pub position_divs: f64,
    pub coupling: Coupling,
    pub bandwidth_limit: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriggerSetup {
    /// Zero-based channel index.
    pub source: u8,
    pub level_volts: f64,
    pub slope: Slope,
    pub sweep: Sweep,
    pub horizontal_position_percent: f64,
    pub kind: TriggerKind,
}

/// `slope` is the polarity of the pulse for `Pulse`; `width_s` is in seconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerKind {
    Edge,
    Pulse { width_s: f64, condition: PulseCondition },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PulseCondition {
    Equal,
    NotEqual,
    Greater,
    Less,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Coupling {
    Ac,
    Dc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Slope {
    Rising,
    Falling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Sweep {
    Auto,
    Normal,
}
