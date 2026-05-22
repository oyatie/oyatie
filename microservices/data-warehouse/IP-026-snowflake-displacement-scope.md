---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-026-snowflake-displacement-scope
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-foundry
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0003, ADR-0008, ADR-0009, ADR-0045, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0314]
journey_ref: J-DW-026-snowflake-displacement
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/platform-architecture.json, specs/tenant-model.json]
write_scope: microservices/data-warehouse/IP-026-snowflake-displacement-scope.md
---

# IP-026 Snowflake Displacement Scope

## Goal

Define the Oyatie Data Warehouse surface that displaces Snowflake account, virtual warehouse, database, schema, share, task, stream, and marketplace patterns with tenant-scoped, cell-aware, policy-first primitives.

## Displacement Summary

Snowflake is strongest when independent teams can provision warehouses, isolate roles, share data, and pay for elastic compute. Oyatie must meet that operational convenience while making tenant scope, policy denial, cost attribution, and provider independence stronger than Snowflake-native account structure.

The target is not compatibility with Snowflake syntax. The target is to make a Snowflake migration plausible because operators can map accounts, warehouses, roles, shares, stages, tasks, streams, and marketplace listings into Oyatie primitives without losing governance evidence.

## Non Goals

- This IP does not implement a Snowflake SQL parser.
- This IP does not import Snowflake account identifiers as authority.
- This IP does not edit any content outside the Data Warehouse IP scope.
- This IP does not cover BigQuery, Redshift, Databricks SQL, Synapse, Firebolt, or ClickHouse beyond migration contrast.

## Snowflake Surface Map

| Snowflake surface | Oyatie replacement | Citation |
|---|---|---|
| Account | `WarehouseTenantScope` plus provider alias | ADR-0002 |
| Virtual warehouse | `WarehouseWorkloadPool` with cost scope | ADR-0199 |
| Database/schema | `WarehouseCatalogNamespace` with tenant grant | ADR-0105 |
| Role | `WarehousePrincipalBinding` with Cedar default-deny | ADR-0008 |
| Share | `WarehouseGovernedShare` with DealSet binding | ADR-0314 |
| Stage | `WarehouseResidencyExportTarget` | specs/tenant-model.json |
| Task | `WarehouseScheduledJob` with scope snapshot | ADR-0105 |
| Stream | `WarehouseChangeFeed` with audit offsets | ADR-0003 |

## Product Requirements

- Requirement 001: account migration stores Snowflake account locator as alias only under ADR-0173.
- Requirement 002: virtual warehouse migration creates workload pool with mandatory cost center under ADR-0199.
- Requirement 003: database migration creates tenant namespace with residency pack evidence under specs/tenant-model.json.
- Requirement 004: schema migration preserves grants as explicit `WarehouseScopeGrant` rows under ADR-0105.
- Requirement 005: role migration maps privileges to normalized actions under ADR-0008.
- Requirement 006: share migration requires DealSet binding before any consumer tenant access under ADR-0314.
- Requirement 007: stage migration validates object-store residency before export target activation under specs/tenant-model.json.
- Requirement 008: task migration stores scope snapshot and current-scope recheck mode under ADR-0105.
- Requirement 009: stream migration emits audit offset checkpoints under ADR-0003.
- Requirement 010: result cache migration refuses reuse when scope version changed under ADR-0008.
- Requirement 011: warehouse auto-resume requires budget admission under ADR-0199.
- Requirement 012: warehouse suspend preserves audit continuity under ADR-0003.
- Requirement 013: zero-copy clone is modeled as a governed snapshot with explicit tenant grant under ADR-0002.
- Requirement 014: time travel is modeled as retention-scoped snapshot access under specs/tenant-model.json.
- Requirement 015: masking policy migration binds to normalized data class under ADR-0008.
- Requirement 016: row access policy migration binds to tenant and principal context under ADR-0008.
- Requirement 017: external function migration requires provider alias declaration under ADR-0173.
- Requirement 018: marketplace listing migration requires DealSet settlement route under ADR-0314.
- Requirement 019: organization-level metadata is redacted across tenants under ADR-0008.
- Requirement 020: all migration evidence carries trace and audit identifiers under ADR-0003.

## Kernel Interfaces

