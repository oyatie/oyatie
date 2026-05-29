---
doc_class: IP
ip_id: IP-005-rest-contract-surface
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
journey_ref: J-DW-005-rest-contract-surface
capability_profile: Tier-1
status: deepened
date: 2026-05-20
owner_team: data-platform-warehouse
---

# IP-005 Data Warehouse rest-contract-surface

Service: data-warehouse
ChangeSet scope: microservices/data-warehouse/IP-005-rest-contract-surface.md
Benchmarks: Snowflake, Databricks, Google BigQuery, AWS Redshift, ClickHouse
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- rest-contract-surface-objective 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- rest-contract-surface-objective 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- rest-contract-surface-objective 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- rest-contract-surface-objective 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- rest-contract-surface-objective 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- rest-contract-surface-objective 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Prerequisites
- rest-contract-surface-prerequisites 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- rest-contract-surface-prerequisites 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- rest-contract-surface-prerequisites 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- rest-contract-surface-prerequisites 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- rest-contract-surface-prerequisites 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- rest-contract-surface-prerequisites 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Implementation steps
- rest-contract-surface-implementation-steps 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- rest-contract-surface-implementation-steps 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- rest-contract-surface-implementation-steps 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- rest-contract-surface-implementation-steps 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- rest-contract-surface-implementation-steps 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- rest-contract-surface-implementation-steps 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Tests and evidence
- rest-contract-surface-tests-and-evidence 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- rest-contract-surface-tests-and-evidence 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- rest-contract-surface-tests-and-evidence 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- rest-contract-surface-tests-and-evidence 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- rest-contract-surface-tests-and-evidence 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- rest-contract-surface-tests-and-evidence 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Rollback
- rest-contract-surface-rollback 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- rest-contract-surface-rollback 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- rest-contract-surface-rollback 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- rest-contract-surface-rollback 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- rest-contract-surface-rollback 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- rest-contract-surface-rollback 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Acceptance criteria
- rest-contract-surface-acceptance-criteria 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- rest-contract-surface-acceptance-criteria 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- rest-contract-surface-acceptance-criteria 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- rest-contract-surface-acceptance-criteria 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- rest-contract-surface-acceptance-criteria 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- rest-contract-surface-acceptance-criteria 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Context
- IP-005 makes the public REST surface the stable contract for tenant OLAP operations.
- Snowflake parity requires query submission, warehouse resize, secure-share creation, and export status endpoints.
- BigQuery parity requires job submission, reservation-aware execution, dataset export, and policy tag response fields.
- Redshift parity requires query run, WLM queue visibility, datashare publication, and unload/export status.
- Databricks SQL parity requires statement execution, warehouse lifecycle, catalog scope grants, and Delta Sharing status.
- Synapse Analytics parity requires SQL pool query execution, pool pause/resume, and export job tracking.
- Firebolt parity requires engine-backed query run, index-aware latency hints, and export initiation.
- ClickHouse Cloud parity requires query execution with settings, service capacity hints, and materialized result export.
- Vertica parity requires query execution, resource-pool selection, and projection-aware explain metadata.
- Teradata Vantage parity requires query bands, workload rules, and governed result export.
- Yellowbrick parity requires resource group query queues and storage locality hints.
- REST handlers are thin API adapters; domain decisions stay in application/usecase layers.
- Every request carries tenant, principal, idempotency, trace, and audit linkage.

