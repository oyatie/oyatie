---
id: ADR-0153
status: Superseded
superseded_by: [ADR-709]
---

# ADR-0153: Outbox Pattern

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture, axis-foundry, axis-messenger,
  axis-mail, axis-social, axis-tasks, axis-calendar
- Tier-A hyperscaler pattern: Stripe transactional outbox + Uber Cadence

## Context

When a request handler must persist state AND emit an event, the
naive implementation does two writes — one to the DB, one to the
event bus. The two writes are not atomic; any failure between them
yields a divergent system state:

- DB written, event lost → consumers missed the change.
- Event published, DB rollback → consumers see a phantom change.

Stripe's billing platform, Uber's Cadence, Confluent's e-commerce
patterns, and every serious distributed-systems shop use the
transactional outbox to avoid this dual-write hazard.

## Decision

Adopt the transactional outbox pattern as the ONLY canonical way
for a µservice to emit an event accompanying a state change.

1. The canonical spec is `docs/standards/outbox-pattern-canonical.md`.
2. The trait surface lives in `crates/oya-shared-outbox-pattern-kernel/`.
3. Every µservice with event-emission requirements creates one
   outbox table per bounded context.
4. The handler write-path appends the outbox row IN THE SAME
   transaction as the aggregate mutation.
5. A publisher worker drains the outbox FIFO and stamps
   `published_at` on each row.
6. Direct `event_bus.publish(...)` outside the outbox is FORBIDDEN.

Initial rollout: five charter cross-cutting carriers (messenger,
mail, social, tasks, calendar) declare `outbox_required: true` in
their PRDs first; remaining 28 µservices follow as their event
contracts go live.

## Consequences

Positive:
- Atomic state-change + event-emission.
- Replay-safe (the outbox is the source of truth for missed events).
- Stripe-grade event correctness.

Negative:
- Per-µservice outbox table + publisher worker.
- Eventual consistency (publisher lag is bounded but non-zero).

## Alternatives considered

- Dual-write to (DB, bus) — REJECTED, no atomicity.
- Distributed transactions / 2PC — REJECTED, well-known scalability
  trap (Pat Helland, "Life Beyond Distributed Transactions").
- CDC (Debezium) — VALID alternative; we may layer Debezium on top
  later for cross-DB capture, but the in-process outbox is the
  default contract.

## References

- microservices.io — Transactional Outbox.
- Stripe billing outbox (engineering blog).
- Uber Cadence — transactional event emission.
- docs/standards/outbox-pattern-canonical.md.
- crates/oya-shared-outbox-pattern-kernel/.
