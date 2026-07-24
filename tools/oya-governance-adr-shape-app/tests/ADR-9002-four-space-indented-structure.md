# ADR-9002: Reject indented pseudo structure

    ## Frontmatter
    | Field | Value |
    | --- | --- |
    | **id** | ADR-9002 |
    | **title** | Reject indented pseudo structure |
    | **status** | Accepted |
    | **date** | 2026-07-24 |
    | **supersedes** | - |
    | **superseded_by** | - |
    | **owner** | crew |
    | **related** | - |
    | **bominal_source** | no Bominal equivalent |

    ## Context
    A code block must not satisfy the ADR structure.

    ## Decision
    Keep structural Markdown outside indented code.

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
    ### Alternative 1 — Pretend indentation is structure
    - Description: A.
    - Pros: B.
    - Cons: C.
    - Reason rejected: D.

    ## References
    - ADR-0056.
