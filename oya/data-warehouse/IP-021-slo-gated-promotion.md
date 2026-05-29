---
doc_class: IP
ip_id: IP-021
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-021: SLO Gated Promotion

## Context
- DW21-CTX-01: This IP prevents warehouse features, migrations, and vendor cutovers from promoting without SLO evidence.
- DW21-CTX-02: Snowflake cutover parity requires query latency and correctness gates.
- DW21-CTX-03: BigQuery migration parity requires bytes-billed estimate drift and slot saturation gates.
- DW21-CTX-04: Redshift migration parity requires WLM queue latency and result parity gates.
- DW21-CTX-05: Databricks SQL migration parity requires statement latency and warehouse warm-start gates.
- DW21-CTX-06: Synapse Analytics migration parity requires DWU performance and serverless scan gates.
- DW21-CTX-07: Firebolt migration parity requires engine start and query latency gates.
- DW21-CTX-08: ClickHouse Cloud migration parity requires ingest freshness and query latency gates.
- DW21-CTX-09: Vertica migration parity requires projection freshness and query performance gates.
- DW21-CTX-10: Teradata Vantage migration parity requires workload class and result parity gates.
- DW21-CTX-11: Yellowbrick migration parity requires cluster queue and query latency gates.
- DW21-CTX-12: Promotion is explicit: candidate, shadow, canary, regional, and global states.
- DW21-CTX-13: SLO gate evidence is stored with audit-chain references and rollback target.
- DW21-CTX-14: A failed SLO gate blocks marketplace settlement and catalog public exposure.
- DW21-CTX-15: Promotion must use local evidence, not vendor dashboards.

## Data Model Deltas
- DW21-DDL-01: Add promotion gate table.
```sql
CREATE TABLE warehouse_slo_promotion_gates (
    gate_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    promotion_subject_kind TEXT NOT NULL CHECK (promotion_subject_kind IN ('dataset','workload_pool','query_template','vendor_cutover','sdk_release','dealset_share')),
    promotion_subject_id UUID NOT NULL,
    promotion_stage TEXT NOT NULL CHECK (promotion_stage IN ('candidate','shadow','canary','regional','global')),
    slo_profile TEXT NOT NULL,
    gate_decision TEXT NOT NULL CHECK (gate_decision IN ('pending','passed','failed','waived')),
    evidence_bundle_ref TEXT NOT NULL,
    rollback_target_ref TEXT NOT NULL,
    policy_decision_id UUID NOT NULL,
    audit_event_id UUID NOT NULL,
    decided_at TIMESTAMPTZ
);
CREATE INDEX wh_slo_gate_subject_idx ON warehouse_slo_promotion_gates(tenant_id, promotion_subject_kind, promotion_subject_id, promotion_stage);
```
- DW21-DDL-02: Add SLO measurement rows.
```sql
CREATE TABLE warehouse_slo_promotion_measurements (
    measurement_id UUID PRIMARY KEY,
    gate_id UUID NOT NULL REFERENCES warehouse_slo_promotion_gates(gate_id) ON DELETE CASCADE,
    metric_name TEXT NOT NULL,
    p50 NUMERIC(18,6),
    p95 NUMERIC(18,6),
    p99 NUMERIC(18,6),
    throughput NUMERIC(18,6),
    availability NUMERIC(9,6),
    objective_met BOOLEAN NOT NULL,
    measured_window tstzrange NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX wh_slo_measurements_gate_idx ON warehouse_slo_promotion_measurements(gate_id, metric_name);
```
- DW21-RUST-01: Promotion gate type.
```rust
pub struct WarehouseSloPromotionGate {
    pub gate_id: SloGateId,
    pub tenant_id: TenantId,
    pub promotion_subject: PromotionSubject,
    pub promotion_stage: PromotionStage,
    pub slo_profile: SloProfileName,
    pub gate_decision: GateDecision,
    pub evidence_bundle_ref: EvidenceBundleRef,
    pub rollback_target_ref: RollbackTargetRef,
    pub policy_decision_id: PolicyDecisionId,
    pub audit_event_id: AuditEventId,
    pub decided_at: Option<DateTime<Utc>>,
}
```
- DW21-RUST-02: Measurement type.
```rust
pub struct WarehouseSloPromotionMeasurement {
    pub measurement_id: SloMeasurementId,
    pub gate_id: SloGateId,
    pub metric_name: SloMetricName,
    pub p50: Option<Decimal>,
    pub p95: Option<Decimal>,
    pub p99: Option<Decimal>,
    pub throughput: Option<Decimal>,
    pub availability: Option<Decimal>,
    pub objective_met: bool,
    pub measured_window: TimeWindow,
}
```
- DW21-RUST-03: `GateDecision::Waived` requires a waiver id and expiry.
- DW21-RUST-04: `PromotionStage` transitions must be sequential.
- DW21-RUST-05: Measurements cannot be updated after gate decision; new run creates new gate.

