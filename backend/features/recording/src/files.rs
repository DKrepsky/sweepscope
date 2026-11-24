use std::io::{self, Write};
use std::path::Path;

/// Opens the record file; tests pass a fake that fails like a full disk.
pub trait RecordFiles: Send + Sync {
    fn create(&self, path: &Path) -> io::Result<Box<dyn RecordWriter>>;
}

pub trait RecordWriter: Write + Send {
    fn sync(&mut self) -> io::Result<()>;
    /// False when the file was removed or replaced while it was open.
    fn still_linked(&self) -> bool;
}
