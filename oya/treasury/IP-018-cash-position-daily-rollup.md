---
doc_class: ImplementationPlan
ip_id: IP-018
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
sap_submodule_equivalents: [TRM-CM Cash Position, TRM-CM Liquidity Forecast Closing, TRM-TM Memo Records, TRM-RM Liquidity Risk Inputs]
---

# IP-018: Cash position daily rollup

## Intent
Implement a deterministic daily cash-position rollup that turns intraday bank-account balance events, statement lines, sweep movements, and pending payments into one tenant-scoped treasury close view.
This plan replaces spreadsheet-based daily position packs and the SAP TRM-CM cash position batch report for the covered journey leg.
The slice is intern-buildable because every persisted table, API, policy hook, workflow node, and test fixture is named.
The rollup is not a general ledger close and does not post accounting journals.
The rollup produces treasury operating evidence for cash managers, auditors, and liquidity forecast consumers.
The design assumes bank-account, cash-position, payment-batch, and sweep-movement primitives already exist from earlier treasury IPs.
The implementation must preserve cell locality and tenant isolation from ADR-0105 and ADR-0319.
The implementation must emit audit events shaped for ADR-0263 audit-chain ingestion.
The implementation must expose both REST and gRPC surfaces because close orchestration and Ops dashboards consume different protocols.
The implementation must not depend on SAP table names at runtime; vendor names appear only in migration adapters and documentation.

## Context
Why: group treasury needs one trusted daily cash position that survives late bank statements, repeated payment callbacks, and cross-currency sweeps.
Why: SAP TRM-CM provides cash position reports, but tenant teams still export CSVs because the batch view cannot explain late-arriving intraday deltas.
Why: Oyatie needs a first-class rollup so liquidity forecasting and payment release can consume the same close state.
Journey leg: j120 treasury close prepares the trusted cash state before a multi-currency hedge decision.
Named persona: Sven Eriksson, Group Treasurer, owns the close dashboard and signs the exception report before 18:00 Europe/Stockholm.
Supporting persona: Mina Cho, Treasury Operations Analyst, fixes rejected bank statement mappings and re-runs only affected accounts.
Pain point: a single late CAMT.053 file currently forces a full spreadsheet rebuild.
Pain point: pending payment batches are counted twice when bank callbacks race with statement imports.
Pain point: cash-pool sweeps from IP-016 must appear as movements before payments settle, but only once in final settled cash.
SAP parity: TRM-CM cash position, memo records, liquidity forecast closing, and TRM-TM transaction cash-flow feeds.
Product outcome: treasury can answer "what was the close cash by account, currency, pool, and region?" with a sealed evidence hash.
Non-goal: intraday trader PnL and valuation are handled by IP-020 and IP-022.
Non-goal: accounting journal posting remains in finance-ledger.
Non-goal: bank connectivity and raw file parsing remain in bank-statement adapters.
Invariant: one tenant, one business date, one close profile produces at most one active rollup version.
Invariant: every rollup line is explainable by source event ids or an explicit manual adjustment with Cedar approval.
Invariant: source events are never deleted or mutated by the rollup worker.
Invariant: a superseded rollup version remains queryable for audit.
Acceptance anchor: an intern can create migrations, handlers, worker nodes, policy checks, fixtures, and runbook entries from this file.

