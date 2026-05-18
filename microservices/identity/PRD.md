---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-identity
microservice: identity
status: Accepted
sales_segment: shared-substrate
tier: internal
milestone_first_ship: M01-foundation
bominal_source: []
related_adrs: [ADR-0117, ADR-0131, ADR-0145, ADR-0148, ADR-0156, ADR-0157, ADR-0162, ADR-0173, ADR-0175, ADR-0179, ADR-0182, ADR-0183, ADR-0187, ADR-0188, ADR-0189, ADR-0190, ADR-0191]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/microservices/manifest-schema.json]
date: 2026-05-18
owner_team: axis-identity + ops-security
doc_status: published
---

# PRD-identity: OIDC + Passkey + SCIM + Step-up Authentication Substrate

## Purpose

The `identity` µservice is oyatie's identity-provider (IdP) substrate. It owns Zitadel deployment + lifecycle (per ADR-0187), the WebAuthn Level-3 relying party (per ADR-0188), the step-up-ACR enforcement contract (per ADR-0189), the SCIM 2.0 inbound provisioning endpoint (per ADR-0190), and the edge-vs-origin authz tier boundary contract (per ADR-0191). It is the single OIDC issuer + SCIM endpoint + WebAuthn RP fleet-wide.

This µservice is **shared substrate**, not a hero product. Every other oyatie µservice depends on its OIDC tokens for `principal` claims that the Cedar PDP (ADR-0183) evaluates at the Istio Ambient waypoint (ADR-0148 Tier 3). It is the precondition for ADR-0145 inter-µservice communication, ADR-0157 api-gateway tier, ADR-0162 per-tenant audit-log slicing, and ADR-0175 tenant-lifecycle workflow.

Per ADR-0131 per-microservice flat layout, this µservice ships under `microservices/identity/` with `src/` as the canonical code root and Helm/Kustomize under `iac/`.

This µservice has no Bominal equivalent; it originates in oyatie.

## Tenant Value

- **Tenant Outcome 1 — Phishing-resistant authentication by default.** Passkey (WebAuthn L3) is the steady-state credential; TOTP is fallback; SMS is forbidden per NIST SP 800-63B.
- **Tenant Outcome 2 — Enterprise SCIM 2.0 provisioning.** Okta / Microsoft Entra / Google Workspace push users/groups; lifecycle states (active/suspended/deleted) propagate automatically.
- **Tenant Outcome 3 — Per-pack data residency.** Each regulatory pack (kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa) runs its own Zitadel Instance; no cross-pack identity replication.
- **Tenant Outcome 4 — Step-up MFA on sensitive operations.** ACR levels (`routine`, `elevated`, `sensitive`, `critical`) gate operations per their intrinsic risk; key rotation, tenant deletion, billing changes require `critical` (hardware key + IT approval).
- **Internal Outcome 5 — Single OIDC issuer fleet-wide.** Every Cedar policy references `principal.iss == "https://identity-{pack}.oyatie.dev"` ; no fragmented trust roots.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to sign in with a Passkey via conditional UI | I never type or paste a password | webauthn-relying-party | Must |
| FR-02 | end-user | to add a hardware key (YubiKey) as backup | I retain access if my primary device is lost | webauthn-relying-party | Must |
| FR-03 | enterprise admin | to push user lifecycle from Okta/Entra/Workspace via SCIM 2.0 | I don't maintain two user lists | scim-server | Must |
| FR-04 | end-user | to be prompted for step-up MFA before sensitive operations | a compromised session can't escalate to critical actions | step-up-orchestrator | Must |
| FR-05 | µservice author | to receive an OIDC ID-token with `tenant_id`, `acr`, `purpose`, `data_class` claims | Cedar PDP can decide per ADR-0145 + ADR-0189 | oidc-issuer | Must |
| FR-06 | DR coordinator | to issue a per-pack Zitadel instance and have it federated nowhere | sovereign-residency rules hold | zitadel-instance-controller | Must |
| FR-07 | platform engineer | to use the `oya` CLI with the OAuth device-code flow | I authenticate without paste-buffer secrets | oidc-issuer | Must |
| FR-08 | tenant admin | to bind their existing OIDC IdP (Google Workspace, Okta) as upstream | end-users SSO without re-onboarding | external-idp-federation | Must |
| FR-09 | security auditor | to query SCIM provisioning events + step-up grant events + sign-in events | audit posture per SOC 2 / ISO 27001 / PIPA holds | audit-emitter | Must |
| FR-10 | HRIS-integration consumer | to push hires/promotions/terminations from non-SCIM HRIS (Workday/BambooHR/Rippling) | non-SCIM tenants also enjoy lifecycle propagation | hris-adapter | Should |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| OIDC token issuance | ≤25ms | ≤80ms | ≤200ms | Zitadel Postgres event-store insert + signing |
| OIDC token verification (JWKS cached) | ≤500µs | ≤2ms | ≤8ms | in-process verify; JWKS refreshed every 24h |
| WebAuthn register/finish | ≤80ms | ≤250ms | ≤600ms | attestation parse + AAGUID validation + DB insert |
| WebAuthn authenticate/finish | ≤30ms | ≤100ms | ≤250ms | signature verify + sign-count check + audit emit |
| SCIM POST Users | ≤150ms | ≤500ms | ≤1500ms | Zitadel admin API + Postgres + audit emit |
| Step-up ACR grant | ≤3s | ≤8s | ≤15s | UX-bound; redirect + Passkey ceremony |
| JWKS endpoint serve | ≤2ms | ≤8ms | ≤25ms | cached in-process; refreshed on rotate |

