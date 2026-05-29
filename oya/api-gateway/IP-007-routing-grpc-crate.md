# IP-007: `oya-api-gateway-routing-grpc` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

gRPC management plane (proto3) for high-throughput route management. Used by policy-engine push channel.

## B — Surface

Defined in `contracts/api_gateway.proto`.

## C — Acceptance criteria

- proto3 only.
- mTLS-required for all RPCs.
- SPIFFE SVID verify on every request.

## Wave 15 A-G substance

### A - Problem
Internal callers and control-plane workers need typed route decisions that avoid REST drift while preserving the same policy and audit behavior.

### B - Approach
Implement `oya-api-gateway-routing-grpc` from `catalog/oya-api-gateway-routing-grpc.yaml` using `contracts/api_gateway.proto` as a control-plane/API layer over routing-usecase, not as a data-plane shortcut.

### C - Deliverables
- Tonic service generated from `contracts/api_gateway.proto`.
- gRPC methods for admission/describe flows represented in proto.
- Status mapping for deny, stale cell epoch, unavailable route bundle, and invalid request envelope.
- Interceptors for SPIFFE identity, trace metadata, and audit correlation.
- Proto compatibility tests with generated clients.

### D - Ordered implementation steps
1. Generate tonic bindings from checked-in proto.
2. Map proto fields to routing-usecase DTOs.
3. Add SPIFFE and trace metadata interceptors.
4. Add gRPC status mapping and leak tests.
5. Add compatibility tests for old/new fields.
6. Confirm REST and gRPC surfaces return equivalent decisions for the same fixtures.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-grpc` passes.
- Proto generation succeeds from `contracts/api_gateway.proto`.
- REST/gRPC parity fixture proves identical admit/deny and audit intent.
- SPIFFE metadata is required for internal control-plane calls.

### F - Evidence
Grounding files: `contracts/api_gateway.proto`, `catalog/oya-api-gateway-routing-grpc.yaml`, `manifest.json`, `ARCHITECTURE.md`, `policy/route-authorization.cedar`, and `policy/ci-scope.cedar`.

### G - Counterpart comparison
Apigee and AWS mainly expose REST management APIs, while Envoy/xDS and some Kong control paths rely on typed internal APIs. Oyatie follows the typed-control-plane model and adds SPIFFE metadata plus audit-chain correlation.

## Remediation notes

- ServiceNow API ingress is the concrete counterpart for typed internal API calls because bulk integration traffic needs stable request envelopes, tenant identity, and predictable deny reasons across generated clients.
- gRPC must prove parity with REST for admit/deny outcomes; the proto surface cannot become a privileged bypass around Cedar, residency, rate-limit, or audit gates.
- Compatibility tests should include unknown future fields, missing SPIFFE metadata, stale cell epoch, and retryable route-table unavailable status.
- Keep generated code out of this IP; this document defines the crate behavior and verification gates, not the generated artifact content.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Proto source | `contracts/api_gateway.proto` | Tonic bindings derive from checked-in proto only. |
| SPIFFE gate | `iac/spire-trust-bundle.yaml` | Missing or invalid SPIFFE metadata denies internal calls. |
| REST parity | `contracts/api-gateway.openapi.yaml` | Same fixture yields same admit/deny as REST. |
| Error mapping | `failure-modes.md` | Route missing, stale cell, policy deny, and unavailable states are distinct. |
| Audit correlation | `contracts/api-gateway.asyncapi.yaml` | gRPC metadata carries audit request correlation. |
| ServiceNow-style integration | ServiceNow API ingress | Generated clients receive stable status details for tenant-scoped resources. |
| Backward compatibility | `contracts/api_gateway.proto` | Unknown future fields do not change current decisions. |
| Route policy | `policy/route-authorization.cedar` | gRPC cannot bypass route policy. |
| CI policy | `policy/ci-scope.cedar` | Route-management RPCs require CI/operator scope. |
| Trace propagation | `contracts/metric-naming-convention.md` | gRPC metrics preserve route and decision labels. |

## Remediation follow-up checklist

- Add one ServiceNow-style tenant resource fixture for generated-client compatibility.
- Add one missing SPIFFE metadata fixture that fails before route lookup.
- Add one unknown-field proto fixture that preserves the current admit/deny decision.
- Add one stale route-table fixture that maps to retryable status without leaking internals.
- Keep REST/gRPC fixture names paired so protocol parity remains mechanically checkable.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-007-routing-grpc-crate.md`.
