//! Foundry audit port: the product-owned seam through which the write
//! spine's facts — applied Actions and refusals alike — leave Foundry
//! for the platform audit chain.
//!
//! The port owns its event type and depends on nothing: what Foundry
//! considers auditable is Foundry's contract, and binding it to the
//! platform's wire shapes would couple every future adapter to one
//! sink's schema. The reference in-memory sink gives the contract an
//! executable meaning.
#![forbid(unsafe_code)]
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// What became of the audited submission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditDisposition {
    /// The Action applied at this per-tenant ordinal.
    Applied { ordinal: u64 },
    /// The Action was refused at the named gate; no ordinal was spent.
    Denied { gate: String },
}

/// One auditable Foundry fact, validated at construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoundryAuditEvent {
    pub tenant_id: String,             // data_class: INTERNAL_ONLY
    pub audit_event_type: String,      // data_class: INTERNAL_ONLY
    pub principal_id: String,          // data_class: INTERNAL_ONLY
    pub decision_id: String,           // data_class: INTERNAL_ONLY
    pub object_ref: String,            // data_class: INTERNAL_ONLY
    pub disposition: AuditDisposition, // data_class: INTERNAL_ONLY
    pub occurred_at_epoch_ms: u64,     // data_class: INTERNAL_ONLY
}

/// Why an event or emission was refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditPortError {
    Empty {
        field: &'static str,
    },
    NotTrimmed {
        field: &'static str,
    },
    /// The sink refused the emission; the detail is diagnostic.
    Sink {
        detail: String,
    },
}

impl FoundryAuditEvent {
    /// Construct a validated event; every refusal is fail-closed.
    pub fn new(
        tenant_id: impl Into<String>,
        audit_event_type: impl Into<String>,
        principal_id: impl Into<String>,
        decision_id: impl Into<String>,
        object_ref: impl Into<String>,
        disposition: AuditDisposition,
        occurred_at_epoch_ms: u64,
    ) -> Result<Self, AuditPortError> {
        let event = Self {
            tenant_id: tenant_id.into(),
            audit_event_type: audit_event_type.into(),
            principal_id: principal_id.into(),
            decision_id: decision_id.into(),
            object_ref: object_ref.into(),
            disposition,
            occurred_at_epoch_ms,
        };
        let gate = match &event.disposition {
            AuditDisposition::Denied { gate } => Some(gate.clone()),
            AuditDisposition::Applied { .. } => None,
        };
        for (field, value) in [
            ("tenant_id", &event.tenant_id),
            ("audit_event_type", &event.audit_event_type),
            ("principal_id", &event.principal_id),
            ("decision_id", &event.decision_id),
            ("object_ref", &event.object_ref),
        ] {
            check(field, value)?;
        }
        if let Some(gate) = gate {
            check("gate", &gate)?;
        }
        Ok(event)
    }

    /// A deterministic identity for this event — the dedup key every
    /// sink honors. Derived only from the event's own facts.
    pub fn event_id(&self) -> String {
        let discriminant = match &self.disposition {
            AuditDisposition::Applied { ordinal } => format!("applied_{ordinal}"),
            AuditDisposition::Denied { gate } => format!("denied_{gate}"),
        };
        format!(
            "fae_{}_{}_{}",
            self.tenant_id, discriminant, self.occurred_at_epoch_ms
        )
    }
}

fn check(field: &'static str, value: &str) -> Result<(), AuditPortError> {
    if value.trim().is_empty() {
        return Err(AuditPortError::Empty { field });
    }
    if value.trim() != value {
        return Err(AuditPortError::NotTrimmed { field });
    }
    Ok(())
}

/// The emission seam: accept one event, idempotently by
/// [`FoundryAuditEvent::event_id`]. Re-emitting an identical event is
/// not an error; divergent content under the same identity is.
pub trait AuditSink {
    fn emit(&mut self, event: FoundryAuditEvent) -> Result<(), AuditPortError>;
}

/// The reference in-memory sink: the contract's executable meaning.
#[derive(Debug, Default)]
pub struct MemoryAuditSink {
    events: Vec<FoundryAuditEvent>,
}

impl MemoryAuditSink {
    pub fn events(&self) -> &[FoundryAuditEvent] {
        &self.events
    }
}

impl AuditSink for MemoryAuditSink {
    fn emit(&mut self, event: FoundryAuditEvent) -> Result<(), AuditPortError> {
        if let Some(stored) = self
            .events
            .iter()
            .find(|stored| stored.event_id() == event.event_id())
        {
            if *stored == event {
                return Ok(());
            }
            return Err(AuditPortError::Sink {
                detail: "divergent content under one event identity".into(),
            });
        }
        self.events.push(event);
        Ok(())
    }
}
