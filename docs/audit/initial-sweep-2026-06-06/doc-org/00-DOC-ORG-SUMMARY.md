# Documentation Organization — Executive Summary

The `/Users/jasonlee/Developer/source` documentation corpus comprises **10,069 markdown
documents** (the global census at `_census-all-docs.txt`), classified across seven area
surveys into a **two-facet taxonomy**: a Diátaxis *category* (reference 4,327 ·
explanation 3,608 · how-to 1,791 · tutorial 147) crossed with a subject *axis*
(spec/contract 2,446 · product 2,008 · operations/runbook 1,613 · research 1,507 ·
governance/compliance 903 · decision/ADR 737 · cloud-substrate 304 · architecture 273 ·
index/meta 78), and labelled with a closed 13-value *doc_type* vocabulary (spec, runbook,
research, reference, governance, adr, guide, architecture, index, readme, product-prd,
other, changelog). Every classified row is complete — zero blank doc_type/category/axis
cells — but doc_type is intentionally **many-to-many** with both facets (e.g. an `adr`
document can be either explanation or how-to; a `guide` spans explanation, how-to, and
tutorial), so the taxonomy is two orthogonal facets layered over doc_type rather than a
single 1:1 lookup. The seven area surveys account for **9,855 unique documents** (9,873
rows reported, including 18 documents intentionally double-listed across the overlapping
*docs-context-research* and *docs-rest* areas), leaving a reconciliation **gap of 214
survey-orphans** — census documents that fell into no area's scope, concentrated in
`tools/agent-skills/` (100), `docs/governance-lanes/` (65), `libs/oya-governance-*` test
fixtures and READMEs (19), `tests/cross-microservice/` (8), and `benchmarks/` (8), plus a
handful of loose top-level `docs/` files (ADR-INDEX, ADR-CONSOLIDATION-PLAN, raw/, audits/).
Action counts surfaced by the *status* column drive the cleanup backlog: 1,184 likely-stale,
129 Proposed (awaiting ratify/drop), 72 duplicate, 28 misplaced, 21 Superseded, 11 orphan,
1 Deprecated, 1 Amended, and 2 empty/none, with the remaining 8,224 marked current. The
**generated-index plan** is to treat the seven inventory TSVs as the single source of truth
and emit, per doc_type and per axis, a deterministic `INDEX.md` (Diátaxis-grouped) plus a
top-level `ADR-INDEX`-style master index regenerated from the TSVs on every sweep — first
folding the 214 orphans into an eighth "tools/libs/tests/benchmarks" survey (or explicitly
excluding fixtures), resolving the 18 cross-area duplicates to a canonical home, and
retiring the likely-stale/duplicate/superseded set before the index is published.
