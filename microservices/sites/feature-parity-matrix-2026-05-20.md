# Sites Feature Parity Matrix - 2026-05-20

Audit owner: single-agent sites audit.

Target microservice: `microservices/sites`.

Counterpart set: Webflow, Squarespace, Wix.

Scope: union-coverage comparison for the `sites` published-web plus intranet product surface.

Doctrine: no tenant-class deltas are authored here; quality is uniform and gaps are expressed as product, context, tenant-class, or implementation-readiness gaps.

Local anchor: `microservices/sites/PRD.md` lines 20-37 define purpose and tenant outcomes.

Local anchor: `microservices/sites/PRD.md` lines 41-70 define FR-01 through FR-28.

Local anchor: `microservices/sites/competitor-parity-matrix.md` lines 20-47 define the older broad competitor matrix.

Local anchor: `microservices/sites/PRD.md` lines 281-295 include Webflow, Squarespace, and Wix among the competitive benchmark sources.

External source anchor: Webflow hosting overview says Webflow hosting is managed, scales to millions of page views per day, targets less than 100 ms response time, and is delivered by Cloudflare at `https://help.webflow.com/hc/en-us/articles/33961342422547-Webflow-hosting-overview` lines 25-28.

External source anchor: Webflow Data API rate limits list 60 requests per minute for Starter/Basic, 120 for CMS/eCommerce/Business, Enterprise custom, site publish limited to one successful publish per minute, and cached content delivery requests effectively unbounded at `https://developers.webflow.com/data/v2.0.0/reference/rate-limits` lines 120-142.

External source anchor: Webflow dynamic content limits list 40 collection lists per page, 10 nested collection lists, and 100 displayed collection items per list unless pagination is used at `https://help.webflow.com/hc/en-us/articles/33961370432275-Dynamic-content-limits` lines 20-37.

External source anchor: Squarespace Commerce API rate limits are 300 requests per minute, equivalent to 5 per second, with Create Order capped at 100 requests per hour per website for API-key auth at `https://developers.squarespace.com/commerce-apis/rate-limits` lines 40-44.

External source anchor: Squarespace page limits allow up to 1,000 pages on current plans, recommend no more than 400, and define per-page limits including 250 gallery images, 20 sections recommended, and 10,000 products site-wide at `https://support.squarespace.com/hc/en-us/articles/206543087-Page-limits` lines 1001-1080.

External source anchor: Squarespace product limits include 10,000 products per store page in version 7.1, 200 in version 7.0, and 250 variants/images/SKUs per product at `https://support.squarespace.com/hc/en-us/articles/205811338-Adding-products-to-your-store` lines 1068-1080 and 1317-1323.

External source anchor: Wix CMS limits define plan-scaled collection item limits from 1,500 to 10,000,000, a 512 KB item size, 10 GB to 100 GB database storage, and free-site limits of 10,000 for Wix Studio accounts or 1,000 for non-Studio at `https://support.wix.com/en/article/cms-understanding-collection-storage-limits-and-quotas` lines 82-99, 113-133, and 230-240.

External source anchor: Wix API docs explain 429 throttling and retry after a minute for too many requests at `https://dev.wix.com/docs/rest/articles/get-started/rate-limits`.

## Section 1 - Counterpart 1 Capability Surface: Webflow

Webflow family: visual website builder plus CMS plus hosted publishing.

Webflow primary buyer: designers, marketers, agencies, and content teams that want design control without self-hosting.

Webflow core strength 1: visual layout designer with designer-grade CSS control.

Webflow core strength 2: hosted CMS collections and collection pages.

Webflow core strength 3: managed hosting and CDN delivery.

Webflow core strength 4: custom domain and SSL workflow.

Webflow core strength 5: design tokens, interactions, responsive controls, and reusable components.

Webflow core strength 6: forms and marketing capture workflows.

Webflow core strength 7: SEO metadata, sitemap, redirects, Open Graph, and clean URLs.

Webflow core strength 8: publish workflow and staging/preview behavior.

