---
doc_class: LocalizationPack
pack_id: AU-PACK-1
doc_id: AU-PACK-1-CONSENT-AND-RIGHTS
title: Australia Consent and Data Subject Rights
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
  - https://www.oaic.gov.au/privacy/australian-privacy-principles/read-the-australian-privacy-principles
  - https://www.oaic.gov.au/privacy/australian-privacy-principles/australian-privacy-principles-guidelines/chapter-8-app-8-cross-border-disclosure-of-personal-information
  - https://www.ahpra.gov.au/About-AHPRA/Privacy.aspx
---

# Australia Consent and Data Subject Rights

This document maps Australian consent, notice, access, and correction controls.
It uses APP 2, APP 3, APP 5, APP 6, APP 7, APP 8, APP 12, and APP 13 as the rights surface.
It treats Australian rights as APP rights, not GDPR rights renamed.
It treats consent as one basis among several APP pathways, not as the only lawful basis.
It treats sensitive information consent as higher risk than ordinary contact-data consent.
It treats APP 8 express informed consent as a special cross-border exception with mandatory warnings.
It treats Ahpra practitioner data as regulator-context personal information requiring purpose controls.

## Authority Citations

Citation-001: Privacy Act Schedule 1 APP 2 anchors anonymity and pseudonymity.
Citation-002: Privacy Act Schedule 1 APP 3 anchors collection and sensitive information consent.
Citation-003: Privacy Act Schedule 1 APP 5 anchors notification of collection.
Citation-004: Privacy Act Schedule 1 APP 6 anchors use and disclosure limits.
Citation-005: Privacy Act Schedule 1 APP 7 anchors direct marketing rules.
Citation-006: Privacy Act Schedule 1 APP 8 anchors cross-border disclosure consent exception.
Citation-007: Privacy Act Schedule 1 APP 12 anchors access rights.
Citation-008: Privacy Act Schedule 1 APP 13 anchors correction rights.
Citation-009: OAIC APP 8 guidance anchors express informed consent warning content for overseas disclosure.
Citation-010: Ahpra privacy page anchors health-practitioner regulator personal-information context.

## Rights Model

Rights-001: Australia access workflow is APP 12 access, not GDPR subject access.
Rights-002: Australia correction workflow is APP 13 correction, not GDPR rectification.
Rights-003: Australia deletion workflow is generally APP 11 destruction/de-identification plus retention policy.
Rights-004: Australia marketing opt-out workflow is APP 7 opt-out.
Rights-005: Australia anonymity workflow is APP 2 feasibility.
Rights-006: Australia collection notice workflow is APP 5.
Rights-007: Australia sensitive-information collection workflow is APP 3.
Rights-008: Australia purpose compatibility workflow is APP 6.
Rights-009: Australia overseas disclosure consent workflow is APP 8.2(b).
Rights-010: Australia complaint workflow is APP 1 governance evidence.
Rights-011: Rights workflows require tenant scope.
Rights-012: Rights workflows require identity assurance proportional to risk.
Rights-013: Rights workflows require data minimisation.
Rights-014: Rights workflows require legal hold conflict checks.
Rights-015: Rights workflows require regulator-report conflict checks.
Rights-016: Rights workflows require safety and fraud conflict checks.
Rights-017: Rights workflows require third-party personal information redaction where applicable.
Rights-018: Rights workflows require response evidence.
Rights-019: Rights workflows require refusal reason where refused.
Rights-020: Rights workflows require ADR-0263 audit event emission.

## Consent Model

Consent-001: Consent record must identify individual.
Consent-002: Consent record must identify tenant.
Consent-003: Consent record must identify purpose.
Consent-004: Consent record must identify data class.
Consent-005: Consent record must identify collection channel.
Consent-006: Consent record must identify notice version.
Consent-007: Consent record must identify consent text.
Consent-008: Consent record must identify timestamp.
Consent-009: Consent record must identify actor or user action.
Consent-010: Consent record must identify withdrawal route.
Consent-011: Consent for sensitive information must be active before collection unless exception applies.
Consent-012: Consent for direct marketing must be linked to opt-out controls.
Consent-013: Consent for overseas disclosure under APP 8.2(b) must include express warning.
Consent-014: Consent for overseas disclosure must not be bundled with ordinary terms.
Consent-015: Consent must be current.
Consent-016: Consent must be specific.
Consent-017: Consent must be voluntary.
Consent-018: Consent must be adequately informed.
Consent-019: Consent withdrawal must prevent future reliance.
Consent-020: Consent withdrawal must not erase historic audit evidence.
Consent-021: Consent renewal must create new version.
Consent-022: Consent scope expansion requires new consent.
Consent-023: Consent text change requires new notice linkage.
Consent-024: Consent for minors or capacity concerns requires separate review.
Consent-025: Consent evidence must not include unnecessary sensitive content in audit stream.

