//! Persists opaque UI-state blobs per namespace in one versioned `settings.json`.
#![forbid(unsafe_code)]

mod file;
mod messages;
mod module;

pub use messages::{LoadUiState, SaveUiState, UiStateEntry};
pub use module::{SettingsDeps, SettingsModule, COMMANDS};
