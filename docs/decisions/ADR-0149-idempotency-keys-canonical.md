---
id: ADR-0149
status: Accepted
---

# ADR-0149: Idempotency Keys Canonical

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture, axis-foundry, axis-all-microservices
- Tier-A hyperscaler pattern: Stripe idempotent requests + AWS ClientToken

## Context

Every state-changing REST/gRPC call across 33 microservices needs
duplicate-suppression discipline. Without an idempotency contract a
client retry (after timeout, after partial failure, after replay) can
double-charge a billing tenant, double-post a message, or double-emit
a workflow event. The Stripe API has shown for ten years that the
`Idempotency-Key` header is the canonical SaaS contract; AWS uses
the same pattern via `ClientToken` / `X-Amz-Idempotency-Token`.

PR #143 review (Fix-Agent-I) flagged the absence of a uniform
idempotency contract across oyatie µservices. Six µservices declare
`Idempotency-Key` in their OpenAPI; the remaining 27 do not.

## Decision

Adopt the canonical `Idempotency-Key` header as MANDATORY on every
state-changing REST operation in every oyatie microservice.

1. The canonical specification is
   `docs/standards/idempotency-keys-canonical.md`.
2. The trait surface lives in
   `crates/oya-shared-idempotency-key-kernel/`.
3. Every µservice OpenAPI 3.2.0 document declares the canonical
   `IdempotencyKey` parameter component AND references it from every
   `POST`/`PUT`/`PATCH`/`DELETE` operation.
4. Compliance is enforced by the new
   `oya-check-idempotency-key-coverage` gate, wired into
   `gate run-all`.

## Consequences

Positive:
- Safe retries across all 33 µservices.
- Stripe-grade duplicate-suppression contract.
- Outbox pattern (ADR-0153) layers cleanly on top.

Negative:
- Per-µservice idempotency store integration work (~33 µservices).
- Cache + DB pressure for keeping (key → response) records.

## Alternatives considered

- No idempotency contract — REJECTED, unsafe under retry.
- Per-µservice ad-hoc idempotency — REJECTED, no uniform observable
  contract.
- Server-generated keys only — REJECTED, breaks at-least-once
  client retry semantics.

## References

- Stripe API idempotency.
- AWS API design — ClientToken.
- docs/standards/idempotency-keys-canonical.md.
- crates/oya-shared-idempotency-key-kernel/.
- crates/oya-check-idempotency-key-coverage/.
- ADR-0145-inter-microservice-communication-reform.
- ADR-0153-outbox-pattern.
