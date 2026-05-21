# Sites Performance Benchmark Numbers - 2026-05-20

Target microservice: `sites`

Counterparts: Webflow / Squarespace / Wix

Audit owner: single-agent ownership-coherence audit

Scope: benchmark numbers, target numbers, and comparison narrative for sites.

Retired model note: this benchmark uses one industry-leader target set, deployment-context overlays, and tenant_class overlays. It does not define a plan ladder.

## Citation Anchor Block

1. Local service PRD performance targets: `microservices/sites/PRD.md:76-85`.
2. Local service SLO evidence: `microservices/sites/slos/page-render.openslo.yaml:16-38`, `microservices/sites/slos/static-asset.openslo.yaml:16-38`, `microservices/sites/slos/publish-pipeline.openslo.yaml:16-38`.
3. Local service capacity model: `microservices/sites/capacity-model.md:51-65`.
4. Canonical deployment and infrastructure constraints: `specs/master-plan-sequencing.json:704-775`, `specs/master-plan-sequencing.json:857-864`.
5. Canonical direction for nine-dimension audit and severity: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3829-4121`.
6. Webflow hosting response and scale statement: official Webflow hosting overview, source URL `https://help.webflow.com/hc/en-us/articles/33961342422547-Webflow-hosting-overview`, read on 2026-05-21.
7. Webflow API and publish limits: official Webflow Data API rate-limits page, source URL `https://developers.webflow.com/data/v2.0.0/reference/rate-limits`, read on 2026-05-21.
8. Squarespace Commerce API and page/product limits: official Squarespace developer and support pages, source URLs `https://developers.squarespace.com/commerce-apis/rate-limits`, `https://support.squarespace.com/hc/en-us/articles/206543087-Page-limits`, and `https://support.squarespace.com/hc/en-us/articles/205811338-Adding-products-to-your-store`, read on 2026-05-21.
9. Wix CMS and API-limit posture: official Wix support/developer pages, source URLs `https://support.wix.com/en/article/cms-understanding-collection-storage-limits-and-quotas` and `https://dev.wix.com/docs/rest/articles/get-started/rate-limits`, read on 2026-05-21.
10. Core Web Vitals thresholds: official web.dev article `https://web.dev/articles/defining-core-web-vitals-thresholds`, lines 119-129 and 156-162 in the retrieved page.

## Methodology Disclosure

This document is a planning benchmark, not a live load-test result.

Numbers fall into four evidence classes.

Class 1: official counterpart limits.

Class 2: public counterpart performance statements.

Class 3: local service targets already written into Sites artifacts.

Class 4: internal planning estimates from the existing sites benchmark artifact.

When a number is a vendor-published limit, the row says `source: official`.

When a number is a local target, the row says `source: local target`.

When a number is estimated from the existing local benchmark document, the row says `source: estimated from local benchmark`.

When a number is an overlay constraint rather than a product target, the row says `source: canonical constraint`.

No live WebPageTest, Lighthouse, k6, wrk, vegeta, or synthetic browser run was executed in this audit.

No private competitor telemetry was used.

No unpublished Oyatie production telemetry was assumed.

The comparison goal is conservative: define the smallest target set that can credibly meet or beat the top public counterpart surface once implemented and validated.

## §1 Methodology

### §1.1 Benchmark Dimensions

Dimension 1: cached public page read latency.

Dimension 2: server-rendered page read latency.

Dimension 3: static asset latency.

Dimension 4: CMS query latency.

Dimension 5: site search latency.

Dimension 6: publish pipeline latency.

Dimension 7: domain and TLS operation latency.

Dimension 8: image optimization latency.

Dimension 9: AI-assisted page build latency.

Dimension 10: API request rate.

Dimension 11: CMS item and page scale.

Dimension 12: product/catalog scale.

Dimension 13: Core Web Vitals thresholds.

Dimension 14: accessibility and SEO correctness.

Dimension 15: availability, RTO, and RPO.

Dimension 16: capacity envelope under sustained public traffic.

Dimension 17: deployment-context constraint behavior.

Dimension 18: tenant_class constraint behavior.

### §1.2 Test Workloads

Workload A: anonymous cached homepage request with CDN edge cache warm.

Workload B: anonymous dynamic page request that resolves navigation, CMS blocks, theme assets, and permissions.

Workload C: authenticated dashboard route loading site list, recent publishes, domain health, and accessibility alerts.

Workload D: CMS collection query returning 50 records with preview metadata.

Workload E: search query against a published site with 100,000 indexed pages.

Workload F: publish 100 changed pages after content edits.

Workload G: publish 10,000 changed pages after template or theme update.

Workload H: issue or renew TLS certificate for a custom domain.

Workload I: transform and optimize one original hero image into responsive variants.

Workload J: AI-assisted page generation from a structured brief.

Workload K: import site content from a counterpart export or scrape-assisted migration bundle.

Workload L: accessibility and SEO validation across a representative site map.

Workload M: API burst for editor and integration operations.

Workload N: sustained public traffic surge during launch or campaign.

### §1.3 Operating System and Architecture Disclosure

The benchmark target is OS-neutral at the service contract level.

The service currently lacks a `supported-oses.json` artifact under `microservices/sites/`, which is a coherence gap recorded in the main audit.

Canonical OS scope remains the project-wide supported OS matrix in `specs/master-plan-sequencing.json:777-815`.

The service target assumes Linux server execution for Kubernetes contexts until the service has explicit OS support evidence.

The service target assumes x86_64 and arm64 infrastructure where the deployment context supports both.

The service target assumes Rust backend binaries and web frontend delivery through the approved web stack.

The service target does not assume Python, Node application servers, Ruby, Go, Java, Scala, Groovy, PHP, or F# runtime components.

