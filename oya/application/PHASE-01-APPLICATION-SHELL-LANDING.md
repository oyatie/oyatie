---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P01-application-shell-landing
status: Active
entry_gate: |
  ADR-0121 (Modular Product Shell) inherited into oyatie glossary as "application"; ADR-0123 cross-product auth contract accepted; ADR-0065 Leptos framework accepted; ADR-0131 flat layout accepted; observability µservice (PRD-observability) shipped with HG-OBS gate green; tenancy µservice in-flight; /specs/per-microservice-flat-layout.json + /specs/agentic-slo-gated-promotion.json published; Cargo workspace ready to accept the 44 new crates under microservices/application/src/crates/.
exit_gate: |
  All 15 IPs merged; oya-governance-promotion-readiness CI lane green for release/application/*; HG-APP gate in /specs/hyperscaler-gates.json registers green; cargo nextest run --workspace exits 0; buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice application exits 0; buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion --microservice application exits 0; pack-kr Application Shell deployed to staging with TTI p99 ≤2s under 10k synthetic concurrent users; CDN global purge drill completes ≤60s; module-loader rejects tampered bundle (SRI + signature drills).
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: observability substrate must precede every µservice's first ship per ADR-0139 bootstrap order
  - milestone: M02-tenant-substrate
    phase: tenancy phase 01
    reason: tenant-context BC consumes tenancy's tenant-resolver + JWT issuance
owner_team: axis-application
related_adrs: [ADR-0056, ADR-0065, ADR-0105, ADR-0117, ADR-0121, ADR-0123, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-application-shell-landing: Land the Application Shell end-to-end

## Purpose

Ship the Application Shell as the tenant front door, with:

- Leptos WASM frontend served from CDN; sub-2-s TTI;
- OIDC + SAML auth gateway with two-cookie + PKCE + nonce contract;
- Cedar-gated shell routing per tenant + per role;
- Module loader with SRI + Ed25519 signed integrity verification;
- Pack-pinned CDN + Postgres + Valkey session store under the M01 pack-kr footprint;
- HG-APP hyperscaler-maturity gate registered + green.

This phase advances master-plan principles:

- Hyperscaler-grade in every practice (Vercel / Linear / Stripe Dashboard / Foundry App Shell parity).
- Nothing scheduled-for-distinct-tracked-work (signed module manifest, Cedar gates, audit trail all land in P01).
- No silent regression (production-tier breach auto-reverts via the rollback primitive inherited from observability).
- Per-microservice flat layout (this phase ships natively under ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `application` | `shell-routing`, `tenant-context`, `auth-gateway`, `module-loader`, `frontend-bundle-serve` | All under `microservices/application/` per ADR-0131 | `oya-application-{shell-routing,tenant-context,auth-gateway,module-loader,frontend-bundle-serve}-{kernel,domain,usecase,api,adapter,adapter-*,rest,worker,sdk,app}` |

Plus these repo-wide artifacts:

- `.github/branch-protection.yaml` — add `oya-application-tti-budget` and `oya-application-route-resolve-budget` to required checks on `release/application/staging` and `release/application/production`.
- `Cargo.toml` (workspace) — register the 44 new crates.
- `/specs/hyperscaler-gates.json` — register HG-APP gate per ADR-0123.
- `microservices/application/slos/*.openslo.yaml` — OpenSLO manifests for TTI + route-resolve + sign-in + module-load SLIs.

### Out-of-scope

- Native client tiers (iOS / Android / desktop / CLI / VR) — per Bominal ADR-0209 inheritance; tracked in subsequent-to-M03-completion phase.
- SCIM 2.0 provisioning — covered by `tenancy` µservice; this phase consumes the SCIM provisioner via tenant-context.
- Billing UX — covered by `billing` µservice; this phase exposes a navigation slot only.
- Web Worker / iframe sandbox decision for module-loader — Open Question 2; tracked under successor-IP ADR; this phase ships with iframe-postMessage default per ADR-0123.

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-shell-routing-kernel.md`](IP-001-shell-routing-kernel.md) | `oya-application-shell-routing-kernel` — port traits + entities (Route, RouteScope, RouteRegistration) | pending | axis-application | — |
| [`IP-002-shell-routing-domain.md`](IP-002-shell-routing-domain.md) | `oya-application-shell-routing-domain` — pure route-matching algebra; longest-prefix + scope-set intersection | pending | axis-application | IP-001 |
| [`IP-003-shell-routing-usecase.md`](IP-003-shell-routing-usecase.md) | `oya-application-shell-routing-usecase` — ResolveRoute + RegisterRoute orchestrators | pending | axis-application | IP-002 |
| [`IP-004-shell-routing-adapter.md`](IP-004-shell-routing-adapter.md) | `oya-application-shell-routing-adapter` — Postgres-backed RouteRegistry + in-memory LRU cache | pending | axis-application | IP-001 |
| [`IP-005-shell-routing-rest.md`](IP-005-shell-routing-rest.md) | `oya-application-shell-routing-rest` — axum router with Cedar policy gate middleware | pending | axis-application | IP-003, IP-004 |
| [`IP-006-tenant-context-kernel.md`](IP-006-tenant-context-kernel.md) | `oya-application-tenant-context-kernel` — port traits (TenantResolver, TenantBindingStore) + entities | pending | axis-application | — |
| [`IP-007-tenant-context-usecase-rest.md`](IP-007-tenant-context-usecase-rest.md) | usecase + adapter + rest layers for tenant-context | pending | axis-application | IP-006 |
| [`IP-008-auth-gateway-kernel-domain.md`](IP-008-auth-gateway-kernel-domain.md) | kernel + domain crates for auth-gateway (Session, OidcProvider, SamlAssertion, Mfa) | pending | axis-application + ops-security | — |
| [`IP-009-auth-gateway-adapters-oidc-saml.md`](IP-009-auth-gateway-adapters-oidc-saml.md) | `adapter-oidc` + `adapter-saml` crates implementing IdP protocols | pending | axis-application + ops-security | IP-008 |
| [`IP-010-auth-gateway-rest-worker.md`](IP-010-auth-gateway-rest-worker.md) | rest layer (sign-in handlers) + worker (session-rotation reaper) | pending | axis-application | IP-009 |
| [`IP-011-module-loader-kernel-domain.md`](IP-011-module-loader-kernel-domain.md) | kernel + domain crates for module-loader (Module, ModuleManifest, IntegrityClaim) | pending | axis-application | — |
| [`IP-012-module-loader-usecase-adapter-cdn.md`](IP-012-module-loader-usecase-adapter-cdn.md) | usecase + adapter + adapter-cdn — SRI + signature verification + CDN bundle fetch | pending | axis-application | IP-011 |
| [`IP-013-frontend-bundle-serve.md`](IP-013-frontend-bundle-serve.md) | frontend-bundle-serve crates: kernel + usecase + adapter-cdn + adapter-postgres + worker | pending | axis-application | IP-012 |
| [`IP-014-leptos-frontend-and-composition.md`](IP-014-leptos-frontend-and-composition.md) | Leptos WASM frontend crate; composition-root `oya-application-*-app` binaries | pending | axis-application | IP-005, IP-007, IP-010, IP-012, IP-013 |
| [`IP-015-application-openslo-and-hg-app.md`](IP-015-application-openslo-and-hg-app.md) | Author OpenSLO manifests + register HG-APP gate; branch-protection lane wiring | pending | axis-application + axis-observability | IP-014 |

Coverage check vs. PRD §"Bounded Contexts": all five BCs covered; the
44 crate slots are reached by IPs 001..014 with IP-015 wiring SLO + gate.

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=statelessness --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=shardability --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=data-class --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=cedar-policy --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=audit-chain --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion --microservice application
buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity-claims --microservice application
```

### End-to-end drills

| Drill | Pass criterion |
|---|---|
| TTI p99 ≤ 2 s under 10 k synthetic concurrent users (k6 + Lighthouse) | green |
| Module-loader rejects tampered bundle (SRI mutation) | green |
| Module-loader rejects bundle signed with revoked key | green |
| `EmployeeTerminated` → session revocation in ≤ 5 s | green |
| CDN global purge ≤ 60 s | green |
| Cross-tenant route attempt fails closed | green |

## Per-IP Test Coverage Threshold

| Layer class | Line cov | Branch cov | Test classes |
|---|---|---|---|
| kernel | 95 % | 80 % | invariant + serde + sealed-trait + data-class annotation |
| domain | 95 % | 90 % | pure algebra + property tests |
| usecase | 90 % | 80 % | orchestrator + happy-path + error-path |
| adapter | 85 % | 75 % | port-impl + I/O mocks |
| rest | 80 % | 70 % | handler + middleware + auth |
| worker | 80 % | 70 % | loop + shutdown + retry |
| sdk | 90 % | 80 % | client + retry + auth-injection |
| app | 60 % | n/a | composition smoke |
| adapter-* (backend-qualified) | 85 % | 75 % | backend mocks |

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Leptos hydration regression breaks TTI | medium | high | Pin Leptos to LTS; lane runs Lighthouse on every PR |
| OIDC IdP outage cascades to all tenants | low | high | Fall back to SAML; degraded-but-readable mode for static surfaces |
| CDN POP cache poisoning | low | critical | SRI + signed manifest; pack-pinned origin shield |
| Module bundle supply-chain compromise | medium | critical | Ed25519 signed manifest; CODEOWNERS on bundle publish path |
| Session-store memory pressure at peak | medium | medium | Sentinel/Cluster + auto-scaling + eviction policy |
| Cross-tenant route confusion (URL guessing) | low | critical | Cedar default-deny + RouteScope tenant_id binding |

## References

- ADR-0056 BNF v4.1; ADR-0065 Leptos; ADR-0105 13-layer enum; ADR-0117 packs;
  ADR-0121 Modular Product Shell (Bominal); ADR-0123 cross-product auth +
  hyperscaler maturity gate; ADR-0139 SLO promotion; ADR-0131 flat layout.
- `microservices/application/PRD.md` (this phase's authority).
- `feedback_glossary_shared_not_platform.md` (Shell→Application override).
- `feedback_quality_performance_scalability_bar.md` (TTI ≤ 2 s target).
