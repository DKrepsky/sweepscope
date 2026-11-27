//! Where the desktop app keeps its files, and a view of them the CLI can never change (ADR 0022 item 4).
use std::path::PathBuf;

use sweepscope_kernel::{FileStorage, KernelError, Storage};

/// Must equal `identifier` in `src-tauri/tauri.conf.json`; a test keeps the two together.
pub const APP_FOLDER: &str = "io.github.dkrepsky.sweepscope";

/// `--config-dir`, else `$XDG_CONFIG_HOME/<app>`, else `$HOME/.config/<app>`.
pub fn config_dir(explicit: Option<PathBuf>, env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let non_empty = |name: &str| env(name).filter(|value| !value.is_empty());
    explicit.or_else(|| {
        non_empty("XDG_CONFIG_HOME")
            .map(|home| PathBuf::from(home).join(APP_FOLDER))
            .or_else(|| non_empty("HOME").map(|home| PathBuf::from(home).join(".config").join(APP_FOLDER)))
    })
}

/// Reads like the app's storage; writes are accepted and dropped, so a script never overwrites `last-setup.json`,
/// the settings or the calibration.
pub struct ReadOnlyStorage {
    inner: FileStorage,
}

impl ReadOnlyStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            inner: FileStorage::new(root),
        }
    }
}

impl Storage for ReadOnlyStorage {
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>, KernelError> {
        self.inner.read(key)
    }

    fn write(&self, _key: &str, _bytes: &[u8]) -> Result<(), KernelError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use sweepscope_kernel::testing::TempDir;

    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        }
    }

    #[test]
    fn given_an_explicit_folder_when_resolved_then_it_wins_over_the_environment() {
        let folder = config_dir(Some("/x".into()), env(&[("XDG_CONFIG_HOME", "/xdg")]));

        assert_eq!(folder, Some(PathBuf::from("/x")));
    }

    #[test]
    fn given_xdg_config_home_when_resolved_then_the_app_folder_is_inside_it() {
        let folder = config_dir(None, env(&[("XDG_CONFIG_HOME", "/xdg"), ("HOME", "/home/u")]));

        assert_eq!(folder, Some(PathBuf::from("/xdg").join(APP_FOLDER)));
    }

    #[test]
    fn given_only_home_when_resolved_then_the_app_folder_is_under_dot_config() {
        let folder = config_dir(None, env(&[("XDG_CONFIG_HOME", ""), ("HOME", "/home/u")]));

        assert_eq!(folder, Some(PathBuf::from("/home/u/.config").join(APP_FOLDER)));
    }

    #[test]
    fn given_no_environment_when_resolved_then_there_is_no_folder() {
        assert_eq!(config_dir(None, env(&[])), None);
    }

    #[test]
    fn given_the_app_folder_constant_when_compared_with_the_tauri_identifier_then_they_are_equal() {
        let config = include_str!("../../src-tauri/tauri.conf.json");

        assert!(config.contains(&format!("\"identifier\": \"{APP_FOLDER}\"")));
    }

    #[test]
    fn given_a_stored_file_when_read_through_the_view_then_its_bytes_come_back() {
        let dir = TempDir::new();
        fs::write(dir.path().join("a.json"), b"one").unwrap();

        let storage = ReadOnlyStorage::new(dir.path());

        assert_eq!(storage.read("a.json"), Ok(Some(b"one".to_vec())));
    }

    #[test]
    fn given_a_write_through_the_view_when_the_folder_is_listed_then_nothing_changed() {
        let dir = TempDir::new();
        fs::write(dir.path().join("a.json"), b"one").unwrap();
        let storage = ReadOnlyStorage::new(dir.path());

        storage.write("a.json", b"two").unwrap();
        storage.write("b.json", b"new").unwrap();

        assert_eq!(fs::read(dir.path().join("a.json")).unwrap(), b"one");
        assert!(!dir.path().join("b.json").exists());
    }
}
