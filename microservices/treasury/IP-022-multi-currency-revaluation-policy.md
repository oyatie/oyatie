---
doc_class: ImplementationPlan
ip_id: IP-022
microservice: treasury
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0319]
journey_id: j120-tenant-treasury-multi-currency-fx-hedge
journey_link: docs/user-journeys/j120-tenant-treasury-multi-currency-fx-hedge/story.md
status: Accepted
date: 2026-05-20
owner: axis-treasury
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [TRM-RM Market Risk Analyzer, TRM-TM Valuation, TRM-CM Cash Management Currency Position]
---

# IP-022: Multi-currency revaluation policy

## Intent
Implement tenant-governed multi-currency revaluation policy for treasury cash, debt, receivable, payable, and hedge exposure views.
The feature defines which balances are revalued, which rate source is used, which accounting basis applies, and which exceptions block close.
The feature displaces SAP TRM-TM valuation customizing and selected TRM-RM currency-risk valuation reports for treasury-owned evidence.
The feature does not post accounting journals; it emits revaluation evidence and handoff payloads for finance-ledger.
The implementation must store policy versions separately from valuation runs.
The implementation must make every revaluation result explainable by source position, rate id, policy line, and calculation formula.
The implementation must support daily, month-end, quarter-end, and ad hoc revaluation cycles.
The implementation must block approval when rates are stale, source positions are stale, or policy coverage is incomplete.
The implementation must emit ADR-0263 audit events for policy changes, run results, approvals, and denies.
The implementation must remain cell-local and tenant-scoped.

## Context
Why: multi-currency tenants need consistent revaluation rules across treasury, accounting, risk, and close dashboards.
Why: SAP valuation customizing is powerful but opaque to product teams and hard to verify from source evidence.
Why: Oyatie needs explicit policy rows so interns can implement rate selection, valuation, exception handling, and ledger handoff without inventing accounting rules.
Journey leg: j120 month-end treasury close revalues EUR cash, JPY debt, KRW payables, and USD hedges before hedge review.
Named persona: Helena Park, Treasury Accounting Manager at AsterFoods Global, owns treasury revaluation review.
Supporting persona: Mateo Silva, Corporate Controller, approves material FX variance handoffs to finance-ledger.
Pain point: teams disagree about whether spot, average, or official close rates were used.
Pain point: stale rates produce silent differences between treasury and accounting reports.
Pain point: hedge and non-hedge positions are revalued using different policy basis but mixed in exports.
SAP parity: TRM-TM valuation areas, TRM-RM market risk analyzer valuation, and TRM-CM currency cash position.
Product outcome: one policy-backed revaluation run shows source amount, carrying amount, revalued amount, FX gain/loss, rate source, and approval.
Non-goal: statutory accounting posting remains in finance-ledger.
Non-goal: hedge effectiveness testing remains in hedge-designation services.
Non-goal: rate vendor ingestion remains in rates.
Invariant: every source position must match exactly one active policy line or become a blocking exception.
Invariant: one revaluation run references one immutable rate set.
Invariant: approved revaluation runs cannot be edited; corrections create superseding runs.
Invariant: material variance thresholds are policy lines, not hardcoded constants.
Acceptance anchor: an intern can implement migrations, rate selection, valuation, approval, ledger handoff, and tests from this file.

