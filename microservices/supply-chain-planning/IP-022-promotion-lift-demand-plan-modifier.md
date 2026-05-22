---
doc_class: ImplementationPlan
ip_id: IP-022
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
sap_submodule_equivalents: [IBP-DP promotion planning, IBP-DP demand planning key figures, IBP-SOP consensus demand]
---

# IP-022: Promotion lift demand plan modifier

## 1. Context with why, journey leg, named persona

This IP defines promotion lift modifiers that adjust an approved demand plan
with governed promotional uplift, cannibalization, and post-event dip effects.

Why this matters: promotion calendars frequently change after baseline demand
planning, and planners need traceable modifiers instead of overwriting forecast
history or consensus demand.

Journey leg: j123 leg 09, "stage promotional lift into the launch demand plan".

Named persona: Sofia, trade promotion planner for consumer packaged goods.

Sofia receives a new retailer promotion and needs a demand-plan modifier that
shows lift, substitution, confidence, and supply handoff impact.

SAP equivalent: IBP-DP promotion planning and demand planning key figures.

Oracle equivalent: Demand Management causal factor and promotion lift.

Microsoft equivalent: demand forecasting adjustments from trade events.

The feature stages modifiers for review and promotion; it does not rewrite the
baseline demand plan.

The feature belongs in ADR-0105 application, usecase, adapter, and governance
layers.

## 2. Scope

In scope: promotion event registration.

In scope: promotion lift profile definition.

In scope: cannibalization and halo effect modeling.

In scope: demand plan modifier proposal.

In scope: planner approval and rejection.

In scope: handoff to ATP, allocation, and replenishment after promotion.

Out of scope: trade spend settlement.

Out of scope: ad campaign bidding.

Out of scope: POS data ingestion owned by demand sensing.

Out of scope: final financial accruals.

## 3. Data Model Deltas

Create table scp_promotion_event.

Column tenant_id: uuid, required, partition key.

Column promotion_event_id: uuid, required.

Column source_system_id: text, required.

Column promotion_name: text, required.

Column retailer_id: text, nullable.

Column channel_id: text, required.

Column region_id: text, required.

Column promotion_start_date: date, required.

Column promotion_end_date: date, required.

Column promotion_type: enum, values price_discount, display, bundle, coupon,
media_push, retailer_feature.

Column expected_funding_owner: enum, values supplier, retailer, shared,
unknown.

Column status: enum, values draft, staged, approved, rejected, expired.

Column audit_id: uuid, required.

Create table scp_promotion_lift_profile.

Column tenant_id: uuid, required.

Column lift_profile_id: uuid, required.

Column promotion_event_id: uuid, required.

Column product_id: text, required.

Column location_scope_json: jsonb, required.

Column baseline_quantity: numeric(18,3), required.

Column lift_percent: numeric(9,6), required.

Column cannibalization_percent: numeric(9,6), required.

Column halo_percent: numeric(9,6), required.

Column post_event_dip_percent: numeric(9,6), required.

Column confidence_score: numeric(6,5), required.

Column evidence_hash: text, required.

Create table scp_promotion_modifier_run.

Column tenant_id: uuid, required, partition key.

Column modifier_run_id: uuid, required.

Column promotion_event_id: uuid, required.

Column demand_plan_id: uuid, required.

Column run_mode: enum, values simulation, review, promote.

Column status: enum, values staged, computed, approved, rejected, promoted.

Column policy_bundle_version: text, required.

Column created_by_principal_id: uuid, required.

Column audit_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_promotion_demand_modifier.

Column tenant_id: uuid, required.

Column demand_modifier_id: uuid, required.

Column modifier_run_id: uuid, required.

Column product_id: text, required.

Column location_id: text, required.

Column demand_date: date, required.

Column baseline_quantity: numeric(18,3), required.

Column lift_quantity: numeric(18,3), required.

Column cannibalized_quantity: numeric(18,3), required.

Column halo_quantity: numeric(18,3), required.

Column net_modifier_quantity: numeric(18,3), required.

Column explanation_json: jsonb, required.

Column modifier_state: enum, values proposed, approved, rejected, promoted,
superseded.

Column ontology_projection_id: uuid, required.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/promotions/events

REST POST /v1/supply-chain-planning/promotions/modifier-runs

