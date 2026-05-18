---
doc_class: SdkPlan
title: SDK + Client-Bindings Plan (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry-control-plane + gtm-customer-success
deciders: axis-foundry-control-plane, council-architecture
related_adrs: [ADR-0130, ADR-0131]
related_artifacts:
  - microservices/foundry-supervisor/contracts/openapi/foundry-supervisor.yaml
  - microservices/foundry-supervisor/contracts/proto/foundry-supervisor.proto
  - microservices/foundry-supervisor/PRD.md
review_cadence: per-SDK-language-launch
doc_status: published
---

# SDK + Client-Bindings Plan (foundry-supervisor µservice)

## Purpose

Tenants and internal teams interacting with the foundry-supervisor control plane need first-party SDKs in the languages their workloads use. Document the SDK strategy: which languages, generation strategy, guarantees, sunset policy.

## Languages

| Language | Priority | Generation strategy | Authority |
|---|---|---|---|
| **Rust** | M01 (primary) | First-party authored (`oya-foundry-supervisor-*-sdk` crates per BC) | axis-foundry-control-plane |
| **TypeScript** | M01+1 | OpenAPI-generated + ergonomic wrappers; published to npm | axis-foundry-control-plane + gtm |
| **Python** | M02 | OpenAPI-generated; published to PyPI | axis-foundry-control-plane + gtm |
| **Go** | M02 | gRPC-generated + ergonomic wrappers | axis-foundry-control-plane + gtm |
| **JVM (Kotlin / Java)** | M03 | gRPC-generated + ergonomic wrappers; Maven Central | axis-foundry-control-plane + gtm |
| **C# / .NET** | M03-onward | OpenAPI-generated; NuGet | axis-foundry-control-plane + gtm |
| **Ruby** | M04-onward (only if tenant demand) | OpenAPI-generated | axis-foundry-control-plane |
| **PHP** | (none) | n/a | n/a |

Prioritisation: oyatie's own µservice languages first; then largest tenant developer-population languages (TypeScript + Python lead).

## Generation Strategy

### Rust SDK (first-party)

One per BC at `microservices/foundry-supervisor/src/crates/oya-foundry-supervisor-<bc>-sdk/`:

- `oya-foundry-supervisor-agent-fleet-lifecycle-sdk`
- `oya-foundry-supervisor-capability-deployment-sdk`
- `oya-foundry-supervisor-autonomy-policy-enforcement-sdk`
- `oya-foundry-supervisor-supervision-event-bus-sdk`
- `oya-foundry-supervisor-kill-switch-circuit-breaker-sdk`

Public surface per crate: `Client::new(opts) -> Client; client.<method>(...)`.

- Authentication: OIDC token provider (closure / trait).
- Tenancy: bound to a tenant at construction; `X-Scope-OrgID` populated.
- Two-person rule: `engage_kill_switch_fleet_wide(...)` takes two `SignatureBundle` arguments; type-system enforces.
- Retry: exponential backoff for transient 5xx + 429.
- Streaming: gRPC streaming where applicable (`stream_kill_switch_events`, `stream_fleet_state`).
- Re-exports types from `oya-foundry-supervisor-*-kernel` for consistent shapes.
- `#![deny(unsafe_code)]`.

### Generated SDKs (TypeScript / Python / Go / JVM / C#)

Pipeline at `microservices/foundry-supervisor/sdk-generation/`:

1. Source: `contracts/openapi/foundry-supervisor.yaml` + `contracts/proto/foundry-supervisor.proto`.
2. OpenAPI → language: `openapi-generator-cli` 7.x with per-language profile.
3. Proto → language: `protoc` + per-language plugin.
4. Ergonomic wrapper: hand-authored thin layer providing:
   - First-class auth helpers (OIDC).
   - Tenant-context binding at client construction.
   - Two-person-rule enforcement at type/structure level for `engage_kill_switch` with `scope=fleet`.
   - Retry policy + circuit-breaker matching Rust SDK.
   - Idiomatic per-language naming.
5. Per-language SDK ships with README + quick-start, versioning matching `foundry-supervisor` major.minor, compatibility matrix, license header.
6. Per-language CI lane: build + lint + integration-test against staging supervisor cluster.

## Public Surface (across languages)

All SDKs expose:

| Capability | Method | Returns |
|---|---|---|
| Admit capability | `admitCapability(tenant, capability_id, yaml)` | `AgentDeployment` |
| List capabilities | `listCapabilities(tenant)` | `CapabilityDefinition[]` |
| List deployments | `listDeployments(tenant, capability_id)` | `AgentDeployment[]` |
| Rollback deployment | `rollbackDeployment(tenant, deployment_id, target_version, reason)` | `AgentDeployment` |
| Get fleet state | `getFleetState(tenant)` | `FleetState` |
| Drain fleet | `drainFleet(tenant, grace_period_seconds, reason)` | `DrainHandle` |
| Stream fleet state | `streamFleetState(tenant, since)` | stream `FleetState` |
| Engage kill-switch (scope ≠ fleet) | `engageKillSwitch(scope, target, reason, signature)` | `KillSwitch` |
| Engage kill-switch (scope = fleet) | `engageKillSwitchFleetWide(reason, signature1, signature2)` | `KillSwitch` |
| Disengage kill-switch | `disengageKillSwitch(scope, target, reason, signatures)` | `KillSwitch` |
| Get kill-switch state | `getKillSwitchState(scope?, target?)` | `KillSwitch[]` |
| Stream kill-switch events | `streamKillSwitchEvents(tenant, since)` | stream `KillSwitchEvent` |
| List autonomy entitlements | `listAutonomyEntitlements(tenant)` | `AutonomyEntitlement[]` |
| Evaluate autonomy precondition (foundry-runtime only) | `evaluateAutonomyPrecondition(tenant, capability, tier)` | `AutonomyDecision` |
| Validate capability schema | `validateCapabilitySchema(yaml)` | `ValidationResult` |

## Tenant SDK Onboarding

| Step | Owner |
|---|---|
| Issue OIDC client + per-tenant SPIFFE bridge | ops-security |
| Provide tenant onboarding doc with SDK quick-start per language | gtm-customer-success |
| Sample workflow: capability admit + monitoring + kill-switch | axis-foundry-control-plane |
| Quarterly SDK update notification (breaking changes 6 mo advance notice) | axis-foundry-control-plane |

## Sunset Policy

| SDK | Sunset trigger | Window |
|---|---|---|
| Any SDK < 1 % usage for ≥ 12 mo | underused | 6 mo notice + migration help |
| SDK whose generator lib is upstream-deprecated | dep-deprecated | 12 mo notice + auto-migration |
| Breaking API change in supervisor | per-release | major version bump; backwards-compat adapter 1 prior major |

Per deprecation-and-migration discipline: every SDK sunset emits an ADR-shaped notice + deprecation-warning in the SDK + tenant comms.

## Versioning

- Supervisor µservice: semver.
- SDK per language: matches supervisor major.minor; SDK patch independent.
- Compatibility matrix per-language; CI lane verifies SDK against current + 1 prior major.

## Open-Source Decision

Defer per-SDK OSS decision until SDK API stable in production ≥ 6 mo. Default closed-source until tenant-driven request or competitive consideration. Open-source-when-stable matches Stripe + Twilio precedent.

## Verification

- Per-SDK CI lane: build + lint + integration-test exit 0.
- Per-SDK compatibility test against supervisor versions N-1, N, N+1.
- Annual SDK telemetry review.

## References

- `microservices/foundry-supervisor/contracts/openapi/foundry-supervisor.yaml`.
- `microservices/foundry-supervisor/contracts/proto/foundry-supervisor.proto`.
- `microservices/foundry-supervisor/PRD.md` BC layer mapping (`-sdk` crate per ADR-0105).
- ADR-0105 (13-layer enum; `sdk` canonical layer).
- OpenAPI Generator — `openapi-generator.tech`.
- gRPC — `grpc.io`.
- Stripe SDK precedent — `stripe.com/docs/libraries`.
- AWS Bedrock Agents SDK precedent — `docs.aws.amazon.com/bedrock/latest/userguide/agents.html`.
