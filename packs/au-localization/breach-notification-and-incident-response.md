---
doc_class: LocalizationPack
pack_id: AU-PACK-1
doc_id: AU-PACK-1-BREACH-INCIDENT
title: Australia Breach Notification and Incident Response
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
  - https://www.oaic.gov.au/privacy/notifiable-data-breaches/about-the-notifiable-data-breaches-scheme
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/preventing-preparing-for-and-responding-to-data-breaches/data-breach-preparation-and-response/part-4-notifiable-data-breach-ndb-scheme
  - https://handbook.apra.gov.au/standard/cps-234
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/health-service-providers/my-health-record/guide-to-mandatory-data-breach-notification-in-the-my-health-record-system
  - https://www.austrac.gov.au/about-us/legislation/amlctf-act
---

# Australia Breach Notification and Incident Response

This document defines Australian breach and incident workflows for AU-PACK-1.
It separates generic security incidents from Privacy Act eligible data breaches.
It separates suspected eligible data breaches from confirmed eligible data breaches.
It separates NDB notification from APRA, AUSTRAC, ASIC, and My Health Record sector workflows.
It uses OAIC NDB guidance as the breach-notification baseline.
It uses OAIC Part 4 guidance for 30-day suspected breach assessment behavior.
It uses APRA CPS 234 for APRA-regulated information-security incident overlays.
It uses My Health Record breach guidance for MHR-specific notification workflows.

## Authority Citations

Citation-001: Privacy Act Part IIIC anchors the Notifiable Data Breaches scheme.
Citation-002: OAIC NDB scheme guidance says covered entities notify affected individuals and OAIC when serious harm is likely.
Citation-003: OAIC Part 4 NDB guidance anchors suspected eligible breach assessment and 30-day expectation.
Citation-004: Privacy Act APP 11 anchors reasonable security, destruction, and de-identification relevance.
Citation-005: APRA CPS 234 anchors material information-security incident notification expectations for APRA-regulated entities.
Citation-006: OAIC My Health Record breach guide anchors MHR-specific mandatory breach notification.
Citation-007: AUSTRAC AML/CTF Act guidance anchors reporting-entity confidentiality and reporting workflow separation.

## Incident Taxonomy

Taxonomy-001: `security_event` is a raw signal.
Taxonomy-002: `security_incident` is a confirmed adverse security event.
Taxonomy-003: `privacy_incident` is an event involving personal information.
Taxonomy-004: `suspected_eligible_data_breach` is a privacy incident requiring NDB assessment.
Taxonomy-005: `eligible_data_breach` is an NDB-notifiable breach.
Taxonomy-006: `mhr_breach` is a My Health Record system breach event.
Taxonomy-007: `apra_material_information_security_incident` is a CPS 234 route.
Taxonomy-008: `austrac_sensitive_reporting_event` is a compartmented AML/CTF route.
Taxonomy-009: `asic_customer_harm_incident` is a securities conduct and remediation route.
Taxonomy-010: `ahpra_practitioner_privacy_incident` is a practitioner-regulator data route.
Taxonomy-011: Incidents can have multiple classifications.
Taxonomy-012: NDB classification does not replace APRA classification.
Taxonomy-013: APRA classification does not replace NDB classification.
Taxonomy-014: My Health Record classification does not replace NDB classification.
Taxonomy-015: AUSTRAC reporting confidentiality can restrict ordinary incident visibility.

## NDB Assessment Model

NDB-001: Intake records discovery time.
NDB-002: Intake records reporter.
NDB-003: Intake records tenant.
NDB-004: Intake records affected service.
NDB-005: Intake records suspected data classes.
NDB-006: Intake records whether personal information is involved.
NDB-007: Intake records whether access was unauthorized.
NDB-008: Intake records whether disclosure was unauthorized.
NDB-009: Intake records whether information was lost.
NDB-010: Intake records whether remedial action can prevent serious harm.
NDB-011: Assessment records affected individuals or cohorts.
NDB-012: Assessment records sensitivity of information.
NDB-013: Assessment records protection measures.
NDB-014: Assessment records likelihood of misuse.
NDB-015: Assessment records potential physical harm.
NDB-016: Assessment records potential psychological harm.
NDB-017: Assessment records potential financial harm.
NDB-018: Assessment records potential reputational harm.
NDB-019: Assessment records potential identity crime risk.
NDB-020: Assessment records other serious harm factors.
NDB-021: Assessment records mitigation actions.
NDB-022: Assessment records whether serious harm is likely.
NDB-023: Assessment records whether eligible data breach exists.
NDB-024: Assessment records privacy officer decision.
NDB-025: Assessment records counsel review where required.
NDB-026: Assessment records target completion within 30 days for suspected eligible breach.
NDB-027: Assessment records reason if assessment exceeds target.
NDB-028: Assessment records notification decision.
NDB-029: Assessment records no-notification reason.
NDB-030: Assessment records evidence hash.

