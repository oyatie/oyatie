# Application performance benchmark numbers

Audit date: 2026-05-20.
Target microservice: `application`.
Counterparts: Heroku, Vercel, Fly.io.
Benchmark posture: single industry-leader target set with deployment-context overlays and tenant_class overlays.
No retired feature-class rows are used.
No retired capability-ladder retirement deltas deliverable is authored.

Five-citation anchor block:
- Application PRD performance targets: TTI p99 <= 2s, route resolve p99 <= 100ms, OIDC callback p99 <= 200ms, module manifest load p95 <= 150ms, audit event seal p99 <= 1s: `microservices/application/PRD.md:79`.
- Application OpenSLO TTI target: p99 warm-cache under 2s: `microservices/application/slos/tti.openslo.yaml:10`.
- Application OpenSLO route resolve target: p99 under 100ms: `microservices/application/slos/route-resolve.openslo.yaml:10`.
- OCI Always Free profile resource ceiling: 4 OCPU, 24 GB memory, storage and network constraints: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514`.
- Canonical OpenTofu/deployment-context rule: six contexts must be evaluated and service IaC must use OpenTofu: `specs/master-plan-sequencing.json:704` and `specs/master-plan-sequencing.json:747`.

External-source anchor block:
- Heroku HTTP routing: https://devcenter.heroku.com/articles/http-routing
- Heroku dyno sizes: https://devcenter3.assets.heroku.com/articles/dyno-sizes
- Heroku review apps: https://devcenter.heroku.com/articles/github-integration-review-apps
- Vercel Function limits: https://vercel.com/docs/functions/limitations
- Vercel CDN overview: https://vercel.com/docs/cdn
- Vercel deployments and generated URLs: https://vercel.com/docs/deployments and https://vercel.com/docs/deployments/generated-urls
- Fly autostop/autostart: https://fly.io/docs/launch/autostop-autostart/
- Fly autoscaling: https://fly.io/docs/reference/autoscaling/
- Fly app configuration: https://fly.io/docs/reference/configuration/
- Fly regions: https://fly.io/docs/reference/regions/
- Fly pricing: https://fly.io/docs/about/pricing/

Methodology disclosure:
- Public counterpart numbers are taken from official docs available during this audit.
- Where official docs provide limits rather than benchmark latencies, the number is classified as `source: official limit`.
- Where Application targets are internal goals, the number is classified as `source: Oyatie target`.
- Where Application lacks implementation evidence, the comparison status is `evidence-needed` rather than asserted as measured.
- The current service benchmark file is treated as historical input only because it uses non-mandated counterparts and retired access-class rows.
- No number in this document is a measured Application production result unless explicitly marked as measured.
- All Application targets require future source, load-test, and OpenSLO evidence before being used in an external claim.

## 1. Methodology

Benchmark dimension 01: shell time to interactive.
Reason: Application PRD makes TTI a primary user-facing performance target.
Metric: p50, p95, p99 warm-cache browser shell TTI.
Workload: authenticated tenant shell bootstrap with app switcher, route manifest, signed module manifest, and one default module shell.
Target OS: primary Linux server OS set plus browser clients.
Target architecture: linux/amd64 and linux/arm64 minimum; arm64 is required for OCI Always Free relevance.
Target deployment contexts: all six canonical contexts.
Tenant_class disclosure: demo_trial uses OCI Always Free profile caps; paid and revenue_share can scale by contract and usage envelope.

Benchmark dimension 02: route resolution latency.
Reason: Application must route before bundle load.
Metric: p50, p95, p99 latency for tenant context load, Cedar permit, route lookup, and response.
Workload: 1 KB route resolve request with JWT/session context and one pack overlay.
Canonical target: p99 <= 100ms, p95 <= 60ms, p50 <= 20ms.
Context overlay: on-prem and colo depend on facility network latency but must meet in-cell service budget.
Tenant_class overlay: tenant_class affects caps, not route resolver quality.

Benchmark dimension 03: module manifest load latency.
Reason: module loading gates product shell render.
Metric: p50, p95, p99 latency for signed manifest fetch and integrity metadata return.
Workload: one signed module manifest, one bundle pointer, one SRI hash, one signer key id.
Canonical target: p95 <= 150ms, p99 <= 250ms.
Context overlay: CDN-backed public contexts should beat on-prem first-hit paths.
Tenant_class overlay: demo_trial can cap module count but not weaken integrity validation.

Benchmark dimension 04: OIDC/SAML callback latency.
Reason: user sign-in is critical path for shell entry.
Metric: p50, p95, p99 after IdP callback reaches Application.
Workload: callback verify, nonce/state check, session creation, audit event enqueue.
Canonical target: p99 <= 200ms, p95 <= 120ms, p50 <= 50ms.
Context overlay: external IdP network time measured separately.
Tenant_class overlay: demo_trial has best-effort SLO but same code path.

Benchmark dimension 05: audit event seal latency.
Reason: Application promises audit-chain evidence.
Metric: p50, p95, p99 from event creation to accepted seal request.
Workload: route-resolved, route-denied, module-loaded, policy-draft, and JIT review events.
Canonical target: p99 <= 1s.
Context overlay: disconnected on-prem/air-gap may buffer locally but must preserve ordering and seal evidence.
Tenant_class overlay: compliance packs allowed for paid and revenue_share may add retention obligations, not slower shell path.

Benchmark dimension 06: concurrent active sessions.
Reason: Application is a tenant shell; session fan-out matters more than raw generic compute.
Metric: active sessions per tenant, active sessions per cell, and active sessions per deployment context.
Workload: connected browser sessions with periodic route refresh, session heartbeat, and app-switcher queries.
Canonical target: 50,000 active sessions per standard cell group before horizontal scale.
Context overlay: OCI Always Free profile is capped materially lower by 4 OCPU and 24 GB memory.
Tenant_class overlay: demo_trial caps concurrent sessions; paid and revenue_share scale with contract.

Benchmark dimension 07: request throughput.
Reason: route resolve, status, manifest, and auth endpoints must handle tenant load.
Metric: sustained RPS and burst RPS.
Workload: 70 percent route resolve, 20 percent module manifest, 5 percent status, 5 percent auth callback synthetic mix.
Canonical target: 10,000 sustained shell-control RPS per production cell group, 50,000 burst for 60 seconds.
Context overlay: OCI Always Free profile target is capped at 500 sustained RPS and 1,500 burst until measured.
Tenant_class overlay: demo_trial gets hard usage caps; paid and revenue_share can buy/earn higher envelopes.

Benchmark dimension 08: preview/review environment creation.
Reason: Heroku and Vercel make previews a first-class developer workflow.
Metric: time from PR/branch event to accessible protected preview shell URL.
Workload: one Application shell preview, one tenant seed, one module manifest.
Canonical target: <= 180 seconds to first protected preview URL.
Context overlay: on-prem/colo previews may use local registry mirrors.
Tenant_class overlay: preview environments are internal/dev workflow, not customer tenant_class.

Benchmark dimension 09: deployment promotion/rollback.
Reason: Vercel and Heroku make release workflows operationally visible.
Metric: time to promote and time to rollback signed module pointer.
Workload: one shell version, one module pointer, one CDN purge, one audit event.
Canonical target: rollback <= 60 seconds after decision, promotion <= 5 minutes after gates pass.
Context overlay: air-gapped environments use signed bundle import and may not meet public-cloud elapsed time.
Tenant_class overlay: customer tenant_class does not change rollback quality.

Benchmark dimension 10: CDN purge.
Reason: Application serves frontend bundles and module manifests.
Metric: p95 and p99 time to purge/update globally.
Workload: one module bundle pointer replacement and asset invalidation.
Canonical target: p95 <= 60 seconds, p99 <= 120 seconds.
Context overlay: on-prem/colo may use local edge cache invalidation instead of global CDN.
Tenant_class overlay: demo_trial shares cache infrastructure but does not lower integrity checks.

## 2. Counterpart numbers

### 2.1 Heroku official numbers

Heroku number H01: initial web response timeout is 30 seconds.
Source: official Heroku HTTP Routing docs.
Use in comparison: Application's own 10-second deadline is intentionally stricter.

Heroku number H02: rolling timeout after response bytes is 55 seconds.
Source: official Heroku HTTP Routing docs.
Use in comparison: Application should stream only when workflow handoff requires it; shell-control endpoints should finish faster.

Heroku number H03: router idle connection timeout is 90 seconds.
Source: official Heroku HTTP Routing docs.
Use in comparison: Application should set explicit keepalive/idle limits in api-gateway handoff.

Heroku number H04: Cedar-generation entry dyno memory begins at 0.5 GB.
Source: official Heroku dyno specs.
Use in comparison: Application demo_trial OCI Always Free profile has much larger aggregate memory ceiling but shared across services.

Heroku number H05: Cedar Standard-2X dyno is 1 GB and 2x CPU share.
Source: official Heroku dyno specs.
Use in comparison: useful low-end paid shell target.

Heroku number H06: Cedar Performance-M dyno is 2.5 GB and 100 percent CPU.
Source: official Heroku dyno specs.
Use in comparison: useful single-tenant performance baseline.

Heroku number H07: Cedar Performance-L dyno is 14 GB.
Source: official Heroku dyno specs.
Use in comparison: paid tenant Application shell should scale beyond this by horizontal cell replicas.

Heroku number H08: Cedar Performance-2XL dyno is 126 GB and 100x compute.
Source: official Heroku dyno specs.
Use in comparison: large tenant cell groups should compare to this only with multi-replica capacity.

Heroku number H09: Fir Classic `dyno-1c-0.5gb` has 1 vCPU and 0.5 GB memory.
Source: official Heroku dyno specs.
Use in comparison: arm/amd64 target tests should include tiny memory profile smoke checks.

Heroku number H10: Fir General Purpose reaches 16 vCPU and 64 GB.
Source: official Heroku dyno specs.
Use in comparison: Application paid/revenue_share public-cloud cell group should meet or beat with horizontal scale.

Heroku number H11: Fir Compute reaches 32 vCPU and 64 GB.
Source: official Heroku dyno specs.
Use in comparison: compute-heavy auth/module operations should not depend on Application; route out to worker services.

Heroku number H12: Fir Memory reaches 16 vCPU and 128 GB.
Source: official Heroku dyno specs.
Use in comparison: Application should remain stateless and avoid requiring memory-heavy single instances.

Heroku number H13: review app inactivity teardown options include 1, 2, 5, 14, or 30 days.
Source: official Heroku Review Apps docs.
Use in comparison: Application preview shell should support explicit teardown policies.

Heroku number H14: review apps are charged while they exist and can use low-cost add-ons.
Source: official Heroku Review Apps docs.
Use in comparison: Application previews should expose cost caps.

Heroku number H15: review apps can be created automatically when pull requests open.
Source: official Heroku Review Apps docs.
Use in comparison: Application currently has no equivalent target; proposed target is <= 180 seconds.

### 2.2 Vercel official numbers

Vercel number V01: Functions concurrency auto-scales up to 30,000 for Hobby and Pro.
Source: official Vercel Functions limits.
Use in comparison: Application target should treat 30,000 concurrent route/control operations as an industry bar for public-cloud scale tests.

Vercel number V02: Enterprise Functions concurrency is 100,000+.
Source: official Vercel Functions limits.
Use in comparison: Application paid/revenue_share large tenants need a multi-cell target path.

Vercel number V03: Function default max duration with Fluid Compute is 300 seconds.
Source: official Vercel Functions limits.
Use in comparison: Application intentionally does not keep shell-control requests open this long.

Vercel number V04: Pro and Enterprise Function max duration can reach 800 seconds.
Source: official Vercel Functions limits.
Use in comparison: long-running tasks belong to workflow-engine, not Application.

Vercel number V05: Edge Runtime must begin sending response within 25 seconds for streaming.
Source: official Vercel Functions limits.
Use in comparison: Application route resolution target is much stricter at p99 <= 100ms.

Vercel number V06: Edge Runtime can stream up to 300 seconds.
Source: official Vercel Functions limits.
Use in comparison: Application shell streams should be limited to status/progress surfaces, not route resolve.

Vercel number V07: Function memory default is 2 GB / 1 vCPU.
Source: official Vercel Functions limits.
Use in comparison: Application per-pod default can be smaller if Rust memory profile is proven.

Vercel number V08: Pro/Enterprise Function memory maximum is 4 GB / 2 vCPU.
Source: official Vercel Functions limits.
Use in comparison: Application should beat this with horizontal Rust services, not per-request memory.

Vercel number V09: Vercel CDN has 126 PoPs.
Source: official Vercel CDN docs.
Use in comparison: Application cannot claim edge parity unless network/CDN handoff provides comparable footprint.

Vercel number V10: Vercel CDN spans 94 cities.
Source: official Vercel CDN docs.
Use in comparison: Application deployment-context overlay must state public-cloud edge footprint.

Vercel number V11: Vercel CDN spans 51 countries.
Source: official Vercel CDN docs.
Use in comparison: Application geo claims require source-backed CDN/provider data.

Vercel number V12: Vercel maintains 20 compute-capable regions behind its PoPs.
Source: official Vercel CDN docs.
Use in comparison: Application cell-region count must be explicit before parity claims.

Vercel number V13: every deployment gets a unique URL.
Source: official Vercel deployment docs.
Use in comparison: Application preview shell target must generate protected URLs.

Vercel number V14: Vercel creates deployments for commits or pull requests from supported Git providers.
Source: official Vercel Git docs.
Use in comparison: Application currently lacks this workflow.

Vercel number V15: generated deployment URLs are public by default but can be private with deployment protection.
Source: official Vercel generated URL docs.
Use in comparison: Application preview shell must default to protected for tenant data.

### 2.3 Fly.io official numbers

Fly number F01: Fly listed 18 public regions in the official region table during this audit.
Source: official Fly regions docs.
Use in comparison: Application must cite network/cell region count before global placement claims.

Fly number F02: gateway regions are marked separately for WireGuard/private network access.
Source: official Fly regions docs.
Use in comparison: Application private-network claims need a network handoff.

Fly number F03: `auto_stop_machines` can be `off`, `stop`, or `suspend`.
Source: official Fly autostop/autostart docs.
Use in comparison: Application demo_trial idle behavior should be explicit.

Fly number F04: `auto_start_machines` controls whether Fly Proxy starts Machines on requests and capacity.
Source: official Fly autostop/autostart docs.
Use in comparison: Application cannot rely on hidden wakeup behavior.

Fly number F05: `min_machines_running` controls running Machines in the primary region.
Source: official Fly autostop/autostart docs.
Use in comparison: Application needs a warm-floor field per deployment_context.

Fly number F06: default `auto_stop_machines` is `off` if unset.
Source: official Fly configuration docs.
Use in comparison: Application should avoid accidental scale-to-zero in paid/revenue_share contexts.

Fly number F07: default `auto_start_machines` is `true` if unset.
Source: official Fly configuration docs.
Use in comparison: Application should encode restart behavior rather than assume it.

Fly number F08: default `min_machines_running` is `0` if unset.
Source: official Fly configuration docs.
Use in comparison: Application demo_trial can choose 0 only when cold-start target is accepted.

Fly number F09: default concurrency type is `connections` when unspecified.
Source: official Fly configuration docs.
Use in comparison: Application should choose request-based soft limits for HTTP shell traffic.

Fly number F10: default soft concurrency limit is 20 when unset.
Source: official Fly configuration docs.
Use in comparison: Application should define explicit per-pod soft/hard limits.

Fly number F11: Fly example VM config shows `shared-cpu-2x`, `1gb`, and `2` CPUs.
Source: official Fly configuration docs.
Use in comparison: Application small-cell tests should include 2 CPU / 1 GB equivalent.

Fly number F12: Fly supports autostop/autostart and metrics-based autoscaling.
Source: official Fly autoscaling docs.
Use in comparison: Application needs both event-driven and metrics-driven scaling design.

Fly number F13: metrics-based autoscaler can create/delete Machines or stop/start existing Machines.
Source: official Fly autoscaling docs.
Use in comparison: Application must state whether cloud-iac/cell can create/delete or only scale replicas.

Fly number F14: running Machine price is named CPU/RAM preset plus roughly $5 per 30 days per GB additional RAM.
Source: official Fly pricing docs.
Use in comparison: Application cost model should be resource envelope plus tenant_class billing, not old access-class rows.

Fly number F15: stopped/suspended Machines do not bill CPU and RAM.
Source: official Fly autostop/autostart docs.
Use in comparison: Application demo_trial cost controls should account for idle state.

## 3. Oyatie target numbers: single industry-leader target set

Metric O01: shell TTI warm p50.
Canonical target: <= 350ms.
Deployment-context overlay: public-cloud and guest-cloud contexts target <= 350ms; on-prem/colo target <= 500ms when local edge is used; OCI Always Free profile target <= 750ms.
Tenant_class overlay: demo_trial cap affects concurrent sessions, not code path; paid and revenue_share target same latency.
Source: Oyatie target derived from PRD p99 <= 2s and Vercel/Linear-style UX pressure.

Metric O02: shell TTI warm p95.
Canonical target: <= 1,200ms.
Deployment-context overlay: public-cloud target <= 1,200ms; on-prem/colo target <= 1,500ms; OCI Always Free profile target <= 1,800ms.
Tenant_class overlay: demo_trial has best-effort SLO but same engineering target.
Source: Oyatie target.

Metric O03: shell TTI warm p99.
Canonical target: <= 2,000ms.
Deployment-context overlay: all contexts target <= 2,000ms after local edge and cache are warm; disconnected on-prem may publish separate local-only measurement.
Tenant_class overlay: paid/revenue_share contractual SLO can bind this; demo_trial reports best-effort but target is unchanged.
Source: PRD and OpenSLO.

Metric O04: route resolve p50.
Canonical target: <= 20ms.
Deployment-context overlay: in-cell target unchanged; cross-region route should not be on hot path.
Tenant_class overlay: unchanged across classes.
Source: Oyatie target under PRD p99 <= 100ms.

Metric O05: route resolve p95.
Canonical target: <= 60ms.
Deployment-context overlay: OCI Always Free profile target <= 90ms under capped concurrency.
Tenant_class overlay: demo_trial capped before saturation; paid/revenue_share scale out.
Source: Oyatie target.

Metric O06: route resolve p99.
Canonical target: <= 100ms.
Deployment-context overlay: all connected contexts target <= 100ms in-cell.
Tenant_class overlay: unchanged across classes.
Source: OpenSLO route-resolve.

Metric O07: OIDC/SAML callback p50.
Canonical target: <= 50ms after provider callback reaches Application.
Deployment-context overlay: external IdP time excluded and measured separately.
Tenant_class overlay: unchanged across classes.
Source: Oyatie target.

Metric O08: OIDC/SAML callback p95.
Canonical target: <= 120ms after provider callback reaches Application.
Deployment-context overlay: OCI Always Free target <= 180ms under capped login RPS.
Tenant_class overlay: demo_trial login bursts capped; paid/revenue_share contract can raise caps.
Source: Oyatie target.

Metric O09: OIDC/SAML callback p99.
Canonical target: <= 200ms after provider callback reaches Application.
Deployment-context overlay: on-prem IdP connector measured separately.
Tenant_class overlay: unchanged code path.
Source: PRD.

Metric O10: module manifest fetch p50.
Canonical target: <= 40ms.
Deployment-context overlay: public CDN-backed contexts target <= 40ms; local on-prem cache target <= 60ms.
Tenant_class overlay: demo_trial may cap module count but not integrity work.
Source: Oyatie target.

Metric O11: module manifest fetch p95.
Canonical target: <= 150ms.
Deployment-context overlay: OCI Always Free profile target <= 200ms under capped RPS.
Tenant_class overlay: unchanged quality.
Source: PRD.

Metric O12: module manifest fetch p99.
Canonical target: <= 250ms.
Deployment-context overlay: on-prem/colo local-cache first-hit may be <= 400ms but must be separately labeled.
Tenant_class overlay: unchanged quality.
Source: Oyatie target.

Metric O13: audit seal p50.
Canonical target: <= 100ms.
Deployment-context overlay: disconnected contexts enqueue locally and seal when link restores; local append target remains <= 100ms.
Tenant_class overlay: paid/revenue_share may have stricter retention, not slower app path.
Source: Oyatie target.

Metric O14: audit seal p95.
Canonical target: <= 500ms.
Deployment-context overlay: remote seal path measured separately from local durable append.
Tenant_class overlay: unchanged.
Source: Oyatie target.

Metric O15: audit seal p99.
Canonical target: <= 1,000ms.
Deployment-context overlay: public-cloud connected path target <= 1,000ms; disconnected contexts publish local-append metric.
Tenant_class overlay: demo_trial receives same tamper-evidence path.
Source: OpenSLO audit seal.

Metric O16: module load success ratio.
Canonical target: >= 99.9 percent excluding Cedar-deny.
Deployment-context overlay: all contexts same target after deployment admission.
Tenant_class overlay: demo_trial may deny unavailable paid/compliance surfaces by policy, not count as failed load.
Source: OpenSLO module-load.

Metric O17: route denied false-negative rate.
Canonical target: 0 known false negatives.
Deployment-context overlay: all contexts.
Tenant_class overlay: all classes.
Source: security target.

Metric O18: disabled module bundle load attempts.
Canonical target: 0 successful disabled loads.
Deployment-context overlay: all contexts.
Tenant_class overlay: all classes.
Source: ADR-APP-001 verification list.

Metric O19: CDN purge p95.
Canonical target: <= 60 seconds.
Deployment-context overlay: on-prem/colo local edge purge has local target <= 30 seconds; public global edge target <= 60 seconds.
Tenant_class overlay: unchanged.
Source: IP-015 HG-APP claim.

Metric O20: CDN purge p99.
Canonical target: <= 120 seconds.
Deployment-context overlay: global public edge only; local contexts publish local purge.
Tenant_class overlay: unchanged.
Source: Oyatie target.

Metric O21: protected preview shell creation.
Canonical target: <= 180 seconds from PR/branch event.
Deployment-context overlay: public-cloud target <= 180s; on-prem/colo target <= 600s with local artifact mirror.
Tenant_class overlay: internal/dev workflow, not customer class.
Source: counterpart-derived target from Heroku/Vercel.

Metric O22: protected preview shell teardown.
Canonical target: <= 60 seconds after close/expiry decision.
Deployment-context overlay: local contexts may use queued cleanup when disconnected.
Tenant_class overlay: internal/dev workflow.
Source: counterpart-derived target.

Metric O23: signed module rollback.
Canonical target: <= 60 seconds after operator decision in connected contexts.
Deployment-context overlay: disconnected on-prem uses signed-bundle rollback and local audit.
Tenant_class overlay: unchanged.
Source: Vercel rollback pressure plus Application module rollback event.

Metric O24: deployment promotion after gates.
Canonical target: <= 5 minutes after required gates pass.
Deployment-context overlay: air-gapped/on-prem can exceed elapsed time due artifact transfer, but local apply target remains <= 5 minutes after artifact arrival.
Tenant_class overlay: unchanged.
Source: Oyatie target.

Metric O25: sustained shell-control throughput.
Canonical target: 10,000 RPS per production cell group.
Deployment-context overlay: OCI Always Free profile target <= 500 RPS sustained until measured; on-prem depends on provisioned hardware; public-cloud scales horizontally.
Tenant_class overlay: demo_trial hard cap; paid/revenue_share scale by contract.
Source: Oyatie target.

Metric O26: burst shell-control throughput.
Canonical target: 50,000 RPS for 60 seconds per production cell group.
Deployment-context overlay: OCI Always Free profile target <= 1,500 RPS burst until measured.
Tenant_class overlay: demo_trial burst quota capped; paid/revenue_share negotiated.
Source: Oyatie target.

Metric O27: active sessions per production cell group.
Canonical target: 50,000 active sessions.
Deployment-context overlay: OCI Always Free profile target <= 5,000 active sessions until measured.
Tenant_class overlay: demo_trial session cap; paid/revenue_share contract cap.
Source: PRD scalability target.

Metric O28: monthly active users per deployment group.
Canonical target: 5,000,000 monthly users with horizontal scale.
Deployment-context overlay: OCI Always Free profile not intended for this scale.
Tenant_class overlay: demo_trial capped by policy; paid/revenue_share scale with economics.
Source: PRD scalability target.

Metric O29: module manifest max payload.
Canonical target: <= 128 KiB compressed per module manifest.
Deployment-context overlay: all contexts.
Tenant_class overlay: unchanged.
Source: Oyatie target to protect TTI.

Metric O30: shell WASM gzip size.
Canonical target: <= 2 MiB gzip for shell frontend.
Deployment-context overlay: all contexts.
Tenant_class overlay: unchanged.
Source: IP-014 test plan.

Metric O31: Cedar evaluation budget.
Canonical target: p99 <= 5ms for route permit evaluation.
Deployment-context overlay: all contexts; policy cache local.
Tenant_class overlay: tenant_class adds attributes but not remote calls.
Source: Oyatie target.

Metric O32: feature flag snapshot injection p95.
Canonical target: <= 25ms after context load.
Deployment-context overlay: all contexts.
Tenant_class overlay: tenant_class may be input attribute but not remote lookup.
Source: ADR-APP-001 SLO target.

Metric O33: flag snapshot staleness p95.
Canonical target: <= 60 seconds.
Deployment-context overlay: disconnected contexts require local invalidation record.
Tenant_class overlay: unchanged.
Source: ADR-APP-001 SLO target.

Metric O34: startup readiness.
Canonical target: <= 30 seconds for Application pod/process readiness in warmed environment.
Deployment-context overlay: OCI Always Free and on-prem hardware may vary; readiness must be measured per context.
Tenant_class overlay: not customer-facing.
Source: Oyatie target against Heroku 30s request timeout and preview workflow pressure.

Metric O35: memory per route resolver replica.
Canonical target: <= 256 MiB steady-state at 1,000 RPS synthetic route mix.
Deployment-context overlay: OCI Always Free profile benefits from low Rust memory profile.
Tenant_class overlay: unchanged.
Source: Oyatie target; requires measurement.

Metric O36: CPU per route resolver replica.
Canonical target: <= 1 vCPU at 1,000 RPS synthetic route mix p95 <= 60ms.
Deployment-context overlay: all contexts; public-cloud can scale replicas.
Tenant_class overlay: unchanged.
Source: Oyatie target; requires measurement.

Metric O37: per-tenant usage cap enforcement latency.
Canonical target: cap decision p99 <= 25ms when local cap snapshot is warm.
Deployment-context overlay: all contexts.
Tenant_class overlay: demo_trial hard cap; paid/revenue_share contract cap.
Source: tenant_class adoption target.

Metric O38: per-tenant quota update propagation.
Canonical target: <= 60 seconds p95 for cap/contract changes to affect route/module admission.
Deployment-context overlay: disconnected on-prem uses signed local policy bundle.
Tenant_class overlay: all classes.
Source: tenant_class adoption target.

Metric O39: status endpoint p99.
Canonical target: <= 100ms from Application when dependencies healthy.
Deployment-context overlay: all contexts.
Tenant_class overlay: unchanged.
Source: public status endpoint target.

Metric O40: health/readiness endpoint p99.
Canonical target: <= 25ms local.
Deployment-context overlay: all contexts.
Tenant_class overlay: unchanged.
Source: liveness/readiness target.

## 4. Comparison narrative

Comparison C01: request timeout.
Heroku official number: 30s initial response and 55s rolling window.
Vercel official number: 300s default Function duration and up to 800s for Pro/Enterprise Function duration.
Fly official number: request duration is app/runtime-defined; proxy autostop/autostart affects machine availability.
Oyatie target: 10s Application deadline with much lower p99 route/auth/module budgets.
Status: ahead for shell-control latency if implemented; evidence-needed for runtime proof.

Comparison C02: frontend edge footprint.
Vercel official number: 126 PoPs, 94 cities, 51 countries, 20 compute-capable regions.
Fly official number: 18 listed regions, with global Anycast.
Heroku official number: dyno routing is platform-managed but not a comparable global CDN footprint.
Oyatie target: must be supplied by network/api-gateway/CDN handoff; Application alone cannot claim parity.
Status: catch-up at Application artifact level.

Comparison C03: preview workflow.
Heroku official number: review apps can be automatic per PR and teardown after 1, 2, 5, 14, or 30 days inactivity.
Vercel official number: every deployment gets unique URL, PRs get preview deployments.
Fly official number: app deploy and regional runtime but not the same PR-preview product surface.
Oyatie target: protected preview shell URL <= 180 seconds and teardown <= 60 seconds.
Status: catch-up.

Comparison C04: runtime scale.
Vercel official number: 30,000 concurrency for Hobby/Pro and 100,000+ Enterprise Functions.
Heroku official number: dyno sizes range from 0.5 GB small units to 126 GB large units.
Fly official number: autoscaling can start/stop existing Machines or use metrics autoscaler.
Oyatie target: 10,000 sustained RPS, 50,000 burst RPS, 50,000 active sessions per production cell group.
Status: parity target; evidence-needed.

Comparison C05: constrained demo/trial infrastructure.
Heroku official number: low-end dynos start at 0.5 GB memory.
Vercel official number: Hobby/Pro functions use 2 GB / 1 vCPU default.
Fly official number: example small Machine shape is shared-cpu-2x with 1 GB memory and 2 CPUs; stopped/suspended Machines avoid CPU/RAM billing.
Oyatie target: OCI Always Free profile cap, 4 OCPU and 24 GB memory shared profile, 500 sustained RPS until measured.
Status: potentially ahead on aggregate free-profile resources, but catch-up because no `iac/oci-guest/always-free/` exists.

Comparison C06: tenant-aware routing.
Heroku/Vercel/Fly provide app/platform routing, not Oyatie's tenant/product context route decision.
Oyatie target: p99 route resolve <= 100ms with Cedar, pack/residency, feature flag snapshot, and audit evidence.
Status: ahead in product semantics; evidence-needed for implementation.

Comparison C07: module integrity.
Heroku/Vercel/Fly deploy apps/assets but do not provide Oyatie's per-product Ed25519 module manifest model.
Oyatie target: p95 manifest <= 150ms, p99 <= 250ms, disabled module successful load count 0.
Status: ahead in design; evidence-needed.

Comparison C08: cost model.
Heroku and Fly publish resource-based pricing units; Vercel publishes function/resource limits and platform pricing elsewhere.
Oyatie target: cost-budget by deployment_context, tenant_class, and usage envelope.
Status: catch-up because current cost-budget preserves old access framing.

Comparison C09: observability.
Heroku/Vercel/Fly expose logs, deployment views, and platform metrics.
Oyatie target: OpenSLO-native metrics, audit-chain event seal, dashboards, and burn-rate gates.
Status: ahead in design; evidence-needed for metrics emission.

Comparison C10: global private/on-prem contexts.
Heroku private spaces and Fly private networking provide partial analogues; Vercel focuses public edge/frontend.
Oyatie target: six contexts including on-prem, colo, and Oyatie-as-cloud-provider.
Status: catch-up because no context OpenTofu directories exist.

## 5. Measurement prerequisites before public claims

Prerequisite P01: create Application source and tests or update manifest status to non-GA.
Prerequisite P02: implement route resolver benchmark fixture.
Prerequisite P03: implement module manifest benchmark fixture.
Prerequisite P04: implement shell TTI browser benchmark fixture.
Prerequisite P05: implement auth callback benchmark fixture with external IdP time separated.
Prerequisite P06: implement audit-chain seal benchmark fixture with local append and remote seal separated.
Prerequisite P07: add deployment_context labels to metrics.
Prerequisite P08: add tenant_class labels or documented observability join strategy.
Prerequisite P09: create OpenTofu modules for all six contexts or cite delegated modules.
Prerequisite P10: create OCI Always Free profile module and measured saturation envelope.
Prerequisite P11: replace retired access rows in old benchmark and cost docs.
Prerequisite P12: add Heroku/Vercel/Fly migration and benchmark tests.
Prerequisite P13: add preview/review shell lifecycle benchmark.
Prerequisite P14: add protected generated URL benchmark.
Prerequisite P15: store signed benchmark evidence under a service-owned path.

## 6. Summary target table

| Metric | Canonical target | Demo/trial cap | Paid cap | Revenue-share cap | Claim status |
|---|---:|---:|---:|---:|---|
| Shell TTI p99 warm | <= 2,000ms | same target, capped load | contractual | contract/economics | target only |
| Route resolve p99 | <= 100ms | same target, capped RPS | contractual | contract/economics | target only |
| Module manifest p95 | <= 150ms | same target | contractual | contract/economics | target only |
| OIDC callback p99 | <= 200ms | same target | contractual | contract/economics | target only |
| Audit seal p99 | <= 1,000ms | same target | contractual | contract/economics | target only |
| Sustained RPS | 10,000 per production cell group | 500 until measured | scales with payment | scales at cost | target only |
| Burst RPS | 50,000 for 60s | 1,500 until measured | scales with payment | scales at cost | target only |
| Active sessions | 50,000 per production cell group | 5,000 until measured | contract | contract | target only |
| Preview shell creation | <= 180s | internal/dev | internal/dev | internal/dev | missing feature |
| Rollback | <= 60s connected | same target | same target | same target | partial design |
| CDN purge p95 | <= 60s | same target | same target | same target | partial design |
| OCI Always Free module | present and planned | required | not default | optional economics | missing |

Final benchmark judgment:
Application has credible internal targets for latency, integrity, and audit behavior.
Application does not yet have credible measured results.
Application must not reuse the historical benchmark document as the batch benchmark because that document uses retired access rows and a different counterpart set.
The next benchmark-ready milestone is source/test availability plus six-context OpenTofu evidence.
