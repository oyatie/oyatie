#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use oya_cloud_ci_freshness_app::{
    Finding, FindingCode, LockPackage, MemberPackage, check_repo_with_regenerated_faces,
    evaluate_face_freshness, evaluate_lock_freshness, parse_lock_packages,
    parse_member_package_manifest, render_findings, render_remediation,
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

    assert!(evaluate_face_freshness(&committed, &regenerated).is_empty());
}

#[test]
fn face_freshness_reports_stale_generated_face() {
    let committed = vec![("scm-facts.generated.json".to_owned(), "old\n".to_owned())];
    let regenerated = vec![("scm-facts.generated.json".to_owned(), "new\n".to_owned())];

    let findings = evaluate_face_freshness(&committed, &regenerated);

    assert_eq!(
        codes(&findings),
        BTreeSet::from([FindingCode::GeneratedFaceStale])
    );
    assert_eq!(findings[0].key, "scm-facts.generated.json");
}

#[test]
fn remediation_includes_exact_sanctioned_commands() {
    let remediation = render_remediation();

    assert!(remediation.contains("cargo metadata >/dev/null"));
    assert!(remediation.contains("infra/ci/materialize-cloud-ci-generated-faces.sh ."));
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
    assert!(rendered.contains("infra/ci/materialize-cloud-ci-generated-faces.sh ."));
}

#[test]
fn repo_checker_combines_lock_and_face_findings() {
    let root = fixture_root();
    std::fs::create_dir_all(root.join("libs/oya-alpha-kernel")).expect("create member");
    std::fs::create_dir_all(root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app"))
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
        root.join(
            "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
        ),
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
