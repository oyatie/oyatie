---
doc_class: CompliancePackOverlay
pack_id: HIPAA-2024
microservice: observability
status: draft
date: 2026-05-20
related_oyatie_adrs: [ADR-0251, ADR-0064, ADR-0263]
---

# observability HIPAA Compliance Pack Overlay

## Pack Identity
- Full pack name: HIPAA Administrative Simplification observability ePHI telemetry overlay.
- Citing jurisdiction: United States federal health information regime.
- Version: HIPAA-2024-v1.
- Canonical source URL: https://www.ecfr.gov/current/title-45/subtitle-A/subchapter-C
- Cited law: 45 CFR Parts 160, 162, and 164.
- Covered observability surface: metrics, logs, traces, exemplars, dashboards, alert payloads, ClickHouse rollups, retention, exports, and incident telemetry.
- Pack activation means observability must never store raw ePHI in telemetry and must retain HIPAA audit evidence.
- The overlay treats span attributes, log messages, labels, exemplar metadata, and dashboard variables as possible PHI leak surfaces.
- Data classes include `OBSERVABILITY_PHI_SIGNAL`, `OBSERVABILITY_PHI_SCRUB_EVENT`, and `OBSERVABILITY_HIPAA_AUDIT_EVIDENCE`.
- Minimum necessary applies to dashboard views and support access.
- ADR-0064 keeps base telemetry neutral while this pack adds scrubbing and retention policy.
- ADR-0251 supplies cell eligibility, breach workflow, and pack bundle signature.
- ADR-0263 is the direct substrate contract for this overlay.
- This overlay excludes PCI-DSS because observability receives tokenized payment telemetry only unless payment service activates PCI scope.
- Any detected PAN-like telemetry is quarantined and escalated to the PCI owner.

## Data Model Deltas
- Add `telemetry.phi_signal` as enum `none|possible|confirmed`.
- Add `telemetry.scrub_profile_id` for HIPAA scrubbing rules.
- Add `telemetry.scrubbed_at` timestamp.
- Add `telemetry.original_payload_hash` to prove redaction without storing PHI.
- Add `telemetry.blocked_label_keys` array.
- Add `telemetry.blocked_attribute_keys` array.
- Add `telemetry.minimum_necessary_dashboard_scope`.
- Add `telemetry.audit_id_required` boolean.
- Add `log.phi_scrub_verdict` as enum `clean|scrubbed|blocked`.
- Add `trace.phi_span_attribute_count`.
- Add `metric.phi_label_rejection_count`.
- Add `exemplar.phi_safe_trace_ref` boolean.
- Add `dashboard.hipaa_view_role_scope`.
- Add `alert.phi_payload_scrubbed` boolean.
- Add `alert.hipaa_breach_candidate_id` nullable.
- Add `rollup.phi_free_aggregation_proof_hash`.
- Add `export_job.hipaa_redaction_manifest_hash`.
- Add `retention.hipaa_audit_floor_iso8601` default `P6Y`.
- Add `support_session.break_glass_reason_id`.
- Add `support_session.telemetry_view_ttl`.
- Add `ingest_reject.phi_reject_reason`.
- Add `audit_shadow.observability_phi_event_id`.
- Add `tenant_observability_config.hipaa_cell_certification`.
- Add `tenant_observability_config.phi_scrub_profile_version`.

