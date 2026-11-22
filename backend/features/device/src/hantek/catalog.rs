use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use hantek6xx4_lib::{DeviceEvent, DeviceInfo, DeviceMonitor};

use super::convert::device_error;
use super::device::HantekDevice;
use crate::{CatalogEvent, DeviceCatalog, DeviceError, DeviceId, ScopeDevice};

/// The attached scopes and the subscribers told about them; one lock keeps replay and live events in order.
#[derive(Default)]
struct Hub {
    inner: Mutex<HubState>,
}

#[derive(Default)]
struct HubState {
    attached: Vec<DeviceInfo>,
    subscribers: Vec<Sender<CatalogEvent>>,
}

impl Hub {
    fn publish(&self, event: DeviceEvent) {
        let mut state = self.state();
        let catalog_event = match event {
            DeviceEvent::Connected(info) => {
                if !state.attached.contains(&info) {
                    state.attached.push(info);
                }
                CatalogEvent::Attached(id_of(info))
            }
            DeviceEvent::Disconnected(info) => {
                state.attached.retain(|attached| *attached != info);
                CatalogEvent::Detached(id_of(info))
            }
        };
        state
            .subscribers
            .retain(|subscriber| subscriber.send(catalog_event.clone()).is_ok());
    }

    fn subscribe(&self) -> Receiver<CatalogEvent> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut state = self.state();
        for info in &state.attached {
            let _ = sender.send(CatalogEvent::Attached(id_of(*info)));
        }
        state.subscribers.push(sender);
        receiver
    }

    fn find(&self, id: &DeviceId) -> Option<DeviceInfo> {
        self.state().attached.iter().copied().find(|info| id_of(*info) == *id)
    }

    fn state(&self) -> MutexGuard<'_, HubState> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn id_of(info: DeviceInfo) -> DeviceId {
    DeviceId(format!("hantek-{}-{}", info.bus, info.address))
}

/// Scopes on the USB bus, announced by the driver's settled hotplug monitor. A scope is only openable after it was announced.
pub struct HantekCatalog {
    hub: Arc<Hub>,
    monitor: Option<DeviceMonitor>,
    forwarder: Option<JoinHandle<()>>,
}

impl HantekCatalog {
    pub fn start() -> Result<Self, DeviceError> {
        let (monitor, events) = DeviceMonitor::channel().map_err(device_error)?;
        let hub = Arc::new(Hub::default());
        let forwarder = std::thread::Builder::new()
            .name("hantek-catalog".to_owned())
            .spawn({
                let hub = Arc::clone(&hub);
                move || events.iter().for_each(|event| hub.publish(event))
            })
            .map_err(|error| DeviceError::Io(error.to_string()))?;
        Ok(Self {
            hub,
            monitor: Some(monitor),
            forwarder: Some(forwarder),
        })
    }
}

impl DeviceCatalog for HantekCatalog {
    fn events(&self) -> Receiver<CatalogEvent> {
        self.hub.subscribe()
    }

    fn open(&self, id: &DeviceId) -> Result<Box<dyn ScopeDevice>, DeviceError> {
        let info = self.hub.find(id).ok_or(DeviceError::NotFound)?;
        let transport = hantek6xx4_lib::UsbTransport::open_device(info).map_err(device_error)?;
        Ok(Box::new(HantekDevice::new(transport)?))
    }
}

impl Drop for HantekCatalog {
    fn drop(&mut self) {
        // Dropping the monitor stops its thread, which closes the event channel and ends the forwarder.
        drop(self.monitor.take());
        if let Some(forwarder) = self.forwarder.take() {
            let _ = forwarder.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(address: u8) -> DeviceInfo {
        DeviceInfo {
            bus: 1,
            address,
            vendor_id: 0x04b5,
            product_id: 0x6cde,
        }
    }

    #[test]
    fn given_connected_and_disconnected_events_when_published_then_subscribers_see_them_in_order() {
        let hub = Hub::default();
        let events = hub.subscribe();

        hub.publish(DeviceEvent::Connected(info(5)));
        hub.publish(DeviceEvent::Disconnected(info(5)));

        assert_eq!(events.try_recv(), Ok(CatalogEvent::Attached(id_of(info(5)))));
        assert_eq!(events.try_recv(), Ok(CatalogEvent::Detached(id_of(info(5)))));
        assert!(events.try_recv().is_err());
    }

    #[test]
    fn given_an_attached_scope_when_a_late_subscriber_arrives_then_the_attached_set_is_replayed() {
        let hub = Hub::default();
        hub.publish(DeviceEvent::Connected(info(5)));
        hub.publish(DeviceEvent::Connected(info(6)));
        hub.publish(DeviceEvent::Disconnected(info(5)));

        let late = hub.subscribe();

        assert_eq!(late.try_recv(), Ok(CatalogEvent::Attached(id_of(info(6)))));
        assert!(late.try_recv().is_err());
    }

    #[test]
    fn given_two_subscribers_when_an_event_is_published_then_both_receive_it() {
        let hub = Hub::default();
        let (first, second) = (hub.subscribe(), hub.subscribe());

        hub.publish(DeviceEvent::Connected(info(5)));

        assert!(first.try_recv().is_ok());
        assert!(second.try_recv().is_ok());
    }

    #[test]
    fn given_a_dropped_subscriber_when_an_event_is_published_then_the_others_still_receive_it() {
        let hub = Hub::default();
        drop(hub.subscribe());
        let alive = hub.subscribe();

        hub.publish(DeviceEvent::Connected(info(5)));

        assert!(alive.try_recv().is_ok());
    }

    #[test]
    fn given_an_attached_scope_when_found_by_id_then_it_is_returned_and_a_detached_one_is_not() {
        let hub = Hub::default();
        hub.publish(DeviceEvent::Connected(info(5)));

        assert_eq!(hub.find(&id_of(info(5))), Some(info(5)));
        hub.publish(DeviceEvent::Disconnected(info(5)));
        assert_eq!(hub.find(&id_of(info(5))), None);
    }

    #[test]
    fn given_an_id_that_was_never_attached_when_opened_then_not_found_without_touching_usb() {
        let catalog = HantekCatalog {
            hub: Arc::new(Hub::default()),
            monitor: None,
            forwarder: None,
        };

        let result = catalog.open(&DeviceId("hantek-9-9".to_owned()));

        assert!(matches!(result, Err(DeviceError::NotFound)));
    }
}
