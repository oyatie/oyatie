---
doc_class: LocalizationPack
pack_id: AU-PACK-1
doc_id: AU-PACK-1-REGULATORY-COVERAGE
title: Australia Regulatory Coverage Matrix
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
  - https://www.oaic.gov.au/privacy/notifiable-data-breaches/about-the-notifiable-data-breaches-scheme
  - https://www.austrac.gov.au/about-us/legislation/amlctf-act
  - https://handbook.apra.gov.au/standard/cps-234
  - https://asic.gov.au/for-finance-professionals/afs-licensees/afs-licensee-obligations/
  - https://www.oaic.gov.au/privacy/privacy-guidance-for-organisations-and-government-agencies/health-service-providers/my-health-record/guide-to-mandatory-data-breach-notification-in-the-my-health-record-system
  - https://www.ahpra.gov.au/About-AHPRA/Privacy.aspx
---

# Australia Regulatory Coverage Matrix

This document maps Australian authorities to runtime controls.
Rows are implementation controls, not legal conclusions.
Each control must be scoped by tenant, service, data class, and product activation.
Each control must emit audit evidence under ADR-0263 when evaluated.
Each unresolved legal applicability question becomes `review_required`.
The matrix covers Privacy Act 1988, APPs, NDB, AUSTRAC, APRA, ASIC, My Health Record, and Ahpra.

## Authority Citations

Authority-001: Privacy Act 1988 Federal Register C2022C00361: Privacy Act sections and Schedule 1 APPs.
Authority-002: OAIC APP text: APP 1 through APP 13 operational wording.
Authority-003: OAIC NDB scheme: eligible data breach notification to affected individuals and OAIC.
Authority-004: AUSTRAC AML/CTF Act page: designated services, enrolment, registration, and obligations.
Authority-005: APRA CPS 234: information security capability and incident notification for APRA-regulated entities.
Authority-006: ASIC AFS licensee obligations: efficient, honest, and fair financial services and licensee duties.
Authority-007: OAIC My Health Record breach guide: MHR mandatory data breach notification.
Authority-008: Ahpra privacy page: personal information handling by Ahpra and National Boards.

## Coverage Matrix

Coverage-001: Privacy Act s 6 personal information maps to `au.personal_information.classify`.
Coverage-002: Privacy Act s 6 sensitive information maps to `au.sensitive_information.consent_or_exception`.
Coverage-003: Privacy Act s 16C overseas accountability maps to `au.overseas_recipient_accountability`.
Coverage-004: Privacy Act Part IIIC eligible data breach maps to `au.ndb.eligible_breach_assess`.
Coverage-005: APP 1 open management maps to `au.privacy_management.profile_required`.
Coverage-006: APP 2 anonymity maps to `au.anonymous_pseudonymous.option_check`.
Coverage-007: APP 3 collection maps to `au.collection.necessary_and_lawful`.
Coverage-008: APP 4 unsolicited information maps to `au.unsolicited.quarantine`.
Coverage-009: APP 5 notice maps to `au.notice.collection_time_required`.
Coverage-010: APP 6 use/disclosure maps to `au.use_disclosure.purpose_bound`.
Coverage-011: APP 7 direct marketing maps to `au.direct_marketing.optout_required`.
Coverage-012: APP 8 cross-border maps to `au.cross_border.reasonable_steps`.
Coverage-013: APP 9 government identifiers maps to `au.gov_identifier.restricted`.
Coverage-014: APP 10 quality maps to `au.quality.accuracy_gate`.
Coverage-015: APP 11 security maps to `au.security.reasonable_steps`.
Coverage-016: APP 12 access maps to `au.access.request_workflow`.
Coverage-017: APP 13 correction maps to `au.correction.request_workflow`.
Coverage-018: NDB likely serious harm maps to `au.ndb.serious_harm_triage`.
Coverage-019: NDB OAIC notification maps to `au.ndb.oaic_notice_required`.
Coverage-020: NDB affected individual notification maps to `au.ndb.individual_notice_required`.
Coverage-021: AUSTRAC designated services map to `au.austrac.designated_service_profile`.
Coverage-022: AUSTRAC enrolment maps to `au.austrac.enrolment_evidence`.
Coverage-023: AUSTRAC remittance or digital currency exchange registration maps to `au.austrac.registration_evidence`.
Coverage-024: AUSTRAC AML/CTF program maps to `au.austrac.program_required`.
Coverage-025: AUSTRAC customer due diligence maps to `au.austrac.cdd_profile_required`.
Coverage-026: AUSTRAC ongoing due diligence maps to `au.austrac.ongoing_monitoring`.
Coverage-027: AUSTRAC reporting maps to `au.austrac.reporting_workflow`.
Coverage-028: APRA CPS 234 information security capability maps to `au.apra.info_sec_capability`.
Coverage-029: APRA CPS 234 control testing maps to `au.apra.control_testing`.
Coverage-030: APRA CPS 234 material incidents maps to `au.apra.material_incident_route`.
Coverage-031: ASIC AFS licensee obligation maps to `au.asic.afs_license_gate`.
Coverage-032: ASIC representative status maps to `au.asic.representative_gate`.
Coverage-033: ASIC cyber resilience guidance maps to `au.asic.cyber_resilience_evidence`.
Coverage-034: My Health Record breach guide maps to `au.mhr.breach_notification_route`.
Coverage-035: Ahpra privacy maps to `au.ahpra.personal_information_purpose`.

