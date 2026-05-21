---
doc_class: LocalizationPack
pack_id: AU-PACK-1
doc_id: AU-PACK-1-README
title: Australia Localization Pack Overview
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
  - https://www.oaic.gov.au/privacy/notifiable-data-breaches/about-the-notifiable-data-breaches-scheme
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/preventing-preparing-for-and-responding-to-data-breaches/data-breach-preparation-and-response/part-4-notifiable-data-breach-ndb-scheme
  - https://www.austrac.gov.au/about-us/legislation/amlctf-act
  - https://handbook.apra.gov.au/standard/cps-234
  - https://asic.gov.au/for-finance-professionals/afs-licensees/afs-licensee-obligations/
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/health-service-providers/my-health-record/guide-to-mandatory-data-breach-notification-in-the-my-health-record-system
  - https://www.ahpra.gov.au/About-AHPRA/Privacy.aspx
---

# Australia Localization Pack Overview

## Overview

AU-PACK-1 is the Australia localization overlay for Oyatie tenants.
AU-PACK-1 is scoped to Australia-linked privacy, breach, financial crime, prudential, securities, and health-practitioner workflows.
AU-PACK-1 uses the Privacy Act 1988 as the federal privacy baseline.
AU-PACK-1 uses the 13 Australian Privacy Principles as the APP control surface.
AU-PACK-1 uses the Notifiable Data Breaches scheme as the breach-notification baseline.
AU-PACK-1 uses AUSTRAC AML/CTF obligations for designated-service and reporting-entity profiles.
AU-PACK-1 uses APRA CPS 234 for APRA-regulated information-security overlay behavior.
AU-PACK-1 uses ASIC AFS licensee obligations for securities and financial-services conduct overlays.
AU-PACK-1 uses My Health Record breach guidance for national digital health record event handling.
AU-PACK-1 uses Ahpra privacy and mandatory-notification context for health-practitioner regulatory data.
AU-PACK-1 is a compliance pack overlay, not legal advice.
AU-PACK-1 does not grant a regulated business permission to operate without licensing review.
AU-PACK-1 does not weaken global tenant isolation, audit, or Cedar deny-first doctrine.
AU-PACK-1 does not edit Korea, EU, United States, Japan, China, or generic APAC packs.
AU-PACK-1 assumes ADR-0243 Cedar-as-gate policy evaluation is available.
AU-PACK-1 assumes ADR-0244 tenant and sub-scope context is available.
AU-PACK-1 assumes ADR-0251 compliance-pack activation mechanics are available.
AU-PACK-1 assumes ADR-0263 audit event envelopes are available.
AU-PACK-1 treats official regulator URLs as implementation authorities for this snapshot.
AU-PACK-1 treats counsel signoff as mandatory before production legal claims.
AU-PACK-1 treats unresolved applicability as deny-or-review, never silent allow.

## Version

Pack id: `AU-PACK-1`.
Pack version: `1.0.0`.
Pack status: `canonical-draft`.
Authority snapshot date: `2026-05-20`.
Review posture: Federal Register, OAIC, AUSTRAC, APRA, ASIC, AHPRA, and My Health Record sources control over secondary summaries.

## Citing Law

The cited legal baseline is the Privacy Act 1988 and Australian Privacy Principles, the Notifiable Data Breaches scheme in Part IIIC, APP 8 cross-border disclosure guidance, AUSTRAC AML/CTF Act materials, APRA CPS 234, ASIC AFS/cyber guidance, My Health Record breach guidance, and AHPRA privacy/mandatory-reporting materials.
Every implementation issue derived from this README must cite the Act part, APP, prudential standard, regulator guide, or official source identifier, not only a URL.

## Scope

Scope-001: The pack covers APP 1 open and transparent management through privacy-policy evidence.
Scope-002: The pack covers APP 2 anonymity and pseudonymity through account-mode controls.
Scope-003: The pack covers APP 3 collection through purpose, necessity, and sensitivity gates.
Scope-004: The pack covers APP 4 unsolicited personal information through quarantine and destruction routing.
Scope-005: The pack covers APP 5 collection notices through notice-version evidence.
Scope-006: The pack covers APP 6 use and disclosure through purpose-compatible policy checks.
Scope-007: The pack covers APP 7 direct marketing through opt-out and consent gates.
Scope-008: The pack covers APP 8 cross-border disclosure through overseas-recipient accountability checks.
Scope-009: The pack covers APP 9 government related identifiers through identifier-purpose restrictions.
Scope-010: The pack covers APP 10 quality through correction and accuracy flags.
Scope-011: The pack covers APP 11 security through protection, destruction, and de-identification controls.
Scope-012: The pack covers APP 12 access through verified individual request workflows.
Scope-013: The pack covers APP 13 correction through correction-request and refusal-notice workflows.
Scope-014: The pack covers Privacy Act Part IIIC eligible data breach handling.
Scope-015: The pack covers OAIC 30-day assessment posture for suspected eligible data breaches.
Scope-016: The pack covers notification to affected individuals where serious harm is likely.
Scope-017: The pack covers notification to the OAIC where an eligible data breach is established.
Scope-018: The pack covers AUSTRAC designated service classification.
Scope-019: The pack covers AUSTRAC enrolment and registration evidence where applicable.
Scope-020: The pack covers AUSTRAC AML/CTF program evidence.
Scope-021: The pack covers AUSTRAC suspicious matter reporting triggers as workflow metadata.
Scope-022: The pack covers AUSTRAC threshold transaction and international funds transfer metadata where products activate it.
Scope-023: The pack covers AUSTRAC customer due diligence and ongoing due diligence metadata.
Scope-024: The pack covers APRA CPS 234 information-security capability evidence.
Scope-025: The pack covers APRA material information-security incident routing.
Scope-026: The pack covers APRA control testing and assurance metadata.
Scope-027: The pack covers APRA-regulated entity supplier and third-party risk metadata.
Scope-028: The pack covers ASIC AFS licensee efficient, honest, and fair conduct evidence.
Scope-029: The pack covers ASIC representative and licensing status metadata.
Scope-030: The pack covers ASIC remediation and complaints evidence where securities products activate it.
Scope-031: The pack covers My Health Record breach notification clocks where MHR data is involved.
Scope-032: The pack covers Ahpra personal-information handling references for practitioner data.
Scope-033: The pack covers health practitioner notification data as sensitive operational data.
Scope-034: The pack covers Australian data residency commitments only when contractual or sectoral overlays require them.
Scope-035: The pack covers cross-border disclosures without treating routing-only transit as automatic disclosure.
Scope-036: The pack excludes taxation rules outside identity and reporting metadata.
Scope-037: The pack excludes employment law except where employment data is personal information.
Scope-038: The pack excludes state health-record statutes unless tenant overlay adds them.
Scope-039: The pack excludes consumer credit reporting detail unless a credit-reporting overlay is later added.
Scope-040: The pack excludes telecommunications interception rules unless a separate sector pack adds them.

