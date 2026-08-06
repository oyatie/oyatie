---
doc_class: ProductRequirements
product: saas-platform
status: planning-closed-contract-authored
date: 2026-06-09
owner: axis-saas
related_oyatie_adrs:
  - ADR-0035
  - ADR-0036
  - ADR-0249
  - ADR-0314
  - ADR-0321
  - ADR-0534
related_microservices:
  - workflow-engine
  - workflow-studio
  - plugin-app-store
  - marketplace
  - ontology
  - policy-engine
  - audit-chain
  - metering
  - tenancy
  - identity
tenant_class: ["controlled_evaluation", "paid"]
planning_refs:
  - specs/masterplan.json#/live_implementation_index/milestones/M03/phases/M03-P04
  - specs/masterplan.json#/live_implementation_index/milestones/M03/phases/M03-P08
live_readiness_claim: target_non_claim_until_changeset_gate_evidence
ci_authority: oya-ci-required
doc_status: published
---

# Oyatie — Product PRD: SaaS Platform

> **Status:** planning-closed contract authored, implementation changesets still required. This document closes the missing per-product PRD surface for the existing SaaS Platform entry; it does not claim live production readiness.
> **Owning team:** [`docs/teams/axis-saas/CHARTER.md`](../../teams/axis-saas/CHARTER.md)
> **Planning authority:** `specs/masterplan.json` M03-P04 and M03-P08.
> **Current-truth authority:** `specs/root-hub-pointers.json` routes to `specs/masterplan.json#masterplan_v2` and the applicable Accepted ADRs. `HANDOFF.md` is a thin fresh-session redirect only. The `registry/stores/*` files remain product-domain inputs, not portfolio plan/status authority.
> **Live CI authority:** the branch-protected `oya-ci-required` context produced by GitHub Actions until the owned runner cutover is proven.

## 1. North star

The SaaS Platform is Oyatie's shared tenant application substrate: a workflow engine, Workflow Studio, ontology-backed business objects, plugin runtime, plugin/app marketplace, public API stability tier, webhook signing, tenant metering, and partner integration surface that every vertical and workplace product can reuse without forking identity, audit, billing, policy, or regional-pack behavior.

The product exists to make tenant business operations programmable and governable while preserving the cohesion thesis from `docs/PRD.md`: one tenancy model, one identity surface, one audit chain, one cloud substrate, one policy posture, and one marketplace/extension model. It is not a standalone local toolchain and it does not make production or hyperscaler claims until M03-P04/M03-P08 changesets have green gate evidence.

## 2. Target users

| Persona | What they get | What they pay for / exchange |
|---|---|---|
| Tenant operator | Governed workflow execution, approvals, plugin installs, marketplace purchases, and audit evidence across their tenant. | Seat, workflow-run, plugin-invocation, and marketplace entitlement metering. |
| Tenant builder / IT admin | Workflow Studio, workflow definition versioning, plugin install governance, webhooks, and integration contracts. | Builder seats, plugin runtime usage, and partner API usage. |
| External developer / ISV | Plugin authoring, signing, trust-tier review, marketplace listing, and DealSet-backed entitlement economics. | Marketplace revenue share, API usage, and review obligations. |
| Vertical product team | Shared workflow/plugin/marketplace primitives for vertical-specific journeys without owning a parallel substrate. | Internal service consumption metered through platform metering. |
| Cloud/control-plane team | Clear SaaS↔Cloud contracts for tenant compute placement, storage residency, billing events, and Kubernetes workload status. | Contract compliance and cloud resource metering. |
| Security/compliance operator | Central evidence for workflow actions, plugin provenance, marketplace takedown, policy decisions, and regional-pack controls. | Audit-chain evidence, SLO evidence, and compliance pack proof. |

## 3. In-scope / out-of-scope

### 3.1 In scope by existing wave