## Data Model Deltas
Table `treasury.cash_position_daily_rollup`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `business_date DATE NOT NULL`.
Column `close_profile_id UUID NOT NULL`.
Column `rollup_version INTEGER NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Draft','PendingApproval','Approved','Superseded','Failed'))`.
Column `base_currency CHAR(3) NOT NULL`.
Column `total_base_amount NUMERIC(22,4) NOT NULL`.
Column `source_event_count INTEGER NOT NULL`.
Column `exception_count INTEGER NOT NULL`.
Column `computed_at TIMESTAMPTZ NOT NULL`.
Column `computed_by_principal_id UUID`.
Column `approved_by_principal_id UUID`.
Column `approved_at TIMESTAMPTZ`.
Column `evidence_hash TEXT NOT NULL`.
Column `cedar_decision_id UUID NOT NULL`.
Constraint `UNIQUE (tenant_id, business_date, close_profile_id, rollup_version)`.
Index `ix_daily_rollup_active` on `(tenant_id, business_date, status)`.
Table `treasury.cash_position_daily_rollup_line`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `rollup_id UUID NOT NULL REFERENCES treasury.cash_position_daily_rollup(id)`.
Column `bank_account_id UUID NOT NULL`.
Column `cash_pool_id UUID`.
Column `currency CHAR(3) NOT NULL`.
Column `opening_available NUMERIC(22,4) NOT NULL`.
Column `statement_delta NUMERIC(22,4) NOT NULL`.
Column `payment_delta NUMERIC(22,4) NOT NULL`.
Column `sweep_delta NUMERIC(22,4) NOT NULL`.
Column `manual_adjustment_delta NUMERIC(22,4) NOT NULL DEFAULT 0`.
Column `closing_available NUMERIC(22,4) NOT NULL`.
Column `base_amount NUMERIC(22,4) NOT NULL`.
Column `fx_rate_id UUID`.
Column `source_event_ids UUID[] NOT NULL`.
Column `exception_code TEXT`.
Constraint `UNIQUE (rollup_id, bank_account_id, currency)`.
Table `treasury.cash_position_close_profile`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `name TEXT NOT NULL`.
Column `business_calendar_id UUID NOT NULL`.
Column `close_cutoff_time TIME NOT NULL`.
Column `close_timezone TEXT NOT NULL`.
Column `late_event_grace_minutes INTEGER NOT NULL DEFAULT 30`.
Column `include_pending_payments BOOLEAN NOT NULL DEFAULT true`.
Column `include_unexecuted_sweeps BOOLEAN NOT NULL DEFAULT true`.
Column `approval_threshold_base NUMERIC(22,4) NOT NULL DEFAULT 0`.
Constraint `UNIQUE (tenant_id, name)`.
Table `treasury.cash_position_rollup_exception`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `rollup_id UUID NOT NULL REFERENCES treasury.cash_position_daily_rollup(id)`.
Column `bank_account_id UUID`.
Column `severity TEXT NOT NULL CHECK (severity IN ('Info','Warning','Blocking'))`.
Column `code TEXT NOT NULL`.
Column `message TEXT NOT NULL`.
Column `source_ref TEXT`.
Column `resolved_by_principal_id UUID`.
Column `resolved_at TIMESTAMPTZ`.
Storage rule: rollup tables are append-only except status transitions Draft to PendingApproval to Approved or Superseded.
Partitioning rule: partition rollup lines by tenant cell and business month.
Retention rule: keep approved rollups for ten years through audit-chain retention class `TreasuryCashPositionClose`.

## API Endpoints
REST `POST /v1/treasury/cash-position-rollups`.
Request example:
```json
{
  "business_date": "2026-05-20",
  "close_profile_id": "8f4c5f6d-1111-4b2a-8f3b-111122223333",
  "mode": "compute",
  "idempotency_key": "northstream:2026-05-20:primary-close"
}
```
Response example:
```json
{
  "rollup_id": "4d9f2e01-2f2e-4a7e-8e63-444455556666",
  "status": "Draft",
  "rollup_version": 3,
  "source_event_count": 18421,
  "exception_count": 2,
  "evidence_hash": "sha256:7d2c..."
}
```
REST `GET /v1/treasury/cash-position-rollups/{rollup_id}` returns header, lines, exceptions, and approval history.
REST `POST /v1/treasury/cash-position-rollups/{rollup_id}/approve`.
Approve request includes `approval_comment`, `expected_evidence_hash`, and optional `exception_acceptance_reason`.
Approve response includes `status`, `approved_at`, `approved_by_principal_id`, and `audit_event_id`.
REST `POST /v1/treasury/cash-position-rollups/{rollup_id}/supersede`.
Supersede request includes `replacement_rollup_id` and `reason_code`.
REST `GET /v1/treasury/cash-position-rollups?business_date=2026-05-20&status=Approved`.
gRPC `TreasuryCashPositionRollupService.ComputeDailyRollup(ComputeDailyRollupRequest) returns (DailyRollup)`.
gRPC request fields: `tenant_id`, `business_date`, `close_profile_id`, `idempotency_key`, `requested_by_principal_id`.
gRPC response fields: `rollup_id`, `version`, `status`, `source_event_count`, `line_count`, `exception_count`, `evidence_hash`.
Error `409 ROLLUP_ALREADY_APPROVED` when an active approved rollup exists and caller did not request supersede.
Error `412 SOURCE_EVENT_WATERMARK_TOO_OLD` when bank statement ingestion is behind the profile cutoff.
Error `403 CASH_POSITION_ROLLUP_APPROVAL_DENIED` when Cedar denies approval.
Idempotency key scope is tenant plus business date plus close profile.

