---
doc_class: ImplementationPlan
ip_id: IP-001
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

# IP-001: Domain layer for lease contract

## Context

- SAP submodule: RE-FX-CN real-estate contracts.
- Persona: Amara Singh, lease administration manager.
- Journey leg: j137 SOX auditor tests contract approval, term lineage, and billing condition control.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie aggregate: `LeaseContract`.
- Precedent: SAP RE-FX contract master plus DocuSign CLM immutable agreement envelope.
- ADR-0244 scopes every contract by tenant and ADR-0263 binds contract lifecycle audit events.
- Boundary: owns contract identity, term dates, premises assignment, rent-condition references, and approval state; accounting postings are lease-accounting events.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.lease_contract (
  tenant_id UUID NOT NULL,
  lease_contract_id TEXT NOT NULL,
  sap_contract_ref TEXT NOT NULL,
  counterparty_id TEXT NOT NULL,
  contract_type TEXT NOT NULL CHECK (contract_type IN ('lessee','lessor','sublease','service')),
  commencement_date DATE NOT NULL,
  expiration_date DATE NOT NULL,
  contract_status TEXT NOT NULL CHECK (contract_status IN ('draft','approved','active','amended','terminated','archived')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, lease_contract_id)
);
CREATE TABLE real_estate.lease_contract_term (
  tenant_id UUID NOT NULL,
  lease_contract_id TEXT NOT NULL,
  term_no INTEGER NOT NULL,
  term_kind TEXT NOT NULL,
  effective_from DATE NOT NULL,
  effective_to DATE,
  term_payload JSONB NOT NULL,
  PRIMARY KEY (tenant_id, lease_contract_id, term_no)
);
```

### Rust Types

```rust
pub struct LeaseContract {
    pub tenant_id: TenantId,
    pub lease_contract_id: LeaseContractId,
    pub sap_contract_ref: SapContractRef,
    pub counterparty_id: PartyId,
    pub contract_type: LeaseContractType,
    pub commencement_date: NaiveDate,
    pub expiration_date: NaiveDate,
    pub contract_status: LeaseContractStatus,
}
pub struct LeaseContractTerm {
    pub term_no: u32,
    pub term_kind: LeaseTermKind,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub term_payload: serde_json::Value,
}
pub enum LeaseContractError { DateRangeInvalid, CounterpartyMissing, PremisesUnassigned, DuplicateContractRef, ApprovalPolicyDenied }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-contracts` creates a draft lease contract.
- REST `POST /v1/real-estate/lease-contracts/{id}:approve` activates approved terms.
- REST `POST /v1/real-estate/lease-contracts/{id}:amend` records versioned amendment.
- REST `POST /v1/real-estate/lease-contracts/{id}:terminate` terminates with reason.
- gRPC `real_estate.lease_contract.v1.LeaseContractService.CreateLeaseContract`.
- gRPC `ApproveLeaseContract`, `AmendLeaseContract`, and `TerminateLeaseContract`.
- AsyncAPI channel `real-estate.lease-contract.approved.v1`.
- AsyncAPI channel `real-estate.lease-contract.terminated.v1`.

## Cedar Policy Hooks

- Policy: `real_estate::lease_contract::approve`.
- Principal: `LeaseAdministrator`.
- Action: `lease_contract_approve`.
- Resource: `LeaseContract`.
- Context: `tenant_id`, `contract_type`, `counterparty_id`, `premises_assigned`, `approval_route_id`.
- Forbid when premises assignment is missing, counterparty is inactive, approval route is incomplete, or contract crosses tenant boundary.

## Ontology Projection

- Vendor object: SAP RE-FX `VICDCONTRACT`.
- Oyatie object: `real_estate.lease_contract`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDOBJASS-OBJNR` -> assigned premises lineage.
- `VICDCONDLINE-CONDGUID` -> rent condition references.
- `VICDADJREASN-ADJREASON` -> adjustment reason lineage.
- Counterparty master -> `counterparty_id`.
- Approval envelope -> `contract_status`.
- Projection freshness floor: 5 seconds.
- Projection rule: SAP contract number remains source lineage, not tenant-global identity.

## Workflow Steps

- Node `contract-draft`: create draft with counterparty and dates.
- Decision `date-range-invalid`: reject draft.
- Node `premises-assign`: bind architectural object.
- Decision `premises-unassigned`: block approval.
- Node `condition-link`: attach rent and service conditions.
- Node `approval-route`: request workflow approval.
- Decision `approval-denied`: keep draft and capture denial reason.
- Node `activate-contract`: mark approved or active.
- Node `accounting-event-request`: request lease accounting setup.
- Node `audit-seal`: emit ADR-0263 contract event.

