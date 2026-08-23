---
doc_class: LocalizationPack
pack_id: KR-PACK-1
doc_id: KR-PACK-1-CYBERSECURITY-INCIDENT-RESPONSE
title: Korea Localization Pack Cybersecurity and Incident Response
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
  - https://www.kisa.or.kr/
  - https://isms.kisa.or.kr/main/csap/intro/index.jsp
---

# Korea Cybersecurity and Incident Response

This document defines KR-PACK-1 cybersecurity and incident response behavior.
It covers controlled-source breach criteria, KISA notification workflow, PIPA/PIPC breach reporting, CSAP incident evidence, and ADR-0263 audit emissions.
It treats Korean incidents as cross-classified events.
A security incident may trigger cybersecurity, privacy, communications, healthcare, electronic-document, and CSAP controls at the same time.
The pack does not allow a single generic incident label to suppress Korean legal clocks.

## Incident Doctrine

Every Korean incident must be classified against privacy impact.
Every Korean incident must be classified against KISA notification criteria.
Every Korean incident must be classified against controlled-source criteria.
Every Korean incident must be classified against CSAP impact when CSAP workload is involved.
Every Korean incident must be classified against medical record impact when healthcare data is involved.
Every Korean incident must be classified against communications secrecy impact when messages or metadata are involved.
Every Korean incident must be classified against electronic document integrity impact when evidence documents are involved.
Every Korean incident must record first-detection time.
Every Korean incident must record classification time.
Every Korean incident must record containment start time.
Every Korean incident must record notification decision time.
Every Korean incident must record notification submission time.
Every Korean incident must preserve evidence in a KR-approved cell.
Every Korean incident must scrub routine audit payloads.
Every Korean incident must block raw PII in tickets.
Every Korean incident must block raw RRN in tickets.
Every Korean incident must block non-KR evidence export unless emergency transfer basis exists.
Every Korean incident must preserve legal hold state.
Every Korean incident must identify tenant and sub-scope.
Every Korean incident must name activated Cedar policies.
Every Korean incident must produce closure evidence.

## Authority Citations

Authority snapshot date: 2026-05-20.
Primary law source: `https://www.law.go.kr/`.
Primary privacy regulator source: `https://www.pipc.go.kr/`.
Primary KISA source: `https://www.kisa.or.kr/`.
Primary CSAP source: `https://isms.kisa.or.kr/main/csap/intro/index.jsp`.
KR Cyber Security Act product label maps to Information and Communications Infrastructure Protection Act, `정보통신기반 보호법`, unless a newer official statute supersedes it.
Information and Communications Infrastructure Protection Act Article 8 governs designation of major information and communications infrastructure.
Information and Communications Infrastructure Protection Act Article 13 governs incident notification to relevant administrative agency, investigative agency, or KISA where criteria apply.
Information and Communications Infrastructure Protection Act Enforcement Decree Article 21 names notification contents such as incident occurrence time and facility, damage details, and other response details.
Information Network Act, `정보통신망 이용촉진 및 정보보호 등에 관한 법률`, informs network service incident cooperation and security measures.
Information Network Act Article 48-3 lineage informs incident reporting and post-incident cooperation where applicable.
PIPA Article 34 governs personal information leakage notification and reporting.
PIPA Enforcement Decree Article 40 governs 72-hour reporting for enumerated leakage cases to PIPC or KISA.
PIPA Article 24-2 governs RRN leakage severity and special handling.
PIPA Article 23 governs sensitive information leakage severity.
Cloud Computing Development and User Protection Act Article 23-2 anchors CSAP certification.
KISA CSAP portal describes cloud security certification controls.
Medical Service Act Article 22 governs medical records affected by incidents.
Medical Service Act Article 23 governs electronic medical record integrity.
Communications Secrets Protection Act governs incidents involving communication content or confirmation data.
Digital Documents and Transactions Act governs incidents affecting electronic document evidence.
Named effective dates must be read from law.go.kr for final bundle build.
Official Korean statutory text controls over delayed translations.
KISA and PIPC current guidance must be checked before production incident runbook release.

## KISA Notification Timeline

