---
doc_class: IP
template_id: TPL-IP-Substance
ip_id: IP-030-firebolt-clickhouse-cloud-displacement-scope
microservice: data-warehouse
status: draft
owner_team: axis-data-platform + axis-observability
date: 2026-05-20
related_adrs: [ADR-0002, ADR-0003, ADR-0008, ADR-0009, ADR-0045, ADR-0105, ADR-0131, ADR-0173, ADR-0199, ADR-0314]
journey_ref: J-DW-030-firebolt-clickhouse-displacement
capability_profile: Tier-1
related_specs: [specs/masterplan.json, specs/platform-architecture.json, specs/tenant-model.json]
write_scope: microservices/data-warehouse/IP-030-firebolt-clickhouse-cloud-displacement-scope.md
---

# IP-030 Firebolt And ClickHouse Cloud Displacement Scope

## Goal

Define the Oyatie Data Warehouse surfaces that displace Firebolt and ClickHouse Cloud for high-concurrency analytical serving, low-latency dashboards, semi-structured ingestion, external object storage, materialized acceleration, and observability-grade query workloads.

## Displacement Summary

Firebolt is optimized for elastic analytical serving and fast query acceleration. ClickHouse Cloud is optimized for high-throughput analytical ingestion and low-latency columnar queries. Oyatie must compete by binding those performance surfaces to tenant-scope, cost admission, cell placement, residency, and audit evidence without adopting provider organization or service identity as authority.

This IP treats Firebolt engines and ClickHouse services as workload-pool aliases. It treats databases, tables, dictionaries, materialized views, and external storage as governed warehouse resources.

## Non Goals

- This IP does not implement ClickHouse SQL extensions or Firebolt SQL extensions.
- This IP does not define observability product dashboards outside the data-warehouse scope.
- This IP does not edit any content outside the Data Warehouse IP scope.
- This IP does not allow provider-native organization IDs to authorize actions.

## Surface Map

| Provider surface | Oyatie replacement | Citation |
|---|---|---|
| Firebolt account/database | tenant scope plus provider alias | ADR-0173 |
| Firebolt engine | workload pool with cost scope | ADR-0199 |
| Firebolt external table | external dataset grant | ADR-0105 |
| Firebolt aggregating index | governed acceleration policy | ADR-0008 |
| ClickHouse organization | tenant scope plus provider alias | ADR-0173 |
| ClickHouse service | workload pool with cell binding | ADR-0009 |
| ClickHouse database/table | catalog namespace and resource | ADR-0105 |
| ClickHouse dictionary | governed external reference | ADR-0105 |
| ClickHouse materialized view | governed materialization | ADR-0008 |
| ClickHouse backup/export | residency export target | specs/tenant-model.json |

## Product Requirements

- Requirement 001: Firebolt account ID is alias only under ADR-0173.
- Requirement 002: ClickHouse organization ID is alias only under ADR-0173.
- Requirement 003: Firebolt engine activation requires cost scope under ADR-0199.
- Requirement 004: ClickHouse service activation requires cost scope under ADR-0199.
- Requirement 005: Firebolt engine placement requires cell binding under ADR-0009.
- Requirement 006: ClickHouse service placement requires cell binding under ADR-0009.
- Requirement 007: Firebolt database import creates tenant namespace under ADR-0002.
- Requirement 008: ClickHouse database import creates tenant namespace under ADR-0002.
- Requirement 009: Firebolt external table requires external dataset grant under ADR-0105.
- Requirement 010: ClickHouse S3 table function and object storage access require external dataset grant under ADR-0105.
- Requirement 011: Firebolt aggregating index creation requires policy-scoped acceleration grant under ADR-0008.
- Requirement 012: ClickHouse materialized view creation requires policy-scoped acceleration grant under ADR-0008.
- Requirement 013: ClickHouse dictionary access requires external reference grant under ADR-0105.
- Requirement 014: Firebolt export requires residency envelope under specs/tenant-model.json.
- Requirement 015: ClickHouse backup and export require residency envelope under specs/tenant-model.json.
- Requirement 016: Firebolt result cache includes scope version under ADR-0008.
- Requirement 017: ClickHouse query cache includes scope version under ADR-0008.
- Requirement 018: commercial data serving requires DealSet binding under ADR-0314.
- Requirement 019: ingestion and query events emit audit evidence under ADR-0003.
- Requirement 020: rollback detaches provider aliases without deleting tenant resources under ADR-0173.

