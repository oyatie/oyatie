---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: identity
status: Accepted
date: 2026-05-27
owner_team: axis-identity + ops-security + council-security
methodology: STRIDE + DREAD + attack trees + OWASP Auth Cheat Sheet 2024 + NIST SP 800-63B AAL ladder + RFC 8725 (JWT BCP) + RFC 9068 (access-token profile) + Cedar formal properties
related_adrs: [ADR-0002, ADR-0003, ADR-0009, ADR-0131, ADR-0145, ADR-0162, ADR-0183, ADR-0187, ADR-0188, ADR-0189, ADR-0190, ADR-0191, ADR-0243, ADR-0244, ADR-0263, ADR-0297, ADR-0313, ADR-0319, ADR-0346, ADR-0347, ADR-0348, ADR-0349]
---

# identity µservice — Threat Model

Identity is **the** highest-impact attack surface in the fleet: compromise of any one of its concerns (token signing key, passkey credential database, SCIM bearer, step-up bypass, edge authz drift) cascades to every consumer µservice. This document enumerates STRIDE threats per surface and the mitigations + detections in place.

> **Canonical consolidation (2026-05-27).** This is the single canonical identity
> threat model. The previously-scattered copies under `design/threat-model.md`,
> `security/threat-model.md`, and `threat-models/threat-model.md` were folded in
> here (no content lost) and removed, so there is exactly one threat-model
> document per the markdown-retirement / single-source-of-truth doctrine. The
> sections are:
> 1. **Service-level STRIDE** (S1–S7: OIDC, signing key, WebAuthn, SCIM, step-up,
>    edge, HRIS) + NIST AAL + OWASP — below.
> 2. **Workload-identity** (machine-to-machine PDP; RFC 8725 test cases) — see
>    [§Workload-identity threat model](#workload-identity-threat-model).
> 3. **Security asset inventory + DREAD + attack trees + telemetry + coverage
>    ledger** — see [§Asset-level threat model (DREAD + attack trees)](#asset-level-threat-model-dread--attack-trees).
> 4. **Autosharding / event-drift doctrine** — see
>    [§Autosharding event-drift threat model](#autosharding-event-drift-threat-model).

## Trust boundaries

1. **Internet ↔ Envoy Gateway edge** — untrusted bytes; only this boundary speaks plaintext HTTP (TLS-terminated here).
2. **Envoy edge ↔ Istio Ambient waypoint** — mTLS via SPIFFE per ADR-0148; both sides hold pack-pinned identities.
3. **Waypoint ↔ Zitadel pods** — mTLS within ambient mesh; pod identities `spiffe://identity.<pack>.oyatie.dev/sa/zitadel-server`.
4. **Zitadel pods ↔ Postgres** — TLS-required; password-authenticated user resolved via OpenBao SecretReference; per-pack network segment.
5. **Zitadel pods ↔ OpenBao** — mTLS; per-µservice scoped tokens; KEK signing on HSM partition per ADR-0117.
6. **External IdP ↔ Zitadel** — OIDC discovery + JWKS over public TLS; Zitadel verifies upstream IdP JWT.
7. **HRIS source ↔ HRIS adapter** — vendor-specific (Workday TLS, BambooHR REST OAuth2, Rippling REST OAuth2); credentials in OpenBao.

## Surfaces

### S1 — OIDC token issuance (POST /oauth/v2/token)

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | Spoofing client_id | Per-tenant client_secret in OpenBao; PKCE mandatory for public clients (RFC 7636) | `IdentityOidcTokenIssued` event tags `client_id`; anomaly alert on novel client_id |
| T | Tampering with token in transit | TLS 1.3 + HSTS preload; mTLS at mesh | TLS-handshake-failure metric |
| R | Repudiation of token issuance | Every issuance emits `IdentityOidcTokenIssued` to audit-chain (Merkle + Ed25519 seal) | Audit-chain completeness SLO 1.0 |
| I | Disclosure of token in logs | `Classified<Token>` redacts on Display; logger sieve refuses `eyJ` prefixes | `oya-check-raw-secret-emission` lane |
| D | DoS via token-issuance flood | Edge rate-limit per IP + per client_id; OpenBao token-cache backed | 429 rate per minute alert |
| E | Privilege escalation via scope creep | Cedar policy on token-introspection refuses `actor.client_id` mismatch | Cedar deny audit |

### S2 — JWT signing key (in OpenBao + HSM)

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | Key impersonation via stolen kid | JWKS rotates every 24h; kid is bound to active key; old kids removed after grace | JWKS-staleness SLO |
| T | Key tampering | HSM-backed signing in regulated packs (KEK never in software memory) | HSM attestation report (KekAttested event) |
| R | Repudiation of signing | Every `Sign(jwt)` call traced; per-jti audit-chain seal | Audit-chain replay query |
| I | Key disclosure via OpenBao | OpenBao policy scopes per-µservice; mTLS; envelope encryption KEK is HSM-pinned | OpenBao audit emission |
| D | DoS on OpenBao | Per-pack HA cluster + soft-disable rotation if cluster degraded | OpenBao up SLO 0.9999 |
| E | Escalation via rotation race | Rotation worker uses two-phase commit + version-pinned cache invalidation | rotation-cascade-recovery runbook |

### S3 — WebAuthn ceremonies (register / authenticate)

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | Phishing relying party (RP-ID mismatch) | RP-ID locked to `<pack>.oyatie.dev`; origin pinned | WebAuthn origin-mismatch alert |
| T | Tampering with attestation object | `webauthn-rs` validates AAGUID against FIDO-MDS3 allowlist (regulated packs) | AAGUID-not-allowlisted finding |
| R | Repudiation of credential registration | `IdentityWebAuthnRegistered` event sealed; per-aaguid trail | Audit-chain query |
| I | Disclosure of credential metadata across tenants | Postgres RLS + tenant_id partition; per-tenant Cedar gate | Cross-tenant Cedar deny audit |
| D | Replay attack via captured assertion | Server-generated challenge with TTL ≤300s; sign-count monotonic enforcement | SignCountRegression error → operator paged |
| E | Cloned authenticator | Sign-count regression triggers credential revocation + reset workflow | Auto-revoke + alert |

### S4 — SCIM 2.0 endpoint (/scim/v2/{tenant}/Users|Groups)

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | SCIM bearer guessing | 256-bit random bearer; constant-time compare; rate-limit per IP | brute-force detection in runbook |
| T | Tampering payload to deactivate other tenant's user | Tenant-scoped URI + bearer; payload tenant cross-check | cross-tenant SCIM Cedar deny |
| R | Repudiation of provisioning | `IdentityScimRequestReceived` + `IdentityUserProvisioned` audit events | Audit-chain query by SCIM bearer ID |
| I | Disclosure of user list to non-admin | SCIM bearer scoped to tenant; OIDC bearer alternative requires `purpose=admin` | Cedar deny audit |
| D | Bulk-list flood | Cursor pagination + max items-per-page; rate-limit per bearer | SCIM rate alert |
| E | Privilege escalation by patching `roles` | `roles` is read-only via SCIM; managed via Cedar policy admin only | SCIM mutation audit |

### S5 — Step-up ACR flow

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | Spoofing ACR claim | ACR signed in JWT; verifier rejects manual claim insertion | JWT signature check |
| T | Tampering with `acr_event_at` to extend session | Claim signed; PDP refuses if `now - acr_event_at > max_age_seconds` | step-up-grant-latency SLO + Cedar deny |
| R | Repudiation of step-up | `IdentityStepUpGranted` + `IdentityStepUpDenied` events sealed | Audit-chain query |
| I | Leak of IT-approval token | `acr=critical` requires JIT IT-approval bound to resource + 5-min window; one-time use | Approval-token audit trail |
| D | Step-up loop (DoS for legitimate user) | Backoff after 3 failed step-ups (15min cool-off) | Brute-force runbook |
| E | Bypass via lower-level acr replay | PDP refuses replay if `acr` JWT `jti` already-seen in cache | jti-replay-cache alert |

### S6 — Edge authz (Coraza WAF + rate limits + geo)

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | IP geo spoofing | MaxMind GeoIP + ASN; both required for sensitive geo decisions | geo-spoof detection alert |
| T | WAF rule bypass via encoding tricks | OWASP CRS v4.25.0 LTS with canonicalisation + double-encoding decoders | WAF-deny rate per rule |
| R | Repudiation of edge deny | `EdgeDeny` event (truncated PII; tier=edge) sealed | Audit-chain query |
| I | Exposure of rate-limit counters | Counters in Valkey-per-pack; not externally readable | n/a |
| D | High-volume DDoS | eBPF XDP at NIC + Cilium L3/L4 filter + per-IP rate caps | DDoS-attack runbook |
| E | Bypassing edge via direct internal path | Network policy: pod ingress only via Envoy Gateway selector | NetworkPolicy deny audit |

### S7 — HRIS adapter (Workday/BambooHR/Rippling)

| STRIDE | Threat | Mitigation | Detection |
|---|---|---|---|
| S | Spoofed HRIS webhook | Pull-only (we poll); no inbound webhook trust | n/a |
| T | Tampered HRIS response | Per-vendor schema validation; reject on shape mismatch | HRIS-shape-error alert |
| R | Repudiation of HRIS-driven lifecycle change | `IdentityHrisHirePulled` + `IdentityHrisTerminationPulled` events | Audit-chain query |
| I | HRIS credential disclosure | Per-tenant OAuth2 in OpenBao; constant-time compare on rotation | OpenBao audit emit |
| D | HRIS endpoint outage | Exponential backoff + DLQ + last-known-good cache | HRIS-poll-success SLO |
| E | HRIS data drift (terminated user still active) | Daily reconciliation job compares Zitadel active-set vs HRIS active-set; alarms on drift | HRIS-reconciliation-drift alert |

## NIST SP 800-63B AAL mapping

| ACR (ADR-0189) | NIST AAL | Factor type | Notes |
|---|---|---|---|
| routine | AAL1 | single-factor crypto / Passkey or password+TOTP | basic re-authentication every 24h |
| elevated | AAL2 | multi-factor crypto (Passkey synced) | re-auth every 4h |
| sensitive | AAL2 with re-auth | multi-factor crypto + recent presentation | re-auth every 1h |
| critical | AAL3 | multi-factor crypto hardware + IT approval | re-auth every 15min |

NIST AAL3 mandates hardware authenticator with verifier impersonation resistance (FIDO2 satisfies). `acr=critical` enforces AAL3 + JIT IT-approval.

## OWASP Top 10 2021 mapping

| OWASP | Concern | Identity mitigation |
|---|---|---|
| A01 Broken Access Control | Cedar PDP per ADR-0183 + step-up per ADR-0189 + tier discipline per ADR-0191 |
| A02 Cryptographic Failures | JWT signing keys in HSM (regulated); JWKS rotation every 24h; TLS 1.3 only |
| A03 Injection | SCIM filter parser is hand-rolled to avoid SQLi-style injection through `eq "..."` strings |
| A04 Insecure Design | Threat model authored before code; STRIDE per surface |
| A05 Security Misconfiguration | Helm + Kustomize per-pack; `lean-a18-identity-vendor-isolation` lane refuses leaks |
| A06 Vulnerable & Outdated Components | `oya-check-vendor-recency` lane on Zitadel chart + webauthn-rs + openidconnect crates |
| A07 ID & Auth Failures | This µservice IS the auth-failure remediation: passkey-first, ACR gates, audit emit, rate-limit |
| A08 Software & Data Integrity Failures | Audit-chain seal; sign-count monotonic; JWKS rotated; HSM-backed |
| A09 Security Logging & Monitoring Failures | Audit-emit-completeness SLO 1.0; 18 distinct event types |
| A10 SSRF | Zitadel discovery URLs allowlisted to known IdP domains; HRIS endpoint registry in code |

---

# Workload-identity threat model

*Bounded context: `workload-identity` (machine-to-machine). Methodology: STRIDE +
RFC 8725 (JWT BCP) + RFC 9068 (access-token profile) + Cedar formal properties.
Related ADRs: ADR-0002, ADR-0131, ADR-0183. Research brief:
`design/hyperscaler-best-practice-brief.md`.*

Workload identity is a top-tier attack surface: a forged token or a confused
authorization decision compromises every downstream µservice that trusts this
PDP. This section enumerates the threats the cited brief (§5) calls out — encoded
as concrete verifier test cases — plus the mitigations and detections. It is
implemented by the `oya-identity-workload-{domain,oidc-adapter,authz-cedar-adapter,app,api,rest}`
crate family.

## Workload trust boundaries

1. **PEP ↔ PDP** — Envoy waypoint / sidecar / in-process gate calls `/authorize`
   or `WorkloadAuthorizer.authorize`. mTLS via SPIFFE (ADR-0148). The PEP's own
   workload token authenticates the call.
2. **PDP ↔ JWKS source** — JWKS fetched over TLS from the per-trust-domain issuer;
   `jku`/`x5u` allowlist-gated (SSRF boundary).
3. **PDP ↔ policy store** — tenant-scoped Cedar partitions; unreachable store →
   embedded-Cedar default-deny.
4. **PDP ↔ revocation denylist** — suspended/retired principal ids; consulted at
   validate time.
5. **PDP ↔ audit chain** — immutable decision log emission (append-only).

## Workload STRIDE by surface

### W1 — Token validation (`/tokens/validate`)

| STRIDE | Threat | Mitigation (RFC ref) | Detection / test |
|---|---|---|---|
| S | **Algorithm confusion** `RS256→HS256` — attacker signs with the RSA public key as an HMAC secret | Server-side `kid`→alg binding; header `alg` never trusted (RFC 8725 §3.1) | Test AC-W-02; metric `oya_identity_workload_validate_request_total{failure="algorithm-mismatch"}` |
| S | **`alg:none`** — unsigned token accepted | Reject `none` unconditionally (RFC 8725 §3.2) | Test AC-W-01 |
| S | **Key substitution / forgery** — token signed by a key not belonging to the issuer | Key-belongs-to-issuer check (RFC 8725 §3.8) | Test (oidc_validation: key-not-belongs-to-issuer) |
| T | **JWKS poisoning / SSRF** via attacker-controlled `jku`/`x5u` | `jku`/`x5u` allowlist-gated; `kid` sanitized; default static trust-domain→JWKS map (RFC 8725 §3.10) | Test AC-W-05 |
| R | Repudiation of a validation outcome | Every outcome emitted to `workload.token-validation.v1` with stable never-reused id (brief §9) | Test AC-W-13 |
| I | **Replay** of a captured token | Short TTL + `aud` binding + `jti` for forensics (brief §5, §7) | `jti` recorded; replay window bounded by `exp` |
| I | Token body leaked in logs | Token bodies never logged; only `sub` + `jti` (brief §9) | Log-sieve gate; Test AC-W-17 |
| D | DoS via expensive verification flood | Per-tenant validate quota; key-set capped ≤100 (brief §10) | 429 rate; key-set-size metric |
| E | **Cross-JWT confusion** — an ID token or refresh token replayed as an access token | Explicit `typ` required (RFC 8725 §3.11–12) | Test (oidc_validation: missing `typ`) |

### W2 — Authorization (`/authorize`, `/authorize-with-token`, `/authorize:batch`)

| STRIDE | Threat | Mitigation | Detection / test |
|---|---|---|---|
| S | Caller spoofs a principal it is not | `aud`/audience binding (W1) feeds the principal projection; cross-trust-domain forbid | Test AC-W-08 |
| T | Tampering with the PARC tuple in transit | mTLS PEP↔PDP | mesh handshake metrics |
| R | Repudiation of a decision | One immutable record per authorize, implicit-vs-explicit-deny preserved (brief §9) | Test AC-W-13; `design/audit-evidence-emission.md` |
| I | **Confused deputy** — caller induces the PDP to authorize against the wrong audience/resource | `aud` bound per principal/trust-domain (`forbid-audience-mismatch`); resource trust-domain checked (brief §5) | Test (cedar_authz: audience-mismatch) |
| I | Existence leak via status code | 403, never 404 (brief §2) | Test AC-W-10 |
| D | Authorize flood | Per-tenant quota; `/authorize:batch`; embedded Cedar | 429 rate |
| E | **Privilege escalation** — principal performs an action beyond least-privilege | Default-deny + capability/scope-scoped permits + forbid-overrides-permit (brief §5, §10) | Tests AC-W-06, AC-W-09 |

### W3 — Lifecycle (`:suspend` / `:retire`)

| STRIDE | Threat | Mitigation | Detection / test |
|---|---|---|---|
| S | Unauthorized suspend/retire | Caller authz → 403 never 404 | contract test |
| T | Status tampering via PATCH | Explicit transition sub-resources only (no status PATCH) (brief §2) | contract review |
| R | Repudiation of a transition | `workload.decision.v1` / lifecycle event sealed | Test AC-W-13 |
| E | **Lifecycle abuse / id reuse** — reusing a retired id to inherit grants | Principal-id immutability + non-reuse; retired tombstoned (brief §5) | Test AC-W-14 |

## Workload attack scenarios → defenses (brief §5 summary)

| Attack | Defense |
|---|---|
| `alg:none`, RS256→HS256 | server-side allowlist, reject none (encoded as mandatory verifier tests) |
| Forgery / key substitution | key-belongs-to-issuer (§3.8) |
| Replay | short TTL + `aud` + `jti` |
| Confused deputy | `aud` bound per principal/trust-domain |
| JWKS poisoning / SSRF | `jku`/`x5u` allowlist; `kid` sanitize (§3.10) |
| Cross-JWT confusion | explicit `typ` (§3.11–12) |
| Privilege escalation | least-privilege + default-deny + forbid-overrides-permit |
| Lifecycle abuse | unique never-reused ids; retired tombstoned |

**NHI 2025 context.** Machine identities outnumber humans ~82:1 (brief §5). The
mitigation posture is short-lived, federated, secretless tokens validated by this
substrate — never long-lived shared secrets. References: RFC 8725 §3.1–3.2, §3.8,
§3.10–12; RFC 9068; Cedar formal properties (arXiv 2403.04651); the cited brief
§5 + load-bearing flags. See the per-test mapping in `IP-001-identity-design.md`
Slice 2/3.

---


# Asset-level threat model (DREAD + attack trees)

*Consolidated from the former `security/threat-model.md`. Security-subdirectory-level analysis: assets, boundaries, ranked threats, mitigations, telemetry, and incident routing. Narrower than the service-root STRIDE above; assumes identity is a Tier-1 substrate.*

This section covers the identity substrate at the asset/boundary level.
It is intentionally narrower than the service root STRIDE and focuses on
assets, boundaries, ranked threats, mitigations, telemetry, and incident routing.
The analysis assumes identity is a Tier-1 substrate: compromise propagates to
every tenant-facing service that accepts OIDC, WebAuthn, SCIM, or step-up claims.

## Asset Inventory

### Named Data Classes

| Asset ID | Named data class | Description | Primary store | Security objective |
|---|---|---|---|---|
| ID-A01 | PrincipalIdentityRecord | Stable user, service, tenant-admin, and workload principal identifiers. | Zitadel/Postgres metadata | Prevent spoofing and cross-tenant alias collision. |
| ID-A02 | WebAuthnCredentialPublicKey | Public credential key, AAGUID, sign counter, RP ID binding, and attestation metadata. | Identity credential store | Prevent replay and authenticator clone acceptance. |
| ID-A03 | RecoveryKeyEnvelope | Tenant/user recovery envelope, recovery ceremony state, key issuance metadata. | OpenBao-backed envelope store | Prevent recovery-key compromise and unauthorized reset. |
| ID-A04 | SessionBindingRecord | Session ID, token family, device binding, ACR level, refresh-token lineage. | Identity session store | Prevent session fixation and token family takeover. |
| ID-A05 | OidcTokenSigningMaterial | JWT signing key handle, JWKS KID, rotation epoch, public resolver state. | HSM/OpenBao/JWKS cache | Prevent token forgery and stale-key confusion. |
| ID-A06 | OAuthAuthorizationCode | Authorization code, PKCE verifier binding, redirect URI, nonce, state hash. | Short-lived transaction cache | Prevent OAuth flow tampering and code substitution. |
| ID-A07 | ExternalIdpTrustConfig | Okta/Auth0/Google issuer metadata, JWKS URI, client credentials, audience rules. | OpenBao and tenant config | Prevent identity-provider compromise blast radius. |
| ID-A08 | ScimProvisioningPayload | SCIM user/group mutations from HRIS or IdP sources. | SCIM adapter queue and audit stream | Prevent mass privilege drift or tenant crossing. |
| ID-A09 | StepUpAcrGrant | Recent authentication context, ACR grant, resource binding, expiry. | Step-up orchestrator state | Prevent sensitive action bypass. |
| ID-A10 | CedarAuthzDecision | Tenant-scope, context-split, ACR predicate, and recovery operator policy outcome. | Policy decision log and audit-chain | Prevent authorization confusion after authentication. |
| ID-A11 | AbuseSignalProfile | Credential stuffing, compromised credential, bot, IP reputation, and attestation signals. | Observability plus audit-chain | Detect account takeover attempts. |
| ID-A12 | AuditEmissionEnvelope | ADR-0263 envelope with tenant_id, trace_id, span_id, audit_id, schema_version, source_microservice. | audit-chain | Preserve non-repudiation and incident correlation. |

### Named External Interfaces

| Interface ID | Interface | Entry point | Principal | Notes |
|---|---|---|---|---|
| ID-I01 | OIDC Authorization Endpoint | `./contracts/openapi/identity.yaml` | Browser user or OAuth client | Handles state, nonce, PKCE, redirect URI, and consent. |
| ID-I02 | OIDC Token Endpoint | `./contracts/openapi/identity.yaml` | OAuth client | Issues tokens and refreshes token families. |
| ID-I03 | JWKS Endpoint | `./contracts/openapi/identity.yaml` | All verifier services | Serves public signing keys and rotation metadata. |
| ID-I04 | WebAuthn Registration | `./reference-implementations/webauthn-passkey-flow-rust-sdk.md` | Authenticated user | Registers passkey credential and attestation. |
| ID-I05 | WebAuthn Authentication | `./reference-implementations/webauthn-passkey-flow-rust-sdk.md` | Browser/user agent | Verifies challenge, origin, RP ID, and sign counter. |
| ID-I06 | SCIM 2.0 API | `./contracts/openapi/identity.yaml` | HRIS or external IdP | Provisions users and groups. |
| ID-I07 | External IdP Federation | `./IP-011-external-idp-federation.md` | Okta/Auth0/Google | Resolves upstream identity into Oyatie principal. |
| ID-I08 | Step-up Orchestrator | `./IP-010-step-up-orchestrator.md` | Calling service | Grants recent ACR for sensitive action. |
| ID-I09 | Multi-context Principal Resolver | `./IP-017-multi-context-principal-resolver.md` | Workload services | Resolves personal/work/conglomerate context. |
| ID-I10 | Audit Event Bridge | `./contracts/asyncapi/identity-events.yaml` | Identity service | Emits sealed identity events. |

### Named Dependencies

| Dependency ID | Dependency | Use | Failure impact | Guardrail |
|---|---|---|---|---|
| ID-D01 | Zitadel | OIDC core, user store, token issuance | Authentication outage or misissued token | `./slos/zitadel-instance-health.openslo.yaml`. |
| ID-D02 | WebAuthn server library | Challenge generation and verification | Replay, origin bypass, or attestation confusion | `./IP-004-webauthn-relying-party-kernel.md`. |
| ID-D03 | OpenBao | Secrets, recovery envelope, IdP client secret, signing handles | Secret disclosure or recovery takeover | `./policy/operator-recovery.cedar`. |
| ID-D04 | HSM/KMS | Signing key isolation | Forged JWT or unprovable rotation | `./runbooks/jwks-rotation.md`. |
| ID-D05 | Cedar policy-engine | Tenant, ACR, context, operator authorization | Broken access control | `./policy/tenant-scope.cedar`. |
| ID-D06 | External IdPs | Federation source of truth | Provider compromise or outage | `./runbooks/idp-failover-drill.md`. |
| ID-D07 | HRIS systems | User lifecycle source | Unauthorized account creation or stale access | `./IP-009-hris-adapter.md`. |
| ID-D08 | audit-chain | Sealed evidence | Repudiation and weak forensics | `./IP-012-audit-emitter.md`. |
| ID-D09 | Observability substrate | Detection and response | Missed attack signals | ADR-0263. |
| ID-D10 | Abuse-defence baseline | Bot/stuffing/rate signals | Credential stuffing amplification | ADR-0297. |

## Trust Boundaries

| Boundary ID | Named boundary | Crosses from | Crosses to | Primary concern |
|---|---|---|---|---|
| ID-B01 | Public-internet boundary | Browser, mobile app, OAuth client | Envoy/Istio identity ingress | Hostile input, bot traffic, redirect abuse. |
| ID-B02 | OAuth redirect boundary | OAuth client redirect URI | Authorization transaction cache | State, nonce, PKCE, and redirect URI tamper. |
| ID-B03 | WebAuthn origin boundary | Browser authenticator ceremony | WebAuthn relying party kernel | RP ID, origin, challenge freshness, sign counter. |
| ID-B04 | Session-cookie boundary | User agent cookie jar | Session binding store | Session fixation, cookie theft, refresh family theft. |
| ID-B05 | Tenant boundary | Principal from tenant A | Tenant-scoped identity records for tenant B | Cross-tenant account or group lookup. |
| ID-B06 | Cell boundary | Identity pod in one cell | Home-cell identity state | Stale session, replication lag, inconsistent revocation. |
| ID-B07 | External-IdP boundary | Okta/Auth0/Google | Federation adapter | IdP compromise, issuer spoof, JWKS poisoning. |
| ID-B08 | HRIS/SCIM boundary | HRIS or IdP provisioning client | SCIM server kernel | Mass mutation, role injection, stale bearer. |
| ID-B09 | Secret boundary | Identity workload | OpenBao/HSM | Signing key and recovery material custody. |
| ID-B10 | Cedar boundary | Authenticated identity | Authorization decision layer | Authn/authz confusion, ACR bypass. |
| ID-B11 | Audit boundary | Identity state change | audit-chain emission bridge | Missing audit_id, non-repudiation gap. |
| ID-B12 | Information-barrier boundary | Front/middle/back-office principal | Context resolver and policy fragments | Scope taint and inappropriate access. |

## STRIDE Analysis

### Spoofing

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| ID-S01 | WebAuthnCredentialPublicKey | ID-B03 | Passkey/WebAuthn replay using a captured assertion inside the challenge TTL. | Account takeover if challenge or sign counter validation fails. |
| ID-S02 | RecoveryKeyEnvelope | ID-B09 | Recovery-key compromise lets attacker impersonate account owner during reset. | Persistent account takeover and MFA reset. |
| ID-S03 | ExternalIdpTrustConfig | ID-B07 | Okta/Auth0/Google tenant IdP is compromised and emits valid-looking upstream claims. | Federated user spoofing across tenant admin flows. |
| ID-S04 | SessionBindingRecord | ID-B04 | Session fixation forces victim onto attacker-known session family. | Victim actions bind to attacker-controlled refresh lineage. |
| ID-S05 | OAuthAuthorizationCode | ID-B02 | OAuth authorization-code substitution after redirect URI manipulation. | Token issuance to wrong client or user. |
| ID-S06 | OidcTokenSigningMaterial | ID-B09 | Stolen or stale KID is used to spoof issuer trust. | Downstream services accept forged tokens. |
| ID-S07 | ScimProvisioningPayload | ID-B08 | Spoofed SCIM bearer creates or reactivates user. | Unauthorized workforce account creation. |
| ID-S08 | PrincipalIdentityRecord | ID-B05 | Tenant ID confusion lets a principal claim membership in another tenant. | Cross-tenant data access. |

### Tampering

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| ID-T01 | OAuthAuthorizationCode | ID-B02 | Attacker tampers with `state`, `nonce`, or PKCE code verifier binding. | Login CSRF or code injection. |
| ID-T02 | SessionBindingRecord | ID-B04 | Refresh token family is modified to bypass rotation and reuse detection. | Long-lived session takeover. |
| ID-T03 | ExternalIdpTrustConfig | ID-B07 | JWKS URI or issuer metadata is changed to attacker-controlled endpoint. | Provider trust poisoning. |
| ID-T04 | RecoveryKeyEnvelope | ID-B09 | Recovery envelope metadata is edited to suppress dual-control requirements. | Unauthorized recovery. |
| ID-T05 | ScimProvisioningPayload | ID-B08 | SCIM PATCH modifies protected role or tenant attributes. | Privilege escalation. |
| ID-T06 | StepUpAcrGrant | ID-B10 | ACR grant timestamp or resource binding is changed. | Sensitive action without fresh auth. |
| ID-T07 | AuditEmissionEnvelope | ID-B11 | Identity event is emitted without audit_id or with wrong tenant_id. | Broken forensic trail. |
| ID-T08 | AbuseSignalProfile | ID-B01 | Bot score or IP reputation headers are forged upstream. | Credential stuffing controls weakened. |

### Repudiation

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| ID-R01 | OidcTokenSigningMaterial | ID-B11 | Token issuance cannot be tied to an audit-chain event. | User denies issuing or using a session. |
| ID-R02 | RecoveryKeyEnvelope | ID-B11 | Recovery ceremony lacks sealed operator and requester evidence. | Reset cannot be defended to tenant or auditor. |
| ID-R03 | ExternalIdpTrustConfig | ID-B07 | Upstream IdP admin claims federation change was not applied. | Ambiguous root cause after compromise. |
| ID-R04 | ScimProvisioningPayload | ID-B08 | HRIS disputes who sent provisioning mutation. | Employment lifecycle audit gap. |
| ID-R05 | WebAuthnCredentialPublicKey | ID-B03 | User disputes passkey registration or revocation. | Credential custody uncertainty. |
| ID-R06 | CedarAuthzDecision | ID-B10 | Authorization deny is not recorded with policy id. | Incident cannot prove why access was denied. |

### Information Disclosure

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| ID-I01 | PrincipalIdentityRecord | ID-B05 | Cross-tenant user enumeration through login, SCIM, or resolver errors. | Privacy breach and targeting list. |
| ID-I02 | RecoveryKeyEnvelope | ID-B09 | Recovery secret or key handle appears in logs. | Account recovery takeover. |
| ID-I03 | OidcTokenSigningMaterial | ID-B09 | Signing key, IdP client secret, or refresh token is logged or exported. | Token forgery or lateral movement. |
| ID-I04 | SessionBindingRecord | ID-B04 | Session cookie lacks secure/httpOnly/sameSite attributes. | Token theft through browser or network path. |
| ID-I05 | ExternalIdpTrustConfig | ID-B07 | Federation metadata exposes tenant directory details. | Tenant intelligence leakage. |
| ID-I06 | AuditEmissionEnvelope | ID-B11 | ADR-0263 emissions include raw PII before scrubbing. | Observability substrate privacy breach. |

### Denial of Service

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| ID-DOS01 | OIDC endpoints | ID-B01 | Authorization or token endpoint flood exhausts signer or DB capacity. | Login outage across fleet. |
| ID-DOS02 | WebAuthnCredentialPublicKey | ID-B03 | Challenge spray creates transaction-cache pressure. | Passkey login degraded. |
| ID-DOS03 | ExternalIdpTrustConfig | ID-B07 | Upstream IdP outage blocks federated tenants. | Workforce login outage. |
| ID-DOS04 | ScimProvisioningPayload | ID-B08 | Bulk SCIM sync storm saturates mutation queues. | Provisioning lag or stale access. |
| ID-DOS05 | OidcTokenSigningMaterial | ID-B09 | HSM/OpenBao latency prevents signing and key resolution. | Token issuance failure. |
| ID-DOS06 | AbuseSignalProfile | ID-B01 | Credential stuffing generates high-cardinality telemetry. | Detection cost spike and login throttling. |

### Elevation of Privilege

| Threat ID | Asset | Boundary | Threat | Impact |
|---|---|---|---|---|
| ID-E01 | StepUpAcrGrant | ID-B10 | Lower ACR claim replayed for high-risk operation. | Sensitive action without required proof. |
| ID-E02 | ScimProvisioningPayload | ID-B08 | SCIM group patch grants tenant admin or operator role. | Tenant-level privilege escalation. |
| ID-E03 | SessionBindingRecord | ID-B04 | Refresh token reuse fails open after race. | Attacker keeps session after victim rotates. |
| ID-E04 | ExternalIdpTrustConfig | ID-B07 | Compromised IdP maps attacker to privileged Oyatie role. | Federation privilege escalation. |
| ID-E05 | RecoveryKeyEnvelope | ID-B09 | Operator recovery override bypasses dual-control Cedar policy. | Account owner displaced. |
| ID-E06 | PrincipalIdentityRecord | ID-B12 | Front/middle/back-office taint not attached to principal. | Information-barrier bypass. |

## DREAD Scoring

Scale: 1 is low, 10 is high. Rank is sorted by total score.

| Rank | Threat ID | Threat | Damage | Reproducibility | Exploitability | Affected users | Discoverability | Total |
|---|---|---|---:|---:|---:|---:|---:|---:|
| 1 | ID-S03 | External IdP compromise accepted as trusted federation. | 10 | 8 | 7 | 10 | 8 | 43 |
| 2 | ID-S06 | JWT signing material or KID trust spoofing. | 10 | 7 | 7 | 10 | 7 | 41 |
| 3 | ID-S02 | Recovery-key compromise. | 9 | 8 | 8 | 8 | 7 | 40 |
| 4 | ID-E04 | Compromised IdP maps attacker to privileged role. | 10 | 7 | 7 | 9 | 7 | 40 |
| 5 | ID-S01 | Passkey/WebAuthn replay or cloned authenticator acceptance. | 9 | 7 | 7 | 8 | 8 | 39 |
| 6 | ID-T01 | OAuth state, nonce, or PKCE tampering. | 8 | 8 | 8 | 8 | 7 | 39 |
| 7 | ID-S04 | Session fixation against refresh-token family. | 8 | 8 | 8 | 7 | 8 | 39 |
| 8 | ID-E05 | Operator recovery override bypass. | 9 | 6 | 6 | 8 | 7 | 36 |
| 9 | ID-E02 | SCIM role injection. | 9 | 7 | 6 | 8 | 6 | 36 |
| 10 | ID-DOS01 | Token endpoint flood. | 8 | 9 | 8 | 8 | 3 | 36 |
| 11 | ID-I02 | Recovery secret leaked in logs. | 9 | 6 | 6 | 8 | 6 | 35 |
| 12 | ID-T03 | IdP JWKS metadata poisoning. | 9 | 6 | 6 | 8 | 5 | 34 |
| 13 | ID-I01 | Cross-tenant principal enumeration. | 6 | 9 | 8 | 7 | 4 | 34 |
| 14 | ID-DOS05 | HSM/OpenBao signing outage. | 8 | 7 | 5 | 9 | 4 | 33 |
| 15 | ID-E06 | Information-barrier taint missing. | 8 | 6 | 5 | 7 | 6 | 32 |

## Attack Trees

### Opportunistic Adversary: Credential Stuffing to Account Takeover

- Goal: obtain a valid tenant session.
  - Path O1: collect leaked username/password pairs.
  - Path O2: submit attempts to ID-I01 and ID-I02 through public-internet boundary ID-B01.
  - Path O3: evade bot scoring by rotating IP and user-agent.
  - Path O4: trigger weak recovery if credential attempt fails.
  - Path O5: replay successful refresh token against session boundary ID-B04.
- Required break: ADR-0297 abuse controls fail to issue `AbuseDefenceCredentialStuffing`.
- Required break: session family reuse detection fails closed incorrectly.
- Required break: step-up policy is not required before admin or payment action.
- Detection pivot: `IdentitySignInFailed`, `AbuseDefenceRateLimitHit`, and `AbuseDefenceCredentialPwned`.

### Targeted Adversary: OAuth Flow Tampering

- Goal: get authorization code or token for victim account.
  - Path T1: register or compromise OAuth client redirect URI.
  - Path T2: induce victim to follow crafted authorization URL.
  - Path T3: alter `state`, `nonce`, or PKCE verifier binding at ID-B02.
  - Path T4: exchange authorization code at token endpoint.
  - Path T5: use token at another service before revocation propagates.
- Required break: redirect URI exact match is missing or normalized unsafely.
- Required break: PKCE verifier is not bound to the code transaction.
- Required break: authorization code replay window is too long.
- Detection pivot: `IdentityOidcTokenIssued`, Cedar deny records, and trace_id chain.

### Insider Adversary: Recovery Override Abuse

- Goal: reset a target account without user consent.
  - Path I1: obtain operator console access.
  - Path I2: request recovery envelope reissue.
  - Path I3: suppress or alter dual-control approval.
  - Path I4: bind new passkey to target principal.
  - Path I5: revoke old credential to lock out target.
- Required break: `./policy/operator-recovery.cedar` allows single-operator reset.
- Required break: recovery event lacks audit_id or sealed approver identity.
- Required break: delayed user notification is not sent.
- Detection pivot: `IdentityWebAuthnRegistered`, `IdentityWebAuthnRevoked`, `OfficeBoundaryAttemptEvaluated`.

### Nation-State Adversary: Federation Provider Takeover

- Goal: compromise a tenant by controlling upstream IdP trust.
  - Path N1: compromise Okta/Auth0/Google admin or signing key.
  - Path N2: publish malicious JWKS or add attacker user to privileged group.
  - Path N3: rely on federation adapter to accept upstream claims.
  - Path N4: mint Oyatie session and step-up by policy inheritance.
  - Path N5: pivot to mail, drive, payments, or audit-chain as tenant admin.
- Required break: issuer and audience pinning fail.
- Required break: federation role mapping lacks Cedar policy review.
- Required break: upstream key rotation anomaly is not detected.
- Detection pivot: `ExternalIdpBound`, `IdentitySignInSucceeded`, `AbuseDefenceSpoofDetected`, and `ConglomerateInformationBarrierCrossingRefused`.

## Mitigations Currently In Place

| Threat ID | Named mitigation | ADR or policy | Named code path or doc |
|---|---|---|---|
| ID-S01 | Challenge TTL, RP ID origin pinning, sign-counter monotonic checks. | ADR-0243, ADR-0263 | `./IP-004-webauthn-relying-party-kernel.md`; `./runbooks/passkey-replay-attack-response.md`. |
| ID-S02 | Recovery envelope held behind OpenBao and operator dual-control policy. | ADR-0243 | `./policy/operator-recovery.cedar`; `./runbooks/recovery-key-mass-issue-investigation.md`. |
| ID-S03 | Issuer, audience, JWKS KID, and tenant mapping pinning. | ADR-0244 | `./IP-011-external-idp-federation.md`; `./runbooks/idp-failover-drill.md`. |
| ID-S04 | Session rotation, refresh family reuse detection, secure cookie attributes. | ADR-0297 | `./decisions/ADR-identity-004-session-class-tiers.md`. |
| ID-S05 | Exact redirect URI, state, nonce, and PKCE binding. | ADR-0145 | `./IP-002-oidc-issuer-kernel.md`. |
| ID-S06 | HSM/OpenBao signing handle and JWKS rotation cadence. | ADR-0003 | `./decisions/ADR-identity-001-jwks-rotation-cadence.md`; `./runbooks/jwks-rotation.md`. |
| ID-S07 | SCIM bearer scoping, constant-time secret comparison, tenant URI binding. | ADR-0244 | `./IP-007-scim-server-kernel.md`; `./policy/tenant-scope.cedar`. |
| ID-S08 | Tenant as universal scoping primitive on every identity read. | ADR-0244 | `./policy/tenant-scope.cedar`; `./IP-017-multi-context-principal-resolver.md`. |
| ID-T01 | OAuth transaction cache stores normalized redirect URI and PKCE hash. | ADR-0145 | `./contracts/openapi/identity.yaml`. |
| ID-T02 | Refresh lineage is append-only and reuse revokes the token family. | ADR-0263 | `./contracts/asyncapi/identity-events.yaml`. |
| ID-T03 | IdP metadata updates require audit emission and admin step-up. | ADR-0243 | `./policy/cedar-acr-predicates.cedar`. |
| ID-T04 | Recovery metadata mutations require operator Cedar permit and audit_id. | ADR-0003 | `./policy/operator-recovery.cedar`. |
| ID-T05 | SCIM protected attributes are deny-listed from external mutation. | ADR-0243 | `./IP-008-scim-adapter-zitadel.md`. |
| ID-T06 | ACR grant is resource-bound and expires by policy. | ADR-0243 | `./IP-010-step-up-orchestrator.md`. |
| ID-T07 | Audit emitter binds identity event names to sealed envelopes. | ADR-0263 | `./IP-012-audit-emitter.md`. |
| ID-E06 | Information-barrier taints flow through principal resolver. | ADR-0319 | `./policy/context-split.cedar`; `./IP-017-multi-context-principal-resolver.md`. |

## Residual Risks Accepted

| Risk ID | Residual risk | Risk owner | Compensating control | Review trigger |
|---|---|---|---|---|
| ID-RR01 | Upstream IdP may be compromised while still producing syntactically valid claims. | axis-identity | Federation anomaly detection and tenant notification playbook. | Any `ExternalIdpBound` or sign-in surge. |
| ID-RR02 | Synced passkeys may behave differently across platform authenticators. | axis-identity | Sign-counter regression alert and recovery ceremony hardening. | WebAuthn library upgrade. |
| ID-RR03 | Recovery keys are user-held and can be socially engineered. | council-security | Dual-control recovery, delayed notification, and risk scoring. | Recovery-key mass issue. |
| ID-RR04 | Session cookies remain exposed to endpoint malware. | ops-security | Step-up for sensitive action and session anomaly scoring. | Credential theft alert. |
| ID-RR05 | External SCIM source can contain stale or malicious role data. | axis-identity | Protected role deny-list and reconciliation jobs. | HRIS integration change. |
| ID-RR06 | JWKS cache lag can briefly accept older public keys. | axis-identity | Rotation overlap window and verifier cache invalidation runbook. | Key rotation incident. |
| ID-RR07 | Observability may not classify newly introduced identity event classes. | axis-observability | ADR-0263 registry update requirement and report-only validator. | New event contract PR. |
| ID-RR08 | Tenant admin can misconfigure federation mappings. | tenant-success-security | Admin step-up, audit trail, and rollback runbook. | New enterprise tenant onboarding. |
| ID-RR09 | Information-barrier context may lag after HR changes. | council-security | Office scope assignment events and periodic reconciliation. | HRIS batch drift. |
| ID-RR10 | Abuse-defence provider outage can reduce bot signal quality. | ops-security | Local rate-limit fallback and `AbuseDefenceVendorOutage` alert. | Vendor outage. |

## Specific Telemetry for Detection

ADR-0263 requires every detection event to carry `tenant_id`, `sub_scope_path`,
`event_id`, `trace_id`, `span_id`, `audit_id`, `schema_version`,
`source_microservice`, `cell_id`, and `jurisdiction_code` when the event is
state-changing. Cedar decision events additionally carry policy id, principal,
action, resource, decision, and deny reason.

| Threat ID | Detection telemetry | ADR-0263 class or service event | Signal |
|---|---|---|---|
| ID-S01 | WebAuthn sign-count regression, challenge replay, RP ID mismatch. | `IdentityWebAuthnRegistered`, `IdentityWebAuthnRevoked`, `AbuseDefenceSpoofDetected` | High-confidence account takeover attempt. |
| ID-S02 | Recovery ceremony from unusual operator, new device, or unusual ASN. | `IdentityWebAuthnRegistered`, `AbuseDefenceCredentialPwned`, `OfficeBoundaryAttemptEvaluated` | Recovery-key compromise or social engineering. |
| ID-S03 | Upstream issuer key change and privileged sign-in spike. | `ExternalIdpBound`, `IdentitySignInSucceeded`, `AbuseDefenceSpoofDetected` | IdP compromise or metadata poisoning. |
| ID-S04 | Refresh token family reuse or fixed session seen before login. | `IdentitySignInSucceeded`, `AbuseDefenceCredentialStuffing` | Session fixation or stolen refresh token. |
| ID-S05 | OAuth state mismatch, nonce mismatch, PKCE failure. | `IdentitySignInFailed`, `AbuseDefenceSpoofDetected` | OAuth flow tampering. |
| ID-S06 | KID unknown, JWKS stale, signature verification failure. | `JwksRotated`, `AbuseDefenceAttestationFailed` | Signing material or verifier drift. |
| ID-S07 | SCIM bearer failure or mutation against protected attribute. | `ScimRequestReceived`, `OfficeBoundaryAttemptDenied` | Provisioning spoof or role injection. |
| ID-S08 | Cross-tenant principal resolution denied. | `ConglomeratePersonalTenantBoundaryRefused`, `OfficeBoundaryAttemptDenied` | Tenant boundary probing. |
| ID-T01 | Authorization code exchange after mismatched redirect or PKCE. | `IdentitySignInFailed`, `AbuseDefenceSpoofDetected` | OAuth injection attempt. |
| ID-T02 | Token family mutation or refresh reuse. | `OidcTokenRevoked`, `AbuseDefenceCredentialStuffing` | Session tamper. |
| ID-T03 | IdP metadata update outside maintenance window. | `ExternalIdpBound`, `OfficeBoundaryClearanceRequested` | Federation trust drift. |
| ID-T04 | Recovery metadata change without paired approval event. | `OfficeBoundaryClearanceDenied`, `IdentityWebAuthnRevoked` | Recovery tamper. |
| ID-DOS01 | Token endpoint 429, signer queue depth, HSM latency. | `AbuseDefenceRateLimitHit`, `AbuseDefenceQuotaExceeded` | Login DoS. |
| ID-DOS03 | IdP outage and federation fallback activation. | `AbuseDefenceVendorOutage`, `IdentitySignInFailed` | Provider outage. |
| ID-E06 | Missing or conflicting office scope on principal. | `OfficeScopeAssignmentChanged`, `InformationBarrierTaintAttached` | Information-barrier bypass attempt. |

## Incident Response Playbook References

| Incident class | Runbook |
|---|---|
| Passkey replay or cloned authenticator | `./runbooks/passkey-replay-attack-response.md` |
| Recovery-key compromise or suspicious mass recovery | `./runbooks/recovery-key-mass-issue-investigation.md` |
| Passkey reset ceremony abuse | `./runbooks/passkey-reset.md` |
| Passkey cross-device debugging | `./runbooks/passkey-cross-device-debug.md` |
| JWKS rotation or stale verifier cache | `./runbooks/jwks-rotation.md` |
| External IdP outage or failover | `./runbooks/idp-failover-drill.md` |
| IP block or brute-force incident | `./runbooks/ip-block-incident.md` |
| Brute-force mitigation | `./runbooks/brute-force-mitigation.md` |
| SCIM provisioning incident | `./runbooks/scim-provisioning-debug.md` |
| Tenant admin onboarding compromise | `./runbooks/tenant-admin-onboard.md` |

## Threat Coverage Ledger

### ID-COV01: Passkey replay coverage

- Threats covered: ID-S01, ID-I05.
- Asset coverage: WebAuthnCredentialPublicKey and AuditEmissionEnvelope.
- Boundary coverage: ID-B03 and ID-B11.
- Required control evidence: challenge TTL, RP ID origin pinning, sign-counter monotonicity, sealed credential lifecycle event.
- Detection evidence: `IdentityWebAuthnRegistered`, `IdentityWebAuthnRevoked`, and `AbuseDefenceSpoofDetected`.

### ID-COV02: Recovery compromise coverage

- Threats covered: ID-S02, ID-T04, ID-E05.
- Asset coverage: RecoveryKeyEnvelope and StepUpAcrGrant.
- Boundary coverage: ID-B09, ID-B10, and ID-B11.
- Required control evidence: `./policy/operator-recovery.cedar`, delayed notification, dual approval, recovery ceremony audit_id.
- Detection evidence: `OfficeBoundaryClearanceRequested`, `OfficeBoundaryClearanceDenied`, and recovery-key runbook trigger.

### ID-COV03: Federation compromise coverage

- Threats covered: ID-S03, ID-T03, ID-E04.
- Asset coverage: ExternalIdpTrustConfig and PrincipalIdentityRecord.
- Boundary coverage: ID-B07 and ID-B05.
- Required control evidence: issuer pinning, audience pinning, JWKS change audit, role mapping review.
- Detection evidence: `ExternalIdpBound`, `ExternalIdpUnbound`, `IdentitySignInSucceeded`, and `AbuseDefenceSpoofDetected`.

### ID-COV04: Session fixation coverage

- Threats covered: ID-S04, ID-T02, ID-E03.
- Asset coverage: SessionBindingRecord and OidcTokenSigningMaterial.
- Boundary coverage: ID-B04 and ID-B09.
- Required control evidence: refresh family rotation, secure cookie attributes, reuse revocation, device binding.
- Detection evidence: `IdentitySignInSucceeded`, `OidcTokenRevoked`, and credential stuffing signal.

### ID-COV05: OAuth flow tampering coverage

- Threats covered: ID-S05, ID-T01.
- Asset coverage: OAuthAuthorizationCode.
- Boundary coverage: ID-B02 and ID-B01.
- Required control evidence: state hash, nonce hash, PKCE verifier binding, exact redirect URI match.
- Detection evidence: token endpoint mismatch trace and `IdentitySignInFailed`.

### ID-COV06: Token signing coverage

- Threats covered: ID-S06, ID-I03, ID-DOS05.
- Asset coverage: OidcTokenSigningMaterial.
- Boundary coverage: ID-B09 and ID-B11.
- Required control evidence: HSM handle, JWKS rotation runbook, verifier cache invalidation, key epoch audit.
- Detection evidence: `JwksRotated`, HSM latency SLO, and signature failure logs with `audit_id`.

### ID-COV07: SCIM provisioning coverage

- Threats covered: ID-S07, ID-T05, ID-E02.
- Asset coverage: ScimProvisioningPayload and PrincipalIdentityRecord.
- Boundary coverage: ID-B08 and ID-B05.
- Required control evidence: bearer scoping, protected-attribute deny-list, tenant URI binding, reconciliation job.
- Detection evidence: `ScimRequestReceived`, protected attribute deny, and HRIS drift alert.

### ID-COV08: Tenant resolver coverage

- Threats covered: ID-S08, ID-I01.
- Asset coverage: PrincipalIdentityRecord and CedarAuthzDecision.
- Boundary coverage: ID-B05 and ID-B12.
- Required control evidence: tenant-scope Cedar policy, multi-context resolver, personal tenant boundary refusal.
- Detection evidence: `ConglomeratePersonalTenantBoundaryRefused` and `OfficeBoundaryAttemptDenied`.

### ID-COV09: ACR step-up coverage

- Threats covered: ID-T06, ID-E01.
- Asset coverage: StepUpAcrGrant and CedarAuthzDecision.
- Boundary coverage: ID-B10 and ID-B04.
- Required control evidence: resource-bound ACR grant, max-age enforcement, jti replay cache.
- Detection evidence: `StepUpGranted`, `StepUpDenied`, and Cedar denied reason in ADR-0263 envelope.

### ID-COV10: Abuse-defence coverage

- Threats covered: ID-T08, ID-DOS01, ID-DOS02, ID-DOS06.
- Asset coverage: AbuseSignalProfile.
- Boundary coverage: ID-B01 and ID-B03.
- Required control evidence: bot score, per-client rate limit, credential-stuffing detection, local fallback limits.
- Detection evidence: `AbuseDefenceRateLimitHit`, `AbuseDefenceCredentialStuffing`, and `AbuseDefenceVendorOutage`.

## Cross-References

- Root service architecture: `./ARCHITECTURE.md`.
- Product requirements: `./PRD.md`.
- OIDC issuer kernel: `./IP-002-oidc-issuer-kernel.md`.
- WebAuthn relying party kernel: `./IP-004-webauthn-relying-party-kernel.md`.
- WebAuthn REST surface: `./IP-005-webauthn-rest.md`.
- AAGUID refresh worker: `./IP-006-aaguid-refresh-worker.md`.
- SCIM server kernel: `./IP-007-scim-server-kernel.md`.
- External IdP federation: `./IP-011-external-idp-federation.md`.
- Audit emitter: `./IP-012-audit-emitter.md`.
- Edge authz rules: `./IP-013-edge-authz-rules.md`.
- Continuous risk scoring: `./IP-014-continuous-risk-scoring.md`.
- Multi-context principal resolver: `./IP-017-multi-context-principal-resolver.md`.
- Identity events contract: `./contracts/asyncapi/identity-events.yaml`.
- Identity OpenAPI contract: `./contracts/openapi/identity.yaml`.
- Tenant scope Cedar policy: `./policy/tenant-scope.cedar`.
- ACR predicates Cedar policy: `./policy/cedar-acr-predicates.cedar`.
- Context split Cedar policy: `./policy/context-split.cedar`.
- Operator recovery Cedar policy: `./policy/operator-recovery.cedar`.
- ADR-0263 observability emission contract: `./../docs/decisions/ADR-0263-observability-emission-contract.md`.
- ADR-0243 Cedar as universal gate: `./../docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- ADR-0244 tenant as universal scoping primitive: `./../docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- ADR-0297 abuse defence baseline: `./../docs/decisions/ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md`.
- ADR-0319 information barrier: `./../docs/decisions/ADR-0319-front-middle-back-office-information-barrier.md`.

## Checkpoint Notes

- This document does not modify identity decisions or runbooks.
- It intentionally references existing incident playbooks rather than editing them.
- It assumes new service-specific audit-event classes must be registered through ADR-0263 before enforcement promotion.
- It accepts that some mitigations are policy/document backed until the implementation packets are promoted.

---


# Autosharding event-drift threat model

*Consolidated from the former `threat-models/threat-model.md` (Wave-15-ZF doctrine propagation). Source ADRs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.*

## §autosharding-event-drift

Source ADRs: ADR-0346, ADR-0347, ADR-0348, ADR-0349.

Threat: autosharding, auto-rebalance, or dynamic sharding automation events can drift from the manifest-declared `sharding_automation` contract, causing a tenant to move to the wrong cell or shard, bypass residency/compliance filters, or leave no reversible audit trail. The threat covers spoofed control-plane principals, tampered threshold inputs, missing audit-chain rows, stale routing cutovers, denial amplification during hot-split/cold-merge, and privilege escalation through unauthorized cross-jurisdiction migration.

Required controls:
- ADR-0348: `oya-governance-sharding-automation-coverage` refuses any microservice manifest without complete autosharding, auto_rebalance, and dynamic_sharding sub-block declarations.
- ADR-0348: `oya-governance-autosharding-manual-mode-refusal` refuses `manual`; the canonical autosharding mode is `control_plane_driven`.
- ADR-0348: `oya-governance-auto-rebalance-residency-honored` requires auto-rebalance to honor residency and compliance packs; cross-jurisdiction migration requires an explicit Cedar permit per ADR-0243.
- ADR-0348: `oya-governance-dynamic-sharding-threshold-coverage` requires explicit hot-split and cold-merge thresholds; default-fill is rejected.
- ADR-0348: `oya-governance-audit-chain-emit-on-automation-events` requires every auto-rebalance, hot-split, and cold-merge event to emit per ADR-0263; `oya-governance-tenant-migration-reversibility` requires a rollback path.
- ADR-0346: `./bin/oya verify --ci-required` is the canonical local pre-push verifier and must mirror cargo fmt, cargo check, cargo clippy, cargo nextest, and `oya gate run-all` before returning success.
- ADR-0347: governance-owned checks use the `oya-governance-*` prefix; threat-model evidence must cite the governance lane names above without reintroducing stale lane vocabulary.
- ADR-0349: Jenkins/GitHub Actions parity and ArgoCD cosign/audit-chain lanes preserve the same controls in self-hostable CI/CD contexts.

Evidence required: every accepted automation event records event_type, tenant_id when tenant-level, cell_id, shard_id when shard-level, pre_state, post_state, residency_check_result, compliance_pack_check_result, cedar_permit_id when applicable, and initiated_by `control_plane:cell-orchestrator` in the audit-chain row. Residual risk remains until Wave 15-ZD proves race-free cutover and rollback under concurrent auto-rebalance, hot-split, and cold-merge jobs.