### §1.4 Deployment Context Disclosure

The canonical deployment contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` per `specs/master-plan-sequencing.json:704-746`.

The current sites microservice path does not yet contain OpenTofu modules for those six contexts.

Existing Sites IaC is Helm and Kustomize only, as shown by `microservices/sites/implementation-plans/IP-001-iac-bootstrap.md:16-27`.

Therefore the benchmark uses a target set plus overlay constraints, not a claim that all contexts are implemented.

The `guest-on-oci` overlay includes the OCI Always Free profile when tenant_class is `demo_trial`.

The `oyatie-public-cloud` overlay assumes Oyatie-operated capacity with elasticity and contractual operating control.

The `guest-on-aws` overlay assumes customer-owned AWS substrate hidden behind the Oyatie deployment contract.

The `guest-on-oci` overlay assumes customer-owned OCI substrate or demo_trial infrastructure depending on tenant_class.

The `on-prem` overlay assumes facility, network, storage, and certificate-management constraints are supplied by the customer or integrator.

The `colo` overlay assumes remote-hands, power, network, and hardware replacement windows affect incident response and capacity expansion.

The `oyatie-as-cloud-provider` overlay assumes Oyatie can expose provider-grade capacity reservations and service-provider reporting.

### §1.5 Tenant Class Disclosure

The benchmark uses three tenant_class overlays: `demo_trial`, `paid`, and `revenue_share`.

`demo_trial` receives the same product-quality bar but hard usage caps, time caps, best-effort SLO, no compliance packs, and no BYOK.

`paid` receives the same product-quality bar with contractual SLO, compliance packs where applicable, BYOK where applicable, and scale proportional to purchased capacity and usage billing.

`revenue_share` receives the same product-quality bar with at-cost or zero-margin substrate assumptions and revenue-metered capacity governance.

The current service artifacts do not yet express these three tenant_class semantics as a first-class API, policy, manifest, or SLO dimension.

That adoption gap is recorded in the coherence audit.

### §1.6 Interpretation Rules

The canonical target is the desired industry-leader service number before deployment overlay constraints.

Deployment overlays narrow or qualify that target when the substrate is intentionally constrained.

Tenant_class overlays cap allocation, not quality.

When a demo_trial tenant is capped, the page, editor, accessibility, and publish experience should still meet the same correctness bar inside the cap.

When a paid tenant funds more capacity, the service should scale by increasing allocation rather than weakening correctness.

When a revenue_share tenant grows, the service should tie capacity expansion to gross-revenue metering and at-cost infrastructure policy.

Performance comparison labels mean:

`ahead` means the target is stronger than the best cited counterpart number.

`parity` means the target is equivalent or materially close.

`catch-up` means the target is below the best counterpart and needs improvement before maturity claim.

`not yet claimable` means the target exists but implementation or live evidence is missing.

## §2 Counterpart Numbers

### §2.1 Webflow Numbers

W-01: Hosting response time claim is less than 100 ms.

Source: official Webflow hosting overview.

Interpretation: Webflow positions managed hosting as globally distributed and fast enough for very high public traffic.

W-02: Hosting scale statement says Webflow can scale to millions of page views per day.

Source: official Webflow hosting overview.

Interpretation: the public claim is qualitative plus a daily-scale order of magnitude, not a guaranteed per-site RPS number.

W-03: Webflow API limit for Starter and Basic site plans is 60 requests per minute.

Source: official Webflow Data API rate-limits page.

Interpretation: this is an integration API limit, not public page-serving throughput.

W-04: Webflow API limit for CMS, Ecommerce, and Business site plans is 120 requests per minute.

Source: official Webflow Data API rate-limits page.

Interpretation: higher commercial usage still has a conservative published default API ceiling.

W-05: Webflow Enterprise API limit is custom.

Source: official Webflow Data API rate-limits page.

Interpretation: the public page does not expose a universal Enterprise numeric ceiling.

W-06: Webflow successful Site Publish operations are limited to one successful publish per minute.

Source: official Webflow Data API rate-limits page.

Interpretation: publish throughput is explicitly constrained in the public API surface.

W-07: Webflow cached content delivery is described as unbounded by the API-limit table.

Source: official Webflow Data API rate-limits page.

Interpretation: cached public delivery is separated from API quota and should not be benchmarked as API traffic.

W-08: Webflow dynamic content supports 40 collection lists per page.

Source: official Webflow dynamic content limits page.

Interpretation: the visual page surface has a per-page dynamic-query complexity ceiling.

W-09: Webflow dynamic content supports 10 nested collection lists per page.

Source: official Webflow dynamic content limits page.

Interpretation: nested CMS composition is capped for runtime and editor predictability.

W-10: Webflow dynamic content supports 100 collection items per collection list before pagination.

Source: official Webflow dynamic content limits page.

Interpretation: large collections are expected to paginate rather than fully render into a page.

W-11: Local benchmark estimate for Webflow static-site generation page read is p50 60 ms and p99 180 ms.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:21`.

Interpretation: this is not a fresh external measurement; it is the existing local audit estimate.

W-12: Local benchmark estimate for Webflow incremental-static-regeneration-like read is p50 380 ms and p99 920 ms.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:39`.

Interpretation: this models cache-miss or regeneration behavior.

W-13: Local benchmark estimate for Webflow static asset delivery is p50 40 ms and p99 120 ms.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:52`.

Interpretation: CDN-warm static delivery is the strongest counterpart surface.

W-14: Local benchmark estimate for Webflow publish pipeline is p50 12 seconds and p99 24 seconds.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:64`.

Interpretation: this is materially slower than the Sites PRD target and therefore an opportunity if implementation proves it.

W-15: Local benchmark estimate for Webflow default accessibility-template pass rate is 94 percent.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:80`.

