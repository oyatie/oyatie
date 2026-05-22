# notes DR failover runbook

## Trigger

- Primary region or cell cannot meet the manifest RTO/RPO contract.
- Cedar, OpenBao, Postgres, Valkey, object storage, or audit-chain dependencies for notes are degraded beyond the service SLO.

## Authority

- ADR-0343 DR matrix.
- Manifest `dr` block in `microservices/notes/manifest.json`.
- Compliance-pack floors in `specs/compliance-pack-floors.json`.

## Procedure

1. Freeze destructive writes for the affected tenant cells unless active-active health is green.
2. Confirm the latest audit-chain seal and Postgres WAL-G/object-store recovery point.
3. Promote the warm region using the declared replication shape: active-active-multi-az-cross-region-warm.
4. Rehydrate declared backup substrates: postgres_wal_g, object_storage_versioned, valkey, openbao_seal_unseal, audit_chain_merkle_seal.
5. Run service smoke checks for public contracts and the highest-risk IP paths cited in the remediation note.
6. Remove the write freeze only after RPO <= 300s and RTO <= 3600s are evidenced.

## Rollback

Return traffic to the original region only after replication lag is zero, audit seals are continuous, and Cedar pack overlays match the promoted region.
