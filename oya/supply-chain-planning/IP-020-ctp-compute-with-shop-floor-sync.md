---
doc_class: ImplementationPlan
ip_id: IP-020
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
sap_submodule_equivalents: [CTP capable-to-promise, PP/DS finite scheduling, aATP supply creation]
---

# IP-020: CTP compute with shop-floor sync

## 1. Context with why, journey leg, named persona

This IP defines capable-to-promise compute that checks whether constrained
production capacity and component supply can support a promise date.

Why this matters: ATP can confirm stock, but make-to-order and constrained
items require shop-floor capacity, component availability, and schedule health
before a promise is safe.

Journey leg: j123 leg 07, "simulate launch capacity promise with shop-floor sync before commitment".

Named persona: Omar, production planner for configured industrial pumps.

Omar needs CTP to answer sales requests using current shop-floor status without
overwriting the released PP/DS schedule.

SAP equivalent: PP/DS finite scheduling with CTP and aATP supply creation.

Oracle equivalent: Global Order Promising capable-to-promise.

Microsoft equivalent: Planning Optimization capable-to-promise with production.

The feature returns promise feasibility and optional planned order proposals.

The feature does not dispatch work orders to machines.

The feature belongs to ADR-0105 kernel, application, adapter, worker, and
governance layers.

## 2. Scope

In scope: CTP request validation.

In scope: bill of material and routing snapshot load.

In scope: shop-floor capacity snapshot sync.

In scope: finite capacity feasibility check.

In scope: component availability check.

In scope: planned order proposal output.

Out of scope: detailed optimizer replacement for PP/DS.

Out of scope: MES command execution.

Out of scope: labor payroll scheduling.

Out of scope: supplier purchase order creation.

## 3. Data Model Deltas

Create table scp_ctp_request.

Column tenant_id: uuid, required, partition key.

Column ctp_request_id: uuid, required.

Column request_source: enum, values sales_order, quote, planner_simulation,
api_partner.

Column product_id: text, required.

Column plant_id: text, required.

Column requested_quantity: numeric(18,3), required.

Column requested_date: date, required.

Column configuration_json: jsonb, nullable.

Column allow_planned_order_proposal: boolean, required.

Column policy_bundle_version: text, required.

Column created_by_principal_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_shop_floor_capacity_snapshot.

Column tenant_id: uuid, required.

Column capacity_snapshot_id: uuid, required.

Column plant_id: text, required.

Column work_center_id: text, required.

Column bucket_start: timestamptz, required.

Column bucket_end: timestamptz, required.

Column available_capacity_minutes: numeric(18,3), required.

Column committed_capacity_minutes: numeric(18,3), required.

Column downtime_minutes: numeric(18,3), required.

Column snapshot_source: enum, values mes, ppds, manual_override.

Column source_event_time: timestamptz, required.

Column evidence_hash: text, required.

Create table scp_ctp_feasibility_result.

Column tenant_id: uuid, required, partition key.

Column ctp_result_id: uuid, required.

Column ctp_request_id: uuid, required.

Column promise_state: enum, values feasible, feasible_later, infeasible,
component_short, capacity_short, policy_denied.

Column confirmed_quantity: numeric(18,3), required.

Column confirmed_date: date, nullable.

Column bottleneck_work_center_id: text, nullable.

Column bottleneck_component_id: text, nullable.

Column explanation_json: jsonb, required.

Column audit_id: uuid, required.

Column ontology_projection_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_ctp_planned_order_proposal.

Column tenant_id: uuid, required.

Column planned_order_proposal_id: uuid, required.

Column ctp_result_id: uuid, required.

Column product_id: text, required.

Column plant_id: text, required.

Column proposed_start_time: timestamptz, required.

Column proposed_finish_time: timestamptz, required.

Column proposed_quantity: numeric(18,3), required.

Column routing_version: text, required.

Column bom_version: text, required.

Column proposal_state: enum, values proposed, accepted, rejected, expired.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/ctp/check

REST GET /v1/supply-chain-planning/ctp/results/{ctp_result_id}

REST POST /v1/supply-chain-planning/ctp/planned-order-proposals/{id}/accept

REST POST /v1/supply-chain-planning/ctp/planned-order-proposals/{id}/reject

