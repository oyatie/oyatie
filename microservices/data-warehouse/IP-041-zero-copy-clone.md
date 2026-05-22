---
ip_id: IP-041
microservice: data-warehouse
title: Zero-copy clone
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-02
binding_adrs: [ADR-0131, ADR-0244, ADR-0329]
counterpart_parity: Snowflake zero-copy clone
capabilities_touched: [zero-copy-clone-create]
billing_components: [storage_bytes]
---

# IP-041 — Zero-copy clone

## §1 Objective

Land Snowflake-class zero-copy clone for tables, schemas, and databases.
Clone is a metadata-only operation: the new object points to the existing
data files. Subsequent writes to either side accrue `storage_bytes` only
for the divergent files.

Closes F-D4-D-02 ("Zero-copy clone — Missing primitive").

## §2 Scope

In scope:

- Table clone (`CREATE TABLE … LIKE … CLONE`).
- Schema clone.
- Database clone (the whole catalog node).
- Cross-`tenant_id` clone refusal (Cedar).
- Time-travel clone (clone AT a past timestamp).

Out of scope:

- Cross-region clone (the source data lives in the source region;
  cross-region replication separate).

## §3 Architecture

### §3.1 Metadata layer

A clone creates a new catalog object with a pointer to the source's data
files. The clone's `_delta_log/_iceberg_metadata/_hudi_timeline` starts at
the clone-point as a *snapshot* of the source.

### §3.2 Divergence

Writes to the clone create new files under the clone's path; writes to
the source create new files under the source's path. The two diverge
independently; existing shared files remain pointed to from both sides
until both sides VACUUM past them.

### §3.3 Cedar gate

`local-zero-copy-clone-scope.cedar` (new) — refuses cross-`tenant_id`
clone; allows within-tenant; allows clone to a different `pack_overlay`
only if the resulting clone is in a region compatible with the source's
residency.

## §4 Billing accrual

- Clone create: zero `storage_bytes` (metadata only).
- Subsequent divergent writes: standard `storage_bytes` accrual.
- VACUUM on either side: only reclaims files no longer referenced by
  *either* side.

## §5 SLO bindings

- `slos/zero-copy-clone-latency.openslo.yaml` — p99 clone ≤ 1 s for a
  ≤ 1 TiB table.

## §6 Failure modes

- Cross-tenant clone → refused with `cross_tenant_clone_forbidden`.
- Clone of a non-existent source → `source_table_not_found`.
- Clone with `time_travel` past the window → `time_travel_window_exceeded`.

## §7 Acceptance criteria

- A 1 TiB table clones in ≤ 1 s.
- A subsequent write on the clone creates a new file; the source's
  file count is unchanged.
- A cross-tenant clone is refused.
- A clone with `AT(TIMESTAMP=…)` within window succeeds; past window is
  refused.

## §8 Risks

- Long-running shared files keep the storage usage high until both sides
  VACUUM; tenants must understand the accrual model.

End of IP-041.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-041-zero-copy-clone.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
