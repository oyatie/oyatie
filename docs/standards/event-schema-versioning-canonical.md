---
doc_class: Standard
title: Event Schema Versioning (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: council-architecture
deciders: council-architecture, axis-foundry, axis-all-microservices
related_adrs: [ADR-0154]
review_cadence: annually
doc_status: published
---

# Event Schema Versioning (Canonical)

## Authority

ADR-0154-event-schema-versioning landed this contract. Every event
emitted by any oyatie microservice MUST carry an explicit `version`
field; consumers MUST honor backward-compatibility per Postel's law.

References: AWS EventBridge schema registry, Confluent Schema Registry,
Stripe's `api_version` envelope.

## Contract

### 1. Every event carries a version field

Every event payload, in every transport (AsyncAPI WebSocket, AMQP,
NATS, Kafka), MUST include:

```json
{
  "event_id": "<ulid>",
  "event_kind": "<oyatie.<microservice>.<verb>.<aggregate>>",
  "version": "1.2.0",
  "tenant_id": "<ulid>",
  "occurred_at": "<rfc3339>",
  "data": { ... }
}
```

`version` follows SemVer:
- MAJOR — breaking change (consumer adapters MUST be updated).
- MINOR — additive field (consumers tolerant per Postel's law).
- PATCH — fix that does not alter semantics.

### 2. Backward-compatibility rules

- Adding optional fields → MINOR bump.
- Adding required fields → MAJOR bump (forbidden in MINOR).
- Removing fields → MAJOR bump.
- Changing semantics of an existing field → MAJOR bump.

### 3. AsyncAPI declaration

Every AsyncAPI 3.1.0 message schema MUST declare:

```yaml
components:
  messages:
    MessagePosted:
      headers:
        type: object
        required: [event_id, event_kind, version, tenant_id, occurred_at]
        properties:
          event_id: { type: string, format: ulid }
          event_kind: { type: string, pattern: "^oya\\." }
          version: { type: string, pattern: "^[0-9]+\\.[0-9]+\\.[0-9]+$" }
          tenant_id: { type: string, format: ulid }
          occurred_at: { type: string, format: date-time }
      payload:
        $ref: "#/components/schemas/MessagePosted_v1_0"
```

Each message MUST link to a version-suffixed schema:
`<MessageName>_v<MAJOR>_<MINOR>` so consumers can target a specific
schema.

### 4. Multi-version coexistence

When a MAJOR bump happens:
- Old version events keep emitting until cutover.
- Producer carries `producer_emits_versions: [1.x, 2.x]` for the
  overlap window (≥ 30 days).
- After cutover, old MAJOR is sunset per ADR-0061
  (no-silent-regression).

### 5. Schema registry

Schemas are stored on-disk in
`microservices/<ms>/contracts/asyncapi/<ms>-events.yaml`. A schema
registry µservice (planned; ADR-0155) will index versions at runtime.

### 6. Validation

The `check-event-schema-versioning` gate enforces that every
AsyncAPI 3.1.0 schema in every µservice declares the canonical
`version` header.

## References

- AWS EventBridge Schema Registry.
- Confluent Schema Registry — Subject Strategy.
- AsyncAPI 3.1.0 spec — Message Versioning.
- ADR-0154-event-schema-versioning.
