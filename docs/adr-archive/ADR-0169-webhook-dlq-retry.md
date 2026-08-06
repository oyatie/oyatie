---
id: ADR-0169
status: Superseded
deciders: council-architecture, ops-sre-reliability, axis-eventing, council-api-sdk
date: 2026-05-18
owner: axis-eventing
supersedes: []
superseded_by: [ADR-0709]
amended_by: [ADR-0632]
related: [ADR-0005, ADR-0011, ADR-0037, ADR-0040, ADR-0145, ADR-0167, ADR-0168]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/per-microservice-flat-layout.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0169 — Webhook DLQ + exponential-backoff retry (Stripe webhook subscriptions pattern)

## Status

Accepted (2026-05-18). Authorizes a canonical webhook-delivery substrate shared across every µservice that pushes outbound webhooks to tenants, with standardized DLQ, exponential-backoff retry, circuit-breaker, and per-tenant endpoint registry. Tier C "nice-to-have" hyperscaler pattern per `/specs/hyperscaler-architecture-invariants.json` audit Row C3.

## ADR-0632 product-protocol reconciliation

Tenant webhook delivery and retry remain public HTTPS REST documented by OpenAPI 3.2.0 plus signed/versioned webhooks, with AsyncAPI/CloudEvents events, SSE, or WebSocket used only where their delivery semantics apply. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. The former tenant-requested gRPC adapter is narrowed to internal-only gRPC/proto3 over HTTP/2 and cannot be selected as a public webhook carrier.

## Context

Multiple Oyatie µservices need to push outbound webhooks to tenant-supplied URLs: workflow run completions, messenger message arrival, social post events, Foundry capability completions, ontology mutation events, audit-chain seal notifications. Today each µservice that wants to deliver webhooks would need to reimplement:

- HTTP delivery with timeout
- Exponential-backoff retry on 5xx / network errors
- Dead-letter queue for terminal failures
- Per-tenant endpoint registration + secret rotation
- HMAC-SHA256 request signing (Stripe-Signature header pattern)
- Replay protection (timestamp + nonce)
- Circuit breaker per tenant endpoint (so a slow tenant doesn't starve other tenants' deliveries)
- Tenant-facing UI/API for "list failed deliveries" and "retry from DLQ"

The canonical hyperscaler references are well-established:

- **Stripe Webhooks** — https://stripe.com/docs/webhooks — HMAC-SHA256 signing, exponential backoff up to 3 days, replayable from dashboard.
- **GitHub Webhooks** — exponential backoff 8 retries over ~8.5 hours; tenant-redeliverable from the UI.
- **Slack Events API** — 3 retries with 1s/4s/30s backoff; documented response semantics.
- **AWS EventBridge → API destinations** — exponential-retry with jitter; built-in DLQ via SQS.
- **PagerDuty Events API v2** — 4 retries with backoff; dedup keys.

If every µservice reimplements this substrate independently, we get N inconsistent behaviors, N CVE surfaces, N tenant-facing inconsistencies, and a fleet-wide failure surface that the SRE on-call cannot reason about uniformly. This is precisely the "every µservice integrates the canonical trait" pattern recently established in ADR-0145 for inter-µservice communication.

## Decision

Oyatie introduces a SHARED webhook-delivery kernel (`crates/oya-shared-webhook-delivery-kernel/`) that every µservice with outbound-webhook needs integrates. The kernel owns:

1. **Delivery trait** — `WebhookDeliveryClient::deliver(endpoint, event, idempotency_key) -> DeliveryReceipt`.
2. **Retry schedule** — exponential backoff `1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 512s, 1024s, 2048s, 4096s` (~75min total); jitter ±20% per Stripe/AWS guidance; max retries: 13. After exhaustion, delivery enters DLQ.
3. **HMAC-SHA256 signing** — `Oya-Signature: t=<ts>,v1=<hex-hmac>` header (Stripe-Signature parity); secret rotated per-tenant on demand.
4. **Replay protection** — timestamp in signature; tenant servers reject signatures older than 5min.
5. **DLQ** — Postgres table `webhook_dlq` per-µservice; rows include failure history (status code + body excerpt for each attempt); tenant-API to list + retry.
6. **Circuit breaker** — per-endpoint; opens after 50 consecutive failures within 5min; half-open after 60s; full retry after sustained success. Per-tenant isolation: one endpoint's failure does not starve other tenants.
7. **Per-tenant endpoint registry** — Postgres table `webhook_endpoint`; tenant API to register/rotate/disable.
8. **Tenant API** — `POST /v1/webhook_endpoints`, `GET /v1/webhook_endpoints/<id>/deliveries`, `POST /v1/webhook_endpoints/<id>/deliveries/<delivery_id>/retry`.

Each µservice that emits outbound webhooks:

- Declares its event catalog (per-µservice `contracts/webhook-events.json`).
- Integrates the `WebhookDeliveryClient` trait via the shared kernel.
- Emits webhook-delivery events to the kernel; the kernel handles the rest.
- Owns its DLQ table (per-µservice Postgres database — cell architecture per ADR-0009).

### Retry schedule rationale

Stripe documents 3-day total retry window with capped intervals. We choose 13 retries over ~75 minutes because:

- Most transient failures resolve within 5 minutes (industry telemetry).
- Three-day retention in active retry queues bloats Postgres state.
- After 75 minutes, the tenant should resolve manually via DLQ-replay (operator surface).
- DLQ retention is 30 days (configurable per-tenant up to 90 days for paid tiers per ADR-0013).

### Idempotency

Every webhook carries an `Oya-Event-Id: evt_<base32>` header. Tenant servers MUST treat duplicate `Oya-Event-Id` values as idempotent — Stripe's documented pattern.

### Per-tenant circuit breaker isolation

Critical: a slow/down tenant endpoint MUST NOT starve other tenants' webhook deliveries. Implementation:

- Per-endpoint goroutine/task pool with bounded concurrency.
- When circuit breaker opens, the endpoint's queue drains into the cooldown bucket but other endpoints continue unaffected.
- Postgres advisory locks per-endpoint to prevent dogpile retries across multiple delivery workers.

## Alternatives considered

### A. Synchronous webhooks (no retry, no queue)
- Pros: zero infrastructure; emitter and tenant directly coupled.
- Cons: one slow tenant blocks the emitter; no retry on transient failure; no DLQ; not the hyperscaler shape. Stripe explicitly built async + retry because synchronous broke at scale.
- **Rejected**: blocks producer; loses events on transient failure; tenant ergonomics fail.

### B. Manual operator retry only (no automatic retry)
- Pros: simple; no exponential-backoff scheduling needed.
- Cons: every transient failure becomes an SRE pager; tenant ergonomics fail; not the hyperscaler shape.
- **Rejected**: ops burden; tenant-side failure UX (every customer waits for human retry).

### C. Per-µservice independent reimplementation
- Pros: each µservice owns its delivery semantics fully.
- Cons: N inconsistent retry schedules, N CVE surfaces for HMAC signing bugs, N tenant-API shapes for "list deliveries", N replay-protection implementations. Fleet-wide ops nightmare. Violates the "shared substrate where uniformity matters" pattern from ADR-0145.
- **Rejected**: violates uniformity-where-it-matters principle; fleet-wide ops cost dominates.

### D. AWS EventBridge as the substrate (managed service)
- Pros: zero in-house code; mature retry + DLQ + signing.
- Cons: AWS-bound (violates ADR-0121 hyperscaler-portable invariant for EU + KR cells); webhook secrets leave Oyatie's residency boundary (ADR-0008 data-use boundary); per-event cost at scale dominates in-house Postgres queue cost.
- **Rejected**: provider lock-in + residency violation.

### E. RabbitMQ / Kafka + per-µservice consumer pattern
- Pros: mature queue tech; well-understood retry semantics via consumer-side delay queues.
- Cons: each µservice still implements the delivery loop, signing, DLQ rendering, tenant API — Kafka is a transport, not a webhook-substrate. We need the BEHAVIORAL contract, not just the queue.
- **Partial accept**: we already use the outbox pattern (ADR-0005) over our event backbone; the webhook-delivery kernel SITS ON TOP of that, providing the behavioral contract.

