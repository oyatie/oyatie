---
doc_class: ImplementationPlan
ip_id: IP-023
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
persona: Mateo Silva, lease abstraction specialist
status: Accepted
date: 2026-05-20
owner_team: axis-real-estate + axis-document-intelligence
---

# IP-023: Lease abstraction

## Context

- SAP submodule: RE-FX-CN contract management.
- Persona: Mateo Silva, lease abstraction specialist.
- Journey leg: j166 acquisition go/no-go requires extracted critical dates, rights, rent terms, and exceptions before diligence signoff.
- SAP tables: `VICDCONTRACT`, `VICDOBJASS`, `VICDCONDLINE`, `VICDADJREASN`.
- Oyatie capability: `LeaseAbstract`.
- Precedent: SAP RE-FX contract clauses plus lease-abstraction workpapers from MRI/Yardi implementations.
- ADR-0263 records abstraction evidence and ADR-0297 gates clause-derived obligations before downstream automation.
- Boundary: captures extracted terms and confidence; legal judgment and contract execution remain legal-ops.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE real_estate.lease_abstract (
  tenant_id UUID NOT NULL,
  lease_abstract_id TEXT NOT NULL,
  lease_contract_id TEXT NOT NULL,
  document_ref TEXT NOT NULL,
  abstraction_status TEXT NOT NULL CHECK (abstraction_status IN ('draft','review_required','approved','rejected','superseded')),
  effective_date DATE NOT NULL,
  expiration_date DATE,
  extraction_confidence NUMERIC(8,6) NOT NULL,
  reviewer_ref TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, lease_abstract_id)
);
CREATE TABLE real_estate.lease_abstract_term (
  tenant_id UUID NOT NULL,
  lease_abstract_term_id TEXT NOT NULL,
  lease_abstract_id TEXT NOT NULL,
  term_kind TEXT NOT NULL,
  term_value JSONB NOT NULL,
  source_span_ref TEXT NOT NULL,
  confidence NUMERIC(8,6) NOT NULL,
  PRIMARY KEY (tenant_id, lease_abstract_term_id)
);
```

### Rust Types

```rust
pub struct LeaseAbstract {
    pub tenant_id: TenantId,
    pub lease_abstract_id: LeaseAbstractId,
    pub lease_contract_id: LeaseContractId,
    pub document_ref: DocumentRef,
    pub abstraction_status: LeaseAbstractStatus,
    pub effective_date: NaiveDate,
    pub expiration_date: Option<NaiveDate>,
    pub extraction_confidence: Decimal,
    pub reviewer_ref: Option<UserRef>,
}
pub struct LeaseAbstractTerm {
    pub term_kind: LeaseTermKind,
    pub term_value: serde_json::Value,
    pub source_span_ref: SourceSpanRef,
    pub confidence: Decimal,
}
pub enum LeaseAbstractError { DocumentMissing, ConfidenceTooLow, ReviewerRequired, PolicyDenied, ContractSyncFailed }
```

## API Endpoints

- REST `POST /v1/real-estate/lease-abstracts` creates an abstraction package.
- REST `POST /v1/real-estate/lease-abstracts/{id}:submit-review`.
- REST `POST /v1/real-estate/lease-abstracts/{id}:approve`.
- REST `GET /v1/real-estate/lease-abstracts/{id}/terms`.
- gRPC `real_estate.lease_abstract.v1.LeaseAbstractService.CreateLeaseAbstract`.
- gRPC `SubmitReview`, `ApproveLeaseAbstract`, and `ListTerms`.
- AsyncAPI channel `real-estate.lease-abstract.review-required.v1`.
- AsyncAPI channel `real-estate.lease-abstract.approved.v1`.
- Consumers: legal-ops, lease-contract, rent-schedule, compliance.

## Cedar Policy Hooks

- Policy: `real_estate::lease_abstract::approve`.
- Principal: `LeaseAbstractionSpecialist`.
- Action: `approve_lease_abstract`.
- Resource: `LeaseAbstract`.
- Context: `tenant_id`, `lease_contract_id`, `extraction_confidence`, `term_count`, `reviewer_ref`, `document_ref`.
- Forbid when confidence is below threshold, source spans are missing, reviewer independence fails, or contract is already locked.

## Ontology Projection

- Vendor object: SAP RE-FX contract clause and condition lineage.
- Oyatie object: `real_estate.lease_abstract`.
- `VICDCONTRACT-CONTRACT` -> `lease_contract_id`.
- `VICDOBJASS-OBJNR` -> leased object association term.
- `VICDCONDLINE-CONDGUID` -> rent condition term.
- `VICDADJREASN-ADJREASON` -> adjustment clause term.
- Source document span -> abstract term evidence.
- Extracted critical dates -> option and rent schedule seeds.
- Projection freshness floor: approved abstract.

## Workflow Steps

- Node `document-ingest`: bind source lease document.
- Decision `document-missing`: block package.
- Node `term-extract`: extract dates, parties, premises, rent, options, and obligations.
- Decision `confidence-low`: route specialist review.
- Node `source-span-verify`: confirm every term has evidence.
- Decision `reviewer-required`: assign independent reviewer.
- Node `abstract-approve`: seal approved terms.
- Node `contract-sync`: publish approved terms to lease-contract and rent-schedule.
- Node `audit-seal`: emit abstraction evidence.

## Audit Events

- `EVT-REAL_ESTATE-LEASE_ABSTRACT-CREATED`.
- `EVT-REAL_ESTATE-LEASE_ABSTRACT-REVIEW_REQUIRED`.
- `EVT-REAL_ESTATE-LEASE_ABSTRACT-APPROVED`.
- `EVT-REAL_ESTATE-LEASE_ABSTRACT-REJECTED`.
- `EVT-REAL_ESTATE-LEASE_ABSTRACT-CONTRACT_SYNCED`.
- `EVT-REAL_ESTATE-LEASE_ABSTRACT-IP_ACCEPTED`.
- ADR-0263 envelope stores document ref, source spans, confidence, reviewer, approved term count, and contract sync ref.

## SLO Targets

- Abstract create p50: 120 ms after document intelligence output is available.
- Abstract create p95: 600 ms.
- Abstract create p99: 2,000 ms for 300 extracted terms.
- Review-submit p95: 300 ms.
- Rationale: extraction may be async, but abstraction package persistence and review routing must stay interactive.

## Failure Modes and Recovery

- Failure: `DOCUMENT-MISSING`; recovery: request legal document ingest.
- Failure: `CONFIDENCE-TOO-LOW`; recovery: require manual specialist review.
- Failure: `SOURCE-SPAN-MISSING`; recovery: reject term and preserve extraction evidence.
- Failure: `REVIEWER-INDEPENDENCE-FAILED`; recovery: reassign reviewer through workflow-engine.
- Failure: `CONTRACT-SYNC-FAILED`; recovery: retry idempotent sync after lease-contract unlock.
- Failure: `TERM-CONFLICT`; recovery: create exception task and hold approval.

## Migration Notes

- Import existing abstracts after lease contracts and documents.
- Preserve original source span or page references where available.
- Mark low-confidence migrated terms as `review_required`.
- Do not seed option/rent automation from migrated abstracts until approval is replayed.
- Rollback path: supersede abstract and remove pending contract sync tasks.
- Backfill order: documents, contracts, object assignments, abstracts, terms, approvals, sync refs.

## Cross-microservice Handoffs

- From document-intelligence: source spans and extracted term candidates.
- From legal-ops: reviewer independence and legal status.
- To lease-contract: approved contract terms.
- To rent-schedule: rent and escalation clauses.
- To workflow-engine: review and conflict tasks.
- To compliance: abstraction evidence package.

## Final Substance-Bar Closeout

| Check | Binding detail |
|---|---|
| Documentation-rigor floor | This IP is intentionally above the 200-line ImplementationPlan floor after the final ERP audit. |
| SAP specificity | Lease abstraction remains bound to SAP RE-FX-CN contract management. |
| Persona specificity | Mateo Silva owns extracted term review, approval, conflict handling, and rollback language. |
| Journey specificity | The j166 acquisition go/no-go leg drives critical-date, rights, rent-term, and exception evidence. |
| DDL anchor | The abstraction, extracted term, source span, approval, and sync-ref tables above are normative. |
| Rust anchor | Lease abstraction, extracted term, reviewer decision, and error types above are anchors. |
| REST anchor | Extract, review, approve, sync, supersede, and explain endpoints are tenant surfaces. |
| gRPC anchor | The lease abstraction service is the worker and replay contract. |
| AsyncAPI anchor | Abstract created, reviewed, approved, conflict-opened, and synced channels carry diligence evidence. |
| Cedar anchor | Abstraction approval is default-deny and must persist `cedar_decision_id`. |
| Ontology anchor | Contract document spans and SAP contract lineage project to abstraction and term nodes. |
| ADR-0263 class binding | Abstraction review checks emit `OfficeBoundaryAttemptEvaluated` plus allow or deny outcome classes. |
| ADR-0263 pack binding | Legal, acquisition, or lease-pack overlay changes emit `OfficePackOverlayChanged`. |
| ADR-0263 security binding | Edge throttling on abstraction APIs emits `AbuseDefenceRateLimitHit`. |
| Audit payload | Include `tenant_id`, `audit_id`, `trace_id`, abstraction id, document id, source span hash, reviewer id, and `cedar_decision_id`. |
| Metric | `oya_real_estate_lease_abstractions_total{tenant_id,cell_id,outcome,status}` caps outcome/status cardinality. |
| Latency histogram | `oya_real_estate_lease_abstraction_duration_seconds` tracks extraction, review, and sync latency. |
| Trace span | `real_estate.lease_abstraction.approve` links document intelligence, legal ops, lease contract, and audit-chain spans. |
| Log schema | Structured logs include `tenant_id`, `principal_id`, `abstraction_id`, `document_id`, `term_count`, and conflict state. |
| Capacity math | Review load uses extracted_terms / reviewer_rate; high-risk clauses require second-review capacity before sync. |
| Multi-region | Abstraction approvals write in lease home cell; DR cells expose read-only abstraction evidence. |
| Sovereign cells | Lease documents and extracted terms remain in-region for legal and sovereign packs. |
| Rollback | Supersede abstract, remove pending contract sync tasks, and replay from last sealed abstraction audit id. |
| Test evidence | Required tests cover unsupported clause, reviewer conflict, source-span hash mismatch, tenant mismatch, and idempotent sync. |
| Rejected shortcut | A generic OCR summary is rejected because it loses legal-review, source-span, and SAP RE-FX contract semantics. |
