//! Device-neutral data and the cross-module messages (ADR 0010). Adding a message needs a line in a story spec and a review.

mod capabilities;
mod connection;
mod frame;
mod identity;
mod messages;
mod setup;

pub use capabilities::{Capabilities, TimeBaseStep, VerticalRange};
pub use connection::{
    ConnectionIssue, ConnectionIssueKind, ConnectionSnapshot, ConnectionState, DeviceSource, PermissionHelp,
};
pub use frame::{ChannelData, Frame, TriggerOutcome};
pub use identity::{CalibrationStatus, DeviceIdentity};
pub use messages::{
    ApplySetup, CaptureCompleted, CaptureOutcome, ConnectionChanged, ForceTrigger, FrameCaptured, FrameContext,
    SetCapturing,
};
pub use setup::{AcquisitionSetup, ChannelSetup, Coupling, Slope, Sweep, TriggerSetup};
