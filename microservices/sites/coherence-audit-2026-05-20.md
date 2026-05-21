# Sites Ownership-Coherence Audit - 2026-05-20

Audit owner: single-agent ownership audit for `microservices/sites`.

Target microservice: `sites`.

Product surface: published-web plus intranet site builder, CMS, domain, SEO, search, CDN delivery, accessibility, and bounded AI page-build.

Counterpart union bar: Webflow, Squarespace, Wix.

Audit date authored: 2026-05-21.

Canonical sources inspected:

- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3829-4121 define the nine audit dimensions, the six contexts, OpenTofu requirements, OS support, Rust-strict policy, and severity semantics.
- `specs/master-plan-sequencing.json` lines 704-746 define the six deployment contexts and their `iac_target` paths.
- `specs/master-plan-sequencing.json` lines 747-775 define OpenTofu as the IaC substrate and forbid Terraform, Pulumi, CloudFormation-primary, ARM-primary, local execution, SSH provisioners, hand-edited state, and unsigned modules.
- `specs/master-plan-sequencing.json` lines 777-815 define the required OS matrix and the per-microservice manifest requirement.
- `specs/master-plan-sequencing.json` lines 817-855 define Rust-strict backend policy and the frontend allowlist.
- `specs/master-plan-sequencing.json` lines 857-865 define the OCI Always Free profile and the service-local module path `iac/oci-guest/always-free/`; line 865 is itself a stale retirement candidate in the root corpus, but this audit only writes findings for the `sites` path.
- `docs/standards/brief-template.md` lines 666-807 define the multi-context anchor and required audit evidence.
- `docs/standards/brief-template.md` lines 809-965 define the OpenTofu anchor and required audit evidence.
- `docs/standards/brief-template.md` lines 967-1123 define the OS support anchor and required audit evidence.
- `docs/standards/brief-template.md` lines 1125-1165 define the Rust-strict anchor.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md` lines 10-45 retire demo_trial/paid/paid/paid compliance_pack tenant_class model and require OCI Always Free wording without tenant_class framing.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md` lines 23-35 define demo/trial and paid behavior; the user prompt for this batch supersedes it by requiring three tenant classes: `demo_trial`, `paid`, and `revenue_share`.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md` lines 10-14 require one agent to own one whole microservice and treat contradictions as product-drift defects.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md` lines 18-63 define the required surfaces and contradiction checks.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md` lines 10-20 require substantive, intern-buildable documents rather than padded scaffolds.

Chat-history evidence inspected:

- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl` line 78 includes the prior repository search result that named `microservices/sites/IP-001-iac-bootstrap.md`, `microservices/sites/IP-011-cdn-delivery-and-pipeline.md`, `microservices/sites/sdk-plan.md`, `microservices/sites/runbooks/asset-optimization-degraded.md`, `microservices/sites/decisions/ADR-SITES-0007-image-and-asset-pipeline.md`, and `microservices/sites/iac/helm/Chart.yaml` as relevant sites artifacts.
- `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl` line 17270 records the active task reminder that Batch 3.2 drops tenant-class-deltas and restructures performance benchmarking to a single industry-leader target plus deployment-context overlay.

Method:

- Read the complete file inventory under `microservices/sites`; 123 files were seen.
- Read or sampled the required PRD, architecture, implementation plans, ADRs, contracts, SLOs, tenant-class material, policies, runbooks, benchmarks, onboarding, tutorials, reference implementation, cost, capacity, compliance, DPIA, incident response, migration, and IaC surfaces.
- Searched for retired tier words and tenant-class vocabulary in `microservices/sites`.
- Searched for forbidden backend language source files in `microservices/sites`; no authored `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.fsx` files were found.
- Checked deployment/IaC directories; only Helm and Kustomize material exists under `microservices/sites/iac`.
- Compared the current artifacts to the user-required top-3 counterparts Webflow, Squarespace, and Wix.

Verdict summary:

- Dimension 1 internal coherence: FINDING.
- Dimension 2 outbound cross-references: FINDING.
- Dimension 3 substance bar: FINDING.
- Dimension 4 canonical-direction alignment: FINDING.
- Dimension 5 industry-counterpart parity: FINDING.
- Dimension 6 multi-context deployment: FINDING.
- Dimension 7 OpenTofu IaC: FINDING.
- Dimension 8 OS support: FINDING.
- Dimension 9 Rust-strict language policy: PASS with documentation gap.

Severity counts in this report:

- P0: 0.
- P1: 8.
- P2: 8.
- P3: 0.

## Section 1 - Purpose

The purpose of this audit is to decide whether `sites` can be handed to implementation without producing the wrong product.

The audit uses the microservice-ownership doctrine, not a surface-only doc-count doctrine.

The unit of inspection is every file under `microservices/sites`, not only PRD or ADR files.

The bar is product coherence, canonical direction, and implementability from the local artifacts.

The PRD defines `sites` as a standalone published-web plus intranet microservice after Connect unbundling, citing ADR-0132 and ADR-0135 in `microservices/sites/PRD.md` lines 20-26.

The PRD product surface is broad: site authoring, URL-routed pages, block composition, themes, navigation, domain binding, SEO, CMS collections, Meilisearch search, forms integration, commerce stub, analytics, accessibility, preview, versioning, i18n, comments, CDN delivery, AI page build, and collaboration in `microservices/sites/PRD.md` lines 22-24.

The tenant outcomes include custom-domain TLS, CDN-grade latency, CMS collections, accessibility and SEO enforcement, AI page build, collaborative editing, and privacy-preserving analytics in `microservices/sites/PRD.md` lines 30-37.

The functional requirement table enumerates 28 requirements in `microservices/sites/PRD.md` lines 41-70.

The performance table gives concrete targets for page render, static assets, CMS queries, search, publish, ACME renew, image optimization, and AI page build in `microservices/sites/PRD.md` lines 76-85.

The availability target is 99.99 percent monthly for read path and 99.95 percent for editor write path in `microservices/sites/PRD.md` lines 107-109.

The PRD says there are eleven primary bounded contexts in `microservices/sites/PRD.md` lines 116-134.

The PRD says the implementation introduces 78 crates across those bounded contexts in `microservices/sites/PRD.md` lines 167-183.

The manifest disagrees with that product shape because it lists only seven bounded contexts in `microservices/sites/manifest.json` lines 6-65.

The manifest also lists only three layers, `adapter`, `app`, and `kernel`, in `microservices/sites/manifest.json` lines 66-70.

The OpenAPI contract is closer to the PRD than the manifest because it names the eleven PRD bounded-context tags in `microservices/sites/contracts/openapi/sites.yaml` lines 34-58.

The proto contract is also closer to the PRD because it defines site, page, domain, URL routing, SEO, CMS collection, search, and CDN services in `microservices/sites/contracts/proto/sites.proto` lines 175-229.

The strongest product direction is therefore PRD plus contracts, while the manifest is incomplete and stale.

This audit treats the PRD plus contracts as the product intent and flags the manifest as a coherence blocker.

The audit must also honor the 2026-05-20 tier retirement directive.

The new deliverables must not introduce tenant-class scaffolding.

Existing tenant_class references are recorded only as Wave 15J retirement candidates.

The performance model must be a single industry-leader target set with deployment-context and tenant-class overlays.

The tenant-class replacement required by this batch is `{demo_trial, paid, revenue_share}`.

The current sites artifacts do not model those three tenant classes.

The only local `tenant_class` references are CI synthetic tenant constructs in `microservices/sites/policy/ci-scope.cedar` lines 54-55 and 81.

The production authorization policy still uses `tenant_tier` in `microservices/sites/policy/tenant-scope.cedar` lines 142-153.

The audit therefore treats tenant-class adoption as a P1 implementation-direction gap, not a minor wording defect.

The audit also evaluates all six deployable contexts because no service-local artifact gives a hard N/A reason for any of them.

The six required contexts are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3854-3871 and `specs/master-plan-sequencing.json` lines 704-746.

`sites` is a tenant-facing published-web surface, so context availability affects the product promise directly.

The audit found no service-local OpenTofu modules for those contexts.

