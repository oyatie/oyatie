# Compliance feature parity matrix against Vanta, Drata, and OneTrust

Audit date: 2026-05-20.
Target µservice: `microservices/compliance/`.
Counterpart bar: union coverage across Vanta, Drata, and OneTrust.
Method: current public official product/help pages plus service-local artifacts.
Verdict: partial parity; Oyatie is strong on compliance-pack enforcement and weak on buyer trust, questionnaire automation, third-party risk, consent, and broad risk operations.

## Five-citation anchor block

1. Canonical direction: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` §D-15..§D-20, especially §D-20.147..§D-20.151 for industry-counterpart parity and §D-20.219..§D-20.224 for counterpart selection discipline.
2. Machine-readable control surface: `specs/master-plan-sequencing.json` lines 704-868 for deployment contexts, OpenTofu, OS, language, and OCI Always Free constraints that any parity implementation must preserve.
3. µservice PRD: `microservices/compliance/PRD.md` lines 16-20, 24-29, 31-36, 48-59, 60-80, and 82-98.
4. µservice architecture/tier evidence: `microservices/compliance/ARCHITECTURE.md` lines 34-220 and `microservices/compliance/capability-tiers/tier-matrix.md` lines 11-184.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md` lines 62-81 and 133-173 for required service artifacts and intern-buildability.

## Public source anchors

WEB-VANTA-PLATFORM: https://www.vanta.com/ lines 196-200, retrieved 2026-05-21.
WEB-VANTA-TRUST: https://www.vanta.com/products/trust-center lines 167-199 and 328-367, retrieved 2026-05-21.
WEB-VANTA-QUESTIONNAIRE: https://help.vanta.com/en/articles/11345464-getting-started-with-questionnaire-automation lines 33-41, retrieved 2026-05-21.
WEB-VANTA-INTEGRATIONS: https://help.vanta.com/en/articles/11346110-maximizing-your-vanta-roi-with-integrations lines 31-36, 93-102, 229-263, retrieved 2026-05-21.
WEB-VANTA-TPRM: https://www.vanta.com/products/third-party-risk-management lines 336-348, retrieved 2026-05-21.
WEB-VANTA-ACCESS: https://help.vanta.com/en/articles/11345416-access-reviews lines 62-76 and 124-132, retrieved 2026-05-21.
WEB-DRATA-COMPLIANCE: https://drata.com/products/compliance lines 218-224, retrieved 2026-05-21.
WEB-DRATA-RISK: https://drata.com/products/risk lines 34-38 and 118-150 from opened pages, retrieved 2026-05-21.
WEB-DRATA-TPRM: https://drata.com/products/third-party-risk-management lines 118-150 and 174-184, retrieved 2026-05-21.
WEB-DRATA-TRUST: https://drata.com/products/assurance/customer-trust-portal lines 34-66 and 75-116, retrieved 2026-05-21.
WEB-ONETRUST-PRODUCTS: https://www.onetrust.com/products/ lines 257-356, retrieved 2026-05-21.
WEB-ONETRUST-DSR: https://www.onetrust.com/products/data-subject-request-dsr-automation/ lines 237-244 and 262-272, retrieved 2026-05-21.
WEB-ONETRUST-TPRM: https://www.onetrust.com/products/third-party-risk-management/ lines 213-254, retrieved 2026-05-21.
WEB-ONETRUST-AI: https://www.onetrust.com/solutions/ai-governance/ lines 251-259 and 420-435, retrieved 2026-05-21.
WEB-ONETRUST-THIRD-PARTY: https://www.onetrust.com/solutions/third-party-management/ lines 217-240, 250-273, and 301-340, retrieved 2026-05-21.

## §1 Counterpart 1 — Vanta capability surface

