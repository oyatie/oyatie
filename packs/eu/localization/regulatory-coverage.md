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

# EU-PACK-1 Regulatory Coverage Matrix

## Coverage Rule

This document is the canonical regulation-to-platform matrix for EU-PACK-1.
Every row maps a legal obligation to a Cedar decision, data delta, API delta, audit event, and evidence owner.
Rows are intentionally operational rather than legal-advice prose.
Rows do not assert tenant applicability.
Rows state what Oyatie must expose when the tenant or sector profile says a rule applies.
Where a regulation uses member-state transposition, this matrix records the EU-level baseline only.
Where a regulation is directly applicable, this matrix records the directly applicable pack behavior.
Where a regulation interacts with GDPR, GDPR minimisation and transfer controls remain the safety floor.
Where a regulation requires external legal judgement, this matrix records the evidence hook and blocks automation from pretending the judgement is done.

## Authority Citations

| Regulation | Official authority URL | Coverage scope |
|---|---|---|
| GDPR Regulation (EU) 2016/679 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32016R0679 | Articles 5, 6, 7, 13, 17, 22, 25, 28, 30, 32, 33, 44, 46, 83. |
| DSA Regulation (EU) 2022/2065 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2065 | Articles 9, 14, 16, 17, 20, 28, 34, 38, 40. |
| DMA Regulation (EU) 2022/1925 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R1925 | Articles 6, 7, 9, 12, 13. |
| AI Act Regulation (EU) 2024/1689 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1689 | Articles 6, 13, 14, 16, 26, 43, 50, 72, 73 and Annex III. |
| eIDAS Regulation (EU) No 910/2014 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32014R0910 | Articles 24, 25, 26. |
| European Digital Identity Framework amendment | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R1183 | eIDAS wallet and trust-service context. |
| NIS2 Directive (EU) 2022/2555 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022L2555 | Articles 7, 12, 14, 20, 21, 23. |
| DORA Regulation (EU) 2022/2554 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022R2554 | Articles 5, 6, 8, 17, 19. |
| CSRD Directive (EU) 2022/2464 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32022L2464 | Sustainability reporting obligations. |
| ESRS Delegated Regulation (EU) 2023/2772 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32023R2772 | ESRS E1, S1, S2 disclosure evidence. |
| ePrivacy Directive 2002/58/EC | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32002L0058 | Article 5(3) terminal-equipment access. |
| EU Cyber Resilience Act Regulation (EU) 2024/2847 | https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX:32024R2847 | Products with digital elements, vulnerability handling, conformity evidence. |
| EU Data Act Regulation (EU) 2023/2854 | https://eur-lex.europa.eu/eli/reg/2023/2854 | Connected-product data access, switching, interoperability, non-personal data safeguards. |

## Coverage Matrix

