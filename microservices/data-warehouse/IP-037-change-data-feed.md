---
ip_id: IP-037
microservice: data-warehouse
title: Change Data Feed (CDF)
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-06
binding_adrs: [ADR-0131, ADR-0145, ADR-0252, ADR-0329]
counterpart_parity: Databricks Change Data Feed + Snowflake Streams + Iceberg incremental scan
capabilities_touched: [change-data-feed-subscribe, lake-table-write]
billing_components: [streaming_ingest_events]
---

# IP-037 — Change Data Feed (CDF)

## §1 Objective

Land Change-Data-Feed-class emission and subscription on Delta tables (and
the equivalent on Iceberg + Hudi). Downstream µservices and tenants
subscribe to per-row delta events with insert / update / delete operation
markers and snapshot versions. Snowflake Streams sit in the same slot.

Closes F-D4-L-06 ("Change Data Feed — Not authored").

## §2 Scope

In scope:

- Delta CDF (`enableChangeDataFeed = true` at table level).
- Iceberg incremental scan (`startingSnapshot` + `endingSnapshot`).
- Hudi MoR delta-log read as CDF source.
- Subscription model: pull (polling) and push (oyatie message bus).
- Subscriber identity: tenant-owned principal OR signed share-consumer.
- Per-event accrual to `streaming_ingest_events` on the *producer*'s
  budget (Snowflake-shape: producer pays for consumer reads).

Out of scope:

- Cross-cloud CDF replication (delegated to `multi-region.md`).

## §3 Architecture

### §3.1 Delta CDF

When a tenant enables CDF on a Delta table, every commit writes a
companion `_change_data/` Parquet block alongside `data/`. Each row in
`_change_data/` carries:

- `_change_type` ∈ {`insert`, `update_preimage`, `update_postimage`,
  `delete`}.
- `_commit_version` (long).
- `_commit_timestamp` (HLC per ADR-0252).

### §3.2 Iceberg incremental scan

Iceberg lacks a separate CDF construct; equivalent is an incremental scan
across snapshots. The data-warehouse exposes the same subscription API and
internally maps to the Iceberg incremental snapshot scan.

### §3.3 Hudi delta-log as CDF source

Hudi MoR tables already emit delta logs per partition; the CDF subscription
on a Hudi MoR table reads these logs directly with per-event marker
synthesis.

### §3.4 Subscription model

- Pull: `GET /v1/lake/tables/{name}/cdf?from_version=N&max_rows=10000`
  returns a batch.
- Push: subscribe a topic on the oyatie message bus; the warehouse
  publishes per-row events.

### §3.5 Pricing

The *producer* accrues `streaming_ingest_events` per emitted row. The
consumer pays nothing for the read itself (it pays for any downstream
compute). This matches Snowflake's secure-share-consumer billing.

## §4 Cedar binding

`local-cdf-subscriber-scope.cedar` (new) restricts subscribers to:

- The producer tenant's own principals.
- Signed share-consumer accounts (`SHARE_CONSUMER` audience type).

Cross-tenant CDF subscription without a share is refused.

## §5 SLO bindings

- `slos/change-data-feed-lag.openslo.yaml` — p99 emission-to-subscriber
  lag ≤ 5 s.

## §6 Failure modes

- CDF disabled on a table that a subscriber requests → refused with
  `cdf_not_enabled`.
- Subscriber falls behind producer ingest rate → backpressure;
  `cdf_subscriber_lagging` metric; eventually subscriber is dropped if
  lag exceeds 7 days.
- Schema drift mid-stream → emit with the new schema; subscribers must
  handle.

## §7 Acceptance criteria

- A 1 M row insert + 100 k updates + 50 k deletes produces a CDF stream
  with 1.15 M rows in the right operation markers.
- A pull subscriber reads in order.
- A push subscriber sees rows in HLC order.
- `streaming_ingest_events` accrues to producer.
- A non-tenant + non-share subscriber is refused.

## §8 Risks

- CDF doubles the write IO; tenants must opt in per table.
- Long-lived subscribers behind producer rate need cap (7 days).

End of IP-037.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-037-change-data-feed.md` matched `p99, SLO, multi-region`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-037-change-data-feed.md` matched `emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