VANTA-01 Compliance automation with continuous monitoring and automated evidence collection; source WEB-VANTA-PLATFORM lines 196-200 and WEB-VANTA-INTEGRATIONS lines 31-36.
VANTA-02 Control monitoring that updates posture in real time; source WEB-VANTA-INTEGRATIONS lines 31-36.
VANTA-03 Evidence mapping back to frameworks; source WEB-VANTA-INTEGRATIONS lines 31-35.
VANTA-04 Compliance framework scoping and asset inclusion/exclusion; source WEB-VANTA-INTEGRATIONS lines 93-98.
VANTA-05 Cloud metadata/configuration verification with no content access; source WEB-VANTA-INTEGRATIONS lines 99-102.
VANTA-06 Identity provider integration for account/access traceability; source WEB-VANTA-INTEGRATIONS lines 108-116.
VANTA-07 HRIS integration for start dates, termination dates, training, background checks, and offboarding evidence; source WEB-VANTA-INTEGRATIONS lines 229-238.
VANTA-08 Vulnerability scanner expansion as later automation phase; source WEB-VANTA-INTEGRATIONS lines 251-263.
VANTA-09 Access reviews with review creation and scheduling; source WEB-VANTA-ACCESS lines 62-76.
VANTA-10 Access review import for unintegrated systems; source WEB-VANTA-ACCESS lines 124-132.
VANTA-11 Access review AI image parser for screenshots/PDFs; source WEB-VANTA-ACCESS lines 128-130.
VANTA-12 Trust Center as public/customer assurance surface; source WEB-VANTA-PLATFORM lines 196-200.
VANTA-13 Trust Center AI buyer question answering; source WEB-VANTA-TRUST lines 167-169.
VANTA-14 Trust Center automated document access approvals; source WEB-VANTA-TRUST lines 174-180.
VANTA-15 Trust Center automated NDA collection; source WEB-VANTA-TRUST lines 178-186.
VANTA-16 CRM and contract-management integrations for gated document access; source WEB-VANTA-TRUST lines 174-176.
VANTA-17 Tailored Trust Center views with tags and filters; source WEB-VANTA-TRUST lines 190-199.
VANTA-18 Continuous controls/test monitoring surfaced externally; source WEB-VANTA-TRUST lines 197-199 and 340-345.
VANTA-19 Trust Center engagement analytics; source WEB-VANTA-TRUST lines 351-363.
VANTA-20 Salesforce/HubSpot access approval and revenue-reporting integrations; source WEB-VANTA-TRUST lines 363-367.
VANTA-21 Questionnaire automation with AI-powered answering; source WEB-VANTA-QUESTIONNAIRE lines 33-41.
VANTA-22 Past questionnaire import to build answer library; source WEB-VANTA-QUESTIONNAIRE lines 37-39.
VANTA-23 Security documentation and policy upload for answer context; source WEB-VANTA-QUESTIONNAIRE lines 37-41.
VANTA-24 Policy sync from compliance into questionnaires; source WEB-VANTA-QUESTIONNAIRE lines 39-41.
VANTA-25 Cross-functional questionnaire collaboration; source WEB-VANTA-QUESTIONNAIRE lines 33-41 and Vanta help results.
VANTA-26 Third-party risk management standalone or add-on; source WEB-VANTA-TPRM lines 336-344.
VANTA-27 AI-powered third-party assessments; source WEB-VANTA-TPRM lines 342-348.
VANTA-28 Continuous third-party monitoring; source WEB-VANTA-TPRM lines 342-348.
VANTA-29 Vendor evidence request and reminders; source Vanta TPRM search snippet and WEB-VANTA-TPRM.
VANTA-30 Vendor questionnaire templates and custom questionnaires; source Vanta TPRM public page and help page.
VANTA-31 Customized inherent and residual risk rubrics; source WEB-VANTA-TPRM lines 336-348.
VANTA-32 Automatic vendor discovery for shadow IT; source Vanta TPRM public page.
VANTA-33 Procurement integration for vendor intake; source Vanta TPRM public page.
VANTA-34 AI review of SOC reports, DPAs, questionnaires, and live Trust Center evidence; source Vanta TPRM public page.
VANTA-35 Unified GRC product framing across compliance, risk, proof, audit, trust, questionnaire, and third-party risk; source WEB-VANTA-PLATFORM lines 196-200.

## §2 Counterpart 2 — Drata capability surface

DRATA-01 Enterprise GRC centralizing governance, controls, risks, policies, and evidence; source WEB-DRATA-COMPLIANCE lines 218-224.
DRATA-02 Compliance automation for evidence collection and control monitoring; source WEB-DRATA-COMPLIANCE lines 218-224.
DRATA-03 Multi-framework support including SOC 2, ISO 27001, GDPR, HIPAA, ISO 42001, DORA, FedRAMP, PCI DSS, and custom frameworks; source WEB-DRATA-COMPLIANCE lines 222-224.
DRATA-04 Shared control mapping across frameworks; source Drata compliance public page.
DRATA-05 Controls and evidence management in a single platform; source Drata compliance public page.
DRATA-06 Monitoring and tests for automated control checks; source Drata compliance public page.
DRATA-07 Daily/continuous compliance monitoring claim; source WEB-DRATA-COMPLIANCE lines 204-211.
DRATA-08 Trust Center / customer trust portal; source WEB-DRATA-TRUST lines 34-37.
DRATA-09 Trust library for documents, questionnaires, and resources; source WEB-DRATA-TRUST lines 55-58.
DRATA-10 Public, gated, restricted, and hidden document access; source WEB-DRATA-TRUST lines 60-63 and 75-88.
DRATA-11 AI-powered trust search; source WEB-DRATA-TRUST lines 64-66.
DRATA-12 Trust Center custom URL and publishing; source WEB-DRATA-TRUST lines 89-98.
DRATA-13 Trust Center upload metadata, owner, source, expiration, and permission levels; source WEB-DRATA-TRUST lines 75-83.
DRATA-14 Trust Center sync with GRC policies and controls; source WEB-DRATA-TRUST lines 81-83.
DRATA-15 AI questionnaire assistance grounded in approved trust content; source WEB-DRATA-TRUST lines 112-116.
DRATA-16 Internal risk management with risk register, owners, and remediation; source WEB-DRATA-RISK lines 34-38.
DRATA-17 Internal risk library with 200+ common risks; source Drata internal risk page search snippet.
DRATA-18 Risk scoring by impact and likelihood; source Drata internal risk page search snippet.
DRATA-19 Risk treatment plans and Jira/task assignment; source Drata internal risk page search snippet.
DRATA-20 Risk posture visualization; source Drata internal risk page search snippet.
DRATA-21 Integrated risk management across internal and vendor risk; source Drata risk page search snippet.
DRATA-22 Third-party risk management as centralized vendor portfolio; source WEB-DRATA-TPRM lines 118-150.
DRATA-23 Agentic document collection from vendor sources; source WEB-DRATA-TPRM lines 118-122.
DRATA-24 AI criteria generation for vendor evaluations; source WEB-DRATA-TPRM lines 126-132.
DRATA-25 Vendor source sync from procurement/CLM and app discovery; source WEB-DRATA-TPRM lines 134-136.
DRATA-26 AI risk summaries from SOC reports, questionnaires, and evidence; source WEB-DRATA-TPRM lines 138-139.
DRATA-27 Vendor risk register with owners, status, and mitigation; source WEB-DRATA-TPRM lines 141-143.
DRATA-28 Third-party directory with vendor details, owner, inherent risk, assessment history, evidence, and linked risks; source WEB-DRATA-TPRM lines 145-147.
DRATA-29 Executive reporting for criteria outcomes, evidence references, and residual risk; source WEB-DRATA-TPRM lines 149-150.
DRATA-30 Evidence-backed third-party assessment quality; source WEB-DRATA-TPRM lines 174-184.
DRATA-31 Risk decision traceability for auditors and stakeholders; source WEB-DRATA-TPRM lines 174-184.
DRATA-32 Vetted technology partner and audit firm ecosystem; source WEB-DRATA-TPRM lines 208-212.
DRATA-33 Agentic platform positioning across trust management; source Drata homepage and product pages.
DRATA-34 Framework expansion through custom frameworks; source WEB-DRATA-COMPLIANCE lines 222-224.
DRATA-35 Assurance workflow tying customer trust and security questionnaires to sales velocity; source WEB-DRATA-TRUST lines 34-66.

