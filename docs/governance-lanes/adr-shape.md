---
doc_status: published
---

# Fitness Lane: adr-shape

- status: Accepted
- date: 2026-05-12
- purpose: Fail closed when an ADR departs from the canonical shape in `docs/templates/adr-template.md`.
- enforces: `docs/templates/adr-template.md`; this lane does not grant lifecycle or merge authority.
- kernel_crate: `oya-governance-adr-shape-kernel` is pure parser/IR validation. The app owns filesystem discovery.
- runner_path: `tools/oya-governance-adr-shape-app`
- inputs: `docs/decisions/ADR-*.md`; explicit paths are supported for integration fixtures.
- ci_invocation: `buck2 test //libs/oya-governance-adr-shape-kernel:oya-governance-adr-shape-kernel-unittest` and `buck2 run //tools/oya-governance-adr-shape-app:oya-governance-adr-shape-app`.
- runtime_budget: 250 ms
- severity: diagnostic/non-admissible until a separately accepted migration and admission decision exists

## Enforced contract

The canonical template asks authors for an imperative present-tense title, but it does not define a controlled verb vocabulary. Imperative-title semantics therefore remain reviewer-only; this lane does not invent a lexical acceptance list. The parser structurally validates the required `## Frontmatter` table and its nine nonempty fields. It preserves the exact legacy lifecycle tokens already present in the ADR corpus (`accepted`, `proposed`, and `Accepted (amendment)`) alongside ADR-0388's canonical forms.

Each ADR MUST declare its Bominal inheritance/override position through the non-empty `bominal_source` row in the required table. It MUST contain these headings, in the declared narrative order where applicable:

- `Context`, `Decision`, `Consequences`, `Alternatives Considered`, and `References`;
- under consequences: `Concrete file and crate changes`, `Integration via Workflow + Ontology`, `Positive`, `Negative`, and `Operational`;
- `Clean Architecture Impact`, with all six template lanes: `dependency-direction`, `cross-product-refusal`, `port-location`, `layer-correctness`, `composition-root-only`, and `sdk-kernel-only`.

Every alternative item MUST provide `Description`, `Pros`, `Cons`, and `Reason rejected`. The parser ignores fenced code, checks heading level/range nesting, and emits a sorted `path`, keyed code, and message for every missing or malformed contract surface.

`docs/templates/adr-template.md` is a scaffold, not a concrete ADR: its literal status-choice placeholder is intentionally reported as `ADR_STATUS_INVALID`. A concrete ADR fixture with all nine table rows and a single lifecycle status must be accepted by both strict validation and this diagnostic audit.

## Test boundary

The kernel target contains only string-to-IR unit fixtures. Filesystem discovery and live-corpus execution remain in the app boundary. A corpus failure is a migration report: do not edit ADR corpus records, index/projection faces, or generated files in this lane to force a green result.
