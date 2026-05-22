---
doc_class: ImplementationPlan
ip_id: IP-023
microservice: supply-chain-planning
related_adrs: [ADR-0105, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0131, ADR-0132]
journey_id: j101-multi-tier-supply-chain-formation
journey_link: docs/user-journeys/j101-multi-tier-supply-chain-formation/story.md
status: Accepted
date: 2026-05-20
owner: axis-supply-chain-planning + axis-erp-parity
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [IBP-RP supply heuristic, PP/DS heuristic planning, APO SNP heuristic]
---

# IP-023: Constraint-based supply heuristic

## 1. Context with why, journey leg, named persona

This IP defines a deterministic supply-planning heuristic that respects material,
capacity, lane, sourcing, and policy constraints before a heavier optimizer is
needed.

Why this matters: planners need fast, explainable supply proposals for common
constraints, and interns need a bounded implementation that is not a black-box
solver.

Journey leg: j101 leg 10, "build a feasible supply path across multi-tier constraints".

Named persona: Leon, supply planner for regional spare parts.

Leon needs a quick heuristic plan showing which demand can be covered by
existing supply, transfers, production, or procurement proposals.

SAP equivalent: IBP-RP supply heuristic, PP/DS heuristic, and APO SNP heuristic.

Oracle equivalent: Supply Planning constrained supply plan.

Kinaxis equivalent: supply response heuristic scenario.

The feature produces explainable proposals, not final execution orders.

The feature belongs in ADR-0105 kernel, worker, application, adapter, and
governance layers.

## 2. Scope

In scope: constraint set registration.

In scope: demand and supply bucket input snapshots.

In scope: deterministic heuristic pass ordering.

In scope: supply proposal generation.

In scope: infeasibility explanation.

In scope: planner approval handoff.

Out of scope: mixed-integer optimization.

Out of scope: autonomous supplier purchase order creation.

Out of scope: plant dispatching.

Out of scope: transport tendering.

## 3. Data Model Deltas

Create table scp_supply_heuristic_run.

Column tenant_id: uuid, required, partition key.

Column heuristic_run_id: uuid, required.

Column planning_scenario_id: uuid, required.

Column horizon_start: date, required.

Column horizon_end: date, required.

Column heuristic_profile_id: text, required.

Column run_mode: enum, values simulation, review, promote.

Column status: enum, values staged, running, computed, approved, rejected,
promoted, failed.

Column policy_bundle_version: text, required.

Column created_by_principal_id: uuid, required.

Column audit_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_supply_constraint.

Column tenant_id: uuid, required.

Column constraint_id: uuid, required.

Column heuristic_run_id: uuid, required.

Column constraint_type: enum, values material, capacity, sourcing, lane,
safety_stock, allocation, calendar.

Column constraint_scope_json: jsonb, required.

Column hard_constraint: boolean, required.

Column priority_rank: integer, required.

Column limit_quantity: numeric(18,3), nullable.

Column limit_minutes: numeric(18,3), nullable.

Column active_from: date, required.

Column active_to: date, required.

Column evidence_hash: text, required.

Create table scp_supply_heuristic_bucket.

Column tenant_id: uuid, required.

Column heuristic_run_id: uuid, required.

Column bucket_id: uuid, required.

Column product_id: text, required.

Column location_id: text, required.

Column bucket_date: date, required.

Column demand_quantity: numeric(18,3), required.

Column on_hand_quantity: numeric(18,3), required.

Column planned_receipt_quantity: numeric(18,3), required.

Column capacity_minutes_available: numeric(18,3), nullable.

Column heuristic_sequence: integer, required.

Create table scp_supply_heuristic_proposal.

Column tenant_id: uuid, required.

Column supply_proposal_id: uuid, required.

Column heuristic_run_id: uuid, required.

Column proposal_type: enum, values consume_on_hand, expedite_receipt,
transfer_supply, create_production, create_purchase_requisition,
leave_short.

Column product_id: text, required.

Column source_location_id: text, nullable.

Column destination_location_id: text, required.

Column proposal_date: date, required.

Column proposal_quantity: numeric(18,3), required.

Column violated_constraint_id: uuid, nullable.

Column explanation_json: jsonb, required.

Column proposal_state: enum, values proposed, approved, rejected, promoted,
superseded.

Column ontology_projection_id: uuid, required.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/supply-heuristics/runs

REST GET /v1/supply-chain-planning/supply-heuristics/runs/{heuristic_run_id}

