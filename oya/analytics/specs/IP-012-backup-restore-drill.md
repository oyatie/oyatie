# IP-012 — Backup Tool + Restore Drill

**Phase:** PHASE-01-ANALYTICS-OLAP-BOOTSTRAP
**Owner:** infra (council-analytics + ops-sre-reliability)
**Authority ADRs:** ADR-0152 RPO/RTO canonical, ADR-0241-dr-business-continuity-portfolio-policy, ADR-0193, ADR-0039 supply chain
**Depends on:** IP-001
**Status:** Planned

## Scope

ClickHouse `BACKUP TABLE` native command to SeaweedFS S3-compat. Daily incremental + weekly full per ADR-0152 RPO ≤ 24h. Quarterly restore drill (gameday) verifying RPO/RTO compliance. All backups cosign-signed per ADR-0039.

## Deliverables

1. `BACKUP` cron schedule via Kubernetes CronJob — daily incremental at 02:30 UTC; weekly full at 02:30 UTC every Sunday.
2. Cosign-signed backup manifest per backup.
3. Restore runbook at `microservices/analytics/runbooks/restore-drill.md` (already authored — extended in this IP).
4. Quarterly restore-drill calendar entry + post-drill report template.
5. Backup-catalog state tracked at `evidence/backups/analytics/<cell>/<date>-<kind>.json`.
6. Cross-cell backup replication within the same residency boundary.

## Acceptance criteria

- Daily incremental backup completes within 4h window (target: 2h at sizing target).
- Weekly full backup completes within 8h window.
- Quarterly drill: restore `tenant_test_drill_${quarter}` from yesterday's backup in <30min.
- RPO ≤ 24h (matches ADR-0152 commitment) — verified by drill timestamp delta.
- RTO ≤ 1h per affected tenant — verified by drill.
- All backup manifests cosign-verifiable; failure to verify aborts the restore.

## Implementation tasks

### T1 — Backup CronJob

File: `microservices/analytics/iac/helm/clickhouse-analytics/templates/backup-cronjob.yaml`

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: clickhouse-backup-daily
  namespace: analytics
spec:
  schedule: "30 2 * * *"  # daily 02:30 UTC
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      backoffLimit: 0
      activeDeadlineSeconds: 14400  # 4h hard ceiling
      template:
        spec:
          serviceAccountName: oya-analytics-backup
          restartPolicy: Never
          containers:
            - name: backup
              image: ghcr.io/oyatie/analytics-backup:<sha>
              env:
                - { name: BACKUP_KIND, value: "incremental" }
                - { name: BACKUP_BUCKET, value: "clickhouse-backup-analytics" }
                - { name: S3_ENDPOINT, value: "http://seaweedfs-s3.observability.svc.cluster.local:8333" }
                - { name: COSIGN_KEY_REF, value: "openbao:secret/data/analytics/cosign-backup/key" }
              command: ["/usr/local/bin/oya-analytics-backup", "run", "--kind=incremental"]
---
apiVersion: batch/v1
kind: CronJob
metadata:
  name: clickhouse-backup-weekly-full
  namespace: analytics
spec:
  schedule: "30 2 * * 0"  # Sunday 02:30 UTC
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      backoffLimit: 0
      activeDeadlineSeconds: 28800  # 8h hard ceiling
      template:
        spec:
          serviceAccountName: oya-analytics-backup
          restartPolicy: Never
          containers:
            - name: backup
              image: ghcr.io/oyatie/analytics-backup:<sha>
              env:
                - { name: BACKUP_KIND, value: "full" }
                - { name: BACKUP_BUCKET, value: "clickhouse-backup-analytics" }
              command: ["/usr/local/bin/oya-analytics-backup", "run", "--kind=full"]
```

### T2 — Backup CLI

File: `crates/oya-analytics-backup-cli/` (new crate).

The binary issues:

```sql
-- Incremental (uses previous full as base).
BACKUP TABLE tenant_${tid}.events
TO S3('http://seaweedfs-s3.observability.svc.cluster.local:8333/clickhouse-backup-analytics/${cell}/${date}-incremental/', 'key', 'secret')
SETTINGS base_backup = S3('.../${last_full_id}');

-- Full (every Sunday).
BACKUP TABLE tenant_${tid}.events
TO S3('.../clickhouse-backup-analytics/${cell}/${date}-full/', 'key', 'secret');
```

Iterates over every per-tenant table (database catalog query `SHOW DATABASES LIKE 'tenant_%'`).

After backup completes, generates a manifest:

```json
{
  "backup_id": "${cell}-${date}-${kind}",
  "cell": "${cell}",
  "kind": "incremental | full",
  "started_at": "...",
  "completed_at": "...",
  "tables": [
    {"database": "tenant_ten_acme", "table": "events", "size_bytes": 12345678, "row_count": 1000000}
  ],
  "base_backup": "${prior_full_id}",
  "cosign_signature": "MEUCIQ..."
}
```

Manifest is cosign-signed via `cosign sign-blob --key openbao://...` and uploaded alongside the backup parts. Manifest path: `s3://clickhouse-backup-analytics/${cell}/${date}-${kind}/manifest.json`.

### T3 — Backup-catalog state

File: `evidence/backups/analytics/<cell>/<date>-<kind>.json` — committed per backup completion via the backup binary's emit-to-git pipeline (deferred to phase 2; for phase 1, kept as cell-local index).

### T4 — Restore CLI

