---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-sites
microservice: sites
status: Accepted
sales_segment: shared-substrate + suite-app
tier: tenant-facing
milestone_first_ship: M03-connect-dissolution
bominal_source: []
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0140 (retired per ADR-0145), ADR-SITES-0001, ADR-SITES-0002, ADR-SITES-0003, ADR-SITES-0004, ADR-SITES-0005, ADR-SITES-0006, ADR-SITES-0007, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-sites
doc_status: published
---

# PRD-sites: Published-Web + Intranet µservice

## Purpose

The `sites` µservice is oyatie's native published-web + intranet substrate — a Google-Sites + WordPress + Squarespace + Webflow + Notion-Sites + Carrd + Framer + Ghost + Hugo-class competitor unified under a single tenant-facing surface. It owns: site (named published space) authoring; URL-routed page rendering (static + dynamic); block-based composition (paragraph, heading, image, video, embed, form, cms-collection); theme + design-token system; navigation (header/footer/sidebar; per-page or global); custom domain binding with ACME-automated TLS (RFC 8555 + DNS-01); SEO surface (meta + Open Graph + Twitter Cards + schema.org JSON-LD + sitemap.xml + robots.txt + canonical); CMS-collection model (structured-content type + entries + relationships); site-wide search (Meilisearch); forms-integration (cross-µservice to `forms`); e-commerce-stub (T-G fintech bridge); privacy-preserving analytics (Plausible-class); WCAG 2.2 AA accessibility; social-share metadata; preview-mode (draft vs published); versioning (publish + rollback); multi-language (i18n + hreflang); comments (cross-µservice to `community`); CDN-delivery (signed cache invalidation); AI-page-build (T2 — generate page from prompt); site-collaboration (Loro CRDT per ADR-SITES-0001).

Per ADR-0132 (no-suite forward-policy) and parallel-session ADR-0135 (Connect unbundle), `sites` is a standalone tenant-facing µservice — no longer part of a Connect suite. The legacy `oya-connect-sites-*` family is deprecated and migrates per ADR-0134 Strangler timeline (see `migration-from-connect.md`).

`sites` is differentiated from sibling µservices as follows: `docs` is for PRIVATE document editing (Word/Notion-class); `community` is for moderated FORUM-style discussion; `sites` is for PUBLISHED websites (intranet + public) with theme + nav + URL routing + CDN delivery. The three share the Loro CRDT collab substrate but cover non-overlapping product surfaces.

## Tenant Value

