---
doc_class: LocalizationPack
pack_id: KR-PACK-1
doc_id: KR-PACK-1-README
title: Korea Localization Pack Overview
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
  - https://isms.kisa.or.kr/main/csap/intro/index.jsp
  - https://www.kisa.or.kr/
---

# Korea Localization Pack Overview

KR-PACK-1 is the first jurisdictional localization pack for Oyatie.
It activates Korea-specific regulatory controls, data placement rules, consent workflows, identifier handling, cybersecurity response rules, audit emissions, and pack precedence for tenants operating in the Republic of Korea.
This README is the root orientation document for `/packs/kr-localization/`.
It does not replace canonical ADRs, runtime schemas, or Cedar fragments.
It gives implementers the human-readable pack map that every machine-readable control must trace back to.

## Pack Identity

Pack ID: `KR-PACK-1`.
Pack short name: `kr-localization`.
Pack jurisdiction: Republic of Korea.
Pack locale baseline: `ko-KR`.
Pack language baseline: Korean primary, English administrative fallback.
Pack status: `canonical-draft`.
Pack owner: Oyatie compliance localization lane.
Pack authority model: official law.go.kr, PIPC, KISA, and CSAP sources override secondary summaries.
Pack precedence model: ADR-0064 canonical base plus jurisdictional pack.
Pack scoping model: ADR-0244 tenant and sub-scope universal scoping.
Pack compliance model: ADR-0251 versioned compliance pack installed by tenant and enforced by Cedar.
Pack observability model: ADR-0263 PII-scrubbed audit emissions.
Pack externality rule: no production tenant claim may rely on this README without a matching signed pack bundle.

## Scope

KR-PACK-1 governs Korean personal information processing.
KR-PACK-1 governs Korean resident registration number handling.
KR-PACK-1 governs Korean consent presentation and withdrawal.
KR-PACK-1 governs minor and youth access restrictions.
KR-PACK-1 governs Korean health record localization when healthcare services are active.
KR-PACK-1 governs Korean communications metadata restrictions when communications services are active.
KR-PACK-1 governs Korean electronic document evidentiary retention when document services are active.
KR-PACK-1 governs CSAP-aligned cell placement when public-sector or CSAP-sensitive workloads are active.
KR-PACK-1 governs KISA/PIPC incident reporting workflows when security or privacy incidents affect Korean records.
KR-PACK-1 does not create a generic APAC policy.
KR-PACK-1 does not authorize offshoring of Korean regulated data by default.
KR-PACK-1 does not relax canonical base security controls.
KR-PACK-1 does not override stronger tenant-specific contractual restrictions.
KR-PACK-1 does not license collection of resident registration numbers without explicit statutory basis.
KR-PACK-1 does not permit broad consent bundling.
KR-PACK-1 does not permit silent age-gate bypass for youth-protected content.
KR-PACK-1 does not permit medical record export without medical-law traceability.
KR-PACK-1 does not permit communications-secret content inspection outside lawful processing basis.
KR-PACK-1 does not permit audit payloads to include raw PII.

## Version

This documentation version is `1.0.0`.
The version date is 2026-05-20.
The pack is designed as the canonical documentation companion for the first KR localization bundle.
The effective runtime bundle must carry an independent signed bundle version.
Documentation version changes do not activate tenant policy by themselves.
Runtime policy activation requires an installed compliance pack and Cedar policy set.
Breaking semantic changes require a new pack bundle version.
Non-breaking citation clarifications may use patch version increments.
Emergency law corrections may ship as policy hotfixes with explicit audit checkpoint.
Every runtime bundle must record source authority snapshot date.
Every runtime bundle must record Cedar policy digest.
Every runtime bundle must record schema digest.
Every runtime bundle must record validation evidence digest.
Every runtime bundle must record ADR references.
Every runtime bundle must record pack precedence order.

## Pack Precedence

Canonical base controls load first.
Tenant contractual controls load after canonical base controls.
KR-PACK-1 loads after tenant-independent canonical base controls.
KR-PACK-1 jurisdictional deny policies override canonical allow policies.
Tenant-specific stricter Korean restrictions override KR-PACK-1 allow policies.
Emergency legal holds override normal retention deletion.
Court or regulator preservation orders override business retention windows.
Medical Service Act retention overrides generic business record disposal.
Communications Secrets Protection Act restrictions override analytics feature access.
Resident registration number restrictions override identity convenience flows.
Youth-protection age gates override general content availability.
CSAP cell pinning overrides generic multi-region balancing.
Cross-border transfer restrictions override generic processor selection.
PIPA sensitive information consent overrides generic consent terms.
PIPA data-subject rights override convenience-only internal workflow queues.
PIPA breach notification duties override ordinary incident backlog priority.
ADR-0263 emission scrubbing overrides local operator debugging preferences.
ADR-0244 tenant/sub-scope routing overrides service-local tenancy shortcuts.
ADR-0251 pack installation state overrides ad hoc environment flags.
Where two Korean laws conflict, legal counsel escalation is mandatory before product release.

## Activated Microservices

