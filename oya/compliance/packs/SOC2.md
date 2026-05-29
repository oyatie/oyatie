---
doc_class: CompliancePackOverlay
pack_id: SOC2-T2
microservice: compliance
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# compliance SOC 2 Compliance Pack Overlay

## Pack Identity
- Full pack name: SOC 2 Type II Trust Services Criteria compliance control overlay.
- Citing jurisdiction: AICPA attestation framework for service organizations.
- Version: SOC2-T2-2017-TSC-2022-POF-v1.
- Canonical source URL: https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022
- Cited framework: 2017 Trust Services Criteria with Revised Points of Focus 2022.
- Covered compliance surface: control catalog, evidence requests, auditor exports, exception tracking, management assertions, access reviews, and control-period manifests.
- Pack activation means compliance becomes the system of record for SOC 2 control evidence across microservices.
- The overlay maps service evidence to selected Trust Services Criteria without storing raw customer data.
- Data classes include `COMPLIANCE_SOC2_CONTROL`, `COMPLIANCE_SOC2_EVIDENCE`, `COMPLIANCE_SOC2_EXCEPTION`, and `COMPLIANCE_SOC2_AUDITOR_EXPORT`.
- Type II evidence must be time-windowed and immutable.
- ADR-0064 keeps SOC 2 as an overlay.
- ADR-0251 supplies pack schema and evidence retention.
- ADR-0263 supplies control evidence emission linkage.
- This overlay excludes PCI-DSS because SOC 2 control evidence is not a payment-data flow.
- If PCI evidence is cross-referenced, only hashes and control ids are stored here.

## Data Model Deltas
- Add `control.control_id`.
- Add `control.trust_service_category`.
- Add `control.common_criteria_ref`.
- Add `control.owner_team`.
- Add `control.frequency`.
- Add `control.population_query_ref`.
- Add `control.sample_selection_seed`.
- Add `control_period.period_id`.
- Add `control_period.start_at`.
- Add `control_period.end_at`.
- Add `control_period.management_assertion_hash`.
- Add `evidence_item.evidence_hash`.
- Add `evidence_item.source_microservice`.
- Add `evidence_item.collection_method`.
- Add `evidence_item.collected_at`.
- Add `evidence_item.redaction_profile`.
- Add `exception.exception_id`.
- Add `exception.criteria_ref`.
- Add `exception.mitigation_status`.
- Add `exception.risk_acceptance_ref`.
- Add `auditor_export.manifest_hash`.
- Add `audit_shadow.compliance_soc2_event_id`.
- Add `tenant_compliance_config.soc2_audit_scope`.
- Add `tenant_compliance_config.auditor_access_window`.

## Cedar Policy Deltas
- Policy `SOC2-compliance-admission-01`: require audit scope before SOC 2 pack activation.
- Policy `SOC2-compliance-control-01`: require owner team for every control.
- Policy `SOC2-compliance-control-02`: require frequency for every active control.
- Policy `SOC2-compliance-period-01`: forbid evidence mutation after period lock.
- Policy `SOC2-compliance-period-02`: require management assertion before period close.
- Policy `SOC2-compliance-evidence-01`: require evidence hash and source microservice.
- Policy `SOC2-compliance-evidence-02`: forbid raw customer data in evidence payload.
- Policy `SOC2-compliance-sample-01`: require sample seed before auditor sampling.
- Policy `SOC2-compliance-exception-01`: forbid exception closure without mitigation or acceptance.
- Policy `SOC2-compliance-exception-02`: require owner assignment for exceptions.
- Policy `SOC2-compliance-export-01`: require redaction profile for auditor export.
- Policy `SOC2-compliance-export-02`: restrict auditor export to access window.
- Policy `SOC2-compliance-access-01`: require MFA for auditor and compliance admin access.
- Policy `SOC2-compliance-review-01`: require quarterly access review evidence.
- Policy `SOC2-compliance-vendor-01`: require vendor evidence before external control reliance.
- Policy `SOC2-compliance-change-01`: require change ticket for control definition changes.
- Policy `SOC2-compliance-retention-01`: forbid evidence purge before audit retention floor.
- Policy `SOC2-compliance-dashboard-01`: restrict control dashboard by audit scope.
- Policy `SOC2-compliance-support-01`: require support case for evidence inspection.
- Policy `SOC2-compliance-audit-01`: require audit seal for every control-period transition.
- Policy `SOC2-compliance-pack-01`: forbid deactivation during active audit period.
- Policy `SOC2-compliance-recollect-01`: require exception link for evidence recollection.
- Policy `SOC2-compliance-assertion-01`: require executive approval for management assertion.
- Policy `SOC2-compliance-crossref-01`: permit cross-pack evidence only as hash and control id.

## API Contract Deltas
- `POST /packs/SOC2-T2/admit` requires audit scope.
- `POST /controls` requires owner team and frequency.
- `PATCH /controls/{id}` requires change ticket.
- `POST /control-periods` creates immutable evidence window.
- `PATCH /control-periods/{id}/lock` freezes evidence mutation.
- `POST /management-assertions` requires executive approval.
- `POST /evidence` requires source microservice and evidence hash.
- `POST /evidence` rejects raw customer data markers.
- `POST /samples` requires sample seed.
- `POST /exceptions` requires owner and criterion ref.
- `PATCH /exceptions/{id}/close` requires mitigation or acceptance.
- `POST /auditor/exports` requires redaction profile.
- `GET /auditor/exports/{id}` enforces access window.
- `POST /access-reviews` stores quarterly review evidence.
- `POST /vendor-evidence` records external control reliance.
- `GET /dashboards/controls` is audit-scope filtered.
- `POST /support/evidence-view` requires support case id.
- `DELETE /evidence/{id}` returns retention conflict.
- `POST /recollect` requires exception link.
- `POST /pack/deactivate` refuses active audit period.

