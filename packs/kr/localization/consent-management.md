---
doc_class: LocalizationPack
pack_id: KR-PACK-1
doc_id: KR-PACK-1-CONSENT-MANAGEMENT
title: Korea Localization Pack Consent Management
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.law.go.kr/
  - https://www.pipc.go.kr/
  - https://www.kisa.or.kr/
---

# Korea Localization Pack Consent Management

This document defines KR-PACK-1 consent capture, consent separation, youth verification, guardian consent, withdrawal, and audit behavior.
It is the canonical consent management document for Korean personal information workflows.
It treats consent as a purpose-specific ledger, not a single account-level flag.
It treats age and guardian flows as independent compliance events.
It treats Korean-language notice text as a required product artifact for Korean data subjects.

## Consent Doctrine

Consent must be explicit when the applicable Korean law requires consent.
Consent must be purpose-specific.
Consent must be separately revocable when the purpose is separately optional.
Consent must not be bundled across unrelated processing purposes.
Consent must not be inferred from continued account use.
Consent must not be inferred from a hidden pre-checked box.
Consent must not be inferred from an English-only notice for Korean users.
Consent must be recorded with notice version.
Consent must be recorded with language version.
Consent must be recorded with purpose code.
Consent must be recorded with data categories.
Consent must be recorded with retention period.
Consent must be recorded with third-party recipient when applicable.
Consent must be recorded with overseas transfer details when applicable.
Consent must be recorded with guardian evidence when subject is under 14.
Consent must be withdrawn through an auditable path.
Consent withdrawal must stop optional processing.
Consent withdrawal must not delete legally required retention records by itself.
Consent ledger rows must carry tenant context.
Consent audit payloads must be PII-scrubbed.

## Authority Citations

Authority snapshot date: 2026-05-20.
Primary law source: `https://www.law.go.kr/`.
Primary privacy regulator source: `https://www.pipc.go.kr/`.
Primary KISA source for privacy complaints and incident intake: `https://www.kisa.or.kr/`.
PIPA is cited as Personal Information Protection Act, `개인정보 보호법`.
PIPA Article 15 governs collection and use.
PIPA Article 17 governs third-party provision.
PIPA Article 18 governs use or provision beyond original purpose where applicable.
PIPA Article 22 governs consent methods and separated consent.
PIPA Article 22-2 governs processing personal information of children under 14.
PIPA Article 23 governs sensitive information.
PIPA Article 24 governs unique identifying information.
PIPA Article 24-2 governs resident registration number restrictions.
PIPA Article 28-8 governs overseas transfer.
PIPA Article 34 governs leakage notification and reporting.
Youth Protection Act is cited as `청소년 보호법`.
Youth Protection Act Article 16 governs age and identity verification for youth-harmful media.
Youth Protection Act Enforcement Decree Article 17 names verification methods.
Information Network Act lineage is cited for non-RRN identity verification alternatives.
Medical Service Act is cited when consent touches health records or patient portal flows.
Communications Secrets Protection Act is cited when consent interfaces touch messaging services.
Digital Documents and Transactions Act is cited when consent records themselves become electronic evidence.
Named effective dates must be read from law.go.kr for bundle build.
Korean official text controls over translation drift.
PIPC guidance controls consent interpretation when guidance is more specific than this document.

## Consent Purpose Taxonomy