## API Endpoints
- DW21-API-01: REST evaluate gate endpoint.
```http
POST /v1/data-warehouse/slo-promotion-gates:evaluate
Idempotency-Key: wh-slo-gate-021
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","promotion_subject_kind":"vendor_cutover","promotion_subject_id":"01JCUTOVER021","promotion_stage":"canary","slo_profile":"warehouse_vendor_cutover_v1","rollback_target_ref":"catalog:01JCATALOG021:previous"}
```
- DW21-API-02: REST measurement append endpoint.
```http
POST /v1/data-warehouse/slo-promotion-gates/{gate_id}/measurements
Content-Type: application/json

{"metric_name":"warehouse_query_latency_ms","p50":42.3,"p95":188.4,"p99":420.8,"throughput":2200.0,"availability":99.982,"objective_met":true,"measured_window":"[2026-05-20T18:00:00Z,2026-05-20T19:00:00Z)"}
```
- DW21-API-03: gRPC gate decision.
```proto
rpc EvaluateWarehouseSloPromotion(EvaluateWarehouseSloPromotionRequest) returns (EvaluateWarehouseSloPromotionResponse);
message EvaluateWarehouseSloPromotionRequest {
  string tenant_id = 1;
  string promotion_subject_kind = 2;
  string promotion_subject_id = 3;
  string promotion_stage = 4;
  string slo_profile = 5;
}
```
- DW21-API-04: AsyncAPI event.
```yaml
warehouse.slo_promotion.gate.failed.v1:
  payload:
    gate_id: 01JWH21GATE
    promotion_stage: canary
    failed_metric: warehouse_query_latency_ms
    audit_event_class: WarehouseSloPromotionGateFailed
```
- DW21-API-05: REST failed gate returns 412 for dependent promotion commands.
- DW21-API-06: gRPC response includes rollback target on failure.
- DW21-API-07: Async passed events trigger next-stage workflow only after audit seal.

## Cedar Policy Hooks
- DW21-CEDAR-01: principal = `Oyatie::Principal::"release_controller:{controller_id}"`.
- DW21-CEDAR-02: action = `Oyatie::Action::"warehouse_slo_promotion_decide"`.
- DW21-CEDAR-03: resource = `Oyatie::WarehousePromotionSubject::"{promotion_subject_id}"`.
- DW21-CEDAR-04: context.previous_stage_passed must be true except candidate.
- DW21-CEDAR-05: context.measurements_complete must be true.
- DW21-CEDAR-06: context.rollback_target_ref must be non-empty.
- DW21-CEDAR-07: context.failed_objectives must be empty unless waiver exists.
- DW21-CEDAR-08: context.waiver_expiry must be future when decision is waived.
- DW21-CEDAR-09: context.audit_event_class must equal pass, fail, or waived class.
- DW21-CEDAR-10: deny if principal lacks `warehouse.release.promote`.

## Ontology Projection
- DW21-ONTO-01: Snowflake query profile latency -> `WarehouseSloMeasurement.query_latency_ms`.
- DW21-ONTO-02: BigQuery job statistics -> `WarehouseSloMeasurement.bytes_scanned_drift`.
- DW21-ONTO-03: Redshift WLM queue time -> `WarehouseSloMeasurement.queue_latency_ms`.
- DW21-ONTO-04: Databricks SQL statement metrics -> `WarehouseSloMeasurement.query_latency_ms`.
- DW21-ONTO-05: Synapse request metrics -> `WarehouseSloMeasurement.scan_latency_ms`.
- DW21-ONTO-06: Firebolt engine start time -> `WarehouseSloMeasurement.engine_start_ms`.
- DW21-ONTO-07: ClickHouse Cloud query_log latency -> `WarehouseSloMeasurement.query_latency_ms`.
- DW21-ONTO-08: Vertica projection refresh -> `WarehouseSloMeasurement.freshness_lag_ms`.
- DW21-ONTO-09: Teradata workload delay -> `WarehouseSloMeasurement.queue_latency_ms`.
- DW21-ONTO-10: Yellowbrick queue delay -> `WarehouseSloMeasurement.queue_latency_ms`.
- DW21-ONTO-11: Vendor availability metric -> local synthetic availability measurement.
- DW21-ONTO-12: Vendor result checksum -> local correctness parity measurement.

