---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: community
runbook_id: search-rebuild
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_artifacts:
  - microservices/community/failure-modes.md (FM-01, FM-08)
doc_status: published
---

# Runbook: search-rebuild

## When to use

- FM-01 (search index rebuild storm)
- FM-08 (Elasticsearch shard corruption)
- Schema migration requiring reindex
- Per-tenant search SLA breach

## Symptoms

- Search p99 > 1.5 s for 5 min.
- ES shard health red / yellow.
- Index document count drift vs. Postgres source-of-truth.

## Detection

- Grafana alert `community-search-latency-p99-critical`.
- Grafana alert `community-search-index-health-red`.
- Daily reconciliation job: `oya_community_search_index_drift_count`.

## Triage

1. Identify scope: single tenant index, single shard, whole cluster?
2. Check whether rebuild is in progress for another tenant (storm risk).
3. Verify Postgres source-of-truth is healthy (rebuild source must be intact).

## Mitigation

### Single-tenant reindex

1. Pause search write path for tenant.
2. Snapshot current index (in case of regression).
3. Create new index with target schema: `community-<tenant_id_short>-<bc>-v<N+1>`.
4. Run reindex worker:
   `cargo run -p oya-community-search-index-cli -- reindex --tenant <T> --from-source postgres --to-index v<N+1>`
5. Verify document count matches.
6. Swap alias to new index.
7. Drop old index after 24 h grace.
8. Resume write path.

### Storm prevention

1. Per-tenant reindex enters queue with token-bucket (1 reindex per 5 min cluster-wide).
2. Staggered windows: scheduled in tenant's low-traffic window.
3. Cap concurrent reindexes per cluster: 4.

### Shard corruption

1. Promote replica shard.
2. Reindex affected shard from Postgres source.
3. Audit-chain witness on shard restore.

## Verification

- Search p99 < 500 ms.
- Index health green.
- Document count drift < 0.1 %.
- Full reindex of 10⁷ docs completes within 60 min (SLO).

## Post-Incident

- If schema bug: ADR for schema migration policy.
- If capacity: ES capacity revision.

## Owner

axis-community.
