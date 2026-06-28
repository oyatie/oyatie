use std::fs;
use std::path::PathBuf;

use oya_governance_cedar_coverage::{
    ENFORCED_RULE, EnforcementStatus, FindingKind, RULE_ID, enforce_cedar_coverage,
};

fn temp_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-governance-cedar-coverage-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp repo");
    root
}

fn write(root: &PathBuf, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent dir")).expect("create parent dir");
    fs::write(path, content).expect("write fixture");
}

#[test]
fn reports_rule_metadata_for_clean_repo() {
    let root = temp_repo("metadata");
    let outcome = enforce_cedar_coverage(&root).expect("check should report metadata");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.enforced_rule, ENFORCED_RULE);
    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert!(outcome.is_success());
    assert!(!outcome.is_scaffolded());
}

#[test]
fn rejects_public_endpoint_without_cedar_policy() {
    let root = temp_repo("missing-policy");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    get:
      operationId: ListWidgets
"#,
    );

    let outcome = enforce_cedar_coverage(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.endpoint_identifiers, vec!["ListWidgets"]);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].kind, FindingKind::MissingCedarPolicy);
    assert_eq!(
        outcome.findings[0].identifier.as_deref(),
        Some("ListWidgets")
    );
}

#[test]
fn accepts_public_endpoint_with_matching_cedar_policy() {
    let root = temp_repo("matching-policy");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    get:
      operationId: ListWidgets
"#,
    );
    write(
        &root,
        "policies/widgets.cedar",
        r#"permit(principal, action == Action::"ListWidgets", resource);"#,
    );

    let outcome = enforce_cedar_coverage(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.endpoint_identifiers, vec!["ListWidgets"]);
    assert_eq!(outcome.cedar_policy_files.len(), 1);
    assert!(outcome.findings.is_empty(), "{outcome:?}");
}

#[test]
fn reports_uncovered_public_endpoint_identifiers() {
    let root = temp_repo("missing-identifier");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    get:
      summary: List widgets
"#,
    );

    let outcome = enforce_cedar_coverage(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(
        outcome.findings[0].kind,
        FindingKind::MissingEndpointIdentifier
    );
    assert!(outcome.findings[0].hint.contains("GET /widgets"));
}