- Interface 001: `snowflake.alias.register` records account, database, warehouse, role, and share aliases under ADR-0173.
- Interface 002: `snowflake.account.import` creates `WarehouseTenantScope` candidates under ADR-0002.
- Interface 003: `snowflake.warehouse.import` creates `WarehouseWorkloadPool` with budget guard under ADR-0199.
- Interface 004: `snowflake.database.import` creates catalog namespace with cell binding under ADR-0009.
- Interface 005: `snowflake.schema.import` creates namespace child and grant set under ADR-0105.
- Interface 006: `snowflake.role.import` creates normalized action grants under ADR-0008.
- Interface 007: `snowflake.share.import` creates governed share plus DealSet route under ADR-0314.
- Interface 008: `snowflake.stage.import` creates residency export target under specs/tenant-model.json.
- Interface 009: `snowflake.task.import` creates scheduled job and replay policy under ADR-0105.
- Interface 010: `snowflake.stream.import` creates change feed and checkpoint ledger under ADR-0003.
- Interface 011: `snowflake.masking.import` creates data-class policy attachment under ADR-0008.
- Interface 012: `snowflake.row_policy.import` creates row filter grant attachment under ADR-0008.
- Interface 013: `snowflake.clone.import` creates governed snapshot metadata under ADR-0002.
- Interface 014: `snowflake.time_travel.import` creates retention snapshot policy under specs/tenant-model.json.
- Interface 015: `snowflake.marketplace.import` creates data product settlement map under ADR-0314.

## Implementation Steps

- Step 001: create alias DTO for Snowflake account locators under ADR-0173.
- Step 002: create workload pool DTO for virtual warehouse sizes under ADR-0199.
- Step 003: create mapping from Snowflake warehouse state to Oyatie pool state under ADR-0045.
- Step 004: create namespace DTO for database and schema hierarchy under ADR-0105.
- Step 005: create grant normalizer from Snowflake privileges to Oyatie actions under ADR-0008.
- Step 006: create share importer that refuses consumer access without DealSet binding under ADR-0314.
- Step 007: create stage importer that attaches residency pack and object target under specs/tenant-model.json.
- Step 008: create task importer that stores schedule, scope snapshot, and replay mode under ADR-0105.
- Step 009: create stream importer that stores offset and audit pointer under ADR-0003.
- Step 010: create masking policy mapper to data classes under ADR-0008.
- Step 011: create row policy mapper to tenant-aware predicates under ADR-0008.
- Step 012: create clone mapper to governed snapshots under ADR-0002.
- Step 013: create time-travel mapper to retention snapshot policy under specs/tenant-model.json.
- Step 014: create marketplace mapper to DealSet settlement under ADR-0314.
- Step 015: create migration dry-run report with unsupported feature list under ADR-0173.
- Step 016: create rollback path that removes aliases without deleting imported Oyatie resources under ADR-0173.
- Step 017: create cost admission check before imported pool activation under ADR-0199.
- Step 018: create cell placement validation before imported namespace activation under ADR-0009.
- Step 019: create audit event family for import, activation, denial, and rollback under ADR-0003.
- Step 020: create parity scorecard for Snowflake displacement readiness under specs/masterplan.json.

## Policy Requirements

- Policy 001: no Snowflake role can become an Oyatie principal without identity binding under ADR-0002.
- Policy 002: no Snowflake share can be consumed without target tenant grant under ADR-0105.
- Policy 003: no imported virtual warehouse can run without cost admission under ADR-0199.
- Policy 004: no imported database can activate outside approved cell placement under ADR-0009.
- Policy 005: no imported stage can write outside residency envelope under specs/tenant-model.json.
- Policy 006: no imported masking policy can downgrade data class protection under ADR-0008.
- Policy 007: no imported task can replay under stale scope unless marked forensic-read-only under ADR-0003.
- Policy 008: no imported stream can expose offsets across tenant roots under ADR-0008.
- Policy 009: no marketplace import can bypass DealSet settlement under ADR-0314.
- Policy 010: no provider alias can satisfy authorization by itself under ADR-0173.

## Observability Requirements

- Observability 001: emit `warehouse.snowflake.import.started` under ADR-0003.
- Observability 002: emit `warehouse.snowflake.alias.registered` under ADR-0173.
- Observability 003: emit `warehouse.snowflake.pool.cost_admitted` under ADR-0199.
- Observability 004: emit `warehouse.snowflake.namespace.cell_bound` under ADR-0009.
- Observability 005: emit `warehouse.snowflake.grant.normalized` under ADR-0008.
- Observability 006: emit `warehouse.snowflake.share.dealset_bound` under ADR-0314.
- Observability 007: emit `warehouse.snowflake.stage.residency_bound` under specs/tenant-model.json.
- Observability 008: emit `warehouse.snowflake.task.scope_snapshotted` under ADR-0105.
- Observability 009: emit `warehouse.snowflake.stream.offset_bound` under ADR-0003.
- Observability 010: emit `warehouse.snowflake.import.rollback` under ADR-0173.

## Test Plan

