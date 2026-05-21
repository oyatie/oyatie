---
doc_class: ImplementationPlan
ip_id: IP-018
microservice: supply-chain-planning
related_adrs: [ADR-0105, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0131, ADR-0132]
journey_id: j124-supply-chain-disruption-emergency-coordination
journey_link: docs/user-journeys/j124-supply-chain-disruption-emergency-coordination/story.md
status: Accepted
date: 2026-05-20
owner: axis-supply-chain-planning + axis-erp-parity
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [IBP-RP response and supply, IBP inventory optimization, SAP APO SNP deployment]
---

# IP-018: Deployment planning across DCs

## 1. Context with why, journey leg, named persona

This IP defines deployment planning across distribution centers when available
supply must be moved from source DCs to demand DCs under service, cost,
transport, and shortage constraints.

Why this matters: planners need a governed deployment proposal before ATP and
allocation decisions consume stale or locally biased supply positions.

Journey leg: j124 leg 05, "rebalance network supply after a disruption changes DC availability".

Named persona: Priya, network deployment planner for US grocery replenishment.

Priya reviews DC imbalances each morning and needs a suggested transfer plan
that explains why each lane was chosen or rejected.

SAP equivalent: APO SNP deployment and IBP-RP response planning.

Oracle equivalent: Supply Planning inventory rebalancing.

Kinaxis equivalent: RapidResponse supply response scenario.

The feature produces proposals, not warehouse execution orders.

The feature belongs to ADR-0105 application, usecase, adapter, and governance
layers.

The implementation must keep tenant, residency, policy decision, and audit
fields visible on every proposed movement.

## 2. Scope

In scope: source and destination DC eligibility.

In scope: available deployable inventory calculation.

In scope: transfer lane constraints.

In scope: service-risk weighted deployment proposal.

In scope: planner approval and rejection.

In scope: handoff to transportation planning after approval.

Out of scope: carrier tendering.

Out of scope: physical warehouse shipment execution.

Out of scope: autonomous write to inventory balances.

Out of scope: financial intercompany settlement.

## 3. Data Model Deltas

Create table scp_dc_deployment_run.

Column tenant_id: uuid, required, partition key.

Column deployment_run_id: uuid, required.

Column planning_scenario_id: uuid, required.

Column horizon_start: date, required.

Column horizon_end: date, required.

Column product_scope_json: jsonb, required.

Column dc_scope_json: jsonb, required.

Column objective: enum, values service_recovery, cost_minimize, expiry_reduce,
balanced.

Column status: enum, values staged, optimized, approved, rejected, expired.

Column policy_bundle_version: text, required.

Column audit_id: uuid, required.

Column created_by_principal_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_dc_deployable_inventory.

Column tenant_id: uuid, required.

Column deployment_run_id: uuid, required.

Column product_id: text, required.

Column dc_id: text, required.

Column inventory_date: date, required.

Column on_hand_quantity: numeric(18,3), required.

Column reserved_quantity: numeric(18,3), required.

Column safety_stock_quantity: numeric(18,3), required.

Column deployable_quantity: numeric(18,3), required.

Column shelf_life_days_remaining: integer, nullable.

Column evidence_hash: text, required.

Create table scp_dc_transfer_lane_constraint.

Column tenant_id: uuid, required.

Column lane_id: text, required.

Column source_dc_id: text, required.

Column destination_dc_id: text, required.

Column transit_days: integer, required.

Column min_transfer_quantity: numeric(18,3), required.

Column max_transfer_quantity: numeric(18,3), required.

Column transport_cost_per_unit: numeric(18,6), required.

Column carbon_cost_per_unit: numeric(18,6), nullable.

Column lane_status: enum, values open, constrained, closed.

Create table scp_dc_deployment_proposal.

Column tenant_id: uuid, required, partition key.

Column deployment_proposal_id: uuid, required.

Column deployment_run_id: uuid, required.

Column product_id: text, required.

Column source_dc_id: text, required.

Column destination_dc_id: text, required.

Column proposed_quantity: numeric(18,3), required.

Column need_date: date, required.

Column service_recovery_score: numeric(8,5), required.

Column cost_score: numeric(8,5), required.

Column rationale_json: jsonb, required.

Column approval_state: enum, values pending, approved, rejected, superseded.

Column ontology_projection_id: uuid, required.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/deployment/runs

REST GET /v1/supply-chain-planning/deployment/runs/{deployment_run_id}

