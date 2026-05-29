# Application feature parity matrix

Audit date: 2026-05-20.
Target microservice: `application`.
Counterpart 1: Heroku.
Counterpart 2: Vercel.
Counterpart 3: Fly.io.
Deliverable rule: no capability-ladder retirement deltas document.
Replacement framing: tenant_class plus deployment-context overlay.
Product boundary: Application is Oyatie's tenant-aware application shell, route resolver, module loader, auth gateway, and tenant admin control surface.
Non-goal: Application is not a full generic compute PaaS by itself.
Method: compare the current Application artifact set against the union of Heroku, Vercel, and Fly.io capabilities that matter to a tenant application entry surface.

Internal evidence anchors:
- Product purpose: `microservices/application/PRD.md:25`.
- Tenant value: `microservices/application/PRD.md:44`.
- Bounded contexts: `microservices/application/manifest.json:6`.
- Route and flag injection ADR: `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md:55`.
- Existing non-mandated competitor set: `microservices/application/competitor-parity-matrix.md:27`.
- Existing benchmark set drift: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:1`.
- OpenAPI contract: `microservices/application/contracts/openapi/application.yaml:1`.
- AsyncAPI contract: `microservices/application/contracts/asyncapi/application-events.yaml:1`.
- OpenSLO TTI: `microservices/application/slos/tti.openslo.yaml:10`.
- OpenSLO route resolve: `microservices/application/slos/route-resolve.openslo.yaml:10`.

External counterpart source anchors:
- Heroku HTTP routing timeout and router behavior: https://devcenter.heroku.com/articles/http-routing
- Heroku dyno specifications: https://devcenter3.assets.heroku.com/articles/dyno-sizes
- Heroku Review Apps: https://devcenter.heroku.com/articles/github-integration-review-apps
- Vercel deployments overview: https://vercel.com/docs/deployments
- Vercel Git deployments and preview deployments: https://vercel.com/docs/git
- Vercel generated deployment URLs: https://vercel.com/docs/deployments/generated-urls
- Vercel CDN overview: https://vercel.com/docs/cdn
- Vercel Functions limits: https://vercel.com/docs/functions/limitations
- Fly.io autostop/autostart Machines: https://fly.io/docs/launch/autostop-autostart/
- Fly.io autoscaling reference: https://fly.io/docs/reference/autoscaling/
- Fly.io app configuration and concurrency: https://fly.io/docs/reference/configuration/
- Fly.io regions: https://fly.io/docs/reference/regions/
- Fly.io pricing and machine resource model: https://fly.io/docs/about/pricing/

Legend:
- `Covered`: Application artifacts already define the capability with credible internal ownership.
- `Partial`: artifacts mention or imply the capability but lack exact implementation, evidence, or canonical context coverage.
- `Missing`: no current Application artifact covers it.
- `External`: capability belongs to another Oyatie microservice or platform layer, but Application must integrate.
- `Retire`: current artifact uses retired access-class framing and must be rewritten.

## Counterpart 1 capability surface: Heroku

Heroku surface H01: app-centric deployment abstraction.
Source: Heroku dynos are the unit used to run app processes and can be resized or scaled.
Application parity: Partial.
Evidence: Application has Helm/Kustomize charts but no OpenTofu deployment-context modules.
Gap: Application cannot yet present a Heroku-like "deploy this tenant shell" abstraction across all six contexts.

Heroku surface H02: process formation and dyno sizing.
Source: Heroku docs list dyno memory/CPU classes from 0.5 GB through 126 GB in Cedar and Fir generations.
Application parity: Partial.
Evidence: capacity-model defines shell, auth, module-loader, storage, CDN, and capacity envelope budgets.
Gap: capacity budgets are not mapped to deployment_context and tenant_class.

Heroku surface H03: request routing with explicit timeout behavior.
Source: Heroku router gives a 30-second initial response window and rolling 55-second windows after response bytes.
Application parity: Covered.
Evidence: FAQ defines api-gateway 15 s, Application 10 s, downstream 5 s, DB 1 s timeout chain at `microservices/application/faqs/platform-engineer-faq.md:94`.
Gap: timeout chain needs implementation tests because no source/tests exist.

Heroku surface H04: review apps for every pull request.
Source: Heroku Review Apps run PR code in disposable apps with unique URLs.
Application parity: Missing.
Evidence: current Application docs do not define PR preview shell environments.
Gap: Vercel-style preview and Heroku-style review app parity is not in Application artifacts.

Heroku surface H05: disposable app lifecycle.
Source: Heroku review apps can be destroyed automatically after inactivity.
Application parity: Missing.
Evidence: no Application lifecycle doc defines automatic teardown of preview shells.
Gap: needs work with cloud-iac, cell, and tenancy.

Heroku surface H06: app config via environment and platform settings.
Application parity: Partial.
Evidence: IP-014 composition root loads config from environment and OpenBao at `microservices/application/IP-014-leptos-frontend-and-composition.md:63`.
Gap: no OpenAPI/admin surface for tenant shell runtime config lifecycle.

Heroku surface H07: add-ons ecosystem.
Application parity: External.
Evidence: Application integrates downstream services through cross-handoffs rather than owning add-ons.
Gap: module registry and tenant admin console need an equivalent "enabled service dependency" UX.

Heroku surface H08: logs and request tracing.
Application parity: Covered.
Evidence: tutorial traces Application through audit-chain and observability at `microservices/application/tutorials/dispatch-intent-and-trace-through-cells.md:81`.
Gap: no implemented observability emission proof.

Heroku surface H09: pipelines and promote-to-production.
Application parity: Partial.
Evidence: IP-015 names branch-protection rules for release/application staging and production at `microservices/application/IP-015-application-openslo-and-hg-app.md:23`.
Gap: no pipeline promotion contract under Application itself.

Heroku surface H10: operational limits surfaced to developers.
Application parity: Partial.
Evidence: capacity-model provides specific budgets.
Gap: no public limit document for Application analogous to Heroku platform limits.

Heroku surface H11: CLI-driven workflows.
Application parity: Partial.
Evidence: tutorial uses `./bin/oya` for cell, tenant, policy, obs, and audit flows.
Gap: CLI examples still carry retired access-class command flags.

Heroku surface H12: private spaces and isolated runtime.
Application parity: Partial.
Evidence: FAQ says production requires Cloud Hypervisor plus Kata containers for cross-tenant traffic at `microservices/application/faqs/platform-engineer-faq.md:87`.
Gap: no six-context IaC proves private/dedicated/on-prem/colo deployment.

Heroku surface H13: managed database/service attachments.
Application parity: External.
Evidence: Application depends on tenancy, identity, audit-chain, observability, cloud-secrets, cloud-iac, and other services.
Gap: attachment model is spread across handoffs, not a tenant-admin workflow.

Heroku surface H14: app health and readiness endpoints.
Application parity: Covered.
Evidence: OpenAPI defines `/health` and `/ready`.
Gap: no readiness implementation or tests.

Heroku surface H15: buildpack/container compatibility.
Application parity: Not applicable as a direct product feature.
Evidence: Application is a Rust/Leptos service, not a generic user-app runner.
Gap: none for Application boundary; generic build/deploy belongs elsewhere.

## Counterpart 2 capability surface: Vercel

Vercel surface V01: unique deployment URLs.
Source: Vercel generates a unique URL for preview or production deployments.
Application parity: Partial.
Evidence: Application OpenAPI server URL is per tenant and pack: `https://{tenant}.app-{pack}.oyatie.com/api/v1`.
Gap: generated preview URLs for per-commit or per-branch shell previews are absent.

