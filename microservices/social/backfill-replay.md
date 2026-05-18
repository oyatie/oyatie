---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: social
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-social
deciders: axis-social, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/social/PRD.md
  - microservices/social/capacity-model.md
  - microservices/social/contracts/asyncapi/social-events.yaml
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (social µservice)

## Purpose

Specify how social handles these scenarios:

1. **Backfill** — search index + feed cache rebuild from canonical Postgres post + profile + follow-graph stores (e.g., after a Meilisearch corruption, Valkey flush, or new ranking version shipped).
2. **Replay** — re-fanout of historical events to a newly subscribed downstream consumer (audit-chain, workflow-engine, mail action-card processor, ontology), or to replay missed events for a tenant onboarded mid-stream.
3. **Federation replay** — bounded re-emission of historical Professional-tier ActivityPub outbox to a federation peer that requests re-sync.

## Backfill (search index rebuild)

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- social backfill-search --tenant <t> --from <iso> --to <iso>`.
- Auto: corruption-detector emits `SearchIndexCorruptionDetected` event; worker picks up + backfills the affected partition.

Procedure:

1. Acquire backfill lease in Valkey (per tenant per index partition; lease TTL = 1h).
2. Snapshot Postgres `social_posts` + `social_profiles` rows in `(tenant_id, context_kind)` partition, ordered by `posted_at`.
3. Stream rows in batches of 1000 → Meilisearch `addDocuments` (idempotent on `post_id` / `profile_id`).
4. After bulk, emit `SearchIndexBackfilled` event with tuple `(tenant_id, partition, row_count, completed_at, signature)`.
5. Per-pack retention: backfill window bounded by retention floor (no backfilling beyond what retention allowed).
6. PHI redaction (pack-us-healthcare) applied at document-emission time, so PHI-stripped fields stay stripped.

### Constraints

- Backfill is read-only against Postgres; no row mutation.
- Cedar policy + redaction (per `policy/redaction-phi.md` (Slice B for social)) applied at document-emission time.
- Per-tenant rate limit: 1 active backfill per partition; cluster cap = 8 concurrent backfills cluster-wide.
- Cost (per `cost-budget.md`): roughly $0.15 per 1M posts backfilled (Postgres read + Meilisearch index + S3 audit log).

### Verification

- Integration test: corrupt Meilisearch index → backfill → search returns identical hits to Postgres `SELECT`.
- Idempotency: re-running same backfill produces no duplicate documents (`post_id` is primary key).

## Backfill (feed cache rebuild)

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- social rebuild-feed-cache --tenant <t> --user-ref <u>`.
- Auto: Valkey cache eviction triggers per-user lazy rebuild on next feed-render.

Procedure:

1. Identify scope: per-user or per-tenant.
2. For each affected user, query Postgres for posts from followed accounts within feed window (default 7 days hot).
3. Rank using current ranking heuristic / model.
4. Write feed slice to Valkey cache with TTL.
5. Emit `FeedCacheRebuilt` event.

### Constraints

- Per-user rebuild bounded by Cedar (cannot pre-populate feed with posts user cannot read).
- Rebuild duration ≤ 10s per user p95.

## Replay (event fanout)

### Contract

Triggers:
- New µservice consumer onboards (e.g., new Workflow event consumer) and needs catch-up.
- Tenant onboarded mid-stream needs historical events for audit-chain seal rebuild.
- Bug-fix in event payload schema; re-emit corrected version.

Procedure:

1. Operator invokes: `cargo run -p oya-dev-cli -- social replay-events --tenant <t> --from <iso> --to <iso> --consumer <id>`.
2. CLI requires 2-person rule + ops-security approval (replay-events can re-trigger side-effects on consumers; must be audit-trail-bounded).
3. Worker scans `social_posts` + `social_audit_events` tables in `[from, to]` window; emits each as a Workflow event with `replay=true` label, `original_event_ts=<...>`.
4. Consumers MUST honour the `replay` label (idempotent processing on `(event_id, tenant_id)` tuple); failure to do so is the consumer's bug.
5. Audit-chain seal: replay emits sealed `EventReplayed` records per batch (one per 1000 events).

### Constraints

- Replay does NOT mutate the original event records; it appends fresh copies with `replay=true`.
- Replay window bounded by retention floor.
- Workflow consumers MUST be idempotent; replay-unsafe consumers are declared via `consumer_metadata.idempotent: false` and refuse the replay (worker logs + emits warning).
- Personal-tier replay never includes federation-egress (Personal-tier never federates; compile-time invariant).

### Verification

- Integration test: synthetic consumer with idempotency tracking; verify replay of 10k events produces no duplicate side-effects.
- Audit-chain integrity: replay event seals link to original; chain reconstructable end-to-end.

## Federation Replay (ActivityPub outbox re-sync)

### Contract

Triggers:
- Federation peer requests re-sync after extended downtime.
- Internal: federation-gateway state corruption detected.

Procedure:

1. Verify peer is on allowlist + has signed `ResyncRequest` per HTTP Signatures.
2. Identify Professional-tier outbox events within requested window (Personal-tier excluded by type-system).
3. Re-emit as ActivityPub Activities with `resync=true` HTTP header + original `published` field.
4. Peer must honour idempotency on `id` field per ActivityPub Activity Vocabulary.
5. Audit-chain seal emitted per batch.

### Constraints

- Max window 90 days (regulatory + cost bound).
- Per-peer rate limit during resync (1k activities/min).
- Personal-tier posts NEVER included; the outbox type signature accepts only `ProfessionalPost`.

### Verification

- Integration test: synthetic peer requests resync; receives only Professional-tier activities; signatures valid.

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill search index (per 1M posts) | per-corruption | ~$0.15 |
| Rebuild feed cache (per 1M users) | per-Redis-flush | ~$0.50 |
| Replay events (per 10k events × 1 consumer) | per-onboard | ~$0.05 |
| Replay events (per 1M events × all consumers) | per-bugfix-replay | ~$5.00 |
| Federation resync (per 100k activities) | per-peer-resync | ~$0.20 |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill quality bounded by retention floor; cannot recover deleted + retention-purged content.
- Replay quality bounded by audit-chain seal availability; events older than the seal-archival horizon (24mo cold-tier) cannot be replayed.
- Federation replay bounded by remote homeserver's policy; oyatie cannot guarantee external delivery.
- Personal-tier content is NEVER replayed via federation (compile-time invariant).
- Minor-account data (age-attestation table) is NEVER replayed via any external path; Cedar `age_verification_reader` entitlement gated.

## References

- `microservices/social/PRD.md`.
- `microservices/social/capacity-model.md`.
- `microservices/social/cost-budget.md`.
- `microservices/social/contracts/asyncapi/social-events.yaml`.
- ADR-0028 audit-chain.
- ADR-0135 (Connect dissolution).
- ADR-0131 (per-microservice flat layout).
- ActivityPub W3C Rec 2018 §5 (Outbox).
- RFC 9421 HTTP Signatures.