Purpose `KR-CONSENT-ACCOUNT-CORE` covers account creation and authentication.
Purpose `KR-CONSENT-SERVICE-CORE` covers service delivery requested by the subject.
Purpose `KR-CONSENT-BILLING` covers billing and settlement.
Purpose `KR-CONSENT-PAYROLL` covers employment payroll processing.
Purpose `KR-CONSENT-FOUR-MAJOR-INSURANCE` covers Korean statutory insurance workflow.
Purpose `KR-CONSENT-HR-ADMIN` covers HR administration.
Purpose `KR-CONSENT-ATS-RECRUITING` covers applicant recruiting workflow.
Purpose `KR-CONSENT-PERFORMANCE` covers employee performance workflow.
Purpose `KR-CONSENT-WORKFORCE-ANALYTICS` covers de-identified workforce analytics.
Purpose `KR-CONSENT-MEDICAL-TREATMENT` covers medical treatment.
Purpose `KR-CONSENT-MEDICAL-PORTAL` covers patient portal access.
Purpose `KR-CONSENT-CLINICAL-TRIAL` covers clinical trial participation.
Purpose `KR-CONSENT-PHARMACY` covers prescription and pharmacy workflow.
Purpose `KR-CONSENT-EMERGENCY` covers emergency medical basis and after-action review.
Purpose `KR-CONSENT-COMMUNICATIONS` covers user-directed communications service.
Purpose `KR-CONSENT-COMMUNITY` covers community participation.
Purpose `KR-CONSENT-YOUTH-AGE-GATE` covers age verification for restricted content.
Purpose `KR-CONSENT-GUARDIAN-UNDER14` covers guardian consent for children under 14.
Purpose `KR-CONSENT-MARKETING` covers optional marketing.
Purpose `KR-CONSENT-THIRD-PARTY` covers third-party provision.
Purpose `KR-CONSENT-OVERSEAS-TRANSFER` covers overseas transfer.
Purpose `KR-CONSENT-SENSITIVE` covers sensitive information.
Purpose `KR-CONSENT-UNIQUE-ID` covers unique identifying information.
Purpose `KR-CONSENT-RRN-STATUTORY` records statutory basis for RRN handling, not ordinary consent.
Purpose `KR-CONSENT-CI-DI` covers alternative identity token linkage.
Purpose `KR-CONSENT-BIOMETRIC` covers biometric authentication or attendance where permitted.
Purpose `KR-CONSENT-ELECTRONIC-DOCUMENT` covers electronic document retention acknowledgment.
Purpose `KR-CONSENT-SECURITY-MONITORING` covers security monitoring notice.
Purpose `KR-CONSENT-INCIDENT-NOTICE` covers communication of incident notification.
Purpose `KR-CONSENT-DATA-SUBJECT-RIGHTS` covers identity verification for data-subject requests.

## Multi-Purpose Consent Enumeration

Every consent screen must enumerate each purpose separately.
Every required purpose must be labeled required.
Every optional purpose must be labeled optional.
Every optional purpose must be independently deniable.
Every optional purpose must be independently withdrawable.
Every purpose row must name data items.
Every purpose row must name retention period.
Every purpose row must name recipient if third-party provision applies.
Every purpose row must name overseas destination if cross-border transfer applies.
Every purpose row must name sensitive information if Article 23 applies.
Every purpose row must name unique identifying information if Article 24 applies.
Every purpose row must state that RRN is processed only under statutory basis if RRN workflow applies.
Every purpose row must include Korean display text.
Every purpose row may include English administrative fallback.
Every purpose row must record notice version.
Every purpose row must record UI component version.
Every purpose row must record capture channel.
Every purpose row must record timestamp.
Every purpose row must record subject age-band classification where minors may be present.
Every purpose row must record guardian consent linkage if subject is under 14.
Every purpose row must record withdrawal endpoint.
Every purpose row must record service owner.
Every purpose row must record Cedar policy IDs evaluated.
Every purpose row must record `tenant_id`.
Every purpose row must record `sub_scope_path` where scoped.
Every purpose row must emit an ADR-0263 audit event when captured or withdrawn.

## Age Verification

Age verification is required before youth-harmful media access.
Age verification is required before age-limited goods or services when Korean youth law applies.
Age verification must not use RRN by default.
Age verification should prefer approved identity verification providers.
Age verification may use CI/DI linkage when legally and technically sufficient.
Age verification must produce an age-band result.
Age verification must avoid returning raw birthdate to ordinary services.
Age verification must record verification method.
Age verification must record verification provider code.
Age verification must record verification timestamp.
Age verification must record token expiration.
Age verification must record allowed content class.
Age verification must record denied content class.
Age verification must record policy IDs.
Age verification must record user-facing Korean notice version.
Age verification must block youth-harmful content if verification is absent.
Age verification must block youth-harmful content if verification is expired.
Age verification must block youth-harmful content if result is minor.
Age verification must block youth-harmful content if provider confidence is insufficient.
Age verification must emit `KrYouthAgeGateEvaluated`.
Age verification denial must emit `KrYouthRestrictedAccessDenied`.
Age verification audit payload must not include raw identity document details.

## Guardian Consent for Minors Under 14

