# plugin-app-store ownership-coherence audit

Audit date: 2026-05-20.
Auditor: sole-owner Wave 3 Batch 3.2 audit lane.
Target microservice: `microservices/plugin-app-store/`.
Deployable-context assumption under test: all six canonical contexts.
Counterpart bar: VS Code Marketplace, Chrome Web Store, Shopify App Store.
Deliverable scope: coherence audit only; no capability-ladder delta deliverable is authored.
Retired model notice: retired four-label ladder capability ladders are retirement findings, not a design surface.
Replacement model tested: `demo_trial`, `paid`, and `revenue_share` tenant classes from the current batch directive.
Canonical sequence anchor: ADR-0328 D-15 through D-20.
Master-plan anchor: `specs/master-plan-sequencing.json`.
Brief-template anchor: `docs/standards/brief-template.md`.
Constraint-memory anchor: May 20 provider, OpenTofu, OS, Rust, OCI, no-ladder, tenant-class, and ownership notes.
Chat-history anchor: `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl` lines mentioning `plugin-app-store`.
Read-only investigation inventory count: 147 files under `microservices/plugin-app-store/`.
Read-only investigation line volume: 19,533 lines under `microservices/plugin-app-store/`.
Source-file sample: no `src/*` source file exists; `src/` is an empty directory observed during inventory.
Test-file sample: no `tests/*` directory exists; PRD and phase docs reference test commands anyway.
Commit status: no commit requested and no commit produced.
Touch boundary: this report and companion reports are the only authored files.

## §1 Purpose

This audit determines whether `plugin-app-store` is a coherent owned microservice rather than a collection of useful but conflicting marketplace notes.
The product purpose in `PRD.md` is a third-party plugin and app distribution surface, not the broader retail/community marketplace; `PRD.md:23-31` states the scope and exclusion boundary.
The same PRD commits to discovery, install, governance, trust, billing, vetting, and audit outcomes for tenants; `PRD.md:37-44` is the tenant-facing outcome list.
The chat history reinforces that `plugin-app-store` must remain distinct from `marketplace` and `community`; chat line `8f603fc7...jsonl:3` names that as a common pitfall.
The chat history also says shared marketplace substrates should be built first and `plugin-app-store` should depend on them for plugin monetization; `8f603fc7...jsonl:776` is the sequencing anchor.
The same prior planning says pre-existing plugin-app-store schemas should migrate toward marketplace substrates instead of redefining the whole substrate locally; `8f603fc7...jsonl:1249-1252` is the migration anchor.
The target counterpart set is confirmed in chat as VS Code Marketplace, Chrome Web Store, and Shopify App Store; `8f603fc7...jsonl:16290` and `8f603fc7...jsonl:16311` name the top three.
The canonical direction now requires all six deployment contexts unless an explicit service-local rationale says otherwise; `specs/master-plan-sequencing.json:704-746` names the contexts.
The canonical IaC substrate is OpenTofu; `specs/master-plan-sequencing.json:747-776` names OpenTofu, required context paths, provider pinning, and forbidden handroll patterns.
The canonical OS surface requires a per-microservice manifest covering the supported OS matrix; `specs/master-plan-sequencing.json:777-816` names the Tier-1 OS list and manifest requirement.
The canonical language policy makes Rust the strict backend language and limits frontend exceptions; `specs/master-plan-sequencing.json:817-856` names Rust, Swift, Kotlin, WinUI 3, and Leptos/WASM SSR exceptions.
The canonical OCI Always Free profile requires an `iac/oci-guest/always-free/` module for eligible demo/sandbox/trial/dev footprints; `specs/master-plan-sequencing.json:857-867` names the path and caps.
The May 20 no-ladder doctrine says the four-label capability system is being retired and old tier language must be scrubbed; `feedback_no_capability_ladder_2026_05_20.md:10-24` is the retirement anchor.
The same no-ladder doctrine says OCI Always Free should be expressed as a tenant-class infrastructure profile, not as a tier; `feedback_no_capability_ladder_2026_05_20.md:28-45` is the re-expression anchor.
The May 20 tenant-class memory names demo and paid class semantics while the current batch directive supersedes it by adding `revenue_share`; `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:23-35` and current task text create the combined test.
The ownership-coherence memory requires one owner per microservice and a contradiction pass over PRD, architecture, ADRs, implementation plans, contracts, SLOs, and supporting docs; `feedback_microservice_ownership_coherence_2026_05_20.md:10-15` and `feedback_microservice_ownership_coherence_2026_05_20.md:18-45` define that bar.
The substance memory rejects line-count-only output and requires evidence that deliverables can drive implementation; `feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-12` and `feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` define the bar.
The audit therefore tests ownership, product purpose, contract coherence, canonical alignment, implementation buildability, runtime operability, counterpart coverage, documentation substance, and migration sequencing.

## §2 Complete inventory

