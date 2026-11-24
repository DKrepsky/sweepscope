//! Spectrum of the captured channels and the math trace, computed only while the FFT view is open (ADR 0018).
#![forbid(unsafe_code)]

mod messages;
mod module;

pub use messages::{
    GetSpectrum, SetSpectrum, SpectrumChanged, SpectrumFrameReady, SpectrumPatch, SpectrumScale, SpectrumState,
    SpectrumWindow, Tick,
};
pub use module::{SpectrumDeps, SpectrumModule};