## §3 Counterpart 3 — OneTrust capability surface

ONETRUST-01 Consent and preference management; source WEB-ONETRUST-PRODUCTS lines 257-277.
ONETRUST-02 Universal consent and preference portal; source WEB-ONETRUST-PRODUCTS lines 263-267.
ONETRUST-03 Consent Management Platform for websites, mobile apps, OTT, and connected TVs; source WEB-ONETRUST-PRODUCTS lines 271-275.
ONETRUST-04 Privacy Operations as lifecycle privacy program; source WEB-ONETRUST-PRODUCTS lines 282-292.
ONETRUST-05 Personal data flow visibility; source WEB-ONETRUST-PRODUCTS lines 288-292.
ONETRUST-06 Asset location and classification; source WEB-ONETRUST-PRODUCTS lines 288-292.
ONETRUST-07 Real-time privacy risk assessment; source WEB-ONETRUST-PRODUCTS lines 288-292.
ONETRUST-08 Incident and privacy notice management; source WEB-ONETRUST-PRODUCTS lines 288-292.
ONETRUST-09 Data Subject Request automation; source WEB-ONETRUST-PRODUCTS lines 296-300.
ONETRUST-10 DSR intake automation; source WEB-ONETRUST-PRODUCTS lines 296-300.
ONETRUST-11 DSR identity verification; source WEB-ONETRUST-PRODUCTS lines 296-300.
ONETRUST-12 DSR data discovery; source WEB-ONETRUST-PRODUCTS lines 296-300.
ONETRUST-13 DSR redaction and secure response; source WEB-ONETRUST-PRODUCTS lines 296-300.
ONETRUST-14 Secure DSR customer portal; source WEB-ONETRUST-DSR lines 237-244.
ONETRUST-15 DSR integration with CMP and Trust Center; source OneTrust DSR page search snippet.
ONETRUST-16 DataGuidance same-day privacy/security regulatory updates; source WEB-ONETRUST-PRODUCTS lines 303-307.
ONETRUST-17 AI Governance; source WEB-ONETRUST-PRODUCTS lines 314-318.
ONETRUST-18 AI inventory of models, datasets, agents, and vendors; source WEB-ONETRUST-AI lines 251-259.
ONETRUST-19 AI ownership and lifecycle status; source WEB-ONETRUST-AI lines 251-259.
ONETRUST-20 AI component dependency understanding; source WEB-ONETRUST-AI lines 251-259.
ONETRUST-21 AI risk assessments, policy enforcement, model monitoring, and documentation; source WEB-ONETRUST-AI lines 420-435.
ONETRUST-22 EU AI Act/global AI regulation compliance workflow; source WEB-ONETRUST-AI lines 426-435.
ONETRUST-23 Compliance Automation; source WEB-ONETRUST-PRODUCTS lines 322-333.
ONETRUST-24 IT Risk Management; source WEB-ONETRUST-PRODUCTS lines 336-340.
ONETRUST-25 Risk dashboards and KRIs; source WEB-ONETRUST-IT-RISK equivalent opened lines 243-245.
ONETRUST-26 Third-party management from intake to risk assessment, mitigation, reporting, monitoring; source WEB-ONETRUST-PRODUCTS lines 347-356.
ONETRUST-27 Third-party risk management lifecycle automation; source WEB-ONETRUST-TPRM lines 213-221.
ONETRUST-28 Third-party customized inventory; source WEB-ONETRUST-TPRM lines 217-220.
ONETRUST-29 Third-party vendor assessments with chosen control framework; source WEB-ONETRUST-TPRM lines 219-232.
ONETRUST-30 Continuous third-party monitoring and reassessment triggers; source WEB-ONETRUST-TPRM lines 220-249.
ONETRUST-31 Third-party mitigation recommendations and workflows; source WEB-ONETRUST-TPRM lines 242-245.
ONETRUST-32 Third-party role-based dashboards and PDF reporting; source WEB-ONETRUST-TPRM lines 252-254.
ONETRUST-33 Third-party due diligence; source WEB-ONETRUST-PRODUCTS lines 360-365.
ONETRUST-34 Third-party risk exchange and control gap reports on thousands of vendors; source WEB-ONETRUST-PRODUCTS lines 368-372.
ONETRUST-35 Cyber risk ratings and breach notifications; source WEB-ONETRUST-THIRD-PARTY lines 269-273.
ONETRUST-36 Third-party critical event triggered automation rules; source WEB-ONETRUST-THIRD-PARTY lines 217-240.
ONETRUST-37 Multi-domain third-party assessments across security, privacy, ethics, compliance, and more; source WEB-ONETRUST-THIRD-PARTY lines 263-266.
ONETRUST-38 Third-party offboarding; source WEB-ONETRUST-PRODUCTS lines 353-356.
ONETRUST-39 Integrations across OneTrust platform; source WEB-ONETRUST-PRODUCTS lines 384-388.
ONETRUST-40 Unified governance platform across privacy, risk, data, AI, and compliance; source WEB-ONETRUST-PRODUCTS lines 133-162 and 257-383.

