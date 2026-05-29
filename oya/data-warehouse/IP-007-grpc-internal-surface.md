---
doc_class: IP
ip_id: IP-007-grpc-internal-surface
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
journey_ref: J-DW-007-grpc-internal-surface
capability_profile: Tier-1
status: deepened
date: 2026-05-20
owner_team: data-platform-warehouse
---

# IP-007 Data Warehouse grpc-internal-surface

Service: data-warehouse
ChangeSet scope: microservices/data-warehouse/IP-007-grpc-internal-surface.md
Benchmarks: Snowflake, Databricks, Google BigQuery, AWS Redshift, ClickHouse
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- grpc-internal-surface-objective 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- grpc-internal-surface-objective 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- grpc-internal-surface-objective 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- grpc-internal-surface-objective 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- grpc-internal-surface-objective 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- grpc-internal-surface-objective 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Prerequisites
- grpc-internal-surface-prerequisites 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- grpc-internal-surface-prerequisites 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- grpc-internal-surface-prerequisites 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- grpc-internal-surface-prerequisites 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- grpc-internal-surface-prerequisites 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- grpc-internal-surface-prerequisites 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Implementation steps
- grpc-internal-surface-implementation-steps 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- grpc-internal-surface-implementation-steps 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- grpc-internal-surface-implementation-steps 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- grpc-internal-surface-implementation-steps 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- grpc-internal-surface-implementation-steps 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- grpc-internal-surface-implementation-steps 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Tests and evidence
- grpc-internal-surface-tests-and-evidence 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- grpc-internal-surface-tests-and-evidence 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- grpc-internal-surface-tests-and-evidence 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- grpc-internal-surface-tests-and-evidence 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- grpc-internal-surface-tests-and-evidence 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- grpc-internal-surface-tests-and-evidence 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Rollback
- grpc-internal-surface-rollback 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- grpc-internal-surface-rollback 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- grpc-internal-surface-rollback 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- grpc-internal-surface-rollback 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- grpc-internal-surface-rollback 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- grpc-internal-surface-rollback 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Acceptance criteria
- grpc-internal-surface-acceptance-criteria 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- grpc-internal-surface-acceptance-criteria 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- grpc-internal-surface-acceptance-criteria 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- grpc-internal-surface-acceptance-criteria 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- grpc-internal-surface-acceptance-criteria 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- grpc-internal-surface-acceptance-criteria 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Context
- IP-007 defines internal gRPC contracts used by workflow, ontology, FinOps, marketplace, and audit-chain callers.
- Snowflake adapter calls need typed query, warehouse, share, and grant messages.
- BigQuery adapter calls need typed job, reservation, export, and row-policy messages.
- Redshift adapter calls need typed statement, WLM, datashare, and unload messages.
- Databricks SQL adapter calls need typed statement, warehouse, catalog, and Delta Sharing messages.
- Synapse Analytics adapter calls need typed workspace, pool, linked service, and request messages.
- Firebolt adapter calls need typed engine, database, index, and query messages.
- ClickHouse Cloud adapter calls need typed service, table, dictionary, and query-setting messages.
- Vertica adapter calls need typed resource pool, projection, external table, and query messages.
- Teradata Vantage adapter calls need typed session, query-band, TASM, and workload exception messages.
- Yellowbrick adapter calls need typed resource group, queue, storage stripe, and query messages.
- gRPC is internal only; public callers use REST or generated SDKs.
- Every method uses tenant-scoped metadata, W3C trace context, deadline, and audit correlation.

