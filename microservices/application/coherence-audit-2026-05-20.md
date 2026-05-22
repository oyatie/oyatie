# Application microservice ownership-coherence audit

Audit date: 2026-05-20.
Target microservice: `application`.
Microservice path: `microservices/application/`.
Audit owner: single-agent application audit lane.
Deliverable set: coherence audit, feature parity matrix, performance benchmark numbers.
Retired deliverable: capability-ladder deltas document; not authored.
Counterpart set mandated for this batch: Heroku, Vercel, Fly.io.
Deployable-context assumption: all six canonical contexts unless evidence proves otherwise.
Current outcome: not ready to claim all six deployment contexts.
Current outcome: not ready to claim OpenTofu-per-context deployability.
Current outcome: not ready to claim tenant_class adoption.
Current outcome: not ready to claim OS support matrix coverage.
Current outcome: ready to preserve the product boundary as the tenant product shell once the gaps below are corrected.

Evidence anchor block:
- Canonical multi-context doctrine requires six deployment contexts and per-context evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1736`.
- Canonical OpenTofu doctrine requires context directories under service-owned IaC: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2277`.
- Master plan fixes OpenTofu as the IaC engine and forbids Terraform, Pulumi, CloudFormation, ARM/Bicep, local-exec, remote-exec, and manual console claims: `specs/master-plan-sequencing.json:747`.
- Master plan fixes primary OS coverage and architecture matrix: `specs/master-plan-sequencing.json:777`.
- Master plan fixes Rust backend and frontend allowlist: `specs/master-plan-sequencing.json:817`.
- Tenant-class replacement directive is the current batch instruction: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10`.
- Capability tiers are retired by doctrine: `feedback_no_capability_ladder_2026_05_20.md:10`.
- Audit deliverables must be verified by substance, not line count alone: `feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10`.
- Application product purpose is the tenant front door and modular product shell: `microservices/application/PRD.md:25`.
- Application route and flag injection authority is recorded in ADR-APP-001: `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md:53`.
- Chat history contains the generated Wave 3 Batch 3.2 application prompt and counterpart set: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16171`.
- Chat history shows the application batch process was launched: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16174`.
- Chat history also contains a later tenant-class correction discussion; this audit follows the current user-provided three-class directive for this batch: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16232`.

## 1. Purpose

This audit asks whether the `application` microservice artifacts are coherent with the current Oyatie direction.
The product purpose is clear: the service is the tenant-aware application shell, not a generic north-south gateway.
The PRD says Application owns one tenant origin, SSO, route authorization, module isolation, signed module loading, and tenant-admin enablement: `microservices/application/PRD.md:44`.
ADR-APP-001 says it resolves tenant context before route matching, resolves session and ACR class before manifest fetch, evaluates Cedar before loading bundles, and injects per-tenant feature flag snapshots: `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md:55`.
That boundary is materially different from Heroku, Vercel, and Fly.io.
Heroku is an application PaaS surface with dynos, pipelines, add-ons, and review apps.
Vercel is an edge/frontend deployment platform with preview deployments, global CDN, functions, and framework-driven build/deploy flows.
Fly.io is a global Machines platform with anycast routing, regional placement, autostop/autostart, and private networking.
Oyatie Application is closer to a tenant product-entry shell and module-control plane.
The correct parity bar is therefore union coverage over deployment, preview/review app, runtime routing, tenant shell, module loading, global routing, auth, observability, and operations.
The audit does not try to turn Application into a generic compute PaaS.
The audit checks whether Application has enough artifacts to honestly claim that a tenant-facing product shell can be deployed and operated across all canonical contexts.
The answer is not yet.
The strongest evidence is product/architecture depth, contract breadth, and SLO/runbook coverage.
The weakest evidence is canonical deployment substrate, OS matrix, tenant_class adoption, and source/test reality.
The most urgent repair is to create the six OpenTofu context directories and the OCI Always Free profile module owned by the application surface or by explicit cross-service handoff to `cloud-iac`.
The second urgent repair is to replace retired tier semantics with tenant_class semantics in docs, examples, and machine-readable manifest surfaces.
The third urgent repair is to reconcile manifest/IP claims with actual `src/` and `tests/` absence.
The fourth urgent repair is to fix cross-microservice handoff citations that point to missing policy files or invalid AsyncAPI paths.
The fifth urgent repair is to align the competitive set to Heroku, Vercel, and Fly.io for this batch.

Scope control:
- This audit wrote only files under `microservices/application/`.
- This audit did not touch other microservices.
- This audit did not update shared doctrine docs.
- This audit did not author a retired capability-ladder retirement deltas deliverable.
- This audit did not make a commit.
- This audit treats existing dirty or untracked artifacts as pre-existing user/workflow-owned content.
- This audit uses the current batch directive for three tenant classes: `demo_trial`, `paid`, and `revenue_share`.
- This audit flags existing retired four-label ladder content only as retirement candidates.
- This audit uses "OCI Always Free profile" for constrained demo/trial infrastructure.
- This audit uses "primary OS set" rather than introducing new feature tiers.

Severity model:
- P0 means a production-safety blocker with an immediate false or dangerous claim.
- P1 means a gate-blocking coherence failure for canonical deployability, contracts, source reality, or operating claim.
- P2 means a documentation or migration gap that must be retired or corrected before claims mature.
- P3 means a cleanup, clarity, or future-proofing issue.

## 2. Inventory

Inventory method: `rg --files microservices/application | sort`.
Inventory count: 135 files.
Inventory line count: 16,633 total lines from `find microservices/application -type f -print0 | xargs -0 wc -l`.
No `README.md` exists in `microservices/application/`; the top-level docs are PRD, architecture, phase plan, capacity, cost, compliance, incident, failure, DPIA, manifest, benchmark, parity, and generated support docs.
No `src/` tree exists under `microservices/application/`.
No `tests/` tree exists under `microservices/application/`.
No `supported-oses.json` exists under `microservices/application/`.
No canonical `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-iaas/` directory exists.
The IaC inventory contains Helm and Kustomize only.
The OpenTofu keyword appears in compliance prose only: `microservices/application/compliance.md:857`.

