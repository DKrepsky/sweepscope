#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod composition;
#[cfg(test)]
mod integration_tests;
mod ipc;
#[cfg(test)]
mod manifest;

use std::sync::Arc;

use ipc::events::{EventEmitter, EventForwarders};
use ipc::picker::DialogPicker;
use ipc::streams::{FrameStreams, RollStreams, SpectrumStreams};
use tauri::Manager;

fn main() {
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
