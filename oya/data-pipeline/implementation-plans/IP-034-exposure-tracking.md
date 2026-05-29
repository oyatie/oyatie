# IP-034 Data Pipeline exposure tracking finalization

Service: data-pipeline
Implementation plan: IP-034
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names exposure tracking as missing.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Add exposure tracking as a lineage sub-context.
- Register downstream consumers for datasets, transforms, semantic metrics, materializations, destination loads, and marketplace packages.
- Cover dashboard, ml_model, customer_api, marketplace_app, marketplace_workflow, ontology_projection, partner_integration, regulatory_report, and internal_report.
- Make impact notification mandatory when upstream schema, freshness, materialization, or DealSet status changes.
- Enforce tenant scope and pack overlay before registering or notifying an exposure.
- Close the dbt Cloud-style exposure gap in audit Section 3.9.2.
- Preserve exposure history append-only for replay and audit.
- Treat marketplace app and workflow exposure as DealSet-bound per IP-014.
- Bind every exposure to ADR-MS-001 lineage facets.
- No writes outside microservices/data-pipeline/ are required for this plan.

## Interfaces
- REST command `POST /data-pipeline/actions/exposure.register`.
- REST command `POST /data-pipeline/actions/exposure.amend`.
- REST command `POST /data-pipeline/actions/exposure.promote`.
- REST command `POST /data-pipeline/actions/exposure.deprecate`.
- REST command `POST /data-pipeline/actions/exposure.notify-impact`.
- REST query `GET /data-pipeline/exposures/{exposure_id}/upstream`.
- REST query `GET /data-pipeline/exposures/{exposure_id}/downstream`.
- gRPC service `DataExposureRegistry`.
- Contract `contracts/exposure-impact-notification-v1.yaml`.
- Capability records `capabilities/exposure-register.yaml` and `exposure-impact-notify.yaml`.
- Cedar fragments `policies/local-exposure-register-scope.cedar` and `local-exposure-impact-notify-scope.cedar`.
- SLO projection `slos/local-exposure-impact-notify-lag.openslo.yaml`.
- Runbook `runbooks/exposure-impact-resolution.md`.

## Data Flow
- Steward registers an exposure with exposure_type, maturity, owner_team, oncall_contact, runbook_url, notify_channels, and upstream_refs.
- Cedar verifies actor, tenant scope, upstream visibility, pack overlay, and DealSet state when needed.
- Registry resolves upstream refs to dataset_id, transform_run_id, semantic_metric_id, destination_load_run_id, connector_id, or package_id.
- Lineage facet payload binds exposure to upstream edges.
- Promotion to production requires owner_team, oncall_contact, runbook_url, and notification channel.
- Upstream drift from IP-026 triggers impact notification.
- Semantic metric version bump from IP-033 triggers impact notification.
- Destination rollback from IP-031 triggers impact notification.
- Materialization rollback from IP-035 triggers impact notification.
- Package deprecation from IP-036 triggers impact notification.
- CDK connector withdraw from IP-037 triggers impact notification.
- Notification failure creates exposure impact dead letter for IP-028 custody.

## Cedar Policy
- Deny exposure.register without tenant scope.
- Deny exposure.register when upstream_ref is unreadable by tenant.
- Deny exposure.register when exposure_type is marketplace_app or marketplace_workflow and DealSet is absent.
- Deny exposure.register for regulatory_report without active compliance pack.
- Deny exposure.promote to production without owner_team, oncall_contact, and runbook_url.
- Deny impact webhook to a jurisdiction outside tenant home_cell unless pack overlay permits.
- Deny exposure.amend that reduces maturity without operator override.
- Deny exposure.deprecate without minimum 14-day grace window.
- Deny exposure.query_downstream when requestor lacks tenant_data_consumer or higher.
- Deny notification payload carrying raw secret or raw tenant identifier.
- Deny mutation during audit-chain outage.
- Deny query on stale cache when pack overlay changed after cache build.

## Event Shapes
- `oya.data.pipeline.exposure.registered` carries tenant_id, tenant_class, exposure_id, exposure_type, maturity, owner_team, upstream_refs.
- `oya.data.pipeline.exposure.amended` carries changed_fields, previous_version, next_version, amendment_reason.
- `oya.data.pipeline.exposure.promoted` carries prior_maturity, next_maturity, production_runbook_url, oncall_contact.
- `oya.data.pipeline.exposure.deprecated` carries grace_window_days, replacement_exposure_id, custody_until.
- `oya.data.pipeline.exposure.impact_notified` carries upstream_ref, change_kind, change_severity, notify_channels, expected_resolution_at.
- `oya.data.pipeline.exposure.impact_notify_dead_letter` carries notification_id, channel, failure_reason, retry_after_ms.
- `oya.data.pipeline.exposure.rolled_back` carries rollback_bundle_id, restored_version, impacted_exposure_ids.
- Every event includes audit_event_id, cedar_decision_id, traceparent, home_cell, and lineage_facet_id.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for exposure registry availability.
- Reuse `read-latency.openslo.yaml` target 0.999 for exposure graph queries.
- Reuse `write-latency.openslo.yaml` target 0.999 for exposure mutations.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for upstream visibility checks.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for exposure events.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 because exposures are lineage consumers.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 for upstream drift impact.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for freshness notifications.
- Reuse `replay-freshness.openslo.yaml` target 0.999 for impact notification replay.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for notification dead letters.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 for transform-driven exposures.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for quality-impact alerts.
- Add `local-exposure-impact-notify-lag.openslo.yaml`: async p95 5s, email p95 30s, webhook p95 60s.

