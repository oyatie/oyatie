---
ip_id: IP-036
microservice: data-warehouse
title: Delta Live Tables declarative ETL
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-05
binding_adrs: [ADR-0131, ADR-0145, ADR-0254, ADR-0329]
counterpart_parity: Databricks Delta Live Tables + Snowflake Dynamic Tables
capabilities_touched: [dlt-pipeline-declare]
billing_components: [compute_credits]
---

# IP-036 — Delta Live Tables (DLT) declarative ETL

## §1 Objective

Land DLT-class declarative ETL on top of the lake-table substrate. Tenants
declare a pipeline as a DAG of named transforms with expectations
(`expect`, `expect_or_drop`, `expect_or_fail`); the runtime materializes
retired-standard and canonical tables incrementally from retired-basic sources and emits
lineage and quality events. Snowflake's Dynamic Tables sit in the same
slot — declarative materialization with a target lag.

Closes F-D4-L-05 ("Delta Live Tables / declarative ETL — Not authored").

## §2 Scope

In scope:

- The pipeline declaration grammar (YAML with embedded SQL).
- The `expect` family of quality assertions (drop / fail / quarantine).
- Incremental materialization (CDF-driven) of retired-standard and canonical tables.
- Per-pipeline target lag (Snowflake-style) — the pipeline refresh
  cadence is dictated by the lag budget, not a fixed schedule.
- Lineage emission to `ontology` µservice per refresh.
- Per-row quality quarantine to a side table when `expect_or_drop` fires.

Out of scope:

- The notebook UI for authoring DLT pipelines (front-end shell µservice).
- Cross-pipeline orchestration (DAGs across pipelines — workflow-engine
  µservice).

## §3 Architecture

### §3.1 Pipeline declaration

```yaml
pipeline:
  id: orders-retired-basic-retired-standard-canonical
  tenant_id: <uuid>
  target_lag: 10m
  edges:
    - name: orders_retired-standard
      source: catalog.retired-basic.orders
      sql: |
        SELECT order_id, customer_id, total_cents
        FROM STREAM(catalog.retired-basic.orders)
      expect:
        - rule: customer_id IS NOT NULL
          policy: drop
        - rule: total_cents >= 0
          policy: fail
    - name: orders_canonical
      source: catalog.retired-standard.orders_retired-standard
      sql: |
        SELECT customer_id, COUNT(*) AS order_count
        FROM catalog.retired-standard.orders_retired-standard
        GROUP BY customer_id
```

### §3.2 Runtime

- Each refresh round reads the change-data feed from the retired-basic source
  (IP-037).
- Transforms run on a per-pipeline Cloud Hypervisor pod set (ADR-0254).
- Output tables are Delta by default; tenant can override per edge.
- Lineage edges emit to `ontology` µservice via direct gRPC (ADR-0145).

### §3.3 Expectations

- `drop` — drop the violating row, log to the pipeline metrics.
- `fail` — abort the refresh; emit `dlt.refresh.failed`.
- `quarantine` — write the violating row to a side table for review.

### §3.4 Target lag

The pipeline scheduler refreshes the slowest stage whenever its observed
lag exceeds the budget. Resource contention (other tenants) does not lift
the lag bound — admission control refuses scheduling if the cluster is
saturated; the operator is alerted; refresh queues.

## §4 Cedar binding

DLT pipeline creation requires `paid` tenant_class +
`compute_credits` enabled. The Cedar `local-warehouse-query-access.cedar`
gates the underlying SELECTs.

## §5 Billing accrual

- Each refresh accrues `compute_credits` for the transform work.
- The CDF subscription accrues `streaming_ingest_events`.
- Quarantine side-table writes accrue `storage_bytes`.

## §6 SLO bindings

- `slos/dlt-pipeline-freshness.openslo.yaml` — observed lag ≤ target lag
  in 99 % of refresh windows.

## §7 Failure modes

- Refresh exceeds budget → operator alerted via SLO breach.
- Expect_or_fail fires → refresh aborted; previous canonical version retained.
- Pod eviction mid-refresh → resume from last CDF offset.
- Cyclic DAG declared → refused at declare time with `dlt_cyclic_dag`.

## §8 Acceptance criteria

- A 3-stage pipeline (retired-basic → retired-standard → canonical) refreshes within target
  lag for 99 % of windows in a 24-h test.
- `expect_or_drop` produces a metrics row but does not fail refresh.
- `expect_or_fail` aborts refresh and emits `dlt.refresh.failed`.
- `expect_or_quarantine` writes to side table.
- Lineage edges land in `ontology` µservice within 60 s of refresh end.

## §9 Risks

- Long-running pipelines under target-lag pressure can starve other
  tenants; mitigated by admission control on cluster-level
  `compute_credits`.

End of IP-036.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-036-delta-live-tables-declarative.md` matched `SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-036-delta-live-tables-declarative.md` matched `emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
