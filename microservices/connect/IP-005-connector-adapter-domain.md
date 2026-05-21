---
ip_id: IP-005
title: "IP-005: connector-adapter domain crate"
microservice: connect
bounded_context: connector-adapter
layers: [domain]
acceptance_status: design-ready
date: 2026-05-20
related_adrs: [ADR-0056, ADR-0105, ADR-0145, ADR-0243, ADR-0246, ADR-0248, ADR-0254, ADR-0296]
companion_docs:
  - microservices/connect/catalog/oya-connect-connector-adapter-domain.yaml
  - microservices/connect/capabilities/connector-invoke.yaml
  - microservices/connect/runbooks/connector-cascade-failure.md
  - microservices/connect/runbooks/connector-rate-limit-saturation.md
doc_status: published
---

# IP-005: connector-adapter domain crate

## Purpose

Implement `oya-connect-connector-adapter-domain` — the per-connector typed action invocation engine with credential sidecar integration, circuit-breaker, exponential-backoff retry-with-jitter, per-connector rate-limit enforcement, and DLQ routing on exhaustion.

## Acceptance criteria

1. `ConnectorAdapterService::invoke(tenant_id, connector, action, payload, idempotency_key)` resolves credentials via sidecar (access token ≤60s TTL from OpenBao), invokes vendor API over HTTP/3 (fallback HTTP/2 → HTTP/1.1), returns canonicalized response.
2. Circuit-breaker: per-tenant per-connector; opens when error_rate > 50% over 300s window (sliding); half-open after 60s; emits `ConnectorCircuitOpen` / `ConnectorCircuitClosed` events.
3. Retry: exponential backoff 1s → 2s → 4s → 8s → 16s with ±25% jitter; max 5 attempts; non-retryable errors (4xx except 429) go to DLQ immediately.
4. Rate-limit: per-connector `RateLimitProfile` (token-bucket); vendor `Retry-After` header honored; `ConnectorRateLimited` event emitted.
5. DLQ routing: after retry exhaustion → `retry-and-dlq-domain::enqueue(tenant_id, connector, action, payload, error_class)`.
6. PagerDuty connector: `emergency_services_class=true` → bypass circuit-breaker open state (always attempt); bypass rate-limit cap (elevated floor); emit `EmergencyServicesConnectorInvoked` audit event.
7. Kata pod isolation: the adapter process runs inside Cloud Hypervisor + Kata per ADR-0254; sidecar credential access never crosses pod boundary.
8. Library-first Cedar eval per ADR-0246 for `connector:invoke` action gate.

## Key types

```rust
pub struct ConnectorInvokeRequest {
    pub tenant_id: TenantId,
    pub connector_name: ConnectorName,
    pub action_name: ActionName,
    pub payload: serde_json::Value,
    pub idempotency_key: IdempotencyKey,
    pub wiring_id: Option<WiringId>,
    pub emergency_services_class: bool,
}

pub enum InvokeResult {
    Success { response: serde_json::Value, latency_ms: u64 },
    VendorError { status: u16, body: String },
    RateLimited { retry_after_ms: u64 },
    CredentialError(CredentialError),
    DlqEnqueued { entry_id: DlqEntryId },
}

impl ConnectorAdapterService {
    pub async fn invoke(&self, req: ConnectorInvokeRequest) -> InvokeResult;
}
```

## Capacity math (Little's Law)

- Target: 50,000 concurrent tenant wirings × avg 1 action/s = 50,000 actions/s
- P99 vendor latency: 500ms → queue depth = 50,000 × 0.5 = 25,000 in-flight
- Provisioned: 100 adapter-worker pods × 500 concurrent Kata-isolated goroutines = 50,000 in-flight capacity
- 10× headroom (mass-event spike): 500 pods max via HPA

## Failure modes

1. **Sidecar unavailable** → return `CredentialError::SidecarUnavailable`; route to DLQ; emit `CredentialSidecarUnavailable` audit event.
2. **Vendor 500 storm** → circuit-breaker opens per-connector; DLQ accumulates; circuit auto-closes after 60s half-open attempt.
3. **PagerDuty vendor 500** → NEVER circuit-open; retry indefinitely (bounded by budget); secondary alert channel notified.
4. **Kata VM boot latency** → cold-start budget ≤500ms; pre-warmed Kata VMs per tenant cell.

## Definition of done

- [ ] Integration test: mock Salesforce API → full invoke → circuit-breaker trip → DLQ enqueue → circuit recovery
- [ ] Unit test: PagerDuty emergency-services bypass of circuit-breaker
- [ ] Load test: 50,000 actions/s for 60s → p99 ≤500ms
- [ ] `cargo clippy -- -D warnings` passes; ≥85% line coverage
