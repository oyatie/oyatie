---
doc_class: LocalizationPack
pack_id: AU-PACK-1
doc_id: AU-PACK-1-DATA-RESIDENCY-CROSS-BORDER
title: Australia Data Residency and Cross-Border Disclosure
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.legislation.gov.au/Details/C2022C00361
  - https://www.oaic.gov.au/privacy/australian-privacy-principles/australian-privacy-principles-guidelines/chapter-8-app-8-cross-border-disclosure-of-personal-information
  - https://www.oaic.gov.au/privacy/australian-privacy-principles/read-the-australian-privacy-principles
  - https://handbook.apra.gov.au/standard/cps-234
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/health-service-providers/my-health-record/guide-to-mandatory-data-breach-notification-in-the-my-health-record-system
---

# Australia Data Residency and Cross-Border Disclosure

This document defines Australia-specific residency claims and APP 8 cross-border controls.
It treats data residency as a claim that must be proven by deployment and contract evidence.
It treats cross-border disclosure as an APP 8 and section 16C question, not as a generic network-location flag.
It relies on OAIC APP 8 guidance for overseas recipient, disclosure, routing, and accountability concepts.
It relies on Privacy Act Schedule 1 APP 8 and section 16C for the core statutory posture.
It relies on APRA CPS 234 where APRA-regulated entities need information-security control evidence.
It relies on My Health Record breach guidance where health records are involved.

## Authority Citations

Citation-001: Privacy Act 1988 section 16C creates overseas disclosure accountability for APP entities.
Citation-002: Privacy Act Schedule 1 APP 8 creates cross-border disclosure requirements.
Citation-003: OAIC APP 8 guidance says overseas recipient means a recipient outside Australia or an external Territory.
Citation-004: OAIC APP 8 guidance distinguishes disclosure from use.
Citation-005: OAIC APP 8 guidance treats routing through overseas servers as usually a use where control is not released.
Citation-006: OAIC APP 8 guidance requires reasonable steps before overseas disclosure unless an exception applies.
Citation-007: OAIC APP 8 guidance recognizes substantially similar law or binding scheme exception.
Citation-008: OAIC APP 8 guidance recognizes express informed consent exception.
Citation-009: OAIC APP 8 guidance recognizes required-or-authorised-by-Australian-law exception.
Citation-010: OAIC APP 8 guidance requires clear warning when relying on informed consent exception.
Citation-011: APP 11 requires reasonable steps to protect personal information.
Citation-012: APRA CPS 234 requires APRA-regulated entities to maintain information-security capability.
Citation-013: My Health Record guidance requires special breach handling where MHR system data is involved.

## Residency Doctrine

Residency-001: Australia residency is not implied by AU-PACK-1 activation alone.
Residency-002: Australia residency is active only when contract, tenant profile, sector profile, or deployment policy says so.
Residency-003: Australia residency requires named approved regions.
Residency-004: Australia residency requires storage-class mapping.
Residency-005: Australia residency requires backup-location mapping.
Residency-006: Australia residency requires log-location mapping.
Residency-007: Australia residency requires analytics-location mapping.
Residency-008: Australia residency requires incident-evidence-location mapping.
Residency-009: Australia residency requires key-management-location mapping where claimed.
Residency-010: Australia residency requires support-access-location mapping where claimed.
Residency-011: Australia residency does not automatically prohibit overseas access unless profile says so.
Residency-012: Australia residency does not automatically satisfy APP 8.
Residency-013: Australia residency does not remove APP 11 security duties.
Residency-014: Australia residency does not remove NDB duties.
Residency-015: Australia residency does not remove APRA CPS 234 duties.
Residency-016: Australia residency profile must identify whether processing, storage, access, backup, or support is constrained.
Residency-017: Australia-only storage profile blocks object-store replication outside allowed AU regions.
Residency-018: Australia-only backup profile blocks disaster-recovery copy outside allowed AU regions.
Residency-019: Australia-only logs profile blocks log sink outside allowed AU regions.
Residency-020: Australia-only analytics profile blocks derived personal information exports.
Residency-021: Australia-only key profile blocks key material outside allowed key jurisdiction.
Residency-022: Australia-only support profile blocks support access by overseas personnel unless exception exists.
Residency-023: Australia contractual residency exceptions require customer approval evidence.
Residency-024: Australia sectoral residency exceptions require regulator or counsel review.
Residency-025: Australia emergency exceptions require incident commander and privacy officer approval.

## Cross-Border Decision Model

