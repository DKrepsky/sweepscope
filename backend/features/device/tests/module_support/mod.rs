#![allow(dead_code)]
//! Rig for the device module tests: simulator devices behind a catalog that can fail, block or count them.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sweepscope_device::{
    AcquisitionSetup, CatalogEvent, ChannelSetup, Coupling, DeviceCatalog, DeviceDeps, DeviceError, DeviceId,
    DeviceModule, GetConnection, ScopeDevice, SimulatedCatalog, SimulatorControl, SimulatorSource, Slope, Sweep,
    TriggerSetup,
};
use sweepscope_device::{Capabilities, DeviceIdentity, Frame};
use sweepscope_kernel::contracts::{CaptureCompleted, ConnectionChanged, ConnectionSnapshot, ConnectionState};
use sweepscope_kernel::testing::{eventually, EventRecorder};
use sweepscope_kernel::{Bus, Subscription, SystemClock};

pub const WAIT: Duration = Duration::from_secs(3);
pub const TB_200US: u8 = 15;
pub const RANGE_1V: u8 = 8;

/// Test hooks shared by a catalog and the devices it opened.
#[derive(Default)]
pub struct Hooks {
    pub open_error: Mutex<Option<DeviceError>>,
    pub capture_error: Mutex<Option<DeviceError>>,
    pub gate: Mutex<Option<Gate>>,
    pub live_devices: AtomicUsize,
    pub opened: AtomicUsize,
    pub configures: AtomicUsize,
}

/// Blocks `capture` until released; `entered` fires once the worker is inside the call.
pub struct Gate {
    pub entered: std::sync::mpsc::Sender<()>,
    pub release: Receiver<()>,
}

pub struct TrackedCatalog {
    inner: SimulatedCatalog,
    hooks: Arc<Hooks>,
}

impl DeviceCatalog for TrackedCatalog {
    fn events(&self) -> Receiver<CatalogEvent> {
        self.inner.events()
    }

    fn open(&self, id: &DeviceId) -> Result<Box<dyn ScopeDevice>, DeviceError> {
        if let Some(error) = self.hooks.open_error.lock().unwrap().clone() {
            return Err(error);
        }
        let device = self.inner.open(id)?;
        self.hooks.opened.fetch_add(1, Ordering::SeqCst);
        self.hooks.live_devices.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(TrackedDevice {
            inner: device,
            hooks: Arc::clone(&self.hooks),
        }))
    }
}

struct TrackedDevice {
    inner: Box<dyn ScopeDevice>,
    hooks: Arc<Hooks>,
}

impl Drop for TrackedDevice {
    fn drop(&mut self) {
        self.hooks.live_devices.fetch_sub(1, Ordering::SeqCst);
    }
}

impl ScopeDevice for TrackedDevice {
    fn identity(&self) -> &DeviceIdentity {
        self.inner.identity()
    }

    fn capabilities(&self) -> &Capabilities {
        self.inner.capabilities()
    }

    fn configure(&mut self, setup: &AcquisitionSetup) -> Result<(), DeviceError> {
        self.inner.configure(setup)?;
        self.hooks.configures.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError> {
        if let Some(gate) = self.hooks.gate.lock().unwrap().take() {
            let _ = gate.entered.send(());
            let _ = gate.release.recv();
        }
        if let Some(error) = self.hooks.capture_error.lock().unwrap().take() {
            return Err(error);
        }
        self.inner.capture(timeout)
    }

    fn force_capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError> {
        self.inner.force_capture(timeout)
    }

    fn start_roll(&mut self) -> Result<(), DeviceError> {
        self.inner.start_roll()
    }

    fn poll_roll(&mut self) -> Result<Frame, DeviceError> {
        self.inner.poll_roll()
    }

    fn stop(&mut self) -> Result<(), DeviceError> {
        self.inner.stop()
    }
}

/// One source (hardware or simulator) of the rig: its control, hooks and the catalog to hand to the module.
pub struct Source {
    pub control: SimulatorControl,
    pub hooks: Arc<Hooks>,
    catalog: Option<Box<dyn DeviceCatalog>>,
}

impl Source {
    pub fn plugged() -> Self {
        Self::new(true)
    }

    pub fn unplugged() -> Self {
        Self::new(false)
    }

    fn new(plugged: bool) -> Self {
        let control = SimulatorControl::new();
        if !plugged {
            control.unplug();
        }
        let hooks = Arc::new(Hooks::default());
        let inner = SimulatedCatalog::new(Arc::new(SystemClock), control.clone());
        let catalog = Box::new(TrackedCatalog {
            inner,
            hooks: Arc::clone(&hooks),
        });
        Self {
            control,
            hooks,
            catalog: Some(catalog),
        }
    }

    fn take_catalog(&mut self) -> Box<dyn DeviceCatalog> {
        self.catalog.take().expect("catalog is handed to the module once")
    }
}

pub struct Rig {
    pub bus: Bus,
    pub connection: EventRecorder<ConnectionChanged>,
    pub completed: Subscription<CaptureCompleted>,
    pub module: DeviceModule,
}

/// Subscribes before the module exists, so the first state is not missed.
pub fn rig(hardware: Option<&mut Source>, simulator: Option<&mut Source>) -> Rig {
    let bus = Bus::new();
    let connection = EventRecorder::new(&bus);
    let completed = bus.subscribe();
    let module = DeviceModule::new(DeviceDeps {
        bus: bus.clone(),
        clock: Arc::new(SystemClock),
        hardware: hardware.map(Source::take_catalog),
        simulator: simulator.map(|source| SimulatorSource {
            catalog: source.take_catalog(),
            control: source.control.clone(),
        }),
    })
    .unwrap();
    Rig {
        bus,
        connection,
        completed,
        module,
    }
}

impl Rig {
    pub fn snapshot(&self) -> ConnectionSnapshot {
        self.bus.request(GetConnection).unwrap()
    }

    pub fn wait_for_state(&self, state: ConnectionState) {
        assert!(
            eventually(WAIT, || self.snapshot().state == state),
            "state stayed {:?}, wanted {state:?}",
            self.snapshot().state
        );
    }

    pub fn states(&self) -> Vec<ConnectionState> {
        self.connection.events().iter().map(|e| e.snapshot.state).collect()
    }
}

pub fn wait_until(condition: impl FnMut() -> bool) -> bool {
    eventually(WAIT, condition)
}

pub fn channel(enabled: bool) -> ChannelSetup {
    ChannelSetup {
        enabled,
        range_index: RANGE_1V,
        position_divs: 0.0,
        coupling: Coupling::Dc,
        bandwidth_limit: false,
    }
}

pub fn setup(sweep: Sweep, level_volts: f64) -> AcquisitionSetup {
    AcquisitionSetup {
        channels: [true, false, false, false].map(channel),
        time_base_index: TB_200US,
        trigger: TriggerSetup {
            source: 0,
            level_volts,
            slope: Slope::Rising,
            sweep,
            horizontal_position_percent: 50.0,
        },
    }
}