REST GET /v1/supply-chain-planning/supply-heuristics/runs/{id}/proposals

REST POST /v1/supply-chain-planning/supply-heuristics/runs/{id}/approve

REST POST /v1/supply-chain-planning/supply-heuristics/runs/{id}/promote

gRPC SupplyHeuristicService.CreateRun accepts CreateSupplyHeuristicRunRequest.

gRPC SupplyHeuristicService.ListProposals accepts ListSupplyHeuristicProposals.

gRPC SupplyHeuristicService.PromoteRun accepts PromoteSupplyHeuristicRunRequest.

Example request:

```json
{
  "tenant_id": "88888888-8888-8888-8888-888888888888",
  "planning_scenario_id": "scenario-spares-eu-w21",
  "horizon": {"start_date": "2026-05-21", "end_date": "2026-07-02"},
  "heuristic_profile_id": "material-first-capacity-second-v2",
  "run_mode": "review"
}
```

Example response:

```json
{
  "heuristic_run_id": "heur-023-0001",
  "status": "computed",
  "proposal_count": 918,
  "short_bucket_count": 37,
  "hard_constraint_violation_count": 0,
  "audit_id": "aud-scp-heuristic-023"
}
```

Promote response includes promoted_supply_plan_delta_id.

Approve request requires approval_reason_code.

Promote request requires idempotency_key.

## 5. Cedar Policy Hooks

Principal type: SupplyPlanner, ScenarioPlanner, PlanningAdmin.

Action: scp::Action::"CreateSupplyHeuristicRun".

Action: scp::Action::"ApproveSupplyHeuristicRun".

Action: scp::Action::"PromoteSupplyHeuristicRun".

Resource: scp::SupplyHeuristicRun::"<tenant_id>/<heuristic_run_id>".

Context tenant_id must equal principal.tenant_id.

Context planning_scenario_id must be in principal.scenario_scope.

Context hard_constraint_override requires PlanningAdmin.

Context promote requires approved run and no hard constraint violation.

Context create_purchase_requisition proposal requires procurement_handoff_allowed.

Policy denial emits SupplyHeuristicPolicyDenied.

## 6. Ontology Projection Field Mapping

heuristic_run_id maps to SupplyHeuristicRun.id.

planning_scenario_id maps to PlanningScenario.id.

heuristic_profile_id maps to HeuristicProfile.id.

constraint_id maps to PlanningConstraint.id.

constraint_type maps to PlanningConstraint.type.

bucket_id maps to SupplyDemandBucket.id.

product_id maps to Product.id.

location_id maps to Location.id.

supply_proposal_id maps to SupplyProposal.id.

proposal_type maps to SupplyProposal.type.

proposal_quantity maps to SupplyProposal.quantity.

violated_constraint_id maps to PlanningConstraintViolation.constraint.

explanation_json maps to PlanningExplanation.drivers.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node ScenarioSnapshotLoad loads demand, supply, lane, and capacity snapshots.

Node HeuristicPolicyAuthorize evaluates Cedar.

Node ConstraintSetLoad orders hard constraints before soft constraints.

Node BucketSequenceBuild orders buckets by demand date and priority.

Node OnHandConsumePass consumes eligible stock first.

Node ReceiptExpeditePass proposes expedited receipts where allowed.

Node TransferSupplyPass proposes transfers across eligible lanes.

Node ProductionCreatePass proposes production where capacity exists.

Node PurchaseRequisitionPass proposes procurement for remaining shortages.

Node InfeasibilityExplain records remaining shortage and violated constraints.

Node PlannerReviewQueue routes run to Leon.

Node PromotePlanDelta writes approved supply plan deltas.

Node AuditSeal emits run, proposal, and promotion events.

Branch HardConstraintViolation blocks approval.

Branch SoftConstraintViolation marks proposal with warning.

Branch NoEligibleLane leaves shortage and explains lane constraint.

Branch CapacityUnavailable leaves shortage or proposes later production.

Branch SimulationOnly never writes supply plan deltas.

## 8. Audit Events

SupplyHeuristicRunCreated records scenario, horizon, and profile.

SupplyHeuristicConstraintSetLoaded records hard and soft counts.

SupplyHeuristicBucketSequenced records bucket count and sort profile.

SupplyHeuristicProposalGenerated records proposal id and proposal type.

SupplyHeuristicHardConstraintBlocked records violated constraint id.

SupplyHeuristicInfeasibilityExplained records short bucket count.

