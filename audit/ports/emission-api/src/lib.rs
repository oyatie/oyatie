//! Audit-chain emission API DTOs.
//!
//! Wave 15-IMPL-truth-up scaffold (2026-05-21). Full request/response surface
//! mapped to `AuditEventEmitAppRequest` / `AuditEventEmitSuccessResponse` lives
//! in IP-005.
#![allow(dead_code)]

use audit_emission_kernel::ChainCoordinate;

/// Surface constant emitted at the `audit-chain.audit-event-emit` endpoint.
pub const AUDIT_EVENT_EMIT_SURFACE: &str = "audit-chain.audit-event-emit";

/// Outbox topic for emitted audit events per ADR-0145 inter-µservice contract.
pub const AUDIT_EVENT_TOPIC: &str = "oya.platform.audit";

/// Producer-side request envelope. Full schema in IP-005.
#[derive(Clone, Debug)]
pub struct AuditEventEmitRequest {
    pub coordinate: ChainCoordinate,
    pub event_id: String,
    pub payload_digest: String,
    pub idempotency_key: String,
}

/// Producer-side response envelope. Full schema in IP-005.
#[derive(Clone, Debug)]
pub struct AuditEventEmitResponse {
    pub audit_id: String,
    pub period_id: String,
    pub merkle_root_ref: String,
}
