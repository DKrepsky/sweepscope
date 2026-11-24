//! Guided zero calibration: the sweep over all ranges, the offsets and their file, the status slice (ADR 0021).
#![forbid(unsafe_code)]

pub mod domain;
mod messages;
mod module;

pub use module::{CalibrationDeps, CalibrationModule};

pub use messages::{
    ChannelTooltip, CloseZeroCalibration, ContinueZeroChannel, GetZeroCalibration, MeasureZeroChannel, OffsetRow,
    OpenZeroCalibration, StartZeroCalibration, TooltipText, UnitCalibration, ZeroCalibrationChanged, ZeroCalibrationState, ZeroSession, ZeroStep,
};
