---
doc_class: IP
ip_id: IP-018
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-018: Capacity Admission Control

## Context
- DW18-CTX-01: This IP admits or rejects warehouse workloads before they overload tenant pools or shared cells.
- DW18-CTX-02: Snowflake virtual warehouse sizing maps to local workload-pool slots.
- DW18-CTX-03: BigQuery reservation slots map to admission tokens and burst quotas.
- DW18-CTX-04: Redshift WLM queues map to admission lanes with concurrency caps.
- DW18-CTX-05: Databricks SQL warehouse clusters map to warm capacity pools.
- DW18-CTX-06: Synapse Analytics DWU pools map to fixed compute token budgets.
- DW18-CTX-07: Firebolt engines map to reservation pools with start latency budgets.
- DW18-CTX-08: ClickHouse Cloud services map to concurrent query and memory admission gates.
- DW18-CTX-09: Vertica resource pools map to memory and execution slots.
- DW18-CTX-10: Teradata workload management maps to workload classes and throttle rules.
- DW18-CTX-11: Yellowbrick cluster queues map to fixed capacity admission lanes.
- DW18-CTX-12: Admission uses cost estimate, tenant priority, residency, and SLO burn rate together.
- DW18-CTX-13: Admission denial is a product event, not an infrastructure crash.
- DW18-CTX-14: Capacity decisions are explainable to tenant admins and auditable under ADR-0263.
- DW18-CTX-15: The controller must shed low-priority backfill before interactive analytics.

## Data Model Deltas
- DW18-DDL-01: Add workload pool capacity state.
```sql
CREATE TABLE warehouse_capacity_pools (
    pool_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    cell_id TEXT NOT NULL,
    pool_kind TEXT NOT NULL CHECK (pool_kind IN ('interactive','batch','backfill','marketplace_share','system')),
    max_concurrent_queries INTEGER NOT NULL CHECK (max_concurrent_queries > 0),
    max_memory_mib BIGINT NOT NULL CHECK (max_memory_mib > 0),
    max_scan_bytes_per_second BIGINT NOT NULL CHECK (max_scan_bytes_per_second > 0),
    reserved_tokens INTEGER NOT NULL DEFAULT 0 CHECK (reserved_tokens >= 0),
    active_tokens INTEGER NOT NULL DEFAULT 0 CHECK (active_tokens >= 0),
    status TEXT NOT NULL CHECK (status IN ('open','degraded','throttled','closed')),
    audit_event_id UUID NOT NULL
);
CREATE INDEX wh_capacity_pool_cell_idx ON warehouse_capacity_pools(cell_id, status, pool_kind);
```
- DW18-DDL-02: Add admission decisions.
```sql
CREATE TABLE warehouse_capacity_admission_decisions (
    decision_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    pool_id UUID NOT NULL REFERENCES warehouse_capacity_pools(pool_id),
    workload_kind TEXT NOT NULL CHECK (workload_kind IN ('query','export','backfill','share_refresh','catalog_maintenance')),
    workload_id UUID NOT NULL,
    requested_tokens INTEGER NOT NULL CHECK (requested_tokens > 0),
    estimated_memory_mib BIGINT NOT NULL,
    estimated_scan_bytes BIGINT NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('admit','queue','deny','preempt_lower_priority')),
    reason_code TEXT NOT NULL,
    policy_decision_id UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_capacity_decision_workload_idx ON warehouse_capacity_admission_decisions(tenant_id, workload_kind, workload_id);
```
- DW18-RUST-01: Capacity pool type.
```rust
pub struct WarehouseCapacityPool {
    pub pool_id: WorkloadPoolId,
    pub tenant_id: TenantId,
    pub cell_id: CellId,
    pub pool_kind: CapacityPoolKind,
    pub max_concurrent_queries: NonZeroU32,
    pub max_memory_mib: NonZeroU64,
    pub max_scan_bytes_per_second: NonZeroU64,
    pub reserved_tokens: u32,
    pub active_tokens: u32,
    pub status: CapacityPoolStatus,
    pub audit_event_id: AuditEventId,
}
```
- DW18-RUST-02: Admission decision type.
```rust
pub struct WarehouseCapacityAdmissionDecision {
    pub decision_id: AdmissionDecisionId,
    pub tenant_id: TenantId,
    pub pool_id: WorkloadPoolId,
    pub workload: WarehouseWorkloadRef,
    pub requested_tokens: NonZeroU32,
    pub estimated_memory_mib: u64,
    pub estimated_scan_bytes: u64,
    pub decision: AdmissionDecisionKind,
    pub reason_code: AdmissionReasonCode,
    pub policy_decision_id: PolicyDecisionId,
    pub expires_at: DateTime<Utc>,
}
```
- DW18-RUST-03: `AdmissionDecisionKind::Queue` carries a not-before timestamp.
- DW18-RUST-04: `AdmissionDecisionKind::PreemptLowerPriority` requires the preempted workload id list.
- DW18-RUST-05: Decision expiration prevents stale capacity tokens from authorizing late execution.

