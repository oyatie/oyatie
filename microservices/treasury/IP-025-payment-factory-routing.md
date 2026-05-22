---
doc_class: ImplementationPlan
ip_id: IP-025
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
sap_submodule_equivalents: [TRM-CM Payment Routing, TRM-TM Settlement Routing, TRM-RM Settlement Risk Controls]
---

# IP-025: Payment factory routing

## Intent
Implement payment factory routing that selects bank account, bank channel, format profile, approval path, and fallback route for treasury-originated payments.
The feature consumes payment instructions from cash sweeps, intercompany netting, treasury payments, and future settlement workflows.
The feature displaces SAP payment medium workbench routing, SAP BCM route release, and Kyriba-style payment factory route tables.
The implementation must produce a route decision before payment batch preparation and must persist decision evidence.
The implementation must not execute payment files directly; execution remains in IP-017 outbound payment and payments adapters.
The implementation must support route preview, route commit, route override approval, and fallback handling.
The implementation must use Cedar to guard route override, restricted channel use, and high-risk corridors.
The implementation must emit ADR-0263 audit events for route evaluation, commit, override, fallback, and deny.
The implementation must integrate with bank account hierarchy graph from IP-021 and workflow policy binding from IP-024.
The implementation must be deterministic when route rules, topology, and instruction inputs are unchanged.

## Context
Why: payment routing is where treasury policy, bank capability, cutoff windows, currency support, fees, and risk controls converge.
Why: SAP DMEE and BCM can route payments, but route logic is often spread across payment methods, house banks, workflow rules, and bank formats.
Why: Oyatie needs one payment factory route decision that is explainable and reusable by cash sweeps, netting, and treasury-originated payments.
Journey leg: j122 treasury operations routes EUR, XOF, USD, and SEK payments across bank channels while avoiding a missed cutoff and a restricted corridor.
Named persona: Aicha Diallo, Cash Operations Analyst at WAFRIA Energy, previews routes before releasing a 200-instruction supplier batch.
Supporting persona: Priya Raman, Treasury Shared Services Lead, approves fallback to a more expensive bank channel during cutoff pressure.
Pain point: route decisions today depend on tribal knowledge about bank cutoffs and currency corridors.
Pain point: route overrides are approved by email and not tied to the payment instruction evidence.
Pain point: fallback routes after bank outage are not evaluated against policy before use.
SAP parity: payment medium workbench, house bank determination, BCM release routing, TRM-TM settlement routing, and TRM-RM settlement risk controls.
Product outcome: every instruction has a selected route, rejected alternatives, policy decision, cutoff status, fee estimate, and fallback plan.
Non-goal: ISO 20022 serialization remains in IP-017.
Non-goal: transport adapters for SWIFT, EBICS, SFTP, and API-direct remain in payments.
Non-goal: bank account graph maintenance remains in IP-021.
Invariant: committed route decision is immutable for a payment instruction unless superseded by approved override.
Invariant: route preview never mutates payment instruction state.
Invariant: restricted corridors require Cedar allow before commit.
Invariant: a fallback route must satisfy the same validation contract as primary route.
Acceptance anchor: an intern can implement schema, route engine, APIs, policy checks, workflows, fixtures, and handoffs from this file.

