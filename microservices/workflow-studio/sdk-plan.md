---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + council-design-system + gtm-customer-success
deciders: axis-workflow, council-architecture
related_adrs: [ADR-0065, ADR-0105, ADR-0131]
related_artifacts:
  - microservices/workflow-studio/contracts/openapi.yaml
  - microservices/workflow-studio/contracts/workflow-studio.proto
  - microservices/workflow-studio/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (workflow-studio µservice)

## Purpose

Tenants integrating with workflow-studio programmatically — agentic-developer role (LLM-emitted specs), CI/CD pipelines (git-backed authoring), tenant operator scripts — need first-party SDKs in their workloads' languages. This document specifies the SDK strategy.

Note: Studio's primary user-facing surface is the browser (Leptos WASM canvas). SDKs are for programmatic tenant integrations only. The browser doesn't use an SDK in the conventional sense — it loads the canvas WASM directly.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M03 (oyatie's own language) | First-party authored (`oya-workflow-studio-visual-canvas-sdk` + per-BC SDK crates) | axis-workflow |
| **TypeScript** | M03+1 (first external-tenant SDK; matches browser-tenant integration) | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-workflow + gtm |
| **Python** | M04 (LLM-orchestrator tenants) | OpenAPI-generated; published to PyPI | axis-workflow + gtm |
| **Go** | M04 (CI/CD agent tenants) | gRPC-generated baseline + ergonomic wrappers; go-module | axis-workflow + gtm |
| **JVM (Kotlin / Java)** | M05+ | gRPC-generated baseline; Maven Central | axis-workflow + gtm |
| **C# / .NET** | M05+ | OpenAPI-generated; NuGet | axis-workflow + gtm |
| **Ruby / PHP** | (none — no demand) | n/a | n/a |

Prioritisation drivers: oyatie's own µservices first; then top tenant-developer-population languages (TypeScript for browser-tenant integrations and CI plugins; Python for LLM-orchestrator tenants).

## Generation Strategy

### Rust SDK (first-party)

Lives in `microservices/workflow-studio/src/crates/oya-workflow-studio-{visual-canvas,dsl-emitter,dsl-loader,collab-crdt,node-library-registry,replay-debugger-frontend,license-gate-cedar}-sdk/`.

Per ADR-0105 `-sdk` is a canonical layer. Each BC's SDK crate is first-party authored with:
- `Client::new(opts) -> Client; client.method(...) -> Result<...>`
- OIDC token provider abstraction (closure or trait).
- Tenant-context binding at construction; `X-Scope-OrgID` header automatically populated.
- Retry policy: exponential backoff for 5xx + 429.
- gRPC streaming via `tonic` for collab-crdt + replay-debugger-frontend.
- Re-exports kernel types so consumers see consistent shapes.
- `#![deny(unsafe_code)]`.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Generation pipeline at `microservices/workflow-studio/sdk-generation/`:

1. Source of truth: `contracts/openapi.yaml` (REST) + `contracts/workflow-studio.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x with language-specific profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer; provides:
   - OIDC auth helpers.
   - Tenant-context binding at client construction.
   - Retry policy + circuit-breaker matching Rust SDK behavior.
   - Per-language idiom (camelCase methods in TS, snake_case in Python, etc.).
5. Per-language SDK ships with:
   - README + quick-start.
   - Versioning matching workflow-studio major.minor.
   - Compatibility matrix.
   - License header + open-source decision recorded.
6. Per-language CI lane: build + lint + integration-test against staging Studio cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Open editor session | `openEditorSession(definition_id)` | `EditorSession` |
| Get editor session | `getEditorSession(session_id)` | `EditorSession` |
| Close editor session | `closeEditorSession(session_id)` | void |
| Load workflow definition | `loadWorkflowDefinition(definition_id, jurisdiction?)` | `{spec, jurisdiction_view}` |
| Save workflow definition | `saveWorkflowDefinition(definition_id, spec, cedar_preview_acknowledged?)` | `SaveResponse` |
| Switch jurisdiction overlay | `switchJurisdictionOverlay(definition_id, jurisdiction)` | `JurisdictionOverlayView` |
| List node libraries | `listNodeLibraries(pack?)` | `NodeLibraryDescriptor[]` |
| Get node library | `getNodeLibrary(library_id)` | `NodeLibraryDescriptor` |
| Request LLM-assist draft | `llmAssistDraft(definition_id, prose, consent_acknowledged, target_jurisdiction?)` | `LlmAssistDraftResponse` |
| Open debugger session | `openDebuggerSession(run_id)` | `DebuggerSession` |
| Stream debugger frames | `streamDebuggerFrames(session_id, from_seq)` | stream of `DebuggerFrame` (gRPC streaming) |
| Resync debugger session | `resyncDebuggerSession(session_id, from_seq)` | `DebuggerSession` |
| Stream CRDT ops (bidirectional) | `streamCrdtOps(...)` | stream `CrdtOpAck` (gRPC bidi streaming) |

Schema authoring is via the browser editor + git PR; SDKs do NOT expose a "create node library" or "publish marketplace template" method — those paths require 2-person rule + signing-key access per `threat-model.md` T-S-04.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OIDC client + tenant-scoped SDK API key via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: spec round-trip via Rust SDK (developer persona) | axis-workflow |
| Sample workflow: LLM-orchestrator tenant submits draft via Python SDK | axis-workflow + foundry-providers |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-workflow |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% of tenant usage for ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib is deprecated upstream | dep-deprecated | 12mo advance notice + auto-migration to replacement |
| Breaking API change in workflow-studio | per-release | major version bump in SDK; backwards-compatible adapter for 1 prior major version |

## Versioning

- workflow-studio µservice version: semver.
- SDK version per language: matches workflow-studio major.minor; SDK patch independent.
- Compatibility matrix: published per-language; CI lane verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK open-source decision until API has been stable in production for ≥ 6mo. Default: keep SDKs closed-source until tenant-driven request OR competitive consideration makes open-source the right move. Same precedent as observability µservice (Stripe + Twilio model).

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against workflow-studio versions N-1, N, N+1.
- Annual SDK telemetry review: usage per SDK; underused flagged for sunset review.

## References

- `microservices/workflow-studio/contracts/openapi.yaml`.
- `microservices/workflow-studio/contracts/workflow-studio.proto`.
- ADR-0105 `sdk` canonical layer.
- ADR-0131 per-microservice flat layout.
- OpenAPI Generator — `openapi-generator.tech`.
- tonic (Rust gRPC) — `github.com/hyperium/tonic`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- Twilio SDK precedent — `twilio.com/docs/libraries`.
