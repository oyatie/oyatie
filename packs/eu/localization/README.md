---
doc_class: LocalizationPack
pack_id: EU-PACK-1
version: "1.0.0"
status: Draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0248
  - ADR-0251
  - ADR-0255
  - ADR-0263
  - ADR-0304
  - ADR-0308
  - ADR-0316
citing_authority_url:
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2065
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R1925
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32014R0910
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1183
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022L2555
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2554
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022L2464
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32023R2772
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32002L0058
  - https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847
  - https://eur-lex.europa.eu/eli/reg/2023/2854
---

# EU-PACK-1 Localization Pack

## Purpose

EU-PACK-1 is the canonical Oyatie localization pack for European Union and European Economic Area operation.
It binds privacy, platform-accountability, cyber-resilience, operational-resilience, digital-identity, sustainability, AI-governance, and data-access obligations into one pack surface.
It is not a marketing checklist.
It is a deployment and tenant-admission control surface.
It describes what a tenant receives when the EU localization pack is active.
It describes what Oyatie must refuse when a tenant configuration conflicts with Union law.
It describes which microservices load policy fragments.
It describes which audit events must be emitted under ADR-0263.
It describes where stricter sector rules override the general pack baseline.
It describes how EU-specific data-model and API deltas must remain reviewable.

## Scope

This pack applies when a tenant processes personal data of people in the EU or EEA.
This pack applies when a tenant offers digital services to EU recipients.
This pack applies when a tenant is a financial entity or ICT third-party provider subject to DORA.
This pack applies when a tenant operates essential or important services under NIS2.
This pack applies when a tenant deploys high-risk AI systems into EU use contexts.
This pack applies when a tenant provides online platform, marketplace, recommender, messaging, or content surfaces that trigger DSA obligations.
This pack applies when a tenant is designated or expected to support DMA gatekeeper interoperability workflows.
This pack applies when a tenant provides or relies on qualified trust services, electronic signatures, seals, registered delivery, website authentication, or wallets under eIDAS.
This pack applies when a tenant manufactures or distributes products with digital elements under the EU Cyber Resilience Act.
This pack applies when a tenant has connected-product or related-service data-sharing duties under the EU Data Act.
This pack applies when a tenant is in CSRD scope or must supply auditable sustainability data to an in-scope customer.
This pack applies to EU member-state overlays only through a stricter-child-pack mechanism.
This pack does not replace member-state employment, labour, tax, sector-supervision, or consumer-law overlays.
This pack does not make legal conclusions for a tenant.
This pack codifies Oyatie platform obligations, control hooks, and evidence surfaces.

## Version

Pack id: `EU-PACK-1`.
Pack version: `1.0.0`.
Pack status: `Draft`.
Pack date: `2026-05-20`.
Compatibility baseline: ADR-0251 compliance-pack primitive.
Policy baseline: ADR-0243 Cedar as universal gate.
Tenant baseline: ADR-0242 and ADR-0244 tenant and sub-scope discipline.
Audit baseline: ADR-0263 observability emission and audit-event-class registration.
Cell baseline: ADR-0248 cellular architecture and EU/EEA cell certification.
AI baseline: ADR-0308 model lifecycle and EU AI Act compliance.
Conflict-resolution baseline: ADR-0304 cross-jurisdiction conflict resolution.
Capability-tier baseline: ADR-0316 compliance pack overlays inside capability tiers.

## Pack Precedence

01. EU-PACK-1 overrides generic global privacy defaults for EU/EEA data subjects.
02. EU-PACK-1 does not override a stricter member-state law pack.
03. EU-PACK-1 does not override a sector-specific EU pack that is stricter for a tenant.
04. DORA overrides NIS2 for financial entities where DORA is lex specialis.
05. GDPR and ePrivacy override Data Act access workflows when personal data confidentiality is at issue.
06. GDPR Chapter V controls personal-data transfers even when a Data Act data-sharing obligation exists.
07. The Data Act controls non-personal connected-product data access where GDPR is not triggered.
08. DSA platform accountability controls illegal-content and recommender duties.
09. DMA controls designated gatekeeper interoperability and anti-circumvention duties.
10. EU AI Act controls prohibited, high-risk, transparency, conformity, post-market, and serious-incident duties.
11. NIS2 controls cybersecurity governance for essential and important entities not covered by DORA.
12. DORA controls ICT risk, incident reporting, resilience testing, and ICT third-party risk for financial entities.
13. eIDAS controls qualified trust services and electronic-signature legal-effect handling.
14. CSRD and ESRS control sustainability evidence and disclosure lineage.
15. ePrivacy controls terminal-equipment access, cookies, tracking pixels, and local-storage consent.
16. EU Cyber Resilience Act controls secure-by-design and vulnerability handling for products with digital elements.
17. EU Data Act controls connected-product data access, switching, interoperability, and non-personal cloud access safeguards.
18. Oyatie default-deny Cedar remains the runtime enforcement primitive.
19. Oyatie audit-chain remains the immutable evidence primitive.
20. Tenant-specific policy may be stricter but cannot weaken this pack.

## Activated Microservices

