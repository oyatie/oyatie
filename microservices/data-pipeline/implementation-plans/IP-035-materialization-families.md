# IP-035 Data Pipeline materialization families finalization

Service: data-pipeline
Implementation plan: IP-035
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names materialization families as missing.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Define the canonical materialization families for transforms and semantic metrics.
- Cover view, table, incremental, ephemeral, and snapshot.
- Bind each family to destination_load_run behavior from IP-031 when a destination object exists.
- Bind refresh cadence to schedule behavior from IP-032.
- Bind semantic metric policy to IP-033.
- Bind downstream impact to exposure tracking from IP-034.
- Bind incremental family to IP-030 watermarks.
- Preserve pack overlay, tenant_class, home_cell, and data_class on every materialization.
- Close the feature matrix gap for dbt-style materializations without copying dbt configuration semantics wholesale.
- No files outside microservices/data-pipeline/ are required for this plan.

## Interfaces
- REST command `POST /data-pipeline/actions/materialization.define`.
- REST command `POST /data-pipeline/actions/materialization.amend`.
- REST command `POST /data-pipeline/actions/materialization.promote`.
- REST command `POST /data-pipeline/actions/materialization.refresh`.
- REST command `POST /data-pipeline/actions/materialization.deprecate`.
- REST query `GET /data-pipeline/materializations/{binding_id}/state`.
- gRPC service `MaterializationPolicyControl`.
- Contract `contracts/materialization-policy-v1.yaml`.
- Capability records `capabilities/materialization-define.yaml` and `materialization-refresh.yaml`.
- Cedar fragments `policies/local-materialization-define-scope.cedar` and `local-materialization-promote-scope.cedar`.
- SLO projections `slos/local-materialization-refresh-success-rate.openslo.yaml` and `local-materialization-refresh-latency.openslo.yaml`.
- Runbooks `runbooks/materialization-refresh-failure.md` and `materialization-incremental-cursor-divergence.md`.

## Data Flow
- Steward defines materialization_policy_binding for a transform_id or semantic_metric_id.
- Cedar validates tenant scope, family choice, destination permission, pack overlay, cost budget, and actor audience.
- View family stores no destination load run; reads resolve through destination query engine.
- Table family triggers full destination_load_run refresh through IP-031.
- Incremental family reads IP-030 captured, landed, or transformed watermark and writes delta load through IP-031.
- Ephemeral family inlines expression into downstream query plan and stores no destination object.
- Snapshot family writes SCD2 history with valid_from and valid_to fields.
- Schedule from IP-032 triggers refresh according to refresh_policy.
- Refresh emits materialization.refreshed or materialization.refresh_failed.
- Rollback uses IP-031 destination rollback and IP-030 watermark repair where needed.
- IP-034 notifies exposures when materialization refresh fails, rolls back, or changes schema.
- IP-033 semantic metrics read materialization_pointer for query planning.

## Cedar Policy
- Deny materialization.define without tenant scope.
- Deny materialization.define without tenant_class.
- Deny family outside view, table, incremental, ephemeral, snapshot.
- Deny ephemeral when destination_id is present.
- Deny table, incremental, or snapshot when destination_id is absent.
- Deny incremental when watermark_binding is missing or not captured, landed, or transformed.
- Deny snapshot when snapshot_strategy is missing.
- Deny define when destination home_cell conflicts with pack overlay.
- Deny refresh when IP-026 drift case is open.
- Deny refresh when IP-017 cost budget is exhausted.
- Deny promote when prior refresh failed without disposition.
- Deny mutation during audit-chain outage.

## Event Shapes
- `oya.data.pipeline.materialization.defined` carries tenant_id, tenant_class, binding_id, family, transform_or_metric_ref, destination_id.
- `oya.data.pipeline.materialization.amended` carries previous_version, next_version, changed_fields, amendment_reason.
- `oya.data.pipeline.materialization.promoted` carries promotion_stage, policy_decision_id, owner_principal_id.
- `oya.data.pipeline.materialization.refresh_started` carries refresh_run_id, schedule_id, watermark_before, destination_load_run_id.
- `oya.data.pipeline.materialization.refreshed` carries refresh_run_id, rows_written, bytes_written, watermark_after, destination_commit_cursor.
- `oya.data.pipeline.materialization.refresh_failed` carries refresh_run_id, failure_reason, rollback_bundle_id, dead_letter_batch_id.
- `oya.data.pipeline.materialization.deprecated` carries grace_window_days, replacement_binding_id, custody_until.
- Every event includes traceparent, audit_event_id, cedar_decision_id, home_cell, and lineage_facet_id where applicable.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for materialization control plane.
- Reuse `write-latency.openslo.yaml` target 0.999 for define and refresh commands.
- Reuse `read-latency.openslo.yaml` target 0.999 for materialization state reads.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for refresh authorization.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for refresh events.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 for transform-derived materializations.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for source freshness.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 for source drift holds.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 for materialized lineage.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for quality-gated inputs.
- Reuse `replay-freshness.openslo.yaml` target 0.999 for materialization rollback replay.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for failed refresh rows.
- Add `local-materialization-refresh-success-rate.openslo.yaml`: table 0.999, incremental 0.9995, snapshot 0.999, view N/A, ephemeral N/A.
- Add `local-materialization-refresh-latency.openslo.yaml`: table p95 10m, incremental p95 2m, snapshot p95 5m for standard workload class.