Complete file inventory:
- `microservices/application/ARCHITECTURE.md`
- `microservices/application/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/application/IP-001-shell-routing-kernel.md`
- `microservices/application/IP-002-shell-routing-domain.md`
- `microservices/application/IP-003-shell-routing-usecase.md`
- `microservices/application/IP-004-shell-routing-adapter.md`
- `microservices/application/IP-005-shell-routing-rest.md`
- `microservices/application/IP-006-tenant-context-kernel.md`
- `microservices/application/IP-007-tenant-context-usecase-rest.md`
- `microservices/application/IP-008-auth-gateway-kernel-domain.md`
- `microservices/application/IP-009-auth-gateway-adapters-oidc-saml.md`
- `microservices/application/IP-010-auth-gateway-rest-worker.md`
- `microservices/application/IP-011-module-loader-kernel-domain.md`
- `microservices/application/IP-012-module-loader-usecase-adapter-cdn.md`
- `microservices/application/IP-013-frontend-bundle-serve.md`
- `microservices/application/IP-014-leptos-frontend-and-composition.md`
- `microservices/application/IP-015-application-openslo-and-hg-app.md`
- `microservices/application/IP-016-tenant-admin-console-control-surface.md`
- `microservices/application/IP-journey-j100-pack-rollout-first-action.md`
- `microservices/application/IP-journey-j16-a11y-substrate-signup-shell.md`
- `microservices/application/IP-journey-j91-us-msb-mtl-overlay.md`
- `microservices/application/IP-journey-j92-br-lgpd-us-parent-dsar.md`
- `microservices/application/IP-journey-j93-in-dpdpa-rbi-overlay.md`
- `microservices/application/IP-journey-j94-sox404-public-company-controls.md`
- `microservices/application/IP-journey-j95-iso27001-soc2-annual-audit.md`
- `microservices/application/IP-journey-j96-ksa-uae-mena-onboarding.md`
- `microservices/application/IP-journey-j97-sg-pdpa-mas-tenant.md`
- `microservices/application/IP-journey-j98-au-privacy-apra-cps234.md`
- `microservices/application/IP-journey-j99-multi-pack-conflict-resolution.md`
- `microservices/application/PHASE-01-APPLICATION-SHELL-LANDING.md`
- `microservices/application/PRD.md`
- `microservices/application/backfill-replay.md`
- `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md`
- `microservices/application/capabilities/module-load.yaml`
- `microservices/application/capabilities/session-emit.yaml`
- `microservices/application/capabilities/shell-render.yaml`
- `microservices/application/capabilities/tenant-admin-console-control.yaml`
- `microservices/application/capability-ladders/tier-matrix.md`
- `microservices/application/capacity-model.md`
- `microservices/application/catalog/oya-application-auth-gateway-adapter-oidc.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-adapter-saml.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-adapter.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-api.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-app.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-domain.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-kernel.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-rest.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-usecase.yaml`
- `microservices/application/catalog/oya-application-auth-gateway-worker.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-adapter-cdn.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-adapter-postgres.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-adapter.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-api.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-app.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-kernel.yaml`
- `microservices/application/catalog/oya-application-frontend-bundle-serve-usecase.yaml`
- `microservices/application/catalog/oya-application-module-loader-adapter-cdn.yaml`
- `microservices/application/catalog/oya-application-module-loader-adapter.yaml`
- `microservices/application/catalog/oya-application-module-loader-api.yaml`
- `microservices/application/catalog/oya-application-module-loader-app.yaml`
- `microservices/application/catalog/oya-application-module-loader-domain.yaml`
- `microservices/application/catalog/oya-application-module-loader-kernel.yaml`
- `microservices/application/catalog/oya-application-module-loader-rest.yaml`
- `microservices/application/catalog/oya-application-module-loader-sdk.yaml`
- `microservices/application/catalog/oya-application-module-loader-usecase.yaml`
- `microservices/application/catalog/oya-application-shell-frontend.yaml`
- `microservices/application/catalog/oya-application-shell-routing-adapter.yaml`
- `microservices/application/catalog/oya-application-shell-routing-api.yaml`
- `microservices/application/catalog/oya-application-shell-routing-app.yaml`
- `microservices/application/catalog/oya-application-shell-routing-domain.yaml`
- `microservices/application/catalog/oya-application-shell-routing-kernel.yaml`
- `microservices/application/catalog/oya-application-shell-routing-rest.yaml`
- `microservices/application/catalog/oya-application-shell-routing-sdk.yaml`
- `microservices/application/catalog/oya-application-shell-routing-usecase.yaml`
- `microservices/application/catalog/oya-application-tenant-admin-console-api.yaml`
- `microservices/application/catalog/oya-application-tenant-admin-console-app.yaml`
- `microservices/application/catalog/oya-application-tenant-admin-console-domain.yaml`
- `microservices/application/catalog/oya-application-tenant-admin-console-kernel.yaml`
- `microservices/application/catalog/oya-application-tenant-admin-console-rest.yaml`
- `microservices/application/catalog/oya-application-tenant-admin-console-usecase.yaml`
- `microservices/application/catalog/oya-application-tenant-context-adapter.yaml`
- `microservices/application/catalog/oya-application-tenant-context-api.yaml`
- `microservices/application/catalog/oya-application-tenant-context-app.yaml`
- `microservices/application/catalog/oya-application-tenant-context-domain.yaml`
- `microservices/application/catalog/oya-application-tenant-context-kernel.yaml`
- `microservices/application/catalog/oya-application-tenant-context-rest.yaml`
- `microservices/application/catalog/oya-application-tenant-context-usecase.yaml`
- `microservices/application/competitor-parity-matrix.md`
- `microservices/application/compliance.md`
- `microservices/application/contracts/asyncapi/application-events.yaml`
- `microservices/application/contracts/openapi/application.yaml`
- `microservices/application/contracts/openapi/tenant-admin-console.yaml`
- `microservices/application/contracts/proto/application.proto`
- `microservices/application/cost-budget.md`
- `microservices/application/cross-microservice-handoffs.md`
- `microservices/application/dashboards/module-load-success.json`
- `microservices/application/dashboards/session-active.json`
- `microservices/application/dashboards/tti-distribution.json`
- `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md`
- `microservices/application/dpia.md`
- `microservices/application/failure-modes.md`
- `microservices/application/faqs/platform-engineer-faq.md`
- `microservices/application/iac/helm/cdn-controller/Chart.yaml`
- `microservices/application/iac/helm/cdn-controller/values.yaml`
- `microservices/application/iac/helm/postgres/Chart.yaml`
- `microservices/application/iac/helm/postgres/values.yaml`
- `microservices/application/iac/helm/shell-app/Chart.yaml`
- `microservices/application/iac/helm/shell-app/templates/deployment.yaml`
- `microservices/application/iac/helm/shell-app/templates/networkpolicy.yaml`
- `microservices/application/iac/helm/shell-app/values.yaml`
- `microservices/application/iac/kustomize/base/kustomization.yaml`
- `microservices/application/iac/kustomize/overlays/pack-kr/kustomization.yaml`
- `microservices/application/incident-response.md`
- `microservices/application/manifest.json`
- `microservices/application/migration-playbooks/from-aws-app-runner.md`
- `microservices/application/multi-region-topology.md`
- `microservices/application/onboarding/platform-engineer-first-week.md`
- `microservices/application/policy/auditor-scope.cedar`
- `microservices/application/policy/ci-scope.cedar`
- `microservices/application/policy/data-residency.md`
- `microservices/application/policy/public-read.cedar`
- `microservices/application/policy/route-isolation.md`
- `microservices/application/policy/schema.cedarschema`
- `microservices/application/policy/tenant-admin-console.cedar`
- `microservices/application/policy/tenant-scope.cedar`
- `microservices/application/reference-implementations/dispatch-intent-rust-sdk.md`
- `microservices/application/runbooks/incident-dispatch-failure.md`
- `microservices/application/scorecards/application-hg-app.yaml`
- `microservices/application/sdk-plan.md`
- `microservices/application/slos/audit-seal.openslo.yaml`
- `microservices/application/slos/module-load.openslo.yaml`
- `microservices/application/slos/oidc-signin.openslo.yaml`
- `microservices/application/slos/route-resolve.openslo.yaml`
- `microservices/application/slos/tti.openslo.yaml`
- `microservices/application/threat-model.md`
- `microservices/application/tutorials/dispatch-intent-and-trace-through-cells.md`

