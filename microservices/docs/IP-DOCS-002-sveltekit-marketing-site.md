# IP-DOCS-002 — SvelteKit marketing + public-docs site

> ADR anchor: ADR-0203, ADR-0185.
> Owner: `oya-docs`.
> Estimate: 8 days.

## Goal

Build the Tier-3 (public-facing) docs surface on SvelteKit.
Source: the Markdown content bundle emitted from the same
files mdbook + TechDocs consume.

## Why this IP

External customers + prospects + search engines consume a
different shape of docs than internal engineers. SvelteKit is
the Phase-1 client stack per ADR-0185 — using it here keeps
the substrate aligned.

## Tasks

### 1. Content bundle generator

- A generator script reads `docs/` + selected
  `microservices/<ms>/` Markdown.
- Emits a per-page JSON bundle the SvelteKit site loads.
- Generator runs at CI time; the bundle ships as a release
  artifact.

### 2. SvelteKit app

- Standard SvelteKit project at
  `microservices/docs/sveltekit/`.
- Routes for marketing pages, onboarding tutorials, API
  reference (Redoc / Stoplight), and search.

### 3. Search

- Build-time search index (Lunr or similar) generated from
  the content bundle.
- Per-tier search index (don't conflate internal Tier-1 with
  external Tier-3).

### 4. Tests

- Build-time: `pnpm build` succeeds.
- Lighthouse score ≥ 95 on the home page.

## Acceptance criteria

- Public-facing site renders with the substrate's marketing
  copy + API reference.
- Same Markdown source backs all three tiers (no drift).

## References

- ADR-0203, ADR-0185.
- SvelteKit upstream.
