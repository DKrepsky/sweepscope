use serde::Serialize;
use sweepscope_kernel::contracts::FormattedValue;
use sweepscope_kernel::{Event, Request};
use ts_rs::TS;

/// The twelve quantities, in the order of `ChannelMeasurements::measurements`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum MeasurementId {
    Frequency,
    Period,
    Vpp,
    Vmax,
    Vmin,
    Vmean,
    Vrms,
    PositiveWidth,
    NegativeWidth,
    DutyCycle,
    RiseTime,
    FallTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub enum InvalidReason {
    NoSignal,
    FewerThanOnePeriod,
    NoCompleteEdge,
    EdgeTooFast,
    NoCompletePulse,
    Clipped,
}

/// Why a cell shows `-`: a code for logic and the text the UI shows as a footnote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct Invalid {
    pub reason: InvalidReason,
    pub text: String,
}

/// Exactly one of `value` and `invalid` is set.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct Measurement {
    pub id: MeasurementId,
    pub label: String,
    pub value: Option<FormattedValue>,
    pub invalid: Option<Invalid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct ChannelMeasurements {
    pub channel: u8,
    /// Any sample of the record sits on a converter rail.
    pub clipped: bool,
    /// Reason text when the range is outside the verified 50 mV to 1 V per division.
    pub unverified: Option<String>,
    /// Always twelve, in the order of `MeasurementId`.
    pub measurements: Vec<Measurement>,
}

/// Measurements slice: the enabled channels of the newest frame, ascending.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "../../../../frontend/src/api/generated/")]
pub struct MeasurementsState {
    #[ts(type = "number")]
    pub revision: u64,
    pub channels: Vec<ChannelMeasurements>,
    pub accuracy_note: String,
}

/// Published at most four times a second, only when a text changed; the shell forwards it as `measurements_changed`.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementsChanged(pub MeasurementsState);
impl Event for MeasurementsChanged {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetMeasurements;
impl Request for GetMeasurements {
    type Response = MeasurementsState;
}
