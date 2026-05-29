# IP-002: `oya-api-gateway-routing-domain` crate

**Status:** design-ready
**Owner:** axis-network
**Authority:** ADR-0056 (Rust BNF) + ADR-0105 (13-layer enum) + ADR-0157.

## A — Scope

The Rust domain-layer crate for the **routing** bounded context. Pure functions; no I/O; no Cedar eval; no Envoy dependency.

## B — API shape

```rust
pub struct Route {
    pub id: RouteId,
    pub upstream: UpstreamId,
    pub path_template: PathTemplate,
    pub auth_class: AuthClass,
    pub rate_limit_class: RateLimitClass,
    pub cache_policy: CachePolicy,
    pub tenant_scope: TenantScope,
    pub cell_jurisdiction: Option<CellJurisdiction>,
}

pub enum AuthClass {
    Anonymous,
    Tenant,
    Admin,
    Partner,
    Machine,
}

pub enum RateLimitClass {
    AnonRead,
    AnonAuthAttempt,
    AuthRead,
    AuthWrite,
    Admin,
    Partner,
    Machine,
}

pub fn validate_route(r: &Route) -> Result<(), RouteValidationError> { /* pure */ }
```

## C — Acceptance criteria

- 100% unit test coverage on `validate_route`.
- Property tests for `PathTemplate` parsing (RFC 6570 URI templates).
- `deny(warnings)`.
- `Cargo.toml` declares ZERO dependencies outside std + serde + thiserror.

## D — Dependencies

- None. This crate sits at the bottom of the dependency DAG.

## E — Tests

- `tests/route_validation.rs` — canonical route fixtures.
- `tests/path_template_property.rs` — proptest property tests.
- `tests/serde.rs` — serialize/deserialize round-trip.

## F — References

- `microservices/api-gateway/catalog/oya-api-gateway-routing-domain.yaml`
- ADR-0056, ADR-0105, ADR-0157

## Wave 15 A-G substance

### A - Problem
Route selection needs a pure model before Envoy, REST, gRPC, or Valkey adapters can be trusted.

### B - Approach
Implement `oya-api-gateway-routing-domain` as the dependency-free routing core named in `manifest.json` and `catalog/oya-api-gateway-routing-domain.yaml`; keep I/O, Cedar evaluation, Envoy xDS, and audit emission outside it.

### C - Deliverables
- Route, upstream, path-template, auth-class, rate-limit-class, cache-policy, tenant-scope, and cell-jurisdiction value types.
- `validate_route` for upstream, template, tenant-scope, and cache/auth compatibility.
- Serde round-trip support within the existing dependency boundary.
- Property corpus for RFC 6570-style path templates.
- Canonical route fixtures tied to `contracts/api-gateway.openapi.yaml`.

### D - Ordered implementation steps
1. Create the crate using ADR-0056/ADR-0105 naming.
2. Add route and path-template value types with no I/O dependencies.
3. Encode validation errors as stable enum variants.
4. Add serde round-trip tests for contract-backed route definitions.
5. Add path-template property tests and invalid-template fixtures.
6. Confirm kernel/usecase crates depend inward on this crate only.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-routing-domain` passes with `deny(warnings)`.
- Property tests cover variables, duplicate variables, empty segments, traversal, and percent-encoding normalization.
- `catalog/oya-api-gateway-routing-domain.yaml` remains the crate record.
- `manifest.json` continues to list IP-002 as design-ready.

### F - Evidence
Grounding files: `manifest.json`, `catalog/oya-api-gateway-routing-domain.yaml`, `PRD.md`, `ARCHITECTURE.md`, `contracts/api-gateway.openapi.yaml`, `policy/route-authorization.cedar`, and `policy/sov-cloud-overlay.cedar`.

### G - Counterpart comparison
Kong models services/routes/upstreams, AWS models resources/methods/stages/integrations, and Apigee models proxy/target endpoints. Oyatie must cover that route identity and upstream intent while adding tenant scope, cell jurisdiction, Cedar action identity, and cache/rate-limit classes.

## Remediation notes

- GitLab API ingress provides the concrete counterpart for path-heavy route matching, project/group scoping, token class separation, and predictable 404/403 boundaries.
- The domain crate should model those concerns as plain route facts: host, method, path template, route class, tenant scope, cell jurisdiction, cache policy, and rate-limit class.
- Avoid adding Envoy, Valkey, Cedar runtime, or HTTP parser types here; counterpart parity is route identity and validation, not gateway execution.

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-002-routing-domain-crate.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `microservices/api-gateway/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-002-routing-domain-crate.md`.
