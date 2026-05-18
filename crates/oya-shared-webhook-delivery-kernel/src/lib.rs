//! Webhook-delivery kernel — per-µservice trait surface for ADR-0169
//! webhook DLQ + retry substrate.
//!
//! # ADR-0169 contract
//!
//! Every µservice that delivers outbound webhooks integrates this trait.
//! The kernel owns: signed delivery (HMAC-SHA256 with `Oya-Signature`
//! header), exponential-backoff retry (1s → 4096s, 13 attempts ≈ 75min),
//! per-endpoint circuit breaker, per-tenant endpoint registry, DLQ
//! Postgres tables, and the tenant-facing API surface declared in
//! ADR-0169 §"Tenant-facing API shape (Tier-A)".
//!
//! # Skeleton scope
//!
//! This crate ships the TRAIT SURFACE only. Production impl (HTTP-1.1
//! adapter, signature scheme, secret-rotation dual-sign, circuit
//! breaker, DLQ Postgres tables, replay API) tracked under
//! `registry/placeholder-debt/adr-follow-ups.yaml#adr-0169-webhook-impl`.
//!
//! # Naming justification
//!
//! `oya-shared-webhook-delivery-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:webhook-delivery>-<layer:kernel>`. The
//! `shared` axis is the canonical Oyatie identifier for cross-µservice
//! substrate per feedback_glossary_shared_not_platform.
//!
//! # References
//!
//! - ADR-0169 — webhook DLQ + retry (this trait surface).
//! - ADR-0005 — eventing backbone outbox pattern (transport layer this
//!   kernel sits on top of).
//! - ADR-0056 — port-in-kernel (this crate is layer=kernel).
//! - Stripe Webhooks — https://stripe.com/docs/webhooks (signature +
//!   retry semantics this kernel parities).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::time::Duration;

/// Logical tenant identifier the webhook is dispatched on behalf of.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(pub String);

/// Per-tenant webhook endpoint identifier (Stripe `we_<base32>` parity).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EndpointId(pub String);

/// Per-delivery identifier (Stripe `evt_<base32>` parity); used as the
/// idempotency key on the tenant server side (ADR-0169 §"Idempotency").
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DeliveryId(pub String);

/// Event payload to deliver. Opaque bytes; the kernel signs and dispatches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WebhookPayload(pub Vec<u8>);

/// Result of a single delivery attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryReceipt {
    /// Tenant returned 2xx; delivery succeeded.
    Delivered { http_status: u16 },
    /// Tenant returned a non-retryable status (4xx except 408/429) per
    /// ADR-0169 §"Failure-mode catalog". Will not be retried.
    DeadLettered { http_status: u16, reason: String },
    /// Retryable failure; will be retried per ADR-0169 §"Retry schedule".
    Retrying {
        attempt: u8,
        next_attempt_in: Duration,
    },
}

/// Webhook-delivery client trait integrated by every µservice that
/// emits outbound webhooks per ADR-0169 §"Decision".
pub trait WebhookDeliveryClient: Send + Sync {
    /// Deliver `payload` to `endpoint` for `tenant`. Returns a
    /// `DeliveryReceipt` describing the immediate outcome; retries
    /// continue asynchronously per ADR-0169 §"Retry schedule".
    fn deliver(
        &self,
        tenant: &TenantId,
        endpoint: &EndpointId,
        delivery_id: &DeliveryId,
        payload: &WebhookPayload,
    ) -> DeliveryReceipt;

    /// Tenant-initiated DLQ replay per ADR-0169 §"Tenant-facing API shape".
    /// SLO ≤60s p99 per ADR-0169 §"Operational" #4.
    fn retry_from_dlq(
        &self,
        tenant: &TenantId,
        endpoint: &EndpointId,
        delivery_id: &DeliveryId,
    ) -> DeliveryReceipt;
}

/// No-op implementation provided so dependents can compile + smoke-test
/// against the trait before the production adapter lands. Production
/// adapter tracked under
/// `registry/placeholder-debt/adr-follow-ups.yaml#adr-0169-webhook-impl`.
pub struct NoopWebhookDeliveryClient;

impl WebhookDeliveryClient for NoopWebhookDeliveryClient {
    fn deliver(
        &self,
        _tenant: &TenantId,
        _endpoint: &EndpointId,
        _delivery_id: &DeliveryId,
        _payload: &WebhookPayload,
    ) -> DeliveryReceipt {
        DeliveryReceipt::DeadLettered {
            http_status: 0,
            reason: "noop-skeleton-per-adr-0169".to_string(),
        }
    }

    fn retry_from_dlq(
        &self,
        _tenant: &TenantId,
        _endpoint: &EndpointId,
        _delivery_id: &DeliveryId,
    ) -> DeliveryReceipt {
        DeliveryReceipt::DeadLettered {
            http_status: 0,
            reason: "noop-skeleton-per-adr-0169".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_dispatches_dead_letter() {
        let client = NoopWebhookDeliveryClient;
        let receipt = client.deliver(
            &TenantId("t_skel".into()),
            &EndpointId("we_skel".into()),
            &DeliveryId("evt_skel".into()),
            &WebhookPayload(b"{}".to_vec()),
        );
        assert!(matches!(receipt, DeliveryReceipt::DeadLettered { .. }));
    }
}
