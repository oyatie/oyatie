---
doc_class: ImplementationPlan
ip_id: IP-019
microservice: treasury
related_adrs: [ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0319]
journey_id: j122-vendor-payment-batch-with-tax-withholding
journey_link: docs/user-journeys/j122-vendor-payment-batch-with-tax-withholding/story.md
status: Accepted
date: 2026-05-20
owner: axis-treasury
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [TRM-CM Cash Management, TRM-TM Money Market Settlement, TRM-RM Counterparty Exposure]
---

# IP-019: Intercompany netting matrix

## Intent
Implement an intercompany netting matrix that compresses affiliate payables and receivables into currency-aware settlement obligations.
The matrix reduces bank fees, trapped cash, and operational payment volume without hiding affiliate-level obligation evidence.
The plan displaces SAP in-house cash netting workbooks and selected TRM-TM settlement surfaces.
The implementation must preserve legal-entity separations and approval boundaries.
The implementation must produce a matrix that finance-ledger can reconcile to source invoices and treasury can settle through payment factory routes.
The implementation must support multi-currency netting groups while keeping one settlement currency per matrix cell.
The implementation must treat sanctions, capital controls, and blocked jurisdictions as hard gates.
The implementation must be deterministic for the same source obligation set and cutoff profile.
The implementation must emit ADR-0263 audit classes for every matrix build, exclusion, approval, and settlement handoff.
The implementation must not create accounting entries; it creates settlement instructions and evidence.

## Context
Why: multinational tenants settle thousands of affiliate invoices with gross payments even when entities owe each other in opposite directions.
Why: SAP IHC and TRM-TM flows usually require specialized consultants for netting calendars, affiliate cutoffs, and exception handling.
Why: Oyatie needs a tenant-native matrix that can feed payments, cash position, and risk exposure without manual Excel consolidation.
Journey leg: j122 settlement preparation compresses affiliate obligations before payment release.
Named persona: Priya Raman, Treasury Shared Services Lead at Metrica Components, owns the intercompany payment calendar.
Supporting persona: Lucas Meyer, Regional Controller, reviews legal-entity exclusions and tax-sensitive offsets.
Pain point: affiliates upload invoices late, forcing spreadsheet recalculation and inconsistent settlement numbers.
Pain point: blocked jurisdictions must be excluded without losing the obligation audit trail.
Pain point: currency conversion rate selection is contested between treasury, tax, and accounting teams.
SAP parity: SAP FIN-FSCM-IHC netting center, TRM-TM settlement flows, and TRM-RM counterparty exposure inputs.
Product outcome: one matrix shows gross, excluded, net, settlement, and residual obligation by payer, payee, currency, and legal entity.
Non-goal: transfer-pricing policy calculation remains in tax services.
Non-goal: invoice dispute management remains in accounts payable and receivable.
Non-goal: payment execution details remain in IP-025 payment factory routing.
Invariant: no obligation is netted unless both source entity and counterparty entity are active matrix members.
Invariant: every exclusion has a reason code and source obligation id.
Invariant: settlement totals equal source gross less excluded obligations within decimal precision.
Invariant: a matrix can be approved once and superseded only by a higher version.
Acceptance anchor: an intern can implement the schema, matrix builder, approvals, policy tests, and handoff contracts from this file.

