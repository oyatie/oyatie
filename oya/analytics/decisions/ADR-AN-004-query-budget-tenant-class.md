---
id: ADR-AN-004
title: "Analytics query budgets are enforced by tenant_class and workload class"
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
decision_owner: council-analytics + ops-finops + ops-sre-reliability
---

# ADR-AN-004: Analytics query budgets are enforced by tenant_class and workload class

## Context

- The named architectural pressure is `shared-warehouse-fairness-with-visible-cost-controls`.
- Analytics is a shared warehouse surface across tenant dashboards, exports, and internal operations.
- ADR-0193 sets ClickHouse as the warehouse substrate.
- ADR-0244 requires tenant-scoped resource accounting.
- ADR-0243 requires Cedar gates before expensive actions.
- Prior incident class `paid-export-starved-demo-trial-dashboards` let one large export consume query slots.
- Prior incident class `unbounded-dashboard-refresh` allowed repeated expensive dashboards.
- Prior incident class `query-budget-hidden-from-tenant` caused support escalations after throttling.
- Prior incident class `manual-kill-no-audit` stopped queries without evidence.
- ClickHouse has query limits and quotas but they must map to tenant_class.
- Tenants need predictable dashboards.
- FinOps needs bounded warehouse spend.
- SRE needs overload protection before ClickHouse refuses inserts.
- Compliance exports need priority but cannot starve operational dashboards.
- Budget enforcement must return explicit 429 responses.
- Budget enforcement must expose retry windows.
- Budget enforcement must be adjustable by tenant_class change.
- Budget enforcement must have pack overlays for regulated exports.
- Budget enforcement must not leak tenant workload details to other tenants.
- Query costs must be measured by bytes read, rows read, CPU seconds, and concurrent slots.
- The implementation must be buildable from this ADR.

## Decision

- We choose `tenant_class analytics query budgets`.
- The named pattern is `Cedar-admitted query plus ClickHouse quota envelope`.
- Query budgets are assigned by tenant_class.
- Tenant classes are `demo_trial` and `paid`.
- demo_trial tenants get 2 concurrent dashboard queries.
- paid tenants get 20 concurrent dashboard queries by default.
- paid contract overlays can raise dashboard concurrency to 100.
- Internal service workloads get 500 concurrent dashboard queries.
- demo_trial tenants get 5 GiB bytes-read per hour.
- paid tenants get 500 GiB bytes-read per hour by default.
- paid contract overlays can raise bytes-read to 5 TiB per hour.
- Internal service workloads get 25 TiB bytes-read per hour.
- demo_trial tenants get 60 CPU-seconds per hour.
- paid tenants get 6,000 CPU-seconds per hour by default.
- paid contract overlays can raise CPU budget to 60,000 CPU-seconds per hour.
- Internal service workloads get 300,000 CPU-seconds per hour.
- Export jobs use separate export slots.
- demo_trial tenants get 0 export slots by default.
- paid tenants get 3 export slots by default.
- paid contract overlays can raise export slots to 20.
- Internal service workloads get 100 export slots.
- Compliance exports can borrow slots only from the export pool.
- Dashboard queries never borrow from export pool.
- Query budget changes propagate within 30 seconds p99.
- Budget overrun returns HTTP 429.
- 429 response includes `retry_after_seconds`.
- 429 response includes `budget_dimension`.
- 429 response includes `tenant_class`.
- ClickHouse quotas enforce bytes and CPU.
- Analytics admission controller enforces concurrent slots.
- Cedar action `analytics.query.admit` gates query admission.
- Cedar action `analytics.query.export_admit` gates export admission.
- Cedar action `analytics.query.budget_override` gates temporary override.
- Overrides expire within 24 hours.
- Overrides require audit-chain evidence.

## Alternatives Considered

### No query budgets

- Pro: simplest product behavior.
- Pro: no tenant-throttling support cases.
- Pro: dashboards run until warehouse saturates.
- Con: noisy tenants can starve others.
- Con: FinOps spend becomes unbounded.
- Con: SRE has no early overload control.
- Tradeoff: simplicity but unacceptable shared-service risk.
- Rejected.

### One global budget for all tenants

- Pro: easy to implement.
- Pro: simple quota shape.
- Pro: clear fleet ceiling.
- Con: unfair across tenant sizes.
- Con: demo_trial and paid tenants get same experience.
- Con: tenant_class loses quota meaning.
- Tradeoff: simple but commercially wrong.
- Rejected.

### Budget only by bytes read

