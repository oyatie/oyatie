//! Cargo differential for bounded source-marker workspace globs.

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
fn bounded_source_markers_match_cargo_and_tolerate_zero_drafts() {
    let root = fixture_root();
    write(
        &root,
        "Cargo.toml",
        "[workspace]\nmembers=[\"*/ports/*/src/..\",\"*/adapters/*/src/..\",\"*/facade/*/src/..\",\"*/ports/draft/*/src/..\",\"*/adapters/draft/*/src/..\",\"app/*/ports/*/src/..\",\"app/*/adapters/*/src/..\",\"app/*/ports/draft/*/src/..\",\"app/*/adapters/draft/*/src/..\",\"app/*/facade/*/src/..\"]\nexclude=[\"*/ports/draft/*\",\"*/adapters/draft/*\",\"app/*/ports/draft/*\",\"app/*/adapters/draft/*\"]\nresolver='2'\n",
    );
    write_crate(&root, "network/ports/blob", "network-blob");
    write_crate(&root, "network/ports/draft/blob", "network-blob-draft");
    write_crate(&root, "network/adapters/blob-s3", "network-blob-s3");
    write_crate(
        &root,
        "network/adapters/draft/blob-s3",
        "network-blob-s3-draft",
    );
    write_crate(&root, "network/facade/edge-app", "network-edge-app");
    write(
        &root,
        "network/facade/proto/network/edge/v1/service.proto",
        "syntax = \"proto3\";\n",
    );
    write_crate(&root, "app/drive/ports/blob", "drive-blob");
    write_crate(&root, "app/drive/ports/draft/blob", "drive-blob-draft");
    write_crate(&root, "app/drive/adapters/blob-s3", "drive-blob-s3");
    write_crate(
        &root,
        "app/drive/adapters/draft/blob-s3",
        "drive-blob-s3-draft",
    );
    write_crate(&root, "app/drive/facade/api-app", "drive-api-app");
    write(
        &root,
        "app/drive/facade/proto/drive/api/v1/service.proto",
        "syntax = \"proto3\";\n",
    );
    for nested_source in [
        "network/ports/blob/tests/fixture/src/lib.rs",
        "network/ports/blob/src/nested/src/lib.rs",
        "network/ports/draft/blob/tests/fixture/src/lib.rs",
        "network/adapters/blob-s3/tests/fixture/src/lib.rs",
        "network/adapters/draft/blob-s3/src/nested/src/lib.rs",
        "app/drive/ports/blob/tests/fixture/src/lib.rs",
        "app/drive/ports/draft/blob/src/nested/src/lib.rs",
        "app/drive/adapters/blob-s3/tests/fixture/src/lib.rs",
        "app/drive/adapters/draft/blob-s3/src/nested/src/lib.rs",
    ] {
        write(&root, nested_source, "pub fn nested_marker() {}\n");
    }

    let expected = BTreeSet::from([
        "app/drive/adapters/blob-s3".to_owned(),
        "app/drive/adapters/draft/blob-s3".to_owned(),
        "app/drive/facade/api-app".to_owned(),
        "app/drive/ports/blob".to_owned(),
        "app/drive/ports/draft/blob".to_owned(),
        "network/adapters/blob-s3".to_owned(),
        "network/adapters/draft/blob-s3".to_owned(),
        "network/facade/edge-app".to_owned(),
        "network/ports/blob".to_owned(),
        "network/ports/draft/blob".to_owned(),
    ]);
    let cargo = cargo_member_dirs(&root);
    let owned = resolve_member_dirs(&root)
        .expect("owned resolver")
        .into_iter()
        .collect::<BTreeSet<_>>();

    assert_eq!(cargo, expected);
    assert_eq!(owned, expected);
    assert_eq!(owned, cargo);

    std::fs::remove_dir_all(root.join("network/ports/draft")).expect("sell capability draft");
    std::fs::remove_dir_all(root.join("network/adapters/draft"))
        .expect("sell capability adapter draft");
    std::fs::remove_dir_all(root.join("app/drive/ports/draft")).expect("sell app draft");
    std::fs::remove_dir_all(root.join("app/drive/adapters/draft")).expect("sell app adapter draft");
    let sold_expected = expected
        .into_iter()
        .filter(|member| !member.contains("/draft/"))
        .collect::<BTreeSet<_>>();
    let sold_owned = resolve_member_dirs(&root)
        .expect("owned resolver after final draft is sold")
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert_eq!(sold_owned, sold_expected);
    assert_eq!(sold_owned, cargo_member_dirs(&root));
    let _ = std::fs::remove_dir_all(root);
}
