---
id: ADR-MS-001
title: Tenant OLAP freshness, dimensional completeness, and lineage contract for data-warehouse
status: Proposed
date: 2026-05-20
microservice: data-warehouse
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0005-eventing-backbone-outbox-pattern
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0199-per-tenant-cost-attribution-finops-substrate
  - ADR-0131-per-microservice-flat-layout
decision_owner: axis-data-warehouse + council-data
---

# ADR-MS-001: Tenant OLAP freshness, dimensional completeness, and lineage contract for data-warehouse

## Context

- Pressure name: analytical truth pressure.
- `data-warehouse` owns tenant OLAP serving, warehouse pipeline scheduling, SLA-tier enforcement, dimensional completeness, query serving, and lineage operations.
- Warehouse results feed analytics, finance, operations dashboards, customer reporting, audits, and data science.
- A warehouse that only stores tables without freshness and lineage controls cannot support regulated decision evidence.
- The service contract exposes `GET /data-warehouse/capabilities`.
- The service contract exposes `POST /data-warehouse/actions/{action_id}`.
- The AsyncAPI contract publishes `ActionAccepted`.
- Local policy files include `tenant-olap-authorization.cedar`, `data-residency.md`, `auditor-scope.cedar`, `ci-scope.cedar`, `abuse-defence.cedar`, and `emergency-services-bypass.cedar`.
- Local SLOs include availability, read latency, write latency, replay freshness, policy decision latency, audit emission lag, freshness, completeness, lineage capture completeness, query latency, schema drift detection, and SLA-tier breach alerting.
- Local dashboards include operating-bar overview, local audit completeness, local domain throughput, local operator remediation, local policy decisions, local SLO burn, compliance pack health, tenant cost capacity, and abuse outcomes.
- Constraint name: tenant OLAP isolation.
- Analytical queries must not leak rows, dimensions, lineage, or aggregate facts across tenants.
- Query acceleration must not collapse tenant, pack, or residency boundaries.
- Constraint name: freshness promise pressure.
- Tenants and downstream dashboards rely on clear freshness commitments.
- A stale warehouse slice must be visible as stale, not silently served as current.
- Constraint name: dimensional completeness pressure.
- Missing dimensions can distort financial, operational, or compliance reports.
- Completeness must be measured before dataset versions are marked ready.
- Constraint name: warehouse as consumer of data-pipeline.
- Data-pipeline owns movement and lineage capture for ingress.
- Data-warehouse owns OLAP materialization, dimensional conformance, serving, and warehouse-specific freshness.
- Constraint name: cost attribution pressure.
- ADR-0199 requires per-tenant cost attribution for compute and storage.
- Warehouse queries can create expensive shared compute; every action must carry tenant, query class, and resource budget.
- Constraint name: policy before query serving.
- Cedar must decide query, export, replay, and schedule actions before storage engine access.
- Denied queries emit evidence rather than becoming generic authorization failures.

## Decision

- Decision name: tenant-scoped warehouse action contract.
- `data-warehouse` will route external mutations and privileged reads through `POST /data-warehouse/actions/{action_id}`.
- Action families are `warehouse-schedule`, `sla-tier-enforcement`, `dimension-completeness`, `query-serving`, `lineage-operation`, `schema-drift-detection`, and `export-control`.
- Every action request must include tenant id, principal id, dataset id, warehouse id, action id, purpose, data class, pack overlay, SLA tier, idempotency key, trace context, and audit target.
- Warehouse schedules must include source dataset ids, expected freshness, materialization strategy, refresh window, and resource budget.
- SLA-tier enforcement must include tier, freshness objective, query latency objective, alert route, and breach action.
- Dimensional completeness checks must include dimension set id, required field set, completeness threshold, observed completeness, and quarantine behavior.
- Query serving actions must include query id, semantic model id, row-level policy version, projection list, filter digest, and result data class.
- Lineage operations must include upstream pipeline run ids, materialized table ids, transformation digest, and lineage completeness state.
- Schema drift detection must include source schema version, warehouse schema version, compatibility decision, and downstream impact list.
- Export control must include destination, export format, data class, retention policy, pack eligibility, and approver when required.
- The service will serve only tenant-scoped query plans.
- The service will reject queries when row-level policy version is missing or stale.
- The service will reject exports when destination pack or residency is not eligible.
- The service will quarantine materialized views when dimensional completeness falls below threshold.
- The service will mark stale dataset versions explicitly and prevent current-status dashboards from using them.
- The service will preserve lineage from data-pipeline input through warehouse materialization.
- The service will attach cost attribution to every query and refresh run.
- The service will emit `ActionAccepted` only after policy, idempotency, cost attribution, lineage, and audit target validation.
- Availability target is 0.999.
- Read latency target is 0.999 for defined good events.
- Write latency target is 0.999 for materialization and schedule operations.
- Replay freshness target is 0.999.
- Local freshness target follows `local-freshness.openslo.yaml`.
- Local completeness target follows `local-completeness.openslo.yaml`.
- Local query latency target follows `local-query-latency.openslo.yaml`.
- Lineage capture completeness target follows `local-lineage-capture-completeness.openslo.yaml`.
- SLA-tier breach alert target is 0.995.
- Schema drift detection target follows `local-schema-drift-detection.openslo.yaml`.
- Metrics may include tenant hash, warehouse id, semantic model id, action family, SLA tier, and outcome.
- Metrics must not include raw query text, customer attributes, or unredacted result fields.