Inventory scope: every file under `microservices/plugin-app-store/` observed during read-only investigation.
Inventory summary: 147 files, 19,533 total lines.
Inventory gap: `README.md` is absent even though the requested artifact list explicitly called for it.
Inventory gap: `cross-microservice-handoffs.md` is absent even though the requested artifact list explicitly called for it.
Inventory gap: `supported-oses.json` is absent even though the master plan requires a per-microservice supported OS manifest at `specs/master-plan-sequencing.json:777-816`.
Inventory gap: `src/` exists as an empty directory, so the Rust-strict source surface is not implemented.
Inventory gap: `tests/` is absent, while `PRD.md:110-124` and `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:121-126` cite test commands and gates.
Inventory observation: `iac/` contains Helm chart material but no canonical six-context OpenTofu directories required by `specs/master-plan-sequencing.json:747-776`.
Inventory observation: `capability-ladders/` exists and is a Wave 15J retirement candidate under `feedback_no_capability_ladder_2026_05_20.md:10-24`.
Inventory item 001: `ARCHITECTURE.md`.
Inventory item 002: `IP-journey-j100-pack-rollout-first-action.md`.
Inventory item 003: `IP-journey-j115-api-capability-entitlement.md`.
Inventory item 004: `IP-journey-j116-publish-install-catalog.md`.
Inventory item 005: `IP-journey-j119-marketplace-auction-surface.md`.
Inventory item 006: `IP-journey-j148-marketplace-return-flow.md`.
Inventory item 007: `IP-journey-j150-creator-brand-marketplace.md`.
Inventory item 008: `IP-journey-j40-vendor-subscription.md`.
Inventory item 009: `IP-journey-j49-marketplace-case-context.md`.
Inventory item 010: `IP-journey-j73-catalog-publication.md`.
Inventory item 011: `IP-journey-j74-install-flow.md`.
Inventory item 012: `IP-journey-j75-quarantine.md`.
Inventory item 013: `IP-journey-j90-marketplace-app-surface.md`.
Inventory item 014: `IP-journey-j91-us-msb-mtl-overlay.md`.
Inventory item 015: `IP-journey-j92-br-lgpd-us-parent-dsar.md`.
Inventory item 016: `IP-journey-j93-in-dpdpa-rbi-overlay.md`.
Inventory item 017: `IP-journey-j94-sox404-public-company-controls.md`.
Inventory item 018: `IP-journey-j95-iso27001-soc2-annual-audit.md`.
Inventory item 019: `IP-journey-j96-ksa-uae-mena-onboarding.md`.
Inventory item 020: `IP-journey-j97-sg-pdpa-mas-tenant.md`.
Inventory item 021: `IP-journey-j98-au-privacy-apra-cps234.md`.
Inventory item 022: `IP-journey-j99-multi-pack-conflict-resolution.md`.
Inventory item 023: `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md`.
Inventory item 024: `PRD.md`.
Inventory item 025: `backfill-replay.md`.
Inventory item 026: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md`.
Inventory item 027: `capabilities/plugin-install.yaml`.
Inventory item 028: `capabilities/plugin-revoke.yaml`.
Inventory item 029: `capabilities/plugin-vetting-decide.yaml`.
Inventory item 030: `capability-ladders/tier-matrix.md`.
Inventory item 031: `capacity-model.md`.
Inventory item 032: `catalog/addon-sdk-adapter.yaml`.
Inventory item 033: `catalog/agent-capability-pack.yaml`.
Inventory item 034: `catalog/agent-review-bot.yaml`.
Inventory item 035: `catalog/api-connector-hub.yaml`.
Inventory item 036: `catalog/approval-workflow-plus.yaml`.
Inventory item 037: `catalog/audit-evidence-exporter.yaml`.
Inventory item 038: `catalog/bi-connector.yaml`.
Inventory item 039: `catalog/compliance-pack-us-msb.yaml`.
Inventory item 040: `catalog/customer-support-agent.yaml`.
Inventory item 041: `catalog/data-import-wizard.yaml`.
Inventory item 042: `catalog/email-deliverability-pro.yaml`.
Inventory item 043: `catalog/feature-flag-console.yaml`.
Inventory item 044: `catalog/identity-sso-bridge.yaml`.
Inventory item 045: `catalog/invoice-ocr-pipeline.yaml`.
Inventory item 046: `catalog/marketplace-syndicator.yaml`.
Inventory item 047: `catalog/observability-agent.yaml`.
Inventory item 048: `catalog/payment-reconciliation.yaml`.
Inventory item 049: `catalog/policy-simulator.yaml`.
Inventory item 050: `catalog/revenue-share-ledger.yaml`.
Inventory item 051: `catalog/usage-metering-adapter.yaml`.
Inventory item 052: `competitor-parity-matrix.md`.
Inventory item 053: `compliance.md`.
Inventory item 054: `contracts/asyncapi/plugin-app-store-events.yaml`.
Inventory item 055: `contracts/openapi/plugin-app-store.yaml`.
Inventory item 056: `contracts/proto/plugin-app-store.proto`.
Inventory item 057: `cost-budget.md`.
Inventory item 058: `dashboards/catalog-search-dashboard.json`.
Inventory item 059: `dashboards/install-flow-dashboard.json`.
Inventory item 060: `dashboards/vetting-dashboard.json`.
Inventory item 061: `decisions/ADR-PAS-0001-install-time-cedar-materialization.md`.
Inventory item 062: `decisions/ADR-PAS-0002-ordered-vetting-pipeline.md`.
Inventory item 063: `decisions/ADR-PAS-0003-runtime-extension-sandbox.md`.
Inventory item 064: `decisions/ADR-PAS-0004-vetting-badge-tiers-(retired four-label ladder)-determined.md`.
Inventory item 065: `decisions/ADR-PAS-0005-rate-limits-default-deny.md`.
Inventory item 066: `decisions/ADR-PAS-0006-billing-events-owned-by-app-store.md`.
Inventory item 067: `decisions/ADR-PAS-0007-audit-chain-authoritative.md`.
Inventory item 068: `deprecation.md`.
Inventory item 069: `dpia.md`.
Inventory item 070: `evidence/cia-map.md`.
Inventory item 071: `failure-modes.md`.
Inventory item 072: `faqs/marketplace-publisher-faq.md`.
Inventory item 073: `iac/helm/cedar-evaluator/Chart.yaml`.
Inventory item 074: `iac/helm/cedar-evaluator/values.yaml`.
Inventory item 075: `iac/helm/cosign/Chart.yaml`.
Inventory item 076: `iac/helm/cosign/values.yaml`.
Inventory item 077: `iac/helm/leptos-app-shell/Chart.yaml`.
Inventory item 078: `iac/helm/leptos-app-shell/values.yaml`.
Inventory item 079: `iac/helm/plugin-app-store-rest/Chart.yaml`.
Inventory item 080: `iac/helm/plugin-app-store-rest/values.yaml`.
Inventory item 081: `iac/helm/plugin-app-store-worker/Chart.yaml`.
Inventory item 082: `iac/helm/plugin-app-store-worker/values.yaml`.
Inventory item 083: `iac/helm/postgres/Chart.yaml`.
Inventory item 084: `iac/helm/postgres/values.yaml`.
Inventory item 085: `iac/helm/trivy/Chart.yaml`.
Inventory item 086: `iac/helm/trivy/values.yaml`.
Inventory item 087: `iac/helm/valkey/Chart.yaml`.
Inventory item 088: `iac/helm/valkey/values.yaml`.
Inventory item 089: `iac/helm/wasmtime-runtime/Chart.yaml`.
Inventory item 090: `iac/helm/wasmtime-runtime/values.yaml`.
Inventory item 091: `implementation-plans/IP-001-catalog-schema-and-indexing.md`.
Inventory item 092: `implementation-plans/IP-002-publisher-onboarding.md`.
Inventory item 093: `implementation-plans/IP-003-vetting-pipeline.md`.
Inventory item 094: `implementation-plans/IP-004-install-and-entitlement.md`.
Inventory item 095: `implementation-plans/IP-005-runtime-sandbox.md`.
Inventory item 096: `implementation-plans/IP-006-billing-and-revenue-share.md`.
Inventory item 097: `implementation-plans/IP-007-admin-governance-console.md`.
Inventory item 098: `implementation-plans/IP-008-ratings-reviews-and-abuse.md`.
Inventory item 099: `implementation-plans/IP-009-search-relevance-and-recommendations.md`.
Inventory item 100: `implementation-plans/IP-010-api-contract-and-sdk.md`.
Inventory item 101: `implementation-plans/IP-011-migration-from-app-exchange.md`.
Inventory item 102: `implementation-plans/IP-012-observability-and-slos.md`.
Inventory item 103: `implementation-plans/IP-013-security-hardening.md`.
Inventory item 104: `implementation-plans/IP-014-release-readiness.md`.
Inventory item 105: `implementation-plans/IP-015-retirement-and-deprecation.md`.
Inventory item 106: `incident-response.md`.
Inventory item 107: `manifest.json`.
Inventory item 108: `migration-playbooks/from-salesforce-appexchange.md`.
Inventory item 109: `multi-region.md`.
Inventory item 110: `onboarding/marketplace-publisher-first-week.md`.
Inventory item 111: `packs/api-connector-hub/manifest.yaml`.
Inventory item 112: `packs/api-connector-hub/policies/egress.cedar`.
Inventory item 113: `packs/audit-evidence-exporter/manifest.yaml`.
Inventory item 114: `packs/audit-evidence-exporter/policies/read-only.cedar`.
Inventory item 115: `packs/compliance-pack-us-msb/manifest.yaml`.
Inventory item 116: `packs/compliance-pack-us-msb/policies/customer-data.cedar`.
Inventory item 117: `packs/customer-support-agent/manifest.yaml`.
Inventory item 118: `packs/customer-support-agent/policies/ticket-access.cedar`.
Inventory item 119: `packs/usage-metering-adapter/manifest.yaml`.
Inventory item 120: `packs/usage-metering-adapter/policies/meter-write.cedar`.
Inventory item 121: `performance-bench.md`.
Inventory item 122: `policy/admin-override.cedar`.
Inventory item 123: `policy/install-approval.cedar`.
Inventory item 124: `policy/publisher-submit.cedar`.
Inventory item 125: `policy/tenant-scope.cedar`.
Inventory item 126: `reference-implementations/install-listing-programmatically-rust-sdk.md`.
Inventory item 127: `runbooks/catalog-index-backfill.md`.
Inventory item 128: `runbooks/install-rollback.md`.
Inventory item 129: `runbooks/publisher-suspension-appeal.md`.
Inventory item 130: `runbooks/restore-audit-chain.md`.
Inventory item 131: `runbooks/revenue-share-reconciliation.md`.
Inventory item 132: `runbooks/runtime-kill-switch.md`.
Inventory item 133: `runbooks/sbom-policy-hotfix.md`.
Inventory item 134: `runbooks/vetting-backlog-drain.md`.
Inventory item 135: `scorecards/overrides.json`.
Inventory item 136: `sdk-plan.md`.
Inventory item 137: `slos/audit-chain-seal-freshness.openslo.yaml`.
Inventory item 138: `slos/catalog-browse-availability.openslo.yaml`.
Inventory item 139: `slos/catalog-browse-latency.openslo.yaml`.
Inventory item 140: `slos/install-worker-error-rate.openslo.yaml`.
Inventory item 141: `slos/plugin-install-availability.openslo.yaml`.
Inventory item 142: `slos/plugin-install-latency.openslo.yaml`.
Inventory item 143: `slos/plugin-revoke-latency.openslo.yaml`.
Inventory item 144: `slos/runtime-invocation-latency.openslo.yaml`.
Inventory item 145: `slos/vetting-pipeline-throughput.openslo.yaml`.
Inventory item 146: `threat-model.md`.
Inventory item 147: `tutorials/publish-paid-plugin-with-sbom-and-stripe.md`.

## §3 Nine-dimension audit

### §3.1 Dimension 1: product purpose and bounded ownership

Verdict: mostly coherent at the intent layer, but blurred at monetization and marketplace substrate boundaries.
The PRD purpose is crisp: plugin and app distribution for Oyatie tenants and publishers; `PRD.md:23-31` states that the service is not the general B2C commerce marketplace.
The PRD asks for discovery, install, governance, trust, billing, vetting, and audit; `PRD.md:37-44` names those tenant outcomes.
The bounded-context section assigns catalog, publisher onboarding, vetting, entitlement, runtime, billing ledger, ratings, abuse, and governance; `PRD.md:127-166` is broad but understandable.
The architecture binds product objects to substrate identifiers; `ARCHITECTURE.md:197-208` helps separate listing, installation, and runtime binding.
The architecture explicitly links marketplace anchors but calls out Plugin App Store as a separate store-like product surface; `ARCHITECTURE.md:383-391` supports the boundary.
The chat history says the service is a digital plugin category surface, not the entire marketplace; `8f603fc7...jsonl:1249-1252` supports that separation.
The chat history also says `marketplace`, `plugin-app-store`, and social/community surfaces are separate commerce surfaces; `8f603fc7...jsonl:16651-16654` prevents bundling them.
The current docs still over-own billing implementation: PRD context `subscription-billing` owns subscriptions and revenue share at `PRD.md:158-161`.
ADR-PAS-0006 says billing events are owned by the app store and aggregated nightly; `decisions/ADR-PAS-0006-billing-events-owned-by-app-store.md:22-34` makes that ownership explicit.
The manifest lists dependencies on `billing-engine`, `stripe-connect`, `finops-ledger`, and `marketplace-substrate`; `manifest.json:235-245` hints the service should depend on shared systems, not reimplement them.
The tutorial models Stripe revenue share locally; `tutorials/publish-paid-plugin-with-sbom-and-stripe.md:296-298` is a product-flow artifact but lacks tenant-class contract alignment.
Conclusion: product purpose is strong, but billing and revenue-share ownership should be reframed as orchestration over shared billing/substrate systems.

### §3.2 Dimension 2: artifact completeness and inventory health

Verdict: high documentation volume with missing root-level glue and no source/test implementation.
The microservice has PRD and architecture docs; `PRD.md:1-205` and `ARCHITECTURE.md:1-972` are substantial.
The architecture begins with an anchor-sweep warning that all stub sections should be expanded during content-pass review; `ARCHITECTURE.md:3` is an internal quality warning.
There are seven local ADRs; `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:1-34` through `decisions/ADR-PAS-0007-audit-chain-authoritative.md:1-34` cover key decisions.
There are fifteen implementation-plan files; `implementation-plans/IP-001-catalog-schema-and-indexing.md` through `implementation-plans/IP-015-retirement-and-deprecation.md` cover implementation sequencing.
There are contract files in OpenAPI, AsyncAPI, and Proto; `contracts/openapi/plugin-app-store.yaml:1-3`, `contracts/asyncapi/plugin-app-store-events.yaml:1-3`, and `contracts/proto/plugin-app-store.proto:1-4` prove contract intent.
There are nine OpenSLO files; `slos/catalog-browse-latency.openslo.yaml:13-15` and `slos/plugin-install-latency.openslo.yaml:13-15` show example objectives.
There are runbooks for backfill, rollback, suspension, audit-chain restore, revenue share, kill switch, SBOM hotfix, and backlog drain; `runbooks/runtime-kill-switch.md` and `runbooks/vetting-backlog-drain.md` are relevant operational artifacts.
There is no README in the inventory, which leaves the service without a root onboarding and ownership summary.
There is no `cross-microservice-handoffs.md`, even though marketplace-substrate, billing, finops, policy, and developer-sdk boundaries are material.
There is no supported OS manifest, despite the master-plan requirement at `specs/master-plan-sequencing.json:777-816`.
There is no test directory, despite PRD acceptance criteria referencing `tests/load/catalog-search.k6.js` at `PRD.md:110`.
There is no source implementation under `src/`, despite the phase plan requiring cargo check and Rust crates; `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:8-16` defines an implementation exit gate.
Conclusion: the artifact set is broad but not implementation-ready because root onboarding, handoff, OS, source, and test evidence are missing.

### §3.3 Dimension 3: internal contract and SLO coherence

Verdict: contract shape exists, but several contracts encode retired tier language and several SLOs conflict with PRD numbers.
The OpenAPI contract exposes listing search, detail, install, revoke, publisher submission, vetting, and governance endpoints; `contracts/openapi/plugin-app-store.yaml:29-221` covers the main surface.
The OpenAPI listing schema uses `vetting_badge` enum values `tenant_class demo_trial`, `tenant_class paid`, `tenant_class paid`, and `compliance_pack-bound paid`; `contracts/openapi/plugin-app-store.yaml:252-254` is a Wave 15J retirement candidate.
The AsyncAPI event catalog includes install, revoke, vetting, billing, runtime, and subscription-tier events; `contracts/asyncapi/plugin-app-store-events.yaml:48-52` includes the stale tier event.
The AsyncAPI event payload carries `tier_before` and `tier_after`; `contracts/asyncapi/plugin-app-store-events.yaml:174-183` is not compatible with the replacement tenant-class model.
The Proto contract repeats the retired vetting badge ladder; `contracts/proto/plugin-app-store.proto:34-40` is a contract-level retirement candidate.
The Cedar tenant-scope policy permits subscription-tier change only when `plugin_vetting_badge` is in the retired ladder; `policy/tenant-scope.cedar:28-45` ties authorization to retired vocabulary.
The PRD says catalog browse p95 is <=200ms and p99 <=500ms; `PRD.md:70-71` is coherent with `slos/catalog-browse-latency.openslo.yaml:13-15`.
The PRD says catalog browse availability is 99.99%; `PRD.md:78` conflicts with `slos/catalog-browse-availability.openslo.yaml:13-15`, which sets 99.9%.
The PRD says vetting p95 is <=4h and p99 <=24h; `PRD.md:74` conflicts with `slos/vetting-pipeline-throughput.openslo.yaml:13-15`, which targets 95% decided within a 1h window.
The PRD says install p95 <=5s and p99 <=15s; `PRD.md:72` is coherent with `slos/plugin-install-latency.openslo.yaml:13-15`.
The PRD says revoke p99 <=30s; `PRD.md:73` is coherent with `slos/plugin-revoke-latency.openslo.yaml:13-15`.
The OpenAPI pricing schema names `free`, `one_time`, and `recurring`; `contracts/openapi/plugin-app-store.yaml:256-258` does not model `demo_trial`, `paid`, or `revenue_share` tenant classes.
Conclusion: contract existence is strong, but the semantic contract must be scrubbed for retired tier terms and upgraded with tenant-class and billing-model separation.

### §3.4 Dimension 4: canonical-direction alignment

Verdict: this is the largest coherence gap; deployment, OpenTofu, OS, tenant-class, and tenant-class retirement migration alignment are incomplete.
Canonical deployment context coverage requires six contexts: oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, and oyatie-as-cloud-provider; `specs/master-plan-sequencing.json:704-746` is the source.
ADR-0328 D-15 requires per-context mapping and explicit unknowns rather than silent omission; `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2241` is the sequence anchor.
The microservice manifest does not declare `deployment_contexts`; `manifest.json:1-9` starts with service identity and `manifest.json:280-284` jumps to promotion state without context inventory.
The architecture has deployment shape prose but only Kubernetes-style shape, not six-context OpenTofu modules; `ARCHITECTURE.md:663-674` is the deployment-shape section.
OpenTofu is the only canonical IaC engine; `specs/master-plan-sequencing.json:747-776` and `feedback_zero_handroll_opentofu_only_2026_05_20.md:10-18` define that policy.
The service has Helm charts under `iac/helm`, with examples such as `iac/helm/plugin-app-store-rest/Chart.yaml` and `iac/helm/plugin-app-store-worker/Chart.yaml`, but no canonical context directories.
The required context paths are `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/`; `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2315` states the OpenTofu rule set.
The OS manifest requirement is explicit; `feedback_os_support_matrix_2026_05_20.md:56-76` requires per-microservice OS support declaration.
No `supported-oses.json` or equivalent service-local OS manifest appears in the inventory.
The Rust-strict policy is explicit; `feedback_rust_strict_only_no_python_2026_05_20.md:10-18` and `feedback_rust_strict_only_no_python_2026_05_20.md:38-60` define allowed and forbidden implementation languages.
No forbidden source files with `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` extensions were found under the service path.
The PRD and phase plan still cite `.js` k6 commands; `PRD.md:110` and `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:124` are documentation-level Rust-strict/process gaps.
The OCI Always Free profile requires `iac/oci-guest/always-free/`; `feedback_oci_always_free_maximization_2026_05_20.md:65-80` and `specs/master-plan-sequencing.json:857-867` define the path.
No `iac/oci-guest/always-free/` module appears in the inventory.
The no-ladder doctrine says capability-ladder directories are retired; `feedback_no_capability_ladder_2026_05_20.md:10-24` applies to `capability-ladders/tier-matrix.md`.
The batch directive adds `revenue_share` as a tenant class; existing local docs mention revenue share but do not model it as a tenant class.

#### §3.4.T Tier retirement candidates

Tenant-class retirement migration candidate count: 46 direct retired four-label ladder line hits.
Tenant-class retirement migration severity default: P2 documentation and contract cleanup, with contract-level candidates prioritized before tutorial/examples.
Candidate 001: `policy/tenant-scope.cedar:44` allows `tenant_class demo_trial`, `tenant_class paid`, `tenant_class paid`, and `compliance_pack-bound paid` badges in policy logic.
Candidate 002: `migration-playbooks/from-salesforce-appexchange.md:88` says paid billing_components tier.
Candidate 003: `tutorials/publish-paid-plugin-with-sbom-and-stripe.md:15` says tenant_class paid tier or higher.
Candidate 004: `decisions/ADR-PAS-0004-vetting-badge-tiers-(retired four-label ladder)-determined.md:3` puts all four tier names in the ADR title.
Candidate 005: `decisions/ADR-PAS-0004-vetting-badge-tiers-(retired four-label ladder)-determined.md:12` repeats the tier ladder in status context.
Candidate 006: `decisions/ADR-PAS-0004-vetting-badge-tiers-(retired four-label ladder)-determined.md:20` asks how trust verdict is assigned.
Candidate 007: `decisions/ADR-PAS-0004-vetting-badge-tiers-(retired four-label ladder)-determined.md:24` records deterministic trust verdict assignment.
Candidate 008: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:13` says on-prem tenant_class paid tier.
Candidate 009: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:19` says tenant_class paid.
Candidate 010: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:20` says tenant_class paid.
Candidate 011: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:29` says tenant_class paid.
Candidate 012: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:35` says tenant_class paid.
Candidate 013: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:36` says tenant_class paid.
Candidate 014: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:43` says tenant_class paid.
Candidate 015: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:49` says tenant_class paid.
Candidate 016: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:58` says tenant_class paid.
Candidate 017: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:64` says tenant_class paid.
Candidate 018: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:65` says tenant_class paid.
Candidate 019: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:72` says tenant_class paid.
Candidate 020: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:78` says tenant_class paid on-prem.
Candidate 021: `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:95` says compliance_pack-bound paid.
Candidate 022: `contracts/openapi/plugin-app-store.yaml:254` declares `enum: [retired four-label ladder]`.
Candidate 023: `capability-ladders/tier-matrix.md:15` says tenant_class demo_trial.
Candidate 024: `capability-ladders/tier-matrix.md:29` says tenant_class demo_trial.
Candidate 025: `capability-ladders/tier-matrix.md:45` says tenant_class paid.
Candidate 026: `capability-ladders/tier-matrix.md:47` says tenant_class demo_trial.
Candidate 027: `capability-ladders/tier-matrix.md:70` says tenant_class paid.
Candidate 028: `capability-ladders/tier-matrix.md:72` says tenant_class paid.
Candidate 029: `capability-ladders/tier-matrix.md:96` says tenant_class paid.
Candidate 030: `capability-ladders/tier-matrix.md:100` says compliance_pack-bound paid.
Candidate 031: `capability-ladders/tier-matrix.md:102` says tenant_class paid.
Candidate 032: `capability-ladders/tier-matrix.md:113` says tenant_class paid.
Candidate 033: `capability-ladders/tier-matrix.md:127` says retired four-label ladder.
Candidate 034: `capability-ladders/tier-matrix.md:135` says paid billing_components.
Candidate 035: `capability-ladders/tier-matrix.md:136` says tenant_class paid.
Candidate 036: `capability-ladders/tier-matrix.md:137` says compliance_pack-bound paid.
Candidate 037: `capability-ladders/tier-matrix.md:138` says compliance_pack-bound paid.
Candidate 038: `capability-ladders/tier-matrix.md:139` says compliance_pack-bound paid.
Candidate 039: `contracts/proto/plugin-app-store.proto:36` declares `VETTING_BADGE_tenant_class demo_trial`.
Candidate 040: `contracts/proto/plugin-app-store.proto:37` declares `VETTING_BADGE_tenant_class paid`.
Candidate 041: `contracts/proto/plugin-app-store.proto:38` declares `VETTING_BADGE_tenant_class paid`.
Candidate 042: `contracts/proto/plugin-app-store.proto:39` declares `VETTING_BADGE_compliance_pack-bound paid`.
Candidate 043: `faqs/marketplace-publisher-faq.md:14` says paid billing_components.
Candidate 044: `faqs/marketplace-publisher-faq.md:26` says tenant_class paid.
Candidate 045: `faqs/marketplace-publisher-faq.md:30` says compliance_pack-bound paid.
Candidate 046: `faqs/marketplace-publisher-faq.md:99` says compliance_pack-bound paid.
Related non-ladder tier terminology: `PRD.md:64`, `PRD.md:84`, `PRD.md:104`, `contracts/asyncapi/plugin-app-store-events.yaml:51`, `contracts/asyncapi/plugin-app-store-events.yaml:174-183`, `manifest.json:260`, `cost-budget.md:19`, and `cost-budget.md:38` use generic tier/subscription-tier wording that should be reviewed during Wave 15J even when the four names are absent.
False positive note: `canonical_signals` in `manifest.json:232` and `manifest.json:306` is observability vocabulary, not a capability ladder.

