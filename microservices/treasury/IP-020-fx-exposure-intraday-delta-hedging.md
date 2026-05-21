---
doc_class: ImplementationPlan
ip_id: IP-020
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
sap_submodule_equivalents: [TRM-RM Market Risk Analyzer, TRM-TM FX Transaction Management, TRM-CM Liquidity Exposure]
---

# IP-020: FX exposure intraday delta hedging

## Intent
Implement intraday FX exposure delta detection and hedge recommendation generation for treasury risk managers.
The feature compares current exposures against approved hedge policy bands and produces proposed hedge tickets.
The feature displaces SAP TRM-RM market risk analyzer worklists and selected TRM-TM FX deal initiation surfaces.
The feature is advisory until an approved principal converts recommendations into hedge transactions.
The implementation must preserve evidence for every exposure input, rate snapshot, policy band, recommendation, approval, and rejection.
The implementation must not auto-trade or connect directly to dealing platforms.
The implementation must support REST for dashboards and gRPC for workflow and risk services.
The implementation must keep calculations deterministic within a rate snapshot and exposure watermark.
The implementation must reject stale rate snapshots rather than silently using old prices.
The implementation must emit ADR-0263 audit event classes for risk evidence.

## Context
Why: treasury teams need intraday visibility when invoices, purchase orders, payroll, cash positions, and market rates move faster than daily hedge reports.
Why: SAP TRM-RM market risk analyzer often runs batch analyses that do not explain the source delta in a form an intern can wire to workflow.
Why: Oyatie needs a hedge recommendation service that is policy-bound and auditable before dealing-system integration.
Journey leg: j120 treasury risk manager reacts to a sudden KRW and EUR exposure swing caused by supplier payment acceleration.
Named persona: Elena Fischer, FX Risk Manager at HelioMed Devices, monitors net EUR, KRW, and USD exposures during European trading hours.
Supporting persona: Omar Haddad, Treasury Dealer, converts approved recommendations into forward deal requests.
Pain point: current exposure spreadsheets mix booked invoices, forecast purchase orders, and cash positions with unclear cutoffs.
Pain point: hedge bands are stored in policy documents but not enforced by system workflow.
Pain point: rate snapshots arrive intraday and must be tied to the exact recommendation.
SAP parity: TRM-RM market risk analyzer, value-at-risk inputs, exposure management, and TRM-TM FX forward transaction initiation.
Product outcome: risk manager sees delta, policy breach, proposed hedge amount, recommended tenor, and approval path.
Non-goal: derivative valuation and accounting effectiveness testing are handled by hedge designation services.
Non-goal: market data vendor ingestion is owned by rates.
Non-goal: trade execution is owned by treasury transaction management or external dealer adapters.
Invariant: every recommendation references exactly one exposure snapshot and one rate snapshot.
Invariant: no recommendation can be approved when source exposure watermark is stale.
Invariant: no hedge ticket can exceed policy ceiling without Cedar break-glass approval.
Invariant: recommendations are immutable after approval or rejection.
Acceptance anchor: an intern can implement schema, calculation service, policy tests, APIs, and workflow nodes from this file.

