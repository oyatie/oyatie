---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-027-bigquery-redshift-displacement-scope
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-cloud
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0003, ADR-0008, ADR-0009, ADR-0045, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0314]
journey_ref: J-DW-027-bigquery-redshift-displacement
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/platform-architecture.json, specs/tenant-model.json]
write_scope: microservices/data-warehouse/IP-027-bigquery-redshift-displacement-scope.md
---

# IP-027 BigQuery And Redshift Displacement Scope

## Goal

Define the Data Warehouse capability set that displaces BigQuery and Redshift for analytical teams that need serverless admission, provisioned capacity, external data, governed sharing, cost controls, residency controls, and SQL-operational evidence without cloud-project lock-in.

## Displacement Summary

BigQuery makes project-scoped serverless analytics easy. Redshift makes AWS-native warehouse clusters, workgroups, datashares, Spectrum, and concurrency scaling familiar. Oyatie must compete by separating tenant scope from cloud account structure and by making cost, cell, policy, and DealSet binding mandatory before query execution.

This IP binds BigQuery project/dataset/job concepts and Redshift cluster/workgroup/datashare concepts to one provider-independent warehouse contract.

## Non Goals

- This IP does not define Snowflake, Databricks SQL, Synapse, Firebolt, or ClickHouse detailed mappings.
- This IP does not edit any content outside the Data Warehouse IP scope.
- This IP does not authorize cloud billing account IDs as tenant IDs.
- This IP does not hand-wave cost controls as after-the-fact reporting.

## Surface Map

| Provider surface | Oyatie replacement | Citation |
|---|---|---|
| BigQuery project | `WarehouseTenantScope` plus alias | ADR-0173 |
| BigQuery dataset | `WarehouseCatalogNamespace` | ADR-0105 |
| BigQuery job | `WarehouseQueryRun` with budget admission | ADR-0199 |
| BigQuery reservation | `WarehouseWorkloadPool` | ADR-0045 |
| BigQuery authorized view | `WarehouseScopeGrant` | ADR-0008 |
| Redshift cluster | `WarehouseWorkloadPool` plus cell binding | ADR-0009 |
| Redshift workgroup | `WarehouseWorkloadPool` | ADR-0199 |
| Redshift datashare | `WarehouseGovernedShare` | ADR-0314 |
| Redshift Spectrum | `WarehouseExternalDatasetBinding` | specs/tenant-model.json |
| Redshift unload | `WarehouseResidencyExportTarget` | specs/tenant-model.json |

## Product Requirements

- Requirement 001: BigQuery project import records project ID as alias only under ADR-0173.
- Requirement 002: Redshift cluster import records cluster ARN as alias only under ADR-0173.
- Requirement 003: BigQuery dataset creation binds tenant, cell, and residency before activation under ADR-0009.
- Requirement 004: Redshift database creation binds tenant, cell, and residency before activation under ADR-0009.
- Requirement 005: BigQuery job submission requires cost admission before execution under ADR-0199.
- Requirement 006: Redshift query submission requires cost admission before execution under ADR-0199.
- Requirement 007: BigQuery reservations map to workload pools with budget caps under ADR-0045.
- Requirement 008: Redshift concurrency scaling maps to workload pool burst limits under ADR-0045.
- Requirement 009: BigQuery authorized views map to explicit grants under ADR-0105.
- Requirement 010: Redshift datashares map to governed shares with DealSet binding when commercial data is used under ADR-0314.
- Requirement 011: BigQuery exports bind residency before object write under specs/tenant-model.json.
- Requirement 012: Redshift unload operations bind residency before object write under specs/tenant-model.json.
- Requirement 013: BigQuery cached results include `scope_version` under ADR-0008.
- Requirement 014: Redshift result cache includes `scope_version` under ADR-0008.
- Requirement 015: BigQuery external tables require external dataset binding under ADR-0105.
- Requirement 016: Redshift Spectrum external tables require external dataset binding under ADR-0105.
- Requirement 017: BigQuery row access policies map to tenant-aware predicates under ADR-0008.
- Requirement 018: Redshift row-level security maps to tenant-aware predicates under ADR-0008.
- Requirement 019: all import, activation, denial, and rollback actions emit audit evidence under ADR-0003.
- Requirement 020: every provider billing metric maps into tenant cost scope under ADR-0199.

## BigQuery Requirements