#### §3.4.C Tenant-class adoption gaps

Tenant-class adoption verdict: gap present.
No `tenant_class` key or enum appears in the service inventory.
No `demo_trial` literal appears in the service inventory.
No `revenue_share` tenant-class literal appears in service contracts, although revenue-share product and catalog examples exist.
Existing product examples mention revenue share as monetization: `catalog/revenue-share-ledger.yaml` exists in inventory and `tutorials/publish-paid-plugin-with-sbom-and-stripe.md:296-298` describes Stripe revenue-share behavior.
Existing publisher FAQ says default revenue split and negotiations exist; `faqs/marketplace-publisher-faq.md:14` is stale because it couples revenue share to paid billing_components language.
Existing onboarding says publishers choose free, one-time, subscription, usage-based, and mixed free+paid pricing; `onboarding/marketplace-publisher-first-week.md:26` models pricing choices but not tenant classes.
The OpenAPI pricing model has `free`, `one_time`, and `recurring`; `contracts/openapi/plugin-app-store.yaml:256-258` does not separate tenant class from app pricing.
The tenant-class memory requires demo and paid tenant semantics while the current batch directive adds revenue-share; `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-113` gives the old class/billing boundary.
Required correction: add a tenant-class dimension at service boundary and keep plugin pricing, publisher payout, and tenant substrate policy as separate fields.

