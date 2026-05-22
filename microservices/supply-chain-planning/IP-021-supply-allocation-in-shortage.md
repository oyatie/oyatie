---
doc_class: ImplementationPlan
ip_id: IP-021
microservice: supply-chain-planning
related_adrs: [ADR-0105, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0131, ADR-0132]
journey_id: j107-supply-chain-disruption-and-failover
journey_link: docs/user-journeys/j107-supply-chain-disruption-and-failover/story.md
status: Accepted
date: 2026-05-20
owner: axis-supply-chain-planning + axis-erp-parity
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [aATP product allocation, ATP allocation check, IBP-RP shortage response]
---

# IP-021: Supply allocation in shortage

## 1. Context with why, journey leg, named persona

This IP defines governed allocation of scarce supply across customers, channels,
regions, and orders when requested demand exceeds constrained available supply.

Why this matters: shortage handling must be explainable, fair by configured
policy, auditable, and safe from high-priority customer leakage.

Journey leg: j107 leg 08, "ration constrained supply during disruption and failover".

Named persona: Hana, allocation manager for pharmaceutical cold-chain supply.

Hana must allocate scarce supply across hospitals, distributors, and ecommerce
without violating contract tiers or regulated-service commitments.

SAP equivalent: S/4HANA aATP product allocation and supply protection.

Oracle equivalent: Global Order Promising allocation rules.

Kinaxis equivalent: constrained allocation scenario planning.

The feature creates allocation reservations and explanations; it does not book
revenue or ship goods.

The feature belongs in ADR-0105 kernel, governance, application, API, and worker
layers.

## 2. Scope

In scope: shortage pool definition.

In scope: allocation rule configuration.

In scope: customer and channel priority tiers.

In scope: fair-share and hard reservation methods.

In scope: allocation reservation outputs.

In scope: exception approval for override.

Out of scope: legal contract authoring.

Out of scope: shipment execution.

Out of scope: price protection or rebate settlement.

Out of scope: replacing ATP compute.

## 3. Data Model Deltas

Create table scp_shortage_pool.

Column tenant_id: uuid, required, partition key.

Column shortage_pool_id: uuid, required.

Column product_id: text, required.

Column location_scope_json: jsonb, required.

Column shortage_start_date: date, required.

Column shortage_end_date: date, required.

Column constrained_quantity: numeric(18,3), required.

Column demand_quantity: numeric(18,3), required.

Column shortage_reason_code: text, required.

Column status: enum, values draft, active, frozen, closed.

Column audit_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_allocation_rule.

Column tenant_id: uuid, required.

Column allocation_rule_id: uuid, required.

Column shortage_pool_id: uuid, required.

Column rule_method: enum, values fair_share, priority_tier, contract_minimum,
emergency_service, manual_exception.

Column customer_scope_json: jsonb, required.

Column channel_scope_json: jsonb, required.

Column priority_weight: numeric(8,5), required.

Column minimum_protected_quantity: numeric(18,3), required.

Column maximum_allocated_quantity: numeric(18,3), nullable.

Column rule_rank: integer, required.

Column enabled: boolean, required.

Create table scp_allocation_run.

Column tenant_id: uuid, required, partition key.

Column allocation_run_id: uuid, required.

Column shortage_pool_id: uuid, required.

Column run_mode: enum, values simulation, commit.

Column status: enum, values staged, computed, committed, rejected, superseded.

Column policy_bundle_version: text, required.

Column created_by_principal_id: uuid, required.

Column audit_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_allocation_reservation.

Column tenant_id: uuid, required.

Column allocation_reservation_id: uuid, required.

Column allocation_run_id: uuid, required.

Column customer_id: text, nullable.

Column channel_id: text, nullable.

Column region_id: text, nullable.

Column product_id: text, required.

Column location_id: text, required.

Column allocated_quantity: numeric(18,3), required.

Column allocation_rank: integer, required.

Column explanation_json: jsonb, required.

Column reservation_state: enum, values proposed, active, consumed, released,
expired.

Column ontology_projection_id: uuid, required.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/allocation/shortage-pools

REST POST /v1/supply-chain-planning/allocation/runs

