---
doc_class: ImplementationPlan
ip_id: IP-016
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
  - SAP GTS-CC commodity-code-numbering
  - SAP GTS-PI preference-processing
  - SAP GTS-EM classification-message-distribution
---

# IP-016: HS code classification engine with FTA preference attach

## Context With Why
- This IP builds the classification decision path that assigns HS codes to tenant goods and attaches eligible free trade agreement preference evidence.
- The why is operational: trade teams lose margin and create customs risk when HS classification and FTA preference evidence live in separate spreadsheets.
- The service must make classification explainable enough for a junior engineer to implement and an auditor to replay.
- The journey leg starts when a product master, bill of materials, or commercial invoice line arrives with incomplete classification.
- The journey leg ends when the line has a versioned HS code, confidence score, evidence bundle, and optional preference attachment.
- Named persona: Maya, a trade compliance analyst at a mid-market manufacturer, needs to classify 500 new SKUs before a Mexico shipment cutoff.
- Maya can accept a low-confidence queue, but she cannot accept hidden preference attachment without rule evidence.
- This IP maps SAP GTS-CC classification behavior and SAP GTS-PI preference attachment into a tenant-scoped Oyatie capability.
- SAP GTS-EM is relevant because approved classifications must be distributable to brokers, ERPs, and customs message surfaces.
- The implementation must not own master product data; it stores classification decisions and references source product identities.
- The implementation must not create customs declarations; it hands classified lines to customs-declaration and broker-filing.
- ADR-0105 keeps the layer vocabulary strict: domain rules do not call adapters, and adapters do not own classification truth.
- ADR-0244 requires every row to carry tenant scope and sub-scope lineage.
- ADR-0243 requires the classification decision to preserve source-system provenance.
- ADR-0253 requires Cedar default-deny before a classification can be created, approved, overridden, or exported.
- ADR-0263 requires audit events to be classed, chainable, and replayable.
- ADR-0304 requires ontology projection for ProductClassification and PreferenceEvidence.
- ADR-0315 sets the SAP parity bar; this is not a generic tagging feature.
- Intern build target: one domain aggregate, one command handler, one repository, one policy check, one projection writer, and one worker.

## Scope Boundaries
- In scope: HS code suggestion, manual override, confidence scoring, FTA preference attach, evidence storage, export to broker message surfaces.
- In scope: tenant product reference, country-of-origin input, tariff schedule version, rule basis, and attached preference program.
- In scope: hold state when confidence, source data, Cedar, ontology, or audit sealing fails.
- Out of scope: product master ownership, supplier onboarding, customs declaration filing, duty calculation, and final customs payment.
- Out of scope: legal advice generation; the engine records rule evidence and workflow approvals.
- Out of scope: automatic preference claim filing; that is handled by customs-declaration and broker-filing.
- Boundary rule: a classification decision may reference a product, but it must not mutate the product master.
- Boundary rule: preference attachment may reference origin evidence, but it must not decide supplier contract compliance.
- Boundary rule: every external schedule import is staged before promotion into active classification rules.
- Boundary rule: every output must be explainable by persisted rule rows, not model-only inference.

## Data Model Deltas
- Table: `global_trade_hs_classification_decision`.
- Column: `tenant_id uuid not null`.
- Column: `classification_id uuid primary key`.
- Column: `product_ref text not null`.
- Column: `source_system_ref text not null`.
- Column: `source_line_ref text null`.
- Column: `hs_code text not null`.
- Column: `hs_code_version text not null`.
- Column: `classification_basis text not null`.
- Column: `confidence_score numeric(5,4) not null`.
- Column: `decision_state text not null check decision_state in ('draft','needs_review','approved','rejected','superseded')`.
- Column: `approved_by_principal text null`.
- Column: `approved_at timestamptz null`.
- Column: `idempotency_key text not null`.
- Column: `policy_bundle_version text not null`.
- Column: `ontology_version text not null`.
- Column: `audit_chain_ref text not null`.
- Index: `gt_hs_classification_tenant_product_idx` on `(tenant_id, product_ref, hs_code_version)`.
- Unique: `gt_hs_classification_idempotency_uq` on `(tenant_id, idempotency_key)`.
- Table: `global_trade_fta_preference_attachment`.
- Column: `tenant_id uuid not null`.
- Column: `preference_attachment_id uuid primary key`.
- Column: `classification_id uuid not null references global_trade_hs_classification_decision(classification_id)`.
- Column: `agreement_code text not null`.
- Column: `origin_country text not null`.
- Column: `destination_country text not null`.
- Column: `rule_of_origin_code text not null`.
- Column: `preference_claim_state text not null check preference_claim_state in ('candidate','attached','held','withdrawn')`.
- Column: `evidence_bundle_ref text not null`.
- Column: `valid_from date not null`.
- Column: `valid_until date null`.
- Column: `sap_gts_pi_equivalent text not null default 'GTS-PI preference determination'`.
- Index: `gt_fta_preference_agreement_idx` on `(tenant_id, agreement_code, origin_country, destination_country)`.
- Table: `global_trade_classification_rule_basis`.
- Column: `tenant_id uuid not null`.
- Column: `rule_basis_id uuid primary key`.
- Column: `classification_id uuid not null`.
- Column: `basis_type text not null check basis_type in ('tariff_note','binding_ruling','attribute_match','manual_rationale','legacy_import')`.
- Column: `basis_ref text not null`.
- Column: `basis_summary text not null`.
- Column: `evidence_hash text not null`.
- Retention: classification decision rows are immutable after approval; corrections create superseding rows.
- Retention: preference attachments can be withdrawn, but withdrawal emits a new audit event rather than deleting evidence.