CrossBorder-001: Determine whether the actor is an APP entity.
CrossBorder-002: Determine whether the data is personal information.
CrossBorder-003: Determine whether the data is held by the APP entity.
CrossBorder-004: Determine whether the recipient is outside Australia or an external Territory.
CrossBorder-005: Determine whether recipient is the same entity.
CrossBorder-006: Determine whether recipient is a related body corporate.
CrossBorder-007: Determine whether the action releases handling from effective control.
CrossBorder-008: Determine whether the action is only routing in transit.
CrossBorder-009: Determine whether a contractor receives personal information.
CrossBorder-010: Determine whether the action is disclosure or use.
CrossBorder-011: Determine whether APP 6 permits the disclosure purpose.
CrossBorder-012: Determine whether APP 8.1 reasonable steps are required.
CrossBorder-013: Determine whether APP 8.2(a) substantially similar protection applies.
CrossBorder-014: Determine whether APP 8.2(b) express informed consent applies.
CrossBorder-015: Determine whether APP 8.2(c) Australian-law authorization applies.
CrossBorder-016: Determine whether another APP 8 exception applies.
CrossBorder-017: Determine whether section 16C accountability remains.
CrossBorder-018: Determine whether overseas recipient has Australian link.
CrossBorder-019: Determine whether foreign law access risk is disclosed where consent is used.
CrossBorder-020: Determine whether individual redress limitations are disclosed where consent is used.
CrossBorder-021: Determine whether recipient safeguards include contract.
CrossBorder-022: Determine whether recipient safeguards include technical controls.
CrossBorder-023: Determine whether recipient safeguards include audit rights.
CrossBorder-024: Determine whether recipient safeguards include deletion or return duties.
CrossBorder-025: Determine whether recipient safeguards include subprocessor restrictions.
CrossBorder-026: Determine whether recipient safeguards include incident notification.
CrossBorder-027: Determine whether recipient safeguards include onward-transfer controls.
CrossBorder-028: Determine whether recipient safeguards include access controls.
CrossBorder-029: Determine whether recipient safeguards include encryption controls.
CrossBorder-030: Determine whether recipient safeguards include retention controls.

## Activated Cedar Policies

Cedar-001: `au.residency.profile_required` denies residency claim without profile.
Cedar-002: `au.residency.storage_region` denies storage outside approved AU region.
Cedar-003: `au.residency.backup_region` denies backup outside approved AU region.
Cedar-004: `au.residency.log_region` denies personal log export outside approved AU region.
Cedar-005: `au.residency.analytics_region` denies analytics export outside approved AU region.
Cedar-006: `au.residency.key_region` denies key use outside approved jurisdiction.
Cedar-007: `au.residency.support_access` denies overseas support access without approved exception.
Cedar-008: `au.cross_border.app_entity_required` denies APP 8 evaluation when APP entity status is unknown.
Cedar-009: `au.cross_border.personal_info_required` skips APP 8 only when non-personal classification is proven.
Cedar-010: `au.cross_border.disclosure_classifier` blocks release until disclosure/use classification is complete.
Cedar-011: `au.cross_border.app6_compatible` denies APP 8 evaluation if APP 6 purpose fails.
Cedar-012: `au.cross_border.reasonable_steps` denies overseas disclosure without safeguard evidence.
Cedar-013: `au.cross_border.substantially_similar` allows exception only with documented enforceable protection.
Cedar-014: `au.cross_border.express_informed_consent` allows exception only with warning and active consent.
Cedar-015: `au.cross_border.australian_law_authorized` allows exception only with Australian-law citation.
Cedar-016: `au.cross_border.section16c_accountability` records accountability state.
Cedar-017: `au.cross_border.onward_transfer` denies recipient onward transfer without control.
Cedar-018: `au.cross_border.contractor_gate` denies overseas contractor access without APP 8 review.
Cedar-019: `au.cross_border.route_only_review` records route-only analysis.
Cedar-020: `au.cross_border.emergency_exception` denies emergency exception without dual approval.

## Data Model Deltas

