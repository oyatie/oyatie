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
  - https://privacy.ca.gov/california-privacy-rights/
  - https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=CIV&sectionNum=1798.135.
  - https://coag.gov/resources/colorado-privacy-act/
  - https://portal.ct.gov/ag/common/data-privacy
  - https://www.oag.state.va.us/consumer-protection/files/tips-and-info/Virginia-Consumer-Data-Protection-Act-Summary-2-2-23.pdf
  - https://attorneygeneral.utah.gov/data-privacy-2/
  - https://dir.texas.gov/technology-legislation/texas-data-privacy-and-security-act
  - https://www.legis.iowa.gov/docs/code/715D.pdf
  - https://iga.in.gov/laws/2026/ic/titles/024#24-15
  - https://www.tn.gov/attorneygeneral/news/2025/4/30/pr25-25.html
  - https://archive.legmt.gov/bills/mca/title_0300/chapter_0140/part_0280/sections_index.html
  - https://www.doj.state.or.us/consumer-protection/id-theft-data-breaches/
  - https://attorneygeneral.delaware.gov/fraud/personal-data-privacy-portal/frequently-asked-questions/
  - https://gc.nh.gov/rsa/html/LII/507-H/507-H-mrg.htm
---

# State Privacy Laws Comparison

This document compares California CCPA/CPRA and thirteen listed state privacy laws for US-PACK-1.
The comparison is implementation-facing, not marketing-facing.
The comparison treats each state law as a jurisdiction overlay.
The comparison does not flatten all state laws into one generic privacy posture.
The comparison records where opt-out, opt-in, child consent, sensitive-data, universal opt-out, appeal, and enforcement rules diverge.
The comparison is intentionally explicit because later Cedar policies need per-state predicates.
The comparison uses current official public authorities reviewed for this pack on 2026-05-20.
The comparison is not legal advice.
The comparison requires counsel review before regulated production use.
The pack default is to apply the most protective triggered state duty unless a narrower sector law or preemption clause controls.

## Authority Citations

- STATE-AUTH-001: California CCPA/CPRA authority is California Civil Code Title 1.81.5, including Sections 1798.100 through 1798.199.100.
- STATE-AUTH-002: California consumer-rights authority includes the California Privacy Protection Agency rights guidance.
- STATE-AUTH-003: California opt-out authority includes Civil Code Section 1798.120 for sale and sharing.
- STATE-AUTH-004: California sensitive-use limitation authority includes Civil Code Section 1798.121.
- STATE-AUTH-005: California opt-out link and preference-signal authority includes Civil Code Section 1798.135.
- STATE-AUTH-006: California sensitive personal information categories are anchored in Civil Code Section 1798.140.
- STATE-AUTH-007: Colorado authority is the Colorado Privacy Act, Colorado Revised Statutes Title 6, Article 1, Part 13.
- STATE-AUTH-008: Colorado AG guidance states that controllers must obtain affirmative consent before processing sensitive data.
- STATE-AUTH-009: Colorado opt-out authority covers sale, targeted advertising, and certain profiling.
- STATE-AUTH-010: Colorado universal opt-out authority is operational through AG-maintained recognized mechanisms.
- STATE-AUTH-011: Connecticut authority is the Connecticut Data Privacy Act, Conn. Gen. Stat. Section 42-515 et seq.
- STATE-AUTH-012: Connecticut AG enforcement reports and guidance supply operational enforcement priorities, including notices and sensitive-data processing.
- STATE-AUTH-013: Virginia authority is the Consumer Data Protection Act, Code of Virginia Title 59.1, Chapter 53.
- STATE-AUTH-014: Virginia OAG guidance confirms access, correction, deletion, portability, opt-out, appeal, and sensitive-data duties.
- STATE-AUTH-015: Utah authority is the Utah Consumer Privacy Act, Utah Code Title 13, Chapter 61.
- STATE-AUTH-016: Utah AG and Division of Consumer Protection guidance confirms opt-out rights for sale and targeted advertising.
- STATE-AUTH-017: Texas authority is the Texas Data Privacy and Security Act, Texas Business and Commerce Code Chapter 541.
- STATE-AUTH-018: Texas Department of Information Resources guidance states that the Texas Attorney General has exclusive enforcement authority.
- STATE-AUTH-019: Texas AG guidance states that small businesses generally exempt from the Act must still obtain consent before selling sensitive data.
- STATE-AUTH-020: Iowa authority is Iowa Code Chapter 715D, Consumer Data Protections.
- STATE-AUTH-021: Iowa Code Section 715D.4 requires sensitive-data notice and an opportunity to opt out for nonexempt processing.
- STATE-AUTH-022: Indiana authority is Indiana Code Title 24, Article 15, Consumer Data Protection.
- STATE-AUTH-023: Indiana authority becomes relevant for this pack as the 2026-effective state overlay.
- STATE-AUTH-024: Tennessee authority is the Tennessee Information Protection Act, Tennessee Code Annotated Title 47, Chapter 18, Part 33.
- STATE-AUTH-025: Tennessee AG guidance confirms opt-out from targeted advertising, profiling, and sale of personal information.
- STATE-AUTH-026: Tennessee AG guidance confirms consent before processing sensitive data.
- STATE-AUTH-027: Montana authority is Montana Code Annotated Title 30, Chapter 14, Part 28.
- STATE-AUTH-028: Montana authority covers opt-out rights, sensitive data, known-child data, and attorney-general complaint routing.
- STATE-AUTH-029: Oregon authority is the Oregon Consumer Privacy Act, ORS 646A.570 through 646A.589.
- STATE-AUTH-030: Oregon DOJ guidance confirms L.O.C.K.E.D. consumer rights, including list, opt-out, copy, know, edit, and delete.
- STATE-AUTH-031: Oregon DOJ guidance states that sensitive data requires consumer permission.
- STATE-AUTH-032: Delaware authority is the Delaware Personal Data Privacy Act, Delaware Code Title 6, Chapter 12D.
- STATE-AUTH-033: Delaware DOJ guidance confirms affirmative consent before collecting and processing sensitive data.
- STATE-AUTH-034: Delaware DOJ guidance confirms universal opt-out recognition beginning January 1, 2026.
- STATE-AUTH-035: New Jersey authority is the New Jersey Data Privacy Act, P.L. 2023, c.266, codified in the consumer-fraud title.
- STATE-AUTH-036: New Jersey overlay is included because it is one of the user-requested state privacy jurisdictions.
- STATE-AUTH-037: New Hampshire authority is RSA Chapter 507-H, Expectation of Privacy.
- STATE-AUTH-038: New Hampshire RSA 507-H requires consent before processing sensitive data.
- STATE-AUTH-039: New Hampshire RSA 507-H treats violations as unfair or deceptive acts enforceable by the attorney general.
- STATE-AUTH-040: Every state overlay in this document must be refreshed before production use because state privacy rulemaking and cure periods change frequently.
- STATE-AUTH-041: Federal sector exemptions do not erase state routing; they create residual-state-duty checks.
- STATE-AUTH-042: HIPAA-exempt PHI must still be labeled so the state overlay can explain why generic privacy export did not run.
- STATE-AUTH-043: GLBA-exempt NPI must still be labeled so state privacy APIs can route to financial privacy workflows.
- STATE-AUTH-044: FCRA-regulated consumer-report data must still be labeled so adverse-action and dispute workflows are preserved.
- STATE-AUTH-045: COPPA known-child treatment is referenced by most listed state laws and must remain a shared child-consent primitive.

