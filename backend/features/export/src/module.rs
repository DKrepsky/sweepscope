use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::Duration;

use sweepscope_kernel::contracts::{FrameCaptured, SavedFile};
use sweepscope_kernel::{write_atomic, Bus, FileFilter, FilePicker, KernelError, Subscription};

use crate::capture::Capture;
use crate::messages::{ExportFormat, ExportFrame, SaveScreenshot};
use crate::{csv, raw, stamp};

const POLL: Duration = Duration::from_millis(50);
const MAX_SCREENSHOT_BYTES: usize = 32 * 1024 * 1024;
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const FORMAT_DOC: &str = include_str!("../../../../docs/formats/frame-raw-v1.md");

pub struct ExportDeps {
    pub bus: Bus,
    pub picker: Arc<dyn FilePicker>,
    pub unix_time_us: fn() -> f64,
}

/// The newest frame and the moment it reached the backend.
struct Newest {
    frames: Subscription<FrameCaptured>,
    latest: Option<(FrameCaptured, String)>,
    unix_time_us: fn() -> f64,
}

impl Newest {
    fn absorb(&mut self) {
        while let Some(captured) = self.frames.try_next() {
            let arrived = stamp::iso_8601((self.unix_time_us)());
            self.latest = Some((captured, arrived));
        }
    }
}

struct Exporter {
    newest: Mutex<Newest>,
    picker: Arc<dyn FilePicker>,
    unix_time_us: fn() -> f64,
}

pub struct ExportModule {
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl ExportModule {
    pub fn new(deps: ExportDeps) -> Result<Self, KernelError> {
        let exporter = Arc::new(Exporter {
            newest: Mutex::new(Newest {
                frames: deps.bus.subscribe_latest::<FrameCaptured>(),
                latest: None,
                unix_time_us: deps.unix_time_us,
            }),
            picker: deps.picker,
            unix_time_us: deps.unix_time_us,
        });
        register_handlers(&deps.bus, &exporter)?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker = std::thread::Builder::new()
            .name("export".into())
            .spawn({
                let stop = Arc::clone(&stop);
                move || follow_frames(&exporter, &stop)
            })
            .map_err(|e| KernelError::fatal(format!("cannot start the export thread: {e}")))?;
        Ok(Self {
            stop,
            worker: Some(worker),
        })
    }
}

impl Drop for ExportModule {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn register_handlers(bus: &Bus, exporter: &Arc<Exporter>) -> Result<(), KernelError> {
    let frame_exporter = Arc::clone(exporter);
    bus.handle::<ExportFrame>(move |request| frame_exporter.export_frame(request.format))?;
    let screenshot_exporter = Arc::clone(exporter);
    bus.handle::<SaveScreenshot>(move |request| screenshot_exporter.save_screenshot(&request.png))
}

fn follow_frames(exporter: &Exporter, stop: &AtomicBool) {
    while !stop.load(Ordering::Acquire) {
        // The wait is a poll interval so the stop flag is seen; the frame itself is taken by `absorb`.
        std::thread::sleep(POLL);
        exporter.newest().absorb();
    }
}

impl Exporter {
    fn newest(&self) -> MutexGuard<'_, Newest> {
        self.newest.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn capture(&self) -> Result<Capture, KernelError> {
        let mut newest = self.newest();
        newest.absorb();
        let (captured, arrived) = newest
            .latest
            .as_ref()
            .ok_or_else(|| KernelError::recoverable("No waveform to export"))?;
        Capture::from_frame(captured, arrived.clone())
    }

    fn export_frame(&self, format: ExportFormat) -> Result<Option<SavedFile>, KernelError> {
        let capture = self.capture()?;
        let now = (self.unix_time_us)();
        match format {
            ExportFormat::Csv => {
                let name = stamp::suggested_name(now, "csv");
                let filter = filter("CSV", "csv");
                self.save_one(&name, &filter, csv::encode(&capture).as_bytes())
            }
            ExportFormat::Raw => {
                let name = stamp::suggested_name(now, "bin");
                let bytes = raw::encode(&capture)?;
                self.save_raw(&name, &bytes)
            }
        }
    }

    fn save_screenshot(&self, png: &[u8]) -> Result<Option<SavedFile>, KernelError> {
        if !png.starts_with(PNG_SIGNATURE) {
            return Err(KernelError::recoverable("The screenshot is not a PNG image"));
        }
        if png.len() > MAX_SCREENSHOT_BYTES {
            return Err(KernelError::recoverable("The screenshot is larger than 32 MiB"));
        }
        let name = stamp::suggested_name((self.unix_time_us)(), "png");
        self.save_one(&name, &filter("PNG image", "png"), png)
    }

    fn save_one(&self, name: &str, filter: &FileFilter, bytes: &[u8]) -> Result<Option<SavedFile>, KernelError> {
        let Some(path) = self.picker.pick_save(name, filter)? else {
            return Ok(None);
        };
        write_atomic(&path, bytes)?;
        Ok(Some(saved(&path)))
    }

    fn save_raw(&self, name: &str, bytes: &[u8]) -> Result<Option<SavedFile>, KernelError> {
        let Some(path) = self.picker.pick_save(name, &filter("Raw capture", "bin"))? else {
            return Ok(None);
        };
        let sidecar = path.with_extension("format.txt");
        write_atomic(&path, bytes)?;
        if let Err(error) = write_atomic(&sidecar, layout_text().as_bytes()) {
            let _ = std::fs::remove_file(&path);
            return Err(error);
        }
        Ok(Some(saved(&path)))
    }
}

fn filter(label: &str, extension: &str) -> FileFilter {
    FileFilter {
        label: label.into(),
        extensions: vec![extension.into()],
    }
}

fn saved(path: &Path) -> SavedFile {
    SavedFile {
        file_name: file_name(path),
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| PathBuf::from(path).display().to_string())
}

/// The text of the "Layout" section of the format document, which travels beside each raw file.
fn layout_text() -> String {
    let mut lines = FORMAT_DOC.lines().skip_while(|line| line.trim() != "## Layout").skip(1);
    let section: Vec<&str> = lines.by_ref().take_while(|line| !line.starts_with("## ")).collect();
    format!("{}\n", section.join("\n").trim())
}