Webflow core strength 9: ecommerce surface for product/checkout use cases.

Webflow core strength 10: Data API for CMS, site, assets, and publishing integrations.

Webflow measured public limit 1: API requests are plan-scaled at 60 or 120 requests per minute, with Enterprise custom limits.

Webflow measured public limit 2: site publish is limited to one successful publish per minute.

Webflow measured public limit 3: cached content delivery API requests are effectively not rate-limited, while uncached origin requests count against plan limits.

Webflow measured public limit 4: dynamic pages can use up to 40 collection lists per page.

Webflow measured public limit 5: nested collection lists are capped at 10 per page.

Webflow measured public limit 6: a collection list displays 100 items unless pagination is enabled.

Webflow measured public limit 7: hosting claims less than 100 ms response time and Cloudflare delivery for Webflow-hosted sites.

Webflow parity required of sites: visual authoring should not require code for a normal marketing site.

Webflow parity required of sites: CMS collection types need field definitions, relationships, entry CRUD, and URL binding.

Webflow parity required of sites: publish must be fast enough to feel immediate.

Webflow parity required of sites: custom domains and TLS must be self-serve.

Webflow parity required of sites: redirects and SEO must preserve search equity during migrations.

Webflow parity required of sites: staging and preview must be safe for drafts.

Webflow parity required of sites: API and event surfaces must support automation without manual dashboard work.

Webflow parity required of sites: asset optimization must support modern responsive formats.

Webflow parity required of sites: designer and content workflows must survive team co-authoring.

Webflow parity required of sites: hosting/CDN behavior must be provable by SLOs, not just claimed.

Current sites coverage: PRD has site creation, page routes, blocks, theme, navigation, domains, SEO, CMS, search, forms, ecommerce stub, analytics, images, preview, rollback, language, comments, cache invalidation, AI, collaboration, webhooks, and legal hold in `microservices/sites/PRD.md` lines 41-70.

Current sites coverage: OpenAPI defines site create/list/update and page operations in `microservices/sites/contracts/openapi/sites.yaml` lines 60-120 and nearby path sections.

Current sites coverage: OpenAPI schemas define Site, Page, Block, SeoMeta, AiPageBuildRequest, DomainBindRequest, RedirectCreateRequest, CollectionTypeCreateRequest, SearchResultPage, and CdnPurgeRequest in `microservices/sites/contracts/openapi/sites.yaml` lines 317-476.

Current sites coverage: PRD targets page-render p95 <= 200 ms and static-asset p95 <= 100 ms in `microservices/sites/PRD.md` lines 76-85.

Current sites coverage: SLOs encode page-render and static-asset latency at `microservices/sites/slos/page-render-latency.openslo.yaml` lines 16-38 and `microservices/sites/slos/static-asset-latency.openslo.yaml` lines 16-38.

Current sites gap: Webflow-class visual layout designer is explicitly M04-onward in `microservices/sites/competitor-parity-matrix.md` lines 48-53.

Current sites gap: publish API parity is partially present, but no source implementation or tests exist under `microservices/sites/src` or `microservices/sites/tests`.

Current sites gap: migration from Webflow exists in a playbook filename, but implementation proof is absent.

Current sites gap: Webflow hosted-CDN parity cannot be proved with current Helm/Kustomize-only IaC because OpenTofu context modules are absent.

Current sites gap: API rate limiting/capacity values are local targets, not implementation-measured results.

Current sites additive opportunity over Webflow: public sites and intranet sites in the same service.

Current sites additive opportunity over Webflow: Loro CRDT alignment with docs, sheets, slides, and workflow-studio.

Current sites additive opportunity over Webflow: ontology binding for CMS entries.

Current sites additive opportunity over Webflow: publish-time accessibility refusal rather than warnings.

Current sites additive opportunity over Webflow: tenant-class plus deployment-context control if implemented through canonical OpenTofu modules.

## Section 2 - Counterpart 2 Capability Surface: Squarespace

Squarespace family: template-driven hosted website builder plus commerce.

