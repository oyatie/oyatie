// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

use application_foundation::{
    AgeBand, AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, EvidenceKind, Foundation, FoundationError,
    IdentityRegistration, Purpose, RunDisposition, RunState, SubjectClass, TenantCapabilityGrant,
    TenantRegistration,
};

pub fn foundation_with_tenant(
    tenant_id: &str,
    autonomy_ceiling: AutonomyTier,
    regulatory_packs: Vec<String>,
) -> Foundation {
    let mut foundation = Foundation::default();
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: tenant_id.into(),
            legal_name: format!("{tenant_id} Legal"),
            home_region: "region-alpha".into(),
            residency_class: "global".into(),
            regulatory_packs,
            autonomy_ceiling,
        })
        .unwrap();
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: tenant_id.into(),
            user_id: "usr_operator".into(),
            primary_identifier: format!("operator@{tenant_id}.example.test"),
            display_name: "Operator".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    super::support::allow_capability_invocation(&mut foundation, tenant_id, "tenant-admin");
    foundation
}

pub fn register_capability(
    foundation: &mut Foundation,
    tenant_id: &str,
    capability_id: &str,
    required_tier: AutonomyTier,
    touched_data_classes: Vec<DataClass>,
) {
    register_capability_with_action(
        foundation,
        tenant_id,
        capability_id,
        "autonomy",
        CapabilityAction::Other,
        required_tier,
        touched_data_classes,
    )
}

pub fn register_capability_with_action(
    foundation: &mut Foundation,
    tenant_id: &str,
    capability_id: &str,
    namespace: &str,
    action: CapabilityAction,
    required_tier: AutonomyTier,
    touched_data_classes: Vec<DataClass>,
) {
    super::support::seed_passing_eval(foundation, capability_id);
    foundation
        .register_capability(CapabilityRegistration {
            capability_id: capability_id.into(),
            namespace: namespace.into(),
            action,
            required_tier,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(
                &touched_data_classes,
            )
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: tenant_id.into(),
            capability_id: capability_id.into(),
            mcp_visible: true,
        })
        .unwrap();
}

pub fn invocation(
    tenant_id: &str,
    capability_id: &str,
    started_at_epoch_seconds: u64,
) -> CapabilityInvocationRequest {
    invocation_with_subject(
        tenant_id,
        capability_id,
        started_at_epoch_seconds,
        SubjectClass::Adult,
    )
}

pub fn invocation_with_subject(
    tenant_id: &str,
    capability_id: &str,
    started_at_epoch_seconds: u64,
    subject_class: SubjectClass,
) -> CapabilityInvocationRequest {
    CapabilityInvocationRequest {
        tenant_id: tenant_id.into(),
        user_id: "usr_operator".into(),
        capability_id: capability_id.into(),
        purpose: Purpose::CapabilityInvocation,
        subject_class,
        budget_window_id: "2026-05".into(),
        projected_cost_micros: 10,
        started_at_epoch_seconds,
    }
}

pub fn assert_autonomy_denial(
    foundation: &Foundation,
    expected_source: &str,
    expected_reason: &str,
) {
    assert_eq!(foundation.foundry_steps().len(), 0);
    let run = foundation
        .foundry_runs()
        .last()
        .expect("autonomy denial records a run");
    assert_eq!(run.state.value, RunState::RejectedAutonomy);
    assert_eq!(run.disposition.value, Some(RunDisposition::FailureAutonomy));
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("autonomy denial records evidence");
    assert_eq!(evidence.kind.value, EvidenceKind::AutonomyDecision);
    assert_eq!(evidence.step_id.value, None);
    assert_eq!(
        evidence.fields.value.get("reason").map(String::as_str),
        Some("autonomy")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("blocking_cap_source")
            .map(String::as_str),
        Some(expected_source)
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("blocking_cap_reason")
            .map(String::as_str),
        Some(expected_reason)
    );
    assert!(
        foundation.audit_chain().events().iter().any(|event| {
            event.surface == "foundry.autonomy.decision" && event.decision == "DENY"
        })
    );
}

pub fn assert_evidence_field(foundation: &Foundation, field: &str, expected: &str) {
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .last()
        .expect("autonomy denial records evidence");
    assert_eq!(
        evidence.fields.value.get(field).map(String::as_str),
        Some(expected),
        "unexpected evidence field {field}"
    );
}
