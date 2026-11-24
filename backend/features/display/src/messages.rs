use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sweepscope_kernel::{CommandDescriptor, Event, Request};
use ts_rs::TS;

/// One encoded wire frame (ADR 0011); latest-wins subscribers only see the newest.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayFrameReady {
    pub bytes: Arc<Vec<u8>>,
}
impl Event for DisplayFrameReady {}

/// How the trace is drawn between samples (ADR 0015). `Sinc` upsamples in the backend; `Dots` sends the samples only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum InterpolationMode {
    Sinc,
    Linear,
    Dots,
}

/// Display slice. `interpolation` is the effective mode; `automatic` is true until the user picks one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DisplayState {
    #[ts(type = "number")]
    pub revision: u64,
    pub interpolation: InterpolationMode,
    pub automatic: bool,
}

/// Published when the state changes; the shell forwards it as `display_changed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayChanged(pub DisplayState);
impl Event for DisplayChanged {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetDisplay;
impl Request for GetDisplay {
    type Response = DisplayState;
}

/// Sets the mode and turns automatic selection off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetInterpolation {
    pub mode: InterpolationMode,
}
impl Request for SetInterpolation {
    type Response = ();
}

pub const COMMANDS: &[CommandDescriptor] = &[CommandDescriptor {
    name: "set_interpolation",
    args: "{ mode: InterpolationMode }",
    result: "null",
    types: &["InterpolationMode"],
}];
