---
id: ADR-9001
status: Accepted
bominal_source: no Bominal equivalent
---

# ADR-9001: Enforce filesystem ADR validation

## Frontmatter

| Field | Value |
| --- | --- |
| **id** | ADR-9001 |
| **title** | Enforce filesystem ADR validation |
| **status** | Accepted |
| **date** | 2026-07-24 |
| **supersedes** | - |
| **superseded_by** | - |
| **owner** | governance |
| **related** | - |
| **bominal_source** | no Bominal equivalent |

## Status
Accepted

## Context
The app reads ADR documents from the filesystem.

## Decision
Validate the filesystem fixture through the app boundary.

## Consequences

### Concrete file and crate changes
| Path / Crate | Change type | BNF v4.1 name | Layer |
| --- | --- | --- | --- |
| `tools/oya-governance-adr-shape-app/` | update | `oya-governance-adr-shape-app` | app |

### Integration via Workflow + Ontology
Not applicable; the integration point is documented in the affected service PRD.

### Positive
- Detects runner regressions.

### Negative
- Adds one fixture.

### Operational
- Runs through Buck2.

## Clean Architecture Impact
| Lane | Impact | Action required |
| --- | --- | --- |
| `dependency-direction` | Not affected | none |
| `cross-product-refusal` | Not affected | none |
| `port-location` | Not affected | none |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Affected | app composition updated |
| `sdk-kernel-only` | Not affected | none |

## Alternatives Considered

### Alternative 1 — Test only the kernel
- Description: Omit filesystem validation.
- Pros: Fewer files.
- Cons: Runner discovery can regress silently.
- Reason rejected: The app boundary needs a real filesystem fixture.

## References
- ADR-0056.