`hr` activates KR employee records, labor-linked identifiers, hiring consent, and payroll handoff controls.
`payroll` activates KR payroll identity, tax, four-major-insurance, RRN minimization, and statutory payroll retention controls.
`accounting` activates KR electronic document retention, tax invoice evidence, and K-GAAP localization controls.
`ats` activates KR applicant consent, sensitive background-check data minimization, and recruiting retention controls.
`grc` activates KR regulatory evidence, CSAP evidence, PIPA evidence, and incident obligation tracking controls.
`performance` activates KR employee appraisal processing, consent and retention controls for employment context.
`workforce-analytics` activates KR de-identification, aggregation threshold, and workforce privacy controls.
`medical` activates KR Medical Service Act record creation, retention, export, and access-trace controls.
`pharmacy` activates KR prescription data, patient identifier minimization, and healthcare audit controls.
`patient` activates KR patient portal consent, guardian consent, and medical-record access controls.
`emergency` activates KR emergency care lawful-basis capture and emergency disclosure audit controls.
`clinical` activates KR clinical trial consent and sensitive health-data controls.
`healthcare-portal` activates KR patient identity assurance and healthcare document retention controls.
`connector` activates Korean communications metadata, message privacy, user consent, and youth safety controls.
`community` activates youth-protected content gating, takedown workflows, and Korean community moderation audit controls.
`payments` activates KR payment personal data, transaction retention, and breach criteria controls.
`insurance` activates KR claims health data, sensitive information consent, and lawful disclosure controls.
`finance-quant` activates KR financial data residency, de-identification, and export-denial controls.
`settlement` activates KR transaction evidence, electronic document preservation, and audit trace controls.
`manufacturing` activates KR industrial facility data, worker safety data, and controlled-source incident controls.
`logistics` activates KR delivery address, location, customs handoff, and cross-border transfer controls.
`facility-ops` activates KR premises access logs, CCTV metadata, and local retention controls.
`procurement` activates KR vendor data, electronic procurement documents, and processor due-diligence controls.
`security` activates KR incident detection, KISA reporting, CSAP evidence, and controlled-source breach controls.
`hospitality` activates KR guest identity minimization, youth restrictions, and payment data retention controls.
`dining` activates KR reservation contact data, minor restrictions for age-limited goods, and receipt evidence controls.
`cellar` activates KR alcohol-related youth-protection age verification and inventory compliance controls.

## Authority Citations

The authority snapshot for this documentation was refreshed on 2026-05-20.
Primary statutory text source: National Law Information Center, `law.go.kr`.
Primary privacy regulator source: Personal Information Protection Commission, `pipc.go.kr`.
Primary cybersecurity and incident intake source: Korea Internet and Security Agency, `kisa.or.kr`.
Primary CSAP source: KISA ISMS and CSAP portal, `isms.kisa.or.kr`.
Personal Information Protection Act is cited as `개인정보 보호법`.
PIPA Article 15 governs collection and use of personal information.
PIPA Article 17 governs provision of personal information to a third party.
PIPA Article 22 governs consent method and consent separation.
PIPA Article 22-2 governs processing of personal information of children under 14.
PIPA Article 23 governs sensitive information restrictions.
PIPA Article 24 governs unique identifying information restrictions.
PIPA Article 24-2 governs resident registration number processing restrictions.
PIPA Article 28-2 governs pseudonymous information processing.
PIPA Article 28-8 governs overseas transfer of personal information.
PIPA Article 34 governs notification and reporting of personal information leakage.
PIPA Enforcement Decree Article 40 governs 72-hour reporting to PIPC or KISA for enumerated leakage cases.
Cloud Computing Development and User Protection Act Article 23-2 governs CSAP security certification.
KISA CSAP portal describes cloud security certification for cloud services under Article 23-2.
Youth Protection Act is cited as `청소년 보호법`.
Youth Protection Act Article 16 governs sale, rental, distribution, viewing, watching, or use of youth-harmful media and age/identity verification.
Youth Protection Act Enforcement Decree Article 17 names permitted age and identity verification methods.
Communications Secrets Protection Act is cited as `통신비밀보호법`.
Communications Secrets Protection Act controls interception, communications confirmation data, and secrecy of communications.
Digital Documents and Transactions Act is cited as `전자문서 및 전자거래 기본법`.
Digital Documents and Transactions Act governs electronic document legal effect and certified electronic document center obligations.
Information and Communications Network Act is cited as `정보통신망 이용촉진 및 정보보호 등에 관한 법률`.
Information Network Act Article 23-2 and Article 23-3 lineage informs non-RRN identity verification and identity confirmation agency controls.
Information Network Act Article 48-3 lineage informs incident reporting and post-incident cooperation.
Medical Service Act is cited as `의료법`.
Medical Service Act Article 22 governs medical records.
Medical Service Act Article 23 governs electronic medical records.
Medical Service Act Article 34 governs remote medical support responsibilities.
KR Cyber Security Act mapping for this pack uses Information and Communications Infrastructure Protection Act, `정보통신기반 보호법`, for critical infrastructure incident controls.
Information and Communications Infrastructure Protection Act Article 8 governs designation of major information and communications infrastructure.
Information and Communications Infrastructure Protection Act Article 13 governs notification of incidents affecting major information and communications infrastructure.
Information and Communications Infrastructure Protection Act Enforcement Decree Article 21 governs required incident notification contents.
Named effective dates must be read from the cited law.go.kr text at bundle build time.
This README records the 2026-05-20 source snapshot date rather than freezing every statute's future amended date.
If law.go.kr shows a later effective date at bundle build, the bundle must update `authority_snapshot`.
If PIPC guidance changes the cross-border transfer basis, KR-PACK-1 must update cross-border transfer controls before release.
If KISA changes CSAP certification status or control group names, KR-PACK-1 must update cell certification controls before release.
If a Korean ministry issues sector-specific rules stricter than this README, the sector pack fragment must override this overview.