## Cedar Policy Deltas
- Policy `HIPAA-observability-ingest-01`: forbid ingest when raw PHI detector returns confirmed.
- Policy `HIPAA-observability-ingest-02`: permit scrubbed ingest when original payload hash exists.
- Policy `HIPAA-observability-label-01`: forbid high-cardinality labels containing patient identifiers.
- Policy `HIPAA-observability-span-01`: require PHI-safe span attribute allowlist.
- Policy `HIPAA-observability-log-01`: require log scrub verdict before Loki storage.
- Policy `HIPAA-observability-metric-01`: require label rejection for PHI-like keys.
- Policy `HIPAA-observability-exemplar-01`: permit exemplar only when trace ref is PHI-safe.
- Policy `HIPAA-observability-dashboard-01`: restrict PHI-risk dashboards to covered workforce roles.
- Policy `HIPAA-observability-alert-01`: scrub alert payload before notification fanout.
- Policy `HIPAA-observability-export-01`: require privacy-office approval for telemetry export.
- Policy `HIPAA-observability-export-02`: require redaction manifest before export release.
- Policy `HIPAA-observability-retention-01`: forbid audit evidence purge before six-year floor.
- Policy `HIPAA-observability-breakglass-01`: permit emergency telemetry view only with reason and TTL <= 1h.
- Policy `HIPAA-observability-route-01`: require HIPAA-certified telemetry storage cell.
- Policy `HIPAA-observability-rollup-01`: require PHI-free aggregation proof before rollup publication.
- Policy `HIPAA-observability-breach-01`: create breach candidate on confirmed telemetry PHI leak.
- Policy `HIPAA-observability-support-01`: require elevated ACR for tenant telemetry support view.
- Policy `HIPAA-observability-query-01`: forbid free-text query returning blocked fields.
- Policy `HIPAA-observability-replay-01`: require re-scrub before telemetry replay.
- Policy `HIPAA-observability-sample-01`: require tail-sampling rules to avoid PHI attribute retention.
- Policy `HIPAA-observability-admin-01`: require audit seal for scrub-profile changes.
- Policy `HIPAA-observability-webhook-01`: forbid alert webhook without BAA destination proof.
- Policy `HIPAA-observability-quarantine-01`: permit quarantine release only with privacy approval.
- Policy `HIPAA-observability-pack-01`: defer deactivation while HIPAA evidence remains retained.

## API Contract Deltas
- `POST /ingest/logs` requires scrub profile version for HIPAA tenants.
- `POST /ingest/logs` rejects confirmed PHI payloads.
- `POST /ingest/traces` rejects blocked span attribute keys.
- `POST /ingest/metrics` rejects PHI-like label keys.
- `POST /ingest/exemplars` requires PHI-safe trace ref.
- `GET /dashboards/{id}` requires covered-workforce role for HIPAA dashboards.
- `POST /alerts/routes` requires BAA proof for external webhook.
- `POST /exports` requires privacy-office approval id.
- `GET /exports/{id}` returns redaction manifest hash.
- `POST /support/break-glass` requires reason id and TTL.
- `POST /rollups/publish` requires PHI-free aggregation proof.
- `POST /scrub-profiles` requires admin audit seal.
- `POST /replay` requires re-scrub option.
- `GET /quarantine/{id}` requires privacy-officer role.
- `POST /quarantine/{id}/release` requires approval id.
- `GET /audit/scrub-events` returns scrub and reject evidence.
- `POST /breach-candidates` starts HIPAA telemetry incident.
- `PATCH /tenant-observability-config` requires HIPAA cell proof.
- `DELETE /evidence/{id}` returns retention conflict before six-year floor.
- `POST /pack/deactivate` returns retained evidence count.

## Workflow Deltas
- Ingest workflow runs PHI detector before storage.
- Confirmed PHI telemetry is quarantined, not stored in primary telemetry.
- Scrub workflow stores payload hash and redaction manifest.
- Metric label validator rejects patient identifiers.
- Trace attribute validator applies allowlist before Tempo ingest.
- Log scrubber applies tenant HIPAA profile before Loki ingest.
- Alert workflow strips PHI before notification fanout.
- Dashboard workflow checks covered workforce role.
- Support break-glass workflow grants one-hour telemetry view.
- Export workflow builds redaction manifest before file creation.
- Rollup workflow proves aggregate is PHI-free.
- Replay workflow re-scrubs historical telemetry.
- Scrub-profile change workflow requires admin approval and audit seal.
- Quarantine release workflow requires privacy-office approval.
- Breach workflow starts when PHI telemetry leak is confirmed.
- Retention workflow blocks evidence purge before six-year floor.
- Storage migration workflow validates HIPAA-certified target cell.
- Pack activation workflow scans existing dashboards and alert routes.
- Pack deactivation waits for retained evidence inventory.
- Audit publication seals every scrub, reject, and export event.

## SLO Deltas
- HIPAA ingest scrub p99 must stay <= 100 ms per telemetry item.
- Confirmed PHI quarantine p99 must complete <= 30 seconds.
- Metric label rejection p99 must stay <= 50 ms.
- Trace attribute filtering p99 must stay <= 100 ms.
- Alert payload scrub p99 must stay <= 200 ms.
- Scrub-profile change audit seal p99 must complete <= 1 second.
- Support break-glass workflow start p99 must complete <= 2 minutes.
- Export redaction manifest generation p99 target is <= 30 minutes.
- Rollup PHI-free proof p99 target is <= 10 minutes.
- Replay re-scrub throughput target is >= 10k events per minute.
- HIPAA breach candidate creation p99 target is <= 5 minutes.
- Retention conflict response p99 must stay <= 300 ms.
- Dashboard authorization p99 must stay <= 100 ms.
- Quarantine release workflow start p99 must complete <= 2 minutes.
- HIPAA telemetry dashboard lag target is <= 5 minutes.
- Scrub false-negative review cadence is daily.

