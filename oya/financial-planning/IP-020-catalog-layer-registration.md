---
doc_class: IP
ip_id: IP-020
microservice: financial-planning
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
journey_ref: FP-JOURNEY-CATALOG-LAYER-REGISTRATION
tenant_class: T2
status: Draft
date: 2026-05-20
owner_team: finance-planning-platform
---

# IP-020 Financial Planning catalog-layer-registration

Service: financial-planning
ChangeSet scope: microservices/financial-planning/IP-020-catalog-layer-registration.md
Benchmarks: Anaplan, Workday Adaptive Planning, OneStream, Vena, Pigment
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321

## Objective
- catalog-layer-registration-objective 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- catalog-layer-registration-objective 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- catalog-layer-registration-objective 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- catalog-layer-registration-objective 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- catalog-layer-registration-objective 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- catalog-layer-registration-objective 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Prerequisites
- catalog-layer-registration-prerequisites 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- catalog-layer-registration-prerequisites 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- catalog-layer-registration-prerequisites 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- catalog-layer-registration-prerequisites 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- catalog-layer-registration-prerequisites 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- catalog-layer-registration-prerequisites 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Implementation steps
- catalog-layer-registration-implementation-steps 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- catalog-layer-registration-implementation-steps 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- catalog-layer-registration-implementation-steps 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- catalog-layer-registration-implementation-steps 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- catalog-layer-registration-implementation-steps 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- catalog-layer-registration-implementation-steps 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Tests and evidence
- catalog-layer-registration-tests-and-evidence 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- catalog-layer-registration-tests-and-evidence 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- catalog-layer-registration-tests-and-evidence 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- catalog-layer-registration-tests-and-evidence 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- catalog-layer-registration-tests-and-evidence 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- catalog-layer-registration-tests-and-evidence 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Rollback
- catalog-layer-registration-rollback 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- catalog-layer-registration-rollback 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- catalog-layer-registration-rollback 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- catalog-layer-registration-rollback 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- catalog-layer-registration-rollback 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- catalog-layer-registration-rollback 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Acceptance criteria
- catalog-layer-registration-acceptance-criteria 001: Financial Planning binds forecast-version-open to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.
- catalog-layer-registration-acceptance-criteria 002: Financial Planning binds scenario-recalculate to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Workday Adaptive Planning plus OneStream.
- catalog-layer-registration-acceptance-criteria 003: Financial Planning binds consolidation-close to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=consolidation_cell, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against OneStream plus Vena.
- catalog-layer-registration-acceptance-criteria 004: Financial Planning binds board-report-seal to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=board_report_packet, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Vena plus Pigment.
- catalog-layer-registration-acceptance-criteria 005: Financial Planning binds driver-model-import to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=forecast_version, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Pigment plus Anaplan.
- catalog-layer-registration-acceptance-criteria 006: Financial Planning binds variance-explain to tenant_id, principal_id, audience_type=FINANCE_PLANNING_OWNER, data_class=scenario_input, marketplace DealSet settlement per ADR-0314, HTTP/3 h3-alt-svc plus ECH/PQC per ADR-0253-amendment, and benchmark parity against Anaplan plus Workday Adaptive Planning.

## Context
- IP-020 registers the financial-planning capability in the Oyatie catalog so forecast, scenario, consolidation, and board reporting surfaces are discoverable.
- Catalog entries must describe real contract digests, tier ceilings, Cedar actions, event topics, and vendor migration families.
- The catalog layer must not treat financial planning as a generic BI service; it owns budget versioning, scenario branching, driver input lineage, consolidation status, and board packet sealing.
- Anaplan migration requires catalog facets for model, list, module, line item, and version objects.
- Workday Adaptive Planning migration requires facets for sheet, level, account, assumption, and version objects.
- Oracle EPM Cloud migration requires facets for cube, form, scenario, rule, and approval unit objects.
- OneStream migration requires facets for cube view, workflow profile, entity, account, and certification objects.
- Vena migration requires facets for workbook, template, named range, contributor, and approval objects.
- Pigment migration requires facets for block, metric, dimension, scenario, and application objects.
- Planful, IBM Planning Analytics, Board, and Jedox remain explicit vendor families for catalog search, migration readiness, and adapter routing.

