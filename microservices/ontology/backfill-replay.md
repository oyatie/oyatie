---
doc_class: BackfillReplayPlan
title: Backfill + Replay (historical type-graph reconstruction)
microservice: ontology
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-ontology + ops-sre-reliability
deciders: axis-ontology, ops-sre-reliability, council-architecture, council-privacy
related_adrs: [ADR-0028, ADR-0059, ADR-0106, ADR-0117, ADR-0131]
related_artifacts:
  - microservices/ontology/runbooks/type-registry-migration.md
  - microservices/ontology/runbooks/clickhouse-rebalance.md
  - microservices/ontology/runbooks/cross-tenant-leak-recovery.md
  - microservices/ontology/runbooks/object-type-deprecation.md
doc_status: published
---

# Backfill + Replay Plan (ontology µservice)

## Purpose

Define the historical type-graph reconstruction procedures: replaying Kafka outbox to rebuild ClickHouse history-mirror, backfilling schema migrations across existing tenant Object Type instances, replaying audit-chain Merkle from raw events when a seal is contested, and restoring tenant Object Types from Postgres PITR backups after corruption or accidental delete.

## Use Cases

| Use case | Primary trigger | Source of truth | Target store |
|---|---|---|---|
| **ClickHouse mirror rebuild** | Mirror corruption; topology change; new analytics pipeline | Kafka outbox topics + Postgres | ClickHouse history-mirror |
| **Schema migration backfill** | Object Type schema evolution touching existing data | Existing Object Type tables | Same tables (rewritten under new schema_version) |
| **Audit-chain re-seal from outbox** | Merkle tamper detected (FM-13); seal disputed | Kafka outbox raw events | New audit-chain Merkle tree |
| **DSR cascade replay** | Subject erasure across historical Object Types + Links | Postgres + ClickHouse | Tombstone records |
| **PITR restore** | Accidental delete; data corruption | Postgres PITR archive | Postgres restoration database; cut-over after validation |
| **Pack-misroute cleanup** | FM-16; rows wrote to wrong pack | Postgres rows in wrong pack | Quarantine + reroute to correct pack |
| **Tenant data export (DSR Art. 15)** | Subject access request | Postgres + ClickHouse + audit-chain | JSON export per subject with provenance |

## ClickHouse Mirror Rebuild

### Trigger

- Mirror data corruption (block-SHA mismatch).
- Topology change (new shard count; partition restructure).
- Schema evolution requiring re-projection of historical Object Type instances.
- Catastrophic ClickHouse failure with PITR loss.

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Quarantine the affected ClickHouse table: `ALTER TABLE <table> DETACH PARTITION <range>` | ≤ 5 min |
| 2 | Snapshot Kafka outbox topic offsets at start of backfill window | ≤ 1 min |
| 3 | Read raw events from Kafka outbox (per-tenant `ontology.events.object-instance-mutated.v1`) | hours |
| 4 | Re-project each event through the current Function projection logic; emit rows into ClickHouse staging table | continuous |
| 5 | Validate: count(rows in ClickHouse staging) == count(events in Kafka outbox window); per-tenant integrity checks pass | ≤ 30 min |
| 6 | Promote staging table to production via atomic `RENAME TABLE` | ≤ 1 min |
| 7 | Validate per-tenant row counts match Postgres source-of-truth (RLS-scoped sampling) | ≤ 30 min |
| 8 | Resume mirror-consumer to live tail | ≤ 5 min |

### Validation

- Per-tenant random sample of 100 Object Type instances; compare ClickHouse mirror vs Postgres canonical; assert 100% match.
- ClickHouse `system.parts` shows expected partition layout.
- Mirror lag returns to ≤ 60 s baseline.

## Schema Migration Backfill

### Trigger

