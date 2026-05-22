# IP-032 Data Pipeline scheduling finalization

Service: data-pipeline
Implementation plan: IP-032
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-032-scheduling.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names scheduling as missing at bounded-context level.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Establish `schedule` as the data-pipeline bounded context for pipeline cadence definition and fired-run evidence.
- Keep workflow-engine as execution orchestrator; data-pipeline owns the schedule facts that decide when a pipeline run may fire.
- Cover cron, interval, event-driven, sensor-driven, continuous, and manual cadence kinds.
- Encode tenant quota, tenant_class capacity, home_cell, pack overlay, drift hold, and cost budget before each fire.
- Give Foundry principals a Cedar-bound scheduling lane without bypassing human operator constraints.
- Close the audit row where scheduling was delegated without local bounded-context evidence.
- Treat schedule fire as an audit event and a cost event.
- Use ADR-MS-001 lineage-first constraints by refusing schedules that would publish unverifiable outputs.
- No writes outside microservices/data-pipeline/ are needed for the plan.

## Interfaces
- REST command `POST /data-pipeline/actions/schedule.define`.
- REST command `POST /data-pipeline/actions/schedule.arm`.
- REST command `POST /data-pipeline/actions/schedule.fire`.
- REST command `POST /data-pipeline/actions/schedule.pause`.
- REST command `POST /data-pipeline/actions/schedule.retire`.
- REST command `POST /data-pipeline/actions/schedule.resolve-sensor`.
- gRPC service `PipelineScheduleControl`.
- Event topic `oya.data.pipeline.schedule.*`.
- Capability records `capabilities/schedule-arm.yaml`, `schedule-fire.yaml`, and `schedule-pause.yaml`.
- Policy fragments `policies/local-schedule-mutation-scope.cedar` and `policies/local-schedule-fire-scope.cedar`.
- Contract `contracts/workflow-template-schedule-trigger-v1.yaml` for workflow-engine handoff.
- Runbooks `runbooks/schedule-missed-tick.md` and `runbooks/schedule-continuous-lease-expired.md`.
- SLO projection `slos/local-schedule-fire-jitter.openslo.yaml`.

## Data Flow
- Operator or Foundry scheduler defines a schedule with cadence_kind, cadence_expression, workflow_template_id, tenant_id, tenant_class, and home_cell.
- Cedar validates tenant scope, actor audience, pack overlay, tenant quota, and schedule mutation authority.
- Schedule state is stored append-only as armed, paused, missed, deferred, retired, or rolled_back.
- HLC-backed clock tick evaluates cron and interval schedules.
- Event matcher evaluates AsyncAPI messages for event schedules.
- Sensor worker evaluates object, table, upstream job, or drift-case predicates.
- Continuous schedule opens a lease and renews it by schedule.continuous.lease_renewed.
- Manual schedule fires once with idempotency_seed.
- Fire request checks IP-017 cost budget, IP-018 capacity, IP-026 drift status, and IP-030 watermark readiness.
- Accepted fire emits workflow handoff to workflow-engine through HTTP/3/gRPC contract.
- Workflow-engine returns workflow_run_id; data-pipeline records scheduled_run_instance.
- Downstream IP-031 destination loads and IP-035 materializations consume the scheduled_run_instance.

## Cedar Policy
- Deny schedule.define without tenant scope.
- Deny schedule.define if cadence_kind is unsupported.
- Deny schedule.amend unless actor is schedule owner, steward, or authorized Foundry lane.
- Deny schedule.arm when tenant quota is exhausted.
- Deny schedule.fire when tenant_class capacity cap for the cadence is exceeded.
- Deny schedule.fire when pack overlay blocks the chosen home_cell.
- Deny schedule.fire when source drift case from IP-026 is open.
- Deny schedule.fire when IP-017 cost budget is tripped.
- Deny schedule.fire when HLC drift exceeds tolerance.
- Deny continuous lease renewal above threshold without human approval.
- Deny sensor cadence below the 30-second floor.
- Deny all schedule mutation during audit-chain outage.

## Event Shapes
- `oya.data.pipeline.schedule.defined` carries tenant_id, tenant_class, schedule_id, cadence_kind, workflow_template_id, owner_principal_id.
- `oya.data.pipeline.schedule.armed` carries next_planned_fire_hlc, quota_snapshot_id, policy_decision_id.
- `oya.data.pipeline.schedule.fired` carries fire_tick_hlc, scheduled_run_id, workflow_run_seed, workflow_run_id, idempotency_seed.
- `oya.data.pipeline.schedule.missed` carries missed_tick_hlc, missed_reason, catch_up_allowed, runbook_url.
- `oya.data.pipeline.schedule.paused` carries paused_reason, paused_by, resume_condition.
- `oya.data.pipeline.schedule.retired` carries retirement_reason, grace_window_days, replacement_schedule_id.
- `oya.data.pipeline.schedule.continuous.lease_renewed` carries lease_id, lease_renew_interval_s, expires_at_hlc.
- Every event includes traceparent, audit_event_id, cedar_decision_id, tenant_class, and home_cell.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for schedule control plane.
- Reuse `write-latency.openslo.yaml` target 0.999 for schedule mutations.
- Reuse `read-latency.openslo.yaml` target 0.999 for schedule status reads.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for fire authorization.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for fire evidence.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for source-ready schedules.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 to keep drift holds timely.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 for scheduled transform runs.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 for scheduled output.
- Reuse `replay-freshness.openslo.yaml` target 0.999 for catch-up and replay fires.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for failed scheduled runs.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for scheduled quality gates.
- Add `local-schedule-fire-jitter.openslo.yaml`: cron p95 5s, interval p95 5s, event p95 2s, sensor p95 30s, manual p95 2s.

