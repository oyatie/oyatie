use oya_governance_capability_tier_coverage::{
    ENFORCED_RULE, EnforcementStatus, RULE_ID, enforce_capability_tier_coverage,
};

#[test]
fn scaffold_reports_rule_metadata() {
    let outcome = enforce_capability_tier_coverage(".").expect("scaffold should report metadata");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.enforced_rule, ENFORCED_RULE);
    assert_eq!(outcome.status, EnforcementStatus::Scaffolded);
    assert!(outcome.is_scaffolded());
}

#[test]
#[ignore = "Wave-3-I implementation will reject missing microservice tier rows"]
fn rejects_missing_microservice_tier_mapping() {}

#[test]
#[ignore = "Wave-3-I implementation will accept one mapping per microservice"]
fn accepts_complete_microservice_tier_mapping() {}

#[test]
#[ignore = "Wave-3-I implementation will report the unmapped service names"]
fn reports_unmapped_microservice_names() {}