## §4 UNION-coverage matrix

| Capability | Vanta | Drata | OneTrust | UNION required | Oyatie compliance has | Gap classification |
|---|---|---|---|---|---|---|
| Continuous evidence collection | yes | yes | partial | yes | yes, PRD/OpenAPI | present |
| Continuous control monitoring | yes | yes | yes | yes | partial, SLO/docs | partial |
| Framework mapping | yes | yes | yes | yes | yes, compliance.md/scorecards | present |
| Multi-framework shared controls | partial | yes | yes | yes | partial | partial |
| Custom frameworks/packs | partial | yes | yes | yes | yes, pack overlays | present |
| SOC 2 coverage | yes | yes | yes | yes | yes | present |
| ISO 27001 coverage | yes | yes | yes | yes | partial, missing JSON mapping | partial |
| GDPR coverage | yes | yes | yes | yes | yes | present |
| HIPAA coverage | yes | yes | partial | yes | yes | present |
| PCI DSS coverage | yes | yes | partial | yes | partial, deferred in PRD | partial |
| FedRAMP coverage | partial | yes | partial | yes | partial, pack exists in manifest only | partial |
| DORA coverage | partial | yes | yes | yes | partial, pack not explicit in root PRD | partial |
| EU AI Act coverage | partial | partial | yes | yes | yes, EU AI Act pack | present |
| Trust center | yes | yes | partial | yes | partial, auditor portal only | missing product surface |
| Trust center gated documents | yes | yes | partial | yes | no endpoint | missing |
| Trust center public/private resource controls | yes | yes | partial | yes | no endpoint | missing |
| Trust center custom branding | yes | yes | partial | yes | no artifact | missing |
| Trust center analytics | yes | yes | partial | yes | no artifact | missing |
| Trust center CRM integration | yes | partial | partial | yes | no artifact | missing |
| Trust center NDA automation | yes | partial | partial | yes | no artifact | missing |
| Trust center AI search/chat | yes | yes | partial | yes | no artifact | missing |
| Security questionnaire automation | yes | yes | partial | yes | no artifact | missing |
| Questionnaire import | yes | partial | partial | yes | no artifact | missing |
| Approved answer knowledge base | yes | yes | partial | yes | no artifact | missing |
| Questionnaire reviewer workflow | yes | partial | partial | yes | no artifact | missing |
| Questionnaire portal/browser support | yes | partial | partial | yes | no artifact | missing |
| Questionnaire policy sync | yes | yes | partial | yes | no artifact | missing |
| Internal risk register | yes | yes | yes | yes | no contract | missing |
| Risk scoring | yes | yes | yes | yes | partial via DPIA risk | partial |
| Risk treatment plan | partial | yes | yes | yes | no contract | missing |
| Risk owner assignment | partial | yes | yes | yes | partial in runbooks | partial |
| Jira/task remediation | partial | yes | partial | yes | no artifact | missing |
| Risk dashboards/KRIs | partial | yes | yes | yes | partial dashboards only | partial |
| Third-party vendor inventory | yes | yes | yes | yes | no artifact | missing |
| Vendor intake | yes | yes | yes | yes | no artifact | missing |
| Vendor risk assessment | yes | yes | yes | yes | no artifact | missing |
| Vendor questionnaires | yes | yes | yes | yes | no artifact | missing |
| Vendor evidence request/reminders | yes | yes | partial | yes | no artifact | missing |
| Vendor SOC report AI summary | yes | yes | partial | yes | no artifact | missing |
| Vendor inherent risk tier | yes | yes | yes | yes | no artifact | missing |
| Vendor residual risk | yes | yes | yes | yes | no artifact | missing |
| Vendor monitoring | yes | yes | yes | yes | no artifact | missing |
| Vendor reassessment triggers | yes | yes | yes | yes | no artifact | missing |
| Vendor risk register | partial | yes | yes | yes | no artifact | missing |
| Vendor executive reporting | partial | yes | yes | yes | no artifact | missing |
| Vendor risk exchange | no | partial | yes | yes | no artifact | missing |
| Third-party due diligence | partial | partial | yes | yes | no artifact | missing |
| Access review workflow | yes | partial | partial | yes | partial evidence collector only | partial |
| Account access import | yes | partial | partial | yes | no endpoint | missing |
| Access review scheduling | yes | partial | partial | yes | no endpoint | missing |
| Employee lifecycle integration | yes | partial | partial | yes | no endpoint | missing |
| HRIS integration evidence | yes | partial | partial | yes | no integration catalog | missing |
| Cloud provider integration evidence | yes | yes | partial | yes | partial via deployment receipts | partial |
| Vulnerability scanner integration | yes | yes | partial | yes | partial collector | partial |
| Security awareness integration | yes | partial | partial | yes | no artifact | missing |
| Policy templates | yes | yes | yes | yes | partial policy files | partial |
| Policy approval workflow | yes | yes | yes | yes | no endpoint | missing |
| Policy acceptance tracking | yes | partial | partial | yes | no endpoint | missing |
| Privacy operations | partial | partial | yes | yes | partial DPIA/DSAR | partial |
| Data map/RoPA | partial | partial | yes | yes | broken RoPA reference | partial-broken |
| Personal data discovery | no | partial | yes | yes | no artifact | missing |
| Privacy notices | no | partial | yes | yes | no artifact | missing |
| Consent management | no | partial | yes | yes | no artifact | missing |
| Preference management | no | no | yes | yes | no artifact | missing |
| Cookie consent CMP | no | no | yes | yes | no artifact | missing |
| Data Subject Request intake | partial | partial | yes | yes | yes | present |
| DSR identity verification | no | partial | yes | yes | no endpoint | missing |
| DSR data discovery | no | partial | yes | yes | partial DSAR cascade intent | partial |
| DSR redaction | no | partial | yes | yes | no endpoint | missing |
| DSR secure response portal | partial | partial | yes | yes | partial auditor/DSAR status only | partial |
| DSR legal hold checks | partial | partial | yes | yes | partial tier docs | partial |
| DSR erasure conflict report | no | partial | yes | yes | yes, tutorial/tier | present |
| Breach notification clock | partial | partial | yes | yes | yes | present |
| Jurisdiction-specific breach guidance | partial | partial | yes | yes | yes, packs/journey IPs | present |
| Regulator evidence export | partial | partial | partial | yes | partial | partial |
| Auditor portal | yes | yes | partial | yes | yes | present |
| Auditor access expiry | partial | partial | partial | yes | yes, PRD | present |
| Audit-chain seal verification | no | partial | partial | yes | yes, differentiator | additive-present |
| Immutable pack registry | no | partial | partial | yes | yes | additive-present |
| Pack hotfix without rewriting history | no | partial | partial | yes | yes | additive-present |
| Higher-restriction-wins enforcement | no | partial | yes | yes | yes | additive-present |
| Cross-jurisdiction transfer evidence | no | partial | yes | yes | yes, paid compliance_pack | additive-present |
| Cell certification attestation | no | no | partial | yes | yes | additive-present |
| Sovereign-pack residency | partial | partial | yes | yes | yes, paid compliance_pack | additive-present |
| Air-gap tiering | no | no | partial | yes | yes, paid compliance_pack | additive-present |
| AI governance inventory | no | partial | yes | yes | no artifact | missing |
| AI ownership/lifecycle status | no | partial | yes | yes | no artifact | missing |
| AI risk assessments | no | partial | yes | yes | partial EU AI Act pack | partial |
| Model/dataset/agent dependency mapping | no | no | yes | yes | no artifact | missing |
| Regulatory intelligence updates | no | partial | yes | yes | no artifact | missing |
| Same-day regulatory update feed | no | no | yes | yes | no artifact | missing |
| Control-gap reports on vendor exchange | no | partial | yes | yes | no artifact | missing |
| Procurement/CLM vendor source sync | partial | yes | partial | yes | no artifact | missing |
| Executive reports | partial | yes | yes | yes | partial dashboards | partial |
| Brandable PDF reports | partial | partial | yes | yes | no artifact | missing |
| Marketplace/customer assurance analytics | yes | yes | partial | yes | no artifact | missing |
| Revenue influence tracking | yes | partial | no | yes | no artifact | missing |
| Partner/audit ecosystem directory | partial | yes | yes | yes | no artifact | missing |
| Integration marketplace/catalog | yes | yes | yes | yes | no artifact | missing |
| Continuous posture external proof | yes | yes | partial | yes | partial auditor portal | partial |
| API for external automation | partial | partial | partial | yes | partial OpenAPI | partial |
| Multi-context deployment proof | no | no | no | Oyatie-specific | no | canonical gap |
| OpenTofu deployment modules | no | no | no | Oyatie-specific | no | canonical gap |
| OS support matrix | no | no | no | Oyatie-specific | no | canonical gap |
| Rust-only backend | no | no | no | Oyatie-specific | no source files found | canonical partial |
| OCI Always Free demo_trial | no | no | no | Oyatie-specific | no | canonical gap |

