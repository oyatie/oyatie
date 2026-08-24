//! Cargo differential for source-marker workspace globs containing a literal parent segment.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use workspace_members_kernel::resolve_member_dirs;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "workspace-members-parent-segments-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("fixture path has parent"))
        .expect("create fixture parent");
    std::fs::write(path, contents).expect("write fixture");
}

fn write_crate(root: &Path, relative: &str, name: &str) {
    write(
        root,
        &format!("{relative}/Cargo.toml"),
        &format!("[package]\nname='{name}'\nversion='0.1.0'\nedition='2021'\n"),
    );
    write(
        root,
        &format!("{relative}/src/lib.rs"),
        "pub fn marker() {}\n",
    );
}

fn cargo_member_dirs(root: &Path) -> BTreeSet<String> {
    let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(root)
        .output()
        .expect("spawn Cargo metadata");
    assert!(
        output.status.success(),
        "Cargo metadata must accept parent-segment globs: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("metadata JSON");
    let workspace_root =
        PathBuf::from(metadata["workspace_root"].as_str().expect("workspace root"));
    metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .map(|package| {
            PathBuf::from(package["manifest_path"].as_str().expect("manifest path"))
                .parent()
                .expect("manifest parent")
                .strip_prefix(&workspace_root)
                .expect("workspace package beneath root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn source_marker_parent_segments_match_cargo_for_capability_and_app_drafts() {
    let root = fixture_root();
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers=[\"*/ports/draft/*/src/..\",\"app/*/ports/**/src/..\"]\nresolver='2'\n",
    );
    write_crate(&root, "network/ports/draft/blob", "network-blob-draft");
    write_crate(&root, "app/drive/ports/blob", "drive-blob");
    write_crate(&root, "app/drive/ports/draft/blob", "drive-blob-draft");

    let expected = BTreeSet::from([
        "app/drive/ports/blob".to_owned(),
        "app/drive/ports/draft/blob".to_owned(),
        "network/ports/draft/blob".to_owned(),
    ]);
    let owned = resolve_member_dirs(&root)
        .expect("owned resolver")
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(owned, expected);
    assert_eq!(owned, cargo_member_dirs(&root));
    let _ = std::fs::remove_dir_all(root);
}
