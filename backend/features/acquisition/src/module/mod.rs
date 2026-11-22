//! Acquisition module: request handlers and one consumer thread around the pure domain (ADR 0010 items 3 and 5).
//!
//! Create it before the device module: it learns the connection from `ConnectionChanged` events and cannot ask for it.

mod commands;
mod core;
mod effects;
mod handlers;
mod shared;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::{CaptureCompleted, ConnectionChanged};
use sweepscope_kernel::{Bus, Clock, KernelError, Subscription};

pub use self::commands::COMMANDS;
use self::shared::Shared;

/// How long the consumer thread waits for a capture before it checks for connection changes, facts and shutdown.
const TICK: Duration = Duration::from_millis(20);

pub struct AcquisitionDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
}

pub struct AcquisitionModule {
    stop: Arc<AtomicBool>,
    consumer: Option<JoinHandle<()>>,
}

impl AcquisitionModule {
    pub fn new(deps: AcquisitionDeps) -> Result<Self, KernelError> {
        let shared = Arc::new(Shared::new(deps.bus.clone(), deps.clock));
        handlers::register(&deps.bus, &shared)?;
        let completed = deps.bus.subscribe::<CaptureCompleted>();
        let connections = deps.bus.subscribe::<ConnectionChanged>();
        let stop = Arc::new(AtomicBool::new(false));
        let consumer = std::thread::Builder::new()
            .name("acquisition".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || consume(&shared, &completed, &connections, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the acquisition thread: {e}")))?;
        Ok(Self {
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
    }
}

/// Connection changes go first, so a capture is judged against the newest connection state.
fn consume(
    shared: &Shared,
    completed: &Subscription<CaptureCompleted>,
    connections: &Subscription<ConnectionChanged>,
    stop: &AtomicBool,
) {
    while !stop.load(Ordering::Acquire) {
        for change in connections.drain() {
            shared.connection(change);
        }
        match completed.recv_timeout(TICK) {
            Some(outcome) => shared.capture(outcome),
            None => shared.tick(),
        }
    }
}
