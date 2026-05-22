---
doc_class: IP
ip_id: IP-008-policy-eval-library-binding
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
journey_ref: J-DW-008-policy-eval-library-binding
capability_profile: Tier-1
status: deepened
date: 2026-05-20
owner_team: data-platform-warehouse
---

# IP-008 Data Warehouse policy-eval-library-binding

Service: data-warehouse
ChangeSet scope: microservices/data-warehouse/IP-008-policy-eval-library-binding.md
Benchmarks: Snowflake, Databricks, Google BigQuery, AWS Redshift, ClickHouse
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- policy-eval-library-binding-objective 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- policy-eval-library-binding-objective 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- policy-eval-library-binding-objective 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- policy-eval-library-binding-objective 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- policy-eval-library-binding-objective 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- policy-eval-library-binding-objective 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Prerequisites
- policy-eval-library-binding-prerequisites 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- policy-eval-library-binding-prerequisites 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- policy-eval-library-binding-prerequisites 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- policy-eval-library-binding-prerequisites 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- policy-eval-library-binding-prerequisites 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- policy-eval-library-binding-prerequisites 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Implementation steps
- policy-eval-library-binding-implementation-steps 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- policy-eval-library-binding-implementation-steps 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- policy-eval-library-binding-implementation-steps 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- policy-eval-library-binding-implementation-steps 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- policy-eval-library-binding-implementation-steps 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- policy-eval-library-binding-implementation-steps 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Tests and evidence
- policy-eval-library-binding-tests-and-evidence 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- policy-eval-library-binding-tests-and-evidence 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- policy-eval-library-binding-tests-and-evidence 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- policy-eval-library-binding-tests-and-evidence 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- policy-eval-library-binding-tests-and-evidence 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- policy-eval-library-binding-tests-and-evidence 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Rollback
- policy-eval-library-binding-rollback 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- policy-eval-library-binding-rollback 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- policy-eval-library-binding-rollback 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- policy-eval-library-binding-rollback 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- policy-eval-library-binding-rollback 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- policy-eval-library-binding-rollback 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Acceptance criteria
- policy-eval-library-binding-acceptance-criteria 001: Data Warehouse binds warehouse-query-run to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.
- policy-eval-library-binding-acceptance-criteria 002: Data Warehouse binds workload-pool-resize to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Databricks plus Google BigQuery.
- policy-eval-library-binding-acceptance-criteria 003: Data Warehouse binds retention-tier-apply to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=retention_tier, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Google BigQuery plus AWS Redshift.
- policy-eval-library-binding-acceptance-criteria 004: Data Warehouse binds cost-budget-enforce to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=cost_allocation_row, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against AWS Redshift plus ClickHouse.
- policy-eval-library-binding-acceptance-criteria 005: Data Warehouse binds dataset-export to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=warehouse_query, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against ClickHouse plus Snowflake.
- policy-eval-library-binding-acceptance-criteria 006: Data Warehouse binds governed-share-create to tenant_id, principal_id, audience_type=DATA_PLATFORM_OPERATOR, data_class=workload_pool, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Snowflake plus Databricks.

## Context
- IP-008 binds all warehouse actions to the shared Cedar evaluation library instead of service-local policy logic.
- Snowflake-style grants become Cedar resources and actions before connector calls.
- BigQuery IAM, policy tags, and row access policies become Cedar context and resource edges.
- Redshift grants, datashare consumers, and WLM operations become Cedar-gated actions.
- Databricks SQL Unity Catalog privileges and SQL warehouse permissions become Cedar resource scopes.
- Synapse Analytics workspace roles, SQL permissions, and linked-service access become Cedar decisions.
- Firebolt account roles, database grants, and engine operations become Cedar decisions.
- ClickHouse Cloud RBAC, row policies, and quota settings become Cedar decisions.
- Vertica roles, resource pools, and projection visibility become Cedar decisions.
- Teradata Vantage profiles, roles, query bands, and TASM rules become Cedar decisions.
- Yellowbrick database roles and resource group operations become Cedar decisions.
- The binding library returns structured decision evidence for REST, gRPC, async replay, workflow nodes, and adapter dispatch.
- Policy decisions must be cacheable only when tenant, principal, action, resource, and context hashes match.