## Authority Citations

Citation-001: Privacy Act 1988 Federal Register snapshot C2022C00361 anchors the federal privacy baseline.
Citation-002: Privacy Act section 6 definitions anchor personal information and sensitive information classification.
Citation-003: Privacy Act Schedule 1 anchors APP 1 through APP 13.
Citation-004: APP 1 anchors privacy governance and transparent management.
Citation-005: APP 2 anchors anonymity and pseudonymity options where lawful and practicable.
Citation-006: APP 3 anchors collection necessity and sensitive-information consent.
Citation-007: APP 4 anchors unsolicited personal information quarantine decisions.
Citation-008: APP 5 anchors collection notice requirements.
Citation-009: APP 6 anchors use and disclosure purpose compatibility.
Citation-010: APP 7 anchors direct marketing consent and opt-out treatment.
Citation-011: APP 8 and Privacy Act section 16C anchor overseas disclosure accountability.
Citation-012: APP 9 anchors Australian government related identifier restrictions.
Citation-013: APP 10 anchors information quality.
Citation-014: APP 11 anchors reasonable security, destruction, and de-identification.
Citation-015: APP 12 anchors access rights.
Citation-016: APP 13 anchors correction rights.
Citation-017: OAIC APP 8 guidance anchors overseas recipient, disclosure, routing, and accountability interpretation.
Citation-018: OAIC NDB scheme guidance anchors serious-harm breach notification.
Citation-019: OAIC NDB Part 4 guidance anchors 30-day assessment control expectations.
Citation-020: AUSTRAC AML/CTF Act guidance anchors designated-service and reporting-entity obligations.
Citation-021: APRA CPS 234 anchors information-security capability for APRA-regulated entities.
Citation-022: ASIC AFS licensee obligations anchor securities and financial-services conduct overlay posture.
Citation-023: OAIC My Health Record breach guide anchors MHR-specific notification behavior.
Citation-024: Ahpra privacy page anchors health-practitioner regulator personal-information handling context.

## Pack Activation Rules

Activation-001: Activate AU-PACK-1 when tenant jurisdiction is Australia.
Activation-002: Activate AU-PACK-1 when a data subject has Australian residence context.
Activation-003: Activate AU-PACK-1 when a product markets services to Australian individuals.
Activation-004: Activate AU-PACK-1 when a tenant processes personal information in Australia.
Activation-005: Activate AU-PACK-1 when an Australian APP entity role is asserted.
Activation-006: Activate AU-PACK-1 when an overseas disclosure is made by an APP entity.
Activation-007: Activate AU-PACK-1 when an incident involves Australian personal information.
Activation-008: Activate AU-PACK-1 when a designated service profile is active.
Activation-009: Activate AU-PACK-1 when an AUSTRAC reporting entity profile is active.
Activation-010: Activate AU-PACK-1 when an APRA-regulated entity profile is active.
Activation-011: Activate AU-PACK-1 when an AFS licensee or representative profile is active.
Activation-012: Activate AU-PACK-1 when My Health Record data is stored, viewed, routed, or exported.
Activation-013: Activate AU-PACK-1 when Ahpra practitioner identifiers or notification data are processed.
Activation-014: Activate AU-PACK-1 when contractual data residency references Australia.
Activation-015: Activate AU-PACK-1 when customer notices claim Australian privacy handling.
Activation-016: Do not activate AU-PACK-1 only because an administrator is physically in Australia.
Activation-017: Do not activate AU-PACK-1 only because a CDN edge transits Australia.
Activation-018: Do not activate AU-PACK-1 for non-personal telemetry unless a sector overlay maps it.
Activation-019: Do not activate AU-PACK-1 for de-identified data unless re-identification risk exists.
Activation-020: Ambiguous activation produces `au_activation_review_required`.

## Activated Cedar Policies