## Comparison Method

- METHOD-001: `opt_out_sale` means the consumer may refuse sale of personal data or personal information.
- METHOD-002: `opt_out_share` means California-style cross-context behavioral advertising sharing where California uses sharing language.
- METHOD-003: `opt_out_targeted_ads` means the consumer may refuse targeted advertising processing.
- METHOD-004: `opt_out_profiling` means the consumer may refuse profiling for covered significant effects.
- METHOD-005: `opt_in_sensitive` means consent is required before sensitive-data processing.
- METHOD-006: `notice_plus_sensitive_opt_out` means sensitive data can be processed only after notice and an opt-out opportunity.
- METHOD-007: `limit_sensitive_use` means California-style right to limit use and disclosure rather than a general affirmative-consent rule.
- METHOD-008: `known_child_coppa` means under-13 data is handled through COPPA-consistent parental consent.
- METHOD-009: `teen_sale_share_opt_in` means California-style sale/share authorization for consumers under 16.
- METHOD-010: `universal_opt_out` means controllers must honor a qualifying browser, device, or preference signal.
- METHOD-011: `appeal_required` means consumer rights denials need an internal appeal route.
- METHOD-012: `private_right_limited` means private suits exist only in narrow circumstances, usually security incidents.
- METHOD-013: `ag_only` means enforcement is exclusive or functionally centralized in the attorney general or designated state authority.
- METHOD-014: `cure_window` means the pack must record whether notice-and-cure is active, expired, discretionary, or unavailable.
- METHOD-015: `sensitive_category_map` records law-specific sensitive categories rather than assuming the California list applies everywhere.

## Activated Cedar Policies

- STATE-CEDAR-001: `us.state_privacy.route_by_resident_state`.
- STATE-CEDAR-002: `us.state_privacy.block_unclassified_residency`.
- STATE-CEDAR-003: `us.state_privacy.enforce_consumer_rights_intake`.
- STATE-CEDAR-004: `us.state_privacy.enforce_access_request_scope`.
- STATE-CEDAR-005: `us.state_privacy.enforce_correction_request_scope`.
- STATE-CEDAR-006: `us.state_privacy.enforce_deletion_request_scope`.
- STATE-CEDAR-007: `us.state_privacy.enforce_portability_request_scope`.
- STATE-CEDAR-008: `us.state_privacy.enforce_opt_out_sale`.
- STATE-CEDAR-009: `us.state_privacy.enforce_opt_out_sharing`.
- STATE-CEDAR-010: `us.state_privacy.enforce_opt_out_targeted_ads`.
- STATE-CEDAR-011: `us.state_privacy.enforce_opt_out_profiling`.
- STATE-CEDAR-012: `us.state_privacy.enforce_sensitive_consent`.
- STATE-CEDAR-013: `us.state_privacy.enforce_sensitive_notice_plus_opt_out`.
- STATE-CEDAR-014: `us.state_privacy.enforce_california_sensitive_limit`.
- STATE-CEDAR-015: `us.state_privacy.enforce_known_child_coppa`.
- STATE-CEDAR-016: `us.state_privacy.enforce_teen_sale_share_opt_in`.
- STATE-CEDAR-017: `us.state_privacy.enforce_authorized_agent`.
- STATE-CEDAR-018: `us.state_privacy.enforce_universal_opt_out_signal`.
- STATE-CEDAR-019: `us.state_privacy.enforce_rights_appeal`.
- STATE-CEDAR-020: `us.state_privacy.enforce_controller_processor_contract`.
- STATE-CEDAR-021: `us.state_privacy.enforce_data_minimization`.
- STATE-CEDAR-022: `us.state_privacy.enforce_purpose_limitation`.
- STATE-CEDAR-023: `us.state_privacy.enforce_privacy_notice_freshness`.
- STATE-CEDAR-024: `us.state_privacy.enforce_non_discrimination_for_rights`.
- STATE-CEDAR-025: `us.state_privacy.enforce_dark_pattern_refusal`.
- STATE-CEDAR-026: `us.state_privacy.require_dpia_for_high_risk_processing`.
- STATE-CEDAR-027: `us.state_privacy.require_third_party_disclosure_list`.
- STATE-CEDAR-028: `us.state_privacy.require_exemption_evidence`.
- STATE-CEDAR-029: `us.state_privacy.require_regulator_export_review`.
- STATE-CEDAR-030: `us.state_privacy.require_state_cure_window_status`.
- STATE-CEDAR-031: `us.state_privacy.require_residual_sector_law_route`.
- STATE-CEDAR-032: `us.state_privacy.require_precedence_explanation`.
- STATE-CEDAR-033: `us.state_privacy.block_conflicting_processor_instruction`.
- STATE-CEDAR-034: `us.state_privacy.block_incompatible_secondary_use`.
- STATE-CEDAR-035: `us.state_privacy.block_sensitive_sale_without_consent`.

## Data Model Deltas