Timeline `KR-INC-T0-DETECTED` starts at first credible detection.
Timeline `KR-INC-T0+15M-TRIAGE` targets initial Korean-regulated-data triage within 15 minutes.
Timeline `KR-INC-T0+1H-CLASSIFY` targets cross-classification within 1 hour.
Timeline `KR-INC-T0+2H-CONTAIN` targets containment action owner assignment within 2 hours.
Timeline `KR-INC-T0+4H-KISA-PRECHECK` targets KISA notification precheck for controlled-source incidents.
Timeline `KR-INC-T0+8H-LEGAL-PRIVACY-REVIEW` targets legal and privacy notification review.
Timeline `KR-INC-T0+24H-KISA-TARGET` is the KR-PACK-1 internal target for KISA notification when controlled-source or critical-infrastructure criteria are met.
Timeline `KR-INC-T0+24H-TENANT-BRIEF` targets affected tenant briefing when disclosure is legally and operationally appropriate.
Timeline `KR-INC-T0+48H-SUPPLEMENTAL-EVIDENCE` targets supplemental evidence package.
Timeline `KR-INC-T0+72H-PIPA-REPORT` tracks PIPA Enforcement Decree Article 40 report timing for enumerated personal-data leakage.
Timeline `KR-INC-T0+72H-POSTURE-REVIEW` targets control posture review.
Timeline `KR-INC-CLOSE-LEGAL-HOLD` closes only after legal hold and preservation duties are resolved.
The 24-hour KISA target is an internal KR-PACK-1 stricter operational target.
The 72-hour PIPA report clock is separately tracked for reportable personal-data leakage.
KISA notification and PIPC/PIPA reporting are not interchangeable.
If a Korean incident meets both cybersecurity and privacy criteria, both clocks run.
If incident details are incomplete, preliminary report state must be recorded.
If notification is delayed by unavoidable circumstances, the reason must be recorded.
If notification is not required, the legal basis for non-notification must be recorded.
If evidence changes after notification, supplemental update must be recorded.

## Controlled-Source Breach Criteria

Criterion `KR-CSB-001` unauthorized access to KR regulated production system.
Criterion `KR-CSB-002` access exceeding authorized privileges.
Criterion `KR-CSB-003` manipulation of stored data.
Criterion `KR-CSB-004` destruction of stored data.
Criterion `KR-CSB-005` concealment of stored data.
Criterion `KR-CSB-006` leakage of stored data.
Criterion `KR-CSB-007` malware introduced into KR regulated service.
Criterion `KR-CSB-008` logic bomb introduced into KR regulated service.
Criterion `KR-CSB-009` denial-of-service disrupting KR regulated service.
Criterion `KR-CSB-010` false command causing processing error.
Criterion `KR-CSB-011` ransomware affecting KR regulated service.
Criterion `KR-CSB-012` compromise of KR identity key material.
Criterion `KR-CSB-013` compromise of RRN derivative key context.
Criterion `KR-CSB-014` compromise of medical record integrity.
Criterion `KR-CSB-015` compromise of communications content secrecy.
Criterion `KR-CSB-016` compromise of electronic document evidence integrity.
Criterion `KR-CSB-017` compromise of CSAP-sensitive workload boundary.
Criterion `KR-CSB-018` unauthorized evidence export.
Criterion `KR-CSB-019` unauthorized processor access.
Criterion `KR-CSB-020` insider misuse of privileged access.
Criterion `KR-CSB-021` supply-chain compromise affecting KR pack controls.
Criterion `KR-CSB-022` audit emission bypass for Korean regulated events.
Criterion `KR-CSB-023` Cedar policy bypass affecting KR controls.
Criterion `KR-CSB-024` residency bypass causing non-KR processing.
Criterion `KR-CSB-025` backup exfiltration of Korean regulated data.
Criterion `KR-CSB-026` logging pipeline leak of Korean PII.
Criterion `KR-CSB-027` telemetry pipeline leak of Korean PII.
Criterion `KR-CSB-028` support tooling leak of Korean PII.
Criterion `KR-CSB-029` test environment leak of Korean production data.
Criterion `KR-CSB-030` unapproved cross-border transfer during incident response.
Meeting one criterion requires controlled-source review.
Meeting one criterion with critical infrastructure impact requires KISA notification precheck.
Meeting one criterion with personal data leakage requires PIPA breach classification.
Meeting one criterion with RRN leakage requires high-severity privacy escalation.
Meeting one criterion with medical record impact requires healthcare legal review.
Meeting one criterion with communications content impact requires communications secrecy review.

## Incident Severity Bands

Severity `KR-INC-S0` is false positive with documented reason.
Severity `KR-INC-S1` is minor event with no regulated data and no service disruption.
Severity `KR-INC-S2` is regulated-data exposure risk without confirmed leakage.
Severity `KR-INC-S3` is confirmed Korean personal data leakage.
Severity `KR-INC-S4` is sensitive data, RRN, medical, communications, or CSAP-sensitive incident.
Severity `KR-INC-S5` is critical infrastructure, systemic compromise, or active exploitation with Korean regulated impact.
S0 requires closure reason.
S1 requires local remediation.
S2 requires privacy review.
S3 requires PIPA clock evaluation.
S4 requires executive and legal escalation.
S5 requires crisis response and KISA path review.
Severity can only increase automatically.
Severity downgrade requires reviewer and evidence.
Severity changes must emit audit events.
Severity band must be visible in tenant GRC dashboard.
Severity band must be visible in incident command console.