Interpretation: Oyatie can target 100 percent only if generated components, templates, and editor guardrails enforce accessibility by construction.

### §2.2 Squarespace Numbers

S-01: Squarespace Commerce API global limit is 300 requests per minute.

Source: official Squarespace Commerce API rate-limits page.

Interpretation: this equates to 5 requests per second across the documented API limit.

S-02: Squarespace Commerce API page states 5 requests per second.

Source: official Squarespace Commerce API rate-limits page.

Interpretation: the per-second ceiling is explicit and conservative.

S-03: Squarespace Create Order API with API key is limited to 100 requests per hour per website.

Source: official Squarespace Commerce API rate-limits page.

Interpretation: order creation can have a much lower write ceiling than general API reads.

S-04: Squarespace current plans can hold up to 1,000 pages.

Source: official Squarespace page-limits support page.

Interpretation: page-count scale is public and finite.

S-05: Squarespace recommends keeping page count under 400 for usability and performance.

Source: official Squarespace page-limits support page.

Interpretation: the official hard ceiling and recommended operating range differ.

S-06: Squarespace gallery sections can contain up to 250 images or videos.

Source: official Squarespace page-limits support page.

Interpretation: media-heavy page composition has an explicit per-section ceiling.

S-07: Squarespace recommends no more than 20 sections per page.

Source: official Squarespace page-limits support page.

Interpretation: editor and runtime performance depend on composition complexity.

S-08: Squarespace version 7.1 store pages can include up to 10,000 products.

Source: official Squarespace products support page.

Interpretation: product catalog scale is significant but finite.

S-09: Squarespace version 7.0 store pages can include up to 200 products.

Source: official Squarespace products support page.

Interpretation: older surface constraints remain relevant for migration planning.

S-10: Squarespace product variant, image, and SKU ceiling is 250 per product.

Source: official Squarespace products support page.

Interpretation: product-model complexity has a per-product ceiling.

S-11: Local benchmark estimate for Squarespace static-site page read is p50 80 ms and p99 240 ms.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:24`.

Interpretation: this is an estimated baseline for public page reads.

S-12: Local benchmark estimate for Squarespace default accessibility-template pass rate is 88 percent.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:80`.

Interpretation: accessibility parity requires template and component enforcement, not only documentation.

### §2.3 Wix Numbers

X-01: Wix CMS Light plan item limit is 1,500 items.

Source: official Wix CMS collection storage limits page.

Interpretation: the lowest paid CMS item envelope is small for content-heavy sites.

X-02: Wix CMS Core plan item limit is 4,000 items.

Source: official Wix CMS collection storage limits page.

Interpretation: small-business CMS usage is still heavily bounded.

X-03: Wix CMS Business plan item limit is 20,000 items.

Source: official Wix CMS collection storage limits page.

Interpretation: mid-market CMS usage is bounded but larger than basic website usage.

X-04: Wix CMS Business Elite plan item limit is 10,000,000 items.

Source: official Wix CMS collection storage limits page.

Interpretation: Wix publishes a very high upper CMS item limit for its largest packaged website plan.

X-05: Wix item size limit is 512 KB per item.

Source: official Wix CMS collection storage limits page.

Interpretation: individual content records have a fixed storage ceiling.

X-06: Wix database storage limit for lower paid CMS plans includes 10 GB.

Source: official Wix CMS collection storage limits page.

Interpretation: storage, not just item count, constrains CMS scale.

X-07: Wix database storage limit for high-end paid CMS plans includes 100 GB.

Source: official Wix CMS collection storage limits page.

Interpretation: storage capacity can become the practical limiter before public page traffic.

X-08: Wix Studio free sites can have 10,000 CMS items.

Source: official Wix CMS collection storage limits page.

Interpretation: some free-site envelopes exceed smaller paid packaged-site CMS limits depending on product surface.

X-09: Wix non-Studio free sites can have 1,000 CMS items.

Source: official Wix CMS collection storage limits page.

Interpretation: free-site envelope differs by authoring surface.

X-10: Wix REST API rate-limit documentation says requests over limit receive HTTP 429 and callers should wait about one minute before retrying.

Source: official Wix REST API rate-limits page.

Interpretation: the public article explains behavior but does not publish one universal numeric ceiling for every API.

X-11: Local benchmark estimate for Wix static-site page read is p50 120 ms and p99 320 ms.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:22`.

Interpretation: this gives Oyatie a clear static-read latency target to beat.

X-12: Local benchmark estimate for Wix static asset delivery is p50 60 ms and p99 160 ms.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:53`.

Interpretation: asset performance target should be at least as strong as this estimate.

X-13: Local benchmark estimate for Wix default accessibility-template pass rate is 62 percent.

Source: estimated from `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:78`.

Interpretation: accessibility is a potential differentiator if Oyatie makes the bar mechanical.

### §2.4 Cross-Counterpart Observations

Observation 1: Webflow exposes the strongest public hosting-performance claim among the three counterparts.

Observation 2: Squarespace exposes clearer hard page and product ceilings than Webflow in the cited support pages.

Observation 3: Wix exposes the largest CMS item ceiling among the cited counterpart docs.

Observation 4: API limits are not equivalent to public page-serving throughput.

Observation 5: Publish limits are often stricter than read limits.

Observation 6: The current Sites PRD target for publish latency is more ambitious than the local Webflow publish estimate.

Observation 7: CMS item ceilings matter for migration and market positioning.

Observation 8: Accessibility percentages in the local benchmark are estimates, not fresh test results.

Observation 9: Core Web Vitals give a vendor-independent user-experience floor.