### §3.5 Dimension 5: deployability and infrastructure readiness

Verdict: not deployable across the six canonical contexts from current artifacts.
The required six contexts are explicit in the master plan; `specs/master-plan-sequencing.json:704-746` names all six.
The ADR requires context-specific IaC directories and operational profile coverage; `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2241` is the deployment-context anchor.
The service only contains Helm charts under `iac/helm`; inventory items 073-090 list the chart files.
The OpenTofu memory says every service should have `iac/` modules per context and missing context is P1 unless explicitly N/A; `feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35` defines that severity.
No file in the service path declares any context as intentionally N/A.
No `iac/guest-on-aws/` module exists.
No `iac/oci-guest/` module exists.
No `iac/oci-guest/always-free/` profile exists.
No `iac/on-prem/` module exists.
No `iac/colo/` module exists.
No `iac/oyatie-public-cloud/` module exists.
No `iac/oyatie-iaas/` module exists; the master plan calls the context `oyatie-as-cloud-provider` while the OpenTofu path uses `oyatie-iaas`.
The architecture describes Kubernetes pods and Cloud Hypervisor/Kata but not context-specific substrate differences; `ARCHITECTURE.md:663-674` is useful but insufficient.
The compliance doc mentions Helm, Kustomize, and OpenTofu together; `compliance.md:790` should be reconciled so OpenTofu is the provisioner and Helm is only workload packaging.
Conclusion: Helm packaging can remain workload packaging, but it cannot substitute for the required OpenTofu context substrate.

