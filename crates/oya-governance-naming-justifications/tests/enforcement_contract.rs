use std::fs;
use std::path::{Path, PathBuf};

use oya_governance_naming_justifications::{
    EnforcementStatus, NamingViolationKind, RULE_ID, enforce_naming_justifications,
};

#[test]
fn accepts_yaml_manifest_with_structured_one_line_proof() {
    let root = fixture_root("naming-pass-yaml");
    write(
        &root,
        "microservices/mail/manifest.yaml",
        r#"name: mail
naming_justifications: "BNF v4.1 service_action_resource=mail.notice.deliver.message; 12-layer-enum=api"
"#,
    );

    let outcome = enforce_naming_justifications(&root).expect("check should run");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.scanned_manifests, 1);
    assert!(outcome.violations.is_empty());
}

#[test]
fn rejects_manifest_missing_naming_justifications() {
    let root = fixture_root("naming-fail-missing");
    write(
        &root,
        "microservices/mail/manifest.json",
        r#"{"name":"mail"}"#,
    );

    let outcome = enforce_naming_justifications(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.violations.len(), 1);
    assert_eq!(outcome.violations[0].line, 1);
    assert_eq!(
        outcome.violations[0].kind,
        NamingViolationKind::MissingField
    );
}

#[test]
fn rejects_proof_without_twelve_layer_enum() {
    let root = fixture_root("naming-fail-layer");
    write(
        &root,
        "microservices/mail/manifest.toml",
        r#"name = "mail"
naming_justifications = "BNF v4.1 service_action_resource=mail.notice.deliver.message"
"#,
    );

    let outcome = enforce_naming_justifications(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert!(outcome.violations.iter().any(|violation| {
        violation.kind == NamingViolationKind::MissingTwelveLayerEnumCitation && violation.line == 2
    }));
}

#[test]
fn rejects_multiline_yaml_proof() {
    let root = fixture_root("naming-fail-multiline");
    write(
        &root,
        "microservices/workflow/manifest.yaml",
        r#"name: workflow
naming_justifications: |
  BNF v4.1 service_action_resource=workflow.run.execute.task
  12-layer-enum=usecase
"#,
    );

    let outcome = enforce_naming_justifications(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert!(
        outcome
            .violations
            .iter()
            .any(|violation| violation.kind == NamingViolationKind::MultiLineProof)
    );
}

#[test]
fn ignores_non_manifest_microservice_files() {
    let root = fixture_root("naming-pass-ignore");
    write(
        &root,
        "microservices/mail/not-a-manifest.yaml",
        "naming_justifications: absent",
    );

    let outcome = enforce_naming_justifications(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.scanned_manifests, 0);
}

fn fixture_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-governance-naming-{}-{}",
        name,
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture root");
    }
    fs::create_dir_all(&root).expect("create fixture root");
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture file has parent"))
        .expect("create fixture parent");
    fs::write(path, content).expect("write fixture file");
}