## Data Model Deltas
Table `treasury.intercompany_netting_cycle`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `cycle_code TEXT NOT NULL`.
Column `business_date DATE NOT NULL`.
Column `cutoff_at TIMESTAMPTZ NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Draft','PendingApproval','Approved','Settling','Settled','Superseded','Failed'))`.
Column `base_currency CHAR(3) NOT NULL`.
Column `netting_calendar_id UUID NOT NULL`.
Column `rate_set_id UUID NOT NULL`.
Column `matrix_version INTEGER NOT NULL`.
Column `source_obligation_count INTEGER NOT NULL`.
Column `excluded_obligation_count INTEGER NOT NULL`.
Column `gross_base_amount NUMERIC(22,4) NOT NULL`.
Column `net_base_amount NUMERIC(22,4) NOT NULL`.
Column `evidence_hash TEXT NOT NULL`.
Column `cedar_decision_id UUID NOT NULL`.
Constraint `UNIQUE (tenant_id, cycle_code, matrix_version)`.
Index `ix_netting_cycle_status` on `(tenant_id, business_date, status)`.
Table `treasury.intercompany_netting_member`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `cycle_id UUID NOT NULL REFERENCES treasury.intercompany_netting_cycle(id)`.
Column `legal_entity_id UUID NOT NULL`.
Column `settlement_bank_account_id UUID NOT NULL`.
Column `default_settlement_currency CHAR(3) NOT NULL`.
Column `region_code TEXT NOT NULL`.
Column `capital_control_flag BOOLEAN NOT NULL DEFAULT false`.
Column `active BOOLEAN NOT NULL DEFAULT true`.
Constraint `UNIQUE (cycle_id, legal_entity_id)`.
Table `treasury.intercompany_netting_obligation`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `cycle_id UUID NOT NULL REFERENCES treasury.intercompany_netting_cycle(id)`.
Column `source_system TEXT NOT NULL`.
Column `source_obligation_id TEXT NOT NULL`.
Column `from_legal_entity_id UUID NOT NULL`.
Column `to_legal_entity_id UUID NOT NULL`.
Column `invoice_number TEXT`.
Column `amount NUMERIC(22,4) NOT NULL`.
Column `currency CHAR(3) NOT NULL`.
Column `due_date DATE NOT NULL`.
Column `eligible BOOLEAN NOT NULL`.
Column `exclusion_reason TEXT`.
Column `rate_id UUID`.
Column `base_amount NUMERIC(22,4) NOT NULL`.
Constraint `UNIQUE (tenant_id, source_system, source_obligation_id)`.
Table `treasury.intercompany_netting_matrix_cell`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `cycle_id UUID NOT NULL REFERENCES treasury.intercompany_netting_cycle(id)`.
Column `payer_legal_entity_id UUID NOT NULL`.
Column `receiver_legal_entity_id UUID NOT NULL`.
Column `settlement_currency CHAR(3) NOT NULL`.
Column `gross_payable NUMERIC(22,4) NOT NULL`.
Column `gross_receivable NUMERIC(22,4) NOT NULL`.
Column `net_settlement_amount NUMERIC(22,4) NOT NULL`.
Column `base_amount NUMERIC(22,4) NOT NULL`.
Column `settlement_bank_account_id UUID NOT NULL`.
Column `payment_route_hint TEXT`.
Column `status TEXT NOT NULL CHECK (status IN ('Open','Approved','Submitted','Settled','Blocked'))`.
Constraint `UNIQUE (cycle_id, payer_legal_entity_id, receiver_legal_entity_id, settlement_currency)`.
Table `treasury.intercompany_netting_approval`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `cycle_id UUID NOT NULL REFERENCES treasury.intercompany_netting_cycle(id)`.
Column `approver_principal_id UUID NOT NULL`.
Column `approval_role TEXT NOT NULL`.
Column `approved_at TIMESTAMPTZ NOT NULL`.
Column `comment TEXT`.
Column `cedar_decision_id UUID NOT NULL`.
Storage rule: obligations are immutable once matrix status moves to PendingApproval.
Partitioning rule: partition obligations and cells by tenant cell and cycle month.
Retention rule: retain approved cycles and obligations for ten years.

