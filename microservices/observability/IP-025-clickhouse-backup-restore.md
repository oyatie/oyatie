# IP-025 — Observability ClickHouse Backup + Restore Drill

**Phase:** PHASE-02-OBSERVABILITY-CLICKHOUSE-EXTENSION
**Owner:** infra (axis-observability + ops-sre-reliability)
**Authority ADRs:** ADR-0152 RPO/RTO, ADR-0193 §"Backup + restore", ADR-0241 DR business-continuity portfolio policy, ADR-0145 inter-microservice communication, ADR-0186 observability backplane
**Depends on:** IP-021
**Status:** Planned
**Phase trace:** PHASE-02 §"Backup + DR drill" (addendum lines 46-52).

## Scope

Use the **ClickHouse native `BACKUP`** command to back up to SeaweedFS S3-compat. Daily incremental + weekly full. Quarterly drill.

This mirrors the analytics µservice's IP-012 (`analytics-clickhouse-backup`) but is **scoped to the observability cluster** (telemetry rollups + ops-portal MVs). Loss of observability data is **recoverable via re-ingest** from Prometheus / Loki / Tempo within their hot windows — so observability's RPO is more lenient than analytics' (24h vs 1h).

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/observability/iac/helm/clickhouse/values.yaml` | edit | extend backup block (lines ~100-130) | infra |
| `microservices/observability/iac/kustomize/components/clickhouse-backup/cronjob.yaml` | create | 1-90 | k8s CronJob wrapping ClickHouse `BACKUP` SQL |
| `microservices/observability/iac/kustomize/components/clickhouse-backup/rbac.yaml` | create | 1-50 | ServiceAccount + Role |
| `microservices/observability/iac/kustomize/components/clickhouse-backup/external-secret.yaml` | create | 1-40 | S3 credentials |
| `microservices/observability/iac/kustomize/components/clickhouse-backup/prometheus-rule.yaml` | create | 1-80 | alerts on backup failure |
| `microservices/observability/runbooks/clickhouse-restore.md` | exists | already authored | restore runbook |
| `microservices/observability/runbooks/clickhouse-backup-drill.md` | create | 1-160 | quarterly drill |
| `crates/oya-observability-clickhouse-backup-watcher/Cargo.toml` | create | 1-30 | small daemon |
| `crates/oya-observability-clickhouse-backup-watcher/src/main.rs` | create | 1-130 | composition root |
| `crates/oya-observability-clickhouse-backup-watcher/src/list_backups.rs` | create | 1-100 | enumerates SeaweedFS prefix |
| `crates/oya-observability-clickhouse-backup-watcher/src/audit_emit.rs` | create | 1-80 | emits backup-event audit |
| `crates/oya-observability-clickhouse-backup-watcher/tests/integration/backup_audit.rs` | create | 1-120 | end-to-end |
| `microservices/observability/contracts/clickhouse-backup/backup-table-list.sql` | create | 1-60 | canonical table-list for backup |

## Backup schedule + retention matrix

| Backup type | Cadence | Retention | Storage target |
|---|---|---|---|
| Incremental (per-partition diff) | Daily 02:00 local | 30 days | `s3://observability-clickhouse-backup/<cell-id>/incremental/<date>/` |
| Full | Weekly Sunday 03:00 local | 4 weeks | `s3://observability-clickhouse-backup/<cell-id>/full/<date>/` |
| Quarterly drill snapshot | First Saturday of Mar/Jun/Sep/Dec | 1 year | `s3://observability-clickhouse-backup/<cell-id>/drill/<date>/` |

## Backup SQL (canonical)

```sql
-- Full backup
BACKUP DATABASE telemetry, DATABASE ops
TO S3('http://seaweedfs-s3.<pack>.svc.cluster.local:8333/observability-clickhouse-backup/<cell-id>/full/<date>/')
SETTINGS compression_method = 'lz4', compression_level = 1;

-- Incremental backup
BACKUP DATABASE telemetry, DATABASE ops
TO S3('http://seaweedfs-s3.<pack>.svc.cluster.local:8333/observability-clickhouse-backup/<cell-id>/incremental/<date>/')
SETTINGS base_backup = S3('<previous full or incremental URL>'),
         compression_method = 'lz4';
```

