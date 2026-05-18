---
doc_class: CompetitorParityMatrix
template_id: TPL-COMPETITOR-PARITY
microservice: sites
status: Accepted
date: 2026-05-17
owner_team: axis-sites + council-product
related_adrs: [ADR-0133, ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0005, ADR-SITES-0006, ADR-SITES-0007]
doc_status: published
---

# Competitor Parity Matrix — sites µservice

## Purpose

Bound the M03 GA scope against named competitor capabilities. Drives
SDK launch order, marketing posture, and ADR-0133 hyperscaler-bar
verification.

## Capability matrix

| Capability | Google Sites | WordPress.com | Squarespace | Wix | Webflow | Notion Sites | Carrd | Framer | Ghost | Hugo | Sanity | Strapi | Contentful | Sitecore | AEM | **oyatie sites (M03)** | **oyatie sites (M04+)** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Block editor | basic | Gutenberg | yes | yes | yes (visual) | yes | basic | yes (Figma-class) | yes | n/a | portable-text | yes | yes | yes | yes | **yes (CRDT)** | yes (visual CSS-grid) |
| Theme system | template | theme | theme | theme | custom | template | basic | custom | theme | theme | n/a | n/a | n/a | template | template | **CSS-in-rust + tokens** | + visual designer |
| Custom domain + TLS | yes | yes | yes | yes | yes | yes | yes | yes | yes | (self-host) | n/a | n/a | n/a | yes | yes | **yes (ACME RFC 8555 + wildcard)** | — |
| SEO (meta + OG + JSON-LD + sitemap + robots + canonical + hreflang) | partial | yes (plugin Yoast) | yes | yes | yes | partial | yes | yes | yes | yes | n/a | n/a | n/a | yes | yes | **yes (native)** | — |
| CMS collections | no | posts/pages | products+blog | basic | yes | databases | no | basic | posts | content-as-files | yes | yes | yes | yes | yes | **yes (hybrid; ADR-SITES-0005)** | + visual schema editor |
| Site-wide search | basic | yes | yes | yes | yes | basic | basic | basic | yes | (self-host) | yes (API) | yes | yes | yes | yes | **yes (Meilisearch)** | — |
| Forms | yes | yes (plugin) | yes | yes | yes | yes (basic) | yes | yes | yes | (self-host) | n/a | n/a | n/a | yes | yes | **yes (via `forms` µservice)** | — |
| E-commerce | basic | WooCommerce | yes | yes | yes | basic | yes | yes | yes (members) | (Snipcart self-host) | n/a | n/a | n/a | yes | yes | **stub (T-G fintech bridge)** | full storefront |
| Privacy-preserving analytics | (GA only) | (Jetpack / GA) | (built-in tracking + GA) | (built-in + GA) | (GA / Plausible plugin) | basic | basic | (GA / Plausible) | yes (own) | (none) | n/a | n/a | n/a | yes | yes | **yes (Plausible-class native)** | — |
| Accessibility (WCAG 2.2 AA) | partial | per-theme | per-theme | per-theme | per-template | per-template | per-template | per-template | per-theme | per-theme | n/a | n/a | n/a | yes | yes | **yes (refuse-publish at < 100%)** | — |
| AMP | yes (older) | plugin | no | no | no | no | no | no | yes | plugin | n/a | n/a | n/a | yes | yes | **deferred (Google deprecation)** | (depends on Google AMP future) |
| Social-share metadata | yes | yes | yes | yes | yes | yes | yes | yes | yes | yes | n/a | n/a | n/a | yes | yes | **yes (OG + Twitter Cards)** | — |
| Preview mode | yes | yes | yes | yes | yes | yes | yes | yes | yes | yes (build) | yes | yes | yes | yes | yes | **yes (signed-token URL)** | — |
| Versioning + rollback | yes | yes (plugin) | yes | yes | yes | yes | no | yes | yes | git | yes | yes | yes | yes | yes | **yes (per-page version)** | — |
| Multi-language + hreflang | yes | yes (plugin Polylang) | partial | yes | yes (plugin) | partial | no | yes | yes (plugin) | yes (plugin) | yes | yes | yes | yes | yes | **yes (native)** | — |
| Comments | basic | Disqus / native | yes | yes | (3rd party) | yes | no | (3rd party) | yes (members) | (Disqus) | n/a | n/a | n/a | yes | yes | **yes (via `community` µservice)** | — |
| CDN delivery | yes | yes | yes | yes | yes | yes | yes | yes | yes | (self-host) | API | API | API | yes | yes | **yes (per-pack edges)** | + visual edge functions |
| AI page-build | (Workspace Gemini) | (Jetpack AI) | (Squarespace AI) | (Wix Studio AI) | (Webflow AI) | (Notion AI) | no | (Framer AI) | no | no | (Sanity AI) | (Strapi AI plugin) | (Contentful AI) | (Sitecore CDP) | (AEM Sensei) | **yes (T2 EU-AI-Act-bounded)** | + T1 author-suggest blocks |
| Loro/CRDT collab | no (Google Docs-class) | no | no | no | no | (Notion proprietary) | no | (real-time cursors) | no | no | (real-time editing) | no | no | no | no | **yes (Loro 1.x cross-µservice aligned)** | + multi-cursor presence |
| WordPress import | (Google takeout / 3rd party) | yes | yes | yes | yes | (3rd party) | no | (3rd party) | yes | yes | yes | yes | yes | yes | yes | **deferred** | yes (M04+) |
| Self-hostable | no | (.org yes) | no | no | no (export only) | no | no | no | yes | yes | (cloud + self-host) | (cloud + self-host) | no | (vendor) | (vendor) | **(via tenant cell)** | — |
| eIDAS AdES (signed-Sites) | no | no | no | no | no | no | no | no | no | no | no | no | no | no | no | **yes (audit-chain Ed25519)** | — |
| EU DSA transparency report | (per DSA-VLOSE) | yes | yes | yes | yes | yes | (below threshold) | yes | yes | (self-host) | n/a | n/a | n/a | yes | yes | **yes (tenant-served report sitemap)** | — |

