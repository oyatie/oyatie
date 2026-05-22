---
doc_class: CompliancePackOverlay
pack_id: EU-AI-ACT-2024-HIGH-RISK
microservice: observability
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# observability EU AI Act Compliance Pack Overlay

## Pack Identity
- Full pack name: EU AI Act observability AI-monitoring overlay.
- Citing jurisdiction: European Union harmonised AI regulation.
- Version: EU-AI-ACT-2024-HIGH-RISK-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- Cited law: Regulation (EU) 2024/1689.
- Covered observability surface: model-touch traces, AI SLOs, drift detectors, incident telemetry, human-review dashboards, model-card evidence, risk logs, and anomaly explanations.
- Pack activation means observability records evidence for AI system monitoring without storing raw prompts or personal data.
- The service itself may use AI for anomaly explanation only through governed model-touch paths.
- Data classes include `OBSERVABILITY_AI_INPUT_EU`, `OBSERVABILITY_AI_OUTPUT_EU`, `OBSERVABILITY_AI_RISK_LOG`, and `OBSERVABILITY_AI_HUMAN_REVIEW`.
- Human oversight evidence is monitored for high-risk AI systems.
- ADR-0064 keeps AI Act monitoring in an overlay.
- ADR-0251 supplies high-risk cell and provider BYOK constraints.
- ADR-0263 supplies mandatory trace/log/metric correlation for model evidence.
- PCI-DSS is omitted because observability does not authorize payments.
- Payment AI telemetry is tokenized and separately governed by payments packs.

## Data Model Deltas
- Add `ai_monitor.model_registry_id`.
- Add `ai_monitor.risk_tier` as enum `minimal|limited|high|prohibited`.
- Add `ai_monitor.annex_iii_context`.
- Add `ai_monitor.human_review_required`.
- Add `ai_monitor.human_review_completed_at`.
- Add `ai_monitor.provider_credential_mode_snapshot`.
- Add `ai_monitor.input_redaction_profile_id`.
- Add `ai_monitor.output_label_required`.
- Add `ai_monitor.transparency_notice_id`.
- Add `ai_monitor.fundamental_rights_assessment_id`.
- Add `ai_monitor.model_card_ref`.
- Add `ai_monitor.drift_detector_version`.
- Add `ai_monitor.threshold_change_id`.
- Add `ai_monitor.rollback_plan_id`.
- Add `ai_monitor.harmful_output_candidate_id`.
- Add `trace.model_touch_trace_ref`.
- Add `metric.ai_slo_id`.
- Add `log.ai_risk_log_hash`.
- Add `dashboard.human_review_queue_scope`.
- Add `alert.ai_incident_route_id`.
- Add `export_job.eu_ai_act_manifest_hash`.
- Add `audit_shadow.eu_ai_act_event_id`.
- Add `tenant_observability_config.eu_ai_act_monitoring_enabled`.
- Add `tenant_observability_config.high_risk_contexts`.

## Cedar Policy Deltas
- Policy `EUAI-observability-ai-01`: forbid AI anomaly explanation when risk tier is prohibited.
- Policy `EUAI-observability-ai-02`: require model registry id for anomaly explanation.
- Policy `EUAI-observability-ai-03`: require input redaction profile before AI explanation.
- Policy `EUAI-observability-ai-04`: forbid provider platform-default when BYOK required.
- Policy `EUAI-observability-monitor-01`: require model-touch trace for high-risk AI systems.
- Policy `EUAI-observability-monitor-02`: require human-review dashboard for high-risk systems.
- Policy `EUAI-observability-drift-01`: require drift detector version for model SLO.
- Policy `EUAI-observability-threshold-01`: forbid silent threshold change.
- Policy `EUAI-observability-threshold-02`: require rollback plan for detector threshold change.
- Policy `EUAI-observability-incident-01`: create incident on harmful AI output candidate.
- Policy `EUAI-observability-export-01`: permit AI evidence export only through compliance.
- Policy `EUAI-observability-export-02`: require manifest hash before export release.
- Policy `EUAI-observability-dashboard-01`: restrict human-review dashboard to approved roles.
- Policy `EUAI-observability-log-01`: forbid raw prompt storage in risk logs.
- Policy `EUAI-observability-metric-01`: require AI SLO id for model health metric.
- Policy `EUAI-observability-route-01`: require EU cell for high-risk AI telemetry.
- Policy `EUAI-observability-card-01`: require model card ref before monitor activation.
- Policy `EUAI-observability-assessment-01`: require fundamental-rights assessment for high-risk context.
- Policy `EUAI-observability-appeal-01`: require appeal metric for AI blocking controls.
- Policy `EUAI-observability-retention-01`: require AI evidence retention floor.
- Policy `EUAI-observability-admin-01`: require audit seal for monitor config changes.
- Policy `EUAI-observability-replay-01`: require redaction before AI telemetry replay.
- Policy `EUAI-observability-pack-01`: defer deactivation while AI evidence is retained.
- Policy `EUAI-observability-sample-01`: require sampling to retain high-risk AI traces.

