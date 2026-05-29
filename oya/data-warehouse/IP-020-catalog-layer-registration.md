---
doc_class: IP
ip_id: IP-020
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-020: Catalog Layer Registration

## Context
- DW20-CTX-01: This IP registers warehouse datasets, views, shares, and workload pools in the catalog layer.
- DW20-CTX-02: Snowflake databases, schemas, tables, and secure shares map to local catalog objects.
- DW20-CTX-03: BigQuery projects, datasets, tables, routines, and Analytics Hub listings map to catalog entries.
- DW20-CTX-04: Redshift clusters, namespaces, schemas, tables, and datashares map to catalog entries.
- DW20-CTX-05: Databricks SQL catalogs, schemas, tables, and shares map through Unity-style metadata without inheriting vendor control.
- DW20-CTX-06: Synapse Analytics databases, external tables, and linked services become migration provenance and dataset objects.
- DW20-CTX-07: Firebolt databases, engines, and external tables map to datasets and workload pools.
- DW20-CTX-08: ClickHouse Cloud databases, tables, dictionaries, and services map to dataset and serving objects.
- DW20-CTX-09: Vertica schemas, projections, depots, and resource pools map to catalog and pool objects.
- DW20-CTX-10: Teradata Vantage databases, tables, views, and workload groups map to catalog objects.
- DW20-CTX-11: Yellowbrick databases, schemas, tables, and resource groups map to catalog entries.
- DW20-CTX-12: Registration is contract-first and rejects objects lacking tenant, data class, residency, and lineage.
- DW20-CTX-13: Catalog state is the read authority for SDKs, marketplace shares, and tenant admin.
- DW20-CTX-14: Catalog registration emits ADR-0263 audit classes for every material state change.
- DW20-CTX-15: Vendor object ids are stored as provenance, not as primary keys.

## Data Model Deltas
- DW20-DDL-01: Add catalog registration table.
```sql
CREATE TABLE warehouse_catalog_registrations (
    registration_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    catalog_object_id UUID NOT NULL,
    object_kind TEXT NOT NULL CHECK (object_kind IN ('database','schema','dataset','view','share','routine','workload_pool','external_location')),
    object_name TEXT NOT NULL,
    data_class TEXT NOT NULL,
    residency_overlay_id UUID,
    lineage_hash BYTEA NOT NULL,
    vendor_source TEXT,
    vendor_object_ref TEXT,
    registration_status TEXT NOT NULL CHECK (registration_status IN ('draft','active','quarantined','retired')),
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX wh_catalog_registration_object_idx ON warehouse_catalog_registrations(tenant_id, catalog_object_id);
```
- DW20-DDL-02: Add catalog field deltas.
```sql
CREATE TABLE warehouse_catalog_field_deltas (
    delta_id UUID PRIMARY KEY,
    registration_id UUID NOT NULL REFERENCES warehouse_catalog_registrations(registration_id) ON DELETE CASCADE,
    field_path TEXT NOT NULL,
    previous_value JSONB,
    next_value JSONB NOT NULL,
    change_reason TEXT NOT NULL,
    changed_by_principal_id UUID NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_catalog_field_delta_registration_idx ON warehouse_catalog_field_deltas(registration_id, changed_at DESC);
```
- DW20-RUST-01: Catalog registration type.
```rust
pub struct WarehouseCatalogRegistration {
    pub registration_id: CatalogRegistrationId,
    pub tenant_id: TenantId,
    pub catalog_object_id: CatalogObjectId,
    pub object_kind: WarehouseCatalogObjectKind,
    pub object_name: CatalogObjectName,
    pub data_class: DataClass,
    pub residency_overlay_id: Option<OverlayId>,
    pub lineage_hash: LineageHash,
    pub vendor_source: Option<WarehouseVendorSource>,
    pub vendor_object_ref: Option<VendorObjectRef>,
    pub registration_status: CatalogRegistrationStatus,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW20-RUST-02: Field delta type.
```rust
pub struct WarehouseCatalogFieldDelta {
    pub delta_id: CatalogFieldDeltaId,
    pub registration_id: CatalogRegistrationId,
    pub field_path: JsonPointer,
    pub previous_value: Option<serde_json::Value>,
    pub next_value: serde_json::Value,
    pub change_reason: CatalogChangeReason,
    pub changed_by_principal_id: PrincipalId,
    pub changed_at: DateTime<Utc>,
}
```
- DW20-RUST-03: Object kind is closed to prevent vendor-specific catalog object leaks.
- DW20-RUST-04: A catalog registration without lineage hash is invalid.
- DW20-RUST-05: Retired objects stay queryable as historical provenance.

## API Endpoints
- DW20-API-01: REST register endpoint.
```http
POST /v1/data-warehouse/catalog/registrations
Idempotency-Key: wh-catalog-register-020
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","object_kind":"dataset","object_name":"finance.monthly_revenue","data_class":"financial_aggregate","residency_overlay_id":"01JOVERLAY020","vendor_source":"redshift","vendor_object_ref":"cluster/prod/schema/finance/table/monthly_revenue","lineage_hash":"sha256:020"}
```
- DW20-API-02: REST retire endpoint.
```http
POST /v1/data-warehouse/catalog/registrations/{registration_id}:retire
Content-Type: application/json

