---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-guardrails + gtm-customer-success
deciders: axis-foundry-guardrails, council-architecture
related_adrs: [ADR-0022, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry-guardrails/contracts/openapi/guardrails.yaml
  - microservices/foundry-guardrails/contracts/proto/guardrails.proto
  - microservices/foundry-guardrails/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (foundry-guardrails µservice)

## Purpose

Tenants composing per-tenant Cedar overlays + querying decision history + receiving stream of guardrail decisions need first-party SDKs. This document specifies SDK strategy: languages, generation, guarantees, sunset.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary; oyatie's language) | First-party authored `oya-foundry-guardrails-prompt-classifier-sdk` + sibling-BC SDKs | axis-foundry-guardrails |
| **TypeScript** | M01+1 | OpenAPI-generated baseline + first-party ergonomic wrappers; published to npm | axis-foundry-guardrails + gtm |
| **Python** | M02 | OpenAPI-generated; PyPI | axis-foundry-guardrails + gtm |
| **Go** | M02 | gRPC-generated baseline + ergonomic wrappers; go-module | axis-foundry-guardrails + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated baseline; Maven Central | axis-foundry-guardrails + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; NuGet | axis-foundry-guardrails + gtm |
| **Ruby** | M04-onward (on demand only) | OpenAPI-generated | axis-foundry-guardrails |

Prioritisation: oyatie's own µservice languages first; then largest tenant developer-population.

## Generation Strategy

### Rust SDK (first-party)

Per BC, one SDK crate at `microservices/foundry-guardrails/src/crates/oya-foundry-guardrails-<bc>-sdk/`.

- Public surface (prompt-classifier):
  - `Client::new(opts) -> Client`
  - `client.classify_prompt(prompt, ctx) -> Result<Classification, ClassifyError>`
  - `client.validate_output(output, ctx) -> Result<Validation, ValidateError>`
  - `client.stream_decisions(filter) -> impl Stream<Item=GuardrailDecision>`
- Authn: OIDC bearer + per-call SPIFFE attestation when in-cluster.
- Tenancy: `Client` bound to tenant at construction; `X-Scope-OrgID` populated.
- Retry: exponential backoff for 5xx / 429.
- Re-exports types from `*-api` crates so consumers see consistent shapes.
- `#![deny(unsafe_code)]`.
- Published to oyatie internal registry; open-source decision scheduled-for-distinct-tracked-work until ≥ 6mo stable.

### Generated SDKs (TS / Python / Go / JVM / C#)

Generation pipeline at `microservices/foundry-guardrails/sdk-generation/`.

1. Source of truth: `contracts/openapi/guardrails.yaml` (REST) + `contracts/proto/guardrails.proto` (gRPC).
2. OpenAPI → language: `openapi-generator-cli` 7.x per-language profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer; first-class auth + tenancy + retry.
5. Per-language SDK ships: README + quick-start + versioning matching guardrails major.minor + compatibility matrix.
6. Per-language CI lane: build + lint + integration-test against staging guardrails cluster.

## Public Surface (across languages)

| Capability | Method | Returns |
|---|---|---|
| Classify prompt (pre-invocation) | `classifyPrompt(prompt, ctx)` | `Classification {verdict, block_reason?, cedar_policy_ids[], classifier_model_versions{}, audit_chain_id}` |
| Validate output (post-invocation) | `validateOutput(output, ctx)` | `Validation {verdict, block_reason?, redaction_diff?, ...}` |
| Stream decisions (own tenant) | `streamDecisions(filter)` | streaming verdicts |
| Read decision detail | `getDecisionDetail(decision_id)` | full detail (Art. 22 explanation) |
| List Cedar overlay fragments (own tenant) | `listTenantOverlays()` | overlay metadata |
| Read Cedar overlay (own tenant) | `getTenantOverlay(overlay_id)` | Cedar fragment text |
| Mark decision as false-positive (FP budget) | `markFalsePositive(decision_id, reason)` | budget remaining |
| Read FP budget | `getFalsePositiveBudget()` | `{used, total, reset_at}` |
| Read jailbreak incident (own tenant) | `getJailbreakIncident(incident_id)` | incident summary |
| List shadow-vs-enforce delta (rule-author scope only) | `listShadowDeltas()` | rule-author dashboard |

**Cedar overlay authoring path**: via git PR (CODEOWNERS + branch-protection enforced); SDK does NOT expose a "create/update Cedar overlay" method. If future tenant feedback surfaces a need for programmatic overlay authoring, that's a per-tenant DPA-recorded entitlement granted via Cedar permit fragment.

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OIDC client + per-tenant scope tokens via OpenBao | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: how to read decision detail in tenant operator UI | axis-foundry-guardrails |
| Quarterly SDK update notification (breaking changes 6mo advance notice) | axis-foundry-guardrails |

## Sunset Policy

| SDK | Sunset trigger | Sunset window |
|---|---|---|
| Any SDK with < 1% tenant usage ≥ 12mo | underused | 6mo advance notice + migration help |
| Any SDK whose generator lib deprecated upstream | dep-deprecated | 12mo + auto-migrate where possible |
| Breaking API change in guardrails µservice | per-release | major bump; backwards-compatible adapter for 1 prior major |

Per ADR-#### deprecation-and-migration discipline: every SDK sunset emits ADR-shaped notice + deprecation-warning in SDK + tenant comms via gtm.

## Versioning

guardrails µservice: semver.
SDK per language: matches guardrails major.minor; SDK patch independent.
Compatibility matrix: published per-language; CI verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK OSS decision until SDK API stable ≥ 6mo. Default: keep closed-source until tenant-driven request OR competitive consideration. Matches Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test: SDK version N+1 works against guardrails versions N-1, N, N+1.
- Annual SDK telemetry review.

## References

- `microservices/foundry-guardrails/contracts/openapi/guardrails.yaml`.
- `microservices/foundry-guardrails/contracts/proto/guardrails.proto`.
- `microservices/foundry-guardrails/PRD.md` (BC layer mapping — `-sdk` per ADR-0105).
- ADR-0105 (13-layer enum; `sdk` canonical layer).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC tooling — `grpc.io`.
- Stripe SDK precedent.
- Twilio SDK precedent.
- `microservices/observability/sdk-plan.md` (sibling shape).
