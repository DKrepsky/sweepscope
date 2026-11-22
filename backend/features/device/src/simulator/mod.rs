//! A simulated 6254BC that runs on an injected clock: no hardware, no real waiting in tests.
//!
//! Limits, so nobody mistakes it for the instrument:
//! - The trigger sits exactly at the requested position; the real scope is off by about 6 samples.
//! - Dead time between frames (readback 15 ms, re-arm 5 ms) is an unverified assumption (`timing`).
//! - Signals are fixed 1 kHz waveforms plus small seeded noise; coupling and bandwidth limit have no effect.
//! - Only edge triggers exist, evaluated on the clean waveform of the source channel.
//! - Frames are exactly 4096 samples per channel, and the device never reports a calibration fault.
//! - A catalog subscriber is told about changes only; it is not sent an event for the device that is already attached.

pub mod edge;
pub mod signal;
pub mod timing;

mod catalog;
mod control;
mod device;
mod profile;

pub use catalog::SimulatedCatalog;
pub use control::{device_id, SimulatorControl};
pub use device::SimulatedDevice;
