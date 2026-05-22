---
doc_class: LocalizationPack
pack_id: AU-PACK-1
doc_id: AU-PACK-1-SECTORAL-OVERLAYS
title: Australia Sectoral Overlays
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.austrac.gov.au/about-us/legislation/amlctf-act
  - https://handbook.apra.gov.au/standard/cps-234
  - https://asic.gov.au/for-finance-professionals/afs-licensees/afs-licensee-obligations/
  - https://asic.gov.au/regulatory-resources/corporate-governance/cyber-resilience/
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/health-service-providers/my-health-record/guide-to-mandatory-data-breach-notification-in-the-my-health-record-system
  - https://www.ahpra.gov.au/About-AHPRA/Privacy.aspx
  - https://www.ahpra.gov.au/Notifications/Mandatory-reporting.aspx
  - https://www.legislation.gov.au/Details/C2022C00361
---

# Australia Sectoral Overlays

This document defines AU-PACK-1 sectoral overlays.
It covers AUSTRAC AML/CTF, APRA banking and prudential controls, ASIC securities and financial services, My Health Record breach workflows, and Ahpra health-practitioner regulatory data.
It does not replace primary APP and NDB controls.
It activates sector overlays only when tenant and product profiles require them.
It routes unresolved regulated-service classification to review.

## Authority Citations

Citation-001: AUSTRAC AML/CTF Act page anchors reporting entity and designated-service obligations.
Citation-002: AUSTRAC guidance identifies enrolment obligations for designated services.
Citation-003: AUSTRAC guidance identifies registration obligations for remittance and digital currency exchange services.
Citation-004: AUSTRAC guidance identifies AML/CTF program obligations.
Citation-005: APRA CPS 234 anchors information-security capability for APRA-regulated entities.
Citation-006: APRA CPS 234 anchors information-security incident notification behavior.
Citation-007: ASIC AFS licensee obligations anchor efficient, honest, and fair financial services.
Citation-008: ASIC cyber resilience page anchors cyber-risk governance expectations for ASIC-regulated firms.
Citation-009: OAIC My Health Record breach guide anchors MHR-specific breach notification.
Citation-010: Ahpra privacy page anchors practitioner regulator personal-information handling.
Citation-011: Ahpra mandatory reporting page anchors mandatory notification context for practitioners, employers, and education providers.
Citation-012: Privacy Act 1988 anchors the APP baseline that still applies to personal information.

## Overlay Activation

Activation-001: AUSTRAC overlay activates when product provides designated service.
Activation-002: AUSTRAC overlay activates when tenant declares reporting entity status.
Activation-003: AUSTRAC overlay activates when remittance service is offered.
Activation-004: AUSTRAC overlay activates when digital currency exchange service is offered.
Activation-005: AUSTRAC overlay activates when suspicious matter workflow exists.
Activation-006: AUSTRAC overlay activates when threshold transaction workflow exists.
Activation-007: AUSTRAC overlay activates when international funds transfer workflow exists.
Activation-008: APRA overlay activates when tenant is APRA-regulated entity.
Activation-009: APRA overlay activates when banking workload is hosted for APRA-regulated entity.
Activation-010: APRA overlay activates when CPS 234 is contractually required.
Activation-011: ASIC overlay activates when product provides financial services in Australia.
Activation-012: ASIC overlay activates when AFS licensee status is asserted.
Activation-013: ASIC overlay activates when representative status is asserted.
Activation-014: ASIC overlay activates when securities advice, dealing, custody, or market-facing workflows exist.
Activation-015: Health overlay activates when My Health Record data is processed.
Activation-016: Health overlay activates when health service provider profile exists.
Activation-017: Ahpra overlay activates when practitioner registration data is processed.
Activation-018: Ahpra overlay activates when practitioner complaint or notification data is processed.
Activation-019: Sector classification unknown produces review-required.
Activation-020: Sector overlay does not deactivate APP or NDB controls.

## AUSTRAC Overlay