gRPC CtpComputeService.CheckCapability accepts CheckCapabilityRequest.

gRPC CtpComputeService.GetResult accepts GetCtpResultRequest.

gRPC CtpComputeService.DecidePlannedOrderProposal accepts DecideProposalRequest.

Example request:

```json
{
  "tenant_id": "55555555-5555-5555-5555-555555555555",
  "request_source": "quote",
  "product_id": "pump-config-a",
  "plant_id": "plant-ulsan",
  "requested_quantity": 4,
  "requested_date": "2026-06-10",
  "configuration": {"motor": "high-torque", "seal": "food-grade"},
  "allow_planned_order_proposal": true
}
```

Example response:

```json
{
  "ctp_result_id": "ctp-020-0001",
  "promise_state": "feasible_later",
  "confirmed_quantity": 4,
  "confirmed_date": "2026-06-13",
  "bottleneck_work_center_id": "wc-assembly-2",
  "planned_order_proposal_id": "pop-020-0001",
  "audit_id": "aud-scp-ctp-020"
}
```

Shop-floor sync endpoint: REST POST /v1/supply-chain-planning/ctp/capacity-snapshots.

Capacity snapshot writes require source_system_id and evidence_hash.

All proposal decisions are idempotent.

## 5. Cedar Policy Hooks

Principal type: ProductionPlanner, CustomerPromiseClient, PlantScheduler.

Action: scp::Action::"CheckCtp".

Action: scp::Action::"WriteShopFloorCapacitySnapshot".

Action: scp::Action::"AcceptCtpPlannedOrderProposal".

Resource: scp::CtpResource::"<tenant_id>/<plant_id>/<product_id>".

Context tenant_id must equal principal.tenant_id.

Context plant_id must be in principal.plant_scope.

Context request_source api_partner requires partner_ctp_allowed.

Context write capacity snapshot requires source_system_trusted.

Context accept planned order requires PlantScheduler role.

Policy denial emits CtpComputePolicyDenied.

## 6. Ontology Projection Field Mapping

ctp_request_id maps to CtpRequest.id.

product_id maps to Product.id.

plant_id maps to ManufacturingPlant.id.

configuration_json maps to ProductConfiguration.attributes.

capacity_snapshot_id maps to CapacitySnapshot.id.

work_center_id maps to WorkCenter.id.

available_capacity_minutes maps to CapacityBucket.available_minutes.

ctp_result_id maps to CtpResult.id.

promise_state maps to CtpResult.state.

bottleneck_work_center_id maps to ConstraintBottleneck.work_center.

bottleneck_component_id maps to ConstraintBottleneck.component.

planned_order_proposal_id maps to PlannedOrderProposal.id.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node RequestNormalize validates product, plant, quantity, and date.

Node CtpPolicyAuthorize evaluates Cedar.

Node BomSnapshotLoad loads effective BOM for requested configuration.

Node RoutingSnapshotLoad loads routing and work center requirements.

Node ShopFloorSnapshotLoad reads latest capacity snapshot.

Node ComponentAvailabilityCheck checks component supply by need date.

Node FiniteCapacitySimulate places load into capacity buckets.

Node BottleneckExplain identifies limiting work center or component.

Node PlannedOrderProposalBuild builds non-executing proposal.

Node AuditSeal emits request, result, and proposal events.

Branch CapacitySnapshotStale rejects state-changing proposal creation.

Branch ComponentShort returns component_short with purchase handoff.

Branch CapacityShort returns capacity_short with bottleneck detail.

Branch FeasibleLater returns later date and optional proposal.

Branch PolicyDenied returns policy_denied without BOM or routing detail.

## 8. Audit Events

CtpCheckRequested records product, plant, quantity, and date.

CtpShopFloorCapacitySnapshotWritten records plant and work center counts.

CtpBomRoutingSnapshotLoaded records version identifiers.

CtpComponentAvailabilityChecked records component count and shortage count.

CtpFiniteCapacitySimulated records bucket count and bottleneck id.

CtpPromiseComputed records promise state and confirmed date.

CtpPlannedOrderProposalCreated records proposal id and routing version.

CtpPlannedOrderProposalAccepted records scheduler and handoff id.

CtpPlannedOrderProposalRejected records reason code.