The audit found only Helm and Kustomize under `microservices/sites/iac`, matching `microservices/sites/IP-001-iac-bootstrap.md` lines 16-34 and the actual inventory of `microservices/sites/iac`.

The audit found no `supported-oses.json`.

The audit found no `src` or `tests` directories under `microservices/sites`.

The lack of source code is not a Rust violation by itself, but it means implementation readiness cannot be claimed from code evidence.

The stop condition for this document is an evidence-backed list of contradictions, canonical gaps, counterpart gaps, and tenant/tier remediation needs.

## Section 2 - Inventory

Inventory command result: 123 files under `microservices/sites`.

Inventory line count across existing files before these audit deliverables: 19,393 lines.

Top-level artifact present: `microservices/sites/ARCHITECTURE.md`, 880 lines.

Top-level artifact present: `microservices/sites/AUDIT-FINDINGS-2026-05-18.json`, 267 lines.

Top-level artifact present: `microservices/sites/IP-001-iac-bootstrap.md`, 99 lines.

Top-level artifact present: `microservices/sites/IP-002-site-bc-kernel.md`, 58 lines.

Top-level artifact present: `microservices/sites/IP-003-page-bc-kernel.md`, 49 lines.

Top-level artifact present: `microservices/sites/IP-004-block-bc-and-loro.md`, 49 lines.

Top-level artifact present: `microservices/sites/IP-005-theme-and-navigation.md`, 45 lines.

Top-level artifact present: `microservices/sites/IP-006-url-routing.md`, 86 lines.

Top-level artifact present: `microservices/sites/IP-007-domain-binding-acme.md`, 49 lines.

Top-level artifact present: `microservices/sites/IP-008-seo-and-sitemap.md`, 53 lines.

Top-level artifact present: `microservices/sites/IP-009-cms-collection.md`, 47 lines.

Top-level artifact present: `microservices/sites/IP-010-search-meilisearch.md`, 92 lines.

Top-level artifact present: `microservices/sites/IP-011-cdn-delivery-and-pipeline.md`, 53 lines.

Top-level artifact present: `microservices/sites/IP-012-policy-dpia-threat-model.md`, 90 lines.

Top-level artifact present: `microservices/sites/IP-013-contracts-and-capabilities.md`, 87 lines.

Top-level artifact present: `microservices/sites/IP-014-dashboards-runbooks-slos.md`, 83 lines.

Top-level artifact present: `microservices/sites/IP-015-hg-sites-maturity-claim.md`, 90 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j91-us-msb-mtl-overlay.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j92-br-lgpd-us-parent-dsar.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j93-in-dpdpa-rbi-overlay.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j94-sox404-public-company-controls.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j95-iso27001-soc2-annual-audit.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j96-ksa-uae-mena-onboarding.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j97-sg-pdpa-mas-tenant.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j98-au-privacy-apra-cps234.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j99-multi-pack-conflict-resolution.md`, 400 lines.

Journey implementation plan present: `microservices/sites/IP-journey-j100-pack-rollout-first-action.md`, 400 lines.

Foundation doc present: `microservices/sites/PHASE-01-SITES-FOUNDATION.md`, 94 lines.

PRD present: `microservices/sites/PRD.md`, 400 lines.

Replay doc present: `microservices/sites/backfill-replay.md`, 150 lines.

Benchmark doc present: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md`, 111 lines.

Capability file present: `microservices/sites/capabilities/T0-suggest.yaml`, 143 lines.

Capability file present: `microservices/sites/capabilities/T1-assist.yaml`, 178 lines.

Capability file present: `microservices/sites/capabilities/T2-auto.yaml`, 113 lines.

Retired-tier matrix present: `microservices/sites/tenant-class/tier-matrix.md`, 135 lines.

Capacity model present: `microservices/sites/capacity-model.md`, 185 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-block-adapter-loro.yaml`, 22 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-cloudflare-cdn-stub.yaml`, 21 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-libvips.yaml`, 21 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-pandoc.yaml`, 21 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-cdn-delivery-adapter-s3.yaml`, 20 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-cdn-delivery-app.yaml`, 18 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-cms-collection-adapter-postgres.yaml`, 20 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-domain-binding-adapter-acme.yaml`, 21 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-domain-binding-adapter-cert-manager.yaml`, 21 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-domain-binding-app.yaml`, 18 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-page-adapter-postgres.yaml`, 22 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-page-app.yaml`, 19 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-page-kernel.yaml`, 18 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-search-adapter-meilisearch.yaml`, 20 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-site-app.yaml`, 19 lines.

Catalog file present: `microservices/sites/catalog/oya-sites-site-kernel.yaml`, 18 lines.

Competitor matrix present: `microservices/sites/competitor-parity-matrix.md`, 91 lines.

Compliance doc present: `microservices/sites/compliance.md`, 1,192 lines.

AsyncAPI contract present: `microservices/sites/contracts/asyncapi/sites-events.yaml`, 380 lines.

OpenAPI contract present: `microservices/sites/contracts/openapi/sites.yaml`, 476 lines.

Proto contract present: `microservices/sites/contracts/proto/sites.proto`, 294 lines.

Cost budget present: `microservices/sites/cost-budget.md`, 97 lines.

Dashboard present: `microservices/sites/dashboards/editor-experience.json`, 92 lines.

Dashboard present: `microservices/sites/dashboards/publish-and-cdn.json`, 97 lines.

Dashboard present: `microservices/sites/dashboards/seo-and-traffic.json`, 88 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0001-crdt-library-selection.md`, 225 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0002-static-vs-dynamic-rendering-strategy.md`, 193 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0003-cdn-substrate-and-cache-strategy.md`, 216 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0004-acme-and-custom-domain-flow.md`, 218 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0005-cms-collection-data-model.md`, 220 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0006-ai-page-build-bounds.md`, 239 lines.

ADR present: `microservices/sites/decisions/ADR-SITES-0007-image-and-asset-pipeline.md`, 238 lines.

ADR index present: `microservices/sites/decisions/README.md`, 68 lines.

Deprecation notice present: `microservices/sites/deprecation-notice.md`, 160 lines.

DPIA present: `microservices/sites/dpia.md`, 220 lines.

Failure modes present: `microservices/sites/failure-modes.md`, 135 lines.

FAQ present: `microservices/sites/faqs/sites-engineer-faq.md`, 143 lines.

IaC Helm chart present: `microservices/sites/iac/helm/Chart.yaml`, 60 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/cronjob.yaml`, 98 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/deployment.yaml`, 64 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/hpa.yaml`, 35 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/networkpolicy.yaml`, 96 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/pdb.yaml`, 22 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/prometheusrule.yaml`, 129 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/service.yaml`, 22 lines.

IaC Helm template present: `microservices/sites/iac/helm/templates/servicemonitor.yaml`, 32 lines.

IaC Helm values present: `microservices/sites/iac/helm/values.yaml`, 228 lines.

IaC Kustomize base present: `microservices/sites/iac/kustomize/base/kustomization.yaml`, 23 lines.

IaC Kustomize base present: `microservices/sites/iac/kustomize/base/namespace.yaml`, 10 lines.

IaC Kustomize base present: `microservices/sites/iac/kustomize/base/serviceaccount.yaml`, 11 lines.

IaC Kustomize overlay present: `microservices/sites/iac/kustomize/overlays/pack-eu/kustomization.yaml`, 38 lines.

IaC Kustomize overlay present: `microservices/sites/iac/kustomize/overlays/pack-kr/kustomization.yaml`, 38 lines.

IaC Kustomize overlay present: `microservices/sites/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml`, 37 lines.

IaC Kustomize overlay present: `microservices/sites/iac/kustomize/overlays/pack-us/kustomization.yaml`, 30 lines.

Incident response present: `microservices/sites/incident-response.md`, 155 lines.

Manifest present: `microservices/sites/manifest.json`, 435 lines.

Connect migration doc present: `microservices/sites/migration-from-connect.md`, 489 lines.

Migration playbook present: `microservices/sites/migration-playbooks/from-webflow-wix-business-and-wordpress.md`, 202 lines.