01. `tenancy` records EU-PACK-1 activation, jurisdiction scope, and tenant-sub-scope inheritance.
02. `identity` binds EU identity assurance, eIDAS trust-service references, and data-subject authentication.
03. `consent-graph` stores GDPR, ePrivacy, AI-disclosure, recommender, and Data Act sharing consent artifacts.
04. `policy-engine` loads the EU Cedar fragments and returns per-request pack decision evidence.
05. `compliance` owns regulatory mapping, pack evidence bundles, and supervisory-response exports.
06. `audit-chain` seals ADR-0263 event classes for EU enforcement evidence.
07. `observability` emits EU pack metrics, trace attributes, and regulator-facing evidence counters.
08. `workflow-engine` executes DSR, breach, incident, conformity, DORA, and CRA workflows.
09. `governance` owns ROPA, DPIA, AI risk, vendor oversight, and Article 30 records.
10. `cell` restricts placement to EU/EEA-certified cells when the tenant selects EU residency.
11. `cloud-iac` provisions EU/EEA zones, data-plane tags, key residency, and network segmentation.
12. `cloud-k8s` binds runtime placement to EU cell pools and certified cluster profiles.
13. `cloud-secrets` stores regional signing keys, SCC evidence digests, and BYOK attestations.
14. `drive` executes erasure, access, export, legal-hold, retention, and portability operations.
15. `mail` executes DSR discovery and ePrivacy direct-marketing suppression controls.
16. `messenger` executes DSR discovery, DSA notice-action coordination, and AI disclosure banners.
17. `social` executes DSA content moderation, recommender transparency, and minor protection controls.
18. `shorts` executes DSA recommender controls, ad transparency, and minor-risk mitigations.
19. `marketplace` executes DSA trader traceability and Data Act access handoff duties.
20. `intelligence` executes EU AI Act classification, human oversight, transparency, and post-market monitoring.
21. `detection` executes NIS2, DORA, CRA, and GDPR breach detection controls.
22. `incident-management` executes GDPR breach, NIS2 incident, DORA major incident, and CRA vulnerability workflows.
23. `itsm` binds incident tickets, remediation owners, and supervisory-report due dates.
24. `data-pipeline` applies minimisation, pseudonymisation, retention, and disclosure-lineage tags.
25. `data-warehouse` stores aggregate-only EU reporting data and prevents raw cross-cell leakage.
26. `analytics` enforces aggregated telemetry boundaries and CSRD metric lineage.
27. `finops-portal` records DORA ICT provider dependencies and cross-border processing cost views.
28. `developer-sdk` exposes EU pack API fields without bypassing Cedar or audit-chain.
29. `api-gateway` attaches EU pack headers, trace context, tenant scope, and purpose-binding attributes.
30. `ops-dashboard-control-center` surfaces pack status, evidence gaps, and regulator-response runbooks.

## Authority Citations

| Authority | Citation URL | Pack binding |
|---|---|---|
| GDPR | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679 | Privacy principles, lawful basis, transparency, rights, processors, security, breach, transfers, penalties. |
| Digital Services Act | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2065 | Intermediary-service orders, notices, statements, complaint paths, minors, systemic risk, data access. |
| Digital Markets Act | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R1925 | Gatekeeper obligations, interoperability, compliance reporting, anti-circumvention. |
| EU AI Act | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689 | High-risk classification, transparency, provider/deployer duties, conformity, post-market monitoring. |
| eIDAS | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32014R0910 | Electronic signatures, trust services, qualified trust-service provider obligations. |
| European Digital Identity amendment | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1183 | Wallet and identity-framework amendments to eIDAS. |
| NIS2 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022L2555 | Cybersecurity governance, risk management, incident reporting, vulnerability coordination. |
| DORA | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2554 | Financial-sector ICT risk, incident management, resilience testing, third-party risk. |
| CSRD | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022L2464 | Sustainability reporting duties and value-chain evidence requests. |
| ESRS | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32023R2772 | ESRS E1 climate, S1 own workforce, S2 value-chain workers disclosures. |
| ePrivacy Directive | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32002L0058 | Terminal-equipment storage and access under Article 5(3). |
| Cyber Resilience Act | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847 | Cybersecurity requirements for products with digital elements. |
| EU Data Act | https://eur-lex.europa.eu/eli/reg/2023/2854 | Connected-product data access, switching, interoperability, non-personal data safeguards. |
| SCC 2021 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32021D0914 | Article 46 transfer module baseline. |
| Adequacy decisions | https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/adequacy-decisions_en | Article 45 transfer pathway. |
| Schrems II | https://eur-lex.europa.eu/legal-content/EN/CASE/?uri=CELEX:62018CJ0311 | Transfer-impact assessment and supplementary-measure trigger. |

## Activated Cedar Policies

| Cedar fragment id | Primary regulation | Decision boundary |
|---|---|---|
| `pack-eu-gdpr-lawful-basis` | GDPR Articles 5 and 6 | Permit processing only when purpose, lawful basis, and data class align. |
| `pack-eu-gdpr-consent-withdrawal` | GDPR Article 7 | Deny consent-based processing after withdrawal or stale consent. |
| `pack-eu-gdpr-transparency-notice` | GDPR Article 13 | Require notice version before collection or purpose expansion. |
| `pack-eu-gdpr-rights-orchestration` | GDPR Articles 15-22 | Permit authenticated DSR workflows and block overdue closures. |
| `pack-eu-gdpr-privacy-by-design` | GDPR Article 25 | Require minimisation, retention, and privacy-default metadata. |
| `pack-eu-gdpr-processor-contract` | GDPR Article 28 | Block processor onboarding without DPA fields and subprocessors. |
| `pack-eu-gdpr-ropa` | GDPR Article 30 | Deny new processing activity without ROPA row. |
| `pack-eu-gdpr-security` | GDPR Article 32 | Require encryption, access control, and resilience attestation. |
| `pack-eu-gdpr-breach-72h` | GDPR Article 33 | Start breach clock and forbid silent close. |
| `pack-eu-gdpr-cross-border-transfer` | GDPR Articles 44 and 46 | Deny personal-data transfer without adequacy, SCC, BCR, derogation, or local-only flag. |
| `pack-eu-dsa-illegal-content-orders` | DSA Article 9 | Require order authenticity, scope, and response evidence. |
| `pack-eu-dsa-notice-action` | DSA Articles 16 and 17 | Require notice intake, decision statement, and appeal surface. |
| `pack-eu-dsa-terms-transparency` | DSA Article 14 | Require terms version and content-moderation policy reference. |
| `pack-eu-dsa-complaints` | DSA Article 20 | Require internal complaint path and dispute status. |
| `pack-eu-dsa-minors` | DSA Article 28 | Deny targeted-ad and recommender paths for protected minors. |
| `pack-eu-dsa-systemic-risk` | DSA Articles 34 and 38 | Require risk assessment and recommender option metadata for VLOP/VLOSE contexts. |
| `pack-eu-dsa-data-access` | DSA Article 40 | Require vetted-researcher and coordinator evidence before data export. |
| `pack-eu-dma-gatekeeper-controls` | DMA Articles 6, 7, 9, 12, 13 | Enforce interoperability, portability, compliance reporting, and anti-circumvention controls. |
| `pack-eu-ai-act-high-risk-classifier` | AI Act Article 6 and Annex III | Classify AI capability before deployment. |
| `pack-eu-ai-act-provider-obligations` | AI Act Article 16 | Require provider technical documentation, logging, human oversight, and quality management. |
| `pack-eu-ai-act-deployer-obligations` | AI Act Article 26 | Require deployer use-case, input-data, monitoring, and human-oversight evidence. |
| `pack-eu-ai-act-transparency` | AI Act Articles 13 and 50 | Require instruction, disclosure, and synthetic-content labelling. |
| `pack-eu-ai-act-conformity` | AI Act Article 43 | Require conformity assessment status before high-risk activation. |
| `pack-eu-ai-act-post-market` | AI Act Articles 72 and 73 | Require post-market plan and serious-incident workflow. |
| `pack-eu-eidas-trust-service` | eIDAS Articles 24, 25, 26 | Enforce qualified-provider evidence and signature legal-effect handling. |
| `pack-eu-nis2-risk-and-reporting` | NIS2 Articles 20, 21, 23 | Require management accountability, risk measures, and staged incident reports. |
| `pack-eu-nis2-vulnerability` | NIS2 Article 12 | Require coordinated-vulnerability-disclosure workflow. |
| `pack-eu-dora-ict-risk` | DORA Articles 5, 6, 8 | Require financial-entity ICT governance and asset identification. |
| `pack-eu-dora-incident-reporting` | DORA Articles 17 and 19 | Require classified ICT incident workflow and report evidence. |
| `pack-eu-dora-tlpt-third-party` | DORA resilience-testing duties | Require threat-led testing and ICT third-party register linkage. |
| `pack-eu-csrd-esrs-lineage` | CSRD and ESRS E1/S1/S2 | Require sustainability metric source, owner, assurance status, and value-chain lineage. |
| `pack-eu-eprivacy-terminal-equipment` | ePrivacy Article 5(3) | Deny non-essential storage/access without consent. |
| `pack-eu-cra-product-security` | EU Cyber Resilience Act | Require secure-by-design, SBOM, vulnerability handling, and reporting controls. |
| `pack-eu-data-act-access` | EU Data Act | Require connected-product data access, recipient authorization, and non-personal transfer safeguards. |

