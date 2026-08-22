---
doc_class: LocalizationPack
pack_id: US-PACK-1
version: "1.0.0"
status: Accepted
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0003
  - ADR-0099
  - ADR-0116
  - ADR-0217
  - ADR-0251
citing_authority_url:
  - https://privacy.ca.gov/california-privacy-rights/rights-under-the-california-consumer-privacy-act/
  - https://www.leginfo.legislature.ca.gov/faces/codes_displayText.xhtml?article=&chapter=&division=3.&lawCode=CIV&part=4.&title=1.81.5.
  - https://www.hhs.gov/hipaa/for-professionals/privacy/laws-regulations/index.html
  - https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C/part-160
  - https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C/part-164
  - https://www.ecfr.gov/current/title-16/chapter-I/subchapter-C/part-313
  - https://www.ecfr.gov/current/title-16/chapter-I/subchapter-C/part-314
  - https://www.ecfr.gov/current/title-34/subtitle-A/part-99
  - https://www.ftc.gov/legal-library/browse/statutes/fair-credit-reporting-act
  - https://www.consumerfinance.gov/rules-policy/regulations/1002/
---

# US-PACK-1 United States Localization Pack

This directory is the canonical United States localization pack for Oyatie.
The pack binds federal privacy, financial-reporting, healthcare, education, child-privacy, credit, accessibility, employment-selection, defense-export, security-control, and cloud-authorization duties into pack-scoped policy, data, API, and audit deltas.
The pack is documentation-only in this slice.
The pack does not modify ADRs.
The pack does not modify microservice manifests.
The pack does not modify other localization packs.
The pack provides the implementation contract for later Cedar fragments, schemas, OpenAPI overlays, audit-chain event classes, and conformance gates.
The pack is tenant-activated, not globally assumed.
The pack is not legal advice and does not replace counsel review for regulated tenants.
The pack is designed so later machine-readable artifacts can be derived without inventing statutory meaning during implementation.
The stop condition for this slice is six authored documents, each at or above the requested line depth, with VCS verify/done/promote evidence captured.
The checkpoint label is `us-localization-pack-w1-2026-05-20`.

## Pack Identity

- Pack id: `US-PACK-1`.
- Pack version: `1.0.0`.
- Pack status: `Accepted`.
- Pack jurisdiction: United States federal plus listed state overlays.
- Pack coverage date: 2026-05-20.
- Pack class: `LocalizationPack`.
- Pack owner surface: compliance pack registry once the registry substrate is available.
- Pack precedence mode: strongest applicable duty wins unless a federal preemption clause displaces state law.
- Pack activation mode: tenant opt-in, sector opt-in, or contract-required opt-in.
- Pack default posture: deny processing when the controller cannot classify the data, actor, purpose, jurisdiction, and authority basis.
- Pack audit posture: every regulated permission and denial emits a typed audit-chain event.
- Pack data posture: regulated categories extend existing data-class naming without changing other packs.
- Pack API posture: APIs add jurisdiction-aware request metadata, consent flags, subject-rights routes, and evidence references.
- Pack Cedar posture: policies are named here but not authored as Cedar files in this slice.
- Pack checkpoint: `us-localization-pack-w1-2026-05-20`.

## Authority Citations

- US-AUTH-001: CCPA and CPRA rights are represented through California Civil Code Title 1.81.5 and the California Privacy Protection Agency consumer-rights summary.
- US-AUTH-002: California sensitive personal information maps to Civil Code Section 1798.140(ae) categories, including precise geolocation, credentials, genetic data, neural data, health, sex life, sexual orientation, and biometric identification.
- US-AUTH-003: California consumer rights include limit, opt-out, correct, know, equal treatment, and delete.
- US-AUTH-004: Colorado privacy obligations map to the Colorado Privacy Act, Colorado Revised Statutes Title 6, Article 1, Part 13.
- US-AUTH-005: Connecticut privacy obligations map to Conn. Gen. Stat. Section 42-515 et seq. and AG guidance on CTDPA enforcement, sensitive data, universal opt-out signals, and teen protections.
- US-AUTH-006: Virginia privacy obligations map to Code of Virginia Title 59.1, Chapter 53.
- US-AUTH-007: Utah privacy obligations map to Utah Code Title 13, Chapter 61 and Utah Division of Consumer Protection UCPA materials.
- US-AUTH-008: Texas privacy obligations map to Texas Business and Commerce Code Chapter 541.
- US-AUTH-009: Iowa privacy obligations map to Iowa Code Chapter 715D.
- US-AUTH-010: Indiana privacy obligations map to Indiana Code Title 24, Article 15 once effective in 2026.
- US-AUTH-011: Tennessee privacy obligations map to the Tennessee Information Protection Act, Tennessee Code Annotated Title 47, Chapter 18, Part 33.
- US-AUTH-012: Montana privacy obligations map to Montana Code Annotated Title 30, Chapter 14, Part 28.
- US-AUTH-013: Oregon privacy obligations map to ORS 646A.570 through 646A.589.
- US-AUTH-014: Delaware privacy obligations map to Delaware Code Title 6, Chapter 12D.
- US-AUTH-015: New Jersey privacy obligations map to P.L. 2023, c.266, codified at N.J. Stat. Section 56:8-166.4 et seq.
- US-AUTH-016: New Hampshire privacy obligations map to RSA Chapter 507-H, Expectation of Privacy.
- US-AUTH-017: HIPAA obligations map to 45 CFR Parts 160, 162, and 164, with Privacy, Security, Breach Notification, and transaction-code-set consequences.
- US-AUTH-018: GLBA obligations map to 15 U.S.C. 6801 et seq. and FTC Privacy Rule 16 CFR Part 313 plus Safeguards Rule 16 CFR Part 314.
- US-AUTH-019: FERPA obligations map to 20 U.S.C. 1232g and 34 CFR Part 99.
- US-AUTH-020: COPPA obligations map to 15 U.S.C. 6501 et seq. and 16 CFR Part 312.
- US-AUTH-021: FCRA obligations map to 15 U.S.C. 1681 et seq., FTC statutory material, and CFPB Regulation V, 12 CFR Part 1022.
- US-AUTH-022: ECOA obligations map to 15 U.S.C. 1691 et seq. and CFPB Regulation B, 12 CFR Part 1002.
- US-AUTH-023: SOX obligations map to Sarbanes-Oxley Act Sections 302, 404, 802, and 906.
- US-AUTH-024: PCAOB AS 2201 governs integrated audits of internal control over financial reporting.
- US-AUTH-025: ADA obligations map to ADA Titles II and III and DOJ regulations at 28 CFR Parts 35 and 36.
- US-AUTH-026: SOC 2 obligations map to AICPA Trust Services Criteria for security, availability, processing integrity, confidentiality, and privacy.
- US-AUTH-027: EEOC UGESP obligations map to the Uniform Guidelines on Employee Selection Procedures and the four-fifths adverse-impact rule of thumb.
- US-AUTH-028: ITAR obligations map to the Arms Export Control Act and 22 CFR Parts 120 through 130.
- US-AUTH-029: NIST SP 800-53 Rev. 5 provides security and privacy controls for systems and organizations.
- US-AUTH-030: FedRAMP Rev. 5 baselines align cloud-service authorization evidence with NIST SP 800-53 Rev. 5.
- US-AUTH-031: NYC Local Law 144 governs AEDT use for hiring and promotion in New York City.
- US-AUTH-032: Colorado SB 24-205 defines obligations for high-risk AI systems used in consequential decisions.
- US-AUTH-033: California AB-331 is tracked as a nonbinding automated decision tool bill reference in this pack unless superseded by enacted California law.
- US-AUTH-034: State privacy laws preserve sector-specific exemptions for HIPAA, GLBA, FCRA, and FERPA data in varying forms; pack routing must classify both entity and data exemptions.
- US-AUTH-035: Federal law can preempt state overlay claims; the pack records federal authority first and applies state overlays only where not displaced.
- US-AUTH-036: Attorney General enforcement is the common model across the listed state privacy statutes; private rights of action are not assumed unless the named law expressly supplies them.
- US-AUTH-037: Universal opt-out signals are mandatory in California and Connecticut and are added in other state overlays where statutory text requires them.
- US-AUTH-038: Child age thresholds differ between COPPA-under-13 duties, California under-16 opt-in sale/share duties, Connecticut teen opt-in sale/targeted-ad duties, and state child definitions tied to COPPA.
- US-AUTH-039: Sensitive-data processing is opt-in in most listed states, but Utah and Iowa use a notice-plus-opt-out posture; Cedar fragments must preserve that difference.
- US-AUTH-040: Financial, health, education, defense, and employment data may be simultaneously regulated; the pack resolves compound duties by stacking classifiers rather than collapsing them.

