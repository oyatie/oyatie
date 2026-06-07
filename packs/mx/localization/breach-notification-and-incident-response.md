---
doc_class: LocalizationPack
pack_id: MX-PACK-1
doc_id: MX-PACK-1-BREACH-INCIDENT-RESPONSE
title: Mexico Breach Notification and Incident Response
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0297
citing_authority_url:
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LFPDPPP.pdf
  - https://www.diputados.gob.mx/LeyesBiblio/ref/lfpdppp.htm
  - https://www.cnbv.gob.mx/ProteccionDatos/Paginas/default.aspx
  - https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20Instituciones%20de%20Tecnolog%C3%ADa%20Financiera.pdf
  - https://www.cnsf.gob.mx/Transparencia/Paginas/ProteccionDatosPersonales.aspx
  - https://www.gob.mx/cnsf/documentos/circular-unica-de-seguros-y-fianzas
  - https://www.gob.mx/cre/que-hacemos
  - https://www.ift.org.mx/
  - https://www.gob.mx/atdt
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LMTR.pdf
---

# Mexico Breach Notification and Incident Response

This document defines Mexico incident classification, breach impact, subject notification, sector escalation, evidence preservation, and ADR-0263 audit behavior.
The Mexico privacy trigger is not a generic 72-hour rule in this pack.
The Mexico privacy trigger is whether a security vulnerability materially affects patrimonial or moral rights under LFPDPPP Art. 20.
Sector overlays may introduce additional reporting or supervisory expectations.
CNBV financial and fintech incidents require heightened information-security handling.
CNSF insurance incidents require supervisory and claims-data context.
CRE energy incidents require operational and user-impact context.
Telecom incidents require user, device, network, and regulator-transition context.

## Authority Citations

- MX-IR-AUTH-001: LFPDPPP Art. 3(V) defines personal data and anchors breach scope.


- MX-IR-AUTH-002: LFPDPPP Art. 3(VI) defines sensitive personal data and heightened breach impact.


- MX-IR-AUTH-003: LFPDPPP Art. 3(III) defines blocking, relevant after incident containment and purpose completion.


- MX-IR-AUTH-004: LFPDPPP Art. 6 responsibility and security principles guide incident evidence.


- MX-IR-AUTH-005: LFPDPPP Art. 13 proportionality shapes minimum necessary investigation access.


- MX-IR-AUTH-006: LFPDPPP Art. 16 notice contact routes affect subject notification delivery.


- MX-IR-AUTH-007: LFPDPPP Art. 19 requires security measures against damage, loss, alteration, destruction, unauthorized use, access, or treatment.


- MX-IR-AUTH-008: LFPDPPP Art. 20 requires prompt notification to affected holders when a security vulnerability materially affects patrimonial or moral rights.


- MX-IR-AUTH-009: LFPDPPP Art. 21 requires the controller to establish and maintain administrative, technical, and physical measures proportional to data risk.


- MX-IR-AUTH-010: LFPDPPP Arts. 22-28 provide post-incident ARCO routing for affected subjects.


- MX-IR-AUTH-011: LFPDPPP Arts. 36-37 govern incident-related disclosure to processors, vendors, regulators, affiliates, or foreign responders.


- MX-IR-AUTH-012: LFPDPPP Arts. 38-39 identify current federal authority functions after the 2025 replacement.


- MX-IR-AUTH-013: The Chamber reference page records current law text and reform history for incident authority.


- MX-IR-AUTH-014: CNBV data-protection pages are official sources for financial-supervisor privacy handling.


- MX-IR-AUTH-015: CNBV fintech cybersecurity provisions create sector information-security expectations for ITF platforms and third parties.


- MX-IR-AUTH-016: CNSF data-protection pages list insurance and bonding treatment inventories relevant to incident scoping.


- MX-IR-AUTH-017: CUSF official CNSF materials provide insurance and bonding regulatory context for supervised entities.


- MX-IR-AUTH-018: CRE authority sources provide energy-sector user, reliability, and regulated-activity context.


