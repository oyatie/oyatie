---
doc_class: CompliancePackOverlay
pack_id: EU-GDPR-2018-baseline
microservice: observability
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# observability GDPR Compliance Pack Overlay

## Pack Identity
- Full pack name: EU GDPR observability personal-data telemetry overlay.
- Citing jurisdiction: European Union and EEA personal-data regime.
- Version: EU-GDPR-2018-baseline-v1.
- Canonical source URL: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- Cited law: Regulation (EU) 2016/679.
- Covered observability surface: logs, metrics, traces, exemplars, dashboards, alerts, query history, exports, retention, and breach telemetry.
- Pack activation means observability treats telemetry as personal data when it can identify a person directly or indirectly.
- IP addresses, user ids, email addresses, device ids, trace baggage, and query text are all personal-data risk points.
- Data classes include `OBSERVABILITY_PERSONAL_DATA_EU`, `OBSERVABILITY_SPECIAL_CATEGORY_EU`, and `OBSERVABILITY_DSAR_EXPORT_EU`.
- GDPR erasure requires telemetry tombstone or aggregation proof, not silent retention.
- ADR-0064 keeps EU privacy behavior in this overlay.
- ADR-0251 supplies pack admission, DSAR, breach, and cell eligibility.
- ADR-0263 supplies emission contract and scrub-before-storage rules.
- PCI-DSS is omitted because observability does not own payment authorization.
- Payment telemetry must be tokenized before it reaches this service.

## Data Model Deltas
- Add `telemetry.eu_personal_data_signal` as enum `none|personal|special_category`.
- Add `telemetry.lawful_basis`.
- Add `telemetry.lawful_basis_evidence_id`.
- Add `telemetry.data_subject_ids_hash`.
- Add `telemetry.erasure_state` as enum `active|restricted|erasure_pending|tombstoned`.
- Add `telemetry.restriction_reason`.
- Add `telemetry.scrub_profile_id`.
- Add `telemetry.original_payload_hash`.
- Add `telemetry.transfer_mechanism`.
- Add `telemetry.eu_residency_cell`.
- Add `log.personal_data_scrub_verdict`.
- Add `trace.baggage_scrubbed` boolean.
- Add `metric.personal_label_rejection_count`.
- Add `exemplar.personal_data_safe` boolean.
- Add `query_history.dsar_subject_hash`.
- Add `dashboard.dpo_visibility_scope`.
- Add `alert.personal_data_payload_scrubbed` boolean.
- Add `rollup.aggregation_anonymity_proof_hash`.
- Add `export_job.gdpr_telemetry_manifest_hash`.
- Add `retention.gdpr_retention_schedule_version`.
- Add `search_index.eu_erasure_rebuild_required`.
- Add `audit_shadow.gdpr_event_id`.
- Add `tenant_observability_config.eu_dpa_version`.
- Add `tenant_observability_config.eu_scrub_profile_version`.

## Cedar Policy Deltas
- Policy `GDPR-observability-ingest-01`: require lawful basis for personal telemetry storage.
- Policy `GDPR-observability-ingest-02`: forbid special-category telemetry unless explicit basis exists.
- Policy `GDPR-observability-label-01`: reject personal-data metric labels.
- Policy `GDPR-observability-baggage-01`: scrub trace baggage before storage.
- Policy `GDPR-observability-log-01`: require personal-data scrub verdict before Loki ingest.
- Policy `GDPR-observability-query-01`: restrict query history to purpose and role.
- Policy `GDPR-observability-dashboard-01`: require DPO visibility scope for personal-data dashboards.
- Policy `GDPR-observability-alert-01`: scrub alert payload before fanout.
- Policy `GDPR-observability-export-01`: permit DSAR telemetry export for verified subject or DPO.
- Policy `GDPR-observability-erasure-01`: permit tombstone when no overriding retention exists.
- Policy `GDPR-observability-erasure-02`: require aggregation proof if tombstone is impossible.
- Policy `GDPR-observability-restrict-01`: block serving during accuracy dispute.
- Policy `GDPR-observability-transfer-01`: forbid non-EEA telemetry route without transfer mechanism.
- Policy `GDPR-observability-retention-01`: forbid blanket indefinite telemetry retention.
- Policy `GDPR-observability-rollup-01`: require anonymity proof before rollup publication.
- Policy `GDPR-observability-breach-01`: start Article 33 clock on confirmed personal telemetry leak.
- Policy `GDPR-observability-admin-01`: require DPO-visible audit for admin query.
- Policy `GDPR-observability-replay-01`: require re-scrub before telemetry replay.
- Policy `GDPR-observability-webhook-01`: require processor DPA for alert webhook.
- Policy `GDPR-observability-ai-01`: require model-touch lawful basis for anomaly explanation.
- Policy `GDPR-observability-index-01`: require index rebuild after erasure.
- Policy `GDPR-observability-exemplar-01`: permit exemplar only when personal-data safe.
- Policy `GDPR-observability-pack-01`: defer deactivation while DSAR or erasure cases are open.
- Policy `GDPR-observability-sample-01`: require sampling policy to avoid personal identifiers.