## API Endpoints
REST `POST /v1/treasury/intercompany-netting/cycles`.
Request example:
```json
{
  "cycle_code": "METRICA-2026-05",
  "business_date": "2026-05-20",
  "cutoff_at": "2026-05-20T16:00:00Z",
  "base_currency": "EUR",
  "rate_set_id": "76d8c6aa-1111-4444-8888-999900001111"
}
```
Response example:
```json
{
  "cycle_id": "c6e0a9c8-2222-4f4f-9555-444466667777",
  "status": "Draft",
  "matrix_version": 1
}
```
REST `POST /v1/treasury/intercompany-netting/cycles/{cycle_id}/members`.
REST `POST /v1/treasury/intercompany-netting/cycles/{cycle_id}/obligations:import`.
Import response returns accepted, excluded, duplicate, and blocked counts.
REST `POST /v1/treasury/intercompany-netting/cycles/{cycle_id}/matrix:build`.
Build response returns source obligation count, cell count, excluded count, net amount, and evidence hash.
REST `POST /v1/treasury/intercompany-netting/cycles/{cycle_id}/approve`.
Approval request includes `approval_role`, `expected_evidence_hash`, and comment.
REST `POST /v1/treasury/intercompany-netting/cycles/{cycle_id}/submit-settlement`.
Submit response returns created payment-factory batch id and submitted cell ids.
REST `GET /v1/treasury/intercompany-netting/cycles/{cycle_id}/matrix`.
gRPC `TreasuryIntercompanyNettingService.BuildMatrix(BuildMatrixRequest) returns (BuildMatrixResponse)`.
gRPC request fields: `tenant_id`, `cycle_id`, `requested_by_principal_id`, `idempotency_key`.
gRPC response fields: `cycle_id`, `matrix_version`, `cell_count`, `net_base_amount`, `evidence_hash`, `exclusion_count`.
Error `409 NETTING_CYCLE_ALREADY_APPROVED` when building after approval.
Error `412 NETTING_EVIDENCE_HASH_MISMATCH` when approval uses stale evidence.
Error `403 NETTING_LEGAL_ENTITY_SCOPE_DENIED` when principal cannot approve all member entities.

## Cedar Policy Hooks
Principal shape: `User::{ id, tenant_id, roles, legal_entity_scope, approval_limit_base, region_scope }`.
Action `Action::"build_intercompany_netting_matrix"`.
Action `Action::"approve_intercompany_netting_matrix"`.
Action `Action::"submit_intercompany_netting_settlement"`.
Resource `IntercompanyNettingCycle::{ tenant_id, status, net_base_amount, member_legal_entity_ids, blocked_region_codes }`.
Context `NettingContext::{ now, expected_evidence_hash, source_cutoff_at, sanctions_screen_result, capital_control_flags }`.
Permit treasury netting managers to build cycles for their tenant.
Permit regional controllers to approve only when all member legal entities are in `principal.legal_entity_scope`.
Permit treasury settlement releasers to submit only approved cycles.
Forbid approval when any member has `capital_control_flag` and context lacks controller approval.
Forbid settlement when `context.sanctions_screen_result != "Clear"`.
Forbid approval when expected evidence hash differs from resource evidence hash.
Require approval limit to cover absolute net base amount.
Emit `IntercompanyNettingPolicyDenied` on every deny.
Policy fixture `policy/intercompany-netting-controller-scope-deny.json`.
Policy fixture `policy/intercompany-netting-capital-control-deny.json`.
Policy fixture `policy/intercompany-netting-hash-mismatch.json`.

## Ontology Projection
SAP IHC netting center maps to `Oyatie::Treasury::IntercompanyNettingCycle`.
SAP affiliate participant maps to `Oyatie::Treasury::IntercompanyNettingMember`.
SAP open item payable maps to `Oyatie::Treasury::IntercompanyNettingObligation`.
SAP settlement proposal maps to `Oyatie::Treasury::IntercompanyNettingMatrixCell`.
TRM-RM counterparty exposure input maps to net base amount by legal entity pair.
Kyriba netting cycle maps to cycle header and cells.
GTreasury intercompany netting worksheet maps to obligations and matrix cells.
Oracle intercompany balancing extract maps to source obligations.
Ontology field `NettingCycle.cycleCode` maps from `cycle_code`.
Ontology field `NettingCycle.cutoffAt` maps from `cutoff_at`.
Ontology field `NettingCycle.evidenceHash` maps from `evidence_hash`.
Ontology field `NettingMember.legalEntity` maps from `legal_entity_id`.
Ontology field `NettingObligation.sourceRef` maps from `source_system` plus `source_obligation_id`.
Ontology field `NettingMatrixCell.payer` maps from `payer_legal_entity_id`.
Ontology field `NettingMatrixCell.receiver` maps from `receiver_legal_entity_id`.
Ontology field `NettingMatrixCell.netAmount` maps from `net_settlement_amount`.
Ontology edge `CYCLE_HAS_MEMBER` connects cycle to member.
Ontology edge `CYCLE_CONTAINS_OBLIGATION` connects cycle to obligation.
Ontology edge `OBLIGATION_COMPRESSED_INTO_CELL` connects eligible obligation to matrix cell.
Ontology edge `CELL_SETTLED_BY_PAYMENT_BATCH` connects cell to IP-025 payment batch.