## API Endpoints
- REST: `POST /v1/global-trade/hs-classifications:classify`.
- REST purpose: accept product attributes and return a draft or approved classification decision.
- REST request example:
```json
{
  "tenant_id": "ten_usa_001",
  "principal_id": "usr_maya_trade",
  "idempotency_key": "cls-2026-05-20-0001",
  "product_ref": "sku-PUMP-88",
  "source_system_ref": "sap-s4:material:100455",
  "destination_country": "US",
  "attributes": {
    "description": "stainless centrifugal pump",
    "material": "stainless steel",
    "use": "industrial fluid movement",
    "unit_value": "820.00"
  },
  "requested_agreements": ["USMCA"]
}
```
- REST response example:
```json
{
  "classification_id": "cls_01jytg_hs_016",
  "decision_state": "needs_review",
  "hs_code": "8413.70.2004",
  "hs_code_version": "HTSUS-2026-r2",
  "confidence_score": 0.8725,
  "preference_candidates": [
    {
      "agreement_code": "USMCA",
      "rule_of_origin_code": "CTH-8413",
      "preference_claim_state": "candidate"
    }
  ],
  "audit_event_class": "EVT-GLOBAL_TRADE-HS_CLASSIFICATION-CLASSIFIED"
}
```
- REST: `POST /v1/global-trade/hs-classifications/{classification_id}:approve`.
- REST: `POST /v1/global-trade/hs-classifications/{classification_id}:attach-preference`.
- REST: `GET /v1/global-trade/hs-classifications/{classification_id}`.
- REST: `GET /v1/global-trade/hs-classifications?product_ref={product_ref}`.
- gRPC: `ClassifyHsCode(ClassifyHsCodeRequest) returns (ClassifyHsCodeResult)`.
- gRPC: `ApproveHsClassification(ApproveHsClassificationRequest) returns (ApproveHsClassificationResult)`.
- gRPC: `AttachFtaPreference(AttachFtaPreferenceRequest) returns (AttachFtaPreferenceResult)`.
- Worker command: `global-trade.hs-classification.import-schedule-version`.
- Worker command: `global-trade.hs-classification.replay-preference-candidates`.
- Error envelope: `POLICY_DENIED`, `SCHEDULE_VERSION_INACTIVE`, `LOW_CONFIDENCE_REVIEW_REQUIRED`, `AUDIT_CHAIN_SEAL_FAILED`.

## Cedar Policy Hooks
- Principal: `GlobalTrade::Principal::"usr_maya_trade"`.
- Action: `GlobalTrade::Action::"ClassifyHsCode"`.
- Resource: `GlobalTrade::Classification::"cls_01jytg_hs_016"`.
- Context field: `tenant_id`.
- Context field: `sub_scope_path`.
- Context field: `tenant_class`.
- Context field: `policy_bundle_version`.
- Context field: `requested_agreements`.
- Context field: `source_system_ref`.
- Allow rule intent: classification analysts can create draft classifications for assigned tenant scope.
- Allow rule intent: senior compliance approvers can approve or supersede classifications.
- Deny rule intent: broker principals cannot approve HS decisions; they can only read exported decisions explicitly shared with them.
- Deny rule intent: preference attach requires both `ClassifyHsCode` approval and `AttachFtaPreference` permission.
- Deny rule intent: inactive tariff schedule versions cannot be used even by administrators.
- Audit on allow: emit decision trace with matched policy id and principal attributes.
- Audit on deny: emit policy-deny event without leaking product attributes outside tenant scope.

