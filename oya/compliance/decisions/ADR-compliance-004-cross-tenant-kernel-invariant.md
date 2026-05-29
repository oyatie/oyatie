---
id: ADR-compliance-004
status: Accepted
deciders: axis-compliance, axis-security, council-architecture
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0183, ADR-0209]
---

# ADR-compliance-004 — Cross-tenant isolation kernel invariant (5-layer guard)

## Decision

Cross-tenant DSAR / evidence assembly is a Sev-1 incident. Defense-in-depth via 5 layers:

1. **API guard** — handler asserts `request.principal.tenant_id == request.subject.tenant_id`.
2. **Domain guard** — Ontology projection walk filters by tenant_id at every step.
3. **Kernel guard** — `oya-shared-compliance-evidence-kernel::coverage_gaps` filters by tenant_id.
4. **Cedar guard** — capability requires `principal.tenant_id == resource.tenant_id`.
5. **Integration test** — `tests/cross_tenant_dsar.rs` asserts zero leakage.

## Consequences

Any single layer can fail without compromising isolation. Layer 5 catches drift in CI.
