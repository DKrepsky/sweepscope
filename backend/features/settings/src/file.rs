use std::collections::BTreeMap;

use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sweepscope_kernel::{KernelError, Storage};

use crate::UiStateEntry;

pub const SETTINGS_KEY: &str = "settings.json";
const CORRUPT_BACKUP_KEY: &str = "settings.json.corrupt";
const MAX_NAMESPACE_LEN: usize = 64;
const MAX_NAMESPACES: usize = 64;
const MAX_JSON_BYTES: usize = 64 * 1024;
const MIN_VERSION: u32 = 1;
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
    validate_namespace(namespace)?;
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

/// Read-modify-write of the whole file. A corrupt file is kept as a backup before it is replaced
/// (the first backup is never overwritten); a file from a newer version is never touched.
pub fn save(storage: &dyn Storage, entry: UiStateEntry) -> Result<(), KernelError> {
    validate(&entry)?;
    let mut file = current_file(storage)?;
    if file.ui.len() >= MAX_NAMESPACES && !file.ui.contains_key(&entry.namespace) {
        return Err(KernelError::recoverable("too many ui state entries"));
    }
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

fn validate(entry: &UiStateEntry) -> Result<(), KernelError> {
    validate_namespace(&entry.namespace)?;
    if entry.version < MIN_VERSION {
        return Err(KernelError::recoverable("invalid ui state version"));
    }
    if entry.json.len() > MAX_JSON_BYTES {
        return Err(KernelError::recoverable("ui state is too large"));
    }
    serde_json::from_str::<IgnoredAny>(&entry.json)
        .map(drop)
        .map_err(|_| KernelError::recoverable("ui state is not valid JSON"))
}

/// Matches `^[a-z][a-z0-9.-]{0,63}$`.
fn validate_namespace(namespace: &str) -> Result<(), KernelError> {
    let mut chars = namespace.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    let rest_ok = chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-'));
    if first_ok && rest_ok && namespace.len() <= MAX_NAMESPACE_LEN {
        Ok(())
    } else {
        Err(KernelError::recoverable("invalid ui state namespace"))
    }
}

fn current_file(storage: &dyn Storage) -> Result<SettingsFile, KernelError> {
    let Some(bytes) = storage.read(SETTINGS_KEY)? else {
        return Ok(SettingsFile::default());
    };
    match parse(&bytes)? {
        Parsed::Valid(file) => Ok(file),
        Parsed::Corrupt => {
            keep_first_backup(storage, &bytes)?;
            Ok(SettingsFile::default())
        }
    }
}

fn keep_first_backup(storage: &dyn Storage, bytes: &[u8]) -> Result<(), KernelError> {
    if storage.read(CORRUPT_BACKUP_KEY)?.is_some() {
        return Ok(());
    }
    storage.write(CORRUPT_BACKUP_KEY, bytes)
}