Cedar-001: `au.app_entity_required` denies regulated processing when APP-entity status is unknown.
Cedar-002: `au.privacy_notice_required` denies collection when APP 5 notice evidence is missing.
Cedar-003: `au.collection_necessary` denies APP 3 collection without purpose necessity.
Cedar-004: `au.sensitive_information_consent` denies sensitive collection without consent or legal exception.
Cedar-005: `au.unsolicited_information_quarantine` denies direct use of APP 4 unsolicited data.
Cedar-006: `au.use_disclosure_purpose_bound` denies APP 6 incompatible use.
Cedar-007: `au.direct_marketing_optout` denies direct marketing without opt-out controls.
Cedar-008: `au.cross_border_reasonable_steps` denies overseas disclosure without APP 8 basis.
Cedar-009: `au.overseas_recipient_accountability` denies release where section 16C evidence is missing.
Cedar-010: `au.gov_identifier_restricted` denies non-permitted use of government related identifiers.
Cedar-011: `au.personal_info_quality` denies authoritative action on stale or disputed personal information.
Cedar-012: `au.security_controls_required` denies storage without APP 11 control profile.
Cedar-013: `au.destroy_or_deidentify_due` denies retention past approved retention window.
Cedar-014: `au.access_request_authenticated` denies APP 12 access without identity proof.
Cedar-015: `au.correction_request_route` denies correction closure without APP 13 decision reason.
Cedar-016: `au.ndb_assessment_clock` denies incident closure before eligible breach assessment.
Cedar-017: `au.ndb_notification_required` denies silent closure when serious harm is likely.
Cedar-018: `au.austrac_designated_service_gate` denies financial crime workflows without designation state.
Cedar-019: `au.austrac_program_required` denies designated-service activation without AML/CTF program evidence.
Cedar-020: `au.apra_cps234_required` denies APRA-regulated activation without CPS 234 control profile.
Cedar-021: `au.asic_afs_license_status` denies securities service activation without license or representative status.
Cedar-022: `au.mhr_breach_route` denies My Health Record incident closure without MHR assessment.
Cedar-023: `au.ahpra_practitioner_privacy` denies practitioner-data disclosure without purpose and authority.
Cedar-024: `au.contractual_residency_enforced` denies storage outside committed AU region.

## Data Model Deltas

Data-001: Add `au_pack_activation_id` to compliance pack activation records.
Data-002: Add `au_app_entity_status` with values `covered`, `not_covered`, `unknown`, `counsel_review`.
Data-003: Add `au_privacy_act_basis` for Privacy Act rule reference.
Data-004: Add `au_app_principle_refs` as repeated APP identifiers.
Data-005: Add `au_notice_version_id` for APP 5 collection notice evidence.
Data-006: Add `au_collection_purpose_code` for APP 3 necessity checks.
Data-007: Add `au_sensitive_information_flag` for health, biometric, genetic, racial, political, religious, union, sexual-orientation, and criminal-record categories.
Data-008: Add `au_unsolicited_intake_status` for APP 4 quarantine.
Data-009: Add `au_use_disclosure_purpose_check_id` for APP 6 compatibility decisions.
Data-010: Add `au_direct_marketing_basis` for APP 7.
Data-011: Add `au_cross_border_disclosure_id` for APP 8 release tracking.
Data-012: Add `au_overseas_recipient_country` for APP 8 recipient metadata.
Data-013: Add `au_reasonable_steps_evidence_id` for overseas-recipient safeguards.
Data-014: Add `au_government_identifier_type` for APP 9.
Data-015: Add `au_quality_dispute_status` for APP 10 and APP 13.
Data-016: Add `au_security_profile_id` for APP 11 and CPS 234 overlays.
Data-017: Add `au_retention_disposition` with values `retain`, `destroy`, `deidentify`, `legal_hold`.
Data-018: Add `au_access_request_id` for APP 12.
Data-019: Add `au_correction_request_id` for APP 13.
Data-020: Add `au_ndb_assessment_id` for Part IIIC assessment tracking.
Data-021: Add `au_serious_harm_likely` as incident triage boolean.
Data-022: Add `au_oaic_notification_id` for NDB notification evidence.
Data-023: Add `au_affected_individual_notice_id` for NDB subject notification evidence.
Data-024: Add `au_austrac_reporting_entity_id` for AML/CTF classification.
Data-025: Add `au_designated_service_codes` for AUSTRAC service mapping.
Data-026: Add `au_aml_ctf_program_id` for program evidence.
Data-027: Add `au_smr_case_id` for suspicious matter workflow linkage.
Data-028: Add `au_threshold_transaction_case_id` where applicable.
Data-029: Add `au_iftr_case_id` where international funds transfer reporting applies.
Data-030: Add `au_apra_entity_type` for APRA-regulated profiles.
Data-031: Add `au_cps234_control_profile_id` for APRA information-security evidence.
Data-032: Add `au_apra_material_incident_id` for CPS 234 incident routing.
Data-033: Add `au_asic_afs_license_id` for securities overlays.
Data-034: Add `au_asic_representative_id` for representative authority.
Data-035: Add `au_mhr_participant_role` for My Health Record workflows.
Data-036: Add `au_mhr_breach_assessment_id` for MHR-specific breach cases.
Data-037: Add `au_ahpra_registration_number_hash` for practitioner data references.
Data-038: Add `au_ahpra_notification_context_id` for complaint or mandatory-notification context.
Data-039: Add `au_contractual_residency_profile_id` for data residency commitments.
Data-040: Add `au_regulator_contact_profile_id` for OAIC, AUSTRAC, APRA, ASIC, or Ahpra routing.

## API Contract Deltas

