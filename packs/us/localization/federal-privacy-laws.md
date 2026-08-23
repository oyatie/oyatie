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
  - https://www.hhs.gov/hipaa/for-professionals/privacy/laws-regulations/index.html
  - https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C/part-160
  - https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C/part-164
  - https://www.ecfr.gov/current/title-16/chapter-I/subchapter-C/part-313
  - https://www.ecfr.gov/current/title-16/chapter-I/subchapter-C/part-314
  - https://www.ecfr.gov/current/title-34/subtitle-A/part-99
  - https://www.ecfr.gov/current/title-16/chapter-I/subchapter-C/part-312
  - https://www.ftc.gov/legal-library/browse/statutes/fair-credit-reporting-act
  - https://www.consumerfinance.gov/rules-policy/regulations/1022/
  - https://www.consumerfinance.gov/rules-policy/regulations/1002/
---

# Federal Privacy Laws

This document maps six federal privacy and decision statutes into US-PACK-1 controls.
The covered laws are HIPAA, GLBA, FERPA, COPPA, FCRA, and ECOA.
The goal is not to restate every statutory clause.
The goal is to provide implementation-grade authority routing, data model deltas, API deltas, audit events, failure modes, and worked examples.
The pack treats each law as a sector overlay.
The pack does not let generic state privacy logic replace sector law.
The pack resolves compound cases by stacking labels and recording controlling authority.
The pack is written for later conversion into schemas, Cedar fragments, and service-specific contracts.
The pack requires counsel review before production enforcement for regulated tenants.
The pack uses official public authorities available on 2026-05-20.

## Authority Citations

- FED-AUTH-001: HIPAA statutory background comes from Public Law 104-191 and HHS Privacy Rule guidance.
- FED-AUTH-002: HIPAA Privacy Rule is codified in 45 CFR Part 160 and Subparts A and E of Part 164.
- FED-AUTH-003: HIPAA Security Rule is codified in 45 CFR Part 164 Subpart C.
- FED-AUTH-004: HIPAA Breach Notification Rule is codified in 45 CFR Part 164 Subpart D.
- FED-AUTH-005: HIPAA administrative definitions include covered entity, business associate, and protected health information in 45 CFR 160.103.
- FED-AUTH-006: HIPAA general use/disclosure logic includes treatment, payment, healthcare operations, authorization, required disclosures, permitted disclosures, and minimum necessary.
- FED-AUTH-007: HIPAA de-identification authority includes 45 CFR 164.514(a)-(b), Safe Harbor, and expert determination.
- FED-AUTH-008: GLBA statutory authority is 15 U.S.C. 6801 et seq.
- FED-AUTH-009: GLBA Privacy Rule for FTC-regulated financial institutions is 16 CFR Part 313.
- FED-AUTH-010: GLBA Safeguards Rule for FTC-regulated financial institutions is 16 CFR Part 314.
- FED-AUTH-011: GLBA data class is nonpublic personal information about a consumer or customer of a financial institution.
- FED-AUTH-012: FERPA statutory authority is 20 U.S.C. 1232g.
- FED-AUTH-013: FERPA regulations are 34 CFR Part 99.
- FED-AUTH-014: FERPA applies to educational agencies and institutions receiving applicable Department of Education funds.
- FED-AUTH-015: FERPA protected object is the education record and personally identifiable information from education records.
- FED-AUTH-016: COPPA statutory authority is 15 U.S.C. 6501 et seq.
- FED-AUTH-017: COPPA Rule is 16 CFR Part 312.
- FED-AUTH-018: COPPA applies to operators of child-directed online services and operators with actual knowledge of collecting personal information from children under 13.
- FED-AUTH-019: FCRA statutory authority is 15 U.S.C. 1681 et seq.
- FED-AUTH-020: FTC maintains a revised public FCRA text and CFPB maintains Regulation V, 12 CFR Part 1022.
- FED-AUTH-021: FCRA covers consumer reporting agencies, furnishers, users of consumer reports, permissible purpose, notices, disputes, and adverse action.
- FED-AUTH-022: ECOA statutory authority is 15 U.S.C. 1691 et seq.
- FED-AUTH-023: CFPB Regulation B implements ECOA at 12 CFR Part 1002.
- FED-AUTH-024: ECOA covers discrimination in credit transactions on prohibited bases.
- FED-AUTH-025: ECOA adverse-action duties require specific and accurate principal reasons in covered credit contexts.
- FED-AUTH-026: Federal privacy laws frequently coexist with state privacy law; US-PACK-1 uses sector labels first and state labels second.
- FED-AUTH-027: HIPAA and FERPA can overlap for student health records; the pack requires explicit education-health boundary classification.
- FED-AUTH-028: GLBA and FCRA can overlap for financial eligibility, creditworthiness, and reporting workflows; the pack requires separate NPI and consumer-report labels.
- FED-AUTH-029: FCRA and ECOA can overlap in credit decisions; the pack records both report-permissible purpose and credit adverse-action reasons.
- FED-AUTH-030: COPPA and state privacy child-sensitive categories can overlap; under-13 parental-consent duties must be satisfied before generic state-sensitive logic.

## Law Routing Overview

- ROUTE-001: Route to HIPAA when the tenant is a covered entity, business associate, or subcontractor business associate.
- ROUTE-002: Route to HIPAA when the object is PHI, ePHI, limited data set, de-identified health information, or healthcare operations evidence.
- ROUTE-003: Route to GLBA when the tenant is an FTC-regulated financial institution or service provider handling customer information.
- ROUTE-004: Route to GLBA when the object is NPI, customer information, privacy notice state, sharing preference, or safeguards evidence.
- ROUTE-005: Route to FERPA when the tenant is an educational agency or institution receiving covered funds.
- ROUTE-006: Route to FERPA when the object is an education record or PII from an education record.
- ROUTE-007: Route to COPPA when the service is child-directed.
- ROUTE-008: Route to COPPA when actual knowledge says the user is under 13.
- ROUTE-009: Route to FCRA when data is obtained from, furnished to, or used as a consumer report.
- ROUTE-010: Route to FCRA when a background check, tenant screening, employment screen, credit report, or insurance eligibility report is used.
- ROUTE-011: Route to ECOA when credit application, underwriting, pricing, servicing, adverse action, or special-purpose credit logic is executed.
- ROUTE-012: Route to state privacy overlay only after sector routing determines whether entity or data exemptions apply.
- ROUTE-013: Route to compound mode when one action uses PHI, NPI, education records, consumer-report data, and credit decision data together.
- ROUTE-014: Route to legal review when tenant role and data role disagree.
- ROUTE-015: Route to deny when no permissible purpose, consent, authorization, notice, or exception can be resolved.