CtpComputePolicyDenied records Cedar decision id.

Events use EVT-SUPPLY_CHAIN_PLANNING-CTP_COMPUTE prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, source_system_id, and residency_pack.

## 9. SLO Targets

p50 CTP check latency: 180 ms for cached BOM, routing, and capacity snapshot.

p95 CTP check latency: 900 ms for configured product with 50 components.

p99 CTP check latency: 2200 ms under 100 concurrent plant requests.

Throughput target: 600 CTP checks per minute per tenant.

Capacity snapshot ingest throughput: 10000 work-center buckets per minute.

Availability target: 99.95 percent monthly for CTP check API.

Rationale: CTP is synchronous for quoting, but capacity simulation is heavier
than ATP and must remain bounded by plant scope.

## 10. Failure Modes + Recovery

Failure mode: capacity snapshot stale.

Recovery: return infeasible with stale_snapshot reason and block proposals.

Failure mode: BOM version missing.

Recovery: reject request with product master data error.

Failure mode: routing has work center not in capacity snapshot.

Recovery: mark capacity_short and open data quality handoff.

Failure mode: component availability service unavailable.

Recovery: fail closed for state-changing proposals.

Failure mode: proposal accepted after new shop-floor snapshot.

Recovery: re-run feasibility before accepting.

Failure mode: audit-chain unavailable.

Recovery: block planned order proposal decisions and retry audit seal.

## 11. Migration Notes with source vendor surfaces

SAP PP/DS source: PDS, resource capacity, and finite schedule buckets.

SAP aATP source: product availability and supply creation integration.

SAP MES source: work center downtime and completion confirmations.

Oracle source: GOP CTP rules and manufacturing capacity collections.

Microsoft source: route, resource, BOM, and master planning capacity data.

Migration maps SAP resource to WorkCenter.id.

Migration maps PDS version to routing_version and bom_version.

Migration maps finite schedule bucket to capacity snapshot bucket.

Migration preserves vendor planned order number only as external_reference.

Migration stores source capacity checksum in evidence_hash.

## 12. Cross-Microservice Handoffs

Handoff from product-master provides BOM and routing versions.

Handoff from shop-floor/MES provides capacity snapshots.

Handoff from inventory provides component availability.

Handoff from procurement receives component shortage signals.

Handoff from ATP escalates stock-unavailable requests to CTP.

Handoff to production-planning receives accepted proposal.

Handoff to order-management receives CTP promise answer.

Handoff to audit-chain emits promise and proposal decision events.

Handoff to ontology projects CtpResult and WorkCenter bottlenecks.

## 13. Intern Build Notes

Build step 01: create request, capacity, result, and proposal migrations.

Build step 02: add repository methods by tenant, plant, product, and date.

Build step 03: implement capacity snapshot ingest validation.

Build step 04: implement RequestNormalize with configuration hash.

Build step 05: implement Cedar authorization before BOM load.

Build step 06: implement BOM snapshot port.

Build step 07: implement routing snapshot port.

Build step 08: implement component availability port.

Build step 09: implement finite capacity bucket simulation.

Build step 10: implement bottleneck explanation builder.

Build step 11: implement planned order proposal creation.

Build step 12: implement proposal accept re-check.

Build step 13: add Cedar fixture for plant scheduler allow.

Build step 14: add Cedar fixture for api partner denial.

Build step 15: add Cedar fixture for untrusted capacity source denial.

Build step 16: write contract test for feasible CTP.

Build step 17: write contract test for feasible later CTP.

Build step 18: write contract test for component short.

Build step 19: write contract test for capacity snapshot stale.

Build step 20: write audit fixture for CtpPromiseComputed.

Build step 21: write audit fixture for capacity snapshot write.

Build step 22: verify proposals are non-executing until accepted.

Build step 23: verify accept re-runs feasibility.

Build step 24: verify tenant isolation with same plant code.

Build step 25: verify p95 latency with 50 component fixture.

Build step 26: verify capacity ingest throughput target.

Build step 27: document SAP PP/DS PDS mapping.

Build step 28: document MES downtime mapping.

Build step 29: add rollback migration in child-first order.

Build step 30: attach PR evidence for API, policy, audit, and SLO checks.
