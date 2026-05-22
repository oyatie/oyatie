# plugin-app-store feature parity matrix

Audit date: 2026-05-20.
Target microservice: `microservices/plugin-app-store/`.
Counterpart 1: VS Code Marketplace.
Counterpart 2: Chrome Web Store.
Counterpart 3: Shopify App Store.
Union-coverage rule: Oyatie should meet or exceed the union of the three counterpart surfaces where the feature fits plugin/app distribution.
Retired-deliverable note: no capability-ladder delta analysis is authored; stale tiers are tracked in the coherence audit.
Tenant-class note: `demo_trial`, `paid`, and `revenue_share` are tenant classes, not feature tiers.
Local purpose anchor: `PRD.md:23-31` says this is a plugin/app distribution surface, not the broad B2C marketplace.
Local feature anchor: `PRD.md:50-67` lists discovery, publisher, vetting, install, governance, runtime, billing, rating, abuse, audit, and API requirements.
Local architecture anchor: `ARCHITECTURE.md:197-208` binds listings, installations, and runtime objects to product/substrate identifiers.
Local marketplace boundary anchor: `8f603fc7...jsonl:1249-1252` says plugin-app-store schemas should depend on shared marketplace substrates.
VS Code source anchor: https://code.visualstudio.com/docs/configure/extensions/extension-marketplace.
VS Code publishing source anchor: https://code.visualstudio.com/api/working-with-extensions/publishing-extension.
Chrome source anchor: https://developer.chrome.com/docs/webstore/publish.
Chrome review source anchor: https://developer.chrome.com/docs/webstore/review-process/.
Chrome policy source anchor: https://developer.chrome.com/docs/webstore/program-policies/policies.
Shopify source anchor: https://shopify.dev/docs/apps/launch/app-store-review.
Shopify requirements source anchor: https://shopify.dev/docs/apps/launch/shopify-app-store/app-store-requirements.
Shopify billing source anchor: https://shopify.dev/docs/apps/launch/billing.
Shopify revenue-share source anchor: https://shopify.dev/docs/apps/launch/distribution/revenue-share.
Shopify store source anchor: https://apps.shopify.com/.

## §1 Scope and scoring

Coverage score `present` means the current service artifacts define the surface with enough detail to implement.
Coverage score `partial` means the surface appears but lacks contract, operational, or canonical-direction detail.
Coverage score `absent` means the surface is missing from the current service artifacts.
Coverage score `misaligned` means the surface exists but conflicts with canonical direction or retired terminology.
Local evidence is cited with repository file lines.
Counterpart evidence is cited with public official source URLs.
This matrix does not compare four retired tiers.
This matrix treats quality as uniform across all tenant classes.
This matrix treats deployment-context overlays as infrastructure constraints, not product-feature downgrades.
This matrix keeps plugin-app-store separate from the broad marketplace service.
This matrix keeps developer SDK generator ownership outside plugin-app-store because `PRD.md:195` assigns SDK ownership to developer-sdk.
This matrix considers revenue share as an Oyatie tenant-class and marketplace monetization capability, not as a tier gate.

## §2 Counterpart 1: VS Code Marketplace capability surface