Subject under 14 requires legal guardian consent when PIPA Article 22-2 applies.
Guardian consent must be captured separately from the child's service interaction.
Guardian consent must identify the guardian relationship basis.
Guardian consent must identify the child subject record without exposing raw child PII in audit.
Guardian consent must identify consent purposes.
Guardian consent must identify data items.
Guardian consent must identify retention period.
Guardian consent must identify third-party recipients when present.
Guardian consent must identify overseas transfer when present.
Guardian consent must identify withdrawal path.
Guardian consent must be recorded with Korean notice version.
Guardian consent must be recorded with language version.
Guardian consent must be recorded with guardian verification method.
Guardian consent must be recorded with capture channel.
Guardian consent must be recorded with timestamp.
Guardian consent must be revocable where legally allowed.
Guardian consent withdrawal must stop optional processing.
Guardian consent withdrawal must preserve legally required evidence.
Guardian consent must emit `KrGuardianConsentCaptured`.
Guardian consent withdrawal must emit `KrGuardianConsentWithdrawn`.
Guardian consent failure must deny processing and emit denial event.
Guardian consent must not allow marketing by default.
Guardian consent must not authorize RRN collection by itself.
Guardian consent must not bypass Youth Protection Act age gate.

## Sensitive Information Consent

Sensitive information requires explicit basis.
Health data is sensitive information.
Biometric data used for identification is sensitive information.
Disability or veteran status may be sensitive depending on context.
Clinical trial records are sensitive.
Insurance claim health details are sensitive.
Employee medical examination results are sensitive.
Sensitive consent must be separate from ordinary service consent.
Sensitive consent must name sensitive data categories.
Sensitive consent must name processing purpose.
Sensitive consent must name retention period.
Sensitive consent must name disclosure recipients.
Sensitive consent must name overseas transfer basis when present.
Sensitive consent must emit `KrPipaSensitiveConsentCaptured`.
Sensitive consent denial must block optional sensitive processing.
Sensitive consent withdrawal must stop optional sensitive processing.
Sensitive consent cannot authorize RRN processing unless statutory basis also exists.
Sensitive consent cannot authorize communications content inspection by itself.
Sensitive consent cannot replace medical record access-purpose trace.

## Overseas Transfer Consent

Overseas transfer consent must be separate from ordinary service consent.
Overseas transfer consent must name destination country.
Overseas transfer consent must name recipient.
Overseas transfer consent must name transferred items.
Overseas transfer consent must name purpose.
Overseas transfer consent must name retention and use period.
Overseas transfer consent must name withdrawal method.
Overseas transfer consent must name onward-transfer constraints.
Overseas transfer consent must link to transfer assessment.
Overseas transfer consent must link to processor due diligence.
Overseas transfer consent must emit `KrCrossBorderTransferConsentCaptured`.
Overseas transfer consent withdrawal must revoke optional transfer.
Overseas transfer consent does not bypass CSAP cell pinning.
Overseas transfer consent does not permit raw RRN export.
Overseas transfer consent does not permit medical record export without healthcare basis.
Overseas transfer consent does not permit communications content analytics export.

## Activated Cedar Policies

`pack-kr-pack-1-pipa-purpose-consent` requires purpose-specific consent.
`pack-kr-pack-1-pipa-separate-consent` requires separate optional-purpose consent.
`pack-kr-pack-1-pipa-sensitive-consent` requires sensitive-information consent basis.
`pack-kr-pack-1-pipa-under14-guardian` requires guardian consent for children under 14.
`pack-kr-pack-1-youth-age-gate` requires youth age and identity verification.
`pack-kr-pack-1-youth-content-deny` denies youth-harmful content to minors.
`pack-kr-pack-1-consent-withdrawal-honor` stops optional processing after withdrawal.
`pack-kr-pack-1-cross-border-transfer-consent` permits transfer only with valid transfer consent.
`pack-kr-pack-1-cross-border-transfer-deny-default` denies transfer without basis.
`pack-kr-pack-1-ci-di-preferred` prefers non-RRN identity verification.
`pack-kr-pack-1-rrn-collection-deny-default` denies RRN collection through consent-only path.
`pack-kr-pack-1-rrn-statutory-basis` requires statutory basis for RRN.
`pack-kr-pack-1-localized-notice-required` requires Korean-language notice.
`pack-kr-pack-1-pii-emission-scrub` scrubs consent audit payloads.
`pack-kr-pack-1-audit-tenant-context` requires tenant context.
`pack-kr-pack-1-audit-jurisdiction-code` requires KR jurisdiction code.
`pack-kr-pack-1-medical-record-access-trace` requires medical access purpose beyond consent.
`pack-kr-pack-1-communications-secret-deny-content-inspection` blocks communications inspection despite generic consent.
`pack-kr-pack-1-pack-precedence-deny-wins` makes consent denials prevail over generic allows.

