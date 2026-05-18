# IP-024 — ClickHouse Cold-Tier Retention Policy

**Phase:** PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION
**Owner:** infra (axis-observability)
**Authority ADRs:** ADR-0193 §"TTL + partition rotation + cold tier", ADR-0049 cross-region replication, ADR-0184 storage tier layering, ADR-0186 observability backplane, ADR-0145 inter-microservice communication
**Depends on:** IP-021
**Status:** Planned
**Phase trace:** PHASE-02 §"Hot → cold tiered retention" (addendum lines 38-44).

## Scope

Wire per-table **TTL clauses** for the 90→365 day window, moving rows from hot (default storage) to cold (S3-compat / SeaweedFS) storage, then deleting at 365 days. Per ADR-0193 §"TTL + partition rotation + cold tier", this is implemented via `TTL <expr> TO DISK 's3_cold'` for the hot→cold transition and `TTL <expr> DELETE` for the final purge.

Per-µservice telemetry retention is intentionally **shorter** than tenant-facing audit retention (which lives in the analytics µservice cluster, per ADR-0184). Observability cold tier is for SRE retrospective analysis; it is not a compliance retention surface.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/observability/iac/helm/clickhouse/values.yaml` | edit | extend storage_policy block (lines ~50-100) | infra |
| `microservices/observability/iac/kustomize/components/clickhouse-storage-policy/configmap.yaml` | create | 1-80 | XML storage policy mounted at `/etc/clickhouse-server/config.d/storage.xml` |
| `microservices/observability/iac/kustomize/overlays/pack-kr/clickhouse/storage-policy.yaml` | create | 1-50 | KR cold tier points at kr-* S3 bucket |
| `microservices/observability/iac/kustomize/overlays/pack-eu/clickhouse/storage-policy.yaml` | create | 1-50 | EU cold tier points at eu-* S3 bucket |
| `microservices/observability/contracts/clickhouse-tables/ttl-otel-metrics.sql` | create | 1-50 | ALTER TABLE adds TTL clause |
| `microservices/observability/contracts/clickhouse-tables/ttl-otel-logs.sql` | create | 1-50 | ALTER TABLE adds TTL clause |
| `microservices/observability/contracts/clickhouse-tables/ttl-otel-traces.sql` | create | 1-50 | ALTER TABLE adds TTL clause |
| `microservices/observability/contracts/clickhouse-tables/ttl-mv-targets.sql` | create | 1-100 | TTL on all 4 MV target tables |
| `microservices/observability/tests/integration/ttl_hot_to_cold_migration.rs` | create | 1-180 | rows age past 90d → on s3_cold |
| `microservices/observability/tests/integration/ttl_cold_read_latency.rs` | create | 1-140 | cold-tier read p99 ≤ 2s |
| `microservices/observability/tests/integration/ttl_delete_at_365d.rs` | create | 1-120 | rows past 365d dropped |
| `microservices/observability/tests/integration/ttl_pack_kr_residency.rs` | create | 1-160 | KR cold objects stay in kr-* |
| `microservices/observability/tests/integration/ttl_pack_eu_residency.rs` | create | 1-160 | EU cold objects stay in eu-* |
| `microservices/observability/runbooks/clickhouse-cold-tier-incident.md` | create | 1-180 | runbook for cold-tier outage |

## Retention matrix

| Signal | Hot retention (default disk) | Cold transition (s3_cold) | Final delete |
|---|---|---|---|
| `otel_metrics` | 0-90 days | 90 days | 365 days |
| `otel_logs` | 0-90 days | 90 days | 365 days |
| `otel_traces` | 0-30 days | 30 days | 180 days |
| `mv_ops_microservice_health_hourly_target` | 0-180 days | 180 days | 730 days |
| `mv_ops_per_cell_capacity_hourly_target` | 0-180 days | 180 days | 730 days |
| `mv_ops_per_tenant_cost_daily_target` | 0-365 days | 365 days | 2555 days (7y per finance) |
| `mv_ops_slo_burn_rate_observed_target` | 0-90 days | 90 days | 365 days |

(Per-tenant cost MV target is retained 7 years per finance / accounting policy; that's the longest retention here. All other ops-portal aggregates are SRE-scoped.)

## Storage policy XML (extract)

```xml
<storage_configuration>
  <disks>
    <s3_cold>
      <type>s3</type>
      <endpoint>http://seaweedfs-s3.<pack>.svc.cluster.local:8333/observability-clickhouse-cold/</endpoint>
      <access_key_id from_env="S3_COLD_ACCESS_KEY"/>
      <secret_access_key from_env="S3_COLD_SECRET_KEY"/>
      <metadata_path>/var/lib/clickhouse/disks/s3_cold_meta/</metadata_path>
    </s3_cold>
  </disks>
  <policies>
    <hot_to_cold>
      <volumes>
        <hot><disk>default</disk></hot>
        <cold><disk>s3_cold</disk></cold>
      </volumes>
      <move_factor>0.0</move_factor>
    </hot_to_cold>
  </policies>