REST GET /v1/supply-chain-planning/allocation/runs/{allocation_run_id}/reservations

REST POST /v1/supply-chain-planning/allocation/runs/{allocation_run_id}/commit

REST POST /v1/supply-chain-planning/allocation/reservations/{id}/release

gRPC SupplyAllocationService.CreateShortagePool accepts CreateShortagePoolRequest.

gRPC SupplyAllocationService.ComputeAllocation accepts ComputeAllocationRequest.

gRPC SupplyAllocationService.CommitAllocation accepts CommitAllocationRequest.

Example run request:

```json
{
  "tenant_id": "66666666-6666-6666-6666-666666666666",
  "shortage_pool_id": "pool-vaccine-kr-w21",
  "run_mode": "simulation",
  "rule_set_version": "allocation-pharma-v4",
  "explanation_level": "audit"
}
```

Example response:

```json
{
  "allocation_run_id": "alloc-021-0001",
  "status": "computed",
  "reservation_count": 214,
  "unallocated_quantity": 120,
  "top_reason": "emergency_service_minimum_protected",
  "audit_id": "aud-scp-allocation-021"
}
```

Commit endpoint creates active allocation reservations.

Release endpoint requires release_reason_code.

All commit and release requests require idempotency_key.

## 5. Cedar Policy Hooks

Principal type: AllocationManager, EmergencySupplyOfficer, ComplianceAuditor.

Action: scp::Action::"CreateShortagePool".

Action: scp::Action::"ComputeShortageAllocation".

Action: scp::Action::"CommitShortageAllocation".

Action: scp::Action::"ReleaseAllocationReservation".

Resource: scp::ShortagePool::"<tenant_id>/<shortage_pool_id>".

Context tenant_id must match principal.tenant_id.

Context emergency_service rule requires EmergencySupplyOfficer role.

Context manual_exception requires override_reason_code.

Context commit requires shortage_pool.status active or frozen.

Context release of consumed reservation is denied.

Policy denial emits SupplyAllocationPolicyDenied.

## 6. Ontology Projection Field Mapping

shortage_pool_id maps to ShortagePool.id.

product_id maps to Product.id.

location_scope_json maps to FulfillmentLocationScope.filter.

shortage_reason_code maps to ShortageCause.code.

allocation_rule_id maps to AllocationRule.id.

rule_method maps to AllocationRule.method.

customer_scope_json maps to CustomerSegmentScope.filter.

allocation_run_id maps to AllocationRun.id.

run_mode maps to AllocationRun.mode.

allocation_reservation_id maps to AllocationReservation.id.

allocated_quantity maps to AllocationReservation.quantity.

explanation_json maps to AllocationExplanation.drivers.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node ShortagePoolValidate checks constrained and demand quantities.

Node AllocationPolicyAuthorize evaluates Cedar.

Node DemandClaimsLoad reads demand by customer, channel, and region.

Node RuleSetLoad orders enabled allocation rules.

Node ProtectedMinimumApply reserves emergency and contract minimum quantities.

Node FairShareCompute distributes remaining quantity by weighted demand.

Node PriorityTierApply applies customer and channel priority.

Node ReservationBuild writes proposed reservations for simulation.

Node CommitReservations activates reservations in one transaction.

Node AuditSeal emits pool, run, and reservation events.

Branch InsufficientForMinimums raises shortage severity critical.

Branch ManualExceptionRequested routes to ComplianceAuditor.

Branch SimulationOnly returns proposed reservations without active holds.

Branch CommitConflict recomputes against latest shortage pool version.

Branch ReleaseDenied returns immutable consumed reservation error.

## 8. Audit Events

SupplyAllocationShortagePoolCreated records product, scope, and quantities.

SupplyAllocationRuleConfigured records rule method, rank, and scopes.

SupplyAllocationRunStarted records mode and rule set version.

SupplyAllocationProtectedMinimumApplied records protected quantity.

SupplyAllocationFairShareComputed records demand and allocation totals.

SupplyAllocationReservationProposed records reservation id and explanation.

SupplyAllocationReservationCommitted records active reservation id.

SupplyAllocationReservationReleased records reason and principal.

