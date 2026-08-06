---
id: ADR-0154
status: Superseded
superseded_by: [ADR-709]
---

# ADR-0154: Event Schema Versioning

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture, axis-foundry, axis-all-microservices
- Tier-A hyperscaler pattern: AWS EventBridge schema registry + Confluent

## Context

Events are the wire-format between producers and consumers across the
33 µservices. As the system evolves, payload shapes change; without
explicit version discipline, a producer can break every consumer
silently (or worse — partial deserialization with corrupt downstream
state).

AWS EventBridge schema registry, Confluent Schema Registry, and
Stripe's `api_version` envelope all use explicit per-event versions
with backward-compatibility rules. AsyncAPI 3.1.0 (which oyatie
already adopts per ADR-0145) supports message versioning via
`bindings` + schema refs.

## Decision

Adopt explicit per-event `version` field as MANDATORY on every event
emitted across every channel (WebSocket, AMQP, NATS, Kafka).

1. The canonical spec is
   `docs/standards/event-schema-versioning-canonical.md`.
2. Every AsyncAPI 3.1.0 message envelope MUST declare the `version`
   header and `event_id` (ULID per ADR-0156).
3. Backward-compatibility rules follow SemVer:
   - MINOR — additive (consumers tolerant).
   - MAJOR — breaking (overlap window ≥ 30 days).
4. Compliance enforced by `oya-check-event-schema-versioning` gate.
5. A schema registry µservice is DEFERRED; the on-disk AsyncAPI
   documents are the source of truth until then.

## Consequences

Positive:
- Producers + consumers evolve independently with explicit contract.
- No silent breaking change.
- Aligns with ADR-0061 (no silent regression).

Negative:
- Per-µservice schema-version discipline.
- Multi-version coexistence overhead during MAJOR cutovers.

## Alternatives considered

- No version field — REJECTED, silent breaking change.
- Schema registry only — REJECTED for now; deferred until producer
  ecosystem stabilizes.
- Single global event-schema version — REJECTED, breaks per-event
  evolution.

## References

- AWS EventBridge Schema Registry.
- Confluent Schema Registry.
- AsyncAPI 3.1.0 — message versioning.
- docs/standards/event-schema-versioning-canonical.md.
- crates/oya-check-event-schema-versioning/.
- ADR-0061 (no silent regression).
