---
doc_class: LocalizationPack
pack_id: MX-PACK-1
doc_id: MX-PACK-1-README
title: Mexico Localization Pack Overview
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
  - https://www.cnbv.gob.mx/ProteccionDatos/Paginas/default.aspx
  - https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20Instituciones%20de%20Tecnolog%C3%ADa%20Financiera.pdf
  - https://www.cnsf.gob.mx/Transparencia/Paginas/ProteccionDatosPersonales.aspx
  - https://www.ift.org.mx/
  - https://www.gob.mx/atdt
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LMTR.pdf
---

# Mexico Localization Pack Overview

## Overview

MX-PACK-1 is the Oyatie localization pack for Mexico private-sector personal data, regulated financial services, insurance and bonding, energy, and telecommunications workflows.
This slice authors documentation only for `/packs/mx-localization/`.
This slice does not alter Korea, European Union, United States, Japan, China, or any other geography pack.
This slice does not create Cedar source files, schemas, OpenAPI overlays, or generation scripts.
Runtime activation remains future work under the compliance-pack registry and Cedar bundle pipeline.

## Scope

The pack covers LFPDPPP private-sector personal-data duties, privacy notices, consent and exceptions, security measures, breach notification to affected holders, ARCO rights, transfer terms, and authority/enforcement routing.
The pack covers CNBV banking and fintech cybersecurity overlays, CNSF insurance and bonding privacy context, CRE energy user and reliability context, and telecom transition coverage spanning user-requested IFT materials plus CRT, ATDT, SICT, and LMTR transition sources.
The pack does not authorize regulated banking, insurance, energy, or telecom activity without sector licensing review.

## Version

Pack id: `MX-PACK-1`.
Pack version: `1.0.0`.
Pack status: `canonical-draft`.
Authority snapshot date: `2026-05-20`.
Review posture: Chamber of Deputies, CNBV, CNSF, CRE, IFT/CRT, ATDT, and LMTR sources control over secondary summaries.

## Citing Law

The cited legal baseline is the Ley Federal de Proteccion de Datos Personales en Posesion de los Particulares, CNBV privacy and fintech cybersecurity materials, CNSF data-protection and insurance circular materials, CRE authority materials, IFT/CRT telecom materials, ATDT transition materials, and the Ley en Materia de Telecomunicaciones y Radiodifusion.
Every implementation issue derived from this README must cite the article, regulator source, circular, or transition authority identifier, not only a URL.

## Authority Citations

- MX-README-AUTH-001: LFPDPPP Art. 1 is the federal private-sector personal-data anchor for this pack.


- MX-README-AUTH-002: LFPDPPP Art. 2 identifies excluded treatments, including purely personal or household use.


- MX-README-AUTH-003: LFPDPPP Art. 3 supplies terms such as aviso de privacidad, consent, data controller, processor, sensitive data, ARCO rights, and transfer.


- MX-README-AUTH-004: LFPDPPP Art. 6 sets the principles floor: lawfulness, consent, information, quality, purpose, loyalty, proportionality, and responsibility.


- MX-README-AUTH-005: LFPDPPP Art. 7 requires processing to follow the law and respect reasonable privacy expectations.


- MX-README-AUTH-006: LFPDPPP Arts. 8-10 provide the consent baseline and exceptions that later runtime policies must model.


- MX-README-AUTH-007: LFPDPPP Art. 14 requires informing data subjects through the privacy notice before or at collection.


- MX-README-AUTH-008: LFPDPPP Arts. 15-18 govern privacy notice content, availability, and collection-channel presentation.


- MX-README-AUTH-009: LFPDPPP Arts. 19-20 govern security and incident communication duties for personal data under controller responsibility.


- MX-README-AUTH-010: LFPDPPP Arts. 22-28 govern ARCO access, rectification, cancellation, opposition, request routing, identity proofing, and response clocks.


- MX-README-AUTH-011: LFPDPPP Arts. 36-37 govern transfers and require transfer terms to be communicated and accepted by recipients.