## Notice Model

Notice-001: APP 5 notice must identify the collecting entity.
Notice-002: APP 5 notice must explain collection circumstances.
Notice-003: APP 5 notice must state whether collection is required or authorized by law where applicable.
Notice-004: APP 5 notice must state purposes of collection.
Notice-005: APP 5 notice must state consequences if information is not collected where applicable.
Notice-006: APP 5 notice must state usual disclosures.
Notice-007: APP 5 notice must state privacy policy access.
Notice-008: APP 5 notice must state complaint process.
Notice-009: APP 5 notice must state overseas disclosure likelihood where applicable.
Notice-010: APP 5 notice must state countries where practicable.
Notice-011: Notice version must be immutable.
Notice-012: Notice version must be linked to collection event.
Notice-013: Notice version must be linked to language.
Notice-014: Notice version must be linked to service surface.
Notice-015: Notice version must be linked to data class.
Notice-016: Notice version must be linked to purpose.
Notice-017: Notice version must be linked to effective date.
Notice-018: Notice version must be linked to retired date where retired.
Notice-019: Notice omission requires exception reason.
Notice-020: Notice evidence must be exportable.

## Activated Cedar Policies

Cedar-001: `au.rights.anonymous_option` checks APP 2 anonymous access.
Cedar-002: `au.rights.pseudonymous_option` checks APP 2 pseudonymous access.
Cedar-003: `au.consent.sensitive_information` denies APP 3 sensitive collection without basis.
Cedar-004: `au.notice.collection_required` denies collection without APP 5 notice evidence.
Cedar-005: `au.use.purpose_compatible` denies APP 6 incompatible use.
Cedar-006: `au.marketing.optout_required` denies APP 7 direct marketing without opt-out.
Cedar-007: `au.marketing.suppressed` denies marketing to opted-out subject.
Cedar-008: `au.cross_border.express_consent_warning` denies APP 8 consent exception without warning.
Cedar-009: `au.access.identity_verified` denies APP 12 export without identity proof.
Cedar-010: `au.access.redaction_required` denies APP 12 export with unreviewed third-party data.
Cedar-011: `au.correction.request_opened` requires APP 13 workflow for correction request.
Cedar-012: `au.correction.refusal_reason` denies closure when refusal reason is missing.
Cedar-013: `au.rights.legal_hold_conflict` routes conflicting requests to review.
Cedar-014: `au.rights.regulator_report_conflict` routes AUSTRAC/APRA/ASIC conflicts to review.
Cedar-015: `au.rights.health_context_review` routes health and Ahpra rights requests to review where sensitive.

## Data Model Deltas

Data-001: `au_consent_id` identifies consent record.
Data-002: `au_consent_kind` stores sensitive_collection, marketing, overseas_disclosure, other.
Data-003: `au_consent_status` stores granted, withdrawn, expired, superseded, disputed.
Data-004: `au_consent_notice_version_id` stores linked APP 5 notice.
Data-005: `au_consent_purpose_code` stores purpose.
Data-006: `au_consent_data_class` stores data class.
Data-007: `au_consent_warning_id` stores APP 8 express warning where applicable.
Data-008: `au_consent_withdrawn_at` stores withdrawal time.
Data-009: `au_anonymity_review_id` stores APP 2 decision.
Data-010: `au_notice_version_id` stores APP 5 notice version.
Data-011: `au_access_request_id` stores APP 12 request.
Data-012: `au_access_request_status` stores open, verifying, fulfilled, refused, closed.
Data-013: `au_access_identity_evidence_id` stores identity proof artifact.
Data-014: `au_access_refusal_reason` stores refusal reason.
Data-015: `au_correction_request_id` stores APP 13 request.
Data-016: `au_correction_status` stores accepted, corrected, refused, annotated, closed.
Data-017: `au_correction_refusal_reason` stores refusal reason.
Data-018: `au_annotation_request_id` stores statement attachment request.
Data-019: `au_rights_conflict_reason` stores legal hold, regulator report, safety, fraud, third_party.
Data-020: `au_rights_response_evidence_id` stores response artifact.