## Activated Cedar Policies

`pack-kr-pack-1-activate` fires when tenant installs `KR-PACK-1`.
`pack-kr-pack-1-deny-uninstalled-use` denies KR-only data classes before pack installation.
`pack-kr-pack-1-tenant-subscope-required` requires ADR-0244 tenant and sub-scope context.
`pack-kr-pack-1-cell-kr-residency` pins KR-regulated primary data to approved KR cells.
`pack-kr-pack-1-csap-cell-pinning` requires CSAP-capable cell certification when `public_sector=true`.
`pack-kr-pack-1-kisa-mid-cell` permits KISA-MID placement only for approved mid-sensitivity workloads.
`pack-kr-pack-1-kisa-bio-cell` permits KISA-BIO placement only for biometric and higher-sensitivity workloads.
`pack-kr-pack-1-icn-mid-cell` permits ICN-MID placement only for interconnect metadata workloads.
`pack-kr-pack-1-pipa-purpose-consent` requires enumerated purpose consent.
`pack-kr-pack-1-pipa-separate-consent` denies bundled consent for separate purpose classes.
`pack-kr-pack-1-pipa-sensitive-consent` requires explicit sensitive-information basis.
`pack-kr-pack-1-pipa-under14-guardian` requires legal guardian consent for children under 14.
`pack-kr-pack-1-youth-age-gate` requires age and identity verification for youth-harmful content.
`pack-kr-pack-1-youth-content-deny` denies minor access to youth-prohibited media.
`pack-kr-pack-1-rrn-collection-deny-default` denies resident registration number collection by default.
`pack-kr-pack-1-rrn-statutory-basis` permits RRN only with statutory basis evidence.
`pack-kr-pack-1-rrn-hash-only` requires irreversible hashed derivative storage unless raw retention is legally mandatory.
`pack-kr-pack-1-ci-di-preferred` prefers CI/DI or non-RRN identity tokens when identity assurance is needed.
`pack-kr-pack-1-cross-border-transfer-deny-default` denies overseas transfer without lawful basis.
`pack-kr-pack-1-cross-border-transfer-consent` permits transfer with valid separate consent and transfer notice.
`pack-kr-pack-1-cross-border-transfer-adequacy` permits transfer to recognized adequate destination when PIPC basis is recorded.
`pack-kr-pack-1-cross-border-transfer-scc` permits processor transfer with KR SCC or equivalent contract artifact.
`pack-kr-pack-1-medical-record-locality` pins medical records to KR-approved healthcare cells.
`pack-kr-pack-1-medical-record-access-trace` requires signed access reason for medical record reads.
`pack-kr-pack-1-communications-secret-deny-content-inspection` denies communication content inspection without legal basis.
`pack-kr-pack-1-communications-metadata-retention` restricts retention of communications confirmation metadata.
`pack-kr-pack-1-electronic-document-evidence` requires preservation metadata for Korean electronic documents.
`pack-kr-pack-1-info-network-security-measures` requires security-measure evidence for information network services.
`pack-kr-pack-1-incident-kisa-triage` requires KISA triage path for Korean controlled-source incidents.
`pack-kr-pack-1-pipa-breach-reporting-window` requires PIPC/KISA report clock on reportable personal-data leakage.
`pack-kr-pack-1-pii-emission-scrub` enforces ADR-0263 PII scrubbing.
`pack-kr-pack-1-audit-tenant-context` denies audit emission without tenant context.
`pack-kr-pack-1-audit-jurisdiction-code` requires `jurisdiction_code=KR`.
`pack-kr-pack-1-deidentified-analytics-threshold` requires de-identification threshold for KR analytics.
`pack-kr-pack-1-processor-due-diligence` requires Korean processor diligence artifact.
`pack-kr-pack-1-lawful-disclosure-log` requires legal basis for regulator, court, or emergency disclosure.
`pack-kr-pack-1-retention-legal-hold` freezes deletion during Korean legal hold.
`pack-kr-pack-1-consent-withdrawal-honor` denies continued optional processing after consent withdrawal.
`pack-kr-pack-1-localized-notice-required` requires Korean-language privacy notice for Korean data subjects.
`pack-kr-pack-1-pack-precedence-deny-wins` makes KR deny policies prevail over base allow policies.

## Data Model Deltas

