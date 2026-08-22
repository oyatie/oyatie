---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-availability-cache-rebuild
status: Accepted
date: 2026-05-17
owner_team: axis-calendar + ops-sre-reliability
severity_applicable: [Sev-1, Sev-2]
related_failure_modes: [FM-03, FM-06]
related_dashboards: [availability-lookup-rate]
doc_status: published
---

# Runbook — Availability Cache Rebuild

## When this runbook fires

- `calendar_availability_cache_hit_ratio` < 30% for > 5min, OR
- `cross_tenant_availability_p99_ms` > 500ms for > 5min, OR
- Valkey primary partition / failover event, OR
- Cross-tenant grant revocation detected without cache invalidation.

## Symptoms

- Tenant interactive scheduling slow (lookups > 500ms).
- Postgres connection-pool spike (cache misses fall through to DB).
- Valkey CPU / memory high.
- Stale cross-tenant projection served post-revocation.

## Probable causes

1. Synchronous TTL expiry across many tenants (cache stampede).
2. Valkey shard failure / failover.
3. Cross-tenant grant revocation event dropped or not propagated.
4. Cardinality spike on new tenant onboarding.

## Triage (within 15 min)

1. Acknowledge OnCall page.
2. Check Grafana dashboard `availability-lookup-rate`.
3. Identify cache hit ratio + Valkey health:
   ```bash
   kubectl exec -n calendar valkey-0 -- valkey-cli INFO replication
   kubectl exec -n calendar valkey-0 -- valkey-cli INFO memory
   ```
4. Check stampede candidate: which `(tenant, attendees, window)` tuples hot?
   ```promql
   topk(10, sum by (tenant_id_hashed) (rate(calendar_availability_cache_miss_count_total[5m])))
   ```
5. If grant-revocation-without-invalidation: identify affected grants:
   ```bash
   oya calendar grant audit --status revoked --since 1h
   ```

## Mitigation steps

### Step 1 — Enable single-flight (per (tenant, attendees, window))

Already on by default in PHASE-01 CS-05. If somehow off, flip via feature flag:

```bash
oya calendar feature-flag set --name availability_single_flight --value on
```

### Step 2 — Warm cache from Postgres

```bash
oya calendar cache warm --tenant <hashed-id> --range "now+0h to now+72h" --audit-reason "RB-availability-cache-rebuild"
```

### Step 3 — Scale Valkey shards if memory > 80%

```bash
helm upgrade calendar-valkey ./iac/helm/valkey --set shardCount=5
```

### Step 4 — If grant-revocation missed invalidation

Force-invalidate affected keys:

```bash
oya calendar grant invalidate --grant-id <id> --audit-reason "RB-availability-cache-rebuild"
```

Or if widespread, rotate cache prefix (forces full cold-start):

```bash
oya calendar cache rotate-prefix --pack <pack> --audit-reason "RB-availability-cache-rebuild"
```

### Step 5 — If Valkey shard down

Patroni-managed failover should auto-promote. Verify:

```bash
kubectl get pods -n calendar -l app=calendar-valkey
```

Manual promotion (last resort, 2-person rule):

```bash
oya calendar valkey promote --shard <n> --approver <ops-security-id> --audit-reason "RB-availability-cache-rebuild"
```

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `calendar_availability_cache_hit_ratio` | > 80% | within 15 min |
| `cross_tenant_availability_p99_ms` | < 500ms | within 5 min |
| Valkey memory util | < 70% | sustained |
| Cross-tenant grant invalidation lag | < 5s | sustained |

## Post-incident review

- Was the TTL jitter sufficient?
- Did grant-revocation event chain include cache-purge?
- Should cardinality limit be lowered for new tenants?
- Update threat-model.md T-D-02 + T-T-04 mitigations if needed.

## Drills

- Quarterly simulated cache-miss storm in staging.
- Annual grant-revocation race-condition test.

## References

- `failure-modes.md` FM-03 + FM-06.
- `threat-model.md` T-D-02, T-T-04.
- `dashboards/availability-lookup-rate.json`.
