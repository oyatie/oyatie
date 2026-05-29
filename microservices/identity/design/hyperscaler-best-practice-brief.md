# Workload Identity + Authorization — Hyperscaler Best-Practice Research Brief (2025–2026)

> Source-grounded design foundation for `microservices/identity` (Rust: OIDC/JWKS ES256/RS256 validation → WorkloadPrincipal lifecycle provision→active→suspended→retired → Cedar PARC authorization behind a swap-in trait). Contracts: OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3, Cedar, OpenSLO. Evidence: [OFFICIAL] IETF/CNCF/vendor / [SUPPLEMENTAL]. Produced 2026-05-26 via `/best-practice-research`.

## Load-bearing flags (read first)
- **Algorithm confusion (RS256→HS256, `alg:none`)** is the #1 JWT vuln (RFC 8725). The `ring` verifier MUST bind each `kid`→algorithm server-side and NEVER trust the token's `alg`.
- **JWT replay**: SPIFFE warns JWT-SVIDs are replay-susceptible (prefers X.509-SVID). Since we're JWKS/JWT-centric, replay defenses (short TTL, `aud` binding, `jti`) are mandatory.
- **PDP audit not free**: AWS Verified Permissions (AVP) `IsAuthorized` is NOT logged to CloudTrail by default and lands in `additionalEventData` — we must emit our own decision log.
- **PARC** (principal-action-resource-context) is the correct Cedar request tuple.

## 1. Architecture
Identity = OIDC issuer + JWKS the RP trusts, then token-exchange to short-lived creds. SPIFFE: `spiffe://<trust-domain>/<workload-path>`, separate trust domains per security environment. AWS IRSA/EKS Pod Identity, Azure Entra Workload ID (federated cred → OIDC issuer validation), GCP WIF (OAuth 2.0 token-exchange: fetch keys → verify sig → check claims/attribute-conditions). PDP/PEP split (AWS PG): centralized PDP, PEPs on APIs; the swap-in Cedar trait IS the PDP boundary, the `authorize` endpoint is the PDP API, callers are PEPs. Lifecycle states live in the control-plane registry; short-lived uncached creds mean suspend/retire must invalidate fast.
**Validation pipeline (RFC 8725 §3 + RFC 9068, ordered fail-fast):** (1) `kid`→key from cached JWKS; (2) verify sig against **server-side algorithm allowlist**; (3) reject `alg:none`; (4) validate `iss` + key-belongs-to-issuer; (5) validate `aud`; (6) `exp`/`nbf`/`iat` with ≤60s skew; (7) sanitize `kid`, validate any `jku`/`x5u` (SSRF); (8) explicit `typ`.
**Adopt:** `trust_domain` first-class on WorkloadPrincipal (SPIFFE ID shape); the 8-step pipeline as ordered fail-fast w/ `kid`→alg table; PDP behind the trait with stable `authorize(principal,action,resource,context)→{decision,determining_policies}` (mirror AVP `IsAuthorized`); suspend/retire → denylist consulted at validate-time + bounded token TTL (minutes).