| Ref | Requirement summary | Oyatie control | Evidence owner |
|---|---|---|---|
| GDPR Art. 5 | Principles: lawfulness, fairness, transparency, purpose limitation, minimisation, accuracy, storage limitation, integrity, confidentiality, accountability. | Processing activities require purpose, lawful basis, data class, retention, security, owner, and audit evidence. | `governance` |
| GDPR Art. 6 | Processing must have lawful basis. | Cedar denies regulated processing unless `gdpr_lawful_basis` is set and compatible with purpose. | `policy-engine` |
| GDPR Art. 7 | Consent must be demonstrable and withdrawable. | Consent records carry purpose, collection method, notice version, withdrawal, and effective status. | `consent-graph` |
| GDPR Art. 13 | Data subjects must receive collection-time information. | Collection APIs require notice version, controller identity, purposes, recipients, retention, rights, and transfer disclosure. | `governance` |
| GDPR Art. 17 | Erasure right must be available subject to lawful exceptions. | DSR workflow executes erasure, tombstone, suppression, and conflict-hold branches. | `drive` |
| GDPR Art. 22 | Automated decisions with legal or similarly significant effect require safeguards. | AI and rules workflows require human review, explanation, objection path, and significant-effect classification. | `intelligence` |
| GDPR Art. 25 | Data protection by design and default. | Default fields enforce minimisation, retention, pseudonymisation, purpose binding, and privacy defaults before activation. | `governance` |
| GDPR Art. 28 | Processor contracts and subprocessor controls. | Processor onboarding blocks without DPA terms, subprocessors, assistance duties, deletion/return terms, and audit rights. | `compliance` |
| GDPR Art. 30 | Records of processing activities. | ROPA export is generated from `ProcessingActivity` and cannot be marked complete with missing owners. | `governance` |
| GDPR Art. 32 | Security of processing. | Encryption, access control, resilience, restore, pseudonymisation, logging, and incident controls are activation prerequisites. | `cloud-secrets` |
| GDPR Art. 33 | Notify supervisory authority without undue delay and where feasible within 72 hours. | Breach clock starts at confirmation and blocks silent closure. | `incident-management` |
| GDPR Art. 44 | Transfers to third countries require Chapter V conditions. | Cross-border transfer API blocks if no transfer pathway exists. | `policy-engine` |
| GDPR Art. 46 | Appropriate safeguards include SCCs and other safeguards. | SCC module, TIA, supplementary measures, and recipient role are required for SCC transfer. | `compliance` |
| GDPR Art. 83 | Administrative fines depend on infringement class and factors. | Enforcement-risk scoring records article, severity, recurrence, mitigation, and cooperation evidence. | `compliance` |
| DSA Art. 9 | Orders to act against illegal content must be processed and answered. | Authority-order intake validates issuer, jurisdiction, scope, deadline, and action result. | `social` |
| DSA Art. 14 | Terms and conditions must describe restrictions and moderation. | Terms versions link to moderation rules, recommender rules, and user-facing change history. | `governance` |
| DSA Art. 16 | Notice and action mechanisms for illegal content. | Notice intake accepts structured content reference, legal ground, evidence, notifier type, and acknowledgement. | `social` |
| DSA Art. 17 | Statement of reasons for restrictions. | Moderation decision API requires reason, policy, automated/manual flag, appeal route, and affected surface. | `social` |
| DSA Art. 20 | Internal complaint-handling system. | Complaint workflow gives users a tracked review path and preserves original decision evidence. | `workflow-engine` |
| DSA Art. 28 | Online protection of minors. | Minor profile blocks targeted advertising, sensitive profiling, and unsafe recommender defaults. | `policy-engine` |
| DSA Art. 34 | VLOP/VLOSE systemic risk assessment. | Systemic risk register maps risk class, mitigation, residual risk, owner, and board signoff. | `compliance` |
| DSA Art. 38 | Recommender-system transparency and options. | Recommender settings expose main parameters and non-profiling option where required. | `shorts` |
| DSA Art. 40 | Data access and scrutiny for regulators and vetted researchers. | Researcher export requires authority request, vetting status, data minimisation, and secure access controls. | `data-pipeline` |
| DMA Art. 6 | Gatekeeper obligations susceptible to specification. | Gatekeeper profile maps each core-platform service to applicable obligation gates. | `compliance` |
| DMA Art. 7 | Interoperability for number-independent interpersonal communications. | Messaging interoperability requests record capability, security design, contact, deadline, and response. | `messenger` |
| DMA Art. 9 | Compliance reports. | Compliance report API exports controls, evidence, testing, and unresolved gaps. | `compliance` |
| DMA Art. 12 | Updating obligations. | Obligation-change registry version-pins updated requirements and forces policy review. | `governance` |
| DMA Art. 13 | Anti-circumvention. | Cedar denies design changes that produce equivalent circumvention of obligation paths. | `policy-engine` |
| AI Act Art. 6 | Classification rules for high-risk AI. | AI system registry classifies Annex III and product-safety high-risk status before activation. | `intelligence` |
| AI Act Art. 13 | Transparency and instructions for use. | Deployer instructions, intended purpose, limitations, human oversight, and logging info are required. | `intelligence` |
| AI Act Art. 14 | Human oversight for high-risk AI. | Human oversight plan names reviewer role, override authority, monitoring signals, and escalation path. | `workflow-engine` |
| AI Act Art. 16 | Provider obligations for high-risk AI. | Provider profile requires quality management, technical docs, logging, conformity, post-market monitoring, and incident handling. | `intelligence` |
| AI Act Art. 26 | Deployer obligations for high-risk AI. | Deployer profile requires use according to instructions, input data relevance, monitoring, logs, and human oversight. | `governance` |
| AI Act Art. 43 | Conformity assessment. | High-risk deployment blocks unless conformity route, evidence, and status are recorded. | `compliance` |
| AI Act Art. 50 | Transparency obligations for certain AI systems. | Chatbot, synthetic content, emotion recognition, and biometric categorisation disclosures are served and logged. | `intelligence` |
| AI Act Art. 72 | Post-market monitoring. | Monitoring plan captures performance, drift, complaints, overrides, incidents, and corrective actions. | `observability` |
| AI Act Art. 73 | Serious incidents and malfunctioning. | Serious incident workflow routes report, remediation, and authority notification evidence. | `incident-management` |
| AI Act Annex III | High-risk use cases. | Annex III category is required for employment, education, essential services, law-enforcement-adjacent, migration, justice, democratic processes, and other covered uses. | `intelligence` |
| eIDAS Art. 24 | Requirements for qualified trust service providers. | Qualified provider validation requires certificate chain, supervisory status, revocation, and timestamp evidence. | `identity` |
| eIDAS Art. 25 | Legal effects of electronic signatures. | Signature workflow cannot reject solely because a signature is electronic; qualified signatures receive qualified treatment. | `workflow-engine` |
| eIDAS Art. 26 | Requirements for advanced electronic signatures. | Advanced signature validation checks unique link, signer identification, sole control, and change detection. | `identity` |
| NIS2 Art. 7 | National cybersecurity strategies. | Member-state overlay records authority and strategy-derived sector obligations without hardcoding national law into baseline. | `compliance` |
| NIS2 Art. 12 | Coordinated vulnerability disclosure and European vulnerability database. | Vulnerability intake records disclosure source, affected service, remediation, publication, and ENISA/CVD handoff state. | `detection` |
| NIS2 Art. 14 | Cooperation Group. | Authority coordination evidence records member-state cooperation requests and response deadlines. | `compliance` |
| NIS2 Art. 20 | Governance. | Covered entity profile requires management approval, training, accountability, and review cadence. | `governance` |
| NIS2 Art. 21 | Cybersecurity risk-management measures. | Risk-control baseline includes incident handling, business continuity, supply chain, vulnerability handling, encryption, access control, and MFA. | `cloud-k8s` |
| NIS2 Art. 23 | Reporting obligations. | Incident workflow supports early warning, incident notification, intermediate report, and final report. | `incident-management` |
| DORA Art. 5 | Governance and organisation. | Financial entity ICT governance requires board-level accountability and control ownership. | `governance` |
| DORA Art. 6 | ICT risk management framework. | ICT risk framework stores strategy, policies, controls, roles, and review cycle. | `compliance` |
| DORA Art. 8 | Identification. | Asset, function, dependency, data, and ICT provider inventory must be current. | `finops-portal` |
| DORA Art. 17 | ICT-related incident management process. | Incident taxonomy, detection, classification, response, escalation, and root-cause workflow are mandatory. | `incident-management` |
| DORA Art. 19 | Reporting of major ICT-related incidents. | Major incident reports are staged and sealed with authority destination. | `incident-management` |
| CSRD | Corporate sustainability reporting. | Reporting profile records undertaking role, reporting year, consolidation boundary, assurance status, and value-chain requests. | `analytics` |
| ESRS E1 | Climate change. | Climate metric model records emissions scopes, transition plan, energy, targets, assumptions, and source systems. | `data-warehouse` |
| ESRS S1 | Own workforce. | Workforce metric model records employee boundary, headcount, working conditions, equal treatment, training, and safety data lineage. | `hr` |
| ESRS S2 | Workers in value chain. | Value-chain worker metric model records supplier boundary, due diligence source, impacts, remediation, and confidence level. | `supply-chain-planning` |
| ePrivacy Art. 5(3) | Storage or access on terminal equipment requires consent unless exemption applies. | SDK, cookie, local-storage, tracking pixel, and fingerprinting calls require consent or strict-necessity exemption. | `api-gateway` |
| EU Cyber Resilience Act | Products with digital elements require cybersecurity-by-design, technical documentation, conformity, vulnerability handling, and reporting. | Product security file, SBOM, support period, update mechanism, and vulnerability workflow are mandatory. | `marketplace` |
| EU Data Act | Users must access connected-product and related-service data; switching, interoperability, and non-personal data safeguards apply. | Access request workflow separates personal from non-personal data and records recipient, product, scope, and export controls. | `data-pipeline` |

## GDPR Coverage Details