## HIPAA Named Provisions

- HIPAA-001: 45 CFR 160.103 defines covered entity.
- HIPAA-002: 45 CFR 160.103 defines business associate.
- HIPAA-003: 45 CFR 160.103 defines protected health information.
- HIPAA-004: 45 CFR 160.103 defines health information.
- HIPAA-005: 45 CFR 160.103 defines standard transaction context.
- HIPAA-006: 45 CFR 164.502 establishes general rules for uses and disclosures of PHI.
- HIPAA-007: 45 CFR 164.502(a) supports use/disclosure only as permitted or required.
- HIPAA-008: 45 CFR 164.502(b) establishes minimum necessary.
- HIPAA-009: 45 CFR 164.502(d) addresses de-identified protected health information.
- HIPAA-010: 45 CFR 164.502(e) addresses business associate disclosures.
- HIPAA-011: 45 CFR 164.504(e) addresses business associate contract requirements.
- HIPAA-012: 45 CFR 164.506 addresses treatment, payment, and healthcare operations.
- HIPAA-013: 45 CFR 164.508 addresses authorizations.
- HIPAA-014: 45 CFR 164.510 addresses uses/disclosures requiring opportunity to agree or object.
- HIPAA-015: 45 CFR 164.512 addresses public interest and benefit activities.
- HIPAA-016: 45 CFR 164.514(a) states de-identification standard.
- HIPAA-017: 45 CFR 164.514(b)(1) supports expert determination.
- HIPAA-018: 45 CFR 164.514(b)(2) supports Safe Harbor identifier removal.
- HIPAA-019: 45 CFR 164.514(e) addresses limited data sets and data use agreements.
- HIPAA-020: 45 CFR 164.520 addresses notice of privacy practices.
- HIPAA-021: 45 CFR 164.524 addresses individual access.
- HIPAA-022: 45 CFR 164.526 addresses amendment.
- HIPAA-023: 45 CFR 164.528 addresses accounting of disclosures.
- HIPAA-024: 45 CFR 164.530 addresses administrative requirements.
- HIPAA-025: 45 CFR 164.308 addresses Security Rule administrative safeguards.
- HIPAA-026: 45 CFR 164.310 addresses Security Rule physical safeguards.
- HIPAA-027: 45 CFR 164.312 addresses Security Rule technical safeguards.
- HIPAA-028: 45 CFR 164.316 addresses Security Rule documentation.
- HIPAA-029: 45 CFR 164.400 through 164.414 address breach notification.
- HIPAA-030: 45 CFR Part 162 addresses transaction and code set standards for administrative simplification.

## GLBA Named Provisions

- GLBA-001: 15 U.S.C. 6801 states the policy that financial institutions have an obligation to respect customer privacy and protect security/confidentiality.
- GLBA-002: 16 CFR Part 313 implements privacy of consumer financial information for FTC-regulated financial institutions.
- GLBA-003: 16 CFR 313.3 defines financial institution, consumer, customer, nonpublic personal information, and nonaffiliated third party for FTC Privacy Rule purposes.
- GLBA-004: 16 CFR 313.4 addresses initial privacy notice to consumers.
- GLBA-005: 16 CFR 313.5 addresses annual privacy notice to customers.
- GLBA-006: 16 CFR 313.6 addresses information to include in privacy notices.
- GLBA-007: 16 CFR 313.7 addresses opt-out notice.
- GLBA-008: 16 CFR 313.10 addresses limits on disclosure to nonaffiliated third parties.
- GLBA-009: 16 CFR 313.13 addresses service-provider and joint-marketing exceptions.
- GLBA-010: 16 CFR 313.14 addresses processing and servicing exceptions.
- GLBA-011: 16 CFR 313.15 addresses other exceptions.
- GLBA-012: 16 CFR 314.1 states purpose and scope of Standards for Safeguarding Customer Information.
- GLBA-013: 16 CFR 314.2 defines customer information and financial institution for Safeguards Rule.
- GLBA-014: 16 CFR 314.3 requires developing, implementing, and maintaining a written information security program.
- GLBA-015: 16 CFR 314.4 specifies elements of an information security program.
- GLBA-016: 16 CFR 314.4 requires qualified individual responsibility.
- GLBA-017: 16 CFR 314.4 requires risk assessment.
- GLBA-018: 16 CFR 314.4 requires safeguards design and implementation.
- GLBA-019: 16 CFR 314.4 requires monitoring and testing.
- GLBA-020: 16 CFR 314.4 requires service-provider oversight.
- GLBA-021: 16 CFR 314.4 requires evaluation and adjustment of the security program.
- GLBA-022: 16 CFR 314.4 requires incident response plan for covered financial institutions.
- GLBA-023: 16 CFR 314.4 requires qualified individual reporting to the board or senior officer.
- GLBA-024: 16 CFR 314.5 includes exemptions for some smaller institutions from selected requirements.
- GLBA-025: FTC breach-reporting amendments require notice for certain security events by covered nonbank financial institutions.

## FERPA Named Provisions

- FERPA-001: 34 CFR 99.1 states the applicability to educational agencies and institutions receiving covered funds.
- FERPA-002: 34 CFR 99.2 states the purpose of protecting privacy of parents and students.
- FERPA-003: 34 CFR 99.3 defines education records.
- FERPA-004: 34 CFR 99.3 defines personally identifiable information.
- FERPA-005: 34 CFR 99.3 defines directory information.
- FERPA-006: 34 CFR 99.4 states parent rights.
- FERPA-007: 34 CFR 99.5 states eligible student rights.
- FERPA-008: 34 CFR 99.7 addresses annual notification.
- FERPA-009: 34 CFR 99.10 addresses inspection and review rights.
- FERPA-010: 34 CFR 99.20 addresses request to amend records.
- FERPA-011: 34 CFR 99.21 addresses hearing rights.
- FERPA-012: 34 CFR 99.30 addresses prior consent for disclosure.
- FERPA-013: 34 CFR 99.31 addresses disclosures not requiring consent.
- FERPA-014: 34 CFR 99.32 addresses disclosure recordkeeping.
- FERPA-015: 34 CFR 99.33 addresses redisclosure limits.
- FERPA-016: 34 CFR 99.34 addresses disclosure to other educational agencies or institutions.
- FERPA-017: 34 CFR 99.35 addresses audit/evaluation and enforcement disclosures.
- FERPA-018: 34 CFR 99.36 addresses health and safety emergency disclosures.
- FERPA-019: 34 CFR 99.37 addresses directory information.
- FERPA-020: 34 CFR 99.60 through 99.67 address enforcement procedure.

