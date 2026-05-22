---
doc_class: ImplementationPlan
ip_id: IP-018
microservice: global-trade
related_adrs:
  - ADR-0105
  - ADR-0243
  - ADR-0244
  - ADR-0253
  - ADR-0263
  - ADR-0304
  - ADR-0315
  - ADR-0329
  - ADR-0330
  - ADR-0331
journey_id: j102-raw-material-purchase-with-quality-attestation
journey_link: docs/user-journeys/j102-raw-material-purchase-with-quality-attestation/story.md
status: Accepted
date: 2026-05-20
owner: axis-global-trade
tenant_class_eligibility: [demo_trial, paid]
sap_submodule_equivalents:
  - SAP GTS-CC compliance-document-control
  - SAP GTS-PI preference-certificate-support
  - SAP GTS-EM electronic-message-and-output-management
---

# IP-018: Document compliance certificate generation

## Context With Why
- This IP builds certificate generation for trade compliance documents such as certificates of origin, preference statements, and export compliance certificates.
- The why is execution: shipments stall when trade documents are manually copied from ERP, classification, and supplier evidence.
- The feature must generate documents from approved data, not from free-form operator text.
- The journey leg starts when a shipment, sales order, customs declaration, or broker filing requests a compliance certificate.
- The journey leg ends when a signed document artifact, template version, evidence bundle, and audit event are available.
- Named persona: Priya, an export coordinator, needs to produce a certificate of origin for a same-day shipment without bypassing compliance approvals.
- Priya can choose an approved template and shipment line set, but she cannot edit HS code, origin, or preference values directly.
- This IP maps SAP GTS-CC document compliance and SAP GTS-PI preference certificate support into Oyatie document generation.
- SAP GTS-EM is relevant because generated certificates must be transmitted to brokers, customers, carriers, and customs message channels.
- The implementation must not own binary long-term archive storage; it stores document metadata and object refs.
- The implementation must not decide origin eligibility; it consumes approved origin and preference evidence.
- ADR-0105 keeps rendering adapters outside domain certificate rules.
- ADR-0243 requires source provenance for every field printed on the certificate.
- ADR-0244 requires tenant and data-residency scoping.
- ADR-0253 requires Cedar checks before preview, generate, sign, void, and transmit.
- ADR-0263 requires certificate generation and voiding events to be chainable.
- ADR-0304 requires ontology projection of TradeDocument, CertificateArtifact, and EvidenceBundle.
- ADR-0315 sets the SAP GTS parity target.
- Intern build target: one certificate request aggregate, one template selector, one field assembler, one renderer port, one signing port, and one transmit worker.

## Scope Boundaries
- In scope: certificate request, template selection, field assembly, preview, generate, sign, void, and transmit.
- In scope: certificate types for origin, preference, export compliance, dangerous goods declaration reference, and broker packet cover.
- In scope: evidence attachment for HS code, origin, preference, denied party clear result, and export control classification.
- In scope: PDF or structured XML output refs with immutable hashes.
- Out of scope: manual document design studio.
- Out of scope: storage of raw customer master or product master records beyond field snapshots.
- Out of scope: final customs filing; broker-filing owns submission.
- Boundary rule: certificate fields are assembled from approved source refs only.
- Boundary rule: preview output must be watermarked and cannot be transmitted.
- Boundary rule: voiding a certificate creates a new state and audit event; it never deletes the artifact.
- Boundary rule: templates are versioned and tenant-approved before use.

