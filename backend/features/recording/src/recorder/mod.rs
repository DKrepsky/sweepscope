//! The recording state machine. It owns the subscription to the roll items, so the bounded bus queue is the recorder queue:
//! when the writer is slow the oldest items are dropped and the next samples show the index jump as a gap record.
mod file;
mod readout;

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sweepscope_kernel::contracts::{RollCaptured, RollItem, SavedFile};
use sweepscope_kernel::{Bus, Clock, KernelError, Subscription};

use crate::files::RecordFiles;
use crate::format::EndReason;
use crate::messages::{RecordingChanged, RecordingFailure, RecordingState};
use file::{Failed, Flow, RecordFile};

pub use readout::suggested_name;

const TICK: Duration = Duration::from_secs(1);

pub struct RecorderDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    pub files: Arc<dyn RecordFiles>,
    pub unix_time_us: fn() -> f64,
}

struct Active {
    file: RecordFile,
    file_name: String,
    started: Instant,
    last_tick: Instant,
}

pub struct Recorder {
    bus: Bus,
    clock: Arc<dyn Clock>,
    files: Arc<dyn RecordFiles>,
    unix_time_us: fn() -> f64,
    items: Subscription<RollCaptured>,
    roll_open: bool,
    active: Option<Active>,
    failure: Option<RecordingFailure>,
    failure_count: u64,
    state: RecordingState,
}

impl Recorder {
    pub fn new(deps: RecorderDeps) -> Self {
        let items = deps.bus.subscribe();
        let mut recorder = Self {
            bus: deps.bus,
            clock: deps.clock,
            files: deps.files,
            unix_time_us: deps.unix_time_us,
            items,
            roll_open: false,
            active: None,
            failure: None,
            failure_count: 0,
            state: idle_state(0, None),
        };
        recorder.state = recorder.snapshot(0);
        recorder
    }

    pub fn state(&self) -> RecordingState {
        self.state.clone()
    }

    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn roll_open(&self) -> bool {
        self.roll_open
    }

    /// Takes the waiting items, then flushes and publishes the slice when a second has passed.
    pub fn pump(&mut self) {
        while let Some(captured) = self.items.try_next() {
            self.accept(&captured);
        }
        self.tick();
    }

    pub fn begin(&mut self, path: &Path) -> Result<SavedFile, KernelError> {
        let file_name = path
            .file_name()
            .map_or_else(|| "the file".to_owned(), |name| name.to_string_lossy().into_owned());
        let refuse =
            |failed: Failed| KernelError::recoverable(format!("cannot record to {file_name}: {}", failed.text));
        let writer = self.files.create(path).map_err(|error| refuse(Failed::of(&error)))?;
        let file = RecordFile::begin(writer, self.unix_time_us).map_err(refuse)?;
        let now = self.clock.now();
        self.failure = None;
        self.active = Some(Active {
            file,
            file_name: file_name.clone(),
            started: now,
            last_tick: now,
        });
        self.publish();
        Ok(SavedFile { file_name })
    }

    pub fn stop(&mut self) {
        let Some(mut active) = self.active.take() else {
            return;
        };
        match active.file.finish(EndReason::Stopped) {
            Ok(()) => self.publish(),
            Err(failed) => self.fail(active, failed),
        }
    }

    fn accept(&mut self, captured: &RollCaptured) {
        self.roll_open = !matches!(captured.item, RollItem::Closed { .. });
        let Some(mut active) = self.active.take() else {
            return;
        };
        match active.file.accept(&captured.item, &captured.context) {
            Ok(Flow::Continue) => self.active = Some(active),
            Ok(Flow::Ended) => self.publish(),
            Err(failed) => self.fail(active, failed),
        }
    }

    fn tick(&mut self) {
        let now = self.clock.now();
        let Some(mut active) = self.active.take() else {
            return;
        };
        if now.saturating_duration_since(active.last_tick) < TICK {
            self.active = Some(active);
            return;
        }
        active.last_tick = now;
        match active.file.flush() {
            Ok(()) => {
                self.active = Some(active);
                self.publish();
            }
            Err(failed) => self.fail(active, failed),
        }
    }

    fn fail(&mut self, active: Active, failed: Failed) {
        self.failure_count += 1;
        self.failure = Some(RecordingFailure {
            id: self.failure_count,
            text: failed.text.to_owned(),
            file_name: active.file_name,
            bytes: active.file.durable_bytes(),
        });
        self.publish();
    }

    fn publish(&mut self) {
        self.state = self.snapshot(self.state.revision + 1);
        self.bus.publish(RecordingChanged(self.state.clone()));
    }

    fn snapshot(&self, revision: u64) -> RecordingState {
        let Some(active) = &self.active else {
            return idle_state(revision, self.failure.clone());
        };
        RecordingState {
            revision,
            active: true,
            file_name: Some(active.file_name.clone()),
            duration: readout::duration(self.clock.now().saturating_duration_since(active.started)),
            bytes_written: readout::bytes(active.file.durable_bytes()),
            failure: self.failure.clone(),
        }
    }
}

fn idle_state(revision: u64, failure: Option<RecordingFailure>) -> RecordingState {
    RecordingState {
        revision,
        active: false,
        file_name: None,
        duration: readout::duration(Duration::ZERO),
        bytes_written: readout::bytes(0),
        failure,
    }
}

#[cfg(test)]
mod tests;
