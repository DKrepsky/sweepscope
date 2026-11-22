//! Acquisition module (WP6 fills this in): handlers, consumer thread and event publishing.

use std::sync::Arc;

use sweepscope_kernel::{Bus, Clock, CommandDescriptor, KernelError};

pub struct AcquisitionDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
}

pub struct AcquisitionModule;

impl AcquisitionModule {
    pub fn new(_deps: AcquisitionDeps) -> Result<Self, KernelError> {
        Err(KernelError::fatal("the acquisition module is not implemented yet"))
    }
}

pub const COMMANDS: &[CommandDescriptor] = &[];
