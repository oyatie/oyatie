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
  - https://www.eeoc.gov/statutes/title-vii-civil-rights-act-1964
  - https://www.eeoc.gov/laws/guidance/uniform-guidelines-employee-selection-procedures
  - https://www.eeoc.gov/statutes/titles-i-and-v-americans-disabilities-act-1990-ada
  - https://www.nyc.gov/site/dca/about/automated-employment-decision-tools.page
  - https://rules.cityofnewyork.us/rule/automated-employment-decision-tools/
  - https://leg.colorado.gov/bills/sb24-205
  - https://leginfo.legislature.ca.gov/faces/billNavClient.xhtml?bill_id=202320240AB331
  - https://www.ecfr.gov/current/title-41/subtitle-B/chapter-60/part-60-3
---

# Discrimination Laws and AI Bias

This document maps U.S. discrimination, accessibility, employee-selection, and automated-decision duties into US-PACK-1.
The document covers EEOC UGESP, the four-fifths rule, Title VII, ADA Title I, NYC Local Law 144, Colorado SB 24-205, and California AB-331.
The document treats California AB-331 as a bill-reference and design-risk input unless a later law refresh confirms enactment or replacement.
The document treats Colorado SB 24-205 as a high-risk AI overlay for consequential decisions.
The document treats NYC Local Law 144 as a binding city-specific automated-employment-decision-tool overlay when the tenant is in scope.
The document keeps discrimination and AI-bias controls separate from state consumer privacy profiling opt-outs.
The document keeps civil-rights duties separate from SOC 2, NIST 800-53, and FedRAMP security-control evidence.
The document does not author executable Cedar in this slice.
The document does not modify microservices in this slice.
The document requires employment counsel, civil-rights counsel, and local-law review before production enforcement.

## Authority Citations

- BIAS-AUTH-001: Title VII authority is the Civil Rights Act of 1964 Title VII as published by the EEOC.
- BIAS-AUTH-002: Title VII prohibits employment discrimination based on race, color, religion, sex, and national origin.
- BIAS-AUTH-003: Title VII covers disparate treatment and disparate impact theories.
- BIAS-AUTH-004: Title VII applies to covered employers and covered employment practices.
- BIAS-AUTH-005: Title VII authority must be evaluated together with EEOC guidance and applicable case law.
- BIAS-AUTH-006: ADA Title I authority is the Americans with Disabilities Act employment title as published by EEOC.
- BIAS-AUTH-007: ADA Title I prohibits discrimination against qualified individuals with disabilities in employment.
- BIAS-AUTH-008: ADA Title I requires reasonable accommodation unless undue hardship applies.
- BIAS-AUTH-009: ADA Title I constrains medical examinations and disability-related inquiries.
- BIAS-AUTH-010: ADA Title I requires accessible processes where application, testing, interview, or employment systems create barriers.
- BIAS-AUTH-011: UGESP authority is the Uniform Guidelines on Employee Selection Procedures.
- BIAS-AUTH-012: UGESP appears in 29 CFR Part 1607 and parallel federal personnel/regulatory locations.
- BIAS-AUTH-013: UGESP Section 4 establishes adverse-impact information expectations.
- BIAS-AUTH-014: UGESP Section 4D describes the four-fifths rule as a general rule of thumb for selection-rate comparison.
- BIAS-AUTH-015: UGESP Section 5 covers general standards for validity studies.
- BIAS-AUTH-016: UGESP Section 5A explains that selection procedures with adverse impact should be validated or justified under the guidelines.
- BIAS-AUTH-017: UGESP validation evidence includes criterion-related, content, and construct validity strategies where appropriate.
- BIAS-AUTH-018: UGESP does not convert four-fifths calculations into a complete legal conclusion.
- BIAS-AUTH-019: Four-fifths output is screening evidence that triggers review, not automatic illegality.
- BIAS-AUTH-020: NYC Local Law 144 regulates automated employment decision tools for employment decisions in New York City.
- BIAS-AUTH-021: NYC Local Law 144 requires a bias audit before covered AEDT use.
- BIAS-AUTH-022: NYC Local Law 144 requires the bias audit to be independent.
- BIAS-AUTH-023: NYC Local Law 144 requires public availability of audit results or summary information.
- BIAS-AUTH-024: NYC Local Law 144 requires candidate or employee notice where the law applies.
- BIAS-AUTH-025: NYC DCWP rules define AEDT and specify bias-audit calculations and notice mechanics.
- BIAS-AUTH-026: Colorado SB 24-205 created a state AI accountability law for high-risk artificial intelligence systems.
- BIAS-AUTH-027: Colorado SB 24-205 defines high-risk AI systems around consequential decisions or substantial factors in consequential decisions.
- BIAS-AUTH-028: Colorado SB 24-205 covers consequential decision domains including education, employment, financial or lending services, essential government services, health care, housing, insurance, and legal services.
- BIAS-AUTH-029: Colorado SB 24-205 creates developer and deployer duties and uses reasonable-care concepts to avoid algorithmic discrimination.
- BIAS-AUTH-030: Colorado SB 24-205 requires risk-management and impact-assessment style evidence for deployers of high-risk systems.
- BIAS-AUTH-031: California AB-331 is cited by user request as an automated decision tool bill reference.
- BIAS-AUTH-032: California AB-331 must be treated as advisory unless current official bill status shows enforceable law.
- BIAS-AUTH-033: AB-331-style automated decision tool definitions remain useful for design risk taxonomy even when not binding.
- BIAS-AUTH-034: FCRA may apply when employment screening uses consumer reports.
- BIAS-AUTH-035: ECOA may apply when AI affects credit decisions.
- BIAS-AUTH-036: Fair housing laws may apply when AI affects housing decisions; this pack flags but does not fully author housing-law controls.
- BIAS-AUTH-037: ADA may apply when AI tools screen out disabled applicants or fail to provide reasonable accommodation.
- BIAS-AUTH-038: State privacy profiling opt-outs may apply alongside discrimination duties.
- BIAS-AUTH-039: SOC 2 Trust Services Criteria can provide assurance evidence but do not prove civil-rights compliance.
- BIAS-AUTH-040: NIST 800-53 and FedRAMP can provide security-control evidence but do not prove anti-discrimination compliance.
- BIAS-AUTH-041: ITAR can constrain access to defense technical data used in model training but does not replace bias obligations.
- BIAS-AUTH-042: Oyatie must not allow a general model-risk label to hide the specific protected class, selection procedure, decision domain, and authority basis.
- BIAS-AUTH-043: Oyatie must record whether a tool is used to substantially assist or replace discretionary decision-making.
- BIAS-AUTH-044: Oyatie must distinguish recommendation, ranking, scoring, screening, classification, and final decision.
- BIAS-AUTH-045: Oyatie must retain human-review and accommodation evidence for high-risk employment systems.

