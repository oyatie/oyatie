---
doc_class: DesignNote
title: Workload-Identity Operational Boundaries
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity + ops-security
related_adrs: [ADR-0002, ADR-0131, ADR-0145, ADR-0182, ADR-0183]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Operational Boundaries

This note draws the lines: what this context owns, what it does not, what it
depends on, and the invariants that hold at every boundary (brief §10).

## What this context OWNS

- The token-validation algorithm (the 8-step RFC 8725/9068 pipeline).
- The authorization decision (Cedar PARC PDP behind `WorkloadAuthorizer`).
- The WorkloadPrincipal lifecycle state machine + fast revocation denylist.
- The immutable decision log (validation + authorization events).

## What this context does NOT own (boundaries)

| Concern | Owner | Boundary |
|---|---|---|
| Token issuance / minting | OIDC-issuer context | We validate, never mint |
| JWKS *serving* for the fleet | OIDC-issuer context | We consume JWKS, never serve |
| Human / interactive auth (passkey, SCIM, step-up) | Human-identity context (`PRD.md`) | Distinct bounded context |
| Edge rate-limiting / WAF | API gateway (ADR-0182) | North-south enforcement upstream of us |
| Secret storage | OpenBao (ADR-0117) | We hold no long-lived secrets |
| Policy authoring UI | Out of scope (files only) | `policy/identity.cedar` |

## Default posture

**Fail-closed / default-deny everywhere** (brief §10). Cedar's formally-proven
default-deny + forbid-overrides-permit + order-independence (arXiv 2403.04651) is
the bedrock; AVP's implicit-deny-unless-explicit-permit is the same model.
Detail: `design/failure-modes.md`.

## Invariants that hold at every boundary

1. **The token header `alg` is never trusted.** The algorithm is bound
   server-side per `kid` (RFC 8725 §3.1). Non-negotiable.
2. **Clock skew ≤ 60s, never disable-able.** (RFC 9068.) Configurable downward,
   never above 60s, never off.
3. **JWKS key-set capped ≤ 100.** (Brief §10, Azure first-100.) Unreachable +
   empty cache = hard-deny + budget burn.
4. **403, never 404 on authz failure.** No existence leak (brief §2).
5. **Principal ids immutable + never reused.** Retired ids tombstoned (brief §5).
6. **Control plane decoupled from data plane.** The eventually-consistent
   lifecycle write ("several seconds", brief §4) never gates hot-path
   *activation*; revocation IS enforced via the fast denylist (brief §10).
7. **Decisions emitted unconditionally** to the audit chain (brief §9).

## North-south vs east-west (ADR-0182 / ADR-0145)

- **North-south** (external/tenant-facing): OpenAPI 3.2.0
  (`contracts/identity.openapi.yaml`), through the API gateway, date-versioned
  per the fleet API-versioning posture.
- **East-west** (mesh-internal hot path): proto3 gRPC
  (`contracts/identity.proto`), governed by ADR-0145, mTLS via SPIFFE
  (ADR-0148). The gRPC face is the latency-critical PEP↔PDP surface.

## Eventual consistency boundary (brief §4, §10)

The brief is explicit that lifecycle writes are eventually consistent (EKS
"several seconds"). The operational rule: never gate a hot-path authorize on a
*just-written* activation (accept brief activation lag), but always consult the
fast revocation denylist for suspended/retired so a compromised identity is cut
off promptly. This split is the F11 entry in `design/failure-modes.md` and is
the reason the authorize-latency SLO is separated from the (looser)
control-plane lifecycle-write latency.

## Dependencies (manifest `depends_on_microservices` subset relevant here)

- `audit-chain` — decision log sealing (ADR-0162).
- `observability` — SLO metrics + burn-rate alerts (ADR-0130/0131).
- `cloud-secrets` / OpenBao — no long-lived secrets held locally (ADR-0117).
- `tenancy` — trust-domain ↔ tenant mapping source of truth.

## References

Brief §2, §4, §5, §9, §10; RFC 8725 §3.1; RFC 9068; ADR-0145, ADR-0182, ADR-0183.
Cross-refs: `design/failure-modes.md`, `design/tenant-isolation.md`,
`runbooks/jwks-fetch-failure.md`, `runbooks/policy-store-unavailable.md`.