REST GET /v1/supply-chain-planning/promotions/modifier-runs/{id}/modifiers

REST POST /v1/supply-chain-planning/promotions/modifier-runs/{id}/approve

REST POST /v1/supply-chain-planning/promotions/modifier-runs/{id}/promote

gRPC PromotionLiftService.CreatePromotionEvent accepts CreatePromotionEventRequest.

gRPC PromotionLiftService.ComputeModifiers accepts ComputePromotionModifiers.

gRPC PromotionLiftService.PromoteModifiers accepts PromotePromotionModifiers.

Example request:

```json
{
  "tenant_id": "77777777-7777-7777-7777-777777777777",
  "promotion_event_id": "promo-retailer-w24",
  "demand_plan_id": "dp-cpg-kr-w21",
  "run_mode": "review",
  "explanation_level": "planner"
}
```

Example response:

```json
{
  "modifier_run_id": "plm-022-0001",
  "status": "computed",
  "modifier_count": 336,
  "net_lift_quantity": 18400,
  "max_confidence_score": 0.88,
  "audit_id": "aud-scp-promotion-lift-022"
}
```

Promote response includes demand_plan_modifier_set_id.

Approve request requires planner_note and approval_reason_code.

Promote request requires idempotency_key.

## 5. Cedar Policy Hooks

Principal type: DemandPlanner, TradePromotionPlanner, PlanningAdmin.

Action: scp::Action::"CreatePromotionEvent".

Action: scp::Action::"ComputePromotionLiftModifier".

Action: scp::Action::"ApprovePromotionLiftModifier".

Action: scp::Action::"PromotePromotionLiftModifier".

Resource: scp::PromotionEvent::"<tenant_id>/<promotion_event_id>".

Context tenant_id must equal principal.tenant_id.

Context retailer_id must be in principal.retailer_scope when present.

Context promotion_type media_push requires marketing_signal_access.

Context promote requires DemandPlanner and approved modifier_run.

Context net_modifier_quantity over guardrail requires PlanningAdmin.

Policy denial emits PromotionLiftPolicyDenied.

## 6. Ontology Projection Field Mapping

promotion_event_id maps to PromotionEvent.id.

promotion_name maps to PromotionEvent.name.

retailer_id maps to Customer.id.

channel_id maps to SalesChannel.id.

region_id maps to Region.id.

promotion_type maps to PromotionEvent.type.

lift_profile_id maps to PromotionLiftProfile.id.

product_id maps to Product.id.

modifier_run_id maps to PromotionModifierRun.id.

demand_plan_id maps to DemandPlan.id.

demand_modifier_id maps to DemandPlanModifier.id.

net_modifier_quantity maps to DemandPlanModifier.quantity_delta.

explanation_json maps to ForecastExplanation.drivers.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node PromotionEventValidate checks date range and channel scope.

Node PromotionPolicyAuthorize evaluates Cedar.

Node BaselineDemandLoad loads approved demand plan rows.

Node LiftProfileLoad reads lift, halo, cannibalization, and dip factors.

Node ModifierCompute builds date and location modifiers.

Node GuardrailCheck compares net lift to tenant thresholds.

Node SupplyImpactPreview computes ATP and replenishment impact summary.

Node PlannerReviewQueue routes run to Sofia.

Node ModifierPromote writes demand-plan modifier set.

Node AuditSeal emits event, run, modifier, and promotion events.

Branch MissingLiftProfile rejects compute with profile gap.

Branch LowConfidence routes to PlanningAdmin review.

Branch GuardrailExceeded blocks promotion until approval.

Branch SimulationOnly never writes demand-plan modifier set.

Branch PromotionExpired rejects promotion unless override is approved.

## 8. Audit Events

PromotionLiftEventCreated records event metadata and source system.

PromotionLiftProfileRegistered records product and lift factors.

PromotionLiftModifierRunCreated records demand plan and run mode.

PromotionLiftModifierComputed records modifier count and net lift.

PromotionLiftGuardrailExceeded records threshold and responsible planner.

PromotionLiftModifierApproved records approver and reason code.

PromotionLiftModifierPromoted records modifier set id.

PromotionLiftModifierRejected records rejection reason.

PromotionLiftPolicyDenied records Cedar decision id.