- **Tenant Outcome 1 — Publish a public website or intranet without leaving oyatie.** Tenants do not need WordPress / Squarespace / Webflow / Wix / Framer / Carrd accounts; sites is a first-party publishing substrate that integrates with the rest of the suite (forms, drive, community, docs, ontology) by construction.
- **Tenant Outcome 2 — Custom-domain delivery with automated TLS.** Bind a custom domain (apex + wildcard) with ACME (RFC 8555) DNS-01 challenge for Let's Encrypt; certs auto-renew without ops intervention.
- **Tenant Outcome 3 — CDN-grade page-render latency.** p95 ≤ 200ms page-render via SSG/ISR hybrid + signed CDN invalidation; static-asset p95 ≤ 100ms (CDN warm).
- **Tenant Outcome 4 — Structured CMS collections.** Sanity-style portable-text + Strapi-style relational hybrid; tenants model "Article", "Product", "Team Member" and bind them to URL patterns.
- **Tenant Outcome 5 — Accessibility + SEO by construction.** WCAG 2.2 AA and SEO meta correctness are SLO-bound at 100%; the editor refuses to publish pages that fail alt-text or canonical-URL checks.
- **Tenant Outcome 6 — AI page build (bounded).** T2 capability to generate a page from a prompt under EU AI Act limited-risk classification (or high-risk in legal/medical/employment overlays, per ADR-SITES-0006 — REFUSED until conformity assessment).
- **Tenant Outcome 7 — Collaborative editing.** Multiple editors co-edit the same page via Loro CRDT, aligned with docs + sheets + slides + workflow-studio per ADR-WS-0001.
- **Tenant Outcome 8 — Privacy-preserving analytics.** Plausible-class first-party analytics — no third-party cookies, ePrivacy Art. 5(3)-conformant.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to create a named site (intranet or public) | I have a published space to author within | site | Must |
| FR-02 | site editor | to create URL-routed pages (static + dynamic) | pages render at clean URLs (e.g., `/about`, `/blog/[slug]`) | page | Must |
| FR-03 | site editor | to compose pages from blocks (paragraph + heading + image + video + embed + form + cms-collection) | content authoring is structured + reusable | block | Must |
| FR-04 | site designer | to apply a theme with design tokens (typography + color + spacing) | brand consistency across the site | theme | Must |
| FR-05 | site editor | to configure navigation (header/footer/sidebar; per-page or global) | site visitors can navigate | navigation | Must |
| FR-06 | site editor | to bind a custom domain (apex + subdomain + wildcard) | the site is reachable at `example.com` | domain-binding | Must |
| FR-07 | site editor | to publish a page and have it served from the CDN within 5s p95 | publishing feels immediate | url-routing + cdn-delivery | Must |
| FR-08 | site editor | to redirect old URLs (301/302/410) | URL migrations preserve link equity | url-routing | Must |
| FR-09 | site editor | to set SEO meta (title, description, Open Graph, Twitter Cards, schema.org JSON-LD, canonical, hreflang) | the site ranks on search + previews well on social | seo | Must |
| FR-10 | site visitor | to crawl `/sitemap.xml` and `/robots.txt` | search engines discover content | seo | Must |
| FR-11 | site editor | to define CMS-collection types (Article, Product, Team Member) with fields + relationships | structured content is modelled | cms-collection | Must |
| FR-12 | site visitor | to search the site (p95 ≤ 300ms) | content discovery works | search | Must |
| FR-13 | site visitor | to submit a form (cross-µservice to `forms`) | the site captures leads / signups | forms-integration | Must |
| FR-14 | site editor | to bind an e-commerce checkout (T-G fintech stub) | the site can sell | e-commerce-stub | Should |
| FR-15 | tenant operator | to view privacy-preserving analytics (pageviews, sessions, referrers; no third-party cookies) | I measure traffic ePrivacy-conformantly | analytics | Must |
| FR-16 | site editor | to upload images and have them automatically converted to WebP/AVIF/JPEG-XL responsive variants | images render fast on every device | (cross-cutting; libvips) | Must |
| FR-17 | site editor | to preview a draft (with private URL) before publishing | I see my changes before they go live | preview-mode | Must |
| FR-18 | site editor | to roll back to a prior published version | mistakes are recoverable | versioning | Must |
| FR-19 | site editor | to publish pages in multiple languages with `hreflang` | multi-language SEO works | multi-language | Must |
| FR-20 | site visitor | to comment on a page (cross-µservice to `community`) | engagement loop works | comments | Should |
| FR-21 | site editor | to invalidate CDN cache for a specific URL or pattern | content updates propagate immediately | cdn-delivery | Must |
| FR-22 | site editor | to generate a page from a prompt (T2 AI-page-build, gated) | rapid prototyping | ai-page-build | Should |
| FR-23 | site editor | to co-edit a page concurrently with another editor (Loro CRDT) | team authoring without lock contention | site-collaboration | Must |
| FR-24 | site editor | to bind an embedded form (from `forms` µservice) | forms appear in pages | forms-integration | Must |
| FR-25 | site editor | to bind a docs link block (from `docs` µservice) | docs embed into pages | (cross-cutting) | Should |
| FR-26 | accessibility reviewer | to receive a publish-time WCAG 2.2 AA report (alt-text + contrast + heading-order + landmark) | regressions are caught before going live | accessibility | Must |
| FR-27 | tenant operator | to receive a webhook on site-publish + page-publish | downstream Workflow can react | (cross-cutting) | Must |
| FR-28 | tenant compliance officer | to put a page under legal hold | publish-state preserved past retention | versioning | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p95 | p99 | Notes |
|---|---|---|---|---|
| Page-render (cached SSG) | ≤30ms | ≤200ms | ≤400ms | Postgres + Valkey page-record + Brotli/Zstd response |
| Static-asset (CDN warm) | ≤20ms | ≤100ms | ≤200ms | Cloudflare-class edge cache |
| CMS-collection query | ≤50ms | ≤150ms | ≤300ms | indexed Postgres + Valkey cache |
| Site-search | ≤100ms | ≤300ms | ≤600ms | Meilisearch per-tenant index |
| Publish (100-page site) | ≤2s | ≤5s | ≤10s | parallel SSG render + S3 upload + CDN purge |
| ACME cert renew | ≤15s | ≤30s | ≤60s | Let's Encrypt DNS-01 |
| Image-optimize (single) | ≤500ms | ≤1s | ≤2s | libvips WebP/AVIF/JPEG-XL |
| AI-page-build (T2) | — | ≤5s | ≤10s | tenant-default LLM; bounded prompt |

### Security

- All site content encrypted-at-rest under tenant-DEK (per Bominal ADR-0111) for non-public-flagged pages; published-public pages are stored unencrypted (the bytes are public by design) but their authorship records carry the audit-chain seal.
- All custom-domain bindings verified at DNS layer via signed challenge (TXT record or HTTP-01) before cert issuance; misroute refused.
- All form submissions traverse the `forms` µservice (cross-µservice via Workflow + Ontology); sites never persists form data directly.
- All AI-page-build prompts are tenant-DEK wrapped; output content is reviewed by the tenant editor before publish (T2 reversibility window 30s); cross-tenant training is structurally forbidden.
- All published pages set `Content-Security-Policy`, `X-Frame-Options`, `Strict-Transport-Security`, `Referrer-Policy: strict-origin-when-cross-origin`; values configurable per tenant per page.
- All Subresource Integrity (SRI) hashes are computed at publish for all `<script>` and `<link rel="stylesheet">` per W3C SRI Recommendation.
- All anonymous public reads are rate-limited per Cedar `public-read.cedar` per-IP/per-tenant budget.

