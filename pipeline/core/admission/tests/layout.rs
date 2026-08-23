//! Tree shape against the public layout engine. Lives in `tests/` because it
//! reads the repo, not because it needs a private API.

use pipeline_admission::ALLOWED_ROOT_DIRS;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn unknown_root_dir_is_red() {
    let allowed: BTreeSet<&str> = ALLOWED_ROOT_DIRS.iter().copied().collect();
    let mut unknown = Vec::new();
    for entry in std::fs::read_dir(repo_root()).expect("read root") {
        let entry = entry.expect("entry");
        if !entry.file_type().expect("ft").is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        if !allowed.contains(name.as_ref()) {
            unknown.push(name.into_owned());
        }
    }
    assert!(
        unknown.is_empty(),
        "unknown root names (not in ALLOWED_ROOT_DIRS): {unknown:?}"
    );
}