Vercel surface V02: preview deployments on pull requests.
Source: Vercel creates preview deployments for every push and unique deployments for PRs.
Application parity: Missing.
Evidence: no Application artifact defines preview shell environment creation.
Gap: needs cloud-iac/cell integration and tenant sandbox lifecycle.

Vercel surface V03: global CDN.
Source: Vercel CDN has 126 PoPs in 94 cities across 51 countries and routes to 20 compute-capable regions.
Application parity: Partial.
Evidence: PRD requires CDN purge under 60 seconds and frontend bundle serve.
Gap: Application docs do not define global PoP count, edge placement, or CDN provider abstraction.

Vercel surface V04: framework-aware deployment.
Application parity: Partial.
Evidence: IP-014 names Leptos WASM frontend and composition-root binaries.
Gap: build and deploy integration for Leptos SSR/selective island hydration is planned, not implemented.

Vercel surface V05: serverless functions.
Application parity: Not a direct Application-owned feature.
Evidence: Application owns route resolution and module loading, not arbitrary tenant function execution.
Gap: if Oyatie wants Vercel function parity, it belongs in a functions/edge/workflow substrate with Application routing integration.

Vercel surface V06: function concurrency scaling.
Source: Vercel Functions docs state concurrency auto-scales up to 30,000 for Hobby/Pro and 100,000+ for Enterprise.
Application parity: Missing as a verified number.
Evidence: capacity model gives 50,000 active sessions and 5,000,000 monthly users, but no execution-concurrency proof.
Gap: benchmark doc must define target concurrency with deployment_context overlay.