- STATE-DATA-001: Add `resident_state_code` with ISO-style two-letter U.S. state values.
- STATE-DATA-002: Add `privacy_law_overlay_ids` as an ordered list of triggered state laws.
- STATE-DATA-003: Add `state_privacy_controller_id` for controller accountability.
- STATE-DATA-004: Add `state_privacy_processor_id` for processor accountability.
- STATE-DATA-005: Add `state_privacy_contract_id` for controller-processor contract evidence.
- STATE-DATA-006: Add `consumer_request_id` for state consumer-rights requests.
- STATE-DATA-007: Add `consumer_request_type` with access, correction, deletion, portability, opt-out, appeal, and authorized-agent values.
- STATE-DATA-008: Add `consumer_request_state_deadline_at` for jurisdiction-specific timing.
- STATE-DATA-009: Add `consumer_request_denial_reason`.
- STATE-DATA-010: Add `consumer_request_appeal_id`.
- STATE-DATA-011: Add `authorized_agent_id`.
- STATE-DATA-012: Add `authorized_agent_proof_type`.
- STATE-DATA-013: Add `sale_opt_out_state`.
- STATE-DATA-014: Add `share_opt_out_state`.
- STATE-DATA-015: Add `targeted_ads_opt_out_state`.
- STATE-DATA-016: Add `profiling_opt_out_state`.
- STATE-DATA-017: Add `universal_opt_out_signal_seen_at`.
- STATE-DATA-018: Add `universal_opt_out_signal_source`.
- STATE-DATA-019: Add `universal_opt_out_signal_validated`.
- STATE-DATA-020: Add `sensitive_data_state_category`.
- STATE-DATA-021: Add `sensitive_processing_authority`.
- STATE-DATA-022: Add `sensitive_consent_id`.
- STATE-DATA-023: Add `sensitive_notice_id`.
- STATE-DATA-024: Add `sensitive_opt_out_state`.
- STATE-DATA-025: Add `california_limit_sensitive_use_state`.
- STATE-DATA-026: Add `known_child_status`.
- STATE-DATA-027: Add `teen_sale_share_authorization_state`.
- STATE-DATA-028: Add `parent_guardian_consent_id`.
- STATE-DATA-029: Add `privacy_notice_version_id`.
- STATE-DATA-030: Add `privacy_notice_jurisdiction_matrix_hash`.
- STATE-DATA-031: Add `third_party_recipient_list_available`.
- STATE-DATA-032: Add `third_party_recipient_list_request_id`.
- STATE-DATA-033: Add `data_protection_assessment_id`.
- STATE-DATA-034: Add `high_risk_processing_purpose`.
- STATE-DATA-035: Add `state_exemption_reason`.
- STATE-DATA-036: Add `sector_exemption_reason`.
- STATE-DATA-037: Add `state_cure_window_status`.
- STATE-DATA-038: Add `regulator_complaint_route`.
- STATE-DATA-039: Add `preemption_or_exemption_explanation`.
- STATE-DATA-040: Add `residual_state_obligation_ids`.

## API Contract Deltas

- STATE-API-001: Add `resident_state` to regulated privacy request bodies.
- STATE-API-002: Add `jurisdiction_overlays` to privacy response bodies.
- STATE-API-003: Add `POST /privacy/us/state/requests` for consumer-rights intake.
- STATE-API-004: Add `GET /privacy/us/state/requests/{id}` for request status.
- STATE-API-005: Add `POST /privacy/us/state/requests/{id}/appeal` for appeal submission.
- STATE-API-006: Add `POST /privacy/us/state/authorized-agents` for agency proof registration.
- STATE-API-007: Add `POST /privacy/us/state/opt-outs/sale` for sale opt-out.
- STATE-API-008: Add `POST /privacy/us/state/opt-outs/share` for California sharing opt-out.
- STATE-API-009: Add `POST /privacy/us/state/opt-outs/targeted-advertising` for targeted-ad opt-out.
- STATE-API-010: Add `POST /privacy/us/state/opt-outs/profiling` for profiling opt-out.
- STATE-API-011: Add `POST /privacy/us/state/universal-opt-out-signals` for signal ingestion.
- STATE-API-012: Add `POST /privacy/us/state/sensitive-consents` for consent-first states.
- STATE-API-013: Add `POST /privacy/us/state/sensitive-opt-outs` for notice-plus-opt-out states.
- STATE-API-014: Add `POST /privacy/us/state/california/limit-sensitive-use` for CPRA limit requests.
- STATE-API-015: Add `POST /privacy/us/state/children/parent-consents` for known-child consent evidence.
- STATE-API-016: Add `POST /privacy/us/state/teens/sale-share-authorizations` for California under-16 authorization.
- STATE-API-017: Add `GET /privacy/us/state/notices/{version}` for jurisdiction-specific notices.
- STATE-API-018: Add `GET /privacy/us/state/third-party-recipients` for states requiring recipient-list visibility.
- STATE-API-019: Add `POST /privacy/us/state/data-protection-assessments` for high-risk processing evidence.
- STATE-API-020: Add `GET /privacy/us/state/exemptions/{object_id}` for exemption explanation.
- STATE-API-021: Add `GET /privacy/us/state/regulator-routes/{state}` for complaint route metadata.
- STATE-API-022: Add `jurisdiction_deadline_at` to every accepted consumer-rights response.
- STATE-API-023: Add `appeal_deadline_at` to every denial response where appeal is available.
- STATE-API-024: Add `opt_out_effective_at` to opt-out acknowledgements.
- STATE-API-025: Add `conflict_resolution_explanation` when sector law changes the state privacy route.
- STATE-API-026: Add `state_cure_window_status` to regulator-facing exports.
- STATE-API-027: Add `evidence_event_ids` to all privacy mutation responses.
- STATE-API-028: Add `policy_fragment_ids` to all enforcement responses.
- STATE-API-029: Add `sensitive_category_basis` to sensitive-processing authorization responses.
- STATE-API-030: Add `universal_opt_out_signal_basis` to signal ingestion responses.

## Audit Event Additions

