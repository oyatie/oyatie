//! Regression coverage for independently reviewed ADR-0719 D-8 bypasses.

use std::collections::BTreeSet;

use pipeline_admission::{
    APP_PRODUCT_DIRS, changed_layout_violations, git_change_paths_from_name_status_z,
    layout_violations,
};

fn rejected(path: &str) -> bool {
    !layout_violations(&[path.to_owned()]).is_empty()
}

#[test]
fn conditional_base_requires_real_core_content() {
    assert!(!rejected("base/core/bytes/src/lib.rs"));

    let paperwork =
        git_change_paths_from_name_status_z(b"A\0base/OWNERS\0A\0base/README.md\0").unwrap();
    assert!(
        changed_layout_violations(&paperwork, &BTreeSet::new())
            .iter()
            .any(|item| item.contains("base") && item.contains("core source"))
    );

    let implementation = git_change_paths_from_name_status_z(
        b"A\0base/OWNERS\0A\0base/core/bytes/Cargo.toml\0A\0base/core/bytes/src/lib.rs\0",
    )
    .unwrap();
    assert!(changed_layout_violations(&implementation, &BTreeSet::new()).is_empty());
}

#[test]
fn app_roster_is_closed_and_missing_products_cannot_be_scaffolds() {
    assert!(APP_PRODUCT_DIRS.contains(&"ledger"));
    assert!(!APP_PRODUCT_DIRS.contains(&"social"));
    assert!(rejected("app/social/OWNERS"));
    assert!(!rejected("app/ledger/OWNERS"));

    let paperwork =
        git_change_paths_from_name_status_z(b"A\0app/ledger/OWNERS\0A\0app/ledger/README.md\0")
            .unwrap();
    assert!(
        changed_layout_violations(&paperwork, &BTreeSet::new())
            .iter()
            .any(|item| item.contains("app/ledger") && item.contains("core source"))
    );

    let implementation = git_change_paths_from_name_status_z(
        b"A\0app/ledger/OWNERS\0A\0app/ledger/core/posting/Cargo.toml\0A\0app/ledger/core/posting/src/lib.rs\0",
    )
    .unwrap();
    assert!(changed_layout_violations(&implementation, &BTreeSet::new()).is_empty());
}

#[test]
fn crate_leaves_follow_the_face_grammar() {
    for path in [
        "network/core/bad_name/Cargo.toml",
        "network/core/cloud-cache/Cargo.toml",
        "network/adapters/sqlite/Cargo.toml",
        "network/facade/edge/Cargo.toml",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    for path in [
        "network/core/query-engine/Cargo.toml",
        "network/ports/blob/Cargo.toml",
        "network/adapters/blob-s3/Cargo.toml",
        "network/facade/edge-app/Cargo.toml",
    ] {
        assert!(!rejected(path), "unexpected rejection: {path}");
    }
}

#[test]
fn crate_trees_reject_nested_dumps_and_non_rust_files() {
    for path in [
        "pipeline/core/admission/src/plan/note.rs",
        "pipeline/core/admission/src/BadName.rs",
        "pipeline/core/admission/src/README.md",
        "pipeline/core/admission/tests/tasks/example.rs",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    for path in [
        "pipeline/core/admission/src/items/quota.rs",
        "pipeline/core/admission/src/domain/value.rs",
        "pipeline/core/admission/build.rs",
        "network/facade/edge-app/tests/e2e/main.rs",
    ] {
        assert!(!rejected(path), "unexpected rejection: {path}");
    }
}

#[test]
fn proto_paths_reject_package_and_filename_shortcuts() {
    assert!(rejected("network/facade/proto/network/edge/v1/v1.proto"));
    assert!(rejected(
        "network/facade/proto/Network/edge/v1/edge_service.proto"
    ));
    assert!(!rejected(
        "network/facade/proto/network/edge/v1/edge_service.proto"
    ));
}