Inventory classification:
- Product definition docs: PRD, architecture, phase plan, ADR, IPs.
- Runtime contract docs: OpenAPI, AsyncAPI, proto, policy, Cedar schema.
- Operations docs: SLOs, dashboards, runbook, incident response, failure modes, capacity model.
- Compliance docs: compliance, DPIA, threat model, regulatory journey IPs.
- Developer enablement docs: FAQ, onboarding, tutorial, reference implementation, migration playbook.
- Machine-readable registry docs: manifest, catalog rows, capabilities, scorecard.
- IaC docs: Helm charts and Kustomize overlays only.
- Retired model docs: `capability-ladders/tier-matrix.md`.
- Benchmark docs: current benchmark uses AWS App Runner, Cloud Run, Fly.io, Azure Container Apps, not the required Heroku/Vercel/Fly.io set.

## 3. 9-dimension audit

### 3.1 Product purpose and counterpart fit

Finding: product purpose is coherent.
Evidence: PRD says Application is the single browser entry into Oyatie tenant surfaces and owns route resolution, session bootstrap, module loading, auth callback, frontend bundle serving, and tenant-admin shell control: `microservices/application/PRD.md:25`.
Evidence: PRD lists tenant value as one front door, time-to-interactive under 2 seconds, one SSO flow, module isolation, and tenant admin enablement: `microservices/application/PRD.md:44`.
Evidence: ADR-APP-001 states Application owns shell routing, tenant context, auth gateway, module loader, and frontend bundle serve: `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md:25`.
Evidence: FAQ separates `api-gateway` from `application`; gateway is protocol surface, Application is product-aware and tenant-aware dispatch: `microservices/application/faqs/platform-engineer-faq.md:8`.
Judgment: this is a tenant product-entry service, not a generic PaaS clone.
Judgment: Heroku, Vercel, and Fly.io are still useful counterparts because they define the public expectations for deploy, preview, routing, scaling, logs, rollback, and developer workflow.
Judgment: the correct union-coverage bar is "Application shell plus deployment/product-entry expectations," not "Application must become all of Heroku or all of Fly.io."
Gap: existing PRD and competitor matrix compare against Vercel, Next.js, Stripe, Linear, Notion, and Foundry, not Heroku/Vercel/Fly.io: `microservices/application/PRD.md:271`.
Gap: existing benchmark compares AWS App Runner, GCP Cloud Run, Fly.io, and Azure Container Apps, not Heroku/Vercel/Fly.io: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:1`.
Severity: P2 for counterpart-set drift because it affects this batch's parity deliverables but not the service boundary itself.

Counterpart interpretation:
- Heroku contributes app lifecycle, dyno scaling, review apps, add-ons, routing timeouts, build/release/run separation, and operational simplicity.
- Vercel contributes preview deployments, globally cached frontend delivery, framework-aware deployment, generated URLs, deployment protection, functions, edge regions, and instant rollback.
- Fly.io contributes global Anycast, Machines, region placement, autostop/autostart, private networking, soft/hard concurrency routing, and multi-region deployment.
- Oyatie Application contributes tenant context, Cedar-gated product routing, signed module manifests, audit-chain events, pack/residency overlays, and Leptos shell composition.

### 3.2 Ownership boundaries

Finding: service-owned bounded contexts are mostly coherent.
Evidence: manifest enumerates bounded contexts for auth-gateway, frontend-bundle-serve, module-loader, shell-routing, tenant-context, and tenant-admin-console: `microservices/application/manifest.json:6`.
Evidence: PRD names bounded contexts and crates from shell-routing through frontend-bundle-serve: `microservices/application/PRD.md:130`.
Evidence: ADR-APP-001 rejects API Gateway owning entry routing because gateway routes traffic while Application routes user product surfaces: `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md:128`.
Evidence: ADR-APP-001 keeps product-specific business flags in product services while surfacing entry flags through Application: `microservices/application/decisions/ADR-APP-001-entry-point-router-with-per-tenant-feature-flag-injection.md:79`.
Evidence: cross-handoffs list inbound callers including api-gateway, identity, tenancy, workflow-engine, developer-sdk, compliance, and governance: `microservices/application/cross-microservice-handoffs.md:20`.
Evidence: cross-handoffs list outbound callees including identity, tenancy, audit-chain, observability, cloud-secrets, compliance, and cloud-iac: `microservices/application/cross-microservice-handoffs.md:39`.
Judgment: the ownership map is good enough to guide implementation.
Gap: the map points to missing or misnamed policy locations.
Evidence: cross-handoffs says Cedar policies are under `microservices/application/policies/`, but inventory has `microservices/application/policy/`: `microservices/application/cross-microservice-handoffs.md:17`.
Evidence: cross-handoffs references `developer-scope.cedar`, but inventory has no such file under application policy: `microservices/application/cross-microservice-handoffs.md:161`.
Evidence: cross-handoffs references `secret-isolation.md`, but no such application policy artifact exists: `microservices/application/cross-microservice-handoffs.md:164`.
Severity: P1 because handoff readers cannot verify the exact permits named in the handoff ledger.

### 3.3 Contracts, events, and policy coherence

Finding: Application has meaningful contract coverage.
Evidence: OpenAPI declares tenant-facing and operator-facing REST surface and states host header plus JWT tenant_id must match: `microservices/application/contracts/openapi/application.yaml:6`.
Evidence: OpenAPI defines `RouteResolveResponse`, `ModuleManifest`, `BundleVersionPointer`, CDN purge request, and status summary schemas: `microservices/application/contracts/openapi/application.yaml:94`.
Evidence: AsyncAPI defines session, module-load, route-access-denied, CDN purge, and rollback channels: `microservices/application/contracts/asyncapi/application-events.yaml:30`.
Evidence: AsyncAPI declares audit-chain, tenancy, observability, and on-call consumers: `microservices/application/contracts/asyncapi/application-events.yaml:10`.
Evidence: proto defines the gRPC service shape for route resolution, tenant context, session, module manifest, and health: `microservices/application/contracts/proto/application.proto:21`.
Evidence: IP-016 scopes tenant admin console to OpenAPI contract, capability, Cedar policy, and audit events: `microservices/application/IP-016-tenant-admin-console-control-surface.md:16`.
Judgment: contract breadth is ahead of source-code reality.
Gap: cross-handoffs cite AsyncAPI `components/schemas` paths while the AsyncAPI file defines payloads under `components/messages`.
Evidence: handoff ledger references `application-events.yaml#/components/schemas/SessionStartedPayload`: `microservices/application/cross-microservice-handoffs.md:75`.
Evidence: AsyncAPI defines `components.messages.SessionStarted`, not `components.schemas.SessionStartedPayload`: `microservices/application/contracts/asyncapi/application-events.yaml:110`.
Gap: route resolution evidence in cross-handoffs references `ApplicationRouteResolved`, but AsyncAPI does not define a route-resolved channel; it defines route-access-denied only: `microservices/application/cross-microservice-handoffs.md:82` and `microservices/application/contracts/asyncapi/application-events.yaml:59`.
Gap: OpenAPI description says all endpoints are governed by `policy/*.cedar`, but some public anonymous routes are declared with empty security; this can be valid only if public-read policy is cited and tested: `microservices/application/contracts/openapi/application.yaml:17`.
Severity: P1 for schema path and event name mismatches because integration consumers will fail contract verification.