## Activated Cedar Policies

`pack-kr-pack-1-cybersecurity-incident-triage` triggers Korean incident classification.
`pack-kr-pack-1-incident-kisa-triage` triggers KISA notification path review.
`pack-kr-pack-1-pipa-breach-reporting-window` starts PIPA leakage report clock.
`pack-kr-pack-1-pii-emission-scrub` scrubs incident audit payloads.
`pack-kr-pack-1-audit-tenant-context` requires tenant context.
`pack-kr-pack-1-audit-jurisdiction-code` requires KR jurisdiction code.
`pack-kr-pack-1-cell-kr-residency` keeps incident evidence in KR cell.
`pack-kr-pack-1-csap-cell-pinning` applies to CSAP incident evidence.
`pack-kr-pack-1-rrn-collection-deny-default` blocks RRN in tickets.
`pack-kr-pack-1-rrn-hash-only` controls RRN derivative incident handling.
`pack-kr-pack-1-cross-border-transfer-deny-default` blocks non-KR evidence export.
`pack-kr-pack-1-medical-record-access-trace` applies when healthcare data is involved.
`pack-kr-pack-1-communications-secret-deny-content-inspection` applies when messages are involved.
`pack-kr-pack-1-electronic-document-evidence` applies when evidence documents are affected.
`pack-kr-pack-1-retention-legal-hold` freezes evidence and deletion.
`pack-kr-pack-1-processor-due-diligence` applies when processor caused or handled incident.
`pack-kr-pack-1-lawful-disclosure-log` records regulator, court, or emergency disclosures.
`pack-kr-pack-1-pack-precedence-deny-wins` prevents generic incident tooling override.

## Data Model Deltas

Add `incident.kr_incident_id`.
Add `incident.kr_first_detected_at`.
Add `incident.kr_classified_at`.
Add `incident.kr_containment_started_at`.
Add `incident.kr_severity_band`.
Add `incident.kr_controlled_source_criteria`.
Add `incident.kr_critical_infrastructure_flag`.
Add `incident.kr_privacy_breach_flag`.
Add `incident.kr_rrn_involved_flag`.
Add `incident.kr_sensitive_info_involved_flag`.
Add `incident.kr_medical_record_involved_flag`.
Add `incident.kr_communications_involved_flag`.
Add `incident.kr_electronic_document_involved_flag`.
Add `incident.kr_csap_workload_involved_flag`.
Add `incident.kr_affected_facility`.
Add `incident.kr_damage_summary`.
Add `incident.kr_kisa_notification_required`.
Add `incident.kr_kisa_notification_due_at`.
Add `incident.kr_kisa_notification_submitted_at`.
Add `incident.kr_kisa_notification_ref`.
Add `incident.kr_pipc_report_required`.
Add `incident.kr_pipc_report_due_at`.
Add `incident.kr_pipc_report_submitted_at`.
Add `incident.kr_pipc_report_ref`.
Add `incident.kr_subject_notification_required`.
Add `incident.kr_subject_notification_due_at`.
Add `incident.kr_subject_notification_submitted_at`.
Add `incident.kr_evidence_cell_id`.
Add `incident.kr_evidence_residency_class`.
Add `incident.kr_emergency_export_id`.
Add `incident.kr_legal_hold_state`.
Add `incident.kr_processor_involved`.
Add `incident.kr_processor_evidence_id`.
Add `incident.kr_cedar_policy_ids`.
Add `incident.kr_closure_evidence_id`.
Add `incident.kr_no_notification_basis`.
Add `incident.kr_delay_reason`.
Add `incident.kr_supplemental_report_refs`.
Add `incident.kr_after_action_review_id`.
Add `incident.kr_customer_briefing_state`.
Transform generic incident severity into KR severity band.
Transform generic incident timestamp into detection/classification/containment clock fields.
Transform generic incident type into controlled-source criteria.
Transform generic breach flag into PIPA report classification.
Transform generic evidence export into residency-controlled evidence workflow.
Transform raw evidence payload into scrubbed audit reference.
Transform notification task into KISA/PIPC/subject separated tasks.
Transform incident closure into legal-hold and evidence-retention checkpoint.

## API Contract Deltas

