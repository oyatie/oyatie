---
doc_class: IP
ip_id: IP-016
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-016: Backfill Replay Worker

## Context
- DW16-CTX-01: This IP creates deterministic warehouse backfill replay for imported vendor history and local correction jobs.
- DW16-CTX-02: Snowflake query history exports become replay batches with warehouse, role, database, and query hash evidence.
- DW16-CTX-03: BigQuery job history exports become replay batches with project, dataset, slot, and reservation evidence.
- DW16-CTX-04: Redshift STL/SVL history exports become replay batches with cluster namespace and WLM queue evidence.
- DW16-CTX-05: Databricks SQL query history becomes replay batches with warehouse id and statement id evidence.
- DW16-CTX-06: Synapse Analytics request history becomes replay batches with pool and workspace evidence.
- DW16-CTX-07: Firebolt query history becomes replay batches with engine id and database evidence.
- DW16-CTX-08: ClickHouse Cloud query_log exports become replay batches with service and database evidence.
- DW16-CTX-09: Vertica query_requests exports become replay batches with depot and resource pool evidence.
- DW16-CTX-10: Teradata DBQL exports become replay batches with workload and account evidence.
- DW16-CTX-11: Yellowbrick query history becomes replay batches with cluster and resource group evidence.
- DW16-CTX-12: Replay never executes vendor SQL; it reconstructs catalog, cost, lineage, and audit state.
- DW16-CTX-13: Replay is idempotent by source checksum, batch ordinal, and ontology target object id.
- DW16-CTX-14: Replay output is consumed by cost budgets, settlement, SLO gates, and audit closeout.
- DW16-CTX-15: Replay can run in shadow mode before it is promoted to authoritative historical state.

## Data Model Deltas
- DW16-DDL-01: Add replay batch table.
```sql
CREATE TABLE warehouse_backfill_replay_batches (
    batch_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    vendor_source TEXT NOT NULL CHECK (vendor_source IN ('snowflake','bigquery','redshift','databricks_sql','synapse_analytics','firebolt','clickhouse_cloud','vertica','teradata_vantage','yellowbrick','oyatie_native')),
    source_export_ref TEXT NOT NULL,
    source_checksum BYTEA NOT NULL,
    replay_mode TEXT NOT NULL CHECK (replay_mode IN ('shadow','authoritative','correction')),
    high_watermark TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('queued','running','paused','completed','failed','quarantined')),
    row_count BIGINT NOT NULL CHECK (row_count >= 0),
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, vendor_source, source_checksum)
);
CREATE INDEX wh_backfill_batches_status_idx ON warehouse_backfill_replay_batches(status, high_watermark);
```
- DW16-DDL-02: Add replay item table.
```sql
CREATE TABLE warehouse_backfill_replay_items (
    item_id UUID PRIMARY KEY,
    batch_id UUID NOT NULL REFERENCES warehouse_backfill_replay_batches(batch_id) ON DELETE CASCADE,
    source_row_number BIGINT NOT NULL,
    vendor_object_ref TEXT NOT NULL,
    target_object_kind TEXT NOT NULL,
    target_object_id UUID,
    replay_outcome TEXT NOT NULL CHECK (replay_outcome IN ('pending','applied','skipped_duplicate','quarantined','failed')),
    failure_code TEXT,
    lineage_hash BYTEA NOT NULL,
    applied_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX wh_backfill_item_once_idx ON warehouse_backfill_replay_items(batch_id, source_row_number);
```
- DW16-RUST-01: Replay batch type.
```rust
pub struct WarehouseBackfillReplayBatch {
    pub batch_id: ReplayBatchId,
    pub tenant_id: TenantId,
    pub vendor_source: WarehouseVendorSource,
    pub source_export_ref: SourceExportRef,
    pub source_checksum: Sha256Digest,
    pub replay_mode: ReplayMode,
    pub high_watermark: DateTime<Utc>,
    pub status: ReplayBatchStatus,
    pub row_count: u64,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW16-RUST-02: Replay item type.
```rust
pub struct WarehouseBackfillReplayItem {
    pub item_id: ReplayItemId,
    pub batch_id: ReplayBatchId,
    pub source_row_number: u64,
    pub vendor_object_ref: VendorObjectRef,
    pub target_object_kind: WarehouseObjectKind,
    pub target_object_id: Option<WarehouseObjectId>,
    pub replay_outcome: ReplayOutcome,
    pub failure_code: Option<ReplayFailureCode>,
    pub lineage_hash: LineageHash,
    pub applied_at: Option<DateTime<Utc>>,
}
```
- DW16-RUST-03: `ReplayMode::Shadow` writes evidence but does not mutate authoritative projections.
- DW16-RUST-04: `ReplayOutcome::SkippedDuplicate` must include the previously applied target object id.
- DW16-RUST-05: Batch status transitions are validated by a pure state machine before persistence.

## API Endpoints
- DW16-API-01: REST enqueue endpoint.
```http
POST /v1/data-warehouse/backfill/replay-batches
Idempotency-Key: wh-backfill-016
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","vendor_source":"snowflake","source_export_ref":"s3://migration/snowflake/query-history-2026-05.csv","source_checksum":"sha256:9d0f","replay_mode":"shadow","high_watermark":"2026-05-20T00:00:00Z"}
```
- DW16-API-02: REST promote endpoint.
```http
POST /v1/data-warehouse/backfill/replay-batches/{batch_id}:promote
Content-Type: application/json

