# IP-033 Data Pipeline semantic layer finalization

Service: data-pipeline
Implementation plan: IP-033
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names semantic-layer and metrics as missing.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Add semantic metric registration as a transform sub-context rather than a new microservice.
- Own metric names, versions, expressions, dimensions, entity joins, time grains, materialization pointers, and exposure references.
- Provide dbt Semantic Layer parity without copying dbt names into the domain model.
- Keep metrics tenant-scoped and tenant_class-aware.
- Enforce pack overlays on dimension access before any metric query plan is produced.
- Feed ontology and analytics with one canonical metric registry contract.
- Bind semantic metric lineage to transform_run_id and destination_load_run evidence.
- Close the feature matrix gap for semantic layer and metrics.
- Preserve ADR-MS-001: metrics cannot become authoritative without source, policy, lineage, and audit evidence.
- No files outside microservices/data-pipeline/ are required for this plan.

## Interfaces
- REST command `POST /data-pipeline/actions/semantic-metric.define`.
- REST command `POST /data-pipeline/actions/semantic-metric.amend`.
- REST command `POST /data-pipeline/actions/semantic-metric.approve`.
- REST command `POST /data-pipeline/actions/semantic-metric.deprecate`.
- REST command `POST /data-pipeline/actions/semantic-metric.query-plan`.
- REST query `GET /data-pipeline/semantic-metrics/{metric_name}`.
- gRPC service `SemanticMetricRegistry`.
- Contract `contracts/semantic-metric-registry-v1.yaml`.
- Capability records `capabilities/semantic-metric-define.yaml` and `semantic-metric-read.yaml`.
- Cedar fragments `policies/local-semantic-metric-define-scope.cedar` and `local-semantic-metric-read-scope.cedar`.
- SLO projection `slos/local-semantic-metric-read-latency.openslo.yaml`.
- Runbook `runbooks/semantic-metric-drift.md`.

## Data Flow
- Steward defines metric with metric_name, metric_kind, expression, source transform_run_id, time dimension, dimensions, and entity joins.
- Cedar validates tenant scope, actor audience, dimension allow-list, pack overlay, and ontology partition.
- Expression parser normalizes metric DSL and produces expression_normalized_hash.
- Metric approval records semantic version and deprecation policy.
- Materialization pointer binds to IP-035 family when metric requires caching or table output.
- Exposure refs bind to IP-034 when a dashboard, API, report, or marketplace app consumes the metric.
- Query-plan command resolves allowed dimensions, time grain, materialization family, and destination class.
- Query execution reads through analytics or ontology contract depending on metric shape.
- Audit event records define, approve, query, and deprecate.
- Lineage facet cites upstream transform and downstream consumers.
- Deprecated metric versions stay readable through custody window for replay.
- Package-managed metrics from IP-036 install through same registry.

## Cedar Policy
- Deny metric.define without tenant scope.
- Deny metric.define without tenant_class.
- Deny metric.define if metric_name violates BNF v4.1 naming.
- Deny metric.define if expression references a column not allowed for the tenant pack.
- Deny metric.define if entity join crosses ontology partition.
- Deny metric.approve unless parser validation passed.
- Deny metric.amend when breaking change lacks MAJOR version bump.
- Deny metric.read when requested dimension is outside dimensions_allowed.
- Deny metric.read when requested time grain is outside time_grains_allowed.
- Deny metric.read when aggregation count violates HIPAA or KR-PIPA minimum.
- Deny metric.deprecate without grace window unless operator override exists.
- Deny semantic mutation during audit-chain outage.

## Event Shapes
- `oya.data.pipeline.semantic_metric.defined` carries tenant_id, tenant_class, metric_name, metric_version, metric_kind, expression_hash.
- `oya.data.pipeline.semantic_metric.approved` carries approval_principal_id, parser_version, materialization_policy, policy_decision_id.
- `oya.data.pipeline.semantic_metric.amended` carries previous_version, next_version, breaking_change, amendment_reason.
- `oya.data.pipeline.semantic_metric.deprecated` carries grace_window_days, replacement_metric_ref, custody_until.
- `oya.data.pipeline.semantic_metric.query_planned` carries requested_dimensions, allowed_dimensions, time_grain, plan_hash, materialization_pointer.
- `oya.data.pipeline.semantic_metric.query_denied` carries denial_reason, denied_dimension, pack_overlay, policy_decision_id.
- `oya.data.pipeline.semantic_metric.rolled_back` carries rolled_back_version, restored_version, rollback_bundle_id.
- Every event includes traceparent, audit_event_id, cedar_decision_id, home_cell, and lineage_facet_id where applicable.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for metric registry availability.
- Reuse `read-latency.openslo.yaml` target 0.999 for metric metadata reads.
- Reuse `write-latency.openslo.yaml` target 0.999 for metric mutations.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for dimension authorization.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for metric events.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 for source transform execution.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 for metric lineage.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for quality-gated metric inputs.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 for metric source drift.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for source freshness.
- Reuse `replay-freshness.openslo.yaml` target 0.999 for metric replay.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for failed query-plan or materialization events.
- Add `local-semantic-metric-read-latency.openslo.yaml`: p95 cached 500ms, incremental 5s, refresh-triggered 60s.

