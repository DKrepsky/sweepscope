use serde::Serialize;
use sweepscope_kernel::contracts::{Availability, FormattedValue};
use sweepscope_kernel::{Event, Request};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ZeroStep {
    Intro,
    Channel,
    Measuring,
    NotShorted,
    Done,
    Failed,
}

/// One row of the Done table: a range and the stored offset of each channel (`None` when the channel has none).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OffsetRow {
    pub range: FormattedValue,
    pub offsets: [Option<FormattedValue>; 4],
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ZeroSession {
    pub step: ZeroStep,
    /// 1 to 4 while a channel is in turn.
    pub channel: Option<u8>,
    pub done_channels: [bool; 4],
    /// Share of the ranges of the current channel that are measured, 0 to 1.
    pub progress: Option<f64>,
    pub not_shorted: Option<Vec<String>>,
    /// True at `NotShorted` when `ContinueZeroChannel` may accept the rejected reading.
    pub can_continue: bool,
    /// Channels that were stored although the shorted-input check failed.
    pub overridden: [bool; 4],
    pub failure: Option<String>,
    /// One row per range, ascending; set at Done only.
    pub table: Option<Vec<OffsetRow>>,
}

/// Zero error texts of one channel: one cell per range index, one string per probe factor (1x, 10x, 100x).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChannelTooltip {
    pub cells: Vec<[String; 3]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TooltipText {
    pub heading: String,
    pub channels: [ChannelTooltip; 4],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UnitCalibration {
    pub calibrated: bool,
    /// ISO 8601 UTC; the frontend formats the date.
    pub stored_at_utc: Option<String>,
    pub tooltip: TooltipText,
}

/// Zero calibration slice (ADR 0013): revisioned, announced when it changes.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ZeroCalibrationState {
    #[ts(type = "number")]
    pub revision: u64,
    pub availability: Availability,
    pub unit: Option<UnitCalibration>,
    pub session: Option<ZeroSession>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZeroCalibrationChanged(pub ZeroCalibrationState);
impl Event for ZeroCalibrationChanged {}

macro_rules! unit_request {
    ($($(#[$doc:meta])* $name:ident),+) => {$(
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name;
        impl Request for $name {
            type Response = ();
        }
    )+};
}

unit_request!(
    /// Opens the dialog: holds the instrument and shows the intro.
    OpenZeroCalibration,
    /// Intro to the first channel.
    StartZeroCalibration,
    /// Measures the channel in turn; returns at once, progress arrives in the slice.
    MeasureZeroChannel,
    /// At `NotShorted`: accepts the rejected reading and measures the remaining ranges without re-measuring.
    ContinueZeroChannel,
    /// Cancels or closes; stores nothing unless the session is Done, releases the hold.
    CloseZeroCalibration
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetZeroCalibration;
impl Request for GetZeroCalibration {
    type Response = ZeroCalibrationState;
}