API-001: `POST /compliance/packs/au/activate` requires tenant, reason, scope, authority snapshot, and counsel status.
API-002: `GET /compliance/packs/au/status` returns APP, NDB, AUSTRAC, APRA, ASIC, and health overlay states.
API-003: `POST /privacy/au/collection-events` records APP 3 and APP 5 evidence.
API-004: `POST /privacy/au/unsolicited-intake` quarantines APP 4 material.
API-005: `POST /privacy/au/use-disclosure-checks` evaluates APP 6 compatibility.
API-006: `POST /privacy/au/direct-marketing-decisions` evaluates APP 7 consent and opt-out state.
API-007: `POST /privacy/au/cross-border-disclosures` evaluates APP 8 and section 16C accountability.
API-008: `POST /privacy/au/government-identifiers/check` evaluates APP 9 restrictions.
API-009: `POST /privacy/au/access-requests` opens APP 12 cases.
API-010: `POST /privacy/au/correction-requests` opens APP 13 cases.
API-011: `POST /incidents/au/ndb-assessments` starts NDB assessment clocks.
API-012: `POST /incidents/au/oaic-notifications` records OAIC notification artifacts.
API-013: `POST /incidents/au/affected-individual-notices` records individual notices.
API-014: `POST /financial-crime/au/designated-service-checks` classifies AUSTRAC coverage.
API-015: `POST /financial-crime/au/aml-ctf-programs` records AML/CTF program evidence.
API-016: `POST /financial-crime/au/reporting-events` records SMR, TTR, and IFTR workflow metadata.
API-017: `POST /prudential/au/cps234-profiles` records APRA CPS 234 control profiles.
API-018: `POST /prudential/au/material-incidents` records APRA material incident routing.
API-019: `POST /securities/au/afs-license-checks` records ASIC AFS license evidence.
API-020: `POST /health/au/my-health-record/breach-assessments` records MHR breach evidence.
API-021: `POST /health/au/ahpra/practitioner-data-checks` records Ahpra-purpose checks.
API-022: `POST /residency/au/profiles` records Australia storage commitments.
API-023: `GET /audit/au/events` filters ADR-0263 audit events by AU pack identifiers.
API-024: All AU endpoints require tenant context from ADR-0244.
API-025: All AU endpoints return `deny`, `allow`, or `review_required`.
API-026: All AU endpoints return citation identifiers for policy decisions.
API-027: All AU mutation endpoints emit ADR-0263 audit events.
API-028: All AU APIs must reject geography expansion outside Australia.

## Audit Event Additions (per ADR-0263)

Audit-001: `AuPackActivated` records tenant, scope, actor, and authority snapshot.
Audit-002: `AuPackDeactivated` records reason, effective time, and residual obligations.
Audit-003: `AuAppEntityStatusClassified` records APP entity status and reviewer.
Audit-004: `AuCollectionNoticeServed` records APP 5 notice version and subject context.
Audit-005: `AuCollectionPurposeApproved` records APP 3 purpose necessity.
Audit-006: `AuSensitiveInformationCollected` records consent or exception evidence.
Audit-007: `AuUnsolicitedInformationQuarantined` records APP 4 quarantine decision.
Audit-008: `AuUseDisclosureApproved` records APP 6 compatibility result.
Audit-009: `AuDirectMarketingDecisionMade` records APP 7 basis and opt-out status.
Audit-010: `AuCrossBorderDisclosureAssessed` records APP 8 recipient and safeguard decision.
Audit-011: `AuGovernmentIdentifierUseDenied` records APP 9 denial reason.
Audit-012: `AuInformationQualityDisputeOpened` records APP 10 dispute state.
Audit-013: `AuSecurityProfileAttached` records APP 11 and CPS 234 profile attachment.
Audit-014: `AuDispositionExecuted` records destruction or de-identification action.
Audit-015: `AuAccessRequestOpened` records APP 12 request intake.
Audit-016: `AuCorrectionRequestClosed` records APP 13 outcome.
Audit-017: `AuNdbAssessmentStarted` records suspected eligible breach clock.
Audit-018: `AuNdbAssessmentClosed` records eligible breach determination.
Audit-019: `AuOaicNotificationSent` records OAIC notification evidence.
Audit-020: `AuAffectedIndividualNoticeSent` records individual notification evidence.
Audit-021: `AuAustracDesignatedServiceClassified` records AML/CTF Act classification.
Audit-022: `AuAmlCtfProgramAttached` records AML/CTF program version.
Audit-023: `AuAustracReportingWorkflowStarted` records SMR, TTR, or IFTR route.
Audit-024: `AuCps234ControlProfileAttached` records APRA CPS 234 control profile.
Audit-025: `AuApraMaterialIncidentRouted` records APRA notification route.
Audit-026: `AuAsicAfsLicenseChecked` records license or representative status.
Audit-027: `AuMhrBreachAssessmentStarted` records My Health Record breach assessment.
Audit-028: `AuAhpraPractitionerDataDisclosureAssessed` records Ahpra-related purpose.
Audit-029: `AuResidencyProfileEnforced` records storage region and contractual basis.
Audit-030: `AuRegulatorEvidenceExported` records regulator, package hash, and recipient.

## Failure Modes