VS-001: In-editor extension discovery is core; VS Code docs show search and install in the Extensions view.
VS-002: Browser Marketplace discovery is supported; VS Code docs link extension details to Marketplace pages.
VS-003: One-click install exists; VS Code docs say the Install button downloads and installs the extension.
VS-004: Manual VSIX install exists; VS Code docs document Install from VSIX and command-line install.
VS-005: Command-line extension management exists; VS Code supports list, install, uninstall, and version output.
VS-006: Workspace recommendations exist; VS Code creates `.vscode/extensions.json` recommendations.
VS-007: Multi-root workspace recommendations exist; docs describe `.code-workspace` recommendations.
VS-008: Installed, disabled, enabled, featured, popular, recent, recommended, update, and workspace unsupported filters exist.
VS-009: Sort by installs, name, published date, rating, and update date exists.
VS-010: Category and tag filtering exists.
VS-011: Supported extension categories include themes, formatters, linters, snippets, testing, language packs, and more.
VS-012: Extension identifier format is `publisher.extension`.
VS-013: Marketplace details expose README, repository link, changelog, dependencies, and related metadata.
VS-014: Extension auto-update is default; users can disable all or per-extension auto-update.
VS-015: Manual update exists through outdated-extension filters and update commands.
VS-016: Disable globally and disable per workspace are user controls.
VS-017: Uninstall flow prompts extension-host restart when needed.
VS-018: Publisher identity is required through Marketplace publisher management.
VS-019: Publishing requires Azure DevOps-backed authentication and a Personal Access Token.
VS-020: `vsce` supports package, publish, metadata retrieval, and unpublish.
VS-021: Publishing blocks user-provided SVGs in icons and untrusted SVGs in README/CHANGELOG images.
VS-022: Publisher reports include acquisition trends, total acquisition counts, ratings, and reviews.
VS-023: Unpublish preserves statistics and leaves an API-discoverable extension.
VS-024: Remove permanently reserves the extension name and removes statistics.
VS-025: Deprecation exists and can point to alternatives or settings.
VS-026: Marketplace-level deprecation rendering is not fully available yet per VS Code docs, but client UI deprecation exists.
VS-027: Verified publisher badges require eligible domain verification.
VS-028: Verified publisher prerequisite includes one or more extensions for at least six months.
VS-029: Verified publisher review is expected within five business days after TXT validation.
VS-030: Eligible verified domains must support HTTPS and HTTP 200 to HEAD.
VS-031: Pricing labels are limited to Free and Trial.
VS-032: Sponsor links are available.
VS-033: `.vscodeignore` controls package contents.
VS-034: Pre-publish hooks are available through extension manifest scripts.
VS-035: Pre-release extensions are supported.
VS-036: Pre-release requires distinct version sequencing.
VS-037: Pre-release support requires VS Code version 1.63.0 or later.
VS-038: Platform-specific extensions exist.
VS-039: Platform targets include Windows x64/arm64, Linux x64/arm64/armhf, Alpine x64/arm64, macOS x64/arm64, and web.
VS-040: Platform-specific packages are useful for native node modules and client-specific dependencies.
VS-041: VS Code Marketplace is strong on editor-integrated discovery and install ergonomics.
VS-042: VS Code Marketplace is weaker than Shopify on native revenue-share and merchant billing.
VS-043: VS Code Marketplace is weaker than Chrome on browser permission review and user data policy enforcement.
VS-044: VS Code Marketplace is a direct counterpart for plugin catalog, extension package, install, update, publisher identity, and review surfaces.
VS-045: Oyatie already covers some VS Code-like install and catalog concepts in `PRD.md:50-67`.
VS-046: Oyatie does not yet cover VS Code platform-specific package targeting in local PRD or contracts.
VS-047: Oyatie does not yet cover VS Code workspace recommendation analogs except indirectly through catalog and install.
VS-048: Oyatie does not yet cover verified publisher domain age or five-business-day validation semantics.
VS-049: Oyatie has stronger audit-chain and tenant governance goals than VS Code docs expose publicly.
VS-050: Oyatie should adopt the VS Code distinction between unpublish and irreversible remove for publisher trust.

## §3 Counterpart 2: Chrome Web Store capability surface

