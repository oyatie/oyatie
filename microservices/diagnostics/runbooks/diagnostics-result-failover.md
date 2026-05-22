# Diagnostics Result Failover Runbook

## Purpose

Restore Diagnostics tenant service within the manifest DR target while preserving tenant isolation, audit evidence, and declared backup-substrate order.

## DR Contract

- RTO p99 seconds: 3600
- RPO p99 seconds: 300
- Active-active: true
- Backup substrates: postgres_wal_g, object_storage_versioned, audit_chain_merkle_seal

## Sequence

1. Freeze nonessential writes and capture the incident evidence id in the tenant audit channel.
2. Promote the healthiest cell replica for the primary state substrate before replaying projections or caches.
3. Restore or replay substrates in this order: postgres_wal_g -> object_storage_versioned -> audit_chain_merkle_seal.
4. Run service smoke checks against the public contracts and representative tenant workflows.
5. Reopen writes only after audit-chain evidence and tenant-facing status are recorded.

## Stop Condition

The service is within the declared RTO/RPO target, contract smoke checks pass, and the incident evidence id is attached to the post-drill record.