## Data Model Deltas
Table `treasury.revaluation_policy`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `policy_code TEXT NOT NULL`.
Column `name TEXT NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Draft','Active','Retired'))`.
Column `base_currency CHAR(3) NOT NULL`.
Column `effective_from DATE NOT NULL`.
Column `effective_to DATE`.
Column `created_by_principal_id UUID NOT NULL`.
Column `approved_by_principal_id UUID`.
Column `approved_at TIMESTAMPTZ`.
Column `cedar_decision_id UUID`.
Constraint `UNIQUE (tenant_id, policy_code, effective_from)`.
Table `treasury.revaluation_policy_line`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `policy_id UUID NOT NULL REFERENCES treasury.revaluation_policy(id)`.
Column `position_source_type TEXT NOT NULL CHECK (position_source_type IN ('Cash','Debt','Payable','Receivable','Forecast','Hedge','Intercompany'))`.
Column `currency CHAR(3)`.
Column `legal_entity_id UUID`.
Column `valuation_basis TEXT NOT NULL CHECK (valuation_basis IN ('SpotClose','AveragePeriod','OfficialCentralBank','ContractRate','HistoricalCost'))`.
Column `rate_source_priority TEXT[] NOT NULL`.
Column `materiality_threshold_base NUMERIC(22,4) NOT NULL`.
Column `stale_rate_max_age_seconds INTEGER NOT NULL`.
Column `requires_controller_approval BOOLEAN NOT NULL DEFAULT false`.
Column `active BOOLEAN NOT NULL DEFAULT true`.
Constraint `UNIQUE (policy_id, position_source_type, currency, legal_entity_id)`.
Table `treasury.revaluation_run`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `policy_id UUID NOT NULL REFERENCES treasury.revaluation_policy(id)`.
Column `run_code TEXT NOT NULL`.
Column `business_date DATE NOT NULL`.
Column `rate_set_id UUID NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Draft','PendingApproval','Approved','Superseded','Failed'))`.
Column `source_position_count INTEGER NOT NULL`.
Column `exception_count INTEGER NOT NULL`.
Column `total_gain_loss_base NUMERIC(22,4) NOT NULL`.
Column `evidence_hash TEXT NOT NULL`.
Column `computed_at TIMESTAMPTZ NOT NULL`.
Column `approved_by_principal_id UUID`.
Column `approved_at TIMESTAMPTZ`.
Column `cedar_decision_id UUID NOT NULL`.
Constraint `UNIQUE (tenant_id, run_code)`.
Index `ix_revaluation_run_status` on `(tenant_id, business_date, status)`.
Table `treasury.revaluation_result_line`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `run_id UUID NOT NULL REFERENCES treasury.revaluation_run(id)`.
Column `policy_line_id UUID NOT NULL REFERENCES treasury.revaluation_policy_line(id)`.
Column `source_position_id TEXT NOT NULL`.
Column `source_type TEXT NOT NULL`.
Column `legal_entity_id UUID`.
Column `currency CHAR(3) NOT NULL`.
Column `source_amount NUMERIC(22,4) NOT NULL`.
Column `carrying_base_amount NUMERIC(22,4) NOT NULL`.
Column `revalued_base_amount NUMERIC(22,4) NOT NULL`.
Column `gain_loss_base_amount NUMERIC(22,4) NOT NULL`.
Column `rate_id UUID`.
Column `valuation_basis TEXT NOT NULL`.
Column `material BOOLEAN NOT NULL`.
Column `exception_code TEXT`.
Constraint `UNIQUE (run_id, source_position_id, source_type)`.
Table `treasury.revaluation_exception`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `run_id UUID NOT NULL REFERENCES treasury.revaluation_run(id)`.
Column `source_position_id TEXT`.
Column `severity TEXT NOT NULL CHECK (severity IN ('Info','Warning','Blocking'))`.
Column `code TEXT NOT NULL`.
Column `message TEXT NOT NULL`.
Column `resolved_by_principal_id UUID`.
Column `resolved_at TIMESTAMPTZ`.
Storage rule: policy lines are versioned by new policy rows; active policy lines are not mutated after approval.
Partitioning rule: result lines partition by tenant cell and business month.
Retention rule: retain approved runs and result lines for ten years.

