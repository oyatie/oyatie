---
doc_class: LocalizationPack
pack_id: MX-PACK-1
doc_id: MX-PACK-1-CONSENT-DSR
title: Mexico Consent and Data Subject Rights
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LFPDPPP.pdf
  - https://www.diputados.gob.mx/LeyesBiblio/ref/lfpdppp.htm
  - https://inicio.inai.org.mx/Publicaciones/Tripticoejercicioderechos.pdf
  - https://micrositios.inai.org.mx/guiastitulares/INAIvolumen04/3_1.html
  - https://www.cnbv.gob.mx/ProteccionDatos/Paginas/default.aspx
  - https://www.cnsf.gob.mx/Transparencia/Paginas/ProteccionDatosPersonales.aspx
  - https://www.ift.org.mx/proteccion_de_datos_personales/informacion_relevante/variable-y-formato-41-portabilidad-de-datos-personales
  - https://www.gob.mx/atdt
---

# Mexico Consent and Data Subject Rights

This document defines privacy notice, consent, sensitive-data, ARCO, portability, and rights-escalation behavior for MX-PACK-1.
It treats current LFPDPPP articles as authority for private-sector controllers.
It treats INAI rights materials as historical and educational sources that require transition tagging after the 2025 statutory replacement.
It separates private-sector LFPDPPP ARCO duties from public-sector transparency-unit workflows.
It keeps CNBV, CNSF, IFT/CRT, and ATDT records in their sector context.

## Authority Citations

- MX-DSR-AUTH-001: LFPDPPP Art. 3(I) defines the privacy notice used to inform the data subject about processing purposes.


- MX-DSR-AUTH-002: LFPDPPP Art. 3(IV) defines consent as free, specific, and informed will.


- MX-DSR-AUTH-003: LFPDPPP Art. 3(VI) defines sensitive personal data for explicit-consent and heightened-risk handling.


- MX-DSR-AUTH-004: LFPDPPP Art. 3(VII) defines ARCO as access, rectification, cancellation, and opposition.


- MX-DSR-AUTH-005: LFPDPPP Art. 6 requires lawfulness, consent, information, quality, purpose, loyalty, proportionality, and responsibility.


- MX-DSR-AUTH-006: LFPDPPP Art. 7 requires data to be collected and treated lawfully and fairly.


- MX-DSR-AUTH-007: LFPDPPP Art. 8 requires consent except where the law provides otherwise.


- MX-DSR-AUTH-008: LFPDPPP Art. 9 requires express consent for sensitive personal data.


- MX-DSR-AUTH-009: LFPDPPP Art. 10 lists processing cases where consent is not required.


- MX-DSR-AUTH-010: LFPDPPP Art. 11 requires data quality and relevance to purpose.


- MX-DSR-AUTH-011: LFPDPPP Art. 12 limits processing to the purposes in the privacy notice.


- MX-DSR-AUTH-012: LFPDPPP Art. 13 requires treatment to be necessary, adequate, and relevant.


- MX-DSR-AUTH-013: LFPDPPP Art. 14 requires the subject to be informed before treatment.


- MX-DSR-AUTH-014: LFPDPPP Art. 15 requires the controller to make the privacy notice available.


- MX-DSR-AUTH-015: LFPDPPP Art. 16 requires notice content including identity, purposes, options, means, transfers, and rights mechanisms.


- MX-DSR-AUTH-016: LFPDPPP Art. 17 governs notice timing and format by collection channel.


- MX-DSR-AUTH-017: LFPDPPP Art. 18 addresses collection from non-subject sources and notice availability.


- MX-DSR-AUTH-018: LFPDPPP Art. 22 grants the data subject ARCO rights.


- MX-DSR-AUTH-019: LFPDPPP Art. 23 anchors access to data and processing information.


- MX-DSR-AUTH-020: LFPDPPP Art. 24 anchors rectification of inaccurate or incomplete data.


