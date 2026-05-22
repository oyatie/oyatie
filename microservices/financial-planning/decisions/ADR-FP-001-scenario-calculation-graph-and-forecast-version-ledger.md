---
id: ADR-FP-001
title: Scenario calculation graph and forecast version ledger
status: Proposed
date: 2026-05-20
microservice: financial-planning
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0105-thirteen-layer-canonical-enum
  - ADR-0131-per-microservice-flat-layout
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0316-tenant-class-activation-over-product-fragmentation
  - ADR-0263-observability-emission-contract
decision_owner: axis-financial-planning
---

# ADR-FP-001: Scenario calculation graph and forecast version ledger

## Context

- Architectural pressure name: finance planning determinism pressure.
- Financial Planning owns forecast, scenario, consolidation, variance, and board-report evidence beyond generic sheets.
- The PRD benchmark set includes Anaplan, Workday Adaptive Planning, OneStream, Vena, and Pigment.
- The service has bounded contexts for forecast-model, budget-cycle, variance, scenario, and consolidation.
- Existing policies include budget lock control, forecast version scope, FX rate backfill guard, close cycle advance, variance explanation approval, and board report seal egress.
- Existing capabilities include forecast-version-open, scenario-recalculate, variance-explain, consolidation-close, driver-model-import, and board-report-seal.
- Existing SLO and dashboard surfaces cover local domain throughput, policy decisions, audit completeness, SLO burn, and tenant cost and capacity.
- Constraint FP-C1: forecasts must be reproducible from inputs, formulas, driver versions, FX rates, and scenario assumptions.
- Constraint FP-C2: finance pack requirements include SOC-2, ISO-27001, SOX-404, GDPR, KR-FSS, and PCI-DSS-L1-v4.
- Constraint FP-C3: budget locks must prevent mutation after approved close windows.
- Constraint FP-C4: scenario recalculation must not mutate approved forecast versions.
- Constraint FP-C5: variance explanations need approval before board-report sealing.
- Constraint FP-C6: FX rate backfill must be bounded and audit visible.
- Constraint FP-C7: consolidation close must preserve intercompany elimination evidence.
- Constraint FP-C8: board reports must reference the exact forecast version and scenario digest.
- Constraint FP-C9: Cedar must authorize all forecast, scenario, close, and report actions.
- Constraint FP-C10: calculation metrics must avoid raw tenant, account, and cost-center labels.
- The service must not recreate spreadsheet free-form ambiguity as a domain model.
- The service must support import from incumbent planning tools while normalizing to Oyatie-owned dimensions.
- The service must handle many scenarios without allowing uncontrolled cost explosions.
- The service must make calculation cache invalidation explainable.
- The service must allow board audiences to inspect source evidence without editing finance state.
- The service must let consolidation workflows pause on data, policy, or FX conflict.
- The service must separate read-only scenario analysis from approved forecast versions.
- The service must support replay for audit and SOX evidence.

## Decision

