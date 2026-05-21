---
doc_class: ImplementationPlan
ip_id: IP-011
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

# IP-011: Usecase layer for lease accounting event

## Context

- SAP submodule: RE-FX-AC lease accounting orchestration.
- Persona: Marcus Lee, lease accounting controller.
- Journey leg: j137 lease accounting event is created, reviewed, and handed to finance-ledger with SOX evidence.
- SAP tables: `VICDCONTRACT`, `VICDCONDLINE`, `VICDOBJASS`, `VICDADJREASN`.
- Oyatie usecase: `OperateLeaseAccountingEvent`.
- Precedent: SAP RE-FX valuation posting orchestration plus Workday accounting journal review workflow.
- ADR-0105 keeps posting request orchestration out of domain and ADR-0263 records every posting attempt.
- Boundary: coordinates event creation, review, posting request, and retry; ledger journal ownership stays finance-ledger.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.lease_accounting_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  lease_accounting_event_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('create','review','request_posting','reverse_request','retry_posting')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE real_estate.lease_posting_handoff (
  tenant_id UUID NOT NULL,
  posting_handoff_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  finance_ledger_ref TEXT,
  handoff_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, posting_handoff_id)
);
```

### Rust Types

```rust
pub struct LeaseAccountingCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub lease_accounting_event_id: LeaseAccountingEventId,
    pub command_kind: LeaseAccountingCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct LeasePostingHandoff {
    pub posting_handoff_id: HandoffId,
    pub command_id: CommandId,
    pub finance_ledger_ref: Option<FinanceLedgerRef>,
    pub handoff_state: HandoffState,
}
pub enum OperateLeaseAccountingEventError { ReviewRequired, PostingDenied, LedgerUnavailable, DuplicatePayload, ReversalPolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-accounting-events/{id}:operate`.
- REST `POST /v1/real-estate/lease-accounting-commands/{command_id}:retry-posting`.
- REST `GET /v1/real-estate/lease-accounting-commands/{command_id}`.
- gRPC `real_estate.accounting_usecase.v1.OperateLeaseAccountingEvent`.
- gRPC `RetryLeasePosting` and `GetLeaseAccountingCommand`.
- AsyncAPI channel `real-estate.lease-accounting.command-succeeded.v1`.
- AsyncAPI channel `real-estate.lease-posting.handoff-failed.v1`.
- Consumers: finance-ledger, workflow-engine, compliance, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::lease_accounting_command::operate`.
- Principal: `LeaseAccountingController`.
- Action: `lease_accounting_command_execute`.
- Resource: `LeaseAccountingEvent`.
- Context: `tenant_id`, `command_kind`, `review_state`, `event_kind`, `amount`, `standard`.
- Forbid when review is required but absent, posting is denied by pack, reversal is outside close window, or duplicate payload conflicts.

## Ontology Projection

- Vendor object: SAP RE-FX lease accounting command.
- Oyatie object: `real_estate.lease_accounting_command`.
- `VICDCONTRACT-CONTRACT` -> lease contract lineage.
- `VICDCONDLINE-CONDGUID` -> condition/payment lineage.
- `VICDOBJASS-OBJNR` -> premises lineage.
- `VICDADJREASN-ADJREASON` -> remeasurement reason.
- Command kind -> accounting event transition.
- Posting handoff -> finance-ledger consumer evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: failed posting handoffs remain retryable without duplicating accounting event.

## Workflow Steps

- Node `command-accept`: dedupe command.
- Node `event-load`: read accounting event and lineage.
- Decision `review-required`: route to workflow approval.
- Decision `posting-denied`: block posting request.
- Node `domain-apply`: update event review or posting-request state.
- Node `ledger-handoff`: send posting request to finance-ledger.
- Decision `ledger-unavailable`: mark handoff retryable.
- Node `posting-ack`: record ledger ref when received.
- Node `analytics-publish`: expose accounting exposure.
- Node `audit-seal`: emit command evidence.

## Audit Events

- `EVT-REAL_ESTATE-LEASE_ACCOUNTING_COMMAND-ACCEPTED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING_COMMAND-REVIEWED`.
- `EVT-REAL_ESTATE-LEASE_POSTING-REQUESTED`.
- `EVT-REAL_ESTATE-LEASE_POSTING-HANDOFF_FAILED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING_COMMAND-POLICY_DENIED`.
- `EVT-REAL_ESTATE-LEASE_ACCOUNTING_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores command kind, review state, event amount, and ledger handoff ref.

## SLO Targets

- Command accept p50: 40 ms.
- Command accept p95: 150 ms.
- Command accept p99: 420 ms.
- Posting handoff p95: 1,000 ms.
- Rationale: controller command receipts are interactive; ledger handoff may cross accounting service boundary.

## Failure Modes and Recovery

- Failure: `REVIEW-REQUIRED`; recovery: create workflow approval and keep event unposted.
- Failure: `POSTING-DENIED`; recovery: capture policy denial and require controller action.
- Failure: `LEDGER-UNAVAILABLE`; recovery: retry handoff with idempotency key.
- Failure: `DUPLICATE-PAYLOAD`; recovery: return prior command result.
- Failure: `REVERSAL-POLICY-DENIED`; recovery: block reversal after close window.
- Failure: `ANALYTICS-PUBLISH-FAILED`; recovery: retry analytics without rolling back posting state.

## Migration Notes

- Convert pending SAP valuation postings into commands only if not posted.
- Import historical postings as immutable accounting event lineage.
- Preserve source posting document ID and valuation area.
- Do not replay postings during migration.
- Rollback path: disable operate endpoint and retain event read-only state.
- Backfill order: accounting events, commands, review state, posting refs.

## Cross-microservice Handoffs

- From workflow-engine: review and approval state.
- To finance-ledger: posting request.
- To portfolio analytics: exposure and liability metrics.
- To compliance: SOX and IFRS event evidence.
- To rent-schedule: payment-related lineage.
- To lease-contract: modification and termination state.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP RE-FX-AC lease accounting event review and posting handoff. |
| Persona specificity | Marcus Lee owns review, approval, ledger handoff, and rollback acceptance language. |
| Journey specificity | The j137 SOX lease accounting leg drives event creation, review, and finance-ledger evidence. |
| DDL anchor | The accounting event command, review state, and posting reference tables above are normative. |
| Rust anchor | Accounting event command, review result, posting reference, and error types above are anchors. |
| REST anchor | Operate, review, approve, reject, and replay endpoints are tenant surfaces. |
| gRPC anchor | The accounting event usecase service is the worker and replay contract. |
| AsyncAPI anchor | Event reviewed, posting requested, and posting reconciled channels carry finance evidence. |
| Cedar anchor | Review and approval commands are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP accounting lineage projects to event command and posting-reference nodes. |
| ADR-0263 class binding | Accounting review checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | SOX, IFRS, or finance-control overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on accounting usecase APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, event id, review state, posting ref, and `cedar_decision_id`. |
| Metric | `oya_real_estate_accounting_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_accounting_usecase_duration_seconds` tracks review and posting handoff latency. |
| Trace span | `real_estate.lease_accounting_event.operate` links workflow, finance-ledger, rent schedule, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `event_id`, `review_state`, `posting_ref`, and liability bucket. |
| Capacity math | Review queue capacity uses pending_events / controller_review_rate and blocks postings above SLA risk. |
| Multi-region | Accounting writes stay in finance home cell; DR cells expose read-only event projections. |
| Sovereign cells | Lease accounting and posting evidence remains in-region for SOX, IFRS, and sovereign packs. |
| Rollback | Disable operate endpoint, retain event read-only state, and replay from last sealed accounting usecase audit id. |
| Test evidence | Required tests cover approval denial, posting retry, tenant mismatch, duplicate event, and replay idempotency. |
| Rejected shortcut | A generic ledger handoff is rejected because it loses SAP RE-FX lease-accounting review semantics. |
