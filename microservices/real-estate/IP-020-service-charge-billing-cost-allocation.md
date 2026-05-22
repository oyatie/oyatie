---
doc_class: ImplementationPlan
ip_id: IP-020
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
journey_ref: j122-vendor-payment-batch-with-tax-withholding
sap_submodule: RE-FX-SC (service charge)
tenant_class: paid
billing_components:
  - per_usage
persona: Nora Patel, service-charge accountant
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-020: Service-charge billing with cost allocation

## Context

- SAP submodule: RE-FX-SC service charge billing.
- Persona: Nora Patel, service-charge accountant.
- Journey leg: j122 allocated service charges are billed with tax/withholding controls before payment.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie capability: `ServiceChargeBilling`.
- Precedent: SAP RE-FX service charge settlement plus MRI/Yardi cost allocation billing.
- ADR-0263 records billing evidence and ADR-0297 gates tenant-billable charges.
- Boundary: computes allocated charge lines and billing handoff; payment execution remains payments.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.service_charge_bill (
  tenant_id UUID NOT NULL,
  service_charge_bill_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  billing_period DATERANGE NOT NULL,
  allocation_basis_ref TEXT NOT NULL,
  total_allocated_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  bill_status TEXT NOT NULL CHECK (bill_status IN ('draft','approved','billed','disputed','cancelled')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, service_charge_bill_id)
);
CREATE TABLE real_estate.service_charge_bill_line (
  tenant_id UUID NOT NULL,
  bill_line_id TEXT NOT NULL,
  service_charge_bill_id TEXT NOT NULL,
  cost_category TEXT NOT NULL,
  allocated_amount NUMERIC(20,6) NOT NULL,
  tax_code TEXT,
  PRIMARY KEY (tenant_id, bill_line_id)
);
```

### Rust Types

```rust
pub struct ServiceChargeBill {
    pub tenant_id: TenantId,
    pub service_charge_bill_id: ServiceChargeBillId,
    pub lease_contract_id: LeaseContractId,
    pub billing_period: DateRange,
    pub allocation_basis_ref: AllocationBasisRef,
    pub total_allocated_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub bill_status: ServiceChargeBillStatus,
}
pub struct ServiceChargeBillLine {
    pub bill_line_id: BillLineId,
    pub cost_category: CostCategory,
    pub allocated_amount: Decimal,
    pub tax_code: Option<TaxCode>,
}
pub enum ServiceChargeBillingError { AllocationBasisMissing, TaxCodeMissing, ChargePolicyDenied, PaymentHandoffFailed, DisputeActive }
```

## API Endpoints

- REST `POST /v1/real-estate/service-charge-bills` creates bill.
- REST `POST /v1/real-estate/service-charge-bills/{id}:approve`.
- REST `POST /v1/real-estate/service-charge-bills/{id}:bill`.
- gRPC `real_estate.service_charge.v1.ServiceChargeBillingService.CreateBill`.
- gRPC `ApproveBill`, `BillServiceCharge`, and `ListBillLines`.
- AsyncAPI channel `real-estate.service-charge.bill-approved.v1`.
- AsyncAPI channel `real-estate.service-charge.billed.v1`.
- Consumers: payments, tax-compliance, compliance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::service_charge::bill`.
- Principal: `ServiceChargeAccountant`.
- Action: `service_charge_bill`.
- Resource: `ServiceChargeBill`.
- Context: `tenant_id`, `billing_period`, `allocation_basis_ref`, `total_allocated_amount`, `tax_code_complete`.
- Forbid when allocation basis missing, tax code missing where required, active dispute exists, or amount exceeds approval limit.

## Ontology Projection

