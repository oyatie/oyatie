---
doc_class: IP
ip_id: IP-015
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0251, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0321]
journey_ref: J89-uk-aadc-minor-ux-adaptation
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-015: Data Residency Pack Overlays

## Context
- DW15-CTX-01: This IP turns data residency from vendor region flags into Oyatie pack-governed query, storage, export, and share controls.
- DW15-CTX-02: Snowflake account regions map to explicit `WarehouseResidencyOverlay` rows.
- DW15-CTX-03: BigQuery dataset locations map to pack evidence and query execution constraints.
- DW15-CTX-04: Redshift RA3 namespace regions map to cell placement requirements.
- DW15-CTX-05: Databricks SQL workspace regions map to warehouse endpoint residency.
- DW15-CTX-06: Synapse Analytics workspace regions map to import/export policy controls.
- DW15-CTX-07: Firebolt engine regions map to workload-pool placement constraints.
- DW15-CTX-08: ClickHouse Cloud service regions map to dataset and replica location guarantees.
- DW15-CTX-09: Vertica Eon communal storage region maps to storage plane residency.
- DW15-CTX-10: Teradata Vantage regions map to account-level migration overlays.
- DW15-CTX-11: Yellowbrick cluster placement maps to fixed compute cell residency.
- DW15-CTX-12: Residency overlays are evaluated before every query, share, export, and backfill replay.
- DW15-CTX-13: Overlay changes require workflow approval when they broaden region, purpose, or actor class.
- DW15-CTX-14: Policy must distinguish storage residency, compute residency, and egress residency.
- DW15-CTX-15: Tenant admins see effective overlays, not vendor-specific region vocabulary.

## Data Model Deltas
- DW15-DDL-01: Add overlay table.
```sql
CREATE TABLE warehouse_residency_overlays (
    overlay_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    pack_code TEXT NOT NULL,
    dataset_id UUID,
    workload_pool_id UUID,
    allowed_storage_regions TEXT[] NOT NULL,
    allowed_compute_regions TEXT[] NOT NULL,
    allowed_export_regions TEXT[] NOT NULL,
    denied_vendor_regions TEXT[] NOT NULL DEFAULT '{}',
    effective_from TIMESTAMPTZ NOT NULL,
    effective_until TIMESTAMPTZ,
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (dataset_id IS NOT NULL OR workload_pool_id IS NOT NULL)
);
CREATE INDEX wh_residency_overlay_tenant_pack_idx ON warehouse_residency_overlays(tenant_id, pack_code, effective_from DESC);
```
- DW15-DDL-02: Add residency evaluation cache for hot query paths.
```sql
CREATE TABLE warehouse_residency_decisions (
    decision_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    overlay_id UUID NOT NULL REFERENCES warehouse_residency_overlays(overlay_id),
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('query','share','export','backfill','catalog_projection')),
    subject_id UUID NOT NULL,
    requested_region TEXT NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('allow','deny','shadow')),
    denial_reason TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_residency_decisions_hot_idx ON warehouse_residency_decisions(tenant_id, subject_kind, subject_id, expires_at);
```
- DW15-RUST-01: Residency overlay domain type.
```rust
pub struct WarehouseResidencyOverlay {
    pub overlay_id: OverlayId,
    pub tenant_id: TenantId,
    pub pack_code: ResidencyPackCode,
    pub dataset_id: Option<DatasetId>,
    pub workload_pool_id: Option<WorkloadPoolId>,
    pub allowed_storage_regions: BTreeSet<RegionCode>,
    pub allowed_compute_regions: BTreeSet<RegionCode>,
    pub allowed_export_regions: BTreeSet<RegionCode>,
    pub denied_vendor_regions: BTreeSet<VendorRegionCode>,
    pub effective_window: TimeWindow,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW15-RUST-02: Residency decision type.
```rust
pub struct WarehouseResidencyDecision {
    pub decision_id: ResidencyDecisionId,
    pub tenant_id: TenantId,
    pub overlay_id: OverlayId,
    pub subject: ResidencySubject,
    pub requested_region: RegionCode,
    pub decision: ResidencyDecisionKind,
    pub denial_reason: Option<ResidencyDenialReason>,
    pub expires_at: DateTime<Utc>,
}
```
- DW15-RUST-03: `ResidencySubject` variants are `Query`, `Share`, `Export`, `Backfill`, and `CatalogProjection`.
- DW15-RUST-04: Region codes are canonical Oyatie region ids, with vendor region ids retained only as evidence.
- DW15-RUST-05: Effective windows cannot overlap for the same dataset and pack unless one is shadow-only.

## API Endpoints
- DW15-API-01: REST overlay creation.
```http
POST /v1/data-warehouse/residency/overlays
Idempotency-Key: wh-residency-overlay-015
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","pack_code":"kr-pipa-enterprise","dataset_id":"01JDATASET015","allowed_storage_regions":["kr-seoul-1"],"allowed_compute_regions":["kr-seoul-1"],"allowed_export_regions":["kr-seoul-1","jp-tokyo-1"],"denied_vendor_regions":["aws-us-east-1","gcp-us"]}
```
- DW15-API-02: REST decision evaluation.
```http
POST /v1/data-warehouse/residency:decide
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","subject_kind":"query","subject_id":"01JQUERY015","dataset_id":"01JDATASET015","requested_region":"kr-seoul-1","purpose":"finance_close"}
```
- DW15-API-03: gRPC command.
```proto
rpc EvaluateWarehouseResidency(EvaluateWarehouseResidencyRequest) returns (EvaluateWarehouseResidencyResponse);
message EvaluateWarehouseResidencyRequest {
  string tenant_id = 1;
  string subject_kind = 2;
  string subject_id = 3;
  string requested_region = 4;
  string pack_code = 5;
}
```
- DW15-API-04: AsyncAPI event.
```yaml
warehouse.residency.overlay.applied.v1:
  payload:
    overlay_id: 01JWH15OVERLAY
    pack_code: kr-pipa-enterprise
    audit_event_class: WarehouseResidencyOverlayApplied