Data-001: `au_residency_profile_id` identifies residency commitment.
Data-002: `au_residency_claim_type` stores storage, processing, access, backup, log, analytics, key, support.
Data-003: `au_allowed_regions` stores approved Australia regions.
Data-004: `au_disallowed_regions` stores blocked regions.
Data-005: `au_residency_contract_ref` stores customer contract evidence.
Data-006: `au_residency_sector_ref` stores sector overlay evidence.
Data-007: `au_residency_exception_id` stores approved exception.
Data-008: `au_cross_border_assessment_id` identifies APP 8 decision.
Data-009: `au_app_entity_status` stores covered, not_covered, unknown, review.
Data-010: `au_personal_information_classification` stores personal, sensitive, deidentified, non_personal, unknown.
Data-011: `au_overseas_recipient_id` stores recipient identity.
Data-012: `au_overseas_recipient_country` stores country or territory.
Data-013: `au_same_entity_flag` stores same-entity analysis.
Data-014: `au_related_body_corporate_flag` stores related-body analysis.
Data-015: `au_disclosure_or_use` stores disclosure, use, route_only, unknown.
Data-016: `au_effective_control_released` stores control-release analysis.
Data-017: `au_app6_purpose_check_id` stores purpose compatibility.
Data-018: `au_app8_basis` stores reasonable_steps, similar_law, informed_consent, australian_law, other_exception, denied.
Data-019: `au_reasonable_steps_evidence_id` stores safeguard evidence.
Data-020: `au_similar_law_evidence_id` stores enforceability evidence.
Data-021: `au_informed_consent_warning_id` stores express warning.
Data-022: `au_cross_border_consent_id` stores consent record.
Data-023: `au_australian_law_authority_ref` stores legal authority citation.
Data-024: `au_section16c_accountability_state` stores accountable, exception, not_applicable, unknown.
Data-025: `au_onward_transfer_controls_id` stores recipient onward-transfer controls.
Data-026: `au_subprocessor_controls_id` stores subprocessor controls.
Data-027: `au_foreign_law_risk_note_id` stores disclosure warning.
Data-028: `au_recipient_incident_notice_sla` stores recipient notification timing.
Data-029: `au_recipient_retention_profile_id` stores deletion/return duties.
Data-030: `au_transfer_review_expires_at` stores periodic review date.

## API Contract Deltas

API-001: `POST /residency/au/profiles` creates residency profile.
API-002: `PATCH /residency/au/profiles/{id}` updates residency evidence.
API-003: `POST /residency/au/evaluate-placement` evaluates storage, backup, log, analytics, key, and support placement.
API-004: `POST /residency/au/exceptions` opens residency exception workflow.
API-005: `GET /residency/au/profiles/{id}/evidence` exports residency evidence.
API-006: `POST /privacy/au/cross-border/assessments` creates APP 8 assessment.
API-007: `POST /privacy/au/cross-border/disclosure-classifier` classifies use versus disclosure.
API-008: `POST /privacy/au/cross-border/reasonable-steps` records safeguard evidence.
API-009: `POST /privacy/au/cross-border/similar-law` records APP 8.2(a) evidence.
API-010: `POST /privacy/au/cross-border/informed-consent` records APP 8.2(b) warning and consent.
API-011: `POST /privacy/au/cross-border/australian-law-authority` records APP 8.2(c) basis.
API-012: `POST /privacy/au/cross-border/onward-transfer-controls` records recipient onward-transfer controls.
API-013: `GET /privacy/au/cross-border/assessments/{id}` returns decision and citation bundle.
API-014: `POST /privacy/au/cross-border/assessments/{id}/review` reopens assessment.
API-015: `GET /audit/au/cross-border/events` returns ADR-0263 events for transfer decisions.

## Audit Event Additions (per ADR-0263)

Audit-001: `AuResidencyProfileCreated` records allowed regions and claim type.
Audit-002: `AuResidencyPlacementAllowed` records workload and region.
Audit-003: `AuResidencyPlacementDenied` records blocked region and policy id.
Audit-004: `AuResidencyExceptionApproved` records approvers and expiry.
Audit-005: `AuResidencyExceptionExpired` records enforcement restart.
Audit-006: `AuCrossBorderAssessmentStarted` records subject, recipient, and action.
Audit-007: `AuDisclosureUseClassified` records use, disclosure, route-only, or unknown.
Audit-008: `AuApp6PurposeChecked` records purpose compatibility for disclosure.
Audit-009: `AuReasonableStepsRecorded` records APP 8.1 safeguard evidence.
Audit-010: `AuSimilarLawExceptionRecorded` records APP 8.2(a) evidence.
Audit-011: `AuInformedConsentWarningServed` records APP 8.2(b) warning.
Audit-012: `AuCrossBorderConsentGranted` records active consent.
Audit-013: `AuCrossBorderConsentWithdrawn` records withdrawal.
Audit-014: `AuAustralianLawAuthorizationRecorded` records APP 8.2(c) authority.
Audit-015: `AuSection16cAccountabilityRecorded` records accountability posture.
Audit-016: `AuOverseasRecipientReviewed` records recipient review.
Audit-017: `AuOnwardTransferControlRecorded` records onward-transfer limits.
Audit-018: `AuCrossBorderDisclosureAllowed` records final allow decision.
Audit-019: `AuCrossBorderDisclosureDenied` records final deny decision.
Audit-020: `AuCrossBorderReviewRequired` records ambiguity reason.

## Failure Modes