## Data Model Deltas

| Model | New field or entity | Purpose |
|---|---|---|
| `TenantCompliancePack` | `eu_pack_status` | Records inactive, preview, active, suspended, or retired state. |
| `TenantCompliancePack` | `eu_pack_version` | Pins active pack version. |
| `TenantCompliancePack` | `member_state_overlays` | Lists stricter national overlays. |
| `TenantCompliancePack` | `sector_overlay` | Records financial, healthcare, public-sector, marketplace, platform, AI, or manufacturer scope. |
| `TenantJurisdictionScope` | `eea_personal_data_scope` | Identifies EU/EEA data-subject processing. |
| `TenantJurisdictionScope` | `eu_service_recipient_scope` | Identifies DSA/DMA online recipient targeting. |
| `ProcessingActivity` | `gdpr_lawful_basis` | Stores Article 6 basis. |
| `ProcessingActivity` | `gdpr_article_9_condition` | Stores special-category condition when used. |
| `ProcessingActivity` | `purpose_binding_id` | Joins Cedar decision to consent and ROPA records. |
| `ProcessingActivity` | `privacy_notice_version` | Captures Article 13/14 disclosure version. |
| `ProcessingActivity` | `cross_border_transfer_pathway` | Stores adequacy, SCC module, BCR, derogation, or EU-only. |
| `ProcessingActivity` | `scc_2021_module` | Stores module 1, 2, 3, or 4. |
| `ProcessingActivity` | `transfer_impact_assessment_id` | Links Schrems II supplementary-measure review. |
| `DataSubjectRequest` | `gdpr_right_type` | Stores access, erasure, rectification, portability, restriction, objection, or automated-decision review. |
| `DataSubjectRequest` | `statutory_due_at` | Stores one-month baseline deadline and extension deadline. |
| `DataSubjectRequest` | `identity_assurance_level` | Captures identity verification before disclosure or erasure. |
| `DataSubjectRequest` | `conflict_hold_reason` | Captures legal hold, security, audit-chain immutability, or third-party rights reason. |
| `DsaModerationCase` | `notice_source` | Records user, trusted flagger, authority, automated detection, or moderator source. |
| `DsaModerationCase` | `statement_of_reasons_id` | Links moderation decision to user notice. |
| `DsaModerationCase` | `complaint_case_id` | Links internal complaint process. |
| `DsaRiskRegister` | `systemic_risk_class` | Records illegal content, fundamental rights, civic discourse, gender-based violence, public health, minors, or consumer protection. |
| `DmaGatekeeperObligation` | `core_platform_service` | Stores DMA core service type. |
| `DmaGatekeeperObligation` | `interoperability_capability_id` | Links technical integration commitments. |
| `AiSystemRegistry` | `ai_act_risk_tier` | Stores prohibited, high-risk, transparency, or minimal-risk. |
| `AiSystemRegistry` | `annex_iii_category` | Stores relevant Annex III use-case category. |
| `AiSystemRegistry` | `article_13_instruction_ref` | Links deployer/user instructions. |
| `AiSystemRegistry` | `article_50_disclosure_ref` | Links chatbot, emotion-recognition, biometric-categorisation, or synthetic-content disclosure. |
| `AiSystemRegistry` | `conformity_assessment_id` | Links Article 43 status. |
| `AiSystemRegistry` | `post_market_monitoring_plan_id` | Links Article 72 plan. |
| `EidasTrustArtifact` | `trust_service_type` | Stores signature, seal, timestamp, registered delivery, certificate, wallet, attestation, archive, ledger. |
| `EidasTrustArtifact` | `qualified_status` | Records qualified, advanced, non-qualified, withdrawn, suspended. |
| `Nis2EntityProfile` | `entity_classification` | Stores essential, important, out-of-scope, or supplier-to-covered-entity. |
| `Nis2EntityProfile` | `member_state_authority` | Stores competent authority for the entity. |
| `DoraFinancialEntityProfile` | `financial_entity_type` | Stores bank, insurer, investment firm, payment institution, crypto-asset provider, or ICT provider relation. |
| `DoraIncident` | `major_incident_classification` | Stores materiality threshold result. |
| `DoraThirdPartyRegister` | `ict_service_provider_id` | Records provider relationship. |
| `CsrdDisclosureMetric` | `esrs_standard` | Stores E1, S1, S2, or cross-cutting standard. |
| `CsrdDisclosureMetric` | `assurance_status` | Records draft, management-approved, limited-assurance, reasonable-assurance, restated. |
| `EprivacyConsent` | `terminal_equipment_access_purpose` | Records cookie, local storage, SDK, pixel, device fingerprinting, or telemetry. |
| `CraProductDigitalElement` | `product_security_class` | Records product category, criticality, and vulnerability reporting duties. |
| `DataActAccessRequest` | `connected_product_id` | Links user data access request to product or related service. |

