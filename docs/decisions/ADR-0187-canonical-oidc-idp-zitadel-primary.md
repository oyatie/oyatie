---
id: ADR-0187
status: Superseded
deciders: council-architecture, axis-identity, ops-security
date: 2026-05-18
owner: axis-identity
supersedes: []
superseded_by: [ADR-0476]
supersession_note: "Zitadel is superseded as the canonical/live OIDC IdP default. Current authority is ADR-0476/ADR-0482: Keycloak is the Phase-1 bridge and oya-identity is the founder-accepted bespoke Rust target after feature parity. D5 ruling. D-DISPOSITIONS-RATIFIED: SUPERSEDE-9-clean, C-13/P2."
related: [ADR-0145, ADR-0182, ADR-0183, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership]
related_specs:
  - /specs/microservices/manifest-schema.json
  - /specs/vendor-lockin-avoidance-registry.json
microservice: identity
versions_current_as_of: 2026-05-18
---

# ADR-0187 — Superseded OIDC IdP: Zitadel primary; OIDC/SAML/SCIM/Passkeys/WebAuthn first-class

## Status

Superseded by ADR-0476 (2026-05-28) and ADR-0482. This ADR is retained for historical rationale and standards coverage only. It no longer names the live canonical OIDC identity provider: current authority uses Keycloak as the Phase-1 bridge and `oya-identity` as the founder-accepted bespoke Rust target after feature parity (`docs/decisions/ADR-0476-oya-identity-bespoke-human-identity.md:29-37`, `:66-78`; `docs/decisions/ADR-0482-bespoke-substrate-roadmap.md:52-60`).

## Context

Per ADR-0145 every inter-µservice call carries an Ed25519-signed OIDC bearer with `tenant_id`, `acr`, and `data_class` claims. Per ADR-0182 north-south traffic terminates at Envoy Gateway and origin authz runs at the Istio Ambient waypoint via Cedar PDP ext_authz. Per ADR-0183 (policy-engine separation) Cedar is the application-tier authz engine. None of those ADRs names the *issuer* of the OIDC tokens those engines consume. This ADR closes that gap.

The hyperscaler bar for an IdP substrate:

- **Multi-tenant native.** Per-tenant realm isolation must be a first-class object, not a deployment pattern; tenant CRUD must be an API call, not a Helm re-release. Google Workspace IAM, Microsoft Entra ID, and Stripe tenants exhibit this shape.
- **OIDC + SAML + SCIM 2.0 + WebAuthn + magic-link + device-code in one binary.** Bolt-on protocol layers fragment trust boundaries and complicate audit (Cloudflare Access, Okta Workforce, Microsoft Entra all ship the union).
- **Open-source license without vendor lock.** Per ADR-0173-vendor-lock-in-avoidance-and-stack-ownership the substrate must be runnable in an air-gapped sovereign pack without phoning home.
- **API-first administration.** No vendor-only web UI for tenant provisioning, role assignment, or policy editing; everything must be reachable from `oya` CLI and Terraform/OpenTofu providers.
- **Operational maturity.** Postgres-backed event-store, Kubernetes-native deployment, horizontal scale, mTLS-native, audit log to external sink.

## Decision

**Historical decision, superseded:** ADR-0187 selected Zitadel v2.55+ (Apache-2.0) as the canonical IdP.[^1] **Current decision:** ADR-0476/ADR-0482 supersede that endpoint choice; Keycloak is the Phase-1 bridge and `oya-identity` is the long-term OIDC/OAuth2/WebAuthn/tenant-federation/MFA target. Do not cite this ADR as a live default or as a ≥50K-tenant trigger for Zitadel.

### Historical rationale for Zitadel (superseded)

| Criterion | Zitadel | Keycloak | Authentik | Ory (Hydra+Kratos+Keto+Oathkeeper) | FusionAuth |
|---|---|---|---|---|---|
| License | Apache-2.0 | Apache-2.0 | MIT | Apache-2.0 | proprietary core, community-edition limited |
| Runtime | Go single binary | Java (Quarkus) | Python (Django+Celery+Redis+Postgres) | 4+ Go services | Java |
| Multi-tenant native | first-class (Instances + Organizations) | realm-per-tenant (workaround) | tenant-per-deployment | tenant-per-deployment | tenant-per-deployment |
| OIDC + SAML + SCIM + WebAuthn | single binary | single binary (heavyweight) | OIDC+SAML; SCIM via Authentik Outposts | OIDC (Hydra) + SAML separate; SCIM not first-class | OIDC+SAML+WebAuthn; SCIM premium tier |
| Release cadence | weekly patch, monthly minor | quarterly major | monthly | per-component, varies | quarterly |
| Postgres event-store | yes (event-sourcing native) | RDBMS-agnostic, mutation-based | Postgres | per-service Postgres | RDBMS-agnostic |
| Kubernetes operator | community Helm chart v9.34.1 (May 2026) | official operator | Helm chart | Kratos+Hydra operator | Helm chart |
| Horizontal scale | stateless replicas + Postgres | stateless replicas + Postgres + Infinispan | stateless replicas + Redis + Postgres | stateless replicas per component | stateless replicas + Postgres |
| Air-gapped deployable | yes (no telemetry phone-home) | yes | yes | yes | community-edition yes, premium phones home |
| Hyperscaler reference customer | scaleway, doctolib | RedHat, Cloudera, government | self-hosted SaaS shops | Ory Cloud (managed), self-hosted | Aiven, Crunchbase |