## §5 Capability families summary table

| Family | UNION required count | Oyatie present | Oyatie partial | Oyatie missing | Assessment |
|---|---:|---:|---:|---:|---|
| Compliance evidence and controls | 14 | 8 | 6 | 0 | Strong core, needs mapping cleanup |
| Trust center and customer assurance | 12 | 1 | 2 | 9 | Behind Vanta/Drata |
| Questionnaire automation | 7 | 0 | 0 | 7 | Absent |
| Internal risk management | 7 | 0 | 3 | 4 | Mostly absent |
| Third-party risk management | 16 | 0 | 1 | 15 | Absent relative to all top-3 |
| Access reviews and personnel evidence | 7 | 0 | 2 | 5 | Evidence collector exists, workflow absent |
| Privacy operations and consent | 13 | 1 | 5 | 7 | DSAR/DPIA present, consent/data map weak |
| DSR automation | 7 | 2 | 4 | 1 | Good start, missing identity/redaction depth |
| Breach/regulator/auditor evidence | 8 | 5 | 3 | 0 | Stronger than average on packs |
| Pack enforcement and sovereignty | 8 | 8 | 0 | 0 | Oyatie additive strength |
| AI governance | 6 | 0 | 1 | 5 | Mostly absent except EU AI Act pack |
| Integration ecosystem | 5 | 0 | 1 | 4 | Absent catalog |
| Oyatie canonical deployment constraints | 5 | 0 | 1 | 4 | Not covered by counterparts; still required |

