use oya_governance_cedar_coverage::{
    ENFORCED_RULE, EnforcementStatus, RULE_ID, enforce_cedar_coverage,
};

#[test]
fn scaffold_reports_rule_metadata() {
    let outcome = enforce_cedar_coverage(".").expect("scaffold should report metadata");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.enforced_rule, ENFORCED_RULE);
    assert_eq!(outcome.status, EnforcementStatus::Scaffolded);
    assert!(outcome.is_scaffolded());
}

#[test]
#[ignore = "Wave-3-I implementation will reject public endpoints without Cedar policy"]
fn rejects_public_endpoint_without_cedar_policy() {}

#[test]
#[ignore = "Wave-3-I implementation will accept public endpoints with matching Cedar policy"]
fn accepts_public_endpoint_with_matching_cedar_policy() {}

#[test]
#[ignore = "Wave-3-I implementation will report uncovered public endpoint identifiers"]
fn reports_uncovered_public_endpoint_identifiers() {}
