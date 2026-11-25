use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use sweepscope_calibration::domain::offsets::{OffsetFile, StoredUnit};
use sweepscope_cli::{capture, run, simulated_environment, CaptureOptions, CliError, ExitStatus, ReadOnlyStorage};
use sweepscope_device::{SignalKind, SignalSpec, SimulatorControl};
use sweepscope_export::{decode_raw, ExportFormat};
use sweepscope_kernel::testing::TempDir;
use sweepscope_kernel::{FileStorage, Storage, SystemClock};

const FAST_TIME_BASE_S: f64 = 2e-4;
const SIMULATOR_UNIT: &str = "simulator";

fn setup_json(sweep: &str) -> Value {
    let channel = |number: u8| {
        json!({"channel": number, "enabled": number == 1, "volts_per_div": 1.0, "position_divs": 0.0,
               "coupling": "dc", "bandwidth_limit": false, "probe": 1.0, "invert": false})
    };
    json!({"format": "sweepscope-setup", "version": 1,
           "channels": [channel(1), channel(2), channel(3), channel(4)],
           "time_base_s_per_div": FAST_TIME_BASE_S,
           "trigger": {"source": 1, "slope": "rising", "sweep": sweep, "level_volts": 0.0, "position_percent": 50.0}})
}

struct Rig {
    dir: TempDir,
    config: TempDir,
    control: SimulatorControl,
}

impl Rig {
    fn new(sweep: &str) -> Self {
        let rig = Self {
            dir: TempDir::new(),
            config: TempDir::new(),
            control: SimulatorControl::new(),
        };
        fs::write(rig.setup(), setup_json(sweep).to_string()).unwrap();
        rig
    }

    fn setup(&self) -> PathBuf {
        self.dir.path().join("setup.json")
    }

    fn out(&self) -> PathBuf {
        self.dir.path().join("out")
    }

    fn options(&self, count: u32, format: ExportFormat) -> CaptureOptions {
        CaptureOptions {
            setup: self.setup(),
            count,
            out: self.out(),
            format,
            timeout: Duration::from_secs(20),
            device_wait: Duration::from_secs(20),
            simulator: true,
            config_dir: Some(self.config.path().to_path_buf()),
        }
    }

    fn run(&self, options: &CaptureOptions) -> Result<String, CliError> {
        let storage: Arc<dyn Storage> = Arc::new(ReadOnlyStorage::new(self.config.path()));
        let environment = simulated_environment(storage, Arc::new(SystemClock), self.control.clone());
        let mut log = Vec::new();
        capture(options, environment, &mut log, &mut Vec::new()).map(|()| String::from_utf8(log).unwrap())
    }

    fn files(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.out()) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn raw_header(&self, file: &str) -> Value {
        let bytes = fs::read(self.out().join(file)).unwrap();
        let length = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        serde_json::from_slice(&bytes[8..8 + length]).unwrap()
    }
}

fn store_offsets(config: &Path, unit: &str) {
    let stored = StoredUnit {
        stored_at_utc: "2026-11-24T00:00:00Z".into(),
        volts: [vec![0.01; 12], vec![], vec![], vec![]],
    };
    OffsetFile::default()
        .with_unit(unit, stored)
        .save(&FileStorage::new(config))
        .unwrap();
}

fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|e| {
            let e = e.unwrap();
            (
                e.file_name().to_string_lossy().into_owned(),
                fs::read(e.path()).unwrap(),
            )
        })
        .collect();
    files.sort();
    files
}

#[test]
fn given_the_simulator_when_three_csv_frames_are_captured_then_three_files_exist_and_the_run_succeeds() {
    let rig = Rig::new("auto");

    let log = rig.run(&rig.options(3, ExportFormat::Csv)).expect("capture");

    assert_eq!(
        rig.files(),
        ["capture-0001.csv", "capture-0002.csv", "capture-0003.csv"]
    );
    assert_eq!(log.lines().count(), 3);
}

#[test]
fn given_raw_format_when_two_frames_are_captured_then_both_decode_and_the_layout_is_written_once() {
    let rig = Rig::new("auto");

    rig.run(&rig.options(2, ExportFormat::Raw)).expect("capture");

    assert_eq!(
        rig.files(),
        ["capture-0001.bin", "capture-0002.bin", "capture.format.txt"]
    );
    let capture = decode_raw(&fs::read(rig.out().join("capture-0002.bin")).unwrap()).expect("decodes");
    assert_eq!(capture.channels.len(), 1);
}

#[test]
fn given_no_scope_when_the_device_wait_ends_then_the_status_is_no_device() {
    let rig = Rig::new("auto");
    rig.control.unplug();
    let mut options = rig.options(1, ExportFormat::Csv);
    options.device_wait = Duration::ZERO;

    let error = rig.run(&options).unwrap_err();

    assert_eq!(error.status, ExitStatus::NoDevice);
    assert!(rig.files().is_empty());
    assert!(!rig.out().exists());
}

