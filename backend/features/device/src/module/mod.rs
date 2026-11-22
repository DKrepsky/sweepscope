//! Device module: connection state machine and one worker thread per opened device (ADR 0012 items 7 to 9).
//!
//! Handlers only store intent (`WorkerControl`) or send a message to the manager thread, so they never block on hardware.

mod connection;
mod control;
mod issue;
mod manager;
mod session;
mod timeout;
mod worker;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use sweepscope_kernel::contracts::{ApplySetup, DeviceSource, ForceTrigger, SetCapturing};
use sweepscope_kernel::{Bus, Clock, CommandDescriptor, KernelError};

use self::connection::ConnectionStore;
use self::manager::{forward_catalog, Catalogs, Manager, Msg};
use self::session::SessionSlot;
use crate::{DeviceCatalog, GetConnection, RetryConnection, SimulatorControl, UseSimulator};

pub struct SimulatorSource {
    pub catalog: Box<dyn DeviceCatalog>,
    pub control: SimulatorControl,
}

pub struct DeviceDeps {
    pub bus: Bus,
    /// Not used by the module itself: devices and catalogs carry their own clock.
    pub clock: Arc<dyn Clock>,
    pub hardware: Option<Box<dyn DeviceCatalog>>,
    pub simulator: Option<SimulatorSource>,
}

/// Frontend-callable commands of this feature, consumed by the shell to generate `commands.ts`.
pub const COMMANDS: &[CommandDescriptor] = &[
    CommandDescriptor {
        name: "use_simulator",
        args: "{ enabled: boolean }",
        result: "null",
        types: &[],
    },
    CommandDescriptor {
        name: "retry_connection",
        args: "{}",
        result: "null",
        types: &[],
    },
];

pub struct DeviceModule {
    inbox: Sender<Msg>,
    stop_forwarding: Arc<AtomicBool>,
    forwarders: Vec<JoinHandle<()>>,
    manager: Option<JoinHandle<()>>,
}

impl DeviceModule {
    /// Registers the handlers and starts the manager thread; the first `ConnectionChanged` (`Disconnected`) is published before this returns.
    pub fn new(deps: DeviceDeps) -> Result<Self, KernelError> {
        let DeviceDeps {
            bus,
            clock: _,
            hardware,
            simulator,
        } = deps;
        let store = ConnectionStore::new(bus.clone());
        let slot = SessionSlot::default();
        let (inbox, messages) = channel();
        let (simulator_catalog, simulator_control) = match simulator {
            Some(source) => (Some(source.catalog), Some(source.control)),
            None => (None, None),
        };

        register_handlers(&bus, &store, &slot, &inbox, simulator_control)?;

        let stop_forwarding = Arc::new(AtomicBool::new(false));
        let forwarders = [
            (DeviceSource::Hardware, hardware.as_deref()),
            (DeviceSource::Simulator, simulator_catalog.as_deref()),
        ]
        .into_iter()
        .filter_map(|(source, catalog)| Some((source, catalog?.events())))
        .map(|(source, events)| forward_catalog(source, events, inbox.clone(), Arc::clone(&stop_forwarding)))
        .collect();

        let catalogs = Catalogs {
            hardware,
            simulator: simulator_catalog,
        };
        let manager = Manager::new(catalogs, store, slot, bus, inbox.clone());
        let manager = std::thread::Builder::new()
            .name("device-manager".to_owned())
            .spawn(move || manager.run(messages))
            .map_err(|error| KernelError::fatal(format!("cannot start the device manager: {error}")))?;

        Ok(Self {
            inbox,
            stop_forwarding,
            forwarders,
            manager: Some(manager),
        })
    }
}

impl Drop for DeviceModule {
    fn drop(&mut self) {
        self.stop_forwarding.store(true, Ordering::Relaxed);
        let _ = self.inbox.send(Msg::Stop);
        for forwarder in self.forwarders.drain(..) {
            let _ = forwarder.join();
        }
        if let Some(manager) = self.manager.take() {
            let _ = manager.join();
        }
    }
}

fn register_handlers(
    bus: &Bus,
    store: &ConnectionStore,
    slot: &SessionSlot,
    inbox: &Sender<Msg>,
    simulator: Option<SimulatorControl>,
) -> Result<(), KernelError> {
    let reader = store.clone();
    bus.handle(move |_: GetConnection| Ok(reader.snapshot()))?;
    register_worker_requests(bus, slot)?;

    let retries = Mutex::new(inbox.clone());
    bus.handle(move |_: RetryConnection| {
        let _ = retries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .send(Msg::Retry);
        Ok(())
    })?;

    bus.handle(move |request: UseSimulator| {
        let control = simulator
            .as_ref()
            .ok_or_else(|| KernelError::recoverable("this build has no simulator"))?;
        if request.enabled {
            control.plug_in();
        } else {
            control.unplug();
        }
        Ok(())
    })
}

/// Requests from `acquisition`: each stores intent for the current worker and returns at once.
fn register_worker_requests(bus: &Bus, slot: &SessionSlot) -> Result<(), KernelError> {
    let current = slot.clone();
    bus.handle(move |request: ApplySetup| {
        current
            .current()
            .ok_or_else(no_session)?
            .apply_setup(request.setup_revision, request.setup);
        Ok(())
    })?;

    let current = slot.clone();
    bus.handle(move |request: SetCapturing| {
        match current.current() {
            Some(control) => control.set_capturing(request.on),
            None if request.on => return Err(no_session()),
            None => {}
        }
        Ok(())
    })?;

    let current = slot.clone();
    bus.handle(move |_: ForceTrigger| {
        current.current().ok_or_else(no_session)?.force();
        Ok(())
    })
}

fn no_session() -> KernelError {
    KernelError::recoverable("no oscilloscope is connected")
}