PromotionLiftSupplyImpactPreviewed records ATP and replenishment signals.

Events use EVT-SUPPLY_CHAIN_PLANNING-PROMOTION_LIFT prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, source_system_id, and residency_pack.

## 9. SLO Targets

p50 modifier compute latency: 300 ms for 100 product-location-day rows.

p95 modifier compute latency: 2400 ms for 25000 rows.

p99 modifier compute latency: 5200 ms for 100000 rows.

Throughput target: 60 modifier runs per tenant per hour.

Modifier read throughput target: 300 pages per minute per tenant.

Availability target: 99.95 percent monthly for compute and promote APIs.

Rationale: promotion planning is batch-review oriented, but planners need quick
iterations before S and OP cutoffs.

## 10. Failure Modes + Recovery

Failure mode: promotion date overlaps locked demand plan window.

Recovery: block promotion and require PlanningAdmin unlock approval.

Failure mode: lift profile has negative confidence.

Recovery: reject profile at validation and emit data quality event.

Failure mode: cannibalization creates negative demand.

Recovery: clamp at zero and record clamp in explanation_json.

Failure mode: supply impact preview fails.

Recovery: keep modifier computed and mark impact preview degraded.

Failure mode: promote conflicts with newer demand-plan version.

Recovery: supersede run and require recompute from latest baseline.

Failure mode: audit-chain unavailable.

Recovery: block promote until audit seal succeeds.

## 11. Migration Notes with source vendor surfaces

SAP IBP-DP source: promotion key figures and planning area attributes.

SAP TPM source: promotion event calendar and retailer attributes.

Oracle source: causal factor, event uplift, and forecast adjustment.

Blue Yonder source: demand event and lift library.

Microsoft source: demand forecast adjustment journal.

Migration maps vendor promotion id to promotion_event_id.

Migration maps causal factor to lift_profile_id.

Migration maps uplift percentage to lift_percent.

Migration maps halo and cannibalization factors into profile columns.

Migration stores vendor extract hash in evidence_hash.

## 12. Cross-Microservice Handoffs

Handoff from trade-promotion provides event calendar and funding owner.

Handoff from demand-plan provides baseline rows.

Handoff from demand-sensing provides recent signal confidence.

Handoff to ATP provides promoted lift by product and location.

Handoff to replenishment provides net modifier demand.

Handoff to allocation provides shortage risk for promoted products.

Handoff to workflow-engine creates review and approval tasks.

Handoff to audit-chain emits event, modifier, and promotion events.

Handoff to ontology projects PromotionEvent and DemandPlanModifier.

## 13. Intern Build Notes

Build step 01: create event, profile, run, and modifier migrations.

Build step 02: add repository methods by tenant and promotion event.

Build step 03: implement promotion date and scope validation.

Build step 04: implement Cedar authorization before baseline demand load.

Build step 05: implement baseline demand read port.

Build step 06: implement lift profile read port.

Build step 07: implement modifier calculation for lift.

Build step 08: implement modifier calculation for cannibalization.

Build step 09: implement modifier calculation for halo.

Build step 10: implement modifier calculation for post-event dip.

Build step 11: implement guardrail check.

Build step 12: implement supply impact preview port.

Build step 13: implement promote transaction to demand-plan modifier set.

Build step 14: add Cedar fixture for trade planner allow.

Build step 15: add Cedar fixture for media signal denial.

Build step 16: add Cedar fixture for guardrail admin escalation.

Build step 17: write contract test for modifier compute.

Build step 18: write contract test for promotion expired rejection.

Build step 19: write contract test for promote version conflict.

Build step 20: write audit fixture for PromotionLiftModifierPromoted.

Build step 21: write audit fixture for PromotionLiftPolicyDenied.

Build step 22: add metric for net lift quantity.

Build step 23: add metric for low-confidence modifier count.

Build step 24: verify simulation does not write modifier set.

Build step 25: verify tenant isolation with same retailer code.

Build step 26: verify p95 compute target with 25000 rows.

Build step 27: document SAP IBP-DP key figure mapping.

Build step 28: document TPM event calendar mapping.

Build step 29: add rollback migration in child-first order.

Build step 30: attach PR evidence for API, policy, audit, and SLO checks.