## Data Model Deltas

Add `consent.kr_consent_id`.
Add `consent.kr_subject_id`.
Add `consent.kr_subject_age_band`.
Add `consent.kr_guardian_consent_id`.
Add `consent.kr_guardian_relationship_basis`.
Add `consent.kr_purpose_code`.
Add `consent.kr_purpose_required_flag`.
Add `consent.kr_optional_flag`.
Add `consent.kr_notice_version`.
Add `consent.kr_notice_language`.
Add `consent.kr_notice_text_digest`.
Add `consent.kr_capture_channel`.
Add `consent.kr_capture_component_version`.
Add `consent.kr_capture_timestamp`.
Add `consent.kr_withdrawal_endpoint`.
Add `consent.kr_withdrawal_timestamp`.
Add `consent.kr_withdrawal_reason_code`.
Add `consent.kr_data_items`.
Add `consent.kr_sensitive_data_categories`.
Add `consent.kr_unique_identifier_categories`.
Add `consent.kr_rrn_statutory_basis_required`.
Add `consent.kr_retention_period`.
Add `consent.kr_third_party_recipient`.
Add `consent.kr_overseas_destination_country`.
Add `consent.kr_overseas_recipient`.
Add `consent.kr_overseas_transfer_items`.
Add `consent.kr_overseas_transfer_purpose`.
Add `consent.kr_overseas_retention_period`.
Add `consent.kr_processor_contract_id`.
Add `consent.kr_transfer_assessment_id`.
Add `consent.kr_policy_ids`.
Add `consent.kr_audit_id`.
Add `consent.kr_status`.
Add `consent.kr_superseded_by`.
Add `consent.kr_exemption_basis`.
Add `consent.kr_denial_reason`.
Add `age_gate.kr_verification_id`.
Add `age_gate.kr_provider_code`.
Add `age_gate.kr_method_code`.
Add `age_gate.kr_age_band`.
Add `age_gate.kr_token_expires_at`.
Add `age_gate.kr_content_class`.
Add `guardian.kr_verification_method`.
Add `guardian.kr_relationship_basis`.
Transform account-level consent into purpose rows.
Transform raw birthdate response into age-band response.
Transform guardian consent into separate ledger entry.
Transform marketing consent into optional revocable purpose.
Transform overseas transfer consent into transfer assessment linkage.
Transform sensitive information consent into category-specific evidence.
Transform CI/DI verification into identity assurance event.
Transform consent audit event into ADR-0263 scrubbed envelope.

## API Contract Deltas

`POST /kr/consents` captures purpose-specific consent.
`POST /kr/consents` requires `tenant_id`.
`POST /kr/consents` requires `subject_id`.
`POST /kr/consents` requires `purpose_code`.
`POST /kr/consents` requires `notice_version`.
`POST /kr/consents` requires `notice_language`.
`POST /kr/consents` requires `data_items`.
`POST /kr/consents` requires `retention_period`.
`POST /kr/consents` returns `consent_id`.
`POST /kr/consents` returns `audit_id`.
`POST /kr/consents` returns `cedar_policy_ids`.
`POST /kr/consents/{consent_id}/withdraw` withdraws optional consent.
`GET /kr/consents/{subject_id}` returns active and historical purpose rows.
`POST /kr/guardian-consents` captures guardian consent for a child under 14.
`POST /kr/guardian-consents/{guardian_consent_id}/withdraw` withdraws guardian consent where allowed.
`POST /kr/youth-age-checks` records age and identity verification result.
`GET /kr/youth-age-checks/{verification_id}` returns age-band and validity only.
`POST /kr/consents/overseas-transfer` captures separate overseas transfer consent.
`POST /kr/consents/sensitive` captures sensitive-information basis.
`POST /kr/consents/third-party-provision` captures third-party provision consent.
`POST /kr/consents/notice-version` records notice version digest.
Every consent API returns `jurisdiction_code=KR`.
Every consent API denies raw RRN as consent-only field.
Every consent API denies prechecked optional consent.
Every consent API denies missing Korean notice for Korean data subjects.
Every consent API emits an ADR-0263 event for state changes.
Every age API suppresses raw birthdate from ordinary service response.
Every guardian API suppresses raw guardian identity evidence from ordinary response.

