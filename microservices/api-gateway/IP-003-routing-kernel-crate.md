# IP-003: `oya-api-gateway-routing-kernel` crate

**Status:** design-ready
**Owner:** axis-network
**Authority:** ADR-0056 + ADR-0105 + ADR-0157.

## A — Scope

The kernel-layer crate for **routing**. Implements ports (trait interfaces) for routes lookup, cell-residency check, and audit emission. Pure traits — no implementations live here.

## B — Ports

```rust
#[async_trait]
pub trait RouteLookupPort {
    async fn lookup(&self, host: &Hostname, path: &Path) -> Option<Route>;
}

#[async_trait]
pub trait CellResidencyPort {
    async fn residency_check(&self, tenant: &TenantId, cell: &CellId) -> ResidencyVerdict;
}

#[async_trait]
pub trait AuditEmitPort {
    async fn emit(&self, event: AuditEvent) -> Result<(), AuditEmitError>;
}
```

## C — Acceptance criteria

- All ports are `async_trait`s with `Send + Sync + 'static`.
- Mock implementations live in `mocks/` for downstream use.
- 100% trait coverage.

## D — Dependencies

- `oya-api-gateway-routing-domain` (sibling).
- `async-trait`, `thiserror`.

## E — References

- ADR-0056, ADR-0105, ADR-0157
- `microservices/api-gateway/IP-002-routing-domain-crate.md`

## Wave 15 A-G substance

### A - Problem
The domain validates routes, but the gateway also needs deterministic ports for route lookup, residency checks, and audit intent.

### B - Approach
Build `oya-api-gateway-routing-kernel` as the port layer named by `manifest.json` and `catalog/oya-api-gateway-routing-kernel.yaml`, depending only on routing-domain plus minimal async/error utilities.

### C - Deliverables
- `RouteLookupPort` for route-id and method/path lookup.
- `CellResidencyPort` consuming signed principal fields `tenant.cell`, `cell_epoch`, `pack`, and `region`.
- `AuditIntentPort` producing typed admission/denial intents.
- Error taxonomy for missing route, stale epoch, residency deny, and unavailable route table.
- Test doubles for routing-usecase and REST/gRPC tests.

### D - Ordered implementation steps
1. Define trait signatures around domain values.
2. Add `RouteDecision` and `ResidencyDecision` structs with deny reasons.
3. Model audit payloads with `tenant_id`, `cell_id`, `route_id`, and `request_id`.
4. Add mock implementations for usecase tests.
5. Test that kernel does not depend on adapter/rest/grpc crates.
6. Confirm names align with `catalog/oya-api-gateway-routing-kernel.yaml`.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-kernel` passes.
- Dependency check confirms only domain plus async/error utility dependencies.
- Tests cover missing route, stale `cell_epoch`, isolated cell, and residency mismatch.
- Audit fields align with `contracts/api-gateway.openapi.yaml` and `manifest.json`.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-routing-kernel.yaml`, `ARCHITECTURE.md` cell-aware-routing, `contracts/api-gateway.openapi.yaml`, `contracts/api_gateway.proto`, `policy/tenant-scope.cedar`, `policy/sov-cloud-overlay.cedar`, and `runbooks/cell-evac.md`.

### G - Counterpart comparison
Envoy Gateway and Kong separate route data-plane behavior from control-plane sources through typed APIs and plugins. Oyatie mirrors that separation and adds Cedar-ready deny reasons, per-cell residency decisions, and audit-chain payloads.

## Remediation notes

- ServiceNow API ingress is the concrete counterpart for kernel-level routing ports because it exposes stable tables/resources while enforcing tenant and integration identity before backend execution.
- The kernel must expose only ports: route lookup, residency verdict, audit intent, and test doubles. No adapter should leak ServiceNow-style integration details, Envoy xDS details, or audit-chain transport into trait definitions.
- Add negative tests for stale `cell_epoch`, unknown route, cross-tenant resource access, and audit-intent construction so downstream REST/gRPC adapters cannot skip kernel gates.
- Keep the kernel counterpart mapping documented because it proves this file is not a generic trait checklist; it is the contract boundary for an API ingress with tenant-scoped resources.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-003-routing-kernel-crate.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-003-routing-kernel-crate.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `microservices/api-gateway/runbooks/edge-admission-regression.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-003-routing-kernel-crate.md`; matched trigger term(s): `plugin`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.
