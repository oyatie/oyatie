---
ip_id: IP-032
microservice: data-warehouse
title: Apache Iceberg write substrate
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-01
binding_adrs: [ADR-0105, ADR-0131, ADR-0145, ADR-0244, ADR-0251, ADR-0329]
counterpart_parity: Apache Iceberg (Snowflake + BigQuery + Databricks support)
capabilities_touched: [iceberg-metadata-register, lake-table-write]
billing_components: [storage_bytes, compute_credits]
---

# IP-032 — Apache Iceberg write substrate

## §1 Objective

Land an Iceberg-class write substrate inside `data-warehouse` so that tenants
can author Iceberg tables on oyatie's tenant-scoped object storage with the
Iceberg metadata pointer model, snapshot lifecycle, partition evolution, and
schema evolution. Iceberg is the cross-vendor open-table-format anchor
(BigQuery BigLake, Snowflake Iceberg tables, Databricks UniForm); Wave-15A
treats it as the *primary* lakehouse format and Delta as the second.

## §2 Scope

In scope:

- Iceberg v2 metadata writer in `lake-engine` layer.
- The `metadata/` JSON file family + the `manifest-list/` Avro files +
  the per-snapshot `manifest/` Avro files in tenant-home object storage.
- Atomic snapshot commit via the catalog (Unity-Catalog-class, IP-034).
- Partition evolution (Iceberg's hidden partitioning model).
- Schema evolution (Iceberg's per-column ID model).
- Time-travel via snapshot ID or timestamp.
- Cedar binding: `local-time-travel-scope.cedar`,
  `local-schema-change-approval.cedar`.

Out of scope:

- Delta interop (IP-031).
- Hudi interop (IP-033).
- Federated query against external Iceberg tables (IP-039).

## §3 Architecture

### §3.1 Catalog binding

Iceberg's atomic commit depends on a catalog that supports
compare-and-swap on the current metadata location. Oyatie's Unity-Catalog-class
namespace (IP-034) provides this via a row in `iceberg_metadata_pointer`
keyed by `(tenant_id, catalog, schema, table)`.

### §3.2 Storage layout

```
s3://oyatie-<tenant_uuid>-warehouse/<catalog>/<schema>/<table>/
  metadata/
    v1.metadata.json         # initial metadata
    v2.metadata.json         # after first commit
    ...
    snap-<snapshot_id>-1-<uuid>.avro    # manifest list per snapshot
  data/
    <partition>/part-...-<uuid>.parquet
```

### §3.3 Commit protocol

1. Writer reads current `iceberg_metadata_pointer` row (= current metadata URL).
2. Writer stages new Parquet files in `data/`.
3. Writer authors new `manifest/` Avro files.
4. Writer authors a new `metadata/vN.metadata.json` with the new snapshot.
5. Writer issues a compare-and-swap on `iceberg_metadata_pointer` from
   vN-1 → vN.
6. On collision, writer rebases against the new pointer and retries.

### §3.4 Snapshot expiration

A tenant's `time_travel_storage_days` purchases dictate snapshot retention:

- 0 days (`demo_trial`): keep only the current snapshot; expire others
  immediately.
- 7..90 days (`paid`): retain snapshots within the purchased window;
  expire older snapshots asynchronously.

Snapshot expiration is a background `worker`-layer job; it reclaims
`storage_bytes` (refund).

## §4 Cedar binding

The Iceberg `evolveSchema` semantics differ from Delta (Iceberg uses
column IDs that survive rename); the Cedar two-person rule applies to
*destructive* changes (drop column, type-narrow). Additive changes
(add column, widen type, rename via column ID) pass.

## §5 Billing accrual

- Per commit: O(commit_size) `storage_bytes` for data + metadata files.
- Snapshot expire: refunds `storage_bytes` past
  `time_travel_storage_days`.
- Federated scan from external compute engine (BigLake, Photon, Snowflake
  external): no `compute_credits` charged because the compute happens
  outside data-warehouse; `federated_query_bytes` accrues on the
  *consuming* tenant (handled by IP-039).

## §6 SLO bindings

- `slos/iceberg-snapshot-commit-latency.openslo.yaml` — p99 ≤ 3 s for
  ≤ 100 MiB commits.

## §7 Failure modes

- Catalog compare-and-swap collision after 5 retries → `iceberg_commit_contention`.
- Stale metadata pointer (catalog returned stale row) → re-read,
  rebase, retry; emit warning metric.
- Partition spec change mid-write → refused with
  `iceberg_partition_spec_drift`; writer must restart with the new spec.

## §8 Migration / interop

- A Delta UniForm table is registrable here via `iceberg-metadata-register`.
- BigQuery's BigLake Iceberg tables can register via `external-table-scan`
  (IP-039); they are read-only externally.

## §9 Acceptance criteria

- A `lake.write` to a new Iceberg table on a `paid` tenant lands a
  `v1.metadata.json` and a snapshot manifest.
- A concurrent commit triggers compare-and-swap retry.
- Time-travel `SELECT * FROM iceberg_table AT(SNAPSHOT_ID => …)` resolves
  in p99 ≤ 1 s.
- Snapshot expiration past the 14-day window reclaims storage_bytes within
  24 h.
- `demo_trial` tenant snapshot retention is exactly 1 (the current snapshot).

## §10 Risks

- Iceberg v3 (with row-lineage) is on the upstream roadmap; this IP targets
  v2; v3 migration is a separate IP.
- Catalog round-trip latency (Unity-Catalog) on commit path; mitigated by
  in-cell catalog cache with TTL ≤ 5 s.

End of IP-032.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-032-apache-iceberg-write-substrate.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
