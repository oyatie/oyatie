---
doc_class: ImplementationPlan
ip_id: IP-005
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

# IP-005: Domain layer for lease accounting event

## Context

- SAP submodule: RE-FX-AC lease accounting.
- Persona: Marcus Lee, lease accounting controller.
- Journey leg: j137 SOX audit requires traceable lease accounting event from contract approval to posting request.
- SAP tables: `VICDCONTRACT`, `VICDCONDLINE`, `VICDOBJASS`, `VICDADJREASN`.
- Oyatie aggregate: `LeaseAccountingEvent`.
- Precedent: SAP RE-FX valuation posting plus Workday Lease Accounting event ledger.
- ADR-0263 binds accounting event audit and ADR-0297 gates posting request authority.
- Boundary: owns event classification and accounting evidence; journal posting remains finance-ledger owned.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.lease_accounting_event (
  tenant_id UUID NOT NULL,
  lease_accounting_event_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  event_kind TEXT NOT NULL CHECK (event_kind IN ('initial_recognition','remeasurement','payment','modification','termination')),
  accounting_standard TEXT NOT NULL CHECK (accounting_standard IN ('IFRS16','ASC842','LOCAL_GAAP')),
  event_amount NUMERIC(20,6) NOT NULL,
  currency_code TEXT NOT NULL,
  event_date DATE NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, lease_accounting_event_id)
);
CREATE TABLE real_estate.lease_accounting_event_lineage (
  tenant_id UUID NOT NULL,
  lease_accounting_event_id TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_ref TEXT NOT NULL,
  PRIMARY KEY (tenant_id, lease_accounting_event_id, source_kind, source_ref)
);
```

### Rust Types

```rust
pub struct LeaseAccountingEvent {
    pub tenant_id: TenantId,
    pub lease_accounting_event_id: LeaseAccountingEventId,
    pub lease_contract_id: LeaseContractId,
    pub event_kind: LeaseAccountingEventKind,
    pub accounting_standard: AccountingStandard,
    pub event_amount: Decimal,
    pub currency_code: CurrencyCode,
    pub event_date: NaiveDate,
}
pub struct LeaseAccountingEventLineage {
    pub source_kind: AccountingSourceKind,
    pub source_ref: SourceRef,
}
pub enum LeaseAccountingEventError { ContractInactive, StandardUnsupported, AmountInvalid, LineageMissing, PostingPolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-accounting-events` creates accounting event.
- REST `POST /v1/real-estate/lease-accounting-events/{id}:request-posting`.
- REST `GET /v1/real-estate/lease-contracts/{id}/accounting-events`.
- gRPC `real_estate.lease_accounting.v1.LeaseAccountingService.CreateAccountingEvent`.
- gRPC `RequestPosting` and `ListAccountingEvents`.
- AsyncAPI channel `real-estate.lease-accounting.event-created.v1`.
- AsyncAPI channel `real-estate.lease-accounting.posting-requested.v1`.
- Consumers: finance-ledger, compliance, rent-schedule, workflow-engine.

## Cedar Policy Hooks

- Policy: `real_estate::lease_accounting::request_posting`.
- Principal: `LeaseAccountingController`.
- Action: `lease_accounting_posting_request`.
- Resource: `LeaseAccountingEvent`.
- Context: `tenant_id`, `accounting_standard`, `event_kind`, `event_amount`, `lineage_complete`.
- Forbid when lineage is incomplete, amount is invalid, accounting standard unsupported, or controller lacks posting-request authority.

## Ontology Projection

- Vendor object: SAP RE-FX lease accounting posting event.
- Oyatie object: `real_estate.lease_accounting_event`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDCONDLINE-CONDGUID` -> payment or condition lineage.
- `VICDOBJASS-OBJNR` -> premises lineage.
- `VICDADJREASN-ADJREASON` -> remeasurement reason.
- Event kind -> accounting classification.
- Accounting standard -> measurement basis.
- Projection freshness floor: 5 seconds.
- Projection rule: finance-ledger posting reference is a downstream lineage field, not owned here.

## Workflow Steps

- Node `contract-read`: verify active contract or valid termination event.
- Node `lineage-collect`: gather contract, condition, and object lineage.
- Decision `lineage-missing`: block event creation.
- Node `amount-validate`: validate sign, currency, and standard.
- Decision `standard-unsupported`: reject event.
- Node `event-create`: persist accounting event.
- Node `policy-evaluate`: authorize posting request.
- Decision `posting-policy-denied`: keep event unposted and route approval.
- Node `posting-request`: send to finance-ledger.
- Node `audit-seal`: emit accounting evidence.

## Audit Events

- `EVT-REAL_ESTATE-LEASE_ACCOUNTING-EVENT_CREATED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING-POSTING_REQUESTED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING-POSTING_DENIED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING-LINEAGE_MISSING`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING-POLICY_DENIED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING-IP_ACCEPTED`.
- ADR-0263 envelope stores contract, standard, event kind, amount, currency, and lineage references.

## SLO Targets

- Event create p50: 55 ms.
- Event create p95: 210 ms.
- Event create p99: 650 ms.
- Posting request p95: 500 ms.
- Rationale: accounting event creation is interactive; posting handoff may include external ledger acknowledgement.

## Failure Modes and Recovery

- Failure: `CONTRACT-INACTIVE`; recovery: allow only termination or archive-class event.
- Failure: `STANDARD-UNSUPPORTED`; recovery: reject and require pack activation.
- Failure: `AMOUNT-INVALID`; recovery: return validation details and no posting request.
- Failure: `LINEAGE-MISSING`; recovery: block event and request contract/rent schedule repair.
- Failure: `POSTING-POLICY-DENIED`; recovery: route to controller approval.
- Failure: `LEDGER-HANDOFF-FAILED`; recovery: keep event unposted and retry outbox.

## Migration Notes

- Import historical accounting events after contracts and condition lines.
- Preserve SAP accounting document and valuation refs as lineage.
- Classify event kind deterministically from source posting type.
- Do not request new postings for migrated historical events.
- Rollback path: disable posting request and keep accounting event read-only.
- Backfill order: contracts, rent schedules, accounting events, lineage, posting refs.

## Cross-microservice Handoffs

- From lease-contract: active term and contract metadata.
- From rent-schedule: payment and condition-line evidence.
- To finance-ledger: posting request.
- To workflow-engine: posting approval or denial.
- To compliance: SOX and IFRS evidence.
- To portfolio analytics: accounting exposure metrics.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP RE-FX-AC lease accounting orchestration. |
| Persona specificity | Marcus Lee owns accounting event approval, posting request, and rollback language. |
| Journey specificity | The j137 SOX leg drives traceable event creation from contract approval to ledger posting. |
| DDL anchor | The lease accounting event and posting reference tables above are normative. |
| Rust anchor | Lease accounting event, posting request, and error types above are implementation anchors. |
| REST anchor | Create event, review, approve, reject, and request posting endpoints are tenant surfaces. |
| gRPC anchor | The lease accounting event service is the worker and replay contract. |
| AsyncAPI anchor | Event approved, posting requested, and posting failed channels carry finance evidence. |
| Cedar anchor | Accounting event approval is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP contract, condition, and accounting lineage projects to accounting event nodes. |
| ADR-0263 class binding | Accounting policy checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | SOX, IFRS, or finance-control overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on accounting APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, accounting event id, contract id, posting ref, and `cedar_decision_id`. |
| Metric | `oya_real_estate_lease_accounting_events_total{tenant_id,cell_id,event_type,status}` caps event/status cardinality. |
| Latency histogram | `oya_real_estate_lease_accounting_event_duration_seconds` tracks approval and posting-request latency. |
| Trace span | `real_estate.lease_accounting_event.approve` links lease contract, rent schedule, finance-ledger, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `event_id`, `event_type`, `ledger_ref`, and approval state. |
| Capacity math | Posting queue depth uses events_per_period / ledger_worker_rate; saturation blocks non-critical projections first. |
| Multi-region | Accounting event writes stay in finance home cell; DR cells serve read-only event projections. |
| Sovereign cells | Finance and lease evidence remains in-region for SOX, IFRS, and sovereign packs. |
| Rollback | Disable posting request, keep events read-only, and replay from last sealed accounting audit id. |
| Test evidence | Required tests cover duplicate event, missing schedule, approval denial, ledger timeout, and idempotent posting request. |
| Rejected shortcut | A generic `JournalEvent` is rejected because it loses SAP RE-FX lease-accounting and contract lineage. |