## Scope and Routing

- BIAS-SCOPE-001: Route to Title VII when the decision domain is employment and protected-class discrimination risk exists.
- BIAS-SCOPE-002: Route to ADA when the decision domain is employment and disability, accommodation, accessibility, medical inquiry, or screen-out risk exists.
- BIAS-SCOPE-003: Route to UGESP when a selection procedure affects hiring, promotion, transfer, training, discipline, layoff, or other covered employment selection.
- BIAS-SCOPE-004: Route to four-fifths analysis when selection rates can be computed for protected groups and comparator groups.
- BIAS-SCOPE-005: Route to NYC LL144 when AEDT is used for employment decision-making in New York City and the tenant meets scope.
- BIAS-SCOPE-006: Route to Colorado SB 24-205 when the AI system is high-risk and affects consequential decision domains.
- BIAS-SCOPE-007: Route to California AB-331 advisory taxonomy when tenant policy requests design-risk tracking or future-law readiness.
- BIAS-SCOPE-008: Route to FCRA when employment screening uses consumer reports.
- BIAS-SCOPE-009: Route to ECOA when the AI system affects credit decisions.
- BIAS-SCOPE-010: Route to state privacy profiling opt-out when consumer profiling laws are triggered.
- BIAS-SCOPE-011: Route to HIPAA if health data is used by a covered entity or business associate.
- BIAS-SCOPE-012: Route to FERPA if education-record data is used in education decisions.
- BIAS-SCOPE-013: Route to COPPA if child personal data is used in a child-directed service.
- BIAS-SCOPE-014: Route to GLBA if financial institution customer information is used.
- BIAS-SCOPE-015: Route to ITAR if defense technical data or foreign-person access restrictions are involved.
- BIAS-SCOPE-016: Route to NIST 800-53 only for control evidence, not civil-rights conclusions.
- BIAS-SCOPE-017: Route to FedRAMP only for cloud authorization evidence, not civil-rights conclusions.
- BIAS-SCOPE-018: Route to SOC 2 only for assurance evidence, not civil-rights conclusions.
- BIAS-SCOPE-019: Route to counsel review when protected-class data is unavailable because missing labels can hide adverse impact.
- BIAS-SCOPE-020: Route to manual review when automated denial produces legal or similarly significant effect.

## EEOC UGESP Four-Fifths Rule

- UGESP-001: Compute group selection rate as selected_count divided by eligible_or_applicant_count.
- UGESP-002: Identify the highest selection rate among comparator groups.
- UGESP-003: Divide each group selection rate by the highest selection rate.
- UGESP-004: Flag adverse-impact concern when the ratio is less than four-fifths or 0.80.
- UGESP-005: Treat four-fifths as a general rule of thumb.
- UGESP-006: Do not treat four-fifths as a complete legal finding.
- UGESP-007: Preserve sample size because small samples can distort ratios.
- UGESP-008: Preserve job, requisition, cohort, period, and selection procedure.
- UGESP-009: Preserve selection stage such as resume screen, assessment, interview, ranking, offer, or promotion.
- UGESP-010: Preserve protected-group dimension where legally available and authorized.
- UGESP-011: Preserve comparator group basis.
- UGESP-012: Preserve missing demographic data handling.
- UGESP-013: Preserve voluntary self-identification source where used.
- UGESP-014: Preserve data minimization and access limits for protected-class data.
- UGESP-015: Preserve whether group labels were used only for audit and not selection.
- UGESP-016: Preserve whether the procedure was automated, manual, or mixed.
- UGESP-017: Preserve whether procedure has validation evidence.
- UGESP-018: Preserve criterion-related validity evidence where claimed.
- UGESP-019: Preserve content validity evidence where claimed.
- UGESP-020: Preserve construct validity evidence where claimed.
- UGESP-021: Preserve business necessity or job-relatedness review.
- UGESP-022: Preserve alternative procedure review where adverse impact exists.
- UGESP-023: Preserve accommodation availability when assessment may screen out disabled applicants.
- UGESP-024: Preserve retest or appeal route.
- UGESP-025: Preserve reviewer identity.
- UGESP-026: Preserve model version.
- UGESP-027: Preserve threshold version.
- UGESP-028: Preserve feature set.
- UGESP-029: Preserve training-data lineage.
- UGESP-030: Preserve validation population.
- UGESP-031: Preserve drift-monitoring evidence.
- UGESP-032: Preserve calibration evidence.
- UGESP-033: Preserve false-positive and false-negative analysis where available.
- UGESP-034: Preserve subgroup stability analysis.
- UGESP-035: Preserve exception handling.
- UGESP-036: Preserve final decision maker.
- UGESP-037: Preserve human override reason.
- UGESP-038: Preserve candidate notice where required.
- UGESP-039: Preserve audit publication link where required.
- UGESP-040: Preserve remediation plan when adverse impact is flagged.

## NYC Local Law 144 AEDT Handling

