// Keep in sync with `ipc::REGISTERED_COMMANDS`; a test compares both with the command manifest.
const COMMANDS: &[&str] = &[
    "load_ui_state",
    "save_ui_state",
    "get_snapshot",
    "set_running",
    "force_trigger",
    "set_channel",
    "step_setting",
    "reset_setting",
    "set_trigger_position",
    "set_trigger_level",
    "set_trigger",
    "use_simulator",
    "retry_connection",
    "open_frame_stream",
    "close_frame_stream",
];

fn main() {
    let app = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app))
        .expect("failed to run the tauri build step");
}