## Data Model Deltas
```sql
CREATE TABLE dw_grpc_method_contract (
  method_slug text PRIMARY KEY,
  package_name text NOT NULL,
  request_type text NOT NULL,
  response_type text NOT NULL,
  cedar_action text NOT NULL,
  deadline_ms integer NOT NULL,
  idempotent boolean NOT NULL,
  audit_event_class text NOT NULL
);
CREATE TABLE dw_grpc_call_ledger (
  call_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  method_slug text NOT NULL REFERENCES dw_grpc_method_contract(method_slug),
  caller_service text NOT NULL,
  grpc_status text,
  deadline_ms integer NOT NULL,
  audit_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
```
```rust
pub struct GrpcCallLedger {
    pub call_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub method_slug: String,
    pub caller_service: String,
    pub grpc_status: Option<String>,
    pub deadline_ms: u32,
    pub audit_id: uuid::Uuid,
}
pub struct InternalQueryPlanRequest {
    pub tenant_id: uuid::Uuid,
    pub dataset_ids: Vec<uuid::Uuid>,
    pub workload_pool_id: uuid::Uuid,
    pub sql_hash: String,
    pub policy_scope_hash: String,
}
pub enum InternalWarehouseStatus { Accepted, Running, Succeeded, Failed, Cancelled, PolicyDenied }
```

## API Endpoints
- REST `POST /v1/data-warehouse/internal/grpc-descriptors:inspect` is the compatibility endpoint for operator-only descriptor diff checks.
```json
{"tenant_id":"018f-tenant","package_name":"oyatie.data_warehouse.v1","include_methods":true}
```
- gRPC `PlanWarehouseQuery(InternalQueryPlanRequest) returns (InternalQueryPlanResponse)`.
```json
{"tenantId":"018f-tenant","datasetIds":["018f-dataset"],"workloadPoolId":"018f-pool","sqlHash":"sha256:abc","policyScopeHash":"sha256:def"}
```
- gRPC `ResizeWarehousePool(ResizeWarehousePoolRequest) returns (ResizeWarehousePoolResponse)`.
```json
{"tenantId":"018f-tenant","workloadPoolId":"018f-pool","targetCapacityUnits":16,"budgetApprovalId":"018f-budget"}
```
- AsyncAPI channel `data-warehouse.grpc.call.failed.v1`.
```json
{"tenant_id":"018f-tenant","method_slug":"PlanWarehouseQuery","grpc_status":"DEADLINE_EXCEEDED","audit_event_class":"WarehouseGrpcCallFailed"}
```

## Cedar Policy Hooks
- principal: `ServicePrincipal::"workflow"`, `ServicePrincipal::"ontology"`, or `ServicePrincipal::"finops"`.
- action: `Action::"dataWarehouse::InternalGrpcCall"`.
- resource: `WarehouseGrpcMethod::"package/method_slug"`.
- context: `tenant_id`, `caller_service`, `method_slug`, `deadline_ms`, `idempotent`, `audit_event_class`, `trace_id`.
- permit requires service-to-service trust, tenant propagation, method allowlist, and deadline below method ceiling.
- deny if metadata lacks `x-oya-tenant-id` or traceparent.
- deny if caller asks for adapter credentials directly.
- deny if non-idempotent method lacks idempotency key.

## Ontology Projection
| Vendor object | Oyatie object | Field deltas |
| --- | --- | --- |
| Snowflake connector session | `WarehouseGrpcAdapterCall` | `session_id` -> `adapter_call_ref`, `warehouse` -> `pool_ref` |
| BigQuery RPC job | `WarehouseGrpcAdapterCall` | `job_id` -> `adapter_call_ref`, `project` -> `source_account_ref` |
| Redshift Data API call | `WarehouseGrpcAdapterCall` | `statement_id` -> `adapter_call_ref`, `workgroup` -> `pool_ref` |
| Databricks SQL statement call | `WarehouseGrpcAdapterCall` | `statement_id` -> `adapter_call_ref`, `warehouse_id` -> `pool_ref` |
| Synapse control call | `WarehouseGrpcAdapterCall` | `request_id` -> `adapter_call_ref`, `workspace` -> `cell_ref` |
| Firebolt engine call | `WarehouseGrpcAdapterCall` | `engine` -> `pool_ref`, `account` -> `source_account_ref` |
| ClickHouse HTTP/native call | `WarehouseGrpcAdapterCall` | `query_id` -> `adapter_call_ref`, `service` -> `cell_ref` |
| Vertica management call | `WarehouseGrpcAdapterCall` | `session_id` -> `adapter_call_ref`, `resource_pool` -> `pool_ref` |
| Teradata CLIv2 call | `WarehouseGrpcAdapterCall` | `session_no` -> `adapter_call_ref`, `query_band` -> `policy_context` |
| Yellowbrick management call | `WarehouseGrpcAdapterCall` | `query_id` -> `adapter_call_ref`, `resource_group` -> `pool_ref` |