## Audit Event Additions

`KrPipaConsentPresented` records notice version and purpose list.
`KrPipaConsentCaptured` records accepted purpose consent.
`KrPipaConsentDenied` records denied optional purpose.
`KrPipaConsentWithdrawn` records withdrawal.
`KrPipaConsentSuperseded` records replacement by newer notice version.
`KrPipaSensitiveConsentCaptured` records sensitive category basis.
`KrThirdPartyConsentCaptured` records recipient and purpose.
`KrCrossBorderTransferConsentCaptured` records destination and recipient class.
`KrGuardianConsentPresented` records under-14 consent notice.
`KrGuardianConsentCaptured` records guardian approval.
`KrGuardianConsentWithdrawn` records guardian withdrawal.
`KrGuardianConsentDenied` records missing or failed guardian approval.
`KrYouthAgeGateEvaluated` records age-band result and method.
`KrYouthRestrictedAccessDenied` records denied content class.
`KrConsentWithdrawalHonored` records stopped optional processing.
`KrConsentWithdrawalBlockedByLegalRetention` records legal retention exception.
`KrConsentNoticeVersionRecorded` records notice text digest.
`KrConsentPolicyDenied` records Cedar policy denial.
Every event carries `tenant_id`.
Every event carries `sub_scope_path` where scoped.
Every event carries `event_id`.
Every event carries `trace_id`.
Every event carries `span_id`.
Every event carries `audit_id`.
Every event carries `schema_version`.
Every event carries `source_microservice`.
Every event carries `cell_id`.
Every event carries `jurisdiction_code=KR`.
Every event payload is PII-scrubbed.

## Failure Modes specific to KR enforcement

Failure mode `KR-CONSENT-FM-001`: consent captured as one account-level flag.
Failure mode `KR-CONSENT-FM-002`: optional marketing bundled with required service consent.
Failure mode `KR-CONSENT-FM-003`: overseas transfer hidden inside general terms.
Failure mode `KR-CONSENT-FM-004`: sensitive information collected under ordinary consent.
Failure mode `KR-CONSENT-FM-005`: RRN collected under consent instead of statutory basis.
Failure mode `KR-CONSENT-FM-006`: Korean notice missing.
Failure mode `KR-CONSENT-FM-007`: notice version missing.
Failure mode `KR-CONSENT-FM-008`: data items missing from purpose row.
Failure mode `KR-CONSENT-FM-009`: retention period missing from purpose row.
Failure mode `KR-CONSENT-FM-010`: third-party recipient missing from third-party consent.
Failure mode `KR-CONSENT-FM-011`: overseas destination missing from transfer consent.
Failure mode `KR-CONSENT-FM-012`: subject under 14 processed without guardian consent.
Failure mode `KR-CONSENT-FM-013`: guardian relationship basis missing.
Failure mode `KR-CONSENT-FM-014`: youth-harmful content served without age gate.
Failure mode `KR-CONSENT-FM-015`: raw birthdate returned to service response.
Failure mode `KR-CONSENT-FM-016`: age verification token reused beyond scope.
Failure mode `KR-CONSENT-FM-017`: withdrawal ignored for optional processing.
Failure mode `KR-CONSENT-FM-018`: withdrawal deletes legally required evidence.
Failure mode `KR-CONSENT-FM-019`: consent audit event contains raw PII.
Failure mode `KR-CONSENT-FM-020`: state-changing consent API omits `audit_id`.
Failure mode `KR-CONSENT-FM-021`: purpose code does not map to service owner.
Failure mode `KR-CONSENT-FM-022`: consent record lacks tenant context.
Failure mode `KR-CONSENT-FM-023`: consent record lacks scoped context.
Failure mode `KR-CONSENT-FM-024`: consent language does not match displayed notice.
Failure mode `KR-CONSENT-FM-025`: consent screen has prechecked optional box.
Failure mode `KR-CONSENT-FM-026`: consent capture continues after notice version retired.
Failure mode `KR-CONSENT-FM-027`: CI/DI verification logged as public identifier.
Failure mode `KR-CONSENT-FM-028`: guardian consent authorizes marketing by default.
Failure mode `KR-CONSENT-FM-029`: sensitive consent used to inspect communications content.
Failure mode `KR-CONSENT-FM-030`: medical consent used without access-purpose trace.

## Worked Examples