### Audit + Compliance

- Every `SitePublished / PagePublished / PageReverted / DomainBound / CertIssued / CmsCollectionUpdated / FormBound / AnalyticsConsentGiven / AiPageBuildAccepted` emits an audit-chain record (Merkle + Ed25519 per Bominal ADR-0028).
- Legal-hold preserves page-version + authorship history past retention expiry.
- Per-jurisdiction retention computed per ADR-0140 Cedar pack overlay; pack-eu honours GDPR Art. 17 right-to-erasure for site authorship records.
- EU DSA Arts. 14 + 27 transparency for moderation actions (publish refusals via content policy).
- WCAG 2.2 AA correctness recorded per publish; correctness lane refuses publish at < 100%.

### Availability + SLO

- Availability target: 99.99% monthly for page-render path (public-facing content must be hard to take offline); 99.95% for editor write path.
- RTO <= 1800s; RPO <= 300s (manifest `dr` block; Postgres logical replication and versioned object storage for published artifacts).
- CDN edge cache survives origin outage for ≥ 24h (cache-control: stale-while-revalidate=86400).

### Data residency

- Tenant data pinned to the tenant's region per ADR-0117 + ADR-0140; cross-region replication forbidden by default; SCC-gated when activated.
- Custom domains: DNS records may resolve to globally-anycast CDN edges, but origin pages live in the tenant pack.

### DR posture (ADR-0343)

- RTO/RPO target: manifest-declared RTO p99 1800s and RPO p99 300s for origin page records, CMS collections, and published artifact manifests, meeting the HIPAA-2024 floor of 3600s/300s, SOC2-T2 floor of 14400s/900s, and KR-PIPA floor of 14400s/900s. Effective floor driver: HIPAA-2024 for healthcare intranet and intake sites.
- Failover reference: manifest `failover_runbook` is `runbooks/dr-failover.md`; supporting edge runbooks remain `runbooks/cdn-cache-purge-cascade.md` and `runbooks/custom-domain-dns-drift.md`.
- Multi-region active-active posture: true per manifest; replication shape is `active-active-multi-az-cross-region-warm` across `postgres_wal_g`, versioned object storage, and Valkey, with CDN edges continuing active-active cached reads.
- Tenant-visible behavior: visitors continue to receive cached pages for at least 24h during an origin outage, and editors see publish operations pause rather than corrupting page versions or custom-domain state.

### Capacity model (ADR-0340)

- Per-tenant baseline: manifest-declared 0.10 vCPU, 256 MiB RAM, 10 GB storage, two Postgres connections, two Valkey connections, and six outbound HTTP connections, with the medium-tenant operating shape of 1k pages, 100k monthly visitors, 5 origin cache-miss RPS, 100 CDN-hit RPS, 5 editor RPS, 20 CMS query RPS, and 2 site-search QPS.
- Scaling dimension: `per_request` for page render/CDN miss, `per_query` for CMS/search, and `per_publish_job` for ISR/SSG and image optimization.
- Cell placement class: Tier-3 per manifest for tenant-facing published-web authoring and origin render state, with Tier-4 CDN edge cache allowed only for public bytes that are already safe to serve globally.
- Autoscaling boundaries: site-rest 3-50 replicas, page/url/cdn workers 5-100, image-optimize workers 2-40, and search/CMS read replicas scaled per tenant collection count and cache-miss pressure.
- Tenant load profile: serves many read-heavy public sites without letting one viral custom domain starve editor writes, ACME renewal, or private intranet authoring.

### Sustainability and cost attribution (ADR-0344)

