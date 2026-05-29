---
doc_class: IP
ip_id: IP-003-ontology-projection
microservice: data-warehouse
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0246
  - ADR-0253-amendment
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0294
  - ADR-0296
  - ADR-0297
  - ADR-0314
  - ADR-0321
journey_ref: J-DW-003-ontology-projection
capability_profile: Tier-1
status: deepened
date: 2026-05-20
owner_team: data-platform-warehouse
---

# IP-003 Data Warehouse ontology-projection

Service: data-warehouse
ChangeSet scope: microservices/data-warehouse/IP-003-ontology-projection.md
Benchmarks: Snowflake, Databricks, Google BigQuery, AWS Redshift, ClickHouse
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- ontology-projection-objective 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- ontology-projection-objective 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- ontology-projection-objective 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- ontology-projection-objective 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- ontology-projection-objective 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- ontology-projection-objective 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Prerequisites
- ontology-projection-prerequisites 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- ontology-projection-prerequisites 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- ontology-projection-prerequisites 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- ontology-projection-prerequisites 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- ontology-projection-prerequisites 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- ontology-projection-prerequisites 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Implementation steps
- ontology-projection-implementation-steps 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- ontology-projection-implementation-steps 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- ontology-projection-implementation-steps 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- ontology-projection-implementation-steps 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- ontology-projection-implementation-steps 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- ontology-projection-implementation-steps 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Tests and evidence
- ontology-projection-tests-and-evidence 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- ontology-projection-tests-and-evidence 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- ontology-projection-tests-and-evidence 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- ontology-projection-tests-and-evidence 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- ontology-projection-tests-and-evidence 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- ontology-projection-tests-and-evidence 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Rollback
- ontology-projection-rollback 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- ontology-projection-rollback 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- ontology-projection-rollback 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- ontology-projection-rollback 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- ontology-projection-rollback 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- ontology-projection-rollback 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Acceptance criteria
- ontology-projection-acceptance-criteria 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- ontology-projection-acceptance-criteria 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- ontology-projection-acceptance-criteria 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- ontology-projection-acceptance-criteria 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- ontology-projection-acceptance-criteria 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- ontology-projection-acceptance-criteria 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Context
- IP-003 owns the semantic projection between vendor warehouse catalogs and Oyatie ontology objects.
- Snowflake databases, schemas, warehouses, stages, shares, masking policies, and query history become tenant-scoped graph vertices.
- BigQuery projects, datasets, reservations, routines, row access policies, and job statistics become cell-local ontology objects.
- Redshift clusters, namespaces, workgroups, WLM queues, datashares, and Spectrum external schemas become `WarehouseScope` and `WarehouseWorkloadPool` objects.
- Databricks SQL warehouses, Unity Catalog metastores, catalogs, schemas, shares, and notebooks become governed warehouse catalog nodes.
- Synapse Analytics workspaces, SQL pools, linked services, dedicated pools, and serverless endpoints become regional execution cells.
- Firebolt databases, engines, fact tables, aggregating indexes, and accounts become low-latency analytical pools.
- ClickHouse Cloud services, databases, tables, materialized views, dictionaries, and roles become high-throughput projection edges.
- Vertica schemas, projections, resource pools, Eon depots, and external tables become storage/execution split objects.
- Teradata Vantage databases, roles, workload rules, query bands, and TASM states become policy-enforced tenant objects.
- Yellowbrick databases, resource groups, query queues, and storage stripes become capacity and query lineage objects.
- This IP does not copy vendor metadata verbatim; it normalizes identity, policy, lineage, and cost before graph publication.
- The projection runs after tenant scope validation, Cedar authorization, and ADR-0263 audit linkage.
- The projection emits no user-visible object until ontology, audit, and cost attribution writes share one `projection_batch_id`.