{"retirement_reason":"vendor_cutover_complete","replacement_catalog_object_id":"01JCATALOG020B","effective_at":"2026-05-20T20:00:00Z"}
```
- DW20-API-03: gRPC register command.
```proto
rpc RegisterWarehouseCatalogObject(RegisterWarehouseCatalogObjectRequest) returns (RegisterWarehouseCatalogObjectResponse);
message RegisterWarehouseCatalogObjectRequest {
  string tenant_id = 1;
  string object_kind = 2;
  string object_name = 3;
  string data_class = 4;
  string lineage_hash = 5;
  string vendor_source = 6;
  string vendor_object_ref = 7;
}
```
- DW20-API-04: AsyncAPI event.
```yaml
warehouse.catalog.registration.activated.v1:
  payload:
    registration_id: 01JWH20REG
    catalog_object_id: 01JCATALOG020
    object_kind: dataset
    audit_event_class: WarehouseCatalogRegistrationActivated
```
- DW20-API-05: REST errors use `422 catalog_lineage_missing`.
- DW20-API-06: gRPC registration returns `ALREADY_EXISTS` when idempotency finds same lineage.
- DW20-API-07: Async retirement events include replacement object id where applicable.

## Cedar Policy Hooks
- DW20-CEDAR-01: principal = `Oyatie::Principal::"catalog_admin:{principal_id}"`.
- DW20-CEDAR-02: action = `Oyatie::Action::"warehouse_catalog_register"`.
- DW20-CEDAR-03: resource = `Oyatie::WarehouseCatalogObject::"{catalog_object_id}"`.
- DW20-CEDAR-04: context.tenant_id must equal resource tenant.
- DW20-CEDAR-05: context.data_class must be approved for the object kind.
- DW20-CEDAR-06: context.lineage_hash_present must be true.
- DW20-CEDAR-07: context.residency_overlay_required must imply overlay id present.
- DW20-CEDAR-08: context.vendor_source must be allowed only for migration provenance.
- DW20-CEDAR-09: context.audit_event_class must equal catalog registration class.
- DW20-CEDAR-10: deny if principal lacks `warehouse.catalog.write`.

## Ontology Projection
- DW20-ONTO-01: Snowflake `DATABASE.SCHEMA.TABLE` -> `WarehouseDataset.object_name`.
- DW20-ONTO-02: BigQuery `project.dataset.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-03: Redshift `database.schema.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-04: Databricks SQL `catalog.schema.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-05: Synapse `database.schema.external_table` -> `WarehouseExternalLocation.object_name`.
- DW20-ONTO-06: Firebolt `database.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-07: ClickHouse Cloud `database.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-08: Vertica `schema.projection` -> `WarehouseDataset.physical_projection_ref`.
- DW20-ONTO-09: Teradata Vantage `database.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-10: Yellowbrick `database.schema.table` -> `WarehouseDataset.object_name`.
- DW20-ONTO-11: Vendor share/listing id -> `WarehouseShare.vendor_share_ref`.
- DW20-ONTO-12: Vendor resource pool -> `WarehouseWorkloadPool.vendor_capacity_ref`.

## Workflow Steps
- DW20-WF-01: Node `ReceiveRegistration` validates object kind and name.
- DW20-WF-02: Node `ResolveDataClass` maps declared fields to catalog data class.
- DW20-WF-03: Node `CheckLineage` verifies lineage hash and source chain.
- DW20-WF-04: Branch `MissingResidencyOverlay` quarantines regulated objects.
- DW20-WF-05: Node `EvaluatePolicy` runs Cedar for registration action.
- DW20-WF-06: Branch `VendorObjectUntrusted` stores draft and requires review.
- DW20-WF-07: Node `PersistRegistration` writes active catalog registration.
- DW20-WF-08: Node `RecordFieldDeltas` stores field-level changes.
- DW20-WF-09: Node `EmitAudit` emits `WarehouseCatalogRegistrationActivated`.
- DW20-WF-10: Node `NotifyOntology` publishes projection update.
- DW20-WF-11: Node `NotifySdkCatalog` refreshes generated client schemas.
- DW20-WF-12: Node `RetireOldObject` closes replaced object without deleting history.

## Audit Events
- DW20-AUDIT-01: `WarehouseCatalogRegistrationRequested` records object kind and data class.
- DW20-AUDIT-02: `WarehouseCatalogRegistrationPolicyDenied` records Cedar decision.
- DW20-AUDIT-03: `WarehouseCatalogRegistrationQuarantined` records missing evidence.
- DW20-AUDIT-04: `WarehouseCatalogRegistrationActivated` records catalog object id.
- DW20-AUDIT-05: `WarehouseCatalogFieldDeltaRecorded` records changed field path.
- DW20-AUDIT-06: `WarehouseCatalogRegistrationRetired` records replacement id.
- DW20-AUDIT-07: `WarehouseCatalogOntologyHandoffQueued` records ontology event id.

## SLO Targets
- DW20-SLO-01: p50 registration <= 40 ms.
- DW20-SLO-02: p95 registration <= 180 ms.
- DW20-SLO-03: p99 registration <= 450 ms.
- DW20-SLO-04: throughput >= 1,000 registrations per second per cell.
- DW20-SLO-05: availability >= 99.95 percent for catalog write APIs.
- DW20-SLO-06: ontology handoff lag p95 <= 5 seconds.
- DW20-SLO-07: SDK catalog refresh p95 <= 30 seconds.
- DW20-SLO-08: lineage-missing false activation rate must be 0.

## Failure Modes + Recovery
- DW20-FAIL-01: Lineage hash missing; quarantine object and block query exposure.
- DW20-FAIL-02: Data class cannot be inferred; store draft and require catalog admin classification.
- DW20-FAIL-03: Ontology handoff fails; registration remains active and outbox retries with same event id.
- DW20-FAIL-04: Duplicate vendor object maps to different catalog object; quarantine both and require merge review.
- DW20-FAIL-05: Residency overlay expires; move object to quarantined until new overlay applies.
- DW20-FAIL-06: Field delta write fails; roll back registration transaction before activation.

## Migration Notes
- DW20-MIG-01: Snowflake secure shares require share name, database, schema, and role evidence.
- DW20-MIG-02: BigQuery routines and datasets require project id normalization.
- DW20-MIG-03: Redshift datashares require producer namespace and consumer namespace.
- DW20-MIG-04: Databricks SQL Unity-style metadata maps to local catalog object kinds.
- DW20-MIG-05: Synapse Analytics external tables require storage source references.
- DW20-MIG-06: Firebolt engines become workload pools, not datasets.
- DW20-MIG-07: ClickHouse Cloud dictionaries become routine or external location objects by use.
- DW20-MIG-08: Vertica projections are physical optimization refs, not primary datasets.
- DW20-MIG-09: Teradata Vantage views preserve lineage to base tables.
- DW20-MIG-10: Yellowbrick resource groups map to workload pools.

## Cross-Microservice Handoffs
- DW20-HANDOFF-01: Ontology receives catalog object and field delta projections.
- DW20-HANDOFF-02: SDK generation receives current contract and object shape.
- DW20-HANDOFF-03: Query planner receives active and quarantined object states.
- DW20-HANDOFF-04: Marketplace receives governed share catalog entries.
- DW20-HANDOFF-05: Audit-chain receives ADR-0263 catalog events.
- DW20-HANDOFF-06: Policy receives Cedar decision evidence and data-class context.
- DW20-HANDOFF-07: Tenant-admin receives catalog visibility and quarantine status.
- DW20-HANDOFF-08: Data-pipeline receives draft/quarantine remediation tasks.
- DW20-HANDOFF-09: Search receives active catalog metadata for discovery.
- DW20-HANDOFF-10: Compliance receives data-class and residency evidence.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-020-catalog-layer-registration.md` matched `p99, SLO, financial`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