`POST /kr/incidents/classify` classifies a security event under KR-PACK-1.
`POST /kr/incidents/classify` requires `tenant_id`.
`POST /kr/incidents/classify` requires `first_detected_at`.
`POST /kr/incidents/classify` requires `affected_services`.
`POST /kr/incidents/classify` accepts `suspected_data_classes`.
`POST /kr/incidents/classify` returns `kr_severity_band`.
`POST /kr/incidents/classify` returns `controlled_source_criteria`.
`POST /kr/incidents/classify` returns `pipa_report_clock`.
`POST /kr/incidents/classify` returns `kisa_notification_clock`.
`POST /kr/incidents/classify` returns `cedar_policy_ids`.
`POST /kr/incidents/{incident_id}/containment/start` records containment start.
`POST /kr/incidents/{incident_id}/kisa-notification` records KISA notification.
`POST /kr/incidents/{incident_id}/kisa-notification` requires affected facility when applicable.
`POST /kr/incidents/{incident_id}/kisa-notification` requires damage summary when available.
`POST /kr/incidents/{incident_id}/kisa-notification` returns report reference.
`POST /kr/incidents/{incident_id}/pipc-notification` records PIPC report.
`POST /kr/incidents/{incident_id}/pipc-notification` returns report reference.
`POST /kr/incidents/{incident_id}/subject-notification` records data-subject notification state.
`POST /kr/incidents/{incident_id}/evidence/add` adds KR-local evidence reference.
`POST /kr/incidents/{incident_id}/evidence/export-request` requests emergency evidence export.
`POST /kr/incidents/{incident_id}/legal-hold/apply` applies legal hold.
`POST /kr/incidents/{incident_id}/legal-hold/release` releases legal hold.
`POST /kr/incidents/{incident_id}/supplemental-report` records updated notification evidence.
`POST /kr/incidents/{incident_id}/close` closes incident with evidence.
`GET /kr/incidents/{incident_id}` returns KR incident state.
Every incident API returns `jurisdiction_code=KR`.
Every state-changing incident API returns `audit_id`.
Every incident API suppresses raw PII from response bodies unless privileged evidence endpoint is explicitly authorized.
Every incident API denies raw RRN in tickets and ordinary evidence summaries.

## Audit Event Additions

`KrIncidentDetected` records first detection time and source.
`KrControlledSourceIncidentClassified` records criteria and severity.
`KrIncidentSeverityChanged` records severity change and reviewer.
`KrIncidentContainmentStarted` records containment action and owner.
`KrIncidentEvidenceAdded` records evidence reference and cell ID.
`KrIncidentEvidenceExportDenied` records missing transfer basis.
`KrEmergencyExportApproved` records time-bound evidence export.
`KrEmergencyExportClosed` records evidence return or deletion.
`KrBreachClockStarted` records privacy breach clock.
`KrRrnBreachClockStarted` records RRN breach clock.
`KrBreachKisaPrecheckStarted` records KISA precheck.
`KrBreachKisaNotified` records KISA notification reference.
`KrBreachPipcPrecheckStarted` records PIPC precheck.
`KrBreachPipcNotified` records PIPC report reference.
`KrSubjectNotificationQueued` records data-subject notification queue.
`KrSubjectNotificationSent` records notification completion.
`KrNoNotificationDecisionRecorded` records legal basis for non-notification.
`KrNotificationDelayRecorded` records delay reason.
`KrIncidentLegalHoldApplied` records hold authority and scope.
`KrIncidentLegalHoldReleased` records release authority.
`KrIncidentSupplementalReportFiled` records updated report reference.
`KrIncidentAfterActionStarted` records review start.
`KrIncidentAfterActionClosed` records corrective actions.
`KrIncidentClosed` records closure evidence.
Every event carries `tenant_id`.
Every event carries `sub_scope_path` where scoped.
Every event carries `event_id`.
Every event carries `trace_id`.
Every event carries `span_id`.
Every event carries `audit_id`.
Every event carries `schema_version`.
Every event carries `source_microservice`.
Every event carries `cell_id`.
Every event carries `jurisdiction_code=KR`.
Every event payload is PII-scrubbed.
No event payload contains raw leaked records.

## Failure Modes specific to KR enforcement