| Wave / plan | Existing scope | Exit evidence |
|---|---|---|
| W-SaaS-Preview / M03-P04-IP-001 | Workflow engine kernel, per-tenant workflow definition versioning, state-machine + DAG execution, jurisdiction overlays, sealed steps, saga compensation. | Workflow execution tests, definition-version migration tests, audit emission checks, deadlock runbook evidence. |
| W-SaaS-Preview / M03-P04-IP-002 | Plugin substrate with Wasmtime/WASI-P2, capability-gated `PluginContext`, Cosign/Rekor provenance, trust tiers, resource caps, and lifecycle actions. | Sandbox escape regression tests, signature/SBOM/license gates, trust-tier/capability policy checks, revoke propagation evidence. |
| W-SaaS-Preview / M03-P04-IP-003 | Marketplace listing, ISV onboarding, trust-tier publishing, tenant install, entitlement, and listing takedown behavior. | Listing/install contract tests, entitlement/DealSet state checks, takedown runbook evidence, fraud/policy gates. |
| Cross-axis / M03-P08-IP-001 | SaaS↔Cloud, SaaS↔Search, and SaaS↔Agent-runtime contracts. | Contract registry entries, consumer fitness lanes, and branch-protected `oya-ci-required` gate evidence. |

### 3.2 Out of scope / anti-scope

- Cloud infrastructure hosting, Kubernetes cluster lifecycle, IAM/KMS/secrets implementation, and storage substrate internals are Cloud-axis responsibilities.
- Model/provider execution is `cloud-intelligence` scope; SaaS consumes capability-bound `oya-intelligence` tenant workflow execution through workflow steps.
- Search indexing implementation is Search-axis scope; SaaS emits consent- and ontology-scoped content contracts.
- Per-vertical domain logic remains with vertical/business/workplace teams; SaaS provides the shared workflow/plugin/marketplace substrate.
- Local CLI, shell scripts, or human-invoked governance are not destination authority. Any existing local command is diagnostic or developer convenience only; enforcement belongs in cloud-native services, declarative manifests, Rust gate crates, and the live `oya-ci-required` context.

## 4. Architecture overview

### 4.1 Bounded context

The SaaS Platform bounded context owns reusable tenant application primitives and the contracts that let other axes consume them:

- `workflow-engine`: durable workflow execution, state-vector persistence, versioned definitions, sealed steps, saga compensation, and audit emission.
- `workflow-studio`: tenant authoring surface that produces governed workflow definitions and templates.
- `plugin-app-store` / plugin substrate: plugin artifact provenance, trust tiers, install/revoke lifecycle, per-installation Cedar policy materialization, and runtime resource caps.
- `marketplace`: listing, install, entitlement, DealSet-backed settlement hooks, review/takedown, and ISV onboarding.
- `metering`: billable SaaS events for workflow runs, plugin invocations, API calls, and marketplace entitlements.
- `public API/webhooks`: stability tier and signed delivery for SaaS-authored external integration events.

### 4.2 Cross-axis seams

| Seam | SaaS responsibility | External owner | Fitness requirement |
|---|---|---|---|
| SaaS↔Cloud | Declare tenant workload, storage residency, billing-event, and cell-affinity requirements. | `axis-cloud` | Cloud resource and billing contracts pass M03-P08 fitness; no local control-plane authority. |
| SaaS↔Search | Emit ontology/data-boundary scoped index events and consent tiers. | `axis-search` | Search consumes only permitted tenant content and rejects stale schema. |
| SaaS↔Agent-runtime | Invoke agents only through capability-bound workflow steps with autonomy ceilings. | `cloud-intelligence` / `oya-intelligence` | Capability registry and audit evidence prove agent action bounds. |
| SaaS↔Workplace/verticals | Provide template library, plugin substrate, and workflow runtime without owning domain semantics. | Workplace and vertical teams | Template definitions compile and pass domain-owner contract checks. |
| SaaS↔Governance/audit | Emit immutable events and consume central Cedar/PaC/CaC/PaaS/CaaS decisions. | central governance + audit-chain | Gate and audit evidence are sealed before readiness claims. |

### 4.3 Delivery authority

- Source-of-truth planning is `specs/masterplan.json`; scratch `.omc`/`.omx` paths are references only when materialized through the committed plan projection.
- Current-truth decisions come from registry stores plus accepted ADRs; stale docs are not authority until reconciled.
- Live merge authority is the single branch-protected `oya-ci-required` context. The owned runner is a future cutover of the same pipeline, not a parallel verdict source.

## 5. Data structures