{"expected_shadow_applied":128844,"expected_quarantined":17,"approval_workflow_id":"01JWF016APPROVE"}
```
- DW16-API-03: gRPC worker lease.
```proto
rpc LeaseWarehouseBackfillReplay(LeaseWarehouseBackfillReplayRequest) returns (LeaseWarehouseBackfillReplayResponse);
message LeaseWarehouseBackfillReplayRequest {
  string tenant_id = 1;
  string worker_id = 2;
  int32 max_items = 3;
  repeated string allowed_vendor_sources = 4;
}
```
- DW16-API-04: AsyncAPI event.
```yaml
warehouse.backfill.replay.batch.completed.v1:
  payload:
    batch_id: 01JWH16BATCH
    vendor_source: snowflake
    applied_count: 128844
    quarantined_count: 17
    audit_event_class: WarehouseBackfillReplayCompleted
```
- DW16-API-05: REST errors use `422 replay_source_checksum_mismatch` for changed export objects.
- DW16-API-06: gRPC lease returns empty assignment when residency overlay denies worker cell.
- DW16-API-07: Async item events are sampled; batch events are mandatory.

## Cedar Policy Hooks
- DW16-CEDAR-01: principal = `Oyatie::Principal::"data_platform_worker:{worker_id}"`.
- DW16-CEDAR-02: action = `Oyatie::Action::"warehouse_backfill_replay_apply"`.
- DW16-CEDAR-03: resource = `Oyatie::WarehouseReplayBatch::"{batch_id}"`.
- DW16-CEDAR-04: context.tenant_id must equal resource tenant id.
- DW16-CEDAR-05: context.vendor_source must be in tenant approved migration vendors.
- DW16-CEDAR-06: context.replay_mode must be `shadow` unless workflow approval exists.
- DW16-CEDAR-07: context.source_checksum_verified must be true.
- DW16-CEDAR-08: context.residency_decision must be `allow`.
- DW16-CEDAR-09: context.audit_event_class must equal `WarehouseBackfillReplayItemApplied` for item mutations.
- DW16-CEDAR-10: deny if worker cell is outside overlay compute region.

## Ontology Projection
- DW16-ONTO-01: Snowflake `QUERY_HISTORY.QUERY_ID` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-02: BigQuery `Job.jobReference.jobId` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-03: Redshift `stl_query.query` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-04: Databricks SQL `statement_id` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-05: Synapse `request_id` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-06: Firebolt `query_id` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-07: ClickHouse Cloud `query_log.query_id` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-08: Vertica `transaction_id.statement_id` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-09: Teradata DBQL `QueryID` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-10: Yellowbrick `query_id` -> `WarehouseQuery.vendor_statement_id`.
- DW16-ONTO-11: Vendor user name -> `WarehousePrincipalAlias.vendor_subject_ref`.
- DW16-ONTO-12: Vendor bytes scanned -> `WarehouseCostUsage.scanned_bytes`.

## Workflow Steps
- DW16-WF-01: Node `ValidateExport` checks checksum, schema version, and row count.
- DW16-WF-02: Node `CreateBatch` inserts queued batch.
- DW16-WF-03: Node `EvaluateResidency` confirms worker cell eligibility.
- DW16-WF-04: Branch `ResidencyDenied` pauses batch and emits denial event.
- DW16-WF-05: Node `LeaseItems` assigns pending rows to a worker.
- DW16-WF-06: Node `ProjectRow` maps vendor row to ontology target object.
- DW16-WF-07: Branch `DuplicateLineage` marks item skipped duplicate.
- DW16-WF-08: Branch `ProjectionFailed` quarantines row with failure code.
- DW16-WF-09: Node `ApplyProjection` writes shadow or authoritative object.
- DW16-WF-10: Node `CheckpointBatch` updates applied and quarantined counts.
- DW16-WF-11: Node `PromoteShadow` replays shadow outcomes as authoritative after approval.
- DW16-WF-12: Node `CompleteBatch` seals final audit event.

## Audit Events
- DW16-AUDIT-01: `WarehouseBackfillReplayBatchQueued` records source checksum and row count.
- DW16-AUDIT-02: `WarehouseBackfillReplayWorkerLeased` records worker id and item count.
- DW16-AUDIT-03: `WarehouseBackfillReplayItemApplied` records target object id and lineage hash.
- DW16-AUDIT-04: `WarehouseBackfillReplayItemQuarantined` records vendor object ref and failure code.
- DW16-AUDIT-05: `WarehouseBackfillReplayDuplicateSkipped` records prior target object id.
- DW16-AUDIT-06: `WarehouseBackfillReplayPromoted` records approval workflow id.
- DW16-AUDIT-07: `WarehouseBackfillReplayCompleted` records final counts and mode.

## SLO Targets
- DW16-SLO-01: p50 item projection <= 12 ms.
- DW16-SLO-02: p95 item projection <= 45 ms.
- DW16-SLO-03: p99 item projection <= 120 ms.
- DW16-SLO-04: throughput >= 12,000 replay items per minute per worker pool.
- DW16-SLO-05: availability >= 99.9 percent for replay lease API.
- DW16-SLO-06: checkpoint lag p95 <= 10 seconds.
- DW16-SLO-07: duplicate detection false negative rate must be 0.
- DW16-SLO-08: quarantine visibility lag p95 <= 30 seconds.

## Failure Modes + Recovery
- DW16-FAIL-01: Source export checksum changes; reject enqueue, preserve previous checksum, and request re-export.
- DW16-FAIL-02: Worker crashes mid-lease; lease expires and pending items are reassigned idempotently.
- DW16-FAIL-03: Ontology projection fails for unknown vendor field; quarantine row and continue batch.
- DW16-FAIL-04: Residency denies worker cell; pause batch until a compliant worker pool is available.
- DW16-FAIL-05: Shadow promotion counts differ; block promotion and emit comparison report.
- DW16-FAIL-06: Audit outbox stalls; stop new leases and drain persisted outbox before resuming.

## Migration Notes
- DW16-MIG-01: Snowflake `QUERY_HISTORY` exports must include warehouse, role, and database columns.
- DW16-MIG-02: BigQuery job exports must include reservation and billing project fields.
- DW16-MIG-03: Redshift history exports must include WLM queue and user id.
- DW16-MIG-04: Databricks SQL history must include warehouse id and workspace id.
- DW16-MIG-05: Synapse Analytics requests must include pool name and request label.
- DW16-MIG-06: Firebolt query exports must include engine id and database name.
- DW16-MIG-07: ClickHouse Cloud `query_log` exports must include normalized user and query_id.
- DW16-MIG-08: Vertica query_requests exports must include depot and resource pool.
- DW16-MIG-09: Teradata Vantage DBQL exports must include account string and QueryID.
- DW16-MIG-10: Yellowbrick history exports must include cluster id and resource group.

## Cross-Microservice Handoffs
- DW16-HANDOFF-01: Data-pipeline receives source export validation and quarantine feedback.
- DW16-HANDOFF-02: Ontology receives projected query, cost, and lineage objects.
- DW16-HANDOFF-03: Cost-budget receives historical usage rows after authoritative replay.
- DW16-HANDOFF-04: Marketplace settlement receives replayed share usage.
- DW16-HANDOFF-05: Audit-chain receives batch and item ADR-0263 events.
- DW16-HANDOFF-06: Workflow receives promotion approvals and failure branches.
- DW16-HANDOFF-07: Policy receives replay mode decisions and worker eligibility context.
- DW16-HANDOFF-08: Catalog receives backfilled dataset lineage links.
- DW16-HANDOFF-09: SLO gate receives replay completeness metrics.
- DW16-HANDOFF-10: Tenant-admin receives migration progress and quarantine summaries.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-016-backfill-replay-worker.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-016-backfill-replay-worker.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
