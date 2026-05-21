---
doc_class: CompliancePackOverlay
pack_id: SOC2-T2
microservice: observability
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# observability SOC 2 Compliance Pack Overlay

## Pack Identity
- Full pack name: SOC 2 Type II Trust Services Criteria observability control overlay.
- Citing jurisdiction: AICPA attestation framework for service organizations.
- Version: SOC2-T2-2017-TSC-2022-POF-v1.
- Canonical source URL: https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022
- Cited framework: 2017 Trust Services Criteria with Revised Points of Focus 2022.
- Covered observability surface: SLO evidence, alerting, log retention, trace completeness, dashboard access, on-call evidence, incident linkage, and control exports.
- Pack activation means observability is both a governed service and the evidence source for other SOC 2 overlays.
- The service must prove Security, Availability, Confidentiality, Processing Integrity, and Privacy evidence without leaking tenant data.
- Data classes include `OBSERVABILITY_SOC2_EVIDENCE`, `OBSERVABILITY_CONTROL_EXCEPTION`, and `OBSERVABILITY_ACCESS_REVIEW_RECORD`.
- Type II evidence must be retained with sampling seeds and control-period manifests.
- ADR-0064 keeps SOC 2 behavior in an overlay.
- ADR-0251 supplies pack signature and evidence retention.
- ADR-0263 supplies the core emission contract being audited.
- Raw logs and traces are not auditor-exported by default.
- This overlay excludes PCI-DSS because observability is outside card authorization scope unless payment telemetry is misrouted.

## Data Model Deltas
- Add `observability_control.control_period_id`.
- Add `observability_control.trust_service_categories`.
- Add `observability_control.cc_mapping`.
- Add `observability_control.owner_team`.
- Add `observability_control.test_frequency`.
- Add `observability_control.last_tested_at`.
- Add `observability_control.exception_state`.
- Add `observability_control.exception_ticket_id`.
- Add `observability_control.evidence_hash`.
- Add `observability_control.sample_selection_seed`.
- Add `slo.evidence_window_id`.
- Add `slo.burn_rate_control_verdict_id`.
- Add `alert.route_review_cycle_id`.
- Add `dashboard.access_review_cycle_id`.
- Add `trace.completeness_sample_id`.
- Add `log.retention_control_state`.
- Add `metric.cardinality_control_verdict_id`.
- Add `oncall.escalation_evidence_id`.
- Add `incident.evidence_link_hash`.
- Add `storage_route.config_change_id`.
- Add `admin_action.approval_chain_hash`.
- Add `export_job.auditor_redaction_profile`.
- Add `tenant_observability_config.soc2_audit_scope`.
- Add `tenant_observability_config.control_period_lock`.

## Cedar Policy Deltas
- Policy `SOC2-observability-admin-01`: require approved case for privileged telemetry access.
- Policy `SOC2-observability-admin-02`: forbid admin query when access review is overdue.
- Policy `SOC2-observability-dashboard-01`: require periodic dashboard access review.
- Policy `SOC2-observability-alert-01`: require alert route review for scoped services.
- Policy `SOC2-observability-export-01`: require redaction profile for auditor export.
- Policy `SOC2-observability-export-02`: forbid raw telemetry export unless tenant approves.
- Policy `SOC2-observability-control-01`: require evidence hash for control test completion.
- Policy `SOC2-observability-control-02`: forbid exception closure without mitigation.
- Policy `SOC2-observability-change-01`: require change ticket for storage route change.
- Policy `SOC2-observability-slo-01`: require SLO evidence window for availability controls.
- Policy `SOC2-observability-trace-01`: require trace completeness sample for processing integrity.
- Policy `SOC2-observability-log-01`: require retention control state for log storage.
- Policy `SOC2-observability-cardinality-01`: require cardinality control verdict.
- Policy `SOC2-observability-oncall-01`: require escalation evidence for paging policy.
- Policy `SOC2-observability-incident-01`: require incident linkage for alert breach.
- Policy `SOC2-observability-vendor-01`: require backend vendor evidence if external.
- Policy `SOC2-observability-sample-01`: permit auditor sample only through redacted view.
- Policy `SOC2-observability-retention-01`: forbid retention change without approval.
- Policy `SOC2-observability-access-01`: require unique principal id for telemetry operations.
- Policy `SOC2-observability-session-01`: require MFA for admin observability changes.
- Policy `SOC2-observability-monitoring-01`: require meta-monitoring for observability itself.
- Policy `SOC2-observability-replay-01`: require replay evidence for backfill.
- Policy `SOC2-observability-pack-01`: forbid pack deactivation during audit period.
- Policy `SOC2-observability-evidence-01`: require signed control-period manifest.