## Data Model Deltas
Table `treasury.fx_exposure_snapshot`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `as_of TIMESTAMPTZ NOT NULL`.
Column `source_watermark TIMESTAMPTZ NOT NULL`.
Column `base_currency CHAR(3) NOT NULL`.
Column `rate_snapshot_id UUID NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Open','Superseded','Failed'))`.
Column `gross_exposure_base NUMERIC(22,4) NOT NULL`.
Column `net_exposure_base NUMERIC(22,4) NOT NULL`.
Column `source_event_count INTEGER NOT NULL`.
Column `evidence_hash TEXT NOT NULL`.
Constraint `UNIQUE (tenant_id, as_of, rate_snapshot_id)`.
Index `ix_fx_exposure_snapshot_tenant_time` on `(tenant_id, as_of DESC)`.
Table `treasury.fx_exposure_bucket`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `snapshot_id UUID NOT NULL REFERENCES treasury.fx_exposure_snapshot(id)`.
Column `currency CHAR(3) NOT NULL`.
Column `tenor_bucket TEXT NOT NULL CHECK (tenor_bucket IN ('Spot','0-7D','8-30D','31-90D','91-180D','181-365D','365D+'))`.
Column `legal_entity_id UUID`.
Column `business_unit_id UUID`.
Column `source_type TEXT NOT NULL CHECK (source_type IN ('Cash','Payable','Receivable','Forecast','Debt','Hedge'))`.
Column `long_amount NUMERIC(22,4) NOT NULL DEFAULT 0`.
Column `short_amount NUMERIC(22,4) NOT NULL DEFAULT 0`.
Column `net_amount NUMERIC(22,4) NOT NULL`.
Column `base_amount NUMERIC(22,4) NOT NULL`.
Column `source_event_ids UUID[] NOT NULL`.
Constraint `UNIQUE (snapshot_id, currency, tenor_bucket, legal_entity_id, business_unit_id, source_type)`.
Table `treasury.fx_hedge_policy_band`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `currency CHAR(3) NOT NULL`.
Column `tenor_bucket TEXT NOT NULL`.
Column `minimum_hedge_ratio NUMERIC(8,6) NOT NULL`.
Column `maximum_hedge_ratio NUMERIC(8,6) NOT NULL`.
Column `target_hedge_ratio NUMERIC(8,6) NOT NULL`.
Column `max_single_ticket_base NUMERIC(22,4) NOT NULL`.
Column `allowed_instruments TEXT[] NOT NULL`.
Column `effective_from DATE NOT NULL`.
Column `effective_to DATE`.
Constraint `UNIQUE (tenant_id, currency, tenor_bucket, effective_from)`.
Table `treasury.fx_delta_hedge_recommendation`.
Column `id UUID PRIMARY KEY DEFAULT gen_random_uuid()`.
Column `tenant_id UUID NOT NULL`.
Column `snapshot_id UUID NOT NULL REFERENCES treasury.fx_exposure_snapshot(id)`.
Column `policy_band_id UUID NOT NULL REFERENCES treasury.fx_hedge_policy_band(id)`.
Column `currency CHAR(3) NOT NULL`.
Column `tenor_bucket TEXT NOT NULL`.
Column `current_hedge_ratio NUMERIC(8,6) NOT NULL`.
Column `target_hedge_ratio NUMERIC(8,6) NOT NULL`.
Column `recommended_direction TEXT NOT NULL CHECK (recommended_direction IN ('Buy','Sell','NoAction'))`.
Column `recommended_amount NUMERIC(22,4) NOT NULL`.
Column `recommended_instrument TEXT NOT NULL`.
Column `base_amount NUMERIC(22,4) NOT NULL`.
Column `status TEXT NOT NULL CHECK (status IN ('Open','PendingApproval','Approved','Rejected','Converted'))`.
Column `reason_code TEXT NOT NULL`.
Column `cedar_decision_id UUID`.
Column `approved_by_principal_id UUID`.
Column `approved_at TIMESTAMPTZ`.
Column `converted_transaction_id UUID`.
Storage rule: recommendations are append-only and immutable after status Approved, Rejected, or Converted.
Partitioning rule: exposure buckets and recommendations partition by tenant cell and as-of day.
Retention rule: retain snapshots, buckets, and recommendations for seven years.

