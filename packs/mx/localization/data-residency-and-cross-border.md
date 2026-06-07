---
doc_class: LocalizationPack
pack_id: MX-PACK-1
doc_id: MX-PACK-1-DATA-RESIDENCY-CROSS-BORDER
title: Mexico Data Residency and Cross-Border Transfers
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0243
  - ADR-0244
  - ADR-0249
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LFPDPPP.pdf
  - https://www.diputados.gob.mx/LeyesBiblio/ref/lfpdppp.htm
  - https://www.cnbv.gob.mx/ProteccionDatos/Paginas/default.aspx
  - https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20Instituciones%20de%20Tecnolog%C3%ADa%20Financiera.pdf
  - https://www.cnsf.gob.mx/Transparencia/Paginas/ProteccionDatosPersonales.aspx
  - https://www.gob.mx/cre/que-hacemos
  - https://www.ift.org.mx/
  - https://www.gob.mx/atdt
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LMTR.pdf
---

# Mexico Data Residency and Cross-Border Transfers

This document defines Mexico placement, access, transfer, remote-support, replication, telemetry, and regulator-reporting controls.
Mexico privacy law does not create a generic data-localization mandate for all private-sector personal data.
This pack still supports Mexico-resident cells because tenants, regulators, contracts, and sector overlays may require local placement.
Residency answers where data is stored or processed.
Transfer answers whether personal data is communicated or made available to another person or jurisdiction.
Remote access can be a transfer even when storage remains in Mexico.
Audit and telemetry exports can be transfers when they contain personal data.

## Authority Citations

- MX-RES-AUTH-001: LFPDPPP Art. 3(XVIII) defines transfer as communication of personal data to a person other than the controller or processor.


- MX-RES-AUTH-002: LFPDPPP Art. 3(III) defines blocking as retention after purpose completion solely for responsibility determination.


- MX-RES-AUTH-003: LFPDPPP Art. 3(XI) defines the law itself, so every transfer row must identify whether LFPDPPP applies.


- MX-RES-AUTH-004: LFPDPPP Art. 6 purpose, proportionality, and responsibility principles shape transfer minimization.


- MX-RES-AUTH-005: LFPDPPP Art. 8 requires consent unless a statutory exception covers the treatment.


- MX-RES-AUTH-006: LFPDPPP Art. 10 exceptions may support processing but do not erase notice and security duties.


- MX-RES-AUTH-007: LFPDPPP Art. 14 requires the subject to be informed about the processing of personal data.


- MX-RES-AUTH-008: LFPDPPP Art. 16 requires privacy notices to include transfer information and mechanisms for rights exercise.


- MX-RES-AUTH-009: LFPDPPP Art. 19 requires security measures for data under controller responsibility, including cross-border handling.


- MX-RES-AUTH-010: LFPDPPP Art. 20 requires affected-holder notification when security vulnerabilities materially affect rights.


- MX-RES-AUTH-011: LFPDPPP Art. 36 requires the controller to communicate privacy notice terms to transfer recipients.


- MX-RES-AUTH-012: LFPDPPP Art. 36 requires the recipient to process data according to the privacy notice accepted by the subject.


- MX-RES-AUTH-013: LFPDPPP Art. 37 allows some transfers without consent, including between affiliates under common processes or policies.


- MX-RES-AUTH-014: LFPDPPP Art. 37 allows some transfers without consent where legally required or needed for public interest administration of justice.


- MX-RES-AUTH-015: LFPDPPP Art. 37 allows some transfers without consent for medical diagnosis, healthcare, treatment, or sanitary-services management.


- MX-RES-AUTH-016: LFPDPPP Art. 37 allows some transfers without consent to fulfill a contract in the interest of the subject.


- MX-RES-AUTH-017: LFPDPPP Art. 37 allows some transfers without consent for legal relationship maintenance or fulfillment between controller and subject.


- MX-RES-AUTH-018: LFPDPPP Arts. 38-39 create the current federal authority path for verification and sanctions after the 2025 replacement.


- MX-RES-AUTH-019: The Chamber reference page records the 2025 and 2025-11 reform history for current transfer-law text.


- MX-RES-AUTH-020: CNBV official sources require sector review when financial data, cybersecurity evidence, outsourcing, or regulated reporting crosses borders.