## Workflow Steps
Workflow `treasury.intercompany_netting.import_obligations`.
Node `load_cycle` verifies status Draft.
Node `normalize_source_obligation` validates legal entities, currencies, due dates, and signs.
Node `screen_counterparties` calls compliance for sanctions and blocked region checks.
Node `classify_eligibility` marks eligible or excluded with reason code.
Node `resolve_rates` attaches rate ids for base amount.
Node `persist_obligations` upserts by source system and source obligation id.
Node `emit_obligations_imported` publishes accepted and excluded counts.
Workflow `treasury.intercompany_netting.build_matrix`.
Node `load_members_and_obligations` collects active members and eligible obligations.
Node `exclude_non_members` creates exclusion rows for missing member pairs.
Node `group_by_pair_currency` builds gross payable and receivable totals.
Node `net_pair_currency` computes net settlement amount.
Node `choose_settlement_account` uses payer member default settlement bank account.
Node `apply_route_hint` calls payment factory route preview.
Node `compute_matrix_evidence_hash` hashes obligations, members, rates, and cells.
Node `persist_matrix_cells` replaces Draft matrix cells for version.
Node `mark_pending_approval` when no blocking exclusions remain.
Branch `blocked_jurisdiction` marks affected cells Blocked and creates blocking exception.
Branch `zero_net_pair` stores no cell but records audit count.
Workflow `treasury.intercompany_netting.submit_settlement`.
Node `cedar_submit_check` verifies approver and sanctions context.
Node `build_payment_factory_batch` creates one batch with cells as instructions.
Node `mark_cells_submitted` records payment batch id.
Node `emit_settlement_submitted`.

## Audit Events
Audit event class `TreasuryIntercompanyNettingCycleCreated`.
Audit event class `TreasuryIntercompanyNettingMemberAdded`.
Audit event class `TreasuryIntercompanyNettingObligationsImported`.
Audit event class `TreasuryIntercompanyNettingObligationExcluded`.
Audit event class `TreasuryIntercompanyNettingMatrixBuilt`.
Audit event class `TreasuryIntercompanyNettingApprovalRequested`.
Audit event class `TreasuryIntercompanyNettingApproved`.
Audit event class `TreasuryIntercompanyNettingSettlementSubmitted`.
Audit event class `TreasuryIntercompanyNettingCellSettled`.
Audit event class `TreasuryIntercompanyNettingPolicyDenied`.
Audit event class `TreasuryIntercompanyNettingSuperseded`.
Audit payload must include cycle id, cycle code, matrix version, evidence hash, and tenant id.
Audit payload for exclusions must include source obligation id and exclusion reason.
Audit payload for approvals must include legal entity scope and Cedar decision id.
Audit retention class is `TreasuryIntercompanyNetting`.
Audit ordering key is `tenant_id:cycle_code:matrix_version`.

## SLO Targets
p50 obligation import latency for 10000 obligations: 700 ms.
p95 obligation import latency for 10000 obligations: 2500 ms.
p99 obligation import latency for 10000 obligations: 5000 ms.
p50 matrix build latency for 10000 obligations and 100 members: 600 ms.
p95 matrix build latency for 10000 obligations and 100 members: 2200 ms.
p99 matrix build latency for 10000 obligations and 100 members: 4500 ms.
Throughput target: 60000 obligations imported per minute per cell.
Throughput target: 5000 matrix cells built per minute per cell.
Availability target for matrix read API: 99.99 percent monthly.
Availability target for import and build APIs: 99.95 percent monthly.
Rationale: netting cycles are periodic, but close windows require fast re-runs after late affiliate uploads.
Rationale: read availability is higher because payments and dashboards depend on approved matrices.
Rationale: import throughput must support enterprise tenants without forcing file splitting for normal cycles.

