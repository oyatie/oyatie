// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use application_foundation::{
    AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, Foundation, FoundationError, IdentityRegistration, Purpose,
    SubjectClass, TenantCapabilityGrant, TenantRegistration,
};
use observability_domain::fields;
use observability_tracing_adapter::TracingCapabilityInvocationObserver;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};

mod observability_tracing_fixtures;

use observability_tracing_fixtures::*;

#[test]
fn capability_invocation_emits_safe_observability_span_fields() {
    let mut foundation =
        Foundation::default().with_invocation_trace_observer(TracingCapabilityInvocationObserver);
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_observability".into(),
            legal_name: "Observability Tenant".into(),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    foundation
        .bind_cell("ten_observability", "region-home-a", "cell-observability-a")
        .expect("tenant can be cell-bound before invocation");
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_observability".into(),
            user_id: "usr_observability_admin".into(),
            primary_identifier: "obs@example.test".into(),
            display_name: "Observability Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(&mut foundation, "ten_observability", "tenant-admin");
    support::seed_passing_eval(&mut foundation, "cap.obs.invoke");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.obs.invoke".into(),
            namespace: "observability".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                DataClass::InternalOnly,
                DataClass::BehavioralTenantProduct,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked.observability".into(),
        })
        .expect("capability is valid");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_observability".into(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("tenant can be licensed for capability");
    foundation
        .grant_data_use(
            "ten_observability",
            Purpose::CapabilityInvocation,
            support::privacy_data_class(DataClass::BehavioralTenantProduct),
        )
        .expect("usage data class grant is recorded before dispatch");
    foundation
        .configure_tenant_cost_budget(CostBudgetRegistration {
            tenant_id: "ten_observability".into(),
            capability_id: None,
            window_id: "2026-05".into(),
            monthly_limit_micros: 1_000,
            per_invocation_limit_micros: 100,
            warning_threshold_percent: 80,
        })
        .expect("cost budget is configured before dispatch");

    let capture = CaptureLayer::default();
    let subscriber = Registry::default().with(capture.clone());
    tracing::subscriber::with_default(subscriber, || {
        foundation
            .invoke_capability_as_principal(
                support::principal(
                    "ten_observability",
                    "usr_observability_admin",
                    AutonomyTier::T2Advisory,
                ),
                CapabilityInvocationRequest {
                    tenant_id: "ten_observability".into(),
                    user_id: "usr_observability_admin".into(),
                    capability_id: capability.id.clone(),
                    purpose: Purpose::CapabilityInvocation,
                    subject_class: SubjectClass::Adult,
                    budget_window_id: "2026-05".into(),
                    projected_cost_micros: 10,
                    started_at_epoch_seconds: 1_000,
                },
            )
            .expect("licensed, budgeted capability invokes");
    });

    let spans = capture.captured_spans();
    let invocation_span = spans
        .iter()
        .find(|span| span.name == "foundry.capability.invoke")
        .expect("capability invocation span is emitted");
    assert_eq!(
        invocation_span
            .fields
            .get(fields::SERVICE_NAME)
            .map(String::as_str),
        Some("foundation-app")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::TENANT_ID)
            .map(String::as_str),
        Some("ten_observability")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::TENANT_REGION)
            .map(String::as_str),
        Some("region-home")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::CELL_ID)
            .map(String::as_str),
        Some("cell-observability-a")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::CELL_BOUND)
            .map(String::as_str),
        Some("true")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::CAPABILITY_ID)
            .map(String::as_str),
        Some("cap.obs.invoke")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::DATA_CLASSES_TOUCHED)
            .map(String::as_str),
        Some("INTERNAL_ONLY,BEHAVIORAL_TENANT_PRODUCT")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::AUTONOMY_TIER)
            .map(String::as_str),
        Some("T2")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::GEN_AI_OPERATION_NAME)
            .map(String::as_str),
        Some("capability_invocation")
    );
    assert_eq!(
        invocation_span
            .fields
            .get(fields::GEN_AI_PROVIDER_NAME)
            .map(String::as_str),
        Some("foundry")
    );

    let events = capture.captured_events();
    assert!(events.iter().any(|event| {
        event
            .fields
            .get(fields::INVOCATION_RESULT)
            .map(String::as_str)
            == Some("started")
    }));
    assert!(events.iter().any(|event| {
        event
            .fields
            .get(fields::INVOCATION_RESULT)
            .map(String::as_str)
            == Some("succeeded")
    }));

    let captured = format!("{spans:?}{events:?}");
    assert!(!captured.contains("obs@example.test"));
    assert!(!captured.contains("foundation-local-provider"));
    assert!(!captured.contains("provider-call:"));
    assert!(!captured.contains("Observability Admin"));
}