- MX-RES-AUTH-021: CNBV fintech cybersecurity provisions require ITF information-security, electronic-means, and third-party service controls.


- MX-RES-AUTH-022: CNSF official privacy inventories require sector review when insurance supervisory, agent, claims, or actuarial records are transferred.


- MX-RES-AUTH-023: CRE public authority sources require energy-sector review when permit, meter, user, or reliability records leave a regulated context.


- MX-RES-AUTH-024: LMTR Arts. 4-5 require telecom records to preserve federal telecom context for networks, spectrum, satellite, and user services.


- MX-RES-AUTH-025: ATDT and IFT/CRT sources require transition tagging when telecom or digital-government identity records move across systems.


## Activated Cedar Policies

- MX-RES-CEDAR-001: `mx_residency_profile_required` denies Mexico-regulated processing without a placement profile.


- MX-RES-CEDAR-002: `mx_residency_not_transfer_safe_harbor` blocks claims that Mexico hosting alone makes transfers lawful.


- MX-RES-CEDAR-003: `mx_transfer_definition_gate` classifies recipient disclosure under LFPDPPP Art. 3(XVIII).


- MX-RES-CEDAR-004: `mx_remote_access_transfer_gate` treats non-Mexico personnel access as transfer review when personal data is visible.


- MX-RES-CEDAR-005: `mx_support_access_minimization_gate` requires just-in-time support, purpose, ticket, and redaction before support access.


- MX-RES-CEDAR-006: `mx_replication_transfer_gate` requires destination, controller, processor, and recipient classification before replication.


- MX-RES-CEDAR-007: `mx_backup_transfer_gate` treats backup storage outside Mexico as a transfer preflight.


- MX-RES-CEDAR-008: `mx_log_export_transfer_gate` treats logs containing identifiers as transfer-relevant.


- MX-RES-CEDAR-009: `mx_trace_export_transfer_gate` treats traces containing tenant, subject, or device identifiers as transfer-relevant.


- MX-RES-CEDAR-010: `mx_metric_export_transfer_gate` treats high-cardinality identifiers in metrics as transfer-relevant.


- MX-RES-CEDAR-011: `mx_recipient_notice_acceptance_gate` requires recipient obligations tied to LFPDPPP Art. 36.


- MX-RES-CEDAR-012: `mx_transfer_consent_gate` requires consent unless an Art. 37 transfer exception is mapped.


- MX-RES-CEDAR-013: `mx_affiliate_transfer_gate` verifies common-process or policy evidence for affiliate transfers.


- MX-RES-CEDAR-014: `mx_legal_obligation_transfer_gate` verifies legal-mandate evidence before no-consent transfer.


- MX-RES-CEDAR-015: `mx_contract_transfer_gate` verifies contract-in-interest-of-subject evidence before no-consent transfer.


- MX-RES-CEDAR-016: `mx_medical_transfer_gate` verifies medical or sanitary-services purpose before no-consent transfer.


- MX-RES-CEDAR-017: `mx_judicial_transfer_gate` verifies judicial or administrative resolution evidence.


- MX-RES-CEDAR-018: `mx_public_interest_transfer_gate` verifies public-interest administration-of-justice basis.


- MX-RES-CEDAR-019: `mx_cnbv_cross_border_gate` requires financial-sector cross-border review.


- MX-RES-CEDAR-020: `mx_cnsf_cross_border_gate` requires insurance-sector cross-border review.


- MX-RES-CEDAR-021: `mx_cre_cross_border_gate` requires energy-sector cross-border review.


- MX-RES-CEDAR-022: `mx_telecom_cross_border_gate` requires subscriber, traffic, portability, or device-data transfer review.


- MX-RES-CEDAR-023: `mx_atdt_identity_cross_border_gate` requires digital-government identity segregation and review.


- MX-RES-CEDAR-024: `mx_transfer_audit_scrub_gate` blocks transfer audit events that include raw personal data.


- MX-RES-CEDAR-025: `mx_transfer_strictest_sector_gate` applies stricter sector controls over generic transfer exceptions.


## Data Model Deltas

- MX-RES-DATA-001: `mx_residency_profile.primary_region` records Mexico, non-Mexico, hybrid, or unknown placement.