CH-001: Developer account registration and dashboard upload are required for publish.
CH-002: Extension ZIP upload is validated before item setup.
CH-003: Maximum supported extension package size is 2GB.
CH-004: A publisher cannot have more than 20 extensions by default, with a limit-increase path.
CH-005: Store Listing tab drives how an item displays in the Chrome Web Store.
CH-006: Privacy tab declares single purpose and user-data handling.
CH-007: Distribution tab declares paid status, countries, and visible users.
CH-008: Test instructions tab provides reviewer credentials when needed.
CH-009: Submit for Review starts the review process.
CH-010: Deferred publishing lets approved items be manually published later.
CH-011: Staged submission expires after 30 days if not published.
CH-012: Review process protects users from scams, data harvesting, malware, and malicious actors.
CH-013: Existing items are periodically reviewed for compliance.
CH-014: Review time is usually a few days, can take a few weeks, and support is recommended after more than three weeks pending.
CH-015: Review uses manual and automated systems.
CH-016: New developers, new extensions, dangerous permissions, and significant code changes can increase review time.
CH-017: Broad host permissions increase review time.
CH-018: Sensitive execution permissions increase review time.
CH-019: Large or hard-to-review code increases review time.
CH-020: Obfuscation is disallowed.
CH-021: Takedown removes items from store search and listing surfaces.
CH-022: Severe violations can permanently suspend developer accounts.
CH-023: Developer communications include automated email, appeals, and support tickets.
CH-024: Program policies require limited use of data.
CH-025: User data use must be limited to disclosed practices.
CH-026: Collection/use of browsing activity is prohibited except for prominent user-facing features.
CH-027: Transfers of user data are constrained to necessary, legal, security, or consented merger contexts.
CH-028: Personalized advertising with user data is prohibited.
CH-029: Human reading of user data is prohibited except narrow consent/security/legal/anonymized operations.
CH-030: Permission requests must be the narrowest needed.
CH-031: Future-proof permission requests are disallowed.
CH-032: Products collecting user data must transmit securely with modern cryptography.
CH-033: 2-Step Verification is required for all developer accounts before publishing or updating.
CH-034: Listing requirements reject blank descriptions or missing icons/screenshots.
CH-035: Listing metadata must be up to date, accurate, comprehensive, and non-misleading.
CH-036: Affiliate ads must be disclosed and tied to user benefit and related user action.
CH-037: Remote code restrictions require reviewable packaged functionality.
CH-038: The Chrome Web Store API can fetch status, upload media, publish, cancel submission, and set published deploy percentage.
CH-039: Deploy percentage API only applies to items with over 10,000 seven-day active users.
CH-040: Deploy percentage target is an integer between 0 and 100 and must only increase.
CH-041: Chrome users can install through Add to Chrome and permission approval.
CH-042: Enhanced Safe Browsing can warn when an extension is not trusted.
CH-043: New developers generally take a few months to become trusted.
CH-044: Users can set site access to on-click, specific sites, or all sites.
CH-045: Users can repair corrupt extensions.
CH-046: Unsupported extensions can be disabled by Chrome for privacy and security protection.
CH-047: Chrome is the strongest counterpart for user-data policy, permission scope, review, staged release, and enforcement.
CH-048: Oyatie partially covers vetting and policy in `PRD.md:55-57` and `decisions/ADR-PAS-0002-ordered-vetting-pipeline.md:22-34`.
CH-049: Oyatie partially covers install-time permission materialization in `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:22-34`.
CH-050: Oyatie does not yet expose Chrome-style staged deploy percentage or review-risk factor taxonomy in contracts.

## §4 Counterpart 3: Shopify App Store capability surface

SH-001: Shopify App Store is the discovery surface for merchants to find apps for their business.
SH-002: Public listings drive browse, recommendations, admin workflow recommendations, and Sidekick discovery.
SH-003: Listing content includes description, features, pricing, and reviews.
SH-004: Apps must be submitted to the App Approval team.
SH-005: Review status lifecycle includes Draft, Submitted, Reviewed, and Published.
SH-006: Apps that fail core requirements can be paused and resubmitted after fixes.
SH-007: Approved public listings are visible by default.
SH-008: Store requirements require complete and accurate submission with documentation and credentials.
SH-009: Pricing information must be in designated pricing details, not in listing images.
SH-010: Listing information must be truthful and accurate.
SH-011: Listing cannot include unsubstantiated claims like first/best/only.
SH-012: Geographic or API permission requirements must be listed when they gate function.
SH-013: App details must explain functionality with enough information for confident install.
SH-014: App screenshots should show actual UI/features and be unique.
SH-015: Demo screencast is required for review when applicable.
SH-016: Test credentials must be valid and grant full feature access.
SH-017: Shopify App Pricing monetizes public apps through standardized merchant billing.
SH-018: Billing API and Shopify App Pricing support one-time, subscription, and usage models depending method.
SH-019: Shopify App Pricing supports recurring and usage charges.
SH-020: Apps published on Shopify App Store must use a Shopify-provided billing solution.
SH-021: Shopify App Pricing automates billing, trials, proration, upgrades, and downgrades.
SH-022: Shopify revenue share lets developers keep 100% of first $1,000,000 gross app revenue from Jan 1, 2025, then 85% above that.
SH-023: Shopify charges a one-time $19 Partner account App Store registration fee.
SH-024: Developers above annual app earnings or company revenue thresholds pay 15% on all app revenue.
SH-025: Revenue share is based on gross sales, not net sales.
SH-026: Revenue share aggregates across associated developer accounts.
SH-027: Shopify App Store ads support search results, category/subcategory pages, and homepage.
SH-028: Search results ads have 4 desktop and 3 mobile placements.
SH-029: Category ads have 4 desktop and 2 mobile placements.
SH-030: Homepage ads have 4 desktop and 4 mobile placements.
SH-031: Ads use CPC pricing.
SH-032: Search ad ranking considers relevance and bid.
SH-033: Shopify store home page says over 16,000 apps are available.
SH-034: Shopify store home page says each app goes through a 100-checkpoint review.
SH-035: Built for Shopify indicates high standards for performance, design, and integration.
SH-036: Shopify performance rules say an app must not reduce storefront Lighthouse performance by more than 10 points for App Store publication.
SH-037: Built for Shopify eligibility also requires mandatory admin and checkout performance criteria.
SH-038: Shopify API limits include GraphQL Admin 100/200/1000/2000 points per second by plan class.
SH-039: Shopify input arrays max at 250.
SH-040: Shopify pagination caps arrays at 25,000 objects and count at 25,001 indicator.
SH-041: Shopify Storefront API has no fixed request-per-minute limit for real buyer requests.
SH-042: Shopify recommends backoff and dynamic behavior from API response metadata.
SH-043: Sales-channel apps may require OAuth, account connection, approval, product feeds, and Billing API.
SH-044: Sales-channel docs require owning or representing a distinct external destination where products are sold.
SH-045: Shopify is the strongest counterpart for billing, revenue share, ads, app review, and merchant-facing listing quality.
SH-046: Oyatie partially covers revenue share in `PRD.md:60`, `PRD.md:160`, and `tutorials/publish-paid-plugin-with-sbom-and-stripe.md:296-298`.
SH-047: Oyatie partially covers app review/vetting through `PRD.md:55` and ADR-PAS-0002.
SH-048: Oyatie does not yet model app-store ads as a stable product surface in plugin-app-store contracts.
SH-049: Oyatie does not yet model Shopify-style standardized billing-provider constraint against its own tenant classes.
SH-050: Oyatie can exceed Shopify on tenant-level audit chain and per-install governance if contract gaps are repaired.

