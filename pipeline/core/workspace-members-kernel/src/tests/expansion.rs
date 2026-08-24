use crate::{ResolveError, resolve_member_dirs_from_str, scan_member_dirs_from_str};

use super::{fixture_root, make_crate, root_manifest};

#[test]
fn star_matches_one_component_and_honors_non_crate_excludes() {
    let root = fixture_root();
    make_crate(&root, "libs/a-kernel");
    make_crate(&root, "libs/b-kernel");
    std::fs::create_dir_all(root.join("libs/not-a-crate")).unwrap();
    make_crate(&root, "libs/group/nested-kernel");

    let manifest = root_manifest(&["libs/*"], &["libs/not-a-crate", "libs/group"]);
    let resolved = resolve_member_dirs_from_str(&manifest, &root).expect("resolve");

    assert_eq!(
        resolved,
        vec!["libs/a-kernel".to_string(), "libs/b-kernel".to_string()]
    );
}

#[test]
fn diagnostic_scan_reports_every_unexcluded_missing_manifest() {
    let root = fixture_root();
    std::fs::create_dir_all(root.join("comms/messenger/chaos"))
        .expect("create non-crate member match");
    std::fs::create_dir_all(root.join("comms/messenger/resilience"))
        .expect("create second non-crate member match");
    std::fs::create_dir_all(root.join("comms/messenger/fixtures"))
        .expect("create excluded non-crate member match");

    let manifest = root_manifest(&["comms/*/*"], &["comms/messenger/fixtures"]);
    let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan diagnostics");
    assert!(scan.member_dirs.is_empty());
    assert_eq!(
        scan.missing_manifests,
        vec![
            "comms/messenger/chaos".to_owned(),
            "comms/messenger/resilience".to_owned(),
        ]
    );

    assert_eq!(
        resolve_member_dirs_from_str(&manifest, &root),
        Err(ResolveError::MissingManifests(vec![
            "comms/messenger/chaos".to_owned(),
            "comms/messenger/resilience".to_owned(),
        ]))
    );
}

#[test]
fn explicit_exclude_suppresses_non_manifest_glob_match() {
    let root = fixture_root();
    std::fs::create_dir_all(root.join("comms/messenger/chaos"))
        .expect("create excluded non-crate member match");

    let manifest = root_manifest(&["comms/*/*"], &["comms/messenger/chaos"]);
    assert!(
        resolve_member_dirs_from_str(&manifest, &root)
            .expect("resolve")
            .is_empty()
    );
}

#[cfg(unix)]
#[test]
fn wildcard_includes_directory_symlink_like_cargo() {
    use std::os::unix::fs::symlink;

    let root = fixture_root();
    make_crate(&root, "real");
    std::fs::create_dir_all(root.join("libs")).unwrap();
    symlink("../real", root.join("libs/link")).unwrap();

    let manifest = root_manifest(&["libs/*"], &[]);
    assert_eq!(
        resolve_member_dirs_from_str(&manifest, &root).expect("resolve symlink"),
        vec!["libs/link".to_owned()]
    );
}

#[cfg(unix)]
#[test]
fn wildcard_reports_directory_symlink_missing_manifest_like_cargo() {
    use std::os::unix::fs::symlink;

    let root = fixture_root();
    std::fs::create_dir_all(root.join("real")).unwrap();
    std::fs::create_dir_all(root.join("libs")).unwrap();
    symlink("../real", root.join("libs/link")).unwrap();

    let manifest = root_manifest(&["libs/*"], &[]);
    let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan symlink");
    assert!(scan.member_dirs.is_empty());
    assert_eq!(scan.missing_manifests, vec!["libs/link".to_owned()]);
}

#[cfg(unix)]
#[test]
fn wildcard_exclude_precedes_directory_symlink_manifest_check() {
    use std::os::unix::fs::symlink;

    let root = fixture_root();
    std::fs::create_dir_all(root.join("real")).unwrap();
    std::fs::create_dir_all(root.join("libs")).unwrap();
    symlink("../real", root.join("libs/link")).unwrap();

    let manifest = root_manifest(&["libs/*"], &["libs/link"]);
    let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan excluded symlink");
    assert!(scan.member_dirs.is_empty());
    assert!(scan.missing_manifests.is_empty());
}

#[cfg(unix)]
#[test]
fn wildcard_skips_dangling_symlink_like_cargo() {
    use std::os::unix::fs::symlink;

    let root = fixture_root();
    std::fs::create_dir_all(root.join("libs")).unwrap();
    symlink("../missing", root.join("libs/link")).unwrap();

    let manifest = root_manifest(&["libs/*"], &[]);
    let scan = scan_member_dirs_from_str(&manifest, &root).expect("scan dangling symlink");
    assert!(scan.member_dirs.is_empty());
    assert!(scan.missing_manifests.is_empty());
}

#[test]
fn exclude_drops_a_nested_workspace_subtree() {
    let root = fixture_root();
    make_crate(&root, "cloud/cloud-data/crates/data-kernel");
    make_crate(&root, "cloud/cloud-kernel/crates/kernel-frame-kernel");
    make_crate(&root, "cloud/cloud-kernel/crates/kernel-hal-kernel");

    let manifest = root_manifest(&["cloud/*/crates/*"], &["cloud/cloud-kernel"]);
    assert_eq!(
        resolve_member_dirs_from_str(&manifest, &root).expect("resolve"),
        vec!["cloud/cloud-data/crates/data-kernel".to_string()]
    );
}

#[test]
fn literal_member_path_is_supported_alongside_globs() {
    let root = fixture_root();
    make_crate(&root, "tools/one-cli");
    make_crate(&root, "libs/x-kernel");

    let manifest = root_manifest(&["libs/*", "tools/one-cli"], &[]);
    assert_eq!(
        resolve_member_dirs_from_str(&manifest, &root).expect("resolve"),
        vec!["libs/x-kernel".to_string(), "tools/one-cli".to_string()]
    );
}

#[test]
fn partial_segment_glob_filters_non_crate_siblings() {
    let root = fixture_root();
    make_crate(&root, "tools/oya-one-cli");
    make_crate(&root, "tools/oya-two-cli");
    make_crate(&root, "tools/one-cli");
    std::fs::create_dir_all(root.join("tools/hooks")).unwrap();
    std::fs::write(root.join("tools/hooks/run.sh"), "#!/bin/sh\n").unwrap();
    std::fs::create_dir_all(root.join("tools/completions/bash")).unwrap();

    let manifest = root_manifest(&["tools/oya-*"], &[]);
    assert_eq!(
        resolve_member_dirs_from_str(&manifest, &root).expect("resolve"),
        vec![
            "tools/oya-one-cli".to_string(),
            "tools/oya-two-cli".to_string()
        ]
    );
}