Squarespace primary buyer: small businesses, creators, service businesses, restaurants, portfolios, and commerce operators.

Squarespace core strength 1: polished template and style system.

Squarespace core strength 2: page and section editor oriented to nontechnical users.

Squarespace core strength 3: commerce product catalog and checkout.

Squarespace core strength 4: built-in domains, SSL, redirects, SEO, and analytics.

Squarespace core strength 5: scheduling, email campaigns, members, gated content, forms, and marketing workflows.

Squarespace core strength 6: product media, variants, inventory, shipping, and order surfaces.

Squarespace core strength 7: managed hosting with support and status operations.

Squarespace core strength 8: page limits and content limits documented for usability and speed.

Squarespace public limit 1: current plans allow up to 1,000 pages.

Squarespace public limit 2: suggested maximum is 400 pages because larger sites may load slowly.

Squarespace public limit 3: version 7.1 has up to 10,000 products per store page.

Squarespace public limit 4: version 7.0 store pages have up to 200 products.

Squarespace public limit 5: each product can have 250 variants, images, and SKUs.

Squarespace public limit 6: gallery sections are capped at 250 images.

Squarespace public limit 7: page sections have no official limit, but no more than 20 is recommended.

Squarespace public limit 8: Commerce APIs allow 300 requests per minute and 5 requests per second equivalent.

Squarespace public limit 9: Create Order by API key is 100 requests per hour per website.

Squarespace parity required of sites: templates and style controls must be usable by nontechnical editors.

Squarespace parity required of sites: commerce cannot remain a permanent stub if sites is a storefront competitor.

Squarespace parity required of sites: content limits must be explicit so tenants know page, media, product, collection, and publish ceilings.

Squarespace parity required of sites: custom domains and SSL must be simple and safe.

Squarespace parity required of sites: SEO and social previews must be built into the editor.

Squarespace parity required of sites: analytics must work without third-party-cookie dependence.

Squarespace parity required of sites: support runbooks must cover domain, certificate, publish, rollback, and asset failure modes.

Squarespace parity required of sites: migration must import pages, products, media, redirects, SEO metadata, and domains.

Current sites coverage: PRD includes theme, navigation, domains, SEO, analytics, CMS, forms, ecommerce stub, preview, versioning, images, and accessibility in `microservices/sites/PRD.md` lines 43-70.

Current sites coverage: runbooks exist for ACME renewal, AI page rollback, asset degradation, CDN purge cascade, custom-domain DNS drift, page export corruption, and publish rollback.

Current sites coverage: domain binding and ACME are first-class in PRD and proto in `microservices/sites/PRD.md` lines 48 and 82-84, and `microservices/sites/contracts/proto/sites.proto` lines 195-200.

Current sites coverage: SLOs include ACME renewal, publish latency, page-render latency, image optimization, SEO correctness, and accessibility correctness.

Current sites coverage: compliance docs and DPIA exist, which Squarespace-like sites require for regulated intranets and public properties.

Current sites gap: full storefront is not M03; local matrix says ecommerce full storefront is subsequent work in `microservices/sites/competitor-parity-matrix.md` lines 50-53.

Current sites gap: no Squarespace-specific migration playbook was found.

Current sites gap: no local content limit matrix maps tenant classes and deployment contexts to page, product, CMS item, media, and publish caps.

Current sites gap: `cost-budget.md` still uses tenant-tier language, which blocks clean tenant-class pricing posture.

Current sites gap: README is absent, so a Squarespace-import implementation agent lacks one cold-start source.

Current sites additive opportunity over Squarespace: same authoring substrate for public web and intranet.

Current sites additive opportunity over Squarespace: ontology-backed CMS entries.

Current sites additive opportunity over Squarespace: publish refusal when accessibility or SEO correctness fails.

Current sites additive opportunity over Squarespace: deployment portability across six contexts, once OpenTofu is implemented.

Current sites additive opportunity over Squarespace: revenue_share tenants can map storefront revenue to platform billing if payment events are wired.

## Section 3 - Counterpart 3 Capability Surface: Wix