| GDPR control id | Article | Pack implementation detail |
|---|---|---|
| `gdpr-principles-001` | 5 | Every processing activity has a controller owner, processor role, purpose, data class, and retention basis. |
| `gdpr-principles-002` | 5 | Accuracy remediation task exists for user-correctable profile data. |
| `gdpr-principles-003` | 5 | Storage limitation is expressed as deletion, anonymisation, archive, or legal-hold profile. |
| `gdpr-principles-004` | 5 | Accountability evidence binds each row to `audit_id`. |
| `gdpr-lawful-basis-001` | 6 | Contract basis is allowed only for service-essential processing. |
| `gdpr-lawful-basis-002` | 6 | Legitimate-interest basis requires assessment id and objection workflow. |
| `gdpr-lawful-basis-003` | 6 | Legal-obligation basis requires citation and jurisdiction. |
| `gdpr-lawful-basis-004` | 6 | Consent basis requires active consent artifact. |
| `gdpr-consent-001` | 7 | Consent withdrawal invalidates future Cedar decisions immediately. |
| `gdpr-consent-002` | 7 | Consent records cannot be bundled across unrelated purposes. |
| `gdpr-notice-001` | 13 | Notice must disclose recipients or recipient categories. |
| `gdpr-notice-002` | 13 | Notice must disclose transfers and safeguards when known. |
| `gdpr-erasure-001` | 17 | Erasure creates source-by-source execution evidence. |
| `gdpr-erasure-002` | 17 | Audit-chain evidence is tombstoned only by pointer where deletion would break Merkle integrity. |
| `gdpr-automated-001` | 22 | Significant-effect automated decisions require human intervention path. |
| `gdpr-design-001` | 25 | Data minimisation field is reviewed at API contract time. |
| `gdpr-processor-001` | 28 | Subprocessor addition triggers tenant notice workflow where required. |
| `gdpr-ropa-001` | 30 | ROPA export is machine-generated from canonical processing rows. |
| `gdpr-security-001` | 32 | Restore test evidence is required for regulated production systems. |
| `gdpr-breach-001` | 33 | Breach clock records discovery, confirmation, notification, and close time. |
| `gdpr-transfer-001` | 44 | Third-country recipient is denied by default. |
| `gdpr-transfer-002` | 46 | SCC module must match controller/processor roles. |
| `gdpr-penalty-001` | 83 | Fine-risk scoring informs severity but never replaces legal judgement. |

## DSA Coverage Details

| DSA control id | Article | Pack implementation detail |
|---|---|---|
| `dsa-order-001` | 9 | Orders are stored separately from ordinary user notices. |
| `dsa-order-002` | 9 | Response evidence includes action taken and authority notification. |
| `dsa-terms-001` | 14 | Terms restrictions are version-pinned to moderation policy. |
| `dsa-terms-002` | 14 | Automated moderation description is linked to AI disclosure where relevant. |
| `dsa-notice-001` | 16 | Notice intake supports trusted flagger status. |
| `dsa-notice-002` | 16 | Notice intake records whether content location is sufficiently precise. |
| `dsa-reason-001` | 17 | Statement of reasons distinguishes account, content, monetisation, and visibility restrictions. |
| `dsa-complaint-001` | 20 | Complaint workflow preserves reviewer independence from original reviewer where configured. |
| `dsa-minor-001` | 28 | Minor-protection gate denies sensitive-data ad targeting. |
| `dsa-risk-001` | 34 | Risk register stores residual risk and mitigation owner. |
| `dsa-recommender-001` | 38 | Recommender settings expose main parameters and non-profiling option when applicable. |
| `dsa-data-access-001` | 40 | Research access exports are pseudonymised unless lawful request requires more. |

## DMA Coverage Details

| DMA control id | Article | Pack implementation detail |
|---|---|---|
| `dma-obligation-001` | 6 | Core-platform service mapping drives per-obligation tasks. |
| `dma-interoperability-001` | 7 | Messaging interoperability requests include security and privacy impact review. |
| `dma-report-001` | 9 | Compliance report captures evidence source and unresolved gap list. |
| `dma-update-001` | 12 | Obligation updates require policy-fragment review. |
| `dma-anti-circumvention-001` | 13 | Release gate blocks features that indirectly re-create a prohibited restriction. |
| `dma-anti-circumvention-002` | 13 | Product experiment logs are sampled for obligation drift. |

## EU AI Act Coverage Details

| AI control id | Article | Pack implementation detail |
|---|---|---|
| `ai-classification-001` | 6 | System activation requires high-risk classifier output. |
| `ai-classification-002` | Annex III | Employment, education, essential services, and credit-adjacent uses receive explicit category tags. |
| `ai-transparency-001` | 13 | Instructions expose intended purpose and limitations. |
| `ai-oversight-001` | 14 | Human oversight plan names reviewer competence and authority. |
| `ai-provider-001` | 16 | Provider evidence includes technical documentation and logging design. |
| `ai-deployer-001` | 26 | Deployer evidence includes input-data relevance and monitoring plan. |
| `ai-conformity-001` | 43 | Conformity status is not inferred from tests; it must be recorded. |
| `ai-disclosure-001` | 50 | Chatbot disclosure is served before interaction. |
| `ai-disclosure-002` | 50 | Synthetic content disclosure is attached to generated media where required. |
| `ai-monitoring-001` | 72 | Post-market monitoring stores signal, threshold, owner, and action. |
| `ai-incident-001` | 73 | Serious incident reports carry authority destination and remediation status. |

## eIDAS Coverage Details

| eIDAS control id | Article | Pack implementation detail |
|---|---|---|
| `eidas-qualified-provider-001` | 24 | Qualified trust-service provider status is validated against certificate and supervisory evidence. |
| `eidas-signature-effect-001` | 25 | Workflow rules cannot downgrade an electronic signature solely for being electronic. |
| `eidas-advanced-signature-001` | 26 | Advanced signature verification checks signer linkage, sole control, and post-signature change detection. |
| `eidas-wallet-001` | 2024/1183 | Wallet attributes are treated as high-assurance identity evidence only when issuer and relying-party constraints pass. |
| `eidas-revocation-001` | 24 | Revocation status is checked at validation time and retained with timestamp. |

## NIS2 Coverage Details

| NIS2 control id | Article | Pack implementation detail |
|---|---|---|
| `nis2-strategy-001` | 7 | Member-state strategy overlays can add stricter controls. |
| `nis2-cvd-001` | 12 | Vulnerability disclosure cases track reporter, product, affected service, remediation, and disclosure state. |
| `nis2-cooperation-001` | 14 | Cooperation-group communications are stored as authority-contact evidence. |
| `nis2-governance-001` | 20 | Management approval is required for covered-entity cyber policy. |
| `nis2-risk-001` | 21 | Risk measures cover incident handling and business continuity. |
| `nis2-risk-002` | 21 | Supply-chain security is required for critical suppliers. |
| `nis2-risk-003` | 21 | Vulnerability handling is integrated with CRA product-security files where applicable. |
| `nis2-report-001` | 23 | Reporting workflow supports early warning. |
| `nis2-report-002` | 23 | Reporting workflow supports incident notification. |
| `nis2-report-003` | 23 | Reporting workflow supports final report. |