## API Endpoints
REST `POST /v1/treasury/fx-exposure/snapshots`.
Request example:
```json
{
  "as_of": "2026-05-20T13:45:00Z",
  "base_currency": "USD",
  "rate_snapshot_id": "f4d191a0-aaaa-4b2b-9f19-111122223333",
  "idempotency_key": "heliomed:fx:2026-05-20T1345Z"
}
```
Response example:
```json
{
  "snapshot_id": "7df5cf20-bbbb-42fd-9010-333344445555",
  "status": "Open",
  "bucket_count": 42,
  "net_exposure_base": "18450000.00",
  "evidence_hash": "sha256:f203..."
}
```
REST `POST /v1/treasury/fx-exposure/snapshots/{snapshot_id}/recommendations:build`.
Recommendation response returns recommendation count, no-action count, breach count, and evidence hash.
REST `GET /v1/treasury/fx-exposure/snapshots/{snapshot_id}/buckets`.
REST `GET /v1/treasury/fx-hedge-recommendations?status=Open&currency=EUR`.
REST `POST /v1/treasury/fx-hedge-recommendations/{recommendation_id}/approve`.
Approve request includes `expected_snapshot_hash`, `approval_comment`, and optional `override_reason`.
REST `POST /v1/treasury/fx-hedge-recommendations/{recommendation_id}/reject`.
REST `POST /v1/treasury/fx-hedge-recommendations/{recommendation_id}/convert-to-ticket`.
Convert response returns `fx_transaction_request_id` and status `Converted`.
gRPC `TreasuryFxExposureService.BuildSnapshot(BuildFxExposureSnapshotRequest) returns (FxExposureSnapshot)`.
gRPC `TreasuryFxExposureService.BuildRecommendations(BuildFxRecommendationsRequest) returns (FxRecommendationSet)`.
Error `412 RATE_SNAPSHOT_STALE` when rate snapshot age exceeds tenant policy.
Error `409 SNAPSHOT_SUPERSEDED` when building recommendations from old snapshot.
Error `403 FX_HEDGE_POLICY_DENIED` when Cedar blocks approval or conversion.

## Cedar Policy Hooks
Principal shape: `User::{ id, tenant_id, roles, hedge_approval_limit_base, allowed_currencies, legal_entity_scope }`.
Action `Action::"build_fx_exposure_snapshot"`.
Action `Action::"approve_fx_delta_hedge_recommendation"`.
Action `Action::"convert_fx_recommendation_to_ticket"`.
Resource `FxDeltaHedgeRecommendation::{ tenant_id, currency, base_amount, recommended_instrument, status, legal_entity_ids }`.
Context `FxHedgeContext::{ now, rate_snapshot_age_seconds, expected_snapshot_hash, market_open, policy_override_reason }`.
Permit risk managers to build snapshots for their tenant.
Permit hedge approvers to approve recommendations within allowed currencies and approval limit.
Forbid approval when rate snapshot age is greater than 900 seconds.
Forbid approval when context expected snapshot hash differs from snapshot evidence hash.
Forbid conversion unless recommendation status is Approved.
Forbid conversion outside market hours unless principal has role `treasury-fx-after-hours-approver`.
Forbid instruments not present in policy band `allowed_instruments`.
Emit `FxDeltaHedgePolicyDenied` for each deny.
Policy fixture `policy/fx-delta-hedge-rate-stale-deny.json`.
Policy fixture `policy/fx-delta-hedge-limit-deny.json`.
Policy fixture `policy/fx-delta-hedge-after-hours-deny.json`.

## Ontology Projection
SAP TRM-RM exposure position maps to `Oyatie::Treasury::FxExposureSnapshot`.
SAP exposure bucket maps to `Oyatie::Treasury::FxExposureBucket`.
SAP hedge policy rule maps to `Oyatie::Treasury::FxHedgePolicyBand`.
SAP TRM-TM FX deal request maps to converted transaction ticket.
Kyriba FX exposure worksheet maps to exposure buckets.
GTreasury hedge recommendation maps to `FxDeltaHedgeRecommendation`.
FIS Quantum currency exposure maps to snapshot and bucket rows.
Ontology field `FxExposureSnapshot.asOf` maps from `as_of`.
Ontology field `FxExposureSnapshot.rateSnapshot` maps from `rate_snapshot_id`.
Ontology field `FxExposureSnapshot.netExposureBase` maps from `net_exposure_base`.
Ontology field `FxExposureBucket.currency` maps from `currency`.
Ontology field `FxExposureBucket.tenor` maps from `tenor_bucket`.
Ontology field `FxExposureBucket.netAmount` maps from `net_amount`.
Ontology field `FxHedgeRecommendation.direction` maps from `recommended_direction`.
Ontology field `FxHedgeRecommendation.amount` maps from `recommended_amount`.
Ontology field `FxHedgeRecommendation.instrument` maps from `recommended_instrument`.
Ontology edge `SNAPSHOT_HAS_BUCKET` connects snapshot to bucket.
Ontology edge `BUCKET_EVALUATED_BY_POLICY_BAND` connects bucket to policy band.
Ontology edge `RECOMMENDATION_DERIVED_FROM_SNAPSHOT` connects recommendation to snapshot.
Ontology edge `RECOMMENDATION_CONVERTED_TO_TRANSACTION` connects recommendation to FX transaction request.