## Pack Scope

- SCOPE-001: Covered tenants include US-resident consumer-facing tenants that activate the US privacy overlay.
- SCOPE-002: Covered tenants include healthcare tenants that create, receive, maintain, or transmit PHI for covered entities or business associates.
- SCOPE-003: Covered tenants include financial-services tenants handling nonpublic personal information or credit-decision data.
- SCOPE-004: Covered tenants include education tenants receiving or processing education records for schools subject to FERPA.
- SCOPE-005: Covered tenants include child-directed services or services with actual knowledge of under-13 users.
- SCOPE-006: Covered tenants include public-company accounting, close, controls, and evidence workflows that support SEC reporting or SOX control assertions.
- SCOPE-007: Covered tenants include HR, recruiting, learning, payroll, promotion, compensation, and screening workflows using selection procedures or automated decision support.
- SCOPE-008: Covered tenants include cloud-service tenants requiring NIST 800-53 or FedRAMP evidence.
- SCOPE-009: Covered tenants include export-controlled defense tenants that store or expose ITAR technical data.
- SCOPE-010: Covered tenants include public-accommodation or state/local-government digital surfaces requiring ADA accessibility handling.
- SCOPE-011: Out of scope: non-US-only tenants with no US subject, no US business nexus, and no US contract requirement.
- SCOPE-012: Out of scope: legal advice on whether a tenant is a covered business; the pack records system controls for tenants that activate or are flagged.
- SCOPE-013: Out of scope: authoring of executable Cedar files in this documentation slice.
- SCOPE-014: Out of scope: registry publication outside `packs/us-localization/`.
- SCOPE-015: Out of scope: modification of CN, KR, or EU localization packs.
- SCOPE-016: Out of scope: replacing sector-specific counsel signoff for HIPAA, GLBA, FCRA, ECOA, ITAR, SOX, or FERPA deployments.
- SCOPE-017: In scope: field-level data classifications that downstream data-class gates can consume.
- SCOPE-018: In scope: endpoint deltas that later OpenAPI overlays can implement.
- SCOPE-019: In scope: audit event classes that audit-chain can type and seal.
- SCOPE-020: In scope: failure modes and worked examples that prevent implementation-time invention.

## Pack Precedence

- PRECEDENCE-001: Federal sector law takes precedence for the data and entities it directly governs.
- PRECEDENCE-002: State privacy law applies to residual personal data not exempted or preempted.
- PRECEDENCE-003: Contractual SOC 2, NIST, and FedRAMP commitments may exceed statutory minimums; the pack treats them as pack overlays only after tenant or product activation.
- PRECEDENCE-004: HIPAA PHI remains PHI when health data is handled by a covered entity or business associate; state consumer-health overlays may also apply to non-HIPAA consumer health data.
- PRECEDENCE-005: GLBA NPI remains GLBA data when held by covered financial institutions; state privacy exemptions may remove the entity or the data depending on statute.
- PRECEDENCE-006: FERPA education records remain education records when maintained by covered educational agencies or institutions.
- PRECEDENCE-007: COPPA parental-consent duties apply before generic state child-sensitive data rules for under-13 online-service processing.
- PRECEDENCE-008: FCRA consumer-reporting duties apply to consumer reports, users, furnishers, and consumer reporting agencies before generic profiling language.
- PRECEDENCE-009: ECOA fair-lending duties apply to credit determinations even when a state privacy law also classifies the data as sensitive.
- PRECEDENCE-010: SOX preservation and certification duties apply to public-company financial-reporting evidence regardless of tenant retention defaults.
- PRECEDENCE-011: ITAR export restrictions outrank ordinary cross-border transfer permissions because export authorization is actor and nationality sensitive.
- PRECEDENCE-012: ADA accessibility duties apply to digital surfaces even when privacy overlays allow data processing.
- PRECEDENCE-013: EEOC UGESP validation duties apply to employment-selection procedures even when AEDT notices are satisfied.
- PRECEDENCE-014: NYC LL 144 applies locally to AEDT hiring/promotion use; it does not replace Title VII or UGESP.
- PRECEDENCE-015: Colorado SB 24-205 high-risk AI duties apply to Colorado consumers and consequential decisions; they do not replace ECOA or Title VII.
- PRECEDENCE-016: California AB-331 is tracked as design guidance only unless a successor is enacted; no binding denial is based solely on AB-331.
- PRECEDENCE-017: The pack chooses the stricter consent, notice, opt-out, retention, audit, or evidence duty when two valid overlays apply and neither is preempted.
- PRECEDENCE-018: The pack blocks processing when legal basis is ambiguous and escalation is required.
- PRECEDENCE-019: The pack records the chosen controlling authority in every high-risk audit event.
- PRECEDENCE-020: The pack requires a per-tenant precedence resolution record for each compound regulated workflow.

## Activated Microservices

- MS-001: `identity` receives age, guardian, authentication, role, and lawful-agent verification deltas.
- MS-002: `tenancy` receives pack activation, state residency, regulated entity, and controller/processor role deltas.
- MS-003: `governance` receives policy-resolution, authority-basis, DPIA, DPA, BAA, and data-protection-assessment deltas.
- MS-004: `audit-chain` receives US pack audit event classes, retention labels, legal-hold flags, and regulator evidence pointers.
- MS-005: `workflow` receives DSAR, privacy appeal, HIPAA incident, SOX evidence, bias audit, and FedRAMP POA&M workflows.
- MS-006: `ontology` receives US regulated-domain nodes, data-class relations, authority edges, and high-risk decision graphs.
- MS-007: `accounting` receives SOX, GLBA, FCRA, ECOA, close, journal, certification, and ICFR scoping deltas.
- MS-008: `payroll` receives EEOC, Title VII, UGESP, ADA accommodation, wage-record, tax-record, and employment-decision deltas.
- MS-009: `hr` receives selection procedure, adverse impact, AEDT, accommodation, personnel file, and retention deltas.
- MS-010: `mail` receives legal hold, eDiscovery, PHI messaging, FERPA disclosure, GLBA notice, and SOX evidence routing deltas.
- MS-011: `messenger` receives regulated chat, PHI/FERPA redaction, child interaction, and audit-preserved communication deltas.
- MS-012: `calendar` receives ADA accommodation scheduling, healthcare appointment metadata, education event privacy, and audit labels.
- MS-013: `network` receives professional identity, recruiting, ad-targeting opt-out, and consumer profiling deltas.
- MS-014: `social` receives child-directed service, targeted advertising, sale/share opt-out, and sensitive-category inference deltas.
- MS-015: `shorts` receives child privacy, biometric/media metadata, targeted advertising, and COPPA age-gate deltas.
- MS-016: `foundry` receives regulated AI development, model evaluation, bias test, evidence pack, and vendor-risk deltas.
- MS-017: `cloud-iac` receives FedRAMP, NIST, ITAR residency, BYOK, and data boundary infrastructure deltas.
- MS-018: `cloud-k8s` receives control-plane hardening, audit log retention, vulnerability scan, and FedRAMP baseline deltas.
- MS-019: `cloud-secrets` receives HIPAA, GLBA, SOX, FedRAMP, and ITAR secret-handling deltas.
- MS-020: `ops-dashboard-control-center` receives regulator evidence, incident clock, DSAR queue, bias audit, and certification dashboard deltas.
- MS-021: `forms` receives consent collection, DSAR intake, FERPA release, HIPAA authorization, and appeal intake deltas where available.
- MS-022: `drive` receives regulated-document classification, retention, legal hold, de-identification evidence, and secure export deltas where available.
- MS-023: `docs` receives document-class markings, source citation references, statutory update tracking, and compliance-pack publishing deltas where available.
- MS-024: `sheets` receives SOX spreadsheet control, model-risk evidence, GLBA data extraction, and audit-control workbook deltas where available.
- MS-025: `slides` receives training evidence, accessibility, compliance presentation export, and board certification evidence deltas where available.
- MS-026: `meet` receives PHI meeting metadata controls, ADA accommodation, recording retention, and audit log deltas where available.
- MS-027: `recordings` receives consent, PHI/education metadata, retention, de-identification, and legal hold deltas where available.
- MS-028: `notes` receives regulated note classification, PHI/FERPA/FCRA flags, retention, and redaction deltas where available.
- MS-029: `tasks` receives compliance task ownership, remediation due dates, DSAR fulfillment steps, and POA&M linkage deltas where available.
- MS-030: `translate` receives language access, ADA effective communication, notice translation, and sensitive-data localization deltas where available.

