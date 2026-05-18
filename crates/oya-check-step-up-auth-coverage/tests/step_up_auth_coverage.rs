use oya_check_step_up_auth_coverage::{FindingKind, scan};

fn fixture(body: &str) -> oya_check_step_up_auth_coverage::StepUpAuthCoverageReport {
    scan("test://spec", body).expect("scan")
}

#[test]
fn clean_spec_with_declarations_is_ok() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /widgets:
    post:
      operationId: createWidget
      x-acr-required: elevated
      responses: {"200": {description: ok}}
    get:
      operationId: listWidgets
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert!(r.ok(), "findings: {:?}", r.findings);
    assert_eq!(r.operations_inspected, 2);
    assert_eq!(r.operations_with_declaration, 1);
}

#[test]
fn missing_declaration_on_mutating_path_flags() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /widgets:
    post:
      operationId: createWidget
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert_eq!(r.findings.len(), 1);
    assert_eq!(r.findings[0].finding_kind, FindingKind::MissingOnMutating);
}

#[test]
fn sensitive_path_below_floor_flags() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /secrets/{id}/rotate:
    post:
      operationId: rotateSecret
      x-acr-required: elevated
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert_eq!(r.findings.len(), 1);
    assert_eq!(
        r.findings[0].finding_kind,
        FindingKind::BelowFloorOnSensitivePath
    );
}

#[test]
fn sensitive_path_at_floor_passes() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /secrets/{id}/rotate:
    post:
      operationId: rotateSecret
      x-acr-required: sensitive
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert!(r.ok(), "findings: {:?}", r.findings);
}

#[test]
fn critical_path_at_critical_passes() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /admin/tenants/{id}:
    delete:
      operationId: deleteTenant
      x-acr-required: critical
      responses: {"204": {description: gone}}
"#;
    let r = fixture(spec);
    assert!(r.ok(), "findings: {:?}", r.findings);
}

#[test]
fn unknown_value_flags() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /widgets:
    post:
      x-acr-required: very-high
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert_eq!(r.findings.len(), 1);
    assert_eq!(r.findings[0].finding_kind, FindingKind::UnknownValue);
}

#[test]
fn exempt_operations_are_skipped() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /webhooks/inbound:
    post:
      operationId: receiveWebhook
      x-acr-exempt: true
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert!(r.ok(), "findings: {:?}", r.findings);
}

#[test]
fn read_only_methods_are_not_required_to_declare() {
    let spec = r#"
openapi: 3.1.0
info: {title: x, version: 1.0.0}
paths:
  /widgets:
    get:
      operationId: listWidgets
      responses: {"200": {description: ok}}
"#;
    let r = fixture(spec);
    assert!(r.ok());
}

#[test]
fn malformed_spec_returns_parse_error() {
    let err = scan("test", ":::: not yaml ::::").err().expect("err");
    let msg = format!("{err}");
    assert!(msg.contains("step-up-auth-coverage parse"));
}

#[test]
fn missing_paths_section_returns_parse_error() {
    let err = scan(
        "test",
        "openapi: 3.1.0\ninfo:\n  title: x\n  version: 1.0.0\n",
    )
    .err()
    .expect("err");
    let msg = format!("{err}");
    assert!(msg.contains("missing top-level `paths`"));
}
