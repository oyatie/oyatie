---
doc_class: Runbook
title: Query engine restart (Function engine + Query engine OOM / runaway projection)
microservice: ontology
severity: "Sev-2 (Function engine OOM) / Sev-3 (single-query DoS)"
status: Accepted
owner_team: axis-ontology + ops-sre-reliability
date: 2026-05-17
related_artifacts:
  - microservices/ontology/failure-modes.md (FM-05 query OOM, FM-10 ClickHouse lag)
  - microservices/ontology/PRD.md §"Performance Targets"
doc_status: published
---

# Runbook: Query engine restart

## Trigger

Any of:
- Function engine pod OOM kill loop (FM-05).
- Query engine 3-layer KG join timeout (FM-10 cascade).
- Tenant-submitted Function with unbounded scan; EXPLAIN check bypassed.
- ClickHouse OLAP query DDoS via expensive aggregations.

## Severity

- Sev-2 if engine-wide impact (multiple tenants affected).
- Sev-3 if isolated to one tenant's query.

## Pre-checks

1. Confirm engine pods are in `CrashLoopBackOff`: `kubectl get pods -n ontology -l app=function-engine` — at least 1 pod restart count > 5.
2. Identify the offending query: `kubectl logs -l app=function-engine --tail=100 | grep "query_id"` extract query_id of the largest-memory call before crash.
3. Confirm Postgres `pg_stat_activity` shows the offending session: `SELECT pid, state, query, query_start FROM pg_stat_activity WHERE state = 'active' AND now() - query_start > interval '30 seconds';`

## Steps

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>` Slack; declare severity; assign IC | ≤ 5 min |
| 2 | Kill the offending Postgres backend: `SELECT pg_terminate_backend(<pid>)` for each session lasting > 5 min | ≤ 2 min |
| 3 | Set per-tenant rate limit at function-engine: `oya-ontology-sdk rate-limit-set --tenant <id> --reads-per-sec 100 --reason "runaway-query incident <id>"` | ≤ 2 min |
| 4 | Scale up function-engine replicas to absorb load: `kubectl scale deployment function-engine -n ontology --replicas=20` (interim; HPA will normalise) | ≤ 5 min |
| 5 | Restart any OOM-killed pods: `kubectl delete pod -l app=function-engine --field-selector=status.phase=Failed` | ≤ 2 min |
| 6 | Verify HPA scales back to baseline; CPU + memory stable | ≤ 30 min |
| 7 | Engage tenant: explain rate-limit; offer Function review | ≤ 1 h |
| 8 | Postmortem within 5 business days | – |

## Permanent fix: EXPLAIN pre-check tightening

If the offending Function passed EXPLAIN but exploded at runtime:
1. File a Function evaluator improvement PR: add memory-projection estimate to EXPLAIN; reject queries where projected memory > threshold (e.g., 100 MB).
2. LEAN check `oya-foundry-fitness-ontology-function-explain-conformance` added to validate every new Function Type schema has a max_memory_projection.
3. Per-tenant cardinality budget enforced (`max_function_result_rows_per_call`).

## ClickHouse cascade (FM-10)

If function-engine OOM cascades to ClickHouse:

| Step | Action | Time |
|---|---|---|
| 1 | Same as above for Postgres | – |
| 2 | Check ClickHouse query queue: `SELECT count() FROM system.processes WHERE elapsed > 60` | ≤ 1 min |
| 3 | If queue > 100: kill offending queries via `KILL QUERY WHERE query_id IN (<list>)` | ≤ 5 min |
| 4 | Tighten per-tenant ClickHouse `max_memory_usage` + `max_execution_time` for the offending tenant | ≤ 5 min |
| 5 | If ClickHouse mirror lag > 5 min: throttle OLAP reads per `runbooks/clickhouse-rebalance.md` | – |

## Verification

After recovery:
- `function-engine` HPA at baseline (2 replicas + buffer).
- Postgres `pg_stat_activity` shows no long-running queries.
- ClickHouse `system.processes` shows no stuck queries.
- p99 Function read latency back to ≤ 50 ms.
- Per-tenant rate limit lifted only after tenant has reviewed + remediated their Function.

## Post-incident updates

- Postmortem within 5 business days.
- If FM-05 recurs ≥ 2 in 12 months for the same tenant: tenant onboarding requires Function review by axis-ontology before high-cost Functions deployed to production.
- If permanent EXPLAIN tightening: PR + lane upgrade.

## References

- `microservices/ontology/failure-modes.md` FM-05, FM-10.
- `microservices/ontology/PRD.md` §"Performance Targets".
- Postgres EXPLAIN — `postgresql.org/docs/16/sql-explain.html`.
- ClickHouse system.processes — `clickhouse.com/docs/en/operations/system-tables/processes`.
