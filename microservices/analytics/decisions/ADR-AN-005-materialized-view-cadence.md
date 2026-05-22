---
id: ADR-AN-005
title: "Materialized views use governed cadence, naming, and chain-depth limits"
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

# ADR-AN-005: Materialized views use governed cadence, naming, and chain-depth limits

## Context

- The named architectural pressure is `streaming-aggregation-with-debuggable-lineage`.
- ADR-0195 establishes ClickHouse materialized views as the default stream-processing tier.
- ADR-0193 establishes analytics storage, TTL, and partitioning constraints.
- ADR-AN-002 establishes partition strategy.
- ADR-AN-003 establishes tenant isolation.
- Prior incident class `free-form-mv-proliferation` created views nobody could trace.
- Prior incident class `deep-chain-latency-hidden` hid dashboard staleness behind chained views.
- Prior incident class `mv-name-no-cadence` made freshness impossible to infer.
- Prior incident class `tenant-id-dropped-in-mv` created cross-tenant aggregate leakage.
- Dashboards need predictable freshness.
- Billing rollups need deterministic day and month aggregation.
- Anomaly windows need minute or five-minute buckets.
- Fleet operations need some fleet-internal rollups.
- ClickHouse `AggregatingMergeTree` state functions require consistent naming.
- MV source and target lineage must be visible to operators.
- MV cadence must align to retention and partition strategy.
- MV chain depth must be limited to keep failures debuggable.
- MV naming must encode cadence.
- MV naming must encode entity and dimension.
- MV definitions must be generated from templates.
- MV definitions must preserve tenant_id, pack_id, and data_class.
- MV definitions must be Cedar-reviewed before becoming tenant-visible.
- The implementation must be buildable from this ADR.

## Decision

- We choose `governed materialized-view cadence`.
- The named pattern is `cadence-prefixed MV catalog with chain-depth cap`.
- ClickHouse version is 26.3 LTS.
- Default target engine is `AggregatingMergeTree`.
- Sum aggregations use `sumState` and `sumMerge`.
- Count aggregations use `countState` and `countMerge`.
- Percentile aggregations use `quantilesState` and `quantilesMerge`.
- Top-K aggregations use `topKState` and `topKMerge`.
- Threshold emitter views may use `MergeTree`.
- Materialized-view name format is `mv_${cadence}_${entity}_${dimension}`.
- Target table name format is `${entity}_${cadence}`.
- Projection-only views use prefix `mv_proj_`.
- Cadence `minute` means one-minute buckets.
- Cadence `five_minute` means five-minute buckets.
- Cadence `hour` means one-hour buckets.
- Cadence `day` means one-day buckets.
- Cadence `month` means one-month buckets.
- L1 minute views are rare and require council-analytics approval.
- L2 five-minute views are used for percentile and top-K dashboard windows.
- L3 hourly views are the default tenant dashboard cadence.
- L4 daily views are the default billing daily cadence.
- L5 monthly views are the default billing monthly cadence.
- Maximum MV chain depth is 2.
- L4 daily to L5 monthly is allowed.
- L1 minute to L2 five-minute is allowed.
- L2 five-minute to L3 hourly is allowed only by exception.
- Anything deeper than 2 is forbidden.
- Every MV declares `source_table`.
- Every MV declares `target_table`.
- Every MV declares `cadence`.
- Every MV declares `chain_depth`.
- Every MV declares `freshness_slo_seconds`.
- Every MV declares `owner`.
- Every MV preserves `tenant_id`.
- Every MV preserves `pack_id`.
- Every MV preserves `data_class`.
- Cedar action `analytics.mv.register` gates MV registration.
- Cedar action `analytics.mv.deploy` gates MV deployment.
- Cedar action `analytics.mv.backfill` gates MV backfill.
- Cedar action `analytics.mv.drop` gates MV removal.

## Alternatives Considered

### Free-form MV cadence and names

- Pro: fastest for individual teams.
- Pro: no central registry.
- Pro: flexible for experiments.
- Con: operators cannot infer freshness.
- Con: duplicate views proliferate.
- Con: chain depth becomes invisible.
- Con: tenant-id preservation is not guaranteed.
- Tradeoff: team flexibility but poor operations.
- Rejected.

### Single hourly cadence for every MV

- Pro: very simple.
- Pro: good default for dashboards.
- Pro: low partition churn.
- Con: anomaly detection needs minute windows.
- Con: billing daily and monthly need native cadences.
- Con: high-freshness tenants would bypass the system.
- Tradeoff: simple but insufficient workload coverage.
- Rejected.

### Deep chained MV hierarchy

