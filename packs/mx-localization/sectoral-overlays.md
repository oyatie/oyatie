---
doc_class: LocalizationPack
pack_id: MX-PACK-1
doc_id: MX-PACK-1-SECTORAL-OVERLAYS
title: Mexico Sectoral Overlays
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
  - https://www.cnbv.gob.mx/ProteccionDatos/Paginas/default.aspx
  - https://www.cnbv.gob.mx/Normatividad/Disposiciones%20de%20car%C3%A1cter%20general%20aplicables%20a%20las%20Instituciones%20de%20Tecnolog%C3%ADa%20Financiera.pdf
  - https://www.gob.mx/cnbv/acciones-y-programas/disposiciones-legales-instituciones-de-credito
  - https://www.cnsf.gob.mx/Transparencia/Paginas/ProteccionDatosPersonales.aspx
  - https://www.gob.mx/cnsf/documentos/circular-unica-de-seguros-y-fianzas
  - https://www.gob.mx/cre/que-hacemos
  - https://www.gob.mx/cre/acciones-y-programas/micrositio-de-permisos-en-materia-de-generacion-de-energia-electrica
  - https://www.ift.org.mx/
  - https://portal.crt.gob.mx/Home/Index?xsndjr=0859d952-dabc-4d42-ad00-bb77a738eb1f
  - https://www.gob.mx/atdt
  - https://www.diputados.gob.mx/LeyesBiblio/pdf/LMTR.pdf
---

# Mexico Sectoral Overlays

This document defines sector-specific overlays layered on top of LFPDPPP.
The overlays do not replace private-sector privacy duties.
The overlays make duties stricter where banking, fintech, insurance, bonding, energy, telecommunications, or digital-government identity contexts apply.
The telecom section explicitly preserves user-requested IFT coverage while naming the 2025-2026 CRT and ATDT transition context.
Runtime bundles must treat unknown sector classification as review-required.

## Authority Citations

- MX-SEC-AUTH-001: LFPDPPP Art. 6 remains the baseline for all sector overlays.


- MX-SEC-AUTH-002: LFPDPPP Art. 9 applies to sensitive data in banking, insurance, energy health/safety, telecom accessibility, and identity contexts.


- MX-SEC-AUTH-003: LFPDPPP Art. 16 applies to sector privacy notices and transfer language.


- MX-SEC-AUTH-004: LFPDPPP Art. 19 applies security duties to every sector processing record.


- MX-SEC-AUTH-005: LFPDPPP Art. 20 applies breach notification analysis to sector incidents.


- MX-SEC-AUTH-006: LFPDPPP Arts. 22-28 apply ARCO rights unless a stricter sector or legal basis controls.


- MX-SEC-AUTH-007: LFPDPPP Arts. 36-37 apply transfer controls before sector data leaves a controller, processor, or region.


- MX-SEC-AUTH-008: CNBV data-protection source is an official privacy context for financial-supervisor personal-data handling.


- MX-SEC-AUTH-009: CNBV legal-dispositions page identifies institutions-of-credit regulation sources for banking overlays.


- MX-SEC-AUTH-010: CNBV fintech cybersecurity provisions define information-security, electronic-means, and third-party controls for ITF institutions.


- MX-SEC-AUTH-011: CNSF data-protection source lists privacy inventories for supervisory insurance and bonding processes.


- MX-SEC-AUTH-012: CNSF CUSF source is the official insurance and bonding circular reference for supervised operations.


- MX-SEC-AUTH-013: CRE authority source describes energy regulator responsibilities for electricity, hydrocarbons, gas, permits, users, and reliability.


- MX-SEC-AUTH-014: CRE generation-permit source illustrates regulated electricity permit workflows and applicant records.


- MX-SEC-AUTH-015: IFT official site preserves telecom user, privacy, accessibility, and public-resource materials.


- MX-SEC-AUTH-016: CRT portal preserves the IFT acervo, public tools, concession registry, normativity, and electronic-window transition surface.


- MX-SEC-AUTH-017: ATDT official site establishes the transformation-digital and telecommunications agency context.


- MX-SEC-AUTH-018: ATDT telecom law communication identifies transition of former IFT functions across the new institutional model.


- MX-SEC-AUTH-019: LMTR Art. 4 defines telecom public networks, spectrum, broadcasting stations, equipment, and satellite systems as general communication routes.