- Decision name: ForecastScenarioGraph v1.
- Adopt a versioned scenario calculation graph plus immutable forecast version ledger.
- Treat `ForecastVersion` as the immutable approved finance state for a planning cycle.
- Treat `ScenarioGraph` as a DAG of driver assumptions, formulas, dimensions, and derived measures.
- Treat `ScenarioRun` as a replayable calculation attempt over one graph version and input snapshot.
- Treat `BudgetLock` as the authority for mutation eligibility by period, role, and pack.
- Treat `VarianceExplanation` as governed evidence that can be drafted, approved, rejected, or superseded.
- Treat `BoardReportSeal` as a signed export over an approved forecast version and approved explanations.
- Store formula definitions, driver models, and dimension maps as versioned metadata rows.
- Store calculation inputs and outputs partitioned by tenant, cell, planning cycle, scenario id, and data class.
- Store large output matrices as object references with content digests.
- Require every scenario run to carry graph digest, input digest, formula version, FX rate table version, policy decision id, and trace id.
- Require scenario graph cycles to be rejected at write time.
- Require recalculation fan-out to cap at 25,000 nodes per interactive run.
- Require larger recalculations to execute as async jobs with progress projections.
- Require interactive scenario recalculation p95 below 2 seconds for graphs up to 25,000 nodes.
- Require async scenario recalculation p95 completion below 15 minutes for graphs up to 10 million calculation cells.
- Require budget lock policy evaluation p95 below 100 ms.
- Require variance explanation freshness p95 below 5 minutes after actuals import.
- Require board report seal generation p95 below 60 seconds for standard packets.
- Require FX backfill to open a new input snapshot and never rewrite an approved forecast version.
- Require consolidation close to create explicit elimination entries and close evidence.
- Require SOX-404 pack to enforce segregation between preparer, approver, and board-report sealer.
- Require board report egress to include redaction manifest and exact digest list.
- Publish `financial.forecast.version_opened.v1`, `financial.scenario.recalculated.v1`, `financial.budget.locked.v1`, `financial.variance.explanation_approved.v1`, `financial.consolidation.closed.v1`, and `financial.board_report.sealed.v1`.
- Use analytics as a read-side consumer, not the authority for finance calculations.
- Use workflow-engine for approvals, close checklists, and review tasks.
- Use ontology projection for ForecastVersion, Scenario, DriverModel, BudgetCycle, VarianceExplanation, and BoardReport.
- Make this ADR authoritative for scenario calculation, forecast immutability, budget locks, variance approvals, consolidation close, and board report seals.

## Alternatives Considered

### Alternative 1: Store planning state as spreadsheet-like cells only

- Pros: familiar for finance users.
- Pros: easy first import from sheets.
- Pros: quick prototyping.
- Cons: formulas and dependencies become opaque.
- Cons: scenario replay is fragile.
- Cons: budget locks and SOX controls are hard to enforce.
- Rejected because finance evidence needs graph-level determinism.

### Alternative 2: Use analytics service as calculation authority

- Pros: OLAP engines are fast for aggregation.
- Pros: avoids a separate calculation engine.
- Pros: easier reporting integration.
- Cons: analytics is a read-side substrate.
- Cons: finance domain rules would leak into analytics.
- Cons: approvals and budget locks need write-side invariants.
- Rejected because financial-planning owns planning domain state.

### Alternative 3: Mutable forecast versions with audit columns

- Pros: smaller storage footprint.
- Pros: simpler update API.
- Pros: familiar database design.
- Cons: approved forecasts can drift.
- Cons: board-report seals become untrustworthy.
- Cons: replay requires reconstructing old rows from logs.
- Rejected because approved forecast state must be immutable.

### Alternative 4: Recalculate every scenario synchronously

- Pros: simpler caller contract.
- Pros: immediate result or failure.
- Pros: fewer job state transitions.
- Cons: large scenarios exceed HTTP and UI budgets.
- Cons: cost control is weak.
- Cons: retry and progress are opaque.
- Rejected because large planning models need async bounded execution.

### Alternative 5: Allow FX backfill to rewrite old values in place

- Pros: easiest correction path.
- Pros: fewer scenario reruns.
- Pros: small UI change.
- Cons: historical reports become unstable.
- Cons: audit trails cannot prove what changed.
- Cons: SOX evidence is weakened.
- Rejected because FX backfill must create new snapshots.

## Consequences

### Positive

- Scenario runs become replayable from digest-bound inputs.
- Approved forecast versions cannot drift under later edits.
- Board report seals have exact data provenance.
- Budget locks and close cycles are enforceable by policy.
- Variance explanations can be audited before external sharing.
- Large scenario jobs can expose progress and cost.
- FX backfills are visible and reversible through new snapshots.
- Consolidation evidence stays tied to elimination entries.
- Finance users can compare scenarios without mutating approved state.
- Imports from incumbent tools normalize into owned dimensions.

### Negative

- The calculation graph requires formula schema governance.
- Async scenario jobs add worker and queue complexity.
- Large matrix output storage needs cost budgeting.
- Formula compatibility tests are required for every schema version.
- Users need clear distinction between scenario draft and approved forecast.
- Board report export must handle digest and redaction manifests.
- SOX segregation rules increase approval workflow complexity.

