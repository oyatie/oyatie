---
doc_class: AutomationIndex
shape: anchor
length_cap: 80
authority_tier: 2
status: pending approval
purpose: |
  Catalogue every auto-doc-generation pipeline + architecture-visualization spec
  under `.omc/automation/`. Anchor for MASTERPLAN Directives 10 and 11. Every
  row names the pipeline id, inputs, outputs, trigger, fitness lane, and the
  mdbook chapter where renders publish.
lift_target: oyatie/docs/automation/INDEX.md
enforced_by: oya-foundry-fitness-doc-freshness
companion_docs:
  - ../plans/MASTERPLAN.md
  - ../../docs/DOC-CATALOG.md
  - ../../docs/AGENTS.md
---

# Oyatie Automation Pipeline Catalogue

> **Status:** pending approval. **Lift target:** `oyatie/docs/automation/INDEX.md`. **Owner:** axis-foundry + council-architecture. **Date:** 2026-05-12.

## 1. Auto-generation pipelines (Directive 10)

| Pipeline id | Inputs | Outputs | Trigger | Fitness lane | mdbook chapter |
|---|---|---|---|---|---|
| `rustdoc` | every workspace `///` comment | per-crate API ref + cross-crate link graph | nightly + per-PR delta | `oya-foundry-fitness-rustdoc-publish` | `/api/rust/<crate>` |
| `openapi` | `contracts/openapi/*.yaml` (3.1) | Redoc + Swagger UI + runtime/schema cross-validation | per-PR + nightly | `oya-foundry-fitness-openapi-publish` | `/api/openapi/<surface>` |
| `adr-index` | `docs/decisions/*.md` frontmatter | `docs/ADR-INDEX.md` | pre-commit + per-PR | `oya-foundry-fitness-adr-index` | `/decisions/INDEX` |
| `runbook-freshness` | `docs/runbooks/**/*.md` `last_verified:` | freshness report + auto-PR | nightly | `oya-foundry-fitness-runbook-freshness` (extant) | `/operations/runbook-health` |
| `fitness-lane-reports` | per-lane JSON emitted by CI | rolled-up mdbook chapter per axis | on-merge | `oya-foundry-fitness-lane-rollup` | `/fitness/<axis>` |
| `schema-doc` | kernel struct `data_class:` + field doc-comments | data-class catalogue | per-PR + nightly | `oya-foundry-fitness-data-class` (extant) | `/data/data-class-catalogue` |
| `changelog` | merged PR body + commit metadata | `docs/CHANGELOG.md` row | on-merge | `oya-foundry-fitness-changelog-row` | `/operations/changelog` |
| `glossary` | source `/// glossary: <term>` blocks | `docs/GLOSSARY.md` + retirement enforcement | per-PR + nightly | `oya-foundry-fitness-glossary` | `/reference/glossary` |

## 2. Architecture-visualization specs (Directive 11)

| Spec id | Source | Render | Trigger | Fitness lane | mdbook chapter |
|---|---|---|---|---|---|
| `architecture-map-kernel` | workspace + contracts + docs/products | Mermaid + D2 + Graphviz | per-PR + nightly | `oya-foundry-fitness-architecture-map-freshness` | `/visualization/architecture` |
| `product-map` | 7 axes × N products with status/owner/wave | Mermaid | nightly | `oya-foundry-fitness-product-map` | `/visualization/product-map` |
| `service-map` | every crate + dep + public API (layered DAG) | D2 + SVG | per-PR | `oya-foundry-fitness-service-map` | `/visualization/service-map` |
| `tech-stack-map` | LTS-pinned external deps + adapter location | Mermaid | nightly | `oya-foundry-fitness-tech-stack-map` | `/visualization/tech-stack` |
| `roadmap-visualization` | MASTERPLAN + milestone INDEXes | Mermaid Gantt | per-PR (masterplan touch) + nightly | `oya-foundry-fitness-roadmap-viz` | `/visualization/roadmap` |
| `dependency-graph` | subplan/phase/IP DAG | Graphviz dot | per-PR + nightly | `oya-foundry-fitness-dep-graph` | `/visualization/dependency-graph` |
| `audit-chain-map` | every EVT-* topic + emitter + consumer | D2 + Mermaid | nightly | `oya-foundry-fitness-audit-chain-map` | `/visualization/audit-chain` |

## 3. Discipline specs (top-level)

| Discipline spec | Output artifact | Fitness lane | Severity ladder |
|---|---|---|---|
| `doc-freshness-discipline` | per-doc-class staleness PR auto-gen | `oya-foundry-fitness-doc-freshness` | BLOCKER Const/Op, HIGH Ref, advisory Working |
| `orphan-detection-discipline` | orphan report per PR | `oya-foundry-fitness-orphan-detection` | HIGH |
| `cross-reference-index-spec` | `docs/CROSS-REFERENCE-INDEX.md` | `oya-foundry-fitness-cross-reference-index` | HIGH |

## 4. Pipeline-to-pipeline dependencies (top edges)

- `adr-index` → `cross-reference-index` (ADR rows feed the consolidated index)
- `rustdoc` + `openapi` + `schema-doc` → `architecture-map-kernel` (kernel walks generated artifacts to attribute API surfaces)
- `runbook-freshness` + `doc-freshness-discipline` → `changelog` (freshness PRs auto-emit CHANGELOG rows via the shared lane)

## 5. Authority + governance

Every spec carries `status: pending approval`, `lift_target:`, `purpose:`, `enforced_by:`. No MVP variants. Specs ≤150 lines; visualization specs ≤200. Cite existing oyatie crates by exact name when extending. Hyperscaler bar: Mermaid native + D2 richer renders + Graphviz where SVG fidelity matters.