## API Contract Deltas
- `POST /ingest/logs` requires EU scrub profile version.
- `POST /ingest/logs` rejects special-category payloads without basis.
- `POST /ingest/traces` strips trace baggage unless allowlisted.
- `POST /ingest/metrics` rejects personal-data label keys.
- `POST /ingest/exemplars` requires personal-data-safe trace ref.
- `POST /dsar/export` starts telemetry export for verified subject.
- `GET /dsar/export/{id}` returns manifest hash.
- `POST /dsar/erasure` starts telemetry tombstone workflow.
- `POST /dsar/restrict` restricts telemetry serving.
- `POST /search/rebuild` requires erasure reason.
- `POST /rollups/publish` requires anonymity proof.
- `POST /alerts/routes` requires processor DPA reference.
- `POST /exports` requires DPO approval.
- `POST /replay` requires re-scrub option.
- `POST /ai/anomaly-explain` requires model-touch lawful basis.
- `GET /query-history` is purpose-filtered.
- `POST /breach-candidates` starts GDPR breach clock.
- `PATCH /tenant-observability-config` records EU DPA version.
- `DELETE /telemetry/{id}` returns retention conflict details.
- `POST /pack/deactivate` refuses open DSAR cases.

## Workflow Deltas
- Ingest workflow classifies personal data before storage.
- Log scrubber removes email, IP, device id, and user id where not needed.
- Trace baggage scrubber runs before Tempo ingest.
- Metric label validator rejects personal identifiers.
- Exemplar validator drops unsafe trace references.
- DSAR export enumerates telemetry by subject hash.
- Erasure workflow tombstones or anonymizes telemetry.
- Restriction workflow blocks normal query serving.
- Search index rebuild runs after erasure.
- Rollup workflow proves anonymity before publication.
- Alert workflow strips personal data before fanout.
- Non-EEA route workflow validates transfer mechanism.
- Replay workflow re-scrubs historical telemetry.
- AI anomaly explanation records lawful basis.
- Breach candidate workflow starts Article 33 clock.
- Admin query workflow creates DPO-visible review event.
- Retention schedule workflow records processing purpose.
- Pack activation scans dashboards and alert routes.
- Pack deactivation waits for open DSAR work.
- Audit bundle publication signs erasure and export manifests.

## SLO Deltas
- GDPR breach regulator-readiness p99 target is <= 60 hours.
- Breach clock creation p99 target is <= 5 minutes.
- Personal-data scrub p99 must stay <= 100 ms per telemetry item.
- DSAR telemetry enumeration first response target is <= 7 days.
- Full telemetry export target is <= 30 days.
- Erasure tombstone or anonymization p99 target is <= 72 hours after approval.
- Search index rebuild target is <= 24 hours.
- Restriction activation p99 must complete <= 15 minutes.
- EU route validation p99 must stay <= 100 ms.
- DPA lookup p99 must stay <= 200 ms.
- Rollup anonymity proof p99 target is <= 10 minutes.
- Admin query audit p99 must complete <= 1 second.
- Alert scrub p99 must stay <= 200 ms.
- Replay re-scrub throughput target is >= 10k events per minute.
- DPO dashboard lag target is <= 15 minutes.
- Scrub false-negative review cadence is daily.