Failure-001: Residency claim without profile blocks external claim.
Failure-002: Residency profile without allowed regions blocks deployment.
Failure-003: Backup sink outside approved region blocks release.
Failure-004: Log sink outside approved region blocks release.
Failure-005: Analytics export outside approved region blocks release.
Failure-006: Support access outside approved geography blocks session.
Failure-007: Cross-border assessment without APP entity state blocks decision.
Failure-008: Personal-information classification unknown blocks export.
Failure-009: Recipient country unknown blocks export.
Failure-010: Use versus disclosure unknown blocks export.
Failure-011: APP 6 purpose compatibility missing blocks export.
Failure-012: Reasonable steps missing blocks APP 8.1 export.
Failure-013: Similar-law exception without enforceability evidence blocks exception.
Failure-014: Informed consent without express warning blocks exception.
Failure-015: Informed consent after withdrawal blocks exception.
Failure-016: Australian-law exception without Australian authority blocks exception.
Failure-017: Section 16C accountability unknown blocks release.
Failure-018: Onward-transfer controls missing blocks processor export.
Failure-019: Foreign-law risk not disclosed where consent is used blocks consent pathway.
Failure-020: Emergency exception without dual approval blocks export.

## Worked Examples

Example-001: A tenant stores personal information in Sydney and backs up to Melbourne; residency profile allows both.
Example-002: A backup job proposes Singapore; Australia-only backup profile denies the job.
Example-003: A log pipeline includes email addresses; Australia-only logs profile requires AU sink or redaction.
Example-004: A de-identified metric export has no re-identification risk; APP 8 is skipped only after classification evidence.
Example-005: A customer support contractor in India views personal information; APP 8 disclosure assessment is required.
Example-006: A packet routes through a foreign network without recipient control; route-only review records use rather than disclosure where facts support it.
Example-007: A related overseas subsidiary receives user data; APP 8 overseas-recipient analysis is required.
Example-008: A processor contract includes APP safeguards; reasonable steps evidence is attached.
Example-009: A user gives express informed consent after warning; APP 8.2(b) exception is recorded.
Example-010: The user withdraws consent; future transfers can no longer rely on that consent.
Example-011: A court order under Australian law compels transfer; APP 8.2(c) authority is recorded.
Example-012: A foreign subpoena alone is presented; APP 8.2(c) does not accept it as Australian-law authority.
Example-013: APRA tenant uses overseas security provider; CPS 234 third-party and APP 8 evidence are both required.
Example-014: My Health Record data is copied to overseas analytics; MHR-specific review blocks unless lawful basis exists.
Example-015: A contractual residency exception expires; enforcement returns to deny outside AU regions.

## Cross-References

CrossRef-001: `README.md` defines AU-PACK-1 activation and residency caveats.
CrossRef-002: `regulatory-coverage.md` maps APP 8 to Cedar policies.
CrossRef-003: `consent-and-data-subject-rights.md` defines consent records used by APP 8.2(b).
CrossRef-004: `breach-notification-and-incident-response.md` defines incident handling for transfer-related breaches.
CrossRef-005: `sectoral-overlays.md` defines APRA and MHR overlays that affect cross-border handling.
CrossRef-006: ADR-0243 defines Cedar gate evaluation.
CrossRef-007: ADR-0244 defines tenant and sub-scope context.
CrossRef-008: ADR-0251 defines compliance-pack overlay activation.
CrossRef-009: ADR-0263 defines audit event envelopes.

## APP 8 Operational Checklist

APP8-001: Confirm the entity is an APP entity.
APP8-002: Confirm the information is personal information.
APP8-003: Confirm the information is held by the APP entity.
APP8-004: Confirm recipient location.
APP8-005: Confirm whether recipient is the same entity.
APP8-006: Confirm whether recipient is a related body corporate.
APP8-007: Confirm whether personal information is made accessible to recipient.
APP8-008: Confirm whether handling leaves effective control.
APP8-009: Confirm whether routing is transit-only.
APP8-010: Confirm APP 6 purpose compatibility.
APP8-011: Confirm overseas recipient role.
APP8-012: Confirm data classes transferred.
APP8-013: Confirm purpose transferred for.
APP8-014: Confirm recipient safeguards.
APP8-015: Confirm contract clauses.
APP8-016: Confirm security controls.
APP8-017: Confirm subprocessor controls.
APP8-018: Confirm onward-transfer limits.
APP8-019: Confirm incident notice obligations.
APP8-020: Confirm deletion or return duties.
APP8-021: Confirm audit rights.
APP8-022: Confirm individual complaint path.
APP8-023: Confirm section 16C accountability.
APP8-024: Confirm APP 8.2 exception if used.
APP8-025: Confirm similar-law evidence if APP 8.2(a) used.
APP8-026: Confirm enforceability evidence if APP 8.2(a) used.
APP8-027: Confirm express warning if APP 8.2(b) used.
APP8-028: Confirm current consent if APP 8.2(b) used.
APP8-029: Confirm withdrawal route if APP 8.2(b) used.
APP8-030: Confirm Australian-law citation if APP 8.2(c) used.
APP8-031: Confirm foreign-law risk warning where relevant.
APP8-032: Confirm retained evidence id.
APP8-033: Confirm reviewer identity.
APP8-034: Confirm review expiry date.
APP8-035: Confirm audit event emission.

