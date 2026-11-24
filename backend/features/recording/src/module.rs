use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::SavedFile;
use sweepscope_kernel::{Bus, Clock, FileFilter, FilePicker, KernelError};

use crate::files::RecordFiles;
use crate::messages::{GetRecording, RecordingState, StartRecording, StopRecording};
use crate::recorder::{suggested_name, Recorder, RecorderDeps};

const POLL: Duration = Duration::from_millis(50);
const NEEDS_ROLL: &str = "Recording needs roll mode (500 ms/div or slower)";
const ALREADY_RECORDING: &str = "Recording is already running";

pub struct RecordingDeps {
    pub bus: Bus,
    pub clock: Arc<dyn Clock>,
    pub picker: Arc<dyn FilePicker>,
    pub files: Arc<dyn RecordFiles>,
    pub unix_time_us: fn() -> f64,
}

struct Shared {
    recorder: Mutex<Recorder>,
    picker: Arc<dyn FilePicker>,
    unix_time_us: fn() -> f64,
}

pub struct RecordingModule {
    shared: Arc<Shared>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl RecordingModule {
    pub fn new(deps: RecordingDeps) -> Result<Self, KernelError> {
        let recorder = Recorder::new(RecorderDeps {
            bus: deps.bus.clone(),
            clock: deps.clock,
            files: deps.files,
            unix_time_us: deps.unix_time_us,
        });
        let shared = Arc::new(Shared {
            recorder: Mutex::new(recorder),
            picker: deps.picker,
            unix_time_us: deps.unix_time_us,
        });
        register_handlers(&deps.bus, &shared)?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("recording".into())
            .spawn({
                let shared = Arc::clone(&shared);
                let stop = Arc::clone(&stop);
                move || follow_items(&shared, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the recording thread: {e}")))?;
        Ok(Self {
            shared,
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for RecordingModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        self.shared.recorder().stop();
    }
}

fn register_handlers(bus: &Bus, shared: &Arc<Shared>) -> Result<(), KernelError> {
    let start = Arc::clone(shared);
    bus.handle::<StartRecording>(move |_| start.start())?;
    let stop = Arc::clone(shared);
    bus.handle::<StopRecording>(move |_| {
        let mut recorder = stop.recorder();
        recorder.pump();
        recorder.stop();
        Ok(())
    })?;
    let state = Arc::clone(shared);
    bus.handle::<GetRecording>(move |_| Ok(state.state()))
}

fn follow_items(shared: &Shared, stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        // A poll interval so the stop flag is seen; the bus queue holds the items that arrive meanwhile.
        std::thread::sleep(POLL);
        shared.recorder().pump();
    }
}

impl Shared {
    fn recorder(&self) -> MutexGuard<'_, Recorder> {
        self.recorder.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn state(&self) -> RecordingState {
        self.recorder().state()
    }

    /// The dialog runs without the lock, so items keep being taken while the user chooses a file.
    fn start(&self) -> Result<Option<SavedFile>, KernelError> {
        self.require_idle_roll_session()?;
        let name = suggested_name((self.unix_time_us)());
        let filter = FileFilter {
            label: "Roll recording".to_owned(),
            extensions: vec!["ssroll".to_owned()],
        };
        let Some(path) = self.picker.pick_save(&name, &filter)? else {
            return Ok(None);
        };
        let mut recorder = self.recorder();
        recorder.pump();
        require_roll_session(&recorder)?;
        recorder.begin(&path).map(Some)
    }

    fn require_idle_roll_session(&self) -> Result<(), KernelError> {
        let mut recorder = self.recorder();
        recorder.pump();
        require_roll_session(&recorder)?;
        if recorder.is_active() {
            return Err(KernelError::recoverable(ALREADY_RECORDING));
        }
        Ok(())
    }
}

fn require_roll_session(recorder: &Recorder) -> Result<(), KernelError> {
    if recorder.roll_open() {
        Ok(())
    } else {
        Err(KernelError::recoverable(NEEDS_ROLL))
    }
}
