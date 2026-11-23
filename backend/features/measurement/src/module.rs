use std::sync::Arc;

use sweepscope_kernel::{Bus, Clock, KernelError};

pub struct MeasurementDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
}

/// Stub: the consumer thread and the `GetMeasurements` handler arrive with work package WP6.
pub struct MeasurementModule;

impl MeasurementModule {
    pub fn new(deps: MeasurementDeps) -> Result<Self, KernelError> {
        let MeasurementDeps { bus: _, clock: _ } = deps;
        Ok(Self)
    }
}
