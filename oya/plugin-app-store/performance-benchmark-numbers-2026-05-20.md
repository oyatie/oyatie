# plugin-app-store performance benchmark numbers

Audit date: 2026-05-20.
Target microservice: `microservices/plugin-app-store/`.
Counterpart set: VS Code Marketplace, Chrome Web Store, Shopify App Store.
Methodology disclosure: public counterparts do not publish complete end-to-end latency benchmarks for catalog search, install, review, and billing, so this report separates official public numbers from explicit estimates.
Methodology disclosure: when a counterpart number is an official product limit or policy duration, it is labeled `source`.
Methodology disclosure: when a counterpart number is a measured/derived UX target without an official public benchmark, it is labeled `estimated from public UX surface`.
Methodology disclosure: Oyatie targets are normative design targets derived from local PRD/SLOs, counterpart union expectations, and canonical deployment constraints.
Methodology disclosure: no retired four-label ladder rows or headings are used.
Methodology disclosure: tenant classes are `demo_trial`, `paid`, and `revenue_share`; they constrain usage, economics, and substrate caps, not quality bar.

Five-citation anchor block:
Citation 1: local PRD latency, availability, scale, security, and acceptance targets at `PRD.md:70-124`.
Citation 2: OpenSLO local objectives at `slos/catalog-browse-latency.openslo.yaml:13-15`, `slos/plugin-install-latency.openslo.yaml:13-15`, `slos/plugin-revoke-latency.openslo.yaml:13-15`, and `slos/runtime-invocation-latency.openslo.yaml:13-15`.
Citation 3: canonical deployment, OpenTofu, OS, Rust, and OCI constraints at `specs/master-plan-sequencing.json:704-867`.
Citation 4: no-ladder and tenant-class doctrine at `feedback_no_capability_ladder_2026_05_20.md:10-45` and `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-113`, superseded by the current batch directive for three tenant classes.
Citation 5: public counterpart sources at https://code.visualstudio.com/docs/configure/extensions/extension-marketplace, https://code.visualstudio.com/api/working-with-extensions/publishing-extension, https://developer.chrome.com/docs/webstore/publish, https://developer.chrome.com/docs/webstore/review-process/, https://developer.chrome.com/docs/webstore/program-policies/policies, https://shopify.dev/docs/apps/launch/app-store-review, https://shopify.dev/docs/apps/launch/distribution/revenue-share, https://shopify.dev/docs/api/usage/limits, and https://apps.shopify.com/.

## §1 Methodology