## DORA Coverage Details

| DORA control id | Article | Pack implementation detail |
|---|---|---|
| `dora-governance-001` | 5 | Management body accountability is required for ICT risk. |
| `dora-framework-001` | 6 | ICT risk management framework is a versioned artifact. |
| `dora-identification-001` | 8 | Critical or important functions are mapped to ICT assets and providers. |
| `dora-incident-001` | 17 | Incident process includes detection, classification, response, and recovery. |
| `dora-report-001` | 19 | Major incident reports include initial, intermediate, and final evidence. |
| `dora-tlpt-001` | resilience testing | Threat-led penetration testing is scheduled for in-scope financial entities. |
| `dora-third-party-001` | third-party risk | ICT third-party register records service, location, subcontractors, and exit strategy. |
| `dora-continuity-001` | framework | Backup and restore evidence is linked to critical functions. |
| `dora-lessons-001` | incident process | Post-incident lessons are mandatory before final closure. |

## CSRD and ESRS Coverage Details

| Sustainability control id | Source | Pack implementation detail |
|---|---|---|
| `csrd-scope-001` | CSRD | Reporting profile stores whether tenant is reporting undertaking, subsidiary, value-chain supplier, or out-of-scope supplier. |
| `csrd-boundary-001` | CSRD | Consolidation boundary is stored with period and approval evidence. |
| `csrd-assurance-001` | CSRD | Assurance status is versioned per metric. |
| `esrs-e1-001` | ESRS E1 | GHG metrics carry source, methodology, emission factor, and restatement state. |
| `esrs-e1-002` | ESRS E1 | Transition-plan evidence is linked to targets and actions. |
| `esrs-e1-003` | ESRS E1 | Energy consumption and mix are stored with location and period. |
| `esrs-s1-001` | ESRS S1 | Own workforce headcount metrics carry boundary and employment type. |
| `esrs-s1-002` | ESRS S1 | Health and safety metrics carry source and incident classification. |
| `esrs-s1-003` | ESRS S1 | Equal treatment metrics record protected-field minimisation and aggregation controls. |
| `esrs-s2-001` | ESRS S2 | Value-chain worker evidence records supplier, source confidence, and remediation state. |
| `esrs-s2-002` | ESRS S2 | Supplier data requests are logged with purpose and retention. |
| `esrs-s2-003` | ESRS S2 | Sensitive worker data is aggregated before cross-tenant reporting. |

## ePrivacy, CRA, and Data Act Coverage Details

| Control id | Source | Pack implementation detail |
|---|---|---|
| `eprivacy-terminal-001` | ePrivacy Art. 5(3) | Consent is required for non-essential cookies, SDK storage, local storage, pixels, and fingerprinting. |
| `eprivacy-terminal-002` | ePrivacy Art. 5(3) | Strictly necessary exemption must name service requested by the user. |
| `eprivacy-terminal-003` | ePrivacy Art. 5(3) | Withdrawal disables future terminal-equipment access. |
| `cra-product-001` | CRA | Product security file stores SBOM, support period, update mechanism, and vulnerability policy. |
| `cra-product-002` | CRA | Vulnerability intake records severity, exploit status, remediation, and notification state. |
| `cra-product-003` | CRA | Product listing gate blocks missing conformity or security metadata. |
| `cra-product-004` | CRA | Security update release evidence is retained. |
| `data-act-access-001` | Data Act | User access request records connected product, related service, requested data, and recipient. |
| `data-act-access-002` | Data Act | Export separates personal and non-personal data before delivery. |
| `data-act-switching-001` | Data Act | Cloud switching request records service category, exit timeline, and technical obstacles. |
| `data-act-safeguard-001` | Data Act | Non-personal third-country governmental access safeguards are recorded. |
| `data-act-contract-001` | Data Act | B2B data-sharing terms flag unfair term review. |

## Activated Cedar Policies

| Policy | Regulations covered | Default effect |
|---|---|---|
| `pack-eu-privacy-core` | GDPR Articles 5, 6, 7, 13, 17, 22, 25, 28, 30, 32, 33, 44, 46, 83 | Deny processing, transfer, or closure missing required privacy evidence. |
| `pack-eu-platform-accountability` | DSA Articles 9, 14, 16, 17, 20, 28, 34, 38, 40 | Deny moderation, recommender, notice, and researcher exports missing evidence. |
| `pack-eu-gatekeeper` | DMA Articles 6, 7, 9, 12, 13 | Deny gatekeeper workflow release missing obligation mapping. |
| `pack-eu-ai-governance` | AI Act Articles 6, 13, 14, 16, 26, 43, 50, 72, 73, Annex III | Deny AI activation missing classification, disclosure, conformity, or monitoring. |
| `pack-eu-trust-services` | eIDAS Articles 24, 25, 26 | Deny legal workflow finalization when trust artifact validation fails. |
| `pack-eu-cyber-baseline` | NIS2 Articles 7, 12, 14, 20, 21, 23 | Deny covered-entity activation missing governance or incident-reporting workflow. |
| `pack-eu-financial-resilience` | DORA Articles 5, 6, 8, 17, 19 | Deny financial-sector production path missing ICT framework or provider register. |
| `pack-eu-sustainability-lineage` | CSRD and ESRS E1/S1/S2 | Deny assurance-ready status when source lineage is missing. |
| `pack-eu-terminal-equipment` | ePrivacy Article 5(3) | Deny terminal access without consent or exemption. |
| `pack-eu-product-data-security` | CRA and Data Act | Deny product listing or data export missing security/access evidence. |

## Data Model Deltas

| Entity | Required fields |
|---|---|
| `EuRegulatoryApplicabilityProfile` | `tenant_id`, `pack_id`, `gdpr_scope`, `dsa_provider_class`, `dma_gatekeeper_status`, `ai_act_scope`, `eidas_scope`, `nis2_entity_class`, `dora_entity_class`, `csrd_scope`, `cra_product_scope`, `data_act_role`, `reviewed_at`, `reviewer_id`. |
| `EuRegulatoryControl` | `control_id`, `regulation`, `article_ref`, `policy_ref`, `data_model_ref`, `api_ref`, `audit_event_ref`, `evidence_owner`, `status`. |
| `EuTransferPathway` | `pathway_type`, `adequacy_country`, `scc_module`, `bcr_ref`, `derogation_ref`, `tia_ref`, `supplementary_measure_ref`, `recipient_role`. |
| `EuPlatformModerationControl` | `dsa_article`, `provider_class`, `notice_channel`, `decision_type`, `complaint_state`, `statement_ref`, `transparency_report_ref`. |
| `EuAiActControl` | `risk_tier`, `annex_iii_category`, `provider_obligation_ref`, `deployer_obligation_ref`, `conformity_status`, `transparency_ref`, `monitoring_ref`. |
| `EuCyberResilienceControl` | `nis2_scope`, `dora_scope`, `cra_scope`, `incident_stage`, `asset_ref`, `provider_ref`, `vulnerability_ref`, `report_ref`. |
| `EuSustainabilityDisclosureControl` | `csrd_scope`, `esrs_standard`, `metric_id`, `source_system`, `assurance_status`, `restatement_ref`, `value_chain_ref`. |
| `EuTerminalAccessControl` | `surface`, `storage_type`, `purpose`, `consent_ref`, `exemption_ref`, `sdk_ref`, `withdrawal_state`. |
| `EuDataActControl` | `connected_product_id`, `related_service_id`, `user_id`, `recipient_id`, `data_category`, `personal_data_flag`, `export_ref`. |