REST GET /v1/supply-chain-planning/deployment/runs/{deployment_run_id}/proposals

REST POST /v1/supply-chain-planning/deployment/proposals/{proposal_id}/approve

REST POST /v1/supply-chain-planning/deployment/proposals/{proposal_id}/reject

gRPC DeploymentPlanningService.CreateRun accepts CreateDeploymentRunRequest.

gRPC DeploymentPlanningService.ListProposals accepts ListDeploymentProposals.

gRPC DeploymentPlanningService.DecideProposal accepts DecideDeploymentProposal.

Example create request:

```json
{
  "tenant_id": "33333333-3333-3333-3333-333333333333",
  "planning_scenario_id": "scenario-us-grocery-w21",
  "horizon": {"start_date": "2026-05-21", "end_date": "2026-06-18"},
  "objective": "service_recovery",
  "product_scope": {"category": "fresh-dairy"},
  "dc_scope": {"region": "us-east"}
}
```

Example proposal response:

```json
{
  "deployment_run_id": "dep-018-run-001",
  "proposal_count": 43,
  "top_proposal": {
    "source_dc_id": "dc-atl",
    "destination_dc_id": "dc-bos",
    "product_id": "milk-1l",
    "proposed_quantity": 9200,
    "service_recovery_score": 0.91
  },
  "audit_id": "aud-scp-deployment-018"
}
```

Approve response includes transportation_plan_request_id.

Reject response requires reason_code and planner_note.

All write endpoints require idempotency_key.

## 5. Cedar Policy Hooks

Principal type: NetworkDeploymentPlanner.

Action: scp::Action::"CreateDeploymentRun".

Action: scp::Action::"ApproveDeploymentProposal".

Action: scp::Action::"RejectDeploymentProposal".

Resource: scp::DeploymentRun::"<tenant_id>/<deployment_run_id>".

Context tenant_id must match principal.tenant_id.

Context source_dc_id and destination_dc_id must be in principal.dc_scope.

Context lane_status closed denies approval.

Context proposed_quantity over threshold requires PlanningAdmin.

Context expiry_reduce objective requires perishable_inventory role.

Policy denial emits DeploymentPlanningPolicyDenied.

Cedar decision id is persisted on the run and proposal decision event.

## 6. Ontology Projection Field Mapping

deployment_run_id maps to DeploymentPlanningRun.id.

planning_scenario_id maps to PlanningScenario.id.

product_scope_json maps to ProductScope.filter.

dc_scope_json maps to DistributionCenterScope.filter.

deployable_inventory.product_id maps to Product.id.

deployable_inventory.dc_id maps to DistributionCenter.id.

lane_id maps to TransferLane.id.

source_dc_id maps to TransferLane.source.

destination_dc_id maps to TransferLane.destination.

deployment_proposal_id maps to InventoryDeploymentProposal.id.

proposed_quantity maps to InventoryDeploymentProposal.quantity.

rationale_json maps to PlanningExplanation.drivers.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node ScenarioSnapshotLoad loads approved network supply state.

Node DCScopeAuthorize applies Cedar scope checks.

Node DeployableInventoryCompute subtracts reservations and safety stock.

Node LaneConstraintLoad reads open lanes and transit days.

Node NeedSignalLoad reads demand and ATP risk by destination DC.

Node ProposalGenerate ranks lane, product, and quantity choices.

Node ProposalGuardrailCheck applies max move and expiry rules.

Node PlannerApprovalQueue routes proposals to Priya.

Node TransportationHandoff creates transport planning request after approval.

Node AuditSeal emits run, proposal, and decision events.

Branch NoOpenLane marks product-destination as unresolved.

Branch InsufficientDeployableSupply creates shortage signal for allocation.

Branch PerishableExpiryRisk promotes expiry_reduce objective.

Branch CostExceedsBenefit suppresses proposal with rationale.

Branch ApprovalSuperseded rejects stale proposal after inventory refresh.

## 8. Audit Events

DeploymentPlanningRunCreated records scenario, scope, and objective.

DeploymentPlanningDeployableInventoryComputed records row count and hash.

DeploymentPlanningLaneConstraintApplied records lane count and closed lanes.

DeploymentPlanningProposalGenerated records proposal id and rationale hash.

DeploymentPlanningProposalApproved records approver and transportation handoff.

DeploymentPlanningProposalRejected records reason code and note hash.

