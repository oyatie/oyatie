---
doc_class: ImplementationPlan
ip_id: IP-004
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
sap_submodule: RE-FX-RA (rent adjustment)
tenant_class: paid
billing_components:
  - per_usage
persona: Keiko Tanaka, rent administration analyst
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-004: Domain layer for rent schedule

## Context

- SAP submodule: RE-FX-RA rent adjustment and condition schedule.
- Persona: Keiko Tanaka, rent administration analyst.
- Journey leg: j122 recurring lease charges must be scheduled with tax and withholding context before payment batch.
- SAP tables: `VICDCONDLINE`, `VICDADJREASN`, `VICDCONTRACT`, `VICDOBJASS`.
- Oyatie aggregate: `RentSchedule`.
- Precedent: SAP RE-FX condition line schedule plus Oracle Lease Accounting payment schedule.
- ADR-0263 binds rent schedule audit and ADR-0297 gates adjustment authorization.
- Boundary: owns rent condition lines, adjustment reasons, and billing schedule; payment execution stays payments-owned.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.rent_schedule (
  tenant_id UUID NOT NULL,
  rent_schedule_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  condition_line_ref TEXT NOT NULL,
  schedule_status TEXT NOT NULL CHECK (schedule_status IN ('draft','active','superseded','cancelled')),
  currency_code TEXT NOT NULL,
  cadence TEXT NOT NULL CHECK (cadence IN ('monthly','quarterly','annual','one_time')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, rent_schedule_id)
);
CREATE TABLE real_estate.rent_schedule_line (
  tenant_id UUID NOT NULL,
  rent_schedule_id TEXT NOT NULL,
  line_no INTEGER NOT NULL,
  due_date DATE NOT NULL,
  amount NUMERIC(20,6) NOT NULL,
  tax_code TEXT,
  adjustment_reason_code TEXT,
  PRIMARY KEY (tenant_id, rent_schedule_id, line_no)
);
```

### Rust Types

```rust
pub struct RentSchedule {
    pub tenant_id: TenantId,
    pub rent_schedule_id: RentScheduleId,
    pub lease_contract_id: LeaseContractId,
    pub condition_line_ref: ConditionLineRef,
    pub schedule_status: RentScheduleStatus,
    pub currency_code: CurrencyCode,
    pub cadence: RentCadence,
}
pub struct RentScheduleLine {
    pub line_no: u32,
    pub due_date: NaiveDate,
    pub amount: Decimal,
    pub tax_code: Option<TaxCode>,
    pub adjustment_reason_code: Option<AdjustmentReasonCode>,
}
pub enum RentScheduleError { ContractInactive, ConditionMissing, CurrencyMismatch, AdjustmentReasonInvalid, NegativeRentDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/rent-schedules` creates schedule from contract condition.
- REST `POST /v1/real-estate/rent-schedules/{id}:activate`.
- REST `POST /v1/real-estate/rent-schedules/{id}:adjust`.
- gRPC `real_estate.rent_schedule.v1.RentScheduleService.CreateRentSchedule`.
- gRPC `ActivateRentSchedule`, `AdjustRentSchedule`, and `ListDueRentLines`.
- AsyncAPI channel `real-estate.rent-schedule.activated.v1`.
- AsyncAPI channel `real-estate.rent-schedule.adjusted.v1`.
- Consumers: payments, finops-portal, lease-accounting, tax-compliance.

## Cedar Policy Hooks

- Policy: `real_estate::rent_schedule::adjust`.
- Principal: `RentAdministrator`.
- Action: `rent_schedule_adjust`.
- Resource: `RentSchedule`.
- Context: `tenant_id`, `lease_contract_id`, `adjustment_reason_code`, `amount_delta`, `currency_code`.
- Forbid when contract inactive, adjustment reason missing, negative rent not allowed by pack, or currency differs from contract.

## Ontology Projection

- Vendor object: SAP RE-FX condition line.
- Oyatie object: `real_estate.rent_schedule`.
- `VICDCONDLINE-CONDGUID` -> `condition_line_ref`.
- `VICDADJREASN-ADJREASON` -> `adjustment_reason_code`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDOBJASS-OBJNR` -> premises assignment lineage.
- Cadence -> schedule recurrence.
- Due line -> billable payment event candidate.
- Projection freshness floor: 5 seconds.
- Projection rule: superseded schedules remain linked for audit and accounting reconciliation.

## Workflow Steps

- Node `contract-read`: verify active lease contract.
- Node `condition-read`: load rent condition line.
- Decision `condition-missing`: reject schedule.
- Node `schedule-generate`: create due lines.
- Decision `currency-mismatch`: require amendment.
- Node `policy-evaluate`: validate adjustment and activation.
- Decision `adjustment-reason-invalid`: block adjustment.
- Node `schedule-activate`: mark billable.
- Node `payment-handoff`: send due lines to payments.
- Node `audit-seal`: emit rent schedule evidence.

## Audit Events

- `EVT-REAL_ESTATE-RENT_SCHEDULE-CREATED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE-ACTIVATED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE-ADJUSTED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE-SUPERSEDED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE-POLICY_DENIED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE-IP_ACCEPTED`.
- ADR-0263 envelope stores `condition_line_ref`, due line count, currency, and adjustment reason.

## SLO Targets

- Schedule create p50: 60 ms.
- Schedule create p95: 220 ms.
- Schedule create p99: 650 ms.
- Due-line list p95: 250 ms for 10,000 lines.
- Rationale: rent admin edits are interactive; large due-line reads support batch payment preparation.

## Failure Modes and Recovery

- Failure: `CONTRACT-INACTIVE`; recovery: reject schedule and route to lease admin.
- Failure: `CONDITION-MISSING`; recovery: block activation until condition line exists.
- Failure: `CURRENCY-MISMATCH`; recovery: require contract amendment.
- Failure: `ADJUSTMENT-REASON-INVALID`; recovery: reject and show allowed reason codes.
- Failure: `NEGATIVE-RENT-DENIED`; recovery: route to credit memo workflow.
- Failure: `PAYMENT-HANDOFF-FAILED`; recovery: keep schedule active and retry outbox.

## Migration Notes

- Import `VICDCONDLINE` after contracts.
- Import adjustment reasons before adjusted schedules.
- Preserve SAP condition GUID as source lineage.
- Do not activate migrated schedule lines with missing tax code when pack requires it.
- Rollback path: disable activation and adjustment commands while keeping read-only schedules.
- Backfill order: contracts, conditions, adjustment reasons, schedules, due lines.

## Cross-microservice Handoffs

- From lease-contract: active contract and term.
- To payments: due rent line candidates.
- To finops-portal: cash forecast.
- To tax-compliance: tax and withholding code.
- To lease-accounting: payment schedule for liability model.
- To compliance: adjustment and activation audit evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP RE-FX-RA rent adjustment. |
| Persona specificity | Keiko Tanaka owns due-line scheduling, tax/withholding checks, and rollback language. |
| Journey specificity | The j122 vendor-payment-batch leg drives due rent, tax, withholding, and adjustment audit behavior. |
| DDL anchor | The rent schedule, due line, condition, and adjustment tables above are normative. |
| Rust anchor | Rent schedule, due line, adjustment, and error types above are implementation anchors. |
| REST anchor | Create schedule, activate, adjust, and generate due lines endpoints are tenant surfaces. |
| gRPC anchor | The rent schedule service is the worker and replay contract. |
| AsyncAPI anchor | Schedule activated, due line created, and adjustment posted channels carry payment evidence. |
| Cedar anchor | Schedule activation is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP condition and adjustment-reason lineage projects to rent schedule and due-line nodes. |
| ADR-0263 class binding | Rent policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Tax, withholding, or lease-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on rent APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, lease contract id, condition ref, due date, tax code, and `cedar_decision_id`. |
| Metric | `oya_real_estate_rent_schedule_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_rent_schedule_duration_seconds` tracks activation and due-line generation latency. |
| Trace span | `real_estate.rent_schedule.activate` links lease contract, tax compliance, payments, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `schedule_id`, `condition_ref`, `tax_code`, and due-line count. |
| Capacity math | Due-line generation batches by schedule_count * periods_remaining and backpressures before payment cutoff risk. |
| Multi-region | Rent schedule writes stay in lease home cell; DR cells serve read-only due-line projections. |
| Sovereign cells | Counterparty, tax, and withholding evidence remains in-region for active regulated packs. |
| Rollback | Disable activation and adjustment commands, keep read-only schedules, and replay from last sealed rent audit id. |
| Test evidence | Required tests cover missing tax code, inactive contract, adjustment reason mismatch, tenant mismatch, and idempotent due-line replay. |
| Rejected shortcut | A generic `BillingSchedule` is rejected because it loses SAP RE-FX rent adjustment and condition-line semantics. |
