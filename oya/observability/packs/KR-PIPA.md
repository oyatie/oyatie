---
doc_class: CompliancePackOverlay
pack_id: KR-PIPA-2023-amendment
microservice: observability
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# observability KR-PIPA Compliance Pack Overlay

## Pack Identity
- Full pack name: Korea Personal Information Protection Act observability telemetry overlay.
- Citing jurisdiction: Republic of Korea personal information regime.
- Version: KR-PIPA-2023-amendment-v1.
- Canonical source URL: https://law.go.kr/LSW/lsInfoP.do?lsId=011357
- Cited law: 개인정보 보호법, Act No. 17799 baseline with current consolidation at law.go.kr.
- Covered observability surface: logs, metrics, traces, exemplars, dashboards, alerts, query history, exports, retention, and breach telemetry.
- Pack activation means observability captures 동의, 보존, 국외이전, and 처리위탁 evidence for telemetry containing Korean personal information.
- Korean resident registration numbers are forbidden in telemetry labels, span attributes, logs, alerts, and dashboards.
- Data classes include `OBSERVABILITY_PI_KR`, `OBSERVABILITY_SENSITIVE_PI_KR`, `OBSERVABILITY_RRN_KR`, and `OBSERVABILITY_CONSENT_LEDGER_KR`.
- Korean-language notices are required for subject-facing telemetry export and breach workflows.
- ADR-0064 keeps Korean telemetry behavior in this overlay.
- ADR-0251 supplies pack admission, retention, and breach hooks.
- ADR-0263 supplies the scrub-before-storage emission boundary.
- PCI-DSS is omitted because observability does not process card authorization.
- PAN-like telemetry is rejected and routed to quarantine.

## Data Model Deltas
- Add `telemetry.kr_pi_signal` as enum `none|personal|sensitive|rrn`.
- Add `telemetry.kr_consent_id` for 동의 ledger linkage.
- Add `telemetry.kr_processing_purpose_id`.
- Add `telemetry.kr_retention_basis_id` for 보존 proof.
- Add `telemetry.kr_retention_until`.
- Add `telemetry.kr_cross_border_transfer_id`.
- Add `telemetry.kr_processor_delegation_id`.
- Add `telemetry.kr_notice_language` default `ko-KR`.
- Add `telemetry.kr_subject_rights_case_id`.
- Add `telemetry.kr_breach_clock_started_at`.
- Add `log.kr_rrn_scrubbed` boolean.
- Add `trace.kr_rrn_attribute_blocked` boolean.
- Add `metric.kr_personal_label_rejection_count`.
- Add `exemplar.kr_pi_safe` boolean.
- Add `query_history.kr_subject_hash`.
- Add `dashboard.kr_dpo_visibility_scope`.
- Add `alert.kr_pi_payload_scrubbed` boolean.
- Add `rollup.kr_anonymity_proof_hash`.
- Add `export_job.kr_subject_rights_manifest_hash`.
- Add `retention.kr_retention_schedule_version`.
- Add `search_index.kr_erasure_rebuild_required`.
- Add `audit_shadow.kr_pipa_event_id`.
- Add `tenant_observability_config.kr_pipa_notice_version`.
- Add `tenant_observability_config.kr_scrub_profile_version`.

