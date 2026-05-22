---
doc_class: ImplementationPlan
ip_id: IP-001
microservice: quality-management
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0320
journey_ref: j101-multi-tier-supply-chain-formation
sap_submodule: QM-IM Inspection Management
tenant_class: paid
billing_components:
  - per_usage
persona: Priya Nair, quality planner
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-001: Domain layer for inspection-plan selection

## Context

- SAP QM submodule: QM-IM Inspection Management.
- Topic: inspection-plan selection.
- Persona: Priya Nair, quality planner.
- Journey: j101 multi-tier supply-chain formation.
- Journey leg: inbound component qualification before supplier release.
- Business trigger: a purchased component arrives for a regulated build.
- SAP precedent: inspection plan task lists, material assignment, usage, and status.
- Oyatie equivalent: `QualityInspectionPlan` aggregate.
- Boundary: pure domain model, no database driver, no HTTP client, no vendor adapter.
- ADR-0105 defines the domain layer placement.
- ADR-0131 keeps the IP inside the flat microservice folder.
- ADR-0244 keeps every selector tenant-scoped.
- ADR-0263 binds audit event classes.
- ADR-0297 keeps policy evaluation explicit.
- ADR-0314 keeps marketplace settlement read-only.
- ADR-0315 drives SAP parity coverage.
- ADR-0329/0330/0331 drives ERP depth as implementation-ready substance.
- Success means the plan selector can explain why one plan won over all candidates.
- Failure means inspection lots are created without a defensible plan binding.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE SCHEMA IF NOT EXISTS quality_management;
CREATE TABLE quality_management.inspection_plan (
  tenant_id UUID NOT NULL,
  inspection_plan_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  plant_code TEXT NOT NULL,
  vendor_id TEXT,
  usage_code TEXT NOT NULL,
  status_code TEXT NOT NULL,
  revision_no INTEGER NOT NULL,
  valid_from TIMESTAMPTZ NOT NULL,
  valid_to TIMESTAMPTZ,
  dynamic_modification_rule_id TEXT,
  sample_scheme_id TEXT NOT NULL,
  selected_by_policy TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, inspection_plan_id, revision_no)
);
CREATE INDEX inspection_plan_material_lookup
  ON quality_management.inspection_plan (tenant_id, material_id, plant_code, status_code);
