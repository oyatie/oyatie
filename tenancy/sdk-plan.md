---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: tenancy
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tenancy + gtm-customer-success
deciders: axis-tenancy, council-architecture
related_adrs: [ADR-0018, ADR-0139, ADR-0131]
related_artifacts:
  - tenancy/contracts/openapi/tenancy.yaml
  - tenancy/contracts/proto/tenancy.proto
  - tenancy/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (tenancy µservice)

## Purpose

Tenants performing programmatic tenancy administration (CI-driven onboarding, automated DSR submission, custom dashboards) need first-party SDKs. This document specifies which languages, how generated, what guarantees, sunset policy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; every other µservice consumes tenancy via Rust kernel) | First-party authored (`oya-tenancy-tenant-lifecycle-sdk` crate) | axis-tenancy |
| **TypeScript** | M01+1 (first external-tenant SDK) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-tenancy + gtm |
| **Python** | M02 | OpenAPI-generated; published to PyPI | axis-tenancy + gtm |
| **Go** | M02 | gRPC-generated baseline + ergonomic wrappers; published as go-module | axis-tenancy + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated + ergonomic wrappers; published to Maven Central | axis-tenancy + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; published to NuGet | axis-tenancy + gtm |
| **Ruby** | M04-onward (only if tenant demand) | OpenAPI-generated | axis-tenancy |
| **PHP** | (none) | n/a | n/a |

Prioritisation: Rust first (every other µservice's kernel consumes `TenantContext` and `TenantId` value-types); TypeScript next for tenant admin dashboards.

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/tenancy/src/crates/oya-tenancy-tenant-lifecycle-sdk/`.

- Public surface: `TenancyClient::new(opts) -> TenancyClient; client.create_tenant(req).await -> Result<Tenant, ...>`.
- Authentication: `TenancyClient` accepts an OIDC token provider (closure / trait impl).
- Tenancy: `TenancyClient` is bound to a tenant at construction; JWT carries claim.
- Retry policy: built-in exponential backoff for transient 5xx and 429.
- Streaming: `client.stream_tenant_lifecycle_events(...)` via gRPC streaming.
- Re-exports types from `oya-tenancy-tenant-lifecycle-kernel` so consumers see consistent shapes.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Published to oyatie's internal crate registry; open-source decision scheduled-for-distinct-tracked-work until API stabilises (M02-onward).

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Pipeline (lives in `microservices/tenancy/sdk-generation/`):

1. Source of truth: `contracts/openapi/tenancy.yaml` (REST) + `contracts/proto/tenancy.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific generator profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer providing:
   - First-class OIDC auth helpers.
   - Tenant-context binding at client construction.
   - Retry policy + circuit-breaker matching Rust SDK behavior.
   - Idiomatic naming + error handling per language convention.
5. Per-language SDK ships with:
   - README + quick-start.
   - Versioning matching tenancy µservice major.minor.
   - Compatibility matrix.
   - License header + open-source decision recorded.
6. Per-language CI lane: build + lint + integration-test against staging tenancy cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Create tenant (platform-operator only) | `createTenant(req)` | `Tenant` |
| Read tenant | `getTenant(tenantId)` | `Tenant` |
| List tenants | `listTenants(filters)` | `Tenant[]` |
| Suspend tenant | `suspendTenant(tenantId, reason)` | `Tenant` |
| Resume tenant | `resumeTenant(tenantId)` | `Tenant` |
| Delete tenant (initiates DSR cascade) | `deleteTenant(tenantId, attestation)` | `DsrRequest` |
| Submit DSR | `submitDsrRequest(req)` | `DsrRequest` |
| Read DSR status | `getDsrRequest(dsrId)` | `DsrRequest` |
| Read proof-of-erasure | `getProofOfErasure(dsrId)` | `ProofOfErasure` |
| Stream lifecycle events | `streamTenantLifecycleEvents(tenantId, since)` | streaming `TenantLifecycleEvent` |
| Issue JWT (internal use) | `issueJwt(tenantId, principalType)` | `IssuedJwt` |
| Verify JWT (every µservice's kernel uses) | `verifyJwt(jwt)` | `JwtVerification` |
| Read cell-assignment | `getCellAssignment(tenantId)` | `CellAssignment` |

The JwtVerifier surface is the load-bearing primitive consumed by every workload µservice's kernel; performance + correctness contract is in PRD §"Performance Targets" + AC-03.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OIDC client credentials + per-pack tenancy API key via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start (per language) | gtm-customer-success |
| Sample workflow: how to programmatically activate / suspend / DSR a tenant | axis-tenancy |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-tenancy |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration to replacement generator if possible |
| Breaking API change in tenancy µservice | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major version |

## Versioning

Tenancy µservice version: semver.
SDK version per language: matches tenancy major.minor; SDK patch independent.
Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

Example:
- tenancy v1.2.0 + Rust SDK v1.2.0 + TypeScript SDK v1.2.0 → all compatible.
- tenancy v2.0.0 + Rust SDK v1.2.0 → compatible via backward-adapter for 12mo after v2 launch.
- tenancy v2.0.0 + Rust SDK v0.9.0 → INCOMPATIBLE; SDK upgrade required.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API has been stable for ≥ 6mo. Default: closed-source until a tenant-driven request makes open-source the right move (Stripe + Twilio precedent).

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against tenancy versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused SDKs flagged for sunset.

## References

- `tenancy/contracts/openapi/tenancy.yaml`.
- `tenancy/contracts/proto/tenancy.proto`.
- `tenancy/PRD.md` BC layer mapping (`-sdk` crate per ADR-0105).
- ADR-0105 (13-layer enum).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- Auth0 Management SDK precedent — `auth0.com/docs/libraries`.