## API Contract Deltas

API-001: `POST /privacy/au/notices` creates APP 5 notice version.
API-002: `GET /privacy/au/notices/{id}` returns immutable notice.
API-003: `POST /privacy/au/consents` records consent.
API-004: `POST /privacy/au/consents/{id}/withdraw` records withdrawal.
API-005: `GET /privacy/au/consents/effective` returns active consent by purpose and data class.
API-006: `POST /privacy/au/anonymity-reviews` records APP 2 decision.
API-007: `POST /privacy/au/access-requests` opens APP 12 request.
API-008: `POST /privacy/au/access-requests/{id}/verify` records identity verification.
API-009: `POST /privacy/au/access-requests/{id}/export` generates access export.
API-010: `POST /privacy/au/access-requests/{id}/refuse` records refusal.
API-011: `POST /privacy/au/correction-requests` opens APP 13 request.
API-012: `POST /privacy/au/correction-requests/{id}/apply` applies correction.
API-013: `POST /privacy/au/correction-requests/{id}/refuse` records refusal.
API-014: `POST /privacy/au/correction-requests/{id}/annotate` records annotation.
API-015: `GET /privacy/au/rights/{subject_id}/timeline` returns rights event timeline.

## Audit Event Additions (per ADR-0263)

Audit-001: `AuNoticeVersionCreated` records APP 5 notice.
Audit-002: `AuNoticeServed` records collection-time notice.
Audit-003: `AuConsentGranted` records consent purpose and data class.
Audit-004: `AuConsentWithdrawn` records withdrawal.
Audit-005: `AuSensitiveConsentChecked` records APP 3 sensitive consent decision.
Audit-006: `AuMarketingOptOutRecorded` records APP 7 opt-out.
Audit-007: `AuMarketingSuppressionApplied` records suppression enforcement.
Audit-008: `AuApp8WarningServed` records overseas disclosure warning.
Audit-009: `AuAnonymityReviewCompleted` records APP 2 decision.
Audit-010: `AuAccessRequestOpened` records APP 12 intake.
Audit-011: `AuAccessIdentityVerified` records verification result.
Audit-012: `AuAccessExportGenerated` records export hash.
Audit-013: `AuAccessRequestRefused` records refusal reason.
Audit-014: `AuCorrectionRequestOpened` records APP 13 intake.
Audit-015: `AuCorrectionApplied` records corrected fields.
Audit-016: `AuCorrectionRequestRefused` records refusal reason.
Audit-017: `AuCorrectionAnnotationApplied` records annotation.
Audit-018: `AuRightsConflictDetected` records conflict class.
Audit-019: `AuRightsResponseSent` records response artifact.
Audit-020: `AuRightsCaseClosed` records closure reason.

## Failure Modes

Failure-001: Sensitive collection without consent or exception is denied.
Failure-002: Collection without notice is denied.
Failure-003: Notice version mutable after service is denied.
Failure-004: Consent without purpose is denied.
Failure-005: Consent without data class is denied.
Failure-006: Consent without withdrawal path is review-required.
Failure-007: APP 8 consent without express warning is denied.
Failure-008: Withdrawn consent used for new processing is denied.
Failure-009: Marketing to opted-out individual is denied.
Failure-010: Access request without identity proof is denied.
Failure-011: Access export with third-party personal information is blocked pending redaction.
Failure-012: Access refusal without reason is denied.
Failure-013: Correction request closure without decision is denied.
Failure-014: Correction refusal without reason is denied.
Failure-015: Annotation request ignored after refusal is review-required.
Failure-016: Legal hold conflict without review is denied.
Failure-017: AUSTRAC report conflict without compartmented review is denied.
Failure-018: Health data rights request without sensitive review is review-required.
Failure-019: Ahpra practitioner request mixed with patient data is review-required.
Failure-020: Rights response without audit event is invalid.