## §5 UNION-coverage matrix

| Capability | VS Code | Chrome | Shopify | Oyatie current | Coverage | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Plugin/app catalog browse | yes | yes | yes | yes | present | `PRD.md:50`; `contracts/openapi/plugin-app-store.yaml:29-66` |
| Keyword search | yes | yes | yes | yes | present | `PRD.md:50`; `implementation-plans/IP-009-search-relevance-and-recommendations.md` |
| Category browse | yes | yes | yes | partial | partial | `contracts/openapi/plugin-app-store.yaml:29-66`; no category taxonomy contract found |
| Tag filtering | yes | limited | yes | partial | partial | `manifest.json:10-21` has doctrine tags but listing tags not explicit |
| Sort by installs | yes | yes | yes | partial | partial | local catalog schema not detailed enough in `contracts/openapi/plugin-app-store.yaml:29-66` |
| Sort by rating | yes | yes | yes | partial | partial | ratings are required at `PRD.md:63`, but sort behavior is not explicit |
| Recently published filter | yes | yes | yes | partial | partial | publish timestamps implied by listing model, not exposed in local contract |
| Featured listings | yes | yes | yes | absent | absent | no featured-listing contract found |
| Sponsored placements | no | limited | yes | partial | partial | `IP-journey-j119-marketplace-auction-surface.md` exists, but no OpenAPI surface |
| App ads | no | no | yes | absent | absent | Shopify ads have CPC placement model; local contract lacks ad placement API |
| Publisher registration | yes | yes | yes | yes | present | `PRD.md:54`; `implementation-plans/IP-002-publisher-onboarding.md` |
| Publisher identity verification | yes | yes | yes | partial | partial | `PRD.md:54`; no domain-age or 2FA contract |
| Verified publisher badge | yes | no | yes via quality labels | partial | partial | `PRD.md:56`; stale badge ladder in `contracts/openapi/plugin-app-store.yaml:252-254` |
| Developer 2FA requirement | not primary | yes | implied | absent | absent | no local publisher account security contract found |
| Publisher analytics | yes | yes | yes | partial | partial | dashboards exist but publisher-facing analytics contract not explicit |
| Acquisition/install counts | yes | yes | yes | partial | partial | `PRD.md:63`; no API field contract for installs visible |
| Ratings | yes | yes | yes | yes | present | `PRD.md:63` |
| Reviews | yes | yes | yes | yes | present | `PRD.md:63` |
| Abuse reports | yes | yes | yes | yes | present | `PRD.md:63` and `PRD.md:65` |
| Takedown flow | yes | yes | yes | partial | partial | `PRD.md:65`; runbook for publisher suspension exists |
| Appeals | yes | yes | yes | partial | partial | `runbooks/publisher-suspension-appeal.md` |
| Unpublish preserving stats | yes | yes | yes | absent | absent | no distinct unpublish/remove state contract found |
| Irreversible remove protection | yes | yes | yes | absent | absent | no name-reservation/delete contract found |
| Deprecation in favor of replacement | yes | limited | yes | partial | partial | `deprecation.md` exists; API semantics not clear |
| Package upload | yes | yes | yes | partial | partial | publisher submission exists; package size/version constraints not explicit |
| Package validation | yes | yes | yes | partial | partial | `PRD.md:55`; vetting pipeline exists |
| Package size maximum | no public single number | 2GB | app-specific | absent | absent | no local max package size found |
| SBOM requirement | not universal | policy-driven | review-driven | yes | present | `PRD.md:56`; tutorial mentions SBOM |
| Signature requirement | partial | partial | partial | yes | present | `PRD.md:56` and Cosign chart inventory |
| Malware scanning | partial | yes | review-driven | yes | present | `PRD.md:55-56`; Trivy chart inventory |
| Policy scanning | partial | yes | yes | yes | present | `PRD.md:55-57`; `policy/*.cedar` |
| Manual review | limited | yes | yes | yes | present | `PRD.md:55`; `decisions/ADR-PAS-0002-ordered-vetting-pipeline.md:22-34` |
| Automated review | yes | yes | yes | yes | present | `decisions/ADR-PAS-0002-ordered-vetting-pipeline.md:22-34` |
| Review-risk factors | no | yes | partial | absent | absent | local docs do not classify broad host/sensitive permission/code size risk |
| Review SLA | 5 business days for verified publisher badge | few days/weeks for extension review | not fixed | yes | partial | `PRD.md:74`; conflicts with `slos/vetting-pipeline-throughput.openslo.yaml:13-15` |
| Reviewer credentials | no | yes | yes | partial | partial | no dedicated review credential object found |
| Demo screencast | no | optional | yes | absent | absent | no local publisher review asset requirement found |
| User install prompt | yes | yes | yes | yes | present | `PRD.md:51-52`; install contract exists |
| Permission grant screen | partial | yes | yes OAuth | yes | present | `PRD.md:51`; `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:22-34` |
| Least-privilege policy | partial | yes | yes | yes | present | `policy/install-approval.cedar`; `policy/tenant-scope.cedar` |
| Site/resource access controls | workspace/client | yes | merchant scopes | partial | partial | Cedar policy exists but Chrome-style site access not explicit |
| Workspace scope install | yes | no | store scope | absent | absent | no workspace/profile install target found |
| Tenant-wide install | no | enterprise policy | merchant store | yes | present | `PRD.md:51`; `contracts/openapi/plugin-app-store.yaml:68-105` |
| Per-user enable/disable | yes | yes | merchant/admin | partial | partial | governance API implied but not explicit |
| Revoke/uninstall | yes | yes | yes | yes | present | `PRD.md:51`; `contracts/openapi/plugin-app-store.yaml:107-124` |
| Extension repair | no | yes | no | absent | absent | no repair workflow found |
| Runtime sandbox | extension host | browser extension sandbox | app hosting | yes | present | `decisions/ADR-PAS-0003-runtime-extension-sandbox.md:22-34` |
| Runtime kill switch | yes via disable | yes via disable/takedown | yes via app delist | yes | present | `runbooks/runtime-kill-switch.md` |
| Runtime invocation limits | no public store standard | browser constraints | API plan constraints | yes | partial | `PRD.md:86-87`; no tenant-class overlay |
| Auto-update | yes | yes | app updates | absent | absent | local install/update lifecycle not explicit |
| Manual update | yes | yes | app updates | absent | absent | no update endpoint visible |
| Staged rollout | no direct docs | yes | app-version releases | absent | absent | no rollout percentage API found |
| Deferred publish | no | yes | review lifecycle | absent | absent | no staged approved/manual publish state found |
| Pre-release channel | yes | limited | limited | absent | absent | no pre-release listing state found |
| Platform-specific packages | yes | browser platform | app scopes | absent | absent | no package target matrix found |
| Offline install | VSIX | CRX policy-limited | no | partial | partial | local docs discuss install but not offline package policy |
| Publisher revenue share | no native broad model | paid extensions deprecated/limited | yes | partial | partial | `PRD.md:60`; stale tier docs in FAQ |
| Paid app subscriptions | trial label only | paid status distribution | yes | yes | partial | pricing schema lacks tenant_class separation |
| Usage billing | no | no | yes | partial | partial | `onboarding/marketplace-publisher-first-week.md:26` |
| One-time purchase | no | distribution paid item | legacy Shopify Billing | partial | partial | OpenAPI pricing includes `one_time` |
| Free trial | trial label | distribution paid/free | yes | partial | partial | tutorial trial lines exist but no tenant class |
| Revenue-share tenant class | no | no | yes as business model | absent | absent | no `tenant_class` literal found |
| Billing provider constraint | no | Chrome payments deprecated/history | Shopify-provided billing | partial | partial | local Stripe Connect tutorial exists without canonical policy split |
| Invoice/chargeback handling | no | limited | yes | partial | partial | finops dependency in `manifest.json:235-245` |
| Merchant/publisher payout ledger | no | limited | yes | partial | partial | `catalog/revenue-share-ledger.yaml`; `runbooks/revenue-share-reconciliation.md` |
| App registration fee | no | developer account | yes | absent | absent | no publisher fee policy found |
| Quality badge | verified publisher | trusted extension | Built for Shopify | partial/misaligned | misaligned | local retired badge ladder at `ADR-PAS-0004` and OpenAPI enum |
| 100-checkpoint review analog | no | review systems | yes | partial | partial | vetting pipeline exists; checkpoint checklist not explicit |
| Storefront performance impact | no | browser performance policy | yes | partial | partial | local runtime SLO exists; Lighthouse delta rule absent |
| API rate limits | client-hosted | store API | Shopify Admin limits | yes | partial | `decisions/ADR-PAS-0005-rate-limits-default-deny.md:22-34` |
| Tenant quota overrides | no | enterprise policy | plan limits | yes | present | `decisions/ADR-PAS-0005-rate-limits-default-deny.md:22-34` |
| Search ads relevance and bid | no | store ads not core | yes | partial | partial | auction journey exists but no plugin contract |
| Category ads | no | no | yes | absent | absent | no placement surface found |
| Homepage ads | no | no | yes | absent | absent | no placement surface found |
| Data privacy declaration | partial | yes | yes | partial | partial | `dpia.md:19-24`; OpenAPI lacks declaration object |
| Limited-use declaration | no | yes | no | absent | absent | no Chrome-like data-use declaration found |
| Protected customer data review | no | yes privacy | yes | partial | partial | `dpia.md` and `compliance.md` exist |
| Cross-border transfer handling | no | policy-driven | policy-driven | yes | present | `dpia.md:44` |
| Appeals/support tickets | yes | yes | yes | partial | partial | suspension appeal runbook exists |
| Developer support requirements | partial | support docs | yes | partial | partial | FAQ exists, no support SLA contract |
| Listing localization | partial | yes | yes | absent | absent | no localization surface found |
| Geographic distribution | no | yes | yes | absent | absent | no countries/regions field found |
| Compliance packs | no | policy | app review | yes | present | `packs/compliance-pack-us-msb/manifest.yaml`; `PRD.md:180-183` |
| BYOK compatibility | no | no | enterprise app | absent | absent | no BYOK listing/install policy found |
| Audit-chain evidence | no | no | limited | yes | ahead | `decisions/ADR-PAS-0007-audit-chain-authoritative.md:22-34` |
| Per-install audit trail | partial | partial | partial | yes | ahead | `PRD.md:66`; `manifest.json:247-262` |
| Multi-region support | marketplace-backed | global CWS | Shopify global | partial | partial | `multi-region.md` exists, but context IaC missing |
| On-prem deployment | no | no | no | claimed | partial | deployment contexts required, IaC missing |
| Colo deployment | no | no | no | claimed | partial | deployment contexts required, IaC missing |
| Guest cloud deployment | no | no | no | claimed | partial | context IaC missing |
| OCI Always Free profile | no | no | no | absent | absent | required by master plan, not counterpart |
| OS support manifest | client OS docs | browser/OS dependent | SaaS/admin | absent | absent | required locally, missing |
| OpenTofu modules | no | no | no | absent | absent | canonical local substrate missing |
| Rust source implementation | no | no | no | absent | absent | `src/` empty |
| Load tests | no | no | no | misaligned | misaligned | `.js` k6 refs at `PRD.md:110` and phase plan line 124 |
| Capability eval sets | no | no | no | partial | partial | capability eval paths missing backing files |