## API Contract Deltas
- `POST /ai/anomaly-explain` requires model registry id.
- `POST /ai/anomaly-explain` requires input redaction profile id.
- `POST /ai/anomaly-explain` rejects prohibited risk tier.
- `POST /ai-monitors` requires model card ref.
- `POST /ai-monitors` requires fundamental-rights assessment for high-risk context.
- `PATCH /ai-monitors/{id}/thresholds` requires rollback plan id.
- `POST /drift-detectors` records detector version.
- `POST /model-touch-traces` records trace ref for high-risk AI systems.
- `POST /human-review-dashboards` requires approved role scope.
- `POST /ai-slos` requires model registry id.
- `POST /ai-incidents` records harmful output candidate.
- `POST /exports/ai-evidence` requires compliance approval.
- `GET /exports/ai-evidence/{id}` returns manifest hash.
- `POST /replay/ai-telemetry` requires redaction option.
- `GET /risk-logs/{id}` returns prompt-free risk log hash.
- `PATCH /tenant-observability-config` records monitoring enablement.
- `GET /model-cards/{id}` returns model card reference.
- `GET /audit/ai-monitor-changes` returns config change events.
- `POST /pack/deactivate` waits for AI evidence retention.
- `GET /dashboards/human-review` requires approved role.

## Workflow Deltas
- AI monitor activation resolves model registry and risk tier.
- High-risk monitor workflow verifies fundamental-rights assessment.
- Model-touch trace workflow stores scrubbed trace reference.
- Human-review dashboard workflow checks reviewer role scope.
- Drift detector workflow records version and evaluation.
- Threshold change workflow requires rollback approval.
- AI SLO workflow binds model id to metric family.
- Harmful output workflow creates AI incident candidate.
- Anomaly explanation workflow redacts inputs before model call.
- Risk log workflow stores hashes, not raw prompts.
- Evidence export workflow builds AI Act manifest.
- Replay workflow redacts AI telemetry before backfill.
- Monitor config change workflow requires admin audit seal.
- BYOK provider workflow completes before high-risk anomaly explanation.
- EU cell workflow blocks high-risk telemetry outside EU cell.
- Appeal metric workflow checks AI blocking controls.
- Sampling workflow retains high-risk AI traces.
- Pack activation scans existing AI dashboards.
- Pack deactivation waits for retained evidence.
- Evidence bundle publication signs manifest into audit-chain.

## SLO Deltas
- AI monitor activation p99 target is <= 10 minutes.
- Model-touch trace ingestion p99 must stay <= 500 ms.
- Human-review dashboard lag target is <= 5 minutes.
- Drift detector evaluation lag target is <= 15 minutes.
- Threshold rollback activation target is <= 30 minutes.
- AI incident candidate creation p99 must complete <= 5 minutes.
- Anomaly explanation redaction p99 must stay <= 500 ms.
- AI evidence export p99 target is <= 4 hours.
- Risk log hash publication p99 must stay <= 1 second.
- High-risk route validation p99 must stay <= 100 ms.
- BYOK provider admission check p99 must stay <= 200 ms.
- AI SLO metric availability target is 99.9 percent.
- Appeal metric dashboard lag target is <= 10 minutes.
- Monitor config audit seal p99 must complete <= 1 second.
- Sampling policy refresh target is <= 5 minutes.
- EU AI Act dashboard lag target is <= 10 minutes.