## Data Model Deltas
- Add a catalog registration table scoped to financial-planning capability versions.
- Add a catalog endpoint map table for REST, gRPC, and AsyncAPI surfaces.
- Add a catalog vendor family table so search can find planning capability coverage by vendor.
- Add a catalog policy hook table so UI and generated SDKs can show policy-bound actions.
```sql
CREATE TYPE fp_catalog_surface AS ENUM ('rest', 'grpc', 'asyncapi', 'cedar', 'ontology');
CREATE TYPE fp_catalog_visibility AS ENUM ('private_preview', 'tenant_enabled', 'region_enabled', 'global');
CREATE TABLE financial_planning_catalog_registration (
  registration_id UUID PRIMARY KEY,
  tenant_id UUID,
  capability_slug TEXT NOT NULL,
  capability_version TEXT NOT NULL,
  visibility fp_catalog_visibility NOT NULL,
  tier TEXT NOT NULL CHECK (tier IN ('T1','T2','T3','T4')),
  owner_team TEXT NOT NULL,
  openapi_sha256 TEXT NOT NULL,
  proto_sha256 TEXT NOT NULL,
  asyncapi_sha256 TEXT NOT NULL,
  cedar_schema_sha256 TEXT NOT NULL,
  ontology_projection_sha256 TEXT NOT NULL,
  registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, capability_slug, capability_version)
);
CREATE TABLE financial_planning_catalog_endpoint (
  endpoint_id UUID PRIMARY KEY,
  registration_id UUID NOT NULL REFERENCES financial_planning_catalog_registration(registration_id),
  surface fp_catalog_surface NOT NULL,
  operation_name TEXT NOT NULL,
  method_or_topic TEXT NOT NULL,
  path_or_subject TEXT NOT NULL,
  data_class TEXT NOT NULL,
  policy_action TEXT NOT NULL
);
CREATE TABLE financial_planning_catalog_vendor_family (
  vendor_family_id UUID PRIMARY KEY,
  registration_id UUID NOT NULL REFERENCES financial_planning_catalog_registration(registration_id),
  vendor_name TEXT NOT NULL,
  vendor_object_family TEXT NOT NULL,
  oyatie_object_family TEXT NOT NULL,
  migration_supported BOOLEAN NOT NULL DEFAULT false
);
```
```rust
pub struct FinancialPlanningCatalogRegistration {
    pub registration_id: Uuid,
    pub capability_slug: String,
    pub capability_version: String,
    pub visibility: CatalogVisibility,
    pub tier: TenantClass,
    pub contract_digests: ContractDigests,
    pub vendor_families: Vec<VendorFamilyProjection>,
}
pub struct CatalogEndpointBinding {
    pub surface: CatalogSurface,
    pub operation_name: String,
    pub path_or_subject: String,
    pub data_class: DataClass,
    pub cedar_action: CedarActionName,
}
pub struct VendorFamilyProjection {
    pub vendor_name: String,
    pub vendor_object_family: String,
    pub oyatie_object_family: String,
    pub migration_supported: bool,
}
```