### §3.6 Dimension 6: OS and language compliance

Verdict: language-source scan is clean, but OS support and test references are not compliant.
The supported OS matrix includes 13 Tier-1 OS families plus ppc64le and s390x architecture overlays; `specs/master-plan-sequencing.json:777-816` is the master source.
The OS memory requires a per-microservice `supported_oses` manifest and makes absence a P1 gap; `feedback_os_support_matrix_2026_05_20.md:56-76` defines the rule.
No `supported-oses.json` exists in the inventory.
The Rust-strict memory says backend implementation must be Rust and non-Rust backend/source files are forbidden unless explicitly allowlisted; `feedback_rust_strict_only_no_python_2026_05_20.md:10-18` and `feedback_rust_strict_only_no_python_2026_05_20.md:51-60` define that policy.
The source scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under the service path.
The Proto file has a `go_package` option at `contracts/proto/plugin-app-store.proto:8`, which is metadata for generated clients, not a Go source file.
The PRD acceptance criteria still refer to `tests/load/catalog-search.k6.js`; `PRD.md:110` conflicts with Rust-strict validation surfaces unless replaced by a Rust-approved load harness or explicitly tool-exceptioned validation.
The phase plan refers to `k6 run tests/load/install-10k.js`; `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:124` repeats the same problem.
The phase plan requires `cargo check`, `cargo test`, and `cargo nextest`; `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:8-16` is aligned with Rust-strict intent.
No source crates exist in `src/`, so the cargo gates are currently aspirational.
Conclusion: the service should add a supported OS manifest and rewrite JS/k6 test references to compliant Rust or sanctioned external-test language policy.

### §3.7 Dimension 7: operational readiness, SLOs, and failure handling

Verdict: operational docs exist and are useful, but readiness is undermined by missing implementation, contradictory SLOs, and absent context-specific runbooks.
The PRD defines availability, latency, scale, security, and compliance targets; `PRD.md:70-94` is the core non-functional requirement source.
The capacity model covers GA scale targets; `capacity-model.md:12-40` is the sizing anchor.
Failure modes cover catalog stale data, install partial failure, vetting backlog, runtime sandbox escape, billing double charge, abuse, dependency outage, policy rollback, quota exhaustion, and audit-chain seal loss; `failure-modes.md:15-69` is a substantive failure map.
Incident response names Sev-1 cross-tenant leak and financial misallocation above $10k; `incident-response.md:15-21` gives useful severity gates.
The cost budget sets per-install, vetting, runtime, catalog, and support cost targets; `cost-budget.md:17-19` is useful but has tier language at `cost-budget.md:19` and `cost-budget.md:38`.
The DPIA maps data flows, cross-tenant leak mitigation, and cross-border transfers; `dpia.md:19-24`, `dpia.md:31`, and `dpia.md:44` are relevant privacy anchors.
The compliance doc enumerates frameworks and evidence handling; `compliance.md:15-26` and `compliance.md:45-49` are useful compliance anchors.
The runtime-sandbox ADR chooses Wasmtime per tenant-plugin; `decisions/ADR-PAS-0003-runtime-extension-sandbox.md:22-34` is a clear risk-control decision.
The install-time Cedar ADR keeps runtime checks under p99 5ms; `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:22-34` is a useful performance-security tradeoff.
The audit-chain ADR declares audit-chain as authoritative; `decisions/ADR-PAS-0007-audit-chain-authoritative.md:22-34` supports evidentiary operations.
The SLO contradiction between PRD and OpenSLO availability must be fixed before launch claims; `PRD.md:78` versus `slos/catalog-browse-availability.openslo.yaml:13-15`.
The SLO contradiction for vetting throughput must be fixed before staffing or queue sizing claims; `PRD.md:74` versus `slos/vetting-pipeline-throughput.openslo.yaml:13-15`.
Conclusion: operational documentation is above scaffold level, but it must be reconciled and tied to real source, tests, and context runbooks.

### §3.8 Dimension 8: counterpart coverage

Verdict: current docs cover some marketplace concepts but use an outdated counterpart set and miss several union-surface features from Chrome and Shopify.
The current local benchmark file compares Salesforce AppExchange and Atlassian Marketplace; `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:1-13` is off the current top-three counterpart set.
The current competitor matrix names revenue share, automated vetting, and SLO commitments; `competitor-parity-matrix.md:20-22` is useful but not mapped to VS Code, Chrome, and Shopify.
VS Code Marketplace provides in-editor install, VSIX install, workspace recommendations, extension filters, sorting by installs/rating/date, verified publishers, pre-release, platform-specific VSIX packages, and publisher analytics.
Chrome Web Store provides developer dashboard upload, 2GB package limit, 20-extension publisher limit before limit increase, privacy declarations, review process, staged/deferred publish, rollout percentage API for >10k active-user items, user permission warnings, and enforcement.
Shopify App Store provides category discovery, listing review, app billing, revenue share, app ads, Built for Shopify badge, 100-checkpoint review, public app install flows, and merchant review/rating surfaces.
The PRD covers discovery, install, vetting, runtime sandbox, billing, ratings, abuse, and governance; `PRD.md:50-67` covers many counterpart surfaces.
The PRD does not explicitly cover Chrome-style staged rollout percentage or VS Code-style platform-specific packages.
The docs do not express Shopify-style merchant-facing app ads or promotional placements, although marketplace auction journeys appear in `IP-journey-j119-marketplace-auction-surface.md`.
The docs do not model Chrome's publisher account trust time or 2-step verification requirement as first-class publisher controls.
The docs partially model revenue share but conflate it with retired tier language in FAQ and benchmark docs; `faqs/marketplace-publisher-faq.md:14` and `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:95` show that drift.
Conclusion: feature parity should be rebuilt against the current three-counterpart union rather than older Salesforce/Atlassian-centered matrices.

### §3.9 Dimension 9: implementation, verification, and launch readiness

Verdict: not launch-ready; current deliverables can guide implementation after coherence fixes.
The phase plan defines exit gates: schema, API, Rust, OpenAPI, AsyncAPI, evidence, cargo, OPA, and installation drill; `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:8-16` is a good implementation target.
The phase plan scopes five Rust crates and defers parent wiring; `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:50-58` is a clear slice boundary.
The phase plan maps IPs to catalog, publisher, vetting, install, runtime, billing, governance, ratings, search, SDK, migration, observability, security, readiness, and retirement; `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:76-90` is a usable work breakdown.
The acceptance criteria refer to missing test paths; `PRD.md:110-124` and `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:121-126` cannot pass in the current tree.
The capabilities files reference missing eval sets such as `capabilities/eval/plugin-install-canonical.jsonl`; `capabilities/plugin-install.yaml:56-60`, `capabilities/plugin-revoke.yaml:54-55`, and `capabilities/plugin-vetting-decide.yaml:56-57` are eval-evidence gaps.
The architecture declares cell eligibility not yet represented in manifest; `ARCHITECTURE.md:321-332` makes that gap explicit.
The manifest promotion state says seed baseline with next gate parent-service wiring; `manifest.json:280-284` confirms the service is not promoted.
The PRD says "Parent product wiring after contract review" is out of scope; `PRD.md:199` is coherent with current non-promoted state.
The developer-sdk boundary says client SDKs are owned by developer-sdk; `PRD.md:195` prevents this service from owning SDK generator decisions.
The plugin-app-store has enough product substance to implement after canonical repairs, but current source/test/IaC gaps block any claim of deployable readiness.

#### §3.9.A Launch-gate evidence checklist