## Failure Modes
- Cron tick missed due to process restart records schedule.missed with catch-up option.
- Interval schedule with long prior run defers unless overlap_allowed is true.
- Event subscription lost pauses schedule and opens operator remediation.
- Sensor poll failure backs off and records missed predicate evidence.
- Continuous lease expiration pauses the stream and refuses silent restart.
- HLC drift freezes fire until clock state is reconciled.
- Drift quarantine from IP-026 blocks run before workflow handoff.
- Cost budget from IP-017 blocks run before compute allocation.
- Capacity admission from IP-018 denies concurrent run pressure.
- Audit-chain outage holds mutation and fire.
- Cedar outage fails closed.
- Workflow-engine unavailable records fire deferred, not fired.

## Migration
- Declare schedule in manifest bounded contexts.
- Keep existing workflow-engine scheduling references as dependency handoff, not authority.
- Backfill current recurring pipeline runs into pipeline_schedule rows.
- Convert ad hoc backfills into manual cadence schedules with one fire.
- Wrap existing cron-like behavior behind schedule.define and schedule.fire.
- Introduce event and sensor cadences after cron and interval are stable.
- Introduce continuous lease last because it has the largest operational blast radius.
- Add tenant_class fields to all schedule events.
- Retire any customer-facing tier language in schedule docs.
- Root IP-032 remains historical evidence; this file is the handoff artifact.
- All schedule migrations are append-only; no historical fire is deleted.
- Workflow-engine contract is versioned before schedule.fire becomes mandatory.

## Dependencies
- IP-001 tenant scope kernel supplies schedule TenantScope.
- IP-002 Cedar default deny supplies schedule policy posture.
- IP-003 ontology projection can be a scheduled downstream consumer.
- IP-004 workflow template library supplies workflow_template_id.
- IP-005 REST contract surface publishes schedule commands.
- IP-006 async event surface publishes schedule events.
- IP-007 gRPC internal surface publishes workflow handoff.
- IP-008 policy eval library binding evaluates schedule Cedar.
- IP-009 credential sidecar binding supplies source credentials at fire.
- IP-010 multi-region cell layout defines home_cell.
- IP-011 observability audit events emits fire evidence.
- IP-012 abuse defence guards exposed schedule mutation paths.
- IP-013 emergency services bypass cannot bypass schedule policy.
- IP-014 marketplace DealSet gates licensed scheduled connectors.
- IP-015 data residency overlays constrain schedule cell.
- IP-016 backfill replay worker consumes manual and catch-up fire.
- IP-017 cost budget enforcer denies over-budget fires.
- IP-018 capacity admission control denies excess concurrent fires.
- IP-019 SDK generation exposes schedule control.
- IP-020 catalog layer registration catalogs schedule capability.
- IP-021 SLO-gated promotion blocks schedule rollout on burn.
- IP-022 chaos drill pack tests missed and duplicated ticks.
- IP-023 DPIA evidence packet records scheduled data movement.
- IP-024 threat model control map maps schedule abuse.
- IP-025 audit findings closeout proves schedule finding closure.
- IP-026 schema drift quarantine blocks unsafe fires.
- IP-027 lineage graph reconciliation proves scheduled outputs.
- IP-028 dead-letter replay custody owns scheduled failure replay.
- IP-029 transform cost attribution records scheduled transform cost.
- IP-030 CDC freshness watermark governance gates source readiness.

## ADR-MS-001 Binding
- Schedule.fire cannot publish ActionAccepted until policy, idempotency, lineage preconditions, and audit target validation are satisfied.
- Schedules that produce transform output inherit lineage-before-commit.
- Backfill fires preserve original event ids and use new replay ids.
- Schedule metrics avoid raw tenant identifiers.
- Schedule run records are append-only.

## Acceptance Gates
- Gate 1: schedule appears in manifest bounded_contexts.
- Gate 2: workflow-template schedule trigger contract is published.
- Gate 3: all six cadence kinds have domain tests.
- Gate 4: Cedar denies cross-tenant schedule mutation.
- Gate 5: IP-017, IP-018, and IP-026 gates run before workflow handoff.
- Gate 6: missed tick runbook exists.
- Gate 7: continuous lease expiration runbook exists.
- Gate 8: schedule events include tenant_class.
- Gate 9: all 12 existing OpenSLOs are cited in promotion checklist.
- Gate 10: local-schedule-fire-jitter SLO is filed.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit scheduling gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-032-scheduling.md:38` - - SLO projection `slos/local-schedule-fire-jitter.openslo.yaml`.; `microservices/data-pipeline/implementation-plans/IP-032-scheduling.md:78` - ## SLO Targets.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-032-scheduling.md:17` - - Encode tenant quota, tenant_class capacity, home_cell, pack overlay, drift hold, and cost budget before each fire.; `microservices/data-pipeline/implementation-plans/IP-032-scheduling.md:21` - - Treat schedule fire as an audit event and a cost event..
