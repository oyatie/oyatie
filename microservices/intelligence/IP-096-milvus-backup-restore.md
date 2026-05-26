# IP-096 — Milvus Backup + Restore Drill

**Phase:** PHASE-02-FOUNDRY-DATA-SUBSTRATE
**Owner:** infra (axis-foundry + ops-sre-reliability)
**Authority ADRs:** ADR-0152 RPO/RTO, ADR-0241 DR business-continuity portfolio policy, ADR-0192 §"Backup and disaster recovery", ADR-0145 inter-microservice communication, ADR-0184 storage tier layering
**Depends on:** IP-091
**Status:** Planned
**Phase trace:** PHASE-02 §"Backup + drill cadence" (addendum lines 60-66).

## Scope

Wire the **Milvus backup tool** (Apache-2.0; ships with Milvus) to emit per-collection backups to the per-cell SeaweedFS S3-compat bucket. Schedule:

- **Daily incremental** (segment-level delta) — 4h window starting 02:00 local cell time.
- **Weekly full** — Sunday 03:00 local; retains the last 4 weeks.

Run a **quarterly restore drill** against `tenant_test_tenant` to validate RPO ≤ 24h and RTO ≤ 30min. File the drill report under `evidence/dr-drills/milvus-<date>.json`.

## File targets

| Path | Action | Line range | Notes |
|---|---|---|---|
| `microservices/intelligence/iac/helm/milvus/values.yaml` | edit | existing backup block (lines ~80-110) | tune cron + retention |
| `microservices/intelligence/iac/kustomize/components/milvus-backup/cronjob.yaml` | create | 1-90 | k8s CronJob wrapping milvus-backup CLI |
| `microservices/intelligence/iac/kustomize/components/milvus-backup/rbac.yaml` | create | 1-50 | ServiceAccount + Role |
| `microservices/intelligence/iac/kustomize/components/milvus-backup/external-secret.yaml` | create | 1-40 | S3 credentials via ExternalSecret |
| `microservices/intelligence/iac/kustomize/components/milvus-backup/prometheus-rule.yaml` | create | 1-80 | alerts on backup failure / drift / SLO |
| `microservices/intelligence/runbooks/milvus-restore.md` | exists | already authored | runbook |
| `microservices/intelligence/runbooks/milvus-backup-drill.md` | create | 1-160 | step-by-step quarterly drill |
| `crates/oya-foundry-milvus-backup-watcher/Cargo.toml` | create | 1-30 | small daemon: lists backups + emits audit |
| `crates/oya-foundry-milvus-backup-watcher/src/main.rs` | create | 1-150 | composition root |
| `crates/oya-foundry-milvus-backup-watcher/src/list_backups.rs` | create | 1-100 | enumerates SeaweedFS prefix |
| `crates/oya-foundry-milvus-backup-watcher/src/audit_emit.rs` | create | 1-90 | emits `milvus.backup.{completed,failed}` |
| `crates/oya-foundry-milvus-backup-watcher/tests/integration/backup_audit.rs` | create | 1-120 | end-to-end |

## Backup schedule + retention matrix

| Backup type | Cadence | Retention | Storage target |
|---|---|---|---|
| Incremental (segment delta) | Daily 02:00 local | 30 days | `s3://milvus-backup/<cell-id>/incremental/<date>/` |
| Full | Weekly Sunday 03:00 local | 4 weeks | `s3://milvus-backup/<cell-id>/full/<date>/` |
| Quarterly drill snapshot | First Saturday of Mar/Jun/Sep/Dec | 1 year | `s3://milvus-backup/<cell-id>/drill/<date>/` |

## Acceptance criteria

- Daily incremental backup completes within **4h window** (verified by the cronjob duration metric).
- Weekly full completes within **8h window**.
- Quarterly drill: restore `tenant_test_tenant` collection in **< 30min** (RTO ≤ 30min).
- **RPO ≤ 24h** (matches ADR-0152 / ADR-0180 portfolio policy).
- Backup completion emits audit-chain event `milvus.backup.completed` with `{cell_id, backup_id, backup_type, bytes, duration_s, collection_count}`.
- Backup failure emits `milvus.backup.failed` + pages foundry-oncall.
- SeaweedFS bucket retention policy enforced (deletes incrementals > 30d, fulls > 4w; drill snapshots > 1y).
- Restore runbook drilled by foundry-oncall every quarter; drill report at `evidence/dr-drills/`.
- All backups encrypted at rest via SeaweedFS server-side encryption + per-cell KMS key.

## Test plan

| Test | Verifies |
|---|---|
| `test_daily_incremental_completes_in_4h` | duration cap |
| `test_weekly_full_completes_in_8h` | duration cap |
| `test_quarterly_drill_rto_30min` | restore time |
| `test_backup_audit_event_emitted` | audit-chain emission |
| `test_backup_failure_pages_oncall` | failure → PagerDuty alert |
| `test_retention_policy_enforced` | SeaweedFS lifecycle rule deletes stale backups |
| `test_restore_preserves_vector_count` | post-restore count matches pre-backup |
| `test_restore_preserves_partition_data` | partition `partition_pii` rows survive |
| `test_restore_handles_schema_evolution` | backup from old schema → restore on new schema with additive columns |
| `test_backup_encryption_at_rest` | SeaweedFS SSE applied to backup objects |
| `test_per_cell_kms_key_used` | per-cell KMS key isolates cells |
| `test_drill_report_emitted` | `evidence/dr-drills/milvus-<date>.json` written on every drill |

