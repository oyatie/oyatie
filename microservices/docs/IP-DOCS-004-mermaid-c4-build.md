# IP-DOCS-004 — Mermaid + C4-PlantUML build pipeline

> ADR anchor: ADR-0203.
> Owner: `oya-docs`.
> Estimate: 2 days.

## Goal

Render Mermaid + C4-PlantUML diagrams committed as source
during the docs build so all three tiers display them.

## Tasks

### 1. Mermaid

- Source diagrams committed as fenced `mermaid` blocks in
  Markdown.
- mdbook + TechDocs render natively.
- SvelteKit renders via the `mermaid` npm package at
  build-time → static SVG.

### 2. C4-PlantUML

- Source diagrams committed as `.puml` files.
- Pre-process at CI to SVG; SVG embedded into Markdown.

### 3. Tests

- Sample diagram in each format renders cleanly across all
  three tiers.

## Acceptance criteria

- All architectural diagrams in `docs/decisions/` render in
  mdbook + TechDocs + SvelteKit.

## References

- ADR-0203.
- Mermaid upstream.
- C4-PlantUML upstream.
