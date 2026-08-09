# Quarterly Restore Drill — Analytics

**Authority:** ADR-0152, ADR-0193 §"Backup and disaster recovery", IP-012
**Cadence:** Quarterly
**Last reviewed:** 2026-05-18

## Pre-drill

1. Pick a non-production tenant id (`tenant_test_drill_<quarter>`).
2. Snapshot current state: row count + last_modified per table.
3. Identify the most recent daily backup id.

## Drill steps

1. Drop the test tenant's database in the secondary cell:
   ```sql
   DROP DATABASE tenant_test_drill_<quarter> SYNC;
   ```
2. Restore from backup:
   ```sql
   RESTORE DATABASE tenant_test_drill_<quarter> FROM S3('http://seaweedfs-s3.observability.svc.cluster.local:8333/clickhouse-backup-analytics/<backup-id>', '<key>', '<secret>');
   ```
3. Verify row counts + last_modified match snapshot.
4. Time the operation (target: <30min).
5. Re-run a sample tenant-facing dashboard query against the restored database → expect identical results to pre-drill snapshot.

## Post-drill

- File `evidence/dr-drills/analytics-clickhouse-<date>.json` with `(operator, RTO, RPO, success, anomalies)`.
- Escalate if RTO > 1h or RPO > 24h.
- Update this runbook with any procedural changes discovered.

## References

- ADR-0152 — RPO/RTO canonical (RPO ≤ 24h, RTO ≤ 1h per tenant).
- IP-012 — Backup tool + restore drill.