Multi-region doc present: `microservices/sites/multi-region.md`, 121 lines.

Onboarding guide present: `microservices/sites/onboarding/sites-engineer-first-week.md`, 173 lines.

Cedar policy present: `microservices/sites/policy/auditor-scope.cedar`, 146 lines.

Cedar policy present: `microservices/sites/policy/ci-scope.cedar`, 130 lines.

Policy doc present: `microservices/sites/policy/data-residency.md`, 213 lines.

Policy doc present: `microservices/sites/policy/editor-isolation.md`, 189 lines.

Cedar policy present: `microservices/sites/policy/public-read.cedar`, 183 lines.

Cedar policy present: `microservices/sites/policy/tenant-scope.cedar`, 256 lines.

Reference implementation doc present: `microservices/sites/reference-implementations/cms-collection-and-page-rust-sdk.md`, 279 lines.

Runbook present: `microservices/sites/runbooks/acme-cert-renewal-failure.md`, 155 lines.

Runbook present: `microservices/sites/runbooks/ai-page-build-rollback.md`, 202 lines.

Runbook present: `microservices/sites/runbooks/asset-optimization-degraded.md`, 159 lines.

Runbook present: `microservices/sites/runbooks/cdn-cache-purge-cascade.md`, 175 lines.

Runbook present: `microservices/sites/runbooks/custom-domain-dns-drift.md`, 168 lines.

Runbook present: `microservices/sites/runbooks/page-export-corruption.md`, 179 lines.

Runbook present: `microservices/sites/runbooks/publish-pipeline-rollback.md`, 182 lines.

Scorecard override present: `microservices/sites/scorecards/overrides.json`, 55 lines.

SDK plan present: `microservices/sites/sdk-plan.md`, 152 lines.

SLO present: `microservices/sites/slos/accessibility-wcag-correctness.openslo.yaml`, 51 lines.

SLO present: `microservices/sites/slos/acme-renew-latency.openslo.yaml`, 42 lines.

SLO present: `microservices/sites/slos/cms-query-latency.openslo.yaml`, 40 lines.

SLO present: `microservices/sites/slos/image-optimize-latency.openslo.yaml`, 41 lines.

SLO present: `microservices/sites/slos/page-render-latency.openslo.yaml`, 42 lines.

SLO present: `microservices/sites/slos/publish-latency.openslo.yaml`, 41 lines.

SLO present: `microservices/sites/slos/seo-meta-correctness.openslo.yaml`, 53 lines.

SLO present: `microservices/sites/slos/site-search-latency.openslo.yaml`, 40 lines.

SLO present: `microservices/sites/slos/static-asset-latency.openslo.yaml`, 41 lines.

Threat model present: `microservices/sites/threat-model.md`, 237 lines.

Tutorial present: `microservices/sites/tutorials/launch-site-with-custom-domain-cms-and-accessibility.md`, 307 lines.

Inventory absence: no `microservices/sites/README.md` was found.

Inventory absence: no `microservices/sites/src/` path was found.

Inventory absence: no `microservices/sites/tests/` path was found.

Inventory absence: no `microservices/sites/supported-oses.json` was found.

Inventory absence: no `microservices/sites/iac/oyatie-public-cloud/` directory was found.

Inventory absence: no `microservices/sites/iac/guest-on-aws/` directory was found.

Inventory absence: no `microservices/sites/iac/oci-guest/` directory was found.

Inventory absence: no `microservices/sites/iac/oci-guest/always-free/` directory was found.

Inventory absence: no `microservices/sites/iac/on-prem/` directory was found.

Inventory absence: no `microservices/sites/iac/colo/` directory was found.

Inventory absence: no `microservices/sites/iac/oyatie-iaas/` directory was found.

Inventory absence: no `.tf` file was found under `microservices/sites/iac`.

Inventory absence: no authored forbidden-language source file was found under `microservices/sites`.

## Section 3 - Nine-Dimension Audit

### Section 3.1 - Dimension 1, Internal Coherence

Verdict: FINDING.

The PRD and contracts define a broad eleven-BC product surface, but the manifest only records seven bounded contexts.

Evidence: PRD bounded contexts are listed in `microservices/sites/PRD.md` lines 116-134.

Evidence: PRD layer mapping and 78-crate statement are in `microservices/sites/PRD.md` lines 167-183.

Evidence: manifest bounded contexts are only `block`, `cdn-delivery`, `cms-collection`, `domain-binding`, `page`, `search`, and `site` in `microservices/sites/manifest.json` lines 6-65.

Evidence: manifest layer set is only `adapter`, `app`, and `kernel` in `microservices/sites/manifest.json` lines 66-70.

Impact: an implementation agent reading the manifest as machine-readable truth would omit `theme`, `navigation`, `url-routing`, and `seo` as first-class BCs.

Impact: an implementation agent would also underbuild the 13-layer architecture because the manifest omits domain, usecase, api, rest, worker, sdk, and backend-qualified adapter layers.

The OpenAPI contract contradicts the manifest by naming the missing BCs as tags.

Evidence: OpenAPI tags include `themes`, `navigation`, `url-routing`, and `seo` in `microservices/sites/contracts/openapi/sites.yaml` lines 34-58.

The proto contract also contradicts the manifest by defining `UrlRoutingService` and `SeoService`.

Evidence: proto services include `UrlRoutingService` and `SeoService` in `microservices/sites/contracts/proto/sites.proto` lines 202-211.

The manifest still depends on `connect` even though the PRD states that sites is standalone after Connect unbundling.

Evidence: PRD standalone/unbundle statement is in `microservices/sites/PRD.md` lines 20-26.

Evidence: manifest dependency list includes `connect` in `microservices/sites/manifest.json` lines 410-430.

Impact: the product boundary remains ambiguous; a builder could wire new work back into the deprecated suite.

The capability entries in manifest are internally inconsistent with their names.

Evidence: manifest assigns `"tier": "T1"` to `T0-suggest`, `T1-assist`, and `T2-auto` in `microservices/sites/manifest.json` lines 82-100.

Impact: capability risk and autonomy handling can be misrouted before the Wave 15J terminology cleanup even begins.

IP-013 expects different contract versions than the checked-in contracts.

Evidence: IP-013 acceptance criteria require OpenAPI 3.1 and AsyncAPI 3.0 in `microservices/sites/IP-013-contracts-and-capabilities.md` lines 44-51.

Evidence: OpenAPI is 3.2.0 in `microservices/sites/contracts/openapi/sites.yaml` line 1.

Evidence: AsyncAPI is 3.1.0 in `microservices/sites/contracts/asyncapi/sites-events.yaml` line 1.

Impact: a CI agent following IP-013 may pin the wrong linter/spec version or fail correct contracts.

IP-015 claims phase-exit maturity depends on evidence files, but the evidence directory is absent.

Evidence: IP-015 requires `microservices/sites/evidence/multispectrum/*.json` in `microservices/sites/IP-015-hg-sites-maturity-claim.md` lines 31-38 and 50-58.

Evidence: inventory found no `microservices/sites/evidence/` path.

Impact: maturity status cannot be asserted as complete.

The architecture document has an explicit generated anchor-sweep marker.

Evidence: `microservices/sites/ARCHITECTURE.md` line 3 says the file was created by Wave-3-C anchor-sweep and must be expanded during content-pass review.

Impact: the doc contains useful sections, but the marker itself proves the artifact is not fully converted from sweep output to settled architecture.

The architecture document repeats `tier product` inventory lines across anchors.

Evidence: examples appear at `microservices/sites/ARCHITECTURE.md` lines 22-30, 84-90, 584-592, and 646-654.

Impact: repeated template inventory is not itself false, but it is a signal that the audit must prefer concrete product sections and contracts over generated anchor repetition.

Dimension 1 finding severity: P1 for manifest/product contract contradiction and stale Connect dependency.

### Section 3.2 - Dimension 2, Outbound Cross-References

Verdict: FINDING.

The PRD references many sibling microservices and correctly states that cross-product flows should go through Workflow or Ontology.

