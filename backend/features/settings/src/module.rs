use std::sync::{Arc, Mutex};

use sweepscope_kernel::{Bus, CommandDescriptor, KernelError, Storage};

use crate::{file, LoadUiState, SaveUiState};

/// Frontend-callable commands of this feature, consumed by the shell to generate `commands.ts`.
pub const COMMANDS: &[CommandDescriptor] = &[
    CommandDescriptor {
        name: "load_ui_state",
        args: "{ namespace: string }",
        result: "UiStateEntry | null",
        types: &["UiStateEntry"],
    },
    CommandDescriptor {
        name: "save_ui_state",
        args: "{ entry: UiStateEntry }",
        result: "null",
        types: &["UiStateEntry"],
    },
];

pub struct SettingsDeps {
    pub bus: Bus,
    pub storage: Arc<dyn Storage>,
}

pub struct SettingsModule;

impl SettingsModule {
    /// Registers the request handlers; fails if another module already handles them.
    pub fn new(deps: SettingsDeps) -> Result<Self, KernelError> {
        let write_lock = Arc::new(Mutex::new(()));
        let storage = deps.storage;

        let reader = Arc::clone(&storage);
        deps.bus
            .handle(move |request: LoadUiState| file::load(reader.as_ref(), &request.namespace))?;
        deps.bus.handle(move |SaveUiState(entry)| {
            let _guard = write_lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            file::save(storage.as_ref(), entry)
        })?;
        Ok(Self)
    }
}