## Notification Content

Notify-001: OAIC statement identifies the entity.
Notify-002: OAIC statement describes the eligible data breach.
Notify-003: OAIC statement describes the kinds of information concerned.
Notify-004: OAIC statement recommends steps individuals should take.
Notify-005: Individual notice identifies entity.
Notify-006: Individual notice describes breach.
Notify-007: Individual notice describes information involved.
Notify-008: Individual notice recommends protective steps.
Notify-009: Individual notice uses accessible language.
Notify-010: Individual notice avoids exposing additional personal information.
Notify-011: Publication notice is used only when direct notice is not practicable.
Notify-012: Publication notice must be prominent.
Notify-013: Notification evidence stores delivery channel.
Notify-014: Notification evidence stores send time.
Notify-015: Notification evidence stores recipient cohort.
Notify-016: Notification evidence stores template version.
Notify-017: Notification evidence stores approver.
Notify-018: Notification evidence stores regulator acknowledgement where available.
Notify-019: Notification evidence stores corrections or updates.
Notify-020: Notification evidence stores final closure.

## Activated Cedar Policies

Cedar-001: `au.incident.personal_info_classifier` requires personal-information triage.
Cedar-002: `au.incident.ndb_assessment_required` starts assessment for suspected eligible data breach.
Cedar-003: `au.incident.ndb_assessment_clock` blocks closure without assessment result.
Cedar-004: `au.incident.serious_harm_triage` requires serious-harm likelihood decision.
Cedar-005: `au.incident.remedial_action_review` requires prevent-serious-harm review.
Cedar-006: `au.incident.eligible_breach_notification` requires OAIC and individual notification when eligible.
Cedar-007: `au.incident.no_notification_reason` requires reason when notification not made.
Cedar-008: `au.incident.mhr_route` routes My Health Record data incidents.
Cedar-009: `au.incident.apra_cps234_route` routes APRA material information-security incidents.
Cedar-010: `au.incident.austrac_compartment` limits suspicious matter and reporting data visibility.
Cedar-011: `au.incident.asic_customer_harm` routes securities customer harm and remediation.
Cedar-012: `au.incident.ahpra_practitioner_data` routes practitioner-regulator data incidents.
Cedar-013: `au.incident.evidence_hash_required` blocks closure without ADR-0263 evidence hash.
Cedar-014: `au.incident.notification_update_required` requires update if notification facts materially change.
Cedar-015: `au.incident.postmortem_required` requires post-incident review for eligible breach.

## Data Model Deltas

Data-001: `au_incident_id` identifies Australian incident overlay case.
Data-002: `au_incident_classification` stores incident taxonomy.
Data-003: `au_personal_information_involved` stores yes, no, unknown.
Data-004: `au_data_classes_involved` stores affected data classes.
Data-005: `au_ndb_assessment_id` stores suspected eligible data breach assessment.
Data-006: `au_ndb_assessment_started_at` stores clock start.
Data-007: `au_ndb_assessment_due_at` stores target due time.
Data-008: `au_ndb_assessment_closed_at` stores closure time.
Data-009: `au_unauthorized_access_flag` stores breach type.
Data-010: `au_unauthorized_disclosure_flag` stores breach type.
Data-011: `au_loss_flag` stores breach type.
Data-012: `au_serious_harm_likelihood` stores likely, unlikely, unknown.
Data-013: `au_remedial_action_status` stores preventing, failed, unavailable, unknown.
Data-014: `au_eligible_data_breach_status` stores eligible, not_eligible, unknown.
Data-015: `au_oaic_notification_id` stores OAIC statement.
Data-016: `au_individual_notice_id` stores affected individual notice.
Data-017: `au_publication_notice_id` stores publication notice.
Data-018: `au_mhr_breach_case_id` stores My Health Record breach case.
Data-019: `au_apra_incident_case_id` stores CPS 234 case.
Data-020: `au_austrac_compartment_id` stores AML/CTF restricted compartment.
Data-021: `au_asic_remediation_case_id` stores securities remediation case.
Data-022: `au_ahpra_privacy_case_id` stores practitioner data case.
Data-023: `au_notification_template_version` stores notice template.
Data-024: `au_incident_evidence_hash` stores ADR-0263 hash.
Data-025: `au_postmortem_id` stores post-incident review.

