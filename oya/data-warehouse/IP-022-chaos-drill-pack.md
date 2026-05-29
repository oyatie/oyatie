---
doc_class: IP
ip_id: IP-022
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-022: Chaos Drill Pack

## Context
- DW22-CTX-01: This IP defines data-warehouse chaos drills that prove query, catalog, replay, budget, and settlement recovery.
- DW22-CTX-02: Snowflake outage drills become local warehouse endpoint and catalog fallback tests.
- DW22-CTX-03: BigQuery quota drills become admission and budget exhaustion tests.
- DW22-CTX-04: Redshift WLM saturation drills become workload-pool queue and preemption tests.
- DW22-CTX-05: Databricks SQL cold-start drills become warm-pool and SLO gate tests.
- DW22-CTX-06: Synapse Analytics control-plane latency drills become migration adapter timeout tests.
- DW22-CTX-07: Firebolt engine-start failure drills become reservation-pool failover tests.
- DW22-CTX-08: ClickHouse Cloud replica lag drills become freshness and query-routing tests.
- DW22-CTX-09: Vertica depot failure drills become storage/cache recovery tests.
- DW22-CTX-10: Teradata Vantage workload throttle drills become budget and capacity enforcement tests.
- DW22-CTX-11: Yellowbrick cluster queue drills become fixed-capacity admission tests.
- DW22-CTX-12: Drills are tenant-scoped and cannot mutate production data unless a break-glass workflow approves.
- DW22-CTX-13: Every drill has stop conditions, rollback, audit events, and SLO assertions.
- DW22-CTX-14: Drill evidence feeds SLO promotion and audit findings closeout.
- DW22-CTX-15: Chaos scenarios must prove local resilience, not vendor failover documentation.

## Data Model Deltas
- DW22-DDL-01: Add chaos drill definition table.
```sql
CREATE TABLE warehouse_chaos_drill_definitions (
    drill_id UUID PRIMARY KEY,
    tenant_id UUID,
    drill_name TEXT NOT NULL,
    target_surface TEXT NOT NULL CHECK (target_surface IN ('query','catalog','capacity','budget','backfill','settlement','residency','sdk')),
    blast_radius TEXT NOT NULL CHECK (blast_radius IN ('synthetic','single_tenant','single_cell','global_shadow')),
    vendor_scenario TEXT,
    enabled BOOLEAN NOT NULL DEFAULT false,
    max_duration_seconds INTEGER NOT NULL CHECK (max_duration_seconds BETWEEN 30 AND 7200),
    rollback_plan_ref TEXT NOT NULL,
    audit_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_chaos_drill_target_idx ON warehouse_chaos_drill_definitions(target_surface, enabled);
```
- DW22-DDL-02: Add chaos drill run table.
```sql
CREATE TABLE warehouse_chaos_drill_runs (
    run_id UUID PRIMARY KEY,
    drill_id UUID NOT NULL REFERENCES warehouse_chaos_drill_definitions(drill_id),
    tenant_id UUID,
    started_by_principal_id UUID NOT NULL,
    run_status TEXT NOT NULL CHECK (run_status IN ('scheduled','running','passed','failed','aborted','rolled_back')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    slo_assertions JSONB NOT NULL DEFAULT '[]',
    failure_observations JSONB NOT NULL DEFAULT '[]',
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL
);
CREATE INDEX wh_chaos_run_status_idx ON warehouse_chaos_drill_runs(run_status, started_at DESC);
```
- DW22-RUST-01: Drill definition type.
```rust
pub struct WarehouseChaosDrillDefinition {
    pub drill_id: ChaosDrillId,
    pub tenant_id: Option<TenantId>,
    pub drill_name: DrillName,
    pub target_surface: WarehouseChaosSurface,
    pub blast_radius: BlastRadius,
    pub vendor_scenario: Option<VendorScenario>,
    pub enabled: bool,
    pub max_duration: Duration,
    pub rollback_plan_ref: RollbackPlanRef,
    pub audit_event_id: AuditEventId,
}
```
- DW22-RUST-02: Drill run type.
```rust
pub struct WarehouseChaosDrillRun {
    pub run_id: ChaosRunId,
    pub drill_id: ChaosDrillId,
    pub tenant_id: Option<TenantId>,
    pub started_by_principal_id: PrincipalId,
    pub run_status: ChaosRunStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub slo_assertions: Vec<SloAssertion>,
    pub failure_observations: Vec<FailureObservation>,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
}
```
- DW22-RUST-03: `BlastRadius::GlobalShadow` can only affect synthetic or shadow traffic.
- DW22-RUST-04: `ChaosRunStatus::Passed` requires every SLO assertion to be true.
- DW22-RUST-05: Drill definitions are immutable after enablement; changes create a new definition.

