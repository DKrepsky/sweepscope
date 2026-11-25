//! The application binary in CLI mode, without a display. Only recognised first arguments are passed:
//! anything else would open a window on the machine that runs the tests.
use std::ffi::OsStr;
use std::fs;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use sweepscope_kernel::testing::TempDir;

const DEADLINE: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(20);

fn run_app<I: IntoIterator<Item = S>, S: AsRef<OsStr>>(args: I) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sweepscope-app"))
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env("SWEEPSCOPE_NO_HARDWARE", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the application binary starts");
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
