//! Turns a captured frame into the wire frame of ADR 0011 (scaling to divisions, window crop, invert).
#![forbid(unsafe_code)]

mod convert;
mod messages;
mod mode;
mod module;
mod wire;

pub use messages::{
    DisplayChanged, DisplayFrameReady, DisplayRollItem, DisplayState, GetDisplay, InterpolationMode, Persistence,
    SetInterpolation, SetPersistence, COMMANDS,
};
pub use module::{DisplayDeps, DisplayModule};
pub use wire::{encode, encode_in_mode, stamp_sent};