## Ontology Projection Field Mapping
- Ontology node: `ProductClassification`.
- `tenant_id` maps to `TenantScopedEntity.tenantId`.
- `classification_id` maps to `ProductClassification.id`.
- `product_ref` maps to `ProductClassification.classifiesProduct`.
- `source_system_ref` maps to `SourceLineage.sourceSystemRef`.
- `hs_code` maps to `ProductClassification.hsCode`.
- `hs_code_version` maps to `ProductClassification.scheduleVersion`.
- `classification_basis` maps to `DecisionEvidence.summary`.
- `confidence_score` maps to `DecisionConfidence.score`.
- `decision_state` maps to `WorkflowState.currentState`.
- `audit_chain_ref` maps to `AuditChain.anchorRef`.
- Ontology node: `PreferenceEvidence`.
- `preference_attachment_id` maps to `PreferenceEvidence.id`.
- `agreement_code` maps to `PreferenceEvidence.tradeAgreement`.
- `origin_country` maps to `PreferenceEvidence.originCountry`.
- `destination_country` maps to `PreferenceEvidence.destinationCountry`.
- `rule_of_origin_code` maps to `PreferenceEvidence.ruleOfOrigin`.
- `evidence_bundle_ref` maps to `EvidenceBundle.ref`.
- Projection write mode: outbox event followed by idempotent ontology upsert.
- Projection failure mode: keep classification approved but mark ontology projection pending and block export.

## Workflow Steps
- Node `ReceiveProductLine`: validate tenant, source, idempotency, and minimal product attributes.
- Node `LoadActiveSchedule`: fetch active HS schedule version for destination country.
- Node `GenerateCandidateCodes`: evaluate attribute rules, binding rulings, and tenant history.
- Branch `NoCandidate`: route to manual classification queue.
- Branch `SingleHighConfidenceCandidate`: create draft with auto-ready state.
- Branch `MultipleCandidates`: create needs-review state with ranked basis rows.
- Node `EvaluatePreferencePrograms`: compute FTA candidates using origin and destination.
- Branch `MissingOriginEvidence`: keep preference candidate held.
- Branch `RuleSatisfied`: create candidate preference attachment.
- Node `RunCedarAuthorization`: enforce create, attach, approve, and export actions.
- Node `PersistDecision`: write decision, basis, and attachment rows in one transaction.
- Node `SealAuditEvent`: append ADR-0263 event and capture audit chain reference.
- Node `ProjectOntology`: publish ProductClassification and PreferenceEvidence projection.
- Node `NotifyWorkflowEngine`: create approval task when review is required.
- Node `ExportClassification`: make approved result available to customs-declaration and broker-filing.
- Branch `AuditSealFailure`: rollback transaction and place source line in retry queue.
- Branch `OntologyProjectionFailure`: hold export and retry projection.
- Branch `PolicyDeny`: return denial envelope and emit deny event.

