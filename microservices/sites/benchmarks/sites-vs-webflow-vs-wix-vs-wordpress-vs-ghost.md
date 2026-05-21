---
doc_class: Benchmark
microservice: sites
benchmark_date: 2026-05-20
related_adrs: [ADR-SITES-0002, ADR-SITES-0003, ADR-0316]
doc_status: published
---

# Benchmarks — oyatie sites vs Webflow / Wix Business / WordPress.com Business / Squarespace / Ghost / Carrd / Framer

Workloads measured: (a) page-render latency (SSG cached), (b) page-render latency (ISR cold), (c) static-asset latency (CDN warm), (d) site-build-and-publish duration for 100-page site, (e) WCAG 2.2 AA pass-rate at default-publish, (f) annual TCO at 50-page commercial site + 100 k visitors/month.

Hardware (oyatie paid): 16× Postgres + 24× CDN edge + 8× Meilisearch + 6× AI-runtime workers across 3 regions.

## Workload (a) — page-render latency (SSG cached)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie sites paid | 28 | 84 |
| oyatie sites paid | 18 | 56 |
| Webflow (their CDN; SSG mode) | ~ 60 (published) | ~ 180 |
| Wix Business | ~ 120 | ~ 320 |
| WordPress.com Business | ~ 140 | ~ 380 |
| Squarespace | ~ 80 | ~ 240 |
| Ghost (commercial host) | ~ 50 | ~ 160 |
| Carrd | ~ 40 | ~ 120 |
| Framer | ~ 50 | ~ 160 |
| Hugo + Netlify (DIY SSG + commercial CDN) | ~ 30 | ~ 80 |

Reading: oyatie paid ties with Hugo+Netlify (the best-in-class DIY SSG host). We beat Webflow + Ghost by ~ 3×; we beat Wix + WordPress + Squarespace by ~ 5×.

PRD target: ≤ 200 ms p95; paid hits 56 ms p99.

## Workload (b) — page-render latency (ISR cold; first request after revalidation)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie sites paid | 180 | 420 |
| Webflow (no native ISR; equivalent is dynamic) | ~ 380 | ~ 920 |
| Wix Business (no native ISR) | ~ 480 | ~ 1 200 |
| WordPress.com Business + caching plugin | ~ 320 | ~ 780 |
| Next.js + Vercel ISR | ~ 80 | ~ 220 |
| Astro + Netlify | ~ 120 | ~ 380 |

Reading: Next.js + Vercel ISR is the leader (a tightly-coupled framework + host). oyatie paid is competitive with Astro + Netlify; ahead of Webflow/Wix/WordPress.

## Workload (c) — static-asset latency (CDN warm)

| Platform | p50 (ms) | p99 (ms) |
|---|---:|---:|
| oyatie sites paid (CDN edge) | 22 | 72 |
| Webflow (their CDN) | ~ 40 | ~ 120 |
| Wix CDN | ~ 60 | ~ 160 |
| Cloudflare-fronted custom hosting | ~ 18 | ~ 60 |
| Netlify (Edge CDN) | ~ 25 | ~ 80 |

PRD target: ≤ 100 ms p95; paid hits 72 ms p99.

## Workload (d) — site-build-and-publish duration for 100-page site

| Platform | p50 (s) | p99 (s) |
|---|---:|---:|
| oyatie sites (parallel page build + cache invalidate) | 18 | 38 |
| Webflow publish | ~ 12 (their managed publish) | ~ 24 |
| WordPress.com publish | ~ 4 (no full build; per-page) | ~ 12 |
| Next.js + Vercel deploy | ~ 90 | ~ 220 |
| Hugo + Netlify | ~ 24 | ~ 60 |
| Astro + Netlify | ~ 48 | ~ 120 |

Reading: WordPress.com is fastest because they DON'T do a full build per publish (each page is independent). Webflow is fastest of the SSG-based platforms. oyatie is competitive.

## Workload (e) — WCAG 2.2 AA pass-rate at default-publish (out-of-box themes / templates)

| Platform | Pass-rate % (axe-core scan; pure default theme) |
|---|---:|
| oyatie sites (publish-gate enforces) | 100 |
| Webflow default themes | ~ 78 |
| Wix default templates | ~ 62 |
| WordPress.com default themes | ~ 84 |
| Squarespace default templates | ~ 88 |
| Ghost default themes | ~ 92 |
| Framer default templates | ~ 70 |

Reading: oyatie is the only platform that REFUSES to publish failing pages. Others provide warnings; tenants can publish anyway. Our publish-rate at 100 % AA is by design.

## Workload (f) — annual TCO at 50-page commercial site + 100k visitors/month

| Platform | Per-site (USD/year) | Notes |
|---|---:|---|
| oyatie sites (per-cell flat; site is 1 of N in the cell) | ~ 1 200 (allocated cell cost) | Cell cost amortised across sites |
| Webflow Business CMS | 432 | Includes hosting + CMS |
| Wix Business Unlimited | 348 | |
| WordPress.com Business | 300 | |
| Squarespace Business | 360 | |
| Ghost (managed) | 350 | Best-in-class for newsletter-style sites |
| Carrd Pro Plus | 49 | Single-page-site focused |
| Framer Mini | 240 | |
| Netlify Business + WordPress + Cloudflare (DIY) | ~ 720 | Includes hosting, plugins, ops |

Reading: at single-site TCO, the managed SaaS platforms (Wix, WordPress.com, Squarespace) are cheapest. oyatie's advantage is at multi-site scale — once a tenant has 10+ sites in a cell, the per-site cost drops to ~ $120 with no tenant-level overhead.

## Reproducibility

Benchmark harness at `benchmarks/sitesbench/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks sites \
    --workload ssg-page-render \
    --tier oyatie-paid \
    --output ./benchmark-results.json
```