## COPPA Named Provisions

- COPPA-001: 16 CFR 312.2 defines child as an individual under 13.
- COPPA-002: 16 CFR 312.2 defines operator.
- COPPA-003: 16 CFR 312.2 defines personal information.
- COPPA-004: 16 CFR 312.2 defines website or online service directed to children.
- COPPA-005: 16 CFR 312.3 states regulation of unfair or deceptive acts in connection with child data.
- COPPA-006: 16 CFR 312.4 addresses notice requirements.
- COPPA-007: 16 CFR 312.5 addresses parental consent.
- COPPA-008: 16 CFR 312.6 addresses parental right to review.
- COPPA-009: 16 CFR 312.7 prohibits conditioning participation on more data than reasonably necessary.
- COPPA-010: 16 CFR 312.8 requires confidentiality, security, and integrity protections.
- COPPA-011: 16 CFR 312.10 addresses data retention and deletion.
- COPPA-012: 16 CFR 312.11 addresses safe harbor programs.
- COPPA-013: COPPA applies where the service is child-directed even if the tenant prefers an age-neutral posture.
- COPPA-014: COPPA applies where actual knowledge exists even if the service is not child-directed.
- COPPA-015: COPPA must be resolved before generic state child-sensitive-data rules.

## FCRA Named Provisions

- FCRA-001: 15 U.S.C. 1681 states Congressional findings and purpose around fair and accurate credit reporting.
- FCRA-002: 15 U.S.C. 1681a defines consumer report.
- FCRA-003: 15 U.S.C. 1681a defines consumer reporting agency.
- FCRA-004: 15 U.S.C. 1681b addresses permissible purposes of consumer reports.
- FCRA-005: 15 U.S.C. 1681c addresses contents of reports and obsolete information limits.
- FCRA-006: 15 U.S.C. 1681d addresses investigative consumer reports.
- FCRA-007: 15 U.S.C. 1681e addresses compliance procedures.
- FCRA-008: 15 U.S.C. 1681g addresses disclosures to consumers.
- FCRA-009: 15 U.S.C. 1681i addresses dispute procedures.
- FCRA-010: 15 U.S.C. 1681m addresses adverse action notices and user duties.
- FCRA-011: 15 U.S.C. 1681s-2 addresses furnisher duties.
- FCRA-012: CFPB Regulation V at 12 CFR Part 1022 implements FCRA obligations for covered actors.
- FCRA-013: FCRA tenant-screening and employment-screening workflows must preserve report source and adverse-action notices.
- FCRA-014: FCRA data cannot be downgraded to generic personal data once used as a consumer report.
- FCRA-015: FCRA use for credit decisions may also trigger ECOA adverse-action and fair-lending analysis.

## ECOA Named Provisions

- ECOA-001: ECOA prohibits credit discrimination on protected bases such as race, color, religion, national origin, sex, marital status, age, public-assistance status, and protected rights exercise.
- ECOA-002: Regulation B, 12 CFR Part 1002, implements ECOA.
- ECOA-003: 12 CFR 1002.2 defines applicant, application, creditor, credit, and prohibited basis.
- ECOA-004: 12 CFR 1002.4 states general anti-discrimination rules.
- ECOA-005: 12 CFR 1002.5 addresses information a creditor may and may not request.
- ECOA-006: 12 CFR 1002.6 addresses rules concerning evaluation of applications.
- ECOA-007: 12 CFR 1002.7 addresses rules concerning extensions of credit.
- ECOA-008: 12 CFR 1002.9 addresses notifications and adverse action.
- ECOA-009: 12 CFR 1002.10 addresses furnishing credit information.
- ECOA-010: 12 CFR 1002.12 addresses record retention.
- ECOA-011: 12 CFR 1002.13 addresses information collection for certain mortgage applications.
- ECOA-012: 12 CFR 1002.14 addresses appraisal and other valuation copies.
- ECOA-013: 12 CFR 1002.15 addresses incentive programs and special purpose credit programs where applicable.
- ECOA-014: ECOA automated decision workflows must retain explainable principal reasons for adverse action.
- ECOA-015: ECOA monitoring features must be segregated from decision features unless a permitted rule allows use.

## Activated Cedar Policies