## Cedar Policy Hooks
Principal shape: `User::{ id, tenant_id, roles, region_scope, approval_limit_base }`.
Action `Action::"compute_cash_position_daily_rollup"`.
Action `Action::"approve_cash_position_daily_rollup"`.
Action `Action::"supersede_cash_position_daily_rollup"`.
Resource `CashPositionDailyRollup::{ tenant_id, business_date, total_base_amount, exception_count, status, close_profile_id }`.
Context `CloseContext::{ now, cell_id, source_watermarks, request_ip, device_posture, expected_evidence_hash }`.
Permit cash managers to compute when resource tenant equals principal tenant.
Permit close approvers to approve when status is PendingApproval and principal approval limit covers `total_base_amount`.
Forbid approval when `context.expected_evidence_hash != resource.evidence_hash`.
Forbid approval when any blocking exception remains unresolved.
Forbid supersede unless principal has role `treasury-close-supervisor`.
Require region scope to include every bank-account region represented by rollup lines.
Log every deny as `TreasuryCashPositionRollupPolicyDenied`.
Attach `cedar_decision_id` to the rollup header and approval audit event.
Policy test fixture `policy/cash-position-rollup-approval-large-amount.json`.
Policy test fixture `policy/cash-position-rollup-hash-mismatch.json`.
Policy test fixture `policy/cash-position-rollup-region-deny.json`.

## Ontology Projection
SAP `FF7A` cash position result maps to `Oyatie::Treasury::CashPositionDailyRollup`.
SAP `FQM_FLOW` cash-management flow maps to `source_event_ids` on rollup lines.
SAP memo record maps to `payment_delta` when source status is pending and approved.
SAP bank statement item maps to `statement_delta` after reconciliation.
SAP cash-pool transfer maps to `sweep_delta` with sweep movement id as source.
Kyriba daily cash worksheet maps to rollup header plus line set.
GTreasury cash worksheet maps to rollup header and exception rows.
Oracle Cash Management bank balance report maps to account-currency rollup lines.
Ontology field `CashPositionDailyRollup.tenant` maps from `tenant_id`.
Ontology field `CashPositionDailyRollup.businessDate` maps from `business_date`.
Ontology field `CashPositionDailyRollup.baseCurrency` maps from `base_currency`.
Ontology field `CashPositionDailyRollup.totalBaseAmount` maps from `total_base_amount`.
Ontology field `CashPositionDailyRollup.evidenceHash` maps from `evidence_hash`.
Ontology field `CashPositionRollupLine.account` maps from `bank_account_id`.
Ontology field `CashPositionRollupLine.pool` maps from `cash_pool_id`.
Ontology field `CashPositionRollupLine.closingAvailable` maps from `closing_available`.
Ontology edge `ROLLUP_CONTAINS_LINE` connects header to each line.
Ontology edge `LINE_DERIVED_FROM_EVENT` connects line to each source event id.
Ontology edge `ROLLUP_APPROVED_BY` connects header to principal.
Projection must be deterministic for the same rollup id.

