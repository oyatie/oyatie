---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-001-tenant-scope-kernel
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-tenancy
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0008, ADR-0009, ADR-0045, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0294, ADR-0314]
journey_ref: J-DW-001-tenant-scope-kernel
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/tenant-model.json, specs/platform-architecture.json]
write_scope: microservices/data-warehouse/IP-001-tenant-scope-kernel.md
---

# IP-001 Tenant Scope Kernel

## Goal

Deliver the tenant-scope kernel for the Data Warehouse service so every warehouse, database, schema, share, notebook, query, grant, budget, retention tier, and export is bound to a provable Oyatie tenant context before any compute, storage, or marketplace settlement action runs.

## Competitive Displacement Thesis

Snowflake, BigQuery, Redshift, Databricks SQL, Synapse Analytics, Firebolt, and ClickHouse Cloud all expose strong account, project, workspace, warehouse, database, or role concepts, but Oyatie must make tenant scope the first-class kernel primitive rather than a surrounding billing or IAM label.

The displacement point is not a larger SQL surface. It is a governed scope ledger that makes every query plan, cache key, result set, share, data product, and cost event impossible to separate from `tenant_id`, `cell_id`, `jurisdiction_pack_set`, `principal_id`, and `dealset_id`.

This IP defines the smallest kernel that downstream REST, async, gRPC, cost, residency, catalog, and SLO IPs can rely on without re-solving tenant isolation.

## Non Goals

- This IP does not define SQL grammar or optimizer rules; those belong to query execution IPs.
- This IP does not define any content outside the Data Warehouse IP scope.
- This IP does not introduce a shared global warehouse namespace.
- This IP does not allow provider-native account identifiers to become the authoritative scope.

## Domain Objects

| Object | Purpose | Primary citation |
|---|---|---|
| `WarehouseTenantScope` | Immutable tenant boundary for every warehouse artifact | ADR-0002 |
| `WarehouseCellBinding` | Region and cell placement of warehouse state | ADR-0009 |
| `WarehousePrincipalBinding` | Principal, role, and audience tuple authorized for warehouse action | ADR-0008 |
| `WarehouseScopeGrant` | Time-bounded grant to dataset, schema, share, or query surface | ADR-0105 |
| `WarehouseDealSetBinding` | Marketplace and settlement link for governed data products | ADR-0314 |
| `WarehouseCostScope` | Per-tenant cost attribution ledger binding | ADR-0199 |
| `WarehouseResidencyScope` | Jurisdiction pack and residency envelope | specs/tenant-model.json |

## Scope Invariants

- Invariant 001: `tenant_id` is mandatory for every warehouse command; empty, wildcard, inherited, and provider-default tenant values are rejected under ADR-0002.
- Invariant 002: `cell_id` is mandatory for storage, compute, catalog, and result-cache placement under ADR-0009.
- Invariant 003: `principal_id` and `audience_type` are resolved before SQL parsing so policy denial can occur before planner work under ADR-0008.
- Invariant 004: `scope_version` increments on every tenant membership, grant, residency, or budget mutation under ADR-0105.
- Invariant 005: `scope_version` participates in result-cache keys so a revoked grant cannot reuse a stale answer under ADR-0008.
- Invariant 006: `jurisdiction_pack_set` is copied into every audit event and export manifest under specs/tenant-model.json.
- Invariant 007: `dealset_id` is required for marketplace-provided datasets and prohibited for first-party internal datasets unless ADR-0314 binding exists.
- Invariant 008: `cost_center_id` is required for compute admission and async replay under ADR-0199.
- Invariant 009: provider account, project, workspace, catalog, or database names are external aliases only under ADR-0173.
- Invariant 010: cross-tenant shares require an explicit `WarehouseScopeGrant` and cannot be inferred from SQL object names under ADR-0002.
- Invariant 011: result exports carry the same tenant scope as the query that produced them under ADR-0008.
- Invariant 012: backfill replay must re-evaluate current scope unless the replay is marked forensic-read-only under ADR-0003.
- Invariant 013: policy evaluation sees normalized warehouse actions, not provider-native verbs, under ADR-0105.
- Invariant 014: scope lock acquisition happens before warehouse pool scheduling under ADR-0045.
- Invariant 015: emergency override classes are not part of this service unless a future approved data-warehouse emergency class exists.

## API Contract

