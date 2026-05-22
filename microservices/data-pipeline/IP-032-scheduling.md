# IP-032 Data Pipeline scheduling bounded context

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-032-scheduling.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2 (scheduling thin), §3.9.3
Benchmarks: Fivetran (sync frequency presets 5m/15m/1h/24h), Airbyte (cron + manual + webhook), dbt Cloud (jobs with cron and event triggers), Estuary Flow (continuous), Hevo (real-time pipelines)
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0251, ADR-0252, ADR-0253, ADR-0254, ADR-0255, ADR-0314, ADR-0321, ADR-0329, ADR-0330, ADR-0331

## Objective
- Promote `schedule` from an implicit, workflow-engine-delegated capability to a named bounded context inside data-pipeline.
- Keep `workflow-engine` as the orchestrator (delegation rule), but record the scheduling policy, cadence, trigger envelope, and tenant-scoped quota at the data-pipeline layer where pipeline ownership lives.
- Cover cron-style cadence, event-driven triggers, manual fire-once, sensor-driven (file landed, message received, upstream completed) triggers, and continuous (streaming) scheduling on a single primitive.
- Make Foundry agents (ADR-0247 oyatie.foundry.*) able to propose, claim, and run scheduled pipelines under Cedar, mirroring human operator behavior.
- Make scheduling cost-attributable per tenant + cadence + cell + pack so capacity-admission control (IP-018) can throttle on a per-tenant basis.

## Delegation rule
- The `schedule` bounded context owns: schedule definition, cadence resolution, trigger evaluation, tenant quota, Cedar policy on schedule mutation, scheduled-run audit evidence, schedule-driven cost attribution.
- The `workflow-engine` microservice owns: the orchestration of each fired run (step ordering, retry, escalation, human-in-the-loop). The schedule fires a workflow template; the workflow runs it.
- The boundary is published in `contracts/workflow-template-schedule-trigger-v1.yaml` (see audit §5.3) and never crossed silently.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §A, §C, §F (UX flows).
- Read `microservices/data-pipeline/ARCHITECTURE.md` §C (bounded contexts) and §D (workflow-engine dependency row).
- Read `microservices/data-pipeline/IP-018-capacity-admission-control.md` for tenant quota interplay.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2 (scheduling missing), §5.3 (workflow-engine boundary).
- Read `microservices/data-pipeline/IP-017-cost-budget-enforcer.md` for cost dimension binding.
- Read `microservices/data-pipeline/IP-002-cedar-default-deny.md` for default-deny posture.
- Read `microservices/data-pipeline/IP-031-destination-connector.md` (this wave) so a scheduled pipeline can include destination_load_run.

