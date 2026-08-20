---
doc_class: Standard
title: Outbox Pattern (Canonical)
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-18
owner_team: council-architecture
deciders: council-architecture, axis-foundry, axis-all-microservices
related_adrs: [ADR-0153, ADR-0149, ADR-0145]
review_cadence: annually
doc_status: published
---

# Outbox Pattern (Canonical)

## Authority

ADR-0153-outbox-pattern landed this contract. Every µservice that
performs a state change accompanied by event emission MUST use the
transactional outbox pattern. Dual-write to (database, event-bus) is
FORBIDDEN — it has no atomicity, no replay safety, and no audit
correctness.

References: Stripe billing outbox, Uber Cadence outbox, Linear's
transactional outbox.

## Contract

### 1. Outbox table

Every µservice with event-emission requirements creates one outbox
table per bounded context:

```sql
CREATE TABLE <bc>_outbox (
    outbox_id        UUID PRIMARY KEY,            -- UUIDv7 per ADR-0709 D-1
    aggregate_id     UUID NOT NULL,
    aggregate_kind   TEXT NOT NULL,
    event_kind       TEXT NOT NULL,
    event_version    TEXT NOT NULL,               -- ADR-0154
    payload          JSONB NOT NULL,
    headers          JSONB NOT NULL,              -- {idempotency_key, traceparent, request_id}
    occurred_at      TIMESTAMPTZ NOT NULL,
    published_at     TIMESTAMPTZ NULL,            -- NULL until publisher emits
    PRIMARY KEY (outbox_id),
    UNIQUE (aggregate_id, event_kind, event_version, headers->>'idempotency_key')
);
CREATE INDEX <bc>_outbox_unpublished_idx ON <bc>_outbox (occurred_at) WHERE published_at IS NULL;
```

### 2. Write path

```rust
let tx = pool.begin().await?;
// 1. Mutate the aggregate inside the SAME transaction
update_aggregate(&tx, ...).await?;
// 2. Append to outbox in the SAME transaction
insert_outbox_row(&tx, outbox_row).await?;
// 3. Commit atomically
tx.commit().await?;
```

The aggregate change and the outbox row land in ONE transaction.
If the bus is down, the outbox row stays as `published_at IS NULL`
and the publisher catches up.

### 3. Publisher path

A per-microservice publisher worker reads unpublished outbox rows in
`occurred_at` order, emits them to the event bus, and stamps
`published_at`. The publisher is at-least-once; consumers MUST be
idempotent (per ADR-0149 + replay contract).

### 4. Trait surface

```rust
pub trait OutboxStore: Send + Sync {
    type TxContext;
    fn append(&self, tx: &mut Self::TxContext, row: OutboxRow)
        -> Result<(), OutboxError>;
    fn next_unpublished(&self, batch_size: usize)
        -> Result<Vec<OutboxRow>, OutboxError>;
    fn mark_published(&self, ids: &[OutboxId])
        -> Result<(), OutboxError>;
}
```

Lives in `oya-shared-outbox-pattern-kernel`.

### 5. NO direct event emission

Inside a request handler, the canonical helper `OutboxStore::append`
is the ONLY way to "emit" an event. Direct
`event_bus.publish(...)` is FORBIDDEN — it has no atomicity with the
DB write.

### 6. Replay semantics

The outbox row's `idempotency_key` lets the publisher safely retry
without duplicate emission. Consumers see `at-least-once` delivery
and use the per-event `(aggregate_id, event_kind, idempotency_key)`
tuple as the dedup key.

### 7. Per-µservice declaration

Every µservice's PRD MUST declare `outbox_required: true|false` in
its PRD. The five charter cross-cutting carriers (messenger, mail,
social, tasks, calendar) declare `outbox_required: true` first; the
remaining 28 µservices follow as their event-emission contracts go
live.

## References

- Microservices.io — Transactional Outbox pattern.
- Chris Richardson, "Microservices Patterns" — Outbox chapter.
- Stripe billing outbox architecture (engineering blog).
- ADR-0153-outbox-pattern.
- ADR-0149-idempotency-keys-canonical.
- ADR-0145-inter-microservice-communication-reform.
