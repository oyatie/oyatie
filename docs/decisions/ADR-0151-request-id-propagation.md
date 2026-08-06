---
id: ADR-0151
status: Accepted
---

# ADR-0151: X-Request-Id Propagation

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture, axis-foundry, axis-observability
- Tier-A hyperscaler pattern: AWS x-amzn-RequestId

## Context

Distributed tracing (OpenTelemetry traceparent + Tempo) covers
per-span analysis but not human-correlated request analysis.
Operators, support staff, and audit reviewers need a single short
identifier per request to correlate logs, alerts, audit-chain
seals, and customer reports.

AWS uses `x-amzn-RequestId`; GCP uses request-scoped IDs in stackdriver.
oyatie has the OpenTelemetry traceparent header (per ADR-0145
Invariant 2) but no separate request-id correlation header. Mixing
these concerns at the metric level (e.g. tagging Mimir metrics with
`request_id`) is the canonical high-cardinality anti-pattern.

## Decision

Adopt the canonical `X-Request-Id` header (ULID) propagated alongside
OpenTelemetry `traceparent` on every inter-µservice call.

1. The canonical spec is
   `docs/standards/request-id-canonical.md`.
2. Every µservice's edge middleware GENERATES a fresh ULID if the
   header is absent, and PROPAGATES it on every outbound call.
3. Every µservice's outbound HTTP/gRPC client adapter INJECTS the
   request-id on every cross-µservice call.
4. `request_id` is FORBIDDEN as a Prometheus/Mimir metric label
   (high-cardinality); it MAY appear as a Tempo span attribute or
   Loki log field only.
5. `microservices/observability/contracts/metric-naming-convention.md`
   is updated to codify the cardinality rule.

## Consequences

Positive:
- Single short id per request for ops + audit.
- Clean separation from per-span trace context.
- High-cardinality discipline preserved in Mimir.

Negative:
- Extra header on every inter-µservice call (~26 bytes).
- Per-µservice middleware integration work.

## Alternatives considered

- Reuse traceparent — REJECTED, span ids change per child, breaking
  single-id correlation.
- Tag request_id on every metric — REJECTED, cardinality explosion.
- W3C baggage header — DEFERRED; X-Request-Id ships first, baggage
  can layer in later.

## References

- AWS Request IDs — x-amzn-RequestId.
- Google Cloud — request-id propagation.
- W3C Trace Context — traceparent.
- docs/standards/request-id-canonical.md.
- ADR-0145-inter-microservice-communication-reform.