## Failure Modes + Recovery
Failure `SOURCE_OBLIGATION_DUPLICATE`: detect unique key conflict; recover by returning duplicate count and preserving first value.
Failure `UNKNOWN_LEGAL_ENTITY`: detect member lookup miss; recover by excluding obligation with blocking reason.
Failure `SANCTIONS_HIT`: detect compliance response; recover by excluding obligation and blocking settlement until compliance clears.
Failure `CAPITAL_CONTROL_BLOCK`: detect member flag or jurisdiction rule; recover by requiring controller approval or exclusion.
Failure `RATE_SET_INCOMPLETE`: detect missing currency pair; recover by blocking build until rates service supplies rate.
Failure `MATRIX_HASH_MISMATCH`: detect approval stale hash; recover by rebuilding or refreshing approval request.
Failure `PAYMENT_FACTORY_ROUTE_UNAVAILABLE`: detect preview failure; recover by building matrix without route hint and blocking settlement submission.
Failure `SETTLEMENT_BATCH_CREATE_FAILED`: detect IP-025 error; recover by retrying idempotent submit.
Failure `PARTIAL_CELL_SUBMISSION`: detect mismatch between submitted cells and payment batch instructions; recover by marking cycle Failed and alerting.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain error; recover by aborting status transition and retrying.
Recovery worker `treasury.intercompany_netting.settlement_reconcile` polls payment factory and updates cell status.
Runbook entry `runbooks/intercompany-netting-cycle-failure.md` should describe import, rebuild, approve, and submit recovery.

## Migration Notes
Source vendor surface: SAP FIN-FSCM-IHC netting center.
Source vendor surface: SAP TRM-TM settlement proposal.
Source vendor surface: SAP open-item extracts from AP and AR.
Source vendor surface: Kyriba intercompany netting module.
Source vendor surface: GTreasury intercompany netting worksheet.
Source vendor surface: Oracle intercompany balancing and settlement reports.
Migration maps SAP company code to `legal_entity_id`.
Migration maps SAP house bank to member settlement bank account.
Migration maps SAP open item document number to `source_obligation_id`.
Migration maps SAP clearing date to cycle business date when historical cycles are imported.
Migration preserves excluded obligations with their original reason if the source system exposes it.
Migration dry-run report lists legal entities without settlement bank accounts.
Migration dry-run report lists currency pairs without tenant-approved rate sets.
Migration acceptance requires two historical cycles to match gross and net totals by currency.

## Cross-microservice Handoffs
Handoff to `accounts-payable`: import affiliate payable obligations and source document metadata.
Handoff to `accounts-receivable`: import affiliate receivable obligations and settlement status.
Handoff to `legal-entity`: resolve entity hierarchy, region, and active status.
Handoff to `compliance`: screen counterparties, regions, and sanctions status.
Handoff to `rates`: resolve matrix rate set and base amounts.
Handoff to `payment-factory`: submit approved net settlement cells.
Handoff to `cash-position`: reflect submitted and settled netting payments.
Handoff to `risk`: expose affiliate counterparty exposure by pair and currency.
Handoff to `workflow`: run import, build, approve, and submit nodes.
Handoff to `audit-chain`: seal every matrix version and exclusion.

## Build Notes
Add database migration for cycles, members, obligations, cells, and approvals.
Add import parser interface for AP, AR, and legacy vendor extracts.
Add domain service `IntercompanyNettingMatrixBuilder`.
Add deterministic grouping sorted by payer, receiver, currency, and source obligation id.
Add Cedar entity schema for cycle and netting context.
Add REST handlers for create, member add, import, build, approve, submit, and read.
Add gRPC handlers for import summary, build, and matrix read.
Add contract tests for duplicate obligations and stale evidence hash approval.
Add workflow tests for blocked jurisdiction and capital-control approval.
Add load fixture with 100 members and 10000 obligations.
Add migration fixture with SAP IHC netting center export.
Add dashboard panels for import count, exclusion count, matrix build latency, and settlement status.
