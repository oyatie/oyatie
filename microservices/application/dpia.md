---
doc_class: DPIA
template_id: TPL-DPIA
microservice: application
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-application
deciders: council-privacy, ops-security, axis-application, council-architecture
methodology: ICO DPIA template (UK) + CNIL DPIA methodology (FR) + GDPR Art. 35 + KR PIPA Art. 33
related_adrs: [ADR-0028, ADR-0056, ADR-0065, ADR-0117, ADR-0121, ADR-0123, ADR-0131]
related_artifacts:
  - microservices/application/threat-model.md
  - microservices/application/policy/route-isolation.md
  - microservices/application/policy/data-residency.md
  - microservices/application/compliance.md
review_cadence: annually + on every processing-purpose / sub-processor / data-class change
high_risk_triggers_engaged:
  - "Art. 35(3)(a): systematic + extensive evaluation including profiling — YES (route gating decisions are quasi-automated per request)"
  - "Art. 35(3)(b): large-scale processing of special-category data — YES (conditional; PHI surfaces via module-loaded bundles in pack-us-healthcare; KR PIPA Art. 23 sensitive)"
  - "Art. 35(3)(c): systematic monitoring of publicly accessible area — NO"
enforced_frameworks:
  - "GDPR Arts. 5, 6, 7, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 36, 44, 46"
  - "ISO 27001:2022 A.5.34 + A.5.31"
  - "SOC 2 Privacy criteria (P1-P8)"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 3, 15, 17, 18, 22-2, 23, 24, 25, 28, 29, 29-2, 33", "PIPC Notice 2020-7"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308(a)(1)(ii)(A) + §164.312(b) + §164.502(b) + §164.514"]
  pack-eu: ["GDPR Arts. 35 + 36", "EDPB Guidelines 4/2019 + 9/2022"]
  pack-jp: ["APPI Arts. 17, 18, 27"]
  pack-sg: ["PDPA Part III + IV", "MAS Notice 644"]
  pack-au: ["Privacy Act 1988 APP 1 + 5 + 6 + 11 + 12"]
  pack-in: ["DPDPA 2023 §10 + §11"]
  pack-br: ["LGPD Arts. 6 + 7 + 11 + 38 (RIPD)"]
  pack-ae: ["UAE PDPL Art. 23"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 9"]
doc_status: published
---

# Data Protection Impact Assessment: application µservice (Application Shell)

## Step 1 — Identify the need for a DPIA

GDPR Art. 35(1) requires a DPIA where processing is **likely to result in a
high risk to the rights and freedoms of natural persons**. The Application
Shell triggers two of the three Art. 35(3) automatic triggers:

| Trigger | Engaged? | Reasoning |
|---|---|---|
| Art. 35(3)(a): Systematic + extensive evaluation including profiling | **YES** | Per-request Cedar policy evaluation is quasi-automated; the route-resolve decision affects which surfaces an employee may access. |
| Art. 35(3)(b): Large-scale processing of special-category data | **YES (conditional)** | Pack-us-healthcare modules can render PHI in the shell; pack-kr KR PIPA Art. 23 sensitive data may appear in module-rendered content. Conditional ⇒ pack-activated. |
| Art. 35(3)(c): Systematic monitoring of publicly accessible area | NO | Application Shell does not monitor public-area cameras / IoT. |

KR PIPC Notice 2020-7 also mandates a DPIA for systems processing sensitive
personal information at scale — engaged for pack-kr.

DPIA is therefore mandatory pre-deployment. Reviewed by EU DPAs (Art. 35)
and KR PIPC (PIPA Art. 33) at first-tenant onboarding in each jurisdiction.

## Step 2 — Describe the processing

### 2.1 Nature

**What**: Application Shell processes employee sign-in credentials, session
identifiers, route requests, module-fetch requests, tenant-admin actions
(product enable/disable, user provisioning, role assignment), and audit
log entries.

**How**: Browser → CDN (public-class only) → per-pack ingress → auth-gateway
(OIDC/SAML verify) → tenant-context (host + JWT → tenant_id) → shell-routing
(Cedar gate) → module-loader (signed manifest verify) → downstream product
µservice via Workflow.

**Where**: Per-pack region-pinned Application clusters (KR / EU / US / etc.).
Pack-pinning enforces residency per ADR-0117.

**When**: Continuous; sub-second per request.

**Who**: Per the actor table in `threat-model.md`. Employees; tenant admins;
operators; CI runners; module publishers; external auditors.

### 2.2 Scope

**Personal-data classes processed:**

| Class | Examples | Lawful basis (GDPR Art. 6) | Volume estimate |
|---|---|---|---|
| `PII_IDENTIFYING` | Employee email / name / employee_id; OIDC `sub` claim | Art. 6(1)(b) contract necessity | 1 per session start |
| `PII_AUTHN_CREDENTIAL` | OIDC/SAML assertion; session cookie; MFA factor | Art. 6(1)(b) + Art. 6(1)(c) legal obligation (security) | 1 per session start |
| `PII_QUASI_IDENTIFIER` | IP address; user-agent; route history | Art. 6(1)(f) legitimate interest (security operations) | per request |
| `BEHAVIORAL_TENANT_PRODUCT` | Module access counts; admin action timestamps | Art. 6(1)(b) + Art. 6(1)(f) | per click |
| `SENSITIVE_PIPA_ART23` | Hashed customer-id + auxiliary data combos | KR PIPA Art. 15 + 23 (explicit consent at tenant onboarding) | 1 per session |
| `SENSITIVE_HIPAA_PHI` (conditional) | Possible when a healthcare module renders PHI in shell-iframe | HIPAA §164.502(b) minimum-necessary; BAA-required tenant only | varies |
| `AUDIT` | Admin actions, route denials, session lifecycle events | Art. 6(1)(c) legal obligation (audit) | per admin action |

**Sub-processors:**

| Sub-processor | Purpose | Region | DPA / BAA status |
|---|---|---|---|
| Oracle Cloud Infrastructure (OCI) | IaaS for cluster + CDN + object storage | per pack | DPA signed; HIPAA-eligible regions for pack-us-healthcare |
| Cloudflare (optional global overlay) | Public-class CDN cache + WAF | global | DPA signed; no per-tenant data |
| Identity Provider per tenant (Okta / Azure AD / KISA-approved) | OIDC/SAML SSO | per tenant | Tenant-owned; oyatie processes assertions only |
| OpenBao (self-hosted) | Secret management | per pack | n/a (self-hosted) |

### 2.3 Retention

| Data class | Retention | Trigger | Source of truth |
|---|---|---|---|
| Session token (Valkey) | TTL on idle ≤ 15 min; absolute 8 h | last access or hard expiry | `policy/data-residency.md` |
| OIDC/SAML assertion (Postgres) | per pack: KR 5 y; EU 6 y; US-HC 6 y | seal + archive | `policy/data-residency.md` |
| Route audit log | per pack policy; min 1 y for SOC2 | DSR cascade | `compliance.md` |
| Admin audit log | per pack policy; min 6 y where required | DSR cascade | `compliance.md` |
| Module access log | 90 d hot; 1 y cold | DSR cascade | — |

### 2.4 Children's data

Not directly processed; the Application Shell is B2B (employee-of-tenant
context). If a tenant assigns child users (e.g., school-context customer),
the tenant warrants this in the DPA and obtains lawful basis.

## Step 3 — Consultation

| Stakeholder | Consulted | Outcome |
|---|---|---|
| Tenant DPOs (pre-launch) | YES | Two-cookie + PKCE + nonce contract acceptable; manifest signing requirements requested (already in design) |
| Council-privacy | YES | Default-deny Cedar + RLS posture approved |
| Ops-security | YES | OWASP ASVS Level 2 (Level 3 for module-loader) accepted |
| KR PIPC (formal consultation) | Scheduled at first-tenant pack-kr onboarding | — |
| EU DPA (Art. 36 prior consultation) | Conditional — only if high-residual-risk pack-eu first tenant | — |

## Step 4 — Necessity + proportionality

### Lawful basis matrix

| Processing | Art. 6 basis | Art. 9 basis (if special category) | Necessity test |
|---|---|---|---|
| Sign-in (OIDC/SAML verify) | 6(1)(b) contract | n/a | Required to operate the contracted product surface |
| Route Cedar evaluation | 6(1)(b) + 6(1)(c) | n/a | Required to enforce contracted access scope |
| Audit logging | 6(1)(c) | 9(2)(g) (substantial public interest re security) | Required by SOC 2, ISO 27001, KR PIPA Art. 29 |
| Module access metrics | 6(1)(f) legitimate interest | n/a | Balanced test: tenant operations vs. employee privacy; aggregated + retention-limited |
| PHI surface (conditional, pack-us-healthcare) | 6(1)(b) + 6(1)(c) | 9(2)(h) provision of healthcare | BAA-bound; minimum-necessary applied at module-loader |

### Data minimisation

- Session token: opaque 256-bit; no PII embedded.
- Audit log: PII reduced to user_id + tenant_id (no email in audit body).
- Module access log: aggregated; per-user resolution only on admin export with second-factor.

## Step 5 — Risk identification + mitigation

(Cross-reference `threat-model.md` STRIDE + LINDDUN catalogue; this section
re-enumerates risks from the data-subject-rights perspective.)

| R-ID | Risk | Likelihood | Severity | Mitigation | Residual |
|---|---|---|---|---|---|
| R-01 | Session-token leak → impersonation | medium | high | HttpOnly + Secure + SameSite=Lax; 256-bit entropy; HMAC rotation | low |
| R-02 | OIDC IdP compromise → mass sign-in spoofing | low | critical | Audience pin; JWKS rotation 30 d; alert on JWKS change | low |
| R-03 | Cross-tenant data leak via cookie misconfiguration | low | critical | Cookie scoped to `.app.oyatie.dev` only; Domain pin | very low |
| R-04 | Route Cedar bypass → unauthorized module render | low | high | Default-deny; lane refuses missing permit | very low |
| R-05 | Module supply-chain compromise → malicious code execution in shell | medium | critical | Ed25519 signed manifest; SRI; iframe sandbox; CSP | low |
| R-06 | Audit-log tampering | low | high | Ed25519 sealed; immutable JSONL append; Merkle root daily | very low |
| R-07 | DSR (erasure) request not honoured for module access log | medium | high | DSR cascade — observability + tenancy + application coordinated; quarterly drill | low |
| R-08 | Cross-pack data movement (e.g., pack-eu data to pack-us) | low | critical | Pack-pinning enforced by Cedar + cloud-iac policy; refusal by default | very low |
| R-09 | PHI rendered to non-BAA employee within tenant (pack-us-healthcare) | medium | critical | Per-role Cedar permit set; HIPAA minimum-necessary applied to module-loader | low |
| R-10 | Profiling via route-access pattern (Art. 22) | low | medium | Logs aggregated > 24 h; no automated decision affects employee with legal effect | low |
| R-11 | Bundle leak via CDN misconfiguration | low | high | CDN serves public-class only; tenant data path mTLS to origin | very low |
| R-12 | Session-store eviction → silent re-sign-in surprise | medium | low | Refresh token + graceful UI prompt | very low |

## Step 6 — Outcomes

### Mandatory controls (Section 5 → built)

- [x] Cedar default-deny on every route (`policy/route-isolation.md`).
- [x] Two-cookie + PKCE + nonce per ADR-0123.
- [x] Module-loader Ed25519 signature + SRI verify.
- [x] Audit-chain Ed25519 seal (per ADR-0028).
- [x] Pack-pinned residency (`policy/data-residency.md`).
- [x] DSR cascade integration with tenancy + observability.
- [x] CSP + iframe sandbox for product modules.

### Acceptance criteria

| AC | Test |
|---|---|
| DPIA-AC-01 | Session-token entropy ≥256-bit | `test_session_entropy_256bit` |
| DPIA-AC-02 | Cross-tenant cookie scope refused | `test_cross_tenant_cookie_refused` |
| DPIA-AC-03 | DSR (erasure) cascade completes within pack SLO | drill |
| DPIA-AC-04 | Pack-residency invariant verified by lane | `oya gate validate residency-pin --ms application` |
| DPIA-AC-05 | All admin actions Ed25519-sealed | `oya gate validate audit-chain --ms application` |

## Step 7 — Sign-off + review

| Role | Holder | Date |
|---|---|---|
| Council-privacy chair | TBD | M03/P01 entry |
| Ops-security lead | TBD | M03/P01 entry |
| Axis-application lead | TBD | M03/P01 entry |
| DPO (tenant first-pack) | per tenant onboarding | per tenant |

Review cadence: annually + on every processing-purpose / sub-processor /
data-class change. Cross-link to `compliance.md` for framework mapping.