## 2. REST API contract (OpenAPI 3.2.0)
AVP canonical authorize: req `{principal{entityType,entityId},action{actionType,actionId},resource{entityType,entityId},context,entities,policyStoreId}` → resp `{decision:ALLOW|DENY, determiningPolicies:[{policyId}], errors:[]}`. **Empty determiningPolicies + DENY = implicit deny.** Also `IsAuthorizedWithToken` + `BatchIsAuthorized`. Return **403 not 404** for no-policy-match (don't leak existence). `ValidationException` reasons (`UnrecognizedEntityType`,`MissingAttribute`,…) = typed-error template. Token validation body = RFC 9068 claims (`iss,exp,aud,sub,client_id,iat,jti`+`nbf,azp`).
**Adopt:** OpenAPI 3.2.0 `POST /authorize` on AVP `IsAuthorized` (PARC+entities+context → `{decision,determiningPolicies,errors}`, implicit-deny documented); `POST /tokens/validate` (RFC 9068 normalized claims); optional `POST /authorize-with-token`; principal lifecycle as explicit transition sub-resources (`:suspend`/`:retire`, not status PATCH); **403 never 404**; typed error schema (algorithm-mismatch/issuer-untrusted/audience-mismatch/expired/jwks-unavailable/principal-suspended).

## 3. Async / events (AsyncAPI 3.1.0)
EKS lists Auditability via CloudTrail; AVP decisions = CloudTrail data events (opt-in, in `additionalEventData`); GCP requires one-to-one subject mapping for audit correlation. Two families: principal-lifecycle + authz-decision.
**Adopt:** channels `identity.principal.lifecycle.v1` (old→new state, actor, trust-domain) + `identity.authz.decision.v1` (PARC+decision+determiningPolicies+correlation id); stable never-reused subject id on every event; decision events are the primary audit substrate (emit unconditionally — managed analog is opt-in).

## 4. SLOs / SLIs (OpenSLO)
OpenSLO `kind:SLO` (`spec.{service,indicator|indicatorRef,timeWindow,budgetingMethod:Occurrences|Timeslices|RatioTimeslices,objectives}`); SLI = `thresholdMetric` (latency) or `ratioMetric{good,total}` (availability/correctness). Centralized-PDP adds a network hop → budget for it or push to embedded/cached eval. Lifecycle writes eventually consistent (EKS "several seconds").
**Adopt:** three `*.openslo.yaml`: `authorize-latency-p99` (thresholdMetric `op:lte`), `validation-availability` (ratioMetric ≥99.9; JWKS-fetch failures burn budget), `decision-correctness` (ratioMetric vs golden policy-decision test set); separate hot-path authorize-latency from looser control-plane lifecycle-write latency; `budgetingMethod:Occurrences`, rolling 30d.

## 5. Threat model
Algorithm confusion → server-side allowlist, never trust header `alg`, reject `none` (RFC 8725 §3.1–3.2). Forgery/key-substitution → key-belongs-to-issuer (§3.8). Replay → short TTL + `aud` + `jti`. Confused deputy → bind `aud` to specific provider/resource (GCP: use pool-provider URL as audience; SPIRE-AWS pins `aud`). JWKS poisoning/SSRF → validate `jku`/`x5u`, sanitize `kid` (§3.10). Cross-JWT confusion → explicit `typ` (§3.11–12). Privilege escalation → least-privilege + default-deny + forbid-overrides-permit. Lifecycle abuse → unique never-reused ids. NHI 2025 context: machine identities ~82:1 vs humans; mitigation = short-lived/federated/secretless.
**Adopt:** encode RFC 8725 mitigations as explicit verifier test cases (alg=none, RS256→HS256, expired/nbf, wrong-aud, malformed-kid); mandatory `aud` binding per principal/trust-domain; principal-id immutability+non-reuse (retired tombstoned); `jku`/`x5u` allowlist-gated, default static trust-domain→JWKS map.

## 6. Multi-tenant isolation
Central fork (AVP): per-tenant store (isolation default, easy off-board, harder global policy mgmt) vs shared store (simpler, must include tenant id in policies+requests, shared quota). Trust-domain-per-tenant = SPIFFE-native. GCP: shared issuer → attribute conditions to verify org (anti-spoof).
**Adopt:** **tenant = trust domain** isolation primitive; scope every principal/JWKS/Cedar policy-set to tenant; default per-tenant policy partitions, shared partition only for global `forbid` guardrails; shared external issuer → require attribute-condition (tenant/org claim), never trust issuer alone.

## 7. Data residency
Thin official guidance (partial-evidence). Tokens/claims carry PII + subject ids for audit correlation; AVP audit = regional CloudTrail; RFC 8725 §3.10 "do not trust received claims" → validate-not-persist.
**Adopt:** minimize claim persistence (store subject/principal id + decision metadata, not full token bodies); tenant/region-pinnable audit + policy store; classify PII vs operational claims in the OpenAPI/AsyncAPI schemas.

## 8. Cost / FinOps
Dominant driver = authorize call volume; mitigation caching/batching (AVP `BatchIsAuthorized`; PEP-side short-TTL decision cache; embedded Cedar to kill per-call network cost). Caution: decision caching trades cost for staleness (suspended principal retains access until expiry) → couple to revocation SLO.
**Adopt:** `POST /authorize:batch` (AVP analog); short-TTL PEP decision cache, max TTL tied to revocation SLO; prefer **embedded in-process Cedar** for hot paths (the swap-in trait enables it; AVP itself documents embedding the Cedar SDK for intermittent-access cases).

## 9. Audit evidence emission
Canonical = AVP `IsAuthorized` + CloudTrail fields: decision core `decision/determiningPolicies/errors` (empty determiningPolicies+DENY=implicit deny); request context `principal/action/resource/context/policyStoreId`; envelope `eventTime/eventSource/eventName/region/sourceIPAddress/userAgent/userIdentity`. GCP non-repudiation = one-to-one subject mapping.
**Adopt:** one immutable record per authorize: `timestamp, tenant/trust-domain, principal/action/resource/context, decision, determiningPolicies(ids), errors, correlation_id` (preserve implicit-vs-explicit-deny); bind to immutable subject id; never log token bodies (log subject + token hash/`jti` for replay forensics); emit unconditionally to `evidence/audit-chain.jsonl`-style chain; capture validation outcomes AND authorization outcomes as distinct event types.

## 10. Operational boundaries + failure modes
Default = fail-closed/default-deny. Cedar is formally proven: default-deny, forbid-overrides-permit, order-independence (arXiv 2403.04651); AVP implicitly denies unless explicit permit. Failure modes: JWKS-fetch-fail → use valid cached keys, total-fail-no-cache → fail closed (Azure stores only first 100 signing keys — cap key-set); clock skew ≤60s (RFC 9068); policy-store-unavailable → embedded Cedar default-deny; lifecycle writes eventually consistent ("several seconds") → don't gate hot-path authorize on a just-written change.
**Adopt:** fail-closed everywhere on authz path (no key→reject; store unreachable→embedded default-deny; bad token→reject); JWKS resilience (respect cache-control, last-known-good in memory, proactive refresh, cap key-set ≤100, unreachable+empty-cache=hard-deny+budget-burn); bounded ≤60s skew (configurable, never disable-able); decouple control-plane (eventually consistent) from data-plane — consult fast revocation/denylist for suspended/retired, accept brief activation lag.

## Highest-leverage adoptions
1. AVP `IsAuthorized` PARC request/response = the OpenAPI 3.2.0 `authorize` contract + Cedar adapter shape.
2. RFC 8725/9068 8-step validation pipeline w/ server-side algorithm allowlist (the #1 vuln class).
3. tenant = trust domain isolation; per-tenant policy partitions.
4. Fail-closed + embedded Cedar (Cedar's proven default-deny/forbid-overrides-permit) + fast revocation denylist for suspend/retire.
5. Unconditional immutable decision log (validation + authorization events) into the audit-chain; subject-id immutability.

## Sources (official unless marked)
RFC 8725 (JWT BCP) · RFC 9068 (JWT access-token profile) · OpenID Core 1.0 · SPIFFE concepts + keyless OIDC-federation-AWS · AWS EKS Pod Identity / IRSA · AWS Verified Permissions design-authz-strategy + IsAuthorized API + CloudTrail logging + Prescriptive Guidance (PDP/PEP) · Azure Entra Workload Identity Federation (2025-04-09) · GCP Workload Identity Federation + best-practices · Cedar reference + naming + "Expressive Fast Safe Analyzable" (arXiv 2403.04651) · OpenSLO v1 spec · [SUPPLEMENTAL] WorkOS/Curity JWT, Aembit/HelpNetSecurity NHI 2025. (Full URLs in the agent research transcript.) D7 data-residency is weakest-evidenced → principled extrapolation.