## §6 Family summary

Family discovery: present but incomplete.
Discovery has catalog search, details, ratings, and publisher docs, but lacks explicit featured, sponsored, geographic, localization, and recommendation contract surfaces.
Family publisher: partial.
Publisher onboarding exists, but domain verification, 2FA, account trust age, registration fee, and review credential semantics are not modeled.
Family packaging: partial.
Package submission and vetting exist, but package-size limits, platform-specific package targets, pre-release channels, and unpublish/remove semantics are missing.
Family review and trust: partial and misaligned.
The vetting pipeline is substantive, but the four-name badge ladder is retired and must be replaced with trust/verdict signals.
Family install and governance: present.
Install, revoke, permission materialization, and Cedar enforcement are the strongest local surfaces.
Family runtime: present.
Wasmtime sandboxing, runtime kill switch, and invocation limits are solid, pending actual Rust implementation.
Family billing and monetization: partial.
Pricing, billing, revenue share, tutorials, and reconciliation docs exist, but tenant_class and shared billing-substrate ownership are missing.
Family ads and promotion: partial to absent.
Marketplace auction journeys exist, but no plugin-app-store contract exposes Shopify-style ads or featured/sponsored placements.
Family security and privacy: present but needs Chrome-like precision.
DPIA, compliance, SBOM, Cosign, Trivy, and Cedar exist, but limited-use declarations, narrow permission explanations, and review-risk factors are missing.
Family deployment: absent for canonical readiness.
Six-context OpenTofu, OCI Always Free, and OS manifests are required local surfaces and not counterpart-derived optionality.