- Pro: minimizes direct reads from raw tables.
- Pro: can reuse intermediate state.
- Pro: may reduce CPU.
- Con: failures are hard to trace.
- Con: staleness compounds.
- Con: backfills become risky.
- Tradeoff: storage and CPU efficiency but poor debuggability.
- Rejected.

### External stream processor for every aggregation

- Pro: clear streaming semantics.
- Pro: mature options like Flink.
- Pro: lower ClickHouse MV coupling.
- Con: adds Apache Flink operations to M03.
- Con: duplicates state management.
- Con: harder to align with tenant row policies.
- Tradeoff: powerful but unnecessary platform complexity.
- Rejected.

### Hand-written SQL in application repos

- Pro: teams own their views.
- Pro: no template system.
- Pro: easy initial commits.
- Con: no global naming enforcement.
- Con: no cadence catalog.
- Con: no central CI fixtures.
- Tradeoff: local ownership but weak governance.
- Rejected.

## Consequences

- Positive: MV freshness is visible in names and metadata.
- Positive: chain-depth cap keeps failures debuggable.
- Positive: tenant and pack preservation is testable.
- Positive: operators can map source to target quickly.
- Positive: billing and anomaly workloads get fit-for-purpose cadence.
- Negative: teams must register new MVs before deployment.
- Negative: L1 minute views require review.
- Negative: template governance slows experiments.
- Negative: MV backfills need explicit plans.
- Neutral: Flink remains an option for future complex stream processing.
- Neutral: fleet-internal views follow the same catalog but separate Cedar actions.
- Follow-up work: implement MV catalog table.
- Follow-up work: add MV lineage dashboard.
- Follow-up work: add MV template linter.
- Follow-up work: add backfill runbook for chain-depth changes.

## Implementation Notes

- Data shape `AnalyticsMaterializedViewV1` contains `mv_name`.
- Data shape `AnalyticsMaterializedViewV1` contains `source_table`.
- Data shape `AnalyticsMaterializedViewV1` contains `target_table`.
- Data shape `AnalyticsMaterializedViewV1` contains `cadence`.
- Data shape `AnalyticsMaterializedViewV1` contains `bucket_width_seconds`.
- Data shape `AnalyticsMaterializedViewV1` contains `chain_depth`.
- Data shape `AnalyticsMaterializedViewV1` contains `freshness_slo_seconds`.
- Data shape `AnalyticsMaterializedViewV1` contains `tenant_scope`.
- Data shape `AnalyticsMaterializedViewV1` contains `data_class`.
- Data shape `AnalyticsMaterializedViewV1` contains `owner`.
- Data shape `AnalyticsMaterializedViewV1` contains `approval_id`.
- Data shape `AnalyticsMaterializedViewV1` contains `ddl_hash`.
- Data shape `AnalyticsMvBackfillPlanV1` contains `backfill_id`.
- Data shape `AnalyticsMvBackfillPlanV1` contains `mv_name`.
- Data shape `AnalyticsMvBackfillPlanV1` contains `window_start`.
- Data shape `AnalyticsMvBackfillPlanV1` contains `window_end`.
- Data shape `AnalyticsMvBackfillPlanV1` contains `rate_limit_rows_per_second`.
- Data shape `AnalyticsMvBackfillPlanV1` contains `rollback_target`.
- Cadence `minute` has `bucket_width_seconds=60`.
- Cadence `five_minute` has `bucket_width_seconds=300`.
- Cadence `hour` has `bucket_width_seconds=3600`.
- Cadence `day` has `bucket_width_seconds=86400`.
- Cadence `month` uses calendar month bucket.
- Example MV name is `mv_hour_workflow_per_tenant`.
- Example MV name is `mv_minute_error_burst_per_tenant`.
- Example MV name is `mv_day_billing_per_resource`.
- Example target table is `workflow_hour`.
- Example target table is `error_burst_minute`.
- Example target table is `billing_day`.
- API endpoint `GET /v1/internal/materialized-views` lists catalog.
- API endpoint `POST /v1/internal/materialized-views/register` registers MV metadata.
- API endpoint `POST /v1/internal/materialized-views/deploy` deploys DDL.
- API endpoint `POST /v1/internal/materialized-views/{mv_name}/backfill` starts backfill.
- API endpoint `DELETE /v1/internal/materialized-views/{mv_name}` drops MV through governance.
- API endpoint `GET /v1/internal/materialized-views/{mv_name}/lineage` returns source and target.
- MV templates live under `microservices/analytics/iac/clickhouse/mv-templates/`.
- DDL rendering uses deterministic template rendering.
- DDL hash uses SHA-256 over canonical SQL normalization.
- ClickHouse target table engine is `AggregatingMergeTree` for stateful aggregates.
- ClickHouse target table engine is `MergeTree` for threshold emitters.
- ORDER BY begins with `(tenant_id, bucket_start)`.
- PARTITION BY follows ADR-AN-002.
- TTL follows ADR-AN-001.
- Cedar principal for registration is `Oyatie::Principal::Service("analytics-ddl-controller")`.
- Cedar principal for deployment is `Oyatie::Principal::Service("analytics-ddl-controller")`.
- Cedar principal for backfill is `Oyatie::Principal::Service("analytics-backfill-worker")`.
- Cedar resource is `Analytics::MaterializedView`.
- Example permit: principal `analytics-ddl-controller`, action `analytics.mv.register`, resource `Analytics::MaterializedView::"mv_hour_workflow_per_tenant"`, context `{cadence:"hour", chain_depth:1, preserves_tenant_id:true}`.
- Example permit: principal `analytics-backfill-worker`, action `analytics.mv.backfill`, resource `Analytics::MaterializedView::"mv_day_billing_per_resource"`, context `{rate_limit_rows_per_second:50000, rollback_target:"billing_day_old"}`.
- Example forbid: registration with context `{chain_depth:3}`.
- Example forbid: deployment with context `{preserves_tenant_id:false}`.
- SLO `analytics-mv-freshness.openslo.yaml` sets hourly MV p95 freshness <= 300 seconds.
- SLO `analytics-mv-minute-freshness.openslo.yaml` sets minute MV p95 freshness <= 30 seconds.
- SLO `analytics-mv-backfill.openslo.yaml` sets approved backfill p99 completion <= 24 hours.
- Failure mode `mv_chain_depth_exceeded` blocks deploy.
- Failure mode `mv_tenant_id_missing` blocks deploy.
- Failure mode `mv_freshness_breach` pages by cadence severity.
- Failure mode `mv_backfill_over_budget` throttles backfill.
- Failure mode `mv_name_collision` blocks registration.

