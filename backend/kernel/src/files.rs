//! Files chosen by the user: the picker trait, atomic writes and bounded reads (ADR 0016 item 1).
//! Error texts name the file, never the folder.
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

use crate::KernelError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    pub label: String,
    pub extensions: Vec<String>,
}

/// Opens the native dialogs. `None` means the user cancelled.
pub trait FilePicker: Send + Sync {
    fn pick_save(&self, suggested_name: &str, filter: &FileFilter) -> Result<Option<PathBuf>, KernelError>;
    fn pick_open(&self, filter: &FileFilter) -> Result<Option<PathBuf>, KernelError>;
}

/// Writes to a temp file in the same folder, syncs, renames over `path`; the temp file never outlives a failure.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), KernelError> {
    write_atomic_with(path, bytes, |from, to| fs::rename(from, to))
}

/// Reads the whole file unless it is larger than `max_bytes`; a larger file is not read.
pub fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, KernelError> {
    let fail = |cause: &str| KernelError::recoverable(format!("cannot read {}: {cause}", file_name(path)));
    let io_fail = |error: io::Error| fail(&cause_of(&error, "the file does not exist"));
    let mut file = File::open(path).map_err(io_fail)?;
    let length = file.metadata().map_err(io_fail)?.len();
    if length > max_bytes {
        return Err(fail("file is too large"));
    }
    let mut bytes = Vec::with_capacity(usize::try_from(length).unwrap_or(0));
    // The limit also guards against a file that grew after the size check.
    (&mut file)
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(io_fail)?;
    if bytes.len() as u64 > max_bytes {
        return Err(fail("file is too large"));
    }
    Ok(bytes)
}

fn write_atomic_with(
    path: &Path,
    bytes: &[u8],
    rename: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<(), KernelError> {
    let fail = |error: &io::Error| {
        KernelError::recoverable(format!(
            "cannot write {}: {}",
            file_name(path),
            cause_of(error, "the folder does not exist")
        ))
    };
    if path.is_dir() {
        return Err(fail(&io::Error::from(ErrorKind::IsADirectory)));
    }
    let staged = staging_path(path);
    let result = stage(&staged, bytes).and_then(|()| rename(&staged, path));
    if let Err(error) = result {
        let _ = fs::remove_file(&staged);
        return Err(fail(&error));
    }
    Ok(())
}

fn stage(staged: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(staged)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn staging_path(target: &Path) -> PathBuf {
    let mut name = OsString::from(".");
    name.push(target.file_name().unwrap_or_default());
    name.push(format!(".{}.tmp", std::process::id()));
    target.with_file_name(name)
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| "the file".to_owned(), |name| name.to_string_lossy().into_owned())
}

fn cause_of(error: &io::Error, not_found: &str) -> String {
    match error.kind() {
        ErrorKind::PermissionDenied => "permission denied".to_owned(),
        ErrorKind::StorageFull => "no space left on the device".to_owned(),
        ErrorKind::NotFound => not_found.to_owned(),
        ErrorKind::IsADirectory => "the path is a folder".to_owned(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TempDir;

    fn leftovers(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn given_a_new_file_when_written_atomically_then_it_holds_the_bytes_and_no_temp_remains() {
        let dir = TempDir::new();
        let path = dir.path().join("out.csv");

        write_atomic(&path, b"abc").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"abc");
        assert_eq!(leftovers(dir.path()), ["out.csv"]);
    }

    #[test]
    fn given_an_existing_file_when_written_atomically_then_it_is_replaced() {
        let dir = TempDir::new();
        let path = dir.path().join("out.csv");
        fs::write(&path, b"old").unwrap();

        write_atomic(&path, b"new").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new");
    }

    #[test]
    fn given_a_missing_folder_when_written_atomically_then_the_error_names_the_file_and_the_cause() {
        let dir = TempDir::new();
        let path = dir.path().join("gone").join("out.csv");

        let error = write_atomic(&path, b"abc").unwrap_err();

        assert_eq!(error.message, "cannot write out.csv: the folder does not exist");
    }

    #[test]
    fn given_a_path_that_is_a_folder_when_written_atomically_then_the_error_says_so() {
        let dir = TempDir::new();

        let error = write_atomic(dir.path(), b"abc").unwrap_err();

        assert!(error.message.ends_with("the path is a folder"), "{}", error.message);
    }

    #[cfg(unix)]
    #[test]
    fn given_a_read_only_folder_when_written_atomically_then_permission_is_denied_and_no_temp_remains() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
        let writable_anyway = File::create(dir.path().join("probe")).is_ok();

        let result = write_atomic(&dir.path().join("out.csv"), b"abc");

        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        if writable_anyway {
            return; // running as root: the permission cannot be denied
        }
        assert_eq!(result.unwrap_err().message, "cannot write out.csv: permission denied");
        assert!(leftovers(dir.path()).is_empty());
    }

    #[test]
    fn given_a_rename_that_fails_when_written_atomically_then_the_temp_is_removed_and_the_old_file_is_intact() {
        let dir = TempDir::new();
        let path = dir.path().join("out.csv");
        fs::write(&path, b"old").unwrap();

        let result = write_atomic_with(&path, b"new", |_, _| Err(io::Error::from(ErrorKind::StorageFull)));

        assert_eq!(
            result.unwrap_err().message,
            "cannot write out.csv: no space left on the device"
        );
        assert_eq!(fs::read(&path).unwrap(), b"old");
        assert_eq!(leftovers(dir.path()), ["out.csv"]);
    }

    #[test]
    fn given_a_file_within_the_limit_when_read_bounded_then_the_bytes_are_returned() {
        let dir = TempDir::new();
        let path = dir.path().join("in.json");
        fs::write(&path, b"12345").unwrap();

        assert_eq!(read_bounded(&path, 5).unwrap(), b"12345");
    }

    #[test]
    fn given_a_file_over_the_limit_when_read_bounded_then_it_is_refused_as_too_large() {
        let dir = TempDir::new();
        let path = dir.path().join("big.json");
        fs::write(&path, vec![0_u8; 1024]).unwrap();

        let error = read_bounded(&path, 1023).unwrap_err();

        assert_eq!(error.message, "cannot read big.json: file is too large");
    }

    #[test]
    fn given_a_missing_file_when_read_bounded_then_the_error_names_the_file() {
        let dir = TempDir::new();

        let error = read_bounded(&dir.path().join("nope.json"), 10).unwrap_err();

        assert_eq!(error.message, "cannot read nope.json: the file does not exist");
    }
}
