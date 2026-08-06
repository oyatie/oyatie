---
id: ADR-SHE-001
title: Spreadsheet Formula Engine with Incremental Recalculation
status: Proposed
date: 2026-05-20
microservice: sheets
related_oyatie_adrs:
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0706-observability-live-apex.md
decision_owner: axis-sheets
---

# ADR-SHE-001: Spreadsheet Formula Engine with Incremental Recalculation

## Context

- Sheets owns cell grid storage, formulas, charts, import/export, collaboration, ACLs, and large-sheet storage.
- Existing ADR-SHEETS-0002 defines formula conformance targets.
- Existing ADR-SHEETS-0004 defines dependency-graph, topological, and parallel-task-graph recalc architecture.
- This ADR binds the formula engine to named incremental recalc semantics and public engine contracts.
- Named pressure SHE-P1: spreadsheet users expect a single edited cell to update dependent values without full workbook recalculation.
- Named pressure SHE-P2: import from Excel and Google Sheets must preserve formula meaning where supported.
- Named pressure SHE-P3: volatile functions such as NOW and RAND must not poison deterministic audit replay.
- Named pressure SHE-P4: collaborative edits can mutate formulas while another client reads derived cells.
- Named pressure SHE-P5: very large workbooks need p95 budgets for 100k and 1m cell recalculation.
- Named precedent: Excel uses dependency trees, dirty marking, and chain recalculation.
- Named precedent: Google Sheets uses server-side collaborative formula execution with incremental dependency tracking.
- Named precedent: HyperFormula uses a graph-backed formula engine with incremental updates.
- Constraint SHE-C1: tenant scope and workbook authority come from ADR-0244.
- Constraint SHE-C2: formula edits, recalc plans, formula errors, and replay divergences emit evidence per ADR-0263.
- Constraint SHE-C3: Cedar gates workbook read, formula write, range recalc, and external data functions per ADR-0243.
- Constraint SHE-C4: formula API versions must be additive under ADR-0258.
- Constraint SHE-C5: formula evaluation must be deterministic for non-volatile functions.
- Constraint SHE-C6: volatile functions must carry explicit evaluation epoch.
- Constraint SHE-C7: dependency graph cycles produce typed formula errors, never scheduler loops.
- Constraint SHE-C8: external data functions are disabled unless a tenant pack and Cedar permit allow them.
- Constraint SHE-C9: formula result cache must be invalidated by dependency edges, not by full workbook clears.
- Constraint SHE-C10: formula import gaps must surface as explicit unsupported-function errors.
- Full recalc is operationally useful for validation and recovery.
- Full recalc is not acceptable on the common edit path.
- Named DAG traversal must be visible in telemetry and test names.
- This ADR complements, not replaces, ADR-SHEETS-0004.

## Decision

- Adopt an incremental recalc formula engine backed by a named dependency DAG.
- Name the DAG traversal `SheetDepDag v1`.
- Store dependency edges from precedent cell or range to dependent formula cell.
- Mark edited cells and their transitive dependents dirty.
- Traverse dirty subgraphs in topological levels using Kahn traversal.
- Evaluate each dirty formula once per recalc epoch.
- Evaluate independent nodes within the same topological level in parallel.
- Keep full recalculation as a maintenance and validation mode only.
- Compile formulas to a typed expression AST before persistence.
- Store both source formula text and compiled AST hash.
- Store formula locale separately from formula semantics.
- Use invariant function ids internally, not localized function display names.
- Represent ranges as first-class graph nodes when they are shared by many formulas.
- Use range compaction to avoid one edge per cell for common ranges.
- Use named ranges as stable dependency nodes.
- Use external data functions as guarded plugin nodes.
- Use volatile functions as explicit epoch dependencies.
- Use cycle detection on every edge update.
- Represent cycle errors as `#CYCLE!`.
- Represent slow formula budget failure as `#SLOW!`.
- Represent unsupported imported formula as `#UNSUPPORTED!`.
- Represent external function deny as `#PERMIT!`.
- Persist formula result values with `recalc_epoch`.
- Persist formula errors with typed error code and source span.
- Publish recalc deltas to clients over WebSocket.
- Keep formulas pure by default.
- Forbid formulas from mutating cells or calling arbitrary network resources.
- Route allowed external data calls through a Cedar-gated connector surface.
- Keep deterministic replay corpus for every supported function.
- Keep import compatibility corpus for Excel, Google Sheets, and LibreOffice.
- Make `SheetDepDag v1` part of the contract for dashboards and runbooks.