- MX-RES-DATA-002: `mx_residency_profile.processing_regions` records all regions where active processing occurs.


- MX-RES-DATA-003: `mx_residency_profile.backup_regions` records backup and disaster-recovery locations.


- MX-RES-DATA-004: `mx_residency_profile.telemetry_regions` records log, metric, trace, crash, and analytic destinations.


- MX-RES-DATA-005: `mx_transfer_record.transfer_id` identifies every disclosure or remote access event.


- MX-RES-DATA-006: `mx_transfer_record.source_region` records where the personal data originates.


- MX-RES-DATA-007: `mx_transfer_record.destination_region` records recipient jurisdiction or access location.


- MX-RES-DATA-008: `mx_transfer_record.recipient_type` records affiliate, processor, controller, regulator, vendor, public authority, or support staff.


- MX-RES-DATA-009: `mx_transfer_record.article_basis` stores `LFPDPPP-ART-36`, `LFPDPPP-ART-37`, or legal-review-required.


- MX-RES-DATA-010: `mx_transfer_record.notice_acceptance_ref` stores recipient acceptance of notice limits.


- MX-RES-DATA-011: `mx_transfer_record.subject_consent_ref` stores consent evidence where no exception applies.


- MX-RES-DATA-012: `mx_transfer_record.exception_code` stores the Art. 37 exception class.


- MX-RES-DATA-013: `mx_transfer_record.sector_overlay_codes` stores CNBV, CNSF, CRE, telecom, ATDT, or tenant-contract overlays.


- MX-RES-DATA-014: `mx_transfer_record.support_ticket_ref` stores support justification for remote access.


- MX-RES-DATA-015: `mx_transfer_record.redaction_profile` stores fields hidden from recipient or operator.


- MX-RES-DATA-016: `mx_transfer_record.encryption_profile` stores cryptographic control evidence.


- MX-RES-DATA-017: `mx_transfer_record.retention_profile` stores retention and blocking obligations.


- MX-RES-DATA-018: `mx_transfer_record.revocation_status` stores revoked, active, expired, or legal-hold states.


- MX-RES-DATA-019: `mx_blocking_record.blocked_at` stores the date data moved into LFPDPPP blocking.


- MX-RES-DATA-020: `mx_blocking_record.reason` stores responsibility, contractual, statutory, litigation, or regulator reason.


- MX-RES-DATA-021: `mx_telemetry_export_record.signal_type` stores log, metric, trace, exemplar, or audit export.


- MX-RES-DATA-022: `mx_telemetry_export_record.personal_data_risk` stores none, low, medium, high, or unknown.


- MX-RES-DATA-023: `mx_regulator_transfer_record.authority_code` stores CNBV, CNSF, CRE, CRT, ATDT, SICT, Secretaria, or legacy INAI.


- MX-RES-DATA-024: `mx_transfer_review.decision` stores allow, deny, legal-review, sector-review, or redaction-required.


- MX-RES-DATA-025: `mx_transfer_review.evidence_refs` stores official URLs, article ids, contracts, and notices.


## API Contract Deltas

- MX-RES-API-001: `POST /privacy/mx/residency/profile` creates or updates Mexico placement state.


- MX-RES-API-002: `GET /privacy/mx/residency/profile/{tenant_id}` returns active processing, backup, telemetry, and support locations.


- MX-RES-API-003: `POST /privacy/mx/transfers/preflight` evaluates a proposed transfer under Arts. 36-37 and sector overlays.


- MX-RES-API-004: `POST /privacy/mx/transfers/remote-access/preflight` evaluates non-Mexico support or admin access.


- MX-RES-API-005: `POST /privacy/mx/transfers/replication/preflight` evaluates data replication and backup movement.


- MX-RES-API-006: `POST /privacy/mx/transfers/telemetry/preflight` evaluates log, metric, trace, crash, and exemplar exports.


- MX-RES-API-007: `POST /privacy/mx/transfers/regulator/preflight` evaluates regulator or public authority disclosure.


- MX-RES-API-008: `POST /privacy/mx/transfers/recipient-acceptance` records recipient acceptance of privacy-notice limits.


- MX-RES-API-009: `POST /privacy/mx/transfers/consent` records subject consent for transfer when required.


