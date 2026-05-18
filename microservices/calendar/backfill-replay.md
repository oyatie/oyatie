---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: calendar
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-calendar
deciders: axis-calendar, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0135, ADR-0131, ADR-CAL-0001, ADR-CAL-0002, ADR-CAL-0004]
related_artifacts:
  - microservices/calendar/PRD.md
  - microservices/calendar/capacity-model.md
  - microservices/calendar/contracts/asyncapi/calendar-events.yaml
  - microservices/calendar/runbooks/calendar-restore.md
  - microservices/calendar/runbooks/availability-cache-rebuild.md
  - microservices/calendar/runbooks/rsvp-storm-throttle.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (calendar µservice)

## Purpose

Specify how calendar handles three scenarios:

1. **Free/busy cache rebuild** — Redis availability-resolver cache
   rebuild from canonical event store (after corruption, after a tzdb
   bump per ADR-CAL-0004, after Cedar policy change that changes
   cross-tenant disclosure rules).
2. **Recurrence materialisation rebuild** — re-expansion of all
   RRULE-bound events in a window (after RRULE engine upgrade per
   ADR-CAL-0002, after tzdb refresh).
3. **Event-lifecycle replay** — re-fanout of historical event-
   lifecycle events to a newly subscribed downstream consumer
   (audit-chain, workflow-engine, observability, ontology), or to
   replay missed events for a tenant onboarded mid-stream.

