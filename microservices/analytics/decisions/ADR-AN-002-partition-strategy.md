---
id: ADR-AN-002
title: "Analytics partitions by workload cadence and tenant-safe time buckets"
status: Accepted
date: 2026-05-18
microservice: analytics
related_oyatie_adrs:
  - ADR-0003
  - ADR-0193
  - ADR-0195
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
decision_owner: council-analytics + ops-sre-reliability + ops-finops
---

# ADR-AN-002: Analytics partitions by workload cadence and tenant-safe time buckets

## Context

- The named architectural pressure is `bounded-partition-count-with-tenant-safe-pruning`.
- Analytics tables must remain queryable under high tenant count and long retention windows.
- ADR-0193 requires partition rotation.
- ADR-0195 requires materialized views to remain operationally debuggable.
- ADR-0244 requires tenant scoping on warehouse rows.
- ADR-0251 requires pack-level data residency and retention overlays.
- Prior incident class `daily-partitions-for-seven-years` created too many partitions for audit tables.
- Prior incident class `monthly-billing-overwrite` grouped mutable daily billing into coarse partitions.
- Prior incident class `tenant-in-partition-key-explosion` created one partition per tenant per day.
- Prior incident class `partition-pruning-miss` scanned a full cell for tenant dashboards.
- ClickHouse partition count affects merges, mutation latency, and memory.
- ClickHouse best practice is to avoid high-cardinality partition keys.
- Tenant id belongs in ORDER BY and primary key pruning, not in partition expression.
- Audit events are high volume but append-only.
- KPI rollups are dashboard-facing and time-bucketed.
- Billing rollups require dispute-window reconstruction.
- Materialized-view intermediates need fast deletion and bounded merges.
- Partition strategy must match ADR-AN-001 TTL classes.
- Partition strategy must support cold-tier movement.
- Partition strategy must support pack overlays without table redesign.
- Partition strategy must expose metrics and dashboards.
- Partition strategy must be enforceable in generated DDL.
- The implementation must be buildable from this ADR.

## Decision

- We choose `workload-cadence partitioning`.
- The named pattern is `time partition plus tenant-ordered primary key`.
- Analytics uses ClickHouse 26.3 LTS MergeTree-family engines.
- We do not include `tenant_id` in partition expressions.
- We include `tenant_id` in ORDER BY keys.
- We include `pack_id` in table database placement, not in partition expression.
- Audit logs partition by month.
- Business KPI rollups partition by month.
- Billing daily rollups partition by month.
- Billing monthly rollups partition by year.
- Telemetry rollups partition by month.
- MV intermediate state partitions by day when source cadence is minute or five-minute.
- MV intermediate state partitions by month when source cadence is hour or day.
- Attestation and receipt tables partition by year.
- Fleet-internal operational tables partition by month.
- Maximum active partitions per table per cell is 5,000.
- Warning threshold is 3,500 active partitions.
- Sev-2 threshold is 4,500 active partitions.
- Deploy-block threshold is 5,000 active partitions.
- Each table declares `partition_strategy_id`.
- Each table declares `partition_time_column`.
- Each table declares `partition_granularity`.
- Partition expressions use `toYYYYMM` for month.
- Partition expressions use `toYYYY` for year.
- Partition expressions use `toDate` only for approved high-churn MV intermediate tables.
- ORDER BY begins with `(tenant_id, bucket_start)` for rollups.
- ORDER BY begins with `(tenant_id, emitted_at, event_type)` for audit events.
- ORDER BY begins with `(tenant_id, invoice_period_start, resource_type)` for billing.
- ORDER BY begins with `(tenant_id, receipt_created_at)` for receipts.
- Primary key mirrors ORDER BY prefix.
- Partition migrations are additive and require backfill plan.
- Cedar action `analytics.partition_strategy.apply` gates strategy changes.
- Cedar action `analytics.partition_backfill.run` gates backfills.
- Cedar action `analytics.partition_drop.execute` delegates to ADR-AN-001 retention control.

## Alternatives Considered

### Partition by tenant and month

- Pro: tenant-level pruning is excellent.
- Pro: tenant exports are simple.
- Pro: noisy tenant isolation is visually obvious.
- Con: partition count explodes with tenant count.
- Con: ClickHouse merges degrade.
- Con: onboarding tenants becomes DDL pressure.
- Con: violates ClickHouse high-cardinality partition guidance.
- Tradeoff: good tenant pruning but poor fleet scalability.
- Rejected.

### Partition by day for every table