## API Contract Deltas

| API surface | Delta | Reason |
|---|---|---|
| `POST /v1/compliance/packs/eu/activate` | Requires member-state overlays and sector profile. | Prevents incomplete EU pack activation. |
| `GET /v1/compliance/packs/eu/status` | Returns pack version, cell eligibility, policy fragments, and evidence health. | Gives admins a single readiness view. |
| `POST /v1/tenancy/jurisdiction-scopes` | Adds EU/EEA data-subject and EU service-recipient booleans. | Distinguishes GDPR from DSA/DMA applicability. |
| `POST /v1/processing-activities` | Requires lawful basis, purpose, notice, retention, transfer, and processor metadata. | Implements GDPR Articles 5, 6, 13, 28, 30. |
| `GET /v1/processing-activities/{id}/ropa` | Exports Article 30 record. | Supplies supervisory evidence. |
| `POST /v1/dsr/requests` | Adds GDPR right type, identity assurance, statutory clock, and extension reason. | Implements Articles 15-22. |
| `POST /v1/dsr/requests/{id}/execute` | Runs source inventory, conflict checks, and evidence capture. | Prevents silent partial fulfilment. |
| `POST /v1/transfers/eu/assess` | Evaluates adequacy, SCC module, BCR, derogation, and TIA state. | Implements Chapter V. |
| `POST /v1/incidents/privacy-breach` | Starts 72-hour supervisory clock and affected-subject workflow. | Implements GDPR Articles 33 and 34. |
| `POST /v1/dsa/notices` | Accepts content notice, illegal-content category, evidence URL, and notifier status. | Implements DSA Article 16. |
| `POST /v1/dsa/moderation-decisions` | Requires statement of reasons and appeal availability. | Implements DSA Article 17 and 20. |
| `GET /v1/dsa/transparency-reports` | Exports moderation, orders, notices, complaints, and recommender metrics. | Supports DSA reporting. |
| `POST /v1/dma/interoperability-requests` | Registers gatekeeper messaging or core-platform interoperability requests. | Implements DMA Article 7. |
| `GET /v1/dma/compliance-report` | Produces compliance evidence bundle. | Supports DMA Articles 9 and 13. |
| `POST /v1/ai/systems/classify` | Classifies EU AI Act risk tier and Annex III category. | Blocks unclassified AI deployment. |
| `POST /v1/ai/systems/{id}/conformity` | Records Article 43 conformity status. | Gates high-risk deployment. |
| `POST /v1/ai/systems/{id}/post-market-events` | Records monitoring signals and serious incidents. | Implements Articles 72 and 73. |
| `POST /v1/eidas/trust-artifacts/verify` | Captures qualified-provider status and signature validation result. | Supports Articles 24-26. |
| `POST /v1/nis2/incidents` | Records early warning, incident notification, intermediate, and final report. | Implements Article 23. |
| `POST /v1/dora/incidents` | Records major ICT incident classification and financial-sector report stages. | Implements DORA Articles 17 and 19. |
| `POST /v1/dora/third-party-register` | Registers ICT provider, service function, location, subcontractors, and exit plan. | Supports DORA third-party oversight. |
| `POST /v1/csrd/metrics` | Captures ESRS metric, boundary, source, assurance status, and restatement lineage. | Supports CSRD evidence. |
| `POST /v1/eprivacy/terminal-consents` | Records terminal-equipment access consent and exemption basis. | Implements Article 5(3). |
| `POST /v1/cra/products` | Registers product with digital elements, SBOM, vulnerability policy, and conformity status. | Supports Cyber Resilience Act duties. |
| `POST /v1/data-act/access-requests` | Initiates connected-product data access and recipient validation. | Supports Data Act access duties. |

## Audit Event Additions (per ADR-0263)

| Event class | Trigger | Retention class |
|---|---|---|
| `EuPackActivated` | Tenant activates EU-PACK-1. | regulated |
| `EuPackSuspended` | Pack is suspended for failed evidence, cell, or policy gating. | regulated |
| `EuResidencyCellAssigned` | Tenant or sub-scope is placed into EU/EEA cell pool. | regulated |
| `EuResidencyPlacementDenied` | Cell assignment violates residency or certification rule. | security |
| `GdprLawfulBasisRecorded` | Processing activity records Article 6 basis. | regulated |
| `GdprPurposeExpansionDenied` | Processing expands beyond notice or consent. | regulated |
| `GdprDsrRequestReceived` | Data subject right request opens. | regulated |
| `GdprDsrRequestFulfilled` | DSR closes with evidence bundle. | regulated |
| `GdprDsrRequestExtended` | DSR deadline extension is applied. | regulated |
| `GdprErasureConflictRecorded` | Erasure is limited by legal hold, audit-chain, or third-party rights. | legal_hold_capable |
| `GdprCrossBorderTransferEvaluated` | Transfer decision is evaluated. | regulated |
| `GdprTransferBlocked` | Transfer is denied for missing pathway or failed TIA. | security |
| `GdprBreachClockStarted` | Privacy breach clock starts. | security |
| `GdprSupervisoryNotificationSent` | Article 33 notice is sent. | regulated |
| `DsaNoticeReceived` | DSA notice arrives. | regulated |
| `DsaModerationDecisionIssued` | Moderation statement of reasons is issued. | regulated |
| `DsaComplaintOpened` | Internal complaint starts. | regulated |
| `DsaMinorProtectionGateDenied` | Minor-protection policy blocks action. | security |
| `DsaSystemicRiskAssessmentSealed` | VLOP/VLOSE risk assessment is sealed. | regulated |
| `DmaInteroperabilityRequestRecorded` | Gatekeeper interoperability request is registered. | regulated |
| `DmaAntiCircumventionDenied` | Circumvention attempt is denied. | security |
| `EuAiSystemClassified` | AI system classification is recorded. | regulated |
| `EuAiHighRiskConformityRecorded` | High-risk conformity status is recorded. | regulated |
| `EuAiTransparencyDisclosureServed` | Article 50 disclosure is served. | regulated |
| `EuAiSeriousIncidentReported` | Article 73 serious incident report is sent. | security |
| `EidasTrustArtifactVerified` | Trust artifact validation completes. | regulated |
| `Nis2RiskMeasureReviewed` | NIS2 risk-control review is completed. | security |
| `Nis2IncidentReportSubmitted` | NIS2 staged report is submitted. | security |
| `DoraIctRiskFrameworkApproved` | DORA ICT risk framework is approved. | regulated |
| `DoraMajorIncidentReportSubmitted` | DORA incident report is sent. | regulated |
| `DoraTlptExerciseCompleted` | Threat-led penetration test closes. | security |
| `CsrdMetricAssuranceUpdated` | ESRS metric assurance status changes. | regulated |
| `EprivacyTerminalConsentRecorded` | Article 5(3) consent is captured. | regulated |
| `CraVulnerabilityDisclosureReceived` | Product vulnerability report arrives. | security |
| `CraSecurityUpdateReleased` | CRA security update is released. | security |
| `DataActAccessRequestFulfilled` | Connected-product access request is fulfilled. | regulated |