## Privacy Act and APP Controls

Privacy-001: APP 1 requires a privacy management profile before AU collection activation.
Privacy-002: APP 1 requires privacy policy URL, owner, review date, and coverage statement.
Privacy-003: APP 1 requires inquiry and complaint handling channels.
Privacy-004: APP 1 requires cross-border disclosure classes in privacy policy where applicable.
Privacy-005: APP 1 requires service owner attestation for each Australian data collection surface.
Privacy-006: APP 2 requires anonymous mode analysis for browse, enquiry, and support workflows.
Privacy-007: APP 2 requires pseudonymous mode analysis where lawful and practicable.
Privacy-008: APP 2 denial requires reason when identity is mandatory.
Privacy-009: APP 2 evidence must not force account creation for low-risk public content.
Privacy-010: APP 2 review must be repeated when product journeys change.
Privacy-011: APP 3 requires collection purpose code.
Privacy-012: APP 3 requires necessity mapping to product function.
Privacy-013: APP 3 requires direct collection preference unless exception is recorded.
Privacy-014: APP 3 requires sensitive information consent or legal exception.
Privacy-015: APP 3 requires health information classification when medical facts are collected.
Privacy-016: APP 4 requires unsolicited data source capture.
Privacy-017: APP 4 requires quick relevance assessment.
Privacy-018: APP 4 requires destruction or de-identification where collection would not have been allowed.
Privacy-019: APP 4 requires quarantine access controls.
Privacy-020: APP 4 requires audit evidence before any retention.
Privacy-021: APP 5 requires collection-time notice.
Privacy-022: APP 5 requires entity identity.
Privacy-023: APP 5 requires purpose disclosure.
Privacy-024: APP 5 requires consequence disclosure where collection is required or optional.
Privacy-025: APP 5 requires complaint contact reference.
Privacy-026: APP 5 requires overseas disclosure statement where applicable.
Privacy-027: APP 6 requires primary-purpose match.
Privacy-028: APP 6 requires secondary purpose compatibility evidence.
Privacy-029: APP 6 requires consent where no exception applies.
Privacy-030: APP 6 requires legal-authority citation when relying on law.
Privacy-031: APP 7 requires direct marketing classification.
Privacy-032: APP 7 requires simple opt-out.
Privacy-033: APP 7 requires opt-out suppression list.
Privacy-034: APP 7 requires source transparency where required.
Privacy-035: APP 7 requires marketing vendor export gate.
Privacy-036: APP 8 requires overseas recipient classification.
Privacy-037: APP 8 requires reasonable steps evidence.
Privacy-038: APP 8 requires substantially similar law or binding scheme evidence where used.
Privacy-039: APP 8 requires express informed consent evidence where used.
Privacy-040: APP 8 requires required-or-authorised-by-Australian-law evidence where used.
Privacy-041: APP 9 requires government identifier type.
Privacy-042: APP 9 requires permitted-use basis.
Privacy-043: APP 9 blocks internal universal identifier reuse.
Privacy-044: APP 9 blocks matching keys based on convenience alone.
Privacy-045: APP 9 requires review for Medicare, tax file, passport, and license identifiers.
Privacy-046: APP 10 requires accuracy status before adverse decisions.
Privacy-047: APP 10 requires stale-data warnings.
Privacy-048: APP 10 requires quality dispute flag propagation.
Privacy-049: APP 10 requires correction workflow linkage.
Privacy-050: APP 10 requires source confidence metadata.
Privacy-051: APP 11 requires security control profile.
Privacy-052: APP 11 requires access control.
Privacy-053: APP 11 requires encryption state.
Privacy-054: APP 11 requires deletion or de-identification trigger.
Privacy-055: APP 11 requires logs for privileged access.
Privacy-056: APP 12 requires identity verification.
Privacy-057: APP 12 requires access export scope control.
Privacy-058: APP 12 requires third-party redaction review.
Privacy-059: APP 12 requires refusal reason where refused.
Privacy-060: APP 12 requires response deadline tracking.
Privacy-061: APP 13 requires correction intake.
Privacy-062: APP 13 requires correction assessment.
Privacy-063: APP 13 requires refusal reason where refused.
Privacy-064: APP 13 requires downstream correction propagation where appropriate.
Privacy-065: APP 13 requires annotation option where correction is refused.