## Data Model Deltas
Table `treasury.payment_route_rule`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `rule_code TEXT NOT NULL`.
Column `priority INTEGER NOT NULL`.
Column `active BOOLEAN NOT NULL DEFAULT true`.
Column `source_account_scope UUID[]`.
Column `legal_entity_scope UUID[]`.
Column `currency_scope CHAR(3)[]`.
Column `country_corridor_scope TEXT[]`.
Column `amount_min NUMERIC(22,4)`.
Column `amount_max NUMERIC(22,4)`.
Column `bank_channel_profile_id UUID NOT NULL`.
Column `format_variant TEXT NOT NULL`.
Column `fee_model_code TEXT`.
Column `cutoff_calendar_id UUID NOT NULL`.
Column `effective_from DATE NOT NULL`.
Column `effective_to DATE`.
Constraint `UNIQUE (tenant_id, rule_code, effective_from)`.
Index `ix_payment_route_rule_priority` on `(tenant_id, active, priority)`.
Table `treasury.payment_route_decision`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `instruction_source_type TEXT NOT NULL CHECK (instruction_source_type IN ('PaymentBatch','SweepMovement','IntercompanyNettingCell','DebtSettlement','ManualTreasuryPayment'))`.
Column `instruction_source_id UUID NOT NULL`.
Column `route_rule_id UUID REFERENCES treasury.payment_route_rule(id)`.
Column `selected_bank_account_id UUID NOT NULL`.
Column `selected_bank_channel_profile_id UUID NOT NULL`.
Column `selected_format_variant TEXT NOT NULL`.
Column `route_status TEXT NOT NULL CHECK (route_status IN ('Preview','Committed','Overridden','FallbackCommitted','Rejected'))`.
Column `amount NUMERIC(22,4) NOT NULL`.
Column `currency CHAR(3) NOT NULL`.
Column `destination_country_code CHAR(2)`.
Column `cutoff_at TIMESTAMPTZ NOT NULL`.
Column `cutoff_status TEXT NOT NULL CHECK (cutoff_status IN ('BeforeCutoff','MissedCutoff','NextDay','ManualReview'))`.
Column `estimated_fee_base NUMERIC(22,4)`.
Column `risk_score INTEGER NOT NULL`.
Column `evidence_hash TEXT NOT NULL`.
Column `cedar_decision_id UUID NOT NULL`.
Column `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
Constraint `UNIQUE (tenant_id, instruction_source_type, instruction_source_id, route_status) DEFERRABLE INITIALLY DEFERRED`.
Index `ix_payment_route_decision_source` on `(tenant_id, instruction_source_type, instruction_source_id)`.
Table `treasury.payment_route_alternative`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `decision_id UUID NOT NULL REFERENCES treasury.payment_route_decision(id)`.
Column `route_rule_id UUID REFERENCES treasury.payment_route_rule(id)`.
Column `bank_channel_profile_id UUID NOT NULL`.
Column `rank INTEGER NOT NULL`.
Column `eligible BOOLEAN NOT NULL`.
Column `rejection_reason TEXT`.
Column `estimated_fee_base NUMERIC(22,4)`.
Column `cutoff_at TIMESTAMPTZ`.
Column `risk_score INTEGER NOT NULL`.
Constraint `UNIQUE (decision_id, rank)`.
Table `treasury.payment_route_override`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `original_decision_id UUID NOT NULL REFERENCES treasury.payment_route_decision(id)`.
Column `override_decision_id UUID REFERENCES treasury.payment_route_decision(id)`.
Column `requested_by_principal_id UUID NOT NULL`.
Column `approved_by_principal_id UUID`.
Column `reason_code TEXT NOT NULL`.
Column `comment TEXT`.
Column `status TEXT NOT NULL CHECK (status IN ('Requested','Approved','Rejected','Applied'))`.
Column `cedar_decision_id UUID`.
Column `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
Column `approved_at TIMESTAMPTZ`.
Storage rule: route decisions are append-only; route override creates a new decision and links it.
Partitioning rule: decisions and alternatives partition by tenant cell and created month.
Retention rule: retain committed route decisions for ten years with payment evidence.