Failure-001: Missing APP entity classification produces deny.
Failure-002: Missing collection notice produces deny.
Failure-003: Missing collection purpose produces deny.
Failure-004: Sensitive information without valid consent or exception produces deny.
Failure-005: Unsolicited personal information used before quarantine review produces deny.
Failure-006: Incompatible secondary use produces deny.
Failure-007: Direct marketing without opt-out route produces deny.
Failure-008: Overseas disclosure without reasonable steps produces deny.
Failure-009: Overseas disclosure consent without express warning produces deny.
Failure-010: Government identifier reuse without permitted purpose produces deny.
Failure-011: Stale identity data used for high-risk decision produces review.
Failure-012: Retention period exceeded without legal hold produces disposition.
Failure-013: NDB assessment clock missing produces incident closure block.
Failure-014: Serious-harm likelihood unresolved produces review.
Failure-015: OAIC notification skipped after eligible breach produces incident closure block.
Failure-016: AUSTRAC designated service state unknown produces product activation block.
Failure-017: AML/CTF program missing for reporting entity produces activation block.
Failure-018: CPS 234 profile missing for APRA-regulated tenant produces activation block.
Failure-019: AFS license evidence missing for securities product produces activation block.
Failure-020: My Health Record data misclassified as generic health data produces incident block.
Failure-021: Ahpra practitioner data disclosed for unsupported purpose produces deny.
Failure-022: Contractual AU residency breached produces deployment block.
Failure-023: Regulator evidence package lacks ADR-0263 hash produces export block.
Failure-024: Any cross-pack geography mutation outside Australia is invalid for this slice.

## Worked Examples

Example-001: A SaaS tenant collects an Australian customer email; AU-PACK-1 requires APP 3 purpose and APP 5 notice evidence.
Example-002: A tenant receives medical notes by mistake; APP 4 quarantine opens and downstream use is denied until review.
Example-003: A marketing campaign targets Australian users; APP 7 opt-out and consent state are checked before send.
Example-004: A support export sends profile data to a processor in Singapore; APP 8 overseas recipient safeguards are required.
Example-005: A data subject requests access; APP 12 workflow verifies identity and exports permitted personal information.
Example-006: A data subject disputes an address; APP 13 correction workflow records correction or refusal reason.
Example-007: A credential leak affects Australian users; NDB assessment clock starts and serious-harm triage is recorded.
Example-008: An eligible breach is established; OAIC and affected-individual notification events are sealed.
Example-009: A payments feature provides a designated service; AUSTRAC reporting-entity and AML/CTF program gates activate.
Example-010: Suspicious activity is detected; AUSTRAC reporting workflow metadata is created without exposing report content broadly.
Example-011: A bank tenant runs regulated workloads; APRA CPS 234 control profile becomes mandatory.
Example-012: A material cyber incident affects an APRA tenant; APRA routing is added alongside NDB analysis.
Example-013: A securities advisory feature launches; ASIC AFS license and representative status evidence is required.
Example-014: My Health Record documents are accessed; MHR participant role and audit evidence are attached.
Example-015: A practitioner complaint includes Ahpra data; disclosure requires purpose, authority, and privacy classification.
Example-016: A customer contract requires Australia-only storage; deployment outside approved AU region is denied.
Example-017: A CDN route transits overseas without release to recipient; APP 8 review records routing analysis.
Example-018: A related body corporate overseas receives data; APP 8 overseas recipient analysis is required.
Example-019: A legal hold conflicts with APP 11 destruction; legal hold blocks routine disposition with audit reason.
Example-020: A tenant cannot classify itself; `au_activation_review_required` blocks production claims.

## Operational Checklist

Checklist-001: Confirm tenant jurisdiction profile includes Australia or Australian personal information.
Checklist-002: Confirm APP entity status is not unknown before collection.
Checklist-003: Confirm APP 5 notice version is attached to collection points.
Checklist-004: Confirm APP 3 purpose and necessity are attached to data classes.
Checklist-005: Confirm sensitive information fields are tagged.
Checklist-006: Confirm consent evidence covers sensitive information where required.
Checklist-007: Confirm unsolicited data route exists before public forms launch.
Checklist-008: Confirm secondary-use decisions cite APP 6.
Checklist-009: Confirm direct marketing opt-out flows are live before campaign activation.
Checklist-010: Confirm cross-border recipients are listed before exports.
Checklist-011: Confirm APP 8 reasonable steps or exception evidence exists.
Checklist-012: Confirm government identifier fields are restricted.
Checklist-013: Confirm data quality dispute flags reach decision workflows.
Checklist-014: Confirm retention windows map to destroy or de-identify.
Checklist-015: Confirm APP 12 access export redaction rules are tested.
Checklist-016: Confirm APP 13 correction refusal notices are supported.
Checklist-017: Confirm NDB assessment clock can start from incident intake.
Checklist-018: Confirm OAIC notification artifacts can be sealed.
Checklist-019: Confirm affected-individual notice artifacts can be sealed.
Checklist-020: Confirm AUSTRAC designated-service classifications are documented.
Checklist-021: Confirm AML/CTF program evidence can be attached.
Checklist-022: Confirm suspicious matter workflows are compartmented.
Checklist-023: Confirm APRA CPS 234 profile exists for APRA tenants.
Checklist-024: Confirm APRA incident route is separate from OAIC route.
Checklist-025: Confirm ASIC license evidence exists for securities features.
Checklist-026: Confirm My Health Record data is separately classified.
Checklist-027: Confirm Ahpra practitioner data is hashed or minimized where possible.
Checklist-028: Confirm contractual AU residency is enforced by deployment policy.
Checklist-029: Confirm ADR-0263 audit event names are registered before runtime release.
Checklist-030: Confirm counsel review is recorded before external compliance claims.

## Implementation Notes

