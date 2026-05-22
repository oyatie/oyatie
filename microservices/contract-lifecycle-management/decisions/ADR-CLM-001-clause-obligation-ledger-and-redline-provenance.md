---
id: ADR-CLM-001
title: Clause obligation ledger and redline provenance
status: Proposed
date: 2026-05-20
microservice: contract-lifecycle-management
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0316-capability-tier-activation-over-product-fragmentation
  - ADR-0263-observability-emission-contract
decision_owner: axis-contract-lifecycle-management
---

# ADR-CLM-001: Clause obligation ledger and redline provenance

## Context

- Architectural pressure name: legal evidence provenance pressure.
- Contract Lifecycle Management owns contract state, clause controls, obligations, approvals, renewal risk, and legal evidence.
- The PRD explicitly says this concern cannot be pushed into generic drive.
- The service benchmark set includes Ironclad, Conga CLM, LinkSquares, Agiloft, and Icertis.
- The PRD defines bounded contexts for contract-intake, clause-library, negotiation, obligation, and renewal.
- Existing policy files include contract-obligation authorization, local clause policy evaluation, redline thread access, signature approval, and obligation egress.
- Existing SLOs include contract cycle time, redline turnaround latency, obligation extraction completeness, clause policy evaluation latency, renewal risk freshness, and signature provider success.
- Existing runbooks cover legal hold, clause policy misfire, redline turnaround lag, renewal risk stale, and signature provider outage.
- Constraint CLM-C1: legal document content is tenant-scoped and may carry GDPR, SOX-404, eIDAS, ESIGN, KR-PIPA, and SOC-2 pack requirements.
- Constraint CLM-C2: contract versions cannot be destructively corrected after counterparty exchange.
- Constraint CLM-C3: obligations extracted from text must retain source span and confidence.
- Constraint CLM-C4: redline comments and clause deviations must preserve actor, source, and timestamp evidence.
- Constraint CLM-C5: signature provider portability must not change the legal packet identity.
- Constraint CLM-C6: renewal risk scores must be explainable and traceable to obligation and date facts.
- Constraint CLM-C7: workflow-engine owns approval routing, but CLM owns legal-domain invariants.
- Constraint CLM-C8: marketplace DealSet settlement cannot be bypassed for commercial obligations.
- Constraint CLM-C9: Cedar must authorize every mutation before storage or provider access.
- Constraint CLM-C10: exported contract packets must include audit references and redaction evidence.
- The architecture names aggregate roots such as `contract_intake_document`, `clause_library_document`, `negotiation_document`, `obligation_document`, and `renewal_document`.
- The service must avoid treating vendor document ids as canonical object names.
- The service must support migration dry-runs from incumbent CLM systems without trust in vendor state.
- The service must support human review when extraction confidence is low.
- The service must give auditors a chain from clause text to obligation task and renewal notice.
- The service must keep redline activity separate from generic collaboration comments.
- The service must make legal hold and export redaction mechanically testable.
- The service must not place raw provider credentials in local tables.
- The service must preserve evidence even when a signature provider is unavailable.

## Decision

