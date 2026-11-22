use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::KernelError;

pub trait Storage: Send + Sync {
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>, KernelError>;
    fn write(&self, key: &str, bytes: &[u8]) -> Result<(), KernelError>;
}

/// Stores each key as a file in one directory. Writes go to a temp file first and are renamed into place.
#[derive(Debug)]
pub struct FileStorage {
    root: PathBuf,
    write_lock: Mutex<()>,
}

impl FileStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            write_lock: Mutex::new(()),
        }
    }

    fn path_for(&self, key: &str) -> Result<PathBuf, KernelError> {
        let plain = !key.is_empty() && !key.starts_with('.') && !key.contains(['/', '\\']);
        if plain {
            Ok(self.root.join(key))
        } else {
            Err(KernelError::fatal(format!("invalid storage key {key:?}")))
        }
    }

    fn stage(&self, target: &Path, bytes: &[u8]) -> Result<PathBuf, KernelError> {
        fs::create_dir_all(&self.root).map_err(io_error("create the storage folder"))?;
        let mut name = target.file_name().unwrap_or_default().to_os_string();
        name.push(format!(".{}.tmp", std::process::id()));
        let staged = target.with_file_name(name);
        let mut file = File::create(&staged).map_err(io_error("write"))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(io_error("write"))?;
        Ok(staged)
    }
}

fn io_error(action: &'static str) -> impl Fn(std::io::Error) -> KernelError {
    move |error| KernelError::recoverable(format!("storage failed to {action}: {}", error.kind()))
}

impl Storage for FileStorage {
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>, KernelError> {
        match fs::read(self.path_for(key)?) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io_error("read")(error)),
        }
    }

    fn write(&self, key: &str, bytes: &[u8]) -> Result<(), KernelError> {
        let target = self.path_for(key)?;
        let _guard = self.write_lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let staged = self.stage(&target, bytes)?;
        fs::rename(&staged, target).map_err(|error| {
            let _ = fs::remove_file(&staged);
            io_error("replace the file")(error)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TempDir;

    #[test]
    fn given_a_missing_key_when_read_then_none_is_returned() {
        let dir = TempDir::new();
        let storage = FileStorage::new(dir.path());

        assert_eq!(storage.read("absent.json"), Ok(None));
    }

    #[test]
    fn given_a_written_key_when_read_then_the_bytes_come_back() {
        let dir = TempDir::new();
        let storage = FileStorage::new(dir.path().join("nested"));

        storage.write("a.json", b"one").unwrap();

        assert_eq!(storage.read("a.json"), Ok(Some(b"one".to_vec())));
    }

    #[test]
    fn given_a_write_interrupted_after_the_temp_file_when_read_then_the_old_content_is_intact() {
        let dir = TempDir::new();
        let storage = FileStorage::new(dir.path());
        storage.write("a.json", b"old").unwrap();

        let target = storage.path_for("a.json").unwrap();
        storage.stage(&target, b"new but never renamed").unwrap();

        assert_eq!(storage.read("a.json"), Ok(Some(b"old".to_vec())));
    }

    #[test]
    fn given_keys_differing_only_in_extension_when_staged_then_temp_files_differ() {
        let dir = TempDir::new();
        let storage = FileStorage::new(dir.path());

        let json = storage.stage(&storage.path_for("a.json").unwrap(), b"j").unwrap();
        let txt = storage.stage(&storage.path_for("a.txt").unwrap(), b"t").unwrap();

        assert_ne!(json, txt);
        assert_eq!(fs::read(json).unwrap(), b"j");
    }

    #[test]
    fn given_a_failing_rename_when_written_then_the_temp_file_is_removed() {
        let dir = TempDir::new();
        let storage = FileStorage::new(dir.path());
        fs::create_dir(dir.path().join("a.json")).unwrap();

        let result = storage.write("a.json", b"new");

        assert!(result.is_err());
        let leftovers = fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".tmp"))
            .count();
        assert_eq!(leftovers, 0);
    }

    #[test]
    fn given_a_key_with_a_path_separator_when_written_then_it_is_refused() {
        let dir = TempDir::new();
        let storage = FileStorage::new(dir.path());

        assert!(storage.write("../escape", b"x").is_err());
    }
}