## Workflow Steps
Workflow `treasury.cash_position.daily_rollup_compute`.
Node `load_close_profile` validates cutoff time, timezone, and grace minutes.
Node `read_source_watermarks` reads bank-statement, payments, and sweep worker watermarks.
Node `assert_watermarks_after_cutoff` branches to `block_for_stale_source` when any required source is late.
Node `load_opening_balances` reads prior approved rollup or first statement balance.
Node `load_statement_deltas` collects reconciled statement lines for business date.
Node `load_payment_deltas` collects pending and settled payment instructions using end-to-end id.
Node `load_sweep_deltas` collects IP-016 sweep movements and avoids double-counting settled payments.
Node `apply_manual_adjustments` includes only Cedar-approved adjustments.
Node `resolve_fx_rates` snapshots base-currency rates with rate ids.
Node `build_account_currency_lines` groups by account and currency.
Node `detect_exceptions` creates warning or blocking exception rows.
Node `compute_evidence_hash` hashes sorted source ids and line values.
Node `persist_rollup_version` appends header and lines in one transaction.
Node `emit_rollup_computed` publishes audit and workflow event.
Branch `block_for_stale_source` creates Failed rollup attempt with `SOURCE_EVENT_WATERMARK_TOO_OLD`.
Branch `manual_adjustment_missing_approval` creates blocking exception and leaves status Draft.
Branch `zero_exception_auto_pending_approval` sets status PendingApproval.
Workflow `treasury.cash_position.daily_rollup_approve`.
Node `reload_rollup_for_update` locks header row.
Node `cedar_approval_check` evaluates principal, action, resource, context.
Node `compare_evidence_hash` rejects stale UI approval.
Node `mark_approved` sets approval fields.
Node `supersede_prior_active` marks older active approved version Superseded.
Node `emit_rollup_approved` publishes audit-chain event.

## Audit Events
Audit event class `TreasuryCashPositionDailyRollupComputeRequested`.
Audit event class `TreasuryCashPositionDailyRollupComputed`.
Audit event class `TreasuryCashPositionDailyRollupFailed`.
Audit event class `TreasuryCashPositionDailyRollupApprovalRequested`.
Audit event class `TreasuryCashPositionDailyRollupApproved`.
Audit event class `TreasuryCashPositionDailyRollupSuperseded`.
Audit event class `TreasuryCashPositionDailyRollupExceptionRaised`.
Audit event class `TreasuryCashPositionDailyRollupExceptionResolved`.
Audit event class `TreasuryCashPositionDailyRollupPolicyDenied`.
Audit payload must include `tenant_id`, `business_date`, `rollup_id`, `rollup_version`, and `evidence_hash`.
Audit payload must include source watermarks for compute events.
Audit payload must include `cedar_decision_id` for approval, deny, and supersede actions.
Audit retention class is `TreasuryCashPositionClose`.
Audit severity is `Info` for compute and approved, `Warning` for exceptions, and `Security` for policy denied.
Audit chain ordering key is `tenant_id:business_date:close_profile_id`.

## SLO Targets
p50 compute latency for 500 accounts and 10000 source events: 450 ms.
p95 compute latency for 500 accounts and 10000 source events: 1800 ms.
p99 compute latency for 500 accounts and 10000 source events: 3500 ms.
p50 approval latency: 80 ms.
p95 approval latency: 250 ms.
p99 approval latency: 500 ms.
Throughput target: 120 rollup computations per minute per cell.
Throughput target: 20000 rollup lines written per minute per cell.
Availability target for compute API: 99.95 percent monthly.
Availability target for approved rollup read API: 99.99 percent monthly.
Freshness target: source watermark lag below 15 minutes for core banking feeds.
Rationale: daily close is time-bound, but reads become a high-availability dependency for dashboards and liquidity forecast.
Rationale: p99 under 3500 ms keeps re-run workflows practical during close windows.
Rationale: line write throughput supports large enterprise tenants without a batch-only architecture.