Schema evolution touches existing data (property added with default value; property tier change; struct property nested-schema change).

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | New schema deployed in shadow mode (writes get new schema; reads honour both old + new) | ≤ 10 min (per `runbooks/type-registry-migration.md`) |
| 2 | Migration job claims per-Citus-shard work: `oya-ontology-sdk backfill --type <name> --from-schema <prev> --to-schema <new>` | hours |
| 3 | Per-shard backfill: stream Object Type rows; rewrite under new schema_version; UPDATE in-place or write to new partition (depending on property tier change) | continuous |
| 4 | Per-row idempotency: row tagged with `_backfill_marker = <new_schema_version>`; re-run safe | continuous |
| 5 | Throttle: max 1000 rows/sec/shard to avoid Postgres I/O saturation | – |
| 6 | Validation: `SELECT count(*) FROM <table> WHERE schema_version = <new>` matches expected per tenant | ≤ 30 min |
| 7 | Emit `SchemaMigrationCompleted` event | automatic |

### Tier-change backfill

When a property tier is loosened (Tier1 → Tier2 — requires 2-person rule per `runbooks/type-registry-migration.md`):
1. Pre-flight: confirm DPO sign-off + 2-person approval recorded.
2. Backfill rewrites the property tier label on every existing row.
3. Cedar fragment updated to permit access at the new tier.
4. Audit-chain emit `PropertyTierChanged{property_id, prev_tier, new_tier, approved_by[2]}`.

## Audit-Chain Re-Seal from Outbox

### Trigger

Merkle root tamper detected (FM-13); seal disputed by tenant; seal lost in audit-chain catastrophe.

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Quarantine the affected (tenant, period) seal: trust-state set to `unverifiable` | ≤ 5 min |
| 2 | Engage ops-security + audit-chain µservice owner | ≤ 10 min |
| 3 | Read raw events from Kafka outbox for the (tenant, period) range | ≤ 1 h |
| 4 | Rebuild Merkle tree deterministically: same hash function + same ordering | ≤ 30 min |
| 5 | Sign new Merkle root with current Ed25519 key (current key, not rotated key from the period — note this in the audit record) | ≤ 1 min |
| 6 | Compare new root vs original sealed root: if match, original was authentic (e.g., reading bug); if mismatch, original was tampered | – |
| 7 | Emit `AuditChainResealed{tenant, period, original_root, new_root, match, reason, executed_at}` | automatic |
| 8 | Tenant + regulator notification: provenance claim for the affected window flagged with re-seal annotation | per legal SLA |

### Trust-state ladder

| State | Meaning |
|---|---|
| `verifiable` | Original seal verifies against Merkle tree from outbox; full trust |
| `unverifiable` | Mismatch detected; investigation pending |
| `re-sealed-authentic` | Re-seal matches original; trust restored (was reading bug, not tamper) |
| `re-sealed-tampered` | Re-seal disagrees with original; original sealing process was compromised; period flagged forever |
| `permanently-unverifiable` | Re-seal impossible (outbox itself corrupted); period flagged forever |

## DSR Cascade Replay

### Trigger

Subject (data subject) requests erasure per GDPR Art. 17 / KR PIPA Art. 36 / DPDPA §12 / LGPD Art. 18.

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Tenant raises DSR request: `oya-ontology-sdk dsr-request --tenant <id> --subject-hash <hash>` | ≤ 1 min |
| 2 | DSR cascade runner enumerates every Object Type table for the tenant; scans for the subject hash in declared subject-identifier properties | ≤ 1 h |
| 3 | Per matched row: soft-delete (Postgres `tombstoned_at = now()`); ClickHouse history-mirror row also tombstoned | continuous |
| 4 | Link Types referencing tombstoned Object instances: tombstone in the same transaction | continuous |
| 5 | Multi-hop scan (depth ≤ 5): identify Object Types that reference the subject indirectly via Link Type chains; cascade tombstone | ≤ 1 d |
| 6 | Per-Object-Type completeness manifest: hash of removed-row-ids; signed by DSR runner SPIFFE identity | ≤ 5 min |
| 7 | Emit `DsrErasureExecuted` event with manifest hash | automatic |
| 8 | After 30-day grace: physical delete of soft-deleted rows | day 30 |
| 9 | Tenant receives DSR completion notification with manifest hash | ≤ 30 d (SLA) |

