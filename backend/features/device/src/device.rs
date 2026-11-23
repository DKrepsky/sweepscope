use std::time::Duration;

use crate::{AcquisitionSetup, Capabilities, DeviceError, DeviceIdentity, Frame};

/// One opened instrument. Calls block; the owner runs them on a dedicated thread.
pub trait ScopeDevice: Send {
    fn identity(&self) -> &DeviceIdentity;
    fn capabilities(&self) -> &Capabilities;
    fn configure(&mut self, setup: &AcquisitionSetup) -> Result<(), DeviceError>;
    /// Blocks until a frame is ready or `timeout` elapses (`Timeout`; in normal sweep that means "no trigger yet").
    fn capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError>;
    /// Like `capture`, but never waits for a trigger: returns a free-run frame (the trigger button of the UI).
    fn force_capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError>;
    fn start_roll(&mut self) -> Result<(), DeviceError>;
    /// Samples since the previous call, possibly none. `Overrun` once after data was probably lost; the next call
    /// continues at the newest data.
    fn poll_roll(&mut self) -> Result<Frame, DeviceError>;
    fn stop(&mut self) -> Result<(), DeviceError>;
}
