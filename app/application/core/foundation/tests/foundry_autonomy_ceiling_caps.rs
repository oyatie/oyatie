// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_foundation::{
    AgeBand, AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, EvidenceKind, Foundation, FoundationError,
    IdentityRegistration, Purpose, RunDisposition, RunState, SubjectClass, TenantCapabilityGrant,
    TenantRegistration,
};

mod foundry_autonomy_ceiling_caps_fixtures;

use foundry_autonomy_ceiling_caps_fixtures::*;

#[test]
fn principal_ceiling_cannot_exceed_tenant_and_records_source() {
    let mut foundation = foundation_with_tenant(
        "ten_cap_principal",
        AutonomyTier::T4AutoExecute,
        vec!["generic-pack".into()],
    );
    register_capability(
        &mut foundation,
        "ten_cap_principal",
        "cap.autonomy.principal",
        AutonomyTier::T3ExecuteWithApproval,
        vec![DataClass::InternalOnly],
    );

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_cap_principal",
                "usr_operator",
                AutonomyTier::T2Advisory,
            ),
            invocation("ten_cap_principal", "cap.autonomy.principal", 10),
        )
        .expect_err("principal ceiling lower than capability tier must deny");

    assert_eq!(denied, FoundationError::AutonomyCeilingExceeded);
    assert_autonomy_denial(&foundation, "principal", "principal_inherited_ceiling");
    assert_evidence_field(&foundation, "principal_ceiling", "T2Advisory");
    assert_evidence_field(&foundation, "tenant_configured_ceiling", "T4AutoExecute");
    assert_evidence_field(&foundation, "denial_threshold", "T2Advisory");
}

#[test]
fn healthcare_phi_cap_limits_t4_tenant_to_t2() {
    let mut foundation = foundation_with_tenant(
        "ten_cap_health",
        AutonomyTier::T4AutoExecute,
        vec!["health-regulated-pack".into()],
    );
    register_capability(
        &mut foundation,
        "ten_cap_health",
        "cap.autonomy.health",
        AutonomyTier::T3ExecuteWithApproval,
        vec![DataClass::Phi],
    );

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_cap_health",
                "usr_operator",
                AutonomyTier::T4AutoExecute,
            ),
            invocation("ten_cap_health", "cap.autonomy.health", 20),
        )
        .expect_err("health-regulated PHI capability is capped to T2");

    assert_eq!(denied, FoundationError::AutonomyCeilingExceeded);
    assert_autonomy_denial(&foundation, "vertical_pack", "vertical_pack_regulated_data");
    assert_evidence_field(&foundation, "vertical_pack_cap", "T2Advisory");
    assert_evidence_field(&foundation, "subject_class_cap", "T4AutoExecute");
}

#[test]
fn fintech_financial_cap_limits_t4_tenant_to_t2() {
    let mut foundation = foundation_with_tenant(
        "ten_cap_fintech",
        AutonomyTier::T4AutoExecute,
        vec!["financial-regulated-pack".into()],
    );
    register_capability(
        &mut foundation,
        "ten_cap_fintech",
        "cap.autonomy.fintech",
        AutonomyTier::T3ExecuteWithApproval,
        vec![DataClass::FinancialRegulatedCredit],
    );

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_cap_fintech",
                "usr_operator",
                AutonomyTier::T4AutoExecute,
            ),
            invocation("ten_cap_fintech", "cap.autonomy.fintech", 30),
        )
        .expect_err("financial regulated capability is capped to T2");

    assert_eq!(denied, FoundationError::AutonomyCeilingExceeded);
    assert_autonomy_denial(&foundation, "vertical_pack", "vertical_pack_regulated_data");
    assert_evidence_field(&foundation, "vertical_pack_cap", "T2Advisory");
}