## API Endpoints
REST `POST /v1/treasury/payment-routes:preview`.
Request example:
```json
{
  "instruction_source_type": "IntercompanyNettingCell",
  "instruction_source_id": "52f6a801-1111-4e9e-9222-333344445555",
  "amount": "1250000.00",
  "currency": "EUR",
  "source_bank_account_id": "8b42a301-2222-4d2c-9333-444455556666",
  "destination_country_code": "FR",
  "requested_execution_date": "2026-05-20"
}
```
Response example:
```json
{
  "decision_id": "60b02c52-3333-4120-a444-555566667777",
  "route_status": "Preview",
  "selected_bank_channel_profile_id": "91f12e2d-4444-4a42-9555-666677778888",
  "selected_format_variant": "pain.001.001.09",
  "cutoff_status": "BeforeCutoff",
  "estimated_fee_base": "18.40",
  "risk_score": 22,
  "alternative_count": 3
}
```
REST `POST /v1/treasury/payment-routes/{decision_id}:commit`.
Commit response returns committed decision id, evidence hash, and Cedar decision id.
REST `POST /v1/treasury/payment-routes/{decision_id}/overrides`.
Override request includes target route rule id, reason code, and comment.
REST `POST /v1/treasury/payment-route-overrides/{override_id}/approve`.
REST `POST /v1/treasury/payment-routes/{decision_id}:fallback`.
Fallback request includes failed channel profile id and failure code.
REST `GET /v1/treasury/payment-routes/{decision_id}` returns decision, alternatives, override history, and audit refs.
gRPC `TreasuryPaymentFactoryRoutingService.PreviewRoute(PreviewRouteRequest) returns (PaymentRouteDecision)`.
gRPC `TreasuryPaymentFactoryRoutingService.CommitRoute(CommitRouteRequest) returns (PaymentRouteDecision)`.
Error `422 PAYMENT_ROUTE_NOT_FOUND` when no eligible route exists.
Error `403 PAYMENT_ROUTE_POLICY_DENIED` when Cedar blocks restricted route, override, or fallback.
Error `409 PAYMENT_ROUTE_ALREADY_COMMITTED` when source instruction already has committed decision.

## Cedar Policy Hooks
Principal shape: `UserOrService::{ id, tenant_id, roles, payment_route_scope, override_limit_base, restricted_corridor_scope }`.
Action `Action::"preview_payment_route"`.
Action `Action::"commit_payment_route"`.
Action `Action::"approve_payment_route_override"`.
Action `Action::"commit_payment_route_fallback"`.
Resource `PaymentRouteDecision::{ tenant_id, amount, currency, destination_country_code, selected_bank_channel_profile_id, risk_score, cutoff_status, route_status }`.
Context `PaymentRouteContext::{ now, source_type, source_account_risk_tier, corridor_restricted, bank_channel_health, expected_evidence_hash }`.
Permit payment factory service principal to preview and commit normal routes.
Permit treasury route supervisors to approve overrides within override limit.
Forbid commit when cutoff status is MissedCutoff unless fallback or next-day route is selected.
Forbid restricted corridor commit unless principal scope includes destination country code.
Forbid fallback when bank channel health is healthy and no failure code is provided.
Forbid override approval by the requester.
Emit `PaymentFactoryRoutePolicyDenied` for each deny.
Policy fixture `policy/payment-route-restricted-corridor-deny.json`.
Policy fixture `policy/payment-route-missed-cutoff-deny.json`.
Policy fixture `policy/payment-route-self-override-deny.json`.

## Ontology Projection
SAP house bank determination maps to `Oyatie::Treasury::PaymentRouteRule`.
SAP payment method and bank country rule maps to route rule scopes.
SAP BCM route release maps to `PaymentRouteDecision` and `PaymentRouteOverride`.
SAP DMEE format selection maps to `selected_format_variant`.
SAP TRM-TM settlement route maps to committed route decision.
Kyriba payment factory route maps to rule and decision rows.
GTreasury payment routing profile maps to route rule and alternatives.
Oracle payment process profile maps to route rule and format variant.
Ontology field `PaymentRouteRule.priority` maps from `priority`.
Ontology field `PaymentRouteRule.currencyScope` maps from `currency_scope`.
Ontology field `PaymentRouteDecision.selectedChannel` maps from `selected_bank_channel_profile_id`.
Ontology field `PaymentRouteDecision.cutoffStatus` maps from `cutoff_status`.
Ontology field `PaymentRouteDecision.riskScore` maps from `risk_score`.
Ontology field `PaymentRouteAlternative.rejectionReason` maps from `rejection_reason`.
Ontology field `PaymentRouteOverride.reasonCode` maps from `reason_code`.
Ontology edge `ROUTE_DECISION_SELECTED_RULE` connects decision to rule.
Ontology edge `ROUTE_DECISION_HAS_ALTERNATIVE` connects decision to alternatives.
Ontology edge `ROUTE_DECISION_OVERRIDDEN_BY` connects original to override decision.
Ontology edge `ROUTE_DECISION_FOR_PAYMENT_SOURCE` connects decision to instruction source.
Projection must include rejected alternatives for audit users and hide fee models from unauthorized readers.