- MX-RES-API-010: `POST /privacy/mx/transfers/exception` records an Art. 37 exception with evidence.


- MX-RES-API-011: `GET /privacy/mx/transfers/{transfer_id}` returns transfer status and evidence refs.


- MX-RES-API-012: `POST /privacy/mx/transfers/{transfer_id}/revoke` revokes ongoing transfer permission.


- MX-RES-API-013: `POST /privacy/mx/transfers/{transfer_id}/legal-hold` preserves transfer records under legal hold.


- MX-RES-API-014: `POST /privacy/mx/blocking` creates a blocked-data record after purpose completion.


- MX-RES-API-015: `GET /privacy/mx/blocking/{record_id}` returns blocked-data purpose, expiry, and permitted access.


- MX-RES-API-016: `POST /privacy/mx/telemetry/redaction-check` validates telemetry redaction before export.


- MX-RES-API-017: `POST /privacy/mx/support-access/grant` creates time-bound support access after transfer review.


- MX-RES-API-018: `POST /privacy/mx/support-access/revoke` revokes support access and emits audit evidence.


- MX-RES-API-019: `GET /privacy/mx/residency/sector-overlays` returns cross-border overlays by sector.


- MX-RES-API-020: `POST /privacy/mx/residency/sector-overlays/evaluate` applies strictest sector rule.


- MX-RES-API-021: `POST /privacy/mx/residency/authority-transition` records authority context for telecom or INAI-era transfers.


- MX-RES-API-022: `GET /privacy/mx/residency/export-map` returns redacted export maps for compliance review.


- MX-RES-API-023: `POST /privacy/mx/residency/export-map` creates an export map without personal-data payload.


- MX-RES-API-024: `POST /privacy/mx/residency/deny` records denied transfer and reason.


- MX-RES-API-025: `GET /privacy/mx/residency/failure-modes` returns residency and transfer failure-mode catalog.


## Audit Event Additions (per ADR-0263)

- MX-RES-AUDIT-001: `MxResidencyProfileCreated` records tenant, primary region, processing regions, and snapshot.


- MX-RES-AUDIT-002: `MxResidencyProfileChanged` records old and new placement values without data payload.


- MX-RES-AUDIT-003: `MxTransferPreflightRequested` records actor, recipient, destination, and purpose.


- MX-RES-AUDIT-004: `MxTransferPreflightAllowed` records basis, notice acceptance, and sector overlays.


- MX-RES-AUDIT-005: `MxTransferPreflightDenied` records missing basis, stale notice, or sector blocker.


- MX-RES-AUDIT-006: `MxTransferLegalReviewRequired` records unresolved legal question and authority refs.


- MX-RES-AUDIT-007: `MxTransferRecipientNoticeBound` records recipient and notice acceptance digest.


- MX-RES-AUDIT-008: `MxTransferConsentCaptured` records consent ref and transfer purpose.


- MX-RES-AUDIT-009: `MxTransferExceptionApplied` records Art. 37 exception code.


- MX-RES-AUDIT-010: `MxRemoteAccessGranted` records support ticket, region, role, expiry, and redaction profile.


- MX-RES-AUDIT-011: `MxRemoteAccessRevoked` records revocation actor and access window.


- MX-RES-AUDIT-012: `MxReplicationTransferAllowed` records replication destination and encryption profile.


- MX-RES-AUDIT-013: `MxBackupTransferDenied` records destination and unmet basis.


- MX-RES-AUDIT-014: `MxTelemetryExportAllowed` records signal type and redaction profile.


- MX-RES-AUDIT-015: `MxTelemetryExportRejected` records raw identifier or sensitive-field rejection.


- MX-RES-AUDIT-016: `MxRegulatorDisclosureReviewed` records authority code and disclosure basis.


- MX-RES-AUDIT-017: `MxCnbvCrossBorderReviewCompleted` records financial-sector decision.


- MX-RES-AUDIT-018: `MxCnsfCrossBorderReviewCompleted` records insurance-sector decision.


- MX-RES-AUDIT-019: `MxCreCrossBorderReviewCompleted` records energy-sector decision.


- MX-RES-AUDIT-020: `MxTelecomCrossBorderReviewCompleted` records telecom-sector decision.