## API Contract Deltas

API-001: `POST /incidents/au/intake` creates Australian incident overlay case.
API-002: `POST /incidents/au/{id}/classify` records taxonomy.
API-003: `POST /incidents/au/{id}/ndb-assessment` starts NDB assessment.
API-004: `PATCH /incidents/au/{id}/ndb-assessment` updates assessment.
API-005: `POST /incidents/au/{id}/ndb-assessment/close` closes assessment.
API-006: `POST /incidents/au/{id}/oaic-notification` records OAIC statement.
API-007: `POST /incidents/au/{id}/individual-notices` records affected individual notices.
API-008: `POST /incidents/au/{id}/publication-notice` records publication notice.
API-009: `POST /incidents/au/{id}/mhr-route` opens My Health Record route.
API-010: `POST /incidents/au/{id}/apra-route` opens CPS 234 route.
API-011: `POST /incidents/au/{id}/austrac-compartment` opens restricted AML/CTF route.
API-012: `POST /incidents/au/{id}/asic-remediation` opens securities remediation route.
API-013: `POST /incidents/au/{id}/ahpra-route` opens practitioner-data route.
API-014: `POST /incidents/au/{id}/postmortem` records post-incident review.
API-015: `GET /incidents/au/{id}/evidence` exports incident evidence package.

## Audit Event Additions (per ADR-0263)

Audit-001: `AuIncidentOverlayCreated` records incident overlay creation.
Audit-002: `AuPersonalInformationTriageCompleted` records personal-information decision.
Audit-003: `AuNdbAssessmentStarted` records assessment clock.
Audit-004: `AuNdbAssessmentUpdated` records assessment update.
Audit-005: `AuSeriousHarmDetermined` records serious-harm decision.
Audit-006: `AuRemedialActionAssessed` records remedial-action decision.
Audit-007: `AuEligibleDataBreachDetermined` records eligibility decision.
Audit-008: `AuOaicNotificationPrepared` records statement preparation.
Audit-009: `AuOaicNotificationSent` records OAIC send evidence.
Audit-010: `AuIndividualNotificationPrepared` records notice preparation.
Audit-011: `AuIndividualNotificationSent` records affected individual send evidence.
Audit-012: `AuPublicNoticePublished` records publication evidence.
Audit-013: `AuMhrBreachRouteOpened` records My Health Record route.
Audit-014: `AuApraIncidentRouteOpened` records CPS 234 route.
Audit-015: `AuAustracIncidentCompartmentOpened` records restricted compartment.
Audit-016: `AuAsicRemediationRouteOpened` records customer harm route.
Audit-017: `AuAhpraPrivacyRouteOpened` records practitioner route.
Audit-018: `AuIncidentEvidenceSealed` records evidence hash.
Audit-019: `AuIncidentPostmortemCompleted` records review.
Audit-020: `AuIncidentOverlayClosed` records final closure.

## Failure Modes

Failure-001: Incident closes without personal-information triage.
Failure-002: Suspected eligible data breach lacks assessment start.
Failure-003: NDB assessment exceeds target without reason.
Failure-004: Serious-harm likelihood remains unknown at closure.
Failure-005: Remedial-action analysis missing at closure.
Failure-006: Eligible breach closes without OAIC notification.
Failure-007: Eligible breach closes without individual notification or valid alternative.
Failure-008: Notification omits kinds of information involved.
Failure-009: Notification omits recommended individual steps.
Failure-010: Notification exposes additional personal information.
Failure-011: My Health Record incident closes through generic NDB path only.
Failure-012: APRA tenant incident closes without CPS 234 materiality review.
Failure-013: AUSTRAC compartment information appears in general incident timeline.
Failure-014: ASIC securities customer harm closes without remediation review.
Failure-015: Ahpra practitioner privacy case closes without purpose review.
Failure-016: Evidence package lacks ADR-0263 hash.
Failure-017: Regulator notification update not sent after material correction.
Failure-018: Postmortem missing for eligible breach.
Failure-019: Legal hold destroys incident evidence.
Failure-020: Cross-border incident lacks APP 8 review.