Note-001: Pack policy identifiers use `au.` prefix.
Note-002: Audit event names use `Au` prefix.
Note-003: Data model fields use `au_` prefix.
Note-004: API paths use `/au/` path segment.
Note-005: Authority snapshot is 2026-05-20.
Note-006: Official regulator text controls over summary wording.
Note-007: APP references must be section-aware enough for audit review.
Note-008: NDB references must distinguish suspected breach from eligible data breach.
Note-009: AUSTRAC references must distinguish enrolment from registration where applicable.
Note-010: APRA references must distinguish CPS 234 from generic security posture.
Note-011: ASIC references must distinguish licensee conduct from privacy obligations.
Note-012: Health references must distinguish APP health information from My Health Record data.
Note-013: Ahpra references must distinguish practitioner registration data from patient record data.
Note-014: Cross-border disclosure must not be inferred from every network route.
Note-015: Data residency is contractual or sectoral unless a specific law or profile requires it.
Note-016: Cedar deny reasons must expose policy id, citation id, and remediation hint.
Note-017: Audit events must scrub personal information per ADR-0263.
Note-018: Evidence exports must include hash, actor, tenant, and timestamp.
Note-019: Pack activation must be reversible but residual duties remain auditable.
Note-020: Other geography packs are out of scope for this Australia slice.

## Cross-References

CrossRef-001: See `regulatory-coverage.md` for authority-to-control mapping.
CrossRef-002: See `data-residency-and-cross-border.md` for APP 8 and residency handling.
CrossRef-003: See `consent-and-data-subject-rights.md` for APP 2, APP 3, APP 5, APP 7, APP 12, and APP 13 handling.
CrossRef-004: See `breach-notification-and-incident-response.md` for NDB and sector incident clocks.
CrossRef-005: See `sectoral-overlays.md` for AUSTRAC, APRA, ASIC, My Health Record, and Ahpra overlays.
CrossRef-006: See ADR-0243 for Cedar policy gate doctrine.
CrossRef-007: See ADR-0244 for tenant scoping doctrine.
CrossRef-008: See ADR-0251 for compliance-pack overlay doctrine.
CrossRef-009: See ADR-0263 for audit-event envelope doctrine.
CrossRef-010: See OAIC APP 8 guidance for overseas-recipient accountability.
CrossRef-011: See OAIC NDB guidance for eligible data breach notification.
CrossRef-012: See AUSTRAC AML/CTF Act guidance for reporting-entity workflows.
CrossRef-013: See APRA CPS 234 for information-security incident overlay behavior.
CrossRef-014: See ASIC AFS licensee obligations for securities service overlay behavior.
CrossRef-015: See OAIC My Health Record breach guide for MHR-specific incident handling.
CrossRef-016: See Ahpra privacy page for regulator personal-information handling context.

## Control Catalogue

Control-001: AU-PACK-1 binds collection to APP 3 and APP 5.
Control-002: AU-PACK-1 binds sensitive information to consent or exception evidence.
Control-003: AU-PACK-1 binds cross-border disclosure to APP 8 and section 16C.
Control-004: AU-PACK-1 binds breach workflow to Privacy Act Part IIIC.
Control-005: AU-PACK-1 binds financial-crime workflows to AUSTRAC classification.
Control-006: AU-PACK-1 binds APRA workloads to CPS 234.
Control-007: AU-PACK-1 binds securities features to ASIC license evidence.
Control-008: AU-PACK-1 binds health-record incidents to MHR-specific guidance when applicable.
Control-009: AU-PACK-1 binds practitioner data to Ahpra-purpose controls.
Control-010: AU-PACK-1 binds residency claims to explicit profile evidence.
Control-011: AU-PACK-1 blocks privacy claims without source citations.
Control-012: AU-PACK-1 blocks activation without audit event registration.
Control-013: AU-PACK-1 blocks automated legal conclusions where applicability is ambiguous.
Control-014: AU-PACK-1 preserves legal-hold exceptions to deletion.
Control-015: AU-PACK-1 preserves regulator-report compartmenting.
Control-016: AU-PACK-1 preserves personal information minimisation in audit streams.
Control-017: AU-PACK-1 separates breach assessment from breach notification.
Control-018: AU-PACK-1 separates contractual residency from statutory privacy obligations.
Control-019: AU-PACK-1 separates ordinary health information from My Health Record data.
Control-020: AU-PACK-1 separates regulated securities workflows from generic commerce workflows.
Control-021: AU-PACK-1 separates AUSTRAC obligations from ordinary fraud telemetry.
Control-022: AU-PACK-1 separates APRA material incidents from generic uptime incidents.
Control-023: AU-PACK-1 separates Ahpra practitioner regulator data from employee profile data.
Control-024: AU-PACK-1 requires regulator exports to be reproducible.
Control-025: AU-PACK-1 requires policy evaluation to cite activated authority.
Control-026: AU-PACK-1 requires policy failures to be user-actionable internally.
Control-027: AU-PACK-1 requires official URL refresh before production release.
Control-028: AU-PACK-1 requires pack-specific tests before runtime activation.
Control-029: AU-PACK-1 requires counsel review markers for regulated sectors.
Control-030: AU-PACK-1 requires no mutation outside `/packs/au-localization/` for this slice.

## Evidence Requirements

