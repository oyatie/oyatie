---
doc_class: LocalizationPack
pack_id: JP-PACK-1
doc_id: JP-PACK-1-CYBER
title: Cybersecurity Basic Act Incident Response Controls
version: 1.0.0
status: canonical-draft
date: 2026-05-20
related_oyatie_adrs:
  - ADR-0064
  - ADR-0244
  - ADR-0251
  - ADR-0263
citing_authority_url:
  - https://www.japaneselawtranslation.go.jp/en/laws/view/4755/je
  - https://www.nisc.go.jp/policy/group/infra/policy.html
  - https://www.meti.go.jp/english/policy/safety_security/cybersecurity/index.html
  - https://www.meti.go.jp/policy/netsecurity/downloadfiles/CSM_Guideline_v3.0_en.pdf
  - https://www.meti.go.jp/policy/netsecurity/incident.html
---

# Cybersecurity Basic Act Incident Response Controls

This document defines the cybersecurity incident-response layer of JP-PACK-1.
The statutory source is the Basic Act on Cybersecurity.
The national coordination source is NISC.
The industrial-sector policy source is METI.
The pack treats cybersecurity as a management, evidence, and coordination obligation.
The pack treats critical infrastructure designation as a tenant and service classification state.
The pack names the NISC critical infrastructure fields in policy.
The pack maps sector ministry ownership before incident response begins.
The pack requires CSIRT ownership for covered tenants.
The pack requires evidence preservation at incident open.
The pack requires cross-border attack classification.
The pack requires NISC information-contact routing when designated critical infrastructure is involved.
The pack requires METI routing when the tenant or service is in a METI-relevant industrial sector.
The pack defines internal notification timelines because the cited Basic Act is framework legislation.
The pack does not falsely claim a single universal Cybersecurity Basic Act statutory hour limit.
The pack sets an internal T+0 clock at incident awareness.
The pack requires severity classification within T+30 minutes.
The pack requires evidence preservation within T+1 hour.
The pack requires management notification within T+2 hours for high severity.
The pack requires sector-ministry routing decision within T+4 hours.
The pack requires NISC contact readiness within T+6 hours for designated critical infrastructure.
The pack requires METI contact readiness within T+6 hours for METI-sector covered incidents.
The pack requires cross-border attack legal review within T+6 hours.
The pack requires first external coordination package within T+24 hours when external contact is required.
The pack requires customer, partner, and public disclosure review within T+24 hours for high severity.
The pack requires regulator-specific statutory clocks to override these internal timelines when stricter.
The pack requires APPI breach clocks to run in parallel when personal information leakage is suspected.
The pack requires telecom serious-accident clocks to run in parallel when telecom secrecy or service continuity is affected.
The pack requires financial regulator clocks to run in parallel when regulated financial services are affected.
The pack uses METI Cybersecurity Management Guidelines as management practice guidance.
The pack requires incident response team procedures.
The pack requires business continuity procedures.
The pack requires supply-chain incident consideration.
The pack requires disclosure timing and content to be specified.
The pack requires evidence such as logs and infected terminals to be preserved where applicable.
The pack requires practical exercises.
The pack requires post-incident recurrence prevention.
The pack requires cross-border attack handoff to legal-ops and security.
The pack requires attacker origin to be treated as investigative hypothesis, not fact, until verified.
The pack requires no public attribution without legal and security approval.
The pack requires source refresh before runtime promotion.
The pack does not build offensive cyber capabilities.
The pack does not replace sector-specific incident statutes.
The pack does not replace privacy breach notification.
The pack does not replace telecom serious-accident reporting.
The pack does not replace financial regulator reporting.
The pack documentation is not legal advice.

## Authority Citations

