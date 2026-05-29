---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-029-synapse-analytics-displacement-scope
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-cloud
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0003, ADR-0008, ADR-0009, ADR-0045, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0314]
journey_ref: J-DW-029-synapse-analytics-displacement
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/platform-architecture.json, specs/tenant-model.json]
write_scope: microservices/data-warehouse/IP-029-synapse-analytics-displacement-scope.md
---

# IP-029 Synapse Analytics Displacement Scope

## Goal

Define the Oyatie Data Warehouse surface that displaces Synapse Analytics workspaces, dedicated SQL pools, serverless SQL pools, pipelines, linked services, external tables, CETAS exports, notebooks, and Fabric-adjacent consumption paths.

## Displacement Summary

Synapse combines Azure workspace governance, SQL pools, data lake external tables, pipelines, notebooks, and Microsoft analytics consumption. Oyatie must compete by removing Azure subscription and workspace authority from the trust root and making every pool, linked service, pipeline, export, and external table pass through Oyatie tenant scope.

This IP focuses on Synapse Analytics displacement while leaving broader Microsoft Fabric product strategy outside this slice.

## Non Goals

- This IP does not implement Azure control-plane automation.
- This IP does not treat Azure subscription, resource group, workspace, or managed identity as authority.
- This IP does not edit any content outside the Data Warehouse IP scope.
- This IP does not define Power BI product behavior except governed consumption boundaries.

## Surface Map

| Synapse surface | Oyatie replacement | Citation |
|---|---|---|
| Workspace | provider alias plus tenant scope | ADR-0173 |
| Dedicated SQL pool | workload pool with cell binding | ADR-0009 |
| Serverless SQL pool | workload pool with cost admission | ADR-0199 |
| Pipeline | scoped scheduled job | ADR-0105 |
| Linked service | provider alias plus credential binding | ADR-0173 |
| External table | external dataset grant | ADR-0105 |
| CETAS/export | residency export target | specs/tenant-model.json |
| Notebook | scoped query session | ADR-0008 |
| Data share | governed share plus DealSet | ADR-0314 |

## Product Requirements

- Requirement 001: workspace import records workspace ID and URL as alias only under ADR-0173.
- Requirement 002: subscription and resource group metadata cannot authorize warehouse actions under ADR-0173.
- Requirement 003: dedicated SQL pool activation requires cell placement under ADR-0009.
- Requirement 004: serverless SQL pool query requires cost admission under ADR-0199.
- Requirement 005: linked service import requires provider alias and credential policy binding under ADR-0173.
- Requirement 006: external table creation requires external dataset grant under ADR-0105.
- Requirement 007: CETAS and export paths require residency envelope under specs/tenant-model.json.
- Requirement 008: pipeline-triggered SQL requires current scope recheck under ADR-0105.
- Requirement 009: notebook-triggered SQL requires principal binding under ADR-0008.
- Requirement 010: managed identity is mapped to Oyatie principal binding before use under ADR-0008.
- Requirement 011: data share creation requires governed share under ADR-0314.
- Requirement 012: commercial data share consumption requires DealSet binding under ADR-0314.
- Requirement 013: materialized view refresh requires scope version check under ADR-0008.
- Requirement 014: result set cache includes scope version under ADR-0008.
- Requirement 015: lake database metadata access redacts cross-tenant names under ADR-0008.
- Requirement 016: pipeline lineage emits audit references under ADR-0003.
- Requirement 017: failed linked service access emits redacted denial evidence under ADR-0003.
- Requirement 018: pool scale operations bind budget forecast under ADR-0199.
- Requirement 019: pool failover preserves cell boundary under ADR-0009.
- Requirement 020: rollback detaches Azure aliases without deleting tenant resources under ADR-0173.

## Workspace Controls

- Workspace 001: Azure workspace name is alias only under ADR-0173.
- Workspace 002: Azure subscription ID is alias only under ADR-0173.
- Workspace 003: Azure resource group is alias only under ADR-0173.
- Workspace 004: managed private endpoint metadata requires cell binding under ADR-0009.
- Workspace 005: workspace admin role requires Oyatie principal binding under ADR-0008.
- Workspace 006: workspace default storage requires residency envelope under specs/tenant-model.json.
- Workspace 007: workspace data share requires governed share under ADR-0314.
- Workspace 008: workspace linked services require credential sidecar binding under ADR-0008.
- Workspace 009: workspace audit stream maps to audit chain under ADR-0003.
- Workspace 010: workspace rollback removes aliases only under ADR-0173.