### 3.4 Canonical-direction alignment

#### 3.4.A Multi-context deployability

Canonical requirement: six contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`: `specs/master-plan-sequencing.json:704`.
Canonical requirement: every audit must evaluate the six context directories, with N/A only when justified: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854`.
Evidence: current application IaC files are Helm and Kustomize only.
Evidence: current application IaC inventory contains `iac/helm/*` and `iac/kustomize/*`, but no context directories.
Evidence: IP-015 only says "Author Helm + Kustomize wiring for the application cluster": `microservices/application/IP-015-application-openslo-and-hg-app.md:26`.
Gap: no `iac/oyatie-public-cloud/`.
Gap: no `iac/guest-on-aws/`.
Gap: no `iac/oci-guest/`.
Gap: no `iac/on-prem/`.
Gap: no `iac/colo/`.
Gap: no `iac/oyatie-iaas/`.
Gap: no manifest context matrix listing supported, unsupported, or N/A contexts.
Severity: P1 because deployability across all six contexts is not proven.

#### 3.4.B OpenTofu IaC

Canonical requirement: OpenTofu is the IaC engine: `specs/master-plan-sequencing.json:747`.
Canonical requirement: per-context directories must include `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and README/admission evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2296`.
Canonical prohibition: Terraform binary, Terraform Cloud, Pulumi, CloudFormation, ARM/Bicep, local-exec, remote-exec, null_resource, SSH bootstrap, and manual console claims are forbidden: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2464`.
Evidence: application only contains Helm and Kustomize IaC files.
Evidence: no `.tf` files exist under `microservices/application/iac/`.
Evidence: OpenTofu appears in compliance prose as part of an inventory list, not as service-owned context modules: `microservices/application/compliance.md:857`.
Evidence: no Terraform/Pulumi/CloudFormation file extension exists under the service path.
Gap: no OpenTofu version pin or module signing in service-owned IaC.
Gap: no per-context `tofu plan` or `tofu apply` evidence.
Gap: no cloud-iac handoff artifact that states Application is deliberately delegating context modules.
Severity: P1 for missing canonical IaC substrate.

#### 3.4.C Tenant-class adoption gaps

Current batch tenant classes: `demo_trial`, `paid`, and `revenue_share`.
Current batch semantics: `demo_trial` is free and capped on the OCI Always Free profile.
Current batch semantics: `paid` is per-seat plus usage-based billing with contractual SLO and optional compliance/BYOK.
Current batch semantics: `revenue_share` is gross-revenue percentage with at-cost or zero-margin substrate.
Doctrine: quality remains uniform across classes.
Search result: `tenant_class` does not appear under `microservices/application/`.
Search result: `demo_trial` does not appear under `microservices/application/`.
Search result: `revenue_share` does not appear under `microservices/application/`.
Search result: "paid" appears only as ordinary entitlement/payment prose in handoffs and old capability docs.
Evidence: cross-handoffs mention delayed paid entitlement events, not tenant_class semantics: `microservices/application/cross-microservice-handoffs.md:122`.
Evidence: OpenAPI schemas include tenant_id and pack, but no tenant_class field or billing-class projection: `microservices/application/contracts/openapi/application.yaml:82`.
Evidence: AsyncAPI messages include `tenant_id` and `pack`, but no tenant_class: `microservices/application/contracts/asyncapi/application-events.yaml:117`.
Evidence: manifest has `tier_classification` and `tenant_classs`, but no tenant_class field: `microservices/application/manifest.json:386`.
Gap: Application does not encode whether tenant_class is consumed, projected, audited, or deliberately owned elsewhere.
Required repair: add tenant_class as a governed context input sourced from tenancy/cloud-billing, not as a client-provided request parameter.
Required repair: specify which Application behaviors differ by tenant_class: usage caps, best-effort versus contractual SLO label, compliance pack gating, BYOK gating, and revenue-share cost-attribution.
Required repair: ensure feature quality and route/module capabilities do not degrade by tenant_class.
Severity: P1 because Application is the tenant entry shell and must not continue to rely on retired feature tiers.

#### 3.4.D OS support matrix

Canonical requirement: primary OS coverage includes Talos, Flatcar, Ubuntu, Debian, RHEL, Rocky, AlmaLinux, Oracle Linux, Amazon Linux, SLES, Photon, Windows Server, and VMware ESXi, with macOS and iOS for dev/client surfaces and out-of-scope markers for AIX, HP-UX, Solaris, z/OS, IBM i, FreeBSD, and OpenBSD: `specs/master-plan-sequencing.json:777`.
Canonical requirement: each microservice must expose a `supported-oses.json`-class manifest or equivalent machine-readable OS posture: `docs/standards/brief-template.md:967`.
Evidence: no `supported-oses.json` exists in application inventory.
Evidence: onboarding mentions local Kata on Linux or OrbStack VM on macOS, but not the canonical support matrix: `microservices/application/onboarding/platform-engineer-first-week.md:22`.
Gap: no OS-by-context support table.
Gap: no Windows Server packaging posture for backend deployment.
Gap: no macOS/iOS client/developer distinction beyond local onboarding.
Gap: no explicit out-of-scope OS claims.
Severity: P1 because the service cannot be audited against the primary OS set.

#### 3.4.E Rust-strict language policy

Canonical requirement: backend source is Rust only and web frontend is Leptos/WASM-SSR with selective island hydration; Swift, Kotlin, and WinUI3 are frontend/client allowlist surfaces: `specs/master-plan-sequencing.json:817`.
Scan result: no forbidden implementation files with extensions `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.fsx` exist under application.
Scan result: no `package.json`, `pyproject.toml`, `Gemfile`, `go.mod`, `pom.xml`, or `build.gradle` exists under application.
Positive evidence: IP-014 names Leptos WASM frontend crate and Rust composition-root binaries: `microservices/application/IP-014-leptos-frontend-and-composition.md:19`.
Positive evidence: reference implementation is Rust and uses `oya-application-sdk`: `microservices/application/reference-implementations/dispatch-intent-rust-sdk.md:1`.
Gap: source tree does not exist, so the policy is asserted by docs rather than implemented artifacts.
Gap: migration playbook includes a Cloudflare Worker snippet in a JavaScript code fence: `microservices/application/migration-playbooks/from-aws-app-runner.md:82`.
Judgment: no forbidden source files are present, but docs should replace the worker snippet with a Rust-compatible edge/proxy pattern or move it to an explicitly external migration note.
Severity: P2 for migration-doc language drift; P1 for absent source/test proof.

#### 3.4.F OCI Always Free profile

Canonical requirement: OCI Always Free profile uses up to 4 OCPU and 24 GB memory, 200 GB block volume, 10 GB object storage, 10 GB archive storage, Autonomous Database constraints, 10 Mbps load balancer, and 10 TB egress: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514`.
Canonical requirement: per-microservice module path is `iac/oci-guest/always-free/`: `specs/master-plan-sequencing.json:857`.
Evidence: no `microservices/application/iac/oci-guest/always-free/` exists.
Evidence: existing capability ladder file uses an AWS-centric entry-level shape rather than OCI Always Free: `microservices/application/capability-ladders/tier-matrix.md:20`.
Gap: demo_trial infrastructure cap is not expressed.
Gap: Application has no hard usage caps tied to OCI Always Free profile.
Gap: capacity model has service performance budgets but not OCI Always Free saturation envelope: `microservices/application/capacity-model.md:195`.
Severity: P1 because demo/trial infrastructure cannot be audited or bounded.

