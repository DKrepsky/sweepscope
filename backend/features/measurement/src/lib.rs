//! Twelve automatic measurements per enabled channel, derived from every captured frame (ADR 0014).
#![forbid(unsafe_code)]

mod channel;
mod messages;
mod module;
mod slice;
mod texts;

pub use messages::{
    ChannelMeasurements, GetMeasurements, Invalid, InvalidReason, Measurement, MeasurementId, MeasurementsChanged,
    MeasurementsState, Warning, WarningSeverity,
};
pub use module::{MeasurementDeps, MeasurementModule};