- NYC-001: Determine whether the tool is an automated employment decision tool under DCWP rules.
- NYC-002: Determine whether the tool is used to substantially assist or replace discretionary decision making.
- NYC-003: Determine whether the employment decision involves hiring or promotion.
- NYC-004: Determine whether the job location or candidate/employee connection triggers New York City coverage.
- NYC-005: Require independent bias audit before covered use.
- NYC-006: Require audit recency within the required period before use.
- NYC-007: Require public summary or public posting where required.
- NYC-008: Require notice to candidates or employees where required.
- NYC-009: Require instructions for requesting alternative selection process or accommodation where required.
- NYC-010: Require data retention for AEDT audit evidence.
- NYC-011: Require source of demographic categories used in audit.
- NYC-012: Require selection rate calculations.
- NYC-013: Require impact ratio calculations.
- NYC-014: Require scoring-rate calculations where applicable under rules.
- NYC-015: Require independent-auditor identity.
- NYC-016: Require audit date.
- NYC-017: Require AEDT version.
- NYC-018: Require job category or employment decision category.
- NYC-019: Require publication URL.
- NYC-020: Require exception handling when historical data is insufficient.
- NYC-021: Block AEDT use if independent audit is missing.
- NYC-022: Block AEDT use if audit is stale.
- NYC-023: Block AEDT use if notice is missing.
- NYC-024: Block AEDT use if public posting is missing where required.
- NYC-025: Block AEDT use if model version differs materially from audited version.
- NYC-026: Block AEDT use if scope changes from hiring to promotion without audit.
- NYC-027: Block AEDT use if accommodation route is missing.
- NYC-028: Block AEDT use if candidate data is repurposed beyond notice.
- NYC-029: Emit AEDT audit acceptance event.
- NYC-030: Emit AEDT use denial event.

## Colorado SB 24-205 High-Risk AI Handling

- COAI-001: Determine whether the system is an artificial intelligence system under Colorado terminology.
- COAI-002: Determine whether the system is high-risk.
- COAI-003: High-risk status includes systems that make or are a substantial factor in consequential decisions.
- COAI-004: Consequential decision domains include employment.
- COAI-005: Consequential decision domains include education enrollment or opportunity.
- COAI-006: Consequential decision domains include financial or lending services.
- COAI-007: Consequential decision domains include essential government services.
- COAI-008: Consequential decision domains include health care services.
- COAI-009: Consequential decision domains include housing.
- COAI-010: Consequential decision domains include insurance.
- COAI-011: Consequential decision domains include legal services.
- COAI-012: Determine whether Oyatie tenant is developer, deployer, or both.
- COAI-013: Require reasonable-care evidence to avoid algorithmic discrimination.
- COAI-014: Require risk-management policy evidence.
- COAI-015: Require impact assessment evidence for deployers where required.
- COAI-016: Require known or reasonably foreseeable risk disclosure handling.
- COAI-017: Require consumer notice where required.
- COAI-018: Require adverse-decision explanation where required.
- COAI-019: Require appeal or correction route where required.
- COAI-020: Require human review route where required.
- COAI-021: Require model purpose statement.
- COAI-022: Require intended-use statement.
- COAI-023: Require prohibited-use statement.
- COAI-024: Require training-data summary where required.
- COAI-025: Require evaluation-data summary where required.
- COAI-026: Require performance and limitation evidence.
- COAI-027: Require bias and discrimination testing evidence.
- COAI-028: Require post-deployment monitoring.
- COAI-029: Require incident reporting workflow.
- COAI-030: Block high-risk system deployment without impact assessment.
- COAI-031: Block substantial decision automation without domain classification.
- COAI-032: Block use outside intended domain.
- COAI-033: Block use after material model change until reassessment.
- COAI-034: Block use when consumer notice is missing.
- COAI-035: Block use when human review route is missing where required.
- COAI-036: Emit high-risk classification event.
- COAI-037: Emit impact assessment completion event.
- COAI-038: Emit material modification event.
- COAI-039: Emit algorithmic discrimination risk event.
- COAI-040: Emit Colorado AI enforcement export event.

## California AB-331 Advisory Handling

- CAADT-001: Treat AB-331 as advisory until current official bill status confirms enforceable law.
- CAADT-002: Record AB-331 reference id when tenant policy wants California automated-decision readiness.
- CAADT-003: Record automated decision tool definition used for design review.
- CAADT-004: Record decision domain.
- CAADT-005: Record consequential decision effect.
- CAADT-006: Record impact assessment draft.
- CAADT-007: Record data source inventory.
- CAADT-008: Record data quality review.
- CAADT-009: Record discrimination-risk review.
- CAADT-010: Record accessibility-risk review.
- CAADT-011: Record notice design.
- CAADT-012: Record opt-out or human-review design if tenant policy requires it.
- CAADT-013: Record model version.
- CAADT-014: Record vendor identity.
- CAADT-015: Record procurement review.
- CAADT-016: Record public-summary readiness.
- CAADT-017: Record counsel status.
- CAADT-018: Do not block production solely under AB-331 unless tenant policy makes it binding.
- CAADT-019: Do not represent AB-331 advisory records as enacted-law compliance.
- CAADT-020: Refresh California law status before any California automated-decision enforcement claim.

## Activated Cedar Policies

