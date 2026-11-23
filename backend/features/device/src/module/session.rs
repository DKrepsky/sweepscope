use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::DeviceSource;

use super::control::WorkerControl;
use super::worker::Worker;
use crate::DeviceId;

/// How long ending a session waits for its worker: a capture in flight cannot be cancelled.
pub const STOP_DEADLINE: Duration = Duration::from_millis(250);

struct DoneGuard(Sender<()>);

impl Drop for DoneGuard {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

/// One opened device and the thread serving it.
pub struct Session {
    pub id: u64,
    pub source: DeviceSource,
    pub device_id: DeviceId,
    control: Arc<WorkerControl>,
    done: Receiver<()>,
    handle: JoinHandle<()>,
}

impl Session {
    pub fn start(id: u64, source: DeviceSource, device_id: DeviceId, worker: Worker) -> std::io::Result<Self> {
        let control = Arc::clone(&worker.control);
        let (finished, done) = channel();
        let handle = std::thread::Builder::new()
            .name("device-worker".to_owned())
            .spawn(move || {
                let _done = DoneGuard(finished);
                worker.run();
            })?;
        Ok(Self {
            id,
            source,
            device_id,
            control,
            done,
            handle,
        })
    }

    pub fn control(&self) -> Arc<WorkerControl> {
        Arc::clone(&self.control)
    }

    /// Tells the worker to stop and joins it; past the deadline the thread is left to finish on its own.
    pub fn end(self) {
        self.control.stop();
        if self.done.recv_timeout(STOP_DEADLINE).is_ok() {
            let _ = self.handle.join();
        }
    }
}

/// The control of the current session, shared with the request handlers (which only store intent).
#[derive(Clone, Default)]
pub struct SessionSlot {
    current: Arc<Mutex<Option<Arc<WorkerControl>>>>,
}

impl SessionSlot {
    pub fn set(&self, control: Option<Arc<WorkerControl>>) {
        *self.lock() = control;
    }

    pub fn current(&self) -> Option<Arc<WorkerControl>> {
        self.lock().clone()
    }

    fn lock(&self) -> MutexGuard<'_, Option<Arc<WorkerControl>>> {
        self.current.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
