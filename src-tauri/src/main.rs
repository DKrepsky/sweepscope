#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod composition;
#[cfg(test)]
mod integration_tests;
mod ipc;
#[cfg(test)]
mod manifest;

use std::ffi::{OsStr, OsString};
use std::process::ExitCode;
use std::sync::Arc;

use ipc::events::{EventEmitter, EventForwarders};
use ipc::picker::DialogPicker;
use ipc::streams::{FrameStreams, RollStreams, SpectrumStreams};
use tauri::Manager;

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let first = args.first().and_then(|first| first.to_str());
    if first.is_some_and(sweepscope_cli::is_command) {
        return run_cli(args);
    }
    run_window();
    ExitCode::SUCCESS
}

/// Must stay free of Tauri and `env_logger` so capturing works without a display (ADR 0022).
fn run_cli(args: Vec<OsString>) -> ExitCode {
    // No lossy conversion: it would silently read or write a differently named file.
    let args: Vec<String> = match args.into_iter().map(OsString::into_string).collect() {
        Ok(args) => args,
        Err(argument) => return reject_argument(&argument),
    };
    if let Err(message) = return_to_caller_dir() {
        eprintln!("sweepscope: {message}");
        return ExitCode::FAILURE;
    }
    ExitCode::from(sweepscope_cli::run(
        &args,
        &|name| std::env::var(name).ok(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ))
}

fn reject_argument(argument: &OsStr) -> ExitCode {
    eprintln!(
        "sweepscope: argument is not valid UTF-8: {}\n{}",
        argument.to_string_lossy(),
        sweepscope_cli::USAGE
    );
    ExitCode::from(sweepscope_cli::ExitStatus::Usage.code())
}

/// So that relative paths work from an AppImage; the window must keep the folder it was started in.
fn return_to_caller_dir() -> Result<(), String> {
    let (Ok(exe), Ok(cwd)) = (std::env::current_exe(), std::env::current_dir()) else {
        return Ok(());
    };
    let Some(dir) = sweepscope_cli::appimage_caller_dir(&|name| std::env::var_os(name), &exe, &cwd) else {
        return Ok(());
    };
    std::env::set_current_dir(&dir).map_err(|error| format!("cannot enter {}: {error}", dir.display()))
}

fn run_window() {
    env_logger::init();
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let backend = composition::compose_on_disk(config_dir, Arc::new(DialogPicker::new(app.handle().clone())))?;
            let emitter: Arc<dyn EventEmitter> = Arc::new(app.handle().clone());
            app.manage(EventForwarders::start(&backend.bus, &emitter)?);
            app.manage(FrameStreams::default());
            app.manage(RollStreams::default());
            app.manage(SpectrumStreams::default());
            app.manage(backend);
            Ok(())
        });
    ipc::install(builder)
        .run(tauri::generate_context!())
        .expect("error while running SweepScope");
}