### Neutral

- Sheets can still serve as an import and collaboration surface.
- Analytics can still consume projections for dashboards.
- Workflow-engine still owns approval tasks.
- Payments and FinOps can remain event consumers.
- External planning tools can remain migration sources.

### Follow-up work

- Add `ScenarioGraph` schema and cycle rejection fixtures.
- Add deterministic formula evaluator conformance suite.
- Add async recalculation progress contract.
- Add FX backfill snapshot test corpus.
- Add board report seal export manifest.
- Add SOX segregation Cedar policy fixtures.
- Add scenario cost estimation dashboard.

## Implementation Notes

### Data Shapes

- `ForecastVersion`: `forecast_version_id`, `tenant_id_hash`, `planning_cycle_id`, `status`, `graph_version`, `input_snapshot_id`, `approved_at`, `ledger_root`, `audit_event_id`.
- `ScenarioGraph`: `scenario_graph_id`, `tenant_id_hash`, `version`, `node_count`, `edge_count`, `formula_digest`, `dimension_digest`, `created_by`, `state`.
- `ScenarioNode`: `node_id`, `scenario_graph_id`, `node_type`, `dimension_ref`, `formula_ref`, `input_ref`, `output_ref`, `data_class`.
- `ScenarioRun`: `run_id`, `scenario_graph_id`, `input_snapshot_id`, `fx_rate_table_version`, `status`, `started_at`, `completed_at`, `output_digest`, `cost_estimate`.
- `BudgetLock`: `budget_lock_id`, `planning_cycle_id`, `scope_path`, `locked_after`, `allowed_roles`, `pack_code`, `policy_decision_id`.
- `VarianceExplanation`: `variance_id`, `forecast_version_id`, `actuals_snapshot_id`, `variance_amount`, `explanation_ref`, `approval_state`, `approver`.
- `ConsolidationClose`: `close_id`, `forecast_version_id`, `entity_scope`, `elimination_entries_ref`, `state`, `evidence_id`.
- `BoardReportSeal`: `seal_id`, `forecast_version_id`, `scenario_run_id`, `approved_explanation_digest`, `redaction_manifest_id`, `seal_digest`.

### API Endpoints

- `POST /v1/financial-planning/forecast-versions` opens a forecast version.
- `GET /v1/financial-planning/forecast-versions/{forecast_version_id}` reads immutable forecast metadata.
- `POST /v1/financial-planning/scenario-graphs` creates or amends a scenario graph draft.
- `POST /v1/financial-planning/scenario-runs` starts calculation.
- `GET /v1/financial-planning/scenario-runs/{run_id}` returns progress and result digest.
- `POST /v1/financial-planning/budget-locks` creates a budget lock.
- `POST /v1/financial-planning/fx-rates/backfill` creates a new input snapshot.
- `POST /v1/financial-planning/variance-explanations/{variance_id}/approve` approves explanation.
- `POST /v1/financial-planning/consolidation-closes` advances close workflow.
- `POST /v1/financial-planning/board-reports/seal` creates governed report seal.

### Cedar Policies

- `financial::forecast::open` requires finance owner or approved planning workflow principal.
- `financial::forecast::approve` requires approver distinct from preparer under SOX pack.
- `financial::scenario::write` requires model owner and unlocked planning cycle.
- `financial::scenario::run` requires cost budget allowance and scenario scope.
- `financial::budget_lock::mutate` requires budget administrator and audit-chain availability.
- `financial::fx::backfill` requires finance admin and backfill reason.
- `financial::variance::approve` requires authorized reviewer and no self-approval when pack demands segregation.
- `financial::close::advance` requires close-cycle role and evidence completeness.
- `financial::board_report::seal` requires approved forecast, approved explanations, and redaction manifest.

### SLO Targets

- `financial_scenario_interactive_recalc_p95_seconds` target is 2.
- `financial_scenario_async_recalc_p95_minutes` target is 15 for 10 million cells.
- `financial_budget_lock_policy_p95_ms` target is 100.
- `financial_variance_explanation_freshness_p95_minutes` target is 5.
- `financial_board_report_seal_p95_seconds` target is 60.
- `financial_audit_emission_lag_p95_seconds` target is 1.
- `financial_scenario_cost_estimate_accuracy` target is within 15 percent.
- `financial_forecast_version_immutability` target is 1.0.