## Data Model Deltas
- Table: `global_trade_certificate_request`.
- Column: `tenant_id uuid not null`.
- Column: `certificate_request_id uuid primary key`.
- Column: `certificate_type text not null check certificate_type in ('certificate_of_origin','preference_statement','export_compliance','broker_packet_cover','customs_supporting')`.
- Column: `source_system_ref text not null`.
- Column: `source_document_ref text not null`.
- Column: `requested_by_principal text not null`.
- Column: `request_state text not null check request_state in ('draft','previewed','generated','signed','transmitted','voided','failed')`.
- Column: `template_version_ref text not null`.
- Column: `idempotency_key text not null`.
- Column: `policy_bundle_version text not null`.
- Column: `ontology_version text not null`.
- Column: `audit_chain_ref text not null`.
- Unique: `gt_certificate_request_idempotency_uq` on `(tenant_id, idempotency_key)`.
- Table: `global_trade_certificate_field_snapshot`.
- Column: `tenant_id uuid not null`.
- Column: `field_snapshot_id uuid primary key`.
- Column: `certificate_request_id uuid not null references global_trade_certificate_request(certificate_request_id)`.
- Column: `field_name text not null`.
- Column: `field_value text not null`.
- Column: `source_ref text not null`.
- Column: `source_event_ref text not null`.
- Column: `evidence_hash text not null`.
- Column: `redaction_class text not null check redaction_class in ('public','counterparty','broker','auditor')`.
- Index: `gt_certificate_field_request_idx` on `(tenant_id, certificate_request_id)`.
- Table: `global_trade_certificate_artifact`.
- Column: `tenant_id uuid not null`.
- Column: `artifact_id uuid primary key`.
- Column: `certificate_request_id uuid not null references global_trade_certificate_request(certificate_request_id)`.
- Column: `artifact_kind text not null check artifact_kind in ('preview_pdf','signed_pdf','xml_message','json_packet')`.
- Column: `object_store_ref text not null`.
- Column: `content_sha256 text not null`.
- Column: `signature_ref text null`.
- Column: `transmission_ref text null`.
- Column: `created_at timestamptz not null`.
- Table: `global_trade_certificate_template_version`.
- Column: `tenant_id uuid not null`.
- Column: `template_version_ref text primary key`.
- Column: `certificate_type text not null`.
- Column: `jurisdiction text not null`.
- Column: `template_hash text not null`.
- Column: `approved_by_principal text not null`.
- Column: `active_from date not null`.
- Column: `active_until date null`.
- Retention: generated artifacts are immutable; voiding creates a void marker and successor link.
- Retention: field snapshots keep source refs for audit replay and do not refresh silently.

## API Endpoints
- REST: `POST /v1/global-trade/certificates:prepare`.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "usr_priya_export",
  "idempotency_key": "cert-2026-05-20-1001",
  "certificate_type": "certificate_of_origin",
  "source_system_ref": "shipment:na",
  "source_document_ref": "shipment:880045",
  "requested_output": ["preview_pdf", "signed_pdf"],
  "line_refs": ["line-1", "line-2"]
}
```
- REST response example:
```json
{
  "certificate_request_id": "cert_01jytg_doc_018",
  "request_state": "previewed",
  "template_version_ref": "coo-us-2026-v3",
  "missing_evidence": [],
  "preview_artifact": {
    "artifact_kind": "preview_pdf",
    "content_sha256": "sha256:preview018"
  },
  "audit_event_class": "EVT-GLOBAL_TRADE-CERTIFICATE-PREVIEWED"
}
```
- REST: `POST /v1/global-trade/certificates/{certificate_request_id}:generate`.
- REST: `POST /v1/global-trade/certificates/{certificate_request_id}:sign`.
- REST: `POST /v1/global-trade/certificates/{certificate_request_id}:transmit`.
- REST: `POST /v1/global-trade/certificates/{certificate_request_id}:void`.
- REST: `GET /v1/global-trade/certificates/{certificate_request_id}`.
- gRPC: `PrepareCertificate(PrepareCertificateRequest) returns (PrepareCertificateResult)`.
- gRPC: `GenerateCertificate(GenerateCertificateRequest) returns (GenerateCertificateResult)`.
- gRPC: `TransmitCertificate(TransmitCertificateRequest) returns (TransmitCertificateResult)`.
- Worker command: `global-trade.certificate.render-artifact`.
- Worker command: `global-trade.certificate.transmit-via-em`.
- Error envelope: `POLICY_DENIED`, `TEMPLATE_INACTIVE`, `SOURCE_EVIDENCE_MISSING`, `PREVIEW_ONLY_ARTIFACT`, `SIGNATURE_FAILED`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"usr_priya_export"`.
- Action: `GlobalTrade::Action::"PrepareCertificate"`.
- Resource: `GlobalTrade::CertificateRequest::"cert_01jytg_doc_018"`.
- Context field: `tenant_id`.
- Context field: `certificate_type`.
- Context field: `source_document_ref`.
- Context field: `template_version_ref`.
- Context field: `requested_output`.
- Context field: `data_residency_region`.
- Generate action: `GlobalTrade::Action::"GenerateCertificate"`.
- Sign action: `GlobalTrade::Action::"SignCertificate"`.
- Transmit action: `GlobalTrade::Action::"TransmitCertificate"`.
- Void action: `GlobalTrade::Action::"VoidCertificate"`.
- Allow rule intent: export coordinators can prepare and preview certificates for assigned shipments.
- Allow rule intent: compliance signers can sign certificates when evidence is complete.
- Deny rule intent: no user can transmit preview artifacts.
- Deny rule intent: no user can edit source-derived certificate fields during generation.
- Deny rule intent: expired templates are unusable even for administrators.
- Audit on allow: include template version, source document, output kind, and evidence bundle hash.
- Audit on deny: include blocked action and certificate request id without exposing private field values.