## Data Model Deltas
```sql
CREATE TYPE warehouse_vendor AS ENUM (
  'snowflake', 'bigquery', 'redshift', 'databricks_sql', 'synapse_analytics',
  'firebolt', 'clickhouse_cloud', 'vertica', 'teradata_vantage', 'yellowbrick'
);
CREATE TABLE dw_ontology_projection_batch (
  projection_batch_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  source_vendor warehouse_vendor NOT NULL,
  source_account_ref text NOT NULL,
  home_cell text NOT NULL,
  ontology_version text NOT NULL,
  started_at timestamptz NOT NULL,
  completed_at timestamptz,
  audit_id uuid NOT NULL,
  UNIQUE (tenant_id, source_vendor, source_account_ref, ontology_version)
);
CREATE TABLE dw_vendor_object_projection (
  projection_id uuid PRIMARY KEY,
  projection_batch_id uuid NOT NULL REFERENCES dw_ontology_projection_batch(projection_batch_id),
  vendor_object_kind text NOT NULL,
  vendor_object_ref text NOT NULL,
  oyatie_object_kind text NOT NULL,
  oyatie_object_id uuid NOT NULL,
  field_delta jsonb NOT NULL,
  policy_scope_hash text NOT NULL,
  lineage_edge_count integer NOT NULL CHECK (lineage_edge_count >= 0)
);
```
```rust
pub enum WarehouseVendor { Snowflake, BigQuery, Redshift, DatabricksSql, SynapseAnalytics, Firebolt, ClickHouseCloud, Vertica, TeradataVantage, Yellowbrick }
pub struct OntologyProjectionBatch {
    pub projection_batch_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub source_vendor: WarehouseVendor,
    pub source_account_ref: String,
    pub home_cell: String,
    pub ontology_version: String,
    pub audit_id: uuid::Uuid,
}
pub struct VendorObjectProjection {
    pub projection_id: uuid::Uuid,
    pub vendor_object_kind: String,
    pub vendor_object_ref: String,
    pub oyatie_object_kind: String,
    pub oyatie_object_id: uuid::Uuid,
    pub field_delta: serde_json::Value,
    pub policy_scope_hash: String,
    pub lineage_edge_count: u32,
}
```

## API Endpoints
- REST `POST /v1/data-warehouse/ontology/projections` starts a projection batch.
```json
{"tenant_id":"018f-tenant","source_vendor":"snowflake","source_account_ref":"acct/us-east-1/prod","ontology_version":"dw-2026-05-20","dry_run":false}
```
- REST `GET /v1/data-warehouse/ontology/projections/{projection_batch_id}` returns counts by vendor kind and Oyatie kind.
```json
{"projection_batch_id":"018f-batch","status":"completed","objects_projected":2841,"lineage_edges":9882,"audit_id":"018f-audit"}
```
- gRPC `ProjectWarehouseOntology(ProjectWarehouseOntologyRequest) returns (ProjectWarehouseOntologyResponse)`.
```json
{"tenantId":"018f-tenant","vendor":"BIGQUERY","sourceAccountRef":"project:finance-prod","includeLineage":true}
```
- AsyncAPI channel `data-warehouse.ontology.projection.completed.v1` publishes the final projection evidence.
```json
{"tenant_id":"018f-tenant","projection_batch_id":"018f-batch","vendor":"redshift","oyatie_object_count":642,"audit_event_class":"WarehouseOntologyProjectionCompleted"}
```

## Cedar Policy Hooks
- principal: `DataWarehouseOperator::"principal_id"` with `tenant_id`, `roles`, `purpose`, and `break_glass=false`.
- action: `Action::"dataWarehouse::ProjectOntology"`.
- resource: `WarehouseProjectionSource::"tenant_id/source_vendor/source_account_ref"`.
- context: `tenant_id`, `home_cell`, `source_vendor`, `ontology_version`, `audit_event_class`, `request_ip`, `mfa_strength`, `data_class`.
- permit requires tenant match, `DATA_PLATFORM_OPERATOR` audience, no emergency bypass flag, and `context.audit_event_class == "WarehouseOntologyProjectionCompleted"`.
- deny if vendor object count exceeds the approved batch ceiling without FinOps approval.
- deny if `source_vendor` is `synapse_analytics` or `bigquery` and regional pack jurisdiction is absent.
- deny if lineage capture is disabled for `warehouse_query` or `governed_share` classes.

## Ontology Projection
| Vendor object | Oyatie object | Field deltas |
| --- | --- | --- |
| Snowflake `DATABASE.SCHEMA.TABLE` | `WarehouseDataset` | `database` -> `namespace`, `schema` -> `domain`, `retention_time` -> `retention_policy_ref` |
| BigQuery `project.dataset.table` | `WarehouseDataset` | `project_id` -> `external_account_ref`, `dataset_id` -> `namespace`, `labels` -> `tag_set` |
| Redshift `namespace.datashare` | `WarehouseGovernedShare` | `producer_account` -> `source_tenant_ref`, `consumer_namespace` -> `share_grantee_ref` |
| Databricks `catalog.schema.table` | `WarehouseCatalogObject` | `metastore_id` -> `catalog_authority_ref`, `owner` -> `principal_ref` |
| Synapse `workspace.sql_pool` | `WarehouseWorkloadPool` | `workspace` -> `cell_ref`, `pool_sku` -> `capacity_shape` |
| Firebolt `engine` | `WarehouseWorkloadPool` | `engine_name` -> `pool_name`, `warmup_method` -> `resume_policy` |
| ClickHouse Cloud `service.database.table` | `WarehouseDataset` | `service_id` -> `execution_cell`, `engine` -> `storage_layout` |
| Vertica `projection` | `WarehousePhysicalLayout` | `segmentation` -> `partition_strategy`, `buddy_projection` -> `replica_ref` |
| Teradata `database.role.query_band` | `WarehousePolicyScope` | `query_band` -> `policy_context`, `profile` -> `resource_governor` |
| Yellowbrick `resource_group` | `WarehouseWorkloadPool` | `queue_name` -> `admission_queue`, `storage_stripe` -> `placement_hint` |