Evidence: `microservices/sites/PRD.md` lines 208-210 list consumed microservices and prohibit direct product imports.

The manifest dependency graph is broader and includes `connect`.

Evidence: `microservices/sites/manifest.json` lines 410-430 list dependencies, including `connect`.

The PRD explicitly says legacy Connect sites migrate away under the strangler timeline.

Evidence: `microservices/sites/PRD.md` line 24 points to `migration-from-connect.md` and says the legacy `oya-connect-sites-*` family is deprecated.

The cross-service references to `forms`, `community`, `docs`, `social`, `mail`, `workflow-engine`, `ontology`, `observability`, `tenancy`, and `audit-chain` are plausible for the product surface.

Evidence: Workflow events produced and consumed are listed in `microservices/sites/PRD.md` lines 229-253.

The OpenAPI contract does not expose cross-service form submission storage, which aligns with PRD's rule that `sites` never persists form data directly.

Evidence: PRD security rule is in `microservices/sites/PRD.md` lines 89-95.

The AsyncAPI contract emits events consumed by forms, community, docs, mail, workflow-engine, audit-chain, and observability.

Evidence: `microservices/sites/contracts/asyncapi/sites-events.yaml` lines 8-20 list those consumers.

The dependency on `cell` is likely a platform-pattern dependency, but the manifest treats it as a microservice dependency.

Evidence: `microservices/sites/manifest.json` lines 423-428 include `network`, `intelligence`, `ontology`, `detection`, `cell`, and `cloud-iac`.

Impact: after the user's active direction that cell is a pattern rather than service, this dependency should be reviewed in the broader Wave 15L path, but this audit records only the sites-local manifestation.

The migration playbook exists for Webflow, Wix Business, and WordPress, but there is no Squarespace-specific playbook despite Squarespace being one of the batch top-3 counterparts.

Evidence: inventory includes `microservices/sites/migration-playbooks/from-webflow-wix-business-and-wordpress.md`, not a Squarespace playbook.

Evidence: PRD competitive list includes Squarespace in `microservices/sites/PRD.md` lines 281-295.

Impact: migration parity is incomplete for one of the top-3 counterparts.

Dimension 2 finding severity: P2 for migration/cross-reference documentation gaps, P1 where `connect` keeps a retired boundary alive.

### Section 3.3 - Dimension 3, Substance Bar

Verdict: FINDING.

The PRD is substantive.

Evidence: PRD purpose and tenant outcomes are specific in `microservices/sites/PRD.md` lines 20-37.

Evidence: PRD functional requirements are specific in `microservices/sites/PRD.md` lines 41-70.

Evidence: PRD performance, security, audit, availability, and data residency sections are specific in `microservices/sites/PRD.md` lines 72-114.

The contracts are substantive.

Evidence: OpenAPI endpoints and schemas are concrete in `microservices/sites/contracts/openapi/sites.yaml` lines 60-120 and 317-476.

Evidence: AsyncAPI channels and envelope are concrete in `microservices/sites/contracts/asyncapi/sites-events.yaml` lines 33-90 and 185-240.

Evidence: proto service and messages are concrete in `microservices/sites/contracts/proto/sites.proto` lines 67-95 and 175-240.

The capacity model is substantive.

Evidence: per-tenant demand and per-cell aggregates are listed in `microservices/sites/capacity-model.md` lines 20-49.

Evidence: capacity envelope and saturation indicators are listed in `microservices/sites/capacity-model.md` lines 51-65 and 133-144.

The SLO files are substantive and have numeric targets.

Evidence: page-render SLO uses p95 <= 200 ms in `microservices/sites/slos/page-render-latency.openslo.yaml` lines 16-38.

Evidence: static-asset SLO uses p95 <= 100 ms in `microservices/sites/slos/static-asset-latency.openslo.yaml` lines 16-38.

Evidence: publish SLO uses p95 <= 5 s for a 100-page site in `microservices/sites/slos/publish-latency.openslo.yaml` lines 16-38.

The architecture document contains many concrete controls, but the generated marker at line 3 remains a substance concern.

Evidence: `microservices/sites/ARCHITECTURE.md` line 3.

The architecture deployment-shape anchor is specific about Kubernetes, Cloud Hypervisor, Kata, sidecars, and rollout behavior.

Evidence: `microservices/sites/ARCHITECTURE.md` lines 571-582.

The architecture abuse-defense anchor is specific about edge rate limits, fingerprints, bot scoring, anti-spoof, and anti-scrape.

Evidence: `microservices/sites/ARCHITECTURE.md` lines 695-706.

The implementation plans are not all sufficient to implement from current canonical direction because IP-001 is Helm/Kustomize only.

Evidence: `microservices/sites/IP-001-iac-bootstrap.md` lines 16-34 and 62-69.

The implementation plans also keep tier-lint gates.

Evidence: `microservices/sites/IP-013-contracts-and-capabilities.md` lines 10 and 25-31.

The absence of README is a usability gap for a fresh implementation agent.

Evidence: inventory found no `microservices/sites/README.md`.

The absence of `src` and `tests` means the artifact set is documentation-rich but not implementation-verifiable.

Evidence: inventory found no `microservices/sites/src/` or `microservices/sites/tests/`.

The chat-history task reminder reinforces that the batch must drop tier-deltas and use restructured performance.

Evidence: chat history line 17270.

Dimension 3 finding severity: P2 for generated-marker/README/test gaps, P1 where stale implementation plans contradict mandatory OpenTofu and tenant-class direction.

### Section 3.4 - Dimension 4, Canonical-Direction Alignment

Verdict: FINDING.

The sites PRD predates the 2026-05-20 canonical updates and does not include `deployment_contexts`.

Evidence: `specs/master-plan-sequencing.json` lines 704-746 define the six context IDs and target paths.

Evidence: `microservices/sites/manifest.json` has no `deployment_contexts` field in lines 1-435.

The sites PRD predates the tier retirement and still has `tier: tenant-facing` in frontmatter.

Evidence: `microservices/sites/PRD.md` line 8.

The manifest keeps `tenant_class`, `tier_classification`, and `criticality_tier`.

Evidence: `microservices/sites/manifest.json` lines 355-359, 383-385, and 431-432.

The SLO files label service objectives by `tier`.

Evidence: page-render SLO label appears in `microservices/sites/slos/page-render-latency.openslo.yaml` lines 7-12.

Evidence: static-asset SLO label appears in `microservices/sites/slos/static-asset-latency.openslo.yaml` lines 7-12.

Evidence: publish SLO label appears in `microservices/sites/slos/publish-latency.openslo.yaml` lines 7-12.

The policies gate AI page build on `tenant_tier` values rather than tenant-class claims.

Evidence: `microservices/sites/policy/tenant-scope.cedar` lines 142-153.

The CI policy uses `tenant_class == "synthetic_test"`, which is a CI-only test classification and not the canonical tenant classes requested by this batch.

Evidence: `microservices/sites/policy/ci-scope.cedar` lines 54-55 and 79-81.

The capability YAMLs continue to refer to canonical-tier schemas and tenant-tier gates.

Evidence: `microservices/sites/capabilities/T0-suggest.yaml` lines 1, 13, 133, and 136 from the tenant-class grep.

Evidence: `microservices/sites/capabilities/T1-assist.yaml` lines 1, 14, 165, and 168 from the tenant-class grep.

Evidence: `microservices/sites/capabilities/T2-auto.yaml` lines 1, 14, 45, 99, and 106 from the tenant-class grep.

The cost budget still speaks in tenant-tier and AI-tier terms.

Evidence: `microservices/sites/cost-budget.md` lines 33 and 48-49 from the tenant-class grep.

The tutorial requires a paid-tier cell.

Evidence: `microservices/sites/tutorials/launch-site-with-custom-domain-cms-and-accessibility.md` line 15.

The existing benchmark document uses tiered Oyatie rows and a `--tier` invocation.

