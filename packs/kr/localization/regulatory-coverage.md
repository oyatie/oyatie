---
doc_class: LocalizationPack
pack_id: KR-PACK-1
doc_id: KR-PACK-1-REGULATORY-COVERAGE
title: Korea Localization Pack Regulatory Coverage
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

# Korea Localization Pack Regulatory Coverage

This document is the canonical KR-PACK-1 regulatory coverage matrix.
It maps named Korean authorities to Oyatie pack controls.
It uses official Korean authority sources as the citation baseline.
It binds law coverage to Cedar policies, data classes, API deltas, audit events, and failure modes.
It is intentionally law-by-law rather than service-by-service.
Service owners must consume this matrix before enabling Korean regulated workflows.

## Coverage Principles

Regulatory coverage is additive over the canonical base.
Regulatory coverage never loosens canonical security controls.
Korean deny controls prevail over generic allow controls.
Sector-specific law coverage prevails over generic PIPA coverage when stricter.
Tenant-specific restrictions prevail over this pack when stricter.
Every law mapping requires a named authority citation.
Every law mapping requires a named obligation.
Every law mapping requires a named exemption or explicit no-exemption statement.
Every law mapping requires at least one data, API, policy, or audit consequence.
Every law mapping must remain traceable to ADR-0064, ADR-0244, ADR-0251, and ADR-0263.

## Authority Citations

Authority snapshot date: 2026-05-20.
Primary law source: National Law Information Center at `https://www.law.go.kr/`.
Primary privacy regulator source: Personal Information Protection Commission at `https://www.pipc.go.kr/`.
Primary incident and CSAP source: Korea Internet and Security Agency at `https://www.kisa.or.kr/`.
Primary CSAP portal: `https://isms.kisa.or.kr/main/csap/intro/index.jsp`.
Citation `KR-PIPA` names the Personal Information Protection Act, `개인정보 보호법`.
Citation `KR-PIPA-17799` preserves the pack feedback label for PIPA Act No. 17799 lineage.
Citation `KR-PIPA-A15` names PIPA Article 15 collection and use.
Citation `KR-PIPA-A17` names PIPA Article 17 third-party provision.
Citation `KR-PIPA-A22` names PIPA Article 22 consent method.
Citation `KR-PIPA-A22-2` names PIPA Article 22-2 children under 14.
Citation `KR-PIPA-A23` names PIPA Article 23 sensitive information.
Citation `KR-PIPA-A24` names PIPA Article 24 unique identifying information.
Citation `KR-PIPA-A24-2` names PIPA Article 24-2 resident registration number restrictions.
Citation `KR-PIPA-A28-2` names PIPA Article 28-2 pseudonymous information.
Citation `KR-PIPA-A28-8` names PIPA Article 28-8 overseas transfer.
Citation `KR-PIPA-A34` names PIPA Article 34 leakage notification and reporting.
Citation `KR-PIPA-ED-A40` names PIPA Enforcement Decree Article 40 72-hour report path.
Citation `KR-CSAP` names Cloud Security Assurance Program.
Citation `KR-CLOUD-A23-2` names Cloud Computing Development and User Protection Act Article 23-2.
Citation `KR-YOUTH` names Youth Protection Act, `청소년 보호법`.
Citation `KR-YOUTH-A16` names Youth Protection Act Article 16 age and identity verification for youth-harmful media.
Citation `KR-YOUTH-ED-A17` names Youth Protection Act Enforcement Decree Article 17 verification methods.
Citation `KR-COMM-SECRETS` names Communications Secrets Protection Act, `통신비밀보호법`.
Citation `KR-EDOC` names Digital Documents and Transactions Act, `전자문서 및 전자거래 기본법`.
Citation `KR-INFONET` names Information and Communications Network Act, `정보통신망 이용촉진 및 정보보호 등에 관한 법률`.
Citation `KR-INFONET-A23-2` names non-RRN identity verification lineage under information network law.
Citation `KR-INFONET-A23-3` names identity confirmation agency lineage.
Citation `KR-INFONET-A48-3` names incident reporting and post-incident cooperation lineage.
Citation `KR-MEDICAL` names Medical Service Act, `의료법`.
Citation `KR-MEDICAL-A22` names Medical Service Act Article 22 medical records.
Citation `KR-MEDICAL-A23` names Medical Service Act Article 23 electronic medical records.
Citation `KR-MEDICAL-A34` names Medical Service Act Article 34 remote medical support.
Citation `KR-CYBER` maps pack terminology "KR Cyber Security Act" to the Information and Communications Infrastructure Protection Act, `정보통신기반 보호법`, unless a newer enacted cyber statute supersedes it.
Citation `KR-CYBER-A8` names designation of major information and communications infrastructure.
Citation `KR-CYBER-A13` names incident notification for major information and communications infrastructure.
Citation `KR-CYBER-ED-A21` names incident notification content.
Effective dates must be read from law.go.kr for the final pack bundle.
This document records source-check date rather than freezing future law amendments.
If law.go.kr lists a future effective date, release engineering must choose the correct effective text for deployment date.
If English translations lag Korean text, Korean official text controls.
If regulator guidance conflicts with this matrix, legal review must update the pack before shipment.

## Coverage Matrix Summary

| Authority | Coverage class | Primary object | Pack consequence |
| --- | --- | --- | --- |
| KR-PIPA | privacy baseline | personal information | consent, minimization, rights, breach reporting |
| KR-PIPA-17799 | feedback lineage | personal information | named pack trace label |
| KR-CSAP | cloud assurance | public-sector cloud workloads | certified KR cell pinning |
| KR-YOUTH | youth protection | youth-harmful media and age-restricted goods | age and identity gate |
| KR-COMM-SECRETS | communications secrecy | content and communications metadata | inspection and retention restriction |
| KR-EDOC | electronic document evidence | electronic documents and certified retention | preservation metadata |
| KR-INFONET | network service security | information network service workflows | identity, safeguards, incident cooperation |
| KR-MEDICAL | healthcare records | medical and electronic medical records | healthcare locality and access purpose |
| KR-CYBER | critical infrastructure security | controlled-source and critical incident evidence | KISA incident notification |

