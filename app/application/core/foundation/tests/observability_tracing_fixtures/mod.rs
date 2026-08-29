// ADR-0083 Tier 3: integration tests use `.unwrap()` / `.expect()` /
// `.expect_err()` / `.unwrap_err()` to assert invariants — Tier 3 exemption.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedSpan {
    pub name: String,
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedEvent {
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
pub struct CaptureLayer {
    pub spans: Arc<Mutex<BTreeMap<u64, CapturedSpan>>>,
    pub events: Arc<Mutex<Vec<CapturedEvent>>>,
}

impl CaptureLayer {
    pub fn captured_spans(&self) -> Vec<CapturedSpan> {
        self.spans
            .lock()
            .expect("captured spans lock is not poisoned")
            .values()
            .cloned()
            .collect()
    }

    pub fn captured_events(&self) -> Vec<CapturedEvent> {
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
pub struct FieldVisitor {
    pub fields: BTreeMap<String, String>,
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
