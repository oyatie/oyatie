---
doc_class: Runbook
title: Feed cache rebuild
microservice: network
severity: "Sev-2 (degradation) / Sev-1 (sustained > 30 min)"
status: Accepted
owner_team: ops-sre-reliability + axis-network
date: 2026-05-17
last_drill_date: 2026-05-17
related_artifacts:
  - microservices/network/failure-modes.md (FM-01, FM-03, FM-04, FM-06, FM-09, FM-11, FM-20)
  - microservices/network/multi-region.md
  - microservices/network/capacity-model.md
doc_status: published
---

# Runbook: Feed cache rebuild (FM-01 + FM-03 + adjacent feed-path failure modes)

## Trigger

- `network_feed_render_requests_per_sec` > 10× baseline for ≥ 1 min OR feed-cache hit rate drops < 70%.
- `network_feed_cache_inconsistency_total` > 0 (Valkey split-brain or AOF corruption).
- Manual rebuild after Valkey flush or schema migration.

## Severity

Sev-2 default; escalate to Sev-1 if sustained > 30 min or if cascades to post-create / connection-action failure.

## Immediate Mitigation (≤ 30 min)

| Step | Action | Time |
|---|---|---|
| 1 | Verify Valkey cluster status: `kubectl -n network get pods -l app=network-redis` (replicas Ready) | ≤ 2 min |
| 2 | Inspect cache breakdown: `network_feed_cache_hit_ratio` by `tenant_id`, `shard_id` | ≤ 3 min |
| 3 | If cache flushed: pause fanout-on-write briefly to avoid thundering herd | ≤ 5 min |
| 4 | Trigger per-user lazy rebuild: cache populates on next feed-render | ≤ 5 min |
| 5 | For hot-tier accounts (>100k connections): trigger fanout-on-write priority rebuild | ≤ 10 min |
| 6 | Serve chronological-fallback feed (always available from Postgres) during rebuild | ≤ 5 min |
| 7 | HPA scale-up on feed-timeline REST pods | ≤ 5 min |

## Full Feed Cache Rebuild

If corruption forces full rebuild:

| Step | Action | Time |
|---|---|---|
| 1 | Snapshot current cache state for forensics | ≤ 5 min |
| 2 | Drop affected per-tenant cache shards | ≤ 5 min |
| 3 | Replay from `network_posts` Postgres table for last 7 days; rank with current ranking heuristic; write to Valkey | up to 30–60 min depending on tenant size |
| 4 | Verify per-user feed slice exists for top-100k active Professional users; lazy populate rest | ≤ 15 min |
| 5 | Verify hit ratio returns to > 95 % within 1h | ≤ 1h |

## Paired Failure-Mode Mitigations (this runbook covers)

### FM-04 §"degraded media" — media-store outage

| Step | Action |
|---|---|
| 1 | Check `network_media_upload_failure_rate`; verify S3 endpoint reachability per pack |
| 2 | Failover to DR-pair S3 replica if available; otherwise surface backlog visibility |
| 3 | Queue uploads in Valkey Streams (Redis wire-compat) until S3 recovers; do not lose in-flight blobs |
| 4 | Lazy-hydrate cold-tier on demand once S3 restored |

### FM-06 §"search degraded" — Meilisearch indexer lag

| Step | Action |
|---|---|
| 1 | Inspect `network_search_indexer_lag_seconds{index="..."}` per index |
| 2 | Scale indexer workers for the lagging index (people / content / skills / jobs / companies / events) |
| 3 | Enable Postgres-ILIKE fallback path for people-search (most-searched surface) |
| 4 | Pause low-priority tenants' indexing |
| 5 | Reconcile via `cargo run -p oya-dev-cli -- network backfill-search --index <name>` per `backfill-replay.md` |

### FM-09 §"media-malware quarantine" — OPSWAT / ClamAV positive

| Step | Action |
|---|---|
| 1 | Verify scanner verdict in audit-chain (`network_media_malware_detected_total` increment) |
| 2 | Confirm blob is in `oya-network-quarantine-<pack>` bucket, NOT production bucket |
| 3 | Audit-chain seal of detection emitted |
| 4 | Tenant security-admin notified via mail bridge |
| 5 | If pattern attack: engage ops-security; rotate per-tenant upload signing keys |

### FM-11 §"notification backlog" — fanout queue depth > 500k

| Step | Action |
|---|---|
| 1 | Inspect `network_notification_fanout_queue_depth` by `tenant_id`, `kind` |
| 2 | Scale notification workers per `capacity-model.md` formula |
| 3 | Coalesce more aggressively into digest (lower real-time:digest split temporarily) |
| 4 | Pause low-priority push channel for non-Page non-endorsement notifications |

### FM-20 §"cell shard" — cell shard migration (delegate to cell µservice)

| Step | Action |
|---|---|
| 1 | Engage cell µservice runbook `runbooks/cell-shard-migration.md` |
| 2 | network responsibilities: pause writes for affected tenant; flush Valkey cache for tenant |
| 3 | Resume after cell µservice signals migration complete; rebuild caches on demand |

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Viral Professional post triggers mass concurrent feed-pulls | Single post in top-emitter; topk(channel) shows skew | accept as legitimate; expand cache TTL |
| Valkey split-brain | sentinel quorum lost; AOF mismatch | rebuild from primary; engage cloud-secrets if HA failure |
| Cache flush from misconfigured Helm | recent deploy correlates | rollback Helm; investigate misconfig |
| Mass-deletion event invalidates cache | high delete rate | recompute ranking with new corpus |
| Newsletter blast cascading | mail-bridge emission spike correlates | coordinate with mail µservice on rate-limit |

## Recovery Verification

- `network_feed_cache_hit_ratio` ≥ 95 % for ≥ 30 min.
- `network_feed_render_p95_seconds` ≤ 0.2.
- `network_feed_cache_inconsistency_total` rate = 0 for ≥ 1h.
- No active alerts on feed path.

## Postmortem Triggers

- If recurring: review feed-cache sizing in `capacity-model.md`.
- If corruption pattern: investigate Valkey cluster topology.
- If tenant ingest pattern unsustainable: capacity review with FinOps.

## References

- `microservices/network/failure-modes.md` FM-01, FM-03, FM-04, FM-06, FM-09, FM-11, FM-20.
- `microservices/network/capacity-model.md` §"Valkey Sizing".
- `microservices/network/multi-region.md`.
- `microservices/social/runbooks/feed-cache-rebuild.md` (sibling reference).
- Valkey Cluster ops docs.