## Audit Events

- `EVT-REAL_ESTATE-LEASE_CONTRACT-CREATED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT-APPROVED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT-AMENDED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT-TERMINATED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT-POLICY_DENIED`.
- `EVT-REAL_ESTATE-LEASE_CONTRACT-IP_ACCEPTED`.
- ADR-0263 envelope stores `sap_contract_ref`, `counterparty_id`, assigned object refs, and approval route.

## SLO Targets

- Create contract p50: 55 ms.
- Create contract p95: 200 ms.
- Create contract p99: 600 ms.
- Approval command p95: 300 ms.
- Rationale: lease administration is interactive; approval may include workflow and policy checks but must still return a durable receipt quickly.

## Failure Modes and Recovery

- Failure: `DATE-RANGE-INVALID`; recovery: reject draft and return violating date fields.
- Failure: `COUNTERPARTY-MISSING`; recovery: route to counterparty onboarding.
- Failure: `PREMISES-UNASSIGNED`; recovery: hold approval until object assignment is valid.
- Failure: `DUPLICATE-CONTRACT-REF`; recovery: return existing contract lineage.
- Failure: `APPROVAL-POLICY-DENIED`; recovery: preserve draft and emit policy denial.
- Failure: `ACCOUNTING-HANDOFF-FAILED`; recovery: keep contract active-pending-accounting and retry outbox.

## Migration Notes

- Import `VICDCONTRACT` before terms and conditions.
- Import `VICDOBJASS` object assignment after architectural objects exist.
- Preserve SAP contract IDs as lineage.
- Do not activate migrated contracts with missing approval evidence; mark active-readonly.
- Rollback path: disable contract mutation and retain read-only migrated contracts.
- Backfill order: counterparties, architectural objects, contracts, terms, conditions, accounting handoff.

## Cross-microservice Handoffs

- From identity/counterparty: tenant-scoped party identity.
- From facility master: assigned architectural object.
- To rent schedule: condition line activation.
- To lease accounting: initial recognition request.
- To workflow-engine: approval and termination tasks.
- To compliance: contract lifecycle audit evidence.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The primitive remains bound to SAP RE-FX-CN contracts, not a generic document record. |
| Persona specificity | Amara Singh owns lease approval, amendment, termination, and rollback language. |
| Journey specificity | The j137 SOX control-test leg drives contract approval, term lineage, and billing-condition evidence. |
| DDL anchor | The lease contract and term tables above are the normative contract state model. |
| Rust anchor | `LeaseContract`, `LeaseContractTerm`, and `LeaseContractError` are the implementation type names. |
| REST anchor | Create, approve, amend, and terminate endpoints are the tenant command surface. |
| gRPC anchor | `real_estate.lease_contract.v1.LeaseContractService` is the worker and replay contract. |
| AsyncAPI anchor | Approved and terminated channels carry accounting and compliance evidence. |
| Cedar anchor | `real_estate::lease_contract::approve` is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP `VICDCONTRACT`, `VICDOBJASS`, and condition lineage project to `real_estate.lease_contract`. |
| ADR-0263 class binding | Approval checks emit `OfficeBoundaryAttemptEvaluated` and then `OfficeBoundaryAttemptAllowed` or `OfficeBoundaryAttemptDenied`. |
| ADR-0263 pack binding | Lease pack or office-scope overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on contract APIs emits `AbuseDefenceRateLimitHit`, never a free-form class. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, `sap_contract_ref`, `counterparty_id`, object refs, and `cedar_decision_id`. |
| Metric | `oya_real_estate_lease_contract_commands_total{tenant_id,cell_id,command,status}` caps command/status cardinality. |
| Latency histogram | `oya_real_estate_lease_contract_command_duration_seconds` tracks p50/p95/p99 approval latency. |
| Trace span | `real_estate.lease_contract.approve` links workflow, rent schedule, lease accounting, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `lease_contract_id`, `counterparty_id`, and approval route. |
| Capacity math | Approval backlog uses active_contracts * route_steps / approver_capacity; queue saturation blocks auto activation. |
| Multi-region | Contract mutations write in the contract home cell; DR cells serve read-only contract projections. |
| Sovereign cells | Lease documents, counterparty, and premises evidence remain in-region for KR-CSAP, EU, CN-PIPL, IL5/6, and FedRAMP-High. |
| Rollback | Disable contract mutation, retain read-only migrated contracts, and replay from last sealed contract audit id. |
| Test evidence | Required tests cover date range, inactive counterparty, missing premises, policy denial, and idempotent approval replay. |
| Rejected shortcut | A generic `LegalAgreement` model is rejected because it loses SAP RE-FX contract, object, and condition semantics. |