- MX-IR-AUTH-019: IFT and CRT telecom sources provide telecom user-rights and regulator-continuity context.


- MX-IR-AUTH-020: ATDT sources provide transformation-digital, telecom transition, and public digital identity context.


- MX-IR-AUTH-021: LMTR Art. 4 defines telecom infrastructure and systems relevant to incident scope.


- MX-IR-AUTH-022: LMTR Art. 5 establishes federal jurisdiction and public-interest status for telecom infrastructure.


- MX-IR-AUTH-023: ADR-0297 abuse-defense controls are relevant when incidents involve bot, spoof, scrape, or platform abuse.


- MX-IR-AUTH-024: ADR-0263 requires structured, PII-scrubbed logs, metrics, traces, and audit events during incidents.


- MX-IR-AUTH-025: ADR-0243 requires incident-response allow and deny actions to be policy-gated, not ad hoc.


## Activated Cedar Policies

- MX-IR-CEDAR-001: `mx_incident_intake_required` creates an incident record before investigation access.


- MX-IR-CEDAR-002: `mx_incident_personal_data_scope` checks whether incident data includes LFPDPPP personal data.


- MX-IR-CEDAR-003: `mx_incident_sensitive_data_scope` checks whether sensitive data is involved.


- MX-IR-CEDAR-004: `mx_incident_security_measure_review` verifies LFPDPPP Art. 19 control posture.


- MX-IR-CEDAR-005: `mx_incident_material_rights_assessment` determines whether Art. 20 notification is triggered.


- MX-IR-CEDAR-006: `mx_incident_notification_required` requires subject notification when material rights impact is true.


- MX-IR-CEDAR-007: `mx_incident_notification_not_required_reason` requires a coded reason when notification is not sent.


- MX-IR-CEDAR-008: `mx_incident_minimum_access` restricts responders to minimum necessary data.


- MX-IR-CEDAR-009: `mx_incident_forensics_redaction` requires redacted forensic exports where possible.


- MX-IR-CEDAR-010: `mx_incident_vendor_access_preflight` routes vendors through transfer and support-access review.


- MX-IR-CEDAR-011: `mx_incident_cross_border_preflight` applies transfer controls to foreign response support.


- MX-IR-CEDAR-012: `mx_incident_arco_aftercare` enables ARCO guidance for affected subjects.


- MX-IR-CEDAR-013: `mx_incident_blocking_after_purpose` moves data to blocking when purpose ends but responsibility windows remain.


- MX-IR-CEDAR-014: `mx_incident_cnbv_overlay` applies financial cybersecurity and supervisory evidence.


- MX-IR-CEDAR-015: `mx_incident_cnbv_itf_overlay` applies ITF cybersecurity provisions for fintech institutions.


- MX-IR-CEDAR-016: `mx_incident_cnsf_overlay` applies insurance and bonding supervisory context.


- MX-IR-CEDAR-017: `mx_incident_cre_overlay` applies energy reliability, user, permit, and operational context.


- MX-IR-CEDAR-018: `mx_incident_telecom_overlay` applies subscriber, traffic, network, device, and user-rights context.


- MX-IR-CEDAR-019: `mx_incident_atdt_identity_overlay` applies public digital identity segregation context.


- MX-IR-CEDAR-020: `mx_incident_telecom_transition_gate` requires IFT, CRT, ATDT, SICT, or LMTR mapping.


- MX-IR-CEDAR-021: `mx_incident_regulator_contact_gate` blocks regulator contact automation without current authority code.


- MX-IR-CEDAR-022: `mx_incident_evidence_preservation` preserves audit, logs, forensic hashes, and decisions.


- MX-IR-CEDAR-023: `mx_incident_audit_scrub_gate` rejects raw personal data in audit events.


- MX-IR-CEDAR-024: `mx_incident_public_statement_gate` blocks public claims without authority-approved facts.


- MX-IR-CEDAR-025: `mx_incident_strictest_sector_gate` applies stricter sector reporting and notification duties.


## Data Model Deltas

- MX-IR-DATA-001: `mx_incident.incident_id` stores incident identifier.


