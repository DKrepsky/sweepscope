//! Test helpers shared by every crate's tests.
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::{Bus, Clock, Event, FileFilter, FilePicker, KernelError, Storage, Subscription};

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

    fn sleep(&self, duration: Duration) {
        self.advance(duration);
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

/// Picker that answers from queues: one entry per dialog, `None` is a cancel. An unscripted dialog is a test bug and fails.
#[derive(Default)]
pub struct ScriptedPicker {
    save: Mutex<VecDeque<Option<PathBuf>>>,
    open: Mutex<VecDeque<Option<PathBuf>>>,
    suggested: Mutex<Vec<String>>,
}

impl ScriptedPicker {
    pub fn new(save: Vec<Option<PathBuf>>, open: Vec<Option<PathBuf>>) -> Self {
        Self {
            save: Mutex::new(save.into()),
            open: Mutex::new(open.into()),
            suggested: Mutex::default(),
        }
    }

    /// Names offered to the save dialogs so far, oldest first.
    pub fn suggested_names(&self) -> Vec<String> {
        self.suggested.lock().unwrap().clone()
    }
}

impl FilePicker for ScriptedPicker {
    fn pick_save(&self, suggested_name: &str, _filter: &FileFilter) -> Result<Option<PathBuf>, KernelError> {
        self.suggested.lock().unwrap().push(suggested_name.to_owned());
        next_answer(&self.save, "save")
    }

    fn pick_open(&self, _filter: &FileFilter) -> Result<Option<PathBuf>, KernelError> {
        next_answer(&self.open, "open")
    }
}

fn next_answer(queue: &Mutex<VecDeque<Option<PathBuf>>>, kind: &str) -> Result<Option<PathBuf>, KernelError> {
    queue
        .lock()
        .unwrap()
        .pop_front()
        .ok_or_else(|| KernelError::fatal(format!("no scripted answer for a {kind} dialog")))
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

/// Polls `condition` every few milliseconds until it holds or `timeout` passes; for the few tests that run real threads.
pub fn eventually(timeout: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