#[test]
fn given_a_normal_sweep_on_a_flat_input_when_the_frame_timeout_passes_then_the_status_is_timeout() {
    let rig = Rig::new("normal");
    rig.control
        .set_signal(0, SignalSpec::new(SignalKind::Sine).with_amplitude(0.0));
    let mut options = rig.options(2, ExportFormat::Csv);
    options.timeout = Duration::ZERO;

    let error = rig.run(&options).unwrap_err();

    assert_eq!(error.status, ExitStatus::Timeout);
    assert!(error.message.contains("0 of 2"), "{error}");
}

#[test]
fn given_an_existing_target_when_captured_then_the_run_fails_and_the_file_is_untouched() {
    let rig = Rig::new("auto");
    fs::create_dir_all(rig.out()).unwrap();
    fs::write(rig.out().join("capture-0002.csv"), b"mine").unwrap();

    let error = rig.run(&rig.options(2, ExportFormat::Csv)).unwrap_err();

    assert_eq!(error.status, ExitStatus::Failure);
    assert_eq!(rig.files(), ["capture-0002.csv"]);
    assert_eq!(fs::read(rig.out().join("capture-0002.csv")).unwrap(), b"mine");
}

#[cfg(unix)]
#[test]
fn given_a_read_only_output_folder_when_captured_then_the_status_is_permission_denied() {
    use std::os::unix::fs::PermissionsExt;
    let rig = Rig::new("auto");
    fs::create_dir_all(rig.out()).unwrap();
    fs::set_permissions(rig.out(), fs::Permissions::from_mode(0o555)).unwrap();
    let writable_anyway = fs::File::create(rig.out().join("probe")).is_ok();

    let result = rig.run(&rig.options(1, ExportFormat::Csv));

    fs::set_permissions(rig.out(), fs::Permissions::from_mode(0o755)).unwrap();
    if writable_anyway {
        return; // running as root: the permission cannot be denied
    }
    assert_eq!(result.unwrap_err().status, ExitStatus::PermissionDenied);
}

#[test]
fn given_a_setup_file_that_is_missing_when_captured_then_the_run_fails() {
    let rig = Rig::new("auto");
    fs::remove_file(rig.setup()).unwrap();

    let error = rig.run(&rig.options(1, ExportFormat::Csv)).unwrap_err();

    assert_eq!(error.status, ExitStatus::Failure);
    assert!(error.message.contains("setup.json"), "{error}");
}

#[test]
fn given_stored_offsets_of_the_unit_when_captured_then_the_frame_is_calibrated_and_the_config_is_unmodified() {
    let rig = Rig::new("auto");
    store_offsets(rig.config.path(), SIMULATOR_UNIT);
    let before = snapshot(rig.config.path());

    rig.run(&rig.options(1, ExportFormat::Raw)).expect("capture");

    assert_eq!(rig.raw_header("capture-0001.bin")["channels"][0]["calibrated"], true);
    assert_eq!(snapshot(rig.config.path()), before);
}

#[test]
fn given_stored_offsets_of_the_unit_when_several_frames_are_captured_then_every_frame_is_calibrated_from_the_first() {
    let rig = Rig::new("auto");
    store_offsets(rig.config.path(), SIMULATOR_UNIT);

    rig.run(&rig.options(3, ExportFormat::Raw)).expect("capture");

    for file in ["capture-0001.bin", "capture-0002.bin", "capture-0003.bin"] {
        assert_eq!(rig.raw_header(file)["channels"][0]["calibrated"], true, "{file}");
    }
}

#[test]
fn given_no_stored_offsets_when_captured_then_the_frame_is_not_calibrated_and_the_config_stays_empty() {
    let rig = Rig::new("auto");

    rig.run(&rig.options(1, ExportFormat::Raw)).expect("capture");

    assert_eq!(rig.raw_header("capture-0001.bin")["channels"][0]["calibrated"], false);
    assert!(snapshot(rig.config.path()).is_empty());
}

#[test]
fn given_a_clamped_trigger_level_when_run_then_the_warning_is_on_stderr_and_stdout_lists_only_files() {
    let rig = Rig::new("auto");
    let mut setup = setup_json("auto");
    setup["trigger"]["level_volts"] = json!(20.0);
    fs::write(rig.setup(), setup.to_string()).unwrap();
    let args: Vec<String> = [
        ("--setup", rig.setup()),
        ("--config-dir", rig.config.path().to_path_buf()),
        ("--out", rig.out()),
    ]
    .iter()
    .flat_map(|(name, path)| [(*name).to_owned(), path.to_str().unwrap().to_owned()])
    .collect();
    let args = [vec!["capture".to_owned(), "--simulator".to_owned()], args].concat();
    let (mut out, mut err) = (Vec::new(), Vec::new());

    let code = run(&args, &|_| None, &mut out, &mut err);

    let (out, err) = (String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap());
    assert_eq!(code, 0, "{err}");
    assert!(err.starts_with("warning: Trigger level"), "{err}");
    assert!(out.lines().all(|line| line.starts_with("wrote ")), "{out}");
    assert_eq!(out.lines().count(), 1, "{out}");
}