### Limitations

- Audit-chain seals immutable; the seal record itself remains for non-repudiation; the subject identifier is removed from mutable Object Types only.
- Data already deleted by retention before DSR request is implicitly honoured.
- Cross-pillar grants used by the subject's principal are revoked at DSR time.

## PITR Restore

### Trigger

Accidental delete; corruption; ransomware (hypothetical).

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Identify scope: which (tenant, Object Type, time-range) affected | ≤ 30 min |
| 2 | Provision restore Postgres instance from PITR archive at desired point-in-time | ≤ 2 h |
| 3 | Validate restored data: tenant samples + RLS verification | ≤ 1 h |
| 4 | Cut-over: pause writes via Helm-deploy of read-only mode on Layer-B; copy affected rows from restore to primary; resume writes | ≤ 30 min |
| 5 | Audit-chain emit `PitrRestoreExecuted{scope, restore_point, source_pitr_archive, executed_at}` | automatic |
| 6 | Postmortem | ≤ 5 business days |

## Pack-Misroute Cleanup (FM-16)

### Trigger

Rows wrote to wrong pack (cross-border transfer violation).

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Identify misrouted rows: cross-pack scan via central tenant-pack registry | ≤ 30 min |
| 2 | Quarantine: move rows to `_misrouted_<src_pack>_<dst_pack>` table in src pack (still in correct legal jurisdiction) | ≤ 1 h |
| 3 | Notify tenant: rows are in quarantine; request consent to delete OR re-write to correct pack | per tenant DPA |
| 4 | Re-write to correct pack via SDK in the workload µservice | ≤ 1 d |
| 5 | After tenant confirmation: delete quarantine rows | per tenant decision |
| 6 | Audit-chain emit `PackMisrouteRecovered`; breach notification chain per `runbooks/cross-tenant-leak-recovery.md` §"Pack misroute" | ≤ 72h |

## Tenant Data Export (DSR Art. 15)

### Trigger

Subject access request — tenant requests export of their own data on behalf of their end-user.

### Procedure

| Phase | Step | Time |
|---|---|---|
| 1 | Tenant raises export request via `oya-ontology-sdk dsr-export --tenant <id> --subject-hash <hash>` | ≤ 1 min |
| 2 | Read every Object Type row matching the subject hash | ≤ 1 h |
| 3 | Project tier-filtered properties (Tier1Sensitive properties excluded unless tenant has DPA entitlement) | ≤ 30 min |
| 4 | Include provenance: audit-chain seal references for each row | ≤ 30 min |
| 5 | Encrypt the export bundle with tenant-provided public key | ≤ 30 min |
| 6 | Deliver via signed S3 presigned URL valid 7d | ≤ 1 d |
| 7 | Emit `DsrExportExecuted` event | automatic |

## Verification

- `oya gate validate ontology-backfill-conformance` — exit 0.
- Quarterly backfill drill: rebuild ClickHouse mirror from outbox; validate against Postgres source-of-truth.
- Annual DSR cascade drill: synthetic subject; full cascade; per-Object-Type completeness manifest validates.

## References

- `microservices/ontology/runbooks/type-registry-migration.md` (schema migration backfill).
- `microservices/ontology/runbooks/clickhouse-rebalance.md` (mirror rebuild).
- `microservices/ontology/runbooks/cross-tenant-leak-recovery.md` (audit chain re-seal; pack misroute).
- `microservices/ontology/runbooks/object-type-deprecation.md` (deprecation backfill).
- ADR-0028 (audit-chain).
- ADR-0050 (Bominal — outbox pattern; inherited).
- GDPR Art. 15 + 17; KR PIPA Art. 36; DPDPA 2023 §12; LGPD Art. 18.
- Postgres PITR docs — `postgresql.org/docs/16/continuous-archiving.html`.
- ClickHouse — `clickhouse.com/docs/en/operations/backup`.
- Kafka consumer offset replay — `kafka.apache.org/documentation/`.