DeploymentPlanningProposalSuperseded records replacement run id.

DeploymentPlanningPolicyDenied records Cedar decision id.

DeploymentPlanningTransportationHandoffCreated records handoff id.

Events use EVT-SUPPLY_CHAIN_PLANNING-DEPLOYMENT_PLANNING prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, source_system_id, and residency_pack.

## 9. SLO Targets

p50 run creation latency: 700 ms for 50 product-location pairs.

p95 proposal generation latency: 6500 ms for 5000 product-location-lane rows.

p99 proposal generation latency: 14000 ms for 25000 rows.

Throughput target: 12 deployment runs per tenant per hour.

Proposal read throughput target: 300 proposal page reads per minute per tenant.

Availability target: 99.9 percent monthly for create and approve APIs.

Rationale: deployment runs are planner-cycle workloads, but approvals must stay
responsive while transportation windows are open.

## 10. Failure Modes + Recovery

Failure mode: inventory snapshot is older than configured freshness.

Recovery: block run creation and emit stale snapshot audit event.

Failure mode: lane constraint source is unavailable.

Recovery: use last sealed constraints and mark run degraded.

Failure mode: deployable quantity becomes negative.

Recovery: clamp to zero and create data quality incident.

Failure mode: proposal approved after lane closure.

Recovery: re-check lane_status in decision transaction and deny.

Failure mode: transportation handoff fails.

Recovery: keep proposal approved_pending_handoff and retry with same idempotency.

Failure mode: audit-chain fails.

Recovery: block transportation handoff until audit seal succeeds.

## 11. Migration Notes with source vendor surfaces

SAP APO SNP source: deployment optimizer lane and product-location data.

SAP IBP-RP source: response planning supply and demand pegging outputs.

SAP EWM source: DC inventory availability export.

Oracle source: Supply Planning transfer recommendation export.

Blue Yonder source: network inventory rebalancing recommendation.

Migration maps vendor location to DistributionCenter.id.

Migration maps stock transfer lane to transfer_lane_constraint.

Migration maps vendor deployment proposal to deployment_proposal.

Migration stores vendor optimizer run id in rationale_json.vendor_run_id.

Migration stores source extract checksum in evidence_hash.

## 12. Cross-Microservice Handoffs

Handoff from inventory provides on-hand and reserved quantities.

Handoff from demand-plan provides destination need by date.

Handoff from ATP provides service recovery risk.

Handoff to transportation-plan creates lane execution planning request.

Handoff to allocation receives unresolved shortage signals.

Handoff to workflow-engine creates planner approval tasks.

Handoff to audit-chain records approval and handoff events.

Handoff to ontology projects DeploymentPlanningRun and TransferLane nodes.

Handoff to notification sends proposal approval alerts.

## 13. Intern Build Notes

Build step 01: create run, inventory, lane, and proposal migrations.

Build step 02: add tenant-scoped repository methods for runs.

Build step 03: add inventory snapshot freshness validator.

Build step 04: implement deployable quantity calculation.

Build step 05: implement lane status filtering.

Build step 06: implement objective-weighted proposal scoring.

Build step 07: implement rationale_json builder with rejected lane reasons.

Build step 08: implement REST create run endpoint.

Build step 09: implement REST list proposals endpoint.

Build step 10: implement approve and reject endpoints.

Build step 11: implement gRPC DecideProposal.

Build step 12: add Cedar fixture for DC scope allow.

Build step 13: add Cedar fixture for lane closed denial.

Build step 14: add Cedar fixture for quantity threshold escalation.

Build step 15: write contract test for create run request.

Build step 16: write proposal generation test with two DCs.

Build step 17: write proposal rejection test with reason code.

Build step 18: write transportation handoff idempotency test.

Build step 19: write audit fixture for proposal approval.

Build step 20: write audit fixture for policy denial.

Build step 21: add metric for stale inventory snapshot count.

Build step 22: add metric for approved pending handoff count.

Build step 23: verify no proposal writes inventory balances.

Build step 24: verify tenant isolation for two overlapping DC codes.

Build step 25: verify p95 proposal generation target.

Build step 26: verify rollback migration removes child tables first.

Build step 27: document SAP APO SNP lane field mapping.

Build step 28: document IBP-RP pegging field mapping.

Build step 29: attach ontology projection evidence.

Build step 30: attach PR evidence for API, policy, audit, and SLO checks.