- MX-SEC-AUTH-020: LMTR Art. 5 establishes federal jurisdiction and public-interest status for telecom infrastructure.


- MX-SEC-AUTH-021: LMTR Art. 6 and related definitions must be checked before runtime telecom classification, even though this doc uses Arts. 4-5 as stable anchors.


- MX-SEC-AUTH-022: ATDT digital-government sources such as Llave MX create identity-boundary concerns when government identity data touches private workflows.


- MX-SEC-AUTH-023: CNBV fintech provisions require incident, outsourcing, security-officer, and information-security evidence before fintech production activation.


- MX-SEC-AUTH-024: CNSF treatment inventories require record-type classification for policyholders, claims, agents, adjusters, and regulatory reporting.


- MX-SEC-AUTH-025: CRE user and permit workflows require care because regulated activity records can become personal data when linked to identifiable persons.


## Activated Cedar Policies

- MX-SEC-CEDAR-001: `mx_sector_profile_required` denies sector workflows without sector classification.


- MX-SEC-CEDAR-002: `mx_sector_strictest_duty` chooses the stricter of LFPDPPP, sector, contract, and tenant policy.


- MX-SEC-CEDAR-003: `mx_cnbv_bank_profile_gate` applies banking overlay to deposit, credit, payment, identity, fraud, and outsourcing records.


- MX-SEC-CEDAR-004: `mx_cnbv_fintech_profile_gate` applies fintech overlay to ITF platform, investor, applicant, transaction, API, and cybersecurity records.


- MX-SEC-CEDAR-005: `mx_cnbv_security_officer_gate` requires responsible security governance markers where fintech cybersecurity provisions apply.


- MX-SEC-CEDAR-006: `mx_cnbv_third_party_gate` requires third-party technology and outsourcing evidence.


- MX-SEC-CEDAR-007: `mx_cnbv_electronic_means_gate` requires electronic-means security evidence for regulated financial workflows.


- MX-SEC-CEDAR-008: `mx_cnsf_policyholder_gate` applies insurance overlay to policyholder and beneficiary records.


- MX-SEC-CEDAR-009: `mx_cnsf_claim_gate` applies insurance overlay to claim, medical, adjuster, and fraud records.


- MX-SEC-CEDAR-010: `mx_cnsf_agent_gate` applies insurance overlay to agent, broker, adjuster, and certification records.


- MX-SEC-CEDAR-011: `mx_cnsf_regulatory_report_gate` applies supervisory-reporting controls.


- MX-SEC-CEDAR-012: `mx_cre_permit_gate` applies energy overlay to permit application, modification, transfer, and compliance records.


- MX-SEC-CEDAR-013: `mx_cre_metering_gate` applies energy overlay to meter and usage records linked to identifiable users.


- MX-SEC-CEDAR-014: `mx_cre_reliability_gate` applies energy overlay to reliability, dispatch, outage, and infrastructure records.


- MX-SEC-CEDAR-015: `mx_cre_hydrocarbon_gate` applies energy overlay to hydrocarbons, gas, petroliferos, petrochemicals, and commercialization records.


- MX-SEC-CEDAR-016: `mx_telecom_subscriber_gate` applies telecom overlay to subscriber and account records.


- MX-SEC-CEDAR-017: `mx_telecom_traffic_gate` applies telecom overlay to traffic, usage, network, and service-quality records.


- MX-SEC-CEDAR-018: `mx_telecom_device_gate` applies telecom overlay to IMEI, terminal equipment, homologation, and device identifiers.


- MX-SEC-CEDAR-019: `mx_telecom_portability_gate` applies telecom overlay to number portability and user-rights workflows.


- MX-SEC-CEDAR-020: `mx_telecom_accessibility_gate` applies telecom overlay to disability accessibility workflows.


- MX-SEC-CEDAR-021: `mx_ift_crt_transition_gate` requires historical IFT, current CRT, and ATDT context tagging.


- MX-SEC-CEDAR-022: `mx_atdt_digital_identity_gate` segregates Llave MX and government digital identity records.


- MX-SEC-CEDAR-023: `mx_sector_transfer_gate` requires sector review before transfer or remote access.


- MX-SEC-CEDAR-024: `mx_sector_incident_gate` requires sector incident overlay before breach closure.


- MX-SEC-CEDAR-025: `mx_sector_audit_scrub_gate` rejects raw sector data in audit events.