## Notifiable Data Breaches Controls

NDB-001: The NDB scheme applies to entities covered by the Privacy Act.
NDB-002: An eligible data breach requires unauthorized access, disclosure, or loss.
NDB-003: An eligible data breach requires likely serious harm.
NDB-004: An eligible data breach requires inability to prevent serious harm through remedial action.
NDB-005: Suspected eligible data breach opens assessment.
NDB-006: Assessment target is expeditious and within 30 days where required by OAIC guidance.
NDB-007: Assessment records incident start time.
NDB-008: Assessment records discovery time.
NDB-009: Assessment records suspected personal information classes.
NDB-010: Assessment records affected individual categories.
NDB-011: Assessment records remedial action considered.
NDB-012: Assessment records serious-harm likelihood.
NDB-013: Assessment records decision maker.
NDB-014: Assessment records counsel or privacy officer review.
NDB-015: Assessment records whether OAIC notification is required.
NDB-016: Assessment records whether affected individual notification is required.
NDB-017: Notification statement includes entity identity.
NDB-018: Notification statement includes description of eligible data breach.
NDB-019: Notification statement includes information concerned.
NDB-020: Notification statement includes recommended steps for individuals.
NDB-021: Notification route supports all affected individuals.
NDB-022: Notification route supports at-risk affected individuals.
NDB-023: Notification route supports publication when direct notice is not practicable.
NDB-024: Closure requires notification evidence or no-notification reason.
NDB-025: Closure requires audit hash under ADR-0263.

## AUSTRAC Controls

AUSTRAC-001: Designated service profile must be evaluated before regulated financial feature launch.
AUSTRAC-002: Reporting entity status must be explicit.
AUSTRAC-003: Enrolment evidence must be stored where required.
AUSTRAC-004: Registration evidence must be stored for remittance service where required.
AUSTRAC-005: Registration evidence must be stored for digital currency exchange where required.
AUSTRAC-006: AML/CTF program id must be attached.
AUSTRAC-007: AML/CTF risk assessment id must be attached.
AUSTRAC-008: Customer identification procedure id must be attached.
AUSTRAC-009: Beneficial ownership evidence id must be attached where relevant.
AUSTRAC-010: Politically exposed person handling must be marked where relevant.
AUSTRAC-011: Sanctions screening evidence must be compartmented.
AUSTRAC-012: Ongoing customer due diligence events must be auditable.
AUSTRAC-013: Suspicious matter route must restrict visibility.
AUSTRAC-014: Threshold transaction route must preserve amount and timing evidence where applicable.
AUSTRAC-015: International funds transfer route must preserve originator and beneficiary metadata where applicable.
AUSTRAC-016: Tipping-off risk must be flagged for suspicious matter workflows.
AUSTRAC-017: Recordkeeping retention profile must be attached.
AUSTRAC-018: Annual compliance report evidence must be attachable where applicable.
AUSTRAC-019: Correspondent banking or high-risk service flags must trigger review if product supports them.
AUSTRAC-020: AUSTRAC reporting artifacts must not be exposed to ordinary customer support.

## APRA CPS 234 Controls

APRA-001: APRA-regulated entity status must be explicit.
APRA-002: CPS 234 information security capability profile must be attached.
APRA-003: Information asset classification must exist.
APRA-004: Threat and vulnerability context must exist.
APRA-005: Control design evidence must exist.
APRA-006: Control operating effectiveness evidence must exist.
APRA-007: Testing schedule must exist.
APRA-008: Control deficiencies must have remediation owner.
APRA-009: Third-party information-security dependency list must exist.
APRA-010: Material information-security incident route must exist.
APRA-011: APRA notification evidence must be compartmented.
APRA-012: Incident severity mapping must support CPS 234 materiality.
APRA-013: Board or senior-management accountability field must exist.
APRA-014: Information security policy version must exist.
APRA-015: Control assurance report must be exportable.

## ASIC Securities Controls