## Consequences

### Positive

1. **Hyperscaler-parity** — Oyatie's webhook surface matches Stripe / GitHub / Slack / AWS EventBridge tenant expectations. Audit Row C3 closed.
2. **One CVE surface** — HMAC signing bugs caught in one crate, not N. Same for replay-protection, signature-rotation, and DLQ rendering.
3. **Tenant ergonomics uniform** — every Oyatie webhook has the same `Oya-Signature` header, same `Oya-Event-Id` idempotency contract, same DLQ-replay API.
4. **Per-tenant isolation** — circuit breaker prevents one slow tenant from starving others (the dominant scale issue in webhook delivery systems).
5. **Tenant CLI integration** — `oya webhook list-deliveries`, `oya webhook retry <delivery-id>` via ADR-0167 tenant CLI.

### Negative

1. **Shared crate as a bottleneck** — every webhook-emitting µservice depends on `oya-shared-webhook-delivery-kernel`. Coordinator-locks-shared-crate gotcha applies (per Pipeline-Clog memory; mitigate via the canonical shared-crate update protocol).
2. **DLQ Postgres footprint** — DLQ tables grow up to 90 days × delivery volume. Estimate: 30k webhooks/day × 13 retry rows × 30 days = ~12M rows per µservice peak. Postgres partitioning by month required.
3. **Per-tenant secret rotation surface** — Stripe-style secret rotation (overlap window) requires endpoint API support; Tier-A semver per ADR-0037.

### Operational

