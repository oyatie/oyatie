---
ip_id: IP-031
microservice: data-warehouse
title: Delta Lake write substrate
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-01
binding_adrs: [ADR-0105, ADR-0131, ADR-0145, ADR-0244, ADR-0251, ADR-0254, ADR-0329]
counterpart_parity: Databricks Delta Lake
capabilities_touched: [lake-table-write, delta-optimize-zorder, change-data-feed-subscribe]
billing_components: [storage_bytes, compute_credits, streaming_ingest_events]
---

# IP-031 — Delta Lake write substrate

## §1 Objective

Land a Delta-Lake-class ACID write substrate inside `data-warehouse` so that
tenants can author Delta tables on oyatie's tenant-scoped object storage with
ACID transactions, time-travel, schema enforcement and evolution,
DELETE/UPDATE/MERGE, OPTIMIZE/Z-ORDER, vacuum, and change-data feed. This
closes the Wave-4 audit finding F-D4-L-01 ("Open table format read/write
(Iceberg + Delta + Hudi) — Not authored").

## §2 Scope

In scope:

- The Delta protocol writer in the `lake-engine` layer (added per ADR-0105 §
  amendment in the manifest).
- The `delta_log` representation in tenant-home object storage (S3, GCS, OCI
  Object Storage per cloud profile; OpenTofu modules in `iac/`).
- Schema enforcement (additive evolution default; non-additive needs two-person
  approval per `local-schema-change-approval.cedar`).
- MERGE INTO with row-level conflict resolution.
- OPTIMIZE / Z-ORDER (compaction + multidimensional clustering).
- VACUUM bound by tenant's `time_travel_storage_days`.
- Time-travel reads via `dataset.timeTravel` (PRD §D.2).
- Cedar binding: `local-schema-change-approval.cedar`,
  `local-time-travel-scope.cedar`.

Out of scope (in this IP, handled by separate IPs):

- Iceberg interop (IP-032).
- Hudi interop (IP-033).
- Unity-Catalog-class namespace (IP-034).
- Auto Loader streaming ingest (IP-035).
- Delta Live Tables (IP-036).
- Change Data Feed (IP-037 — covered in summary here but full spec is IP-037).

## §3 Architecture

### §3.1 Layer placement

```
api          → REST /v1/lake/tables/{name}/write
rest         → command envelope + tenant scope check
application  → DeltaWriteOrchestrator (idempotency + admission)
usecase      → DeltaWriteUseCase (Cedar PEP + cost preflight)
domain       → DeltaTable aggregate (schema, version, partition)
kernel       → DeltaProtocol invariants (ACID protocol versioning)
adapter      → DeltaCommitAdapter (commit to object storage)
worker       → DeltaWorker (background OPTIMIZE / VACUUM)
governance   → DeltaAuditEmitter (audit chain)
lake-engine  → DeltaWriterCore (the actual protocol)
```

### §3.2 Storage layout

```
s3://oyatie-<tenant_uuid>-warehouse/<catalog>/<schema>/<table>/
  _delta_log/
    00000000000000000000.json
    00000000000000000001.json
    ...
    _last_checkpoint
  data/
    part-00000-<uuid>-c000.snappy.parquet
    ...
```

The bucket is per-tenant and pinned to the tenant's home region per
`pack_overlay`. Cross-tenant access is impossible at the bucket policy level;
Cedar is the *second* gate.

### §3.3 Commit protocol

1. Reader takes a current `_last_checkpoint` pointer.
2. Writer stages new Parquet files under `data/`.
3. Writer attempts atomic put of `00000000000000000NNN.json` (the next log
   number) with putIfAbsent semantics (S3 ConditionalWrites, GCS
   ifGenerationMatch, OCI object versioning).
4. On collision, writer rebases the commit against the latest log and retries
   up to 5 times with exponential backoff.
5. Every 10 commits, a checkpoint Parquet is written and `_last_checkpoint`
   updated.

### §3.4 ACID guarantees

- Atomicity: a commit either lands as a single JSON file in `_delta_log` or
  not at all.
- Consistency: readers see a consistent snapshot at `_last_checkpoint` time.
- Isolation: serializable via the putIfAbsent commit; one writer wins per
  log number.
- Durability: object-storage durability + per-tenant cross-region replication
  via `multi-region.md`.

## §4 Cedar binding

`local-schema-change-approval.cedar` is updated to refuse
`DeltaTable::evolveSchema` with a non-additive change unless two distinct
principals attest. The cedar entity model is:

```cedar
entity DeltaTable in DataWarehouse {
  schema_version: Long,
  protocol_version: Long,
  tenant_id: String,
};

action evolveSchema appliesTo {
  principal: [HumanOperator, ServiceTenantWorkload],
  resource: DeltaTable,
};
```

## §5 Billing accrual

- Every commit: O(commit_size) accrues to `storage_bytes` for the data files
  + a constant `delta_log` overhead (typically < 1 KiB per commit, but
  still accrues).
- OPTIMIZE / Z-ORDER: O(input_bytes) accrues to `compute_credits` for the
  read-and-rewrite work.
- VACUUM: storage_bytes refunded for reclaimed files past
  `time_travel_storage_days`.
- CDF subscribe: `streaming_ingest_events` accrues per emitted CDF row
  (covered by IP-037).

## §6 SLO bindings

- `slos/delta-write-commit-latency.openslo.yaml` — p99 ≤ 2 s for ≤ 100 MiB
  commits.

## §7 Failure modes

- Commit collision after 5 retries → return `delta_commit_contention`,
  emit incident metric, runbook `runbooks/delta-commit-contention.md`.
- Log file corruption → quarantine the log file, fall forward to last
  checkpoint, alert SRE.
- Underlying object storage outage → command refused with
  `lake_storage_unavailable`, queue the write for retry up to 30 s.
- Vacuum past time-travel window → safe by design (vacuum honors window).
- Vacuum within time-travel window → refused with
  `vacuum_inside_retention_window`.

## §8 Migration / interop

- A Delta table can be exported to Iceberg via the `delta-uniform` adapter
  (Databricks UniForm-equivalent; IP-032 reads this back).
- A Hudi table is *not* directly readable as Delta (IP-033 separate).

## §9 Acceptance criteria

- A `lake.write` to a new Delta table on a `paid` tenant completes within
  the SLO and produces a `_delta_log/00000000000000000000.json` entry.
- A second concurrent writer triggers commit collision, retries, and lands
  as `00000000000000000001.json` after backoff.
- An OPTIMIZE/Z-ORDER on a 10 GiB table accrues `compute_credits` ≈
  proportional to bytes read+written and emits OPTIMIZE-success audit row.
- A VACUUM on a table with 14-day time-travel refuses to reclaim files
  newer than 14 days.
- A `demo_trial` tenant's OPTIMIZE attempt is refused with
  `tenant_class_cap_exceeded` (OPTIMIZE is paid-only).

## §10 Risks

- Object-storage conditional-write semantics differ across clouds; the
  Delta protocol library must abstract this in the adapter layer.
- Cross-cloud replication of `_delta_log` requires careful ordering;
  delegated to `multi-region.md` SLA.

End of IP-031.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-031-delta-lake-write-substrate.md` matched `p99, SLO, multi-region`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-031-delta-lake-write-substrate.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