| Entity / aggregate | Required fields | Persistence / partitioning | Audit and event obligations |
|---|---|---|---|
| `WorkflowDefinition` | `workflow_id`, `version`, `tenant_id`, `state_machine`, `dag_per_state`, `sealed_steps`, `jurisdiction_overlay`, `capability_bindings` | Partition by tenant and workflow; version immutable after publish. | Publish, migrate, sunset, and overlay changes emit audit events. |
| `WorkflowRun` | `run_id`, `workflow_id`, `version`, `tenant_id`, `state_vector`, `current_state`, `saga_stack`, `idempotency_key` | Partition by tenant/cell; state-vector snapshots retained for replay evidence. | Every transition emits state, input/output hashes, executor identity, and duration. |
| `PluginArtifact` | `plugin_id`, `version`, `digest`, `trust_tier`, `capabilities`, `data_classes`, `resource_caps`, `signature_ref`, `sbom_ref` | Artifact digest is immutable; trust-tier decisions are versioned. | Submit, approve, reject, block, and restore events are sealed. |
| `PluginInstallation` | `installation_id`, `tenant_id`, `plugin_id`, `version`, `granted_capabilities`, `policy_ref`, `status` | Partition by tenant and plugin; revocation is explicit state, never deletion. | Install, capability change, invocation, cap exhaustion, and revoke events emit. |
| `MarketplaceListing` | `listing_id`, `publisher_tenant_id`, `artifact_ref`, `trust_tier`, `commercial_terms`, `status`, `jurisdiction_packs` | Listing state is versioned; discovery index is derived. | Publish, review, hide, freeze, takedown, restore, and appeal events emit. |
| `DealSetEntitlement` | `deal_set_id`, `tenant_scope`, `counterparty_roles`, `entitlement_terms`, `settlement_terms`, `status` | DealSet is tenant-scoped with explicit counterparty roles. | Offer, accept, grant, revoke, refund, dispute, and settlement transitions emit. |
| `SaaSMeteringEvent` | `event_id`, `tenant_id`, `surface`, `usage_kind`, `quantity`, `idempotency_key`, `billing_ref` | Idempotent by event and tenant; forwarded to cloud/billing seam. | Metering completeness is measured; missing billable event is a gate failure. |

Schema migrations must be reversible, tenant-scoped, and audit-emitting. Cross-axis schema changes require M03-P08 contract registry updates and fitness-lane evidence.

## 6. Optimization practices

- **Cell routing and tenant blast radius:** route workflow runs and plugin invocations by tenant/cell; do not use cross-tenant queues for mutable operations.
- **Idempotency:** all workflow transitions, plugin lifecycle actions, listing state changes, and metering events carry idempotency keys scoped by tenant and aggregate.
- **Backpressure:** per-tenant and per-plugin quotas prevent one tenant/plugin from exhausting shared workflow or runtime capacity.
- **Caching:** cache only derived listing/search/discovery projections; source-of-truth listing, install, entitlement, and workflow state remain transactional.
- **Batching:** batch metering and analytics after audit emission, never before mutation evidence.
- **Observability:** each surface exposes golden signals, tenant/cell labels, structured logs, traces, and audit-chain correlation IDs.
- **Progressive delivery:** enable by tenant cohort, plugin digest, workflow definition version, or listing jurisdiction; rollback means explicit compensating state, not row deletion.
- **FinOps:** meter workflow-runs, plugin invocations, marketplace entitlement state changes, and public API/webhook delivery costs.

## 7. Regional pack interactions

| Regional seam | SaaS behavior |
|---|---|
| Residency | Workflow state, plugin data access, marketplace entitlement, and audit evidence respect tenant region/cell binding. |
| Jurisdiction overlay | Workflow definitions accept mandatory regional steps that tenant admins cannot remove. |
| Tax and invoicing | Marketplace and plugin revenue events emit tax-region fields for billing/tax owners. |
| Sanctions/export control | Marketplace listings, plugin capabilities, and DealSet entitlements are denyable by regional pack policy. |
| Privacy/data classes | Plugin manifests and workflow steps declare data classes; Cedar and Data Use Boundary policy deny incompatible grants. |
| Regulated evidence | PHI, PCI, financial, public-sector, and regional-pack incidents preserve sealed audit evidence and compliance owner handoff. |

## 8. In-house vs external dependency posture

| Component | Posture | Rationale / guardrail |
|---|---|---|
| Workflow engine | In-house canonical engine per ADR-0035. | Hybrid state-machine + DAG, per-tenant versioning, overlays, sealed steps, and agent-authored steps are core differentiators. |
| Wasmtime/WASI-P2 | Adopted runtime boundary per ADR-0036. | Runtime is sandbox boundary only; plugin authority remains capability-gated and tenant-scoped. |
| Cosign/Rekor/SBOM tooling | Adopted provenance controls per ADR-0036/ADR-0534. | Required for artifact trust; failure blocks production loading. |
| Marketplace settlement | In-house DealSet primitive per ADR-0314. | Avoids one table per product/module and preserves tenant/audit/settlement cohesion. |
| Governance/policy | Central PaC/CaC/PaaS/CaaS and Cedar. | No scattered CLI governance or per-service policy dialects. |
| CI/gates | Shared Rust gate logic invoked by `oya-ci-required`. | One live required context; no parallel CI authority or local false-green verdict. |