### Scenario 1: Payroll Consent and Statutory RRN

An employee enters the Korean payroll onboarding flow.
The consent UI presents payroll processing as required.
The consent UI presents optional workforce analytics separately.
The consent UI presents Korean notice text.
The consent UI identifies data items and retention.
The employee accepts required payroll consent.
The employee rejects optional analytics consent.
The payroll workflow asks whether RRN is needed.
The RRN policy requires statutory basis.
The consent record does not authorize RRN by itself.
The statutory basis artifact is recorded separately.
The audit stream emits `KrPipaConsentCaptured`.
The audit stream emits `KrRrnStatutoryBasisAccepted`.
The analytics service cannot process the employee record under optional analytics purpose.

### Scenario 2: Child Under 14 Patient Portal

A patient portal account is created for a child under 14.
The age-band classifier marks the subject `under14`.
The portal blocks ordinary consent completion.
The guardian consent screen is presented in Korean.
The guardian relationship basis is captured.
The guardian approves patient portal use.
The consent ledger links child subject to guardian consent ID.
The audit stream emits `KrGuardianConsentCaptured`.
The portal allows only purposes covered by guardian consent.
Marketing remains disabled by default.
If guardian consent is withdrawn, optional portal processing stops.

### Scenario 3: Youth-Restricted Community Board

A Korean user opens a youth-harmful media board.
The board requires age and identity verification.
The user has no valid verification token.
The policy denies access.
The audit stream emits `KrYouthRestrictedAccessDenied`.
The user completes approved verification.
The result returned to the service is adult age band.
The raw birthdate is not returned.
The audit stream emits `KrYouthAgeGateEvaluated`.
The user receives access only for the verified content class.

### Scenario 4: Overseas Processor Consent

A tenant wants to use a foreign customer analytics processor.
The consent UI presents overseas transfer separately.
The consent UI names destination country.
The consent UI names recipient.
The consent UI names data items.
The consent UI names purpose and retention period.
The user declines overseas transfer.
The cross-border transfer policy denies export for that user.
The audit stream emits `KrCrossBorderTransferConsentCaptured` with declined status.
No data is sent to the processor.
The service may still use KR-local processing if another lawful basis applies.

### Scenario 5: Sensitive Medical Research Consent

A clinical workflow asks to use patient data for research.
The workflow classifies health data as sensitive.
The workflow presents medical treatment purpose separately from research purpose.
The patient accepts treatment use.
The patient declines optional research use.
The treatment workflow proceeds.
The research dataset excludes the patient.
The audit stream emits `KrPipaSensitiveConsentCaptured` for treatment basis.
The audit stream emits `KrPipaConsentDenied` for research purpose.
If later pseudonymized research is proposed, Article 28-2 analysis must be recorded separately.

## Cross-References

Pack overview: `packs/kr-localization/README.md`.
Regulatory coverage: `packs/kr-localization/regulatory-coverage.md`.
Data residency: `packs/kr-localization/data-residency.md`.
RRN handling: `packs/kr-localization/resident-id-number-rrn-handling.md`.
Incident response: `packs/kr-localization/cybersecurity-and-incident-response.md`.
ADR-0064 localization pack architecture: `docs/decisions/ADR-0709-general-live-apex.md`.
ADR-0244 tenant scoping: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
ADR-0251 compliance pack mechanics: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
ADR-0263 audit event emission: `docs/decisions/ADR-0706-observability-live-apex.md`.
KR pack manifest seed: `docs/localization-packs/kr/pack.yaml`.
Official PIPA source: `https://www.law.go.kr/`.
Official PIPC source: `https://www.pipc.go.kr/`.
Official KISA source: `https://www.kisa.or.kr/`.

## Consent Requirement Register

