---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-002-cedar-default-deny
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-policy
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0003, ADR-0007, ADR-0008, ADR-0009, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0314]
journey_ref: J-DW-002-cedar-default-deny
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/tenant-model.json, specs/platform-architecture.json]
write_scope: microservices/data-warehouse/IP-002-cedar-default-deny.md
---

# IP-002 Cedar Default Deny

## Goal

Define the Data Warehouse Cedar default-deny policy layer that sits between tenant-scope resolution and every warehouse action: query execution, metadata browsing, warehouse pool scaling, external data reads, exports, shares, marketplace consumption, cache reuse, scheduled jobs, and provider-alias migration.

## Displacement Thesis

Snowflake, BigQuery, Redshift, Databricks SQL, Synapse Analytics, Firebolt, and ClickHouse Cloud each provide access controls, but their native controls are anchored in provider-specific account, project, workspace, cluster, service, role, or catalog structures. Oyatie displaces those structures by making Cedar default-deny the portable policy root for every warehouse surface.

The policy layer must deny by default before SQL planning, before cache lookup, before external table resolution, before cost-bearing compute admission, before export materialization, and before share publication.

## Non Goals

- This IP does not define SQL grammar.
- This IP does not define query optimizer behavior.
- This IP does not edit content outside the Data Warehouse IP scope.
- This IP does not let provider-native IAM, RBAC, or catalog grants become final authority.

## Policy Decision Inputs

| Input | Required | Citation |
|---|---:|---|
| `tenant_id` | yes | ADR-0002 |
| `principal_id` | yes | ADR-0007 |
| `audience_type` | yes | ADR-0008 |
| `action` | yes | ADR-0105 |
| `resource_ref` | yes | ADR-0105 |
| `cell_id` | yes | ADR-0009 |
| `scope_version` | yes | ADR-0008 |
| `jurisdiction_pack_set` | yes | specs/tenant-model.json |
| `cost_center_id` | compute actions | ADR-0199 |
| `dealset_id` | marketplace actions | ADR-0314 |
| `provider_alias_ref` | migration and alias actions | ADR-0173 |

## Default-Deny Invariants

- Invariant 001: missing tenant context denies under ADR-0002.
- Invariant 002: missing principal context denies under ADR-0007.
- Invariant 003: missing audience type denies under ADR-0008.
- Invariant 004: missing resource reference denies under ADR-0105.
- Invariant 005: missing action mapping denies under ADR-0105.
- Invariant 006: missing cell binding denies storage, compute, and export actions under ADR-0009.
- Invariant 007: missing jurisdiction pack denies export and external data actions under specs/tenant-model.json.
- Invariant 008: missing cost center denies compute, scaling, scheduled refresh, and replay actions under ADR-0199.
- Invariant 009: missing DealSet binding denies commercial marketplace dataset consumption under ADR-0314.
- Invariant 010: provider alias references never authorize by themselves under ADR-0173.
- Invariant 011: cache reuse denies when `scope_version` differs from cached result under ADR-0008.
- Invariant 012: metadata browse denies cross-tenant object existence disclosure under ADR-0008.
- Invariant 013: external table access denies unless external dataset grant exists under ADR-0105.
- Invariant 014: governed share creation denies unless source and target tenant grants exist under ADR-0002.
- Invariant 015: rollback actions deny unless audit preservation is guaranteed under ADR-0003.

## Action Taxonomy

- Action 001: `warehouse.query.run` evaluates tenant, principal, grant, cost, cell, and cache scope under ADR-0008.
- Action 002: `warehouse.query.cancel` evaluates tenant and principal ownership under ADR-0007.
- Action 003: `warehouse.metadata.browse` evaluates redacted metadata access under ADR-0008.
- Action 004: `warehouse.namespace.create` evaluates tenant and cell placement under ADR-0009.
- Action 005: `warehouse.table.create` evaluates namespace grant and residency pack under ADR-0105.
- Action 006: `warehouse.external_table.create` evaluates external dataset grant under ADR-0105.
- Action 007: `warehouse.export.create` evaluates residency envelope under specs/tenant-model.json.
- Action 008: `warehouse.pool.start` evaluates cost scope under ADR-0199.
- Action 009: `warehouse.pool.resize` evaluates budget forecast under ADR-0199.
- Action 010: `warehouse.share.create` evaluates source grant, target grant, and DealSet under ADR-0314.
- Action 011: `warehouse.share.consume` evaluates target tenant grant and DealSet under ADR-0314.
- Action 012: `warehouse.cache.read` evaluates unchanged scope version under ADR-0008.
- Action 013: `warehouse.cache.write` evaluates tenant and policy result binding under ADR-0008.
- Action 014: `warehouse.job.schedule` evaluates principal, cost, and current-scope recheck mode under ADR-0105.
- Action 015: `warehouse.job.replay` evaluates current scope unless forensic-read-only under ADR-0003.
- Action 016: `warehouse.alias.register` evaluates provider-independence controls under ADR-0173.
- Action 017: `warehouse.alias.detach` evaluates rollback evidence under ADR-0173.
- Action 018: `warehouse.marketplace.consume` evaluates DealSet settlement under ADR-0314.
- Action 019: `warehouse.materialization.refresh` evaluates grants, cost, and scope version under ADR-0105.
- Action 020: `warehouse.backup.restore` evaluates residency, cell, and audit preservation under ADR-0003.

