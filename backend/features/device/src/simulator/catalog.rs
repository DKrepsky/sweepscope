use std::sync::mpsc::Receiver;
use std::sync::Arc;

use sweepscope_kernel::Clock;

use super::control::{device_id, SimulatorControl};
use super::device::SimulatedDevice;
use crate::{CatalogEvent, DeviceCatalog, DeviceError, DeviceId, ScopeDevice};

/// Catalog with the one simulated instrument; `SimulatorControl` plugs it in and out.
pub struct SimulatedCatalog {
    clock: Arc<dyn Clock>,
    control: SimulatorControl,
}

impl SimulatedCatalog {
    pub fn new(clock: Arc<dyn Clock>, control: SimulatorControl) -> Self {
        Self { clock, control }
    }
}

impl DeviceCatalog for SimulatedCatalog {
    fn events(&self) -> Receiver<CatalogEvent> {
        self.control.subscribe()
    }

    fn open(&self, id: &DeviceId) -> Result<Box<dyn ScopeDevice>, DeviceError> {
        if *id != device_id() || !self.control.is_plugged() {
            return Err(DeviceError::NotFound);
        }
        Ok(Box::new(SimulatedDevice::new(self.clock.clone(), self.control.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sweepscope_kernel::testing::FakeClock;

    fn catalog() -> (SimulatedCatalog, SimulatorControl) {
        let control = SimulatorControl::new();
        (
            SimulatedCatalog::new(Arc::new(FakeClock::default()), control.clone()),
            control,
        )
    }

    #[test]
    fn given_a_plugged_simulator_when_its_id_is_opened_then_a_simulated_device_is_returned() {
        let (catalog, _) = catalog();

        let device = catalog.open(&device_id()).unwrap();

        assert!(device.identity().simulated);
    }

    #[test]
    fn given_an_unplugged_simulator_when_opened_then_not_found() {
        let (catalog, control) = catalog();
        control.unplug();

        assert_eq!(catalog.open(&device_id()).err(), Some(DeviceError::NotFound));
    }

    #[test]
    fn given_an_unknown_id_when_opened_then_not_found() {
        let (catalog, _) = catalog();

        assert_eq!(
            catalog.open(&DeviceId("other".into())).err(),
            Some(DeviceError::NotFound)
        );
    }

    #[test]
    fn given_a_catalog_when_the_simulator_is_unplugged_and_plugged_in_then_it_emits_detached_then_attached() {
        let (catalog, control) = catalog();
        let events = catalog.events();

        control.unplug();
        control.plug_in();

        assert_eq!(events.try_recv(), Ok(CatalogEvent::Detached(device_id())));
        assert_eq!(events.try_recv(), Ok(CatalogEvent::Attached(device_id())));
    }
}
