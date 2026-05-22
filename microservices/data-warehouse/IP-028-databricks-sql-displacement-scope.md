---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-028-databricks-sql-displacement-scope
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-ontology
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0003, ADR-0008, ADR-0009, ADR-0045, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0314]
journey_ref: J-DW-028-databricks-sql-displacement
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/platform-architecture.json, specs/tenant-model.json]
write_scope: microservices/data-warehouse/IP-028-databricks-sql-displacement-scope.md
---

# IP-028 Databricks SQL Displacement Scope

## Goal

Define how Oyatie Data Warehouse displaces Databricks SQL warehouse, workspace, Unity Catalog, Delta Sharing, notebook query, dashboard, and lakehouse governance surfaces with tenant-scoped SQL operations.

## Displacement Summary

Databricks SQL wins by joining lakehouse storage, catalog governance, notebooks, dashboards, SQL warehouses, and sharing. Oyatie must compete by treating notebooks and SQL warehouses as clients of the tenant-scope kernel, not as privileged execution contexts that can bypass policy.

The critical displacement claim is that Unity Catalog-like governance becomes Oyatie `WarehouseScopeGrant` and ontology-backed catalog registration, while SQL warehouse cost and scaling are bound to tenant budgets before any query starts.

## Non Goals

- This IP does not implement Spark execution semantics.
- This IP does not import Databricks workspace identity as Oyatie tenant identity.
- This IP does not edit any content outside the Data Warehouse IP scope.
- This IP does not define ML lifecycle or feature store behavior.

## Surface Map

| Databricks SQL surface | Oyatie replacement | Citation |
|---|---|---|
| Workspace | provider alias plus tenant scope | ADR-0173 |
| SQL warehouse | `WarehouseWorkloadPool` | ADR-0199 |
| Unity Catalog metastore | `WarehouseCatalogNamespace` | ADR-0105 |
| Catalog/schema/table | ontology-linked catalog resource | specs/platform-architecture.json |
| Delta Sharing | `WarehouseGovernedShare` plus DealSet | ADR-0314 |
| Notebook query | scoped query session | ADR-0008 |
| Dashboard | governed result publication | ADR-0105 |
| External location | residency-bound external dataset | specs/tenant-model.json |

## Product Requirements

- Requirement 001: workspace import records workspace URL and ID as alias only under ADR-0173.
- Requirement 002: SQL warehouse import binds pool size to tenant cost scope under ADR-0199.
- Requirement 003: SQL warehouse start requires cost admission under ADR-0199.
- Requirement 004: metastore import creates catalog namespace with tenant binding under ADR-0002.
- Requirement 005: catalog import creates ontology-linked namespace under specs/platform-architecture.json.
- Requirement 006: schema import creates grant-governed child namespace under ADR-0105.
- Requirement 007: table import records residency and cell placement under ADR-0009.
- Requirement 008: external location import validates residency envelope under specs/tenant-model.json.
- Requirement 009: notebook query requires principal binding and current scope under ADR-0008.
- Requirement 010: dashboard publication requires governed result publication grant under ADR-0105.
- Requirement 011: Delta Sharing provider setup requires DealSet binding for commercial data under ADR-0314.
- Requirement 012: Delta Sharing recipient setup requires target tenant grant under ADR-0105.
- Requirement 013: row filter import maps to tenant-aware predicate under ADR-0008.
- Requirement 014: column mask import maps to data-class policy under ADR-0008.
- Requirement 015: query history redacts cross-tenant object names under ADR-0008.
- Requirement 016: lineage import emits audit references under ADR-0003.
- Requirement 017: materialized view refresh requires scope recheck under ADR-0105.
- Requirement 018: scheduled dashboard refresh requires budget admission under ADR-0199.
- Requirement 019: serverless SQL routing must preserve cell boundary under ADR-0009.
- Requirement 020: all imported identifiers remain provider aliases under ADR-0173.

## Notebook And Dashboard Controls

- Control 001: notebook-issued SQL carries `principal_id`, `tenant_id`, and `scope_version` under ADR-0008.
- Control 002: notebook-issued SQL cannot use workspace admin identity as tenant authority under ADR-0173.
- Control 003: notebook result download binds export residency under specs/tenant-model.json.
- Control 004: notebook scheduled jobs recheck current scope before execution under ADR-0105.
- Control 005: dashboard tiles bind query results to governed publication grants under ADR-0105.
- Control 006: dashboard cache keys include `scope_version` under ADR-0008.
- Control 007: dashboard sharing across tenants requires governed share under ADR-0314.
- Control 008: notebook cluster or warehouse start requires budget admission under ADR-0199.
- Control 009: lineage collection emits audit references under ADR-0003.
- Control 010: notebook and dashboard object names are redacted on denial under ADR-0008.