#### 3.4.G Retired tier-model residues outside explicit tier names

Evidence: manifest capability entries use `"tier": "T2"` and `"tier": "T1"`: `microservices/application/manifest.json:116`.
Evidence: manifest has `tenant_classs`: `microservices/application/manifest.json:386`.
Evidence: manifest has `tier_classification`: `microservices/application/manifest.json:413`.
Evidence: manifest has `criticality_tier`: `microservices/application/manifest.json:450`.
Evidence: PRD frontmatter includes `tier: B2B`: `microservices/application/PRD.md:8`.
Evidence: cost budget says "XS tier" for launch costs: `microservices/application/cost-budget.md:43`.
Evidence: cost budget has a per-tenant unit economics table with a `Tier` column: `microservices/application/cost-budget.md:70`.
Judgment: these are not retired four-label ladder hits, but they are still Wave 15J follow-up candidates because they preserve tier semantics.
Severity: P2.

#### 3.4.T Tier retirement candidates

All entries below are Wave 15J retirement candidates.
Default severity: P2 unless the same content also causes a P1 implementation or claims failure.
Search expression: `tenant_class demo_trial|tenant_class paid|tenant_class paid|compliance_pack-bound paid` under `microservices/application`.
Total explicit retired tier-name hits found: 27.

Tier retirement candidate 01:
- File: `microservices/application/reference-implementations/dispatch-intent-rust-sdk.md:85`.
- Text shape: `TierExpectation::AtLeasttenant_class demo_trial`.
- Impact: reference SDK teaches retired tenant access model.
- Severity: P2.

Tier retirement candidate 02:
- File: `microservices/application/tutorials/dispatch-intent-and-trace-through-cells.md:8`.
- Text shape: `tenant_class=demo_trial`.
- Impact: tutorial creates demo tenant with retired access class.
- Severity: P2.

Tier retirement candidate 03:
- File: `microservices/application/tutorials/dispatch-intent-and-trace-through-cells.md:18`.
- Text shape: `tier=tenant_class demo_trial`.
- Impact: tutorial cell topology output preserves retired class.
- Severity: P2.

Tier retirement candidate 04:
- File: `microservices/application/tutorials/dispatch-intent-and-trace-through-cells.md:19`.
- Text shape: `tier=tenant_class demo_trial`.
- Impact: tutorial repeats retired class in second cell.
- Severity: P2.

Tier retirement candidate 05:
- File: `microservices/application/tutorials/dispatch-intent-and-trace-through-cells.md:31`.
- Text shape: `tier: tenant_class demo_trial`.
- Impact: tenant show output preserves retired class.
- Severity: P2.

Tier retirement candidate 06:
- File: `microservices/application/migration-playbooks/from-aws-app-runner.md:14`.
- Text shape: SMB tenant_class paid tenant, tenant_class paid tenant, compliance_pack-bound paid with regulated packs.
- Impact: migration plan budgets are stratified by retired feature tiers instead of tenant_class and deployment context.
- Severity: P2.

Tier retirement candidate 07:
- File: `microservices/application/migration-playbooks/from-aws-app-runner.md:40`.
- Text shape: `--tier tenant_class paid`.
- Impact: tenant provisioning command uses retired tier flag.
- Severity: P2.

Tier retirement candidate 08:
- File: `microservices/application/migration-playbooks/from-aws-app-runner.md:45`.
- Text shape: `tenant_class paid tenants add --tier tenant_class paid`.
- Impact: migration playbook instructs retired tier migration.
- Severity: P2.

Tier retirement candidate 09:
- File: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:12`.
- Text shape: `application` row uses retired entry-level name.
- Impact: benchmark headline uses retired model and must be replaced by single target plus context/tenant_class overlays.
- Severity: P1 because it also invalidates benchmark methodology.

Tier retirement candidate 10:
- File: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:24`.
- Text shape: warm floor tied to retired entry-level class.
- Impact: performance capacity assumption is tier-segmented.
- Severity: P2.

Tier retirement candidate 11:
- File: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:49`.
- Text shape: TCO row uses retired mid-market class.
- Impact: cost comparison is tier-segmented.
- Severity: P2.

Tier retirement candidate 12:
- File: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:55`.
- Text shape: cost advantage references retired class.
- Impact: benchmark narrative perpetuates retired model.
- Severity: P2.

Tier retirement candidate 13:
- File: `microservices/application/faqs/platform-engineer-faq.md:51`.
- Text shape: FAQ asks about `Tier::tenant_class demo_trial`.
- Impact: engineer FAQ teaches retired enum.
- Severity: P2.

Tier retirement candidate 14:
- File: `microservices/application/faqs/platform-engineer-faq.md:53`.
- Text shape: answer says retired entry class is a production tenant class.
- Impact: directly contradicts no-capability-ladder doctrine.
- Severity: P2.