## Free/busy cache rebuild

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- calendar backfill-freebusy --tenant <t> --from <iso> --to <iso>`.
- Auto: cache-corruption-detector emits `FreeBusyCacheCorruptionDetected` event; the
  availability-resolver-worker picks up + backfills the affected partition.
- Auto: tzdb refresh worker (per ADR-CAL-0004) emits `TzdbReleaseChanged`;
  the availability-resolver-worker invalidates affected partitions and rebuilds.

Procedure:

1. Acquire backfill lease in Redis (per tenant per partition; lease TTL = 1h).
2. Enumerate Postgres `calendar_events` rows in `(tenant_id, context, starts_at_year_month)` partition.
3. For each event, expand recurrence per the (potentially new) RRULE engine.
4. Compute the free/busy projection for the event's window (respecting context isolation).
5. Bulk-write to Redis with idempotency key `(tenant_id, context, attendee_id, slot_hash)`.
6. Emit `FreeBusyCacheBackfilled` event with tuple
   `(tenant_id, partition, row_count, completed_at, signature)`.
7. Per-pack retention: backfill window bounded by retention floor.

### Performance

- Backfill rate target: ≥1000 events/sec per partition.
- A 100k-event tenant rebuilds in <2 minutes.
- Avoid `runbooks/availability-cache-rebuild.md` triggering — backfill should be the slow path, not the panic path.

## Recurrence materialisation rebuild

### Trigger sources

- RRULE engine version bump (per ADR-CAL-0002 — when `rrule-rs` LTS
  pin moves and the named edge-case test matrix asserts a behaviour
  change).
- tzdb refresh (per ADR-CAL-0004) — DST rule change may shift
  recurrence-bound occurrences.
- Operator-invoked: `cargo run -p oya-dev-cli -- calendar
  rebuild-recurrence --tenant <t> --window <yyyy-mm-dd..yyyy-mm-dd>`.

### Contract

1. Per-tenant lock on the recurrence-engine for the window.
2. Enumerate all events with non-null RRULE in the window.
3. Re-expand each per the (new) engine.
4. Compare to prior materialisation; emit a per-event diff record.
5. Write the new materialisation to Postgres.
6. Emit `RecurrenceMaterialisationRebuilt` with diff summary
   `(tenant_id, window, event_count, diff_count)`.
7. For events with diff_count > 0 in the next 30 days, emit
   `RecurrenceMaterialisationRebuildAffectedAttendees` to notify the
   organiser; tenant policy decides whether to auto-update invitations.

### Bounded materialisation

Per PRD AC-10, no expansion may exceed 5y horizon. Backfill
explicitly refuses windows > 5y.

## Event-lifecycle replay

### Trigger sources

- New downstream consumer onboarded (e.g., a new audit-chain instance
  needs to replay 30d of history).
- Tenant onboarded mid-stream (the Workflow-engine binding needs to
  catch up on the tenant's recent event history).
- Consumer requests replay for a specific time window for debugging
  / forensics.

### Contract

1. Acquire replay lease per (tenant_id, consumer_id) — replay leases
   are exclusive to prevent double-delivery.
2. Snapshot the `calendar_event_lifecycle.v1` event log in
   `(tenant_id, partition_id)` partition, ordered by `event_id`.
3. Stream events in batches of 1000 → consumer's Workflow webhook
   (idempotent per `event_id`).
4. After bulk, emit `EventLifecycleReplayCompleted` with tuple
   `(tenant_id, consumer_id, event_count, completed_at, signature)`.
5. Per-pack retention: replay window bounded by retention floor (no
   replay of expired-retention events).

### Performance

- Replay rate: ≥1000 events/sec per consumer.
- Idempotency: every replayed event carries `(event_id, replay_attempt_n, original_emitted_at)`
  for the consumer to deduplicate.

## RSVP fanout replay

### Special case

If the invitation-flow worker has been paused (e.g., per
`runbooks/rsvp-storm-throttle.md` Case C) and a backlog has accumulated,
the same replay procedure applies with two adjustments:

1. Replay is throttled to the storm-throttle rate (default 100 rps).
2. Replayed RSVPs are deduplicated by `(invitation_id, attendee_id,
   decided_at)`; per the RFC 5546 §5.2 last-write-wins rule, only
   the latest `decided_at` for any (invitation, attendee) tuple
   actually mutates state. Prior decideds are emitted as historical
   audit events without state effect.

## Per-µservice consumer contracts

| Downstream | Replay onboarding | Replay catch-up window |
|---|---|---|
| `audit-chain` | replay from tenant onboarding | full retention horizon (per-pack; up to 5y for KR-FSS) |
| `workflow-engine` | replay last 30d on consumer onboarding | 30d default; configurable to 90d |
| `observability` | replay last 24h on consumer onboarding | 24h default; configurable to 7d |
| `mail` (invitation bridge) | no replay (mail is downstream-only; lost invitations are NOT replayed because they may have already been delivered out-of-band) | n/a |
| `ontology` | replay last 7d on consumer onboarding | 7d default |
| `tenancy` | no replay (tenancy is upstream-only) | n/a |

## Verification

- [ ] Backfill / replay rate ≥1000 events/sec measured in
  benchmark `cargo bench -p oya-calendar-event-store-worker -- backfill`.
- [ ] Backfill idempotency property test passes —
  `cargo nextest run -p oya-calendar-event-store-domain -- backfill_idempotent`.
- [ ] Replay window bounded by retention — `cargo nextest run -p
  oya-calendar-event-store-domain -- replay_retention_bound`.
- [ ] RSVP replay deduplication — `cargo nextest run -p
  oya-calendar-invitation-flow-domain -- rsvp_replay_dedupe`.

## References

- ADR-0028 — Audit-chain (Ed25519 + Merkle).
- ADR-CAL-0002 — RRULE engine; replay implications.
- ADR-CAL-0004 — tzdb refresh; cache invalidation.
- `microservices/calendar/contracts/asyncapi/calendar-events.yaml`.
- `microservices/calendar/runbooks/calendar-restore.md` (referenced for full-restore path).
- `microservices/calendar/runbooks/availability-cache-rebuild.md` (referenced for cache rebuild path).
- `microservices/calendar/runbooks/rsvp-storm-throttle.md` (referenced for storm throttling).
- `microservices/messenger/backfill-replay.md` — sibling reference.