## Data Model Deltas

- MX-SEC-DATA-001: `mx_sector_profile.sector_codes` stores CNBV_BANK, CNBV_FINTECH, CNSF, CRE, TELECOM, ATDT_IDENTITY, or NONE.


- MX-SEC-DATA-002: `mx_sector_profile.license_or_registration_ref` stores tenant-supplied license, registration, or legal-review reference.


- MX-SEC-DATA-003: `mx_cnbv_bank_record.kind` stores deposit, credit, payment, fraud, customer, credential, AML-adjacent, or outsourcing class.


- MX-SEC-DATA-004: `mx_cnbv_fintech_record.kind` stores applicant, investor, project, platform, transaction, electronic means, or cybersecurity class.


- MX-SEC-DATA-005: `mx_cnbv_security_record.control_area` stores access, encryption, incident, continuity, third party, audit, or governance area.


- MX-SEC-DATA-006: `mx_cnbv_third_party_record.provider_ref` stores technology or service provider identity.


- MX-SEC-DATA-007: `mx_cnsf_record.kind` stores policyholder, insured, beneficiary, claim, agent, adjuster, actuary, reserve, or report.


- MX-SEC-DATA-008: `mx_cnsf_record.supervisory_purpose` stores reporting, inspection, authorization, solvency, claims, or complaint context.


- MX-SEC-DATA-009: `mx_cnsf_record.sensitive_claim_flag` marks medical, disability, biometric, or other sensitive claim data.


- MX-SEC-DATA-010: `mx_cre_record.kind` stores permit, applicant, meter, usage, outage, reliability, user, facility, or tariff class.


- MX-SEC-DATA-011: `mx_cre_record.energy_domain` stores electricity, hydrocarbons, gas, petroliferos, petrochemicals, bioenergy, or electromobility.


- MX-SEC-DATA-012: `mx_cre_record.identifiability` stores direct, indirect, aggregated, anonymized, or unknown.


- MX-SEC-DATA-013: `mx_telecom_record.kind` stores subscriber, device, traffic, portability, accessibility, complaint, network, or satellite class.


- MX-SEC-DATA-014: `mx_telecom_record.authority_context` stores IFT, CRT, ATDT, SICT, LMTR, or unknown.


- MX-SEC-DATA-015: `mx_telecom_record.user_right_context` stores privacy, portability, accessibility, emergency, quality, billing, or complaint.


- MX-SEC-DATA-016: `mx_device_identifier.kind` stores IMEI, SIM, terminal, homologation, network equipment, or other device id.


- MX-SEC-DATA-017: `mx_atdt_identity_record.kind` stores Llave MX, expediente digital, government credential, or public-service account class.


- MX-SEC-DATA-018: `mx_sector_transfer_review.sector_code` stores the sector responsible for stricter transfer check.


- MX-SEC-DATA-019: `mx_sector_incident_review.sector_code` stores the sector responsible for incident escalation.


- MX-SEC-DATA-020: `mx_sector_rights_review.sector_code` stores the sector responsible for ARCO response overlay.


- MX-SEC-DATA-021: `mx_sector_notice_delta` stores sector-specific privacy notice clauses.


- MX-SEC-DATA-022: `mx_sector_audit_profile` stores scrub and retention profile by sector.


- MX-SEC-DATA-023: `mx_sector_conflict.source_codes` stores conflicting authorities.


- MX-SEC-DATA-024: `mx_sector_conflict.resolution` stores strictest, deny, legal-review, or sector-review result.


- MX-SEC-DATA-025: `mx_sector_authority_refs` stores official URL and article or provision ids.


## API Contract Deltas

- MX-SEC-API-001: `POST /sector/mx/profile` stores tenant sector profile.


- MX-SEC-API-002: `GET /sector/mx/profile/{tenant_id}` returns active Mexico sector overlays.


- MX-SEC-API-003: `POST /sector/mx/cnbv/bank/preflight` evaluates banking overlay applicability.


- MX-SEC-API-004: `POST /sector/mx/cnbv/fintech/preflight` evaluates fintech overlay applicability.


- MX-SEC-API-005: `POST /sector/mx/cnbv/security/preflight` evaluates cybersecurity and electronic-means obligations.


- MX-SEC-API-006: `POST /sector/mx/cnbv/third-party/preflight` evaluates provider and outsourcing risks.