- STATE-AUDIT-001: `us.state_privacy.overlay_resolved`.
- STATE-AUDIT-002: `us.state_privacy.request_received`.
- STATE-AUDIT-003: `us.state_privacy.request_verified`.
- STATE-AUDIT-004: `us.state_privacy.request_denied`.
- STATE-AUDIT-005: `us.state_privacy.request_completed`.
- STATE-AUDIT-006: `us.state_privacy.appeal_opened`.
- STATE-AUDIT-007: `us.state_privacy.appeal_resolved`.
- STATE-AUDIT-008: `us.state_privacy.authorized_agent_registered`.
- STATE-AUDIT-009: `us.state_privacy.authorized_agent_rejected`.
- STATE-AUDIT-010: `us.state_privacy.sale_opt_out_applied`.
- STATE-AUDIT-011: `us.state_privacy.share_opt_out_applied`.
- STATE-AUDIT-012: `us.state_privacy.targeted_ads_opt_out_applied`.
- STATE-AUDIT-013: `us.state_privacy.profiling_opt_out_applied`.
- STATE-AUDIT-014: `us.state_privacy.universal_opt_out_signal_received`.
- STATE-AUDIT-015: `us.state_privacy.universal_opt_out_signal_rejected`.
- STATE-AUDIT-016: `us.state_privacy.sensitive_consent_recorded`.
- STATE-AUDIT-017: `us.state_privacy.sensitive_opt_out_recorded`.
- STATE-AUDIT-018: `us.state_privacy.california_limit_sensitive_use_recorded`.
- STATE-AUDIT-019: `us.state_privacy.known_child_parent_consent_recorded`.
- STATE-AUDIT-020: `us.state_privacy.teen_sale_share_authorization_recorded`.
- STATE-AUDIT-021: `us.state_privacy.privacy_notice_presented`.
- STATE-AUDIT-022: `us.state_privacy.privacy_notice_changed`.
- STATE-AUDIT-023: `us.state_privacy.processor_instruction_rejected`.
- STATE-AUDIT-024: `us.state_privacy.dpia_required`.
- STATE-AUDIT-025: `us.state_privacy.dpia_completed`.
- STATE-AUDIT-026: `us.state_privacy.third_party_recipient_list_exported`.
- STATE-AUDIT-027: `us.state_privacy.exemption_applied`.
- STATE-AUDIT-028: `us.state_privacy.residual_duty_recorded`.
- STATE-AUDIT-029: `us.state_privacy.regulator_export_generated`.
- STATE-AUDIT-030: `us.state_privacy.cure_window_status_recorded`.

## Failure Modes

- STATE-FAIL-001: The tenant treats all U.S. states as one privacy regime and misses a state-specific sensitive-data rule.
- STATE-FAIL-002: The tenant uses California CCPA/CPRA semantics for states that require affirmative sensitive-data consent.
- STATE-FAIL-003: The tenant applies affirmative consent to Iowa-style sensitive processing where the law uses notice plus opt-out, creating incorrect product behavior.
- STATE-FAIL-004: The tenant forgets that California sale/share for minors under 16 requires opt-in authorization.
- STATE-FAIL-005: The tenant records parental consent without verifying whether the consumer is a known child under COPPA.
- STATE-FAIL-006: The tenant receives a universal opt-out signal but applies it only to sale and not to targeted advertising where the state requires both.
- STATE-FAIL-007: The tenant receives a GPC-like signal but cannot prove the source, timestamp, or scope.
- STATE-FAIL-008: The tenant denies a consumer request without preserving appeal instructions.
- STATE-FAIL-009: The tenant processes sensitive data under a stale consent after purpose drift.
- STATE-FAIL-010: The tenant uses a dark-pattern flow that delays or burdens opt-out.
- STATE-FAIL-011: The tenant relies on a sector-law exemption without recording the residual state obligations.
- STATE-FAIL-012: The tenant deletes data that must be retained for audit, fraud, legal-hold, or security purposes.
- STATE-FAIL-013: The tenant refuses deletion but does not explain the retention authority.
- STATE-FAIL-014: The tenant shares data with a processor lacking a controller-processor contract.
- STATE-FAIL-015: The processor follows a controller instruction that violates a state privacy duty.
- STATE-FAIL-016: The tenant fails to update privacy notices when state overlay coverage changes.
- STATE-FAIL-017: The tenant processes teen data for targeted advertising without checking state-specific teen restrictions.
- STATE-FAIL-018: The tenant misclassifies precise geolocation as ordinary data where the state defines it as sensitive.
- STATE-FAIL-019: The tenant exports a third-party recipient list that includes privileged or security-sensitive details without review.
- STATE-FAIL-020: The tenant cannot show which state law governed a denied request.
- STATE-FAIL-021: The tenant assumes a cure window exists after it has expired or become discretionary.
- STATE-FAIL-022: The tenant treats employee or B2B data as in-scope where the state law excludes it, or excludes it where the state law covers it.
- STATE-FAIL-023: The tenant handles household data as individual-only data in California.
- STATE-FAIL-024: The tenant cannot distinguish sale, sharing, targeted advertising, profiling, and disclosure.
- STATE-FAIL-025: The tenant lets advertising tags run before opt-out state is loaded.
- STATE-FAIL-026: The tenant fails to propagate opt-out to downstream processors.
- STATE-FAIL-027: The tenant accepts authorized-agent requests without proof.
- STATE-FAIL-028: The tenant rejects authorized-agent requests categorically where state law allows them.
- STATE-FAIL-029: The tenant cannot map a consumer request to all service surfaces holding the data.
- STATE-FAIL-030: The tenant claims compliance without regulator-ready audit events.

## Per-State Comparison Matrix

### California CCPA/CPRA

- CA-001: Primary authority is California Civil Code Sections 1798.100 through 1798.199.100.
- CA-002: Scope uses California residents and covered businesses meeting statutory thresholds.
- CA-003: Consumer rights include know, access, delete, correct, opt-out of sale/share, limit sensitive use, portability, and non-discrimination.
- CA-004: Sale is opt-out by default for adults.
- CA-005: Sharing for cross-context behavioral advertising is opt-out by default for adults.
- CA-006: Sensitive personal information is handled through a right to limit use and disclosure rather than a universal opt-in rule.
- CA-007: Sale/share of personal information of consumers under 16 requires opt-in authorization.
- CA-008: Under 13 authorization is provided by parent or guardian.
- CA-009: Ages 13 through 15 may authorize sale/share themselves under the CCPA/CPRA structure.
- CA-010: Sensitive categories include government identifiers, account credentials, precise geolocation, racial or ethnic origin, religious or philosophical beliefs, union membership, mail/email/text contents, genetic data, biometric data, health, sex life, sexual orientation, and neural data where current law includes it.
- CA-011: Enforcement includes the California Privacy Protection Agency and California Attorney General.
- CA-012: Private right is limited and primarily tied to certain security incidents.
- CA-013: Universal opt-out and preference signal handling must be supported where applicable.
- CA-014: Pack policy uses `limit_sensitive_use` rather than `opt_in_sensitive`.
- CA-015: Pack policy uses distinct `opt_out_sale` and `opt_out_share` flags.
- CA-016: Pack API must expose California-specific limit-sensitive-use routes.
- CA-017: Pack data model must preserve household linkage where a request concerns household personal information.
- CA-018: Pack audit must record whether a signal satisfied California preference-signal requirements.
- CA-019: Pack failure handling must reject dark-pattern opt-out flows.
- CA-020: Pack precedence gives sector-law exemptions evidence but keeps residual notice and audit explanation.

### Colorado Privacy Act

