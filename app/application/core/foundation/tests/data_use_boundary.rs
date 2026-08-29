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
fn data_use_grants_reject_non_privacy_markers() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_marker_grant".into(),
            legal_name: "Marker Grant Tenant".into(),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    let grant_events_before = foundation
        .audit_chain()
        .events()
        .iter()
        .filter(|event| event.surface == "privacy.data-use.grant")
        .count();

    for data_class in [DataClass::Audit, DataClass::Secret, DataClass::Children] {
        assert_eq!(
            foundation.try_grant_legacy_data_use(
                &tenant.id,
                Purpose::CapabilityInvocation,
                data_class
            ),
            Err(FoundationError::InvalidInput)
        );
    }

    let grant_events_after = foundation
        .audit_chain()
        .events()
        .iter()
        .filter(|event| event.surface == "privacy.data-use.grant")
        .count();
    assert_eq!(grant_events_after, grant_events_before);
}

#[test]
fn capability_invocation_requires_purpose_bound_data_class_grant() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_delta".into(),
            legal_name: "Delta Search".into(),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling: AutonomyTier::T3ExecuteWithApproval,
        })
        .expect("tenant can be onboarded");
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_delta_admin".into(),
            primary_identifier: "admin@delta.example".into(),
            display_name: "Delta Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, &tenant.id, "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.search.index-customer");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.search.index-customer".into(),
            namespace: "search".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
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

    let denied = foundation
        .invoke_capability_as_principal(
            support::principal(
                &tenant.id,
                user.id.value.as_str(),
                AutonomyTier::T3ExecuteWithApproval,
            ),
            CapabilityInvocationRequest {
                tenant_id: tenant.id.clone(),
                user_id: user.id.value.as_str().to_string(),
                capability_id: capability.id.clone(),
                purpose: Purpose::SearchIndex,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("search indexing PII requires explicit data-use grant");
    assert_eq!(denied, FoundationError::DataUseNotAllowed);
    let denied_run = foundation
        .foundry_runs()
        .last()
        .expect("data denial records a rejected run");
    assert_eq!(denied_run.state.value, RunState::RejectedClass);
    assert_eq!(
        denied_run.disposition.value,
        Some(RunDisposition::FailureClass)
    );
    let denied_evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("data denial records evidence");
    assert_eq!(denied_evidence.step_id.value, None);
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("reason")
            .map(String::as_str),
        Some("data_boundary")
    );

    foundation
        .grant_data_use(
            &tenant.id,
            Purpose::SearchIndex,
            support::privacy_data_class(DataClass::PiiIdentifying),
        )
        .expect("privacy council grant can be recorded");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: tenant.id.clone(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("cost budget is configured before dispatch");

    let receipt = foundation
        .invoke_capability_as_principal(
            support::principal(
                &tenant.id,
                user.id.value.as_str(),
                AutonomyTier::T3ExecuteWithApproval,
            ),
            CapabilityInvocationRequest {
                tenant_id: tenant.id.clone(),
                user_id: user.id.value.as_str().to_string(),
                capability_id: capability.id.clone(),
                purpose: Purpose::SearchIndex,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect("grant allows purpose-bound search indexing");
    assert_eq!(receipt.capability_id, capability.id);
    assert!(foundation.audit_chain().verify());
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| event.surface == "privacy.data-use.grant" && event.decision == "ALLOW")
    );
}

#[test]
fn minor_subject_ads_are_denied_by_composite_data_use_boundary() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_minor_ads".into(),
            legal_name: "Minor Ads Tenant".into(),
            home_region: "region-recovery".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_minor_ads_admin".into(),
            primary_identifier: "minor-ads@example.test".into(),
            display_name: "Minor Ads Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, &tenant.id, "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.ads.internal");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.ads.internal".into(),
            namespace: "ads".into(),
            action: CapabilityAction::Other,
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
                purpose: Purpose::AdsTargeting,
                subject_class: SubjectClass::Minor {
                    age_band: application_foundation::AgeBand::Under13,
                },
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("minor-subject ads are denied even for InternalOnly data");

    assert_eq!(denied, FoundationError::DataUseNotAllowed);
    let denied_run = foundation
        .foundry_runs()
        .last()
        .expect("minor-subject denial records a rejected run");
    assert_eq!(denied_run.state.value, RunState::RejectedClass);
    assert_eq!(
        denied_run.disposition.value,
        Some(RunDisposition::FailureClass)
    );
    let denied_evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("minor-subject denial records evidence");
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("reason")
            .map(String::as_str),
        Some("data_boundary")
    );
}
