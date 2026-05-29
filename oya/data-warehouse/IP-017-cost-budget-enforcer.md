---
doc_class: IP
ip_id: IP-017
microservice: data-warehouse
related_adrs: [ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321]
journey_ref: J150-creator-economy-shorts-creator-monetization-stack
capability_profile: Tier-1
status: draft
date: 2026-05-20
owner_team: axis-data-platform
---
# IP-017: Cost Budget Enforcer

## Context
- DW17-CTX-01: This IP enforces tenant warehouse spend before queries, shares, backfills, or exports consume capacity.
- DW17-CTX-02: Snowflake warehouse credit controls map to local budget envelopes.
- DW17-CTX-03: BigQuery slot reservations and on-demand bytes map to spend guardrails.
- DW17-CTX-04: Redshift concurrency scaling and RA3 storage map to budget categories.
- DW17-CTX-05: Databricks SQL serverless and pro warehouse DBUs map to workload pool budgets.
- DW17-CTX-06: Synapse Analytics DWU and serverless TB scanned map to estimate gates.
- DW17-CTX-07: Firebolt engine hours map to reservation budget lines.
- DW17-CTX-08: ClickHouse Cloud compute credits map to pool quota deltas.
- DW17-CTX-09: Vertica Eon depot and communal storage charges map to storage and compute budgets.
- DW17-CTX-10: Teradata Vantage workload charges map to account budget envelopes.
- DW17-CTX-11: Yellowbrick reserved cluster hours map to fixed capacity budget consumption.
- DW17-CTX-12: Enforcement happens before planning, after planning estimate, and after actual usage reconciliation.
- DW17-CTX-13: The enforcer must never rely on vendor invoice timing for decision authority.
- DW17-CTX-14: Budget overrides require workflow approval and audit-chain evidence.
- DW17-CTX-15: Cost decisions feed finance, tenant-admin, marketplace settlement, and SLO promotion.

## Data Model Deltas
- DW17-DDL-01: Add budget envelope table.
```sql
CREATE TABLE warehouse_cost_budget_envelopes (
    envelope_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    scope_kind TEXT NOT NULL CHECK (scope_kind IN ('tenant','dataset','workload_pool','dealset','backfill_batch')),
    scope_id UUID NOT NULL,
    currency_code CHAR(3) NOT NULL,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    hard_limit_micros BIGINT NOT NULL CHECK (hard_limit_micros >= 0),
    soft_limit_micros BIGINT NOT NULL CHECK (soft_limit_micros >= 0),
    consumed_micros BIGINT NOT NULL DEFAULT 0 CHECK (consumed_micros >= 0),
    reserved_micros BIGINT NOT NULL DEFAULT 0 CHECK (reserved_micros >= 0),
    status TEXT NOT NULL CHECK (status IN ('active','soft_exceeded','hard_exceeded','suspended')),
    audit_event_id UUID NOT NULL,
    CHECK (soft_limit_micros <= hard_limit_micros)
);
CREATE UNIQUE INDEX wh_budget_envelope_scope_period_idx ON warehouse_cost_budget_envelopes(tenant_id, scope_kind, scope_id, period_start, period_end);
```
- DW17-DDL-02: Add budget reservation table.
```sql
CREATE TABLE warehouse_cost_budget_reservations (
    reservation_id UUID PRIMARY KEY,
    envelope_id UUID NOT NULL REFERENCES warehouse_cost_budget_envelopes(envelope_id),
    tenant_id UUID NOT NULL,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('query','share','export','backfill','settlement')),
    source_id UUID NOT NULL,
    estimated_micros BIGINT NOT NULL CHECK (estimated_micros >= 0),
    actual_micros BIGINT,
    decision TEXT NOT NULL CHECK (decision IN ('allowed','soft_warned','denied','released','reconciled')),
    policy_decision_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    reconciled_at TIMESTAMPTZ
);
CREATE INDEX wh_budget_reservation_source_idx ON warehouse_cost_budget_reservations(tenant_id, source_kind, source_id);
```
- DW17-RUST-01: Budget envelope type.
```rust
pub struct WarehouseCostBudgetEnvelope {
    pub envelope_id: BudgetEnvelopeId,
    pub tenant_id: TenantId,
    pub scope: BudgetScope,
    pub currency_code: IsoCurrency,
    pub period: TimeWindow,
    pub hard_limit_micros: MoneyMicros,
    pub soft_limit_micros: MoneyMicros,
    pub consumed_micros: MoneyMicros,
    pub reserved_micros: MoneyMicros,
    pub status: BudgetEnvelopeStatus,
    pub audit_event_id: AuditEventId,
}
```
- DW17-RUST-02: Reservation type.
```rust
pub struct WarehouseCostBudgetReservation {
    pub reservation_id: BudgetReservationId,
    pub envelope_id: BudgetEnvelopeId,
    pub tenant_id: TenantId,
    pub source: BudgetSource,
    pub estimated_micros: MoneyMicros,
    pub actual_micros: Option<MoneyMicros>,
    pub decision: BudgetDecision,
    pub policy_decision_id: PolicyDecisionId,
    pub reconciled_at: Option<DateTime<Utc>>,
}
```
- DW17-RUST-03: `BudgetDecision::Denied` includes limit, estimate, and overage values.
- DW17-RUST-04: `BudgetSource` ids point to local objects, not vendor jobs.
- DW17-RUST-05: Reconciliation is monotonic and cannot reduce audit-visible consumed totals without refund event.

