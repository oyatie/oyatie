---
doc_class: ImplementationPlan
ip_id: IP-022
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
journey_ref: j125-cost-of-quality-close
sap_submodule: QM-CA Corrective and Preventive Actions
tenant_class: paid
billing_components:
  - per_usage
persona: Julian Park, quality finance analyst
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-022: Quality Cost PAF ledger

## Context

- SAP QM submodule: QM-CA Corrective and Preventive Actions.
- Topic: Quality Cost ledger using PAF model.
- Persona: Julian Park, quality finance analyst.
- Journey: j125 cost of quality close.
- Journey leg: prevention, appraisal, internal failure, and external failure costs post into finance-ready ledger.
- SAP precedent: quality-related costs from inspection, scrap, rework, returns, and supplier claims.
- Oyatie aggregate: `QualityCostLedgerEntry`.
- Boundary: quality cost classification, evidence link, and finance handoff.
- ADR-0105 separates ledger classification from finance posting adapter.
- ADR-0131 keeps the plan local to quality-management.
- ADR-0244 protects tenant financial evidence.
- ADR-0263 binds quality cost audit events.
- ADR-0297 requires Cedar before cost posting.
- ADR-0314 keeps marketplace settlement separate.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires ERP-grade implementation depth.
- PAF classification must be explainable.
- Finance remains the owner of accounting ledger postings.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.quality_cost_ledger_entry (
  tenant_id UUID NOT NULL,
  quality_cost_entry_id TEXT NOT NULL,
  paf_category TEXT NOT NULL,
  cost_source_type TEXT NOT NULL,
  source_ref TEXT NOT NULL,
  amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  cost_center_id TEXT NOT NULL,
  finance_posting_ref TEXT,
  classification_reason TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, quality_cost_entry_id)
);
CREATE TABLE quality_management.quality_cost_allocation (
  tenant_id UUID NOT NULL,
  allocation_id TEXT NOT NULL,
  quality_cost_entry_id TEXT NOT NULL,
  allocation_target TEXT NOT NULL,
  allocation_percent NUMERIC(10,6) NOT NULL,
  allocated_amount NUMERIC(20,6) NOT NULL,
  PRIMARY KEY (tenant_id, allocation_id)
);
```

### Rust Types

```rust
pub struct QualityCostLedgerEntry {
    pub tenant_id: TenantId,
    pub quality_cost_entry_id: CostEntryId,
    pub paf_category: PafCategory,
    pub cost_source_type: CostSourceType,
    pub source_ref: SourceRef,
    pub amount: Money,
    pub cost_center_id: CostCenterId,
    pub finance_posting_ref: Option<FinancePostingRef>,
    pub classification_reason: ClassificationReason,
    pub allocations: Vec<QualityCostAllocation>,
}
pub enum PafCategory { Prevention, Appraisal, InternalFailure, ExternalFailure }
pub enum CostSourceType { Inspection, Audit, Scrap, Rework, Return, Warranty, SupplierClaim, Training }
pub enum QualityCostError {
    NegativeAmount,
    UnknownPafCategory,
    AllocationDoesNotSum,
    FinancePostingPolicyDenied,
    SourceEvidenceMissing,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/quality-cost-ledger-entries`.
- Creates PAF-classified quality cost entry.
- `POST /v1/quality-management/quality-cost-ledger-entries/{entry_id}:allocate`.
- Allocates cost across targets.
- `POST /v1/quality-management/quality-cost-ledger-entries/{entry_id}:post-finance`.
- Sends finance posting request.
- `GET /v1/quality-management/quality-cost-ledger-entries/{entry_id}`.
- Returns classification reason, source evidence, and finance ref.

### gRPC

- Service: `quality_management.quality_cost.v1.QualityCostService`.
- `rpc CreateQualityCostEntry(CreateQualityCostEntryRequest) returns (QualityCostEntryView)`.
- `rpc AllocateQualityCost(AllocateQualityCostRequest) returns (QualityCostEntryView)`.
- `rpc PostQualityCostToFinance(PostQualityCostRequest) returns (QualityCostEntryView)`.
- `rpc StreamQualityCostEvents(StreamQualityCostEventsRequest) returns (stream QualityCostEvent)`.

### AsyncAPI

- Channel: `quality-management.quality-cost.created.v1`.
- Channel: `quality-management.quality-cost.posted-to-finance.v1`.
- Message: `QualityCostCreated`.
- Message: `QualityCostPostedToFinance`.
- Payload includes `quality_cost_entry_id`, `paf_category`, `amount`, `currency_code`, `cost_center_id`, `audit_event_class`.
- Consumers: finance, supplier scorecard, CAPA, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::quality_cost::create`.
- Principal: `QualityFinanceAnalyst`.
- Action: `quality_cost_create`.
- Resource: `QualityCostLedgerEntry`.
- Context: `source_ref`, `paf_category`, `amount`, `cost_center_id`, `pack_ids`.
- Policy: `quality_management::quality_cost::post_finance`.
- Principal: `QualityFinanceAnalyst`.
- Action: `quality_cost_post_finance`.
- Resource: `QualityCostLedgerEntry`.
- Context: `finance_posting_ref`, `allocation_sum`, `source_evidence_state`, `authorized_cost_centers`.
- Forbid: amount is negative.
- Forbid: source evidence missing.
- Forbid: allocation does not sum to 100 percent.
- Forbid: principal lacks cost center authority.

## Ontology Projection

- Vendor object: SAP QM quality cost and controlling documents.
- Oyatie object: `quality_management.quality_cost_ledger_entry`.
- SAP inspection activity cost -> `Appraisal`.
- SAP training or prevention campaign -> `Prevention`.
- SAP scrap movement -> `InternalFailure`.
- SAP customer return or warranty -> `ExternalFailure`.
- SAP cost center -> `cost_center_id`.
- SAP controlling document -> `finance_posting_ref`.
- TIPQA MRB cost -> internal failure cost.
- TrackWise deviation cost -> failure cost.
- ETQ Reliance complaint cost -> external failure cost.
- MasterControl training record -> prevention cost.
- Projection freshness floor: 10 seconds.
- Projection consumer: finance and quality analytics.
- Projection rule: accounting ledger remains finance-owned.

## Workflow Steps

- Node `source-event-received`: scrap, rework, audit, training, or return creates candidate cost.
- Node `source-evidence-load`: source record and amount evidence loaded.
- Decision `source-evidence-missing`: reject entry.
- Node `paf-classify`: classify prevention, appraisal, internal failure, or external failure.
- Decision `category-unknown`: route analyst review.
- Node `cost-center-resolve`: map source to cost center.
- Decision `cost-center-unauthorized`: reject posting.
- Node `allocation-create`: optional allocation across product, supplier, or plant.
- Decision `allocation-not-100`: block finance post.
- Node `cedar-create`: evaluate create policy.
- Node `entry-create`: persist entry.
- Node `cedar-finance`: evaluate finance post policy.
- Node `finance-post`: send finance command.
- Node `supplier-score-signal`: publish failure cost signal.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish quality cost.
- Node `close`: entry immutable except finance ref.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-QUALITY_COST-CREATED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_COST-ALLOCATED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_COST-POSTED_TO_FINANCE`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_COST-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-QUALITY_COST-IP_ACCEPTED`.
- ADR-0263 envelope stores `paf_category`.
- ADR-0263 envelope stores `amount`.
- ADR-0263 envelope stores `currency_code`.
- ADR-0263 envelope stores `cost_center_id`.
- ADR-0263 envelope stores `finance_posting_ref`.

## SLO Targets

- Entry create p50: 60 ms.
- Entry create p95: 220 ms.
- Finance post request p95: 700 ms excluding finance service processing.
- Quality cost report p95: 300 ms.
- Throughput: 150 entries per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: quality cost is close-critical but not production-hot.

## Failure Modes and Recovery

- Failure: source evidence is missing.
- Recovery: `Q-COST-SOURCE-EVIDENCE-BLOCK` rejects entry until evidence exists.
- Failure: amount is negative.
- Recovery: `Q-COST-AMOUNT-REJECT` rejects entry.
- Failure: allocation does not sum to 100 percent.
- Recovery: `Q-COST-ALLOCATION-BLOCK` keeps entry unposted.
- Failure: finance posting fails.
- Recovery: `Q-COST-FINANCE-REPLAY` retries post command idempotently.
- Failure: cost center authority missing.
- Recovery: `Q-COST-AUTH-DENY` routes access request to finance owner.
- Failure: PAF category is uncertain.
- Recovery: `Q-COST-CLASSIFICATION-REVIEW` creates analyst task.

## Migration Notes

- Source vendor: SAP QM with SAP CO references.
- Migrate inspection, scrap, rework, and warranty costs into PAF entries.
- Source vendor: TIPQA maps MRB cost records.
- Source vendor: TrackWise maps deviation and CAPA cost records.
- Source vendor: ETQ Reliance maps complaint cost.
- Source vendor: MasterControl maps prevention training cost.
- Historical finance posting refs migrate read-only.
- Unclassified historical cost migrates as blocked review.
- Rollback path: disable finance posting and retain quality ledger entries.
- Marketplace supplier chargebacks remain outside this ledger.

## Cross-microservice Handoffs

- From quality-hold: scrap and return costs.
- From CAPA: prevention and corrective action costs.
- From audit: appraisal and prevention costs.
- From customer complaints: external failure costs.
- To finance: accounting ledger posting.
- To supplier scorecard: failure cost input.
- To procurement: supplier recovery claim evidence.
- To ontology: PAF quality cost projection.

## Verification

- Unit: negative amount rejected.
- Unit: allocation must sum to 100 percent.
- Unit: missing source evidence blocks finance posting.
- Contract: REST create returns PAF category and reason.
- Contract: gRPC stream emits finance posted event.
- Event: quality cost created event validates.
- Policy: Cedar denies unauthorized cost center.
- Projection: SAP CO quality cost fixture maps field-for-field.
- SLO: entry create p95 under 220 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-QUALITY_COST-IP_ACCEPTED`.
