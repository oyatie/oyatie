---
doc_class: DesignNote
title: Workload-Identity Data Residency
microservice: identity
bounded_context: workload-identity
status: Proposed
date: 2026-05-26
owner_team: axis-identity + council-privacy
related_adrs: [ADR-0002, ADR-0162, ADR-0179]
research_brief: microservices/identity/design/hyperscaler-best-practice-brief.md
---

# Workload-Identity — Data Residency

> The cited brief flags D7 (data residency) as its weakest-evidenced domain
> (§7, "thin official guidance — principled extrapolation"). This note is
> explicit about which positions are sourced and which are principled
> extrapolation, per the honest-claims discipline.

## What data this context touches

| Data | Class | Persisted? |
|---|---|---|
| WorkloadPrincipal id (`sub`) | PII_QUASI_IDENTIFYING (identifying for a workload) | Yes — id + lifecycle only |
| Trust domain / tenant id | INTERNAL_ONLY | Yes |
| Capabilities / scopes | INTERNAL_ONLY | Yes |
| Operational claims (e.g. `source_cell`) | INTERNAL_ONLY | Yes (operational subset only) |
| Full token body | AUTHENTICATION | **No** — validate-not-persist |
| Token `jti` | AUTHENTICATION | Hash/id only, for replay forensics |
| Decision records | AUDIT | Yes (immutable, see audit-evidence-emission) |

## Adopted positions (brief §7)

1. **Minimize claim persistence.** Store the subject/principal id + decision
   metadata; never persist full token bodies. RFC 8725 §3.10 ("do not trust
   received claims") reinforces validate-not-persist. [SOURCED]
2. **Tenant/region-pinnable audit + policy store.** The decision log and the
   Cedar policy partitions are pinnable to the tenant's region/cell so identity
   audit follows the tenant's residency (AVP audit is regional; analog adopted).
   [SOURCED for the AVP analog; PINNING mechanism is oyatie-specific]
3. **Classify PII vs operational claims in the schemas.** The OpenAPI/AsyncAPI
   schemas separate operational context (allowed in `context`) from
   identity/PII claims; PII is never placed in the Cedar `context` record.
   [SOURCED §7]

## Residency mechanics (ADR-0179 alignment)

- The trust-domain→JWKS map and the Cedar policy partitions are per-tenant and
  therefore inherit the tenant's pack residency (e.g. pack-kr stays in KR,
  pack-eu in EU). There is no cross-pack federation of workload-identity state,
  matching the human-identity residency stance (ADR-0179).
- Decision records are written to the tenant/region-pinned audit chain
  (ADR-0162 per-tenant slicing). A decision generated in `eu-frankfurt-1` is
  sealed in the EU slice.

## Principled extrapolation (clearly marked)

The brief is thin on official residency guidance for the authorization substrate
specifically. The following are principled extrapolations, not vendor-sourced:

- We treat a workload principal id as quasi-identifying because it can be
  correlated to a tenant's internal topology, and therefore residency-bind it
  like the audit trail.
- We assume the strictest applicable pack floor wins when a principal spans
  contexts (it should not, given trust-domain = tenant, but the floor is the
  safe default).

## References

Brief §7 (explicitly weakest-evidenced); RFC 8725 §3.10; ADR-0162; ADR-0179.
Cross-refs: `design/tenant-isolation.md`, `design/audit-evidence-emission.md`.