## API Contract Deltas

| Endpoint | Required EU fields |
|---|---|
| `POST /v1/eu/applicability/assess` | `tenant_id`, `service_surfaces`, `sectors`, `ai_systems`, `products`, `data_transfers`, `reporting_role`. |
| `GET /v1/eu/coverage/matrix` | `pack_id`, `version`, `regulation_filter`, `article_filter`, `status_filter`. |
| `POST /v1/eu/controls/{control_id}/evidence` | `evidence_ref`, `owner`, `valid_from`, `valid_until`, `source_system`, `audit_id`. |
| `POST /v1/eu/gdpr/processing-activities` | `lawful_basis`, `purpose_id`, `notice_ref`, `retention_ref`, `transfer_pathway_ref`, `ropa_owner`. |
| `POST /v1/eu/dsa/moderation-cases` | `provider_class`, `article_ref`, `content_ref`, `notice_ref`, `decision_ref`, `appeal_ref`. |
| `POST /v1/eu/dma/gatekeeper-obligations` | `core_platform_service`, `designation_ref`, `obligation_ref`, `interoperability_scope`, `reporting_ref`. |
| `POST /v1/eu/ai/systems` | `risk_tier`, `annex_iii_category`, `provider_ref`, `deployer_ref`, `conformity_ref`, `monitoring_ref`. |
| `POST /v1/eu/eidas/artifacts` | `trust_service_type`, `qualified_status`, `certificate_ref`, `revocation_state`, `validation_time`. |
| `POST /v1/eu/nis2/entity-profile` | `entity_class`, `sector`, `authority`, `management_approval_ref`, `risk_measures_ref`. |
| `POST /v1/eu/dora/financial-profile` | `financial_entity_type`, `critical_functions`, `ict_risk_framework_ref`, `third_party_register_ref`. |
| `POST /v1/eu/csrd/metric-lineage` | `esrs_standard`, `metric_id`, `source_system`, `period`, `boundary`, `assurance_status`. |
| `POST /v1/eu/eprivacy/terminal-access` | `surface`, `purpose`, `storage_type`, `consent_ref`, `exemption_ref`. |
| `POST /v1/eu/cra/product-security-file` | `product_id`, `sbom_ref`, `support_period`, `update_mechanism`, `vulnerability_contact`, `conformity_ref`. |
| `POST /v1/eu/data-act/access-request` | `product_id`, `user_id`, `recipient_id`, `data_categories`, `personal_data_flag`, `export_format`. |

## Audit Event Additions (per ADR-0263)

| Event class | Required payload fields |
|---|---|
| `EuRegulatoryApplicabilityAssessed` | `tenant_id`, `pack_id`, `regulation_scope`, `sector_scope`, `reviewer_id`, `assessed_at`. |
| `EuRegulatoryControlEvidenceAttached` | `tenant_id`, `control_id`, `evidence_ref`, `owner`, `valid_until`, `audit_id`. |
| `EuRegulatoryControlExpired` | `tenant_id`, `control_id`, `expired_at`, `blocking_level`, `owner`. |
| `GdprLawfulBasisRecorded` | `tenant_id`, `processing_activity_id`, `lawful_basis`, `purpose_id`, `notice_ref`. |
| `GdprTransferBlocked` | `tenant_id`, `transfer_id`, `recipient_country`, `missing_pathway`, `tia_state`. |
| `DsaNoticeReceived` | `tenant_id`, `notice_id`, `provider_class`, `article_ref`, `notifier_type`. |
| `DsaModerationDecisionIssued` | `tenant_id`, `case_id`, `restriction_type`, `statement_ref`, `appeal_ref`. |
| `DmaComplianceEvidenceUpdated` | `tenant_id`, `core_platform_service`, `obligation_ref`, `report_ref`, `status`. |
| `EuAiSystemClassified` | `tenant_id`, `ai_system_id`, `risk_tier`, `annex_iii_category`, `classifier_version`. |
| `EuAiConformityGateDenied` | `tenant_id`, `ai_system_id`, `missing_ref`, `risk_tier`, `deployment_ref`. |
| `EidasTrustArtifactVerified` | `tenant_id`, `artifact_id`, `trust_service_type`, `qualified_status`, `validation_result`. |
| `Nis2EntityProfileApproved` | `tenant_id`, `entity_class`, `authority`, `approval_ref`, `approved_at`. |
| `Nis2IncidentReportSubmitted` | `tenant_id`, `incident_id`, `stage`, `authority`, `submitted_at`. |
| `DoraIctRiskFrameworkApproved` | `tenant_id`, `framework_id`, `financial_entity_type`, `approver_id`, `valid_until`. |
| `DoraMajorIncidentReportSubmitted` | `tenant_id`, `incident_id`, `report_stage`, `authority`, `classification_ref`. |
| `CsrdMetricLineageSealed` | `tenant_id`, `metric_id`, `esrs_standard`, `source_system`, `assurance_status`. |
| `EprivacyTerminalAccessDenied` | `tenant_id`, `surface`, `purpose`, `storage_type`, `reason`. |
| `CraProductSecurityFileApproved` | `tenant_id`, `product_id`, `sbom_ref`, `support_period`, `conformity_ref`. |
| `DataActAccessRequestClassified` | `tenant_id`, `request_id`, `product_id`, `personal_data_flag`, `recipient_id`. |

## Failure Modes specific to EU enforcement