- CO-001: Primary authority is Colorado Revised Statutes Title 6, Article 1, Part 13.
- CO-002: Consumer rights include access, correction, deletion, portability, opt-out, and appeal.
- CO-003: Sale is opt-out.
- CO-004: Targeted advertising is opt-out.
- CO-005: Profiling with legal or similarly significant effects is opt-out.
- CO-006: Sensitive-data processing requires affirmative consent.
- CO-007: Known-child data requires parent or lawful guardian consent under COPPA-consistent handling.
- CO-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health condition or diagnosis, sex life, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child personal data, and precise geolocation.
- CO-009: Universal opt-out mechanisms are recognized through Colorado AG processes.
- CO-010: Enforcement is by the Colorado Attorney General and district attorneys.
- CO-011: No general private right of action is modeled by this pack.
- CO-012: Data protection assessments are required for high-risk processing.
- CO-013: Pack policy must treat Colorado sensitive processing as `opt_in_sensitive`.
- CO-014: Pack policy must honor recognized universal opt-out signals.
- CO-015: Pack policy must emit appeal events for denied requests.
- CO-016: Pack API must include opt-out routes for sale, targeted advertising, and profiling.
- CO-017: Pack data model must retain DPIA evidence for high-risk processing.
- CO-018: Pack audit must record the source of universal opt-out signal recognition.
- CO-019: Pack failure handling must catch precision geolocation downgrades.
- CO-020: Pack precedence must stack Colorado AI obligations separately from Colorado privacy obligations.

### Connecticut Data Privacy Act

- CT-001: Primary authority is Conn. Gen. Stat. Section 42-515 et seq.
- CT-002: Consumer rights include access, correction, deletion, portability, opt-out, and appeal.
- CT-003: Sale is opt-out.
- CT-004: Targeted advertising is opt-out.
- CT-005: Profiling with legal or similarly significant effects is opt-out.
- CT-006: Sensitive-data processing requires consent.
- CT-007: Known-child processing follows COPPA-consistent parental consent.
- CT-008: Sensitive categories include race, ethnicity, religion, health condition, sex life, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- CT-009: Connecticut has universal opt-out expectations that must be represented as signal handling where applicable.
- CT-010: Enforcement is by the Connecticut Attorney General.
- CT-011: The pack records enforcement report posture because AG reports identify operational focus areas.
- CT-012: No general private right of action is modeled by this pack.
- CT-013: Pack policy must treat Connecticut sensitive processing as `opt_in_sensitive`.
- CT-014: Pack policy must route teen or child amendments through counsel-reviewed state metadata before production.
- CT-015: Pack API must include appeal instructions after request denial.
- CT-016: Pack data model must retain `regulator_complaint_route`.
- CT-017: Pack audit must capture privacy notice presentation.
- CT-018: Pack failure handling must catch stale notices and sensitive-data misclassification.
- CT-019: Pack precedence must preserve HIPAA, GLBA, FERPA, COPPA, and FCRA exemptions.
- CT-020: Pack refresh must re-check CTDPA amendments before production.

### Virginia Consumer Data Protection Act

- VA-001: Primary authority is Code of Virginia Title 59.1, Chapter 53.
- VA-002: Consumer rights include confirmation, access, correction, deletion, portability, opt-out, and appeal.
- VA-003: Sale is opt-out.
- VA-004: Targeted advertising is opt-out.
- VA-005: Profiling with legal or similarly significant effects is opt-out.
- VA-006: Sensitive-data processing requires consent.
- VA-007: Known-child data is routed to COPPA-consistent consent.
- VA-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health diagnosis, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- VA-009: Enforcement is by the Virginia Attorney General.
- VA-010: No private right of action is modeled by this pack.
- VA-011: Cure mechanics must be recorded for regulator-ready evidence.
- VA-012: Pack policy must require appeal metadata after denial.
- VA-013: Pack policy must reject processing beyond disclosed purposes unless authority exists.
- VA-014: Pack API must expose deletion and portability routes.
- VA-015: Pack data model must distinguish pseudonymous data and identifiable data.
- VA-016: Pack audit must record opt-out propagation to processors.
- VA-017: Pack failure handling must catch targeted-advertising tags firing after opt-out.
- VA-018: Pack precedence must preserve FCRA/ECOA rules for credit workflows.
- VA-019: Pack precedence must preserve HIPAA rules for PHI workflows.
- VA-020: Pack refresh must check current AG guidance before production.

### Utah Consumer Privacy Act

- UT-001: Primary authority is Utah Code Title 13, Chapter 61.
- UT-002: Consumer rights include access, deletion, portability, and opt-out for sale and targeted advertising.
- UT-003: Utah does not create the same correction right as several other state laws in the pack baseline.
- UT-004: Sale is opt-out.
- UT-005: Targeted advertising is opt-out.
- UT-006: Utah does not use the same broad opt-out profiling route as Colorado, Connecticut, and Virginia in this pack baseline.
- UT-007: Sensitive-data processing uses notice and an opportunity to opt out.
- UT-008: Known-child processing follows COPPA-consistent treatment.
- UT-009: Sensitive categories include racial or ethnic origin, religious beliefs, sexual orientation, citizenship or immigration status, medical history, mental or physical health condition or treatment, genetic or biometric data for identification, geolocation, and known-child personal data.
- UT-010: Enforcement is routed through Utah consumer protection processes and the Utah Attorney General.
- UT-011: No private right of action is modeled by this pack.
- UT-012: Pack policy must use `notice_plus_sensitive_opt_out`, not `opt_in_sensitive`.
- UT-013: Pack API must provide sensitive-data notices before nonexempt processing.
- UT-014: Pack audit must record the date and content of sensitive notice.
- UT-015: Pack failure handling must catch incorrect affirmative-consent UX that contradicts tenant-configured Utah posture.
- UT-016: Pack failure handling must also catch missing opt-out after notice.
- UT-017: Pack precedence must preserve GLBA exemptions for financial institutions.
- UT-018: Pack precedence must preserve HIPAA exemptions for covered health data.
- UT-019: Pack refresh must check Utah DCP and AG guidance before production.
- UT-020: Pack product text must avoid implying Utah consumers have every right available in California.

### Texas Data Privacy and Security Act