Add `tenant.compliance_packs[]` value `KR-PACK-1`.
Add `tenant.kr_pack_status`.
Add `tenant.kr_authority_snapshot_date`.
Add `tenant.kr_primary_cell_id`.
Add `tenant.kr_disaster_recovery_cell_id`.
Add `tenant.kr_csap_level`.
Add `tenant.kr_public_sector_flag`.
Add `subject.kr_residency_status`.
Add `subject.kr_age_band`.
Add `subject.kr_under_14_flag`.
Add `subject.kr_youth_protection_restricted_flag`.
Add `subject.kr_guardian_consent_id`.
Add `consent.kr_purpose_code`.
Add `consent.kr_purpose_text_ko`.
Add `consent.kr_purpose_text_en`.
Add `consent.kr_separate_consent_required`.
Add `consent.kr_sensitive_data_basis`.
Add `consent.kr_cross_border_basis`.
Add `consent.kr_cross_border_destination`.
Add `consent.kr_cross_border_recipient`.
Add `consent.kr_cross_border_retention_period`.
Add `identity.kr_rrn_present_flag`.
Add `identity.kr_rrn_statutory_basis_code`.
Add `identity.kr_rrn_hash_digest`.
Add `identity.kr_rrn_hash_key_version`.
Add `identity.kr_ci_token`.
Add `identity.kr_di_token`.
Add `identity.kr_identity_provider_code`.
Add `identity.kr_identity_assurance_event_id`.
Add `data_class.PI_KR_PIPA`.
Add `data_class.PI_KR_SENSITIVE`.
Add `data_class.PI_KR_UNDER14`.
Add `data_class.PI_KR_MINOR_14_18`.
Add `data_class.PI_KR_RRN`.
Add `data_class.PI_KR_CI`.
Add `data_class.PI_KR_DI`.
Add `data_class.PI_KR_BIOMETRIC`.
Add `data_class.PI_KR_MEDICAL_RECORD`.
Add `data_class.PI_KR_COMMUNICATION_METADATA`.
Add `data_class.PI_KR_ELECTRONIC_DOCUMENT`.
Add `data_class.SEC_KR_INCIDENT_ARTIFACT`.
Transform Korean phone numbers into region-normalized contact fields.
Transform Korean addresses into road-name address plus legacy-lot optional fields.
Transform raw RRN values into immediate validation result plus irreversible derivative.
Transform consent records into separately revocable purpose records.
Transform cross-border processor references into approval artifacts.
Transform audit payloads into ADR-0263 scrubbed event envelopes.
Transform medical record access reasons into signed access ledger entries.
Transform communications metadata into minimization-limited records.
Transform incident evidence into controlled-source evidence envelopes.
Transform CSAP evidence into cell-certification records.

## API Contract Deltas

`GET /localization-packs/KR-PACK-1` returns pack metadata and installed policy digest.
`POST /tenants/{tenant_id}/packs/KR-PACK-1/install` requires authority snapshot and bundle digest.
`GET /tenants/{tenant_id}/kr/compliance-state` returns pack activation, CSAP, cell, and breach-clock state.
`POST /kr/consents` captures purpose-specific consent in Korean and English notice context.
`POST /kr/consents/{consent_id}/withdraw` withdraws optional purpose consent.
`GET /kr/consents/{subject_id}` returns purpose-level consent ledger entries.
`POST /kr/guardian-consents` captures guardian consent for a child under 14.
`POST /kr/youth-age-checks` records age and identity verification for restricted media.
`POST /kr/identity/rrn/validate` validates RRN format without persisting raw RRN.
`POST /kr/identity/rrn/hash` returns irreversible derivative only under statutory basis.
`POST /kr/identity/ci-di/link` links CI/DI alternative identifiers.
`POST /kr/cross-border-transfer-assessments` evaluates consent, adequacy, SCC, and processor basis.
`GET /kr/cross-border-transfer-assessments/{id}` returns transfer basis and audit references.
`POST /kr/data-residency/evaluate` returns cell pinning decision.
`GET /kr/csap/evidence/{tenant_id}` returns CSAP evidence digest and certification state.
`POST /kr/medical-record-access` captures access purpose before medical record read.
`POST /kr/electronic-document/preserve` records electronic document retention and evidentiary metadata.
`POST /kr/incidents/classify` maps a security event to KR incident classes.
`POST /kr/incidents/{incident_id}/kisa-notification` records KISA notification state.
`POST /kr/incidents/{incident_id}/pipc-notification` records PIPC notification state.
`GET /kr/audit/events/{audit_id}` returns scrubbed KR audit envelope.
Every KR API requires `tenant_id`.
Every KR API requires `sub_scope_path` when action touches scoped data.
Every state-changing KR API returns `audit_id`.
Every state-changing KR API returns `jurisdiction_code=KR`.
Every KR API that evaluates policy returns `cedar_policy_ids`.
Every KR API that denies an action returns a named legal control reference.
Every KR API that touches consent returns the consent language version.
Every KR API that touches minors returns age-band classification, not raw birthdate.
Every KR API that touches RRN must exclude raw RRN from response bodies.
Every KR API that touches incident evidence must exclude raw exploit payloads from routine audit output.

## Audit Event Additions

All KR audit events must follow ADR-0263.
Every KR audit event carries `tenant_id`.
Every KR audit event carries `sub_scope_path`.
Every KR audit event carries `event_id`.
Every KR audit event carries `trace_id`.
Every KR audit event carries `span_id`.
Every KR audit event carries `audit_id`.
Every KR audit event carries `schema_version`.
Every KR audit event carries `source_microservice`.
Every KR audit event carries `cell_id`.
Every KR audit event carries `jurisdiction_code=KR`.
Every KR audit event carries a PII-scrubbed payload.
`KrPackActivated` records pack installation, bundle digest, authority snapshot, and policy digest.
`KrPackDeactivated` records deactivation request and legal-hold check.
`KrPackCellPinned` records primary and disaster recovery cell decisions.
`KrCsapEvidencePulled` records CSAP evidence digest without embedding raw certificate material.
`KrPipaConsentCaptured` records subject, purpose, notice version, and lawful basis.
`KrPipaConsentWithdrawn` records withdrawal timestamp and affected processing purposes.
`KrPipaSensitiveConsentCaptured` records sensitive data category without raw sensitive payload.
`KrGuardianConsentCaptured` records guardian relationship basis and minor age band.
`KrYouthAgeGateEvaluated` records age-gate result without raw birthdate.
`KrYouthRestrictedAccessDenied` records denied youth-restricted content class.
`KrRrnCollectionDenied` records attempted collection context and denial reason.
`KrRrnStatutoryBasisAccepted` records authority basis code and reviewer.
`KrRrnHashDerived` records hash key version and irreversible derivative fingerprint only.
`KrCiDiLinked` records alternative identity provider and token class.
`KrCrossBorderTransferDenied` records destination, recipient class, and missing basis.
`KrCrossBorderTransferApproved` records basis type and approval artifact digest.
`KrMedicalRecordAccessed` records access purpose, practitioner role, and patient scope.
`KrElectronicDocumentPreserved` records document class, retention rule, and evidence digest.
`KrCommunicationsMetadataAccessed` records lawful basis and minimization scope.
`KrControlledSourceIncidentClassified` records incident source class and severity.
`KrIncidentContainmentStarted` records containment control family and accountable role.
`KrBreachKisaNotified` records KISA notification clock and report reference.
`KrBreachPipcNotified` records PIPC notification clock and report reference.
`KrDataSubjectRequestReceived` records request class and deadline.
`KrDataSubjectRequestCompleted` records outcome and legal exemption if denied.
`KrProcessorDueDiligenceApproved` records processor evidence digest.
`KrProcessorDueDiligenceDenied` records rejected processor reason.
`KrLegalHoldApplied` records hold authority and affected data classes.
`KrLegalHoldReleased` records release authority and disposal queue result.

