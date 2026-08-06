---
id: ADR-0476
title: "oya-identity: bespoke Rust human identity substrate"
status: Superseded
date: 2026-05-28
authority: founder
owner: platform-tenancy-identity
milestone: M-IDENTITY-V2
planning_impact: true
supersedes: [ADR-0421]
superseded_by: [ADR-701]
related: [ADR-0421, ADR-0394, ADR-0083, ADR-0411, ADR-0434, ADR-0416, ADR-0406, ADR-0509]
---

# ADR-0476 — oya-identity: bespoke Rust human identity substrate

## Status

Accepted — 2026-05-28 (founder-locked). Supersedes ADR-0421 (Keycloak).

## Context

ADR-0421 adopted Keycloak as a Phase-1 stepping stone for human identity. Keycloak is
OSS and proven, but it is a JVM dependency, not self-owned, and not aligned with Oyatie's
Rust-native doctrine (ADR-0002). Hyperscaler precedent is unambiguous: Google Identity
Platform, AWS IAM Identity Center, and Meta's human auth are all bespoke, internally-owned
substrates — not OSS products operated in production. The identity plane is a product
primitive, not a runtime dependency.

Founder direction 2026-05-28: build `oya-identity`, a bespoke Rust-native OIDC provider
and OAuth 2.0 authorization server. Keycloak runs as a transitional Phase-1 bridge during
the build period and is retired once oya-identity reaches feature parity.

## Decision

Build **oya-identity** — a bespoke, Rust-native human identity substrate — under
`microservices/oya-identity/`. Keycloak (ADR-0421) is the Phase-1 bridge; oya-identity
is the canonical long-term target.

The planning and tracking bridge for the ADR-0476 identity surface with the
ADR-0506, ADR-0507, and ADR-0508 provider bridges is
`oya/identity/IP-017-bespoke-identity-authn-crypto-bridge.md`; its ownership
marker is `oya/identity/OWNERS`.

### D1 — µservice scaffold

Ships as single hyperscaler-pattern crate per ADR-0509; subsystems live as mod under `src/<subsystem>/`. New µservice `microservices/oya-identity/` as a single Rust crate with Axum (ADR-0002) + Connect-RPC (ADR-0416). Key dependencies:

- `openidconnect-rs` (Apache 2.0) — OIDC provider primitives
- `oxide-auth` (MIT) — OAuth 2.0 authorization server framework
- `webauthn-rs` (Apache 2.0) — WebAuthn / passkey credential management

PostgreSQL (ADR-0406) backend for all persistent identity state. Cedar (ADR-0083)
evaluates every authorization decision using both SPIFFE (workload) and human principals.

### D2 — Protocol surface

Phase-1 (this ADR): OIDC provider + OAuth 2.0 authorization server + WebAuthn passkeys +
TOTP/HOTP MFA. PKCE enforced; implicit flow prohibited. Tenant IdP federation via OIDC
brokering (enterprises bring Okta/Auth0/AzureAD; oya-identity brokers, never stores
federated credentials or PII).

SAML 2.0 deferred to Phase-2.

### D3 — Tenant realm isolation

Per-tenant realm provisioned via Crossplane (ADR-0411) `TenantIdentityRealm` XR on
tenant onboarding. Tenant users, groups, and roles are fully isolated — no cross-tenant
trust. Cedar evaluates unified policy over SPIFFE SVIDs (workload principals) and
oya-identity JWTs (human principals) in a single policy namespace.

### D4 — Integration surface

oya-identity becomes the OIDC issuer for every µservice consumer:

- **oya-vcs** (ADR-0409): GitHub human browser auth
- **Rust-native portal** (ADR-0434): sign-in for web and desktop shell
- **oya-billing**: tenant-admin human auth
- **oya-status**: admin sign-in
- All µservice OIDC consumers via standard PKCE flows

Keycloak ADR-0421 ingress endpoints are preserved and traffic-shadowed during Phase-1 for
zero-downtime migration. Cut-over is gated on oya-identity feature parity + passing
oya-identity integration test suite.

### D5 — Multi-region + session replication

