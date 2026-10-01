use super::*;

#[test]
fn docs_pr_does_not_pay_postgres() {
    assert!(!live_postgres_required(
        CadenceEvent::PullRequest,
        &["docs/decisions/ADR-0719-eac-serving-control-north-star.md"]
    ));
    assert!(!live_postgres_required(
        CadenceEvent::MergeGroup,
        &["README.md", "docs/AGENTS.md"]
    ));
}

#[test]
fn adapter_change_pays_postgres_on_pr_and_queue() {
    let paths = ["iam/adapters/identity-scim-store-postgres/src/lib.rs"];
    assert!(live_postgres_required(CadenceEvent::PullRequest, &paths));
    assert!(live_postgres_required(CadenceEvent::MergeGroup, &paths));
}

#[test]
fn workflow_change_pays_postgres() {
    assert!(live_postgres_required(
        CadenceEvent::PullRequest,
        &[".github/workflows/live-postgres.yml"]
    ));
}

#[test]
fn provider_changes_pay_real_reindeer_qualification() {
    let path =
        "build/dependency-declarations/adapters/generation-reindeer/src/items/provider_source.rs";
    assert!(reindeer_source_qualification_required(
        CadenceEvent::PullRequest,
        &[path]
    ));
    assert!(reindeer_source_qualification_required(
        CadenceEvent::MergeGroup,
        &[path]
    ));
}

#[test]
fn qualification_inputs_pay_on_both_protected_events() {
    for path in reindeer_qualification_exact_paths() {
        for event in [CadenceEvent::PullRequest, CadenceEvent::MergeGroup] {
            assert!(
                presubmit_change_gates(event, [path]).reindeer_source_qualification(),
                "{event:?} omitted {path}"
            );
        }
    }
    assert!(!reindeer_source_qualification_required(
        CadenceEvent::PostsubmitPush,
        &["build/dependency-declarations/adapters/generation-reindeer/src/lib.rs"]
    ));
}

#[test]
fn postsubmit_and_dispatch_always_pay() {
    for event in [CadenceEvent::PostsubmitPush, CadenceEvent::WorkflowDispatch] {
        assert!(live_postgres_required(event, &[]));
        assert!(backbone_postgres_required(event, &[]));
        assert!(compute_lifecycle_postgres_required(event, &[]));
    }
}

#[test]
fn crates_cover_the_five_live_packages() {
    assert_eq!(LIVE_POSTGRES_CRATES.len(), 5);
}

#[test]
fn live_job_names_every_live_crate() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let y = std::fs::read_to_string(root.join(".github/workflows/live-postgres.yml"))
        .expect("live-postgres.yml");
    let manifest_path = "iam/adapters/identity-scim-store-postgres/Cargo.toml";
    let manifest = std::fs::read_to_string(root.join(manifest_path))
        .expect("SCIM store manifest")
        .parse::<toml::Value>()
        .expect("valid SCIM store manifest");
    let manifest_package = manifest["package"]["name"].as_str().expect("package name");
    for crate_name in LIVE_POSTGRES_CRATES {
        let selected = y.contains(&format!("-p {crate_name}"))
            || (*crate_name == manifest_package
                && y.contains(&format!("--manifest-path {manifest_path}")));
        assert!(selected, "live-postgres.yml missing crate {crate_name}");
    }
    assert!(y.contains("--no-tests=error"));
    assert!(y.contains("--run-ignored only"));
}

#[test]
fn one_path_traversal_produces_all_gate_outputs() {
    use std::cell::Cell;

    let visits = Cell::new(0);
    let paths = [
        "iam/adapters/identity-scim-store-postgres/src/lib.rs",
        "Cargo.lock",
        "docs/not-visited.md",
    ];
    let gates = presubmit_change_gates(
        CadenceEvent::PullRequest,
        paths.into_iter().inspect(|_| {
            visits.set(visits.get() + 1);
        }),
    );

    assert!(gates.backbone_postgres());
    assert!(gates.compute_lifecycle_postgres());
    assert!(gates.reindeer_source_qualification());
    assert_eq!(visits.get(), 2);
}
