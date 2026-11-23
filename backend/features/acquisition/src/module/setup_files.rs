//! The two file requests of the setup: pick a file, then read or write it. The dialog may stay open for a long time,
//! so no lock of the module is held while it is.
use std::path::Path;
use std::sync::Arc;

use sweepscope_kernel::contracts::SavedFile;
use sweepscope_kernel::{read_bounded, write_atomic, FileFilter, FilePicker, KernelError};

use super::shared::Shared;
use crate::domain::setup_file::SetupFile;
use crate::SetupLoaded;

const SUGGESTED_NAME: &str = "sweepscope-setup.json";
const MAX_SETUP_BYTES: u64 = 1024 * 1024;

pub struct SetupFiles {
    picker: Arc<dyn FilePicker>,
}

impl SetupFiles {
    pub fn new(picker: Arc<dyn FilePicker>) -> Self {
        Self { picker }
    }

    pub fn save(&self, shared: &Shared) -> Result<Option<SavedFile>, KernelError> {
        let bytes = shared.read(|core| core.setup_bytes())?;
        let Some(path) = self.picker.pick_save(SUGGESTED_NAME, &filter())? else {
            return Ok(None);
        };
        write_atomic(&path, &bytes)?;
        Ok(Some(SavedFile {
            file_name: file_name(&path),
        }))
    }

    pub fn load(&self, shared: &Shared) -> Result<Option<SetupLoaded>, KernelError> {
        shared.read(|core| core.ensure_files_available())?;
        let Some(path) = self.picker.pick_open(&filter())? else {
            return Ok(None);
        };
        let file = SetupFile::parse(&read_bounded(&path, MAX_SETUP_BYTES)?)?;
        let mut report = None;
        shared.act(|core, fx, now| {
            report = Some(core.load_setup(&file, fx, now)?);
            Ok(())
        })?;
        Ok(report.map(|report| SetupLoaded {
            file_name: file_name(&path),
            warnings: report.warnings,
        }))
    }
}

fn filter() -> FileFilter {
    FileFilter {
        label: "SweepScope setup".into(),
        extensions: vec!["json".into()],
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}