Launch gate 001: product purpose must remain plugin/app distribution; evidence currently passes through `PRD.md:23-31`.
Launch gate 002: broad marketplace ownership must remain out of this service; evidence currently passes through `PRD.md:193` and chat `8f603fc7...jsonl:1249-1252`.
Launch gate 003: developer SDK generator ownership must remain out of this service; evidence currently passes through `PRD.md:195`.
Launch gate 004: package/catalog API must keep OpenAPI as a generated contract surface; evidence currently exists at `contracts/openapi/plugin-app-store.yaml:1-3`.
Launch gate 005: eventing API must keep AsyncAPI as a generated contract surface; evidence currently exists at `contracts/asyncapi/plugin-app-store-events.yaml:1-3`.
Launch gate 006: internal RPC/API contract must keep Proto as a generated contract surface; evidence currently exists at `contracts/proto/plugin-app-store.proto:1-4`.
Launch gate 007: OpenAPI must remove the four-name vetting enum before launch; blocker evidence is `contracts/openapi/plugin-app-store.yaml:252-254`.
Launch gate 008: AsyncAPI must remove subscription-tier event vocabulary before launch; blocker evidence is `contracts/asyncapi/plugin-app-store-events.yaml:51`.
Launch gate 009: Proto must remove the four-name vetting enum before launch; blocker evidence is `contracts/proto/plugin-app-store.proto:34-40`.
Launch gate 010: Cedar policy must stop authorizing by retired badge names before launch; blocker evidence is `policy/tenant-scope.cedar:44`.
Launch gate 011: PRD acceptance tests must point to existing tests before launch; blocker evidence is `PRD.md:110-124`.
Launch gate 012: phase-plan acceptance tests must point to existing tests before launch; blocker evidence is `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:121-126`.
Launch gate 013: Rust source crates must exist before cargo gates can be used as completion evidence; intended crate boundary is `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:50-58`.
Launch gate 014: capability eval fixtures must exist before autonomy/eval claims can be used; blocker evidence is `capabilities/plugin-install.yaml:56-60`.
Launch gate 015: install eval fixtures must not be the only missing eval set; revoke and vetting have the same issue at `capabilities/plugin-revoke.yaml:54-55` and `capabilities/plugin-vetting-decide.yaml:56-57`.
Launch gate 016: catalog availability target must be single-sourced; conflict evidence is `PRD.md:78` and `slos/catalog-browse-availability.openslo.yaml:13-15`.
Launch gate 017: vetting throughput target must be single-sourced; conflict evidence is `PRD.md:74` and `slos/vetting-pipeline-throughput.openslo.yaml:13-15`.
Launch gate 018: deployment context matrix must exist before deployability claims; required evidence source is `specs/master-plan-sequencing.json:704-746`.
Launch gate 019: OpenTofu context modules must exist before infrastructure readiness claims; required evidence source is `specs/master-plan-sequencing.json:747-776`.
Launch gate 020: OCI Always Free profile must exist before demo_trial infrastructure claims; required evidence source is `specs/master-plan-sequencing.json:857-867`.
Launch gate 021: supported OS manifest must exist before OS-support claims; required evidence source is `feedback_os_support_matrix_2026_05_20.md:56-76`.
Launch gate 022: K8s/Helm packaging must be framed as workload deployment, not canonical IaC substrate; mixed evidence is `compliance.md:790`.
Launch gate 023: Wasmtime runtime sandbox decision is launch-positive if implemented; evidence is `decisions/ADR-PAS-0003-runtime-extension-sandbox.md:22-34`.
Launch gate 024: install-time Cedar materialization is launch-positive if implemented; evidence is `decisions/ADR-PAS-0001-install-time-cedar-materialization.md:22-34`.
Launch gate 025: audit-chain authority is launch-positive if implemented; evidence is `decisions/ADR-PAS-0007-audit-chain-authoritative.md:22-34`.
Launch gate 026: rate-limit default deny is launch-positive if tenant_class overlays are added; evidence is `decisions/ADR-PAS-0005-rate-limits-default-deny.md:22-34`.
Launch gate 027: ordered vetting is launch-positive if SLO contradiction is resolved; evidence is `decisions/ADR-PAS-0002-ordered-vetting-pipeline.md:22-34`.
Launch gate 028: billing-event ownership is launch-risk unless shared billing substrate handoff is documented; evidence is `decisions/ADR-PAS-0006-billing-events-owned-by-app-store.md:22-34`.
Launch gate 029: catalog browse SLO must be reconciled with dashboards; dashboard inventory items 058-060 exist but SLO mismatch remains.
Launch gate 030: incident-response severity must be tied to pager/runbook automation before production; severity evidence is `incident-response.md:15-21`.
Launch gate 031: failure-mode list is launch-positive but needs tests; evidence is `failure-modes.md:15-69`.
Launch gate 032: DPIA is launch-positive but needs contract mapping to data-use declarations; evidence is `dpia.md:19-24`.
Launch gate 033: cross-border handling must be tied to deployment contexts; current anchor is `dpia.md:44`.
Launch gate 034: compliance evidence is launch-positive but must be context-aware; anchor is `compliance.md:15-26`.
Launch gate 035: compliance evidence handling exists but needs CI/admission hooks; anchor is `compliance.md:45-49`.
Launch gate 036: runtime kill-switch runbook exists and should be linked from incident response; inventory item 132 confirms presence.
Launch gate 037: audit-chain restore runbook exists and should be tested against ADR-PAS-0007; inventory item 130 confirms presence.
Launch gate 038: vetting-backlog drain runbook exists and should be aligned with the resolved vetting SLO; inventory item 134 confirms presence.
Launch gate 039: revenue-share reconciliation runbook exists but should move ledger authority to finops/shared billing; inventory item 131 confirms presence.
Launch gate 040: publisher suspension appeal runbook exists and should be linked to review/takedown policy; inventory item 129 confirms presence.
Launch gate 041: local catalog examples are useful for implementation tests; inventory items 032-051 confirm broad catalog fixture coverage.
Launch gate 042: local policy examples are useful for authorization tests; inventory items 122-125 confirm policy files exist.
Launch gate 043: local pack manifests are useful for install-flow tests; inventory items 111-120 confirm sample packs exist.
Launch gate 044: dashboards are useful for observability smoke tests; inventory items 058-060 confirm dashboards exist.
Launch gate 045: scorecard overrides exist but need ownership semantics; inventory item 135 confirms presence.
Launch gate 046: SDK plan exists but must stay in developer-sdk handoff lane; inventory item 136 and `PRD.md:195` define the boundary.
Launch gate 047: Salesforce migration playbook is useful but not a primary counterpart benchmark for this batch; inventory item 108 and current counterpart directive define the correction.
Launch gate 048: tutorial content is useful but must remove stale tier gates; `tutorials/publish-paid-plugin-with-sbom-and-stripe.md:15` is the blocker.
Launch gate 049: FAQ content is useful but must remove stale tier gates; `faqs/marketplace-publisher-faq.md:14` is the blocker.
Launch gate 050: benchmark content is useful only after rebaselining from Salesforce/Atlassian to VS Code/Chrome/Shopify; blocker evidence is `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:1-13`.

#### §3.9.B Evidence-backed remediation sequence

