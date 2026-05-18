---
id: ADR-SITES-0002
status: Accepted
date: 2026-05-17
microservice: sites
deciders: axis-sites, council-architecture, ops-sre-reliability
owner: axis-sites + council-architecture
supersedes: []
superseded_by: []
related:
  - ADR-0056
  - ADR-0105
  - ADR-0131
  - ADR-0133
  - ADR-SITES-0003
related_artifacts:
  - microservices/sites/PRD.md §"Performance" (page-render p95 ≤ 200ms; static-asset p95 ≤ 100ms)
  - microservices/sites/IP-003-page-bc-kernel.md
  - microservices/sites/IP-011-cdn-delivery-and-pipeline.md
purpose: |
  Choose the rendering strategy for sites' published pages: full SSR
  (server renders on every request), full SSG (static rendered once at
  publish), or hybrid SSG/ISR (SSG with on-demand revalidation).
---

# ADR-SITES-0002: Rendering strategy — SSG/ISR hybrid; full SSR rejected; pure SSG rejected

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Sites' published pages have two operating envelopes:
1. **Static, immutable post-publish content** (blog posts, marketing pages, intranet pages, CMS-collection entries that don't change between publishes). Cached at CDN edge; ~10k RPS per tenant baseline.
2. **Dynamic-but-mostly-static content** (CMS-collection-driven listings, search-results pages, personalised intranet landing pages). Generally cacheable but may need on-demand revalidation when underlying CMS-collection changes.

Three rendering strategies are widely deployed:
- **Full SSR**: server renders HTML on every request. (WordPress.org with no caching; Drupal default; old Squarespace.)
- **Full SSG**: pages pre-rendered once at publish; served from CDN with no server compute on request. (Hugo, 11ty, Jekyll, Gatsby pre-Cloud, Astro static mode.)
- **SSG/ISR hybrid**: pages pre-rendered at publish (SSG), with on-demand revalidation when CMS-collection or block content changes (ISR — Incremental Static Regeneration). (Next.js SSG/ISR mode; Astro on-demand; Webflow.)

Per PRD-sites §"Performance" targets:
- page-render p95 ≤ 200ms (anonymous public-facing) — requires CDN-edge serving for the dominant path.
- publish p95 ≤ 5s for 100-page site — requires fast SSG; full SSR has no publish cost but lacks the cache.
- CMS-query p95 ≤ 150ms — fast Postgres reads; cacheable.

Per ADR-SITES-0003 (CDN substrate), the CDN edge is the primary
delivery surface. Per ADR-0133 (industry best-practice), the chosen
strategy must scale to 50M monthly visitors per cell (Q4 2026
projection).

## Decision

The sites µservice ships an **SSG/ISR hybrid** rendering strategy:
- **At publish time**: every page is fully pre-rendered (SSG); the
  resulting HTML + assets are uploaded to S3; CDN cache populated
  via signed purge of prior version.
- **At request time**: CDN edge serves the cached SSG output; cache
  TTL = 24h with `stale-while-revalidate=86400`.
- **On CMS-collection change** (e.g., new Article entry): the
  affected pages' SSG output is revalidated on-demand (ISR) — the
  next request triggers re-render at origin; cache is updated; older
  edge cache serves stale-while-revalidate until updated.
- **On AI-page-build T2 accept**: the page enters DRAFT state; no
  publish; no CDN cache update. Tenant must explicitly publish.

Concrete implementation:
- `oya-sites-page-usecase::publish_page` orchestrates the SSG render;
  emits `PagePublished` Workflow event; cdn-delivery worker invalidates
  CDN cache.
- `oya-sites-cdn-delivery-worker` handles ISR revalidation on
  `CmsEntryWritten` events (only for pages bound to the changed
  collection).
- Cache-key includes version-hash per ADR-SITES-0003.

## Alternatives Considered

### A. Full SSR (server renders on every request)

- **Pros**:
  - Always-fresh content; no staleness window.
  - Simpler mental model (every request → render → respond).
  - WordPress.org parity (some tenants migrate from this model).
- **Cons**:
  - Page-render p95 ≤ 200ms is impossible at scale without aggressive
    CDN caching → which would re-introduce SSG/ISR anyway.
  - Cost: every anonymous read consumes Postgres + Valkey + render compute.
    At 50k RPS per cell, this is ~$100k/mo extra compute cost.
  - Cold-start latency on first request to a rarely-viewed page.
- **Rejected** because it violates the p95 ≤ 200ms target at the
  cost envelope; CDN caching is the dominant performance lever.

### B. Pure SSG (no revalidation)

- **Pros**:
  - Simplest model; no on-demand revalidation logic.
  - Hugo / 11ty / Jekyll parity.
  - Maximally cacheable.
- **Cons**:
  - CMS-collection entry changes require a full site re-publish to
    propagate; for a 1000-page CMS-driven site, this is slow + lossy.
  - Cannot support draft preview without a separate render path.
  - Cannot personalise (intranet landing per-user).
- **Rejected** because CMS-collection-driven sites would re-publish
  too aggressively, and per-user intranet personalisation needs an
  on-demand path.

### C. Edge-rendering (Cloudflare Workers / Vercel Edge Functions)

- **Pros**:
  - Render at edge (geo-near visitor); no origin round-trip.
  - Tenant code can execute at edge.
- **Cons**:
  - Hard vendor lock-in to Cloudflare Workers / Vercel Edge.
  - Per ADR-SITES-0003, we maintain CDN substrate-portability — not
    binding compute to a single CDN provider.
  - Per ADR-0117, data-residency forbids tenant compute at arbitrary
    edge nodes (residency-aware compute requires per-pack edges only).
- **Rejected** because it conflicts with substrate-portability and
  per-pack residency.

### D. SSG/ISR hybrid  ← **CHOSEN**

- **Pros**:
  - Page-render p95 ≤ 200ms achievable via CDN edge cache.
  - Publish p95 ≤ 5s for 100-page site achievable (parallel SSG).
  - CMS-collection changes trigger targeted ISR (not full site
    re-publish).
  - Substrate-portable: ISR runs at the origin (our µservice plane),
    not at the CDN edge — no vendor lock-in.
  - WordPress / Webflow / Squarespace / Next.js parity for the
    hybrid model.
- **Cons**:
  - More complex than pure SSG; ISR adds origin-render path.
  - Cache invalidation logic must be correct (Hyrum #5 — version-hash
    cache key); see ADR-SITES-0003.
  - Stale-while-revalidate window means edge may serve old content for
    up to 24h post-publish — mitigated by signed CDN purge.
- **Accepted** as the production strategy.

## Consequences

### Positive

- **Performance targets met.** p95 ≤ 200ms reachable via CDN cache; publish p95 ≤ 5s reachable via parallel SSG.
- **Substrate-portability.** ISR runs at origin; CDN provider is replaceable.
- **CMS-collection efficiency.** Targeted ISR on entry changes; no full-site re-publish.
- **Preview-mode supported.** Draft pages render via ISR on signed-token URL; no separate SSR path.

### Negative

- **Cache invalidation correctness is critical.** Per ADR-SITES-0003 cache-key version-hash format. Hyrum #5 in migration guide.
- **ISR origin compute on every CMS-collection change.** At scale, can be expensive; mitigated by debouncing within publish-pipeline-worker.
- **Stale-while-revalidate window** = visitors may see ≤ 24h-stale content after a publish if signed-purge didn't propagate to their edge. Mitigated by signed purge p95 ≤ 2s + monitoring per `runbooks/cdn-cache-purge-cascade.md`.

### Operational

- **publish-pipeline-worker** orchestrates SSG render.
- **isr-worker** handles on-demand revalidation triggered by
  `CmsEntryWritten` / `BlockUpdated` events.
- **Signed CDN purge** triggers on publish.

### Regulatory

- **GDPR Art. 17 erasure**: erasing a page triggers full cache purge across all edges; ISR refuses subsequent re-render of erased page.
- **EU DSA Art. 14 transparency**: publish-refusal records served via tenant transparency-report page; SSG-rendered.
- **WCAG 2.2 AA**: SSG output validated at publish time; refused if < 100% conformance.

## Verification

- [ ] **Page-render p95 ≤ 200ms** —
  `cargo bench -p oya-sites-cdn-delivery-adapter-s3 -- page_render`.
- [ ] **Publish (100-page site) p95 ≤ 5s** —
  `cargo bench -p oya-sites-cdn-delivery-usecase -- publish_100_pages`.
- [ ] **ISR revalidation on CmsEntryWritten** —
  `cargo nextest run -p oya-sites-cdn-delivery-worker -- isr_revalidate`.

## References

- ADR-0056, ADR-0105, ADR-0131, ADR-0133, ADR-SITES-0003.
- Next.js ISR documentation — `nextjs.org/docs/pages/building-your-application/data-fetching/incremental-static-regeneration`.
- Astro static mode — `docs.astro.build`.
- Webflow rendering — `developers.webflow.com`.
- HTTP `stale-while-revalidate` per RFC 5861.
- Google SRE Workbook ch. 18 (load balancing).
- `microservices/sites/PRD.md` §"Performance".
- `microservices/sites/IP-003-page-bc-kernel.md`.
- `microservices/sites/IP-011-cdn-delivery-and-pipeline.md`.
