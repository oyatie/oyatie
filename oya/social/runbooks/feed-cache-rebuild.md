---
doc_class: Runbook
title: Feed cache rebuild
microservice: social
severity: "Sev-2 (degradation) / Sev-1 (sustained > 30 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-social
date: 2026-05-17
related_artifacts:
  - microservices/social/failure-modes.md (FM-01, FM-03)
  - microservices/social/multi-region.md
  - microservices/social/capacity-model.md
doc_status: published
---

# Runbook: Feed cache rebuild (FM-01 + FM-03)

## Trigger

- `social_feed_render_requests_per_sec` > 10× baseline for ≥ 1 min OR feed-cache hit rate drops < 70%.
- `social_feed_cache_inconsistency_total` > 0 (Valkey split-brain or AOF corruption).
- Manual rebuild after Valkey flush or schema migration.

## Severity

Sev-2 default; escalate to Sev-1 if sustained > 30 min or if cascades to post-create failure.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify Valkey cluster status: `kubectl -n social get pods -l app=social-valkey` (replicas Ready) | ≤ 2 min |
| 2 | Inspect cache breakdown: `social_feed_cache_hit_ratio` by `tenant_id`, `shard_id` | ≤ 3 min |
| 3 | If cache flushed: pause fanout-on-write briefly to avoid thundering herd | ≤ 5 min |
| 4 | Trigger per-user lazy rebuild: cache populates on next feed-render | ≤ 5 min |
| 5 | For hot-tier accounts (>100k followers): trigger fanout-on-write priority rebuild | ≤ 10 min |
| 6 | Serve chronological-fallback feed (always available from Postgres) during rebuild | ≤ 5 min |
| 7 | HPA scale-up on feed-timeline REST pods | ≤ 5 min |

## Full Feed Cache Rebuild

If corruption forces full rebuild:

| Step | Action | Time |
|---|---|---|
| 1 | Snapshot current cache state for forensics | ≤ 5 min |
| 2 | Drop affected per-tenant cache shards | ≤ 5 min |
| 3 | Replay from `social_posts` Postgres table for last 7 days; rank with current ranking heuristic; write to Valkey | up to 30–60 min depending on tenant size |
| 4 | Verify per-user feed slice exists for top-100k active users; lazy populate rest | ≤ 15 min |
| 5 | Verify hit ratio returns to > 95 % within 1h | ≤ 1h |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Viral post triggers mass concurrent feed-pulls | Single post in top-emitter; topk(channel) shows skew | accept as legitimate; expand cache TTL |
| Valkey split-brain | sentinel quorum lost; AOF mismatch | rebuild from primary; engage cloud-secrets if HA failure |
| Cache flush from misconfigured Helm | recent deploy correlates | rollback Helm; investigate misconfig |
| Mass-deletion event invalidates cache | high delete rate | recompute ranking with new corpus |

## Recovery Verification

- `social_feed_cache_hit_ratio` ≥ 95 % for ≥ 30 min.
- `social_feed_render_p95_seconds` ≤ 0.2.
- `social_feed_cache_inconsistency_total` rate = 0 for ≥ 1h.
- No active alerts on feed path.

## Postmortem Triggers

- If recurring: review feed-cache sizing in `capacity-model.md`.
- If corruption pattern: investigate Valkey cluster topology.
- If tenant ingest pattern unsustainable: capacity review with FinOps.

## References

- `microservices/social/failure-modes.md` FM-01, FM-03.
- `microservices/social/capacity-model.md` §"Valkey Sizing".
- `microservices/social/multi-region.md`.
- Valkey Cluster ops docs.
