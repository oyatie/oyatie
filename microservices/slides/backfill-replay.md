---
doc_class: BackfillReplay
template_id: TPL-BACKFILL-REPLAY
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability
doc_status: published
---

# Backfill + replay plan — slides µservice

## Backfill scenarios

### B-1 — Restore deck content from S3 snapshot after Postgres logical corruption

Source: S3 deck-snapshot bucket (cross-region replicated).

Steps:
1. Identify affected `(tenant_id, deck_id)` set + corruption point-in-time.
2. Freeze writes for affected decks via per-deck lease in Redis (refuse new edits).
3. Restore Postgres rows from latest pre-corruption snapshot.
4. Replay CRDT op stream from `oya-slides-collab-op-log` archive (S3 partitioned by tenant_id × day).
5. Re-issue Ed25519 audit seals for replay (audit-chain accepts seal_kind=backfill).
6. Tenant notification + verification via UI banner; resume writes.

RTO: 30min for ≤ 1k decks; 4h for ≤ 100k decks (parallelizable per tenant).

### B-2 — Backfill audit-chain seals after audit-chain µservice outage

Source: slides-side write-ahead log (`oya-slides-audit-pending`) in Postgres.

Steps:
1. On audit-chain recovery, replay queued seal requests from oldest.
2. Verify seal monotonicity per-tenant.
3. Mark replayed seals with `seal_kind=backfill`.

RTO: ≤ 1h for ≤ 24h of queued seals.

### B-3 — Backfill chart-live-link refreshes after sheets µservice outage

Source: per-chart-bind metadata in Postgres.

Steps:
1. On sheets recovery, enumerate active chart binds.
2. For each, trigger refresh via sheets-SDK; mark refresh-time + audit row.
3. Per-pack rate-limit to avoid sheets thundering-herd.

RTO: 30min for ≤ 10k charts.

### B-4 — Backfill ontology Presentation/Slide entities from deck-history

Source: Postgres deck-history + audit-chain seals.

Steps:
1. Enumerate decks where `ontology_link_at` is null.
2. For each, emit `PresentationCreated` + `SlideCreated` event with backfill flag.
3. ontology consumes + acks; slides updates `ontology_link_at`.

RTO: 1h for ≤ 100k decks.

### B-5 — Backfill broadcast-mode session ontology entities

Source: broadcast-mode-worker session logs + audit-chain seals.

Steps:
1. Enumerate broadcast sessions where ontology `BroadcastSession` entity is missing.
2. Emit `BroadcastStarted` + `BroadcastEnded` events with backfill flag.

RTO: 30min for ≤ 10k sessions.

## Replay scenarios

### R-1 — Replay tenant deck history for compliance audit

Source: audit-chain Ed25519 ledger.

Steps:
1. Tenant or auditor submits replay request via SDK with `(tenant_id, from_ts, to_ts, scope)`.
2. slides materializes replay by streaming audit-chain seals in order + materializing deck state at each save.
3. Output: ordered timeline + per-event details + signature verification result.
4. Output stored to S3 with audit-chain seal of replay itself.

RTO: 1h for ≤ 1y of tenant history.

### R-2 — Replay CRDT op stream for collab debugging

Source: `oya-slides-collab-op-log` archive (90d hot, 1y warm).

Steps:
1. Engineer requests replay via internal CLI (auth via OIDC + audit).
2. Stream loaded into a local Loro instance + replayed deterministically.
3. Output: per-op result + final CRDT state + any conflict surfaced.

RTO: 30min for ≤ 24h of single-deck op log.

### R-3 — Replay broadcast session for QA

Source: broadcast-mode-worker session log + LiveKit recording (if tenant opted in).

Steps:
1. Engineer or tenant requests replay (tenant via SDK with audit).
2. Slides materializes deck state at broadcast time + LiveKit recording offset.
3. Output: timeline + slide-at-time + audience-engagement snapshots.

RTO: 30min for ≤ 1h broadcast.

### R-4 — Replay AI-content-generation invocation for risk-class audit

Source: foundry-runtime archive (90d).

Steps:
1. Compliance officer requests via internal CLI.
2. Slides materializes prompt + completion + risk-class + decision.
3. Output: ordered timeline + verification result.

RTO: 15min for any single invocation.

## Determinism guarantees

- Loro CRDT op stream replay produces byte-identical final state (deterministic projection per ADR-SLIDES-0001).
- PPTX export deterministic mode (OOXML key ordering + ffmpeg deterministic flags).
- PDF/A export deterministic (WeasyPrint stable mode + per-pack timestamp normalization).

## Replay safety

- Replay never re-emits side effects (no actual deck save, no broadcast start, no AI invocation).
- Replay never writes outside of replay-output S3 prefix.
- Replay never bypasses Cedar ACL — replay output limited to caller's authorization scope.
- Replay always audit-sealed (replay-of-replay traceable).

## Backfill safety

- Backfill always idempotent (sequence_num monotonic; replays don't double-apply).
- Backfill always rate-limited per-pack to avoid downstream µservice thundering-herd.
- Backfill always audit-sealed.

## References

- ADR-SLIDES-0001 Loro CRDT deterministic replay.
- ADR-SLIDES-0003 export deterministic mode.
- ADR-SLIDES-0006 AI risk-class audit.
- audit-chain replay contract (per audit-chain µservice docs).
