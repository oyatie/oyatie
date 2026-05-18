# Milvus Restore Runbook

**Authority:** ADR-0192 §"Backup and disaster recovery", ADR-0152 RPO/RTO
**Last reviewed:** 2026-05-18

## Restore single collection from backup

```bash
# 1. List backups
kubectl exec -n foundry milvus-backup-0 -- \
  milvus-backup list --bucket milvus-backup --prefix cells/<cell-id>/

# 2. Restore
kubectl exec -n foundry milvus-backup-0 -- \
  milvus-backup restore \
    --collection tenant_ten_acme__rag_corpus \
    --backup-name <backup-id> \
    --target-collection tenant_ten_acme__rag_corpus_restored

# 3. Verify count
kubectl exec -n foundry milvus-proxy-0 -- \
  /milvus/bin/milvus_cli show collection tenant_ten_acme__rag_corpus_restored

# 4. Cutover via rename (Milvus 2.6 supports atomic collection rename)
```

## Quarterly drill checklist

- [ ] Pick test tenant (typically `tenant_test_tenant`).
- [ ] Note pre-drill collection state (`vector_count`, last `inserted_at`).
- [ ] Run restore.
- [ ] Verify post-restore matches pre-drill state.
- [ ] Time the operation end-to-end.
- [ ] File post-drill report at `evidence/dr-drills/milvus-<date>.json`.
- [ ] If RTO > 1h or RPO > 24h, escalate to capacity planning.

## Full cluster recovery (DR)

If a cell's entire Milvus cluster is lost (storage volume corruption, etc.):

1. Spin up replacement cluster via `helm install foundry-milvus ...` in the recovery cell.
2. For each backup in the cell's backup bucket, restore in tenant-priority order.
3. Per-tenant cutover: redirect the `MilvusVectorStore` adapter's endpoint at the recovery cluster.
4. Resume ingest from the Pulsar consumer offset persisted at backup time.

Expected RTO for full cluster recovery: 4-8h per cell depending on collection count.