## Residency Operational Checklist

ResidencyCheck-001: Confirm storage region.
ResidencyCheck-002: Confirm compute region.
ResidencyCheck-003: Confirm backup region.
ResidencyCheck-004: Confirm disaster-recovery region.
ResidencyCheck-005: Confirm log sink region.
ResidencyCheck-006: Confirm trace sink region.
ResidencyCheck-007: Confirm metrics sink region.
ResidencyCheck-008: Confirm analytics warehouse region.
ResidencyCheck-009: Confirm object storage replication policy.
ResidencyCheck-010: Confirm database read-replica policy.
ResidencyCheck-011: Confirm search index region.
ResidencyCheck-012: Confirm cache region.
ResidencyCheck-013: Confirm queue region.
ResidencyCheck-014: Confirm data lake region.
ResidencyCheck-015: Confirm key management region.
ResidencyCheck-016: Confirm secret store region.
ResidencyCheck-017: Confirm support access geography.
ResidencyCheck-018: Confirm incident evidence geography.
ResidencyCheck-019: Confirm admin session recording geography.
ResidencyCheck-020: Confirm export destination geography.
ResidencyCheck-021: Confirm retention disposition geography.
ResidencyCheck-022: Confirm legal hold geography.
ResidencyCheck-023: Confirm regulator export geography.
ResidencyCheck-024: Confirm exception expiry.
ResidencyCheck-025: Confirm contract source.
ResidencyCheck-026: Confirm sector source.
ResidencyCheck-027: Confirm counsel review.
ResidencyCheck-028: Confirm customer approval where needed.
ResidencyCheck-029: Confirm deployment policy id.
ResidencyCheck-030: Confirm ADR-0263 evidence hash.

## Data Class Transfer Rules

Transfer-001: Basic contact information requires APP 6 and APP 8 review before overseas disclosure.
Transfer-002: Account authentication data requires APP 11 control review before transfer.
Transfer-003: Health information requires sensitive-information handling and APP 8 review.
Transfer-004: My Health Record data requires MHR-specific review before transfer.
Transfer-005: Government identifier data requires APP 9 review before transfer.
Transfer-006: Financial transaction data requires AUSTRAC and APP review where designated service applies.
Transfer-007: APRA tenant security telemetry requires CPS 234 and APP review when personal information appears.
Transfer-008: Securities advice data requires ASIC, APP, and retention review.
Transfer-009: Practitioner registration data requires Ahpra-purpose review.
Transfer-010: Complaint data requires confidentiality and APP review.
Transfer-011: Support transcript data requires APP 6 and APP 8 review.
Transfer-012: Marketing audience data requires APP 7 and APP 8 review.
Transfer-013: Analytics event data requires personal-information classification before export.
Transfer-014: De-identified data requires re-identification risk assessment before export.
Transfer-015: Aggregated data requires threshold and singling-out assessment before export.
Transfer-016: Audit data requires ADR-0263 minimisation before export.
Transfer-017: Regulator export data requires regulator-specific route.
Transfer-018: Backup data follows residency profile and APP 11 controls.
Transfer-019: Log data follows residency profile and minimisation controls.
Transfer-020: Derived profile data follows the strictest source data class.

## Residency and Transfer Evidence Rows