- Decision name: ContractObligationLedger v1.
- Adopt an append-only clause, redline, and obligation ledger as the canonical CLM evidence model.
- Treat `ContractPacket` as the stable legal packet identity across drafts, redlines, signature envelopes, and renewal events.
- Treat `ClauseVersion` as an immutable normalized clause body plus source span map.
- Treat `RedlineEvent` as the only authority for counterparty edits, comments, acceptances, and rejections.
- Treat `ObligationFact` as an extracted or human-authored obligation with source span, due basis, owner role, and confidence.
- Treat `RenewalRiskFact` as a projection over obligation facts, notice dates, counterparty history, and clause deviations.
- Store legal content in tenant-scoped object storage under data-class and retention policy.
- Store normalized metadata and ledger rows in Postgres.
- Store only object references, content digests, and redaction manifests in relational rows.
- Require every ledger append to include tenant, principal, contract packet id, policy decision id, trace id, and audit event id.
- Require all ledger rows to be immutable after append.
- Allow correction only through compensating ledger events.
- Require `ClauseDeviation` to classify deviations as fallback, non-standard, high-risk, prohibited, or approved-exception.
- Require prohibited deviations to pause the workflow until legal approver action.
- Require obligation extraction confidence at least 0.85 for auto-proposed obligations.
- Require confidence below 0.95 to create a human review task before external notification.
- Require confidence below 0.70 to mark extraction as advisory only.
- Require clause policy evaluation p95 below 200 ms and p99 below 500 ms.
- Require redline event append p95 below 300 ms.
- Require obligation extraction completeness target at least 0.98 against the canonical fixture set.
- Require renewal risk freshness p95 below 15 minutes after relevant ledger append.
- Require signature provider portability by storing provider envelope id as a child reference, not packet identity.
- Require signature packet export to include ledger root, provider evidence, identity evidence, and redaction manifest.
- Require legal hold to freeze destructive provider actions while permitting scoped read and export.
- Require eIDAS and ESIGN packs to retain consent, signature intent, authentication, and completion certificate references.
- Require SOX-404 pack to bind obligation changes to approval route and segregation-of-duties evidence.
- Publish `clm.contract.packet.created.v1`, `clm.clause.versioned.v1`, `clm.redline.appended.v1`, `clm.obligation.proposed.v1`, `clm.obligation.approved.v1`, `clm.renewal.risk.changed.v1`, and `clm.signature.packet.sealed.v1`.
- Use workflow-engine for approvals and review tasks.
- Use ontology projection for ContractPacket, Clause, Obligation, Counterparty, RenewalNotice, and SignaturePacket.
- Make this ADR authoritative for clause provenance, redline history, obligation extraction, renewal risk, and signature packet identity.

## Alternatives Considered

### Alternative 1: Store contract drafts as mutable documents only

- Pros: simple document editing model.
- Pros: close to generic drive behavior.
- Pros: fewer domain tables.
- Cons: cannot trace obligations to source clause versions.
- Cons: redline history can be overwritten.
- Cons: legal hold and audit export become fragile.
- Rejected because CLM needs legal provenance beyond drive.

### Alternative 2: Make the signature provider envelope the canonical contract id

- Pros: aligns with DocuSign-style provider workflows.
- Pros: simplifies signed packet export for one provider.
- Pros: reduces local identity mapping.
- Cons: breaks provider portability.
- Cons: cannot represent pre-signature negotiation faithfully.
- Cons: provider outage or migration changes packet identity.
- Rejected because the legal packet must remain Oyatie-owned.

### Alternative 3: Auto-create all extracted obligations without review

- Pros: fastest workflow.
- Pros: fewer human tasks.
- Pros: attractive for high-volume imports.
- Cons: false obligations can trigger incorrect notices.
- Cons: low-confidence extraction would become authoritative.
- Cons: regulated packs require accountable review.
- Rejected because confidence thresholds and review are required.

### Alternative 4: Generic workflow comments for redlines

- Pros: reuses collaboration infrastructure.
- Pros: easier UI integration.
- Pros: fewer CLM-specific event types.
- Cons: loses source span and clause deviation semantics.
- Cons: cannot prove counterparty edit provenance.
- Cons: comments are not enough for legal evidence.
- Rejected because redlines are legal state, not generic discussion.

### Alternative 5: One table per vendor import source

- Pros: migration adapters can preserve source shapes.
- Pros: faster first importer for one vendor.
- Pros: easier one-time reconciliation.
- Cons: creates vendor-shaped domain model.
- Cons: makes cross-vendor audit export inconsistent.
- Cons: violates service-owned canonical object names.
- Rejected because imports must normalize into Oyatie ledger state.

## Consequences

### Positive

- Every obligation has a traceable source clause or human-authored basis.
- Redlines become legal evidence instead of comments.
- Signature providers can be changed without changing packet identity.
- Low-confidence extraction creates explicit review work.
- Renewal risk is explainable from ledger facts.
- Legal hold can freeze provider actions without freezing audit reads.
- Pack-specific export requirements can be built from common evidence.
- Vendor migrations can be dry-run against a canonical ledger.
- Clause policy misfires are easier to debug.
- Contract export redaction becomes mechanically verifiable.

### Negative

- The ledger model has more rows than mutable-document storage.
- Extraction fixtures and confidence calibration require ongoing maintenance.
- User flows need clear review states for low-confidence obligations.
- Provider adapters must map evidence into a provider-neutral packet shape.
- Append-only correction requires training for support operators.
- Large contracts need content-addressed storage and streaming export.
- Redline UX needs precise source-span handling.