## Data Model Deltas
```sql
CREATE TABLE dw_policy_decision_cache (
  decision_id uuid PRIMARY KEY,
  tenant_id uuid NOT NULL,
  principal_hash text NOT NULL,
  action text NOT NULL,
  resource_hash text NOT NULL,
  context_hash text NOT NULL,
  effect text NOT NULL CHECK (effect IN ('permit','deny')),
  reason_code text NOT NULL,
  expires_at timestamptz NOT NULL,
  audit_id uuid NOT NULL,
  UNIQUE (tenant_id, principal_hash, action, resource_hash, context_hash)
);
CREATE TABLE dw_policy_hook_registry (
  hook_slug text PRIMARY KEY,
  capability text NOT NULL,
  cedar_action text NOT NULL,
  resource_kind text NOT NULL,
  required_context_keys text[] NOT NULL,
  cache_ttl_seconds integer NOT NULL
);
```
```rust
pub struct WarehousePolicyDecision {
    pub decision_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub principal_hash: String,
    pub action: String,
    pub resource_hash: String,
    pub context_hash: String,
    pub effect: PolicyEffect,
    pub reason_code: String,
    pub expires_at: time::OffsetDateTime,
    pub audit_id: uuid::Uuid,
}
pub enum PolicyEffect { Permit, Deny }
pub struct CedarWarehouseContext {
    pub tenant_id: uuid::Uuid,
    pub audience_type: String,
    pub data_class: String,
    pub source_vendor: Option<String>,
    pub audit_event_class: String,
    pub budget_impact_minor_units: i64,
}
```

## API Endpoints
- REST `POST /v1/data-warehouse/policy:explain` returns operator-visible decision evidence.
```json
{"tenant_id":"018f-tenant","action":"dataWarehouse::RunQuery","resource":"WarehouseDataset::018f-dataset","context":{"data_class":"warehouse_query","source_vendor":"clickhouse_cloud"}}
```
- REST `GET /v1/data-warehouse/policy-hooks/{hook_slug}` exposes required context keys for contract tests.
```json
{"hook_slug":"warehouse-query-run","capability":"warehouse-query-run","required_context_keys":["tenant_id","data_class","audit_event_class"]}
```
- gRPC `EvaluateWarehousePolicy(EvaluateWarehousePolicyRequest) returns (EvaluateWarehousePolicyResponse)`.
```json
{"tenantId":"018f-tenant","principal":"DataWarehouseOperator::018f-principal","action":"dataWarehouse::ResizePool","resource":"WarehouseWorkloadPool::018f-pool"}
```
- AsyncAPI channel `data-warehouse.policy.decision.v1`.
```json
{"tenant_id":"018f-tenant","decision_id":"018f-decision","effect":"deny","reason_code":"budget_approval_missing","audit_event_class":"WarehousePolicyDecisionRecorded"}
```

## Cedar Policy Hooks
- principal: `DataWarehouseOperator`, `TenantAdmin`, `ServicePrincipal`, or `WarehouseEventSubscriber`.
- action: `dataWarehouse::RunQuery`, `ResizePool`, `ApplyRetention`, `CreateGovernedShare`, `ReplayEvents`, `ProjectOntology`.
- resource: `WarehouseDataset`, `WarehouseWorkloadPool`, `WarehouseGovernedShare`, `WarehouseProjectionSource`, or `WarehouseEventChannel`.
- context: `tenant_id`, `audience_type`, `data_class`, `source_vendor`, `budget_impact`, `dealset_id`, `audit_event_class`, `regional_pack`.
- permit requires tenant equality and action-specific capability tier.
- deny if `source_vendor` metadata is absent for migrated vendor-backed objects.
- deny if policy cache entry is expired or was computed without the current regional pack.
- deny if query action requests result materialization for restricted data class.

