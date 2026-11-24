//! Generated TypeScript must land in `frontend/src/api/generated`, never outside the repository.
//! ts-rs resolves `export_to` against `<crate>/bindings`, so the number of `..` depends on the crate depth.
use std::fs;
use std::path::{Component, Path, PathBuf};

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

fn crate_dir_of(file: &Path) -> PathBuf {
    file.ancestors()
        .find(|dir| dir.join("Cargo.toml").is_file())
        .expect("a source file inside a crate")
        .to_path_buf()
}

fn export_targets(source: &str) -> Vec<&str> {
    source
        .split("export_to = \"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect()
}

fn normalise(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

#[test]
fn given_every_export_to_in_the_backend_when_resolved_then_it_is_the_generated_folder_inside_the_repo() {
    let root = repo_root();
    let expected = root.join("frontend/src/api/generated");
    let mut files = Vec::new();
    rust_files(&root.join("backend"), &mut files);

    let mut checked = 0;
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        for target in export_targets(&source) {
            let resolved = normalise(&crate_dir_of(&file).join("bindings").join(target));
            assert_eq!(resolved, expected, "{} exports to {target}", file.display());
            checked += 1;
        }
    }
    assert!(checked > 0, "no export_to found: the guard is blind");
}
