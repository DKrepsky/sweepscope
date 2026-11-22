//! Display module (WP7 fills this in): `subscribe_latest::<FrameCaptured>`, publishes `DisplayFrameReady`.

use sweepscope_kernel::{Bus, KernelError};

pub struct DisplayDeps {
    pub bus: Bus,
    pub unix_time_us: fn() -> f64,
}

pub struct DisplayModule;

impl DisplayModule {
    pub fn new(_deps: DisplayDeps) -> Result<Self, KernelError> {
        Err(KernelError::fatal("the display module is not implemented yet"))
    }
}