## API Contract Deltas
- `POST /admin/query` requires support case or change ticket.
- `POST /dashboards/{id}/access-review` records review decision.
- `POST /alerts/routes/{id}/review` records route review decision.
- `POST /auditor/exports` requires redaction profile.
- `GET /auditor/exports/{id}` returns evidence hash and sample seed.
- `POST /controls/tests` records selected TSC category.
- `PATCH /controls/exceptions/{id}` requires mitigation or acceptance.
- `POST /storage-routes` requires change ticket.
- `POST /slo/evidence-windows` records availability evidence.
- `POST /trace/completeness-samples` stores trace sample id.
- `POST /logs/retention-tests` records retention control state.
- `POST /metrics/cardinality-tests` stores verdict.
- `POST /oncall/escalation-evidence` stores paging evidence.
- `POST /incidents/{id}/observability-evidence` links evidence hash.
- `GET /vendor/backend/evidence` returns backend evidence.
- `GET /admin/actions` requires MFA-authenticated caller.
- `PATCH /tenant-observability-config` records SOC 2 audit scope.
- `POST /pack/deactivate` refuses active audit window.
- `GET /meta-monitoring/evidence` returns self-monitoring proof.
- `POST /evidence/manifests` signs control-period manifest.

## Workflow Deltas
- Quarterly access review enumerates dashboard and admin access.
- Privileged telemetry query creates support-case evidence.
- Alert route review verifies paging and escalation paths.
- Storage route change workflow requires approval and rollback proof.
- SLO evidence workflow locks burn-rate windows.
- Trace completeness workflow samples representative traces.
- Log retention workflow proves retention and deletion controls.
- Cardinality control workflow tests label budgets.
- On-call escalation workflow records paging evidence.
- Incident linkage workflow preserves alert and trace evidence hash.
- Backend vendor review refreshes evidence.
- Retention-rule change workflow requires owner approval.
- Auditor export workflow defaults to redacted telemetry metadata.
- Control exception workflow tracks mitigation and acceptance.
- Meta-monitoring workflow proves observability monitors itself.
- Replay workflow records backfill evidence.
- Common Criteria review checks unique user and MFA evidence.
- Audit period close freezes sample seed.
- Pack deactivation waits for audit-period close.
- Evidence bundle publication signs manifest into audit-chain.

