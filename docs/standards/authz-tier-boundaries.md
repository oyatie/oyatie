---
doc_class: Standard
template_id: TPL-STANDARD
standard_id: authz-tier-boundaries
status: Accepted
date: 2026-05-18
owner_team: axis-identity + axis-api-gateway + council-architecture
related_adrs: [ADR-0157, ADR-0182, ADR-0183, ADR-0191]
related_lanes: [lean-a17-authz-tier-discipline]
---

# Authz Tier Boundaries — Standard

Per ADR-0191, oyatie enforces authorisation in TWO STRICTLY DISJOINT tiers. This document is the canonical boundary table. Cedar policies and Envoy filters MUST conform; the `check-authz-tier-discipline` gate refuses violations.

## Tier definitions

### Edge tier (Envoy Gateway, ADR-0182 north-south)

Owns concerns the edge can decide WITHOUT identity:

- IP geo / ASN deny-list (MaxMind)
- IP rate-limit (per-IP + per-tenant header + per-endpoint)
- Bot detection (Coraza WAF + custom rules)
- WAF (OWASP CRS v4.25.0 LTS)
- DDoS shape (eBPF XDP at NIC + Cilium L3/L4)
- TLS termination (TLS 1.3)

### Origin tier (Istio Ambient waypoint + Cedar PDP, ADR-0183)

Owns concerns that require identity context:

- OIDC bearer verification (signature + iss + aud + exp)
- Principal-Action-Resource (PAR) via Cedar policy
- Tenant-scope (cross-tenant deny)
- Residency (cross-pack deny)
- Time-of-day / business-hours
- ACR floor (step-up per ADR-0189)
- Data-class gate (PII export, audit export)
- Purpose binding
- Idempotency replay defense

## Boundary table

| If you want to … | Enforce at … | NOT at … |
|---|---|---|
| Block country | edge (IP geo) | origin |
| Block user in country | origin (Cedar `principal.residency`) | edge |
| Rate-limit misbehaving IP | edge | origin |
| Rate-limit misbehaving tenant after auth | origin (per-tenant counter) | edge |
| Block SQL-injection payload | edge (WAF CRS) | origin |
| Block cross-tenant access | origin (Cedar tenant-scope) | edge |
| Require MFA next 15min | origin (Cedar acr_required) | edge |
| Block known-bad ASN | edge (ASN deny-list) | origin |
| Block known-bad bot User-Agent | edge (WAF) | origin |

**Hard rule**: if the answer is "both," the design is wrong. Pick one tier.

## Failure-mode independence

- Edge outage → origin MUST NOT over-permit. Cedar refuses requests without an OIDC bearer regardless of edge state.
- Origin outage → edge MUST NOT over-deny. Edge continues serving cached health responses; ext_authz fail-open is forbidden per ADR-0183.

## Audit

Each tier emits its own deny event:

- Edge denies → `EdgeDeny{ reason, tier: edge }`
- Origin denies → `OriginDeny{ reason, tier: origin }`

Both seal to audit-chain. The `tier` field distinguishes for debugging.

## Forbidden patterns

### In Cedar policies (origin tier)

DO NOT reference: `client_ip`, `remote_ip`, `source_ip`, `asn`, `geoip`, `country_code`, `rate_limit`, `waf`, `bot_score`, `ddos`, `user_agent_regex`.

### In Envoy filters (edge tier)

DO NOT reference: `principal.acr`, `acr_level`, `acr_required`, `principal.tenant_id`, `tenant_residency`, `data_class`, `purpose_binding`, `step_up_required`, `cedar_principal`, `oidc_subject`.

If you genuinely need an edge concern in Cedar (e.g., a debug-time exemption for an IP block), add the line-suppression marker `// authz-tier-discipline: ok (<reason>)` and document in PR.

## Verification

CI lane `lean-a17-authz-tier-discipline` runs `check-authz-tier-discipline`:
- scans every `.cedar` file for forbidden edge needles.
- scans every Envoy filter YAML for forbidden origin needles.
- reports findings advisory-mode for 60 days, then blocker.

## Cross-references

- ADR-0191 (this standard's authority)
- ADR-0157 (gateway tier)
- ADR-0182 (north-south vs east-west)
- ADR-0183 (Cedar vs Kyverno separation)
- `crates/check-authz-tier-discipline`