## Worked Examples

Example-001: User signs up with email; APP 5 notice is served and APP 3 purpose is recorded.
Example-002: User provides health information; sensitive consent or exception is required.
Example-003: User browses public help; APP 2 review says accountless access is practicable.
Example-004: User asks for profile export; APP 12 workflow verifies identity before export.
Example-005: User asks to correct phone number; APP 13 workflow applies correction.
Example-006: User asks to correct disputed fraud marker; review may refuse correction and attach annotation.
Example-007: User opts out of marketing; APP 7 suppression applies to future campaigns.
Example-008: User consents to overseas processor after express warning; APP 8.2(b) record is created.
Example-009: User withdraws overseas transfer consent; future reliance on consent is blocked.
Example-010: AUSTRAC suspicious matter record is requested by subject; regulator-report conflict routes to review.
Example-011: APRA incident evidence includes employee data; access request redacts third-party details.
Example-012: Ahpra practitioner asks for data held in complaint context; purpose and regulator constraints are reviewed.
Example-013: Patient data in My Health Record context triggers health-specific review before export.
Example-014: Privacy notice changes; new collection events use new notice version while old evidence remains immutable.
Example-015: Legal hold exists; destruction request is refused with reason and audit event.

## Cross-References

CrossRef-001: `README.md` defines AU-PACK-1 rights scope.
CrossRef-002: `regulatory-coverage.md` maps APP rights to control ids.
CrossRef-003: `data-residency-and-cross-border.md` uses consent for APP 8.2(b).
CrossRef-004: `breach-notification-and-incident-response.md` defines incident communications distinct from rights responses.
CrossRef-005: `sectoral-overlays.md` defines AUSTRAC, APRA, ASIC, MHR, and Ahpra conflicts.
CrossRef-006: ADR-0243 defines Cedar policy gates.
CrossRef-007: ADR-0244 defines tenant scoping for rights.
CrossRef-008: ADR-0251 defines compliance pack activation.
CrossRef-009: ADR-0263 defines audit event envelopes.

## APP 12 Access Checklist

Access-001: Receive request.
Access-002: Record subject.
Access-003: Record tenant.
Access-004: Verify identity.
Access-005: Scope systems.
Access-006: Scope date range.
Access-007: Scope data classes.
Access-008: Check legal hold.
Access-009: Check regulator report conflict.
Access-010: Check safety conflict.
Access-011: Check fraud conflict.
Access-012: Check third-party personal information.
Access-013: Check confidential commercial information.
Access-014: Check health sensitivity.
Access-015: Check practitioner complaint context.
Access-016: Prepare export.
Access-017: Redact blocked content.
Access-018: Attach explanation.
Access-019: Record refusal if refused.
Access-020: Record partial refusal if partially refused.
Access-021: Record delivery channel.
Access-022: Record response date.
Access-023: Record export hash.
Access-024: Emit audit event.
Access-025: Close request.

## APP 13 Correction Checklist

Correction-001: Receive request.
Correction-002: Record disputed field.
Correction-003: Record requested value.
Correction-004: Record evidence from individual.
Correction-005: Verify identity.
Correction-006: Locate source system.
Correction-007: Locate downstream replicas.
Correction-008: Check authoritative source.
Correction-009: Check legal hold conflict.
Correction-010: Check regulator report conflict.
Correction-011: Check fraud conflict.
Correction-012: Check health sensitivity.
Correction-013: Decide correction.
Correction-014: Apply correction if accepted.
Correction-015: Propagate correction where appropriate.
Correction-016: Refuse with reason if not accepted.
Correction-017: Offer annotation where required.
Correction-018: Record response artifact.
Correction-019: Emit audit event.
Correction-020: Close request.

## Consent Evidence Checklist

