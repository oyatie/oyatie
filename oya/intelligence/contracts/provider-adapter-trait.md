---
doc_class: ContractSpec
title: Provider Adapter Trait
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence
related_adrs: [ADR-0255, ADR-0145, ADR-0296]
doc_status: published
---

# Provider Adapter Trait

## Purpose

Define the Rust trait every provider adapter implements. The trait is the port boundary between
the `model-routing-kernel` and the per-provider adapter (`providers-adapter-anthropic`,
`providers-adapter-openai`, etc.). The trait constrains adapters to the canonical envelope shape
and the audit-tap commit invariant.

## Trait surface (Rust)

```rust
use std::time::Duration;

use async_trait::async_trait;
use futures::Stream;

use crate::{
    domain::dispatch_request::DispatchRequest,
    domain::dispatch_response::{DispatchResponseChunk, DispatchResponseSummary},
    domain::routing_decision::RoutingDecision,
    domain::secret_reference::CredentialHandle,
    error::AdapterError,
};

#[async_trait]
pub trait ProviderAdapter: Send + Sync + 'static {
    /// Provider name (lowercase kebab-case; matches Provider enum).
    fn name(&self) -> &'static str;

    /// Provider region (e.g., "us-east-1", "eu-frankfurt-1"); some adapters are multi-region.
    fn region(&self) -> &'static str;

    /// Modalities this adapter supports.
    fn modalities(&self) -> &'static [Modality];

    /// Whether the adapter supports BAA (HIPAA), FedRAMP, KR-CSAP, etc.
    fn compliance_flags(&self) -> ComplianceFlags;

    /// Cost-floor per 1M tokens; used by routing kernel for cost-based selection.
    fn cost_per_million_tokens(&self, model: &str) -> Option<CostFloor>;

    /// Health probe; returns latency p99 + budget remaining.
    async fn health(&self) -> Result<ProviderHealth, AdapterError>;

    /// Issue a single-shot dispatch; returns the full DispatchResponseSummary.
    /// Adapters MUST commit audit-tap before returning (atomicity).
    async fn issue_single(
        &self,
        request: DispatchRequest,
        handle: CredentialHandle,
        routing: RoutingDecision,
    ) -> Result<DispatchResponseSummary, AdapterError>;

    /// Issue a streaming dispatch; returns an async stream of chunks.
    /// Final chunk MUST carry the cost record + provider's audit-tap delta + signature.
    async fn issue_stream(
        &self,
        request: DispatchRequest,
        handle: CredentialHandle,
        routing: RoutingDecision,
    ) -> Result<impl Stream<Item = Result<DispatchResponseChunk, AdapterError>>, AdapterError>;

    /// Provider-specific timeout (default: 60s; reasoning models may extend).
    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }

    /// Provider-specific QPS budget (token-bucket client-side throttle).
    fn qps_budget(&self) -> u32;

    /// Adapter-side circuit-breaker state.
    fn circuit_state(&self) -> CircuitState;
}
```

## Invariants

### I-01: Credential never enters adapter memory

The `CredentialHandle` is opaque; adapters MUST NOT log it, serialise it, or hold a reference to
its underlying value. The handle is consumed at HTTP-call assembly time by the OpenBao sidecar.

### I-02: Audit-tap atomicity

Adapters MUST emit the audit-tap event before returning to the caller. The dispatch-usecase
orchestrator wraps the adapter call with audit-tap commit; an adapter failure pre-commit refunds
the cost record and emits a `DispatchFailed` event.

### I-03: Untrusted-content delimiter

Adapters MUST honour `PromptPart::untrusted_content == true` by injecting the provider's
delimited-input convention (e.g., Anthropic XML tags `<user_message>...</user_message>`; OpenAI
`additional_user_input` field).

### I-04: Modality coverage disclosure

Adapters MUST refuse modalities not declared in `modalities()` rather than silently degrade.

### I-05: Cost-floor honesty

`cost_per_million_tokens()` MUST return the actual upstream price for the requested model.

### I-06: Provider rate-limit response

Adapters MUST distinguish 429 (rate-limit) from 5xx (provider error) and return the appropriate
error variant; the dispatch-usecase orchestrator routes accordingly per
`runbooks/provider-rate-limit-saturation.md` + `provider-outage-*.md`.

### I-07: Streaming chunk schema

Streaming chunks MUST be valid `DispatchResponseChunk` values; partial-JSON-decoded chunks not
permitted. The adapter accumulates incomplete provider chunks before emitting.

### I-08: Provider error pass-through with classification

Adapters MUST classify provider errors into the substrate's `AdapterError` enum (rate_limit,
auth_failure, invalid_request, server_error, timeout, content_policy_violation,
context_length_exceeded, model_not_found, ...) rather than passing raw provider errors.

## Per-provider adapter conformance test

`tests/provider_adapter_conformance.rs` runs a fixed suite against every implementation:

- Single-shot dispatch with text modality returns DispatchResponseSummary.
- Streaming dispatch yields ≥ 1 chunk.
- Untrusted-content delimiter survives a round-trip.
- 429 + 5xx + timeout each map to distinct AdapterError variants.
- Health probe returns within 5s.
- Cost-floor returns a positive value for at least one declared model.
- Credential handle is consumed only via the sidecar path (assert no `.value` access).

## References

- ADR-0255 — Intelligence as two-layer AI Substrate.
- ADR-0145 — Inter-microservice communication reform (direct gRPC + 3 invariants).
- ADR-0296 — Sidecar credential-handle.
- `microservices/intelligence/ARCHITECTURE.md` §3 (Library-first dispatch flow).
- `microservices/intelligence/runbooks/sidecar-credential-handle-expired.md`.