1. `crates/oya-shared-webhook-delivery-kernel/` is the canonical trait surface (this ADR's skeleton).
2. Public delivery adapters use HTTPS HTTP/1.1 (default) or HTTP/2 (long-poll-friendly tenants); gRPC is not a tenant-selectable webhook carrier.
3. Every µservice with outbound webhooks integrates via a 30-line wiring change.
4. DLQ-replay SLO: tenant-initiated retry executes within 60s p99.
5. Webhook-delivery telemetry: per-tenant deliveries/sec, retry-rate, DLQ-fill-rate, p99 e2e latency. Exposed via the observability µservice per ADR-0139.

### Tenant-facing API shape (Tier-A)

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/v1/webhook_endpoints` | register endpoint URL + secret rotation token |
| `GET` | `/v1/webhook_endpoints` | list registered endpoints for the tenant |
| `GET` | `/v1/webhook_endpoints/{id}` | endpoint detail |
| `PATCH` | `/v1/webhook_endpoints/{id}` | rotate secret, enable/disable, update URL |
| `DELETE` | `/v1/webhook_endpoints/{id}` | unregister endpoint |
| `GET` | `/v1/webhook_endpoints/{id}/deliveries` | list deliveries with status filter |
| `GET` | `/v1/webhook_endpoints/{id}/deliveries/{delivery_id}` | delivery detail incl. all retry attempts |
| `POST` | `/v1/webhook_endpoints/{id}/deliveries/{delivery_id}/retry` | replay a single delivery from DLQ |
| `POST` | `/v1/webhook_endpoints/{id}/secret_rotations` | begin secret rotation (dual-sign window) |

The shape mirrors Stripe's `/v1/webhook_endpoints` REST surface for tooling parity.

### Signature scheme

The `Oya-Signature` header carries one or more comma-separated values:

```
Oya-Signature: t=1715900000,v1=hex-sha256-hmac,v1_rotated=hex-sha256-hmac
```

- `t` — unix timestamp of dispatch.
- `v1` — HMAC-SHA256 of `t.body` keyed on the active secret.
- `v1_rotated` — only present during a secret-rotation window; signed with the prior secret. Tenant servers MAY validate either v1 to accept the request during rotation.

Replay window: ≤300s. Tenants reject signatures older than the window.

### Secret-rotation lifecycle

1. Tenant calls `POST /v1/webhook_endpoints/{id}/secret_rotations` → kernel emits new secret; tenant stores it.
2. Kernel signs subsequent deliveries with BOTH the old and new secret (dual-sign window: tenant-configurable, default 72h).
3. Tenant confirms cutover via `POST /v1/webhook_endpoints/{id}/secret_rotations/{rotation_id}/confirm`.
4. Kernel stops signing with the old secret.

This is the Stripe-documented rotation pattern; we adopt it directly.

### Failure-mode catalog

| Failure | Detection | Response |
|---|---|---|
| Endpoint returns 5xx | per-delivery | retry per schedule |
| Endpoint returns 4xx (except 408/429) | per-delivery | DO NOT retry; emit to DLQ immediately (Stripe behavior) |
| Endpoint returns 429 | per-delivery | retry per schedule with Retry-After honored |
| Endpoint times out (>30s) | per-delivery | retry per schedule; classify as transient |
| Endpoint fails 50 consecutive | per-endpoint | open circuit breaker; pause deliveries 60s; half-open probe |
| Endpoint domain DNS-fails | per-delivery | retry per schedule; classify as transient |
| TLS certificate invalid | per-delivery | DO NOT retry; emit to DLQ with `tls_invalid` reason |
| Signature secret missing in vault | per-delivery | hard-fail; alert ops; DO NOT retry until vault repaired |

### Performance budgets

- p99 first-attempt delivery latency from emit-event to HTTP request issued: ≤2s.
- p99 e2e to tenant-server-200-OK: ≤5s (tenant-server-bound).
- DLQ insert latency: ≤100ms p99.
- DLQ-replay API: ≤60s p99 from API call to retry attempt issued.
- Circuit-breaker state propagation across delivery workers: ≤5s p99.

### Migration / rollout plan

1. M01 slice: shared-kernel crate skeleton + trait surface (this ADR's companion).
2. M01.5: HTTP-1.1 adapter + DLQ Postgres tables; one µservice (workflow) integrates as the pilot.
3. M02: remaining webhook-emitting µservices integrate; tenant-API surface goes Tier-A.
4. M02.5: `oya webhook list-deliveries` + `oya webhook retry` CLI commands ship (ADR-0167).
5. M03: optional internal-only gRPC/proto3 over HTTP/2 adapter for sibling-service delivery orchestration; never tenant-facing.

## References

- Stripe Webhooks — https://stripe.com/docs/webhooks — canonical reference; HMAC-SHA256 signing, exponential backoff, retry-from-dashboard.
- Stripe Webhook Signatures — https://stripe.com/docs/webhooks/signatures — `Stripe-Signature` header format we parity in `Oya-Signature`.
- GitHub Webhook Retry — https://docs.github.com/en/webhooks/about-webhooks — 8 retries over ~8.5h; redeliver from UI.
- Slack Events API — https://api.slack.com/apis/events-api — 3 retries with documented backoff.
- AWS EventBridge → API destinations — https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-api-destinations.html — built-in DLQ + retry.
- PagerDuty Events API v2 — https://developer.pagerduty.com/docs/events-api-v2/overview/ — dedup-key idempotency; 4 retries.
- RFC 2104 — HMAC: Keyed-Hashing for Message Authentication — signing primitive.
- Hystrix circuit-breaker pattern — Netflix — per-endpoint failure isolation.
- ADR-0005 — eventing backbone outbox pattern (transport layer this kernel sits on top of).
- ADR-0011 — cross-microservice contract registry (webhook event catalogs registered here).
- ADR-0037 — public API stability tiers (webhook-endpoint API is Tier-A).
- ADR-0040 — progressive delivery canary (DLQ + circuit-breaker integrate with canary-rollback signal).
- ADR-0145 — inter-microservice communication reform ("every µservice integrates the canonical trait" precedent).
- ADR-0167 — tenant-facing CLI (`oya webhook ...` commands integrate here).
- ADR-0168 — public status page (webhook-delivery health surfaces here).
- `/specs/hyperscaler-architecture-invariants.json` — audit Row C3 closes here.
