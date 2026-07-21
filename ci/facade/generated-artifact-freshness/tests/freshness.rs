#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use ci_generated_artifact_freshness::{
    FACE_REMEDIATION_COMMAND, FACE_SETTLE_PROTOCOL, Finding, FindingCode, LockPackage,
    MemberPackage, check_repo_with_regenerated_faces, evaluate_face_determinism,
    evaluate_face_freshness, evaluate_lock_freshness, parse_lock_packages,
    parse_member_package_manifest, read_decommitted_face_names, render_findings,
    render_remediation,
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn codes(findings: &[Finding]) -> BTreeSet<FindingCode> {
    findings.iter().map(|finding| finding.code).collect()
}

fn fixture_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-freshness-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

#[test]
fn lock_freshness_is_green_when_member_packages_match_sourceless_lock_entries() {
    let members = vec![
        MemberPackage::new("libs/oya-alpha-kernel", "oya-alpha-kernel", "0.1.0"),
        MemberPackage::new("tools/oya-beta-cli", "oya-beta-cli", "0.1.0"),
    ];
    let lock_packages = vec![
        LockPackage::path("oya-alpha-kernel", "0.1.0"),
        LockPackage::path("oya-beta-cli", "0.1.0"),
        LockPackage::external("serde", "1.0.0"),
    ];

    assert!(evaluate_lock_freshness(&members, &lock_packages).is_empty());
}

#[test]
fn lock_freshness_reports_missing_member_package() {
    let members = vec![MemberPackage::new(
        "libs/oya-missing-kernel",
        "oya-missing-kernel",
        "0.1.0",
    )];
    let lock_packages = vec![];

    let findings = evaluate_lock_freshness(&members, &lock_packages);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::LockMissingMemberPackage])
    );
    assert_eq!(findings[0].key, "libs/oya-missing-kernel");
}

#[test]
fn lock_freshness_reports_stale_member_version() {
    let members = vec![MemberPackage::new(
        "libs/oya-stale-kernel",
        "oya-stale-kernel",
        "0.2.0",
    )];
    let lock_packages = vec![LockPackage::path("oya-stale-kernel", "0.1.0")];

    let findings = evaluate_lock_freshness(&members, &lock_packages);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::LockStaleMemberVersion])
    );
    assert!(findings[0].detail.contains("0.1.0"));
    assert!(findings[0].detail.contains("0.2.0"));
}

#[test]
fn lock_freshness_reports_orphan_sourceless_lock_package() {
    let members = vec![MemberPackage::new(
        "libs/oya-live-kernel",
        "oya-live-kernel",
        "0.1.0",
    )];
    let lock_packages = vec![
        LockPackage::path("oya-live-kernel", "0.1.0"),
        LockPackage::path("oya-orphan-kernel", "0.1.0"),
    ];

    let findings = evaluate_lock_freshness(&members, &lock_packages);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::LockOrphanPathPackage])
    );
    assert_eq!(findings[0].key, "oya-orphan-kernel");
}

#[test]
fn lock_parser_keeps_only_package_name_version_and_source_presence() {
    let lock = r#"
version = 4

[[package]]
name = "oya-alpha-kernel"
version = "0.1.0"

[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    let packages = parse_lock_packages(lock).expect("parse lock packages");

    assert_eq!(
        packages,
        vec![
            LockPackage::path("oya-alpha-kernel", "0.1.0"),
            LockPackage::external("serde", "1.0.0"),
        ]
    );
}

#[test]
fn face_freshness_is_green_when_committed_and_regenerated_bytes_match() {
    let committed = vec![("scm-facts.generated.json".to_owned(), "{ }\n".to_owned())];
    let regenerated = vec![("scm-facts.generated.json".to_owned(), "{ }\n".to_owned())];

    assert!(evaluate_face_freshness(&committed, &regenerated, &BTreeSet::new()).is_empty());
}

#[test]
fn face_freshness_reports_stale_generated_face() {
    let committed = vec![("scm-facts.generated.json".to_owned(), "old\n".to_owned())];
    let regenerated = vec![("scm-facts.generated.json".to_owned(), "new\n".to_owned())];

    let findings = evaluate_face_freshness(&committed, &regenerated, &BTreeSet::new());

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::GeneratedFaceStale])
    );
    assert_eq!(findings[0].key, "scm-facts.generated.json");
    assert!(findings[0].detail.contains("commit content changes first"));
    assert!(
        findings[0]
            .detail
            .contains("faces regenerate from the TRACKED TREE STATE")
    );
    assert!(
        findings[0]
            .detail
            .contains("never mix content and regenerated faces in one commit")
    );
    assert!(
        findings[0]
            .detail
            .contains("commit only PR-owned generated face diffs")
    );
}