ConsentCheck-001: Purpose is specific.
ConsentCheck-002: Data class is specific.
ConsentCheck-003: Notice version is attached.
ConsentCheck-004: User action is captured.
ConsentCheck-005: Timestamp is captured.
ConsentCheck-006: Withdrawal route is available.
ConsentCheck-007: Consent is not bundled beyond purpose.
ConsentCheck-008: Consent text is retained.
ConsentCheck-009: Consent locale is retained.
ConsentCheck-010: Consent screen version is retained.
ConsentCheck-011: Sensitive information flag is checked.
ConsentCheck-012: Cross-border warning is checked where relevant.
ConsentCheck-013: Marketing opt-out is checked where relevant.
ConsentCheck-014: Capacity concern is checked where relevant.
ConsentCheck-015: Consent expiry is checked where relevant.
ConsentCheck-016: Consent supersession is tracked.
ConsentCheck-017: Withdrawal is effective prospectively.
ConsentCheck-018: Audit event is emitted.
ConsentCheck-019: Evidence hash is stored.
ConsentCheck-020: Policy id is stored.

## Notice Evidence Checklist

NoticeCheck-001: Entity identity is present.
NoticeCheck-002: Collection purpose is present.
NoticeCheck-003: Legal authority is present where applicable.
NoticeCheck-004: Required-or-optional consequence is present where applicable.
NoticeCheck-005: Usual disclosures are present.
NoticeCheck-006: Overseas disclosure note is present where applicable.
NoticeCheck-007: Privacy policy reference is present.
NoticeCheck-008: Complaint process is present.
NoticeCheck-009: Contact channel is present.
NoticeCheck-010: Effective date is present.
NoticeCheck-011: Version id is present.
NoticeCheck-012: Product surface is present.
NoticeCheck-013: Data class is present.
NoticeCheck-014: Locale is present.
NoticeCheck-015: Rendering evidence is present.
NoticeCheck-016: Collection event linkage is present.
NoticeCheck-017: Retirement status is present.
NoticeCheck-018: Exception reason is present if notice was not served.
NoticeCheck-019: Audit event is present.
NoticeCheck-020: Evidence export is available.

## Rights Evidence Rows