## API Endpoints
REST `POST /v1/treasury/revaluation-policies`.
Request example:
```json
{
  "policy_code": "GLOBAL-TREASURY-IFRS-2026",
  "base_currency": "USD",
  "effective_from": "2026-05-01",
  "lines": [
    {
      "position_source_type": "Cash",
      "valuation_basis": "SpotClose",
      "rate_source_priority": ["ECB", "Reuters"],
      "materiality_threshold_base": "25000.00",
      "stale_rate_max_age_seconds": 3600
    }
  ]
}
```
Response example:
```json
{
  "policy_id": "8eab3a21-1111-4777-a222-333344445555",
  "status": "Draft",
  "line_count": 1
}
```
REST `POST /v1/treasury/revaluation-policies/{policy_id}/approve`.
REST `POST /v1/treasury/revaluation-runs`.
Run request includes `policy_id`, `business_date`, `rate_set_id`, `run_code`, and idempotency key.
Run response includes run id, status, source count, exception count, total gain/loss, and evidence hash.
REST `GET /v1/treasury/revaluation-runs/{run_id}` returns header, result lines, and exceptions.
REST `POST /v1/treasury/revaluation-runs/{run_id}/approve`.
REST `POST /v1/treasury/revaluation-runs/{run_id}/handoff-to-ledger`.
Handoff response returns finance-ledger batch id and accepted line count.
gRPC `TreasuryRevaluationService.ComputeRun(ComputeRevaluationRunRequest) returns (RevaluationRun)`.
gRPC `TreasuryRevaluationService.GetRun(GetRevaluationRunRequest) returns (RevaluationRunDetail)`.
Error `422 REVALUATION_POLICY_COVERAGE_GAP` when a source position has no policy line.
Error `412 REVALUATION_RATE_STALE` when any required rate violates stale threshold.
Error `403 REVALUATION_APPROVAL_DENIED` when Cedar blocks policy or run approval.

## Cedar Policy Hooks
Principal shape: `User::{ id, tenant_id, roles, controller_scope, approval_limit_base, policy_admin }`.
Action `Action::"approve_revaluation_policy"`.
Action `Action::"compute_revaluation_run"`.
Action `Action::"approve_revaluation_run"`.
Action `Action::"handoff_revaluation_to_ledger"`.
Resource `RevaluationRun::{ tenant_id, status, total_gain_loss_base, exception_count, policy_id, material_line_count }`.
Context `RevaluationContext::{ now, expected_evidence_hash, rate_set_age_seconds, blocking_exception_count, ledger_period_open }`.
Permit treasury accounting managers to compute runs for active policies.
Permit controllers to approve runs within approval limit and controller scope.
Forbid approval when blocking exception count is greater than zero.
Forbid approval when expected evidence hash differs from run evidence hash.
Forbid ledger handoff when ledger period is closed.
Forbid policy approval by the policy creator unless principal has independent controller role.
Emit `MultiCurrencyRevaluationPolicyDenied` on every deny.
Policy fixture `policy/revaluation-policy-self-approval-deny.json`.
Policy fixture `policy/revaluation-run-blocking-exception-deny.json`.
Policy fixture `policy/revaluation-ledger-period-closed-deny.json`.

## Ontology Projection
SAP valuation area maps to `Oyatie::Treasury::RevaluationPolicy`.
SAP valuation rule maps to `RevaluationPolicyLine`.
SAP TRM valuation run maps to `RevaluationRun`.
SAP valuation result item maps to `RevaluationResultLine`.
SAP exchange rate type maps to `valuation_basis` and `rate_source_priority`.
Kyriba accounting revaluation export maps to run and result lines.
GTreasury valuation worksheet maps to result lines and exceptions.
Oracle revaluation process output maps to result lines and ledger handoff.
Ontology field `RevaluationPolicy.policyCode` maps from `policy_code`.
Ontology field `RevaluationRun.businessDate` maps from `business_date`.
Ontology field `RevaluationRun.totalGainLossBase` maps from `total_gain_loss_base`.
Ontology field `RevaluationResultLine.sourcePosition` maps from `source_position_id`.
Ontology field `RevaluationResultLine.carryingBaseAmount` maps from `carrying_base_amount`.
Ontology field `RevaluationResultLine.revaluedBaseAmount` maps from `revalued_base_amount`.
Ontology field `RevaluationResultLine.gainLossBaseAmount` maps from `gain_loss_base_amount`.
Ontology edge `POLICY_HAS_LINE` connects policy to policy line.
Ontology edge `RUN_USES_POLICY` connects run to policy.
Ontology edge `RUN_CONTAINS_RESULT_LINE` connects run to result line.
Ontology edge `RESULT_LINE_REVALUES_POSITION` connects result line to source position.
Ontology edge `RUN_HANDOFF_TO_LEDGER_BATCH` connects approved run to finance-ledger batch.
Projection must include material flag for variance analytics.