SupplyAllocationManualExceptionRequested records override reason.

SupplyAllocationPolicyDenied records Cedar decision id.

Events use EVT-SUPPLY_CHAIN_PLANNING-SUPPLY_ALLOCATION prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, source_system_id, and residency_pack.

## 9. SLO Targets

p50 allocation simulation latency: 250 ms for 100 demand claims.

p95 allocation simulation latency: 1800 ms for 10000 demand claims.

p99 allocation commit latency: 3200 ms for 10000 reservations.

Throughput target: 120 allocation simulations per tenant per hour.

Reservation read throughput target: 400 pages per minute per tenant.

Availability target: 99.95 percent monthly for simulation and commit APIs.

Rationale: shortage allocation is planner-facing but can block customer promise
answers during severe supply constraints.

## 10. Failure Modes + Recovery

Failure mode: demand claims exceed shortage pool by extreme outlier.

Recovery: cap claim at configured max and emit data quality event.

Failure mode: rule set has overlapping hard minimums.

Recovery: fail validation and require allocation manager repair.

Failure mode: commit conflicts with newer run.

Recovery: supersede stale run and recompute from latest pool version.

Failure mode: emergency_service rule used without role.

Recovery: deny with SupplyAllocationPolicyDenied.

Failure mode: audit-chain unavailable.

Recovery: block commit; simulations can run with degraded banner.

Failure mode: reservation release races with consumption.

Recovery: transaction checks reservation_state and denies consumed release.

## 11. Migration Notes with source vendor surfaces

SAP aATP source: product allocation object and characteristic catalog.

SAP aATP source: supply protection and allocation sequence.

SAP IBP-RP source: shortage response and constrained supply plan.

Oracle source: allocation rule and GOP supply allocation setup.

Kinaxis source: allocation workbook and scenario response rules.

Migration maps product allocation object to shortage_pool.

Migration maps allocation characteristic to customer and channel scopes.

Migration maps allocation sequence to rule_rank.

Migration preserves vendor allocation result in explanation_json.vendor.

Migration stores source rule checksum in audit evidence.

## 12. Cross-Microservice Handoffs

Handoff from ATP sends shortage signals and customer demand claims.

Handoff from deployment planning sends unresolved supply gaps.

Handoff from inventory provides constrained quantity.

Handoff from order-management consumes allocation reservations.

Handoff to ecommerce returns customer-safe allocation messages.

Handoff to workflow-engine routes manual exceptions.

Handoff to audit-chain emits pool, run, and reservation events.

Handoff to ontology projects ShortagePool and AllocationReservation.

Handoff to notification sends allocation exception alerts.

## 13. Intern Build Notes

Build step 01: create pool, rule, run, and reservation migrations.

Build step 02: add unique active pool constraint by product and scope.

Build step 03: add repository methods scoped by tenant and pool.

Build step 04: implement shortage pool quantity validation.

Build step 05: implement Cedar authorization for pool and run actions.

Build step 06: implement demand claims load port.

Build step 07: implement protected minimum calculation.

Build step 08: implement fair-share calculation.

Build step 09: implement priority tier adjustment.

Build step 10: implement explanation_json for each reservation.

Build step 11: implement simulation run write path.

Build step 12: implement commit transaction.

Build step 13: implement release transaction.

Build step 14: add Cedar fixture for emergency officer allow.

Build step 15: add Cedar fixture for manual exception denial.

Build step 16: write simulation test for fair share.

Build step 17: write simulation test for contract minimum.

Build step 18: write commit conflict test.

Build step 19: write release consumed denial test.

Build step 20: write audit fixture for reservation committed.

Build step 21: write audit fixture for policy denied.

Build step 22: add metric for unallocated quantity.

Build step 23: add metric for manual exception count.

Build step 24: verify no simulation creates active reservation.

Build step 25: verify tenant isolation with same customer code.

Build step 26: verify p95 simulation target with 10000 claims.

Build step 27: document SAP aATP characteristic mapping.

Build step 28: document Oracle GOP allocation mapping.

Build step 29: add rollback migration in child-first order.

Build step 30: attach PR evidence for API, policy, audit, and SLO checks.