- Pro: small delete windows.
- Pro: precise retention.
- Pro: easier daily billing backfill.
- Con: seven-year audit tables exceed partition ceiling.
- Con: merge pressure grows across every workload.
- Con: cold-tier movement creates excessive object churn.
- Tradeoff: precision but too many partitions.
- Rejected.

### Partition by month for every table

- Pro: simple convention.
- Pro: good fit for audit and KPI.
- Pro: low partition count.
- Con: minute-level MV intermediates retain too much hot state.
- Con: yearly receipt tables do not need monthly churn.
- Con: daily billing repair touches more data than needed.
- Tradeoff: simple but not workload-aware enough.
- Rejected.

### Partition by hash tenant only

- Pro: even distribution.
- Pro: stable partition count.
- Pro: tenant isolation feels direct.
- Con: TTL and retention by time become expensive.
- Con: cold-tier movement cannot align to time.
- Con: regulator evidence by date is harder.
- Tradeoff: distribution but bad retention semantics.
- Rejected.

### Use Citus/Postgres for partitioned analytics tables

- Pro: familiar relational partitioning.
- Pro: strong constraints and transactions.
- Pro: easier tenant RLS.
- Con: analytic scan performance is weaker.
- Con: ClickHouse materialized-view pipeline is lost.
- Con: storage cost and CPU increase for dashboard workloads.
- Tradeoff: stronger OLTP governance but wrong engine for analytics.
- Rejected.

## Consequences

- Positive: partition count stays bounded.
- Positive: tenant pruning remains effective through ORDER BY.
- Positive: retention and partitioning align.
- Positive: MV intermediates can delete quickly.
- Positive: CI can validate partition expressions.
- Negative: tenant export can scan multiple month partitions.
- Negative: table-specific strategies require migration metadata.
- Negative: wrong ORDER BY order can ruin pruning.
- Negative: daily MV partitions need close monitoring.
- Neutral: future large tenants may require dedicated cell placement.
- Neutral: Citus remains metadata store, not warehouse store.
- Follow-up work: implement `oya-governance-partition-strategy`.
- Follow-up work: add partition-count forecast to capacity model.
- Follow-up work: add ClickHouse system.parts dashboard.
- Follow-up work: add backfill plan template for partition strategy changes.

## Implementation Notes

- Data shape `AnalyticsPartitionStrategyV1` contains `strategy_id`.
- Data shape `AnalyticsPartitionStrategyV1` contains `workload_class`.
- Data shape `AnalyticsPartitionStrategyV1` contains `partition_time_column`.
- Data shape `AnalyticsPartitionStrategyV1` contains `partition_expression`.
- Data shape `AnalyticsPartitionStrategyV1` contains `partition_granularity`.
- Data shape `AnalyticsPartitionStrategyV1` contains `order_by_columns`.
- Data shape `AnalyticsPartitionStrategyV1` contains `primary_key_columns`.
- Data shape `AnalyticsPartitionStrategyV1` contains `active_partition_warning`.
- Data shape `AnalyticsPartitionStrategyV1` contains `active_partition_block`.
- Data shape `AnalyticsPartitionStrategyV1` contains `pack_overlay_id`.
- Data shape `PartitionBackfillPlanV1` contains `plan_id`.
- Data shape `PartitionBackfillPlanV1` contains `source_table`.
- Data shape `PartitionBackfillPlanV1` contains `target_table`.
- Data shape `PartitionBackfillPlanV1` contains `copy_window_start`.
- Data shape `PartitionBackfillPlanV1` contains `copy_window_end`.
- Data shape `PartitionBackfillPlanV1` contains `dual_write_enabled`.
- Data shape `PartitionBackfillPlanV1` contains `rollback_partition`.
- Audit table DDL uses `PARTITION BY toYYYYMM(emitted_at)`.
- Audit table DDL uses `ORDER BY (tenant_id, emitted_at, event_type, event_id)`.
- KPI table DDL uses `PARTITION BY toYYYYMM(bucket_start)`.
- KPI table DDL uses `ORDER BY (tenant_id, bucket_start, metric_name, dimension_hash)`.
- Billing daily table DDL uses `PARTITION BY toYYYYMM(billing_day)`.
- Billing monthly table DDL uses `PARTITION BY toYYYY(invoice_period_start)`.
- MV minute intermediate DDL uses `PARTITION BY toDate(bucket_start)`.
- Receipt table DDL uses `PARTITION BY toYYYY(receipt_created_at)`.
- API endpoint `GET /v1/internal/partition-strategies` lists strategies.
- API endpoint `POST /v1/internal/partition-strategies/apply` applies strategy metadata.
- API endpoint `POST /v1/internal/partition-backfills` creates backfill plan.
- API endpoint `GET /v1/internal/partition-health` returns active partition counts.
- API endpoint `GET /v1/internal/partition-health/{table_name}` returns table-level health.
- ClickHouse system table `system.parts` is the source for active partition counts.
- ClickHouse setting `parts_to_delay_insert` is tuned per table class.
- ClickHouse setting `parts_to_throw_insert` is lower for MV intermediates.
- Cedar principal for strategy apply is `Oyatie::Principal::Service("analytics-ddl-controller")`.
- Cedar principal for backfill is `Oyatie::Principal::Service("analytics-backfill-worker")`.
- Cedar resource for strategy is `Analytics::PartitionStrategy`.
- Cedar resource for backfill is `Analytics::PartitionBackfillPlan`.
- Example permit: principal `analytics-ddl-controller`, action `analytics.partition_strategy.apply`, resource `Analytics::PartitionStrategy::"audit_monthly_v1"`, context `{workload_class:"audit_log", granularity:"month", active_partition_block:5000}`.
- Example permit: principal `analytics-backfill-worker`, action `analytics.partition_backfill.run`, resource `Analytics::PartitionBackfillPlan::"pb_01HY"`, context `{dual_write_enabled:true, rollback_partition:"202605"}`.
- Example forbid: strategy apply with context `{partition_expression:"tenant_id"}`.
- Example forbid: strategy apply with context `{estimated_active_partitions:6200}`.
- SLO `analytics-partition-health.openslo.yaml` sets partition health reconcile p99 <= 10 minutes.
- SLO `analytics-partition-backfill.openslo.yaml` sets backfill lag p99 <= 24 hours for approved migrations.
- SLO `analytics-query-pruning.openslo.yaml` requires 95 percent of tenant dashboard queries to touch <= 3 partitions.
- Failure mode `partition_count_warning` creates ticket.
- Failure mode `partition_count_block` blocks migration.
- Failure mode `tenant_id_partition_expression` blocks deploy.
- Failure mode `order_by_missing_tenant_id` blocks deploy.
- Failure mode `backfill_dual_write_missing` blocks migration.