Evidence: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md` lines 13, 19-20, 30-32, 38, 45, 51, 57, and 104-110.

The service lacks OCI Always Free profile modules.

Evidence: required path is `iac/oci-guest/always-free/` per `specs/master-plan-sequencing.json` line 864.

Evidence: actual sites IaC tree contains only Helm and Kustomize directories.

The service lacks OpenTofu modules for all six contexts.

Evidence: required context paths are in `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3861-3871.

Evidence: actual sites IaC tree contains only `microservices/sites/iac/helm` and `microservices/sites/iac/kustomize`.

The service lacks `supported-oses.json`.

Evidence: required manifest is defined in `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3950-3999 and `specs/master-plan-sequencing.json` lines 777-815.

The service is currently Rust-policy clean for authored source files because no forbidden language source files were found.

Evidence: the forbidden-language search returned no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.fsx` files under `microservices/sites`.

The proto contains `option go_package`, but that is a generated-target option in a `.proto` file, not authored Go source.

Evidence: `microservices/sites/contracts/proto/sites.proto` line 8.

Dimension 4 finding severity: P1 for canonical deployment/IaC/OS/tenant-class violations; P2 for terminology retirement and documentation-only tier remnants.

#### Section 3.4.T - Tier Retirement Candidates

Scope rule: this subsection lists only demo_trial/paid/paid/paid compliance_pack references inside `microservices/sites`; generic T0/T1/T2 autonomy language is cataloged separately as tenant-class cleanup, not as direct demo_trial/paid/paid/paid compliance_pack retirement.

Default severity: P2 Wave 15J retirement candidate, unless the line currently gates behavior.

Candidate 1: `microservices/sites/tenant-class/tier-matrix.md:13` uses "demo_trial" in a preview tier heading.

Candidate 2: `microservices/sites/tenant-class/tier-matrix.md:22` says CMS collection is not available at demo_trial.

Candidate 3: `microservices/sites/tenant-class/tier-matrix.md:28` limits image formats at demo_trial.

Candidate 4: `microservices/sites/tenant-class/tier-matrix.md:38` relaxes page-render latency at demo_trial.

Candidate 5: `microservices/sites/tenant-class/tier-matrix.md:44` uses "paid" in a production-default heading.

Candidate 6: `microservices/sites/tenant-class/tier-matrix.md:46` says paid adds to demo_trial.

Candidate 7: `microservices/sites/tenant-class/tier-matrix.md:76` uses "paid" in a multi-region heading.

Candidate 8: `microservices/sites/tenant-class/tier-matrix.md:78` says paid adds to paid.

Candidate 9: `microservices/sites/tenant-class/tier-matrix.md:101` compares paid and paid annual cell cost.

Candidate 10: `microservices/sites/tenant-class/tier-matrix.md:105` uses "paid compliance_pack" in a sovereign-pack heading.

Candidate 11: `microservices/sites/tenant-class/tier-matrix.md:107` says paid compliance_pack adds to paid.

Candidate 12: `microservices/sites/tenant-class/tier-matrix.md:117` compares paid compliance_pack latency to paid.

Candidate 13: `microservices/sites/tenant-class/tier-matrix.md:119` compares paid compliance_pack SLO posture to paid.

Candidate 14: `microservices/sites/tenant-class/tier-matrix.md:126` says some surfaces differ at demo_trial.

Candidate 15: `microservices/sites/tenant-class/tier-matrix.md:133` defines demo_trial to paid, paid to paid, and paid to paid compliance_pack promotion.

Candidate 16: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:13` says hardware is "oyatie paid".

Candidate 17: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:19` has an "oyatie sites paid" row.

Candidate 18: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:20` has an "oyatie sites paid" row.

Candidate 19: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:30` interprets "oyatie paid" against competitors.

Candidate 20: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:32` says paid hits a p99 value.

Candidate 21: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:38` has an "oyatie sites paid" row.

Candidate 22: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:45` says Oyatie paid is competitive.

Candidate 23: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:51` has an "oyatie sites paid (CDN edge)" row.

Candidate 24: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:57` says paid hits a p99 value.

Candidate 25: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:109` includes `--tier oyatie-paid` in the benchmark invocation.

Candidate 26: `microservices/sites/tutorials/launch-site-with-custom-domain-cms-and-accessibility.md:15` requires "A paid tenant_class sites cell."

False positive excluded: `microservices/sites/IP-003-page-bc-kernel.md:41` says "reference corpus"; that is not a capability-tenant_class reference.

Retirement interpretation: the entire `microservices/sites/tenant-class/tier-matrix.md` file is a Wave 15J retirement candidate because it is explicitly titled "Capability Tier Matrix (ADR-0316)" in lines 1-13.

Retirement interpretation: the benchmark doc should be reworked, not simply deleted, because it contains useful workload dimensions in `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md` lines 9-11.

Retirement interpretation: the tutorial prerequisite should become a tenant-class and deployment-context prerequisite, not a feature-tier prerequisite.

Retirement interpretation: no new tenant-class-deltas deliverable is authored for this audit.

Retirement interpretation: the new performance report in this batch uses a single industry-leader target set and overlays deployment context plus tenant class.

#### Section 3.4.C - Tenant-Class Adoption Gaps

Canonical batch requirement: tenant classes are `demo_trial`, `paid`, and `revenue_share`.

Current service expression: no `demo_trial` string was found under `microservices/sites`.

Current service expression: no `revenue_share` string was found under `microservices/sites`.

Current service expression: no production `paid` tenant-class semantics were found under `microservices/sites`.

Only `tenant_class` occurrence in active policy is CI synthetic test handling.

Evidence: `microservices/sites/policy/ci-scope.cedar` lines 54-55 require `resource.tenant_class == "synthetic_test"`.

Evidence: `microservices/sites/policy/ci-scope.cedar` line 81 repeats `resource.tenant_class == "synthetic_test"`.

The production AI page-build policy still keys off tenant tier.

Evidence: `microservices/sites/policy/tenant-scope.cedar` lines 142-153.

The production cost budget still keys off tenant-tier.

Evidence: `microservices/sites/cost-budget.md` line 33.

The runbooks still include tier-specific override language.

Evidence: `microservices/sites/runbooks/asset-optimization-degraded.md` lines 91-92 from the tenant-class grep.

Required remediation: move AI page-build admission to principal claims carrying `tenant_class` plus usage-cap status.

Required remediation: demo_trial should cap publish count, active site count, monthly page renders, AI page-build usage, custom-domain count, and CMS item count without changing feature quality.

Required remediation: paid should remove demo caps and rely on per-seat plus usage billing meters.

Required remediation: revenue_share should preserve the same product quality but emit gross-revenue attribution events to billing and audit-chain.

Required remediation: compliance pack and BYOK eligibility must be expressed as tenant-class plus contract posture, not as capability tier.

Required remediation: OpenAPI should not accept tenant_class as an end-user request parameter unless an admin control plane owns that mutation; current OpenAPI schemas in `microservices/sites/contracts/openapi/sites.yaml` lines 317-476 do not expose tenant_class, which is correct.

Required remediation: AsyncAPI events should include tenant class through trusted envelope claims or audit-chain metadata where billing and caps need evidence; current envelope includes `tenant_id` but no tenant class in `microservices/sites/contracts/asyncapi/sites-events.yaml` lines 185-197.

Required remediation: capability YAMLs should remove tenant-tier gates and cite the tenant-class authority chain.

Required remediation: dashboards should remove "by tier" panels and use tenant_class, context, and workload labels.

Evidence: `microservices/sites/dashboards/editor-experience.json` line 63 has a "T0/T1/T2 invocations by tier" panel from the tenant-class grep.

Tenant-class adoption verdict: gap present.

Tenant-class adoption severity: P1 because policy admission still depends on retired tier semantics.

### Section 3.5 - Dimension 5, Industry-Counterpart Parity

Verdict: FINDING.

The local PRD names a broad competitor set, but this batch scopes the union bar to Webflow, Squarespace, and Wix.

Evidence: PRD competitor table includes Webflow, Squarespace, and Wix in `microservices/sites/PRD.md` lines 281-295.

The existing competitor matrix is broader than this batch and includes Google Sites, WordPress, Notion Sites, Carrd, Framer, Ghost, Hugo, Sanity, Strapi, Contentful, Sitecore, and AEM.

