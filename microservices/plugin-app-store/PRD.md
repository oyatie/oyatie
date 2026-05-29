---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-plugin-app-store
microservice: plugin-app-store
status: Accepted
sales_segment: hero-product
tier: external-facing
milestone_first_ship: M04-ecosystem-substrate
bominal_source:
  - ADR-0037   # Plugin substrate (superseded for new work by ADR-0213)
  - ADR-0028   # Audit chain
  - ADR-0022   # Autonomy tiers
related_adrs: [ADR-0001, ADR-0007, ADR-0008, ADR-0056, ADR-0065, ADR-0105, ADR-0110, ADR-0123, ADR-0131, ADR-0132, ADR-0139, ADR-0147, ADR-0181, ADR-0200, ADR-0211, ADR-0212, ADR-0213, ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, ADR-0345]
related_specs: [/specs/microservices/plugin-app-store.json, /specs/per-microservice-flat-layout.json]
related_unbundle_adr: ADR-0213
unbundle_sibling: microservices/developer-sdk/
date: 2026-05-18
owner_team: axis-ecosystem + council-architecture
doc_status: published
---

# PRD-plugin-app-store: Plugin/App Store — third-party developer plugin catalog + install + vetting + billing

## Purpose

The `plugin-app-store` µservice is oyatie's **third-party developer plugin/app distribution surface** — the consumer + governance half of the Ecosystem-as-a-Service product per ADR-0213. The sibling `developer-sdk` µservice owns the developer-facing surface (SDK + portal + sandbox + payout). This µservice owns: the public plugin catalog, the per-tenant install flow, the per-plugin permission grant, the plugin version state machine, the deterministic vetting pipeline (Cosign + Trivy + Wasmtime + Cedar + WCAG + AI-Act + perf), the per-installation rate-limit substrate, the per-plugin subscription + billing aggregation feeding finops-portal, and the per-plugin action audit trail.

This µservice is **NOT** a B2C commerce marketplace. Per ADR-0213 §Disambiguation, the substring `marketplace` in oyatie µservice names is reserved for a future B2C commerce µservice (Amazon/Shopify class). This µservice handles plugin distribution (Apple App Store / VS Code Marketplace / AWS Marketplace SaaS catalog / JetBrains Marketplace shape) ONLY.

The hero-product framing: tenants in 2026 evaluate platforms on the strength of their plugin ecosystems. WeChat won the China super-app market on mini-program count + quality. Apple sustains a $20B+/yr developer payout. Stripe became indispensable partly via the Stripe Apps marketplace. Shopify's app store is a leading edge of their developer-platform moat. Oyatie's plugin-app-store is the substrate that makes oyatie a **platform** rather than a **product**.

This µservice operates at the **application** layer of the 12-layer Workflow + Ontology architecture per `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`: plugin-app-store consumes ontology object-type descriptors for typed plugin manifests; emits per-plugin action events to the workflow-engine event bus; bridges to audit-chain for seal events; routes through tenancy for per-tenant install + identity for permission grants; runs in its own composition-root `app` crate.

## Tenant Value