- Per-call emission claim: page render, CDN purge, image optimization, search, publish, and AI-page-build audit rows emit `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, site, provider, cell, and compliance_pack axes.
- Carbon-aware provider routing: yes for image optimization, AI-page-build, and batch publish jobs where the tenant's pack and SLO allow; no for page-render hot path, custom-domain failover, or legal-hold preservation.
- Tenant transparency surface: finops-portal shows per-site CDN, origin render, image, search, and AI build cost lines so public websites and intranets can be charged back separately.
- Regulatory driver: CSRD, SB-253, and SEC climate disclosure reporting require site-delivery emissions by tenant and provider, not only aggregate CDN invoices.

### API versioning posture (ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet across version header, URL prefix, and proto3 field for site/page/domain/search/webhook contracts.
- SDK semver model: site SDKs use `major.minor.patch`; generated clients bump major only when the date-versioned public contract breaks.
- Support window: last 3 public versions are supported for at least 180 days.
- Per-tenant pinning: yes for editor APIs, publish webhooks, custom-domain automation, and external CMS integrations.
- Internal-mesh exemption: yes; direct gRPC among Sites components remains ADR-0145 mesh-internal while public callers use date carriers.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename for new crates). Eleven primary BCs (the "11 packs" of the µservice surface; not to be confused with the regional pack overlays).

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `site` | `oya-sites-site-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | Named published space; visibility flag (public/intranet); domain binding pointer | `Site`, `SiteVisibility`, `SiteOwner` |
| `page` | `oya-sites-page-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` | URL-routed page (static + dynamic); version + publish state | `Page`, `PageVersion`, `PageDraftState`, `PageBindings` |
| `block` | `oya-sites-block-{kernel,domain,usecase,api,adapter,adapter-loro,app}` | Block composition + Loro CRDT alignment for collab; portable-text serialisation | `Block`, `BlockKind{Paragraph,Heading,Image,Video,Embed,Form,CmsCollection,Code,Quote,Divider}`, `PortableTextNode` |
| `theme` | `oya-sites-theme-{kernel,domain,usecase,api,adapter,app}` | CSS-in-rust scoped CSS; design-token bundling via LightningCSS | `Theme`, `DesignToken`, `ThemeCompiled` |
| `navigation` | `oya-sites-navigation-{kernel,domain,usecase,api,adapter,app}` | Header / footer / sidebar nav; per-page or global; hierarchical menu | `NavBar`, `NavItem`, `NavScope{Global,PerPage}` |
| `url-routing` | `oya-sites-url-routing-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,app}` | Clean URLs + redirects (301/302/410); URL signature stability | `Route`, `Redirect`, `RouteMatch` |
| `domain-binding` | `oya-sites-domain-binding-{kernel,domain,usecase,api,adapter,adapter-acme,adapter-cert-manager,rest,worker,app}` | Custom-domain DNS verify + ACME cert auto-renew per RFC 8555 | `Domain`, `DnsVerification`, `Certificate`, `AcmeChallenge` |
| `seo` | `oya-sites-seo-{kernel,domain,usecase,api,adapter,app}` | Meta + Open Graph + Twitter Cards + schema.org JSON-LD + sitemap + robots + canonical + hreflang | `SeoMeta`, `OpenGraphTags`, `Sitemap`, `RobotsTxt`, `JsonLdDocument` |
| `cms-collection` | `oya-sites-cms-collection-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}` | Structured-content type + entries + relationships | `CollectionType`, `Entry`, `FieldDefinition`, `Relationship` |
| `search` | `oya-sites-search-{kernel,domain,usecase,api,adapter,adapter-meilisearch,rest,worker,app}` | Per-tenant site search index | `SearchIndex`, `SearchQuery`, `SearchResult` |
| `cdn-delivery` | `oya-sites-cdn-delivery-{kernel,domain,usecase,api,adapter,adapter-s3,adapter-cloudflare-cdn-stub,rest,worker,app}` | Published artifact storage + signed cache invalidation | `PublishedArtifact`, `CdnCacheKey`, `InvalidationRequest` |

The five cross-cutting capabilities (`forms-integration`, `e-commerce-stub`, `analytics`, `accessibility`, `preview-mode`, `versioning`, `multi-language`, `comments`, `ai-page-build`, `site-collaboration`) are NOT separate BCs — they are concerns implemented across the 11 primary BCs (per ADR-0132 single-concern µservice rule).

Naming justification (one of eleven; same shape applies to others) — `page`:

```
NAME: oya-sites-page-<layer>
JUSTIFICATION:
- microservice = sites: this µservice; ADR-0056 v4.1 flat BNF + ADR-0131
  per-microservice folder. No shared|vertical bisection.
- bc-tokens = page: primary BC for URL-routed page authoring; siblings
  (site, block, theme, navigation, url-routing, domain-binding, seo,
  cms-collection, search, cdn-delivery) justify explicit BC token per
  ADR-0056 v4.1 BC-optionality rule.
- layer = <layer>: one crate per layer per ADR-0105 13-value canonical
  enum.
  - kernel: port-trait + entity types (Page, PageVersion, PageDraftState,
    PageBindings). Zero I/O. data_class annotations.
  - domain: pure page-invariant math (version monotonicity, draft↔
    published transitions, URL routing precedence, hreflang reciprocity).
  - usecase (per ADR-0106): orchestrators (create-page, update-page,
    publish-page, revert-page, schedule-publish) reading via ports.
  - api: protocol-neutral typed contracts.
  - adapter: protocol-neutral implementations of kernel ports.
  - adapter-postgres: backend-qualified adapter (per ADR-0105
    Amendment 3); implements PageRepository against Postgres with RLS.
  - rest: HTTP handler/route layer.
  - worker: long-lived background workers (publish queue, scheduled
    publish, version-garbage-collection).
  - sdk: client library for tenants + workflow consumers.
  - app: composition root binary.
- exemptions claimed: none.
```

Layer mapping table per BC (13-layer enum from ADR-0105; `usecase` per ADR-0106):