## Personal Information Protection Act Coverage

Authority name: Personal Information Protection Act.
Korean name: `개인정보 보호법`.
Pack citation code: `KR-PIPA`.
Feedback lineage code: `KR-PIPA-17799`.
Primary regulator: Personal Information Protection Commission.
Primary incident professional institution path: KISA where appointed by decree.
Named effective date rule: use law.go.kr effective text for deployment date.
Cited article: Article 15 collection and use.
Cited article: Article 17 provision to third party.
Cited article: Article 22 consent method.
Cited article: Article 22-2 children under 14.
Cited article: Article 23 sensitive information.
Cited article: Article 24 unique identifying information.
Cited article: Article 24-2 resident registration number.
Cited article: Article 28-2 pseudonymous information.
Cited article: Article 28-8 overseas transfer.
Cited article: Article 34 leakage notification and reporting.
Cited decree article: Enforcement Decree Article 40 report timing and KISA/PIPC destination.
Named obligation: purpose-specific lawful basis before processing.
Named obligation: separate consent for separate processing purposes.
Named obligation: explicit handling of sensitive information.
Named obligation: explicit handling of unique identifying information.
Named obligation: RRN default denial without statutory basis.
Named obligation: child-under-14 guardian consent.
Named obligation: cross-border transfer basis and notice.
Named obligation: leakage notification to data subjects without delay.
Named obligation: report to PIPC or KISA within decree-defined timeline for reportable leakage.
Named obligation: use pseudonymization only within permitted purposes.
Named obligation: implement safety measures proportionate to data class.
Named exemption: processing without consent where statute permits or requires.
Named exemption: processing without consent where necessary for contract performance under applicable PIPA basis.
Named exemption: pseudonymous information for statistics, scientific research, or public-interest archiving under Article 28-2.
Named exemption: report exception where decree conditions show materially low rights-infringement risk after recovery or deletion.
Named exemption: legal hold blocks erasure while authority-preserved evidence remains required.
Named no-exemption: convenience analytics does not exempt consent or pseudonymization.
Named no-exemption: account signup does not exempt purpose enumeration.
Named no-exemption: payroll convenience does not exempt RRN statutory-basis evidence.
Data class: `PI_KR_PIPA`.
Data class: `PI_KR_SENSITIVE`.
Data class: `PI_KR_UNDER14`.
Data class: `PI_KR_MINOR_14_18`.
Data class: `PI_KR_RRN`.
Data class: `PI_KR_BIOMETRIC`.
Cedar policy: `pack-kr-pack-1-pipa-purpose-consent`.
Cedar policy: `pack-kr-pack-1-pipa-separate-consent`.
Cedar policy: `pack-kr-pack-1-pipa-sensitive-consent`.
Cedar policy: `pack-kr-pack-1-pipa-under14-guardian`.
Cedar policy: `pack-kr-pack-1-rrn-collection-deny-default`.
Cedar policy: `pack-kr-pack-1-cross-border-transfer-deny-default`.
Cedar policy: `pack-kr-pack-1-pipa-breach-reporting-window`.
API impact: `POST /kr/consents`.
API impact: `POST /kr/guardian-consents`.
API impact: `POST /kr/identity/rrn/hash`.
API impact: `POST /kr/cross-border-transfer-assessments`.
API impact: `POST /kr/incidents/classify`.
Audit event: `KrPipaConsentCaptured`.
Audit event: `KrPipaConsentWithdrawn`.
Audit event: `KrGuardianConsentCaptured`.
Audit event: `KrRrnCollectionDenied`.
Audit event: `KrCrossBorderTransferDenied`.
Audit event: `KrBreachPipcNotified`.
Failure mode: bundled consent accepted as sufficient.
Failure mode: RRN stored as ordinary identifier.
Failure mode: overseas transfer approved without basis artifact.
Failure mode: breach clock not started after leakage classification.

## Cloud Security Assurance Program Coverage

Authority name: Cloud Security Assurance Program.
Korean source: KISA ISMS/CSAP portal.
Pack citation code: `KR-CSAP`.
Statutory anchor: Cloud Computing Development and User Protection Act Article 23-2.
Named effective date rule: use current KISA certification framework on bundle build date.
Covered workloads: Korean public-sector cloud workloads.
Covered workloads: tenant workloads contractually requiring CSAP-equivalent cell assurance.
Covered workloads: regulated evidence repositories marked CSAP-sensitive.
Covered cells: KISA-MID when mid-sensitivity certification evidence is present.
Covered cells: KISA-BIO when biometric or high-sensitivity evidence is present.
Covered cells: ICN-MID when interconnect metadata controls are present.
Named obligation: route CSAP-sensitive workloads to a certified cell.
Named obligation: record certification scope.
Named obligation: record certification status.
Named obligation: record certification expiry or refresh date.
Named obligation: preserve evidence digest without copying full certificate into routine audit.
Named obligation: block public-sector workload if cell is uncertified.
Named obligation: block unsupported subprocessor path for CSAP-sensitive workload.
Named obligation: surface certification denial to tenant GRC dashboard.
Named exemption: non-public-sector private workload may use non-CSAP KR cell if no contract or data class requires CSAP.
Named exemption: synthetic test data may use non-CSAP development cell when production Korean data is absent.
Named exemption: emergency containment copy may exist transiently if incident response authority approves and audit event records scope.
Named no-exemption: public-sector production data cannot be routed by cost-only balancing.
Named no-exemption: expired certification cannot satisfy CSAP cell pinning.
Named no-exemption: cloud-provider marketing page cannot replace pack evidence digest.
Data class: `SEC_KR_CSAP_EVIDENCE`.
Data class: `SEC_KR_CELL_CERTIFICATION`.
Data class: `PI_KR_PIPA`.
Data model delta: `tenant.kr_csap_level`.
Data model delta: `tenant.kr_primary_cell_id`.
Data model delta: `tenant.kr_public_sector_flag`.
Cedar policy: `pack-kr-pack-1-csap-cell-pinning`.
Cedar policy: `pack-kr-pack-1-kisa-mid-cell`.
Cedar policy: `pack-kr-pack-1-kisa-bio-cell`.
Cedar policy: `pack-kr-pack-1-icn-mid-cell`.
Cedar policy: `pack-kr-pack-1-processor-due-diligence`.
API impact: `POST /kr/data-residency/evaluate`.
API impact: `GET /kr/csap/evidence/{tenant_id}`.
API impact: `GET /tenants/{tenant_id}/kr/compliance-state`.
Audit event: `KrCsapEvidencePulled`.
Audit event: `KrPackCellPinned`.
Audit event: `KrProcessorDueDiligenceApproved`.
Failure mode: public-sector tenant uses uncertified cell.
Failure mode: CSAP certificate status is stale.
Failure mode: certification scope excludes active service.
Failure mode: DR cell lacks matching CSAP assurance.