- MX-IR-DATA-002: `mx_incident.detected_at` stores detection timestamp.


- MX-IR-DATA-003: `mx_incident.discovered_by` stores detector role, system, or external reporter.


- MX-IR-DATA-004: `mx_incident.personal_data_involved` stores true, false, or unknown.


- MX-IR-DATA-005: `mx_incident.sensitive_data_involved` stores true, false, or unknown.


- MX-IR-DATA-006: `mx_incident.data_categories` stores ordinary, sensitive, financial, insurance, energy, telecom, device, traffic, or identity classes.


- MX-IR-DATA-007: `mx_incident.security_failure_type` stores damage, loss, alteration, destruction, unauthorized use, access, treatment, or unknown.


- MX-IR-DATA-008: `mx_incident.material_rights_impact` stores yes, no, unknown, or legal-review.


- MX-IR-DATA-009: `mx_incident.impact_reason` stores patrimonial, moral, both, none, or unclear.


- MX-IR-DATA-010: `mx_incident.notification_required` stores yes, no, or pending.


- MX-IR-DATA-011: `mx_incident.notification_basis` stores LFPDPPP Art. 20 or sector basis.


- MX-IR-DATA-012: `mx_incident.notification_channels` stores email, postal, phone, portal, public notice, regulator, or other.


- MX-IR-DATA-013: `mx_incident.affected_subject_count` stores count or bounded estimate.


- MX-IR-DATA-014: `mx_incident.regulated_sector_codes` stores CNBV, CNSF, CRE, telecom, ATDT, or none.


- MX-IR-DATA-015: `mx_incident.authority_transition_code` stores Secretaria, legacy INAI, IFT, CRT, ATDT, SICT, or LMTR context.


- MX-IR-DATA-016: `mx_incident.forensic_evidence_refs` stores hashes, locations, and access controls.


- MX-IR-DATA-017: `mx_incident.vendor_access_refs` stores support or forensics vendor access approvals.


- MX-IR-DATA-018: `mx_incident.cross_border_refs` stores transfer preflight ids.


- MX-IR-DATA-019: `mx_incident.containment_actions` stores disabled keys, revoked sessions, blocked exports, or isolated systems.


- MX-IR-DATA-020: `mx_incident.recovery_actions` stores restoration, patch, credential rotation, or data correction actions.


- MX-IR-DATA-021: `mx_incident.subject_aftercare_refs` stores ARCO, monitoring, support, or advisory references.


- MX-IR-DATA-022: `mx_incident.public_statement_refs` stores approved external communications.


- MX-IR-DATA-023: `mx_incident.lessons_learned_refs` stores post-incident review artifacts.


- MX-IR-DATA-024: `mx_incident.audit_redaction_profile` stores ADR-0263 scrub profile.


- MX-IR-DATA-025: `mx_incident.closed_at` stores closure timestamp only after evidence completion.


## API Contract Deltas

- MX-IR-API-001: `POST /incidents/mx` creates Mexico incident intake.


- MX-IR-API-002: `GET /incidents/mx/{incident_id}` returns incident metadata without raw personal data.


- MX-IR-API-003: `POST /incidents/mx/{incident_id}/scope` records personal-data and sensitive-data scope.


- MX-IR-API-004: `POST /incidents/mx/{incident_id}/security-review` records Art. 19 control review.


- MX-IR-API-005: `POST /incidents/mx/{incident_id}/rights-impact` records Art. 20 material-rights assessment.


- MX-IR-API-006: `POST /incidents/mx/{incident_id}/notification-plan` records subject notification plan.


- MX-IR-API-007: `POST /incidents/mx/{incident_id}/notification-batch` records notification dispatch evidence.


- MX-IR-API-008: `POST /incidents/mx/{incident_id}/no-notification` records no-notification reason.


- MX-IR-API-009: `POST /incidents/mx/{incident_id}/sector-overlay` records CNBV, CNSF, CRE, telecom, or ATDT overlay.