Benchmark dimension 001: catalog search latency p50, p95, and p99.
Benchmark dimension 002: listing detail latency p50, p95, and p99.
Benchmark dimension 003: install workflow latency p50, p95, and p99.
Benchmark dimension 004: revoke workflow latency p50, p95, and p99.
Benchmark dimension 005: policy materialization latency p99.
Benchmark dimension 006: runtime invocation latency p50, p95, and p99.
Benchmark dimension 007: publisher submission validation latency.
Benchmark dimension 008: automated vetting decision latency.
Benchmark dimension 009: manual vetting service objective.
Benchmark dimension 010: catalog throughput per region.
Benchmark dimension 011: concurrent install operations per region.
Benchmark dimension 012: tenant-plugin installations per region.
Benchmark dimension 013: active plugin listings at GA and hyperscale.
Benchmark dimension 014: package size maximum.
Benchmark dimension 015: staged rollout percentage granularity.
Benchmark dimension 016: pricing/billing event processing latency.
Benchmark dimension 017: revenue-share reconciliation latency.
Benchmark dimension 018: audit-chain seal freshness.
Benchmark dimension 019: admin/governance operation latency.
Benchmark dimension 020: availability objective per critical surface.
Test workload 001: catalog search with 100,000 listing index, 30% filtered search, 50% keyword search, 20% personalized/recommended search.
Test workload 002: catalog search with 1,000,000 listing index for hyperscale-readiness validation.
Test workload 003: listing detail fetch with metadata, ratings, vetting verdict, package versions, and tenant eligibility.
Test workload 004: install path with policy materialization, entitlement write, audit-chain append, and runtime prewarm.
Test workload 005: revoke path with entitlement removal, runtime kill switch, audit-chain append, and policy cache invalidation.
Test workload 006: publisher submission with package upload, SBOM, signature, metadata, privacy declaration, and review assets.
Test workload 007: vetting pipeline with static scan, policy scan, sandbox run, manual review queue, and publisher notifications.
Test workload 008: billing event path with plugin pricing, usage event, payout ledger, and finops handoff.
Test workload 009: revenue-share reconciliation path for gross-revenue reporting and at-cost substrate accounting.
Test workload 010: runtime invocation with sandbox startup, cached policy check, and bounded host capability access.
OS disclosure: targets assume canonical supported OS matrix coverage from `specs/master-plan-sequencing.json:777-816`, but the current service lacks `supported-oses.json`.
Architecture disclosure: targets assume x86_64 and arm64 first, with ppc64le and s390x support tracked as canonical overlays.
Deployment context disclosure: six contexts are evaluated: oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, and oyatie-as-cloud-provider.
Tenant-class disclosure: `demo_trial` enforces usage caps and OCI Always Free profile constraints.
Tenant-class disclosure: `paid` scales with per-seat licensing, usage charges, and contractual SLOs.
Tenant-class disclosure: `revenue_share` scales at-cost or zero-margin substrate in exchange for gross-revenue percentage.
Quality disclosure: all three tenant classes target the same functional quality; only usage, cost, and substrate limits differ.
Evidence disclosure: local PRD targets are authoritative unless contradicted by OpenSLO files, in which case this report flags reconciliation need.

## §2 Counterpart numbers

### §2.1 VS Code Marketplace numbers

VS-001: source: verified publisher eligibility requires one or more extensions published for at least 6 months.
VS-002: source: verified publisher validation review is expected within 5 business days after domain TXT verification.
VS-003: source: verified publisher domain must support HTTPS and return HTTP 200 to HEAD requests.
VS-004: source: Marketplace extension pricing label has 2 public values in docs, Free and Trial.
VS-005: source: supported platform-specific targets listed by VS Code docs total 10 values: Windows x64, Windows arm64, Linux x64, Linux arm64, Linux armhf, Alpine x64, Alpine arm64, macOS x64, macOS arm64, and web.
VS-006: source: pre-release extension support requires VS Code 1.63.0 or later.
VS-007: source: platform-specific extension support requires VS Code 1.61.0 or later.
VS-008: source: extension sort examples include 5 axes: installs, name, published date, rating, and update date.
VS-009: source: extension filters include installed, disabled, enabled, featured, popular, recent, recommended, updates, and workspace unsupported, which is 9 documented filter families.
VS-010: source: command-line extension management includes list, install, uninstall, and version-style flows, which gives at least 4 core CLI operations.
VS-011: source: `.vscodeignore` can reduce package footprint by excluding development files.
VS-012: source: unpublish can hide an extension while retaining statistics.
VS-013: source: remove permanently removes statistics and reserves the name.
VS-014: estimated from public UX surface: catalog search should feel interactive below 300ms p95 for in-editor discovery; VS Code docs do not publish an official p95.
VS-015: estimated from public UX surface: install should remain within several seconds for typical VSIX packages; VS Code docs do not publish official install p95.
VS takeaway 001: VS Code's strongest official numbers are publisher trust age, review days, platform targets, filter/sort count, and lifecycle semantics.
VS takeaway 002: VS Code does not publish enough end-to-end latency numbers to serve as the sole performance source.
VS takeaway 003: Oyatie should beat the estimated UX latency while matching lifecycle breadth.

### §2.2 Chrome Web Store numbers