## Youth Protection Act Coverage

Authority name: Youth Protection Act.
Korean name: `청소년 보호법`.
Pack citation code: `KR-YOUTH`.
Primary cited article: Article 16.
Primary decree citation: Enforcement Decree Article 17.
Named effective date rule: use law.go.kr text in force for the content availability date.
Covered workflow: youth-harmful media sale.
Covered workflow: youth-harmful media rental.
Covered workflow: youth-harmful media distribution.
Covered workflow: youth-harmful media viewing.
Covered workflow: youth-harmful media use.
Covered workflow: age-limited goods in dining or cellar contexts.
Named obligation: verify counterparty age.
Named obligation: verify counterparty identity when Article 16 applies.
Named obligation: deny youth access to youth-harmful media.
Named obligation: use permitted verification methods.
Named obligation: record age-band result instead of raw birthdate where possible.
Named obligation: localize denial text into Korean for user-facing flows.
Named obligation: prevent RRN collection as the default age-gate shortcut.
Named obligation: record verification event with policy ID.
Named exemption: unrestricted content does not require youth-harmful age gate.
Named exemption: adult-only verified session may reuse current verification token during its validity window.
Named exemption: legal guardian workflow may handle minors where the service is not youth-prohibited.
Named no-exemption: self-declared age is not enough for youth-harmful media.
Named no-exemption: foreign interface language does not avoid Korean youth restriction for Korean offering.
Named no-exemption: community moderation tag cannot override statutory youth-restricted classification.
Data class: `PI_KR_MINOR_14_18`.
Data class: `PI_KR_UNDER14`.
Data class: `KR_YOUTH_RESTRICTED_CONTENT`.
Data model delta: `subject.kr_age_band`.
Data model delta: `subject.kr_youth_protection_restricted_flag`.
Cedar policy: `pack-kr-pack-1-youth-age-gate`.
Cedar policy: `pack-kr-pack-1-youth-content-deny`.
Cedar policy: `pack-kr-pack-1-ci-di-preferred`.
API impact: `POST /kr/youth-age-checks`.
API impact: `POST /kr/guardian-consents`.
Audit event: `KrYouthAgeGateEvaluated`.
Audit event: `KrYouthRestrictedAccessDenied`.
Failure mode: adult content served before verification.
Failure mode: RRN collected only to simplify age gate.
Failure mode: minor age-band stored with raw birthdate in audit payload.
Failure mode: denial lacks policy reference.

## Communications Secrets Protection Act Coverage

Authority name: Communications Secrets Protection Act.
Korean name: `통신비밀보호법`.
Pack citation code: `KR-COMM-SECRETS`.
Named effective date rule: use law.go.kr text in force for the communication event date.
Covered object: communication content.
Covered object: communication confirmation data.
Covered object: message metadata linked to Korean users.
Covered object: support access to messages.
Covered object: search indexing of communication content.
Covered object: analytics extraction from communication content.
Named obligation: deny interception or content inspection without legal basis.
Named obligation: minimize retention of communications metadata.
Named obligation: record lawful disclosure basis.
Named obligation: segregate support access from ordinary product access.
Named obligation: preserve communications evidence only under legal hold or lawful process.
Named obligation: scrub communication content from routine audit payloads.
Named obligation: restrict message indexing where content inspection would occur.
Named obligation: classify communications metadata as regulated personal information when linked to user.
Named exemption: user-directed message retrieval by the sender or recipient may proceed within product contract.
Named exemption: lawful process may require preservation or disclosure.
Named exemption: automated malware scanning may proceed only within documented security basis and minimization boundary.
Named no-exemption: operator curiosity is never lawful basis.
Named no-exemption: broad product analytics cannot inspect content.
Named no-exemption: debug export cannot include message content by default.
Data class: `PI_KR_COMMUNICATION_METADATA`.
Data class: `KR_COMMUNICATION_CONTENT`.
Data class: `SEC_KR_LEGAL_PROCESS`.
Data model delta: `communication.kr_minimization_scope`.
Data model delta: `communication.kr_lawful_access_basis`.
Cedar policy: `pack-kr-pack-1-communications-secret-deny-content-inspection`.
Cedar policy: `pack-kr-pack-1-communications-metadata-retention`.
Cedar policy: `pack-kr-pack-1-lawful-disclosure-log`.
API impact: `POST /kr/communications/access-purpose`.
API impact: `POST /kr/legal-disclosures`.
Audit event: `KrCommunicationsMetadataAccessed`.
Audit event: `KrLawfulDisclosureRecorded`.
Failure mode: search index stores message content without basis.
Failure mode: support ticket contains raw message content.
Failure mode: legal process disclosure lacks authority reference.
Failure mode: metadata retained after minimization window.

## Digital Documents and Transactions Act Coverage

