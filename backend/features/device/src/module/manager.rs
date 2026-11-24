use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::{ConnectionIssue, ConnectionIssueKind, ConnectionState, DeviceSource};
use sweepscope_kernel::{Bus, Clock};

use super::connection::{ConnectionStore, Status};
use super::control::WorkerControl;
use super::issue::issue_for;
use super::session::{Session, SessionSlot};
use super::worker::{Worker, WorkerEnd};
use crate::{CatalogEvent, DeviceCatalog, DeviceError, DeviceId, ScopeDevice};

/// How often a forwarder looks at its stop flag while its catalog is quiet.
const FORWARD_POLL: Duration = Duration::from_millis(20);

pub enum Msg {
    Catalog(DeviceSource, CatalogEvent),
    Worker { session: u64, end: WorkerEnd },
    Retry,
    Stop,
}

/// Moves the events of one catalog into the manager's inbox until told to stop.
pub fn forward_catalog(
    source: DeviceSource,
    events: Receiver<CatalogEvent>,
    inbox: Sender<Msg>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match events.recv_timeout(FORWARD_POLL) {
                Ok(event) => {
                    if inbox.send(Msg::Catalog(source, event)).is_err() {
                        return;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    })
}

#[derive(Default)]
struct Attached {
    hardware: Option<DeviceId>,
    simulator: Option<DeviceId>,
}

impl Attached {
    fn get(&self, source: DeviceSource) -> &Option<DeviceId> {
        match source {
            DeviceSource::Hardware => &self.hardware,
            DeviceSource::Simulator => &self.simulator,
        }
    }

    fn set(&mut self, source: DeviceSource, id: Option<DeviceId>) {
        match source {
            DeviceSource::Hardware => self.hardware = id,
            DeviceSource::Simulator => self.simulator = id,
        }
    }
}

pub struct Catalogs {
    pub hardware: Option<Box<dyn DeviceCatalog>>,
    pub simulator: Option<Box<dyn DeviceCatalog>>,
}

impl Catalogs {
    fn get(&self, source: DeviceSource) -> Option<&dyn DeviceCatalog> {
        match source {
            DeviceSource::Hardware => self.hardware.as_deref(),
            DeviceSource::Simulator => self.simulator.as_deref(),
        }
    }
}

/// The connection state machine (ADR 0012 items 7 and 8). Runs on its own thread; opening a device blocks it, never a handler.
pub struct Manager {
    catalogs: Catalogs,
    attached: Attached,
    session: Option<Session>,
    /// An open or a capture failed: do not open again until a new `Attached` or a retry.
    errored: bool,
    next_session: u64,
    store: ConnectionStore,
    slot: SessionSlot,
    bus: Bus,
    clock: Arc<dyn Clock>,
    inbox: Sender<Msg>,
}

impl Manager {
    pub fn new(
        catalogs: Catalogs,
        store: ConnectionStore,
        slot: SessionSlot,
        bus: Bus,
        clock: Arc<dyn Clock>,
        inbox: Sender<Msg>,
    ) -> Self {
        Self {
            catalogs,
            attached: Attached::default(),
            session: None,
            errored: false,
            next_session: 1,
            store,
            slot,
            bus,
            clock,
            inbox,
        }
    }

    pub fn run(mut self, messages: Receiver<Msg>) {
        while let Ok(message) = messages.recv() {
            match message {
                Msg::Stop => break,
                other => self.handle(other),
            }
        }
        self.end_session();
    }

    fn handle(&mut self, message: Msg) {
        match message {
            Msg::Catalog(source, CatalogEvent::Attached(id)) => self.attach(source, id),
            Msg::Catalog(source, CatalogEvent::Detached(id)) => self.detach(source, &id),
            Msg::Worker { session, end } => self.worker_ended(session, end),
            Msg::Retry => self.retry(),
            Msg::Stop => {}
        }
    }

    fn attach(&mut self, source: DeviceSource, id: DeviceId) {
        self.attached.set(source, Some(id));
        if source == DeviceSource::Simulator {
            self.store.defer_simulator_active(true);
        }
        self.errored = false;
        self.reconcile();
        self.store.flush();
    }

    fn detach(&mut self, source: DeviceSource, id: &DeviceId) {
        if self.attached.get(source).as_ref() == Some(id) {
            self.attached.set(source, None);
        }
        if source == DeviceSource::Simulator {
            self.store.defer_simulator_active(false);
        }
        if self
            .session
            .as_ref()
            .is_some_and(|s| s.source == source && s.device_id == *id)
        {
            self.end_session();
            self.errored = false;
            self.store.set_status(Status::disconnected(), None);
        } else if self.session.is_none() && self.errored {
            self.errored = false;
            self.store.set_status(Status::disconnected(), None);
        }
        self.reconcile();
        self.store.flush();
    }

    fn worker_ended(&mut self, session: u64, end: WorkerEnd) {
        let Some(active) = self.session.as_ref().filter(|s| s.id == session) else {
            return;
        };
        let source = active.source;
        match end {
            WorkerEnd::Disconnected => {
                self.attached.set(source, None);
                self.end_session();
                self.store.set_status(Status::disconnected(), None);
                self.reconcile();
            }
            WorkerEnd::Failed(error) => {
                self.errored = true;
                self.store
                    .set_status(Status::error(source, issue_or_default(&error)), None);
            }
        }
    }

    fn retry(&mut self) {
        if self.store.state() == ConnectionState::Ready {
            return;
        }
        self.end_session();
        self.errored = false;
        self.reconcile();
    }

    /// Brings the session in line with what is attached: hardware beats the simulator, one device at a time.
    fn reconcile(&mut self) {
        let displaced = self
            .session
            .as_ref()
            .is_some_and(|s| s.source == DeviceSource::Simulator && self.attached.hardware.is_some());
        if displaced {
            self.end_session();
            self.store.set_status(Status::disconnected(), None);
        }
        if self.session.is_some() || self.errored {
            return;
        }
        let target = match (&self.attached.hardware, &self.attached.simulator) {
            (Some(id), _) => Some((DeviceSource::Hardware, id.clone())),
            (None, Some(id)) => Some((DeviceSource::Simulator, id.clone())),
            (None, None) => None,
        };
        if let Some((source, id)) = target {
            self.open(source, id);
        }
    }

    fn open(&mut self, source: DeviceSource, id: DeviceId) {
        let Some(catalog) = self.catalogs.get(source) else {
            return;
        };
        self.store.set_status(Status::connecting(source), None);
        match catalog.open(&id) {
            Ok(device) => self.start_session(source, id, device),
            Err(DeviceError::NotFound | DeviceError::Disconnected) => {
                self.attached.set(source, None);
                self.store.set_status(Status::disconnected(), None);
                self.reconcile();
            }
            Err(error) => {
                self.errored = true;
                self.store
                    .set_status(Status::error(source, issue_or_default(&error)), None);
            }
        }
    }

    fn start_session(&mut self, source: DeviceSource, id: DeviceId, device: Box<dyn ScopeDevice>) {
        let identity = device.identity().clone();
        let capabilities = device.capabilities().clone();
        let session_id = self.next_session;
        self.next_session += 1;
        let inbox = self.inbox.clone();
        let worker = Worker {
            device,
            control: Arc::new(WorkerControl::new()),
            bus: self.bus.clone(),
            clock: Arc::clone(&self.clock),
            report: Box::new(move |end| {
                let _ = inbox.send(Msg::Worker {
                    session: session_id,
                    end,
                });
            }),
        };
        match Session::start(session_id, source, id, worker) {
            Ok(session) => {
                self.slot.set(Some(session.control()));
                self.session = Some(session);
                self.store
                    .set_status(Status::ready(source, identity), Some(capabilities));
            }
            Err(error) => {
                self.errored = true;
                let issue = issue_or_default(&DeviceError::Io(error.to_string()));
                self.store.set_status(Status::error(source, issue), None);
            }
        }
    }

    fn end_session(&mut self) {
        self.slot.set(None);
        if let Some(session) = self.session.take() {
            session.end();
        }
    }
}

fn issue_or_default(error: &DeviceError) -> ConnectionIssue {
    issue_for(error).unwrap_or_else(|| ConnectionIssue {
        kind: ConnectionIssueKind::Communication,
        message: error.to_string(),
        help: None,
    })
}
