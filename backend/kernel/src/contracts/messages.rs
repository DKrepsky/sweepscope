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
    /// False when the range is outside the verified 50 mV to 1 V per division.
    pub range_verified: [bool; 4],
    pub seconds_per_div: f64,
}

/// Published by `acquisition` for every accepted frame; latest-wins subscribers only see the newest.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameCaptured {
    pub frame: Arc<Frame>,
    pub context: FrameContext,
}
impl Event for FrameCaptured {}

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
