//! Acquisition module: request handlers and one consumer thread around the pure domain (ADR 0010 items 3 and 5).
//!
//! Create it before the device module: it learns the connection from `ConnectionChanged` events and cannot ask for it.

mod autoset;
mod commands;
mod core;
mod derive;
mod effects;
mod handlers;
mod hold;
mod last_setup;
mod roll;
mod setup_files;
mod shared;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::{CaptureCompleted, ConnectionChanged, RollPolled, ZeroOffsetsChanged};
use sweepscope_kernel::{Bus, Clock, FilePicker, KernelError, Storage, Subscription};

pub use self::commands::COMMANDS;
use self::setup_files::SetupFiles;
use self::shared::Shared;

/// How long the consumer thread waits for a capture before it checks for connection changes, facts and shutdown.
const TICK: Duration = Duration::from_millis(20);

pub struct AcquisitionDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    /// Holds `last-setup.json`.
    pub storage: Arc<dyn Storage>,
    pub picker: Arc<dyn FilePicker>,
}

pub struct AcquisitionModule {
    shared: Arc<Shared>,
    stop: Arc<AtomicBool>,
    consumer: Option<JoinHandle<()>>,
}

impl AcquisitionModule {
    pub fn new(deps: AcquisitionDeps) -> Result<Self, KernelError> {
        let shared = Arc::new(Shared::new(deps.bus.clone(), deps.clock, deps.storage));
        handlers::register(&deps.bus, &shared, SetupFiles::new(deps.picker))?;
        let completed = deps.bus.subscribe::<CaptureCompleted>();
        let connections = deps.bus.subscribe::<ConnectionChanged>();
        let polled = deps.bus.subscribe::<RollPolled>();
        let offsets = deps.bus.subscribe::<ZeroOffsetsChanged>();
        let stop = Arc::new(AtomicBool::new(false));
        let consumer = std::thread::Builder::new()
            .name("acquisition".into())
            .spawn({
                let (shared, stop) = (Arc::clone(&shared), Arc::clone(&stop));
                move || {
                    consume(
                        &shared,
                        &Inputs {
                            completed,
                            connections,
                            polled,
                            offsets,
                        },
                        &stop,
                    )
                }
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the acquisition thread: {e}")))?;
        Ok(Self {
            shared,
            stop,
            consumer: Some(consumer),
        })
    }
}

impl Drop for AcquisitionModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(consumer) = self.consumer.take() {
            let _ = consumer.join();
        }
        self.shared.flush();
    }
}

struct Inputs {
    completed: Subscription<CaptureCompleted>,
    connections: Subscription<ConnectionChanged>,
    polled: Subscription<RollPolled>,
    offsets: Subscription<ZeroOffsetsChanged>,
}

/// Connection changes and zero offsets go first, so a capture is judged against the newest state. They are drained
/// again after the wait, because they can arrive while it blocks and the capture that wakes it may depend on them.
fn consume(shared: &Shared, inputs: &Inputs, stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        apply_pending(shared, inputs);
        for polled in inputs.polled.drain() {
            shared.roll_polled(polled);
        }
        match inputs.completed.recv_timeout(TICK) {
            Some(outcome) => {
                apply_pending(shared, inputs);
                shared.capture(outcome);
            }
            None => shared.tick(),
        }
    }
}

fn apply_pending(shared: &Shared, inputs: &Inputs) {
    for change in inputs.connections.drain() {
        shared.connection(change);
    }
    for change in inputs.offsets.drain() {
        shared.zero_offsets(change);
    }
}
