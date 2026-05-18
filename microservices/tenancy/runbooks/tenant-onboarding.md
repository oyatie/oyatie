---
doc_class: Runbook
title: Tenant onboarding (activation + RLS install + cell assignment)
microservice: tenancy
severity: "Sev-3 (single-tenant stuck) / Sev-2 (multi-tenant)"
status: Accepted
owner_team: axis-tenancy
date: 2026-05-17
related_artifacts:
  - microservices/tenancy/failure-modes.md (FM-05 activation stuck; FM-07 Valkey; FM-11 OpenBao; FM-15 overload)
  - microservices/tenancy/incident-response.md
  - microservices/tenancy/PRD.md (FR-03 activation ≤ 5min)
doc_status: published
---

# Runbook: Tenant onboarding

## Trigger

A tenant activation has not completed within the p99 5min target, OR has failed entirely. Detected by:
- `oya_tenancy_activation_duration_seconds{quantile="0.99"} > 600` sustained ≥ 5min, OR
- Manual tenant support ticket: "my activation hasn't completed."

## Severity

- Single tenant stuck < 30min with clear cause: Sev-3.
- Multi-tenant activation backlog OR persistent > 1h: Sev-2 (operational SLO breach risk).

## Activation steps (normal)

For reference, normal happy-path activation:

1. Platform-operator or tenant-self-serve `POST /tenants` to tenant-lifecycle-rest.
2. `tenant-lifecycle-usecase::CreateTenant` validates inputs + Cedar policy + jurisdiction.
3. `tenant-lifecycle-adapter-postgres::insert_tenant` writes the Tenant row.
4. `tenant-lifecycle-worker` picks up the activation task.
5. `cell-assignment-usecase::AssignCell` selects least-loaded cell.
6. Citus shard creation: `CREATE TABLE` partitioned + `pg_dist_partition` row.
7. RLS migration runner: for every tenant-bound table, `ALTER TABLE ... ENABLE/FORCE ROW LEVEL SECURITY` + `CREATE POLICY tenant_isolation`.
8. Post-migration validator: confirm every tenant-bound table is `force_rls=true` + has expected policy.
9. Workflow event emission: `TenantActivated` (consumed by every µservice for cache warm-up).
10. Audit-chain seal: `Ed25519` over the full activation envelope.
11. Tenant record status → `Activated`.

Total: ≤ 30s p50, ≤ 5min p99 (ADR-0118 + AC-01).

## Pre-checks (recovery)

1. Identify the stuck tenant: `cargo run -p oya-dev-cli -- tenancy status --tenant <tenant_id_hash>`.
2. Verify activation phase: query `tenants.status` + `tenancy.activation_log`.
3. Check Citus cluster: `SELECT * FROM citus_get_active_worker_nodes();` — all expected workers reporting.
4. Check Patroni: `patronictl list` — cluster healthy + leader stable.
5. Check OpenBao tenant-resolver: `curl https://openbao.<pack>.oyatie.dev/v1/sys/health` returns 200.
6. Check Valkey caches: both validate + cell-assignment caches reachable.

## Recovery Path A — Stuck migration (deadlock)

Cause: sqlx migration deadlock; e.g., concurrent activation taking exclusive locks on shared system table.

| Step | Action |
|---|---|
| 1 | Identify blocking session via `pg_locks` + `pg_stat_activity`. |
| 2 | If blocked > 5min: `pg_cancel_backend(<pid>)` (graceful); escalate to `pg_terminate_backend` if persists. |
| 3 | Restart activation worker for the stuck tenant. |
| 4 | Verify activation completes within next 30s; if not, escalate to Path B. |

## Recovery Path B — Citus shard creation failure

Cause: shard placement contention; insufficient worker capacity; Citus version mismatch.

| Step | Action |
|---|---|
| 1 | Verify worker capacity: `SELECT shardid, nodename FROM pg_dist_shard JOIN pg_dist_placement USING (shardid);` |
| 2 | If workers near capacity (>80%): add Citus worker (HPA scale-up); rebalance pending shards. |
| 3 | If Citus version mismatch (worker vs coordinator): engage ops-sre-reliability; coordinated upgrade per Citus docs. |
| 4 | Retry activation worker once cluster stable. |

## Recovery Path C — RLS migration failure (post-migration validation fails)

Cause: schema bug in RLS YAML manifest; migration runner skipped `FORCE` keyword.

| Step | Action |
|---|---|
| 1 | Inspect validation failure: `oya_tenancy_rls_post_migration_validation_failed{tenant_id=<>, table=<>}`. |
| 2 | If table missing `FORCE ROW LEVEL SECURITY`: run `ALTER TABLE <table> FORCE ROW LEVEL SECURITY` via DBA JIT (2-person rule + audit-chain seal). |
| 3 | If policy missing: re-run RLS YAML → DDL emission for that table only. |
| 4 | Re-run post-migration validator; verify pass. |
| 5 | Emit `TenantActivated` event manually if needed (worker won't re-emit). |
| 6 | Postmortem: why did the RLS YAML / runner skip enforcement? Update CI lane `rls-force-on-tenant-tables` if gap exposed. |

## Recovery Path D — Cell assignment loops (no cell selected)

Cause: all cells in jurisdiction at >80% capacity; cell-health probe failing.

| Step | Action |
|---|---|
| 1 | Verify cell-health metrics: `oya_tenancy_cell_health{pack=<>, cell_id=<>}`. |
| 2 | If all cells overloaded: provision new cell via cell-assignment-adapter-citus + Helm scale-out; await Patroni cluster readiness (~ 30min new cell). |
| 3 | If cell-health probe broken: restart probe loop; verify probe Cedar policy not denying. |
| 4 | Re-run cell-assignment-usecase for the stuck tenant. |

## Recovery Path E — OpenBao tenant-resolver unavailable

Per FM-11.

| Step | Action |
|---|---|
| 1 | Verify `cloud-secrets` µservice status. |
| 2 | Pause new-tenant activation globally; existing tenants unaffected (JWT verification uses cached pubkeys). |
| 3 | Engage cloud-secrets on-call; restore OpenBao tenant-resolver. |
| 4 | Resume activation queue once OpenBao back; backlog drains within minutes. |

## Recovery Path F — Valkey-validate cache cold (post-failover)

Sub-pattern of FM-07.

| Step | Action |
|---|---|
| 1 | Verify Valkey reachable + cluster size correct. |
| 2 | Allow Valkey to warm via natural traffic; p99 will recover within ~ 5min as cache fills. |
| 3 | If Postgres struggling under cache-miss load: tighten per-tenant validate rate limits temporarily; engage HPA to scale tenant-lifecycle-rest. |

## Verification

After completion:
- Tenant status = `Activated`.
- RLS policies on all tenant-bound tables: `SELECT relname, relforcerowsecurity FROM pg_class WHERE relname IN (<expected tables>);` — all true.
- `TenantActivated` event emitted (verify via audit-chain seal log).
- Tenant-validate hot-path: tenant can authenticate within 30s.
- Tenant operator notified of completion.

## Post-incident updates

- Postmortem within 5 business days (Sev-2+).
- If activation latency systematically near p99 ceiling: re-evaluate capacity-model.md formulae.
- If RLS validation failed: harden CI lane + runner.

## References

- `microservices/tenancy/PRD.md` FR-03 + AC-01.
- `microservices/tenancy/failure-modes.md` FM-05 + FM-07 + FM-11 + FM-15.
- `microservices/tenancy/incident-response.md`.
- `microservices/tenancy/policy/rls-isolation.md` (Invariant RLS-05 migration enforcement).
- Citus operational guide — `docs.citusdata.com`.