## Failure Modes
- Expression parse failure refuses define and emits query_denied-style evidence.
- Pack overlay loader failure fails closed.
- Ontology entity unavailable holds approval and links lineage-gap-repair.
- Materialization failure downgrades metric to view only when policy permits.
- Deprecated version read beyond custody window is denied.
- Dimension request violating lawful basis is denied.
- Query plan reaching unavailable destination returns stale banner if cached and policy permits.
- Cedar outage denies writes and protected reads.
- Audit-chain outage holds mutation.
- Package-installed metric lockfile drift blocks query-plan.
- Schema drift in input opens IP-026 case and freezes metric promotion.
- Exposure update failure sends IP-034 impact notification to retry.

## Migration
- Add semantic-layer to manifest bounded_sub_contexts under transform.
- Keep existing transform context; do not create semantic-layer microservice.
- Backfill current transform-derived metrics into semantic_metric_definition rows.
- Start with simple and ratio metrics before cumulative, conversion, and funnel.
- Convert root IP-033 into historical evidence; this implementation-plans file is the handoff.
- Require tenant_class on every metric event.
- Replace any customer-tier pricing phrasing with tenant_class metering.
- Add pack restriction overlays before exposing metric.read.
- Add ontology and analytics consumers after registry contract stabilizes.
- Preserve previous metric names as aliases only with explicit deprecation.
- Every migration step is append-only.
- No metric rewrite deletes prior audit evidence.

## Dependencies
- IP-001 tenant scope kernel supplies tenant and principal facts.
- IP-002 Cedar default deny gates define and read.
- IP-003 ontology projection resolves entity joins.
- IP-004 workflow templates can run metric materialization workflows.
- IP-005 REST surface publishes semantic commands.
- IP-006 async event surface publishes semantic events.
- IP-007 gRPC surface publishes registry service.
- IP-008 policy eval binding evaluates dimension Cedar.
- IP-009 credential sidecar supplies destination query credentials.
- IP-010 multi-region cell layout constrains metric home_cell.
- IP-011 observability audit events records metric operations.
- IP-012 abuse defence protects exposed metric read paths.
- IP-013 emergency bypass cannot bypass pack overlays.
- IP-014 DealSet settlement licenses marketplace metrics.
- IP-015 data residency overlays restrict dimensions.
- IP-016 backfill replay worker replays metric source runs.
- IP-017 cost budget enforcer guards metric materialization cost.
- IP-018 capacity admission guards high-cardinality metric reads.
- IP-019 SDK generation exposes metric clients.
- IP-020 catalog registration catalogs semantic metric domain.
- IP-021 SLO promotion blocks semantic rollout on burn.
- IP-022 chaos drills test parser and dimension denial.
- IP-023 DPIA evidence records sensitive metric fields.
- IP-024 threat model maps metric exfiltration.
- IP-025 audit closeout proves semantic-layer finding closure.
- IP-026 drift quarantine blocks unsafe metric source changes.
- IP-027 lineage reconciliation seals metric edges.
- IP-028 dead-letter custody owns failed materialization replay.
- IP-029 transform cost attribution feeds metric compute cost.
- IP-030 watermark governance marks metric freshness.

## ADR-MS-001 Binding
- Semantic metrics derive from transform actions and must cite transform_run_id.
- Metric query plans cannot override lineage-first evidence.
- Replay uses original event id plus new replay id.
- Quality failure above threshold quarantines metric version.
- Metrics avoid raw tenant identifiers in telemetry.
- Append-only metric versions satisfy auditability.

## Acceptance Gates
- Gate 1: semantic-layer appears under transform bounded_sub_contexts.
- Gate 2: semantic metric registry contract is published.
- Gate 3: metric.define, amend, approve, deprecate, query-plan, and read exist in REST and gRPC.
- Gate 4: Cedar denies unauthorized dimensions and time grains.
- Gate 5: pack overlay tests cover KR-PIPA, HIPAA, GDPR, and PCI.
- Gate 6: lineage facet exists for every approved metric.
- Gate 7: metric materialization binds to IP-035.
- Gate 8: metric exposure refs bind to IP-034.
- Gate 9: all 12 existing OpenSLOs are cited in the promotion checklist.
- Gate 10: local-semantic-metric-read-latency SLO is filed.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit semantic-layer gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md:36` - - SLO projection `slos/local-semantic-metric-read-latency.openslo.yaml`.; `microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md:77` - ## SLO Targets.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md:82` - - Reuse `audit-emission-lag.openslo.yaml` target 0.999 for metric events.; `microservices/data-pipeline/implementation-plans/IP-033-semantic-layer.md:137` - - IP-017 cost budget enforcer guards metric materialization cost..
