// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod support;

use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use observability_domain::fields;
use observability_tracing_adapter::TracingCapabilityInvocationObserver;
use application_app::{
    AutonomyTier, CapabilityAction, CapabilityInvocationRequest, CapabilityRegistration,
    CostBudgetRegistration, DataClass, Foundation, FoundationError, IdentityRegistration, Purpose,
    SubjectClass, TenantCapabilityGrant, TenantRegistration,
};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, Registry};

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapturedSpan {
    name: String,
    fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CapturedEvent {
    fields: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
struct CaptureLayer {
    spans: Arc<Mutex<BTreeMap<u64, CapturedSpan>>>,
    events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl CaptureLayer {
    fn captured_spans(&self) -> Vec<CapturedSpan> {
        self.spans
            .lock()
            .expect("captured spans lock is not poisoned")
            .values()
            .cloned()
            .collect()
    }

    fn captured_events(&self) -> Vec<CapturedEvent> {
        self.events
            .lock()
            .expect("captured events lock is not poisoned")
            .clone()
    }
}

impl<S> Layer<S> for CaptureLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::default();
        attrs.record(&mut visitor);
        self.spans
            .lock()
            .expect("captured spans lock is not poisoned")
            .insert(
                id.into_u64(),
                CapturedSpan {
                    name: attrs.metadata().name().to_string(),
                    fields: visitor.fields,
                },
            );
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::default();
        values.record(&mut visitor);
        if let Some(span) = self
            .spans
            .lock()
            .expect("captured spans lock is not poisoned")
            .get_mut(&id.into_u64())
        {
            span.fields.extend(visitor.fields);
        }
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldVisitor::default();
        event.record(&mut visitor);
        self.events
            .lock()
            .expect("captured events lock is not poisoned")
            .push(CapturedEvent {
                fields: visitor.fields,
            });
    }
}

#[derive(Default)]
struct FieldVisitor {
    fields: BTreeMap<String, String>,
}

impl Visit for FieldVisitor {
    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}

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
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
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
            touched_privacy_data_classes: application_app::privacy_data_classes_from(&[
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
