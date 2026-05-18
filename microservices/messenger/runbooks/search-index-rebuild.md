---
doc_class: Runbook
title: Search index rebuild
microservice: messenger
severity: "Sev-3 (search is best-effort) / Sev-2 if persistent"
status: Accepted
owner_team: axis-messenger + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/messenger/failure-modes.md (FM-06)
  - microservices/messenger/capacity-model.md
doc_status: published
---

# Runbook: Search index rebuild (FM-06)

## Trigger

- `messenger_search_indexer_lag_seconds` > 60s sustained.
- Tantivy / Elasticsearch index corruption.
- Manual rebuild needed after schema migration.

## Severity

Sev-3 by design (search is best-effort; live-fallback to Postgres LIKE). Escalate to Sev-2 if persistent > 4h or affecting many tenants.

## Immediate Mitigation (≤ 1h)

| Step | Action | Time |
|---|---|---|
| 1 | Verify indexer worker pods: `kubectl -n messenger get pods -l app=message-stream-worker,role=indexer` | ≤ 2 min |
| 2 | Inspect lag breakdown: `messenger_search_indexer_lag_seconds` by `tenant_id`, `shard_id` | ≤ 5 min |
| 3 | Scale indexer workers: HPA scale-up (or manual `kubectl scale`) | ≤ 5 min |
| 4 | If high-volume tenant skewing lag: enable per-tenant indexing rate limit | ≤ 5 min |
| 5 | Enable Postgres-LIKE fallback path: search-rest sets feature flag `search.fallback_postgres = true` | ≤ 5 min |
| 6 | Tenant-visible UX: "search results may be stale by up to X minutes" banner | ≤ 5 min |

## Full Index Rebuild

If corruption forces full rebuild:

| Step | Action | Time |
|---|---|---|
| 1 | Snapshot current index for forensics | ≤ 10 min |
| 2 | Drop affected per-tenant index shards | ≤ 5 min |
| 3 | Replay from message-stream event log (Postgres `message_events` table) with `tenant_id` filter | depends on volume; up to 1–4 h for medium tenant |
| 4 | Verify document count matches Postgres `count(*) from messages where tenant_id = ...` | ≤ 10 min |
| 5 | Disable Postgres-LIKE fallback once Tantivy lag < 60s | ≤ 5 min |

## Diagnosis

| Hypothesis | Investigation |
|---|---|
| Ingest spike from a few tenants | per-tenant breakdown |
| Indexer worker CPU saturation | `kubectl top` |
| Bad query plan in indexer | profile recent code changes |
| Disk pressure on Tantivy PV | `df -h` in pod |
| Mass-deletion event triggering re-merge storm | check delete event rate |

## Recovery Verification

- `messenger_search_indexer_lag_seconds` ≤ 5s for ≥ 30 min.
- `messenger_search_query_p99_seconds` ≤ 0.35.
- No active alerts on search path.

## Postmortem

- If recurring: review indexer sizing in `capacity-model.md`.
- If corruption pattern: investigate Tantivy version / hardware.
- If tenant ingest pattern unsustainable: capacity review with FinOps.

## References

- `microservices/messenger/failure-modes.md` FM-06.
- `microservices/messenger/capacity-model.md` §"Tantivy/ES Sizing".
- Tantivy operations docs.
