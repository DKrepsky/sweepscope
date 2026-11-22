mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::*;
use sweepscope_device::{
    CatalogEvent, DeviceCatalog, DeviceError, DeviceId, ScopeDevice, SimulatedDevice, SimulatorControl, Sweep,
};
use sweepscope_kernel::testing::FakeClock;
use sweepscope_kernel::Clock;

fn unplugged_rig() -> Rig {
    let mut rig = rig();
    configured(&mut rig, &setup([true; 4], TB_500MS));
    rig.device.start_roll().unwrap();
    rig.control.unplug();
    rig
}

#[test]
fn given_an_unplugged_simulator_when_configure_is_called_then_it_is_disconnected() {
    let mut rig = unplugged_rig();

    assert_eq!(
        rig.device.configure(&setup([true; 4], TB_200US)),
        Err(DeviceError::Disconnected)
    );
}

#[test]
fn given_an_unplugged_simulator_when_capture_is_called_then_it_is_disconnected() {
    let mut rig = unplugged_rig();

    assert_eq!(
        rig.device.capture(Duration::from_secs(1)).err(),
        Some(DeviceError::Disconnected)
    );
}

#[test]
fn given_an_unplugged_simulator_when_poll_roll_is_called_then_it_is_disconnected() {
    let mut rig = unplugged_rig();

    assert_eq!(rig.device.poll_roll().err(), Some(DeviceError::Disconnected));
}

#[test]
fn given_an_unplugged_simulator_when_start_roll_or_stop_is_called_then_it_is_disconnected() {
    let mut rig = unplugged_rig();

    assert_eq!(rig.device.start_roll(), Err(DeviceError::Disconnected));
    assert_eq!(rig.device.stop(), Err(DeviceError::Disconnected));
}

#[test]
fn given_a_catalog_when_unplugged_and_plugged_in_then_detached_and_attached_are_emitted_and_open_follows_the_state() {
    let rig = rig();
    let catalog = catalog(&rig.clock, &rig.control);
    let events = catalog.events();
    let id = sweepscope_device::device_id();
    assert_eq!(events.try_recv(), Ok(CatalogEvent::Attached(id.clone())));

    rig.control.unplug();
    let while_unplugged = catalog.open(&id).err();
    rig.control.plug_in();
    let after_plug_in = catalog.open(&id).is_ok();

    assert_eq!(events.try_recv(), Ok(CatalogEvent::Detached(id.clone())));
    assert_eq!(events.try_recv(), Ok(CatalogEvent::Attached(id)));
    assert_eq!(while_unplugged, Some(DeviceError::NotFound));
    assert!(after_plug_in);
}

#[test]
fn given_an_unknown_device_id_when_opened_then_not_found() {
    let rig = rig();

    let result = catalog(&rig.clock, &rig.control).open(&DeviceId("nope".into()));

    assert_eq!(result.err(), Some(DeviceError::NotFound));
}

/// Clock that unplugs the simulator during its Nth sleep, then counts any later sleeps.
struct UnplugDuringSleep {
    inner: FakeClock,
    control: SimulatorControl,
    unplug_on: usize,
    sleeps: AtomicUsize,
}

impl Clock for UnplugDuringSleep {
    fn now(&self) -> Instant {
        self.inner.now()
    }

    fn sleep(&self, duration: Duration) {
        self.inner.sleep(duration);
        if self.sleeps.fetch_add(1, Ordering::SeqCst) + 1 == self.unplug_on {
            self.control.unplug();
        }
    }
}

#[test]
fn given_a_capture_waiting_for_a_trigger_when_unplugged_during_a_wait_slice_then_it_returns_disconnected_without_sleeping_again(
) {
    let control = SimulatorControl::new();
    let clock = Arc::new(UnplugDuringSleep {
        inner: FakeClock::default(),
        control: control.clone(),
        unplug_on: 3,
        sleeps: AtomicUsize::new(0),
    });
    let mut device = SimulatedDevice::new(clock.clone(), control);
    let mut config = setup([true; 4], TB_200US);
    config.trigger.sweep = Sweep::Normal;
    config.trigger.level_volts = 3.0;
    device.configure(&config).unwrap();

    let result = device.capture(Duration::from_secs(30));

    assert_eq!(result.err(), Some(DeviceError::Disconnected));
    assert_eq!(clock.sleeps.load(Ordering::SeqCst), 3);
}

#[test]
fn given_an_opened_device_when_the_simulator_is_unplugged_and_plugged_in_then_the_old_handle_stays_disconnected() {
    let mut rig = rig();
    let catalog = catalog(&rig.clock, &rig.control);

    rig.control.unplug();
    rig.control.plug_in();

    assert_eq!(
        rig.device.configure(&setup([true; 4], TB_200US)),
        Err(DeviceError::Disconnected)
    );
    let mut reopened = catalog.open(&sweepscope_device::device_id()).unwrap();
    assert!(reopened.configure(&setup([true; 4], TB_200US)).is_ok());
}
