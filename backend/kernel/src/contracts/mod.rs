//! Device-neutral data and the cross-module messages (ADR 0010). Adding a message needs a line in a story spec and a review.

mod capabilities;
mod connection;
mod frame;
mod identity;
mod messages;
mod setup;
mod shared;

pub use capabilities::{Capabilities, TimeBaseStep, VerticalRange};
pub use connection::{
    ConnectionIssue, ConnectionIssueKind, ConnectionSnapshot, ConnectionState, DeviceSource, PermissionHelp,
};
pub use frame::{ChannelData, Frame, TriggerOutcome};
pub use identity::{CalibrationStatus, DeviceIdentity};
pub use messages::{
    ApplySetup, CaptureCompleted, CaptureOutcome, CaptureProbe, ConnectionChanged, DerivedTrace, DerivedUnit,
    ForceTrigger, FrameCaptured, FrameContext, HoldInstrument, ProbeCaptured, RollCaptured, RollCloseReason,
    RollContext, RollItem, RollPolled, RollPolledItem, SetCapturing, ZeroOffsets, ZeroOffsetsChanged,
};
pub use setup::{AcquisitionSetup, ChannelSetup, Coupling, PulseCondition, Slope, Sweep, TriggerKind, TriggerSetup};
pub use shared::{Availability, FormattedValue, SavedFile};
