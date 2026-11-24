use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sweepscope_kernel::contracts::{Availability, FormattedValue};
use sweepscope_kernel::{Event, Request};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SpectrumScale {
    Dbv,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SpectrumWindow {
    Rectangular,
    Hann,
    Hamming,
    Blackman,
}

/// A graticule label: `at` is the position as a fraction of the screen, from the left or the top.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Tick {
    pub at: f64,
    pub text: String,
}

/// Spectrum slice. `active` is the one presentation fact the backend owns: spectra are computed only while it is true.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SpectrumState {
    #[ts(type = "number")]
    pub revision: u64,
    pub active: bool,
    pub window: SpectrumWindow,
    pub scale: SpectrumScale,
    pub bin_spacing: FormattedValue,
    pub span: FormattedValue,
    pub sample_rate: FormattedValue,
    pub record_points: FormattedValue,
    pub top_label: String,
    pub ticks_x: Vec<Tick>,
    pub ticks_y: Vec<Tick>,
    pub availability: Availability,
}

/// Optional fields: a missing field leaves the setting alone.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SpectrumPatch {
    #[ts(optional)]
    pub active: Option<bool>,
    #[ts(optional)]
    pub window: Option<SpectrumWindow>,
    #[ts(optional)]
    pub scale: Option<SpectrumScale>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetSpectrum {
    pub patch: SpectrumPatch,
}
impl Request for SetSpectrum {
    type Response = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetSpectrum;
impl Request for GetSpectrum {
    type Response = SpectrumState;
}

/// Published when the slice changes; the shell forwards it as `spectrum_changed`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectrumChanged(pub SpectrumState);
impl Event for SpectrumChanged {}

/// One encoded spectrum frame (`SSS1`, ADR 0018); latest-wins subscribers only see the newest.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectrumFrameReady {
    pub bytes: Arc<Vec<u8>>,
}
impl Event for SpectrumFrameReady {}