#[test]
fn minor_subject_class_cap_limits_internal_capability_to_t1() {
    let mut foundation = foundation_with_tenant(
        "ten_cap_minor",
        AutonomyTier::T4AutoExecute,
        vec!["generic-pack".into()],
    );
    register_capability(
        &mut foundation,
        "ten_cap_minor",
        "cap.autonomy.minor",
        AutonomyTier::T2Advisory,
        vec![DataClass::InternalOnly],
    );

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal("ten_cap_minor", "usr_operator", AutonomyTier::T4AutoExecute),
            invocation_with_subject(
                "ten_cap_minor",
                "cap.autonomy.minor",
                40,
                SubjectClass::Minor {
                    age_band: AgeBand::Under14,
                },
            ),
        )
        .expect_err("minor subject cap limits elevated capability");

    assert_eq!(denied, FoundationError::AutonomyCeilingExceeded);
    assert_autonomy_denial(&foundation, "subject_class", "subject_class_risk");
    assert_evidence_field(&foundation, "subject_class_cap", "T1ViewOnly");
    assert_evidence_field(&foundation, "effective_ceiling", "T1ViewOnly");
}

#[test]
fn typed_agentic_ads_bid_defaults_to_t1() {
    let mut foundation = foundation_with_tenant(
        "ten_cap_ads",
        AutonomyTier::T4AutoExecute,
        vec!["generic-pack".into()],
    );
    register_capability_with_action(
        &mut foundation,
        "ten_cap_ads",
        "cap.ads.bid",
        "ads",
        CapabilityAction::AdsBid,
        AutonomyTier::T2Advisory,
        vec![DataClass::InternalOnly],
    );

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal("ten_cap_ads", "usr_operator", AutonomyTier::T4AutoExecute),
            invocation("ten_cap_ads", "cap.ads.bid", 45),
        )
        .expect_err("agentic ads bid defaults to T1 without a founder override");

    assert_eq!(denied, FoundationError::AutonomyCeilingExceeded);
    assert_autonomy_denial(&foundation, "agentic_ads", "agentic_ads_default");
    assert_evidence_field(&foundation, "agentic_ads_cap", "T1ViewOnly");
    assert_evidence_field(&foundation, "denial_threshold", "T1ViewOnly");
}

#[test]
fn low_tier_capability_does_not_run_at_t4_under_t4_tenant() {
    let mut foundation = foundation_with_tenant(
        "ten_cap_low_tier",
        AutonomyTier::T4AutoExecute,
        vec!["generic-pack".into()],
    );
    register_capability(
        &mut foundation,
        "ten_cap_low_tier",
        "cap.autonomy.lowtier",
        AutonomyTier::T1ViewOnly,
        vec![DataClass::InternalOnly],
    );
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_cap_low_tier".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 10_000,
            per_invocation_limit_micros: 1_000,
            warning_threshold_percent: 80,
        })
        .unwrap();

    let receipt = foundation
        .invoke_capability_as_principal(
            support::principal(
                "ten_cap_low_tier",
                "usr_operator",
                AutonomyTier::T4AutoExecute,
            ),
            invocation("ten_cap_low_tier", "cap.autonomy.lowtier", 50),
        )
        .expect("T1 capability is allowed under T4 tenant/principal");

    assert!(receipt.run_id.is_some());
    let run = foundation
        .foundry_runs()
        .last()
        .expect("allowed invocation records a run");
    assert_eq!(run.state.value, RunState::Succeeded);
    assert_eq!(run.autonomy_tier_used.value, AutonomyTier::T1ViewOnly);
    let autonomy_evidence = foundation
        .foundry_evidence_chain()
        .records()
        .iter()
        .find(|evidence| evidence.kind.value == EvidenceKind::AutonomyDecision)
        .expect("allowed invocation emits autonomy decision evidence");
    assert_eq!(autonomy_evidence.step_id.value, None);
    assert_eq!(
        autonomy_evidence
            .fields
            .value
            .get("decision")
            .map(String::as_str),
        Some("ALLOW")
    );
    assert_eq!(
        autonomy_evidence
            .fields
            .value
            .get("effective_ceiling")
            .map(String::as_str),
        Some("T1ViewOnly")
    );
}
