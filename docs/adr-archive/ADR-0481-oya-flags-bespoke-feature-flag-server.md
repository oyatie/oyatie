---
id: ADR-0481
title: "oya-flags: bespoke Rust feature flag server superseding flagd"
status: Superseded
date: 2026-05-28
authority: founder
amends: [ADR-0428]
milestone: M-FEATURE-FLAGS-V2
planning_impact: true
supersedes: []
superseded_by: [ADR-0709]
amended_by: [ADR-0632]
related: [ADR-0428, ADR-0083, ADR-0407, ADR-0411, ADR-0397, ADR-0476, ADR-0408, ADR-0509]
deliverables:
  - id: D1
    description: "New µservice microservices/oya-flags/ — Rust workspace, Axum public REST plus internal-only gRPC/proto3 over HTTP/2. Speaks the OpenFeature flag-evaluation protocol (ADR-0428 SDK compat retained). PostgreSQL backend for flag definitions; in-memory hot cache for evaluation hot-path."
    exit_criteria: "microservices/oya-flags/src/ compiles; cargo nextest -p oya-flags passes; OpenFeature provider test client resolves a boolean flag against the running server."
    verified_by: "cargo nextest -p oya-flags + cargo clippy -p oya-flags -- -D warnings"
  - id: D2
    description: "Targeting rules: tenant-id, region, user, percentage rollout, time-bound, custom Cedar (ADR-0083) policy expressions. Sub-millisecond evaluation latency via in-process cache."
    exit_criteria: "EvaluationEngine resolves all six targeting dimensions; targeting unit tests cover 100% of rule combinator paths; p99 evaluation latency < 1 ms in benchmark harness."
    verified_by: "cargo nextest -p oya-flags (targeting suite) + cargo bench -p oya-flags eval_latency"
  - id: D3
    description: "Flag-as-code: definitions in oya-vcs (ADR-0409) GitOps repo; Buck2 CI (ADR-0408) validates schema; flag changes flow through PR review. Audit trail = git history."
    exit_criteria: "Buck2 gate oya-gate-validate-flag-schema is BLOCKER on flag bundle PRs; schema validator rejects malformed bundles; well-formed bundles pass and are applied to the running server within 30 s."
    verified_by: "planned hermetic cloud-ci/Buck2 flag-schema gate target (must be wired before D3 completion) + integration test: apply bundle -> evaluate flag"
  - id: D4
    description: "OTel (ADR-0407) integration: every evaluation emits span with flag_key + variation + tenant. Cloud-intelligence routing (ADR-0384) uses oya-flags for Cluster I/II/III selection."
    exit_criteria: "feature_flag.key and feature_flag.value OTel span attributes present on every evaluation span in Grafana Tempo; cloud-intelligence routing reads cluster-tier flag from oya-flags on every request."
    verified_by: "cargo nextest -p oya-flags (otel suite) + trace in Tempo shows feature_flag.* attributes"
  - id: D5
    description: "Tenant flag plane via Crossplane (ADR-0411) TenantApplication XR. Tenants set per-tenant flags via Self-Service UI (ADR-0434). oya-identity (ADR-0476) human auth + SPIFFE service auth."
    exit_criteria: "TenantApplication XR provisions a per-tenant flag namespace; Self-Service UI flag panel lists and toggles tenant-scoped flags; Cedar policy forbids cross-tenant flag writes."
    verified_by: "cloud-ci/Rust honest-claims gate packet + Self-Service UI integration test: toggle flag -> evaluate flag returns updated variation"
owner: council-platform
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.

# ADR-0481 — oya-flags: bespoke Rust feature flag server superseding flagd

## Status

Accepted — 2026-05-28 (founder-locked). Amends ADR-0428 (server provider only; OpenFeature SDK
protocol unchanged).

## ADR-0632 product-protocol reconciliation

Tenant and operator flag APIs use public HTTPS REST documented by OpenAPI 3.2.0, with signed/versioned webhooks, AsyncAPI/CloudEvents events, SSE, or WebSocket used where their semantics apply. Public GraphQL, gRPC, gRPC-Web, and Connect are forbidden. The OpenFeature gRPC adapter is internal-only gRPC/proto3 over HTTP/2.

## Context

ADR-0428 adopted the OpenFeature SDK + flagd DaemonSet as the feature flag substrate. The OpenFeature
SDK protocol is retained as the standard evaluation API. However, flagd is a Go binary with its own
configuration DSL, limited targeting expressiveness, and no native Cedar integration. It adds an
external dependency that diverges from the Rust-first doctrine.

**Hyperscaler precedent for bespoke flag servers:**

- **Google** — internal Gatekeeper flag system; OpenFeature SDK provides the standard evaluation
  protocol for external consumers but the server is bespoke.
- **Meta** — GateKeeper / ExperimentFramework; fully internal, open SDK compat layer for third-party
  integration.
- **Stripe** — internal flag server; standard evaluation API exposed to application code.