Remediation 001: freeze the product boundary first so plugin-app-store does not absorb broad marketplace ownership; use `PRD.md:23-31` and chat `8f603fc7...jsonl:3`.
Remediation 002: create `cross-microservice-handoffs.md` before editing billing contracts; use dependency evidence at `manifest.json:235-245`.
Remediation 003: map shared marketplace-substrate, billing-engine, finops-ledger, policy, audit-chain, identity, and developer-sdk handoffs in that file.
Remediation 004: remove the four-name badge ladder from OpenAPI, Proto, Cedar, ADR, FAQ, tutorial, benchmark, and capability-ladder docs in one Wave 15J cleanup.
Remediation 005: replace the badge ladder with structured trust/verdict fields so publisher trust remains expressible without feature tiers.
Remediation 006: add `tenant_class` to contracts only after pricing and payout fields are separated.
Remediation 007: add `demo_trial` infrastructure caps through OCI Always Free profile, not through product-feature reduction.
Remediation 008: add `paid` contract semantics for per-seat plus usage-based billing and contractual SLO.
Remediation 009: add `revenue_share` contract semantics for gross-revenue percentage and at-cost substrate accounting.
Remediation 010: rewrite `pluginSubscriptionTierChanged` as billing-plan, entitlement-plan, or payout-arrangement change after domain naming is chosen.
Remediation 011: reconcile catalog availability by choosing 99.99% or 99.9% and updating PRD/OpenSLO together.
Remediation 012: reconcile vetting p95 by choosing the queue objective and updating PRD/OpenSLO/runbook together.
Remediation 013: add supported OS manifest before implementation claims because OS support can affect packaging/runtime test strategy.
Remediation 014: add OpenTofu context directories before claiming deployability in any context.
Remediation 015: add `iac/oci-guest/always-free/` before any demo_trial OCI claim.
Remediation 016: keep Helm charts as workload packaging under OpenTofu, not a substitute for substrate provisioning.
Remediation 017: replace `.js` k6 references with Rust-approved load harnesses or documented external-tool exceptions.
Remediation 018: create Rust crates named by the phase plan before running cargo gates.
Remediation 019: add eval fixtures referenced by capability YAML files before using capability eval results in promotion.
Remediation 020: update counterpart docs to VS Code, Chrome, and Shopify before using feature parity for product planning.
Remediation 021: preserve audit-chain, Cedar, and Wasmtime decisions because they are coherent differentiators.
Remediation 022: verify every correction with file-level diff review and targeted tests before promotion.
Remediation 023: do not add a fourth tier-delta deliverable because the current directive retired it.
Remediation 024: do not create new retired four-label ladder headings, tables, or model rows during cleanup.
Remediation 025: stop when the service has coherent contracts, canonical substrate docs, tenant-class semantics, and executable tests.

## §4 Findings table

| ID | Severity | Finding | Evidence | Required correction |
| --- | --- | --- | --- | --- |
| PAS-COH-001 | P1 | Six deployment contexts are not declared or implemented for this service. | Required by `specs/master-plan-sequencing.json:704-746`; service manifest lacks context declaration at `manifest.json:1-9` and `manifest.json:280-284`; architecture only gives generic deployment shape at `ARCHITECTURE.md:663-674`. | Add explicit context matrix and context modules or explicit N/A rationale for each context. |
| PAS-COH-002 | P1 | OpenTofu canonical substrate is missing; current `iac/` is Helm-only workload packaging. | OpenTofu policy at `specs/master-plan-sequencing.json:747-776`; Helm inventory items 073-090; compliance mixes Helm/Kustomize/OpenTofu at `compliance.md:790`. | Add six canonical OpenTofu context directories and make Helm subordinate workload packaging. |
| PAS-COH-003 | P1 | OCI Always Free profile is absent. | Requirement at `specs/master-plan-sequencing.json:857-867` and `feedback_oci_always_free_maximization_2026_05_20.md:65-80`; no `iac/oci-guest/always-free/` inventory item exists. | Add `iac/oci-guest/always-free/` with demo_trial caps or document an explicit non-applicability rationale. |
| PAS-COH-004 | P1 | Supported OS manifest is absent. | Requirement at `specs/master-plan-sequencing.json:777-816` and `feedback_os_support_matrix_2026_05_20.md:56-76`; no supported-OS inventory item exists. | Add `supported-oses.json` with support, test, and exception status for the canonical matrix. |
| PAS-COH-005 | P1 | Source and test implementation are missing despite implementation gates. | Phase exit gates at `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:8-16`; missing tests conflict with `PRD.md:110-124`. | Land Rust crates and compliant tests before claiming implementation readiness. |
| PAS-COH-006 | P1 | Product billing ownership is over-localized relative to shared marketplace and billing substrate direction. | PRD billing context at `PRD.md:158-161`; ADR ownership at `decisions/ADR-PAS-0006-billing-events-owned-by-app-store.md:22-34`; dependency hints at `manifest.json:235-245`; chat sequencing at `8f603fc7...jsonl:776`. | Reframe app store as publisher/install orchestration over shared billing, marketplace-substrate, and finops systems. |
| PAS-COH-007 | P1 | Acceptance tests cite forbidden or missing JS/k6 paths. | `PRD.md:110` and `PHASE-01-PLUGIN-APP-STORE-SUBSTRATE.md:124`; Rust policy at `feedback_rust_strict_only_no_python_2026_05_20.md:10-18`. | Replace with compliant Rust load/integration harnesses or formal exception documentation. |
| PAS-COH-008 | P1 | Current counterpart docs are off target for the required top-three union. | Existing benchmark file is Salesforce/Atlassian at `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:1-13`; chat confirms VS Code/Chrome/Shopify at `8f603fc7...jsonl:16290`. | Replace older counterpart matrix with the current three-counterpart union surface. |
| PAS-COH-009 | P2 | retired four-label ladder terms remain in service contracts and policy. | OpenAPI enum at `contracts/openapi/plugin-app-store.yaml:254`; Proto enum at `contracts/proto/plugin-app-store.proto:36-39`; Cedar policy at `policy/tenant-scope.cedar:44`. | Retire the four-name ladder and replace with trust/verdict and tenant-class-neutral semantics. |
| PAS-COH-010 | P2 | The `capability-ladders/` directory is a direct Wave 15J retirement candidate. | No-ladder doctrine at `feedback_no_capability_ladder_2026_05_20.md:10-24`; local file `capability-ladders/tier-matrix.md:127` carries the four-name ladder. | Remove or quarantine the directory in the Wave 15J scrub; do not build new work on it. |
| PAS-COH-011 | P2 | Tenant-class semantics are absent from contracts and manifest. | OpenAPI pricing model at `contracts/openapi/plugin-app-store.yaml:256-258`; manifest lacks tenant class near identity and promotion keys at `manifest.json:1-9` and `manifest.json:280-284`; tenant-class direction at `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-113`. | Add `tenant_class` to tenant/account policy surfaces and keep it separate from plugin pricing. |
| PAS-COH-012 | P2 | Catalog availability target conflicts between PRD and OpenSLO. | PRD target `PRD.md:78`; OpenSLO target `slos/catalog-browse-availability.openslo.yaml:13-15`. | Choose one authoritative target and update PRD/OpenSLO together. |
| PAS-COH-013 | P2 | Vetting latency/throughput target conflicts between PRD and OpenSLO. | PRD target `PRD.md:74`; OpenSLO target `slos/vetting-pipeline-throughput.openslo.yaml:13-15`. | Decide whether p95 is 1h or 4h and update queue, staffing, and SLO documents. |
| PAS-COH-014 | P2 | Eval-set paths exist but backing eval files are absent from inventory. | Capability eval paths at `capabilities/plugin-install.yaml:56-60`, `capabilities/plugin-revoke.yaml:54-55`, and `capabilities/plugin-vetting-decide.yaml:56-57`. | Add eval fixtures or remove evaluation claims until fixtures land. |
| PAS-COH-015 | P2 | README is absent. | Requested artifact list includes README; inventory has no `README.md`; root purpose exists only in `PRD.md:23-31`. | Add README with ownership, contracts, context matrix, build/test commands, and handoff map. |
| PAS-COH-016 | P2 | Cross-microservice handoff document is absent. | Ownership memory requires contradiction/handoff review at `feedback_microservice_ownership_coherence_2026_05_20.md:18-45`; inventory has no `cross-microservice-handoffs.md`. | Add handoff doc for marketplace-substrate, billing, finops, policy, developer-sdk, identity, and audit-chain. |
| PAS-COH-017 | P2 | Architecture marks itself as anchor-sweep generated and needs content-pass validation. | `ARCHITECTURE.md:3`. | Run a content pass that keeps only verified architecture and deletes scaffold residue. |
| PAS-COH-018 | P2 | Cell eligibility is not declared in manifest. | Architecture states missing manifest declaration at `ARCHITECTURE.md:321-332`. | Add manifest cell eligibility and context interaction rules. |
| PAS-COH-019 | P2 | Generic subscription-tier event vocabulary remains after tier retirement. | AsyncAPI event name at `contracts/asyncapi/plugin-app-store-events.yaml:51`; payload fields at `contracts/asyncapi/plugin-app-store-events.yaml:174-183`; manifest audit event at `manifest.json:260`. | Rename events around plan, billing component, or entitlement change without tier semantics. |
| PAS-COH-020 | P2 | Cost budget uses stale tier wording. | `cost-budget.md:19` and `cost-budget.md:38`. | Rewrite cost controls around tenant_class and usage caps. |
| PAS-COH-021 | P3 | Proto carries `go_package` metadata despite Rust-strict backend policy. | `contracts/proto/plugin-app-store.proto:8`. | Keep only if generator policy explicitly permits client metadata; otherwise move generator-specific options to SDK-owned config. |
| PAS-COH-022 | P3 | Marketplace and publisher docs include useful revenue-share ideas but mix them with stale tier language. | `faqs/marketplace-publisher-faq.md:14`, `onboarding/marketplace-publisher-first-week.md:26`, and `tutorials/publish-paid-plugin-with-sbom-and-stripe.md:296-298`. | Preserve revenue-share flows while deleting tier gates. |
| PAS-COH-023 | P3 | Existing local performance benchmark is partly useful but obsolete for this batch's counterpart set. | `performance-bench.md:36-40` and `benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:1-13`. | Keep any durable metric only after re-sourcing against VS Code, Chrome, and Shopify. |
| PAS-COH-024 | P3 | Developer SDK boundary is correctly assigned away from this service but should be repeated in handoff docs. | `PRD.md:195`; developer-sdk memory is scoped only to developer-sdk at `feedback_developer_sdk_stainless_generator_2026_05_20.md:1-20`. | Add cross-service handoff so plugin-app-store does not own generator policy. |