Failure mode `KR-INC-FM-001`: incident not classified against Korean controls.
Failure mode `KR-INC-FM-002`: first detection time missing.
Failure mode `KR-INC-FM-003`: classification time missing.
Failure mode `KR-INC-FM-004`: controlled-source criteria not evaluated.
Failure mode `KR-INC-FM-005`: critical infrastructure impact ignored.
Failure mode `KR-INC-FM-006`: KISA notification precheck not started.
Failure mode `KR-INC-FM-007`: KISA notification lacks affected facility.
Failure mode `KR-INC-FM-008`: KISA notification lacks damage summary.
Failure mode `KR-INC-FM-009`: KISA notification reference missing.
Failure mode `KR-INC-FM-010`: PIPA breach clock not started.
Failure mode `KR-INC-FM-011`: PIPC report reference missing.
Failure mode `KR-INC-FM-012`: subject notification state missing.
Failure mode `KR-INC-FM-013`: RRN leakage not high-severity.
Failure mode `KR-INC-FM-014`: sensitive information leakage not high-severity.
Failure mode `KR-INC-FM-015`: medical record incident not healthcare reviewed.
Failure mode `KR-INC-FM-016`: communications content incident not secrecy reviewed.
Failure mode `KR-INC-FM-017`: electronic document integrity incident not preserved.
Failure mode `KR-INC-FM-018`: CSAP incident not mapped to certified cell evidence.
Failure mode `KR-INC-FM-019`: incident evidence leaves Korea without basis.
Failure mode `KR-INC-FM-020`: incident ticket contains raw PII.
Failure mode `KR-INC-FM-021`: incident ticket contains raw RRN.
Failure mode `KR-INC-FM-022`: audit event contains raw leaked sample.
Failure mode `KR-INC-FM-023`: legal hold not applied.
Failure mode `KR-INC-FM-024`: deletion runs during investigation.
Failure mode `KR-INC-FM-025`: supplemental report not filed after material update.
Failure mode `KR-INC-FM-026`: no-notification decision lacks legal basis.
Failure mode `KR-INC-FM-027`: delayed notification lacks reason.
Failure mode `KR-INC-FM-028`: closure lacks evidence.
Failure mode `KR-INC-FM-029`: processor-caused incident lacks processor evidence.
Failure mode `KR-INC-FM-030`: Cedar policy bypass not escalated as pack-control incident.

## Worked Examples

### Scenario 1: RRN Leakage

Security monitoring detects export of a payroll table.
The classifier identifies Korean tenant context.
The classifier identifies `PI_KR_RRN`.
The classifier sets severity `KR-INC-S4`.
The PIPA breach clock starts.
The RRN breach clock starts.
KISA/PIPC prechecks start.
Containment blocks the export token.
Evidence remains in KR incident cell.
Tickets receive scrubbed fingerprints only.
The audit stream emits `KrRrnBreachClockStarted`.
The audit stream emits `KrIncidentContainmentStarted`.
Notification references are recorded when submitted.
Closure waits for legal hold review.

### Scenario 2: Critical Infrastructure Disruption

A denial-of-service attack disrupts a KR controlled service.
The classifier maps the event to `KR-CSB-009`.
The affected facility is recorded.
The damage summary is recorded.
The severity is set to `KR-INC-S5`.
KISA notification target starts.
The incident command role is assigned.
The containment task begins.
The audit stream emits `KrBreachKisaPrecheckStarted`.
The audit stream emits `KrBreachKisaNotified` after submission.
If personal data leakage is later confirmed, PIPA clock starts separately.

### Scenario 3: Medical Record Tampering

Medical service detects unauthorized modification of electronic medical record.
The classifier maps medical record impact.
The classifier maps electronic document integrity impact.
The severity is set to `KR-INC-S4`.
The Medical Service Act review task starts.
The electronic record integrity evidence is preserved.
Access-purpose logs are frozen.
The audit stream emits `KrControlledSourceIncidentClassified`.
The audit stream emits `KrIncidentLegalHoldApplied`.
No raw diagnosis text appears in routine audit events.

### Scenario 4: Communications Content Exposure

Support workflow accidentally exposes message content in a ticket.
The classifier maps Communications Secrets Protection Act impact.
The classifier maps PIPA impact if message contains personal information.
The ticket is scrubbed.
The support export channel is suspended.
The affected users and tenant are scoped.
The audit stream emits `KrIncidentDetected`.
The audit stream emits `KrIncidentContainmentStarted`.
Legal review decides notification path.
The no-notification decision, if any, requires legal basis.

### Scenario 5: Non-KR Forensics Export Request

Security engineer requests export of incident disk image to global forensics lab.
The evidence contains Korean regulated records.
The residency policy denies export by default.
The incident commander requests emergency export.
The emergency request narrows evidence to indicators and scrubbed logs.
The export receives expiration and deletion requirement.
The audit stream emits `KrEmergencyExportApproved`.
The lab returns analysis.
The export copy is deleted.
The audit stream emits `KrEmergencyExportClosed`.
If deletion proof is absent, the incident cannot close.

## Cross-References