## Cedar Entity Model

- Entity 001: `WarehouseTenant` maps to `tenant_id` under ADR-0002.
- Entity 002: `WarehousePrincipal` maps to identity principal under ADR-0007.
- Entity 003: `WarehouseAudience` maps to audience type under ADR-0008.
- Entity 004: `WarehouseNamespace` maps database, schema, catalog, and provider alias namespaces under ADR-0105.
- Entity 005: `WarehouseTable` maps governed table resources under ADR-0105.
- Entity 006: `WarehouseExternalDataset` maps external table sources under specs/tenant-model.json.
- Entity 007: `WarehouseWorkloadPool` maps compute surfaces under ADR-0199.
- Entity 008: `WarehouseShare` maps governed data shares under ADR-0314.
- Entity 009: `WarehouseExportTarget` maps object storage exports under specs/tenant-model.json.
- Entity 010: `WarehouseProviderAlias` maps Snowflake, BigQuery, Redshift, Databricks, Synapse, Firebolt, and ClickHouse Cloud aliases under ADR-0173.
- Entity 011: `WarehouseScopeGrant` maps authorization edges under ADR-0105.
- Entity 012: `WarehouseCell` maps placement boundaries under ADR-0009.
- Entity 013: `WarehouseCostScope` maps budget and chargeback controls under ADR-0199.
- Entity 014: `WarehouseDealSet` maps marketplace settlement under ADR-0314.
- Entity 015: `WarehouseAuditEvent` maps policy evidence under ADR-0003.

## Provider-Specific Deny Cases

- Snowflake 001: deny if account locator is used as tenant authority under ADR-0173.
- Snowflake 002: deny if virtual warehouse resumes without cost scope under ADR-0199.
- Snowflake 003: deny if secure share lacks DealSet binding under ADR-0314.
- Snowflake 004: deny if stage export lacks residency envelope under specs/tenant-model.json.
- Snowflake 005: deny if role grant lacks Oyatie principal binding under ADR-0007.
- BigQuery 001: deny if project ID is used as tenant authority under ADR-0173.
- BigQuery 002: deny if job submission lacks budget admission under ADR-0199.
- BigQuery 003: deny if authorized view lacks explicit grant under ADR-0105.
- BigQuery 004: deny if export crosses residency envelope under specs/tenant-model.json.
- BigQuery 005: deny if cached result scope version is stale under ADR-0008.
- Redshift 001: deny if cluster ARN is used as tenant authority under ADR-0173.
- Redshift 002: deny if datashare lacks source and target grants under ADR-0105.
- Redshift 003: deny if Spectrum table lacks external dataset grant under ADR-0105.
- Redshift 004: deny if concurrency scaling lacks cost scope under ADR-0199.
- Redshift 005: deny if unload crosses residency envelope under specs/tenant-model.json.
- Databricks 001: deny if workspace ID is used as tenant authority under ADR-0173.
- Databricks 002: deny if notebook query lacks principal binding under ADR-0008.
- Databricks 003: deny if Delta Sharing lacks DealSet binding under ADR-0314.
- Databricks 004: deny if external location lacks residency envelope under specs/tenant-model.json.
- Databricks 005: deny if SQL warehouse starts without cost scope under ADR-0199.
- Synapse 001: deny if subscription ID is used as tenant authority under ADR-0173.
- Synapse 002: deny if linked service lacks credential policy binding under ADR-0008.
- Synapse 003: deny if serverless SQL query lacks cost admission under ADR-0199.
- Synapse 004: deny if CETAS crosses residency envelope under specs/tenant-model.json.
- Synapse 005: deny if pipeline retry skips current-scope recheck under ADR-0105.
- Firebolt 001: deny if account ID is used as tenant authority under ADR-0173.
- Firebolt 002: deny if engine activation lacks budget admission under ADR-0199.
- Firebolt 003: deny if external table lacks external dataset grant under ADR-0105.
- Firebolt 004: deny if aggregating index bypasses row or column policy under ADR-0008.
- Firebolt 005: deny if export lacks residency envelope under specs/tenant-model.json.
- ClickHouse 001: deny if organization ID is used as tenant authority under ADR-0173.
- ClickHouse 002: deny if service scaling lacks budget forecast under ADR-0199.
- ClickHouse 003: deny if dictionary access lacks external reference grant under ADR-0105.
- ClickHouse 004: deny if materialized view bypasses row or column policy under ADR-0008.
- ClickHouse 005: deny if backup lacks residency envelope under specs/tenant-model.json.