- FED-CEDAR-001: `us-federal-hipaa-covered-entity-route` activates HIPAA policy if tenant role is covered entity.
- FED-CEDAR-002: `us-federal-hipaa-business-associate-route` activates HIPAA policy if tenant role is business associate or subcontractor.
- FED-CEDAR-003: `us-federal-hipaa-phi-minimum-necessary` denies PHI use above recorded minimum-necessary scope.
- FED-CEDAR-004: `us-federal-hipaa-treatment-exception` permits treatment use without applying minimum necessary when treatment exception is valid.
- FED-CEDAR-005: `us-federal-hipaa-baa-required` denies PHI disclosure to a vendor without valid BAA.
- FED-CEDAR-006: `us-federal-hipaa-authorization-required` requires authorization for non-permitted PHI use.
- FED-CEDAR-007: `us-federal-hipaa-limited-data-set-dua` requires data use agreement for limited data set.
- FED-CEDAR-008: `us-federal-hipaa-safe-harbor-check` requires Safe Harbor identifier removal evidence.
- FED-CEDAR-009: `us-federal-hipaa-expert-determination-check` requires expert determination evidence.
- FED-CEDAR-010: `us-federal-hipaa-breach-notification-clock` starts breach workflows on confirmed or reasonably believed unauthorized PHI exposure.
- FED-CEDAR-011: `us-federal-glba-financial-institution-route` activates GLBA when tenant meets financial-institution role.
- FED-CEDAR-012: `us-federal-glba-npi-classification` requires NPI data label on covered customer information.
- FED-CEDAR-013: `us-federal-glba-privacy-notice-required` requires initial and annual notice evidence.
- FED-CEDAR-014: `us-federal-glba-sharing-optout-required` requires opt-out handling for nonaffiliated third-party sharing.
- FED-CEDAR-015: `us-federal-glba-safeguards-program-required` denies NPI processing without safeguards program reference.
- FED-CEDAR-016: `us-federal-glba-service-provider-oversight` requires contract and oversight evidence for service providers.
- FED-CEDAR-017: `us-federal-ferpa-funded-institution-route` activates FERPA for covered educational agencies and institutions.
- FED-CEDAR-018: `us-federal-ferpa-education-record-label` requires education-record classification.
- FED-CEDAR-019: `us-federal-ferpa-consent-or-exception` denies disclosure lacking consent or 34 CFR 99.31 exception.
- FED-CEDAR-020: `us-federal-ferpa-directory-info-notice` requires annual notice and opt-out handling for directory information.
- FED-CEDAR-021: `us-federal-ferpa-redisclosure-limit` restricts recipient redisclosure without valid basis.
- FED-CEDAR-022: `us-federal-coppa-child-directed-route` activates COPPA for child-directed services.
- FED-CEDAR-023: `us-federal-coppa-actual-knowledge-route` activates COPPA for known under-13 users.
- FED-CEDAR-024: `us-federal-coppa-parental-consent-required` requires verifiable parental consent before covered collection.
- FED-CEDAR-025: `us-federal-coppa-necessary-data-only` denies conditioning child participation on excessive data.
- FED-CEDAR-026: `us-federal-coppa-retention-delete` enforces retention limited to purpose and deletion after purpose.
- FED-CEDAR-027: `us-federal-fcra-permissible-purpose` denies consumer-report access without permissible purpose.
- FED-CEDAR-028: `us-federal-fcra-adverse-action-notice` requires notice when report data contributes to adverse action.
- FED-CEDAR-029: `us-federal-fcra-furnisher-dispute` requires furnisher dispute handling and audit.
- FED-CEDAR-030: `us-federal-fcra-obsolete-info-filter` blocks stale report content where age limits apply.
- FED-CEDAR-031: `us-federal-ecoa-credit-decision-route` activates ECOA for credit transactions.
- FED-CEDAR-032: `us-federal-ecoa-prohibited-basis-deny` denies decision use of prohibited-basis features outside permitted monitoring.
- FED-CEDAR-033: `us-federal-ecoa-adverse-action-reasons` requires principal reason codes and notice reference.
- FED-CEDAR-034: `us-federal-ecoa-record-retention` preserves covered credit records for required periods.
- FED-CEDAR-035: `us-federal-ecoa-special-purpose-credit-program` allows scoped use only for documented SPCP basis.
- FED-CEDAR-036: `us-federal-sector-precedence` stacks sector label before state privacy overlay.
- FED-CEDAR-037: `us-federal-sector-exemption-check` validates whether state law exempts entity, data, or processing purpose.
- FED-CEDAR-038: `us-federal-compound-resolution` records controlling law when several federal overlays apply.
- FED-CEDAR-039: `us-federal-regulator-export-scope` scopes evidence exports to the requesting law and data class.
- FED-CEDAR-040: `us-federal-retention-delete-conflict` preserves records when federal retention overrides deletion.

## Data Model Deltas

- FED-DATA-001: `hipaa_role` enum: `none`, `covered_entity`, `business_associate`, `subcontractor_business_associate`.
- FED-DATA-002: `hipaa_transaction_context` boolean marks Part 162 transaction relevance.
- FED-DATA-003: `phi_status` enum: `not_phi`, `phi`, `ephi`, `limited_data_set`, `deidentified`.
- FED-DATA-004: `phi_subject_id` references patient or member identity.
- FED-DATA-005: `minimum_necessary_scope` records fields, recipients, and purpose.
- FED-DATA-006: `hipaa_permitted_use_code` records treatment, payment, operations, authorization, required disclosure, public interest, limited data set, or other basis.
- FED-DATA-007: `baa_ref` references business associate agreement.
- FED-DATA-008: `dua_ref` references limited data set data use agreement.
- FED-DATA-009: `hipaa_authorization_ref` references individual authorization.
- FED-DATA-010: `deid_method` enum: `safe_harbor`, `expert_determination`, `not_deidentified`.
- FED-DATA-011: `deid_identifier_removal_checklist_ref` records Safe Harbor evidence.
- FED-DATA-012: `deid_expert_determination_ref` records expert determination evidence.
- FED-DATA-013: `hipaa_breach_assessment_ref` records breach risk assessment.
- FED-DATA-014: `glba_financial_institution_flag` marks covered tenant role.
- FED-DATA-015: `glba_customer_status` enum: `consumer`, `customer`, `former_customer`, `not_applicable`.
- FED-DATA-016: `npi_status` enum: `not_npi`, `npi`, `customer_information`.
- FED-DATA-017: `glba_privacy_notice_ref` records notice version and delivery.
- FED-DATA-018: `glba_optout_status` records nonaffiliated third-party sharing opt-out.
- FED-DATA-019: `glba_safeguards_program_ref` links written information security program.
- FED-DATA-020: `glba_service_provider_contract_ref` links service-provider restrictions.
- FED-DATA-021: `ferpa_institution_flag` marks covered educational agency or institution.
- FED-DATA-022: `education_record_status` enum: `not_education_record`, `education_record`, `directory_information`.
- FED-DATA-023: `ferpa_parent_or_eligible_student_id` references rights holder.
- FED-DATA-024: `ferpa_consent_ref` records written consent.
- FED-DATA-025: `ferpa_exception_code` records 34 CFR 99.31 exception.
- FED-DATA-026: `ferpa_directory_notice_ref` records annual directory notice.
- FED-DATA-027: `ferpa_redisclosure_restriction_ref` records recipient constraint.
- FED-DATA-028: `coppa_child_directed_flag` marks service route.
- FED-DATA-029: `coppa_actual_knowledge_flag` marks known under-13 route.
- FED-DATA-030: `coppa_child_user_id` references under-13 subject.
- FED-DATA-031: `coppa_parent_id` references verified parent or guardian.
- FED-DATA-032: `coppa_parental_consent_ref` records method and timestamp.
- FED-DATA-033: `coppa_notice_ref` records direct and online notice.
- FED-DATA-034: `coppa_retention_purpose_ref` records child-data retention purpose.
- FED-DATA-035: `fcra_actor_role` enum: `consumer_reporting_agency`, `furnisher`, `user`, `service_provider`, `not_applicable`.
- FED-DATA-036: `consumer_report_status` enum: `not_report`, `consumer_report`, `investigative_consumer_report`.
- FED-DATA-037: `fcra_permissible_purpose_code` records reason under FCRA.
- FED-DATA-038: `fcra_report_source_ref` records CRA or furnisher origin.
- FED-DATA-039: `fcra_adverse_action_notice_ref` records notice.
- FED-DATA-040: `fcra_dispute_ref` records consumer dispute.
- FED-DATA-041: `ecoa_credit_transaction_flag` marks credit context.
- FED-DATA-042: `ecoa_applicant_id` references applicant.
- FED-DATA-043: `ecoa_credit_product_type` records product class.
- FED-DATA-044: `ecoa_decision_outcome` records approve, deny, counteroffer, incomplete, withdraw, or no_action.
- FED-DATA-045: `ecoa_adverse_action_reason_codes` records principal reasons.
- FED-DATA-046: `ecoa_prohibited_basis_monitoring_ref` isolates monitoring data.
- FED-DATA-047: `ecoa_special_purpose_credit_program_ref` records SPCP basis.
- FED-DATA-048: `sector_state_exemption_resolution` records whether state overlay applies.
- FED-DATA-049: `federal_retention_override_until` records retention deadline.
- FED-DATA-050: `federal_authority_basis_refs` stores law and provision IDs.