## Data Model Deltas
```sql
CREATE TABLE dw_rest_request_ledger (
  request_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  principal_id uuid NOT NULL,
  rest_operation text NOT NULL,
  idempotency_key text NOT NULL,
  request_body_hash text NOT NULL,
  response_body_hash text,
  status_code integer,
  audit_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, rest_operation, idempotency_key)
);
CREATE TABLE dw_rest_operation_capability (
  operation_slug text PRIMARY KEY,
  cedar_action text NOT NULL,
  resource_kind text NOT NULL,
  async_result_channel text,
  slo_profile text NOT NULL
);
```
```rust
pub struct RestRequestLedger {
    pub request_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub principal_id: uuid::Uuid,
    pub rest_operation: String,
    pub idempotency_key: String,
    pub request_body_hash: String,
    pub response_body_hash: Option<String>,
    pub status_code: Option<u16>,
    pub audit_id: uuid::Uuid,
}
pub struct WarehouseQueryRunRequest {
    pub tenant_id: uuid::Uuid,
    pub workload_pool_id: uuid::Uuid,
    pub sql_text: String,
    pub statement_parameters: serde_json::Value,
    pub result_policy: ResultPolicy,
}
pub enum ResultPolicy { InlineLimit { max_rows: u32 }, AsyncExport { format: String }, DenyResultMaterialization }
```

## API Endpoints
- REST `POST /v1/data-warehouse/queries:run` submits a tenant-scoped query.
```json
{"tenant_id":"018f-tenant","workload_pool_id":"018f-pool","sql_text":"select region, sum(amount) from sales group by region","statement_parameters":{},"result_policy":{"inline_limit":{"max_rows":500}}}
```
- REST `POST /v1/data-warehouse/workload-pools/{pool_id}:resize` changes capacity after FinOps guardrails.
```json
{"tenant_id":"018f-tenant","target_size":"medium","budget_justification":"quarter-close","effective_until":"2026-05-21T00:00:00Z"}
```
- REST `POST /v1/data-warehouse/governed-shares` creates a governed share with DealSet linkage.
```json
{"tenant_id":"018f-tenant","dataset_id":"018f-dataset","consumer_tenant_id":"018f-consumer","dealset_id":"018f-dealset","expiration":"2026-06-20T00:00:00Z"}
```
- gRPC parity call `RunWarehouseQuery(RunWarehouseQueryRequest) returns (RunWarehouseQueryResponse)`.
```json
{"tenantId":"018f-tenant","workloadPoolId":"018f-pool","sqlText":"select count(*) from fact_orders","resultMode":"ASYNC_EXPORT"}
```
- AsyncAPI channel `data-warehouse.rest.query.accepted.v1`.
```json
{"tenant_id":"018f-tenant","request_id":"018f-request","query_id":"018f-query","audit_event_class":"WarehouseRestQueryAccepted"}
```

## Cedar Policy Hooks
- principal: `DataWarehouseOperator::"principal_id"`, `TenantAdmin::"principal_id"`, or `ServicePrincipal::"principal_id"`.
- action: `Action::"dataWarehouse::RestQueryRun"`, `Action::"dataWarehouse::RestPoolResize"`, or `Action::"dataWarehouse::RestGovernedShareCreate"`.
- resource: `WarehouseDataset::"tenant_id/dataset_id"` or `WarehouseWorkloadPool::"tenant_id/pool_id"`.
- context: `tenant_id`, `idempotency_key`, `request_body_hash`, `audience_type`, `data_class`, `budget_impact`, `audit_event_class`.
- permit requires matching tenant, canonical idempotency key, approved audience, and data class compatible with requested result policy.
- deny if SQL text references unprojected objects.
- deny resize when budget impact exceeds FinOps approved delta.
- deny governed share when marketplace DealSet status is not active.