- BigQuery 001: reject job submission without `tenant_id` under ADR-0002.
- BigQuery 002: reject job submission without `cost_center_id` under ADR-0199.
- BigQuery 003: reject dataset creation without `cell_id` under ADR-0009.
- BigQuery 004: reject export without residency target under specs/tenant-model.json.
- BigQuery 005: reject authorized view creation without explicit grant under ADR-0105.
- BigQuery 006: reject project-level IAM as sufficient authorization under ADR-0008.
- BigQuery 007: reject cached result reuse after grant revoke under ADR-0008.
- BigQuery 008: reject public dataset consumption without DealSet binding when commercial settlement applies under ADR-0314.
- BigQuery 009: record query plan admission as audit event under ADR-0003.
- BigQuery 010: record project and dataset identifiers as aliases under ADR-0173.
- BigQuery 011: normalize slot reservation to workload pool under ADR-0045.
- BigQuery 012: normalize BI Engine acceleration to cache policy under ADR-0008.
- BigQuery 013: normalize materialized views to governed materializations under ADR-0105.
- BigQuery 014: normalize transfer service jobs to async replay with current-scope check under ADR-0105.
- BigQuery 015: normalize data clean room usage to governed share under ADR-0314.

## Redshift Requirements

- Redshift 001: reject query submission without `tenant_id` under ADR-0002.
- Redshift 002: reject serverless workgroup activation without cost scope under ADR-0199.
- Redshift 003: reject cluster activation without cell placement under ADR-0009.
- Redshift 004: reject unload without residency target under specs/tenant-model.json.
- Redshift 005: reject datashare creation without explicit grant under ADR-0105.
- Redshift 006: reject IAM role as sufficient warehouse authorization under ADR-0008.
- Redshift 007: reject result cache reuse after grant revoke under ADR-0008.
- Redshift 008: reject marketplace data consumption without DealSet binding under ADR-0314.
- Redshift 009: record concurrency scaling admission as audit event under ADR-0003.
- Redshift 010: record cluster, namespace, and workgroup identifiers as aliases under ADR-0173.
- Redshift 011: normalize RA3 managed storage placement to cell binding under ADR-0009.
- Redshift 012: normalize Spectrum external table access to external dataset grant under ADR-0105.
- Redshift 013: normalize federated query access to provider alias and grant under ADR-0173.
- Redshift 014: normalize materialized views to governed materializations under ADR-0105.
- Redshift 015: normalize datashare consumer access to cross-tenant share contract under ADR-0314.

## Implementation Steps

- Step 001: implement provider alias DTO for BigQuery project and Redshift ARN under ADR-0173.
- Step 002: implement tenant scope import for project, dataset, cluster, namespace, and workgroup under ADR-0002.
- Step 003: implement cost admission mapper for BigQuery jobs and Redshift queries under ADR-0199.
- Step 004: implement workload pool mapper for reservations, slots, clusters, and workgroups under ADR-0045.
- Step 005: implement cell placement mapper for dataset and cluster placement under ADR-0009.
- Step 006: implement residency mapper for extract, export, unload, and external data paths under specs/tenant-model.json.
- Step 007: implement authorized view and datashare grant mapper under ADR-0105.
- Step 008: implement row access policy mapper for BigQuery and Redshift under ADR-0008.
- Step 009: implement external table mapper for BigLake, Spectrum, and object storage references under ADR-0105.
- Step 010: implement materialized view mapper to governed materialization under ADR-0105.
- Step 011: implement clean room and datashare DealSet mapper under ADR-0314.
- Step 012: implement cache invalidation on scope mutation under ADR-0008.
- Step 013: implement audit event family for import and query admission under ADR-0003.
- Step 014: implement dry-run report with parity gaps under specs/masterplan.json.
- Step 015: implement rollback that detaches aliases without deleting tenant resources under ADR-0173.
- Step 016: implement denial redaction for missing datasets and tables under ADR-0008.
- Step 017: implement budget overrun denial for slots and concurrency scaling under ADR-0199.
- Step 018: implement external data residency denial under specs/tenant-model.json.
- Step 019: implement migration fixture with one BigQuery dataset and one Redshift datashare under ADR-0314.
- Step 020: implement provider-independent scorecard for cloud warehouse displacement under specs/platform-architecture.json.

## Policy Requirements

- Policy 001: project ID and cluster ARN are aliases, not authority under ADR-0173.
- Policy 002: cloud IAM role is not sufficient without Oyatie principal binding under ADR-0008.
- Policy 003: dataset and namespace access require explicit grant under ADR-0105.
- Policy 004: query admission requires cost scope under ADR-0199.
- Policy 005: data placement requires cell binding under ADR-0009.
- Policy 006: export and unload require residency envelope under specs/tenant-model.json.
- Policy 007: cached result reuse requires unchanged scope version under ADR-0008.
- Policy 008: marketplace data consumption requires DealSet binding under ADR-0314.
- Policy 009: external table access requires external dataset grant under ADR-0105.
- Policy 010: every denial returns redacted resource facts under ADR-0008.

## Observability Requirements