ResidencyRow-001: Residency evidence records contract claim.
ResidencyRow-002: Residency evidence records sector claim.
ResidencyRow-003: Residency evidence records tenant claim.
ResidencyRow-004: Residency evidence records product claim.
ResidencyRow-005: Residency evidence records approved AU region.
ResidencyRow-006: Residency evidence records storage location.
ResidencyRow-007: Residency evidence records compute location.
ResidencyRow-008: Residency evidence records backup location.
ResidencyRow-009: Residency evidence records disaster recovery location.
ResidencyRow-010: Residency evidence records log sink location.
ResidencyRow-011: Residency evidence records metric sink location.
ResidencyRow-012: Residency evidence records trace sink location.
ResidencyRow-013: Residency evidence records analytics location.
ResidencyRow-014: Residency evidence records search index location.
ResidencyRow-015: Residency evidence records cache location.
ResidencyRow-016: Residency evidence records queue location.
ResidencyRow-017: Residency evidence records object store replication.
ResidencyRow-018: Residency evidence records database replica.
ResidencyRow-019: Residency evidence records key management location.
ResidencyRow-020: Residency evidence records secret location.
ResidencyRow-021: Residency evidence records support access geography.
ResidencyRow-022: Residency evidence records admin session geography.
ResidencyRow-023: Residency evidence records incident evidence geography.
ResidencyRow-024: Residency evidence records regulator export geography.
ResidencyRow-025: Residency evidence records exception.
ResidencyRow-026: Residency evidence records exception approver.
ResidencyRow-027: Residency evidence records exception expiry.
ResidencyRow-028: Residency evidence records exception revocation.
ResidencyRow-029: Residency evidence records customer approval.
ResidencyRow-030: Residency evidence records counsel review.
ResidencyRow-031: APP8 evidence records APP entity status.
ResidencyRow-032: APP8 evidence records personal information status.
ResidencyRow-033: APP8 evidence records held information status.
ResidencyRow-034: APP8 evidence records recipient country.
ResidencyRow-035: APP8 evidence records recipient identity.
ResidencyRow-036: APP8 evidence records same entity analysis.
ResidencyRow-037: APP8 evidence records related body corporate analysis.
ResidencyRow-038: APP8 evidence records overseas recipient analysis.
ResidencyRow-039: APP8 evidence records disclosure analysis.
ResidencyRow-040: APP8 evidence records use analysis.
ResidencyRow-041: APP8 evidence records route-only analysis.
ResidencyRow-042: APP8 evidence records effective control release.
ResidencyRow-043: APP8 evidence records contractor role.
ResidencyRow-044: APP8 evidence records processor role.
ResidencyRow-045: APP8 evidence records subprocessor role.
ResidencyRow-046: APP8 evidence records APP 6 purpose check.
ResidencyRow-047: APP8 evidence records primary purpose.
ResidencyRow-048: APP8 evidence records secondary purpose.
ResidencyRow-049: APP8 evidence records consent purpose.
ResidencyRow-050: APP8 evidence records legal authority.
ResidencyRow-051: Reasonable steps evidence records contract terms.
ResidencyRow-052: Reasonable steps evidence records privacy obligations.
ResidencyRow-053: Reasonable steps evidence records security controls.
ResidencyRow-054: Reasonable steps evidence records encryption.
ResidencyRow-055: Reasonable steps evidence records access controls.
ResidencyRow-056: Reasonable steps evidence records audit rights.
ResidencyRow-057: Reasonable steps evidence records incident notice.
ResidencyRow-058: Reasonable steps evidence records onward transfer controls.
ResidencyRow-059: Reasonable steps evidence records subprocessor controls.
ResidencyRow-060: Reasonable steps evidence records deletion duties.
ResidencyRow-061: Reasonable steps evidence records return duties.
ResidencyRow-062: Reasonable steps evidence records retention controls.
ResidencyRow-063: Reasonable steps evidence records training controls.
ResidencyRow-064: Reasonable steps evidence records monitoring controls.
ResidencyRow-065: Reasonable steps evidence records enforcement controls.
ResidencyRow-066: Similar law evidence records country.
ResidencyRow-067: Similar law evidence records law.
ResidencyRow-068: Similar law evidence records binding scheme.
ResidencyRow-069: Similar law evidence records enforceability.
ResidencyRow-070: Similar law evidence records individual redress.
ResidencyRow-071: Similar law evidence records counsel review.
ResidencyRow-072: Similar law evidence records expiry review.
ResidencyRow-073: Similar law evidence records limitation.
ResidencyRow-074: Similar law evidence records residual risk.
ResidencyRow-075: Similar law evidence records final decision.
ResidencyRow-076: Informed consent evidence records warning text.
ResidencyRow-077: Informed consent evidence records no Privacy Act accountability warning.
ResidencyRow-078: Informed consent evidence records redress limitation warning.
ResidencyRow-079: Informed consent evidence records foreign law access risk.
ResidencyRow-080: Informed consent evidence records recipient.
ResidencyRow-081: Informed consent evidence records country.
ResidencyRow-082: Informed consent evidence records data class.
ResidencyRow-083: Informed consent evidence records purpose.
ResidencyRow-084: Informed consent evidence records current consent.
ResidencyRow-085: Informed consent evidence records withdrawal.
ResidencyRow-086: Australian law evidence records authority.
ResidencyRow-087: Australian law evidence records section.
ResidencyRow-088: Australian law evidence records order.
ResidencyRow-089: Australian law evidence records scope.
ResidencyRow-090: Australian law evidence records recipient.
ResidencyRow-091: Australian law evidence records data class.
ResidencyRow-092: Australian law evidence records purpose.
ResidencyRow-093: Australian law evidence records expiration.
ResidencyRow-094: Australian law evidence records reviewer.
ResidencyRow-095: Australian law evidence records final decision.
ResidencyRow-096: Section16C evidence records accountability active.
ResidencyRow-097: Section16C evidence records exception active.
ResidencyRow-098: Section16C evidence records overseas act risk.
ResidencyRow-099: Section16C evidence records recipient breach handling.
ResidencyRow-100: Section16C evidence records audit linkage.
ResidencyRow-101: Transfer class evidence records contact data.
ResidencyRow-102: Transfer class evidence records authentication data.
ResidencyRow-103: Transfer class evidence records health data.
ResidencyRow-104: Transfer class evidence records My Health Record data.
ResidencyRow-105: Transfer class evidence records government identifier data.
ResidencyRow-106: Transfer class evidence records financial transaction data.
ResidencyRow-107: Transfer class evidence records APRA telemetry.
ResidencyRow-108: Transfer class evidence records securities advice data.
ResidencyRow-109: Transfer class evidence records practitioner registration data.
ResidencyRow-110: Transfer class evidence records complaint data.
ResidencyRow-111: Transfer class evidence records support transcript.
ResidencyRow-112: Transfer class evidence records marketing audience.
ResidencyRow-113: Transfer class evidence records analytics event.
ResidencyRow-114: Transfer class evidence records deidentified data.
ResidencyRow-115: Transfer class evidence records aggregate data.
ResidencyRow-116: Transfer class evidence records audit data.
ResidencyRow-117: Transfer class evidence records regulator export.
ResidencyRow-118: Transfer class evidence records backup data.
ResidencyRow-119: Transfer class evidence records log data.
ResidencyRow-120: Transfer class evidence records derived profile.
ResidencyRow-121: Policy evidence records `au.residency.profile_required`.
ResidencyRow-122: Policy evidence records `au.residency.storage_region`.
ResidencyRow-123: Policy evidence records `au.residency.backup_region`.
ResidencyRow-124: Policy evidence records `au.residency.log_region`.
ResidencyRow-125: Policy evidence records `au.residency.analytics_region`.
ResidencyRow-126: Policy evidence records `au.residency.key_region`.
ResidencyRow-127: Policy evidence records `au.residency.support_access`.
ResidencyRow-128: Policy evidence records `au.cross_border.reasonable_steps`.
ResidencyRow-129: Policy evidence records `au.cross_border.express_informed_consent`.
ResidencyRow-130: Policy evidence records `au.cross_border.section16c_accountability`.
ResidencyRow-131: API evidence records residency profile creation.
ResidencyRow-132: API evidence records placement evaluation.
ResidencyRow-133: API evidence records residency exception.
ResidencyRow-134: API evidence records residency export.
ResidencyRow-135: API evidence records APP 8 assessment.
ResidencyRow-136: API evidence records disclosure classification.
ResidencyRow-137: API evidence records reasonable steps.
ResidencyRow-138: API evidence records similar law.
ResidencyRow-139: API evidence records informed consent.
ResidencyRow-140: API evidence records Australian law authority.
ResidencyRow-141: Audit evidence records `AuResidencyProfileCreated`.
ResidencyRow-142: Audit evidence records `AuResidencyPlacementAllowed`.
ResidencyRow-143: Audit evidence records `AuResidencyPlacementDenied`.
ResidencyRow-144: Audit evidence records `AuCrossBorderAssessmentStarted`.
ResidencyRow-145: Audit evidence records `AuDisclosureUseClassified`.
ResidencyRow-146: Audit evidence records `AuReasonableStepsRecorded`.
ResidencyRow-147: Audit evidence records `AuInformedConsentWarningServed`.
ResidencyRow-148: Audit evidence records `AuSection16cAccountabilityRecorded`.
ResidencyRow-149: Audit evidence records `AuCrossBorderDisclosureAllowed`.
ResidencyRow-150: Audit evidence records `AuCrossBorderDisclosureDenied`.
ResidencyRow-151: Failure evidence records missing residency profile.
ResidencyRow-152: Failure evidence records disallowed storage region.
ResidencyRow-153: Failure evidence records disallowed backup region.
ResidencyRow-154: Failure evidence records disallowed log region.
ResidencyRow-155: Failure evidence records disallowed support geography.
ResidencyRow-156: Failure evidence records unknown APP entity.
ResidencyRow-157: Failure evidence records unknown personal-information status.
ResidencyRow-158: Failure evidence records unknown recipient country.
ResidencyRow-159: Failure evidence records unknown disclosure classification.
ResidencyRow-160: Failure evidence records missing APP 6 purpose.
ResidencyRow-161: Failure evidence records missing reasonable steps.
ResidencyRow-162: Failure evidence records weak similar-law evidence.
ResidencyRow-163: Failure evidence records missing express warning.
ResidencyRow-164: Failure evidence records withdrawn consent.
ResidencyRow-165: Failure evidence records non-Australian law authority.
ResidencyRow-166: Failure evidence records missing section 16C state.
ResidencyRow-167: Failure evidence records missing onward transfer control.
ResidencyRow-168: Failure evidence records missing foreign-law risk warning.
ResidencyRow-169: Failure evidence records emergency without approval.
ResidencyRow-170: Failure evidence records missing audit event.
ResidencyRow-171: Source evidence records Privacy Act URL.
ResidencyRow-172: Source evidence records OAIC APP 8 URL.
ResidencyRow-173: Source evidence records OAIC APP text URL.
ResidencyRow-174: Source evidence records APRA CPS 234 URL.
ResidencyRow-175: Source evidence records My Health Record breach URL.
ResidencyRow-176: Source evidence records APP 8 citation.
ResidencyRow-177: Source evidence records section 16C citation.
ResidencyRow-178: Source evidence records APP 11 citation.
ResidencyRow-179: Source evidence records CPS 234 citation.
ResidencyRow-180: Source evidence records MHR citation.
ResidencyRow-181: Worked evidence records Sydney storage scenario.
ResidencyRow-182: Worked evidence records Singapore backup denial scenario.
ResidencyRow-183: Worked evidence records offshore support scenario.
ResidencyRow-184: Worked evidence records route-only transit scenario.
ResidencyRow-185: Worked evidence records overseas subsidiary scenario.
ResidencyRow-186: Worked evidence records processor safeguards scenario.
ResidencyRow-187: Worked evidence records informed consent scenario.
ResidencyRow-188: Worked evidence records consent withdrawal scenario.
ResidencyRow-189: Worked evidence records Australian court order scenario.
ResidencyRow-190: Worked evidence records foreign subpoena scenario.
ResidencyRow-191: Worked evidence records APRA supplier scenario.
ResidencyRow-192: Worked evidence records MHR analytics denial scenario.
ResidencyRow-193: Release evidence records frontmatter.
ResidencyRow-194: Release evidence records required sections.
ResidencyRow-195: Release evidence records line count.
ResidencyRow-196: Release evidence records source URLs.
ResidencyRow-197: Release evidence records no other geography.
ResidencyRow-198: Release evidence records no generated script.
ResidencyRow-199: Release evidence records Australia scope.
ResidencyRow-200: Release evidence records final verification.
ResidencyRow-201: Handoff evidence records residency owner.
ResidencyRow-202: Handoff evidence records privacy owner.
ResidencyRow-203: Handoff evidence records infrastructure owner.
ResidencyRow-204: Handoff evidence records security owner.
ResidencyRow-205: Handoff evidence records compliance owner.
ResidencyRow-206: Handoff evidence records health owner.
ResidencyRow-207: Handoff evidence records prudential owner.
ResidencyRow-208: Handoff evidence records audit owner.
ResidencyRow-209: Handoff evidence records contract owner.
ResidencyRow-210: Handoff evidence records counsel owner.
ResidencyRow-211: Final evidence records tenant.
ResidencyRow-212: Final evidence records workload.
ResidencyRow-213: Final evidence records recipient.
ResidencyRow-214: Final evidence records data class.
ResidencyRow-215: Final evidence records policy.
ResidencyRow-216: Final evidence records citation.
ResidencyRow-217: Final evidence records decision.
ResidencyRow-218: Final evidence records reason.
ResidencyRow-219: Final evidence records hash.
ResidencyRow-220: Final evidence records timestamp.
ResidencyRow-221: Final evidence records residency claim is explicit.
ResidencyRow-222: Final evidence records APP 8 is not residency.
ResidencyRow-223: Final evidence records transit is not automatic disclosure.
ResidencyRow-224: Final evidence records related-body review.
ResidencyRow-225: Final evidence records contractor review.
ResidencyRow-226: Final evidence records recipient review.
ResidencyRow-227: Final evidence records safeguard review.
ResidencyRow-228: Final evidence records exception review.
ResidencyRow-229: Final evidence records accountability review.
ResidencyRow-230: Final evidence records closure review.
ResidencyRow-231: Final evidence records official authority grounding.
ResidencyRow-232: Final evidence records section-aware citation.
ResidencyRow-233: Final evidence records source-hint inclusion.
ResidencyRow-234: Final evidence records bespoke transfer matrix.
ResidencyRow-235: Final evidence records deny-first policy.
ResidencyRow-236: Final evidence records review-required ambiguity.
ResidencyRow-237: Final evidence records audit minimisation.
ResidencyRow-238: Final evidence records pack isolation.
ResidencyRow-239: Final evidence records source list.
ResidencyRow-240: Final evidence records line threshold.
ResidencyRow-241: Final evidence records field delta.
ResidencyRow-242: Final evidence records endpoint delta.
ResidencyRow-243: Final evidence records event delta.
ResidencyRow-244: Final evidence records failure delta.
ResidencyRow-245: Final evidence records example delta.
ResidencyRow-246: Final evidence records cross-reference delta.
ResidencyRow-247: Final evidence records final source refresh.
ResidencyRow-248: Final evidence records final counsel marker.
ResidencyRow-249: Final evidence records final status.
ResidencyRow-250: Final evidence records final handoff.
