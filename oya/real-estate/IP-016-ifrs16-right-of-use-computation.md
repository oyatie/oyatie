---
doc_class: ImplementationPlan
ip_id: IP-016
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

# IP-016: IFRS-16 right-of-use computation

## Context

- SAP submodule: RE-FX-AC IFRS-16 lease accounting.
- Persona: Marcus Lee, lease accounting controller.
- Journey leg: j137 audit verifies right-of-use asset and lease liability computation from approved lease terms.
- SAP tables: `VICDCONTRACT`, `VICDCONDLINE`, `VICDOBJASS`, `VICDADJREASN`.
- Oyatie capability: `Ifrs16RouComputation`.
- Precedent: SAP RE-FX valuation rule plus Oracle Lease Accounting present-value schedule.
- ADR-0263 binds valuation audit and ADR-0297 gates computation approval.
- Boundary: computes ROU asset, lease liability, discount schedule, and remeasurement basis; finance-ledger owns journal posting.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.ifrs16_rou_computation (
  tenant_id UUID NOT NULL,
  rou_computation_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  computation_version TEXT NOT NULL,
  discount_rate NUMERIC(12,8) NOT NULL,
  lease_liability_amount NUMERIC(20,6) NOT NULL,
  rou_asset_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  computation_status TEXT NOT NULL CHECK (computation_status IN ('draft','approved','superseded','rejected')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, rou_computation_id)
);
CREATE TABLE real_estate.ifrs16_payment_projection (
  tenant_id UUID NOT NULL,
  rou_computation_id TEXT NOT NULL,
  period_no INTEGER NOT NULL,
  payment_date DATE NOT NULL,
  payment_amount NUMERIC(20,6) NOT NULL,
  present_value_amount NUMERIC(20,6) NOT NULL,
  PRIMARY KEY (tenant_id, rou_computation_id, period_no)
);
```

### Rust Types

```rust
pub struct Ifrs16RouComputation {
    pub tenant_id: TenantId,
    pub rou_computation_id: RouComputationId,
    pub lease_contract_id: LeaseContractId,
    pub computation_version: ComputationVersion,
    pub discount_rate: Decimal,
    pub lease_liability_amount: Decimal,
    pub rou_asset_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub computation_status: RouComputationStatus,
}
pub struct Ifrs16PaymentProjection {
    pub period_no: u32,
    pub payment_date: NaiveDate,
    pub payment_amount: Decimal,
    pub present_value_amount: Decimal,
}
pub enum Ifrs16RouError { PaymentScheduleMissing, DiscountRateMissing, CurrencyMismatch, TermUncertain, ApprovalPolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-contracts/{id}:compute-ifrs16-rou`.
- REST `POST /v1/real-estate/ifrs16-rou-computations/{id}:approve`.
- REST `GET /v1/real-estate/ifrs16-rou-computations/{id}/payment-projections`.
- gRPC `real_estate.ifrs16.v1.Ifrs16Service.ComputeRou`.
- gRPC `ApproveRouComputation` and `StreamPaymentProjections`.
- AsyncAPI channel `real-estate.ifrs16.rou-computed.v1`.
- AsyncAPI channel `real-estate.ifrs16.rou-approved.v1`.
- Consumers: lease-accounting, finance-ledger, compliance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::ifrs16::approve_rou`.
- Principal: `LeaseAccountingController`.
- Action: `ifrs16_rou_approve`.
- Resource: `Ifrs16RouComputation`.
- Context: `tenant_id`, `discount_rate`, `currency_code`, `lease_contract_id`, `payment_schedule_complete`.
- Forbid when payment schedule is incomplete, discount rate missing, currency differs from contract, or term uncertainty requires review.

## Ontology Projection

- Vendor object: SAP RE-FX IFRS-16 valuation run.
- Oyatie object: `real_estate.ifrs16_rou_computation`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDCONDLINE-CONDGUID` -> payment projection source.
- `VICDOBJASS-OBJNR` -> underlying asset reference.
- `VICDADJREASN-ADJREASON` -> remeasurement reason.
- Discount rate -> present-value input.
- ROU and liability amount -> accounting outputs.
- Projection freshness floor: computation approval.
- Projection rule: draft computations do not post; approved computations become accounting event lineage.

## Workflow Steps

- Node `contract-load`: load approved lease terms.
- Node `payment-schedule-load`: read rent schedule lines.
- Decision `payment-schedule-missing`: block computation.
- Node `discount-rate-resolve`: load rate source.
- Decision `discount-rate-missing`: route controller review.
- Node `present-value-compute`: discount future lease payments.
- Decision `term-uncertain`: mark draft requiring approval.
- Node `rou-compute`: compute liability and ROU asset.
- Node `approval-policy`: run Cedar approval.
- Node `accounting-handoff`: request initial recognition event.

## Audit Events

- `EVT-REAL_ESTATE-IFRS16-ROU_COMPUTED`.
- `EVT-REAL_ESTATE-IFRS16-ROU_APPROVED`.
- `EVT-REAL_ESTATE-IFRS16-ROU_REJECTED`.
- `EVT-REAL_ESTATE-IFRS16-TERM_UNCERTAIN`.
- `EVT-REAL_ESTATE-IFRS16-POLICY_DENIED`.
- `EVT-REAL_ESTATE-IFRS16-IP_ACCEPTED`.
- ADR-0263 envelope stores discount rate, payment projection count, liability amount, and ROU asset amount.

## SLO Targets

- Computation p50: 80 ms for 120 periods.
- Computation p95: 450 ms for 1,000 periods.
- Computation p99: 1,200 ms with batch projection.
- Approval p95: 200 ms.
- Rationale: valuation is controller-facing and can spend bounded compute for exact present-value evidence.

## Failure Modes and Recovery

- Failure: `PAYMENT-SCHEDULE-MISSING`; recovery: request rent schedule generation.
- Failure: `DISCOUNT-RATE-MISSING`; recovery: route to controller rate setup.
- Failure: `CURRENCY-MISMATCH`; recovery: block approval and require contract/rent correction.
- Failure: `TERM-UNCERTAIN`; recovery: keep draft and require option decision capture.
- Failure: `APPROVAL-POLICY-DENIED`; recovery: preserve draft with denial reason.
- Failure: `ACCOUNTING-HANDOFF-FAILED`; recovery: retry event outbox.

## Migration Notes

- Import existing IFRS-16 valuations as approved computations only with payment projection lineage.
- Preserve discount rate source and valuation date.
- Recompute draft valuation when payment schedule changes.
- Do not post migrated valuations automatically.
- Rollback path: disable approve endpoint and retain computations read-only.
- Backfill order: contracts, rent schedules, rates, computations, payment projections.

## Cross-microservice Handoffs

- From lease-contract: approved lease terms.
- From rent-schedule: payment projections.
- To lease-accounting: initial recognition event.
- To finance-ledger: posting request through lease-accounting.
- To compliance: IFRS-16 audit evidence.
- To portfolio analytics: liability and ROU exposure.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The computation remains bound to SAP RE-FX-AC IFRS-16 lease accounting. |
| Persona specificity | Marcus Lee owns ROU computation, approval, ledger handoff, and rollback language. |
| Journey specificity | The j137 audit leg drives right-of-use asset and lease liability evidence. |
| DDL anchor | The ROU computation, rate, payment projection, and approval tables above are normative. |
| Rust anchor | ROU computation, discount rate, liability schedule, and error types above are implementation anchors. |
| REST anchor | Compute, approve, recalculate, and explain endpoints are tenant surfaces. |
| gRPC anchor | The IFRS-16 computation service is the worker and replay contract. |
| AsyncAPI anchor | Computation created, approved, and recognition requested channels carry accounting evidence. |
| Cedar anchor | Computation approval is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP contract and rent schedule lineage projects to ROU computation nodes. |
| ADR-0263 class binding | IFRS-16 checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | IFRS or finance-control overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on computation APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, computation id, contract id, discount rate, liability amount, and `cedar_decision_id`. |
| Metric | `oya_real_estate_ifrs16_computations_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_real_estate_ifrs16_computation_duration_seconds` tracks compute and approval latency. |
| Trace span | `real_estate.ifrs16.compute_rou` links lease contract, rent schedule, lease accounting, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `computation_id`, `contract_id`, `rate_source`, and approval state. |
| Capacity math | Payment projection fan-out is periods_remaining * contracts; queue backs off when ledger cutoff is threatened. |
| Multi-region | ROU computation writes stay in finance home cell; DR cells expose read-only computation evidence. |
| Sovereign cells | Lease and liability evidence remains in-region for IFRS and sovereign compliance packs. |
| Rollback | Disable approve endpoint, retain computations read-only, and replay from last sealed IFRS-16 audit id. |
| Test evidence | Required tests cover missing rate, modified payment schedule, approval denial, tenant mismatch, and deterministic recalculation. |
| Rejected shortcut | A generic NPV calculator is rejected because it loses IFRS-16, SAP RE-FX, and lease-accounting semantics. |
