#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod composition;
#[cfg(test)]
mod integration_tests;
mod ipc;
#[cfg(test)]
mod manifest;

use std::process::ExitCode;
use std::sync::Arc;

use ipc::events::{EventEmitter, EventForwarders};
use ipc::picker::DialogPicker;
use ipc::streams::{FrameStreams, RollStreams, SpectrumStreams};
use tauri::Manager;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    if args.first().is_some_and(|first| sweepscope_cli::is_command(first)) {
        return run_cli(&args);
    }
    run_window();
    ExitCode::SUCCESS
}

/// Must stay free of Tauri and `env_logger` so capturing works without a display (ADR 0022).
fn run_cli(args: &[String]) -> ExitCode {
    let env = |name: &str| std::env::var(name).ok();
    ExitCode::from(sweepscope_cli::run(
        args,
        &env,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ))
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