ASIC-001: AFS license status must be explicit before financial service activation.
ASIC-002: Representative status must be explicit where service is provided by representative.
ASIC-003: Efficient, honest, and fair obligation evidence must be mapped to service controls.
ASIC-004: Financial product advice workflow must be tagged where applicable.
ASIC-005: Dealing workflow must be tagged where applicable.
ASIC-006: Market operation workflow must be tagged where applicable.
ASIC-007: Custody or asset-holding workflow must be tagged where applicable.
ASIC-008: Complaints and dispute resolution evidence must be attached where applicable.
ASIC-009: Remediation workflow must be attached where customer harm is detected.
ASIC-010: Cyber resilience evidence must be available for securities-facing systems.
ASIC-011: Misleading claim checks must cover Australian regulated-service marketing copy.
ASIC-012: License variation or cancellation state must block service activation.
ASIC-013: Representative termination state must block representative-led service activation.
ASIC-014: Advice record retention profile must be attachable where applicable.
ASIC-015: ASIC evidence exports must include source controls and audit hashes.

## Health and Practitioner Controls

Health-001: Health information is sensitive information under APP 3.
Health-002: Health service provider profile must activate sensitive data handling.
Health-003: My Health Record data must be tagged separately from ordinary health data.
Health-004: My Health Record breach guide route must activate for MHR data breaches.
Health-005: MHR breach assessment must identify participant role.
Health-006: MHR breach assessment must identify system access context.
Health-007: MHR notification route must not be collapsed into generic NDB-only closure.
Health-008: Ahpra practitioner registration number must be hashed or minimized where possible.
Health-009: Ahpra complaint or notification data must carry purpose and authority.
Health-010: Ahpra-related disclosure must be limited to regulator or lawful workflow context.
Health-011: Practitioner public-register data must not be enriched with private complaint data.
Health-012: Practitioner notification data must be compartmented from general HR data.
Health-013: Mandatory notification context must trigger legal review before disclosure automation.
Health-014: Patient data and practitioner data must remain separately classified.
Health-015: Health breach evidence must preserve clinical sensitivity and audit minimisation.

## Activated Cedar Policies

Cedar-001: `au.coverage.privacy_act` decides whether Privacy Act controls activate.
Cedar-002: `au.coverage.app_entity` denies unknown APP entity status.
Cedar-003: `au.coverage.app_principle` maps data action to APP ids.
Cedar-004: `au.coverage.ndb` maps incidents to NDB assessment.
Cedar-005: `au.coverage.austrac` maps services to designated-service review.
Cedar-006: `au.coverage.apra` maps tenant profile to CPS 234.
Cedar-007: `au.coverage.asic` maps financial service to AFS license evidence.
Cedar-008: `au.coverage.mhr` maps health records to MHR breach route.
Cedar-009: `au.coverage.ahpra` maps practitioner data to Ahpra purpose controls.
Cedar-010: `au.coverage.review_required` blocks ambiguous launch states.

## Data Model Deltas

Data-001: `au_authority_ref` stores authority id.
Data-002: `au_rule_ref` stores section, APP, or regulator rule id.
Data-003: `au_control_id` stores Cedar policy id.
Data-004: `au_control_owner` stores internal owner.
Data-005: `au_control_status` stores planned, active, suspended, or retired.
Data-006: `au_control_evidence_id` stores evidence artifact.
Data-007: `au_regulator` stores OAIC, AUSTRAC, APRA, ASIC, Ahpra, or Health.
Data-008: `au_app_ref` stores APP id.
Data-009: `au_sector_profile` stores privacy, aml_ctf, prudential, securities, health, or practitioner.
Data-010: `au_review_required_reason` stores ambiguity reason.

## API Contract Deltas

API-001: `GET /compliance/packs/au/coverage` returns active authority mappings.
API-002: `POST /compliance/packs/au/coverage/evaluate` evaluates tenant and service scope.
API-003: `GET /compliance/packs/au/coverage/{control_id}` returns citation and evidence requirements.
API-004: `POST /compliance/packs/au/coverage/review` opens review-required workflow.
API-005: `GET /compliance/packs/au/coverage/export` exports regulator-ready matrix with audit hashes.
API-006: Coverage APIs require tenant path.
API-007: Coverage APIs require pack version.
API-008: Coverage APIs return citation ids.
API-009: Coverage APIs return Cedar policy ids.
API-010: Coverage APIs emit audit events.

## Audit Event Additions (per ADR-0263)