#[test]
fn decommit_class_face_is_green_without_a_committed_copy() {
    // ADR-0595: a de-commit-class face has no committed copy on disk but regenerates fine.
    // The old "uncommitted generated face" stale clause must NOT fire for it.
    let committed: Vec<(String, String)> = Vec::new();
    let regenerated = vec![("ttl-policy.generated.json".to_owned(), "fresh\n".to_owned())];
    let decommitted = BTreeSet::from(["ttl-policy.generated.json".to_owned()]);

    assert!(evaluate_face_freshness(&committed, &regenerated, &decommitted).is_empty());
}

#[test]
fn decommit_class_face_does_not_require_byte_parity_when_a_stale_copy_lingers() {
    // A lingering on-disk copy (e.g. a previous materialization) must not trigger byte parity
    // for a de-commit-class face: the source of truth is the regeneration, not the disk bytes.
    let committed = vec![(
        "ttl-policy.generated.json".to_owned(),
        "lingering\n".to_owned(),
    )];
    let regenerated = vec![("ttl-policy.generated.json".to_owned(), "fresh\n".to_owned())];
    let decommitted = BTreeSet::from(["ttl-policy.generated.json".to_owned()]);

    assert!(evaluate_face_freshness(&committed, &regenerated, &decommitted).is_empty());
}

#[test]
fn controller_owned_main_materialized_face_does_not_require_pr_byte_parity() {
    // The gate-baseline face must remain committed on the integration branch for merge-base
    // ratchet consumers, but contributor PRs must not carry generated baseline byte churn.
    let committed = vec![(
        "gate-baseline.generated.json".to_owned(),
        "old\n".to_owned(),
    )];
    let regenerated = vec![(
        "gate-baseline.generated.json".to_owned(),
        "fresh\n".to_owned(),
    )];
    let non_pr_owned = BTreeSet::from(["gate-baseline.generated.json".to_owned()]);

    assert!(evaluate_face_freshness(&committed, &regenerated, &non_pr_owned).is_empty());
}

#[test]
fn decommit_class_face_is_stale_when_regeneration_stops_producing_it() {
    // A producer that silently stops emitting a declared de-commit-class face must RED — it has
    // no committed copy to fall back on, so the gate is its only guard.
    let committed: Vec<(String, String)> = Vec::new();
    let regenerated: Vec<(String, String)> = Vec::new();
    let decommitted = BTreeSet::from(["ttl-policy.generated.json".to_owned()]);

    let findings = evaluate_face_freshness(&committed, &regenerated, &decommitted);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::GeneratedFaceStale])
    );
    assert_eq!(findings[0].key, "ttl-policy.generated.json");
    assert!(
        findings[0]
            .detail
            .contains("was not produced by regeneration")
    );
}

#[test]
fn committed_class_face_keeps_byte_parity_when_other_faces_are_decommitted() {
    // Scope guard: de-committing one face must NOT weaken byte parity for a still-committed face.
    let committed = vec![(
        "example-committed-face.generated.json".to_owned(),
        "old\n".to_owned(),
    )];
    let regenerated = vec![(
        "example-committed-face.generated.json".to_owned(),
        "new\n".to_owned(),
    )];
    let decommitted = BTreeSet::from(["ttl-policy.generated.json".to_owned()]);

    let findings = evaluate_face_freshness(&committed, &regenerated, &decommitted);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::GeneratedFaceStale])
    );
    assert_eq!(findings[0].key, "example-committed-face.generated.json");
    assert!(findings[0].detail.contains("differ from regenerated"));
}