- MX-RES-AUDIT-021: `MxAtdtIdentityTransferBlocked` records digital-government identity segregation failure.


- MX-RES-AUDIT-022: `MxBlockedDataEntered` records blocked-data reason and retention horizon.


- MX-RES-AUDIT-023: `MxBlockedDataReleased` records expiry and deletion or continued-hold decision.


- MX-RES-AUDIT-024: `MxTransferExportMapGenerated` records redacted export-map metadata.


- MX-RES-AUDIT-025: `MxTransferAuditPayloadRejected` records ADR-0263 scrub failure.


## Failure Modes

- MX-RES-FAIL-001: Mexico hosting is treated as a control, not as automatic transfer permission.


- MX-RES-FAIL-002: Remote access without destination and role must be denied.


- MX-RES-FAIL-003: Remote access without support ticket must be denied.


- MX-RES-FAIL-004: Transfer without recipient notice acceptance must be denied under Art. 36.


- MX-RES-FAIL-005: Transfer without consent or Art. 37 exception must be denied.


- MX-RES-FAIL-006: Affiliate transfer without common-process or policy evidence must be denied.


- MX-RES-FAIL-007: Medical transfer without healthcare or sanitary-service purpose must be denied.


- MX-RES-FAIL-008: Contract transfer without subject-interest evidence must be denied.


- MX-RES-FAIL-009: Legal-obligation transfer without authority evidence must be denied.


- MX-RES-FAIL-010: Judicial transfer without resolution evidence must be denied.


- MX-RES-FAIL-011: Replication to unknown region must be denied.


- MX-RES-FAIL-012: Backup to unknown subprocessor must be denied.


- MX-RES-FAIL-013: Telemetry export containing direct identifiers must be rejected unless redacted.


- MX-RES-FAIL-014: Trace export containing sensitive data must be rejected.


- MX-RES-FAIL-015: Metric export with high-cardinality subject identifiers must be rejected.


- MX-RES-FAIL-016: CNBV-regulated data transfer without financial-sector review must be denied.


- MX-RES-FAIL-017: CNSF-regulated data transfer without insurance-sector review must be denied.


- MX-RES-FAIL-018: CRE-regulated data transfer without energy-sector review must be denied.


- MX-RES-FAIL-019: Telecom data transfer without transition authority mapping must be denied.


- MX-RES-FAIL-020: ATDT digital identity transfer into private tenant graph must be denied unless specifically approved.


- MX-RES-FAIL-021: Regulator disclosure without authority code and article refs must be denied.


- MX-RES-FAIL-022: Blocked data access outside responsibility determination must be denied.


- MX-RES-FAIL-023: Transfer audit events containing raw personal data must be rejected.


- MX-RES-FAIL-024: Transfer based only on tenant admin approval must be denied.


- MX-RES-FAIL-025: Transfer mapping that touches another geography pack must be rejected for this slice.


## Worked Examples

- MX-RES-EXAMPLE-001: A Mexico-hosted CRM record viewed by a United States support agent still requires transfer preflight.


- MX-RES-EXAMPLE-002: A Mexico user database replicated to a non-Mexico disaster-recovery region requires recipient and destination review.


- MX-RES-EXAMPLE-003: A Mexico privacy notice that omits transfers fails an export to a foreign analytics vendor.


- MX-RES-EXAMPLE-004: A processor inside the same corporate group can rely on Art. 37 only if common policy evidence exists.


- MX-RES-EXAMPLE-005: A medical emergency support transfer can use a health exception only for necessary data.


- MX-RES-EXAMPLE-006: A court order export must cite the resolution and preserve minimum necessary disclosure.


- MX-RES-EXAMPLE-007: A CNBV fintech vendor export requires cybersecurity and third-party service review.


- MX-RES-EXAMPLE-008: A bank fraud-investigation export requires CNBV overlay and LFPDPPP transfer basis.


- MX-RES-EXAMPLE-009: An insurance claim file sent to an adjuster requires CNSF context and privacy notice compatibility.


- MX-RES-EXAMPLE-010: A reinsurance data room requires insurance-sector and cross-border review.


- MX-RES-EXAMPLE-011: A CRE electricity user record sent to a grid-services vendor requires energy context and transfer basis.