## API Contract Deltas

- FED-API-001: `POST /federal/hipaa/route` resolves HIPAA role and PHI status.
- FED-API-002: `POST /federal/hipaa/phi/use` records use basis and minimum necessary scope.
- FED-API-003: `POST /federal/hipaa/phi/disclosure` records disclosure basis, recipient, BAA, and authorization status.
- FED-API-004: `POST /federal/hipaa/phi/access-request` opens individual PHI access workflow.
- FED-API-005: `POST /federal/hipaa/phi/amendment-request` opens PHI amendment workflow.
- FED-API-006: `POST /federal/hipaa/phi/accounting-request` opens accounting-of-disclosures workflow.
- FED-API-007: `POST /federal/hipaa/deidentify/safe-harbor` records Safe Harbor evidence.
- FED-API-008: `POST /federal/hipaa/deidentify/expert-determination` records expert determination evidence.
- FED-API-009: `POST /federal/hipaa/breach/assess` opens breach assessment.
- FED-API-010: `POST /federal/hipaa/breach/notify` records notification evidence.
- FED-API-011: `POST /federal/glba/route` resolves GLBA institution and NPI status.
- FED-API-012: `POST /federal/glba/privacy-notice` records notice delivery and version.
- FED-API-013: `POST /federal/glba/optout` records nonaffiliated third-party sharing opt-out.
- FED-API-014: `POST /federal/glba/safeguards/program` records safeguards program reference.
- FED-API-015: `POST /federal/glba/service-provider` records provider contract and oversight.
- FED-API-016: `POST /federal/ferpa/route` resolves institution and education-record status.
- FED-API-017: `POST /federal/ferpa/disclosure` records consent or exception basis.
- FED-API-018: `POST /federal/ferpa/directory-info` records directory notice and opt-out state.
- FED-API-019: `POST /federal/ferpa/amendment-request` opens record amendment workflow.
- FED-API-020: `POST /federal/ferpa/redisclosure` evaluates recipient redisclosure restrictions.
- FED-API-021: `POST /federal/coppa/route` resolves child-directed or actual-knowledge route.
- FED-API-022: `POST /federal/coppa/notice` records direct and online notice delivery.
- FED-API-023: `POST /federal/coppa/parental-consent` records verifiable parental consent.
- FED-API-024: `POST /federal/coppa/parent-review` records parent review request.
- FED-API-025: `POST /federal/coppa/delete` records child-data deletion.
- FED-API-026: `POST /federal/fcra/report/access` records permissible purpose before report access.
- FED-API-027: `POST /federal/fcra/adverse-action` records notice workflow.
- FED-API-028: `POST /federal/fcra/dispute` opens dispute and reinvestigation tracking.
- FED-API-029: `POST /federal/fcra/furnisher-update` records furnisher correction.
- FED-API-030: `POST /federal/fcra/report-filter` filters obsolete or impermissible report data.
- FED-API-031: `POST /federal/ecoa/credit-decision` records credit decision and reason codes.
- FED-API-032: `POST /federal/ecoa/adverse-action` records adverse-action notice.
- FED-API-033: `POST /federal/ecoa/prohibited-basis-check` evaluates protected-basis feature use.
- FED-API-034: `POST /federal/ecoa/spcp` records special-purpose credit program basis.
- FED-API-035: `POST /federal/ecoa/record-retention` records credit record preservation.
- FED-API-036: `POST /federal/compound/authority-resolution` records cross-law precedence.
- FED-API-037: `GET /federal/classification/{object_id}` returns active federal labels.
- FED-API-038: `POST /federal/retention/override` records federal retention override.
- FED-API-039: `POST /federal/regulator-export` creates law-scoped evidence export.
- FED-API-040: `POST /federal/exemption/state-privacy` resolves federal sector exemption against state overlay.

## Audit Event Additions

- FED-AUDIT-001: `FederalHipaaRouteResolved` records HIPAA role and PHI status.
- FED-AUDIT-002: `FederalHipaaPhiUseRecorded` records permitted use, scope, and actor.
- FED-AUDIT-003: `FederalHipaaPhiDisclosureRecorded` records recipient, basis, BAA, and authorization.
- FED-AUDIT-004: `FederalHipaaPhiAccessRequestOpened` records individual request.
- FED-AUDIT-005: `FederalHipaaPhiAmendmentRequestOpened` records amendment request.
- FED-AUDIT-006: `FederalHipaaAccountingRequestOpened` records accounting request.
- FED-AUDIT-007: `FederalHipaaDeidentificationRecorded` records method and evidence.
- FED-AUDIT-008: `FederalHipaaBreachAssessmentOpened` records incident clock.
- FED-AUDIT-009: `FederalHipaaBreachNotificationRecorded` records recipient and timestamp.
- FED-AUDIT-010: `FederalGlbaRouteResolved` records financial institution and NPI status.
- FED-AUDIT-011: `FederalGlbaNoticeDelivered` records notice version.
- FED-AUDIT-012: `FederalGlbaOptOutRecorded` records sharing opt-out.
- FED-AUDIT-013: `FederalGlbaSafeguardsProgramLinked` records program reference.
- FED-AUDIT-014: `FederalGlbaServiceProviderLinked` records provider contract.
- FED-AUDIT-015: `FederalFerpaRouteResolved` records institution and record status.
- FED-AUDIT-016: `FederalFerpaDisclosureRecorded` records consent or exception.
- FED-AUDIT-017: `FederalFerpaDirectoryInfoDecision` records directory notice and opt-out.
- FED-AUDIT-018: `FederalFerpaAmendmentRequestOpened` records amendment request.
- FED-AUDIT-019: `FederalFerpaRedisclosureEvaluated` records recipient restriction.
- FED-AUDIT-020: `FederalCoppaRouteResolved` records child-directed or actual knowledge route.
- FED-AUDIT-021: `FederalCoppaNoticeDelivered` records notice evidence.
- FED-AUDIT-022: `FederalCoppaParentalConsentRecorded` records method.
- FED-AUDIT-023: `FederalCoppaParentReviewOpened` records review request.
- FED-AUDIT-024: `FederalCoppaChildDataDeleted` records deletion.
- FED-AUDIT-025: `FederalFcraReportAccessed` records permissible purpose.
- FED-AUDIT-026: `FederalFcraAdverseActionNoticeIssued` records notice.
- FED-AUDIT-027: `FederalFcraDisputeOpened` records dispute.
- FED-AUDIT-028: `FederalFcraFurnisherCorrectionRecorded` records update.
- FED-AUDIT-029: `FederalFcraObsoleteInfoFiltered` records filter action.
- FED-AUDIT-030: `FederalEcoaCreditDecisionRecorded` records decision and reasons.
- FED-AUDIT-031: `FederalEcoaAdverseActionNoticeIssued` records notice.
- FED-AUDIT-032: `FederalEcoaProhibitedBasisCheckCompleted` records feature review.
- FED-AUDIT-033: `FederalEcoaSpcpBasisRecorded` records SPCP basis.
- FED-AUDIT-034: `FederalEcoaRecordRetentionApplied` records retention.
- FED-AUDIT-035: `FederalCompoundAuthorityResolved` records precedence.
- FED-AUDIT-036: `FederalStateExemptionResolved` records entity/data exemption.
- FED-AUDIT-037: `FederalRetentionOverrideApplied` records retention override.
- FED-AUDIT-038: `FederalRegulatorExportCreated` records scoped export.
- FED-AUDIT-039: `FederalSectorProcessingDenied` records missing authority basis.
- FED-AUDIT-040: `FederalSectorProcessingApproved` records resolved basis.