- Test 001: account alias import cannot authorize a query by itself under ADR-0173.
- Test 002: virtual warehouse import fails without cost center under ADR-0199.
- Test 003: database import fails without cell binding under ADR-0009.
- Test 004: schema import preserves grant denial for missing principal under ADR-0008.
- Test 005: share import fails without DealSet binding under ADR-0314.
- Test 006: stage import fails outside residency pack under specs/tenant-model.json.
- Test 007: task import records scope snapshot under ADR-0105.
- Test 008: stream import records audit offset under ADR-0003.
- Test 009: masking policy import refuses downgrade under ADR-0008.
- Test 010: row policy import preserves tenant predicate under ADR-0002.
- Test 011: clone import requires explicit target grant under ADR-0105.
- Test 012: time travel import respects retention scope under specs/tenant-model.json.
- Test 013: marketplace listing import emits DealSet settlement route under ADR-0314.
- Test 014: dry-run report lists unsupported Snowflake features without activating resources under ADR-0173.
- Test 015: rollback removes aliases and preserves audit chain under ADR-0003.

## Risk Register

- Risk 001: Snowflake role inheritance may collapse multiple Oyatie principals into one provider role; mitigation is explicit principal expansion under ADR-0008.
- Risk 002: Snowflake account-level grants may look tenant-global; mitigation is alias-only import plus manual tenant binding under ADR-0173.
- Risk 003: Snowflake secure shares may hide consumer context; mitigation is target tenant grant plus DealSet binding under ADR-0314.
- Risk 004: Snowflake stages may point at object stores outside allowed residency; mitigation is residency envelope validation under specs/tenant-model.json.
- Risk 005: Snowflake tasks may replay after a grant revoke; mitigation is current-scope recheck under ADR-0105.
- Risk 006: Snowflake streams may expose historical rows after revocation; mitigation is audit-offset plus policy re-evaluation under ADR-0008.
- Risk 007: Snowflake result cache semantics may imply stale reuse; mitigation is scope-version cache key under ADR-0008.
- Risk 008: Snowflake marketplace listings may obscure settlement owner; mitigation is DealSet route mapping under ADR-0314.
- Risk 009: Snowflake warehouse auto-resume may create unbudgeted spend; mitigation is cost admission under ADR-0199.
- Risk 010: Snowflake region labels may not match Oyatie cell policy; mitigation is explicit cell binding under ADR-0009.
- Risk 011: Snowflake cloned objects may bypass source grants; mitigation is governed snapshot grant under ADR-0105.
- Risk 012: Snowflake time travel may exceed tenant retention policy; mitigation is retention-scoped snapshot access under specs/tenant-model.json.
- Risk 013: Snowflake external functions may call unapproved egress paths; mitigation is provider alias and policy binding under ADR-0173.
- Risk 014: Snowflake masking policy import may lose data-class semantics; mitigation is data-class mapper under ADR-0008.
- Risk 015: Snowflake row policy import may miss tenant predicates; mitigation is predicate fixture under ADR-0002.

## Rollback Requirements

- Rollback 001: detach Snowflake account aliases without deleting tenant namespaces under ADR-0173.
- Rollback 002: suspend imported workload pools before removing cost scopes under ADR-0199.
- Rollback 003: revoke governed shares before detaching DealSet routes under ADR-0314.
- Rollback 004: invalidate result caches after every grant rollback under ADR-0008.
- Rollback 005: preserve audit event pointers for import and rollback under ADR-0003.
- Rollback 006: preserve residency evidence for any export target previously activated under specs/tenant-model.json.
- Rollback 007: leave cell bindings intact until storage movement is separately approved under ADR-0009.
- Rollback 008: emit a dry-run rollback plan before live rollback under specs/masterplan.json.
- Rollback 009: refuse rollback if it would orphan a marketplace consumer grant under ADR-0314.
- Rollback 010: refuse rollback if it would erase provider alias provenance under ADR-0173.

## Evidence Artifacts

- Artifact 001: Snowflake alias import report with account, database, warehouse, role, and share mappings under ADR-0173.
- Artifact 002: Snowflake cost admission report for every imported virtual warehouse under ADR-0199.
- Artifact 003: Snowflake cell placement report for every imported namespace under ADR-0009.
- Artifact 004: Snowflake grant normalization report for every role and privilege under ADR-0008.
- Artifact 005: Snowflake share settlement report for every commercial share under ADR-0314.
- Artifact 006: Snowflake residency report for every stage and export under specs/tenant-model.json.
- Artifact 007: Snowflake audit report for every import, denial, activation, and rollback event under ADR-0003.

## Acceptance Criteria

