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
use ipc::frames::FrameStreams;
use tauri::Manager;

fn main() {
    env_logger::init();
    let builder = tauri::Builder::default().setup(|app| {
        let config_dir = app.path().app_config_dir()?;
        let backend = composition::compose_on_disk(config_dir)?;
        let emitter: Arc<dyn EventEmitter> = Arc::new(app.handle().clone());
        app.manage(EventForwarders::start(&backend.bus, &emitter)?);
        app.manage(FrameStreams::default());
        app.manage(backend);
        Ok(())
    });
    ipc::install(builder)
        .run(tauri::generate_context!())
        .expect("error while running SweepScope");
}
