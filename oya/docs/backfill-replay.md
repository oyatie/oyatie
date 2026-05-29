---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: docs
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-docs
deciders: axis-docs, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0135, ADR-0131, ADR-DOCS-0001, ADR-DOCS-0003, ADR-DOCS-0006]
related_artifacts:
  - microservices/docs/PRD.md
  - microservices/docs/capacity-model.md
  - microservices/docs/contracts/asyncapi/docs-events.yaml
  - microservices/docs/runbooks/doc-version-restore-corruption.md
  - microservices/docs/runbooks/attachment-restore.md
  - microservices/docs/runbooks/embed-source-stale-detection.md
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (docs µservice)

## Purpose

Specify how docs handles four scenarios:

1. **Per-doc cache rebuild** — Valkey doc-cache rebuild from canonical Postgres + S3 source (after corruption, after Cedar policy change, after upgrades).
2. **CRDT op-log compaction + snapshot rebuild** — version-aligned compaction per ADR-DOCS-0001.
3. **Document-lifecycle replay** — re-fanout of historical doc-lifecycle events to a newly subscribed downstream consumer.
4. **Embed-snapshot rebuild** — re-fetch cross-µservice embeds when source changed or grant revoked.

## Per-doc cache rebuild

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- docs backfill-cache --tenant <t> --document <d>`.
- Auto: cache-corruption-detector emits `DocCacheCorruptionDetected` event; document-store-worker picks up + backfills.
- Auto: Cedar policy reload emits `DocAclPolicyChanged`; document-store-worker invalidates affected per-block ACL projection cache + rebuilds.

Procedure:

1. Acquire backfill lease in Valkey (per tenant per doc; lease TTL = 1h).
2. Re-read canonical from Postgres + S3.
3. Re-evaluate per-block ACL projection.
4. Bulk-write to Valkey with idempotency key `(tenant_id, doc_id, version_sha)`.
5. Emit `DocCacheBackfilled` event with tuple `(tenant_id, doc_id, version_sha, completed_at, signature)`.

### Performance

- Backfill rate target: ≥500 docs/sec per tenant.
- A 1k-doc tenant rebuilds in <2 minutes.

## CRDT op-log compaction + snapshot rebuild

### Trigger sources

- Version-aligned compaction (per ADR-DOCS-0001): runs after every Nth version increment (default N=100); collapses op-log to snapshot + delta-log.
- Operator-invoked: `cargo run -p oya-dev-cli -- docs compact-crdt --tenant <t> --document <d> --target-version <v>`.
- Auto: Loro upgrade triggers full re-snapshot drill against 100-doc reference corpus before promotion.

### Contract

1. Per-doc lock on collab-crdt-worker for the document.
2. Read all ops from current version's op-log.
3. Re-merge through pinned Loro version into a fresh snapshot.
4. Persist snapshot to S3 + write op-log compaction marker to Postgres.
5. Verify byte-equality against previously emitted projection via `project_to_canonical(state) -> CanonicalBlockTree` (AC-02 invariant).
6. Emit `CrdtOpLogCompacted` event with `(tenant_id, doc_id, from_version_sha, to_version_sha, op_count_collapsed, signature)`.
7. Old op-log retained for 30d for forensic + restore.

### Byte-equality invariant

Per AC-02: `load(emit(canvas-doc))` must be byte-equal to the original. Compaction respects: emit deterministically orders Loro nodes by stable `TreeID` and lex-sorts map keys at the projection boundary. Validated by `cargo nextest run -p oya-docs-document-store-domain -- round_trip_byte_equality` (100-doc reference corpus).

### Bounded op-log size

Per ADR-DOCS-0001 + capacity-model: per-doc op-log size ≤ 100MB warm; compaction triggered when exceeded.

## Document-lifecycle replay

### Trigger sources

- New downstream consumer onboarded (e.g., a new audit-chain instance needs to replay 30d of history).
- Tenant onboarded mid-stream.
- Consumer requests replay for a specific time window for debugging / forensics.

### Contract

1. Acquire replay lease per (tenant_id, consumer_id) — exclusive to prevent double-delivery.
2. Snapshot the `docs.document.lifecycle.v1` + `docs.comments.v1` + `docs.suggestions.v1` + `docs.sharing.v1` event logs in `(tenant_id, partition_id)` partition, ordered by `event_id`.
3. Stream events in batches of 1000 → consumer's Workflow webhook (idempotent per `event_id`).
4. After bulk, emit `DocLifecycleReplayCompleted` with tuple `(tenant_id, consumer_id, event_count, completed_at, signature)`.
5. Per-pack retention: replay window bounded by retention floor.

### Performance

- Replay rate: ≥1000 events/sec per consumer.
- Idempotency: every replayed event carries `(event_id, replay_attempt_n, original_emitted_at)` for consumer dedup.

## Embed-snapshot rebuild

### Trigger sources

- Cross-µservice embed source emitted a change event (`WorkflowStudioDefinitionPublished` / `SheetsCellChanged`).
- Embed-snapshot TTL expired (default 5min ± jitter).
- Operator-invoked: `cargo run -p oya-dev-cli -- docs refresh-embeds --tenant <t> --document <d>`.
- Grant revoked: source-side ACL revocation propagates and refreshes return redacted placeholder.

### Contract

1. Acquire embed-refresh lease per `(document_id, embed_ref)`.
2. mTLS call to source µservice with embedding doc's principal (so source can evaluate ACL passthrough).
3. If source returns snapshot: cache in Valkey with TTL + jitter; emit `EmbedRefreshed` event.
4. If source returns 403 (ACL revoked): replace cached snapshot with redacted placeholder; emit `EmbedAccessRevoked` event.
5. If source timeout/unavailable: keep prior cached snapshot; emit `EmbedSourceUnavailable` event; surface "stale" banner to viewer.
6. Embed depth bound: refuse fetch if depth > 3; emit `EmbedLoopDetected` event.

### Performance

- Refresh rate: per-resolver-pod ≥500/s with single-flight coalescing.
- Cross-pack mesh latency budget: 500ms p99.

## RSVP-like flow doesn't apply — docs has no equivalent storm pattern; substitute case:

### Export-job replay (Special case)

If the export-import-worker pool has been drained (e.g., per `runbooks/export-pipeline-failure-pandoc-rollback.md`) and a backlog accumulated, the same replay procedure applies:

1. Replay is throttled to per-tenant rate.
2. Replayed export jobs are deduplicated by `(export_job_id)`; only the latest version of the source doc is exported.

## Per-µservice consumer contracts

| Downstream | Replay onboarding | Replay catch-up window |
|---|---|---|
| `audit-chain` | replay from tenant onboarding | full retention horizon (per-pack; up to 6y for HIPAA pack) |
| `workflow-engine` | replay last 30d on consumer onboarding | 30d default; configurable to 90d |
| `observability` | replay last 24h on consumer onboarding | 24h default; configurable to 7d |
| `mail` (share notification bridge) | no replay (mail is downstream-only; lost share-notifications are NOT replayed because they may have already been delivered out-of-band) | n/a |
| `messenger` (mention bridge) | no replay | n/a |
| `ontology` | replay last 7d on consumer onboarding | 7d default |
| `tenancy` | no replay (tenancy is upstream-only) | n/a |
| `workflow-studio` (embed-resolver source) | embed-snapshot fetch is access-time only; no replay needed | n/a |

## Verification

- [ ] Backfill / replay rate ≥1000 events/sec measured in benchmark `cargo bench -p oya-docs-document-store-worker -- backfill`.
- [ ] Backfill idempotency property test passes — `cargo nextest run -p oya-docs-document-store-domain -- backfill_idempotent`.
- [ ] Replay window bounded by retention — `cargo nextest run -p oya-docs-document-store-domain -- replay_retention_bound`.
- [ ] CRDT compaction round-trip byte-equality — `cargo nextest run -p oya-docs-document-store-domain -- round_trip_byte_equality`.
- [ ] Embed-refresh single-flight + coalescing — `cargo nextest run -p oya-docs-embed-resolver-domain -- single_flight`.

## References

- ADR-0028 — Audit-chain (Ed25519 + Merkle).
- ADR-DOCS-0001 — CRDT library; compaction implications.
- ADR-DOCS-0003 — Export pipeline; rollback path.
- ADR-DOCS-0006 — DOCX import fidelity; named edge-case test matrix awareness on replay.
- `microservices/docs/contracts/asyncapi/docs-events.yaml`.
- `microservices/docs/runbooks/doc-version-restore-corruption.md` (full restore path).
- `microservices/docs/runbooks/attachment-restore.md` (attachment recovery).
- `microservices/docs/runbooks/embed-source-stale-detection.md` (embed-refresh runbook).
- `microservices/calendar/backfill-replay.md` — sibling reference.
- `microservices/workflow-studio/backfill-replay.md` — sibling reference (CRDT-aligned).
