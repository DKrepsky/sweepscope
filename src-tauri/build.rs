// Keep in sync with `ipc::REGISTERED_COMMANDS`; a test compares both with the settings manifest.
const COMMANDS: &[&str] = &["load_ui_state", "save_ui_state"];

fn main() {
    let app = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app))
        .expect("failed to run the tauri build step");
}