## Ontology Projection
| Vendor object | Oyatie object | Field deltas |
| --- | --- | --- |
| Snowflake role grant | `WarehousePolicyGrant` | `role` -> `principal_ref`, `privilege` -> `cedar_action` |
| BigQuery IAM binding | `WarehousePolicyGrant` | `member` -> `principal_ref`, `role` -> `cedar_action` |
| Redshift ACL | `WarehousePolicyGrant` | `grantee` -> `principal_ref`, `privilege_type` -> `cedar_action` |
| Databricks privilege | `WarehousePolicyGrant` | `principal` -> `principal_ref`, `privilege` -> `cedar_action` |
| Synapse role assignment | `WarehousePolicyGrant` | `assignee` -> `principal_ref`, `roleDefinitionId` -> `cedar_action` |
| Firebolt role | `WarehousePolicyGrant` | `role_name` -> `principal_group_ref`, `privileges` -> `cedar_action_set` |
| ClickHouse grant | `WarehousePolicyGrant` | `role_name` -> `principal_group_ref`, `access_type` -> `cedar_action` |
| Vertica role grant | `WarehousePolicyGrant` | `role_name` -> `principal_group_ref`, `object_name` -> `resource_ref` |
| Teradata profile | `WarehousePolicyGrant` | `profile_name` -> `resource_governor`, `role_name` -> `principal_ref` |
| Yellowbrick privilege | `WarehousePolicyGrant` | `user_name` -> `principal_ref`, `resource_group` -> `resource_ref` |

## Workflow Steps
- node `assemble_principal`: normalize identity and service principal claims.
- node `assemble_resource`: resolve warehouse object, vendor backing, and tenant ownership.
- node `assemble_context`: require all hook keys and hash context.
- branch `cache_eligible`: use cached permit only for read-only and stable-context actions.
- node `evaluate_cedar_library`: call shared Cedar library and capture policy ids.
- branch `deny`: stop operation, persist reason code, and emit decision event.
- node `persist_decision`: store decision evidence with TTL and audit id.
- node `return_decision`: send effect, reason, and obligation list to caller.

## Audit Events
- `WarehousePolicyDecisionRequested`: policy input accepted.
- `WarehousePolicyDecisionCacheHit`: cache used for stable-context decision.
- `WarehousePolicyDecisionRecorded`: permit or deny persisted.
- `WarehousePolicyContextMissing`: required context key absent.
- `WarehousePolicyVendorScopeMissing`: migrated vendor object lacks source vendor metadata.
- `WarehousePolicyCacheExpired`: stale cache entry rejected.

## SLO Targets
| Metric | Target |
| --- | --- |
| p50 policy decision latency | 5 ms |
| p95 policy decision latency | 25 ms |
| p99 policy decision latency | 75 ms |
| throughput | 25,000 decisions/sec per cell |
| availability | 99.99% for policy evaluation binding |

## Failure Modes + Recovery
- Cedar library unavailable: fail closed for writes, allow no cache refresh, and alert policy-engine.
- Missing context key: deny with machine-readable reason and contract-test pointer.
- Cache poisoning suspicion: purge tenant cache partition and force fresh evaluations.
- Vendor grant import mismatch: quarantine grant projection and deny affected actions.
- Regional pack update: expire all decisions with old regional-pack hash.
- Audit event write failure: return denial for state-changing actions.

## Migration Notes
- Snowflake grants migrate from role SQL to Cedar grants with source role retained for audit.
- BigQuery IAM roles migrate to action sets and policy tags become context fields.
- Redshift privileges and datashares migrate into resource-specific policies.
- Databricks Unity Catalog privileges migrate with catalog/schema/table resource hierarchy.
- Synapse workspace RBAC migrates with SQL permission overlays.
- Firebolt roles migrate with engine operations separated from database operations.
- ClickHouse RBAC migrates row policies and quotas into context obligations.
- Vertica resource pool controls migrate to workload-pool actions.
- Teradata query-band rules migrate to Cedar context predicates.
- Yellowbrick resource group permissions migrate to workload-pool resources.

## Cross-Microservice Handoffs
- Policy-engine owns library version, policy store, and decision semantics.
- Identity owns principal canonicalization before policy evaluation.
- Tenancy owns tenant hierarchy and sovereign child constraints.
- Audit-chain owns decision evidence sealing.
- Ontology receives policy grant projections.
- FinOps receives budget-related obligations from decisions.
- Workflow receives denial branches and remediation tasks.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-008-policy-eval-library-binding.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-008-policy-eval-library-binding.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
