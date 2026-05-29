---
doc_class: ImplementationPlan
ip_id: IP-012
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
persona: Omar Haddad, quality auditor
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-012: Usecase layer for finding lifecycle

## Context

- SAP QM submodule: QM-AU Quality Audits.
- Topic: finding lifecycle `Open -> Investigated -> Mitigated -> Verified -> Closed`.
- Persona: Omar Haddad, quality auditor.
- Journey: j119 quality audit readiness.
- Journey leg: audit evidence becomes a controlled finding and closure package.
- SAP precedent: audit finding, corrective task, follow-up verification, and closure.
- Oyatie usecase: `AdvanceAuditFinding`.
- Boundary: orchestrates evidence, CAPA, notification, supplier scheduling, and closure.
- ADR-0105 places lifecycle orchestration in usecase.
- ADR-0131 keeps this IP local to the microservice.
- ADR-0244 protects supplier and audit tenant scope.
- ADR-0263 binds lifecycle event classes.
- ADR-0297 requires Cedar on every state transition.
- ADR-0314 keeps marketplace settlement outside audit findings.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Finding closure must prove mitigation and verification.
- Reopening is explicit and audited.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.audit_finding (
  tenant_id UUID NOT NULL,
  finding_id TEXT NOT NULL,
  audit_id TEXT NOT NULL,
  evidence_id TEXT NOT NULL,
  finding_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  lifecycle_state TEXT NOT NULL,
  owner_principal_id TEXT NOT NULL,
  due_at TIMESTAMPTZ NOT NULL,
  capa_case_id TEXT,
  verified_by_principal_id TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, finding_id)
);
CREATE TABLE quality_management.audit_finding_transition (
  tenant_id UUID NOT NULL,
  transition_id TEXT NOT NULL,
  finding_id TEXT NOT NULL,
  from_state TEXT NOT NULL,
  to_state TEXT NOT NULL,
  reason_code TEXT NOT NULL,
  evidence_ref TEXT,
  transitioned_by_principal_id TEXT NOT NULL,
  transitioned_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, transition_id)
);
```

### Rust Types

```rust
pub struct AuditFinding {
    pub tenant_id: TenantId,
    pub finding_id: FindingId,
    pub audit_id: AuditId,
    pub evidence_id: EvidenceId,
    pub finding_type: FindingType,
    pub severity: FindingSeverity,
    pub lifecycle_state: FindingState,
    pub owner_principal_id: PrincipalId,
    pub due_at: DateTime<Utc>,
    pub capa_case_id: Option<CapaCaseId>,
    pub verified_by_principal_id: Option<PrincipalId>,
}
pub enum FindingState { Open, Investigated, Mitigated, Verified, Closed, Reopened, Cancelled }
pub enum FindingType { ProcessGap, SupplierGap, DocumentationGap, SafetyGap, DataIntegrityGap }
pub enum FindingTransitionError {
    InvalidStateTransition,
    MissingMitigationEvidence,
    MissingVerificationPrincipal,
    CapaNotEffective,
    OwnerOutsideAuditScope,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/audit-findings`.
- Opens a finding from verified audit evidence.
- `POST /v1/quality-management/audit-findings/{finding_id}:investigate`.
- Moves `Open` to `Investigated`.
- `POST /v1/quality-management/audit-findings/{finding_id}:mitigate`.
- Moves `Investigated` to `Mitigated`.
- `POST /v1/quality-management/audit-findings/{finding_id}:verify`.
- Moves `Mitigated` to `Verified`.
- `POST /v1/quality-management/audit-findings/{finding_id}:close`.
- Moves `Verified` to `Closed`.

### gRPC

- Service: `quality_management.audit_finding.v1.AuditFindingService`.
- `rpc OpenFinding(OpenFindingRequest) returns (FindingReceipt)`.
- `rpc AdvanceFinding(AdvanceFindingRequest) returns (FindingReceipt)`.
- `rpc ReopenFinding(ReopenFindingRequest) returns (FindingReceipt)`.
- `rpc StreamFindingLifecycle(StreamFindingLifecycleRequest) returns (stream AuditFindingEvent)`.

### AsyncAPI

- Channel: `quality-management.audit-finding.opened.v1`.
- Channel: `quality-management.audit-finding.transitioned.v1`.
- Channel: `quality-management.audit-finding.closed.v1`.
- Message: `AuditFindingOpened`.
- Message: `AuditFindingTransitioned`.
- Payload includes `finding_id`, `from_state`, `to_state`, `severity`, `capa_case_id`, `audit_event_class`.
- Consumers: CAPA, workflow-engine, supplier-quality, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::audit_finding::open`.
- Principal: `LeadAuditor`.
- Action: `audit_finding_open`.
- Resource: `VerifiedAuditEvidence`.
- Context: `audit_scope`, `severity`, `supplier_id`, `standard_ref`, `pack_ids`.
- Policy: `quality_management::audit_finding::transition`.
- Principal: `FindingOwner` or `LeadAuditor`.
- Action: `audit_finding_transition`.
- Resource: `AuditFinding`.
- Context: `from_state`, `to_state`, `mitigation_evidence`, `verification_principal`, `capa_effectiveness_state`.
- Forbid: skip from Open to Closed.
- Forbid: Mitigated to Verified without independent verifier.
- Forbid: Closed without verified CAPA when CAPA is required.
- Forbid: owner outside audit scope.

## Ontology Projection

- Vendor object: SAP QM audit finding and follow-up action.
- Oyatie object: `quality_management.audit_finding`.
- Vendor finding id -> `finding_id`.
- Vendor audit id -> `audit_id`.
- Vendor evidence link -> `evidence_id`.
- Vendor severity -> `severity`.
- Vendor responsible person -> `owner_principal_id`.
- Vendor due date -> `due_at`.
- Vendor corrective action -> `capa_case_id`.
- Vendor follow-up verification -> `Verified`.
- TrackWise deviation finding -> `audit_finding`.
- ETQ Reliance audit finding -> finding lifecycle.
- MasterControl audit observation -> finding evidence link.
- Projection freshness floor: 5 seconds.
- Projection consumer: compliance and CAPA.
- Projection rule: transition history is append-only.

## Workflow Steps

- Node `evidence-verified`: finding can open only from verified evidence.
- Node `finding-open`: state `Open`.
- Decision `severity-critical`: create quality hold or notification.
- Node `owner-assign`: assign finding owner.
- Node `investigation-record`: root facts and impact recorded.
- Decision `insufficient-investigation`: stay Open.
- Node `state-investigated`: move to `Investigated`.
- Node `mitigation-plan`: mitigation evidence and CAPA link recorded.
- Decision `capa-required`: create or link CAPA.
- Node `state-mitigated`: move to `Mitigated`.
- Node `independent-verify`: verifier reviews evidence.
- Decision `verification-failed`: reopen to `Investigated`.
- Node `state-verified`: move to `Verified`.
- Decision `closure-pack-missing`: block closure.
- Node `state-closed`: move to `Closed`.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish lifecycle projection.
- Node `close`: closed finding is immutable except reopen.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-AUDIT_FINDING-OPENED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_FINDING-TRANSITIONED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_FINDING-VERIFIED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_FINDING-CLOSED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-IP_ACCEPTED`.
- ADR-0263 envelope stores `finding_id`.
- ADR-0263 envelope stores `from_state`.
- ADR-0263 envelope stores `to_state`.
- ADR-0263 envelope stores `reason_code`.
- ADR-0263 envelope stores `capa_case_id`.

## SLO Targets

- Transition command p50: 65 ms.
- Transition command p95: 230 ms.
- Transition command p99: 650 ms.
- Lifecycle event dispatch p95: 500 ms.
- Throughput: 100 finding transitions per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: finding state is workflow-critical but not warehouse-hot.

## Failure Modes and Recovery

- Failure: invalid transition attempts Open -> Closed.
- Recovery: `FINDING-TRANSITION-DENY` rejects and returns allowed next states.
- Failure: mitigation evidence is missing.
- Recovery: `FINDING-MITIGATION-EVIDENCE-GATE` keeps state Investigated.
- Failure: verifier is same as owner.
- Recovery: `FINDING-INDEPENDENT-VERIFY-DENY` requires independent verifier.
- Failure: CAPA effectiveness is failed.
- Recovery: `FINDING-CAPA-REOPEN` moves back to Investigated.
- Failure: transition event dispatch stalls.
- Recovery: `FINDING-OUTBOX-REPLAY` replays append-only transition.
- Failure: supplier audit scope changes while finding is open.
- Recovery: `FINDING-SCOPE-RECHECK` blocks closure until scope is revalidated.

## Migration Notes

- Source vendor: SAP QM.
- Migrate audit findings and follow-up actions.
- Source vendor: Sparta Systems TrackWise maps deviations into finding lifecycle.
- Source vendor: ETQ Reliance maps audit findings into Open or Closed states.
- Source vendor: MasterControl maps observations and CAPA links into transitions.
- Source vendor: IQS-AQM maps supplier audit findings into supplier-scoped findings.
- Unknown vendor terminal states migrate as `Verified`, not `Closed`.
- Missing mitigation evidence migrates as `Open`.
- Rollback path: freeze transitions and keep read-only finding history.
- Reopened findings keep full transition history.

## Cross-microservice Handoffs

- From audit-evidence: verified evidence.
- To CAPA: corrective action case.
- To quality-notification: defect notification for severe findings.
- To quality-hold: containment for critical findings.
- To workflow-engine: lifecycle tasks and due dates.
- To compliance: closure package.
- To supplier scorecard: supplier finding signal.
- To ontology: finding lifecycle projection.

## Verification

- Unit: invalid transition rejected.
- Unit: same owner cannot verify mitigation.
- Unit: CAPA failed reopens finding.
- Contract: REST close requires verified state.
- Contract: gRPC stream emits transition sequence.
- Event: closed event validates.
- Policy: Cedar denies owner outside audit scope.
- Projection: TrackWise finding fixture maps field-for-field.
- SLO: transition p95 under 230 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-IP_ACCEPTED`.