Wix family: hosted website builder plus Studio, CMS, ecommerce, app marketplace, bookings, Velo, and AI builder.

Wix primary buyer: small businesses, agencies, creators, shops, service providers, and builders needing many business vertical features.

Wix core strength 1: drag-and-drop site builder and Wix Studio workflows.

Wix core strength 2: CMS with plan-scaled data limits.

Wix core strength 3: ecommerce, bookings, forms, app marketplace, and business products.

Wix core strength 4: Velo/backend extensibility and REST APIs.

Wix core strength 5: AI website and content generation.

Wix core strength 6: media manager and storage limits.

Wix core strength 7: SEO, domains, SSL, redirects, and analytics.

Wix core strength 8: operational guardrails through rate limits, quotas, and monitoring.

Wix public limit 1: Light plan CMS limit is 1,500 items.

Wix public limit 2: Core plan CMS limit is 4,000 items.

Wix public limit 3: Business plan CMS limit is 20,000 items.

Wix public limit 4: Business Elite and Elite plans support 10,000,000 collection items.

Wix public limit 5: each CMS item can store 512 KB across non-media fields.

Wix public limit 6: Light/Core/Business database storage cap is 10 GB.

Wix public limit 7: Business Elite and Elite database storage cap is 100 GB.

Wix public limit 8: Wix Studio free sites can have 10,000 collection items.

Wix public limit 9: non-Studio free sites can have 1,000 collection items.

Wix public limit 10: Wix API throttling returns 429 and the docs advise waiting a minute before retrying.

Wix parity required of sites: CMS limits must be explicit and scalable.

Wix parity required of sites: AI page build must be bounded, safe, and useful for nontechnical users.

Wix parity required of sites: app-like integrations should use Workflow and Ontology rather than arbitrary plugin execution.

Wix parity required of sites: business vertical blocks need forms, checkout, booking-like integrations, membership, analytics, and campaigns.

Wix parity required of sites: media upload and image optimization must have clear per-tenant caps.

Wix parity required of sites: public read rate limits and anti-abuse controls must be visible and operable.

Wix parity required of sites: APIs must have explicit rate limits and retry behavior.

Wix parity required of sites: tenant classes must distinguish trial caps from paid scaling without quality degradation.

Current sites coverage: PRD includes CMS, forms, commerce stub, analytics, image pipeline, AI page build, comments, and webhooks in `microservices/sites/PRD.md` lines 53-70.

Current sites coverage: AI page build is bounded to safe overlays in OpenAPI via `context_overlay` in `microservices/sites/contracts/openapi/sites.yaml` lines 413-418.

Current sites coverage: tenant-scope Cedar refuses HR/legal/medical/employment/credit AI page-build overlays in `microservices/sites/policy/tenant-scope.cedar` lines 158-165.

Current sites coverage: architecture abuse defense covers edge rate limits, JA4 fingerprint, tenant, route class, bot score, anti-spoof, and anti-scrape in `microservices/sites/ARCHITECTURE.md` lines 695-706.

Current sites coverage: capacity model defines active sites, active pages, origin renders, CMS writes, search QPS, ACME renewals, image jobs, CRDT sessions, publish jobs, and CDN invalidations in `microservices/sites/capacity-model.md` lines 51-65.

Current sites gap: tenant classes are not modeled for trial/paid/revenue-share usage caps.

Current sites gap: production policy uses `tenant_tier` instead of tenant_class in `microservices/sites/policy/tenant-scope.cedar` lines 142-153.

Current sites gap: no Wix-specific migration guide for CMS collection quotas and Velo/backend semantics exists beyond the combined playbook filename.

Current sites gap: no working source tests prove AI refusal, rate-limit behavior, or cap enforcement.

Current sites gap: API rate-limit behavior is implied by Cedar and capacity docs, not contractually declared in OpenAPI extensions.

Current sites additive opportunity over Wix: stricter audit-chain evidence for publish and AI acceptance.

Current sites additive opportunity over Wix: privacy-preserving analytics by default.