- **Tenant Outcome 1 — Discovery in under 60 seconds.** Tenant operator opens the plugin store, filters by category + capability requested + price + rating, opens detail page. Search results return in < 200ms p95. n8n / Zapier app-directory parity.
- **Tenant Outcome 2 — One-click install with informed consent.** Apple-style permission grant modal lists every capability the plugin will exercise on tenant data. Tenant grants or denies; install completes within 5 seconds p95.
- **Tenant Outcome 3 — Plugin governance for tenant admins.** Per-plugin install policy ("only plugins with vetting badge X"; "only plugins from publisher Y"; "no PII-touching plugins"); per-plugin uninstall + audit trail; per-plugin spend cap.
- **Tenant Outcome 4 — Per-plugin trust signals.** Every plugin shows: vetting status badge, last-vetted date, declared capabilities, declared data classes, install count, average rating, publisher identity (KYC-verified). No trust-by-vibes.
- **Tenant Outcome 5 — Per-plugin rate limit + circuit breaker.** Misbehaving plugin cannot DoS tenant; per-installation rate limit (100 req/s default, admin-overridable); circuit breaker on declared-capability error rate > 5%; auto-suspension at 50% error rate sustained 5 min.
- **Tenant Outcome 6 — Per-plugin billing reconciliation.** Tenant sees one consolidated finops-portal invoice with per-plugin line items; per-seat or flat monthly; cancellation with proration; no surprises.
- **Internal Outcome 7 — Vetting pipeline is auditable + repeatable.** Every submitted plugin transit produces a forensic trail; rejection reasons are structured + machine-readable for the developer-sdk dev portal to surface.
- **Internal Outcome 8 — Plugin-action audit trail seals via audit-chain.** Every plugin action on tenant data emits a seal event; tenant can produce a full forensic trail for compliance + dispute resolution.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | tenant operator | to browse the public plugin catalog | I can discover plugins | plugin-catalog | Must |
| FR-02 | tenant operator | to search + filter by category + capability + price + rating | I find relevant plugins fast | plugin-catalog | Must |
| FR-03 | tenant operator | to view per-plugin detail (description, screenshots, permissions, pricing, ratings, vetting badge) | I make an informed install decision | plugin-catalog | Must |
| FR-04 | tenant operator | to install a plugin on my tenant | the plugin runs in my tenant context | plugin-install | Must |
| FR-05 | tenant operator | to grant or deny each capability the plugin requests at install time | I control what the plugin can do | plugin-install + per-plugin-permissions | Must |
| FR-06 | tenant admin | to set per-plugin install policy (allowed publishers, required vetting tiers, forbidden capabilities) | tenant operators only install policy-conformant plugins | per-plugin-permissions | Must |
| FR-07 | tenant operator | to uninstall a plugin | it stops running in my tenant context | plugin-install | Must |
| FR-08 | plugin developer (via developer-sdk) | to submit a new plugin version for vetting | I publish to the catalog | plugin-lifecycle + vetting-pipeline | Must |
| FR-09 | vetting-pipeline | to verify Cosign signature, scan with Trivy, validate Wasmtime isolation, check capability scope, check data-use boundary, check WCAG, classify AI-Act risk, verify perf budget | only safe plugins are published | vetting-pipeline | Must |
| FR-10 | vetting reviewer | to manually approve / reject the synthesized vetting report | human review on the auto report | vetting-pipeline | Must |
| FR-11 | platform admin | to revoke a malicious plugin (kill-switch all installations) | tenant safety in incident response | plugin-lifecycle | Must |
| FR-12 | tenant operator | to view per-plugin audit trail (every action it took on my tenant data) | I have forensic visibility | audit-stream | Must |
| FR-13 | tenant admin | to set per-plugin rate-limit overrides (allow higher quota for trusted plugins) | trusted plugins are not throttled inappropriately | per-plugin-rate-limit | Should |
| FR-14 | finops-portal | to receive per-plugin billing events for invoice aggregation | tenant gets one consolidated invoice | subscription-billing | Must |
| FR-15 | tenant operator | to manage subscriptions (pause, resume, cancel, change billing_components) | I control my spend | subscription-billing | Must |
| FR-16 | plugin runtime | to be denied + audit-logged when it attempts an action outside its declared capabilities | declared-capability scope is enforced | per-plugin-permissions | Must |
| FR-17 | platform admin | to view vetting-queue health (submissions / day, p95 vetting-decision latency, rejection rate by cause) | I tune the vetting pipeline | vetting-pipeline | Should |

## Non-functional Requirements

### Performance
- Plugin catalog search p95 ≤ 200ms; p99 ≤ 500ms (cold + warm).
- Plugin install p95 ≤ 5s; p99 ≤ 15s (includes Cedar policy materialization + Wasmtime engine pre-warm).
- Plugin revoke p99 ≤ 30s (kill-switch propagation to all installations across cells).
- Vetting pipeline p95 ≤ 4 hours; p99 ≤ 24 hours; SLA ≤ 5 business days.
- Per-plugin runtime cold-start p99 ≤ 300ms (Wasmtime pre-cached engine).

### Availability
- Plugin catalog browse: 99.99% (no single tenant action depends on plugin install latency on the read path).
- Plugin install flow: 99.95% (write path; degrades to retry).
- Vetting pipeline: 99% (async; per-stage retry).
- Per-plugin runtime: 99.9% (varies by published plugin SLO declaration).

### Scalability
- 100k plugins in catalog at GA; 1M plugins at hyperscaler tier.
- 10k concurrent installs at peak.
- 1M tenant-plugin installations active per region.
- Per-plugin sandbox: 100 concurrent invocations per installation default; admin-overridable.

