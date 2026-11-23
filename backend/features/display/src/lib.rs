//! Turns a captured frame into the wire frame of ADR 0011 (scaling to divisions, window crop, invert).
#![forbid(unsafe_code)]

mod convert;
mod messages;
mod module;
mod wire;

pub use messages::{
    DisplayChanged, DisplayFrameReady, DisplayState, GetDisplay, InterpolationMode, SetInterpolation, COMMANDS,
};
pub use module::{DisplayDeps, DisplayModule};
pub use wire::{encode, stamp_sent};