Current sites additive opportunity over Wix: CRDT collaboration shared with office-suite surfaces.

Current sites additive opportunity over Wix: deployment contexts beyond hosted SaaS, once OpenTofu modules exist.

Current sites additive opportunity over Wix: no feature stratification by retired tiers; tenant-class usage caps preserve uniform quality.

## Section 4 - Union-Coverage Matrix

Legend: Covered means the product artifact specifies the capability. Partial means product intent exists but implementation, context, tenant-class, or counterpart-specific detail is missing. Gap means current artifacts do not cover it.

| Capability | Webflow | Squarespace | Wix | Current sites verdict | Primary local evidence | Gap note |
|---|---|---|---|---|---|---|
| Visual layout authoring | Core | Core | Core | Partial | `competitor-parity-matrix.md:48-53` | Webflow-class visual designer is M04-onward. |
| Block editor | Core | Core | Core | Covered | `PRD.md:45`; `openapi/sites.yaml:381-388` | Needs source proof. |
| Theme/design tokens | Core | Core | Core | Covered | `PRD.md:46`; `PRD.md:125` | Manifest omits theme BC. |
| Navigation | Core | Core | Core | Covered | `PRD.md:47`; `PRD.md:126` | Manifest omits navigation BC. |
| Clean URL pages | Core | Core | Core | Covered | `PRD.md:44`; `PRD.md:127` | Manifest omits url-routing BC. |
| Redirects | Core | Core | Core | Covered | `PRD.md:50`; `openapi/sites.yaml:427-433` | Needs import tests. |
| Custom domains | Core | Core | Core | Covered | `PRD.md:48`; `openapi/sites.yaml:420-425` | No OpenTofu context modules. |
| TLS automation | Core | Core | Core | Covered | `PRD.md:82-84`; `proto/sites.proto:195-200` | Needs context/IaC proof. |
| SEO metadata | Core | Core | Core | Covered | `PRD.md:51-52`; `openapi/sites.yaml:390-412` | Manifest omits SEO BC. |
| Sitemap and robots | Core | Core | Core | Covered | `PRD.md:52`; `proto/sites.proto:208-211` | Needs source tests. |
| CMS collections | Core | Secondary | Core | Covered | `PRD.md:53`; `openapi/sites.yaml:435-447` | Tenant-class limits absent. |
| Collection page item limits | Public docs | Public docs | Public docs | Partial | `capacity-model.md:22-65` | Local caps not tied to tenant class. |
| Site search | Core | Core | Core | Covered | `PRD.md:54`; `proto/sites.proto:223-225` | Needs implementation proof. |
| Forms | Core | Core | Core | Covered | `PRD.md:55,66`; AsyncAPI `FormBound` | Cross-service proof is external. |
| Ecommerce | Core | Core | Core | Partial | `PRD.md:56`; `competitor-parity-matrix.md:50-53` | Stub only at M03. |
| Analytics | Core | Core | Core | Covered | `PRD.md:57`; `PRD.md:302` | Needs meter contract. |
| Image optimization | Core | Core | Core | Covered | `PRD.md:58`; image SLO | Needs source proof. |
| Preview mode | Core | Core | Core | Covered | `PRD.md:59` | Needs contract endpoint proof. |
| Version rollback | Core | Core | Core | Covered | `PRD.md:60`; `proto/sites.proto:185-193` | Needs source tests. |
| Multi-language and hreflang | Core | Partial | Core | Covered | `PRD.md:61`; `openapi/sites.yaml:410-412` | Needs migration playbook coverage. |
| Comments | Integration | Core | Core | Covered | `PRD.md:62`; PRD cross-service links | Depends on community. |
| CDN purge | Core | Core | Core | Covered | `PRD.md:63`; `openapi/sites.yaml:464-469` | No OpenTofu proof. |
| AI page build | Emerging | Emerging | Core | Covered | `PRD.md:64`; `openapi/sites.yaml:413-418` | Policy uses `tenant_tier`. |
| Collaboration | Partial | Partial | Partial | Covered | `PRD.md:65`; `PRD.md:299-301` | Needs source proof. |
| Webhooks | API | Limited | API | Covered | `PRD.md:69`; AsyncAPI channels | Needs rate-limit contract. |
| Legal hold | Enterprise | Limited | Limited | Covered | `PRD.md:70`; `openapi/sites.yaml:353` | Strong additive surface. |
| Accessibility gate | Advisory | Advisory | Advisory | Covered | `PRD.md:68`; accessibility SLO | Implementation proof absent. |
| Commerce product import | Relevant | Core | Core | Gap | migration playbook inventory | Squarespace product migration absent. |
| Webflow import | Core migration | Not applicable | Not applicable | Partial | combined migration playbook | Needs test harness. |
| Squarespace import | Not applicable | Core migration | Not applicable | Gap | inventory absence | Top-3 gap. |
| Wix import | Not applicable | Not applicable | Core migration | Partial | combined migration playbook | Needs Wix CMS quota mapping. |
| API rate limits | Public docs | Public docs | Public docs | Partial | `policy/public-read.cedar`, architecture abuse defense | OpenAPI lacks explicit limits. |
| Plan/cap limits | Public docs | Public docs | Public docs | Partial | `capacity-model.md:20-65` | Use tenant classes, not tiers. |
| Deployment portability | Not core | Not core | Not core | Gap | ADR-0328 context rule | No context modules. |
| OCI Always Free profile | Not core | Not core | Free plan analogue | Gap | `master-plan-sequencing.json:857-865` | No `iac/oci-guest/always-free/`. |
| On-prem/colo hosting | Not core | Not core | Not core | Gap | ADR-0328 context rule | No context docs or modules. |
| OpenTofu provisioning | Not competitor feature | Not competitor feature | Not competitor feature | Gap | `IP-001:16-69` | Helm/Kustomize only. |
| OS support manifest | Not competitor feature | Not competitor feature | Not competitor feature | Gap | inventory absence | Needed for Oyatie doctrine. |
| Tenant-class adoption | Not competitor feature | Not competitor feature | Plan model analogue | Gap | `tenant-scope.cedar:142-153` | Production policy uses tenant tier. |

