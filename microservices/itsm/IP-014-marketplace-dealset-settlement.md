---
doc_class: IP
ip_id: IP-014-marketplace-dealset-settlement
microservice: itsm
status: rewritten-wave-15-ip-substance
date: 2026-05-21
owner_team: axis-itsm + marketplace
counterparts: [ServiceNow ITSM, Jira Service Management, Freshservice]
source_artifacts:
  - microservices/itsm/contracts/openapi-v1.yaml
  - microservices/itsm/contracts/asyncapi-v1.yaml
  - microservices/itsm/manifest.json
  - microservices/itsm/src/domain/mod.rs
---

# IP-014 ITSM Marketplace DealSet Settlement

## A. Problem
ITSM service catalogs often become vendor-local marketplaces: ServiceNow Store apps, Atlassian Marketplace apps, and Freshservice orchestration integrations can bill or provision outside the tenant's governed settlement model. Oyatie needs ITSM service-catalog publish and request fulfillment to use DealSet settlement from ADR-0314.

The stamped IP repeated "marketplace DealSet" without naming the ITSM surfaces that settle.

## B. Approach
Bind DealSet to service-catalog and integration actions:

| ITSM action | DealSet role |
|---|---|
| `service-catalog-publish` | listing id + revenue share plan |
| `requests.submit` | purchase/entitlement check |
| provider adapter invoke | marketplace provider compensation |
| import migration | non-billable alias import unless a migration app is purchased |

The existing `contracts/openapi-v1.yaml` already requires `deal_set_id` on `ActionRequest`; this IP turns that into actual validation and evidence.

## C. Deliverables
- Validate `deal_set_id` in REST/gRPC command context for catalog and provider actions.
- Emit DealSet id in `ItsmActionAccepted` AsyncAPI payload where settlement applies.
- Add domain invariant for marketplace settlement on service-catalog publish.
- Add tests that catalog publish fails without DealSet for paid marketplace listings.
- Add dashboard metric for settlement-bound ITSM actions.

## D. Implementation
1. Classify ITSM actions into settlement-required, settlement-optional, and settlement-forbidden.
2. For settlement-required actions, reject missing or inactive `deal_set_id`.
3. Ensure `deal_set_id` is not required for core incident open or emergency acknowledgement.
4. Add marketplace port lookup to verify tenant entitlement and provider listing.
5. Add audit event dimensions: `deal_set_id`, listing id, provider id, tenant id, and action.
6. Update AsyncAPI payload docs to clarify when `deal_set_id` is required.
7. Add tests for service-catalog publish with valid, missing, and cross-tenant DealSet ids.
8. Add rollback behavior: disable marketplace catalog listing while core ITSM remains online.

## E. Acceptance
- Service-catalog publish cannot settle outside DealSet.
- Core incident, SLA, and change actions do not require unrelated marketplace ids.
- DealSet evidence appears in accepted events for marketplace-bound actions.
- Cross-tenant DealSet ids are denied before workflow execution.

## F. Evidence
- `contracts/openapi-v1.yaml` currently requires `deal_set_id` in `ActionRequest`.
- `contracts/asyncapi-v1.yaml` currently includes `deal_set_id` in `ActionAccepted`.
- `manifest.json` lists marketplace and billing as dependencies.
- ADR-0314 governs DealSet settlement.

## G. Counterparts
| Counterpart | Gap closed by this IP |
|---|---|
| ServiceNow Store / service catalog | Listings settle through Oyatie DealSet |
| Atlassian Marketplace for JSM | Marketplace app economics cannot bypass tenant governance |
| Freshservice Orchestration Center | Provider invocation has explicit settlement evidence |

## H. Cold-start buildability notes
- First classify which ITSM actions need settlement.
- Keep incident open and SLA recompute settlement-forbidden.
- Validate `deal_set_id` only after tenant and purpose parse.
- Add cross-tenant DealSet tests before provider integration.
- Emit settlement evidence only for marketplace-bound actions.
- Keep provider id and listing id redacted where needed.
- Use ADR-0314 vocabulary consistently.
- Do not create a new marketplace microservice path.
- Roll back by disabling listings, not core service desk actions.
- Keep catalog publish tied to service-catalog capability.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/itsm/contracts/asyncapi-v1.yaml`, `microservices/itsm/contracts/itsm-v1.proto`, `microservices/itsm/contracts/local-asyncapi-v1.yaml`, `microservices/itsm/contracts/local-openapi-v1.yaml`, `microservices/itsm/contracts/local-operations-v1.proto`, `microservices/itsm/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`openapi`, `asyncapi`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