#[test]
fn decommit_exemption_matches_canonical_path_not_basename() {
    // Security guard: the non-PR-owned exemption must key on the CANONICAL FULL PATH, never the
    // basename. A deceptive manifest row at a NON-canonical path that merely shares a basename with
    // a still-PR-owned face must NOT retire that committed face's byte-parity. Legitimate canonical
    // rows are the positive controls.
    let root = fixture_root();
    std::fs::create_dir_all(root.join("registry")).expect("create registry dir");
    let manifest = r#"{
  "artifacts": [
    {
      "path": "foo/scm-facts.generated.json",
      "materialization_mode": "not-tracked-in-git"
    },
    {
      "path": "ci/facade/artifact-inventory-registry/ttl-policy.generated.json",
      "materialization_mode": "not-tracked-in-git"
    },
    {
      "path": "ci/facade/artifact-inventory-registry/gate-baseline.generated.json",
      "materialization_mode": "main-branch-materialized"
    }
  ]
}"#;
    std::fs::write(
        root.join("registry/generated-artifact-control-plane.json"),
        manifest,
    )
    .expect("write control-plane manifest");

    let names = read_decommitted_face_names(&root);

    // The deceptive non-canonical row must NOT exempt the real, still-committed scm-facts face:
    // its basename must be absent so byte-parity continues to guard the committed face.
    assert!(
        !names.contains("scm-facts.generated.json"),
        "non-canonical basename-colliding row must not retire the committed scm-facts byte-parity"
    );
    // Positive control: a legitimately de-committed canonical face IS exempted.
    assert!(
        names.contains("ttl-policy.generated.json"),
        "a canonical-path de-commit row must be exempted by its basename"
    );
    assert!(
        names.contains("gate-baseline.generated.json"),
        "a canonical-path controller-owned baseline row must be exempted by its basename"
    );
    assert_eq!(names.len(), 2, "only canonical-path rows may be exempted");
}

#[test]
fn determinism_canary_is_green_for_byte_identical_regenerations() {
    let first = vec![("ttl-policy.generated.json".to_owned(), "fresh\n".to_owned())];
    let second = vec![("ttl-policy.generated.json".to_owned(), "fresh\n".to_owned())];
    let decommitted = BTreeSet::from(["ttl-policy.generated.json".to_owned()]);

    assert!(evaluate_face_determinism(&first, &second, &decommitted).is_empty());
}

#[test]
fn determinism_canary_reds_for_nondeterministic_regeneration() {
    let first = vec![("ttl-policy.generated.json".to_owned(), "run-a\n".to_owned())];
    let second = vec![("ttl-policy.generated.json".to_owned(), "run-b\n".to_owned())];
    let decommitted = BTreeSet::from(["ttl-policy.generated.json".to_owned()]);

    let findings = evaluate_face_determinism(&first, &second, &decommitted);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::GeneratedFaceStale])
    );
    assert_eq!(findings[0].key, "ttl-policy.generated.json");
    assert!(findings[0].detail.contains("not deterministic"));
}

#[test]
fn determinism_canary_detects_masterplan_drift_when_product_graph_is_stable() {
    let first = vec![
        (
            "masterplan.generated.json".to_owned(),
            "first plan\n".to_owned(),
        ),
        ("product-graph.html".to_owned(), "stable graph\n".to_owned()),
    ];
    let second = vec![
        (
            "masterplan.generated.json".to_owned(),
            "second plan\n".to_owned(),
        ),
        ("product-graph.html".to_owned(), "stable graph\n".to_owned()),
    ];
    let decommitted = BTreeSet::from([
        "masterplan.generated.json".to_owned(),
        "product-graph.html".to_owned(),
    ]);

    let findings = evaluate_face_determinism(&first, &second, &decommitted);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, FindingCode::GeneratedFaceStale);
    assert_eq!(findings[0].key, "masterplan.generated.json");
}

#[test]
fn determinism_canary_detects_board_sync_drift_when_other_projections_are_stable() {
    let first = vec![
        (
            "board-sync.generated.json".to_owned(),
            "first board\n".to_owned(),
        ),
        (
            "masterplan.generated.json".to_owned(),
            "stable plan\n".to_owned(),
        ),
    ];
    let second = vec![
        (
            "board-sync.generated.json".to_owned(),
            "second board\n".to_owned(),
        ),
        (
            "masterplan.generated.json".to_owned(),
            "stable plan\n".to_owned(),
        ),
    ];
    let decommitted = BTreeSet::from([
        "board-sync.generated.json".to_owned(),
        "masterplan.generated.json".to_owned(),
    ]);

    let findings = evaluate_face_determinism(&first, &second, &decommitted);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, FindingCode::GeneratedFaceStale);
    assert_eq!(findings[0].key, "board-sync.generated.json");
}