## Section 5 - Family Summary

Family summary 1: Webflow sets the strongest bar for visual design control and CMS collection authoring.

Family summary 2: Squarespace sets the strongest bar for polished nontechnical business-site creation and commerce defaults.

Family summary 3: Wix sets the strongest bar for broad small-business vertical breadth, app-like integrations, AI builder, CMS scale, and usage quotas.

Family summary 4: Sites already aims at all three families in one product surface.

Family summary 5: Sites has stronger written ambitions than the local implementation evidence can prove.

Family summary 6: Sites has stronger compliance, audit-chain, CRDT, and ontology ambitions than all three counterparts.

Family summary 7: Sites is behind Webflow on visual layout designer maturity.

Family summary 8: Sites is behind Squarespace on finished storefront/product-catalog UX because ecommerce is a stub.

Family summary 9: Sites is behind Wix on explicit published CMS quota and app ecosystem semantics.

Family summary 10: Sites is ahead on product-theory for intranet plus public site in one service.

Family summary 11: Sites is ahead on publish-time accessibility and SEO refusal if the stated SLO behavior is implemented.

Family summary 12: Sites is ahead on auditability if AsyncAPI events and audit-chain integration are implemented.

Family summary 13: Sites is not yet ahead operationally because no source, tests, context modules, or OS manifest prove the claims.

Family summary 14: The local broad competitor matrix is useful but should not replace this top-3 union view.

Family summary 15: The local benchmark document must be superseded or rewritten because it uses retired tier terminology.

Family summary 16: The migration playbook family should add Squarespace-specific extraction and import paths.

Family summary 17: The tenant-class model must be added before quota comparisons with Wix or trial behavior can be coherent.

Family summary 18: Deployment contexts are an Oyatie differentiator, not a Webflow/Squarespace/Wix feature; their absence is still a blocker because the user made them canonical.