## Failure Modes specific to EU enforcement

| Failure mode | EU-specific risk | Pack response |
|---|---|---|
| Missing lawful basis | GDPR Article 6 non-compliance. | Cedar denies processing and emits `GdprPurposeExpansionDenied`. |
| Stale consent | GDPR Article 7 and ePrivacy Article 5(3) breach. | Consent graph marks inactive and policy denies. |
| Incomplete privacy notice | GDPR Article 13 breach. | API rejects collection until notice version is linked. |
| Overdue DSR | Articles 12 and 15-22 enforcement exposure. | Workflow escalates before due date and blocks closure without evidence. |
| Unmapped processor | GDPR Article 28 breach. | Processor onboarding denied until DPA and subprocessors are present. |
| Missing ROPA | GDPR Article 30 gap. | ProcessingActivity cannot promote to active. |
| Silent security incident | GDPR, NIS2, DORA, or CRA reporting miss. | Incident classification starts multiple statutory clocks. |
| Non-EU transfer without pathway | GDPR Chapter V violation. | Transfer policy denies and requires TIA or EU-only placement. |
| DSA notice ignored | DSA Article 16 failure. | Notice workflow enforces acknowledgement and decision evidence. |
| No statement of reasons | DSA Article 17 failure. | Moderation decision API refuses publish. |
| Minor targeted ad | DSA Article 28 failure. | Cedar blocks protected-minor ad targeting. |
| VLOP systemic risk stale | DSA Article 34 failure. | Systemic risk assessment expiration blocks recommender release. |
| DMA interoperability delay | Gatekeeper enforcement risk. | Interoperability SLA and evidence tasks are opened. |
| AI system not classified | EU AI Act deployment risk. | Intelligence activation is denied until classification exists. |
| High-risk AI lacks conformity | EU AI Act Article 43 failure. | Deployment is blocked for Annex III high-risk systems. |
| AI disclosure omitted | EU AI Act Article 50 breach. | UI and API call fail closed unless disclosure served. |
| Qualified signature mishandled | eIDAS legal-effect risk. | Trust artifact verification is required before legal workflow finalization. |
| NIS2 management accountability missing | Article 20 governance risk. | Entity profile cannot activate without accountable approvers. |
| DORA ICT provider unregistered | DORA third-party oversight risk. | Financial-tenant pack activation blocks dependent production release. |
| CSRD metric lacks lineage | Assurance and greenwashing risk. | Metric cannot move to assurance-ready. |
| Cookie consent omitted | ePrivacy Article 5(3) risk. | Terminal-equipment access is denied except strictly necessary cases. |
| CRA vulnerability report ignored | Product security enforcement risk. | Security update workflow and authority notification clock open. |
| Data Act request over-shares personal data | GDPR/Data Act conflict. | GDPR minimisation and transfer gates override export. |

## Worked Examples

### Example 1: SaaS tenant processing EU customer data

Tenant `acme-eu` activates EU-PACK-1.
The tenant selects EU/EEA residency.
`tenancy` records `eu_pack_status=active`.
`cell` assigns `eu-eea-certified` cell candidates only.
`ProcessingActivity` rows require Article 6 lawful basis.
`consent-graph` stores consent only where consent is the chosen basis.
`drive`, `mail`, `messenger`, and `social` index subject-identifying records for DSR workflows.
`policy-engine` denies cross-border personal-data transfer until adequacy or SCC evidence is present.
`audit-chain` seals `EuPackActivated` and `EuResidencyCellAssigned`.
The admin dashboard shows green residency, yellow ROPA if any processing activity lacks Article 30 fields, and red transfer if SCC module is missing.

### Example 2: Online marketplace with user content

Tenant `marketplace-eu` activates EU-PACK-1.
`marketplace` marks seller pages as DSA intermediary-service surfaces.
`social` and `shorts` load DSA notice-action policy.
An authority order arrives under DSA Article 9.
`DsaModerationCase` records order authenticity, issuing authority, scope, and affected content.
`workflow-engine` routes the order to trust-and-safety reviewers.
`policy-engine` blocks content action until a statement of reasons template is linked where required.
`audit-chain` seals `DsaNoticeReceived` and `DsaModerationDecisionIssued`.
If the tenant is classified as VLOP, `DsaRiskRegister` also requires annual systemic risk assessment.

### Example 3: Financial tenant under DORA

Tenant `bank-eu` activates EU-PACK-1 with sector overlay `financial`.
`compliance` classifies DORA applicability.
`DoraFinancialEntityProfile` stores entity type and competent authority.
`finops-portal` records ICT third-party dependencies.
`incident-management` classifies an outage as a potential major ICT incident.
`workflow-engine` starts initial, intermediate, and final report clocks.
`audit-chain` seals `DoraMajorIncidentReportSubmitted`.
NIS2 remains mapped for background cyber posture, but DORA is the controlling sector rule for ICT incident reporting.

### Example 4: AI recruiting assistant

Tenant `hr-eu` activates an AI candidate-ranking system.
`intelligence` classifies the system as Annex III employment high-risk.
`AiSystemRegistry` stores the Annex III category.
`policy-engine` denies production deployment until Article 13 instructions and Article 43 conformity evidence exist.
`consent-graph` records candidate transparency acknowledgements where applicable.
Human oversight steps are assigned to HR reviewers.
`audit-chain` seals `EuAiSystemClassified` and `EuAiHighRiskConformityRecorded`.
Post-market monitoring collects drift, override, complaint, and serious-incident signals.