## Authority-Driven Activation Matrix

- ACT-001: Activate state privacy overlays when `consumer_residency_state` matches CA, CO, CT, VA, UT, TX, IA, IN, TN, MT, OR, DE, NJ, or NH.
- ACT-002: Activate HIPAA when `hipaa_role` is `covered_entity`, `business_associate`, or `subcontractor_business_associate`.
- ACT-003: Activate GLBA when `glba_financial_institution` is true or data class is `NPI_US_GLBA`.
- ACT-004: Activate FERPA when `education_agency_or_institution` is true or data class is `EDU_RECORD_US_FERPA`.
- ACT-005: Activate COPPA when service is child-directed or actual knowledge indicates a user under 13.
- ACT-006: Activate FCRA when data is used in a consumer report, furnished to a CRA, received from a CRA, or used for covered eligibility decisions.
- ACT-007: Activate ECOA when data is used for credit application, underwriting, pricing, adverse action, or credit-account servicing decisions.
- ACT-008: Activate SOX when workflows support issuer financial reporting, disclosure controls, ICFR, audit evidence, certification, or record preservation.
- ACT-009: Activate ADA when surfaces are public accommodations, state/local government services, employment accommodation workflows, or accessibility-critical interfaces.
- ACT-010: Activate SOC 2 when a tenant contract or product assurance boundary references Trust Services Criteria.
- ACT-011: Activate UGESP when selection procedures affect hire, transfer, promotion, retention, compensation, or other employment decisions.
- ACT-012: Activate ITAR when data class includes technical data, defense article metadata, defense service content, or USML jurisdiction references.
- ACT-013: Activate NIST 800-53 when tenant security baseline, federal agency contract, or internal control mapping requires SP 800-53 Rev. 5.
- ACT-014: Activate FedRAMP when a cloud-service offering is being authorized, assessed, continuously monitored, or packaged for agency use.
- ACT-015: Activate NYC LL 144 when AEDT is used to screen candidates or evaluate employees for hire or promotion in New York City.
- ACT-016: Activate Colorado SB 24-205 when high-risk AI affects Colorado consumers in covered consequential decisions.
- ACT-017: Track California AB-331 as nonbinding design guidance for ADT notices and impact assessments unless a binding successor exists.
- ACT-018: Activate compound mode when more than one authority applies to one data object or decision event.
- ACT-019: Deny activation if tenant lacks lawful basis fields required by the selected overlay.
- ACT-020: Deny promotion from advisory to enforced pack until executable Cedar and schema artifacts are separately claimed, verified, and promoted.

## Activated Cedar Policies

- CEDAR-001: `us-pack-default-deny-unclassified-regulated-data` denies processing when US regulated data lacks data class, purpose, actor role, jurisdiction, and authority basis.
- CEDAR-002: `us-pack-state-privacy-sale-share-optout` denies sale or targeted-ad processing after valid opt-out.
- CEDAR-003: `us-pack-state-privacy-sensitive-consent` requires opt-in consent where state law requires sensitive-data consent.
- CEDAR-004: `us-pack-state-privacy-sensitive-notice-optout` requires notice plus opt-out for Utah and Iowa sensitive-data handling.
- CEDAR-005: `us-pack-california-cpra-limit-sensitive-use` enforces California limit-use choice for sensitive personal information.
- CEDAR-006: `us-pack-california-under16-sale-share-optin` requires opt-in sale/share consent for known California consumers under 16.
- CEDAR-007: `us-pack-connecticut-teen-targeting-optin` requires opt-in sale or targeted-ad processing for Connecticut consumers under 16.
- CEDAR-008: `us-pack-universal-optout-signal-honor` honors GPC or comparable opt-out preference signals where mandatory.
- CEDAR-009: `us-pack-state-privacy-dsar-access` permits authenticated access requests subject to exemption and verification rules.
- CEDAR-010: `us-pack-state-privacy-dsar-delete` permits deletion requests unless legal hold, statutory retention, or exception blocks deletion.
- CEDAR-011: `us-pack-state-privacy-dsar-correct` permits correction requests with provenance-preserving amendment records.
- CEDAR-012: `us-pack-state-privacy-portability` permits portable-copy exports only through identity-verified channels.
- CEDAR-013: `us-pack-state-privacy-appeal` requires appeal workflow availability when denial is issued by a covered controller.
- CEDAR-014: `us-pack-hipaa-phi-minimum-necessary` denies PHI use beyond the minimum necessary standard except treatment and other recognized exceptions.
- CEDAR-015: `us-pack-hipaa-baa-required` denies PHI disclosure to service providers lacking a valid BAA.
- CEDAR-016: `us-pack-hipaa-safe-harbor-deid` permits Safe Harbor output only after required identifier removal and no actual re-identification knowledge.
- CEDAR-017: `us-pack-hipaa-expert-determination-deid` permits expert-determination output only with stored expert basis and residual-risk statement.
- CEDAR-018: `us-pack-hipaa-breach-clock` starts breach-assessment and notification deadlines on confirmed unauthorized PHI access.
- CEDAR-019: `us-pack-glba-npi-safeguards` denies NPI processing outside an active information-security program.
- CEDAR-020: `us-pack-glba-privacy-notice-sharing` requires GLBA notice and opt-out treatment for nonaffiliated third-party sharing where applicable.
- CEDAR-021: `us-pack-ferpa-education-record-disclosure` denies disclosure of education records unless consent or a 34 CFR 99.31 exception applies.
- CEDAR-022: `us-pack-ferpa-directory-info` permits directory information only after annual notice and opt-out state are satisfied.
- CEDAR-023: `us-pack-coppa-parental-consent` denies under-13 online collection without verifiable parental consent unless a narrow exception applies.
- CEDAR-024: `us-pack-coppa-data-minimization` denies child-data retention beyond the purpose and required safety window.
- CEDAR-025: `us-pack-fcra-permissible-purpose` denies consumer-report access without a permissible purpose.
- CEDAR-026: `us-pack-fcra-adverse-action-notice` requires adverse-action notice evidence when FCRA report data materially affects denial or pricing.
- CEDAR-027: `us-pack-ecoa-prohibited-basis` denies credit-decision features that directly encode protected characteristics unless used for permitted monitoring or special-purpose credit program logic.
- CEDAR-028: `us-pack-ecoa-adverse-action` requires timely adverse-action reasons for covered credit decisions.
- CEDAR-029: `us-pack-sox-evidence-retention` denies deletion or mutation of SOX-relevant records inside statutory or litigation-preservation windows.
- CEDAR-030: `us-pack-sox-certification-chain` requires CFO/CEO certification workflow evidence for issuer reporting flows.
- CEDAR-031: `us-pack-icfr-change-control` denies financial-reporting control changes without owner approval, test evidence, and segregation-of-duties review.
- CEDAR-032: `us-pack-pcaob-as2201-topdown-scope` requires top-down risk scoping for ICFR audit evidence claims.
- CEDAR-033: `us-pack-ada-accessibility-accommodation` requires accessible alternative or accommodation handling for covered digital and employment workflows.
- CEDAR-034: `us-pack-soc2-trust-service-control-map` requires customer commitments to map to one or more Trust Services Criteria.
- CEDAR-035: `us-pack-ugesp-adverse-impact-detect` requires four-fifths adverse-impact checks for employment selection procedures.
- CEDAR-036: `us-pack-ugesp-validation-required` requires validity evidence when adverse impact is detected and the selection procedure remains in use.
- CEDAR-037: `us-pack-nyc-ll144-aedt-bias-audit` denies NYC AEDT use when no current independent bias audit is published.
- CEDAR-038: `us-pack-colorado-high-risk-ai-care` requires reasonable-care evidence for Colorado high-risk AI systems.
- CEDAR-039: `us-pack-ca-ab331-advisory-impact-assessment` records advisory ADT impact assessment evidence without enforcing binding California denial.
- CEDAR-040: `us-pack-itar-technical-data-export` denies technical data export to foreign persons or locations without authorization or exemption.
- CEDAR-041: `us-pack-itar-foreign-person-access` denies support access when nationality, location, and authorization are not resolved.
- CEDAR-042: `us-pack-nist80053-control-evidence` requires control implementation evidence for activated SP 800-53 control families.
- CEDAR-043: `us-pack-fedramp-baseline-evidence` requires FedRAMP baseline, SSP, SAP, SAR, POA&M, and continuous-monitoring evidence when activated.
- CEDAR-044: `us-pack-fedramp-vulnerability-remediation` applies FedRAMP vulnerability remediation clocks to cloud-service findings.
- CEDAR-045: `us-pack-compound-authority-resolution` requires explicit controlling-authority resolution for compound regulated actions.
- CEDAR-046: `us-pack-regulator-export-minimization` restricts regulator evidence exports to scoped, logged, review-approved bundles.
- CEDAR-047: `us-pack-rights-agent-verification` permits authorized agents only after statutory and identity verification checks.
- CEDAR-048: `us-pack-retention-vs-delete-precedence` denies deletion when SOX, HIPAA, GLBA, FERPA, legal hold, or security retention overrides a privacy deletion request.
- CEDAR-049: `us-pack-consent-revocation-propagation` requires revocation propagation to processors and downstream stores.
- CEDAR-050: `us-pack-processor-contract-required` denies processor disclosures lacking DPA, BAA, processor agreement, or service-provider restrictions as applicable.
- CEDAR-051: `us-pack-sensitive-inference-guard` denies inference of sensitive categories for ad targeting, employment, credit, or housing unless permitted and tested.
- CEDAR-052: `us-pack-health-data-nonhipaa-overlay` applies state consumer-health protections where health data falls outside HIPAA.
- CEDAR-053: `us-pack-biometric-processing` requires biometric notice, consent, retention, and purpose limits according to applicable authority.
- CEDAR-054: `us-pack-geolocation-processing` requires precise-geolocation classification and state-sensitive-data handling.
- CEDAR-055: `us-pack-neural-data-california` classifies California neural data as sensitive personal information under current CCPA text.
- CEDAR-056: `us-pack-education-health-boundary` resolves HIPAA/FERPA overlap for student health records.
- CEDAR-057: `us-pack-credit-employment-boundary` resolves FCRA/ECOA/Title VII overlap for background checks and credit eligibility.
- CEDAR-058: `us-pack-audit-chain-worm-retention` requires WORM-style audit preservation for regulated decisions and exceptions.
- CEDAR-059: `us-pack-emergency-breakglass` permits emergency access only with purpose, scope, duration, reviewer, and post-access audit.
- CEDAR-060: `us-pack-pack-deactivation-deny-unsafe` denies deactivation while regulated data, active workflows, legal holds, or open incidents remain unresolved.