Evidence: `microservices/sites/competitor-parity-matrix.md` lines 20-47.

The broader matrix is useful background but dilutes the top-3 union coverage needed for this audit.

Webflow parity requires visual designer, CMS collection, hosting/CDN, custom domain/TLS, SEO, forms, ecommerce, search, preview, versioning, multi-language, comments/integrations, and API operations.

Evidence: local PRD already calls Webflow a visual website builder with CMS and hosting in `microservices/sites/PRD.md` line 288.

Squarespace parity requires templates, style system, commerce, product catalog, page limits, analytics, SEO, scheduling, domains, SSL, and content limits.

Evidence: local PRD already calls Squarespace a hosted website builder with theme, ecommerce, and analytics in `microservices/sites/PRD.md` line 286.

Wix parity requires hosted website builder, CMS, ecommerce, bookings/app ecosystem, AI builder, Velo/backend extensibility, SEO, media, domains, and plan-scaled CMS limits.

Evidence: local PRD already calls Wix a hosted website builder with theme, ecommerce, and bookings in `microservices/sites/PRD.md` line 287.

The sites PRD covers Webflow/Squarespace/Wix core authoring, domain, SEO, CMS, search, forms, ecommerce stub, analytics, preview, versioning, i18n, comments, CDN, AI, and collaboration requirements in `microservices/sites/PRD.md` lines 41-70.

The local M03 gap list already admits Webflow-class visual layout designer is not present until M04.

Evidence: `microservices/sites/competitor-parity-matrix.md` lines 48-53.

The local M03 gap list admits full storefront is a future item.

Evidence: `microservices/sites/competitor-parity-matrix.md` lines 50-53.

The local PRD differentiator claim that no competitor unifies intranet and public site under one substrate remains plausible.

Evidence: `microservices/sites/PRD.md` lines 297-305.

The local PRD differentiator claim around Loro CRDT alignment is plausible but not implementation-proven because there is no `src`.

Evidence: `microservices/sites/PRD.md` lines 299-301 and inventory absence of `src`.

The current migration playbook covers Webflow and Wix but not Squarespace.

Evidence: inventory includes `microservices/sites/migration-playbooks/from-webflow-wix-business-and-wordpress.md`.

The current benchmark doc includes Webflow, Wix, Squarespace, and extra platforms but uses retired tier rows.

Evidence: `microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md` lines 9-24.

Dimension 5 severity: P2 for counterpart surface gaps where product intent exists; P1 for implementation-proof gaps where claims are used as maturity gates.

### Section 3.6 - Dimension 6, Multi-Context Deployment

Verdict: FINDING.

Required contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3854-3871.

Evidence: `specs/master-plan-sequencing.json` lines 704-746.

Required brief field: supported contexts plus N/A reasons where any context is not supported.

Evidence: `docs/standards/brief-template.md` lines 690-700.

Required audit evidence: context IDs, N/A IDs, paths inspected, tenant onboarding flow, and network/IAM/observability/billing seams.

Evidence: `docs/standards/brief-template.md` lines 786-807.

Sites has no `deployment_contexts` in manifest.

Evidence: `microservices/sites/manifest.json` lines 1-435 contain no such field.

Sites has no service-local supported-contexts doc.

Evidence: inventory found no `microservices/sites/supported-contexts.*` artifact.

Sites IaC directories do not use the six context paths.

Evidence: actual `microservices/sites/iac` tree has only Helm and Kustomize directories.

IP-001 speaks in pack overlays rather than deployment contexts.

Evidence: `microservices/sites/IP-001-iac-bootstrap.md` lines 20-34 and 40-56.

The architecture deployment shape references Kubernetes pods, Cloud Hypervisor, and Kata but not the six context IDs.

Evidence: `microservices/sites/ARCHITECTURE.md` lines 571-582.

The capacity model has per-cell scale but no six-context overlay.

Evidence: `microservices/sites/capacity-model.md` lines 51-65 and 116-131.

The multi-region doc was inventoried, but deployment-context proof still requires context-specific modules and onboarding paths.

Finding: missing `iac/oyatie-public-cloud/` module or N/A reason.

Finding: missing `iac/guest-on-aws/` module or N/A reason.

Finding: missing `iac/oci-guest/` module or N/A reason.

Finding: missing `iac/on-prem/` module or N/A reason.

Finding: missing `iac/colo/` module or N/A reason.

Finding: missing `iac/oyatie-iaas/` module or N/A reason.

Finding: missing tenant onboarding flow through OpenTofu for each supported context.

Finding: missing network seam by context.

Finding: missing IAM seam by context.

Finding: missing observability seam by context.

Finding: missing billing seam by context.

Impact: public website/intranet hosting is a tenant-facing product promise; if context placement is not explicit, downstream implementation can become public-cloud-only by accident.

Dimension 6 severity: P1 because a non-Big-8 in-scope microservice has a false or ungrounded deployment support posture under ADR-0328 lines 4106-4110.

### Section 3.7 - Dimension 7, OpenTofu IaC

Verdict: FINDING.

OpenTofu is the canonical IaC engine.

Evidence: `specs/master-plan-sequencing.json` lines 747-775.

ADR-0328 requires OpenTofu for every supported context.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3897-3940.

Required files are `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md`.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3912-3920.

Brief-template required files are the same.

Evidence: `docs/standards/brief-template.md` lines 840-850.

Sites currently has no `.tf` files under `microservices/sites`.

Evidence: inventory and IaC tree show only Helm and Kustomize.

IP-001 explicitly defines the IaC bootstrap as Helm plus Kustomize.

Evidence: `microservices/sites/IP-001-iac-bootstrap.md` line 16.

IP-001 acceptance gates are Helm lint, Kustomize dry run, and cargo governance checks.

Evidence: `microservices/sites/IP-001-iac-bootstrap.md` lines 62-69.

There is no `tofu init`, `tofu plan`, or `tofu apply` path in IP-001.

Evidence: `microservices/sites/IP-001-iac-bootstrap.md` lines 62-69.

There is no OpenTofu version pin.

Evidence: no `versions.tf` file exists under `microservices/sites/iac`.

There is no provider pin.

Evidence: no OpenTofu module file exists under `microservices/sites/iac`.

There is no module signing evidence.

Evidence: no OpenTofu module package or cosign evidence exists under `microservices/sites/iac`.

There is no state backend mapping by context.

Evidence: no context module directories exist under `microservices/sites/iac`.

There is no OCI Always Free profile module.

Evidence: required `iac/oci-guest/always-free/` path from `specs/master-plan-sequencing.json` line 864 is absent.

Helm and Kustomize can still be downstream Kubernetes deployment artifacts.

But they cannot be the canonical tenant-provisioning substrate under the current OpenTofu rule.

Dimension 7 severity: P1.

### Section 3.8 - Dimension 8, OS Support

Verdict: FINDING.

ADR-0328 requires `supported-oses.json`.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3950-3999.

Master sequencing requires per-microservice OS manifest.

Evidence: `specs/master-plan-sequencing.json` line 814.

Tier 1 OSes are enumerated in `specs/master-plan-sequencing.json` lines 779-793.

Tier 2 test-only OSes are enumerated in `specs/master-plan-sequencing.json` lines 794-797.

Out-of-scope OSes are enumerated in `specs/master-plan-sequencing.json` lines 798-805.

Architecture matrix is enumerated in `specs/master-plan-sequencing.json` lines 806-812.

Brief-template requires the OS manifest path and CI evidence.

Evidence: `docs/standards/brief-template.md` lines 967-1123.

Sites has no `microservices/sites/supported-oses.json`.

Evidence: inventory absence.

Sites has no package-format matrix for RPM, DEB, container image, Talos extension, Flatcar extension, macOS package, or Homebrew mapping.

Evidence: inventory and OS search found no such manifest.

Sites has Kubernetes Helm and Kustomize artifacts, but a Kubernetes deployment unit does not replace OS compatibility claims for on-prem and colo contexts.

Evidence: `docs/standards/brief-template.md` lines 1074-1092.

