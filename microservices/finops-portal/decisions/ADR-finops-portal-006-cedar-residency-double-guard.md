---
adr_id: finops-portal-006
authored: 2026-05-18
status: accepted
authority_chain: ADR-0183 + ADR-0008
microservice: finops-portal
---

# ADR finops-portal-006 — Cedar residency double-guard

## Context

`finops-portal` enforces residency at multiple layers
(NetworkPolicy, image registry, nodeSelector, helm overlays).
Cedar authz adds a logical-layer guard. The risk is that a single
mis-configuration leaks data across residency regions.

The defense-in-depth principle says: any single missed control
should not cause a leak. ADR-0008 (data-use-boundary) requires
defense-in-depth.

## Decision

Every Cedar policy that involves a residency-scoped resource
carries **two** guards:

1. A primary `permit` clause guarded by `principal.residency_region
   == resource.residency_region`.
2. A defensive `forbid` clause that explicitly denies when the
   residency regions differ.

The two-guard pattern is implemented in:

- `regulator-evidence-emit.cedar`: forbid clauses on cross-pack
  AND cross-region.
- `tenant-isolation.cedar`: implicit deny-by-default + explicit
  forbid for PHI without phi_authorized claim.

## Rationale

Cedar evaluates `forbid` after `permit`; an explicit forbid
trumps a leaky permit. So even if a permit is mis-authored, the
forbid catches the cross-region case.

## Consequences

- Slightly more verbose Cedar policies (1 forbid per residency-
  scoped permit).
- Unit tests cover both layers (the permit alone would not block
  the leak; the forbid does).
- Authoring discipline: every new permit on a residency-scoped
  resource MUST be paired with a forbid.

## Alternatives considered

- **Single permit**: rejected because of defense-in-depth.
- **Schema-level enforcement**: Cedar schema doesn't express
  field-equality constraints; can't replace runtime checks.

## References

- ADR-0183 cedar-policy-discipline.
- ADR-0008 data-use-boundary.
- IP-007.
- `policy/cedar/regulator-evidence-emit.cedar`.
- `threat-model.md`.