- BIAS-CEDAR-001: `us.bias.require_decision_domain_classification`.
- BIAS-CEDAR-002: `us.bias.require_protected_class_audit_boundary`.
- BIAS-CEDAR-003: `us.bias.require_selection_procedure_inventory`.
- BIAS-CEDAR-004: `us.bias.require_ugesp_adverse_impact_analysis`.
- BIAS-CEDAR-005: `us.bias.require_four_fifths_ratio`.
- BIAS-CEDAR-006: `us.bias.require_validation_evidence_for_adverse_impact`.
- BIAS-CEDAR-007: `us.bias.require_alternative_procedure_review`.
- BIAS-CEDAR-008: `us.bias.require_title_vii_review`.
- BIAS-CEDAR-009: `us.bias.require_ada_accessibility_review`.
- BIAS-CEDAR-010: `us.bias.require_reasonable_accommodation_route`.
- BIAS-CEDAR-011: `us.bias.block_medical_inquiry_without_authority`.
- BIAS-CEDAR-012: `us.bias.require_nyc_ll144_scope_check`.
- BIAS-CEDAR-013: `us.bias.require_nyc_ll144_independent_bias_audit`.
- BIAS-CEDAR-014: `us.bias.require_nyc_ll144_candidate_notice`.
- BIAS-CEDAR-015: `us.bias.require_nyc_ll144_public_summary`.
- BIAS-CEDAR-016: `us.bias.require_colorado_ai_high_risk_classification`.
- BIAS-CEDAR-017: `us.bias.require_colorado_ai_impact_assessment`.
- BIAS-CEDAR-018: `us.bias.require_colorado_ai_human_review_route`.
- BIAS-CEDAR-019: `us.bias.require_colorado_ai_consumer_notice`.
- BIAS-CEDAR-020: `us.bias.require_california_ab331_advisory_status`.
- BIAS-CEDAR-021: `us.bias.block_high_risk_ai_without_domain`.
- BIAS-CEDAR-022: `us.bias.block_aedt_without_required_audit`.
- BIAS-CEDAR-023: `us.bias.block_selection_tool_after_material_unaudited_change`.
- BIAS-CEDAR-024: `us.bias.block_unvalidated_adverse_impact_procedure`.
- BIAS-CEDAR-025: `us.bias.require_fcra_route_for_employment_screening`.
- BIAS-CEDAR-026: `us.bias.require_ecoa_route_for_credit_ai`.
- BIAS-CEDAR-027: `us.bias.require_ferpa_route_for_education_ai`.
- BIAS-CEDAR-028: `us.bias.require_hipaa_route_for_health_ai`.
- BIAS-CEDAR-029: `us.bias.require_state_privacy_profiling_route`.
- BIAS-CEDAR-030: `us.bias.require_model_monitoring_for_drift`.
- BIAS-CEDAR-031: `us.bias.require_training_data_lineage`.
- BIAS-CEDAR-032: `us.bias.require_vendor_ai_risk_attestation`.
- BIAS-CEDAR-033: `us.bias.require_human_override_reason`.
- BIAS-CEDAR-034: `us.bias.require_candidate_or_consumer_explanation`.
- BIAS-CEDAR-035: `us.bias.require_counsel_review_for_binding_claim`.

## Data Model Deltas

- BIAS-DATA-001: Add `decision_domain`.
- BIAS-DATA-002: Add `decision_effect_level`.
- BIAS-DATA-003: Add `selection_procedure_id`.
- BIAS-DATA-004: Add `selection_stage`.
- BIAS-DATA-005: Add `protected_class_dimension`.
- BIAS-DATA-006: Add `protected_class_data_source`.
- BIAS-DATA-007: Add `protected_class_data_access_purpose`.
- BIAS-DATA-008: Add `eligible_count`.
- BIAS-DATA-009: Add `selected_count`.
- BIAS-DATA-010: Add `selection_rate`.
- BIAS-DATA-011: Add `highest_selection_rate`.
- BIAS-DATA-012: Add `impact_ratio`.
- BIAS-DATA-013: Add `four_fifths_flag`.
- BIAS-DATA-014: Add `sample_size_warning`.
- BIAS-DATA-015: Add `validation_strategy`.
- BIAS-DATA-016: Add `validation_study_id`.
- BIAS-DATA-017: Add `business_necessity_review_id`.
- BIAS-DATA-018: Add `alternative_procedure_review_id`.
- BIAS-DATA-019: Add `reasonable_accommodation_route_id`.
- BIAS-DATA-020: Add `accessibility_barrier_id`.
- BIAS-DATA-021: Add `medical_inquiry_authority`.
- BIAS-DATA-022: Add `aedt_scope_state`.
- BIAS-DATA-023: Add `nyc_ll144_bias_audit_id`.
- BIAS-DATA-024: Add `nyc_ll144_independent_auditor_id`.
- BIAS-DATA-025: Add `nyc_ll144_public_summary_url`.
- BIAS-DATA-026: Add `nyc_ll144_notice_id`.
- BIAS-DATA-027: Add `colorado_ai_role`.
- BIAS-DATA-028: Add `colorado_high_risk_system_state`.
- BIAS-DATA-029: Add `colorado_consequential_decision_domain`.
- BIAS-DATA-030: Add `colorado_impact_assessment_id`.
- BIAS-DATA-031: Add `algorithmic_discrimination_risk_id`.
- BIAS-DATA-032: Add `human_review_route_id`.
- BIAS-DATA-033: Add `consumer_or_candidate_notice_id`.
- BIAS-DATA-034: Add `model_version_id`.
- BIAS-DATA-035: Add `model_material_change_id`.
- BIAS-DATA-036: Add `training_data_lineage_id`.
- BIAS-DATA-037: Add `evaluation_data_lineage_id`.
- BIAS-DATA-038: Add `drift_monitoring_id`.
- BIAS-DATA-039: Add `vendor_ai_attestation_id`.
- BIAS-DATA-040: Add `california_ab331_status`.
- BIAS-DATA-041: Add `california_ab331_advisory_assessment_id`.
- BIAS-DATA-042: Add `fcra_employment_screening_route`.
- BIAS-DATA-043: Add `ecoa_credit_ai_route`.
- BIAS-DATA-044: Add `ferpa_education_ai_route`.
- BIAS-DATA-045: Add `hipaa_health_ai_route`.
- BIAS-DATA-046: Add `state_privacy_profiling_route`.
- BIAS-DATA-047: Add `human_override_reason`.
- BIAS-DATA-048: Add `decision_explanation_id`.
- BIAS-DATA-049: Add `counsel_review_id`.
- BIAS-DATA-050: Add `binding_claim_allowed`.

## API Contract Deltas