```yaml
service: data-warehouse
surface: tenant-scope-kernel
version: v1
authoritative_scope_fields:
  - tenant_id
  - cell_id
  - principal_id
  - audience_type
  - jurisdiction_pack_set
  - scope_version
  - cost_center_id
  - residency_boundary_id
  - dealset_id
```

## Commands

- Command 001: `warehouse.scope.resolve` receives `{tenant_id, principal_id, action, resource_ref}` and returns a signed `WarehouseTenantScope` per ADR-0002.
- Command 002: `warehouse.scope.bind_query` attaches scope to parsed query intent before optimizer admission per ADR-0045.
- Command 003: `warehouse.scope.bind_pool` attaches cost and cell placement before warehouse pool resize per ADR-0199.
- Command 004: `warehouse.scope.bind_share` validates cross-tenant share grants before data product publication per ADR-0314.
- Command 005: `warehouse.scope.bind_export` fixes export residency and retention before object storage write per specs/tenant-model.json.
- Command 006: `warehouse.scope.revoke` invalidates grants, result cache keys, query sessions, and async replay tokens per ADR-0008.
- Command 007: `warehouse.scope.freeze` creates forensic read-only scope for audit preservation per ADR-0003.
- Command 008: `warehouse.scope.unfreeze` requires dual-control authorization and emits audit evidence per ADR-0003.
- Command 009: `warehouse.scope.explain` returns denial reasons without leaking object existence across tenants per ADR-0008.
- Command 010: `warehouse.scope.project_alias` maps provider-native aliases to Oyatie scope without making them authoritative per ADR-0173.

## Storage Model

- Storage 001: `warehouse_tenant_scopes` stores immutable scope records keyed by `{tenant_id, scope_version}` under ADR-0002.
- Storage 002: `warehouse_scope_grants` stores grant edges keyed by `{tenant_id, resource_ref, principal_id}` under ADR-0105.
- Storage 003: `warehouse_cell_bindings` stores cell placement and failover pairs under ADR-0009.
- Storage 004: `warehouse_residency_scopes` stores jurisdiction pack overlays under specs/tenant-model.json.
- Storage 005: `warehouse_cost_scopes` stores cost center and budget attachment under ADR-0199.
- Storage 006: `warehouse_dealset_bindings` stores marketplace dataset settlement envelope under ADR-0314.
- Storage 007: `warehouse_scope_audit_refs` stores audit-chain event pointers under ADR-0003.
- Storage 008: `warehouse_provider_aliases` stores external Snowflake, BigQuery, Redshift, Databricks, Synapse, Firebolt, and ClickHouse aliases under ADR-0173.
- Storage 009: `warehouse_scope_cache` stores short-lived positive policy materialization with `scope_version` in the key under ADR-0008.
- Storage 010: `warehouse_scope_denials` stores redacted denial facts for operator troubleshooting under ADR-0008.

## Snowflake Displacement Requirements

- Snowflake 001: replace account/role/warehouse as the isolation root with `WarehouseTenantScope` under ADR-0002.
- Snowflake 002: require cost scope before virtual warehouse resume under ADR-0199.
- Snowflake 003: bind data sharing to `WarehouseDealSetBinding` instead of provider share names under ADR-0314.
- Snowflake 004: invalidate result reuse on scope grant changes under ADR-0008.
- Snowflake 005: capture region and cell placement independently from provider region labels under ADR-0009.
- Snowflake 006: emit audit-chain references for grant, share, and export decisions under ADR-0003.
- Snowflake 007: model external Snowflake database names as aliases only under ADR-0173.
- Snowflake 008: refuse cross-tenant clone semantics without explicit grant evidence under ADR-0105.
- Snowflake 009: encode residency pack decisions before data product materialization under specs/tenant-model.json.
- Snowflake 010: preserve workload pool names as operator conveniences, not authorization boundaries under ADR-0008.

## BigQuery Displacement Requirements

- BigQuery 001: replace project/dataset defaults with mandatory tenant scope under ADR-0002.
- BigQuery 002: bind job submission to cost center and budget before execution under ADR-0199.
- BigQuery 003: require cell placement for regional dataset creation under ADR-0009.
- BigQuery 004: treat authorized views as `WarehouseScopeGrant` rows under ADR-0105.
- BigQuery 005: prevent cached result reuse after grant revocation under ADR-0008.
- BigQuery 006: attach residency pack to exports and transfer jobs under specs/tenant-model.json.
- BigQuery 007: record query and dataset access as audit-chain events under ADR-0003.
- BigQuery 008: map Google project identifiers as aliases only under ADR-0173.
- BigQuery 009: require marketplace dataset usage to pass DealSet binding under ADR-0314.
- BigQuery 010: redact denial explanations to avoid cross-tenant dataset discovery under ADR-0008.