Observation 10: Oyatie needs live tests before any production maturity claim.

## §3 Oyatie Target Numbers

### §3.1 Canonical Industry-Leader Target Set

O-01: Public cached page read latency target is p50 <= 30 ms, p95 <= 120 ms, p99 <= 250 ms.

Source: local target derived from `microservices/sites/PRD.md:76-79` and tightened for cached edge reads.

Canonical target: ahead of local Webflow static-read estimate at p50 and p99.

Deployment overlay: all six contexts should meet the target when edge cache is deployed and warm.

tenant_class overlay: demo_trial may cap traffic volume, but cache-hit latency inside cap remains the same target.

O-02: Server-rendered page read latency target is p50 <= 50 ms, p95 <= 200 ms, p99 <= 400 ms.

Source: local target from `microservices/sites/PRD.md:76-79`.

Canonical target: ahead of the local Webflow regeneration estimate and competitive with cached counterpart reads.

Deployment overlay: on-prem and colo require local facility network measurements before claim.

tenant_class overlay: paid and revenue_share tenants scale allocation; demo_trial receives hard concurrent-request caps.

O-03: Static asset delivery target is p50 <= 20 ms, p95 <= 100 ms, p99 <= 200 ms.

Source: local target from `microservices/sites/PRD.md:80-81` and `microservices/sites/slos/static-asset.openslo.yaml:16-38`.

Canonical target: ahead at p50, parity-to-ahead at p99 against local Webflow and Wix estimates.

Deployment overlay: edge presence and object-store replication determine whether context can claim this target.

tenant_class overlay: demo_trial may cap storage and transfer, not asset correctness.

O-04: CMS query target is p50 <= 50 ms, p95 <= 150 ms, p99 <= 300 ms.

Source: local target from `microservices/sites/PRD.md:81-82`.

Canonical target: industry-leader-grade for editor responsiveness and dynamic pages.

Deployment overlay: guest-on-oci demo_trial must degrade through usage caps before weakening query correctness.

tenant_class overlay: paid and revenue_share tenants require scaling rules for collection count and query volume.

O-05: Site search target is p50 <= 100 ms, p95 <= 300 ms, p99 <= 600 ms.

Source: local target from `microservices/sites/PRD.md:81-82`.

Canonical target: sufficient for public site search and editor search.

Deployment overlay: on-prem and colo require local search-index placement and storage IOPS evidence.

tenant_class overlay: demo_trial can cap indexed pages and queries per day.

O-06: Publish 100 pages target is p50 <= 2 seconds, p95 <= 5 seconds, p99 <= 10 seconds.

Source: local target from `microservices/sites/PRD.md:82-83` and `microservices/sites/slos/publish-pipeline.openslo.yaml:16-38`.

Canonical target: ahead of local Webflow publish estimate if implemented.

Deployment overlay: constrained contexts may queue large publishes, but must expose queue state and expected completion.

tenant_class overlay: demo_trial may cap publish frequency; paid and revenue_share scale with allocated workers.

O-07: Publish 10,000 pages target is p50 <= 60 seconds, p95 <= 180 seconds, p99 <= 300 seconds.

Source: local target derived from the PRD publish target and capacity model worker assumptions in `microservices/sites/capacity-model.md:116-131`.

Canonical target: necessary for large marketing sites and migrations.

Deployment overlay: OCI Always Free profile cannot claim this at full volume without explicit queue caps.

tenant_class overlay: paid and revenue_share tenants can buy or earn capacity expansion; demo_trial is capped.

O-08: Custom domain DNS verification target is p50 <= 10 seconds, p95 <= 60 seconds, p99 <= 5 minutes after DNS propagation is visible.

Source: local target derived from domain workflows in `microservices/sites/PRD.md:59-60`.

Canonical target: parity with strong managed-site expectations.

Deployment overlay: external DNS providers and customer TTLs are outside direct Oyatie control.

tenant_class overlay: demo_trial may limit number of custom domains.

O-09: TLS certificate issuance or renewal target is p50 <= 15 seconds, p95 <= 30 seconds, p99 <= 60 seconds after validation is ready.

Source: local target derived from custom-domain and security requirements in `microservices/sites/PRD.md:89-97`.

Canonical target: industry-leader-grade for custom domain onboarding.

Deployment overlay: on-prem and colo need outbound ACME path, CA policy, and secret-storage integration.

tenant_class overlay: paid and revenue_share can use BYOK where allowed; demo_trial does not.

O-10: Image optimization target is p50 <= 500 ms, p95 <= 1 second, p99 <= 2 seconds for a standard 4K image derivative set.

Source: local target from `microservices/sites/PRD.md:83-84`.

Canonical target: competitive for editor preview and publish pipelines.

Deployment overlay: CPU-constrained environments must queue image jobs before dropping quality.

tenant_class overlay: demo_trial can cap daily original uploads and concurrent transforms.

O-11: AI-assisted page build target is p50 <= 2.5 seconds, p95 <= 5 seconds, p99 <= 10 seconds for a bounded structured brief.

Source: local target from `microservices/sites/PRD.md:84-85`.

Canonical target: fast enough for interactive authoring.

Deployment overlay: contexts without approved model access or local inference capacity must disable or defer this capability with a clear policy surface.

tenant_class overlay: demo_trial can cap daily generation count; paid and revenue_share require usage metering.

O-12: Accessibility conformance target is 100 percent pass on generated templates for WCAG 2.2 AA automated gates plus manual-review queue for non-automatable criteria.

Source: local target from `microservices/sites/PRD.md:61-65`, `microservices/sites/PRD.md:89-97`, and local benchmark estimates at `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:72-84`.

Canonical target: ahead of all cited local counterpart accessibility estimates.