Vercel surface V07: function duration limits.
Source: Vercel Functions with Fluid Compute have 300s defaults and higher Pro/Enterprise maximums; Edge Runtime must start sending within 25s and can stream up to 300s.
Application parity: Covered for Application requests by shorter service deadlines.
Evidence: Application FAQ defines 10s Application deadline.
Gap: long-running jobs should route to workflow-engine, not Application.

Vercel surface V08: generated URL protection.
Source: Vercel generated URLs are publicly accessible by default but can be private using deployment protection.
Application parity: Partial.
Evidence: OpenAPI and Cedar public-read policies distinguish anonymous status and asset fetch.
Gap: preview URL protection model is absent.

Vercel surface V09: instant rollback by deployment.
Source: Vercel Git docs describe instant rollbacks when reverting changes assigned to a custom domain.
Application parity: Partial.
Evidence: AsyncAPI defines `application.module.rolled.back` event.
Gap: module rollback exists as event shape, but no deployment rollback workflow is implemented.

Vercel surface V10: observability/log links per deployment.
Application parity: Partial.
Evidence: dashboards exist for module load, session active, and TTI.
Gap: dashboards are not tied to deployment IDs or preview URLs.

Vercel surface V11: edge cache purge and no-downtime release.
Application parity: Partial.
Evidence: OpenAPI defines CDN purge request and IP-015 includes CDN purge drill claim.
Gap: global purge under 60s is claimed but not proven.

Vercel surface V12: deployment API.
Application parity: External.
Evidence: Application has OpenAPI for shell operations, not deployment creation.
Gap: deployment-control-plane/cloud-iac should expose deploy APIs and Application should consume outputs.

Vercel surface V13: project/team member collaboration.
Application parity: Partial.
Evidence: tenant admin console exists for tenant-local controls.
Gap: no project/team collaboration workflow equivalent to Vercel projects.

Vercel surface V14: static asset optimization.
Application parity: Partial.
Evidence: frontend-bundle-serve and module manifest contain SRI and bundle URL fields.
Gap: no image/font/static optimization contract.

Vercel surface V15: branch-based environment separation.
Application parity: Missing.
Evidence: Application branch-protection mentions release/application staging and production, but branch preview shells are absent.
Gap: add branch environment model or point to cloud-iac.

## Counterpart 3 capability surface: Fly.io

Fly surface F01: global Anycast routing.
Source: Fly says users connect to nearest server through global Anycast network.
Application parity: External/Partial.
Evidence: FAQ says api-gateway handles anycast and Application handles tenant dispatch.
Gap: Application must cite api-gateway/network handoff for global routing proof.

Fly surface F02: machine-based runtime abstraction.
Source: Fly Machines are app instances in regions.
Application parity: Partial.
Evidence: Application docs use cells and Cloud Hypervisor pods.
Gap: no exact per-context cell/runtime OpenTofu modules.