## §7 Headline gap analysis

Gap headline 001: the service is strongest where it models tenant install governance and auditability.
Gap headline 002: the service is weakest where it must meet canonical deployment doctrine.
Gap headline 003: the service has no deployable six-context substrate.
Gap headline 004: the service has no supported OS declaration.
Gap headline 005: the service has no source or test implementation.
Gap headline 006: the service currently embeds retired tier vocabulary in OpenAPI, Proto, Cedar, ADR, FAQ, benchmark, tutorial, and capability-ladder artifacts.
Gap headline 007: the service does not encode `tenant_class`.
Gap headline 008: the service confuses pricing model, tenant class, revenue-share deal, and subscription plan.
Gap headline 009: the service needs a clean trusted-publisher model after the badge ladder is retired.
Gap headline 010: the service needs staged rollout and update lifecycle contracts to meet Chrome-grade and VS Code-grade release safety.
Gap headline 011: the service needs platform-specific package targets for native plugin parity with VS Code.
Gap headline 012: the service needs app-store ads or a deliberate handoff to the shared marketplace search/auction surface for Shopify-grade promotion.
Gap headline 013: the service needs Chrome-grade permission-review and user-data declaration detail.
Gap headline 014: the service needs Shopify-grade listing QA assets, demo credentials, and review lifecycle states.
Gap headline 015: the service should keep its stronger-than-counterpart audit-chain commitments.
Gap headline 016: the service should keep install-time Cedar materialization because it is a defensible local differentiator.
Gap headline 017: the service should delete or quarantine capability-ladder content during Wave 15J rather than normalizing it into new docs.
Gap headline 018: the service should rebuild counterpart docs around VS Code, Chrome, and Shopify, not Salesforce and Atlassian.
Gap headline 019: the service should add a README and cross-microservice handoff file before implementation.
Gap headline 020: the service should align all SLO numbers before capacity or staffing plans are trusted.