## Workflow Steps
Workflow `treasury.payment_factory.route_preview`.
Node `load_instruction_source` reads payment batch, sweep movement, netting cell, debt settlement, or manual payment.
Node `load_account_topology` queries IP-021 for source account, legal entity, risk tier, and route eligibility.
Node `load_active_route_rules` filters by date, currency, corridor, amount, and legal entity.
Node `evaluate_cutoffs` calculates cutoff status using rule calendar and timezone.
Node `score_route_risk` scores corridor, amount, channel, account risk, and bank health.
Node `estimate_route_fee` applies fee model when configured.
Node `rank_eligible_routes` sorts by eligibility, risk score, cutoff, fee, and priority.
Node `cedar_preview_check` verifies preview permission.
Node `persist_preview_decision` stores selected route and alternatives.
Node `emit_route_previewed`.
Branch `no_eligible_route` marks rejected decision and returns structured reasons.
Workflow `treasury.payment_factory.route_commit`.
Node `reload_preview_decision`.
Node `cedar_commit_check`.
Node `compare_evidence_hash`.
Node `persist_committed_decision`.
Node `emit_route_committed`.
Workflow `treasury.payment_factory.route_override`.
Node `create_override_request`.
Node `cedar_override_approval_check`.
Node `build_override_decision`.
Node `mark_override_applied`.
Node `emit_route_overridden`.
Workflow `treasury.payment_factory.route_fallback`.
Node `capture_channel_failure`.
Node `exclude_failed_channel`.
Node `rerun_route_preview_for_fallback`.
Node `cedar_fallback_commit_check`.
Node `persist_fallback_committed_decision`.
Node `emit_route_fallback_committed`.

## Audit Events
Audit event class `TreasuryPaymentFactoryRoutePreviewRequested`.
Audit event class `TreasuryPaymentFactoryRoutePreviewed`.
Audit event class `TreasuryPaymentFactoryRouteRejected`.
Audit event class `TreasuryPaymentFactoryRouteCommitted`.
Audit event class `TreasuryPaymentFactoryRouteOverrideRequested`.
Audit event class `TreasuryPaymentFactoryRouteOverrideApproved`.
Audit event class `TreasuryPaymentFactoryRouteOverrideRejected`.
Audit event class `TreasuryPaymentFactoryRouteFallbackCommitted`.
Audit event class `TreasuryPaymentFactoryRoutePolicyDenied`.
Audit event class `TreasuryPaymentFactoryRouteRuleChanged`.
Audit payload must include tenant id, decision id, source type, source id, selected channel, format variant, cutoff status, and evidence hash.
Audit payload for alternatives must include rejection reason counts and selected rank.
Audit payload for overrides must include requester, approver, reason code, and original decision id.
Audit payload for policy denies must include Cedar decision id and denied action.
Audit retention class is `TreasuryPaymentRoutingEvidence`.
Audit ordering key is `tenant_id:instruction_source_type:instruction_source_id`.

## SLO Targets
p50 route preview latency with 100 active rules: 70 ms.
p95 route preview latency with 100 active rules: 220 ms.
p99 route preview latency with 100 active rules: 500 ms.
p50 route commit latency: 50 ms.
p95 route commit latency: 180 ms.
p99 route commit latency: 400 ms.
p50 fallback route latency: 90 ms.
p95 fallback route latency: 300 ms.
p99 fallback route latency: 700 ms.
Throughput target: 20000 route previews per minute per cell.
Throughput target: 10000 route commits per minute per cell.
Availability target for route preview API: 99.99 percent monthly.
Availability target for route commit API: 99.99 percent monthly.
Rationale: payment preparation calls routing for every instruction and cannot tolerate batch-scale latency.
Rationale: route commit is on payment release critical path and must stay highly available.
Rationale: fallback routing must be fast during bank outage incidents.