## API Endpoints
- DW22-API-01: REST schedule drill.
```http
POST /v1/data-warehouse/chaos/drills/{drill_id}/runs
Idempotency-Key: wh-chaos-run-022
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","requested_blast_radius":"single_tenant","scheduled_at":"2026-05-20T22:00:00Z","stop_on_first_slo_failure":true}
```
- DW22-API-02: REST abort drill.
```http
POST /v1/data-warehouse/chaos/runs/{run_id}:abort
Content-Type: application/json

{"abort_reason":"query_latency_p99_above_stop_condition","rollback_now":true}
```
- DW22-API-03: gRPC drill runner.
```proto
rpc RunWarehouseChaosDrill(RunWarehouseChaosDrillRequest) returns (RunWarehouseChaosDrillResponse);
message RunWarehouseChaosDrillRequest {
  string drill_id = 1;
  string tenant_id = 2;
  string requested_blast_radius = 3;
  bool stop_on_first_slo_failure = 4;
}
```
- DW22-API-04: AsyncAPI event.
```yaml
warehouse.chaos.drill.run.failed.v1:
  payload:
    run_id: 01JWH22RUN
    drill_name: budget-hard-limit-denial
    failed_slo: warehouse_capacity_admission_p99
    audit_event_class: WarehouseChaosDrillRunFailed
```
- DW22-API-05: REST schedule returns 403 if Cedar rejects blast radius.
- DW22-API-06: gRPC abort returns final rollback status.
- DW22-API-07: Async passed events attach evidence bundle refs.

## Cedar Policy Hooks
- DW22-CEDAR-01: principal = `Oyatie::Principal::"resilience_engineer:{principal_id}"`.
- DW22-CEDAR-02: action = `Oyatie::Action::"warehouse_chaos_drill_run"`.
- DW22-CEDAR-03: resource = `Oyatie::WarehouseChaosDrill::"{drill_id}"`.
- DW22-CEDAR-04: context.blast_radius must not exceed drill definition blast radius.
- DW22-CEDAR-05: context.tenant_id is required for single-tenant drills.
- DW22-CEDAR-06: context.rollback_plan_ref must exist and be current.
- DW22-CEDAR-07: context.stop_conditions_declared must be true.
- DW22-CEDAR-08: context.production_mutation must be false unless break-glass approval exists.
- DW22-CEDAR-09: context.audit_event_class must equal `WarehouseChaosDrillRunStarted`.
- DW22-CEDAR-10: deny if principal lacks `warehouse.chaos.run`.

## Ontology Projection
- DW22-ONTO-01: Snowflake incident `WAREHOUSE_SUSPENDED` -> `WarehouseChaosScenario.capacity_unavailable`.
- DW22-ONTO-02: BigQuery `quotaExceeded` -> `WarehouseChaosScenario.quota_exhausted`.
- DW22-ONTO-03: Redshift WLM queue saturation -> `WarehouseChaosScenario.queue_saturated`.
- DW22-ONTO-04: Databricks SQL cold start -> `WarehouseChaosScenario.warm_pool_unavailable`.
- DW22-ONTO-05: Synapse control-plane delay -> `WarehouseChaosScenario.control_plane_slow`.
- DW22-ONTO-06: Firebolt engine start timeout -> `WarehouseChaosScenario.engine_start_failed`.
- DW22-ONTO-07: ClickHouse Cloud replica lag -> `WarehouseChaosScenario.replica_stale`.
- DW22-ONTO-08: Vertica depot unavailable -> `WarehouseChaosScenario.cache_depot_unavailable`.
- DW22-ONTO-09: Teradata workload throttle -> `WarehouseChaosScenario.workload_throttled`.
- DW22-ONTO-10: Yellowbrick queue saturation -> `WarehouseChaosScenario.fixed_capacity_exhausted`.
- DW22-ONTO-11: Vendor incident id -> `WarehouseChaosDrillRun.vendor_scenario_evidence`.
- DW22-ONTO-12: Local stop condition -> `WarehouseChaosDrillDefinition.stop_condition`.