### Example 5: Connected device vendor

Tenant `device-eu` ships a connected product.
`CraProductDigitalElement` records SBOM, update mechanism, support period, and vulnerability contact.
`DataActAccessRequest` records user access requests for product and related-service data.
`policy-engine` distinguishes personal data from non-personal telemetry.
GDPR minimisation and transfer gates apply to personal data.
The Data Act access workflow applies to eligible product data.
CRA vulnerability reports open security update and notification workflows.
`audit-chain` seals `CraVulnerabilityDisclosureReceived`, `CraSecurityUpdateReleased`, and `DataActAccessRequestFulfilled`.

## Cross-References

| Artifact | Relationship |
|---|---|
| `packs/eu-localization/regulatory-coverage.md` | Regulation-by-regulation control matrix. |
| `packs/eu-localization/data-residency-and-cross-border.md` | EU/EEA placement, Schrems II, SCC 2021 modules, adequacy. |
| `packs/eu-localization/dsr-and-portability.md` | GDPR Article 15-22 workflows. |
| `packs/eu-localization/high-risk-ai-systems.md` | AI Act Annex III and high-risk operation. |
| `packs/eu-localization/dora-operational-resilience.md` | DORA ICT risk, incident, third-party, and TLPT duties. |
| `docs/decisions/ADR-0702-identity-authz-live-apex.md` | Tenant substrate. |
| `docs/decisions/ADR-0700-ci-admission-live-apex.md` | Policy substrate. |
| `docs/decisions/ADR-0702-identity-authz-live-apex.md` | Tenant sub-scope. |
| `docs/decisions/ADR-0700-ci-admission-live-apex.md` | Cell placement and certification. |
| `docs/decisions/ADR-0708-platform-foundations-live-apex.md` | Compliance-pack primitive. |
| `docs/decisions/ADR-0701-monorepo-capability-live-apex.md` | AI substrate and EU AI Act tier UI. |
| `docs/decisions/ADR-0706-observability-live-apex.md` | Audit-event emission contract. |
| `docs/decisions/ADR-0709-general-live-apex.md` | Jurisdiction conflict handling. |
| `docs/decisions/ADR-0709-general-live-apex.md` | AI Act model lifecycle. |
| `specs/audit-event-class-registry.json` | Event class registry shape. |
| `specs/capability-tier-schema.json` | Capability-tier compliance overlay shape. |

## Operational Checkpoints

01. Confirm tenant EU-PACK-1 activation is explicit.
02. Confirm member-state overlays are named or explicitly absent.
03. Confirm sector overlay is classified.
04. Confirm EU/EEA data-subject processing is declared.
05. Confirm EU service-recipient targeting is declared.
06. Confirm DSA platform class is declared.
07. Confirm DMA gatekeeper status is declared.
08. Confirm AI systems are classified before deployment.
09. Confirm high-risk AI has conformity status before deployment.
10. Confirm DORA financial-entity status is declared.
11. Confirm NIS2 essential/important entity status is declared.
12. Confirm CSRD reporting status is declared.
13. Confirm CRA product-with-digital-elements status is declared.
14. Confirm Data Act connected-product status is declared.
15. Confirm eIDAS trust-service status is declared.
16. Confirm ePrivacy terminal-equipment access inventory exists.
17. Confirm Article 30 ROPA rows exist for all processing activities.
18. Confirm Article 28 processor contracts exist for processors.
19. Confirm SCC module or adequacy pathway exists for cross-border personal-data transfers.
20. Confirm transfer impact assessment exists where Schrems II risk is present.
21. Confirm breach, incident, AI, DORA, CRA, and DSA workflows have statutory clocks.
22. Confirm ADR-0263 event classes are registered before emitted.
23. Confirm logs, traces, metrics, and audit-chain evidence use tenant_id and pack_id.
24. Confirm dashboard surfaces distinguish warning from enforcement failure.
25. Confirm pack suspension blocks risky actions without deleting evidence.

## Non-Goals

This pack does not create member-state labour-law guidance.
This pack does not create tax localization.
This pack does not create language translation files.
This pack does not create country-specific cookie banner copy.
This pack does not replace counsel review for tenant legal positions.
This pack does not certify a tenant under any EU scheme.
This pack does not assert DSA VLOP/VLOSE status.
This pack does not assert DMA gatekeeper designation.
This pack does not assert DORA financial-entity status.
This pack does not assert NIS2 essential or important entity status.
This pack does not assert CSRD in-scope undertaking status.
This pack does not assert CRA product classification.
This pack does not assert Data Act role classification.
This pack does not weaken existing Oyatie default-deny policy.
This pack does not weaken another active compliance pack.

## Pack Stop Condition

The pack is ready for documentation handoff when all six documents exist.
The pack is ready for documentation handoff when each document has the required frontmatter.
The pack is ready for documentation handoff when each document includes authority citations.
The pack is ready for documentation handoff when each document names activated Cedar policies.
The pack is ready for documentation handoff when each document names data model deltas.
The pack is ready for documentation handoff when each document names API contract deltas.
The pack is ready for documentation handoff when each document names ADR-0263 audit additions.
The pack is ready for documentation handoff when each document names EU enforcement failure modes.
The pack is ready for documentation handoff when each document includes worked examples.
The pack is ready for documentation handoff when each document cross-references the companion pack docs.
The pack is ready for documentation handoff when retired VCS ratchet accepts `eu_pack_docs:6`.
The pack is ready for documentation handoff when retired VCS ratchet accepts `eu_pack_docs:6`.
The pack is ready for documentation handoff when retired VCS ratchet accepts the requested bundle.

## Microservice Activation Matrix