- MX-README-AUTH-012: LFPDPPP Arts. 38-39 route 2025-era authority functions to the Secretaria for promotion, vigilance, verification, and sanctions.


- MX-README-AUTH-013: The Chamber reference page records the LFPDPPP current text and reform lineage, including the DOF 2025 replacement and later amendment.


- MX-README-AUTH-014: CNBV's data-protection page is an official authority source for how the banking and securities supervisor presents privacy and ARCO obligations.


- MX-README-AUTH-015: CNBV fintech cybersecurity provisions are official sector rules for ITF security, electronic means, third parties, and information-security governance.


- MX-README-AUTH-016: CNSF's data-protection page is an official authority source for insurance and bonding supervisory data-processing inventories and ARCO handling.


- MX-README-AUTH-017: IFT's public site and the CRT portal preserve telecom regulator continuity and user-rights materials during the transition from the former IFT surface.


- MX-README-AUTH-018: ATDT's official site is cited for 2025-2026 transformation-digital and telecommunications transition authority context.


- MX-README-AUTH-019: LMTR Art. 4 treats spectrum, telecom public networks, broadcasting stations, equipment, and satellite systems as general communication routes.


- MX-README-AUTH-020: LMTR Art. 5 makes those telecom and broadcasting routes federal jurisdiction and declares telecom infrastructure of public interest and utility.


## Activated Cedar Policies

- MX-README-CEDAR-001: `mx_pack_active` gates every Mexico-specific allow decision on an installed `MX-PACK-1` tenant bundle.


- MX-README-CEDAR-002: `mx_lfpdppp_private_sector_scope` denies generic private-sector personal-data processing when the controller cannot classify applicability under LFPDPPP Arts. 1-2.


- MX-README-CEDAR-003: `mx_privacy_notice_required` denies collection unless an aviso de privacidad record exists for the purpose, collection channel, and data category.


- MX-README-CEDAR-004: `mx_consent_required_unless_exception` denies processing unless consent exists or a mapped LFPDPPP Art. 10 exception is present.


- MX-README-CEDAR-005: `mx_sensitive_data_explicit_consent` denies sensitive-data processing unless the explicit-sensitive path is satisfied and separately auditable.


- MX-README-CEDAR-006: `mx_arco_intake_enabled` requires a subject-rights endpoint before any tenant claims Mexico private-sector coverage.


- MX-README-CEDAR-007: `mx_arco_response_clock` starts a Mexico ARCO deadline timer when a complete request is accepted.


- MX-README-CEDAR-008: `mx_transfer_recipient_bound` denies transfer unless the recipient is bound to the privacy notice and transfer purpose.


- MX-README-CEDAR-009: `mx_cross_border_transfer_review` requires destination, recipient, transfer basis, and sector overlay before personal data leaves the Mexico placement plan.


- MX-README-CEDAR-010: `mx_breach_notify_subjects` requires affected-person notification when a security vulnerability materially affects property or moral rights.


- MX-README-CEDAR-011: `mx_cnbv_financial_overlay` applies banking and fintech security rules when the tenant operates as a CNBV-supervised or CNBV-adjacent financial entity.


- MX-README-CEDAR-012: `mx_cnsf_insurance_overlay` applies insurance and bonding evidence rules when claims, policyholder, beneficiary, actuarial, or agent records are in scope.


- MX-README-CEDAR-013: `mx_cre_energy_overlay` requires permit, metering, reliability, user, and infrastructure-context evidence before energy workflows process regulated records.


- MX-README-CEDAR-014: `mx_telecom_user_privacy_overlay` requires telecom user privacy and rights controls for subscriber, usage, traffic, device, IMEI, and complaint workflows.


- MX-README-CEDAR-015: `mx_ift_crt_transition_guard` blocks regulator-specific automation when the request cannot distinguish historical IFT material from current CRT or ATDT authority.


