mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use common::*;
use sweepscope_device::{
    CatalogEvent, DeviceCatalog, DeviceError, DeviceId, ScopeDevice, SimulatedDevice, SimulatorControl, Sweep,
};
use sweepscope_kernel::SystemClock;

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
fn given_a_simulator_unplugged_and_plugged_in_when_a_device_call_is_made_then_it_works_again() {
    let mut rig = unplugged_rig();

    rig.control.plug_in();

    assert!(rig.device.configure(&setup([true; 4], TB_200US)).is_ok());
}

#[test]
fn given_a_catalog_when_unplugged_and_plugged_in_then_detached_and_attached_are_emitted_and_open_follows_the_state() {
    let rig = rig();
    let catalog = catalog(&rig.clock, &rig.control);
    let events = catalog.events();
    let id = sweepscope_device::simulator::device_id();

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

#[test]
fn given_a_capture_waiting_for_a_trigger_when_unplugged_from_another_thread_then_it_returns_disconnected_within_a_poll()
{
    let control = SimulatorControl::new();
    let mut device = SimulatedDevice::new(Arc::new(SystemClock), control.clone());
    let mut config = setup([true; 4], TB_200US);
    config.trigger.sweep = Sweep::Normal;
    config.trigger.level_volts = 3.0;
    device.configure(&config).unwrap();
    let unplugger = {
        let control = control.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            control.unplug();
        })
    };
    let started = Instant::now();

    let result = device.capture(Duration::from_secs(30));

    unplugger.join().unwrap();
    assert_eq!(result.err(), Some(DeviceError::Disconnected));
    assert!(
        started.elapsed() < Duration::from_millis(50 + 20 + 200),
        "{:?}",
        started.elapsed()
    );
}