## Failure Modes + Recovery
Failure `NO_ELIGIBLE_ROUTE`: detect empty eligible route list; recover by route rule update or manual payment exception.
Failure `CUTOFF_MISSED`: detect cutoff status; recover by selecting next-day or fallback channel with approval.
Failure `RESTRICTED_CORRIDOR_DENIED`: detect Cedar deny; recover by restricted-corridor approver or different route.
Failure `BANK_CHANNEL_UNHEALTHY`: detect health status; recover by fallback route excluding failed channel.
Failure `ROUTE_ALREADY_COMMITTED`: detect source uniqueness; recover by reading existing committed decision.
Failure `OVERRIDE_SELF_APPROVAL`: detect requester equals approver; recover by assigning independent approver.
Failure `TOPOLOGY_ACCOUNT_INELIGIBLE`: detect IP-021 route eligibility false; recover by graph governance update.
Failure `FEE_MODEL_MISSING`: detect missing optional fee model; recover by ranking without fee and warning.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain error; recover by aborting commit or override transition.
Failure `PARTIAL_ALTERNATIVE_WRITE`: prevent with transaction; repair by marking preview Rejected and recomputing.
Recovery worker `treasury.payment_factory.route_health_reconcile` rechecks channel health and marks fallback opportunities.
Runbook entry `runbooks/payment-factory-routing-failure.md` should cover no route, cutoff miss, bank outage, and override.

## Migration Notes
Source vendor surface: SAP house bank determination.
Source vendor surface: SAP payment method configuration.
Source vendor surface: SAP DMEE format tree assignment.
Source vendor surface: SAP BCM approval and route release.
Source vendor surface: SAP TRM-TM settlement route configuration.
Source vendor surface: Kyriba payment factory route tables.
Source vendor surface: GTreasury payment routing profiles.
Source vendor surface: Oracle payment process profiles.
Migration maps SAP company code and house bank to source account scope.
Migration maps SAP payment method to format variant and channel profile.
Migration maps SAP bank country and payment currency rules to corridor and currency scopes.
Migration maps SAP amount limits to amount min and max.
Migration imports route priority from explicit source priority where available or deterministic order by specificity.
Migration dry-run report lists route rules without bank channel profile.
Migration dry-run report lists route rules referencing accounts absent from IP-021 graph.
Migration acceptance requires historical payment batches to select same bank channel for at least configured tenant tolerance.

## Cross-microservice Handoffs
Handoff to `bank-account-graph`: read topology, route eligibility, risk tier, legal entity, and signer context.
Handoff to `payments`: provide committed route decisions for payment batch preparation and submission.
Handoff to `cash-pooling`: route sweep movement payments.
Handoff to `intercompany-netting`: route net settlement cells.
Handoff to `debt`: route debt settlement payments.
Handoff to `workflow`: orchestrate preview, commit, override, and fallback nodes.
Handoff to `policy`: evaluate restricted route, override, and fallback Cedar checks.
Handoff to `calendar`: compute bank cutoff and next business day.
Handoff to `audit-chain`: seal route decision, alternatives, overrides, and denies.
Handoff to `ops-dashboard`: expose route rejection, cutoff misses, channel health, and override rate.

## Build Notes
Add database migration for route rules, decisions, alternatives, and overrides.
Add domain service `PaymentFactoryRouteEngine`.
Add deterministic route scorer with tests for priority, cutoff, fee, and risk order.
Add Cedar schema for route decision and route context.
Add REST handlers for preview, commit, override request, override approve, fallback, and read.
Add gRPC handlers for preview and commit.
Add workflow adapters for IP-016 sweep, IP-019 netting, and IP-017 payment batch sources.
Add contract tests for no eligible route, restricted corridor, cutoff miss, and duplicate commit.
Add workflow tests for fallback after unhealthy bank channel.
Add load fixture with 100 active rules and 20000 route previews per minute target.
Add migration fixture with SAP house bank determination and payment method export.
Add dashboard panels for preview latency, commit latency, no-route count, cutoff miss count, and override approvals.