## SQL Pool Controls

- Pool 001: dedicated SQL pool ID is alias only under ADR-0173.
- Pool 002: serverless SQL endpoint ID is alias only under ADR-0173.
- Pool 003: dedicated pool resume requires cost admission under ADR-0199.
- Pool 004: serverless query requires budget admission under ADR-0199.
- Pool 005: pool placement requires cell binding under ADR-0009.
- Pool 006: query admission requires policy evaluation before execution under ADR-0008.
- Pool 007: result cache includes tenant scope version under ADR-0008.
- Pool 008: pool scale emits audit event under ADR-0003.
- Pool 009: pool exhaustion returns redacted denial under ADR-0008.
- Pool 010: pool rollback suspends compute without deleting catalog resources under ADR-0045.

## Pipeline And Linked Service Controls

- Pipeline 001: pipeline trigger requires current tenant scope under ADR-0105.
- Pipeline 002: pipeline parameter cannot override tenant scope under ADR-0002.
- Pipeline 003: pipeline SQL activity binds principal and audience under ADR-0008.
- Pipeline 004: pipeline copy activity binds residency envelope under specs/tenant-model.json.
- Pipeline 005: pipeline retry rechecks scope version under ADR-0008.
- LinkedService 001: linked service URL is alias only under ADR-0173.
- LinkedService 002: linked service credential uses policy-bound credential reference under ADR-0008.
- LinkedService 003: linked service external data requires external dataset grant under ADR-0105.
- LinkedService 004: linked service failure emits audit event under ADR-0003.
- LinkedService 005: linked service commercial data requires DealSet binding under ADR-0314.

## Implementation Steps

- Step 001: implement Synapse workspace alias import under ADR-0173.
- Step 002: implement subscription and resource group alias capture under ADR-0173.
- Step 003: implement dedicated SQL pool mapper to workload pool under ADR-0009.
- Step 004: implement serverless SQL endpoint mapper to workload pool under ADR-0199.
- Step 005: implement pool scale budget admission under ADR-0199.
- Step 006: implement linked service alias mapper under ADR-0173.
- Step 007: implement linked service credential policy binding under ADR-0008.
- Step 008: implement external table dataset grant mapper under ADR-0105.
- Step 009: implement CETAS and export residency mapper under specs/tenant-model.json.
- Step 010: implement pipeline trigger scope recheck under ADR-0105.
- Step 011: implement notebook query scope injection under ADR-0008.
- Step 012: implement managed identity principal mapper under ADR-0008.
- Step 013: implement data share DealSet mapper under ADR-0314.
- Step 014: implement materialized view refresh scope check under ADR-0008.
- Step 015: implement audit event family for workspace, pool, pipeline, and linked service actions under ADR-0003.
- Step 016: implement metadata denial redaction under ADR-0008.
- Step 017: implement pool failover cell-boundary gate under ADR-0009.
- Step 018: implement dry-run report for Synapse migration under specs/masterplan.json.
- Step 019: implement rollback alias detachment under ADR-0173.
- Step 020: implement displacement scorecard row under specs/platform-architecture.json.

## Policy Requirements

- Policy 001: Azure subscription cannot authorize warehouse action under ADR-0173.
- Policy 002: Azure workspace cannot authorize warehouse action under ADR-0173.
- Policy 003: managed identity requires Oyatie principal binding under ADR-0008.
- Policy 004: pool execution requires cost admission under ADR-0199.
- Policy 005: pool placement requires cell binding under ADR-0009.
- Policy 006: external table access requires grant under ADR-0105.
- Policy 007: export and CETAS require residency envelope under specs/tenant-model.json.
- Policy 008: pipeline retry requires current scope version under ADR-0008.
- Policy 009: commercial sharing requires DealSet binding under ADR-0314.
- Policy 010: every activation and denial emits audit evidence under ADR-0003.

## Observability Requirements

- Observability 001: emit `warehouse.synapse.workspace.alias_registered` under ADR-0173.
- Observability 002: emit `warehouse.synapse.pool.cell_bound` under ADR-0009.
- Observability 003: emit `warehouse.synapse.pool.cost_admitted` under ADR-0199.
- Observability 004: emit `warehouse.synapse.linked_service.credential_bound` under ADR-0008.
- Observability 005: emit `warehouse.synapse.external_table.grant_bound` under ADR-0105.
- Observability 006: emit `warehouse.synapse.export.residency_bound` under specs/tenant-model.json.
- Observability 007: emit `warehouse.synapse.pipeline.scope_rechecked` under ADR-0105.
- Observability 008: emit `warehouse.synapse.notebook.query_scoped` under ADR-0008.
- Observability 009: emit `warehouse.synapse.share.dealset_bound` under ADR-0314.
- Observability 010: emit `warehouse.synapse.rollback.alias_detached` under ADR-0173.

