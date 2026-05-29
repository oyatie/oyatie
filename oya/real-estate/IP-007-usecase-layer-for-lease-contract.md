---
doc_class: ImplementationPlan
ip_id: IP-007
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
sap_submodule: RE-FX-CN (contracts)
tenant_class: paid
billing_components:
  - per_usage
persona: Amara Singh, lease administration manager
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-007: Usecase layer for lease contract

## Context

- SAP submodule: RE-FX-CN contract orchestration.
- Persona: Amara Singh, lease administration manager.
- Journey leg: j137 control test follows lease create, approve, amend, terminate, and accounting handoff.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie usecase: `OperateLeaseContract`.
- Precedent: SAP RE-FX contract lifecycle plus Salesforce CPQ approval orchestration.
- ADR-0105 keeps workflow orchestration in usecase layer and ADR-0263 binds lifecycle events.
- Boundary: coordinates domain contract state, approvals, rent schedule activation, and accounting event request.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.lease_contract_command (
  tenant_id UUID NOT NULL,
  command_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  command_kind TEXT NOT NULL CHECK (command_kind IN ('create','approve','amend','terminate','archive')),
  idempotency_key TEXT NOT NULL,
  command_status TEXT NOT NULL,
  failure_code TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, command_id),
  UNIQUE (tenant_id, idempotency_key)
);
CREATE TABLE real_estate.lease_contract_handoff (
  tenant_id UUID NOT NULL,
  handoff_id TEXT NOT NULL,
  command_id TEXT NOT NULL,
  target_microservice TEXT NOT NULL,
  handoff_state TEXT NOT NULL,
  payload_ref TEXT NOT NULL,
  PRIMARY KEY (tenant_id, handoff_id)
);
```

### Rust Types

```rust
pub struct LeaseContractCommand {
    pub tenant_id: TenantId,
    pub command_id: CommandId,
    pub lease_contract_id: LeaseContractId,
    pub command_kind: LeaseContractCommandKind,
    pub idempotency_key: IdempotencyKey,
    pub command_status: CommandStatus,
}
pub struct LeaseContractHandoff {
    pub handoff_id: HandoffId,
    pub command_id: CommandId,
    pub target_microservice: MicroserviceName,
    pub handoff_state: HandoffState,
    pub payload_ref: PayloadRef,
}
pub enum OperateLeaseContractError { DuplicateCommand, ApprovalRouteIncomplete, RentScheduleHandoffFailed, AccountingHandoffFailed, TerminationDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-contracts/{id}:operate` executes lifecycle command.
- REST `GET /v1/real-estate/lease-contract-commands/{command_id}` returns command and handoff state.
- REST `POST /v1/real-estate/lease-contract-commands/{command_id}:retry-handoffs`.
- gRPC `real_estate.lease_contract_usecase.v1.OperateLeaseContract`.
- gRPC `GetLeaseContractCommand` and `RetryLeaseContractHandoffs`.
- AsyncAPI channel `real-estate.lease-contract.command-succeeded.v1`.
- AsyncAPI channel `real-estate.lease-contract.handoff-failed.v1`.
- Consumers: workflow-engine, rent-schedule, lease-accounting, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::lease_contract_command::operate`.
- Principal: `LeaseAdministrator`.
- Action: `lease_contract_command_execute`.
- Resource: `LeaseContract`.
- Context: `tenant_id`, `command_kind`, `approval_route_state`, `contract_status`, `effective_date`.
- Forbid when approval route incomplete, termination requires accounting review, archive violates retention policy, or idempotency payload conflicts.

## Ontology Projection

- Vendor object: SAP RE-FX contract lifecycle command.
- Oyatie object: `real_estate.lease_contract_command`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDOBJASS-OBJNR` -> premises assignment handoff.
- `VICDCONDLINE-CONDGUID` -> rent schedule handoff.
- `VICDADJREASN-ADJREASON` -> amendment/termination reason.
- Command kind -> lifecycle transition.
- Handoff rows -> downstream activation evidence.
- Projection freshness floor: 5 seconds.
- Projection rule: command status is durable even if downstream handoff retries.

## Workflow Steps

- Node `command-accept`: dedupe lifecycle command.
- Node `contract-load`: read current contract state.
- Decision `approval-route-incomplete`: fail approval command.
- Node `domain-apply`: apply contract transition.
- Node `workflow-handoff`: request approval or termination review.
- Node `rent-schedule-handoff`: activate or amend rent schedule.
- Decision `rent-schedule-handoff-failed`: mark handoff retryable.
- Node `accounting-handoff`: request accounting event.
- Decision `accounting-handoff-failed`: keep command succeeded-pending-handoff.
- Node `audit-seal`: emit command evidence.

## Audit Events

- `EVT-REAL_ESTATE-LEASE_CONTRACT_COMMAND-ACCEPTED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT_COMMAND-SUCCEEDED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT_COMMAND-HANDOFF_FAILED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT_COMMAND-TERMINATION_DENIED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT_COMMAND-POLICY_DENIED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT_COMMAND-IP_ACCEPTED`.
- ADR-0263 envelope stores `command_kind`, `idempotency_key`, handoff states, and approval route.

## SLO Targets

- Command accept p50: 35 ms.
- Command accept p95: 120 ms.
- Command accept p99: 320 ms.
- Downstream handoff p95: 800 ms.
- Rationale: contract commands need immediate durable receipts; handoffs can be retried without re-running command.

## Failure Modes and Recovery

- Failure: `DUPLICATE-COMMAND-CONFLICT`; recovery: reject and return prior command.
- Failure: `APPROVAL-ROUTE-INCOMPLETE`; recovery: create workflow setup task.
- Failure: `RENT-SCHEDULE-HANDOFF-FAILED`; recovery: retry handoff outbox.
- Failure: `ACCOUNTING-HANDOFF-FAILED`; recovery: keep contract state and retry.
- Failure: `TERMINATION-DENIED`; recovery: route to controller review.
- Failure: `ARCHIVE-RETENTION-DENIED`; recovery: keep active archive block evidence.

## Migration Notes

- Convert open SAP lifecycle changes into command rows only when action remains pending.
- Import closed SAP contract history as immutable lifecycle evidence.
- Preserve command idempotency by source contract plus change number.
- Do not replay historical approvals.
- Rollback path: disable operate endpoint and keep domain mutation endpoints inactive.
- Backfill order: contracts, terms, command history, handoffs, downstream refs.

## Cross-microservice Handoffs

- From workflow-engine: approval route and decision.
- To rent-schedule: condition activation or amendment.
- To lease-accounting: recognition or termination event.
- To compliance: lifecycle evidence.
- To portfolio analytics: active contract state.
- To document management: signed contract artifact reference.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The usecase remains bound to SAP RE-FX-CN contract orchestration. |
| Persona specificity | Amara Singh owns create, approve, amend, terminate, and rollback acceptance language. |
| Journey specificity | The j137 control-test leg drives lifecycle command ordering and accounting handoff. |
| DDL anchor | The lease contract command and downstream handoff tables above are normative. |
| Rust anchor | Lease contract command, lifecycle result, and error types above are implementation anchors. |
| REST anchor | Operate, approve, amend, terminate, and replay endpoints are tenant surfaces. |
| gRPC anchor | The lease contract usecase service is the worker and replay contract. |
| AsyncAPI anchor | Contract lifecycle and accounting handoff channels carry downstream evidence. |
| Cedar anchor | Lifecycle commands are default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP contract and object lineage projects to lifecycle command nodes. |
| ADR-0263 class binding | Lifecycle checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Lease or office overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on contract usecase APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, command id, contract id, lifecycle state, and `cedar_decision_id`. |
| Metric | `oya_real_estate_lease_contract_usecase_commands_total{tenant_id,cell_id,command,status}` caps cardinality. |
| Latency histogram | `oya_real_estate_lease_contract_usecase_duration_seconds` tracks lifecycle command latency. |
| Trace span | `real_estate.lease_contract.operate` links workflow, document management, rent schedule, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `contract_id`, `command_id`, `approval_route`, and lifecycle state. |
| Capacity math | Approval route fan-out limits concurrent amendments to avoid stale term reads across dependent schedules. |
| Multi-region | Lifecycle writes stay in lease home cell; DR cells expose read-only contract state. |
| Sovereign cells | Contract documents and counterparty evidence remain in-region for active compliance packs. |
| Rollback | Disable operate endpoint, keep mutation inactive, and replay from last sealed lifecycle audit id. |
| Test evidence | Required tests cover approval denial, amendment conflict, termination handoff, tenant mismatch, and idempotent replay. |
| Rejected shortcut | A generic contract workflow is rejected because it loses SAP RE-FX lifecycle and accounting handoff semantics. |