Authority-001: Basic Act on Cybersecurity is Act No. 104 of 2014.
Authority-002: Japanese Law Translation law view 4755 is the English implementation reference.
Authority-003: The Basic Act states national cybersecurity policy principles and responsibilities.
Authority-004: The Basic Act establishes the Cybersecurity Strategic Headquarters framework.
Authority-005: METI states overall cybersecurity policy coordination is conducted by NISC under the Cybersecurity Strategic Headquarters.
Authority-006: METI states each ministry is responsible for cybersecurity policies in respective areas.
Authority-007: METI states its mission is improving cybersecurity measures in the industrial sector.
Authority-008: METI page lists critical infrastructure areas under its policy scope: gas, electric power, petroleum, credit, chemical.
Authority-009: NISC critical infrastructure page defines critical infrastructure as services whose interruption could greatly affect national life or socioeconomic activity.
Authority-010: NISC action-plan page names 15 critical infrastructure fields.
Authority-011: NISC field 01 is information communications.
Authority-012: NISC field 02 is finance.
Authority-013: NISC field 03 is aviation.
Authority-014: NISC field 04 is airports.
Authority-015: NISC field 05 is railways.
Authority-016: NISC field 06 is electric power.
Authority-017: NISC field 07 is gas.
Authority-018: NISC field 08 is government and administrative services including local governments.
Authority-019: NISC field 09 is medical.
Authority-020: NISC field 10 is water.
Authority-021: NISC field 11 is logistics.
Authority-022: NISC field 12 is chemical.
Authority-023: NISC field 13 is credit.
Authority-024: NISC field 14 is petroleum.
Authority-025: NISC field 15 is ports.
Authority-026: NISC page names competent ministries for critical infrastructure fields.
Authority-027: NISC page names FSA for finance.
Authority-028: NISC page names MIC for information communications and local governments.
Authority-029: NISC page names MHLW for medical.
Authority-030: NISC page names METI for electric power, gas, chemical, credit, and petroleum.
Authority-031: NISC page names MLIT for aviation, airports, railways, water, logistics, and ports.
Authority-032: METI Cybersecurity Management Guidelines Direction 7 requires incident response team and procedures.
Authority-033: METI Direction 8 requires business continuity and recovery team and procedures.
Authority-034: METI Direction 9 requires supply-chain status understanding and measures.
Authority-035: METI Direction 10 requires information gathering, sharing, and disclosure.
Authority-036: METI guideline says management should know who should be notified and what information needs disclosure.
Authority-037: METI guideline recommends prompt reporting to management and concerned parties.
Authority-038: METI guideline recommends preserving evidence such as logs and infected terminals.
Authority-039: METI guideline recommends exercises covering reporting procedures to ministries and agencies.
Authority-040: METI incident page points affected companies to JPCERT/CC for incident response coordination.
Authority-041: METI incident page points organizations to IPA for incident consultation, reports, and vulnerability information.
Authority-042: This pack's T+30 minute classification is an internal control.
Authority-043: This pack's T+1 hour evidence preservation is an internal control.
Authority-044: This pack's T+2 hour management notice is an internal control.
Authority-045: This pack's T+4 hour sector-ministry routing decision is an internal control.
Authority-046: This pack's T+6 hour NISC/METI readiness is an internal control.
Authority-047: This pack's T+24 hour external package target is an internal control.
Authority-048: Sector-specific statutory clocks override pack internal clocks when stricter.
Authority-049: Personal information leakage uses APPI and privacy incident workflows in parallel.
Authority-050: Telecom secrecy or service suspension uses telecom serious-accident workflows in parallel.
Authority-051: Financial regulated incidents use financial service workflows in parallel.
Authority-052: Cross-border attack classification must record evidence and uncertainty.
Authority-053: Cross-border attack notification must route to legal-ops before external attribution.
Authority-054: Critical infrastructure designation is a status, not a self-marketing claim.
Authority-055: Critical infrastructure service and critical system must be service-specific.
Authority-056: A tenant may be critical infrastructure in one service and ordinary in another.
Authority-057: Incident readiness must be tested before production claim.
Authority-058: Public disclosure must be reviewed for accuracy and harm minimization.
Authority-059: Supply-chain incidents require partner and processor coordination.
Authority-060: Source refresh is mandatory before signed promotion.

## Activated Cedar Policies

