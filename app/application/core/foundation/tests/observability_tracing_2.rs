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
fn capability_invocation_redacts_forbidden_data_class_labels_from_spans() {
    let mut foundation =
        Foundation::default().with_invocation_trace_observer(TracingCapabilityInvocationObserver);
    foundation
        .onboard_tenant(TenantRegistration {
            tenant_id: "ten_forbidden_observability".into(),
            legal_name: "Forbidden Observability Tenant".into(),
            home_region: "region-home".into(),
            residency_class: "strict_home_region".into(),
            regulatory_packs: vec!["pack-alpha".into()],
            autonomy_ceiling: AutonomyTier::T2Advisory,
        })
        .expect("tenant can be onboarded");
    foundation
        .upsert_identity(IdentityRegistration {
            tenant_id: "ten_forbidden_observability".into(),
            user_id: "usr_forbidden_obs_admin".into(),
            primary_identifier: "forbidden-obs@example.test".into(),
            display_name: "Forbidden Observability Admin".into(),
            roles: vec!["tenant-admin".into()],
        })
        .expect("identity is valid");
    support::allow_capability_invocation(
        &mut foundation,
        "ten_forbidden_observability",
        "tenant-admin",
    );
    support::seed_passing_eval(&mut foundation, "cap.obs.phi");
    let capability = foundation
        .register_capability(CapabilityRegistration {
            capability_id: "cap.obs.phi".into(),
            namespace: "observability".into(),
            action: CapabilityAction::Other,
            required_tier: AutonomyTier::T2Advisory,
            touched_privacy_data_classes: application_foundation::privacy_data_classes_from(&[
                DataClass::InternalOnly,
                DataClass::Phi,
            ])
            .unwrap(),
            evidence_topic: "oya.foundry.capability.invoked.observability".into(),
        })
        .expect("capability is valid");
    foundation
        .grant_capability_to_tenant(TenantCapabilityGrant {
            tenant_id: "ten_forbidden_observability".into(),
            capability_id: capability.id.clone(),
            mcp_visible: true,
        })
        .expect("tenant can be licensed for capability");

    let capture = CaptureLayer::default();
    let subscriber = Registry::default().with(capture.clone());
    tracing::subscriber::with_default(subscriber, || {
        let denied = foundation
            .invoke_capability_as_principal(
                support::principal(
                    "ten_forbidden_observability",
                    "usr_forbidden_obs_admin",
                    AutonomyTier::T2Advisory,
                ),
                CapabilityInvocationRequest {
                    tenant_id: "ten_forbidden_observability".into(),
                    user_id: "usr_forbidden_obs_admin".into(),
                    capability_id: capability.id.clone(),
                    purpose: Purpose::SearchIndex,
                    subject_class: SubjectClass::Adult,
                    budget_window_id: "2026-05".into(),
                    projected_cost_micros: 10,
                    started_at_epoch_seconds: 1_000,
                },
            )
            .expect_err("PHI search indexing is denied by the data boundary");
        assert_eq!(denied, FoundationError::DataUseNotAllowed);
    });

    let spans = capture.captured_spans();
    let invocation_span = spans
        .iter()
        .find(|span| span.name == "foundry.capability.invoke")
        .expect("capability invocation span is emitted before data-boundary denial");
    assert_eq!(
        invocation_span
            .fields
            .get(fields::DATA_CLASSES_TOUCHED)
            .map(String::as_str),
        Some("FORBIDDEN_DATA_CLASS_PRESENT")
    );
    let captured = format!("{spans:?}{:?}", capture.captured_events());
    assert!(!captured.contains("PHI"));
    assert!(!captured.contains("forbidden-obs@example.test"));
    assert!(!captured.contains("Forbidden Observability Admin"));
}