## Redshift Displacement Requirements

- Redshift 001: replace cluster/database identity with tenant-scoped warehouse identity under ADR-0002.
- Redshift 002: bind concurrency scaling admission to tenant budget under ADR-0199.
- Redshift 003: scope data sharing and datashare consumption through explicit grants under ADR-0105.
- Redshift 004: attach cell placement to RA3 or serverless namespace aliases under ADR-0009.
- Redshift 005: prevent unload/export without residency scope under specs/tenant-model.json.
- Redshift 006: audit role grants and external schema access under ADR-0003.
- Redshift 007: require provider alias mapping for cluster, workgroup, and namespace names under ADR-0173.
- Redshift 008: enforce tenant scope before spectrum or external table planning under ADR-0008.
- Redshift 009: bind marketplace datasets to DealSet settlement before consumption under ADR-0314.
- Redshift 010: reject global default search paths that cross tenant resource roots under ADR-0002.

## Databricks SQL Displacement Requirements

- Databricks 001: replace workspace/catalog defaults with `WarehouseTenantScope` under ADR-0002.
- Databricks 002: bind SQL warehouse start and resize to cost scope under ADR-0199.
- Databricks 003: represent Unity Catalog shares as `WarehouseScopeGrant` records under ADR-0105.
- Databricks 004: attach cell and jurisdiction placement before table materialization under ADR-0009.
- Databricks 005: ensure notebooks cannot bypass scope by direct warehouse token usage under ADR-0008.
- Databricks 006: map workspace, metastore, and warehouse identifiers as aliases under ADR-0173.
- Databricks 007: audit notebook, query, share, and export scope decisions under ADR-0003.
- Databricks 008: bind marketplace datasets to DealSet controls before Delta Sharing use under ADR-0314.
- Databricks 009: prohibit grant inference from catalog naming conventions under ADR-0002.
- Databricks 010: prevent query history disclosure across tenant roots under ADR-0008.

## Synapse Analytics Displacement Requirements

- Synapse 001: replace workspace/pool defaults with tenant scope under ADR-0002.
- Synapse 002: bind dedicated and serverless SQL pool admission to cost scope under ADR-0199.
- Synapse 003: attach cell placement before data lake external table use under ADR-0009.
- Synapse 004: represent linked-service access as provider alias plus scope grant under ADR-0173.
- Synapse 005: ensure pipeline-triggered SQL actions resolve scope before execution under ADR-0105.
- Synapse 006: attach residency pack to exports, CETAS, and data lake writes under specs/tenant-model.json.
- Synapse 007: audit pool, linked service, and notebook access decisions under ADR-0003.
- Synapse 008: prevent Power BI or Fabric-style external consumption without DealSet binding when marketplace data is involved under ADR-0314.
- Synapse 009: redact denied metadata lookups to avoid cross-tenant object discovery under ADR-0008.
- Synapse 010: prohibit tenant inference from Azure subscription or resource group names under ADR-0173.

## Firebolt And ClickHouse Cloud Displacement Requirements

- Firebolt 001: replace database/engine defaults with tenant scope under ADR-0002.
- Firebolt 002: bind engine start and scale operations to cost scope under ADR-0199.
- Firebolt 003: require grant-backed access to external tables and object storage under ADR-0105.
- Firebolt 004: attach cell placement before replicated or multi-region serving under ADR-0009.
- Firebolt 005: prevent result cache reuse after scope mutation under ADR-0008.
- ClickHouse 001: replace organization/service/database defaults with tenant scope under ADR-0002.
- ClickHouse 002: bind cloud service scale-up and query admission to cost scope under ADR-0199.
- ClickHouse 003: require row policy and dictionary access to flow through scope grants under ADR-0105.
- ClickHouse 004: attach residency pack to object storage, backups, and exports under specs/tenant-model.json.
- ClickHouse 005: map service, cluster, and database names as provider aliases under ADR-0173.

## Implementation Steps

