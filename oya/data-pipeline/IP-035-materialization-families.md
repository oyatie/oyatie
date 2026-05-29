# IP-035 Data Pipeline materialization families

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-035-materialization-families.md
Authored: 2026-05-21
Source audit: microservices/data-pipeline/coherence-audit-2026-05-20.md §3.9.2 (materialization families missing), §3.9.3
Benchmarks: dbt Cloud (`materialized: table | view | incremental | ephemeral | snapshot`), Snowflake (Dynamic Tables), Databricks (Delta Live Tables), Materialize (continuous SQL), Iceberg (incremental views)
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0242, ADR-0243, ADR-0244, ADR-0245, ADR-0247, ADR-0248, ADR-0251, ADR-0252, ADR-0253, ADR-0254, ADR-0255, ADR-0314, ADR-0321, ADR-0329, ADR-0330, ADR-0331

## Objective
- Cover the dbt Cloud-shaped materialization families surface flagged missing in audit §3.9.2.
- Name the five canonical materialization families and bind each one to a domain rule, a Cedar policy, a destination_load_run shape (IP-031), and a cost dimension (IP-017).
- Make materialization choice declarative: a transform or semantic metric declares its policy and the materializer adapter resolves it.
- Make every materialization tenant-, cell-, and pack-aware so that residency overlays cannot be silently bypassed.
- Bind incremental materialization to IP-030 CDC watermarks so that the incremental cursor advances are evidence-bearing.

## Materialization families (resolves audit §3.9.2)
- `view`: query is a logical view in the destination; no data is duplicated; reads recompute every time.
- `table`: query result is materialized as a full destination table; refreshed on schedule or trigger.
- `incremental`: query result is appended/merged into a destination table based on a watermark cursor (IP-030).
- `ephemeral`: query result is inlined into downstream queries; no destination object exists (analogue to dbt ephemeral CTE).
- `snapshot`: query result is captured into an SCD2 history table on a cadence; preserves change history with `valid_from`, `valid_to` columns.