## Cedar Policy Deltas
- Policy `KRPIPA-observability-ingest-01`: require processing purpose for Korean PI telemetry.
- Policy `KRPIPA-observability-ingest-02`: forbid RRN telemetry storage.
- Policy `KRPIPA-observability-label-01`: reject metric labels containing Korean identifiers.
- Policy `KRPIPA-observability-trace-01`: block RRN-like span attributes.
- Policy `KRPIPA-observability-log-01`: require Korean PI scrub verdict before storage.
- Policy `KRPIPA-observability-consent-01`: require 동의 ledger when processing is consent-based.
- Policy `KRPIPA-observability-retention-01`: require 보존 ledger entry for retention.
- Policy `KRPIPA-observability-transfer-01`: forbid cross-border telemetry route without transfer id.
- Policy `KRPIPA-observability-processor-01`: require 처리위탁 registry for telemetry processors.
- Policy `KRPIPA-observability-query-01`: restrict sensitive PI query to approved purpose.
- Policy `KRPIPA-observability-export-01`: require verified identity for subject-rights export.
- Policy `KRPIPA-observability-erasure-01`: permit erasure after retention basis expires.
- Policy `KRPIPA-observability-erasure-02`: require anonymization proof when tombstone impossible.
- Policy `KRPIPA-observability-alert-01`: scrub Korean PI from alert payloads.
- Policy `KRPIPA-observability-dashboard-01`: require DPO-visible scope for Korean PI dashboards.
- Policy `KRPIPA-observability-breach-01`: start Korean breach workflow on confirmed telemetry leak.
- Policy `KRPIPA-observability-admin-01`: require DPO-visible audit for admin query.
- Policy `KRPIPA-observability-replay-01`: require re-scrub before replay.
- Policy `KRPIPA-observability-webhook-01`: require processor delegation for alert webhook.
- Policy `KRPIPA-observability-ai-01`: require explicit consent for AI anomaly explanation touching Korean PI.
- Policy `KRPIPA-observability-index-01`: require index rebuild after erasure.
- Policy `KRPIPA-observability-route-01`: require KR resident telemetry in KR cell.
- Policy `KRPIPA-observability-pack-01`: defer deactivation while ledgers are open.
- Policy `KRPIPA-observability-preview-01`: scrub Korean PI from dashboard previews.

## API Contract Deltas
- `POST /ingest/logs` requires KR scrub profile version.
- `POST /ingest/logs` rejects RRN payloads.
- `POST /ingest/traces` strips blocked span attributes.
- `POST /ingest/metrics` rejects Korean PI label keys.
- `POST /consent/capture` stores Korean notice text hash.
- `POST /consent/withdraw` stops consent-based telemetry processing.
- `POST /retention/rules` requires 보존 basis id.
- `POST /dsr/export` requires verified identity.
- `POST /dsr/erasure` starts Korean telemetry erasure workflow.
- `POST /search/rebuild` requires erasure reason.
- `POST /rollups/publish` requires anonymity proof.
- `POST /alerts/routes` requires processor delegation id.
- `POST /exports` requires DPO approval.
- `POST /replay` requires re-scrub option.
- `POST /ai/anomaly-explain` requires Korean PI model-touch consent.
- `GET /privacy-notices/kr/observability` returns Korean notice.
- `POST /breach-candidates` starts KR breach workflow.
- `PATCH /tenant-observability-config` records KR notice version.
- `DELETE /telemetry/{id}` returns retention conflict details.
- `POST /pack/deactivate` returns open ledger count.

## Workflow Deltas
- Ingest workflow classifies Korean PI and RRN.
- RRN-like telemetry is rejected before storage.
- Consent capture stores 동의 text hash and timestamp.
- Consent withdrawal disables consent-based telemetry processing.
- Retention change records 보존 basis and expiry.
- Cross-border telemetry route checks transfer notice.
- Processor webhook setup verifies 처리위탁 registry.
- Subject-rights export verifies identity before enumeration.
- Erasure workflow tombstones or anonymizes telemetry.
- Search index rebuild runs after erasure.
- Rollup workflow proves anonymity before publication.
- Alert workflow strips Korean PI before fanout.
- Replay workflow re-scrubs historical telemetry.
- AI anomaly explanation requires explicit Korean PI consent.
- Breach candidate workflow starts KR notification timeline.
- DPO admin query review is created for support access.
- Dashboard preview workflow scrubs Korean PI.
- KR cell migration refuses non-KR destination for resident telemetry.
- Pack deactivation waits for consent and retention ledgers.
- Audit bundle publication signs Korean manifests.

## SLO Deltas
- KR breach workflow creation p99 target is <= 5 minutes.
- Korean DPO notification p99 target is <= 24 hours for confirmed leak.
- Korean PI scrub p99 must stay <= 100 ms per telemetry item.
- RRN rejection p99 must stay <= 50 ms.
- Consent capture p99 must stay <= 300 ms.
- Consent withdrawal propagation p99 target is <= 30 minutes.
- Korean subject-rights export target is <= 10 days internal.
- Erasure tombstone or anonymization p99 target is <= 72 hours after approval.
- Search index rebuild target is <= 24 hours.
- KR route residency check p99 must stay <= 100 ms.
- Processor delegation lookup p99 must stay <= 200 ms.
- Korean notice retrieval p99 must stay <= 150 ms.
- Retention ledger write p99 must stay <= 500 ms.
- Admin query audit p99 must complete <= 1 second.
- Alert scrub p99 must stay <= 200 ms.
- Korean observability dashboard lag target is <= 15 minutes.

