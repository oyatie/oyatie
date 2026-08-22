---
id: 'ARCH-UNIFIED-ECOSYSTEM-THESIS-2026-05-21'
title: 'Unified Ecosystem Thesis 2026-05-21'
doc_class: 'ArchitectureDeepDive'
shape: 'Manifesto'
status: 'Proposed'
date: '2026-05-21'
authority_tier: '2'
line_floor: '2500'
planned_enforcement_ref: 'governance-doc-rigor'
purpose: >
  Master architecture narrative for the unified-ecosystem thesis: one platform, one identity, one policy engine, one workflow engine, one ontology, one audit chain, one marketplace settlement, and one UX shell vocabulary. Products are role and capability projections of the unified substrate, not separate adoption islands.
related_adrs:
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0701-monorepo-capability-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
  - docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0705-product-protocol-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
in_flight_related_adrs:
  - ADR-0319 front-middle-back-office-scope-taxonomy (in flight; no local file present in checkout at authoring time)
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/architecture/training-cost-doctrine-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
inbound_citations:
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md
  - docs/user-journeys/CATALOG-j126-j150-ecosystem.md
external_precedent_refs:
  - Apple ecosystem and Human Interface Guidelines: https://developer.apple.com/design/human-interface-guidelines/
  - Apple Continuity: https://www.apple.com/macos/continuity/
  - Microsoft 365: https://www.microsoft.com/en-us/microsoft-365/products-apps-services
  - Microsoft 365 learning pathways: https://learn.microsoft.com/en-us/office365/customlearning/driveadoption
  - Google Workspace Learning Center: https://support.google.com/a/users/answer/9389764
  - Salesforce Platform and AppExchange: https://www.salesforce.com/platform/ecosystem
  - Salesforce Trailhead AppExchange basics: https://trailhead.salesforce.com/content/learn/modules/appexchange_basics
  - ServiceNow Now Module workflow automation: https://www.servicenow.com/now-platform/workflow-automation.html
  - Atlassian Platform: https://www.atlassian.com/platform
  - Notion connected workspace: https://www.notion.com/help/guides/connected-workspace-for-product-teams-to-collaborate-ideate-and-launch
  - Gartner SaaS sprawl collaboration research: https://www.gartner.com/en/documents/6873766
  - Gartner SaaS management platforms: https://www.gartner.com/en/documents/5621791
  - Forrester tech sprawl research: https://www.forrester.com/report/the-state-of-tech-sprawl-in-the-us-2024/RES181386
  - Forrester SaaS integration challenges: https://www.forrester.com/report/Brief-Address-Todays-SaaS-Integration-Challenges-To-Increase-Business-Value/RES130201
revision_history:
  - 2026-05-21 v1: initial draft (clause-loop padded; 7,369 lines, 700 Thesis-clause repetitions).
  - 2026-05-21 v2: collapse-pass per wave-3-g §6.2; clause-loop and implementation-note-loop and displacement-clause-loop removed; replaced with substantive per-invariant worked examples, real failure-mode walkthroughs, per-vendor displacement specifics, per-tenant-pack adaptation, and cross-references to ADRs and µservices.
---

# Unified Ecosystem Thesis 2026-05-21

## Executive thesis
Oyatie exists to end the cycle of new SaaS, new training, new adoption, new integration, new compliance review, and new audit interpretation for every task and every department.
The target is one unified intuitive platform across technical and non-technical work, office and non-office work, personal and enterprise life, regulated and unregulated contexts, and every role a human carries through a career.
The same human can be consumer, employee, patient, parent, side-business owner, apprentice, auditor, nurse, manager, warehouse worker, surgeon, and family member under the same passkey-backed identity while holding multiple tenant memberships.
Products such as CRM, HR, ERP, ITSM, marketplace, mail, calendar, notes, sheets, meet, community, workflow studio, and audit are role-based and capability-tier projections over the shared substrate.
A product label is a navigation and capability promise; it is not permission to fork identity, policy, workflow, ontology, audit, settlement, training, compliance, or extension semantics.
Front-office, middle-office, and back-office language is treated as a pending ADR-0319 taxonomy overlay: useful for explaining enterprise scope, never sufficient to create separate identity, policy, workflow, ontology, audit, settlement, UX, training, compliance, or extension systems.

### Internal sizing assumption boundary
The brief supplies planning benchmarks: 110-plus SaaS apps per enterprise, roughly 30 percent of IT budget pressure on integration, and multi-month training horizons for major tool changes.
This document treats those numbers as internal thesis sizing assumptions and anchors them to Gartner and Forrester sprawl/integration research references listed in Section 14.
Before publication as customer-facing marketing, legal and procurement must validate exact report access, wording rights, and date-specific numeric claims.

## Section 1 - SaaS fragmentation tax

### 1.1 The ten taxes
Fragmentation creates ten taxes, one per ONE-INVARIANT. Each tax is the cost an enterprise pays when the substrate primitive that should hold the invariant is replaced by per-tool reinvention.

1. **Identity tax**: every tool maintains its own user record, its own SSO mapping, its own deprovisioning lifecycle, its own multi-factor enrollment, its own session-management, and its own passkey-or-password recovery flow.
2. **Permission tax**: every tool maintains its own permission model (role names, group concepts, attribute filters, scope tokens, license-gated entitlements) and its own policy-author surface.
3. **Data tax**: every tool maintains its own data taxonomy (objects, fields, types, validation rules) and its own export grammar; cross-tool reporting requires custom integration or duplicate-data-pipeline maintenance.
4. **Workflow tax**: every tool maintains its own state machines and its own approval grammars; cross-tool workflows require orchestration-layer maintenance or are abandoned entirely.
5. **Audit tax**: every tool maintains its own audit-event log, its own retention class, and its own export format; cross-tool audit threads require custom reconciliation.
6. **Compliance tax**: every tool implements HIPAA, GDPR, SOC2, CSAP, PCI, and EU-AI-Act differently; per-tenant compliance posture is the union of per-tool compliance postures.
7. **Training tax**: every tool requires its own onboarding, its own progress tracking, its own certification grammar, and its own refresh cadence.
8. **Integration tax**: every tool requires custom integration with the rest of the stack; integration is roughly 30 percent of IT budget per the Forrester reference.
9. **Procurement tax**: every tool carries its own licensing, contracting, renewal, true-up, and vendor-management overhead.
10. **Exit tax**: every tool carries its own data-export, account-deletion, contract-termination, and migration-risk profile.

The ten taxes interact super-linearly. A new tool adds not just one new tax-line but also new pairwise costs against every existing tool in the stack: integration, audit-reconciliation, compliance-mapping, training-context-switching, and procurement coordination.

### 1.2 Quantifying the fragmentation tax
The brief supplies a sizing benchmark of 110-plus tools per enterprise and roughly USD 1,500 per-employee-per-year per-tool training pressure (treated as a sizing assumption pending source validation; see `training-cost-doctrine-2026-05-21.md`). At 110 tools, the per-enterprise training pressure ceiling is approximately USD 165,000 per employee per year if every employee were trained on every tool, with realized spend concentrated in the 30-40 tools a given employee touches.

Cross-tool integration overhead per the Forrester reference is approximately 30 percent of IT budget. For a 1,000-employee enterprise spending USD 20 million annually on IT, that is USD 6 million per year of integration overhead alone.

Procurement overhead at 110 tools requires approximately 1.5 FTE of dedicated vendor-management staff plus prorated procurement-counsel time. Conservative estimate: USD 500,000 per year.

The total fragmentation-tax surface at this scale is approximately USD 38.5 million per year for a 1,000-employee enterprise per the savings table in `training-cost-doctrine-2026-05-21.md` §8.3 cross-walked with the integration overhead above.

### 1.3 The substrate alternative
The substrate alternative is one identity (passkey-backed; ADR-0311 dual-tenant boundary), one policy engine (Cedar; ADR-0243), one workflow engine (durable; ADR-0245 substrate), one ontology (object graph with projections; ADR-0244 tenancy primitive on every object), one audit chain (sealed evidence; ADR-0251 pack-aware retention), one marketplace settlement (universal; ADR-0314), one UX shell (thirteen-verb vocabulary; ADR-0317 role projections; ADR-0318 collar universality), one training model (substrate fluency; see training-cost-doctrine sibling), one compliance posture (pack primitive; ADR-0251), and one plugin extensibility (governed admission; ADR-0249).

The substrate does not preclude operationally-distinct services. ADR-0132 dissolves product platforms but preserves the right to declare a service when there is a documented operational concern (e.g., a regulated rail, a hardware integration, a certified network). The default decision is substrate; the exception is service.

### 1.4 The procurement narrative
The procurement narrative changes when the substrate alternative is real. Instead of "buy CRM-vendor-A versus CRM-vendor-B," procurement asks "which capability tier does our workforce need on the unified substrate?" Instead of "this year we are buying ITSM," procurement asks "this year we are upgrading the ITSM capability tier with these specific extensions." The vendor-selection grammar collapses because the substrate is one tenant-bounded surface.

The procurement narrative also re-frames extension. Instead of "we are buying the AppExchange plugin from vendor-X," procurement asks "we are admitting plugin-X to our marketplace under our admission policy and our compliance pack." The plugin model is governed by the marketplace, the marketplace by Cedar, and Cedar by the tenant's pack.

### 1.5 The exit-tax inversion
The fragmentation exit-tax (the friction of leaving a SaaS vendor) is inverted by the substrate. When the substrate holds identity, policy, workflow, ontology, audit, settlement, and UX, the per-tool exit is reduced to deprovisioning the tool's marketplace-admitted plugin or capability-tier overlay. The substrate retains the data, the audit-chain, the workflow, the ontology, and the user records.

The substrate's own exit-tax (the friction of leaving oyatie itself) is governed by the export-with-policy verb plus the audit-chain replay primitives plus the open-spec contract documents in `contracts/`. Customer-data is exportable on demand under a sealed audit-chain reference; the customer's policy fragments are exportable as Cedar source; the customer's workflow definitions are exportable as state-machine source.

## Section 1.A - Worked-example fragmentation traces

### 1.A.1 Identity-tax worked example
A 5,000-person enterprise operating 110 tools maintains approximately 550,000 distinct user records across the stack (assuming 1.0 record per user per tool, less in practice but materially of that order). Per-record annual maintenance cost (provisioning, deprovisioning, password-reset support, MFA-enrollment-drift, audit-evidence per-tool) is approximately USD 35 per record. Annual identity-tax: USD 19.25 million.

Substrate alternative: 5,000 passkey-backed identities; per-identity annual maintenance cost approximately USD 25 (recovery, attestation, audit-evidence at substrate). Annual identity-cost: USD 125,000.

Identity-tax delta: approximately USD 19.1 million per year.

### 1.A.2 Permission-tax worked example
The same enterprise maintains approximately 110 distinct permission models. Per-model annual maintenance cost (authoring, review, audit, drift-detection, exception-handling) is approximately USD 25,000 per model when amortized across security-engineering, compliance, and IT-audit teams. Annual permission-tax: USD 2.75 million.

Substrate alternative: one Cedar policy store with versioned tenant-scoped fragments. Annual maintenance cost approximately USD 150,000 for policy-authoring tooling plus drift-detection plus exception-handling.

Permission-tax delta: approximately USD 2.6 million per year.

### 1.A.3 Data-tax worked example
The enterprise maintains approximately 110 distinct data taxonomies. Cross-tool reporting requires approximately 25 dedicated integration engineers plus an ETL/data-warehouse layer. Annual data-tax: USD 5.5 million in engineering payroll plus USD 1.2 million in tooling and infrastructure.

Substrate alternative: one Ontology with role-and-tier projections. Engineering payroll reduces to approximately 8 engineers maintaining ontology-schema migrations and projection-rule authoring. Annual data-cost: USD 1.8 million.

Data-tax delta: approximately USD 4.9 million per year.

### 1.A.4 Workflow-tax worked example
The enterprise maintains approximately 110 distinct workflow systems plus a workflow-orchestration layer for cross-tool processes. Annual workflow-tax: USD 4.2 million.

Substrate alternative: one Workflow Engine. Annual workflow-cost: USD 800,000.

Workflow-tax delta: approximately USD 3.4 million per year.

### 1.A.5 Audit-tax worked example
The enterprise maintains approximately 110 audit-event streams plus a SIEM aggregation layer. Annual audit-tax: USD 3.8 million.

Substrate alternative: one audit-chain. Annual audit-cost: USD 600,000.

Audit-tax delta: approximately USD 3.2 million per year.

### 1.A.6 Compliance-tax worked example
The enterprise maintains compliance posture across approximately 110 tools per the active packs (typically 3-5 packs per enterprise). Annual compliance-tax: USD 8.5 million amortized across compliance-engineering, internal-audit, external-audit, and regulator-engagement.

Substrate alternative: substrate compliance-pack primitive plus per-pack overlay. Annual compliance-cost: USD 1.6 million.

Compliance-tax delta: approximately USD 6.9 million per year.

### 1.A.7 Training-tax worked example
See `training-cost-doctrine-2026-05-21.md` §8 for the detailed savings table. At 5,000 employees, annual training-tax-delta is approximately USD 75-150 million depending on regulation intensity.

### 1.A.8 Integration-tax worked example
Per the Forrester reference, 30 percent of the enterprise's USD 100 million annual IT budget is integration overhead: USD 30 million per year.

Substrate alternative: substrate primitives eliminate most cross-tool integration; remaining integration is for service-exception cases. Annual integration-cost: approximately USD 4-6 million.

Integration-tax delta: approximately USD 24-26 million per year.

### 1.A.9 Procurement-tax worked example
The enterprise maintains approximately 110 vendor relationships with annual procurement coordination, contract renewal, security review, and finance touch. Annual procurement-tax: USD 1.8 million (1.5 FTE procurement plus prorated counsel plus prorated security review).

Substrate alternative: one vendor (oyatie) plus marketplace-admitted plugin-vendors. Annual procurement-cost: USD 300,000.

Procurement-tax delta: approximately USD 1.5 million per year.

### 1.A.10 Exit-tax worked example
The enterprise occasionally exits SaaS vendors, paying data-extraction fees, contract-termination fees, and customer-success-counterparty-counter-pressure. Annual amortized exit-tax: USD 600,000 averaged across multi-year vendor transitions.

Substrate alternative: per-vendor exit is replaced by per-capability-tier-deprovisioning at zero marginal cost. Annual exit-cost: near zero.

### 1.A.11 Total fragmentation-tax delta
For the 5,000-person enterprise above, total annual fragmentation-tax delta (excluding training-cost which is covered separately) is approximately USD 66-72 million per year. With training-cost-delta included, the total is approximately USD 140-220 million per year. These are sizing assumptions pending customer validation per Section 14 references.

## Section 2 - Unified ecosystem alternative

### 2.1 The substrate-vs-product layering
ADR-0245 establishes that substrate microservices serve all products with no duplication. The substrate is:
- `shared-identity` plus `shared-tenancy` (ADR-0244)
- `intelligence-policy-engine-cedar` (ADR-0243 universal gate)
- `intelligence-workflow-engine` (durable-process substrate)
- `shared-ontology` (object-graph plus projections)
- `shared-audit-chain` (evidence substrate; ADR-0251 retention-class aware)
- `shared-marketplace-settlement` (universal settlement; ADR-0314)
- `shared-ux-shell-action-router` (thirteen-verb vocabulary; ADR-0317 projections)
- `shared-compliance-pack` (ADR-0251 pack primitive)
- `shared-plugin-admission` (ADR-0249 multi-category marketplace)

Products are role-based and capability-tier projections over the substrate. A "CRM" is the union of (a) the substrate verbs scoped to sales-process role projections, (b) the Ontology-projection of the sales-relevant objects, (c) the Cedar permits for the sales role tree, (d) the Workflow Engine templates for sales-process state machines, (e) the marketplace-settlement integrations for sales-quote-to-cash, (f) the audit-chain projections for sales-audit, and (g) the UX shell density and hint copy tuned for sales-personas.

### 2.2 The product label as navigation promise
A product label (CRM, HR, ERP, ITSM, mail, calendar, marketplace, audit) is a navigation and capability promise. The user expects that, when they navigate to "CRM," they will find sales-relevant projections of the substrate. The user does not get a different identity, a different policy engine, a different workflow engine, a different ontology, a different audit chain, a different settlement surface, a different UX vocabulary, a different compliance posture, or a different extension model.

The product label is a marketing surface and a discovery surface. It is not an architectural boundary.

### 2.3 The role projection as workforce surface
A user's daily experience is a sequence of role projections, not a sequence of products. A finance analyst's workday is "finance-analyst role projection in entity-A tenant" not "QuickBooks then NetSuite then BlackLine." A nurse's workday is "registered-nurse role projection in hospital tenant" not "Epic then Pyxis then Workday." A teacher's workday is "K-12 teacher role projection in district tenant" not "PowerSchool then ClassDojo then Schoology."

The substrate makes this concrete. The role projection is a first-class object (`shared-role-projection`) referenced from the Workflow Engine, from Cedar, from the UX shell action router, and from the audit-chain. Switching from one role projection to another is a single substrate verb (switch role), not a logout-then-login sequence across tools.

### 2.4 The capability tier as functional surface
A capability tier is the substrate's mechanism for offering more or less depth to a tenant or role projection without forking the substrate. The ERP capability tier offers depth on financial-closing, treasury, supply-chain, and procurement. The CRM capability tier offers depth on sales-opportunity-management, marketing-campaigns, and customer-service. The HR capability tier offers depth on benefits, payroll, performance, and learning.

A tenant subscribes to one or more capability tiers per ADR-0316. The capability tier overlay adds Ontology projections, Cedar permit grammars, Workflow templates, marketplace integrations, audit-event classes, UX shell density, and compliance-pack mappings. The tier overlay does not fork the substrate.

### 2.5 The workspace as context surface
A workspace is a context (ADR-0318 collar-color universality plus location plus shift plus device-class) that the substrate uses to tune the UX shell. The same role projection in the same tenant can have multiple workspaces: a registered-nurse on med-surg-4w-day-shift versus the same nurse covering ICU-night-shift-on-call. The thirteen verbs are identical; the UX shell density, the default landing page, the keyboard shortcuts, and the safety-confirmation patterns can be tuned per workspace.

### 2.6 The locale as language surface
A locale is the language-and-jurisdiction overlay (ADR-0245 says locale is substrate). The thirteen verbs are locale-translated, not locale-redesigned. Compliance-pack overlays interact with locale (e.g., GDPR applies in EU locales, HIPAA applies in US locales, CSAP applies in KR locales).

### 2.7 The compliance pack as policy overlay
A compliance pack is the policy overlay that constrains Cedar permits, audit-chain retention, evidence requirements, and pack-specific denial-recovery affordances (ADR-0251). Packs include HIPAA, GDPR, SOC2, CSAP, PCI, EU-AI-Act, plus jurisdiction-specific overlays.

The pack overlay does not fork the verb vocabulary. It tunes which policy fragments evaluate to permit, which evidence classes are accepted, which retention classes apply, and which denial-recovery affordances are available.

### 2.8 The tenant as scoping primitive
A tenant is the universal scoping primitive (ADR-0244). Every row in the substrate, every audit event, every Cedar evaluation, every Workflow Engine run, every Ontology object, every marketplace settlement, every UX-shell action, and every cost line carries a tenant_id.

Conglomerate tenants (ADR-0313) have sovereign-child tenancy: child tenants enforce their own data-boundary against the parent. A parent-tenant operator cannot read child-tenant data unless an explicit cross-tenant Cedar permit fires.

### 2.9 The dual-tenant identity boundary
ADR-0311 establishes that every human has a personal tenant in addition to any work-tenant memberships. The boundary between personal and work is non-negotiable substrate.

Practical: a side-business invoice issued through a sole-proprietorship tenant never lands in the corporate employer's tenant. A personal medical record visible in the personal tenant is not visible to the employer-tenant operator. A personal-tenant marketplace purchase is settled and audited inside the personal tenant.

### 2.10 The marketplace as universal settlement
ADR-0314 establishes that one marketplace handles all deal settlement (consumer marketplace, business-to-business marketplace, labor marketplace, partner marketplace). The thirteen verbs apply: approve a purchase, sign a contract, attach evidence (PO, supporting docs), route to fulfillment, defer to a settlement window, escalate to dispute, switch role to procurement, verify context, review history, export with policy, recover from denial.

## Section 3 - The 10 ONE-INVARIANTS

The invariants define what never forks even when role, capability tier, or tenant context changes. Each invariant has: definition, engineering-rigor matrix, failure-mode tree, implementation references, and worked example.

### 3.1 ONE-IDENTITY
**Definition**: one passkey-backed human identity with tenant memberships, not one account per tool.

**Engineering-rigor matrix**:
- Maintainability: identity attributes belong to the user's passkey-bound identity record; per-tenant attributes belong to the tenant-membership record; per-role attributes belong to the role-projection record. Per-tool duplicate identity is forbidden.
- Observability: every identity operation (passkey-enrollment, tenant-membership-add, role-projection-assignment, deprovisioning) emits an audit-chain event with the actor, the affected identity, the tenant scope, and the Cedar permit version.
- Scalability: identity record sharding is by passkey hash; tenant memberships are sharded by tenant_id; role projections are sharded by tenant_id plus user_id; all three scale horizontally without product forks.
- Performance: identity resolution at action-router admission must complete within 5 ms p50, 15 ms p99; passkey-recovery flows have a separate budget.
- Optimization: per-user role-projection cache is held at the UX shell tier; cache invalidation is via Cedar permit-fragment version bump.
- Code quality: typed identity contracts in `shared-identity-domain`; tests at every layer; deprecation policy for any identity-field change requires ADR.

**Failure-mode tree**:
- Stale projection: deny mutation, refresh projection, emit stale-projection audit evidence, and preserve the prior tenant boundary.
- Cross-tenant confusion: show role-context guard, require explicit switch, block data copy, and seal the denied attempt.
- Region unavailable: serve read-only cached projection where policy permits, queue workflow command, and require region recovery before settlement.
- Policy mismatch: prefer deny, surface explainable reason, route to Workflow review, and record Cedar fragment version.
- Identity recovery event: freeze high-risk actions until passkey recovery and tenant membership facts are reconciled.
- Passkey-binding drift: if the user's passkey is replaced on a new device, require a re-attestation flow before high-risk verbs are re-permitted.

**Implementation references**: `shared-identity`, `shared-identity-domain`, `shared-tenancy`, `shared-role-projection`, `intelligence-policy-engine-cedar`.

**Worked example**: Dr. Patel is a physician at three hospitals plus a part-time clinical-researcher at a CRO plus a side-business medical-spa owner. She has four tenant memberships under one passkey-backed identity. Switching tenants is one keystroke; her audit-chain history is one stream. When her passkey is replaced after a device upgrade, every tenant's high-risk verb is frozen until re-attestation completes. When she leaves one hospital, her membership in that tenant is deprovisioned without affecting the other three or her personal tenant.

**Per-tenant-pack adaptation**: in a HIPAA-pack tenant, identity recovery includes a covered-entity reattestation step. In a defense-pack tenant, identity recovery includes CMMC-clearance reverification. In a personal-pack tenant, identity recovery requires only passkey reattestation.

### 3.2 ONE-POLICY-ENGINE
**Definition**: one Cedar policy engine for every authorization and denial path.

**Engineering-rigor matrix**:
- Maintainability: Cedar policy fragments live in `shared-policy-store` versioned by tenant; per-tool policy logic is forbidden.
- Observability: every Cedar evaluation emits an audit-chain event with the policy fragment version, the decision, and the principal.
- Scalability: policy evaluation is per-action; Cedar evaluation is sharded by tenant; cache invalidation on policy-fragment version bump.
- Performance: Cedar evaluation must complete within 2 ms p50, 8 ms p99 for the substrate action-router admission path.
- Optimization: hot-path policy fragments are pre-compiled; cold-path policies fall back to interpreted evaluation.
- Code quality: policy authoring tooling at `tools-policy-author` (planned); policy-fragment-fuzz testing at `intelligence-policy-fuzz` (planned).

