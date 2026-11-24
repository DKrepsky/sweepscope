use std::sync::Arc;

use super::{AcquisitionSetup, Capabilities, ConnectionSnapshot, Frame};
use crate::{Event, Request};

/// Published by `device` whenever the connection slice changes; `capabilities` is set while a device is ready.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionChanged {
    pub snapshot: ConnectionSnapshot,
    pub capabilities: Option<Capabilities>,
}
impl Event for ConnectionChanged {}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureOutcome {
    Frame(Arc<Frame>),
    /// Normal sweep: no trigger yet.
    TimedOut,
}

/// Published by the `device` worker; `setup_revision` is the revision of the setup the capture ran with.
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureCompleted {
    pub setup_revision: u64,
    pub outcome: CaptureOutcome,
}
impl Event for CaptureCompleted {}

/// What `display` needs to turn a frame into the visible window: crop and invert come from the acquisition state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameContext {
    pub setup_revision: u64,
    pub window_start: usize,
    pub window_len: usize,
    pub invert: [bool; 4],
    /// Probe factor per channel (1, 10 or 100), for the volts conversion in `measurement` and `export`.
    pub probe: [f64; 4],
    /// True when zero offsets of the connected unit were applied to the channel.
    pub calibrated: [bool; 4],
    pub seconds_per_div: f64,
}

/// Published by `acquisition` for every accepted frame; latest-wins subscribers only see the newest.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameCaptured {
    pub frame: Arc<Frame>,
    pub context: FrameContext,
    /// The math trace of the same record (ADR 0019); `None` while math is off or unavailable.
    pub derived: Option<Arc<DerivedTrace>>,
}
impl Event for FrameCaptured {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedUnit {
    Volts,
    VoltsSquared,
}

/// Probe-scaled result of the math channel, one value per sample of the record.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedTrace {
    pub volts: Vec<f64>,
    pub unit: DerivedUnit,
    pub volts_per_div: f64,
    pub position_divs: f64,
}

/// What the consumers of roll items need to scale them; `session` changes whenever the setup does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RollContext {
    pub session: u32,
    pub invert: [bool; 4],
    pub probe: [f64; 4],
    pub seconds_per_div: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RollPolledItem {
    Samples { first_index: u64, frame: Arc<Frame> },
    Gap { after: u64, missed: u64 },
}

/// Published by the `device` worker while it rolls. Indices count from 0 at the start of a session; the session itself is assigned by `acquisition`.
#[derive(Debug, Clone, PartialEq)]
pub struct RollPolled {
    pub setup_revision: u64,
    pub item: RollPolledItem,
}
impl Event for RollPolled {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollCloseReason {
    Stopped,
    LeftRoll,
    Disconnected,
    SetupChanged,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RollItem {
    Opened,
    Samples { first_index: u64, frame: Arc<Frame> },
    Gap { after: u64, missed: u64 },
    Closed { reason: RollCloseReason },
}

/// Published by `acquisition` for `display` and `recording`. Consumers keep an expected index: samples that start later
/// are preceded by a `Gap { after: expected - 1, missed: first_index - expected }`; a `Gap` moves it to `after + 1 + missed`.
#[derive(Debug, Clone, PartialEq)]
pub struct RollCaptured {
    pub session: u32,
    pub item: RollItem,
    pub context: RollContext,
}
impl Event for RollCaptured {}

/// Sent by `acquisition`; the handler stores the intent and returns at once.
#[derive(Debug, Clone, PartialEq)]
pub struct ApplySetup {
    pub setup_revision: u64,
    pub setup: AcquisitionSetup,
}
impl Request for ApplySetup {
    type Response = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetCapturing {
    pub on: bool,
}
impl Request for SetCapturing {
    type Response = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForceTrigger;
impl Request for ForceTrigger {
    type Response = ();
}

/// Input-referred zero offsets of one unit: per channel, indexed by range index; an empty vector means none.
#[derive(Debug, Clone, PartialEq)]
pub struct ZeroOffsets {
    pub unit_id: String,
    pub volts: [Vec<f64>; 4],
}

/// `calibration` to `acquisition`: stop capturing and refuse setup and run commands until released.
/// Refused with a recoverable error while the instrument cannot be held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoldInstrument {
    pub on: bool,
    pub reason: String,
}
impl Request for HoldInstrument {
    type Response = ();
}

/// Applies `setup`, drops `discard` frames, keeps the next `keep` and answers with `ProbeCaptured`; returns at once.
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureProbe {
    pub id: u64,
    pub setup: AcquisitionSetup,
    pub discard: u8,
    pub keep: u8,
}
impl Request for CaptureProbe {
    type Response = ();
}

/// Empty `frames` means the probe failed or timed out. The frames never reach the display.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCaptured {
    pub id: u64,
    pub frames: Vec<Arc<Frame>>,
}
impl Event for ProbeCaptured {}

/// Published by `calibration`; `None` means no offsets exist for the connected unit.
#[derive(Debug, Clone, PartialEq)]
pub struct ZeroOffsetsChanged {
    pub offsets: Option<ZeroOffsets>,
}
impl Event for ZeroOffsetsChanged {}