Policy-001: `pack-jp-cyber-activate` loads cyber incident rules.
Policy-002: `pack-jp-cyber-critical-infra-classification` requires critical infrastructure field classification.
Policy-003: `pack-jp-cyber-critical-infra-info-comms` tags information communications.
Policy-004: `pack-jp-cyber-critical-infra-finance` tags finance.
Policy-005: `pack-jp-cyber-critical-infra-aviation` tags aviation.
Policy-006: `pack-jp-cyber-critical-infra-airports` tags airports.
Policy-007: `pack-jp-cyber-critical-infra-railways` tags railways.
Policy-008: `pack-jp-cyber-critical-infra-electric-power` tags electric power.
Policy-009: `pack-jp-cyber-critical-infra-gas` tags gas.
Policy-010: `pack-jp-cyber-critical-infra-government-admin` tags government and administrative services.
Policy-011: `pack-jp-cyber-critical-infra-medical` tags medical.
Policy-012: `pack-jp-cyber-critical-infra-water` tags water.
Policy-013: `pack-jp-cyber-critical-infra-logistics` tags logistics.
Policy-014: `pack-jp-cyber-critical-infra-chemical` tags chemical.
Policy-015: `pack-jp-cyber-critical-infra-credit` tags credit.
Policy-016: `pack-jp-cyber-critical-infra-petroleum` tags petroleum.
Policy-017: `pack-jp-cyber-critical-infra-ports` tags ports.
Policy-018: `pack-jp-cyber-sector-ministry-owner` assigns sector ministry.
Policy-019: `pack-jp-cyber-nisc-designation-check` checks NISC contact applicability.
Policy-020: `pack-jp-cyber-meti-sector-check` checks METI sector applicability.
Policy-021: `pack-jp-cyber-csirt-required` requires CSIRT owner.
Policy-022: `pack-jp-cyber-ir-plan-required` requires incident response plan.
Policy-023: `pack-jp-cyber-bcp-required` requires business continuity plan.
Policy-024: `pack-jp-cyber-exercise-required` requires practical exercise evidence.
Policy-025: `pack-jp-cyber-supply-chain-check` checks supply-chain impact.
Policy-026: `pack-jp-cyber-incident-open` starts T+0 clock.
Policy-027: `pack-jp-cyber-classify-t30m` requires classification within 30 minutes.
Policy-028: `pack-jp-cyber-preserve-evidence-t1h` requires evidence preservation within one hour.
Policy-029: `pack-jp-cyber-management-notice-t2h` requires high-severity management notice within two hours.
Policy-030: `pack-jp-cyber-sector-routing-t4h` requires ministry routing decision within four hours.
Policy-031: `pack-jp-cyber-nisc-readiness-t6h` requires NISC package readiness within six hours when applicable.
Policy-032: `pack-jp-cyber-meti-readiness-t6h` requires METI package readiness within six hours when applicable.
Policy-033: `pack-jp-cyber-cross-border-review-t6h` requires cross-border attack review within six hours.
Policy-034: `pack-jp-cyber-external-package-t24h` requires external package target within 24 hours when applicable.
Policy-035: `pack-jp-cyber-disclosure-review-t24h` requires disclosure review within 24 hours for high severity.
Policy-036: `pack-jp-cyber-applies-stricter-sector-clock` applies stricter parallel regulatory clocks.
Policy-037: `pack-jp-cyber-appi-parallel` opens APPI leakage workflow when PI may be affected.
Policy-038: `pack-jp-cyber-telecom-parallel` opens telecom serious-accident workflow when telecom affected.
Policy-039: `pack-jp-cyber-finance-parallel` opens financial regulator workflow when finance affected.
Policy-040: `pack-jp-cyber-cross-border-attack-flag` records cross-border attack flag.
Policy-041: `pack-jp-cyber-attribution-deny-public` blocks public attribution without approval.
Policy-042: `pack-jp-cyber-legal-ops-cross-border` routes cross-border attack to legal-ops.
Policy-043: `pack-jp-cyber-jpcert-consultation` records JPCERT/CC consultation path.
Policy-044: `pack-jp-cyber-ipa-consultation` records IPA consultation path.
Policy-045: `pack-jp-cyber-log-preservation` preserves logs.
Policy-046: `pack-jp-cyber-terminal-preservation` preserves affected endpoints where applicable.
Policy-047: `pack-jp-cyber-forensics-chain` requires evidence chain of custody.
Policy-048: `pack-jp-cyber-containment-owner` requires containment owner.
Policy-049: `pack-jp-cyber-recovery-owner` requires recovery owner.
Policy-050: `pack-jp-cyber-recurrence-prevention` requires recurrence prevention record.
Policy-051: `pack-jp-cyber-postmortem-required` requires post-incident review.
Policy-052: `pack-jp-cyber-partner-notification` requires partner impact review.
Policy-053: `pack-jp-cyber-customer-notification-review` requires customer notification review.
Policy-054: `pack-jp-cyber-public-disclosure-review` requires public disclosure review.
Policy-055: `pack-jp-cyber-no-offensive-tooling` blocks offensive capability creation.
Policy-056: `pack-jp-cyber-audit-redaction` redacts cyber audit payloads.
Policy-057: `pack-jp-cyber-exception-expiry` requires exception expiry.
Policy-058: `pack-jp-cyber-source-stale-deny` blocks stale authority snapshot.
Policy-059: `pack-jp-cyber-promote-evidence-required` blocks promotion without evidence.
Policy-060: `pack-jp-cyber-deactivation-block-open-incident` blocks pack deactivation during open incident.

## Data Model Deltas