## Audit-event class additions
- `ObservabilityGdprIngestPreflighted` records stream and verdict.
- `ObservabilityGdprPayloadScrubbed` records profile and payload hash.
- `ObservabilityGdprMetricLabelRejected` records label key hash.
- `ObservabilityGdprTraceBaggageScrubbed` records trace id hash.
- `ObservabilityGdprExemplarDropped` records reason.
- `ObservabilityGdprDsarExportStarted` records case id.
- `ObservabilityGdprDsarExportCompleted` records manifest hash.
- `ObservabilityGdprTelemetryTombstoned` records telemetry id.
- `ObservabilityGdprTelemetryAnonymized` records proof hash.
- `ObservabilityGdprRestrictionApplied` records reason.
- `ObservabilityGdprIndexRebuilt` records shard id.
- `ObservabilityGdprRollupProofCreated` records rollup id.
- `ObservabilityGdprAlertPayloadScrubbed` records route id.
- `ObservabilityGdprReplayRescrubbed` records replay id.
- `ObservabilityGdprAiExplanationChecked` records model id.
- `ObservabilityGdprBreachClockStarted` records candidate id.
- `ObservabilityGdprAdminQueryReviewed` records review id.
- `ObservabilityGdprRetentionScheduleChanged` records schedule.
- `ObservabilityGdprRouteBlocked` records target cell.
- `ObservabilityGdprPackDeactivationDeferred` records open cases.

## Failure Modes specific to this pack
- Personal-data detector unavailable; recovery is fail-closed for high-risk streams.
- Scrub profile missing; recovery is reject ingest.
- Metric label includes email; recovery is reject sample and page service owner.
- Trace baggage contains personal data; recovery is strip baggage and open event.
- DSAR subject hash misses telemetry; recovery is rerun from audit-chain cursor.
- Erasure cannot tombstone aggregated metric; recovery is anonymization proof.
- Search shard serves erased telemetry; recovery is remove shard and rebuild.
- Alert webhook lacks DPA; recovery is disable route.
- Transfer mechanism expires; recovery is block non-EEA route.
- AI anomaly explanation lacks lawful basis; recovery is block model touch.
- Admin query lacks DPO case; recovery is revoke session.
- Retention schedule is indefinite; recovery is reject schedule.
- Rollup proof fails; recovery is keep rollup unpublished.
- Replay bypasses scrubber; recovery is halt replay.
- Breach clock fails to start; recovery is retroactive event and page compliance.
- Pack deactivation requested with open DSAR; recovery is defer.
- Dashboard exposes personal variable; recovery is disable dashboard.
- Query history leaks subject id; recovery is tombstone query log.
- EU cell outage suggests non-EU failover; recovery is buffer or reject.
- Scrub false negative discovered; recovery is retroactive scrub and incident review.

## Cross-µservice coordination
- `tenancy` provides EU cell placement and active pack roster.
- `identity` verifies data subject, DPO role, and admin identity.
- `compliance` owns DSAR, processing register, DPA, and breach cases.
- `audit-chain` seals scrub, erasure, export, and breach events.
- `policy-engine` loads all `GDPR-observability-*` fragments.
- `workflow-engine` runs DSAR, erasure, restriction, and breach workflows.
- `mail` emits personal-data-safe telemetry under this profile.
- `drive` emits personal-data-safe telemetry under this profile.
- `calendar` emits personal-data-safe telemetry under this profile.
- `notification` receives scrubbed alert payloads.
- `incident-response` consumes telemetry leak candidates.
- `admin-console` renders DPO-visible dashboards.
- `data-warehouse` receives anonymized aggregates only.
- `storage` provides EU telemetry backend proof.
- `support` uses DPO-visible query path.
- `dlp-virus-scan` contributes personal-data detector verdicts.
- `release-engine` gates scrub profile rollout.
- `legal` defines export redaction rules.
- `localization` provides EU privacy notices.
- `pack-registry` signs this GDPR observability overlay.