## Domain model
- Aggregate: `pipeline_schedule`.
- Identity: `tenant_id + schedule_id`.
- Sub-aggregate: `schedule_trigger_binding` (one row per trigger kind attached to the schedule).
- Sub-aggregate: `scheduled_run_instance` (one row per fired run with cadence_clock_tick, trigger_event_id, workflow_template_id, workflow_run_id correlation).
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR`, `oyatie.foundry.scheduler`, or `oyatie.foundry.pipeline_operator` audience.
- Required policy decision: Cedar permit from `local-schedule-mutation-scope.cedar` and `local-schedule-fire-scope.cedar`.
- Required cadence: `cron`, `interval`, `event`, `sensor`, `continuous`, `manual`.
- Required trigger envelope: tenant, principal, idempotency_seed, cell, pack, audit chain target.
- Required quota: per-tenant max-concurrent runs, per-cadence floor, per-cell concurrency cap.
- Required cost dimensions: scheduled_run_count, cadence_cpu_seconds, cadence_queue_time_ms.
- Required disposition: armed, paused, fired, missed, deferred, retired.

## Cadence kinds (resolves audit §3.9.2 missing-primitive)
- `cron`: standard 5-field cron (minute hour day-of-month month day-of-week), tenant-local timezone, ADR-0252 HLC-stamped tick.
- `interval`: fixed interval (e.g., every 5 minutes); aligns Fivetran 5m/15m/1h/24h preset surface.
- `event`: triggered by an AsyncAPI event match expression (e.g., `oya.cloud.objects.object_landed`, `oya.workflow.run.completed`, `oya.data.warehouse.table_loaded`).
- `sensor`: poll-based (object exists, table row count > threshold, upstream schedule fired, drift case resolved).
- `continuous`: streaming-style perpetual run (analogue to Estuary Flow continuous capture); modeled as a single long-running scheduled_run_instance with renewable lease.
- `manual`: explicit operator or Foundry fire; required for one-off backfills.

## Trigger evaluation rules
- Cron tick fires once per tenant timezone cron expression; HLC tick prevents double-fire on clock skew.
- Interval tick fires only after the previous run reaches a terminal state, unless `overlap_allowed = true` is set on the schedule.
- Event match uses CloudEvents 1.0 attributes + AsyncAPI message body filter; events must carry tenant_id + principal_id + cell to qualify.
- Sensor polls run at a tenant-configurable cadence with a floor (minimum 30 seconds) to prevent quota burn.
- Continuous schedules emit lease-renewal events every `lease_renew_interval_s`; missed renewal moves the schedule to `missed` then `paused`.
- Manual fires require Cedar permit + idempotency_seed; no double-fire on retried operator click.

## Implementation steps
- Add `schedule` to `manifest.json` `bounded_contexts`.
- Add ARCHITECTURE.md §C entry naming `pipeline_schedule` aggregate.
- Add `src/domain/schedule.rs` with `PipelineSchedule`, `ScheduleTriggerBinding`, `Cadence` enum, `ScheduleDisposition` enum.
- Add `src/usecase/schedule.rs` exposing `schedule.define`, `schedule.amend`, `schedule.arm`, `schedule.pause`, `schedule.fire`, `schedule.retire`, `schedule.resolve_sensor`, `schedule.renew_continuous_lease`.
- Add `src/adapter/schedule_clock.rs` (HLC-stamped tick source per ADR-0252).
- Add `src/worker/schedule_tick.rs` (single tenant home cell process per tenant).
- Add `local-schedule-mutation-scope.cedar` and `local-schedule-fire-scope.cedar` to `policies/`.
- Add `oya.data.pipeline.schedule.armed`, `schedule.fired`, `schedule.missed`, `schedule.paused`, `schedule.retired`, `schedule.continuous.lease_renewed` to AsyncAPI surface.
- Add `capabilities/schedule-arm.yaml`, `capabilities/schedule-fire.yaml`, `capabilities/schedule-pause.yaml`.
- Add `catalog/oya-data-pipeline-schedule-domain.yaml`.
- Add SLO `local-schedule-fire-jitter.openslo.yaml` (target p95 fire-after-tick latency: cron 5s, interval 5s, event 2s, sensor 30s, continuous 0s, manual 2s).
- Add runbook `schedule-missed-tick.md` for cadence misses and `schedule-continuous-lease-expired.md` for streaming lease failure.
- Wire `schedule.fire` to dispatch `workflow_template_id + workflow_run_seed` to workflow-engine via the contracted gRPC over HTTP/3 surface (ADR-0253).
- Publish `contracts/workflow-template-schedule-trigger-v1.yaml` as the cross-microservice contract closing audit §5.3.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `schedule_id` is mandatory.
- `cadence_kind` is mandatory.
- `cadence_expression` is mandatory (cron string, interval seconds, event match expr, sensor predicate, continuous lease params, or manual).
- `tenant_timezone` is mandatory for cron.
- `overlap_allowed` is mandatory.
- `quota_per_tenant_concurrent` is mandatory.
- `quota_per_cell_concurrent` is mandatory.
- `cedar_decision_id` is mandatory on mutation and fire.
- `audit_event_id` is mandatory on every disposition transition.
- `last_fire_tick_hlc` is mandatory after first fire.
- `next_planned_fire_hlc` is mandatory while armed.
- `lease_renew_interval_s` is mandatory for continuous.
- `cost_dimensions` is mandatory (scheduled_run_count, cadence_cpu_seconds, cadence_queue_time_ms).

## Foundry integration (resolves audit §3.5)
- `oyatie.foundry.scheduler` may propose schedules via `schedule.define`.
- `oyatie.foundry.pipeline_operator` may amend/arm/pause schedules.
- Foundry-fired schedules require Cedar permit just like human-fired schedules — no privileged bypass.
- Foundry schedule mutation emits an additional `principal.foundry_lane` evidence field for ADR-0247 lane attribution.
- Foundry continuous schedules require human approval for `lease_renew_interval_s > 3600` to keep agentic loops bounded.

## Policy gates
- Cedar denies schedule.define without tenant scope.
- Cedar denies schedule.amend if amender is not the schedule owner or delegated steward.
- Cedar denies schedule.arm if tenant quota is exhausted.
- Cedar denies schedule.fire if pack overlay blocks the cell (e.g., KR-PIPA prohibits cross-jurisdiction fire).
- Cedar denies schedule.fire if upstream IP-026 drift case is open for any source object the workflow template references.
- Cedar denies schedule.fire if cost-budget enforcer (IP-017) has tripped the tenant budget.
- Cedar denies schedule.fire if audit-chain is unavailable.
- Cedar denies schedule.fire if HLC drift exceeds the tolerance window (ADR-0252).
- Cedar denies continuous lease renewal beyond `lease_renew_interval_s_max` without operator approval.
- Cedar denies sensor poll cadence below the 30s floor.

## Benchmark displacement
- Fivetran sync-frequency preset parity: oyatie schedule `interval` cadence covers 5m/15m/1h/24h plus custom intervals.
- Airbyte schedule parity: cron + manual + webhook coverage by `cron`, `manual`, and `event` cadences.
- dbt Cloud jobs parity: `cron` + `event` cadences for dbt-job-style scheduling; `sensor` cadence covers deferred-state runs.
- Estuary Flow continuous parity: `continuous` cadence with lease renewal.
- Hevo real-time pipeline parity: `continuous` cadence + `event` cadence over change-data-capture sources.
- Vendor-specific names (e.g., Fivetran "Premium plan sync frequency") do not become canonical schedule names; they map to `interval` rows.

## Failure handling
- If cron tick is missed (process restart, cell evict), emit `schedule.missed` with the missed HLC tick and offer operator-driven catch-up via `schedule.fire` with `catch_up = true`.
- If sensor poll fails, retry with backoff and log to `runbooks/schedule-missed-tick.md`.
- If event subscription is severed, emit `schedule.paused` with `paused_reason = event_subscription_lost` and require operator reconnect.
- If continuous lease expires, emit `schedule.continuous.lease_expired` and follow `runbooks/schedule-continuous-lease-expired.md`.
- If Cedar is unavailable, fail closed for fire; arm/pause are also denied until Cedar recovers.
- If audit-chain is unavailable, hold fire; emit degraded banner.
- If HLC drifts beyond tolerance, hold fire and route to `runbooks/replay-cursor-rollback.md` (HLC anchor verification).

## Tests and evidence
- Unit test: cron parser rejects invalid expressions.
- Unit test: HLC tick monotonic across restarts.
- Contract test: schedule.define rejects missing cadence_kind.
- Contract test: schedule.fire rejects missing workflow_template_id.
- Policy test: cross-tenant schedule mutation denied.
- Policy test: foundry scheduler denied for tenant that has not opted into foundry lane.
- Capacity test: tenant quota cap enforced at fire time.
- Replay test: missed cron tick can be backfilled via catch_up command.
- SLO test: local-schedule-fire-jitter burn opens runbook link.
- Audit test: define, arm, fire, pause, retire all share schedule correlation id.

## Rollback
- Roll back schedule mutation by creating a `rolled_back` amendment record (schedule definitions are append-only).
- Pause active schedules during rollback to prevent fires during the rollback window.
- Recompute `next_planned_fire_hlc` after rollback applies.
- Preserve audit-chain evidence for every fired run before rollback.
- Notify workflow-engine to stop dispatching for rolled-back schedule ids.
- Link rollback to `runbooks/schedule-missed-tick.md`.

## Acceptance criteria
- `schedule` is listed in manifest.json `bounded_contexts`.
- ARCHITECTURE.md §C declares `pipeline_schedule` aggregate.
- PRD §C and §D enumerate schedule commands.
- All six cadence kinds (cron, interval, event, sensor, continuous, manual) have a domain test and a cedar policy test.
- `contracts/workflow-template-schedule-trigger-v1.yaml` is published with workflow-engine sign-off.
- Foundry principals can schedule and fire under Cedar.
- IP-018 capacity admission control consumes schedule quota fields.
- IP-017 cost-budget enforcer consumes schedule cost dimensions.
- Cross-microservice finding audit §5.3 is resolved: workflow-template schedule trigger contract is published.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2, §5.3.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` §7 scheduling.
- `microservices/data-pipeline/IP-018-capacity-admission-control.md` quota.
- `microservices/data-pipeline/IP-017-cost-budget-enforcer.md` cost dimensions.
- `microservices/data-pipeline/IP-031-destination-connector.md` destination commit.
- `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` continuous watermark.
- `ADR-0247` Foundry under Cedar.
- `ADR-0248` cell topology.
- `ADR-0252` HLC default for cadence tick.
- `ADR-0253` HTTP/3 for cross-microservice dispatch.
- `ADR-0314` DealSet (when a scheduled run depends on a licensed connector).
- `ADR-0321` documentation-rigor.