Audit-001: `AuCoverageEvaluated` records authority mapping decision.
Audit-002: `AuAuthorityCitationAttached` records URL and rule reference.
Audit-003: `AuCoverageReviewRequired` records unresolved applicability.
Audit-004: `AuPrivacyActCoverageActivated` records APP entity state.
Audit-005: `AuNdbCoverageActivated` records incident profile.
Audit-006: `AuAustracCoverageActivated` records designated service profile.
Audit-007: `AuApraCoverageActivated` records APRA entity profile.
Audit-008: `AuAsicCoverageActivated` records AFS service profile.
Audit-009: `AuHealthCoverageActivated` records MHR or health-service profile.
Audit-010: `AuAhpraCoverageActivated` records practitioner-data profile.
Audit-011: `AuCoverageExported` records evidence package hash.
Audit-012: `AuCoverageSuspended` records reason and residual obligations.

## Failure Modes

Failure-001: Authority URL missing from frontmatter blocks release.
Failure-002: APP reference without APP number blocks release.
Failure-003: NDB control without serious-harm triage blocks incident closure.
Failure-004: AUSTRAC control without designated service profile blocks financial feature launch.
Failure-005: APRA control without APRA-regulated status blocks prudential claim.
Failure-006: ASIC control without AFS license evidence blocks securities claim.
Failure-007: MHR control without MHR data tag blocks health incident closure.
Failure-008: Ahpra control without purpose basis blocks practitioner-data disclosure.
Failure-009: Control owner missing blocks production activation.
Failure-010: Audit event missing blocks evidence export.

## Worked Examples

Example-001: Australian consumer profile collection maps to APP 3 and APP 5.
Example-002: Marketing export maps to APP 6, APP 7, and APP 8 if offshore.
Example-003: Credential compromise maps to APP 11 and NDB Part IIIC.
Example-004: Payment remittance feature maps to AUSTRAC designated-service review.
Example-005: Bank tenant onboarding maps to APRA CPS 234 coverage.
Example-006: Investment advice workflow maps to ASIC AFS license evidence.
Example-007: MHR document access incident maps to OAIC MHR breach guide.
Example-008: Practitioner complaint evidence maps to Ahpra privacy purpose controls.
Example-009: Contractual Australia storage claim maps to residency profile but not a blanket statutory rule.
Example-010: Unknown applicability maps to review-required state.

## Cross-References

CrossRef-001: `README.md` defines pack activation.
CrossRef-002: `data-residency-and-cross-border.md` expands APP 8.
CrossRef-003: `consent-and-data-subject-rights.md` expands APP 2, APP 3, APP 5, APP 7, APP 12, and APP 13.
CrossRef-004: `breach-notification-and-incident-response.md` expands NDB and incident response.
CrossRef-005: `sectoral-overlays.md` expands AUSTRAC, APRA, ASIC, My Health Record, and Ahpra.
CrossRef-006: ADR-0243 defines Cedar gate posture.
CrossRef-007: ADR-0244 defines tenant scope.
CrossRef-008: ADR-0251 defines pack overlay mechanics.
CrossRef-009: ADR-0263 defines audit event evidence.

## Matrix Detail Rows