## Alternatives Considered

### Full Workbook Recalc on Every Edit

- Pros: simple invalidation model.
- Pros: easy to reason about correctness.
- Pros: avoids stale dependency graph bugs.
- Cons: cannot meet large workbook latency budgets.
- Cons: wastes CPU for isolated edits.
- Cons: collaboration creates needless recalculation storms.
- Rejected because it fails 100k and 1m cell budgets.

### Cell-Local Formula Evaluation Only

- Pros: fastest for formulas with no dependencies.
- Pros: minimal graph storage.
- Pros: easy first version.
- Cons: dependent cells remain stale.
- Cons: charts and named ranges cannot trust derived values.
- Cons: users expect spreadsheet dependency propagation.
- Rejected because it is not a real spreadsheet formula engine.

### Actor Per Cell

- Pros: naturally reactive.
- Pros: isolates slow formulas by actor.
- Pros: maps conceptually to dependency propagation.
- Cons: actor overhead is high for 1m cell workbooks.
- Cons: deterministic ordering is harder.
- Cons: cycle detection needs a global view anyway.
- Rejected because per-cell actor overhead exceeds the value.

### External Formula Engine as Black Box

- Pros: faster to adopt if a library has strong compatibility.
- Pros: external library may already implement many functions.
- Pros: import parity improves quickly.
- Cons: tenant policy, audit, and replay hooks may be missing.
- Cons: graph telemetry can become opaque.
- Cons: external data functions may bypass Cedar.
- Rejected as the sole architecture; libraries may be used behind Oyatie ports.

### Incremental Recalc over SheetDepDag

- Pros: recalculates only affected cells.
- Pros: exposes graph health and dirty-set size as metrics.
- Pros: composes with parallel topological execution.
- Cons: graph correctness becomes load-bearing.
- Cons: range compaction needs careful invalidation.
- Cons: volatile functions need special epochs.
- Accepted because it matches spreadsheet expectations and scale budgets.

## Consequences

- Positive: ordinary edits avoid full workbook recalculation.
- Positive: large workbook SLOs are tied to measurable dirty-set size.
- Positive: deterministic formula replay can diagnose audit disputes.
- Positive: formula import failures are typed and visible.
- Positive: external function access is policy-bound.
- Positive: charts and conditional formatting consume fresh dependency results.
- Positive: parallel evaluation remains deterministic by topological level.
- Positive: recalc storms can be throttled by dirty-set size and work class.
- Negative: dependency graph corruption can cause stale values if not detected.
- Negative: range compaction makes debugging edges more complex.
- Negative: volatile functions require epoch controls that users may not expect.
- Negative: formula compatibility surface is large and long-lived.
- Negative: external data functions require connector governance.
- Neutral: full recalc remains available for recovery.
- Neutral: unsupported functions are valid imported state until migrated.
- Neutral: named ranges become graph nodes, not just UI labels.
- Neutral: recalc workers stay service-local to sheets.
- Neutral: formula engine ports can wrap a third-party library where it passes conformance.

## Implementation Notes