AUSTRAC-001: Store designated-service classification.
AUSTRAC-002: Store reporting-entity status.
AUSTRAC-003: Store enrolment evidence.
AUSTRAC-004: Store registration evidence where remittance applies.
AUSTRAC-005: Store registration evidence where digital currency exchange applies.
AUSTRAC-006: Store AML/CTF program version.
AUSTRAC-007: Store ML/TF risk assessment.
AUSTRAC-008: Store customer identification procedure.
AUSTRAC-009: Store beneficial owner procedure.
AUSTRAC-010: Store politically exposed person handling.
AUSTRAC-011: Store sanctions screening linkage.
AUSTRAC-012: Store ongoing due diligence state.
AUSTRAC-013: Store suspicious matter route.
AUSTRAC-014: Store threshold transaction route where applicable.
AUSTRAC-015: Store international funds transfer route where applicable.
AUSTRAC-016: Store recordkeeping profile.
AUSTRAC-017: Store compliance reporting profile.
AUSTRAC-018: Restrict suspicious matter visibility.
AUSTRAC-019: Flag tipping-off risk.
AUSTRAC-020: Link privacy data minimisation to AML evidence retention.
AUSTRAC-021: Link AUSTRAC reporting to incident compartments where needed.
AUSTRAC-022: Link AUSTRAC customer data to APP 3 collection purpose.
AUSTRAC-023: Link AUSTRAC data use to APP 6 legal obligation where applicable.
AUSTRAC-024: Link AUSTRAC identity data to APP 9 review if government identifiers appear.
AUSTRAC-025: Link AUSTRAC recordkeeping to deletion exceptions.

## APRA Overlay

APRA-001: Store APRA-regulated entity status.
APRA-002: Store entity type.
APRA-003: Store CPS 234 control profile.
APRA-004: Store information asset inventory.
APRA-005: Store threat context.
APRA-006: Store vulnerability context.
APRA-007: Store information-security capability statement.
APRA-008: Store control design evidence.
APRA-009: Store control operating effectiveness evidence.
APRA-010: Store testing cadence.
APRA-011: Store control deficiency register.
APRA-012: Store remediation plan.
APRA-013: Store third-party dependency inventory.
APRA-014: Store third-party assurance evidence.
APRA-015: Store incident materiality criteria.
APRA-016: Store APRA notification route.
APRA-017: Store board or senior accountability field.
APRA-018: Store audit and assurance exports.
APRA-019: Link CPS 234 incident to NDB where personal information is involved.
APRA-020: Link CPS 234 controls to APP 11 reasonable security.
APRA-021: Link overseas security suppliers to APP 8.
APRA-022: Link APRA evidence exports to ADR-0263 hashes.
APRA-023: Link APRA material incidents to postmortem actions.
APRA-024: Link APRA remediation to deployment gates.
APRA-025: Link APRA control testing to release evidence.

## ASIC Overlay

ASIC-001: Store AFS license status.
ASIC-002: Store AFS license number or evidence reference.
ASIC-003: Store representative status.
ASIC-004: Store representative authority scope.
ASIC-005: Store service type.
ASIC-006: Store financial product type.
ASIC-007: Store advice versus dealing classification.
ASIC-008: Store custody or asset-holding classification.
ASIC-009: Store market-facing classification.
ASIC-010: Store efficient, honest, and fair control mapping.
ASIC-011: Store disclosure document evidence where applicable.
ASIC-012: Store complaints and dispute resolution profile.
ASIC-013: Store remediation profile.
ASIC-014: Store customer harm incident route.
ASIC-015: Store cyber resilience evidence.
ASIC-016: Store license variation status.
ASIC-017: Store license cancellation status.
ASIC-018: Store representative termination status.
ASIC-019: Store marketing claims review.
ASIC-020: Store record retention profile.
ASIC-021: Link ASIC customer data to APP controls.
ASIC-022: Link ASIC incident data to NDB where personal information is involved.
ASIC-023: Link ASIC remediation to audit evidence.
ASIC-024: Link ASIC license checks to service activation.
ASIC-025: Link ASIC cyber resilience to APRA where dual-regulated.

## My Health Record Overlay

MHR-001: Store My Health Record data flag.
MHR-002: Store participant role.
MHR-003: Store access context.
MHR-004: Store upload context.
MHR-005: Store download context.
MHR-006: Store consumer access context.
MHR-007: Store healthcare provider access context.
MHR-008: Store emergency access context.
MHR-009: Store breach assessment id.
MHR-010: Store MHR-specific notification route.
MHR-011: Store OAIC coordination evidence.
MHR-012: Store affected individual communication evidence.
MHR-013: Store system operator communication evidence where applicable.
MHR-014: Store health data sensitivity label.
MHR-015: Store clinical document type.
MHR-016: Store access log reference.
MHR-017: Store correction or access conflict references.
MHR-018: Link MHR breach to NDB where applicable.
MHR-019: Link MHR access to APP 11 security controls.
MHR-020: Link MHR export to APP 8 where overseas disclosure is proposed.