## Workflow Deltas
- Pack admission workflow records SOC 2 audit scope.
- Control authoring workflow requires owner, frequency, and criterion mapping.
- Control change workflow requires ticket and approval.
- Control-period workflow locks evidence window.
- Evidence collection workflow stores hashes from source services.
- Evidence validation workflow rejects raw customer data.
- Sample workflow freezes deterministic sample seed.
- Exception workflow tracks owner, mitigation, and acceptance.
- Recollection workflow links evidence retry to exception.
- Management assertion workflow requires executive approval.
- Auditor export workflow applies redaction profile and access window.
- Access review workflow collects quarterly user review evidence.
- Vendor evidence workflow records external reliance proof.
- Dashboard workflow filters by audit scope.
- Retention workflow blocks evidence purge.
- Support view workflow records case id.
- Cross-pack evidence workflow stores only hash and control id.
- Active audit period workflow blocks pack deactivation.
- Audit bundle workflow seals every period transition.
- Deadline workflow pages owners for missing evidence.

## SLO Deltas
- Evidence collection from source services p99 target is <= 15 minutes.
- Evidence validation p99 must stay <= 2 seconds.
- Control-period lock p99 target is <= 5 minutes.
- Sample seed publication target is <= 1 hour after period close.
- Exception creation p99 must complete <= 2 minutes.
- Exception owner assignment target is <= 1 business day.
- Auditor export manifest p99 target is <= 30 minutes.
- Access review completion cadence is quarterly.
- Vendor evidence refresh cadence is monthly or per contract.
- Dashboard evidence freshness target is <= 15 minutes.
- Management assertion approval target is before period close.
- Evidence purge refusal p99 must stay <= 300 ms.
- Support evidence view audit p99 must complete <= 1 second.
- Control change propagation p99 target is <= 5 minutes.
- Missing evidence alert fires within 15 minutes of due time.
- SOC 2 compliance dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `ComplianceSoc2PackAdmissionStarted` records tenant id and scope.
- `ComplianceSoc2PackAdmissionApproved` records audit scope.
- `ComplianceSoc2ControlCreated` records criterion ref.
- `ComplianceSoc2ControlChanged` records change ticket.
- `ComplianceSoc2ControlPeriodCreated` records period id.
- `ComplianceSoc2ControlPeriodLocked` records period id.
- `ComplianceSoc2ManagementAssertionApproved` records assertion hash.
- `ComplianceSoc2EvidenceAccepted` records source microservice.
- `ComplianceSoc2EvidenceRejectedRawData` records detector verdict.
- `ComplianceSoc2SampleSeedFrozen` records seed hash.
- `ComplianceSoc2ExceptionOpened` records criterion ref.
- `ComplianceSoc2ExceptionClosed` records mitigation.
- `ComplianceSoc2EvidenceRecollected` records exception id.
- `ComplianceSoc2AuditorExportCreated` records manifest hash.
- `ComplianceSoc2AccessReviewStored` records cycle id.
- `ComplianceSoc2VendorEvidenceStored` records vendor id.
- `ComplianceSoc2SupportEvidenceViewed` records case id.
- `ComplianceSoc2RetentionPurgeRefused` records retention floor.
- `ComplianceSoc2MissingEvidenceAlerted` records control id.
- `ComplianceSoc2PackDeactivationDeferred` records active period.

## Failure Modes specific to this pack
- Audit scope missing; recovery is reject pack admission.
- Control owner missing; recovery is reject control creation.
- Evidence contains raw customer data; recovery is reject and quarantine.
- Evidence hash mismatch appears; recovery is recollect from source service.
- Period lock attempted with missing evidence; recovery is block lock and alert owner.
- Sample seed missing; recovery is block auditor sample.
- Exception has no owner; recovery is auto-assign to control owner.
- Exception closure lacks mitigation; recovery is keep exception open.
- Management assertion missing; recovery is block period close.
- Auditor export outside access window; recovery is deny export.
- Redaction profile missing; recovery is reject export.
- Vendor evidence expires; recovery is mark control reliance degraded.
- Access review overdue; recovery is block new privileged grants.
- Support view lacks case; recovery is deny view.
- Control change lacks ticket; recovery is rollback definition.
- Pack deactivation requested mid-period; recovery is defer.
- Dashboard scope too broad; recovery is restrict by audit scope.
- Cross-pack evidence includes payload; recovery is reject and store hash only.
- Audit-chain backpressure appears; recovery is fail-closed for period transitions.
- Deadline monitor lags; recovery is page compliance owner.

## Cross-µservice coordination
- `tenancy` enforces SOC 2 pack activation and audit-period scope.
- `identity` provides auditor, executive, and compliance admin roles.
- `audit-chain` seals control, evidence, exception, and period events.
- `observability` provides SLO evidence and dashboard freshness.
- `policy-engine` loads all `SOC2-compliance-*` fragments.
- `workflow-engine` runs evidence, exception, access review, and export workflows.
- `mail` returns SOC 2 evidence hashes for mail controls.
- `drive` returns SOC 2 evidence hashes for drive controls.
- `calendar` returns SOC 2 evidence hashes for calendar controls.
- `incident-response` returns incident evidence hashes.
- `admin-console` renders SOC 2 dashboards.
- `legal` reviews auditor access and representation letters.
- `support` uses case-bound evidence views.
- `data-warehouse` receives aggregate control metrics only.
- `notification` routes owner alerts.
- `vendor-management` supplies external control evidence.
- `release-engine` supplies change evidence.
- `security` supplies access-review and vulnerability evidence.
- `storage` provides evidence retention proof.
- `pack-registry` signs this SOC 2 compliance overlay.
