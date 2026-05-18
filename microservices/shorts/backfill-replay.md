---
doc_class: BackfillReplay
title: Backfill + Replay Playbook
microservice: shorts
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-shorts + ops-sre-reliability
deciders: axis-shorts, ops-sre-reliability, council-architecture
related_adrs: [ADR-0028, ADR-0126, ADR-0131]
related_artifacts:
  - microservices/shorts/failure-modes.md
  - microservices/shorts/incident-response.md
  - microservices/shorts/multi-region.md
review_cadence: quarterly
doc_status: published
---

# Backfill + Replay Playbook (shorts µservice)

## Purpose

Define safe procedures for: re-running transcode pipeline over historical videos (re-encode at improved codec); rebuilding feed-timeline cache from Postgres; replaying notifications during outage; replaying Workflow events to downstream µservices; re-running copyright-claim fingerprint match against an updated corpus; re-running moderation classifier on historical content after a classifier version promotion; rebuilding Meilisearch index from Postgres source-of-truth; rebuilding DSR-cascade-affected indexes.

## Idempotency

All replay paths MUST be idempotent. Each operation carries `idempotency_key = (microservice, operation, target_ref, version_tag)`. Receiving systems dedupe.

## Backfill Catalog

### BF-01: Transcode re-encode (codec upgrade — AV1 adoption)

**When**: When AV1 codec is adopted at scale; existing H.264-only blobs are augmented with AV1 variants for storage savings (~30%).

**Procedure**:
1. Identify videos in `videos` table where `transcode_codec_set DOES NOT include 'av1'`.
2. Enqueue per-video transcode job with `target_codec_set = ['h264', 'h265', 'av1']`; idempotency_key = `(video_id, version=av1-add)`.
3. Worker pool: separate KEDA-style autoscale; budget bounded to off-peak hours.
4. New manifest published; CDN edge primes; original manifest stays for legacy clients.
5. Audit-chain seal: `BackfillTranscodeEmitted{video_id, codec_set, replayer_id}`.

**Throttling**: per-tenant rate limit; per-pack daily budget; off-peak window enforcement.

**SLO**: full pack backfill ≤ 30 days; per-tenant backfill ≤ 7 days.

### BF-02: Feed-timeline cache rebuild

**When**: Redis flush; Redis cluster rebuild; ranking-model version promotion (P03+).

**Procedure**:
1. Snapshot current cache for forensics.
2. Drop affected per-tenant cache shards.
3. Replay from `videos` Postgres table over last 7d posting window.
4. Apply current ranking heuristic / model.
5. Re-warm hot accounts (top 100k creators per pack); lazy-populate cold tier.
6. Verify hit-ratio ≥ 95% within 1h.

**Idempotency**: per-(viewer, video) feed-entry insert is upsert; replayer-id versioned.

**SLO**: ≤ 1h pack-wide rebuild for XS tier; scales with capacity tier.

### BF-03: Notification replay

**When**: Notification worker outage; queue backup recovery; missed-delivery audit.

**Procedure**:
1. Identify `notifications` rows where `delivery_state = pending` and `attempted_at < now - 5min`.
2. Re-enqueue with original idempotency_key + replayer_id.
3. Per-(recipient_ref, notification_id) idempotency at recipient side; duplicate delivery rejected.
4. Coalesce digests for low-priority notifications.

**Idempotency**: recipient-side dedupe via `notification_id`; recipient logs hash of delivered notifications for 30d.

**SLO**: ≤ 1h replay catch-up after worker recovery.

### BF-04: Workflow event replay (to downstream µservices)

**When**: Downstream µservice (audit-chain, observability, ontology, foundry-runtime) was unreachable for a window; events backed up in AMQP DLQ.

**Procedure**:
1. Identify DLQ depth; estimate replay duration.
2. Replay events in original order with `event_id` preserved + `replayer_id` tag.
3. Receiving µservice idempotency on `event_id`.
4. Pause replay if downstream backpressure.

**Idempotency**: every Workflow event carries unique `event_id` ULID; receiver dedupes for 24h window.

**SLO**: replay rate ≤ 10x sustained baseline; aim for full catch-up ≤ 1h.

### BF-05: Copyright fingerprint match re-run

**When**: New licensed corpus addition; fingerprint algorithm update; per-licensor namespace correction.

**Procedure**:
1. Identify videos posted after `cutoff_timestamp` (or all videos if full re-run).
2. Enqueue per-video fingerprint job with `corpus_version = X.Y.Z`.
3. Workers run Chromaprint audio-fingerprint + DCT video perceptual-hash against new corpus.
4. New matches emit `CopyrightClaimMatchEmitted` event.
5. Per-licensor namespace: if new matches found, escalate to ops-legal for licensor confirmation before action.
6. If pre-existing claims exist on a video, the new corpus match is additive (not replacement).