- Pro: maps to ClickHouse native metrics.
- Pro: easy to explain.
- Pro: good FinOps correlation.
- Con: CPU-heavy aggregations can still starve cluster.
- Con: concurrency can still exhaust query threads.
- Con: exports can block dashboards.
- Tradeoff: useful but incomplete overload control.
- Rejected.

### Budget only by concurrency

- Pro: protects query slots.
- Pro: easy admission control.
- Pro: simple 429 semantics.
- Con: one query can read huge data.
- Con: CPU and IO spend remain unbounded.
- Con: tenants cannot understand cost drivers.
- Tradeoff: protects slots but not resource consumption.
- Rejected.

### Dedicated warehouse per enterprise tenant

- Pro: strong isolation.
- Pro: enterprise noisy-neighbor risk disappears.
- Pro: custom budgets possible.
- Con: high cost and operational overhead.
- Con: not appropriate for every enterprise tenant.
- Con: capacity fragmentation.
- Tradeoff: excellent isolation but poor default economics.
- Partial accept: available as a pack/cell overlay for paid tenant_class workloads.

## Consequences

- Positive: shared warehouse capacity is protected.
- Positive: tenants see explicit budget dimensions.
- Positive: tenant_class maps to real resource envelopes.
- Positive: dashboards are isolated from export overload.
- Positive: overrides are governed and expiring.
- Negative: tenants can see 429 responses during bursts.
- Negative: budget tuning becomes product and FinOps work.
- Negative: multi-dimensional quotas are harder to test.
- Negative: tenant_class changes need rapid propagation.
- Neutral: dedicated warehouses remain a future overlay.
- Neutral: internal tenants still need their own budget.
- Follow-up work: implement tenant-facing budget meter.
- Follow-up work: add query cost estimator in dashboard builder.
- Follow-up work: add budget override runbook.
- Follow-up work: add capacity forecast tied to tenant_class mix.

## Implementation Notes

- Data shape `AnalyticsQueryBudgetV1` contains `tenant_id`.
- Data shape `AnalyticsQueryBudgetV1` contains `tenant_class`.
- Data shape `AnalyticsQueryBudgetV1` contains `dashboard_concurrency`.
- Data shape `AnalyticsQueryBudgetV1` contains `export_concurrency`.
- Data shape `AnalyticsQueryBudgetV1` contains `bytes_read_per_hour`.
- Data shape `AnalyticsQueryBudgetV1` contains `cpu_seconds_per_hour`.
- Data shape `AnalyticsQueryBudgetV1` contains `rows_read_per_hour`.
- Data shape `AnalyticsQueryBudgetV1` contains `override_expires_at`.
- Data shape `AnalyticsQueryBudgetV1` contains `policy_version`.
- Data shape `AnalyticsQueryAdmissionV1` contains `query_id`.
- Data shape `AnalyticsQueryAdmissionV1` contains `tenant_id`.
- Data shape `AnalyticsQueryAdmissionV1` contains `template_id`.
- Data shape `AnalyticsQueryAdmissionV1` contains `budget_dimension`.
- Data shape `AnalyticsQueryAdmissionV1` contains `estimated_bytes`.
- Data shape `AnalyticsQueryAdmissionV1` contains `estimated_cpu_seconds`.
- Data shape `AnalyticsQueryAdmissionV1` contains `slot_kind`.
- Data shape `AnalyticsQueryAdmissionV1` contains `decision`.
- API endpoint `POST /v1/query/{template_id}/admit` admits dashboard query.
- API endpoint `POST /v1/export/{template_id}/admit` admits export query.
- API endpoint `GET /v1/tenants/{tenant_id}/analytics/budget` returns budget state.
- API endpoint `POST /v1/internal/analytics/budget-overrides` creates override.
- API endpoint `DELETE /v1/internal/analytics/budget-overrides/{override_id}` revokes override.
- API endpoint `GET /v1/internal/analytics/budget-health` returns fleet saturation.
- ClickHouse quota name format is `tenant_{tenant_id_hash}_{tenant_class}`.
- ClickHouse `max_concurrent_queries_for_user` mirrors dashboard slots.
- ClickHouse `max_bytes_to_read` is set per query from hourly remaining budget.
- ClickHouse `max_execution_time` is set from CPU budget estimate.
- Valkey 8.0 stores hourly counters for admission speed.
- PostgreSQL 16.6 stores authoritative budget policy.
- Budget counters reset on UTC hour boundary.
- tenant_class-change event topic is `persistent://analytics/{pack_id}/tenant-class-change`.
- tenant_class-change consumer updates Valkey and ClickHouse users.
- Cedar principal for query admission is `Oyatie::Principal::Service("analytics-query-api")`.
- Cedar principal for export admission is `Oyatie::Principal::Service("analytics-export-worker")`.
- Cedar principal for override is `Oyatie::Principal::Service("analytics-sre-oncall")`.
- Cedar resource for budget is `Analytics::QueryBudget`.
- Example permit: principal `analytics-query-api`, action `analytics.query.admit`, resource `Analytics::QueryBudget::"tenant_01"`, context `{tenant_class:"paid", slot_kind:"dashboard", bytes_remaining_gib:120, concurrent_used:4, concurrent_limit:20}`.
- Example forbid: same action with context `{tenant_class:"demo_trial", slot_kind:"dashboard", concurrent_used:2, concurrent_limit:2}`.
- Example permit: principal `analytics-export-worker`, action `analytics.query.export_admit`, resource `Analytics::QueryBudget::"tenant_02"`, context `{tenant_class:"paid", billing_components:["per_usage"], export_slots_used:2, export_slots_limit:3}`.
- Example forbid: principal `analytics-sre-oncall`, action `analytics.query.budget_override`, context `{expires_hours:72}`.
- SLO `analytics-budget-propagation.openslo.yaml` sets tenant_class-change p99 <= 30 seconds.
- SLO `analytics-query-admission.openslo.yaml` sets admission p99 <= 50 ms.
- SLO `analytics-throttle-correctness.openslo.yaml` sets false-admit count to zero.
- Failure mode `budget_counter_unavailable` fails closed for exports and uses conservative dashboard limit.
- Failure mode `clickhouse_quota_update_failed` blocks tenant_class-change completion.
- Failure mode `override_expired_not_removed` opens Sev-2.
- Failure mode `dashboard_starved_by_export` opens Sev-1.
- Failure mode `budget_leak_between_tenants` opens Sev-1.