Authority name: Digital Documents and Transactions Act.
Korean name: `전자문서 및 전자거래 기본법`.
Pack citation code: `KR-EDOC`.
Named effective date rule: use law.go.kr text in force for document creation or retention event.
Covered object: electronic document.
Covered object: electronic transaction evidence.
Covered object: certified electronic document center handoff.
Covered object: tax invoice evidence where accounting service is active.
Covered object: procurement contract evidence.
Covered object: settlement evidence.
Named obligation: preserve document identity.
Named obligation: preserve creation timestamp.
Named obligation: preserve integrity hash.
Named obligation: preserve retention rule.
Named obligation: preserve disposal rule.
Named obligation: preserve legal-hold state.
Named obligation: record certified repository handoff where used.
Named obligation: make evidence metadata available to audit workflow.
Named exemption: non-regulated transient draft may follow ordinary retention if no legal record status attaches.
Named exemption: duplicate convenience copy may be deleted when canonical preserved record remains intact.
Named exemption: synthetic electronic document in test environment may use test retention markers.
Named no-exemption: exported PDF copy does not replace canonical integrity metadata.
Named no-exemption: deletion request does not override legal hold.
Named no-exemption: document migration cannot drop chain-of-custody metadata.
Data class: `PI_KR_ELECTRONIC_DOCUMENT`.
Data class: `KR_ELECTRONIC_TRANSACTION_EVIDENCE`.
Data model delta: `document.kr_integrity_hash`.
Data model delta: `document.kr_preservation_rule_id`.
Data model delta: `document.kr_legal_hold_state`.
Cedar policy: `pack-kr-pack-1-electronic-document-evidence`.
Cedar policy: `pack-kr-pack-1-retention-legal-hold`.
API impact: `POST /kr/electronic-document/preserve`.
API impact: `POST /kr/electronic-document/dispose`.
Audit event: `KrElectronicDocumentPreserved`.
Audit event: `KrLegalHoldApplied`.
Audit event: `KrLegalHoldReleased`.
Failure mode: document hash missing.
Failure mode: evidence retention rule missing.
Failure mode: migration drops evidence chain.
Failure mode: disposal runs during legal hold.

## Information and Communications Network Act Coverage

Authority name: Information and Communications Network Act.
Korean name: `정보통신망 이용촉진 및 정보보호 등에 관한 법률`.
Pack citation code: `KR-INFONET`.
Named effective date rule: use law.go.kr text in force for network service processing date.
Cited lineage: Article 23-2 non-RRN identity verification alternative.
Cited lineage: Article 23-3 identity confirmation agency.
Cited lineage: Article 48-3 incident reporting and cooperation.
Covered object: information network service user data.
Covered object: CI and DI alternative identifiers.
Covered object: network service incident evidence.
Covered object: spam and unlawful transmission reporting where relevant.
Covered object: information security measures for network services.
Named obligation: prefer non-RRN identity verification where legally sufficient.
Named obligation: record identity confirmation agency where used.
Named obligation: record CI/DI token class.
Named obligation: record information security measure evidence.
Named obligation: cooperate with incident reporting path where required.
Named obligation: keep unlawful spam and abuse workflows separate from privacy rights workflows.
Named obligation: classify network incident as privacy breach when personal information leakage also occurs.
Named obligation: block CI/DI leakage into display names or logs.
Named exemption: internal synthetic identity token may be used for testing without identity confirmation agency.
Named exemption: CI/DI may be absent where service does not require real-name identity.
Named exemption: another legally sufficient identity basis may replace CI/DI when recorded.
Named no-exemption: CI/DI tokens are still personal information.
Named no-exemption: RRN may not be collected merely because identity assurance is inconvenient.
Named no-exemption: security incident reporting does not eliminate PIPA breach reporting where both apply.
Data class: `PI_KR_CI`.
Data class: `PI_KR_DI`.
Data class: `SEC_KR_INCIDENT_ARTIFACT`.
Data model delta: `identity.kr_ci_token`.
Data model delta: `identity.kr_di_token`.
Data model delta: `identity.kr_identity_provider_code`.
Cedar policy: `pack-kr-pack-1-ci-di-preferred`.
Cedar policy: `pack-kr-pack-1-info-network-security-measures`.
Cedar policy: `pack-kr-pack-1-incident-kisa-triage`.
API impact: `POST /kr/identity/ci-di/link`.
API impact: `POST /kr/incidents/classify`.
Audit event: `KrCiDiLinked`.
Audit event: `KrControlledSourceIncidentClassified`.
Failure mode: CI token appears in UI.
Failure mode: DI token shared across tenant boundary.
Failure mode: identity provider code omitted.
Failure mode: network incident not cross-classified as PIPA breach.

## Medical Service Act Coverage

Authority name: Medical Service Act.
Korean name: `의료법`.
Pack citation code: `KR-MEDICAL`.
Named effective date rule: use law.go.kr text in force for care or record event.
Cited article: Article 22 medical records.
Cited article: Article 23 electronic medical records.
Cited article: Article 34 remote medical support.
Covered object: diagnosis records.
Covered object: treatment records.
Covered object: prescription records.
Covered object: clinical notes.
Covered object: electronic medical records.
Covered object: remote medical support logs.
Covered object: healthcare portal patient access.
Named obligation: create required medical record.
Named obligation: preserve medical record for legally required period.
Named obligation: protect electronic medical record integrity.
Named obligation: capture practitioner role for medical access.
Named obligation: capture treatment or administrative purpose.
Named obligation: preserve change history.
Named obligation: prevent patient record access outside scope.
Named obligation: pin Korean medical records to KR-approved healthcare cell.
Named exemption: emergency care may use emergency lawful basis but must audit after the fact.
Named exemption: patient-authorized disclosure may proceed within signed scope.
Named exemption: legally compelled disclosure may proceed with authority record.
Named no-exemption: analytics access does not equal treatment access.
Named no-exemption: support role does not equal practitioner role.
Named no-exemption: copied record export does not drop Medical Service Act retention.
Data class: `PI_KR_MEDICAL_RECORD`.
Data class: `PI_KR_SENSITIVE`.
Data class: `KR_REMOTE_MEDICAL_SUPPORT_LOG`.
Data model delta: `medical.kr_record_retention_rule`.
Data model delta: `medical.kr_access_purpose`.
Data model delta: `medical.kr_practitioner_role`.
Cedar policy: `pack-kr-pack-1-medical-record-locality`.
Cedar policy: `pack-kr-pack-1-medical-record-access-trace`.
Cedar policy: `pack-kr-pack-1-pipa-sensitive-consent`.
API impact: `POST /kr/medical-record-access`.
API impact: `POST /kr/medical-record-export`.
Audit event: `KrMedicalRecordAccessed`.
Audit event: `KrMedicalRecordExported`.
Failure mode: clinician access lacks purpose.
Failure mode: record stored in non-healthcare KR cell.
Failure mode: export lacks patient or legal basis.
Failure mode: electronic record change lacks integrity trace.

