---
doc_class: ImplementationPlan
ip_id: IP-003
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

# IP-003: Domain layer for certificate-of-analysis release

## Context

- SAP QM submodule: QM-QC Quality Control.
- Topic: certificate of analysis from released inspection results.
- Persona: Hana Cho, customer quality manager.
- Journey: j102 customer-facing compliance release.
- Journey leg: finished batch ships only after customer-facing quality certificate is sealed.
- SAP precedent: QM certificate profiles and batch characteristics.
- Oyatie aggregate: `CertificateOfAnalysis`.
- Boundary: domain rules for eligibility, content, and certificate state.
- ADR-0105 keeps certificate invariants inside the domain layer.
- ADR-0131 keeps this IP in the microservice folder.
- ADR-0244 requires tenant-scoped customer views.
- ADR-0263 defines certificate audit classes.
- ADR-0297 requires policy proof before release.
- ADR-0314 keeps revenue settlement outside this service.
- ADR-0315 requires SAP QM parity.
- ADR-0329/0330/0331 requires ERP-grade implementation depth.
- Certificate release must be reproducible from immutable inspection evidence.
- Certificate release must distinguish internal result from customer-visible claim.

## Data Model Deltas

### PostgreSQL DDL

```sql
CREATE TABLE quality_management.certificate_of_analysis (
  tenant_id UUID NOT NULL,
  certificate_id TEXT NOT NULL,
  inspection_lot_id TEXT NOT NULL,
  material_id TEXT NOT NULL,
  batch_id TEXT NOT NULL,
  customer_id TEXT,
  certificate_profile_id TEXT NOT NULL,
  state TEXT NOT NULL,
  language_code TEXT NOT NULL,
  released_by_principal_id TEXT,
  released_at TIMESTAMPTZ,
  signature_ref TEXT,
  cedar_decision_id TEXT NOT NULL,
  audit_event_class TEXT NOT NULL,
  created_hlc TEXT NOT NULL,
  PRIMARY KEY (tenant_id, certificate_id)
);
CREATE TABLE quality_management.certificate_characteristic (
  tenant_id UUID NOT NULL,
  certificate_id TEXT NOT NULL,
  line_no INTEGER NOT NULL,
  characteristic_code TEXT NOT NULL,
  reported_value TEXT NOT NULL,
  reported_unit TEXT,
  lower_spec TEXT,
  upper_spec TEXT,
  visible_to_customer BOOLEAN NOT NULL,
  evidence_ref TEXT NOT NULL,
  PRIMARY KEY (tenant_id, certificate_id, line_no)
);
```

### Rust Types

```rust
pub struct CertificateOfAnalysis {
    pub tenant_id: TenantId,
    pub certificate_id: CertificateId,
    pub inspection_lot_id: InspectionLotId,
    pub material_id: MaterialId,
    pub batch_id: BatchId,
    pub customer_id: Option<CustomerId>,
    pub profile_id: CertificateProfileId,
    pub language_code: LanguageCode,
    pub state: CertificateState,
    pub characteristics: Vec<CertificateCharacteristic>,
    pub signature_ref: Option<SignatureRef>,
}
pub struct CertificateCharacteristic {
    pub line_no: u16,
    pub characteristic_code: CharacteristicCode,
    pub reported_value: ReportedValue,
    pub reported_unit: Option<UnitOfMeasure>,
    pub specification: Option<SpecificationLimit>,
    pub visible_to_customer: bool,
    pub evidence_ref: EvidenceRef,
}
pub enum CertificateState { Draft, PendingReview, Released, Revoked }
pub enum CertificateError {
    MissingAcceptedUsageDecision,
    CustomerHiddenRequiredLine,
    ResultEvidenceNotImmutable,
    CrossTenantEvidence,
    ReleaseWithoutSignature,
}
```

## API Endpoints

### REST

- `POST /v1/quality-management/certificates-of-analysis`.
- Drafts a certificate from a lot and certificate profile.
- `POST /v1/quality-management/certificates-of-analysis/{certificate_id}:review`.
- Moves draft into review and freezes visible lines.
- `POST /v1/quality-management/certificates-of-analysis/{certificate_id}:release`.
- Releases with signature reference and customer visibility policy.
- `POST /v1/quality-management/certificates-of-analysis/{certificate_id}:revoke`.
- Revokes when source evidence is corrected.
- `GET /v1/quality-management/certificates-of-analysis/{certificate_id}/customer-view`.
- Returns only customer-visible lines.

### gRPC

- Service: `quality_management.certificate.v1.CertificateOfAnalysisService`.
- `rpc DraftCertificate(DraftCertificateRequest) returns (CertificateReceipt)`.
- `rpc ReleaseCertificate(ReleaseCertificateRequest) returns (CertificateReceipt)`.
- `rpc RevokeCertificate(RevokeCertificateRequest) returns (CertificateReceipt)`.
- `rpc GetCustomerCertificate(GetCustomerCertificateRequest) returns (CustomerCertificateView)`.

### AsyncAPI

- Channel: `quality-management.certificate-of-analysis.released.v1`.
- Channel: `quality-management.certificate-of-analysis.revoked.v1`.
- Message: `CertificateOfAnalysisReleased`.
- Message: `CertificateOfAnalysisRevoked`.
- Payload includes `certificate_id`, `inspection_lot_id`, `batch_id`, `customer_id`, `signature_ref`, `audit_event_class`.
- Consumers: customer-portal, warehouse shipping, compliance, ontology.

## Cedar Policy Hooks

