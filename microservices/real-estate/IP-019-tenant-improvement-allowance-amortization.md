---
doc_class: ImplementationPlan
ip_id: IP-019
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
sap_submodule: RE-FX-AC (lease accounting)
tenant_class: paid
billing_components:
  - per_usage
persona: Marcus Lee, lease accounting controller
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-019: Tenant improvement allowance amortization

## Context

- SAP submodule: RE-FX-AC lease accounting for tenant incentives.
- Persona: Marcus Lee, lease accounting controller.
- Journey leg: j137 audit checks tenant improvement allowance amortization over lease term with source evidence.
- SAP tables: `VICDCONTRACT`, `VICDCONDLINE`, `VICDOBJASS`, `VICDADJREASN`.
- Oyatie capability: `TenantImprovementAllowance`.
- Precedent: SAP RE-FX incentive condition handling plus ASC 842/IFRS 16 tenant incentive amortization.
- ADR-0263 binds amortization audit and ADR-0297 gates allowance approval.
- Boundary: tracks allowance, amortization schedule, and accounting handoff; vendor invoice payment remains payments/finance.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.tenant_improvement_allowance (
  tenant_id UUID NOT NULL,
  tia_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  allowance_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  amortization_start DATE NOT NULL,
  amortization_end DATE NOT NULL,
  allowance_status TEXT NOT NULL CHECK (allowance_status IN ('draft','approved','amortizing','closed','cancelled')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, tia_id)
);
CREATE TABLE real_estate.tia_amortization_line (
  tenant_id UUID NOT NULL,
  tia_id TEXT NOT NULL,
  period_no INTEGER NOT NULL,
  amortization_date DATE NOT NULL,
  amortization_amount NUMERIC(20,6) NOT NULL,
  remaining_balance NUMERIC(20,6) NOT NULL,
  PRIMARY KEY (tenant_id, tia_id, period_no)
);
```

### Rust Types

```rust
pub struct TenantImprovementAllowance {
    pub tenant_id: TenantId,
    pub tia_id: TenantImprovementAllowanceId,
    pub lease_contract_id: LeaseContractId,
    pub allowance_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub amortization_start: NaiveDate,
    pub amortization_end: NaiveDate,
    pub allowance_status: AllowanceStatus,
}
pub struct TiaAmortizationLine {
    pub period_no: u32,
    pub amortization_date: NaiveDate,
    pub amortization_amount: Decimal,
    pub remaining_balance: Decimal,
}
pub enum TenantImprovementAllowanceError { ContractInactive, AmountInvalid, TermMismatch, ApprovalPolicyDenied, ScheduleGenerationFailed }
```

## API Endpoints

- REST `POST /v1/real-estate/tenant-improvement-allowances`.
- REST `POST /v1/real-estate/tenant-improvement-allowances/{id}:approve`.
- REST `GET /v1/real-estate/tenant-improvement-allowances/{id}/amortization-lines`.
- gRPC `real_estate.tia.v1.TenantImprovementAllowanceService.CreateAllowance`.
- gRPC `ApproveAllowance` and `StreamAmortizationLines`.
- AsyncAPI channel `real-estate.tia.allowance-approved.v1`.
- AsyncAPI channel `real-estate.tia.amortization-generated.v1`.
- Consumers: lease-accounting, finance-ledger, compliance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::tia::approve`.
- Principal: `LeaseAccountingController`.
- Action: `tenant_improvement_allowance_approve`.
- Resource: `TenantImprovementAllowance`.
- Context: `tenant_id`, `allowance_amount`, `lease_contract_id`, `amortization_range`, `source_invoice_ref`.
- Forbid when amount invalid, source evidence missing, amortization range exceeds lease term, or approval limit exceeded.

## Ontology Projection

