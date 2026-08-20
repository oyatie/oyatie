---
doc_class: Standard
title: X-Request-Id Propagation (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: council-architecture
deciders: council-architecture, axis-foundry, axis-observability
related_adrs: [ADR-0151, ADR-0145]
review_cadence: annually
doc_status: published
---

# X-Request-Id Propagation (Canonical)

## Authority

ADR-0151-request-id-propagation landed this contract. AWS request IDs
(`x-amzn-RequestId`) and GCP request IDs are the industry references.
Every inter-microservice call in oyatie MUST propagate the canonical
`X-Request-Id` header alongside the W3C `traceparent` OpenTelemetry
trace context.

## Contract

### 1. Header format

```
X-Request-Id: <uuidv7>
```

- UUIDv7 per ADR-0709 D-1, which names request IDs explicitly. The earlier
  text said ULID and cited ADR-0156 — that ADR is the PII registry decision and
  says nothing about identifiers.
- 36-character canonical hyphenated form, with the version nibble pinned to 7.
  The machine-readable declaration is
  `governance/check/id-discipline/id-discipline-policy.json`.

### 2. Propagation rules

- Inbound request: if the header is absent, the µservice's edge
  middleware GENERATES a fresh ULID and stamps it on the
  per-request context.
- Outbound call: the per-µservice middleware injects the active
  request id into every inter-µservice HTTP/gRPC call.
- Response: the same `X-Request-Id` is echoed in every outbound
  response (the request-id MUST be the SAME across request +
  response, so clients can correlate).

### 3. Relationship to OpenTelemetry traceparent

`X-Request-Id` is a request-scoped correlation id consumed by humans
(operators, support, audit) and by Loki log queries. It is DISTINCT
from the W3C `traceparent` (which is consumed by Tempo). Both MUST
be propagated together.

| Header           | Consumer              | Cardinality   |
|------------------|-----------------------|---------------|
| `X-Request-Id`   | Loki + audit-chain    | per-request   |
| `traceparent`    | Tempo                 | per-span      |
| `Idempotency-Key`| idempotency store     | per-action    |

### 4. Cardinality discipline

`request_id` is HIGH-CARDINALITY. It MUST NEVER appear as a
Prometheus / Mimir metric label (per
`microservices/observability/contracts/metric-naming-convention.md`).
It MAY appear as:
- a Tempo span attribute (low pressure; span-scoped).
- a Loki log field (line-scoped).
- an audit-chain seal field.

### 5. Per-µservice middleware spec

Every µservice MUST integrate a `RequestIdMiddleware` at the
inbound HTTP edge and `RequestIdClientInterceptor` at every outbound
client (gRPC + HTTP). The implementation lives in
`crates/oya-shared-request-id-kernel`.

### 6. Validation

The `oya-check-request-id-propagation` gate (planned; see
adr-follow-ups.yaml) audits that:
- every µservice REST adapter declares the
  `X-Request-Id` middleware.
- every cross-µservice client adapter declares the
  request-id interceptor.

## References

- AWS Request IDs — `x-amzn-RequestId`.
- GCP — request-id propagation.
- ADR-0151-request-id-propagation.
- ADR-0145-inter-microservice-communication-reform.
- microservices/observability/contracts/metric-naming-convention.md.