## Ahpra Overlay

Ahpra-001: Store practitioner registration number hash.
Ahpra-002: Store practitioner board or profession.
Ahpra-003: Store public register source flag.
Ahpra-004: Store private complaint source flag.
Ahpra-005: Store notification context.
Ahpra-006: Store mandatory notification context.
Ahpra-007: Store voluntary notification context.
Ahpra-008: Store employer notification context.
Ahpra-009: Store education provider notification context.
Ahpra-010: Store treating practitioner context where relevant.
Ahpra-011: Store reasonable belief evidence reference where relevant.
Ahpra-012: Store notifiable conduct category where relevant.
Ahpra-013: Store privacy purpose basis.
Ahpra-014: Store disclosure authority basis.
Ahpra-015: Store confidentiality restriction.
Ahpra-016: Store regulator recipient.
Ahpra-017: Store subject access conflict.
Ahpra-018: Store correction conflict.
Ahpra-019: Link Ahpra data to APP sensitive review where health information appears.
Ahpra-020: Link Ahpra disclosure to audit minimisation.

## Activated Cedar Policies

Cedar-001: `au.austrac.designated_service_required` blocks ambiguous designated-service launch.
Cedar-002: `au.austrac.reporting_entity_required` blocks ambiguous reporting-entity workflow.
Cedar-003: `au.austrac.aml_program_required` blocks designated-service activation without program evidence.
Cedar-004: `au.austrac.compartment_required` blocks broad access to suspicious matter data.
Cedar-005: `au.apra.entity_profile_required` blocks APRA claim without profile.
Cedar-006: `au.apra.cps234_profile_required` blocks APRA workload without CPS 234 profile.
Cedar-007: `au.apra.material_incident_route` blocks incident closure without materiality review.
Cedar-008: `au.asic.afs_license_required` blocks financial service activation without license evidence.
Cedar-009: `au.asic.representative_scope` blocks representative action outside authority.
Cedar-010: `au.asic.customer_harm_route` blocks customer harm closure without remediation review.
Cedar-011: `au.mhr.data_flag_required` blocks MHR processing without classification.
Cedar-012: `au.mhr.breach_route_required` blocks MHR incident closure without MHR route.
Cedar-013: `au.ahpra.purpose_required` blocks practitioner data use without purpose.
Cedar-014: `au.ahpra.disclosure_authority` blocks practitioner disclosure without authority.
Cedar-015: `au.sector.review_required` blocks ambiguous sector claims.

## Data Model Deltas

Data-001: `au_sector_overlay_id` identifies sector overlay.
Data-002: `au_sector_overlay_type` stores austrac, apra, asic, mhr, ahpra.
Data-003: `au_sector_activation_status` stores inactive, active, review_required, suspended.
Data-004: `au_sector_authority_ref` stores official URL and rule reference.
Data-005: `au_sector_counsel_review_id` stores review evidence.
Data-006: `au_austrac_designated_service_codes` stores service codes.
Data-007: `au_austrac_reporting_entity_status` stores status.
Data-008: `au_austrac_program_id` stores AML/CTF program.
Data-009: `au_apra_entity_type` stores APRA entity type.
Data-010: `au_apra_cps234_profile_id` stores CPS 234 profile.
Data-011: `au_asic_afs_license_ref` stores AFS license evidence.
Data-012: `au_asic_representative_ref` stores representative evidence.
Data-013: `au_mhr_data_flag` stores MHR data classification.
Data-014: `au_mhr_participant_role` stores participant role.
Data-015: `au_ahpra_practitioner_ref_hash` stores practitioner reference hash.
Data-016: `au_ahpra_notification_context` stores notification context.
Data-017: `au_sector_incident_route_id` stores incident route.
Data-018: `au_sector_evidence_hash` stores ADR-0263 evidence hash.
Data-019: `au_sector_visibility_compartment_id` stores restricted compartment.
Data-020: `au_sector_retention_profile_id` stores retention profile.

## API Contract Deltas

