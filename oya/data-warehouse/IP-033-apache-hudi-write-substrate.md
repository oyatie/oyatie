---
ip_id: IP-033
microservice: data-warehouse
title: Apache Hudi write substrate
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-01
binding_adrs: [ADR-0105, ADR-0131, ADR-0145, ADR-0244, ADR-0251, ADR-0329]
counterpart_parity: Apache Hudi
capabilities_touched: [lake-table-write]
billing_components: [storage_bytes, compute_credits]
---

# IP-033 — Apache Hudi write substrate

## §1 Objective

Land Apache Hudi as the third open table format inside `data-warehouse`.
Hudi's defining properties — copy-on-write (CoW) tables for read-optimized
workloads and merge-on-read (MoR) tables for streaming-update-heavy workloads
— complement Delta (best for batch ACID + OPTIMIZE) and Iceberg (best for
cross-vendor catalog interop). Together the three formats give a tenant the
choice that Databricks Lakehouse exposes natively.

## §2 Scope

In scope:

- Hudi CoW writer (read-optimized).
- Hudi MoR writer (streaming-friendly with delta logs).
- Hudi clustering (Z-ORDER-class).
- Hudi compaction (MoR delta-log → base file).
- Hudi cleaning (analog of Delta vacuum / Iceberg snapshot expire).
- Hudi timeline-server read path.

Out of scope:

- Hudi-to-Delta cross-format reads (UniForm-style); not in Wave-15A.
- Hudi metadata table on a Foundation DB-class store; this IP uses object
  storage timeline.

## §3 Architecture

### §3.1 Table layouts

```
# Copy-on-write
.hoodie/
  hoodie.properties
  20260521120000.commit            # timeline file
data/
  <partition>/<file_id>_<commit>_<seq>.parquet

# Merge-on-read
.hoodie/
  20260521120000.deltacommit
data/
  <partition>/<file_id>_<commit>_<seq>.parquet    # base file
  <partition>/.<file_id>_<commit>.log.<n>.avro    # delta log
```

### §3.2 Commit protocol

Hudi uses a per-table timeline (an ordered set of action files under
`.hoodie/`). Atomic commit requires a single timeline file to be
authoritatively written. On object storage we use the same conditional-write
primitive as Delta + Iceberg.

### §3.3 Choosing CoW vs MoR

The capability YAML `lake-table-write` accepts a `table_type` parameter:

- `CoW` for read-heavy analytics; commits rewrite base files.
- `MoR` for write-heavy streaming; commits append to delta logs;
  compaction rewrites base files in the background.

Per-tenant default is `CoW` for `demo_trial` (simpler) and writer-chosen
for `paid`.

## §4 Cedar binding

Same as IP-031 / IP-032: `local-schema-change-approval.cedar`,
`local-time-travel-scope.cedar`.

## §5 Billing accrual

- CoW commit: O(rewritten_partition_size) `storage_bytes` + the
  proportional `compute_credits` for the rewrite.
- MoR delta-log append: O(append_size) `storage_bytes`; compaction
  separately accrues `compute_credits`.
- Clustering (Z-ORDER-class): `compute_credits` only.

## §6 SLO bindings

- Hudi commits are graded against the same SLO as Delta
  (`slos/delta-write-commit-latency.openslo.yaml`); Hudi-specific SLO
  optional in Wave-15B.

## §7 Failure modes

- CoW partition rewrite collision → retry with exponential backoff up to
  5 times; refuse with `hudi_commit_contention`.
- MoR delta-log accumulation past compaction threshold → emit
  `hudi_compaction_pressure` warning; auto-trigger compaction.
- Timeline-server unavailable → reader falls back to in-process timeline
  scan; SLA degrades from p99 ≤ 1 s to p99 ≤ 3 s.

## §8 Migration / interop

- Hudi → Delta: not directly readable; tenant must rewrite via
  `data-pipeline` µservice.
- Hudi → Iceberg: same as above.

## §9 Acceptance criteria

- A CoW `lake.write` produces a `.hoodie/<commit>.commit` file and
  partition data files.
- A MoR `lake.write` produces a delta-log file; subsequent compaction
  produces a new base file.
- Clustering on a CoW table reduces query latency on the clustered columns
  by ≥ 30 % vs unclustered baseline.
- Cleaning reclaims storage past `time_travel_storage_days`.

## §10 Risks

- Hudi metadata table on object storage adds round-trips; mitigated by
  in-cell metadata cache.
- Three open formats means the µservice has three protocol surfaces to
  maintain; mitigated by sharing the `lake-engine` layer across all three.

End of IP-033.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-033-apache-hudi-write-substrate.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