## Verification

- Test `mv_name_requires_cadence_prefix` validates naming.
- Test `mv_target_name_matches_entity_cadence` validates target naming.
- Test `mv_chain_depth_max_two` validates chain cap.
- Test `mv_hour_default_for_dashboard` validates default cadence.
- Test `mv_day_billing_allowed` validates billing cadence.
- Test `mv_month_from_day_allowed` validates chain exception.
- Test `mv_preserves_tenant_id_pack_id_data_class` validates isolation.
- Test `mv_register_requires_cedar` validates policy.
- Test `mv_backfill_requires_plan` validates backfill governance.
- Test `mv_ddl_hash_is_deterministic` validates rendering.
- Test `mv_name_collision_rejected` validates catalog.
- Test `minute_mv_requires_approval` validates review.
- Metric `oya_analytics_mv_freshness_seconds` tracks freshness by cadence.
- Metric `oya_analytics_mv_chain_depth` tracks depth.
- Metric `oya_analytics_mv_backfill_rows_per_second` tracks backfill.
- Metric `oya_analytics_mv_deploy_total` tracks deployments.
- Dashboard `analytics-materialized-views.json` shows catalog and freshness.
- Dashboard `analytics-mv-lineage.json` shows source-to-target graph.
- Dashboard `analytics-mv-backfill.json` shows backfill status and throttling.
- CI check `oya-governance-mv-naming` validates naming.
- CI check `analytics-mv-chain-depth` validates depth.
- CI check `analytics-mv-tenant-preservation` validates tenant_id, pack_id, and data_class.
- CI check `analytics-mv-template-determinism` renders templates twice.
- CI check `analytics-mv-cedar` validates permits and forbids.
- Load test runs 10 million source rows into hourly MV under freshness SLO.
- Chaos test pauses MV consumer and expects freshness alert.
- Backfill drill runs quarterly on `mv_hour_workflow_per_tenant`.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0193: Analytics storage, TTL, partition rotation, and cold tier.
- ADR-0195: Materialized views as stream-processing default.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0263: Observability emission contract.
- ADR-AN-001: Analytics TTL policy.
- ADR-AN-002: Analytics partition strategy.
- ADR-AN-003: Row-level tenant isolation.
- ClickHouse 26.3 LTS Materialized View documentation.
- ClickHouse AggregatingMergeTree documentation.
- ClickHouse aggregate function combinators documentation.
- PostgreSQL 16.6 documentation.
- Cedar policy language documentation.
- NIST SP 800-53 Rev. 5 AU-12.
- ISO/IEC 27001:2022 A.8.16.
- SOC 2 CC7.2.
