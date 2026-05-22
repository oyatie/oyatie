---
doc_class: ImplementationPlan
ip_id: IP-016
microservice: supply-chain-planning
related_adrs: [ADR-0105, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0131, ADR-0132]
journey_id: j123-multi-tenant-coordinated-product-launch
journey_link: docs/user-journeys/j123-multi-tenant-coordinated-product-launch/story.md
status: Accepted
date: 2026-05-20
owner: axis-supply-chain-planning + axis-erp-parity
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [IBP-DP demand sensing, IBP-DP statistical forecasting, SAP Fiori demand planning alerts]
---

# IP-016: Demand sensing ML signal joiner

## 1. Context with why, journey leg, named persona

This IP defines the intern-buildable demand sensing joiner for short-horizon
signals that modify the supply-chain-planning demand plan without bypassing
tenant policy or audit-chain evidence.

Why this matters: planners need near-real-time sell-through, order, weather,
search, marketplace, and promotion signals joined into one explainable demand
signal before the forecast release window closes.

Journey leg: j123 leg 03, "sense launch-demand shocks, explain drivers, and stage planner review".

Named persona: Mira, regional demand planner for Korea consumer electronics.

Mira starts with an approved weekly demand plan and needs a controlled way to
see ML signal lift before committing changes to IBP-DP style planning views.

The joiner is not the forecasting model; it is the governed signal assembly
surface that makes model inputs explainable and replayable.

SAP equivalent: IBP-DP demand sensing signal ingestion and alert worklists.

Oracle equivalent: Demand Management sensing collection with forecast override
workbench.

Microsoft equivalent: Dynamics 365 demand forecasting demand signal enrichment.

The implementation must keep raw external signals separate from joined demand
signals so residency and source rights are enforceable.

The feature belongs in the usecase and adapter bands from ADR-0105.

The feature emits ADR-0263 observability and audit link fields for every joined
signal set.

The feature uses ADR-0244 tenant scoping on every source row and joined result.

The feature uses ADR-0253 transport rules for inbound signal feeds and gRPC
handoffs.

The feature inherits ADR-0315 evidence requirements for ERP parity claims.

## 2. Scope

In scope: demand-signal source registration.

In scope: normalized signal staging tables.

In scope: join-plan configuration for product, location, customer, and day.

In scope: model-output score ingestion from approved ML services.

In scope: explainability payloads for planners and auditors.

In scope: write staging to demand plan modifiers.

Out of scope: training the ML model.

Out of scope: autonomous commit to the demand plan baseline.

Out of scope: marketplace settlement or ad campaign billing.

Out of scope: replacing the canonical demand-plan aggregate.

## 3. Data Model Deltas

Create table scp_demand_signal_source.

Column tenant_id: uuid, required, partition key.

Column signal_source_id: uuid, required, stable identifier.

Column source_system_id: text, required, maps to SourceSystemRecord.

Column source_kind: enum, values pos_sell_through, ecommerce_order, weather,
search_trend, promotion_calendar, marketplace_inventory, manual_adjustment.

Column source_rights_class: enum, values owned, processor, licensed, public.

Column residency_pack: text, required.

Column freshness_slo_seconds: integer, required.

Column enabled: boolean, required.

Column created_by_principal_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_demand_signal_observation.

Column tenant_id: uuid, required, partition key.

Column observation_id: uuid, required.

Column signal_source_id: uuid, required.

Column product_id: text, required.

Column location_id: text, required.

Column customer_segment_id: text, nullable.

Column demand_date: date, required.

Column signal_name: text, required.

Column signal_value: numeric(18,6), required.

Column signal_unit: text, required.

Column confidence_score: numeric(6,5), required.

Column source_event_time: timestamptz, required.

Column ingestion_time: timestamptz, required.

Column evidence_hash: text, required.

Create table scp_demand_signal_join_run.

Column tenant_id: uuid, required, partition key.

Column join_run_id: uuid, required.

Column demand_plan_id: uuid, required.

Column planning_horizon_start: date, required.

Column planning_horizon_end: date, required.

Column join_key_strategy: enum, values sku_dc_day, sku_market_day,
sku_customer_day.

Column model_score_version: text, required.

Column policy_bundle_version: text, required.

Column status: enum, values staged, scored, rejected, expired, promoted.

Column audit_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_demand_signal_join_result.

Column tenant_id: uuid, required, partition key.

Column join_result_id: uuid, required.

Column join_run_id: uuid, required.

Column product_id: text, required.

Column location_id: text, required.

Column demand_date: date, required.

Column baseline_quantity: numeric(18,3), required.

Column sensed_delta_quantity: numeric(18,3), required.

Column sensed_delta_percent: numeric(9,6), required.

Column confidence_score: numeric(6,5), required.