- MX-DSR-AUTH-021: LFPDPPP Art. 25 anchors cancellation and blocking logic.


- MX-DSR-AUTH-022: LFPDPPP Art. 26 anchors opposition to treatment.


- MX-DSR-AUTH-023: LFPDPPP Arts. 27-28 govern ARCO procedure duties and denial paths.


- MX-DSR-AUTH-024: Legacy INAI ARCO materials explain historical rights mechanics but require transition tagging under current authority.


- MX-DSR-AUTH-025: IFT portability materials show a public-sector portability pattern that must not be copied into private-sector workflow without scope review.


## Activated Cedar Policies

- MX-DSR-CEDAR-001: `mx_notice_available_before_collection` denies collection without a privacy notice linked to channel and purpose.


- MX-DSR-CEDAR-002: `mx_notice_content_complete` checks identity, domicile, purposes, options, rights means, transfers, and notice-change procedures.


- MX-DSR-CEDAR-003: `mx_notice_channel_compatible` checks physical, electronic, optical, audio, visual, or other technology presentation.


- MX-DSR-CEDAR-004: `mx_notice_purpose_binding` denies use outside stated purposes.


- MX-DSR-CEDAR-005: `mx_consent_specific_informed` denies consent records without purpose, notice, and subject evidence.


- MX-DSR-CEDAR-006: `mx_consent_revocation_route` requires withdrawal or revocation mechanics when consent is the basis.


- MX-DSR-CEDAR-007: `mx_consent_exception_mapped` requires LFPDPPP Art. 10 exception code when consent is absent.


- MX-DSR-CEDAR-008: `mx_sensitive_explicit_consent` denies sensitive-data use without express consent or valid exception.


- MX-DSR-CEDAR-009: `mx_minors_context_review` requires heightened review for child or adolescent data even when sector law is outside this pack.


- MX-DSR-CEDAR-010: `mx_arco_access_intake` creates access request case and deadline.


- MX-DSR-CEDAR-011: `mx_arco_rectification_intake` creates correction request case and evidence review.


- MX-DSR-CEDAR-012: `mx_arco_cancellation_intake` creates cancellation request case and blocking assessment.


- MX-DSR-CEDAR-013: `mx_arco_opposition_intake` creates opposition request case and purpose review.


- MX-DSR-CEDAR-014: `mx_arco_identity_verification` denies disclosure until requester identity or representation is verified.


- MX-DSR-CEDAR-015: `mx_arco_deadline_clock` computes and stores response deadline for accepted requests.


- MX-DSR-CEDAR-016: `mx_arco_denial_reason_required` requires a coded denial reason when rights requests are refused.


- MX-DSR-CEDAR-017: `mx_arco_completion_evidence` requires delivery or action evidence before closure.


- MX-DSR-CEDAR-018: `mx_portability_scope_review` treats portability as legal-review or sector-public-source pattern unless private-sector authority is confirmed.


- MX-DSR-CEDAR-019: `mx_legacy_inai_rights_transition` tags INAI educational sources as legacy or historical guidance.


- MX-DSR-CEDAR-020: `mx_secretaria_rights_authority` tags current federal authority route after the 2025 LFPDPPP replacement.


- MX-DSR-CEDAR-021: `mx_cnbv_rights_overlay` applies financial-sector privacy handling when rights requests concern financial or fintech records.


- MX-DSR-CEDAR-022: `mx_cnsf_rights_overlay` applies insurance supervisory and policyholder context to rights requests.


- MX-DSR-CEDAR-023: `mx_telecom_rights_overlay` applies subscriber, portability, complaint, and device context to rights requests.


- MX-DSR-CEDAR-024: `mx_atdt_identity_rights_boundary` blocks private reuse of government digital identity rights workflows.


- MX-DSR-CEDAR-025: `mx_dsr_audit_scrub_gate` rejects ARCO audit events containing raw personal data.


## Data Model Deltas

- MX-DSR-DATA-001: `mx_notice.id` stores the privacy notice identifier.


