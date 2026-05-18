---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-application + ops-security
deciders: council-architecture, ops-security, axis-application, council-privacy
methodology: STRIDE + LINDDUN + OWASP Top 10 (2021) + OWASP ASVS v4 + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0065, ADR-0105, ADR-0117, ADR-0121, ADR-0123, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every auth-protocol or module-loader change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC7.1, CC7.2, CC7.4, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.17, A.5.23, A.5.26, A.5.31, A.5.32, A.8.2, A.8.3, A.8.5, A.8.7, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 6, 25, 28, 32, 33, 35"
  - "OWASP Top 10 (2021): A01-A10"
  - "OWASP ASVS v4 Levels 1+2 (Level 3 for module-loader integrity verification)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.1-2.12", "KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29", "KR 전자금융감독규정 (when payment-tenant)"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308 / §164.310 / §164.312 / §164.514 (Technical Safeguards + de-identification)"]
  pack-eu: ["GDPR Arts. 25 + 32 + 35", "eIDAS 910/2014 (when shell carries QES)", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24"]
  pack-sg: ["PDPA 2012 §11-26", "MAS-TRM v2021"]
  pack-au: ["Privacy Act 1988 APP 1-13", "APRA-CPS 234"]
  pack-in: ["DPDPA 2023 §6-10", "RBI Master Direction on Outsourcing"]
  pack-br: ["LGPD Arts. 6/7/11/14/18/33/46/48"]
  pack-ae: ["UAE PDPL Federal Decree-Law 45/2021"]
  pack-ksa: ["PDPL Royal Decree M/19/2021", "SAMA Cybersecurity Framework 2017"]
doc_status: published
---

# Threat Model: application µservice (Application Shell)

## Purpose

Identify, classify, and mitigate threats to the Application Shell's
confidentiality, integrity, availability, and privacy posture. Because every
tenant interaction passes through Application, a compromise here cascades to
every product. This is the canonical security artifact reviewed by SOC 2
Type 2 examiners, ISO 27001 auditors, GDPR DPAs, and KR PIPC at first-tenant
onboarding.

## Scope

### In-scope

| Component | Concern |
|---|---|
| Leptos WASM frontend | Browser-side: XSS, supply-chain injection of the WASM blob, message-channel abuse |
| `shell-routing` REST | Server-side: route confusion, IDOR, broken Cedar policy |
| `tenant-context` middleware | Tenant confusion / spoofed `tenant_id` / cookie-domain leak |
| `auth-gateway` REST + worker | OIDC/SAML attacks (token-replay, key-confusion, audience-claim mishandling); session hijacking; MFA bypass |
| `module-loader` | Bundle supply-chain (signature forgery, SRI bypass, dependency confusion), CDN-cache poisoning |
| `frontend-bundle-serve` | CDN purge auth, cache-key poisoning, origin-shield mTLS |
| Postgres + Citus shell-state | RLS escape; SQL injection at admin surface |
| Valkey/Redis session store | Eviction-based collisions, MULTI/EXEC race conditions |

### Out-of-scope

- Threats to individual product µservice domain logic — owned by each
  product's threat model. Application enforces Cedar at the shell boundary;
  product-internal authorization is the product's responsibility.
- Threats to OpenBao secret-manager — owned by `cloud-secrets`.
- Threats to GitHub Actions runners — owned by `governance` CI substrate.
- Threats to native client tiers (iOS / Android / desktop) — covered in
  subsequent-to-M03-completion successor-IP threat model.

## Trust Boundaries

```text
┌─ Public Internet ────────────────────────────────────────────────────────┐
│                                                                          │
│   Employee browser                Tenant admin browser                   │
│         │                                  │                             │
│         │ (HTTPS, OIDC, mTLS, HTTP/3)      │                             │
│         ▼                                  ▼                             │
│  ┌─ Per-pack CDN edge (OCI CDN / Cloudflare overlay) ─────────────────┐  │
│  │  - TLS 1.3 termination; HSTS preload                               │  │
│  │  - WAF (OWASP CRS) + bot management + rate-limit                   │  │
│  │  - DDOS protection (provider + Cloudflare)                         │  │
│  │  - Cache: public-class assets only (WASM, fonts, CSS)              │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                              │                                           │
└──────────────────────────────│───────────────────────────────────────────┘
                               ▼
┌─ Per-pack Application cluster ───────────────────────────────────────────┐
│                                                                          │
│  Trust boundary 1: External → Cluster ingress                            │
│                                                                          │
│  ┌─ Istio/Envoy gateway (mTLS termination from CDN; OIDC verify) ─┐      │
│  └────────────────────────────────────────────────────────────────┘      │
│                              │                                           │
│  Trust boundary 2: tenant-context middleware (hostname + JWT → tenant)   │
│                                                                          │
│  ┌─ shell-routing-rest ──────────┐    ┌─ auth-gateway-rest ─────┐        │
│  │  - Cedar policy gate          │    │  - OIDC + SAML handlers │        │
│  │  - Route resolve              │    │  - Two-cookie + PKCE    │        │
│  └───────────────────────────────┘    └─────────────────────────┘        │
│                │                                  │                      │
│  Trust boundary 3: Per-tenant Postgres RLS scope (tenant_id label)       │
│                │                                                         │
│  ┌─ Postgres + Citus (tenant_id shard key; RLS row scope) ─────────┐     │
│  │  - shell_state, session, audit, module_manifest tables          │     │
│  └─────────────────────────────────────────────────────────────────┘     │
│                                                                          │
│  ┌─ Valkey/Redis session store (per-pack cluster; AOF + RDB persistence)┐│
│  │  - SETEX session:<sha256>; eviction allkeys-lru                      ││
│  └──────────────────────────────────────────────────────────────────────┘│
│                                                                          │
│  Trust boundary 4: module-loader → CDN origin (signed manifest fetch)    │
│                │                                                         │
│  ┌─ module-loader-usecase ─────────────────────────────────────────┐     │
│  │  - SRI hash verify                                              │     │
│  │  - Ed25519 signature verify against publisher key (OpenBao)     │     │
│  │  - Per-product key pinning                                      │     │
│  └─────────────────────────────────────────────────────────────────┘     │
│                                                                          │
│  Trust boundary 5: auth-gateway → IdP (OIDC / SAML)                      │
│                │                                                         │
│  ┌─ adapter-oidc / adapter-saml ──────────────────────────────────┐      │
│  │  - JWKS pinning per IdP; rotate via OpenBao                    │      │
│  │  - audience claim = `<pack>.app.oyatie.dev`                    │      │
│  │  - nonce + state PKCE                                          │      │
│  └────────────────────────────────────────────────────────────────┘      │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Actors

| Actor | Trust | Authn | Authz |
|---|---|---|---|
| Anonymous browser | untrusted | none | public-class only (WASM bundle, CSS) |
| Employee | semi-trusted | OIDC/SAML + MFA | Cedar `tenant-scope.cedar` (role-bound) |
| Tenant admin | trusted-elevated | OIDC + WebAuthn step-up | Cedar `tenant-scope.cedar` (admin) |
| Operator (oyatie ops) | trusted-elevated | SPIFFE + JIT elevation via OpenBao | Cedar `auditor-scope.cedar` |
| CI runner | machine-trusted | SPIFFE workload identity | Cedar `ci-scope.cedar` |
| Module publisher (product µservice) | machine-trusted | SPIFFE + Ed25519 publish key | sdk-bound publish capability |
| External auditor | trusted-readonly | JIT elevation; time-boxed token | Cedar `auditor-scope.cedar` |

## STRIDE Threat Catalogue

### S — Spoofing

| ID | Threat | Affected | Likelihood | Impact | Mitigation | Tests |
|---|---|---|---|---|---|---|
| S-01 | OIDC token replay (stolen JWT replayed against shell) | auth-gateway | medium | high | Nonce binding; short TTL (10 min access); rotating refresh; audience pin | `test_jwt_replay_rejected` |
| S-02 | OIDC IdP key confusion (alg=none / HS256 with public key) | auth-gateway | low | critical | Pinned JWKS; algorithm allow-list ES256/RS256 only; explicit alg verify | `test_oidc_alg_none_rejected`, `test_oidc_hs256_rejected` |
| S-03 | SAML signature wrap / XSW attacks | auth-gateway | medium | critical | xmlsec library with strict canonicalization; XSW-1..8 fixtures | `test_saml_xsw_*` battery |
| S-04 | Cookie domain hijack (subdomain takeover serving cookies) | auth-gateway | low | high | Cookie scoped to `.app.oyatie.dev`; Domain restricted; HSTS preload | n/a (operational) |
| S-05 | Tenant spoofing via host header | tenant-context | medium | critical | Hostname → tenant lookup signed by tenancy µservice; mismatch fails closed | `test_host_header_spoof_rejected` |
| S-06 | Module publisher impersonation via dependency confusion | module-loader | medium | critical | Per-product pinned Ed25519 publish key; manifest signer == registered publisher | `test_module_signer_mismatch` |

### T — Tampering

| ID | Threat | Affected | Likelihood | Impact | Mitigation | Tests |
|---|---|---|---|---|---|---|
| T-01 | CDN cache poisoning of WASM bundle | frontend-bundle-serve | low | critical | SRI hash in shell HTML; bundle hash in HTTP header; per-pack origin shield mTLS | `test_cdn_sri_drift_alarms` |
| T-02 | Module manifest tampering on disk / in CDN | module-loader | low | critical | Ed25519 detached signature on manifest; verify before any byte executes | `test_manifest_signature_invalid_rejected` |
| T-03 | XSS via reflected admin-portal input | shell-routing | medium | high | CSP `default-src 'self'`; no inline scripts; output-encode by default in Leptos; OWASP ZAP scan in CI | `test_admin_portal_zap_clean` |
| T-04 | CSRF on admin actions (product enable/disable) | shell-routing | medium | high | Double-submit cookie; SameSite=Lax on session; explicit X-CSRF-Token header on mutating routes | `test_csrf_token_required` |
| T-05 | SQL injection on admin user-management form | shell-routing | low | critical | Parameterized queries; sqlx compile-time check; deny prepared bypass | `test_sql_injection_blocked_*` |
| T-06 | Session ID guessing (low-entropy session token) | auth-gateway | low | critical | 256-bit random; HMAC-bound to user-agent; rotating PKCE nonce | `test_session_entropy_256bit` |
| T-07 | Module-loader cache-key collision | module-loader | low | high | Cache key includes content hash + signer key id + pack | `test_cache_key_collision_avoidance` |

### R — Repudiation

| ID | Threat | Affected | Likelihood | Impact | Mitigation | Tests |
|---|---|---|---|---|---|---|
| R-01 | Admin denies product-enablement action | tenant-context | medium | medium | Ed25519-sealed audit record per ADR-0028; immutable JSONL append | `test_admin_audit_seal_present` |
| R-02 | User denies session start (login attribution dispute) | auth-gateway | low | medium | OIDC IdP-side log + oyatie audit-chain dual record; correlation id propagated | `test_session_audit_correlation_id` |
| R-03 | Module publisher denies publishing a malicious bundle | module-loader | medium | critical | Signer key id + signing timestamp recorded; OpenBao key audit | `test_publisher_audit_trail` |

### I — Information disclosure

| ID | Threat | Affected | Likelihood | Impact | Mitigation | Tests |
|---|---|---|---|---|---|---|
| I-01 | Cross-tenant route leak (URL guessing yields another tenant's surface) | shell-routing | medium | critical | Cedar default-deny; RouteScope.tenant_id bind; `test_cross_tenant_route_denied` | property-test all routes |
| I-02 | Session cookie leak via subresource fetch | auth-gateway | low | high | HttpOnly + Secure + SameSite=Lax; CSP report-only on third-party loads | n/a |
| I-03 | Admin portal exposes other tenant's audit log via filter param | shell-routing | low | critical | RLS row scope on audit table; Cedar gate on route param | `test_audit_filter_cross_tenant_denied` |
| I-04 | Module manifest contains internal-only metadata leaked to browser | module-loader | low | medium | Manifest split: public-facing vs. internal; only public part shipped | `test_manifest_split_no_internal_leak` |
| I-05 | Tenant ID inferred from origin hostname | tenant-context | low | medium | Use `<hash>.app.oyatie.dev` (opaque) not `<tenant-name>.app.oyatie.dev` for sensitive tenants; opt-in cosmetic hostname for non-sensitive | n/a (DNS-level) |

### D — Denial of service

| ID | Threat | Affected | Likelihood | Impact | Mitigation | Tests |
|---|---|---|---|---|---|---|
| D-01 | Auth-gateway flood (credential stuffing) | auth-gateway | high | medium | Rate-limit per source IP + per username; exponential backoff; bot challenge | `test_auth_rate_limit_enforced` |
| D-02 | Session-store memory exhaustion (creating many short-lived sessions) | auth-gateway | medium | high | Per-IP session quota; eviction-policy alarm; pre-warmed pool | `test_session_quota_enforced` |
| D-03 | CDN origin overload on cold start | frontend-bundle-serve | medium | medium | Origin-shield POP + circuit-breaker on origin; pre-warmed bundle versions | `test_origin_shield_engagement` |
| D-04 | WASM bundle decompress bomb | frontend-bundle-serve | low | medium | Per-bundle size cap + decompression-ratio cap | `test_decompress_bomb_rejected` |
| D-05 | Cedar policy evaluation slow path | shell-routing | medium | medium | Pre-compile Cedar policies; per-request budget 10 ms; circuit-break to default-deny on timeout | `test_cedar_eval_budget` |

### E — Elevation of privilege

| ID | Threat | Affected | Likelihood | Impact | Mitigation | Tests |
|---|---|---|---|---|---|---|
| E-01 | Employee role escalates to admin via bypassed Cedar check | shell-routing | low | critical | Default-deny; deny-overrides; lane refuses route without explicit permit | `test_cedar_default_deny` |
| E-02 | Module ships embedded JS that escapes its scope | module-loader | medium | critical | CSP `script-src 'self'`; per-module iframe sandbox; postMessage with origin pin | `test_module_iframe_escape_blocked` |
| E-03 | Session token forging using leaked HMAC key | auth-gateway | low | critical | HMAC key in OpenBao; per-pack rotation 30 d; alert on key fetch outside expected svc account | n/a (operational) |
| E-04 | Admin-only mutation reachable via direct gRPC bypass of shell | tenant-context | low | high | Downstream µservice always re-checks `tenant_id` claim; never trusts shell uniquely | `test_downstream_tenant_recheck` |

## LINDDUN Privacy Threats

| ID | Threat | Affected | Mitigation |
|---|---|---|---|
| L-01 (Linkability) | Cross-tenant linkability via session correlation id | auth-gateway | Correlation id rotates per session; hashed with tenant-scope salt |
| L-02 (Identifiability) | URL path encodes employee email | shell-routing | Use opaque user_id; lane refuses path with `@` |
| L-03 (Non-repudiation) | Excessive forensic data captured on revoked user | auth-gateway | Retention policy per pack; DSR cascade |
| L-04 (Detectability) | Login-flow timing oracle leaks user existence | auth-gateway | Constant-time response; same error shape for "unknown" vs. "wrong" |
| L-05 (Disclosure of info) | Stack-trace exposed in production error pages | shell-routing | Production builds strip stack; sealed error id only |
| L-06 (Unawareness) | Tenant unaware which products active for which roles | tenant-context | Admin-portal product-enablement visibility; audit log export |
| L-07 (Non-compliance) | Session retention exceeds pack policy | auth-gateway | Pack-scoped TTL config; lane refuses default outside pack |

## Threats by Brief Concern

The brief calls out these specific concerns; the catalogue above maps them
explicitly:

| Brief concern | Catalogue refs |
|---|---|
| XSS | T-03 (CSP + Leptos auto-encoding + ZAP scan) |
| CSRF | T-04 (double-submit cookie + SameSite=Lax + X-CSRF-Token) |
| Session hijacking | S-04, T-06, E-03 (cookie scope + entropy + HMAC rotation) |
| Module-loader supply-chain | S-06, T-02, R-03, E-02 (signed manifest + iframe sandbox + audit) |
| Tenant-context confusion | S-05, I-01, I-03, E-04 (hostname signing + Cedar + RLS + downstream recheck) |

## Mitigation Coverage Matrix

| Mitigation | Threats covered | Owner |
|---|---|---|
| Cedar policy gate (default-deny) | I-01, I-03, E-01 | axis-application |
| OIDC/SAML strict verify (alg-pin, JWKS, audience) | S-01, S-02, S-03 | ops-security |
| Two-cookie + PKCE + nonce | S-01, T-06, I-02 | ops-security |
| SRI hash + Ed25519 signed manifest | T-01, T-02, S-06, R-03 | axis-application |
| CSP + iframe sandbox | T-03, E-02 | axis-application |
| RLS on Postgres + tenant-scoped queries | I-01, I-03, E-04 | axis-application |
| Audit-chain Ed25519 seals | R-01, R-02, R-03 | axis-application |
| Rate limit + DDOS protection | D-01, D-02, D-03 | ops-sre-reliability |
| Constant-time response in auth flow | L-04 | ops-security |
| OpenBao for all secrets + key rotation | E-03, I-02 | ops-security |

## Residual Risks

| Residual | Description | Acceptance |
|---|---|---|
| Browser-side XSS via novel Leptos vulnerability | Defended by CSP + ASVS but zero-day possible | Quarterly Leptos CVE sweep; lane refuses unpatched LTS |
| Module publisher private key compromise | Defended by OpenBao audit but undetected exfil possible | 30-d rotation + per-product separation of duties |
| Tenant-aware DNS hijack | Out of scope (DNS provider) | Document in tenant onboarding; HSTS preload mitigates |

## Review

- Quarterly review by council-architecture + ops-security.
- On every change to OIDC / SAML / module-loader: emergency review within 5 business days.
- Evidence: `microservices/application/evidence/threat-model-review/<yyyymm>.md`.
