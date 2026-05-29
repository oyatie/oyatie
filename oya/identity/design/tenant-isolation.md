---
doc_class: DesignNote
title: Workload-Identity Tenant Isolation (Tenant = Trust Domain)
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity
related_adrs: [ADR-0002, ADR-0162, ADR-0183, ADR-0244]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Tenant Isolation

## The isolation primitive: tenant = trust domain

The cited brief's highest-leverage isolation adoption (§6): **tenant = trust
domain.** This is SPIFFE-native — `spiffe://<trust-domain>/<workload-path>` with
a separate trust domain per security environment maps cleanly onto one trust
domain per tenant. Every workload-identity artifact is scoped to the trust domain:

- The principal id (`spiffe://<tenant-trust-domain>/...`).
- The JWKS set the issuer is trusted for (trust-domain→JWKS map).
- The Cedar policy partition (`policyStoreId` == trust domain).
- The decision log slice (ADR-0162 per-tenant).

This aligns with ADR-0244 (tenant as the universal scoping primitive).

## Policy partitioning (brief §6)

The brief contrasts the AVP central fork — per-tenant store (isolation default,
easy off-board, harder global policy management) vs shared store (simpler, must
include tenant id in every policy + request, shared quota). We adopt:

- **Default: per-tenant policy partitions.** Each tenant's `permit`s live in its
  own partition (`policyStoreId`). Off-boarding is dropping the partition.
- **Shared partition: global `forbid` guardrails only.** The cross-cutting
  guardrails (`forbid-suspended-principal`, `forbid-cross-trust-domain`,
  `forbid-audience-mismatch`, `forbid-sensitive-write-without-mfa` in
  `policy/identity.cedar`) live in a shared partition so they cannot be removed
  per-tenant. Because `forbid` overrides `permit` (Cedar formal property), the
  shared guardrails always win over any tenant `permit`.

## Structural cross-tenant impossibility

Cross-tenant authorization is not a feature that is *disabled* — it is
structurally impossible:

1. A request's `policyStoreId` selects a single tenant's partition; a principal
   from tenant A cannot reach tenant B's permits.
2. The global `forbid-cross-trust-domain` denies any tuple where
   `principal.trust_domain != resource.trust_domain`, belt-and-braces over (1).

Both are exercised by AC-W-08.

## Shared external issuer caveat (brief §6)

If a tenant federates a **shared** external issuer (e.g. a single corporate OIDC
issuer serving many tenants), trusting the issuer alone is insufficient — it
invites tenant spoofing. Per the brief (§6, GCP attribute-condition pattern), we
require an **attribute condition**: the token must carry a tenant/org claim that
is verified against the expected trust domain. The issuer is never trusted in
isolation. (This mirrors GCP WIF's anti-spoof attribute conditions.)

## Quota isolation

Per-tenant authorize quotas (manifest `hyperscaler_inv_coverage.tenant_rate_limit`
analog) prevent one tenant's authorize storm from starving another — the
capacity-isolation rationale the human-identity context already applies.

## References

Brief §6 (tenant=trust-domain, per-tenant vs shared store, attribute-condition
for shared issuers); ADR-0162; ADR-0183; ADR-0244. Policy:
`policy/identity.cedar`. Cross-ref: `design/data-residency.md`.
