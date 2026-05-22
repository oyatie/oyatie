# Runbook — Capacity Rebalance

**Authority:** ADR-0193, ADR-0001 cohesion, capacity-model.md
**Owner:** ops-sre-reliability + council-analytics
**Trigger:** 70% of any hard ceiling (capacity-model.md §1) breached for 7 consecutive days, OR direct page from capacity-planning.
**Severity:** Sev 3 (planning) → Sev 2 (urgent action required)

## When to run

Page received: `ClickHouseCapacityThreshold` PrometheusRule. Inspect the cell-level metrics for the cell that paged.

## Pre-flight (run first)

1. Confirm which ceiling breached:
   ```
   kubectl get prometheusrule -n analytics analytics-capacity-thresholds -o yaml | grep threshold
   ```
2. Pull last 7 d of relevant time-series from the Grafana `analytics-overview` dashboard.
3. Confirm this is not a transient spike (one tenant misbehaving, since-recovered).

## Decision tree

### 5,000-tenant ceiling approached

```
tenant_count / 10000 ≥ 0.7 for 7 d
```

**Action:** Shard onto a second cell-local cluster.

1. Provision `analytics-clickhouse-2` via Helm:
   ```
   helm install analytics-clickhouse-2 microservices/analytics/iac/helm/clickhouse-analytics/ \
     --namespace analytics \
     --set cluster.name=analytics-clickhouse-2 \
     -f microservices/analytics/iac/helm/clickhouse-analytics/values-cluster-2.yaml
   ```
2. Update the connection-pool sharding map at `crates/oya-analytics-app/config/cluster-routing.yaml`:
   ```yaml
   clusters:
     - name: analytics-clickhouse-1
       weight: 1
       tenant_hash_mod: 2
       tenant_hash_residue: 0
     - name: analytics-clickhouse-2
       weight: 1
       tenant_hash_mod: 2
       tenant_hash_residue: 1
   ```
3. Roll out the app per `crates/oya-analytics-app/`.
4. New tenants land on `analytics-clickhouse-2` (the under-loaded cluster).
5. Tenant migration of existing tenants is **forbidden** during onboard transition; existing tenants stay where they are.
6. Verify routing: `kubectl logs -n analytics deployment/oya-analytics-app | grep "route to cluster"`.

### Hot tier 70% full

```
hot_tier_used_bytes / hot_tier_total_bytes ≥ 0.7 for 7 d
```

**Action:**

1. Verify TTL is firing (parts migrating to cold tier):
   ```
   clickhouse-client --query "SELECT name, disk_name, bytes_on_disk FROM system.parts WHERE database LIKE 'tenant_%' ORDER BY bytes_on_disk DESC LIMIT 50"
   ```
2. If TTL stuck: `SYSTEM TTL ON CLUSTER oya-cell` forces a TTL pass.
3. If TTL is firing but hot still growing: tighten the hot-window from 90 d → 60 d for demo_trial tenants:
   ```sql
   ALTER TABLE tenant_${tid}.events
     MODIFY TTL emitted_at + INTERVAL 60 DAY TO DISK 's3_cold';
   ```
4. If still growing: add NVMe shards (raise replica count from 6 → 8).

### Query QPS ≥ 70% of 50K fleet-wide

```
sum(rate(ClickHouseProfileEvents_Query[5m])) ≥ 35000 for 7 d
```

**Action:**

1. Add server replicas. Helm `values.yaml`:
   ```yaml
   replicas:
     server: 8  # up from 6
   ```
2. `helm upgrade`. New replicas auto-join the shards via Keeper.
3. Wait 10 minutes for new replica to catch up via `system.replication_queue`.

### Per-tenant ceiling breached (single tenant)

```
tenant_X rows ≥ 50B   OR   tenant_X tables ≥ 100
```

**Action:**

1. Account team review: is this tenant within their contract tier?
2. If yes: move tenant to a dedicated cell-local cluster (see "5,000-tenant" path).
3. If no: tier upgrade conversation.

### Concurrent queries per replica ≥ 100

```
max(ClickHouseMetrics_Query) ≥ 100
```

**Action:**

1. Audit query patterns: which workload is generating concurrent queries?
2. Per-tenant QUOTA review (IP-011): the offending tenant's `max_concurrent_user_queries` should be tightened.

## Communication

- Pre-emptive notice to customers if expected disruption: NONE for in-cluster scale-out; brief read-only window for tenant migration.
- Notify `#analytics-incidents` on declare and on complete.
- Capacity-planning ticket required to track the action and outcome.

## Verification

After the rebalance:

1. Capacity-planning dashboard returns to <70% on the breached ceiling.
2. No SLO burn observed for 24 h post-action.
3. Update `evidence/capacity-actions/<date>.json` with `(action, observed_before, observed_after, operator)`.

## Escalation

If the rebalance does not bring the ceiling below 70% within 7 days: page `capacity-planning` lead; consider cell-split (new cell in the same residency boundary).

## References

- ADR-0193, ADR-0009 cell architecture, ADR-0010 regional packs.
- `microservices/analytics/capacity-model.md` (canonical ceilings).
- `microservices/analytics/specs/IP-001-clickhouse-cluster-iac.md` (cluster IaC).