| Microservice | EU pack activation duty | Primary evidence |
|---|---|---|
| `tenancy` | Store pack activation, sector overlay, member-state overlay, and sub-scope inheritance. | `EuPackActivated` audit row plus tenant compliance snapshot. |
| `tenancy` | Reject silent activation through imported tenant templates. | `EuPackActivationDenied` audit row with source template id. |
| `tenancy` | Preserve pack state during tenant split, merge, or sub-scope move. | Tenant scope migration evidence bundle. |
| `identity` | Verify data-subject identity before SAR, portability, rectification, or erasure disclosure. | Identity assurance decision and challenge transcript hash. |
| `identity` | Bind eIDAS trust artifacts to workflow actors where qualified signatures are used. | `EidasTrustArtifactVerified` audit row. |
| `identity` | Refuse privileged admin bypass for EU DSR identity checks. | Cedar denial with privileged principal and purpose. |
| `consent-graph` | Store GDPR consent with purpose, withdrawal, expiry, and notice version. | Consent artifact and `GdprConsentRecorded`. |
| `consent-graph` | Store ePrivacy terminal-equipment consent separately from GDPR service consent. | `EprivacyTerminalConsentRecorded`. |
| `consent-graph` | Store AI disclosure acknowledgements without treating them as legal consent when not consent-based. | AI transparency acknowledgement artifact. |
| `policy-engine` | Load pack fragments at tenant activation. | Cedar fragment manifest digest. |
| `policy-engine` | Evaluate every EU pack decision with `tenant_id`, `pack_id`, `jurisdiction_code`, and `purpose_id`. | Cedar decision envelope. |
| `policy-engine` | Deny actions when the active sector overlay requires stricter policy. | Cedar denial event and sector overlay id. |
| `compliance` | Maintain regulation-to-control matrix. | Regulatory coverage export. |
| `compliance` | Maintain supervisory response bundles. | Regulator evidence package digest. |
| `compliance` | Maintain pack evidence-health score. | Compliance dashboard snapshot. |
| `audit-chain` | Seal every ADR-0263 EU pack event class. | Merkle root, event class, schema version. |
| `audit-chain` | Preserve immutable evidence where erasure conflicts with audit retention. | `GdprErasureConflictRecorded`. |
| `audit-chain` | Support per-regulation evidence export. | Filtered audit-chain export manifest. |
| `observability` | Emit EU pack metrics with cardinality budget. | Prometheus metric family with pack labels. |
| `observability` | Link logs and traces to audit events. | W3C trace id and `audit_id`. |
| `observability` | Surface statutory clock health. | Regulator-clock dashboard panel. |
| `workflow-engine` | Execute DSR workflow with due-date and extension states. | Workflow state transcript. |
| `workflow-engine` | Execute breach and cyber incident workflow branching. | Incident workflow stage evidence. |
| `workflow-engine` | Execute AI conformity and post-market workflows. | AI lifecycle evidence bundle. |
| `governance` | Maintain ROPA records. | Article 30 record export. |
| `governance` | Maintain DPIA and AI risk-assessment artifacts. | DPIA and risk file digests. |
| `governance` | Maintain processor and subprocessor records. | DPA and subprocessor register snapshot. |
| `cell` | Enforce EU/EEA placement for EU-residency tenants. | Cell assignment decision. |
| `cell` | Deny cell migration to non-approved regions when EU-only flag is active. | `EuResidencyPlacementDenied`. |
| `cell` | Record failover target eligibility. | Cell failover eligibility matrix. |
| `cloud-iac` | Provision EU cell resources with data-residency tags. | Infrastructure state digest. |
| `cloud-iac` | Attach key-residency constraints. | KMS policy attestation. |
| `cloud-iac` | Reject non-EU backup target for EU-only workloads. | Admission denial evidence. |
| `cloud-k8s` | Bind workloads to certified cluster pools. | Namespace and node-pool placement report. |
| `cloud-k8s` | Enforce taints and labels for EU pack workloads. | Kubernetes admission log digest. |
| `cloud-k8s` | Preserve rolling-update evidence for regulated workloads. | Deployment rollout transcript. |
| `cloud-secrets` | Store EU transfer and signing material in approved key domains. | Secret location and key policy digest. |
| `cloud-secrets` | Record SCC/TIA evidence hashes without storing raw legal files in application logs. | Evidence digest pointer. |
| `cloud-secrets` | Support break-glass access with EU audit class. | Break-glass event and reviewer signature. |
| `drive` | Search subject data for DSR access, erasure, and portability. | Data inventory result. |
| `drive` | Respect legal hold and audit-chain immutability conflicts. | Erasure conflict evidence. |
| `drive` | Export portable data in structured format. | Portability package manifest. |
| `mail` | Discover mailbox data for DSR and breach investigations. | Mail inventory manifest. |
| `mail` | Enforce ePrivacy direct-marketing suppression state. | Suppression decision event. |
| `mail` | Preserve Article 5(3) consent separation for tracking pixels. | Terminal-equipment consent linkage. |
| `messenger` | Discover conversational data for DSR workflows. | Message inventory manifest. |
| `messenger` | Surface AI disclosure when bot or synthetic assistant is used. | `EuAiTransparencyDisclosureServed`. |
| `messenger` | Support DMA interoperability request logging where applicable. | Interoperability request artifact. |
| `social` | Process DSA content notices and statements of reasons. | Moderation case evidence. |
| `social` | Enforce minor-protection recommender and ad gates. | Minor-protection Cedar decision. |
| `social` | Export transparency-report metrics. | DSA transparency bundle. |
| `shorts` | Apply recommender transparency and alternative-feed controls. | Recommender setting evidence. |
| `shorts` | Apply DSA systemic-risk controls for VLOP/VLOSE tenants. | Risk-assessment linkage. |
| `shorts` | Block non-consented tracking SDK access. | ePrivacy denial event. |
| `marketplace` | Capture trader and product traceability for DSA surfaces. | Trader traceability register. |
| `marketplace` | Coordinate Data Act access requests involving connected products. | Data Act access case. |
| `marketplace` | Prevent product listing if CRA product-security metadata is missing. | CRA product gate decision. |
| `intelligence` | Classify every AI capability under EU AI Act risk tiers. | `EuAiSystemClassified`. |
| `intelligence` | Enforce Article 13 instructions and Article 50 disclosures. | Instruction and disclosure refs. |
| `intelligence` | Execute post-market monitoring signal capture. | Monitoring signal event. |
| `detection` | Detect personal-data breach indicators. | Breach candidate evidence. |
| `detection` | Detect NIS2/DORA cyber incident indicators. | Cyber incident classification evidence. |
| `detection` | Detect CRA actively exploited vulnerability signals. | Product vulnerability signal evidence. |
| `incident-management` | Route incidents to GDPR, NIS2, DORA, CRA, or combined workflows. | Incident classification tree. |
| `incident-management` | Maintain statutory notification clocks. | Clock state transition log. |
| `incident-management` | Record remediation and final-report closure. | Final report bundle. |
| `itsm` | Bind incident records to remediation tickets. | Ticket linkage. |
| `itsm` | Track regulatory owner assignment. | Owner assignment event. |
| `itsm` | Preserve escalation history. | Escalation timeline. |
| `data-pipeline` | Apply minimisation and pseudonymisation before downstream use. | Data transformation lineage. |
| `data-pipeline` | Attach ESRS metric source lineage. | CSRD source trace. |
| `data-pipeline` | Prevent raw personal-data export into non-EU pipelines. | Transfer denial event. |
| `data-warehouse` | Store aggregate-only reporting tables where cross-cell rollup is needed. | Aggregate lineage manifest. |
| `data-warehouse` | Reject raw EU personal data in global reporting datasets. | Warehouse admission denial. |
| `data-warehouse` | Preserve CSRD restatement lineage. | Restatement chain. |
| `analytics` | Compute DSA, DORA, NIS2, AI, and CSRD metrics from approved aggregates. | Analytics query evidence. |
| `analytics` | Prevent small-cell reidentification in reports. | k-anonymity or suppression decision. |
| `analytics` | Link dashboard views to pack evidence. | Dashboard evidence pointer. |
| `finops-portal` | Register ICT providers and critical functions for DORA. | Third-party register export. |
| `finops-portal` | Track EU data-transfer cost and residency cost attribution. | Cost attribution audit row. |
| `finops-portal` | Support exit-plan cost evidence for critical ICT providers. | Exit-plan estimate. |
| `developer-sdk` | Expose pack-aware headers and response fields. | SDK contract test evidence. |
| `developer-sdk` | Prevent client-side bypass of DSR or transfer gates. | Negative fixture result. |
| `developer-sdk` | Publish EU error codes and retry semantics. | API reference version. |
| `api-gateway` | Attach `pack_id`, `purpose_id`, `tenant_id`, and jurisdiction attributes. | Gateway trace attributes. |
| `api-gateway` | Reject missing purpose for regulated actions. | 403 denial with Cedar reason. |
| `api-gateway` | Preserve W3C trace context into all EU workflows. | Trace propagation evidence. |
| `ops-dashboard-control-center` | Show EU pack readiness by regulation and microservice. | Readiness dashboard snapshot. |
| `ops-dashboard-control-center` | Show statutory clock risk. | Clock health panel. |
| `ops-dashboard-control-center` | Show checkpoint status for VCS promote. | Pack promotion evidence summary. |

