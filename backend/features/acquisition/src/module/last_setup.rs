//! `last-setup.json` in the application storage (ADR 0016 item 6). Failures to write are not worth interrupting
//! the user: the next change tries again.
use std::sync::Arc;

use sweepscope_kernel::Storage;

use crate::domain::setup_file::SetupFile;

const KEY: &str = "last-setup.json";
const CORRUPT_KEY: &str = "last-setup.json.corrupt";

pub enum Stored {
    Nothing,
    Setup(SetupFile),
    Unreadable(String),
}

pub struct LastSetup {
    storage: Arc<dyn Storage>,
}

impl LastSetup {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// An unreadable file is kept as `.corrupt` (the first one stays) so the next save does not destroy it.
    pub fn load(&self) -> Stored {
        let bytes = match self.storage.read(KEY) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Stored::Nothing,
            Err(error) => return Stored::Unreadable(error.message),
        };
        match SetupFile::parse(&bytes) {
            Ok(file) => Stored::Setup(file),
            Err(error) => {
                self.keep_corrupt(&bytes);
                Stored::Unreadable(error.message)
            }
        }
    }

    pub fn save(&self, file: &SetupFile) {
        let _ = self.storage.write(KEY, &file.to_bytes());
    }

    fn keep_corrupt(&self, bytes: &[u8]) {
        if matches!(self.storage.read(CORRUPT_KEY), Ok(None)) {
            let _ = self.storage.write(CORRUPT_KEY, bytes);
        }
    }
}