- MX-README-CEDAR-016: `mx_atdt_digital_identity_guard` requires Llave MX or digital-government identity data to remain segregated from private-sector tenant identity graphs.


- MX-README-CEDAR-017: `mx_sector_conflict_strictest_wins` chooses the stricter rule when LFPDPPP, CNBV, CNSF, CRE, telecom, or tenant contract duties conflict.


- MX-README-CEDAR-018: `mx_authority_snapshot_required` denies production activation when the authority snapshot date is older than the configured refresh threshold.


- MX-README-CEDAR-019: `mx_no_other_geo_mutation` records that this pack must not activate or mutate kr, eu, us, jp, cn, or other geography controls.


- MX-README-CEDAR-020: `mx_audit_scrubbed_payload_only` enforces ADR-0263 by denying audit emission that contains raw sensitive personal data.


## Data Model Deltas

- MX-README-DATA-001: Add `mx_pack_state` with pack id, version, authority snapshot date, and activation status.


- MX-README-DATA-002: Add `mx_lfpdppp_scope_classification` with private-sector, public-sector, household, processor-only, and out-of-scope values.


- MX-README-DATA-003: Add `mx_privacy_notice_id` to every Mexico-covered processing activity.


- MX-README-DATA-004: Add `mx_notice_channel` for physical, electronic, optical, audio, visual, or other technology collection paths.


- MX-README-DATA-005: Add `mx_consent_record_id` for consented processing.


- MX-README-DATA-006: Add `mx_consent_exception_code` for LFPDPPP Art. 10 exception mapping.


- MX-README-DATA-007: Add `mx_sensitive_personal_data_flag` for categories that may affect the most intimate sphere or create discrimination risk.


- MX-README-DATA-008: Add `mx_arco_case_id` for access, rectification, cancellation, and opposition requests.


- MX-README-DATA-009: Add `mx_arco_identity_evidence_ref` for identity and representative verification.


- MX-README-DATA-010: Add `mx_transfer_basis` for domestic, cross-border, controller-processor, legal obligation, consented, or exception transfer paths.


- MX-README-DATA-011: Add `mx_transfer_recipient_notice_acceptance` to record that the recipient accepted privacy-notice limitations.


- MX-README-DATA-012: Add `mx_breach_impact_rights_assessment` to decide whether notification duties are triggered.


- MX-README-DATA-013: Add `mx_sector_overlay_codes` for CNBV, CNSF, CRE, telecom, ATDT digital identity, and tenant-contract overlays.


- MX-README-DATA-014: Add `mx_regulator_transition_state` with legacy-INAI, Secretaria, CRT, ATDT, SICT, and unknown values as applicable.


- MX-README-DATA-015: Add `mx_financial_security_profile` for CNBV banking, fintech, outsourcing, electronic means, and incident response evidence.


- MX-README-DATA-016: Add `mx_insurance_record_kind` for policyholder, beneficiary, claim, agent, adjuster, actuarial, and regulatory-reporting records.


- MX-README-DATA-017: Add `mx_energy_record_kind` for customer, meter, permit, dispatch, reliability, billing, and field-service records.


- MX-README-DATA-018: Add `mx_telecom_record_kind` for subscriber, traffic, device, network, portability, accessibility, and complaint records.


- MX-README-DATA-019: Add `mx_authority_url_refs` to each pack decision for exact official-source traceability.


- MX-README-DATA-020: Add `mx_audit_payload_scrub_profile` so ADR-0263 audit rows identify redaction behavior without exposing personal data.


## API Contract Deltas

- MX-README-API-001: `POST /localization/mx/activate` must require pack version, authority snapshot, and tenant scope.


- MX-README-API-002: `GET /localization/mx/authority-snapshot` must return official URLs and article ids used by the active pack.


- MX-README-API-003: `POST /privacy/mx/notices` must accept privacy notice metadata before collection begins.


- MX-README-API-004: `GET /privacy/mx/notices/{id}` must expose notice purpose, transfer statements, contact route, and rights route.