**Throttling**: per-licensor namespace daily cap; off-peak window.

**SLO**: per-licensor full re-run ≤ 7 days at S tier.

### BF-06: Moderation classifier re-run on historical content

**When**: Classifier version promoted to a new generation; backfill required for transparency + Art. 73 evidence; rollback required after detected false-positive event.

**Procedure**:
1. Identify videos posted within `(version_X.start_ts, version_X.end_ts)` window.
2. Enqueue per-video classifier job with `classifier_version = X.Y.Z`.
3. Compare new verdict to historical verdict; if disagreement:
   - **For rollback case** (mass false-positive event from new version): restore auto-hidden videos to visible.
   - **For improvement case** (new version detects what old missed): escalate to manual reviewer queue; do NOT auto-hide without human review.
4. Audit-chain seal: `BackfillModerationClassifierEmitted{video_id, classifier_version, replayer_id, action_taken}`.
5. Each backfill verdict carries `eu_ai_act_label: ai_generated_assessment` + transparency record per Art. 50.

**EU AI Act Art. 73 considerations**: if backfill is part of a serious-incident recovery, the 15-day notification clock applies; council-privacy + ops-legal engaged.

**SLO**: rollback path ≤ 6h pack-wide; improvement path ≤ 7 days.

### BF-07: Meilisearch index rebuild

**When**: Index corruption; shard rebalance; schema change.

**Procedure**:
1. Halt indexer for affected shards.
2. Rebuild index from Postgres source of truth via streaming cursor.
3. Per-tenant index isolation maintained.
4. Verify query parity between rebuilt index and Postgres ILIKE-fallback for sample queries.
5. Restore indexer; backpressure-coalesce backlog.

**Idempotency**: index document keyed by `(tenant_id, document_id)`; upsert.

**SLO**: ≤ 6h per pack at XS tier; scales with capacity.

### BF-08: DSR-cascade re-run

**When**: Right-to-erasure request audit reveals incomplete propagation (cache or search staleness); regulatory follow-up.

**Procedure**:
1. Identify subject's ULID + all affected videos / comments / reactions / shares.
2. Re-execute DSR cascade per `policy/data-residency.md` §DSR; mark rows tombstoned + redact identifiers.
3. Search re-index in redacted form.
4. CDN cache purge for affected URLs (signed-URL TTL ensures expiry within 15 min absent purge).
5. Audit-chain note `DsrCascadeBackfillEmitted{subject_ref, replayer_id}` (does NOT include redacted content).

**Idempotency**: DSR-cascade is idempotent (re-running yields same end state).

**SLO**: ≤ 30 days from request per GDPR; faster where local law requires.

### BF-09: Auto-caption re-run

**When**: ASR model upgrade; per-locale model improvement; mass-correct after detected systemic mistranscription.

**Procedure**:
1. Identify videos in target window or affected locale.
2. Enqueue per-video caption job with `asr_version = X.Y.Z + locale`.
3. Worker calls foundry-runtime ASR; emits WebVTT + TTML.
4. New caption track replaces old; old retained in S3 with `caption_replaced` tag for audit.
5. Creator notification (per `Notification` workflow) if mistranscription severity warrants.

**Idempotency**: per-(video_id, asr_version, locale) caption-track keying.

**SLO**: ≤ 14 days per pack at S tier.

### BF-10: Audit-chain seal backfill

**When**: Audit-chain µservice was unreachable; backlog of unsealed state transitions in shorts WAL.

**Procedure**:
1. Identify unsealed transitions in `shorts_audit_chain_pending` table.
2. Re-emit to audit-chain in original order with original timestamps preserved.
3. Audit-chain idempotency on `(microservice, event_id, original_timestamp)`.
4. Verify all backfilled seals visible in audit-chain query within 1h.

**SLO**: ≤ 24h backfill catch-up; Sev-2 if exceeded.

## Verification

CI lane `oya-governance-backfill-idempotency --microservice shorts` validates:
- Every backfill operation has documented idempotency key.
- Every backfill job emits replay-tagged audit-chain record.
- Replay-tag never overwrites original timestamps.
- Per-licensor / per-pack rate limits configured.

## References

- ADR-0028 audit-chain.
- Parallel ADR-0126.
- ADR-0131.
- `microservices/shorts/failure-modes.md`.
- `microservices/shorts/incident-response.md`.
- `microservices/shorts/multi-region.md`.
- `microservices/social/backfill-replay.md` (sibling pattern).
- GDPR Art. 17 + Art. 33.
- EU AI Act Art. 73.
- DMCA §512(c)(3) + §512(i)(1)(A).