- MX-SEC-API-007: `POST /sector/mx/cnsf/preflight` evaluates insurance or bonding overlay applicability.


- MX-SEC-API-008: `POST /sector/mx/cnsf/claims/preflight` evaluates claims and sensitive-data treatment.


- MX-SEC-API-009: `POST /sector/mx/cnsf/reporting/preflight` evaluates supervisory reporting records.


- MX-SEC-API-010: `POST /sector/mx/cre/preflight` evaluates energy overlay applicability.


- MX-SEC-API-011: `POST /sector/mx/cre/metering/preflight` evaluates meter and usage identifiability.


- MX-SEC-API-012: `POST /sector/mx/cre/reliability/preflight` evaluates reliability and operational records.


- MX-SEC-API-013: `POST /sector/mx/telecom/preflight` evaluates telecom overlay applicability.


- MX-SEC-API-014: `POST /sector/mx/telecom/device/preflight` evaluates device and IMEI records.


- MX-SEC-API-015: `POST /sector/mx/telecom/portability/preflight` evaluates portability workflows.


- MX-SEC-API-016: `POST /sector/mx/telecom/accessibility/preflight` evaluates accessibility workflows.


- MX-SEC-API-017: `POST /sector/mx/telecom/transition` records IFT, CRT, ATDT, SICT, or LMTR context.


- MX-SEC-API-018: `POST /sector/mx/atdt/identity/preflight` evaluates government digital identity boundaries.


- MX-SEC-API-019: `POST /sector/mx/transfer/preflight` applies sector transfer controls.


- MX-SEC-API-020: `POST /sector/mx/incident/preflight` applies sector incident controls.


- MX-SEC-API-021: `POST /sector/mx/rights/preflight` applies sector ARCO controls.


- MX-SEC-API-022: `POST /sector/mx/conflict/resolve` resolves sector conflicts.


- MX-SEC-API-023: `GET /sector/mx/authority/{sector_code}` returns authority refs.


- MX-SEC-API-024: `GET /sector/mx/failure-modes` returns sector failure-mode catalog.


- MX-SEC-API-025: `POST /sector/mx/audit-redaction-check` validates sector audit payload redaction.


## Audit Event Additions (per ADR-0263)

- MX-SEC-AUDIT-001: `MxSectorProfileCreated` records tenant and sector codes.


- MX-SEC-AUDIT-002: `MxSectorProfileUpdated` records changed sector codes and evidence refs.


- MX-SEC-AUDIT-003: `MxCnbvBankOverlayApplied` records banking workflow and authority refs.


- MX-SEC-AUDIT-004: `MxCnbvFintechOverlayApplied` records fintech workflow and cybersecurity context.


- MX-SEC-AUDIT-005: `MxCnbvSecurityControlReviewed` records electronic-means or information-security review.


- MX-SEC-AUDIT-006: `MxCnbvThirdPartyReviewed` records technology provider and outsourcing review.


- MX-SEC-AUDIT-007: `MxCnsfOverlayApplied` records insurance or bonding workflow.


- MX-SEC-AUDIT-008: `MxCnsfClaimReviewed` records claim data and sensitive-data context.


- MX-SEC-AUDIT-009: `MxCnsfReportingReviewed` records supervisory reporting context.


- MX-SEC-AUDIT-010: `MxCreOverlayApplied` records energy domain and regulated activity.


- MX-SEC-AUDIT-011: `MxCreMeteringReviewed` records identifiability of meter or usage data.


- MX-SEC-AUDIT-012: `MxCreReliabilityReviewed` records operational reliability context.


- MX-SEC-AUDIT-013: `MxTelecomOverlayApplied` records telecom workflow and authority context.


- MX-SEC-AUDIT-014: `MxTelecomDeviceReviewed` records device identifier review.


- MX-SEC-AUDIT-015: `MxTelecomPortabilityReviewed` records portability workflow.


- MX-SEC-AUDIT-016: `MxTelecomAccessibilityReviewed` records accessibility workflow.


- MX-SEC-AUDIT-017: `MxTelecomTransitionResolved` records IFT, CRT, ATDT, SICT, or LMTR mapping.


- MX-SEC-AUDIT-018: `MxAtdtIdentityBoundaryReviewed` records government identity segregation decision.


