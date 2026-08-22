// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_app::{
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
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
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
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
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
                    age_band: application_app::AgeBand::Under13,
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
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
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
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
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

#[test]
fn analytics_pci_invocation_is_denied_even_with_recorded_grant() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_analytics_pci".into(),
            legal_name: "Analytics PCI Tenant".into(),
            home_region: "region-recovery".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-gamma".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_analytics_pci_admin".into(),
            primary_identifier: "analytics-pci@example.test".into(),
            display_name: "Analytics PCI Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, &tenant.id, "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.analytics.pci");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.analytics.pci".into(),
            namespace: "analytics".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T1ViewOnly,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::Pci,
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
            Purpose::Analytics,
            support::privacy_data_class(DataClass::Pci),
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
                purpose: Purpose::Analytics,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect_err("analytics PCI is hard denied even with a recorded grant");

    assert_eq!(denied, FoundationError::DataUseNotAllowed);
    let denied_run = foundation
        .foundry_runs()
        .last()
        .expect("analytics PCI denial records a rejected run");
    assert_eq!(denied_run.state.value, RunState::RejectedClass);
    assert_eq!(
        denied_run.disposition.value,
        Some(RunDisposition::FailureClass)
    );
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
            && event.purpose == Purpose::Analytics
            && event.decision == "DENY"
    }));
    let denied_evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("analytics PCI denial records evidence");
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
        Some("hard_denied_data_class")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("effective_purpose")
            .map(String::as_str),
        Some("Analytics")
    );
    assert_eq!(
        denied_evidence
            .fields
            .value
            .get("denied_data_class")
            .map(String::as_str),
        Some("PCI")
    );
}

#[test]
fn hard_denied_data_classes_cannot_be_enabled_by_recorded_grants() {
    let mut foundation = Foundation::default();
    let tenant = foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_privacy".into(),
            legal_name: "Privacy Boundary Tenant".into(),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling: AutonomyTier::T3ExecuteWithApproval,
        })
        .expect("tenant can be onboarded");
    let user = foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant.id.clone(),
            user_id: "usr_privacy_admin".into(),
            primary_identifier: "privacy@tenant.example".into(),
            display_name: "Privacy Admin".into(),
            roles: vec!["privacy-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, &tenant.id, "privacy-admin");

    let hard_denied_classes = [
        ("phi", DataClass::Phi),
        ("pci", DataClass::Pci),
        ("sensitive-pipa-art23", DataClass::SensitivePipaArticle23),
        (
            "financial-regulated-credit",
            DataClass::FinancialRegulatedCredit,
        ),
    ];

    for (class_label, data_class) in hard_denied_classes {
        for (purpose_label, purpose) in [
            ("search-index", Purpose::SearchIndex),
            ("ads-targeting", Purpose::AdsTargeting),
        ] {
            let capability_id = format!("cap.privacy.{class_label}.{purpose_label}");
            support::seed_passing_eval(&mut foundation, &capability_id);
            let capability =
                foundation
                    .register_capability(CapabilityRegistration {
                        capability_id,
                        namespace: "privacy".into(),
                        action: CapabilityAction::Other,
                        required_tier: AutonomyTier::T2Advisory,
                        touched_privacy_data_classes:
                            application_app::privacy_data_classes_from(&[data_class]).unwrap(),
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
                .grant_data_use(&tenant.id, purpose, support::privacy_data_class(data_class))
                .expect("recording a grant is allowed for auditability");

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
                        purpose,
                        subject_class: application_app::SubjectClass::Adult,
                        budget_window_id: "2026-05".into(),
                        projected_cost_micros: 10,
                        started_at_epoch_seconds: 1_000,
                    },
                )
                .expect_err("HARD_DENY classes ignore consent grants");
            assert_eq!(denied, FoundationError::DataUseNotAllowed);
            assert_eq!(
                foundation
                    .foundry_runs()
                    .last()
                    .expect("hard deny records a run")
                    .state
                    .value,
                RunState::RejectedClass
            );
        }
    }

    assert!(foundation.audit_chain().verify());
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| event.surface == "foundry.capability.invoke" && event.decision == "DENY")
    );
}