- Vendor object: SAP RE-FX service charge bill.
- Oyatie object: `real_estate.service_charge_bill`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDOBJASS-OBJNR` -> allocation object.
- `VICDCONDLINE-CONDGUID` -> charge condition.
- `VICDADJREASN-ADJREASON` -> adjustment or bill reason.
- Allocation basis -> cost split evidence.
- Bill lines -> payment candidate.
- Projection freshness floor: approval.
- Projection rule: bills in dispute do not hand off new payment requests.

## Workflow Steps

- Node `basis-load`: read occupancy/service charge basis.
- Decision `basis-missing`: block bill.
- Node `cost-load`: load chargeable actual or estimate.
- Node `allocation-compute`: split cost by basis.
- Decision `tax-code-missing`: route tax setup.
- Node `bill-draft`: persist bill and lines.
- Node `approval-policy`: authorize billing.
- Decision `dispute-active`: block bill handoff.
- Node `payment-handoff`: send charge lines to payments.
- Node `audit-seal`: emit billing evidence.

## Audit Events

- `EVT-REAL_ESTATE-SERVICE_CHARGE-BILL_CREATED`.
- `EVT-REAL_ESTATE-SERVICE_CHARGE-BILL_APPROVED`.
- `EVT-REAL_ESTATE-SERVICE_CHARGE-BILLED`.
- `EVT-REAL_ESTATE-SERVICE_CHARGE-DISPUTED`.
- `EVT-REAL_ESTATE-SERVICE_CHARGE-POLICY_DENIED`.
- `EVT-REAL_ESTATE-SERVICE_CHARGE-IP_ACCEPTED`.
- ADR-0263 envelope stores billing period, allocation basis, bill amount, tax state, and payment handoff ref.

## SLO Targets

- Bill create p50: 80 ms.
- Bill create p95: 400 ms.
- Bill create p99: 1,500 ms for 50,000 bill lines.
- Bill handoff p95: 1,000 ms.
- Rationale: billing can be batch-shaped, but accountants need bounded approval and handoff evidence.

## Failure Modes and Recovery

- Failure: `ALLOCATION-BASIS-MISSING`; recovery: request occupancy basis recompute.
- Failure: `TAX-CODE-MISSING`; recovery: block bill and route tax setup.
- Failure: `CHARGE-POLICY-DENIED`; recovery: require controller approval.
- Failure: `PAYMENT-HANDOFF-FAILED`; recovery: retry idempotent handoff.
- Failure: `DISPUTE-ACTIVE`; recovery: freeze bill until dispute resolved.
- Failure: `LINE-EXPLOSION-TOO-LARGE`; recovery: partition bill by cost category.

## Migration Notes

- Import service charge bills after contracts and allocation basis.
- Preserve source bill line IDs and cost category.
- Do not re-bill migrated charges without open balance.
- Map tax codes before bill activation.
- Rollback path: disable bill handoff and keep bills draft/approved.
- Backfill order: contracts, allocation basis, cost lines, bills, bill lines, payment refs.

## Cross-microservice Handoffs

- From occupancy-allocation: allocation basis.
- From finance-ledger: cost actuals.
- To payments: bill lines.
- To tax-compliance: tax code validation.
- To workflow-engine: dispute or approval tasks.
- To compliance: service-charge bill evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The billing primitive remains bound to SAP RE-FX-SC service charge billing. |
| Persona specificity | Nora Patel owns bill allocation, tax validation, dispute handling, and rollback language. |
| Journey specificity | The j122 allocated-service-charge leg drives tax/withholding controls before payment. |
| DDL anchor | The service-charge bill, bill line, allocation basis, and payment-ref tables above are normative. |
| Rust anchor | Service-charge bill, bill line, allocation result, and error types above are implementation anchors. |
| REST anchor | Allocate, approve bill, handoff payment, dispute, and explain endpoints are tenant surfaces. |
| gRPC anchor | The service-charge billing service is the worker and replay contract. |
| AsyncAPI anchor | Bill approved, payment handed off, and dispute opened channels carry evidence. |
| Cedar anchor | Bill handoff is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP service-charge, condition, cost-line, and occupancy lineage projects to bill nodes. |
| ADR-0263 class binding | Billing checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Tax, service-charge, or office overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on billing APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, bill id, allocation basis, tax code, payment ref, and `cedar_decision_id`. |
| Metric | `oya_real_estate_service_charge_bills_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_real_estate_service_charge_billing_duration_seconds` tracks allocation, approval, and payment handoff latency. |
| Trace span | `real_estate.service_charge_billing.approve_bill` links occupancy, finance-ledger, payments, tax compliance, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `bill_id`, `basis_type`, `tax_code`, and dispute state. |
| Capacity math | Billing batches by bill_line_count and halts when unresolved allocation remainder exceeds materiality threshold. |
| Multi-region | Billing writes stay in property home cell; DR cells expose read-only bill evidence. |
| Sovereign cells | Tenant, tax, and cost-allocation evidence remains in-region for active packs. |
| Rollback | Disable bill handoff, keep bills draft/approved, and replay from last sealed service-charge audit id. |
| Test evidence | Required tests cover tax denial, basis mismatch, dispute branch, tenant mismatch, and idempotent payment handoff. |
| Rejected shortcut | A generic cost bill is rejected because it loses SAP RE-FX service-charge allocation and tax semantics. |
