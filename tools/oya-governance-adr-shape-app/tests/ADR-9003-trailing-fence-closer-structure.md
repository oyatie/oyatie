# ADR-9003: Reject trailing fence closer structure

```md
placeholder
```still-open
## Frontmatter
| Field | Value |
| --- | --- |
| **id** | ADR-9003 |
| **title** | Reject trailing fence closer structure |
| **status** | Accepted |
| **date** | 2026-07-24 |
| **supersedes** | - |
| **superseded_by** | - |
| **owner** | crew |
| **related** | - |
| **bominal_source** | no Bominal equivalent |

## Context
A fake complete ADR must remain fenced.

## Decision
Accept only a closing fence with whitespace after its marker run.

## Consequences
### Concrete file and crate changes
### Integration via Workflow + Ontology
### Positive
### Negative
### Operational

## Clean Architecture Impact
| Lane | Impact | Action required |
| --- | --- | --- |
| `dependency-direction` | Not affected | none |
| `cross-product-refusal` | Not affected | none |
| `port-location` | Not affected | none |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |

## Alternatives Considered
### Alternative 1 — Treat trailing text as a closer
- Description: A.
- Pros: B.
- Cons: C.
- Reason rejected: D.

## References
- ADR-0056.
