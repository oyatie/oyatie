---
ip_id: IP-035
microservice: data-warehouse
title: Auto Loader streaming ingest
wave: Wave-15A-DATA-WAREHOUSE-FIX
date: 2026-05-21
owner: solo-owner-data-warehouse
status: drafted
priority: P1
defect_closed: F-D4-L-04
binding_adrs: [ADR-0105, ADR-0131, ADR-0145, ADR-0252, ADR-0329]
counterpart_parity: Databricks Auto Loader + Snowflake Snowpipe + BigQuery Storage Write API
capabilities_touched: [auto-loader-stream-ingest, change-data-feed-subscribe]
billing_components: [streaming_ingest_events, storage_bytes]
---

# IP-035 — Auto Loader streaming ingest

## §1 Objective

Land a high-throughput streaming ingest path that covers the
Auto-Loader-class (Databricks), Snowpipe-class (Snowflake), and
Storage-Write-API-class (BigQuery) union. Tenants land HL7 / FHIR / Kafka /
S3-drop / GCS-drop / OCI-drop events into a Delta or Iceberg retired-basic table
with schema inference, exactly-once delivery, and per-event accrual.

Closes F-D4-L-04 ("Auto Loader / streaming ingest — Not authored").

## §2 Scope

In scope:

- File-notification + directory-listing ingest source (S3 + GCS + OCI
  Object Storage + Azure Blob).
- Kafka + Kinesis + Pub/Sub + OCI Streaming source.
- Schema inference (from first N rows; configurable).
- Schema evolution policy (`failOnNewColumns`, `addNewColumns`,
  `rescue`).
- Exactly-once via offset commit alongside Delta/Iceberg commit.
- Dead-letter queue for parse-failure rows.

Out of scope:

- DLT (declarative ETL on top of streams; IP-036).
- CDF emit (downstream of ingest; IP-037).

## §3 Architecture

### §3.1 Layer placement

- `api`/`rest` → `POST /v1/lake/streams/{id}/start`, `…/stop`,
  `…/status`.
- `application` → `AutoLoaderOrchestrator` (admission +
  `streaming_ingest_events` budget).
- `worker` → `IngestRunner` (long-running per-stream worker on a Cloud
  Hypervisor pod per ADR-0254).
- `lake-engine` → `retired-basicTableWriter` (Delta / Iceberg / Hudi sink).

### §3.2 Offset commit

The ingest commits an offset alongside the lake table commit. Concretely:

- A Delta sink appends a `_offsets/<stream_id>/v<N>.json` file in the same
  commit as the data write; the offset advance is therefore atomic with
  the data write.
- An Iceberg sink stores the offset in a snapshot property
  (`stream.<id>.offset`).
- A Hudi sink stores the offset in `.hoodie/.stream/<id>.offset`.

### §3.3 Exactly-once guarantees

- Sources are pulled in monotonically increasing offsets.
- The sink commit is conditional on the previous offset row.
- On restart, the ingest reads the last committed offset and resumes.
- Duplicate delivery from the source is deduplicated at the ingest layer.

### §3.4 Schema inference

The first 1000 events (configurable) sample to infer a schema. After that:

- `failOnNewColumns` — new column triggers `ingest_schema_drift_refused`.
- `addNewColumns` — new column is appended (additive evolution, no
  approval needed).
- `rescue` — new column is captured under a `_rescued_data` map column.

## §4 Cedar binding

`local-schema-change-approval.cedar` extends to Auto-Loader schema
evolution: additive auto-approves; non-additive is refused by streaming
ingest (the operator must run a separate `dataset.evolveSchema` with
two-person rule).

## §5 Billing accrual

- `streaming_ingest_events` accrues per event ingested.
- `storage_bytes` accrues per byte written.
- A `demo_trial` tenant is capped at 1 M events / day.
- A `paid` tenant pays per event.

## §6 SLO bindings

- `slos/auto-loader-ingest-throughput.openslo.yaml` — p99 sustained
  throughput ≥ 1 M events per second per stream on a `paid` tenant.

## §7 Failure modes

- Source unreachable → backoff + retry; emit `ingest_source_unreachable`.
- Sink commit failure → roll back offset; do not advance; emit
  `ingest_sink_commit_failed`.
- Schema drift in `failOnNewColumns` mode → refuse + alert.
- Cloud Hypervisor pod eviction → IngestRunner restarts on another pod;
  resume from last committed offset.
- Backpressure (sink slower than source) → emit
  `ingest_backpressure_observed` metric; do not drop events.

## §8 Acceptance criteria

- An S3 directory-listing source with 1 M events ingests into a Delta
  retired-basic table in under 1 s.
- A restart resumes from the last committed offset with zero duplicates.
- A schema drift in `addNewColumns` mode lands new column.
- A `demo_trial` tenant exceeding 1 M events / day is refused.

## §9 Risks

- Pod evictions can cause pause windows; mitigated by per-stream
  Kubernetes PriorityClass + PDB.
- Source-side delivery semantics vary; the source adapter library is the
  contract.

End of IP-035.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-035-auto-loader-streaming-ingest.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