## Audit Events
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-CLASSIFY_REQUESTED`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-CANDIDATES_GENERATED`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-LOW_CONFIDENCE_HELD`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-APPROVED`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-SUPERSEDED`.
- `EVT-GLOBAL_TRADE-FTA_PREFERENCE-CANDIDATE_ATTACHED`.
- `EVT-GLOBAL_TRADE-FTA_PREFERENCE-ATTACHMENT_HELD`.
- `EVT-GLOBAL_TRADE-FTA_PREFERENCE-WITHDRAWN`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-ONTOLOGY_PROJECTED`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-POLICY_DENIED`.
- `EVT-GLOBAL_TRADE-HS_CLASSIFICATION-AUDIT_SEALED`.
- Event fields: `event_id`, `event_class`, `tenant_id`, `principal_id`, `resource_id`, `policy_bundle_version`, `ontology_version`, `occurred_at`, `prev_event_hash`, `event_hash`.
- Event hash input: canonical JSON without transport metadata.
- Event visibility: tenant auditor can read all events for assigned tenant; broker can read exported approval events only.

## SLO Targets
- Availability target: 99.95 percent monthly for classification writes.
- Throughput target: 250 classification requests per second per region.
- p50 latency target: 80 ms for cached schedule classification without preference attach.
- p95 latency target: 450 ms for classification plus preference candidate evaluation.
- p99 latency target: 1200 ms for multi-candidate classification requiring rule-basis expansion.
- Worker freshness target: imported tariff schedule promotion completes within 10 minutes for 100,000 rows.
- Audit seal target: 99.99 percent successful first-attempt seal rate.
- Ontology projection target: 99 percent projected within 60 seconds.
- Rationale: interactive analysts need sub-second feedback, while batch imports can tolerate worker latency if audit sealing is durable.
- Burn alert: page when p99 exceeds 1500 ms for 15 minutes or audit seal failures exceed 0.1 percent.

## Failure Modes And Recovery
- Failure: tariff schedule source import has duplicate HS code rows; recovery: reject source batch and keep prior active schedule.
- Failure: source product lacks material or use; recovery: create `needs_review` with missing-field reasons.
- Failure: Cedar deny on attach preference; recovery: persist no attachment and emit policy deny event.
- Failure: confidence below tenant threshold; recovery: workflow task assigned to compliance analyst queue.
- Failure: binding ruling evidence hash mismatch; recovery: hold approval and require evidence re-ingest.
- Failure: ontology projection unavailable; recovery: retry outbox with exponential backoff and block export.
- Failure: audit chain seal fails; recovery: rollback decision transaction and enqueue source line for retry.
- Failure: downstream broker export times out; recovery: keep approved decision and retry export through SAP GTS-EM equivalent channel.
- Failure: two users approve conflicting HS codes; recovery: optimistic lock rejects stale approval and requires supersession path.
- Failure: inactive FTA agreement is requested; recovery: return inactive agreement error and emit held candidate event.

## Migration Notes With Source Vendor Surfaces
- SAP source: SAP GTS commodity code classification extracts.
- SAP source: SAP GTS preference processing determination tables.
- SAP source: SAP GTS electronic messaging outbound classification payloads.
- Oracle source: GTM item classification and trade agreement qualification exports.
- Descartes source: classification workbench export files.
- Amber Road source: product classification and origin qualification data feeds.
- Spreadsheet source: tenant HTS mapping workbooks with product SKU, country, and evidence columns.
- Migration step: stage all imported classifications as `legacy_import` basis rows.
- Migration step: compute evidence hash for every imported note, ruling, and agreement reference.
- Migration step: do not auto-approve imported rows unless tenant migration policy says legacy approvals are trusted.
- Migration step: create supersession links when an imported row conflicts with an existing active decision.
- Migration step: record source timezone and file checksum in audit context.
- Migration step: run a sample replay for ten high-volume SKUs before bulk promotion.

## Cross-Microservice Handoffs
- To customs-declaration: approved HS code, schedule version, and preference attachment reference.
- To broker-filing: exportable classification packet and SAP GTS-EM style outbound status.
- To trade-document: HS code and origin evidence for certificates and commercial invoice documents.
- To workflow-engine: manual review, approval, supersession, and exception nodes.
- To ontology: ProductClassification and PreferenceEvidence projections.
- To audit-chain: ADR-0263 event append and anchor verification.
- To marketplace: read-only capability entitlement check, no settlement ownership.
- To notification: analyst review assignment and approval completion notices.
- To data-residency: region policy for classification evidence and tariff schedule source files.
- To observability: latency, throughput, audit seal, projection lag, and deny-spike metrics.

## Implementation Checklist
- Add domain aggregate `HsClassificationDecision`.
- Add domain value object `HsCodeVersion`.
- Add domain value object `ClassificationBasis`.
- Add domain value object `PreferenceAttachment`.
- Add repository interface for classification decisions.
- Add repository interface for preference attachments.
- Add transaction boundary around decision, basis, attachment, and outbox writes.
- Add command handler for classify request.
- Add command handler for approve request.
- Add command handler for attach preference request.
- Add Cedar authorization adapter calls before mutation.
- Add schedule lookup port with active-version semantics.
- Add preference rule lookup port with agreement-version semantics.
- Add ontology projection outbox writer.
- Add audit event appender with ADR-0263 event class names.
- Add REST route for classify.
- Add REST route for approve.
- Add REST route for attach preference.
- Add gRPC method for internal batch classification.
- Add worker for schedule import staging.
- Add worker for preference candidate replay.
- Add fixtures for low confidence and high confidence classification.
- Add fixtures for inactive tariff schedule.
- Add fixtures for inactive preference agreement.
- Add fixtures for ontology projection retry.
- Add unit tests for classification candidate ranking.
- Add unit tests for preference eligibility attach.
- Add policy tests for analyst, approver, broker, and auditor principals.
- Add contract tests for REST request and response envelopes.
- Add replay tests for idempotency key reuse.
- Add migration tests for legacy SAP GTS classification imports.
- Add dashboard panels for p50, p95, p99, throughput, and seal failures.
- Add runbook links for schedule import failure and projection lag.
- Add sample curl request in API contract docs.
- Add sample grpcurl request in internal contract docs.
- Add evidence export for auditor replay.
- Add negative test proving brokers cannot approve classifications.
- Add negative test proving inactive schedules cannot be used.
- Add performance test for 100,000-row schedule import.
- Add acceptance evidence referencing this IP id.
