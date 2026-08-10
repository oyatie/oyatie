---
doc_class: Runbook
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0248
  - ADR-0253
  - ADR-0263
companion_docs:
  - console/runbooks/incident-command.md
  - console/slos/command-availability.openslo.yaml
  - microservices/ops-dashboard-control-center/capacity-model.md
  - console/runbooks/admin-action-rollback.md
planned_enforcement_ref: oya-governance-microservice-doc-set
---

# Runbook: Dashboard Performance Degradation

## A — Trigger conditions

- P99 request latency > 500ms sustained for ≥5 min (SLO budget burn alert).
- Command queue depth > 100: `oya_ops_control_center_command_queue_depth > 100`.
- Cedar eval P99 > 10ms (2× budget): `histogram_quantile(0.99, oya_ops_control_center_cedar_eval_duration_seconds_bucket) > 0.010`.
- SSE push backlog > 1000 events: `oya_ops_control_center_sse_backlog_events > 1000`.
- HPA not scaling (pod count at max but latency still high).
- Database connection pool saturation: `oya_ops_control_center_db_pool_waiting > 50`.

## B — Pre-checks

1. **[≤30s]** Check overall system health: `GET /ops/v1/health/detailed` → identify which subsystem is degraded.
2. **[≤30s]** Check current pod count: `kubectl get pods -n ops-dashboard -l app=ops-dashboard-control-center`.
3. **[≤30s]** Check SLO burn rate: `dashboards/ops-overview.json` → error budget panel.
4. **[≤30s]** Check if this correlates with a recent deployment: `GET /ops/v1/deployments?window=1h`.
5. **[≤30s]** Check upstream dependency health: observability µservice, tenancy µservice, policy-engine.

## C — Procedure

### Path A — Cedar eval latency elevated

1. **[≤2min]** Check Cedar policy bundle size: `GET /ops/v1/cedar/policy-bundle/stats` → if bundle > 1 MB, fragment proliferation.
2. **[≤2min]** Check cedar eval cache hit rate: `oya_ops_control_center_cedar_cache_hit_ratio`. If < 0.8: cache eviction issue.
3. **[≤5min]** If cache issue: rolling restart pods to warm cache. `kubectl rollout restart deployment/ops-dashboard-cedar-sidecar -n ops-dashboard`.
4. **[≤2min]** If bundle size issue: escalate to pack-author team to audit fragment count.

### Path B — Database pool saturation

1. **[≤2min]** Check PgBouncer pool stats: `SHOW POOLS;` on PgBouncer admin socket.
2. **[≤2min]** Check slow queries: `SELECT pid, query, state, wait_event_type FROM pg_stat_activity WHERE state != 'idle' ORDER BY duration DESC LIMIT 20;`
3. **[≤5min]** If long-running query: cancel with `SELECT pg_cancel_backend(pid)` — budget ≤5s wait first.
4. **[≤5min]** Scale up PgBouncer pool size: `kubectl edit configmap pgbouncer-config -n ops-dashboard` → increase `max_client_conn`.
   Rollback: revert configmap change.

### Path C — Pod count at HPA max, latency still high

1. **[≤2min]** Check if a single BC is the bottleneck: `oya_ops_control_center_request_duration_seconds{bc=~".+"}` — isolate the slowest BC.
2. **[≤2min]** Check upstream dependency: if `observability` µservice SLO is degraded, our reads will slow.
3. **[≤5min]** If upstream issue: enable circuit breaker on slow dependency. `kubectl annotate deployment ops-dashboard-cluster-health "circuit-breaker/enabled=true"`.
4. **[≤5min]** If self-inflicted: rollback recent deployment via `runbooks/admin-action-rollback.md`.

### Path D — SSE push backlog

1. **[≤2min]** Check SSE consumer connection count: `oya_ops_control_center_sse_active_connections`.
2. **[≤2min]** If too many stale connections: drain them. `POST /ops/v1/sse/drain-stale-connections` (T2 step-up required).
3. **[≤2min]** Check SSE payload size vs 32 KiB budget: `oya_ops_control_center_sse_payload_bytes_bucket`.
   If > 32 KiB average: payload explosion — identify noisy BC and reduce event frequency.

## D — Verification

- `histogram_quantile(0.99, oya_ops_control_center_request_duration_seconds_bucket) < 0.500` — ≤500ms P99.
- `oya_ops_control_center_command_queue_depth < 10`.
- SLO burn rate normal: `dashboards/ops-overview.json` → error budget panel showing recovery.

## E — Rollback

Each path documents its own rollback inline. General rollback: if degradation started with a deployment, roll back via `runbooks/admin-action-rollback.md`.

## F — Post-incident

- Capacity model update if peak load exceeded projections: `capacity-model.md §6 bottleneck analysis`.
- If Cedar bundle size was root cause: add bundle-size CI gate.
- SLO error budget consumed: document in quarterly SLO review.

## G — References

- `capacity-model.md`
- `ARCHITECTURE.md §capacity-math`
- `slos/command-availability.openslo.yaml`
- `dashboards/ops-overview.json`
