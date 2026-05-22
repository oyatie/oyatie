---
ip_id: IP-039
microservice: data-warehouse
title: Federated query (BigLake / Snowflake-external-class)
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-D-08
binding_adrs: [ADR-0131, ADR-0145, ADR-0244, ADR-0329]
counterpart_parity: BigQuery BigLake + Snowflake external tables + Databricks federated
capabilities_touched: [federated-external-table-query]
billing_components: [federated_query_bytes, compute_credits]
---

# IP-039 — Federated query (external tables)

## §1 Objective

Land federated query inside `data-warehouse` so tenants can query Parquet /
ORC / Iceberg / Delta / Hudi files on foreign storage (their own S3, GCS,
OCI, Azure Blob, or another oyatie tenant's published share) without
ingesting the data. BigLake (BigQuery), external tables (Snowflake), and
federated catalogs (Databricks) are the counterparts.

Closes F-D4-D-08 ("Federated query / external tables — Not authored").

## §2 Scope

In scope:

- External-table declaration: `CREATE EXTERNAL TABLE … LOCATION 's3://…'`.
- Format support: Parquet, ORC, Iceberg, Delta, Hudi.
- Per-tenant allow-list of foreign URLs.
- Pushdown predicates and projection.
- Cross-cloud read (BigLake Omni-equivalent).

Out of scope:

- Foreign federated WRITE (read-only externally).
- BigQuery / Snowflake / Redshift JDBC federation; Wave-15B.

## §3 Architecture

### §3.1 External-table declaration

The external table is a metadata-only registration in the catalog. The
declaration carries:

- Format (Parquet / ORC / Iceberg / Delta / Hudi).
- Location URL (e.g. `s3://other-account/...`).
- Optional schema hint (else inferred).
- Credentials (per-tenant CMK / KMS reference).

### §3.2 Cedar gate

`local-federated-query-target.cedar` refuses scans against URLs not on the
tenant's allow-list. The allow-list is maintained per tenant via an
operator-only API.

### §3.3 Query path

The query optimizer pushes filters + projections down to the file scan.
Cross-region scans warn the user about latency (no refusal — user
acknowledges).

### §3.4 Billing

- `federated_query_bytes` accrues per byte scanned from foreign storage.
- `compute_credits` accrues for the query work (CPU + memory).

## §4 Cedar binding

`local-federated-query-target.cedar` (new) — URL allow-list and
`paid` requirement (no federated query on `demo_trial`).

## §5 SLO bindings

- `slos/federated-query-overhead.openslo.yaml` — p99 overhead ≤ 20 % over
  equivalent native scan.

## §6 Failure modes

- Foreign URL unreachable → refused with `external_storage_unreachable`.
- Foreign URL not on allow-list → refused with
  `federated_target_not_allowed`.
- Schema drift on foreign source → query continues with best-effort
  projection; non-projecting columns return NULL.

## §7 Acceptance criteria

- A scan against a 1 GiB foreign Parquet completes within 20 % of native
  scan latency.
- An off-list URL is refused.
- A cross-cloud scan completes (with latency warning).

## §8 Risks

- Egress cost on foreign storage may surprise the tenant; oyatie does not
  re-bill that (it's the tenant's own cloud account).

End of IP-039.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-039-federated-query-biglake-class.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-039-federated-query-biglake-class.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