- MX-IR-API-010: `POST /incidents/mx/{incident_id}/telecom-transition` records IFT, CRT, ATDT, SICT, or LMTR authority context.


- MX-IR-API-011: `POST /incidents/mx/{incident_id}/regulator-contact` records current authority contact decision.


- MX-IR-API-012: `POST /incidents/mx/{incident_id}/vendor-access/preflight` evaluates forensics or support vendor access.


- MX-IR-API-013: `POST /incidents/mx/{incident_id}/cross-border/preflight` evaluates foreign responder access.


- MX-IR-API-014: `POST /incidents/mx/{incident_id}/containment` records containment action.


- MX-IR-API-015: `POST /incidents/mx/{incident_id}/recovery` records recovery action.


- MX-IR-API-016: `POST /incidents/mx/{incident_id}/arco-aftercare` records affected-subject rights aftercare.


- MX-IR-API-017: `POST /incidents/mx/{incident_id}/blocking` creates blocked-data handling.


- MX-IR-API-018: `POST /incidents/mx/{incident_id}/public-statement/preflight` checks external statement evidence.


- MX-IR-API-019: `POST /incidents/mx/{incident_id}/forensic-evidence` records forensic evidence hashes.


- MX-IR-API-020: `GET /incidents/mx/{incident_id}/timeline` returns redacted timeline.


- MX-IR-API-021: `GET /incidents/mx/{incident_id}/audit-events` returns event ids and scrub status.


- MX-IR-API-022: `POST /incidents/mx/{incident_id}/close` closes only after required evidence exists.


- MX-IR-API-023: `GET /incidents/mx/failure-modes` returns failure-mode catalog.


- MX-IR-API-024: `POST /incidents/mx/audit-redaction-check` validates audit payload redaction.


- MX-IR-API-025: `POST /incidents/mx/{incident_id}/deny-action` records denied incident action.


## Audit Event Additions (per ADR-0263)

- MX-IR-AUDIT-001: `MxIncidentCreated` records incident id, tenant, detector, and detected timestamp.


- MX-IR-AUDIT-002: `MxIncidentPersonalDataScoped` records personal-data involvement and categories.


- MX-IR-AUDIT-003: `MxIncidentSensitiveDataScoped` records sensitive-data involvement without raw values.


- MX-IR-AUDIT-004: `MxIncidentSecurityReviewCompleted` records Art. 19 control-review result.


- MX-IR-AUDIT-005: `MxIncidentMaterialRightsAssessed` records Art. 20 rights-impact result.


- MX-IR-AUDIT-006: `MxIncidentNotificationRequired` records notification basis and affected class.


- MX-IR-AUDIT-007: `MxIncidentNotificationNotRequired` records no-notification reason.


- MX-IR-AUDIT-008: `MxIncidentNotificationPlanned` records channels, owner, and deadlines.


- MX-IR-AUDIT-009: `MxIncidentNotificationSent` records batch id and delivery evidence.


- MX-IR-AUDIT-010: `MxIncidentNotificationFailed` records failed channel and retry state.


- MX-IR-AUDIT-011: `MxIncidentSectorOverlayApplied` records sector code and trigger.


- MX-IR-AUDIT-012: `MxIncidentCnbvReviewCompleted` records financial cybersecurity review.


- MX-IR-AUDIT-013: `MxIncidentCnsfReviewCompleted` records insurance supervisory review.


- MX-IR-AUDIT-014: `MxIncidentCreReviewCompleted` records energy operational review.


- MX-IR-AUDIT-015: `MxIncidentTelecomReviewCompleted` records telecom and transition review.


- MX-IR-AUDIT-016: `MxIncidentAtdtIdentityReviewCompleted` records digital-identity segregation review.


- MX-IR-AUDIT-017: `MxIncidentVendorAccessApproved` records vendor, purpose, expiry, and redaction.


- MX-IR-AUDIT-018: `MxIncidentVendorAccessDenied` records missing basis or transfer failure.


- MX-IR-AUDIT-019: `MxIncidentContainmentApplied` records containment action.


- MX-IR-AUDIT-020: `MxIncidentRecoveryApplied` records recovery action.


