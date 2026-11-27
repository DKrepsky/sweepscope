//! An AppImage starts the program inside its own `usr` folder, which breaks the caller's relative paths (ADR 0024
//! item 7).
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The folder to return to, when an AppImage moved the program away from the caller's folder.
pub fn appimage_caller_dir(env: &dyn Fn(&str) -> Option<OsString>, exe: &Path, cwd: &Path) -> Option<PathBuf> {
    // `PWD`, not `OWD`: extract-and-run sets no `OWD` and lets a stale one through.
    let absolute = |name: &str| env(name).map(PathBuf::from).filter(|path| path.is_absolute());
    let app_dir = absolute("APPDIR")?.canonicalize().ok()?;
    // Both must be inside: an installed program can inherit `APPDIR` from another AppImage.
    let inside = |path: &Path| path.canonicalize().is_ok_and(|real| real.starts_with(&app_dir));
    if inside(exe) && inside(cwd) {
        absolute("PWD")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    use sweepscope_kernel::testing::TempDir;

    use super::*;

    const CALLER: &str = "/home/u/work";

    struct Mounted {
        root: TempDir,
        app_dir: PathBuf,
        exe: PathBuf,
        cwd: PathBuf,
    }

    fn program_in(folder: &Path) -> (PathBuf, PathBuf) {
        let cwd = folder.join("usr");
        let exe = cwd.join("bin").join("sweepscope");
        fs::create_dir_all(cwd.join("bin")).unwrap();
        fs::write(&exe, b"").unwrap();
        (exe, cwd)
    }

    fn mounted() -> Mounted {
        let root = TempDir::new();
        let app_dir = root.path().join("app");
        let (exe, cwd) = program_in(&app_dir);
        Mounted {
            root,
            app_dir,
            exe,
            cwd,
        }
    }

    fn caller_dir(app_dir: Option<&Path>, pwd: Option<&str>, exe: &Path, cwd: &Path) -> Option<PathBuf> {
        let env = |name: &str| match name {
            "APPDIR" => app_dir.map(|path| path.as_os_str().to_owned()),
            "PWD" => pwd.map(OsString::from),
            _ => None,
        };
        appimage_caller_dir(&env, exe, cwd)
    }

    #[test]
    fn given_the_program_and_its_folder_inside_appdir_when_asked_then_it_is_the_pwd_folder() {
        let image = mounted();

        let folder = caller_dir(Some(&image.app_dir), Some(CALLER), &image.exe, &image.cwd);

        assert_eq!(folder, Some(PathBuf::from(CALLER)));
    }

    #[test]
    fn given_appdir_reached_through_a_link_when_asked_then_it_is_the_pwd_folder() {
        let image = mounted();
        let link = image.root.path().join("link");
        symlink(&image.app_dir, &link).unwrap();

        let folder = caller_dir(Some(&link), Some(CALLER), &image.exe, &image.cwd);

        assert_eq!(folder, Some(PathBuf::from(CALLER)));
    }

    #[test]
    fn given_no_appdir_when_asked_then_there_is_no_folder() {
        let image = mounted();

        assert_eq!(caller_dir(None, Some(CALLER), &image.exe, &image.cwd), None);
    }

    #[test]
    fn given_an_empty_appdir_when_asked_then_there_is_no_folder() {
        let image = mounted();

        let folder = caller_dir(Some(Path::new("")), Some(CALLER), &image.exe, &image.cwd);

        assert_eq!(folder, None);
    }

    #[test]
    fn given_a_relative_appdir_when_asked_then_there_is_no_folder() {
        // `.` would resolve to the test's working directory, which holds both paths.
        let here = std::env::current_dir().unwrap();

        let folder = caller_dir(Some(Path::new(".")), Some(CALLER), &here.join("Cargo.toml"), &here);

        assert_eq!(folder, None);
    }

    #[test]
    fn given_an_installed_program_with_an_inherited_appdir_when_asked_then_there_is_no_folder() {
        let image = mounted();
        let (installed, _) = program_in(&image.root.path().join("installed"));

        let folder = caller_dir(Some(&image.app_dir), Some(CALLER), &installed, &image.cwd);

        assert_eq!(folder, None);
    }

    #[test]
    fn given_a_working_folder_outside_appdir_when_asked_then_there_is_no_folder() {
        let image = mounted();

        let folder = caller_dir(Some(&image.app_dir), Some(CALLER), &image.exe, image.root.path());

        assert_eq!(folder, None);
    }

    #[test]
    fn given_a_sibling_folder_with_the_same_name_prefix_when_asked_then_there_is_no_folder() {
        let image = mounted();
        let (exe, cwd) = program_in(&image.root.path().join("app-other"));

        let folder = caller_dir(Some(&image.app_dir), Some(CALLER), &exe, &cwd);

        assert_eq!(folder, None);
    }

    #[test]
    fn given_no_pwd_when_asked_then_there_is_no_folder() {
        let image = mounted();

        let folder = caller_dir(Some(&image.app_dir), None, &image.exe, &image.cwd);

        assert_eq!(folder, None);
    }

    #[test]
    fn given_a_relative_pwd_when_asked_then_there_is_no_folder() {
        let image = mounted();

        let folder = caller_dir(Some(&image.app_dir), Some("work"), &image.exe, &image.cwd);

        assert_eq!(folder, None);
    }

    /// `0xff` never occurs in UTF-8.
    fn non_utf8(name: &str) -> OsString {
        OsString::from_vec([name.as_bytes(), &[0xff]].concat())
    }

    #[test]
    fn given_a_pwd_and_an_appdir_that_are_not_utf8_when_asked_then_the_pwd_folder_is_returned_unchanged() {
        let root = TempDir::new();
        let app_dir = root.path().join(non_utf8("app"));
        let (exe, cwd) = program_in(&app_dir);
        let pwd = non_utf8("/home/u/work");
        let env = |name: &str| match name {
            "APPDIR" => Some(app_dir.clone().into_os_string()),
            "PWD" => Some(pwd.clone()),
            _ => None,
        };

        let folder = appimage_caller_dir(&env, &exe, &cwd);

        assert_eq!(folder, Some(PathBuf::from(&pwd)));
    }
}
