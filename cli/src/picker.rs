use std::path::PathBuf;

use sweepscope_kernel::{FileFilter, FilePicker, KernelError};

/// Answers `LoadSetup`'s open dialog with the path from the command line; there is no save dialog.
pub struct SetupPath(pub PathBuf);

impl FilePicker for SetupPath {
    fn pick_save(&self, _suggested_name: &str, _filter: &FileFilter) -> Result<Option<PathBuf>, KernelError> {
        Err(KernelError::fatal("the command line writes no files through a dialog"))
    }

    fn pick_open(&self, _filter: &FileFilter) -> Result<Option<PathBuf>, KernelError> {
        Ok(Some(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter() -> FileFilter {
        FileFilter {
            label: "any".into(),
            extensions: vec![],
        }
    }

    #[test]
    fn given_a_setup_path_when_the_open_dialog_is_asked_then_that_path_is_returned() {
        let picker = SetupPath("s.json".into());

        assert_eq!(picker.pick_open(&filter()), Ok(Some(PathBuf::from("s.json"))));
    }

    #[test]
    fn given_a_setup_path_when_the_save_dialog_is_asked_then_it_is_refused() {
        let picker = SetupPath("s.json".into());

        assert!(picker.pick_save("x.json", &filter()).is_err());
    }
}