## Unity Catalog Replacement Controls

- Catalog 001: metastore ID becomes provider alias under ADR-0173.
- Catalog 002: catalog root binds to tenant scope under ADR-0002.
- Catalog 003: schema root binds to explicit grant under ADR-0105.
- Catalog 004: table root binds to cell placement under ADR-0009.
- Catalog 005: external location binds to residency pack under specs/tenant-model.json.
- Catalog 006: storage credential binds to credential-sidecar policy under ADR-0008.
- Catalog 007: row filter binds to normalized predicate under ADR-0008.
- Catalog 008: column mask binds to data-class policy under ADR-0008.
- Catalog 009: lineage output binds to audit chain under ADR-0003.
- Catalog 010: Delta Sharing output binds to DealSet when commercial under ADR-0314.
- Catalog 011: catalog grants cannot be inferred from names under ADR-0002.
- Catalog 012: group grants require identity principal mapping under ADR-0008.
- Catalog 013: table clone requires explicit snapshot grant under ADR-0105.
- Catalog 014: share recipient activation requires target tenant binding under ADR-0105.
- Catalog 015: provider-native privilege strings map to normalized actions under ADR-0008.

## SQL Warehouse Controls

- Warehouse 001: warehouse ID is alias only under ADR-0173.
- Warehouse 002: warehouse start requires cost center under ADR-0199.
- Warehouse 003: warehouse resize requires budget forecast under ADR-0199.
- Warehouse 004: serverless routing requires cell compatibility under ADR-0009.
- Warehouse 005: query admission requires policy evaluation before planner work under ADR-0008.
- Warehouse 006: result cache includes scope version under ADR-0008.
- Warehouse 007: query cancellation emits audit event under ADR-0003.
- Warehouse 008: pool exhaustion denial records cost and capacity reason under ADR-0199.
- Warehouse 009: pool promotion follows SLO-gated promotion contract under specs/masterplan.json.
- Warehouse 010: warehouse rollback suspends pool without revoking unrelated grants under ADR-0045.

## Implementation Steps

- Step 001: implement Databricks workspace alias import under ADR-0173.
- Step 002: implement SQL warehouse to workload pool mapper under ADR-0199.
- Step 003: implement metastore to catalog namespace mapper under ADR-0105.
- Step 004: implement catalog and schema ontology registration mapper under specs/platform-architecture.json.
- Step 005: implement table cell binding mapper under ADR-0009.
- Step 006: implement external location residency mapper under specs/tenant-model.json.
- Step 007: implement notebook query scope injector under ADR-0008.
- Step 008: implement dashboard publication grant mapper under ADR-0105.
- Step 009: implement Delta Sharing provider mapper under ADR-0314.
- Step 010: implement Delta Sharing recipient grant mapper under ADR-0105.
- Step 011: implement row filter mapper under ADR-0008.
- Step 012: implement column mask mapper under ADR-0008.
- Step 013: implement lineage audit pointer mapper under ADR-0003.
- Step 014: implement materialized view refresh scope recheck under ADR-0105.
- Step 015: implement scheduled dashboard budget admission under ADR-0199.
- Step 016: implement serverless SQL cell compatibility gate under ADR-0009.
- Step 017: implement denial redaction for workspace, catalog, schema, table, and dashboard names under ADR-0008.
- Step 018: implement dry-run report for Databricks SQL migration under specs/masterplan.json.
- Step 019: implement rollback for imported workspace aliases under ADR-0173.
- Step 020: implement scorecard row for Databricks SQL displacement readiness under specs/platform-architecture.json.

## Policy Requirements

- Policy 001: workspace identity is alias only under ADR-0173.
- Policy 002: notebook identity requires Oyatie principal binding under ADR-0008.
- Policy 003: SQL warehouse compute requires cost scope under ADR-0199.
- Policy 004: catalog access requires explicit grant under ADR-0105.
- Policy 005: external locations require residency envelope under specs/tenant-model.json.
- Policy 006: serverless SQL routing requires cell compatibility under ADR-0009.
- Policy 007: Delta Sharing requires governed share and DealSet binding where commercial under ADR-0314.
- Policy 008: row filters and masks cannot weaken tenant data class policy under ADR-0008.
- Policy 009: lineage emission cannot leak cross-tenant object names under ADR-0008.
- Policy 010: all activation, denial, and rollback actions emit audit evidence under ADR-0003.

