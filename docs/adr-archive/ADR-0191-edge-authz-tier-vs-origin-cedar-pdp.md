---
id: ADR-0191
status: Superseded
deciders: council-architecture, axis-identity, ops-security, axis-api-gateway
date: 2026-05-18
owner: axis-identity + axis-api-gateway
supersedes: []
superseded_by: [ADR-0702]
related: [ADR-0145, ADR-0148, ADR-0157, ADR-0178, ADR-0182, ADR-0183, ADR-0187]
related_specs:
  - /specs/microservices/manifest-schema.json
microservice: identity
versions_current_as_of: 2026-05-18
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0191 — Edge authz tier separation: Envoy Gateway edge (IP/rate/WAF/bot/DDoS) vs Istio waypoint Cedar PDP origin (identity/context/step-up/data-class); never overlap

## Status

Accepted (2026-05-18). Defines two strictly disjoint authorization tiers: the *edge* tier at Envoy Gateway (north-south per ADR-0182, gateway tier per ADR-0157) enforces what the edge can see without identity (IP, rate, geo, ASN, bot signature, WAF rule, DDoS shape). The *origin* tier at the Istio Ambient waypoint enforces what only identity-context can decide (principal, action, resource, tenant, residency, time, ACR, data-class). **No concern is enforced at both tiers.**

## Context

Without an explicit boundary table, authz duplicates: WAF rules creep into Cedar policy, Cedar policies attempt geo/IP gates the edge already enforces, gateway rate-limits get re-implemented per µservice. The result: drift, contradictory denies, audit confusion ("which tier rejected this?"), and weight-bearing logic in two places.

The hyperscaler reference shape:

- **Cloudflare** — WAF + bot + DDoS + geo at the edge; Cloudflare Access (identity) is a separate product mounting downstream.
- **AWS** — CloudFront + WAF + Shield at the edge; IAM / Verified Permissions (Cedar) at the origin.
- **Google** — Cloud Armor + reCAPTCHA at the edge; IAM Conditions at the origin.

The lesson: **edge knows packets; origin knows people. Drop what packets reveal at the edge; let the origin decide what people reveal.**

## Decision

**Two tiers; one concern per tier. Neither tier reimplements the other.**

### Edge tier — Envoy Gateway (ADR-0182, ADR-0157)

Owned by `api-gateway` µservice. Enforces:

| Concern | Mechanism | Source of truth | Failure response |
|---|---|---|---|
| IP block (geo deny-list) | MaxMind GeoIP + per-pack policy | residency policy + abuse ledger | 403 with `X-Block-Reason: geo` |
| IP block (ASN deny-list) | MaxMind ASN | abuse ledger | 403 with `X-Block-Reason: asn` |
| IP rate limit (per-IP) | Envoy rate-limit filter | Redis-backed per-IP counters | 429 with `Retry-After` |
| Tenant rate limit (per-Org-ID header, pre-auth) | Envoy rate-limit filter | per-pack tenant quotas (ADR-0155) | 429 with `Retry-After` |
| Endpoint rate limit (per-route) | Envoy rate-limit filter | per-endpoint registry (ADR-0178) | 429 with `Retry-After` |
| Bot detection | Coraza WAF v3 + custom oyatie rules | OWASP CRS v4.25.0 LTS [^1] | 403 with `X-Block-Reason: bot` |
| WAF (OWASP CRS v4.x) | Coraza WAF v3 [^1] | OWASP CRS v4.25.0 LTS | 403 with `X-Block-Reason: waf` |
| DDoS shape (L3/L4) | eBPF XDP at NIC + Cilium L3/L4 filter (ADR-0148 Tier 1) | per-pack DDoS profile | drop (no response) |
| TLS termination | Envoy with mTLS to upstream | per-pack TLS chain | 502 |

### Origin tier — Istio Ambient waypoint Cedar PDP (ADR-0145, ADR-0183, ADR-0148 Tier 3)

Owned by every µservice's waypoint policy. Enforces:

| Concern | Mechanism | Source of truth | Failure response |
|---|---|---|---|
| Identity verification | OIDC bearer verification | Zitadel JWKS (ADR-0187) | 401 |
| Principal-action-resource (PAR) | Cedar policy | per-µservice `policy/*.cedar` | 403 |
| Tenant-scope (cross-tenant deny) | Cedar policy | tenant binding in JWT claim | 403 |
| Residency (cross-pack deny) | Cedar policy + residency lookup | residency policy + tenant pack | 403 |
| Time-of-day / business-hours | Cedar policy | per-tenant policy | 403 |
| ACR floor (step-up) | Cedar policy + JWT `acr` (ADR-0189) | per-action `acr_required` | 401 with `X-Step-Up-Required` |
| Data-class gate (PII export, audit export) | Cedar policy + JWT `data_class` claim | per-action data-class envelope | 403 |
| Purpose binding | Cedar policy + JWT `purpose` | per-action purpose registry | 403 |
| Idempotency replay | Idempotency-key cache (ADR-0149) | per-tenant Redis cache | 200 cached / 409 conflict |

### Boundary discipline

| If you find yourself wanting to … | Then enforce at … |
|---|---|
| block requests from a country | edge (IP geo) |
| block requests from a user in a country | origin (Cedar policy referencing principal.residency) |
| rate-limit a misbehaving IP | edge |
| rate-limit a misbehaving tenant after auth | origin (post-auth per-tenant counter) — NOT edge |
| block a SQL-injection payload | edge (WAF CRS) |
| block a request that targets another tenant's resource | origin (Cedar tenant-scope) |
| require MFA for the next 15 minutes for a sensitive operation | origin (Cedar acr_required) |
| block a known bot user-agent | edge (WAF) |
| block a known bot ASN even if useragent looks human | edge (ASN deny-list) |

