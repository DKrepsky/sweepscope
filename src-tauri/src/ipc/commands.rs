use sweepscope_kernel::{Bus, KernelError};
use sweepscope_settings::{LoadUiState, SaveUiState, UiStateEntry};
use tauri::State;

use crate::composition::Backend;

/// Every command passed to `handler`; compared in tests with the manifest, `build.rs` and the capability.
#[cfg(test)]
pub const REGISTERED_COMMANDS: &[&str] = &["load_ui_state", "save_ui_state"];

pub fn load_ui_state_with(bus: &Bus, namespace: String) -> Result<Option<UiStateEntry>, KernelError> {
    bus.request(LoadUiState { namespace })
}

pub fn save_ui_state_with(bus: &Bus, entry: UiStateEntry) -> Result<(), KernelError> {
    bus.request(SaveUiState(entry))
}

#[tauri::command]
fn load_ui_state(backend: State<'_, Backend>, namespace: String) -> Result<Option<UiStateEntry>, KernelError> {
    load_ui_state_with(&backend.bus, namespace)
}

#[tauri::command]
fn save_ui_state(backend: State<'_, Backend>, entry: UiStateEntry) -> Result<(), KernelError> {
    save_ui_state_with(&backend.bus, entry)
}

pub fn install(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![load_ui_state, save_ui_state])
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sweepscope_kernel::testing::MemoryStorage;
    use sweepscope_kernel::ErrorCategory;

    use super::*;
    use crate::composition::compose;

    fn entry(namespace: &str) -> UiStateEntry {
        UiStateEntry {
            namespace: namespace.into(),
            version: 1,
            json: "{\"percent\":100}".into(),
        }
    }

    fn composed_bus() -> Backend {
        compose(Arc::new(MemoryStorage::default())).expect("compose")
    }

    #[test]
    fn given_a_composed_app_when_an_unknown_namespace_is_loaded_then_it_returns_none() {
        let backend = composed_bus();
        let loaded = load_ui_state_with(&backend.bus, "app.unknown".into());
        assert_eq!(loaded, Ok(None));
    }

    #[test]
    fn given_a_saved_entry_when_loaded_through_the_handlers_then_it_round_trips() {
        let backend = composed_bus();
        save_ui_state_with(&backend.bus, entry("app.text-scale")).expect("save");
        let loaded = load_ui_state_with(&backend.bus, "app.text-scale".into());
        assert_eq!(loaded, Ok(Some(entry("app.text-scale"))));
    }

    #[test]
    fn given_a_handler_error_when_serialised_for_the_webview_then_it_has_category_and_message_without_paths() {
        let error = load_ui_state_with(&Bus::new(), "app.text-scale".into()).expect_err("no handler registered");

        let json = serde_json::to_value(&error).expect("serialise");
        assert_eq!(error.category, ErrorCategory::Fatal);
        assert_eq!(json["category"], "fatal");
        assert!(json["message"].is_string());
        assert!(!error.message.contains('/') && !error.message.contains('\\'));
    }

    #[test]
    fn given_the_registered_commands_when_compared_with_the_manifest_then_they_are_equal() {
        let manifest: Vec<&str> = sweepscope_settings::COMMANDS.iter().map(|c| c.name).collect();
        assert_eq!(REGISTERED_COMMANDS, manifest.as_slice());
    }
}