Deployment overlay: no context should weaken accessibility correctness.

tenant_class overlay: no tenant_class should weaken accessibility correctness.

O-13: SEO validation target is 100 percent pass for generated sitemap, canonical URL, metadata, robots policy, structured data, redirect, and noindex rules covered by the service contract.

Source: local target from `microservices/sites/PRD.md:61-65`.

Canonical target: industry-leader-grade correctness target, not a speed metric.

Deployment overlay: all contexts require the same generated artifact correctness.

tenant_class overlay: no tenant_class should weaken SEO correctness.

O-14: Core Web Vitals target is LCP <= 2.5 seconds, INP <= 200 ms, and CLS <= 0.1 at the 75th percentile for public pages.

Source: official web.dev thresholds, retrieved lines 119-129 and 156-162.

Canonical target: parity with the web-wide quality threshold, but service-specific page and asset targets should be much faster where Oyatie controls the stack.

Deployment overlay: field data must be segmented by context before maturity claim.

tenant_class overlay: demo_trial caps traffic and site size but not user-experience correctness inside allowed use.

O-15: Read availability target is 99.99 percent monthly for public pages.

Source: local target derived from `microservices/sites/PRD.md:101-104`.

Canonical target: competitive for business-critical marketing sites.

Deployment overlay: paid and revenue_share claims depend on deployment context readiness; demo_trial is best effort.

tenant_class overlay: contractual SLO applies to paid and eligible revenue_share tenants, not demo_trial.

O-16: Write and publish availability target is 99.95 percent monthly.

Source: local target derived from `microservices/sites/PRD.md:101-104`.

Canonical target: high enough for editor and launch workflows.

Deployment overlay: on-prem and colo need local failure-mode contracts.

tenant_class overlay: demo_trial can be best effort while preserving data safety.

O-17: RTO target is <= 15 minutes for core public read recovery.

Source: local target from `microservices/sites/PRD.md:109-114` and failure-mode expectations in `microservices/sites/failure-modes.md`.

Canonical target: strong managed-service expectation.

Deployment overlay: customer-owned contexts need tested restore paths before claim.

tenant_class overlay: demo_trial may have best-effort recovery timing, but paid and revenue_share require contractual recovery posture.

O-18: RPO target is <= 60 seconds for content writes.

Source: local target from `microservices/sites/PRD.md:109-114`.

Canonical target: strong enough for editor confidence and migration safety.

Deployment overlay: substrate replication and backup modules must prove the number in each context.

tenant_class overlay: data-loss tolerance should not be relaxed silently for any tenant_class.

O-19: Baseline active-sites envelope is 50,000 active sites and stretch envelope is 500,000 active sites.

Source: local target from `microservices/sites/capacity-model.md:51-65`.

Canonical target: enough for public-cloud and service-provider ambitions.

Deployment overlay: OCI Always Free profile is far below the full stretch envelope and must be capped.

tenant_class overlay: demo_trial allocation is intentionally small; paid and revenue_share scale with capacity policy.

O-20: Baseline origin-render envelope is 5,000 requests per second and stretch envelope is 50,000 requests per second.

Source: local target from `microservices/sites/capacity-model.md:51-65`.

Canonical target: enough to beat small packaged-site API ceilings and support major launches.

Deployment overlay: guest-on-aws, guest-on-oci, on-prem, and colo need explicit node, cache, and database sizing.

tenant_class overlay: demo_trial caps sustained origin traffic.

O-21: Baseline CMS-write envelope is 500 writes per second and stretch envelope is 5,000 writes per second.

Source: local target from `microservices/sites/capacity-model.md:51-65`.

Canonical target: far above cited counterpart default API ceilings.

Deployment overlay: storage and event-log partitions decide whether this target is claimable.

tenant_class overlay: paid and revenue_share can scale write throughput; demo_trial is capped.

O-22: Baseline search-query envelope is 5,000 queries per second and stretch envelope is 50,000 queries per second.

Source: local target from `microservices/sites/capacity-model.md:51-65`.

Canonical target: supports large public-site search.

Deployment overlay: search-index replication must be context-specific.

tenant_class overlay: demo_trial caps queries and indexed pages.

O-23: Baseline image-job envelope is 500 jobs per minute and stretch envelope is 5,000 jobs per minute.

Source: local target from `microservices/sites/capacity-model.md:51-65`.

Canonical target: enough for bulk migrations and media-heavy campaigns.

Deployment overlay: CPU, storage bandwidth, and object-store write throughput determine context claim.

tenant_class overlay: demo_trial caps image jobs; paid and revenue_share scale by worker allocation.

O-24: Baseline publish envelope is 5 publish operations per second and stretch envelope is 50 publish operations per second.

Source: local target from `microservices/sites/capacity-model.md:51-65`.

Canonical target: stronger than the cited Webflow successful publish-per-minute public API limit.

Deployment overlay: publish fanout and CDN invalidation need context-specific proof.

tenant_class overlay: demo_trial caps publish cadence; paid and revenue_share scale by worker pool.

O-25: API integration target is at least 600 authenticated operations per minute per tenant before paid capacity expansion.

Source: derived target from counterpart API limits and Sites capacity model; requires future contract ratification.

Canonical target: above Webflow 60/120 rpm and Squarespace 300 rpm public default limits.

Deployment overlay: rate limiters and quotas must be context-aware.

tenant_class overlay: demo_trial uses lower caps; paid and revenue_share can receive higher signed quotas.

### §3.2 Deployment-Context Overlays

Context overlay 1: `oyatie-public-cloud`.

Read latency: canonical target applies.

Publish latency: canonical target applies when the Sites publish worker pool is provisioned to capacity model baseline.