Data-001: Add `data_class.CYBER_JP_INCIDENT`.
Data-002: Add `data_class.CYBER_JP_CRITICAL_INFRA`.
Data-003: Add `data_class.CYBER_JP_CROSS_BORDER_ATTACK`.
Data-004: Add `cyber.jp_incident_id`.
Data-005: Add `cyber.jp_incident_awareness_at`.
Data-006: Add `cyber.jp_incident_severity`.
Data-007: Add `cyber.jp_incident_classified_at`.
Data-008: Add `cyber.jp_incident_t30m_due_at`.
Data-009: Add `cyber.jp_evidence_preserved_at`.
Data-010: Add `cyber.jp_evidence_t1h_due_at`.
Data-011: Add `cyber.jp_management_notified_at`.
Data-012: Add `cyber.jp_management_t2h_due_at`.
Data-013: Add `cyber.jp_sector_routing_decided_at`.
Data-014: Add `cyber.jp_sector_t4h_due_at`.
Data-015: Add `cyber.jp_nisc_ready_at`.
Data-016: Add `cyber.jp_nisc_t6h_due_at`.
Data-017: Add `cyber.jp_meti_ready_at`.
Data-018: Add `cyber.jp_meti_t6h_due_at`.
Data-019: Add `cyber.jp_cross_border_reviewed_at`.
Data-020: Add `cyber.jp_cross_border_t6h_due_at`.
Data-021: Add `cyber.jp_external_package_ready_at`.
Data-022: Add `cyber.jp_external_package_t24h_due_at`.
Data-023: Add `cyber.jp_disclosure_reviewed_at`.
Data-024: Add `cyber.jp_disclosure_t24h_due_at`.
Data-025: Add `cyber.jp_critical_infra_field`.
Data-026: Add `cyber.jp_critical_infra_service`.
Data-027: Add `cyber.jp_critical_system_ref`.
Data-028: Add `cyber.jp_sector_ministry`.
Data-029: Add `cyber.jp_nisc_contact_required_flag`.
Data-030: Add `cyber.jp_nisc_contact_ref`.
Data-031: Add `cyber.jp_meti_contact_required_flag`.
Data-032: Add `cyber.jp_meti_contact_ref`.
Data-033: Add `cyber.jp_csirt_owner`.
Data-034: Add `cyber.jp_ir_plan_ref`.
Data-035: Add `cyber.jp_bcp_ref`.
Data-036: Add `cyber.jp_exercise_evidence_ref`.
Data-037: Add `cyber.jp_supply_chain_impact_flag`.
Data-038: Add `cyber.jp_supplier_contact_ref`.
Data-039: Add `cyber.jp_cross_border_attack_flag`.
Data-040: Add `cyber.jp_cross_border_origin_hypothesis`.
Data-041: Add `cyber.jp_cross_border_confidence`.
Data-042: Add `cyber.jp_public_attribution_approved_flag`.
Data-043: Add `cyber.jp_jpcert_consultation_ref`.
Data-044: Add `cyber.jp_ipa_consultation_ref`.
Data-045: Add `cyber.jp_log_preservation_ref`.
Data-046: Add `cyber.jp_terminal_preservation_ref`.
Data-047: Add `cyber.jp_chain_of_custody_ref`.
Data-048: Add `cyber.jp_containment_owner`.
Data-049: Add `cyber.jp_recovery_owner`.
Data-050: Add `cyber.jp_recurrence_prevention_ref`.
Data-051: Add `cyber.jp_postmortem_ref`.
Data-052: Add `cyber.jp_partner_notification_review_ref`.
Data-053: Add `cyber.jp_customer_notification_review_ref`.
Data-054: Add `cyber.jp_public_disclosure_review_ref`.
Data-055: Add `cyber.jp_parallel_appi_clock_id`.
Data-056: Add `cyber.jp_parallel_telecom_clock_id`.
Data-057: Add `cyber.jp_parallel_finance_clock_id`.
Data-058: Add `audit.jp_cyber_event_type`.
Data-059: Add `audit.jp_cyber_redaction_profile`.
Data-060: Add `tenant.jp_cyber_deactivation_block_reason`.

## API Contract Deltas

API-001: Add `POST /cyber/jp/critical-infra/classify`.
API-002: Add `GET /cyber/jp/critical-infra/fields`.
API-003: Add `POST /cyber/jp/sector-ministry/assign`.
API-004: Add `POST /cyber/jp/readiness/csirt`.
API-005: Add `POST /cyber/jp/readiness/ir-plan`.
API-006: Add `POST /cyber/jp/readiness/bcp`.
API-007: Add `POST /cyber/jp/readiness/exercise`.
API-008: Add `POST /cyber/jp/incident/open`.
API-009: Add `POST /cyber/jp/incident/classify`.
API-010: Add `POST /cyber/jp/incident/preserve-evidence`.
API-011: Add `POST /cyber/jp/incident/management-notice`.
API-012: Add `POST /cyber/jp/incident/sector-routing`.
API-013: Add `POST /cyber/jp/incident/nisc-readiness`.
API-014: Add `POST /cyber/jp/incident/meti-readiness`.
API-015: Add `POST /cyber/jp/incident/cross-border-review`.
API-016: Add `POST /cyber/jp/incident/external-package`.
API-017: Add `POST /cyber/jp/incident/disclosure-review`.
API-018: Add `POST /cyber/jp/incident/parallel-clocks`.
API-019: Add `POST /cyber/jp/incident/jpcert-consultation`.
API-020: Add `POST /cyber/jp/incident/ipa-consultation`.
API-021: Add `POST /cyber/jp/incident/log-preservation`.
API-022: Add `POST /cyber/jp/incident/terminal-preservation`.
API-023: Add `POST /cyber/jp/incident/chain-of-custody`.
API-024: Add `POST /cyber/jp/incident/containment-owner`.
API-025: Add `POST /cyber/jp/incident/recovery-owner`.
API-026: Add `POST /cyber/jp/incident/partner-review`.
API-027: Add `POST /cyber/jp/incident/customer-review`.
API-028: Add `POST /cyber/jp/incident/public-disclosure-review`.
API-029: Add `POST /cyber/jp/incident/recurrence-prevention`.
API-030: Add `POST /cyber/jp/incident/postmortem`.
API-031: Add `POST /cyber/jp/exception`.
API-032: Add `POST /audit/jp/cyber/event`.
API-033: Require `incident_awareness_at` on incident open.
API-034: Require `severity` within T+30 minutes.
API-035: Require `evidence_preservation_ref` within T+1 hour.
API-036: Require `management_notice_ref` within T+2 hours for high severity.
API-037: Require `sector_ministry` within T+4 hours.
API-038: Require `nisc_package_ref` within T+6 hours when applicable.
API-039: Require `meti_package_ref` within T+6 hours when applicable.
API-040: Require `cross_border_review_ref` within T+6 hours when cross-border flag is true.
API-041: Require `external_package_ref` within T+24 hours when external contact is required.
API-042: Require `disclosure_review_ref` within T+24 hours for high severity.
API-043: Return `403 cyber_pack_not_active` when pack is missing.
API-044: Return `409 cyber_critical_infra_classification_required` when field is missing.
API-045: Return `409 cyber_csirt_required` when owner is missing.
API-046: Return `409 cyber_evidence_preservation_required` when preservation is missing.
API-047: Return `409 cyber_sector_routing_required` when ministry routing is missing.
API-048: Return `409 cyber_cross_border_review_required` when cross-border review is missing.
API-049: Return `451 cyber_public_attribution_blocked` without approval.
API-050: Return `409 cyber_parallel_clock_required` when sector law applies.
API-051: Require idempotency keys on mutating incident APIs.
API-052: Require tenant and sub-scope headers.
API-053: Default audit reads to redacted incident evidence.
API-054: Never expose raw secrets in cyber audit reads.
API-055: Block deactivation while incident is open.