## API Endpoints
- DW18-API-01: REST admission endpoint.
```http
POST /v1/data-warehouse/capacity:admit
Idempotency-Key: wh-capacity-admit-018
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","pool_id":"01JPOOL018","workload_kind":"query","workload_id":"01JQUERY018","requested_tokens":2,"estimated_memory_mib":4096,"estimated_scan_bytes":34359738368,"priority":"interactive"}
```
- DW18-API-02: REST release endpoint.
```http
POST /v1/data-warehouse/capacity/decisions/{decision_id}:release
Content-Type: application/json

{"actual_memory_peak_mib":3584,"actual_scan_bytes":30064771072,"completed_at":"2026-05-20T19:44:00Z"}
```
- DW18-API-03: gRPC admission service.
```proto
rpc AdmitWarehouseCapacity(AdmitWarehouseCapacityRequest) returns (AdmitWarehouseCapacityResponse);
message AdmitWarehouseCapacityRequest {
  string tenant_id = 1;
  string pool_id = 2;
  string workload_kind = 3;
  string workload_id = 4;
  uint32 requested_tokens = 5;
  uint64 estimated_memory_mib = 6;
  uint64 estimated_scan_bytes = 7;
}
```
- DW18-API-04: AsyncAPI event.
```yaml
warehouse.capacity.admission.queued.v1:
  payload:
    decision_id: 01JWH18ADMIT
    pool_id: 01JPOOL018
    workload_id: 01JQUERY018
    reason_code: pool_tokens_exhausted
    audit_event_class: WarehouseCapacityAdmissionQueued
```
- DW18-API-05: REST deny returns `429 warehouse_capacity_exhausted`.
- DW18-API-06: gRPC queue returns `RESOURCE_EXHAUSTED` plus retry delay metadata.
- DW18-API-07: Async release events include actual resource usage for capacity tuning.

## Cedar Policy Hooks
- DW18-CEDAR-01: principal = `Oyatie::Principal::"warehouse_scheduler:{scheduler_id}"`.
- DW18-CEDAR-02: action = `Oyatie::Action::"warehouse_capacity_admit"`.
- DW18-CEDAR-03: resource = `Oyatie::WarehouseCapacityPool::"{pool_id}"`.
- DW18-CEDAR-04: context.tenant_id must equal pool tenant.
- DW18-CEDAR-05: context.workload_priority controls preemption eligibility.
- DW18-CEDAR-06: context.estimated_scan_bytes must fit tenant burst quota.
- DW18-CEDAR-07: context.slo_burn_rate must be below emergency throttle threshold for non-interactive work.
- DW18-CEDAR-08: context.residency_decision must be `allow`.
- DW18-CEDAR-09: context.audit_event_class must match decision event class.
- DW18-CEDAR-10: deny if principal lacks `warehouse.capacity.schedule`.

## Ontology Projection
- DW18-ONTO-01: Snowflake `WAREHOUSE_SIZE` -> `WarehouseCapacityPool.max_concurrent_queries` estimate profile.
- DW18-ONTO-02: BigQuery `Reservation.slotCapacity` -> `WarehouseCapacityPool.reserved_tokens`.
- DW18-ONTO-03: Redshift `wlm_query_slot_count` -> `WarehouseCapacityAdmissionDecision.requested_tokens`.
- DW18-ONTO-04: Databricks SQL `cluster_size` -> `WarehouseCapacityPool.max_memory_mib`.
- DW18-ONTO-05: Synapse `DWU` -> `WarehouseCapacityPool.max_scan_bytes_per_second`.
- DW18-ONTO-06: Firebolt `engine_size` -> `WarehouseCapacityPool.reserved_tokens`.
- DW18-ONTO-07: ClickHouse Cloud `max_memory_usage` -> `WarehouseCapacityPool.max_memory_mib`.
- DW18-ONTO-08: Vertica `RESOURCE_POOL.MAXMEMORYSIZE` -> `WarehouseCapacityPool.max_memory_mib`.
- DW18-ONTO-09: Teradata Vantage `ThrottleLimit` -> `WarehouseCapacityPool.max_concurrent_queries`.
- DW18-ONTO-10: Yellowbrick `resource_group` -> `WarehouseCapacityPool.pool_kind`.
- DW18-ONTO-11: Vendor queue wait time -> `WarehouseCapacityTelemetry.queue_wait_ms`.
- DW18-ONTO-12: Local priority -> `WarehouseWorkload.priority_class`.