</storage_configuration>
```

## Acceptance criteria

- Telemetry older than 90 days lives on `s3_cold` (verified by `system.parts.disk_name` inspection).
- Telemetry older than 365 days dropped via TTL (verified by `ttl_delete_at_365d.rs`).
- **Cold-tier read latency p99 ≤ 2s** for ops-portal-class queries (verified by `ttl_cold_read_latency.rs`).
- KR pack cold objects stored exclusively in kr-* S3 buckets; EU equivalent for eu-* (verified by `ttl_pack_kr_residency.rs` / `ttl_pack_eu_residency.rs`).
- TTL DDL is applied non-destructively (ALTER TABLE, no data move at apply time).
- Per ADR-0049 residency requirements, cold objects never leave the source pack's region.
- ClickHouse merge-thread CPU does not exceed 60% sustained during cold-tier migration windows.
- Per-tenant cost MV retention is 7 years (longest); all other tables capped at 365d-730d.

## Test plan

| Test | Verifies |
|---|---|
| `test_ttl_clause_applied_on_otel_metrics` | DDL contains TTL clause |
| `test_row_at_91d_on_s3_cold` | inject row with timestamp 91d ago → on s3_cold |
| `test_row_at_366d_dropped` | inject row with timestamp 366d ago → absent |
| `test_cold_read_latency_p99_2s` | query cold partition → p99 ≤ 2s |
| `test_cold_storage_pack_kr_residency` | KR pack cold object in kr-* bucket only |
| `test_cold_storage_pack_eu_residency` | EU pack cold object in eu-* bucket only |
| `test_ttl_dll_idempotent` | re-applying ALTER is no-op |
| `test_ttl_applied_during_low_qps_window` | TTL move job scheduled at 02:00-04:00 local |
| `test_merge_thread_cpu_under_60pct` | merge thread CPU bounded |
| `test_per_tenant_cost_7y_retention` | cost MV TTL set to 2555 days |
| `test_ttl_metrics_visible` | `clickhouse_ttl_moves_total` / `clickhouse_ttl_drops_total` increment |
| `test_cold_tier_outage_degrades_gracefully` | S3 unreachable → hot queries unaffected; cold queries error with clear message |

## Evidence emission

- **Audit chain (ADR-0145):** `clickhouse.ttl.{move,drop,policy_change}` events to `oya.observability.audit.clickhouse.ttl`.
- **Metrics:** `clickhouse_ttl_moves_total`, `clickhouse_ttl_drops_total`, `clickhouse_storage_disk_used_bytes{disk}`, `clickhouse_part_count_by_disk{disk}`.
- **Capacity report:** monthly `evidence/capacity/observability-clickhouse-tiering-<month>.json` showing hot/cold partition counts + bytes per table.
- **Residency attestation:** quarterly residency attestation that cold objects geography matches expected pack.

## Rollback procedure

1. **Mistaken TTL.** ALTER TABLE to remove or extend TTL. Already-moved rows on s3_cold can be moved back via `ALTER TABLE ... MOVE PART ... TO DISK 'default'` (slow; large partitions). Already-deleted rows are unrecoverable from this surface — restore from backup (IP-025).
2. **Cold-tier outage.** Cold queries fail with a clear error; hot queries unaffected. Ops portal degrades gracefully (older time ranges return partial data with a banner).
3. **Wrong pack residency.** Audit-chain alert fires; runbook quarantines the misplaced objects + initiates intra-region rebuild via the per-pack overlay; high-severity residency violation per ADR-0049.
4. **Storage cost overrun.** Monthly capacity report flags overruns; mitigation = bump `move_factor` or shorten hot retention.

## Blocking deps

- IP-021 (cluster).
- Per-cell SeaweedFS S3-compat bucket `observability-clickhouse-cold` provisioned with per-pack residency configured (per Fix-S).
- Per-pack KMS key for cold object encryption.
- ExternalSecret operator deployed.

## Exit criteria

All test rows green; 90d burn-in shows successful hot→cold migrations on real telemetry volume; cold-tier read latency SLO unburned; residency attestation accepted; runbook drilled.

## Out of scope

- Tenant-facing audit retention (analytics µservice).
- Backup of cold-tier (IP-025 covers backup of hot + cold).
- Per-tenant retention overrides (out of scope here; lives in analytics µservice).

## Observability mapping

| Signal | Metric | Alert |
|---|---|---|
| TTL moves | `clickhouse_ttl_moves_total` | — (informational) |
| TTL drops | `clickhouse_ttl_drops_total` | — |
| Disk usage by tier | `clickhouse_storage_disk_used_bytes{disk}` | `ClickHouseHotDiskFull` (> 85% sustained 15min) |
| Part count by disk | `clickhouse_part_count_by_disk{disk}` | `ClickHouseTooManyParts` (> 1000 per shard) |
| Merge thread CPU | `clickhouse_background_pool_used` | `ClickHouseMergeSaturated` (> 60% sustained 30min) |
| Cold read latency | `clickhouse_query_duration_seconds{disk="s3_cold"}` | `ClickHouseColdReadSlow` (p99 > 2s) |

## Cost model

| Tier | Storage cost/GB-month | Egress cost/GB | Typical retention |
|---|---|---|---|
| Hot (local SSD via PV) | ~$0.08-0.15 | n/a (in-cluster) | 0-90d |
| Cold (SeaweedFS S3-compat) | ~$0.02-0.04 | $0.005-0.01 | 90-365d |

Monthly capacity report (`evidence/capacity/observability-clickhouse-tiering-<month>.json`) attributes cost per signal type + per-pack. Cost overrun threshold = +20% over prior month → automated capacity-planning notification.

## References

- ADR-0193 §"TTL + partition rotation + cold tier".
- ADR-0049 — cross-region replication.
- ADR-0184 — storage tier layering.
- ADR-0186 — observability backplane.
- ADR-0145 — communication reform.
- Runbook: `microservices/observability/runbooks/clickhouse-cold-tier-incident.md`.