### Security
- Cosign signature verification mandatory on every install.
- Wasmtime sandbox isolation enforced; no syscall escapes (per ADR-0147 baseline).
- Per-installation Cedar policy materialized at install time; runtime authorization through central evaluator.
- Per-plugin audit trail seals via audit-chain (ADR-0003); chain integrity verified daily.
- Vetting pipeline rejection on any of: signature failure, Trivy CVE Critical+ unfixed, Wasmtime syscall escape, capability-scope mismatch, data-use-boundary violation, WCAG fail, AI-Act misclassification, perf budget breach.

### Compliance
- GDPR Article 28 data processor agreement with developer at onboarding (developer-sdk).
- Per-plugin data-use boundary enforced; cross-boundary access denied + audit-logged.
- Per-pack overlays for plugin availability (some plugins not available in pack-kr / pack-eu / pack-us-healthcare; per-pack vetting + locale).
- AI-Act risk class declared per plugin if AI capabilities consumed.

### Cost
- Vetting pipeline budget: $0.50 per submitted version (Trivy + Cosign + Wasmtime ephemeral run).
- Per-plugin runtime cost passed through to plugin's billing_components contract (free plugins on shared cells; paid plugins on dedicated cells per ADR-0199).

### DR posture (ADR-0343)

- RTO/RPO target: manifest-declared RTO p99 900s and RPO p99 60s for catalog, install grants, vetting decisions, and subscription-billing events. Applicable floors are EU-AI-ACT-2024-HIGH-RISK 1800s/300s with multi-region required for AI plugins, SOC2-T2 14400s/900s, and ISO27001-2022 14400s/3600s; the manifest target is stricter than those floors.
- Failover reference: manifest `failover_runbook` is `runbooks/dr-failover.md`; supporting drills remain `microservices/plugin-app-store/multi-region.md`, `runbooks/plugin-revoke-propagation-slow.md`, `runbooks/vetting-pipeline-stuck.md`, and `runbooks/wasmtime-sandbox-escape-suspected.md`.
- Multi-region active-active posture: true per manifest; replication shape is `active-active-multi-az-cross-region-warm` across `postgres_wal_g`, versioned object storage, Valkey, and audit-chain Merkle seals.
- Tenant-visible behavior: tenants can still browse catalog entries during a regional event, while install and revoke actions either complete once with a sealed audit row or enter a visible retry state.

### Capacity model (ADR-0340)

- Per-tenant baseline: manifest-declared 0.25 vCPU, 512 MiB RAM, 2 GB storage, three Postgres connections, four Valkey connections, and eight outbound HTTP connections.
- Scaling dimension: `per_capability` per manifest for install, vetting, signing, permission checks, and Wasmtime admission, with `per_request` for catalog/search and `per_plugin_invocation` for runtime accounting as secondary dimensions.
- Cell placement class: Tier-2 capability cell for catalog, install, vetting, and billing state; Tier-0 pod runtime is reserved for tenant plugin execution sandboxes declared by ADR-0338.
- Autoscaling boundaries: catalog and install APIs scale 3-100 replicas per active region, vetting workers scale 2-50 per queue, and Tier-0 sandbox engines scale from 100 to 1000 warm engines per cell with per-installation concurrency caps.
- Tenant load profile: serves 100k catalog plugins, 10k concurrent installs, and 1M active tenant-plugin installations while keeping kill-switch propagation under the plugin revoke SLO.

### Sustainability and cost attribution (ADR-0344)