- Acceptance 001: Snowflake account, warehouse, database, schema, role, share, stage, task, stream, clone, time travel, masking, and row policy surfaces have Oyatie replacements.
- Acceptance 002: every replacement names the governing tenant, policy, cost, cell, residency, marketplace, or provider-independence citation.
- Acceptance 003: provider identifiers remain aliases and never become authority.
- Acceptance 004: migration can run in dry-run mode without activating resources.
- Acceptance 005: import activation emits audit evidence for every accepted resource.
- Acceptance 006: rollback removes alias bindings without deleting unrelated resources.
- Acceptance 007: all acceptance evidence can be produced from files inside `microservices/data-warehouse/IP-*.md` scope.

## Required Section Addendum

## Context
- Persona: Marcus Lee, Data Platform Lead, is moving governed marts from Snowflake without letting account locators become tenant authority.
- Vendor surface subsumed: Snowflake account, warehouse, database, schema, role, secure share, stage, task, stream, clone, and marketplace listing.
- This slice exists because Snowflake convenience must be retained while Oyatie owns tenant scope, Cedar denial, cost admission, and ADR-0263 evidence.

## Data Model Deltas
```sql
create table dw_snowflake_projection_imports (
    import_id uuid primary key,
    tenant_id uuid not null,
    account_locator text not null,
    warehouse_name text not null,
    database_name text not null,
    schema_name text not null,
    source_role text not null,
    oyatie_resource_ref text not null,
    scope_version bigint not null,
    dealset_ref text,
    audit_event_class text not null
);
```
```rust
pub struct SnowflakeProjectionImport { pub import_id: Uuid, pub tenant_id: Uuid, pub account_locator: String, pub warehouse_name: String, pub database_name: String, pub schema_name: String, pub source_role: String, pub oyatie_resource_ref: String, pub scope_version: i64, pub dealset_ref: Option<String>, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/migrations/snowflake/imports
{"tenant_id":"t_dw","account_locator":"xy12345.us-east-1","warehouse_name":"FINANCE_WH","database_name":"FPNA","schema_name":"BOARD","source_role":"SYSADMIN","dry_run":true}
```
```yaml
grpc: {service: oyatie.data_warehouse.SnowflakeMigrationService, rpc: ImportSnowflakeProjection}
asyncapi: {publish: data-warehouse.snowflake.import.projected.v1, payload: {import_id: uuid, tenant_id: uuid, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"snowflake-import-project", resource)
when { context.tenant_id == resource.tenant_id && context.provider_alias_ref != "" && context.scope_version == resource.scope_version };
forbid(principal, action, resource)
when { context.account_locator == context.tenant_id || context.source_role in ["ACCOUNTADMIN", "SECURITYADMIN"] };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Snowflake account | `WarehouseProviderAlias` | account locator stored as alias only |
| Virtual warehouse | `WarehouseWorkloadPool` | size class plus cost center |
| Secure share | `WarehouseGovernedShare` | target tenant plus DealSet |
| Stage | `WarehouseResidencyExportTarget` | storage URL plus residency envelope |

## Workflow Steps
- Node `snowflake-discover`: collect account, role, warehouse, database, schema, share, stage, task, and stream metadata.
- Branch `role-is-authority`: deny and require Oyatie principal binding.
- Node `dry-run-project`: build resource refs without activation.
- Branch `residency-mismatch`: refuse stage activation and emit recovery packet.
- Node `activate`: create workload, catalog, share, and export projections after policy allow.

## Audit Events
- `DataWarehouseSnowflakeImportStarted`
- `DataWarehouseSnowflakeAliasProjected`
- `DataWarehouseSnowflakeRoleDenied`
- `DataWarehouseSnowflakeShareDealSetBound`
- `DataWarehouseSnowflakeImportActivated`
- `DataWarehouseSnowflakeRollbackBound`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| dry-run import | 90 ms | 500 ms | 900 ms | 250 objects/s | 99.9% |
| activation | 120 ms | 750 ms | 1.4 s | 120 objects/s | 99.9% |

## Failure Modes + Recovery
- `account-locator-authority`: deny, preserve alias, require tenant binding.
- `secure-share-missing-dealset`: quarantine share and request marketplace binding.
- `stage-residency-escape`: refuse export target and attach residency runbook.
- `task-replay-stale-scope`: mark task forensic-read-only until current-scope recheck passes.

## Migration Notes
- Snowflake warehouses become workload pools with cost admission.
- Snowflake tasks become scheduled jobs with scope snapshots.
- Snowflake streams become change feeds with audit offsets.
- Snowflake zero-copy clone becomes governed snapshot metadata.

## Cross-Microservice Handoffs
- tenancy validates tenant and scope version.
- policy-engine evaluates alias and grant actions.
- marketplace validates DealSet share settlement.
- residency validates stage and export target.
- cost-ledger attaches warehouse spend.
- audit-chain records ADR-0263 event classes.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-026-snowflake-displacement-scope.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-026-snowflake-displacement-scope.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-026-snowflake-displacement-scope.md` matched `cost, attribution`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
