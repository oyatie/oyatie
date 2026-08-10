---
doc_class: ADRIndex
microservice: sites
date: 2026-05-17
owner_team: axis-sites + council-privacy
doc_status: published
---

# sites µservice — service-scoped ADRs

This directory holds ADRs that govern the `sites` µservice exclusively,
per the per-microservice flat layout in ADR-0131. Cross-cutting ADRs
that govern multiple µservices remain at `docs/decisions/` at the
repo root.

Each ADR closes one Open Question (or derived gap) surfaced in
`microservices/sites/PRD.md`, in
`microservices/sites/PHASE-01-SITES-FOUNDATION.md`, or in a
capability / runbook / threat-model / DPIA artifact under
`microservices/sites/`.

## Index

| ID | Title | Status | Date | Closes |
|---|---|---|---|---|
| [ADR-SITES-0001](./ADR-SITES-0001-crdt-library-selection.md) | CRDT library selection — Loro 1.x aligned with docs/sheets/slides/workflow-studio | Accepted | 2026-05-17 | PRD §"Bounded Contexts" → block BC; AC-10 |
| [ADR-SITES-0002](./ADR-SITES-0002-static-vs-dynamic-rendering-strategy.md) | Rendering strategy — SSG/ISR hybrid (vs full SSR vs pure SSG) | Accepted | 2026-05-17 | PRD §"Performance" (page-render p95 ≤ 200ms) + PRD §"Bounded Contexts" → cdn-delivery BC |
| [ADR-SITES-0003](./ADR-SITES-0003-cdn-substrate-and-cache-strategy.md) | CDN substrate + cache strategy — Cloudflare-class primary; self-managed Varnish/Caddy fallback | Accepted | 2026-05-17 | PRD §"Performance" + Hyrum #5 (cache-key version-hash) + runbook `cdn-cache-purge-cascade.md` |
| [ADR-SITES-0004](./ADR-SITES-0004-acme-and-custom-domain-flow.md) | ACME RFC 8555 + custom-domain flow — Let's Encrypt DNS-01 primary; HTTP-01 fallback; multi-account pool | Accepted | 2026-05-17 | PRD §FR-06 + AC-03 + Hyrum #4 + runbook `acme-cert-renewal-failure.md` |
| [ADR-SITES-0005](./ADR-SITES-0005-cms-collection-data-model.md) | CMS-collection data model — hybrid portable-text + relational | Accepted | 2026-05-17 | PRD §FR-11 + AC-04 + PRD Open Question 4 |
| [ADR-SITES-0006](./ADR-SITES-0006-ai-page-build-bounds.md) | AI-page-build EU AI Act bounds — T2 with HR/legal/medical refusal | Accepted | 2026-05-17 | PRD §FR-22 + AC-13 + capabilities/T2-auto.yaml |
| [ADR-SITES-0007](./ADR-SITES-0007-image-and-asset-pipeline.md) | Image + asset pipeline — libvips streaming; WebP/AVIF/JPEG-XL output | Accepted | 2026-05-17 | PRD §FR-16 + Performance (image-optimize p95 ≤ 1s) + runbook `asset-optimization-degraded.md` |

## Authoring conventions

- ADR ID format: `ADR-SITES-XXXX` (4-digit, scope-prefixed) per
  ADR-0131 service-scoped-ADR convention.
- Each ADR carries: Status, Date (ISO yyyy-mm-dd), Context, Decision,
  Alternatives Considered (≥3 per decision; each with Pros/Cons/
  Rejected reason), Consequences (≥3 downstream impacts), Verification,
  References.
- Service-scoped ADRs may reference cross-cutting ADRs (`ADR-####` at
  repo root) and sibling µservice ADRs. Cross-µservice citations
  encouraged where decisions are genuinely paired (e.g., ADR-SITES-0001
  ↔ ADR-WS-0001 + ADR-DOCS-0001 + ADR-SHEETS-0001 + ADR-SLIDES-0001 for
  the Loro CRDT alignment).
- Lifecycle per ADR-0131 §"ADR Lifecycle": `Proposed → Accepted →
  (Superseded by ADR-SITES-NNNN | Deprecated)`. Never delete;
  supersede.

## Open questions not yet closed

| PRD Open Question | Status | Notes |
|---|---|---|
| #1 (Webflow-class visual layout designer) | Open | Post-M03 decision; pairs with sheets visual editor |
| #2 (AMP-stub emission) | Open | Awaiting Google AMP futures clarity; defer |
| #3 (WordPress import path) | Open | Post-M04; pairs with `migration-from-wordpress.md` authoring |
| HR/legal/medical T2 conformity assessment | Open | Awaits ADR-SITES-XXXX EU AI Act Annex III §3 conformity; until then, REFUSED at Cedar |

These remain in `microservices/sites/PRD.md` §"Open Questions"; future
ADRs land here with sequential IDs.

## References

- ADR-0131 (per-microservice flat layout + service-scoped ADR convention).
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
- `microservices/docs/decisions/README.md` — sibling µservice ADR index pattern.
- `microservices/calendar/decisions/README.md` — sibling µservice ADR index pattern.