## Test Plan

- Test 001: workspace alias cannot authorize query under ADR-0173.
- Test 002: subscription alias cannot authorize query under ADR-0173.
- Test 003: dedicated pool activation fails without cell binding under ADR-0009.
- Test 004: serverless query fails without cost scope under ADR-0199.
- Test 005: linked service access fails without credential binding under ADR-0008.
- Test 006: external table creation fails without grant under ADR-0105.
- Test 007: CETAS fails without residency envelope under specs/tenant-model.json.
- Test 008: pipeline retry rechecks current scope under ADR-0008.
- Test 009: managed identity fails without principal binding under ADR-0008.
- Test 010: commercial share fails without DealSet binding under ADR-0314.
- Test 011: metadata denial redacts object existence under ADR-0008.
- Test 012: pool scale emits audit event under ADR-0003.
- Test 013: pool failover rejects incompatible cell under ADR-0009.
- Test 014: dry-run migration does not activate resources under specs/masterplan.json.
- Test 015: rollback detaches aliases without deleting tenant resources under ADR-0173.

## Risk Register

- Risk 001: Azure subscription metadata may be mistaken for tenant authority; mitigation is alias-only import under ADR-0173.
- Risk 002: workspace admin grants may be mistaken for Oyatie authorization; mitigation is principal binding under ADR-0008.
- Risk 003: managed identity may bypass principal mapping; mitigation is identity binding under ADR-0008.
- Risk 004: dedicated SQL pool scale may ignore tenant budget; mitigation is cost admission under ADR-0199.
- Risk 005: serverless SQL may read cross-cell data lake objects; mitigation is cell binding under ADR-0009.
- Risk 006: linked services may point at unapproved data sources; mitigation is external dataset grant under ADR-0105.
- Risk 007: CETAS may export outside residency; mitigation is residency envelope under specs/tenant-model.json.
- Risk 008: pipeline retries may run after grant revoke; mitigation is scope-version recheck under ADR-0008.
- Risk 009: data shares may bypass settlement; mitigation is DealSet binding under ADR-0314.
- Risk 010: metadata browse may leak object existence; mitigation is redacted denial under ADR-0008.
- Risk 011: pool failover may cross cell boundary; mitigation is failover gate under ADR-0009.
- Risk 012: rollback may orphan linked-service aliases; mitigation is alias detachment report under ADR-0173.

## Evidence Artifacts

- Artifact 001: Synapse workspace and subscription alias report under ADR-0173.
- Artifact 002: SQL pool cost and cell binding report under ADR-0199.
- Artifact 003: linked service credential policy report under ADR-0008.
- Artifact 004: external table grant report under ADR-0105.
- Artifact 005: CETAS and export residency report under specs/tenant-model.json.
- Artifact 006: data share DealSet report under ADR-0314.
- Artifact 007: pipeline and pool audit report under ADR-0003.

## Acceptance Criteria

- Acceptance 001: Synapse workspace, SQL pool, pipeline, linked service, external table, CETAS, notebook, and data share surfaces have Oyatie replacements.
- Acceptance 002: Azure-native identifiers are represented only as aliases.
- Acceptance 003: every execution path binds tenant, policy, cost, cell, residency, and audit controls as applicable.
- Acceptance 004: commercial sharing has DealSet settlement binding.
- Acceptance 005: this IP remains inside data-warehouse IP write scope.

## Required Section Addendum

## Context
- Persona: Daniel Kim, Enterprise Data Architect, must migrate Synapse workspaces, dedicated pools, serverless SQL, pipelines, linked services, and CETAS exports without importing Azure subscription authority.
- Vendor surface subsumed: Synapse workspace, SQL pool, Spark pool, pipeline, trigger, linked service, credential, external table, and CETAS location.
- The slice exists because Azure resource hierarchy must remain provider alias evidence while Oyatie owns data-warehouse scope and auditability.