- MX-RES-EXAMPLE-012: A meter telemetry export is personal data when it can identify a household or facility operator.


- MX-RES-EXAMPLE-013: A telecom subscriber export to a network vendor requires telecom overlay and transition authority mapping.


- MX-RES-EXAMPLE-014: A device blacklist or IMEI workflow requires device identifier and subscriber linkage review.


- MX-RES-EXAMPLE-015: A trace containing CURP-like identity values is rejected by the telemetry redaction check.


- MX-RES-EXAMPLE-016: A metric labeled by subject email is rejected even if the metric value is aggregate.


- MX-RES-EXAMPLE-017: A support session using screen share must restrict visibility to the approved ticket scope.


- MX-RES-EXAMPLE-018: A blocked record cannot be used for marketing analytics after the original purpose has ended.


- MX-RES-EXAMPLE-019: A regulator disclosure to a current authority must not cite legacy INAI alone.


- MX-RES-EXAMPLE-020: A telecom disclosure citing the former IFT must identify whether CRT, ATDT, or LMTR now governs the action.


- MX-RES-EXAMPLE-021: An export map shared with counsel contains fields, purposes, and authorities but not personal-data payloads.


- MX-RES-EXAMPLE-022: A tenant moving from Mexico-only hosting to hybrid hosting creates a residency profile change event.


- MX-RES-EXAMPLE-023: A SaaS backup vendor change triggers transfer preflight even if application traffic stays in Mexico.


- MX-RES-EXAMPLE-024: A deactivated pack still preserves transfer records until retention and legal-hold duties end.


- MX-RES-EXAMPLE-025: A cross-border transfer denied for missing Art. 37 exception cannot be approved by changing the UI label.


## Cross-References

- MX-RES-XREF-001: `README.md` identifies MX-PACK-1 activation and no-touch scope.


- MX-RES-XREF-002: `regulatory-coverage.md` maps LFPDPPP Arts. 36-37 to control names.


- MX-RES-XREF-003: `consent-and-data-subject-rights.md` supplies consent dependencies for transfers.


- MX-RES-XREF-004: `breach-notification-and-incident-response.md` supplies incident handling for transfer exposures.


- MX-RES-XREF-005: `sectoral-overlays.md` supplies CNBV, CNSF, CRE, telecom, and ATDT sector checks.


- MX-RES-XREF-006: ADR-0243 binds transfer decisions to Cedar gates.


- MX-RES-XREF-007: ADR-0244 binds residency decisions to tenant and sub-scope context.


- MX-RES-XREF-008: ADR-0249 is relevant for cross-region replication and residency discipline.


- MX-RES-XREF-009: ADR-0251 binds residency controls into compliance-pack activation.


- MX-RES-XREF-010: ADR-0263 binds transfer audit events to structured, scrubbed emission.


- MX-RES-XREF-011: LFPDPPP Art. 3(XVIII) is the transfer-definition anchor.


- MX-RES-XREF-012: LFPDPPP Art. 16 is the privacy-notice transfer-language anchor.


- MX-RES-XREF-013: LFPDPPP Art. 36 is the recipient notice acceptance anchor.


- MX-RES-XREF-014: LFPDPPP Art. 37 is the no-consent transfer exception anchor.


- MX-RES-XREF-015: LFPDPPP Art. 19 is the transfer security-measures anchor.


- MX-RES-XREF-016: LFPDPPP Art. 20 is the transfer exposure notification anchor.


- MX-RES-XREF-017: CNBV sources are required for financial transfer overlays.


- MX-RES-XREF-018: CNSF sources are required for insurance transfer overlays.


- MX-RES-XREF-019: CRE sources are required for energy transfer overlays.


- MX-RES-XREF-020: IFT and CRT sources are required for telecom transition mapping.


- MX-RES-XREF-021: ATDT sources are required for digital-government and telecom transition context.


- MX-RES-XREF-022: LMTR Arts. 4-5 are required for telecom infrastructure and federal-jurisdiction context.


- MX-RES-XREF-023: Export maps must use redacted evidence only.


- MX-RES-XREF-024: Remote support must use short-lived access and transfer preflight.


- MX-RES-XREF-025: Residency changes must emit audit events before new placement is used.