- TX-001: Primary authority is Texas Business and Commerce Code Chapter 541.
- TX-002: Consumer rights include access, correction, deletion, portability, opt-out, and appeal.
- TX-003: Sale is opt-out.
- TX-004: Targeted advertising is opt-out.
- TX-005: Profiling in furtherance of decisions producing legal or similarly significant effects is opt-out.
- TX-006: Sensitive-data processing requires consent.
- TX-007: Small businesses generally exempt from the Act must still obtain consent before selling sensitive data.
- TX-008: Known-child data follows COPPA-consistent treatment.
- TX-009: Sensitive categories include racial or ethnic origin, religious beliefs, health diagnosis, sexuality, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- TX-010: Enforcement is exclusively by the Texas Attorney General.
- TX-011: No private right of action is modeled by this pack.
- TX-012: Pack policy must expose small-business-sensitive-sale gating.
- TX-013: Pack policy must require data-protection assessments for covered high-risk processing.
- TX-014: Pack API must return Texas AG complaint route metadata after denied appeals.
- TX-015: Pack data model must retain `small_business_sensitive_sale_consent_required`.
- TX-016: Pack audit must record Chapter 541 cure status when relevant.
- TX-017: Pack failure handling must catch sale of sensitive data by exempt small businesses without consent.
- TX-018: Pack precedence must preserve HIPAA, GLBA, FCRA, FERPA, and COPPA exemptions.
- TX-019: Pack refresh must check the Texas DIR and AG pages before production.
- TX-020: Pack regulator export must identify Chapter 541 sections by control family.

### Iowa Consumer Data Protections

- IA-001: Primary authority is Iowa Code Chapter 715D.
- IA-002: Consumer rights include access, deletion, portability, and opt-out for sale.
- IA-003: Iowa correction rights are not modeled as broad baseline rights in this pack.
- IA-004: Sale is opt-out.
- IA-005: Iowa does not use the same broad targeted-advertising opt-out as several other state laws in this pack baseline.
- IA-006: Sensitive-data processing requires clear notice and an opportunity to opt out for nonexempt processing.
- IA-007: Known-child processing follows COPPA-consistent treatment.
- IA-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health diagnosis, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- IA-009: Enforcement is by the Iowa Attorney General.
- IA-010: No private right of action is modeled by this pack.
- IA-011: Pack policy must use `notice_plus_sensitive_opt_out`, not `opt_in_sensitive`.
- IA-012: Pack API must provide a state-specific sensitive-processing notice.
- IA-013: Pack data model must record nonexempt purpose.
- IA-014: Pack audit must preserve notice delivery and opt-out opportunity evidence.
- IA-015: Pack failure handling must catch treating Iowa as Colorado-like consent-first.
- IA-016: Pack failure handling must catch treating Iowa as California-like sensitive-use limitation only.
- IA-017: Pack precedence must preserve sector exemptions.
- IA-018: Pack refresh must check current Iowa Code edition before production.
- IA-019: Pack product text must avoid listing correction as a universal Iowa right.
- IA-020: Pack regulator export must be restricted to Iowa Chapter 715D evidence.

### Indiana Consumer Data Protection

- IN-001: Primary authority is Indiana Code Title 24, Article 15.
- IN-002: Effective-date handling is mandatory because Indiana is a 2026 overlay in this pack.
- IN-003: Consumer rights include confirmation, access, correction, deletion, portability, opt-out, and appeal.
- IN-004: Sale is opt-out.
- IN-005: Targeted advertising is opt-out.
- IN-006: Profiling with legal or similarly significant effects is opt-out.
- IN-007: Sensitive-data processing requires consent.
- IN-008: Known-child data follows COPPA-consistent treatment.
- IN-009: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health diagnosis, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- IN-010: Enforcement is by the Indiana Attorney General.
- IN-011: No private right of action is modeled by this pack.
- IN-012: Pack policy must include effective-date gates.
- IN-013: Pack API must return pre-effective advisory status if invoked before enforcement date in archival replay.
- IN-014: Pack data model must record `law_effective_at`.
- IN-015: Pack audit must record whether the request occurred before or after effective date.
- IN-016: Pack failure handling must catch premature binding enforcement in historical data.
- IN-017: Pack failure handling must catch failure to activate after effective date.
- IN-018: Pack precedence must preserve state-specific exemption evidence.
- IN-019: Pack refresh must check current Indiana Code before production.
- IN-020: Pack regulator export must separate pre-effective simulation from binding enforcement.

### Tennessee Information Protection Act

- TN-001: Primary authority is Tennessee Code Annotated Title 47, Chapter 18, Part 33.
- TN-002: Consumer rights include confirm/access, correction, deletion, portability, and opt-out.
- TN-003: Sale is opt-out.
- TN-004: Targeted advertising is opt-out.
- TN-005: Profiling is opt-out where covered by the Act.
- TN-006: Sensitive-data processing requires consent.
- TN-007: Known-child processing follows COPPA-consistent treatment.
- TN-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health diagnosis, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- TN-009: Enforcement is by the Tennessee Attorney General.
- TN-010: Tennessee includes a privacy-program affirmative-defense concept tied to recognized privacy frameworks.
- TN-011: No private right of action is modeled by this pack.
- TN-012: Pack policy must record privacy-program evidence separately from consumer-rights events.
- TN-013: Pack API must expose opt-out for targeted advertising, profiling, and sale.
- TN-014: Pack data model must retain `privacy_program_framework_basis`.
- TN-015: Pack audit must capture reasonable administrative, technical, and physical security evidence.
- TN-016: Pack failure handling must catch using the affirmative defense as permission to skip rights.
- TN-017: Pack failure handling must catch sensitive processing without consent.
- TN-018: Pack precedence must preserve sector exemptions.
- TN-019: Pack refresh must check Tennessee AG guidance before production.
- TN-020: Pack regulator export must include privacy-program evidence only when counsel approves.

### Montana Consumer Data Privacy Act

- MT-001: Primary authority is Montana Code Annotated Title 30, Chapter 14, Part 28.
- MT-002: Consumer rights include access, correction, deletion, portability, opt-out, and appeal.
- MT-003: Sale is opt-out.
- MT-004: Targeted advertising is opt-out.
- MT-005: Profiling with legal or similarly significant effects is opt-out.
- MT-006: Sensitive-data processing requires consent under the MCDPA model.
- MT-007: Known-child data follows COPPA-consistent treatment.
- MT-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health diagnosis, sex life, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- MT-009: Universal opt-out signal obligations must be represented where applicable.
- MT-010: Enforcement is by the Montana Attorney General.
- MT-011: No private right of action is modeled by this pack.
- MT-012: Cure-window state must be checked because statutory cure treatment can sunset or change.
- MT-013: Pack policy must treat Montana sensitive processing as consent-first.
- MT-014: Pack API must expose appeal and AG complaint route metadata.
- MT-015: Pack data model must retain universal opt-out signal status.
- MT-016: Pack audit must preserve data-protection assessment availability.
- MT-017: Pack failure handling must catch stale cure assumptions after April 2026.
- MT-018: Pack failure handling must catch missing universal opt-out handling.
- MT-019: Pack refresh must check MCA current text before production.
- MT-020: Pack regulator export must separate privacy-law events from other Montana data-broker duties.

