# IP-010 — Cross-Cell Federation via Distributed Engine

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** infra (council-analytics)
**Authority ADRs:** ADR-0193, ADR-0049 cross-region replication and residency, ADR-0009 cell architecture, ADR-AN-003-row-level-tenant-isolation
**Depends on:** IP-001
**Status:** Planned

## Scope

ClickHouse `Distributed` engine provides transparent cross-shard query routing within a single cell — the canonical pattern for multi-shard reads. This IP wires Distributed tables on top of the per-tenant local tables and additionally configures the explicit `remote()` function for the rare cross-cell aggregation pattern (internal ops dashboards only). **Tenant queries never federate across cells** — tenant data is residency-bound per ADR-0049.

The IP has two halves:

1. **Within-cell distribution** — every per-tenant local table gets a parallel Distributed table that routes queries across the 3 shards.
2. **Cross-cell aggregation** — `fleet_internal.*` aggregate tables federate across cells via `remote()`, gated by Cedar to `Role::InternalAdmin` only.

## Deliverables

1. `Distributed` engine table for every per-tenant local table — created at tenant onboard time by IP-002's bootstrap controller using a render template.
2. `remote()` function configuration in `clickhouse-config` ConfigMap — per-cell endpoint allowlist with mTLS verification.
3. `fleet_internal` database created by IaC (separate from per-tenant databases).
4. Cross-cell federation pattern documented at `microservices/analytics/iac/clickhouse/federation/`.
5. Per-pack overlay enforcing federation boundaries — KR pack federates only across kr-* cells; EU pack only across eu-* cells.
6. Integration test verifying tenant principals are denied by Cedar from issuing `remote()`.

## Acceptance criteria

- `SELECT count() FROM tenant_ten_acme.events_distributed` succeeds within a cell (Distributed routes across shards transparently).
- `EXPLAIN PLAN SELECT ... FROM tenant_ten_acme.events_distributed` shows shard fan-out.
- Cross-cell `remote()` query is authorized only for the `internal_admin` principal; Cedar denies tenant principals with a `forbid` evidence entry in the audit-chain.
- KR-pack cluster CANNOT `remote()` to an EU-pack cluster — network policy denies the SYN; Cedar denies before network attempt.
- Cross-cell aggregation job runs nightly at 02:00 UTC and emits `fleet_internal.daily_capacity_aggregate` for the internal capacity-planning dashboard.

## Implementation tasks

### T1 — Distributed engine table template

File: `microservices/analytics/iac/clickhouse/mv-templates/distributed-table-template.sql`

```sql
-- Renders per per-tenant local table.
CREATE TABLE IF NOT EXISTS tenant_${tid}.events_distributed
ON CLUSTER analytics-clickhouse-1
AS tenant_${tid}.events
ENGINE = Distributed(
    'analytics-clickhouse-1',  -- cluster name (matches remote_servers.xml)
    'tenant_${tid}',            -- database
    'events',                   -- local table
    rand()                      -- sharding key — random fan-out for reads
);
```

The bootstrap controller (IP-002 T1) extends its onboard reconciliation to render this template for every local table. Idempotent: `IF NOT EXISTS`.

### T2 — remote_servers.xml configuration

File: `microservices/analytics/iac/helm/clickhouse-analytics/templates/configmap-remote-servers.yaml`

The ClickHouse `remote_servers.xml` declares the cluster topology. In `values.yaml`, the per-pack overlay supplies the allowed cross-cell endpoints:

```yaml
remoteServers:
  analytics-clickhouse-1:
    shards:
      - replicas:
          - host: chi-analytics-clickhouse-1-0-0.analytics.svc.cluster.local
            port: 9000
          - host: chi-analytics-clickhouse-1-0-1.analytics.svc.cluster.local
            port: 9000
      - replicas:
          - host: chi-analytics-clickhouse-1-1-0.analytics.svc.cluster.local
            port: 9000
          - host: chi-analytics-clickhouse-1-1-1.analytics.svc.cluster.local
            port: 9000
      - replicas:
          - host: chi-analytics-clickhouse-1-2-0.analytics.svc.cluster.local
            port: 9000
          - host: chi-analytics-clickhouse-1-2-1.analytics.svc.cluster.local
            port: 9000
  # Cross-cell endpoints (per-pack overlay supplies this list).
  cross-cell-kr:
    shards:
      - replicas:
          - host: clickhouse.analytics.kr-seoul-1.oyatie.io
            port: 9440  # secure
            secure: 1
```

For the KR pack overlay, `cross-cell-kr` includes ONLY kr-* cells. EU pack: ONLY eu-* cells. Cross-region federation is structurally prevented by the absence of cross-region endpoints in the configmap.

### T3 — Cedar policy for cross-cell

File: `microservices/analytics/policy/residency.cedar` (already authored — see §"residency-no-cross-cell-federation-from-tenant").

```cedar
@id("residency-no-cross-cell-federation-from-tenant")
forbid (
  principal,
  action == Action::"CrossCellQuery",
  resource
) unless {
  principal in Role::"InternalAdmin"
};
```

The adapter layer (IP-003) detects `remote()` in the SQL renderer and emits an Action::"CrossCellQuery" check before dispatch.

### T4 — Fleet-internal aggregate table

File: `microservices/analytics/iac/clickhouse/mv-templates/fleet-internal-tables.sql`

```sql
CREATE DATABASE IF NOT EXISTS fleet_internal ON CLUSTER analytics-clickhouse-1;

-- Per-cell daily capacity aggregate.
CREATE TABLE IF NOT EXISTS fleet_internal.daily_capacity_aggregate_local
ON CLUSTER analytics-clickhouse-1
(
    day Date,
    cell String,
    tenant_id String,
    total_rows UInt64,
    total_bytes UInt64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(day)
ORDER BY (cell, day, tenant_id)
TTL day + INTERVAL 90 DAY DELETE;

-- Row-level policy per ADR-AN-003.
CREATE ROW POLICY fleet_internal_tenant_scope_admin
ON fleet_internal.daily_capacity_aggregate_local
USING 1
TO Role::InternalAdmin;

CREATE ROW POLICY fleet_internal_tenant_scope_deny_others
ON fleet_internal.daily_capacity_aggregate_local
USING 0
TO ALL EXCEPT Role::InternalAdmin;
```

### T5 — Cross-cell aggregation job

File: `crates/oya-analytics-cross-cell-aggregator-app/` (new crate). Cron-style binary invoked daily at 02:00 UTC.

```rust
// crates/oya-analytics-cross-cell-aggregator-app/src/main.rs (sketch)
//
// 1. For each configured peer cell in the same residency boundary:
//    SELECT day, '${cell}' AS cell, tenant_id, count() AS total_rows, sum(bytes_on_disk) AS total_bytes
//    FROM cluster('cross-cell-${pack}', tenant_*.events_distributed)
//    WHERE day = yesterday()
//    GROUP BY day, tenant_id;
// 2. INSERT INTO fleet_internal.daily_capacity_aggregate_local.
// 3. Emit `oya.analytics.fleet.daily_capacity_emitted.v1` to the outbound channel.
```

Run via Kubernetes CronJob:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: cross-cell-aggregator
  namespace: analytics
spec:
  schedule: "0 2 * * *"
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: oya-analytics-cross-cell-aggregator
          containers:
            - name: aggregator
              image: ghcr.io/oyatie/analytics-cross-cell-aggregator:<sha>
          restartPolicy: OnFailure