- MX-README-API-005: `POST /privacy/mx/consents` must persist consent evidence, channel, language, timestamp, and subject identity.


- MX-README-API-006: `POST /privacy/mx/consent-exceptions` must require a coded statutory exception and legal-evidence reference.


- MX-README-API-007: `POST /privacy/mx/arco/access` must create an access request with identity proof and target data scope.


- MX-README-API-008: `POST /privacy/mx/arco/rectification` must create a correction request with evidence for the proposed correction.


- MX-README-API-009: `POST /privacy/mx/arco/cancellation` must create a cancellation request and blocking assessment.


- MX-README-API-010: `POST /privacy/mx/arco/opposition` must create an opposition request with processing-purpose selector.


- MX-README-API-011: `GET /privacy/mx/arco/{case_id}` must expose status, deadline, decision, and escalation path.


- MX-README-API-012: `POST /privacy/mx/transfers/preflight` must return transfer allow, deny, or legal-review-required.


- MX-README-API-013: `POST /privacy/mx/breaches` must create a breach-impact assessment and notification plan.


- MX-README-API-014: `POST /sector/mx/cnbv/security-events` must receive CNBV overlay events without raw secrets or credentials.


- MX-README-API-015: `POST /sector/mx/cnsf/reporting-records` must classify insurance supervisory records by treatment purpose.


- MX-README-API-016: `POST /sector/mx/cre/regulated-records` must classify energy records by permit, user, metering, or reliability purpose.


- MX-README-API-017: `POST /sector/mx/telecom/user-rights` must classify telecom privacy, portability, accessibility, and complaint workflows.


- MX-README-API-018: `POST /sector/mx/authority-transition-check` must return whether current request references INAI, Secretaria, IFT, CRT, ATDT, or SICT.


- MX-README-API-019: `GET /audit/mx/events/{id}` must return ADR-0263 event metadata with personal-data fields redacted.


- MX-README-API-020: `POST /localization/mx/deactivate` must preserve retention, audit, blocked-data, and legal-hold obligations after pack deactivation.


## Audit Event Additions (per ADR-0263)

- MX-README-AUDIT-001: `MxPackActivated` records tenant, pack version, authority snapshot, and actor.


- MX-README-AUDIT-002: `MxAuthoritySnapshotLoaded` records official URLs, article ids, and freshness result.


- MX-README-AUDIT-003: `MxPrivacyNoticeRegistered` records notice id, purpose, channel, transfer language, and owner.


- MX-README-AUDIT-004: `MxPrivacyNoticePresented` records subject, notice id, collection channel, and presentation timestamp.


- MX-README-AUDIT-005: `MxConsentCaptured` records consent id, notice id, purpose, and evidence digest.


- MX-README-AUDIT-006: `MxConsentExceptionApplied` records exception code, approving actor, and legal evidence ref.


- MX-README-AUDIT-007: `MxSensitiveDataProcessingDenied` records data class, purpose, and missing explicit-consent reason.


- MX-README-AUDIT-008: `MxArcoRequestAccepted` records ARCO type, case id, requester, identity-proof status, and deadline.


- MX-README-AUDIT-009: `MxArcoRequestCompleted` records case id, outcome, delivery route, and redaction profile.


- MX-README-AUDIT-010: `MxArcoRequestDenied` records denial reason, authority basis, and appeal/escalation route.


- MX-README-AUDIT-011: `MxTransferPreflightCompleted` records destination, recipient, transfer basis, and sector overlays.


- MX-README-AUDIT-012: `MxTransferRecipientBound` records recipient acceptance of privacy-notice constraints.


- MX-README-AUDIT-013: `MxCrossBorderTransferDenied` records destination, missing basis, and legal-review checkpoint.


- MX-README-AUDIT-014: `MxBreachImpactAssessed` records incident id, affected rights assessment, and notification trigger.


