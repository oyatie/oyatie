---
doc_class: ImplementationPlan
ip_id: IP-006
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

# IP-006: Domain layer for audit-evidence and audit plan templates

## Context

- SAP QM submodule: QM-AU Quality Audits.
- Topic: audit plan template and objective evidence binding.
- Persona: Omar Haddad, quality auditor.
- Journey: j119 quality audit readiness.
- Journey leg: auditor prepares a supplier or internal process audit.
- SAP precedent: audit question lists, audit plans, audit objects, and evidence attachments.
- Oyatie aggregate: `QualityAuditEvidence`.
- Boundary: audit template, evidence claim, finding link, and immutable evidence state.
- ADR-0105 keeps audit evidence in the domain ring.
- ADR-0131 keeps this implementation plan inside the microservice.
- ADR-0244 requires tenant and supplier-audit isolation.
- ADR-0263 defines audit event class names.
- ADR-0297 requires policy decisions before evidence attachment.
- ADR-0314 prevents marketplace settlement mutation.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready ERP depth.
- Evidence is not a generic file attachment.
- Evidence is a typed assertion with provenance and clause coverage.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.audit_plan_template (
  tenant_id UUID NOT NULL,
  template_id TEXT NOT NULL,
  template_code TEXT NOT NULL,
  audit_type TEXT NOT NULL,
  standard_ref TEXT NOT NULL,
  version_no INTEGER NOT NULL,
  state TEXT NOT NULL,
  owner_team TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, template_id, version_no)
);
CREATE TABLE quality_management.audit_evidence (
  tenant_id UUID NOT NULL,
  evidence_id TEXT NOT NULL,
  audit_id TEXT NOT NULL,
  template_id TEXT NOT NULL,
  clause_ref TEXT NOT NULL,
  evidence_kind TEXT NOT NULL,
  evidence_uri TEXT NOT NULL,
  source_system TEXT NOT NULL,
  evidence_hash TEXT NOT NULL,
  state TEXT NOT NULL,
  collected_by_principal_id TEXT NOT NULL,
  PRIMARY KEY (tenant_id, evidence_id)
);
```

### Rust Types

```rust
pub struct AuditPlanTemplate {
    pub tenant_id: TenantId,
    pub template_id: AuditTemplateId,
    pub template_code: TemplateCode,
    pub audit_type: AuditType,
    pub standard_ref: StandardRef,
    pub version_no: RevisionNo,
    pub state: TemplateState,
    pub questions: Vec<AuditQuestion>,
}
pub struct QualityAuditEvidence {
    pub tenant_id: TenantId,
    pub evidence_id: EvidenceId,
    pub audit_id: AuditId,
    pub template_id: AuditTemplateId,
    pub clause_ref: ClauseRef,
    pub evidence_kind: EvidenceKind,
    pub evidence_uri: EvidenceUri,
    pub evidence_hash: EvidenceHash,
    pub source_system: SourceSystem,
    pub state: EvidenceState,
}
pub enum AuditType { InternalProcess, Supplier, Customer, Regulatory, System }
pub enum EvidenceKind { Observation, Interview, Record, Photo, Measurement, SystemLog }
pub enum EvidenceState { Draft, Collected, Verified, Rejected, Superseded }
pub enum AuditEvidenceError {
    MissingClauseRef,
    HashMismatch,
    TemplateNotReleased,
    EvidenceUriNotAllowed,
    CrossTenantAudit,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/audit-plan-templates`.
- Creates or revises an audit plan template.
- `POST /v1/quality-management/audit-plan-templates/{template_id}:release`.
- Releases template version for use.
- `POST /v1/quality-management/audit-evidence`.
- Attaches typed evidence to an audit and clause.
- `POST /v1/quality-management/audit-evidence/{evidence_id}:verify`.
- Moves evidence into verified state after hash and clause review.
- `GET /v1/quality-management/audit-evidence/{evidence_id}`.
- Returns evidence metadata, not raw blob.

### gRPC

- Service: `quality_management.audit.v1.AuditEvidenceService`.
- `rpc CreateTemplate(CreateAuditTemplateRequest) returns (AuditTemplateReceipt)`.
- `rpc ReleaseTemplate(ReleaseAuditTemplateRequest) returns (AuditTemplateReceipt)`.
- `rpc AttachEvidence(AttachEvidenceRequest) returns (AuditEvidenceReceipt)`.
- `rpc VerifyEvidence(VerifyEvidenceRequest) returns (AuditEvidenceReceipt)`.
- `rpc StreamAuditEvidence(StreamAuditEvidenceRequest) returns (stream AuditEvidenceEvent)`.

### AsyncAPI

- Channel: `quality-management.audit-template.released.v1`.
- Channel: `quality-management.audit-evidence.collected.v1`.
- Channel: `quality-management.audit-evidence.verified.v1`.
- Message: `AuditTemplateReleased`.
- Message: `AuditEvidenceCollected`.
- Payload carries `audit_id`, `template_id`, `clause_ref`, `evidence_kind`, `evidence_hash`, `audit_event_class`.
- Consumers: compliance, workflow-engine, supplier-quality, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::audit_template::release`.
- Principal: `LeadAuditor`.
- Action: `audit_template_release`.
- Resource: `AuditPlanTemplate`.
- Context: `tenant_id`, `standard_ref`, `reviewer_count`, `pack_ids`.
- Policy: `quality_management::audit_evidence::attach`.
- Principal: `Auditor`.
- Action: `audit_evidence_attach`.
- Resource: `Audit`.
- Context: `supplier_id`, `audit_scope`, `evidence_kind`, `allowed_uri_scheme`.
- Forbid: evidence URI uses unmanaged storage.
- Forbid: template version is not released.
- Forbid: audit supplier scope excludes supplier id.
- Forbid: clause ref is absent for regulated packs.

## Ontology Projection

- Vendor object: SAP QM audit management question list.
- Oyatie object: `quality_management.audit_plan_template`.
- Vendor audit plan id -> `template_id`.
- Vendor audit type -> `audit_type`.
- Vendor checklist question -> `AuditQuestion`.
- Vendor standard clause -> `clause_ref`.
- Vendor evidence attachment id -> `evidence_id`.
- Vendor evidence file hash -> `evidence_hash`.
- Vendor evidence source -> `source_system`.
- Vendor auditor id -> `collected_by_principal_id`.
- Vendor finding id -> downstream finding reference.
- Vendor audit status -> template or evidence `state`.
- IQS-AQM audit checklist -> `audit_plan_template`.
- MasterControl audit package -> `audit_evidence`.
- ETQ Reliance audit object -> `audit_id`.
- Projection freshness floor: 15 seconds.
- Projection access: compliance can read, raw evidence stays in storage boundary.

## Workflow Steps

- Node `template-draft`: auditor authors plan template.
- Node `clause-map`: questions bind to ISO or pack clauses.
- Decision `missing-clause`: block regulated template release.
- Node `peer-review`: lead auditor reviews question coverage.
- Decision `review-count-low`: send back to draft.
- Node `cedar-template-release`: evaluate release policy.
- Node `template-release`: state `Released`.
- Node `audit-start`: audit instance references template.
- Node `evidence-collect`: auditor attaches evidence metadata.
- Decision `bad-uri-scheme`: reject evidence.
- Node `hash-verify`: hash matches stored object.
- Decision `hash-mismatch`: reject and open incident.
- Node `clause-coverage-update`: clause coverage matrix updates.
- Node `evidence-verify`: evidence state `Verified`.
- Decision `evidence-rejected`: create finding candidate.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish evidence projection.
- Node `close`: evidence ready for finding lifecycle.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_TEMPLATE-RELEASED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-COLLECTED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-VERIFIED`.
- `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-IP_ACCEPTED`.
- ADR-0263 envelope stores `standard_ref`.
- ADR-0263 envelope stores `clause_ref`.
- ADR-0263 envelope stores `evidence_hash`.
- ADR-0263 envelope stores `source_system`.
- ADR-0263 envelope stores `audit_scope`.

## SLO Targets

- Evidence attach latency p50: 85 ms.
- Evidence attach latency p95: 300 ms.
- Evidence attach latency p99: 900 ms.
- Template release p95: 400 ms.
- Throughput: 100 evidence attachments per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: audit evidence is not as hot as lot creation, but loss is compliance-critical.

## Failure Modes and Recovery

- Failure: evidence hash does not match stored object.
- Recovery: `AUDIT-EVIDENCE-HASH-REJECT` rejects verification and opens incident.
- Failure: template has no clause coverage for regulated pack.
- Recovery: `AUDIT-TEMPLATE-CLAUSE-GAP` blocks release and creates review task.
- Failure: evidence URI is outside approved storage boundary.
- Recovery: `AUDIT-EVIDENCE-URI-DENY` rejects attachment with policy event.
- Failure: supplier audit evidence is attached to wrong supplier scope.
- Recovery: `AUDIT-SUPPLIER-SCOPE-REPAIR` moves evidence only through audited command.
- Failure: compliance projection lags beyond freshness floor.
- Recovery: `AUDIT-PROJECTION-REPLAY` rebuilds clause matrix from events.
- Failure: template release event is duplicated.
- Recovery: `AUDIT-TEMPLATE-IDEMPOTENT` accepts same version and ignores duplicate.

## Migration Notes

- Source vendor: SAP QM.
- Migrate audit plans as template versions.
- Migrate audit questions as `AuditQuestion`.
- Source vendor: IQS-AQM maps checklist sections to template clauses.
- Source vendor: MasterControl maps controlled audit packages to template plus evidence.
- Source vendor: ETQ Reliance maps audit objects and attachments into evidence records.
- Source vendor: Sparta Systems TrackWise maps audit findings into IP-012 lifecycle links.
- Preserve original evidence hash when vendor supplies it.
- Generate hash during migration when vendor lacks one.
- Rollback path: retain evidence metadata and disable verification transitions.

## Cross-microservice Handoffs

- To workflow-engine: audit review and evidence verification tasks.
- To compliance: clause coverage matrix.
- To supplier-quality scorecard: supplier audit evidence.
- To quality-notification: rejected evidence can open defect notification.
- To CAPA: finding can require corrective action.
- To ontology: audit evidence projection.
- To storage: raw object retrieval boundary.
- To marketplace: supplier audit badge read-only view.

## Verification

- Unit: template without clause ref denied for regulated pack.
- Unit: hash mismatch rejects evidence verification.
- Unit: unmanaged URI denied.
- Contract: REST evidence response omits raw blob.
- Contract: gRPC stream emits collected and verified events.
- Event: evidence verified schema validates.
- Policy: Cedar denies supplier scope mismatch.
- Projection: IQS-AQM checklist fixture maps field-for-field.
- SLO: evidence attach p95 under 300 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-AUDIT_EVIDENCE-IP_ACCEPTED`.