If the answer is **both**, the design is wrong. Re-decide one tier exclusively.

### Audit emission

- Edge denies → `EdgeDeny` event with `reason` enum (geo, asn, rate, waf, bot, ddos), partial PII (truncated User-Agent, /24 IP), no principal (pre-auth).
- Origin denies → `OriginDeny` event with `reason` enum (auth, policy, residency, step-up, data-class), full principal + tenant + action + resource.
- Both emit to `audit-chain` (Bominal ADR-0028 seal).

### Failure-mode independence

- Edge tier failure (Envoy outage) does NOT cause origin tier to over-permit. Waypoint Cedar PDP refuses requests without an OIDC bearer regardless of edge state.
- Origin tier failure (Cedar PDP outage) does NOT cause edge tier to over-deny. Edge continues serving cached health responses; ext_authz fail-open is forbidden per ADR-0183.

## Alternatives considered

### Single-tier (everything at the gateway)

Rejected. Gateway cannot reach the per-tenant entity graph; identity-aware decisions require origin context.

### Single-tier (everything at the origin)

Rejected. Origin should not see DDoS traffic; per-µservice WAF would n+1 redundant work and fail to coordinate across endpoints.

### Cloudflare Workers as authz tier

Considered for the edge tier; rejected in favor of Envoy Gateway per ADR-0182 (lock-in posture + sovereign-pack air-gap compatibility per ADR-0173).

### Dual enforcement (defense-in-depth on identical concern)

Rejected. Duplicate enforcement creates contradictory denies during config drift; debugging cost > marginal defense gain.

## Consequences

### Positive

- Single audit log records exactly one denying tier per request.
- Policy drift is detectable: `oya-check-authz-tier-discipline` gate refuses any Cedar policy that mentions IP / ASN / geo, and refuses any Envoy filter that mentions OIDC principal claims.
- Performance: edge drops at packet/L7-pre-auth boundary, saving origin CPU.

### Negative

- Adds an explicit standards-doc requirement: developers must read `docs/standards/authz-tier-boundaries.md` before writing a Cedar policy or an Envoy filter.
- Bot detection sometimes wants to vary by tenant ("tenant X opted in to stricter bot rules") — that case routes through a per-tenant edge-config registry; the boundary remains intact.

### Neutral

- The `oya-check-authz-tier-discipline` gate runs in advisory mode at first; promotion to blocker after 60 days of clean output.

## Implementation

- Standards doc `docs/standards/authz-tier-boundaries.md` lists the boundary table with concrete file-path examples.
- `crates/oya-check-authz-tier-discipline` advisory gate scans Cedar files + Envoy filter configs.
- `microservices/identity/iac/kustomize/components/edge-authz-rules/` ships Coraza WAF rules + rate-limit configs + geo blocks.
- `microservices/api-gateway/` declares edge filter chain referencing this ADR.

## Verification

- Lane `lean-a17-authz-tier-discipline` (advisory) runs `oya-check-authz-tier-discipline` against the repo.
- Integration test: cross-tenant request denied at origin (Cedar) with audit event tagged `tier=origin`.
- Integration test: known-bad-ASN request denied at edge (Envoy) with audit event tagged `tier=edge`.
- Smoke test: edge outage → origin still enforces tenant-scope (failure-mode independence).

## In-house roadmap

Per user directive 2026-05-18, evaluated under in-house policy. Every substrate listed here is open-source or open-standard with no vendor lock; no Phase-2 replacement required.

| Component | Tier | Substrate | Status | Phase-2 trigger |
|---|---|---|---|---|
| Edge IP / ASN / geo filter | edge | Envoy Gateway filter (CNCF) | KEEP | none — CNCF standard |
| Edge rate limit | edge | Envoy rate-limit filter (CNCF) | KEEP | none |
| Edge WAF rules | edge | Coraza WAF v3 (OWASP, Go, Apache-2.0) + OWASP CRS v4.25.0 LTS | KEEP | only if Coraza becomes unmaintained |
| Edge DDoS shape | edge | Cilium L3/L4 (CNCF) + eBPF XDP | KEEP | none — Linux kernel substrate |
| Edge TLS termination | edge | Envoy (CNCF) | KEEP | none |
| Origin OIDC verification | origin | `oya-shared-oidc-client-kernel` (in-house) | KEEP | n/a — already in-house |
| Origin Cedar PDP | origin | Cedar runtime via `oya-policy-cedar-domain` / `-api` (in-house consumer of Cedar OSS) | KEEP | only if Cedar becomes unmaintained |
| Origin step-up evaluator | origin | `oya-identity-step-up-orchestrator-*` (in-house) | KEEP | n/a — already in-house |
| Origin data-class gate | origin | `oya-data-boundary-kernel` (in-house) | KEEP | n/a — already in-house |

The boundary discipline itself (`docs/standards/authz-tier-boundaries.md`) is in-house policy; the `oya-check-authz-tier-discipline` advisory gate is in-house code.

Conclusion: every component in both tiers is OSS or in-house; no vendor swap required.

## Cross-references

- ADR-0145 inter-microservice-communication-reform
- ADR-0148 service-mesh-cilium-ambient-layered
- ADR-0157 api-gateway-tier
- ADR-0178 layered-throttling-tiers
- ADR-0182 api-gateway-north-south-vs-service-mesh-east-west-separation
- ADR-0183 policy-engine-separation-cedar-app-authz-kyverno-admission
- ADR-0187 canonical-oidc-idp-zitadel-primary
- OWASP CRS v4.25.0 LTS (https://coreruleset.org/)

[^1]: Versions current as of 2026-05-18. Coraza WAF v3 + OWASP CRS v4.25.0 LTS. Sources: https://github.com/corazawaf/coraza ; https://coreruleset.org/
