//! Test helpers shared by every crate's tests.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{Bus, Clock, Event, KernelError, Storage, Subscription};

/// Clock that only moves when told to.
#[derive(Clone)]
pub struct FakeClock {
    now: Arc<Mutex<Instant>>,
}

impl Default for FakeClock {
    fn default() -> Self {
        Self {
            now: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

impl FakeClock {
    pub fn advance(&self, by: Duration) {
        *self.now.lock().unwrap() += by;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Instant {
        *self.now.lock().unwrap()
    }
}

/// In-memory storage; clones share the same contents, so a second module instance can reopen it.
#[derive(Clone, Default)]
pub struct MemoryStorage {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl Storage for MemoryStorage {
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>, KernelError> {
        Ok(self.files.lock().unwrap().get(key).cloned())
    }

    fn write(&self, key: &str, bytes: &[u8]) -> Result<(), KernelError> {
        self.files.lock().unwrap().insert(key.to_owned(), bytes.to_vec());
        Ok(())
    }
}

/// Collects every event of one type published after it was created.
pub struct EventRecorder<E> {
    subscription: Subscription<E>,
    seen: Mutex<Vec<E>>,
}

impl<E: Event> EventRecorder<E> {
    pub fn new(bus: &Bus) -> Self {
        Self {
            subscription: bus.subscribe(),
            seen: Mutex::new(Vec::new()),
        }
    }

    pub fn events(&self) -> Vec<E> {
        let mut seen = self.seen.lock().unwrap();
        seen.extend(self.subscription.drain());
        seen.clone()
    }
}

/// Unique temp directory removed on drop.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = format!(
            "sweepscope-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
