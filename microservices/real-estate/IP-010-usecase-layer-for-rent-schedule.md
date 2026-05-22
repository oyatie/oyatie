---
doc_class: ImplementationPlan
ip_id: IP-010
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

# IP-010: Usecase layer for rent schedule

## Context

- SAP submodule: RE-FX-RA rent adjustment orchestration.
- Persona: Keiko Tanaka, rent administration analyst.
- Journey leg: j122 due rent lines are prepared for payment batch with tax, withholding, and adjustment audit trail.
- SAP tables: `VICDCONDLINE`, `VICDADJREASN`, `VICDCONTRACT`, `VICDOBJASS`.
- Oyatie usecase: `OperateRentSchedule`.
- Precedent: SAP RE-FX condition adjustment plus Oracle payment schedule generation.
- ADR-0105 keeps schedule orchestration in usecase layer and ADR-0263 binds payment-line evidence.
- Boundary: coordinates schedule activation, adjustment, payment handoff, and accounting evidence.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.rent_schedule_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  rent_schedule_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('create','activate','adjust','cancel','prepare_payment')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE real_estate.rent_payment_handoff (
  tenant_id UUID NOT NULL,
  payment_handoff_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  due_line_refs JSONB NOT NULL,
  handoff_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, payment_handoff_id)
);
```

### Rust Types

```rust
pub struct RentScheduleCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub rent_schedule_id: RentScheduleId,
    pub command_kind: RentScheduleCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct RentPaymentHandoff {
    pub payment_handoff_id: HandoffId,
    pub command_id: CommandId,
    pub due_line_refs: Vec<RentDueLineRef>,
    pub handoff_state: HandoffState,
}
pub enum OperateRentScheduleError { ScheduleInactive, AdjustmentDenied, TaxCodeMissing, PaymentHandoffFailed, DuplicatePayload }
```

## API Endpoints

- REST `POST /v1/real-estate/rent-schedules/{id}:operate`.
- REST `POST /v1/real-estate/rent-schedules/{id}:prepare-payment`.
- REST `GET /v1/real-estate/rent-schedule-commands/{command_id}`.
- gRPC `real_estate.rent_usecase.v1.OperateRentSchedule`.
- gRPC `PrepareRentPayment` and `GetRentScheduleCommand`.
- AsyncAPI channel `real-estate.rent-schedule.command-succeeded.v1`.
- AsyncAPI channel `real-estate.rent-payment.handoff-created.v1`.
- Consumers: payments, tax-compliance, lease-accounting, finops-portal.

## Cedar Policy Hooks

- Policy: `real_estate::rent_schedule_command::operate`.
- Principal: `RentAdministrator`.
- Action: `rent_schedule_command_execute`.
- Resource: `RentSchedule`.
- Context: `tenant_id`, `command_kind`, `adjustment_reason_code`, `tax_code_present`, `payment_window`.
- Forbid when adjustment reason is invalid, tax code missing for payment, schedule inactive, or duplicate payload conflicts.

## Ontology Projection

- Vendor object: SAP RE-FX rent condition command.
- Oyatie object: `real_estate.rent_schedule_command`.
- `VICDCONDLINE-CONDGUID` -> `rent_schedule_id`.
- `VICDADJREASN-ADJREASON` -> adjustment reason.
- `VICDCONTRACT-CONTRACT` -> lease contract lineage.
- `VICDOBJASS-OBJNR` -> object assignment lineage.
- Command kind -> schedule state transition.
- Payment handoff -> due-line consumer evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: payment handoff retries do not duplicate due lines.

## Workflow Steps

- Node `command-accept`: dedupe schedule command.
- Node `schedule-load`: read schedule and due lines.
- Decision `schedule-inactive`: reject payment preparation.
- Decision `tax-code-missing`: block payment handoff.
- Node `adjustment-policy`: validate reason and amount.
- Decision `adjustment-denied`: keep schedule unchanged.
- Node `domain-apply`: activate or adjust schedule.
- Node `payment-handoff`: send due lines to payments.
- Decision `handoff-failed`: keep retryable handoff.
- Node `audit-seal`: emit schedule command event.

## Audit Events

- `EVT-REAL_ESTATE-RENT_SCHEDULE_COMMAND-ACCEPTED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE_COMMAND-SUCCEEDED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE_COMMAND-ADJUSTMENT_DENIED`.
- `EVT-REAL_ESTATE-RENT_PAYMENT-HANDOFF_CREATED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE_COMMAND-POLICY_DENIED`.
- `EVT-REAL_ESTATE-RENT_SCHEDULE_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores command kind, adjustment reason, tax code state, and due-line refs.