## §6 Headline gap analysis — top 15 missing capabilities

1. Customer trust center product surface: Oyatie has auditor portal intent but no buyer-facing resource library, gated access, request approval, branding, analytics, or CRM integration; hook into `contracts/openapi.yaml` with `/trust/resources`, `/trust/access-requests`, and `/trust/analytics`.
2. Questionnaire automation: Vanta and Drata both make AI-assisted questionnaires core; hook into compliance with `Questionnaire`, `ApprovedAnswer`, `KnowledgeResource`, `ReviewAssignment`, and `AnswerProvenance` entities.
3. Third-party vendor inventory: all top-3 cover vendor lifecycle; hook into compliance or a vendor-risk boundary with `VendorProfile`, `VendorOwner`, `DataAccessClass`, and `CriticalityTier`.
4. Vendor assessment workflow: Vanta/Drata/OneTrust all support questionnaires and evidence review; add `VendorAssessment`, `EvidenceRequest`, `Finding`, `FollowUp`, and `ResidualRiskDecision`.
5. Continuous vendor monitoring: top-3 support monitoring and reassessment; integrate with detection/observability and emit `compliance.vendor_risk.signal.detected`.
6. Internal risk register: Drata and Vanta risk pages show risk registers and treatment; add `RiskScenario`, `RiskScore`, `RiskOwner`, `TreatmentPlan`, and control mapping.
7. Risk treatment workflow: add task/Jira equivalent through workflow-engine and ensure audit-chain evidence for accept/transfer/mitigate/avoid decisions.
8. Consent/preference management: OneTrust has dedicated CMP and preference surface; either implement here or formally hand off to consent µservice with compliance pack linkage.
9. RoPA/data map: current `compliance.md` claims `policy/ropa.json` but it is missing; add machine-readable RoPA schema and data-flow inventory.
10. Personal data discovery and classification: OneTrust privacy operations emphasizes data assets and personal data monitoring; hook to data catalog/ontology and emit evidence for privacy packs.
11. DSR identity verification: OneTrust DSR automation includes identity verification; add state machine before export/delete/rectify.
12. DSR redaction and secure response: add secure portal/download package model and redaction trace rather than status-only endpoint.
13. AI governance inventory: OneTrust AI governance tracks models, datasets, agents, vendors, ownership, lifecycle, and dependencies; add EU AI Act pack-backed AI asset inventory or handoff to intelligence/governance.
14. Integration catalog: Vanta/Drata/OneTrust depend on integrations; add catalog for IdP, HRIS, cloud provider, vuln scanner, training, ticketing, procurement, and trust-center sources.
15. Customer assurance analytics: Vanta and Drata expose trust-center analytics and business impact; add event schema for views, access grants, resource downloads, buyer questions, and revenue correlation without leaking customer data.

## §7 Additive surface — Oyatie capabilities not clearly present in all counterparts

1. Versioned, signed compliance packs as first-class tenant-installed objects.
2. Cedar-evaluated higher-restriction-wins precedence across packs.
3. Audit-chain seal on pack publish, activation, hotfix, conflict resolution, and effective-policy projection.
4. Deterministic conflict reports for cross-jurisdiction pack collisions.
5. Cell certification attestation tied to pack eligibility.
6. Sovereign-pack residency and cross-pack federation disablement for paid compliance_pack.
7. Regulator-attested pack publishing ceremony.
8. Cross-jurisdictional transfer evidence for PIPL, GDPR, PIPA, SCCs, and UK transfer regimes.
9. Pack-bound DPO workspace concept.
10. EU AI Act refusal pipeline tied to platform features.
11. Legal-hold pipeline by pack and regulation.
12. Evidence replay from pack overlay history.
13. Compliance-pack hotfix without rewriting history.
14. Cross-cell policy ceiling of metadata-only unless pack policy allows more.
15. Tenant-cell placement based on compliance pack certification.
16. OpenTofu per-context deployment requirement as a platform audit property.
17. OCI Always Free demo_trial requirement for guest-on-OCI.
18. Rust-strict backend and no scripting doctrine for service implementation.
19. OS support matrix that explicitly includes enterprise Linux, Talos, Flatcar, Photon, and macOS M5+.
20. Audit-agent nine-dimension coherence gate as a quality control surface.