## Alternatives Considered

### Alternative 1: Treat warehouse as just storage behind data-pipeline

- Pros: fewer service boundaries.
- Pros: data-pipeline already has lineage and replay logic.
- Cons: query serving, dimensional completeness, and SLA-tier enforcement need warehouse semantics.
- Cons: OLAP policy and cost attribution are not movement-only concerns.
- Cons: freshness and query latency require warehouse-local SLOs.
- Rejected because warehouse serving is its own governed microservice responsibility.

### Alternative 2: Use one global analytical lake for all tenants

- Pros: easier central optimization.
- Pros: simpler shared reporting.
- Cons: tenant and pack isolation become query filters instead of topology and policy.
- Cons: cross-tenant leakage risk increases.
- Cons: per-tenant cost attribution becomes approximate.
- Rejected because tenant OLAP isolation is mandatory.

### Alternative 3: Use vendor warehouse APIs as the public contract

- Pros: proven query engines and ecosystem tools.
- Pros: faster compatibility with BI clients.
- Cons: vendor API semantics expose provider-specific policy gaps.
- Cons: cost, residency, and lineage evidence differ per engine.
- Cons: service clients would depend on engine-specific behavior.
- Rejected because warehouse engines are adapters, not the Oyatie contract.

### Alternative 4: Serve stale data silently when refresh fails

- Pros: dashboards remain visually available.
- Pros: fewer visible incidents.
- Cons: users make decisions on false freshness.
- Cons: audit evidence becomes misleading.
- Cons: SLA-tier breach alerts lose meaning.
- Rejected because stale state must be explicit and policy-visible.

### Alternative 5: Enforce row-level security only in SQL views

- Pros: familiar warehouse pattern.
- Pros: efficient in mature engines.
- Cons: policy becomes engine-specific and hard to audit uniformly.
- Cons: exports and scheduled refreshes can bypass view access.
- Cons: Cedar decisions are missing from evidence chain.
- Rejected because Cedar must decide before storage engine access.

## Consequences

### Positive

- Tenants get explicit freshness, completeness, query latency, and lineage evidence.
- Downstream dashboards can reject stale or incomplete dataset versions.
- Warehouse costs can be attributed to tenant, dataset, query class, and refresh.
- Exports become policy-controlled actions rather than ad hoc downloads.
- Engine migration remains possible because clients use Oyatie action contracts.
- Row-level and pack policy decisions are auditable before query execution.
- Materialization failures can quarantine views without corrupting current reports.
- Data-pipeline lineage remains visible through warehouse serving.

### Negative

- Query serving must carry richer context than ordinary SQL clients expect.
- Warehouse adapters must translate Cedar decisions into engine-specific filters safely.
- Completeness checks can delay report availability.
- Explicit stale marking may increase visible incidents.
- Cost attribution metadata adds overhead to refresh and query runs.
- Export controls require review workflows for regulated packs.
- Semantic model and query id governance must be maintained.

### Neutral