## API Endpoints
- DW17-API-01: REST reservation endpoint.
```http
POST /v1/data-warehouse/cost-budgets:reserve
Idempotency-Key: wh-budget-reserve-017
Content-Type: application/json

{"tenant_id":"018f8d8f-6fd1-7c28-bd2c-91c4045a0401","scope_kind":"workload_pool","scope_id":"01JPOOL017","source_kind":"query","source_id":"01JQUERY017","estimated_micros":1840000,"currency_code":"USD"}
```
- DW17-API-02: REST reconcile endpoint.
```http
POST /v1/data-warehouse/cost-budgets/reservations/{reservation_id}:reconcile
Content-Type: application/json

{"actual_micros":1712000,"usage_basis":{"scanned_bytes":9223372032,"execution_ms":1420,"vendor_source":"bigquery"}}
```
- DW17-API-03: gRPC decision.
```proto
rpc ReserveWarehouseBudget(ReserveWarehouseBudgetRequest) returns (ReserveWarehouseBudgetResponse);
message ReserveWarehouseBudgetRequest {
  string tenant_id = 1;
  string scope_kind = 2;
  string scope_id = 3;
  string source_kind = 4;
  string source_id = 5;
  int64 estimated_micros = 6;
}
```
- DW17-API-04: AsyncAPI event.
```yaml
warehouse.cost_budget.reservation.denied.v1:
  payload:
    reservation_id: 01JWH17RESERVE
    envelope_id: 01JWH17ENVELOPE
    overage_micros: 840000
    audit_event_class: WarehouseCostBudgetReservationDenied
```
- DW17-API-05: REST denial returns 402-like domain error `budget_hard_limit_exceeded`.
- DW17-API-06: gRPC denial returns `RESOURCE_EXHAUSTED`.
- DW17-API-07: Async soft warnings are delivered to tenant-admin and ops-dashboard.

## Cedar Policy Hooks
- DW17-CEDAR-01: principal = `Oyatie::Principal::"warehouse_query_planner:{planner_id}"`.
- DW17-CEDAR-02: action = `Oyatie::Action::"warehouse_budget_reserve"`.
- DW17-CEDAR-03: resource = `Oyatie::WarehouseBudgetEnvelope::"{envelope_id}"`.
- DW17-CEDAR-04: context.tenant_id must equal envelope tenant.
- DW17-CEDAR-05: context.estimated_micros must be less than remaining hard budget unless approved override exists.
- DW17-CEDAR-06: context.source_kind must be allowed for envelope scope.
- DW17-CEDAR-07: context.currency_code must equal envelope currency.
- DW17-CEDAR-08: context.override_workflow_id is required after hard limit.
- DW17-CEDAR-09: context.audit_event_class must equal `WarehouseCostBudgetReservationAllowed` or denial class.
- DW17-CEDAR-10: deny if principal lacks `warehouse.cost.reserve`.

## Ontology Projection
- DW17-ONTO-01: Snowflake `CREDITS_USED_CLOUD_SERVICES` -> `WarehouseCostUsage.compute_credit_units`.
- DW17-ONTO-02: BigQuery `totalBytesBilled` -> `WarehouseCostUsage.scanned_bytes`.
- DW17-ONTO-03: Redshift `concurrency_scaling_seconds` -> `WarehouseCostUsage.burst_compute_seconds`.
- DW17-ONTO-04: Databricks SQL `dbus` -> `WarehouseCostUsage.compute_credit_units`.
- DW17-ONTO-05: Synapse `data_processed_bytes` -> `WarehouseCostUsage.scanned_bytes`.
- DW17-ONTO-06: Firebolt `engine_seconds` -> `WarehouseCostUsage.engine_seconds`.
- DW17-ONTO-07: ClickHouse Cloud `compute_units` -> `WarehouseCostUsage.compute_credit_units`.
- DW17-ONTO-08: Vertica `depot_seconds` -> `WarehouseCostUsage.engine_seconds`.
- DW17-ONTO-09: Teradata Vantage `amp_cpu_seconds` -> `WarehouseCostUsage.compute_seconds`.
- DW17-ONTO-10: Yellowbrick `cluster_seconds` -> `WarehouseCostUsage.engine_seconds`.
- DW17-ONTO-11: Vendor invoice cost -> `WarehouseCostEvidence.vendor_invoice_amount` only after reconciliation.
- DW17-ONTO-12: Local estimate -> `WarehouseBudgetReservation.estimated_micros` before execution.