## Firebolt Controls

- Firebolt 001: account alias cannot authorize query under ADR-0173.
- Firebolt 002: database alias cannot authorize query under ADR-0173.
- Firebolt 003: engine start requires budget admission under ADR-0199.
- Firebolt 004: engine resize requires budget forecast under ADR-0199.
- Firebolt 005: engine placement requires cell compatibility under ADR-0009.
- Firebolt 006: external table access requires external dataset grant under ADR-0105.
- Firebolt 007: aggregating index creation requires scope-aware acceleration policy under ADR-0008.
- Firebolt 008: result cache includes `scope_version` under ADR-0008.
- Firebolt 009: export target requires residency pack under specs/tenant-model.json.
- Firebolt 010: data share requires DealSet when commercial under ADR-0314.
- Firebolt 011: query admission emits audit event under ADR-0003.
- Firebolt 012: engine exhaustion denial is redacted under ADR-0008.
- Firebolt 013: imported table names cannot imply grants under ADR-0002.
- Firebolt 014: dry-run migration cannot activate engines under specs/masterplan.json.
- Firebolt 015: rollback detaches aliases only under ADR-0173.

## ClickHouse Cloud Controls

- ClickHouse 001: organization alias cannot authorize query under ADR-0173.
- ClickHouse 002: service alias cannot authorize query under ADR-0173.
- ClickHouse 003: service start requires budget admission under ADR-0199.
- ClickHouse 004: service scale requires budget forecast under ADR-0199.
- ClickHouse 005: service placement requires cell compatibility under ADR-0009.
- ClickHouse 006: database access requires tenant namespace grant under ADR-0105.
- ClickHouse 007: table access requires principal binding under ADR-0008.
- ClickHouse 008: dictionary access requires external reference grant under ADR-0105.
- ClickHouse 009: materialized view creation requires scope-aware acceleration policy under ADR-0008.
- ClickHouse 010: backup and export require residency pack under specs/tenant-model.json.
- ClickHouse 011: query cache includes `scope_version` under ADR-0008.
- ClickHouse 012: commercial serving requires DealSet when marketplace data is used under ADR-0314.
- ClickHouse 013: ingestion and query admission emit audit events under ADR-0003.
- ClickHouse 014: dry-run migration cannot activate services under specs/masterplan.json.
- ClickHouse 015: rollback detaches aliases only under ADR-0173.

## Analytical Serving Requirements

- Serving 001: dashboard query admission requires principal, tenant, and cost scope under ADR-0008.
- Serving 002: high-concurrency admission uses workload pool tokens under ADR-0045.
- Serving 003: low-latency acceleration cannot bypass row or column policy under ADR-0008.
- Serving 004: precomputed aggregates carry source scope version under ADR-0008.
- Serving 005: observability-grade ingestion carries tenant and cell identifiers under ADR-0009.
- Serving 006: replayed ingestion rechecks current scope under ADR-0105.
- Serving 007: hot-cache eviction emits cost and scope metrics under ADR-0199.
- Serving 008: object storage reads require residency-compatible external dataset grant under specs/tenant-model.json.
- Serving 009: commercial dashboard data requires DealSet trace under ADR-0314.
- Serving 010: all denial paths redact cross-tenant object existence under ADR-0008.

## Implementation Steps

