# IP-033 Data Pipeline semantic-layer metrics registration

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-033-semantic-layer.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2 (semantic-layer missing), §3.9.3
Benchmarks: dbt Cloud (metrics:, MetricFlow semantic layer), Cube (semantic layer), Looker (LookML semantic model), AtScale (universal semantic layer)
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0251, ADR-0252, ADR-0253, ADR-0254, ADR-0255, ADR-0314, ADR-0321, ADR-0329, ADR-0330, ADR-0331

## Objective
- Cover the dbt Cloud-shaped semantic layer surface that the audit flagged as a missing primitive in §3.9.2.
- Register tenant-scoped metric definitions, dimensions, time grains, joins, and entity bindings inside data-pipeline so that downstream `analytics` and `ontology` µservices read a single canonical metric registry rather than recomputing.
- Treat semantic-layer metrics as first-class transformed datasets: every metric has a transform_run_id lineage, a destination_load_run for materialized variants, and a Cedar-gated read path.
- Make the semantic layer pack-aware: KR-PIPA / HIPAA-2024 / GDPR may restrict the dimensions a metric can carry per tenant.
- Keep data-pipeline as the metric-definition owner; downstream µservices consume the registry, never override it.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §A and §C (transform context).
- Read `microservices/data-pipeline/ARCHITECTURE.md` §C transform context.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2 missing dbt Cloud surface.
- Read `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` semantic-layer row.
- Read `microservices/data-pipeline/IP-031-destination-connector.md` for destination commit semantics.
- Read `microservices/data-pipeline/IP-035-materialization-families.md` for materialization integration.
- Read `microservices/ontology/manifest.json` for the ontology consumer side.
- Read `microservices/analytics/manifest.json` for the analytics consumer side.

