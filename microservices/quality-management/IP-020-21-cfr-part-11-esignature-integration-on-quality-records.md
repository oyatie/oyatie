---
doc_class: ImplementationPlan
ip_id: IP-020
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
journey_ref: j123-regulated-quality-record-approval
sap_submodule: QM-CA Corrective and Preventive Actions
tenant_class: paid
billing_components:
  - per_usage
persona: Dr. Anika Rao, regulated quality approver
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-020: 21 CFR Part 11 e-signature integration on quality records

## Context

- SAP QM submodule: QM-CA Corrective and Preventive Actions.
- Topic: 21 CFR Part 11 e-signature integration on quality records.
- Persona: Dr. Anika Rao, regulated quality approver.
- Journey: j123 regulated quality record approval.
- Journey leg: CAPA, certificate, audit finding, or usage decision requires signed approval.
- SAP precedent: digital signature strategy for QM transactions.
- Oyatie aggregate: `QualityRecordSignature`.
- Boundary: signature challenge, record hash, signer meaning, and approval gate.
- ADR-0105 keeps signature domain separate from external signature provider.
- ADR-0131 keeps the IP local to quality-management.
- ADR-0244 protects tenant and signer identity.
- ADR-0263 binds signature audit events.
- ADR-0297 requires Cedar before signature acceptance.
- ADR-0314 keeps marketplace settlement outside signature flow.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Signature is bound to exact record hash and stated meaning.
- Signature cannot be copied from one record version to another.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.quality_record_signature (
  tenant_id UUID NOT NULL,
  signature_id TEXT NOT NULL,
  record_type TEXT NOT NULL,
  record_id TEXT NOT NULL,
  record_version TEXT NOT NULL,
  record_hash TEXT NOT NULL,
  signer_principal_id TEXT NOT NULL,
  signature_meaning TEXT NOT NULL,
  signature_provider TEXT NOT NULL,
  signature_state TEXT NOT NULL,
  signed_at TIMESTAMPTZ,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  PRIMARY KEY (tenant_id, signature_id)
);
CREATE TABLE quality_management.signature_challenge (
  tenant_id UUID NOT NULL,
  challenge_id TEXT NOT NULL,
  signature_id TEXT NOT NULL,
  challenge_hash TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  challenge_state TEXT NOT NULL,
  PRIMARY KEY (tenant_id, challenge_id)
);
```

### Rust Types

```rust
pub struct QualityRecordSignature {
    pub tenant_id: TenantId,
    pub signature_id: SignatureId,
    pub record_type: QualityRecordType,
    pub record_id: RecordId,
    pub record_version: RecordVersion,
    pub record_hash: RecordHash,
    pub signer_principal_id: PrincipalId,
    pub signature_meaning: SignatureMeaning,
    pub signature_provider: SignatureProvider,
    pub state: SignatureState,
    pub signed_at: Option<DateTime<Utc>>,
}
pub enum QualityRecordType { UsageDecision, CertificateRelease, AuditFindingClose, CapaEffectiveness, HoldRelease }
pub enum SignatureMeaning { Reviewed, Approved, Released, Rejected, VerifiedEffective }
pub enum SignatureState { ChallengeIssued, Signed, Rejected, Expired, Revoked }
pub enum SignatureError {
    RecordHashChanged,
    ChallengeExpired,
    SignerNotAuthorized,
    MeaningNotAllowedForRecord,
    ProviderAttestationInvalid,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/quality-record-signatures`.
- Creates signature challenge for a quality record.
- `POST /v1/quality-management/quality-record-signatures/{signature_id}:complete`.
- Completes signature with provider attestation.
- `POST /v1/quality-management/quality-record-signatures/{signature_id}:revoke`.
- Revokes signature when record is superseded.
- `GET /v1/quality-management/quality-record-signatures/{signature_id}`.
- Returns signer, meaning, record hash, and state.

### gRPC

- Service: `quality_management.signature.v1.QualityRecordSignatureService`.
- `rpc CreateSignatureChallenge(CreateSignatureChallengeRequest) returns (SignatureChallengeView)`.
- `rpc CompleteSignature(CompleteSignatureRequest) returns (SignatureReceipt)`.
- `rpc RevokeSignature(RevokeSignatureRequest) returns (SignatureReceipt)`.
- `rpc StreamSignatureEvents(StreamSignatureEventsRequest) returns (stream QualitySignatureEvent)`.

### AsyncAPI

- Channel: `quality-management.signature.challenge-issued.v1`.
- Channel: `quality-management.signature.completed.v1`.
- Channel: `quality-management.signature.revoked.v1`.
- Message: `QualitySignatureChallengeIssued`.
- Message: `QualitySignatureCompleted`.
- Payload includes `signature_id`, `record_type`, `record_id`, `record_hash`, `signature_meaning`, `signature_state`, `audit_event_class`.
- Consumers: CAPA, certificate publication, audit finding, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::signature::challenge`.
- Principal: `QualityApprover`.
- Action: `signature_challenge_create`.
- Resource: `QualityRecord`.
- Context: `record_type`, `record_hash`, `signature_meaning`, `pack_ids`, `mfa_state`.
- Policy: `quality_management::signature::complete`.
- Principal: `QualityApprover`.
- Action: `signature_complete`.
- Resource: `QualityRecordSignature`.
- Context: `provider_attestation`, `challenge_state`, `record_hash_current`, `signer_role`.
- Forbid: current record hash differs from challenge hash.
- Forbid: challenge expired.
- Forbid: signer role cannot apply requested meaning.
- Forbid: provider attestation is invalid.

## Ontology Projection

- Vendor object: SAP QM digital signature strategy.
- Oyatie object: `quality_management.quality_record_signature`.
- SAP signature strategy -> allowed signature meaning.
- SAP signer user -> `signer_principal_id`.
- SAP signed transaction -> `record_type`.
- SAP object key -> `record_id`.
- SAP digital signature hash -> `record_hash`.
- SAP signature timestamp -> `signed_at`.
- MasterControl approval signature -> signature completed.
- TrackWise electronic signature -> quality record signature.
- ETQ Reliance approval -> signature completed.
- IQS-AQM approval -> signature record.
- Projection freshness floor: 5 seconds.
- Projection consumer: compliance and quality record owners.
- Projection rule: provider attestation stays evidence, not ontology payload.

## Workflow Steps

- Node `record-freeze`: quality record version is frozen.
- Node `record-hash`: canonical hash is computed.
- Node `meaning-select`: approver selects signature meaning.
- Decision `meaning-not-allowed`: reject challenge.
- Node `cedar-challenge`: evaluate challenge policy.
- Node `challenge-issue`: signature challenge created.
- Node `provider-auth`: signer completes provider authentication.
- Decision `challenge-expired`: mark expired.
- Node `provider-attestation-verify`: verify external attestation.
- Decision `attestation-invalid`: reject signature.
- Node `hash-recheck`: compare current record hash.
- Decision `record-hash-changed`: revoke challenge and require new version.
- Node `cedar-complete`: evaluate complete policy.
- Node `signature-complete`: state `Signed`.
- Node `record-unblock`: unblock dependent CAPA, CoA, finding, or hold release.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish signature metadata.
- Node `close`: signature immutable unless revoked.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-SIGNATURE-CHALLENGE_ISSUED`.
- `EVT-QUALITY_MANAGEMENT-SIGNATURE-COMPLETED`.
- `EVT-QUALITY_MANAGEMENT-SIGNATURE-REVOKED`.
- `EVT-QUALITY_MANAGEMENT-SIGNATURE-POLICY_DENIED`.
- `EVT-QUALITY_MANAGEMENT-SIGNATURE-IP_ACCEPTED`.
- ADR-0263 envelope stores `signature_id`.
- ADR-0263 envelope stores `record_type`.
- ADR-0263 envelope stores `record_hash`.
- ADR-0263 envelope stores `signature_meaning`.
- ADR-0263 envelope stores `signature_provider`.

## SLO Targets

- Challenge create p50: 80 ms.
- Challenge create p95: 300 ms.
- Complete signature p95: 800 ms excluding provider latency.
- Signature lookup p95: 100 ms.
- Throughput: 100 signature challenges per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: regulated approvals block release workflows but depend partly on external provider latency.

## Failure Modes and Recovery

- Failure: record hash changes after challenge.
- Recovery: `SIGNATURE-HASH-CHANGED-REVOKE` revokes challenge and requires new signature.
- Failure: challenge expires.
- Recovery: `SIGNATURE-CHALLENGE-EXPIRED` marks expired and lets approver recreate.
- Failure: signer role not authorized for meaning.
- Recovery: `SIGNATURE-ROLE-DENY` blocks completion.
- Failure: provider attestation invalid.
- Recovery: `SIGNATURE-ATTESTATION-REJECT` stores provider evidence and rejects.
- Failure: dependent workflow misses signed event.
- Recovery: `SIGNATURE-OUTBOX-REPLAY` replays completed event.
- Failure: provider outage during completion.
- Recovery: `SIGNATURE-PROVIDER-RETRY` keeps challenge pending until expiry.

## Migration Notes

- Source vendor: SAP QM.
- Migrate digital signature records from QM transactions where hash and signer exist.
- Source vendor: MasterControl maps approval signatures into signature records.
- Source vendor: TrackWise maps e-sign approvals into signature completed records.
- Source vendor: ETQ Reliance maps workflow approvals into signature meaning.
- Source vendor: IQS-AQM maps audit approval signatures into record signatures.
- Migrated signatures without record hash become evidence-only and cannot unblock records.
- Provider attestation may be migrated as detached evidence.
- Rollback path: require manual signature recreation for migrated records.
- Part 11 mode requires MFA state in Cedar context.

## Cross-microservice Handoffs

- From CAPA: effectiveness verification signature request.
- From certificate-of-analysis: release signature request.
- From audit-finding: closure signature request.
- From quality-hold: release signature request.
- To compliance: Part 11 signature evidence.
- To identity: signer role and MFA validation.
- To ontology: signature metadata projection.
- To workflow-engine: signature task state.

## Verification

- Unit: record hash change revokes challenge.
- Unit: expired challenge cannot complete.
- Unit: unauthorized meaning denied.
- Contract: REST complete returns signed state.
- Contract: gRPC stream emits completed event.
- Event: signature completed event validates.
- Policy: Cedar denies invalid provider attestation.
- Projection: MasterControl signature fixture maps field-for-field.
- SLO: challenge create p95 under 300 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-SIGNATURE-IP_ACCEPTED`.
