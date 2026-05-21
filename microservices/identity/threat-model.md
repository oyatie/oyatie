---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-security
methodology: STRIDE + OWASP Auth Cheat Sheet 2024 + NIST SP 800-63B AAL ladder
related_adrs: [ADR-0145, ADR-0162, ADR-0183, ADR-0187, ADR-0188, ADR-0189, ADR-0190, ADR-0191]
---

# identity µservice — Threat Model

Identity is **the** highest-impact attack surface in the fleet: compromise of any one of its concerns (token signing key, passkey credential database, SCIM bearer, step-up bypass, edge authz drift) cascades to every consumer µservice. This document enumerates STRIDE threats per surface and the mitigations + detections in place.

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