`KR-CONSENT-REQ-001` consent must be purpose-specific.
`KR-CONSENT-REQ-002` consent must be separately revocable for optional purposes.
`KR-CONSENT-REQ-003` consent must include Korean notice text.
`KR-CONSENT-REQ-004` consent must record notice version.
`KR-CONSENT-REQ-005` consent must record language version.
`KR-CONSENT-REQ-006` consent must record purpose code.
`KR-CONSENT-REQ-007` consent must record data items.
`KR-CONSENT-REQ-008` consent must record retention period.
`KR-CONSENT-REQ-009` consent must record third-party recipient where present.
`KR-CONSENT-REQ-010` consent must record overseas transfer details where present.
`KR-CONSENT-REQ-011` consent must record sensitive categories where present.
`KR-CONSENT-REQ-012` consent must record unique identifier categories where present.
`KR-CONSENT-REQ-013` consent must not authorize RRN without statutory basis.
`KR-CONSENT-REQ-014` consent must not be prechecked for optional purposes.
`KR-CONSENT-REQ-015` consent must not be hidden in general terms.
`KR-CONSENT-REQ-016` consent must not be account-only for multiple purposes.
`KR-CONSENT-REQ-017` consent must not be inferred from silence.
`KR-CONSENT-REQ-018` consent must not be inferred from continued use.
`KR-CONSENT-REQ-019` consent capture must return audit ID.
`KR-CONSENT-REQ-020` consent capture must return Cedar policies.
`KR-CONSENT-REQ-021` consent denial must return failure mode.
`KR-CONSENT-REQ-022` withdrawal must stop optional processing.
`KR-CONSENT-REQ-023` withdrawal must preserve legally required records.
`KR-CONSENT-REQ-024` withdrawal must emit audit event.
`KR-CONSENT-REQ-025` withdrawal must identify affected purposes.
`KR-CONSENT-REQ-026` guardian consent must apply for subjects under 14.
`KR-CONSENT-REQ-027` guardian consent must be separate from child interaction.
`KR-CONSENT-REQ-028` guardian consent must record relationship basis.
`KR-CONSENT-REQ-029` guardian consent must record guardian verification method.
`KR-CONSENT-REQ-030` guardian consent must record child subject link through scrubbed identifier.
`KR-CONSENT-REQ-031` guardian consent must not default to marketing.
`KR-CONSENT-REQ-032` guardian consent must not bypass youth restrictions.
`KR-CONSENT-REQ-033` age verification must occur before youth-harmful media access.
`KR-CONSENT-REQ-034` age verification must return age band.
`KR-CONSENT-REQ-035` age verification must not return raw birthdate to ordinary services.
`KR-CONSENT-REQ-036` age verification must record provider.
`KR-CONSENT-REQ-037` age verification must record method.
`KR-CONSENT-REQ-038` age verification must record token expiration.
`KR-CONSENT-REQ-039` age verification must record content class.
`KR-CONSENT-REQ-040` expired age token must deny access.
`KR-CONSENT-REQ-041` insufficient age proof must deny access.
`KR-CONSENT-REQ-042` minor result must deny youth-harmful content access.
`KR-CONSENT-REQ-043` sensitive information consent must be separate.
`KR-CONSENT-REQ-044` sensitive information consent must name category.
`KR-CONSENT-REQ-045` sensitive information consent must name purpose.
`KR-CONSENT-REQ-046` sensitive information consent must name retention.
`KR-CONSENT-REQ-047` sensitive information consent must name recipient where present.
`KR-CONSENT-REQ-048` sensitive information denial must block optional sensitive processing.
`KR-CONSENT-REQ-049` health data requires sensitive treatment.
`KR-CONSENT-REQ-050` biometric data requires sensitive treatment when used for identification.
`KR-CONSENT-REQ-051` cross-border consent must be separate.
`KR-CONSENT-REQ-052` cross-border consent must name destination.
`KR-CONSENT-REQ-053` cross-border consent must name recipient.
`KR-CONSENT-REQ-054` cross-border consent must name items.
`KR-CONSENT-REQ-055` cross-border consent must name purpose.
`KR-CONSENT-REQ-056` cross-border consent must name retention.
`KR-CONSENT-REQ-057` cross-border consent must name withdrawal method.
`KR-CONSENT-REQ-058` cross-border consent must link transfer assessment.
`KR-CONSENT-REQ-059` cross-border consent must not bypass residency policy.
`KR-CONSENT-REQ-060` third-party consent must name recipient.
`KR-CONSENT-REQ-061` third-party consent must name recipient purpose.
`KR-CONSENT-REQ-062` third-party consent must name provided items.
`KR-CONSENT-REQ-063` third-party consent must name retention by recipient.
`KR-CONSENT-REQ-064` marketing consent must be optional.
`KR-CONSENT-REQ-065` marketing consent must be withdrawable.
`KR-CONSENT-REQ-066` marketing denial must not block required service.
`KR-CONSENT-REQ-067` workforce analytics consent must be optional unless de-identified basis is separately recorded.
`KR-CONSENT-REQ-068` clinical trial consent must remain separate from treatment consent.
`KR-CONSENT-REQ-069` patient portal consent must not replace medical access reason.
`KR-CONSENT-REQ-070` communications consent must not authorize content inspection.
`KR-CONSENT-REQ-071` community consent must include moderation and youth-safety notice where applicable.
`KR-CONSENT-REQ-072` security monitoring notice must distinguish monitoring from optional analytics.
`KR-CONSENT-REQ-073` consent API must reject missing tenant ID.
`KR-CONSENT-REQ-074` consent API must reject missing purpose code.
`KR-CONSENT-REQ-075` consent API must reject missing notice version.
`KR-CONSENT-REQ-076` consent API must reject missing Korean notice for Korean subjects.
`KR-CONSENT-REQ-077` consent API must reject missing retention period.
`KR-CONSENT-REQ-078` consent API must reject missing transfer details for overseas purpose.
`KR-CONSENT-REQ-079` consent API must reject missing guardian link for under-14 subject.
`KR-CONSENT-REQ-080` consent API must reject prechecked optional flag.
`KR-CONSENT-REQ-081` consent records must be immutable after capture.
`KR-CONSENT-REQ-082` consent updates must supersede previous rows.
`KR-CONSENT-REQ-083` consent history must be retained as evidence.
`KR-CONSENT-REQ-084` consent revocation must create new row.
`KR-CONSENT-REQ-085` consent notice text digest must match rendered text.
`KR-CONSENT-REQ-086` consent component version must be recorded.
`KR-CONSENT-REQ-087` consent capture channel must be recorded.
`KR-CONSENT-REQ-088` consent source IP should be minimized or tokenized.
`KR-CONSENT-REQ-089` consent device data must be minimized.
`KR-CONSENT-REQ-090` consent audit must be PII-scrubbed.
`KR-CONSENT-REQ-091` consent audit must include jurisdiction code.
`KR-CONSENT-REQ-092` consent audit must include source microservice.
`KR-CONSENT-REQ-093` consent audit must include cell ID.
`KR-CONSENT-REQ-094` consent audit must include trace ID.
`KR-CONSENT-REQ-095` consent audit must include span ID.
`KR-CONSENT-REQ-096` consent audit must include schema version.
`KR-CONSENT-REQ-097` consent dashboard must expose active purpose state.
`KR-CONSENT-REQ-098` consent dashboard must expose withdrawal state.
`KR-CONSENT-REQ-099` consent dashboard must expose guardian state.
`KR-CONSENT-REQ-100` consent dashboard must expose overseas transfer state.
`KR-CONSENT-REQ-101` consent dashboard must expose sensitive data state.
`KR-CONSENT-REQ-102` consent dashboard must expose notice version.
`KR-CONSENT-REQ-103` consent dashboard must expose policy IDs.
`KR-CONSENT-REQ-104` consent migration must preserve history.
`KR-CONSENT-REQ-105` consent migration must not merge separate purposes.
`KR-CONSENT-REQ-106` consent migration must not convert opt-out to opt-in.
`KR-CONSENT-REQ-107` consent migration must not drop withdrawal records.
`KR-CONSENT-REQ-108` consent migration must not drop guardian records.
`KR-CONSENT-REQ-109` consent migration must not drop notice digests.
`KR-CONSENT-REQ-110` consent migration must emit checkpoint event.
`KR-CONSENT-REQ-111` service onboarding must register purpose codes.
`KR-CONSENT-REQ-112` service onboarding must register notice strings.
`KR-CONSENT-REQ-113` service onboarding must register audit events.
`KR-CONSENT-REQ-114` service onboarding must register withdrawal handling.
`KR-CONSENT-REQ-115` service onboarding must register failure modes.
`KR-CONSENT-REQ-116` service onboarding must register data classes.
`KR-CONSENT-REQ-117` service onboarding must register processor recipients.
`KR-CONSENT-REQ-118` service onboarding must register overseas destinations.
`KR-CONSENT-REQ-119` service onboarding must register minor handling.
`KR-CONSENT-REQ-120` service onboarding must register legal review owner.

## Checkpoint

This file is scoped to `/packs/kr-localization/`.
It does not edit ADRs.
It does not edit microservices.
It does not edit other packs.
It must be line-count verified with the rest of KR-PACK-1.
It must be lifecycle-verified with Oya VCS after all six docs exist.