- ClickHouse, DuckDB, BigQuery, Snowflake, Redshift, or Iceberg engines may be adapters.
- Data-pipeline remains the ingress and transform lineage owner.
- Analytics remains an interpreter of behavioral metrics, not warehouse authority.
- BI clients may connect through adapter endpoints after policy enforcement.
- Cached results are allowed when tenant, pack, policy version, and freshness match.

### Follow-up work

- Add semantic model registry for warehouse query-serving actions.
- Add freshness breach alert playbook by SLA tier.
- Add dimensional completeness fixture suite for finance and operational marts.
- Add export-control approval workflow for regulated packs.
- Add query-cost attribution dashboard tied to ADR-0199.
- Add stale dataset rejection tests in downstream dashboards.

## Implementation Notes

### Data Shapes

- `WarehouseActionRequest` fields: `tenant_id`, `principal_id`, `action_id`, `dataset_id`, `warehouse_id`, `purpose`, `data_class`, `pack_overlay`, `sla_tier`, `idempotency_key`, `traceparent`, `audit_target`.
- `WarehouseSchedule` fields: `schedule_id`, `source_dataset_ids`, `expected_freshness_seconds`, `materialization_strategy`, `refresh_window`, `resource_budget_id`.
- `SlaTierPolicy` fields: `sla_tier`, `freshness_objective_seconds`, `query_latency_ms`, `alert_route`, `breach_action`, `owner_team`.
- `DimensionCompletenessCheck` fields: `dimension_set_id`, `required_fields`, `threshold`, `observed_completeness`, `quarantine_behavior`, `evidence_id`.
- `QueryServingAction` fields: `query_id`, `semantic_model_id`, `row_policy_version`, `projection_list`, `filter_digest`, `result_data_class`, `cache_key`.
- `LineageMaterialization` fields: `materialization_id`, `pipeline_run_ids`, `table_ids`, `transform_digest`, `lineage_completeness`, `evidence_id`.
- `SchemaDriftDecision` fields: `source_schema_version`, `warehouse_schema_version`, `compatibility`, `impact_list`, `quarantine_state`.
- `ExportControl` fields: `export_id`, `destination`, `format`, `data_class`, `retention_policy`, `pack_eligibility`, `approver_id`.
- `ActionAccepted` event fields: `tenant_id_hash`, `action_id`, `dataset_id`, `warehouse_id`, `policy_version`, `lineage_id`, `cost_attribution_id`, `evidence_id`.

### API Endpoints

- `GET /data-warehouse/capabilities` lists warehouse actions, SLA tiers, semantic model support, and pack constraints.
- `POST /data-warehouse/actions/warehouse-schedule.create` creates or updates a refresh schedule.
- `POST /data-warehouse/actions/sla-tier-enforcement.create` binds dataset to SLA tier.
- `POST /data-warehouse/actions/dimension-completeness.create` evaluates required dimensions.
- `POST /data-warehouse/actions/query-serving.create` executes a policy-gated query action.
- `POST /data-warehouse/actions/lineage-operation.create` seals materialization lineage.
- `POST /data-warehouse/actions/schema-drift-detection.create` records compatibility and impact.
- `POST /data-warehouse/actions/export-control.create` approves or denies export.
- Every action returns `warehouse_action_id`, `dataset_version`, `freshness_state`, `lineage_id`, `cost_attribution_id`, and `evidence_id`.

### Cedar Policies

- `policy/tenant-olap-authorization.cedar` authorizes tenant-scoped OLAP query and export actions.
- `policy/data-residency.md` binds dataset, query, and export to tenant pack.
- `policy/auditor-scope.cedar` allows evidence and lineage review without raw result access.
- `policy/ci-scope.cedar` allows fixture and contract validation.
- `policy/abuse-defence.cedar` protects expensive query paths from abuse.
- `policy/emergency-services-bypass.cedar` cannot bypass row-level, tenant, or pack policy.
- Policy must deny query serving when row-level policy version is missing.
- Policy must deny export when destination pack is ineligible.
- Policy must deny cache reuse when freshness, tenant, pack, or policy version differs.

### SLO Targets

