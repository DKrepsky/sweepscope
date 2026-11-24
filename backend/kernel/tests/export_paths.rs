//! Generated TypeScript must land in `frontend/src/api/generated`, never outside the repository.
//! ts-rs resolves `export_to` against the cwd of whichever crate runs the export (a dependency is
//! exported from its dependants' tests too), so no depth is right for every crate. One absolute
//! export dir, set in `.cargo/config.toml`, is; `export_to` must not come back.
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target" && n != "node_modules") {
                rust_files(&path, found);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn given_the_cargo_config_when_ts_rs_picks_its_export_dir_then_it_is_the_generated_folder_inside_the_repo() {
    let export_dir = PathBuf::from(env!("TS_RS_EXPORT_DIR"));

    assert_eq!(export_dir, repo_root().join("frontend/src/api/generated"));
}

#[test]
fn given_the_backend_sources_when_scanned_then_no_type_overrides_the_export_location() {
    let mut files = Vec::new();
    rust_files(&repo_root().join("backend"), &mut files);
    let this_file = Path::new(file!()).file_name().unwrap();

    let offenders: Vec<_> = files
        .iter()
        .filter(|f| f.file_name() != Some(this_file))
        .filter(|f| fs::read_to_string(f).unwrap().contains("export_to"))
        .collect();

    assert!(files.len() > 10, "the scan found no sources: the guard is blind");
    assert!(
        offenders.is_empty(),
        "export_to would bypass the export dir: {offenders:?}"
    );
}