Column driver_summary_json: jsonb, required.

Column ontology_projection_id: uuid, required.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/demand-sensing/join-runs

Request field tenant_id: tenant UUID from principal context.

Request field demand_plan_id: approved plan snapshot to sense against.

Request field horizon: start_date and end_date.

Request field join_key_strategy: sku_dc_day by default.

Request field source_filter: optional list of signal_source_id values.

Request field explanation_level: compact, planner, or audit.

Example request:

```json
{
  "tenant_id": "11111111-1111-1111-1111-111111111111",
  "demand_plan_id": "dp-2026-kr-w21",
  "horizon": {"start_date": "2026-05-21", "end_date": "2026-06-04"},
  "join_key_strategy": "sku_dc_day",
  "source_filter": ["pos-kr-retail", "weather-kr"],
  "explanation_level": "planner"
}
```

Example response:

```json
{
  "join_run_id": "jsr-016-0001",
  "status": "staged",
  "joined_rows": 18420,
  "material_delta_rows": 612,
  "audit_id": "aud-scp-demand-sensing-016",
  "next_action": "review_demand_signal_delta"
}
```

gRPC DemandSensingSignalJoiner.CreateJoinRun accepts CreateJoinRunRequest.

gRPC DemandSensingSignalJoiner.GetJoinRun returns joined row counts and audit id.

gRPC DemandSensingSignalJoiner.StreamJoinResults streams result pages by key.

Async event scp.demand_sensing.join_run.staged publishes join_run_id and audit_id.

Idempotency key: tenant_id plus demand_plan_id plus horizon plus source_filter_hash.

## 5. Cedar Policy Hooks

Principal type: SupplyChainPlanner.

Action: scp::Action::"CreateDemandSignalJoinRun".

Resource: scp::DemandPlan::"<tenant_id>/<demand_plan_id>".

Context tenant_id must equal principal.tenant_id.

Context residency_pack must be in principal.allowed_residency_packs.

Context source_rights_class cannot be licensed unless principal has
licensed_signal_access.

Context explanation_level audit requires principal.role in ComplianceAuditor or
PlanningAdmin.

Default deny applies when source_filter includes disabled signal sources.

Cedar decision id is written to scp_demand_signal_join_run.policy_decision_id.

Policy denial emits DemandSensingSignalJoinPolicyDenied.

## 6. Ontology Projection Field Mapping

DemandSignalJoinRun.tenant_id maps to Tenant.id.

DemandSignalJoinRun.demand_plan_id maps to DemandPlan.id.

DemandSignalJoinRun.created_by_principal_id maps to Principal.id.

DemandSignalJoinRun.source_system_id maps to SourceSystemRecord.id.

DemandSignalJoinRun.audit_id maps to AuditEvidence.id.

DemandSignalJoinRun.join_run_id maps to WorkflowRun.external_id.

DemandSignalJoinResult.product_id maps to Product.id.

DemandSignalJoinResult.location_id maps to Location.id.

DemandSignalJoinResult.demand_date maps to PlanningTimeBucket.date.

DemandSignalJoinResult.driver_summary_json maps to ForecastExplanation.drivers.

DemandSignalJoinResult.confidence_score maps to ForecastExplanation.confidence.

DemandSignalJoinResult.sensed_delta_quantity maps to DemandPlanModifier.delta.

## 7. Workflow Steps

Node SignalSourceAuthorize checks Cedar policy and source rights.

Node SignalFreshnessScan rejects stale observations by source freshness SLO.

Node BaselineSnapshotLoad loads immutable demand plan rows.

Node SignalNormalize coerces units into planning unit of measure.

Node JoinKeyResolve builds product, location, customer, and day keys.

Node ModelScoreAttach reads approved ML score outputs.

Node ExplainabilityAssemble builds driver_summary_json.

Node MaterialityClassify marks rows above tenant materiality threshold.

Node PlannerReviewQueue publishes review tasks for Mira.

Node AuditSeal emits ADR-0263 event and audit_id.

Branch MissingSignalSource returns partial join with warning severity.

Branch SourceRightsDenied fails closed and emits policy denial.

Branch ConfidenceBelowFloor creates informational row, not a modifier.

Branch DeltaAboveGuardrail requires PlanningAdmin approval.

Branch JoinRunPromoted writes demand-plan modifier proposals.

## 8. Audit Events

DemandSensingSignalSourceRegistered records source metadata and principal.

DemandSensingObservationIngested records source, bucket, evidence hash.

DemandSensingJoinRunCreated records demand_plan_id, horizon, and policy bundle.

DemandSensingJoinRunScored records model_score_version and row counts.

DemandSensingJoinRunRejected records reason, branch, and principal.

DemandSensingJoinResultMaterialized records material deltas and confidence.

