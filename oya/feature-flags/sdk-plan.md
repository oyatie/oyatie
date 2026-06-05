---
doc_class: SDKPlan
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0159
  - ADR-0253
  - ADR-0258
companion_docs:
  - microservices/feature-flags/contracts/openfeature-sdk-contract.md
  - microservices/feature-flags/competitor-parity-matrix.md
  - microservices/feature-flags/ARCHITECTURE.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# SDK Plan — Feature Flags

## OpenFeature compliance

All oyatie feature-flags SDKs implement the [OpenFeature specification](https://openfeature.dev/) as server-side providers. OpenFeature is the CNCF standard for feature-flag SDK interoperability (hyperscaler precedent: LaunchDarkly, Split.io, Statsig all publish OpenFeature providers).

OpenFeature provider contract: `contracts/openfeature-sdk-contract.md`.

## Phase 1 SDKs (this phase)

Rust is the canonical checked-in SDK implementation. Non-Rust OpenFeature providers are compatibility products generated from contracts under pinned, registry-owned lanes; they do not create repo-root pnpm authority, product TypeScript application logic, or ad-hoc Python surfaces.

### Rust SDK (`oya-feature-flags-sdk`)

- **Crate**: `oya-feature-flags-sdk` (workspace member).
- **Interface**: `FlagClient` implementing `OpenFeatureProvider` trait.
- **Local cache**: `DashMap<(TenantId, FlagKey), CachedVariant>` with 30s TTL; refresh via background tokio task.
- **Transport**: gRPC over HTTP/3 (tonic + quic); fallback to HTTP/2 gRPC; fallback to HTTP/1.1 REST.
- **Streaming updates**: SSE subscription to `/api/v1/flags/stream` for push-based cache invalidation (sub-1s update on kill-switch or targeted flag change).
- **Evaluation context**: `EvaluationContext { tenant_id, principal_id, persona_tier, cohort_ids, consent_purposes, audience_type }`.
- **Performance**: local cache hit ≤0.001ms; cache miss (gRPC) ≤1ms p99.
- **Initialization**: `FlagClient::new(config).await?` — connects, loads warm cache, subscribes to SSE stream.
- **Shutdown**: `client.shutdown().await` — flushes pending audit events, closes connections.

```rust
// Usage example
let enabled = client.bool_value("new-checkout", false, &ctx).await?;
let variant = client.string_value("button-color", "blue", &ctx).await?;
let config = client.object_value::<CheckoutConfig>("checkout-config", default, &ctx).await?;
```

### Generated compatibility SDKs

- **TypeScript**: allowed only as a strict, generated OpenFeature provider with pinned dependencies, Buck2-owned generation/check targets, and a registry row documenting why Rust/WASM is not sufficient for the consumer boundary.
- **Python**: allowed only as a generated compatibility provider when a customer integration requires it; no first-party service, CI, or monorepo automation authority.

## Phase 2 SDKs (Q4 2026)

| SDK | OpenFeature provider | Framework integrations |
|---|---|---|
| Go | `go.openfeature.dev` provider | `net/http`, `gin`, `echo` middleware |
| Java | `dev.openfeature:sdk` provider | Spring Boot starter, Jakarta EE |
| .NET | `OpenFeature.Contrib.Providers.Oyatie` | ASP.NET Core middleware |
| Swift / iOS | Custom (OpenFeature iOS SDK is pre-1.0) | SwiftUI environment |

## SDK versioning policy (ADR-0258)

- SDKs follow SemVer independently of the server contract.
- MAJOR bump: breaking change to `EvaluationContext` shape or `FlagClient` interface.
- MINOR bump: new flag types, new context fields (backwards-compatible).
- PATCH bump: bug fixes, performance.
- Deprecation: MINOR-deprecated fields sunset after 2 minor versions.
- SDK ↔ server compatibility: server maintains backwards compatibility for 2 major SDK versions.

## Streaming architecture

SSE stream endpoint: `GET /api/v1/flags/stream?tenant_id={tenant_id}` (HTTP/3; falls back to HTTP/2).

Events emitted on stream:
- `flag-state-changed`: flag value changed; includes `flag_key`, `new_default_variant`.
- `kill-switch-activated`: kill-switch engaged; includes `flag_key`.
- `kill-switch-deactivated`: kill-switch disengaged.
- `pack-override-applied`: pack-mandated override; SDK should invalidate affected flags.

SDK handling: on event receipt, invalidate affected keys in local cache; next evaluation triggers fresh fetch.

## SDK feature matrix

| Feature | Rust | Generated TypeScript compatibility | Generated Python compatibility |
|---|---|---|---|
| Boolean evaluation | ✓ | ✓ | ✓ |
| String evaluation | ✓ | ✓ | ✓ |
| Number evaluation | ✓ | ✓ | ✓ |
| JSON object evaluation | ✓ | ✓ | ✓ |
| Local in-process cache | ✓ | ✓ | ✓ |
| SSE streaming updates | ✓ | ✓ | ✓ |
| OpenFeature compliance | ✓ | ✓ | ✓ |
| gRPC transport | ✓ | roadmap | roadmap |
| HTTP/3 transport | ✓ | ✓ | ✓ |
| Evaluation context typing | ✓ (Rust types) | ✓ (TS strict) | ✓ (dataclass) |
| Emergency-services audience type | ✓ | ✓ | ✓ |
| Pack-override awareness | ✓ | ✓ | ✓ |