| Failure | Impact | Required response |
|---|---|---|
| Applicability profile is stale after tenant adds marketplace features. | DSA duties can be missed. | Re-run applicability assessment and block marketplace release. |
| AI feature is launched from a generic feature flag. | AI Act classification can be bypassed. | Require AI system registry row before feature flag promotion. |
| Processor is added through procurement but not compliance. | GDPR Article 28 record is incomplete. | Block data access until DPA and subprocessors are recorded. |
| Data export treats mixed personal and non-personal data as Data Act only. | GDPR controls can be bypassed. | Split export and apply GDPR first. |
| Financial tenant uses generic incident workflow. | DORA report sequence can be missed. | Route to DORA workflow when financial profile is active. |
| Essential entity omits management approval. | NIS2 governance evidence gap. | Block NIS2 profile activation. |
| VLOP status changes but recommender controls remain generic. | DSA systemic risk duties are missed. | Force DSA provider-class reclassification. |
| Signature validation ignores revocation state. | eIDAS trust artifact is unreliable. | Refuse workflow finalization. |
| Sustainability metric is manually overwritten. | CSRD assurance lineage is broken. | Seal restatement event and require reviewer approval. |
| Product SBOM is absent from marketplace listing. | CRA product security evidence gap. | Block listing or activation. |
| Cookie SDK is added by marketing without consent registration. | ePrivacy Article 5(3) exposure. | Gateway denies terminal-equipment access. |
| Transfer uses SCCs without recipient-law review. | Schrems II supplementary-measure gap. | Require TIA and supplementary measure evidence. |

## Worked Examples

### Example A: Combined GDPR and Data Act export

A connected-equipment tenant receives a request for usage data.
The request includes device telemetry, user account data, and service logs.
`DataActAccessRequest` records the connected product and requested categories.
`policy-engine` classifies user account data as personal.
GDPR Article 15 and portability rules apply to the personal-data subset.
The Data Act access workflow applies to non-personal product telemetry.
If the recipient is outside the EEA, GDPR Chapter V controls the personal subset.
The export manifest contains separate checksums for personal and non-personal data.
Audit-chain seals `DataActAccessRequestClassified` and `GdprCrossBorderTransferEvaluated`.

### Example B: DSA content notice and AI-generated media

A user reports synthetic video as illegal content.
`social` opens a DSA Article 16 notice.
`intelligence` confirms the media has AI-generated content provenance.
Article 50 disclosure evidence is checked.
The moderator removes visibility and issues a statement of reasons.
The user receives complaint-handling instructions.
Audit-chain seals `DsaNoticeReceived`, `DsaModerationDecisionIssued`, and `EuAiTransparencyDisclosureServed`.

### Example C: DORA and NIS2 overlap

A financial tenant reports a material outage.
The tenant is both a financial entity and an important entity.
DORA is treated as sector-specific for ICT incident reporting.
NIS2 risk-management controls remain visible as background controls.
The report workflow uses DORA incident stages.
The NIS2 profile records the incident as covered by DORA reporting.
Audit-chain seals `DoraMajorIncidentReportSubmitted`.

### Example D: AI Act high-risk employment system

An HR tenant enables automated candidate screening.
`AiSystemRegistry` records Annex III employment category.
`policy-engine` denies launch until Article 13 instructions and Article 43 conformity status are present.
Article 26 deployer obligations are assigned to the tenant administrator.
Article 14 human oversight is assigned to HR reviewers.
Article 72 monitoring starts at launch.
Article 73 serious incident workflow is prewired.
Audit-chain seals `EuAiSystemClassified` and `EuAiHighRiskConformityRecorded`.

### Example E: CSRD supplier evidence request

A customer asks a supplier tenant for ESRS S2 value-chain worker information.
The supplier is not itself an in-scope reporting undertaking.
EU-PACK-1 still records the request as value-chain evidence.
The metric line captures supplier boundary, source system, aggregation level, and confidence.
Sensitive workforce data is aggregated before export.
Audit-chain seals `CsrdMetricLineageSealed`.

## Cross-References

| Document | Use |
|---|---|
| `packs/eu-localization/README.md` | Pack overview, precedence, microservice activation. |
| `packs/eu-localization/data-residency-and-cross-border.md` | GDPR Chapter V, SCC, adequacy, Schrems II detail. |
| `packs/eu-localization/dsr-and-portability.md` | GDPR Articles 15-22 workflow detail. |
| `packs/eu-localization/high-risk-ai-systems.md` | AI Act Article 6, 13, 14, 16, 26, 43, 50, 72, 73, Annex III detail. |
| `packs/eu-localization/dora-operational-resilience.md` | DORA Article 5, 6, 8, 17, 19 and TLPT detail. |
| `docs/decisions/ADR-0700-ci-admission-live-apex.md` | Cedar policy layering. |
| `docs/decisions/ADR-0708-platform-foundations-live-apex.md` | Compliance pack model. |
| `docs/decisions/ADR-0706-observability-live-apex.md` | Audit-event requirements. |
| `docs/decisions/ADR-0709-general-live-apex.md` | Conflict handling. |
| `docs/decisions/ADR-0709-general-live-apex.md` | AI Act lifecycle. |

## Control Status Vocabulary

`not_applicable` means the tenant or sector profile does not trigger the control.
`unassessed` means applicability has not been reviewed.
`evidence_missing` means the control applies but no evidence exists.
`evidence_stale` means the evidence exists but the validity window has expired.
`blocked` means Cedar must deny the regulated action.
`warning` means the action may proceed but dashboard and audit evidence must show risk.
`active` means the control is enforceable and evidence is current.
`manual_review` means the action requires human legal, security, compliance, or product review.
`retired` means a control was superseded by a newer pack version and kept for historical traceability.

## Matrix Maintenance Rules

Every new row must name an official authority.
Every new row must name an Oyatie control.
Every new row must name an evidence owner.
Every new row must identify whether GDPR overrides or coexists.
Every new row that emits evidence must map to ADR-0263.
Every new row that blocks runtime action must map to a Cedar policy.
Every new row that changes API shape must update API contract deltas.
Every new row that changes data shape must update data model deltas.
Every new row that touches AI systems must update the AI registry mapping.
Every new row that touches financial entities must update the DORA profile mapping.
Every new row that touches connected products must update CRA/Data Act mapping.
Every new row that touches sustainability must update ESRS metric lineage.
Every new row must remain tenant-scoped.
Every new row must preserve member-state overlay extensibility.
Every new row must preserve pack-level precedence.

## Per-Regulation Enforcement Checklist

