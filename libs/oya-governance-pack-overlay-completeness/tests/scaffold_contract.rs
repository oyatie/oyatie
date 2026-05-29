use oya_governance_pack_overlay_completeness::{
    ENFORCED_RULE, EnforcementStatus, RULE_ID, enforce_pack_overlay_completeness,
};

#[test]
fn scaffold_reports_rule_metadata() {
    let outcome = enforce_pack_overlay_completeness(".").expect("scaffold should report metadata");

    assert_eq!(outcome.rule_id, RULE_ID);
    assert_eq!(outcome.enforced_rule, ENFORCED_RULE);
    assert_eq!(outcome.status, EnforcementStatus::Scaffolded);
    assert!(outcome.is_scaffolded());
}

#[test]
#[ignore = "Wave-3-I implementation will reject services missing applicable pack overlays"]
fn rejects_service_missing_applicable_pack_overlay() {}

#[test]
#[ignore = "Wave-3-I implementation will accept services with every applicable pack overlay"]
fn accepts_service_with_complete_applicable_pack_overlays() {}

#[test]
#[ignore = "Wave-3-I implementation will report the service and missing pack overlay"]
fn reports_missing_pack_overlay_by_service() {}
