---
doc_class: ImplementationPlan
ip_id: IP-024
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
journey_ref: j127-aql-sampling-plan-selection
sap_submodule: QM-IM Inspection Management
tenant_class: paid
billing_components:
  - per_usage
persona: Mei Tan, process quality statistician
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-024: Statistical sampling plan generator with MIL-STD-105E AQL tables

## Context

- SAP QM submodule: QM-IM Inspection Management.
- Topic: statistical sampling plan generator using MIL-STD-105E AQL tables.
- Persona: Mei Tan, process quality statistician.
- Journey: j127 AQL sampling plan selection.
- Journey leg: lot size and inspection level produce sample size, accept number, and reject number.
- SAP precedent: sampling procedures, sampling schemes, and inspection severity.
- Oyatie aggregate: `AqlSamplingPlan`.
- Boundary: AQL table lookup, severity state, and generated sample requirement.
- ADR-0105 keeps sampling math pure and testable.
- ADR-0131 keeps the plan in quality-management.
- ADR-0244 protects tenant-specific sampling overrides.
- ADR-0263 binds sampling plan audit events.
- ADR-0297 requires Cedar before reduced or skip sampling.
- ADR-0314 keeps supplier settlement outside sampling.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Generated plan must show its table row.
- Overrides must not hide the AQL basis.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.aql_sampling_plan (
  tenant_id UUID NOT NULL,
  sampling_plan_id TEXT NOT NULL,
  inspection_level TEXT NOT NULL,
  aql_value NUMERIC(10,4) NOT NULL,
  lot_size_min INTEGER NOT NULL,
  lot_size_max INTEGER NOT NULL,
  code_letter TEXT NOT NULL,
  sample_size INTEGER NOT NULL,
  accept_number INTEGER NOT NULL,
  reject_number INTEGER NOT NULL,
  severity_state TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, sampling_plan_id)
);
CREATE TABLE quality_management.aql_generation_decision (
  tenant_id UUID NOT NULL,
  generation_decision_id TEXT NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  sampling_plan_id TEXT NOT NULL,
  lot_size INTEGER NOT NULL,
  generated_sample_size INTEGER NOT NULL,
  generation_reason TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, generation_decision_id)
);
```

### Rust Types

```rust
pub struct AqlSamplingPlan {
    pub tenant_id: TenantId,
    pub sampling_plan_id: SamplingPlanId,
    pub inspection_level: InspectionLevel,
    pub aql_value: Decimal,
    pub lot_size_range: LotSizeRange,
    pub code_letter: CodeLetter,
    pub sample_size: u32,
    pub accept_number: u32,
    pub reject_number: u32,
    pub severity_state: SamplingSeverityState,
}
pub struct AqlGenerationDecision {
    pub generation_decision_id: DecisionId,
    pub inspection_lot_id: InspectionLotId,
    pub sampling_plan_id: SamplingPlanId,
    pub lot_size: u32,
    pub generated_sample_size: u32,
    pub generation_reason: GenerationReason,
}
pub enum InspectionLevel { GeneralI, GeneralII, GeneralIII, SpecialS1, SpecialS2, SpecialS3, SpecialS4 }
pub enum SamplingSeverityState { Reduced, Normal, Tightened }
pub enum AqlSamplingError {
    LotSizeOutOfRange,
    AqlValueUnsupported,
    CodeLetterMissing,
    ReducedSamplingPolicyDenied,
    TableRevisionUnapproved,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/aql-sampling-plans`.
- Registers approved AQL plan table rows.
- `POST /v1/quality-management/inspection-lots/{inspection_lot_id}:generate-sampling-plan`.
- Generates sample requirement for lot size and severity.
- `GET /v1/quality-management/aql-sampling-plans/{sampling_plan_id}`.
- Returns table row and approvals.
- `GET /v1/quality-management/aql-generation-decisions/{decision_id}`.
- Returns generated sample decision and reason.

### gRPC

- Service: `quality_management.aql_sampling.v1.AqlSamplingService`.
- `rpc RegisterAqlPlan(RegisterAqlPlanRequest) returns (AqlSamplingPlanView)`.
- `rpc GenerateSamplingPlan(GenerateSamplingPlanRequest) returns (AqlGenerationDecisionView)`.
- `rpc GetAqlPlan(GetAqlPlanRequest) returns (AqlSamplingPlanView)`.
- `rpc StreamSamplingDecisions(StreamSamplingDecisionsRequest) returns (stream AqlSamplingEvent)`.

### AsyncAPI

- Channel: `quality-management.aql-sampling.generated.v1`.
- Channel: `quality-management.aql-sampling.plan-registered.v1`.
- Message: `AqlSamplingGenerated`.
- Message: `AqlSamplingPlanRegistered`.
- Payload includes `inspection_lot_id`, `inspection_level`, `aql_value`, `sample_size`, `accept_number`, `reject_number`, `audit_event_class`.
- Consumers: inspection-lot, inspection-plan, supplier scorecard, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::aql_sampling::generate`.
- Principal: `InspectionLotWorker`.
- Action: `aql_sampling_generate`.
- Resource: `InspectionLot`.
- Context: `lot_size`, `inspection_level`, `aql_value`, `severity_state`, `regulated_material`.
- Policy: `quality_management::aql_sampling::reduced`.
- Principal: `InspectionLotWorker`.
- Action: `aql_reduced_sampling_apply`.
- Resource: `AqlSamplingPlan`.
- Context: `supplier_risk_tier`, `dynamic_modification_decision`, `pack_ids`, `recent_reject_count`.
- Forbid: unsupported AQL value.
- Forbid: reduced sampling for regulated material unless pack permits.
- Forbid: table revision not approved.
- Forbid: lot size outside table row range.

## Ontology Projection

- Vendor object: SAP QM sampling procedure and scheme.
- Oyatie object: `quality_management.aql_sampling_plan`.
- SAP sampling procedure -> `sampling_plan_id`.
- SAP inspection level -> `inspection_level`.
- SAP AQL value -> `aql_value`.
- SAP lot size interval -> `lot_size_range`.
- SAP code letter -> `code_letter`.
- SAP sample size -> `sample_size`.
- SAP accept number -> `accept_number`.
- SAP reject number -> `reject_number`.
- SAP tightened/reduced indicator -> `severity_state`.
- IQS-AQM sampling plan -> AQL row.
- TIPQA sampling table -> AQL row.
- Projection freshness floor: release-time static plus decision events.
- Projection consumer: inspection-lot and supplier analytics.
- Projection rule: generated decision always references table row.

## Workflow Steps

- Node `table-register`: statistician registers approved AQL row.
- Node `table-review`: table revision checked.
- Decision `revision-unapproved`: block registration.
- Node `lot-size-read`: lot size read from inspection lot.
- Decision `lot-size-out-of-range`: reject generation.
- Node `severity-load`: normal, tightened, or reduced state loaded.
- Decision `reduced-requested`: evaluate reduced sampling policy.
- Node `aql-row-match`: find row by lot size, level, and AQL.
- Decision `row-missing`: reject generation.
- Node `sample-size-derive`: derive sample size and accept/reject numbers.
- Decision `sample-size-greater-lot`: cap at lot size and record reason.
- Node `cedar-generate`: evaluate generation policy.
- Node `decision-record`: persist generated sampling decision.
- Node `lot-sample-update`: update lot sample requirement.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish sampling decision.
- Node `close`: lot receives sampling decision id.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-AQL_SAMPLING-PLAN_REGISTERED`.
- `EVT-QUALITY_MANAGEMENT-AQL_SAMPLING-GENERATED`.
- `EVT-QUALITY_MANAGEMENT-AQL_SAMPLING-REDUCED_DENIED`.
- `EVT-QUALITY_MANAGEMENT-AQL_SAMPLING-TABLE_REJECTED`.
- `EVT-QUALITY_MANAGEMENT-AQL_SAMPLING-IP_ACCEPTED`.
- ADR-0263 envelope stores `inspection_level`.
- ADR-0263 envelope stores `aql_value`.
- ADR-0263 envelope stores `code_letter`.
- ADR-0263 envelope stores `sample_size`.
- ADR-0263 envelope stores `accept_number`.

## SLO Targets

- Sampling generation p50: 15 ms.
- Sampling generation p95: 60 ms.
- Sampling generation p99: 140 ms.
- AQL plan lookup p95: 30 ms.
- Throughput: 1,000 generations per second per cell.
- Availability: 99.97 percent monthly.
- Rationale: lot creation can call sampling generation synchronously.

## Failure Modes and Recovery

- Failure: AQL value unsupported.
- Recovery: `AQL-VALUE-REJECT` rejects generation and names supported values.
- Failure: lot size outside registered rows.
- Recovery: `AQL-LOT-RANGE-BLOCK` requires table registration or manual plan.
- Failure: reduced sampling denied.
- Recovery: `AQL-REDUCED-DENY-NORMAL` falls back to normal severity.
- Failure: table revision is unapproved.
- Recovery: `AQL-TABLE-REVISION-BLOCK` blocks registration.
- Failure: generated sample size exceeds lot size.
- Recovery: `AQL-SAMPLE-CAP-LOT` caps sample size and audits reason.
- Failure: lot sample update fails.
- Recovery: `AQL-LOT-UPDATE-REPLAY` replays generated decision.

## Migration Notes

- Source vendor: SAP QM.
- Migrate sampling procedures and schemes into AQL plan rows.
- Source vendor: IQS-AQM maps inspection sampling plans.
- Source vendor: TIPQA maps AQL tables.
- Source vendor: ETQ Reliance maps inspection plans with sample criteria.
- Source vendor: MasterControl maps approved sampling SOPs into table revision evidence.
- Historical lot sampling decisions migrate as immutable generation decisions.
- Unsupported custom AQL values migrate blocked for review.
- Rollback path: default to inspection plan static sample sizes.
- No imported AQL row can enable reduced sampling without Cedar release.

## Cross-microservice Handoffs

- From inspection-lot: lot size and material criticality.
- From dynamic modification: severity state.
- To inspection-lot: generated sample requirement.
- To supplier scorecard: reduced or tightened sampling signal.
- To audit-evidence: sampling table approval evidence.
- To workflow-engine: missing table review tasks.
- To ontology: sampling plan projection.
- To compliance: regulated sampling rationale.

## Verification

- Unit: unsupported AQL value rejected.
- Unit: reduced sampling denied for regulated material.
- Unit: sample size caps at lot size.
- Contract: REST generation returns accept and reject numbers.
- Contract: gRPC stream emits generated event.
- Event: sampling generated event validates.
- Policy: Cedar denies unapproved table revision.
- Projection: SAP sampling scheme fixture maps field-for-field.
- SLO: generation p95 under 60 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-AQL_SAMPLING-IP_ACCEPTED`.