## Audit Event Additions

Audit-001: Emit `EVT-JP-CYBER-CRITICAL-INFRA-CLASSIFIED`.
Audit-002: Emit `EVT-JP-CYBER-SECTOR-MINISTRY-ASSIGNED`.
Audit-003: Emit `EVT-JP-CYBER-CSIRT-EVIDENCE`.
Audit-004: Emit `EVT-JP-CYBER-IR-PLAN-EVIDENCE`.
Audit-005: Emit `EVT-JP-CYBER-BCP-EVIDENCE`.
Audit-006: Emit `EVT-JP-CYBER-EXERCISE-EVIDENCE`.
Audit-007: Emit `EVT-JP-CYBER-INCIDENT-OPENED`.
Audit-008: Emit `EVT-JP-CYBER-INCIDENT-CLASSIFIED`.
Audit-009: Emit `EVT-JP-CYBER-T30M-MISSED`.
Audit-010: Emit `EVT-JP-CYBER-EVIDENCE-PRESERVED`.
Audit-011: Emit `EVT-JP-CYBER-T1H-MISSED`.
Audit-012: Emit `EVT-JP-CYBER-MANAGEMENT-NOTIFIED`.
Audit-013: Emit `EVT-JP-CYBER-T2H-MISSED`.
Audit-014: Emit `EVT-JP-CYBER-SECTOR-ROUTED`.
Audit-015: Emit `EVT-JP-CYBER-T4H-MISSED`.
Audit-016: Emit `EVT-JP-CYBER-NISC-READY`.
Audit-017: Emit `EVT-JP-CYBER-METI-READY`.
Audit-018: Emit `EVT-JP-CYBER-T6H-MISSED`.
Audit-019: Emit `EVT-JP-CYBER-CROSS-BORDER-REVIEWED`.
Audit-020: Emit `EVT-JP-CYBER-EXTERNAL-PACKAGE-READY`.
Audit-021: Emit `EVT-JP-CYBER-T24H-MISSED`.
Audit-022: Emit `EVT-JP-CYBER-DISCLOSURE-REVIEWED`.
Audit-023: Emit `EVT-JP-CYBER-PARALLEL-APPI-CLOCK`.
Audit-024: Emit `EVT-JP-CYBER-PARALLEL-TELECOM-CLOCK`.
Audit-025: Emit `EVT-JP-CYBER-PARALLEL-FINANCE-CLOCK`.
Audit-026: Emit `EVT-JP-CYBER-JPCERT-CONSULTATION`.
Audit-027: Emit `EVT-JP-CYBER-IPA-CONSULTATION`.
Audit-028: Emit `EVT-JP-CYBER-LOG-PRESERVED`.
Audit-029: Emit `EVT-JP-CYBER-TERMINAL-PRESERVED`.
Audit-030: Emit `EVT-JP-CYBER-CHAIN-OF-CUSTODY`.
Audit-031: Emit `EVT-JP-CYBER-CONTAINMENT-OWNER`.
Audit-032: Emit `EVT-JP-CYBER-RECOVERY-OWNER`.
Audit-033: Emit `EVT-JP-CYBER-PARTNER-REVIEW`.
Audit-034: Emit `EVT-JP-CYBER-CUSTOMER-REVIEW`.
Audit-035: Emit `EVT-JP-CYBER-PUBLIC-DISCLOSURE-REVIEW`.
Audit-036: Emit `EVT-JP-CYBER-PUBLIC-ATTRIBUTION-BLOCKED`.
Audit-037: Emit `EVT-JP-CYBER-RECURRENCE-PREVENTION`.
Audit-038: Emit `EVT-JP-CYBER-POSTMORTEM`.
Audit-039: Emit `EVT-JP-CYBER-OFFENSIVE-TOOLING-BLOCKED`.
Audit-040: Emit `EVT-JP-CYBER-EXCEPTION-CREATED`.
Audit-041: Emit `EVT-JP-CYBER-EXCEPTION-EXPIRED`.
Audit-042: Emit `EVT-JP-CYBER-AUDIT-REDACTED`.
Audit-043: Emit `EVT-JP-CYBER-SOURCE-SNAPSHOT-STALE`.
Audit-044: Emit `EVT-JP-CYBER-PROMOTION-EVIDENCE-SEALED`.
Audit-045: Emit `EVT-JP-CYBER-DEACTIVATION-BLOCKED`.