## Workflow Steps
- node `collect_vendor_catalog`: fetch catalog, role, policy, lineage, and cost metadata under source-specific least privilege.
- branch `vendor_requires_region_normalization`: BigQuery, Synapse Analytics, and Teradata Vantage enrich jurisdiction from regional packs.
- node `normalize_identity`: map vendor users, service accounts, shares, and roles to Oyatie principals and tenant scopes.
- node `project_catalog_objects`: write `WarehouseDataset`, `WarehouseCatalogObject`, and `WarehousePhysicalLayout` vertices.
- node `project_execution_objects`: write workload pools, resource groups, reservations, engines, and queues.
- branch `lineage_gap_detected`: halt publication and emit a failed batch if source job/query history cannot be reconciled.
- node `publish_ontology_edges`: attach lineage, ownership, policy, residency, and cost edges atomically.
- node `seal_audit_evidence`: emit ADR-0263 class and link `audit_id` to all projected rows.

## Audit Events
- `WarehouseOntologyProjectionStarted`: emitted before the first vendor read.
- `WarehouseOntologyProjectionObjectMapped`: emitted for each committed Oyatie object batch.
- `WarehouseOntologyProjectionLineageGap`: emitted when query/job lineage is incomplete.
- `WarehouseOntologyProjectionPolicyDenied`: emitted on Cedar denial before graph mutation.
- `WarehouseOntologyProjectionCompleted`: emitted after graph and audit rows are sealed.
- All events carry `tenant_id`, `principal_id`, `trace_id`, `span_id`, `audit_id`, `source_vendor`, and `projection_batch_id` per ADR-0263.

## SLO Targets
| Metric | Target |
| --- | --- |
| p50 projection API acceptance | 120 ms |
| p95 projection API acceptance | 450 ms |
| p99 projection API acceptance | 900 ms |
| throughput | 750 vendor objects/sec per projection worker |
| availability | 99.95% for projection control plane |

## Failure Modes + Recovery
- Vendor catalog throttling: pause collection, persist cursor, resume with exponential backoff, and emit `WarehouseOntologyProjectionVendorThrottled`.
- Missing policy owner: quarantine only that vendor object, continue dry-run counts, and require tenant admin owner mapping.
- Lineage history truncation: stop publication, request historical export from Snowflake, BigQuery, Redshift, or Databricks SQL, then replay.
- Cross-tenant identifier collision: reject the batch and rotate the source account alias mapping before retry.
- Ontology schema mismatch: keep vendor snapshot in staging, run additive migration, then replay with the same `projection_batch_id`.
- Audit sidecar unavailable: fail closed before graph mutation and retry after ADR-0263 signing health is restored.

## Migration Notes
- Snowflake migrations start with `SHOW GRANTS`, `ACCOUNT_USAGE`, masking policies, shares, and warehouses.
- BigQuery migrations start with Cloud Asset Inventory, `INFORMATION_SCHEMA`, reservations, row policies, and job history exports.
- Redshift migrations start with system views, datashares, WLM queues, namespace metadata, and Spectrum external schemas.
- Databricks SQL migrations start with Unity Catalog, SQL warehouse configs, query history, Delta Sharing, and workspace permissions.
- Synapse Analytics migrations start with workspace ARM inventory, dedicated SQL pool metadata, linked services, and firewall rules.
- Firebolt migrations start with engine definitions, database permissions, aggregating indexes, and account limits.
- ClickHouse Cloud migrations start with service metadata, system tables, dictionaries, grants, and query log export.
- Vertica migrations start with catalog projections, resource pools, Eon depot state, and external table mappings.
- Teradata Vantage migrations start with dbc tables, roles, profiles, TASM rules, and query bands.
- Yellowbrick migrations start with databases, schemas, resource groups, queue rules, and storage layout metadata.

## Cross-Microservice Handoffs
- Tenancy receives projected tenant scope claims and rejects objects without tenant ownership.
- Identity receives principal alias mappings for vendor users, service accounts, and groups.
- Policy-engine receives Cedar resource identifiers for every projected warehouse object.
- Audit-chain receives ADR-0263 event envelopes and Merkle seal references.
- Ontology receives graph vertices, lineage edges, and object field deltas.
- Workflow receives failed-branch remediation tasks for lineage, owner, and residency gaps.
- FinOps receives workload pool, query cost, reservation, and resource group mappings.
- Marketplace receives governed-share object links for DealSet settlement.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-003-ontology-projection.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-003-ontology-projection.md` matched `cost, attribution`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