## §8 Additive surface for next implementation wave

Additive surface 001: add `tenant_class` to install, quota, billing, and infrastructure policy contracts.
Additive surface 002: add separate fields for `plugin_pricing_model`, `publisher_payout_model`, and `tenant_class`.
Additive surface 003: add `trusted_publisher_status` with evidence fields rather than retired four-label ladder.
Additive surface 004: add `review_risk_factors` covering dangerous permissions, broad host scopes, large package, obfuscation, external network access, and sensitive data.
Additive surface 005: add `review_assets` for demo credentials, test instructions, screenshots, screencast, privacy policy, and support contact.
Additive surface 006: add `publisher_security` fields for 2FA, domain verification, contact verification, signing identity, and associated accounts.
Additive surface 007: add `package_constraints` for max size, allowed artifacts, SBOM, signature, platform target, and runtime compatibility.
Additive surface 008: add `platform_targets` analogous to VS Code platform-specific packages but expressed in Oyatie OS/arch terms.
Additive surface 009: add `release_channel` for stable, pre-release, staged, emergency rollback, and deprecated.
Additive surface 010: add `rollout_percentage` with monotonic increase rules and audit evidence.
Additive surface 011: add `deferred_publish_until` and `approval_expires_at` for reviewer-approved but unpublished releases.
Additive surface 012: add `unpublish` distinct from irreversible remove.
Additive surface 013: add name-reservation rules after irreversible remove.
Additive surface 014: add workspace/team/tenant install scopes if Oyatie needs VS Code workspace recommendation parity.
Additive surface 015: add `recommendation_context` for workspace, role, tenant policy, and installed dependency recommendations.
Additive surface 016: add `listing_locales` and `geo_availability` for Chrome/Shopify-style distribution control.
Additive surface 017: add sponsored-placement objects only if shared marketplace search/auction does not own them.
Additive surface 018: add CPC/ad relevance handoff to marketplace if ads are out of plugin-app-store scope.
Additive surface 019: add limited-use data declaration and permission rationale to publisher submission.
Additive surface 020: add storefront/admin performance impact fields where plugins touch customer-facing surfaces.
Additive surface 021: add Lighthouse or equivalent impact budget for web UI plugins.
Additive surface 022: add admin-console load budget for governance and publisher consoles.
Additive surface 023: add OCI Always Free `demo_trial` caps as infrastructure policy, not product feature reduction.
Additive surface 024: add paid tenant elastic scaling policy tied to deployment context and contractual SLO.
Additive surface 025: add revenue_share substrate at-cost policy and gross-revenue reporting evidence.
Additive surface 026: add publisher revenue reconciliation evidence that points to finops ledger.
Additive surface 027: add audit-chain evidence for every install, revoke, review, payout, and policy override.
Additive surface 028: add explicit supported OS manifest with per-OS test status.
Additive surface 029: add OpenTofu context directories before claiming six-context deployability.
Additive surface 030: add Rust source crates and compliant tests before using PRD acceptance criteria as evidence.