### Security

- **Phishing-resistant first factor** by default per ADR-0188.
- **Tokens never logged in plaintext.** Use `Classified<Token>` wrapper from `oya-data-boundary-kernel`; logger redacts.
- **JWT signing keys in OpenBao** per ADR-0117; HSM partition in regulated packs.
- **Per-tenant SCIM bearers** rotate every 90 days; cross-tenant token leak = single-tenant blast radius.
- **JWKS rotation every 24h**; signing key rotation every 90 days.
- **Constant-time bearer comparison** for SCIM auth.
- **OWASP A07 (Auth Failures) hardening**: rate-limited login (edge), account lockout after N failed auth (5 in 5min → 15min cool-off), password-policy NIST SP 800-63B compliant (when password fallback used), MFA-fatigue protection (no push without number-matching).
- **STRIDE per endpoint**: T (token replay → JWT `jti` cache), I (info disclosure → no PII in error responses), R (repudiation → audit-chain seal every grant), D (DoS → edge rate-limit per ADR-0191), E (privilege escalation → ACR floor per ADR-0189).

### Audit + Compliance

- Every `IdentityUserProvisioned`, `IdentityUserSuspended`, `IdentityUserDeleted`, `IdentitySignInSucceeded`, `IdentitySignInFailed`, `IdentityStepUpGranted`, `IdentityWebAuthnRegistered`, `IdentityScimRequestReceived`, `IdentityOidcTokenIssued` event seals into `audit-chain` per ADR-0162 within ≤1s.
- Retention per pack: KR PIPA Enforcement Decree Art. 30 (≥1y), HIPAA §164.316(b)(2) (6y), GDPR Art. 30 (purpose-bounded), PCI-DSS v4.0 §10.5.1 (≥1y, 3mo immediately available).
- SOC 2 CC6.1 (logical access), ISO 27001 A.9 (access control), PCI-DSS v4.0 §8 (identify users) compliance evidence emitted per-pack.

### Availability

- 99.99% per-pack OIDC token-issuance SLO (≤52.6 min/year unplanned downtime).
- 99.95% SCIM endpoint availability (≤4.4h/year).
- Active-active per-pack within region; no cross-pack failover per ADR-0179 sovereign-cloud-per-regional-pack.

## Bounded Contexts

| BC name | Description | Layers |
|---|---|---|
| zitadel-instance-controller | Manages Zitadel Instance lifecycle (deploy, upgrade, Postgres migration, JWKS rotation). | adapter, api, app, domain, kernel, usecase |
| oidc-issuer | OIDC issuance + introspection + JWKS serving + token revocation. Hot path. | adapter, api, app, domain, kernel, rest, sdk, usecase |
| webauthn-relying-party | WebAuthn L3 register/authenticate; AAGUID allowlist; FIDO-MDS3 refresh; conditional UI + caBLE. | adapter, api, app, domain, kernel, rest, usecase, worker |
| scim-server | SCIM 2.0 RFC 7643/7644 inbound endpoint per tenant. | adapter, api, app, domain, kernel, rest, usecase |
| hris-adapter | Pluggable adapter contract for non-SCIM HRIS (Workday/BambooHR/Rippling); poller. | adapter, api, app, domain, kernel, usecase, worker |
| step-up-orchestrator | Step-up ACR grant flow (`elevated` → `sensitive` → `critical`); JIT IT-approval bridge. | adapter, api, app, domain, kernel, usecase |
| external-idp-federation | Upstream IdP federation (Google Workspace, Okta, Entra) via OIDC + SAML. | adapter, api, app, domain, kernel, usecase |
| audit-emitter | Bridge to `audit-chain` µservice for all identity-class events. | adapter, api, app, kernel, usecase |

## Capabilities (T0/T1/T2/T3)

| Tier | Capability | EU AI Act risk class |
|---|---|---|
| T0 | oidc-token-issue (hot path; deterministic; no agent decisioning) | none |
| T0 | oidc-token-verify (hot path; deterministic) | none |
| T0 | webauthn-authenticate (hot path; cryptographic protocol) | none |
| T1 | scim-user-provision (lifecycle propagation) | minimal |
| T1 | step-up-acr-grant (factor evaluation) | limited (operational decisioning) |
| T2 | hris-integration-poll (HRIS event ingest) | limited |
| T2 | external-idp-federate (upstream IdP trust binding) | limited |

## Out of Scope

- Legacy SAML 1.1 / WS-Federation (rejected; obsolete).
- Self-built crypto (use Zitadel + webauthn-rs).
- Per-µservice IdP federation (one IdP fleet-wide).
- Cross-pack identity replication (sovereign-residency forbidden).
- Mobile-device SDK (deferred to `microservices/mobile-shell` future µservice).

## Cross-references

- ADR-0117 (OpenBao SecretReference)
- ADR-0131 (per-microservice flat layout)
- ADR-0145 (inter-microservice communication; OIDC bearer)
- ADR-0148 (service-mesh layered)
- ADR-0156 (PII registry)
- ADR-0162 (per-tenant audit-log slicing)
- ADR-0175 (tenant-lifecycle-workflow)
- ADR-0179 (sovereign-cloud-per-regional-pack)
- ADR-0182 (gateway north-south vs mesh east-west)
- ADR-0183 (Cedar app-authz vs Kyverno admission)
- ADR-0187 (canonical OIDC IdP Zitadel)
- ADR-0188 (passkey WebAuthn L3)
- ADR-0189 (step-up ACR classes)
- ADR-0190 (SCIM 2.0 inbound)
- ADR-0191 (edge vs origin authz tier)
