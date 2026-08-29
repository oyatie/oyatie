// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, Foundation, FoundationError, IdentityRegistration, Purpose,
    RunDisposition, RunState, SubjectClass, TenantCapabilityGrant, TenantRegistration,
};

#[test]
fn ads_action_cannot_underdeclare_effective_data_use_purpose() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_ads_underdeclared".into(),
            legal_name: "Ads Underdeclared Tenant".into(),
            home_region: "region-recovery".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_ads_underdeclared_admin".into(),
            primary_identifier: "ads-underdeclared@example.test".into(),
            display_name: "Ads Underdeclared Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, &tenant.id, "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.ads.underdeclared");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.ads.underdeclared".into(),
            namespace: "ads".into(),
            action: CapabilityAction::AdsBid,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .expect("capability is valid");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: tenant.id.clone(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("tenant can be licensed for capability");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: tenant.id.clone(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("budget exists so a pass-through would invoke");

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(&tenant.id, user.id.value.as_str(), AutonomyTier::T2Advisory),
            CapabilityInvocationRequest {
                tenant_id: tenant.id.clone(),
                user_id: user.id.value.as_str().to_string(),
                capability_id: capability.id.clone(),
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("ads actions cannot hide behind a generic invocation purpose");

    assert_eq!(denied, FoundationError::DataUseNotAllowed);
    assert_eq!(foundation.foundry_steps().len(), 0);
    assert!(
        !foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| event.surface == "foundry.cost-budget.reserve")
    );
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "privacy.data-use.evaluate"
            && event.purpose == Purpose::AdsTargeting
            && event.decision == "DENY"
    }));
    let denied_evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("underdeclared purpose denial records evidence");
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("reason")
            .map(String::as_str),
        Some("data_boundary")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("data_use_denial_reason")
            .map(String::as_str),
        Some("underdeclared_ads_purpose")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("requested_purpose")
            .map(String::as_str),
        Some("CapabilityInvocation")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("effective_purpose")
            .map(String::as_str),
        Some("AdsTargeting")
    );
    assert!(
        denied_evidence
            .fields
            .value
            .contains_key("data_use_audit_event_hash")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("subject_class")
            .map(String::as_str),
        Some("Adult")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("data_classes")
            .map(String::as_str),
        Some("INTERNAL_ONLY")
    );
}

#[test]
fn ads_targeting_pii_is_denied_even_with_recorded_grant() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_ads_pii".into(),
            legal_name: "Ads PII Tenant".into(),
            home_region: "region-recovery".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_ads_pii_admin".into(),
            primary_identifier: "ads-pii@example.test".into(),
            display_name: "Ads PII Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, &tenant.id, "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.ads.pii");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.ads.pii".into(),
            namespace: "ads".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                DataClass::PiiIdentifying,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .expect("capability is valid");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: tenant.id.clone(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("tenant can be licensed for capability");
    foundation
        .grant_data_use(
            &tenant.id,
            Purpose::AdsTargeting,
            support::privacy_data_class(DataClass::PiiIdentifying),
        )
        .expect("recording a grant is allowed for auditability");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: tenant.id.clone(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("budget exists so a pass-through would invoke");

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(&tenant.id, user.id.value.as_str(), AutonomyTier::T2Advisory),
            CapabilityInvocationRequest {
                tenant_id: tenant.id.clone(),
                user_id: user.id.value.as_str().to_string(),
                capability_id: capability.id.clone(),
                purpose: Purpose::AdsTargeting,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("PII_IDENTIFYING is never eligible for ads targeting");

    assert_eq!(denied, FoundationError::DataUseNotAllowed);
    assert_eq!(foundation.foundry_steps().len(), 0);
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "privacy.data-use.evaluate"
            && event.purpose == Purpose::AdsTargeting
            && event.decision == "DENY"
    }));
    let denied_evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("ads PII denial records evidence");
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("data_use_denial_reason")
            .map(String::as_str),
        Some("hard_denied_data_class")
    );
}