### Neutral

- Drive may still store generic file attachments.
- Workflow-engine still owns approval orchestration.
- Marketplace still owns commercial settlement.
- Identity still owns principal authentication.
- External providers remain adapters rather than legal authority.

### Follow-up work

- Add `ContractPacket` and `ObligationFact` schemas.
- Add a canonical fixture corpus for obligation extraction.
- Add redline source-span roundtrip tests.
- Add signature provider evidence adapters for the first two providers.
- Add legal hold export runbook.
- Add renewal risk explanation dashboard.
- Add marketplace DealSet obligation handoff tests.

## Implementation Notes

### Data Shapes

- `ContractPacket`: `contract_packet_id`, `tenant_id_hash`, `counterparty_id`, `current_clause_set_id`, `status`, `legal_hold_state`, `data_class`, `retention_pack`, `ledger_root`.
- `ClauseVersion`: `clause_version_id`, `contract_packet_id`, `clause_key`, `version`, `normalized_text_ref`, `source_span_map`, `digest`, `created_by`, `evidence_id`.
- `RedlineEvent`: `redline_event_id`, `contract_packet_id`, `clause_key`, `event_type`, `actor_type`, `source_span`, `payload_ref`, `previous_digest`, `next_digest`, `audit_event_id`.
- `ObligationFact`: `obligation_id`, `contract_packet_id`, `clause_version_id`, `source_span`, `owner_role`, `due_basis`, `confidence`, `review_state`, `status`.
- `ClauseDeviation`: `deviation_id`, `clause_version_id`, `policy_rule_id`, `severity`, `decision`, `approver`, `expires_at`, `evidence_id`.
- `RenewalRiskFact`: `risk_id`, `contract_packet_id`, `notice_date`, `risk_score`, `explanation_refs`, `freshness_deadline`, `evidence_id`.
- `SignaturePacket`: `signature_packet_id`, `contract_packet_id`, `provider`, `provider_envelope_id`, `signer_refs`, `completion_certificate_ref`, `seal_digest`.
- `RedactionManifest`: `manifest_id`, `contract_packet_id`, `export_id`, `redacted_fields`, `policy_decision_id`, `created_at`, `evidence_id`.

### API Endpoints

- `POST /v1/clm/contracts` creates a contract packet.
- `GET /v1/clm/contracts/{contract_packet_id}` returns packet metadata and current ledger root.
- `POST /v1/clm/contracts/{contract_packet_id}/clauses` appends a clause version.
- `POST /v1/clm/contracts/{contract_packet_id}/redlines` appends a redline event.
- `POST /v1/clm/contracts/{contract_packet_id}/obligations/extract` starts extraction.
- `POST /v1/clm/obligations/{obligation_id}/review` records review decision.
- `POST /v1/clm/contracts/{contract_packet_id}/renewal-risk/recalculate` recalculates risk.
- `POST /v1/clm/contracts/{contract_packet_id}/signature-packets` creates provider-neutral signature packet.
- `POST /v1/clm/contracts/{contract_packet_id}/legal-hold` activates or releases legal hold.
- `POST /v1/clm/contracts/{contract_packet_id}/exports` creates export with redaction manifest.

### Cedar Policies

- `clm::contract::create` requires tenant admin, legal owner, or approved intake workflow principal.
- `clm::clause::append` requires packet write permission and no blocking legal hold.
- `clm::redline::append` requires counterparty scope or internal negotiator scope.
- `clm::obligation::extract` requires legal owner or approved worker principal.
- `clm::obligation::approve` requires legal approver distinct from extraction worker when confidence is below 0.95.
- `clm::deviation::approve_exception` requires legal approver and pack-specific segregation rules.
- `clm::signature::seal` requires completed signer evidence and provider-neutral packet digest.
- `clm::export::read` requires auditor scope and redaction manifest.
- `clm::legal_hold::mutate` requires legal hold officer and audit-chain availability.

### SLO Targets

- `clm_clause_policy_eval_p95_ms` target is 200.
- `clm_clause_policy_eval_p99_ms` target is 500.
- `clm_redline_append_p95_ms` target is 300.
- `clm_obligation_extract_completeness` target is 0.98.
- `clm_renewal_risk_freshness_p95_minutes` target is 15.
- `clm_signature_provider_success` target is 0.995.
- `clm_contract_cycle_time_budget_compliance` target is 0.95.
- `clm_audit_emission_lag_p95_seconds` target is 1.