## Failure Modes specific to KR enforcement

Failure mode `KR-FM-001`: tenant has Korean data but no installed KR pack.
Failure mode `KR-FM-002`: tenant installs KR pack without authority snapshot.
Failure mode `KR-FM-003`: policy evaluates without tenant_id.
Failure mode `KR-FM-004`: policy evaluates without sub_scope_path for scoped record.
Failure mode `KR-FM-005`: KR regulated data routes to non-KR primary cell.
Failure mode `KR-FM-006`: CSAP workload routes to uncertified cell.
Failure mode `KR-FM-007`: consent purpose bundled across unrelated processing purposes.
Failure mode `KR-FM-008`: sensitive information collected under generic consent.
Failure mode `KR-FM-009`: child under 14 processed without guardian consent.
Failure mode `KR-FM-010`: youth-restricted media served without age/identity check.
Failure mode `KR-FM-011`: raw RRN persisted without statutory basis.
Failure mode `KR-FM-012`: RRN derivative is reversible or unsalted.
Failure mode `KR-FM-013`: CI/DI token logged as ordinary user-visible identifier.
Failure mode `KR-FM-014`: cross-border transfer occurs without consent, adequacy, SCC, or statutory basis.
Failure mode `KR-FM-015`: processor subprocessors not recorded for overseas transfer.
Failure mode `KR-FM-016`: medical record access lacks signed purpose.
Failure mode `KR-FM-017`: communications metadata retained past KR minimization window.
Failure mode `KR-FM-018`: electronic document lacks preservation evidence metadata.
Failure mode `KR-FM-019`: incident classifier misses PIPA breach clock.
Failure mode `KR-FM-020`: KISA notification event lacks report reference.
Failure mode `KR-FM-021`: audit emission carries raw PII.
Failure mode `KR-FM-022`: audit event omits jurisdiction code.
Failure mode `KR-FM-023`: deletion proceeds despite Korean legal hold.
Failure mode `KR-FM-024`: de-identified analytics uses group size below threshold.
Failure mode `KR-FM-025`: localized Korean privacy notice version missing.
Failure mode `KR-FM-026`: tenant-specific stricter restriction ignored.
Failure mode `KR-FM-027`: law source changed after bundle build with no checkpoint.
Failure mode `KR-FM-028`: service activates sector data class without sector law mapping.
Failure mode `KR-FM-029`: incident evidence exported to non-KR debugging workspace.
Failure mode `KR-FM-030`: human operator bypasses Cedar denial through service-local flag.

## Worked Examples

### Scenario 1: Korean Payroll Tenant Activation

Tenant `kr-payroll-demo` installs `KR-PACK-1`.
The install API records `authority_snapshot_date=2026-05-20`.
The install API records the Cedar policy digest.
The install API records the schema digest.
The tenant has `payroll`, `hr`, and `accounting` active.
The policy engine loads canonical base first.
The policy engine loads KR deny policies after base allow policies.
The payroll service attempts to collect employee identity data.
The employee identity workflow requests RRN.
The RRN policy denies collection until statutory basis code is present.
The payroll workflow supplies statutory payroll basis evidence.
The RRN policy permits one-time validation.
The RRN policy requires irreversible derivative creation.
The API returns no raw RRN.
The audit stream emits `KrRrnStatutoryBasisAccepted`.
The audit stream emits `KrRrnHashDerived`.
The payroll record stores `PI_KR_RRN` derivative only.
The tenant remains pinned to a KR primary cell.
The evidence ledger links the transaction to ADR-0244, ADR-0251, and ADR-0263.

### Scenario 2: Youth-Restricted Community Content

The community service hosts a board marked youth-restricted.
A Korean user requests access.
The Cedar engine evaluates `pack-kr-pack-1-youth-age-gate`.
The request lacks verified age and identity evidence.
The service denies access.
The denial reason cites Youth Protection Act Article 16.
The API returns a localized Korean denial message.
The audit event is `KrYouthRestrictedAccessDenied`.
The event records content class and policy ID.
The event does not record raw identity document details.
The user completes an approved age verification path.
The service records `KrYouthAgeGateEvaluated`.
The service grants adult access only after age-band result is adult.
If the result is minor, the denial remains in force.
The workflow never asks for RRN as a convenience shortcut.