## KR Cyber Security Act Coverage

Authority name in product language: KR Cyber Security Act.
Official pack mapping: Information and Communications Infrastructure Protection Act.
Korean name: `정보통신기반 보호법`.
Pack citation code: `KR-CYBER`.
Named effective date rule: use law.go.kr text in force for incident date.
Cited article: Article 8 designation of major information and communications infrastructure.
Cited article: Article 13 notification of incidents.
Cited decree article: Enforcement Decree Article 21 incident notification content.
Covered object: controlled-source incident evidence.
Covered object: major information and communications infrastructure event.
Covered object: service disruption caused by unauthorized access.
Covered object: data manipulation or destruction.
Covered object: malware or logic-bomb interference.
Covered object: denial-of-service or false-command disruption.
Named obligation: classify incident against controlled-source criteria.
Named obligation: identify affected facility or service.
Named obligation: identify damage details.
Named obligation: notify relevant administrative agency, investigative agency, or KISA where Article 13 applies.
Named obligation: preserve incident report reference.
Named obligation: prevent evidence export to non-approved debug workspace.
Named obligation: connect incident response with PIPA breach clock when personal information is involved.
Named obligation: record containment start.
Named exemption: ordinary non-security service bug may remain outside KR-CYBER if no incident criterion is met.
Named exemption: synthetic tabletop incident may use test notification markers.
Named exemption: non-critical private tenant incident may follow Information Network Act/PIPA path unless infrastructure criteria apply.
Named no-exemption: ransomware affecting KR regulated systems is not ordinary availability incident.
Named no-exemption: internal operator misuse can be controlled-source breach if criteria are met.
Named no-exemption: infrastructure incident reporting does not eliminate privacy breach reporting.
Data class: `SEC_KR_INCIDENT_ARTIFACT`.
Data class: `SEC_KR_CONTROLLED_SOURCE_BREACH`.
Data class: `SEC_KR_CRITICAL_INFRASTRUCTURE_EVENT`.
Data model delta: `incident.kr_controlled_source_class`.
Data model delta: `incident.kr_kisa_notification_ref`.
Data model delta: `incident.kr_pipc_notification_ref`.
Cedar policy: `pack-kr-pack-1-cybersecurity-incident-triage`.
Cedar policy: `pack-kr-pack-1-incident-kisa-triage`.
Cedar policy: `pack-kr-pack-1-pipa-breach-reporting-window`.
API impact: `POST /kr/incidents/classify`.
API impact: `POST /kr/incidents/{incident_id}/kisa-notification`.
Audit event: `KrControlledSourceIncidentClassified`.
Audit event: `KrIncidentContainmentStarted`.
Audit event: `KrBreachKisaNotified`.
Failure mode: incident criteria not evaluated.
Failure mode: KISA notification lacks affected facility.
Failure mode: incident report omits damage details.
Failure mode: infrastructure event misses privacy cross-classification.

## Activated Cedar Policies

`pack-kr-pack-1-activate` covers installed pack state.
`pack-kr-pack-1-deny-uninstalled-use` covers use before installation.
`pack-kr-pack-1-tenant-subscope-required` covers ADR-0244 context.
`pack-kr-pack-1-pack-precedence-deny-wins` covers Korean deny precedence.
`pack-kr-pack-1-cell-kr-residency` covers general KR cell routing.
`pack-kr-pack-1-csap-cell-pinning` covers CSAP workloads.
`pack-kr-pack-1-kisa-mid-cell` covers KISA-MID placement.
`pack-kr-pack-1-kisa-bio-cell` covers KISA-BIO placement.
`pack-kr-pack-1-icn-mid-cell` covers ICN-MID placement.
`pack-kr-pack-1-pipa-purpose-consent` covers PIPA Article 15 and 22.
`pack-kr-pack-1-pipa-separate-consent` covers separated consent.
`pack-kr-pack-1-pipa-sensitive-consent` covers PIPA Article 23.
`pack-kr-pack-1-pipa-under14-guardian` covers PIPA Article 22-2.
`pack-kr-pack-1-youth-age-gate` covers Youth Protection Act Article 16.
`pack-kr-pack-1-youth-content-deny` covers youth-harmful content denial.
`pack-kr-pack-1-rrn-collection-deny-default` covers PIPA Article 24-2.
`pack-kr-pack-1-rrn-statutory-basis` covers statutory-basis exception.
`pack-kr-pack-1-rrn-hash-only` covers RRN irreversible derivative requirement.
`pack-kr-pack-1-ci-di-preferred` covers identity verification alternatives.
`pack-kr-pack-1-cross-border-transfer-deny-default` covers PIPA Article 28-8.
`pack-kr-pack-1-cross-border-transfer-consent` covers transfer consent.
`pack-kr-pack-1-cross-border-transfer-adequacy` covers recognized adequate basis.
`pack-kr-pack-1-cross-border-transfer-scc` covers contractual transfer safeguards.
`pack-kr-pack-1-medical-record-locality` covers Medical Service Act locality.
`pack-kr-pack-1-medical-record-access-trace` covers medical access reason.
`pack-kr-pack-1-communications-secret-deny-content-inspection` covers communications secrecy.
`pack-kr-pack-1-communications-metadata-retention` covers metadata minimization.
`pack-kr-pack-1-electronic-document-evidence` covers electronic document preservation.
`pack-kr-pack-1-info-network-security-measures` covers network service safeguards.
`pack-kr-pack-1-cybersecurity-incident-triage` covers KR-CYBER incident classification.
`pack-kr-pack-1-incident-kisa-triage` covers KISA notification routing.
`pack-kr-pack-1-pipa-breach-reporting-window` covers PIPA breach report clocks.
`pack-kr-pack-1-pii-emission-scrub` covers ADR-0263 scrubbed emissions.
`pack-kr-pack-1-audit-tenant-context` covers tenant context.
`pack-kr-pack-1-audit-jurisdiction-code` covers KR jurisdiction stamping.
`pack-kr-pack-1-deidentified-analytics-threshold` covers pseudonymization and aggregation controls.
`pack-kr-pack-1-processor-due-diligence` covers processor/subprocessor evidence.
`pack-kr-pack-1-lawful-disclosure-log` covers legal disclosure.
`pack-kr-pack-1-retention-legal-hold` covers legal holds.
`pack-kr-pack-1-consent-withdrawal-honor` covers consent withdrawal.
`pack-kr-pack-1-localized-notice-required` covers Korean-language notices.