- BIAS-API-001: Add `POST /compliance/us/bias/decision-domains`.
- BIAS-API-002: Add `POST /compliance/us/bias/selection-procedures`.
- BIAS-API-003: Add `POST /compliance/us/bias/ugesp/four-fifths`.
- BIAS-API-004: Add `POST /compliance/us/bias/ugesp/validation-studies`.
- BIAS-API-005: Add `POST /compliance/us/bias/ugesp/alternative-procedure-reviews`.
- BIAS-API-006: Add `POST /compliance/us/bias/title-vii/reviews`.
- BIAS-API-007: Add `POST /compliance/us/bias/ada/accessibility-reviews`.
- BIAS-API-008: Add `POST /compliance/us/bias/ada/accommodation-routes`.
- BIAS-API-009: Add `POST /compliance/us/bias/ada/medical-inquiry-checks`.
- BIAS-API-010: Add `POST /compliance/us/bias/nyc-ll144/scope-checks`.
- BIAS-API-011: Add `POST /compliance/us/bias/nyc-ll144/bias-audits`.
- BIAS-API-012: Add `POST /compliance/us/bias/nyc-ll144/notices`.
- BIAS-API-013: Add `POST /compliance/us/bias/nyc-ll144/public-summaries`.
- BIAS-API-014: Add `POST /compliance/us/bias/colorado-ai/high-risk-classifications`.
- BIAS-API-015: Add `POST /compliance/us/bias/colorado-ai/impact-assessments`.
- BIAS-API-016: Add `POST /compliance/us/bias/colorado-ai/consumer-notices`.
- BIAS-API-017: Add `POST /compliance/us/bias/colorado-ai/human-review-routes`.
- BIAS-API-018: Add `POST /compliance/us/bias/california-ab331/advisory-assessments`.
- BIAS-API-019: Add `POST /compliance/us/bias/model-material-changes`.
- BIAS-API-020: Add `POST /compliance/us/bias/training-data-lineage`.
- BIAS-API-021: Add `POST /compliance/us/bias/drift-monitoring`.
- BIAS-API-022: Add `POST /compliance/us/bias/vendor-attestations`.
- BIAS-API-023: Add `POST /compliance/us/bias/human-overrides`.
- BIAS-API-024: Add `POST /compliance/us/bias/decision-explanations`.
- BIAS-API-025: Add `POST /compliance/us/bias/counsel-reviews`.
- BIAS-API-026: Add `GET /compliance/us/bias/selection-procedures/{id}/risk`.
- BIAS-API-027: Add `GET /compliance/us/bias/aedt/{id}/readiness`.
- BIAS-API-028: Add `GET /compliance/us/bias/high-risk-ai/{id}/readiness`.
- BIAS-API-029: Add `binding_claim_allowed` to readiness responses.
- BIAS-API-030: Add `evidence_event_ids` to every bias mutation response.

## Audit Event Additions

- BIAS-AUDIT-001: `us.bias.decision_domain_classified`.
- BIAS-AUDIT-002: `us.bias.selection_procedure_registered`.
- BIAS-AUDIT-003: `us.bias.protected_class_audit_boundary_recorded`.
- BIAS-AUDIT-004: `us.bias.four_fifths_calculated`.
- BIAS-AUDIT-005: `us.bias.adverse_impact_flagged`.
- BIAS-AUDIT-006: `us.bias.validation_study_recorded`.
- BIAS-AUDIT-007: `us.bias.alternative_procedure_reviewed`.
- BIAS-AUDIT-008: `us.bias.title_vii_review_recorded`.
- BIAS-AUDIT-009: `us.bias.ada_accessibility_review_recorded`.
- BIAS-AUDIT-010: `us.bias.reasonable_accommodation_route_recorded`.
- BIAS-AUDIT-011: `us.bias.medical_inquiry_blocked`.
- BIAS-AUDIT-012: `us.bias.nyc_ll144_scope_checked`.
- BIAS-AUDIT-013: `us.bias.nyc_ll144_bias_audit_accepted`.
- BIAS-AUDIT-014: `us.bias.nyc_ll144_bias_audit_rejected`.
- BIAS-AUDIT-015: `us.bias.nyc_ll144_notice_recorded`.
- BIAS-AUDIT-016: `us.bias.nyc_ll144_public_summary_recorded`.
- BIAS-AUDIT-017: `us.bias.colorado_ai_high_risk_classified`.
- BIAS-AUDIT-018: `us.bias.colorado_ai_impact_assessment_completed`.
- BIAS-AUDIT-019: `us.bias.colorado_ai_consumer_notice_recorded`.
- BIAS-AUDIT-020: `us.bias.colorado_ai_human_review_route_recorded`.
- BIAS-AUDIT-021: `us.bias.algorithmic_discrimination_risk_recorded`.
- BIAS-AUDIT-022: `us.bias.california_ab331_advisory_assessment_recorded`.
- BIAS-AUDIT-023: `us.bias.model_material_change_recorded`.
- BIAS-AUDIT-024: `us.bias.training_data_lineage_recorded`.
- BIAS-AUDIT-025: `us.bias.drift_monitoring_recorded`.
- BIAS-AUDIT-026: `us.bias.vendor_ai_attestation_recorded`.
- BIAS-AUDIT-027: `us.bias.human_override_recorded`.
- BIAS-AUDIT-028: `us.bias.decision_explanation_recorded`.
- BIAS-AUDIT-029: `us.bias.counsel_review_recorded`.
- BIAS-AUDIT-030: `us.bias.binding_claim_blocked`.

## Failure Modes

