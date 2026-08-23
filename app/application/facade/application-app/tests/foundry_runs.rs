// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use application_app::{
    AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, EvidenceKind, Foundation, IdentityRegistration, Purpose,
    RunDisposition, RunState, StepDisposition, StepKind, StepState, SubjectClass,
    TenantCapabilityGrant, TenantRegistration,
};

#[test]
fn successful_capability_invocation_records_foundry_run_lifecycle() {
    let mut foundation = Foundation::default();
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_runs".into(),
            legal_name: "Runs Tenant".into(),
            home_region: "secondary-region".into(),
            residency_class: "global".into(),
            regulatory_packs: vec!["pack-secondary".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .unwrap();
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_runs".into(),
            user_id: "usr_runs_admin".into(),
            primary_identifier: "runs@example.test".into(),
            display_name: "Runs Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .unwrap();
    support::allow_capability_invocation(&mut foundation, "ten_runs", "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.demo.runs");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.demo.runs".into(),
            namespace: "demo".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
                DataClass::InternalOnly,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked.custom".into(),
        })
        .unwrap();
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_runs".into(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .unwrap();
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_runs".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .unwrap();

    let receipt = foundation
        .invoke_capability_as_principal(
            support::principal("ten_runs", "usr_runs_admin", AutonomyTier::T2Advisory),
            CapabilityInvocationRequest {
                tenant_id: "ten_runs".into(),
                user_id: "usr_runs_admin".into(),
                capability_id: capability.id,
                purpose: Purpose::CapabilityInvocation,
                subject_class: SubjectClass::Adult,
                budget_window_id: "2026-05".into(),
                projected_cost_micros: 10,
                started_at_epoch_seconds: 1_000,
            },
        )
        .expect("licensed, budgeted capability invokes");

    assert_eq!(receipt.run_id.as_deref(), Some("run_000000000001"));
    let runs = foundation.foundry_runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].state.value, RunState::Succeeded);
    assert_eq!(runs[0].disposition.value, Some(RunDisposition::Success));
    assert_eq!(
        runs[0].idempotency_key.value.as_str(),
        receipt.cost_reservation_id.as_deref().unwrap()
    );
    assert_eq!(
        receipt.foundry_step_id.as_deref(),
        Some("step_000000000001_000001")
    );
    let steps = foundation.foundry_steps();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].kind.value, StepKind::ProviderCall);
    assert_eq!(steps[0].provider_kind.value, "foundation-local");
    assert_eq!(steps[0].model_ref.value.as_deref(), Some("foundation-app"));
    assert_eq!(steps[0].state.value, StepState::Succeeded);
    assert_eq!(steps[0].disposition.value, Some(StepDisposition::Succeeded));
    assert_eq!(
        receipt.foundry_evidence_id.as_deref(),
        Some("ev_000000000002")
    );
    assert!(foundation.foundry_evidence_chain().verify());
    assert_eq!(foundation.foundry_evidence_chain().records().len(), 2);
    assert_eq!(
        foundation.foundry_evidence_chain().records()[0].kind.value,
        EvidenceKind::AutonomyDecision
    );
    let evidence = foundation
        .foundry_evidence_chain()
        .records()
        .iter()
        .find(|record| {
            record.evidence_id.value.as_str() == receipt.foundry_evidence_id.as_deref().unwrap()
        })
        .expect("capability invocation evidence exists");
    assert_eq!(evidence.evidence_id.value, "ev_000000000002");
    assert_eq!(
        evidence.evidence_id.value.as_str(),
        receipt.foundry_evidence_id.as_deref().unwrap()
    );
    assert_eq!(
        evidence.run_id.value.as_str(),
        receipt.run_id.as_deref().unwrap()
    );
    assert_eq!(
        evidence.step_id.value.as_deref(),
        receipt.foundry_step_id.as_deref()
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("audit_event_hash")
            .map(String::as_str),
        Some(receipt.evidence_event_hash.as_str())
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("cost_reservation_id")
            .map(String::as_str),
        receipt.cost_reservation_id.as_deref()
    );
    assert_eq!(
        evidence.fields.value.get("run_id").map(String::as_str),
        receipt.run_id.as_deref()
    );
    assert_eq!(
        evidence.fields.value.get("step_id").map(String::as_str),
        receipt.foundry_step_id.as_deref()
    );
    assert_eq!(
        evidence.fields.value.get("provider_id").map(String::as_str),
        Some("foundation-local")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_mode")
            .map(String::as_str),
        Some("Api")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_region")
            .map(String::as_str),
        Some("secondary-region")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_model_ref")
            .map(String::as_str),
        Some("foundation-app")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_call_idempotency_key")
            .map(String::as_str),
        Some("provider-call:run_000000000001:step_000000000001_000001:foundation-local:001")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_call_receipt_id")
            .map(String::as_str),
        Some(
            "provider-call-receipt:provider-call:run_000000000001:step_000000000001_000001:foundation-local:001"
        )
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_call_attempt")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_projected_cost_micros")
            .map(String::as_str),
        Some("10")
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("provider_p95_latency_ms")
            .map(String::as_str),
        Some("1")
    );
    assert!(
        !format!("{:?}", evidence.fields.value).contains("foundation-local-provider"),
        "provider receipt evidence must not expose secret reference names"
    );
    assert!(
        !format!("{:?}", evidence.fields.value).contains("provider_account"),
        "provider receipt evidence must not expose provider account fields"
    );
    assert_eq!(
        evidence
            .fields
            .value
            .get("evidence_topic")
            .map(String::as_str),
        Some("oya.foundry.capability.invoked.custom")
    );
    assert!(foundation.audit_chain().events().iter().any(|event| {
        event.surface == "oya.foundry.capability.invoked.custom"
            && event.decision == "ALLOW"
            && event.hash == receipt.evidence_event_hash
    }));
    for surface in [
        "foundry.cost-budget.reserve",
        "foundry.run.start",
        "foundry.provider.route",
        "foundry.provider.call",
        "foundry.step.emit",
        "foundry.capability.invoke",
        "foundry.run.complete",
        "foundry.evidence.emit",
        "foundry.evidence.topic.emit",
    ] {
        assert!(
            foundation
                .audit_chain()
                .events()
                .iter()
                .any(|event| event.surface == surface && event.decision == "ALLOW"),
            "missing ALLOW audit surface {surface}"
        );
    }
    let audit_events = foundation.audit_chain().events();
    let run_complete_index = audit_events
        .iter()
        .position(|event| event.surface == "foundry.run.complete" && event.decision == "ALLOW")
        .expect("run completion is audited");
    let capability_topic_index = audit_events
        .iter()
        .position(|event| event.hash == receipt.evidence_event_hash)
        .expect("receipt evidence topic audit is present");
    let topic_emit_index = audit_events
        .iter()
        .rposition(|event| {
            event.surface == "foundry.evidence.topic.emit" && event.decision == "ALLOW"
        })
        .expect("capability evidence topic emission is audited");
    let evidence_emit_index = audit_events
        .iter()
        .rposition(|event| event.surface == "foundry.evidence.emit" && event.decision == "ALLOW")
        .expect("capability evidence emission is audited");
    assert!(
        run_complete_index < capability_topic_index,
        "capability evidence topic audit must wait for run finalization"
    );
    assert!(
        run_complete_index < topic_emit_index,
        "success evidence topic emission must wait for run finalization"
    );
    assert!(
        run_complete_index < evidence_emit_index,
        "success evidence emission must wait for run finalization"
    );
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| { event.surface == "foundry.step.emit" && event.decision == "ALLOW" })
    );
    assert!(
        foundation
            .audit_chain()
            .events()
            .iter()
            .any(|event| { event.surface == "foundry.evidence.emit" && event.decision == "ALLOW" })
    );
    assert_eq!(foundation.outbox_records().len(), 2);
    let capability_outbox = foundation.outbox_records().last().unwrap();
    assert_eq!(
        capability_outbox.topic.value,
        "oya.foundry.capability.invoked.custom"
    );
    assert_eq!(
        capability_outbox.idempotency_key.value,
        receipt.foundry_evidence_id.unwrap()
    );
}