Fly surface F03: regional placement.
Source: Fly supports apps in many regions and exposes `FLY_REGION`.
Application parity: Partial.
Evidence: Application uses cell_id and pack_residency fields in route responses.
Gap: no deployment_context and region-placement manifest.

Fly surface F04: autostop/autostart.
Source: Fly Proxy can stop/suspend Machines when idle and start them on demand.
Application parity: Missing for Application itself.
Evidence: no Application doc defines idle stop/start policy.
Gap: demo_trial tenant_class could use bounded idle scale-down, but not currently specified.

Fly surface F05: minimum running machines.
Source: Fly config supports `min_machines_running`.
Application parity: Partial.
Evidence: old retired access docs define warm pools, but those must be retired.
Gap: warm floor must move to deployment_context plus tenant_class usage cap.

Fly surface F06: soft/hard concurrency routing.
Source: Fly `http_service.concurrency` has soft and hard limit settings.
Application parity: Partial.
Evidence: capacity model defines request and active-session budgets.
Gap: no concrete per-route hard/soft concurrency configuration.

Fly surface F07: private networking.
Source: Fly provides WireGuard/private network surfaces.
Application parity: External.
Evidence: Application relies on network/cell layers.
Gap: handoff to network microservice needs exact private-network capability citation.

Fly surface F08: dynamic request routing.
Source: Fly supports routing requests to regions/fallbacks using request routing headers.
Application parity: Covered conceptually.
Evidence: Application route resolver and cell dispatch decide downstream routing.
Gap: exact region/fallback API is absent.

Fly surface F09: machine migration and host failure handling.
Application parity: External.
Evidence: failure-modes cover route resolver, module loader, and auth failures, not host migration.
Gap: cell microservice should own host migration; Application should cite failure cascade behavior.

Fly surface F10: app-level custom domains and TLS.
Application parity: Partial.
Evidence: OpenAPI server uses tenant/pack host pattern and requires host/JWT match.
Gap: no custom-domain onboarding flow is defined in Application artifacts.

Fly surface F11: per-region volumes.
Application parity: Not direct.
Evidence: PRD says Application is stateless and downstream services own hard state.
Gap: none if stateless boundary is preserved.

Fly surface F12: local CLI and deploy experience.
Application parity: Partial.
Evidence: tutorial uses `oya` CLI.
Gap: deployment CLI path is missing and examples need tenant_class correction.

Fly surface F13: resource pricing per Machine.
Source: Fly pricing is based on named CPU/RAM preset plus RAM.
Application parity: Partial.
Evidence: cost-budget exists, but uses old class language.
Gap: cost-budget needs context/class/usage model.

Fly surface F14: multi-region resilient apps.
Application parity: Partial.
Evidence: multi-region-topology doc exists.
Gap: topology needs OpenTofu context proof and no retired access classes.

Fly surface F15: organization/app isolation.
Application parity: Covered conceptually.
Evidence: tenant_id, pack, Cedar, and module isolation are first-class.
Gap: actual implementation and tests absent.

## Union coverage matrix

