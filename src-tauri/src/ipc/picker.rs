use std::path::PathBuf;

use sweepscope_kernel::{FileFilter, FilePicker, KernelError};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};

const MAIN_WINDOW: &str = "main";

/// The native dialogs, opened from Rust only: the webview never names a path (ADR 0016 item 2).
/// Dialogs block, so callers run on a blocking task and never on the main thread.
pub struct DialogPicker {
    app: AppHandle,
}

impl DialogPicker {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn dialog(&self, filter: &FileFilter) -> FileDialogBuilder<tauri::Wry> {
        let extensions = extensions_of(filter);
        let dialog = self.app.dialog().file().add_filter(&filter.label, &extensions);
        match self.app.get_webview_window(MAIN_WINDOW) {
            Some(window) => dialog.set_parent(&window),
            None => dialog,
        }
    }
}

fn extensions_of(filter: &FileFilter) -> Vec<&str> {
    filter.extensions.iter().map(String::as_str).collect()
}

fn into_path(chosen: Option<FilePath>) -> Result<Option<PathBuf>, KernelError> {
    chosen
        .map(|file| {
            file.into_path()
                .map_err(|_| KernelError::recoverable("the chosen location is not a file on this computer"))
        })
        .transpose()
}

impl FilePicker for DialogPicker {
    fn pick_save(&self, suggested_name: &str, filter: &FileFilter) -> Result<Option<PathBuf>, KernelError> {
        into_path(self.dialog(filter).set_file_name(suggested_name).blocking_save_file())
    }

    fn pick_open(&self, filter: &FileFilter) -> Result<Option<PathBuf>, KernelError> {
        into_path(self.dialog(filter).blocking_pick_file())
    }
}

#[cfg(test)]
mod tests {
    use sweepscope_kernel::ErrorCategory;

    use super::*;

    #[test]
    fn given_a_filter_when_its_extensions_are_listed_then_they_keep_their_order() {
        let filter = FileFilter {
            label: "CSV".into(),
            extensions: vec!["csv".into(), "txt".into()],
        };

        assert_eq!(extensions_of(&filter), ["csv", "txt"]);
    }

    #[test]
    fn given_a_cancelled_dialog_when_converted_then_it_is_none() {
        assert_eq!(into_path(None), Ok(None));
    }

    #[test]
    fn given_a_chosen_file_when_converted_then_it_is_that_path() {
        let chosen = FilePath::Path(PathBuf::from("a.csv"));

        assert_eq!(into_path(Some(chosen)), Ok(Some(PathBuf::from("a.csv"))));
    }

    #[test]
    fn given_a_chosen_url_that_is_not_a_file_when_converted_then_the_error_is_recoverable_without_a_path() {
        let url = "https://example.com/a.csv".parse().expect("url");

        let error = into_path(Some(FilePath::Url(url))).expect_err("not a file");

        assert_eq!(error.category, ErrorCategory::Recoverable);
        assert!(!error.message.contains('/'));
    }
}