## SLO Targets

- Command accept p50: 35 ms.
- Command accept p95: 130 ms.
- Command accept p99: 360 ms.
- Payment handoff p95: 800 ms for 5,000 due lines.
- Rationale: rent admins need fast receipts; payment handoff may batch due-line payloads.

## Failure Modes and Recovery

- Failure: `SCHEDULE-INACTIVE`; recovery: block payment prep and return schedule state.
- Failure: `ADJUSTMENT-DENIED`; recovery: preserve prior schedule and emit denial event.
- Failure: `TAX-CODE-MISSING`; recovery: route to tax-compliance setup.
- Failure: `PAYMENT-HANDOFF-FAILED`; recovery: retry outbox without duplicating due lines.
- Failure: `DUPLICATE-PAYLOAD`; recovery: return prior command.
- Failure: `ACCOUNTING-SYNC-FAILED`; recovery: retry accounting event handoff.

## Migration Notes

- Import open rent adjustments as commands only when not posted.
- Import historical schedule changes as immutable evidence.
- Compute idempotency keys from source condition and change number.
- Do not replay historical payment handoffs.
- Rollback path: disable operate and payment prep endpoints.
- Backfill order: schedules, due lines, commands, payment handoffs.

## Cross-microservice Handoffs

- From lease-contract: active contract and condition state.
- To payments: payment handoff and due lines.
- To tax-compliance: tax and withholding checks.
- To lease-accounting: adjusted payment schedule.
- To finops-portal: cash forecast.
- To compliance: schedule command evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP RE-FX-RA rent adjustment orchestration. |
| Persona specificity | Keiko Tanaka owns due-line preparation, tax checks, and rollback acceptance language. |
| Journey specificity | The j122 payment-batch leg drives due-line, withholding, adjustment, and cash-forecast behavior. |
| DDL anchor | The rent command, due-line, and payment-handoff tables above are normative. |
| Rust anchor | Rent schedule command, due-line result, and error types above are implementation anchors. |
| REST anchor | Operate, prepare payment, adjust, and replay endpoints are tenant surfaces. |
| gRPC anchor | The rent schedule usecase service is the worker and replay contract. |
| AsyncAPI anchor | Due line prepared, payment handed off, and adjustment posted channels carry downstream evidence. |
| Cedar anchor | Rent operations are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP rent adjustment and condition-line lineage projects to due-line command nodes. |
| ADR-0263 class binding | Rent operation checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Tax, withholding, or payment overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on rent usecase APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, schedule id, due line id, tax code, withholding code, and `cedar_decision_id`. |
| Metric | `oya_real_estate_rent_schedule_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_rent_schedule_usecase_duration_seconds` tracks due-line and handoff latency. |
| Trace span | `real_estate.rent_schedule.prepare_payment` links lease contract, payments, tax compliance, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `schedule_id`, `due_line_id`, `tax_code`, and handoff status. |
| Capacity math | Payment prep batches by due-line count and withholds when tax validation latency threatens batch cutoff. |
| Multi-region | Payment-prep writes stay in lease home cell; DR cells expose read-only due-line projections. |
| Sovereign cells | Tax and counterparty evidence remains in-region for active regulated packs. |
| Rollback | Disable operate and payment-prep endpoints, keep schedules read-only, and replay from last sealed schedule audit id. |
| Test evidence | Required tests cover tax denial, withholding missing, inactive contract, tenant mismatch, and idempotent payment handoff. |
| Rejected shortcut | A generic payment schedule is rejected because it loses SAP RE-FX rent adjustment and tax/withholding semantics. |