## Workflow Steps
- node `extract_grpc_metadata`: validate tenant, traceparent, caller service, and idempotency key.
- node `check_descriptor_version`: reject callers using retired proto versions.
- node `evaluate_service_policy`: run Cedar for method and caller.
- branch `method_requires_budget`: call FinOps before query plan or resize execution.
- node `invoke_domain_usecase`: execute typed usecase without exposing vendor credentials.
- branch `deadline_budget_low`: return retryable status before adapter dispatch.
- node `record_grpc_ledger`: persist status, method, deadline, and audit id.
- node `emit_grpc_event`: publish failure or completion event for observability.

## Audit Events
- `WarehouseGrpcCallAccepted`: internal method admitted.
- `WarehouseGrpcPolicyDenied`: Cedar denied internal call.
- `WarehouseGrpcDeadlineExceeded`: method exceeded contract deadline.
- `WarehouseGrpcCallFailed`: method returned non-ok status.
- `WarehouseGrpcDescriptorVersionRejected`: caller used retired proto version.
- `WarehouseGrpcCallCompleted`: ledger row sealed with final status.

## SLO Targets
| Metric | Target |
| --- | --- |
| p50 internal gRPC admission | 20 ms |
| p95 internal gRPC admission | 90 ms |
| p99 internal gRPC admission | 200 ms |
| throughput | 8,000 unary calls/sec per cell |
| availability | 99.98% for internal gRPC surface |

## Failure Modes + Recovery
- Missing tenant metadata: reject with `INVALID_ARGUMENT` and audit caller service.
- Proto version mismatch: return `FAILED_PRECONDITION` and include descriptor migration link.
- Deadline exceeded before adapter call: return retryable failure and no vendor mutation.
- Caller service not allowlisted: return `PERMISSION_DENIED` and emit policy denial.
- Ledger write failure: fail closed for non-read methods and mark read methods degraded.
- Streaming backpressure: slow producer, keep cursor, and emit gRPC flow-control event.

## Migration Notes
- Snowflake adapter RPCs wrap JDBC/SQL API calls behind typed internal methods.
- BigQuery adapter RPCs wrap Jobs, Reservations, and Datasets calls.
- Redshift adapter RPCs wrap Data API and Serverless workgroup operations.
- Databricks SQL adapter RPCs wrap Statements, Warehouses, and Unity Catalog operations.
- Synapse adapter RPCs wrap ARM and SQL pool calls with regional cell metadata.
- Firebolt adapter RPCs wrap engine lifecycle and query calls.
- ClickHouse Cloud adapter RPCs wrap service API and query execution.
- Vertica adapter RPCs wrap management API and SQL execution.
- Teradata adapter RPCs wrap workload and query-band calls.
- Yellowbrick adapter RPCs wrap query queue and resource-group APIs.

## Cross-Microservice Handoffs
- Workflow calls internal gRPC methods for durable node execution.
- Ontology calls descriptor-backed methods for projection enrichment.
- FinOps calls workload and query estimate methods.
- Audit-chain receives call ledger and status events.
- Observability consumes gRPC latency, status, and deadline metrics.
- SDK generation consumes descriptors but exposes only public-safe methods.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-007-grpc-internal-surface.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-007-grpc-internal-surface.md` matched `cost, finops`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