- Per-call emission claim: catalog search, install, revoke, vetting stage, runtime invocation, and subscription-billing audit rows emit `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with tenant, plugin, provider, cell, and compliance_pack axes.
- Carbon-aware provider routing: yes for vetting batches, catalog indexing, and non-urgent package replication; no for plugin revoke, kill-switch propagation, sandbox-escape response, or tenant install consent writes.
- Tenant transparency surface: plugin admin and finops-portal show per-plugin catalog, vetting, runtime, and subscription line items alongside the consolidated invoice.
- Regulatory driver: CSRD, SB-253, and SEC climate disclosure reporting need plugin ecosystem emissions separated from core platform cost, especially when third-party plugins drive tenant spend.

### API versioning posture (ADR-0342)

- Public API version model: `YYYY-MM-DD` carrier triplet across version header, URL prefix, and proto3 field for catalog, install, permission, vetting, rating, and billing contracts.
- SDK semver model: plugin marketplace SDKs use `major.minor.patch`; breaking catalog or install contract changes require a major SDK bump plus a new date-versioned public API.
- Support window: last 3 public versions are supported for at least 180 days.
- Per-tenant pinning: yes for plugin manifests, install APIs, permission grants, and tenant admin automations.
- Internal-mesh exemption: yes; ADR-0145 direct gRPC remains valid between catalog, vetting, billing, and runtime-adjacent internal components.

## Acceptance Criteria (AC)

| ID | Criterion | Test |
|---|---|---|
| AC-01 | Catalog search returns ≤ 200ms p95 | `tests/load/catalog-search.k6.js` p95 measured |
| AC-02 | Install completes ≤ 5s p95 including Cedar materialization | `tests/e2e/install-flow.rs` measured |
| AC-03 | Revoke propagates to all installations ≤ 30s p99 | `tests/e2e/revoke-propagation.rs` measured |
| AC-04 | Vetting pipeline fails open (rejects) on Cosign signature absent | unit test in `vetting-pipeline-domain` |
| AC-05 | Vetting pipeline rejects Trivy CVE-Critical-unfixed | unit test in `vetting-pipeline-domain` |
| AC-06 | Per-plugin Cedar policy denies out-of-scope action + audit-logs | integration test |
| AC-07 | Per-plugin rate-limit enforces declared cap with no leakage | integration test against Valkey |
| AC-08 | Per-plugin runtime cold-start ≤ 300ms p99 | bench test |
| AC-09 | Audit-chain seal emitted for every plugin action on tenant data | integration test with audit-chain stub |
| AC-10 | Billing aggregation feeds finops-portal with byte-equal totals | integration test with finops-portal stub |
| AC-11 | Plugin-lifecycle state machine refuses invalid transitions | unit test in `plugin-lifecycle-domain` |
| AC-12 | Kill-switch revokes plugin across all tenants ≤ 30s p99 with no data loss | scripted e2e drill |
| AC-13 | Per-pack overlay denies plugin in pack-kr if developer did not declare KR pack support | integration test per pack |
| AC-14 | Deterministic-replay: vetting decision reproduces byte-equally on identical input | replay test |

## Bounded Contexts

### plugin-catalog
- **Description**: Public catalog of published plugins; browse + search + category tree + ratings; read-heavy.
- **Crates**: per manifest.json `bounded_contexts[0].crates` (9 crates kernel→app).
- **Storage**: Postgres for catalog records; full-text index via Postgres `tsvector` initially, ClickHouse for ratings aggregation at scale.
- **Read replicas**: catalog is read-heavy; 3+ replicas per region; cached at Cilium L4 gateway.

### plugin-install
- **Description**: Tenant-scoped install + uninstall + per-plugin permission grant capture.
- **Crates**: per manifest.json (9 crates).
- **Storage**: Postgres for installation records; Valkey for in-flight install state.

### plugin-lifecycle
- **Description**: Version state machine: draft → submitted → vetting → published → deprecated → retired + revoked kill-switch.
- **Crates**: per manifest.json (6 crates).
- **Storage**: Postgres event-sourced lifecycle log.

### vetting-pipeline
- **Description**: Deterministic 8-stage pipeline (Cosign → Trivy → Wasmtime-isolation → capability-scope → data-use-boundary → WCAG → AI-Act class → perf budget).
- **Crates**: per manifest.json (7 crates).
- **Storage**: Postgres for vetting decisions; S3 (SeaweedFS) for vetting artifact archive.

### per-plugin-permissions
- **Description**: Per-plugin Cedar policy fragment generation + install-time grant capture.
- **Crates**: per manifest.json (4 crates).
- **Substrate**: Cedar 4.x via adapter; central evaluator owned by governance µservice.

### per-plugin-rate-limit
- **Description**: Per-installation rate-limit enforcement (default 100 req/s; per-plugin override).
- **Crates**: per manifest.json (4 crates).
- **Substrate**: Valkey-backed token bucket; 200-byte-per-installation footprint.

### subscription-billing
- **Description**: Per-plugin subscription state machine + tenant billing aggregation feeding finops-portal.
- **Crates**: per manifest.json (5 crates).
- **Storage**: Postgres event-sourced subscription log + nightly aggregator job.

### audit-stream
- **Description**: Per-plugin action audit trail; every action seals via audit-chain µservice.
- **Crates**: per manifest.json (3 crates).
- **Substrate**: emits to audit-chain µservice's outbox.

## Persona Map

| Persona | Surface | Capabilities | Primary BC |
|---|---|---|---|
| tenant-operator | Leptos web UI | browse, install, uninstall, configure, view audit trail | plugin-catalog + plugin-install + audit-stream |
| tenant-admin | Leptos web UI | set install policy, override rate limit, cap spend, revoke install | per-plugin-permissions + per-plugin-rate-limit + subscription-billing |
| platform-admin | Backstage admin | review vetting queue, revoke malicious plugins, view vetting metrics | vetting-pipeline + plugin-lifecycle |
| plugin-developer | dev portal (developer-sdk) | submit version, view vetting status, view installs + revenue | plugin-lifecycle (write) + subscription-billing (read) |
| ai-agent (T2/T3) | REST / SDK | install on behalf of human-approved tenant; constrained by Cedar | plugin-install (constrained) |

## Cross-product integration

**Workflow + Ontology routing only.** plugin-app-store imports nothing from other product µservices' crates. All cross-product data flow:
- **To audit-chain**: per-plugin action seal events via event-bus.
- **To finops-portal**: billing aggregation events via event-bus.
- **To workflow-engine**: plugin lifecycle events via event-bus.
- **To ontology**: Plugin, PluginVersion, PluginInstallation, PluginSubscription projections.
- **From identity**: principal-id resolution.
- **From tenancy**: tenant-id resolution.
- **From governance**: Cedar policy evaluation.
- **From cloud-secrets**: OpenBao signing key resolution for signature verification.
- **From developer-sdk**: plugin submission events.

## Out of scope

- B2C commerce marketplace (Amazon/Shopify class). RESERVED for future `marketplace` µservice; out of scope.
- LinkedIn-class job/social surface. RESERVED for `community` µservice expansion; out of scope.
- Plugin development tools (SDK, codegen, portal). OWNED by sibling `developer-sdk` µservice.
- Cross-tenant plugin marketplace (e.g., one tenant publishes a plugin only their partner tenants see). Future ADR.
- Plugin-to-plugin direct messaging. Forbidden; plugins communicate only via Workflow + Ontology adapter layer.

## References

- ADR-0213 (Ecosystem-as-a-Service architecture — Plugin/App Store substrate).
- ADR-0037 (Bominal plugin substrate — superseded for new work).
- ADR-0131 (per-microservice flat layout).
- ADR-0132 (no-grouping policy).
- Memory: `feedback_quality_performance_scalability_bar.md`; `feedback_workflow_objectgraph_adapter_layer.md`; `feedback_canonical_base_localization.md`.

## Doctrine refs (ADR-0346..0349)

- ADR-0346 — `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, invoking `cargo fmt --all --check`, `cargo check --workspace --all-targets --keep-going`, `cargo clippy --workspace --all-targets --keep-going -- -D warnings`, `cargo nextest run --workspace --no-fail-fast`, and `oya gate run-all --ci-required`; enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- ADR-0347 — every `oya-governance-*` CI lane prefix in the Oyatie corpus RENAMES to `oya-governance-*` in a single bulk-rename pull request (Wave 15-ZB); enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- ADR-0348 — cellular topology MUST support AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING; every µservice `manifest.json` gains a `sharding_automation` block declaring per-automation-mode configuration, with residency, threshold, audit-chain, and rollback coverage enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- ADR-0349 — Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts and ArgoCD replaces manual `kubectl apply` and Helm CLI deploys, with parity, cosign, tenant namespace, JCasC, and audit-chain enforcement by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.