| BC | kernel | domain | usecase | api | adapter | adapter-postgres | adapter-valkey | adapter-s3 | adapter-loro | adapter-meilisearch | adapter-pandoc | adapter-libvips | adapter-acme | adapter-cert-manager | adapter-cloudflare-cdn-stub | rest | worker | sdk | app |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `site` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `page` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| `block` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | ✓ | — | — | — | — | — | — | — | — | — | ✓ |
| `theme` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ |
| `navigation` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ |
| `url-routing` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | — | — | ✓ |
| `domain-binding` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | ✓ | ✓ | — | ✓ | ✓ | — | ✓ |
| `seo` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | — | — | — | — | ✓ |
| `cms-collection` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `search` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | — | — | ✓ | — | — | — | — | — | ✓ | ✓ | — | ✓ |
| `cdn-delivery` | ✓ | ✓ | ✓ | ✓ | ✓ | — | — | ✓ | — | — | ✓ | ✓ | — | — | ✓ | ✓ | ✓ | — | ✓ |

Total crates introduced by this µservice: **78** (eleven BC families).

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implemented in | Data classes touched |
|---|---|---|---|
| `SiteRepository` | `oya-sites-site-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `PageRepository` | `oya-sites-page-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` + `PUBLIC_BY_TENANT_CHOICE` |
| `BlockStore` | `oya-sites-block-kernel` | `-adapter-loro` | `BEHAVIORAL_TENANT_PRODUCT` + `PUBLIC_BY_TENANT_CHOICE` |
| `ThemeCompiler` | `oya-sites-theme-kernel` | `-adapter` (LightningCSS) | `INTERNAL_ONLY` |
| `NavigationResolver` | `oya-sites-navigation-kernel` | `-adapter` | `INTERNAL_ONLY` |
| `RouteResolver` | `oya-sites-url-routing-kernel` | `-adapter-postgres` | `INTERNAL_ONLY` |
| `DnsVerifier` | `oya-sites-domain-binding-kernel` | `-adapter` | `INTERNAL_ONLY` |
| `AcmeClient` | `oya-sites-domain-binding-kernel` | `-adapter-acme` | `SECRET_TLS_KEY` (per-tenant private key) |
| `CertificateStore` | `oya-sites-domain-binding-kernel` | `-adapter-cert-manager` | `SECRET_TLS_KEY` |
| `SeoMetaProducer` | `oya-sites-seo-kernel` | `-adapter` | `PUBLIC_BY_TENANT_CHOICE` |
| `CollectionRepository` | `oya-sites-cms-collection-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` + `PUBLIC_BY_TENANT_CHOICE` |
| `SearchIndex` | `oya-sites-search-kernel` | `-adapter-meilisearch` | `BEHAVIORAL_TENANT_PRODUCT` + `PUBLIC_BY_TENANT_CHOICE` |
| `ArtifactStore` | `oya-sites-cdn-delivery-kernel` | `-adapter-s3` | `PUBLIC_BY_TENANT_CHOICE` |
| `CacheInvalidator` | `oya-sites-cdn-delivery-kernel` | `-adapter-cloudflare-cdn-stub` | `INTERNAL_ONLY` |
| `ImagePipeline` | `oya-sites-cdn-delivery-kernel` | `-adapter-libvips` | `PUBLIC_BY_TENANT_CHOICE` |
| `MarkdownRenderer` | `oya-sites-block-kernel` | (via `-adapter` calling `oya-sites-cdn-delivery-adapter-pandoc`) | `PUBLIC_BY_TENANT_CHOICE` |
| `RetentionPolicyResolver` | `oya-sites-page-kernel` | `-adapter` (resolves to `tenancy` µservice via Workflow) | `AUDIT` |
| `LegalHoldStore` | `oya-sites-page-kernel` | `-adapter-postgres` | `AUDIT` |

Data-class enforcement: every kernel struct field carries a `#[data_class(...)]` annotation; the `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `sites` MUST NOT import another product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). Consumed µservices: `tenancy` (tenant + identity resolution), `audit-chain` (seal emission), `forms` (form embed via Ontology bindings), `drive` (asset storage cross-link), `community` (comments cross-link), `docs` (docs cross-link), `social` (social-share metadata), `mail` (newsletter from CMS collection), `workflow-engine` (publish-triggers-workflow), `ontology` (Site/Page/Block entity bindings), `observability` (telemetry). LEAN-A2 CI lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice sites`
- `oya gate validate lean-a2 --microservice sites`
- `oya gate validate port-location --microservice sites`
- `oya gate validate layer-correctness --microservice sites`
- `oya gate validate per-microservice-layout --microservice sites`
- `oya gate validate statelessness --microservice sites`
- `oya gate validate shardability --microservice sites`
- `oya gate validate hyperscaler-maturity --microservice sites`
- `oya gate validate rfc-8555-conformance --microservice sites` (NEW; ACME)
- `oya gate validate wcag-2.2-aa-conformance --microservice sites` (NEW; accessibility)
- `oya gate validate schema-org-jsonld-conformance --microservice sites` (NEW; SEO)
- `oya gate validate sitemap-xml-conformance --microservice sites` (NEW; SEO sitemap.xml format)