## Verification

- Unit test `contract_packet_identity_survives_signature_provider_change`.
- Unit test `clause_version_is_immutable_after_append`.
- Unit test `redline_event_requires_source_span`.
- Unit test `obligation_fact_requires_clause_or_human_basis`.
- Unit test `renewal_risk_fact_references_obligation_facts`.
- Unit test `legal_hold_blocks_destructive_provider_action`.
- Property test `ledger_root_changes_on_every_append`.
- Property test `redline_roundtrip_preserves_source_span`.
- Property test `obligation_due_basis_parser_rejects_ambiguous_dates`.
- Fuzz test `vendor_import_dry_run_never_creates_authoritative_rows`.
- Cedar test `cross_tenant_redline_access_denied`.
- Cedar test `low_confidence_obligation_requires_review`.
- Cedar test `signature_seal_requires_completed_signer_evidence`.
- Cedar test `export_requires_redaction_manifest`.
- Cedar test `legal_hold_mutation_requires_legal_hold_officer`.
- Contract test `clm_openapi_contract_packet_paths_match_router`.
- Contract test `clm_asyncapi_events_include_ledger_root`.
- Contract test `clm_proto_signature_packet_matches_rest_shape`.
- Integration test `intake_to_clause_to_obligation_to_renewal_risk`.
- Integration test `prohibited_clause_deviation_pauses_workflow`.
- Integration test `signature_provider_outage_preserves_packet_identity`.
- Integration test `legal_hold_export_redacts_required_fields`.
- Integration test `marketplace_dealset_obligation_handoff_preserves_audit_ref`.
- Replay test `ledger_events_rebuild_contract_projection`.
- Load test `ten_thousand_redline_events_append_under_p95_budget`.
- Load test `obligation_extraction_batch_emits_progress_and_review_tasks`.
- Chaos test `audit_chain_unavailable_pauses_high_risk_mutation`.
- Chaos test `openbao_unavailable_blocks_provider_signature`.
- Metric `oya_clm_ledger_append_total`.
- Metric `oya_clm_obligation_extract_confidence_bucket`.
- Metric `oya_clm_redline_append_duration_ms`.
- Metric `oya_clm_clause_deviation_total`.
- Metric `oya_clm_signature_provider_success_ratio`.
- Dashboard `clm-obligation-extraction`.
- Dashboard `clm-redline-turnaround`.
- Dashboard `clm-renewal-risk-freshness`.
- Dashboard `clm-signature-provider-health`.
- Alert `ClmLowConfidenceBacklogBurn`.
- Alert `ClmClausePolicyLatencyBurn`.
- Alert `ClmSignatureProviderOutage`.

## References

- Internal: microservices/contract-lifecycle-management/PRD.md.
- Internal: microservices/contract-lifecycle-management/ARCHITECTURE.md.
- Internal: microservices/contract-lifecycle-management/policy/contract-obligation-authorization.cedar.
- Internal: microservices/contract-lifecycle-management/policies/local-clause-policy-evaluation.cedar.
- Internal: microservices/contract-lifecycle-management/policies/local-redline-thread-access.cedar.
- Internal: microservices/contract-lifecycle-management/slos/local-obligation-extract-completeness.openslo.yaml.
- Internal: microservices/contract-lifecycle-management/slos/local-redline-turnaround-latency.openslo.yaml.
- Internal: microservices/contract-lifecycle-management/slos/local-renewal-risk-freshness.openslo.yaml.
- Internal: microservices/contract-lifecycle-management/runbooks/legal-hold-activation.md.
- Internal: microservices/contract-lifecycle-management/runbooks/contract-export-redaction.md.
- DocuSign eSignature API documentation.
- DocuSign CLM documentation.
- Ironclad API documentation.
- Icertis Contract Intelligence documentation.
- ISO 14533 long-term signature profiles.
- ETSI EN 319 102-1 electronic signatures and infrastructures.
- NIST SP 800-63 Digital Identity Guidelines.
- W3C Verifiable Credentials Data Model.
- OpenAPI Specification.
- AsyncAPI Specification.
- CloudEvents Specification.
- W3C Trace Context.
- RFC 3161: Time-Stamp Protocol.
- RFC 9110: HTTP Semantics.