## Ontology Projection Field Mapping
- Ontology node: `TradeDocument`.
- `certificate_request_id` maps to `TradeDocument.id`.
- `certificate_type` maps to `TradeDocument.documentType`.
- `source_document_ref` maps to `TradeDocument.sourceDocumentRef`.
- `request_state` maps to `TradeDocument.lifecycleState`.
- `template_version_ref` maps to `TradeDocument.templateVersion`.
- Ontology node: `CertificateArtifact`.
- `artifact_id` maps to `CertificateArtifact.id`.
- `artifact_kind` maps to `CertificateArtifact.kind`.
- `object_store_ref` maps to `CertificateArtifact.storageRef`.
- `content_sha256` maps to `CertificateArtifact.contentHash`.
- `signature_ref` maps to `CertificateArtifact.signatureRef`.
- Ontology node: `EvidenceBundle`.
- `field_snapshot_id` maps to `EvidenceBundle.fieldEvidenceId`.
- `field_name` maps to `EvidenceBundle.fieldName`.
- `source_ref` maps to `EvidenceBundle.sourceRef`.
- `source_event_ref` maps to `EvidenceBundle.sourceEventRef`.
- `evidence_hash` maps to `EvidenceBundle.evidenceHash`.
- Projection mode: write TradeDocument after prepare, CertificateArtifact after render, and EvidenceBundle for every printed field.
- Projection guard: object store refs are projected, not raw binary contents.

## Workflow Steps
- Node `ReceiveCertificateRequest`: validate tenant, source document, type, idempotency, and requested output.
- Node `ResolveTemplateVersion`: select active template by tenant, certificate type, jurisdiction, and language.
- Branch `TemplateInactive`: fail closed and emit template blocked event.
- Node `CollectSourceEvidence`: read approved HS classification, origin, preference, screening, export control, and shipment refs.
- Branch `MissingEvidence`: return missing evidence list and create workflow task.
- Branch `EvidenceComplete`: assemble immutable field snapshots.
- Node `RunCedarAuthorization`: check prepare, generate, sign, transmit, or void action.
- Node `RenderPreview`: produce watermarked preview artifact.
- Node `GenerateFinalArtifact`: render final PDF, XML, or JSON packet.
- Node `SignArtifact`: apply tenant certificate or signing service reference.
- Node `SealAuditEvent`: append certificate event to ADR-0263 audit chain.
- Node `ProjectOntology`: project TradeDocument, CertificateArtifact, and EvidenceBundle.
- Node `TransmitViaElectronicMessaging`: hand off signed artifact to broker or external party.
- Branch `SignatureFailure`: keep generated state and assign signer retry workflow.
- Branch `TransmitFailure`: keep signed state and retry SAP GTS-EM equivalent worker.
- Branch `VoidRequested`: mark voided and notify consumers with successor requirement.

## Audit Events
- `EVT-GLOBAL_TRADE-CERTIFICATE-PREPARE_REQUESTED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-TEMPLATE_SELECTED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-EVIDENCE_SNAPSHOTTED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-PREVIEWED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-GENERATED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-SIGNED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-TRANSMITTED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-VOIDED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-MISSING_EVIDENCE_HELD`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-CERTIFICATE-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `certificate_request_id`, `template_version_ref`, `artifact_hash`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event rule: preview and final artifacts use distinct event classes.
- Event rule: voided certificates retain artifact hash and void reason.

## SLO Targets
- Availability target: 99.95 percent monthly for prepare and generate endpoints.
- Throughput target: 120 certificate requests per second per region.
- p50 latency target: 150 ms for prepare with cached evidence.
- p95 latency target: 800 ms for preview render.
- p99 latency target: 2500 ms for final PDF render and signing.
- Transmit freshness target: 99 percent of signed artifacts handed to electronic messaging within 2 minutes.
- Audit seal target: 99.99 percent first-attempt seal rate.
- Artifact integrity target: 100 percent generated artifacts have content hash before visibility.
- Rationale: document preparation is interactive, while final render can run asynchronously when large attachments are present.
- Burn alert: page when final render p99 exceeds 5 seconds for 20 minutes or signature failure rate exceeds 1 percent.

