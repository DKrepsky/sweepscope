//! The files of one run: `capture-NNNN.csv|bin`, never overwriting what is already there.
use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use sweepscope_export::{file_extension, raw_layout_text, ExportFormat};

use crate::exit::CliError;

const LAYOUT_FILE: &str = "capture.format.txt";

pub struct FrameFiles {
    dir: PathBuf,
    format: ExportFormat,
    count: u32,
    created_dir: Cell<bool>,
}

impl FrameFiles {
    pub fn new(dir: PathBuf, format: ExportFormat, count: u32) -> Self {
        Self {
            dir,
            format,
            count,
            created_dir: Cell::new(false),
        }
    }

    pub fn file_name(&self, number: u32) -> String {
        format!("capture-{number:04}.{}", file_extension(self.format))
    }

    /// Creates the folder and refuses the run when any of its files exists, before the scope is touched.
    pub fn prepare(&self) -> Result<(), CliError> {
        self.created_dir.set(!self.dir.exists());
        fs::create_dir_all(&self.dir).map_err(|error| CliError::io("create", &self.dir.to_string_lossy(), &error))?;
        let taken = (1..=self.count)
            .map(|number| self.file_name(number))
            .find(|name| self.dir.join(name).symlink_metadata().is_ok());
        match taken {
            Some(name) => Err(CliError::failure(format!("{name} already exists, nothing was written"))),
            None => Ok(()),
        }
    }

    /// After a failed run: removes the folder this run created when nothing was written into it.
    pub fn remove_if_created_and_empty(&self) {
        if self.created_dir.get() {
            let _ = fs::remove_dir(&self.dir);
        }
    }

    /// Writes frame `number` (from 1); the first raw frame also leaves the layout document beside it.
    pub fn write(&self, number: u32, bytes: &[u8]) -> Result<PathBuf, CliError> {
        let name = self.file_name(number);
        let path = self.dir.join(&name);
        create_new(&path, &name, bytes)?;
        if number == 1 && self.format == ExportFormat::Raw {
            self.write_layout()?;
        }
        Ok(path)
    }

    fn write_layout(&self) -> Result<(), CliError> {
        let path = self.dir.join(LAYOUT_FILE);
        if path.symlink_metadata().is_ok() {
            return Ok(());
        }
        create_new(&path, LAYOUT_FILE, raw_layout_text().as_bytes())
    }
}

fn create_new(path: &Path, name: &str, bytes: &[u8]) -> Result<(), CliError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| match error.kind() {
            ErrorKind::AlreadyExists => CliError::failure(format!("{name} already exists, it was left untouched")),
            _ => CliError::io("create", name, &error),
        })?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| CliError::io("write", name, &error))
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::testing::TempDir;

    use super::*;
    use crate::exit::ExitStatus;

    fn files(dir: &TempDir, format: ExportFormat, count: u32) -> FrameFiles {
        FrameFiles::new(dir.path().to_path_buf(), format, count)
    }

    #[test]
    fn given_a_frame_number_when_named_then_it_has_four_digits_and_the_format_extension() {
        let dir = TempDir::new();

        assert_eq!(files(&dir, ExportFormat::Csv, 1).file_name(7), "capture-0007.csv");
        assert_eq!(files(&dir, ExportFormat::Raw, 1).file_name(12), "capture-0012.bin");
    }

    #[test]
    fn given_a_missing_folder_when_prepared_then_it_is_created() {
        let dir = TempDir::new();
        let out = dir.path().join("a").join("b");

        FrameFiles::new(out.clone(), ExportFormat::Csv, 1).prepare().unwrap();

        assert!(out.is_dir());
    }

    #[test]
    fn given_a_folder_created_by_the_run_when_nothing_was_written_then_it_is_removed() {
        let dir = TempDir::new();
        let out = dir.path().join("out");
        let target = FrameFiles::new(out.clone(), ExportFormat::Csv, 1);
        target.prepare().unwrap();

        target.remove_if_created_and_empty();

        assert!(!out.exists());
    }

    #[test]
    fn given_a_folder_created_by_the_run_when_a_frame_was_written_then_it_stays() {
        let dir = TempDir::new();
        let out = dir.path().join("out");
        let target = FrameFiles::new(out.clone(), ExportFormat::Csv, 1);
        target.prepare().unwrap();
        target.write(1, b"x").unwrap();

        target.remove_if_created_and_empty();

        assert!(out.join("capture-0001.csv").is_file());
    }

    #[test]
    fn given_an_existing_empty_folder_when_the_run_fails_then_it_is_kept() {
        let dir = TempDir::new();
        let target = files(&dir, ExportFormat::Csv, 1);
        target.prepare().unwrap();

        target.remove_if_created_and_empty();

        assert!(dir.path().is_dir());
    }

    #[test]
    fn given_a_later_target_that_exists_when_prepared_then_the_run_is_refused_and_the_file_is_untouched() {
        let dir = TempDir::new();
        fs::write(dir.path().join("capture-0002.csv"), b"mine").unwrap();

        let error = files(&dir, ExportFormat::Csv, 3).prepare().unwrap_err();

        assert_eq!(error.status, ExitStatus::Failure);
        assert!(error.message.starts_with("capture-0002.csv already exists"), "{error}");
        assert_eq!(fs::read(dir.path().join("capture-0002.csv")).unwrap(), b"mine");
    }

    #[test]
    fn given_a_file_created_after_preparing_when_written_then_it_is_not_overwritten() {
        let dir = TempDir::new();
        let target = files(&dir, ExportFormat::Csv, 1);
        target.prepare().unwrap();
        fs::write(dir.path().join("capture-0001.csv"), b"late").unwrap();

        let error = target.write(1, b"new").unwrap_err();

        assert_eq!(error.status, ExitStatus::Failure);
        assert_eq!(fs::read(dir.path().join("capture-0001.csv")).unwrap(), b"late");
    }

    #[test]
    fn given_csv_frames_when_written_then_only_the_frame_files_exist() {
        let dir = TempDir::new();
        let target = files(&dir, ExportFormat::Csv, 2);

        target.write(1, b"a").unwrap();
        target.write(2, b"b").unwrap();

        let mut names: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        names.sort();
        assert_eq!(names, ["capture-0001.csv", "capture-0002.csv"]);
    }

    #[test]
    fn given_raw_frames_when_written_then_the_layout_document_is_written_once() {
        let dir = TempDir::new();
        let target = files(&dir, ExportFormat::Raw, 2);

        target.write(1, b"a").unwrap();
        target.write(2, b"b").unwrap();

        let layout = fs::read_to_string(dir.path().join(LAYOUT_FILE)).unwrap();
        assert!(layout.starts_with("A SweepScope raw capture"), "{layout}");
    }

    #[cfg(unix)]
    #[test]
    fn given_a_read_only_folder_when_a_frame_is_written_then_the_status_is_permission_denied() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new();
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o555)).unwrap();
        let writable_anyway = fs::File::create(dir.path().join("probe")).is_ok();

        let result = files(&dir, ExportFormat::Csv, 1).write(1, b"a");

        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o755)).unwrap();
        if writable_anyway {
            return; // running as root: the permission cannot be denied
        }
        assert_eq!(result.unwrap_err().status, ExitStatus::PermissionDenied);
    }
}