SupplyHeuristicRunApproved records approver and reason code.

SupplyHeuristicRunPromoted records supply plan delta id.

SupplyHeuristicRunRejected records rejection reason.

SupplyHeuristicPolicyDenied records Cedar decision id.

Events use EVT-SUPPLY_CHAIN_PLANNING-SUPPLY_HEURISTIC prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, source_system_id, and residency_pack.

## 9. SLO Targets

p50 heuristic run latency: 900 ms for 1000 buckets and 100 constraints.

p95 heuristic run latency: 8500 ms for 50000 buckets and 1000 constraints.

p99 heuristic run latency: 18000 ms for 150000 buckets and 5000 constraints.

Throughput target: 20 heuristic runs per tenant per hour.

Proposal read throughput target: 300 pages per minute per tenant.

Availability target: 99.9 percent monthly for create and promote APIs.

Rationale: heuristic planning is heavier than ATP but must remain interactive
enough for scenario iteration and planner review.

## 10. Failure Modes + Recovery

Failure mode: hard constraint input is missing.

Recovery: fail run before proposal generation and emit constraint gap event.

Failure mode: bucket sequence creates non-deterministic order.

Recovery: require stable tie-breaker by product, location, and date.

Failure mode: lane constraints contradict sourcing rules.

Recovery: prefer hard constraint, leave shortage, and explain conflict.

Failure mode: promotion conflicts with newer scenario snapshot.

Recovery: supersede run and require recompute from current snapshot.

Failure mode: procurement handoff is disabled.

Recovery: keep purchase requisition proposals in proposed state only.

Failure mode: audit-chain unavailable.

Recovery: block promote until audit seal succeeds.

## 11. Migration Notes with source vendor surfaces

SAP IBP-RP source: supply heuristic profile and response planning constraints.

SAP PP/DS source: heuristic planning procedure and finite resource buckets.

SAP APO SNP source: heuristic deployment and supply planning buckets.

Oracle source: constrained supply plan and sourcing rule constraints.

Kinaxis source: scenario heuristic workbook and constraint tables.

Migration maps SAP heuristic procedure to heuristic_profile_id.

Migration maps PP/DS resource bucket to capacity constraint.

Migration maps SNP lane to lane constraint.

Migration maps vendor planned supply element to supply_proposal.

Migration stores vendor constraint checksum in evidence_hash.

## 12. Cross-Microservice Handoffs

Handoff from demand-plan provides demand buckets.

Handoff from inventory provides on-hand and receipt buckets.

Handoff from deployment planning provides transfer lane options.

Handoff from CTP provides capacity feasibility signals.

Handoff to procurement provides purchase requisition proposals.

Handoff to production-planning provides production proposals.

Handoff to transportation-plan provides transfer proposals.

Handoff to audit-chain emits run, proposal, and promote events.

Handoff to ontology projects constraints, buckets, and proposals.

## 13. Intern Build Notes

Build step 01: create run, constraint, bucket, and proposal migrations.

Build step 02: add indexes by tenant, run, product, location, and date.

Build step 03: implement scenario snapshot read port.

Build step 04: implement Cedar authorization before snapshot load.

Build step 05: implement constraint ordering with hard-first sort.

Build step 06: implement deterministic bucket sequence.

Build step 07: implement on-hand consume pass.

Build step 08: implement expedited receipt pass.

Build step 09: implement transfer supply pass.

Build step 10: implement production create pass.

Build step 11: implement purchase requisition pass.

Build step 12: implement infeasibility explanation builder.

Build step 13: implement run approval transaction.

Build step 14: implement promote transaction to supply plan delta.

Build step 15: add Cedar fixture for supply planner create.

Build step 16: add Cedar fixture for hard override denial.

Build step 17: add Cedar fixture for procurement handoff denial.

Build step 18: write deterministic ordering test.

Build step 19: write hard constraint blocking test.

Build step 20: write soft constraint warning test.

Build step 21: write promotion conflict test against scenario version.

Build step 22: write audit fixture for run promoted.

Build step 23: write audit fixture for policy denied.

Build step 24: add metric for short bucket count.

Build step 25: add metric for hard constraint blocked count.

Build step 26: verify simulation does not write supply plan delta.

Build step 27: verify p95 run latency with 50000 buckets.

Build step 28: document SAP heuristic profile mapping.

Build step 29: add rollback migration in child-first order.

Build step 30: attach PR evidence for API, policy, audit, and SLO checks.