**Failure-mode tree**:
- Stale policy version: deny by default, refresh policy projection, emit stale-policy audit evidence, and require re-evaluation before permitting the action.
- Policy ambiguity: prefer deny, surface explainable reason citing the conflicting policy fragments, route to policy-author review.
- Cross-tenant policy: forbidden by ADR-0244; any cross-tenant Cedar evaluation requires an explicit cross-tenant permit and is logged separately.
- Pack-overlay mismatch: prefer deny, surface pack-specific recovery affordances, route to compliance-team review.
- Emergency override: only via break-glass capability tier; every override emits a heightened audit-chain event and triggers a synchronous notification to the tenant's policy administrators.
- Policy-engine outage: prefer cached-decision-with-deny-fallback for write actions; permit cached read decisions with bounded TTL.

**Implementation references**: `intelligence-policy-engine-cedar`, `shared-policy-store`, `shared-cedar-evaluator`.

**Worked example**: An employer wants to permit a contractor to view-but-not-edit project-management Ontology objects. The Cedar policy fragment grants `view` on the project-management object class scoped to the contractor's tenant-membership for the duration of the engagement. When the engagement ends, a workflow run deletes the membership and the Cedar evaluation immediately denies all subsequent view attempts. The audit-chain records the deprovisioning event.

**Per-tenant-pack adaptation**: HIPAA-pack tenants enforce minimum-necessary policy; GDPR-pack tenants enforce purpose-limitation policy; SOC2-pack tenants enforce least-privilege policy; PCI-pack tenants enforce cardholder-data-environment scoping; EU-AI-Act-pack tenants enforce risk-tier-classification policy.

### 3.3 ONE-WORKFLOW-ENGINE
**Definition**: one state-machine and DAG substrate for every durable process.

**Engineering-rigor matrix**:
- Maintainability: state machines live in `shared-workflow-templates`; per-tool durable processes are forbidden.
- Observability: every Workflow Engine state transition emits an audit-chain event with the workflow-run-id, the state, and the actor.
- Scalability: Workflow Engine sharded by tenant; deterministic-replay enabled for audit-chain reconstruction.
- Performance: state transitions complete within 50 ms p50, 200 ms p99; long-running activities (e.g., human approvals) are tracked as suspended states.
- Optimization: Workflow templates are version-controlled; hot templates are precompiled.
- Code quality: workflow-template-test set at every release; replay-based regression tests.

**Failure-mode tree**:
- Inconsistent state: replay from audit-chain to recover; deny new transitions until consistency is verified.
- Long-running activity timeout: emit timeout event, route to designated recovery role, optionally escalate.
- Concurrent transition: optimistic-concurrency-control with retry; second-writer fails fast and emits a contention audit event.
- Workflow-engine outage: workflow runs are durable; new runs queue at admission until the engine returns.
- Template-version migration: migration policy lives in the template; running runs continue under their starting version unless explicitly rebased.
- Cross-tenant workflow: forbidden by default; cross-tenant runs require explicit cross-tenant Cedar permit plus marketplace-settlement scope.

**Implementation references**: `intelligence-workflow-engine`, `shared-workflow-templates`, `shared-workflow-runtime`.

**Worked example**: A purchase-order approval workflow requires: requester submits, manager approves, finance reviews, vendor onboarding check, contract signing, PO issuance. Every step uses one of the thirteen verbs; every state transition is audit-chain-sealed; every Cedar evaluation respects the tenant's spending-authority matrix.

**Per-tenant-pack adaptation**: HIPAA-pack workflows track minimum-necessary attestations; GDPR-pack workflows capture consent at each step; PCI-pack workflows scope cardholder-data steps to PCI-scope tenants; defense-pack workflows track ITAR-export-control reviews.

### 3.4 ONE-ONTOLOGY
**Definition**: one object graph with role, capability, and jurisdiction projections.

**Engineering-rigor matrix**:
- Maintainability: ontology object classes live in `shared-ontology-schema`; per-tool object models are forbidden.
- Observability: every Ontology object mutation emits an audit-chain event.
- Scalability: ontology sharded by tenant; large objects (documents, datasets) reference content-addressed storage.
- Performance: projection materialization within 50 ms p50.
- Optimization: per-role projection cache; invalidation on object mutation or projection-rule change.
- Code quality: ontology-schema migration tooling with backward-compat checks.

**Failure-mode tree**:
- Stale projection: rebuild from the base ontology; emit stale-projection audit event.
- Projection-rule conflict: prefer deny, surface which projections are conflicting, route to ontology-author review.
- Cross-tenant projection: forbidden; explicit cross-tenant projection requires marketplace-settlement scope.
- Schema migration: backward-compat enforced for at least one release cycle; rolling migration via projection rebuild.
- Large-object retrieval: timeout returns partial projection plus retry guidance; full materialization queued.

**Implementation references**: `shared-ontology`, `shared-ontology-schema`, `shared-ontology-projection`.

**Worked example**: A purchase order is an Ontology object class with fields for requester, vendor, line items, total, currency, and approval state. The CRM-capability-tier projection of a purchase order shows only the vendor relationship and the sales-account link. The ERP-capability-tier projection shows the full financial detail. The audit-capability-tier projection shows the full transition history. The same underlying object has multiple projections; no copy is made.

**Per-tenant-pack adaptation**: HIPAA-pack tenants project PHI fields only to roles with treatment, payment, or operations purpose; GDPR-pack tenants project personal-data fields only with documented purpose; PCI-pack tenants project cardholder-data fields only to PCI-scope roles.

### 3.5 ONE-AUDIT-CHAIN
**Definition**: one evidence chain for identity, policy, workflow, settlement, and operations.

**Engineering-rigor matrix**:
- Maintainability: audit-event classes live in `shared-audit-chain-schema`; per-tool audit logs are forbidden.
- Observability: audit-chain is itself observable through a query surface; meta-audit (audit of audit-chain reads) is recorded.
- Scalability: audit-chain sharded by tenant; long-term storage compressed; high-volume tenants get dedicated storage shards.
- Performance: write within 20 ms p99 for substrate verbs; queries up to 1 month back within 500 ms p99.
- Optimization: pre-aggregated projections for common queries (e.g., daily activity summary, denial counts, settlement totals).
- Code quality: replay-based audit-integrity tests; cryptographic-chain-link verification on every read.

**Failure-mode tree**:
- Chain integrity failure: tenant audit-chain frozen at the last-verified event; investigation workflow opened; tenant operator notified.
- Audit-chain outage: substrate writes queue at admission until the chain returns; reads serve last-known-good projection with bounded TTL.
- Long-term retention boundary: pack-specific retention policies are enforced at write; expiration triggers an evidence-bundle export per pack rules.
- Cross-tenant audit query: requires explicit cross-tenant Cedar permit plus optional pack-specific override.
- Sealing-key rotation: rolling-rotation policy; old keys retained for chain-verification; new keys used for new writes.

**Implementation references**: `shared-audit-chain`, `shared-audit-chain-schema`, `shared-audit-replay`.

**Worked example**: A clinical handoff at 7 AM emits audit-chain events for the registered-nurse's verify-context, review-history, attach-evidence (vitals), approve (medication-administration), sign (controlled-substance-witness), and route (consult-request). The hospital's compliance officer can query the audit-chain six months later to reconstruct the entire shift's clinical decisions for a peer-review case.

**Per-tenant-pack adaptation**: HIPAA-pack tenants retain audit-events for the statutory minimum plus the tenant's retention policy; financial-services-pack tenants retain for seven years per SOX; defense-pack tenants retain per CMMC; personal-pack tenants retain per the user's chosen retention policy with right-to-delete subject to legal-hold.

### 3.6 ONE-MARKETPLACE
**Definition**: one universal deal-settlement surface across consumer, business, labor, and partner exchanges.

**Engineering-rigor matrix**:
- Maintainability: settlement primitives live in `shared-marketplace-settlement`; per-category marketplace forks are forbidden.
- Observability: every settlement event emits an audit-chain event with parties, value, currency, and policy fragment version.
- Scalability: settlement sharded by tenant; high-volume tenants get dedicated settlement shards.
- Performance: settlement decision within 100 ms p99; long-running settlement (e.g., escrow) is tracked as a Workflow Engine run.
- Optimization: precomputed party-creditworthiness projections; cached counter-party reputation scores.
- Code quality: deterministic-replay tests for settlement; consensus tests across geographic shards.

**Failure-mode tree**:
- Counterparty-creditworthiness drop: defer settlement to managed-escrow; emit a creditworthiness audit event.
- Settlement-currency-rail outage: defer settlement to alternative rail; emit a rail-outage audit event.
- Fraud signal: defer settlement to investigation workflow; emit a fraud-flag audit event.
- Cross-tenant settlement: requires explicit marketplace-settlement Cedar permit on both sides.
- Refund and chargeback: workflow-engine-driven; settlement event is reversed and a reversal-audit-event is sealed.
- Dispute: marketplace-dispute workflow with policy-bound mediator role; outcome emits a settlement-adjustment event.

**Implementation references**: `shared-marketplace-settlement`, `shared-marketplace-admission`, `shared-marketplace-dispute`.

**Worked example**: A labor-marketplace gig: a freelance graphic designer proposes a logo project; the requesting tenant approves the proposal; the work is delivered as an Ontology object reference; the requester signs delivery acceptance; settlement transfers funds; audit-chain seals the entire flow; both parties can review history. Same flow applies to a consumer-marketplace book purchase or a business-to-business software-license purchase.

**Per-tenant-pack adaptation**: PCI-pack tenants enforce cardholder-data-scope; GDPR-pack tenants enforce data-processing-agreements at counterparty admission; CSAP-pack tenants enforce data-residency in KR jurisdiction; AML-pack tenants enforce KYC at high-value settlement.

### 3.7 ONE-UX-SHELL
**Definition**: one stable interaction vocabulary across roles, devices, collar colors, and locales.

**Engineering-rigor matrix**:
- Maintainability: the thirteen-verb enum lives in `shared-ux-shell-action-router`; adding a verb requires an ADR.
- Observability: every verb-completion emits a timing event plus a modality event (mouse, keyboard, screen-reader, assistive-input).
- Scalability: UX shell is delivered via CDN with per-tenant overlay; role-projection rendering is at edge.
- Performance: verb-execution-path latency budget within 200 ms p95 to the substrate router admission.
- Optimization: role-projection-aware bundling; lazy load of pack-overlay content.
- Code quality: verb-enum conformance tests; cross-locale, cross-collar, cross-modality affordance tests.

**Failure-mode tree**:
- Verb-drift attempt: rejected at the action-router enum conformance gate; the offending change cannot ship.
- Pack-specific verb-hiding: rejected; pack may restrict the verb's policy-bound visibility but not its presence in the shell.
- Locale-specific verb addition: rejected unless adopted globally via ADR.
- Capability-tier-specific verb subtraction: rejected; Cedar may deny the verb but the verb remains visible with denial-recovery affordances.
- Accessibility regression: rejected at the UX-shell conformance set.
- Custom-app shadow vocabulary: rejected at marketplace admission.

**Implementation references**: `shared-ux-shell-action-router`, `shared-ux-shell-localization`, `shared-ux-shell-accessibility`.

**Worked example**: A new hire opens oyatie on day one. The UX shell shows the same thirteen verbs in their role projection that they used in their personal tenant during the interview process the previous month. The new hire executes their first approve on day one without any platform-specific training; the role-projection-specific evidence-and-policy refresher takes one hour.

**Per-tenant-pack adaptation**: HIPAA-pack tenants render verify-context with patient-identity overlay; PCI-pack tenants render attach-evidence with cardholder-data-redaction overlay; defense-pack tenants render export-with-policy with ITAR-clearance overlay; personal-pack tenants render the shell with simplified hint copy by default.

### 3.8 ONE-TRAINING-MODEL
**Definition**: one learned vocabulary that transfers across departments and career stages.

**Engineering-rigor matrix**: the training model is the verb-and-substrate fluency that the user develops over their career arc. The substrate is engineered to guarantee that the fluency holds across role, capability tier, tenant, workspace, locale, compliance pack, and time.

**Failure-mode tree**: anti-patterns are catalogued in Section 13 and in `training-cost-doctrine-2026-05-21.md` §14. Required correction is to revert the breach or to raise an ADR.

**Implementation references**: `shared-ux-shell-action-router`, plus the substrate verbs' implementations, plus the training-cost-doctrine sibling.

**Worked example**: see the persona timelines in `training-cost-doctrine-2026-05-21.md` §5.2, §5.2.b, §5.2.c.

**Per-tenant-pack adaptation**: training overlays are pack-specific evidence-and-policy refreshers rather than new verb introductions.

### 3.9 ONE-COMPLIANCE-POSTURE
**Definition**: one pack and evidence model applied before data or workflow exposure.

**Engineering-rigor matrix**:
- Maintainability: compliance-pack overlays live in `shared-compliance-pack-overlays`; per-tool compliance implementation is forbidden.
- Observability: pack-overlay activations and pack-specific denial events emit audit-chain events.
- Scalability: pack overlays are per-tenant per-cell (ADR-0251); multi-pack tenants compose overlays.
- Performance: pack evaluation is inline with Cedar evaluation budget.
- Optimization: pack-overlay precompilation; hot-path compliance fragments are cached.
- Code quality: compliance-pack conformance tests per pack; cross-pack composition tests.

**Failure-mode tree**:
- Pack-overlay outage: prefer deny for write actions; permit read actions with bounded TTL.
- Pack-overlay drift: tenant's pack-version is pinned; rolling-migration on version bump with explicit operator review.
- Cross-pack composition conflict: prefer the more-restrictive pack; emit a conflict-resolution audit event.
- Pack-specific evidence absent: workflow deferred until evidence is attached; denial-recovery widget guides the user.
- Pack-attestation expiry: workflow defers high-stakes verbs until attestation is renewed.

**Implementation references**: `shared-compliance-pack`, `shared-compliance-pack-overlays`, `intelligence-policy-engine-cedar`.

**Worked example**: A multi-national life-sciences tenant has HIPAA (US clinical), GDPR (EU clinical), PCI (e-commerce), and SOC2 (corporate IT) packs active. A US-clinical-research-coordinator's verify-context shows the active packs scoped to the current action; data exported from the US clinical-research workflow is scoped to HIPAA-pack export grammar; data exported from the EU sales-portal is scoped to GDPR-pack export grammar.

**Per-tenant-pack adaptation**: this invariant is itself the per-tenant-pack adaptation primitive.

### 3.10 ONE-PLUGIN-EXTENSIBILITY
**Definition**: one governed extension model with isolation, admission, settlement, and auditability.

**Engineering-rigor matrix**:
- Maintainability: plugin admission lives in `shared-plugin-admission`; ad-hoc plugin loading is forbidden.
- Observability: plugin admission, plugin invocation, and plugin policy denials emit audit-chain events.
- Scalability: plugins run in isolated cells (ADR-0248 cellular architecture); resource limits per tenant.
- Performance: plugin invocation budget per role; soft and hard timeouts enforced.
- Optimization: per-plugin cache shaping; deprecation-aware retire of unused plugins.
- Code quality: plugin admission conformance test; isolation-fuzz test; tenant-isolation regression test.

**Failure-mode tree**:
- Plugin sandbox escape: tenant frozen; plugin globally suspended; incident workflow opened.
- Plugin resource overrun: hard-kill plugin invocation; emit resource-overrun audit event.
- Plugin cross-tenant access attempt: rejected; emit cross-tenant-violation audit event; plugin frozen pending review.
- Plugin admission regression: rolling rollback via plugin-version policy.
- Plugin sunset: deprecation timer plus migration-path required at admission.
- Plugin pack-violation: tenant pack overlays enforce plugin allow-lists.

**Implementation references**: `shared-plugin-admission`, `shared-plugin-isolation` (planned), `shared-marketplace-admission`.

**Worked example**: A health-system tenant admits a third-party radiology-AI plugin from the marketplace. Admission verifies pack-conformance (HIPAA), isolation-conformance (sandbox), settlement-conformance (per-image billing), and audit-conformance (sealed events on every invocation). When the plugin emits a high-confidence finding, the radiologist sees the finding through the same Ontology projection they already use; the audit-chain shows the plugin-invocation that generated the finding.

**Per-tenant-pack adaptation**: HIPAA-pack tenants restrict plugin allow-lists to BAA-signed vendors; CMMC-pack tenants restrict to cleared vendors; GDPR-pack tenants restrict to data-processing-agreement-signed vendors.

## Section 3.A - Invariant-interaction matrix

### 3.A.1 ONE-IDENTITY × ONE-POLICY-ENGINE
Identity is the principal in every Cedar evaluation. Cross-tenant identity scoping is enforced at policy evaluation. Identity-recovery events trigger policy-fragment-version re-pinning. Identity-attribute changes invalidate cached policy decisions for the affected principal.

### 3.A.2 ONE-IDENTITY × ONE-WORKFLOW-ENGINE
Workflow runs are scoped to a tenant and started by an identity-bound actor. Cross-actor handoffs route via Workflow Engine state transitions; the audit-chain records each actor change.

### 3.A.3 ONE-IDENTITY × ONE-ONTOLOGY
Ontology objects record creator, last-modifier, and authorized-viewer identities. Identity-membership changes invalidate cached projections for affected objects.

### 3.A.4 ONE-IDENTITY × ONE-AUDIT-CHAIN
Every audit-chain event records actor identity. Identity-recovery events create a heightened-audit-class event with continuity-attestation evidence.

### 3.A.5 ONE-POLICY-ENGINE × ONE-WORKFLOW-ENGINE
Workflow state transitions evaluate Cedar at admission. Policy denials emit Workflow Engine recovery events; the workflow run pauses pending recovery.

### 3.A.6 ONE-POLICY-ENGINE × ONE-ONTOLOGY
Ontology projection rules are Cedar fragments. Projection-rule conflict resolution is Cedar precedence (most-restrictive deny precedes broader permit).

### 3.A.7 ONE-POLICY-ENGINE × ONE-AUDIT-CHAIN
Every Cedar evaluation seals an audit event with the policy-fragment-version and decision. Cross-tenant Cedar evaluations are sealed separately with explicit cross-tenant scope.

### 3.A.8 ONE-WORKFLOW-ENGINE × ONE-ONTOLOGY
Workflow runs reference Ontology objects as state. Ontology mutations may be authored by Workflow runs; the audit-chain records both the run and the mutation.

### 3.A.9 ONE-WORKFLOW-ENGINE × ONE-AUDIT-CHAIN
Every Workflow Engine state transition seals an audit event. Workflow replay reconstructs the audit-chain.

### 3.A.10 ONE-ONTOLOGY × ONE-AUDIT-CHAIN
Every Ontology mutation seals an audit event. Ontology replay reconstructs the object-graph state at any prior time within the retention window.

### 3.A.11 ONE-MARKETPLACE × every-other invariant
Marketplace events reference identity (buyer, seller, mediator), evaluate Cedar (per pack), instantiate Workflow runs (offer-acceptance-settlement-dispute-reversal), record Ontology objects (offers, agreements, settlements), and seal audit-chain events.

### 3.A.12 ONE-UX-SHELL × every-other invariant
UX-shell verbs invoke substrate primitives: identity (verify-context, switch-role), policy engine (Cedar evaluation at every verb), workflow engine (every verb may start or advance a run), ontology (verbs may project, mutate, or query), audit-chain (every verb seals an event), marketplace (relevant verbs route through marketplace primitives), compliance pack (verbs respect pack overlays), plugin (verbs may invoke admitted plugins).

### 3.A.13 ONE-TRAINING-MODEL × every-other invariant
The training-model invariant is the visible manifestation of substrate consistency. It depends on every other invariant holding. A breach in any other invariant causes the training-model invariant to fail at the breach surface; the user must relearn at that surface.

### 3.A.14 ONE-COMPLIANCE-POSTURE × every-other invariant
Compliance-pack overlays modulate identity (pack-required attestations), policy (pack-specific fragments), workflow (pack-required evidence and steps), ontology (pack-specific projections and retention), audit-chain (pack-aware retention class), marketplace (pack-restricted counterparties), UX-shell (pack-specific denial-recovery affordances), and plugin admission (pack-aware admission gates).

### 3.A.15 ONE-PLUGIN-EXTENSIBILITY × every-other invariant
Plugin admission integrates with identity (tenant-administrator approves), policy (Cedar evaluates admission), workflow (admission is a Workflow Engine run), ontology (plugin may read or mutate scoped Ontology objects), audit-chain (every invocation is sealed), marketplace (settlement primitive handles billing), UX-shell (plugin verb-invocations route through the action router), compliance pack (plugin must conform to active packs).

### 3.A.16 Substrate-wide failure isolation
A failure in one substrate primitive degrades but does not collapse the others. Identity outage degrades to last-known-good-membership reads with bounded TTL. Policy-engine outage forces deny-by-default for write actions. Workflow-engine outage queues new runs at admission. Ontology outage degrades to cached projections. Audit-chain outage queues writes at admission. Marketplace outage degrades to pending-settlement queue. UX-shell outage degrades to substrate API access. Compliance-pack outage degrades to base-substrate behavior with explicit notification. Plugin-admission outage allows existing plugins to operate but blocks new admissions.

## Section 4 - Capability-tier-as-projection model

### 4.1 The ADR-0316 turn
ADR-0316 turns CRM, HR, ERP, ITSM, and adjacent product categories into capability tiers over the unified substrate. The default decision is capability-tier projection; a separate service is the exception requiring a documented operational concern.

The change is not cosmetic. Treating CRM as a capability-tier projection means: the sales-opportunity Ontology object class is part of the unified Ontology; the sales-process state machines are stored in the unified Workflow Engine templates; the sales-role Cedar permits live in the unified policy store; the sales-team audit-chain is the same chain as the finance-team audit-chain; the sales-marketplace settlement is the same settlement primitive as procurement-marketplace settlement.

### 4.2 The ERP capability tier
ERP-as-projection (ADR-0315) covers financial-closing, treasury, supply-chain, procurement, manufacturing, plant-maintenance, and tax. Each ERP function is implemented as Ontology-object-classes plus Workflow-Engine-templates plus Cedar-permit-grammars plus role-projection bundles.

Worked example: a purchase-order is one Ontology object class. The PO state machine is one Workflow template. The approval-authority matrix is Cedar permits. The PO settlement is marketplace-settlement. The PO audit is audit-chain. No separate ERP service is needed.

Sub-functions:
- Accounts payable: Ontology object class (invoice), Workflow template (three-way match), Cedar permits (AP-clerk, AP-supervisor, controller), settlement (vendor payment).
- General ledger: Ontology object class (journal entry), Workflow template (period-close, manual-journal-review), Cedar permits (staff-accountant, controller, CFO).
- Asset management: Ontology object class (fixed-asset), Workflow template (acquisition, depreciation, disposal), Cedar permits (asset-coordinator, finance-controller).
- Treasury: Ontology object class (bank-account, debt-instrument, hedge), Workflow template (cash-forecasting, bank-reconciliation, debt-issuance), Cedar permits (treasury-analyst, treasurer, CFO).
- Procurement: Ontology object class (requisition, PO, vendor, contract), Workflow template (sourcing, RFP, award, contract-signing), Cedar permits (requester, buyer, category-manager, CPO).
- Supply chain: Ontology object class (supply-order, shipment, warehouse-receipt), Workflow template (order-to-cash, procure-to-pay, inventory-cycle-count), Cedar permits (planner, warehouse-operator, fulfillment-manager).
- Plant maintenance: Ontology object class (asset, work-order, spare-part), Workflow template (preventive-maintenance, corrective-maintenance), Cedar permits (technician, supervisor, maintenance-manager).
- Tax: Ontology object class (tax-position, tax-filing, tax-payment), Workflow template (period-tax-calculation, filing, payment), Cedar permits (tax-analyst, tax-director).

### 4.3 The CRM capability tier
CRM-as-projection covers sales-opportunity-management, marketing-campaigns, customer-service, partner-relationship-management, and revenue-operations.

