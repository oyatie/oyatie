---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-identity-workload
microservice: identity
bounded_context: workload-identity
status: Proposed
tier: hero-substrate
tier_subtype: substrate-identity
date: 2026-05-26
owner_team: axis-identity
doc_status: published
related_adrs:
  - ADR-0002
  - ADR-0131
  - ADR-0145
  - ADR-0162
  - ADR-0182
  - ADR-0183
related_crates:
  - identity-workload-domain
  - identity-workload-oidc-adapter
  - identity-workload-authz-cedar-adapter
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# PRD — Workload-Identity + Authorization Substrate

> Scope note. This PRD covers the **workload-identity** bounded context only —
> machine-to-machine (non-human) identity and authorization, per ADR-0002 and
> the `identity-workload-*` crates. It is deliberately distinct from the
> human-identity PRD at `microservices/identity/PRD.md` (OIDC issuer, WebAuthn,
> SCIM, step-up), which it does not modify. Every recommendation here is grounded
> in the cited hyperscaler research brief
> (`microservices/identity/design/hyperscaler-best-practice-brief.md`).

---

## 1. Problem

Machine identities now outnumber human identities by roughly 82:1, and the
dominant compromise vector for them is not phishing but **token forgery and
authorization confusion** (brief §5, NHI-2025 context). Every µservice in the
fleet needs to answer two questions on every inter-service call:

1. *Is this caller who it claims to be?* — token validation.
2. *Is this caller allowed to do this?* — authorization.

Today those answers are scattered: each service hand-rolls JWT checks (the
single richest source of the algorithm-confusion vuln class, RFC 8725) and each
service embeds ad-hoc allow/deny logic. The workload-identity substrate
centralizes the **policy decision** (a PDP) and the **token-validation
algorithm** so that:

- The #1 JWT vuln class — algorithm confusion (`RS256→HS256`, `alg:none`) — is
  foreclosed once, server-side, instead of being re-introduced per service
  (brief load-bearing flag #1).
- Authorization is a single, formally-grounded, default-deny Cedar evaluation
  (brief §10) rather than N divergent implementations.
- Suspending or retiring a compromised workload identity takes effect fast,
  bounded by short token TTL + a revocation denylist (brief §1, §10).

### 1.1 Why this is its own bounded context

The human-identity concerns (interactive login, passkeys, SCIM, step-up) have
fundamentally different latency, threat, and lifecycle shapes from
machine-to-machine. Per ADR-0131 (flat, single-concern µservices) and ADR-0132
(no suites), the workload path is modeled as its own bounded context with a pure
domain kernel and two swap-in adapters. This keeps the hot authorize path free
of the human-login dependency surface.

### 1.2 What the hyperscalers do (and what we adopt)

| Source | Pattern | What we adopt |
|---|---|---|
| AWS Verified Permissions `IsAuthorized` | PARC request → `{decision, determiningPolicies, errors}`; empty determiningPolicies + DENY = implicit deny | The exact `/authorize` contract shape + Cedar adapter response (brief §2, §9) |
| RFC 8725 (JWT BCP) + RFC 9068 (access-token profile) | Ordered validation; server-side alg binding; reject `none` | The 8-step pipeline in §3 below (brief §1, §5) |
| SPIFFE | `spiffe://<trust-domain>/<workload-path>`; trust domain per security environment | `trust_domain` first-class on the principal; **trust domain = tenant** (brief §1, §6) |
| AWS Prescriptive Guidance | Centralized PDP, PEPs on APIs | The Cedar trait IS the PDP boundary; `/authorize` is the PDP API; callers are PEPs (brief §1) |
| Cedar (arXiv 2403.04651) | Default-deny, forbid-overrides-permit, order-independence (formally proven) | The authorization semantics the policy set relies on (brief §10) |

---

## 2. Goals and scope

### 2.1 Goals

- Validate OIDC/JWS workload tokens (ES256, RS256, RS384, RS512 via `ring`) with
  a server-side algorithm allowlist — never trusting the token header `alg`.
- Provide a Cedar PARC authorization PDP behind a swap-in trait
  (`WorkloadAuthorizer`), so the policy engine can move from an in-crate faithful
  Cedar evaluator to the upstream `cedar-policy` crate without touching callers.
- Model the WorkloadPrincipal lifecycle (provision → active → suspended →
  retired) with tenant, capabilities, scopes, and claims as first-class fields.
- Emit an immutable decision log (validation outcomes AND authorization
  outcomes) as the primary audit substrate.
- Fail closed on every uncertainty.

### 2.2 MVP scope (T3 first slice)

The MVP is the union of the three crates already declared in the manifest:

1. `identity-workload-domain` — pure domain kernel (zero deps): the
   WorkloadPrincipal model, the lifecycle state machine, and the PARC decision
   types.
2. `identity-workload-oidc-adapter` — `ring`-backed JWS validation
   implementing the 8-step pipeline; projects verified claims into a principal.
3. `identity-workload-authz-cedar-adapter` — the in-crate faithful Cedar
   evaluator behind `WorkloadAuthorizer` (default-deny, forbid-wins, lifecycle
   precondition), with the upstream `cedar-policy` crate as the documented swap
   target.

The REST surface (`/authorize`, `/tokens/validate`, `/authorize-with-token`,
`/authorize:batch`, principal lifecycle transitions) and the gRPC surface are
specified in `contracts/identity.openapi.yaml` and `contracts/identity.proto`.

### 2.3 Non-Goals (explicit)

1. **Human / interactive authentication** — passkeys, WebAuthn, social IdP,
   SCIM, step-up. Owned by the human-identity context (`PRD.md`); out of scope here.
2. **Token issuance / minting** — this context VALIDATES tokens; it does not mint
   them. Issuance is the OIDC-issuer context.
3. **JWKS *serving*** — we *consume* JWKS to validate; serving the fleet JWKS is
   the OIDC-issuer context.
4. **Policy authoring UI** — Cedar policies are authored as files
   (`policy/identity.cedar`); no in-product policy editor in this slice.
5. **Cross-tenant / cross-trust-domain authorization** — structurally forbidden
   (brief §6); never a feature.
6. **Long-lived workload credentials** — we assume short-lived, frequently-rotated
   tokens (brief §1, §5); long-lived secret distribution is out of scope.
7. **Decision caching at the PDP** — a PEP-side short-TTL cache is a FinOps option
   (§7), but the PDP itself does not cache decisions in the MVP, to avoid the
   staleness-vs-revocation hazard (brief §8).

---

## 3. Recommended design

### 3.1 The 8-step token-validation pipeline (ordered, fail-fast)

Per RFC 8725 §3 + RFC 9068, executed in order; any failure stops the pipeline and
returns the typed reason (brief §1):

1. **`kid` → key.** Resolve the key id from the cached JWKS. Unknown `kid` →
   refuse (no implicit "try them all").
2. **Verify signature against the server-side algorithm allowlist.** The
   algorithm is chosen from the `kid`→alg binding, NOT from the token header.
   Allowlist: `{ES256, RS256, RS384, RS512}`.
3. **Reject `alg:none`.** Unconditionally (RFC 8725 §3.2) → `alg-none-rejected`.
4. **Validate `iss` + key-belongs-to-issuer.** The resolved key must belong to
   the asserted issuer (RFC 8725 §3.8) → `issuer-untrusted` on mismatch.
5. **Validate `aud`.** Must match the PEP/resource's expected audience
   (confused-deputy defense, brief §5) → `audience-mismatch`.
6. **Validate `exp` / `nbf` / `iat`.** With a configurable skew **≤ 60s** that is
   never disable-able (RFC 9068) → `expired` / `not-yet-valid`.
7. **SSRF defenses.** Sanitize `kid`; if a `jku`/`x5u` is present it MUST be on
   the allowlist (default: the static trust-domain→JWKS map) → `jku-not-allowlisted`.
8. **Explicit `typ`.** Require the access-token `typ` (cross-JWT-confusion
   defense, RFC 8725 §3.11–12).

This pipeline is implemented in `identity-workload-oidc-adapter` and exposed
as `/tokens/validate` and the `ValidateToken` RPC. Each numbered step maps to a
mandatory verifier test case (see §6 and the canonical `../threat-model.md#workload-identity-threat-model`).

### 3.2 Trust domain = tenant

The WorkloadPrincipal carries a SPIFFE-shaped `trust_domain` that **equals the
tenant** (brief §1, §6). Every JWKS set, every principal, and every Cedar policy
partition (`policyStoreId`) is scoped to the trust domain. Cross-trust-domain
authorization is impossible by construction (a global Cedar `forbid`, §3.4).

### 3.3 PDP / PEP split

The `WorkloadAuthorizer` trait is the PDP boundary (brief §1). `/authorize` is
the PDP API; the callers (Envoy waypoints, sidecars, in-process gates) are PEPs.
The response mirrors AVP `IsAuthorized`: `{decision, determiningPolicies,
errors}`. An empty `determiningPolicies` on a `DENY` is an **implicit deny** (no
permit matched); a non-empty one is an explicit `forbid`. Both denials are
preserved distinctly in the decision log for forensics (brief §9).

### 3.4 Fail-closed Cedar

The Cedar adapter relies on Cedar's formally-proven default-deny +
forbid-overrides-permit + order-independence (brief §10). The policy set
(`policy/identity.cedar`) layers global `forbid` guardrails (suspended principal,
cross-trust-domain, audience mismatch, sensitive-write-without-MFA) over
capability/scope `permit`s. On any failure of the path — no key, store
unreachable, malformed token, suspended principal — the answer is DENY
(`design/failure-modes.md`).

### 3.5 Lifecycle + fast revocation

The lifecycle state machine is `provision → active → {suspended ⇄ active} →
retired`. `retired` is terminal and tombstoned (ids never reused, brief §5).
Suspend/retire write the principal id to a fast revocation denylist consulted at
token-validate time. Because workload tokens are short-lived (minutes),
end-to-end revocation latency is bounded by `token_ttl + denylist_propagation`
(brief §1, §10). The control plane is eventually consistent ("several seconds",
brief §4); the hot authorize path is NOT gated on a just-written activation, but
IS gated on the denylist for suspend/retire.

---

## 4. API surface (summary)

| Endpoint | Purpose | Contract |
|---|---|---|
| `POST /authorize` | PARC decision (AVP IsAuthorized) | `contracts/identity.openapi.yaml` |
| `POST /authorize:batch` | Batch PARC (AVP BatchIsAuthorized) | ditto |
| `POST /authorize-with-token` | Validate token then authorize | ditto |
| `POST /tokens/validate` | 8-step RFC 9068 validation | ditto |
| `POST /principals/{id}:suspend` | active → suspended | ditto |
| `POST /principals/{id}:retire` | → retired (terminal) | ditto |
| gRPC `WorkloadAuthorizer` / `WorkloadTokenValidator` | east-west hot path | `contracts/identity.proto` |

Authorization failures return **403, never 404** (no existence leak). Token
failures return a typed **422**. Store/JWKS unavailability returns **503**, which
PEPs MUST treat as DENY.

---

## 5. Non-functional requirements

| Field | Value | Evidence |
|---|---|---|
| Authorize latency | p99 ≤ 10ms (embedded Cedar) | `slos/authorize-latency-p99.openslo.yaml` |
| Validation availability | ≥ 99.9% definitive non-5xx verdicts | `slos/validation-availability.openslo.yaml` |
| Decision correctness | ≥ 99.999% golden-set match | `slos/decision-correctness.openslo.yaml` |
| Clock skew | ≤ 60s, never disable-able | RFC 9068; `design/operational-boundaries.md` |
| JWKS key-set cap | ≤ 100 signing keys held | brief §10 (Azure first-100 cap) |
| Fail posture | fail-closed on every authz-path uncertainty | `design/failure-modes.md` |
| Data residency | tenant/region-pinnable; claims minimized, not persisted | `design/data-residency.md` |
| Audit | one immutable record per authorize + per validation | `design/audit-evidence-emission.md` |

---

## 6. Acceptance Criteria

Applying convergent `/idea-refine` discipline, the design is accepted when every
criterion below is independently verifiable. Each maps to a brief recommendation
and a test surface.

| ID | Criterion | Verification |
|---|---|---|
| AC-W-01 | A token presenting `alg:none` is rejected with `alg-none-rejected`; never validated. | nextest (oidc-adapter) |
| AC-W-02 | A token signed `HS256` whose `kid` maps to an RSA key is rejected `algorithm-mismatch` (RS256→HS256 confusion foreclosed). | nextest |
| AC-W-03 | The token header `alg` is never read to select the verification algorithm; the `kid`→alg binding is the sole source. | code review + nextest |
| AC-W-04 | `exp`/`nbf`/`iat` are enforced with skew ≤ 60s; skew cannot be configured above 60s nor disabled. | nextest |
| AC-W-05 | A `jku`/`x5u` not on the allowlist is refused `jku-not-allowlisted` (SSRF). | nextest |
| AC-W-06 | `/authorize` with no matching permit returns `DENY` + **empty** `determiningPolicies` (implicit deny), distinct from explicit forbid. | nextest (cedar-adapter) |
| AC-W-07 | A suspended principal is denied with determining policy `forbid-suspended-principal`. | nextest |
| AC-W-08 | A principal acting outside its trust domain is denied (`forbid-cross-trust-domain`), regardless of any permit. | nextest |
| AC-W-09 | A sensitive write without `mfa_present` in context is denied even when the capability is held (forbid wins). | nextest |
| AC-W-10 | Authorization failure returns HTTP 403, never 404; resource existence is not leaked. | contract test |
| AC-W-11 | Policy-store unavailability resolves to fail-closed DENY (embedded Cedar default-deny) and burns the validation-availability budget. | integration + SLO assertion |
| AC-W-12 | JWKS fetch failure with empty cache resolves to a hard-deny 503, not an allow. | integration; `runbooks/jwks-fetch-failure.md` |
| AC-W-13 | Every authorize call AND every validation outcome emits exactly one immutable event with a stable, never-reused subject id. | nextest + AsyncAPI contract |
| AC-W-14 | A retired principal id is tombstoned and cannot be reused for a new provision. | nextest |
| AC-W-15 | The `WorkloadAuthorizer` trait can be swapped from the in-crate evaluator to upstream `cedar-policy` without changing callers. | architecture-boundaries gate |
| AC-W-16 | `identity-workload-domain` imports zero peer crates (pure kernel). | architecture-boundaries gate |
| AC-W-17 | Token bodies are never logged; only `sub` + `jti` appear in forensic logs. | code review + log-sieve gate |
| AC-W-18 | The golden policy-decision corpus covers explicit-permit, explicit-forbid, implicit-deny, suspended, cross-trust-domain, and sensitive-write cases, and is replayed by the correctness prober. | test corpus review |

---

## 7. Cost / FinOps posture

Dominant cost driver is authorize call volume (brief §8). Mitigations adopted:
`POST /authorize:batch` (AVP analog), a PEP-side short-TTL decision cache whose
max TTL is tied to the revocation SLO, and **embedded in-process Cedar** on hot
paths via the swap-in trait. Detail: `design/cost-finops.md`.

---

## 8. Open questions

| # | Question | Bias |
|---|---|---|
| 1 | Should the PDP offer an optional decision cache, or push all caching to PEPs? | PEP-side only in MVP (staleness hazard, brief §8). |
| 2 | Adopt upstream `cedar-policy` immediately, or ship the in-crate evaluator first? | In-crate first (offline workspace); swap when vendored. |
| 3 | Max batch size for `/authorize:batch`? | 30 (AVP parity); revisit with load evidence. |

---

## 9. References

- Research brief: `microservices/identity/design/hyperscaler-best-practice-brief.md`
- Contracts: `contracts/identity.openapi.yaml`, `contracts/identity.asyncapi.yaml`, `contracts/identity.proto`
- Policy: `policy/identity.cedar`
- SLOs: `slos/authorize-latency-p99.openslo.yaml`, `slos/validation-availability.openslo.yaml`, `slos/decision-correctness.openslo.yaml`
- Design notes: `../threat-model.md` (canonical, consolidated threat model — workload section `#workload-identity-threat-model`), `design/failure-modes.md`, `design/data-residency.md`, `design/cost-finops.md`, `design/audit-evidence-emission.md`, `design/tenant-isolation.md`, `design/operational-boundaries.md`
- Implementation plan: `IP-001-identity-design.md`
- ADRs: ADR-0002 (tenant-and-identity-kernel), ADR-0131 (flat layout), ADR-0183 (Cedar policy-engine separation)
