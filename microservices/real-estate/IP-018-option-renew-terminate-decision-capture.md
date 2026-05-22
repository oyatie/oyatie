---
doc_class: ImplementationPlan
ip_id: IP-018
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
journey_ref: j166-cso-mira-g-strategic-acquisition-go-no-go
sap_submodule: RE-FX-CN (contracts)
tenant_class: paid
billing_components:
  - per_usage
persona: Amara Singh, lease administration manager
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-erp-parity
---

# IP-018: Option-to-renew and terminate decision capture

## Context

- SAP submodule: RE-FX-CN lease option clauses.
- Persona: Amara Singh, lease administration manager.
- Journey leg: j166 strategic portfolio review decides whether to renew, terminate, or let an option lapse.
- SAP tables: `VICDCONTRACT`, `VICDCONDLINE`, `VICDOBJASS`, `VICDADJREASN`.
- Oyatie capability: `LeaseOptionDecision`.
- Precedent: SAP RE-FX option management plus NetSuite lease renewal option tracking.
- ADR-0263 binds decision audit and ADR-0297 gates financially material option exercise.
- Boundary: records option terms, decision, evidence, and downstream amendment request; valuation effects are lease-accounting.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.lease_option (
  tenant_id UUID NOT NULL,
  lease_option_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  option_kind TEXT NOT NULL CHECK (option_kind IN ('renew','terminate','purchase','expand','contract')),
  notice_deadline DATE NOT NULL,
  effective_date DATE NOT NULL,
  option_status TEXT NOT NULL CHECK (option_status IN ('open','exercised','declined','lapsed')),
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, lease_option_id)
);
CREATE TABLE real_estate.lease_option_decision (
  tenant_id UUID NOT NULL,
  option_decision_id TEXT NOT NULL,
  lease_option_id TEXT NOT NULL,
  decision TEXT NOT NULL CHECK (decision IN ('exercise','decline','defer')),
  decision_reason TEXT NOT NULL,
  decided_by TEXT NOT NULL,
  decided_at TIMESTAMPTZ NOT NULL,
  PRIMARY KEY (tenant_id, option_decision_id)
);
```

### Rust Types

```rust
pub struct LeaseOption {
    pub tenant_id: TenantId,
    pub lease_option_id: LeaseOptionId,
    pub lease_contract_id: LeaseContractId,
    pub option_kind: LeaseOptionKind,
    pub notice_deadline: NaiveDate,
    pub effective_date: NaiveDate,
    pub option_status: LeaseOptionStatus,
}
pub struct LeaseOptionDecision {
    pub option_decision_id: OptionDecisionId,
    pub lease_option_id: LeaseOptionId,
    pub decision: LeaseOptionDecisionKind,
    pub decision_reason: String,
    pub decided_by: PrincipalId,
    pub decided_at: DateTime<Utc>,
}
pub enum LeaseOptionError { NoticeDeadlinePassed, EvidenceMissing, DecisionPolicyDenied, AccountingReviewRequired, DuplicateDecision }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-options` records option clause.
- REST `POST /v1/real-estate/lease-options/{id}:decide`.
- REST `GET /v1/real-estate/lease-contracts/{id}/options`.
- gRPC `real_estate.lease_option.v1.LeaseOptionService.RecordOption`.
- gRPC `DecideOption` and `ListLeaseOptions`.
- AsyncAPI channel `real-estate.lease-option.decision-recorded.v1`.
- AsyncAPI channel `real-estate.lease-option.lapsed.v1`.
- Consumers: lease-contract, lease-accounting, workflow-engine, portfolio-analytics.

## Cedar Policy Hooks

- Policy: `real_estate::lease_option::decide`.
- Principal: `LeaseAdministrator`.
- Action: `lease_option_decide`.
- Resource: `LeaseOption`.
- Context: `tenant_id`, `option_kind`, `notice_deadline`, `decision`, `financial_materiality`, `evidence_ref`.
- Forbid when notice deadline passed without override, evidence missing, material option lacks controller review, or duplicate final decision exists.

## Ontology Projection

- Vendor object: SAP RE-FX contract option clause.
- Oyatie object: `real_estate.lease_option_decision`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDCONDLINE-CONDGUID` -> economic term lineage.
- `VICDOBJASS-OBJNR` -> premises affected by option.
- `VICDADJREASN-ADJREASON` -> decision reason mapping.
- Notice deadline -> decision timeliness evidence.
- Decision -> contract amendment trigger.
- Projection freshness floor: 5 seconds.
- Projection rule: lapsed options remain immutable and cannot be overwritten by later decisions.

## Workflow Steps

