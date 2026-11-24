//! The CLI's own small composition (ADR 0022 item 2): acquisition and device, nothing that draws or analyses.
use std::sync::Arc;

use sweepscope_acquisition::{AcquisitionDeps, AcquisitionModule};
use sweepscope_device::{DeviceCatalog, DeviceDeps, DeviceModule, SimulatorSource};
use sweepscope_kernel::{Bus, Clock, FilePicker, KernelError, Storage};

/// Everything the run needs from the outside, so tests can swap the world for the simulator.
pub struct Environment {
    pub storage: Arc<dyn Storage>,
    pub clock: Arc<dyn Clock>,
    pub hardware: Option<Box<dyn DeviceCatalog>>,
    pub simulator: Option<SimulatorSource>,
    pub unix_time_us: fn() -> f64,
}

/// Modules stay alive here. Fields drop in declaration order: the consumer stops before the device it reads from.
pub struct Backend {
    _acquisition: AcquisitionModule,
    _device: DeviceModule,
}

/// The caller owns the bus, so it can subscribe before the device starts publishing.
pub fn compose(bus: &Bus, environment: Environment, picker: Arc<dyn FilePicker>) -> Result<Backend, KernelError> {
    let acquisition = AcquisitionModule::new(AcquisitionDeps {
        bus: bus.clone(),
        clock: Arc::clone(&environment.clock),
        storage: environment.storage,
        picker,
    })?;
    let device = DeviceModule::new(DeviceDeps {
        bus: bus.clone(),
        clock: environment.clock,
        hardware: environment.hardware,
        simulator: environment.simulator,
    })?;
    Ok(Backend {
        _acquisition: acquisition,
        _device: device,
    })
}