## Audit-event class additions
- `ObservabilityEuAiMonitorActivated` records model id and tier.
- `ObservabilityEuAiRiskTierResolved` records context.
- `ObservabilityEuAiModelTouchTraceStored` records trace ref.
- `ObservabilityEuAiHumanReviewDashboardCreated` records scope.
- `ObservabilityEuAiDriftDetectorVersioned` records detector version.
- `ObservabilityEuAiThresholdChanged` records rollback plan.
- `ObservabilityEuAiSloBound` records SLO id.
- `ObservabilityEuAiHarmfulOutputCandidateCreated` records candidate id.
- `ObservabilityEuAiAnomalyExplanationRequested` records model id.
- `ObservabilityEuAiRiskLogHashed` records hash.
- `ObservabilityEuAiEvidenceManifestCreated` records manifest hash.
- `ObservabilityEuAiReplayRedacted` records replay id.
- `ObservabilityEuAiMonitorConfigChanged` records change id.
- `ObservabilityEuAiProviderByokChecked` records mode.
- `ObservabilityEuAiHighRiskRouteBlocked` records cell id.
- `ObservabilityEuAiAppealMetricChecked` records control id.
- `ObservabilityEuAiSamplingPolicyRefreshed` records policy id.
- `ObservabilityEuAiModelCardChecked` records card ref.
- `ObservabilityEuAiAssessmentChecked` records assessment id.
- `ObservabilityEuAiPackDeactivationDeferred` records retained evidence count.

## Failure Modes specific to this pack
- Model registry unavailable; recovery is disable AI monitor activation.
- Fundamental-rights assessment missing; recovery is block high-risk monitor.
- Model-touch trace missing; recovery is mark AI evidence incomplete.
- Human-review dashboard stale; recovery is page AI governance owner.
- Drift detector fails; recovery is mark model SLO degraded.
- Threshold changed silently; recovery is rollback and open incident.
- AI incident candidate lacks trace; recovery is reconstruct from audit-chain.
- Anomaly explanation stores raw prompt; recovery is purge log and disable feature.
- Evidence export includes raw prompt; recovery is revoke and rebuild redacted export.
- Risk log hash mismatch appears; recovery is rebuild from source events.
- Replay bypasses redaction; recovery is halt replay.
- BYOK provider mode missing; recovery is block high-risk anomaly explanation.
- EU cell unavailable; recovery is buffer or reject high-risk AI telemetry.
- Appeal metric absent; recovery is mark control incomplete.
- Sampling policy drops high-risk traces; recovery is override sampling profile.
- Monitor config change lacks audit seal; recovery is rollback config.
- Pack deactivation requested during retention; recovery is defer.
- Model card expires; recovery is disable monitor.
- AI SLO metric missing tenant label; recovery is reject metric.
- Dashboard role scope too broad; recovery is restrict dashboard.

## Cross-µservice coordination
- `tenancy` provides EU pack roster and high-risk context.
- `identity` provides reviewer roles and admin identity.
- `compliance` owns model inventory, assessments, and AI evidence exports.
- `audit-chain` seals monitor, drift, incident, and export events.
- `policy-engine` loads all `EUAI-observability-*` fragments.
- `workflow-engine` runs review, incident, and evidence workflows.
- `model-registry` publishes model cards and version pins.
- `foundry-runtime` or AI gateway enforces provider BYOK.
- `incident-response` handles harmful AI output candidates.
- `admin-console` renders AI monitor dashboards.
- `data-warehouse` receives aggregate AI SLO metrics.
- `release-engine` enforces threshold rollback plans.
- `mail` emits AI touchpoint traces to this monitor.
- `drive` emits AI touchpoint traces to this monitor.
- `calendar` emits AI touchpoint traces to this monitor.
- `legal` reviews high-risk monitor requirements.
- `support` cannot inspect raw prompts.
- `localization` provides EU transparency labels.
- `security` consumes drift and anomaly alerts.
- `pack-registry` signs this EU AI Act observability overlay.