```
- DW15-API-05: REST errors use `403 residency_region_denied`.
- DW15-API-06: gRPC denial returns `FAILED_PRECONDITION` when overlay is absent.
- DW15-API-07: Async decisions include `requested_region` and `decision`, not copied row data.

## Cedar Policy Hooks
- DW15-CEDAR-01: principal = `Oyatie::Principal::"data_platform_operator:{principal_id}"`.
- DW15-CEDAR-02: action = `Oyatie::Action::"warehouse_residency_overlay_apply"`.
- DW15-CEDAR-03: resource = `Oyatie::WarehouseDataset::"{dataset_id}"`.
- DW15-CEDAR-04: context.pack_code must match tenant active compliance pack.
- DW15-CEDAR-05: context.requested_region must be in resource.allowed_storage_regions for storage mutations.
- DW15-CEDAR-06: context.compute_region must be in overlay.allowed_compute_regions for query planning.
- DW15-CEDAR-07: context.export_region must be in overlay.allowed_export_regions for exports.
- DW15-CEDAR-08: context.vendor_region must not be in overlay.denied_vendor_regions.
- DW15-CEDAR-09: context.audit_event_class must equal `WarehouseResidencyDecisionRecorded` for decisions.
- DW15-CEDAR-10: deny if principal lacks `warehouse.residency.admin` for overlay mutation.

## Ontology Projection
- DW15-ONTO-01: Snowflake `ACCOUNT.REGION` -> `WarehouseResidencyOverlay.vendor_region_evidence`.
- DW15-ONTO-02: BigQuery `Dataset.location` -> `WarehouseDataset.storage_region`.
- DW15-ONTO-03: Redshift `Namespace.region` -> `WarehouseWorkloadPool.compute_region`.
- DW15-ONTO-04: Databricks SQL `Warehouse.workspace_region` -> `WarehouseEndpoint.compute_region`.
- DW15-ONTO-05: Synapse `Workspace.location` -> `WarehouseImportProvenance.source_region`.
- DW15-ONTO-06: Firebolt `Engine.region` -> `WarehouseWorkloadPool.compute_region`.
- DW15-ONTO-07: ClickHouse Cloud `Service.region` -> `WarehouseDataset.replica_regions`.
- DW15-ONTO-08: Vertica `CommunalStorage.region` -> `WarehouseDataset.storage_region`.
- DW15-ONTO-09: Teradata Vantage `Account.region` -> `WarehouseMigrationSource.region`.
- DW15-ONTO-10: Yellowbrick `Cluster.location` -> `WarehouseWorkloadPool.compute_region`.
- DW15-ONTO-11: Vendor multi-region labels are normalized to explicit storage, compute, and export lists.
- DW15-ONTO-12: Vendor unrestricted location values project to denied state until reviewed.

## Workflow Steps
- DW15-WF-01: Node `LoadTenantPack` reads active data residency pack.
- DW15-WF-02: Node `ResolveVendorRegion` maps imported vendor region to Oyatie region.
- DW15-WF-03: Node `BuildOverlay` calculates storage, compute, and export allow lists.
- DW15-WF-04: Branch `BroadensResidency` requires governance approval.
- DW15-WF-05: Node `EvaluateCedar` tests the overlay mutation.
- DW15-WF-06: Branch `DeniedVendorRegion` blocks and creates migration remediation task.
- DW15-WF-07: Node `PersistOverlay` inserts non-overlapping effective overlay.
- DW15-WF-08: Node `WarmDecisionCache` precomputes decisions for hot datasets.
- DW15-WF-09: Node `EmitAudit` emits `WarehouseResidencyOverlayApplied`.
- DW15-WF-10: Node `NotifyQueryPlanner` updates cell-local planner cache.
- DW15-WF-11: Branch `ShadowOnly` records advisory decisions without blocking queries.
- DW15-WF-12: Node `ExpirePreviousOverlay` closes superseded overlay window.

## Audit Events
- DW15-AUDIT-01: `WarehouseResidencyOverlayProposed` records requested region deltas.
- DW15-AUDIT-02: `WarehouseResidencyOverlayApplied` records effective overlay id and pack.
- DW15-AUDIT-03: `WarehouseResidencyDecisionRecorded` records allow or deny outcome.
- DW15-AUDIT-04: `WarehouseResidencyVendorRegionDenied` records vendor source and region.
- DW15-AUDIT-05: `WarehouseResidencyOverlaySuperseded` records previous overlay id.
- DW15-AUDIT-06: `WarehouseResidencyShadowDecisionObserved` records advisory-only mismatch.
- DW15-AUDIT-07: `WarehouseResidencyPlannerCacheRefreshed` records planner cache generation.

## SLO Targets
- DW15-SLO-01: p50 residency decision <= 8 ms from cache.
- DW15-SLO-02: p95 residency decision <= 35 ms including Cedar call.
- DW15-SLO-03: p99 overlay apply <= 500 ms.
- DW15-SLO-04: throughput >= 2,000 residency decisions per second per cell.
- DW15-SLO-05: availability >= 99.99 percent for residency decisions.
- DW15-SLO-06: overlay propagation p95 <= 5 seconds across query planners.
- DW15-SLO-07: denied-region false allow rate must be 0.
- DW15-SLO-08: stale overlay cache age p99 <= 30 seconds.

## Failure Modes + Recovery
- DW15-FAIL-01: Vendor region cannot be mapped; deny operation, create ontology mapping task, and keep source quarantined.
- DW15-FAIL-02: Overlay overlaps current effective window; reject mutation with conflict and require supersession path.
- DW15-FAIL-03: Planner cache update fails; overlay persists, query planner remains fail-closed on cache miss, and retry outbox.
- DW15-FAIL-04: Pack service is unavailable; use last sealed pack snapshot for read decisions and block overlay writes.
- DW15-FAIL-05: Shadow decision detects vendor drift; create migration note and keep blocking disabled until approved.
- DW15-FAIL-06: Export region denied after query succeeds; block export and emit `WarehouseResidencyDecisionRecorded`.

## Migration Notes
- DW15-MIG-01: Snowflake account region strings require organization-level account locator evidence.
- DW15-MIG-02: BigQuery multi-region `US` and `EU` locations must be decomposed into pack-approved semantics.
- DW15-MIG-03: Redshift namespace region and S3 export bucket region must be evaluated separately.
- DW15-MIG-04: Databricks SQL workspace and external location regions need separate overlay checks.
- DW15-MIG-05: Synapse Analytics storage account location must not be inferred from workspace alone.
- DW15-MIG-06: Firebolt engine and storage region pairing must be captured before cutover.
- DW15-MIG-07: ClickHouse Cloud service replicas require explicit replica region list.
- DW15-MIG-08: Vertica Eon communal storage bucket region controls storage residency.
- DW15-MIG-09: Teradata Vantage migration extracts need source account region proof.
- DW15-MIG-10: Yellowbrick fixed cluster location maps to compute residency only.

## Cross-Microservice Handoffs
- DW15-HANDOFF-01: Compliance receives pack overlay decisions and denied-region events.
- DW15-HANDOFF-02: Policy receives Cedar decision context and overlay mutation evidence.
- DW15-HANDOFF-03: Query planner consumes cache generation and overlay ids.
- DW15-HANDOFF-04: Data-pipeline receives denied export and backfill decisions.
- DW15-HANDOFF-05: Ontology receives vendor region field deltas and canonical region ids.
- DW15-HANDOFF-06: Audit-chain receives all ADR-0263 event classes.
- DW15-HANDOFF-07: Tenant-admin receives effective storage, compute, and export regions.
- DW15-HANDOFF-08: Workflow receives approval branches for broadened overlays.
- DW15-HANDOFF-09: Catalog receives overlay references per dataset.
- DW15-HANDOFF-10: Marketplace receives region eligibility for governed shares.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-015-data-residency-pack-overlays.md` matched `p99, SLO, multi-region`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