## SLO Deltas
- Privileged telemetry query audit p99 must complete <= 1 second.
- Access review evidence freshness target is <= 24 hours.
- Alert route review evidence freshness target is <= 24 hours.
- SLO evidence window publication target is <= 15 minutes.
- Trace completeness sample publication target is <= 1 hour.
- Log retention test publication target is <= 24 hours.
- Cardinality control verdict p99 target is <= 10 minutes.
- On-call escalation evidence p99 target is <= 5 minutes.
- Auditor redacted export p99 target is <= 4 hours.
- Control exception creation p99 must complete <= 2 minutes.
- Backend vendor evidence refresh cadence is monthly.
- Access review cadence is quarterly.
- Sample seed publication target is <= 1 hour after period close.
- Incident evidence linkage p99 must complete <= 10 minutes.
- Meta-monitoring dashboard lag target is <= 5 minutes.
- SOC 2 observability dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `ObservabilitySoc2PrivilegedQueryRequested` records case id.
- `ObservabilitySoc2PrivilegedQueryGranted` records MFA and TTL.
- `ObservabilitySoc2AccessReviewStarted` records cycle id.
- `ObservabilitySoc2AccessReviewCompleted` records exceptions count.
- `ObservabilitySoc2AlertRouteReviewed` records route id.
- `ObservabilitySoc2StorageRouteChanged` records change ticket.
- `ObservabilitySoc2SloEvidenceWindowLocked` records SLO id.
- `ObservabilitySoc2TraceCompletenessSampled` records sample id.
- `ObservabilitySoc2LogRetentionTested` records backend.
- `ObservabilitySoc2CardinalityVerdictStored` records metric family.
- `ObservabilitySoc2OncallEvidenceStored` records escalation id.
- `ObservabilitySoc2AuditorExportCreated` records redaction profile.
- `ObservabilitySoc2ControlExceptionOpened` records criterion id.
- `ObservabilitySoc2ControlExceptionClosed` records mitigation.
- `ObservabilitySoc2IncidentEvidenceLinked` records incident id.
- `ObservabilitySoc2VendorBackendReviewed` records provider id.
- `ObservabilitySoc2SampleSeedFrozen` records audit period.
- `ObservabilitySoc2EvidenceBundleSigned` records bundle hash.
- `ObservabilitySoc2PackDeactivationDeferred` records audit period.
- `ObservabilitySoc2MetaMonitoringBreachRecorded` records SLO id.

## Failure Modes specific to this pack
- Auditor export includes raw telemetry; recovery is revoke and regenerate redacted bundle.
- Access review overdue; recovery is freeze new dashboard grants.
- Admin query lacks case id; recovery is terminate session and open exception.
- Alert route review missing; recovery is disable route for scoped service.
- Storage route changed without ticket; recovery is rollback route.
- SLO evidence window missing; recovery is mark control degraded.
- Trace completeness sample fails; recovery is open processing-integrity exception.
- Log retention test fails; recovery is open retention exception.
- Cardinality verdict missing; recovery is block metric schema promotion.
- On-call evidence missing; recovery is page SRE lead.
- Backend vendor evidence expires; recovery is disable provider for scoped tenants.
- Control exception has no owner; recovery is assign observability control owner.
- Evidence hash mismatch appears; recovery is rebuild from audit-chain.
- Sample seed changes after freeze; recovery is void sample.
- Pack deactivation requested mid-period; recovery is defer.
- MFA status missing for admin action; recovery is deny.
- Meta-monitoring outage occurs; recovery is open availability exception.
- Incident evidence references deleted trace; recovery is use audit-chain tombstone.
- Retention change approval expires; recovery is keep prior rule.
- Replay evidence missing; recovery is halt backfill.

## Cross-µservice coordination
- `tenancy` provides tenant pack roster and audit-period scope.
- `identity` provides unique principal, MFA, and access-review subjects.
- `compliance` owns SOC 2 control catalog, exceptions, and auditor requests.
- `audit-chain` signs evidence hashes and period manifests.
- `policy-engine` loads all `SOC2-observability-*` fragments.
- `workflow-engine` runs access review, exception, and evidence workflows.
- `incident-response` provides incident ids for linked evidence.
- `admin-console` renders scoped evidence without raw telemetry.
- `storage` provides backend retention evidence.
- `support` supplies approved case ids for privileged queries.
- `data-warehouse` receives aggregate control metrics.
- `legal` defines auditor redaction profiles.
- `notification` routes review reminders.
- `vendor-management` supplies backend provider evidence.
- `release-engine` records observability service change evidence.
- `mail` consumes SOC 2 observability evidence for mail controls.
- `drive` consumes SOC 2 observability evidence for drive controls.
- `calendar` consumes SOC 2 observability evidence for calendar controls.
- `security` consumes cardinality and trace completeness exceptions.
- `pack-registry` signs this SOC 2 observability overlay.