## Data Model Deltas

Add `regulatory_coverage.kr_authority_code`.
Add `regulatory_coverage.kr_authority_name_ko`.
Add `regulatory_coverage.kr_authority_name_en`.
Add `regulatory_coverage.kr_article_reference`.
Add `regulatory_coverage.kr_effective_date`.
Add `regulatory_coverage.kr_obligation_code`.
Add `regulatory_coverage.kr_exemption_code`.
Add `regulatory_coverage.kr_policy_id`.
Add `regulatory_coverage.kr_data_class`.
Add `regulatory_coverage.kr_api_contract_id`.
Add `regulatory_coverage.kr_audit_event_class`.
Add `regulatory_coverage.kr_failure_mode_id`.
Add `regulatory_coverage.kr_source_snapshot_date`.
Add `tenant.kr_regulatory_profile`.
Add `tenant.kr_sector_flags`.
Add `subject.kr_privacy_status`.
Add `processor.kr_transfer_status`.
Add `incident.kr_regulatory_classification`.
Add `document.kr_regulatory_retention_rule`.
Transform authority references into normalized citation codes.
Transform article text into stable citation references, not pasted statute text.
Transform obligation rows into policy-linked control rows.
Transform exemptions into explicit policy branches.
Transform sector law mappings into data class gates.
Transform breach classifications into incident clocks.
Transform cross-border transfer classifications into assessment records.
Transform medical classifications into access-purpose requirements.
Transform communications classifications into content-inspection denials.
Transform electronic-document classifications into preservation metadata.

## API Contract Deltas

`GET /kr/regulatory-coverage` returns law-by-law coverage rows.
`GET /kr/regulatory-coverage/{authority_code}` returns one authority mapping.
`GET /kr/regulatory-coverage/{authority_code}/obligations` returns named obligations.
`GET /kr/regulatory-coverage/{authority_code}/exemptions` returns named exemptions.
`GET /kr/regulatory-coverage/{authority_code}/policies` returns activated Cedar policies.
`GET /kr/regulatory-coverage/{authority_code}/data-classes` returns data class impact.
`GET /kr/regulatory-coverage/{authority_code}/api-deltas` returns API impact.
`GET /kr/regulatory-coverage/{authority_code}/audit-events` returns audit event additions.
`GET /kr/regulatory-coverage/{authority_code}/failure-modes` returns failure mode references.
`POST /kr/regulatory-coverage/evaluate` evaluates a proposed service workflow.
`POST /kr/regulatory-coverage/cross-classify` maps incidents across PIPA, Information Network Act, and KR-CYBER.
`POST /kr/regulatory-coverage/source-snapshot` records authority source snapshot metadata.
Every response includes `pack_id=KR-PACK-1`.
Every response includes `authority_snapshot_date`.
Every response includes `cedar_policy_ids` when policy consequence exists.
Every denial includes `failure_mode_id`.
Every state-changing operation emits an ADR-0263 audit event.

## Audit Event Additions

`KrRegulatoryCoverageEvaluated` records authority codes and policy IDs used in evaluation.
`KrAuthoritySnapshotRecorded` records law.go.kr, PIPC, KISA, and CSAP source snapshot metadata.
`KrAuthorityCoverageUpdated` records a controlled update to coverage matrix.
`KrAuthorityExemptionApplied` records named exemption and approving role.
`KrAuthorityExemptionDenied` records exemption request denial.
`KrRegulatoryCrossClassificationCompleted` records incident or workflow cross-classification.
`KrPipaCoverageTriggered` records PIPA coverage activation.
`KrCsapCoverageTriggered` records CSAP coverage activation.
`KrYouthCoverageTriggered` records youth protection activation.
`KrCommunicationsSecretsCoverageTriggered` records communications secrecy activation.
`KrElectronicDocumentCoverageTriggered` records electronic document coverage activation.
`KrInformationNetworkCoverageTriggered` records network service coverage activation.
`KrMedicalCoverageTriggered` records healthcare coverage activation.
`KrCyberCoverageTriggered` records critical infrastructure coverage activation.
All events carry `tenant_id`.
All events carry `sub_scope_path` where scoped.
All events carry `jurisdiction_code=KR`.
All events carry `audit_id`.
All events carry `source_microservice`.
All events carry PII-scrubbed payloads.

## Failure Modes specific to KR enforcement

Failure mode `KR-REG-FM-001`: law coverage missing for active Korean service.
Failure mode `KR-REG-FM-002`: authority citation missing article reference.
Failure mode `KR-REG-FM-003`: effective date is stale or absent.
Failure mode `KR-REG-FM-004`: named obligation has no policy consequence.
Failure mode `KR-REG-FM-005`: named exemption applied without evidence.
Failure mode `KR-REG-FM-006`: PIPA breach not cross-classified with cybersecurity incident.
Failure mode `KR-REG-FM-007`: healthcare record treated as generic PIPA only.
Failure mode `KR-REG-FM-008`: communications content treated as ordinary analytics input.
Failure mode `KR-REG-FM-009`: youth restriction omitted from community workflow.
Failure mode `KR-REG-FM-010`: CSAP-sensitive workload routed by generic residency only.
Failure mode `KR-REG-FM-011`: electronic document retention omitted from accounting workflow.
Failure mode `KR-REG-FM-012`: CI/DI handled as non-personal identifier.
Failure mode `KR-REG-FM-013`: RRN handled under unique-identifier rule but not Article 24-2 rule.
Failure mode `KR-REG-FM-014`: KR-CYBER mapping ignored because product name differs from law title.
Failure mode `KR-REG-FM-015`: audit event contains pasted statutory text with PII context.
Failure mode `KR-REG-FM-016`: authority snapshot not attached to pack bundle.
Failure mode `KR-REG-FM-017`: legal update changes citation but pack digest is unchanged.
Failure mode `KR-REG-FM-018`: tenant contract stricter than pack but not represented in precedence.
Failure mode `KR-REG-FM-019`: exemption branch bypasses deny policy.
Failure mode `KR-REG-FM-020`: source translation drift overrides Korean official text.