## Failure Modes + Recovery
Failure `SOURCE_EVENT_WATERMARK_TOO_OLD`: detect from source watermark table; recover by re-running after source catches up.
Failure `FX_RATE_MISSING`: detect during rate resolution; recover by using approved prior-day rate only when Cedar override exists.
Failure `PAYMENT_DUPLICATE_SOURCE`: detect duplicate end-to-end id across payment and statement; recover by marking one source as settlement confirmation.
Failure `MANUAL_ADJUSTMENT_UNAPPROVED`: detect adjustment without approval event; recover by routing to treasury close approver.
Failure `EVIDENCE_HASH_MISMATCH`: detect approve request mismatch; recover by forcing UI refresh and re-approval.
Failure `ROLLUP_WRITE_CONFLICT`: detect unique constraint conflict; recover by incrementing version after reloading active header.
Failure `AUDIT_CHAIN_UNAVAILABLE`: detect audit append failure; recover by leaving rollup Draft and retrying audit append.
Failure `TENANT_REGION_SCOPE_DENIED`: detect Cedar deny; recover by assigning approver with correct region scope.
Failure `LATE_BANK_STATEMENT_AFTER_APPROVAL`: detect source event timestamp after approved close; recover by creating superseding rollup version.
Failure `PARTIAL_LINE_PERSISTENCE`: prevent with transaction; if detected by integrity scan, mark rollup Failed and alert.
Recovery worker `treasury.cash_position.rollup_repair` only creates a new version; it never mutates approved line values.
Runbook entry `runbooks/cash-position-daily-rollup-close-failure.md` should be created by the implementation PR.
Operator action is limited to retry, supersede, or resolve exceptions; raw source values remain immutable.

## Migration Notes
Source vendor surface: SAP TRM-CM cash position report `FF7A`.
Source vendor surface: SAP S/4HANA cash management flow table `FQM_FLOW`.
Source vendor surface: SAP memo records used for expected payments.
Source vendor surface: SAP Bank Communication Management payment status.
Source vendor surface: Kyriba cash worksheet export.
Source vendor surface: GTreasury cash positioning workbook.
Source vendor surface: Oracle Cash Management bank balances.
Migration maps each SAP company code and house bank account to `bank_account_id`.
Migration maps SAP planning level to source event class and exception code.
Migration maps memo record expiry date to payment delta inclusion window.
Migration imports only approved historical close packs as rollup headers and lines.
Migration must compute evidence hashes during import and label them as `legacy-import`.
Migration must not infer approvals when no approver exists; use `LegacyApprovedByExternalSystem`.
Migration dry-run report lists account mismatches, currency mismatches, and unmapped planning levels.
Migration acceptance requires one month of parallel close matching within tenant tolerance.

## Cross-microservice Handoffs
Handoff to `bank-statement`: consume reconciled statement lines and feed watermarks.
Handoff to `payments`: consume payment batches, payment instructions, and bank ack state.
Handoff to `liquidity-forecast`: publish approved closing available amounts by account and currency.
Handoff to `workflow`: execute compute and approval workflow nodes.
Handoff to `ontology`: project rollup graph nodes and derivation edges.
Handoff to `audit-chain`: seal compute, approve, supersede, and exception events.
Handoff to `identity`: resolve principal roles, region scope, and approval limits.
Handoff to `calendar`: validate close business date and local cutoff.
Handoff to `rates`: snapshot base-currency FX rates.
Handoff to `ops-dashboard`: expose close status, exception count, and source watermark lag.

## Build Notes
Add database migration for four tables and indexes named above.
Add repository methods for idempotent compute request lookup.
Add domain service `DailyCashPositionRollupService`.
Add worker handler for `treasury.cash_position.daily_rollup_compute`.
Add REST controller methods for compute, get, approve, and supersede.
Add gRPC service implementation for compute and read.
Add Cedar schema entities for `CashPositionDailyRollup` and `CloseContext`.
Add contract tests for REST idempotency and approval hash mismatch.
Add workflow tests for late CAMT.053 source events.
Add load fixture with 500 accounts, 12 currencies, 10000 source events.
Add migration dry-run fixture with SAP FF7A export columns.
Add SLO dashboard panels for compute latency, read latency, and stale watermarks.