## Observability Requirements

- Observability 001: emit `warehouse.databricks.workspace.alias_registered` under ADR-0173.
- Observability 002: emit `warehouse.databricks.sql_warehouse.cost_admitted` under ADR-0199.
- Observability 003: emit `warehouse.databricks.metastore.catalog_bound` under ADR-0105.
- Observability 004: emit `warehouse.databricks.table.cell_bound` under ADR-0009.
- Observability 005: emit `warehouse.databricks.external_location.residency_bound` under specs/tenant-model.json.
- Observability 006: emit `warehouse.databricks.notebook.query_scoped` under ADR-0008.
- Observability 007: emit `warehouse.databricks.dashboard.publication_bound` under ADR-0105.
- Observability 008: emit `warehouse.databricks.delta_share.dealset_bound` under ADR-0314.
- Observability 009: emit `warehouse.databricks.lineage.audit_bound` under ADR-0003.
- Observability 010: emit `warehouse.databricks.rollback.alias_detached` under ADR-0173.

## Test Plan

- Test 001: workspace alias cannot authorize query under ADR-0173.
- Test 002: SQL warehouse start fails without cost scope under ADR-0199.
- Test 003: metastore import fails without tenant binding under ADR-0002.
- Test 004: catalog access fails without explicit grant under ADR-0105.
- Test 005: table import fails without cell placement under ADR-0009.
- Test 006: external location import fails without residency envelope under specs/tenant-model.json.
- Test 007: notebook query fails without principal binding under ADR-0008.
- Test 008: dashboard publication fails without result publication grant under ADR-0105.
- Test 009: Delta Sharing commercial path fails without DealSet binding under ADR-0314.
- Test 010: row filter import preserves tenant predicate under ADR-0008.
- Test 011: column mask import preserves data class policy under ADR-0008.
- Test 012: lineage import emits audit reference under ADR-0003.
- Test 013: materialized view refresh rechecks current scope under ADR-0105.
- Test 014: serverless SQL route fails on incompatible cell under ADR-0009.
- Test 015: rollback detaches aliases without deleting tenant resources under ADR-0173.

## Risk Register

- Risk 001: workspace admins may be mistaken for tenant authorities; mitigation is Oyatie principal binding under ADR-0008.
- Risk 002: Unity Catalog grants may be imported without tenant predicates; mitigation is grant normalization under ADR-0105.
- Risk 003: notebook tokens may bypass SQL admission; mitigation is scope injection under ADR-0008.
- Risk 004: dashboard cache may reuse stale data; mitigation is scope-version binding under ADR-0008.
- Risk 005: external locations may point outside residency envelope; mitigation is residency validation under specs/tenant-model.json.
- Risk 006: Delta Sharing recipients may bypass settlement; mitigation is DealSet binding under ADR-0314.
- Risk 007: serverless SQL routes may cross cell boundary; mitigation is cell compatibility gate under ADR-0009.
- Risk 008: SQL warehouse auto-start may create unbudgeted spend; mitigation is cost admission under ADR-0199.
- Risk 009: lineage import may leak cross-tenant object names; mitigation is redacted lineage under ADR-0008.
- Risk 010: provider workspace IDs may become hidden authority; mitigation is alias-only import under ADR-0173.
- Risk 011: materialized view refresh may run after revoke; mitigation is current-scope recheck under ADR-0105.
- Risk 012: row filters and masks may weaken data class controls; mitigation is data-class mapper under ADR-0008.

## Evidence Artifacts

- Artifact 001: Databricks workspace alias report under ADR-0173.
- Artifact 002: SQL warehouse cost admission report under ADR-0199.
- Artifact 003: Unity Catalog grant normalization report under ADR-0105.
- Artifact 004: notebook and dashboard scope-injection report under ADR-0008.
- Artifact 005: external location residency report under specs/tenant-model.json.
- Artifact 006: Delta Sharing DealSet report under ADR-0314.
- Artifact 007: lineage audit report under ADR-0003.

## Acceptance Criteria

- Acceptance 001: Databricks workspace, SQL warehouse, Unity Catalog, Delta Sharing, notebook, dashboard, external location, row filter, column mask, and lineage surfaces have Oyatie replacements.
- Acceptance 002: every replacement carries citations for tenant, policy, cost, cell, residency, marketplace, or provider-independence controls.
- Acceptance 003: notebook and dashboard paths cannot bypass the tenant-scope kernel.
- Acceptance 004: Databricks SQL migration can run as dry-run with unsupported feature reporting.
- Acceptance 005: this IP remains inside data-warehouse IP write scope.

