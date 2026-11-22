#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

mod composition;
mod ipc;
#[cfg(test)]
mod manifest;

use tauri::Manager;

fn main() {
    let builder = tauri::Builder::default().setup(|app| {
        let config_dir = app.path().app_config_dir()?;
        app.manage(composition::compose_on_disk(config_dir)?);
        Ok(())
    });
    ipc::install(builder)
        .run(tauri::generate_context!())
        .expect("error while running SweepScope");
}