```

### T6 — Integration test — cross-tenant denial

File: `crates/oya-analytics-cross-cell-aggregator-app/tests/cross_cell_denial.rs`

```rust
#[tokio::test]
async fn test_tenant_principal_cannot_remote() {
    let adapter = ClickHouseOlapClient::for_test().await;
    let tenant_principal = Principal::test_tenant("ten_acme");
    let err = adapter
        .raw_sql(&tenant_principal, "SELECT * FROM remote('eu-frankfurt-1', tenant_ten_bryan.events)")
        .await
        .expect_err("should be denied");
    assert!(matches!(err, KernelError::CedarForbid(_)));
}

#[tokio::test]
async fn test_internal_admin_can_remote_within_pack() {
    let adapter = ClickHouseOlapClient::for_test_kr_pack().await;
    let admin = Principal::test_internal_admin();
    let result = adapter
        .raw_sql(&admin, "SELECT count() FROM remote('cross-cell-kr', system.uptime)")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_internal_admin_cannot_remote_across_residency() {
    // KR-pack admin → EU-pack endpoint: should be denied (endpoint not in remote_servers.xml).
    let adapter = ClickHouseOlapClient::for_test_kr_pack().await;
    let admin = Principal::test_internal_admin();
    let err = adapter
        .raw_sql(&admin, "SELECT count() FROM remote('cross-cell-eu', system.uptime)")
        .await
        .expect_err("eu endpoint not allowlisted in kr pack");
    assert!(err.to_string().contains("unknown cluster") || err.to_string().contains("residency"));
}
```

### T7 — NetworkPolicy

File: `microservices/analytics/iac/helm/clickhouse-analytics/templates/network-policy-cross-cell.yaml`

Cross-cell egress allowlist: only mTLS endpoints in the same residency boundary. Implemented via Cilium ClusterMesh routes; the NetworkPolicy denies all egress except the allowlist.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: clickhouse-cross-cell-egress
  namespace: analytics
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/component: clickhouse-server
  policyTypes:
    - Egress
  egress:
    # Allow within-cell.
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: analytics
    # Allow cross-cell within residency pack — per-pack overlay supplies the FQDN allowlist.
    # Default (this file): no cross-cell allowed.
```

## Out of scope

- Tenant-principal cross-cell federation (forbidden by design).
- Federation across residency boundaries (forbidden by ADR-0049).
- Bidirectional cross-cell replication of per-tenant data (deferred — phase 2 DR feature).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Cross-cell endpoint unreachable | aggregator job fails | retry; CronJob next-run; alert if 3 consecutive failures |
| Distributed table missing on one shard | query fails with "unknown table" | bootstrap controller re-reconciles |
| `remote()` from tenant principal | Cedar forbid + audit event | denied at API gateway; no SQL dispatch |
| Cross-pack federation attempt | NetworkPolicy denies SYN | Cedar denies earlier; network is backstop |

## SLO commitment (downstream IP-014)

- Within-cell `Distributed` query latency overhead: ≤ 10ms p99 vs local table.
- Cross-cell aggregation job: completes within 30 min, p99.
- Aggregation freshness: < 26h (yesterday's data by 02:00 UTC + 30min).

## Rollback

- IP-010 is purely additive (Distributed tables alongside local tables).
- Rollback: drop the Distributed tables; queries fall back to local.
- The CronJob is independently disablable.

## Evidence emission

- Each `remote()` call emits `oya.analytics.audit.cross_cell_query.v1` with `(principal, target_cell, sql_hash, ts)`.
- Cross-cell aggregator emits `oya.analytics.fleet.daily_capacity_emitted.v1` per successful run.
- Cedar denials emit `oya.analytics.cedar.forbid.v1` with the policy fragment id (`residency-no-cross-cell-federation-from-tenant`).

## References

- ADR-0193 §"Cross-cell federation".
- ADR-0049 cross-region replication and residency.
- ADR-0009 cell architecture.
- ADR-AN-003-row-level-tenant-isolation.
- ClickHouse Distributed engine docs: https://clickhouse.com/docs/engines/table-engines/special/distributed.
