---
doc_status: published
---

# ADR supersession template

> Per [`docs/STANDARDS-AND-TEMPLATES.md`](../STANDARDS-AND-TEMPLATES.md) §2. Used when authoring an ADR that supersedes another. Validated by `adr-supersession-graph` (the graph of `Supersedes:` links must remain a DAG).

## ADR-####: <Title>

> **Status:** Proposed | Accepted
> **Supersedes:** ADR-MMMM (link)
> **Superseded-by:** -
> **Owner:** <team>
> **Date:** <YYYY-MM-DD>
> **Related:** <list>

## Context

What changed since ADR-MMMM. What invariant from ADR-MMMM is preserved. What invariant from ADR-MMMM is *broken* (and why the break is justified).

## Decision

The new decision in full. Cite ADR-MMMM only via the `Supersedes:` header; do NOT inline the old decision.

## Migration path

For consumers of ADR-MMMM:
- What changes for them
- Migration window (deprecation horizon per ADR-0037 stability tier)
- Compatibility shim (if any) and shim sunset date

## Consequences

### Positive
- ...

### Negative
- ...

### Operational
- Runbook(s) updated: `<list>`
- CI lane(s) updated: `<list>`

## Sources
- ADR-MMMM (the superseded ADR)
- [`ADR-INDEX.md`](../ADR-INDEX.md)
- [`ADR-LEGACY-REGRESSION-MAPPING.md`](../ADR-LEGACY-REGRESSION-MAPPING.md) (if applicable)