## Failure Modes

- FED-FAIL-001: PHI is processed as generic state personal data and bypasses HIPAA basis checks.
- FED-FAIL-002: A vendor receives PHI without BAA linkage.
- FED-FAIL-003: Minimum necessary is applied to treatment disclosures where the exception permits broader exchange, creating false denials.
- FED-FAIL-004: Minimum necessary is skipped for operations analytics, creating over-disclosure.
- FED-FAIL-005: Safe Harbor de-identification retains dates or small geography fields improperly.
- FED-FAIL-006: Expert determination lacks residual-risk documentation.
- FED-FAIL-007: HIPAA breach clock starts after public disclosure instead of discovery.
- FED-FAIL-008: NPI is processed without GLBA Safeguards program reference.
- FED-FAIL-009: GLBA privacy notice is delivered once but not versioned.
- FED-FAIL-010: Nonaffiliated third-party sharing ignores GLBA opt-out state.
- FED-FAIL-011: Service provider is treated as exempt without contract restrictions.
- FED-FAIL-012: FERPA education record is exported under generic DSAR tooling.
- FED-FAIL-013: FERPA directory information is disclosed without annual notice and opt-out check.
- FED-FAIL-014: FERPA exception code is stored but recipient redisclosure restrictions are missing.
- FED-FAIL-015: Student health record is misrouted between HIPAA and FERPA without boundary resolution.
- FED-FAIL-016: COPPA under-13 collection relies on teen opt-in instead of parental consent.
- FED-FAIL-017: Child-directed service claims age neutrality without actual design review.
- FED-FAIL-018: COPPA retention persists child data after purpose completion.
- FED-FAIL-019: FCRA report is accessed for curiosity, fraud hunt, or internal analytics without permissible purpose.
- FED-FAIL-020: FCRA adverse-action workflow omits report source notice.
- FED-FAIL-021: Furnisher dispute correction is not propagated.
- FED-FAIL-022: Obsolete report information is retained in decision cache.
- FED-FAIL-023: ECOA credit denial reason is generic or post-hoc.
- FED-FAIL-024: ECOA protected-basis monitoring data leaks into underwriting features.
- FED-FAIL-025: Special-purpose credit program claims lack documented eligibility basis.
- FED-FAIL-026: FCRA and ECOA notices are conflated and one required notice is omitted.
- FED-FAIL-027: Federal sector exemption is applied to all tenant data, hiding non-sector state privacy duties.
- FED-FAIL-028: State privacy deletion removes federally required retention records.
- FED-FAIL-029: Audit event records only the pack id and not the controlling provision.
- FED-FAIL-030: Regulator export includes data outside the requested statute or affected subjects.
- FED-FAIL-031: Processor role is unclassified, so controller duties are misapplied.
- FED-FAIL-032: Business associate subcontractor is treated as ordinary processor.
- FED-FAIL-033: FERPA eligible-student transition is missed after student reaches the relevant age/status.
- FED-FAIL-034: COPPA parent withdrawal is not propagated to downstream stores.
- FED-FAIL-035: GLBA Safeguards board report evidence is absent for covered institution.
- FED-FAIL-036: HIPAA audit logs are deleted before investigation window closes.
- FED-FAIL-037: ECOA record retention is shorter than credit decision audit need.
- FED-FAIL-038: FCRA consumer report copy is stored in unsecured support ticket.
- FED-FAIL-039: FERPA amendment denial lacks hearing path.
- FED-FAIL-040: HIPAA authorization form is used after revocation.

## Worked Examples

