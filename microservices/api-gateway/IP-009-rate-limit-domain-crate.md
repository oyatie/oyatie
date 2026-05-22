# IP-009: `oya-api-gateway-rate-limit-domain` crate

**Status:** design-ready
**Owner:** axis-network

## A — Scope

Domain layer for **rate-limit** bounded context. Token-bucket + sliding-window primitives. Pure.

## B — API shape

```rust
pub struct TokenBucket {
    pub key: BucketKey,
    pub capacity: u32,
    pub refill_rate_per_min: u32,
}

pub enum BucketKey {
    PerIP(IpAddr),
    PerFingerprint(Fingerprint),
    PerTenant(TenantId),
    PerTenantPerRoute(TenantId, RouteId),
}

pub fn consume(b: &TokenBucket, tokens: u32, now: Timestamp) -> ConsumeOutcome { /* pure */ }
```

## C — Acceptance criteria

- Property tests on bucket invariants.
- 100% coverage.

## Wave 15 A-G substance

### A - Problem
Rate limits protect the shared edge, but core token-bucket rules must be deterministic before Valkey, Envoy, or per-cell aggregation is involved.

### B - Approach
Implement `oya-api-gateway-rate-limit-domain` from `catalog/oya-api-gateway-rate-limit-domain.yaml` as pure rate-limit math: bucket identity, route class, tenant class, refill policy, burst policy, and decision result.

### C - Deliverables
- Token-bucket and sliding-window value types for per-IP, fingerprint, tenant, and route-class limits.
- `RateLimitDecision` with remaining, reset, retry-after, and audit-deny reason.
- Pack/tenant override model compatible with `policy/rate-limit.cedar`.
- Property tests for refill, monotonic consumption, burst ceiling, and clock skew.
- Fixture defaults matching `ARCHITECTURE.md` B-6.

### D - Ordered implementation steps
1. Define bucket keys and route classes without Valkey or Envoy types.
2. Implement refill/consume against injected `Clock`.
3. Add default limit table for anonymous, authenticated, admin, partner, and machine classes.
4. Add policy override parsing as plain data.
5. Add property tests for refill and denial invariants.
6. Add serialization fixtures for adapter and REST/gRPC response headers.

### E - Acceptance gates
- `cargo test -p oya-api-gateway-rate-limit-domain` passes.
- Property tests cover burst exhaustion, refill, reset calculation, and time skew.
- No wall-clock dependency exists.
- Decisions produce `RateLimit-Limit`, `RateLimit-Remaining`, `RateLimit-Reset`, and `Retry-After`.

### F - Evidence
Grounding files: `catalog/oya-api-gateway-rate-limit-domain.yaml`, `ARCHITECTURE.md`, `PRD.md`, `policy/rate-limit.cedar`, `runbooks/rate-limit-saturation.md`, `slos/edge-availability.openslo.yaml`, and `performance-benchmark-numbers-2026-05-20.md`.

### G - Counterpart comparison
AWS usage plans, Kong rate-limiting plugins, and Apigee Quota/SpikeArrest provide quota and burst enforcement. Oyatie matches that behavior while preserving deterministic per-cell fallback and Cedar-driven overrides.

## Remediation notes

- Stripe API idempotency/rate-limit pressure is the concrete counterpart for this domain because payment-style APIs need stable bucket keys, retry hints, idempotency-aware admission, and deterministic burst math.
- The domain must distinguish duplicate idempotent retries from fresh write attempts before producing final retry-after guidance for adapter surfaces.
- Property tests should include Stripe-style tenant+route write buckets, anonymous read buckets, webhook retry buckets, refill drift, and clock-skew boundaries.
- This crate remains pure; Valkey scripts, Envoy filters, and network time sources belong to IP-010 or later runtime crates.

## File-specific validation matrix

| Check | Local artifact | Expected evidence |
| --- | --- | --- |
| Bucket keys | `policy/rate-limit.cedar` | Tenant, route, fingerprint, source IP, and route class are modelled. |
| Deterministic time | `capacity-model.md` | Clock is injected and never read from wall time. |
| Retry hints | `contracts/api-gateway.openapi.yaml` | `RateLimit-*` and `Retry-After` values are representable. |
| Stripe-style writes | Stripe API idempotency/rate-limit pressure | Idempotent retry and fresh write consume different paths. |
| Burst ceiling | `performance-benchmark-numbers-2026-05-20.md` | Burst math can support gateway throughput targets. |
| Anonymous class | `PRD.md` | Public read and auth-attempt buckets are separate. |
| Audit deny | `contracts/api-gateway.asyncapi.yaml` | Exceeded events carry bucket class and route identifiers. |
| Valkey handoff | `IP-010-rate-limit-adapter-valkey.md` | Domain output is enough for atomic persistence. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `microservices/api-gateway/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `microservices/api-gateway/contracts/api-gateway.openapi.yaml`, `microservices/api-gateway/contracts/api-gateway.asyncapi.yaml`, `microservices/api-gateway/contracts/api_gateway.proto`, `microservices/api-gateway/IP-009-rate-limit-domain-crate.md`.

## DR posture (per ADR-0343)

- Target source: `microservices/api-gateway/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `86400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `audit_chain_merkle_seal`, `valkey`].
- Surface evidence: `microservices/api-gateway/runbooks/cell-evac.md`, `microservices/api-gateway/runbooks/rate-limit-saturation.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-009-rate-limit-domain-crate.md`.

## Pod runtime tier (per ADR-0338)

- `pod_runtime_tier: 0`.
- Justification: tenant-customer code is present in this IP's execution path; Tier 0 requires Kata plus Cloud Hypervisor isolation.
- Surface evidence: `microservices/api-gateway/runbooks/edge-admission-regression.md`, `microservices/api-gateway/manifest.json`, `microservices/api-gateway/IP-009-rate-limit-domain-crate.md`; matched trigger term(s): `plugin`.
- Admission expectation: spawned workloads for this path use `kata-cloud-hypervisor`; first-party helpers may only run outside Tier 0 when split into a separate non-tenant-customer IP.