## Workflow Steps
- DW17-WF-01: Node `EstimateCost` computes query, export, share, or backfill estimate.
- DW17-WF-02: Node `FindEnvelope` resolves most specific active budget envelope.
- DW17-WF-03: Node `EvaluateBudgetPolicy` runs Cedar with estimate and remaining limits.
- DW17-WF-04: Branch `HardLimitExceeded` denies execution.
- DW17-WF-05: Branch `SoftLimitExceeded` allows only with warning event.
- DW17-WF-06: Node `ReserveAmount` atomically increments reserved budget.
- DW17-WF-07: Node `ExecuteSource` lets caller continue with reservation id.
- DW17-WF-08: Node `ReconcileActuals` converts actual usage into consumed budget.
- DW17-WF-09: Branch `ActualExceedsEstimate` emits overrun and may suspend envelope.
- DW17-WF-10: Node `ReleaseUnused` removes unused reservation amount.
- DW17-WF-11: Node `NotifyFinance` posts period cost deltas.
- DW17-WF-12: Node `PublishTenantWarning` updates tenant-admin budget view.

## Audit Events
- DW17-AUDIT-01: `WarehouseCostBudgetReservationRequested` records estimate and source.
- DW17-AUDIT-02: `WarehouseCostBudgetReservationAllowed` records remaining budget.
- DW17-AUDIT-03: `WarehouseCostBudgetReservationSoftWarned` records soft threshold crossing.
- DW17-AUDIT-04: `WarehouseCostBudgetReservationDenied` records hard limit overage.
- DW17-AUDIT-05: `WarehouseCostBudgetReservationReconciled` records actual usage.
- DW17-AUDIT-06: `WarehouseCostBudgetEnvelopeSuspended` records suspension reason.
- DW17-AUDIT-07: `WarehouseCostBudgetOverrideApproved` records approval workflow id.

## SLO Targets
- DW17-SLO-01: p50 budget reserve <= 15 ms.
- DW17-SLO-02: p95 budget reserve <= 60 ms.
- DW17-SLO-03: p99 budget reserve <= 140 ms.
- DW17-SLO-04: throughput >= 5,000 budget decisions per second per cell.
- DW17-SLO-05: availability >= 99.99 percent for reserve API.
- DW17-SLO-06: reconciliation lag p95 <= 60 seconds after query completion.
- DW17-SLO-07: false allow past hard limit must be 0.
- DW17-SLO-08: warning delivery p95 <= 10 seconds.

## Failure Modes + Recovery
- DW17-FAIL-01: Estimate service fails; deny high-cost operations and allow only configured low-cost metadata reads.
- DW17-FAIL-02: Envelope row lock contention; retry with bounded backoff and return 429 after budget decision timeout.
- DW17-FAIL-03: Actual usage exceeds estimate; reconcile actuals, emit overrun event, and optionally suspend scope.
- DW17-FAIL-04: Currency mismatch; reject reservation and require finance exchange-rate setup.
- DW17-FAIL-05: Override approval expires; deny new reservations and keep existing reservations valid until reconciliation.
- DW17-FAIL-06: Vendor usage arrives late; reconcile into current open correction period with audit reason.

## Migration Notes
- DW17-MIG-01: Snowflake credits require warehouse-size and cloud-services split.
- DW17-MIG-02: BigQuery slot reservation and on-demand bytes must be separated.
- DW17-MIG-03: Redshift concurrency scaling charges require burst category mapping.
- DW17-MIG-04: Databricks SQL DBUs require warehouse type and Photon mode evidence.
- DW17-MIG-05: Synapse Analytics DWU and serverless TB charges use different estimation formulas.
- DW17-MIG-06: Firebolt engine hours map to fixed engine-second rates.
- DW17-MIG-07: ClickHouse Cloud compute units and storage charges must not be collapsed.
- DW17-MIG-08: Vertica Eon depot seconds and communal storage use separate budget categories.
- DW17-MIG-09: Teradata Vantage workload accounting maps to account-level budgets.
- DW17-MIG-10: Yellowbrick reserved cluster hours map to reservation budgets.

## Cross-Microservice Handoffs
- DW17-HANDOFF-01: Finance receives reconciled actual micros per envelope period.
- DW17-HANDOFF-02: Tenant-admin receives soft warning and hard denial events.
- DW17-HANDOFF-03: Query planner receives reservation id or denial before execution.
- DW17-HANDOFF-04: Marketplace settlement receives cost basis for dealset lines.
- DW17-HANDOFF-05: Audit-chain receives ADR-0263 events for reserve and reconcile.
- DW17-HANDOFF-06: Workflow receives override approval tasks.
- DW17-HANDOFF-07: Policy receives Cedar decision contexts.
- DW17-HANDOFF-08: Observability receives budget decision latency and denial counters.
- DW17-HANDOFF-09: Catalog receives suspended dataset or pool status when budget blocks access.
- DW17-HANDOFF-10: Data-pipeline receives backfill budget reservation requirements.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/data-warehouse/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/data-warehouse/IP-017-cost-budget-enforcer.md` matched `p99, SLO`; anchors `microservices/data-warehouse/runbooks/cross-region-replica-lag.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/data-warehouse/IP-017-cost-budget-enforcer.md` matched `cost`; anchors `microservices/data-warehouse/runbooks/warehouse-cost-spike.md, microservices/data-warehouse/src/lib.rs`; type anchor `microservices/data-warehouse/src/lib.rs::ServiceDescriptor`.
