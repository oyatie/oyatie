---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-runtime + gtm-customer-success
deciders: axis-foundry-runtime, council-architecture
related_adrs: [ADR-0025, ADR-0131]
related_artifacts:
  - microservices/foundry-runtime/contracts/openapi/foundry-runtime.yaml
  - microservices/foundry-runtime/contracts/proto/foundry-runtime.proto
  - microservices/foundry-runtime/PRD.md (BC layer mapping with -sdk crate)
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (foundry-runtime µservice)

## Purpose

Tenants invoking capabilities programmatically need first-party SDKs in the languages their workloads use. This document specifies the SDK strategy: which languages, generation strategy, guarantees, sunset policy. Mirrors the observability µservice's sdk-plan.md shape.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie's own language) | First-party authored (`oya-foundry-runtime-capability-executor-sdk` crate; per PRD §"BC layer mapping") | axis-foundry-runtime |
| **TypeScript** | M01+1 (first external-tenant SDK) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-foundry-runtime + gtm |
| **Python** | M02 | OpenAPI-generated; published to PyPI | axis-foundry-runtime + gtm |
| **Go** | M02 | gRPC-generated baseline + ergonomic wrappers; published as go-module | axis-foundry-runtime + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated baseline + ergonomic wrappers; published to Maven Central | axis-foundry-runtime + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; published to NuGet | axis-foundry-runtime + gtm |
| **Ruby** | M04-onward (tenant-demand-gated) | OpenAPI-generated | axis-foundry-runtime |

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/foundry-runtime/src/crates/oya-foundry-runtime-capability-executor-sdk/`.

- Public surface: `Client::new(opts) -> Client; client.dispatch(capability_id, input, opts) -> Result<Invocation, Error>; client.stream_invocation(invocation_id) -> impl Stream<Item=Invocation>; client.get_session(id) -> Result<Session, Error>; client.get_autonomy_ceiling() -> Result<AutonomyCeiling, Error>`.
- Authentication: `Client` accepts an OIDC token provider (closure / trait impl).
- Tenancy: `Client` bound to tenant at construction; `X-Scope-OrgID` automatically populated.
- Retry: built-in exponential backoff for transient 5xx + 429 + 503; respects `Retry-After`.
- Streaming: gRPC streaming for `StreamInvocation`.
- Idempotency: `dispatch` accepts `idempotency_key` per OpenAPI contract.
- Re-exports types from `oya-foundry-runtime-capability-executor-kernel` so consumers see consistent shapes.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Published to oyatie's internal crate registry; future open-source decision when stable.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Generation pipeline at `microservices/foundry-runtime/sdk-generation/` (Slice D extension):

1. Source of truth: `contracts/openapi/foundry-runtime.yaml` (REST) + `contracts/proto/foundry-runtime.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific generator profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer providing:
   - First-class OIDC token provider abstraction.
   - Tenant-context binding at client construction.
   - Retry policy matching Rust SDK.
   - Idiomatic naming + error handling.
5. Per-language SDK ships with:
   - README + quick-start.
   - Version matching foundry-runtime major.minor.
   - Compatibility matrix.
   - License header + open-source decision recorded.
6. Per-language CI lane: build + lint + integration-test against staging foundry-runtime cluster.

## Public Surface (across languages)

| Capability | Method | Returns |
|---|---|---|
| Dispatch capability | `dispatchCapability(capability_id, input, opts)` | `Invocation` |
| Stream invocation lifecycle | `streamInvocation(invocation_id)` | streaming Invocation (push-based) |
| Get invocation | `getInvocation(invocation_id)` | `Invocation` |
| Cancel invocation | `cancelInvocation(invocation_id)` | `CancelInvocationResponse` |
| Get session | `getSession(session_id)` | `Session` |
| Get autonomy ceiling | `getAutonomyCeiling()` | `AutonomyCeiling` |
| Get capability descriptor | `getCapabilityDescriptor(capability_id)` | `CapabilityDescriptor` |
| Validate capability descriptor schema | `validateCapabilityDescriptorSchema(yaml_bytes)` | `{valid, errors[]}` |

Capability descriptor authoring is via git PR through `foundry-supervisor`; SDKs do NOT expose a "create/update descriptor" method. If future tenant feedback surfaces a need for programmatic authoring, that's a per-tenant DPA-recorded entitlement granted via Cedar.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue per-tenant OIDC client + API key via OpenBao | ops-security |
| Provide quick-start doc per language | gtm-customer-success |
| Sample workflow: dispatch in tenant CI; stream until completion; read session | axis-foundry-runtime |
| Quarterly update notification (breaking changes 6mo advance notice) | axis-foundry-runtime |

## Sunset Policy

| SDK | Sunset trigger | Window |
|---|---|---|
| < 1% tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Generator lib deprecated upstream | dep-deprecated | 12mo advance + auto-migration to replacement |
| Breaking API change | per-release | major version bump; backward-compat adapter for 1 prior major |

## Versioning

- foundry-runtime µservice: semver.
- SDK per language: matches foundry-runtime major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI verifies SDK against current + 1 prior major.

Example:
- foundry-runtime v1.2.0 + Rust SDK v1.2.0 + TypeScript SDK v1.2.0 → compatible.
- foundry-runtime v2.0.0 + Rust SDK v1.2.0 → compatible via backward-adapter for 12mo.
- foundry-runtime v2.0.0 + Rust SDK v0.9.0 → INCOMPATIBLE; upgrade required.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API stable in production for ≥ 6mo. Default closed-source until tenant-driven request or competitive consideration makes open-source the right move (matches Stripe + Twilio precedent).

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: version N+1 works against foundry-runtime N-1, N, N+1.
- Annual SDK telemetry review.

## References

- `microservices/foundry-runtime/contracts/openapi/foundry-runtime.yaml`.
- `microservices/foundry-runtime/contracts/proto/foundry-runtime.proto`.
- `microservices/foundry-runtime/PRD.md` BC layer mapping.
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC — `grpc.io`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- Twilio SDK precedent — `twilio.com/docs/libraries`.