## Workflow Steps
Workflow `treasury.revaluation.policy_approve`.
Node `load_policy_for_update` verifies Draft status.
Node `validate_policy_line_coverage` checks duplicate line specificity.
Node `cedar_policy_approval_check` prevents unapproved self-approval.
Node `mark_policy_active` retires overlapping active policy when effective dates overlap.
Node `emit_policy_approved`.
Workflow `treasury.revaluation.compute_run`.
Node `load_active_policy`.
Node `load_source_positions` reads cash, debt, AP, AR, hedge, forecast, and intercompany sources.
Node `match_policy_lines` assigns exactly one policy line per source position.
Node `resolve_rates_by_basis` selects rates from rate set and priority list.
Node `detect_stale_or_missing_rates` creates blocking exceptions.
Node `calculate_revalued_amounts` computes carrying, revalued, and gain/loss base amounts.
Node `classify_material_variances` compares against policy thresholds.
Node `compute_revaluation_evidence_hash` hashes sources, policy, rates, and result values.
Node `persist_run_results` writes run, lines, and exceptions transactionally.
Node `emit_run_computed`.
Branch `policy_coverage_gap` writes Failed run and blocking exceptions.
Branch `no_blocking_exceptions` sets status PendingApproval.
Workflow `treasury.revaluation.handoff_to_ledger`.
Node `cedar_handoff_check`.
Node `build_ledger_revaluation_payload`.
Node `submit_to_finance_ledger`.
Node `record_ledger_batch_id`.
Node `emit_ledger_handoff_completed`.

## Audit Events
Audit event class `TreasuryRevaluationPolicyCreated`.
Audit event class `TreasuryRevaluationPolicyApproved`.
Audit event class `TreasuryRevaluationPolicyRetired`.
Audit event class `TreasuryRevaluationRunRequested`.
Audit event class `TreasuryRevaluationRunComputed`.
Audit event class `TreasuryRevaluationRunFailed`.
Audit event class `TreasuryRevaluationExceptionRaised`.
Audit event class `TreasuryRevaluationRunApproved`.
Audit event class `TreasuryRevaluationLedgerHandoffSubmitted`.
Audit event class `TreasuryRevaluationPolicyDenied`.
Audit event class `TreasuryRevaluationRunSuperseded`.
Audit payload must include tenant id, policy id, run id, business date, rate set id, and evidence hash.
Audit payload for policy events must include effective dates and policy line count.
Audit payload for result events must include total gain loss and material line count.
Audit payload for denies must include Cedar decision id and denied action.
Audit retention class is `TreasuryMultiCurrencyRevaluation`.
Audit ordering key is `tenant_id:business_date:policy_code`.

## SLO Targets
p50 run computation for 50000 source positions: 1200 ms.
p95 run computation for 50000 source positions: 4500 ms.
p99 run computation for 50000 source positions: 9000 ms.
p50 run read for 1000 result lines: 100 ms.
p95 run read for 1000 result lines: 350 ms.
p99 run read for 1000 result lines: 800 ms.
Throughput target: 20 revaluation runs per minute per tenant cell.
Throughput target: 75000 result lines written per minute per cell.
Availability target for run read API: 99.99 percent monthly.
Availability target for compute API: 99.95 percent monthly.
Rationale: revaluation is close-critical but batch-like, so compute p99 can be higher than read p99.
Rationale: result write throughput supports enterprise close without file splitting.
Rationale: approval and read paths need higher availability because finance-ledger close depends on them.