## Failure Modes

Failure-001: Critical infrastructure field is missing.
Failure-002: Service claims critical infrastructure without NISC field mapping.
Failure-003: Sector ministry owner is missing.
Failure-004: CSIRT owner is missing.
Failure-005: Incident response plan is missing.
Failure-006: Business continuity plan is missing.
Failure-007: Exercise evidence is missing.
Failure-008: Incident opens without awareness timestamp.
Failure-009: Severity classification misses T+30 minutes.
Failure-010: Evidence preservation misses T+1 hour.
Failure-011: Management notification misses T+2 hours for high severity.
Failure-012: Sector routing misses T+4 hours.
Failure-013: NISC readiness misses T+6 hours when applicable.
Failure-014: METI readiness misses T+6 hours when applicable.
Failure-015: Cross-border attack review misses T+6 hours.
Failure-016: External package misses T+24 hours when required.
Failure-017: Disclosure review misses T+24 hours for high severity.
Failure-018: APPI leakage clock is not opened.
Failure-019: Telecom serious-accident clock is not opened.
Failure-020: Financial incident clock is not opened.
Failure-021: Cross-border attack origin is asserted as fact without evidence.
Failure-022: Public attribution is made without approval.
Failure-023: JPCERT/CC path is ignored when technical coordination is needed.
Failure-024: IPA path is ignored when vulnerability report is needed.
Failure-025: Logs are not preserved.
Failure-026: Affected terminals are reimaged before evidence decision.
Failure-027: Chain of custody is missing.
Failure-028: Containment owner is missing.
Failure-029: Recovery owner is missing.
Failure-030: Supplier impact is not assessed.
Failure-031: Partner notification review is missing.
Failure-032: Customer notification review is missing.
Failure-033: Public disclosure review is missing.
Failure-034: Recurrence prevention is missing.
Failure-035: Postmortem is missing.
Failure-036: Offensive tooling is introduced.
Failure-037: Audit event contains secrets.
Failure-038: Exception has no expiry.
Failure-039: Source snapshot is stale.
Failure-040: Pack deactivation occurs during open incident.

## Worked Examples

Example-001: A finance tenant is classified as critical infrastructure field finance.
Example-002: Sector ministry owner is FSA.
Example-003: The tenant also has METI-relevant cloud supply-chain dependency.
Example-004: A ransomware event is detected at 10:00 JST.
Example-005: Incident open sets T+0 at 10:00 JST.
Example-006: Severity classification is due by 10:30 JST.
Example-007: Evidence preservation is due by 11:00 JST.
Example-008: Management notice is due by 12:00 JST for high severity.
Example-009: Sector routing is due by 14:00 JST.
Example-010: NISC and METI readiness are due by 16:00 JST if applicable.
Example-011: Cross-border attack review is due by 16:00 JST if flagged.
Example-012: External package target is due by 10:00 JST next day when required.
Example-013: Disclosure review is due by 10:00 JST next day for high severity.
Example-014: Logs and infected terminals are preserved.
Example-015: Chain of custody is created.
Example-016: APPI parallel clock opens because customer data may be affected.
Example-017: Financial parallel clock opens because regulated service is affected.
Example-018: The team suspects foreign attacker infrastructure.
Example-019: Cross-border attack flag is set as hypothesis.
Example-020: Public attribution is blocked.
Example-021: Legal-ops reviews external contact.
Example-022: Security prepares technical indicators.
Example-023: JPCERT/CC consultation path is recorded if technical coordination is needed.
Example-024: IPA consultation path is recorded if vulnerability reporting is needed.
Example-025: A medical tenant is classified as critical infrastructure field medical.
Example-026: Sector ministry owner is MHLW.
Example-027: A supply-chain vendor breach is reported.
Example-028: Supply-chain impact flag is set.
Example-029: Partner notification review is opened.
Example-030: Customer notification review is opened.
Example-031: Business continuity plan is checked.
Example-032: Recovery owner is assigned.
Example-033: A power tenant is classified as electric power.
Example-034: Sector ministry owner is METI.
Example-035: METI readiness path is required.
Example-036: The event affects OT monitoring but no customer personal data.
Example-037: APPI parallel clock remains closed.
Example-038: Cyber incident clock remains open.
Example-039: A telecom tenant has service suspension.
Example-040: Telecom serious-accident parallel clock opens.
Example-041: MIC path is evaluated.
Example-042: NISC path is evaluated if critical infrastructure designation applies.
Example-043: A port logistics tenant detects DDoS.
Example-044: Sector ministry owner is MLIT.
Example-045: Critical service impact is evaluated.
Example-046: External package is prepared if contact is required.
Example-047: A developer asks to deploy offensive countermeasures.
Example-048: Offensive tooling policy blocks the request.
Example-049: A press draft names a nation-state actor.
Example-050: Public attribution policy blocks until approval.
Example-051: A postmortem identifies missing exercise.
Example-052: Exercise evidence requirement blocks future readiness claim.
Example-053: A source refresh finds METI guidance updated.
Example-054: Authority snapshot becomes stale.
Example-055: Promotion blocks until documentation and runtime controls refresh.