## Parity gaps to close at M03

1. **Webflow-class visual layout designer** — Webflow's CSS-grid + flexbox visual canvas is unique among hosted builders; deferred to M04+. Compensating control: portable-text + design-tokens already gives most of the customisation surface; visual canvas is editor-UX, not data-model.
2. **WordPress import path** — needed for migration tenants; deferred to M04+.
3. **AMP-HTML emission** — Google deprecation signals; we monitor + decide post-M04.
4. **E-commerce full storefront** — only a stub at M03; full integration through T-G fintech bridge (post-M04).

## Differentiators (none of the competitors offer)

1. **Loro CRDT alignment across `sites` + `docs` + `sheets` + `slides` + `workflow-studio`** — same engine across the entire collab suite per ADR-SITES-0001.
2. **Intranet + public site under one µservice** — Google Sites does intranet; everyone else does public; nobody does both with same authoring substrate per-tenant.
3. **CMS-collection + Ontology binding** — collections become first-class entities in the cross-µservice Ontology graph.
4. **Privacy-preserving Plausible-class analytics by default** — no third-party cookies; ePrivacy-conformant out of the box.
5. **EU AI Act conformity by construction** — T2 AI-page-build refuses HR/legal/medical context until conformity assessment (ADR-SITES-0006).
6. **eIDAS AdES via audit-chain** — published pages bear Ed25519 seals; signed-Sites use case unlocked.

## Pricing posture (vs competitors)

- WordPress.com Business: $25/mo — competes with our pro $99/mo (we charge premium for collab + ontology + privacy).
- Squarespace Business: $36/mo — competes with our pro.
- Webflow CMS: $29/mo — competes with our pro; CMS-collection differentiator clear.
- Sitecore: enterprise-priced — competes with our enterprise tier (per-quote).

## Audit cadence

- Quarterly: parity matrix review with council-product.
- Per-release: gap-closure verdict.
- Annual: pricing posture review.

## References

- ADR-0133 (industry best-practice + competitor benchmark).
- ADR-SITES-0001 (Loro); ADR-SITES-0002 (rendering); ADR-SITES-0005
  (CMS-collection); ADR-SITES-0006 (AI-page-build); ADR-SITES-0007
  (image pipeline).
- Competitor sources cited in PRD.md §"Competitive Benchmark".
- WordPress.org developer documentation.
- Webflow developer documentation.
- Notion API documentation.
- Framer developer documentation.
- Ghost API documentation.
- Hugo / 11ty / Astro / Gatsby documentation.
- Sanity / Strapi / Contentful / Storyblok documentation.
- Sitecore + AEM enterprise documentation.