At the time, ADR-0187 judged Zitadel's multi-tenant Instances-Organizations model to be the production shape Oyatie needed from day one. ADR-0476/ADR-0482 supersede that judgment for the live endpoint: Keycloak now bridges the build period, and `oya-identity` is the founder-accepted target after feature parity.

### Historical Zitadel-issued surface (superseded by `oya-identity` target)

- **OIDC ID-tokens** with `tenant_id`, `acr` (ACR per ADR-0189), `purpose`, `data_class` custom claims; JWKS rotates every 24h; tokens are short-lived (15min access, 24h refresh, 90d session).
- **SAML 2.0 assertions** for enterprise federation (Okta, Microsoft Entra, Google Workspace, OneLogin, Ping Federate).
- **SCIM 2.0 endpoint** (RFC 7643/7644) at `/scim/v2/{tenant}` for inbound provisioning from Okta / Entra / Workspace (ADR-0190).
- **WebAuthn relying party** (Level 3) with Passkey + cross-device caBLE + conditional UI (ADR-0188).
- **Device authorization grant** (RFC 8628) for `oya` CLI + service accounts on headless agents.
- **Magic-link sign-in** with TOTP fallback (RFC 6238) — SMS rejected per NIST SP 800-63B §5.1.3.

### Historical deployment shape (superseded)

- One Zitadel Instance per regulatory pack (kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa) per ADR-0240-sovereign-cloud-per-regional-pack. No cross-pack federation; each pack owns its own users + sessions + audit log.
- Per-tenant Organization within the pack's Instance. Tenant lifecycle managed via Workflow saga (ADR-0175 tenant-lifecycle-workflow).
- Postgres event-store on the pack's primary cluster (ADR-0179 postgres-connection-pooling-pgcat); read replicas per ADR-0172 CQRS.
- Secrets (JWT signing key, OIDC client secrets, SCIM bearer tokens) live in OpenBao per ADR-0117 SecretReference; referenced as `${openbao:secret/identity/<pack>/<tenant>/<purpose>}`.

### What this ADR does NOT do

- Does not specify the Cedar policy schema (ADR-0183 owns that).
- Does not specify the Envoy ext_authz gRPC contract (ADR-0145 owns that).
- Does not specify the audit-chain seal contract (Bominal ADR-0028 owns that).
- Does not specify the Step-up ACR levels (ADR-0189 owns those).
- Does not specify the SCIM provisioning adapter contract (ADR-0190 owns that).

## Historical alternatives considered (superseded)

### Keycloak

Mature, ubiquitous, RedHat-backed. Rejected primarily because realm-per-tenant doesn't fit multi-tenant SaaS economics (every tenant onboarding is a realm config push, every Keycloak upgrade touches every realm). Java runtime is heavier (≥ 1.5 GB heap per replica) than Go. Release cadence (quarterly minor) lags Zitadel's weekly cadence. SCIM endpoint requires the experimental `keycloak-scim-extension` plugin.

### Authentik

Python (Django+Celery) substrate is heavier; SCIM provisioning runs through Authentik Outposts (separate processes) rather than first-class on the Authentik core. Smaller community; not yet a hyperscaler-reference choice.

### Ory (Hydra + Kratos + Keto + Oathkeeper)

Most composable; least operationally simple. Four separate Go services + 4 separate Postgres schemas + 4 separate Helm releases. SCIM is not first-class; would need a 5th component. Selected by Ory Cloud (managed) and a handful of self-hosters; rejected here because the operational surface multiplies blast radius.

### FusionAuth

Commercial-first; community edition limits tenants. SCIM and SAML are premium-tier. Phones home in premium. Rejected per ADR-0173-vendor-lock-in.

### Self-built IdP

Superseded by ADR-0476 founder direction. Human identity is now treated as a product primitive and bespoke substrate target; Keycloak bridges the build period, and `oya-identity` replaces the historical Zitadel default after feature parity.

### Cloud-provider IdP (Auth0, Cognito, Microsoft Entra External ID)

Rejected. Each is a managed-service vendor lock-in; air-gapped sovereign packs (pack-kr-sovereign, pack-ae-sovereign, pack-ksa-sovereign) cannot run them. Pricing scales linearly with MAU which is hostile to multi-tenant SaaS economics.