## Acceptance criteria

- Daily incremental backup completes within **4h window**.
- Weekly full completes within **8h window**.
- Quarterly drill: restore `telemetry.otel_metrics` (last 24h of data) in **< 30min** (RTO ≤ 30min).
- **RPO ≤ 24h** (matches ADR-0152 / ADR-0180 portfolio).
- Backup completion emits audit-chain event `clickhouse.backup.completed` with `{cell_id, backup_id, backup_type, bytes, duration_s, table_count}`.
- Backup failure emits `clickhouse.backup.failed` + pages observability-oncall.
- SeaweedFS bucket retention policy enforced (deletes incrementals > 30d, fulls > 4w; drill snapshots > 1y).
- Restore runbook drilled by observability-oncall every quarter; drill report at `evidence/dr-drills/`.
- All backups encrypted at rest via SeaweedFS server-side encryption + per-cell KMS key.
- Backup table list excludes ephemeral / debug tables; only the canonical tables (`otel_metrics`, `otel_logs`, `otel_traces`, all `mv_*_target` tables) are backed up.

## Test plan

| Test | Verifies |
|---|---|
| `test_daily_incremental_completes_in_4h` | duration cap |
| `test_weekly_full_completes_in_8h` | duration cap |
| `test_quarterly_drill_rto_30min` | restore time |
| `test_backup_audit_event_emitted` | audit-chain emission |
| `test_backup_failure_pages_oncall` | failure → PagerDuty alert |
| `test_retention_policy_enforced` | SeaweedFS lifecycle deletes stale backups |
| `test_restore_preserves_row_count` | post-restore row count matches pre-backup |
| `test_restore_preserves_mv_target_data` | MV target tables restored intact |
| `test_restore_handles_schema_evolution` | backup from old schema → restore on new schema (additive only) |
| `test_backup_encryption_at_rest` | SeaweedFS SSE applied |
| `test_per_cell_kms_key_used` | per-cell KMS key isolates cells |
| `test_drill_report_emitted` | `evidence/dr-drills/clickhouse-<date>.json` written |
| `test_table_list_excludes_ephemeral` | debug / temp tables excluded from backup |

## Quarterly drill procedure (summary; full at `clickhouse-backup-drill.md`)

1. **Pre-drill (T-7 days).** Schedule slot; notify observability-oncall + capacity; pick target shard for restore (recommend the canary shard).
2. **Pre-drill (T-1 day).** Note pre-drill state per table: row_count, partition_count, sample_hash.
3. **Drill day.** Run restore CLI per the runbook against a **separate database namespace** (`telemetry_restore_drill`); time end-to-end; verify post-restore matches pre-drill state.
4. **Post-drill.** File `evidence/dr-drills/clickhouse-<date>.json` with: cell_id, backup_id used, RTO observed, RPO observed, success/fail, anomalies.
5. **Cleanup.** Drop the `telemetry_restore_drill` namespace.
6. **Escalation.** If RTO > 1h or RPO > 24h, page capacity-planning + observability-architect.

## Evidence emission

- **Audit chain (ADR-0145):** `clickhouse.backup.{completed,failed}`, `clickhouse.drill.{started,completed,failed}` to `oya.observability.audit.clickhouse.dr`.
- **Metrics:** `observability_clickhouse_backup_duration_seconds{type}`, `observability_clickhouse_backup_bytes{type}`, `observability_clickhouse_backup_last_success_ts`, `observability_clickhouse_backup_failures_total`.
- **Drill evidence:** `evidence/dr-drills/observability-clickhouse-<cell-id>-<date>.json`.
- **Backup manifest:** every backup writes a sidecar `manifest.json` in S3, listing table_name, partition_list, row_count, schema_hash, encryption_kms_key_id.