## Required Section Addendum

## Context
- Persona: Nisha Patel, Lakehouse Platform Owner, must migrate Databricks SQL warehouses, Unity Catalog objects, notebooks, dashboards, and Delta Sharing without importing workspace authority.
- Vendor surface subsumed: Databricks workspace, SQL warehouse, catalog, schema, table, external location, notebook, dashboard, Delta Sharing recipient, and job.
- The slice displaces Databricks SQL by binding every query and sharing path to Oyatie tenant scope before compute starts.

## Data Model Deltas
```sql
create table dw_databricks_sql_projection_imports (
    import_id uuid primary key,
    tenant_id uuid not null,
    workspace_id text not null,
    sql_warehouse_id text not null,
    unity_catalog_ref text not null,
    notebook_ref text,
    dashboard_ref text,
    delta_share_ref text,
    external_location_ref text,
    audit_event_class text not null
);
```
```rust
pub struct DatabricksSqlProjectionImport { pub import_id: Uuid, pub tenant_id: Uuid, pub workspace_id: String, pub sql_warehouse_id: String, pub unity_catalog_ref: String, pub notebook_ref: Option<String>, pub dashboard_ref: Option<String>, pub delta_share_ref: Option<String>, pub external_location_ref: Option<String>, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/migrations/databricks-sql/projections
{"tenant_id":"t_dw","workspace_id":"dbc-123","sql_warehouse_id":"wh-456","unity_catalog_ref":"main.finance.margin","delta_share_ref":"share_board_metrics","dry_run":true}
```
```yaml
grpc: {service: oyatie.data_warehouse.DatabricksSqlMigrationService, rpc: ProjectLakehouseSurface}
asyncapi: {publish: data-warehouse.databricks-sql.projected.v1, payload: {import_id: uuid, workspace_id: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"databricks-sql-project", resource)
when { context.tenant_id == resource.tenant_id && context.workspace_id != context.tenant_id && context.cost_center_id != "" };
forbid(principal, action, resource)
when { context.notebook_owner == "workspace-admin" && context.principal_binding == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Databricks workspace | `WarehouseProviderAlias` | workspace id remains alias |
| SQL warehouse | `WarehouseWorkloadPool` | warehouse size becomes capacity profile |
| Unity Catalog table | `WarehouseCatalogResource` | catalog path becomes resource ref |
| Delta Sharing recipient | `WarehouseGovernedShare` | recipient becomes target tenant grant |

## Workflow Steps
- Node `workspace-read`: load workspace and Unity Catalog metadata.
- Branch `workspace-admin-authority`: deny and require Oyatie principal mapping.
- Node `notebook-project`: convert notebooks to governed query templates.
- Branch `external-location-residency`: block if storage path violates pack.
- Node `share-project`: bind Delta Sharing recipients to DealSet-aware governed shares.

## Audit Events
- `DataWarehouseDatabricksWorkspaceRead`
- `DataWarehouseDatabricksSqlWarehouseProjected`
- `DataWarehouseDatabricksNotebookDenied`
- `DataWarehouseDatabricksExternalLocationBlocked`
- `DataWarehouseDatabricksDeltaShareBound`
- `DataWarehouseDatabricksProjectionActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| workspace projection | 100 ms | 650 ms | 1.2 s | 200 objects/s | 99.9% |
| notebook policy check | 45 ms | 220 ms | 430 ms | 500 checks/s | 99.95% |

## Failure Modes + Recovery
- `workspace-admin-overreach`: deny and create principal remediation task.
- `unity-catalog-path-collision`: quarantine path and require ontology rename.
- `external-location-pack-violation`: block activation and request residency approval.
- `delta-share-no-dealset`: hold share projection until marketplace binding exists.

## Migration Notes
- Databricks SQL warehouses become cost-admitted workload pools.
- Unity Catalog grants become normalized warehouse scope grants.
- Notebooks become governed query templates, never authority.
- Delta Sharing recipients become target tenant grants with settlement evidence.

## Cross-Microservice Handoffs
- ontology resolves Unity Catalog path collisions.
- policy-engine evaluates notebook and table actions.
- cost-ledger binds SQL warehouse spend.
- residency checks external locations.
- marketplace binds Delta Sharing commercial routes.
- audit-chain stores ADR-0263 migration events.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-028-databricks-sql-displacement-scope.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-028-databricks-sql-displacement-scope.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-028-databricks-sql-displacement-scope.md` matched `cost, emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