## §9 Counterpart-to-local migration notes

Migration note 001: VS Code's `publisher.extension` identity should map to Oyatie publisher namespace plus plugin slug.
Migration note 002: VS Code's workspace recommendations should map to Oyatie tenant, role, installed-pack, and policy-context recommendations.
Migration note 003: VS Code's platform-specific package targets should map to Oyatie OS/arch compatibility declared in `supported-oses.json`.
Migration note 004: VS Code's verified publisher process should map to a non-tier trusted-publisher evidence object.
Migration note 005: VS Code's unpublish/remove distinction should map to separate reversible delist and irreversible removal commands.
Migration note 006: VS Code's pre-release channel should map to release-channel metadata, not tenant class.
Migration note 007: Chrome's 2GB package limit should become a signed-package maximum or lower security-reviewed maximum.
Migration note 008: Chrome's 20-extension publisher default should become an abuse-prevention default before trust history exists.
Migration note 009: Chrome's 30-day deferred publish expiry should become approved-release expiry.
Migration note 010: Chrome's monotonic rollout percentage should become plugin-update rollout policy.
Migration note 011: Chrome's review-risk taxonomy should become a first-class vetting input rather than prose-only review guidance.
Migration note 012: Chrome's user-data declaration should become a machine-readable publisher submission field.
Migration note 013: Chrome's limited-use policy should become a data-use assertion validated by policy and audit-chain evidence.
Migration note 014: Chrome's developer 2FA requirement should become a publisher-security prerequisite.
Migration note 015: Shopify's 100-checkpoint review should become vetting checklist depth, not a marketing phrase.
Migration note 016: Shopify's app billing should map to shared billing-engine and finops-ledger dependencies.
Migration note 017: Shopify's revenue-share schedule should inform Oyatie revenue_share tenant economics without copying percentages by default.
Migration note 018: Shopify's app ads should map to shared marketplace auction/search unless plugin-app-store owns listing promotion directly.
Migration note 019: Shopify's storefront performance rule should map to plugin runtime and web-surface impact budgets.
Migration note 020: Shopify's listing quality requirements should map to publisher submission validation.
Migration note 021: Local `PRD.md:50-67` already covers the core install/governance/vetting family and should be preserved.
Migration note 022: Local ADR-PAS-0001 should be preserved because install-time Cedar materialization is stronger than most counterpart public docs.
Migration note 023: Local ADR-PAS-0003 should be preserved because Wasmtime sandboxing gives a concrete runtime boundary.
Migration note 024: Local ADR-PAS-0007 should be preserved because audit-chain authority is a product differentiator.
Migration note 025: Local ADR-PAS-0004 should be retired or rewritten because its title and decision are tied to the obsolete four-name ladder.
Migration note 026: Local `competitor-parity-matrix.md` should be retained only as historical context after the current top-three matrix lands.
Migration note 027: Local `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md` should not drive Wave 3 Batch 3.2 decisions.
Migration note 028: Local FAQ/tutorial revenue-share content should be preserved after removing stale tier gates.
Migration note 029: Local catalog YAML fixtures should seed parity tests for search, install, revenue share, and compliance-pack flows.
Migration note 030: Local runbooks should be tied to counterpart-derived lifecycle events such as staged rollout, delist, appeal, and emergency disable.