- Step 001: implement Firebolt account and database alias import under ADR-0173.
- Step 002: implement ClickHouse organization and service alias import under ADR-0173.
- Step 003: implement Firebolt engine to workload pool mapper under ADR-0199.
- Step 004: implement ClickHouse service to workload pool mapper under ADR-0199.
- Step 005: implement cell placement mapper for engines and services under ADR-0009.
- Step 006: implement Firebolt database namespace mapper under ADR-0002.
- Step 007: implement ClickHouse database namespace mapper under ADR-0002.
- Step 008: implement external table and S3 table function grant mapper under ADR-0105.
- Step 009: implement dictionary external reference grant mapper under ADR-0105.
- Step 010: implement aggregating index policy mapper under ADR-0008.
- Step 011: implement materialized view policy mapper under ADR-0008.
- Step 012: implement export and backup residency mapper under specs/tenant-model.json.
- Step 013: implement query cache scope-version binding under ADR-0008.
- Step 014: implement commercial serving DealSet mapper under ADR-0314.
- Step 015: implement ingestion audit event family under ADR-0003.
- Step 016: implement query admission audit event family under ADR-0003.
- Step 017: implement high-concurrency workload token admission under ADR-0045.
- Step 018: implement denial redaction under ADR-0008.
- Step 019: implement dry-run migration reports for Firebolt and ClickHouse Cloud under specs/masterplan.json.
- Step 020: implement rollback alias detachment under ADR-0173.

## Policy Requirements

- Policy 001: provider organization and account identifiers are aliases only under ADR-0173.
- Policy 002: engine and service activation require cost admission under ADR-0199.
- Policy 003: engine and service placement require cell binding under ADR-0009.
- Policy 004: database and table access require tenant and principal binding under ADR-0002.
- Policy 005: external object storage access requires external dataset grant under ADR-0105.
- Policy 006: acceleration structures cannot weaken row or column policy under ADR-0008.
- Policy 007: backup and export require residency envelope under specs/tenant-model.json.
- Policy 008: commercial serving requires DealSet binding under ADR-0314.
- Policy 009: cache reuse requires unchanged scope version under ADR-0008.
- Policy 010: every activation, ingestion, query, denial, and rollback emits audit evidence under ADR-0003.

## Observability Requirements

- Observability 001: emit `warehouse.firebolt.account.alias_registered` under ADR-0173.
- Observability 002: emit `warehouse.firebolt.engine.cost_admitted` under ADR-0199.
- Observability 003: emit `warehouse.firebolt.engine.cell_bound` under ADR-0009.
- Observability 004: emit `warehouse.firebolt.external_table.grant_bound` under ADR-0105.
- Observability 005: emit `warehouse.firebolt.export.residency_bound` under specs/tenant-model.json.
- Observability 006: emit `warehouse.clickhouse.organization.alias_registered` under ADR-0173.
- Observability 007: emit `warehouse.clickhouse.service.cost_admitted` under ADR-0199.
- Observability 008: emit `warehouse.clickhouse.service.cell_bound` under ADR-0009.
- Observability 009: emit `warehouse.clickhouse.dictionary.grant_bound` under ADR-0105.
- Observability 010: emit `warehouse.clickhouse.backup.residency_bound` under specs/tenant-model.json.
- Observability 011: emit `warehouse.serving.cache.scope_version_bound` under ADR-0008.
- Observability 012: emit `warehouse.serving.dealset_bound` under ADR-0314.
- Observability 013: emit `warehouse.serving.ingestion.audit_bound` under ADR-0003.
- Observability 014: emit `warehouse.serving.query.audit_bound` under ADR-0003.
- Observability 015: emit `warehouse.serving.rollback.alias_detached` under ADR-0173.

## Test Plan