## Rollback procedure

1. **Restore from backup.** Per the runbook at `microservices/observability/runbooks/clickhouse-restore.md`.
2. **Backup-tool failure.** Rollback the watcher image tag via Helm; cronjob runs the native ClickHouse `BACKUP` SQL directly (no separate tool needed for ClickHouse, unlike Milvus).
3. **Corrupted backup detected.** Quarantine in `s3://observability-clickhouse-backup/<cell-id>/quarantine/`; use the previous successful backup; page observability-oncall.
4. **Drill fails mid-run.** Abort; drop the drill namespace; file high-severity finding; reschedule drill within 14 days.
5. **Data loss recovery from re-ingest (preferred for observability).** Because observability data is recoverable via Prometheus / Loki / Tempo within hot windows, the **preferred** recovery for in-hot-window data loss is re-ingest from those backends, not backup restore. Backup restore is reserved for cold-tier data (> 90 days).

## Blocking deps

- IP-021 (cluster) accepted.
- SeaweedFS S3-compat bucket `observability-clickhouse-backup` provisioned per cell + lifecycle rules.
- Per-cell KMS key minted for backup encryption.
- ExternalSecret operator deployed.
- IP-024 storage policy applied (cold partitions are part of the backup surface).

## Exit criteria

First-quarter drill completes within RTO + RPO targets; drill report at `evidence/dr-drills/`; observability-oncall + capacity-oncall signed off; PrometheusRule loaded; daily + weekly cron run for 30 consecutive days with 0 failures.

## Out of scope

- Tenant-facing analytics backup (analytics µservice owns its own backup IP).
- Cross-region backup replication for observability (out of scope per ADR-0049 — observability data is replaceable from upstream backends).
- Per-tenant restore (no per-tenant residency for observability cluster; tenant_id is just a label here).

## Backup manifest schema (sidecar `manifest.json`)

```json
{
  "schema_version": 1,
  "cell_id": "<cell-id>",
  "backup_id": "<uuidv7>",
  "backup_type": "incremental|full",
  "base_backup_id": "<uuid|null>",
  "databases": ["telemetry", "ops"],
  "tables": ["otel_metrics", "otel_logs", "otel_traces",
             "mv_ops_microservice_health_hourly_target", ...],
  "row_count_total": 12345678901,
  "bytes_total": 234567890123,
  "schema_hash": "<blake3-of-table-schemas>",
  "encryption_kms_key_id": "<kms-key-arn>",
  "started_at": "2026-05-18T02:00:00Z",
  "completed_at": "2026-05-18T03:12:45Z",
  "audit_chain_seal_id": "<ed25519-seal-id>"
}
```

## Capacity sizing

| Resource | Backup cronjob | Watcher daemon |
|---|---|---|
| CPU request | 2 | 0.1 |
| CPU limit | 4 | 0.5 |
| Memory request | 4Gi | 256Mi |
| Memory limit | 8Gi | 512Mi |

Backup window 02:00-06:00 local; watcher daemon always-on.

## Observability mapping

| Signal | Metric | Alert |
|---|---|---|
| Backup duration | `observability_clickhouse_backup_duration_seconds{type}` | `ClickHouseBackupSlow` |
| Last success age | `observability_clickhouse_backup_last_success_ts` | `ClickHouseBackupStale` |
| Failures | `observability_clickhouse_backup_failures_total` | `ClickHouseBackupFailed` (any → page) |
| Bytes total | `observability_clickhouse_backup_bytes{type}` | — |

## References

- ADR-0152 — RPO/RTO targets.
- ADR-0193 §"Backup + restore".
- ADR-0241 — DR business-continuity portfolio policy.
- ADR-0145 — communication reform.
- ADR-0186 — observability backplane.
- Runbooks: `clickhouse-restore.md`, `clickhouse-backup-drill.md`.
- Upstream: ClickHouse native BACKUP command (since 24.3).