## Data Model Deltas
```sql
create table dw_synapse_projection_imports (
    import_id uuid primary key,
    tenant_id uuid not null,
    subscription_id text not null,
    workspace_name text not null,
    pool_name text not null,
    pipeline_name text,
    linked_service_name text,
    credential_ref text,
    ctas_target text,
    audit_event_class text not null
);
```
```rust
pub struct SynapseProjectionImport { pub import_id: Uuid, pub tenant_id: Uuid, pub subscription_id: String, pub workspace_name: String, pub pool_name: String, pub pipeline_name: Option<String>, pub linked_service_name: Option<String>, pub credential_ref: Option<String>, pub ctas_target: Option<String>, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/migrations/synapse/projections
{"tenant_id":"t_dw","subscription_id":"sub-123","workspace_name":"corp-synapse","pool_name":"dedicated-finance","pipeline_name":"nightly_margin","linked_service_name":"adls_finance"}
```
```yaml
grpc: {service: oyatie.data_warehouse.SynapseMigrationService, rpc: ProjectSynapseWorkspace}
asyncapi: {publish: data-warehouse.synapse.projected.v1, payload: {import_id: uuid, workspace_name: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"synapse-project", resource)
when { context.tenant_id == resource.tenant_id && context.subscription_id != context.tenant_id && context.credential_policy_ref != "" };
forbid(principal, action, resource)
when { context.linked_service_secret == "inline" || context.cetas_residency_status == "violating" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Synapse workspace | `WarehouseProviderAlias` | workspace and subscription stored as aliases |
| Dedicated SQL pool | `WarehouseWorkloadPool` | DWU profile becomes capacity profile |
| Linked service | `WarehouseExternalDataset` | credential ref becomes sidecar binding |
| CETAS target | `WarehouseResidencyExportTarget` | path becomes export target with pack evidence |

## Workflow Steps
- Node `workspace-inventory`: collect workspace, pool, pipeline, linked service, and credential metadata.
- Branch `subscription-as-tenant`: deny and require Oyatie tenant mapping.
- Node `credential-project`: bind linked service credentials to credential sidecar.
- Branch `cetas-residency-fail`: block target and emit recovery event.
- Node `pipeline-project`: convert pipeline retry into current-scope recheck workflow.

## Audit Events
- `DataWarehouseSynapseWorkspaceInventoried`
- `DataWarehouseSynapsePoolProjected`
- `DataWarehouseSynapseLinkedServiceBound`
- `DataWarehouseSynapseCetasBlocked`
- `DataWarehouseSynapsePipelineScopeSnapshotted`
- `DataWarehouseSynapseProjectionActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| workspace projection | 95 ms | 580 ms | 1.1 s | 180 objects/s | 99.9% |
| credential binding | 40 ms | 210 ms | 420 ms | 350 bindings/s | 99.95% |

## Failure Modes + Recovery
- `subscription-id-authority`: deny and preserve subscription as alias.
- `inline-linked-service-secret`: block projection and require sidecar migration.
- `cetas-cross-region`: reject export target and attach residency remediation.
- `pipeline-retry-stale-scope`: force current-scope recheck before replay.

## Migration Notes
- Synapse dedicated pools become workload pools with cost admission.
- Synapse pipelines become scheduled jobs with scope snapshots.
- Linked services become external dataset grants plus credential sidecar bindings.
- CETAS outputs become residency-validated exports.

## Cross-Microservice Handoffs
- credential-sidecar owns linked service secrets.
- residency owns CETAS target approval.
- workflow-engine owns pipeline replay state.
- cost-ledger owns pool and serverless spend.
- policy-engine normalizes Synapse actions.
- audit-chain records ADR-0263 events.

## Counterpart Lens
This Synapse Analytics slice sits behind the Big-8 data-warehouse envelope rather than outside it. The direct target is Azure Synapse, but acceptance is measured against the same Oyatie displacement bar used for Snowflake, Google BigQuery, Databricks, and AWS Redshift: external workspace identifiers are aliases only, query execution is tenant/Cedar scoped, and analytical sharing cannot bypass DealSet settlement.

| Counterpart | Synapse-specific gap closed here |
|---|---|
| Snowflake | Secure-share and warehouse authority are replaced by governed share plus tenant scope. |
| Google BigQuery | Serverless SQL-style access is admitted through Oyatie cost and policy controls. |
| Databricks | Lake/external table access keeps Unity-Catalog-class namespace pressure in scope. |
| AWS Redshift | Dedicated pool placement maps to cell-bound workload pools with audit evidence. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-029-synapse-analytics-displacement-scope.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-029-synapse-analytics-displacement-scope.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-029-synapse-analytics-displacement-scope.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