- MX-IR-AUDIT-021: `MxIncidentForensicEvidenceStored` records evidence hash and storage location.


- MX-IR-AUDIT-022: `MxIncidentPublicStatementApproved` records approved statement ref.


- MX-IR-AUDIT-023: `MxIncidentClosed` records closure and residual-risk state.


- MX-IR-AUDIT-024: `MxIncidentActionDenied` records denied response action.


- MX-IR-AUDIT-025: `MxIncidentAuditPayloadRejected` records raw personal data in audit payload.


## Failure Modes

- MX-IR-FAIL-001: Incident intake missing personal-data scope cannot close.


- MX-IR-FAIL-002: Incident involving unknown data category must be treated as review-required.


- MX-IR-FAIL-003: Sensitive-data involvement cannot be downgraded by changing labels.


- MX-IR-FAIL-004: Art. 19 security review missing means containment remains incomplete.


- MX-IR-FAIL-005: Art. 20 rights-impact unknown means notification decision cannot close.


- MX-IR-FAIL-006: Notification not sent without a coded no-notification reason is invalid.


- MX-IR-FAIL-007: Subject notification without approved content and channel is invalid.


- MX-IR-FAIL-008: Vendor forensics access without transfer preflight is denied.


- MX-IR-FAIL-009: Foreign responder access without cross-border preflight is denied.


- MX-IR-FAIL-010: Incident report containing raw personal data in audit is rejected.


- MX-IR-FAIL-011: CNBV financial incident without sector overlay is incomplete.


- MX-IR-FAIL-012: CNBV ITF cybersecurity incident without third-party and electronic-means review is incomplete.


- MX-IR-FAIL-013: CNSF claim-file incident without insurance overlay is incomplete.


- MX-IR-FAIL-014: CRE metering or reliability incident without energy overlay is incomplete.


- MX-IR-FAIL-015: Telecom subscriber or traffic incident without telecom overlay is incomplete.


- MX-IR-FAIL-016: Telecom incident citing former IFT without transition mapping is incomplete.


- MX-IR-FAIL-017: ATDT digital identity incident commingled with private tenant identity graph is blocked.


- MX-IR-FAIL-018: Public statement before evidence review is blocked.


- MX-IR-FAIL-019: Containment action without audit evidence is not accepted.


- MX-IR-FAIL-020: Recovery action without verification evidence is not accepted.


- MX-IR-FAIL-021: Closing incident before ARCO aftercare review is blocked when subjects are affected.


- MX-IR-FAIL-022: Blocking expired incident data without responsibility review is blocked.


- MX-IR-FAIL-023: Erasing forensic evidence before retention expiry is blocked.


- MX-IR-FAIL-024: Applying another geography breach clock by default is rejected.


- MX-IR-FAIL-025: Incident automation touching non-Mexico packs violates slice boundary.


## Worked Examples

- MX-IR-EXAMPLE-001: A leaked customer email table triggers personal-data scope and Art. 20 rights-impact assessment.


- MX-IR-EXAMPLE-002: A leaked health questionnaire triggers sensitive-data scope and likely subject notification planning.


- MX-IR-EXAMPLE-003: A lost encrypted backup still requires security review and encryption evidence.


- MX-IR-EXAMPLE-004: Unauthorized admin access creates incident intake and minimum-access audit review.


- MX-IR-EXAMPLE-005: A compromised fintech API key triggers CNBV ITF cybersecurity overlay.


- MX-IR-EXAMPLE-006: A banking phishing incident triggers financial overlay and personal-data breach assessment.


- MX-IR-EXAMPLE-007: A claim adjuster portal exposure triggers CNSF insurance overlay.


- MX-IR-EXAMPLE-008: A reinsurance data-room leak triggers CNSF and cross-border review.


- MX-IR-EXAMPLE-009: A smart-meter breach triggers CRE energy overlay and identifiability assessment.


- MX-IR-EXAMPLE-010: An EV charging data leak triggers location and energy-user assessment.


