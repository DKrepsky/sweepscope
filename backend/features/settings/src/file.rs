use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sweepscope_kernel::{KernelError, Storage};

use crate::UiStateEntry;

pub const SETTINGS_KEY: &str = "settings.json";
const CORRUPT_BACKUP_KEY: &str = "settings.json.corrupt";
const SUPPORTED_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct StoredEntry {
    version: u32,
    json: String,
}

#[derive(Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    ui: BTreeMap<String, StoredEntry>,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            version: SUPPORTED_VERSION,
            ui: BTreeMap::new(),
        }
    }
}

enum Parsed {
    Valid(SettingsFile),
    Corrupt,
}

fn parse(bytes: &[u8]) -> Result<Parsed, KernelError> {
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return Ok(Parsed::Corrupt);
    };
    let version = value.get("version").and_then(Value::as_u64);
    if version.is_some_and(|v| v > u64::from(SUPPORTED_VERSION)) {
        return Err(KernelError::recoverable("settings file was written by a newer version"));
    }
    Ok(serde_json::from_value(value).map_or(Parsed::Corrupt, Parsed::Valid))
}

fn corrupt_error() -> KernelError {
    KernelError::recoverable("settings file is unreadable")
}

pub fn load(storage: &dyn Storage, namespace: &str) -> Result<Option<UiStateEntry>, KernelError> {
    let Some(bytes) = storage.read(SETTINGS_KEY)? else {
        return Ok(None);
    };
    match parse(&bytes)? {
        Parsed::Corrupt => Err(corrupt_error()),
        Parsed::Valid(file) => Ok(file.ui.get(namespace).map(|stored| UiStateEntry {
            namespace: namespace.to_owned(),
            version: stored.version,
            json: stored.json.clone(),
        })),
    }
}

/// Read-modify-write of the whole file. A corrupt file is kept as a backup before it is replaced;
/// a file from a newer version is never touched.
pub fn save(storage: &dyn Storage, entry: UiStateEntry) -> Result<(), KernelError> {
    let mut file = current_file(storage)?;
    file.ui.insert(
        entry.namespace,
        StoredEntry {
            version: entry.version,
            json: entry.json,
        },
    );
    let bytes = serde_json::to_vec_pretty(&file).map_err(|_| KernelError::fatal("settings could not be encoded"))?;
    storage.write(SETTINGS_KEY, &bytes)
}

fn current_file(storage: &dyn Storage) -> Result<SettingsFile, KernelError> {
    let Some(bytes) = storage.read(SETTINGS_KEY)? else {
        return Ok(SettingsFile::default());
    };
    match parse(&bytes)? {
        Parsed::Valid(file) => Ok(file),
        Parsed::Corrupt => {
            storage.write(CORRUPT_BACKUP_KEY, &bytes)?;
            Ok(SettingsFile::default())
        }
    }
}
