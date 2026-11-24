//! Inert until its own package fills it in: it registers nothing and publishes nothing.
use sweepscope_kernel::{Bus, KernelError};

pub struct SpectrumDeps {
    pub bus: Bus,
}

pub struct SpectrumModule;

impl SpectrumModule {
    pub fn new(_deps: SpectrumDeps) -> Result<Self, KernelError> {
        Ok(Self)
    }
}
