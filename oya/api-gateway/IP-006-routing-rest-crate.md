# IP-006: `oya-api-gateway-routing-rest` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

REST surface (OpenAPI 3.2.0) for the routing control plane. Tenant operators consume this to read route configurations + propose changes.

## B — Surface

Defined in `contracts/api-gateway.openapi.yaml`.

- `GET /v1/routes` — list.
- `GET /v1/routes/{id}` — describe.
- `POST /v1/routes/{id}/canary` — shift canary weight (gated by Cedar `ci-scope.cedar`).
- `POST /v1/routes/{id}/bluegreen/swap` — swap blue/green (dual-approval).

## C — Acceptance criteria

- OpenAPI 3.2.0 validated by `oya-governance-openapi-validate`.
- ≥85% coverage on handler logic.
- Tonic OpenAPI auto-gen from `contracts/`.

## Wave 15 A-G substance

### A - Problem
The REST surface is the external management and admission contract readers inspect first, so it must not imply unsupported route-product or usage-plan management.

### B - Approach
Implement `oya-api-gateway-routing-rest` from `catalog/oya-api-gateway-routing-rest.yaml` as an Axum REST surface backed by `contracts/api-gateway.openapi.yaml`, covering contract-backed admission and route describe/list behavior first.

### C - Deliverables
- Axum handlers for admission check, route describe/list where present in OpenAPI, and health/readiness.
- OpenAPI-aligned request/response structs.
- Error mapper for tenant/cell/policy denial, malformed route, stale cell, and unavailable route table.
- Middleware for trace-context and audit correlation.
- Contract tests round-tripping OpenAPI examples.

### D - Ordered implementation steps
1. Map DTOs from the existing OpenAPI contract.
2. Wire admission handlers to routing-usecase.
3. Add deny/error response mapping and correlation headers.
4. Add handler tests for allowed, denied, malformed, and unavailable states.
5. Add OpenAPI conformance tests for examples and required fields.
6. Document broader management-surface gaps from `feature-parity-matrix-2026-05-20.md` as non-goals.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-rest` passes.
- OpenAPI examples validate and include required tenant/cell/route/request fields.
- REST deny responses avoid leaking upstream internals.
- SLO labels align with edge latency and availability OpenSLO files.

### F - Evidence
Grounding files: `contracts/api-gateway.openapi.yaml`, `catalog/oya-api-gateway-routing-rest.yaml`, `PRD.md`, `ARCHITECTURE.md`, `feature-parity-matrix-2026-05-20.md`, `policy/route-authorization.cedar`, and `slos/edge-latency-p95.openslo.yaml`.

### G - Counterpart comparison
AWS API Gateway, Kong Admin API, and Apigee expose broad management surfaces. This IP covers Oyatie's current contract-backed admission surface and does not overclaim missing usage-plan, developer-app, or API-product resources.

## Remediation notes

- GitLab API ingress is the concrete counterpart for REST because route describe/list, route admission, token failures, and rate-limit responses must be inspectable by operators and client libraries.
- REST handlers must remain contract-led by `contracts/api-gateway.openapi.yaml`; adding endpoints before OpenAPI examples and error schemas exist creates unsupported management-surface claims.
- Handler tests should cover GitLab-style project/group route paths, malformed token context, stale cell headers, and rate-limit denial response headers.
- This IP should keep non-goals explicit: full API-product catalogs, developer-app lifecycle, and usage-plan administration are not implied by route admission REST coverage.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| OpenAPI source | `contracts/api-gateway.openapi.yaml` | Request/response DTOs are generated or manually aligned. |
| Admission response | `contracts/api-gateway.openapi.yaml` | Admit, deny, malformed, and unavailable responses are represented. |
| Header behavior | `PRD.md` | Trace, tenant, cell, and retry headers are preserved or emitted. |
| Policy denial | `policy/route-authorization.cedar` | REST denial does not leak upstream internals. |
| Rate limit denial | `policy/rate-limit.cedar` | `RateLimit-*` and `Retry-After` headers match domain decisions. |
| GitLab-style paths | GitLab API ingress | Nested group/project route fixtures remain deterministic. |
| Contract examples | `contracts/api-gateway.openapi.yaml` | Examples round-trip through handler tests. |
| SLO labels | `slos/edge-latency-p95.openslo.yaml` | REST latency labels map to edge latency objectives. |
| Non-goal guard | `feature-parity-matrix-2026-05-20.md` | Missing API-product resources are listed as gaps, not implied support. |
| Audit correlation | `contracts/api-gateway.asyncapi.yaml` | REST response carries request ID used by audit events. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-006-routing-rest-crate.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-006-routing-rest-crate.md`.