## Integration via Workflow + Ontology

### Workflow events produced

| Event | Topic | Trigger | Consumed by | Idempotency key |
|---|---|---|---|---|
| `SitePublished` | `sites.site.lifecycle.v1` | new site published | audit-chain, workflow-engine | `site_id` |
| `PagePublished` | `sites.page.lifecycle.v1` | new page version published | audit-chain, search (reindex), cdn-delivery (purge), social (refresh meta) | `page_id + version` |
| `PageReverted` | `sites.page.lifecycle.v1` | revert to prior version | audit-chain, cdn-delivery (purge) | `page_id + reverted_to_version` |
| `DomainBound` | `sites.domain.lifecycle.v1` | custom domain verified | audit-chain | `domain` |
| `CertIssued` | `sites.domain.cert.v1` | ACME cert issued | audit-chain, cdn-delivery (cert-load) | `domain + cert_serial` |
| `CertRenewed` | `sites.domain.cert.v1` | ACME cert renewed | audit-chain, cdn-delivery (cert-reload) | `domain + cert_serial` |
| `CmsCollectionUpdated` | `sites.cms.collection.v1` | collection schema change | search (reindex), audit-chain | `collection_id + version` |
| `FormBound` | `sites.forms.binding.v1` | form embedded on a page | forms (handshake), audit-chain | `page_id + form_id` |
| `AiPageBuildAccepted` | `sites.ai.page_build.v1` | T2 AI page build user-confirmed | audit-chain (with EU AI Act flags) | `page_id + build_id` |
| `LegalHoldApplied` / `LegalHoldReleased` | `audit.sites.legal_hold.v1` | hold transition | audit-chain, governance | `page_id + hold_id` |

### Workflow events consumed

| Event | Producer | Handler | Action |
|---|---|---|---|
| `TenantOnboarded` | `tenancy` | site usecase | provision tenant-DEK; create default Site shell |
| `TenantOffboarded` | `tenancy` | site usecase | mark sites for retention sweep / legal-hold scan |
| `FormSubmitted` | `forms` | page usecase | optional Workflow trigger if page declares "on-form-submit" handler |
| `CommunityCommentPosted` | `community` | page usecase | refresh page comment-count surface; revalidate ISR |
| `DocsLinkUpdated` | `docs` | block usecase | revalidate docs-link blocks; trigger CDN purge |
| `WorkflowTrigger` | `workflow-engine` | publish usecase | scheduled-publish, batch-republish |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit |
|---|---|---|---|
| `Site{site_id, tenant, visibility, owner_user, ...}` | `sites→Tenant`, `sites→User(owner)` | `site` | Ed25519 |
| `Page{page_id, site_id, route, version, draft|published}` | `pages→Site` | `page` | Ed25519 |
| `Block{block_id, page_id, kind, version}` | `blocks→Page` | `block` | Ed25519 |
| `Theme{theme_id, site_id, tokens_hash}` | `themed→Site` | `theme` | Ed25519 |
| `Domain{domain, site_id, dns_verified_at, cert_expiry}` | `binds→Site` | `domain-binding` | Ed25519 |
| `CollectionType{collection_id, site_id, schema_hash}` | `collected→Site` | `cms-collection` | Ed25519 |
| `Entry{entry_id, collection_id, fields_hash}` | `entry_of→CollectionType` | `cms-collection` | Ed25519 |
| `LegalHold{hold_id, page_id, opened_by, opened_at}` | `holds→Page` | `page` | Ed25519 |

### Ontology reads

| Object | Read by | Query shape |
|---|---|---|
| `User` (tenant directory) | `site`, `page` | by `(tenant_id, user_id)` |
| `Tenant` | `site`, `domain-binding` | by `tenant_id` |
| `RetentionPolicy` | `page` | by `(tenant_id, pack)` |
| `Form` (cross-µservice) | `block` (form-block) | by `(tenant_id, form_id)` via Ontology |
| `DocsDocument` (cross-µservice) | `block` (docs-link block) | by `(tenant_id, doc_id)` via Ontology |
| `CommunityThread` (cross-µservice) | `block` (community-comments block) | by `(tenant_id, thread_id)` via Ontology |

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Google Sites | Workspace Sites | intranet authoring; theme; nav; embed | `support.google.com/sites` |
| WordPress.org | self-hosted CMS | block editor (Gutenberg); themes; plugins; multi-site | `developer.wordpress.org` |
| WordPress.com | hosted CMS | hosted; custom domain; analytics | `developer.wordpress.com` |
| Squarespace | hosted website builder | theme; e-commerce; analytics | `developers.squarespace.com` |
| Wix | hosted website builder | theme; e-commerce; bookings | `dev.wix.com` |
| Webflow | visual website builder | CMS; CSS-grid layout; CMS-collection; hosting | `developers.webflow.com` |
| Notion Sites | Notion-page publishing | Notion-to-public-site | `developers.notion.com` |
| Carrd | one-page sites | minimalist; landing pages | `carrd.co` |
| Framer Sites | designer-first builder | Figma-class canvas; CMS; hosting | `framer.com/developers` |
| Ghost | publishing-focused | newsletter; membership; CMS | `ghost.org/docs` |
| Hugo / 11ty / Astro / Gatsby | SSG framework | self-host; CMS-source-of-truth | `gohugo.io`, `11ty.dev`, `astro.build`, `gatsbyjs.com` |
| Sanity / Strapi / Contentful / Storyblok | headless CMS | content modelling; API delivery | `sanity.io`, `strapi.io`, `contentful.com`, `storyblok.com` |
| Sitecore / Adobe Experience Manager | enterprise WCM | personalisation; A/B; multi-channel | `sitecore.com`, `adobe.com/aem` |

