//! A simulated 6254BC that runs on an injected clock: no hardware, no real waiting in tests.
//!
//! Limits, so nobody mistakes it for the instrument:
//! - The trigger sits exactly at the requested position; the real scope is off by about 6 samples.
//! - Dead time between frames (readback 15 ms, re-arm 5 ms) is an unverified assumption (`timing`).
//! - Signals are waveforms with a set frequency, amplitude and offset plus small seeded noise; coupling and bandwidth limit have no effect.
//! - Every signal gets a per-channel, per-range zero error (defaults from protocol A.3); `Shorted` has no waveform and shows Gaussian noise per range instead.
//! - Edge and pulse triggers exist, evaluated on the clean waveform of the source channel; a pulse is
//!   aligned to its leading edge and a missing pulse behaves like a missing edge (`Normal` times out).
//! - Frames are exactly 4096 samples per channel, and the device never reports a calibration fault.
//! - A new catalog subscriber is first sent `Attached` for the device if it is plugged in, then every change.
//! - After `unplug()` a device handle stays `Disconnected` even after `plug_in()`: open it again, as after re-enumeration.
//! - `Auto` sweep ignores the capture timeout (the cycle always completes); only `Normal` sweep returns `Timeout`.

mod catalog;
mod control;
mod device;
mod edge;
mod profile;
mod pulse;
mod signal;
mod timing;

pub use catalog::SimulatedCatalog;
pub use control::{device_id, SimulatorControl};
pub use device::SimulatedDevice;
pub use signal::{SignalKind, SignalSpec};