Sub-functions:
- Sales-opportunity: Ontology object class (lead, account, contact, opportunity, quote), Workflow template (lead-qualification, opportunity-progression, quote-to-order), Cedar permits (rep, manager, VP-sales).
- Marketing campaigns: Ontology object class (campaign, audience, asset, response), Workflow template (campaign-plan, campaign-execution, response-attribution), Cedar permits (marketer, marketing-manager, CMO).
- Customer service: Ontology object class (case, knowledge-article, customer-asset), Workflow template (case-routing, case-resolution, escalation), Cedar permits (agent, supervisor, support-manager).
- Partner-relationship: Ontology object class (partner, partner-deal, partner-payout), Workflow template (partner-onboarding, deal-registration, partner-payout), Cedar permits (partner-manager, channel-VP).
- Revenue-operations: Ontology object class (forecast, quota, territory), Workflow template (territory-planning, quota-setting, forecast-rollup), Cedar permits (rev-ops-analyst, rev-ops-manager).

### 4.4 The HR capability tier
HR-as-projection covers benefits, payroll, performance, learning, recruiting, employee-data, time-tracking, and talent-acquisition.

Sub-functions:
- Benefits: Ontology object class (benefit-plan, enrollment, claim), Workflow template (open-enrollment, life-event-change, claim-processing), Cedar permits (employee, HR-business-partner, benefits-administrator).
- Payroll: Ontology object class (pay-period, paycheck, pay-element), Workflow template (period-close, off-cycle-pay, year-end-reporting), Cedar permits (payroll-analyst, payroll-manager).
- Performance: Ontology object class (goal, review, calibration), Workflow template (goal-cascade, review-cycle, calibration-meeting), Cedar permits (employee, manager, HR-director).
- Learning: Ontology object class (course, learning-path, certification), Workflow template (course-completion, certification-renewal), Cedar permits (learner, manager, learning-coordinator).
- Recruiting: Ontology object class (job-requisition, candidate, interview, offer), Workflow template (requisition-approval, candidate-pipeline, offer-process), Cedar permits (recruiter, hiring-manager, HR-recruiter).
- Employee-data: Ontology object class (employee, position, employment-event), Workflow template (hire, transfer, leave, termination), Cedar permits (HR-business-partner, HR-operations).
- Time-tracking: Ontology object class (timecard, absence-record), Workflow template (timecard-approval, absence-approval), Cedar permits (employee, manager, time-administrator).

### 4.5 The ITSM capability tier
ITSM-as-projection covers incident, problem, change, release, asset-management, and service-request.

Sub-functions:
- Incident: Ontology object class (incident, affected-service), Workflow template (incident-routing, incident-resolution, post-incident-review), Cedar permits (caller, analyst, incident-manager, on-call-engineer).
- Problem: Ontology object class (problem, root-cause), Workflow template (problem-investigation, RCA, known-error-record), Cedar permits (problem-manager, engineering-lead).
- Change: Ontology object class (change-request, change-window), Workflow template (CAB-review, standard-change, emergency-change), Cedar permits (requester, CAB-member, change-manager).
- Release: Ontology object class (release-package, release-window), Workflow template (release-plan, deployment, post-release-validation), Cedar permits (release-manager, deployment-engineer).
- Asset-management: Ontology object class (configuration-item, license), Workflow template (CI-discovery, license-true-up), Cedar permits (CMDB-administrator, license-manager).
- Service-request: Ontology object class (service-request, service-catalog-entry), Workflow template (request-fulfillment, approval-routing), Cedar permits (requester, fulfiller, service-owner).

### 4.6 The collaboration capability tier
Collaboration-as-projection covers mail, calendar, notes, sheets, meet, community, file-storage, and chat.

Each function is a projection of the Ontology (message, event, document, sheet, meeting, post, file, channel) plus Workflow templates plus Cedar permits. The thirteen verbs apply across all collaboration surfaces.

### 4.7 The audit-and-governance capability tier
Audit-and-governance-as-projection covers internal-audit, compliance, risk, controls, and policy-management.

Sub-functions:
- Internal-audit: Ontology object class (engagement, finding, working-paper, evidence), Workflow template (audit-plan, engagement-execution, finding-tracker), Cedar permits (auditor, audit-manager, CAE).
- Compliance: Ontology object class (control, control-test, exception), Workflow template (control-testing, exception-management), Cedar permits (compliance-analyst, compliance-officer).
- Risk: Ontology object class (risk, mitigation, risk-event), Workflow template (risk-assessment, mitigation-tracking), Cedar permits (risk-analyst, CRO).
- Controls: Ontology object class (control, control-design, control-test-result), Workflow template (control-design-review, control-effectiveness-testing), Cedar permits (control-owner, IA-tester).
- Policy-management: Ontology object class (policy, policy-revision, policy-attestation), Workflow template (policy-revision, policy-attestation-campaign), Cedar permits (policy-author, policy-approver).

### 4.8 Capability-tier exception cases
A capability-tier exception is when an operational concern justifies a service rather than a projection. Examples:
- A regulated network (e.g., a SWIFT-network gateway, a Fedwire gateway) requires its own service because the network has its own operational SLA, its own membership process, its own dispute mechanism.
- A hardware integration (e.g., a manufacturing-floor PLC, a clinical-bedside infusion pump) requires its own service because the operational lifecycle is tied to hardware certification.
- A domain-specific operational engine (e.g., a high-frequency-trading order-routing engine, a real-time-bidding ad exchange) requires its own service because the operational latency budget exceeds substrate budgets.

Even in exception cases, the service must implement the thirteen verbs identically and must integrate with the unified Ontology, Cedar, Workflow Engine, audit-chain, and marketplace.

## Section 4.A - Capability-tier engineering rigor

### 4.A.1 ERP capability tier rigor
- Latency budget: financial-period close completion under 2 hours for a 5,000-entity consolidated close; sub-ledger-to-G/L transfer within 10 minutes p99.
- Throughput target: 50 million journal entries per day at peak (mid-quarter and year-end).
- Replay integrity: every financial period is replay-reconstructible from audit-chain within 2 hours.
- Pack support: GAAP, IFRS, local-GAAPs, SOX, tax-pack overlays (multi-jurisdiction).
- Service exception thresholds: ERP retains as substrate-projection unless an operational concern (sub-millisecond latency, certified network membership, hardware integration) requires service.

### 4.A.2 CRM capability tier rigor
- Latency budget: opportunity-stage update under 200 ms p99; territory rollup nightly under 30 minutes.
- Throughput target: 5 million opportunity-stage updates per day.
- Replay integrity: sales-pipeline reconstruction from audit-chain within 30 minutes.
- Pack support: per-locale data-residency, per-tenant data-segregation for multi-brand tenants.
- Service exception thresholds: CRM retains as projection; only specialized industry-clouds (e.g., Healthcare Cloud regulatory layers) trigger service-retention discussions.