## API Endpoints
- REST `PUT /v1/catalog/financial-planning/registrations/{capability_version}` upserts the capability catalog row.
```json
{"capability_slug":"financial-planning","capability_version":"2026.05.ip020","visibility":"tenant_enabled","tier":"T2","owner_team":"finance-planning-platform","openapi_sha256":"sha256:fp-openapi","proto_sha256":"sha256:fp-proto","asyncapi_sha256":"sha256:fp-asyncapi","cedar_schema_sha256":"sha256:fp-cedar","ontology_projection_sha256":"sha256:fp-ontology"}
```
- REST `POST /v1/catalog/financial-planning/vendor-families` registers vendor migration search metadata.
```json
{"registration_id":"f48b8ef0-6ac1-4795-b64e-40f09109c079","vendor_name":"Oracle EPM Cloud","vendor_object_family":"PlanningCube","oyatie_object_family":"FinancialPlanModel","migration_supported":true}
```
- gRPC `CatalogRegistrationService.RegisterFinancialPlanningCapability` validates contract digests and publishes catalog visibility.
```json
{"capabilitySlug":"financial-planning","capabilityVersion":"2026.05.ip020","surfaces":["REST","GRPC","ASYNCAPI","CEDAR","ONTOLOGY"]}
```
- gRPC `CatalogRegistrationService.ListVendorFamilies` returns planning vendor parity metadata to migration tooling.
```json
{"capabilitySlug":"financial-planning","vendors":["Anaplan","Workday Adaptive Planning","Oracle EPM Cloud","OneStream","Vena","Pigment","Planful","IBM Planning Analytics","Board","Jedox"]}
```
- AsyncAPI topic `catalog.financial-planning.registration.changed.v1` announces visibility changes.
```json
{"event_id":"evt-catalog-ip020","capability_slug":"financial-planning","capability_version":"2026.05.ip020","visibility":"tenant_enabled","changed_by":"catalog-registrar"}
```

## Cedar Policy Hooks
- principal: `CatalogRegistrar::"financial-planning-platform"`.
- action: `Action::"catalog:RegisterFinancialPlanningCapability"`.
- resource: `CatalogCapability::"financial-planning/2026.05.ip020"`.
- context: `{ "tier": "T2", "all_contract_digests_present": true, "owner_team": "finance-planning-platform" }`.
- principal: `FinancePlanningUser::"<principal_id>"`.
- action: `Action::"catalog:ReadFinancialPlanningCapability"`.
- resource: `CatalogCapability::"financial-planning"`.
- context: `{ "tenant_enabled": true, "audience_type": "FINANCE_PLANNING_OWNER", "region": "us-east-1" }`.

## Ontology Projection
- Vendor object `Anaplan Model` maps to Oyatie `FinancialPlanModel` with field delta `catalog_vendor_family_ref`.
- Vendor object `Workday Adaptive Sheet` maps to Oyatie `PlanningInputSurface` with field delta `catalog_sheet_family`.
- Vendor object `Oracle EPM ApprovalUnit` maps to Oyatie `PlanningApprovalNode` with field delta `approval_unit_ref`.
- Vendor object `OneStream CubeView` maps to Oyatie `FinancialCubeView` with field delta `catalog_cube_view_ref`.
- Vendor object `Vena Template` maps to Oyatie `PlanningTemplate` with field delta `workbook_template_ref`.
- Vendor object `Pigment Block` maps to Oyatie `PlanningModelBlock` with field delta `metric_block_ref`.
- Oyatie object `CapabilityCatalogEntry` gains field delta `financial_planning_vendor_families`.
- Oyatie object `CapabilityCatalogEntry` gains field delta `financial_planning_policy_actions`.
- Oyatie object `CapabilityCatalogEntry` gains field delta `financial_planning_contract_digest_set`.
- Oyatie object `CapabilityCatalogEntry` gains field delta `financial_planning_supported_surfaces`.

## Workflow Steps
- Node `ContractDigestCollect`: collect OpenAPI, protobuf, AsyncAPI, Cedar, and ontology digests from the financial-planning build.
- Node `CatalogShapeValidate`: validate mandatory owner, tier, visibility, and data-class fields.
- Branch `DigestMissing`: reject registration and emit `AuditChainCatalogRegistrationRejected`.
- Node `VendorFacetLoad`: load Anaplan, Adaptive, Oracle EPM, OneStream, Vena, Pigment, Planful, IBM Planning Analytics, Board, and Jedox families.
- Node `EndpointBind`: attach REST, gRPC, and AsyncAPI endpoints to catalog operation names.
- Branch `PolicyActionMissing`: keep registration private preview until every mutating endpoint has a Cedar action.
- Node `VisibilityPromote`: move registration to tenant-enabled after SLO and threat-map references exist.
- Node `CatalogEventEmit`: publish `catalog.financial-planning.registration.changed.v1`.
- Branch `ConsumerCacheStale`: emit cache bust events to SDK portal and admin console.
- Node `SearchIndexRefresh`: index vendor families and financial-planning object terms.