## Workflow Steps
Workflow `treasury.fx_exposure.build_snapshot`.
Node `load_rate_snapshot` validates freshness and supported pairs.
Node `load_cash_positions` consumes approved rollups and intraday account balances.
Node `load_payables_receivables` consumes AP and AR open items by due date.
Node `load_forecasts` consumes approved liquidity forecasts.
Node `load_existing_hedges` consumes active hedge designation and transaction records.
Node `bucket_by_currency_tenor_entity` groups by currency, tenor, entity, business unit, and source type.
Node `compute_net_exposure` calculates long, short, net, and base amounts.
Node `compute_snapshot_evidence_hash` hashes source ids, rates, and bucket values.
Node `persist_snapshot_and_buckets` writes snapshot and buckets in one transaction.
Node `emit_snapshot_built` publishes audit-chain event.
Workflow `treasury.fx_exposure.build_recommendations`.
Node `load_policy_bands` selects effective bands for snapshot as-of date.
Node `calculate_current_hedge_ratio` compares existing hedges to gross exposure.
Node `detect_policy_breach` branches to no-action, under-hedged, or over-hedged.
Node `size_recommendation_to_target` caps by max single ticket base.
Node `select_instrument` chooses first tenant-approved instrument by tenor.
Node `persist_recommendations` writes open recommendations.
Node `emit_recommendations_built`.
Branch `no_policy_band` creates blocking exception and no recommendation.
Branch `no_action_within_band` records no-action count and evidence.
Workflow `treasury.fx_exposure.approve_and_convert`.
Node `cedar_approval_check`.
Node `mark_recommendation_approved`.
Node `create_fx_transaction_request`.
Node `mark_converted`.
Node `emit_recommendation_converted`.

## Audit Events
Audit event class `TreasuryFxExposureSnapshotRequested`.
Audit event class `TreasuryFxExposureSnapshotBuilt`.
Audit event class `TreasuryFxExposureSnapshotFailed`.
Audit event class `TreasuryFxDeltaHedgeRecommendationsBuilt`.
Audit event class `TreasuryFxDeltaHedgeRecommendationOpened`.
Audit event class `TreasuryFxDeltaHedgeRecommendationApproved`.
Audit event class `TreasuryFxDeltaHedgeRecommendationRejected`.
Audit event class `TreasuryFxDeltaHedgeRecommendationConverted`.
Audit event class `TreasuryFxDeltaHedgePolicyDenied`.
Audit event class `TreasuryFxRateSnapshotStaleRejected`.
Audit payload must include tenant id, snapshot id, rate snapshot id, as-of, and evidence hash.
Audit payload for recommendation events must include currency, tenor bucket, direction, amount, base amount, and policy band id.
Audit payload for approval events must include Cedar decision id and approver principal id.
Audit retention class is `TreasuryFxRiskEvidence`.
Audit ordering key is `tenant_id:as_of:rate_snapshot_id`.

## SLO Targets
p50 snapshot build latency for 50000 source events: 900 ms.
p95 snapshot build latency for 50000 source events: 3000 ms.
p99 snapshot build latency for 50000 source events: 6000 ms.
p50 recommendation build latency for 200 buckets: 120 ms.
p95 recommendation build latency for 200 buckets: 500 ms.
p99 recommendation build latency for 200 buckets: 900 ms.
Throughput target: 30 full snapshots per minute per tenant cell.
Throughput target: 1000 recommendations built per minute per tenant cell.
Availability target for exposure read API: 99.99 percent monthly.
Availability target for snapshot build API: 99.95 percent monthly.
Rate freshness target: recommendations use snapshots no older than 15 minutes unless policy says shorter.
Rationale: intraday risk response needs sub-minute rebuilds after large source events.
Rationale: recommendation build is light and should not be the bottleneck.
Rationale: read availability is high because risk dashboards and approval queues depend on it.

