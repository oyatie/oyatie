---
doc_class: ImplementationPlan
ip_id: IP-019
microservice: supply-chain-planning
related_adrs: [ADR-0105, ADR-0243, ADR-0244, ADR-0253, ADR-0263, ADR-0315, ADR-0131, ADR-0132]
journey_id: j112-tenant-to-tenant-rfq-and-bid
journey_link: docs/user-journeys/j112-tenant-to-tenant-rfq-and-bid/story.md
status: Accepted
date: 2026-05-20
owner: axis-supply-chain-planning + axis-erp-parity
tenant_class: paid
billing_components:
  - per_usage
sap_submodule_equivalents: [ATP availability check, aATP product availability, IBP-RP response confirmation]
---

# IP-019: ATP compute

## 1. Context with why, journey leg, named persona

This IP defines available-to-promise compute for requested product, location,
quantity, customer, and date combinations.

Why this matters: sales, ecommerce, service, and allocation flows need a
deterministic promise answer that respects supply, reservations, allocations,
tenant policy, and audit evidence.

Journey leg: j112 leg 06, "answer a tenant-to-tenant promise request before bid commitment".

Named persona: Elena, customer service lead handling enterprise order promises.

Elena needs an immediate confirmed quantity and date, plus a clear explanation
when the requested date cannot be promised.

SAP equivalent: classic ATP availability check and S/4HANA aATP product
availability check.

Oracle equivalent: Global Order Promising availability check.

Microsoft equivalent: Dynamics 365 available-to-promise calculation.

The feature computes promise answers; it does not create sales orders.

The feature belongs to ADR-0105 kernel, application, API, and governance bands.

The feature must be safe for high-volume synchronous requests.

## 2. Scope

In scope: ATP request validation.

In scope: supply bucket load and reservation subtraction.

In scope: allocation consumption check.

In scope: substitute location and later-date alternatives.

In scope: explanation and audit event emission.

In scope: soft hold creation when caller requests hold.

Out of scope: CTP finite capacity scheduling.

Out of scope: pricing, tax, or payment authorization.

Out of scope: carrier date promise.

Out of scope: final order capture.

## 3. Data Model Deltas

Create table scp_atp_request.

Column tenant_id: uuid, required, partition key.

Column atp_request_id: uuid, required.

Column request_source: enum, values sales_order, ecommerce_cart, service_case,
planner_simulation, api_partner.

Column product_id: text, required.

Column location_id: text, required.

Column customer_id: text, nullable.

Column requested_quantity: numeric(18,3), required.

Column requested_date: date, required.

Column unit_of_measure: text, required.

Column allow_alternatives: boolean, required.

Column request_hold: boolean, required.

Column policy_bundle_version: text, required.

Column created_at: timestamptz, required.

Create table scp_atp_supply_bucket.

Column tenant_id: uuid, required.

Column product_id: text, required.

Column location_id: text, required.

Column supply_date: date, required.

Column supply_type: enum, values on_hand, purchase_order, production_order,
deployment_inbound, transfer_inbound, return_expected.

Column gross_quantity: numeric(18,3), required.

Column reserved_quantity: numeric(18,3), required.

Column allocation_reserved_quantity: numeric(18,3), required.

Column atp_quantity: numeric(18,3), required.

Column source_system_id: text, required.

Column evidence_hash: text, required.

Create table scp_atp_result.

Column tenant_id: uuid, required, partition key.

Column atp_result_id: uuid, required.

Column atp_request_id: uuid, required.

Column promise_state: enum, values full, partial, none, alternative, policy_denied.

Column confirmed_quantity: numeric(18,3), required.

Column confirmed_date: date, nullable.

Column confirmed_location_id: text, nullable.

Column short_quantity: numeric(18,3), required.

Column explanation_json: jsonb, required.

Column audit_id: uuid, required.

Column ontology_projection_id: uuid, required.

Column created_at: timestamptz, required.

Create table scp_atp_soft_hold.

Column tenant_id: uuid, required.

Column soft_hold_id: uuid, required.

Column atp_result_id: uuid, required.

Column product_id: text, required.

Column location_id: text, required.

Column held_quantity: numeric(18,3), required.

Column expires_at: timestamptz, required.

