//! Spectrum of the captured channels and the math trace, computed only while the FFT view is open (ADR 0018).
#![forbid(unsafe_code)]

mod analysis;
mod axis;
mod messages;
mod module;
mod slice;
mod state;
mod wire;

pub use messages::{
    GetSpectrum, SetSpectrum, SpectrumChanged, SpectrumFrameReady, SpectrumPatch, SpectrumScale, SpectrumState,
    SpectrumWindow, Tick,
};
pub use module::{SpectrumDeps, SpectrumModule};
pub use wire::{encode, stamp_sent, ChannelSpectrum, FrameHeader, MATH_SLOT};