- MX-DSR-DATA-002: `mx_notice.controller_identity` stores controller name and contact route.


- MX-DSR-DATA-003: `mx_notice.purposes_primary` stores primary purposes.


- MX-DSR-DATA-004: `mx_notice.purposes_secondary` stores secondary purposes and opt-out routing.


- MX-DSR-DATA-005: `mx_notice.transfer_statement` stores transfer recipients and purposes.


- MX-DSR-DATA-006: `mx_notice.rights_mechanism` stores ARCO request channel and requirements.


- MX-DSR-DATA-007: `mx_notice.change_procedure` stores notice update mechanics.


- MX-DSR-DATA-008: `mx_notice.presentation_channel` stores collection-mode presentation.


- MX-DSR-DATA-009: `mx_consent.id` stores consent evidence identifier.


- MX-DSR-DATA-010: `mx_consent.mode` stores express, tacit, explicit-sensitive, revoked, or exception.


- MX-DSR-DATA-011: `mx_consent.notice_id` links consent to notice.


- MX-DSR-DATA-012: `mx_consent.purpose_id` links consent to purpose.


- MX-DSR-DATA-013: `mx_consent.evidence_digest` stores the evidence hash rather than raw capture.


- MX-DSR-DATA-014: `mx_consent.revoked_at` stores withdrawal timestamp.


- MX-DSR-DATA-015: `mx_consent.exception_code` stores LFPDPPP Art. 10 exception when applicable.


- MX-DSR-DATA-016: `mx_sensitive_category` stores health, genetics, beliefs, union, politics, sexual preference, biometric, financial, or other sensitive class.


- MX-DSR-DATA-017: `mx_arco_case.case_id` stores rights-request identifier.


- MX-DSR-DATA-018: `mx_arco_case.right_type` stores access, rectification, cancellation, or opposition.


- MX-DSR-DATA-019: `mx_arco_case.identity_status` stores pending, verified, rejected, representative-verified, or deficient.


- MX-DSR-DATA-020: `mx_arco_case.deadline_at` stores response deadline.


- MX-DSR-DATA-021: `mx_arco_case.decision` stores fulfilled, partial, denied, deficient, or withdrawn.


- MX-DSR-DATA-022: `mx_arco_case.denial_reason` stores legal, identity, retention, inexistence, duplicated, or other basis.


- MX-DSR-DATA-023: `mx_arco_case.delivery_channel` stores secure portal, postal, in-person, electronic, or other channel.


- MX-DSR-DATA-024: `mx_rights_transition.authority_code` stores Secretaria, legacy INAI, CNBV, CNSF, IFT, CRT, or ATDT context.


- MX-DSR-DATA-025: `mx_rights_audit.redaction_profile` stores audit redaction behavior for rights workflows.


## API Contract Deltas

- MX-DSR-API-001: `POST /privacy/mx/notices` registers a Mexico privacy notice.


- MX-DSR-API-002: `PATCH /privacy/mx/notices/{notice_id}` updates notice metadata and emits change event.


- MX-DSR-API-003: `GET /privacy/mx/notices/{notice_id}` returns notice metadata without private subject data.


- MX-DSR-API-004: `POST /privacy/mx/notices/{notice_id}/presentation` records notice presentation.


- MX-DSR-API-005: `POST /privacy/mx/consents` records consent linked to notice and purpose.


- MX-DSR-API-006: `POST /privacy/mx/consents/{consent_id}/revoke` records revocation.


- MX-DSR-API-007: `POST /privacy/mx/consent-exceptions` records Art. 10 exception evidence.


- MX-DSR-API-008: `POST /privacy/mx/sensitive-data/preflight` evaluates explicit consent or exception.


- MX-DSR-API-009: `POST /privacy/mx/arco/access` creates access case.


- MX-DSR-API-010: `POST /privacy/mx/arco/rectification` creates rectification case.


- MX-DSR-API-011: `POST /privacy/mx/arco/cancellation` creates cancellation case.


