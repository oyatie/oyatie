//! OWNERS and CODEOWNERS occupancy. Integration tests: they read the tree
//! through the public occupant map.

use pipeline_admission::{
    ALLOWED_ROOT_DIRS, BUILD_ROOT_DIRS, META_ROOTS, ROOT_OCCUPANT, is_capability_root,
    owners_occupant,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn owners_body(path: &Path) -> String {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    text.trim().to_owned()
}

fn walk_owners(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("entry");
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "target"
            || name == ".git"
            || name.starts_with('.') && path.is_dir() && name != ".github"
        {
            continue;
        }
        let ft = entry.file_type().expect("ft");
        if ft.is_dir() {
            walk_owners(&path, out);
        } else if name == "OWNERS" {
            out.push(path);
        }
    }
}

fn app_products() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for entry in std::fs::read_dir(repo_root().join("app")).expect("app/") {
        let entry = entry.expect("entry");
        if !entry.file_type().expect("ft").is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        out.insert(name);
    }
    out
}

fn capabilities() -> BTreeSet<&'static str> {
    ALLOWED_ROOT_DIRS
        .iter()
        .chain(BUILD_ROOT_DIRS)
        .copied()
        .filter(|d| is_capability_root(d))
        .filter(|capability| repo_root().join(capability).is_dir())
        .collect()
}

#[test]
fn every_owners_file_is_the_path_occupant() {
    let root = repo_root();
    let mut files = Vec::new();
    walk_owners(&root, &mut files);
    assert!(!files.is_empty(), "expected OWNERS files");
    let mut mismatches = Vec::new();
    for path in &files {
        let rel = path
            .strip_prefix(&root)
            .expect("under root")
            .to_string_lossy()
            .replace('\\', "/");
        let Some(want) = owners_occupant(&rel) else {
            mismatches.push(format!("{rel}: no occupant"));
            continue;
        };
        let got = owners_body(path);
        if got != want {
            mismatches.push(format!("{rel}: want {want:?} got {got:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn required_owners_files_exist() {
    let root = repo_root();
    let mut missing = Vec::new();
    let mut require = |rel: String| {
        if !root.join(&rel).is_file() {
            missing.push(rel);
        }
    };
    require("OWNERS".into());
    require("app/OWNERS".into());
    for meta in META_ROOTS {
        if *meta == "app" {
            continue;
        }
        require(format!("{meta}/OWNERS"));
    }
    for cap in capabilities() {
        require(format!("{cap}/OWNERS"));
    }
    for product in app_products() {
        require(format!("app/{product}/OWNERS"));
    }
    assert!(missing.is_empty(), "missing OWNERS: {missing:?}");
}

fn codeowners_rules(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut bits = line.split_whitespace();
        let pattern = bits.next().expect("pattern").to_owned();
        let owners: Vec<&str> = bits.collect();
        assert_eq!(
            owners.len(),
            1,
            "CODEOWNERS {pattern}: want one occupant, got {owners:?}"
        );
        out.insert(pattern, owners[0].to_owned());
    }
    out
}

#[test]
fn codeowners_is_the_github_adapter_of_occupants() {
    let text = std::fs::read_to_string(repo_root().join(".github/CODEOWNERS")).expect("CODEOWNERS");
    let got = codeowners_rules(&text);

    let mut want = BTreeMap::new();
    want.insert("*".into(), "@jason931225".into());
    want.insert(".github/workflows/".into(), "@oyatie/pipeline".into());
    for cap in capabilities() {
        want.insert(format!("{cap}/"), format!("@oyatie/{cap}"));
    }
    for product in app_products() {
        want.insert(format!("app/{product}/"), format!("@oyatie/{product}"));
    }

    assert_eq!(got, want);
    assert_eq!(got.get("*").map(String::as_str), Some("@jason931225"));
    assert_eq!(
        owners_occupant("app/OWNERS").as_deref(),
        Some(ROOT_OCCUPANT)
    );
}