- Step 001: define `WarehouseTenantScope` domain type with non-empty validated fields per ADR-0002.
- Step 002: define `WarehouseScopeVersion` monotonic counter and cache-key binding per ADR-0008.
- Step 003: define `WarehouseCellBinding` with active, standby, and forbidden cell sets per ADR-0009.
- Step 004: define `WarehouseCostScope` with budget, cost center, and admission decision fields per ADR-0199.
- Step 005: define `WarehouseResidencyScope` with jurisdiction pack set and export restrictions per specs/tenant-model.json.
- Step 006: define `WarehouseDealSetBinding` for marketplace datasets per ADR-0314.
- Step 007: define provider alias registry for Snowflake, BigQuery, Redshift, Databricks, Synapse, Firebolt, and ClickHouse Cloud per ADR-0173.
- Step 008: add scope resolution port for REST, gRPC, async, and replay callers per ADR-0131.
- Step 009: add denial explanation DTO that redacts resource existence per ADR-0008.
- Step 010: add audit event envelope for scope resolve, grant, revoke, freeze, and export decisions per ADR-0003.
- Step 011: add cache invalidation path keyed by `scope_version` and grant mutation per ADR-0008.
- Step 012: add cross-tenant grant validation with explicit source and target tenant fields per ADR-0002.
- Step 013: add integration fixture for marketplace dataset access requiring DealSet binding per ADR-0314.
- Step 014: add integration fixture for provider alias lookup refusing authority escalation per ADR-0173.
- Step 015: add migration plan for existing thin generated IP lines into this contract without touching out-of-scope services.

## Test And Evidence Plan

- Evidence 001: unit test rejects missing `tenant_id` for every warehouse command under ADR-0002.
- Evidence 002: unit test rejects provider-native account as authoritative tenant value under ADR-0173.
- Evidence 003: unit test invalidates cache key after grant revoke by `scope_version` under ADR-0008.
- Evidence 004: unit test binds cost center before compute admission under ADR-0199.
- Evidence 005: unit test binds cell placement before dataset creation under ADR-0009.
- Evidence 006: unit test binds residency pack before export under specs/tenant-model.json.
- Evidence 007: integration test rejects cross-tenant share without `WarehouseScopeGrant` under ADR-0105.
- Evidence 008: integration test accepts marketplace dataset only with `WarehouseDealSetBinding` under ADR-0314.
- Evidence 009: integration test emits audit event for grant, revoke, and export under ADR-0003.
- Evidence 010: canonicalen denial test proves no cross-tenant object existence leak under ADR-0008.
- Evidence 011: provider parity test maps Snowflake aliases without authority escalation under ADR-0173.
- Evidence 012: provider parity test maps BigQuery aliases without authority escalation under ADR-0173.
- Evidence 013: provider parity test maps Redshift aliases without authority escalation under ADR-0173.
- Evidence 014: provider parity test maps Databricks SQL aliases without authority escalation under ADR-0173.
- Evidence 015: provider parity test maps Synapse aliases without authority escalation under ADR-0173.
- Evidence 016: provider parity test maps Firebolt aliases without authority escalation under ADR-0173.
- Evidence 017: provider parity test maps ClickHouse Cloud aliases without authority escalation under ADR-0173.
- Evidence 018: cost test proves revoked tenant cannot submit query through stale session under ADR-0008.
- Evidence 019: residency test proves export cannot cross forbidden cell boundary under ADR-0009.
- Evidence 020: audit test proves scope decisions carry trace, span, tenant, principal, and cell identifiers under ADR-0003.

## Acceptance Criteria

- Acceptance 001: every Data Warehouse IP can cite this scope kernel as the mandatory tenant binding source.
- Acceptance 002: every provider-displacement surface has explicit alias-only language under ADR-0173.
- Acceptance 003: every query, share, export, and pool action binds tenant, principal, cell, cost, and residency before execution.
- Acceptance 004: every marketplace dataset path binds DealSet settlement before consumption.
- Acceptance 005: every denial path avoids cross-tenant object discovery.
- Acceptance 006: every cache path includes `scope_version`.
- Acceptance 007: every grant mutation emits audit-chain evidence.
- Acceptance 008: every export path carries jurisdiction pack evidence.
- Acceptance 009: every compute admission path carries cost scope evidence.
- Acceptance 010: every cross-tenant share requires an explicit grant.
- Acceptance 011: every provider alias is non-authoritative.
- Acceptance 012: every rollback can revoke grants and invalidate cache without moving storage.
- Acceptance 013: every service interface receives scope by value rather than reading ambient context.
- Acceptance 014: every test fixture names Snowflake, BigQuery, Redshift, Databricks SQL, Synapse Analytics, Firebolt, or ClickHouse Cloud parity where relevant.
- Acceptance 015: this IP remains within `microservices/data-warehouse/IP-*.md` write scope.