#[test]
fn determinism_canary_ignores_committed_class_faces() {
    // The determinism canary is scoped to de-commit-class faces only.
    let first = vec![(
        "example-committed-face.generated.json".to_owned(),
        "a\n".to_owned(),
    )];
    let second = vec![(
        "example-committed-face.generated.json".to_owned(),
        "b\n".to_owned(),
    )];

    assert!(evaluate_face_determinism(&first, &second, &BTreeSet::new()).is_empty());
}

#[test]
fn remediation_includes_exact_sanctioned_commands() {
    let remediation = render_remediation();

    assert!(remediation.contains("cargo metadata >/dev/null"));
    assert!(remediation.contains(FACE_REMEDIATION_COMMAND));
    assert!(remediation.contains(FACE_SETTLE_PROTOCOL));
    assert!(remediation.contains("commit content changes first"));
    assert!(remediation.contains("faces regenerate from the TRACKED TREE STATE"));
    assert!(remediation.contains("never mix content and regenerated faces in one commit"));
    assert!(remediation.contains("commit only PR-owned generated face diffs"));
    assert!(remediation.contains("not contributor PRs"));
}

#[test]
fn member_manifest_parser_resolves_workspace_inherited_version() {
    let manifest = r#"
[package]
name = "oya-alpha-kernel"
version.workspace = true
"#;

    let package = parse_member_package_manifest("libs/oya-alpha-kernel", manifest, "0.7.0")
        .expect("parse member package");

    assert_eq!(
        package,
        MemberPackage::new("libs/oya-alpha-kernel", "oya-alpha-kernel", "0.7.0",)
    );
}

#[test]
fn rendered_findings_include_codes_keys_details_and_remediation() {
    let findings = vec![Finding {
        code: FindingCode::GeneratedFaceStale,
        key: "scm-facts.generated.json".to_owned(),
        detail: "committed bytes differ from regenerated bytes".to_owned(),
    }];

    let rendered = render_findings(&findings);

    assert!(rendered.contains("generated_face_stale"));
    assert!(rendered.contains("scm-facts.generated.json"));
    assert!(rendered.contains("committed bytes differ"));
    assert!(rendered.contains(FACE_REMEDIATION_COMMAND));
}

#[test]
fn repo_checker_combines_lock_and_face_findings() {
    let root = fixture_root();
    std::fs::create_dir_all(root.join("libs/oya-alpha-kernel")).expect("create member");
    std::fs::create_dir_all(root.join("ci/facade/artifact-inventory-registry"))
        .expect("create faces dir");
    std::fs::write(
        root.join("Cargo.toml"),
        r#"
[workspace]
members = ["libs/*"]
resolver = "2"

[workspace.package]
version = "0.2.0"
"#,
    )
    .expect("write root manifest");
    std::fs::write(
        root.join("libs/oya-alpha-kernel/Cargo.toml"),
        r#"
[package]
name = "oya-alpha-kernel"
version.workspace = true
"#,
    )
    .expect("write member manifest");
    std::fs::write(
        root.join("Cargo.lock"),
        r#"
version = 4

[[package]]
name = "oya-alpha-kernel"
version = "0.1.0"
"#,
    )
    .expect("write stale lock");
    std::fs::write(
        root.join("ci/facade/artifact-inventory-registry/scm-facts.generated.json"),
        "old\n",
    )
    .expect("write committed face");

    let report = check_repo_with_regenerated_faces(
        &root,
        vec![("scm-facts.generated.json".to_owned(), "new\n".to_owned())],
    )
    .expect("check repo");

    assert_eq!(
        codes(&report.findings),
        BTreeSet::from([
            FindingCode::LockStaleMemberVersion,
            FindingCode::GeneratedFaceStale,
        ])
    );
}
