---
adr_id: ADR-MS-006
microservice: data-warehouse
title: tenant_class billing-component decomposition
date: 2026-05-21
status: accepted
wave: Wave-15A-DATA-WAREHOUSE-FIX
defect_closed: F-D4-C-04
binding_adrs: [ADR-0244, ADR-0329, ADR-0331]
---

# ADR-MS-006 — tenant_class billing-component decomposition

## Context

ADR-0331 retires the retired named capability levels capacity ladder in favor of
`tenant_class ∈ {demo_trial, paid}` with a composable
`paid.billing_components` array. The audit (F-D4-C-01..04) flagged that
data-warehouse had not modeled this at all.

This decision enumerates the eleven billing components that apply to
data-warehouse and binds each one to a capability YAML.

## Decision

The eleven billing components and their per-capability accrual:

| Component | What it counts | Capabilities that accrue against it |
|---|---|---|
| `compute_credits` | T-shirt-size-hour units | warehouse-query-run, delta-optimize-zorder, dlt-pipeline-declare, federated-external-table-query, time-travel-restore, container-udf-execute |
| `storage_bytes` | active + long-term + time-travel + fail-safe storage bytes | lake-table-write, dataset-export, retention-tier-apply, auto-loader-stream-ingest |
| `egress_gb` | cross-region + cross-cloud + share-out bytes | dataset-export, retention-tier-apply (cross-tier moves) |
| `share_consumer_events` | per-consumer-row-read events | governed-share-create, reader-account-share-publish |
| `ml_training_units` | per trainer-reported unit | sql-ml-train |
| `streaming_ingest_events` | per ingested event | auto-loader-stream-ingest, change-data-feed-subscribe |
| `vector_index_serving` | per vector query | vector-index-serve |
| `federated_query_bytes` | bytes scanned from foreign storage | federated-external-table-query, warehouse-query-run (when external scan) |
| `container_udf_seconds` | CPU-second of UDF execution | container-udf-execute |
| `time_travel_storage_days` | retention window days × write volume | time-travel-restore, lake-table-write (when window > 0) |
| `fail_safe_storage_days` | post-time-travel days × write volume | time-travel-restore |

Composability rule: a `paid` tenant may enable any subset. Unused
components accrue zero. A B2C-style hobbyist who never shares pays zero
`share_consumer_events`; a healthcare tenant who pins in-region pays zero
`egress_gb`.

`demo_trial` accrues zero charge on all components and is hard-capped at
the limits in `manifest.json` (50 GiB storage, 100 MB per query, 1 M
ingest events / day, 0 share consumers, 0 ML training, 0 vector serving,
0 federated query, 0 container UDF, 0 time-travel, 0 fail-safe).

## Consequences

- Tenants who upgrade from `demo_trial` to `paid` flip a single boolean;
  no data re-ingest.
- Each capability YAML declares its `billingComponents` field; CI lint
  verifies the union of accrued components matches the manifest.

## Alternatives considered

- Flat per-query / per-byte / per-hour pricing without component
  decomposition: rejected because it doesn't match how Snowflake /
  BigQuery / Databricks bill, and doesn't support the composability
  promise.

End of ADR-MS-006.
