---
doc_class: AutomationIndex
shape: anchor
length_cap: 80
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Catalogue every auto-doc-generation pipeline + architecture-visualization spec
  under `docs/automation/`. Anchor for MASTERPLAN Directives 10 and 11. Every
  row names the pipeline id, inputs, outputs, trigger, fitness lane, and the
  mdbook chapter where renders publish.
planned_enforcement_ref: governance-doc-freshness
companion_docs:
  - ../../docs/MASTERPLAN.md
  - ../../docs/DOC-CATALOG.md
  - ../../docs/AGENTS.md
doc_status: published
---

# Oyatie Automation Pipeline Catalogue

> **Status:** Accepted catalogue. **Owner:** axis-foundry + council-architecture. **Date:** 2026-05-12. **ADRs:** ADR-0052, ADR-0053, ADR-0054. Lane names are planned/advisory unless the row explicitly says extant.

## 1. Planned auto-generation pipelines (Directive 10)

| Pipeline id | Inputs | Outputs | Target trigger | Planned/active fitness lane | mdbook chapter |
|---|---|---|---|---|---|
| `rustdoc` | every workspace `///` comment | per-crate API ref + cross-crate link graph | nightly + per-PR delta | `governance-rustdoc-publish` | `/api/rust/<crate>` |
| `openapi` | `contracts/openapi/*.yaml` (3.1) | Redoc + Swagger UI + runtime/schema cross-validation | per-PR + nightly | `governance-openapi-publish` | `/api/openapi/<surface>` |
| `adr-index` | `docs/decisions/*.md` frontmatter | `docs/ADR-INDEX.md` | pre-commit + per-PR | `governance-adr-index` | `/decisions/INDEX` |
| `runbook-freshness` | `docs/runbooks/**/*.md` `last_verified:` | freshness report + auto-PR | nightly | `governance-runbook-freshness` (extant) | `/operations/runbook-health` |
| `fitness-lane-reports` | per-lane JSON emitted by CI | rolled-up mdbook chapter per axis | on-merge | `governance-lane-rollup` | `/fitness/<axis>` |
| `schema-doc` | kernel struct `data_class:` + field doc-comments | data-class catalogue | per-PR + nightly | `governance-data-class` (extant) | `/data/data-class-catalogue` |
| `changelog` | merged PR body + commit metadata | `docs/CHANGELOG.md` row | on-merge | `governance-changelog-row` | `/operations/changelog` |
| `glossary` | source `/// glossary: <term>` blocks | `docs/GLOSSARY.md` + retirement enforcement | per-PR + nightly | `governance-glossary` | `/reference/glossary` |

## 2. Planned architecture-visualization specs (Directive 11)

| Spec id | Source | Render | Target trigger | Planned/active fitness lane | mdbook chapter |
|---|---|---|---|---|---|
| `architecture-map-kernel` | workspace + contracts + docs/products | Mermaid + D2 + Graphviz | per-PR + nightly | `governance-architecture-map-freshness` | `/visualization/architecture` |
| `product-map` | 7 axes × N products with status/owner/wave | Mermaid | nightly | `governance-product-map` | `/visualization/product-map` |
| `service-map` | every crate + dep + public API (layered DAG) | D2 + SVG | per-PR | `governance-service-map` | `/visualization/service-map` |
| `tech-stack-map` | LTS-pinned external deps + adapter location | Mermaid | nightly | `governance-tech-stack-map` | `/visualization/tech-stack` |
| `roadmap-visualization` | MASTERPLAN + milestone INDEXes | Mermaid Gantt | per-PR (masterplan touch) + nightly | `governance-roadmap-viz` | `/visualization/roadmap` |
| `dependency-graph` | subplan/phase/IP DAG | Graphviz dot | per-PR + nightly | `governance-dep-graph` | `/visualization/dependency-graph` |
| `audit-chain-map` | every EVT-* topic + emitter + consumer | D2 + Mermaid | nightly | `governance-audit-chain-map` | `/visualization/audit-chain` |

## 3. Discipline specs (top-level)

| Discipline spec | Output artifact | Planned/active fitness lane | Severity ladder |
|---|---|---|---|
| `doc-freshness-discipline` | per-doc-class staleness PR auto-gen | `governance-doc-freshness` | BLOCKER Const/Op, HIGH Ref, advisory Working |
| `orphan-detection-discipline` | orphan report per PR | `governance-orphan-detection` | HIGH |
| `cross-reference-index-spec` | `docs/CROSS-REFERENCE-INDEX.md` | `governance-cross-reference-index` | HIGH |

## 4. Pipeline-to-pipeline dependencies (top edges)

- `adr-index` → `cross-reference-index` (ADR rows feed the consolidated index)
- `rustdoc` + `openapi` + `schema-doc` → `architecture-map-kernel` (kernel walks generated artifacts to attribute API surfaces)
- `runbook-freshness` + `doc-freshness-discipline` → `changelog` (freshness PRs auto-emit CHANGELOG rows via the shared lane)

## 5. Authority + governance

Every spec carries `status: Accepted`, `date:`, `purpose:`, and either `enforced_by:` for active workflow + quality-lane controls or `planned_enforcement_ref:` for planned/advisory controls. No prototype variants. Specs ≤150 lines; visualization specs ≤200. Cite existing oyatie crates by exact name when extending. Hyperscaler bar: Mermaid native + D2 richer renders + Graphviz where SVG fidelity matters. Governing ADRs: ADR-0052 (artifact inventory), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim pattern).