## Workflow Steps
- DW18-WF-01: Node `EstimateResources` receives planner estimate.
- DW18-WF-02: Node `ResolvePool` maps workload to tenant pool and cell.
- DW18-WF-03: Node `CheckResidency` confirms cell eligibility.
- DW18-WF-04: Node `EvaluatePolicy` runs Cedar with priority and SLO burn.
- DW18-WF-05: Branch `DenyForPolicy` returns denial with audit event.
- DW18-WF-06: Node `ReadPoolState` locks pool capacity row.
- DW18-WF-07: Branch `EnoughTokens` admits and increments active tokens.
- DW18-WF-08: Branch `QueueAllowed` records queued decision and retry delay.
- DW18-WF-09: Branch `PreemptBackfill` cancels lower-priority backfill and admits interactive work.
- DW18-WF-10: Branch `DenyExhausted` returns capacity exhausted.
- DW18-WF-11: Node `ReleaseTokens` decrements active tokens on completion.
- DW18-WF-12: Node `TunePool` updates capacity profile from actual usage.

## Audit Events
- DW18-AUDIT-01: `WarehouseCapacityAdmissionRequested` records estimate and priority.
- DW18-AUDIT-02: `WarehouseCapacityAdmissionGranted` records token allocation.
- DW18-AUDIT-03: `WarehouseCapacityAdmissionQueued` records retry delay.
- DW18-AUDIT-04: `WarehouseCapacityAdmissionDenied` records reason code.
- DW18-AUDIT-05: `WarehouseCapacityAdmissionPreemptedLowerPriority` records preempted workloads.
- DW18-AUDIT-06: `WarehouseCapacityTokensReleased` records actual usage.
- DW18-AUDIT-07: `WarehouseCapacityPoolThrottled` records SLO burn trigger.

## SLO Targets
- DW18-SLO-01: p50 admission decision <= 10 ms.
- DW18-SLO-02: p95 admission decision <= 40 ms.
- DW18-SLO-03: p99 admission decision <= 90 ms.
- DW18-SLO-04: throughput >= 8,000 admission decisions per second per cell.
- DW18-SLO-05: availability >= 99.99 percent for admission API.
- DW18-SLO-06: token leak rate must be 0 after release reconciliation.
- DW18-SLO-07: queue retry accuracy p95 within 20 percent of actual wait.
- DW18-SLO-08: emergency throttle propagation <= 2 seconds.

## Failure Modes + Recovery
- DW18-FAIL-01: Pool row lock contention; use short retry and return queue decision before timeout.
- DW18-FAIL-02: Scheduler admits but executor fails to start; release tokens through decision expiration sweeper.
- DW18-FAIL-03: Actual memory exceeds estimate; throttle pool and adjust future estimator.
- DW18-FAIL-04: SLO burn rate spikes; close non-interactive pools and emit throttle event.
- DW18-FAIL-05: Residency cache unavailable; deny cross-cell work and allow only same-cell cached decisions.
- DW18-FAIL-06: Preemption cancel fails; do not admit replacement workload until token release is observed.

## Migration Notes
- DW18-MIG-01: Snowflake warehouse size and auto-suspend history seed pool profiles.
- DW18-MIG-02: BigQuery reservation slot capacity seeds reserved token counts.
- DW18-MIG-03: Redshift WLM queue configuration seeds pool lanes.
- DW18-MIG-04: Databricks SQL warehouse cluster size seeds memory and concurrency estimates.
- DW18-MIG-05: Synapse Analytics DWU level seeds scan throughput estimates.
- DW18-MIG-06: Firebolt engine size seeds reservation pool capacity.
- DW18-MIG-07: ClickHouse Cloud memory settings seed memory admission.
- DW18-MIG-08: Vertica resource pools seed memory and priority lanes.
- DW18-MIG-09: Teradata workload throttles seed tenant pool classes.
- DW18-MIG-10: Yellowbrick resource groups seed fixed capacity lanes.

## Cross-Microservice Handoffs
- DW18-HANDOFF-01: Query planner receives admit, queue, deny, or preempt decision.
- DW18-HANDOFF-02: Workflow receives preemption and throttle branches.
- DW18-HANDOFF-03: Cost-budget receives capacity reservation correlation ids.
- DW18-HANDOFF-04: SLO engine receives burn-rate throttle decisions.
- DW18-HANDOFF-05: Tenant-admin receives capacity queue and denial explanations.
- DW18-HANDOFF-06: Audit-chain receives ADR-0263 admission events.
- DW18-HANDOFF-07: Policy receives Cedar decision context.
- DW18-HANDOFF-08: Observability receives pool utilization metrics.
- DW18-HANDOFF-09: Data-pipeline receives backfill preemption notices.
- DW18-HANDOFF-10: Catalog receives pool health state for dataset availability.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-018-capacity-admission-control.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-018-capacity-admission-control.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
