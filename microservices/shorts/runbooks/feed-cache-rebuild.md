---
doc_class: Runbook
title: Feed cache rebuild
microservice: shorts
severity: "Sev-2 (degradation) / Sev-1 (sustained > 30 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-shorts
date: 2026-05-17
related_artifacts:
  - microservices/shorts/failure-modes.md (FM-01, FM-03)
  - microservices/shorts/multi-region.md
  - microservices/shorts/capacity-model.md
  - microservices/shorts/backfill-replay.md (BF-02)
doc_status: published
---

# Runbook: Feed cache rebuild (FM-01 + FM-03)

## Trigger

- `oya_shorts_feed_cache_hit_ratio` < 70 %.
- `oya_shorts_feed_render_requests_per_sec` > 10× baseline for ≥ 1min (viral event).
- `oya_shorts_feed_cache_inconsistency_total` > 0 (Valkey split-brain or AOF corruption).
- Manual rebuild after Valkey flush or schema migration or ranking-model version promotion (P03+).

## Severity

Sev-2 default; escalate to Sev-1 if sustained > 30 min OR cascades to upload/publish failure.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify Valkey cluster status: `kubectl -n shorts get pods -l app=shorts-redis` (replicas Ready) | ≤ 2 min |
| 2 | Inspect cache breakdown: `oya_shorts_feed_cache_hit_ratio` by `tenant_id`, `shard_id` | ≤ 3 min |
| 3 | If cache flushed: pause fanout-on-write briefly to avoid thundering herd | ≤ 5 min |
| 4 | Trigger per-viewer lazy rebuild: cache populates on next feed-render | ≤ 5 min |
| 5 | For hot-tier creators (>100k followers): trigger fanout-on-write priority rebuild | ≤ 10 min |
| 6 | Serve chronological-fallback feed (always available from Postgres) during rebuild | ≤ 5 min |
| 7 | HPA scale-up on feed-timeline REST pods | ≤ 5 min |

## Full Feed Cache Rebuild (BF-02 path)

If corruption forces full rebuild:

| Step | Action | Time |
|---|---|---|
| 1 | Snapshot current cache state for forensics | ≤ 5 min |
| 2 | Drop affected per-tenant cache shards | ≤ 5 min |
| 3 | Replay from `shorts_videos` Postgres table for last 7 days; rank with current ranking heuristic / model; write to Valkey | 30–60 min depending on tenant size |
| 4 | Verify per-viewer feed slice exists for top-100k active viewers; lazy populate rest | ≤ 15 min |
| 5 | Verify hit ratio returns to > 95 % within 1h | ≤ 1h |

## Per-Tier Minor Adjustments

- Minor accounts (per `age-gate`): chronological-only feed; cache rebuild for minor accounts is per-(viewer, chronological-cursor) — no ranking computation needed; fast path.
- Premium-tier creator feed: priority lane during rebuild; pre-warm during off-peak.
- DRM-protected content surfaces: same cache key but EME license re-issuance triggered on rebuild.

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Viral video triggers mass concurrent feed-pulls | single video in top-emitter; topk(channel) shows skew | accept as legitimate; expand cache TTL |
| Valkey split-brain | sentinel quorum lost; AOF mismatch | rebuild from primary; engage cloud-secrets if HA failure |
| Cache flush from misconfigured Helm | recent deploy correlates | rollback Helm; investigate misconfig |
| Mass-deletion event invalidates cache | high delete rate from DSR-cascade or DMCA takedown | recompute ranking with new corpus |
| Ranking-model version promotion (P03+) | new classifier version deployed | gradual cache rebuild via canary; monitor drift |

## Recovery Verification

- `oya_shorts_feed_cache_hit_ratio` ≥ 95 % for ≥ 30 min.
- `oya_shorts_feed_render_p95_seconds` ≤ 0.25 (matches PRD target).
- `oya_shorts_feed_cache_inconsistency_total` rate = 0 for ≥ 1h.
- No active alerts on feed path.

## Postmortem Triggers

- If recurring: review feed-cache sizing in `capacity-model.md`.
- If corruption pattern: investigate Valkey cluster topology.
- If tenant ingest pattern unsustainable: capacity review with FinOps.

## References

- `microservices/shorts/failure-modes.md` FM-01, FM-03.
- `microservices/shorts/capacity-model.md` §"Valkey".
- `microservices/shorts/multi-region.md`.
- `microservices/shorts/backfill-replay.md` BF-02.
- Valkey Cluster ops docs.