CH-001: source: maximum uploaded extension package size is 2GB.
CH-002: source: default publisher limit is 20 extensions before requesting an increase.
CH-003: source: staged publish after approval expires after 30 days if not published.
CH-004: source: review usually completes in a few days.
CH-005: source: review can take a few weeks.
CH-006: source: developers should contact support if review is pending longer than 3 weeks.
CH-007: source: staged rollout deploy percentage is an integer from 0 to 100.
CH-008: source: staged rollout percentage can only be increased, not decreased.
CH-009: source: deploy percentage API only applies to items with more than 10,000 active users in the past 7 days.
CH-010: source: Chrome Web Store requires 2-Step Verification for all developer accounts before publishing or updating.
CH-011: source: policy review is periodic for existing items, not only first publish.
CH-012: source: dangerous permission use can increase review time.
CH-013: source: broad host permissions can increase review time.
CH-014: source: large or hard-to-review code can increase review time.
CH-015: source: obfuscation is prohibited.
CH takeaway 001: Chrome publishes the strongest package, rollout, review-duration, and trust-control numbers among the three counterparts.
CH takeaway 002: Chrome's 2GB upload ceiling is the clearest public package-size bar.
CH takeaway 003: Chrome's 30-day approved-but-unpublished expiry is a useful release-governance target.
CH takeaway 004: Chrome's 10,000-active-user threshold for rollout percentage API is a useful scale-gating reference, not an Oyatie cap.

### §2.3 Shopify App Store numbers

SH-001: source: Shopify App Store public home states more than 16,000 apps are available.
SH-002: source: Shopify App Store public home states every app goes through a 100-checkpoint review.
SH-003: source: Shopify Partner account App Store registration fee is $19 one time.
SH-004: source: revenue-share policy allows 100% developer share for the first $1,000,000 USD in annual gross app revenue beginning January 1, 2025.
SH-005: source: revenue above the first $1,000,000 USD uses 85% developer share and 15% Shopify share.
SH-006: source: developers above revenue/company thresholds pay 15% on all app revenue.
SH-007: source: revenue-share calculation uses gross sales rather than net sales.
SH-008: source: Shopify app ads search results placements are 4 on desktop and 3 on mobile.
SH-009: source: Shopify app ads category/subcategory placements are 4 on desktop and 2 on mobile.
SH-010: source: Shopify app ads homepage placements are 4 on desktop and 4 on mobile.
SH-011: source: Shopify ad pricing is cost-per-click.
SH-012: source: GraphQL Admin API base point budget is 100 points per second for standard apps.
SH-013: source: GraphQL Admin API point budget is 200 points per second for Advanced Shopify.
SH-014: source: GraphQL Admin API point budget is 1,000 points per second for Shopify Plus.
SH-015: source: GraphQL Admin API point budget is 2,000 points per second for Commerce Components.
SH-016: source: input arrays are capped at 250 items.
SH-017: source: pagination returns up to 25,000 objects, with counts returning 25,001 when more are available.
SH-018: source: public app storefront performance cannot reduce Lighthouse score by more than 10 points for App Store publication.
SH takeaway 001: Shopify supplies the strongest billing, revenue-share, ads-placement, app-count, review-checkpoint, and API-rate public numbers.
SH takeaway 002: Shopify's revenue-share policy is the closest public counterpart to Oyatie's `revenue_share` tenant class.
SH takeaway 003: Shopify's 100-checkpoint review is a good target for Oyatie vetting checklist depth.

## §3 Oyatie target numbers

### §3.1 Single industry-leader target set