Row-001: Privacy Act personal information classification requires data inventory owner.
Row-002: Privacy Act sensitive information classification requires elevated handling.
Row-003: Privacy Act APP entity classification requires tenant legal profile.
Row-004: Privacy Act Australian link analysis requires review for overseas tenants.
Row-005: APP 1 requires privacy policy lineage.
Row-006: APP 1 requires governance owner.
Row-007: APP 1 requires complaint channel.
Row-008: APP 1 requires inquiry channel.
Row-009: APP 1 requires policy publication state.
Row-010: APP 1 requires policy review cadence.
Row-011: APP 2 requires anonymous access feasibility.
Row-012: APP 2 requires pseudonymous access feasibility.
Row-013: APP 2 requires identity-required justification.
Row-014: APP 2 requires low-risk mode mapping.
Row-015: APP 2 requires accountless support analysis.
Row-016: APP 3 requires collection minimisation.
Row-017: APP 3 requires direct collection preference.
Row-018: APP 3 requires sensitive consent.
Row-019: APP 3 requires health information flag.
Row-020: APP 3 requires purpose compatibility.
Row-021: APP 4 requires unsolicited source.
Row-022: APP 4 requires permitted retention decision.
Row-023: APP 4 requires destruction path.
Row-024: APP 4 requires de-identification path.
Row-025: APP 4 requires review queue.
Row-026: APP 5 requires notice at or before collection where practicable.
Row-027: APP 5 requires identity of entity.
Row-028: APP 5 requires purpose of collection.
Row-029: APP 5 requires usual disclosures.
Row-030: APP 5 requires overseas disclosure note where applicable.
Row-031: APP 6 requires primary purpose.
Row-032: APP 6 requires secondary purpose exception.
Row-033: APP 6 requires consent state.
Row-034: APP 6 requires legal requirement reference.
Row-035: APP 6 requires disclosure recipient category.
Row-036: APP 7 requires marketing channel classification.
Row-037: APP 7 requires opt-out channel.
Row-038: APP 7 requires consent basis.
Row-039: APP 7 requires suppression enforcement.
Row-040: APP 7 requires vendor campaign gate.
Row-041: APP 8 requires overseas recipient country.
Row-042: APP 8 requires recipient legal basis.
Row-043: APP 8 requires reasonable steps artifact.
Row-044: APP 8 requires exception artifact where used.
Row-045: APP 8 requires withdrawal handling where consent is used.
Row-046: APP 9 requires identifier type.
Row-047: APP 9 requires permitted adoption state.
Row-048: APP 9 requires permitted use state.
Row-049: APP 9 requires matching prohibition evidence.
Row-050: APP 9 requires high-risk identifier review.
Row-051: APP 10 requires source timestamp.
Row-052: APP 10 requires confidence score.
Row-053: APP 10 requires dispute flag.
Row-054: APP 10 requires authoritative-source marker.
Row-055: APP 10 requires adverse decision quality gate.
Row-056: APP 11 requires access controls.
Row-057: APP 11 requires security controls.
Row-058: APP 11 requires destruction trigger.
Row-059: APP 11 requires de-identification trigger.
Row-060: APP 11 requires privileged access audit.
Row-061: APP 12 requires request identity proof.
Row-062: APP 12 requires export minimisation.
Row-063: APP 12 requires refusal reason.
Row-064: APP 12 requires fee review if fee exists.
Row-065: APP 12 requires response tracker.
Row-066: APP 13 requires correction request.
Row-067: APP 13 requires correction decision.
Row-068: APP 13 requires refusal notice.
Row-069: APP 13 requires annotation request support.
Row-070: APP 13 requires downstream notification review.
Row-071: NDB requires breach type.
Row-072: NDB requires serious harm assessment.
Row-073: NDB requires remedial action review.
Row-074: NDB requires OAIC notice artifact.
Row-075: NDB requires individual notice artifact.
Row-076: AUSTRAC requires designated service mapping.
Row-077: AUSTRAC requires reporting entity profile.
Row-078: AUSTRAC requires AML/CTF program profile.
Row-079: AUSTRAC requires reporting workflow compartment.
Row-080: AUSTRAC requires recordkeeping profile.
Row-081: APRA requires entity profile.
Row-082: APRA requires information asset inventory.
Row-083: APRA requires control testing evidence.
Row-084: APRA requires incident notification path.
Row-085: APRA requires third-party control evidence.
Row-086: ASIC requires AFS license mapping.
Row-087: ASIC requires representative mapping.
Row-088: ASIC requires conduct obligation mapping.
Row-089: ASIC requires complaint and remediation evidence.
Row-090: ASIC requires cyber resilience evidence.
Row-091: MHR requires participant role.
Row-092: MHR requires access context.
Row-093: MHR requires breach guide routing.
Row-094: MHR requires notification evidence.
Row-095: MHR requires separate health data classification.
Row-096: Ahpra requires practitioner data purpose.
Row-097: Ahpra requires privacy classification.
Row-098: Ahpra requires disclosure basis.
Row-099: Ahpra requires complaint data compartment.
Row-100: Ahpra requires public-register separation.

## Regulator Control Evidence Appendix