Column hold_state: enum, values active, consumed, expired, released.

Column audit_event_class: text, required.

## 4. API Endpoints

REST POST /v1/supply-chain-planning/atp/check

REST GET /v1/supply-chain-planning/atp/results/{atp_result_id}

REST POST /v1/supply-chain-planning/atp/soft-holds/{soft_hold_id}/release

gRPC AtpComputeService.CheckAvailability accepts CheckAvailabilityRequest.

gRPC AtpComputeService.GetResult accepts GetAtpResultRequest.

gRPC AtpComputeService.ReleaseSoftHold accepts ReleaseSoftHoldRequest.

Example request:

```json
{
  "tenant_id": "44444444-4444-4444-4444-444444444444",
  "request_source": "ecommerce_cart",
  "product_id": "sku-echo-10",
  "location_id": "dc-seoul",
  "customer_id": "cust-kr-887",
  "requested_quantity": 12,
  "requested_date": "2026-05-23",
  "allow_alternatives": true,
  "request_hold": true
}
```

Example response:

```json
{
  "atp_result_id": "atp-019-0001",
  "promise_state": "partial",
  "confirmed_quantity": 8,
  "confirmed_date": "2026-05-23",
  "short_quantity": 4,
  "soft_hold_id": "hold-019-0001",
  "audit_id": "aud-scp-atp-019"
}
```

The API returns alternatives ordered by date, location, and policy score.

Idempotency key: request_source plus caller_order_ref plus product and date.

Soft hold release is idempotent.

## 5. Cedar Policy Hooks

Principal type: OrderPromiseClient, CustomerServiceAgent, PlannerSimulationUser.

Action: scp::Action::"CheckAtp".

Action: scp::Action::"CreateAtpSoftHold".

Action: scp::Action::"ReleaseAtpSoftHold".

Resource: scp::AtpResource::"<tenant_id>/<product_id>/<location_id>".

Context tenant_id must equal principal.tenant_id.

Context customer_id must be in principal.customer_scope when present.

Context request_source api_partner requires partner contract claim.

Context request_hold requires hold_allowed.

Context alternative_location_id must be in principal.location_scope.

Policy denial emits AtpComputePolicyDenied.

Allowed hold emits AtpSoftHoldCreated.

## 6. Ontology Projection Field Mapping

atp_request_id maps to AtpRequest.id.

request_source maps to AtpRequest.source.

product_id maps to Product.id.

location_id maps to FulfillmentLocation.id.

customer_id maps to Customer.id.

requested_quantity maps to Quantity.amount.

requested_date maps to PromiseRequest.requested_date.

atp_result_id maps to AtpResult.id.

promise_state maps to AtpResult.state.

confirmed_quantity maps to PromiseConfirmation.quantity.

confirmed_date maps to PromiseConfirmation.date.

soft_hold_id maps to InventorySoftHold.id.

audit_id maps to AuditEvidence.id.

## 7. Workflow Steps

Node RequestNormalize validates units and dates.

Node AtpPolicyAuthorize evaluates Cedar.

Node SupplyBucketLoad reads eligible supply buckets.

Node ReservationSubtract subtracts active reservations and holds.

Node AllocationCheck subtracts allocation-reserved quantities.

Node PromiseSearch finds same-location full or partial promise.

Node AlternativeSearch runs only when allow_alternatives is true.

Node ExplanationBuild creates shortage and alternative reasons.

Node SoftHoldCreate runs only when request_hold is true.

Node AuditSeal emits request, result, and hold events.

Branch FullPromise returns full state and optional hold.

Branch PartialPromise returns partial state and shortage explanation.

Branch NoPromise returns none with first available future date.

Branch AlternativePromise returns alternative location or date.

Branch PolicyDenied returns policy_denied without supply details.

## 8. Audit Events

AtpCheckRequested records source, product, location, quantity, and date.

AtpSupplyBucketRead records bucket count and snapshot hash.

AtpPromiseComputed records promise state and confirmed quantity.

AtpAlternativeGenerated records alternative count and top alternative.

AtpSoftHoldCreated records held quantity and expiry.

AtpSoftHoldReleased records release principal and reason.

AtpSoftHoldExpired records expiry time and held quantity.