- Node `option-record`: capture option clause and deadline.
- Node `deadline-monitor`: emit upcoming deadline alert.
- Decision `deadline-passed`: mark lapsed or require override.
- Node `evidence-collect`: attach market, occupancy, and accounting evidence.
- Decision `evidence-missing`: block final decision.
- Node `policy-evaluate`: authorize decision.
- Decision `accounting-review-required`: route to controller review.
- Node `decision-record`: persist exercise/decline/defer.
- Node `contract-handoff`: create amendment or termination command.
- Node `audit-seal`: emit option decision evidence.

## Audit Events

- `EVT-REAL_ESTATE-LEASE_OPTION-RECORDED`.
- `EVT-REAL_ESTATE-LEASE_OPTION-DEADLINE_ALERTED`.
- `EVT-REAL_ESTATE-LEASE_OPTION-DECISION_RECORDED`.
- `EVT-REAL_ESTATE-LEASE_OPTION-LAPSED`.
- `EVT-REAL_ESTATE-LEASE_OPTION-POLICY_DENIED`.
- `EVT-REAL_ESTATE-LEASE_OPTION-IP_ACCEPTED`.
- ADR-0263 envelope stores option kind, deadline, decision, financial materiality, and evidence ref.

## SLO Targets

- Option record p50: 45 ms.
- Option record p95: 160 ms.
- Option record p99: 420 ms.
- Decision record p95: 250 ms.
- Rationale: lease option decisions are human-governed but must produce immediate immutable evidence at deadline.

## Failure Modes and Recovery

- Failure: `NOTICE-DEADLINE-PASSED`; recovery: mark lapsed or require legal override.
- Failure: `EVIDENCE-MISSING`; recovery: keep decision draft and request attachments.
- Failure: `DECISION-POLICY-DENIED`; recovery: route to approval workflow.
- Failure: `ACCOUNTING-REVIEW-REQUIRED`; recovery: block contract handoff until review.
- Failure: `DUPLICATE-DECISION`; recovery: return existing final decision.
- Failure: `CONTRACT-HANDOFF-FAILED`; recovery: retry amendment/termination command.

## Migration Notes

- Import option clauses from contract terms with notice deadlines.
- Preserve source clause text reference and economic term lineage.
- Mark expired options as lapsed unless source shows exercise evidence.
- Do not create amendments from migrated decisions automatically.
- Rollback path: disable decide endpoint and keep option tracking read-only.
- Backfill order: contracts, option clauses, evidence refs, decisions, handoffs.

## Cross-microservice Handoffs

- From lease-contract: option clause and current status.
- From portfolio analytics: market and vacancy evidence.
- To lease-accounting: material option assessment.
- To lease-contract: amendment or termination command.
- To workflow-engine: legal/controller review.
- To compliance: option decision audit trail.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | The decision capture remains bound to SAP RE-FX-CN lease option clauses. |
| Persona specificity | Amara Singh owns renew/terminate/lapse capture, evidence review, and rollback language. |
| Journey specificity | The j166 strategic portfolio review leg drives option decision evidence and legal review. |
| DDL anchor | The option clause, decision, evidence ref, and handoff tables above are normative. |
| Rust anchor | Option decision, evidence ref, handoff result, and error types above are implementation anchors. |
| REST anchor | Capture decision, approve, lapse, and handoff endpoints are tenant surfaces. |
| gRPC anchor | The option decision service is the worker and replay contract. |
| AsyncAPI anchor | Option captured, approved, terminated, and lapsed channels carry portfolio evidence. |
| Cedar anchor | Option decision is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | SAP lease option and contract lineage projects to option decision nodes. |
| ADR-0263 class binding | Option decision checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Legal, portfolio, or lease-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on option APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, option id, contract id, decision type, reviewer id, and `cedar_decision_id`. |
| Metric | `oya_real_estate_option_decisions_total{tenant_id,cell_id,decision,status}` caps decision/status cardinality. |
| Latency histogram | `oya_real_estate_option_decision_duration_seconds` tracks capture and approval latency. |
| Trace span | `real_estate.option_decision.capture` links lease contract, portfolio analytics, workflow, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `option_id`, `contract_id`, `decision_type`, and evidence hash. |
| Capacity math | Review queue uses options_expiring_90d / reviewer_capacity and escalates when cutoff risk exceeds threshold. |
| Multi-region | Option decisions write in lease home cell; DR cells expose read-only option evidence. |
| Sovereign cells | Contract, market, and legal evidence remains in-region for active packs. |
| Rollback | Disable decide endpoint, keep option tracking read-only, and replay from last sealed option audit id. |
| Test evidence | Required tests cover expired option, missing evidence, legal denial, tenant mismatch, and idempotent handoff. |
| Rejected shortcut | A generic reminder task is rejected because it loses SAP RE-FX option clause and portfolio decision semantics. |