## Audit Events
- ADR-0263 `AuditChainCatalogRegistrationStarted` records registrar, capability slug, and version.
- ADR-0263 `AuditChainContractDigestCaptured` records all catalog-bound contract hashes.
- ADR-0263 `AuditChainPolicyDecisionRecorded` records Cedar allow or deny on catalog registration.
- ADR-0263 `AuditChainCatalogVisibilityChanged` records visibility movement.
- ADR-0263 `AuditChainExternalVendorMappingImported` records vendor families.
- ADR-0263 `AuditChainCapabilityPublished` records tenant-enabled catalog publication.

## SLO Targets
- p50 catalog registration write: 90 ms.
- p95 catalog registration write: 260 ms.
- p99 catalog registration write: 650 ms.
- p50 catalog search read: 45 ms.
- p95 catalog search read: 140 ms.
- p99 catalog search read: 400 ms.
- throughput: 300 registration reads per second per region and 25 registration writes per minute.
- availability: 99.99 percent for catalog reads and 99.95 percent for registration writes.

## Failure Modes + Recovery
- Scenario 1: OpenAPI digest is missing; recovery refuses visibility promotion and requests IP-019 regeneration.
- Scenario 2: Catalog row has no Cedar mutating action; recovery holds private preview and opens a threat-map handoff to IP-024.
- Scenario 3: Vendor family duplicates Planful and Adaptive object names; recovery namespaces by `vendor_name` and keeps canonical Oyatie object names stable.
- Scenario 4: AsyncAPI topic name changes after catalog registration; recovery marks endpoint stale and emits a cache refresh event.
- Scenario 5: Tenant visibility is enabled before SLO gates exist; recovery reverts to private preview and emits IP-021 handoff.
- Scenario 6: Search index fails after registration commit; recovery retries index refresh without rolling back the catalog row.

## Migration Notes
- Anaplan catalog entries expose model, module, list, line item, version, and action import coverage.
- Workday Adaptive Planning entries expose sheet, account, level, assumption, version, and workflow coverage.
- Oracle EPM Cloud entries expose cube, form, rule, approval unit, scenario, and period-lock coverage.
- OneStream entries expose cube view, workflow profile, entity, account, scenario, and certification coverage.
- Vena entries expose workbook, template, named range, contributor, approval, and Excel lineage coverage.
- Pigment entries expose application, block, metric, dimension, scenario, and formula coverage.
- Planful entries expose budget entity, template, scenario, planning cycle, and reporting package coverage.
- IBM Planning Analytics entries expose TM1 cube, dimension, subset, process, and cell coverage.
- Board entries expose capsule, dataview, procedure, entity, and version coverage.
- Jedox entries expose cube, dimension, element, rule, integrator job, and splashing coverage.

## Cross-Microservice Handoffs
- To SDK generation: IP-019 receives catalog-visible package coordinates and contract digest references.
- To SLO promotion: IP-021 receives visibility candidates and endpoint operation names.
- To chaos drills: IP-022 receives catalog operation names for drill targeting.
- To DPIA: IP-023 receives vendor families and data-class coverage.
- To threat model: IP-024 receives Cedar actions and endpoint metadata.
- To audit closeout: IP-025 receives catalog evidence links for finding remediation.
- To marketplace: catalog entries expose paid connector settlement metadata.
- To admin console: search indexes expose financial-planning capability cards and vendor migration readiness.
