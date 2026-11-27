//! The `capture` run: wait for the scope, load the setup, run, write N frames, stop (ADR 0022).
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sweepscope_acquisition::{InstrumentChanged, LoadSetup, SetRunning, ZeroOffsetsApplied};
use sweepscope_calibration::domain::offsets::OffsetFile;
use sweepscope_export::{encode_frame, iso_8601_utc};
use sweepscope_kernel::contracts::{
    ConnectionChanged, ConnectionIssueKind, ConnectionState, FrameCaptured, ZeroOffsetsChanged,
};
use sweepscope_kernel::{Bus, KernelError, Storage, Subscription};

use crate::args::CaptureOptions;
use crate::composition::{compose, Environment};
use crate::exit::{CliError, ExitStatus};
use crate::output::FrameFiles;
use crate::picker::SetupPath;

/// `log` receives one line per written file and nothing else, so scripts can read it.
pub fn capture(
    options: &CaptureOptions,
    environment: Environment,
    log: &mut dyn Write,
    warnings: &mut dyn Write,
) -> Result<(), CliError> {
    let files = FrameFiles::new(options.out.clone(), options.format, options.count);
    files.prepare()?;
    let result = run(options, environment, &files, log, warnings);
    if result.is_err() {
        files.remove_if_created_and_empty();
    }
    result
}

fn run(
    options: &CaptureOptions,
    environment: Environment,
    files: &FrameFiles,
    log: &mut dyn Write,
    warnings: &mut dyn Write,
) -> Result<(), CliError> {
    let bus = Bus::new();
    let connections = bus.subscribe::<ConnectionChanged>();
    let instruments = bus.subscribe::<InstrumentChanged>();
    let storage = Arc::clone(&environment.storage);
    let unix_time_us = environment.unix_time_us;
    let _backend = compose(&bus, environment, Arc::new(SetupPath(options.setup.clone()))).map_err(failure)?;

    let unit_id = wait_until_ready(&connections, &instruments, options.device_wait)?;
    publish_stored_offsets(&bus, storage.as_ref(), unit_id.as_deref(), options.device_wait)?;
    load_setup(&bus, warnings)?;
    let frames = bus.subscribe::<FrameCaptured>();
    bus.request(SetRunning { running: true }).map_err(failure)?;
    let result = write_frames(&frames, files, options, unix_time_us, log);
    let _ = bus.request(SetRunning { running: false });
    result
}

fn failure(error: KernelError) -> CliError {
    CliError::failure(error.message)
}

/// Returns the unit id of the ready device once acquisition has taken it over; commands fail before that.
/// A missing USB permission ends the wait early.
fn wait_until_ready(
    connections: &Subscription<ConnectionChanged>,
    instruments: &Subscription<InstrumentChanged>,
    wait: Duration,
) -> Result<Option<String>, CliError> {
    let deadline = Instant::now() + wait;
    let remaining = || deadline.saturating_duration_since(Instant::now());
    let mut last_issue = None;
    loop {
        let Some(change) = connections.recv_timeout(remaining()) else {
            return Err(no_device(wait, last_issue));
        };
        let snapshot = change.snapshot;
        match (snapshot.state, snapshot.issue) {
            (ConnectionState::Ready, _) => {
                instruments
                    .recv_timeout(remaining())
                    .ok_or_else(|| no_device(wait, None))?;
                return Ok(snapshot.identity.and_then(|identity| identity.unit_id));
            }
            (_, Some(issue)) if issue.kind == ConnectionIssueKind::PermissionDenied => {
                return Err(CliError::new(ExitStatus::PermissionDenied, issue.message));
            }
            (_, Some(issue)) => last_issue = Some(issue.message),
            _ => {}
        }
    }
}

fn no_device(wait: Duration, last_issue: Option<String>) -> CliError {
    let detail = last_issue.map_or_else(String::new, |issue| format!(" ({issue})"));
    CliError::new(
        ExitStatus::NoDevice,
        format!("no scope became ready within {}s{detail}", wait.as_secs_f64()),
    )
}

/// Returns once acquisition applied the offsets, so the first frame is already calibrated.
fn publish_stored_offsets(
    bus: &Bus,
    storage: &dyn Storage,
    unit_id: Option<&str>,
    wait: Duration,
) -> Result<(), CliError> {
    let offsets = OffsetFile::load(storage).map_err(failure)?;
    let offsets = unit_id.and_then(|unit| offsets.offsets_for(unit));
    let applied = bus.subscribe::<ZeroOffsetsApplied>();
    bus.publish(ZeroOffsetsChanged { offsets });
    applied
        .recv_timeout(wait)
        .map(|_| ())
        .ok_or_else(|| CliError::failure("the stored zero offsets were not applied"))
}

fn load_setup(bus: &Bus, warnings: &mut dyn Write) -> Result<(), CliError> {
    let loaded = bus.request(LoadSetup).map_err(failure)?;
    let loaded = loaded.ok_or_else(|| CliError::failure("the setup file was not loaded"))?;
    for warning in loaded.warnings {
        let _ = writeln!(warnings, "warning: {warning}");
    }
    Ok(())
}

fn write_frames(
    frames: &Subscription<FrameCaptured>,
    files: &FrameFiles,
    options: &CaptureOptions,
    unix_time_us: fn() -> f64,
    log: &mut dyn Write,
) -> Result<(), CliError> {
    for number in 1..=options.count {
        let captured = frames.recv_timeout(options.timeout).ok_or_else(|| {
            CliError::new(
                ExitStatus::Timeout,
                format!(
                    "no frame within {}s, {} of {} written",
                    options.timeout.as_secs_f64(),
                    number - 1,
                    options.count
                ),
            )
        })?;
        let bytes = encode_frame(&captured, &iso_8601_utc(unix_time_us()), options.format).map_err(failure)?;
        let path = files.write(number, &bytes)?;
        let _ = writeln!(log, "wrote {}", path.display());
    }
    Ok(())
}
