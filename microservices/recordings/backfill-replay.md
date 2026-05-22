---
doc_class: BackfillReplayPlan
template_id: TPL-BACKFILL
microservice: recordings
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-recordings + ops-sre-reliability
related_adrs: [ADR-RECORDINGS-0001, ADR-RECORDINGS-0007]
doc_status: published
---

# Backfill & Replay Plan: recordings µservice

## Scope

Two replay surfaces are first-class:

1. **Transcript backfill** — re-run Whisper-large on legacy media uploaded
   before transcription was enabled (or before Whisper-large was pinned).
2. **Search-index rebuild** — full reconstruction of the Meilisearch index
   from transcript Workflow events.
3. **Audit-chain re-anchor** — re-seal historical recordings under a new
   audit-chain root if the previous chain is rotated.
4. **Strangler-migration replay** — replay legacy `oya-connect-recordings-domain`
   archive entries through the new ingest contract per ADR-RECORDINGS-0007.

## Procedures

### 1. Transcript backfill

- Workflow event: `RecordingBackfillTranscribeRequested` (tenant-admin opt-in).
- Worker: `oya-recordings-transcript-worker` consumes; throttles to N
  concurrent Whisper jobs to avoid blocking real-time ingest.
- Per-pack consent gate: backfill refused unless tenant has signed updated
  ToS that include transcription clause.
- Cost capped per tenant per month (per `cost-budget.md`).

### 2. Search-index rebuild

- Trigger: search-index corruption alert or quarterly drill.
- Procedure: scan `transcript` rows in Postgres by `(tenant_id, recording_id)`,
  emit `TranscriptIndexUpdated` events; Meilisearch worker re-indexes.
- Estimated time: ≤ 6h for 1M transcripts.
- Run as canary on staging cell first.

### 3. Audit-chain re-anchor

- Triggered by audit-chain µservice rotation event.
- Recordings emits a `recordings.recording.reAnchored.v1` event per
  recording; new Merkle commitment computed.
- No source-data mutation.

### 4. Strangler migration replay (ADR-RECORDINGS-0007 ingest contract)

- For each legacy `RecordingArchiveEntry`: emit a synthetic
  `RecordingIngestRequest` with `source_kind: legacy_workspace_recording`.
- New µservice processes via the standard ingest pipeline; re-runs
  transcription + diarization if tenant opts in.
- Idempotent: re-runs match the same `recording_id`; no duplicate archive
  rows.
- Per ADR-0134 Phase 2 adapter soak.

## Verification

- Each backfill emits an audit-chain seal of the backfilled-row-count for
  the day.
- Search-index rebuild validated by per-tenant search-quality benchmark
  (baseline queries return same top-K within ε tolerance).

## References

- ADR-RECORDINGS-0001, ADR-RECORDINGS-0007.
- ADR-0134.
- `runbooks/transcript-pipeline-degraded-whisper.md`.