Tier retirement candidate 15:
- File: `microservices/application/faqs/platform-engineer-faq.md:54`.
- Text shape: says dev-cell cannot promote to retired class.
- Impact: preserves old promotion semantics.
- Severity: P2.

Tier retirement candidate 16:
- File: `microservices/application/faqs/platform-engineer-faq.md:74`.
- Text shape: emergency dispatch is restricted to retired top class.
- Impact: feature gating by retired class conflicts with uniform quality bar.
- Severity: P2.

Tier retirement candidate 17:
- File: `microservices/application/faqs/platform-engineer-faq.md:158`.
- Text shape: code enum `Tier::{retired four-label ladder}`.
- Impact: identifies likely code contract to retire when source lands.
- Severity: P2.

Tier retirement candidate 18:
- File: `microservices/application/capability-ladders/tier-matrix.md:13`.
- Text shape: canonical ladder names all four retired names.
- Impact: whole file is retired model.
- Severity: P2.

Tier retirement candidate 19:
- File: `microservices/application/capability-ladders/tier-matrix.md:15`.
- Text shape: retired entry-level section heading.
- Impact: whole section must be replaced by tenant_class/context overlay.
- Severity: P2.

Tier retirement candidate 20:
- File: `microservices/application/capability-ladders/tier-matrix.md:28`.
- Text shape: Cedar permit name includes retired entry-level class.
- Impact: policy examples must be rewritten around tenant_class and capability permits.
- Severity: P2.

Tier retirement candidate 21:
- File: `microservices/application/capability-ladders/tier-matrix.md:32`.
- Text shape: retired second-class section heading.
- Impact: retired model.
- Severity: P2.

Tier retirement candidate 22:
- File: `microservices/application/capability-ladders/tier-matrix.md:45`.
- Text shape: Cedar permit name includes retired second-class name.
- Impact: policy examples must be rewritten.
- Severity: P2.

Tier retirement candidate 23:
- File: `microservices/application/capability-ladders/tier-matrix.md:49`.
- Text shape: retired third-class section heading.
- Impact: retired model.
- Severity: P2.

Tier retirement candidate 24:
- File: `microservices/application/capability-ladders/tier-matrix.md:62`.
- Text shape: retired third-class adds admin override action.
- Impact: feature quality is stratified by retired class.
- Severity: P2.

Tier retirement candidate 25:
- File: `microservices/application/capability-ladders/tier-matrix.md:66`.
- Text shape: retired fourth-class section heading.
- Impact: retired model.
- Severity: P2.

Tier retirement candidate 26:
- File: `microservices/application/capability-ladders/tier-matrix.md:79`.
- Text shape: retired third-class permits plus emergency actions.
- Impact: feature gates conflict with uniform quality bar.
- Severity: P2.

Tier retirement candidate 27:
- File: `microservices/application/capability-ladders/tier-matrix.md:88`.
- Text shape: retired top classes default to BYOK-required.
- Impact: BYOK semantics must move to tenant_class/compliance-pack contract.
- Severity: P2.

Additional explicit retired-name hits inside the same retired file:
- `microservices/application/capability-ladders/tier-matrix.md:93`.
- `microservices/application/capability-ladders/tier-matrix.md:94`.
- `microservices/application/capability-ladders/tier-matrix.md:95`.
These are included in the same retirement cluster for the capability-ladder matrix and should be scrubbed with the file-level retirement.

### 3.5 Operational readiness, SLOs, and incident response

Finding: SLO coverage is concrete for shell responsiveness, route resolution, sign-in, module load, and audit seal.
Evidence: TTI OpenSLO requires p99 warm-cache path under 2 seconds: `microservices/application/slos/tti.openslo.yaml:10`.
Evidence: route resolve OpenSLO requires p99 under 100 ms: `microservices/application/slos/route-resolve.openslo.yaml:10`.
Evidence: module load success rate is above 99.9 percent excluding Cedar deny: `microservices/application/slos/module-load.openslo.yaml:10`.
Evidence: IP-015 registers SLO authoring and HG-APP gate registration: `microservices/application/IP-015-application-openslo-and-hg-app.md:19`.
Evidence: incident-response defines severity classes and roles: `microservices/application/incident-response.md:32`.
Evidence: failure-modes contains 16 named failure modes and mitigations: `microservices/application/failure-modes.md:34`.
Evidence: capacity-model provides TTI, auth, module-loader, storage, CDN, and capacity-envelope budgets: `microservices/application/capacity-model.md:44`.
Judgment: operations documentation is comparatively strong.
Gap: SLOs are labeled by `pack` and `tenant_id`, not by deployment_context or tenant_class; that blocks the new context/class overlay model.
Gap: no evidence that OpenSLO metrics exist in source because no source exists.
Gap: no evidence that dashboards are wired to emitted metrics because there is no implementation.
Gap: benchmark doc claims continuous production perf rig evidence but the cited file is not present under service path: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:18`.
Severity: P1 for evidence/implementation gap; P2 for label taxonomy migration.

### 3.6 Security, privacy, and compliance

Finding: compliance and security documentation is broad.
Evidence: compliance docs cover SOC 2, ISO 27001, GDPR, KR PIPA, HIPAA, ASVS, and continuous evidence: `microservices/application/compliance.md:32`.
Evidence: DPIA identifies processing triggers and data classes: `microservices/application/dpia.md:42`.
Evidence: threat model exists and is part of inventory: `microservices/application/threat-model.md`.
Evidence: OpenAPI declares OIDC plus session cookie schemes: `microservices/application/contracts/openapi/application.yaml:34`.
Evidence: AsyncAPI routes audit-relevant events to audit-chain: `microservices/application/contracts/asyncapi/application-events.yaml:10`.
Evidence: policy directory has Cedar policy files and a schema.
Judgment: the service has the right compliance vocabulary for a tenant entry shell.
Gap: tenant_class must control compliance pack eligibility for demo_trial versus paid/revenue_share without lowering feature quality.
Gap: BYOK semantics are still expressed through retired class docs: `microservices/application/capability-ladders/tier-matrix.md:88`.
Gap: emergency dispatch is documented as retired-class-only instead of incident-governed and uniformly available when policy permits: `microservices/application/faqs/platform-engineer-faq.md:74`.
Gap: cross-handoffs cite missing policy artifacts, which weakens compliance traceability: `microservices/application/cross-microservice-handoffs.md:161`.
Severity: P1 for policy traceability; P2 for tenant_class/compliance migration.

### 3.7 Implementation and test readiness

Finding: implementation plans are detailed but not landed as source.
Evidence: IP-001 creates a Rust kernel crate at `microservices/application/src/crates/oya-application-shell-routing-kernel/`: `microservices/application/IP-001-shell-routing-kernel.md:26`.
Evidence: IP-001 acceptance gates require cargo check, build, clippy, nextest, deny, doc, and policy gates: `microservices/application/IP-001-shell-routing-kernel.md:111`.
Evidence: IP-014 creates Leptos frontend and composition-root binaries: `microservices/application/IP-014-leptos-frontend-and-composition.md:19`.
Evidence: IP-014 acceptance gates include WASM build and Rust release builds: `microservices/application/IP-014-leptos-frontend-and-composition.md:72`.
Evidence: manifest marks IP-001 through IP-016 with `acceptance_status: ga`: `microservices/application/manifest.json:174`.
Evidence: no `microservices/application/src/` exists.
Evidence: no `microservices/application/tests/` exists.
Judgment: the implementation plan/source reality mismatch is a gate-blocking coherence issue.
Gap: test names are specified in IPs, but test files are absent.
Gap: manifest says GA while plans say pending or accepted; for example IP-001 has `status: pending`: `microservices/application/IP-001-shell-routing-kernel.md:7`.
Gap: IP-016 has a different frontmatter schema (`doc_kind`, `id`, `owner_team`) from the earlier IP template: `microservices/application/IP-016-tenant-admin-console-control-surface.md:1`.
Severity: P1 because readers can wrongly infer implementation completion.

### 3.8 Documentation substance and anti-pattern screen

Finding: there is a lot of substantive documentation.
Evidence: architecture is 754 lines and covers principal models, data inventory, policy, secrets, edge cases, and credential isolation.
Evidence: compliance is 1,038 lines and contains expanded anchors.
Evidence: journey IPs provide extensive regulatory slices.
Positive: docs are not merely file stubs.
Concern: some docs still carry generated or anchor-sweep language.
Evidence: architecture starts with "Anchor-sweep generated" and says to expand all stub sections during content-pass review: `microservices/application/ARCHITECTURE.md:1`.
Concern: regulatory journey IPs include many repeated "slice detail" headings and should be reviewed for substance quality.
Evidence: `IP-journey-j16-a11y-substrate-signup-shell.md` has repeated step headings through at least Step 72: `microservices/application/IP-journey-j16-a11y-substrate-signup-shell.md:94`.
Concern: docs cite benchmark evidence files that are not in inventory.
Evidence: benchmark says numbers come from `bench-evidence-2026-05-08T11:42:01Z.json`: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-fly-vs-azure-container-apps.md:69`.
Severity: P2 for generated-doc cleanup; P1 where evidence files are cited as claim support but missing.

