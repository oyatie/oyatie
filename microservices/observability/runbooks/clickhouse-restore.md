# ClickHouse Restore Runbook — Observability

**Authority:** ADR-0193, ADR-0152
**Last reviewed:** 2026-05-18

## Restore single table

```bash
kubectl exec -n observability chi-oya-observability-clickhouse-... -- clickhouse-client --query "
  RESTORE TABLE observability.telemetry_rollup_hourly
  FROM S3('http://seaweedfs-s3.observability.svc.cluster.local:8333/clickhouse-backup/<backup-id>/', '<access-key>', '<secret-key>')
"
```

## Quarterly drill

- [ ] Pick a non-critical telemetry table.
- [ ] Restore from last week's backup.
- [ ] Time the operation.
- [ ] Verify row count matches pre-drop snapshot.
- [ ] File post-drill report at `evidence/dr-drills/observability-clickhouse-<date>.json`.

## Full cluster recovery

Mirrors `microservices/foundry/runbooks/milvus-restore.md` shape but for ClickHouse. Expected RTO 2-4h per cell.