| Checklist id | Regulation | Enforcement check |
|---|---|---|
| `check-gdpr-001` | GDPR | Every personal-data processing path has a `ProcessingActivity` row. |
| `check-gdpr-002` | GDPR | Every `ProcessingActivity` row has one Article 6 lawful basis. |
| `check-gdpr-003` | GDPR | Consent basis points to an active consent artifact. |
| `check-gdpr-004` | GDPR | Notice basis points to an Article 13 notice version. |
| `check-gdpr-005` | GDPR | Retention policy has deletion, anonymisation, archive, or legal-hold outcome. |
| `check-gdpr-006` | GDPR | Article 17 erasure workflow covers every indexed source system. |
| `check-gdpr-007` | GDPR | Article 22 significant-effect classification exists for automated decision paths. |
| `check-gdpr-008` | GDPR | Article 25 privacy defaults are recorded for new data fields. |
| `check-gdpr-009` | GDPR | Processor role and DPA are recorded before processor data access. |
| `check-gdpr-010` | GDPR | Article 30 ROPA export is complete for active processing rows. |
| `check-gdpr-011` | GDPR | Article 32 security controls are linked to restore-test evidence. |
| `check-gdpr-012` | GDPR | Article 33 breach workflow has 72-hour clock fields. |
| `check-gdpr-013` | GDPR | Chapter V transfer pathway is recorded for every third-country recipient. |
| `check-gdpr-014` | GDPR | SCC transfer has module, recipient role, TIA, and supplementary measures. |
| `check-gdpr-015` | GDPR | Article 83 risk score never closes an issue without evidence owner approval. |
| `check-dsa-001` | DSA | Provider class is set for every EU-facing digital-service surface. |
| `check-dsa-002` | DSA | Authority orders use the Article 9 intake path, not ordinary notice path. |
| `check-dsa-003` | DSA | Terms version is linked to moderation policy. |
| `check-dsa-004` | DSA | Notice-action intake exposes user and trusted-flagger fields. |
| `check-dsa-005` | DSA | Moderation restriction cannot publish without statement of reasons. |
| `check-dsa-006` | DSA | Internal complaint path exists for appealable decisions. |
| `check-dsa-007` | DSA | Minor-protection gate blocks targeted-ad and unsafe recommender paths. |
| `check-dsa-008` | DSA | VLOP/VLOSE profile triggers systemic-risk register. |
| `check-dsa-009` | DSA | Recommender transparency settings are exposed where required. |
| `check-dsa-010` | DSA | Vetted researcher access is separated from ordinary exports. |
| `check-dma-001` | DMA | Gatekeeper designation status is recorded as designated, not-designated, monitoring, or unknown. |
| `check-dma-002` | DMA | Core-platform service is mapped before DMA controls activate. |
| `check-dma-003` | DMA | Interoperability requests have privacy and security review. |
| `check-dma-004` | DMA | Compliance reports include evidence and unresolved gaps. |
| `check-dma-005` | DMA | Anti-circumvention checks run on release candidates. |
| `check-ai-001` | AI Act | Every AI system has risk classification before preview. |
| `check-ai-002` | AI Act | Annex III category is set for high-risk candidates. |
| `check-ai-003` | AI Act | Article 13 instructions exist for high-risk systems. |
| `check-ai-004` | AI Act | Article 14 human oversight plan exists for high-risk systems. |
| `check-ai-005` | AI Act | Article 16 provider obligations are assigned when Oyatie is provider. |
| `check-ai-006` | AI Act | Article 26 deployer obligations are assigned when tenant deploys. |
| `check-ai-007` | AI Act | Article 43 conformity status exists before high-risk production. |
| `check-ai-008` | AI Act | Article 50 disclosure is served for relevant AI interactions. |
| `check-ai-009` | AI Act | Article 72 post-market monitoring plan has signal owners. |
| `check-ai-010` | AI Act | Article 73 serious incident workflow is wired. |
| `check-eidas-001` | eIDAS | Trust artifact stores certificate chain and revocation state. |
| `check-eidas-002` | eIDAS | Qualified-provider status is verified at validation time. |
| `check-eidas-003` | eIDAS | Workflow cannot reject a signature solely because it is electronic. |
| `check-eidas-004` | eIDAS | Advanced signature checks unique link, signer identification, sole control, and change detection. |
| `check-nis2-001` | NIS2 | Entity profile distinguishes essential, important, supplier, and out-of-scope. |
| `check-nis2-002` | NIS2 | Member-state authority is stored where known. |
| `check-nis2-003` | NIS2 | Management approval is attached to cyber risk policy. |
| `check-nis2-004` | NIS2 | Article 21 risk measures cover supply chain and vulnerability handling. |
| `check-nis2-005` | NIS2 | Incident workflow supports early warning, notification, intermediate, and final report. |
| `check-nis2-006` | NIS2 | Coordinated vulnerability disclosure does not expose reporter identity in ordinary logs. |
| `check-dora-001` | DORA | Financial entity profile is set before DORA controls apply. |
| `check-dora-002` | DORA | ICT risk framework is approved and current. |
| `check-dora-003` | DORA | Critical and important functions map to ICT assets. |
| `check-dora-004` | DORA | ICT third-party register exists for critical dependencies. |
| `check-dora-005` | DORA | Major incident classification has threshold evidence. |
| `check-dora-006` | DORA | Threat-led penetration test schedule exists where applicable. |
| `check-csrd-001` | CSRD | Reporting profile distinguishes direct reporter and value-chain supplier. |
| `check-csrd-002` | CSRD | ESRS E1 metrics have source and methodology. |
| `check-csrd-003` | CSRD | ESRS S1 metrics are aggregated before protected-field reporting. |
| `check-csrd-004` | CSRD | ESRS S2 supplier evidence carries confidence and remediation state. |
| `check-csrd-005` | CSRD | Restatements are append-only and never overwrite prior reported values. |
| `check-eprivacy-001` | ePrivacy | Terminal-equipment access inventory includes cookies, SDKs, pixels, local storage, and fingerprinting. |
| `check-eprivacy-002` | ePrivacy | Strictly necessary exemption names user-requested service. |
| `check-eprivacy-003` | ePrivacy | Consent withdrawal disables future non-essential access. |
| `check-cra-001` | Cyber Resilience Act | Product with digital elements has product security file. |
| `check-cra-002` | Cyber Resilience Act | SBOM and support period are recorded. |
| `check-cra-003` | Cyber Resilience Act | Vulnerability contact and vulnerability handling workflow exist. |
| `check-cra-004` | Cyber Resilience Act | Security update evidence is retained. |
| `check-data-act-001` | Data Act | Connected-product and related-service role is recorded. |
| `check-data-act-002` | Data Act | Access request separates personal and non-personal data. |
| `check-data-act-003` | Data Act | Recipient authorization is verified before export. |
| `check-data-act-004` | Data Act | Switching request records timeline and technical obstacles. |
| `check-data-act-005` | Data Act | Non-personal third-country access safeguards are recorded. |

## Regulator Evidence Packages