- Data shape `FormulaCell`: `{tenant_id, workbook_id, sheet_id, cell_ref, formula_text, ast_hash, locale, result_ref, recalc_epoch}`.
- Data shape `FormulaAst`: `{ast_hash, engine_version, function_ids, references, volatile_flags, external_flags, serialized_ast}`.
- Data shape `SheetDepNode`: `{tenant_id, workbook_id, node_id, node_kind, ref, shard_id}`.
- Data shape `SheetDepEdge`: `{tenant_id, workbook_id, from_node_id, to_node_id, edge_kind, created_by_ast_hash}`.
- Data shape `RecalcPlan`: `{tenant_id, workbook_id, plan_id, dirty_roots, levels, graph_version, created_at}`.
- Data shape `FormulaResult`: `{cell_ref, value_kind, value_json, error_code, error_span, recalc_epoch}`.
- Data shape `VolatileEpoch`: `{tenant_id, workbook_id, epoch_id, function_family, evaluation_time, reason}`.
- Data shape `ExternalFunctionCall`: `{tenant_id, workbook_id, cell_ref, connector_id, request_hash, permit_id}`.
- Postgres table `sheet_formula_cell` stores formula text and AST hash.
- Postgres table `sheet_dep_edge` stores compacted dependency edges.
- Valkey hot cache stores active workbook DAG shards.
- Cold large-sheet storage stores formula cells beside Arrow or Parquet blocks.
- Named ranges compile to `SheetDepNode` entries with kind `named_range`.
- Shared range references compile to `SheetDepNode` entries with kind `range`.
- REST endpoint `PUT /v1/sheets/workbooks/{workbook_id}/cells/{cell_ref}/formula` writes formula source.
- REST endpoint `POST /v1/sheets/workbooks/{workbook_id}/recalc` starts guarded recalculation.
- REST endpoint `GET /v1/sheets/workbooks/{workbook_id}/formula-errors` lists typed formula errors.
- REST endpoint `GET /v1/sheets/workbooks/{workbook_id}/dep-graph/{cell_ref}` returns debug graph slices.
- REST endpoint `POST /v1/sheets/workbooks/{workbook_id}/full-recalc` is maintenance-only and Cedar-gated.
- WebSocket message `sheets.recalc.delta.v1` streams changed formula results.
- AsyncAPI channel `sheets.formula.changed.v1` publishes accepted formula changes.
- AsyncAPI channel `sheets.recalc.plan.created.v1` publishes plan shape and dirty-set size.
- AsyncAPI channel `sheets.recalc.completed.v1` publishes completion and error counts.
- AsyncAPI channel `sheets.formula.external.denied.v1` publishes Cedar-denied external calls.
- Cedar permit `sheets::formula::write` requires workbook edit permission.
- Cedar permit `sheets::formula::full_recalc` requires maintainer or tenant admin role.
- Cedar permit `sheets::formula::external_call` requires connector grant and pack allowance.
- Cedar forbid `sheets::formula::external_call` when `resource.data_class in ["PHI", "PCI"]` unless pack grant exists.
- Cedar forbid `sheets::formula::write` when formula contains unsupported network function.
- Audit event `EVT-SHEETS-FORMULA-WRITTEN` includes formula hash and referenced ranges.
- Audit event `EVT-SHEETS-RECALC-PLAN-CREATED` includes dirty root count and level count.
- Audit event `EVT-SHEETS-RECALC-DIVERGENCE` includes expected hash and actual hash.
- Audit event `EVT-SHEETS-EXTERNAL-FUNCTION-DENIED` includes connector id and Cedar policy id.
- Metric `sheets_formula_parse_latency_ms` tracks compile latency.
- Metric `sheets_recalc_dirty_cell_count` tracks plan size.
- Metric `sheets_recalc_level_count` tracks DAG depth.
- Metric `sheets_recalc_duration_ms` tracks plan execution time.
- Metric `sheets_formula_error_total` tracks error code counts.
- Metric `sheets_formula_external_denied_total` tracks policy denials.
- Trace span `sheets.formula.compile` records parser version and AST hash.
- Trace span `sheets.recalc.plan` records dirty roots, edge count, and graph version.
- Trace span `sheets.recalc.evaluate_level` records level index and parallel width.
- Log schema `SheetsRecalcDecisionLog` includes workbook hash, graph version, dirty count, and fallback mode.
- SLO target: single-cell dependent recalc p99 <= 150 ms for dirty sets below 1,000 cells.
- SLO target: 100k cell recalc p95 <= 1 second.
- SLO target: 1m cell recalc p95 <= 10 seconds.
- SLO target: formula parse p99 <= 20 ms for supported functions.
- SLO target: formula replay divergence count equals zero.
- Capacity math: a 100k cell workbook with 5 percent formula cells and average fanout 3 has about 15k dependency edges before range compaction.
- Capacity math: dirtying 1,000 cells with p95 formula evaluation 100 microseconds gives 100 ms raw compute before scheduler and IO overhead.
- Capacity math: 1m cells with 10 percent formulas and average fanout 5 yields 500k edges; shard DAG by sheet and range node to keep worker memory below 512 MiB.
- Rollback path: disable incremental cache and run full recalc maintenance mode for affected workbooks.
- Rollback path: rebuild `sheet_dep_edge` from persisted formula AST references.
- Rollback path: pin previous formula engine version for workbooks with import regressions.
- Multi-region path: formula writes occur in home cell; remote cells consume result snapshots.
- Sovereign-cell path: external data functions execute only in approved pack regions.
- Versioning: function ids are stable and additive within formula engine v1.
- Deprecation: formula functions require 365-day read and recalc support after write deprecation.