## Implementation Steps

- Step 001: define Cedar entity schema for warehouse tenants, principals, namespaces, resources, grants, cells, cost scopes, DealSets, and provider aliases under ADR-0007.
- Step 002: define normalized action enum for all warehouse commands under ADR-0105.
- Step 003: define default-deny policy bundle with no implicit allow path under ADR-0008.
- Step 004: define provider-alias deny rules for account, project, cluster, workspace, service, organization, and database identifiers under ADR-0173.
- Step 005: define cost-admission deny rules for query, pool, job, refresh, and replay actions under ADR-0199.
- Step 006: define cell-placement deny rules for namespace, storage, compute, backup, and failover actions under ADR-0009.
- Step 007: define residency deny rules for export, backup, external table, and linked service actions under specs/tenant-model.json.
- Step 008: define DealSet deny rules for commercial data share and marketplace consumption under ADR-0314.
- Step 009: define cache deny rules for stale `scope_version` under ADR-0008.
- Step 010: define metadata denial redaction output under ADR-0008.
- Step 011: define audit event emission for every allow and deny decision under ADR-0003.
- Step 012: define policy bundle versioning and rollback plan under ADR-0003.
- Step 013: define migration dry-run rule to fail closed on unknown provider privilege under ADR-0173.
- Step 014: define scheduled job current-scope recheck rule under ADR-0105.
- Step 015: define forensic-read-only replay exception with audit preservation under ADR-0003.
- Step 016: define materialization refresh policy with source grant recheck under ADR-0105.
- Step 017: define external dataset grant policy under ADR-0105.
- Step 018: define governed share source and target tenant policy under ADR-0002.
- Step 019: define principal audience policy for data platform operators under ADR-0008.
- Step 020: define policy scorecard evidence under specs/masterplan.json.

## Test Plan

- Test 001: missing tenant denies under ADR-0002.
- Test 002: missing principal denies under ADR-0007.
- Test 003: missing action mapping denies under ADR-0105.
- Test 004: provider alias alone denies under ADR-0173.
- Test 005: stale cache scope version denies under ADR-0008.
- Test 006: metadata browse denial redacts object existence under ADR-0008.
- Test 007: query run without cost scope denies under ADR-0199.
- Test 008: namespace create without cell binding denies under ADR-0009.
- Test 009: export without residency envelope denies under specs/tenant-model.json.
- Test 010: commercial share without DealSet denies under ADR-0314.
- Test 011: external table without grant denies under ADR-0105.
- Test 012: scheduled job replay after revoke denies under ADR-0105.
- Test 013: forensic replay without audit preservation denies under ADR-0003.
- Test 014: Snowflake account alias cannot authorize under ADR-0173.
- Test 015: BigQuery project alias cannot authorize under ADR-0173.
- Test 016: Redshift cluster alias cannot authorize under ADR-0173.
- Test 017: Databricks workspace alias cannot authorize under ADR-0173.
- Test 018: Synapse subscription alias cannot authorize under ADR-0173.
- Test 019: Firebolt account alias cannot authorize under ADR-0173.
- Test 020: ClickHouse organization alias cannot authorize under ADR-0173.

## Evidence Artifacts

- Artifact 001: Cedar schema snapshot for warehouse entities under ADR-0007.
- Artifact 002: Cedar policy bundle snapshot for default-deny rules under ADR-0008.
- Artifact 003: provider alias denial report under ADR-0173.
- Artifact 004: cost-admission denial report under ADR-0199.
- Artifact 005: cell-placement denial report under ADR-0009.
- Artifact 006: residency denial report under specs/tenant-model.json.
- Artifact 007: DealSet denial report under ADR-0314.
- Artifact 008: external dataset denial report under ADR-0105.
- Artifact 009: audit event report for allow and deny decisions under ADR-0003.
- Artifact 010: provider parity deny-case report for all seven displacement surfaces under specs/platform-architecture.json.

## Acceptance Criteria

- Acceptance 001: every warehouse action denies unless tenant, principal, action, resource, and policy context are complete.
- Acceptance 002: every compute action denies unless cost scope is present.
- Acceptance 003: every placement action denies unless cell binding is present.
- Acceptance 004: every export or backup action denies unless residency envelope is present.
- Acceptance 005: every commercial data consumption path denies unless DealSet binding is present.
- Acceptance 006: every provider alias denies as a standalone authority.
- Acceptance 007: every cache read denies on stale scope version.
- Acceptance 008: every metadata denial redacts cross-tenant object existence.
- Acceptance 009: every allow and deny emits audit evidence.
- Acceptance 010: every provider-displacement surface has explicit deny cases.
- Acceptance 011: every migration dry-run fails closed on unknown privilege.
- Acceptance 012: every rollback preserves policy and audit provenance.
- Acceptance 013: every test maps to a named citation.
- Acceptance 014: this IP remains inside data-warehouse IP write scope.