## Data Model Deltas

- DATA-001: Add `US_PERSONAL_DATA` for generic US personal data covered by state privacy laws.
- DATA-002: Add `US_SENSITIVE_DATA` for sensitive state privacy categories requiring opt-in, limit-use, or opt-out handling.
- DATA-003: Add `US_CONSUMER_HEALTH_DATA` for non-HIPAA consumer health data under state overlays.
- DATA-004: Add `PHI_US_HIPAA` for protected health information held or transmitted by covered entities or business associates.
- DATA-005: Add `EHI_US_HIPAA` for electronic protected health information requiring Security Rule safeguards.
- DATA-006: Add `NPI_US_GLBA` for nonpublic personal information handled by financial institutions.
- DATA-007: Add `EDU_RECORD_US_FERPA` for FERPA education records.
- DATA-008: Add `CHILD_PI_US_COPPA` for COPPA-covered personal information from children under 13.
- DATA-009: Add `CONSUMER_REPORT_US_FCRA` for FCRA consumer-report data.
- DATA-010: Add `CREDIT_DECISION_US_ECOA` for ECOA-covered credit-decision inputs, outputs, and reasons.
- DATA-011: Add `ICFR_EVIDENCE_US_SOX` for internal-control evidence tied to financial reporting.
- DATA-012: Add `FINANCIAL_REPORT_RECORD_US_SOX` for issuer reporting records, workpapers, approvals, and certifications.
- DATA-013: Add `ACCESSIBILITY_REQUEST_US_ADA` for accommodation and accessibility support records.
- DATA-014: Add `SELECTION_PROCEDURE_US_UGESP` for tests, screens, scoring, interview filters, and eligibility rubrics.
- DATA-015: Add `AEDT_OUTPUT_US_NYC_LL144` for NYC automated employment decision tool outputs.
- DATA-016: Add `HIGH_RISK_AI_US_CO_SB205` for Colorado high-risk AI system inputs, outputs, risk analyses, and notices.
- DATA-017: Add `ADT_ASSESSMENT_US_CA_AB331_ADVISORY` for nonbinding California AB-331 assessment records.
- DATA-018: Add `TECHNICAL_DATA_US_ITAR` for ITAR-controlled technical data.
- DATA-019: Add `CONTROL_EVIDENCE_US_NIST80053` for SP 800-53 implementation and assessment records.
- DATA-020: Add `AUTHORIZATION_PACKAGE_US_FEDRAMP` for FedRAMP SSP, SAP, SAR, POA&M, continuous monitoring, and boundary artifacts.
- DATA-021: Add `consumer_residency_state` as a nullable two-letter state code with provenance.
- DATA-022: Add `authority_basis_refs` as a nonempty list on regulated processing events.
- DATA-023: Add `controller_processor_role` with values `controller`, `processor`, `service_provider`, `contractor`, `business_associate`, and `subcontractor`.
- DATA-024: Add `regulated_entity_flags` for HIPAA, GLBA, FERPA, FCRA, ECOA, SOX, ITAR, FedRAMP, and SOC 2 activation.
- DATA-025: Add `sensitive_category_refs` for race, ethnicity, religion, health, sexual orientation, citizenship, immigration, genetic, biometric, precise geolocation, child, neural, financial credential, and union categories.
- DATA-026: Add `minor_age_band` with values `unknown`, `under_13`, `13_to_15`, `16_to_17`, and `adult`.
- DATA-027: Add `guardian_consent_ref` for COPPA and child-sensitive data processing.
- DATA-028: Add `sale_share_optout_status` for state privacy opt-out handling.
- DATA-029: Add `targeted_ad_optout_status` for targeted advertising opt-out handling.
- DATA-030: Add `profiling_optout_status` for legal-or-similarly-significant profiling.
- DATA-031: Add `universal_optout_signal_seen_at` for GPC or comparable signal receipt.
- DATA-032: Add `sensitive_processing_consent_ref` for opt-in sensitive processing.
- DATA-033: Add `utah_iowa_sensitive_notice_ref` for notice-plus-opt-out sensitive data flows.
- DATA-034: Add `limit_sensitive_use_status` for California CPRA limit-use choices.
- DATA-035: Add `dsar_request_id` across access, deletion, correction, portability, and appeal workflows.
- DATA-036: Add `dsar_denial_reason_code` to preserve statutory exception reasoning.
- DATA-037: Add `appeal_deadline_at` for state privacy appeals.
- DATA-038: Add `baa_ref` for HIPAA business-associate agreements.
- DATA-039: Add `minimum_necessary_scope` for PHI use and disclosure decisions.
- DATA-040: Add `deidentification_method` with values `safe_harbor`, `expert_determination`, `limited_data_set`, and `not_deidentified`.
- DATA-041: Add `hipaa_authorization_ref` for uses requiring individual authorization.
- DATA-042: Add `glba_privacy_notice_version` for NPI notice evidence.
- DATA-043: Add `glba_safeguards_program_ref` for information-security program linkage.
- DATA-044: Add `ferpa_consent_ref` for education-record disclosures.
- DATA-045: Add `ferpa_exception_code` for 34 CFR 99.31 disclosures.
- DATA-046: Add `coppa_verifiable_parental_consent_method` for consent evidence.
- DATA-047: Add `fcra_permissible_purpose_code` for consumer-report access.
- DATA-048: Add `adverse_action_notice_ref` for FCRA and ECOA decision notices.
- DATA-049: Add `ecoa_prohibited_basis_monitoring_flag` to distinguish monitoring from decision use.
- DATA-050: Add `sox_control_id` for ICFR control references.
- DATA-051: Add `sox_evidence_retention_until` for preservation windows.
- DATA-052: Add `icfr_key_report_flag` for reports used in financial controls.
- DATA-053: Add `pcaob_as2201_scope_ref` for top-down risk scoping.
- DATA-054: Add `ada_accessibility_issue_type` for keyboard, screen-reader, caption, contrast, timing, and alternative-format issues.
- DATA-055: Add `soc2_tsc_category` for security, availability, processing_integrity, confidentiality, and privacy.
- DATA-056: Add `ugesp_selection_rate_group` for adverse-impact calculations.
- DATA-057: Add `ugesp_validation_study_ref` for criterion, content, or construct validity evidence.
- DATA-058: Add `aedt_bias_audit_ref` for NYC LL 144 audits.
- DATA-059: Add `high_risk_ai_impact_assessment_ref` for Colorado SB 205 and advisory ADT records.
- DATA-060: Add `itar_authorization_ref` for export license, agreement, exemption, or jurisdiction determination.
- DATA-061: Add `foreign_person_access_status` for ITAR access review.
- DATA-062: Add `nist80053_control_family` for AC, AU, CA, CM, CP, IA, IR, MA, MP, PE, PL, PM, PS, PT, RA, SA, SC, SI, and SR.
- DATA-063: Add `fedramp_impact_level` for LI-SaaS, Low, Moderate, or High.
- DATA-064: Add `fedramp_3pao_assessment_ref` for authorization package linkage.
- DATA-065: Add `compound_authority_resolution_id` when two or more overlays apply.
- DATA-066: Add `regulated_processor_contract_ref` for DPAs, service-provider terms, BAAs, and processor agreements.
- DATA-067: Add `legal_hold_ref` to override deletion and retention requests.
- DATA-068: Add `breakglass_access_ref` for emergency access events.
- DATA-069: Add `regulator_export_bundle_ref` for evidence submitted externally.
- DATA-070: Add `pack_activation_state` with values `not_applicable`, `advisory`, `active`, `suspended`, and `retired`.