- Observability 001: emit `warehouse.bigquery.project.alias_registered` under ADR-0173.
- Observability 002: emit `warehouse.bigquery.job.cost_admitted` under ADR-0199.
- Observability 003: emit `warehouse.bigquery.dataset.cell_bound` under ADR-0009.
- Observability 004: emit `warehouse.bigquery.export.residency_bound` under specs/tenant-model.json.
- Observability 005: emit `warehouse.bigquery.authorized_view.grant_bound` under ADR-0105.
- Observability 006: emit `warehouse.redshift.cluster.alias_registered` under ADR-0173.
- Observability 007: emit `warehouse.redshift.query.cost_admitted` under ADR-0199.
- Observability 008: emit `warehouse.redshift.namespace.cell_bound` under ADR-0009.
- Observability 009: emit `warehouse.redshift.unload.residency_bound` under specs/tenant-model.json.
- Observability 010: emit `warehouse.redshift.datashare.dealset_bound` under ADR-0314.

## Test Plan

- Test 001: BigQuery project alias cannot authorize query under ADR-0173.
- Test 002: Redshift cluster alias cannot authorize query under ADR-0173.
- Test 003: BigQuery job fails without cost scope under ADR-0199.
- Test 004: Redshift query fails without cost scope under ADR-0199.
- Test 005: BigQuery dataset fails without cell binding under ADR-0009.
- Test 006: Redshift namespace fails without cell binding under ADR-0009.
- Test 007: BigQuery export fails without residency envelope under specs/tenant-model.json.
- Test 008: Redshift unload fails without residency envelope under specs/tenant-model.json.
- Test 009: BigQuery authorized view fails without explicit grant under ADR-0105.
- Test 010: Redshift datashare fails without explicit grant under ADR-0105.
- Test 011: BigQuery public dataset consumption requires DealSet binding under ADR-0314.
- Test 012: Redshift marketplace datashare requires DealSet binding under ADR-0314.
- Test 013: BigQuery cache invalidates after grant revoke under ADR-0008.
- Test 014: Redshift cache invalidates after grant revoke under ADR-0008.
- Test 015: both providers emit import and admission audit events under ADR-0003.

## Risk Register

- Risk 001: BigQuery project-level IAM may be mistaken for Oyatie authorization; mitigation is principal binding under ADR-0008.
- Risk 002: Redshift IAM roles may be mistaken for Oyatie authorization; mitigation is principal binding under ADR-0008.
- Risk 003: BigQuery reservations may overrun tenant budget; mitigation is cost admission under ADR-0199.
- Risk 004: Redshift concurrency scaling may overrun tenant budget; mitigation is cost admission under ADR-0199.
- Risk 005: BigQuery regional datasets may conflict with cell policy; mitigation is cell binding under ADR-0009.
- Risk 006: Redshift cluster placement may conflict with cell policy; mitigation is cell binding under ADR-0009.
- Risk 007: BigQuery authorized views may leak source table existence; mitigation is redacted denial under ADR-0008.
- Risk 008: Redshift datashares may leak source namespace existence; mitigation is redacted denial under ADR-0008.
- Risk 009: BigQuery external tables may bypass residency; mitigation is external dataset grant plus residency envelope under specs/tenant-model.json.
- Risk 010: Redshift Spectrum may bypass residency; mitigation is external dataset grant plus residency envelope under specs/tenant-model.json.
- Risk 011: BigQuery public datasets may bypass settlement; mitigation is DealSet binding under ADR-0314.
- Risk 012: Redshift marketplace datashares may bypass settlement; mitigation is DealSet binding under ADR-0314.
- Risk 013: provider cache behavior may reuse stale results; mitigation is scope-version binding under ADR-0008.
- Risk 014: cloud billing labels may drift from tenant budget; mitigation is Oyatie cost scope under ADR-0199.
- Risk 015: rollback may orphan aliases; mitigation is alias detachment evidence under ADR-0173.

## Evidence Artifacts

- Artifact 001: BigQuery project and dataset alias report under ADR-0173.
- Artifact 002: Redshift cluster, namespace, and workgroup alias report under ADR-0173.
- Artifact 003: joint query cost admission report under ADR-0199.
- Artifact 004: joint cell placement report under ADR-0009.
- Artifact 005: joint grant normalization report under ADR-0105.
- Artifact 006: joint residency export report under specs/tenant-model.json.
- Artifact 007: joint DealSet settlement report under ADR-0314.
- Artifact 008: joint audit emission report under ADR-0003.

## Acceptance Criteria

- Acceptance 001: BigQuery and Redshift account-like identifiers are represented only as aliases.
- Acceptance 002: dataset, namespace, job, query, reservation, workgroup, datashare, external table, export, and unload paths have Oyatie replacements.
- Acceptance 003: every replacement binds tenant, policy, cost, cell, residency, and audit evidence where relevant.
- Acceptance 004: every commercial sharing path binds DealSet settlement.
- Acceptance 005: every test requirement has provider-specific assertions for BigQuery or Redshift.
- Acceptance 006: this IP remains within data-warehouse IP write scope.

