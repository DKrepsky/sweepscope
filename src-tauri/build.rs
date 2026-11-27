// Keep in sync with `ipc::REGISTERED_COMMANDS`; a test compares both with the command manifest.
const COMMANDS: &[&str] = &[
    "load_ui_state",
    "save_ui_state",
    "get_snapshot",
    "set_interpolation",
    "export_frame",
    "save_screenshot",
    "save_setup",
    "load_setup",
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
    "arm_single",
    "start_autoset",
    "set_math",
    "set_persistence",
    "set_spectrum",
    "start_recording",
    "stop_recording",
    "open_roll_stream",
    "close_roll_stream",
    "open_spectrum_stream",
    "close_spectrum_stream",
    "open_zero_calibration",
    "start_zero_calibration",
    "measure_zero_calibration",
    "continue_zero_calibration",
    "close_zero_calibration",
];

fn main() {
    let app = tauri_build::AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app))
        .expect("failed to run the tauri build step");
}
