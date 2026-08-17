use std::fs;
use std::path::{Path, PathBuf};

use oya_governance_audit_event_emission::{
    ENFORCED_RULE, EnforcementStatus, FindingKind, RULE_ID, enforce_audit_event_emission,
};

fn temp_repo(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "oya-governance-audit-event-emission-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp repo");
    root
}

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("parent dir")).expect("create parent dir");
    fs::write(path, content).expect("write fixture");
}

#[test]
fn reports_rule_metadata_for_clean_repo() {
    let root = temp_repo("metadata");
    let outcome = enforce_audit_event_emission(&root).expect("check should report metadata");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.enforced_rule, ENFORCED_RULE);
    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert!(outcome.is_success());
    assert!(!outcome.is_scaffolded());
}

#[test]
fn rejects_missing_scan_root_instead_of_passing_empty() {
    let root = std::env::temp_dir().join(format!(
        "oya-governance-audit-event-emission-missing-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);

    assert!(enforce_audit_event_emission(&root).is_err());
}

#[test]
fn rejects_mutating_endpoint_without_audit_event() {
    let root = temp_repo("missing-audit-event");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    post:
      operationId: CreateWidget
"#,
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.mutating_endpoint_identifiers, vec!["CreateWidget"]);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].kind, FindingKind::MissingAuditEvent);
    assert_eq!(
        outcome.findings[0].identifier.as_deref(),
        Some("CreateWidget")
    );
}

#[test]
fn accepts_mutating_endpoint_with_registered_audit_event() {
    let root = temp_repo("registered-audit-event");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    post:
      operationId: CreateWidget
"#,
    );
    write(
        &root,
        "registry/audit-events/widgets.yaml",
        r#"events:
  - class: widget.created
    emitted_by: CreateWidget
    adr: ADR-0263
"#,
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.mutating_endpoint_identifiers, vec!["CreateWidget"]);
    assert_eq!(outcome.audit_evidence_files.len(), 1);
    assert!(outcome.findings.is_empty(), "{outcome:?}");
}
#[test]
fn accepts_openapi_json_mutation_with_registered_audit_event() {
    let root = temp_repo("openapi-json-registered");
    write(
        &root,
        "contracts/openapi/widgets.openapi.json",
        r#"{
  "openapi": "3.0.0",
  "paths": {
    "/widgets": {
      "post": {
        "operationId": "CreateWidget"
      }
    }
  }
}"#,
    );
    write(
        &root,
        "registry/audit-events/widgets.yaml",
        r#"events:
  - class: widget.created
    emitted_by: CreateWidget
    adr: ADR-0263
"#,
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Passed);
    assert_eq!(outcome.mutating_endpoint_identifiers, vec!["CreateWidget"]);
    assert!(outcome.findings.is_empty(), "{outcome:?}");
}
#[test]
fn rejects_malformed_openapi_json_instead_of_falling_back_to_empty_success() {
    let root = temp_repo("malformed-openapi-json");
    write(
        &root,
        "contracts/openapi/widgets.openapi.json",
        r#"{"openapi":"3.0.0","paths":{"#,
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(
        outcome.findings[0].kind,
        FindingKind::MissingEndpointIdentifier
    );
}

#[test]
fn rejects_comment_or_superstring_audit_event_evidence() {
    let root = temp_repo("comment-superstring-audit-event");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    post:
      operationId: CreateWidget
"#,
    );
    write(
        &root,
        "registry/audit-events/widgets.yaml",
        r#"events:
  # emitted_by: CreateWidget
  - class: widget.previewed
    emitted_by: CreateWidgetPreview
    adr: ADR-0263
"#,
    );
    write(
        &root,
        "evidence/audit/prose.md",
        "CreateWidget appears in prose but not as a structured registered emitter.",
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].kind, FindingKind::MissingAuditEvent);
    assert_eq!(
        outcome.findings[0].identifier.as_deref(),
        Some("CreateWidget")
    );
}

#[test]
fn ignores_incidental_audit_named_files_without_registry_evidence() {
    let root = temp_repo("incidental-audit-file");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    post:
      operationId: CreateWidget
"#,
    );
    write(
        &root,
        "notes/audit-event-comment.md",
        "CreateWidget appears in prose but is not registered audit-event evidence.",
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert!(outcome.audit_evidence_files.is_empty());
    assert_eq!(outcome.findings[0].kind, FindingKind::MissingAuditEvent);
}

#[test]
fn reports_missing_mutating_endpoint_identifiers() {
    let root = temp_repo("missing-identifier");
    write(
        &root,
        "contracts/openapi/widgets.openapi.yaml",
        r#"openapi: 3.0.0
paths:
  /widgets:
    post:
      summary: Create widget
"#,
    );

    let outcome = enforce_audit_event_emission(&root).expect("check should run");

    assert_eq!(outcome.status, EnforcementStatus::Failed);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(
        outcome.findings[0].kind,
        FindingKind::MissingEndpointIdentifier
    );
    assert!(outcome.findings[0].hint.contains("POST /widgets"));
}