## ADR-0339 adoption
- Lifecycle: PROPOSED for `plugin-app-store` until service wrappers invoke signed shared OpenTofu modules and implementation evidence lands.
- ADR-0339 adoption keeps reusable HCL in `microservices/cloud-iac/modules/<context>/<primitive>/`; `plugin-app-store` owns primitive selection and tenant-scoped variables.
- Manifest contract: `iac_module_invocations` declares 4 module pin(s) across 1 context(s).
- Scaling input: `per_capability` with cell placement `Tier-2` drives wrapper sizing rather than provider defaults.
- Supply-chain input: every future module source pin requires ADR-0181 cosign attestation, provider lock evidence, and catalog discoverability.
- Thin-wrapper rule: per-context `main.tf` files contain module invocations only, stay at or below 80 logical lines, and never own shared primitive bodies.
- Tenant rule: wrappers pass tenant_id, tenant_class, compliance-pack labels, cell_id, workload class, and cost tags explicitly.
- API rule: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3 contracts remain versioned independently from IaC module semantic versions.
- Maintainability rule: quarterly module windows move pins deliberately; primitive replacement uses dual-run evidence and an audit-visible sunset path.
- Done boundary: this PRD section is document-stage adoption only and does not claim wrapper migration, OpenTofu apply, or cloud resource creation.
- Verification: ADR citation, cohesion, and doc inventory gates must pass before this adoption can be reported complete.
