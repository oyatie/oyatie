---
adr_id: finops-portal-001
authored: 2026-05-18
status: accepted
authority_chain: ADR-0199 FinOps canonical
microservice: finops-portal
classification: internal
---

# ADR finops-portal-001 — Pin FOCUS spec version 1.3

## Context

The FOCUS spec (FinOps Open Cost & Usage Specification) is the
canonical cross-cloud cost schema. ADR-0199 D-4 named FOCUS as the
export schema but did not pin a specific version. `finops-portal`
ships an export pipeline (IP-014) that must encode against a
specific version because the schema fields differ subtly between
versions.

## Decision

Pin FOCUS spec to version **1.3** for the IP-014 export pipeline.

## Rationale

1. FOCUS 1.3 was released 2025-11; it is the current stable.
2. 1.3 introduces the `EffectiveCost` column the chargeback
   formula uses to express credit-applied amounts.
3. 1.3 disambiguates `ServiceCategory` from `ServiceName`,
   matching our `cost_center → ServiceCategory` mapping.
4. CloudHealth, Apptio, Vantage have all shipped 1.3 support; we
   land on parity rather than 1.2 (one release behind).

## Consequences

- `oya-finops-portal-focus-export-kernel` declares `schema_version
  = "1.3"` in the manifest.
- Tenants downloading FOCUS data receive 1.3-conformant files.
- A future bump to FOCUS 1.4 is gated by:
  - A new ADR.
  - A new translator alongside the 1.3 translator (version-aware
    dispatch).
  - A 6-month deprecation window for 1.3 export endpoints.

## Alternatives considered

- **FOCUS 1.2**: still in use by some legacy consumers. Rejected
  because new tenants expect 1.3; 1.2 lacks `EffectiveCost`.
- **Internal proprietary schema**: rejected per ADR-0199 explicit
  preference for FOCUS canonical.

## References

- ADR-0199 FinOps canonical.
- IP-014 FOCUS export pipeline.
- https://focus.finops.org/focus-specification/v1.3/