All three chose bespoke servers because full control over targeting logic, storage layout, audit
semantics, and policy enforcement is necessary at scale. The OpenFeature SDK provides the open
compatibility protocol — the server behind it is an implementation detail.

Oyatie must own its flag evaluation semantics to integrate Cedar targeting expressions, SPIFFE service
auth, and the PostgreSQL-backed GitOps audit trail without depending on flagd's roadmap.

## Decision

Replace flagd with **oya-flags**, a bespoke Rust feature flag server built on Axum for public HTTPS REST plus internal-only gRPC/proto3 over HTTP/2, speaking the OpenFeature flag-evaluation protocol. ADR-0428 SDK adoption is preserved; only the
server provider changes.

- **Server**: `microservices/oya-flags/` — Rust and Axum. It exposes public HTTPS REST and implements the OpenFeature flag-evaluation gRPC protocol only for internal sibling-service calls so existing SDK wiring (ADR-0428 Phase 2) continues unchanged.
- **Storage**: PostgreSQL (ADR-0406) for flag definitions. In-memory hot cache for evaluation
  hot-path; cache invalidated on flag bundle push.
- **Targeting**: tenant-id, region, user, percentage rollout, time-bound windows, and arbitrary
  Cedar (ADR-0083) policy expressions. Sub-millisecond evaluation via in-process cache.
- **Flag-as-code**: flag bundles in oya-vcs (ADR-0409) GitOps repo. Buck2 CI (ADR-0408) runs
  schema validation as a BLOCKER gate. Audit trail = git history.
- **Observability**: every evaluation emits OTel (ADR-0407) span with `feature_flag.key`,
  `feature_flag.value`, and `tenant_id`. Cloud-intelligence routing (ADR-0384) reads
  Cluster I/II/III selection flags from oya-flags.
- **Auth**: oya-identity (ADR-0476) for human operator/tenant auth; SPIFFE mTLS for service-to-service
  evaluation calls. Cedar guards flag-modification API; per-tenant scope enforced at admission.
- **Tenant plane**: Crossplane (ADR-0411) `TenantApplication` XR provisions a per-tenant flag
  namespace. Tenants toggle flags via Self-Service UI (ADR-0434).

## Hyperscaler-lens

| Criterion | Result |
|---|---|
| Active upstream | Bespoke IS the Google/Meta/Stripe pattern; oya-flags is fully in-house. |
| Clean license | All deps (axum, tokio, serde, sqlx, tonic) are MIT/Apache 2. |
| Fully self-hostable | Single Rust binary; no external managed service required. |
| Hyperscaler-equivalent | Google Gatekeeper, Meta ExperimentFramework, Stripe flags — all bespoke servers with open SDK compat. |

All four criteria pass.

## Alternatives

| Alternative | Reason not chosen |
|---|---|
| flagd (ADR-0428 Phase-1) | Go binary; no Cedar integration; flagd DSL limits targeting expressiveness; external roadmap dependency. |
| LaunchDarkly | Commercial managed SaaS; violates self-hostable requirement. |
| Unleash | Bespoke API, not OpenFeature-native; adds Go/Node dependency to a Rust-first platform. |

## Consequences

**Positive**
- Full control over targeting logic, Cedar integration, storage schema, and audit semantics.
- Rust-first doctrine maintained; no Go runtime in the flag evaluation path.
- OpenFeature SDK compat retained — application code is unaffected by the server swap.
- Sub-millisecond evaluation via in-process cache; no per-request network hop to a DaemonSet.

**Negative**
- ~4–6 month investment to reach feature parity with flagd and ship D1–D5. Accepted per founder
  direction 2026-05-28.

## Integration

| System | Integration point |
|---|---|
| Cedar (ADR-0083) | Cedar expressions in targeting rules; flag-modification API authz. |
| OTel (ADR-0407) | `feature_flag.key`/`value`/`tenant_id` span attributes on every evaluation. |
| ADR-0384 cloud-intelligence | Cluster I/II/III routing reads flag from oya-flags on every request. |
| Buck2-CI (ADR-0408) | Schema validation gate on flag bundle PRs. |
| Crossplane (ADR-0411) | `TenantApplication` XR provisions per-tenant flag namespace. |
| oya-identity (ADR-0476) | Human + SPIFFE auth for flag API. |
| Self-Service UI (ADR-0434) | Tenant flag panel reads/writes via oya-flags API. |

## Promotion Rationale

Bespoke flag server is the hyperscaler pattern. OpenFeature SDK provides the stable open protocol;
oya-flags owns the server implementation. Full Rust doctrine, Cedar targeting, and PostgreSQL
audit trail are only achievable with an in-house server. The ~4–6 month cost is justified by
permanent elimination of the flagd external dependency and full control over the flag evaluation
substrate.

## Implementation pattern (ADR-0509 alignment)

Per ADR-0509 (Hyperscaler service decomposition pattern), `oya-flags` ships as **single-crate-per-service with mod-based subsystems**. Per-use-case crate sprawl is superseded. Use cases remain valid as domain concepts (subsystem boundaries inside `src/<subsystem>/`).