## API Contract Deltas

- API-001: `POST /privacy/us/activate` records tenant activation, jurisdiction basis, sector flags, and owner attestation.
- API-002: `GET /privacy/us/status` returns active overlays, pack version, state coverage, and enforcement posture.
- API-003: `POST /privacy/us/dsar/access` opens a state privacy access request.
- API-004: `POST /privacy/us/dsar/delete` opens a deletion request with retention-precedence evaluation.
- API-005: `POST /privacy/us/dsar/correct` opens a correction request with provenance-safe amendment semantics.
- API-006: `POST /privacy/us/dsar/portable-copy` opens a portable-copy export with identity verification.
- API-007: `POST /privacy/us/dsar/appeal` opens an appeal for denied consumer-rights requests.
- API-008: `POST /privacy/us/opt-out/sale-share` records sale/share opt-out.
- API-009: `POST /privacy/us/opt-out/targeted-advertising` records targeted-advertising opt-out.
- API-010: `POST /privacy/us/opt-out/profiling` records profiling opt-out for legal or similarly significant effects.
- API-011: `POST /privacy/us/opt-out/preference-signal` records universal opt-out preference signal detection.
- API-012: `POST /privacy/us/consent/sensitive` records sensitive-data consent.
- API-013: `POST /privacy/us/consent/minor` records guardian or teen opt-in consent.
- API-014: `POST /privacy/us/consent/revoke` revokes consent and emits propagation tasks.
- API-015: `GET /privacy/us/notices/{notice_id}` returns applicable privacy, GLBA, COPPA, FERPA, or HIPAA notice content.
- API-016: `POST /hipaa/us/phi/use` records PHI use, minimum-necessary scope, and permitted purpose.
- API-017: `POST /hipaa/us/phi/disclose` records PHI disclosure and BAA or authorization basis.
- API-018: `POST /hipaa/us/deidentify/safe-harbor` records Safe Harbor de-identification evidence.
- API-019: `POST /hipaa/us/deidentify/expert` records expert-determination evidence.
- API-020: `POST /hipaa/us/breach/assess` opens PHI breach assessment.
- API-021: `POST /glba/us/npi/process` records NPI processing tied to safeguards-program evidence.
- API-022: `POST /glba/us/privacy-notice/deliver` records GLBA notice delivery.
- API-023: `POST /ferpa/us/disclosure` records education-record disclosure basis.
- API-024: `POST /ferpa/us/directory-info/optout` records directory information opt-out.
- API-025: `POST /coppa/us/parental-consent` records verifiable parental consent.
- API-026: `POST /coppa/us/delete-child-data` opens child-data deletion.
- API-027: `POST /fcra/us/report/access` records consumer-report access with permissible purpose.
- API-028: `POST /fcra/us/adverse-action` records adverse-action notice generation.
- API-029: `POST /ecoa/us/credit-decision` records credit-decision basis and protected-basis guardrails.
- API-030: `POST /ecoa/us/adverse-action` records ECOA adverse-action notice reasons.
- API-031: `POST /sox/us/control/change` records ICFR control changes.
- API-032: `POST /sox/us/evidence/preserve` records SOX evidence preservation.
- API-033: `POST /sox/us/certification` records Section 302 or 906 certification workflow.
- API-034: `POST /sox/us/icfr/scope` records Section 404 and AS 2201 scoping.
- API-035: `POST /ada/us/accessibility-issue` records accessibility issue and accommodation path.
- API-036: `POST /soc2/us/control-map` records Trust Services Criteria mapping.
- API-037: `POST /ugesp/us/selection-procedure` registers a selection procedure.
- API-038: `POST /ugesp/us/adverse-impact-check` records four-fifths analysis.
- API-039: `POST /aedt/us/nyc-ll144/bias-audit` registers AEDT bias-audit evidence.
- API-040: `POST /ai/us/co-sb205/high-risk-system` registers Colorado high-risk AI assessment.
- API-041: `POST /ai/us/ca-ab331/advisory-assessment` records advisory automated decision tool assessment.
- API-042: `POST /itar/us/export-review` evaluates technical data export, foreign person access, and authorization.
- API-043: `POST /nist/us/800-53/control-evidence` records SP 800-53 control evidence.
- API-044: `POST /fedramp/us/package` records authorization package artifacts.
- API-045: `POST /fedramp/us/conmon/finding` records continuous-monitoring finding and remediation timer.
- API-046: `POST /privacy/us/authority-resolution` records compound authority precedence.
- API-047: `GET /privacy/us/classification/{object_id}` returns US data-class labels and active restrictions.
- API-048: `POST /privacy/us/processor-contract` records DPA, BAA, service-provider, or processor terms.
- API-049: `POST /privacy/us/regulator-export` creates a scoped regulator evidence bundle.
- API-050: `POST /privacy/us/deactivate` begins safe deactivation and denies completion while regulated obligations remain.

## Audit Event Additions