Target 001: catalog search p50 <= 80ms.
Target 002: catalog search p95 <= 180ms.
Target 003: catalog search p99 <= 450ms.
Target 004: listing detail p50 <= 60ms.
Target 005: listing detail p95 <= 160ms.
Target 006: listing detail p99 <= 400ms.
Target 007: install workflow p50 <= 1.5s.
Target 008: install workflow p95 <= 4s.
Target 009: install workflow p99 <= 12s.
Target 010: revoke workflow p50 <= 2s.
Target 011: revoke workflow p95 <= 12s.
Target 012: revoke workflow p99 <= 25s.
Target 013: policy materialization p99 <= 5ms, matching ADR-PAS-0001 intent at `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:22-34`.
Target 014: runtime invocation p50 <= 40ms for warm sandbox call.
Target 015: runtime invocation p95 <= 150ms for warm sandbox call.
Target 016: runtime invocation p99 <= 300ms, matching PRD cold-start boundary at `PRD.md:75`.
Target 017: publisher submission validation p50 <= 30s for syntactic metadata/SBOM/signature acceptance.
Target 018: publisher submission validation p95 <= 2m for automated validation queue.
Target 019: publisher submission validation p99 <= 10m under normal queue load.
Target 020: automated vetting decision p50 <= 15m.
Target 021: automated vetting decision p95 <= 60m.
Target 022: manual vetting escalation p95 <= 4h.
Target 023: manual vetting escalation p99 <= 24h.
Target 024: full vetting SLA <= 5 business days, matching `PRD.md:74`.
Target 025: catalog browse availability >= 99.99%, matching `PRD.md:78` and requiring OpenSLO correction.
Target 026: install/revoke availability >= 99.95%, matching `PRD.md:79`.
Target 027: vetting pipeline availability >= 99.0%, matching `PRD.md:80`.
Target 028: runtime sandbox availability >= 99.9%, matching `PRD.md:81`.
Target 029: active plugin listings at GA >= 100,000, matching `PRD.md:84`.
Target 030: active plugin listings at hyperscale validation >= 1,000,000, matching `PRD.md:84` after tier wording is scrubbed.
Target 031: concurrent installs per region >= 10,000, matching `PRD.md:85`.
Target 032: tenant-plugin installations per region >= 1,000,000, matching `PRD.md:86`.
Target 033: concurrent invocations per installation default >= 100, matching `PRD.md:87`.
Target 034: maximum plugin package size target = 2GB, matching Chrome's public upload ceiling unless Oyatie security review sets a lower signed-package cap.
Target 035: staged rollout percentage granularity = 1 integer percent from 0 to 100.
Target 036: staged rollout monotonicity = increases only unless rollback explicitly starts a new emergency release record.
Target 037: approved-but-unpublished release expiry = 30 days, matching Chrome's staged publish expiry.
Target 038: publisher trust probation = 6 months of clean history before trusted-publisher marker, matching VS Code verified-publisher duration.
Target 039: trusted-publisher validation review <= 5 business days, matching VS Code verified-publisher validation.
Target 040: publisher account extension/package default cap = 20 active plugins before review, matching Chrome's default publisher-limit signal.
Target 041: reviewer checklist depth >= 100 checks, matching Shopify's public 100-checkpoint review claim.
Target 042: app-store ad search placements = no more than 4 desktop and 3 mobile sponsored slots when ads are enabled.
Target 043: app-store category placements = no more than 4 desktop and 2 mobile sponsored slots when ads are enabled.
Target 044: app-store homepage placements = no more than 4 desktop and 4 mobile sponsored slots when ads are enabled.
Target 045: GraphQL/Admin-equivalent marketplace API budget baseline >= 100 points/s per paid tenant integration.
Target 046: high-scale paid tenant API budget >= 1,000 points/s with contractual SLO and payment.
Target 047: revenue-share gross-revenue reconciliation p95 <= 24h.
Target 048: revenue-share gross-revenue reconciliation p99 <= 72h.
Target 049: billing event ingestion p95 <= 60s.
Target 050: billing event ingestion p99 <= 5m.
Target 051: audit-chain seal freshness p95 <= 5m.
Target 052: audit-chain seal freshness p99 <= 15m.
Target 053: admin governance policy override p95 <= 500ms.
Target 054: admin governance policy override p99 <= 2s.
Target 055: abuse takedown emergency disable p95 <= 60s after authorized decision.
Target 056: package malware scan p95 <= 30m for packages below 2GB.
Target 057: package malware scan p99 <= 2h for packages below 2GB.
Target 058: permission manifest review p95 <= 30m automated.
Target 059: privacy declaration validation p95 <= 15m automated.
Target 060: support escalation for review pending longer than 3 weeks, matching Chrome's public support threshold.

### §3.2 Deployment-context overlays