## Audit-event class additions
- `ObservabilityKrPipaConsentCaptured` records consent id and text hash.
- `ObservabilityKrPipaConsentWithdrawn` records consent id.
- `ObservabilityKrPipaPurposeChecked` records purpose id.
- `ObservabilityKrPipaRrnRejected` records stream id.
- `ObservabilityKrPipaPayloadScrubbed` records scrub profile.
- `ObservabilityKrPipaMetricLabelRejected` records label key hash.
- `ObservabilityKrPipaTraceAttributeBlocked` records attribute key hash.
- `ObservabilityKrPipaRetentionLedgerWritten` records 보존 basis.
- `ObservabilityKrPipaCrossBorderNoticeShown` records transfer id.
- `ObservabilityKrPipaProcessorDelegationChecked` records delegation id.
- `ObservabilityKrPipaSubjectRightsExportStarted` records case id.
- `ObservabilityKrPipaSubjectRightsExportCompleted` records manifest hash.
- `ObservabilityKrPipaTelemetryTombstoned` records telemetry id.
- `ObservabilityKrPipaTelemetryAnonymized` records proof hash.
- `ObservabilityKrPipaAdminQueryReviewed` records DPO review id.
- `ObservabilityKrPipaAiExplanationConsentChecked` records model surface.
- `ObservabilityKrPipaBreachWorkflowStarted` records candidate id.
- `ObservabilityKrPipaKrCellRouteBlocked` records target cell.
- `ObservabilityKrPipaNoticeVersionChanged` records notice version.
- `ObservabilityKrPipaPackDeactivationDeferred` records open ledger count.

## Failure Modes specific to this pack
- Korean PI detector unavailable; recovery is fail-closed for high-risk streams.
- RRN appears in telemetry; recovery is reject payload and page service owner.
- Consent ledger unavailable; recovery is block consent-based telemetry processing.
- Korean notice hash mismatches; recovery is disable affected consent capture.
- Cross-border transfer id missing; recovery is block route.
- Processor delegation registry stale; recovery is suspend webhook.
- Consent withdrawal races replay; recovery is halt replay.
- Retention basis expires during hold; recovery is restrict and route review.
- Subject identity verification fails; recovery is deny export.
- Erasure index rebuild fails; recovery is remove shard from serving.
- KR cell outage suggests non-KR failover; recovery is buffer or reject.
- AI anomaly explanation attempts Korean PI without consent; recovery is block model touch.
- Admin query lacks DPO case; recovery is revoke support session.
- Breach workflow clock fails to start; recovery is create retroactive event.
- Dashboard preview leaks Korean PI; recovery is disable dashboard.
- Replay bypasses scrubber; recovery is halt replay.
- Korean-language notice unavailable; recovery is fail-closed for new processing.
- Pack deactivation requested with open ledgers; recovery is defer.
- Alert payload leaks Korean PI; recovery is disable route and open incident.
- Scrub false negative discovered; recovery is retroactive scrub and incident review.

## Cross-µservice coordination
- `tenancy` provides KR cell placement and active KR-PIPA roster.
- `identity` verifies subject identity, DPO roles, and admin identity.
- `compliance` owns 동의, 보존, 국외이전, and 처리위탁 ledgers.
- `audit-chain` seals Korean telemetry events.
- `policy-engine` loads all `KRPIPA-observability-*` fragments.
- `workflow-engine` runs consent, erasure, breach, and subject-rights workflows.
- `mail` emits Korean PI-safe telemetry under this profile.
- `drive` emits Korean PI-safe telemetry under this profile.
- `calendar` emits Korean PI-safe telemetry under this profile.
- `localization` provides Korean privacy notices.
- `notification` receives scrubbed alert payloads.
- `incident-response` consumes Korean telemetry leak candidates.
- `admin-console` surfaces KR observability configuration.
- `data-warehouse` receives anonymized aggregates only.
- `support` requires DPO-visible query path.
- `dlp-virus-scan` contributes RRN detector verdicts.
- `release-engine` gates scrub profile rollout.
- `legal` defines Korean export redaction rules.
- `security` consumes telemetry leak alerts.
- `pack-registry` signs this KR-PIPA observability overlay.