| # | Union capability | Heroku | Vercel | Fly.io | Application status | Evidence | Gap class |
|---:|---|---|---|---|---|---|---|
| 1 | Tenant-aware product shell | Low | Medium | Low | Covered | PRD:25, ADR-APP-001:55 | source absent |
| 2 | Route resolution before bundle load | Medium | Medium | Medium | Covered | ADR-APP-001:57 | tests absent |
| 3 | Cedar/default-deny route authorization | Low | Low | Low | Covered | OpenAPI:6, policy dir | policy refs drift |
| 4 | Signed module manifests | Low | Low | Low | Covered | OpenAPI ModuleManifest:105 | source absent |
| 5 | SRI bundle verification | Medium | Medium | Low | Covered | OpenAPI ModuleManifest:111 | tests absent |
| 6 | Per-tenant origin | Medium | Medium | Medium | Covered | OpenAPI server:24 | custom domain flow missing |
| 7 | OIDC/SAML auth flow | Medium | Medium | Medium | Covered | OpenAPI auth paths:179 | implementation absent |
| 8 | Session lifecycle events | Medium | Medium | Medium | Covered | AsyncAPI session-started:31 | event implementation absent |
| 9 | Module load events | Low | Medium | Low | Covered | AsyncAPI module-loaded:45 | event tests absent |
| 10 | Route denied events | Medium | Medium | Medium | Covered | AsyncAPI route-access-denied:59 | route-resolved missing |
| 11 | CDN purge | Medium | High | Medium | Partial | OpenAPI CdnPurgeRequest:127 | global proof missing |
| 12 | Preview/review environments | High | High | Medium | Missing | no artifact | product gap |
| 13 | Unique deployment URL | High | High | Medium | Partial | per-tenant server URL:24 | preview URL absent |
| 14 | Branch-based preview | High | High | Medium | Missing | no artifact | product gap |
| 15 | Instant rollback | Medium | High | Medium | Partial | AsyncAPI rollback:73 | workflow absent |
| 16 | Runtime scale envelope | High | High | High | Partial | capacity-model:195 | no context overlay |
| 17 | Autoscale soft/hard limits | Medium | High | High | Partial | capacity docs | config absent |
| 18 | Autostop/autostart | Low | Medium | High | Missing | no artifact | demo cap gap |
| 19 | Minimum warm floor | Medium | Medium | High | Retire | old access-class docs | replace model |
| 20 | Request timeout policy | High | High | Medium | Covered | FAQ:94 | tests absent |
| 21 | Long-running function policy | Medium | High | Medium | Covered by routing out | FAQ timeout chain | workflow handoff needed |
| 22 | Global routing/Anycast | Medium | High | High | External | FAQ gateway split:8 | handoff proof missing |
| 23 | Regional placement | Medium | High | High | Partial | cell_id fields | OpenTofu absent |
| 24 | Private networking | High | Medium | High | External | network dependency | explicit citation missing |
| 25 | Custom domains/TLS | High | High | High | Partial | host/JWT matching | lifecycle missing |
| 26 | Review app teardown | High | Medium | Medium | Missing | no artifact | lifecycle missing |
| 27 | App config lifecycle | High | Medium | Medium | Partial | OpenBao load in IP-014 | admin config missing |
| 28 | Add-on/dependency attachment | High | Medium | Medium | External | handoffs | tenant UX missing |
| 29 | Logs and traces | High | High | High | Partial | tutorial trace:81 | implementation absent |
| 30 | Audit-chain evidence | Medium | Medium | Low | Covered | AsyncAPI consumers:10 | source absent |
| 31 | Per-tenant quota | Medium | High | High | Missing | tenant_class absent | cloud-billing handoff needed |
| 32 | Billing model overlay | High | High | High | Missing | no tenant_class | replacement gap |
| 33 | Usage caps for trial/demo | Medium | High | High | Missing | no tenant_class | OCI profile gap |
| 34 | Contractual SLO surface | Medium | High | Medium | Partial | OpenSLO docs | class/context labels absent |
| 35 | Best-effort SLO surface | Medium | Medium | Medium | Missing | no tenant_class | demo semantics absent |
| 36 | Compliance pack gating | High | Medium | Low | Partial | compliance docs | tenant_class gates absent |
| 37 | BYOK gating | High | Medium | Medium | Retire | retired class file | rewrite needed |
| 38 | Tenant admin console | Medium | Medium | Low | Covered | IP-016:12 | implementation absent |
| 39 | Policy simulation | Low | Low | Low | Covered | IP-016:14 | implementation absent |
| 40 | Public status endpoint | High | High | High | Covered | OpenAPI /status:168 | implementation absent |
| 41 | Health/readiness probes | High | High | High | Covered | OpenAPI /health,/ready | implementation absent |
| 42 | OpenSLO-native SLOs | Low | Medium | Medium | Covered | slos/*.openslo.yaml | metrics absent |
| 43 | Multi-window burn alerts | Medium | Medium | Medium | Partial | IP-015 | wiring absent |
| 44 | Deployment API | Medium | High | High | External | cloud-iac dep | no handoff detail |
| 45 | Runtime source language policy | Medium | Medium | Medium | Partial | Rust/Leptos IPs | source absent |
| 46 | SDK reference example | Medium | Medium | High | Partial | Rust SDK doc | retired access residue |
| 47 | CLI tutorial | High | High | High | Partial | tutorial doc | retired access residue |
| 48 | Migration playbook | High | Medium | Medium | Partial | AWS App Runner playbook | Heroku/Vercel/Fly missing |
| 49 | Cost model | High | High | High | Partial | cost-budget.md | old access model |
| 50 | OS support manifest | Medium | Medium | Medium | Missing | no supported-oses.json | canonical gap |
| 51 | OpenTofu per-context deploy | Medium | Medium | Medium | Missing | no context dirs | canonical gap |
| 52 | OCI Always Free profile | Low | Medium | Medium | Missing | no path | demo infra gap |
| 53 | On-prem deployment | Low | Medium | Medium | Missing | no on-prem iac | canonical gap |
| 54 | Colo deployment | Low | Low | Medium | Missing | no colo iac | canonical gap |
| 55 | Oyatie-as-cloud-provider deployment | Low | Low | Medium | Missing | no oyatie-iaas iac | canonical gap |
| 56 | Guest-on-AWS deployment | High | Medium | Medium | Missing | no guest-on-aws iac | canonical gap |
| 57 | Guest-on-OCI deployment | Medium | Medium | Medium | Missing | no oci-guest iac | canonical gap |
| 58 | Public-cloud first-party deployment | High | High | High | Missing | no oyatie-public-cloud iac | canonical gap |
| 59 | Data residency in routing | Medium | Medium | Medium | Covered conceptually | RouteResolveResponse pack_residency | source absent |
| 60 | Pack overlay awareness | Low | Medium | Low | Covered conceptually | AsyncAPI `pack` fields | class migration needed |
| 61 | Module kill switch | Medium | Medium | Medium | Covered conceptually | ADR-APP-001:178 | endpoint absent from OpenAPI |
| 62 | App switcher | Low | Medium | Low | Covered conceptually | ADR-APP-001:78 | OpenAPI path absent |
| 63 | Feature flag snapshot injection | Medium | High | Medium | Covered conceptually | ADR-APP-001:61 | schema absent |
| 64 | OpenFeature-compatible context | Medium | High | Medium | Covered conceptually | ADR-APP-001:62 | source absent |
| 65 | Third-party flag avoidance | Low | Medium | Low | Covered | ADR-APP-001:64 | test absent |
| 66 | Deployment protection for previews | Medium | High | Medium | Missing | no artifact | product gap |
| 67 | Generated URL privacy controls | Medium | High | Medium | Missing | no artifact | product gap |
| 68 | Source/test readiness | High | High | High | Missing | no src/tests | implementation gap |
| 69 | Contract path correctness | High | High | High | Partial | handoff refs drift | contract gap |
| 70 | Benchmarked public claims | High | High | High | Partial | old benchmark | evidence gap |

## Family summary

Family A: Tenant shell and product routing.
Status: Application is ahead of the counterparts on tenant-aware product shell semantics.
Reason: Heroku, Vercel, and Fly.io expose app/platform deployment primitives; Application exposes tenant context, Cedar route authorization, module manifests, pack/residency overlays, and audit-chain events.
Risk: those claims remain documentation-only until source/tests land.

Family B: Developer deployment workflow.
Status: Application is behind Heroku and Vercel.
Reason: no review app, preview deployment, generated preview URL, deployment protection, branch environment, or disposable shell lifecycle is defined.
Risk: a platform engineer migrating from Heroku/Vercel will expect these first.

Family C: Global runtime placement.
Status: Application is behind Fly.io and Vercel at the service-owned artifact level.
Reason: Application delegates global routing to api-gateway/network/cell but lacks exact handoff and per-context OpenTofu proof.
Risk: all-context deployability remains unclaimable.

Family D: Runtime scaling and caps.
Status: partial.
Reason: capacity-model has useful service budgets, but no tenant_class caps, no deployment_context overlays, and no soft/hard concurrency settings.
Risk: demo_trial, paid, and revenue_share behavior cannot be enforced consistently.

Family E: Contract and policy surfaces.
Status: strong but inconsistent.
Reason: OpenAPI/AsyncAPI/proto/Cedar docs exist, but cross-handoff paths and event references drift.
Risk: integration consumers cannot verify end-to-end names.

Family F: Operational posture.
Status: strong documentation, weak executable evidence.
Reason: OpenSLOs, dashboards, runbooks, capacity model, failure modes, and incident response exist.
Risk: no source/tests/metrics prove the docs.

Family G: Canonical platform constraints.
Status: failing.
Reason: six-context OpenTofu layout, OCI Always Free profile, supported OS manifest, and tenant_class model are absent.
Risk: canonical direction claims are not supportable.

Family H: Retired access-class cleanup.
Status: failing.
Reason: tutorials, migration, benchmark, FAQ, SDK reference, and capability directory retain retired access-class names.
Risk: Wave 15J retirement cannot succeed if Application remains a prominent old-model example.

## Headline gap analysis

Gap 1: no review/preview app parity.
Counterpart pressure: Heroku and Vercel both make per-PR disposable environments a first-class workflow.
Application state: no equivalent artifact.
Recommended owner: Application plus cloud-iac plus cell.
Priority: P1 if customer migration pitch includes Heroku/Vercel parity; P2 otherwise.

Gap 2: no generated preview URL model.
Counterpart pressure: Vercel generated deployment URLs are central to review workflows.
Application state: only per-tenant endpoint template exists.
Recommended owner: Application plus network plus cloud-iac.
Priority: P2.

Gap 3: no service-owned OpenTofu contexts.
Counterpart pressure: all three counterparts make deployment target concrete.
Application state: Helm/Kustomize only.
Recommended owner: cloud-iac integration lane plus Application variable contracts.
Priority: P1.

Gap 4: no OCI Always Free profile.
Counterpart pressure: demo/trial economics require hard infra caps.
Application state: no path, no capacity cap overlay.
Recommended owner: Application plus cloud-iac.
Priority: P1.

Gap 5: no tenant_class adoption.
Counterpart pressure: billing and usage controls are fundamental to all three counterparts.
Application state: old access-class residue and no tenant_class field.
Recommended owner: tenancy/cloud-billing for source of truth; Application for projection and enforcement behavior.
Priority: P1.

Gap 6: benchmark evidence is not reusable.
Counterpart pressure: public platform comparisons must cite public docs and reproducible tests.
Application state: old benchmark has non-mandated counterparts, retired access rows, and missing evidence path.
Recommended owner: Application performance lane.
Priority: P1.

Gap 7: source/test absence.
Counterpart pressure: Heroku/Vercel/Fly all center working deployable app artifacts.
Application state: implementation plans without source tree.
Recommended owner: Application implementation lane.
Priority: P1.

Gap 8: handoff drift.
Counterpart pressure: app platforms hide integration complexity; Oyatie must prove exact internal contracts.
Application state: missing policy refs and AsyncAPI path mismatches.
Recommended owner: Application contract lane.
Priority: P1.

Gap 9: retired access-class language.
Counterpart pressure: new commercial model is tenant_class; user directive explicitly retires old classes.
Application state: tutorial, FAQ, benchmark, SDK, migration, and retired directory preserve old terms.
Recommended owner: Wave 15J cleanup lane.
Priority: P2.

Gap 10: OS matrix.
Counterpart pressure: deployment claims require OS support clarity.
Application state: no supported-oses manifest.
Recommended owner: Application platform lane.
Priority: P1.

## Additive surface

Additive surface A01: Preview Shell.
Description: per-branch/per-PR tenant shell environment with generated URL, deployment protection, teardown policy, and traceable audit label.
Counterpart basis: Heroku Review Apps and Vercel Preview Deployments.
Oyatie-specific twist: preview shells must preserve tenant context and Cedar policy simulation.
Required artifacts: OpenAPI admin endpoint, cloud-iac OpenTofu module input, DNS route, audit event, runbook, SLO label.

Additive surface A02: Context Deployment Descriptor.
Description: machine-readable per-deployment_context support and OpenTofu module descriptor.
Counterpart basis: Heroku dyno/runtime classes, Vercel deployment environments, Fly regional machine placement.
Oyatie-specific twist: six canonical contexts, not just public cloud.
Required artifacts: `iac/<context>/main.tf`, variables, outputs, versions, admission evidence, context manifest.

Additive surface A03: Tenant Class Projection.
Description: Application consumes tenant_class from tenancy/cloud-billing and uses it for caps, SLO labels, compliance/BYOK gating, and cost attribution.
Counterpart basis: paid/free usage plans across app platforms.
Oyatie-specific twist: quality is uniform; only caps, SLO contract, and commercial terms change.
Required artifacts: schema, AsyncAPI field or observability join rule, admin read model, SLO labels, docs.

Additive surface A04: Signed Module Release Channel.
Description: Vercel-style immutable deployments but for signed modules in the Application shell.
Counterpart basis: Vercel generated deployments and rollback.
Oyatie-specific twist: Ed25519 manifest, SRI, Cedar route permits, audit-chain event.
Required artifacts: route for module release, event for module release created, rollback runbook, dashboard.

Additive surface A05: Regional Dispatch Overlay.
Description: Fly-style regional request placement expressed as tenant/cell/pack dispatch policy.
Counterpart basis: Fly regions and dynamic routing.
Oyatie-specific twist: cell_id plus pack_residency plus compliance overlay.
Required artifacts: routing policy schema, route-resolved event, region/cell label in SLOs.

Additive surface A06: Application Limits Page.
Description: public/internal limits page for request timeout, route-resolve latency, module size, max modules per tenant, sessions, CDN purge, and concurrency.
Counterpart basis: Heroku and Vercel limits docs.
Oyatie-specific twist: deployment-context and tenant_class overlays.
Required artifacts: limits doc, machine-readable limit manifest, tenant admin view.

Additive surface A07: Migration Playbooks for Mandated Counterparts.
Description: Heroku-to-Application, Vercel-to-Application, and Fly-to-Application migration playbooks.
Counterpart basis: this batch's union-coverage bar.
Oyatie-specific twist: do not model Application as generic PaaS; map app tenant RBAC workflows to tenant shell and cell runtime.
Required artifacts: three playbooks plus risk register.

Additive surface A08: Handoff Contract Linter.
Description: linter that verifies cross-microservice-handoffs paths against OpenAPI/AsyncAPI/proto/Cedar files.
Counterpart basis: platform providers hide contract drift; Oyatie needs explicit proof.
Oyatie-specific twist: all handoff rows must cite existing service-owned or delegated artifacts.
Required artifacts: linter rule, CI evidence, repaired handoff refs.

Additive surface A09: Context/Class Aware SLO Labels.
Description: add deployment_context and tenant_class label strategy to Application SLOs and dashboards.
Counterpart basis: platform plan and region observability.
Oyatie-specific twist: tenant_class does not lower quality; it controls contract/cap interpretation.
Required artifacts: OpenSLO updates, dashboard updates, metric emission spec.

Additive surface A10: Source Reality Gate.
Description: manifest cannot mark implementation plans as GA unless source and tests exist.
Counterpart basis: deployable platforms expose working artifacts.
Oyatie-specific twist: BNF layer and Rust-strict checks.
Required artifacts: manifest status schema, source existence check, cargo lane evidence.

## Closing parity judgment

Application is directionally strong as a tenant-aware product shell.
Application is not yet coherent as a deployable all-context microservice.
Application is ahead of Heroku/Vercel/Fly on signed tenant module-routing semantics.
Application is behind Heroku/Vercel/Fly on developer deployment workflow.
Application is behind Fly/Vercel on explicit global runtime placement evidence.
Application is behind Heroku/Vercel on preview/review app workflow.
Application must not preserve old access-class semantics while adopting tenant_class.
Application needs the three-part repair path: canonical deployment substrate, tenant_class/classless commercial model, and implementation evidence.
