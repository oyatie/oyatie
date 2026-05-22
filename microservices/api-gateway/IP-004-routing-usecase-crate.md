# IP-004: `oya-api-gateway-routing-usecase` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

Use-case layer. Coordinates kernel ports to execute one request-routing decision.

## B — Use case shape

```rust
pub struct RouteRequestUseCase<L, R, A>
where L: RouteLookupPort, R: CellResidencyPort, A: AuditEmitPort {
    lookup: L, residency: R, audit: A,
}

impl<L, R, A> RouteRequestUseCase<L, R, A> {
    pub async fn execute(&self, ctx: RequestContext) -> RouteDecision { /* pure orchestration */ }
}
```

## C — Acceptance criteria

- No I/O; all dependencies injected via traits.
- 95% line coverage.
- Property tests on `execute` invariants.

## D — Dependencies

- `oya-api-gateway-routing-domain`, `oya-api-gateway-routing-kernel`.

## E — References

- ADR-0056 + ADR-0105

## Wave 15 A-G substance

### A - Problem
The gateway needs one orchestrator that turns a normalized request envelope into admit, deny, or fallback without mixing protocol parsing into residency and audit order.

### B - Approach
Implement `oya-api-gateway-routing-usecase` as the application usecase named in `catalog/oya-api-gateway-routing-usecase.yaml`, consuming kernel ports and returning decisions usable by REST, gRPC, and worker surfaces.

### C - Deliverables
- `AdmitRequest` usecase with request envelope, principal context, route candidate, and route class.
- Ordered pipeline: normalize, lookup, check cell epoch, enforce pack/region, apply route class, produce audit intent.
- Fail-closed outcomes for missing tenant cell, isolated cell, stale epoch, unauthorized action, and no route.
- HTTP/gRPC-safe decision result.
- Fixtures for public, tenant, partner, admin, and machine routes.

### D - Ordered implementation steps
1. Define input/output DTOs using kernel/domain types.
2. Wire lookup, residency, and audit-intent ports in order.
3. Add deny-reason mapping to status codes.
4. Add route-class tests from `policy/route-authorization.cedar`.
5. Add cell-aware fail-closed tests from `ARCHITECTURE.md`.
6. Share integration fixtures with routing-rest and routing-grpc.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-usecase` passes.
- No upstream route is returned before residency and route authorization pass.
- Denies include audit intent for `oya.api_gateway.request.denied`.
- Decisions preserve trace and `x-oya-cell-id` requirements from `PRD.md`.

### F - Evidence
Grounding files: `PRD.md`, `ARCHITECTURE.md`, `catalog/oya-api-gateway-routing-usecase.yaml`, `policy/route-authorization.cedar`, `policy/tenant-scope.cedar`, `policy/sov-cloud-overlay.cedar`, and `contracts/api-gateway.openapi.yaml`.

### G - Counterpart comparison
AWS method execution, Kong plugin chains, and Apigee policy flows all impose ordered admission. Oyatie's usecase matches that ordering while making Cedar deny, cell residency, SPIFFE upstream, and audit-chain intent first-class.

## Remediation notes

- Salesforce API ingress is the concrete counterpart for ordered admission because request context, org identity, API version, limits, and object permissions must be resolved before a backend mutation is reachable.
- The usecase must preserve an explicit order: normalize request, identify route, verify tenant/cell, evaluate route authorization, apply rate-limit class, produce audit intent, then return an admit/deny decision.
- Tests should compare REST and gRPC fixture decisions for the same Salesforce-style org/API-version route to prevent protocol surfaces from drifting.
- Any optimization that returns an upstream before residency and authorization complete should be treated as a correctness regression.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Request normalization | `contracts/api-gateway.openapi.yaml` | Tenant, cell, route, trace, idempotency, and principal fields map into one envelope. |
| Route lookup order | `IP-003-routing-kernel-crate.md` | Lookup occurs before authorization but does not admit by itself. |
| Residency order | `policy/sov-cloud-overlay.cedar` | Cell and pack verdicts gate upstream selection. |
| Route authorization | `policy/route-authorization.cedar` | Route action deny maps to status and audit intent. |
| Rate-limit hook | `policy/rate-limit.cedar` | Usecase exposes route class for IP-009/IP-010 checks. |
| Audit output | `contracts/api-gateway.asyncapi.yaml` | Denied and admitted decisions produce event intent. |
| REST/gRPC parity | `contracts/api_gateway.proto` | Same fixture produces same decision on both protocols. |
| Counterpart readiness | Salesforce API ingress | Org/API-version context is resolved before object access. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-004-routing-usecase-crate.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `microservices/api-gateway/runbooks/edge-admission-regression.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-004-routing-usecase-crate.md`; matched trigger term(s): `plugin`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.