## Prerequisites
- Read `microservices/data-pipeline/PRD.md` §C transform context.
- Read `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- Read `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` materialization families row.
- Read `microservices/data-pipeline/IP-029-transform-cost-attribution.md` for cost dimensions.
- Read `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` for incremental cursor binding.
- Read `microservices/data-pipeline/IP-031-destination-connector.md` for destination commit semantics.
- Read `microservices/data-pipeline/IP-033-semantic-layer.md` for metric materialization_policy binding.

## Domain model
- Aggregate: `materialization_policy_binding`.
- Identity: `tenant_id + transform_id_or_metric_id + materialization_version`.
- Required actor: `principal_id` with `DATA_PIPELINE_OPERATOR`, `tenant_data_steward`, or `oyatie.foundry.materialization_curator` audience.
- Required policy decision: Cedar permit from `local-materialization-define-scope.cedar` and `local-materialization-promote-scope.cedar`.
- Required family: one of `view`, `table`, `incremental`, `ephemeral`, `snapshot`.
- Required destination_id: required for `view`, `table`, `incremental`, `snapshot`; forbidden for `ephemeral`.
- Required refresh_policy: cron string, interval seconds, sensor predicate, or event match (binds to IP-032 schedule).
- Required watermark_binding: required for `incremental`; references IP-030 watermark_kind.
- Required snapshot_strategy: required for `snapshot`; one of `timestamp`, `check_columns`, `surrogate_key_hash`.
- Required evidence: query expression hash, destination_table_binding, last refresh_run_id, refresh_cost_dimensions.

## Implementation steps
- Add `materialization-policy` as a sub-context of `transform` bounded context.
- Add `src/domain/materialization.rs` with `MaterializationPolicyBinding`, `MaterializationFamily` enum, `SnapshotStrategy` enum, `RefreshPolicy` variant.
- Add `src/usecase/materialization.rs` exposing `materialization.define`, `materialization.amend`, `materialization.promote`, `materialization.refresh`, `materialization.deprecate`, `materialization.read_state`.
- Add `src/adapter/materializer/<family>.rs` per family with a stable `Materializer` trait.
- Add `local-materialization-define-scope.cedar` and `local-materialization-promote-scope.cedar`.
- Add `oya.data.pipeline.materialization.defined`, `.amended`, `.refreshed`, `.refresh_failed`, `.deprecated` to AsyncAPI surface.
- Add `capabilities/materialization-define.yaml` and `capabilities/materialization-refresh.yaml`.
- Add `catalog/oya-data-pipeline-transform-materialization-domain.yaml`.
- Add SLO `local-materialization-refresh-success-rate.openslo.yaml` (0.999 for table, 0.9995 for incremental, 0.999 for snapshot; view/ephemeral N/A).
- Add SLO `local-materialization-refresh-latency.openslo.yaml` with family-specific p95 targets.
- Add runbook `materialization-refresh-failure.md` and `materialization-incremental-cursor-divergence.md`.

## Per-family rules
### view
- No destination_load_run created on define; reads route through destination's query engine.
- Schema changes propagate immediately (next read sees new shape).
- Cost: zero materialization cost; read cost is destination's query cost.
- Refresh: not applicable.
- Failure mode: stale only if underlying source table reverts.
- Pack overlay: residency follows destination's pack overlay.

### table
- destination_load_run created per refresh (IP-031) with full-overwrite or replace-on-commit disposition.
- Schema changes require IP-026 drift disposition before refresh.
- Cost: per-refresh bytes_loaded, rows_loaded, commit_duration_ms.
- Refresh: scheduled via IP-032; manual refresh allowed.
- Failure mode: rollback to prior committed table via destination_commit_cursor.
- Pack overlay: destination cell must be in tenant pack allow-list.

### incremental
- destination_load_run created per refresh with merge/append disposition.
- Schema changes require IP-026 drift disposition.
- Watermark advancement (IP-030) gates the cursor; refresh advances watermark on commit.
- Cost: per-refresh delta bytes_loaded, delta rows_loaded, commit_duration_ms.
- Refresh: cron / interval / event / sensor cadence via IP-032.
- Failure mode: rollback restores watermark and previous incremental cursor.
- Pack overlay: residency same as table; watermark advancement is tenant-cell scoped.

### ephemeral
- No destination_id; expression is inlined as CTE into downstream queries.
- Schema changes propagate by recompilation of downstream queries.
- Cost: zero materialization cost; compile-time only.
- Refresh: not applicable.
- Failure mode: only fails if expression compilation fails.
- Pack overlay: inherits from downstream consumer.

### snapshot
- destination_load_run created per snapshot tick (IP-031) with SCD2 merge disposition.
- Snapshot strategy decides change detection: `timestamp`, `check_columns`, or `surrogate_key_hash`.
- Schema changes require IP-026 drift disposition + snapshot_strategy amendment.
- Cost: per-snapshot delta bytes, delta rows, valid_from/valid_to column writes.
- Refresh: scheduled cadence; manual snapshot allowed.
- Failure mode: rollback creates `rolled_back` snapshot generation with `superseded` valid_to.
- Pack overlay: SCD2 history must remain inside the same cell as the source; cross-cell SCD2 requires pack permit.

## Evidence payload
- `tenant_id` is mandatory.
- `home_cell` is mandatory.
- `transform_id_or_metric_id` is mandatory.
- `materialization_family` is mandatory.
- `materialization_version` is mandatory.
- `destination_id` is mandatory except for ephemeral.
- `refresh_policy` is mandatory except for view and ephemeral.
- `watermark_binding` is mandatory for incremental.
- `snapshot_strategy` is mandatory for snapshot.
- `query_expression_hash` is mandatory.
- `last_refresh_run_id` is mandatory after first refresh.
- `refresh_cost_dimensions` is mandatory after first refresh.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.

## Policy gates
- Cedar denies materialization.define without tenant scope.
- Cedar denies materialization.define if destination_id is in a cell the tenant pack overlay forbids.
- Cedar denies materialization.define if family is `incremental` and watermark_binding kind is not one of {captured, landed, transformed}.
- Cedar denies materialization.define if family is `snapshot` and snapshot_strategy is missing.
- Cedar denies materialization.refresh if IP-026 drift case is open for source.
- Cedar denies materialization.refresh if IP-017 cost-budget is exhausted.
- Cedar denies materialization.refresh if audit-chain is unavailable.
- Cedar denies materialization.promote if previous materialization is in `refresh_failed` state without disposition.
- Cedar denies materialization.deprecate without grace_window_days >= 7.
- Cedar denies materialization.read_state if requestor lacks `tenant_data_consumer` audience.

## Benchmark displacement
- dbt Cloud `materialized:` parity: all five families covered.
- Snowflake Dynamic Tables parity: `incremental` with `LAG` parameter equivalent.
- Databricks Delta Live Tables parity: `incremental` + `snapshot` with streaming source binding.
- Materialize continuous SQL parity: `incremental` with `continuous` IP-032 cadence.
- Iceberg incremental views parity: `incremental` with manifest-commit destination class (IP-031 lakehouse class).
- Vendor names do not become canonical family names.

## Failure handling
- If refresh fails mid-commit, mark materialization as `refresh_failed`, open destination_load_run rollback case (IP-031), and link `runbooks/materialization-refresh-failure.md`.
- If incremental cursor diverges (e.g., source watermark moves backward), open `runbooks/materialization-incremental-cursor-divergence.md` and freeze refresh.
- If snapshot strategy detects no changes for `expected_change_window`, emit a freshness warning rather than failing.
- If ephemeral expression compilation fails, fail downstream query compilation with refusal evidence.
- If Cedar is unavailable, fail closed for define/promote/refresh; read_state may serve cached state.
- If audit-chain is unavailable, hold define/promote/refresh.

## Tests and evidence
- Unit test: MaterializationFamily enum exhaustive in switch.
- Unit test: snapshot_strategy validator rejects unknown strategies.
- Contract test: materialization.define rejects incremental without watermark_binding.
- Contract test: materialization.define rejects snapshot without snapshot_strategy.
- Contract test: materialization.define rejects ephemeral with destination_id.
- Policy test: cross-cell destination denied without pack permit.
- Policy test: drift-case-open blocks refresh.
- Replay test: rollback restores destination_commit_cursor.
- Replay test: incremental rollback restores watermark to prior value.
- SLO test: local-materialization-refresh-success-rate burn opens runbook.
- Audit test: define, refresh, deprecate share correlation id.

## Rollback
- Roll back materialization by amending policy (append-only).
- If destination_load_run exists for this materialization, rollback via IP-031 destination rollback.
- Recompute IP-030 watermark for incremental rollback.
- Notify IP-034 exposures consuming this materialization.
- Notify IP-033 semantic metrics referencing this materialization.
- Link rollback to `runbooks/materialization-refresh-failure.md`.

## Acceptance criteria
- All five materialization families covered with domain rules, Cedar policies, destination_load_run binding, and cost dimensions.
- `local-materialization-define-scope.cedar` and `local-materialization-promote-scope.cedar` exist.
- SLO and runbook artifacts exist.
- IP-030 watermark binding wired for incremental.
- IP-031 destination_load_run wired for table / incremental / snapshot.
- IP-033 semantic metrics can reference a materialization_policy_binding.
- IP-034 exposures pick up impact on materialization rollback.

## Citation map
- `microservices/data-pipeline/coherence-audit-2026-05-20.md` §3.9.2.
- `microservices/data-pipeline/feature-parity-matrix-2026-05-20.md` materialization families row.
- `microservices/data-pipeline/IP-030-cdc-freshness-watermark-governance.md` incremental cursor.
- `microservices/data-pipeline/IP-031-destination-connector.md` destination commit semantics.
- `microservices/data-pipeline/IP-033-semantic-layer.md` metric materialization_policy.
- `microservices/data-pipeline/IP-034-exposure-tracking.md` exposure impact.
- `ADR-0245` substrate vs product layering.
- `ADR-0251` pack overlay.
- `ADR-0321` documentation rigor.

## Operator review prompts
- Reviewer asks whether family fits query semantics (view vs table vs incremental vs ephemeral vs snapshot).
- Reviewer asks whether destination_id sits in the correct cell.
- Reviewer asks whether refresh_policy matches downstream freshness expectations.
- Reviewer asks whether incremental watermark_binding is correct kind.
- Reviewer asks whether snapshot_strategy fits source change detection model.
- Reviewer asks whether refresh budget is sufficient given cadence.
- Reviewer asks whether pack overlay permits the destination cell.
- Reviewer signs the materialization case with the same audit correlation id.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-035-materialization-families.md:54` - - Add SLO `local-materialization-refresh-success-rate.openslo.yaml` (0.999 for table, 0.9995 for incremental, 0.999 for snapshot; view/ephemeral N/A).; `microservices/data-pipeline/IP-035-materialization-families.md:55` - - Add SLO `local-materialization-refresh-latency.openslo.yaml` with family-specific p95 targets..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-035-materialization-families.md:12` - - Name the five canonical materialization families and bind each one to a domain rule, a Cedar policy, a destination_load_run shape (IP-031), and a cost dimension (IP-...; `microservices/data-pipeline/IP-035-materialization-families.md:28` - - Read `microservices/data-pipeline/IP-029-transform-cost-attribution.md` for cost dimensions..