Pack overview: `packs/kr-localization/README.md`.
Regulatory coverage: `packs/kr-localization/regulatory-coverage.md`.
Data residency: `packs/kr-localization/data-residency.md`.
Consent management: `packs/kr-localization/consent-management.md`.
RRN handling: `packs/kr-localization/resident-id-number-rrn-handling.md`.
ADR-0064 localization pack architecture: `docs/decisions/ADR-0709-general-live-apex.md`.
ADR-0244 tenant scoping: `docs/decisions/ADR-0702-identity-authz-live-apex.md`.
ADR-0251 compliance pack mechanics: `docs/decisions/ADR-0708-platform-foundations-live-apex.md`.
ADR-0263 audit event emission contract: `docs/decisions/ADR-0706-observability-live-apex.md`.
KR pack manifest seed: `docs/localization-packs/kr/pack.yaml`.
Official law source: `https://www.law.go.kr/`.
Official PIPC source: `https://www.pipc.go.kr/`.
Official KISA source: `https://www.kisa.or.kr/`.
Official CSAP source: `https://isms.kisa.or.kr/main/csap/intro/index.jsp`.

## Incident Requirement Register

`KR-INC-REQ-001` every Korean incident must record first detection time.
`KR-INC-REQ-002` every Korean incident must record classification time.
`KR-INC-REQ-003` every Korean incident must record containment start time.
`KR-INC-REQ-004` every Korean incident must record tenant ID.
`KR-INC-REQ-005` every Korean incident must record sub-scope where applicable.
`KR-INC-REQ-006` every Korean incident must evaluate PIPA impact.
`KR-INC-REQ-007` every Korean incident must evaluate KISA path.
`KR-INC-REQ-008` every Korean incident must evaluate controlled-source criteria.
`KR-INC-REQ-009` every Korean incident must evaluate CSAP impact.
`KR-INC-REQ-010` every Korean incident must evaluate medical impact.
`KR-INC-REQ-011` every Korean incident must evaluate communications impact.
`KR-INC-REQ-012` every Korean incident must evaluate electronic-document impact.
`KR-INC-REQ-013` every Korean incident must evaluate RRN impact.
`KR-INC-REQ-014` every Korean incident must evaluate sensitive-information impact.
`KR-INC-REQ-015` every Korean incident must preserve evidence in KR cell.
`KR-INC-REQ-016` every Korean incident must scrub audit payloads.
`KR-INC-REQ-017` every Korean incident must block raw PII in tickets.
`KR-INC-REQ-018` every Korean incident must block raw RRN in tickets.
`KR-INC-REQ-019` every Korean incident must block non-KR evidence export by default.
`KR-INC-REQ-020` every Korean incident must record legal hold state.
`KR-INC-REQ-021` controlled-source unauthorized access must trigger review.
`KR-INC-REQ-022` controlled-source privilege excess must trigger review.
`KR-INC-REQ-023` controlled-source data manipulation must trigger review.
`KR-INC-REQ-024` controlled-source data destruction must trigger review.
`KR-INC-REQ-025` controlled-source data concealment must trigger review.
`KR-INC-REQ-026` controlled-source data leakage must trigger review.
`KR-INC-REQ-027` controlled-source malware must trigger review.
`KR-INC-REQ-028` controlled-source logic bomb must trigger review.
`KR-INC-REQ-029` controlled-source denial-of-service must trigger review.
`KR-INC-REQ-030` controlled-source false command must trigger review.
`KR-INC-REQ-031` ransomware must trigger controlled-source review.
`KR-INC-REQ-032` identity key compromise must trigger controlled-source review.
`KR-INC-REQ-033` RRN derivative key compromise must trigger controlled-source review.
`KR-INC-REQ-034` medical integrity compromise must trigger controlled-source review.
`KR-INC-REQ-035` communications secrecy compromise must trigger controlled-source review.
`KR-INC-REQ-036` electronic evidence compromise must trigger controlled-source review.
`KR-INC-REQ-037` CSAP boundary compromise must trigger controlled-source review.
`KR-INC-REQ-038` evidence export must trigger residency review.
`KR-INC-REQ-039` processor incident must trigger processor evidence review.
`KR-INC-REQ-040` insider misuse must trigger privileged access review.
`KR-INC-REQ-041` KR-INC-S0 closure requires false-positive reason.
`KR-INC-REQ-042` KR-INC-S1 closure requires remediation.
`KR-INC-REQ-043` KR-INC-S2 requires privacy review.
`KR-INC-REQ-044` KR-INC-S3 requires PIPA clock.
`KR-INC-REQ-045` KR-INC-S4 requires legal and executive escalation.
`KR-INC-REQ-046` KR-INC-S5 requires crisis response.
`KR-INC-REQ-047` severity downgrade requires reviewer.
`KR-INC-REQ-048` severity downgrade requires evidence.
`KR-INC-REQ-049` notification not required decision requires basis.
`KR-INC-REQ-050` notification delay requires reason.
`KR-INC-REQ-051` KISA notification must record report reference.
`KR-INC-REQ-052` PIPC report must record report reference.
`KR-INC-REQ-053` subject notification must record queue and send state.
`KR-INC-REQ-054` supplemental report must record reference.
`KR-INC-REQ-055` after-action review must record corrective actions.
`KR-INC-REQ-056` incident closure must record evidence.
`KR-INC-REQ-057` incident closure must check legal hold.
`KR-INC-REQ-058` incident closure must check notification completion.
`KR-INC-REQ-059` incident closure must check evidence retention.
`KR-INC-REQ-060` incident closure must check export copy deletion.
`KR-INC-REQ-061` KISA precheck target is T0+4h.
`KR-INC-REQ-062` KISA internal notification target is T0+24h where triggered.
`KR-INC-REQ-063` PIPA report clock tracks decree-defined 72h path where triggered.
`KR-INC-REQ-064` tenant briefing target is T0+24h where appropriate.
`KR-INC-REQ-065` supplemental evidence target is T0+48h.
`KR-INC-REQ-066` posture review target is T0+72h.
`KR-INC-REQ-067` KISA notification content includes incident time.
`KR-INC-REQ-068` KISA notification content includes affected facility.
`KR-INC-REQ-069` KISA notification content includes damage details.
`KR-INC-REQ-070` KISA notification content includes response details.
`KR-INC-REQ-071` PIPA report classification checks affected subject count.
`KR-INC-REQ-072` PIPA report classification checks sensitive information.
`KR-INC-REQ-073` PIPA report classification checks unique identifying information.
`KR-INC-REQ-074` PIPA report classification checks unlawful external access.
`KR-INC-REQ-075` RRN leakage always receives high-priority privacy review.
`KR-INC-REQ-076` medical record leakage always receives healthcare review.
`KR-INC-REQ-077` communications content exposure always receives communications secrecy review.
`KR-INC-REQ-078` electronic document tampering always receives evidence integrity review.
`KR-INC-REQ-079` CSAP workload incident always receives cell-certification review.
`KR-INC-REQ-080` processor-caused incident always receives due diligence review.
`KR-INC-REQ-081` incident API must return policy IDs.
`KR-INC-REQ-082` incident API must return audit ID for state changes.
`KR-INC-REQ-083` incident API must return jurisdiction code KR.
`KR-INC-REQ-084` incident API must suppress raw leaked records.
`KR-INC-REQ-085` incident API must suppress raw RRN.
`KR-INC-REQ-086` incident API must protect evidence endpoint.
`KR-INC-REQ-087` incident dashboard must show KISA clock.
`KR-INC-REQ-088` incident dashboard must show PIPC clock.
`KR-INC-REQ-089` incident dashboard must show subject notification state.
`KR-INC-REQ-090` incident dashboard must show legal hold state.
`KR-INC-REQ-091` incident dashboard must show export state.
`KR-INC-REQ-092` incident dashboard must show affected services.
`KR-INC-REQ-093` incident dashboard must show data classes.
`KR-INC-REQ-094` incident dashboard must show severity.
`KR-INC-REQ-095` incident dashboard must show controlled-source criteria.
`KR-INC-REQ-096` incident dashboard must show closure blockers.
`KR-INC-REQ-097` incident evidence must be immutable after record.
`KR-INC-REQ-098` incident evidence must include integrity hash.
`KR-INC-REQ-099` incident evidence must include custody chain.
`KR-INC-REQ-100` incident evidence must include residency class.
`KR-INC-REQ-101` incident evidence must include access log.
`KR-INC-REQ-102` incident evidence must include disposal rule.
`KR-INC-REQ-103` emergency export must be time-bound.
`KR-INC-REQ-104` emergency export must be scoped.
`KR-INC-REQ-105` emergency export must be approved.
`KR-INC-REQ-106` emergency export must be closed.
`KR-INC-REQ-107` emergency export must record deletion proof.
`KR-INC-REQ-108` emergency export must not include raw RRN unless separately lawful and unavoidable.
`KR-INC-REQ-109` incident legal hold must block deletion.
`KR-INC-REQ-110` incident legal hold must block evidence overwrite.
`KR-INC-REQ-111` incident legal hold must record authority.
`KR-INC-REQ-112` incident legal hold must record scope.
`KR-INC-REQ-113` incident legal hold release must record authority.
`KR-INC-REQ-114` after-action review must update controls.
`KR-INC-REQ-115` after-action review must update tests.
`KR-INC-REQ-116` after-action review must update runbooks.
`KR-INC-REQ-117` after-action review must update processor records if applicable.
`KR-INC-REQ-118` after-action review must update tenant briefing if applicable.
`KR-INC-REQ-119` after-action review must update pack docs if law mapping changed.
`KR-INC-REQ-120` after-action review must preserve evidence digest.