### Oregon Consumer Privacy Act

- OR-001: Primary authority is ORS 646A.570 through 646A.589.
- OR-002: Consumer rights include know, confirm, access, correction, deletion, portability, opt-out, and recipient-list visibility.
- OR-003: Sale is opt-out.
- OR-004: Targeted advertising is opt-out.
- OR-005: Profiling is opt-out where covered by the Act.
- OR-006: Sensitive-data processing requires permission or consent.
- OR-007: Children under 13 are treated as sensitive-data subjects.
- OR-008: Sensitive categories include national origin, racial or ethnic background, religious beliefs, mental or physical condition or diagnosis, sexual orientation, gender identity, status as transgender or nonbinary, crime-victim status, citizenship or immigration status, precise geolocation, genetic or biometric data for identification, and child data.
- OR-009: Universal opt-out is recognized by Oregon beginning January 1, 2026.
- OR-010: Enforcement is by the Oregon Attorney General.
- OR-011: No private right of action is modeled by this pack.
- OR-012: Pack policy must expose third-party recipient-list exports.
- OR-013: Pack policy must treat Oregon sensitive data as permission-first.
- OR-014: Pack API must expose recipient-list request and universal opt-out signal endpoints.
- OR-015: Pack data model must preserve `third_party_recipient_list_available`.
- OR-016: Pack audit must record universal opt-out signal receipt.
- OR-017: Pack failure handling must catch omission of nonprofit coverage where applicable.
- OR-018: Pack failure handling must catch missing child-sensitive classification.
- OR-019: Pack refresh must check Oregon DOJ guidance before production.
- OR-020: Pack regulator export must include OCPA enforcement report references when used.

### Delaware Personal Data Privacy Act

- DE-001: Primary authority is Delaware Code Title 6, Chapter 12D.
- DE-002: Consumer rights include access, correction, deletion, portability, opt-out, and appeal.
- DE-003: Sale is opt-out.
- DE-004: Targeted advertising is opt-out.
- DE-005: Profiling with legal or similarly significant effects is opt-out.
- DE-006: Sensitive-data processing requires affirmative consent.
- DE-007: Known-child processing follows COPPA-consistent treatment.
- DE-008: Sensitive categories include racial or ethnic origin, religious beliefs, health information, sex life, sexual orientation, transgender or nonbinary status, citizenship or immigration status, genetic or biometric data for identification, precise geolocation, and child data.
- DE-009: Universal opt-out mechanisms must be recognized beginning January 1, 2026.
- DE-010: Enforcement is by the Delaware Department of Justice and Attorney General.
- DE-011: No private right of action is modeled by this pack.
- DE-012: Pack policy must treat Delaware sensitive processing as consent-first.
- DE-013: Pack API must include universal opt-out signal handling.
- DE-014: Pack data model must capture transgender or nonbinary sensitive category where Delaware authority lists it.
- DE-015: Pack audit must preserve consent before sensitive collection and processing.
- DE-016: Pack failure handling must catch collecting sensitive data before consent.
- DE-017: Pack failure handling must catch missing universal opt-out recognition after 2026-01-01.
- DE-018: Pack precedence must preserve GLBA, HIPAA, FCRA, and other data exemptions.
- DE-019: Pack refresh must check Delaware DOJ portal before production.
- DE-020: Pack regulator export must include DOJ complaint route.

### New Jersey Data Privacy Act

- NJ-001: Primary authority is P.L. 2023, c.266, the New Jersey consumer data privacy law.
- NJ-002: Consumer rights include confirmation/access, correction, deletion, portability, opt-out, and appeal.
- NJ-003: Sale is opt-out.
- NJ-004: Targeted advertising is opt-out.
- NJ-005: Profiling with legal or similarly significant effects is opt-out.
- NJ-006: Sensitive-data processing requires consent in the New Jersey model.
- NJ-007: Known-child data follows COPPA-consistent treatment.
- NJ-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health condition, treatment, or diagnosis, financial information, sex life, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, child data, and precise geolocation.
- NJ-009: Universal opt-out obligations must be represented where applicable.
- NJ-010: Enforcement is through New Jersey consumer protection authorities.
- NJ-011: No broad private right of action is modeled by this pack.
- NJ-012: Cure-window status must be checked because New Jersey cure mechanics are time-bound.
- NJ-013: Pack policy must treat New Jersey sensitive processing as consent-first.
- NJ-014: Pack API must expose appeal and opt-out routes.
- NJ-015: Pack data model must record financial-information sensitive category where New Jersey requires it.
- NJ-016: Pack audit must preserve universal opt-out recognition where applicable.
- NJ-017: Pack failure handling must catch omission of financial information from sensitive classification.
- NJ-018: Pack failure handling must catch stale cure-window assumptions.
- NJ-019: Pack refresh must check official New Jersey statutory text before production.
- NJ-020: Pack regulator export must identify the consumer-fraud enforcement route.

### New Hampshire Expectation of Privacy

- NH-001: Primary authority is RSA Chapter 507-H, Expectation of Privacy.
- NH-002: Consumer rights include confirmation, access, correction, deletion, portability, opt-out, and appeal.
- NH-003: Sale is opt-out.
- NH-004: Targeted advertising is opt-out.
- NH-005: Profiling with legal or similarly significant effects is opt-out.
- NH-006: Sensitive-data processing requires consent.
- NH-007: Known-child data follows COPPA-consistent treatment.
- NH-008: Sensitive categories include racial or ethnic origin, religious beliefs, mental or physical health condition or diagnosis, sex life, sexual orientation, citizenship or immigration status, genetic or biometric data for identification, known-child data, and precise geolocation.
- NH-009: Universal opt-out preference-signal handling is represented through RSA 507-H signal provisions.
- NH-010: Violations are treated as unfair or deceptive acts under RSA 358-A and enforced by the attorney general.
- NH-011: No private right beyond the state enforcement model is represented without counsel review.
- NH-012: Pack policy must treat New Hampshire sensitive processing as consent-first.
- NH-013: Pack API must include opt-out preference signal handling.
- NH-014: Pack data model must preserve signal conflict resolution for loyalty or controller-specific settings.
- NH-015: Pack audit must record when a signal overrides conflicting controller settings.
- NH-016: Pack failure handling must catch failure to honor opt-out preference signals.
- NH-017: Pack failure handling must catch missing consent for sensitive data.
- NH-018: Pack precedence must preserve sector exemptions.
- NH-019: Pack refresh must check the current RSA text before production.
- NH-020: Pack regulator export must identify RSA 358-A enforcement linkage.

