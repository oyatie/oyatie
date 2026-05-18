# IP-DOCS-001 — mdbook + Backstage TechDocs pipeline

> ADR anchor: ADR-0203, ADR-0170.
> Owner: `oya-docs`.
> Estimate: 4 days.

## Goal

Build the Tier-1 (mdbook) and Tier-2 (Backstage TechDocs)
rendering pipelines so internal engineers can browse the same
Markdown source through two surfaces.

## Why this IP

Per ADR-0203 three tiers serve three audiences. Tiers 1 + 2
share the Markdown source-of-truth; only Tier 3 (SvelteKit)
needs a separate content bundle. This IP wires the shared
pipeline.

## Tasks

### 1. mdbook source root

- `docs/standards/`, `docs/operators/`, `docs/decisions/` are
  the canonical mdbook source roots.
- `book.toml` declares the chapters + theme.

### 2. mdbook build

- CI job `docs.mdbook.render` runs `mdbook build` on every
  PR; failure blocks merge.
- Render published to internal docs bucket on `dev`
  promotion.

### 3. Backstage TechDocs

- Per-µservice `mkdocs.yml` declares the Markdown sources for
  that service's catalog entry.
- `techdocs-cli generate` runs as part of the cluster Helm
  release for the Backstage µservice.

### 4. Tests

- `mdbook build` returns 0 on a sample chapter set.
- Sample µservice TechDocs renders end-to-end.

## Acceptance criteria

- Every standards / decisions doc renders in mdbook.
- Every µservice docs surface renders in TechDocs.

## References

- ADR-0203, ADR-0170.
- mdbook upstream.
- Backstage TechDocs upstream.