## Verification

- Test `demo_trial_budget_dashboard_concurrency_two` validates demo_trial tenant_class.
- Test `paid_budget_dashboard_concurrency_twenty` validates paid tenant_class.
- Test `paid_contract_overlay_dashboard_concurrency_hundred` validates paid contract overlay.
- Test `export_slots_separate_from_dashboard_slots` validates pool separation.
- Test `budget_overrun_returns_429` validates response.
- Test `429_includes_retry_after_budget_dimension_and_tenant_class` validates payload.
- Test `tenant_class_change_propagates_under_30_seconds` validates propagation.
- Test `override_expires_within_24_hours` validates override bound.
- Test `export_counter_unavailable_fails_closed` validates safety.
- Test `dashboard_counter_unavailable_uses_conservative_limit` validates availability.
- Test `clickhouse_quota_matches_budget_policy` validates quotas.
- Metric `oya_analytics_query_admission_ms` must meet p99 <= 50 ms.
- Metric `oya_analytics_budget_overrun_total` tracks 429 by dimension.
- Metric `oya_analytics_budget_false_admit_total` must remain zero.
- Metric `oya_analytics_tenant_class_change_propagation_ms` must meet p99 <= 30 seconds.
- Dashboard `analytics-query-budget.json` shows budget use by tenant_class.
- Dashboard `analytics-query-throttling.json` shows 429 and retry windows.
- Dashboard `analytics-export-slots.json` shows export pool saturation.
- CI check `analytics-budget-tenant-class-fixtures` validates tenant_class matrix.
- CI check `analytics-budget-cedar` validates permits and forbids.
- CI check `analytics-clickhouse-quota-render` validates rendered quotas.
- CI check `analytics-budget-openapi` validates 429 schema.
- Load test simulates 10,000 tenants and verifies no false admits.
- Chaos test kills Valkey counters and verifies fallback behavior.
- FinOps review exports budget burn monthly.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0193: Analytics storage, TTL, partition rotation, and cold tier.
- ADR-0195: Materialized views as stream-processing default.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0263: Observability emission contract.
- ClickHouse 26.3 LTS quotas documentation.
- ClickHouse settings for max bytes and max execution time.
- Valkey 8.0 documentation.
- PostgreSQL 16.6 documentation.
- RFC 6585: HTTP 429 Too Many Requests.
- SOC 2 CC7.2 and CC9.2.
- ISO/IEC 27001:2022 A.8.6 capacity management.
- NIST SP 800-53 Rev. 5 SC-6.
- FinOps Foundation allocation and shared-cost guidance.
