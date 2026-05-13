---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P09
title: Visualization-as-Code (Foundry-Owned Architecture / Product / Service / Tech-Stack Maps)
status: stub
purpose: Auto-generate architecture, product, service, tech-stack, roadmap, dependency-graph diagrams from canonical sources; publish via mdbook; CI lane catches drift.
---

# M-CC-P09 — Visualization-as-Code

## Purpose
Per MASTERPLAN §2 Directive 11. Hand-drawn diagrams age out of sync; the sustainable form is generated-from-truth. Foundry owns the visualization kernel.

## Acceptance
- `crates/oya-foundry-architecture-map-kernel` walks Cargo workspace + `contracts/` + `docs/products/` + `docs/ROADMAP.md` + `docs/ADR-INDEX.md` + milestone frontmatter; emits Mermaid + D2 + Graphviz.
- Generated outputs at `docs/site/architecture/`: product-map.svg, service-map.svg, architecture-diagram.svg, tech-stack-diagram.svg, roadmap.svg, dependency-graph.svg.
- `oya-foundry-fitness-architecture-map-freshness` lane CI-blocks PRs that change source-of-truth without regenerating visualizations.
- Renders publish via `oya-foundry-mdbook-kernel` (from M-CC-P02).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | `oya-foundry-architecture-map-kernel` source walkers (Cargo metadata, OpenAPI parser, frontmatter parser) | stub | [`IP-001-architecture-map-walkers.md`](IP-001-architecture-map-walkers.md) |
| IP-002 | Mermaid + D2 + Graphviz emitters | stub | [`IP-002-mermaid-d2-graphviz-emitters.md`](IP-002-mermaid-d2-graphviz-emitters.md) |
| IP-003 | mdbook publishing integration | stub | [`IP-003-mdbook-publish-integration.md`](IP-003-mdbook-publish-integration.md) |
| IP-004 | `oya-foundry-fitness-architecture-map-freshness` lane | stub | [`IP-004-architecture-map-freshness-lane.md`](IP-004-architecture-map-freshness-lane.md) |

## Estimated parallelism
4 agents.

## Symbols-touched
`crates/oya-foundry-architecture-map-{kernel,app}-*`, `crates/oya-foundry-fitness-architecture-map-freshness-kernel`, `docs/site/architecture/`.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P09 complete: visualization-as-code kernel + lane green; product/service/architecture/tech-stack/roadmap/dependency-graph all auto-generated and freshness-gated" -i critical -k "M-CC,P09,visualization-as-code,complete"
```
