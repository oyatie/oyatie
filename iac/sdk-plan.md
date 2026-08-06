---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud-iac + gtm-customer-success
deciders: axis-cloud-iac, council-architecture
related_adrs: [ADR-0139, ADR-0131]
related_artifacts:
  - iac/contracts/openapi/cloud-iac.yaml
  - iac/contracts/proto/cloud-iac.proto
  - iac/PRD.md (BC layer mapping; sdk per ADR-0105)
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (cloud-iac µservice)

## Purpose

cloud-iac is internal substrate, but it ships SDKs for two consumer categories:
1. **oyatie µservice authors** — read own apply state + drift report; trigger plan-preview programmatically; consume ApplyExecuted events.
2. **Tenant operators (rare)** — read own µservice's apply state via tenant-facing dashboard SDK; future programmatic surface scheduled-for-distinct-tracked-work.

This document specifies SDK strategy: which languages, how generated, what guarantees, sunset policy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie's own language) | First-party authored (`oya-cloud-iac-iac-renderer-sdk` + `oya-cloud-iac-iac-registry-sdk` crates; per PRD BC layer mapping) | axis-cloud-iac |
| **TypeScript** | M01+1 (first external tenant SDK) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-cloud-iac + gtm |
| **Python** | M02 | OpenAPI-generated; published to PyPI | axis-cloud-iac + gtm |
| **Go** | M02 | gRPC-generated baseline + ergonomic wrappers; published as go-module | axis-cloud-iac + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated baseline; published to Maven Central | axis-cloud-iac + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; published to NuGet | axis-cloud-iac + gtm |
| **Ruby / PHP** | scheduled-for-distinct-tracked-work — no current tenant demand | n/a | n/a |

Prioritisation: oyatie's own µservice languages first; then largest tenant developer-population languages (TS + Py lead).

## Generation Strategy

### Rust SDK (first-party)

Two crates per the BC mapping:
- `iac/src/crates/oya-cloud-iac-iac-renderer-sdk/` — render + plan-preview client.
- `iac/src/crates/oya-cloud-iac-iac-registry-sdk/` — registry reads (apply-state, drift report, provenance).

Public surface:
- `RendererClient::new(opts) -> RendererClient; client.trigger_render(...) -> Result<RenderId, Error>`.
- `RegistryClient::new(opts) -> RegistryClient; client.get_apply_state(...) -> Result<ApplyStateRecord, Error>`.

Properties:
- Authentication: client accepts OIDC token provider (closure / trait impl).
- Scope binding: client is bound to a microservice_scope at construction; X-Microservice header automatically populated.
- Retry policy: built-in exponential backoff for transient 5xx and 429.
- Streaming: `client.stream_apply_jobs(...) -> impl Stream<Item=ApplyJob>` via gRPC streaming.
- Re-exports types from `oya-cloud-iac-iac-renderer-kernel` + `-iac-registry-kernel` so consumers see consistent shapes.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Published to oyatie internal crate registry; open-source decision scheduled-for-distinct-tracked-work per Stripe/Twilio precedent.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Generation pipeline (lives in `iac/sdk-generation/`):

1. Source of truth: `contracts/openapi/cloud-iac.yaml` (REST) + `contracts/proto/cloud-iac.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x with per-language generator profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer on top of generated code; provides:
   - First-class auth helpers (OIDC token provider abstraction).
   - microservice_scope binding at client construction.
   - Retry policy + circuit-breaker matching Rust SDK.
   - Idiomatic naming + error handling per language convention.
5. Per-language SDK ships with:
   - README + quick-start.
   - Versioning matches cloud-iac µservice major.minor.
   - Compatibility matrix: which SDK version works with which cloud-iac µservice version.
   - License header + open-source decision recorded.
6. Per-language CI lane: build + lint + integration-test against staging cloud-iac cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Trigger render | `triggerRender(microservice, sha, pack, env)` | `RenderId` |
| Read render result | `getRenderResult(microservice, render_id)` | `RenderedManifest` |
| Plan-preview | `planPreview(microservice, pack, env, sha)` | `PlanPreview` |
| Read apply state | `getApplyState(microservice, env)` | `ApplyStateRecord` |
| Read apply job | `getApplyJob(microservice, apply_id)` | `ApplyJob` |
| Stream apply jobs | `streamApplyJobs(microservice, since)` | streaming `ApplyJob` |
| Read drift report | `getDriftReport(microservice, pack, env)` | `DriftReport` |
| Read provenance | `getProvenance(digest)` | `Provenance` |
| Validate chart signature | `validateChartSignature(digest, sig)` | `{valid, rekor_log_index}` |

Apply + Rollback are NOT exposed via SDK (those are SPIFFE-only paths invoked by internal workers; tenants do not directly trigger apply / rollback).

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OIDC client + scope binding for µservice via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start (per language) | gtm-customer-success |
| Sample workflow: how to consume ApplyExecuted events + render plan-previews | axis-cloud-iac |
| Quarterly SDK update notification (breaking changes 6mo advance) | axis-cloud-iac |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance + auto-migration if possible |
| Breaking API change in cloud-iac µservice | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major |

## Versioning

cloud-iac µservice version: semver.
SDK version per language: matches cloud-iac major.minor; SDK patch independent.
Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API has been stable in production for ≥ 6mo. Default: keep SDKs closed-source until tenant-driven request makes open-source the right move. Matches Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against cloud-iac versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused SDKs flagged for sunset.

## References

- `iac/contracts/openapi/cloud-iac.yaml`.
- `iac/contracts/proto/cloud-iac.proto`.
- `iac/PRD.md` BC layer mapping (`-sdk` per ADR-0105).
- ADR-0105 (13-layer enum; sdk canonical).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Stripe + Twilio SDK precedent.
- `microservices/observability/sdk-plan.md` (parent template).