## Required Section Addendum

## Context
- Persona: Elena Gomez, Analytics Engineering Manager, must move BigQuery projects and Redshift clusters into Oyatie without inheriting provider project or cluster authority.
- Vendor surface subsumed: BigQuery project, dataset, job, authorized view, reservation, transfer config; Redshift cluster, database, schema, datashare, Spectrum table, unload path.
- This slice preserves migration affordances while making tenant scope, policy, cost, residency, and audit evidence stronger than provider-native IAM.

## Data Model Deltas
```sql
create table dw_bq_redshift_projection_imports (
    import_id uuid primary key,
    tenant_id uuid not null,
    source_vendor text not null check (source_vendor in ('bigquery','redshift')),
    provider_container text not null,
    dataset_or_cluster text not null,
    object_ref text not null,
    normalized_action text not null,
    cost_center_id uuid not null,
    residency_boundary_id uuid not null,
    audit_event_class text not null
);
```
```rust
pub struct BqRedshiftProjectionImport { pub import_id: Uuid, pub tenant_id: Uuid, pub source_vendor: WarehouseVendor, pub provider_container: String, pub dataset_or_cluster: String, pub object_ref: String, pub normalized_action: WarehouseAction, pub cost_center_id: Uuid, pub residency_boundary_id: Uuid, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/migrations/bigquery-redshift/projections
{"tenant_id":"t_dw","source_vendor":"bigquery","provider_container":"bq-prj-finance","dataset_or_cluster":"finance_dw","object_ref":"views.board_margin","normalized_action":"warehouse.query.run"}
```
```yaml
grpc: {service: oyatie.data_warehouse.BqRedshiftMigrationService, rpc: ProjectProviderObject}
asyncapi: {publish: data-warehouse.bq-redshift.object.projected.v1, payload: {import_id: uuid, source_vendor: string, normalized_action: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"provider-object-project", resource)
when { context.tenant_id == resource.tenant_id && context.cost_center_id != "" && context.residency_boundary_id != "" };
forbid(principal, action, resource)
when { context.provider_container == context.tenant_id || context.provider_iam_role == "owner" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| BigQuery project | `WarehouseProviderAlias` | project id stored as alias |
| BigQuery reservation | `WarehouseWorkloadPool` | slot commitment becomes capacity token |
| Redshift cluster | `WarehouseWorkloadPool` | cluster ARN becomes provider alias |
| Redshift datashare | `WarehouseGovernedShare` | producer/consumer grants become DealSet-aware scope grants |

## Workflow Steps
- Node `provider-export-read`: ingest BigQuery and Redshift metadata snapshots.
- Branch `provider-container-as-tenant`: deny and require explicit tenant mapping.
- Node `cost-project`: bind reservation, slot, concurrency-scaling, or WLM spend to cost center.
- Branch `residency-gap`: refuse export, unload, or transfer config activation.
- Node `projection-activate`: create catalog, pool, share, and job projections.

## Audit Events
- `DataWarehouseBigQueryProjectionStarted`
- `DataWarehouseRedshiftProjectionStarted`
- `DataWarehouseProviderContainerDenied`
- `DataWarehouseProviderCostScopeBound`
- `DataWarehouseProviderResidencyBound`
- `DataWarehouseProviderProjectionActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| metadata projection | 75 ms | 420 ms | 850 ms | 400 objects/s | 99.9% |
| cost binding | 35 ms | 160 ms | 300 ms | 700 bindings/s | 99.95% |

## Failure Modes + Recovery
- `bigquery-owner-import`: deny owner role as authority and map only named grants.
- `redshift-cluster-tenant-collapse`: quarantine cluster alias and require tenant split.
- `reservation-without-cost-center`: refuse query-job activation until cost center exists.
- `unload-residency-escape`: block export and attach residency remediation.

## Migration Notes
- BigQuery jobs become warehouse scheduled or ad hoc query runs with budget admission.
- BigQuery authorized views become explicit scope grants.
- Redshift datashares become governed shares with source and target grants.
- Redshift Spectrum tables become external dataset grants.

## Cross-Microservice Handoffs
- tenancy owns tenant alias mapping.
- policy-engine normalizes provider IAM to warehouse actions.
- cost-ledger owns reservation and concurrency spend.
- residency owns export and unload envelopes.
- marketplace owns commercial datashare settlement.
- audit-chain owns ADR-0263 projection evidence.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-027-bigquery-redshift-displacement-scope.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-027-bigquery-redshift-displacement-scope.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-027-bigquery-redshift-displacement-scope.md` matched `cost, emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