- MX-SEC-AUDIT-019: `MxSectorTransferReviewed` records sector transfer decision.


- MX-SEC-AUDIT-020: `MxSectorIncidentReviewed` records sector incident decision.


- MX-SEC-AUDIT-021: `MxSectorRightsReviewed` records sector ARCO decision.


- MX-SEC-AUDIT-022: `MxSectorConflictDetected` records conflicting overlays.


- MX-SEC-AUDIT-023: `MxSectorConflictResolved` records strictest, deny, legal-review, or sector-review outcome.


- MX-SEC-AUDIT-024: `MxSectorActionDenied` records missing sector evidence or authority.


- MX-SEC-AUDIT-025: `MxSectorAuditPayloadRejected` records raw sector data in audit payload.


## Failure Modes

- MX-SEC-FAIL-001: Unknown sector classification must be treated as review-required.


- MX-SEC-FAIL-002: CNBV banking data handled as ordinary CRM data must be rejected.


- MX-SEC-FAIL-003: CNBV fintech cybersecurity provisions applied to non-ITF tenants without evidence must be rejected.


- MX-SEC-FAIL-004: CNBV fintech workflow without third-party service review must be rejected.


- MX-SEC-FAIL-005: CNBV electronic-means workflow without security evidence must be rejected.


- MX-SEC-FAIL-006: CNSF policyholder records handled without insurance purpose must be rejected.


- MX-SEC-FAIL-007: CNSF claims records containing health data without sensitive-data review must be rejected.


- MX-SEC-FAIL-008: CNSF supervisory reports without reporting purpose must be rejected.


- MX-SEC-FAIL-009: CRE permit applicant records treated as anonymous must be rejected when identifiable.


- MX-SEC-FAIL-010: CRE metering records treated as aggregate without proof must be rejected.


- MX-SEC-FAIL-011: CRE reliability incident records disclosed without operational review must be rejected.


- MX-SEC-FAIL-012: Telecom subscriber records handled without telecom overlay must be rejected.


- MX-SEC-FAIL-013: Telecom traffic records handled as generic analytics must be rejected.


- MX-SEC-FAIL-014: Telecom device identifiers handled without subscriber-linkage review must be rejected.


- MX-SEC-FAIL-015: Telecom portability records without user-rights context must be rejected.


- MX-SEC-FAIL-016: Telecom accessibility records without sensitive or disability context review must be rejected.


- MX-SEC-FAIL-017: IFT-only citation without CRT, ATDT, SICT, or LMTR transition mapping must be rejected.


- MX-SEC-FAIL-018: ATDT digital identity data merged into tenant login graph must be rejected.


- MX-SEC-FAIL-019: Sector transfer without overlay-specific transfer review must be rejected.


- MX-SEC-FAIL-020: Sector incident without overlay-specific incident review must be rejected.


- MX-SEC-FAIL-021: Sector ARCO response without overlay-specific rights review must be rejected.


- MX-SEC-FAIL-022: Sector conflict resolved by generic allow must be rejected.


- MX-SEC-FAIL-023: Sector audit payload containing raw financial, insurance, energy, telecom, or identity data must be rejected.


- MX-SEC-FAIL-024: Sector authority ref without official URL must be rejected.


- MX-SEC-FAIL-025: Sector work touching other geography packs must be rejected.


## Worked Examples

- MX-SEC-EXAMPLE-001: A bank customer profile activates CNBV banking overlay and LFPDPPP notice controls.


- MX-SEC-EXAMPLE-002: A loan-origination fraud score activates CNBV banking, sensitive-data review if special categories appear, and audit scrub checks.


- MX-SEC-EXAMPLE-003: A fintech crowdfunding project page activates CNBV fintech disclosure and cybersecurity overlay.


- MX-SEC-EXAMPLE-004: A fintech API outage activates CNBV ITF cybersecurity and incident workflow.


- MX-SEC-EXAMPLE-005: A cloud provider for an ITF platform activates CNBV third-party review.


- MX-SEC-EXAMPLE-006: A policyholder onboarding workflow activates CNSF policyholder overlay.


- MX-SEC-EXAMPLE-007: A life-insurance claim with medical documents activates CNSF claim overlay and sensitive-data consent.


- MX-SEC-EXAMPLE-008: An adjuster mobile app activates CNSF agent or adjuster records overlay.