- FED-EXAMPLE-001: A health provider exports appointment analytics; system labels PHI and requires healthcare operations basis plus minimum necessary.
- FED-EXAMPLE-002: A billing vendor receives claims data; system denies until BAA is linked.
- FED-EXAMPLE-003: A research team requests a limited data set; system requires DUA and limited data set label.
- FED-EXAMPLE-004: A de-identification job uses Safe Harbor; system records identifier-removal checklist and blocks if any retained identifier is detected.
- FED-EXAMPLE-005: A health breach is confirmed; system opens breach assessment and starts notification evidence tracking.
- FED-EXAMPLE-006: A fintech tenant sends customer NPI to analytics vendor; system requires GLBA service-provider restrictions and safeguards evidence.
- FED-EXAMPLE-007: A bank updates annual notice; system records version and delivery cohort.
- FED-EXAMPLE-008: A customer opts out of GLBA sharing; system blocks nonaffiliated marketing disclosures.
- FED-EXAMPLE-009: A school exports gradebook data; system labels education record and checks consent or exception.
- FED-EXAMPLE-010: A registrar discloses directory information; system confirms annual notice and opt-out state.
- FED-EXAMPLE-011: A parent requests record amendment; system opens FERPA amendment workflow and preserves hearing path.
- FED-EXAMPLE-012: A child-directed game collects email for account creation; system requires COPPA parental consent or deletes unneeded data.
- FED-EXAMPLE-013: A parent requests child-data deletion; system records deletion and retention exceptions.
- FED-EXAMPLE-014: A tenant screening report is pulled; system records FCRA permissible purpose before access.
- FED-EXAMPLE-015: Employment background report contributes to rejection; system records FCRA adverse-action notice.
- FED-EXAMPLE-016: A credit model denies a loan; system records ECOA principal reasons and prevents protected-basis feature use.
- FED-EXAMPLE-017: A creditor wants to offer special-purpose credit; system records program basis and monitoring separation.
- FED-EXAMPLE-018: A consumer disputes report data; system opens FCRA dispute workflow and freezes stale decision reuse.
- FED-EXAMPLE-019: A student health clinic uses health records maintained by a university; system requires HIPAA/FERPA boundary resolution.
- FED-EXAMPLE-020: A mortgage workflow uses credit report and protected-basis monitoring data; system stacks FCRA and ECOA and isolates monitoring fields.
- FED-EXAMPLE-021: A privacy deletion request covers GLBA data; system applies retention and notice constraints before deletion.
- FED-EXAMPLE-022: A state privacy access export covers PHI; system routes to HIPAA access workflow instead of generic export.
- FED-EXAMPLE-023: A regulator asks for FCRA evidence; system exports only report access, notice, dispute, and correction events.
- FED-EXAMPLE-024: A processor requests law-specific exemption; system records entity/data exemption and residual state duties.
- FED-EXAMPLE-025: A federal law retention override blocks deletion; system records override and subject-facing explanation.

## Cross-References

- FED-XREF-001: `packs/us-localization/README.md` defines pack precedence and activated microservices.
- FED-XREF-002: `packs/us-localization/hipaa-phi-handling.md` expands HIPAA PHI, minimum necessary, BAA, and de-identification requirements.
- FED-XREF-003: `packs/us-localization/state-privacy-laws-comparison.md` explains residual state privacy obligations after sector routing.
- FED-XREF-004: `packs/us-localization/sox-and-financial-reporting.md` handles financial-reporting controls outside GLBA and ECOA.
- FED-XREF-005: `packs/us-localization/discrimination-laws-and-ai-bias.md` handles employment selection, AEDT, and AI bias overlays.
- FED-XREF-006: `specs/microservices/accounting.json` is the likely host for GLBA, FCRA, ECOA, and SOX financial workflows.
- FED-XREF-007: `specs/microservices/hr.json` is the likely host for FCRA employment screening and selection logic.
- FED-XREF-008: `specs/microservices/payroll.json` is the likely host for employment and wage records that may coexist with FCRA/EEOC workflows.
- FED-XREF-009: `specs/microservices/identity.json` is the likely host for parent, guardian, authorized-agent, and identity verification.
- FED-XREF-010: `specs/microservices/governance.json` is the likely host for authority resolution and processor contracts.
- FED-XREF-011: `specs/microservices/audit-chain` catalog entries are future hosts for federal audit events.
- FED-XREF-012: `registry/catalog/check-data-class.yaml` is a future gate for federal data labels.
- FED-XREF-013: `registry/catalog/check-compliance-evidence-coverage.yaml` is a future gate for federal evidence completeness.
- FED-XREF-014: `registry/catalog/check-cedar-fragment-coverage.yaml` is a future gate for executable Cedar fragments.
- FED-XREF-015: Official HHS HIPAA guidance must remain the primary source for HIPAA rule interpretation.
- FED-XREF-016: Official eCFR pages must remain the current citation source for CFR parts.
- FED-XREF-017: FTC sources remain primary for GLBA Safeguards and FCRA statutory reference material where FTC jurisdiction applies.
- FED-XREF-018: CFPB Regulation B and Regulation V pages remain primary for ECOA and FCRA regulatory implementation.
- FED-XREF-019: Department of Education and eCFR sources remain primary for FERPA.
- FED-XREF-020: FTC COPPA and eCFR Part 312 remain primary for COPPA.

## Federal Implementation Checkpoint