## Workflow Steps
- DW22-WF-01: Node `SelectDrill` loads enabled drill definition.
- DW22-WF-02: Node `EvaluateBlastRadius` confirms tenant and cell scope.
- DW22-WF-03: Node `EvaluatePolicy` runs Cedar against the requested run.
- DW22-WF-04: Branch `PolicyDenied` refuses schedule and emits denial.
- DW22-WF-05: Node `ArmStopConditions` subscribes to SLO and error-budget metrics.
- DW22-WF-06: Node `StartFault` injects synthetic delay, denial, saturation, or stale replica.
- DW22-WF-07: Branch `StopConditionHit` aborts and rolls back immediately.
- DW22-WF-08: Node `ObserveRecovery` records p50, p95, p99, throughput, and availability.
- DW22-WF-09: Node `RollbackFault` restores baseline configuration.
- DW22-WF-10: Node `AssertSlo` marks run passed or failed.
- DW22-WF-11: Node `EmitAudit` emits run completion event.
- DW22-WF-12: Node `PublishEvidence` sends results to SLO gate and audit closeout.

## Audit Events
- DW22-AUDIT-01: `WarehouseChaosDrillScheduled` records drill and requested time.
- DW22-AUDIT-02: `WarehouseChaosDrillRunStarted` records blast radius and stop conditions.
- DW22-AUDIT-03: `WarehouseChaosFaultInjected` records fault type and target surface.
- DW22-AUDIT-04: `WarehouseChaosStopConditionTriggered` records metric and threshold.
- DW22-AUDIT-05: `WarehouseChaosRollbackCompleted` records rollback duration.
- DW22-AUDIT-06: `WarehouseChaosDrillRunPassed` records evidence bundle.
- DW22-AUDIT-07: `WarehouseChaosDrillRunFailed` records failed SLO and observations.

## SLO Targets
- DW22-SLO-01: p50 drill scheduler latency <= 50 ms.
- DW22-SLO-02: p95 drill scheduler latency <= 250 ms.
- DW22-SLO-03: p99 drill scheduler latency <= 700 ms.
- DW22-SLO-04: throughput >= 30 scheduled drills per minute.
- DW22-SLO-05: availability >= 99.9 percent for drill scheduling.
- DW22-SLO-06: stop-condition reaction p95 <= 5 seconds.
- DW22-SLO-07: rollback completion p95 <= 60 seconds for synthetic faults.
- DW22-SLO-08: production data mutation count must be 0 without break-glass approval.

## Failure Modes + Recovery
- DW22-FAIL-01: Stop condition monitor fails; abort drill and roll back fault immediately.
- DW22-FAIL-02: Fault injection partially applies; execute rollback plan and mark run failed.
- DW22-FAIL-03: Metrics stream lags; extend observation only within max duration, otherwise fail run.
- DW22-FAIL-04: Rollback fails; escalate incident, freeze matching drill definition, and block promotions.
- DW22-FAIL-05: Blast radius exceeds request; terminate run and emit safety violation event.
- DW22-FAIL-06: Evidence publication fails; keep run status pending evidence and retry outbox.

## Migration Notes
- DW22-MIG-01: Snowflake warehouse suspension informs local capacity-unavailable drills.
- DW22-MIG-02: BigQuery quotaExceeded responses inform budget and admission denial drills.
- DW22-MIG-03: Redshift WLM saturation informs queue backpressure drills.
- DW22-MIG-04: Databricks SQL cold starts inform warm-pool drills.
- DW22-MIG-05: Synapse Analytics control-plane delays inform timeout drills.
- DW22-MIG-06: Firebolt engine start failures inform reservation pool failover drills.
- DW22-MIG-07: ClickHouse Cloud replica lag informs freshness routing drills.
- DW22-MIG-08: Vertica depot failures inform cache recovery drills.
- DW22-MIG-09: Teradata workload throttles inform workload class drills.
- DW22-MIG-10: Yellowbrick queue saturation informs fixed-capacity drills.

## Cross-Microservice Handoffs
- DW22-HANDOFF-01: Observability receives drill metrics and stop-condition subscriptions.
- DW22-HANDOFF-02: SLO gate receives pass/fail evidence for promotions.
- DW22-HANDOFF-03: Audit-chain receives ADR-0263 drill events.
- DW22-HANDOFF-04: Workflow receives rollback and incident escalation branches.
- DW22-HANDOFF-05: Tenant-admin receives drill notices and evidence summaries.
- DW22-HANDOFF-06: Query planner receives synthetic capacity and routing faults.
- DW22-HANDOFF-07: Cost-budget receives quota and budget exhaustion drill outcomes.
- DW22-HANDOFF-08: Catalog receives stale/failure surface drill state.
- DW22-HANDOFF-09: Policy receives Cedar blast-radius decision evidence.
- DW22-HANDOFF-10: Incident-management receives failed rollback escalations.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-022-chaos-drill-pack.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