- Test 001: Firebolt account alias cannot authorize query under ADR-0173.
- Test 002: ClickHouse organization alias cannot authorize query under ADR-0173.
- Test 003: Firebolt engine start fails without cost scope under ADR-0199.
- Test 004: ClickHouse service start fails without cost scope under ADR-0199.
- Test 005: Firebolt engine activation fails without cell binding under ADR-0009.
- Test 006: ClickHouse service activation fails without cell binding under ADR-0009.
- Test 007: Firebolt external table fails without grant under ADR-0105.
- Test 008: ClickHouse dictionary access fails without grant under ADR-0105.
- Test 009: Firebolt aggregating index cannot bypass policy under ADR-0008.
- Test 010: ClickHouse materialized view cannot bypass policy under ADR-0008.
- Test 011: Firebolt export fails without residency envelope under specs/tenant-model.json.
- Test 012: ClickHouse backup fails without residency envelope under specs/tenant-model.json.
- Test 013: commercial serving fails without DealSet binding under ADR-0314.
- Test 014: ingestion and query paths emit audit evidence under ADR-0003.
- Test 015: rollback detaches aliases without deleting tenant resources under ADR-0173.

## Acceptance Criteria

- Acceptance 001: Firebolt account, database, engine, external table, aggregating index, export, and serving surfaces have Oyatie replacements.
- Acceptance 002: ClickHouse Cloud organization, service, database, table, dictionary, materialized view, backup, export, and serving surfaces have Oyatie replacements.
- Acceptance 003: every replacement binds tenant, policy, cost, cell, residency, marketplace, and audit controls where relevant.
- Acceptance 004: provider identifiers remain aliases only.
- Acceptance 005: this IP remains inside data-warehouse IP write scope.
- Acceptance 006: high-concurrency serving cannot bypass workload-token admission under ADR-0045.
- Acceptance 007: low-latency acceleration cannot bypass scope-version cache controls under ADR-0008.

## Required Section Addendum

## Context
- Persona: Owen Hart, Real-Time Analytics Lead, must replace Firebolt engines and ClickHouse Cloud services while preserving low-latency dashboards under Oyatie policy.
- Vendor surface subsumed: Firebolt account, engine, database, table, aggregating index; ClickHouse Cloud organization, service, database, materialized view, dictionary, and backup.
- The slice exists because acceleration features must not bypass tenant scope, cost admission, row policy, or ADR-0263 evidence.

## Data Model Deltas
```sql
create table dw_realtime_acceleration_imports (
    import_id uuid primary key,
    tenant_id uuid not null,
    source_vendor text not null check (source_vendor in ('firebolt','clickhouse_cloud')),
    acceleration_surface text not null,
    engine_or_service_ref text not null,
    table_ref text not null,
    index_or_view_ref text,
    workload_token_ref text not null,
    scope_version bigint not null,
    audit_event_class text not null
);
```
```rust
pub struct RealtimeAccelerationImport { pub import_id: Uuid, pub tenant_id: Uuid, pub source_vendor: WarehouseVendor, pub acceleration_surface: String, pub engine_or_service_ref: String, pub table_ref: String, pub index_or_view_ref: Option<String>, pub workload_token_ref: String, pub scope_version: i64, pub audit_event_class: AuditEventClass }
```

## API Endpoints
```http
POST /v1/data-warehouse/migrations/realtime-acceleration/projections
{"tenant_id":"t_dw","source_vendor":"clickhouse_cloud","engine_or_service_ref":"svc-lowlatency","table_ref":"events.board_metric","index_or_view_ref":"mv_board_rollup","workload_token_ref":"wlt_123"}
```
```yaml
grpc: {service: oyatie.data_warehouse.RealtimeAccelerationMigrationService, rpc: ProjectAccelerationSurface}
asyncapi: {publish: data-warehouse.realtime-acceleration.projected.v1, payload: {import_id: uuid, source_vendor: string, audit_event_class: string}}
```

## Cedar Policy Hooks
```cedar
permit(principal, action == WarehouseAction::"acceleration-project", resource)
when { context.tenant_id == resource.tenant_id && context.workload_token_ref != "" && context.scope_version == resource.scope_version };
forbid(principal, action, resource)
when { context.low_latency_override == true && context.policy_bypass_reason == "" };
```

