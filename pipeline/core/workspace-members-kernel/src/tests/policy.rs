use crate::member_glob::is_windows_filesystem_loop_error_code;
use crate::{
    ResolveError, WorkspaceManifestEntries, member_entries_cover_dir, pattern_covers_dir,
    scan_member_dirs_from_str, segment_matches, workspace_manifest_entries_from_str,
};

use super::{fixture_root, root_manifest};

#[test]
fn segment_matches_anchors_both_ends() {
    assert!(segment_matches("oya-*", "oya-foo"));
    assert!(segment_matches("oya-*", "oya-"));
    assert!(!segment_matches("oya-*", "foo"));
    assert!(!segment_matches("oya-*", "xoya-foo"));
    assert!(!segment_matches("oya-*", "completions"));
    assert!(segment_matches("*", "anything"));
    assert!(segment_matches("*-app", "gate-app"));
    assert!(!segment_matches("*-app", "gate-lib"));
}

#[test]
fn manifest_entries_reader_returns_raw_members_and_excludes() {
    let manifest = root_manifest(
        &["libs/oya-*", "cloud/*/crates/oya-*"],
        &["cloud/cloud-kernel"],
    );
    let entries = workspace_manifest_entries_from_str(&manifest).expect("entries");
    assert_eq!(
        entries.members,
        vec!["libs/oya-*".to_owned(), "cloud/*/crates/oya-*".to_owned()]
    );
    assert_eq!(entries.exclude, vec!["cloud/cloud-kernel".to_owned()]);
}

#[test]
fn missing_workspace_table_is_a_shape_error() {
    let root = fixture_root();
    let error = scan_member_dirs_from_str("[package]\nname = \"x\"\n", &root)
        .expect_err("must reject non-workspace manifest");
    assert!(matches!(error, ResolveError::Shape(_)));
}

#[test]
fn malformed_root_manifest_is_a_parse_error() {
    let root = fixture_root();
    let error = scan_member_dirs_from_str("[workspace\nmembers = []\n", &root)
        .expect_err("must reject malformed root manifest");
    assert!(matches!(error, ResolveError::Parse(_)));
}

#[test]
fn malformed_exclude_is_a_shape_error() {
    let root = fixture_root();
    let error = scan_member_dirs_from_str(
        "[workspace]\nmembers = []\nexclude = \"libs/skip\"\n",
        &root,
    )
    .expect_err("must reject a non-array [workspace].exclude value");
    assert!(matches!(error, ResolveError::Shape(_)));
}

#[cfg(unix)]
#[test]
fn symlink_loops_are_skipped_like_cargo_for_glob_and_literal_members() {
    use std::os::unix::fs::symlink;

    let root = fixture_root();
    symlink("loop", root.join("loop")).unwrap();
    for member in ["*", "loop"] {
        let manifest =
            format!("[workspace]\nmembers = [\"{member}\"]\nexclude = []\nresolver = \"2\"\n");
        let scan = scan_member_dirs_from_str(&manifest, &root)
            .expect("Cargo skips a self-referential symlink member");
        assert!(scan.member_dirs.is_empty(), "member pattern: {member}");
        assert!(
            scan.missing_manifests.is_empty(),
            "member pattern: {member}"
        );
    }
}

#[test]
fn windows_filesystem_loop_error_code_is_recognized_without_windows_host() {
    assert!(is_windows_filesystem_loop_error_code(Some(1921)));
    assert!(!is_windows_filesystem_loop_error_code(Some(40)));
    assert!(!is_windows_filesystem_loop_error_code(None));
}

#[test]
fn expansion_read_dir_errors_fail_closed() {
    let root = fixture_root();
    let not_a_directory = root.join("not-a-directory");
    std::fs::write(&not_a_directory, "fixture").unwrap();

    let result = scan_member_dirs_from_str(
        "[workspace]\nmembers = [\"*\"]\nexclude = []\n",
        &not_a_directory,
    );
    assert!(
        matches!(&result, Err(ResolveError::InspectMemberPath { .. })),
        "a member directory read error must fail closed: {result:?}"
    );
}

#[test]
fn pattern_covers_dir_honors_normalized_component_globs() {
    assert!(pattern_covers_dir("libs/oya-*", "libs/oya-foo-kernel"));
    assert!(!pattern_covers_dir("libs/oya-*", "libs/foo-kernel"));
    assert!(!pattern_covers_dir("libs/*", "libs/group/nested-kernel"));
    assert!(pattern_covers_dir("messaging/*/*", "messaging/core/domain"));
    assert!(pattern_covers_dir("*/ports/*/src/..", "network/ports/blob"));
    assert!(pattern_covers_dir(
        "app/*/ports/**/src/..",
        "app/drive/ports/draft/blob"
    ));
    assert!(pattern_covers_dir(
        "app/*/ports/**/src/..",
        "app/drive/ports/blob"
    ));
    assert!(!pattern_covers_dir("../*/core/*", "network/core/route"));
}

#[test]
fn member_entries_cover_dir_applies_excludes() {
    let entries = WorkspaceManifestEntries {
        members: vec!["cloud/*/crates/*".to_owned()],
        exclude: vec!["cloud/cloud-kernel".to_owned()],
    };
    assert!(member_entries_cover_dir(
        &entries,
        "cloud/cloud-data/crates/data-kernel"
    ));
    assert!(!member_entries_cover_dir(
        &entries,
        "cloud/cloud-kernel/crates/kernel-frame-kernel"
    ));
    assert!(!member_entries_cover_dir(&entries, "libs/foo-kernel"));
}