Key parity gaps to close (ordered):

1. **Native intranet + public site under one µservice** — Google Sites does intranet; Squarespace does public; nobody does both with the same authoring substrate at the per-tenant level. **Differentiator.**
2. **Loro-CRDT-aligned collaborative editing across docs + sheets + slides + sites** — no competitor unifies the collab substrate across their CMS + their docs + their slides. **Differentiator.**
3. **CMS-collection + Ontology binding** — Webflow has CMS-collections; nobody binds them to a tenant-wide Ontology that flows into other product µservices. **Differentiator.**
4. **Privacy-preserving analytics by default** — Plausible-class without third-party cookies; WordPress / Squarespace / Wix all ship Google Analytics integrations. **Differentiator.**
5. **ACME wildcard cert for tenant subdomains + custom-domain DNS-01** — Webflow + Squarespace cover this; we must reach parity.
6. **Webflow / Framer CSS-grid layout** — visual designer-class control over layout; scheduled-for-distinct-tracked-work to M04.
7. **WordPress plugin ecosystem** — out of scope; oyatie's Workflow-engine is the plugin substrate.

## Performance Targets (canonical bench surface)

| Metric | Target | Verification |
|---|---|---|
| Page-render p95 (cached SSG) | ≤ 200ms | `cargo bench -p oya-sites-cdn-delivery-adapter-s3 -- page_render` |
| Static-asset p95 (CDN warm) | ≤ 100ms | `cargo bench -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- static_asset` |
| CMS-collection query p95 | ≤ 150ms | `cargo bench -p oya-sites-cms-collection-adapter-postgres -- query` |
| Site-search p95 | ≤ 300ms | `cargo bench -p oya-sites-search-adapter-meilisearch -- query` |
| Publish (100-page site) p95 | ≤ 5s | `cargo bench -p oya-sites-cdn-delivery-usecase -- publish_100_pages` |
| ACME cert renew p95 | ≤ 30s | `cargo bench -p oya-sites-domain-binding-adapter-acme -- renew` |
| Image-optimize (single) p95 | ≤ 1s | `cargo bench -p oya-sites-cdn-delivery-adapter-libvips -- optimize` |
| AI-page-build p95 (T2) | ≤ 5s | `cargo bench -p oya-sites-page-usecase -- ai_page_build` |

Error budget: monthly 99.99% availability for read path → ~4.3 min/month.

## Horizontal Scalability

State strategy (per Bominal ADR-0019): `mixed`. Postgres (site/page/collection metadata; per-tenant RLS); Valkey (page-render cache; per-tenant key prefix); S3 (published artifacts); Meilisearch (per-tenant index); Loro CRDT log (per-page edit history); stateless workers for publish-pipeline + cert-renew + image-optimize + search-reindex.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active sites | 50k | 500k | Postgres connection pool > 70% |
| Pages/s read (cached) | 10k | 100k | cdn-delivery-rest p95 > 200ms |
| Page publishes/s | 100 | 1k | publish-worker queue depth > 60s |
| CMS-collection writes/s | 500 | 5k | postgres FOR UPDATE wait > 200ms |
| Site-search QPS | 5k | 50k | meilisearch CPU > 70% |
| ACME cert renewals/day | 1k | 10k | Let's Encrypt rate-limit (per ADR-SITES-0004) |
| Image-optimize jobs/min | 500 | 5k | libvips worker queue > 5min |
| Concurrent Loro CRDT edit sessions | 5k | 50k | crdt-relay pod CPU > 70% |

Scale-out policy:
- Kubernetes HPA: rest pods scale on CPU > 70%; min 3, max 100.
- Postgres: per-tenant logical shard; cross-cell replication-factor 3 with Patroni.
- Valkey: cluster mode; per-tenant key prefix; eviction policy `allkeys-lru` for page-render cache.
- Meilisearch: per-tenant index; sharded by tenant_id hash; cross-cell replication 2.
- Pre-warmed pool: 5 standby pods; cold-start ≤ 700ms.