## Verification

- Unit test `formula_write_builds_ast_and_edges` checks compile and graph update.
- Unit test `single_cell_edit_marks_transitive_dependents_dirty` checks invalidation.
- Unit test `cycle_returns_cycle_error_not_loop` checks cycle behavior.
- Unit test `volatile_now_uses_epoch_dependency` checks deterministic epoch model.
- Unit test `external_function_requires_cedar_permit` checks policy binding.
- Property test `incremental_recalc_matches_full_recalc` generates random DAG workbooks.
- Property test `range_compaction_preserves_dirty_set` checks range invalidation.
- Property test `topological_levels_have_no_internal_edges` checks traversal.
- Fuzz test `formula_parser_rejects_malformed_imports` covers hostile XLSX imports.
- Integration test `collab_formula_edits_converge_to_same_results` checks concurrent edits.
- Integration test `unsupported_excel_function_imports_as_typed_error` checks import parity.
- Integration test `chart_updates_after_formula_delta` checks downstream chart path.
- Integration test `named_range_rename_invalidates_dependents` checks named nodes.
- Load test `recalc_100k_cells_under_one_second` validates service SLO.
- Load test `recalc_1m_cells_under_ten_seconds` validates large workbook SLO.
- Chaos test `dag_cache_loss_rebuilds_from_formula_rows` checks hot-cache recovery.
- Chaos test `worker_crash_mid_recalc_replays_plan_idempotently` checks plan recovery.
- Metric check: dashboard `sheets/recalc-engine-health` shows dirty cells, levels, and duration.
- Metric check: dashboard `sheets/editor-experience` shows formula parse latency.
- Alert check: `sheets_recalc_divergence_total` above zero pages immediately.
- Audit check: every full recalc has a Cedar decision and audit event.
- Static check: formula engine has no arbitrary network imports.
- Contract check: OpenAPI marks full recalc as maintenance and internal.
- Regression check: ADR-SHEETS-0004 remains consistent with `SheetDepDag v1`.

## References

- Microsoft Excel recalculation documentation.
- Google Sheets function and recalculation documentation.
- HyperFormula dependency graph documentation.
- Kahn topological sorting algorithm.
- LibreOffice Calc formula function reference.
- OpenDocument Formula specification.
- ECMA-376 Office Open XML formula references.
- Cedar policy language documentation.
- ADR-SHEETS-0002 formula-engine conformance target.
- ADR-SHEETS-0004 recalc-engine architecture.
- ADR-0243 Cedar-as-universal-gate.
- ADR-0263 observability-emission-contract.
- microservices/sheets/PRD.md.
- microservices/sheets/capacity-model.md.
- microservices/sheets/runbooks/recalc-storm-throttle.md.