Overlay public-cloud 001: oyatie-public-cloud should meet 100% of the target set with elastic regional scale.
Overlay public-cloud 002: oyatie-public-cloud should support 100,000 active listings at GA and 1,000,000 at hyperscale validation.
Overlay public-cloud 003: oyatie-public-cloud should support 10,000 concurrent installs per region.
Overlay public-cloud 004: oyatie-public-cloud should support paid and revenue_share tenants without substrate caps beyond contractual quotas.
Overlay aws 001: guest-on-aws should meet the same latency targets when provisioned with recommended managed database, cache, object store, and queue profile.
Overlay aws 002: guest-on-aws throughput should be capped only by customer account quotas and selected instance classes.
Overlay aws 003: guest-on-aws should require explicit quota warnings when customer AWS limits prevent target throughput.
Overlay oci 001: guest-on-oci full paid deployments should meet the target set when not constrained to Always Free profile.
Overlay oci 002: OCI Always Free profile is for demo_trial infrastructure and should cap throughput to the configured 4 OCPU profile example from the current batch directive.
Overlay oci 003: OCI Always Free demo_trial target is catalog search p95 <= 300ms up to capped daily volume.
Overlay oci 004: OCI Always Free demo_trial target is install p95 <= 8s up to capped install volume.
Overlay oci 005: OCI Always Free demo_trial target is concurrent installs capped at 100 per region-equivalent deployment.
Overlay oci 006: OCI Always Free demo_trial target is active plugin listings cached locally at 10,000 unless external catalog federation is used.
Overlay on-prem 001: on-prem must publish a facility-specific capacity sheet before contractual targets are promised.
Overlay on-prem 002: on-prem must still meet correctness, audit, policy, and security targets even when latency depends on customer hardware.
Overlay on-prem 003: on-prem package scan p99 may be facility-bound for 2GB packages, but must declare the hardware profile.
Overlay colo 001: colo should meet public-cloud targets when Oyatie controls the facility profile.
Overlay colo 002: colo must publish bandwidth, storage, HSM, and object-store assumptions before scale claims.
Overlay iaas 001: oyatie-as-cloud-provider should meet or beat oyatie-public-cloud targets under Oyatie-controlled substrate.
Overlay iaas 002: oyatie-as-cloud-provider should expose tenant-visible capacity reservations for paid and revenue_share tenants.
Overlay all 001: all six contexts must provide OpenTofu plan/apply evidence before deployability is claimed.
Overlay all 002: all six contexts must provide supported OS evidence before OS-support claims are made.
Overlay all 003: all six contexts must preserve uniform functional quality across tenant classes.

### §3.3 Tenant-class overlays

Tenant demo_trial 001: demo_trial uses OCI Always Free profile where possible.
Tenant demo_trial 002: demo_trial daily catalog search cap target = 10,000 searches per tenant.
Tenant demo_trial 003: demo_trial daily install cap target = 25 install or update operations.
Tenant demo_trial 004: demo_trial active plugin cap target = 25 installed plugins.
Tenant demo_trial 005: demo_trial concurrent runtime invocation cap target = 10 per tenant.
Tenant demo_trial 006: demo_trial manual vetting is best-effort and can be deprioritized after paid/revenue_share queues while preserving safety.
Tenant demo_trial 007: demo_trial does not receive compliance packs by default.
Tenant demo_trial 008: demo_trial does not receive BYOK by default.
Tenant demo_trial 009: demo_trial still receives the same security vetting and audit-chain correctness.
Tenant paid 001: paid tenant catalog and install targets match the canonical target set.
Tenant paid 002: paid tenants scale per-seat plus usage with contractual SLO.
Tenant paid 003: paid tenants may receive compliance packs.
Tenant paid 004: paid tenants may receive BYOK.
Tenant paid 005: paid tenants can buy higher API budgets without changing feature quality.
Tenant paid 006: paid tenants can buy dedicated review capacity without bypassing policy.
Tenant revenue_share 001: revenue_share tenant target quality matches paid.
Tenant revenue_share 002: revenue_share substrate is at-cost or zero-margin when contractually selected.
Tenant revenue_share 003: revenue_share gross-revenue reconciliation p95 target is <= 24h.
Tenant revenue_share 004: revenue_share gross-revenue reconciliation p99 target is <= 72h.
Tenant revenue_share 005: revenue_share tenants require stronger payout, audit, and gross-sales evidence than demo_trial or normal paid.
Tenant revenue_share 006: revenue_share tenants may justify larger ad/promotion and API budgets because economics are tied to gross revenue.
Tenant all 001: tenant class never gates core security, audit, vetting correctness, or data isolation.
Tenant all 002: tenant class never maps to retired four-label ladder.
Tenant all 003: tenant class should be visible in policy/audit context but not confuse plugin pricing model.