## Worked Examples

### Scenario 1: Payroll RRN Coverage

The payroll service proposes RRN collection.
The coverage evaluator maps the workflow to KR-PIPA Article 24-2.
The evaluator also maps it to the RRN-specific documentation.
The evaluator names obligation `statutory-basis-before-RRN`.
The evaluator names exemption `statute-requires-processing`.
The evaluator activates `pack-kr-pack-1-rrn-statutory-basis`.
The evaluator activates `pack-kr-pack-1-rrn-hash-only`.
The API response lists `PI_KR_RRN`.
The audit event is `KrRegulatoryCoverageEvaluated`.
The service may proceed only after basis evidence is attached.

### Scenario 2: Community Youth Coverage

The community service labels a board youth-harmful.
The coverage evaluator maps the workflow to Youth Protection Act Article 16.
The evaluator names obligation `age-and-identity-verification`.
The evaluator names no exemption for self-declared age.
The evaluator activates `pack-kr-pack-1-youth-age-gate`.
The evaluator activates `pack-kr-pack-1-youth-content-deny`.
The API response lists `KR_YOUTH_RESTRICTED_CONTENT`.
The denial references `KR-REG-FM-009` if age-gate is missing.

### Scenario 3: Medical Portal Coverage

The patient portal exposes Korean medical records.
The coverage evaluator maps the workflow to Medical Service Act Articles 22 and 23.
The evaluator also maps health data to PIPA Article 23.
The evaluator names obligation `medical-access-purpose`.
The evaluator names obligation `electronic-record-integrity`.
The evaluator activates medical locality and access-trace policies.
The response requires `KrMedicalRecordAccessed`.
The portal cannot rely only on generic PIPA consent.

### Scenario 4: Incident Cross-Classification

The security service reports unauthorized access to a Korean production cell.
The coverage evaluator maps the event to KR-CYBER Article 13 when infrastructure criteria apply.
The evaluator maps the event to PIPA Article 34 when personal data is leaked.
The evaluator maps the event to Information Network Act incident cooperation when network service criteria apply.
The evaluator starts multiple clocks.
The evaluator emits `KrRegulatoryCrossClassificationCompleted`.
KISA notification and PIPC reporting remain separate tracked actions.

### Scenario 5: Electronic Procurement Evidence

The procurement service stores a signed electronic contract.
The coverage evaluator maps the record to Digital Documents and Transactions Act coverage.
The evaluator names obligation `integrity-hash-preservation`.
The evaluator names obligation `retention-rule-preservation`.
The evaluator activates `pack-kr-pack-1-electronic-document-evidence`.
The evaluator emits `KrElectronicDocumentCoverageTriggered`.
Deletion is denied while legal hold or retention remains active.

## Cross-References

Pack overview: `packs/kr-localization/README.md`.
Data residency detail: `packs/kr-localization/data-residency.md`.
Consent detail: `packs/kr-localization/consent-management.md`.
RRN detail: `packs/kr-localization/resident-id-number-rrn-handling.md`.
Incident detail: `packs/kr-localization/cybersecurity-and-incident-response.md`.
Canonical base ADR: `docs/decisions/ADR-0709-general-live-apex.md`.
Tenant scoping ADR: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
Compliance pack ADR: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
Emission contract ADR: `docs/decisions/ADR-0706-observability-live-apex.md`.
KR planning source: `docs/localization-packs/kr.md`.
KR manifest source: `docs/localization-packs/kr/pack.yaml`.
Official law source: `https://www.law.go.kr/`.
Official PIPC source: `https://www.pipc.go.kr/`.
Official KISA source: `https://www.kisa.or.kr/`.
Official CSAP source: `https://isms.kisa.or.kr/main/csap/intro/index.jsp`.

## Obligation Register

`KR-REG-OB-001` PIPA requires lawful basis before collection.
`KR-REG-OB-002` PIPA requires purpose specificity before use.
`KR-REG-OB-003` PIPA requires separate consent where separate processing basis is needed.
`KR-REG-OB-004` PIPA requires sensitive-information basis before processing.
`KR-REG-OB-005` PIPA requires unique-identifying-information basis before processing.
`KR-REG-OB-006` PIPA Article 24-2 requires statutory RRN basis.
`KR-REG-OB-007` PIPA Article 22-2 requires guardian consent for children under 14.
`KR-REG-OB-008` PIPA Article 28-8 requires overseas transfer basis.
`KR-REG-OB-009` PIPA Article 34 requires leakage notification and reporting workflow.
`KR-REG-OB-010` PIPA decree requires 72-hour report handling for enumerated leakage cases.
`KR-REG-OB-011` CSAP requires certified cell for public-sector cloud workloads.
`KR-REG-OB-012` CSAP requires evidence digest for certification status.
`KR-REG-OB-013` CSAP requires DR cell certification review.
`KR-REG-OB-014` Youth Protection Act requires age verification for youth-harmful media.
`KR-REG-OB-015` Youth Protection Act requires identity verification where Article 16 applies.
`KR-REG-OB-016` Youth Protection Act requires denial of youth access to youth-harmful media.
`KR-REG-OB-017` Communications Secrets Protection Act requires content-inspection denial by default.
`KR-REG-OB-018` Communications Secrets Protection Act requires lawful basis for disclosure.
`KR-REG-OB-019` Communications Secrets Protection Act requires metadata minimization.
`KR-REG-OB-020` Digital Documents Act requires electronic document preservation metadata.
`KR-REG-OB-021` Digital Documents Act requires integrity hash.
`KR-REG-OB-022` Digital Documents Act requires retention rule preservation.
`KR-REG-OB-023` Information Network Act lineage requires identity verification alternatives to RRN.
`KR-REG-OB-024` Information Network Act requires incident cooperation where triggered.
`KR-REG-OB-025` Information Network Act requires network service safeguards evidence.
`KR-REG-OB-026` Medical Service Act requires medical record creation.
`KR-REG-OB-027` Medical Service Act requires medical record preservation.
`KR-REG-OB-028` Medical Service Act requires electronic medical record integrity.
`KR-REG-OB-029` Medical Service Act requires medical access purpose trace.
`KR-REG-OB-030` KR-CYBER requires controlled-source incident classification.
`KR-REG-OB-031` KR-CYBER requires affected facility identification.
`KR-REG-OB-032` KR-CYBER requires damage details for notification.
`KR-REG-OB-033` KR-CYBER requires KISA or authority notification where Article 13 applies.
`KR-REG-OB-034` Cross-law coverage requires incident cross-classification.
`KR-REG-OB-035` Cross-law coverage requires tenant and sub-scope context.
`KR-REG-OB-036` Cross-law coverage requires PII-scrubbed emissions.
`KR-REG-OB-037` Cross-law coverage requires Korean-language user-facing privacy notices.
`KR-REG-OB-038` Cross-law coverage requires processor evidence.
`KR-REG-OB-039` Cross-law coverage requires legal-hold enforcement.
`KR-REG-OB-040` Cross-law coverage requires deny-wins pack precedence.

