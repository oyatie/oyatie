use oya_governance_audit_event_emission::{
    ENFORCED_RULE, EnforcementStatus, RULE_ID, enforce_audit_event_emission,
};

#[test]
fn scaffold_reports_rule_metadata() {
    let outcome = enforce_audit_event_emission(".").expect("scaffold should report metadata");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.enforced_rule, ENFORCED_RULE);
    assert_eq!(outcome.status, EnforcementStatus::Scaffolded);
    assert!(outcome.is_scaffolded());
}

#[test]
#[ignore = "Wave-3-I implementation will reject state-changing endpoints without audit event emission"]
fn rejects_mutating_endpoint_without_audit_event() {}

#[test]
#[ignore = "Wave-3-I implementation will accept mutating endpoints with registered audit event emission"]
fn accepts_mutating_endpoint_with_registered_audit_event() {}

#[test]
#[ignore = "Wave-3-I implementation will report missing ADR-0263 event classes"]
fn reports_missing_registered_audit_event_classes() {}