- BIAS-FAIL-001: Tenant deploys employment screen without selection-procedure inventory.
- BIAS-FAIL-002: Tenant computes four-fifths ratio without applicant or eligible denominator.
- BIAS-FAIL-003: Tenant treats four-fifths pass as proof of lawful selection.
- BIAS-FAIL-004: Tenant treats four-fifths fail as automatic legal liability.
- BIAS-FAIL-005: Tenant lacks protected-class audit boundary and cannot test adverse impact.
- BIAS-FAIL-006: Tenant uses protected-class data in selection rather than audit.
- BIAS-FAIL-007: Tenant flags adverse impact but lacks validation study.
- BIAS-FAIL-008: Tenant flags adverse impact but does not review less discriminatory alternatives.
- BIAS-FAIL-009: Tenant uses AI assessment that screens out disabled applicants without accommodation route.
- BIAS-FAIL-010: Tenant asks disability-related questions before authority exists.
- BIAS-FAIL-011: Tenant deploys NYC AEDT without independent bias audit.
- BIAS-FAIL-012: Tenant deploys NYC AEDT with stale audit.
- BIAS-FAIL-013: Tenant deploys NYC AEDT without notice.
- BIAS-FAIL-014: Tenant deploys NYC AEDT with different model version than audited version.
- BIAS-FAIL-015: Tenant treats NYC LL144 audit as national civil-rights clearance.
- BIAS-FAIL-016: Tenant deploys Colorado high-risk AI without domain classification.
- BIAS-FAIL-017: Tenant deploys Colorado high-risk AI without impact assessment.
- BIAS-FAIL-018: Tenant deploys Colorado high-risk AI without consumer notice where required.
- BIAS-FAIL-019: Tenant deploys Colorado high-risk AI without human review route where required.
- BIAS-FAIL-020: Tenant treats Colorado AI evidence as ECOA adverse-action compliance.
- BIAS-FAIL-021: Tenant treats California AB-331 advisory record as binding enacted-law compliance.
- BIAS-FAIL-022: Tenant ignores FCRA for employment screening consumer reports.
- BIAS-FAIL-023: Tenant ignores ECOA for credit AI.
- BIAS-FAIL-024: Tenant ignores FERPA for education AI.
- BIAS-FAIL-025: Tenant ignores HIPAA for health AI.
- BIAS-FAIL-026: Tenant ignores state privacy profiling opt-out for consumer profiling.
- BIAS-FAIL-027: Tenant uses training data with undocumented lineage.
- BIAS-FAIL-028: Tenant changes model materially after audit without reassessment.
- BIAS-FAIL-029: Tenant provides no explanation for adverse automated decision.
- BIAS-FAIL-030: Tenant claims SOC 2, NIST, or FedRAMP proves non-discrimination.
- BIAS-FAIL-031: Tenant claims legal compliance without counsel review.
- BIAS-FAIL-032: Tenant cannot reconstruct which model version made a decision.
- BIAS-FAIL-033: Tenant cannot reconstruct final human decision maker.
- BIAS-FAIL-034: Tenant cannot reconstruct human override reason.
- BIAS-FAIL-035: Tenant cannot separate audit-only protected-class data from operational scoring.

## Worked Examples

- BIAS-EXAMPLE-001: Hiring screen selects 20 percent of Group A and 40 percent of Group B; system computes impact ratio 0.50 and flags adverse-impact review.
- BIAS-EXAMPLE-002: Hiring screen selects 81 percent as much for Group A as the highest group; system records no four-fifths flag but preserves legal-review caveat.
- BIAS-EXAMPLE-003: Sample has three applicants in a group; system records sample-size warning before interpreting ratio.
- BIAS-EXAMPLE-004: Assessment has adverse impact but strong validation study; system records validation evidence and counsel review.
- BIAS-EXAMPLE-005: Assessment has adverse impact and no validation; system blocks production use.
- BIAS-EXAMPLE-006: Alternative procedure has lower adverse impact and similar validity; system routes to selection owner for replacement decision.
- BIAS-EXAMPLE-007: Applicant asks for accessible assessment format; system routes to ADA accommodation workflow.
- BIAS-EXAMPLE-008: Chatbot asks disability-related medical question; system blocks inquiry absent authority.
- BIAS-EXAMPLE-009: NYC employer uses ranking AEDT for promotion; system requires independent bias audit and notice.
- BIAS-EXAMPLE-010: NYC AEDT audit covers old model version; system blocks use after material model update.
- BIAS-EXAMPLE-011: Public audit summary URL is missing; system blocks NYC LL144 readiness.
- BIAS-EXAMPLE-012: Colorado lender uses AI to rank loan eligibility; system routes to Colorado high-risk AI and ECOA.
- BIAS-EXAMPLE-013: Colorado employer uses AI to reject applicants; system routes to Colorado high-risk AI, Title VII, ADA, UGESP, and possibly FCRA.
- BIAS-EXAMPLE-014: Colorado health insurer uses AI for coverage decision; system routes to Colorado high-risk AI and health-sector privacy review.
- BIAS-EXAMPLE-015: California tenant asks for AB-331 assessment; system creates advisory assessment and blocks binding compliance claim.
- BIAS-EXAMPLE-016: Vendor supplies black-box screening model; system requires vendor attestation, data lineage, and auditability before use.
- BIAS-EXAMPLE-017: Model drifts after six months; system opens reassessment and freezes binding readiness claim.
- BIAS-EXAMPLE-018: Human recruiter overrides AI rejection; system records override reason and final decision maker.
- BIAS-EXAMPLE-019: Consumer asks for decision explanation; system returns domain-specific explanation route.
- BIAS-EXAMPLE-020: Employment screen uses credit report; system routes to FCRA employment screening notice and authorization workflow.
- BIAS-EXAMPLE-021: Credit model uses protected-basis proxy; system routes to ECOA and blocks protected-basis feature use.
- BIAS-EXAMPLE-022: Education placement model uses student records; system routes to FERPA boundary review.
- BIAS-EXAMPLE-023: Health triage model uses PHI; system routes to HIPAA authority and minimum-necessary review.
- BIAS-EXAMPLE-024: Consumer profiling opt-out applies to a recommender; system routes to state privacy profiling opt-out.
- BIAS-EXAMPLE-025: SOC 2 report exists; system records assurance evidence but does not mark bias controls compliant.
- BIAS-EXAMPLE-026: FedRAMP authorization exists for cloud host; system records cloud control evidence but does not mark bias controls compliant.
- BIAS-EXAMPLE-027: NIST 800-53 controls cover audit logs; system reuses audit evidence while keeping discrimination review separate.
- BIAS-EXAMPLE-028: ITAR technical data trains a model; system blocks foreign-person access and separately evaluates bias obligations.
- BIAS-EXAMPLE-029: Counsel rejects binding compliance claim; system marks `binding_claim_allowed=false`.
- BIAS-EXAMPLE-030: Regulator requests AEDT evidence; system generates scoped export with audit ids and redactions.