## Failure Modes
- Missing lineage facet holds exposure.register.
- Unreachable notify target retries with backoff.
- Webhook jurisdiction violation is denied before outbound call.
- DealSet lapse marks marketplace exposure invalid.
- Owner contact missing blocks production promotion.
- Upstream ref deleted or rolled back emits impact notification before deprecation.
- Notification dead-letter replay uses IP-028 custody.
- Cedar outage fails closed for mutation and protected query.
- Audit-chain outage holds mutation.
- Pack overlay update invalidates cached exposure reads.
- Query graph cycle opens lineage reconciliation case.
- Runbook missing blocks production maturity.

## Migration
- Add exposure-tracking to manifest bounded_sub_contexts under lineage.
- Convert dashboard and report references into data_exposure records.
- Convert marketplace package consumers into marketplace exposure records with DealSet ids.
- Add tenant_class to all exposure events.
- Move any tier-oriented owner language to tenant_class capacity or paid billing components.
- Backfill upstream_refs from IP-027 lineage graph.
- Backfill metric exposure_refs from IP-033 after metric registry exists.
- Backfill materialization exposure refs from IP-035 after family registry exists.
- Root IP-034 remains historical evidence; this file is the handoff artifact.
- Every migration row is append-only and retains old owner notes.
- No foreign microservice writes are needed.
- Cross-service consumers use contracts only.

## Dependencies
- IP-001 tenant scope kernel supplies exposure TenantScope.
- IP-002 Cedar default deny gates registration and notification.
- IP-003 ontology projection is an exposure type.
- IP-004 workflow templates may be marketplace_workflow exposures.
- IP-005 REST surface publishes exposure endpoints.
- IP-006 async event surface publishes exposure events.
- IP-007 gRPC surface publishes exposure registry.
- IP-008 policy eval binding checks upstream visibility.
- IP-009 credential sidecar never leaks secrets into exposure payloads.
- IP-010 multi-region layout constrains notification jurisdiction.
- IP-011 audit events records exposure operations.
- IP-012 abuse defence protects webhook targets.
- IP-013 emergency bypass cannot bypass exposure policy.
- IP-014 DealSet settlement licenses marketplace exposure types.
- IP-015 data residency overlays restrict channels.
- IP-016 backfill replay updates exposure impact windows.
- IP-017 cost budget enforcer can notify over-budget exposures.
- IP-018 capacity admission can notify deferred exposure refreshes.
- IP-019 SDK generation exposes exposure clients.
- IP-020 catalog registration catalogs exposure domain.
- IP-021 SLO promotion blocks exposure rollout on burn.
- IP-022 chaos drills test notification dead letter.
- IP-023 DPIA packet records regulatory report exposure.
- IP-024 threat model maps exposure exfiltration risk.
- IP-025 audit closeout proves exposure finding closure.
- IP-026 schema drift quarantine triggers exposure impact.
- IP-027 lineage graph reconciliation supplies upstream refs.
- IP-028 dead-letter replay custody replays failed notifications.
- IP-029 transform cost attribution can trigger cost exposure impact.
- IP-030 watermark governance triggers freshness impact.

## ADR-MS-001 Binding
- Exposure tracking consumes OpenLineage-compatible facets from lineage operations.
- Downstream consumers are part of audit evidence for authoritative outputs.
- Replay and rollback notify impacted exposures before silent reuse.
- Quality failures above threshold quarantine outputs and notify exposures.
- Exposure telemetry avoids raw tenant identifiers.
- Exposure history is append-only.

## Acceptance Gates
- Gate 1: exposure-tracking appears under lineage bounded_sub_contexts.
- Gate 2: all nine exposure types have domain tests.
- Gate 3: Cedar denies marketplace exposure without DealSet.
- Gate 4: production promotion requires owner_team, oncall_contact, and runbook_url.
- Gate 5: impact notifications fire from IP-026, IP-031, IP-033, IP-035, IP-036, and IP-037.
- Gate 6: notification dead letters route to IP-028 custody.
- Gate 7: exposure impact contract is published.
- Gate 8: lineages are queryable upstream and downstream.
- Gate 9: all 12 existing OpenSLOs are cited in promotion checklist.
- Gate 10: local-exposure-impact-notify-lag SLO is filed.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit exposure gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md:37` - - SLO projection `slos/local-exposure-impact-notify-lag.openslo.yaml`.; `microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md:78` - ## SLO Targets.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md:83` - - Reuse `audit-emission-lag.openslo.yaml` target 0.999 for exposure events.; `microservices/data-pipeline/implementation-plans/IP-034-exposure-tracking.md:138` - - IP-017 cost budget enforcer can notify over-budget exposures..