## Audit-event class additions
- `ObservabilityPhiIngestPreflighted` records stream and verdict.
- `ObservabilityPhiPayloadScrubbed` records profile and payload hash.
- `ObservabilityPhiPayloadRejected` records reason.
- `ObservabilityPhiMetricLabelRejected` records label key hash.
- `ObservabilityPhiSpanAttributeBlocked` records attribute key hash.
- `ObservabilityPhiLogStoredScrubbed` records storage backend.
- `ObservabilityPhiAlertPayloadScrubbed` records route id.
- `ObservabilityPhiDashboardViewed` records role scope.
- `ObservabilityPhiBreakGlassStarted` records reason id.
- `ObservabilityPhiExportManifestCreated` records manifest hash.
- `ObservabilityPhiRollupProofCreated` records rollup id.
- `ObservabilityPhiReplayRescrubbed` records replay id.
- `ObservabilityPhiScrubProfileChanged` records profile version.
- `ObservabilityPhiQuarantined` records quarantine id.
- `ObservabilityPhiQuarantineReleased` records approver.
- `ObservabilityPhiBreachCandidateCreated` records candidate id.
- `ObservabilityPhiEvidencePurgeRefused` records retention floor.
- `ObservabilityPhiRouteBlocked` records target cell.
- `ObservabilityPhiWebhookRefused` records destination id.
- `ObservabilityPhiPackDeactivationDeferred` records retained count.

## Failure Modes specific to this pack
- PHI detector is unavailable; recovery is fail-closed for high-risk telemetry streams.
- Scrub profile version missing; recovery is reject ingest.
- Log payload contains confirmed PHI; recovery is quarantine and alert privacy office.
- Metric label contains patient id; recovery is reject sample and page service owner.
- Trace span attribute leaks PHI; recovery is drop attribute and open incident.
- Alert route has no BAA proof; recovery is disable external route.
- Dashboard exposes PHI-risk variable; recovery is disable dashboard until fixed.
- Support view lacks break-glass reason; recovery is deny access.
- Export manifest mismatch appears; recovery is revoke export and rebuild.
- Rollup proof cannot be generated; recovery is keep rollup unpublished.
- Replay bypasses scrubber; recovery is halt replay and quarantine output.
- Scrub-profile change lacks audit seal; recovery is rollback profile.
- Quarantine storage fills; recovery is stop affected telemetry ingest.
- HIPAA-certified telemetry cell unavailable; recovery is buffer or reject.
- Audit-chain backpressure appears; recovery is fail-closed for profile changes and exports.
- Breach candidate clock fails to start; recovery is retroactive event and page compliance.
- Retention purge requested early; recovery is refuse purge.
- Exemplar points to unsafe trace; recovery is drop exemplar.
- Pack deactivation requested with retained evidence; recovery is defer.
- PHI false negative discovered; recovery is retroactive scrub, incident review, and service owner task.

## Cross-µservice coordination
- `tenancy` provides HIPAA cell placement and active pack roster.
- `identity` provides covered workforce role and elevated ACR claims.
- `compliance` provides BAA proof, breach workflow, and privacy-office approvals.
- `audit-chain` seals scrub, reject, export, and breach events.
- `policy-engine` loads all `HIPAA-observability-*` fragments.
- `workflow-engine` runs quarantine, break-glass, export, and breach workflows.
- `mail` must emit PHI-safe telemetry under this profile.
- `drive` must emit PHI-safe telemetry under this profile.
- `calendar` must emit PHI-safe telemetry under this profile.
- `notification` receives only scrubbed alert payloads.
- `incident-response` consumes PHI telemetry leak candidates.
- `admin-console` renders HIPAA dashboard access.
- `data-warehouse` receives only PHI-free aggregates.
- `storage` provides HIPAA-certified telemetry backend proof.
- `support` uses break-glass telemetry view.
- `dlp-virus-scan` contributes PHI detector verdicts.
- `release-engine` gates scrub profile rollout.
- `cloud-kms` or OpenBao stores profile signing keys.
- `legal` defines export redaction rules.
- `pack-registry` signs this HIPAA observability overlay.