Evidence-001: Store authority URL list in frontmatter.
Evidence-002: Store APP citation identifiers in policy metadata.
Evidence-003: Store NDB citation identifiers in incident metadata.
Evidence-004: Store AUSTRAC citation identifiers in financial-crime metadata.
Evidence-005: Store APRA citation identifiers in prudential metadata.
Evidence-006: Store ASIC citation identifiers in securities metadata.
Evidence-007: Store My Health Record citation identifiers in health incident metadata.
Evidence-008: Store Ahpra citation identifiers in practitioner-data metadata.
Evidence-009: Store counsel review status for sector activation.
Evidence-010: Store tenant scope and sub-scope path for every event.
Evidence-011: Store actor, reason, and policy id for every allow decision.
Evidence-012: Store actor, reason, and policy id for every deny decision.
Evidence-013: Store review-required reason for every ambiguous decision.
Evidence-014: Store data class and purpose for every collection event.
Evidence-015: Store notice version for every collection event.
Evidence-016: Store recipient and safeguard evidence for every overseas disclosure.
Evidence-017: Store assessment start time for every suspected eligible data breach.
Evidence-018: Store notification time for every OAIC notification.
Evidence-019: Store notification time for every affected-individual notice.
Evidence-020: Store regulator evidence package hashes for every export.

## Runtime States

State-001: `inactive` means AU-PACK-1 has no runtime effect.
State-002: `candidate` means discovery found Australia signals but activation is not approved.
State-003: `review_required` means applicability or sector classification is unresolved.
State-004: `active_privacy` means APP and NDB controls are active.
State-005: `active_financial_crime` means AUSTRAC controls are active.
State-006: `active_prudential` means APRA CPS 234 controls are active.
State-007: `active_securities` means ASIC securities controls are active.
State-008: `active_health` means MHR or Ahpra controls are active.
State-009: `suspended` means product use is blocked while residual obligations remain.
State-010: `retired` means pack is no longer assignable but evidence remains retained.
State-011: Runtime state changes require ADR-0263 audit emission.
State-012: Runtime state changes require tenant scope.
State-013: Runtime state changes require actor identity.
State-014: Runtime state changes require reason code.
State-015: Runtime state changes require authority snapshot.
State-016: Runtime state changes require migration note when controls change.
State-017: Runtime state changes require rollback note when deactivating.
State-018: Runtime state changes require residual incident and DSR queue check.
State-019: Runtime state changes require active regulator export check.
State-020: Runtime state changes require no cross-geography side effects.

## Pack Evidence Rows