## Required Section Addendum

## Context
- Persona: Maya Iyer, warehouse tenant administrator, needs Snowflake, BigQuery, Redshift, Databricks SQL, Synapse Analytics, Firebolt, ClickHouse Cloud, Vertica, Teradata Vantage, and Yellowbrick migrations to share one scope root.
- Vendor surface subsumed: provider account, project, cluster, workspace, service, database, role, share, export, reservation, and cache identifiers.

## Data Model Deltas
```sql
create table dw_tenant_scope_kernels (
    tenant_id uuid not null,
    scope_version bigint not null,
    cell_id text not null,
    jurisdiction_pack_set text[] not null,
    cost_center_id uuid not null,
    residency_boundary_id uuid not null,
    active_policy_bundle text not null,
    audit_event_class text not null,
    primary key (tenant_id, scope_version)
);
```
```rust
pub struct WarehouseTenantScopeKernel { pub tenant_id: Uuid, pub scope_version: i64, pub cell_id: String, pub jurisdiction_pack_set: Vec<String>, pub cost_center_id: Uuid, pub residency_boundary_id: Uuid, pub active_policy_bundle: String, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/scope/resolve
{"tenant_id":"t_dw","principal_id":"p_data_admin","action":"warehouse.query.run","resource_ref":"catalog.finance.margin","source_vendor":"snowflake"}
```
```yaml
grpc: {service: oyatie.data_warehouse.TenantScopeService, rpc: ResolveWarehouseScope}
asyncapi: {publish: data-warehouse.scope.resolved.v1, payload: {tenant_id: uuid, scope_version: integer, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"scope-resolve", resource)
when { context.tenant_id == resource.tenant_id && context.scope_version == resource.scope_version && context.cost_center_id != "" };
forbid(principal, action, resource)
when { context.provider_account_ref == context.tenant_id || context.scope_version < resource.scope_version };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Snowflake account | `WarehouseProviderAlias` | account locator becomes alias |
| BigQuery project | `WarehouseProviderAlias` | project id becomes alias |
| Redshift cluster | `WarehouseWorkloadPool` | cluster ARN becomes capacity ref |
| Databricks workspace | `WarehouseProviderAlias` | workspace id becomes alias |

## Workflow Steps
- Node `resolve-tenant`: validate tenant, principal, action, resource, cost, cell, and residency context.
- Branch `provider-id-as-tenant`: deny and require explicit alias binding.
- Node `read-scope-version`: bind current version into result cache and job replay.
- Branch `scope-version-stale`: invalidate cache and return conflict.
- Node `emit-scope-resolved`: publish ADR-0263 evidence.

## Audit Events
- `DataWarehouseTenantScopeResolved`
- `DataWarehouseTenantScopeDenied`
- `DataWarehouseProviderAliasRejected`
- `DataWarehouseScopeVersionAdvanced`
- `DataWarehouseScopeRollbackBound`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| scope resolve | 8 ms | 35 ms | 75 ms | 5,000 rps per cell | 99.99% |
| scope version update | 30 ms | 150 ms | 320 ms | 500 updates/min | 99.95% |

## Failure Modes + Recovery
- `provider-id-as-tenant`: deny and preserve provider id as alias only.
- `scope-version-stale`: invalidate cache, replay policy, and return 409.
- `cost-center-missing`: deny compute admission and request FinOps binding.
- `residency-boundary-gap`: deny export or external table activation.

## Migration Notes
- Snowflake accounts, BigQuery projects, Redshift clusters, Databricks workspaces, Synapse subscriptions, Firebolt accounts, ClickHouse organizations, Vertica depots, Teradata systems, and Yellowbrick clusters map to aliases.
- Provider IAM and RBAC grants map to Oyatie scope grants only after Cedar evaluation.

## Cross-Microservice Handoffs
- tenancy validates tenant membership.
- policy-engine evaluates scope actions.
- cost-ledger binds compute admission.
- residency validates exports and external locations.
- marketplace binds governed shares.
- audit-chain seals ADR-0263 scope events.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-001-tenant-scope-kernel.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-001-tenant-scope-kernel.md` matched `p99, SLO, multi-region`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-001-tenant-scope-kernel.md` matched `cost, attribution`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