- MX-README-AUDIT-015: `MxBreachSubjectNoticeIssued` records notification batch, channel, and delivery evidence.


- MX-README-AUDIT-016: `MxCnbvOverlayApplied` records financial-service overlay and security-control bundle.


- MX-README-AUDIT-017: `MxCnsfOverlayApplied` records insurance or bonding treatment purpose and supervisory context.


- MX-README-AUDIT-018: `MxCreOverlayApplied` records energy permit, user, reliability, or metering context.


- MX-README-AUDIT-019: `MxTelecomTransitionResolved` records whether IFT, CRT, ATDT, or SICT authority was used.


- MX-README-AUDIT-020: `MxAuditPayloadRejected` records ADR-0263 denial when an event attempts to include raw personal data.


## Failure Modes

- MX-README-FAIL-001: Fail closed when the tenant cannot prove LFPDPPP applicability or exclusion.


- MX-README-FAIL-002: Fail closed when a privacy notice is missing, stale, or not tied to the collection channel.


- MX-README-FAIL-003: Fail closed when consent is required but the consent evidence is not specific, informed, or replayable.


- MX-README-FAIL-004: Fail closed when sensitive data is processed under generic consent.


- MX-README-FAIL-005: Fail closed when an ARCO request lacks identity proof but preserve an intake record.


- MX-README-FAIL-006: Fail closed when an ARCO response deadline cannot be computed.


- MX-README-FAIL-007: Fail closed when a transfer recipient has not accepted privacy-notice constraints.


- MX-README-FAIL-008: Fail legal-review-required when cross-border transfers involve CNBV, CNSF, CRE, telecom, or ATDT identity data.


- MX-README-FAIL-009: Fail notification-required when breach impact on property or moral rights cannot be ruled out.


- MX-README-FAIL-010: Fail compliance-review-required when legacy INAI enforcement materials conflict with the 2025 LFPDPPP authority text.


- MX-README-FAIL-011: Fail transition-review-required when a telecom workflow references the former IFT without mapping the current CRT or ATDT surface.


- MX-README-FAIL-012: Fail sector-review-required when banking, fintech, insurance, energy, or telecom classification is unknown.


- MX-README-FAIL-013: Fail closed when audit payloads include raw sensitive data.


- MX-README-FAIL-014: Fail closed when the authority snapshot omits article ids and only stores generic URLs.


- MX-README-FAIL-015: Fail closed when another geography pack would be edited or activated by the Mexico slice.


- MX-README-FAIL-016: Fail closed when pack deactivation would erase required audit, legal hold, or blocked-data records.


- MX-README-FAIL-017: Fail closed when regulator reporting uses unencrypted exports or unsanitized logs.


- MX-README-FAIL-018: Fail closed when sector overlays disagree and no strictest-duty resolution exists.


- MX-README-FAIL-019: Fail review-required when public-sector data subject rules are confused with private-sector LFPDPPP rules.


- MX-README-FAIL-020: Fail review-required when Mexico user records are commingled with non-Mexico telemetry without redaction evidence.


## Worked Examples

- MX-README-EXAMPLE-001: A SaaS tenant collecting Mexican customer emails must present an aviso de privacidad and store `mx_privacy_notice_id` before collection.


- MX-README-EXAMPLE-002: A tenant using health data for insurance underwriting must activate CNSF and sensitive-data paths, not only generic LFPDPPP consent.


- MX-README-EXAMPLE-003: A fintech tenant under CNBV-adjacent rules must classify security incidents through the CNBV overlay and ADR-0263 audit events.


- MX-README-EXAMPLE-004: A telecom tenant importing subscriber data must classify subscriber, device, traffic, portability, and complaint records before API ingestion.


- MX-README-EXAMPLE-005: A Mexico customer deletion request becomes `MxArcoRequestAccepted` with cancellation type and response clock.


- MX-README-EXAMPLE-006: A correction request for an insurance beneficiary record requires identity proof and relationship proof before rectification.