## Notification Evidence Register

`KR-INC-EVID-001` KISA notification package must include incident occurrence time.
`KR-INC-EVID-002` KISA notification package must include affected facility or service.
`KR-INC-EVID-003` KISA notification package must include damage summary.
`KR-INC-EVID-004` KISA notification package must include containment status.
`KR-INC-EVID-005` KISA notification package must include responder contact.
`KR-INC-EVID-006` KISA notification package must include supplemental evidence plan.
`KR-INC-EVID-007` PIPC report package must include leakage discovery time.
`KR-INC-EVID-008` PIPC report package must include leaked data categories.
`KR-INC-EVID-009` PIPC report package must include affected subject scale where known.
`KR-INC-EVID-010` PIPC report package must include leakage path where known.
`KR-INC-EVID-011` PIPC report package must include mitigation actions.
`KR-INC-EVID-012` PIPC report package must include subject notification plan.
`KR-INC-EVID-013` subject notification package must include incident summary.
`KR-INC-EVID-014` subject notification package must include affected item categories.
`KR-INC-EVID-015` subject notification package must include protective action guidance.
`KR-INC-EVID-016` subject notification package must include help channel.
`KR-INC-EVID-017` subject notification package must include date of notification.
`KR-INC-EVID-018` no-notification decision must include legal authority.
`KR-INC-EVID-019` no-notification decision must include reviewer identity.
`KR-INC-EVID-020` no-notification decision must include affected criteria.
`KR-INC-EVID-021` delay decision must include unavoidable circumstance description.
`KR-INC-EVID-022` delay decision must include delayed-until timestamp.
`KR-INC-EVID-023` delay decision must include interim containment evidence.
`KR-INC-EVID-024` supplemental report must include changed facts.
`KR-INC-EVID-025` supplemental report must include prior report reference.
`KR-INC-EVID-026` supplemental report must include new evidence digest.
`KR-INC-EVID-027` controlled-source evidence must include criterion code.
`KR-INC-EVID-028` controlled-source evidence must include impacted data class.
`KR-INC-EVID-029` controlled-source evidence must include affected cell.
`KR-INC-EVID-030` controlled-source evidence must include custody chain.
`KR-INC-EVID-031` RRN evidence must include scrubbed derivative fingerprint only.
`KR-INC-EVID-032` medical evidence must exclude diagnosis text from routine package.
`KR-INC-EVID-033` communications evidence must exclude message content from routine package.
`KR-INC-EVID-034` electronic document evidence must include integrity hash.
`KR-INC-EVID-035` CSAP evidence must include certification digest.
`KR-INC-EVID-036` processor evidence must include processor role and country.
`KR-INC-EVID-037` emergency export evidence must include approval ID.
`KR-INC-EVID-038` emergency export evidence must include expiration.
`KR-INC-EVID-039` emergency export evidence must include closure proof.
`KR-INC-EVID-040` legal hold evidence must include hold authority.
`KR-INC-EVID-041` closure evidence must include notification completion state.
`KR-INC-EVID-042` closure evidence must include corrective action state.
`KR-INC-EVID-043` closure evidence must include evidence retention state.
`KR-INC-EVID-044` closure evidence must include open risk acceptance if any.
`KR-INC-EVID-045` audit evidence must include ADR-0263 envelope validation.
`KR-INC-EVID-046` audit evidence must include PII scrub validation.
`KR-INC-EVID-047` audit evidence must include tenant and sub-scope validation.
`KR-INC-EVID-048` audit evidence must include jurisdiction code validation.
`KR-INC-EVID-049` audit evidence must include policy ID validation.
`KR-INC-EVID-050` audit evidence must include immutable event ID.
`KR-INC-EVID-051` tenant briefing evidence must include approved message version.
`KR-INC-EVID-052` tenant briefing evidence must include recipients.
`KR-INC-EVID-053` tenant briefing evidence must include send timestamp.
`KR-INC-EVID-054` tenant briefing evidence must include known limitations.
`KR-INC-EVID-055` after-action evidence must include root-cause class.
`KR-INC-EVID-056` after-action evidence must include preventive control.
`KR-INC-EVID-057` after-action evidence must include detective control.
`KR-INC-EVID-058` after-action evidence must include runbook update.
`KR-INC-EVID-059` after-action evidence must include test update.
`KR-INC-EVID-060` after-action evidence must include owner and due date.

## Checkpoint

This file is scoped to `/packs/kr-localization/`.
It does not edit ADRs.
It does not edit microservices.
It does not edit other packs.
It must be line-count verified with the rest of KR-PACK-1.
It must be lifecycle-verified with retired VCS ratchet after all six docs exist.
