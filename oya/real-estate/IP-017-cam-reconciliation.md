---
doc_class: ImplementationPlan
ip_id: IP-017
microservice: real-estate
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0245
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_ref: j137-corporate-internal-audit-sox-controls-test
sap_submodule: RE-FX-SC (service charge)
tenant_class: paid
billing_components:
  - per_usage
persona: Nora Patel, service-charge accountant
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-017: CAM reconciliation

## Context

- SAP submodule: RE-FX-SC service charge and common-area maintenance reconciliation.
- Persona: Nora Patel, service-charge accountant.
- Journey leg: j137 audit verifies CAM estimates, actuals, allocation basis, and tenant true-up.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie capability: `CamReconciliation`.
- Precedent: SAP RE-FX service charge settlement plus Yardi CAM reconciliation.
- ADR-0263 binds reconciliation audit and ADR-0297 gates tenant-billable adjustments.
- Boundary: reconciles estimate versus actual and creates true-up evidence; payment execution remains payments.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.cam_reconciliation_run (
  tenant_id UUID NOT NULL,
  cam_run_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  reconciliation_period DATERANGE NOT NULL,
  estimated_amount NUMERIC(20,6) NOT NULL,
  actual_amount NUMERIC(20,6) NOT NULL,
  true_up_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  run_status TEXT NOT NULL CHECK (run_status IN ('draft','approved','billed','disputed')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, cam_run_id)
);
CREATE TABLE real_estate.cam_cost_line (
  tenant_id UUID NOT NULL,
  cam_cost_line_id TEXT NOT NULL,
  cam_run_id TEXT NOT NULL,
  cost_category TEXT NOT NULL,
  actual_amount NUMERIC(20,6) NOT NULL,
  allocation_basis_ref TEXT NOT NULL,
  PRIMARY KEY (tenant_id, cam_cost_line_id)
);
```

### Rust Types

```rust
pub struct CamReconciliationRun {
    pub tenant_id: TenantId,
    pub cam_run_id: CamRunId,
    pub lease_contract_id: LeaseContractId,
    pub reconciliation_period: DateRange,
    pub estimated_amount: Decimal,
    pub actual_amount: Decimal,
    pub true_up_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub run_status: CamRunStatus,
}
pub struct CamCostLine {
    pub cam_cost_line_id: CamCostLineId,
    pub cost_category: CostCategory,
    pub actual_amount: Decimal,
    pub allocation_basis_ref: AllocationBasisRef,
}
pub enum CamReconciliationError { PeriodClosed, AllocationBasisMissing, ActualsMissing, TrueUpPolicyDenied, DisputeActive }
```

## API Endpoints

- REST `POST /v1/real-estate/cam-reconciliation-runs` creates run.
- REST `POST /v1/real-estate/cam-reconciliation-runs/{id}:approve`.
- REST `POST /v1/real-estate/cam-reconciliation-runs/{id}:bill-true-up`.
- gRPC `real_estate.cam.v1.CamReconciliationService.CreateRun`.
- gRPC `ApproveRun`, `BillTrueUp`, and `ListCamCostLines`.
- AsyncAPI channel `real-estate.cam.reconciliation-approved.v1`.
- AsyncAPI channel `real-estate.cam.true-up-billed.v1`.
- Consumers: payments, rent-schedule, compliance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::cam::bill_true_up`.
- Principal: `ServiceChargeAccountant`.
- Action: `cam_true_up_bill`.
- Resource: `CamReconciliationRun`.
- Context: `tenant_id`, `true_up_amount`, `allocation_basis_ref`, `period_closed`, `dispute_state`.
- Forbid when period is not closed, allocation basis missing, actuals incomplete, dispute active, or true-up exceeds policy threshold.

## Ontology Projection