## Cross-References

- BIAS-XREF-001: `packs/us-localization/README.md` defines pack scope and activated microservices.
- BIAS-XREF-002: `packs/us-localization/federal-privacy-laws.md` defines FCRA, ECOA, FERPA, COPPA, HIPAA, and GLBA routing.
- BIAS-XREF-003: `packs/us-localization/state-privacy-laws-comparison.md` defines profiling opt-out and sensitive-data overlays.
- BIAS-XREF-004: `packs/us-localization/sox-and-financial-reporting.md` defines financial-reporting controls that must not be confused with bias controls.
- BIAS-XREF-005: `packs/us-localization/hipaa-phi-handling.md` defines health-data handling for health AI.
- BIAS-XREF-006: `specs/microservices/hr.json` is the likely host for employment-selection workflows.
- BIAS-XREF-007: `specs/microservices/payroll.json` may host employment records and workforce audit evidence.
- BIAS-XREF-008: `specs/microservices/accounting.json` may host credit, lending, and financial-decision workflows.
- BIAS-XREF-009: `specs/microservices/identity.json` may host protected-class audit boundary and candidate identity.
- BIAS-XREF-010: `specs/microservices/governance.json` is the likely host for authority routing and regulator export.
- BIAS-XREF-011: `specs/microservices/audit-chain` registry entries are future hosts for bias audit events.
- BIAS-XREF-012: `registry/catalog/check-high-risk-auto-decision-refusal.yaml` is a future gate for high-risk AI refusal.
- BIAS-XREF-013: `registry/catalog/check-compliance-evidence-coverage.yaml` is a future gate for bias evidence completeness.
- BIAS-XREF-014: `registry/catalog/check-data-class.yaml` is a future gate for protected-class and sensitive-data boundaries.
- BIAS-XREF-015: EEOC UGESP and Title VII sources remain primary for employee-selection and discrimination interpretation.
- BIAS-XREF-016: EEOC ADA sources remain primary for accommodation and disability inquiry interpretation.
- BIAS-XREF-017: NYC DCWP pages and rules remain primary for Local Law 144 AEDT interpretation.
- BIAS-XREF-018: Colorado official bill and law materials remain primary for SB 24-205 interpretation.
- BIAS-XREF-019: California official bill status remains primary for AB-331 advisory or binding treatment.
- BIAS-XREF-020: The checkpoint for this document is `us-localization-pack-w1-2026-05-20`.

## Civil Rights and AI Checkpoint