- MX-SEC-EXAMPLE-009: A solvency or actuarial report activates CNSF supervisory reporting overlay.


- MX-SEC-EXAMPLE-010: An electricity generation permit application activates CRE permit overlay.


- MX-SEC-EXAMPLE-011: A commercial energy customer meter stream activates CRE metering and identifiability review.


- MX-SEC-EXAMPLE-012: A gas commercialization customer list activates CRE hydrocarbon or gas overlay.


- MX-SEC-EXAMPLE-013: An outage communication system activates CRE reliability and user-impact review.


- MX-SEC-EXAMPLE-014: A mobile subscriber account activates telecom subscriber overlay.


- MX-SEC-EXAMPLE-015: A call-detail or network usage record activates telecom traffic overlay.


- MX-SEC-EXAMPLE-016: An IMEI blacklist workflow activates telecom device overlay.


- MX-SEC-EXAMPLE-017: A number portability request activates telecom portability overlay.


- MX-SEC-EXAMPLE-018: A disability accessibility support case activates telecom accessibility and sensitive-context review.


- MX-SEC-EXAMPLE-019: A satellite connectivity service activates LMTR infrastructure context.


- MX-SEC-EXAMPLE-020: A telecom complaint page citing IFT must also tag CRT or ATDT transition context.


- MX-SEC-EXAMPLE-021: A Llave MX integration activates ATDT identity boundary review.


- MX-SEC-EXAMPLE-022: A tenant operating fintech and telecom products activates both overlays and strictest conflict resolution.


- MX-SEC-EXAMPLE-023: A financial breach with telecom device data triggers CNBV and telecom incident reviews.


- MX-SEC-EXAMPLE-024: A transfer of insurance claim data to foreign reinsurer triggers CNSF and LFPDPPP transfer controls.


- MX-SEC-EXAMPLE-025: A sector record emitted into audit with raw personal data is rejected under ADR-0263.


## Cross-References

- MX-SEC-XREF-001: `README.md` describes MX-PACK-1 identity and no-touch scope.


- MX-SEC-XREF-002: `regulatory-coverage.md` maps sector authorities into coverage rows.


- MX-SEC-XREF-003: `data-residency-and-cross-border.md` applies sector transfer review.


- MX-SEC-XREF-004: `consent-and-data-subject-rights.md` applies sector rights review.


- MX-SEC-XREF-005: `breach-notification-and-incident-response.md` applies sector incident review.


- MX-SEC-XREF-006: ADR-0243 makes every sector decision a Cedar gate.


- MX-SEC-XREF-007: ADR-0244 binds sector decisions to tenant and sub-scope.


- MX-SEC-XREF-008: ADR-0251 turns sector overlays into compliance-pack bundle deltas.


- MX-SEC-XREF-009: ADR-0263 requires sector audit events to be scrubbed.


- MX-SEC-XREF-010: LFPDPPP Art. 6 remains the privacy principles floor for sectors.


- MX-SEC-XREF-011: LFPDPPP Art. 9 remains the sensitive-data floor for sectors.


- MX-SEC-XREF-012: LFPDPPP Art. 19 remains the security floor for sectors.


- MX-SEC-XREF-013: LFPDPPP Art. 20 remains the breach notification analysis floor for sectors.


- MX-SEC-XREF-014: LFPDPPP Arts. 36-37 remain the transfer floor for sectors.


- MX-SEC-XREF-015: CNBV data-protection source anchors financial privacy context.


- MX-SEC-XREF-016: CNBV fintech cybersecurity source anchors ITF security context.


- MX-SEC-XREF-017: CNBV institutions-of-credit sources anchor banking context.


- MX-SEC-XREF-018: CNSF data-protection source anchors insurance privacy inventories.


- MX-SEC-XREF-019: CUSF source anchors insurance and bonding regulatory context.


- MX-SEC-XREF-020: CRE sources anchor energy regulated-activity context.


- MX-SEC-XREF-021: IFT source anchors historical telecom regulator and user-rights context.


- MX-SEC-XREF-022: CRT portal anchors IFT acervo and transition tools.


- MX-SEC-XREF-023: ATDT source anchors current transformation-digital and telecom transition context.


- MX-SEC-XREF-024: LMTR Arts. 4-5 anchor telecom infrastructure and federal jurisdiction.


- MX-SEC-XREF-025: Sector overlays must not mutate any non-Mexico localization pack.