## Failure Modes + Recovery
Failure `RATE_SNAPSHOT_STALE`: detect age above policy; recover by requesting fresh rates and retrying.
Failure `MISSING_RATE_PAIR`: detect unresolved currency pair; recover by excluding affected bucket and raising blocking exception.
Failure `SOURCE_WATERMARK_STALE`: detect AP, AR, cash, or hedge source lag; recover by retrying after source catches up.
Failure `POLICY_BAND_MISSING`: detect no effective band; recover by routing to treasury policy admin.
Failure `RECOMMENDATION_EXCEEDS_LIMIT`: detect amount above max ticket; recover by splitting into multiple recommendations or requiring override.
Failure `APPROVAL_HASH_MISMATCH`: detect stale expected hash; recover by refreshing recommendation view.
Failure `MARKET_CLOSED`: detect context market open false; recover by waiting or using after-hours approver.
Failure `FX_TICKET_CREATE_FAILED`: detect transaction service error; recover with idempotent convert retry.
Failure `AUDIT_APPEND_FAILED`: detect audit-chain failure; recover by aborting status transition and retrying.
Failure `PARTIAL_BUCKET_WRITE`: prevent with transaction; if integrity scan finds it, mark snapshot Failed.
Recovery worker `treasury.fx_exposure.recommendation_reconcile` checks converted tickets and marks status.
Runbook entry `runbooks/fx-exposure-delta-hedging-failure.md` should describe stale rates, policy gaps, and conversion retries.

## Migration Notes
Source vendor surface: SAP TRM-RM Market Risk Analyzer exposure positions.
Source vendor surface: SAP TRM-TM FX forward and swap transactions.
Source vendor surface: SAP cash management forecast flows.
Source vendor surface: Kyriba FX exposure and hedge recommendation worksheets.
Source vendor surface: GTreasury FX exposure management.
Source vendor surface: FIS Quantum exposure reports.
Migration maps SAP risk category to source type.
Migration maps SAP planning level and date bucket to tenor bucket.
Migration maps SAP transaction type to existing hedge source events.
Migration maps SAP hedge policy documents to policy band rows.
Migration must import historical recommendations as `legacy-import` status Rejected or Converted based on evidence.
Migration dry-run report lists currencies without policy bands.
Migration dry-run report lists exposures that cannot resolve source legal entity.
Migration acceptance requires one month of exposure snapshots matching legacy totals by currency and tenor.

## Cross-microservice Handoffs
Handoff to `rates`: consume rate snapshots and currency pair metadata.
Handoff to `cash-position`: consume approved daily rollups and intraday account balances.
Handoff to `accounts-payable`: consume open payables and supplier currency exposure.
Handoff to `accounts-receivable`: consume open receivables and customer currency exposure.
Handoff to `liquidity-forecast`: consume approved forecasted cash flows.
Handoff to `hedge-designation`: consume existing hedges and convert approved recommendations.
Handoff to `workflow`: orchestrate snapshot, recommendation, approval, and conversion flows.
Handoff to `ontology`: project exposure graph and recommendation edges.
Handoff to `audit-chain`: seal snapshot, recommendation, approval, and policy deny events.
Handoff to `ops-dashboard`: expose freshness, breach count, and recommendation queue metrics.

## Build Notes
Add database migration for snapshots, buckets, policy bands, and recommendations.
Add domain service `FxExposureSnapshotBuilder`.
Add domain service `FxDeltaHedgeRecommendationBuilder`.
Add deterministic tenor bucketing utility with unit tests.
Add Cedar schema for recommendation resource and FX hedge context.
Add REST handlers for snapshot build, recommendation build, read, approve, reject, and convert.
Add gRPC handlers for snapshot and recommendation build.
Add contract tests for stale rate, stale hash approval, and conversion idempotency.
Add workflow tests for under-hedged, over-hedged, and no-action branches.
Add load fixture with 50000 source events and 200 buckets.
Add migration fixture with SAP TRM-RM exposure export.
Add dashboard panels for rate age, exposure build latency, breach count, and converted recommendations.
