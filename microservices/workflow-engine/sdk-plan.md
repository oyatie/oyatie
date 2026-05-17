---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: workflow-engine
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + gtm-customer-success
deciders: axis-workflow, council-architecture
related_adrs: [ADR-0035, ADR-0103, ADR-0131]
related_artifacts:
  - microservices/workflow-engine/contracts/openapi/workflow-engine.yaml
  - microservices/workflow-engine/contracts/proto/workflow-engine.proto
  - microservices/workflow-engine/PRD.md (per-BC sdk crate; closes hyperscaler SDK gap)
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (workflow-engine µservice)

## Purpose

Workload µservices + tenant applications need first-party SDKs to publish workflow events, subscribe to events, start/pause/cancel runs, and query run state. This document specs the SDK strategy: which languages, how generated, what guarantees, sunset policy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M02b (primary; oyatie's own language) | First-party authored (`oya-workflow-engine-{spec-store,execution-engine,event-bus,replay-debugger-backend}-sdk` crates per PRD §"BC layer mapping") | axis-workflow |
| **TypeScript** | M02b+1 (first external-tenant SDK; Studio depends on it) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-workflow + gtm |
| **Python** | M03 | OpenAPI-generated; published to PyPI | axis-workflow + gtm |
| **Go** | M03 | gRPC-generated baseline + ergonomic wrappers; published as go-module | axis-workflow + gtm |
| **JVM (Kotlin / Java)** | M04 | gRPC-generated baseline + ergonomic wrappers; published to Maven Central | axis-workflow + gtm |
| **C# / .NET** | M04+ | OpenAPI-generated; published to NuGet | axis-workflow + gtm |
| **Ruby** | M05+ (only if tenant demand surfaces) | OpenAPI-generated | axis-workflow |
| **PHP** | (none — no current tenant demand) | n/a | n/a |

Prioritisation drivers: oyatie's own µservice languages first (Rust); then largest tenant developer-population languages (TypeScript + Python lead).

## Generation Strategy

### Rust SDK (first-party)

Lives in:
- `microservices/workflow-engine/src/crates/oya-workflow-engine-spec-store-sdk/`
- `microservices/workflow-engine/src/crates/oya-workflow-engine-execution-engine-sdk/`
- `microservices/workflow-engine/src/crates/oya-workflow-engine-event-bus-sdk/`
- `microservices/workflow-engine/src/crates/oya-workflow-engine-replay-debugger-backend-sdk/`

Each crate exposes:
- A `Client::new(opts) -> Client` constructor with OIDC token provider.
- Tenant binding at construction; `tenant_id` server-side-stamped; client cannot override.
- Async methods matching the REST/gRPC surface.
- Built-in exponential backoff retry on 5xx and 429.
- Streaming: `client.subscribe(...) -> impl Stream<Item=WorkflowEvent>` via gRPC bidirectional streaming.
- Re-exports types from the corresponding `*-kernel` and `*-api` crates so consumers see consistent shapes.
- No `unsafe`; `#![deny(unsafe_code)]`.
- Idempotency-key helper utilities (avoid manual UUID generation).
- Replay-safe constants for use inside step bodies (deterministic-replay contract per `policy/spec-integrity.md`).

### Event-Bus SDK Special Surface

The `event-bus-sdk` is the cross-µservice integration adapter. Every other µservice will publish + subscribe via this SDK. It provides:

```rust
// pseudo-code
let client = workflow_engine_event_bus_sdk::Client::new(opts).await?;

// Publish
client.publish::<EmployeeHired>(EmployeeHired {
    tenant_id: server_stamped,  // overwritten server-side
    employee_id: ...,
}, opts).await?;

// Subscribe
let stream = client.subscribe::<PayrollRunCompleted>(filter_opts).await?;
while let Some(event) = stream.next().await {
    // handle (must be idempotent)
}
```

Type registry: at compile time, each event type implements a marker trait `WorkflowEvent`; the SDK refuses publish/subscribe on unregistered types. The registry is itself spec'd at `/specs/workflow-event-registry.json` (governance µservice).

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Generation pipeline (lives in `microservices/workflow-engine/sdk-generation/`, Slice D):

1. Source of truth: `contracts/openapi/workflow-engine.yaml` + `contracts/proto/workflow-engine.proto`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific generator.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer providing:
   - First-class auth helpers (OIDC token provider abstraction).
   - Tenant-context binding at client construction.
   - Retry policy + circuit-breaker matching Rust SDK behavior.
   - Idiomatic naming + error handling per language convention.
   - Type-safe event registry (TypeScript via discriminated unions; Python via Pydantic).
5. Per-language SDK ships with:
   - README + quick-start.
   - Versioning matching workflow-engine µservice major.minor.
   - Compatibility matrix.
   - License header + open-source decision recorded.
6. Per-language CI lane: build + lint + integration-test against staging engine cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Submit workflow spec | `submitWorkflowSpec(spec_id, body, signature)` | `WorkflowSpec` |
| Read workflow spec | `getWorkflowSpec(spec_id, version_sha)` | `WorkflowSpec` |
| Start workflow run | `startWorkflowRun(spec_id, version_sha, input, env)` | `WorkflowRun` |
| Read run state | `getWorkflowRun(run_id)` | `WorkflowRun` |
| Pause / resume / cancel | `pauseRun / resumeRun / cancelRun` | `WorkflowRun` |
| Signal | `signalRun(run_id, signal_name, payload)` | `SignalResponse` |
| Stream run updates | `streamRunUpdates(run_id)` | streaming `RunUpdate` |
| Publish event | `publishEvent(event_type, payload, idempotency_key)` | `PublishResponse` |
| Subscribe to events | `subscribe(filter, opts)` | streaming `WorkflowEvent` |
| Replay events from offset | `replayEvents(sub_id, from, to)` | `ReplayResponse` |
| Start replay session | `startReplay(run_id, from_step, to_step)` | `ReplaySession` |
| Get run analytics | `getRunAnalytics(spec_id, from, to)` | `RunAnalytics` |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue SDK API key + spec-signing key via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: how to publish + subscribe to events in tenant deploy pipeline | axis-workflow |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-workflow |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration to replacement generator if possible |
| Breaking API change in workflow-engine µservice | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major version |

Per ADR-NNNN deprecation-and-migration discipline: every SDK sunset emits an ADR-shaped notice + deprecation-warning in the SDK + tenant comms via gtm-customer-success.

## Versioning

Workflow-engine µservice version: semver.
SDK version per language: matches engine major.minor; SDK patch independent.
Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK open-source decision until SDK API has been stable in production for ≥ 6mo. Default: keep SDKs closed-source until a tenant-driven request makes open-source the right move. Open-source-when-stable matches Stripe + Twilio + Temporal precedent (Temporal SDKs are open-source).

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against engine versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused SDKs flagged for sunset review.

## References

- `microservices/workflow-engine/contracts/openapi/workflow-engine.yaml`.
- `microservices/workflow-engine/contracts/proto/workflow-engine.proto`.
- `microservices/workflow-engine/PRD.md` BC layer mapping.
- ADR-0105 (13-layer enum; `sdk` is one canonical layer).
- `/specs/workflow-event-registry.json` (event type registry; governance µservice).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Temporal SDKs — `docs.temporal.io/sdks/`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
