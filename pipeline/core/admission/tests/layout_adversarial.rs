//! Regression coverage for independently reviewed ADR-0719 D-8 bypasses.

use std::collections::BTreeSet;

use pipeline_admission::{
    APP_PRODUCT_DIRS, cargo_manifest_violations, changed_layout_violations,
    git_change_paths_from_name_status_z, layout_violations,
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
            .any(|item| item.contains("base") && item.contains("core crate"))
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
            .any(|item| item.contains("app/ledger") && item.contains("core crate"))
    );

    let implementation_without_law = git_change_paths_from_name_status_z(
        b"A\0app/ledger/OWNERS\0A\0app/ledger/core/posting/Cargo.toml\0A\0app/ledger/core/posting/src/lib.rs\0",
    )
    .unwrap();
    assert!(
        changed_layout_violations(&implementation_without_law, &BTreeSet::new())
            .iter()
            .any(|item| item.contains("D-36") && item.contains("ADR.md"))
    );

    let implementation = git_change_paths_from_name_status_z(
        b"A\0app/ledger/OWNERS\0A\0app/ledger/ADR.md\0A\0app/ledger/PRD.md\0A\0app/ledger/SPEC.md\0A\0app/ledger/PLAN.md\0A\0app/ledger/core/posting/Cargo.toml\0A\0app/ledger/core/posting/src/lib.rs\0",
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
        "network/ports/blob-draft/Cargo.toml",
        "network/adapters/blob-s3-draft/Cargo.toml",
        "network/facade/edge/Cargo.toml",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    for path in [
        "network/core/query-engine/Cargo.toml",
        "network/ports/blob/Cargo.toml",
        "network/adapters/blob-s3/Cargo.toml",
        "network/ports/draft/blob/Cargo.toml",
        "network/adapters/draft/blob-s3/Cargo.toml",
        "network/facade/app/Cargo.toml",
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
        "pipeline/core/admission/src/main.rs",
        "pipeline/core/admission/src/bin/helper.rs",
        "pipeline/core/admission/tests/tasks/example.rs",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    for path in [
        "pipeline/core/admission/src/items/quota.rs",
        "pipeline/core/admission/src/domain/value.rs",
        "pipeline/core/admission/build.rs",
        "network/facade/edge-app/src/main.rs",
        "network/facade/edge-app/src/lib.rs",
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
    assert!(rejected(
        "network/facade/proto/iam/edge/v1/edge_service.proto"
    ));
    assert!(!rejected(
        "network/facade/proto/network/edge/v1/edge_service.proto"
    ));
}

#[test]
fn owner_docs_and_app_meta_do_not_reintroduce_global_law() {
    assert!(rejected("network/docs/design/plan/todo.md"));
    assert!(rejected("network/docs/runbooks/tasks/todo.md"));
    assert!(rejected("network/docs/design/shadow/Cargo.toml"));
    assert!(rejected("network/docs/design/shadow/src/lib.rs"));
    assert!(rejected("network/docs/runbooks/example/build.rs"));
    assert!(!rejected("network/docs/design/routing.md"));
    for path in ["app/ADR.md", "app/PRD.md", "app/SPEC.md", "app/PLAN.md"] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    assert!(!rejected("app/OWNERS"));
    assert!(!rejected("app/README.md"));
}

#[test]
fn cedar_and_iac_descendants_are_closed_payloads() {
    for path in [
        "network/cedar/tasks/todo.cedar",
        "network/cedar/policy.json",
        "network/iac/plan/todo.textproto",
        "network/iac/helm/values.yaml",
        "network/iac/state.yaml",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    for path in [
        "network/cedar/policies.cedar",
        "network/cedar/network_schema.cedarschema",
        "network/iac/network_state.proto",
        "network/iac/cells/us_east.textproto",
    ] {
        assert!(!rejected(path), "unexpected rejection: {path}");
    }
}

#[test]
fn observability_accepts_only_direct_generated_openslo_outputs() {
    for path in [
        "network/observability/slos/dashboard.json",
        "network/observability/slos/availability.openslo.yaml",
        "network/observability/slos/dashboards/availability.generated.openslo.yaml",
        "network/observability/slos/availability.generated.json",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    assert!(!rejected(
        "network/observability/slos/availability.generated.openslo.yaml"
    ));
}

#[test]
fn sold_proto_and_owner_docs_reject_draft_and_law_dumps() {
    for path in [
        "network/facade/proto/network/draft/v1/service.proto",
        "network/facade/proto/network/blob_draft/v1/service.proto",
        "network/docs/design/decisions/ADR-copy.md",
        "network/docs/design/PRD-copy.md",
        "network/docs/design/SPEC-copy.md",
        "network/docs/design/PLAN-copy.md",
        "network/docs/design/specs/foo.md",
        "network/docs/design/IPs/note.md",
        "network/docs/concepts/ADR-copy.md",
        "network/docs/runbooks/catalog.yaml",
        "network/docs/design/scorecards/weekly.md",
        "docs/ports/draft/blob/Cargo.toml",
        "templates/adapters/draft/blob-s3/src/lib.rs",
    ] {
        assert!(rejected(path), "expected rejection: {path}");
    }
    assert!(!rejected(
        "network/docs/design/cache/invalidation_strategy.md"
    ));
}

#[test]
fn changed_manifests_bind_package_and_rustc_identity() {
    let path = "network/ports/blob/Cargo.toml";
    assert!(cargo_manifest_violations(path, "[package]\nname = 'network-blob'\n").is_empty());
    assert!(!cargo_manifest_violations(path, "[package]\nname = 'other'\n").is_empty());
    assert!(
        !cargo_manifest_violations(
            path,
            "[package]\nname = 'network-blob'\n[lib]\nname = 'alias'\n"
        )
        .is_empty()
    );
    assert!(
        cargo_manifest_violations(
            "network/ports/draft/blob/Cargo.toml",
            "[package]\nname = 'network-blob-draft'\n"
        )
        .is_empty()
    );
}