- MX-IR-EXAMPLE-011: A mobile subscriber database leak triggers telecom overlay and transition authority mapping.


- MX-IR-EXAMPLE-012: A device IMEI disclosure triggers telecom device and subscriber-linkage assessment.


- MX-IR-EXAMPLE-013: A Llave MX credential incident is treated as public digital identity context, not ordinary tenant login only.


- MX-IR-EXAMPLE-014: A foreign incident-response firm cannot view raw Mexican personal data before transfer preflight.


- MX-IR-EXAMPLE-015: A crash dump containing names is blocked from telemetry export.


- MX-IR-EXAMPLE-016: A trace with CURP-like values is rejected by audit redaction check.


- MX-IR-EXAMPLE-017: A subject notification batch records channel, timing, and delivery evidence.


- MX-IR-EXAMPLE-018: A no-notification decision records why patrimonial or moral rights were not materially affected.


- MX-IR-EXAMPLE-019: A public incident statement cites verified facts and omits personal data.


- MX-IR-EXAMPLE-020: A post-incident ARCO request reuses the incident evidence scope without disclosing other subjects.


- MX-IR-EXAMPLE-021: A blocked incident record is retained only for responsibility determination.


- MX-IR-EXAMPLE-022: A recovery action rotating keys emits recovery evidence and verification result.


- MX-IR-EXAMPLE-023: A false positive incident closes only after personal-data scope and security review are recorded.


- MX-IR-EXAMPLE-024: A sector incident with conflicting duties follows strictest-sector gate.


- MX-IR-EXAMPLE-025: A Mexico incident does not automatically copy EU GDPR 72-hour language into the pack.


## Cross-References

- MX-IR-XREF-001: `README.md` defines pack activation and audit posture.


- MX-IR-XREF-002: `regulatory-coverage.md` maps LFPDPPP Arts. 19-20.


- MX-IR-XREF-003: `data-residency-and-cross-border.md` governs vendor and foreign responder access.


- MX-IR-XREF-004: `consent-and-data-subject-rights.md` governs ARCO aftercare after incidents.


- MX-IR-XREF-005: `sectoral-overlays.md` defines CNBV, CNSF, CRE, telecom, and ATDT escalation.


- MX-IR-XREF-006: ADR-0243 requires incident actions to be Cedar-gated.


- MX-IR-XREF-007: ADR-0244 requires tenant and sub-scope on every incident event.


- MX-IR-XREF-008: ADR-0251 binds incident controls to compliance-pack activation.


- MX-IR-XREF-009: ADR-0263 governs structured, scrubbed incident audit emission.


- MX-IR-XREF-010: ADR-0297 governs abuse-defense incidents that involve bot, spoof, scrape, or platform abuse.


- MX-IR-XREF-011: LFPDPPP Art. 19 is the security-measures anchor.


- MX-IR-XREF-012: LFPDPPP Art. 20 is the affected-subject notification anchor.


- MX-IR-XREF-013: LFPDPPP Arts. 36-37 govern incident responder transfers.


- MX-IR-XREF-014: CNBV fintech cybersecurity provisions inform financial incident overlays.


- MX-IR-XREF-015: CNBV privacy sources inform financial data-subject impact.


- MX-IR-XREF-016: CNSF privacy inventories inform insurance incident scoping.


- MX-IR-XREF-017: CUSF materials inform insurance supervisory context.


- MX-IR-XREF-018: CRE materials inform energy user and reliability context.


- MX-IR-XREF-019: IFT and CRT materials inform telecom user incident handling.


- MX-IR-XREF-020: ATDT materials inform telecom transition and digital identity context.


- MX-IR-XREF-021: LMTR Arts. 4-5 inform telecom infrastructure incident scope.


- MX-IR-XREF-022: Subject notification content must align with privacy notice contact routes.


- MX-IR-XREF-023: Forensic exports must run redaction and transfer preflight.


- MX-IR-XREF-024: Public statements require evidence review and legal approval.


- MX-IR-XREF-025: Incident closure requires scope, decision, notification, containment, recovery, and audit evidence.