File: `crates/oya-analytics-backup-cli/src/restore.rs`

```rust
pub async fn restore(opts: RestoreOpts) -> Result<()> {
    // 1. Pull manifest.
    let manifest = s3_get_manifest(&opts.backup_id).await?;
    // 2. Verify cosign signature.
    cosign_verify_blob(&manifest, &opts.cosign_pubkey).await?;
    // 3. Drop the target database (if requested).
    if opts.drop_first {
        clickhouse_exec("DROP DATABASE IF EXISTS ${db} ON CLUSTER ${cluster}").await?;
    }
    // 4. RESTORE statement.
    clickhouse_exec("RESTORE DATABASE ${db} FROM S3('...', 'key', 'secret')").await?;
    // 5. Verify row counts.
    for table in manifest.tables {
        let observed = clickhouse_exec(&format!("SELECT count() FROM {}.{}", table.database, table.table)).await?;
        if observed != table.row_count {
            return Err(RestoreError::RowCountMismatch);
        }
    }
    // 6. Emit `oya.analytics.backup.restored.v1` audit event.
    audit_emit("oya.analytics.backup.restored.v1", &manifest).await?;
    Ok(())
}
```

### T5 — Cross-cell replication

Daily backup uploaded to the primary cell's S3 endpoint. A separate Kubernetes CronJob copies the backup to a secondary cell within the same residency boundary:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: clickhouse-backup-replicate-cross-cell
  namespace: analytics
spec:
  schedule: "30 3 * * *"  # 1h after backup
  jobTemplate:
    spec:
      template:
        spec:
          containers:
            - name: replicate
              image: mcr.microsoft.com/azure-cli:latest  # or rclone
              command: ["rclone", "sync", "primary:clickhouse-backup-analytics/${date}", "secondary:clickhouse-backup-analytics/${date}"]
```

KR pack: primary kr-seoul-1 → secondary kr-busan-1. EU: eu-frankfurt-1 → eu-paris-1. US-healthcare: us-east-1 → us-west-1.

### T6 — Quarterly drill calendar

File: `evidence/dr-drill-calendar.md` — quarterly schedule.

| Quarter | Drill date | Tenant id | Operator |
|---|---|---|---|
| Q1-2026 | 2026-02-15 | tenant_test_drill_q1_2026 | TBD |
| Q2-2026 | 2026-05-15 | tenant_test_drill_q2_2026 | TBD |
| Q3-2026 | 2026-08-15 | tenant_test_drill_q3_2026 | TBD |
| Q4-2026 | 2026-11-15 | tenant_test_drill_q4_2026 | TBD |

### T7 — Drill post-report template

File: `evidence/dr-drills/_template.json`

```json
{
  "drill_id": "analytics-clickhouse-<cell>-<date>",
  "operator": "alice@oyatie",
  "started_at": "...",
  "completed_at": "...",
  "rto_observed_minutes": 22,
  "rto_target_minutes": 60,
  "rpo_observed_hours": 18,
  "rpo_target_hours": 24,
  "success": true,
  "anomalies": [],
  "row_count_match": true,
  "cosign_verified": true,
  "post_drill_query_match": true,
  "action_items": []
}
```

## Out of scope

- Per-row erasure (covered by IP-002 offboard + ADR-0038 DSR).
- Backup of system database tables (system.query_log etc. — separately backed up by observability µservice).
- Live point-in-time recovery beyond daily granularity (deferred — phase 2).

## Failure modes

| Mode | Detection | Mitigation |
|---|---|---|
| Backup CronJob > 4h | activeDeadlineSeconds kills | next-run retry; alert |
| S3 endpoint down | backup CLI error | retry; if persistent, page |
| Cosign key compromise | signing manifest fails | rotate via `runbooks/backup-key-rotation.md`; re-sign last 30 days |
| Backup manifest tampered | restore signature verify fails | reject; fall back to prior backup; forensic |
| Restore mismatch (row count) | restore CLI aborts | escalate; investigate |

## SLO commitment (downstream IP-014)

- Daily backup completion within 4h: 99.9%.
- Weekly full within 8h: 99.9%.
- Restore drill RTO < 1h: 100%.
- Restore drill RPO < 24h: 100%.

## Rollback

- Backup CronJob is independent; disable via Helm `backupCronjob.enabled=false`.
- Restore is a manual operation under runbook control; no automatic rollback semantics needed.

## Evidence emission

- Every backup emits `oya.analytics.backup.completed.v1` with `(backup_id, kind, tables, rows, bytes, cosign_signature, duration)`.
- Every restore emits `oya.analytics.backup.restored.v1` with `(backup_id, target_db, verified, duration)`.
- Quarterly drill emits the post-drill report into `evidence/dr-drills/analytics-clickhouse-<date>.json` and a one-line summary into the audit-chain.

## References

- ADR-0152 RPO/RTO canonical (RPO ≤ 24h, RTO ≤ 1h per tenant).
- ADR-0241-dr-business-continuity-portfolio-policy.
- ADR-0039 supply chain (cosign signing).
- ADR-0043 secrets (OpenBao for cosign keys).
- `microservices/analytics/runbooks/restore-drill.md`.
- ClickHouse BACKUP docs: https://clickhouse.com/docs/operations/backup

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/analytics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/analytics/specs/IP-012-backup-restore-drill.md:234` - ## SLO commitment (downstream IP-014).

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/analytics/specs/IP-012-backup-restore-drill.md:246` - ## Evidence emission.