- Policy: `quality_management::certificate_of_analysis::draft`.
- Principal: `QualityManager`.
- Action: `certificate_draft`.
- Resource: `InspectionLotResultSet`.
- Context: `tenant_id`, `usage_decision`, `result_lock_state`, `customer_contract_ref`.
- Policy: `quality_management::certificate_of_analysis::release`.
- Principal: `QualityManager` or `QualifiedPerson`.
- Action: `certificate_release`.
- Resource: `CertificateOfAnalysis`.
- Context: `signature_ref`, `customer_visibility_profile`, `pack_ids`, `language_code`.
- Forbid: inspection lot usage decision is not `Accepted`.
- Forbid: result evidence is mutable.
- Forbid: required customer line is hidden.
- Forbid: release without signature for regulated pack.

## Ontology Projection

- Vendor object: SAP QM certificate profile.
- Oyatie object: `quality_management.certificate_of_analysis`.
- `QCERT-CERT_NO` -> `certificate_id`.
- `QALS-PRUEFLOS` -> `inspection_lot_id`.
- `MCH1-CHARG` -> `batch_id`.
- `KNA1-KUNNR` -> `customer_id`.
- SAP characteristic name -> `characteristic_code`.
- SAP result value -> `reported_value`.
- SAP result unit -> `reported_unit`.
- SAP tolerance lower -> `lower_spec`.
- SAP tolerance upper -> `upper_spec`.
- SAP certificate language -> `language_code`.
- SAP release status -> `state`.
- SAP digital signature -> `signature_ref`.
- Profile line visibility -> `visible_to_customer`.
- Projection freshness floor: 10 seconds.
- Projection mode: immutable release snapshot.

## Workflow Steps

- Node `lot-accepted`: certificate can only start after accepted usage decision.
- Node `profile-load`: customer and material profile is selected.
- Decision `missing-profile`: create planner task, no draft.
- Node `result-freeze-check`: confirm source results are immutable.
- Decision `mutable-result`: block certificate draft.
- Node `line-project`: map result characteristics to certificate lines.
- Decision `customer-hidden-required`: fail release.
- Node `language-render`: render labels in customer language.
- Node `cedar-draft`: evaluate draft policy.
- Node `draft-create`: state `Draft`.
- Node `review-submit`: state `PendingReview`.
- Decision `regulated-pack`: require qualified-person signature.
- Node `signature-bind`: attach signature reference.
- Node `cedar-release`: evaluate release policy.
- Node `release-seal`: state `Released`.
- Node `customer-mirror-publish`: publish customer-safe view.
- Node `shipping-ack`: notify warehouse.
- Node `audit-seal`: emit ADR-0263 class.
- Node `close`: certificate is immutable except revoke.

## Audit Events

- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-CHANGED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-DRAFTED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-RELEASED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-REVOKED`.
- `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-IP_ACCEPTED`.
- ADR-0263 envelope stores `certificate_profile_id`.
- ADR-0263 envelope stores `customer_id`.
- ADR-0263 envelope stores `signature_ref`.
- ADR-0263 envelope stores `visible_line_count`.
- ADR-0263 envelope stores `source_result_hash`.

## SLO Targets

- Draft latency p50: 90 ms.
- Draft latency p95: 350 ms.
- Draft latency p99: 900 ms.
- Customer view fetch p95: 120 ms.
- Throughput: 80 releases per second per cell.
- Availability: 99.95 percent monthly.
- Rationale: shipping gates can wait seconds, but customer view reads must remain quick.

## Failure Modes and Recovery

- Failure: lot usage decision is still pending.
- Recovery: `COA-USAGE-DECISION-BLOCK` keeps certificate absent and links lot task.
- Failure: certificate profile is missing a required customer line.
- Recovery: `COA-PROFILE-REPAIR` opens controlled profile amendment.
- Failure: result evidence changes after draft.
- Recovery: `COA-DRAFT-INVALIDATE` revokes draft and replays from immutable result set.
- Failure: regulated release lacks signature reference.
- Recovery: `COA-SIGNATURE-RETRY` routes to e-signature provider.
- Failure: customer mirror publish fails.
- Recovery: `COA-CUSTOMER-MIRROR-REPLAY` replays released event.
- Failure: wrong customer visibility pack is selected.
- Recovery: `COA-VISIBILITY-ROLLBACK` revokes and reissues with corrected profile.

## Migration Notes

- Source vendor: SAP QM.
- Migrate released certificates from certificate profile output and batch result links.
- Preserve SAP certificate number as `source_certificate_no`.
- Preserve original language as `language_code`.
- Source vendor: MasterControl maps approved quality documents into profile attachments.
- Source vendor: ETQ Reliance maps customer complaint certificate attachments into evidence refs.
- Source vendor: Sparta Systems TrackWise maps batch release records as certificate sources.
- Revoke migrated certificates when source vendor marks superseded or corrected.
- Rollback path: keep release snapshots but disable customer mirror publication.
- Customer-facing PDFs remain derived artifacts, not canonical records.

## Cross-microservice Handoffs

- From inspection-lot: accepted usage decision.
- From quality-control result recording: immutable characteristic values.
- To warehouse: shipping release gate.
- To customer-portal: customer-safe certificate mirror.
- To compliance: regulated batch release evidence.
- To ontology: certificate release projection.
- To workflow-engine: review and signature state.
- To marketplace: supplier/customer evidence visibility only.

## Verification

- Unit: release denied without accepted usage decision.
- Unit: hidden required line fails release.
- Unit: immutable result evidence required.
- Contract: REST customer view redacts internal-only lines.
- Contract: gRPC release requires signature for regulated pack.
- Event: released event validates.
- Policy: Cedar denies cross-tenant result evidence.
- Projection: SAP certificate fixture maps field-for-field.
- SLO: customer view p95 under 120 ms.
- Evidence: emit `EVT-QUALITY_MANAGEMENT-CERTIFICATE_OF_ANALYSIS-IP_ACCEPTED`.