Cross-region: M03 launches in KR (ap-seoul-1); M04 expands to EU + US per ADR-0117 jurisdiction pack.

Sharding: sites partitioned by `tenant_id`; pages partitioned by `(site_id, version_year_month)`; cms-entries partitioned by `(collection_id)`; published artifacts on S3 keyed by `(tenant_id, site_id, version_hash, route_path)`.

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Site create + first page publish completes p95 ≤ 5s | `cargo bench` |
| AC-02 | Page-render p95 (cached SSG) ≤ 200ms | `cargo bench -p oya-sites-cdn-delivery-adapter-s3 -- page_render` |
| AC-03 | ACME DNS-01 cert issuance + DNS verify completes p95 ≤ 30s for `*.tenant.example` wildcard | `cargo nextest -p oya-sites-domain-binding-adapter-acme -- dns01_wildcard` |
| AC-04 | CMS-collection query p95 ≤ 150ms for 1000-entry collection | `cargo bench -p oya-sites-cms-collection-adapter-postgres -- query_1000` |
| AC-05 | sitemap.xml output validates against the Sitemap protocol XSD | `cargo nextest -p oya-sites-seo-domain -- sitemap_xsd_validate` |
| AC-06 | schema.org JSON-LD output validates against schema.org JSON-LD context | `cargo nextest -p oya-sites-seo-domain -- jsonld_context_validate` |
| AC-07 | Published page passes WCAG 2.2 AA correctness lane (alt-text, contrast 4.5:1, heading-order, landmark) | `cargo nextest -p oya-sites-page-usecase -- wcag22_aa` |
| AC-08 | Legal-hold preserves page-version + authorship history past retention | `cargo nextest -p oya-sites-page-domain -- legal_hold` |
| AC-09 | Tenant-DEK envelope encryption applied to non-public page content; verified at rest | `tests/e2e/encryption-at-rest.rs` |
| AC-10 | Loro CRDT merge of concurrent edits on the same page converges deterministically | `cargo nextest -p oya-sites-block-adapter-loro -- crdt_converge` |
| AC-11 | Audit-chain seal emitted for every site/page/domain/cert/collection lifecycle event | `cargo nextest -p oya-sites-site-app -- audit_chain_emission` |
| AC-12 | `oya gate validate per-microservice-layout --microservice sites` exit 0 | ADR-0131 lane |
| AC-13 | T2 AI-page-build refuses HR/legal/medical-context prompts pending ADR-SITES-XXXX conformity | `cargo nextest -p oya-sites-page-usecase -- ai_page_build_refusal_hr` |
| AC-14 | CDN cache invalidation on publish completes p95 ≤ 2s | `cargo bench -p oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub -- invalidate` |
| AC-15 | URL signature stability: 301/302/410 redirect map preserves Hyrum's-Law surfaces from legacy `oya-connect-sites-*` | `cargo nextest -p oya-sites-url-routing-domain -- redirect_signature_stability` |

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | Should we ship a Webflow-class visual layout designer (CSS-grid + flexbox visual editor) at M04 or defer to M05? | council-product | M04 decision |
| 2 | AMP-stub: should we generate AMP-HTML variants alongside canonical HTML, or skip (Google deprecating AMP signals)? | axis-sites | subsequent-to-M04-completion |
| 3 | WordPress import path: should we ship an importer for tenants migrating off WordPress.org/.com? | council-product | subsequent-to-M04-completion |
| 4 | CMS-collection data model: portable-text (Sanity) vs relational (Strapi) vs custom hybrid — see ADR-SITES-0005 (hybrid chosen) | axis-sites | resolved by ADR-SITES-0005 |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application→usecase | layer rename |
| ADR-0117 | Cloud-native infrastructure | data residency |
| ADR-0135 | Connect unbundle (parallel session) | sites independence |
| ADR-0139 | Agentic SLO-gated promotion | gate authority |
| ADR-0131 | Per-microservice flat layout | layout authority |
| ADR-0132 | Product-suite + bundle dissolution | µservice independence |
| ADR-0133 | Industry-best-practice conformance | hyperscaler-grade bar |
| ADR-0134 | Connect dissolution Strangler migration | migration policy |
| ADR-0140 | Cedar policy enforcement | policy substrate |
| ADR-SITES-0001 | CRDT library selection (Loro 1.x) | collab substrate |
| ADR-SITES-0002 | Static vs dynamic rendering (SSG/ISR hybrid) | rendering architecture |
| ADR-SITES-0003 | CDN substrate + cache strategy | cache + purge model |
| ADR-SITES-0004 | ACME + custom-domain flow | TLS automation |
| ADR-SITES-0005 | CMS-collection data model | content modelling |
| ADR-SITES-0006 | AI-page-build bounds (EU AI Act) | T2 autonomy bound |
| ADR-SITES-0007 | Image + asset pipeline (libvips + WebP/AVIF/JPEG-XL) | media optimisation |

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-foundry-fitness-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