## Workflow Steps
- DW21-WF-01: Node `CreateGate` records subject, stage, SLO profile, and rollback target.
- DW21-WF-02: Node `CollectMeasurements` loads latency, throughput, availability, and correctness evidence.
- DW21-WF-03: Node `EvaluateObjectives` compares measurement rows with profile thresholds.
- DW21-WF-04: Branch `MissingMeasurements` keeps gate pending.
- DW21-WF-05: Branch `ObjectiveFailed` marks gate failed and blocks promotion.
- DW21-WF-06: Branch `WaiverRequested` routes to governance approval.
- DW21-WF-07: Node `EvaluatePolicy` runs Cedar for final decision.
- DW21-WF-08: Node `SealEvidence` writes evidence bundle and audit event.
- DW21-WF-09: Node `PromoteStage` advances subject to next stage.
- DW21-WF-10: Node `NotifyRollbackOwner` sends rollback target on failure.
- DW21-WF-11: Node `NotifyMarketplace` allows public exposure only after pass.
- DW21-WF-12: Node `ArchiveGate` freezes measurements after decision.

## Audit Events
- DW21-AUDIT-01: `WarehouseSloPromotionGateCreated` records subject and stage.
- DW21-AUDIT-02: `WarehouseSloPromotionMeasurementRecorded` records metric and objective result.
- DW21-AUDIT-03: `WarehouseSloPromotionGatePassed` records evidence bundle ref.
- DW21-AUDIT-04: `WarehouseSloPromotionGateFailed` records failed metric.
- DW21-AUDIT-05: `WarehouseSloPromotionGateWaived` records waiver id and expiry.
- DW21-AUDIT-06: `WarehouseSloPromotionRollbackTargetSelected` records rollback target.
- DW21-AUDIT-07: `WarehouseSloPromotionStageAdvanced` records old and new stage.

## SLO Targets
- DW21-SLO-01: p50 gate evaluation <= 80 ms.
- DW21-SLO-02: p95 gate evaluation <= 400 ms.
- DW21-SLO-03: p99 gate evaluation <= 1,200 ms.
- DW21-SLO-04: throughput >= 100 gate evaluations per minute per cell.
- DW21-SLO-05: availability >= 99.95 percent for gate API.
- DW21-SLO-06: measurement ingestion lag p95 <= 30 seconds.
- DW21-SLO-07: failed gate propagation to dependent commands <= 5 seconds.
- DW21-SLO-08: false promotion with failed objective must be 0.

## Failure Modes + Recovery
- DW21-FAIL-01: Measurement window incomplete; keep gate pending and block stage promotion.
- DW21-FAIL-02: Evidence bundle write fails; do not decide gate until bundle persists.
- DW21-FAIL-03: Waiver expires during promotion; revert gate to failed and notify release owner.
- DW21-FAIL-04: Rollback target missing; fail gate even if metrics pass.
- DW21-FAIL-05: Vendor parity checksum mismatch; fail correctness objective and quarantine migrated subject.
- DW21-FAIL-06: Audit event outbox delayed; block dependent promotion until audit seal completes.

## Migration Notes
- DW21-MIG-01: Snowflake parity gate compares query latency, row counts, and credit estimates.
- DW21-MIG-02: BigQuery parity gate compares bytes billed, slot saturation, and result checksums.
- DW21-MIG-03: Redshift parity gate compares WLM latency and result checksums.
- DW21-MIG-04: Databricks SQL parity gate compares statement latency and warm start.
- DW21-MIG-05: Synapse Analytics parity gate compares DWU scans and serverless estimates.
- DW21-MIG-06: Firebolt parity gate compares engine start and query runtime.
- DW21-MIG-07: ClickHouse Cloud parity gate compares query latency and ingest freshness.
- DW21-MIG-08: Vertica parity gate compares projection freshness and latency.
- DW21-MIG-09: Teradata Vantage parity gate compares workload delay and result checksums.
- DW21-MIG-10: Yellowbrick parity gate compares queue wait and query runtime.

## Cross-Microservice Handoffs
- DW21-HANDOFF-01: Release controller receives pass, fail, or waived decision.
- DW21-HANDOFF-02: Marketplace receives public exposure approval only after pass.
- DW21-HANDOFF-03: Catalog receives promotion stage status.
- DW21-HANDOFF-04: Audit-chain receives ADR-0263 gate events.
- DW21-HANDOFF-05: Observability receives SLO measurement rows.
- DW21-HANDOFF-06: Workflow receives waiver and rollback tasks.
- DW21-HANDOFF-07: Tenant-admin receives promotion status and failed objectives.
- DW21-HANDOFF-08: Policy receives Cedar decision context.
- DW21-HANDOFF-09: Data-pipeline receives vendor parity remediation tasks.
- DW21-HANDOFF-10: Cost-budget receives promotion block state for expensive workloads.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-021-slo-gated-promotion.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