- MX-README-EXAMPLE-007: A cross-border support access request to view Mexican payroll data must run transfer preflight even when data stays hosted in Mexico.


- MX-README-EXAMPLE-008: A model-training export that includes Mexican customer text is denied unless data is disassociated or a valid transfer basis exists.


- MX-README-EXAMPLE-009: A breach involving leaked customer phone numbers creates impact assessment, subject notification planning, and audit-chain evidence.


- MX-README-EXAMPLE-010: A CNBV fintech credential leak is both a personal-data breach assessment and a sector cybersecurity event.


- MX-README-EXAMPLE-011: A CNSF regulatory report can include policyholder identifiers only through the stated supervisory treatment purpose.


- MX-README-EXAMPLE-012: A CRE energy-meter record is not automatically financial data but can become personal data when linked to an identifiable user.


- MX-README-EXAMPLE-013: A telecom IMEI workflow must use the telecom overlay because device identifiers can link to subscribers and user rights.


- MX-README-EXAMPLE-014: A user-rights page citing IFT materials must record whether it is historical IFT, current CRT, or ATDT transition context.


- MX-README-EXAMPLE-015: A tenant using Llave MX-derived identity data must segregate government digital-identity claims from private SaaS identity data.


- MX-README-EXAMPLE-016: A processor-only tenant must still store controller instructions, processor role, and audit proof.


- MX-README-EXAMPLE-017: A household-use exclusion is denied for business analytics because LFPDPPP Art. 2 exclusion does not fit enterprise processing.


- MX-README-EXAMPLE-018: A transfer to a foreign CRM vendor is denied until recipient obligations, notice language, and sector overlays are complete.


- MX-README-EXAMPLE-019: A security log containing CURP-like identifiers is rejected by ADR-0263 scrub policy unless redacted or hashed with approved profile.


- MX-README-EXAMPLE-020: A tenant turning off MX-PACK-1 must keep ARCO, breach, transfer, and legal-hold records until retention duties expire.


## Cross-References

- MX-README-XREF-001: See `regulatory-coverage.md` for law-to-control mapping across LFPDPPP, CNBV, CNSF, CRE, telecom, and ATDT transition surfaces.


- MX-README-XREF-002: See `data-residency-and-cross-border.md` for domestic placement, remote access, cross-border transfers, and regulator reporting.


- MX-README-XREF-003: See `consent-and-data-subject-rights.md` for consent, privacy notices, ARCO, portability, and transition authority handling.


- MX-README-XREF-004: See `breach-notification-and-incident-response.md` for incident intake, breach impact, notification, and sector escalation.


- MX-README-XREF-005: See `sectoral-overlays.md` for CNBV banking and fintech, CNSF insurance, CRE energy, IFT/CRT/ATDT telecom, and telecoms.


- MX-README-XREF-006: ADR-0243 is the Cedar-as-universal-gate baseline for every policy name in this pack.


- MX-README-XREF-007: ADR-0244 is the tenant and sub-scope baseline for every Mexico pack decision.


- MX-README-XREF-008: ADR-0251 is the compliance-pack bundle model that will later turn these docs into signed runtime bundles.


- MX-README-XREF-009: ADR-0263 is the audit emission baseline; every event listed here must scrub PII at emission.


- MX-README-XREF-010: LFPDPPP Art. 6 principles should be mirrored by runtime data minimization and purpose checks.


- MX-README-XREF-011: LFPDPPP Arts. 15-18 privacy notice controls should drive notice API contracts.


- MX-README-XREF-012: LFPDPPP Arts. 22-28 ARCO controls should drive subject-rights workflow contracts.


- MX-README-XREF-013: LFPDPPP Arts. 36-37 transfer controls should drive cross-border preflight contracts.


- MX-README-XREF-014: LFPDPPP Arts. 38-39 authority controls should drive regulator-transition metadata.


- MX-README-XREF-015: CNBV data-protection and fintech cybersecurity sources should drive banking and fintech overlays.


