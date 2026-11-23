use std::sync::Arc;

use sweepscope_kernel::{Bus, FilePicker, KernelError};

pub struct ExportDeps {
    pub bus: Bus,
    pub picker: Arc<dyn FilePicker>,
    pub unix_time_us: fn() -> f64,
}

/// Stub: the frame subscription and the request handlers arrive with work package WP7.
pub struct ExportModule;

impl ExportModule {
    pub fn new(deps: ExportDeps) -> Result<Self, KernelError> {
        let ExportDeps {
            bus: _,
            picker: _,
            unix_time_us: _,
        } = deps;
        Ok(Self)
    }
}