Family summary 19: The OpenTofu absence blocks any claim that sites can be deployed consistently across contexts.

Family summary 20: The OS manifest absence blocks on-prem and colo credibility even if hosted public-cloud behavior is otherwise specified.

## Section 6 - Headline Gap Analysis

Headline gap 1: Manifest-to-PRD mismatch.

Evidence: PRD names 11 BCs; manifest names 7.

User impact: builders can implement a smaller service than the product requires.

Counterpart impact: Webflow, Squarespace, and Wix all require theme, navigation, URL routing, and SEO as first-class surfaces.

Remediation: align manifest with PRD and contracts before using manifest for codegen or gates.

Headline gap 2: Visual designer maturity.

Evidence: local matrix marks Webflow-class visual layout designer as M04-onward in `microservices/sites/competitor-parity-matrix.md` lines 48-53.

User impact: Webflow replacement claims are premature for designer-led use cases.

Counterpart impact: Webflow's strongest differentiator remains unclosed.

Remediation: define a visual-layout workstream or bound the M03 claim to structured editor plus tokens.

Headline gap 3: Ecommerce completion.

Evidence: PRD only requires ecommerce stub in `microservices/sites/PRD.md` line 56; local matrix says full storefront is later in `microservices/sites/competitor-parity-matrix.md` lines 50-53.

User impact: Squarespace and Wix storefront displacement is partial.

Counterpart impact: product catalog, checkout, inventory, and product media import need real scope.

Remediation: decide whether storefront belongs in sites or T-G fintech bridge for the M03/M04 boundary.

Headline gap 4: Tenant-class quota model.

Evidence: production policy uses `tenant_tier` in `microservices/sites/policy/tenant-scope.cedar` lines 142-153.

User impact: trial, paid, and revenue_share behavior cannot be compared to Wix plan quotas or demo infrastructure caps.

Counterpart impact: Wix's quota clarity is stronger than sites' current tenant-class clarity.

Remediation: add tenant_class semantics and per-class usage caps without feature-quality degradation.

Headline gap 5: Deployment context implementation.

Evidence: only Helm and Kustomize exist under sites IaC.

User impact: deployment promises are unsupported for all six contexts.

Counterpart impact: deployment portability is Oyatie-specific and should be a differentiator, but it is currently a gap.

Remediation: add OpenTofu modules per context and keep Helm/Kustomize as subordinate payloads.

Headline gap 6: Squarespace migration path.

Evidence: no Squarespace-specific migration playbook was found.

User impact: top-3 union migration coverage is incomplete.

Counterpart impact: Squarespace pages, products, galleries, redirects, image metadata, and style tokens need a path.

Remediation: author a dedicated Squarespace migration playbook with extraction, mapping, validation, and rollback.

Headline gap 7: Source/test proof.

Evidence: no `src` or `tests` path exists under the service.

User impact: many claims are specification claims, not behavior evidence.

Counterpart impact: parity cannot be demonstrated without runnable implementation or tests.

Remediation: create Rust crates and tests consistent with the manifest once the manifest is corrected.

Headline gap 8: API and rate-limit contract.

Evidence: OpenAPI defines core schemas but no explicit rate-limit headers or cap errors in the inspected schema lines.

User impact: integrations do not know retry or quota behavior.

Counterpart impact: Webflow, Squarespace, and Wix all publish at least some request-limit behavior.

Remediation: add rate-limit response headers, error shapes, and tenant-class cap events.

Headline gap 9: Old tier artifacts still shape parity claims.

Evidence: the old benchmark and tier matrix contain retired terminology.

User impact: future agents may continue feature gating by retired terms.

Counterpart impact: comparison becomes pricing-tier-driven instead of industry-leader-grade uniform.

Remediation: retire tier matrix and benchmark tiers in Wave 15J; use this document and the performance benchmark as replacement shape.

Headline gap 10: README absence.

Evidence: inventory found no README.

User impact: cold-start implementation and audit handoff require reading too many files.

