use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::files::{RecordFiles, RecordWriter};

/// Production files. The picker has already confirmed an overwrite, so an existing file is truncated.
pub struct OsRecordFiles;

impl RecordFiles for OsRecordFiles {
    fn create(&self, path: &Path) -> io::Result<Box<dyn RecordWriter>> {
        let file = File::create(path)?;
        Ok(Box::new(OsRecordWriter {
            file,
            path: path.to_owned(),
        }))
    }
}

struct OsRecordWriter {
    file: File,
    path: PathBuf,
}

impl Write for OsRecordWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl RecordWriter for OsRecordWriter {
    fn sync(&mut self) -> io::Result<()> {
        self.file.sync_data()
    }

    /// Writing to an unlinked file succeeds silently on Linux, so the open file is compared with the path.
    fn still_linked(&self) -> bool {
        same_file(&self.file, &self.path)
    }
}

#[cfg(unix)]
fn same_file(file: &File, path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (file.metadata(), std::fs::metadata(path)) {
        (Ok(open), Ok(linked)) => open.dev() == linked.dev() && open.ino() == linked.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn same_file(_file: &File, path: &Path) -> bool {
    path.exists()
}