DemandSensingPlannerReviewQueued records assignee and review deadline.

DemandSensingJoinRunPromoted records modifier ids and audit_id linkage.

DemandSensingSignalJoinPolicyDenied records Cedar decision id.

Event names use class casing above and EVT-SUPPLY_CHAIN_PLANNING-DEMAND_SENSING
topic prefixes in the audit-chain envelope.

Every event carries tenant_id, principal_id, trace_id, audit_id, source_system_id,
policy_bundle_version, and residency_pack.

## 9. SLO Targets

p50 join-run creation latency: 450 ms for 10k candidate rows.

p95 join-run creation latency: 2200 ms for 100k candidate rows.

p99 join-run creation latency: 4800 ms for 250k candidate rows.

Throughput target: 30 join runs per tenant per minute.

Availability target: 99.95 percent monthly for create and read APIs.

Freshness target: 95 percent of enabled signals joined within source SLO.

Rationale: planner review windows tolerate seconds, not minutes, but stale
signal joins create incorrect supply signals.

## 10. Failure Modes + Recovery

Failure mode: source feed arrives without tenant_id.

Recovery: quarantine observation and emit DemandSensingObservationQuarantined.

Failure mode: model score version is not approved.

Recovery: reject join run before result materialization.

Failure mode: join cardinality expands unexpectedly.

Recovery: halt at JoinKeyResolve and require admin review.

Failure mode: audit-chain emission fails.

Recovery: pause promotion, keep staged results read-only, retry AuditSeal.

Failure mode: planner promotes stale run.

Recovery: compare join_run.created_at against freshness window and fail closed.

Failure mode: source rights revoked after staging.

Recovery: expire staged results and remove them from review queues.

## 11. Migration Notes with source vendor surfaces

SAP IBP-DP source: demand sensing key figures and planning area attributes.

SAP IBP-DP source: alert subscriptions and Fiori planning view filters.

Oracle source: Demand Management measure catalog and collection calendar.

Microsoft source: demand forecast entries and external signal tables.

NetSuite source: demand planning item-location history exports.

Migration maps vendor planning area to demand_plan_id plus join_key_strategy.

Migration preserves vendor signal names in source_system_signal_name.

Migration loads only approved history windows per residency pack.

Migration creates one signal_source row per vendor extract class.

Migration stores vendor extract checksum in evidence_hash.

## 12. Cross-Microservice Handoffs

Handoff to pricing uses material demand lift by product and location.

Handoff to marketplace is read-only and carries no settlement instruction.

Handoff to inventory uses demand-plan modifier proposals only after promotion.

Handoff to procurement uses aggregate lift by supplier-item group.

Handoff to workflow-engine uses PlannerReviewQueue task ids.

Handoff to audit-chain carries DemandSensingJoinRunPromoted.

Handoff to ontology writes DemandSignalJoinRun and ForecastExplanation nodes.

Handoff to notification sends planner review alerts.

## 13. Intern Build Notes

Build step 01: create migrations for the four tables above.

Build step 02: add repository methods scoped by tenant_id.

Build step 03: add DTO validation for horizon and join_key_strategy.

Build step 04: implement source enabled checks before reading observations.

Build step 05: implement freshness filtering per source_freshness_slo_seconds.

Build step 06: implement unit normalization with explicit conversion table.

Build step 07: implement join keys with deterministic string formatting.

Build step 08: implement materiality threshold lookup per tenant.

Build step 09: persist driver_summary_json with source contribution weights.

Build step 10: persist ontology_projection_id after projection success.

Build step 11: write Cedar fixtures for allowed planner create.

Build step 12: write Cedar fixtures for licensed signal denial.

Build step 13: write API contract tests for REST create.

Build step 14: write gRPC tests for StreamJoinResults pagination.

Build step 15: write replay test for duplicate idempotency key.

Build step 16: write audit fixture for DemandSensingJoinRunCreated.

Build step 17: write audit fixture for DemandSensingJoinRunPromoted.

Build step 18: add dashboard panels for stale source count.

Build step 19: add dashboard panels for material delta count.

Build step 20: add runbook link for source feed quarantine.

Build step 21: verify no raw external signal leaves residency pack.

Build step 22: verify planner review cannot see denied sources.

Build step 23: verify promotion writes modifier proposals, not baseline rows.

Build step 24: verify audit_id links every state transition.

Build step 25: verify tenant isolation with two-tenant fixture.

Build step 26: verify p95 target with 100k candidate rows.

Build step 27: verify p99 target with 250k candidate rows.

Build step 28: document vendor field mapping in migration evidence.

Build step 29: add rollback migration dropping tables in dependency order.

Build step 30: attach PR evidence for API, Cedar, audit, and SLO checks.
