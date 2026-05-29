# IP-006 — Cold-Tier S3 Disk + TTL Retention

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** infra (council-analytics)
**Authority ADRs:** ADR-0193 §"TTL + partition rotation + cold tier", ADR-0049 residency, ADR-AN-001-ttl-policy
**Depends on:** IP-001
**Status:** Planned

## Scope

Configure ClickHouse 26.3's native S3 disk and tiered storage policies. Emit `TTL ... TO DISK 's3_cold'` clauses on per-tenant tables. Verify hot→cold migration runs without query-time disruption. Tighten per-pack overlays so that cold-tier endpoints are residency-bound (KR data lives in kr-* S3; EU in eu-* S3; HIPAA-attested for us-healthcare).

Per ADR-AN-001-ttl-policy, the TTL values are workload-class-specific (audit log 90d→7yr; KPI 90d→1yr; billing 30d→7yr). This IP enforces those values via the Helm chart's `values.workloadTtl` projection.

## Deliverables

1. `storage_configuration` overlay in the Helm chart at `microservices/analytics/iac/helm/clickhouse-analytics/templates/clickhouse-installation.yaml` (already in template files).
2. Per-tenant table DDL convention with explicit `TTL ... TO DISK 's3_cold'` + `TTL ... DELETE` for compliance retention drop.
3. Per-pack residency overlay (already authored at `iac/kustomize/overlays/pack-*/cold-tier-patch.yaml`).
4. Hot→cold migration smoke test at 1TB scale.
5. CI lane verifying every per-tenant table DDL emits a TTL clause matching its workload class.
6. Cold-tier health PrometheusRule (S3 5xx rate, cold-tier query p99).

## Acceptance criteria

- Tables created with `TTL emitted_at + INTERVAL 90 DAY TO DISK 's3_cold'` migrate older partitions to cold tier (verified via `system.parts.disk_name` after a forced TTL pass).
- `SELECT count() FROM tenant_X.audit_events WHERE emitted_at < now() - INTERVAL 91 DAY` reads from `s3_cold` (verified via `system.query_log.read_bytes` correlated with `system.parts.disk_name`).
- 7-year compliance retention enforced via `TTL emitted_at + INTERVAL 7 YEAR DELETE` — verified at smoke test.
- Cold-tier read p99 ≤ 2s for 1-day partitions (matches IP-008 cold-tier SLO).
- Per-pack overlays bind cold tier to the correct regional S3 endpoint (KR → seaweedfs-s3-kr; EU → seaweedfs-s3-eu; etc.).
- TTL-presence CI lane denies a PR adding a new per-tenant table without a TTL clause.

## Implementation tasks

### T1 — storage.xml configuration

Already in `microservices/analytics/iac/helm/clickhouse-analytics/templates/clickhouse-installation.yaml` (the `files.storage.xml` configmap entry):

```xml
<storage_configuration>
  <disks>
    <s3_cold>
      <type>s3</type>
      <endpoint>http://seaweedfs-s3.observability.svc.cluster.local:8333/clickhouse-cold/</endpoint>
      <access_key_id from_env="S3_ACCESS_KEY"/>
      <secret_access_key from_env="S3_SECRET_KEY"/>
    </s3_cold>
  </disks>
  <policies>
    <hot_cold>
      <volumes>
        <hot><disk>default</disk></hot>
        <cold><disk>s3_cold</disk></cold>
      </volumes>
    </hot_cold>
  </policies>
</storage_configuration>
```

Per-pack overlays override the `endpoint` to the regional S3.

### T2 — DDL convention (per workload class)

The DDL templates in `microservices/analytics/iac/clickhouse/mv-templates/` and `iac/clickhouse/usage-source.sql` and `iac/clickhouse/mv-templates/audit-log-table.sql` already include the canonical TTL clauses per ADR-AN-001:

```sql
-- Audit log (90d hot, 7yr delete)
CREATE TABLE tenant_${tid}.audit_events (...)
ENGINE = ReplacingMergeTree(emitted_at)
PARTITION BY toYYYYMM(emitted_at)
ORDER BY (tenant_id, axis, emitted_at, event_id)
TTL emitted_at + INTERVAL 90 DAY TO DISK 's3_cold',
    emitted_at + INTERVAL 7 YEAR DELETE
SETTINGS storage_policy = 'hot_cold';
```