API-001: `POST /sector/au/austrac/classify` classifies designated service.
API-002: `POST /sector/au/austrac/programs` records AML/CTF program.
API-003: `POST /sector/au/austrac/reporting-routes` records reporting route.
API-004: `POST /sector/au/apra/profiles` records APRA CPS 234 profile.
API-005: `POST /sector/au/apra/incidents` records APRA materiality route.
API-006: `POST /sector/au/asic/license-checks` records AFS license check.
API-007: `POST /sector/au/asic/remediation` records securities remediation route.
API-008: `POST /sector/au/mhr/classify` records My Health Record data classification.
API-009: `POST /sector/au/mhr/breach-routes` records MHR breach route.
API-010: `POST /sector/au/ahpra/practitioner-data` records practitioner data purpose.
API-011: `POST /sector/au/ahpra/notifications` records notification context.
API-012: `GET /sector/au/{overlay_id}/evidence` exports sector evidence.

## Audit Event Additions (per ADR-0263)

Audit-001: `AuSectorOverlayActivated` records overlay type and scope.
Audit-002: `AuSectorOverlayReviewRequired` records ambiguity.
Audit-003: `AuAustracServiceClassified` records designated-service decision.
Audit-004: `AuAustracProgramRecorded` records AML/CTF program.
Audit-005: `AuAustracRestrictedCompartmentOpened` records reporting compartment.
Audit-006: `AuApraCps234ProfileRecorded` records CPS 234 profile.
Audit-007: `AuApraMaterialityReviewed` records incident materiality.
Audit-008: `AuAsicAfsLicenseRecorded` records license evidence.
Audit-009: `AuAsicRepresentativeScopeChecked` records representative scope.
Audit-010: `AuAsicRemediationOpened` records customer harm route.
Audit-011: `AuMhrDataClassified` records MHR classification.
Audit-012: `AuMhrBreachRouteOpened` records MHR breach route.
Audit-013: `AuAhpraPractitionerDataClassified` records practitioner data classification.
Audit-014: `AuAhpraNotificationContextRecorded` records notification context.
Audit-015: `AuSectorEvidenceExported` records evidence hash.

## Failure Modes

Failure-001: AUSTRAC designated service unknown blocks financial-crime feature launch.
Failure-002: AML/CTF program missing blocks reporting-entity activation.
Failure-003: Suspicious matter data visible in general support queue is invalid.
Failure-004: APRA entity claim without CPS 234 profile is invalid.
Failure-005: APRA material incident closes without APRA route is invalid.
Failure-006: ASIC financial service launches without AFS license evidence is invalid.
Failure-007: ASIC representative acts outside scope is invalid.
Failure-008: MHR data handled as generic health data is invalid.
Failure-009: MHR breach closes without MHR-specific route is invalid.
Failure-010: Ahpra notification data disclosed without purpose is invalid.
Failure-011: Practitioner public data merged with private complaint data is invalid.
Failure-012: Sector evidence export without ADR-0263 hash is invalid.
Failure-013: Sector activation without counsel review marker is review-required.
Failure-014: Sector overlap conflict without precedence decision is review-required.
Failure-015: Sector overlay deactivation with open obligations is invalid.

## Worked Examples

Example-001: Digital currency exchange feature activates AUSTRAC registration evidence.
Example-002: Remittance workflow activates AUSTRAC reporting route.
Example-003: Suspicious activity case opens restricted compartment.
Example-004: Bank tenant activates APRA CPS 234 control profile.
Example-005: APRA incident route opens after ransomware affects information assets.
Example-006: Investment advice surface activates ASIC AFS license check.
Example-007: Representative portal blocks action outside representative authority.
Example-008: Customer harm from securities workflow opens ASIC remediation route.
Example-009: My Health Record document export activates MHR classification.
Example-010: MHR unauthorized access opens MHR breach route and NDB assessment.
Example-011: Practitioner complaint intake activates Ahpra notification context.
Example-012: Mandatory reporting context records reasonable belief evidence reference.
Example-013: Public Ahpra register data remains separated from private complaint data.
Example-014: Dual APRA and ASIC tenant activates both overlays with separate evidence.
Example-015: Ambiguous fintech feature enters sector review-required state.

## Cross-References

