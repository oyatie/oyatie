---
doc_class: ContractSpec
title: Backfill + Replay Contract
microservice: meet
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-meet
deciders: axis-meet, council-architecture, ops-sre-reliability
related_adrs: [ADR-0028, ADR-0135, ADR-0131]
related_artifacts:
  - microservices/meet/PRD.md
  - microservices/meet/capacity-model.md
  - microservices/meet/contracts/asyncapi/meet-events.yaml
review_cadence: annually
doc_status: published
---

# Backfill + Replay Contract (meet µservice)

## Purpose

Specify how meet handles two scenarios:

1. **Backfill** — transcript search index rebuild from S3 transcripts, or recording manifest reindex from Postgres, after a corruption or new analyser deploy.
2. **Replay** — re-fanout of historical Meeting / Recording / Transcript events to a newly subscribed downstream consumer (audit-chain, workflow-engine, foundry-runtime, mail), or to replay missed events for a tenant onboarded mid-stream.

## Backfill (transcript search index rebuild)

### Contract

Trigger sources:
- Operator-invoked: `cargo run -p oya-dev-cli -- meet backfill-transcript-search --tenant <t> --from <iso> --to <iso>`.
- Auto: corruption-detector emits `TranscriptSearchIndexCorruptionDetected` event; worker picks up + backfills the affected partition.

Procedure:

1. Acquire backfill lease in Valkey (per tenant per index partition; lease TTL = 1h).
2. List S3 transcripts under `s3://oya-meet-transcripts-<pack>/<tenant_id>/` filtered by `[from, to]` window.
3. Stream transcript JSON in batches of 1000 → Meilisearch `addDocuments` (idempotent on `transcript_id`).
4. After bulk, emit `TranscriptSearchIndexBackfilled` event with tuple `(tenant_id, partition, row_count, completed_at, signature)`.
5. Per-pack retention: backfill window bounded by retention floor (no backfilling beyond what retention allowed).

### Constraints

- Backfill is read-only against S3; no blob mutation.
- Cedar policy + redaction (per `policy/redaction-phi.md`) applied at document-emission time, so PHI-stripped fields stay stripped.
- Per-tenant rate limit: 1 active backfill per partition; cluster cap = 4 concurrent backfills cluster-wide.
- Cost (per `cost-budget.md`): roughly $0.20 per 1k transcripts backfilled (S3 read + Meilisearch index + audit log).

### Verification

- Integration test: corrupt Meilisearch index → backfill → search returns identical hits to S3 transcript content.
- Idempotency: re-running same backfill produces no duplicate documents (`transcript_id` is primary key).

### Runbook

See `runbooks/transcription-classifier-rollback.md` §"Index rebuild" (Slice-A authored).

## Backfill (recording manifest reindex)

### Contract

Trigger:
- Operator-invoked when Postgres recording-manifest table lags behind S3 actual blobs (e.g., after restore-from-snapshot).

Procedure:

1. Enumerate S3 recording bucket; for each blob, compute (recording_id, content_hash).
2. Compare against Postgres manifest; surface deltas.
3. For missing manifest rows: insert via `meet-recording-rest` (server-side; respects audit-chain).
4. For orphaned manifest rows: mark `orphaned=true`; do NOT auto-delete (eDiscovery preservation).
5. Emit `RecordingManifestReconciled` event.

### Constraints

- Read-only against S3.
- Cedar policy enforces tenant scope.

## Replay (event fanout)

### Contract

Triggers:
- New µservice consumer onboards (e.g., new Workflow event consumer) and needs catch-up.
- Tenant onboarded mid-stream needs historical events for audit-chain seal rebuild.
- Bug-fix in event payload schema; re-emit corrected version.

Procedure:

1. Operator invokes: `cargo run -p oya-dev-cli -- meet replay-events --tenant <t> --from <iso> --to <iso> --consumer <id>`.
2. CLI requires 2-person rule + ops-security approval (replay-events can re-trigger side-effects on consumers; must be audit-trail-bounded).
3. Worker scans `meet_meetings` + `meet_recordings` + `meet_transcripts` + `meet_audit_events` tables in `[from, to]` window; emits each as a Workflow event with `replay=true` label + `original_event_ts=<...>`.
4. Consumers MUST honour the `replay` label (idempotent processing on `(event_id, tenant_id)` tuple); failure to do so is the consumer's bug.
5. Audit-chain seal: replay emits sealed `EventReplayed` records per batch (one per 1000 events).

### Constraints

- Replay does NOT mutate the original event records; appends fresh copies with `replay=true`.
- Replay window bounded by retention floor.
- Workflow consumers MUST be idempotent; replay-unsafe consumers are declared via `consumer_metadata.idempotent: false` and refuse the replay (worker logs + emits warning).
- E2E-mode meeting media cannot be replayed beyond client-side scope (since server has no plaintext); replay of E2E events emits the encrypted blob ref + signature only, no body.

### Verification

- Integration test: synthetic consumer with idempotency tracking; verify replay of 10k events produces no duplicate side-effects.
- Audit-chain integrity: replay event seals link to original; chain reconstructable end-to-end.

### Runbook

See `runbooks/recording-storage-degraded.md` §"Event replay" (Slice B).

## Cost Model

| Operation | Frequency | Estimated cost per call |
|---|---|---|
| Backfill transcript search index (per 1k transcripts) | per-corruption | ~$0.20 |
| Backfill recording manifest (per 100k recordings) | per-restore-event | ~$0.50 |
| Replay events (per 10k events × 1 consumer) | per-onboard | ~$0.10 |
| Replay events (per 1M events × all consumers) | per-bugfix-replay | ~$10.00 |

Cost surfaced in `cost-budget.md` §"Cost-Optimisation Levers".

## Limitations

- Backfill quality bounded by retention floor; cannot recover deleted + retention-purged content.
- Replay quality bounded by audit-chain seal availability; events older than the seal-archival horizon (24mo cold-tier) cannot be replayed.
- E2E-mode meeting body replay is impossible by design (plaintext never lived server-side).
- Cross-pack replay forbidden (residency invariant).

## References

- `microservices/meet/PRD.md`.
- `microservices/meet/capacity-model.md`.
- `microservices/meet/cost-budget.md`.
- `microservices/meet/contracts/asyncapi/meet-events.yaml`.
- ADR-0028 audit-chain.
- ADR-0135 (net-new µservice).
- ADR-0131 (per-microservice flat layout).
- `microservices/messenger/backfill-replay.md` (shape reference).