RightsRow-001: Rights evidence records APP 2 anonymous option.
RightsRow-002: Rights evidence records APP 2 pseudonymous option.
RightsRow-003: Rights evidence records identity-required reason.
RightsRow-004: Rights evidence records accountless journey.
RightsRow-005: Rights evidence records service necessity.
RightsRow-006: Rights evidence records APP 3 collection purpose.
RightsRow-007: Rights evidence records APP 3 collection source.
RightsRow-008: Rights evidence records APP 3 sensitive consent.
RightsRow-009: Rights evidence records APP 3 health information flag.
RightsRow-010: Rights evidence records APP 3 direct collection review.
RightsRow-011: Rights evidence records APP 5 notice id.
RightsRow-012: Rights evidence records APP 5 notice text.
RightsRow-013: Rights evidence records APP 5 notice surface.
RightsRow-014: Rights evidence records APP 5 notice language.
RightsRow-015: Rights evidence records APP 5 notice timestamp.
RightsRow-016: Rights evidence records APP 5 entity identity.
RightsRow-017: Rights evidence records APP 5 collection purpose.
RightsRow-018: Rights evidence records APP 5 usual disclosures.
RightsRow-019: Rights evidence records APP 5 complaint path.
RightsRow-020: Rights evidence records APP 5 overseas disclosure note.
RightsRow-021: Rights evidence records APP 6 primary purpose.
RightsRow-022: Rights evidence records APP 6 secondary purpose.
RightsRow-023: Rights evidence records APP 6 consent.
RightsRow-024: Rights evidence records APP 6 legal authority.
RightsRow-025: Rights evidence records APP 6 disclosure recipient.
RightsRow-026: Rights evidence records APP 7 marketing channel.
RightsRow-027: Rights evidence records APP 7 consent basis.
RightsRow-028: Rights evidence records APP 7 opt-out path.
RightsRow-029: Rights evidence records APP 7 suppression list.
RightsRow-030: Rights evidence records APP 7 vendor export gate.
RightsRow-031: Rights evidence records APP 8 express warning.
RightsRow-032: Rights evidence records APP 8 overseas recipient.
RightsRow-033: Rights evidence records APP 8 consent scope.
RightsRow-034: Rights evidence records APP 8 consent withdrawal.
RightsRow-035: Rights evidence records APP 8 redress warning.
RightsRow-036: Rights evidence records APP 12 request id.
RightsRow-037: Rights evidence records APP 12 identity proof.
RightsRow-038: Rights evidence records APP 12 search scope.
RightsRow-039: Rights evidence records APP 12 export scope.
RightsRow-040: Rights evidence records APP 12 refusal reason.
RightsRow-041: Rights evidence records APP 12 third-party redaction.
RightsRow-042: Rights evidence records APP 12 legal hold conflict.
RightsRow-043: Rights evidence records APP 12 regulator conflict.
RightsRow-044: Rights evidence records APP 12 health sensitivity review.
RightsRow-045: Rights evidence records APP 12 response artifact.
RightsRow-046: Rights evidence records APP 13 request id.
RightsRow-047: Rights evidence records APP 13 disputed field.
RightsRow-048: Rights evidence records APP 13 requested value.
RightsRow-049: Rights evidence records APP 13 source system.
RightsRow-050: Rights evidence records APP 13 authoritative source.
RightsRow-051: Rights evidence records APP 13 correction decision.
RightsRow-052: Rights evidence records APP 13 refusal reason.
RightsRow-053: Rights evidence records APP 13 annotation request.
RightsRow-054: Rights evidence records APP 13 downstream propagation.
RightsRow-055: Rights evidence records APP 13 response artifact.
RightsRow-056: Consent evidence records purpose specificity.
RightsRow-057: Consent evidence records data class specificity.
RightsRow-058: Consent evidence records collection channel.
RightsRow-059: Consent evidence records user action.
RightsRow-060: Consent evidence records text version.
RightsRow-061: Consent evidence records screen version.
RightsRow-062: Consent evidence records locale.
RightsRow-063: Consent evidence records timestamp.
RightsRow-064: Consent evidence records withdrawal link.
RightsRow-065: Consent evidence records withdrawal timestamp.
RightsRow-066: Consent evidence records expiry.
RightsRow-067: Consent evidence records supersession.
RightsRow-068: Consent evidence records dispute.
RightsRow-069: Consent evidence records capacity review.
RightsRow-070: Consent evidence records guardian review.
RightsRow-071: Notice evidence records immutable version.
RightsRow-072: Notice evidence records retired version.
RightsRow-073: Notice evidence records product surface.
RightsRow-074: Notice evidence records data class.
RightsRow-075: Notice evidence records purpose code.
RightsRow-076: Notice evidence records rendering proof.
RightsRow-077: Notice evidence records delivery proof.
RightsRow-078: Notice evidence records omission exception.
RightsRow-079: Notice evidence records complaint contact.
RightsRow-080: Notice evidence records privacy policy URL.
RightsRow-081: Access evidence records identity assurance level.
RightsRow-082: Access evidence records request channel.
RightsRow-083: Access evidence records deadline.
RightsRow-084: Access evidence records extension reason.
RightsRow-085: Access evidence records export format.
RightsRow-086: Access evidence records export hash.
RightsRow-087: Access evidence records delivery channel.
RightsRow-088: Access evidence records partial refusal.
RightsRow-089: Access evidence records full refusal.
RightsRow-090: Access evidence records closure.
RightsRow-091: Correction evidence records intake channel.
RightsRow-092: Correction evidence records evidence provided.
RightsRow-093: Correction evidence records field owner.
RightsRow-094: Correction evidence records correction applied.
RightsRow-095: Correction evidence records correction rejected.
RightsRow-096: Correction evidence records annotation applied.
RightsRow-097: Correction evidence records downstream notice.
RightsRow-098: Correction evidence records rollback guard.
RightsRow-099: Correction evidence records closure.
RightsRow-100: Correction evidence records response hash.
RightsRow-101: Conflict evidence records legal hold.
RightsRow-102: Conflict evidence records AUSTRAC restricted report.
RightsRow-103: Conflict evidence records APRA incident evidence.
RightsRow-104: Conflict evidence records ASIC remediation evidence.
RightsRow-105: Conflict evidence records My Health Record data.
RightsRow-106: Conflict evidence records Ahpra practitioner data.
RightsRow-107: Conflict evidence records third-party data.
RightsRow-108: Conflict evidence records safety risk.
RightsRow-109: Conflict evidence records fraud risk.
RightsRow-110: Conflict evidence records confidentiality risk.
RightsRow-111: Health rights evidence records health information flag.
RightsRow-112: Health rights evidence records sensitive consent.
RightsRow-113: Health rights evidence records patient access route.
RightsRow-114: Health rights evidence records practitioner route.
RightsRow-115: Health rights evidence records MHR conflict.
RightsRow-116: Practitioner rights evidence records Ahpra registration hash.
RightsRow-117: Practitioner rights evidence records board.
RightsRow-118: Practitioner rights evidence records public register source.
RightsRow-119: Practitioner rights evidence records complaint source.
RightsRow-120: Practitioner rights evidence records notification source.
RightsRow-121: Marketing evidence records audience id.
RightsRow-122: Marketing evidence records campaign id.
RightsRow-123: Marketing evidence records opt-out state.
RightsRow-124: Marketing evidence records suppression proof.
RightsRow-125: Marketing evidence records vendor proof.
RightsRow-126: Cross-border consent evidence records recipient.
RightsRow-127: Cross-border consent evidence records country.
RightsRow-128: Cross-border consent evidence records warning.
RightsRow-129: Cross-border consent evidence records consequence.
RightsRow-130: Cross-border consent evidence records withdrawal.
RightsRow-131: Audit evidence records `AuNoticeServed`.
RightsRow-132: Audit evidence records `AuConsentGranted`.
RightsRow-133: Audit evidence records `AuConsentWithdrawn`.
RightsRow-134: Audit evidence records `AuAccessRequestOpened`.
RightsRow-135: Audit evidence records `AuAccessExportGenerated`.
RightsRow-136: Audit evidence records `AuAccessRequestRefused`.
RightsRow-137: Audit evidence records `AuCorrectionRequestOpened`.
RightsRow-138: Audit evidence records `AuCorrectionApplied`.
RightsRow-139: Audit evidence records `AuCorrectionRequestRefused`.
RightsRow-140: Audit evidence records `AuRightsCaseClosed`.
RightsRow-141: Policy evidence records `au.rights.anonymous_option`.
RightsRow-142: Policy evidence records `au.consent.sensitive_information`.
RightsRow-143: Policy evidence records `au.notice.collection_required`.
RightsRow-144: Policy evidence records `au.marketing.optout_required`.
RightsRow-145: Policy evidence records `au.access.identity_verified`.
RightsRow-146: Policy evidence records `au.correction.refusal_reason`.
RightsRow-147: Policy evidence records `au.rights.legal_hold_conflict`.
RightsRow-148: Policy evidence records `au.rights.regulator_report_conflict`.
RightsRow-149: Policy evidence records `au.rights.health_context_review`.
RightsRow-150: Policy evidence records `au.cross_border.express_consent_warning`.
RightsRow-151: API evidence records notice creation.
RightsRow-152: API evidence records consent creation.
RightsRow-153: API evidence records consent withdrawal.
RightsRow-154: API evidence records access intake.
RightsRow-155: API evidence records access verification.
RightsRow-156: API evidence records access export.
RightsRow-157: API evidence records access refusal.
RightsRow-158: API evidence records correction intake.
RightsRow-159: API evidence records correction apply.
RightsRow-160: API evidence records correction refusal.
RightsRow-161: API evidence records correction annotation.
RightsRow-162: API evidence records rights timeline.
RightsRow-163: Source evidence records Privacy Act URL.
RightsRow-164: Source evidence records APP text URL.
RightsRow-165: Source evidence records APP 8 URL.
RightsRow-166: Source evidence records Ahpra privacy URL.
RightsRow-167: Source evidence records APP 2 citation.
RightsRow-168: Source evidence records APP 3 citation.
RightsRow-169: Source evidence records APP 5 citation.
RightsRow-170: Source evidence records APP 6 citation.
RightsRow-171: Source evidence records APP 7 citation.
RightsRow-172: Source evidence records APP 8 citation.
RightsRow-173: Source evidence records APP 12 citation.
RightsRow-174: Source evidence records APP 13 citation.
RightsRow-175: Release evidence records frontmatter.
RightsRow-176: Release evidence records required sections.
RightsRow-177: Release evidence records line count.
RightsRow-178: Release evidence records Australia scope.
RightsRow-179: Release evidence records no other geography.
RightsRow-180: Release evidence records final verification.
RightsRow-181: Data evidence records `au_consent_id`.
RightsRow-182: Data evidence records `au_notice_version_id`.
RightsRow-183: Data evidence records `au_access_request_id`.
RightsRow-184: Data evidence records `au_correction_request_id`.
RightsRow-185: Data evidence records `au_rights_response_evidence_id`.
RightsRow-186: Data evidence records `au_rights_conflict_reason`.
RightsRow-187: Failure evidence records missing sensitive consent.
RightsRow-188: Failure evidence records missing notice.
RightsRow-189: Failure evidence records missing purpose.
RightsRow-190: Failure evidence records missing withdrawal.
RightsRow-191: Failure evidence records missing APP 8 warning.
RightsRow-192: Failure evidence records using withdrawn consent.
RightsRow-193: Failure evidence records marketing suppression failure.
RightsRow-194: Failure evidence records missing identity proof.
RightsRow-195: Failure evidence records missing redaction.
RightsRow-196: Failure evidence records missing refusal reason.
RightsRow-197: Failure evidence records missing annotation route.
RightsRow-198: Failure evidence records missing conflict review.
RightsRow-199: Failure evidence records missing audit event.
RightsRow-200: Failure evidence records missing response artifact.
RightsRow-201: Worked evidence records signup scenario.
RightsRow-202: Worked evidence records health collection scenario.
RightsRow-203: Worked evidence records anonymous browse scenario.
RightsRow-204: Worked evidence records access export scenario.
RightsRow-205: Worked evidence records correction scenario.
RightsRow-206: Worked evidence records marketing opt-out scenario.
RightsRow-207: Worked evidence records overseas consent scenario.
RightsRow-208: Worked evidence records AUSTRAC conflict scenario.
RightsRow-209: Worked evidence records APRA redaction scenario.
RightsRow-210: Worked evidence records Ahpra complaint scenario.
RightsRow-211: Handoff evidence records privacy owner.
RightsRow-212: Handoff evidence records rights owner.
RightsRow-213: Handoff evidence records consent owner.
RightsRow-214: Handoff evidence records notice owner.
RightsRow-215: Handoff evidence records access owner.
RightsRow-216: Handoff evidence records correction owner.
RightsRow-217: Handoff evidence records marketing owner.
RightsRow-218: Handoff evidence records health owner.
RightsRow-219: Handoff evidence records practitioner owner.
RightsRow-220: Handoff evidence records audit owner.
RightsRow-221: Final evidence records tenant.
RightsRow-222: Final evidence records subject.
RightsRow-223: Final evidence records actor.
RightsRow-224: Final evidence records policy.
RightsRow-225: Final evidence records citation.
RightsRow-226: Final evidence records timestamp.
RightsRow-227: Final evidence records result.
RightsRow-228: Final evidence records reason.
RightsRow-229: Final evidence records hash.
RightsRow-230: Final evidence records closure.
RightsRow-231: Final evidence records APP rights are not GDPR renamed.
RightsRow-232: Final evidence records consent is not universal basis.
RightsRow-233: Final evidence records deletion uses APP 11 posture.
RightsRow-234: Final evidence records access uses APP 12 posture.
RightsRow-235: Final evidence records correction uses APP 13 posture.
RightsRow-236: Final evidence records marketing uses APP 7 posture.
RightsRow-237: Final evidence records collection uses APP 3 posture.
RightsRow-238: Final evidence records notice uses APP 5 posture.
RightsRow-239: Final evidence records overseas consent uses APP 8 posture.
RightsRow-240: Final evidence records complaint uses APP 1 posture.
RightsRow-241: Final evidence records official authority grounding.
RightsRow-242: Final evidence records section-aware citations.
RightsRow-243: Final evidence records required URLs.
RightsRow-244: Final evidence records bespoke rights matrix.
RightsRow-245: Final evidence records no script artifact.
RightsRow-246: Final evidence records no external geography.
RightsRow-247: Final evidence records exactly one Australia pack.
RightsRow-248: Final evidence records review-required ambiguity.
RightsRow-249: Final evidence records deny-first policy.
RightsRow-250: Final evidence records audit-minimized rights evidence.
RightsRow-251: Final evidence records line threshold.
RightsRow-252: Final evidence records source list.
RightsRow-253: Final evidence records final state.