CrossRef-001: `README.md` defines AU-PACK-1 sector scope.
CrossRef-002: `regulatory-coverage.md` maps sector authorities to controls.
CrossRef-003: `data-residency-and-cross-border.md` defines overseas recipient controls for sector processors.
CrossRef-004: `consent-and-data-subject-rights.md` defines rights conflicts for sector records.
CrossRef-005: `breach-notification-and-incident-response.md` defines sector incident routes.
CrossRef-006: ADR-0243 defines Cedar policy gates.
CrossRef-007: ADR-0244 defines tenant scope.
CrossRef-008: ADR-0251 defines pack overlays.
CrossRef-009: ADR-0263 defines audit evidence.

## Sector Checklist

Check-001: Identify product sector.
Check-002: Identify tenant sector.
Check-003: Identify regulator.
Check-004: Identify service classification.
Check-005: Identify licensing state.
Check-006: Identify reporting state.
Check-007: Identify incident route.
Check-008: Identify data classes.
Check-009: Identify privacy overlaps.
Check-010: Identify NDB overlaps.
Check-011: Identify cross-border overlaps.
Check-012: Identify residency overlaps.
Check-013: Identify rights conflicts.
Check-014: Identify retention conflicts.
Check-015: Identify evidence owner.
Check-016: Identify counsel reviewer.
Check-017: Identify audit events.
Check-018: Identify export package.
Check-019: Identify open obligations.
Check-020: Identify closure gate.

## Sector Evidence Rows

