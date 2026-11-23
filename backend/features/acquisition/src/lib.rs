//! Domain state of the instrument: channels, time base, trigger, run intent, window geometry, availability
//! of controls and the status and facts slices (ADR 0010, ADR 0013).
#![forbid(unsafe_code)]

mod domain;
mod messages;
mod module;

pub use domain::{
    geometry, next_status, record_percent, trigger_index_in_window, CaptureMeter, Change, Defaults, Geometry,
    Instrument, Restore, SetupFile, SetupReport, StatusEvent,
};
pub use messages::{
    AcquisitionStatus, Availability, ChannelPatch, ChannelState, ControlsAvailability, FactsChanged, ForceTriggerNow,
    GetInstrument, HorizontalState, InstrumentChanged, InstrumentFacts, InstrumentSnapshot, InstrumentState, LoadSetup,
    Phase, ResetSetting, SaveSetup, ScaleChoice, SetChannel, SetRunning, SetTrigger, SetTriggerLevel,
    SetTriggerPosition, SettingTarget, SetupLoaded, SetupNotice, StateSnapshot, StatusChanged, StepSetting,
    TriggerPatch, TriggerState,
};
pub use module::{AcquisitionDeps, AcquisitionModule, COMMANDS};
pub use sweepscope_kernel::contracts::FormattedValue;