- AUDIT-001: `UsPackActivated` records tenant, version, scope, sector flags, and activating authority.
- AUDIT-002: `UsPackActivationDenied` records missing scope evidence or conflicting tenant state.
- AUDIT-003: `UsStatePrivacyRequestOpened` records DSAR type, state, authenticated actor, and deadline.
- AUDIT-004: `UsStatePrivacyRequestDenied` records statutory exception and appeal instructions.
- AUDIT-005: `UsStatePrivacyRequestFulfilled` records fulfillment evidence and completion time.
- AUDIT-006: `UsPrivacyAppealOpened` records appeal deadline and reviewing role.
- AUDIT-007: `UsPrivacyAppealResolved` records outcome, reason, and regulator complaint instructions.
- AUDIT-008: `UsOptOutRecorded` records sale, share, targeted-ad, profiling, or preference-signal opt-out.
- AUDIT-009: `UsSensitiveConsentRecorded` records category, purpose, duration, and consent collector.
- AUDIT-010: `UsConsentRevoked` records downstream propagation tasks.
- AUDIT-011: `UsMinorConsentRecorded` records guardian, teen, or COPPA consent path.
- AUDIT-012: `UsCaliforniaLimitSensitiveUseRecorded` records CPRA limit-use action.
- AUDIT-013: `UsUniversalOptOutSignalHonored` records signal source and covered purposes.
- AUDIT-014: `UsHipaaPhiUseApproved` records permitted purpose and minimum-necessary scope.
- AUDIT-015: `UsHipaaPhiUseDenied` records missing authorization, BAA, or purpose basis.
- AUDIT-016: `UsHipaaBaaLinked` records BAA version and parties.
- AUDIT-017: `UsHipaaDeidentificationCompleted` records Safe Harbor or expert determination method.
- AUDIT-018: `UsHipaaBreachAssessmentOpened` records incident clock and affected data classes.
- AUDIT-019: `UsGlbaNpiProcessed` records safeguards program and notice basis.
- AUDIT-020: `UsGlbaNoticeDelivered` records notice version and sharing choices.
- AUDIT-021: `UsFerpaDisclosureApproved` records consent or exception basis.
- AUDIT-022: `UsFerpaDisclosureDenied` records missing consent or invalid exception.
- AUDIT-023: `UsCoppaParentalConsentVerified` records method and child data purpose.
- AUDIT-024: `UsCoppaDataDeleted` records deletion target and retention exception.
- AUDIT-025: `UsFcraReportAccessed` records permissible purpose.
- AUDIT-026: `UsFcraAdverseActionNoticeIssued` records notice reference and decision context.
- AUDIT-027: `UsEcoaDecisionRecorded` records prohibited-basis guardrail state.
- AUDIT-028: `UsEcoaAdverseActionNoticeIssued` records principal reasons and delivery time.
- AUDIT-029: `UsSoxControlChanged` records approval, test evidence, and SOD review.
- AUDIT-030: `UsSoxEvidencePreserved` records retention deadline and legal-hold state.
- AUDIT-031: `UsSoxCertificationCompleted` records Section 302 or 906 signer chain.
- AUDIT-032: `UsIcfrScopeApproved` records Section 404 and AS 2201 scoping rationale.
- AUDIT-033: `UsAdaAccessibilityIssueOpened` records barrier type and accommodation route.
- AUDIT-034: `UsSoc2ControlMapped` records Trust Services Criteria category and customer commitment.
- AUDIT-035: `UsUgespAdverseImpactChecked` records selection rates and four-fifths result.
- AUDIT-036: `UsUgespValidationEvidenceLinked` records validation study reference.
- AUDIT-037: `UsNycLl144BiasAuditPublished` records AEDT, audit date, and public result link.
- AUDIT-038: `UsColoradoHighRiskAiAssessmentRecorded` records consequential decision category.
- AUDIT-039: `UsCaliforniaAb331AdvisoryAssessmentRecorded` records nonbinding ADT review evidence.
- AUDIT-040: `UsItarExportReviewApproved` records authorization basis.
- AUDIT-041: `UsItarExportReviewDenied` records foreign person or destination blocker.
- AUDIT-042: `UsNist80053ControlEvidenceRecorded` records control family and assessment status.
- AUDIT-043: `UsFedrampPackageArtifactRecorded` records SSP, SAP, SAR, POA&M, or ConMon artifact.
- AUDIT-044: `UsFedrampFindingTimerStarted` records severity and remediation deadline.
- AUDIT-045: `UsCompoundAuthorityResolved` records controlling authority and rejected alternatives.
- AUDIT-046: `UsRegulatorExportCreated` records export scope, reviewer, destination, and hash.
- AUDIT-047: `UsProcessorContractLinked` records agreement class and processing instructions.
- AUDIT-048: `UsBreakglassAccessUsed` records emergency basis and post-access review.
- AUDIT-049: `UsRetentionOverrideApplied` records deletion override authority.
- AUDIT-050: `UsPackDeactivationBlocked` records active obligations preventing deactivation.

## Failure Modes

- FAIL-001: A tenant activates the US pack without choosing covered states; remediation is to set residency scope or mark sector-only activation.
- FAIL-002: A controller treats all state privacy laws as opt-in; remediation is to distinguish Utah and Iowa sensitive-data notice-plus-opt-out posture.
- FAIL-003: A controller treats all state privacy laws as opt-out; remediation is to enforce consent for sensitive data where required.
- FAIL-004: A California workflow ignores limit-use of sensitive personal information; remediation is CPRA limit-use propagation.
- FAIL-005: A Connecticut teen-targeting workflow uses opt-out instead of opt-in; remediation is under-16 sale/targeted-ad consent.
- FAIL-006: A processor contract is missing; remediation is to block disclosure until processor terms are linked.
- FAIL-007: A DSAR delete request deletes SOX evidence; remediation is retention-precedence evaluation.
- FAIL-008: A DSAR access response exposes another consumer's data; remediation is record-level identity filtering and export review.
- FAIL-009: A universal opt-out preference signal is logged but not honored; remediation is opt-out state fanout.
- FAIL-010: A HIPAA workflow applies generic privacy consent instead of permitted-use analysis; remediation is PHI purpose classification.
- FAIL-011: A HIPAA service provider receives PHI without BAA; remediation is deny and contract linkage.
- FAIL-012: A de-identification job claims Safe Harbor while retaining a listed identifier; remediation is block export and rerun de-id.
- FAIL-013: A GLBA tenant stores NPI without safeguards program linkage; remediation is safeguards evidence gate.
- FAIL-014: A FERPA workflow treats student records as ordinary tenant records; remediation is education-record label and disclosure exception handling.
- FAIL-015: A COPPA service collects under-13 persistent identifiers without parental consent or exception; remediation is consent or deletion.
- FAIL-016: An FCRA report is used for eligibility without permissible purpose; remediation is deny and audit.
- FAIL-017: An ECOA model uses prohibited-basis proxy variables without monitoring isolation; remediation is feature suppression or fair-lending review.
- FAIL-018: SOX control evidence is mutable without approval; remediation is WORM preservation and control owner approval.
- FAIL-019: ICFR scoping omits a key report used in financial close; remediation is AS 2201 top-down scope update.
- FAIL-020: Accessibility defects are tracked as UX bugs only; remediation is ADA issue classification and accommodation path.
- FAIL-021: SOC 2 controls are asserted without mapping to customer commitments; remediation is Trust Services Criteria mapping.
- FAIL-022: UGESP four-fifths analysis is skipped for an automated screen; remediation is selection-rate computation.
- FAIL-023: NYC AEDT is used after audit age exceeds one year; remediation is deny use until audit refresh.
- FAIL-024: Colorado high-risk AI is deployed without impact assessment; remediation is reasonable-care evidence package.
- FAIL-025: California AB-331 advisory material is enforced as binding law; remediation is advisory flag and no statutory denial.
- FAIL-026: ITAR technical data is exposed to offshore support; remediation is nationality/location access review.
- FAIL-027: NIST 800-53 evidence is claimed at framework level without control-family evidence; remediation is per-control evidence mapping.
- FAIL-028: FedRAMP package omits POA&M linkage; remediation is package completeness gate.
- FAIL-029: A regulator export contains unscoped unrelated tenant data; remediation is export minimization and hash review.
- FAIL-030: Pack deactivation proceeds while open incidents remain; remediation is block deactivation.
- FAIL-031: State law thresholds are applied to sector data that is exempt; remediation is entity/data exemption classification.
- FAIL-032: Sector exemptions are over-applied to all tenant data; remediation is object-level classification.
- FAIL-033: Child age is unknown but targeted advertising continues; remediation is age-band uncertainty handling.
- FAIL-034: Consent withdrawal does not propagate to processors; remediation is revocation fanout tracking.
- FAIL-035: Sensitive inference is created by model outputs but absent in input labels; remediation is output classification.
- FAIL-036: High-risk AI logs omit decision category; remediation is consequential-decision taxonomy mapping.
- FAIL-037: Credit adverse-action reasons are too generic; remediation is principal-reason validation.
- FAIL-038: HIPAA breach clocks start at public confirmation instead of discovery; remediation is internal incident timestamp discipline.
- FAIL-039: FERPA directory information lacks annual notice evidence; remediation is block directory disclosure.
- FAIL-040: GLBA privacy notice is delivered but not versioned; remediation is notice version audit.
- FAIL-041: SOX certification chain omits disclosure-control evidence; remediation is Section 302 evidence bundle.
- FAIL-042: PCAOB AS 2201 scoping does not document top-down risk; remediation is scope rationale artifact.
- FAIL-043: ADA accommodation request is stored without confidentiality controls; remediation is sensitive HR data classification.
- FAIL-044: SOC 2 privacy criterion is activated without privacy notice consistency; remediation is service commitment review.
- FAIL-045: ITAR export authorization is assumed from US hosting; remediation is actor and destination review.
- FAIL-046: FedRAMP continuous monitoring findings lack remediation deadlines; remediation is finding clock policy.
- FAIL-047: NIST control inheritance is claimed from a provider without shared-responsibility boundary; remediation is inheritance evidence.
- FAIL-048: State appeal deadlines are not tracked; remediation is per-state appeal SLA.
- FAIL-049: Universal opt-out conflicts with loyalty-program preference; remediation is statute-specific precedence handling.
- FAIL-050: A compound workflow chooses the least restrictive overlay; remediation is strongest-duty resolver.