### 3.9 Competitive, performance, and claims posture

Finding: Application has strong differentiated claims.
Evidence: signed module manifest and per-product key isolation are named differentiators: `microservices/application/competitor-parity-matrix.md:116`.
Evidence: Cedar-expressed routing and pack-residency forbid-by-default are named differentiators: `microservices/application/competitor-parity-matrix.md:121`.
Evidence: benchmark claims warm no-cold-start and p50/p95/p99 numbers: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-fly-vs-azure-container-apps.md:12`.
Concern: current benchmark is not source-backed enough for a fresh audit.
Concern: current benchmark uses retired class rows and non-mandated counterpart set.
Concern: current benchmark claims vendor cold-start medians from dated blog/status sources but does not cite URLs in the service doc: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-fly-vs-azure-container-apps.md:18`.
Concern: current benchmark states Application wins with 0 ms cold start due to warm pool sized by retired class: `microservices/application/benchmarks/application-vs-aws-app-runner-vs-fly-vs-azure-container-apps.md:24`.
Judgment: claims must be rebuilt around Heroku/Vercel/Fly.io with single industry-leader target and deployment/tenant_class overlays.
Severity: P1 for benchmark evidence integrity.

## 4. Findings table

| ID | Severity | Finding | Evidence | Required correction |
|---|---:|---|---|---|
| APP-COH-001 | P1 | Missing six canonical OpenTofu deployment-context directories blocks all-context deployability. | Canonical dirs required by ADR-0328:2277; current IaC inventory only Helm/Kustomize. | Add or explicitly delegate `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, `iac/oyatie-iaas/` with OpenTofu plans. |
| APP-COH-002 | P1 | No OCI Always Free profile module exists for demo/trial constrained infrastructure. | Master plan requires `iac/oci-guest/always-free/`: `specs/master-plan-sequencing.json:857`; service has no such path. | Add OCI Always Free profile module and capacity caps, or cite cloud-iac-owned module with service inputs/outputs. |
| APP-COH-003 | P1 | No supported OS manifest exists. | OS matrix canonical: `specs/master-plan-sequencing.json:777`; no `supported-oses.json` in inventory. | Add machine-readable support matrix for primary OS set, dev/client OSes, out-of-scope OSes, and arch matrix. |
| APP-COH-004 | P1 | Tenant_class semantics are absent. | No `tenant_class`, `demo_trial`, or `revenue_share` hits under service; manifest still has `tenant_classs`: `microservices/application/manifest.json:386`. | Add tenant_class consumption/projection semantics and remove retired tier-driven access. |
| APP-COH-005 | P1 | Manifest/IPs imply GA source while no source or tests exist. | Manifest says IPs are GA at `microservices/application/manifest.json:174`; IP-001 status is pending at `microservices/application/IP-001-shell-routing-kernel.md:7`; no `src/` or `tests/`. | Correct manifest status or land source/tests and evidence lanes. |
| APP-COH-006 | P1 | Cross-service handoff ledger references missing policy paths. | `policies/` path at `cross-microservice-handoffs.md:17`; `developer-scope.cedar` at line 161; `secret-isolation.md` at line 164. | Point to existing `policy/` files or add the missing policy artifacts with schema validation. |
| APP-COH-007 | P1 | Handoff schema refs do not match AsyncAPI structure. | Handoff references `components/schemas/SessionStartedPayload` at line 75; AsyncAPI defines `components.messages` at line 110. | Rewrite handoff refs to exact AsyncAPI message paths or add schemas. |
| APP-COH-008 | P1 | Route-resolved event is cited but not present in AsyncAPI. | Handoff emits `ApplicationRouteResolved` at line 82; AsyncAPI defines route-access-denied channel at line 59 and no route-resolved channel. | Add `application.route.resolved` event or remove the claim. |
| APP-COH-009 | P1 | Benchmark claims are not acceptable as current evidence. | Benchmark uses retired class rows at line 12 and missing evidence file at line 69. | Rebuild benchmark using Heroku/Vercel/Fly.io, public citations, and service-owned evidence paths. |
| APP-COH-010 | P2 | Existing parity set drifts from batch counterpart set. | PRD benchmark starts at `PRD.md:271`; competitor matrix uses Vercel/Next.js/Stripe/Linear/Notion/Foundry at `competitor-parity-matrix.md:27`. | Keep those as historical comparisons or add Heroku/Vercel/Fly.io union matrix. |
| APP-COH-011 | P2 | Retired tier names appear in 27 explicit hits. | Full catalog in §3.4.T. | Retire `capability-ladders/`, update examples, migration, FAQ, benchmark, and SDK terminology. |
| APP-COH-012 | P2 | Manifest preserves non-name tier fields. | `tier`, `tenant_classs`, `tier_classification`, `criticality_tier`: `manifest.json:116`, `386`, `413`, `450`. | Replace with capability class, risk class, tenant_class policy, or criticality without feature-ladder semantics. |
| APP-COH-013 | P2 | Cost model uses launch cost by tenant_class. | `cost-budget.md:43` and `cost-budget.md:70`. | Reframe cost by deployment context, tenant_class, and usage envelope. |
| APP-COH-014 | P2 | Migration playbook uses retired tenant provisioning flags. | `from-aws-app-runner.md:40` and `from-aws-app-runner.md:45`. | Replace with tenant_class, billing model, compliance packs, and deployment context. |
| APP-COH-015 | P2 | Migration playbook embeds a JavaScript Worker snippet. | `from-aws-app-runner.md:82`. | Replace with Rust-compatible proxy pattern or explicitly mark as external temporary migration artifact. |
| APP-COH-016 | P2 | Architecture doc retains generated anchor-sweep marker. | `ARCHITECTURE.md:1`. | Remove generator residue after verifying each section has real service substance. |
| APP-COH-017 | P2 | Journey IPs need substance review for repeated step scaffolding. | `IP-journey-j16-a11y-substrate-signup-shell.md:94`. | Collapse repetitive steps into named implementation tasks with clear evidence and tests. |
| APP-COH-018 | P2 | SLO labels are not yet context/class aware. | TTI query labels pack and tenant_id at `slos/tti.openslo.yaml:19`; no deployment_context or tenant_class. | Add deployment_context and tenant_class labels if observability substrate supports them. |
| APP-COH-019 | P2 | BYOK semantics are documented through retired class defaults. | `capability-ladders/tier-matrix.md:88`. | Reframe BYOK by tenant_class, compliance pack, and contract obligation. |
| APP-COH-020 | P2 | Emergency dispatch is documented as retired-class-only. | `faqs/platform-engineer-faq.md:74`. | Reframe as break-glass policy governed by role, incident state, and audit evidence. |
| APP-COH-021 | P3 | No README exists. | Inventory has PRD and architecture but no README. | Add a minimal pointer README if repository guidance expects one. |
| APP-COH-022 | P3 | IP-016 frontmatter schema differs from IP-001..IP-015. | `IP-016-tenant-admin-console-control-surface.md:1`; IP-001 template at `IP-001-shell-routing-kernel.md:1`. | Normalize frontmatter or document exception. |
| APP-COH-023 | P3 | Chat history contains a tenant-class correction conflict against the current prompt. | Chat line 16232 says two classes; current task mandates three classes. | Treat current prompt as controlling for this batch and resolve doctrine in Wave 15J control artifact. |

Finding counts:
- P0: 0.
- P1: 9.
- P2: 11.
- P3: 3.

## 5. Open questions

1. Should Application own the six OpenTofu context modules directly, or should `cloud-iac` own reusable modules with Application-owned variable contracts and per-context admission evidence?
2. What is the exact tenant_class source of truth for this batch: current prompt's three-class model, or the later chat-history two-class correction captured at line 16232?
3. Should Application event payloads include tenant_class directly, or should observability join tenant_class from tenancy/cloud-billing at query time?
4. Should demo_trial tenant_class be allowed to access all Application shell capabilities with usage/time caps, or should compliance/BYOK/admin surfaces be hidden by policy while keeping code quality uniform?
5. What is the canonical replacement name for `tenant_classs` in manifest fields: `capability_risk_class`, `service_criticality`, or a different machine-readable field?
6. Are the manifest `acceptance_status: ga` values intended as plan status, implementation status, or target status?
7. Where should source evidence for `oya-application-*` crates live if not under `microservices/application/src/`?
8. Should `policy/secret-isolation.md` be application-owned, or should cloud-secrets own that policy and Application only cite it?
9. Should `developer-scope.cedar` move to `developer-sdk`, with Application only declaring the `read_sandbox_manifest` resource/action?
10. Should `application.route.resolved` be added to AsyncAPI, or should the route-resolved audit event be emitted only to audit-chain outside the workflow bus?
11. Should `pack` labels in OpenSLO queries be supplemented or replaced by deployment_context and tenant_class labels?
12. What is the target public claim for Application versus Heroku: app lifecycle parity, review app parity, or only tenant shell parity?
13. What is the target public claim for Application versus Vercel: preview deployment parity, edge delivery parity, or only signed shell-module parity?
14. What is the target public claim for Application versus Fly.io: anycast/runtime placement parity, Machines-style tenant cell placement, or only routing/location parity?
15. Should migration playbooks continue to mention AWS App Runner, or should this batch add Heroku/Vercel/Fly.io migrations as the primary counterpart playbooks?
16. Should the JavaScript Worker migration snippet be removed immediately under Rust-strict policy or preserved as a third-party migration bridge with a waiver?
17. What evidence location should replace the missing benchmark file `bench-evidence-2026-05-08T11:42:01Z.json`?
18. Should the retired `capability-ladders/` directory be deleted in Wave 15J or converted into tenant_class plus deployment-context overlays?
19. Should `criticality_tier` remain if it means operational criticality rather than feature tier, or should it be renamed to avoid ambiguity?
20. Should Application expose tenant_class in admin console read models so operators can inspect caps/SLO posture without making it a client input?

<!-- ORCHESTRATOR REPORT
  microservice: application
  deliverables_landed:
    - microservices/application/coherence-audit-2026-05-20.md: 718 lines
    - microservices/application/feature-parity-matrix-2026-05-20.md: 550 lines
    - microservices/application/performance-benchmark-numbers-2026-05-20.md: 653 lines
  inventory_files_seen: 135
  inventory_lines_read: 16633
  chat_history_matches_processed: 212 match lines scanned; high-signal lines cited: 16171, 16174, 16232
  findings_p0: 0
  findings_p1: 9
  findings_p2: 11
  findings_p3: 3
  tier_retirement_candidates_found: 27 explicit retired-name hits; primary cite list: reference-implementations/dispatch-intent-rust-sdk.md:85; tutorials/dispatch-intent-and-trace-through-cells.md:8,18,19,31; migration-playbooks/from-aws-app-runner.md:14,40,45; benchmarks/application-vs-aws-app-runner-vs-cloud-run-vs-fly-vs-azure-container-apps.md:12,24,49,55; faqs/platform-engineer-faq.md:51,53,54,74,158; capability-ladders/tier-matrix.md:13,15,28,32,45,49,62,66,79,88
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share surface under microservices/application, while manifest still carries tenant_classs and tier_classification
  top_3_counterparts_confirmed: Heroku / Vercel / Fly.io
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1921
-->