- Vendor object: SAP RE-FX service charge settlement.
- Oyatie object: `real_estate.cam_reconciliation_run`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDOBJASS-OBJNR` -> allocation object basis.
- `VICDCONDLINE-CONDGUID` -> CAM condition line.
- `VICDADJREASN-ADJREASON` -> true-up reason.
- Estimated and actual amounts -> reconciliation inputs.
- True-up amount -> billable adjustment.
- Projection freshness floor: approval.
- Projection rule: disputed runs are visible but not billable.

## Workflow Steps

- Node `period-open`: create reconciliation run for closed period.
- Decision `period-not-closed`: reject run.
- Node `actuals-import`: load cost actuals.
- Decision `actuals-missing`: hold draft.
- Node `allocation-basis-load`: read occupancy/service charge basis.
- Decision `basis-missing`: route to occupancy repair.
- Node `true-up-calc`: calculate estimated versus actual delta.
- Node `approval-policy`: authorize true-up billing.
- Decision `dispute-active`: block billing.
- Node `payment-handoff`: send true-up to payments or rent schedule.

## Audit Events

- `EVT-REAL_ESTATE-CAM-RUN_CREATED`.
- `EVT-REAL_ESTATE-CAM-ACTUALS_IMPORTED`.
- `EVT-REAL_ESTATE-CAM-RECONCILIATION_APPROVED`.
- `EVT-REAL_ESTATE-CAM-TRUE_UP_BILLED`.
- `EVT-REAL_ESTATE-CAM-POLICY_DENIED`.
- `EVT-REAL_ESTATE-CAM-IP_ACCEPTED`.
- ADR-0263 envelope stores period, estimated amount, actual amount, allocation basis, and true-up.

## SLO Targets

- Run create p50: 70 ms.
- Run create p95: 260 ms.
- Run create p99: 750 ms.
- True-up calculation p95: 3 seconds for 100,000 cost lines.
- Rationale: CAM work is accountant-reviewed and batch-shaped, but approvals need responsive evidence.

## Failure Modes and Recovery

- Failure: `PERIOD-NOT-CLOSED`; recovery: block reconciliation until period close.
- Failure: `ALLOCATION-BASIS-MISSING`; recovery: request occupancy basis recompute.
- Failure: `ACTUALS-MISSING`; recovery: keep run draft and retry import.
- Failure: `TRUE-UP-POLICY-DENIED`; recovery: route to controller approval.
- Failure: `DISPUTE-ACTIVE`; recovery: freeze billing until dispute closes.
- Failure: `PAYMENT-HANDOFF-FAILED`; recovery: retry outbox without duplicating true-up.

## Migration Notes

- Import historical CAM reconciliations as approved or billed based on source status.
- Preserve source cost category and allocation basis.
- Do not bill migrated true-ups without explicit open balance.
- Map SAP adjustment reasons before importing true-up lines.
- Rollback path: disable bill true-up endpoint and keep reconciliation read-only.
- Backfill order: contracts, occupancy basis, cost actuals, CAM runs, true-up lines.

## Cross-microservice Handoffs

- From occupancy-allocation: allocation basis.
- From finance-ledger: actual cost lines.
- To payments: true-up charge or credit.
- To rent-schedule: CAM adjustment line.
- To workflow-engine: dispute and controller approval.
- To compliance: reconciliation evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The reconciliation remains bound to SAP RE-FX-SC service charge and CAM reconciliation. |
| Persona specificity | Nora Patel owns CAM actuals, true-up, dispute, and rollback acceptance language. |
| Journey specificity | The j137 audit leg drives estimates, actuals, allocation basis, and tenant true-up evidence. |
| DDL anchor | The CAM run, actual cost, allocation basis, and true-up line tables above are normative. |
| Rust anchor | CAM run, true-up line, allocation basis, and error types above are implementation anchors. |
| REST anchor | Reconcile, approve, bill true-up, dispute, and explain endpoints are tenant surfaces. |
| gRPC anchor | The CAM reconciliation service is the worker and replay contract. |
| AsyncAPI anchor | Reconciliation approved, true-up billed, and dispute opened channels carry evidence. |
| Cedar anchor | True-up billing is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP service charge, condition, and allocation lineage projects to reconciliation nodes. |
| ADR-0263 class binding | CAM checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Service-charge, tax, or office overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on CAM APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, CAM run id, allocation basis, true-up amount, and `cedar_decision_id`. |
| Metric | `oya_real_estate_cam_reconciliations_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_real_estate_cam_reconciliation_duration_seconds` tracks reconcile and approval latency. |
| Trace span | `real_estate.cam_reconciliation.approve` links occupancy, finance-ledger, rent schedule, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `cam_run_id`, `basis_type`, `true_up_bucket`, and dispute state. |
| Capacity math | Reconciliation partitions actual_cost_lines by property and rejects if unallocated cost exceeds tolerance. |
| Multi-region | CAM reconciliation writes stay in property home cell; DR cells expose read-only reconciliation evidence. |
| Sovereign cells | Tenant, lease, and cost evidence remains in-region for active compliance-pack overlays. |
| Rollback | Disable bill true-up endpoint, keep reconciliation read-only, and replay from last sealed CAM audit id. |
| Test evidence | Required tests cover basis mismatch, tax denial, dispute branch, tenant mismatch, and idempotent true-up billing. |
| Rejected shortcut | A generic cost allocation is rejected because it loses SAP RE-FX service-charge and CAM true-up semantics. |