## §5 Open questions

Open question 001: should `vetting_badge` survive as a trust/verdict signal after the four-name ladder is removed, or should it be replaced by structured evidence scores?
Open question 002: should `revenue_share` be a tenant class on the consuming Oyatie tenant, a billing arrangement on the publisher, or both with separate fields?
Open question 003: should publisher monetization be owned entirely by shared marketplace/billing substrates, with plugin-app-store only storing listing-visible pricing metadata?
Open question 004: should Chrome-style staged rollout percentages be required for plugin installs and updates before GA?
Open question 005: should VS Code-style platform-specific packages be first-class for plugins that ship native binaries?
Open question 006: should Shopify-style app ads be in plugin-app-store or in shared marketplace search/auction surfaces?
Open question 007: should the OCI Always Free profile permit runtime sandbox invocation, or only catalog browse and install dry-runs for demo_trial tenants?
Open question 008: should on-prem and colo require local signing/key transparency mirrors for plugin packages before install is allowed?
Open question 009: should the service expose tenant-class at API boundary, or should tenant class be injected only through identity/entitlement context?
Open question 010: should audit-chain restore be owned by plugin-app-store runbooks or by a platform audit-chain service with plugin-specific procedures?
Open question 011: should old Salesforce/AppExchange migration docs stay as migration-playbook references after the current three-counterpart feature bar is added?
Open question 012: should source implementation begin with catalog/search or with install entitlement, given the current highest-risk gaps are contract semantics and substrate readiness?

### §5.1 Decision-ready clarification backlog

Clarification 001: decide whether `trusted_publisher_status` is a publisher property, a listing property, or both.
Clarification 002: decide whether `vetting_verdict` is immutable per package version or recomputed when scanner policy changes.
Clarification 003: decide whether scanner-policy changes can quarantine already-installed plugin versions without a new package upload.
Clarification 004: decide whether installation rollback is tenant-local, region-local, or globally coordinated.
Clarification 005: decide whether `demo_trial` tenants can publish public plugins or only install/test private listings.
Clarification 006: decide whether `revenue_share` tenants are allowed on on-prem and colo contexts where gross-revenue telemetry may be harder to verify.
Clarification 007: decide whether gross-revenue reporting is publisher-provided, platform-observed, or both.
Clarification 008: decide whether revenue-share ledger events are plugin-app-store events or finops-ledger events.
Clarification 009: decide whether app-store ad placements are owned by plugin-app-store or shared marketplace search/auction.
Clarification 010: decide whether sponsored placements are allowed for plugins with elevated data permissions.
Clarification 011: decide whether package size maximum is Chrome-parity 2GB or lower for Oyatie security/cost reasons.
Clarification 012: decide whether staged rollout can begin below Chrome's 10,000-active-user threshold because Oyatie tenants need early controlled rollout.
Clarification 013: decide whether failed manual review can be appealed to plugin-app-store or to a shared trust/safety process.
Clarification 014: decide whether `supported-oses.json` should live at microservice root or under `specs/` once the empty `specs/` directory is used.
Clarification 015: decide whether WebAssembly/plugin runtime compatibility is declared per package version or per listing.
Clarification 016: decide whether on-prem installs require a local package mirror before production use.
Clarification 017: decide whether colo installs require Oyatie-operated signing infrastructure inside the colo boundary.
Clarification 018: decide whether plugin reviews and ratings are tenant-local, global, or both.
Clarification 019: decide whether abuse reports are visible to publishers before investigation closes.
Clarification 020: decide whether audit-chain event names must be stable before source implementation starts.

### §5.2 Closed assumptions used by this audit

Closed assumption 001: the service remains a plugin/app store, not the broad marketplace, because `PRD.md:23-31` and chat `8f603fc7...jsonl:3` agree.
Closed assumption 002: all six deployment contexts remain in scope because no service-local N/A rationale was found and `specs/master-plan-sequencing.json:704-746` requires them.
Closed assumption 003: OpenTofu is required because `specs/master-plan-sequencing.json:747-776` and `feedback_zero_handroll_opentofu_only_2026_05_20.md:10-18` agree.
Closed assumption 004: Helm charts can remain only as workload packaging because canonical IaC is OpenTofu.
Closed assumption 005: OS support must be explicit because `feedback_os_support_matrix_2026_05_20.md:56-76` requires a per-microservice manifest.
Closed assumption 006: Rust is the backend implementation language because `feedback_rust_strict_only_no_python_2026_05_20.md:10-18` is explicit.
Closed assumption 007: `.js` test commands are not acceptable as unstated implementation/test surfaces under the current Rust-strict doctrine.
Closed assumption 008: retired four-label ladder lines are retirement candidates, not design inputs.
Closed assumption 009: `capability-ladders/tier-matrix.md` is a retirement candidate because the no-ladder memory retires that corpus.
Closed assumption 010: `demo_trial`, `paid`, and `revenue_share` are the working tenant classes because the current user directive is newer than the two-class memory.
Closed assumption 011: the top-three counterpart bar is VS Code Marketplace, Chrome Web Store, and Shopify App Store because the current task and chat `8f603fc7...jsonl:16290` agree.
Closed assumption 012: public counterpart performance numbers are incomplete, so the benchmark report must label official source numbers separately from estimates.
Closed assumption 013: no commit should be made because the execution rules explicitly say no commits.
Closed assumption 014: no other microservice should be touched because the execution rules limit writes to plugin-app-store.
Closed assumption 015: the fourth capability-ladder delta deliverable is retired and intentionally absent.

<!-- ORCHESTRATOR REPORT
  µservice: plugin-app-store
  deliverables_landed:
    - microservices/plugin-app-store/coherence-audit-2026-05-20.md (606 lines)
    - microservices/plugin-app-store/feature-parity-matrix-2026-05-20.md (411 lines)
    - microservices/plugin-app-store/performance-benchmark-numbers-2026-05-20.md (317 lines)
  inventory_files_seen: 147
  inventory_lines_read: 19533
  chat_history_matches_processed: 12
  findings_p0: 0
  findings_p1: 8
  findings_p2: 12
  findings_p3: 4
  tier_retirement_candidates_found: 46; citations: policy/tenant-scope.cedar:44; migration-playbooks/from-salesforce-appexchange.md:88; tutorials/publish-paid-plugin-with-sbom-and-stripe.md:15; decisions/ADR-PAS-0004-vetting-badge-tiers-(retired four-label ladder)-determined.md:3,12,20,24; benchmarks/salesforce-appexchange-vs-atlassian-marketplace-vs-oyatie.md:13,19,20,29,35,36,43,49,58,64,65,72,78,95; contracts/openapi/plugin-app-store.yaml:254; capability-ladders/tier-matrix.md:15,29,45,47,70,72,96,100,102,113,127,135,136,137,138,139; contracts/proto/plugin-app-store.proto:36,37,38,39; faqs/marketplace-publisher-faq.md:14,26,30,99
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share contract semantics found, while pricing and revenue-share examples exist separately
  top_3_counterparts_confirmed: VS Code Marketplace / Chrome Web Store / Shopify App Store
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1334
-->
