//! The application binary in CLI mode, without a display. Only recognised first arguments are passed:
//! anything else would open a window on the machine that runs the tests.
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use sweepscope_kernel::testing::TempDir;

const DEADLINE: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(20);

fn app<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(args: I) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sweepscope-app"));
    command
        .args(args)
        .env_remove("APPDIR")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env("SWEEPSCOPE_NO_HARDWARE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn run_app<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(args: I) -> Output {
    finish(&mut app(args))
}

fn finish(command: &mut Command) -> Output {
    let mut child = command.spawn().expect("the application binary starts");
    let started = Instant::now();
    while child.try_wait().expect("child status").is_none() {
        if started.elapsed() > DEADLINE {
            let _ = child.kill();
            let _ = child.wait();
            panic!("the application did not exit within {DEADLINE:?}");
        }
        thread::sleep(POLL);
    }
    child.wait_with_output().expect("child output")
}

fn setup_json() -> Value {
    let channel = |number: u8| {
        json!({"channel": number, "enabled": number == 1, "volts_per_div": 1.0, "position_divs": 0.0,
               "coupling": "dc", "bandwidth_limit": false, "probe": 1.0, "invert": false})
    };
    json!({"format": "sweepscope-setup", "version": 1,
           "channels": [channel(1), channel(2), channel(3), channel(4)],
           "time_base_s_per_div": 2e-4,
           "trigger": {"source": 1, "slope": "rising", "sweep": "auto", "level_volts": 0.0, "position_percent": 50.0}})
}

/// Runs `capture` with a setup file, empty config folder and output folder inside `dir`.
fn run_capture(dir: &TempDir, extra: &[&str]) -> Output {
    let setup = dir.path().join("setup.json");
    fs::write(&setup, setup_json().to_string()).unwrap();
    let paths = [
        ("--setup", setup),
        ("--config-dir", dir.path().join("config")),
        ("--out", dir.path().join("out")),
    ];
    let mut args = vec![std::ffi::OsString::from("capture")];
    for (name, path) in paths {
        args.push(name.into());
        args.push(path.into_os_string());
    }
    args.extend(extra.iter().map(Into::into));
    run_app(args)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn given_no_display_when_the_simulator_is_captured_then_the_files_are_written_and_the_exit_code_is_0() {
    let dir = TempDir::new();

    let output = run_capture(&dir, &["--simulator", "--count", "2"]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(dir.path().join("out/capture-0001.csv").is_file());
    assert!(dir.path().join("out/capture-0002.csv").is_file());
}

#[test]
fn given_no_hardware_when_captured_without_the_simulator_then_the_exit_code_is_10() {
    let dir = TempDir::new();

    let output = run_capture(&dir, &["--device-wait", "1"]);

    assert_eq!(output.status.code(), Some(10), "{}", stderr(&output));
}

#[test]
fn given_an_unknown_capture_option_when_run_then_the_exit_code_is_2_with_the_usage() {
    let output = run_app(["capture", "--bogus"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("usage: sweepscope capture"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn given_help_when_run_then_the_usage_is_printed_and_the_exit_code_is_0() {
    let output = run_app(["--help"]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("usage: sweepscope capture"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn given_version_when_run_then_the_version_is_printed_and_the_exit_code_is_0() {
    let output = run_app(["--version"]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        stdout(&output).trim(),
        concat!("sweepscope ", env!("CARGO_PKG_VERSION"))
    );
}

/// The folder holding the binary stands in for the mounted image.
fn build_folder() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_sweepscope-app")).parent().unwrap()
}

/// Stands in for `<image>/usr`; it has to be inside the build folder, so `TempDir` cannot be used.
struct ImageFolder(PathBuf);

impl ImageFolder {
    fn new(name: &str) -> Self {
        let path = build_folder().join(format!("cli-mode-{}-{name}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for ImageFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_setup(folder: &Path) {
    fs::write(folder.join("s.json"), setup_json().to_string()).unwrap();
}

/// Captures with relative paths, started in `cwd` the way an AppImage mounted at `app_dir` starts it.
fn run_moved(app_dir: &Path, cwd: &Path, pwd: &Path, extra: &[&str]) -> Output {
    let args = ["capture", "--setup", "s.json", "--simulator", "--config-dir", "cfg"];
    finish(
        app(args.iter().chain(extra))
            .env("APPDIR", app_dir)
            .env("PWD", pwd)
            .current_dir(cwd),
    )
}

#[test]
fn given_an_appimage_start_when_captured_with_relative_paths_then_the_files_are_in_the_caller_folder() {
    let image = ImageFolder::new("default-out");
    let caller = TempDir::new();
    write_setup(caller.path());

    let output = run_moved(build_folder(), &image.0, caller.path(), &[]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(caller.path().join("capture-0001.csv").is_file());
}

#[test]
fn given_an_appimage_start_when_captured_with_a_relative_out_then_it_is_inside_the_caller_folder() {
    let image = ImageFolder::new("relative-out");
    let caller = TempDir::new();
    write_setup(caller.path());

    let output = run_moved(build_folder(), &image.0, caller.path(), &["--out", "out"]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(caller.path().join("out/capture-0001.csv").is_file());
}

#[test]
fn given_an_appdir_not_holding_the_program_when_captured_then_pwd_is_ignored() {
    let image = ImageFolder::new("unrelated");
    write_setup(&image.0);
    let unrelated = TempDir::new();
    let caller = TempDir::new();

    let output = run_moved(unrelated.path(), &image.0, caller.path(), &[]);

    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(image.0.join("capture-0001.csv").is_file());
    assert!(!caller.path().join("capture-0001.csv").exists());
}

#[test]
fn given_a_caller_folder_that_cannot_be_entered_when_captured_then_it_fails_without_writing() {
    let image = ImageFolder::new("gone");
    write_setup(&image.0);
    let caller = TempDir::new();
    let gone = caller.path().join("gone");

    let output = run_moved(build_folder(), &image.0, &gone, &[]);

    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let expected = format!("sweepscope: cannot enter {}: ", gone.display());
    assert!(stderr(&output).starts_with(&expected), "{}", stderr(&output));
    assert!(!image.0.join("capture-0001.csv").exists());
}