## Historical consequences (superseded)

### Positive

- Single OIDC issuer fleet-wide simplifies token introspection, JWKS rotation, audit emission, and Cedar `principal.iss` policy authoring.
- Multi-tenant native model collapses tenant onboarding into a single Workflow saga call.
- Postgres event-store is reusable observability data (every state change is an event with provenance).
- Apache-2.0 license + Go single binary + air-gap support = sovereign-pack compatible.
- Open-standard wire protocols (OIDC, SAML, SCIM, WebAuthn) preserve exit optionality per ADR-0173.

### Negative

- Zitadel's documentation around Cedar integration is thin (we own the ext_authz bridge per ADR-0145).
- Zitadel's UI assumes a single browser session model; B2B enterprises with locked-down browsers (Internet Explorer compat shims, mandatory proxies) require fallback to SAML enterprise SSO with the customer's own IdP federated upstream.
- Zitadel's per-Instance Postgres schema must be migrated on every minor upgrade; this is a controlled change-management event handled by the `identity-zitadel-upgrade` runbook.

### Neutral

- Per-pack deployment ties Zitadel availability to pack availability — no cross-pack failover by design (sovereign regulation forbids cross-pack identity replication).

## Historical implementation (superseded)

The following records the 2026-05-18 Zitadel plan for provenance only; ADR-0476/ADR-0482 own current bridge and target implementation.

- Helm chart `microservices/identity/iac/helm/zitadel/` pinned to chart version 9.34.1, app version v2.55.0.[^1]
- Postgres connection via `SecretReference: ${openbao:secret/identity/{pack}/postgres-dsn}`.
- JWT signing key in OpenBao with HSM partition per ADR-0117 (regulated packs only; sandbox packs use software keys).
- TLS termination at Istio Ambient ztunnel (mTLS) per ADR-0148.
- Audit emission to `audit-chain` µservice via the AsyncAPI `IdentityEvents` channel.
- ext_authz bridge: Cedar PDP at waypoint consumes the OIDC bearer claims (`tenant_id`, `acr`, `purpose`, `data_class`).

## Historical verification (superseded)

- `oya-check-vendor-recency` gate: Zitadel chart version ≥ 9.34.1 (May 2026 baseline).[^1]
- `oya-check-license-policy` gate: Apache-2.0 confirmed; no BSL/SSPL transitive dependency.
- `oya-check-vendor-lockin-discipline` gate: only OIDC/SAML/SCIM/WebAuthn standard wire formats accepted.
- Integration test: `oya-shared-oidc-client-kernel` verifies a Zitadel-issued JWT, fetches JWKS, validates audience, checks expiry, parses `tenant_id` + `acr` claims.

## Current identity roadmap (ADR-0476 / ADR-0482 supersession)

ADR-0476 founder-locks `oya-identity`, a bespoke Rust-native OIDC provider and OAuth 2.0 authorization server, with Keycloak preserved as the transitional Phase-1 bridge during the build period. ADR-0482 places `oya-identity` in Tier 1 with Keycloak parallel-run and feature-parity cutover. The live bridge/target sequence is therefore:

| Phase | Substrate | Cutover discipline |
|---|---|---|
| Phase 1 bridge | Keycloak | Preserve bridge endpoints and traffic shadowing during build; do not expose Keycloak-specific details outside the identity adapter boundary. |
| Bespoke target | `oya-identity` | Cut over only after OIDC + OAuth 2.0 + WebAuthn + tenant IdP federation + MFA feature parity and the `oya-identity` integration suite pass. |

Zitadel-specific text in this ADR remains provenance only. It must not be used as the current ownership-ratchet default, and ADR-0394's Internal Developer Platform portal/BFF must not be confused with the OIDC identity provider.

## Cross-references

- ADR-0145 inter-microservice-communication-reform (OIDC bearer canonical)
- ADR-0182 api-gateway-north-south-vs-service-mesh-east-west-separation
- ADR-0183 policy-engine-separation-cedar-app-authz-kyverno-admission
- ADR-0173-vendor-lock-in-avoidance-and-stack-ownership
- ADR-0240-sovereign-cloud-per-regional-pack
- ADR-0188 passkey-webauthn-substrate (sibling)
- ADR-0189 step-up-authentication-acr-classes (sibling)
- ADR-0190 scim-2-provisioning-enterprise-tenants (sibling)
- ADR-0191 edge-authz-tier-vs-origin-cedar-pdp (sibling)

[^1]: Versions current as of 2026-05-18. Zitadel chart v9.34.1 released 2026-05-04, supports Zitadel ≥ v2.55. Sources: https://github.com/zitadel/zitadel-charts/releases ; https://artifacthub.io/packages/helm/zitadel/zitadel
