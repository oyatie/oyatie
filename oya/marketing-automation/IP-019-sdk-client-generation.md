---
doc_class: ImplementationPlan
ip_id: IP-019-sdk-client-generation
microservice: marketing-automation
bounded_contexts: [rest, grpc, asyncapi, sdk]
related_adrs: [ADR-0105, ADR-0244, ADR-0253-amendment, ADR-0263, ADR-0321, ADR-0328]
status: proposed
date: 2026-05-21
owner: axis-marketing-automation + developer-experience
tenant_class_aware: true
---

# IP-019: SDK Client Generation

## A. Problem

The service lists REST, gRPC, AsyncAPI, and SDK layers, but stamped IP-019 did not identify which clients are generated or how drift is prevented. HubSpot, Marketo, and Mailchimp all expose developer APIs and SDKs; Oyatie needs generated clients that preserve tenant headers, idempotency keys, HTTP/3 transport expectations, and typed marketing objects without hand-written divergence.

## B. Approach

Use `sdk-plan.md`, `contracts/openapi-v1.yaml`, `contracts/marketing-automation-v1.proto`, `contracts/asyncapi-v1.yaml`, and `src/adapter/*` registries as the SDK generation sources. Generate tenant-facing REST SDKs from OpenAPI and internal clients from proto; AsyncAPI fixtures feed event consumer examples.

## C. Deliverables

| Artifact | Change |
|---|---|
| `sdk-plan.md` | Define generated SDK languages, supported auth, idempotency behavior, and versioning. |
| `contracts/openapi-v1.yaml` | Add operation ids and schemas specific enough for generated clients. |
| `contracts/marketing-automation-v1.proto` | Provide internal client stubs for workers and workflow-engine. |
| `contracts/asyncapi-v1.yaml` | Provide event payload schemas for typed consumers. |
| `tests/integration.rs` | Add contract fixture checks so generated SDK examples match server stubs. |

## D. Implementation

1. Normalize OpenAPI paths with stable `operationId` values for capability listing and action invocation.
2. Add idempotency and request id headers to OpenAPI so generated SDKs cannot omit them.
3. Generate Rust SDK as the primary repo-owned SDK surface; document other languages only if generation lanes exist.
4. Generate gRPC internal client from proto for worker/workflow-engine consumers.
5. Add examples for `LaunchJourney`, `SyncSegment`, `EnforceSuppression`, and `RollupAttribution`.
6. Verify generated clients include tenant id, principal id, and tenant_class handling as gateway-derived context rather than caller-supplied trust.
7. Add SDK version compatibility note: contract minor versions are backward compatible; major versions require migration guide.

## E. Acceptance

- `cargo test -p oya-marketing-automation-campaign-journey-app scaffold_declares_expected_contracts`
- `cargo run -p oya-dev-cli -- gate validate openapi-contract-binding --microservice marketing-automation`
- `cargo run -p oya-dev-cli -- gate validate sdk-plan --microservice marketing-automation`
- Manual evidence: SDK examples compile against the current OpenAPI/proto schemas.

## F. Evidence

- Local docs: `sdk-plan.md`.
- Local contracts: `contracts/openapi-v1.yaml`, `contracts/marketing-automation-v1.proto`, `contracts/asyncapi-v1.yaml`.
- Local source: `src/adapter/http.rs`, `src/adapter/grpc.rs`, `src/adapter/asyncapi.rs`.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| HubSpot Marketing Hub | Private-app API usage gets generated tenant-safe clients. |
| Adobe Marketo Engage | REST/Bulk-style operations get typed client coverage. |
| Mailchimp | Audience, journey, and webhook APIs can be consumed without ad hoc HTTP code. |

## H. Local Traceability

- SDK plan: `sdk-plan.md`.
- REST contract: `contracts/openapi-v1.yaml`.
- gRPC contract: `contracts/marketing-automation-v1.proto`.
- Event contract: `contracts/asyncapi-v1.yaml`.
- HTTP registry: `src/adapter/http.rs`.
- gRPC registry: `src/adapter/grpc.rs`.
- Async registry: `src/adapter/asyncapi.rs`.
- Required header: tenant id.
- Required header: principal id.
- Required header: request id.
- Required header: idempotency key.
- Example command: `LaunchJourney`.
- Example command: `SyncSegment`.
- Example command: `EnforceSuppression`.
- Example command: `RollupAttribution`.
- Compatibility rule: minor contract versions are backward compatible.
- Failure state: generated SDK trusts client-supplied tenant_class.
- Failure state: SDK example compiles against stale contract.

## API Versioning (per ADR-0342)

- contract_surface: [`microservices/marketing-automation/contracts/asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-asyncapi-v1.yaml`, `microservices/marketing-automation/contracts/local-openapi-v1.yaml`, `microservices/marketing-automation/contracts/local-operations-v1.proto`, `microservices/marketing-automation/contracts/marketing-automation-v1.proto`, `microservices/marketing-automation/contracts/openapi-v1.yaml`]; detected_types: OpenAPI, AsyncAPI, proto3; trigger_terms: [`asyncapi`, `openapi`, `.proto`].
- carrier: `YYYY-MM-DD` via header `Oyatie-Version`, URL prefix `/v/<date>/`, and proto3 envelope field tag `8001`.
- declared_version: `2026-05-21`; supported_window: latest `N=3` public date versions for `>=180` days.
- internal_mesh_exemption: internal gRPC remains unaffected per ADR-0145; this section applies at public contract boundaries.