## §4 Comparison narrative

Comparison 001: catalog p95 target of 180ms is ahead of the local PRD p95 ceiling of 200ms at `PRD.md:70`.
Comparison 002: catalog p99 target of 450ms is ahead of the local PRD p99 ceiling of 500ms at `PRD.md:70`.
Comparison 003: install p95 target of 4s is ahead of the PRD p95 ceiling of 5s at `PRD.md:72`.
Comparison 004: install p99 target of 12s is ahead of the PRD p99 ceiling of 15s at `PRD.md:72`.
Comparison 005: revoke p99 target of 25s is ahead of the PRD p99 ceiling of 30s at `PRD.md:73`.
Comparison 006: runtime p99 target of 300ms is parity with the PRD p99 cold-start ceiling at `PRD.md:75`.
Comparison 007: policy materialization p99 target of 5ms is parity with ADR-PAS-0001 at `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:22-34`.
Comparison 008: active listing target of 100,000 at GA is parity with `PRD.md:84`.
Comparison 009: hyperscale listing target of 1,000,000 keeps the PRD's numeric ambition while scrubbing the stale tier wording at `PRD.md:84`.
Comparison 010: concurrent install target of 10,000 per region is parity with `PRD.md:85`.
Comparison 011: tenant-plugin installation target of 1,000,000 per region is parity with `PRD.md:86`.
Comparison 012: default concurrent invocation target of 100 per installation is parity with `PRD.md:87`.
Comparison 013: 2GB package limit is catch-up to Chrome's public package ceiling and should be reduced only with a security rationale.
Comparison 014: 30-day approved-but-unpublished expiry is catch-up to Chrome's staged publish governance.
Comparison 015: 0-100 integer rollout percentage is catch-up to Chrome's staged rollout deploy percentage API.
Comparison 016: 10,000-active-user gate is not adopted as an Oyatie cap; Oyatie should support staged rollout at lower scale because tenant install safety matters before public scale.
Comparison 017: 6-month trusted-publisher probation is catch-up to VS Code verified-publisher duration.
Comparison 018: 5-business-day trusted-publisher review is catch-up to VS Code verified-publisher review expectation.
Comparison 019: 20-plugin default publisher cap is catch-up to Chrome's default publisher limit and reduces abuse before trust history exists.
Comparison 020: 100-checkpoint vetting depth is parity with Shopify's public review depth claim.
Comparison 021: 4/3 search ad placement cap is parity with Shopify if Oyatie adopts ads.
Comparison 022: 4/2 category ad placement cap is parity with Shopify if Oyatie adopts ads.
Comparison 023: 4/4 homepage ad placement cap is parity with Shopify if Oyatie adopts ads.
Comparison 024: 100 points/s baseline API budget is parity with Shopify's standard GraphQL Admin API plan class.
Comparison 025: 1,000 points/s paid high-scale budget is parity with Shopify Plus point budget.
Comparison 026: 2,000 points/s target for Oyatie-as-cloud-provider reserved capacity can match Shopify Commerce Components scale when contracted.
Comparison 027: revenue-share p95 reconciliation <=24h is ahead of many app-store payout cycles but requires finops integration.
Comparison 028: audit-chain seal freshness p99 <=15m is ahead of public counterpart audit surfaces because counterparts do not expose equivalent per-install seal freshness.
Comparison 029: OCI Always Free demo_trial overlay intentionally caps throughput below paid targets without lowering feature quality.
Comparison 030: on-prem and colo overlays are catch-up until hardware/facility-specific capacity sheets exist.
Comparison 031: current artifact state cannot prove any target because no Rust source, tests, OpenTofu context modules, or OS manifest exist.
Comparison 032: the target set is implementable only after PAS-COH-001 through PAS-COH-007 from the coherence audit are addressed.
Comparison 033: catalog availability must be reconciled because PRD says 99.99% while the OpenSLO says 99.9%.
Comparison 034: vetting p95 must be reconciled because PRD says 4h while the OpenSLO targets 1h within a throughput window.
Comparison 035: no target in this report creates a retired tier; all limits are deployment-context or tenant-class overlays.