## Failure Modes + Recovery
Failure `POLICY_COVERAGE_GAP`: detect unmatched source position; recover by adding policy line and recomputing.
Failure `MULTIPLE_POLICY_LINES_MATCH`: detect ambiguous specificity; recover by retiring or narrowing overlapping line.
Failure `RATE_MISSING`: detect no rate from priority list; recover by loading approved rate or blocking run.
Failure `RATE_STALE`: detect stale rate age; recover by requesting fresh rate set.
Failure `SOURCE_POSITION_STALE`: detect source watermark lag; recover by recomputing after source catches up.
Failure `LEDGER_PERIOD_CLOSED`: detect finance-ledger status; recover by reopening period or creating next-period handoff.
Failure `APPROVAL_HASH_MISMATCH`: detect stale UI evidence; recover by refreshing run view.
Failure `MATERIAL_VARIANCE_UNAPPROVED`: detect threshold breach without controller role; recover by routing approval to controller.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain error; recover by aborting transition and retrying.
Failure `PARTIAL_RESULT_WRITE`: prevent with transaction; repair by marking run Failed and recomputing.
Recovery worker `treasury.revaluation.ledger_handoff_reconcile` polls finance-ledger batch acceptance.
Runbook entry `runbooks/multi-currency-revaluation-failure.md` should cover rate, policy, and ledger handoff failures.

## Migration Notes
Source vendor surface: SAP TRM-TM valuation customizing.
Source vendor surface: SAP exchange rate type configuration.
Source vendor surface: SAP Market Risk Analyzer valuation outputs.
Source vendor surface: SAP cash management currency positions.
Source vendor surface: Kyriba accounting revaluation export.
Source vendor surface: GTreasury valuation worksheet.
Source vendor surface: Oracle General Ledger revaluation output for comparison.
Migration maps SAP valuation area to revaluation policy.
Migration maps SAP valuation class and product type to position source type.
Migration maps SAP exchange rate type to valuation basis and rate priority.
Migration maps SAP valuation date to revaluation run business date.
Migration imports historical valuation outputs as result lines with source type `legacy-import`.
Migration dry-run report lists source types without policy coverage.
Migration dry-run report lists currencies without approved rate sources.
Migration acceptance requires a prior close cycle to match gain/loss totals within tenant tolerance.

## Cross-microservice Handoffs
Handoff to `rates`: consume immutable rate sets and rate ids.
Handoff to `cash-position`: consume cash balances and approved rollups.
Handoff to `debt`: consume debt principal, interest, and carrying values.
Handoff to `accounts-payable`: consume open payables by currency and legal entity.
Handoff to `accounts-receivable`: consume open receivables by currency and legal entity.
Handoff to `hedge-designation`: consume hedge positions and accounting basis.
Handoff to `intercompany-netting`: consume approved residual intercompany balances.
Handoff to `finance-ledger`: submit approved revaluation result payload.
Handoff to `workflow`: run policy approval, compute, approval, and handoff nodes.
Handoff to `audit-chain`: seal policy, run, approval, and ledger handoff evidence.

## Build Notes
Add database migration for policy, policy line, run, result line, and exception tables.
Add domain service `MultiCurrencyRevaluationPolicyService`.
Add domain service `RevaluationRunCalculator`.
Add deterministic policy-line matching with specificity tests.
Add Cedar schema for revaluation run and context.
Add REST handlers for policy create, policy approve, run compute, run read, run approve, and ledger handoff.
Add gRPC handlers for compute and read.
Add contract tests for policy coverage gap, stale rate, and closed ledger period.
Add workflow tests for material variance approval and superseding run.
Add load fixture with 50000 source positions and 40 currencies.
Add migration fixture with SAP valuation area and exchange-rate-type export.
Add dashboard panels for run latency, exception count, material variance count, and ledger handoff status.