## Worked Examples

Example-001: Lost laptop with encrypted disk triggers personal-information triage and remedial-action review.
Example-002: Database exfiltration with customer identity data starts NDB assessment.
Example-003: Password reset mistake emails data to wrong recipient and triggers unauthorized disclosure analysis.
Example-004: Remedial action prevents serious harm, and no-notification reason is recorded.
Example-005: Serious harm is likely, so OAIC and affected-individual notices are prepared.
Example-006: Direct notice is impracticable, so publication notice evidence is recorded.
Example-007: My Health Record document is exposed, so MHR-specific route opens.
Example-008: APRA-regulated tenant suffers ransomware, so CPS 234 materiality route opens.
Example-009: Suspicious matter report is implicated, so AUSTRAC restricted compartment hides details from normal support.
Example-010: Securities customers receive incorrect advice due to compromised system, so ASIC remediation route opens.
Example-011: Ahpra practitioner complaint data is exposed, so practitioner privacy route opens.
Example-012: Overseas processor caused breach, so APP 8 and section 16C accountability evidence are linked.
Example-013: Notification facts change after forensic review, so updated notice event is emitted.
Example-014: Postmortem records root cause and control remediation.
Example-015: Closure seals evidence hash and residual obligations.

## Cross-References

CrossRef-001: `README.md` defines AU-PACK-1 breach scope.
CrossRef-002: `regulatory-coverage.md` maps NDB, APRA, AUSTRAC, ASIC, and health coverage.
CrossRef-003: `data-residency-and-cross-border.md` defines APP 8 handling for overseas processors.
CrossRef-004: `consent-and-data-subject-rights.md` separates rights responses from breach notices.
CrossRef-005: `sectoral-overlays.md` defines sector incident overlays.
CrossRef-006: ADR-0243 defines Cedar deny-first controls.
CrossRef-007: ADR-0244 defines tenant and sub-scope evidence.
CrossRef-008: ADR-0251 defines compliance-pack mechanics.
CrossRef-009: ADR-0263 defines audit event envelopes.

## Assessment Checklist

Assess-001: Identify incident owner.
Assess-002: Identify privacy owner.
Assess-003: Identify security owner.
Assess-004: Identify affected tenant.
Assess-005: Identify affected service.
Assess-006: Identify affected region.
Assess-007: Identify affected data class.
Assess-008: Identify affected individuals.
Assess-009: Identify unauthorized access.
Assess-010: Identify unauthorized disclosure.
Assess-011: Identify loss.
Assess-012: Identify containment.
Assess-013: Identify remedial action.
Assess-014: Identify serious harm factors.
Assess-015: Identify sector overlays.
Assess-016: Identify cross-border processors.
Assess-017: Identify notification path.
Assess-018: Identify legal hold.
Assess-019: Identify evidence artifacts.
Assess-020: Identify closure criteria.

## Notification Checklist

Notice-001: Confirm eligible breach status.
Notice-002: Confirm OAIC notification requirement.
Notice-003: Confirm affected individual notification requirement.
Notice-004: Confirm entity identity.
Notice-005: Confirm breach description.
Notice-006: Confirm information kinds.
Notice-007: Confirm recommended individual steps.
Notice-008: Confirm contact channel.
Notice-009: Confirm template version.
Notice-010: Confirm approval.
Notice-011: Confirm delivery list.
Notice-012: Confirm accessibility.
Notice-013: Confirm no over-disclosure.
Notice-014: Confirm send time.
Notice-015: Confirm publication time if used.
Notice-016: Confirm regulator acknowledgment if available.
Notice-017: Confirm update process.
Notice-018: Confirm evidence hash.
Notice-019: Confirm audit event.
Notice-020: Confirm closure.

## Incident Evidence Rows

