//! Domain state of the instrument: channels, time base, trigger, run intent, window geometry, availability
//! of controls and the status and facts slices (ADR 0010, ADR 0013).
#![forbid(unsafe_code)]

mod domain;
mod messages;
mod module;

pub use messages::{
    AcquisitionStatus, Availability, ChannelPatch, ChannelState, ControlsAvailability, FactsChanged, ForceTriggerNow,
    FormattedValue, GetInstrument, HorizontalState, InstrumentChanged, InstrumentFacts, InstrumentSnapshot,
    InstrumentState, Phase, ResetSetting, ScaleChoice, SetChannel, SetRunning, SetTrigger, SetTriggerLevel,
    SetTriggerPosition, SettingTarget, StateSnapshot, StatusChanged, StepSetting, TriggerPatch, TriggerState,
};
pub use module::{AcquisitionDeps, AcquisitionModule, COMMANDS};