## Worked Examples

- STATE-EXAMPLE-001: California adult opts out of sale and sharing; system sets `sale_opt_out_state=active` and `share_opt_out_state=active`.
- STATE-EXAMPLE-002: California consumer limits sensitive use; system blocks sensitive use outside allowed CPRA purposes.
- STATE-EXAMPLE-003: California 15-year-old attempts sale/share authorization; system records teen authorization and prevents default opt-out assumption.
- STATE-EXAMPLE-004: Colorado resident submits universal opt-out signal; system honors sale and targeted-advertising opt-out after signal validation.
- STATE-EXAMPLE-005: Colorado controller wants to process precise geolocation; system requires sensitive consent.
- STATE-EXAMPLE-006: Connecticut denial of deletion request; system returns appeal route and AG complaint metadata.
- STATE-EXAMPLE-007: Virginia consumer opts out of profiling; system blocks automated eligibility profiling with legal effect.
- STATE-EXAMPLE-008: Utah tenant processes health-related sensitive data; system presents notice and opportunity to opt out, not a Colorado-style consent prompt.
- STATE-EXAMPLE-009: Texas small business sells sensitive data; system requires consent despite small-business exemption.
- STATE-EXAMPLE-010: Iowa controller processes sensitive data for a nonexempt purpose; system records notice and opt-out opportunity.
- STATE-EXAMPLE-011: Indiana request arrives before 2026 effective date in replay; system marks simulation rather than binding enforcement.
- STATE-EXAMPLE-012: Tennessee tenant claims privacy-program defense; system stores framework evidence but still enforces opt-out and consent.
- STATE-EXAMPLE-013: Montana cure status is requested in May 2026; system requires current-law check before relying on cure.
- STATE-EXAMPLE-014: Oregon resident asks for recipient list; system exports a reviewed list of specific third-party recipients where available.
- STATE-EXAMPLE-015: Delaware sensitive data collection begins; system blocks until affirmative consent exists.
- STATE-EXAMPLE-016: New Jersey workflow classifies financial information; system marks it sensitive under New Jersey overlay.
- STATE-EXAMPLE-017: New Hampshire opt-out signal conflicts with a loyalty setting; system honors the signal and records conflict notice.
- STATE-EXAMPLE-018: HIPAA PHI is requested under a generic state access route; system redirects to HIPAA access workflow with state residual explanation.
- STATE-EXAMPLE-019: GLBA NPI is requested under California delete route; system records exemption and financial privacy route.
- STATE-EXAMPLE-020: Processor receives incompatible instruction; system rejects the instruction and emits processor-instruction audit event.
- STATE-EXAMPLE-021: Consumer requests deletion across multiple states due to relocation; system applies current resident-state route and records historical overlays.
- STATE-EXAMPLE-022: Advertising tag loads before opt-out state; system blocks the tag and records pre-enforcement denial.
- STATE-EXAMPLE-023: Authorized agent submits Delaware opt-out; system verifies agency proof before applying opt-out.
- STATE-EXAMPLE-024: Oregon child data is collected; system marks all personal data from that known child as sensitive.
- STATE-EXAMPLE-025: State privacy request touches legal-hold records; system denies deletion for retained records and explains authority.
- STATE-EXAMPLE-026: Universal opt-out signal lacks required metadata; system rejects the signal and prompts direct opt-out route.
- STATE-EXAMPLE-027: Consumer appeal succeeds; system completes original request and emits appeal-resolved event.
- STATE-EXAMPLE-028: Consumer appeal fails; system returns regulator complaint route where state law requires it.
- STATE-EXAMPLE-029: Purpose drift occurs after sensitive consent; system blocks secondary use until new authority exists.
- STATE-EXAMPLE-030: Tenant adds new processing purpose; system requires privacy-notice update and DPIA check.

## Cross-References

- STATE-XREF-001: `packs/us-localization/README.md` defines pack precedence and activated microservices.
- STATE-XREF-002: `packs/us-localization/federal-privacy-laws.md` defines sector-law routing before state privacy overlays.
- STATE-XREF-003: `packs/us-localization/hipaa-phi-handling.md` defines PHI-specific handling where state privacy exemptions apply.
- STATE-XREF-004: `packs/us-localization/sox-and-financial-reporting.md` defines financial reporting controls outside state consumer privacy.
- STATE-XREF-005: `packs/us-localization/discrimination-laws-and-ai-bias.md` defines AI and discrimination overlays that can combine with state privacy profiling.
- STATE-XREF-006: `specs/microservices/identity.json` is the likely home for resident-state verification and authorized-agent proof.
- STATE-XREF-007: `specs/microservices/governance.json` is the likely home for authority resolution and regulator export.
- STATE-XREF-008: `specs/microservices/audit-chain` registry entries are future hosts for state privacy events.
- STATE-XREF-009: `specs/microservices/anonymous.json` is a likely host for de-identification and anonymization workflows.
- STATE-XREF-010: `specs/microservices/intelligence.json` is a likely host for compliance evidence build workflows.
- STATE-XREF-011: `registry/catalog/check-data-class.yaml` is a future state sensitive-category gate.
- STATE-XREF-012: `registry/catalog/check-compliance-evidence-coverage.yaml` is a future state evidence completeness gate.
- STATE-XREF-013: `registry/catalog/check-cedar-fragment-coverage.yaml` is a future Cedar coverage gate.
- STATE-XREF-014: `registry/catalog/check-high-risk-auto-decision-refusal.yaml` connects state profiling opt-outs to AI decision guardrails.
- STATE-XREF-015: California official CPPA pages are the citation source for consumer-rights descriptions.
- STATE-XREF-016: Official state code pages or attorney-general portals must replace vendor summaries during implementation.
- STATE-XREF-017: Every state law in this file needs a `law_effective_at` and `law_refreshed_at` value before runtime enforcement.
- STATE-XREF-018: Every consumer-facing privacy notice generated from this file needs counsel review.
- STATE-XREF-019: Every processor contract generated from this file needs state-specific controller instruction language.
- STATE-XREF-020: The checkpoint for this document is `us-localization-pack-w1-2026-05-20`.
