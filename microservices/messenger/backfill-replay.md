---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-messenger
deciders: axis-messenger, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/messenger/PRD.md
  - microservices/messenger/capacity-model.md
  - microservices/messenger/contracts/asyncapi/messenger-events.yaml
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (messenger µservice)

## Purpose

Specify how messenger handles two scenarios:

1. **Backfill** — search index rebuild from canonical message store
   (e.g., after a Meilisearch corruption or a new analyser shipped).
2. **Replay** — re-fanout of historical events to a newly subscribed
   downstream consumer (audit-chain, workflow-engine, mail action-card
   processor), or to replay missed events for a tenant onboarded mid-stream.

## Backfill (search index rebuild)

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- messenger backfill-search --tenant <t> --from <iso> --to <iso>`.
- Auto: corruption-detector emits `SearchIndexCorruptionDetected` event;
  worker picks up + backfills the affected partition.

Procedure:

1. Acquire backfill lease in Redis (per tenant per index partition; lease TTL = 1h).
2. Snapshot Postgres `messenger_messages` rows in `(tenant_id, channel_id)`
   partition, ordered by `posted_at`.
3. Stream rows in batches of 1000 → Meilisearch `addDocuments` (idempotent
   on `message_id`).
4. After bulk, emit `SearchIndexBackfilled` event with tuple
   `(tenant_id, partition, row_count, completed_at, signature)`.
5. Per-pack retention: backfill window bounded by retention floor
   (no backfilling beyond what retention allowed).

### Constraints

- Backfill is read-only against Postgres; no row mutation.
- Cedar policy + redaction (per `policy/redaction-phi.md`) applied at
  document-emission time, so PHI-stripped fields stay stripped.
- Per-tenant rate limit: 1 active backfill per partition; cluster cap = 8
  concurrent backfills cluster-wide.
- Cost (per `cost-budget.md`): roughly $0.10 per 1M messages backfilled
  (Postgres read + Meilisearch index + S3 audit log).

### Verification

- Integration test: corrupt Meilisearch index → backfill → search returns
  identical hits to Postgres `SELECT`.
- Idempotency: re-running same backfill produces no duplicate documents
  (`message_id` is primary key).

### Runbook

See [`runbooks/search-index-rebuild.md`](runbooks/search-index-rebuild.md)
(Slice-A authored).

## Replay (event fanout)

### Contract

Triggers:
- New µservice consumer onboards (e.g., new Workflow event consumer) and
  needs catch-up.
- Tenant onboarded mid-stream needs historical events for audit-chain seal
  rebuild.
- Bug-fix in event payload schema; re-emit corrected version.

Procedure:

1. Operator invokes: `cargo run -p oya-dev-cli -- messenger replay-events --tenant <t> --from <iso> --to <iso> --consumer <id>`.
2. CLI requires 2-person rule + ops-security approval (replay-events can
   re-trigger side-effects on consumers; must be audit-trail-bounded).
3. Worker scans `messenger_messages` + `messenger_audit_events` tables
   in `[from, to]` window; emits each as a Workflow event with
   `replay=true` label, `original_event_ts=<...>`.
4. Consumers MUST honour the `replay` label (idempotent processing on
   `(event_id, tenant_id)` tuple); failure to do so is the consumer's bug.
5. Audit-chain seal: replay emits sealed `EventReplayed` records per
   batch (one per 1000 events).

### Constraints

- Replay does NOT mutate the original event records; it appends fresh
  copies with `replay=true`.
- Replay window bounded by retention floor.
- Workflow consumers MUST be idempotent; replay-unsafe consumers are
  declared via `consumer_metadata.idempotent: false` and refuse the
  replay (worker logs + emits warning).
- Personal-context messages cannot be replayed beyond client-side scope
  (since server has no plaintext); replay of Personal-DM events emits
  the encrypted blob ref + signature only, no body.

### Verification

- Integration test: synthetic consumer with idempotency tracking; verify
  replay of 10k events produces no duplicate side-effects.
- Audit-chain integrity: replay event seals link to original; chain
  reconstructable end-to-end.

### Runbook

See [`runbooks/event-replay.md`](runbooks/event-replay.md) (Slice B).

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill search index (per 1M messages) | per-corruption | ~$0.10 |
| Replay events (per 10k events × 1 consumer) | per-onboard | ~$0.05 |
| Replay events (per 1M events × all consumers) | per-bugfix-replay | ~$5.00 |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill quality bounded by retention floor; cannot recover deleted +
  retention-purged content.
- Replay quality bounded by audit-chain seal availability; events older
  than the seal-archival horizon (24mo cold-tier) cannot be replayed.
- Personal-context body replay is impossible by design (E2E plaintext
  never lived server-side).
- Federation replay (Matrix-bridge) is bounded by the remote homeserver's
  retention — not controlled by oyatie.

## References

- `microservices/messenger/PRD.md`.
- `microservices/messenger/capacity-model.md`.
- `microservices/messenger/cost-budget.md`.
- `microservices/messenger/contracts/asyncapi/messenger-events.yaml`.
- ADR-0028 audit-chain.
- ADR-0126 (Connect dual-context, parallel).
- ADR-0131 (per-microservice flat layout).
