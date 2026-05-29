---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: observability
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-observability + gtm-customer-success
deciders: axis-observability, council-architecture
related_adrs: [ADR-0139, ADR-0131]
related_artifacts:
  - microservices/observability/contracts/openapi/slo-engine.yaml
  - microservices/observability/contracts/proto/slo-engine.proto
  - microservices/observability/PRD.md (FR-01 OpenSLO authoring; SDK identified as gap #19)
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (observability µservice)

## Purpose

Tenants authoring OpenSLO manifests + querying eligibility verdicts programmatically need first-party SDKs in the languages their workloads use. This document specifies the SDK strategy: which languages, how generated, what guarantees, sunset policy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie's own language) | First-party authored (`oya-observability-slo-engine-sdk` crate; per PRD §"BC layer mapping") | axis-observability |
| **TypeScript** | M01+1 (first external-tenant SDK) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-observability + gtm |
| **Python** | M02 | OpenAPI-generated; published to PyPI | axis-observability + gtm |
| **Go** | M02 | gRPC-generated baseline + ergonomic wrappers; published as go-module | axis-observability + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated baseline + ergonomic wrappers; published to Maven Central | axis-observability + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; published to NuGet | axis-observability + gtm |
| **Ruby** | M04-onward (only if tenant demand surfaces) | OpenAPI-generated | axis-observability |
| **PHP** | (none — no current tenant demand) | n/a | n/a |

Prioritisation drivers: oyatie's own µservice languages first; then largest tenant developer-population languages (TypeScript + Python lead).

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/observability/src/crates/oya-observability-slo-engine-sdk/`.

- Public surface: `Client::new(opts) -> Client; client.get_eligibility_verdict(...) -> Result<EligibilityVerdict, ...>`.
- Authentication: `Client` accepts an OIDC token provider (closure / trait impl).
- Tenancy: `Client` is bound to a tenant at construction; `X-Scope-OrgID` header automatically populated.
- Retry policy: built-in exponential backoff for transient 5xx and 429.
- Streaming: `client.stream_eligibility_verdicts(...) -> impl Stream<Item=EligibilityVerdict>` via gRPC streaming.
- Re-exports types from `oya-observability-slo-engine-kernel` so consumers see consistent shapes.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Published to oyatie's internal crate registry; future open-source decision when SDK API stabilises.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Generation pipeline (lives in `microservices/observability/sdk-generation/`, Slice D):

1. Source of truth: `contracts/openapi/slo-engine.yaml` (REST) + `contracts/proto/slo-engine.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific generator profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer on top of generated code; provides:
   - First-class auth helpers (OIDC token provider abstraction).
   - Tenant-context binding at client construction.
   - Retry policy + circuit-breaker matching Rust SDK behavior.
   - Idiomatic naming + error handling per language convention.
5. Per-language SDK ships with:
   - README + quick-start.
   - Versioning matching observability µservice major.minor.
   - Compatibility matrix: which SDK version works with which observability µservice version.
   - License header + open-source decision recorded.
6. Per-language CI lane: build + lint + integration-test against staging observability cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Read latest verdict | `getEligibilityVerdict(ms, sha, env)` | `EligibilityVerdict` |
| Stream verdicts | `streamEligibilityVerdicts(ms, env, since)` | streaming verdict (push-based) |
| Read release pointer | `getReleasePointer(ms, env)` | `ReleasePointer` |
| List OpenSLO manifests | `listOpenSloManifests(ms)` | `OpenSloManifest[]` |
| Read OpenSLO manifest | `getOpenSloManifest(ms, sli)` | `OpenSloManifest` |
| Validate OpenSLO YAML | `validateOpenSloSchema(yaml_bytes)` | `{valid: bool, errors: SchemaError[]}` |
| Read burn-rate | `getBurnRate(ms, env, sli)` | `BurnRateSnapshot` |

OpenSLO manifest authoring is via git PR (CODEOWNERS + Jenkins/Forgejo required checks enforced). The SDKs do NOT expose a "create/update manifest" method directly — that path is via the tenant operator's git workflow, validated at PR time. If future tenant feedback surfaces a need for programmatic manifest authoring, that's a per-tenant DPA-recorded entitlement granted via Cedar.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OTel API key + Mimir read key for the tenant via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start (per language) | gtm-customer-success |
| Sample workflow: how to subscribe to EligibilityChanged events in tenant deploy pipeline | axis-observability |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-observability |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration to replacement generator if possible |
| Breaking API change in observability µservice | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major version |

Per ADR-#### deprecation-and-migration discipline: every SDK sunset emits an ADR-shaped notice + deprecation-warning in the SDK + tenant comms via gtm-customer-success.

## Versioning

Observability µservice version: semver.
SDK version per language: matches observability major.minor; SDK patch independent.
Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

Example:
- observability v1.2.0 + Rust SDK v1.2.0 + TypeScript SDK v1.2.0 → all compatible.
- observability v2.0.0 + Rust SDK v1.2.0 → compatible via backward-adapter for 12mo after v2 launch.
- observability v2.0.0 + Rust SDK v0.9.0 → INCOMPATIBLE; SDK upgrade required.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API has been stable in production for ≥ 6mo. Default: keep SDKs closed-source until a tenant-driven request (or a competitive consideration) makes open-source the right move. Open-source-when-stable matches Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against observability versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused SDKs flagged for sunset review.

## References

- `microservices/observability/contracts/openapi/slo-engine.yaml`.
- `microservices/observability/contracts/proto/slo-engine.proto`.
- `microservices/observability/PRD.md` BC layer mapping (`-sdk` crate per ADR-0105).
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- Twilio SDK precedent — `twilio.com/docs/libraries`.