CREATE TABLE quality_management.inspection_plan_operation (
  tenant_id UUID NOT NULL,
  inspection_plan_id TEXT NOT NULL,
  revision_no INTEGER NOT NULL,
  operation_no INTEGER NOT NULL,
  characteristic_code TEXT NOT NULL,
  measurement_unit TEXT NOT NULL,
  lower_spec NUMERIC(20,6),
  upper_spec NUMERIC(20,6),
  destructive_test BOOLEAN NOT NULL DEFAULT FALSE,
  equipment_class TEXT,
  PRIMARY KEY (tenant_id, inspection_plan_id, revision_no, operation_no)
);
```

### Rust Types

```rust
pub struct QualityInspectionPlan {
    pub tenant_id: TenantId,
    pub inspection_plan_id: InspectionPlanId,
    pub material_id: MaterialId,
    pub plant_code: PlantCode,
    pub vendor_id: Option<VendorId>,
    pub usage_code: InspectionUsageCode,
    pub status: PlanStatus,
    pub revision_no: RevisionNo,
    pub validity: ValidityWindow,
    pub dynamic_modification_rule_id: Option<RuleId>,
    pub sample_scheme_id: SampleSchemeId,
    pub operations: Vec<InspectionOperation>,
}
pub struct InspectionOperation {
    pub operation_no: u16,
    pub characteristic_code: CharacteristicCode,
    pub specification: SpecificationLimit,
    pub measurement_unit: UnitOfMeasure,
    pub destructive_test: bool,
    pub equipment_class: Option<EquipmentClass>,
}
pub enum PlanStatus { Draft, Released, Blocked, Superseded }
pub enum PlanSelectionError {
    NoReleasedPlan,
    AmbiguousCandidateSet,
    CrossTenantCandidate,
    ExpiredValidityWindow,
    MissingRequiredCharacteristic,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/inspection-plans`.
- Creates a released or draft plan revision.
- Requires `tenant_id`, `material_id`, `plant_code`, `usage_code`, and operations.
- Returns `202` with `inspection_plan_id`, `revision_no`, and `audit_event_class`.
- `POST /v1/quality-management/inspection-plans:select`.
- Selects the best plan for an inspection lot candidate.
- Request includes `material_id`, `vendor_id`, `plant_code`, `usage_code`, and lot origin.
- Response includes selected plan, ranked rejects, and Cedar decision id.
- `GET /v1/quality-management/inspection-plans/{inspection_plan_id}/explain`.
- Returns the candidate scoring trace.

### gRPC

- Service: `quality_management.inspection_plan.v1.InspectionPlanService`.
- `rpc CreateInspectionPlan(CreateInspectionPlanRequest) returns (InspectionPlanReceipt)`.
- `rpc SelectInspectionPlan(SelectInspectionPlanRequest) returns (PlanSelectionDecision)`.
- `rpc ExplainPlanSelection(ExplainPlanSelectionRequest) returns (PlanSelectionTrace)`.
- gRPC metadata carries `tenant-id`, `principal-id`, `policy-bundle-version`, and `correlation-id`.

### AsyncAPI

- Channel: `quality-management.inspection-plan.changed.v1`.
- Message: `InspectionPlanChanged`.
- Channel: `quality-management.inspection-plan.selected.v1`.
- Message: `InspectionPlanSelected`.
- Payload includes `tenant_id`, `inspection_plan_id`, `revision_no`, `material_id`, `plant_code`, `selection_reason`, `audit_event_class`.
- Consumer groups: inspection-lot, production-planning, ontology, compliance.

## Cedar Policy Hooks

- Policy: `quality_management::inspection_plan::create`.
- Principal: `QualityPlanner` or `QualityEngineer`.
- Action: `inspection_plan_create`.
- Resource: `InspectionPlan::{tenant_id, material_id, plant_code}`.
- Context: `tenant_id`, `principal_id`, `authorized_plants`, `pack_ids`, `policy_bundle_version`.
- Policy: `quality_management::inspection_plan::select`.
- Principal: `InspectionLotWorker`.
- Action: `inspection_plan_select`.
- Resource: `InspectionPlanCandidateSet`.
- Context: `lot_origin`, `vendor_risk_tier`, `material_criticality`, `regulatory_pack`.
- Forbid: candidate plan tenant differs from context tenant.
- Forbid: status is not `Released`.
- Forbid: validity window excludes lot receipt time.
- Permit branch records `cedar_decision_id` into the aggregate event.

## Ontology Projection

- Vendor object: SAP QM `PLKO` task-list header.
- Oyatie object: `quality_management.inspection_plan`.
- `PLKO-PLNTY` -> `plan_type`.
- `PLKO-PLNNR` -> `inspection_plan_id`.
- `PLKO-PLNAL` -> `revision_no`.
- `MAPL-MATNR` -> `material_id`.
- `MAPL-WERKS` -> `plant_code`.
- `MAPL-LIFNR` -> `vendor_id`.
- `QPMK-MERKNR` -> `characteristic_code`.
- `QPMK-MSEHI` -> `measurement_unit`.
- `QPMK-TOLERANZUN` -> `lower_spec`.
- `QPMK-TOLERANZOB` -> `upper_spec`.
- Vendor status text -> `status`.
- Vendor deletion flag -> `lifecycle_state`.
- Vendor change number -> `source_change_ref`.
- Projection freshness floor: 5 seconds.
- Projection ownership: quality-management writes, ontology reads.

## Workflow Steps

- Node `plan-draft-start`: planner creates candidate plan.
- Node `characteristic-bind`: engineer binds measurable characteristics.
- Node `sample-scheme-bind`: planner binds sample scheme.
- Node `validity-review`: system checks effective date windows.
- Decision `missing-characteristic`: send back to engineering.
- Decision `duplicate-active-plan`: require supersede or block.
- Decision `regulated-material`: require compliance reviewer signoff.
- Decision `vendor-specific-plan`: bind vendor id and vendor risk tier.
- Node `cedar-preflight`: evaluate create and release policy.
- Node `release-plan`: state moves Draft -> Released.
- Node `selector-index-refresh`: update candidate lookup.
- Node `lot-candidate-arrives`: inspection lot asks for selection.
- Decision `exact-vendor-match`: prefer vendor-specific plan.
- Decision `material-plant-match`: prefer plant plan.
- Decision `generic-material-match`: allow only low-risk pack.
- Node `selection-explain`: persist ranked decision.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish read model.
- Node `close`: selector ready for lot creation.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-SELECTED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-SUPERSEDED`.
- `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-IP_ACCEPTED`.
- ADR-0263 envelope includes `tenant_id`.
- ADR-0263 envelope includes `principal_id`.
- ADR-0263 envelope includes `correlation_id`.
- ADR-0263 envelope includes `cedar_decision_id`.
- ADR-0263 envelope includes `policy_bundle_version`.

## SLO Targets

- Selection latency p50: 25 ms.
- Selection latency p95: 90 ms.
- Selection latency p99: 180 ms.
- Create or revise p95: 250 ms.
- Throughput: 400 selections per second per cell.
- Availability: 99.95 percent monthly for selection.
- Rationale: lot creation is blocked on selection, but plan authoring is not on the hot path.

## Failure Modes and Recovery

- Failure: no released plan for regulated material.
- Recovery: `PLAN-GAP-QUARANTINE` creates a blocked lot and notifies quality planner.
- Failure: two candidate plans tie on material, plant, vendor, and usage.
- Recovery: `PLAN-SELECTION-TIEBREAK` requires explicit supersede or priority ranking.
- Failure: SAP import sends an expired validity window.
- Recovery: `VALIDITY-WINDOW-REJECT` stores import evidence and rejects release.
- Failure: Cedar denies a supplier-specific plan because principal lacks plant scope.
- Recovery: `PLANT-SCOPE-REAUTH` routes to identity and tenancy.
- Failure: ontology projection lags beyond freshness floor.
- Recovery: `SELECTOR-READMODEL-REPLAY` rebuilds from plan events.
- Failure: dynamic modification rule points to a retired rule id.
- Recovery: `DMR-DETACH-AND-HOLD` blocks release until IP-007 rule repair.

## Migration Notes

- Source vendor: SAP QM.
- Import object set: `PLKO`, `PLPO`, `MAPL`, `QPMK`.
- Migration path: stage vendor rows into `inspection_plan_import_staging`.
- De-duplicate by tenant, material, plant, usage, vendor, and revision.
- Preserve SAP task-list group as `source_task_list_group`.
- Preserve SAP alternative as `source_alternative`.
- Reject unreleased SAP status unless migration pack enables draft import.
- Source vendor: IQS-AQM can map control plans into the same aggregate with weaker operation identity.
- Source vendor: MasterControl requires document-control link preservation.
- Rollback path: disable selector index and retain imported plan rows as blocked.

## Cross-microservice Handoffs

- To inspection-lot: selected plan id and operation list.
- To production-planning: plan readiness for manufactured material release.
- To warehouse: inbound inspection routing for received goods.
- To procurement: supplier-specific plan gap notification.
- To ontology: `inspection_plan` projection.
- To compliance: regulated characteristic evidence.
- To workflow-engine: approval state for regulated plan release.
- To marketplace: read-only supplier quality posture, no settlement mutation.

## Verification

- Unit: selection rejects expired candidate.
- Unit: vendor-specific plan outranks generic plan.
- Unit: cross-tenant candidate is impossible.
- Contract: REST select response includes ranked rejects.
- Contract: gRPC metadata preserves policy bundle.
- Event: AsyncAPI schema validates `InspectionPlanSelected`.
- Policy: Cedar deny on plant scope mismatch.
- Projection: SAP `PLKO` sample maps field-for-field.
- SLO: selector benchmark meets p95 90 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-INSPECTION_PLAN-IP_ACCEPTED`.