- MX-README-XREF-016: CNSF data-protection and CUSF sources should drive insurance and bonding overlays.


- MX-README-XREF-017: CRE public authority sources should drive energy user, permit, metering, and reliability overlays.


- MX-README-XREF-018: IFT and CRT sources should drive telecom user-rights and historical-regulator mapping.


- MX-README-XREF-019: ATDT sources should drive digital-government and telecom transition notes.


- MX-README-XREF-020: LMTR Arts. 4-5 should drive telecom infrastructure and federal-jurisdiction classification.


## Pack Completion Ledger

- MX-README-LEDGER-001: This directory is intentionally limited to the six requested Markdown documents.


- MX-README-LEDGER-002: `README.md` is the overview and activation map.


- MX-README-LEDGER-003: `regulatory-coverage.md` is the law and regulator coverage matrix.


- MX-README-LEDGER-004: `data-residency-and-cross-border.md` is the placement and transfer control map.


- MX-README-LEDGER-005: `consent-and-data-subject-rights.md` is the notice, consent, ARCO, and rights control map.


- MX-README-LEDGER-006: `breach-notification-and-incident-response.md` is the incident and notification control map.


- MX-README-LEDGER-007: `sectoral-overlays.md` is the regulated-sector control map.


- MX-README-LEDGER-008: Mexico private-sector privacy coverage starts with LFPDPPP Arts. 1-3.


- MX-README-LEDGER-009: Mexico processing principles start with LFPDPPP Art. 6.


- MX-README-LEDGER-010: Mexico consent handling starts with LFPDPPP Arts. 8-10.


- MX-README-LEDGER-011: Mexico privacy notice handling starts with LFPDPPP Arts. 14-18.


- MX-README-LEDGER-012: Mexico security and breach handling starts with LFPDPPP Arts. 19-20.


- MX-README-LEDGER-013: Mexico ARCO handling starts with LFPDPPP Arts. 22-28.


- MX-README-LEDGER-014: Mexico transfer handling starts with LFPDPPP Arts. 36-37.


- MX-README-LEDGER-015: Mexico post-2025 authority handling starts with LFPDPPP Arts. 38-39.


- MX-README-LEDGER-016: CNBV banking and fintech overlays remain sector-specific and do not replace the LFPDPPP floor.


- MX-README-LEDGER-017: CNSF insurance and bonding overlays remain sector-specific and do not replace the LFPDPPP floor.


- MX-README-LEDGER-018: CRE energy overlays remain sector-specific and do not replace the LFPDPPP floor.


- MX-README-LEDGER-019: IFT materials remain useful for historical telecom coverage and must be paired with CRT or ATDT transition context.


- MX-README-LEDGER-020: ATDT materials identify the current transformation-digital and telecom transition context.


- MX-README-LEDGER-021: LMTR Arts. 4-5 identify telecom infrastructure and federal-jurisdiction context.


- MX-README-LEDGER-022: Every runtime event named by this pack must emit through ADR-0263 scrubbed audit semantics.


- MX-README-LEDGER-023: Every future Cedar policy named by this pack must remain deny-first for missing authority, scope, or evidence.


- MX-README-LEDGER-024: Every future data-model delta named by this pack must carry tenant and sub-scope context.


- MX-README-LEDGER-025: Every future API delta named by this pack must reject raw personal data in audit payloads.


- MX-README-LEDGER-026: Every future transfer decision named by this pack must distinguish storage location from legal transfer.


- MX-README-LEDGER-027: Every future rights workflow named by this pack must distinguish private-sector LFPDPPP from public-sector transparency-unit practice.


- MX-README-LEDGER-028: Every future sector workflow named by this pack must resolve conflicts by strictest applicable duty.


- MX-README-LEDGER-029: Every future authority refresh must check official URLs before changing pack status.


- MX-README-LEDGER-030: Every future Mexico pack implementation must preserve this slice boundary unless a new task explicitly widens it.