Capacity: baseline and stretch envelopes are claimable only after OpenTofu modules, autoscaling policy, and load-test evidence exist.

Availability: paid and revenue_share contractual SLO can apply after production runbooks and telemetry gates are in place.

Current evidence state: not yet claimable because Sites has Helm/Kustomize bootstrap but lacks context-specific OpenTofu.

Context overlay 2: `guest-on-aws`.

Read latency: canonical target applies when the customer VPC, edge, cache, and database shape meet minimum sizing.

Publish latency: canonical target applies when worker, queue, object-store, and CDN-invalidation modules are deployed.

Capacity: baseline envelope requires AWS substrate mapping hidden behind Oyatie's provider-agnostic contract.

Availability: contractual SLO depends on customer-owned account permissions and observability.

Current evidence state: not yet claimable because no `iac/guest-on-aws` module exists under Sites.

Context overlay 3: `guest-on-oci`.

Read latency: canonical target applies for paid and revenue_share tenants with adequate OCI resources.

Publish latency: canonical target applies for adequately provisioned OCI cells.

Capacity: full baseline envelope is not compatible with the demo_trial OCI Always Free profile.

Availability: paid and revenue_share tenants can target contractual SLO after OCI module and proof exist.

Current evidence state: not yet claimable because no `iac/oci-guest` or `iac/oci-guest/always-free` module exists under Sites.

Context overlay 4: `on-prem`.

Read latency: canonical target applies only after facility network, load balancer, cache, database, and storage are measured.

Publish latency: canonical target applies only if worker, queue, storage, and invalidation paths are local or sufficiently connected.

Capacity: baseline envelope requires minimum hardware profile and spare capacity.

Availability: SLO must account for customer power, network, backup, and patch windows.

Current evidence state: not yet claimable because no `iac/on-prem` module exists under Sites.

Context overlay 5: `colo`.

Read latency: canonical target applies where edge/cache placement and transit latency support it.

Publish latency: canonical target applies only after remote-hands and storage failure-mode constraints are modeled.

Capacity: capacity expansion lead time is longer than public-cloud contexts.

Availability: SLO must include facility redundancy and replacement windows.

Current evidence state: not yet claimable because no `iac/colo` module exists under Sites.

Context overlay 6: `oyatie-as-cloud-provider`.

Read latency: canonical target applies as provider-grade promise after capacity reservations and cell placement are implemented.

Publish latency: canonical target applies after provider-owned queue, cache, and fanout systems are proven.

Capacity: stretch envelope is the natural target for this context.

Availability: strongest SLO posture belongs here once implementation exists.

Current evidence state: not yet claimable because no `iac/oyatie-iaas` module exists under Sites.

### §3.3 OCI Always Free Profile Overlay

The canonical OCI Always Free profile includes 4 OCPU, 24 GB RAM, 200 GB block volume, 10 GB object/archive storage, two Autonomous Databases at 20 GB each, 10 TB egress, and 10 Mbps load balancer per `specs/master-plan-sequencing.json:857-864`.

This profile is infrastructure for demo_trial tenants, not a separate feature-quality level.

Demo public cached reads target: p50 <= 30 ms, p95 <= 120 ms, p99 <= 250 ms while cache hit rate stays high and traffic is inside cap.

Demo origin-render cap: 25 sustained requests per second before queueing or friendly limit response.

Demo active-site cap: 10 active sites per tenant.

Demo page cap: 5,000 published pages per tenant.

Demo CMS item cap: 25,000 records per tenant.

Demo storage cap: 10 GB object/archive-equivalent usage per tenant unless shared pool governance sets a lower cap.

Demo publish cadence cap: one publish operation per five minutes per tenant.

Demo image-job cap: 10 image jobs per minute per tenant.

Demo AI-build cap: 10 successful AI page builds per day per tenant and one concurrent build.

Demo API cap: 60 authenticated operations per minute per tenant.

Demo custom-domain cap: one custom domain per tenant.

Demo availability posture: best effort with transparent status and data safety.

Demo compliance posture: no compliance packs and no BYOK.

Demo upgrade behavior: paid conversion removes demo caps by moving the tenant into paid allocation policy, not by changing service correctness.

### §3.4 Tenant Class Overlays

tenant_class `demo_trial`: free or time-limited evaluation allocation.

demo_trial latency target: same as canonical inside usage caps.

demo_trial throughput target: hard capped by OCI Always Free profile and product guardrails.

demo_trial availability target: best effort, with no contractual credits.

demo_trial compliance target: no compliance packs.

demo_trial key-management target: no BYOK.

demo_trial billing target: zero charge, but usage and time caps are enforced.

tenant_class `paid`: per-seat license plus usage-based billing.

paid latency target: canonical target applies.

paid throughput target: scales with purchased capacity and usage charges.

paid availability target: contractual SLO where context implementation is proven.

paid compliance target: compliance packs allowed when applicable.

paid key-management target: BYOK allowed when applicable.

paid billing target: metered usage and seats.

tenant_class `revenue_share`: Oyatie receives a percentage of customer gross revenue.

revenue_share latency target: canonical target applies when revenue volume funds the substrate.

revenue_share throughput target: grows with revenue-linked capacity policy.

revenue_share availability target: contractual where the commercial agreement requires it.

revenue_share compliance target: compliance packs allowed when applicable and funded.

revenue_share key-management target: BYOK allowed when applicable.

revenue_share billing target: gross-revenue metering plus at-cost or zero-margin substrate accounting.

## §4 Comparison Narrative

### §4.1 Cached and Public Page Reads

Best counterpart evidence: Webflow claims less than 100 ms hosting response and local estimate p50 60 ms / p99 180 ms.