- `availability.openslo.yaml`: warehouse availability target 0.999.
- `read-latency.openslo.yaml`: read-latency target 0.999.
- `write-latency.openslo.yaml`: write-latency target 0.999.
- `replay-freshness.openslo.yaml`: replay freshness target 0.999.
- `audit-emission-lag.openslo.yaml`: audit emission lag target 0.999.
- `local-freshness.openslo.yaml`: freshness objective for warehouse schedules.
- `local-completeness.openslo.yaml`: dimensional completeness objective.
- `local-lineage-capture-completeness.openslo.yaml`: lineage capture completeness objective.
- `local-query-latency.openslo.yaml`: local query latency objective.
- `local-schema-drift-detection.openslo.yaml`: schema drift detection objective.
- `local-sla-tier-breach-alert.openslo.yaml`: SLA-tier breach alert target 0.995.

## Verification

- Unit test `warehouse_action_requires_tenant_dataset_sla_and_audit_target`.
- Unit test `query_serving_requires_row_policy_version`.
- Unit test `dimension_completeness_quarantines_below_threshold`.
- Unit test `export_control_requires_destination_pack_eligibility`.
- Unit test `cache_key_includes_tenant_pack_policy_and_freshness`.
- Property test `query_cache_never_reuses_cross_tenant_result`.
- Property test `freshness_state_monotonic_until_new_materialization`.
- Cedar test `tenant_olap_denies_cross_tenant_query`.
- Cedar test `tenant_olap_denies_missing_row_policy_version`.
- Cedar test `data_residency_denies_ineligible_export_destination`.
- Cedar test `auditor_scope_reads_lineage_without_result_payload`.
- Contract test `openapi-v1.yaml_contains_capabilities_and_actions`.
- Contract test `asyncapi-v1.yaml_publishes_action_accepted`.
- Integration test `query_serving_emits_policy_and_cost_evidence`.
- Integration test `materialization_requires_lineage_completeness`.
- Integration test `stale_dataset_marked_not_current`.
- Integration test `sla_breach_alert_emits_with_tier`.
- Integration test `export_control_denies_cross_pack_destination`.
- Integration test `schema_drift_quarantines_impacted_model`.
- Load test `query_latency_meets_local_query_latency_target`.
- Load test `freshness_meets_local_freshness_target`.
- Load test `lineage_capture_completeness_meets_target`.
- Load test `sla_breach_alert_meets_0995_target`.
- Chaos test `warehouse_engine_unavailable_preserves_stale_marker`.
- Chaos test `policy_engine_unavailable_blocks_export`.
- Chaos test `lineage_sink_unavailable_blocks_current_dataset_status`.
- Replay test `warehouse_replay_does_not_double_charge_cost_attribution`.
- Metric `oya_data_warehouse_availability_good_total`.
- Metric `oya_data_warehouse_query_latency_good_total`.
- Metric `oya_data_warehouse_freshness_good_total`.
- Metric `oya_data_warehouse_completeness_good_total`.
- Metric `oya_data_warehouse_lineage_capture_completeness_good_total`.
- Metric `oya_data_warehouse_sla_tier_breach_alert_good_total`.
- Dashboard `dashboards/local-audit-completeness.json`.
- Dashboard `dashboards/local-domain-throughput.json`.
- Dashboard `dashboards/local-policy-decisions.json`.
- Dashboard `dashboards/local-slo-burn.json`.
- Dashboard `dashboards/tenant-cost-and-capacity.json`.
- Dashboard `dashboards/compliance-pack-health.json`.
- Runbook check `runbooks/freshness-breach.md` covers stale marker and alert.
- Runbook check `runbooks/export-policy-denial.md` covers regulated export review.
- Promotion gate blocks if current dataset lacks lineage completeness.
- Promotion gate blocks if query cache omits tenant, pack, policy, or freshness key.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0005: Eventing backbone outbox pattern.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0199: Per-tenant cost attribution FinOps substrate.
- Oyatie ADR-0131: Per-microservice flat layout.
- OpenLineage specification.
- Apache Iceberg table specification.
- Delta Lake transaction log protocol documentation.
- ClickHouse documentation: row policy, projections, and materialized views.
- DuckDB documentation: analytical engine and replacement scans.
- Google BigQuery documentation: row-level security and authorized views.
- Snowflake documentation: secure views, masking policies, and access history.
- Amazon Redshift documentation: row-level security and materialized views.
- RFC 9110: HTTP Semantics.
- Google SRE Workbook: SLOs and alerting.
- Cedar policy language documentation.