IncidentRow-001: Breach evidence records discovery timestamp.
IncidentRow-002: Breach evidence records report source.
IncidentRow-003: Breach evidence records affected tenant.
IncidentRow-004: Breach evidence records affected product.
IncidentRow-005: Breach evidence records affected service.
IncidentRow-006: Breach evidence records affected data store.
IncidentRow-007: Breach evidence records affected region.
IncidentRow-008: Breach evidence records personal information status.
IncidentRow-009: Breach evidence records sensitive information status.
IncidentRow-010: Breach evidence records health information status.
IncidentRow-011: Breach evidence records My Health Record status.
IncidentRow-012: Breach evidence records government identifier status.
IncidentRow-013: Breach evidence records financial information status.
IncidentRow-014: Breach evidence records practitioner data status.
IncidentRow-015: Breach evidence records APRA tenant status.
IncidentRow-016: Breach evidence records AUSTRAC workflow status.
IncidentRow-017: Breach evidence records ASIC workflow status.
IncidentRow-018: Breach evidence records unauthorized access.
IncidentRow-019: Breach evidence records unauthorized disclosure.
IncidentRow-020: Breach evidence records loss of information.
IncidentRow-021: Breach evidence records containment action.
IncidentRow-022: Breach evidence records eradication action.
IncidentRow-023: Breach evidence records recovery action.
IncidentRow-024: Breach evidence records remedial action.
IncidentRow-025: Breach evidence records remedial action effectiveness.
IncidentRow-026: Breach evidence records serious harm factor identity crime.
IncidentRow-027: Breach evidence records serious harm factor financial harm.
IncidentRow-028: Breach evidence records serious harm factor physical harm.
IncidentRow-029: Breach evidence records serious harm factor psychological harm.
IncidentRow-030: Breach evidence records serious harm factor reputational harm.
IncidentRow-031: Breach evidence records serious harm factor humiliation.
IncidentRow-032: Breach evidence records serious harm factor discrimination.
IncidentRow-033: Breach evidence records serious harm factor family violence risk.
IncidentRow-034: Breach evidence records serious harm factor clinical risk.
IncidentRow-035: Breach evidence records affected individual count.
IncidentRow-036: Breach evidence records affected cohort.
IncidentRow-037: Breach evidence records vulnerability cohort.
IncidentRow-038: Breach evidence records notification decision.
IncidentRow-039: Breach evidence records no-notification decision.
IncidentRow-040: Breach evidence records privacy officer approval.
IncidentRow-041: Breach evidence records counsel review.
IncidentRow-042: Breach evidence records incident commander.
IncidentRow-043: Breach evidence records security reviewer.
IncidentRow-044: Breach evidence records communications reviewer.
IncidentRow-045: Breach evidence records customer support reviewer.
IncidentRow-046: Breach evidence records regulator owner.
IncidentRow-047: Breach evidence records OAIC statement draft.
IncidentRow-048: Breach evidence records OAIC statement final.
IncidentRow-049: Breach evidence records OAIC submission timestamp.
IncidentRow-050: Breach evidence records OAIC acknowledgement.
IncidentRow-051: Breach evidence records individual notice draft.
IncidentRow-052: Breach evidence records individual notice final.
IncidentRow-053: Breach evidence records individual notice delivery.
IncidentRow-054: Breach evidence records publication notice draft.
IncidentRow-055: Breach evidence records publication notice final.
IncidentRow-056: Breach evidence records publication URL.
IncidentRow-057: Breach evidence records recommended step password reset.
IncidentRow-058: Breach evidence records recommended step credit monitoring.
IncidentRow-059: Breach evidence records recommended step scam vigilance.
IncidentRow-060: Breach evidence records recommended step medical follow-up.
IncidentRow-061: Breach evidence records recommended step identity document replacement.
IncidentRow-062: Breach evidence records support channel.
IncidentRow-063: Breach evidence records call center script.
IncidentRow-064: Breach evidence records email template.
IncidentRow-065: Breach evidence records postal template.
IncidentRow-066: Breach evidence records accessibility review.
IncidentRow-067: Breach evidence records localization review.
IncidentRow-068: Breach evidence records over-disclosure review.
IncidentRow-069: Breach evidence records under-disclosure review.
IncidentRow-070: Breach evidence records correction notice path.
IncidentRow-071: Breach evidence records update notice path.
IncidentRow-072: Breach evidence records APRA materiality review.
IncidentRow-073: Breach evidence records APRA route owner.
IncidentRow-074: Breach evidence records APRA notification evidence.
IncidentRow-075: Breach evidence records CPS 234 control link.
IncidentRow-076: Breach evidence records AUSTRAC compartment.
IncidentRow-077: Breach evidence records AUSTRAC visibility restriction.
IncidentRow-078: Breach evidence records AUSTRAC tipping-off review.
IncidentRow-079: Breach evidence records suspicious matter separation.
IncidentRow-080: Breach evidence records ASIC customer harm review.
IncidentRow-081: Breach evidence records ASIC remediation route.
IncidentRow-082: Breach evidence records ASIC complaint route.
IncidentRow-083: Breach evidence records ASIC cyber resilience evidence.
IncidentRow-084: Breach evidence records MHR participant role.
IncidentRow-085: Breach evidence records MHR system context.
IncidentRow-086: Breach evidence records MHR notification route.
IncidentRow-087: Breach evidence records MHR consumer impact.
IncidentRow-088: Breach evidence records Ahpra practitioner context.
IncidentRow-089: Breach evidence records Ahpra notification context.
IncidentRow-090: Breach evidence records Ahpra privacy purpose.
IncidentRow-091: Breach evidence records cross-border processor.
IncidentRow-092: Breach evidence records APP 8 assessment.
IncidentRow-093: Breach evidence records section 16C accountability.
IncidentRow-094: Breach evidence records overseas recipient safeguards.
IncidentRow-095: Breach evidence records contract breach notice.
IncidentRow-096: Breach evidence records supplier incident report.
IncidentRow-097: Breach evidence records forensic report.
IncidentRow-098: Breach evidence records root cause.
IncidentRow-099: Breach evidence records contributing factor.
IncidentRow-100: Breach evidence records corrective action.
IncidentRow-101: Breach evidence records preventive action.
IncidentRow-102: Breach evidence records control owner.
IncidentRow-103: Breach evidence records remediation due date.
IncidentRow-104: Breach evidence records remediation completion date.
IncidentRow-105: Breach evidence records residual risk.
IncidentRow-106: Breach evidence records risk acceptance.
IncidentRow-107: Breach evidence records board reporting.
IncidentRow-108: Breach evidence records executive reporting.
IncidentRow-109: Breach evidence records customer reporting.
IncidentRow-110: Breach evidence records regulator reporting.
IncidentRow-111: Breach evidence records evidence package hash.
IncidentRow-112: Breach evidence records audit event id.
IncidentRow-113: Breach evidence records trace id.
IncidentRow-114: Breach evidence records policy id.
IncidentRow-115: Breach evidence records citation id.
IncidentRow-116: Breach evidence records retention profile.
IncidentRow-117: Breach evidence records legal hold.
IncidentRow-118: Breach evidence records deletion exception.
IncidentRow-119: Breach evidence records incident closure gate.
IncidentRow-120: Breach evidence records postmortem gate.
IncidentRow-121: Breach evidence records lessons learned.
IncidentRow-122: Breach evidence records tabletop update.
IncidentRow-123: Breach evidence records runbook update.
IncidentRow-124: Breach evidence records detection update.
IncidentRow-125: Breach evidence records alert tuning.
IncidentRow-126: Breach evidence records access review.
IncidentRow-127: Breach evidence records credential rotation.
IncidentRow-128: Breach evidence records key rotation.
IncidentRow-129: Breach evidence records token revocation.
IncidentRow-130: Breach evidence records session revocation.
IncidentRow-131: Breach evidence records backup integrity.
IncidentRow-132: Breach evidence records restore test.
IncidentRow-133: Breach evidence records log preservation.
IncidentRow-134: Breach evidence records chain of custody.
IncidentRow-135: Breach evidence records time synchronization.
IncidentRow-136: Breach evidence records data minimisation failure.
IncidentRow-137: Breach evidence records APP 11 failure.
IncidentRow-138: Breach evidence records APP 6 failure.
IncidentRow-139: Breach evidence records APP 8 failure.
IncidentRow-140: Breach evidence records APP 10 failure.
IncidentRow-141: Breach evidence records suspected breach start.
IncidentRow-142: Breach evidence records thirty-day target.
IncidentRow-143: Breach evidence records target breach reason.
IncidentRow-144: Breach evidence records assessment update cadence.
IncidentRow-145: Breach evidence records assessment final decision.
IncidentRow-146: Breach evidence records eligible breach yes.
IncidentRow-147: Breach evidence records eligible breach no.
IncidentRow-148: Breach evidence records eligible breach unknown.
IncidentRow-149: Breach evidence records review-required state.
IncidentRow-150: Breach evidence records denial state.
IncidentRow-151: Breach evidence records closure state.
IncidentRow-152: Breach evidence records open obligations.
IncidentRow-153: Breach evidence records residual notifications.
IncidentRow-154: Breach evidence records residual monitoring.
IncidentRow-155: Breach evidence records residual support.
IncidentRow-156: Breach evidence records regulator follow-up.
IncidentRow-157: Breach evidence records customer follow-up.
IncidentRow-158: Breach evidence records insurance notice.
IncidentRow-159: Breach evidence records law enforcement contact.
IncidentRow-160: Breach evidence records public statement.
IncidentRow-161: Breach evidence records media holding statement.
IncidentRow-162: Breach evidence records internal briefing.
IncidentRow-163: Breach evidence records executive approval.
IncidentRow-164: Breach evidence records privacy commissioner contact.
IncidentRow-165: Breach evidence records MHR operator contact.
IncidentRow-166: Breach evidence records APRA contact.
IncidentRow-167: Breach evidence records ASIC contact.
IncidentRow-168: Breach evidence records AUSTRAC contact restriction.
IncidentRow-169: Breach evidence records Ahpra contact.
IncidentRow-170: Breach evidence records customer contract notice.
IncidentRow-171: Breach evidence records processor notice.
IncidentRow-172: Breach evidence records subprocessor notice.
IncidentRow-173: Breach evidence records onward recipient notice.
IncidentRow-174: Breach evidence records data map update.
IncidentRow-175: Breach evidence records asset inventory update.
IncidentRow-176: Breach evidence records dependency inventory update.
IncidentRow-177: Breach evidence records vendor risk update.
IncidentRow-178: Breach evidence records DPIA update if used.
IncidentRow-179: Breach evidence records privacy impact update.
IncidentRow-180: Breach evidence records control library update.
IncidentRow-181: Breach evidence records rule exception.
IncidentRow-182: Breach evidence records manual override.
IncidentRow-183: Breach evidence records override approver.
IncidentRow-184: Breach evidence records override expiry.
IncidentRow-185: Breach evidence records override revocation.
IncidentRow-186: Breach evidence records communication hold.
IncidentRow-187: Breach evidence records evidence preservation hold.
IncidentRow-188: Breach evidence records support escalation.
IncidentRow-189: Breach evidence records identity protection offer.
IncidentRow-190: Breach evidence records medical safety offer.
IncidentRow-191: Breach evidence records financial safety offer.
IncidentRow-192: Breach evidence records fraud watch.
IncidentRow-193: Breach evidence records abuse monitoring.
IncidentRow-194: Breach evidence records scam monitoring.
IncidentRow-195: Breach evidence records account lock.
IncidentRow-196: Breach evidence records account unlock.
IncidentRow-197: Breach evidence records password reset completion.
IncidentRow-198: Breach evidence records MFA reset completion.
IncidentRow-199: Breach evidence records API key rotation completion.
IncidentRow-200: Breach evidence records certificate rotation completion.
IncidentRow-201: Breach evidence records data restoration completion.
IncidentRow-202: Breach evidence records patch deployment.
IncidentRow-203: Breach evidence records rule deployment.
IncidentRow-204: Breach evidence records detector deployment.
IncidentRow-205: Breach evidence records customer segmentation.
IncidentRow-206: Breach evidence records notice segmentation.
IncidentRow-207: Breach evidence records high-risk cohort.
IncidentRow-208: Breach evidence records vulnerable cohort.
IncidentRow-209: Breach evidence records employee cohort.
IncidentRow-210: Breach evidence records practitioner cohort.
IncidentRow-211: Breach evidence records patient cohort.
IncidentRow-212: Breach evidence records financial customer cohort.
IncidentRow-213: Breach evidence records regulated customer cohort.
IncidentRow-214: Breach evidence records unregulated customer cohort.
IncidentRow-215: Breach evidence records nonpersonal data exclusion.
IncidentRow-216: Breach evidence records deidentified data review.
IncidentRow-217: Breach evidence records reidentification risk.
IncidentRow-218: Breach evidence records aggregation threshold.
IncidentRow-219: Breach evidence records audit minimisation.
IncidentRow-220: Breach evidence records final evidence seal.
IncidentRow-221: Breach evidence records final source citation.
IncidentRow-222: Breach evidence records final line count.
IncidentRow-223: Breach evidence records final status.
IncidentRow-224: Breach evidence records final handoff.
IncidentRow-225: Breach evidence records final scope.
IncidentRow-226: Breach evidence records final pack id.
IncidentRow-227: Breach evidence records final tenant id.
IncidentRow-228: Breach evidence records final audit id.
IncidentRow-229: Breach evidence records final closure id.
IncidentRow-230: Breach evidence records final verification.
IncidentRow-231: Breach evidence records OAIC NDB source URL.
IncidentRow-232: Breach evidence records OAIC Part 4 source URL.
IncidentRow-233: Breach evidence records Privacy Act source URL.
IncidentRow-234: Breach evidence records APRA CPS 234 source URL.
IncidentRow-235: Breach evidence records My Health Record source URL.
IncidentRow-236: Breach evidence records AUSTRAC source URL.
IncidentRow-237: Breach evidence records APP 11 link.
IncidentRow-238: Breach evidence records APP 8 link.
IncidentRow-239: Breach evidence records Part IIIC link.
IncidentRow-240: Breach evidence records ADR-0263 link.
IncidentRow-241: Breach evidence records ADR-0244 link.
IncidentRow-242: Breach evidence records ADR-0251 link.
IncidentRow-243: Breach evidence records ADR-0243 link.
IncidentRow-244: Breach evidence records no cross-pack edit.
IncidentRow-245: Breach evidence records Australia-only scope.
IncidentRow-246: Breach evidence records official-source grounding.
IncidentRow-247: Breach evidence records regulator-route separation.
IncidentRow-248: Breach evidence records notification-route separation.
IncidentRow-249: Breach evidence records assessment-route separation.
IncidentRow-250: Breach evidence records incident-route separation.
IncidentRow-251: Breach evidence records support-route separation.
IncidentRow-252: Breach evidence records evidence-route separation.
IncidentRow-253: Breach evidence records NDB closure readiness.
IncidentRow-254: Breach evidence records MHR closure readiness.
IncidentRow-255: Breach evidence records APRA closure readiness.
IncidentRow-256: Breach evidence records AUSTRAC closure readiness.
IncidentRow-257: Breach evidence records ASIC closure readiness.
IncidentRow-258: Breach evidence records Ahpra closure readiness.
IncidentRow-259: Breach evidence records privacy closure readiness.
IncidentRow-260: Breach evidence records security closure readiness.
IncidentRow-261: Breach evidence records customer closure readiness.
IncidentRow-262: Breach evidence records regulator closure readiness.
IncidentRow-263: Breach evidence records final postmortem.
IncidentRow-264: Breach evidence records final remediation.
IncidentRow-265: Breach evidence records final monitoring.
IncidentRow-266: Breach evidence records final communication.
IncidentRow-267: Breach evidence records final evidence export.
IncidentRow-268: Breach evidence records final audit export.
IncidentRow-269: Breach evidence records final regulator export.
IncidentRow-270: Breach evidence records final customer export.
IncidentRow-271: Breach evidence records final support artifact.
IncidentRow-272: Breach evidence records final lessons artifact.
IncidentRow-273: Breach evidence records final runbook artifact.
IncidentRow-274: Breach evidence records final detector artifact.
IncidentRow-275: Breach evidence records final control artifact.
IncidentRow-276: Breach evidence records final source artifact.
IncidentRow-277: Breach evidence records final rights artifact.
IncidentRow-278: Breach evidence records final residency artifact.
IncidentRow-279: Breach evidence records final sector artifact.
IncidentRow-280: Breach evidence records final release artifact.
IncidentRow-281: Breach evidence records final owner.
IncidentRow-282: Breach evidence records final reviewer.
IncidentRow-283: Breach evidence records final approver.
IncidentRow-284: Breach evidence records final timestamp.
IncidentRow-285: Breach evidence records final source refresh.
IncidentRow-286: Breach evidence records final counsel marker.
IncidentRow-287: Breach evidence records final compliance marker.
IncidentRow-288: Breach evidence records final privacy marker.
IncidentRow-289: Breach evidence records final security marker.
IncidentRow-290: Breach evidence records final incident marker.
IncidentRow-291: Breach evidence records final APRA marker.
IncidentRow-292: Breach evidence records final AUSTRAC marker.
IncidentRow-293: Breach evidence records final ASIC marker.
IncidentRow-294: Breach evidence records final MHR marker.
IncidentRow-295: Breach evidence records final Ahpra marker.
IncidentRow-296: Breach evidence records final OAIC marker.
IncidentRow-297: Breach evidence records final NDB marker.
IncidentRow-298: Breach evidence records final APP marker.
IncidentRow-299: Breach evidence records final pack marker.
IncidentRow-300: Breach evidence records final verification marker.