## Domain model
- Aggregate: `semantic_metric_definition`.
- Identity: `tenant_id + metric_name + metric_version`.
- Sub-aggregate: `metric_dimension_binding` (one row per allowed dimension).
- Sub-aggregate: `metric_time_grain_binding` (one row per time grain offered).
- Sub-aggregate: `metric_entity_join_binding` (one row per join key into ontology entities).
- Sub-aggregate: `metric_materialization_pointer` (links to IP-035 materialization family).
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR`, `oyatie.foundry.semantic_steward`, or `tenant_data_steward` audience.
- Required policy decision: Cedar permit from `local-semantic-metric-define-scope.cedar` and `local-semantic-metric-read-scope.cedar`.
- Required evidence: metric expression, source transform_run_id, semantic version, pack-aware dimension allow-list.
- Required custody: deprecated metric versions stay readable for replay purposes.

## Metric definition shape
- `metric_name`: BNF v4.1 kebab-case (e.g., `gross-merchandise-value`, `monthly-active-users`).
- `metric_version`: semver MAJOR.MINOR.PATCH; breaking changes increment MAJOR.
- `metric_kind`: `simple`, `ratio`, `derived`, `cumulative`, `conversion`, `funnel`.
- `expression`: declarative metric expression in oyatie-canonical metric DSL (a subset of MetricFlow with explicit tenant binding).
- `time_dimension`: required field name for time-grain aggregation.
- `time_grains_allowed`: subset of {`minute`, `hour`, `day`, `week`, `month`, `quarter`, `year`}.
- `dimensions_allowed`: tenant-scoped allow-list of dimension fields (pack-restricted).
- `entity_joins`: list of ontology entity types and join expressions.
- `materialization_policy`: `view`, `incremental`, `materialized_table`, `cached_query` (binds to IP-035).
- `exposure_refs`: list of exposure_ids that consume this metric (binds to IP-034).
- `pack_restriction_overlay`: per-pack dimension and grain allow-list.

## Implementation steps
- Add `semantic-layer` sub-context within the `transform` bounded context (rather than a new top-level context) per ADR-0132 no-grouping policy and audit §6.2.
- Add `src/domain/semantic_metric.rs` with `SemanticMetricDefinition`, `MetricKind` enum, `MetricExpression` parser.
- Add `src/usecase/semantic_metric.rs` exposing `metric.define`, `metric.amend`, `metric.approve`, `metric.deprecate`, `metric.query_plan`, `metric.read`.
- Add `src/adapter/metric_expression_parser.rs` (canonical metric DSL).
- Add `local-semantic-metric-define-scope.cedar` and `local-semantic-metric-read-scope.cedar` to `policies/`.
- Add `oya.data.pipeline.semantic_metric.defined`, `.amended`, `.approved`, `.deprecated`, `.queried` to AsyncAPI surface.
- Add `capabilities/semantic-metric-define.yaml` and `capabilities/semantic-metric-read.yaml`.
- Add `catalog/oya-data-pipeline-transform-semantic-metric-domain.yaml`.
- Publish `contracts/semantic-metric-registry-v1.yaml` (REST + gRPC mirror) consumed by `analytics` and `ontology`.
- Add SLO `local-semantic-metric-read-latency.openslo.yaml` (p95 read latency 500ms for cached, 5s for incremental, 60s for materialized refresh trigger).
- Add runbook `semantic-metric-drift.md` for semantic version conflicts.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `metric_name` is mandatory.
- `metric_version` is mandatory.
- `metric_kind` is mandatory.
- `expression_normalized_hash` is mandatory.
- `time_dimension` is mandatory.
- `time_grains_allowed` is mandatory.
- `dimensions_allowed` is mandatory.
- `entity_joins` is mandatory.
- `materialization_policy` is mandatory.
- `pack_restriction_overlay` is mandatory when any pack restricts metric dimensions.
- `cedar_decision_id` is mandatory on define/amend/approve/deprecate.
- `audit_event_id` is mandatory.
- `transform_run_id_source` is mandatory when metric is derived from a transform.

## Policy gates
- Cedar denies metric.define without tenant scope.
- Cedar denies metric.define if metric_name collides with a deprecated metric version still in custody window.
- Cedar denies metric.define if expression references a column not allowed for the tenant's pack overlay.
- Cedar denies metric.define if entity_joins reference an ontology entity outside the tenant's ontology partition.
- Cedar denies metric.amend if amendment crosses MAJOR boundary without approval.
- Cedar denies metric.approve if expression fails parse validation.
- Cedar denies metric.read if requesting dimensions outside dimensions_allowed.
- Cedar denies metric.read if requesting time_grain outside time_grains_allowed.
- Cedar denies metric.read if requestor lacks `tenant_data_consumer` or higher audience.
- Cedar denies metric.deprecate without an advisory grace_window_days >= 30 unless explicit operator override.

## Pack-overlay restrictions
- KR-PIPA: PII-derived dimensions (e.g., resident_registration_number_suffix) cannot be exposed in metric reads even if registered.
- HIPAA-2024: PHI-derived dimensions require minimum_aggregation_count to prevent re-identification.
- GDPR: lawful_basis tag is required on every dimension that joins a person entity.
- PCI-DSS-L1-v4: PAN-related dimensions are forbidden entirely from semantic metrics.
- SOC-2: any semantic metric must reference at least one ingest source with a documented control objective.
- ISO-27001: semantic metric definition mutations require change-record evidence.

## Benchmark displacement
- dbt Cloud `metrics:` parity means tenant-scoped metric registry with name, expression, type, time_grains, dimensions, joins.
- MetricFlow parity means semantic version handling, deprecation flow, and expression normalization.
- Cube semantic-layer parity means a registry consumed by multiple downstream tools (in oyatie's case, `analytics` and `ontology`).
- Looker LookML parity means tenant-scoped role-based dimension restriction (oyatie does it via Cedar + pack overlay rather than LookML access_filters).
- AtScale universal-semantic-layer parity means cross-tool consistency: oyatie metrics are computed once and read everywhere.
- Vendor names do not become canonical oyatie metric names; they remain reference pressure.

## Failure handling
- If expression parser fails, reject define and emit refusal evidence.
- If entity join target ontology entity is unavailable, hold define and link `runbooks/lineage-gap-repair.md`.
- If pack overlay loader fails, fail closed: define denied.
- If materialization (IP-035) fails for the metric, mark metric_version as `materialization_failed` and force fall-back to `view` policy with an alert.
- If a read crosses dimensions_allowed boundary, emit refusal evidence and surface to the operator.
- If Cedar is unavailable, fail closed for write; reads may serve cached metric data with a stale-banner.
- If audit-chain is unavailable, hold mutation.

## Tests and evidence
- Unit test: metric expression parser rejects unbalanced expressions.
- Unit test: semantic version comparator rejects backward MAJOR transitions without approval.
- Contract test: metric.define rejects metric_kind outside allowed enum.
- Contract test: metric.read rejects unauthorized dimension.
- Policy test: KR-PIPA pack blocks resident_registration_number_suffix dimension.
- Policy test: HIPAA-2024 pack enforces minimum_aggregation_count.
- Replay test: deprecated metric version still readable during grace window.
- SLO test: local-semantic-metric-read-latency burn opens runbook.
- Audit test: define and approve share correlation id.
- Cross-microservice test: ontology entity join resolution.
- Cross-microservice test: analytics metric read pulls from semantic-metric-registry-v1 contract.

## Rollback
- Roll back metric definition by creating a `rolled_back` amendment.
- Preserve every prior metric_version as evidence (append-only registry).
- If a metric materialization (IP-035) exists, roll it back via destination_load_run rollback (IP-031).
- Recompute downstream exposure (IP-034) status to reflect rollback.
- Emit `oya.data.pipeline.semantic_metric.rolled_back`.
- Link rollback to `runbooks/semantic-metric-drift.md`.
- Notify `analytics` and `ontology` consumers via AsyncAPI.

## Acceptance criteria
- Semantic metric registry lives under the `transform` bounded context (no new top-level context).
- Metric define / amend / approve / deprecate / read commands exist in OpenAPI and gRPC contracts.
- Cedar gates enforce tenant scope, pack overlay, and audience.
- `contracts/semantic-metric-registry-v1.yaml` is published and consumed by `analytics` and `ontology`.
- SLO `local-semantic-metric-read-latency.openslo.yaml` exists with target numbers.
- Runbook `semantic-metric-drift.md` exists.
- Audit chain emits define / amend / approve / deprecate / queried events with correlation id.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` semantic-layer row.
- `microservices/data-pipeline/IP-031-destination-connector.md` destination commit.
- `microservices/data-pipeline/IP-034-exposure-tracking.md` exposure binding.
- `microservices/data-pipeline/IP-035-materialization-families.md` materialization policy.
- `ADR-0245` substrate vs product layering.
- `ADR-0251` compliance pack primitive.
- `ADR-0321` documentation rigor.

## Operator review prompts
- Reviewer asks whether metric_kind is the simplest fit.
- Reviewer asks whether time_dimension is the canonical event-time column.
- Reviewer asks whether dimensions_allowed respects pack overlay.
- Reviewer asks whether entity_joins point at the correct ontology entities.
- Reviewer asks whether materialization_policy fits expected query patterns.
- Reviewer asks whether deprecation grace window is sufficient.
- Reviewer asks whether exposure refs are consistent with downstream tooling.
- Reviewer signs the metric case with the same audit correlation id.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-033-semantic-layer.md:62` - - Add SLO `local-semantic-metric-read-latency.openslo.yaml` (p95 read latency 500ms for cached, 5s for incremental, 60s for materialized refresh trigger).; `microservices/data-pipeline/IP-033-semantic-layer.md:96` - - HIPAA-2024: PHI-derived dimensions require minimum_aggregation_count to prevent re-identification..