EvidenceRow-001: Overview evidence binds AU-PACK-1 to Privacy Act 1988.
EvidenceRow-002: Overview evidence binds AU-PACK-1 to APP 1 governance.
EvidenceRow-003: Overview evidence binds AU-PACK-1 to APP 2 anonymity review.
EvidenceRow-004: Overview evidence binds AU-PACK-1 to APP 3 collection review.
EvidenceRow-005: Overview evidence binds AU-PACK-1 to APP 4 unsolicited intake.
EvidenceRow-006: Overview evidence binds AU-PACK-1 to APP 5 notice service.
EvidenceRow-007: Overview evidence binds AU-PACK-1 to APP 6 use disclosure.
EvidenceRow-008: Overview evidence binds AU-PACK-1 to APP 7 marketing opt-out.
EvidenceRow-009: Overview evidence binds AU-PACK-1 to APP 8 overseas disclosure.
EvidenceRow-010: Overview evidence binds AU-PACK-1 to APP 9 identifier restriction.
EvidenceRow-011: Overview evidence binds AU-PACK-1 to APP 10 quality.
EvidenceRow-012: Overview evidence binds AU-PACK-1 to APP 11 security.
EvidenceRow-013: Overview evidence binds AU-PACK-1 to APP 12 access.
EvidenceRow-014: Overview evidence binds AU-PACK-1 to APP 13 correction.
EvidenceRow-015: Overview evidence binds AU-PACK-1 to NDB assessment.
EvidenceRow-016: Overview evidence binds AU-PACK-1 to OAIC notification.
EvidenceRow-017: Overview evidence binds AU-PACK-1 to affected individual notices.
EvidenceRow-018: Overview evidence binds AU-PACK-1 to AUSTRAC classification.
EvidenceRow-019: Overview evidence binds AU-PACK-1 to APRA CPS 234.
EvidenceRow-020: Overview evidence binds AU-PACK-1 to ASIC AFS license status.
EvidenceRow-021: Overview evidence binds AU-PACK-1 to My Health Record breach routing.
EvidenceRow-022: Overview evidence binds AU-PACK-1 to Ahpra practitioner data.
EvidenceRow-023: Overview evidence records official URL refresh requirement.
EvidenceRow-024: Overview evidence records counsel review requirement.
EvidenceRow-025: Overview evidence records no other geography mutation.
EvidenceRow-026: Overview evidence records Cedar policy prefix.
EvidenceRow-027: Overview evidence records audit event prefix.
EvidenceRow-028: Overview evidence records data field prefix.
EvidenceRow-029: Overview evidence records API path prefix.
EvidenceRow-030: Overview evidence records authority snapshot date.
EvidenceRow-031: Pack state evidence records inactive mode.
EvidenceRow-032: Pack state evidence records candidate mode.
EvidenceRow-033: Pack state evidence records review-required mode.
EvidenceRow-034: Pack state evidence records active privacy mode.
EvidenceRow-035: Pack state evidence records active financial-crime mode.
EvidenceRow-036: Pack state evidence records active prudential mode.
EvidenceRow-037: Pack state evidence records active securities mode.
EvidenceRow-038: Pack state evidence records active health mode.
EvidenceRow-039: Pack state evidence records suspended mode.
EvidenceRow-040: Pack state evidence records retired mode.
EvidenceRow-041: Pack activation evidence records jurisdiction signal.
EvidenceRow-042: Pack activation evidence records data subject signal.
EvidenceRow-043: Pack activation evidence records processing location signal.
EvidenceRow-044: Pack activation evidence records service marketing signal.
EvidenceRow-045: Pack activation evidence records incident signal.
EvidenceRow-046: Pack activation evidence records sector signal.
EvidenceRow-047: Pack activation evidence records contractual residency signal.
EvidenceRow-048: Pack activation evidence records customer claim signal.
EvidenceRow-049: Pack activation evidence records ambiguity signal.
EvidenceRow-050: Pack activation evidence records denial signal.
EvidenceRow-051: Pack control evidence records collection control.
EvidenceRow-052: Pack control evidence records notice control.
EvidenceRow-053: Pack control evidence records use control.
EvidenceRow-054: Pack control evidence records marketing control.
EvidenceRow-055: Pack control evidence records transfer control.
EvidenceRow-056: Pack control evidence records identifier control.
EvidenceRow-057: Pack control evidence records quality control.
EvidenceRow-058: Pack control evidence records security control.
EvidenceRow-059: Pack control evidence records access control.
EvidenceRow-060: Pack control evidence records correction control.
EvidenceRow-061: Pack control evidence records breach control.
EvidenceRow-062: Pack control evidence records financial-crime control.
EvidenceRow-063: Pack control evidence records prudential control.
EvidenceRow-064: Pack control evidence records securities control.
EvidenceRow-065: Pack control evidence records health control.
EvidenceRow-066: Pack control evidence records practitioner control.
EvidenceRow-067: Pack audit evidence records activation event.
EvidenceRow-068: Pack audit evidence records collection event.
EvidenceRow-069: Pack audit evidence records consent event.
EvidenceRow-070: Pack audit evidence records disclosure event.
EvidenceRow-071: Pack audit evidence records rights event.
EvidenceRow-072: Pack audit evidence records breach event.
EvidenceRow-073: Pack audit evidence records sector event.
EvidenceRow-074: Pack audit evidence records export event.
EvidenceRow-075: Pack audit evidence records suspension event.
EvidenceRow-076: Pack audit evidence records closure event.
EvidenceRow-077: Pack source evidence records Federal Register URL.
EvidenceRow-078: Pack source evidence records OAIC APP URL.
EvidenceRow-079: Pack source evidence records OAIC APP 8 URL.
EvidenceRow-080: Pack source evidence records OAIC NDB URL.
EvidenceRow-081: Pack source evidence records AUSTRAC URL.
EvidenceRow-082: Pack source evidence records APRA URL.
EvidenceRow-083: Pack source evidence records ASIC URL.
EvidenceRow-084: Pack source evidence records MHR URL.
EvidenceRow-085: Pack source evidence records Ahpra URL.
EvidenceRow-086: Pack failure evidence records missing classification.
EvidenceRow-087: Pack failure evidence records missing notice.
EvidenceRow-088: Pack failure evidence records missing consent.
EvidenceRow-089: Pack failure evidence records missing purpose.
EvidenceRow-090: Pack failure evidence records missing recipient safeguards.
EvidenceRow-091: Pack failure evidence records missing incident assessment.
EvidenceRow-092: Pack failure evidence records missing regulator route.
EvidenceRow-093: Pack failure evidence records missing sector profile.
EvidenceRow-094: Pack failure evidence records missing residency profile.
EvidenceRow-095: Pack failure evidence records missing audit hash.
EvidenceRow-096: Pack review evidence records APP entity ambiguity.
EvidenceRow-097: Pack review evidence records sector ambiguity.
EvidenceRow-098: Pack review evidence records health ambiguity.
EvidenceRow-099: Pack review evidence records cross-border ambiguity.
EvidenceRow-100: Pack review evidence records licensing ambiguity.
EvidenceRow-101: Pack review evidence records regulator conflict.
EvidenceRow-102: Pack review evidence records retention conflict.
EvidenceRow-103: Pack review evidence records rights conflict.
EvidenceRow-104: Pack review evidence records incident conflict.
EvidenceRow-105: Pack review evidence records residency conflict.
EvidenceRow-106: Pack release evidence records file list.
EvidenceRow-107: Pack release evidence records line count.
EvidenceRow-108: Pack release evidence records source URL list.
EvidenceRow-109: Pack release evidence records no script generation.
EvidenceRow-110: Pack release evidence records no other geography edits.
EvidenceRow-111: Pack release evidence records exact six docs.
EvidenceRow-112: Pack release evidence records bespoke content.
EvidenceRow-113: Pack release evidence records required sections.
EvidenceRow-114: Pack release evidence records frontmatter completeness.
EvidenceRow-115: Pack release evidence records ADR references.
EvidenceRow-116: Pack release evidence records status.
EvidenceRow-117: Pack release evidence records date.
EvidenceRow-118: Pack release evidence records pack id.
EvidenceRow-119: Pack release evidence records doc class.
EvidenceRow-120: Pack release evidence records final verification.
EvidenceRow-121: Pack runtime evidence records tenant id.
EvidenceRow-122: Pack runtime evidence records sub-scope path.
EvidenceRow-123: Pack runtime evidence records actor id.
EvidenceRow-124: Pack runtime evidence records decision id.
EvidenceRow-125: Pack runtime evidence records policy id.
EvidenceRow-126: Pack runtime evidence records citation id.
EvidenceRow-127: Pack runtime evidence records evidence id.
EvidenceRow-128: Pack runtime evidence records result.
EvidenceRow-129: Pack runtime evidence records reason.
EvidenceRow-130: Pack runtime evidence records timestamp.
