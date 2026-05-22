---
doc_class: ImplementationPlan
ip_id: IP-019
microservice: quality-management
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0132
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0294
  - ADR-0297
  - ADR-0314
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
  - ADR-0320
journey_ref: j119-quality-audit-readiness
sap_submodule: QM-AU Quality Audits
tenant_class: paid
billing_components:
  - per_usage
persona: Leila Santos, compliance quality manager
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-019: ISO 9001:2015 clause traceability map

## Context

- SAP QM submodule: QM-AU Quality Audits.
- Topic: ISO 9001:2015 clause traceability map.
- Persona: Leila Santos, compliance quality manager.
- Journey: j119 quality audit readiness.
- Journey leg: audit evidence is traced to ISO clauses before certification review.
- SAP precedent: audit management checklists and quality management system evidence.
- Oyatie aggregate: `IsoClauseTraceabilityMap`.
- Boundary: clause map, evidence coverage, control ownership, and certification gap state.
- ADR-0105 keeps traceability model separate from raw audit evidence.
- ADR-0131 keeps the IP with quality-management.
- ADR-0244 protects tenant certification evidence.
- ADR-0263 binds traceability audit events.
- ADR-0297 requires Cedar before clause coverage claims.
- ADR-0314 keeps marketplace certification display read-only.
- ADR-0315 requires ERP parity against SAP QM audit depth.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Clause mapping must support evidence gaps.
- Certification readiness must be a computed state, not marketing prose.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.iso_clause_map (
  tenant_id UUID NOT NULL,
  clause_map_id TEXT NOT NULL,
  standard_ref TEXT NOT NULL,
  clause_ref TEXT NOT NULL,
  clause_title TEXT NOT NULL,
  owner_team TEXT NOT NULL,
  required_evidence_count INTEGER NOT NULL,
  current_coverage_state TEXT NOT NULL,
  certification_scope TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, clause_map_id)
);
CREATE TABLE quality_management.iso_clause_evidence_link (
  tenant_id UUID NOT NULL,
  link_id TEXT NOT NULL,
  clause_map_id TEXT NOT NULL,
  evidence_id TEXT NOT NULL,
  evidence_strength TEXT NOT NULL,
  reviewer_principal_id TEXT NOT NULL,
  linked_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, link_id),
  UNIQUE (tenant_id, clause_map_id, evidence_id)
);
```

### Rust Types

```rust
pub struct IsoClauseTraceabilityMap {
    pub tenant_id: TenantId,
    pub clause_map_id: ClauseMapId,
    pub standard_ref: StandardRef,
    pub clause_ref: ClauseRef,
    pub clause_title: String,
    pub owner_team: OwnerTeam,
    pub required_evidence_count: u16,
    pub current_coverage_state: ClauseCoverageState,
    pub certification_scope: CertificationScope,
    pub evidence_links: Vec<IsoClauseEvidenceLink>,
}
pub enum ClauseCoverageState { NotStarted, Partial, Covered, Reviewed, GapAccepted }
pub enum EvidenceStrength { Weak, Moderate, Strong, PrimaryRecord }
pub enum IsoTraceabilityError {
    UnknownClause,
    DuplicateEvidenceLink,
    EvidenceOutsideCertificationScope,
    CoverageClaimPolicyDenied,
    ReviewerNotIndependent,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/iso-clause-maps`.
- Creates clause map row for ISO 9001:2015.
- `POST /v1/quality-management/iso-clause-maps/{clause_map_id}:link-evidence`.
- Links verified audit evidence to clause.
- `POST /v1/quality-management/iso-clause-maps/{clause_map_id}:review-coverage`.
- Moves clause to reviewed or gap accepted.
- `GET /v1/quality-management/iso-clause-maps/readiness`.
- Returns certification readiness by scope and clause.

### gRPC

- Service: `quality_management.iso_traceability.v1.IsoTraceabilityService`.
- `rpc CreateClauseMap(CreateClauseMapRequest) returns (ClauseMapReceipt)`.
- `rpc LinkClauseEvidence(LinkClauseEvidenceRequest) returns (ClauseMapView)`.
- `rpc ReviewClauseCoverage(ReviewClauseCoverageRequest) returns (ClauseMapView)`.
- `rpc GetCertificationReadiness(GetCertificationReadinessRequest) returns (CertificationReadiness)`.

### AsyncAPI

- Channel: `quality-management.iso-clause.evidence-linked.v1`.
- Channel: `quality-management.iso-clause.coverage-reviewed.v1`.
- Message: `IsoClauseEvidenceLinked`.
- Message: `IsoClauseCoverageReviewed`.
- Payload includes `standard_ref`, `clause_ref`, `coverage_state`, `evidence_strength`, `audit_event_class`.
- Consumers: compliance, audit evidence, marketplace, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::iso_traceability::link_evidence`.
- Principal: `ComplianceQualityManager`.
- Action: `iso_clause_evidence_link`.
- Resource: `IsoClauseTraceabilityMap`.
- Context: `certification_scope`, `evidence_scope`, `reviewer_independence`, `pack_ids`.
- Policy: `quality_management::iso_traceability::review_coverage`.
- Principal: `LeadAuditor`.
- Action: `iso_clause_coverage_review`.
- Resource: `IsoClauseTraceabilityMap`.
- Context: `coverage_state`, `required_evidence_count`, `linked_evidence_count`, `gap_reason`.
- Forbid: evidence outside certification scope.
- Forbid: reviewer is original evidence collector.
- Forbid: Covered with fewer than required evidence links.
- Forbid: marketplace display when coverage state is Partial.

## Ontology Projection

- Vendor object: ISO audit checklist from SAP QM or QMS vendor.
- Oyatie object: `quality_management.iso_clause_map`.
- ISO clause number -> `clause_ref`.
- ISO clause title -> `clause_title`.
- Audit checklist section -> `certification_scope`.
- Evidence attachment -> `evidence_id`.
- Evidence strength rubric -> `evidence_strength`.
- Auditor review -> `current_coverage_state`.
- Gap acceptance -> `GapAccepted`.
- IQS-AQM clause checklist -> clause map row.
- MasterControl certification package -> evidence link.
- ETQ Reliance compliance matrix -> clause map import.
- Projection freshness floor: 30 seconds.
- Projection consumer: compliance and marketplace trust display.
- Projection rule: marketplace can display only reviewed certification posture.

## Workflow Steps

- Node `clause-load`: ISO 9001:2015 canonical clauses loaded.
- Node `scope-bind`: certification scope is bound.
- Decision `unknown-clause`: reject map row.
- Node `evidence-search`: audit evidence candidates listed.
- Node `evidence-link`: manager links evidence.
- Decision `evidence-out-of-scope`: reject link.
- Decision `reviewer-not-independent`: reject review.
- Node `coverage-calc`: evidence count and strength computed.
- Decision `coverage-partial`: create gap task.
- Decision `coverage-covered`: require lead auditor review.
- Decision `gap-accepted`: record signed gap reason.
- Node `cedar-review`: evaluate coverage claim.
- Node `readiness-update`: update certification readiness.
- Node `marketplace-redact`: prepare read-only posture.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish clause map.
- Node `close`: readiness state is queryable.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-ISO_CLAUSE_MAP-CREATED`.
- `EVT-QUALITY_MANAGEMENT-ISO_CLAUSE-EVIDENCE_LINKED`.
- `EVT-QUALITY_MANAGEMENT-ISO_CLAUSE-COVERAGE_REVIEWED`.
- `EVT-QUALITY_MANAGEMENT-ISO_CLAUSE-GAP_ACCEPTED`.
- `EVT-QUALITY_MANAGEMENT-ISO_TRACEABILITY-IP_ACCEPTED`.
- ADR-0263 envelope stores `standard_ref`.
- ADR-0263 envelope stores `clause_ref`.
- ADR-0263 envelope stores `coverage_state`.
- ADR-0263 envelope stores `certification_scope`.
- ADR-0263 envelope stores `evidence_strength`.

## SLO Targets

- Evidence link p50: 70 ms.
- Evidence link p95: 250 ms.
- Evidence link p99: 700 ms.
- Readiness query p95: 300 ms.
- Throughput: 150 evidence links per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: certification readiness is compliance-critical but not production hot-path.

## Failure Modes and Recovery

- Failure: clause ref is unknown.
- Recovery: `ISO-CLAUSE-UNKNOWN-REJECT` rejects map row.
- Failure: linked evidence is outside certification scope.
- Recovery: `ISO-EVIDENCE-SCOPE-DENY` rejects link and creates review task.
- Failure: reviewer is not independent.
- Recovery: `ISO-REVIEWER-INDEPENDENCE-DENY` requires another reviewer.
- Failure: coverage claim lacks required evidence.
- Recovery: `ISO-COVERAGE-GAP` marks Partial and emits gap task.
- Failure: marketplace display tries to show partial coverage as certified.
- Recovery: `ISO-MARKETPLACE-DENY` blocks display and audits policy denial.
- Failure: evidence link event fails.
- Recovery: `ISO-LINK-OUTBOX-REPLAY` replays link event.

## Migration Notes

- Source vendor: SAP QM.
- Migrate audit checklist clauses into ISO clause map rows.
- Source vendor: IQS-AQM maps certification checklist rows into clause maps.
- Source vendor: MasterControl maps controlled quality manual sections into evidence links.
- Source vendor: ETQ Reliance maps compliance matrix into clause coverage.
- Source vendor: Sparta Systems TrackWise maps audit observations into clause gaps.
- Historical certification claims migrate as `Partial` until evidence links are verified.
- Marketplace display defaults off for migrated claims.
- Rollback path: disable readiness claim while preserving clause links.
- Clause titles remain source-versioned for ISO 9001:2015.

## Cross-microservice Handoffs

- From audit-evidence: verified evidence records.
- From audit-finding: open clause gaps.
- To compliance: certification readiness map.
- To marketplace: reviewed certification posture only.
- To workflow-engine: evidence gap remediation tasks.
- To ontology: clause coverage projection.
- To document-control: quality manual evidence.
- To supplier scorecard: supplier certification gap signal.

## Verification

- Unit: unknown clause rejected.
- Unit: reviewer independence enforced.
- Unit: Covered requires required evidence count.
- Contract: REST readiness returns clause states.
- Contract: gRPC coverage review returns evidence links.
- Event: evidence linked event validates.
- Policy: Cedar denies marketplace partial claim.
- Projection: ETQ compliance matrix fixture maps field-for-field.
- SLO: readiness query p95 under 300 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-ISO_TRACEABILITY-IP_ACCEPTED`.