- MX-DSR-API-012: `POST /privacy/mx/arco/opposition` creates opposition case.


- MX-DSR-API-013: `POST /privacy/mx/arco/{case_id}/identity` records identity proof.


- MX-DSR-API-014: `POST /privacy/mx/arco/{case_id}/deficiency` records missing requirements.


- MX-DSR-API-015: `POST /privacy/mx/arco/{case_id}/decision` records fulfillment or denial.


- MX-DSR-API-016: `GET /privacy/mx/arco/{case_id}` returns status and deadline.


- MX-DSR-API-017: `GET /privacy/mx/arco/{case_id}/export` returns only authorized redacted response artifacts.


- MX-DSR-API-018: `POST /privacy/mx/portability/preflight` requires legal or sector review.


- MX-DSR-API-019: `POST /privacy/mx/rights/authority-transition` records legacy or current authority context.


- MX-DSR-API-020: `POST /privacy/mx/rights/sector-profile` records CNBV, CNSF, telecom, or ATDT overlays.


- MX-DSR-API-021: `GET /privacy/mx/rights/sector-profile/{case_id}` returns sector overlays for a rights case.


- MX-DSR-API-022: `POST /privacy/mx/rights/deny` records denial with article and evidence.


- MX-DSR-API-023: `GET /privacy/mx/rights/failure-modes` returns failure-mode catalog.


- MX-DSR-API-024: `GET /privacy/mx/rights/audit-events` returns ADR-0263 event classes.


- MX-DSR-API-025: `POST /privacy/mx/rights/audit-redaction-check` validates audit payload redaction.


## Audit Event Additions (per ADR-0263)

- MX-DSR-AUDIT-001: `MxPrivacyNoticeCreated` records notice id, controller, purposes, and authority snapshot.


- MX-DSR-AUDIT-002: `MxPrivacyNoticeUpdated` records notice id, changed fields, and effective date.


- MX-DSR-AUDIT-003: `MxPrivacyNoticePresented` records collection channel and subject evidence ref.


- MX-DSR-AUDIT-004: `MxConsentRecorded` records consent mode, purpose, notice, and evidence digest.


- MX-DSR-AUDIT-005: `MxConsentRevoked` records consent id and revocation timestamp.


- MX-DSR-AUDIT-006: `MxConsentExceptionRecorded` records Art. 10 exception code and evidence.


- MX-DSR-AUDIT-007: `MxSensitiveDataConsentValidated` records category and explicit-consent evidence.


- MX-DSR-AUDIT-008: `MxSensitiveDataConsentDenied` records missing or invalid explicit consent.


- MX-DSR-AUDIT-009: `MxArcoAccessRequested` records case id and identity status.


- MX-DSR-AUDIT-010: `MxArcoRectificationRequested` records correction case and evidence status.


- MX-DSR-AUDIT-011: `MxArcoCancellationRequested` records cancellation case and blocking review.


- MX-DSR-AUDIT-012: `MxArcoOppositionRequested` records opposition case and purpose selector.


- MX-DSR-AUDIT-013: `MxArcoIdentityVerified` records case id and verification method.


- MX-DSR-AUDIT-014: `MxArcoDeficiencyIssued` records missing identity, representation, or scope details.


- MX-DSR-AUDIT-015: `MxArcoDecisionIssued` records decision, authority basis, and delivery route.


- MX-DSR-AUDIT-016: `MxArcoAccessDelivered` records delivery channel and redaction profile.


- MX-DSR-AUDIT-017: `MxArcoRectificationApplied` records fields corrected without exposing values.


- MX-DSR-AUDIT-018: `MxArcoCancellationBlocked` records blocking reason and retention horizon.


- MX-DSR-AUDIT-019: `MxArcoOppositionApplied` records processing purpose disabled.


- MX-DSR-AUDIT-020: `MxArcoDenied` records denial reason and article basis.