No new dependency is introduced by this PRD. Future dependency additions require ADR/license/security review and changeset gate evidence.

## 9. Success metrics

| Metric | Target / gate | Evidence source |
|---|---|---|
| W-SaaS-Preview functional gate | Tenant onboarding, plugin install, and marketplace listing are functional. | M03-P04 changeset tests and `oya-ci-required` gate output. |
| Workflow execution latency | p99 under 500 ms for synchronous steps where the step is not waiting on a human/external timer. | Workflow SLO dashboard and test fixtures. |
| Plugin sandbox escape incidents | 0 unresolved production escapes; every attempted escape emits deny/audit evidence. | Runtime security metrics and plugin incident runbook evidence. |
| Marketplace review turnaround | ≤ 5 business days for complete verified-ISV submissions. | Listing review queue metrics. |
| Metering completeness | 100% of billable workflow/plugin/marketplace/API events emitted once. | Metering idempotency and reconciliation checks. |
| Cross-axis contract drift | 0 unreviewed contract changes at merge. | M03-P08 fitness lanes and branch-protected `oya-ci-required`. |
| Audit evidence completeness | 100% of mutable workflow/plugin/listing/entitlement state changes sealed. | Audit-chain correlation checks. |

## Competitive benchmark

| Comparator | SaaS Platform benchmark stance | Evidence / caveat |
|---|---|---|
| Salesforce Flow / AppExchange | Oyatie targets a unified workflow + plugin + marketplace substrate instead of separate automation, extension, entitlement, and audit surfaces. | Target contract only until M03-P04/M03-P08 changesets prove workflow execution, plugin install, marketplace listing, entitlement, and audit-chain behavior behind `oya-ci-required`. |
| ServiceNow App Engine / Store | Oyatie requires tenant-scoped workflow and plugin execution with sealed audit evidence, regional-pack overlays, and central policy authority. | Success gates include p99 under 500 ms for synchronous workflow steps, 0 unresolved production sandbox escapes, and 100% billable-event emission. |
| Zapier / n8n automation ecosystems | Oyatie treats workflow automation as an in-tenant governed execution fabric, not an external integration-only tool. | Local CLI checks remain diagnostic; cloud-native control-plane evidence and branch-protected CI are the readiness authority. |
| AWS Marketplace / Atlassian Marketplace | Oyatie's marketplace benchmark is install governance, trust-tier publishing, takedown, DealSet entitlement state, and tenant/audit continuity. | Listing review target is ≤ 5 business days for complete verified-ISV submissions; live readiness remains a target/non-claim until gate evidence exists. |

## 10. Risks and mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| Plugin sandbox escape crosses tenant/data boundaries. | Catastrophic | Wasmtime/WASI-P2, no raw host access, capability-gated `PluginContext`, Cosign/Rekor/SBOM, resource caps, and Sev 1 runbook. |
| Workflow deadlock stalls tenant operations. | High | Versioned definitions, state-vector restore, saga compensation, idempotency, per-tenant quarantine, and deadlock runbook. |
| Marketplace fraudulent or unsafe listing is published. | High | Trust-tier review, publisher KYB/KYC, artifact provenance, listing takedown runbook, and DealSet entitlement state. |
| Cross-axis contract drift breaks Cloud/Search/Agent-runtime consumers. | High | M03-P08 contract registry, consumer fitness lanes, and `oya-ci-required` blocking context. |
| Local command output is mistaken for production authority. | High | D-CLOUD-NATIVE and D-CICD-AUTHORITY: only cloud-native pipeline/controller evidence and branch-protected `oya-ci-required` can satisfy readiness gates. |
| Metering gaps create billing or compliance defects. | Medium | Idempotent metering events, reconciliation gates, audit-chain correlation, and FinOps ownership. |
| Product breadth turns into disconnected surfaces. | Medium | Axis-saas charter, PRD scope boundaries, shared aggregates, and central governance. |