## Required Section Addendum

## Context
- Persona: Stefan Novak, data security lead, needs provider-native roles from Snowflake, BigQuery, Redshift, Databricks SQL, Synapse Analytics, Firebolt, ClickHouse Cloud, Vertica, Teradata Vantage, and Yellowbrick to fail closed until Cedar approves them.
- Vendor surface subsumed: provider IAM role, warehouse role, project owner, cluster admin, workspace admin, service admin, share recipient, and export writer.

## Data Model Deltas
```sql
create table dw_cedar_decision_ledger (
    decision_id uuid primary key,
    tenant_id uuid not null,
    principal_id uuid not null,
    warehouse_action text not null,
    resource_ref text not null,
    provider_alias_ref text,
    scope_version bigint not null,
    decision text not null check (decision in ('allow','deny')),
    deny_reason text,
    audit_event_class text not null,
    decided_at timestamptz not null default now()
);
```
```rust
pub struct WarehouseCedarDecision { pub decision_id: Uuid, pub tenant_id: Uuid, pub principal_id: Uuid, pub warehouse_action: WarehouseAction, pub resource_ref: String, pub provider_alias_ref: Option<String>, pub scope_version: i64, pub decision: CedarDecision, pub deny_reason: Option<String>, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/policy/decisions
{"tenant_id":"t_dw","principal_id":"p_analyst","action":"warehouse.query.run","resource_ref":"catalog.finance.margin","provider_alias_ref":"snowflake:xy123","scope_version":42}
```
```yaml
grpc: {service: oyatie.data_warehouse.PolicyDecisionService, rpc: EvaluateWarehouseAction}
asyncapi: {publish: data-warehouse.policy.decision.v1, payload: {decision_id: uuid, warehouse_action: string, decision: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"warehouse.query.run", resource)
when { context.tenant_id == resource.tenant_id && context.scope_version == resource.scope_version && context.cost_center_id != "" };
forbid(principal, action, resource)
when { context.provider_role in ["ACCOUNTADMIN", "Owner", "ClusterAdmin", "WorkspaceAdmin"] && context.oyatie_principal_binding == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Snowflake role | `WarehouseScopeGrant` | role name becomes source evidence |
| BigQuery IAM binding | `WarehousePrincipalBinding` | member becomes principal candidate |
| Redshift datashare grant | `WarehouseShareGrant` | producer/consumer become tenant refs |
| Databricks workspace admin | `WarehousePrincipalBinding` | admin flag becomes denied evidence |

## Workflow Steps
- Node `policy-context-build`: gather tenant, principal, action, resource, scope, cost, residency, and provider alias.
- Branch `provider-admin-authority`: deny and require Oyatie principal binding.
- Node `cedar-evaluate`: evaluate normalized warehouse action.
- Branch `deny`: persist denial and redact cross-tenant object existence.
- Node `allow`: bind decision id to planner, exporter, share, or job replay path.

## Audit Events
- `DataWarehousePolicyEvaluationRequested`
- `DataWarehousePolicyAllowed`
- `DataWarehousePolicyDenied`
- `DataWarehouseProviderAdminDenied`
- `DataWarehousePolicyRollbackBound`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| policy decision | 9 ms | 38 ms | 80 ms | 6,000 rps per cell | 99.99% |
| denial audit write | 14 ms | 75 ms | 150 ms | 2,000 eps per cell | 99.99% |

## Failure Modes + Recovery
- `provider-admin-overreach`: deny and require principal binding.
- `scope-version-stale`: deny cache/planner use and force re-evaluation.
- `cost-center-missing`: deny compute action and route to FinOps.
- `audit-chain-outage`: fail closed for mutations and allow redacted explain only.

## Migration Notes
- Provider roles from Snowflake, BigQuery, Redshift, Databricks SQL, Synapse Analytics, Firebolt, ClickHouse Cloud, Vertica, Teradata Vantage, and Yellowbrick become evidence only.
- Unknown provider privilege fails dry-run and creates policy review work.

## Cross-Microservice Handoffs
- policy-engine owns Cedar bundle compilation and evaluation.
- tenancy resolves principal membership.
- cost-ledger validates compute actions.
- residency validates exports and external tables.
- audit-chain records ADR-0263 allow and deny events.
- query-engine receives allowed decision ids before planning.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-002-cedar-default-deny.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-002-cedar-default-deny.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-002-cedar-default-deny.md` matched `cost, emission`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