## Exemption Register

`KR-REG-EX-001` statutory processing may replace consent only when statute is recorded.
`KR-REG-EX-002` contract necessity may support processing only within documented scope.
`KR-REG-EX-003` pseudonymous research may proceed only within PIPA Article 28-2 boundaries.
`KR-REG-EX-004` breach report exception may apply only when decree conditions are documented.
`KR-REG-EX-005` synthetic data may bypass production residency only when no real Korean regulated data is present.
`KR-REG-EX-006` non-public-sector private workload may bypass CSAP only when no CSAP contract applies.
`KR-REG-EX-007` adult verification token may be reused only inside token validity and scope.
`KR-REG-EX-008` user-directed message retrieval may access communications content only for authorized participant.
`KR-REG-EX-009` lawful process may require disclosure only with authority record.
`KR-REG-EX-010` non-regulated draft may follow ordinary document retention only before record status attaches.
`KR-REG-EX-011` CI/DI may be absent when real-name identity is not legally required.
`KR-REG-EX-012` emergency medical care may proceed before ordinary consent but requires after-action audit.
`KR-REG-EX-013` tabletop incident may use test notification markers only in non-production training.
`KR-REG-EX-014` duplicate convenience document copy may be deleted when canonical record remains intact.
`KR-REG-EX-015` legal hold is never an exemption from audit; it is an exemption from deletion.
`KR-REG-EX-016` controller-approved disclosure is never an exemption from PII scrubbing in audit payloads.
`KR-REG-EX-017` tenant contract may narrow legal permission but cannot expand statutory permission.
`KR-REG-EX-018` regulator guidance may narrow pack permission until legal review resolves conflict.
`KR-REG-EX-019` emergency containment copy may exist only with incident approval and deletion checkpoint.
`KR-REG-EX-020` any unlisted exemption is denied by default.

## Traceability Register

`KR-REG-TRACE-001` PIPA Article 15 maps to `pack-kr-pack-1-pipa-purpose-consent`.
`KR-REG-TRACE-002` PIPA Article 17 maps to transfer and third-party provision assessment.
`KR-REG-TRACE-003` PIPA Article 22 maps to separate consent records.
`KR-REG-TRACE-004` PIPA Article 22-2 maps to guardian consent records.
`KR-REG-TRACE-005` PIPA Article 23 maps to sensitive-information basis.
`KR-REG-TRACE-006` PIPA Article 24 maps to unique identifier basis.
`KR-REG-TRACE-007` PIPA Article 24-2 maps to RRN statutory basis.
`KR-REG-TRACE-008` PIPA Article 28-2 maps to de-identified analytics thresholds.
`KR-REG-TRACE-009` PIPA Article 28-8 maps to overseas transfer assessment.
`KR-REG-TRACE-010` PIPA Article 34 maps to breach clocks.
`KR-REG-TRACE-011` PIPA Enforcement Decree Article 40 maps to PIPC/KISA reporting task.
`KR-REG-TRACE-012` Cloud Act Article 23-2 maps to CSAP cell certification.
`KR-REG-TRACE-013` Youth Protection Act Article 16 maps to age and identity gate.
`KR-REG-TRACE-014` Youth Decree Article 17 maps to accepted verification method registry.
`KR-REG-TRACE-015` Communications Secrets Protection Act maps to content-inspection deny.
`KR-REG-TRACE-016` Digital Documents Act maps to preservation metadata.
`KR-REG-TRACE-017` Information Network Act identity lineage maps to CI/DI preference.
`KR-REG-TRACE-018` Information Network Act incident lineage maps to KISA cooperation.
`KR-REG-TRACE-019` Medical Service Act Article 22 maps to record creation and retention.
`KR-REG-TRACE-020` Medical Service Act Article 23 maps to EMR integrity controls.
`KR-REG-TRACE-021` Medical Service Act Article 34 maps to remote support log controls.
`KR-REG-TRACE-022` ICIPA Article 8 maps to critical-infrastructure designation context.
`KR-REG-TRACE-023` ICIPA Article 13 maps to KISA notification path.
`KR-REG-TRACE-024` ICIPA Decree Article 21 maps to incident notification content fields.
`KR-REG-TRACE-025` ADR-0064 maps to pack architecture.
`KR-REG-TRACE-026` ADR-0244 maps to tenant and sub-scope enforcement.
`KR-REG-TRACE-027` ADR-0251 maps to installed compliance pack and cells.
`KR-REG-TRACE-028` ADR-0263 maps to audit event envelope.
`KR-REG-TRACE-029` `docs/localization-packs/kr/pack.yaml` maps to prior KR pack seed.
`KR-REG-TRACE-030` `packs/kr-localization/README.md` maps to pack overview.

## Checkpoint

This file is scoped to `/packs/kr-localization/`.
It does not edit ADRs.
It does not edit microservices.
It does not edit other packs.
It must be verified with line count, required heading presence, and Oya VCS lifecycle commands.