There is no Oracle Linux arm64/Ampere evidence for the OCI context.

Evidence: `docs/standards/brief-template.md` lines 1094-1096 require this where OCI applies.

There is no explicit exclusion of Intel macOS, pre-M5 Apple Silicon, FreeBSD, OpenBSD, Windows Server, or Solaris.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 3981-3988.

There are no `src` or `tests` directories to prove portability.

Evidence: inventory absence.

Dimension 8 severity: P1 because deployment support would be ungrounded.

### Section 3.9 - Dimension 9, Rust-Strict Language Policy

Verdict: PASS with documentation gap.

Master sequencing makes Rust the backend language.

Evidence: `specs/master-plan-sequencing.json` lines 817-855.

ADR-0328 allows `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, OpenAPI YAML, AsyncAPI YAML, OpenSLO YAML, SQL migrations, and Markdown for backend artifacts.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 4011-4032.

ADR-0328 forbids Python, JavaScript application logic, TypeScript application logic, Ruby, Perl, PHP, Java, Scala, Groovy, Go, F#, and C++ except approved FFI shims.

Evidence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 4050-4078.

The sites path has no authored forbidden-language source files.

Evidence: forbidden-language file search returned empty.

The sites path currently has no Rust `src` files either.

Evidence: inventory absence.

The reference implementation is a Markdown doc for a Rust SDK, not Rust source.

Evidence: `microservices/sites/reference-implementations/cms-collection-and-page-rust-sdk.md` was inventoried as documentation.

The proto `go_package` option is not authored Go source and falls under generated target metadata.

Evidence: `microservices/sites/contracts/proto/sites.proto` line 8.

IP-001 acceptance gates use Helm and kubectl command examples.

Evidence: `microservices/sites/IP-001-iac-bootstrap.md` lines 62-69.

Those command examples are not application logic, but the OpenTofu direction will require replacing the primary provisioning gate.

Rust policy gap: manifest does not declare the canonical build invocation.

Evidence: canonical invocation is `cargo build --workspace --release --all-features --locked` in `specs/master-plan-sequencing.json` line 853.

Rust policy gap: there is no service-local ADR for any non-Rust exception because no exception appears needed.

Dimension 9 severity: no P1 language violation found; P2 documentation gap for build posture.

## Section 4 - Findings Table

| ID | Severity | Dimension | Finding | Evidence |
|---|---:|---|---|---|
| SITES-COH-001 | P1 | Dim 1 | Product model split: PRD/contracts define 11 BCs and 78 crates, manifest defines 7 BCs and 3 layers. | `PRD.md:116-183`; `manifest.json:6-70`; `openapi/sites.yaml:34-58`; `proto/sites.proto:202-211` |
| SITES-COH-002 | P1 | Dim 1/2 | Manifest still depends on `connect` after PRD says sites is standalone after Connect unbundle. | `PRD.md:20-26`; `manifest.json:410-430` |
| SITES-COH-003 | P1 | Dim 4/6 | Six deployment contexts are not declared and no context modules or N/A reasons exist. | `ADR-0328:3854-3871`; `master-plan-sequencing.json:704-746`; inventory absence |
| SITES-COH-004 | P1 | Dim 7 | OpenTofu is absent; current IaC is Helm/Kustomize only. | `master-plan-sequencing.json:747-775`; `IP-001-iac-bootstrap.md:16-69`; IaC inventory |
| SITES-COH-005 | P1 | Dim 8 | `supported-oses.json` and OS/arch/package CI matrix are absent. | `master-plan-sequencing.json:777-815`; `brief-template.md:967-1123`; inventory absence |
| SITES-COH-006 | P1 | Dim 4 | Tenant-class model is absent from production semantics; policy still gates AI on `tenant_tier`. | `tenant-scope.cedar:142-153`; `ci-scope.cedar:54-55,81` |
| SITES-COH-007 | P1 | Dim 1 | IP-013 expects OpenAPI 3.1 and AsyncAPI 3.0 while checked-in contracts are OpenAPI 3.2.0 and AsyncAPI 3.1.0. | `IP-013-contracts-and-capabilities.md:44-51`; `openapi/sites.yaml:1`; `asyncapi/sites-events.yaml:1` |
| SITES-COH-008 | P1 | Dim 1/3 | Hyperscaler maturity claim depends on missing evidence files. | `IP-015-hg-sites-maturity-claim.md:31-58`; inventory absence |
| SITES-COH-009 | P2 | Dim 4 | 26 line-level demo_trial/paid/paid/paid compliance_pack retirement candidates remain. | Section 3.4.T |
| SITES-COH-010 | P2 | Dim 4 | Generic `tier`, `tenant_class`, `tier_classification`, and SLO `tier` labels remain and need Wave 15J tenant-class rewrite. | `manifest.json:355-385`; SLO label lines; capability grep |
| SITES-COH-011 | P2 | Dim 4/7 | OCI Always Free profile module is absent. | `master-plan-sequencing.json:857-865`; IaC inventory |
| SITES-COH-012 | P2 | Dim 3 | Architecture still carries generated anchor-sweep marker. | `ARCHITECTURE.md:3` |
| SITES-COH-013 | P2 | Dim 3 | README is absent for a fresh implementation agent. | inventory absence |
| SITES-COH-014 | P2 | Dim 5 | Existing benchmark doc uses retired tier rows and extra competitor scope. | `benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:9-24,104-110` |
| SITES-COH-015 | P2 | Dim 5 | Squarespace migration path is not present despite top-3 counterpart scope. | inventory migration playbooks; `PRD.md:281-295` |
| SITES-COH-016 | P2 | Dim 9 | Rust-strict source policy passes, but canonical build posture is not expressed in a service manifest because no source/build manifest exists. | `master-plan-sequencing.json:817-855`; inventory absence |

### Finding Detail - SITES-COH-001

Severity: P1.

The PRD is the richest product articulation and says there are eleven primary bounded contexts.

The manifest omits four of those contexts.

The contracts retain several omitted contexts.

The conflict is not cosmetic because manifest-driven code generation, gate registration, dependency graphs, and catalog checks can underbuild the service.

The remediation is to regenerate or hand-update `manifest.json` so its bounded contexts and layers match the PRD and contracts.

The remediation must not reintroduce tenant-class semantics while doing so.

### Finding Detail - SITES-COH-002

Severity: P1.

The PRD says sites is standalone and no longer Connect.

The manifest still depends on Connect.

This can wire future work through a retired boundary.

The remediation is to delete the Connect dependency from sites-local machine-readable surfaces after confirming the migration doc has the strangler history.

### Finding Detail - SITES-COH-003

Severity: P1.

The six deployment contexts are mandatory unless a service-local N/A reason is recorded.

There are no service-local N/A reasons.

There are no context-specific directories.

The remediation is to add `deployment_contexts` to the machine-readable manifest and bind each supported context to OpenTofu module paths.

If any context is impossible, the reason must name the missing primitive and a revisit gate.

### Finding Detail - SITES-COH-004

Severity: P1.

The current IaC plan and file tree are Kubernetes packaging artifacts, not OpenTofu tenant provisioning artifacts.

OpenTofu must own the tenant provisioning path, state backend, provider versions, and module signing.

Helm and Kustomize can remain as payloads invoked by OpenTofu modules.

The remediation is not to rename Helm to OpenTofu; it is to add real OpenTofu modules per context and keep Helm/Kustomize as subordinate deploy artifacts where appropriate.

### Finding Detail - SITES-COH-005

Severity: P1.

The service claims hosted and on-prem/colo-relevant behavior through the batch default contexts, but it has no OS matrix.

The OS support artifact must define Tier 1 rows, Tier 2 test-only rows, exclusions, architecture matrix, package format mapping, and CI gate status.

The remediation should include Oracle Linux arm64 for OCI and container compatibility for all Linux Tier 1 OSes.

### Finding Detail - SITES-COH-006

Severity: P1.

Production policy is not aligned to tenant classes.

The existing `tenant_tier` gate maps to SaaS feature-tier behavior and conflicts with the retired-tier doctrine.

The remediation is to shift policy to trusted tenant_class claims plus usage caps, compliance-pack eligibility, BYOK eligibility, and revenue-share billing evidence where applicable.

### Finding Detail - SITES-COH-007

Severity: P1.

IP-013 is stale relative to the checked-in contract versions.

This can make CI reject the correct contract or encourage downgrading contracts to match the IP.

The remediation is to update IP-013 acceptance criteria to OpenAPI 3.2.0 and AsyncAPI 3.1.0, then remove tenant-class-lint gate names.

### Finding Detail - SITES-COH-008

Severity: P1.

IP-015 cannot be accepted without evidence files.

The implementation plan claims 30-day SLO eligibility and reviewer approvals, but the evidence path is absent.

The remediation is to keep maturity claim status non-green until the evidence directory and linked gate outputs exist.

### Finding Detail - SITES-COH-009

Severity: P2.

The direct retired tier terms are isolated mostly in the old tier matrix, benchmark doc, and tutorial prerequisite.

The recommended Wave 15J action is to retire the tier matrix, rewrite the benchmark into a context/tenant-class overlay, and rewrite the tutorial prerequisite.

### Finding Detail - SITES-COH-010

Severity: P2.

Generic tier vocabulary remains across service docs and machine-readable surfaces.

Not every `tier` token is demo_trial/paid/paid/paid compliance_pack, but the terms are ambiguous after the retirement directive.

The remediation is to distinguish autonomy level, criticality, workload class, tenant class, and deployment context explicitly.

### Finding Detail - SITES-COH-011

Severity: P2.

The OCI Always Free profile is a required demo/trial infrastructure profile.

The absence is already covered by the bigger OpenTofu gap, but it deserves its own tracking row because demo_trial adoption depends on it.

The remediation is `microservices/sites/iac/oci-guest/always-free/` with hard limits for page renders, active sites, CMS items, publish jobs, and AI usage.

### Finding Detail - SITES-COH-012

Severity: P2.

The architecture doc is useful but still self-identifies as anchor-sweep output.

The remediation is a content-pass edit that removes the marker only after section content is made non-templated and contradictions are resolved.

### Finding Detail - SITES-COH-013

Severity: P2.

The missing README makes cold-start implementation harder.

The README should summarize product purpose, canonical context/IaC/OS/language policy, run/test commands, and current blockers.

### Finding Detail - SITES-COH-014

Severity: P2.

The old benchmark has useful workload ideas but wrong structure.

The new performance deliverable in this batch replaces tier rows with a single industry-leader target set and overlays.

### Finding Detail - SITES-COH-015

Severity: P2.

The migration path covers Webflow and Wix but not Squarespace.

Squarespace is a top-3 counterpart in this batch.

The remediation is a Squarespace import playbook for pages, navigation, products, image assets, SEO metadata, domains, redirects, analytics, and commerce content.

### Finding Detail - SITES-COH-016

Severity: P2.

No forbidden source files were found, so Rust-strict passes.

But no Rust build manifest or source exists under the service, so source build posture is not demonstrable.

The remediation is to add the Rust crates or explicit workspace ownership evidence before claiming implementation completeness.

## Section 5 - Open Questions

Open question 1: Should `sites` support all six contexts immediately, or should any context have a service-local N/A reason with a named blocker?

Open question 2: Does the user-required `revenue_share` tenant class remain a separate class for sites, or should it become a paid billing component per the older memory file?

Open question 3: Should `tenant_class` appear only in identity/gateway claims, or should sites events include a trusted denormalized tenant_class field for audit and billing proofs?

Open question 4: Should the service manifest be the canonical machine-readable source for bounded contexts, or should PRD plus contracts drive manifest regeneration?

Open question 5: Should `connect` remain only in historical migration docs after remediation, with all live manifest dependencies removed?

Open question 6: Should the `cell` dependency be retained as a service dependency until Wave 15L, or replaced with pattern/runtime terminology in sites-local docs now?

Open question 7: Should `sites` expose a hosted visual designer in M03, or is the Webflow-class visual layout gap explicitly M04 onward?

Open question 8: What is the canonical demo_trial cap package for sites: active sites, pages, CMS items, custom domains, monthly page views, AI page builds, storage, or all of the above?

Open question 9: What usage events must sites emit to cloud-billing for paid and revenue_share tenants?

Open question 10: Should anonymous page-render metering be charged by request, bandwidth, cache miss, or published-site allocation?

Open question 11: Should the OCI Always Free profile allow custom domains, or only Oyatie subdomains for demo_trial tenants?

Open question 12: Should demo_trial tenants receive ACME certificates, or should certificates be limited to paid and revenue_share contexts because of rate-limit and abuse risk?

Open question 13: Should AI page-build be usage-capped for demo_trial and usage-billed for paid/revenue_share, or should it require an explicit contract addendum?

Open question 14: Should existing T0/T1/T2 capability YAMLs be renamed away from `tier` while preserving autonomy-level semantics?

Open question 15: Should SLO labels currently named `tier` become `slo_class`, `workload_class`, or `criticality_class`?

Open question 16: Should the old capability matrix be deleted in Wave 15J or retained under a retired-history folder?

Open question 17: Should the old benchmark doc be superseded by this batch performance document, or should it be edited in place during Wave 15J?

Open question 18: Should Squarespace import be its own migration playbook or appended to the existing Webflow/Wix/WordPress playbook?

Open question 19: Should the manifest keep `network`, `intelligence`, and `detection` dependencies, or are some of those cross-service references too broad for sites?

Open question 20: Should `sites` define a public-read SLA separate from editor-write SLA in the manifest and OpenAPI extensions?

Open question 21: Should page render SLO include Core Web Vitals LCP/INP/CLS synthetic checks in addition to backend p95/p99 latency?

Open question 22: Should tenant-class compliance-pack eligibility live in sites Cedar policy or only in platform policy-engine overlays?

Open question 23: Should revenue_share tenants require storefront/payment event integration before publishing commerce pages?

Open question 24: Should demo_trial tenants be allowed to export site content after trial expiry without converting to paid?

Open question 25: Should `supported-oses.json` include macOS M5+ for local authoring tools, or only for admin CLI/package tooling outside the microservice path?

Open question 26: Should proto generated-language options be restricted or annotated for Rust-strict provenance even when no generated files are checked in?

Open question 27: Should `IP-015` remain pending until 30-day SLO evidence exists, or should it be split into registration and evidence phases?

Open question 28: Should architecture marker cleanup happen before or after manifest regeneration?

Open question 29: Should OpenTofu modules own only substrate resources, or also invoke Helm/Kustomize installation through a cloud-iac-approved provider boundary?

Open question 30: Should the `sites` API include admin-only endpoints for demo_trial cap introspection, or should that remain in cloud-billing/identity surfaces?

<!-- ORCHESTRATOR REPORT
  µservice: sites
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/sites/coherence-audit-2026-05-20.md (1354 lines)
    - /Users/jasonlee/oyatie/microservices/sites/feature-parity-matrix-2026-05-20.md (574 lines)
    - /Users/jasonlee/oyatie/microservices/sites/performance-benchmark-numbers-2026-05-20.md (1174 lines)
  inventory_files_seen: 123
  inventory_lines_read: 19393
  chat_history_matches_processed: 2
  findings_p0: 0
  findings_p1: 8
  findings_p2: 8
  findings_p3: 0
  tier_retirement_candidates_found: 26 line-level candidates; microservices/sites/tenant-class/tier-matrix.md:13,22,28,38,44,46,76,78,101,105,107,117,119,126,133; microservices/sites/benchmarks/sites-vs-webflow-vs-wix-vs-wordpress-vs-ghost.md:13,19,20,30,32,38,45,51,57,109; microservices/sites/tutorials/launch-site-with-custom-domain-cms-and-accessibility.md:15
  tenant_class_adoption_gaps: yes - no first-class demo_trial/paid/revenue_share semantics; only synthetic_test CI attributes and production tenant_tier gates were found
  top_3_counterparts_confirmed: Webflow / Squarespace / Wix
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 3102
-->