## Closing parity verdict

The service should not be represented as full Vanta + Drata + OneTrust union parity today.
It can honestly claim a strong platform-native compliance-pack and evidence foundation.
It can partially claim DSAR, breach notification, auditor portal, regulator evidence, and control mapping.
It cannot yet claim top-3 parity for trust center, questionnaire automation, TPRM, consent, broad privacy operations, internal risk management, AI governance inventory, or integration marketplace.
The next remediation wave should decide which of those missing surfaces belong inside compliance and which belong to dedicated µservices with formal handoffs.

## §8 Source-to-Oyatie implementation hook ledger

HOOK-01 Trust center resource library: public evidence from Vanta Trust Center and Drata Customer Trust Portal maps to an Oyatie `TrustResource` aggregate, absent from `contracts/openapi.yaml:13-139`.
HOOK-02 Trust center access request: Vanta trust center access and Drata portal request workflows map to `TrustAccessRequest`, absent from current contracts.
HOOK-03 NDA-gated evidence release: Vanta trust center automation suggests `TrustAccessGrant` with policy evidence sealed through audit-chain.
HOOK-04 Trust center analytics: Vanta reports buyer activity tracking; Oyatie should capture `trust.resource.viewed`, `trust.file.downloaded`, and `trust.question.asked` events.
HOOK-05 Trust center CRM attribution: Vanta revenue-influence claims imply optional CRM handoff, but compliance should own only evidence provenance and buyer-resource audit events.
HOOK-06 Questionnaire import: Vanta and Drata questionnaire pages map to `QuestionnaireImportJob`, missing from current OpenAPI.
HOOK-07 Questionnaire answer source: counterpart automation maps to `ApprovedAnswer` plus `AnswerSourceEvidence` so generated answers cite audit-chain objects.
HOOK-08 Questionnaire review assignment: counterpart collaboration maps to `QuestionnaireReviewTask` owned by compliance or a workflow µservice.
HOOK-09 Questionnaire export: counterpart buyer workflows map to `CompletedQuestionnairePackage` with signed output and reviewer identity.
HOOK-10 TPRM vendor profile: Vanta, Drata, and OneTrust all require `VendorProfile`, absent from `manifest.json:165-174` dependencies and contracts.
HOOK-11 TPRM inherent risk: counterpart vendor workflows require `VendorInherentRiskAssessment` with data category, criticality, region, and service access.
HOOK-12 TPRM assessment template: counterpart vendor questionnaires require `VendorAssessmentTemplate`, separate from compliance packs.
HOOK-13 TPRM residual risk decision: OneTrust risk workflow maps to `VendorResidualRiskDecision` sealed through audit-chain.
HOOK-14 TPRM continuous monitoring: Drata and OneTrust monitoring maps to detection/observability events consumed by compliance.
HOOK-15 Risk register: Drata and Vanta risk pages map to `RiskScenario`, `RiskOwner`, `RiskScore`, and `TreatmentPlan`.
HOOK-16 Risk-control mapping: risk register entries must attach to `ControlMapping`, currently only sketched by `IP-022-compliance-control-mapping-domain.md`.
HOOK-17 Risk acceptance: counterpart GRC workflows require approval chain, expiration, compensating controls, and evidence retention.
HOOK-18 Access review campaign: Vanta access reviews map to `AccessReviewCampaign`, absent from compliance contracts.
HOOK-19 Access review subject: access review needs subject identity, application, entitlement, reviewer, and decision artifacts.
HOOK-20 Access review remediation: review revocation must hand off to identity/IAM while compliance records evidence.
HOOK-21 Personnel evidence: Vanta/Drata employee evidence maps to HRIS/training integrations, absent from local integration catalog.
HOOK-22 Policy acceptance: counterpart compliance automation maps to policy acknowledgement evidence and training completion evidence.
HOOK-23 Asset inventory: Vanta and Drata rely on systems/assets; Oyatie should either consume platform inventory or declare compliance-owned asset evidence views.
HOOK-24 Framework mapping: current pack registry covers frameworks; remediation should normalize framework/control mapping into machine-readable pack schemas.
HOOK-25 Custom controls: Vanta/Drata enterprise surfaces imply tenant-defined controls with inherited evidence mappings.
HOOK-26 Control tests: counterpart monitoring requires control-test definitions, frequencies, owners, evidence queries, and exceptions.
HOOK-27 Exception management: failed controls require exception state, approval, expiration, and corrective-action proof.
HOOK-28 Auditor request queue: PRD mentions auditor endpoint; counterpart parity requires request/response state machine, due date, redaction, and release approval.
HOOK-29 Audit package generation: counterpart audit readiness maps to `AuditPackage` with scope, period, controls, evidence bundle, and seal verification.
HOOK-30 DSR intake: OneTrust DSR automation maps to current `/dsar` endpoints but needs identity verification and secure response model.
HOOK-31 DSR identity verification: add verification challenge, evidence source, confidence, and manual escalation.
HOOK-32 DSR redaction: add export redaction plan, redactor identity, policy reason, and subject-visible package evidence.
HOOK-33 DSR deletion orchestration: add delete/rectify cascades across microservices, with compliance owning pack rule and evidence.
HOOK-34 Data map: OneTrust data mapping maps to missing `policy/ropa.json` and should become a first-class `ProcessingActivity` catalog.
HOOK-35 RoPA control: current `compliance.md:42` reference to missing RoPA file is the implementation hook for privacy operation parity.
HOOK-36 Consent inventory: OneTrust CMP/preference capabilities require a boundary decision; compliance can own evidence while a consent µservice owns runtime preference UX.
HOOK-37 Preference proof: compliance should capture consent/preference evidence events, not necessarily run consent UI.
HOOK-38 Cookie/banner surface: OneTrust has public consent UI; Oyatie compliance should declare this as out-of-scope or formally own it.
HOOK-39 Data discovery: OneTrust privacy operations imply data source scan integration; compliance should consume catalog/classification evidence.
HOOK-40 Privacy impact assessment: current DPIA plans map well to OneTrust PIA, but need contract endpoints and template schema.
HOOK-41 DPIA approval workflow: add reviewer roles, risk treatment, legal basis, and residual-risk evidence.
HOOK-42 Breach clock: Oyatie is strong here; keep authority, subject, regulator, and internal notification deadlines as pack data.
HOOK-43 Breach evidence package: add export object combining incident, timeline, decision log, notices, and regulator response.
HOOK-44 Regulator portal: current artifacts mention regulator evidence; counterpart parity requires role-grant, expiration, scope, and activity logs.
HOOK-45 Regulatory intelligence: OneTrust regulatory-change features map to pack update feeds and jurisdiction delta review.
HOOK-46 AI system inventory: OneTrust AI Governance maps to `AISystem`, `ModelOwner`, `UseCase`, `TrainingDataClass`, and `RiskClass`.
HOOK-47 AI vendor dependency: AI governance should reuse TPRM vendor model for model/API providers.
HOOK-48 AI assessment: add EU AI Act assessment workflow with pack-bound evidence and refusal reasons.
HOOK-49 AI monitoring: integrate intelligence/detection outputs as evidence rather than embedding runtime model telemetry in compliance.
HOOK-50 Integration catalog: counterpart integration ecosystems require a machine-readable catalog of evidence sources and scopes.
HOOK-51 Integration health: add source connector freshness, last successful evidence pull, and stale evidence alerts.
HOOK-52 Integration authorization: connector credentials should flow through OpenBao and service-specific lease evidence.
HOOK-53 Migration from Vanta: existing migration playbook should map competitor objects to Oyatie controls, packs, evidence, and missing fields.
HOOK-54 Migration from Drata: add import mapping for controls, evidence, trust center resources, risk register, and questionnaire answers.
HOOK-55 Migration from OneTrust: existing OneTrust playbook should include consent, DSR, data map, TPRM, AI governance, and regulatory-change gaps.
HOOK-56 Public API parity: current OpenAPI should expand by bounded domains rather than one generic compliance endpoint.
HOOK-57 Event parity: AsyncAPI should include trust, questionnaire, vendor, risk, consent-evidence, and AI-governance events when scope decisions land.
HOOK-58 Proto parity: `contracts/compliance.proto:5-20` needs service coverage for pack registry, evidence, regulator export, and parity domains.
HOOK-59 Pack schema parity: counterpart frameworks should map to versioned pack objects with controls, tests, evidence queries, and product surfaces.
HOOK-60 Evidence object parity: every domain above should emit a common evidence envelope with tenant, pack, actor, action, object, seal, retention, and redaction policy.
HOOK-61 Canonical context hook: every new product capability must state whether it runs in all six deployment contexts or is intentionally gated.
HOOK-62 Canonical IaC hook: every deployable parity surface must be supported by OpenTofu modules, not hand-edited environment notes.
HOOK-63 Canonical OS hook: self-managed parity claims need package/runtime declarations across the Tier-1 OS matrix.
HOOK-64 Canonical language hook: implementation hooks must land in Rust backend crates or allowed Swift/Kotlin/WinUI3 frontend directories only.
HOOK-65 Canonical OCI hook: demo_trial parity surfaces must be sized for OCI Always Free or explicitly paid tenant_class on OCI.
HOOK-66 Wave 14 aggregation hook: decide trust center ownership before adding endpoints to avoid duplicating a customer-assurance service.
HOOK-67 Wave 14 aggregation hook: decide TPRM ownership before adding vendor entities to avoid splitting procurement and compliance records.
HOOK-68 Wave 14 aggregation hook: decide consent ownership before compliance absorbs runtime preference management.
HOOK-69 Wave 14 aggregation hook: decide AI governance ownership before duplicating intelligence/governance inventories.
HOOK-70 Wave 14 aggregation hook: keep compliance as the evidence and pack authority even when adjacent products own runtime UX.
HOOK-71 Final hook verdict: the union-coverage gap is product-scope breadth, not only documentation depth.
HOOK-72 Final hook verdict: the additive pack substrate should remain the service's differentiator while missing counterpart surfaces are added through explicit boundaries.