## Failure Modes
- Full table refresh fails after partial write and opens IP-031 rollback.
- Incremental cursor divergence freezes refresh and opens watermark repair.
- Snapshot strategy cannot detect changes and opens operator review.
- Ephemeral expression compilation fails and blocks downstream query.
- View source schema drift creates warning until IP-026 disposition.
- Pack overlay changes make destination illegal and pause refresh.
- Cost budget denies scheduled refresh before destination write.
- Audit-chain outage holds define, promote, and refresh.
- Cedar outage fails closed for refresh.
- Exposure notification failure from IP-034 becomes dead letter.
- Semantic metric query-plan falls back to view only when policy permits.
- Lockfile drift from package-managed materialization blocks refresh.

## Migration
- Add materialization-policy to manifest bounded_sub_contexts under transform.
- Backfill current transform outputs as table or incremental based on existing watermark evidence.
- Register ephemeral only for expression reuse that has no destination object.
- Register snapshot only after source has stable change detection fields.
- Root IP-035 remains historical evidence; this file is the implementation-plans handoff.
- Add tenant_class to all materialization events.
- Replace any tier-delta materialization wording with tenant_class cost and quota language.
- Introduce view and table first.
- Add incremental after IP-030 watermark checks pass.
- Add snapshot after SCD2 rollback tests pass.
- Every old materialization state remains append-only.
- Cross-service consumers use contracts, not direct writes.

## Dependencies
- IP-001 tenant scope kernel supplies materialization TenantScope.
- IP-002 Cedar default deny gates define and refresh.
- IP-003 ontology projection consumes materialized projections.
- IP-004 workflow template library schedules refresh workflows.
- IP-005 REST surface publishes materialization endpoints.
- IP-006 async event surface publishes materialization events.
- IP-007 gRPC surface publishes policy control.
- IP-008 policy eval binding checks family Cedar.
- IP-009 credential sidecar supplies destination credentials.
- IP-010 multi-region layout constrains destination cell.
- IP-011 audit events records refresh.
- IP-012 abuse defence protects external refresh triggers.
- IP-013 emergency bypass cannot bypass materialization policy.
- IP-014 DealSet licenses marketplace materialization templates.
- IP-015 residency overlays restrict destinations.
- IP-016 backfill replay repairs failed refresh runs.
- IP-017 cost budget enforcer denies over-budget refresh.
- IP-018 capacity admission controls refresh concurrency.
- IP-019 SDK generation exposes materialization clients.
- IP-020 catalog registration catalogs family adapters.
- IP-021 SLO promotion blocks rollout on refresh burn.
- IP-022 chaos drills test partial refresh and rollback.
- IP-023 DPIA evidence records materialized data classes.
- IP-024 threat map covers stale and cross-cell materialization.
- IP-025 audit closeout proves materialization finding closure.
- IP-026 drift quarantine blocks unsafe refresh.
- IP-027 lineage reconciliation seals materialized lineage.
- IP-028 dead-letter custody handles failed refresh rows.
- IP-029 transform cost attribution records refresh cost.
- IP-030 watermark governance gates incremental refresh.

## ADR-MS-001 Binding
- Transform outputs cannot become authoritative before lineage, policy, input snapshot, and audit evidence are sealed.
- Materialization refresh must not bypass data-quality failure quarantine.
- Incremental refresh replay re-evaluates Cedar and idempotency.
- Refresh cost attribution attaches tenant, dataset, transform, and connector dimensions.
- Metrics avoid raw tenant identifiers.
- Materialization state is append-only.

## Acceptance Gates
- Gate 1: materialization-policy appears under transform bounded_sub_contexts.
- Gate 2: all five materialization families have domain tests.
- Gate 3: Cedar denies illegal family/destination combinations.
- Gate 4: incremental refresh requires IP-030 watermark_binding.
- Gate 5: table, incremental, and snapshot bind to IP-031 destination_load_run.
- Gate 6: schedule refresh binds to IP-032.
- Gate 7: semantic metrics bind materialization pointer through IP-033.
- Gate 8: exposure impact fires through IP-034.
- Gate 9: all 12 existing OpenSLOs are cited in promotion checklist.
- Gate 10: local materialization SLOs are filed.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit materialization gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md:36` - - SLO projections `slos/local-materialization-refresh-success-rate.openslo.yaml` and `local-materialization-refresh-latency.openslo.yaml`.; `microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md:77` - ## SLO Targets.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md:41` - - Cedar validates tenant scope, family choice, destination permission, pack overlay, cost budget, and actor audience.; `microservices/data-pipeline/implementation-plans/IP-035-materialization-families.md:63` - - Deny refresh when IP-017 cost budget is exhausted..