- BIAS-CHECK-001: Employment-selection workflow must classify decision domain before policy evaluation.
- BIAS-CHECK-002: Employment-selection workflow must classify selection procedure before impact analysis.
- BIAS-CHECK-003: Employment-selection workflow must classify selection stage.
- BIAS-CHECK-004: Employment-selection workflow must classify applicant or employee cohort.
- BIAS-CHECK-005: Employment-selection workflow must classify protected-class audit boundary.
- BIAS-CHECK-006: Employment-selection workflow must ensure protected-class data is audit-only unless legal basis supports other use.
- BIAS-CHECK-007: Four-fifths calculation must preserve numerator and denominator.
- BIAS-CHECK-008: Four-fifths calculation must preserve comparator group.
- BIAS-CHECK-009: Four-fifths calculation must preserve highest selection rate.
- BIAS-CHECK-010: Four-fifths calculation must preserve impact ratio.
- BIAS-CHECK-011: Four-fifths calculation must preserve sample-size warning.
- BIAS-CHECK-012: Four-fifths calculation must preserve caveat that it is not a final legal conclusion.
- BIAS-CHECK-013: UGESP adverse-impact flag must trigger validation review.
- BIAS-CHECK-014: UGESP adverse-impact flag must trigger alternative procedure review.
- BIAS-CHECK-015: UGESP validation review must record criterion-related evidence where used.
- BIAS-CHECK-016: UGESP validation review must record content-validity evidence where used.
- BIAS-CHECK-017: UGESP validation review must record construct-validity evidence where used.
- BIAS-CHECK-018: UGESP validation review must record job-relatedness basis.
- BIAS-CHECK-019: Title VII review must record protected basis and employment practice.
- BIAS-CHECK-020: Title VII review must record disparate-treatment risk when intentional differential treatment is alleged.
- BIAS-CHECK-021: Title VII review must record disparate-impact risk when neutral practice produces adverse impact.
- BIAS-CHECK-022: Title VII review must record business necessity review where applicable.
- BIAS-CHECK-023: Title VII review must record counsel state before binding claim.
- BIAS-CHECK-024: ADA review must record disability-accessibility barrier.
- BIAS-CHECK-025: ADA review must record reasonable-accommodation route.
- BIAS-CHECK-026: ADA review must record undue-hardship determination only with counsel-approved process.
- BIAS-CHECK-027: ADA review must block medical inquiries without authority.
- BIAS-CHECK-028: ADA review must preserve alternative assessment route.
- BIAS-CHECK-029: NYC LL144 scope check must record job location and candidate or employee connection.
- BIAS-CHECK-030: NYC LL144 scope check must record whether the tool substantially assists or replaces discretion.
- BIAS-CHECK-031: NYC LL144 scope check must record hiring or promotion decision type.
- BIAS-CHECK-032: NYC LL144 scope check must require independent audit when in scope.
- BIAS-CHECK-033: NYC LL144 scope check must require audit recency.
- BIAS-CHECK-034: NYC LL144 scope check must require public-summary URL where required.
- BIAS-CHECK-035: NYC LL144 scope check must require notice where required.
- BIAS-CHECK-036: NYC LL144 scope check must require alternative selection process or accommodation route where required.
- BIAS-CHECK-037: NYC LL144 scope check must block materially changed model version after audit.
- BIAS-CHECK-038: Colorado AI route must classify developer, deployer, or both.
- BIAS-CHECK-039: Colorado AI route must classify high-risk status.
- BIAS-CHECK-040: Colorado AI route must classify consequential decision domain.
- BIAS-CHECK-041: Colorado AI route must classify employment domain.
- BIAS-CHECK-042: Colorado AI route must classify education domain.
- BIAS-CHECK-043: Colorado AI route must classify financial or lending domain.
- BIAS-CHECK-044: Colorado AI route must classify essential government services domain.
- BIAS-CHECK-045: Colorado AI route must classify health care domain.
- BIAS-CHECK-046: Colorado AI route must classify housing domain.
- BIAS-CHECK-047: Colorado AI route must classify insurance domain.
- BIAS-CHECK-048: Colorado AI route must classify legal services domain.
- BIAS-CHECK-049: Colorado AI route must require impact assessment where deployer duties apply.
- BIAS-CHECK-050: Colorado AI route must require reasonable-care evidence.
- BIAS-CHECK-051: Colorado AI route must require consumer notice where required.
- BIAS-CHECK-052: Colorado AI route must require human review route where required.
- BIAS-CHECK-053: Colorado AI route must require material-modification reassessment.
- BIAS-CHECK-054: Colorado AI route must require algorithmic-discrimination risk log.
- BIAS-CHECK-055: California AB-331 route must record advisory status.
- BIAS-CHECK-056: California AB-331 route must block binding compliance claim unless law refresh changes status.
- BIAS-CHECK-057: California AB-331 route must preserve bill-status source.
- BIAS-CHECK-058: California AB-331 route must preserve automated-decision-tool taxonomy for future readiness.
- BIAS-CHECK-059: FCRA employment screening route must run when consumer report supports employment decision.
- BIAS-CHECK-060: ECOA route must run when credit decision or credit eligibility is affected.
- BIAS-CHECK-061: FERPA route must run when education records support education decision.
- BIAS-CHECK-062: HIPAA route must run when PHI supports health decision.
- BIAS-CHECK-063: State privacy profiling route must run when consumer profiling opt-out is triggered.
- BIAS-CHECK-064: ITAR route must run when defense technical data supports model training or decision support.
- BIAS-CHECK-065: SOC 2 evidence must not be represented as civil-rights compliance.
- BIAS-CHECK-066: NIST 800-53 evidence must not be represented as civil-rights compliance.
- BIAS-CHECK-067: FedRAMP evidence must not be represented as civil-rights compliance.
- BIAS-CHECK-068: Model version must be bound to every automated decision.
- BIAS-CHECK-069: Training-data lineage must be bound to every model readiness claim.
- BIAS-CHECK-070: Evaluation-data lineage must be bound to every model readiness claim.
- BIAS-CHECK-071: Drift monitoring must be bound to every ongoing readiness claim.
- BIAS-CHECK-072: Vendor attestation must be bound to every third-party AI readiness claim.
- BIAS-CHECK-073: Human override must record final decision maker.
- BIAS-CHECK-074: Human override must record reason.
- BIAS-CHECK-075: Decision explanation must record domain-specific authority.
- BIAS-CHECK-076: Candidate or consumer notice must record delivery evidence.
- BIAS-CHECK-077: Regulator export must redact protected-class audit-only data where disclosure is not authorized.
- BIAS-CHECK-078: Regulator export must preserve audit ids.
- BIAS-CHECK-079: Protected-class data must not leak into model features without reviewed authority.
- BIAS-CHECK-080: Accessibility review must include keyboard, screen-reader, timing, language, and alternative-format risks where product surface is involved.
- BIAS-CHECK-081: Bias review must include model, threshold, feature, data, and process risks.
- BIAS-CHECK-082: Bias review must include human decision maker behavior where automation substantially assists but does not replace decision making.
- BIAS-CHECK-083: Bias review must include monitoring after deployment.
- BIAS-CHECK-084: Bias review must include remediation owner.
- BIAS-CHECK-085: Bias review must include retest schedule.
- BIAS-CHECK-086: Future Cedar fragments must cite this bias checkpoint.
- BIAS-CHECK-087: Future schema migrations must cite this bias checkpoint.
- BIAS-CHECK-088: Future OpenAPI overlays must cite this bias checkpoint.
- BIAS-CHECK-089: Future audit registry entries must cite this bias checkpoint.
- BIAS-CHECK-090: Future tests must cover four-fifths warning and caveat.
- BIAS-CHECK-091: Future tests must cover NYC audit missing, stale, and model-changed denials.
- BIAS-CHECK-092: Future tests must cover Colorado high-risk AI impact-assessment denial.
- BIAS-CHECK-093: Future tests must cover AB-331 advisory non-binding status.
- BIAS-CHECK-094: Future tests must cover ADA accommodation route.
- BIAS-CHECK-095: Future tests must cover FCRA employment screening route.
- BIAS-CHECK-096: Future tests must cover ECOA credit AI route.
- BIAS-CHECK-097: Bias checkpoint name is `us-localization-pack-w1-2026-05-20`.
- BIAS-CHECK-098: Bias checkpoint evidence token is `us_pack_docs:6`.
- BIAS-CHECK-099: Bias checkpoint status is documentation-authored and implementation-pending.
- BIAS-CHECK-100: Bias checkpoint requires EEOC, NYC DCWP, Colorado, and California official source refresh before runtime enforcement.
- BIAS-CHECK-101: Bias checkpoint requires counsel review before binding civil-rights compliance claim.
- BIAS-CHECK-102: Bias checkpoint requires no ADR edits in this slice.
- BIAS-CHECK-103: Bias checkpoint requires no microservice edits in this slice.
- BIAS-CHECK-104: Bias checkpoint requires no other pack edits in this slice.
- BIAS-CHECK-105: Bias checkpoint stop condition is VCS promote or clean VCS blocker.
- BIAS-CHECK-106: Bias checkpoint final report must include line-count and VCS evidence.