Oyatie target: p50 <= 30 ms, p95 <= 120 ms, p99 <= 250 ms for cached public reads.

Comparison: ahead at p50, parity-to-slightly-behind at p99 versus the local Webflow estimate.

Current claim state: not yet claimable.

Why: current artifacts define targets but do not include implementation, load-test evidence, or OpenTofu deployment modules for all contexts.

Required proof: edge-cache test, warm-cache test, cold-cache test, per-context network measurement, and public-page synthetic monitoring.

### §4.2 Server-Rendered Page Reads

Best counterpart evidence: Webflow local regeneration estimate p50 380 ms / p99 920 ms.

Oyatie target: p50 <= 50 ms, p95 <= 200 ms, p99 <= 400 ms.

Comparison: ahead if implemented.

Current claim state: not yet claimable.

Why: the PRD target is strong, but the architecture lacks corresponding measured code paths.

Required proof: SSR route load test, CMS query trace, cache-miss profile, and p95/p99 regression gate.

### §4.3 Static Asset Delivery

Best counterpart evidence: Webflow local static asset estimate p50 40 ms / p99 120 ms.

Oyatie target: p50 <= 20 ms, p95 <= 100 ms, p99 <= 200 ms.

Comparison: ahead at p50, parity-to-behind at p99 depending on edge design.

Current claim state: not yet claimable.

Why: static asset SLO exists, but context-specific edge/object-store modules do not.

Required proof: asset warm-cache tests by geography, invalidation test, compression test, and object-store failover test.

### §4.4 CMS Query and Dynamic Content Scale

Best counterpart evidence: Wix publishes CMS item ceilings up to 10,000,000 items; Webflow publishes dynamic per-page limits.

Oyatie target: p50 <= 50 ms, p95 <= 150 ms, p99 <= 300 ms for CMS queries, with baseline 500 writes per second.

Comparison: ahead on write-throughput target, not yet comparable on item-count ceiling.

Current claim state: not yet claimable.

Why: current artifacts do not define a tested CMS item maximum for Sites.

Required proof: collection-size tests, pagination tests, index tests, and migration-scale tests.

### §4.5 Search

Best counterpart evidence: none of the cited public counterpart pages provide a universal search-latency benchmark.

Oyatie target: p50 <= 100 ms, p95 <= 300 ms, p99 <= 600 ms and baseline 5,000 search queries per second.

Comparison: target is industry-leader-grade, but external parity cannot be proven from cited public sources alone.

Current claim state: not yet claimable.

Why: no implementation proof and no counterpart apples-to-apples public number.

Required proof: query latency by corpus size, index-refresh latency, multi-tenant isolation test, and degraded-index recovery test.

### §4.6 Publish Pipeline

Best counterpart evidence: Webflow successful Site Publish API is limited to one successful publish per minute, and local benchmark estimate is p50 12 seconds / p99 24 seconds.

Oyatie target: publish 100 pages p50 <= 2 seconds, p95 <= 5 seconds, p99 <= 10 seconds.

Comparison: ahead if implementation proves the number.

Current claim state: not yet claimable.

Why: publish SLO exists, but evidence path from IP-015 is absent in current inventory.

Required proof: publish fanout benchmark, incremental publish test, full-site publish test, rollback test, and CDN invalidation evidence.

### §4.7 Domain and TLS Operations

Best counterpart evidence: counterpart docs expose custom-domain surfaces but not universal public latency numbers in the cited sources.

Oyatie target: DNS verification p95 <= 60 seconds after visible propagation; TLS issuance p95 <= 30 seconds after validation readiness.

Comparison: parity target for managed-site expectations.

Current claim state: not yet claimable.

Why: no ACME or DNS operation benchmark evidence was found under Sites.

Required proof: ACME staging test, DNS provider matrix, renewal test, secret-rotation test, and failed-validation recovery test.

### §4.8 Image Optimization

Best counterpart evidence: counterpart public docs in this audit do not expose universal image-transform latency.

Oyatie target: p50 <= 500 ms, p95 <= 1 second, p99 <= 2 seconds for a standard derivative set.

Comparison: strong target, external parity not provable from cited public sources alone.

Current claim state: not yet claimable.

Why: target exists in PRD but implementation evidence is absent.

Required proof: image corpus benchmark, format-conversion test, responsive-variant test, and queue saturation test.

### §4.9 AI-Assisted Page Build

Best counterpart evidence: counterpart public docs in this audit do not expose universal page-generation latency for comparable AI workflows.

Oyatie target: p50 <= 2.5 seconds, p95 <= 5 seconds, p99 <= 10 seconds.

Comparison: target is acceptable for interactive authoring, but counterpart comparison is not conclusive.

Current claim state: not yet claimable.

Why: current policy has production tenant-tier gating language and no tenant_class adoption; AI access needs policy cleanup.

Required proof: prompt-bounded generation benchmark, refusal-path test, policy test, and usage-metering test.

### §4.10 API Rate and Integration Operations

Best counterpart evidence: Webflow defaults to 60 or 120 requests per minute for cited API classes; Squarespace Commerce API publishes 300 requests per minute and 5 requests per second.

Oyatie target: at least 600 authenticated operations per minute per tenant before paid capacity expansion.

Comparison: ahead on target.

Current claim state: not yet claimable.

Why: OpenAPI/AsyncAPI/proto contracts exist, but no tenant_class quota contract or rate-limit implementation evidence was found.

Required proof: rate-limiter contract, quota policy, abuse protection, idempotency behavior, and per-context load test.

### §4.11 CMS Item and Page Scale

Best counterpart evidence: Wix publishes up to 10,000,000 CMS items; Squarespace publishes up to 1,000 pages on current plans and up to 10,000 products on version 7.1 store pages.