## Quarterly drill procedure (summary; full at `milvus-backup-drill.md`)

1. **Pre-drill (T-7 days).** Schedule slot; notify foundry-oncall + capacity; pick test tenant (default `tenant_test_tenant`).
2. **Pre-drill (T-1 day).** Note pre-drill collection state (vector_count, last_inserted_at, partition_count, sample_vector_hash).
3. **Drill day.** Run restore CLI per the runbook; time end-to-end; verify post-restore matches pre-drill state.
4. **Post-drill.** File `evidence/dr-drills/milvus-<date>.json` with: cell_id, backup_id used, RTO observed, RPO observed (delta between backup time and current time), success/fail, anomalies.
5. **Escalation.** If RTO > 1h or RPO > 24h, page capacity-planning + foundry-architect; file high-severity finding.

## Evidence emission

- **Audit chain (ADR-0145):** `milvus.backup.{completed,failed}`, `milvus.drill.{started,completed,failed}` events to `oya.foundry.audit.milvus.dr`.
- **Metrics:** `foundry_milvus_backup_duration_seconds{type}`, `foundry_milvus_backup_bytes{type}`, `foundry_milvus_backup_last_success_ts`, `foundry_milvus_backup_failures_total`.
- **Drill evidence:** `evidence/dr-drills/milvus-<cell-id>-<date>.json`.
- **Backup manifest:** every backup writes a sidecar `manifest.json` next to the backup payload in S3, listing collection_name, partition_list, vector_count, schema_hash, encryption_kms_key_id.

## Rollback procedure

1. **Restore from backup.** Per the runbook at `microservices/intelligence/runbooks/milvus-restore.md` — covers single-collection restore, partial-tenant restore, full-cluster DR.
2. **Backup tool failure.** Rollback the milvus-backup tool image tag via Helm; cronjob picks up new image at next run; in the interim, run manually via `kubectl exec milvus-backup-0 -- milvus-backup ...`.
3. **Corrupted backup detected.** Quarantine in `s3://milvus-backup/<cell-id>/quarantine/`; use the previous successful backup; page foundry-oncall to investigate root cause.
4. **Drill fails mid-run.** Abort drill; restore the test tenant from a known-good backup; file high-severity finding; reschedule drill within 14 days.

## Blocking deps

- IP-091 (cluster) accepted.
- SeaweedFS S3-compat bucket `milvus-backup` provisioned per cell + lifecycle rules configured (per Fix-S).
- Per-cell KMS key minted for backup encryption.
- ExternalSecret + ExternalSecretStore operator deployed (per supervisor µservice IaC).

## Exit criteria

First-quarter drill completes within RTO + RPO targets; drill report at `evidence/dr-drills/`; foundry-oncall + capacity-oncall have signed the drill report; PrometheusRule loaded; daily + weekly cron run for 30 consecutive days with 0 failures.

## Backup manifest schema (sidecar `manifest.json`)

```json
{
  "schema_version": 1,
  "cell_id": "<cell-id>",
  "backup_id": "<uuidv7>",
  "backup_type": "incremental|full",
  "base_backup_id": "<uuid|null>",
  "collection_count": 1234,
  "collections": ["tenant_ten_acme__rag_corpus", ...],
  "vector_count_total": 1234567890,
  "bytes_total": 123456789012,
  "schema_hash": "<blake3-of-collection-schemas>",
  "encryption_kms_key_id": "<kms-key-arn>",
  "started_at": "2026-05-18T02:00:00Z",
  "completed_at": "2026-05-18T03:42:15Z",
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
| Concurrent collection backups | 4 | n/a |

Backup window 02:00-06:00 local; runs as a high-priority kubernetes Job. Watcher daemon runs always-on; emits audit events + Prometheus metrics for backup health.

## Observability mapping

| Signal | Metric | Alert |
|---|---|---|
| Backup duration | `foundry_milvus_backup_duration_seconds{type}` | `MilvusBackupSlow` (> 4h incremental, > 8h full) |
| Last success age | `foundry_milvus_backup_last_success_ts` | `MilvusBackupStale` (no success in 36h) |
| Failures | `foundry_milvus_backup_failures_total` | `MilvusBackupFailed` (any failure → page) |
| Bytes total | `foundry_milvus_backup_bytes{type}` | — |

## References

- ADR-0192 §"Backup and disaster recovery".
- ADR-0152 — RPO/RTO targets.
- ADR-0241 — DR business-continuity portfolio policy.
- ADR-0184 — storage tier layering.
- ADR-0145 — communication reform.
- Runbooks: `milvus-restore.md`, `milvus-backup-drill.md`.
- Upstream tool: github.com/zilliztech/milvus-backup (Apache-2.0).

## Wave 15 counterpart anchor

- Counterparts: Snowflake Cortex Search, Databricks Vector Search, OpenAI vector stores, and Palantir AIP ontology retrieval.
- Gap closure: this IP closes Foundry retrieval/vector substrate for tenant-isolated agent grounding and eval replay.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