Per-region oya-identity replicas per ADR-0418 multi-region topology. Session state
replicated via Pulsar (ADR-0397) cross-region topics. SPIFFE federation (ADR-0394)
provides cross-region trust anchors. No session affinity to a single region.

## Hyperscaler lens

| Criterion | Verdict |
|---|---|
| Active upstream | Bespoke — Oyatie-owned; no upstream dependency risk |
| License | Bespoke — product asset; dependency crates Apache 2.0 / MIT |
| Self-hostable | Yes — fully self-hosted on Talos substrate |
| Hyperscaler-internal equivalent | Google Identity Platform / AWS IAM Identity Center / Meta human auth — all bespoke. oya-identity IS the hyperscaler-internal equivalent |

All four criteria pass. Bespoke ownership satisfies criterion (d) by construction.

## Alternatives considered

| Alternative | Reason not chosen |
|---|---|
| **Keycloak (ADR-0421)** | Phase-1 stepping stone only; JVM, not Rust-native, not self-owned |
| **Ory Hydra + Kratos + Keto** | Apache 2.0, unbundled Go stack; three services + custom glue; Go conflicts with Rust doctrine |
| **Zitadel** | Go-based; newer; smaller federation adoption; same Go-stack objection |

## Consequences

**Positive:**
- Full product ownership of the identity plane — no JVM, no external OSS runtime dependency
- Productizable: oya-identity can be offered as a tenant-facing identity primitive
- Rust-native; consistent with ADR-0002 Rust doctrine and performance SLOs
- Cedar unified policy plane covers both human and workload principals in one language

**Negative / accepted:**
- Large investment: ~6–12 months to full feature parity with Keycloak. Accepted per
  founder direction. Keycloak bridges the gap during the build period.
- WebAuthn + OIDC + OAuth 2.0 correctness requires careful implementation and audit.
  Mitigated by using well-tested Rust crates (`openidconnect-rs`, `oxide-auth`,
  `webauthn-rs`) and a comprehensive integration test suite.

## Integration

oya-identity exposes:
- OIDC discovery: `/.well-known/openid-configuration` per tenant realm
- OAuth 2.0 authorization + token endpoints
- WebAuthn registration + authentication endpoints
- Connect-RPC gRPC surface for internal µservice consumption (token introspection, user lookup)

Cedar principal namespace: `User::"<sub>"` from oya-identity JWT; `Workload::"<spiffe-id>"`
from SPIFFE SVID. Unified Cedar policy evaluates both.

## Promotion rationale

Bespoke identity is a hyperscaler-grade architectural requirement and a founder-locked
direction. Keycloak was the correct Phase-1 choice; oya-identity is the correct
long-term choice. Founder-locked Accepted to open M-IDENTITY-V2 milestone.

## Implementation pattern (ADR-0509 alignment)

Per ADR-0509 (Hyperscaler service decomposition pattern), `oya-identity` ships as **single-crate-per-service with mod-based subsystems**. Per-use-case crate sprawl is superseded. Use cases remain valid as domain concepts (subsystem boundaries inside `src/<subsystem>/`).

## Historical residual from ADR-187 (E3 fold 2026-08-06)

**Title:** ADR-0187-canonical-oidc-idp-zitadel-primary

**Preserved decision gist:** **Zitadel v2.55+ (Apache-2.0) is the canonical IdP, deployed via the official zitadel-charts Helm chart v9.34.1.**[^1] Zitadel runs as the `identity` µservice control plane and is the single issuer of OIDC ID-tokens, SAML assertions, SCIM 2.0 endpoint, and WebAuthn relying party for every tenant. ### Why Zitadel | Criterion | Zitadel | Keycloak | Authentik | Ory (Hydra+Kratos+Keto+Oathkeeper) | FusionAuth | |---|---|---|---|---|---| | License | Apache-2.0 | Apache-2.0 | MIT | Apache-2.0 | proprietary core, community-edition limited | | Runtime | Go single binary | Java (Quarkus) | Python (Djan

_Source file archived after fold; full body in git history / docs/adr-archive/._