Counterpart impact: not a product parity issue, but a delivery quality issue.

Remediation: create a README after canonical decisions settle.

## Section 7 - Additive Surface

Additive surface 1: Sites can unify public websites and intranets under one authoring substrate.

Additive surface 2: Sites can use the same CRDT collaboration model as docs, sheets, slides, and workflow-studio.

Additive surface 3: Sites can bind CMS collection entries to the tenant ontology rather than keeping them inside a website-only silo.

Additive surface 4: Sites can emit signed audit-chain records for site publication, page publication, domain binding, certificate issuance, CMS updates, form binding, analytics consent, AI acceptance, and legal hold.

Additive surface 5: Sites can refuse publish when WCAG or SEO correctness fails, raising the default quality bar above advisory-only competitors.

Additive surface 6: Sites can run in six deployment contexts after OpenTofu modules exist.

Additive surface 7: Sites can provide OCI Always Free profile infrastructure for demo_trial tenants without weakening product quality.

Additive surface 8: Sites can map paid tenants to per-seat and usage-billed publishing workloads.

Additive surface 9: Sites can map revenue_share tenants to commerce and marketplace revenue attribution.

Additive surface 10: Sites can avoid feature stratification and still impose trial usage caps.

Additive surface 11: Sites can integrate with compliance packs for public-sector, healthcare, finance, and regional data-residency obligations.

Additive surface 12: Sites can treat custom-domain, TLS, and legal-hold operations as audited control-plane events rather than dashboard-only changes.

Additive surface 13: Sites can expose tenant-safe webhooks through Workflow instead of one-off plugin scripts.

Additive surface 14: Sites can use Cedar policy for publish, AI, public read, editor isolation, auditor scope, and CI scope.

Additive surface 15: Sites can let workflow automation act on site events without giving direct database access.

Additive surface 16: Sites can make privacy-preserving analytics default instead of an optional integration.

Additive surface 17: Sites can define deployment-context overlays for performance and cost instead of marketing plan names.

Additive surface 18: Sites can add import validation that compares source-site URL maps, media hashes, SEO metadata, and redirect coverage.

Additive surface 19: Sites can produce pack-specific accessibility overlays, such as KR, EU public-sector, and Section 508 requirements.

Additive surface 20: Sites can use audit evidence to prove migration, publish, rollback, and legal-hold workflows after implementation lands.

## Section 8 - Union Backlog

Backlog item 1: Align manifest BCs and layers with PRD plus contracts.

Backlog item 2: Remove stale dependency from live manifest surfaces.

Backlog item 3: Add tenant-class behavior document or manifest section for `demo_trial`, `paid`, and `revenue_share`.

Backlog item 4: Rewrite production Cedar gates from tenant-tier to tenant_class plus cap/contract claims.

Backlog item 5: Add rate-limit headers and quota error schemas to OpenAPI.

Backlog item 6: Add API rate-limit and publish-rate documentation that can be compared to Webflow and Squarespace.

Backlog item 7: Add Squarespace migration playbook.

Backlog item 8: Extend Wix migration coverage to CMS item limits, field size, media handling, and app-collection exclusions.

Backlog item 9: Add Webflow migration coverage for collection lists, nested collection lists, and publish-rate behavior.

Backlog item 10: Add a visual-designer implementation plan or explicitly bound M03 to structured editor.

Backlog item 11: Add full storefront boundary decision for Squarespace/Wix parity.

Backlog item 12: Add OpenTofu context modules for all supported contexts.

Backlog item 13: Add OCI Always Free profile with demo_trial caps.

Backlog item 14: Add `supported-oses.json`.

Backlog item 15: Add source crates and tests for core paths.

Backlog item 16: Add publish, rollback, domain, and CMS import tests.

Backlog item 17: Add accessibility and SEO correctness tests tied to SLOs.

Backlog item 18: Rewrite old benchmark doc away from retired terms.

Backlog item 19: Retire `tenant-class/tier-matrix.md`.

Backlog item 20: Create README after the canonical repairs land.