## Ontology Projection
| Vendor object | Oyatie object | Field delta |
|---|---|---|
| Firebolt engine | `WarehouseWorkloadPool` | engine family becomes acceleration profile |
| Firebolt aggregating index | `WarehouseMaterialization` | index becomes governed materialization |
| ClickHouse Cloud service | `WarehouseWorkloadPool` | service id becomes provider alias |
| ClickHouse materialized view | `WarehouseMaterialization` | view refresh gets scope-version key |

## Workflow Steps
- Node `acceleration-inventory`: collect engine, service, table, index, view, dictionary, and backup metadata.
- Branch `low-latency-bypass`: deny if policy context is absent.
- Node `token-admit`: bind workload token before acceleration activation.
- Branch `scope-version-stale`: invalidate materialized view projection.
- Node `activate`: create governed materialization and workload pool projection.

## Audit Events
- `DataWarehouseRealtimeAccelerationInventoried`
- `DataWarehouseFireboltEngineProjected`
- `DataWarehouseClickHouseServiceProjected`
- `DataWarehouseAccelerationPolicyBypassDenied`
- `DataWarehouseMaterializationScopeInvalidated`
- `DataWarehouseRealtimeAccelerationActivated`

## SLO Targets
| Path | p50 | p95 | p99 | Throughput | Availability |
|---|---:|---:|---:|---:|---:|
| acceleration admission | 18 ms | 75 ms | 150 ms | 1,500 checks/s | 99.95% |
| materialization invalidation | 25 ms | 140 ms | 280 ms | 600 invalidations/s | 99.9% |

## Failure Modes + Recovery
- `policy-bypass-for-speed`: deny and require workload token evidence.
- `scope-version-stale-materialization`: invalidate projection and rebuild after Cedar allow.
- `dictionary-cross-tenant-leak`: quarantine dictionary and require external grant.
- `backup-residency-gap`: block restore and attach residency evidence.

## Migration Notes
- Firebolt engines become workload pools with workload-token admission.
- Firebolt aggregating indexes become governed materializations.
- ClickHouse dictionaries become external dataset grants.
- ClickHouse materialized views include scope-version cache controls.

## Cross-Microservice Handoffs
- policy-engine evaluates low-latency and materialization actions.
- cost-ledger charges engine and service acceleration spend.
- residency validates backup and restore targets.
- ontology maps index and view names to governed resources.
- observability tracks tail latency and invalidation lag.
- audit-chain stores ADR-0263 acceleration events.

## Counterpart Lens
Firebolt and ClickHouse Cloud are the direct low-latency analytical serving targets, but the implementation must still fit the broader Snowflake, Google BigQuery, Databricks, and AWS Redshift displacement envelope. Oyatie should not create a separate analytical-serving authority path: serving indexes, query acceleration, and hot projections remain tenant-scoped Data Warehouse capabilities with Cedar, audit, cost, and residency evidence.

| Counterpart | Low-latency gap closed here |
|---|---|
| Snowflake | Query acceleration is governed by Oyatie warehouse/job admission instead of a vendor warehouse. |
| Google BigQuery | BI-style serving remains cost-admitted and policy-scoped. |
| Databricks | Lakehouse serving keeps table/catalog semantics instead of a separate product island. |
| AWS Redshift | Hot analytical serving uses cell-aware placement and signed audit references. |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/data-warehouse/IP-030-firebolt-clickhouse-cloud-displacement-scope.md` matched `asyncapi`; contract files `microservices/data-warehouse/contracts/openapi-v1.yaml, microservices/data-warehouse/contracts/asyncapi-v1.yaml, microservices/data-warehouse/contracts/data-warehouse-v1.proto`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-030-firebolt-clickhouse-cloud-displacement-scope.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-030-firebolt-clickhouse-cloud-displacement-scope.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
