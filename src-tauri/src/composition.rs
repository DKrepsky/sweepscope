use std::path::PathBuf;
use std::sync::Arc;

use sweepscope_kernel::{Bus, FileStorage, KernelError, Storage};
use sweepscope_settings::{SettingsDeps, SettingsModule};

/// The wired backend. Modules are kept alive here; the bus is the only way in.
pub struct Backend {
    pub bus: Bus,
    _settings: SettingsModule,
}

pub fn compose(storage: Arc<dyn Storage>) -> Result<Backend, KernelError> {
    let bus = Bus::new();
    let settings = SettingsModule::new(SettingsDeps {
        bus: bus.clone(),
        storage,
    })?;
    Ok(Backend {
        bus,
        _settings: settings,
    })
}

pub fn compose_on_disk(config_dir: PathBuf) -> Result<Backend, KernelError> {
    compose(Arc::new(FileStorage::new(config_dir)))
}