## Worked Examples

- EXAMPLE-001: California marketing tenant receives GPC; system records `UsUniversalOptOutSignalHonored` and denies sale/share and targeted advertising.
- EXAMPLE-002: California user limits sensitive use; analytics can use sensitive data only for permitted operational purposes and must not infer characteristics for ads.
- EXAMPLE-003: Connecticut consumer age 15 is targeted for ads; system denies until opt-in consent is recorded.
- EXAMPLE-004: Utah controller processes precise geolocation; system requires clear notice and opt-out opportunity instead of treating it as consent-only.
- EXAMPLE-005: Iowa controller processes sensitive health preference outside HIPAA; system records notice-plus-opt-out and blocks sale if opt-out exists.
- EXAMPLE-006: Texas controller receives deletion request; system evaluates legal hold, GLBA, FCRA, SOX, and security exceptions before deletion.
- EXAMPLE-007: Oregon consumer revokes consent; system schedules processor fanout and stops processing within the configured revocation clock.
- EXAMPLE-008: Delaware child data is sensitive; system requires parental consent when child-sensitive processing is known.
- EXAMPLE-009: New Jersey targeted advertising route receives opt-out; ad profile write is denied and audit chained.
- EXAMPLE-010: New Hampshire profiling opt-out applies to a legal-effect automated decision; decision service must route to non-profiling review.
- EXAMPLE-011: HIPAA analytics job receives PHI; system records treatment, payment, operations, authorization, or other basis and minimum necessary scope.
- EXAMPLE-012: HIPAA de-id export chooses Safe Harbor; system checks identifier-removal evidence and stores no-actual-knowledge attestation.
- EXAMPLE-013: GLBA bank customer profile is shared with vendor; system requires Safeguards evidence and service-provider restrictions.
- EXAMPLE-014: FERPA school roster export is requested; system permits only consent, directory-info, school-official, audit/evaluation, or other valid exception path.
- EXAMPLE-015: COPPA child-directed game collects persistent identifiers; system requires verifiable parental consent or narrow support-for-internal-operations exception.
- EXAMPLE-016: FCRA background check affects hiring denial; system records permissible purpose and sends adverse-action notice workflow.
- EXAMPLE-017: ECOA loan model denies credit; system stores principal reasons and checks prohibited-basis feature isolation.
- EXAMPLE-018: SOX close workflow changes a key spreadsheet; system records change approval, testing, and ICFR link before use.
- EXAMPLE-019: PCAOB AS 2201 scope review identifies revenue recognition as significant; system links key controls, reports, and test results.
- EXAMPLE-020: ADA user cannot operate a form by keyboard; system opens accessibility issue and records alternative effective communication.
- EXAMPLE-021: SOC 2 customer asks for privacy criterion coverage; system returns mapped controls, evidence, and service commitments.
- EXAMPLE-022: UGESP screen filters applicants; system computes group selection rates and flags four-fifths concern.
- EXAMPLE-023: NYC employer uses AEDT for promotion; system denies unless independent bias audit is less than one year old and public notice is linked.
- EXAMPLE-024: Colorado HR AI ranks candidates for employment; system treats it as high-risk AI and records reasonable-care evidence.
- EXAMPLE-025: California automated tool performs housing eligibility; AB-331 assessment is stored as advisory, while FCRA/ECOA/fair-housing authorities drive binding policy if applicable.
- EXAMPLE-026: ITAR technical drawing is opened by offshore support; system denies foreign-person access absent authorization.
- EXAMPLE-027: NIST 800-53 tenant activates AC and AU control families; system requires account-management and audit-log evidence per control.
- EXAMPLE-028: FedRAMP Moderate package is assembled; system requires SSP, SAP, SAR, POA&M, authorization boundary, and ConMon references.
- EXAMPLE-029: Compound healthcare-financial tenant stores payment records with PHI; system stacks HIPAA, GLBA, and state privacy labels.
- EXAMPLE-030: Privacy delete request touches audit logs; system deletes user-facing data but preserves audit-chain records under retention override.
- EXAMPLE-031: Authorized agent submits opt-out; system verifies agency authority before applying opt-out.
- EXAMPLE-032: Consumer appeals denial; system routes to reviewer and includes attorney-general complaint instructions when required.
- EXAMPLE-033: Processor receives consent revocation; system audits propagation and blocks further incompatible processing.
- EXAMPLE-034: Regulator requests evidence; system creates scoped export with review approval and hash.
- EXAMPLE-035: Tenant tries to deactivate US pack; system blocks while DSAR, legal hold, or regulated data remains unresolved.

## Cross-References