## Ontology Projection
| Vendor object | Oyatie object | Field deltas |
| --- | --- | --- |
| Snowflake REST SQL API statement | `WarehouseQueryRun` | `statementHandle` -> `query_id`, `warehouse` -> `workload_pool_id` |
| BigQuery jobs.insert | `WarehouseQueryRun` | `jobReference` -> `query_id`, `maximumBytesBilled` -> `budget_ceiling` |
| Redshift Data API statement | `WarehouseQueryRun` | `statement_id` -> `query_id`, `workgroup_name` -> `workload_pool_id` |
| Databricks SQL statement | `WarehouseQueryRun` | `statement_id` -> `query_id`, `warehouse_id` -> `workload_pool_id` |
| Synapse SQL request | `WarehouseQueryRun` | `request_id` -> `query_id`, `pool_name` -> `workload_pool_id` |
| Firebolt query | `WarehouseQueryRun` | `query_id` -> `query_id`, `engine_name` -> `workload_pool_id` |
| ClickHouse query_id | `WarehouseQueryRun` | `query_id` -> `query_id`, `settings` -> `execution_options` |
| Vertica query request | `WarehouseQueryRun` | `transaction_id` -> `query_id`, `resource_pool` -> `workload_pool_id` |
| Teradata query band | `WarehouseQueryRun` | `query_band` -> `policy_context`, `session_id` -> `query_id` |
| Yellowbrick query queue | `WarehouseQueryRun` | `query_id` -> `query_id`, `resource_group` -> `workload_pool_id` |

## Workflow Steps
- node `parse_rest_envelope`: verify tenant, principal, trace context, and idempotency key.
- node `hash_request_body`: persist canonical hash before policy evaluation.
- node `evaluate_cedar`: check action/resource/context against tenant policy.
- branch `operation_is_async`: return `202 Accepted` and publish accepted event.
- node `dispatch_usecase`: call query, resize, retention, export, or share usecase.
- branch `inline_result_allowed`: cap rows and redact columns before response serialization.
- node `record_response_hash`: persist response hash and status code.
- node `seal_rest_audit`: link request ledger row to ADR-0263 audit event.

## Audit Events
- `WarehouseRestRequestReceived`: request envelope accepted for validation.
- `WarehouseRestPolicyDenied`: Cedar denial with action/resource/context hash.
- `WarehouseRestQueryAccepted`: async query accepted.
- `WarehouseRestPoolResizeAccepted`: capacity resize accepted.
- `WarehouseRestGovernedShareCreated`: share created and DealSet linked.
- `WarehouseRestRequestCompleted`: final response hash persisted.

## SLO Targets
| Metric | Target |
| --- | --- |
| p50 REST admission latency | 80 ms |
| p95 REST admission latency | 250 ms |
| p99 REST admission latency | 650 ms |
| throughput | 1,200 REST requests/sec per regional cell |
| availability | 99.97% for REST control surface |

## Failure Modes + Recovery
- Duplicate idempotency key with different body hash: return `409`, keep original ledger, emit policy-safe audit event.
- Missing trace context: create server root span, mark context repair, and allow only read-only operations.
- SQL parser timeout: fail query admission before vendor dispatch and return `422`.
- DealSet service unavailable: accept no governed-share writes and expose retry-after.
- Inline result too large: downgrade to async export when policy permits or return `413`.
- Audit ledger write failure: fail closed before usecase dispatch.

## Migration Notes
- Snowflake SQL API callers map `statementHandle` polling to Oyatie query status endpoints.
- BigQuery Jobs API callers map project/dataset fields to tenant and dataset ids.
- Redshift Data API callers map cluster/workgroup to `workload_pool_id`.
- Databricks SQL Statement API callers map warehouse id and catalog context into request fields.
- Synapse callers map SQL pool endpoints to regional cell workload pools.
- Firebolt callers map engine name to workload pool and account to tenant alias.
- ClickHouse Cloud callers map query settings into explicit execution options.
- Vertica callers map resource pool to workload pool and projection hints to explain metadata.
- Teradata callers map query bands to Cedar context fields.
- Yellowbrick callers map resource groups and queues to admission controls.

## Cross-Microservice Handoffs
- API Gateway receives route, authn, rate-limit, and idempotency header requirements.
- Identity supplies principal projection and audience type.
- Policy-engine evaluates every REST operation before usecase dispatch.
- Audit-chain seals request and response hashes.
- Workflow receives async operation run ids.
- FinOps validates resize and high-cost query budgets.
- Marketplace validates governed-share DealSet status.
- Ontology resolves all dataset and workload object identifiers.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-005-rest-contract-surface.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-005-rest-contract-surface.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