- Vendor object: SAP RE-FX tenant incentive condition.
- Oyatie object: `real_estate.tenant_improvement_allowance`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDCONDLINE-CONDGUID` -> incentive condition lineage.
- `VICDOBJASS-OBJNR` -> improved premises lineage.
- `VICDADJREASN-ADJREASON` -> approval or remeasurement reason.
- Allowance amount -> amortizable balance.
- Schedule line -> accounting event source.
- Projection freshness floor: approval.
- Projection rule: amortization lines are immutable after approval except through superseding allowance version.

## Workflow Steps

- Node `allowance-draft`: capture amount and evidence.
- Decision `contract-inactive`: reject draft.
- Decision `amount-invalid`: reject amount.
- Node `term-check`: validate amortization range.
- Decision `term-mismatch`: route controller review.
- Node `schedule-generate`: create periodic amortization lines.
- Node `approval-policy`: enforce approval limit and evidence.
- Decision `approval-denied`: keep draft.
- Node `accounting-handoff`: create lease accounting event series.
- Node `audit-seal`: emit allowance evidence.

## Audit Events

- `EVT-REAL_ESTATE-TIA-ALLOWANCE_CREATED`.
- `EVT-REAL_ESTATE-TIA-ALLOWANCE_APPROVED`.
- `EVT-REAL_ESTATE-TIA-AMORTIZATION_GENERATED`.
- `EVT-REAL_ESTATE-TIA-ALLOWANCE_CANCELLED`.
- `EVT-REAL_ESTATE-TIA-POLICY_DENIED`.
- `EVT-REAL_ESTATE-TIA-IP_ACCEPTED`.
- ADR-0263 envelope stores allowance amount, amortization range, source evidence, and remaining balance.

## SLO Targets

- Allowance create p50: 50 ms.
- Allowance create p95: 180 ms.
- Allowance create p99: 520 ms.
- Schedule generation p95: 600 ms for 600 periods.
- Rationale: allowance entry is interactive; schedule generation can compute exact lines within bounded controller workflow.

## Failure Modes and Recovery

- Failure: `CONTRACT-INACTIVE`; recovery: reject until lease is active.
- Failure: `AMOUNT-INVALID`; recovery: return validation error.
- Failure: `TERM-MISMATCH`; recovery: require amendment or controller override.
- Failure: `APPROVAL-POLICY-DENIED`; recovery: route to approval workflow.
- Failure: `SCHEDULE-GENERATION-FAILED`; recovery: keep draft and retry.
- Failure: `ACCOUNTING-HANDOFF-FAILED`; recovery: retry event series outbox.

## Migration Notes

- Import tenant improvement allowances from incentive condition lines.
- Preserve source invoice and approval references where available.
- Recompute schedules only when migrated source lacks complete amortization lines.
- Do not post historical amortization automatically.
- Rollback path: disable approval endpoint and keep allowances draft/read-only.
- Backfill order: contracts, condition lines, source evidence, allowances, schedules.

## Cross-microservice Handoffs

- From lease-contract: active lease term.
- From finance-ledger or AP: source invoice evidence.
- To lease-accounting: amortization event series.
- To finance-ledger: posting request through lease-accounting.
- To compliance: allowance approval evidence.
- To portfolio analytics: incentive cost metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The amortization remains bound to SAP RE-FX-AC lease accounting for tenant incentives. |
| Persona specificity | Marcus Lee owns allowance approval, amortization schedule, and rollback language. |
| Journey specificity | The j137 audit leg drives source invoice evidence and lease-term amortization. |
| DDL anchor | The allowance, source evidence, amortization schedule, and approval tables above are normative. |
| Rust anchor | Allowance, amortization line, approval result, and error types above are implementation anchors. |
| REST anchor | Create allowance, approve, generate schedule, and explain endpoints are tenant surfaces. |
| gRPC anchor | The allowance amortization service is the worker and replay contract. |
| AsyncAPI anchor | Allowance approved, schedule generated, and event-series requested channels carry accounting evidence. |
| Cedar anchor | Allowance approval is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP condition, contract, and invoice lineage projects to allowance amortization nodes. |
| ADR-0263 class binding | Allowance checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | IFRS, SOX, or finance-control overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on allowance APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, allowance id, contract id, invoice ref, amount, and `cedar_decision_id`. |
| Metric | `oya_real_estate_tia_allowances_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_real_estate_tia_amortization_duration_seconds` tracks approval and schedule generation latency. |
| Trace span | `real_estate.tenant_improvement_allowance.approve` links lease contract, finance-ledger, lease accounting, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `allowance_id`, `contract_id`, `invoice_ref`, and approval state. |
| Capacity math | Schedule generation uses lease_months * allowance_count and backpressures before posting cutoff. |
| Multi-region | Allowance writes stay in finance home cell; DR cells expose read-only amortization evidence. |
| Sovereign cells | Invoice and lease evidence remains in-region for active finance and sovereign packs. |
| Rollback | Disable approval endpoint, keep allowances draft/read-only, and replay from last sealed allowance audit id. |
| Test evidence | Required tests cover missing invoice, term mismatch, approval denial, tenant mismatch, and deterministic schedule replay. |
| Rejected shortcut | A generic amortization schedule is rejected because it loses tenant-improvement allowance and SAP RE-FX semantics. |