| Package id | Contents | Normal owner |
|---|---|---|
| `pkg-gdpr-supervisory-inquiry` | Applicability profile, ROPA rows, lawful basis, notices, DPA records, security controls, breach logs, transfer pathway evidence. | `compliance` |
| `pkg-gdpr-breach-notification` | Breach classification, discovery timeline, categories of data and subjects, likely consequences, mitigation, contact point, notice copy. | `incident-management` |
| `pkg-gdpr-transfer-review` | Recipient, role, country, transfer pathway, SCC module, TIA, supplementary measures, residual risk, approval. | `compliance` |
| `pkg-dsa-order-response` | Authority order, authenticity checks, affected content, action taken, response sent, statement of reasons when applicable. | `social` |
| `pkg-dsa-transparency-report` | Notices, orders, moderation decisions, complaints, automated moderation, recommender parameters, minor-protection metrics. | `analytics` |
| `pkg-dsa-vetted-researcher-access` | Request, researcher status, legal basis, dataset scope, minimisation, access control, audit trail. | `data-pipeline` |
| `pkg-dma-compliance` | Gatekeeper status, core-platform service map, obligations, technical measures, interoperability evidence, anti-circumvention review. | `compliance` |
| `pkg-ai-high-risk-file` | Classification, Annex III category, intended purpose, instructions, risk management, data governance, logs, human oversight, conformity status. | `intelligence` |
| `pkg-ai-serious-incident` | Incident classification, system id, affected users, malfunction, harm, mitigation, authority notice, corrective action. | `incident-management` |
| `pkg-eidas-signature-validation` | Trust artifact, certificate chain, qualified status, revocation check, timestamp, validation result. | `identity` |
| `pkg-nis2-incident` | Entity profile, incident type, early warning, incident notification, intermediate updates, final report, remediation. | `incident-management` |
| `pkg-nis2-risk-measures` | Management approval, policies, asset inventory, supplier controls, vulnerability handling, training, business continuity. | `governance` |
| `pkg-dora-ict-risk` | Financial entity profile, critical functions, ICT framework, asset map, provider register, testing, continuity. | `finops-portal` |
| `pkg-dora-major-incident` | Classification, materiality threshold, impact, reports, communications, root cause, remediation, lessons learned. | `incident-management` |
| `pkg-dora-tlpt` | Scope, threat intelligence, test plan, provider, findings, remediation, retest, management signoff. | `detection` |
| `pkg-csrd-esrs` | Reporting scope, metric lineage, methodology, source system, controls, restatements, assurance status. | `analytics` |
| `pkg-eprivacy-consent` | Terminal-equipment inventory, consent strings, exemptions, withdrawal logs, SDK versions, suppression evidence. | `consent-graph` |
| `pkg-cra-product-security` | Product file, SBOM, support period, secure development evidence, vulnerability intake, update release, conformity status. | `marketplace` |
| `pkg-data-act-access` | Request, product, data categories, recipient, personal-data split, export manifest, refusal or fulfilment reason. | `data-pipeline` |

## Negative Fixture Expectations

| Fixture id | Input | Expected denial |
|---|---|---|
| `neg-gdpr-no-lawful-basis` | Processing activity has purpose but no Article 6 basis. | `GDPR_LAWFUL_BASIS_REQUIRED`. |
| `neg-gdpr-no-notice` | Collection endpoint omits notice version. | `GDPR_NOTICE_VERSION_REQUIRED`. |
| `neg-gdpr-transfer-no-pathway` | Export to third country omits adequacy, SCC, BCR, or derogation. | `GDPR_TRANSFER_PATHWAY_REQUIRED`. |
| `neg-dsa-no-statement` | Moderation restriction has no statement of reasons. | `DSA_STATEMENT_OF_REASONS_REQUIRED`. |
| `neg-dsa-minor-targeting` | Ad segment includes protected minor flag. | `DSA_MINOR_TARGETING_FORBIDDEN`. |
| `neg-dma-no-core-service` | DMA obligation is created without core-platform service. | `DMA_INTEROPERABILITY_SCOPE_REQUIRED`. |
| `neg-ai-unclassified` | AI system is launched without risk tier. | `EU_AI_CLASSIFICATION_REQUIRED`. |
| `neg-ai-no-conformity` | Annex III high-risk system lacks conformity status. | `EU_AI_CONFORMITY_REQUIRED`. |
| `neg-eidas-invalid-revocation` | Trust artifact has revoked certificate. | `EIDAS_TRUST_ARTIFACT_INVALID`. |
| `neg-nis2-no-profile` | Covered entity workflow starts without entity profile. | `NIS2_ENTITY_PROFILE_REQUIRED`. |
| `neg-dora-no-third-party-register` | Critical ICT provider is referenced without register row. | `DORA_ICT_PROVIDER_REGISTER_REQUIRED`. |
| `neg-csrd-no-lineage` | ESRS metric is marked assurance-ready without source system. | `CSRD_METRIC_LINEAGE_REQUIRED`. |
| `neg-eprivacy-no-consent` | Tracking SDK accesses device storage without consent. | `EPRIVACY_TERMINAL_CONSENT_REQUIRED`. |
| `neg-cra-no-product-file` | Product listing lacks SBOM and support period. | `CRA_PRODUCT_SECURITY_FILE_REQUIRED`. |
| `neg-data-act-no-scope` | Access request omits product and related-service id. | `DATA_ACT_ACCESS_SCOPE_REQUIRED`. |

## Coverage Checkpoint

Checkpoint id: `eu-regulatory-coverage-matrix`.
Checkpoint owner: `codex-eu-localization-pack-w1`.
Checkpoint scope: this document only.
Checkpoint confirms GDPR Article 5 coverage.
Checkpoint confirms GDPR Article 6 coverage.
Checkpoint confirms GDPR Article 7 coverage.
Checkpoint confirms GDPR Article 13 coverage.
Checkpoint confirms GDPR Article 17 coverage.
Checkpoint confirms GDPR Article 22 coverage.
Checkpoint confirms GDPR Articles 25, 28, 30, 32, 33, 44, 46, and 83 coverage.
Checkpoint confirms DSA Articles 9, 14, 16, 17, 20, 28, 34, 38, and 40 coverage.
Checkpoint confirms DMA Articles 6, 7, 9, 12, and 13 coverage.
Checkpoint confirms EU AI Act Articles 6, 13, 14, 16, 26, 43, 50, 72, 73, and Annex III coverage.
Checkpoint confirms eIDAS Articles 24, 25, and 26 coverage.
Checkpoint confirms NIS2 Articles 7, 12, 14, 20, 21, and 23 coverage.
Checkpoint confirms DORA Articles 5, 6, 8, 17, and 19 coverage.
Checkpoint confirms CSRD, ESRS E1, ESRS S1, ESRS S2, ePrivacy Article 5(3), EU Cyber Resilience Act, and EU Data Act coverage.
Checkpoint evidence target: `eu_pack_docs:6`.