### Scenario 3: Cross-Border Processor Request

A tenant tries to use a non-Korean analytics processor.
The dataset includes Korean personal information.
The data class is `PI_KR_PIPA`.
The transfer assessment API evaluates destination, recipient, retention, and safeguards.
The tenant lacks separate cross-border consent.
The tenant lacks an adequacy basis.
The tenant lacks a KR SCC or equivalent transfer contract.
The Cedar policy denies export.
The API returns `KR-FM-014`.
The audit event is `KrCrossBorderTransferDenied`.
The payload records processor class and missing basis only.
The payload excludes personal data samples.
If the tenant later supplies valid SCC evidence, the approval event records artifact digest.
The approval remains scoped to the named processor and data class.
Subprocessor changes require a new assessment.

### Scenario 4: Medical Record Access

A clinician opens a Korean patient record.
The medical service tags the record `PI_KR_MEDICAL_RECORD`.
The Cedar policy requires KR cell residency.
The Cedar policy requires signed access purpose.
The request includes practitioner role.
The request includes treatment purpose.
The request includes tenant and patient sub-scope.
The access policy permits the read.
The audit event is `KrMedicalRecordAccessed`.
The audit event records purpose, role, and record category.
The audit event does not include diagnosis text.
If the request lacks purpose, access is denied.
If the record is in a non-KR cell, access is denied and incident triage starts.

### Scenario 5: Reportable Personal Data Leakage

The security service detects unauthorized export from a Korean tenant.
The incident classifier identifies personal data leakage.
The classifier checks affected subject count.
The classifier checks whether sensitive or unique identifying information is involved.
The classifier checks whether external illegal access occurred.
The PIPA breach clock starts.
The KR controlled-source incident path starts.
The incident owner receives KISA/PIPC notification tasks.
The audit stream emits `KrControlledSourceIncidentClassified`.
The audit stream emits `KrIncidentContainmentStarted`.
The audit stream emits `KrBreachKisaNotified` when notification is filed.
The audit stream emits `KrBreachPipcNotified` when PIPC reporting is filed.
The event payloads carry report references, not raw leaked records.
Post-incident deletion is blocked until legal hold is resolved.

## Cross-References

Canonical base localization decision: `docs/decisions/ADR-0709-general-live-apex.md`.
Tenant scoping decision: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Compliance pack and cell certification decision: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
Observability emission contract: `docs/decisions/ADR-0706-observability-live-apex.md`.
Existing KR planning source: `docs/localization-packs/kr.md`.
Existing KR pack manifest source: `docs/localization-packs/kr/pack.yaml`.
Regional pack precedent: `docs/regional-packs/oya-pack-kr/PACK.md`.
Compliance pack schema: `specs/compliance-pack-schema.json`.
Cedar fragment schema: `specs/cedar-fragment-schema.json`.
Audit event class registry: `specs/audit-event-class-registry.json`.
Data residency detail: `packs/kr-localization/data-residency.md`.
Consent detail: `packs/kr-localization/consent-management.md`.
RRN detail: `packs/kr-localization/resident-id-number-rrn-handling.md`.
Cybersecurity detail: `packs/kr-localization/cybersecurity-and-incident-response.md`.
Regulatory matrix detail: `packs/kr-localization/regulatory-coverage.md`.
PIPA authority source: `https://www.law.go.kr/`.
PIPC authority source: `https://www.pipc.go.kr/`.
KISA authority source: `https://www.kisa.or.kr/`.
CSAP authority source: `https://isms.kisa.or.kr/main/csap/intro/index.jsp`.

## Canonical Requirement Register

