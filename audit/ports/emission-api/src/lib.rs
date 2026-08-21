//! Audit-chain emission API DTOs.
//!
//! Producer-side request/response envelopes for the
//! `audit-chain.audit-event-emit` surface. The validation and canonical-encoding
//! rules these DTOs are carried through live in `audit/core/emission-domain`;
//! the wire schema is `audit/contracts/openapi/audit-chain.yaml`.
#![allow(dead_code)]

use audit_emission_kernel::ChainCoordinate;

/// Surface constant emitted at the `audit-chain.audit-event-emit` endpoint.
pub const AUDIT_EVENT_EMIT_SURFACE: &str = "audit-chain.audit-event-emit";

/// Outbox topic for emitted audit events per ADR-0145 inter-µservice contract.
pub const AUDIT_EVENT_TOPIC: &str = "oya.platform.audit";

/// Producer-side request envelope. Validated by `audit/core/emission-domain`.
#[derive(Clone, Debug)]
pub struct AuditEventEmitRequest {
    pub coordinate: ChainCoordinate,
    pub event_id: String,
    pub payload_digest: String,
    pub idempotency_key: String,
}

/// Producer-side response envelope.
#[derive(Clone, Debug)]
pub struct AuditEventEmitResponse {
    pub audit_id: String,
    pub period_id: String,
    pub merkle_root_ref: String,
}