```sql
-- Workflow KPI rollup (90d hot, 1yr delete)
CREATE TABLE tenant_${tid}.workflow_hour (...)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMMDD(hour)
ORDER BY (tenant_id, hour)
TTL hour + INTERVAL 90 DAY TO DISK 's3_cold',
    hour + INTERVAL 365 DAY DELETE
SETTINGS storage_policy = 'hot_cold';
```

```sql
-- Billing daily (30d hot, 7yr delete)
CREATE TABLE tenant_${tid}.billing_day (...)
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(day)
ORDER BY (tenant_id, resource_type, day)
TTL day + INTERVAL 30 DAY TO DISK 's3_cold',
    day + INTERVAL 7 YEAR DELETE
SETTINGS storage_policy = 'hot_cold';
```

### T3 — Per-pack overlay (residency-bound cold endpoint)

Per-pack overlays at:

- `microservices/analytics/iac/kustomize/overlays/pack-kr/cold-tier-patch.yaml` — `endpoint: http://seaweedfs-s3-kr.observability.svc.cluster.local:8333/clickhouse-cold-kr/`
- `microservices/analytics/iac/kustomize/overlays/pack-eu/cold-tier-patch.yaml` — endpoint kr-frankfurt
- `microservices/analytics/iac/kustomize/overlays/pack-ksa/cold-tier-patch.yaml` — endpoint ksa
- `microservices/analytics/iac/kustomize/overlays/pack-uae/cold-tier-patch.yaml` — endpoint uae
- `microservices/analytics/iac/kustomize/overlays/pack-us-healthcare/cold-tier-patch.yaml` — endpoint AWS S3 + KMS encryption

The overlay patches the `ClickHouseInstallation` CR via strategic merge. Already authored.

### T4 — Smoke test: hot→cold migration

File: `scripts/iac/cold-tier-smoke-test.sh`

```bash
#!/bin/bash
set -euo pipefail

NAMESPACE=analytics
TENANT=test_cold_tier_smoke

# 1. Create a test tenant and table.
kubectl exec -n $NAMESPACE clickhouse-server-0 -- clickhouse-client --query "
CREATE DATABASE IF NOT EXISTS tenant_$TENANT ON CLUSTER analytics-clickhouse-1;
CREATE TABLE tenant_$TENANT.events_test (
    emitted_at DateTime, payload String
)
ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(emitted_at)
ORDER BY (emitted_at)
TTL emitted_at + INTERVAL 1 SECOND TO DISK 's3_cold'
SETTINGS storage_policy = 'hot_cold';
"

# 2. Insert 1TB of synthetic data.
kubectl exec -n $NAMESPACE clickhouse-server-0 -- clickhouse-client --query "
INSERT INTO tenant_$TENANT.events_test
SELECT
    now() - INTERVAL number SECOND AS emitted_at,
    arrayStringConcat(arrayMap(x -> randomString(1024), range(1024))) AS payload
FROM numbers(1000000);
"

# 3. Wait for TTL pass.
sleep 60
kubectl exec -n $NAMESPACE clickhouse-server-0 -- clickhouse-client --query "SYSTEM TTL ON CLUSTER analytics-clickhouse-1;"
sleep 30

# 4. Verify parts on cold tier.
PARTS_ON_COLD=$(kubectl exec -n $NAMESPACE clickhouse-server-0 -- clickhouse-client --query "
SELECT count() FROM system.parts WHERE database = 'tenant_$TENANT' AND active AND disk_name = 's3_cold';
")
if [ "$PARTS_ON_COLD" -lt 1 ]; then
    echo "FAIL: no parts on cold tier after TTL pass"
    exit 1
fi

# 5. Query cold-tier data and measure latency.
LATENCY_MS=$(kubectl exec -n $NAMESPACE clickhouse-server-0 -- clickhouse-client --query "
SELECT toMilliseconds(now() - now()) FROM tenant_$TENANT.events_test WHERE emitted_at < now() - INTERVAL 1 DAY LIMIT 100;
" --time)
echo "Cold-tier sample query latency: $LATENCY_MS ms"

# 6. Cleanup.
kubectl exec -n $NAMESPACE clickhouse-server-0 -- clickhouse-client --query "DROP DATABASE tenant_$TENANT ON CLUSTER analytics-clickhouse-1;"
```

### T5 — TTL-presence CI lane

File: `.github/workflows/ttl-presence-check.yml`

A CI lane that runs a SQL static analysis on every DDL file under `microservices/analytics/iac/clickhouse/`:

```bash
for sql in $(find microservices/analytics/iac/clickhouse -name '*.sql'); do
    if grep -E 'CREATE TABLE.*tenant_\$\{tid\}' "$sql" > /dev/null; then
        if ! grep -E 'TTL\s+\w+\s+\+\s+INTERVAL' "$sql" > /dev/null; then
            echo "FAIL: $sql defines per-tenant table without TTL clause"
            exit 1
        fi
    fi
done
```

The lane gates merge per ADR-AN-001 conformance.

### T6 — Cold-tier health PrometheusRule

In `microservices/analytics/iac/helm/clickhouse-analytics/templates/prometheus-rule.yaml`, add:

```yaml
- alert: ClickHouseColdTierS3ErrorRate
  expr: sum(rate(ClickHouseProfileEvents_S3RequestsErrors{cluster=~"analytics.*"}[5m])) > 0.05
  for: 5m
  labels:
    severity: page
    runbook: microservices/analytics/runbooks/cold-tier-latency.md
  annotations:
    summary: "Cold-tier S3 error rate > 5%"

- alert: ClickHouseColdTierQueryLatency
  expr: histogram_quantile(0.99, sum by (le) (rate(http_request_duration_seconds_bucket{route="/v1/audit-log/search",tier="cold"}[5m]))) > 2
  for: 5m
  labels:
    severity: page
    runbook: microservices/analytics/runbooks/cold-tier-latency.md
  annotations:
    summary: "Cold-tier query p99 > 2s"
```

### T7 — Per-table TTL refresh on workload-class change

If ADR-AN-001 is amended (e.g., audit log retention extended to 10 yr), the bootstrap controller's reconciliation lane emits `ALTER TABLE ... MODIFY TTL` for every per-tenant table:

```sql
ALTER TABLE tenant_${tid}.audit_events
MODIFY TTL emitted_at + INTERVAL 90 DAY TO DISK 's3_cold',
            emitted_at + INTERVAL 10 YEAR DELETE;
```

Idempotent.

## Out of scope

- WARM tier (intermediate between hot and S3) — deferred to phase 2.
- Cross-cell cold-tier replication — covered by IP-012 backup pipeline.
- Per-tenant TTL override (paid tenant_class contract overlay extends hot window) — deferred to phase 2.

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| TTL not firing (parts stuck on hot) | reconciliation lane; system.parts.disk_name | `SYSTEM TTL ON CLUSTER` force pass; runbook |
| S3 endpoint unreachable on TTL pass | TTL pass logged error | retry; alert; runbook cold-tier-latency.md |
| Wrong S3 endpoint in pack overlay | smoke test fails before deploy | gate at CI |
| TTL DELETE drops rows before retention age | impossible at well-formed DDL | CI lane verifies TTL interval matches workload class |

## SLO commitment (downstream IP-014)

- TTL pass cadence: every 24h (default ClickHouse); manual force via `SYSTEM TTL`.
- Cold-tier query p95 ≤ 2s (per `slos/audit-log-query-cold-latency.openslo.yaml`).
- Cold-tier S3 error rate: < 1% (alert at 5%).

## Rollback

- Storage policy change rollback: revert Helm chart values.
- Per-table TTL rollback: `ALTER TABLE ... MODIFY TTL` to the previous interval.
- No data loss on rollback — TTL TODISK is reversible (parts can move back to default disk if hot capacity available).

## Evidence emission

- Per TTL pass: `system.part_log` rows with `MOVE_PART` event.
- Per S3 5xx: Prometheus counter `ClickHouseProfileEvents_S3RequestsErrors`.
- Per smoke-test run: `evidence/smoke-tests/cold-tier-<date>.json`.

## References

- ADR-0193 §"TTL + partition rotation + cold tier".
- ADR-0049 cross-region replication and residency.
- ADR-AN-001-ttl-policy.
- `microservices/analytics/iac/kustomize/overlays/pack-*/cold-tier-patch.yaml`.
- ClickHouse S3 storage docs: https://clickhouse.com/docs/engines/table-engines/integrations/s3.
- ClickHouse TTL docs: https://clickhouse.com/docs/sql-reference/statements/alter/ttl

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-006-cold-tier-s3-ttl.md:22` - 6. Cold-tier health PrometheusRule (S3 5xx rate, cold-tier query p99).; `microservices/analytics/specs/IP-006-cold-tier-s3-ttl.md:29` - - Cold-tier read p99 ≤ 2s for 1-day partitions (matches IP-008 cold-tier SLO)..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-006-cold-tier-s3-ttl.md:250` - ## Evidence emission.
