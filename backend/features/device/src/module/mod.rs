//! Device module (WP3 fills this in): connection manager and one worker thread per opened device.

use std::sync::Arc;

use sweepscope_kernel::{Bus, Clock, CommandDescriptor, KernelError};

use crate::{DeviceCatalog, SimulatorControl};

pub struct SimulatorSource {
    pub catalog: Box<dyn DeviceCatalog>,
    pub control: SimulatorControl,
}

pub struct DeviceDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    pub hardware: Option<Box<dyn DeviceCatalog>>,
    pub simulator: Option<SimulatorSource>,
}

pub struct DeviceModule;

impl DeviceModule {
    pub fn new(_deps: DeviceDeps) -> Result<Self, KernelError> {
        Err(KernelError::fatal("the device module is not implemented yet"))
    }
}

pub const COMMANDS: &[CommandDescriptor] = &[];