## 11. Open questions

No product-scope question blocks this PRD surface. Implementation readiness remains blocked until M03-P04 and M03-P08 changesets produce green tests, contract registry evidence, and branch-protected gate evidence. Any new question discovered during implementation must be attached to the relevant changeset rather than widening this PRD.

## 12. Decision log

| Date | Decision | Source |
|---|---|---|
| 2026-05-09 | Workflow engine is hybrid state-machine + DAG with per-tenant versioning, sealed steps, saga compensation, and audit emission. | ADR-0035 |
| 2026-05-09 | Plugin substrate uses Wasmtime/WASI-P2, `PluginContext`, Cosign/Rekor, trust tiers, resource caps, and marketplace economics. | ADR-0036 |
| 2026-05-20 | Marketplace is a multi-category doctrine and DealSet settlement primitive, not a product-specific table island. | ADR-0249, ADR-0314 |
| 2026-05-19 | M03-P04 and M03-P08 are planning-closed contracts that still require changeset verification and gate evidence before production exit. | `specs/masterplan.json` |
| 2026-06-09 | SaaS Platform per-product PRD authored from existing docs/stores/masterplan; live readiness remains target/non-claim. | task-7 worker evidence |

## 13. Sources scanned

- `HANDOFF.md` — thin fresh-session redirect; derive repo/CI/current truth from its canonical targets and live GitHub state.
- `registry/stores/design-store.json` — D-CLOUD-NATIVE, D-CICD-AUTHORITY, D-GOVERNANCE-CENTRAL, plugin/workflow/marketplace current-truth entries.
- `registry/stores/instructions-store.json` — no CLI authority, no shell/governance CLI, one `oya-ci-required` context, GitHub Actions live authority.
- `registry/stores/registry-store.json` and `registry/stores/canon-id-crosswalk.json` — canon/store authority chain.
- `specs/masterplan.json` — M03-P04 SaaS Platform Preview and M03-P08 cross-axis contract registry planning authority.
- `docs/PRD.md` — W-SaaS-Preview scope, cohesion thesis, and cross-axis integration table.
- `docs/products/README.md` — required per-product PRD sections and SaaS Platform index entry.
- `docs/teams/axis-saas/CHARTER.md` — team scope, dependencies, metrics, and risk register.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`.
- `docs/decisions/ADR-0705-product-protocol-live-apex.md`.
- `docs/decisions/ADR-0705-product-protocol-live-apex.md`.
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`.
- `docs/runbooks/saas/workflow-engine-deadlock.md`, `docs/runbooks/saas/plugin-runtime-sandbox-escape.md`, and `docs/runbooks/saas/marketplace-listing-takedown.md`.

## 2a. Acceptance criteria traceability (required)

This section is a planning-maturity contract only. It does **not** claim runtime, product-ready, or hyperscaler-ready status; promotion still requires fresh CI, SLO, security, SBOM, rollback/DR, owner/RACI, and product-pain evidence.

| AC-ID | Given | When | Then | Test ID | Test path |
|---|---|---|---|---|---|
| SAAS-PRD-AC-001 | The SaaS Platform PRD is used as a planning contract and tenant workflow, plugin, marketplace, billing, and audit contracts are referenced by a promotion packet | The planned-maturity gate scans product PRDs | workflow/plugin/marketplace acceptance is linked to test and evidence paths instead of a generic readiness sentence | SAAS-PRD-GATE-001 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |
| SAAS-PRD-AC-002 | SaaS subscription or partner integration readiness is evaluated | Readiness evidence is evaluated | fresh tenant workflow execution, plugin publish, marketplace listing, billing, and audit evidence is required outside this PRD | SAAS-PRD-GATE-002 | `cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app/tests/planned_maturity.rs::live_product_prds_capabilities_and_retired_plan_refs_are_maturity_gated` |

## 9b. Verification commands (required) — one runnable check per metric

| Metric | Verification command | Pass criterion | CI lane |
|---|---|---|---|
| SaaS workflow/plugin/marketplace planning maturity | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | At least one SaaS row names workflow, plugin, marketplace, tenant, and audit/billing evidence obligations | `oya-ci-required` |
| SaaS product-ready non-claim boundary | `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-planned-maturity-app:oya-cloud-ci-planned-maturity-app-gate` | A SaaS promotion packet cannot treat this PRD as product-ready evidence without fresh CI and product-pain proof | `oya-ci-required` |
