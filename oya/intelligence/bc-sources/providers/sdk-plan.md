---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: foundry-providers
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + gtm-customer-success
deciders: axis-foundry, council-architecture
related_adrs: [ADR-0025, ADR-0131]
related_artifacts:
  - microservices/intelligence-providers/contracts/openapi/provider-router.yaml
  - microservices/intelligence-providers/contracts/proto/provider-invoke.proto
  - microservices/intelligence-providers/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (foundry-providers µservice)

## Purpose

Workload µservices and external tenants invoking provider-router programmatically need first-party SDKs. This document specifies the SDK strategy: languages, generation, guarantees, sunset policy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie own) | First-party authored (`oya-foundry-providers-router-sdk` crate) | axis-foundry |
| **TypeScript** | M01+1 | OpenAPI-generated + ergonomic wrapper; published to oyatie internal npm | axis-foundry + gtm |
| **Python** | M02 | OpenAPI-generated + wrapper; published to PyPI | axis-foundry + gtm |
| **Go** | M02 | gRPC-generated + wrapper; published as go-module | axis-foundry + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated + wrapper; Maven Central | axis-foundry + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; NuGet | axis-foundry + gtm |

Workload µservices written in Rust use the first-party SDK; external tenants pick the language matching their stack.

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/intelligence-providers/src/crates/oya-foundry-providers-router-sdk/`.

- Public surface: `Client::new(opts) -> Client; client.invoke(req) -> Result<InvokeResponse, Error>; client.decide(req) -> Result<RouterDecision, Error>`.
- Authentication: OIDC token provider trait; SPIFFE workload-identity helper.
- Tenancy: `Client` bound to a tenant at construction; `tenant_id` automatically populated.
- Retry: exponential backoff for transient 5xx and 429; honours `Retry-After`.
- Streaming: `client.invoke_stream(req) -> impl Stream<Item=InvokeStreamChunk>` via gRPC streaming.
- Re-exports types from `oya-foundry-providers-router-api` so consumers see consistent shapes.
- `#![deny(unsafe_code)]`.
- **Never** logs or surfaces credential bytes (consistent with `policy/credential-isolation.md`).

### Generated SDKs

Generation pipeline at `microservices/intelligence-providers/sdk-generation/`:

1. Source of truth: `contracts/openapi/provider-router.yaml` (REST) + `contracts/proto/provider-invoke.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper (hand-authored): auth helpers, tenant-binding, retry policy, idiomatic error handling.
5. Per-language CI lane: build + lint + integration-test against staging foundry-providers cluster.

## Public Surface

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Invoke provider (one-shot) | `invoke(InvokeRequest)` | `InvokeResponse` |
| Invoke streaming | `invoke_stream(InvokeRequest)` | stream of `InvokeStreamChunk` |
| Decide (dry-run) | `decide(DecideRequest)` | `RouterDecision` |
| Health snapshot | `providers_health(filter)` | list of `ProviderHealthSnapshot` |
| List capabilities | `list_capabilities()` | list of `CapabilityProfile` |
| Read tenant config | `get_tenant_provider_config(tenant)` | `TenantProviderConfig` |
| Update tenant config | `update_tenant_provider_config(tenant, cfg)` | `TenantProviderConfig` |

Credentials NEVER appear in any SDK surface — tenants reference `SecretReference` URIs as opaque strings; the SDK passes the URI through to the µservice which resolves it via OpenBao.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Tenant operator obtains OIDC client + (per-vendor) SecretReference paths via OpenBao | ops-security |
| Tenant operator integrates SDK in workload code | tenant team |
| Provide tenant onboarding doc with per-language quick-start | gtm-customer-success |
| Quarterly SDK update notification | axis-foundry |

## Sunset Policy

| SDK | Sunset trigger | Window |
|---|---|---|
| Any SDK with < 1 % tenant usage for ≥ 12 mo | underused | 6 mo notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12 mo notice + auto-migrate to replacement generator |
| Breaking API change | per-release | major version bump; backwards-compat adapter for 1 prior major version |

## Versioning

- foundry-providers µservice version: semver.
- SDK per-language version: matches major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API has been stable in production ≥ 6 mo. Default: internal-only until tenant-driven or competitive consideration; matches Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against foundry-providers versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused SDKs flagged.

## References

- `microservices/intelligence-providers/contracts/openapi/provider-router.yaml`.
- `microservices/intelligence-providers/contracts/proto/provider-invoke.proto`.
- `microservices/intelligence-providers/PRD.md`.
- ADR-0105 (sdk is a canonical layer in the 13-layer enum).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- Twilio SDK precedent — `twilio.com/docs/libraries`.