AtpComputePolicyDenied records Cedar decision id.

AtpResultProjected records ontology projection id.

Events use EVT-SUPPLY_CHAIN_PLANNING-ATP_COMPUTE prefixes.

Every event carries tenant_id, principal_id, trace_id, audit_id,
policy_bundle_version, source_system_id, and residency_pack.

## 9. SLO Targets

p50 ATP check latency: 45 ms for one product-location request.

p95 ATP check latency: 180 ms with allocation and alternatives enabled.

p99 ATP check latency: 450 ms under 200 concurrent tenant requests.

Throughput target: 2000 ATP checks per minute per tenant.

Soft hold throughput target: 600 hold writes per minute per tenant.

Availability target: 99.99 percent monthly for check API.

Rationale: ATP is synchronous customer-facing infrastructure and must answer
within transaction flow latency budgets.

## 10. Failure Modes + Recovery

Failure mode: supply bucket snapshot missing.

Recovery: return none with degraded reason and emit data gap event.

Failure mode: allocation service unavailable.

Recovery: fail closed for allocated products and return policy-safe message.

Failure mode: soft hold write conflict.

Recovery: recompute ATP inside transaction and retry once.

Failure mode: unit conversion missing.

Recovery: reject request with validation error before supply read.

Failure mode: audit-chain unavailable.

Recovery: return result only when non-state-changing; block soft hold.

Failure mode: partner exceeds request rate.

Recovery: throttle and emit AtpComputeRateLimited.

## 11. Migration Notes with source vendor surfaces

SAP ECC source: ATP check scope and checking group.

SAP S/4HANA source: aATP product availability and confirmations.

SAP IBP-RP source: response confirmation quantities.

Oracle source: Global Order Promising availability rules.

Microsoft source: master planning ATP settings and inventory dimensions.

Migration maps SAP checking group to policy context.

Migration maps ATP category to supply_type.

Migration maps confirmation schedule line to atp_result.

Migration preserves vendor promise explanation in explanation_json.vendor.

Migration stores vendor extract hash in evidence_hash.

## 12. Cross-Microservice Handoffs

Handoff from inventory provides supply buckets and reservations.

Handoff from allocation provides allocation-reserved quantities.

Handoff from deployment planning provides inbound transfer supply.

Handoff from production planning provides production order supply.

Handoff to order-management returns promise result and soft hold id.

Handoff to ecommerce returns customer-safe promise message.

Handoff to audit-chain emits request, result, and hold events.

Handoff to ontology projects AtpRequest and AtpResult.

Handoff to abuse-defence shares partner throttling events.

## 13. Intern Build Notes

Build step 01: create request, supply bucket, result, and hold migrations.

Build step 02: add repository methods keyed by tenant and product-location.

Build step 03: implement RequestNormalize validation.

Build step 04: implement Cedar authorization before supply reads.

Build step 05: implement supply bucket query by date range.

Build step 06: implement reservation and hold subtraction.

Build step 07: implement allocation subtraction.

Build step 08: implement same-location promise search.

Build step 09: implement alternative location search.

Build step 10: implement explanation_json builder.

Build step 11: implement soft hold transaction.

Build step 12: implement hold release idempotency.

Build step 13: add Cedar fixture for ecommerce ATP allow.

Build step 14: add Cedar fixture for partner contract denial.

Build step 15: add Cedar fixture for hold disallowed.

Build step 16: write contract test for full promise.

Build step 17: write contract test for partial promise.

Build step 18: write contract test for alternative promise.

Build step 19: write transaction test for hold conflict.

Build step 20: write audit fixture for AtpPromiseComputed.

Build step 21: write audit fixture for AtpSoftHoldCreated.

Build step 22: add p99 load fixture for concurrent requests.

Build step 23: verify no policy-denied response leaks supply buckets.

Build step 24: verify audit_id exists on every result.

Build step 25: verify tenant isolation with identical SKU codes.

Build step 26: verify soft hold expiry worker emits expiry event.

Build step 27: document SAP checking group mapping.

Build step 28: document Oracle promising rule mapping.

Build step 29: add rollback migration in child-first order.

Build step 30: attach PR evidence for API, policy, audit, and SLO checks.