## Verification

- Unit test `scenario_graph_rejects_cycles`.
- Unit test `forecast_version_immutable_after_approval`.
- Unit test `budget_lock_blocks_mutation_after_locked_after`.
- Unit test `fx_backfill_creates_new_input_snapshot`.
- Unit test `board_report_seal_requires_approved_forecast`.
- Unit test `variance_explanation_requires_approval_before_seal`.
- Property test `same_graph_and_input_produce_same_output_digest`.
- Property test `formula_topological_sort_is_stable`.
- Property test `cost_estimate_within_expected_bound_for_fixture`.
- Fuzz test `formula_parser_rejects_unsafe_functions`.
- Cedar test `scenario_run_denies_missing_cost_budget`.
- Cedar test `sox_pack_rejects_preparer_self_approval`.
- Cedar test `board_report_denies_without_redaction_manifest`.
- Cedar test `fx_backfill_requires_reason`.
- Contract test `financial_openapi_scenario_paths_match_router`.
- Contract test `financial_asyncapi_events_include_graph_digest`.
- Contract test `financial_proto_scenario_run_matches_rest_shape`.
- Integration test `forecast_open_scenario_run_approve_board_report`.
- Integration test `budget_lock_blocks_late_driver_edit`.
- Integration test `consolidation_close_emits_elimination_evidence`.
- Integration test `variance_explanation_approval_unblocks_report_seal`.
- Replay test `forecast_ledger_rebuilds_approved_version`.
- Load test `twenty_five_thousand_node_interactive_recalc_under_2s`.
- Load test `ten_million_cell_async_recalc_under_15m`.
- Chaos test `worker_crash_restarts_scenario_run_from_checkpoint`.
- Chaos test `audit_chain_unavailable_pauses_board_report_seal`.
- Metric `oya_financial_scenario_run_duration_ms`.
- Metric `oya_financial_scenario_node_count`.
- Metric `oya_financial_budget_lock_denial_total`.
- Metric `oya_financial_variance_explanation_state_total`.
- Metric `oya_financial_board_report_seal_total`.
- Dashboard `financial-local-domain-throughput`.
- Dashboard `financial-local-policy-decisions`.
- Dashboard `financial-slo-burn`.
- Dashboard `financial-tenant-cost-and-capacity`.
- Alert `FinancialScenarioRecalcLatencyBurn`.
- Alert `FinancialBudgetLockDenySpike`.
- Alert `FinancialBoardReportSealStalled`.

## References

- Internal: microservices/financial-planning/PRD.md.
- Internal: microservices/financial-planning/ARCHITECTURE.md.
- Internal: microservices/financial-planning/policies/local-budget-lock-control.cedar.
- Internal: microservices/financial-planning/policies/local-forecast-version-scope.cedar.
- Internal: microservices/financial-planning/policies/local-fx-rate-backfill-guard.cedar.
- Internal: microservices/financial-planning/policies/local-board-report-seal-egress.cedar.
- Internal: microservices/financial-planning/capabilities/scenario-recalculate.yaml.
- Internal: microservices/financial-planning/capabilities/board-report-seal.yaml.
- Internal: microservices/financial-planning/IP-028-oracle-epm-onestream-close-displacement.md.
- Internal: microservices/financial-planning/IP-030-planful-driver-import-displacement.md.
- Anaplan Calculation Engine and model-building documentation.
- Workday Adaptive Planning documentation.
- OneStream platform documentation.
- Pigment planning platform documentation.
- Microsoft Excel formula language documentation.
- OpenFormula specification.
- FASB ASC 830 foreign currency matters.
- COSO Internal Control Integrated Framework.
- PCAOB AS 2201 internal control over financial reporting.
- OpenAPI Specification.
- AsyncAPI Specification.
- CloudEvents Specification.
- W3C Trace Context.
- RFC 9110: HTTP Semantics.