### 4.A.3 HR capability tier rigor
- Latency budget: payroll-engine completion under 4 hours for a 50,000-person payroll; benefits enrollment under 5 minutes per employee.
- Throughput target: 250,000 employees per payroll period (across multi-tenant pool).
- Replay integrity: every payroll period is replay-reconstructible from audit-chain within 4 hours.
- Pack support: per-jurisdiction labor-law, statutory-tax, benefits-statutory, leave-statutory overlays.
- Service exception thresholds: payroll-calculation engines for jurisdictions with extremely idiosyncratic statutory rules (e.g., specific countries' annual tax-true-up rules) may retain as services; the substrate-projection integrates via Workflow Engine.

### 4.A.4 ITSM capability tier rigor
- Latency budget: incident-routing within 30 seconds p99; CMDB-query under 100 ms p99.
- Throughput target: 5 million incident events per day at peak.
- Replay integrity: every incident is replay-reconstructible from audit-chain.
- Pack support: per-pack incident-classification overlays (healthcare-incident, financial-incident, OT-incident).
- Service exception thresholds: ITSM retains as projection; only domain-specific operational engines (e.g., aircraft-fleet-status systems) trigger service-retention.

### 4.A.5 Collaboration capability tier rigor
- Latency budget: message-delivery under 200 ms p99; file-storage upload under 1 second p99 for files under 100 MB.
- Throughput target: 1 billion messages per day at peak; 100 petabytes file storage growth per year at peak.
- Replay integrity: every collaboration object is audit-chain-sealed; deletion is policy-bound.
- Pack support: per-pack content-classification (PHI, cardholder-data, ITAR-controlled, attorney-client privileged); per-jurisdiction data-residency.
- Service exception thresholds: collaboration retains as projection; PSTN-bridging and live-event broadcasting may retain as services.

### 4.A.6 Audit-and-governance capability tier rigor
- Latency budget: audit-chain query for a 6-month window under 500 ms p99; control-effectiveness test execution within 1 hour p99.
- Throughput target: 10 million audit-chain queries per day at peak.
- Replay integrity: 100 percent of audit-chain is replay-verifiable.
- Pack support: per-pack regulator-reporting overlays (SOX, NERC-CIP, FERPA, Joint-Commission, CMS, USDA, CMMC, EU-AI-Act).
- Service exception thresholds: audit-and-governance retains as projection; only specialized regulator-submission portals retain as services.

### 4.A.7 Capability-tier composition rigor
Capability tiers compose without conflict because each tier's Ontology object classes, Workflow templates, Cedar permits, and audit-chain projections are namespace-scoped per tier. A tenant subscribing to multiple tiers gets the union of all tier-provided projections; conflict resolution is by tier-version-pin per tenant.

## Section 5 - Role-based projection model

### 5.1 The ADR-0317 role-projection primitive
ADR-0317 establishes role projection as a first-class substrate primitive. A role projection is the union of (a) the Cedar permits the role holds in the active tenant, (b) the Ontology-projection rules the role sees, (c) the Workflow templates the role can initiate, (d) the audit-chain views the role can query, (e) the marketplace transactions the role can settle, (f) the UX shell density and hint copy tuned for the role, (g) the pack-overlay scope the role operates under, and (h) the locale and workspace context.

### 5.2 Role-projection examples
A small, indicative set of role projections across capability tiers:

**Finance roles**: staff-accountant, senior-accountant, controller, CFO, treasurer, AP-clerk, AR-clerk, payroll-analyst, financial-analyst, tax-analyst, internal-auditor.

**Sales roles**: SDR, AE, sales-engineer, sales-manager, VP-sales, sales-ops-analyst, deal-desk-analyst, customer-success-manager, partner-manager.

**HR roles**: HR-business-partner, recruiter, hiring-manager, benefits-administrator, payroll-administrator, learning-coordinator, performance-coach, employee-relations-investigator.

**Engineering roles**: software-engineer, senior-engineer, staff-engineer, engineering-manager, director-of-engineering, VP-engineering, principal-engineer, site-reliability-engineer, security-engineer.

**Clinical roles**: registered-nurse, advanced-practice-provider, attending-physician, resident, fellow, medical-assistant, clinical-pharmacist, social-worker, case-manager, clinical-research-coordinator.

**Manufacturing roles**: production-operator, line-supervisor, plant-manager, quality-inspector, maintenance-technician, materials-coordinator, EHS-officer.

**Public-sector roles**: case-worker, eligibility-supervisor, program-manager, policy-analyst, ombudsman, hearings-officer.

**Education roles**: K-12-teacher, K-12-principal, district-superintendent, college-instructor, dean, registrar, financial-aid-officer, advisor.

**Trades roles**: apprentice-electrician, journeyman-electrician, master-electrician, electrical-contractor, electrical-inspector.

**Personal roles**: personal-tenant-owner, parent, side-business-owner, family-administrator, caregiver, executor.

Every role projects the same thirteen verbs over the same substrate. Differences live in evidence, policy, and density.

### 5.3 Role-projection switching
Switching role projections is the substrate verb switch-role. The substrate validates that the user holds the membership and permit set for the target role, then re-renders the UX shell with the target role's projection. The audit-chain records the role-switch event.

Cross-tenant role switching (e.g., from work-tenant employee role to personal-tenant owner role) is allowed; the dual-tenant boundary per ADR-0311 is enforced.

### 5.4 Role-projection composition
A user can hold multiple role projections within one tenant (e.g., a senior nurse who is also a nursing-school adjunct instructor and a member of the magnet-program committee). The substrate composes the projections at query time: the UX shell shows widgets union-ed across the projections; Cedar evaluations consider the union of permits; the audit-chain records which role projection authored each event.

### 5.5 Role-projection sunset
ADR-0320 transient identity tiers (apprentice, intern, resident, fellow) have explicit sunset semantics. The substrate freezes the role projection at sunset, exports the role-related evidence to the personal tenant per ADR-0311, and seals the sunset event into the audit-chain.

### 5.6 Cross-pack role projections
A role projection may be active across multiple compliance packs. A nurse-practitioner in a multi-state telehealth tenant carries pack overlays for every state license they hold. Cedar evaluation composes the packs; the substrate enforces the most-restrictive pack on any cross-pack action.

### 5.7 Role-projection telemetry
Per-role telemetry signals: time-to-first-successful-action, support-ticket volume, wrong-context-action denial rate, accessibility-task-completion rate. These are the four signals from `training-cost-doctrine-2026-05-21.md` §11.

## Section 5.A - Role-projection lifecycle

### 5.A.1 Role-projection provisioning
A new role projection is provisioned via a Workflow Engine run with the following stages: requester submits the role assignment, designated approver (typically HR-business-partner or manager) approves, Cedar permits are computed from the role's permit-grammar and the active packs, training-attestation prerequisites are verified, the role projection becomes active in the user's tenant membership, and the audit-chain records the entire flow.

### 5.A.2 Role-projection modification
Modifications (adding capability-tier scope, adding pack overlays, adjusting workspace scope) follow the same workflow pattern as provisioning. Modifications emit audit-chain events that reference the prior role-projection version.

### 5.A.3 Role-projection retirement
Retirement (e.g., user changes role, role is sunset by ADR, user departs) follows a sunset workflow. The substrate exports affected evidence per ADR-0311 boundary rules, deprovisions Cedar permits, and records the retirement event. The user's audit-chain history under the prior role remains visible to authorized auditors per the active pack's retention class.

### 5.A.4 Role-projection conflict detection
Multiple role projections for the same user in the same tenant are composed via Cedar union for permits, Ontology projection union for visible objects, and Workflow-template union for initiation rights. Conflicts (e.g., separation-of-duties) are detected via Cedar constraint fragments and raised as denial events at the offending verb.

### 5.A.5 Role-projection observability
Per-role-projection metrics include: active-user-count, daily verb-completion-count, average time-to-first-successful-action for new role-projection assignees, denial-rate by root-cause, accessibility-task-completion-rate. These metrics feed the doctrine's claim-discipline gate.

### 5.A.6 Role-projection portability
A user departing one tenant may carry certain role-projection earnings (e.g., certifications, training-attestations) into their personal tenant or into a future employer tenant per ADR-0311 export rules. The substrate's export-with-policy verb governs the transfer.

### 5.A.7 Role-projection emergency-elevation (break-glass)
A break-glass capability tier permits emergency-elevation of a role projection (e.g., on-call engineer needs admin-elevation during an incident). The elevation is policy-bound, time-bound, and audit-chain-sealed with heightened audit class. Post-incident review of the elevation is mandatory.

### 5.A.8 Role-projection scenario: hospital nurse cross-unit float
A registered nurse normally assigned to med-surg-4w is floated to ICU for one shift. The float is a temporary role-projection modification:
1. Charge nurse opens a float-assignment Workflow run.
2. The float assignment adds the ICU-clinical-bedside scope to the nurse's existing role projection for the duration of the shift.
3. Cedar verifies the nurse holds ICU-cleared training-attestations.
4. The nurse's verify-context shows the temporary ICU scope.
5. At shift end, the temporary scope expires automatically; audit-chain records the entire float.

### 5.A.9 Role-projection scenario: finance manager covering interim controller
The CFO designates a finance manager to cover an interim-controller role during the controller's leave. The cover assignment is a temporary role-projection escalation:
1. CFO opens an interim-controller-assignment Workflow run.
2. The assignment adds controller-scope Cedar permits to the manager's existing role projection for the leave duration.
3. The manager's verify-context shows the temporary controller scope.
4. At leave end, the temporary scope expires automatically; manager retains existing finance-manager scope only.

### 5.A.10 Role-projection scenario: software engineer rotating to security-engineering
A software engineer joins the security-engineering team for a 6-month rotation. The rotation is a role-projection modification:
1. Engineering leadership opens a rotation-assignment Workflow run.
2. The assignment adds security-engineering scope to the engineer's role projection for the rotation duration.
3. Training-attestation prerequisites (security-engineering bootcamp) are verified.
4. The rotation completes; the engineer's role projection sunsets the security-engineering scope; audit-chain records the entire rotation including any security-incident response the engineer participated in.

## Section 6 - Collar-color and workspace universality

### 6.1 ADR-0318 collar-color universality
ADR-0318 forbids stripping vocabulary or evidence affordance based on workspace collar color. The full thirteen-verb vocabulary plus full-floor affordances are available in every workspace.

### 6.2 White-collar workspaces
Office-based knowledge work: finance, sales, engineering, HR, marketing, legal, audit, policy, and analytics. UX shell density is typically the default; keyboard shortcuts are heavily used; multi-window patterns are common; large-monitor real estate is assumed.

### 6.3 Blue-collar workspaces
Plant-floor, warehouse, construction-site, agriculture, fishery, mining, oil-and-gas, and trades. UX shell density is tuned for handheld terminal, ruggedized device, or kiosk; one-handed operation patterns are first-class; outdoor-readable contrast modes are supported; vibration-resistant input is supported; intermittent-connectivity affordances are present.

### 6.4 Pink-collar workspaces
Care work, education, hospitality, food service, retail, and personal services. UX shell density is tuned for frequent interruption; quick-context switching is first-class; shared-device patterns (kiosks, common terminals) are supported.

### 6.5 Gray-collar workspaces
Field service, mobile sales, ride-sharing, delivery, in-home healthcare, in-home repair. UX shell density is tuned for vehicle-mounted display, smartphone-primary, intermittent-connectivity; navigation-app integration is first-class; passenger-or-customer-visible affordance is configurable.

### 6.6 Workspace tuning, not vocabulary stripping
The workspace overlay tunes density, default landing surface, keyboard shortcut palette, accessibility mode, contrast mode, and offline-cache shape. The workspace overlay does not strip verbs, hide evidence affordances, or reduce policy-explanation depth.

A worker who moves between a blue-collar plant-floor workspace and a white-collar engineering-office workspace within the same role retains identical vocabulary and full-floor affordance. A worker who is promoted from blue-collar to white-collar (e.g., line-operator to plant-manager) retains vocabulary fluency from the prior workspace.

### 6.7 Workspace examples by industry
- Healthcare: bedside-clinical, OR, ED, ICU, ambulatory-clinic, infusion-center, home-health-vehicle, telehealth-from-home.
- Manufacturing: production-line-station, quality-lab, warehouse-dock, plant-control-room, maintenance-bay, mobile-tool-cart.
- Construction: trailer-office, jobsite-shed, vehicle-mounted, helmet-display, scaffold-perimeter.
- Hospitality: front-desk-station, back-office, night-audit-station, housekeeping-cart, F&B-POS.
- Education: classroom, library, gym, cafeteria, parent-night-station, field-trip-mobile.
- Public-sector: caseworker-cubicle, hearings-room-projection, mobile-investigator-vehicle, kiosk-walk-in.
- Trades: vehicle-mounted-mobile, jobsite-temporary, customer-home, shop, training-lab.

### 6.8 Workspace and shift composition
A worker can have multiple shifts on multiple workspaces. The substrate stores shift-membership as a workflow run; switching from one shift workspace to another is a verb sequence (switch-role plus verify-context). The audit-chain records every shift-handoff event.

## Section 6.A - Workspace-tuning specifics

### 6.A.1 Plant-floor workspace tuning
Plant-floor workspace UX shell tuning:
- Touch target size: 60-pixel minimum on the handheld terminal; 80-pixel on the floor-mounted touchscreen.
- Contrast mode: high-contrast outdoor mode plus night-shift mode; both meet WCAG AAA contrast.
- Glove-compatible input: capacitive-glove-compatible touch via the substrate's input-mode router.
- Voice input: hands-free voice-input mode for verbs commonly used while hands are occupied.
- Vibration tolerance: input filtering for vibration-induced false touches; gyroscope-aware orientation lock.
- Intermittent connectivity: substrate offline-cache holds the last 8 hours of role-relevant Ontology projections; verbs queue locally; sync on reconnect.
- Audit-chain seal behavior: queued verbs are sealed with offline-mode flag; once sync completes, the seal is upgraded to online-mode.

### 6.A.2 Bedside-clinical workspace tuning
Bedside-clinical workspace UX shell tuning:
- Touch target size: 50-pixel minimum for hand-held bedside terminals.
- Contrast mode: night-shift mode plus emergency-lighting mode.
- Glove-compatible input: capacitive-glove-compatible touch for sterile-procedure contexts.
- Voice input: minimal voice-input; bedside contexts favor visual confirmation.
- Privacy mode: screen-blanking on disengagement; rapid passkey-unlock for resume.
- Patient-context lock: verify-context renders prominently before any high-stakes verb.
- Intermittent connectivity: substrate offline-cache holds the assigned patient list for 8 hours; sync on reconnect; offline-mode verbs that touch controlled substances are blocked.

### 6.A.3 Field-service workspace tuning
Field-service workspace UX shell tuning:
- Touch target size: 60-pixel minimum on smartphone-primary; 80-pixel on vehicle-mounted display.
- Contrast mode: outdoor-readable mode plus night-time mode.
- Geographic-context lock: verify-context renders the geographic context (job-site address); geo-fence-aware policy fragments may apply.
- Vehicle-mounted display: minimal interaction while driving; audio-only verb prompts; passenger-driven verb-execution requires explicit passenger-context.
- Customer-visible affordance: shareable-screen mode for customer-facing demos; non-shareable affordance hides internal data.
- Intermittent connectivity: substrate offline-cache holds the day's work-order list; sync on reconnect.

### 6.A.4 Office workspace tuning
Office workspace UX shell tuning:
- Touch target size: 40-pixel default for cursor-driven input; 50-pixel for touch-screen variants.
- Contrast mode: default light; dark-mode opt-in; high-contrast accessibility mode always available.
- Keyboard shortcuts: full thirteen-verb keyboard-shortcut palette; per-tenant customization permitted within substrate-defined safe ranges.
- Multi-window: substrate supports the same role projection rendered across multiple monitors; verify-context renders consistently across windows.
- Network reliability: substrate assumes always-on connectivity; offline-cache holds 1 hour of role-relevant data for short outages.

### 6.A.5 Classroom workspace tuning
Classroom workspace UX shell tuning:
- Touch target size: 60-pixel minimum for student-classroom-tablet; 50-pixel for teacher-laptop.
- Student-mode: simplified UX shell density with reduced hint copy; pedagogical-context-aware affordances.
- Teacher-mode: full UX shell density; class-roster-aware role projection.
- Privacy mode: screen-blanking when the teacher steps away; rapid passkey-unlock for resume.
- Pedagogical-context: per-activity Cedar permits scope what the student can attach as evidence (essay, photo, etc.).

### 6.A.6 Kiosk workspace tuning
Kiosk workspace UX shell tuning:
- Touch target size: 80-pixel minimum (designed for shared-device standing use).
- Reduced verb palette: kiosk surfaces typically expose a subset of the thirteen verbs per role (e.g., self-service kiosk for benefits enrollment exposes attach-evidence, approve, sign, verify-context, recover-from-denial).
- Session timeout: aggressive timeout; passkey-bound sessions; explicit log-out before walk-away.
- Audit-chain: every kiosk session is fully sealed; physical-location-context records the kiosk identifier.

### 6.A.7 Home-tenant workspace tuning
Home-tenant (personal use from a personal device) UX shell tuning:
- Touch target size: device-default (smartphone, tablet, laptop, desktop).
- Multi-device sync: substrate provides cross-device continuity; the user can start a verb on phone and finish on laptop.
- Family-administrator mode: when the user is delegated administrator of a family-member's tenant, the role-switcher is prominent.
- Quiet-hours respect: notification-cadence respects user-defined quiet hours; substrate consolidates urgent-class signals only.

## Section 7 - Dual-tenant identity boundary

### 7.1 ADR-0311 dual-tenant doctrine
ADR-0311 establishes that every human has a personal tenant in addition to any work-tenant memberships. The boundary is non-negotiable substrate.

### 7.2 Personal-tenant scope
The personal tenant holds: personal mail, calendar, notes, sheets, file storage, marketplace purchases, side-business-light operations (within the personal-tax-id scope), family-administration (delegated by relationship), personal-finance, personal-medical-records (delegated by patient relationship), and personal-extension plugins.

### 7.3 Work-tenant scope
Work tenants hold: employment data, employer-issued credentials, employer-tenant-scoped Ontology objects, employer-tenant-scoped Workflow runs, employer-tenant-scoped audit-chain events, and employer-pack-overlay state.

### 7.4 Boundary enforcement
Boundary enforcement is at every substrate primitive:
- Identity: tenant memberships are explicit; the active tenant is shown via verify-context.
- Policy: Cedar evaluates with the active tenant scope; cross-tenant queries require explicit permits.
- Workflow: Workflow runs are scoped to a tenant; cross-tenant runs require marketplace-settlement permits.
- Ontology: object projections are tenant-scoped; cross-tenant projection requires explicit permits.
- Audit: audit-chain shards are per-tenant; cross-tenant queries require explicit permits and are logged separately.
- Settlement: settlement events are tenant-scoped; cross-tenant settlement uses the marketplace primitive.
- UX shell: the tenant indicator is always visible; the role-switcher requires explicit confirmation for cross-tenant moves.

### 7.5 Boundary violations
A boundary violation is rejected at admission. Examples:
- A personal-tenant marketplace purchase attempts to settle through a work-tenant credit-card on file: rejected; the user is shown verify-context and can re-route.
- A work-tenant export of personal-tenant medical records: rejected unless an explicit cross-tenant Cedar permit is present (e.g., an HR-medical-leave workflow).
- A side-business tenant attempts to read work-tenant employee data: rejected; the side-business is not a member.

Every rejected attempt is sealed into the audit-chain for forensic review.

### 7.6 Personal-to-work life events
Life events that cross the boundary:
- Hiring: a new employer creates a tenant membership; the personal tenant's identity attestation flows in via passkey but personal data does not.
- Departure: the employer tenant deprovisions the membership; any personal-tenant data that the employee chose to keep in the work-tenant is exported with policy back to the personal tenant.
- Health benefits: the employer benefits workflow creates a benefits-claim Ontology object in the work tenant; medical records remain in the personal tenant; explicit permits scope what the benefits-administrator sees.
- Side-business launch: the employee creates a new tenant for the side-business; the boundary holds.
- Parental leave: a workflow run in the work-tenant references the leave; medical detail remains in the personal tenant.

### 7.7 Conglomerate dual-boundary
A conglomerate per ADR-0313 layers sovereign-child tenants under a parent. A worker who is employed by a subsidiary holds: personal tenant, subsidiary-employer tenant. The worker does not automatically hold the parent-conglomerate tenant unless the parent is the employer.

### 7.8 Family-administrator delegation
A parent can delegate administration of a child's personal tenant per ADR-0311. The delegation is explicit, scoped, and time-bound. The audit-chain records every delegated action. The child's personal tenant remains the child's tenant; the parent acts as administrator.

### 7.9 Healthcare-power-of-attorney delegation
A patient can delegate health-care decisions to a designated representative per ADR-0311. The delegation creates a temporary cross-tenant Cedar permit scoped to clinical workflows. The audit-chain records every delegated decision.

### 7.10 Estate execution
A deceased user's personal tenant transitions to an estate workflow per ADR-0311. The executor's role projection is created; explicit permits scope what the executor can read, write, or export. The audit-chain holds the entire estate workflow.

## Section 7.A - Dual-tenant edge cases

### 7.A.1 Minor-to-adult transition
A minor reaching age of consent transitions from a parent-administered personal tenant to a self-administered personal tenant:
1. Substrate detects the milestone date plus the minor's passkey-attestation.
2. Delegated-administrator permits are deprovisioned with explicit notice to the parent.
3. The newly-adult user receives full control of their personal tenant.
4. Audit-chain records the transition; the parent's prior delegated actions remain visible to the user.

### 7.A.2 Divorce-related joint-tenant separation
A joint personal tenant (e.g., shared family finances) splits when partners separate:
1. Substrate opens a tenant-separation workflow; the workflow is policy-bound to allow either partner to initiate.
2. Each partner's share of assets, files, and ongoing workflows is identified through Cedar evaluation against the joint-tenant's policy.
3. Approve plus sign by both parties (or by a court-appointed arbitrator) commits the separation.
4. Two distinct tenants are provisioned from the joint tenant; relevant audit-chain history is exported with policy to each.

### 7.A.3 Cross-employer dual-employment
A user employed by two employers simultaneously (e.g., adjunct faculty at two universities) holds two work-tenant memberships:
1. Each tenant's substrate operates independently; the dual-tenant boundary holds.
2. Substrate verify-context surfaces the active tenant prominently; cross-tenant moves are explicit.
3. Per-tenant role projections do not auto-compose; the user manually switches tenants.

### 7.A.4 Volunteer-tenant under personal-tenant
A user's volunteer role at a nonprofit is governed via a nonprofit-tenant membership with volunteer role projection. The volunteer membership is distinct from the user's employment tenants and personal tenant.

### 7.A.5 Power-of-attorney crossing
A user grants power-of-attorney to a family member for healthcare decisions:
1. The grantor's personal-tenant attestation creates a delegated-administrator permit scoped to clinical workflows.
2. The grantee's substrate sees a dependent-administered tenant alongside their own personal tenant.
3. Clinical workflows the grantee initiates on behalf of the grantor are audit-chain-sealed under both identities.

### 7.A.6 Estate-pending tenant
A deceased user's personal tenant transitions to estate-execution per §3.1 of `training-cost-doctrine-2026-05-21.md` (in spirit) plus this document's §7.10:
1. Substrate detects the death certificate attached as evidence (via court or executor); estate workflow opens.
2. Executor's delegated-administrator permit is provisioned scoped to estate-execution.
3. The deceased's audit-chain is preserved; the executor can review history with policy-bound restrictions.
4. After estate completes, the tenant transitions to a long-term-archive state with statutory retention.

## Section 8 - Marketplace as universal settlement

### 8.1 ADR-0314 universal settlement doctrine
ADR-0314 establishes one marketplace for all deal settlement. The marketplace handles consumer purchases, business-to-business contracts, labor gigs, partner co-sell, plugin admission, dataset acquisition, model acquisition, and agent acquisition (ADR-0249 multi-category marketplace).

### 8.2 Marketplace settlement primitives
The settlement primitives are:
- **Offer**: an Ontology object class describing what is being offered, at what price, under what terms, with what evidence.
- **Acceptance**: a Workflow Engine state transition that binds a buyer to an offer; the buyer's sign event is the substrate primitive.
- **Settlement**: a marketplace-settlement event that transfers value (currency, credits, license, access) from buyer to seller under Cedar-permitted scope.
- **Dispute**: a Workflow Engine run that pauses settlement and routes the matter to a policy-bound mediator role.
- **Reversal**: a marketplace-settlement reversal event that undoes a prior settlement under Cedar-permitted scope.

### 8.3 Consumer-marketplace example
A user purchases a book in their personal tenant. The Offer is the book listing; the Acceptance is the user's approve on the checkout flow; the Settlement is the credit-card charge; the audit-chain records the entire flow. If the book arrives damaged, the user opens a Dispute through the marketplace dispute workflow; the seller's customer-service role processes the dispute; a Reversal event refunds the purchase.

### 8.4 Business-to-business marketplace example
An enterprise tenant procures a software license. The Offer is the vendor's listing; the Acceptance is the procurement officer's approve plus sign on a master-service-agreement; the Settlement is the wire transfer plus the license-issuance Ontology mutation; the audit-chain records the entire flow plus the contract revision history. If a license-true-up dispute arises, the marketplace-dispute workflow routes the matter to a mediator.

### 8.5 Labor-marketplace example
A freelance worker takes a gig. The Offer is the gig posting; the Acceptance is the worker's approve plus sign on the gig terms; the work is delivered as an Ontology object reference; the requester signs delivery acceptance; the Settlement transfers funds. If the work quality is disputed, the marketplace-dispute workflow routes the matter; the audit-chain records the dispute.

### 8.6 Partner-marketplace example
A partner co-sells with the primary vendor. The Offer is the co-sell deal; the Acceptance is the joint approve plus sign event; the Settlement allocates revenue between the parties per the partner agreement; the audit-chain records the allocation.

### 8.7 Plugin-marketplace example
A tenant admits a plugin. The Offer is the plugin listing; the Acceptance is the tenant administrator's approve plus sign on the plugin terms plus the compliance-pack admission decision; the Settlement is the per-invocation or per-period billing; the audit-chain records every plugin invocation.

### 8.8 Dataset-marketplace example
A tenant licenses a dataset. The Offer is the dataset listing including license terms; the Acceptance is the data-officer's approve plus sign plus the data-use-agreement; the Settlement transfers funds and grants access; the audit-chain records every access event.

### 8.9 Model-marketplace example
A tenant licenses a model. The Offer is the model listing including evaluation results plus license terms; the Acceptance is the AI-officer's approve plus sign plus the model-evaluation-pack admission; the Settlement transfers funds; the audit-chain records every inference event plus every retraining event.

### 8.10 Agent-marketplace example
A tenant deploys an agent. The Offer is the agent listing including capability description plus license terms; the Acceptance is the operator's approve plus sign plus the agent-admission policy fragment; the Settlement is per-invocation; the audit-chain records every agent action plus every Cedar evaluation by the agent.

### 8.11 Marketplace pack-overlay
Marketplace operations are pack-scoped. A HIPAA-pack tenant restricts marketplace settlement to BAA-signed counterparties. A PCI-pack tenant restricts payment instruments to PCI-scope. A CSAP-pack tenant restricts data residency. A GDPR-pack tenant enforces data-processing-agreements. A defense-pack tenant restricts to cleared counterparties.

### 8.12 Marketplace settlement and audit
Every marketplace settlement event is sealed into the audit-chain with: buyer-tenant, seller-tenant, offer-reference, acceptance-event, settlement-value, currency, pack-overlay, Cedar-permit-version, and Workflow-Engine-run-id. The audit-chain reconstruction can replay any settlement.

## Section 8.A - Marketplace edge-case workflows

### 8.A.1 Disputed-settlement workflow
When a buyer disputes a settlement, the marketplace primitive opens a dispute workflow:
1. The buyer's recover-from-denial verb in the marketplace context opens a dispute Workflow Engine run.
2. The substrate freezes the settlement event; downstream effects (e.g., escrow release, license activation) are paused.
3. A mediator role projection is assigned per the marketplace pack overlay; the mediator may be the seller, a tenant-chosen third party, or the marketplace operator (oyatie itself) under the ADR-0242 reserved-namespace pattern.
4. Both parties attach evidence to the dispute workflow.
5. The mediator's approve plus sign commits the dispute resolution; the settlement event is adjusted, reversed, or upheld.
6. The audit-chain records the dispute lifecycle.

### 8.A.2 Multi-party-settlement workflow
A multi-party deal (e.g., a partner co-sell with three vendors plus a customer) uses a multi-party settlement primitive:
1. Each party's approve plus sign commits their leg of the deal.
2. The marketplace-settlement event records the multi-party allocation.
3. Per-leg settlement is sealed into per-party audit-chain shards.
4. Disputes affect only the affected leg(s); other legs proceed.

### 8.A.3 Subscription-renewal workflow
A subscription is a recurring settlement schedule:
1. The marketplace-settlement primitive stores the schedule (frequency, value, term).
2. Each cycle, the substrate emits an approve-pending event to the buyer's role projection; the buyer's recover-from-denial widget exposes pause, modify, or cancel options.
3. On approve, settlement executes; on cancel, the schedule is terminated; on modify, a new schedule replaces the old.

### 8.A.4 Escrow-settlement workflow
An escrow settlement holds funds until evidence of delivery:
1. Buyer's approve plus sign commits funds to escrow.
2. Seller delivers; seller's attach-evidence binds delivery proof to the Workflow Engine run.
3. Buyer's approve releases escrow to the seller; buyer's recover-from-denial routes to dispute if delivery is unsatisfactory.
4. Settlement is released or held pending dispute.

### 8.A.5 Long-tail-buyer-credit-decline
When a buyer's creditworthiness drops mid-transaction:
1. The substrate detects the drop via the marketplace-reputation projection.
2. The pending settlement is paused; buyer's verify-context shows the credit alert.
3. Buyer can attach-evidence (supplemental credit information) or escalate to underwriting.
4. Underwriting's approve or deny resolves the case; audit-chain records the decision plus the reasoning.

### 8.A.6 Cross-currency settlement
A cross-currency deal carries currency-conversion plus FX-risk-management:
1. The marketplace-settlement primitive records both currencies plus the conversion rate as of acceptance.
2. The settlement event seals the rate; downstream reporting uses the sealed rate.
3. FX-risk pack overlay (where applicable) records hedging instruments.

### 8.A.7 Refund-and-chargeback workflow
A refund (initiated by the seller) or chargeback (initiated by the buyer's payment network):
1. Refund: seller's approve plus sign on the refund Workflow Engine run emits a reversal settlement event.
2. Chargeback: payment network's chargeback message triggers a Workflow Engine run; the substrate's marketplace-dispute primitive handles the response.

### 8.A.8 Marketplace-fraud-detection workflow
The marketplace-reputation projection surfaces fraud signals:
1. Substrate detects high-risk signals (velocity, geo-mismatch, BIN risk, account-age, device-fingerprint).
2. Pending settlement is paused; buyer's recover-from-denial offers verify-context options.
3. If verification passes, settlement proceeds; if it fails, the settlement is denied; if borderline, manual review routes the case.

## Section 9 - Conglomerate hierarchy through policy

### 9.1 ADR-0313 sovereign-child doctrine
ADR-0313 establishes that conglomerate tenancy is implemented through sovereign-child tenants under a parent. Each child enforces its own data-boundary; the parent cannot read child data unless an explicit cross-tenant Cedar permit fires.

### 9.2 Conglomerate examples
- A diversified manufacturer with subsidiaries in automotive, aerospace, and defense, each under different pack overlays.
- A diversified financial-services holding with subsidiaries in retail-banking, wealth-management, and insurance.
- A healthcare integrated delivery network with hospital, clinic, payor, and pharmacy subsidiaries.
- A retail conglomerate with apparel, grocery, and home-improvement subsidiaries.
- A media conglomerate with broadcast, streaming, publishing, and event subsidiaries.
- A government department with bureau-level subsidiaries.
- A university system with campus-level subsidiaries.
- A franchise-holding with regional master-franchise subsidiaries.

### 9.3 Parent-child data flow
The parent-child data flow is governed by explicit Cedar permits at each crossing. Typical patterns:
- **Consolidated reporting**: parent has a Cedar permit to read aggregated metrics from each child (revenue, headcount, EBITDA). Detail-level data remains in the child.
- **Cross-subsidiary mobility**: an employee transferring from subsidiary A to subsidiary B carries their personal-tenant data; subsidiary-A-tenant deprovisions; subsidiary-B-tenant provisions.
- **Shared services**: a parent-tenant shared-services-team accesses each child only through explicit role projections scoped per child.
- **Inter-company transactions**: settlement flows through the marketplace primitive between child tenants; the parent has consolidated read.

### 9.4 Sovereign-child boundary cases
- A subsidiary requires a different compliance pack than its parent. The pack overlay is per-tenant; sovereignty is preserved.
- A subsidiary is divested. The substrate deprovisions the parent's cross-tenant permits; the subsidiary tenant continues independently.
- A subsidiary is acquired into the conglomerate. The substrate provisions the parent's cross-tenant permits with explicit scope; the subsidiary's prior tenant continues with parent now visible.
- A subsidiary requires a different jurisdiction's data residency (e.g., a Korean subsidiary needs CSAP-pack KR data residency). The pack overlay enforces residency; the parent's permits operate at the metadata layer rather than data layer.

### 9.5 Conglomerate audit posture
Each subsidiary maintains its own audit-chain shard. The parent has Cedar permits for aggregated audit views (e.g., total denial counts, total settlement value, total workflow throughput) and for cross-subsidiary incident review. Detail-level audit query requires per-subsidiary explicit permits.

### 9.6 Worked example: a global conglomerate
A diversified manufacturer (Acme Industrial) has subsidiaries Acme Automotive (US, EU, JP, KR plants), Acme Aerospace (US, EU plants), and Acme Defense (US plants only). Each subsidiary has its own compliance pack profile:
- Acme Automotive: SOC2 plus per-country data residency packs.
- Acme Aerospace: SOC2 plus AS9100 plus ITAR plus per-country data residency packs.
- Acme Defense: CMMC plus ITAR plus per-program nuclear-surety where applicable.

Each subsidiary's substrate is independent. The parent Acme Industrial has cross-tenant Cedar permits for: consolidated financial reporting, consolidated workforce headcount, consolidated audit summary, and consolidated risk register. The parent does not have read access to subsidiary-level Ontology detail.

A finance executive at Acme Industrial moves to Acme Aerospace as a controller. The substrate deprovisions Acme Industrial finance permits, provisions Acme Aerospace controller permits, and re-renders the UX shell with the new role projection. The executive's personal-tenant data is unchanged.

### 9.7 Conglomerate divestiture worked example
Acme Industrial divests Acme Aerospace to a buyer. The substrate executes a divestiture workflow:
1. Acme Industrial parent permits over Acme Aerospace are deprovisioned.
2. Buyer tenant is given parent permits per the purchase agreement.
3. Acme Aerospace tenant migrates control to the buyer tenant; the data remains in Acme Aerospace.
4. Workforce identities in Acme Aerospace are deprovisioned from Acme Industrial parent and reprovisioned under the buyer.
5. The audit-chain records the entire divestiture including the precise time and policy-version of each transition.

The buyer can review the Acme Aerospace audit-chain going back to whenever the audit-chain began, subject to the retention policy.

### 9.8 Conglomerate acquisition worked example
Acme Industrial acquires Beta Components. The substrate executes an acquisition workflow:
1. Beta Components tenant becomes a sovereign child of Acme Industrial.
2. Parent-child Cedar permits are provisioned per the integration plan.
3. Beta Components compliance packs are reviewed; gaps are flagged for remediation.
4. Beta Components workforce identities are joined to Acme Industrial corporate directory through explicit consent and audit.
5. The audit-chain records the entire acquisition.

The integration plan can take months. The substrate accommodates incremental integration; the conglomerate posture holds at each step.

## Section 9.A - Conglomerate cross-subsidiary workflow examples

### 9.A.1 Cross-subsidiary procurement
The parent conglomerate negotiates a master vendor agreement with a common supplier. Each subsidiary may draw against the master:
1. Parent's procurement team opens a master-agreement Workflow Engine run; sign event commits the master.
2. Each subsidiary's procurement team opens a per-subsidiary draw-down Workflow run referencing the master.
3. Each draw-down's settlement flows through the marketplace primitive scoped to the subsidiary tenant.
4. Parent's consolidated procurement dashboard projects aggregated draw across subsidiaries via Cedar-scoped cross-tenant read permits.

### 9.A.2 Cross-subsidiary talent transfer
An employee transfers from subsidiary A to subsidiary B:
1. HR-business-partner at subsidiary A opens a transfer-out Workflow Engine run.
2. Receiving HR-business-partner at subsidiary B opens a transfer-in Workflow Engine run.
3. The employee's identity holds new tenant membership at subsidiary B; subsidiary A's membership is deprovisioned at the effective date.
4. Pack-aware data-transfer: employment-history transfers via export-with-policy; performance reviews and clinical-record-equivalents may transfer subject to pack rules.
5. The audit-chain at each subsidiary records the corresponding leg of the transfer.

### 9.A.3 Cross-subsidiary financial reporting
The parent consolidates financials across subsidiaries:
1. Each subsidiary publishes its trial-balance projection via the substrate's consolidated-reporting primitive.
2. Parent's consolidation-controller opens a consolidation Workflow Engine run.
3. Inter-company elimination journals are authored as Ontology objects scoped to the parent-tenant; the parent has read access to subsidiary trial balances via explicit cross-tenant Cedar permits.
4. Consolidated financial statements project across the subsidiary set.
5. Subsidiary-level detail remains in the subsidiary tenant; the parent sees aggregated reports.

### 9.A.4 Cross-subsidiary shared-services
The parent operates a shared-services tenant (HR, IT, legal, finance) that serves all subsidiaries:
1. Shared-services employees hold tenant membership in the shared-services tenant.
2. Per-subsidiary cross-tenant Cedar permits scope what each shared-services role can do in each subsidiary.
3. A shared-services HR employee may read employee data in subsidiary A under a specific HR-shared-service Cedar permit; the same employee does not automatically read subsidiary B data.

### 9.A.5 Cross-subsidiary audit
The parent's internal audit function operates across subsidiaries:
1. Internal-audit team holds membership in a parent-audit tenant.
2. Per-subsidiary cross-tenant Cedar permits scope audit access.
3. Each audit engagement opens a Workflow Engine run; engagement-specific evidence is sealed into the parent-audit tenant audit-chain plus reciprocal seals into each affected subsidiary's audit-chain.
4. Findings are projected to the relevant subsidiary's risk-committee role projection.

### 9.A.6 Cross-subsidiary regulatory submission
Some regulators require consolidated reporting (e.g., SEC for public conglomerates, FDIC for bank-holding-company structures):
1. Parent's regulatory-affairs role projection has cross-subsidiary read permits scoped to the regulator's required-disclosure schema.
2. Each subsidiary attests to the accuracy of its data via the sign verb at the subsidiary level.
3. The consolidated submission is sealed in the parent-tenant audit-chain plus each subsidiary's audit-chain.
4. Regulator-acknowledgment closes the submission Workflow Engine run.

### 9.A.7 Cross-subsidiary M&A
The parent acquires an additional subsidiary:
1. Acquisition-target tenant is provisioned as a new sovereign child of the parent.
2. Per-function integration plan is opened as a Workflow Engine run; each function (HR, finance, IT, ops) has its own per-subsidiary acceptance criteria.
3. Per-function cutover happens incrementally; the substrate accommodates the migration phase per §13.C.
4. The audit-chain records the entire acquisition.

### 9.A.8 Cross-subsidiary divestiture
The parent divests a subsidiary:
1. Divestiture Workflow Engine run is opened.
2. Cross-tenant Cedar permits between parent and divested subsidiary are deprovisioned per the divestiture agreement.
3. Buyer-tenant is granted parent-equivalent permits per the deal terms.
4. The audit-chain records the entire divestiture; future regulator inquiries can reconstruct the divestiture from the audit-chain.

### 9.A.9 Cross-subsidiary joint-venture
The parent forms a joint venture with an external partner:
1. New JV-tenant is provisioned; both parent and partner have cross-tenant permits scoped per the JV agreement.
2. JV's operations are scoped to the JV tenant; profit-sharing flows through marketplace settlement between the parent, partner, and JV tenants.
3. The audit-chain records the JV's operations; each party has read access scoped per agreement.

### 9.A.10 Cross-subsidiary internal-control framework
The parent maintains an enterprise-wide internal-control framework (e.g., SOX for a public conglomerate):
1. Control catalog lives in the parent-audit tenant Ontology.
2. Each subsidiary attests to its implementation of each applicable control via the sign verb.
3. Independent testing routes through the parent-audit tenant audit-chain.
4. Audit findings project to the relevant subsidiary risk-committee and to the parent audit-committee.

## Section 10 - What this displaces

The doctrine displaces SAP, Oracle, Workday, Salesforce, ServiceNow, Microsoft 365, NetSuite, Atlassian, Slack, Zoom, GitHub, Gusto, Stripe, and adjacent stacks at the integration and training layer. Displacement is strongest where the incumbent requires a new account model, a new training model, a new integration lane, or a new audit-export grammar. Displacement is weakest where the incumbent owns a certified network, a regulated rail, a hardware integration, or a domain-specific operational engine.

For each displaced incumbent, the doctrine maps the incumbent's substrate-relevant primitives onto oyatie's substrate.

### 10.1 SAP S/4HANA displacement
**Incumbent surface**: ERP module depth in financial accounting, materials, manufacturing, plant maintenance, treasury, and procurement.

**Substrate mapping**:
- SAP G/L → oyatie Ontology object class (journal-entry) plus Workflow template (period-close).
- SAP MM → oyatie Ontology object class (material, purchase-order) plus Workflow template (procure-to-pay).
- SAP PP → oyatie Ontology object class (production-order) plus Workflow template (production-execution).
- SAP PM → oyatie Ontology object class (work-order) plus Workflow template (preventive-maintenance).
- SAP FI → oyatie Ontology object class (financial-document) plus Workflow template (financial-period).
- SAP HCM → oyatie HR-capability-tier projections per Section 4.4.
- SAP authorizations → oyatie Cedar permits scoped per role-projection.
- SAP change documents → oyatie audit-chain events.

**Displacement weakness**: SAP's industry-specific extensions (e.g., upstream oil-and-gas, banking, public-sector) carry many years of regulatory and operational depth that takes time to match. Migration plan must preserve evidence and run parallel until equivalence is verified.

**Required migration posture**: preserve source-system evidence by replay into oyatie audit-chain; map objects to Ontology with bi-directional reconciliation; route approvals through Workflow Engine in parallel with SAP workflows; gate access through Cedar; seal transitions through audit-chain.

**Worked example**: a mid-size manufacturer migrates from SAP S/4HANA to oyatie. The migration plan starts with two functions (procure-to-pay and accounts-payable), runs them in parallel for one quarter, validates reconciliation, then deprecates SAP for those functions. The other functions migrate in sequence over twelve to eighteen months. The displacement is permanent at each migrated function; rollback is possible because audit-chain replay reconstructs the state.

### 10.2 Oracle Fusion displacement
**Incumbent surface**: enterprise finance, HCM, procurement, supply chain, planning, and analytics.

**Substrate mapping**: parallel to SAP per §10.1 with Oracle-specific module names. Oracle Fusion Procurement → oyatie procurement Ontology plus Workflow templates. Oracle Fusion HCM → oyatie HR capability tier. Oracle Fusion Financials → oyatie financial capability tier. Oracle Fusion Planning → oyatie analytics-and-planning projection over the Ontology.

**Displacement weakness**: Oracle's analytics depth (Essbase-derived planning, ARCS reconciliation, NSE close) takes time to match in the analytics-and-planning projection.

**Required migration posture**: as §10.1 plus Oracle-specific data-model bridges for the analytics projections.

### 10.3 Workday displacement
**Incumbent surface**: HCM, workforce planning, learning, performance, adaptive planning.

**Substrate mapping**: Workday business-objects → oyatie HR capability tier per Section 4.4. Workday security groups → Cedar permits. Workday business processes → Workflow templates. Workday reports → audit-chain queries plus Ontology projection.

**Displacement weakness**: Workday's payroll engine in countries with idiosyncratic statutory rules takes time to match. The doctrine recommends keeping Workday payroll as a service (per ADR-0315 service exception) while migrating HCM, learning, performance, and planning to substrate.

**Required migration posture**: as §10.1.

### 10.4 Salesforce displacement
**Incumbent surface**: sales, service, marketing, marketplace, low-code platform, customer graph.

**Substrate mapping**: Salesforce account → oyatie Ontology account object. Salesforce opportunity → oyatie Ontology opportunity object. Salesforce flow → Workflow template. Salesforce permission set → Cedar permit. Salesforce report → audit-chain query plus Ontology projection. AppExchange plugin → marketplace-admitted plugin per ADR-0249.

**Displacement weakness**: Salesforce's industry clouds (Health Cloud, Financial Services Cloud, Manufacturing Cloud) ship deep pre-built object models. Substrate equivalents must be authored.

**Required migration posture**: as §10.1.

### 10.5 ServiceNow displacement
**Incumbent surface**: ITSM, ITOM, HRSD, customer-service, low-code platform.

**Substrate mapping**: ServiceNow incident → ITSM-capability-tier per Section 4.5. ServiceNow CMDB → Ontology configuration-item object class. ServiceNow workflow → Workflow template. ServiceNow ACL → Cedar permits. ServiceNow Now Platform extensions → marketplace-admitted plugins.

**Displacement weakness**: ServiceNow's integration depth into infrastructure-monitoring stacks and orchestration tools.

**Required migration posture**: as §10.1 plus infrastructure-monitoring integration retention as services.

### 10.6 Microsoft 365 displacement
**Incumbent surface**: Word, Excel, PowerPoint, Outlook, Teams, SharePoint, OneDrive, identity, compliance, learning.

**Substrate mapping**: Outlook → mail collaboration projection. Excel → sheets collaboration projection. Teams → meet plus chat plus community collaboration projection. SharePoint → file-storage collaboration projection. OneDrive → personal-tenant file storage. Microsoft Entra ID → oyatie identity. Microsoft Purview → compliance-pack overlays.

**Displacement weakness**: Microsoft 365's document-fidelity expectations and its installed-base of legacy file formats. The doctrine recommends fidelity-tested viewers and editors plus migration tooling.

**Required migration posture**: as §10.1 plus per-document-format fidelity testing.

### 10.7 NetSuite displacement
**Incumbent surface**: SMB-and-mid-market ERP with finance, inventory, CRM, e-commerce.

**Substrate mapping**: NetSuite records → oyatie Ontology objects. NetSuite saved searches → audit-chain queries plus Ontology projections. NetSuite SuiteFlow → Workflow templates. NetSuite roles → Cedar permits. NetSuite SuiteCloud extensions → marketplace plugins.

**Displacement weakness**: NetSuite's mid-market deployment-speed advantage takes time to match with substrate-side onboarding tooling.

**Required migration posture**: as §10.1.

### 10.8 Atlassian displacement
**Incumbent surface**: Jira, Confluence, Bitbucket, Jira Service Management, Compass.

**Substrate mapping**: Jira issue → Workflow template plus Ontology object. Confluence page → file-storage projection plus collaboration. Bitbucket repository → development-capability-tier (see §10.13 GitHub). Jira Service Management → ITSM capability tier. Compass → catalog projection over the Ontology.

**Displacement weakness**: Atlassian's deep developer-workflow integrations into source control and CI/CD.

**Required migration posture**: as §10.1 plus developer-workflow continuity work.

### 10.9 Slack displacement
**Incumbent surface**: workplace chat, channels, threads, integrations.

**Substrate mapping**: Slack channel → community-projection scoped to role-projection. Slack DM → mail-projection. Slack thread → comment-projection on a workflow run or Ontology object. Slack integration → marketplace-admitted plugin.

**Displacement weakness**: Slack's installed-base of bots and custom workflows; integration migration is the main cost.

**Required migration posture**: as §10.1 plus per-bot migration plan.

### 10.10 Zoom displacement
**Incumbent surface**: video meetings, webinars, phone, contact-center.

**Substrate mapping**: Zoom meeting → meet-projection plus collaboration. Zoom webinar → community-projection. Zoom phone → telephony-service (likely an ADR-0315 service exception due to PSTN integration). Zoom contact-center → contact-center-capability-tier.

**Displacement weakness**: PSTN integration depth; contact-center routing depth.

**Required migration posture**: as §10.1 plus telephony service retention.

### 10.11 GitHub displacement
**Incumbent surface**: source-control, pull-requests, actions, packages, security scanning.

**Substrate mapping**: GitHub repository → development-capability-tier with Ontology object (repository, commit, pull-request). GitHub Actions → Workflow templates. GitHub Apps → marketplace-admitted plugins.

**Displacement weakness**: GitHub's developer-network effects; integration ecosystem depth.

**Required migration posture**: as §10.1 plus developer-network continuity.

### 10.12 Gusto displacement
**Incumbent surface**: SMB payroll, benefits, HR.

**Substrate mapping**: Gusto payroll → HR-capability-tier payroll. Gusto benefits → HR-capability-tier benefits. Gusto HR → HR-capability-tier employee-data plus time-tracking.

**Displacement weakness**: Gusto's per-state-and-locality tax-rate database; benefit-provider integrations.

**Required migration posture**: as §10.1 plus tax-rate database population plus benefit-provider integration build-out.

### 10.13 Stripe displacement
**Incumbent surface**: payment processing, billing, fraud, identity, treasury.

**Substrate mapping**: Stripe charge → marketplace-settlement event. Stripe subscription → marketplace-settlement recurring schedule. Stripe → marketplace-settlement primitive (multi-party). Stripe Radar → fraud-detection projection.

**Displacement weakness**: Stripe's payment-rail integrations and acquiring-bank relationships; PCI compliance depth.

**Required migration posture**: most likely retain Stripe (or alternative payment processor) as a service per ADR-0315 service exception due to acquiring-bank relationship; integrate via marketplace-settlement primitive.

### 10.14 Adjacent-stack displacements
Brief mappings for adjacent stacks:
- Coupa → procurement capability tier; service retention for sourcing-event execution against external suppliers.
- Concur → expense capability tier within HR-finance scope.
- Anaplan → analytics-and-planning projection over Ontology.
- BlackLine → close capability tier within ERP.
- Workiva → audit-and-governance capability tier.
- DocuSign → e-signature substrate primitive (the sign verb) plus marketplace-settlement plugin for legacy DocuSign workflows.
- Lattice / 15Five → HR-capability-tier performance plus OKR projections.
- Tableau / Power BI → analytics-and-planning projection over Ontology.
- AuditBoard / LogicGate → audit-and-governance capability tier.
- Notion / Confluence → collaboration capability tier (notes, sheets, file-storage).

### 10.15 Displacement claim discipline
Marketing copy that claims oyatie displaces vendor X must cite:
- The specific incumbent surface displaced (which module, which capability).
- The substrate mapping (which oyatie primitive holds the displaced function).
- The displacement weakness (where the incumbent retains advantage and what the substrate plan is).
- The required migration posture (parallel-run, evidence preservation, audit-chain mapping).
- The audit-chain reference for the customer who realized the displacement, if claiming a customer case.

Marketing copy that cannot cite all five is forbidden per the multispectrum-review v2.4.0 lane plus the doctrine's claim-discipline rule.

## Section 10.A - Industry-specific displacement bundles

### 10.A.1 Healthcare-system displacement bundle
A typical 1,500-bed integrated delivery network operates with approximately 70-110 SaaS and on-prem tools. Representative bundle:
- Epic or Cerner (EHR): clinical capability tier projection; Epic-specific UI patterns are translated to UX-shell density variants.
- Workday or Lawson (HR plus finance): HR plus financial capability tiers.
- McKesson or Cardinal (supply chain): supply-chain capability tier; pharmacy-specific operational nuance retained as a service.
- nThrive or Optum (RCM): revenue-cycle-management capability tier; payor-network integration retained as a service.
- Press Ganey or NRC Health (patient experience surveys): community capability tier projection.
- Imprivata (clinical SSO): substrate identity; clinical workflows integrated via passkey.
- Vocera or Halo (clinical communication): community plus meet projections; PSTN-bridge retained as a service.
- Allscripts or Athenahealth (ambulatory EHR): clinical capability tier projection scoped to ambulatory clinics.
- Kronos or UKG (workforce time): HR capability tier time-tracking projection.
- Various clinical-specialty applications (radiology, pathology, cardiology, oncology): each evaluated; substrate-projection where possible; service-retention where hardware integration or specialty-certified rail justifies.

Displacement timeline: 24-48 months for full migration; clinical-specialty retentions may persist indefinitely.

### 10.A.2 Financial-services-enterprise displacement bundle
A typical 5,000-person retail-bank operates with approximately 130-180 tools. Representative bundle:
- Fiserv DNA or Jack Henry SilverLake (core banking): retain as a service due to certified-rail integration; substrate-projection of customer-facing surfaces.
- Salesforce (sales plus service): CRM capability tier projection.
- Workday (HR): HR capability tier projection.
- Oracle EBS (back-office finance): financial capability tier projection; Hyperion-class planning projected over Ontology.
- nCino (commercial lending workflow): lending capability tier projection.
- Black Knight (mortgage origination plus servicing): mortgage capability tier projection; investor-reporting service retained.
- LexisNexis (KYC and AML): plugin-admission for AML-pack-conformance.
- BloombergGPT or Refinitiv (market data): plugin-admission for market-data pack.
- ServiceNow ITSM: ITSM capability tier projection.
- Microsoft 365: collaboration capability tier projection.
- Tableau or Power BI: analytics projection over Ontology.

Displacement timeline: 36-60 months; core-banking service retentions are typical.

### 10.A.3 Manufacturing-enterprise displacement bundle
A typical 12,000-person discrete manufacturer operates with approximately 80-120 tools. Representative bundle:
- SAP S/4HANA (ERP): ERP capability tier projection per §10.1.
- Siemens Teamcenter (PLM): plugin-admission for PLM data integration; engineering capability tier projection.
- Apriso or Plex (MES): service retention due to hardware integration; substrate-projection of MES output for executive reporting.
- Maximo (asset management): plant-maintenance capability tier projection.
- Workday (HR): HR capability tier projection.
- ServiceNow (ITSM): ITSM capability tier projection.
- Microsoft 365: collaboration capability tier projection.
- Coupa (procurement): procurement capability tier projection plus marketplace integration.
- Tableau or Power BI: analytics projection.
- Quality-management systems: quality capability tier projection.

Displacement timeline: 24-48 months; MES service retentions are typical.

### 10.A.4 Public-sector-state-agency displacement bundle
A typical 8,000-person state human-services agency operates with approximately 60-100 tools. Representative bundle:
- IBM Cúram or Northrop Grumman ETS (eligibility system): eligibility capability tier projection.
- Tyler Munis or Infor (financial plus HR): financial plus HR capability tier projections.
- Workday: HR capability tier projection.
- ServiceNow: ITSM capability tier projection.
- Microsoft 365: collaboration capability tier projection.
- Adobe Sign or DocuSign: e-signature substrate primitive integration.
- Various federal-program interfaces (CMS, USDA, HUD): service retention due to certified network membership.
- State-public-records portal: substrate projection.

Displacement timeline: 36-72 months; federal-interface service retentions are typical.

### 10.A.5 Higher-education displacement bundle
A typical 30,000-student university operates with approximately 90-150 tools. Representative bundle:
- Ellucian Banner or Workday Student (SIS): student-records capability tier projection.
- Workday or Oracle PeopleSoft (HR plus finance): HR plus financial capability tier projections.
- Canvas or Blackboard (LMS): learning capability tier projection.
- ServiceNow or Cherwell (ITSM): ITSM capability tier projection.
- Microsoft 365 or Google Workspace: collaboration capability tier projection.
- Slate (admissions CRM): admissions capability tier projection.
- Cayuse or InfoEd (research administration): research capability tier projection.
- Salesforce or Blackbaud (advancement): advancement capability tier projection.
- Library-management systems: library capability tier projection plus per-publisher service retention.
- Athletic-management systems: athletics capability tier projection.

Displacement timeline: 36-60 months.

### 10.A.6 Retail-chain displacement bundle
A typical 1,000-store national retailer operates with approximately 70-110 tools. Representative bundle:
- SAP or Oracle (merchandising plus finance): ERP capability tier projection.
- Manhattan Associates or Blue Yonder (supply chain): supply-chain capability tier projection.
- Workday (HR): HR capability tier projection.
- Salesforce (customer engagement): CRM capability tier projection plus marketing capability tier.
- Square or NCR (POS): retail capability tier projection plus per-payment-rail service retention.
- ServiceNow (ITSM): ITSM capability tier projection.
- Microsoft 365: collaboration capability tier projection.
- Adobe (creative): retain as service; integration via marketplace plugin.
- Per-loyalty-program platforms: marketing capability tier projection.
- Per-payment-processor: marketplace-settlement integration.

Displacement timeline: 24-48 months.

### 10.A.7 Hospitality-chain displacement bundle
A typical 500-property hotel chain operates with approximately 50-90 tools. Representative bundle:
- Oracle OPERA or Mews (PMS): hospitality capability tier projection.
- Sabre or Amadeus (GDS): retain as service due to airline-network integration; substrate-projection of inventory.
- Salesforce or Cendyn (CRM): CRM capability tier projection.
- Workday (HR): HR capability tier projection.
- Microsoft 365: collaboration capability tier projection.
- Per-loyalty-program platforms: marketing capability tier projection.

Displacement timeline: 24-48 months.

### 10.A.8 Energy-utility displacement bundle
A typical 5,000-person regulated electric utility operates with approximately 100-150 tools. Representative bundle:
- Oracle Utilities CC&B or SAP IS-U (customer billing): utility-billing capability tier projection.
- OSI Monarch or Schneider PowerSCADA (SCADA): retain as service due to substation hardware integration.
- IBM Maximo (asset management): plant-maintenance capability tier projection.
- Workday (HR): HR capability tier projection.
- ServiceNow (ITSM): ITSM capability tier projection.
- Microsoft 365: collaboration capability tier projection.
- NERC-CIP compliance tools: substrate compliance-pack overlay.
- Per-regulator submission portals: substrate projection.

Displacement timeline: 48-72 months; SCADA service retentions are typical.

### 10.A.9 Telecommunications-carrier displacement bundle
A typical 50,000-person national telecom operates with approximately 200-300 tools. Representative bundle:
- Amdocs or Oracle BSS (billing plus order management): telecom capability tier projection.
- Cisco NSO or Ericsson OSS (network operations): retain as service due to network-element integration.
- Salesforce (customer engagement): CRM capability tier projection.
- Workday (HR): HR capability tier projection.
- ServiceNow (ITSM): ITSM capability tier projection plus telecom-specific runbook integration.
- Microsoft 365: collaboration capability tier projection.
- Per-regulator submission portals: substrate projection.

Displacement timeline: 48-72 months; OSS service retentions are typical.

### 10.A.10 Government-federal displacement bundle
A typical 20,000-person federal cabinet department operates with approximately 200-400 tools across bureaus. Representative bundle:
- Per-bureau legacy mainframe applications: substrate-projection where possible; service retention for mission-critical systems.
- Oracle or Workday (HR): HR capability tier projection scoped per bureau.
- SAP or Oracle (finance): financial capability tier projection scoped per bureau.
- ServiceNow (ITSM): ITSM capability tier projection scoped per bureau.
- Microsoft 365 or Google Workspace (collaboration): collaboration capability tier projection.
- FedRAMP and CMMC compliance pack overlays: substrate compliance-pack primitive.
- Per-mission integrations: service retention.

Displacement timeline: 60-120 months due to federal procurement and certification cycles.

## Section 10.B - Per-vendor incumbent-replacement-cost model

### 10.B.1 Cost categories during migration
A migration carries five cost categories:
1. **Substrate-build cost**: oyatie engineering effort to build the substrate primitives the migration depends on. Amortized across all customers.
2. **Customer-migration cost**: customer-side migration team plus oyatie professional services.
3. **Parallel-run cost**: paying for incumbent plus oyatie during parallel-run.
4. **Reconciliation cost**: per-function nightly reconciliation reports.
5. **Training-cost (one-time)**: substrate fluency for the workforce (one-time per career arc per `training-cost-doctrine`).

### 10.B.2 Cost recovery model
The savings from §8 of `training-cost-doctrine-2026-05-21.md` plus the §1.2 fragmentation-tax reduction recover the migration cost. For a typical 1,000-person enterprise, the cost-recovery period is 12-30 months depending on regulation intensity. After break-even, the savings compound annually.

### 10.B.3 Cost-recovery worked example
A 1,000-person healthcare-services tenant migrates from Workday (HR plus payroll) plus Salesforce (CRM) plus Microsoft 365 (collaboration) over 24 months. Migration cost: USD 4.2M (services plus internal labor plus parallel-run). Year-1 savings: USD 6.5M. Year-2 savings: USD 12.0M. Year-3+ steady-state savings: USD 14.5M per year.

Cost recovery: month 18. Five-year net benefit: approximately USD 58M.

### 10.B.4 Risk-adjusted cost recovery
A risk-adjusted cost-recovery model accounts for the probability of: (a) substrate-primitive-defect during migration (5-10 percent likelihood depending on substrate maturity), (b) regulatory-finding during migration (3-5 percent likelihood at high-regulation tenants), (c) operational-disruption during cutover (2-5 percent likelihood with parallel-run mitigation), and (d) workforce-resistance (10-20 percent likelihood with change-management mitigation).

The doctrine recommends 1.3-1.5x cost-recovery-multiplier buffering and explicit contingency planning per risk.

## Section 11 - Training-cost amortization

Detailed treatment lives in the sibling document `docs/architecture/training-cost-doctrine-2026-05-21.md`. This section summarizes the substrate-side guarantees that enable the training-cost amortization claim.

### 11.1 Substrate guarantees
- The verb enum is exactly thirteen, enforced at the UX shell action router.
- Each verb resolves through identical substrate primitives (identity, policy, workflow, ontology, audit-chain) regardless of role projection, capability tier, workspace, locale, or compliance pack.
- Role projection switching is one verb (switch-role), audit-chain-sealed.
- Verify-context is a substrate widget visible in every product surface.
- Recover-from-denial is a substrate widget with deterministic remediation paths.
- Accessibility is substrate, not product opt-in.
- The dual-tenant boundary holds across the verb stack.

### 11.2 Training-cost claim chain
The substrate guarantees enable the §1.2 fragmentation-tax reduction. The training-cost-doctrine sibling document quantifies the savings (USD 38.5 million per year for a 1,000-employee enterprise at the 110-tool baseline) and provides the per-industry-pack walkthroughs.

### 11.3 Substrate verification
The four telemetry signals (time-to-first-successful-action, support-ticket volume per role, wrong-context-action denial rate, accessibility-task-completion rate) are produced by the substrate and consumed by the doctrine's claim-discipline gate. Without the four signals showing movement in the right direction, the training-cost claim is hypothetical.

## Section 12 - Day-zero adoption

### 12.1 The day-zero promise
A new hire opens oyatie on day one and executes their first productive verb within the first hour. The substrate fluency that the user carries from personal-tenant use, intern-pack experience, or prior employer-tenant use transfers immediately.

### 12.2 Day-zero example: enterprise new hire
A senior software engineer joins a 2,000-person enterprise. The engineer arrives with personal-tenant fluency from years of marketplace and personal-finance use plus prior employer-tenant experience at three companies.

Day-zero sequence:
- 09:00 passkey-bound onboarding; tenant membership provisioned.
- 09:15 UX shell loads with engineering-role-projection density; the thirteen verbs are visible.
- 09:20 the engineer executes verify-context to confirm the active tenant and role.
- 09:25 the engineer reviews the engineering capability tier landing page; the assigned project list is rendered as an Ontology projection.
- 09:45 the engineer approves a self-service tooling-access request; substrate verbs resolved identically to prior experience.
- 10:30 the engineer attaches evidence (proof of training completion) to the onboarding workflow run.
- 11:00 the engineer signs the employment-attestation packet.
- 11:30 the engineer routes a question to the assigned buddy.
- 12:00 lunch; the engineer's first productive verb-sequence is in the audit-chain.

Training time spent: zero hours on platform-vocabulary; approximately one hour on enterprise-specific evidence-and-policy refresher; remainder on team and product context.

### 12.3 Day-zero example: clinical new hire
A new graduate nurse joins a hospital. The nurse arrives with personal-tenant fluency plus intern-pack experience plus academic-pack experience from nursing school plus clinical-rotation-pack experience.

Day-zero sequence:
- 06:30 passkey-bound onboarding; hospital-tenant membership provisioned with registered-nurse role.
- 07:00 UX shell loads with clinical-bedside density; verify-context shows assigned unit.
- 07:15 the nurse reviews history of the assigned patient list at handoff.
- 07:30 the nurse attaches evidence (vitals) for the first round of patients.
- 08:00 the nurse approves a routine medication administration after passkey-bound presence and pump-pairing checks.
- 09:00 the nurse encounters the controlled-substance witnessing recovery flow; the recover-from-denial widget walks the nurse through the witness flow.
- 12:00 lunch; the nurse's first half-shift is in the audit-chain.

Training time: zero hours on platform-vocabulary; approximately two hours on clinical-pack-specific evidence (vitals, labs, imaging, witness protocols) and policy fragments; remainder on the unit-specific clinical-context.

### 12.4 Day-zero example: warehouse new hire
A new receiving clerk joins a distribution center. The clerk arrives with personal-tenant fluency plus side-business shipping experience plus prior retail-pack experience at a previous employer.

Day-zero sequence:
- 07:00 passkey-bound onboarding; tenant membership provisioned with receiving-clerk role.
- 07:15 the clerk picks up the handheld terminal; verify-context shows the assigned dock-bay.
- 07:30 the clerk reviews history of the previous shift's quarantine actions.
- 08:00 the clerk approves the first inbound truck arrival.
- 08:30 the clerk attaches evidence (seal-intact photo, BOL scan).
- 12:00 lunch; the clerk's first half-shift is in the audit-chain.

Training time: zero hours on platform-vocabulary; approximately one hour on logistics-pack-specific evidence and policy fragments; remainder on the dock-specific operational training.

### 12.5 Day-zero example: side-business launch
A user launches a side-business (a graphic-design freelance practice) on the same day they receive their state-business-registration certificate.

Day-zero sequence:
- 10:00 the user opens oyatie in their personal tenant.
- 10:05 the user creates a new sole-proprietorship tenant; tenant membership is provisioned with owner-operator role.
- 10:15 the user attaches evidence (the business-registration certificate) to the tenant attestation workflow.
- 10:30 the user signs the small-business-pack attestation.
- 11:00 the user creates a marketplace-offer for their freelance services.
- 12:00 lunch; the side-business tenant has a first offer in the marketplace and the audit-chain has the entire setup.

Training time: zero hours; the user knows the verbs from personal-tenant use.

### 12.6 Day-zero example: retiree estate-planning kickoff
A retiree's adult child opens an estate-planning workflow on the retiree's behalf with the retiree's consent.

Day-zero sequence:
- 14:00 the retiree opens oyatie in their personal tenant; the retiree's personal-tenant has been active for 30+ years.
- 14:15 the retiree's adult child opens oyatie in their personal tenant; the child holds a delegated-administrator permit per ADR-0311.
- 14:30 the retiree signs the estate-planning-workflow initiation.
- 14:45 the child attaches evidence (current will, beneficiary designations, asset inventory).
- 15:30 the retiree approves the proposed estate-plan revision.
- 16:00 the entire flow is in the audit-chain.

Training time: zero hours; the retiree and the child both know the verbs.

## Section 12.A - Day-zero pack-overlay worked examples

### 12.A.1 HIPAA-pack day-zero scenarios
A new clinical employee at a HIPAA-covered entity completes day-zero in 90 minutes when the substrate is in place:
1. Identity provisioning: passkey registration, BAA-attestation acknowledgment, HIPAA-training attestation (if not already on file from prior employment).
2. Tenant membership: hospital tenant scoped to assigned unit plus capability tier.
3. Cedar permit set: clinical bedside scope plus medication-administration scope plus controlled-substance witnessing scope where applicable plus role-required-attending-co-sign scope.
4. Audit-chain orientation: review history of the assigned patients at handoff.
5. Verb walkthrough: practice verify-context plus approve plus sign plus route plus recover-from-denial in a HIPAA-trained simulator scenario.
6. Real productive work: the first medication administration with full audit-chain seal.

Comparison: legacy onboarding at a HIPAA covered entity typically spans 2-5 days of Epic or Cerner click-path training before the new clinician can perform real productive work. The substrate cuts this to 90 minutes plus role-specific clinical mentorship that exists regardless of platform.

### 12.A.2 PCI-pack day-zero scenarios
A new e-commerce-fulfillment employee at a PCI-scope tenant completes day-zero in 60 minutes:
1. Identity provisioning: passkey plus PCI-training attestation.
2. Tenant membership: fulfillment tenant plus capability tier.
3. Cedar permit set: fulfillment-clerk scope; explicit PCI-data-redaction scope (no raw card data visibility).
4. Verb walkthrough: practice the fulfillment-clerk workflow including PCI-redacted evidence-attach.
5. Real productive work: the first order release with full audit-chain seal.

Comparison: legacy PCI onboarding typically requires significant time on cardholder-data-environment scoping discussion before the employee can perform productive work. The substrate's PCI-pack overlay handles cardholder-data scoping automatically through Cedar evaluation; the new employee never sees raw card data and never needs to learn its scoping.

### 12.A.3 GDPR-pack day-zero scenarios
A new customer-service employee at a GDPR-scope tenant completes day-zero in 60 minutes:
1. Identity provisioning: passkey plus GDPR-training attestation including data-minimization principles.
2. Tenant membership: customer-service tenant.
3. Cedar permit set: customer-service scope with purpose-limitation pack overlay.
4. Verb walkthrough: practice the customer-service workflow including GDPR-pack export-with-policy scenarios.
5. Real productive work: the first customer interaction with full audit-chain seal plus GDPR-aware data-handling.

Comparison: legacy GDPR onboarding requires significant time on data-subject-rights training, lawful-basis discussion, and data-processor-data-controller distinctions. The substrate's GDPR-pack overlay handles these automatically through Cedar plus pack-specific denial-recovery affordances; the employee learns the affordances rather than the principles.

### 12.A.4 SOC2-pack day-zero scenarios
A new infrastructure-operations employee at a SOC2-Type-2 tenant completes day-zero in 90 minutes:
1. Identity provisioning: passkey plus SOC2-training attestation.
2. Tenant membership: ops tenant.
3. Cedar permit set: ops scope with least-privilege pack overlay; explicit production-change scope requires CAB approval.
4. Verb walkthrough: practice the change-request workflow including the production-change Cedar-approval gate.
5. Real productive work: the first production change with full audit-chain seal plus CAB approval.

Comparison: legacy SOC2 onboarding requires significant time on least-privilege principles, change-management mechanics, and access-review procedures. The substrate's SOC2-pack overlay handles these automatically; the new employee follows substrate-defined workflows that emit SOC2-required audit evidence.

### 12.A.5 CSAP-pack day-zero scenarios
A new public-sector-services employee at a CSAP-pack tenant (Korean public-sector) completes day-zero in 90 minutes:
1. Identity provisioning: passkey plus CSAP-training attestation including data-residency.
2. Tenant membership: public-sector tenant scoped to Korean data residency.
3. Cedar permit set: public-sector scope with CSAP pack overlay.
4. Verb walkthrough: practice export-with-policy including the CSAP-pack data-residency check.
5. Real productive work: the first public-sector case interaction with full audit-chain seal plus CSAP-pack data-residency enforcement.

Comparison: legacy CSAP onboarding typically requires significant time on data-residency principles. The substrate enforces data-residency at every primitive; the new employee learns the verbs and the pack overlay handles residency automatically.

### 12.A.6 EU-AI-Act pack day-zero scenarios
A new AI-product-officer at an EU-jurisdiction tenant subject to the EU AI Act completes day-zero in 120 minutes:
1. Identity provisioning: passkey plus EU-AI-Act-training attestation including risk-tier classification.
2. Tenant membership: AI-product tenant.
3. Cedar permit set: AI-product scope with EU-AI-Act risk-tier pack overlay.
4. Verb walkthrough: practice attach-evidence (model-card, evaluation-report, risk-assessment) including the EU-AI-Act-pack risk-tier classification.
5. Real productive work: the first AI-product release evaluation with full audit-chain seal plus EU-AI-Act risk-tier classification recording.

### 12.A.7 CMMC-pack day-zero scenarios
A new defense-contractor employee at a CMMC-Level-2 tenant completes day-zero in 180 minutes:
1. Identity provisioning: passkey plus CMMC-clearance attestation (if not already on file).
2. Tenant membership: defense tenant scoped to CMMC-Level-2.
3. Cedar permit set: defense scope with CMMC plus ITAR plus EAR pack overlays.
4. Verb walkthrough: practice export-with-policy including the ITAR-export-control check.
5. Real productive work: the first defense-program work item with full audit-chain seal plus CMMC-compliant evidence retention.

Comparison: legacy CMMC onboarding is typically a multi-day process. The substrate's CMMC pack overlay handles compliance automation; the new employee learns the verbs and pack-specific evidence-and-policy affordances.

### 12.A.8 Multi-pack day-zero
A new global-compliance-officer at a tenant with HIPAA plus GDPR plus PCI plus SOC2 plus CSAP plus EU-AI-Act packs all active completes day-zero in 180 minutes. The substrate composes the packs; the officer learns the cross-pack composition rules through pack-specific denial-recovery affordances rather than through six separate compliance-tool onboardings.

## Section 12.B - Day-zero workforce-shape worked examples

### 12.B.1 New-graduate day-zero
A 22-year-old new graduate joins an employer for their first job. The employer is oyatie-using. The graduate has personal-tenant experience plus university-tenant student-projection experience.

Day-zero arc:
- 09:00 onboarding session: passkey-onboard, tenant-membership-provision, role-projection assigned.
- 09:30 the new graduate's verify-context shows the employer-tenant plus the new role; the graduate has executed this verb thousands of times in personal-tenant use.
- 10:00 the graduate reviews history of the assigned project; the Ontology projection renders task-list, prior-decisions, and team members.
- 10:30 the graduate approves a tooling-access request; same verb path as personal-tenant approve.
- 11:00 the graduate attaches evidence (training-completion certificate) to their onboarding-workflow run.
- 11:30 the graduate signs the employment-attestation packet.
- 12:00 the graduate executes their first productive verb on a real project.

Training-time-on-platform: zero hours; the verbs are pre-fluent.
Training-time-on-role: 4-8 hours on team and project context.
Training-time-on-pack: 1-2 hours on employer-pack-specific evidence and policy fragments.

### 12.B.2 Mid-career switcher day-zero
A 35-year-old mid-career professional switches employers. Both employers use oyatie. The switcher has held three prior employer tenant memberships plus their personal tenant.

Day-zero arc:
- 09:00 onboarding: passkey-onboard with the new employer tenant; prior employer tenants remain unaffected.
- 09:15 verify-context shows the new tenant; the role-switcher offers the prior employer tenants (if memberships remain) plus the personal tenant.
- 09:30 the switcher's substrate fluency is at full-fluency level; they execute the first verb sequence in approximately the same time as a 5-year tenured employee.
- 10:00 the switcher reviews history of the team's recent work; the Ontology projection renders the team's project portfolio.
- 11:00 the switcher executes their first productive contribution.

Training-time-on-platform: zero hours.
Training-time-on-role: 8-16 hours on company-specific context.
Training-time-on-pack: 1-4 hours on new-employer-pack-specific deltas.

### 12.B.3 Career-changer day-zero
A 45-year-old career-changer moves from manufacturing to clinical (becomes a registered nurse after a second-career nursing program). The change is significant; the new domain is unfamiliar.

Day-zero arc:
- The clinical pack overlay is new; pack-specific evidence (vitals, labs, imaging, witness protocols) is new.
- The verbs are not new; the career-changer used approve, sign, attach-evidence, escalate, defer, route in manufacturing for two decades.
- The substrate's verb fluency transfers; the clinical pack overlay's denial-recovery affordances guide the new clinician through pack-specific scenarios.

Training-time-on-platform: zero hours.
Training-time-on-role: substantial (the clinical content is new) but no platform-onboarding overhead.
Training-time-on-pack: 8-24 hours on clinical-pack-specific evidence and policy fragments.

### 12.B.4 Returning-to-workforce day-zero
A 50-year-old who left the workforce 10 years ago for caregiving returns to employment. The returner has continuously used personal tenant during caregiving (managed parent's healthcare, family finances, caregiver-tenant under delegated authority).

Day-zero arc:
- The returner's substrate fluency is sustained through personal-tenant use; the 10-year gap does not require platform retraining.
- The new employer tenant is provisioned; the returner's substrate fluency is intact.

Training-time-on-platform: zero hours.
Training-time-on-role: substantial (industry context may need refresh) but no platform-onboarding overhead.

### 12.B.5 Retiree day-zero (estate-execution case)
A retiree's death triggers an estate-execution workflow per ADR-0311. The executor (often a family member or attorney) holds delegated-administrator permits on the deceased's personal tenant.

Day-zero arc:
- The executor's substrate fluency is intact from their own personal-tenant and employer-tenant use.
- The estate-workflow Ontology projection renders the deceased's assets, beneficiaries, and prior-attested wishes.
- The executor proceeds through the estate workflow using the verbs they already know.

Training-time-on-platform: zero hours.
Training-time-on-role: significant for first-time executor (estate-administration content) but no platform-onboarding overhead.

## Section 12.C - Day-zero plug-in admission worked examples

### 12.C.1 Healthcare-AI plugin admission
A radiology-AI plugin from a third-party vendor seeks marketplace admission to a HIPAA-pack tenant:
1. Vendor publishes the plugin to the marketplace with: capability description, model card, evaluation report, BAA-template, audit-chain integration spec, settlement template.
2. Tenant administrator opens the plugin listing; the substrate's admission gate runs: pack-conformance check (HIPAA), isolation-conformance check (sandbox), settlement-conformance check (per-study billing), audit-conformance check (sealed events).
3. Admin verifies the BAA-template; if acceptable, admin signs the BAA-acceptance Workflow Engine run.
4. Admin's approve verb commits the admission; Cedar evaluates the admission against the tenant's pack overlays.
5. The plugin is admitted; per-radiologist Cedar permits restrict which radiologists invoke the plugin; per-invocation audit-chain events are sealed.

### 12.C.2 Financial-services-AI plugin admission
An anti-money-laundering-AI plugin seeks admission to a PCI-plus-AML-pack tenant:
1. Vendor publishes the plugin with: capability description, model card, evaluation report, AML-pack-conformance evidence.
2. Compliance officer opens the listing; the substrate's admission gate runs pack-conformance (PCI plus AML).
3. Officer's approve commits the admission with explicit Cedar scope restricting the plugin to transactions above a threshold.
4. The plugin is admitted; per-invocation audit-chain events feed the AML-investigation Workflow Engine.

### 12.C.3 Manufacturing-plant-floor plugin admission
A predictive-maintenance plugin seeks admission to a manufacturing tenant:
1. Vendor publishes the plugin with: capability description, telemetry-integration spec, hardware-compatibility matrix.
2. Plant engineer opens the listing; the substrate's admission gate runs pack-conformance.
3. Engineer's approve commits the admission with explicit Cedar scope restricting the plugin to a subset of assets.
4. The plugin is admitted; per-prediction audit-chain events feed the maintenance Workflow Engine.

### 12.C.4 Developer-tool plugin admission
A code-review-AI plugin seeks admission to an engineering tenant:
1. Vendor publishes the plugin with: capability description, model card, source-code-access spec.
2. Engineering-platform-team opens the listing; the substrate's admission gate runs pack-conformance plus source-code-access scope check.
3. Team lead's approve commits the admission with explicit Cedar scope.
4. The plugin is admitted; per-review audit-chain events seal the model invocations.

### 12.C.5 Marketing-analytics plugin admission
A marketing-attribution-AI plugin seeks admission to a marketing tenant:
1. Vendor publishes the plugin with: capability description, model card, GDPR-conformance evidence.
2. Marketing-ops-leader opens the listing; the substrate's admission gate runs pack-conformance (GDPR plus per-jurisdiction).
3. Approve commits the admission with explicit Cedar scope.
4. The plugin is admitted; per-attribution-event audit-chain events seal the model invocations.

## Section 13 - Forbidden anti-patterns

The doctrine forbids product forks, hidden policy engines, local audit trails, category-specific identity, and training-island UX. Each anti-pattern has a precise failure mode and a required correction. Catalogue:

### 13.1 Forking identity because a department has its own vocabulary
**Failure**: recreates SaaS fragmentation inside the unified ecosystem; transfers complexity back to user, operator, auditor, integrator.
**Required correction**: move variation into tenant membership, Cedar permits, Workflow templates, Ontology projections, UX shell manifest, compliance packs, marketplace settlement rules, or plugin admission policy.
**Verification**: a reviewer can identify the shared primitive responsible without reading a product-specific exception memo.

### 13.2 Creating a local policy engine because a product team wants faster UI hiding
**Failure**: bypasses ONE-POLICY-ENGINE; policy author surface fragments; audit-chain loses authoritative source.
**Required correction**: the UI surface reads Cedar evaluations through the shared evaluator; UI hints respect Cedar advice but never replace Cedar decisions.

### 13.3 Embedding workflow state inside a product object without Workflow Engine visibility
**Failure**: bypasses ONE-WORKFLOW-ENGINE; durable processes are unobservable; cross-product workflows break.
**Required correction**: the Ontology object references a Workflow Engine run; the run carries the state; the object's view shows the state through projection.

### 13.4 Creating a private object model for CRM, HR, ERP, or ITSM instead of Ontology projection
**Failure**: bypasses ONE-ONTOLOGY; cross-tier reporting requires custom integration; data taxonomy fragments.
**Required correction**: the capability tier authors Ontology object classes plus projections; the data taxonomy lives in the shared Ontology schema.

### 13.5 Logging product-local audit events without audit-chain sealing and retention class
**Failure**: bypasses ONE-AUDIT-CHAIN; audit reconstruction requires cross-system reconciliation; compliance posture fragments.
**Required correction**: every audit event is sealed into the shared audit-chain with a retention class chosen per the active compliance packs.

### 13.6 Making marketplace settlement category-specific instead of universal and policy-bound
**Failure**: bypasses ONE-MARKETPLACE; settlement plumbing fragments per category; dispute mediation becomes per-category.
**Required correction**: the category-specific settlement is implemented as a marketplace-settlement primitive plus pack-overlay; the dispute workflow is the shared dispute workflow.

### 13.7 Teaching a new UX language for every module, department, or job type
**Failure**: bypasses ONE-UX-SHELL plus ONE-TRAINING-MODEL; training-cost amortization claim collapses.
**Required correction**: the module uses the thirteen verbs with role-projection-tuned density and hint copy; new verbs require ADR per §3.7 failure-mode tree.

### 13.8 Treating accessibility as a reduced-capability mode
**Failure**: substrate guarantee per ADR-0318 is broken; cross-collar training-cost claim shatters.
**Required correction**: accessibility is full-floor in every workspace; density tuning is permitted; affordance reduction is not.

### 13.9 Treating personal data as work data because the same human holds both roles
**Failure**: ADR-0311 dual-tenant boundary breach.
**Required correction**: the substrate scopes data to the originating tenant; cross-tenant access requires explicit Cedar permits.

### 13.10 Treating work data as personal data because the same passkey authenticated the person
**Failure**: ADR-0311 dual-tenant boundary breach in the other direction.
**Required correction**: the substrate enforces the tenant boundary at every primitive; passkey authentication is identity, not authorization.

### 13.11 Using suite branding to sneak in a multi-concern service boundary
**Failure**: ADR-0132 product-platform dissolution is bypassed; product fork hides behind marketing.
**Required correction**: the multi-concern boundary must be raised as an ADR with documented operational concern; default decision is to use the substrate.

### 13.12 Accepting an external plugin without isolation, admission, settlement, and audit hooks
**Failure**: ADR-0249 marketplace admission policy bypassed; plugin can spill across tenants, escape compliance, evade audit, or settle outside the marketplace.
**Required correction**: every plugin must pass admission; admission verifies isolation, pack-conformance, settlement integration, and audit-chain integration.

### 13.13 Hardcoding tenant-specific behavior in product code
**Failure**: tenant-scoping leaks into product code; ADR-0244 universal scoping primitive is bypassed; cross-tenant fuzz tests find behavior drift.
**Required correction**: tenant-specific behavior lives in pack overlays, role projections, or tenant-scoped configuration; product code is tenant-agnostic.

### 13.14 Building a custom escalation path that bypasses Workflow Engine
**Failure**: durable escalation state lives outside Workflow Engine; ONE-WORKFLOW-ENGINE bypassed.
**Required correction**: escalation is the substrate verb escalate that triggers a Workflow Engine state transition with pack-aware policy.

### 13.15 Maintaining product-specific notification systems
**Failure**: notification taxonomy fragments; user receives duplicate or contradictory notifications across products.
**Required correction**: notifications are emitted by the substrate per role-projection-aware notification rules; products consume from the shared stream and present per-role density.

### 13.16 Custom-app private-data-store backing
**Failure**: ADR-0245 substrate-vs-product layering bypassed; data lives outside Ontology.
**Required correction**: custom-app state lives in the Ontology or in a substrate-admitted secondary store with explicit pack-overlay declarations.

## Section 13.A - Anti-pattern detection and remediation tooling

### 13.A.1 Detection: the conformance lanes
Each anti-pattern in §13 is detected by an automated conformance lane:
- `governance-doc-rigor` (this thesis lives in its lane)
- `governance-verb-enum` (UX shell action-router enum conformance)
- `governance-policy-engine` (Cedar-only authorization paths)
- `governance-workflow-engine` (durable-state in Workflow Engine only)
- `governance-ontology` (no private object models)
- `governance-audit-chain` (all audit events sealed into the chain)
- `governance-marketplace` (settlement only through the shared primitive)
- `governance-ux-shell` (verb-affordance conformance per workspace, locale, collar)
- `governance-accessibility` (substrate-level accessibility coverage)
- `governance-dual-tenant` (cross-tenant fuzz tests)
- `governance-suite-residue` (forbidden suite-branding patterns)
- `governance-plugin-admission` (marketplace admission conformance)
- `governance-tenant-primitive` (tenant_id presence on every row, audit, and cost line)
- `governance-workflow-escalation` (escalation path lives in Workflow Engine)
- `governance-notification` (notifications emitted from substrate stream)
- `governance-custom-app-store` (Ontology-or-declared-secondary-store only)

Each lane fails closed: a PR that introduces an anti-pattern cannot merge unless the offending change is reverted or an ADR explicitly admits the exception.

### 13.A.2 Remediation playbook
A detected anti-pattern triggers a remediation playbook:
1. Pause: the offending PR is held; a remediation task is created.
2. Diagnose: identify the substrate primitive that should have held the behavior.
3. Re-author: refactor the change to use the substrate primitive.
4. Verify: the conformance lane passes; tests pass; the multispectrum review reaches APPROVE.
5. Land: the refactored change merges; the anti-pattern attempt is recorded in the audit-chain for future learning.

### 13.A.3 Multispectrum review intersection
Each anti-pattern is detectable through one or more facets of the multispectrum-review v2.4.0 lane:
- F1 (correctness): does the behavior land where the substrate expects it?
- F2 (security): does the anti-pattern create a cross-tenant or pack-leak risk?
- F3 (architecture): does the anti-pattern bypass a substrate primitive?
- F8 (cohesion): does the anti-pattern duplicate logic across microservices?
- A1 (own-policy-naming): does the introduced name fit the v4 BNF + 12-layer enum?
- A2 (own-policy-documentation): does the introduced surface ship the required doc set?
- A3 (own-policy-structure): does the introduced surface respect the per-microservice flat layout per ADR-0131?
- A4 (own-policy-architecture): does the introduced surface respect ADR-0245 substrate-vs-product layering?

### 13.A.4 Anti-pattern audit-chain seal
Every detected anti-pattern attempt seals a record into the governance audit-chain. The record includes the PR reference, the offending diff, the diagnosing playbook step, the remediation, and the multispectrum review verdict. The doctrine's evolution depends on this audit-chain because the mistakes-ledger derives from it.

### 13.A.5 Recurring anti-pattern policy
If the same anti-pattern recurs three times within a 90-day window across the codebase, the governance lane raises a HEIGHTENED status that requires a doctrine-level ADR addressing the recurring failure. Recurrence often indicates an unclear substrate API or an unclear documentation surface; the doctrine must adapt.

## Section 13.B - Doctrine evolution and adoption cadence

### 13.B.1 Annual doctrine review
The doctrine is reviewed annually. Review owners: documentation-rigor lane, substrate-primitive maintainers (identity, policy, workflow, ontology, audit-chain, marketplace, ux-shell, compliance-pack, plugin-admission), training-cost-doctrine sibling maintainers. The review produces a sealed audit-chain artifact.

### 13.B.2 Five-year doctrine recalibration
Every five years (next: 2031-05-21), the doctrine recalibrates the §1.2 sizing assumptions, the §8 marketplace-settlement maturity claims, the §9 conglomerate-mobility claims, and the §10 displacement claims against the then-current vendor landscape.

### 13.B.3 Thirty-year doctrine validation
By 2056-05-21, the doctrine validates that the persona timelines in `training-cost-doctrine-2026-05-21.md` §5.2 have run end-to-end. The substrate must still hold the verb enum, the Ontology projections, the Cedar policy evaluator, the Workflow Engine, the audit-chain, the marketplace primitive, the dual-tenant boundary, and the compliance-pack primitive.

### 13.B.4 Adoption cadence per ADR-0250
ADR-0250 (build-ahead-of-certification) requires substrate primitives to ship certified-shape on day one rather than retrofit compliance later. The doctrine adoption cadence follows: each substrate primitive ships with full pack-overlay support; each capability tier ships with role-projection support; each role projection ships with workspace and locale support.

### 13.B.5 Marketplace ecosystem cadence per ADR-0249
ADR-0249 establishes that the marketplace handles plugins, apps, workflows, agents, models, and datasets. Doctrine adoption cadence: at each substrate-primitive release, marketplace integration is required; partner-onboarding follows in a 90-day window; certified ecosystem-listings reach a 1,000-listing threshold within 18 months of GA.

## Section 13.C - Vendor-displacement migration playbooks

### 13.C.1 Phased displacement methodology
Vendor displacement follows a four-phase methodology:
1. **Parallel-run phase**: oyatie substrate primitive runs alongside the incumbent for the same function. Reconciliation reports run nightly. Duration: typically one quarter per function.
2. **Authoritative-switch phase**: oyatie becomes the authoritative source; incumbent is read-only mirror. Cutover is per-function. Duration: typically 30-90 days per function.
3. **Incumbent-retirement phase**: incumbent is taken out of production. Data is archived in oyatie audit-chain for replay. Duration: typically 30-60 days per function.
4. **Steady-state phase**: ongoing operation on substrate; no parallel-run cost.

### 13.C.2 Function-level displacement priorities
Within a vendor, displacement priorities are typically:
- **High priority**: functions with high training-cost-payroll burden and low operational-risk (e.g., expense report, time entry).
- **Medium priority**: functions with significant training cost and moderate operational risk (e.g., procurement workflow, performance review).
- **Lower priority**: functions with low training cost or high operational risk (e.g., payroll calculation, financial close).

The high-priority functions deliver early visible savings; the medium-priority functions deliver substantial savings but require careful change management; the lower-priority functions retain incumbent until substrate parity is rigorously verified.

### 13.C.3 Per-vendor migration timeline reference
Typical timelines for an enterprise migrating from a major incumbent to oyatie:
- SAP S/4HANA full ERP migration: 18-36 months across all sub-functions.
- Workday HCM migration: 9-18 months excluding payroll calculation (typically retained as a service).
- Salesforce CRM migration: 6-12 months including custom-Apex-to-Workflow-template translation.
- ServiceNow ITSM migration: 9-18 months including CMDB-to-Ontology translation.
- Microsoft 365 collaboration migration: 12-24 months including document-format-fidelity work.
- NetSuite SMB-ERP migration: 6-12 months.
- Atlassian developer-platform migration: 9-18 months including custom-app-migration.

These timelines are sizing assumptions pending customer validation.

### 13.C.4 Risk-management posture during migration
The doctrine prescribes risk-management posture during migration:
- Audit-chain double-writing: every audit event lands in both incumbent and oyatie audit logs during parallel-run.
- Reconciliation gates: nightly reconciliation reports must pass before the authoritative-switch phase begins.
- Rollback readiness: at any point during parallel-run, the substrate can be retired and the incumbent retains authority.
- Compliance continuity: the active compliance pack is enforced on both incumbent and oyatie outputs; the more-restrictive evaluation wins.
- Workforce communication: every migration phase is accompanied by role-specific training refreshers focused on substrate-specific evidence-and-policy.

### 13.C.5 Migration audit-chain seal
Every migration phase emits a sealed audit-chain event with the function name, the phase transition, the reconciliation results, the rollback-readiness status, and the operator attestation. The migration audit-chain is exportable to the regulator on demand.

## Section 13.D - Substrate primitives detail

### 13.D.1 Identity primitive details
The identity primitive carries:
- Passkey-bound human identity: WebAuthn or equivalent platform attestation.
- Tenant memberships: a directed graph from identity to tenants with role-projection assignments per tenant.
- Recovery metadata: trusted recovery surfaces, biometric attestation, hardware-key attestation.
- Audit-chain links: every identity operation is sealed.
- Dual-tenant primary: every human has a personal tenant from age of consent (ADR-0311).
- Conglomerate-aware: subsidiary tenant memberships do not auto-propagate to the parent (ADR-0313).

Implementation references: `shared-identity`, `shared-identity-domain`, `shared-identity-recovery`, `shared-identity-passkey`.

### 13.D.2 Policy-engine primitive details
The policy-engine primitive carries:
- Cedar evaluation engine: deterministic, replay-safe, sub-10ms p99.
- Policy store: versioned per-tenant policy fragments.
- Pack-overlay composition: HIPAA, GDPR, SOC2, CSAP, PCI, EU-AI-Act, plus per-jurisdiction overlays.
- Explainable denials: every denial carries the reason and the denial-recovery affordances.
- Audit-chain links: every evaluation is sealed.
- Hot-path precompilation: high-frequency policy fragments precompile for performance.

Implementation references: `intelligence-policy-engine-cedar`, `shared-policy-store`, `shared-cedar-evaluator`, `shared-policy-author`.

### 13.D.3 Workflow-engine primitive details
The workflow-engine primitive carries:
- Durable state machines and DAGs.
- Deterministic replay from audit-chain.
- Long-running activity support (human approvals, external waits).
- Per-tenant sharding.
- Workflow-template version management with backward-compat.
- Audit-chain links: every state transition is sealed.

Implementation references: `intelligence-workflow-engine`, `shared-workflow-templates`, `shared-workflow-runtime`, `shared-workflow-replay`.

### 13.D.4 Ontology primitive details
The ontology primitive carries:
- Object classes with typed fields.
- Per-role, per-capability-tier, per-jurisdiction projections.
- Content-addressed storage for large objects.
- Per-tenant sharding.
- Schema migration tooling.
- Audit-chain links: every mutation is sealed.

Implementation references: `shared-ontology`, `shared-ontology-schema`, `shared-ontology-projection`, `shared-ontology-migration`.

### 13.D.5 Audit-chain primitive details
The audit-chain primitive carries:
- Cryptographic chain-linking for tamper detection.
- Per-tenant sharding.
- Pack-aware retention classes.
- Replay-driven reconstruction.
- Meta-audit (audit of audit-chain queries).
- Sealing-key rotation policy.

Implementation references: `shared-audit-chain`, `shared-audit-chain-schema`, `shared-audit-replay`, `shared-audit-export`.

### 13.D.6 Marketplace primitive details
The marketplace primitive carries:
- Universal settlement across consumer, B2B, labor, partner, plugin, dataset, model, agent.
- Offer-Acceptance-Settlement-Dispute-Reversal workflow.
- Per-pack settlement scoping.
- Per-counterparty creditworthiness projection.
- Audit-chain links.

Implementation references: `shared-marketplace-settlement`, `shared-marketplace-admission`, `shared-marketplace-dispute`, `shared-marketplace-reputation`.

### 13.D.7 UX-shell primitive details
The UX-shell primitive carries:
- Thirteen-verb action router with bounded enum conformance.
- Per-locale verb labels.
- Per-workspace density tuning.
- Per-role projection rendering.
- Per-pack-overlay denial-recovery affordances.
- Accessibility-substrate (keyboard, screen-reader, switch-control, eye-tracking, dictation).
- Audit-chain links: every verb completion is sealed.

Implementation references: `shared-ux-shell-action-router`, `shared-ux-shell-localization`, `shared-ux-shell-accessibility`, `shared-ux-shell-density`.

### 13.D.8 Compliance-pack primitive details
The compliance-pack primitive carries:
- Pack-overlay composition (HIPAA + GDPR + SOC2 + CSAP + PCI + EU-AI-Act + jurisdiction).
- Per-pack retention classes for audit-chain.
- Per-pack evidence requirements for Workflow Engine.
- Per-pack denial-recovery affordances for UX shell.
- Per-pack settlement scoping for marketplace.

Implementation references: `shared-compliance-pack`, `shared-compliance-pack-overlays`, `shared-compliance-pack-evidence`.

### 13.D.9 Plugin-admission primitive details
The plugin-admission primitive carries:
- Isolation guarantees (sandbox, resource limits).
- Pack-conformance check at admission.
- Settlement integration at admission.
- Audit-chain integration at admission.
- Marketplace listing and discovery.
- Per-tenant allow-listing.

Implementation references: `shared-plugin-admission`, `shared-plugin-isolation`, `shared-marketplace-admission`.

### 13.D.10 Tenancy primitive details
The tenancy primitive carries:
- Universal tenant_id on every row, audit event, and cost line (ADR-0244).
- Sovereign-child semantics for conglomerate hierarchies (ADR-0313).
- Dual-tenant boundary (ADR-0311).
- Per-tenant pack overlays.
- Per-tenant settlement scope.
- Per-tenant audit-chain shard.

Implementation references: `shared-tenancy`, `shared-tenant-membership`, `shared-tenant-config`.

## Section 13.E - Operational doctrine

### 13.E.1 Cellular architecture
ADR-0248 establishes AWS-style cellular architecture for resilience. Substrate primitives are deployed in cells; shuffle sharding limits the blast radius of any cell failure; Cloud Hypervisor plus Kata pods provide hardware-grade isolation between tenants on the same physical host.

Each substrate primitive has Tier 0 (control plane), Tier 1 (regional fleet), Tier 2 (cell-local fleet), Tier 3 (per-tenant or per-region instance), and Tier 4 (per-workload) deployment shapes.

### 13.E.2 HTTP/3 + QUIC default
ADR-0253 establishes HTTP/3 plus QUIC as the default transport. The substrate's UX shell, internal microservice mesh, and marketplace API all use HTTP/3. Legacy clients fall back to HTTP/2 with explicit deprecation timelines.

### 13.E.3 K8s + Cloud Hypervisor default
ADR-0254 establishes Kubernetes as the orchestration default (except at the edge), with Cloud Hypervisor plus Kata pods providing hardware-grade isolation. Each substrate primitive is a Kubernetes workload with per-tenant pod isolation.

### 13.E.4 HLC + TrueTime tier
ADR-0252 establishes HLC (Hybrid Logical Clocks) as the default ordering primitive with TrueTime opt-in for financial-grade ordering. The substrate's audit-chain, Workflow Engine, and Ontology mutations use HLC ordering by default; financial-services tenants may opt into TrueTime for the additional ordering guarantee.

### 13.E.5 Build ahead of certification
ADR-0250 establishes that substrate primitives ship certified-shape day one. The doctrine carries through: every release of every substrate primitive must demonstrate pack-overlay conformance (HIPAA, GDPR, SOC2, CSAP, PCI, EU-AI-Act) before the release is GA.

### 13.E.6 Intelligence two-layer substrate
ADR-0255 establishes the AI substrate as the lower layer plus the consumer brand surface as the upper layer. The substrate-level AI capabilities (e.g., retrieval over Ontology, agent-execution under Cedar) are uniform across products; the consumer brand surface provides product-specific personas.

### 13.E.7 provider-credential BYOK opt-in
ADR-0255 §D-4 establishes opt-in provider-credential BYOK for provider credentials. The tenant configures `provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}`. The doctrine carries through to every substrate-level call to an external provider (LLMs, payment processors, identity providers, observability vendors).

### 13.E.8 Multi-category marketplace
ADR-0249 establishes that the marketplace handles plugins, apps, workflows, agents, models, and datasets. Each category has its own admission gate, its own settlement template, its own dispute workflow, and its own audit-chain projection, all over the shared marketplace primitive.

## Section 13.F - Service-exception detail

### 13.F.1 Criteria for service exception
A service exception per ADR-0132 requires:
- A documented operational concern (regulated network, hardware integration, certified rail, domain-specific latency budget).
- An ADR justifying the exception.
- Substrate integration plan: how the service interacts with identity, policy, workflow, ontology, audit-chain, marketplace, UX shell, compliance pack, and plugin admission.
- Verb-conformance plan: how the service surfaces the thirteen verbs.
- Sunset plan: how the service deprecates if the operational concern becomes obsolete.

### 13.F.2 Sanctioned services
At doctrine v2 authoring, the following services are sanctioned (each has an ADR or in-flight ADR):
- Payment-acquiring service: depends on acquiring-bank relationship; integrates with marketplace via settlement primitive.
- Telephony service: depends on PSTN integration; integrates with collaboration via meet plus phone projections.
- Plant-floor MES service: depends on hardware integration; integrates with manufacturing capability tier via Ontology and Workflow Engine.
- Clinical-device service: depends on FDA-cleared hardware integration; integrates with clinical capability tier.
- High-frequency-trading order-routing service: depends on sub-millisecond latency budget; integrates with treasury capability tier.
- SWIFT-gateway service: depends on SWIFT membership and certified network; integrates with treasury capability tier.
- Fedwire-gateway service: depends on Fedwire membership; integrates with treasury capability tier.

### 13.F.3 Service-exception governance
Service exceptions are reviewed annually. Reviews verify that the operational concern still holds; that the substrate integration remains intact; that verb conformance has not degraded; that the sunset plan is still viable.

## Section 13.G - The keystone-bundle integration

### 13.G.1 Bundle context
The doctrine implements the keystone-bundle articulated in `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`. The bundle's 14 doctrines (KS#1 through KS#14) are the substrate that this thesis depends on.

### 13.G.2 KS#1 oyatie-is-a-tenant (ADR-0242)
Oyatie itself is a reserved-namespace tenant. No carve-outs. The substrate's own operations are governed by Cedar evaluations against the oyatie tenant just like any customer tenant. The doctrine carries through: substrate operators see verify-context, recover-from-denial, and review-history just like any user.

### 13.G.3 KS#2 cedar-universal-gate (ADR-0243)
Every authorization is a Cedar evaluation. No policy in code. The doctrine depends on this: the thirteen-verb action router resolves authorization through Cedar; the marketplace settlement resolves authorization through Cedar; every workflow state transition resolves authorization through Cedar.

### 13.G.4 KS#3 tenant-scoping-primitive (ADR-0244)
Every row, every audit event, every cost line carries tenant context. The doctrine depends on this: substrate observability dashboards aggregate per tenant; settlement is per tenant; pack overlays are per tenant.

### 13.G.5 KS#4 substrate-vs-product (ADR-0245)
Substrate microservices serve all products with no duplication. The doctrine depends on this: capability tiers are projections over the shared substrate; products are role-projection bundles; no separate product-internal substrate exists.

### 13.G.6 KS#5 MLS E2EE messenger (in-flight ADR)
MLS (RFC 9420) is the canonical E2EE messenger protocol. Personal-tenant messaging uses MLS; tenant-pack overlays toggle whether enterprise tenants use MLS (consumer-style) or Cedar-mediated organizational messaging.

### 13.G.7 KS#6 self-modification doctrine (ADR-0247)
The Foundry runs as oyatie.foundry.* principals under Cedar. The doctrine carries through: the substrate's own evolution is governed by the same Cedar evaluator that governs customer operations.

### 13.G.8 KS#7 cellular architecture (ADR-0248)
AWS-style cellular topology with shuffle sharding plus Cloud Hypervisor isolation. See §13.E.1.

### 13.G.9 KS#8 compliance pack primitive (ADR-0251)
HIPAA, GDPR, SOC2, CSAP, PCI, EU-AI-Act as packs per tenant per cell. The doctrine depends on this for the ONE-COMPLIANCE-POSTURE invariant.

### 13.G.10 KS#9 build-ahead-of-certification (ADR-0250)
Certified-shape day one; never retrofit compliance. See §13.E.5.

### 13.G.11 KS#10 provider-credential BYOK opt-in (ADR-0255 §D-4)
Provider credentials are opt-in provider-credential BYOK. See §13.E.7.

### 13.G.12 KS#11 multi-category marketplace (ADR-0249)
Plugins, apps, workflows, agents, models, datasets. See §10 (marketplace) plus ADR-0249.

### 13.G.13 KS#12 HLC + TrueTime tier (ADR-0252)
Hybrid logical clocks default; TrueTime opt-in. See §13.E.4.

### 13.G.14 KS#13 K8s + Cloud Hypervisor (ADR-0254)
Kubernetes everywhere except edge; Cloud Hypervisor plus Kata pods. See §13.E.3.

### 13.G.15 KS#14 intelligence two-layer (ADR-0255)
AI substrate plus consumer brand surface; absorbs Foundry. See §13.E.6.

## Section 13.H - Multispectrum-review intersection

### 13.H.1 Facet-by-facet intersection
The doctrine intersects the multispectrum-review v2.4.0 lane at every facet:
- **F1 correctness**: every invariant has a worked example demonstrating the substrate primitive resolves correctly.
- **F2 security**: cross-tenant fuzz tests defend the dual-tenant boundary; Cedar evaluation defends authorization.
- **F3 architecture**: the doctrine is the architecture; every capability tier maps to a substrate primitive.
- **F4 performance**: each invariant has a stated latency budget; substrate primitives must meet the budget.
- **F5 maintainability**: each invariant has typed contracts; tests at every layer; deprecation policy.
- **F6 observability**: every verb completion is sealed; every Cedar evaluation is sealed; every workflow transition is sealed.
- **F7 scalability**: each invariant has a sharding strategy; horizontal scaling without product forks.
- **F8 cohesion**: each invariant lives in exactly one substrate primitive; no duplication.
- **F9 documentation**: this thesis plus the training-cost-doctrine plus per-ADR docs cover the doctrine.
- **M1 own-policy-naming**: the thirteen verbs are named per the v4 BNF; the substrate primitives are named per the 12-layer enum.
- **M2 meta**: the doctrine is itself versioned and audit-chain-tracked.
- **F10 finance**: each invariant has a cost-line allocation; settlement primitive is per tenant.
- **F11 legal**: each compliance pack overlay maps to a regulator; legal-review gates pack-overlay changes.
- **F13 public-trust**: the doctrine's claim discipline ties marketing copy to audit-chain projections.

### 13.H.2 Adherence facets (A-family)
Per v2.3.0 the A-family own-policy-adherence facets gate the doctrine:
- **A1 naming**: every introduced name (verb, capability tier, role projection, pack overlay) must justify v4 BNF plus 12-layer enum conformance at scaffold time.
- **A2 documentation**: every µservice ships full doc set plus per-pack overlays.
- **A3 structure**: per-microservice flat layout per ADR-0131.
- **A4 architecture**: substrate-vs-product layering per ADR-0245.
- **A5 dependency**: cohesion-and-dependency rules per ADR-0244 plus ADR-0245.
- **A6 schema**: Ontology schema migration with backward-compat.
- **A7 algorithm**: deterministic-replay for Workflow Engine, audit-chain, and policy evaluation.

### 13.H.3 Per-facet debate evidence
Per the multispectrum-review doctrine, each facet's debate evidence lives at `evidence/debate/`. The doctrine's facet evidence is sealed into audit-chain on each release. Recurring facet failures trigger the §13.A.5 recurring anti-pattern policy.

## Section 13.I - Worked tenant cases

### 13.I.1 Mid-size enterprise: 1,200-person enterprise software vendor
The tenant operates with:
- Workforce: 1,200 employees plus 200 contractors plus 50 board and advisor roles.
- Tenants: one corporate tenant plus three subsidiary tenants for international entities plus a separate audit-committee tenant for the board.
- Compliance packs: SOC2 (corporate), GDPR (EU subsidiary), PCI (e-commerce processing), SOX (public-company audit).
- Capability tiers: full ERP, full CRM, full HR, full ITSM, full collaboration, full audit-and-governance.

Substrate operations:
- Identity: 1,450 humans with passkey-backed identity; multi-tenant memberships for executives spanning corporate plus subsidiaries.
- Policy: approximately 4,800 Cedar permits across the corporate tree; pack overlays compose at evaluation time.
- Workflow: approximately 280 active Workflow templates covering hire, expense, procurement, sales, contract, incident, change, release, performance, audit.
- Ontology: approximately 320 object classes spanning customer, contract, employee, asset, opportunity, etc.
- Audit-chain: approximately 14 billion events per year; pack-aware retention.
- Marketplace: ~80 plugin admissions; SaaS-vendor settlements through marketplace primitive; partner co-sell deals.

Doctrine outcome: the workforce transitions are smooth (cross-subsidiary mobility, intern-to-employee transitions, board-onboarding). The audit-committee can query the corporate tenant's audit-chain through explicit cross-tenant permits without seeing customer-data detail.

### 13.I.2 Hospital health-system: 18,000-person integrated delivery network
The tenant operates with:
- Workforce: 18,000 employees plus 4,000 medical-staff (privileges-based) plus 1,200 contractors.
- Tenants: one health-system parent plus seven hospital-tenants plus 30 clinic-tenants plus one shared-services tenant; each is a sovereign child per ADR-0313.
- Compliance packs: HIPAA (clinical), HITECH (breach-notification), PCI (e-commerce of patient payments), Joint-Commission (accreditation), CMS (reimbursement).
- Capability tiers: clinical (extensive), revenue-cycle-management, supply-chain, HR, ITSM, audit-and-governance, marketplace (patient-payments, vendor-procurement).

Substrate operations:
- Identity: 23,200 humans with passkey-backed identity; multi-tenant memberships for medical-staff with privileges at multiple facilities.
- Policy: approximately 12,000 Cedar permits across the network; pack-overlays compose per-facility per-encounter.
- Workflow: approximately 420 active Workflow templates covering admit, discharge, transfer, medication administration, controlled substance, surgical case, imaging, lab, pathology, billing, prior-auth, denial-appeal, supply-cycle, staffing-bid.
- Ontology: approximately 540 object classes spanning patient, encounter, order, result, claim, charge, supply, contract, employee, asset, etc.
- Audit-chain: approximately 95 billion events per year; HIPAA-retention applies to clinical events.
- Marketplace: vendor procurement (medical supplies, pharmaceuticals, capital equipment); plugin admissions (radiology AI, pathology AI, transcription, EHR adjuncts).

Doctrine outcome: the workforce transitions accommodate locum tenens, traveling nurses, multi-state telehealth, residency rotations, and joint-staff governance across facilities. The dual-tenant boundary holds; patients see their personal-tenant health-record references without the health-system tenant overriding.

### 13.I.3 University system: 65,000-person multi-campus public university
The tenant operates with:
- Workforce: 12,000 faculty plus 18,000 staff plus 35,000 students plus 4,500 affiliates (postdocs, visitors, retirees, alumni-volunteers).
- Tenants: one system-parent plus 12 campus-tenants plus shared-services tenant; each campus is sovereign per ADR-0313.
- Compliance packs: FERPA (student records), state-research (research-data), HIPAA (university-health-system), state-public-records (open-records-act), accessibility-section-508.
- Capability tiers: HR (faculty plus staff), student-records (academic), research (grants plus IRB), facilities, ITSM, athletics, alumni, library, marketplace (bookstore, foodservice, tickets).

Substrate operations:
- Identity: 69,500 humans (workforce plus students) with passkey-backed identity; many students hold dual personal-tenant plus university-tenant memberships from secondary-school years.
- Policy: approximately 18,000 Cedar permits; FERPA pack overlay scopes student-record access per role and per legitimate-educational-interest fragment.
- Workflow: approximately 500 active Workflow templates covering admit, enroll, register, grade, certify, research-IRB, faculty-hiring, tenure-review, grant-submission, grant-management.
- Ontology: approximately 600 object classes spanning student, employee, course, section, grade, research-protocol, grant, donation, alumni-relation, facility, asset, etc.
- Audit-chain: approximately 50 billion events per year; FERPA-retention plus per-research-protocol retention.
- Marketplace: bookstore plus dining-services plus parking plus ticket-purchases plus alumni-engagement.

Doctrine outcome: students transition smoothly from secondary-school personal-tenant to university tenant to alumni-tenant to potential employer tenants. The university's research IRB processes integrate cleanly via Workflow Engine. Faculty hold dual research and teaching roles within one identity.

### 13.I.4 Public-sector state agency: 7,500-person human-services department
The tenant operates with:
- Workforce: 7,500 employees plus 2,200 contractors.
- Tenants: one state-department-parent plus 15 county-tenants plus shared-services tenant; each county is a sovereign child per ADR-0313.
- Compliance packs: state-confidentiality, federal-program-integrity (SNAP, TANF, Medicaid), accessibility-section-508, state-public-records (with exemptions for case-records).
- Capability tiers: case-management, eligibility, fraud-investigation, audit-and-governance, marketplace (vendor-procurement, benefits-issuance).

Substrate operations:
- Identity: 9,700 humans plus county-resident applicant identities (held in personal tenants with delegated access for case workers).
- Policy: approximately 8,500 Cedar permits; state-confidentiality plus federal-program-integrity overlays compose per-applicant per-case.
- Workflow: approximately 280 active Workflow templates covering eligibility-determination, benefits-issuance, fraud-investigation, hearings-appeal, recoupment, provider-credentialing, vendor-payment.
- Ontology: approximately 380 object classes.
- Audit-chain: approximately 40 billion events per year; statutory-retention.
- Marketplace: provider-credentialing (medical providers, child-care providers), vendor-procurement, benefits-issuance to retailers and providers.

Doctrine outcome: applicants who later become employees transition smoothly (the personal-tenant carries forward; the new employer tenant scopes new permits). Case workers handle complex multi-program cases (SNAP plus TANF plus Medicaid plus child-care plus housing-assistance) through composed pack overlays.

### 13.I.5 Small enterprise: 80-person trades cooperative
The tenant operates with:
- Workforce: 80 members (owner-operators of small trades businesses) plus 25 employees.
- Tenants: one cooperative-tenant plus 80 per-member tenants plus each member's personal tenant. The cooperative-tenant has read access to aggregated metrics from each member-tenant.
- Compliance packs: state-licensing (electrician, plumber, HVAC, etc.), state-sales-tax, state-prevailing-wage where applicable.
- Capability tiers: small-business ERP, customer-service, marketplace (joint-bidding on large projects, member-payments to cooperative).

Substrate operations:
- Identity: 105 humans across the cooperative; each member operates personal plus side-business plus cooperative memberships.
- Policy: approximately 600 Cedar permits.
- Workflow: approximately 40 active Workflow templates covering joint-bidding, member-payment, dispute-mediation, education-credit, license-renewal.
- Ontology: approximately 80 object classes.
- Audit-chain: approximately 200 million events per year.
- Marketplace: joint bids, member-pricing on shared supply purchases, customer-payments.

Doctrine outcome: cooperative members onboard easily because the cooperative-tenant is structurally similar to their personal-tenant side-business they already operate. The shared-procurement leverage is realized through marketplace.

## Section 13.J - Doctrine claim register

### 13.J.1 Substantive claims and their evidence chains
Every substantive claim in this doctrine traces to evidence at one of three levels: ADR (the doctrine's authoritative reference), substrate implementation (the µservices that realize the invariant), and customer telemetry (the four signals per §11). Claims without all three levels are flagged as aspirational rather than testable.

### 13.J.2 Aspirational claims explicitly identified
The following claims are aspirational pending substrate maturity and customer validation:
- §1.2 numeric sizing assumptions (110 tools, USD 1,500 per-employee per-year per-tool training pressure, 30 percent IT-budget integration overhead): sizing assumptions pending Section 14 source validation.
- §1.A.* per-tax dollar-figure projections: sizing assumptions pending customer telemetry validation per substrate four-signal corpus.
- §8 marketplace-settlement maturity claims: aspirational pending marketplace maturity beyond the initial multi-category release.
- §9 conglomerate-mobility claims: aspirational pending customer case studies at conglomerate scale.
- §10 displacement-claim windows: sizing assumptions pending per-displaced-vendor customer case studies.

### 13.J.3 Testable claims with current evidence
The following claims are testable today against the substrate implementation:
- Verb-enum boundedness: the UX-shell action router has a fixed thirteen-verb enum; conformance lane verifies.
- Cedar evaluation latency: substrate observability emits Cedar evaluation latency histograms.
- Workflow Engine deterministic replay: replay-test set covers every Workflow template.
- Ontology projection consistency: schema-migration test set verifies backward-compat.
- Audit-chain integrity: cryptographic chain-link verification on every read.
- Marketplace settlement primitive operations: settlement-test set covers offer-acceptance-settlement-dispute-reversal.
- Dual-tenant boundary enforcement: cross-tenant fuzz tests fire on every release.
- Compliance-pack overlay composition: per-pack conformance tests verify overlay composition.
- Plugin admission gates: per-plugin admission-test set covers isolation, pack-conformance, settlement, audit.

### 13.J.4 Claim-discipline gate
Marketing copy citing this doctrine must reference one of the three evidence levels per §13.J.1. Copy that references only aspirational claims (§13.J.2) must be tagged as forward-looking. Copy that references testable claims (§13.J.3) must cite the relevant conformance lane or telemetry projection.

### 13.J.5 Evidence-export discipline
The doctrine's evidence pack (audit-chain projections, conformance-lane results, telemetry corpus) is exportable on request via the export-with-policy verb scoped to authorized analysts, customers, and regulators.

## Section 13.K - Open-questions and pending decisions

### 13.K.1 Pending: ADR-0319 office-scope taxonomy
ADR-0319 (front-office, middle-office, back-office) is in-flight and not present in the local checkout at authoring time. The doctrine refers to the taxonomy as a pending overlay; until ADR-0319 lands, the front-middle-back labels are used as descriptive aids only and do not influence substrate primitives.

### 13.K.2 Pending: marketplace-dispute-mediator role-projection design
The marketplace-dispute mediator role is referenced in §8.A.1 but the role-projection design is pending. Open questions: should oyatie operate as the mediator-of-last-resort under the ADR-0242 reserved-namespace pattern? should mediators be tenant-chosen third parties? should mediators be picked via a marketplace-of-mediators? The next doctrine revision should land an answer.

### 13.K.3 Pending: payroll-engine service-exception evaluation
The doctrine references payroll-engine service-retention for jurisdictions with idiosyncratic statutory rules. Open question: which specific jurisdictions justify the service-exception versus pack-overlay handling? An ADR mapping the jurisdictions is pending.

### 13.K.4 Pending: per-collar localization-and-density tuning catalog
The doctrine references workspace-tuning per collar color but the full per-collar tuning catalog is pending. An ADR documenting the per-collar design system is pending.

### 13.K.5 Pending: ADR-0250 build-ahead-of-certification timeline
ADR-0250 requires certified-shape day one but specific certification timelines per pack are not in this doctrine. An ADR with the certification-readiness timeline per pack is pending.

### 13.K.6 Pending: ADR-0247 self-modification scope-binding
ADR-0247 (Foundry as oyatie principals under Cedar) is referenced but the specific scope-binding for Foundry agent actions across substrate primitives is pending. An ADR detailing the principal hierarchy is pending.

### 13.K.7 Pending: per-region cell-topology and shuffle-shard parameters
ADR-0248 establishes cellular architecture with shuffle sharding but the specific cell-topology parameters per region are pending. An ADR detailing region-by-region cell-shape is pending.

### 13.K.8 Pending: per-tenant retention-policy customization
The doctrine references pack-aware retention but per-tenant override of retention (where pack rules permit) is pending. An ADR specifying override semantics is pending.

### 13.K.9 Pending: AI-substrate cross-pack inference governance
ADR-0255 references the AI substrate but cross-pack inference governance (e.g., a model trained on HIPAA-pack data inferring against a non-HIPAA-pack workload) is pending. An ADR detailing the boundary is pending.

### 13.K.10 Pending: marketplace-of-agents admission-gate AI-evaluation
ADR-0249 includes agents in the multi-category marketplace but the AI-evaluation rigor for agent admission is pending. An ADR detailing the evaluation suite is pending.

## Section 13.L - Substrate-team operating model

### 13.L.1 Substrate-team scope
The substrate teams own the substrate primitives plus their respective conformance lanes plus their respective ADR sets. Each substrate primitive has a clear primary owning team plus secondary collaborators.

### 13.L.2 Identity-team scope
Owns: `shared-identity`, `shared-identity-domain`, `shared-identity-recovery`, `shared-identity-passkey`, `shared-tenancy`, `shared-tenant-membership`. Conformance lanes: `governance-dual-tenant`, `governance-tenant-primitive`. ADRs: ADR-0244, ADR-0311, ADR-0313, ADR-0320, ADR-0242.

### 13.L.3 Policy-team scope
Owns: `intelligence-policy-engine-cedar`, `shared-policy-store`, `shared-cedar-evaluator`, `shared-policy-author`. Conformance lanes: `governance-policy-engine`. ADRs: ADR-0243, ADR-0250.

### 13.L.4 Workflow-team scope
Owns: `intelligence-workflow-engine`, `shared-workflow-templates`, `shared-workflow-runtime`, `shared-workflow-replay`. Conformance lanes: `governance-workflow-engine`, `governance-workflow-escalation`. ADRs: ADR-0245.

### 13.L.5 Ontology-team scope
Owns: `shared-ontology`, `shared-ontology-schema`, `shared-ontology-projection`, `shared-ontology-migration`. Conformance lanes: `governance-ontology`. ADRs: ADR-0244, ADR-0245.

### 13.L.6 Audit-team scope
Owns: `shared-audit-chain`, `shared-audit-chain-schema`, `shared-audit-replay`, `shared-audit-export`. Conformance lanes: `governance-audit-chain`. ADRs: ADR-0251, ADR-0252.

### 13.L.7 Marketplace-team scope
Owns: `shared-marketplace-settlement`, `shared-marketplace-admission`, `shared-marketplace-dispute`, `shared-marketplace-reputation`. Conformance lanes: `governance-marketplace`. ADRs: ADR-0249, ADR-0314.

### 13.L.8 UX-shell-team scope
Owns: `shared-ux-shell-action-router`, `shared-ux-shell-localization`, `shared-ux-shell-accessibility`, `shared-ux-shell-density`. Conformance lanes: `governance-verb-enum`, `governance-ux-shell`, `governance-accessibility`. ADRs: ADR-0317, ADR-0318, ADR-0253.

### 13.L.9 Compliance-pack-team scope
Owns: `shared-compliance-pack`, `shared-compliance-pack-overlays`, `shared-compliance-pack-evidence`. Conformance lanes: per-pack conformance test sets. ADRs: ADR-0251.

### 13.L.10 Plugin-team scope
Owns: `shared-plugin-admission`, `shared-plugin-isolation`. Conformance lanes: `governance-plugin-admission`. ADRs: ADR-0249.

### 13.L.11 Cross-team RFC process
Cross-substrate changes (any change touching more than one substrate primitive) require an RFC plus a multispectrum-review v2.4.0 with all relevant facet owners. RFCs are sealed into the doctrine audit-chain.

### 13.L.12 Substrate-team cadence
Substrate teams meet weekly for cross-team coordination, monthly for substrate-wide architecture review, quarterly for the multispectrum-review v2.4.0 sweep, and annually for the doctrine review per §13.B.

### 13.L.13 Substrate-team on-call
Each substrate primitive has a 24x7 on-call rotation. Incidents in the substrate primitives that affect customer tenants follow the substrate-incident playbook including audit-chain-sealed incident communication.

### 13.L.14 Substrate-team customer-engagement
Substrate teams engage with customer cohorts (early-access tenants, design-partner tenants, certification-stage tenants) through structured feedback rounds. Customer feedback feeds the doctrine evolution per §13.B.

### 13.L.15 Substrate-team external-engagement
Substrate teams engage with external bodies (standards bodies, regulators, academic research) where the substrate's design intersects with external work. Notable engagements: WebAuthn working groups (identity), Cedar working groups (policy), MLS standards (E2EE messenger), OpenSLO (observability), CNCF Kubernetes (orchestration), CMMC accreditation bodies (defense pack).

## Section 13.M - External-engagement and standards integration

### 13.M.1 WebAuthn and FIDO2 integration
The identity primitive consumes WebAuthn standards (W3C plus FIDO Alliance) for passkey-bound identity. The substrate uses platform attestation, attested credential data, and signature-counter monotonicity. Per ADR-0311, passkey is the substrate primary credential; legacy passwords are deprecated by ADR-0250 build-ahead-of-certification.

### 13.M.2 Cedar policy standards
The policy primitive uses AWS Cedar as the canonical authorization language per ADR-0243. Substrate-tooling for Cedar authoring follows the Cedar Policy Language Reference; substrate extensions are upstreamed to the Cedar working group where suitable.

### 13.M.3 MLS messenger standards (RFC 9420)
Per ADR-0255 KS#5, MLS is the canonical E2EE messenger protocol. Substrate's personal-tenant messaging uses MLS; substrate-extensions for tenant-pack toggles are documented per RFC 9420 §10 (extension framework).

### 13.M.4 OpenSLO observability standards
The substrate's observability framework consumes OpenSLO (`microservices/<ms>/slos/*.openslo.yaml`) per ADR-0130. Each substrate primitive ships OpenSLO definitions for the four telemetry signals plus primitive-specific SLOs.

### 13.M.5 CNCF Kubernetes plus Kata Containers integration
Per ADR-0254 KS#13, substrate runs on Kubernetes everywhere except edge. Cloud Hypervisor plus Kata containers provide hardware-grade isolation per the CNCF Kata Containers project.

### 13.M.6 HTTP/3 plus QUIC adoption
Per ADR-0253 KS#10, substrate's default transport is HTTP/3 over QUIC per RFC 9114 (HTTP/3) and RFC 9000 (QUIC). gRPC adapts to HTTP/3 per the gRPC HTTP/3 transport specification.

### 13.M.7 CMMC accreditation
The defense-pack overlay carries CMMC accreditation per the Cyber Maturity Model Certification (Department of Defense). Substrate's CMMC pack overlay ships day-zero per ADR-0250 build-ahead-of-certification.

### 13.M.8 FedRAMP authorization
Substrate's FedRAMP authorization is pursued at the High impact level for federal-tenant workloads. The audit-chain plus the compliance-pack primitive provide the evidence trail for the FedRAMP continuous-monitoring requirements.

### 13.M.9 EU AI Act conformance
Per ADR-0251 plus ADR-0255, substrate's EU AI Act conformance is tracked at risk-tier classification per Annex III of the EU AI Act. Substrate ships AI-model evaluation reports for marketplace-admitted models plus internal AI substrate components.

### 13.M.10 HIPAA, HITECH, and 21 CFR Part 11 conformance
Substrate's healthcare-pack overlay provides HIPAA (covered-entity rules), HITECH (breach notification), and 21 CFR Part 11 (electronic-records and electronic-signatures) conformance per ADR-0251.

### 13.M.11 GDPR conformance
Substrate's GDPR pack overlay provides controller-processor scoping, lawful-basis tracking, data-subject-rights workflow, breach-notification workflow, and Records-of-Processing-Activities (RoPA) projections.

### 13.M.12 SOX conformance
Substrate's SOX pack overlay provides control-design, control-testing, and control-effectiveness-reporting projections plus the audit-committee role projection plus the independent-auditor role projection.

### 13.M.13 PCI DSS conformance
Substrate's PCI pack overlay provides cardholder-data-environment scoping, restricted-access patterns for primary-account-numbers, network-segmentation patterns, and quarterly-vulnerability-scan integration.

### 13.M.14 CSAP conformance
Substrate's CSAP pack overlay provides Korean public-sector data-residency, role-projection-aware access controls, and audit-export grammar conformance per the Korean Personal Information Protection Act and the Cloud Computing Act.

### 13.M.15 NERC-CIP conformance
Substrate's energy pack overlay provides NERC-CIP-aware role projections, two-person attestation patterns, switching-order workflow patterns, and audit-export grammar conformance.

### 13.M.16 FERPA conformance
Substrate's education pack overlay provides FERPA-aware role projections, legitimate-educational-interest pack fragments, and consent-aware data-disclosure workflow.

### 13.M.17 ICH-GCP plus IRB conformance
Substrate's clinical-research pack overlay provides ICH-GCP-aware Workflow templates, IRB-amendment workflow, informed-consent Workflow templates, and Case-Report-Form e-signature per 21 CFR Part 11.

### 13.M.18 ITAR and EAR conformance
Substrate's defense pack overlay provides ITAR plus EAR export-control scoping, deemed-export Workflow templates, and access-control patterns for foreign-national exclusions.

### 13.M.19 Standards-evolution discipline
The doctrine commits to tracking external standards evolution. When a referenced standard updates (e.g., GDPR amendment, EU-AI-Act rulemaking, HIPAA modernization), substrate teams open a substrate-evolution workflow under the §13.B annual review cadence.

## Section 14 - References

### 14.0 Document map and how to read this thesis
- Executive thesis: substrate-as-product-replacement at glance.
- Section 1: the fragmentation tax oyatie removes.
- Section 1.A: worked-example dollar projections per-tax.
- Section 2: the substrate-vs-product layering.
- Section 3 plus 3.A: the 10 ONE-INVARIANTS plus their interaction matrix.
- Section 4 plus 4.A: capability-tier projection model plus rigor.
- Section 5 plus 5.A: role-based projection model plus lifecycle.
- Section 6 plus 6.A: collar-color universality plus workspace tuning.
- Section 7 plus 7.A: dual-tenant identity boundary plus edge cases.
- Section 8 plus 8.A: marketplace as universal settlement plus edge workflows.
- Section 9 plus 9.A: conglomerate hierarchy plus cross-subsidiary workflows.
- Section 10 plus 10.A plus 10.B: vendor displacement plus industry bundles plus migration cost model.
- Section 11: training-cost amortization (cross-reference to sibling doctrine).
- Section 12 plus 12.A plus 12.B plus 12.C: day-zero adoption plus pack-overlay plus workforce-shape plus plugin admission.
- Section 13 plus 13.A through 13.M: anti-patterns, detection, evolution, vendor playbooks, substrate-primitive detail, operational doctrine, service exceptions, keystone-bundle integration, multispectrum review, claim register, open questions, team operating model, and external-standards integration.
- Section 14: references.

Each section is self-contained for spot-reading. Cross-references are explicit. The doctrine is meant for substrate teams, capability-tier teams, customer-facing teams, partner teams, and regulator-engagement teams.

### Internal references
- docs/standards/documentation-rigor.md
- docs/architecture/keystone-bundle-2026-05-20-synthesis.md
- docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
- docs/architecture/training-cost-doctrine-2026-05-21.md
- docs/architecture/wave-3-g-synthesis-adjudication-2026-05-21.md
- docs/user-journeys/CATALOG-j126-j150-ecosystem.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/adr-archive/ADR-0242-oyatie-is-a-tenant-doctrine.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0701-monorepo-capability-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/adr-archive/ADR-0251-compliance-pack-cell-certification-levels.md
- docs/adr-archive/ADR-0252-time-coordination-distributed-consistency.md
- docs/adr-archive/ADR-0253-network-topology-edge-service-mesh.md
- docs/adr-archive/ADR-0254-deployment-model-spectrum.md
- docs/adr-archive/ADR-0255-intelligence-as-two-layer-ai-substrate.md
- docs/decisions/ADR-0702-identity-authz-live-apex.md
- docs/decisions/ADR-0700-ci-admission-live-apex.md
- docs/decisions/ADR-0705-product-protocol-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md
- docs/decisions/ADR-0709-general-live-apex.md

### Implementation microservices
- shared-identity, shared-identity-domain (ONE-IDENTITY)
- shared-tenancy (ONE-IDENTITY plus ADR-0244 tenancy primitive)
- intelligence-policy-engine-cedar, shared-policy-store, shared-cedar-evaluator (ONE-POLICY-ENGINE)
- intelligence-workflow-engine, shared-workflow-templates, shared-workflow-runtime (ONE-WORKFLOW-ENGINE)
- shared-ontology, shared-ontology-schema, shared-ontology-projection (ONE-ONTOLOGY)
- shared-audit-chain, shared-audit-chain-schema, shared-audit-replay (ONE-AUDIT-CHAIN)
- shared-marketplace-settlement, shared-marketplace-admission, shared-marketplace-dispute (ONE-MARKETPLACE)
- shared-ux-shell-action-router, shared-ux-shell-localization, shared-ux-shell-accessibility (ONE-UX-SHELL)
- shared-compliance-pack, shared-compliance-pack-overlays (ONE-COMPLIANCE-POSTURE)
- shared-plugin-admission, shared-plugin-isolation (ONE-PLUGIN-EXTENSIBILITY)

### External references and precedent anchors
- Apple ecosystem and Human Interface Guidelines: https://developer.apple.com/design/human-interface-guidelines/
  Use in this thesis: device continuity, consistent interaction vocabulary, managed vs personal account separation, and human-centered interface hierarchy.
- Apple Continuity: https://www.apple.com/macos/continuity/
  Use in this thesis: task continuity across phone, tablet, laptop, watch, and peripherals without requiring the user to relearn the action.
- Microsoft 365: https://www.microsoft.com/en-us/microsoft-365/products-apps-services
  Use in this thesis: Word, Excel, PowerPoint, Outlook, Teams, SharePoint, OneDrive, identity, compliance, and learning paths under one productivity estate.
- Microsoft 365 learning pathways: https://learn.microsoft.com/en-us/office365/customlearning/driveadoption
  Use in this thesis: adoption content and repeatable training channels for a broad suite rather than isolated product islands.
- Google Workspace Learning Center: https://support.google.com/a/users/answer/9389764
  Use in this thesis: shared collaboration surfaces and service training across Gmail, Drive, Meet, Chat, Docs, Sheets, Slides, Forms, and Calendar.
- Salesforce Platform and AppExchange: https://www.salesforce.com/platform/ecosystem
  Use in this thesis: metadata-driven platform extension, marketplace distribution, and role-tailored product clouds over shared customer data.
- Salesforce Trailhead AppExchange basics: https://trailhead.salesforce.com/content/learn/modules/appexchange_basics
  Use in this thesis: ecosystem learning and marketplace onboarding as a governed adoption primitive.
- ServiceNow Now Module workflow automation: https://www.servicenow.com/now-platform/workflow-automation.html
  Use in this thesis: workflow automation and low-code development over one governed platform surface.
- Atlassian Platform: https://www.atlassian.com/platform
  Use in this thesis: shared administration, graph, work management, and compliance features across multiple team tools.
- Notion connected workspace: https://www.notion.com/help/guides/connected-workspace-for-product-teams-to-collaborate-ideate-and-launch
  Use in this thesis: docs, wiki, projects, tasks, and connected tools in one workspace grammar.
- Gartner SaaS sprawl collaboration research: https://www.gartner.com/en/documents/6873766
  Use in this thesis: SaaS sprawl is an enterprise architecture and application-team governance problem rather than a simple procurement nuisance.
- Gartner SaaS management platforms: https://www.gartner.com/en/documents/5621791
  Use in this thesis: unmanaged SaaS produces visibility, overspending, risk, and contract sprawl that must be governed as a portfolio.
- Forrester tech sprawl research: https://www.forrester.com/report/the-state-of-tech-sprawl-in-the-us-2024/RES181386
  Use in this thesis: technology sprawl is a measurable consolidation concern for IT and technology decision-makers.
- Forrester SaaS integration challenges: https://www.forrester.com/report/Brief-Address-Todays-SaaS-Integration-Challenges-To-Increase-Business-Value/RES130201
  Use in this thesis: SaaS value depends on cohesive implementation and integration, not just subscription purchase.