SectorRow-001: AUSTRAC evidence records designated service.
SectorRow-002: AUSTRAC evidence records reporting entity.
SectorRow-003: AUSTRAC evidence records enrolment.
SectorRow-004: AUSTRAC evidence records remittance registration.
SectorRow-005: AUSTRAC evidence records digital currency registration.
SectorRow-006: AUSTRAC evidence records AML/CTF program.
SectorRow-007: AUSTRAC evidence records ML/TF risk assessment.
SectorRow-008: AUSTRAC evidence records customer due diligence.
SectorRow-009: AUSTRAC evidence records beneficial ownership.
SectorRow-010: AUSTRAC evidence records politically exposed person.
SectorRow-011: AUSTRAC evidence records sanctions screening.
SectorRow-012: AUSTRAC evidence records ongoing due diligence.
SectorRow-013: AUSTRAC evidence records suspicious matter route.
SectorRow-014: AUSTRAC evidence records threshold transaction route.
SectorRow-015: AUSTRAC evidence records international funds transfer route.
SectorRow-016: AUSTRAC evidence records recordkeeping profile.
SectorRow-017: AUSTRAC evidence records compliance reporting.
SectorRow-018: AUSTRAC evidence records restricted compartment.
SectorRow-019: AUSTRAC evidence records tipping-off review.
SectorRow-020: AUSTRAC evidence records APP 6 legal obligation link.
SectorRow-021: APRA evidence records regulated entity status.
SectorRow-022: APRA evidence records entity type.
SectorRow-023: APRA evidence records CPS 234 profile.
SectorRow-024: APRA evidence records information asset inventory.
SectorRow-025: APRA evidence records threat context.
SectorRow-026: APRA evidence records vulnerability context.
SectorRow-027: APRA evidence records security capability.
SectorRow-028: APRA evidence records control design.
SectorRow-029: APRA evidence records operating effectiveness.
SectorRow-030: APRA evidence records testing cadence.
SectorRow-031: APRA evidence records deficiency register.
SectorRow-032: APRA evidence records remediation plan.
SectorRow-033: APRA evidence records third-party inventory.
SectorRow-034: APRA evidence records third-party assurance.
SectorRow-035: APRA evidence records materiality criteria.
SectorRow-036: APRA evidence records notification route.
SectorRow-037: APRA evidence records accountability field.
SectorRow-038: APRA evidence records audit export.
SectorRow-039: APRA evidence records APP 11 link.
SectorRow-040: APRA evidence records APP 8 supplier link.
SectorRow-041: ASIC evidence records AFS license.
SectorRow-042: ASIC evidence records license evidence.
SectorRow-043: ASIC evidence records representative status.
SectorRow-044: ASIC evidence records representative authority.
SectorRow-045: ASIC evidence records service type.
SectorRow-046: ASIC evidence records financial product type.
SectorRow-047: ASIC evidence records advice classification.
SectorRow-048: ASIC evidence records dealing classification.
SectorRow-049: ASIC evidence records custody classification.
SectorRow-050: ASIC evidence records market classification.
SectorRow-051: ASIC evidence records efficient honest fair control.
SectorRow-052: ASIC evidence records disclosure document.
SectorRow-053: ASIC evidence records complaints profile.
SectorRow-054: ASIC evidence records remediation profile.
SectorRow-055: ASIC evidence records customer harm route.
SectorRow-056: ASIC evidence records cyber resilience.
SectorRow-057: ASIC evidence records license variation.
SectorRow-058: ASIC evidence records license cancellation.
SectorRow-059: ASIC evidence records representative termination.
SectorRow-060: ASIC evidence records marketing claims review.
SectorRow-061: MHR evidence records data flag.
SectorRow-062: MHR evidence records participant role.
SectorRow-063: MHR evidence records access context.
SectorRow-064: MHR evidence records upload context.
SectorRow-065: MHR evidence records download context.
SectorRow-066: MHR evidence records consumer context.
SectorRow-067: MHR evidence records provider context.
SectorRow-068: MHR evidence records emergency context.
SectorRow-069: MHR evidence records breach assessment.
SectorRow-070: MHR evidence records notification route.
SectorRow-071: MHR evidence records OAIC coordination.
SectorRow-072: MHR evidence records individual communication.
SectorRow-073: MHR evidence records system operator communication.
SectorRow-074: MHR evidence records clinical document type.
SectorRow-075: MHR evidence records access log.
SectorRow-076: Ahpra evidence records registration hash.
SectorRow-077: Ahpra evidence records board.
SectorRow-078: Ahpra evidence records profession.
SectorRow-079: Ahpra evidence records public register source.
SectorRow-080: Ahpra evidence records complaint source.
SectorRow-081: Ahpra evidence records notification context.
SectorRow-082: Ahpra evidence records mandatory notification context.
SectorRow-083: Ahpra evidence records voluntary notification context.
SectorRow-084: Ahpra evidence records employer context.
SectorRow-085: Ahpra evidence records education provider context.
SectorRow-086: Ahpra evidence records treating practitioner context.
SectorRow-087: Ahpra evidence records reasonable belief reference.
SectorRow-088: Ahpra evidence records notifiable conduct category.
SectorRow-089: Ahpra evidence records privacy purpose.
SectorRow-090: Ahpra evidence records disclosure authority.
SectorRow-091: Ahpra evidence records confidentiality restriction.
SectorRow-092: Ahpra evidence records regulator recipient.
SectorRow-093: Ahpra evidence records subject access conflict.
SectorRow-094: Ahpra evidence records correction conflict.
SectorRow-095: Ahpra evidence records audit minimisation.
SectorRow-096: Policy evidence records `au.austrac.designated_service_required`.
SectorRow-097: Policy evidence records `au.austrac.reporting_entity_required`.
SectorRow-098: Policy evidence records `au.austrac.aml_program_required`.
SectorRow-099: Policy evidence records `au.austrac.compartment_required`.
SectorRow-100: Policy evidence records `au.apra.entity_profile_required`.
SectorRow-101: Policy evidence records `au.apra.cps234_profile_required`.
SectorRow-102: Policy evidence records `au.apra.material_incident_route`.
SectorRow-103: Policy evidence records `au.asic.afs_license_required`.
SectorRow-104: Policy evidence records `au.asic.representative_scope`.
SectorRow-105: Policy evidence records `au.asic.customer_harm_route`.
SectorRow-106: Policy evidence records `au.mhr.data_flag_required`.
SectorRow-107: Policy evidence records `au.mhr.breach_route_required`.
SectorRow-108: Policy evidence records `au.ahpra.purpose_required`.
SectorRow-109: Policy evidence records `au.ahpra.disclosure_authority`.
SectorRow-110: Policy evidence records `au.sector.review_required`.
SectorRow-111: API evidence records AUSTRAC classify.
SectorRow-112: API evidence records AUSTRAC program.
SectorRow-113: API evidence records AUSTRAC reporting route.
SectorRow-114: API evidence records APRA profile.
SectorRow-115: API evidence records APRA incident.
SectorRow-116: API evidence records ASIC license check.
SectorRow-117: API evidence records ASIC remediation.
SectorRow-118: API evidence records MHR classify.
SectorRow-119: API evidence records MHR breach route.
SectorRow-120: API evidence records Ahpra practitioner data.
SectorRow-121: API evidence records Ahpra notification.
SectorRow-122: API evidence records sector export.
SectorRow-123: Audit evidence records `AuSectorOverlayActivated`.
SectorRow-124: Audit evidence records `AuSectorOverlayReviewRequired`.
SectorRow-125: Audit evidence records `AuAustracServiceClassified`.
SectorRow-126: Audit evidence records `AuAustracProgramRecorded`.
SectorRow-127: Audit evidence records `AuAustracRestrictedCompartmentOpened`.
SectorRow-128: Audit evidence records `AuApraCps234ProfileRecorded`.
SectorRow-129: Audit evidence records `AuApraMaterialityReviewed`.
SectorRow-130: Audit evidence records `AuAsicAfsLicenseRecorded`.
SectorRow-131: Audit evidence records `AuAsicRepresentativeScopeChecked`.
SectorRow-132: Audit evidence records `AuAsicRemediationOpened`.
SectorRow-133: Audit evidence records `AuMhrDataClassified`.
SectorRow-134: Audit evidence records `AuMhrBreachRouteOpened`.
SectorRow-135: Audit evidence records `AuAhpraPractitionerDataClassified`.
SectorRow-136: Audit evidence records `AuAhpraNotificationContextRecorded`.
SectorRow-137: Audit evidence records `AuSectorEvidenceExported`.
SectorRow-138: Data evidence records `au_sector_overlay_id`.
SectorRow-139: Data evidence records `au_sector_overlay_type`.
SectorRow-140: Data evidence records `au_sector_activation_status`.
SectorRow-141: Data evidence records `au_sector_authority_ref`.
SectorRow-142: Data evidence records `au_sector_counsel_review_id`.
SectorRow-143: Data evidence records `au_austrac_designated_service_codes`.
SectorRow-144: Data evidence records `au_austrac_reporting_entity_status`.
SectorRow-145: Data evidence records `au_austrac_program_id`.
SectorRow-146: Data evidence records `au_apra_entity_type`.
SectorRow-147: Data evidence records `au_apra_cps234_profile_id`.
SectorRow-148: Data evidence records `au_asic_afs_license_ref`.
SectorRow-149: Data evidence records `au_asic_representative_ref`.
SectorRow-150: Data evidence records `au_mhr_data_flag`.
SectorRow-151: Data evidence records `au_mhr_participant_role`.
SectorRow-152: Data evidence records `au_ahpra_practitioner_ref_hash`.
SectorRow-153: Data evidence records `au_ahpra_notification_context`.
SectorRow-154: Data evidence records `au_sector_incident_route_id`.
SectorRow-155: Data evidence records `au_sector_evidence_hash`.
SectorRow-156: Failure evidence records unknown AUSTRAC designation.
SectorRow-157: Failure evidence records missing AML/CTF program.
SectorRow-158: Failure evidence records broad suspicious matter visibility.
SectorRow-159: Failure evidence records missing CPS 234 profile.
SectorRow-160: Failure evidence records missing APRA route.
SectorRow-161: Failure evidence records missing AFS license.
SectorRow-162: Failure evidence records representative outside scope.
SectorRow-163: Failure evidence records generic MHR handling.
SectorRow-164: Failure evidence records missing MHR breach route.
SectorRow-165: Failure evidence records Ahpra purpose missing.
SectorRow-166: Failure evidence records public-private practitioner merge.
SectorRow-167: Failure evidence records missing audit hash.
SectorRow-168: Failure evidence records counsel review missing.
SectorRow-169: Failure evidence records sector overlap conflict.
SectorRow-170: Failure evidence records open obligations at deactivation.
SectorRow-171: Source evidence records AUSTRAC URL.
SectorRow-172: Source evidence records APRA CPS 234 URL.
SectorRow-173: Source evidence records ASIC AFS URL.
SectorRow-174: Source evidence records ASIC cyber resilience URL.
SectorRow-175: Source evidence records My Health Record breach URL.
SectorRow-176: Source evidence records Ahpra privacy URL.
SectorRow-177: Source evidence records Ahpra mandatory reporting URL.
SectorRow-178: Source evidence records Privacy Act URL.
SectorRow-179: Source evidence records AML/CTF Act citation.
SectorRow-180: Source evidence records CPS 234 citation.
SectorRow-181: Source evidence records AFS licensee obligation citation.
SectorRow-182: Source evidence records My Health Record breach citation.
SectorRow-183: Source evidence records Ahpra privacy citation.
SectorRow-184: Source evidence records Ahpra mandatory notification citation.
SectorRow-185: Source evidence records APP baseline citation.
SectorRow-186: Worked evidence records digital currency exchange scenario.
SectorRow-187: Worked evidence records remittance scenario.
SectorRow-188: Worked evidence records suspicious activity scenario.
SectorRow-189: Worked evidence records bank tenant scenario.
SectorRow-190: Worked evidence records APRA ransomware scenario.
SectorRow-191: Worked evidence records investment advice scenario.
SectorRow-192: Worked evidence records representative scope scenario.
SectorRow-193: Worked evidence records securities remediation scenario.
SectorRow-194: Worked evidence records MHR export scenario.
SectorRow-195: Worked evidence records MHR unauthorized access scenario.
SectorRow-196: Worked evidence records practitioner complaint scenario.
SectorRow-197: Worked evidence records mandatory notification scenario.
SectorRow-198: Worked evidence records public register separation scenario.
SectorRow-199: Worked evidence records dual APRA ASIC scenario.
SectorRow-200: Worked evidence records ambiguous fintech scenario.
SectorRow-201: Handoff evidence records financial-crime owner.
SectorRow-202: Handoff evidence records prudential owner.
SectorRow-203: Handoff evidence records securities owner.
SectorRow-204: Handoff evidence records health owner.
SectorRow-205: Handoff evidence records practitioner owner.
SectorRow-206: Handoff evidence records privacy owner.
SectorRow-207: Handoff evidence records security owner.
SectorRow-208: Handoff evidence records audit owner.
SectorRow-209: Handoff evidence records counsel owner.
SectorRow-210: Handoff evidence records compliance owner.
SectorRow-211: Release evidence records frontmatter.
SectorRow-212: Release evidence records required sections.
SectorRow-213: Release evidence records line count.
SectorRow-214: Release evidence records source URLs.
SectorRow-215: Release evidence records Australia-only scope.
SectorRow-216: Release evidence records no other geography.
SectorRow-217: Release evidence records no generation script.
SectorRow-218: Release evidence records sector specificity.
SectorRow-219: Release evidence records official source grounding.
SectorRow-220: Release evidence records final verification.
SectorRow-221: Final evidence records AUSTRAC overlay.
SectorRow-222: Final evidence records APRA overlay.
SectorRow-223: Final evidence records ASIC overlay.
SectorRow-224: Final evidence records My Health Record overlay.
SectorRow-225: Final evidence records Ahpra overlay.
SectorRow-226: Final evidence records APP baseline.
SectorRow-227: Final evidence records NDB overlap.
SectorRow-228: Final evidence records cross-border overlap.
SectorRow-229: Final evidence records residency overlap.
SectorRow-230: Final evidence records rights conflict.
SectorRow-231: Final evidence records retention conflict.
SectorRow-232: Final evidence records incident route.
SectorRow-233: Final evidence records evidence route.
SectorRow-234: Final evidence records regulator route.
SectorRow-235: Final evidence records customer route.
SectorRow-236: Final evidence records sector review.
SectorRow-237: Final evidence records counsel marker.
SectorRow-238: Final evidence records policy marker.
SectorRow-239: Final evidence records audit marker.
SectorRow-240: Final evidence records closure marker.
SectorRow-241: Final evidence records tenant.
SectorRow-242: Final evidence records service.
SectorRow-243: Final evidence records product.
SectorRow-244: Final evidence records regulator.
SectorRow-245: Final evidence records policy id.
SectorRow-246: Final evidence records citation id.
SectorRow-247: Final evidence records evidence id.
SectorRow-248: Final evidence records hash.
SectorRow-249: Final evidence records timestamp.
SectorRow-250: Final evidence records status.
SectorRow-251: Final evidence records line threshold.
SectorRow-252: Final evidence records final source list.
SectorRow-253: Final evidence records final handoff.
SectorRow-254: Final evidence records final source refresh.
SectorRow-255: Final evidence records final pack isolation.
SectorRow-256: Final evidence records final verification marker.
