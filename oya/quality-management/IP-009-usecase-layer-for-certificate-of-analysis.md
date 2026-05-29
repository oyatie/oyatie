---
doc_class: ImplementationPlan
ip_id: IP-009
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
journey_ref: j102-customer-facing-compliance-release
sap_submodule: QM-QC Quality Control
tenant_class: paid
billing_components:
  - per_usage
persona: Hana Cho, customer quality manager
status: Accepted
date: 2026-05-20
owner_team: axis-quality-management + axis-erp-parity
---

# IP-009: Usecase layer for certificate publication and customer mirror

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: customer-facing certificate mirror.
- Persona: Hana Cho, customer quality manager.
- Journey: j102 customer-facing compliance release.
- Journey leg: accepted lot becomes customer-readable evidence before shipment.
- SAP precedent: quality certificate output determination and customer certificate profile.
- Oyatie usecase: `PublishCertificateOfAnalysis`.
- Boundary: orchestration across accepted usage decision, signature, customer mirror, and warehouse release.
- ADR-0105 places orchestration in usecase.
- ADR-0131 keeps the file local to the microservice.
- ADR-0244 protects customer and tenant scoping.
- ADR-0263 binds publication audit events.
- ADR-0297 requires Cedar checks before external visibility.
- ADR-0314 prevents settlement mutation.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires implementation-ready detail.
- Publication is not PDF generation; it is a signed external projection.
- Customer mirror is revocable and traceable.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.certificate_publication (
  tenant_id UUID NOT NULL,
  publication_id TEXT NOT NULL,
  certificate_id TEXT NOT NULL,
  customer_id TEXT NOT NULL,
  mirror_uri TEXT NOT NULL,
  publication_state TEXT NOT NULL,
  visibility_profile_id TEXT NOT NULL,
  expires_at TIMESTAMPTZ,
  released_by_principal_id TEXT NOT NULL,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, publication_id),
  UNIQUE (tenant_id, certificate_id, customer_id)
);
CREATE TABLE quality_management.certificate_publication_access (
  tenant_id UUID NOT NULL,
  publication_id TEXT NOT NULL,
  access_subject_id TEXT NOT NULL,
  access_scope TEXT NOT NULL,
  granted_hlc TEXT NOT NULL,
  revoked_hlc TEXT,
  PRIMARY KEY (tenant_id, publication_id, access_subject_id)
);
```

### Rust Types

```rust
pub struct CertificatePublication {
    pub tenant_id: TenantId,
    pub publication_id: PublicationId,
    pub certificate_id: CertificateId,
    pub customer_id: CustomerId,
    pub mirror_uri: MirrorUri,
    pub state: PublicationState,
    pub visibility_profile_id: VisibilityProfileId,
    pub expires_at: Option<DateTime<Utc>>,
    pub access: Vec<CertificatePublicationAccess>,
}
pub enum PublicationState { Pending, Published, Revoked, Expired }
pub enum AccessScope { View, Download, VerifySignature }
pub enum CertificatePublicationError {
    CertificateNotReleased,
    CustomerNotEntitled,
    VisibilityProfileDenied,
    MirrorWriteFailed,
    AccessGrantOutsideTenant,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/certificates-of-analysis/{certificate_id}:publish`.
- Publishes released certificate to customer mirror.
- `POST /v1/quality-management/certificate-publications/{publication_id}:grant-access`.
- Grants customer-facing subject access.
- `POST /v1/quality-management/certificate-publications/{publication_id}:revoke`.
- Revokes mirror and access grants.
- `GET /v1/quality-management/certificate-publications/{publication_id}`.
- Returns mirror state and signature verification metadata.

### gRPC

- Service: `quality_management.certificate_publication.v1.CertificatePublicationService`.
- `rpc PublishCertificate(PublishCertificateRequest) returns (PublicationReceipt)`.
- `rpc GrantPublicationAccess(GrantPublicationAccessRequest) returns (PublicationReceipt)`.
- `rpc RevokePublication(RevokePublicationRequest) returns (PublicationReceipt)`.
- `rpc VerifyPublication(VerifyPublicationRequest) returns (PublicationVerification)`.

### AsyncAPI

- Channel: `quality-management.certificate-publication.published.v1`.
- Channel: `quality-management.certificate-publication.revoked.v1`.
- Message: `CertificatePublicationPublished`.
- Message: `CertificatePublicationRevoked`.
- Payload includes `publication_id`, `certificate_id`, `customer_id`, `mirror_uri`, `visibility_profile_id`, `audit_event_class`.
- Consumers: customer-portal, warehouse, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::certificate_publication::publish`.
- Principal: `CustomerQualityManager`.
- Action: `certificate_publish`.
- Resource: `CertificateOfAnalysis`.
- Context: `customer_id`, `customer_contract_ref`, `visibility_profile_id`, `pack_ids`.
- Policy: `quality_management::certificate_publication::grant_access`.
- Principal: `CustomerQualityManager`.
- Action: `certificate_publication_grant_access`.
- Resource: `CertificatePublication`.
- Context: `access_subject_id`, `access_scope`, `customer_tenant_link`.
- Forbid: certificate state is not released.
- Forbid: customer is not entitled to the batch.
- Forbid: visibility profile exposes internal-only characteristic.
- Forbid: access subject lacks customer tenant link.

## Ontology Projection

- Vendor object: SAP QM quality certificate output.
- Oyatie object: `quality_management.certificate_publication`.
- SAP certificate number -> `certificate_id`.
- SAP output recipient -> `customer_id`.
- SAP output language -> certificate language in mirror.
- SAP output status -> `publication_state`.
- SAP print archive id -> `mirror_uri`.
- SAP customer partner function -> access subject mapping.
- SAP revocation/correction -> `PublicationState::Revoked`.
- MasterControl published document -> `mirror_uri`.
- ETQ Reliance customer complaint package -> publication context.
- Projection freshness floor: 10 seconds.
- Projection rule: publication view is customer-safe only.
- Projection consumer: customer portal verifies signature from mirror metadata.

## Workflow Steps

- Node `certificate-released`: IP-003 release event arrives.
- Node `entitlement-load`: customer contract and batch entitlement loaded.
- Decision `customer-not-entitled`: block publication.
- Node `visibility-profile-load`: customer-safe fields loaded.
- Decision `internal-line-exposed`: fail closed.
- Node `mirror-render`: signed JSON and optional PDF projection generated.
- Node `cedar-publish`: evaluate publish policy.
- Node `mirror-write`: write immutable customer mirror.
- Decision `mirror-write-fail`: retry from outbox.
- Node `access-grant`: grant default customer access subjects.
- Decision `regulated-pack`: add expiry and verification requirements.
- Node `warehouse-release-ack`: tell warehouse certificate exists.
- Node `customer-notify`: ask mail/connect through notification boundary.
- Node `audit-seal`: emit ADR-0263 class.
- Node `ontology-project`: publish mirror metadata.
- Node `revocation-watch`: listen for certificate revoke.
- Decision `certificate-revoked`: revoke publication.
- Node `close`: publication is published or revoked.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_PUBLICATION-PUBLISHED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_PUBLICATION-ACCESS_GRANTED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_PUBLICATION-REVOKED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-RELEASED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-IP_ACCEPTED`.
- ADR-0263 envelope stores `publication_id`.
- ADR-0263 envelope stores `customer_id`.
- ADR-0263 envelope stores `mirror_uri`.
- ADR-0263 envelope stores `visibility_profile_id`.
- ADR-0263 envelope stores `access_subject_id`.

## SLO Targets

- Publication p50: 160 ms.
- Publication p95: 800 ms.
- Publication p99: 2 seconds.
- Verify customer mirror p95: 150 ms.
- Throughput: 50 publications per second per cell.
- Availability: 99.9 percent monthly.
- Rationale: publication can tolerate retries, but mirror verification must be fast for shipment review.

## Failure Modes and Recovery

- Failure: customer entitlement cannot be proven.
- Recovery: `COA-PUB-ENTITLEMENT-BLOCK` keeps certificate internal.
- Failure: visibility profile exposes an internal-only line.
- Recovery: `COA-PUB-REDO-PROFILE` blocks mirror and creates profile repair task.
- Failure: mirror write fails after release.
- Recovery: `COA-PUB-OUTBOX-RETRY` retries immutable write.
- Failure: publication access is granted to wrong customer tenant.
- Recovery: `COA-PUB-ACCESS-REVOKE` revokes grant and audits incident.
- Failure: certificate is revoked after publication.
- Recovery: `COA-PUB-REVOCATION-CASCADE` revokes mirror and notifies customer portal.
- Failure: customer portal cannot verify signature.
- Recovery: `COA-PUB-SIGNATURE-REBUILD` regenerates signature metadata from release snapshot.

## Migration Notes

- Source vendor: SAP QM.
- Migrate certificate output records as publication snapshots.
- Preserve recipient and archive id where available.
- Source vendor: MasterControl maps published quality documents to mirrors.
- Source vendor: ETQ Reliance maps customer complaint evidence packages to mirror candidates.
- Source vendor: Sparta Systems TrackWise maps batch release approvals into publication readiness.
- Historical PDFs remain derived artifacts, not canonical source.
- Access grants are not migrated unless source vendor has subject identity proof.
- Rollback path: revoke all new mirrors and retain internal certificates.
- Customer notifications are replayable from published event.

## Cross-microservice Handoffs

- From certificate-of-analysis: released certificate event.
- To customer-portal: customer-safe mirror.
- To warehouse: shipment gate satisfaction.
- To compliance: publication evidence.
- To mail/connect: customer notification request.
- To ontology: publication projection.
- To identity/tenancy: customer subject access proof.
- To marketplace: read-only quality evidence for buyer trust.

## Verification

- Unit: unreleased certificate cannot publish.
- Unit: internal-only characteristic blocks mirror.
- Unit: wrong customer entitlement denied.
- Contract: REST publish returns mirror URI and verification hash.
- Contract: gRPC verify returns signature metadata.
- Event: published event validates.
- Policy: Cedar denies access subject outside customer tenant.
- Projection: SAP output fixture maps field-for-field.
- SLO: mirror verify p95 under 150 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-IP_ACCEPTED`.