## Error Code Additions

| Error code | HTTP status | Meaning |
|---|---|---|
| `EU_PACK_NOT_ACTIVE` | 409 | Tenant attempted EU-only operation before pack activation. |
| `EU_MEMBER_STATE_OVERLAY_REQUIRED` | 422 | Tenant declared member-state processing but did not select overlay. |
| `EU_SECTOR_PROFILE_REQUIRED` | 422 | Tenant requires DORA, NIS2, CSRD, AI, DSA, DMA, CRA, or Data Act classification. |
| `GDPR_LAWFUL_BASIS_REQUIRED` | 422 | Processing activity lacks Article 6 basis. |
| `GDPR_NOTICE_VERSION_REQUIRED` | 422 | Collection lacks linked Article 13 notice. |
| `GDPR_ROPA_REQUIRED` | 409 | Processing activity lacks Article 30 record. |
| `GDPR_PROCESSOR_DPA_REQUIRED` | 409 | Processor is missing Article 28 contract fields. |
| `GDPR_TRANSFER_PATHWAY_REQUIRED` | 409 | Personal-data transfer lacks Article 44/46 pathway. |
| `GDPR_TIA_REQUIRED` | 409 | Transfer requires Schrems II transfer-impact assessment. |
| `GDPR_DSR_IDENTITY_ASSURANCE_REQUIRED` | 403 | Requester identity is not verified enough for disclosure. |
| `DSA_STATEMENT_OF_REASONS_REQUIRED` | 409 | Moderation decision lacks statement of reasons. |
| `DSA_MINOR_TARGETING_FORBIDDEN` | 403 | Protected-minor targeting is blocked. |
| `DMA_INTEROPERABILITY_SCOPE_REQUIRED` | 422 | Gatekeeper workflow lacks core-platform service scope. |
| `EU_AI_CLASSIFICATION_REQUIRED` | 409 | AI system lacks EU AI Act risk classification. |
| `EU_AI_CONFORMITY_REQUIRED` | 409 | High-risk AI lacks conformity status. |
| `EU_AI_DISCLOSURE_REQUIRED` | 409 | Article 50 disclosure was not served. |
| `EIDAS_TRUST_ARTIFACT_INVALID` | 422 | Qualified or advanced trust artifact failed validation. |
| `NIS2_ENTITY_PROFILE_REQUIRED` | 409 | NIS2-covered tenant lacks entity profile. |
| `DORA_ENTITY_PROFILE_REQUIRED` | 409 | Financial tenant lacks DORA profile. |
| `DORA_ICT_PROVIDER_REGISTER_REQUIRED` | 409 | Critical ICT dependency lacks register entry. |
| `CSRD_METRIC_LINEAGE_REQUIRED` | 409 | ESRS metric lacks source lineage. |
| `EPRIVACY_TERMINAL_CONSENT_REQUIRED` | 403 | Terminal-equipment access lacks consent or exemption. |
| `CRA_PRODUCT_SECURITY_FILE_REQUIRED` | 409 | Product with digital elements lacks security file. |
| `DATA_ACT_ACCESS_SCOPE_REQUIRED` | 422 | Data Act request lacks connected-product or related-service scope. |

## Checkpoint Record

Checkpoint id: `eu-localization-pack-w1-docs`.
Checkpoint owner: `codex-eu-localization-pack-w1`.
Checkpoint scope: `packs/eu-localization/`.
Checkpoint evidence target: `eu_pack_docs:6`.
Checkpoint edit boundary: six new Markdown documents only.
Checkpoint exclusion: no ADR edits.
Checkpoint exclusion: no microservice edits.
Checkpoint exclusion: no `packs/kr-localization` edits.
Checkpoint exclusion: no other pack edits.
Checkpoint validation path: line count, required section grep, VCS verify, VCS done, VCS promote.