## Verification

- Test `audit_partition_is_monthly` validates audit DDL.
- Test `kpi_partition_is_monthly` validates KPI DDL.
- Test `billing_monthly_partition_is_yearly` validates billing DDL.
- Test `mv_minute_intermediate_partition_is_daily` validates MV DDL.
- Test `receipt_partition_is_yearly` validates receipt DDL.
- Test `tenant_id_not_in_partition_expression` validates high-cardinality guard.
- Test `tenant_id_first_in_order_by` validates pruning.
- Test `partition_strategy_requires_metadata` validates migration metadata.
- Test `partition_count_above_block_blocks_deploy` validates guardrail.
- Test `backfill_plan_requires_rollback_partition` validates migration safety.
- Metric `oya_analytics_active_partitions` tracks active partitions by table.
- Metric `oya_analytics_partition_health_reconcile_seconds` must stay below 600.
- Metric `oya_analytics_query_partitions_touched` tracks pruning.
- Metric `oya_analytics_parts_to_delay_insert_total` tracks ClickHouse pressure.
- Dashboard `analytics-partition-health.json` shows active partitions and thresholds.
- Dashboard `analytics-query-pruning.json` shows partitions touched by query class.
- Dashboard `analytics-backfill.json` shows dual-write and lag.
- CI check `analytics-partition-ddl` validates partition expressions.
- CI check `analytics-partition-order-by` validates tenant-first ordering.
- CI check `analytics-partition-count-forecast` validates 36-month forecast.
- CI check `analytics-backfill-plan-required` validates migration plans.
- Load test runs dashboard queries over 24 months of partitions.
- Chaos test blocks merges and verifies partition pressure alerts.
- Backfill drill runs quarterly on a sample KPI table.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0193: Analytics storage, TTL, partition rotation, and cold tier.
- ADR-0195: Materialized views as stream-processing default.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0263: Observability emission contract.
- ClickHouse 26.3 LTS MergeTree documentation.
- ClickHouse partitioning best practices.
- ClickHouse system.parts documentation.
- PostgreSQL 16.6 partitioning documentation.
- Citus 12.1 documentation.
- NIST SP 800-53 Rev. 5 AU-11 and SI-12.
- GDPR Art. 5(1)(e).
- SOC 2 CC7.2.