RegEvidence-001: Privacy Act evidence row maps section 6 to data classification.
RegEvidence-002: Privacy Act evidence row maps section 16C to overseas accountability.
RegEvidence-003: Privacy Act evidence row maps Part IIIC to NDB assessment.
RegEvidence-004: Privacy Act evidence row maps Schedule 1 to APP controls.
RegEvidence-005: OAIC evidence row maps APP 1 to governance.
RegEvidence-006: OAIC evidence row maps APP 2 to anonymity.
RegEvidence-007: OAIC evidence row maps APP 3 to collection.
RegEvidence-008: OAIC evidence row maps APP 4 to unsolicited handling.
RegEvidence-009: OAIC evidence row maps APP 5 to notice.
RegEvidence-010: OAIC evidence row maps APP 6 to use and disclosure.
RegEvidence-011: OAIC evidence row maps APP 7 to marketing.
RegEvidence-012: OAIC evidence row maps APP 8 to transfer.
RegEvidence-013: OAIC evidence row maps APP 9 to identifiers.
RegEvidence-014: OAIC evidence row maps APP 10 to quality.
RegEvidence-015: OAIC evidence row maps APP 11 to security.
RegEvidence-016: OAIC evidence row maps APP 12 to access.
RegEvidence-017: OAIC evidence row maps APP 13 to correction.
RegEvidence-018: NDB evidence row maps unauthorized access to breach type.
RegEvidence-019: NDB evidence row maps unauthorized disclosure to breach type.
RegEvidence-020: NDB evidence row maps loss to breach type.
RegEvidence-021: NDB evidence row maps serious harm to notification trigger.
RegEvidence-022: NDB evidence row maps remedial action to eligibility.
RegEvidence-023: NDB evidence row maps OAIC notice to regulator artifact.
RegEvidence-024: NDB evidence row maps individual notice to subject artifact.
RegEvidence-025: NDB evidence row maps publication notice to alternative artifact.
RegEvidence-026: AUSTRAC evidence row maps designated service to activation.
RegEvidence-027: AUSTRAC evidence row maps reporting entity to tenant profile.
RegEvidence-028: AUSTRAC evidence row maps enrolment to evidence artifact.
RegEvidence-029: AUSTRAC evidence row maps registration to evidence artifact.
RegEvidence-030: AUSTRAC evidence row maps AML/CTF program to policy gate.
RegEvidence-031: AUSTRAC evidence row maps customer due diligence to identity workflow.
RegEvidence-032: AUSTRAC evidence row maps ongoing due diligence to monitoring.
RegEvidence-033: AUSTRAC evidence row maps suspicious matter to restricted workflow.
RegEvidence-034: AUSTRAC evidence row maps threshold transaction to reporting workflow.
RegEvidence-035: AUSTRAC evidence row maps international funds transfer to reporting workflow.
RegEvidence-036: APRA evidence row maps CPS 234 capability to control profile.
RegEvidence-037: APRA evidence row maps information assets to inventory.
RegEvidence-038: APRA evidence row maps threats to risk profile.
RegEvidence-039: APRA evidence row maps vulnerabilities to risk profile.
RegEvidence-040: APRA evidence row maps control testing to assurance artifact.
RegEvidence-041: APRA evidence row maps material incident to notification route.
RegEvidence-042: APRA evidence row maps third-party dependency to supplier control.
RegEvidence-043: APRA evidence row maps remediation to control deficiency register.
RegEvidence-044: APRA evidence row maps board accountability to governance field.
RegEvidence-045: APRA evidence row maps audit export to evidence package.
RegEvidence-046: ASIC evidence row maps AFS license to service gate.
RegEvidence-047: ASIC evidence row maps representative status to authority gate.
RegEvidence-048: ASIC evidence row maps efficient honest fair to conduct control.
RegEvidence-049: ASIC evidence row maps complaint handling to remediation profile.
RegEvidence-050: ASIC evidence row maps cyber resilience to security evidence.
RegEvidence-051: ASIC evidence row maps license variation to activation state.
RegEvidence-052: ASIC evidence row maps license cancellation to denial state.
RegEvidence-053: ASIC evidence row maps representative termination to denial state.
RegEvidence-054: ASIC evidence row maps customer harm to remediation route.
RegEvidence-055: ASIC evidence row maps marketing claim to review gate.
RegEvidence-056: My Health Record evidence row maps MHR data to health overlay.
RegEvidence-057: My Health Record evidence row maps participant role to access context.
RegEvidence-058: My Health Record evidence row maps breach guide to notification route.
RegEvidence-059: My Health Record evidence row maps system access to audit evidence.
RegEvidence-060: My Health Record evidence row maps clinical document to sensitive data.
RegEvidence-061: Ahpra evidence row maps privacy page to personal-information handling.
RegEvidence-062: Ahpra evidence row maps mandatory reporting to notification context.
RegEvidence-063: Ahpra evidence row maps practitioner registration to hashed identifier.
RegEvidence-064: Ahpra evidence row maps complaint data to compartment.
RegEvidence-065: Ahpra evidence row maps public register data to separation rule.
RegEvidence-066: Coverage evidence records control owner.
RegEvidence-067: Coverage evidence records source URL.
RegEvidence-068: Coverage evidence records rule id.
RegEvidence-069: Coverage evidence records Cedar id.
RegEvidence-070: Coverage evidence records data delta.
RegEvidence-071: Coverage evidence records API delta.
RegEvidence-072: Coverage evidence records audit event.
RegEvidence-073: Coverage evidence records failure mode.
RegEvidence-074: Coverage evidence records worked example.
RegEvidence-075: Coverage evidence records cross reference.
RegEvidence-076: Coverage evidence records review owner.
RegEvidence-077: Coverage evidence records counsel state.
RegEvidence-078: Coverage evidence records tenant state.
RegEvidence-079: Coverage evidence records service state.
RegEvidence-080: Coverage evidence records sector state.
RegEvidence-081: Coverage evidence records residency state.
RegEvidence-082: Coverage evidence records incident state.
RegEvidence-083: Coverage evidence records rights state.
RegEvidence-084: Coverage evidence records transfer state.
RegEvidence-085: Coverage evidence records retention state.
RegEvidence-086: Coverage evidence records source freshness.
RegEvidence-087: Coverage evidence records activation date.
RegEvidence-088: Coverage evidence records policy version.
RegEvidence-089: Coverage evidence records pack version.
RegEvidence-090: Coverage evidence records status.
RegEvidence-091: Coverage evidence records line count verification.
RegEvidence-092: Coverage evidence records no other geography change.
RegEvidence-093: Coverage evidence records exact requested file set.
RegEvidence-094: Coverage evidence records official regulator source.
RegEvidence-095: Coverage evidence records Federal Register source.
RegEvidence-096: Coverage evidence records OAIC source.
RegEvidence-097: Coverage evidence records AUSTRAC source.
RegEvidence-098: Coverage evidence records APRA source.
RegEvidence-099: Coverage evidence records ASIC source.
RegEvidence-100: Coverage evidence records Ahpra source.
RegEvidence-101: Coverage evidence records Health source.
RegEvidence-102: Coverage evidence records ADR-0243 linkage.
RegEvidence-103: Coverage evidence records ADR-0244 linkage.
RegEvidence-104: Coverage evidence records ADR-0251 linkage.
RegEvidence-105: Coverage evidence records ADR-0263 linkage.
RegEvidence-106: Coverage evidence records authority matrix.
RegEvidence-107: Coverage evidence records sector matrix.
RegEvidence-108: Coverage evidence records privacy matrix.
RegEvidence-109: Coverage evidence records breach matrix.
RegEvidence-110: Coverage evidence records health matrix.
RegEvidence-111: Coverage evidence records regulator export.
RegEvidence-112: Coverage evidence records denial state.
RegEvidence-113: Coverage evidence records review state.
RegEvidence-114: Coverage evidence records allow state.
RegEvidence-115: Coverage evidence records final audit seal.
RegEvidence-116: Coverage evidence records policy enforcement point.
RegEvidence-117: Coverage evidence records data owner.
RegEvidence-118: Coverage evidence records service owner.
RegEvidence-119: Coverage evidence records compliance owner.
RegEvidence-120: Coverage evidence records privacy owner.
RegEvidence-121: Coverage evidence records security owner.
RegEvidence-122: Coverage evidence records financial-crime owner.
RegEvidence-123: Coverage evidence records prudential owner.
RegEvidence-124: Coverage evidence records securities owner.
RegEvidence-125: Coverage evidence records health owner.
RegEvidence-126: Coverage evidence records practitioner-data owner.
RegEvidence-127: Coverage evidence records evidence hash.
RegEvidence-128: Coverage evidence records export hash.
RegEvidence-129: Coverage evidence records review hash.
RegEvidence-130: Coverage evidence records closure hash.
RegEvidence-131: Coverage evidence records pack isolation.
RegEvidence-132: Coverage evidence records Australia-only geography.
RegEvidence-133: Coverage evidence records source-hint inclusion.
RegEvidence-134: Coverage evidence records section-aware citation.
RegEvidence-135: Coverage evidence records no generic APAC substitution.
RegEvidence-136: Coverage evidence records no legal-advice claim.
RegEvidence-137: Coverage evidence records production counsel gate.
RegEvidence-138: Coverage evidence records runtime deny default.
RegEvidence-139: Coverage evidence records ambiguity handling.
RegEvidence-140: Coverage evidence records release readiness.
RegEvidence-141: Coverage evidence records schema delta.
RegEvidence-142: Coverage evidence records endpoint delta.
RegEvidence-143: Coverage evidence records audit delta.
RegEvidence-144: Coverage evidence records failure delta.
RegEvidence-145: Coverage evidence records example delta.
RegEvidence-146: Coverage evidence records source delta.
RegEvidence-147: Coverage evidence records regulator delta.
RegEvidence-148: Coverage evidence records tenant delta.
RegEvidence-149: Coverage evidence records sector delta.
RegEvidence-150: Coverage evidence records final count.
RegEvidence-151: Coverage evidence records final source list.
RegEvidence-152: Coverage evidence records final verification.
RegEvidence-153: Coverage evidence records final scope.
RegEvidence-154: Coverage evidence records final status.
RegEvidence-155: Coverage evidence records final handoff.