`KR-README-REQ-001` records that KR-PACK-1 is localization pack number one.
`KR-README-REQ-002` records that the pack is jurisdictional, not product-specific.
`KR-README-REQ-003` records that canonical base policy must be installed first.
`KR-README-REQ-004` records that Korean deny controls override generic allow controls.
`KR-README-REQ-005` records that tenant-specific stricter controls override pack defaults.
`KR-README-REQ-006` records that legal holds override deletion schedules.
`KR-README-REQ-007` records that authority sources must be official where possible.
`KR-README-REQ-008` records that PIPA is the privacy baseline for Korean personal data.
`KR-README-REQ-009` records that CSAP is the cloud assurance baseline for CSAP-sensitive cells.
`KR-README-REQ-010` records that Youth Protection Act age gates apply to youth-harmful media.
`KR-README-REQ-011` records that Communications Secrets Protection Act controls communication secrecy.
`KR-README-REQ-012` records that Digital Documents and Transactions Act controls electronic document evidence.
`KR-README-REQ-013` records that Information Network Act lineage controls network service safeguards.
`KR-README-REQ-014` records that Medical Service Act controls medical record handling.
`KR-README-REQ-015` records that critical-infrastructure cyber incidents map to ICIPA controls.
`KR-README-REQ-016` records that every KR policy decision must include `tenant_id`.
`KR-README-REQ-017` records that every scoped KR policy decision must include `sub_scope_path`.
`KR-README-REQ-018` records that every KR state change must return `audit_id`.
`KR-README-REQ-019` records that every KR audit event must carry `jurisdiction_code=KR`.
`KR-README-REQ-020` records that raw PII is forbidden in routine audit payloads.
`KR-README-REQ-021` records that RRN collection is denied by default.
`KR-README-REQ-022` records that statutory RRN basis must be recorded before processing.
`KR-README-REQ-023` records that RRN derivatives must be irreversible.
`KR-README-REQ-024` records that CI/DI identifiers are preferred where sufficient.
`KR-README-REQ-025` records that consent must be purpose-specific.
`KR-README-REQ-026` records that consent withdrawal must stop optional processing.
`KR-README-REQ-027` records that children under 14 require guardian consent.
`KR-README-REQ-028` records that age-band results should replace raw birthdate in service responses.
`KR-README-REQ-029` records that cross-border transfers are denied by default.
`KR-README-REQ-030` records that cross-border consent must be separate and explicit.
`KR-README-REQ-031` records that adequacy basis must name regulator recognition.
`KR-README-REQ-032` records that SCC basis must name the contract artifact.
`KR-README-REQ-033` records that CSAP workloads require certified cells.
`KR-README-REQ-034` records that KR public-sector workloads require stricter cell review.
`KR-README-REQ-035` records that KISA-MID, KISA-BIO, and ICN-MID are placement labels, not laws.
`KR-README-REQ-036` records that placement labels must map to evidence-backed cell capabilities.
`KR-README-REQ-037` records that medical records require signed access purpose.
`KR-README-REQ-038` records that electronic documents require preservation metadata.
`KR-README-REQ-039` records that communications metadata minimization applies by default.
`KR-README-REQ-040` records that incident notification clocks start at detection or classification trigger.
`KR-README-REQ-041` records that PIPA breach reporting clocks must be tracked independently.
`KR-README-REQ-042` records that KISA incident notification workflow must preserve report references.
`KR-README-REQ-043` records that controlled-source incident evidence must be scrubbed before audit emission.
`KR-README-REQ-044` records that analytics must use de-identification or aggregation thresholds.
`KR-README-REQ-045` records that processor due diligence is required for Korean processors.
`KR-README-REQ-046` records that overseas subprocessors require transfer basis review.
`KR-README-REQ-047` records that Korean notices must be available in Korean.
`KR-README-REQ-048` records that English fallback notices do not replace Korean notices.
`KR-README-REQ-049` records that policy IDs must be named in denials.
`KR-README-REQ-050` records that service-local bypass flags must not override Cedar.
`KR-README-REQ-051` records that pack bundle activation must be signed.
`KR-README-REQ-052` records that pack bundle activation must be auditable.
`KR-README-REQ-053` records that every bundle must include schema digest.
`KR-README-REQ-054` records that every bundle must include policy digest.
`KR-README-REQ-055` records that every bundle must include validation evidence.
`KR-README-REQ-056` records that every bundle must include authority snapshot.
`KR-README-REQ-057` records that every law update requires pack review.
`KR-README-REQ-058` records that every regulator guidance change requires impact triage.
`KR-README-REQ-059` records that sector-specific Korean rules may narrow pack permissions.
`KR-README-REQ-060` records that ADR-0064 remains the pack architecture source.
`KR-README-REQ-061` records that ADR-0244 remains the tenancy source.
`KR-README-REQ-062` records that ADR-0251 remains the compliance pack source.
`KR-README-REQ-063` records that ADR-0263 remains the emission source.
`KR-README-REQ-064` records that README examples are normative for documentation, not executable tests.
`KR-README-REQ-065` records that executable tests must be created in the implementation lane.
`KR-README-REQ-066` records that docs may not grant production readiness by themselves.
`KR-README-REQ-067` records that KR pack release must verify every referenced Cedar policy exists.
`KR-README-REQ-068` records that KR pack release must verify every data class exists.
`KR-README-REQ-069` records that KR pack release must verify every audit event class exists.
`KR-README-REQ-070` records that KR pack release must verify every API delta has an owner.
`KR-README-REQ-071` records that every microservice activation must identify data classes.
`KR-README-REQ-072` records that every microservice activation must identify failure modes.
`KR-README-REQ-073` records that every microservice activation must identify audit events.
`KR-README-REQ-074` records that workforce services must treat employee data as personal information.
`KR-README-REQ-075` records that healthcare services must treat health data as sensitive information.
`KR-README-REQ-076` records that connect services must treat message metadata as regulated.
`KR-README-REQ-077` records that finance services must treat transaction data as regulated personal data when linked to a person.
`KR-README-REQ-078` records that industrial services must classify incident telemetry for controlled-source criteria.
`KR-README-REQ-079` records that hospitality services must minimize guest identity data.
`KR-README-REQ-080` records that dining and cellar services must enforce age-restricted goods rules.
`KR-README-REQ-081` records that denial messages must be localized when user-facing.
`KR-README-REQ-082` records that internal operator messages must include legal control IDs.
`KR-README-REQ-083` records that data residency decisions must name selected cell.
`KR-README-REQ-084` records that data residency denials must name missing certification.
`KR-README-REQ-085` records that consent APIs must return notice version.
`KR-README-REQ-086` records that consent APIs must return purpose code.
`KR-README-REQ-087` records that consent APIs must return revocability.
`KR-README-REQ-088` records that consent APIs must not infer bundled consent from account creation.
`KR-README-REQ-089` records that minor workflows must not expose raw guardian identifiers.
`KR-README-REQ-090` records that guardian consent must be independently withdrawable where legally allowed.
`KR-README-REQ-091` records that RRN validation must be transient by default.
`KR-README-REQ-092` records that RRN storage exceptions must be time-bound.
`KR-README-REQ-093` records that RRN access must require privileged purpose.
`KR-README-REQ-094` records that RRN access must generate separate audit event.
`KR-README-REQ-095` records that raw RRN must never appear in application logs.
`KR-README-REQ-096` records that raw RRN must never appear in support tickets.
`KR-README-REQ-097` records that CI/DI tokens must be treated as personal information.
`KR-README-REQ-098` records that identity providers must be recorded by code.
`KR-README-REQ-099` records that identity assurance events must be tamper-evident.
`KR-README-REQ-100` records that all KR pack fields require migration review before release.
`KR-README-REQ-101` records that cross-border data maps must identify destination country.
`KR-README-REQ-102` records that cross-border data maps must identify recipient.
`KR-README-REQ-103` records that cross-border data maps must identify purpose.
`KR-README-REQ-104` records that cross-border data maps must identify retention period.
`KR-README-REQ-105` records that cross-border data maps must identify safeguards.
`KR-README-REQ-106` records that cross-border data maps must identify data subject notice version.
`KR-README-REQ-107` records that transfer approvals expire when processor scope changes.
`KR-README-REQ-108` records that transfer approvals expire when destination changes.
`KR-README-REQ-109` records that transfer approvals expire when data class changes.
`KR-README-REQ-110` records that transfer denials must be sticky until new evidence is submitted.
`KR-README-REQ-111` records that CSAP evidence must be periodically refreshed.
`KR-README-REQ-112` records that CSAP certificate material must not be copied into routine event payloads.
`KR-README-REQ-113` records that KR public-sector tenant onboarding requires GRC review.
`KR-README-REQ-114` records that CSAP exception requests require legal and security approvals.
`KR-README-REQ-115` records that KISA notification state must be visible in incident dashboard.
`KR-README-REQ-116` records that PIPC notification state must be visible in privacy dashboard.
`KR-README-REQ-117` records that incident clocks must survive service restart.
`KR-README-REQ-118` records that incident clocks must preserve first-detection time.
`KR-README-REQ-119` records that incident clocks must preserve classification time.
`KR-README-REQ-120` records that incident clocks must preserve notification submission time.
`KR-README-REQ-121` records that privacy notices must identify controller or processor role.
`KR-README-REQ-122` records that privacy notices must identify collection items.
`KR-README-REQ-123` records that privacy notices must identify purposes.
`KR-README-REQ-124` records that privacy notices must identify retention periods.
`KR-README-REQ-125` records that privacy notices must identify third-party recipients.
`KR-README-REQ-126` records that privacy notices must identify overseas transfer details.
`KR-README-REQ-127` records that privacy notices must identify data-subject rights.
`KR-README-REQ-128` records that data-subject rights workflow must be localized.
`KR-README-REQ-129` records that data-subject requests must preserve request deadlines.
`KR-README-REQ-130` records that rejected data-subject requests must cite legal basis.
`KR-README-REQ-131` records that deleted data must leave only legal audit residue.
`KR-README-REQ-132` records that audit residue must remain PII-scrubbed.
`KR-README-REQ-133` records that lawful disclosure workflows must record request authority.
`KR-README-REQ-134` records that lawful disclosure workflows must record scope.
`KR-README-REQ-135` records that lawful disclosure workflows must record production timestamp.
`KR-README-REQ-136` records that lawful disclosure workflows must record withheld fields.
`KR-README-REQ-137` records that emergency disclosure workflows must record emergency basis.
`KR-README-REQ-138` records that emergency disclosure workflows must receive after-action review.
`KR-README-REQ-139` records that medical record exports require patient or legal basis.
`KR-README-REQ-140` records that medical record exports require destination tracking.
`KR-README-REQ-141` records that electronic medical records require integrity controls.
`KR-README-REQ-142` records that electronic medical records require change history.
`KR-README-REQ-143` records that electronic medical records require signature verification where applicable.
`KR-README-REQ-144` records that electronic documents require retention rule assignment.
`KR-README-REQ-145` records that electronic documents require integrity hash.
`KR-README-REQ-146` records that electronic documents require disposal workflow after retention.
`KR-README-REQ-147` records that communication content is never analytics input without lawful basis.
`KR-README-REQ-148` records that communication metadata needs minimization classification.
`KR-README-REQ-149` records that message search must honor communications-secret restrictions.
`KR-README-REQ-150` records that service diagnostics must not exfiltrate Korean records.
`KR-README-REQ-151` records that support impersonation must be logged with KR policy ID.
`KR-README-REQ-152` records that support access must have purpose and expiration.
`KR-README-REQ-153` records that production debugging must prefer synthetic or scrubbed records.
`KR-README-REQ-154` records that screenshots of Korean PII are regulated evidence.
`KR-README-REQ-155` records that exports to spreadsheets require export authorization.
`KR-README-REQ-156` records that exports to spreadsheets require retention and deletion tracking.
`KR-README-REQ-157` records that bulk export must classify transfer and breach risk.
`KR-README-REQ-158` records that rejected bulk export must produce audit evidence.
`KR-README-REQ-159` records that pack docs must be updated when implementation identifiers change.
`KR-README-REQ-160` records that pack docs must be updated when authority citations change.

## Checkpoint

This README is a documentation artifact.
It intentionally does not edit ADRs.
It intentionally does not edit microservices.
It intentionally does not edit other localization packs.
It is scoped to `/packs/kr-localization/`.
The required lifecycle claim was made before authoring.
Verification must confirm the six requested documents exist.
Verification must confirm each document has at least 600 lines.
Verification must confirm each document contains required headings.
Verification must run the required Oya VCS `verify` command without `--intent`.
Completion must run the required Oya VCS `done` command.
Promotion must run the required Oya VCS `promote` command.