Oyatie target: baseline 50,000 active sites, but current docs do not define a maximum CMS item count per site.

Comparison: catch-up on explicit CMS item ceiling documentation.

Current claim state: not yet claimable.

Why: Sites needs explicit collection, item, page, product, and media ceilings.

Required proof: capacity model extension and benchmark data by item count.

### §4.12 Accessibility and SEO Correctness

Best counterpart evidence: local benchmark estimates Webflow 94 percent, Squarespace 88 percent, and Wix 62 percent accessibility pass rates for default templates.

Oyatie target: 100 percent automated gate pass for generated templates plus manual-review queue for non-automatable criteria.

Comparison: ahead if enforced mechanically.

Current claim state: not yet claimable.

Why: target is documented, but mechanical test inventory is absent.

Required proof: axe or equivalent automated gate, template corpus, manual-review workflow, and regression threshold.

### §4.13 Core Web Vitals

Best external threshold: LCP <= 2.5 seconds, INP <= 200 ms, CLS <= 0.1 at p75.

Oyatie target: meet those field thresholds and keep controlled service-side latencies far below them.

Comparison: parity on web-wide quality thresholds, with room to lead on server-side latency.

Current claim state: not yet claimable.

Why: no field or lab Core Web Vitals evidence was found under Sites.

Required proof: Lighthouse CI, real-user monitoring, device/network segmentation, and p75 reporting by deployment context.

### §4.14 Availability and Recovery

Best counterpart evidence: cited public pages do not expose an apples-to-apples contractual availability number for all counterpart site surfaces.

Oyatie target: 99.99 percent reads, 99.95 percent writes, RTO <= 15 minutes, RPO <= 60 seconds.

Comparison: strong target, external parity not provable from cited public sources alone.

Current claim state: not yet claimable.

Why: SLOs exist, but deployment-context infrastructure and restore evidence are missing.

Required proof: backup/restore drill, failover drill, error-budget telemetry, incident runbook test, and context-specific SLO mapping.

### §4.15 OCI Always Free Profile

Best counterpart evidence: counterpart free or entry envelopes differ by vendor and are not directly equivalent.

Oyatie target: demo_trial tenants run inside OCI Always Free profile caps while preserving service correctness.

Comparison: differentiated if implemented, because the model ties free usage to explicit infrastructure caps rather than reduced product quality.

Current claim state: not yet claimable.

Why: no `iac/oci-guest/always-free` module exists under Sites.

Required proof: OpenTofu module, quota policy, load test within cap, cap-exceeded behavior, and conversion path to paid.

### §4.16 Deployment Context Maturity

Best counterpart evidence: Webflow and Wix are managed SaaS surfaces; Squarespace is a managed SaaS surface; none of the cited public docs prove customer-owned AWS/OCI/on-prem/colo deployability.

Oyatie target: all six canonical deployment contexts unless audit finds otherwise.

Comparison: potentially ahead on deployment optionality.

Current claim state: not yet claimable.

Why: current Sites IaC is not OpenTofu and not context-complete.

Required proof: six context modules, shared inputs/outputs, smoke tests, security tests, rollback tests, and cost envelopes.

## §5 Benchmark Acceptance Gate

Gate 1: keep the benchmark target set single and uniform.

Evidence required: this file contains one canonical target set and overlays only by deployment context and tenant_class.

Gate 2: do not use retired plan-ladder labels.

Evidence required: verification grep must return no forbidden label matches in this file.

Gate 3: cite every target or estimate.

Evidence required: each counterpart number and Oyatie target row names a local artifact, official source, or canonical constraint.

Gate 4: distinguish target from claim.

Evidence required: every comparison narrative includes current claim state.

Gate 5: require context-specific proof before deployment maturity claims.

Evidence required: each deployment context overlay says what evidence is missing.

Gate 6: keep demo_trial quality uniform.

Evidence required: demo_trial rows cap allocation and usage, not correctness.

Gate 7: identify missing benchmark artifacts.

Evidence required: comparison sections name absent load tests, Core Web Vitals evidence, quota policy, and OpenTofu modules.

Gate 8: keep counterpart comparison public-source based.

Evidence required: no private or unverifiable competitor telemetry is assumed.

Gate 9: treat old local estimates as estimates.

Evidence required: rows sourced from the existing benchmark document are labeled estimated.

Gate 10: require future implementation proof.

Evidence required: all target-vs-counterpart wins remain not yet claimable until code, IaC, and tests exist.

## §6 Summary

The Sites service has ambitious performance targets in PRD and capacity artifacts.

Those targets are credible as an industry-leader planning bar.

They are not yet production maturity evidence.

The strongest public counterpart hosting claim is Webflow's less-than-100-ms response statement.

The strongest public counterpart CMS item ceiling is Wix's 10,000,000-item CMS limit.

The clearest public counterpart API ceiling is Squarespace's 300 requests per minute / 5 requests per second Commerce API limit.

The clearest public counterpart publish ceiling is Webflow's one successful Site Publish operation per minute.

Oyatie targets can beat several counterpart ceilings if implemented.

Oyatie cannot yet claim all-context deployability because Sites lacks context-specific OpenTofu modules.

Oyatie cannot yet claim demo_trial infrastructure readiness because Sites lacks OCI Always Free profile IaC.

Oyatie cannot yet claim tenant_class performance governance because the service lacks first-class `demo_trial`, `paid`, and `revenue_share` semantics.

Oyatie cannot yet claim Core Web Vitals maturity because no field or lab evidence was found under Sites.

The next benchmark work should implement measured load-test suites, OpenTofu context modules, tenant_class quota policy, and per-context evidence bundles.