## §5 Validation and adoption notes

Validation note 001: targets are not completion evidence until Rust source, tests, and OpenTofu modules exist.
Validation note 002: catalog latency targets should be validated with a Rust load harness, not the stale `.js` k6 path in `PRD.md:110`.
Validation note 003: install throughput should be validated with the install drill requested by `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:121-126` after the test path is corrected.
Validation note 004: runtime invocation latency should be validated with Wasmtime sandbox benchmarks tied to ADR-PAS-0003.
Validation note 005: policy materialization latency should be validated against ADR-PAS-0001 p99 intent.
Validation note 006: billing ingestion should be validated with shared billing-engine and finops-ledger handoff evidence.
Validation note 007: revenue-share reconciliation should be validated with gross-revenue audit evidence before public revenue_share tenant claims.
Validation note 008: OCI Always Free profile targets should be tested on the configured demo_trial infrastructure, not inferred from paid deployments.
Validation note 009: on-prem targets should require a declared hardware profile before contractual comparison.
Validation note 010: colo targets should require facility bandwidth, storage, and HSM assumptions before comparison.
Validation note 011: guest-on-aws targets should include account quota detection before failure is attributed to service logic.
Validation note 012: guest-on-oci paid targets should be separated from OCI Always Free demo_trial targets.
Validation note 013: oyatie-public-cloud targets should be the default benchmark baseline because Oyatie controls the substrate.
Validation note 014: oyatie-as-cloud-provider targets should be at least equal to oyatie-public-cloud when substrate capacity is equivalent.
Validation note 015: supported OS benchmark results should include x86_64 and arm64 first and track ppc64le/s390x as required overlays.
Validation note 016: package-size benchmarks should include 10MB, 100MB, 1GB, and 2GB signed packages.
Validation note 017: vetting benchmarks should separate automated queue time from manual reviewer time.
Validation note 018: review pending beyond 3 weeks should trigger support escalation because Chrome uses that public threshold.
Validation note 019: trusted-publisher validation should not bypass package vetting; VS Code's 5-business-day validation is identity validation, not package safety validation.
Validation note 020: Shopify's 100-checkpoint benchmark should be converted to explicit checklist assertions before being used as evidence.
Validation note 021: Shopify's API point budgets should inspire API throttling shape, but Oyatie must define its own cost model.
Validation note 022: ad placement caps should be ignored unless plugin-app-store owns ads rather than shared marketplace search/auction.
Validation note 023: audit-chain seal freshness should be validated with seal generation, replication, restore, and tamper-evidence cases.
Validation note 024: abuse takedown p95 should be validated with authorization, audit, runtime kill switch, and delist propagation.
Validation note 025: SLO docs should be regenerated only after PRD/OpenSLO contradictions are resolved.
Validation note 026: no benchmark row should be copied into a four-label tier matrix during Wave 15J retirement.
Validation note 027: the single target set should remain the quality bar for demo_trial, paid, and revenue_share, with only usage caps and substrate ceilings varying.
Validation note 028: counterpart numbers from official docs should be refreshed before implementation if public limits change.
Validation note 029: estimated counterpart latency values should be treated as design pressure, not external factual claims.
Validation note 030: the stop condition for this benchmark report is a source-cited target table plus explicit validation gaps, not passing benchmark execution.