- FED-CHECK-001: HIPAA route must be decided before generic state privacy route.
- FED-CHECK-002: HIPAA route must classify covered entity, business associate, subcontractor, or non-HIPAA health service.
- FED-CHECK-003: HIPAA route must distinguish PHI, ePHI, limited data set, and de-identified health information.
- FED-CHECK-004: HIPAA route must record treatment, payment, operations, authorization, required-by-law, or other permitted basis.
- FED-CHECK-005: HIPAA route must record whether minimum necessary applies.
- FED-CHECK-006: HIPAA route must record BAA status before processor access.
- FED-CHECK-007: HIPAA route must record de-identification method before analytics reuse.
- FED-CHECK-008: HIPAA route must record breach assessment state after impermissible use or disclosure.
- FED-CHECK-009: GLBA route must be decided before generic financial-data route.
- FED-CHECK-010: GLBA route must classify financial institution, customer, consumer, service provider, NPI, and customer information.
- FED-CHECK-011: GLBA route must record privacy notice delivery.
- FED-CHECK-012: GLBA route must record opt-out state where nonaffiliate sharing is covered.
- FED-CHECK-013: GLBA route must record Safeguards Rule security program evidence.
- FED-CHECK-014: GLBA route must record customer-information incident evidence.
- FED-CHECK-015: GLBA route must not be used as SOX ICFR certification evidence without explicit crosswalk.
- FED-CHECK-016: FERPA route must classify educational agency, institution, school official, parent, eligible student, education record, and PII.
- FED-CHECK-017: FERPA route must record consent, directory-information basis, school-official exception, audit/evaluation basis, or other exception.
- FED-CHECK-018: FERPA route must record redisclosure restrictions.
- FED-CHECK-019: FERPA route must record amendment-request workflow.
- FED-CHECK-020: FERPA route must resolve student-health boundary before HIPAA route.
- FED-CHECK-021: COPPA route must classify child-directed service, actual knowledge, child under 13, operator, parent, and verifiable parental consent.
- FED-CHECK-022: COPPA route must record notice to parent before collection.
- FED-CHECK-023: COPPA route must record parental consent method.
- FED-CHECK-024: COPPA route must record parent access, deletion, and collection refusal.
- FED-CHECK-025: COPPA route must record data minimization and retention limits for child data.
- FED-CHECK-026: COPPA route must not be replaced by state teen or sensitive-data rules.
- FED-CHECK-027: FCRA route must classify consumer report, consumer reporting agency, furnisher, user, permissible purpose, and adverse action.
- FED-CHECK-028: FCRA route must record employment screening authorization and disclosure where employment screening is in scope.
- FED-CHECK-029: FCRA route must record pre-adverse and adverse-action notices where required.
- FED-CHECK-030: FCRA route must record dispute and reinvestigation workflow.
- FED-CHECK-031: FCRA route must record furnisher correction workflow.
- FED-CHECK-032: FCRA route must not be collapsed into generic privacy access because consumer-report rights are specialized.
- FED-CHECK-033: ECOA route must classify creditor, applicant, credit transaction, prohibited basis, and adverse action.
- FED-CHECK-034: ECOA route must record principal reasons for adverse action.
- FED-CHECK-035: ECOA route must record protected-basis data segregation.
- FED-CHECK-036: ECOA route must record special-purpose credit program basis where applicable.
- FED-CHECK-037: ECOA route must record appraisal, notification, and monitoring evidence where applicable.
- FED-CHECK-038: ECOA route must not allow model explanations to replace specific adverse-action reasons.
- FED-CHECK-039: Compound HIPAA plus GLBA workflows must stack PHI and NPI labels.
- FED-CHECK-040: Compound FCRA plus ECOA workflows must stack permissible-purpose and adverse-action labels.
- FED-CHECK-041: Compound FERPA plus COPPA workflows must stack eligible-student and parent-consent labels.
- FED-CHECK-042: Compound state privacy plus HIPAA workflows must record exemption or residual-duty explanation.
- FED-CHECK-043: Compound state privacy plus GLBA workflows must record exemption or residual-duty explanation.
- FED-CHECK-044: Compound state privacy plus FCRA workflows must preserve consumer-report dispute and correction duties.
- FED-CHECK-045: Compound state privacy plus ECOA workflows must preserve adverse-action duties.
- FED-CHECK-046: Compound employment screening workflows must route through FCRA, Title VII, ADA, UGESP, and state privacy where triggered.
- FED-CHECK-047: Every federal route must emit authority-resolution audit evidence.
- FED-CHECK-048: Every federal denial must include controlling authority and user-facing safe explanation.
- FED-CHECK-049: Every federal export must carry scoped recipient, purpose, authority, and hash.
- FED-CHECK-050: Every federal deletion refusal must record retention authority.
- FED-CHECK-051: Every federal retention override must be visible to privacy request workflows.
- FED-CHECK-052: Every federal consent must be revocable unless law or contract controls otherwise.
- FED-CHECK-053: Every federal authorization must be versioned.
- FED-CHECK-054: Every federal notice must be versioned.
- FED-CHECK-055: Every federal data class must carry source law and confidence.
- FED-CHECK-056: Every federal route must record `law_refreshed_at`.
- FED-CHECK-057: Every federal route must record counsel-review state before production.
- FED-CHECK-058: Every federal processor handoff must record contract authority.
- FED-CHECK-059: Every federal processor handoff must record downstream subprocessors where required.
- FED-CHECK-060: Every federal processor instruction must be rejectable when it conflicts with law.
- FED-CHECK-061: Every federal audit event must retain actor, tenant, subject, object, purpose, authority, service, result, and evidence id.
- FED-CHECK-062: Every federal model-training request must prove data-use authority and minimization.
- FED-CHECK-063: Every federal AI decision route must check FCRA, ECOA, FERPA, HIPAA, COPPA, and discrimination overlays.
- FED-CHECK-064: Every federal child-data route must check COPPA before state minor-specific overlays.
- FED-CHECK-065: Every federal education route must check FERPA before generic document permissions.
- FED-CHECK-066: Every federal healthcare route must check HIPAA before generic health-data tags.
- FED-CHECK-067: Every federal financial privacy route must check GLBA before generic financial-data tags.
- FED-CHECK-068: Every federal credit-reporting route must check FCRA before generic risk-scoring tags.
- FED-CHECK-069: Every federal credit-decision route must check ECOA before generic model-explanation tags.
- FED-CHECK-070: Every future Cedar fragment must cite the specific federal checkpoint line it implements.
- FED-CHECK-071: Every future schema migration must cite the specific federal data delta it implements.
- FED-CHECK-072: Every future API route must cite the specific federal API delta it implements.
- FED-CHECK-073: Every future audit event registry entry must cite the specific federal audit addition it implements.
- FED-CHECK-074: Every future regression test must cite a failure mode or worked example.
- FED-CHECK-075: Federal checkpoint name is `us-localization-pack-w1-2026-05-20`.
- FED-CHECK-076: Federal checkpoint status is documentation-authored and implementation-pending.
- FED-CHECK-077: Federal checkpoint evidence token is `us_pack_docs:6`.
- FED-CHECK-078: Federal checkpoint should fail verification if this file drops below requested line depth.
- FED-CHECK-079: Federal checkpoint should fail verification if required headings are removed.
- FED-CHECK-080: Federal checkpoint should fail verification if frontmatter loses pack identity.
- FED-CHECK-081: Federal checkpoint should fail verification if official citations are replaced by vendor summaries.
- FED-CHECK-082: Federal checkpoint should fail verification if AB-style advisory language is imported into binding federal law.
- FED-CHECK-083: Federal checkpoint should fail verification if sector labels are flattened into `personal_data`.
- FED-CHECK-084: Federal checkpoint should fail verification if audit-chain evidence ids are optional.
- FED-CHECK-085: Federal checkpoint should fail verification if counsel review is represented as already complete.
- FED-CHECK-086: Federal checkpoint should fail verification if production readiness is claimed.
- FED-CHECK-087: Federal checkpoint should fail verification if other localization packs are modified by this slice.
- FED-CHECK-088: Federal checkpoint should fail verification if ADR files are modified by this slice.
- FED-CHECK-089: Federal checkpoint should fail verification if microservice manifests are modified by this slice.
- FED-CHECK-090: Federal checkpoint stop condition is VCS promote accepted or clean VCS blocker reported.
- FED-CHECK-091: Federal checkpoint final report must list six US pack documents.
- FED-CHECK-092: Federal checkpoint final report must include line-count evidence.
- FED-CHECK-093: Federal checkpoint final report must include heading-contract evidence.
- FED-CHECK-094: Federal checkpoint final report must include exact VCS lifecycle status.
- FED-CHECK-095: Federal checkpoint final report must identify no-touch scopes.
- FED-CHECK-096: Federal checkpoint final report must cite official source families.
- FED-CHECK-097: Federal checkpoint final report must not overstate legal correctness.
- FED-CHECK-098: Federal checkpoint final report must document remaining implementation risk.
- FED-CHECK-099: Federal checkpoint final report must halt after promote rather than starting implementation.
- FED-CHECK-100: Federal checkpoint final report must preserve the user-requested positional pack scope.