- MX-DSR-AUDIT-021: `MxPortabilityReviewRequired` records portability-source and legal question.


- MX-DSR-AUDIT-022: `MxLegacyInaiRightsSourceUsed` records legacy INAI educational source and transition tag.


- MX-DSR-AUDIT-023: `MxRightsSectorOverlayApplied` records CNBV, CNSF, telecom, or ATDT overlay.


- MX-DSR-AUDIT-024: `MxRightsAuthorityTransitionResolved` records current authority path.


- MX-DSR-AUDIT-025: `MxRightsAuditPayloadRejected` records raw personal data in audit payload.


## Failure Modes

- MX-DSR-FAIL-001: Collection without a privacy notice is denied.


- MX-DSR-FAIL-002: Notice missing controller identity is denied.


- MX-DSR-FAIL-003: Notice missing purposes is denied.


- MX-DSR-FAIL-004: Notice missing transfer statement cannot support transfers.


- MX-DSR-FAIL-005: Notice missing ARCO route cannot support Mexico pack activation.


- MX-DSR-FAIL-006: Consent without purpose is denied.


- MX-DSR-FAIL-007: Consent without notice linkage is denied.


- MX-DSR-FAIL-008: Consent without evidence digest is denied.


- MX-DSR-FAIL-009: Sensitive-data processing under tacit consent is denied.


- MX-DSR-FAIL-010: Consent-exception use without Art. 10 code is denied.


- MX-DSR-FAIL-011: ARCO access without identity proof is held deficient, not fulfilled.


- MX-DSR-FAIL-012: ARCO rectification without correction evidence is held deficient.


- MX-DSR-FAIL-013: ARCO cancellation that conflicts with legal retention goes to blocking review.


- MX-DSR-FAIL-014: ARCO opposition that conflicts with legal obligation goes to denial review.


- MX-DSR-FAIL-015: ARCO deadline missing means the request cannot be accepted as complete.


- MX-DSR-FAIL-016: ARCO response export containing another subject's data is rejected.


- MX-DSR-FAIL-017: Portability claim copied from public-sector IFT material requires legal review.


- MX-DSR-FAIL-018: Legacy INAI enforcement route without current authority transition is rejected.


- MX-DSR-FAIL-019: CNBV record rights response without financial overlay is rejected.


- MX-DSR-FAIL-020: CNSF claim rights response without insurance overlay is rejected.


- MX-DSR-FAIL-021: Telecom subscriber rights response without telecom overlay is rejected.


- MX-DSR-FAIL-022: ATDT identity rights response reused for private-sector identity graph is rejected.


- MX-DSR-FAIL-023: Audit event containing raw request payload is rejected.


- MX-DSR-FAIL-024: Notice update without change procedure evidence is rejected.


- MX-DSR-FAIL-025: Data use beyond notice purpose is denied.


## Worked Examples

- MX-DSR-EXAMPLE-001: A checkout form collecting email and phone presents the privacy notice before submission.


- MX-DSR-EXAMPLE-002: A marketing opt-in records consent tied to secondary purpose.


- MX-DSR-EXAMPLE-003: A customer support workflow using contract necessity records Art. 10 exception evidence.


- MX-DSR-EXAMPLE-004: A health questionnaire requires express sensitive-data consent.


- MX-DSR-EXAMPLE-005: A biometric authentication feature requires sensitive-data classification and explicit consent.


- MX-DSR-EXAMPLE-006: A data subject asks for access and receives redacted treatment details after identity proof.


- MX-DSR-EXAMPLE-007: A subject corrects address data by supplying proof of current address.


- MX-DSR-EXAMPLE-008: A cancellation request for an active contract is blocked until contractual responsibilities expire.


- MX-DSR-EXAMPLE-009: An opposition request disables marketing processing but not legally required invoice retention.


- MX-DSR-EXAMPLE-010: A denied request cites the exact basis and preserves escalation evidence.


- MX-DSR-EXAMPLE-011: A CNBV financial account rights request routes through financial overlay before disclosure.