## Operator review prompts
- Reviewer asks whether cadence_kind matches operational intent.
- Reviewer asks whether tenant timezone is the intended schedule timezone.
- Reviewer asks whether overlap_allowed is correct.
- Reviewer asks whether tenant quota is sufficient and not exhausted.
- Reviewer asks whether pack overlay permits the schedule cell.
- Reviewer asks whether upstream drift cases are clear before arming.
- Reviewer asks whether cost-budget headroom permits the cadence.
- Reviewer asks whether HLC clock state is healthy.
- Reviewer asks whether continuous lease renewal fits the operational window.
- Reviewer signs the schedule case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `KR-PIPA-2023-amendment` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=semi-annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-032-scheduling.md:71` - - Add SLO `local-schedule-fire-jitter.openslo.yaml` (target p95 fire-after-tick latency: cron 5s, interval 5s, event 2s, sensor 30s, continuous 0s, manual 2s).; `microservices/data-pipeline/IP-032-scheduling.md:138` - - SLO test: local-schedule-fire-jitter burn opens runbook link..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-032-scheduling.md:15` - - Make scheduling cost-attributable per tenant + cadence + cell + pack so capacity-admission control (IP-018) can throttle on a per-tenant basis.; `microservices/data-pipeline/IP-032-scheduling.md:18` - - The `schedule` bounded context owns: schedule definition, cadence resolution, trigger evaluation, tenant quota, Cedar policy on schedule mutation, scheduled-run audi....