- XREF-001: `packs/us-localization/federal-privacy-laws.md` covers HIPAA, GLBA, FERPA, COPPA, FCRA, and ECOA.
- XREF-002: `packs/us-localization/state-privacy-laws-comparison.md` covers CCPA/CPRA and CO, CT, VA, UT, TX, IA, IN, TN, MT, OR, DE, NJ, and NH.
- XREF-003: `packs/us-localization/sox-and-financial-reporting.md` covers SOX Sections 302, 404, 802, 906, ICFR, and PCAOB AS 2201.
- XREF-004: `packs/us-localization/hipaa-phi-handling.md` covers 45 CFR Parts 160, 162, and 164, PHI, minimum necessary, BAAs, and de-identification.
- XREF-005: `packs/us-localization/discrimination-laws-and-ai-bias.md` covers UGESP, Title VII, NYC LL 144, Colorado SB 24-205, and California AB-331 advisory treatment.
- XREF-006: `packs/cn-pipl/README.md` is the existing compliance-pack style reference, but US-PACK-1 is not derived from CN-specific PIPL semantics.
- XREF-007: `specs/root-hub-pointers.json` remains the repo discovery entry point.
- XREF-008: `specs/master-plan-sequencing.json` remains master-plan sequencing authority.
- XREF-009: `specs/microservices/manifests-index.json` lists candidate microservices activated by pack deltas.
- XREF-010: `registry/audit-chain/shards/foundation-demo.log` demonstrates audit-chain event style but is not modified by this pack.
- XREF-011: `docs/standards/logging-tracing.md` provides trace and audit context fields relevant to regulated events.
- XREF-012: `registry/catalog/check-compliance-evidence-coverage.yaml` is a future gate candidate for pack evidence.
- XREF-013: `registry/catalog/check-cedar-fragment-coverage.yaml` is a future gate candidate for executable Cedar fragment coverage.
- XREF-014: `registry/catalog/check-data-class.yaml` is a future gate candidate for data-class enforcement.
- XREF-015: `registry/catalog/check-high-risk-auto-decision-refusal.yaml` is a future gate candidate for high-risk AI decisions.
- XREF-016: `registry/catalog/check-a11y-discipline.yaml` is a future gate candidate for ADA-related accessibility evidence.
- XREF-017: `registry/catalog/check-slsa-l3-evidence-grounded.yaml` supports control evidence and provenance posture for NIST and FedRAMP overlays.
- XREF-018: `registry/catalog/check-audit-chain-seal-coverage.yaml` supports regulated event sealing.
- XREF-019: `docs/decisions/ADR-0709-general-live-apex.md` remains the Oya VCS coordination reference.
- XREF-020: `docs/decisions/ADR-0708-platform-foundations-live-apex.md` is cited by the existing CN pack and remains a likely future compliance-pack primitive reference.

## Pack Checkpoint

- CHECKPOINT-001: Checkpoint name is `us-localization-pack-w1-2026-05-20`.
- CHECKPOINT-002: Checkpoint scope is documentation only.
- CHECKPOINT-003: Checkpoint directory is `packs/us-localization/`.
- CHECKPOINT-004: Checkpoint artifact count is six markdown documents.
- CHECKPOINT-005: Checkpoint excludes `packs/kr-localization/`.
- CHECKPOINT-006: Checkpoint excludes `packs/eu-localization/`.
- CHECKPOINT-007: Checkpoint excludes ADR edits.
- CHECKPOINT-008: Checkpoint excludes microservice manifest edits.
- CHECKPOINT-009: Checkpoint excludes Cedar file generation.
- CHECKPOINT-010: Checkpoint excludes OpenAPI file generation.
- CHECKPOINT-011: Checkpoint records statutory authority but does not claim legal advice.
- CHECKPOINT-012: Checkpoint records activated Cedar policy names but not executable policy bodies.
- CHECKPOINT-013: Checkpoint records data model deltas but not schema migrations.
- CHECKPOINT-014: Checkpoint records API contract deltas but not route implementations.
- CHECKPOINT-015: Checkpoint records audit event additions but not registry inserts.
- CHECKPOINT-016: Checkpoint records failure modes to drive future tests.
- CHECKPOINT-017: Checkpoint records worked examples to drive future acceptance criteria.
- CHECKPOINT-018: Checkpoint records cross-references to guide follow-on implementation.
- CHECKPOINT-019: Checkpoint requires official-source refresh before runtime enforcement.
- CHECKPOINT-020: Checkpoint requires counsel review before regulated tenant activation.
- CHECKPOINT-021: Checkpoint requires sector routing before state privacy routing.
- CHECKPOINT-022: Checkpoint requires strongest triggered duty when no preemption or exemption controls.
- CHECKPOINT-023: Checkpoint requires residual-duty explanation when an exemption is applied.
- CHECKPOINT-024: Checkpoint requires denial by default for unclassified data, actor, purpose, jurisdiction, or authority.
- CHECKPOINT-025: Checkpoint requires audit-chain evidence for every regulated allow, denial, override, and export.
- CHECKPOINT-026: Checkpoint requires per-state sensitive-data semantics rather than generic U.S. sensitive-data handling.
- CHECKPOINT-027: Checkpoint requires California sale/share/minor semantics to remain distinct from consent-first states.
- CHECKPOINT-028: Checkpoint requires Utah and Iowa notice-plus-opt-out semantics to remain distinct from consent-first states.
- CHECKPOINT-029: Checkpoint requires HIPAA PHI handling to remain distinct from generic state privacy export.
- CHECKPOINT-030: Checkpoint requires GLBA NPI handling to remain distinct from generic consumer privacy export.
- CHECKPOINT-031: Checkpoint requires FERPA education-record handling to remain distinct from HIPAA student-health boundary guesses.
- CHECKPOINT-032: Checkpoint requires COPPA parental-consent handling before state child-sensitive processing.
- CHECKPOINT-033: Checkpoint requires FCRA permissible-purpose, notice, dispute, and adverse-action handling.
- CHECKPOINT-034: Checkpoint requires ECOA protected-basis and adverse-action reason handling.
- CHECKPOINT-035: Checkpoint requires SOX certification, ICFR, retention, and PCAOB AS 2201 handling outside privacy DSAR logic.
- CHECKPOINT-036: Checkpoint requires ADA accommodation and accessibility handling outside generic UI compliance labels.
- CHECKPOINT-037: Checkpoint requires UGESP four-fifths output to remain a review trigger, not a legal conclusion.
- CHECKPOINT-038: Checkpoint requires NYC LL 144 AEDT audit handling only when scoped.
- CHECKPOINT-039: Checkpoint requires Colorado SB 24-205 high-risk AI handling for consequential decision domains.
- CHECKPOINT-040: Checkpoint requires California AB-331 to remain advisory unless official status changes.
- CHECKPOINT-041: Checkpoint requires ITAR export-control access restrictions to remain separate from privacy and bias logic.
- CHECKPOINT-042: Checkpoint requires NIST 800-53 reuse to remain control-evidence reuse, not statutory compliance.
- CHECKPOINT-043: Checkpoint requires FedRAMP reuse to remain cloud-authorization evidence, not SOX or civil-rights proof.
- CHECKPOINT-044: Checkpoint requires SOC 2 reuse to remain assurance evidence, not legal compliance proof.
- CHECKPOINT-045: Checkpoint requires tenant activation metadata before enforcement.
- CHECKPOINT-046: Checkpoint requires law-effective-date metadata for Indiana, Colorado AI, and universal opt-out changes.
- CHECKPOINT-047: Checkpoint requires law-refresh metadata before production activation.
- CHECKPOINT-048: Checkpoint requires service ownership assignment before implementation.
- CHECKPOINT-049: Checkpoint requires regression tests before generated policy enforcement.
- CHECKPOINT-050: Checkpoint requires regulator-export review before external disclosure.
- CHECKPOINT-051: Checkpoint requires processor-contract review before downstream processing.
- CHECKPOINT-052: Checkpoint requires human-review routes for high-risk AI where applicable.
- CHECKPOINT-053: Checkpoint requires data-subject, consumer, patient, student, child, borrower, applicant, and employee identities to stay distinct.
- CHECKPOINT-054: Checkpoint requires every worked example to map to at least one future test case.
- CHECKPOINT-055: Checkpoint requires every API delta to carry evidence event ids.
- CHECKPOINT-056: Checkpoint requires every audit event name to remain pack-scoped with `us.` prefix.
- CHECKPOINT-057: Checkpoint requires future generated files to cite this documentation pack as source.
- CHECKPOINT-058: Checkpoint requires no claim that the pack is complete legal compliance.
- CHECKPOINT-059: Checkpoint is ready for VCS verify when all six files are at or above requested line depth.
- CHECKPOINT-060: Checkpoint is ready for VCS done after verify accepts `us_pack_docs:6`.
- CHECKPOINT-061: Checkpoint is ready for VCS promote after done accepts `us_pack_docs:6`.
- CHECKPOINT-062: Checkpoint stop condition is successful promote or a cleanly reported VCS blocker.
- CHECKPOINT-063: Checkpoint final report must name changed files and validation evidence.
- CHECKPOINT-064: Checkpoint final report must state that KR, EU, ADRs, and microservices were not touched.
- CHECKPOINT-065: Checkpoint final report must include source URLs because official legal sources shaped the pack.