## Failure Modes And Recovery
- Failure: active template missing; recovery: fail closed and assign template activation task.
- Failure: source evidence is incomplete; recovery: return missing evidence and create workflow task.
- Failure: user attempts manual field edit; recovery: reject request and emit policy deny event.
- Failure: object storage write succeeds but audit seal fails; recovery: mark artifact quarantined and retry seal before visibility.
- Failure: signing provider unavailable; recovery: keep generated artifact and retry signing worker.
- Failure: broker transmission fails; recovery: keep signed state and retry via SAP GTS-EM equivalent queue.
- Failure: template hash mismatch; recovery: block render and require template re-approval.
- Failure: ontology projection fails; recovery: retry projection and block external transmission until projected.
- Failure: duplicate idempotency key; recovery: return existing certificate request.
- Failure: void request races with transmit; recovery: lock request and require successor certificate if transmit already occurred.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS document output history and certificate of origin records.
- SAP source: SAP GTS preference processing certificate support records.
- SAP source: SAP GTS electronic messaging output logs.
- Oracle source: GTM trade document and document generation history.
- Descartes source: certificate generation and broker document packet exports.
- Amber Road source: export document generation history.
- Tenant source: legacy certificate PDF archive with shipment references.
- Migration step: import artifact metadata before binary refs.
- Migration step: compute SHA-256 for every migrated certificate artifact.
- Migration step: mark migrated PDFs as `legacy_signed` only when signature provenance is available.
- Migration step: map legacy template names to template version refs.
- Migration step: create field snapshots for migrated records when source fields are recoverable.
- Migration step: void duplicate historical certificates instead of deleting them.

## Cross-Microservice Handoffs
- From HS classification: approved HS code and preference attachment evidence.
- From denied-party screening: clear status or hit disposition reference.
- From export-control classification: license and control classification evidence.
- From customs-declaration: declaration refs and line refs requiring documents.
- To broker-filing: signed certificate artifact refs for packet submission.
- To workflow-engine: missing evidence, signer approval, void approval, and retry tasks.
- To ontology: TradeDocument, CertificateArtifact, and EvidenceBundle projections.
- To audit-chain: ADR-0263 event append and chain verification.
- To object storage: immutable artifact write and content hash verification.
- To notification: signer task, transmission success, void notice, and missing evidence notice.

## Implementation Checklist
- Add aggregate `CertificateRequest`.
- Add entity `CertificateFieldSnapshot`.
- Add entity `CertificateArtifact`.
- Add entity `CertificateTemplateVersion`.
- Add value object `CertificateType`.
- Add value object `ArtifactHash`.
- Add repository for certificate requests.
- Add repository for certificate artifacts.
- Add repository for template versions.
- Add field assembler service.
- Add template selector service.
- Add renderer port.
- Add signing port.
- Add electronic messaging transmit port.
- Add command handler for prepare.
- Add command handler for generate.
- Add command handler for sign.
- Add command handler for transmit.
- Add command handler for void.
- Add Cedar checks for preview, generate, sign, transmit, and void.
- Add REST route for prepare.
- Add REST route for generate.
- Add REST route for sign.
- Add REST route for transmit.
- Add REST route for void.
- Add gRPC methods for internal certificate operations.
- Add worker for render artifact.
- Add worker for transmit retry.
- Add ontology projection writer.
- Add audit appender with ADR-0263 event classes.
- Add fixture for complete certificate of origin.
- Add fixture for missing preference evidence.
- Add fixture for preview-only artifact transmit denial.
- Add fixture for signature failure.
- Add unit tests for field snapshot source mapping.
- Add unit tests for template version selection.
- Add policy tests for coordinator, signer, auditor, and broker.
- Add contract tests for prepare and generate endpoints.
- Add replay tests for idempotency key reuse.
- Add migration tests for SAP GTS certificate history.
- Add performance test for 10,000 certificate previews per hour.
- Add dashboard panels for render latency, signature failures, transmit backlog, and audit seal failures.
- Add acceptance evidence referencing this IP id.
