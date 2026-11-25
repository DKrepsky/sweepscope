//! Headless capture: a second composition root without Tauri, run by the application binary (ADR 0022).
#![forbid(unsafe_code)]

mod args;
mod capture;
mod composition;
mod config;
mod exit;
mod output;
mod picker;

use std::io::Write;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sweepscope_device::{DeviceCatalog, HantekCatalog, SimulatedCatalog, SimulatorControl, SimulatorSource};
use sweepscope_kernel::{Clock, Storage, SystemClock};

pub use args::{is_command, parse, CaptureOptions, Command, USAGE};
pub use capture::capture;
pub use composition::Environment;
pub use config::{config_dir, ReadOnlyStorage, APP_FOLDER};
pub use exit::{CliError, ExitStatus};

const NO_HARDWARE_VARIABLE: &str = "SWEEPSCOPE_NO_HARDWARE";

/// Runs the program for `args` (without the program name) and returns the process exit code.
pub fn run(args: &[String], env: &dyn Fn(&str) -> Option<String>, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    match execute(args, env, out) {
        Ok(()) => ExitStatus::Success.code(),
        Err(error) => {
            let _ = writeln!(err, "sweepscope: {}", error.message);
            if error.status == ExitStatus::Usage {
                let _ = writeln!(err, "{USAGE}");
            }
            error.status.code()
        }
    }
}

fn execute(args: &[String], env: &dyn Fn(&str) -> Option<String>, out: &mut dyn Write) -> Result<(), CliError> {
    match parse(args)? {
        Command::Help => print(out, USAGE),
        Command::Version => print(out, concat!("sweepscope ", env!("CARGO_PKG_VERSION"))),
        Command::Capture(options) => {
            let environment = system_environment(&options, env)?;
            capture(&options, environment, out)
        }
    }
}

fn print(out: &mut dyn Write, text: &str) -> Result<(), CliError> {
    writeln!(out, "{text}").map_err(|error| CliError::failure(error.to_string()))
}

fn system_environment(options: &CaptureOptions, env: &dyn Fn(&str) -> Option<String>) -> Result<Environment, CliError> {
    let folder = config_dir(options.config_dir.clone(), env)
        .ok_or_else(|| CliError::failure("cannot find the config folder, give --config-dir"))?;
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let storage: Arc<dyn Storage> = Arc::new(ReadOnlyStorage::new(folder));
    let hardware_off = options.simulator || env(NO_HARDWARE_VARIABLE).as_deref() == Some("1");
    Ok(Environment {
        storage,
        hardware: if hardware_off { None } else { hardware_catalog() },
        simulator: options.simulator.then(|| simulator_source(Arc::clone(&clock))),
        clock,
        unix_time_us,
    })
}

fn hardware_catalog() -> Option<Box<dyn DeviceCatalog>> {
    HantekCatalog::start()
        .ok()
        .map(|catalog| Box::new(catalog) as Box<dyn DeviceCatalog>)
}

fn simulator_source(clock: Arc<dyn Clock>) -> SimulatorSource {
    let control = SimulatorControl::new();
    SimulatorSource {
        catalog: Box::new(SimulatedCatalog::new(clock, control.clone())),
        control,
    }
}

fn unix_time_us() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |since| since.as_secs_f64() * 1_000_000.0)
}

/// For tests and tools that build their own environment around the simulator.
pub fn simulated_environment(
    storage: Arc<dyn Storage>,
    clock: Arc<dyn Clock>,
    control: SimulatorControl,
) -> Environment {
    Environment {
        storage,
        hardware: None,
        simulator: Some(SimulatorSource {
            catalog: Box::new(SimulatedCatalog::new(Arc::clone(&clock), control.clone())),
            control,
        }),
        clock,
        unix_time_us,
    }
}