## Cross-References

CrossRef-001: See `README.md` for JP pack activation and precedence.
CrossRef-002: See `appi-personal-information-protection.md` for privacy leakage parallel clocks.
CrossRef-003: See `telecommunications-business-act.md` for telecom serious-accident parallel clocks.
CrossRef-004: See `financial-services-act-and-banking-act.md` for finance parallel clocks.
CrossRef-005: See Japanese Law Translation law view 4755 for the Basic Act.
CrossRef-006: See NISC critical infrastructure action-plan page for named fields.
CrossRef-007: See METI cybersecurity page for ministry coordination context.
CrossRef-008: See METI Cybersecurity Management Guidelines for incident-response directions.
CrossRef-009: See METI incident page for JPCERT/CC and IPA routing references.
CrossRef-010: See ADR-0064 for canonical base controls.
CrossRef-011: See ADR-0244 for tenant and sub-scope context.
CrossRef-012: See ADR-0251 for compliance-pack mechanics.
CrossRef-013: See ADR-0263 for audit redaction.
CrossRef-014: Security owns incident classification and evidence preservation.
CrossRef-015: Legal-ops owns cross-border attack review and attribution approval.
CrossRef-016: GRC owns critical infrastructure designation evidence.
CrossRef-017: Ops Dashboard owns incident clock display.
CrossRef-018: Control Center owns emergency operator actions.
CrossRef-019: Workflow owns timeline enforcement.
CrossRef-020: Audit-chain owns redacted replay.
CrossRef-021: Runtime tests must cover every NISC field named in this document.
CrossRef-022: Runtime tests must prove T+30 minute classification gate.
CrossRef-023: Runtime tests must prove T+1 hour evidence gate.
CrossRef-024: Runtime tests must prove T+6 hour cross-border review gate.
CrossRef-025: Runtime tests must prove public attribution is blocked without approval.
CrossRef-026: Runtime tests must prove stricter sector clocks override internal pack clocks.
CrossRef-027: Documentation review must confirm internal timelines are not misrepresented as universal statutory deadlines.
CrossRef-028: Checkpoint state for this document is authored and ready for line-count verification.
CrossRef-029: Field tests must classify information communications.
CrossRef-030: Field tests must classify finance.
CrossRef-031: Field tests must classify aviation.
CrossRef-032: Field tests must classify airports.
CrossRef-033: Field tests must classify railways.
CrossRef-034: Field tests must classify electric power.
CrossRef-035: Field tests must classify gas.
CrossRef-036: Field tests must classify government and administrative services.
CrossRef-037: Field tests must classify medical.
CrossRef-038: Field tests must classify water.
CrossRef-039: Field tests must classify logistics.
CrossRef-040: Field tests must classify chemical.
CrossRef-041: Field tests must classify credit.
CrossRef-042: Field tests must classify petroleum.
CrossRef-043: Field tests must classify ports.
CrossRef-044: Ministry tests must route information communications to MIC.
CrossRef-045: Ministry tests must route local government services to MIC.
CrossRef-046: Ministry tests must route finance to FSA.
CrossRef-047: Ministry tests must route medical to MHLW.
CrossRef-048: Ministry tests must route electric power to METI.
CrossRef-049: Ministry tests must route gas to METI.
CrossRef-050: Ministry tests must route chemical to METI.
CrossRef-051: Ministry tests must route credit to METI.
CrossRef-052: Ministry tests must route petroleum to METI.
CrossRef-053: Ministry tests must route aviation to MLIT.
CrossRef-054: Ministry tests must route airports to MLIT.
CrossRef-055: Ministry tests must route railways to MLIT.
CrossRef-056: Ministry tests must route water to MLIT.
CrossRef-057: Ministry tests must route logistics to MLIT.
CrossRef-058: Ministry tests must route ports to MLIT.
CrossRef-059: Readiness tests must require CSIRT owner before critical infrastructure activation.
CrossRef-060: Readiness tests must require incident response plan before production claim.
CrossRef-061: Readiness tests must require business continuity plan before production claim.
CrossRef-062: Readiness tests must require exercise evidence before production claim.
CrossRef-063: Supply-chain tests must require supplier impact assessment.
CrossRef-064: Timeline tests must create T+0 at awareness.
CrossRef-065: Timeline tests must fail missing T+30 classification.
CrossRef-066: Timeline tests must fail missing T+1 evidence preservation.
CrossRef-067: Timeline tests must fail missing T+2 management notice for high severity.
CrossRef-068: Timeline tests must fail missing T+4 sector routing.
CrossRef-069: Timeline tests must fail missing T+6 NISC readiness when applicable.
CrossRef-070: Timeline tests must fail missing T+6 METI readiness when applicable.
CrossRef-071: Timeline tests must fail missing T+6 cross-border review when flagged.
CrossRef-072: Timeline tests must fail missing T+24 external package when required.
CrossRef-073: Timeline tests must fail missing T+24 disclosure review for high severity.
CrossRef-074: Parallel-clock tests must open APPI workflow when personal information may be affected.
CrossRef-075: Parallel-clock tests must open telecom workflow when secrecy or service continuity may be affected.
CrossRef-076: Parallel-clock tests must open financial workflow when regulated finance may be affected.
CrossRef-077: Parallel-clock tests must prove stricter statutory clocks replace pack internal targets.
CrossRef-078: Cross-border tests must treat origin as hypothesis by default.
CrossRef-079: Cross-border tests must require confidence level.
CrossRef-080: Cross-border tests must require legal-ops review.
CrossRef-081: Cross-border tests must block public attribution without approval.
CrossRef-082: Evidence tests must require log preservation reference.
CrossRef-083: Evidence tests must require terminal preservation decision.
CrossRef-084: Evidence tests must require chain of custody.
CrossRef-085: Containment tests must require containment owner.
CrossRef-086: Recovery tests must require recovery owner.
CrossRef-087: Partner tests must require partner notification review.
CrossRef-088: Customer tests must require customer notification review.
CrossRef-089: Disclosure tests must require public disclosure review.
CrossRef-090: JPCERT tests must record consultation path when technical coordination is used.
CrossRef-091: IPA tests must record consultation path when vulnerability reporting is used.
CrossRef-092: Postmortem tests must require recurrence prevention record.
CrossRef-093: Postmortem tests must require postmortem reference.
CrossRef-094: Offensive-control tests must block offensive tooling.
CrossRef-095: Audit tests must redact secrets.
CrossRef-096: Audit tests must preserve clock timestamps.
CrossRef-097: Audit tests must preserve sector routing decision.
CrossRef-098: Audit tests must preserve cross-border review decision.
CrossRef-099: Audit tests must preserve public attribution approval state.
CrossRef-100: Exception tests must require expiry.
CrossRef-101: Deactivation tests must block pack deactivation during open cyber incident.
CrossRef-102: Source tests must prove Basic Act source snapshot is current.
CrossRef-103: Source tests must prove NISC critical infrastructure page snapshot is current.
CrossRef-104: Source tests must prove METI cybersecurity page snapshot is current.
CrossRef-105: Source tests must prove METI guideline snapshot is current.
CrossRef-106: Source tests must prove METI incident page snapshot is current.
CrossRef-107: Documentation review must confirm all 15 NISC fields are named.
CrossRef-108: Documentation review must confirm all sector ministries named from NISC page are mapped.
CrossRef-109: Documentation review must confirm METI Direction 7 is reflected.
CrossRef-110: Documentation review must confirm METI Direction 8 is reflected.
CrossRef-111: Documentation review must confirm METI Direction 9 is reflected.
CrossRef-112: Documentation review must confirm METI Direction 10 is reflected.
CrossRef-113: Documentation review must confirm evidence preservation is included.
CrossRef-114: Documentation review must confirm incident exercises are included.
CrossRef-115: Documentation review must confirm cross-border attack notification route is included.
CrossRef-116: Documentation review must confirm no offensive tooling is authorized.
CrossRef-117: Documentation review must confirm no universal statutory hour limit is invented.
CrossRef-118: Runtime review must confirm NISC readiness is conditional on applicability.
CrossRef-119: Runtime review must confirm METI readiness is conditional on applicability.
CrossRef-120: Runtime review must confirm sector law overlays can override internal clocks.
CrossRef-121: Runtime review must confirm APPI, telecom, and finance workflows run in parallel.
CrossRef-122: Runtime review must confirm cyber audit payloads are redacted.
CrossRef-123: Runtime review must confirm public disclosure cannot bypass review.
CrossRef-124: Runtime review must confirm customer notification review is separate from public disclosure.
CrossRef-125: Runtime review must confirm partner notification review is separate from customer notification.
CrossRef-126: Runtime review must confirm chain of custody cannot be edited silently.
CrossRef-127: Runtime review must confirm postmortem cannot close without recurrence prevention.
CrossRef-128: Runtime review must confirm missed internal targets emit missed-clock events.
CrossRef-129: Runtime review must confirm management notification is required for high severity.
CrossRef-130: Runtime review must confirm field classification is service-specific.
CrossRef-131: Runtime review must confirm a tenant can have multiple critical infrastructure fields.
CrossRef-132: Runtime review must confirm ordinary tenants still get baseline incident controls.
CrossRef-133: Runtime review must confirm legal-ops owns cross-border attribution.
CrossRef-134: Runtime review must confirm security owns technical classification.
CrossRef-135: Runtime review must confirm GRC owns critical infrastructure designation evidence.
CrossRef-136: Runtime review must confirm workflow owns timeline enforcement.
CrossRef-137: Runtime review must confirm ops-dashboard shows open clocks.
CrossRef-138: Runtime review must confirm control-center actions are audit-bound.
CrossRef-139: Pack promotion must attach `jp_pack_docs:6`.
CrossRef-140: Checkpoint state for this document is line-counted and ready for VCS verification.