- MX-DSR-EXAMPLE-012: A CNSF claim-file access request hides third-party claimant data.


- MX-DSR-EXAMPLE-013: A telecom subscriber request for portability requires telecom overlay and current authority mapping.


- MX-DSR-EXAMPLE-014: A device identifier access request treats IMEI as potentially linked to subscriber data.


- MX-DSR-EXAMPLE-015: A public-sector IFT portability template is stored as reference, not private-sector authority.


- MX-DSR-EXAMPLE-016: A legacy INAI brochure is cited only as historical explanation of ARCO mechanics.


- MX-DSR-EXAMPLE-017: A Secretaria authority route is used for current LFPDPPP enforcement path.


- MX-DSR-EXAMPLE-018: A privacy notice update creates an audit event and new presentation requirement.


- MX-DSR-EXAMPLE-019: A consent revocation stops future optional processing while preserving audit history.


- MX-DSR-EXAMPLE-020: A subject-rights export is generated from scoped fields, not raw database dump.


- MX-DSR-EXAMPLE-021: A tenant admin cannot approve disclosure before identity proof.


- MX-DSR-EXAMPLE-022: A processor forwards rights request to controller and logs role evidence.


- MX-DSR-EXAMPLE-023: A request from an authorized representative stores representation evidence.


- MX-DSR-EXAMPLE-024: A deficient request stores what is missing without exposing extra personal data.


- MX-DSR-EXAMPLE-025: A rights workflow emitting raw payload to audit is rejected by ADR-0263 scrub checks.


## Cross-References

- MX-DSR-XREF-001: `README.md` names the pack activation posture.


- MX-DSR-XREF-002: `regulatory-coverage.md` maps notice, consent, and ARCO article ids.


- MX-DSR-XREF-003: `data-residency-and-cross-border.md` depends on notice and consent for transfer decisions.


- MX-DSR-XREF-004: `breach-notification-and-incident-response.md` depends on contact routes from the privacy notice.


- MX-DSR-XREF-005: `sectoral-overlays.md` defines CNBV, CNSF, telecom, and ATDT rights overlays.


- MX-DSR-XREF-006: ADR-0243 binds rights decisions to Cedar policy.


- MX-DSR-XREF-007: ADR-0244 binds rights decisions to tenant and sub-scope.


- MX-DSR-XREF-008: ADR-0251 binds rights controls into compliance-pack bundles.


- MX-DSR-XREF-009: ADR-0263 binds rights audit events to scrubbed emission.


- MX-DSR-XREF-010: LFPDPPP Art. 3(I) defines privacy notice.


- MX-DSR-XREF-011: LFPDPPP Art. 3(IV) defines consent.


- MX-DSR-XREF-012: LFPDPPP Art. 3(VII) defines ARCO.


- MX-DSR-XREF-013: LFPDPPP Art. 8 defines consent baseline.


- MX-DSR-XREF-014: LFPDPPP Art. 9 defines sensitive-data express consent.


- MX-DSR-XREF-015: LFPDPPP Art. 10 defines consent exceptions.


- MX-DSR-XREF-016: LFPDPPP Arts. 14-18 define privacy notice behavior.


- MX-DSR-XREF-017: LFPDPPP Arts. 22-28 define ARCO behavior.


- MX-DSR-XREF-018: CNBV sources affect rights handling for financial data.


- MX-DSR-XREF-019: CNSF sources affect rights handling for insurance data.


- MX-DSR-XREF-020: IFT and CRT sources affect telecom rights handling.


- MX-DSR-XREF-021: ATDT sources affect digital-government identity boundaries.


- MX-DSR-XREF-022: Legacy INAI sources require transition tagging.


- MX-DSR-XREF-023: Public-sector portability sources require scope review before private-sector use.


- MX-DSR-XREF-024: Rights exports must not contain unrelated subject data.


- MX-DSR-XREF-025: Rights audit rows must use redaction profiles, not raw values.

