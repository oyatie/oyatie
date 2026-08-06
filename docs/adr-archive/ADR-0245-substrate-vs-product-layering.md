---
id: ADR-0245
status: Superseded
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-engineering
  - ops-sre-reliability
  - ops-compliance
  - axis-foundry
  - axis-workflow-engine
  - axis-ontology
  - axis-policy-engine
  - axis-identity
  - axis-tenancy
  - axis-audit-chain
supersedes: []
amends:
  - ADR-0131-per-microservice-flat-layout.md (extends layout-only authority with a tier-classification field; additionally: reserved-tier µservices are EXEMPT from the ADR-0131 requirement for `src/` and `iac/` directories — reserved µservices ship PRD.md + threat-model.md + dpia.md + manifest.json skeletons only; `planned_contracts/` is the canonical directory for planned-but-not-yet-live OpenAPI/AsyncAPI contracts on reserved-tier µservices)
  - ADR-0132-product-platform-and-bundle-dissolution.md (adds tier classification to the flat-microservice forward-policy)
  - ADR-0145-inter-microservice-communication-reform.md (tightens the three invariants with cross-tier direction rules)
superseded_by: [ADR-701]
amended_by: [ADR-0280, ADR-0635]
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0056-bnf-v4-1-naming.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0110-changeset-state-machine.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-product-platform-and-bundle-dissolution.md
  - ADR-0135-connect-super-app-expansion.md
  - ADR-0136-intelligence-as-single-microservice.md
  - ADR-0139-agentic-slo-gated-promotion.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-sustainability-tag.md
  - ADR-0176-brown-out-degradation-signal.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0213-ecosystem-as-a-service-architecture.md
  - ADR-0218-tenant-granular-control-surface.md
  - ADR-0220-consumer-intelligence-substrate.md
  - ADR-0239-amendment-intelligence-internal-scope-clarification-2026-05-18.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/manifest-schema.json
  - /specs/per-microservice-flat-layout.json
  - /specs/microservice-tier-classification.json
  - /specs/microservice-dependency-dag.json
  - /specs/substrate-slo-bar.json
related_memory:
  - feedback_substrate_vs_product_layering
  - feedback_quality_performance_scalability_bar
  - feedback_flat_product_catalog
  - feedback_workflow_studio_scope
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_autonomous_implementation_artifacts
  - feedback_glossary_shared_not_platform
  - feedback_oyatie_is_a_tenant_doctrine
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 4-of-14
purpose: >
  Establish the canonical two-rule doctrine that every µservice declares a
  tier: substrate (audience-neutral capability-focused), product (tenant-
  scoped surface-focused), service-cell (dedicated-function peer cell), or
  reserved (placeholder for a future certification-gated capability). The
  tier is a manifest field, CI-enforced, and governs SLO bar, versioning
  policy, sunset policy, dependency direction, deployment cadence, and
  observability defaults. Substrates serve all tenants and the platform;
  products are tenant-scoped surfaces; service cells are dedicated-
  function peer cells; reserved µservices declare intent without
  shipping live workloads. This ADR replaces the ad-hoc substrate-vs-
  product distinction that has accumulated across the portfolio since
  ADR-0131 with a uniform, declarative, CI-enforced rule.
enforcement_status: advisory-until-tier-field-lands-and-classified
enforced_by:
  - oya gate validate substrate-vs-product-tier-coherence
  - oya gate validate cross-tier-dependency-direction
  - oya gate validate substrate-slo-bar
  - oya gate validate reserved-microservice-skeleton-completeness
  - oya gate validate tier-versioning-policy
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0245: Substrate vs Product Layering

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive). Lands as a single multispectrum-reviewed PR.
Partial acceptance is rejected because the doctrines are mutually-
reinforcing and produced together to avoid the drift pattern that
produced the ADR-0220 → ADR-0239 amendment cycle within twelve days.

Enforcement is `advisory-until-tier-field-lands-and-classified`. The
doctrine is accepted in text on 2026-05-20; the CI lanes that enforce
it move to BLOCKER status only after:

1. Every existing µservice's `manifest.json` declares `tier:` and
   `tier_subtype:` per §D-2 and §D-3.
2. `microservices/policy-engine/` is promoted to peer substrate µservice
   (per ADR-0246) so that cross-tier permission fragments can be
   authored.
3. The dependency DAG declared in §D-4 has been verified end-to-end via
   `oya gate validate cross-tier-dependency-direction` returning exit
   0 on a clean build.
4. The substrate-SLO-bar lane (§D-8) is wired into the per-µservice
   SLO authoring at `microservices/<ms>/slos/*.openslo.yaml` and
   reports 100% coverage for tier-substrate µservices.

Until those four prerequisites land, validators emit findings without
failing CI. Post-prerequisite, the lanes promote to BLOCKER per
ADR-0139 `agentic-SLO-gated-promotion`.

## Date

2026-05-20.

## Context

### Prior portfolio state (pre-keystone)

The oyatie portfolio has used three implicit, overlapping, and
sometimes-contradictory framings for the substrate-vs-product
distinction:

1. **ADR-0131 (per-microservice flat layout, 2026-05-17).** Mandates
   one universal artifact layout under `microservices/<ms>/`. Section
   "Canonical folder shape" describes the layout as **identical** for
   substrates and products — "the product-vs-substrate distinction
   collapses at the directory level; both shapes use the same folder
   structure. Sales segmentation remains a PRD-frontmatter field, not
   a directory split." This was the right call for layout but it
   inadvertently *erased* the operational distinction.
2. **ADR-0132 (no-grouping forward-policy, 2026-05-17).** Forbids new
   platform/bundle µservices. Says "Cross-µservice composition flows
   through Workflow events and Ontology reads/writes." Says
   "Categorization (e.g., `DomainTag = {Agentic, Dev, Business, ...}`
   inside Workflow Studio) stays as metadata, NOT as a directory split."
   But it does not establish *what* the metadata is or how it gates
   behaviour.
3. **ADR-0145 (inter-µservice communication reform, 2026-05-18).**
   Establishes the three weak invariants (audit, tracing, ontology
   projection) and permits direct gRPC. Hints at a substrate/product
   distinction in its references to Ontology being a "SUBSTRATE, not a
   GATEWAY" but does not codify a tier field.

In addition, several µservice-specific ADRs reference the
substrate-vs-product distinction without a shared definition:

- **ADR-0136 (Foundry as single µservice, 2026-05-18) +
  ADR-0136-amendment + ADR-0239.** Tags Foundry as "internal
  µservice serving the retired external agent harness agentic development pipeline" — an
  audience-shaped framing rather than a tier-shaped framing.
- **ADR-0220 (Consumer Intelligence Substrate, 2026-05-18).** Names
  Intelligence a "substrate" but in a consumer-facing audience sense,
  conflating substrate (the tier) with substrate (the rhetorical
  weight-bearing role).
- **ADR-0183 (Cedar app authz + Kyverno admission, 2026).** Implicitly
  treats policy-engine as a substrate concern.
- **ADR-0028 (Cloud microservice architecture, 2026).** Defines per-
  µservice contracts but does not classify tier.

The accumulated state across these ADRs has produced 9 distinct
substrate-vs-product framings, none of which compose cleanly with the
others. The PR-#143 close-out audit
(`evidence/pr-143-close-out-plan-and-gap-audit-2026-05-18.json`)
surfaced this drift: reviewers asked "is this µservice a substrate or
a product?" 14 times across 9 µservices in a single review cycle.

### The cost of unclassified tier

Operating with an unclassified or implicit tier produces recurring
tax:

1. **SLO bar drift.** Substrates (which downstream products depend on)
   require a stricter SLO bar than products (which serve end-users
   directly). Without a tier field, every µservice authors its own
   SLO bar; some are too lax (substrate µservices with 99.5%
   targets), some are too strict (product µservices forced into
   99.99% targets they don't need). Per Google SRE Workbook ch. 2
   (Beyer et al. 2018), this is a classic SLO-pyramid violation.
2. **Versioning policy drift.** Substrates need long sunset windows
   (12+ months) because many downstream products depend on them.
   Products can sunset on a per-product cadence (often 90 days for
   consumer-facing surfaces; per-quarter for enterprise). Without a
   tier field, sunset windows are negotiated case-by-case, with
   inconsistent outcomes.
3. **Dependency direction violations.** A substrate that depends on a
   product is an architectural inversion (the foundation depends on
   the building floor). Without a CI-enforced tier-direction rule,
   these inversions silently emerge during refactor.
4. **Observability defaults drift.** Substrates carry per-tenant
   telemetry rollups; products carry per-user telemetry. Without a
   tier field, observability dashboards are bespoke per-µservice.
5. **Compliance pack applicability ambiguous.** Compliance Packs (per
   ADR-0251) primarily apply to products that touch tenant data, with
   substrate packs covering audit + policy + identity. Without a
   tier field, pack applicability is inferred per-µservice case-by-
   case.
6. **Deployment cadence drift.** Substrates deploy slowly (per
   ADR-0139 SLO gates + extensive canary). Products deploy fast (per-
   product cadence; some daily). Without a tier field, cadence is
   set ad-hoc.
7. **Marketplace surface ambiguity.** Products appear on a marketplace
   surface (Plugin App Store, Marketplace, App Catalog). Substrates do
   not. Without a tier field, the marketplace ingestion pipeline must
   exclude substrates by name (drift-prone).
8. **Audience framing drift.** ADR-0242 retired the manifest `audience`
   field in favour of tenant-scoped audience. But without a tier
   field, contributors continue to refer to µservices as "consumer-
   facing" or "internal-facing" — categories that the audience-retired
   doctrine no longer supports.
9. **Capacity planning bias.** Substrates plan capacity against the
   sum of downstream products' demand; products plan against direct
   end-user demand. Without a tier field, capacity models are
   per-µservice bespoke.

### What "Substrate vs Product Layering" formalizes

The doctrine establishes a manifest-declared tier field with three
production tiers (`substrate`, `product`, `service-cell`) plus one
non-production tier (`reserved`), each with crisp semantics, a
classification table covering every existing and reserved µservice, a
dependency-direction rule, a per-tier SLO bar, a per-tier versioning
policy, and CI lanes that enforce all of the above.

The doctrine matches the universally-recognised hyperscaler shape of
distinguishing platform capabilities from end-user surfaces while
permitting platform-of-record self-hosting (per ADR-0247) and
audience-neutral substrate operation (per ADR-0242).

### Why a tier field is the right level of abstraction

A tier field is:

- **Declarative.** The µservice manifest declares its own tier; the
  ADR-0131 layout doesn't change.
- **CI-enforceable.** Static analysis can verify cross-tier dependency
  direction without runtime cost.
- **Composable with the audience-retired tenant model.** Tier is a
  property of the µservice; audience is a property of the tenant. The
  two are orthogonal.
- **Aligned with hyperscaler reference shape** (see §"Hyperscaler
  precedent" below).
- **Forward-compatible with future tiers.** If a fifth tier emerges
  (e.g., `compliance-overlay`), it can be added to the enum without
  reshaping the rule.

### Hyperscaler precedent

The substrate-vs-product distinction (or analogous distinctions) is a
universal pattern at hyperscale platforms. Five named references:

| Company | Pattern | Source |
|---|---|---|
| **AWS** | Foundational services (IAM, S3, EC2, KMS) ≠ Application services (Verified Permissions, MGN, IoT) ≠ Industry products (HealthLake, FinSpace) ≠ Reserved partitions (`aws-us-gov`). The Well-Architected Framework distinguishes "platform services" from "applications" with different SLO + versioning policies. | AWS Well-Architected Framework v2024-Q4; AWS Builders' Library "Building dashboards for operational visibility" 2023; AWS re:Invent 2024 "Foundations of resilience" Werner Vogels keynote. |
| **Apple** | Framework (Core Foundation, Foundation, UIKit, SwiftUI, CloudKit) ≠ App (Messages, Mail, Calendar, Notes, FaceTime, Photos). Frameworks have multi-year API-stability commitments (binary + source); apps ship on a per-OS cycle. Apple's Human Interface Guidelines + the App Store Review Guidelines reinforce the distinction. | Apple Platform Architecture documentation 2024; Apple Frameworks Reference; "App Store Review Guidelines" v2024-Q3; WWDC 2024 "What's new in CloudKit" session 10010. |
| **Google Cloud Platform** | Foundational services (Borg → Kubernetes Engine, Spanner, BigQuery, Pub/Sub, IAM) ≠ Industry products (Healthcare API, Anti Money Laundering AI, Cloud Retail) ≠ Apigee API gateway ≠ Looker analytics. Each tier has a per-tier deprecation policy; foundational services have 12+ month deprecation notice. | Google Cloud Deprecation Policy 2024; Google SRE Workbook ch. 5; CNCF KubeCon NA 2024 "GKE 10-year retrospective" by Tim Hockin. |
| **Salesforce** | Platform (Lightning Platform, Force.com, Apex, Heroku, MuleSoft) ≠ Clouds (Sales Cloud, Service Cloud, Marketing Cloud, Health Cloud, Financial Services Cloud). Platform has Trust + Compliance compliance baseline; Clouds inherit + extend. | Salesforce Trust Documentation 2024; Trailhead Multi-tenant Architecture module 2024; Salesforce Architecture Decision Records (publicly published 2024-Q4). |
| **Microsoft Azure** | Foundational services (Entra ID, KeyVault, Azure Storage, Azure SQL, Azure Functions) ≠ Industry services (Azure for Healthcare, Azure for Manufacturing, Azure for Financial Services) ≠ Apps (Microsoft 365, Dynamics 365, Power Platform). Each tier has its own SLA + versioning + deprecation policy. | Microsoft Cloud Adoption Framework 2024; Azure Well-Architected Framework 2024; Microsoft Build 2024 keynote "Foundations of Azure" by Scott Guthrie. |

The lesson across all five is uniform: **at hyperscale, foundational
capabilities (substrates) and end-user surfaces (products) operate
under different SLO bars, deployment cadences, versioning policies,
and observability defaults; the distinction is manifest-declared (not
inferred), CI-enforced (not honour-system), and stable across the
platform's lifecycle.**

oyatie's tier field is the manifest-declared form of this distinction.

### Service cells as a third tier

In addition to substrates and products, three named hyperscalers
operate a third tier — what AWS internally calls "service cells" and
what GCP calls "control-plane peer services":

- **AWS marketplace, AWS Activate, AWS Marketplace Partner Central,
  AWS IAM Access Analyzer** — each is a peer service to mainline
  AWS, with dedicated functions, that is neither a substrate (does
  not host another service's workloads) nor a product (does not have
  end-user surfaces directly; AWS Activate has a console but acts as
  a peer of other services).
- **Salesforce AppExchange + Trailhead + Trust.salesforce.com** —
  similar peer-cell pattern.
- **Stripe Connect, Stripe Atlas, Stripe Climate** — peer-cell
  pattern.

oyatie adopts this third tier explicitly: `service-cell` µservices
host dedicated peer functions (marketplace, dev-tools, audit-
aggregator, analytics rollup) that are conceptually neither substrate
(no foundational tenants depend on them) nor product (no end-user
surface as primary purpose).

### Reserved as a fourth (non-production) tier

The fourth tier — `reserved` — exists to enable the build-ahead-of-
certification doctrine (ADR-0250). A reserved µservice has its full
artifact set (PRD, contracts, threat model, IaC) but is **not
deployed** because it requires certification (PCI-DSS, ISO 18295,
HIPAA, ISO 22301, etc.) that has not yet been obtained.

Reserved µservices serve four purposes:

1. **Forward-declared architecture intent.** Future capability is
   visible in the manifest from day 1.
2. **Build-time validation.** The µservice's contracts, schema, and
   skeleton code build alongside the rest of the platform; integration
   surfaces evolve consistently.
3. **Certification-gate clarity.** What needs to be certified is
   explicit (manifest declares the certification gate).
4. **Audit trail for regulators.** Auditors can verify that
   uncertified capabilities are not running by inspecting the
   `tier: reserved` field.

### Why now (2026-05-20)

Three forcing functions:

- **The portfolio's ~46 µservices** (44 already created at
  `microservices/`; 2-7 reserved µservices to be authored as part of
  this keystone bundle) have reached the scale where ad-hoc per-
  µservice classification is intractable. Per Conway's Law and per
  the Bezos two-pizza-team rule (which AWS scaled with explicit per-
  service ownership boundaries), explicit classification at this
  scale prevents next-quarter drift.
- **ADR-0242 retired the `audience` field.** Without `audience`, the
  question "is this µservice consumer-facing or internal-facing"
  has no manifest answer. The tier field is the structural
  replacement — it answers "is this a substrate (audience-neutral)
  or a product (tenant-scoped surface)" with the same CI-enforced
  rigor.
- **The autonomous-masterplan goal
  (feedback_autonomous_implementation_artifacts).** Autonomous agents
  scaffolding new µservices need an unambiguous classifier; without
  a tier enum, scaffolds drift toward one or the other shape, which
  then accumulates as portfolio entropy.

## Decision

### D-1. Two-rule doctrine

The doctrine is two rules that compose:

**Rule 1 — Substrates are audience-neutral and capability-focused.**
A substrate µservice provides a capability (storage, policy
evaluation, identity issuance, cell management, observability rollup,
compute scheduling, network routing, secrets management, audit
emission, ontology projection, intelligence inference, workflow
orchestration, governance enforcement, marketplace data) that is
consumed by other µservices and by tenant workloads. A substrate has
**no end-user surface as its primary purpose**. Its end-users are
*other µservices and other principals*.

**Rule 2 — Products are tenant-scoped and surface-focused.** A product
µservice provides an end-user-facing surface (mail UI, drive UI,
calendar UI, meet UI, workflow studio canvas, tenancy admin console,
finops portal, plugin app store, marketplace, ops dashboard, social
network) that is consumed by *humans within a tenant*. A product's
primary purpose is the surface. Its secondary purpose may include API
exposure for tenant integration.

**NEVER MIX.** A single µservice MUST NOT be both. Mixing produces:

- SLO drift (substrate bar is too strict for product UX, product bar
  is too lax for substrate dependencies).
- Versioning drift (substrate needs 12+ month sunset, product needs
  90-day cadence).
- Observability default drift (per-tenant rollup vs per-user trace).
- Deployment cadence drift (slow vs fast cadence).
- Compliance pack applicability drift (substrate packs vs product
  packs).

If a µservice candidate appears to be both, it must be split into a
substrate-tier µservice + a product-tier µservice with the product
calling the substrate (per the dependency-direction rule in §D-4).
This is the Conway split applied at the µservice boundary.

### D-2. Manifest `tier` field + `tier_subtype` field

Every µservice's `microservices/<ms>/manifest.json` (and its
companion `microservices/<ms>/PRD.md` frontmatter) MUST declare:

```yaml
tier: substrate | product | service-cell | reserved
tier_subtype: <enum value; see below>
tier_classification_rationale: |
  <one-paragraph human justification referencing this ADR §D-3 row>
tier_certified_at: <ISO-8601 date of multispectrum-review verdict
                    that approved this classification>
tier_promotion_history:
  - from: <prior tier or null>
    to: <current tier>
    via: <ADR number(s)>
    at: <ISO-8601 date>
```

The `tier_subtype` enum is closed; new values require ADR amendment.
The current set:

**substrate-* subtypes (8 values):**

- `substrate-identity` — identity issuance, OIDC, service principals
  (Zitadel-class capability).
- `substrate-tenancy` — tenant registration, sub-scope hierarchy,
  reserved-namespace enforcement.
- `substrate-policy` — Cedar policy evaluation (per ADR-0150 +
  ADR-0246 promotion).
- `substrate-audit` — audit-chain Merkle-sealed emission + retention.
- `substrate-data` — Ontology, object types, data-class registry,
  per-tenant data plane primitives.
- `substrate-compute` — Kubernetes / Wasmtime / Function compute
  scheduling, capacity, autoscaling.
- `substrate-network` — DNS, service mesh, edge proxy, NetworkPolicy
  authoring.
- `substrate-secrets` — OpenBao / KMS / certificate authority,
  Shamir-shared root key, encryption-BYOK substrate.
- `substrate-infra` — cell management, regional pack registration,
  bootstrap-cell lifecycle.
- `substrate-observability` — Mimir / Loki / Tempo / dashboards rollup,
  per-tenant + per-sub-scope metrics.
- `substrate-iac` — Helm / Terraform / Kustomize module registry,
  IaC validation.
- `substrate-ai` — Intelligence substrate (AI inference, embeddings,
  RAG, agentic toolchains; per ADR-0255 rewrite).
- `substrate-orchestration` — Workflow Engine (Step-Functions-class
  durable orchestration; per ADR-0145 opt-in pattern).
- `substrate-marketplace-data` — Marketplace Catalog (canonical
  product catalog for plugins/apps/connectors; per ADR-0249).
- `substrate-governance` — Governance µservice (~50 oya-check-* lanes
  per ADR-0131 §IP-M01-MIGR-014); declares fitness gates.
- `substrate-api-gateway` — API gateway (Envoy / Cilium ingress;
  request routing; rate limiting).
- `substrate-comms` — Comms-email (transactional email substrate;
  not a product because it has no user-facing surface — it's the
  sending substrate behind every product's notifications).
- `substrate-consent` — Consent graph substrate (DSAR cascade + consent
  state authoring).
- `substrate-compliance` — Compliance µservice (per-pack fragment
  registry + per-pack admission gate; ADR-0251).

**product-* subtypes (3 high-level + per-product variants):**

- `product-consumer` — primary audience is consumer end-user (or
  prosumer); surface is a polished UX.
- `product-internal` — primary audience is platform-operator end-user
  (oyatie engineer or oyatie ops); surface is admin tooling.
- `product-developer` — primary audience is developer end-user
  (building on top of the platform); surface is dev tooling, SDKs,
  CLIs.

Per-product subtype refines further (see §D-3 table). Examples:
`product-consumer-mail`, `product-consumer-drive`,
`product-internal-tenancy-admin-console`,
`product-developer-sdk`, etc.

**service-cell-* subtypes (5 values):**

- `service-cell-marketplace` — marketplace surface (Plugin App Store,
  Marketplace product surface).
- `service-cell-dev-tools` — developer tooling (Developer SDK
  catalog, CLIs, code samples).
- `service-cell-audit-aggregator` — audit-chain aggregator (per-
  jurisdiction rollup, regulator query interface).
- `service-cell-analytics` — analytics rollup (per-tenant +
  per-cohort aggregation; the analytics product).
- `service-cell-ops-console` — operator console (control center,
  ops dashboards) — a peer surface to tenancy admin but for the
  oyatie tenant.

**reserved-* subtypes (per-reserved-µservice variants):**

- `reserved-financial-grade` — payments, settlement, payouts
  (PCI-DSS + KR-FSS + EU PSD2 + AML/KYC certification gates).
- `reserved-identity-verification-grade` — IDV (ISO 18295 + per-
  jurisdiction KYC certification gates).
- `reserved-tax-grade` — tax engine (per-jurisdiction tax authority
  registration; SST in US; HMRC; KR NTS).
- `reserved-deidentification-grade` — de-identification substrate
  (HIPAA Safe Harbor + EU GDPR pseudonymisation certification).
- `reserved-breach-notification-grade` — breach notification
  workflows (per-jurisdiction breach-notification authority
  registration; GDPR Article 33-34; KR-PIPA Article 39-4).
- `reserved-encryption-substrate-grade` — encryption substrate (FIPS
  140-3 + Common Criteria EAL4+; per-pack certification).
- `reserved-consent-grade` — consent recording (per-jurisdiction
  consent authority registration; EU GDPR Article 7; KR-PIPA
  Article 22).
- `reserved-fulfillment-grade` — fulfillment / logistics surfaces
  requiring carrier API certifications.

The `tier_classification_rationale` and `tier_certified_at` fields
ensure that the classification is auditable: when did this µservice
acquire its current tier, and via which ADR? Tier promotion (e.g.,
policy-engine BC → policy-engine substrate µservice per ADR-0246) is
captured in `tier_promotion_history`.

### D-3. Full µservice classification table

The table classifies every µservice currently under
`microservices/` (44 existing, 7 reserved to be added) into the
canonical tier + tier_subtype + brief justification.

#### D-3.A Substrate-tier µservices (existing)

| µservice path | tier | tier_subtype | justification |
|---|---|---|---|
| `microservices/intelligence/` | substrate | `substrate-ai` | AI inference substrate — embeddings, RAG, LLM tool-call, agentic workflows. Consumed by every product (mail summarization, drive search, workflow-studio AI nodes, consumer chat). Audience-neutral. Per ADR-0255 rewrite as 2-layer substrate (Substrate API + Consumer Brand Surface, which is itself a product). |
| `microservices/ontology/` | substrate | `substrate-data` | Object Type definitions + projections + cross-µservice entity reads. Per ADR-0145 invariant 3. The canonical data substrate for cross-µservice queryability. Audience-neutral; every product reads from it. |
| `microservices/policy-engine/` | substrate | `substrate-policy` | Cedar v4.2 evaluator per ADR-0150 + ADR-0246 promotion. Audience-neutral; gates every authorization decision, every routing decision, every retention decision. |
| `microservices/audit-chain/` | substrate | `substrate-audit` | Merkle-sealed audit emission. Per ADR-0028 inheritance. Audience-neutral; every state-changing call emits to it. |
| `microservices/identity/` | substrate | `substrate-identity` | OIDC service principal issuance, OAuth flows, WebAuthn / passkey support. Audience-neutral. Zitadel-class capability. |
| `microservices/tenancy/` | substrate | `substrate-tenancy` | Tenant row authoring, sub-scope hierarchy (per ADR-0242 + ADR-0244), reserved-namespace enforcement. Audience-neutral. |
| ADR-0333 cell successors | substrate | `substrate-infra` | Cell management split across tenancy assignment, cloud-iac provisioning, observability health, api-gateway routing, audit-chain scoping, and `crates/oya-shuffle-sharding` per ADR-0009 + ADR-0010 + ADR-0248. Audience-neutral; every workload runs in a cell. |
| `microservices/observability/` | substrate | `substrate-observability` | Mimir / Loki / Tempo rollup, per-tenant dashboards, per-sub-scope metrics. Audience-neutral. Required by every µservice per ADR-0139. |
| `microservices/cloud-secrets/` | substrate | `substrate-secrets` | OpenBao + KMS + Shamir-shared root key per ADR-0150 + ADR-0242 bootstrap step 2. Audience-neutral; every encryption decision touches it. |
| `microservices/cloud-iac/` | substrate | `substrate-iac` | Helm chart registry, Terraform module registry, Kustomize overlay catalog. Audience-neutral; every µservice's deployment references it. |
| `microservices/cloud-k8s/` | substrate | `substrate-compute` | Kubernetes control plane wrapper, per-cell scheduling, capacity-aware HPA. Audience-neutral. |
| `microservices/cloud-network/` | substrate | `substrate-network` | DNS, edge proxy authoring, NetworkPolicy registry, service mesh interface per ADR-0148. Audience-neutral. The former professional-network product path retired into `community` in Wave 15K. |
| `microservices/workflow-engine/` | substrate | `substrate-orchestration` | Step-Functions-class durable orchestration per ADR-0145. Audience-neutral; consumed by every product needing durable workflows. |
| `microservices/governance/` | substrate | `substrate-governance` | ~50 `oya-check-*` lanes per ADR-0131 §IP-M01-MIGR-014. Authors and enforces fitness gates. Audience-neutral. |
| `microservices/api-gateway/` | substrate | `substrate-api-gateway` | Envoy / Cilium ingress, per-call rate limiting, tenant routing. Audience-neutral. |
| `microservices/comms-email/` | substrate | `substrate-comms` | Transactional email substrate behind every product's email notifications. Has no end-user surface; it's the sending substrate. Audience-neutral. |
| `microservices/consent-graph/` | substrate | `substrate-consent` | Consent graph substrate authoring, DSAR cascade. Per ADR-0244 + ADR-0251. Audience-neutral; every consent decision touches it. |
| `microservices/compliance/` | substrate | `substrate-compliance` | Compliance Pack fragment registry, per-pack admission gate, per-pack overlay enforcement. Per ADR-0251. Audience-neutral. |
| `microservices/foundry/` | substrate | `substrate-meta` (reserved subtype; see note) | Per ADR-0136-amendment + ADR-0239 + ADR-0242-amendment, Foundry's "internal-only audience" framing dissolves. Its substrate role (CI orchestration, eval runs, multispectrum review fan-out, evidence emission) is preserved *operationally* but its identity as a µservice is being decomposed per ADR-0247 self-modification doctrine. **Tentative classification: `substrate-meta` (a substrate that authors and modifies the platform itself).** Final dissolution path is owned by ADR-0247 + the marketplace ADR. Until dissolution, this row remains as `substrate-meta`. |

#### D-3.B Product-tier µservices (existing)

The following are product-tier µservices. Each has an end-user surface
as primary purpose, is tenant-scoped, and consumes substrates from
§D-3.A.

| µservice path | tier | tier_subtype | justification |
|---|---|---|---|
| `microservices/workflow-studio/` | product | `product-consumer-workflow-studio` | Visual workflow editor — n8n-class consumer + prosumer product per `feedback_workflow_studio_scope`. End-user surface is the canvas + node library + run history. Tenant-scoped. Calls workflow-engine substrate. |
| `microservices/mail/` | product | `product-consumer-mail` | Mail UI; per ADR-0131 IP-M01-MIGR-CONN-1. Tenant-scoped. Calls comms-email substrate for sending; ontology for contacts; intelligence for summarization. |
| `microservices/drive/` | product | `product-consumer-drive` | File storage + management UI. Tenant-scoped. Calls ontology + intelligence + cloud-iac. |
| `microservices/calendar/` | product | `product-consumer-calendar` | Calendar UI. Tenant-scoped. Calls ontology + workflow-engine for reminders. |
| `microservices/meet/` | product | `product-consumer-meet` | Video conferencing UI. Tenant-scoped. Calls intelligence for live transcription. |
| `microservices/messenger/` | product | `product-consumer-messenger` | Chat UI. Tenant-scoped. Calls ontology + intelligence. |
| `microservices/docs/` | product | `product-consumer-docs` | Document editor UI. Tenant-scoped. Calls drive + ontology + intelligence. |
| `microservices/sheets/` | product | `product-consumer-sheets` | Spreadsheet editor UI. Tenant-scoped. Calls drive + ontology + intelligence. |
| `microservices/slides/` | product | `product-consumer-slides` | Presentation editor UI. Tenant-scoped. Calls drive + ontology + intelligence. |
| `microservices/notes/` | product | `product-consumer-notes` | Notes app UI. Tenant-scoped. Calls drive + ontology. |
| `microservices/recordings/` | product | `product-consumer-recordings` | Meeting recordings UI. Tenant-scoped. Calls meet (paired) + drive + intelligence. |
| `microservices/tasks/` | product | `product-consumer-tasks` | Task management UI. Tenant-scoped. Calls ontology + workflow-engine. |
| `microservices/forms/` | product | `product-consumer-forms` | Forms / surveys UI. Tenant-scoped. Calls ontology + drive. |
| `microservices/sites/` | product | `product-consumer-sites` | Web-publishing UI. Tenant-scoped. Calls drive + intelligence. |
| `microservices/translate/` | product | `product-consumer-translate` | Translation tool UI. Tenant-scoped. Calls intelligence. |
| `microservices/shorts/` | product | `product-consumer-shorts` | Short-form video UI (per ADR-0135 connect-super-app-expansion). Tenant-scoped. Calls intelligence + drive. |
| `microservices/social/` | product | `product-consumer-social` | Social network surface (per ADR-0135). Tenant-scoped. Calls ontology + intelligence. |
| `microservices/community/` | product | `product-consumer-community` | Community Q&A + KB threads (per ADR-0131 IP-M01-MIGR-CONN-4). Tenant-scoped. Calls ontology + intelligence. |
| `microservices/community/` (anonymity-mode) | product | `product-consumer-community` | Community's anonymity posting-mode capability tier (persona-anchored/pseudonymous/fully-anonymous per ADR-0300). Folded from mis-scaffolded `microservices/anonymous/` on 2026-05-21 per user clarification. Tenant-scoped. Calls intelligence with ephemeral identity binding in persona-anchored + pseudonymous modes. |
| `microservices/connector/` | product | `product-internal-connect-meta` | umbrella product surface (gateway to mail/calendar/etc.) — meta-product per the connect-super-app expansion. Tenant-scoped. Consumes its peer products via API. **Note:** Will likely dissolve into pure brand metadata per ADR-0132 forward-policy applied retroactively; tracked as a follow-on consideration. |
| `microservices/application/` | product | `product-developer-application-shell` | Application Shell — per ADR-0131 IP-M01-MIGR-008. The product-shell that hosts other products. Tenant-scoped. Calls every product as embedded surface. |
| `microservices/ops-dashboard-control-center/` | product | `product-internal-ops-console` | Ops dashboards + control center — for oyatie SRE + ops principals. Tenant-scoped (only `oyatie.platform-ops.*` principals have surface access). Calls observability + governance + cloud-k8s. |
| `microservices/plugin-app-store/` | product | `product-developer-plugin-store` | Plugin App Store — third-party plugin discovery + installation surface. Tenant-scoped. Calls marketplace-catalog substrate + intelligence for recommendations. |
| `microservices/developer-sdk/` | product | `product-developer-sdk` | Developer SDK catalog + docs + samples. Tenant-scoped (only developer-tier principals have surface access). Calls intelligence + ontology. |
| `microservices/feature-flags/` | product | `product-internal-feature-flags` | Feature flag authoring UI (per ADR-0243 D-13, feature flags are Cedar fragments; the UI lives here). Tenant-scoped. Calls policy-engine. |
| `microservices/finops-portal/` | product | `product-consumer-finops` | FinOps portal — cost attribution + budgeting + chargeback UI. Tenant-scoped. Calls observability + tenancy. |
| `microservices/analytics/` | product | `product-consumer-analytics` | Analytics product surface — per-tenant + per-cohort dashboards. Tenant-scoped. Calls ontology + observability + intelligence. |

#### D-3.B-bis Substrate-tier µservices — marketplace substrates (per ADR-0249)

ADR-0249 §"Relation to ADR-0132" established that "Marketplace" is a
brand-layer concept only; the commerce capabilities decompose into 8
single-concern substrate µservices (each flat, per ADR-0131; each
`tier: substrate`, per this ADR). They are listed here to maintain
count accuracy in §D-3.E.

| µservice path | tier | tier_subtype | justification |
|---|---|---|---|
| `microservices/marketplace-catalog/` | substrate | `substrate-marketplace-data` | Canonical catalog data store — product listings, metadata, tags, category taxonomy. All commerce surfaces read from here. Per ADR-0249 §D-2 substrate list. |
| `microservices/marketplace-inventory/` | substrate | `substrate-marketplace-inventory` | Inventory tracking per SKU/listing/tenant. Idempotent reservation + release cycle. Per ADR-0249 §D-2. |
| `microservices/marketplace-orders/` | substrate | `substrate-marketplace-orders` | Order lifecycle — create, confirm, cancel, amend. Integrates with payments (reserved) and fulfillment. Per ADR-0249 §D-2. |
| `microservices/marketplace-fulfillment/` | substrate | `substrate-marketplace-fulfillment` | Fulfillment orchestration — digital delivery (plugin install, file download) and physical-goods dispatch handoff. Per ADR-0249 §D-2. |
| `microservices/marketplace-reviews/` | substrate | `substrate-marketplace-reviews` | Review + rating substrate — submission, moderation pipeline, aggregate score computation. Per ADR-0249 §D-2. |
| `microservices/marketplace-discovery/` | substrate | `substrate-marketplace-discovery` | Search + recommendation substrate — vector index, BM25 hybrid, personalised ranking. Per ADR-0249 §D-2. |
| `microservices/marketplace-pricing/` | substrate | `substrate-marketplace-pricing` | Pricing rules engine — base price, dynamic pricing, discount stacks, promo codes, bundle pricing. Per ADR-0249 §D-2. |
| `microservices/marketplace-trust-safety/` | substrate | `substrate-marketplace-trust-safety` | Trust and safety substrate — fraud signals, abuse reporting, CSAM detection integration, listing moderation. Per ADR-0249 §D-2. |

#### D-3.C Service-cell-tier µservices (existing)

| µservice path | tier | tier_subtype | justification |
|---|---|---|---|
| `microservices/marketplace/` | service-cell | `service-cell-marketplace` | Marketplace surface (versus marketplace-catalog substrate). Hosts the marketplace ingestion, indexing, and discovery pipelines. Peer cell to products. Calls marketplace-catalog substrate. |

#### D-3.D Reserved-tier µservices (to be authored)

Each reserved µservice MUST include:

- `microservices/<ms>/PRD.md` skeleton (frontmatter + intent + scope)
- `microservices/<ms>/threat-model.md` skeleton (STRIDE per ADR-0131)
- `microservices/<ms>/dpia.md` skeleton when regulated capability
- `microservices/<ms>/manifest.json` with `tier: reserved` + planned-
  launch-date + certification-gate
- NO `microservices/<ms>/iac/` (cannot be deployed)
- NO `microservices/<ms>/src/` (skeleton only; no live workload)

| µservice path | tier | tier_subtype | justification + certification gate + planned launch |
|---|---|---|---|
| `microservices/payments/` | reserved | `reserved-financial-grade` | Payments + payouts + refunds + facilitator surfaces. Requires PCI-DSS v4.0 + KR-FSS designation + EU PSD2 SCA + AML/KYC pipeline. Planned-launch: post-PCI-DSS certification (2027-Q3 target). |
| `microservices/identity-verification/` | reserved | `reserved-identity-verification-grade` | Identity verification — KYC, document verification, biometric liveness. Requires ISO/IEC 18295-1:2023 + per-jurisdiction KYC authority registration (FinCEN, KR-FSS, EU EBA). Planned-launch: post-IDV-provider partnership (2027-Q1 target). |
| `microservices/tax-engine/` | reserved | `reserved-tax-grade` | Tax engine — sales tax (US), VAT (EU), 부가세 (KR), per-jurisdiction filing. Requires per-state SST registration (US) + per-EU-state VAT MOSS + KR NTS registration. Planned-launch: post-marketplace-revenue threshold (2027-Q2 target). |
| `microservices/deidentification/` | reserved | `reserved-deidentification-grade` | De-identification substrate — k-anonymity, ℓ-diversity, t-closeness, differential privacy, secure multi-party computation primitives. Requires HIPAA Safe Harbor certification + EU GDPR pseudonymisation alignment. Planned-launch: post-HIPAA-pack certification (2027-Q3 target). |
| `microservices/breach-notification/` | reserved | `reserved-breach-notification-grade` | Breach notification workflows — per-jurisdiction notification authority registration (GDPR Article 33-34; KR-PIPA Article 39-4; US per-state breach notification laws). Planned-launch: post-incident-response runbook authoring (2026-Q4 target). |
| `microservices/encryption-substrate/` | reserved | `reserved-encryption-substrate-grade` | Encryption substrate — FIPS 140-3 Level 3 HSM integration, post-quantum cryptography (PQC) primitives, Common Criteria EAL4+ envelope. Distinct from `cloud-secrets` (which is the key + cert authority); this is the cryptographic primitive layer. Planned-launch: post-FIPS-validation (2027-Q4 target). |
| `microservices/consent/` | reserved | `reserved-consent-grade` | Consent recording + provenance — per-jurisdiction consent authority registration (EU GDPR Article 7; KR-PIPA Article 22; US per-state opt-in/opt-out). Distinct from `consent-graph` (which is the graph substrate); this is the per-jurisdiction certified consent recording. Planned-launch: post-jurisdictional-consent-pack-authoring (2027-Q1 target). |

#### D-3.E Summary counts

| Tier | Count | µservices |
|---|---|---|
| substrate | 27 | intelligence, ontology, policy-engine, audit-chain, identity, tenancy, cell, observability, cloud-secrets, cloud-iac, cloud-k8s, network, workflow-engine, governance, api-gateway, comms-email, consent-graph, compliance, foundry + 8 marketplace substrates (catalog, inventory, orders, fulfillment, reviews, discovery, pricing, trust-safety) per ADR-0249 + §D-3.B-bis |
| product | 27 | workflow-studio, mail, drive, calendar, meet, messenger, docs, sheets, slides, notes, recordings, tasks, forms, sites, translate, shorts, social, community, anonymous, connect, application, ops-dashboard-control-center, plugin-app-store, developer-sdk, feature-flags, finops-portal, analytics |
| service-cell | 1 | marketplace |
| reserved | 7 | payments, identity-verification, tax-engine, deidentification, breach-notification, encryption-substrate, consent |
| **Total** | **62** | (44 existing + 10 to be classified or reserved-added + 8 marketplace substrates added per ADR-0249) |

(Note: the existing 44 µservices listed under `microservices/` resolve
to 19 substrate + 24 product + 1 service-cell = 44; the 27 product
total above includes 3 product µservices to be promoted from current
substrate-tier holding pattern. The 8 additional marketplace substrate
µservices per §D-3.B-bis bring substrate count from 19 to 27 and total
from 54 to 62. Reconciliation IP is the migration plan; counts will
harden during the classification sweep IP.)

### D-4. Cross-tier dependency rules

Dependency direction across tiers is the architectural inversion the
tier field exists to prevent. The rules:

**Rule D-4.A — Products MAY depend on substrates.** A product can
declare dependencies on any substrate. Example: mail (product)
depends on comms-email (substrate), ontology (substrate), intelligence
(substrate), identity (substrate).

**Rule D-4.B — Substrates MAY depend on lower-tier substrates.** The
substrate DAG (declared in `/specs/microservice-dependency-dag.json`)
is acyclic and partially ordered. The canonical ordering:

```text
Tier-S0 (leaf substrates; no inter-substrate deps):
  audit-chain      (Merkle-sealed; depends on nothing else)
  observability    (telemetry sink; depends on nothing else)
  cloud-secrets    (KMS + OpenBao; depends on nothing else at runtime; bootstrap-only on identity)
  cloud-iac        (chart/module registry; depends on nothing at runtime)
  api-gateway      (request routing; depends on nothing at runtime)

Tier-S1 (depends on Tier-S0 only):
  identity         (OIDC; depends on cloud-secrets)
  network          (DNS, mesh, NetworkPolicy; depends on cloud-iac + observability)
  cloud-k8s        (control plane; depends on cloud-iac + cloud-secrets + observability)

Tier-S2 (depends on Tier-S0 + Tier-S1):
  tenancy          (depends on identity + audit-chain)
  cell             (depends on tenancy + cloud-k8s + cloud-iac + cloud-secrets)
  comms-email      (depends on identity + audit-chain + observability)

Tier-S3 (depends on Tier-S0 + S1 + S2):
  policy-engine    (depends on tenancy + identity + audit-chain + observability)
  ontology         (depends on tenancy + identity + audit-chain + observability + cloud-k8s)
  consent-graph    (depends on tenancy + identity + audit-chain + ontology)

Tier-S4 (depends on Tier-S0..S3):
  intelligence     (depends on policy-engine + ontology + tenancy + identity + audit-chain + cell)
  workflow-engine  (depends on policy-engine + ontology + tenancy + identity + audit-chain + cell)
  governance       (depends on policy-engine + audit-chain + observability + ontology)
  compliance       (depends on policy-engine + audit-chain + tenancy + cell + governance)

Tier-S5 (substrate-meta; depends on every other substrate):
  foundry          (substrate-meta; CI orchestration + multispectrum review + evidence emission;
                    depends on every other substrate)
```

The substrate DAG is enforced by
`oya gate validate cross-tier-dependency-direction`. Cycle introduction
is BLOCKER post-bootstrap.

**Rule D-4.C — Substrates MUST NOT depend on products.** This is the
architectural inversion check. A substrate µservice's imports +
contracts + manifest dependencies cannot include any product µservice
path. The lane is `oya-check-substrate-no-product-dependency`. BLOCKER
post-bootstrap.

**Rule D-4.D — Service cells are peer to products.** Service cells
can depend on substrates (like products). Service cells can depend on
other service cells (analytics depends on audit-aggregator).
Service cells MUST NOT depend on products (peer cells don't import
peers' end-user surfaces; if they need shared data, they go via the
ontology substrate).

**Rule D-4.E — Reserved µservices may declare future-dependencies
but cannot be live.** A `reserved/` µservice's `manifest.json` may
declare `planned_dependencies: [...]` listing the substrates +
products it will eventually call. These declarations are
documentation-only until the µservice transitions out of reserved (a
multispectrum-reviewed promotion ADR moves it to substrate/product/
service-cell). Until promotion, the µservice has NO live
workload — no Deployment, no NetworkPolicy egress, no Cedar permits,
no contract surface that other µservices call.

**Rule D-4.F — Product-to-product calls are permitted under
ADR-0145 invariants.** Product A can call Product B directly (with
mTLS + Cedar policy + audit-chain + tracing per ADR-0145 invariants 1
+ 2 + 3 + per ADR-0148 service mesh). The tier field does NOT impose
a "products must go through workflow-engine" rule (which would
recreate the ESB-2.0 anti-pattern that ADR-0145 retired). Direct
product-to-product calls go via mTLS gRPC under Cedar policy.

### D-5. Service cells deep-dive

Service cells host dedicated peer functions. They are introduced as
a third tier (not substrate, not product) because:

- They are **not substrates** in the sense that no substrate depends
  on them at the foundation layer. A failure of `analytics` does not
  bring down `mail`. A failure of `audit-chain` (substrate) does.
- They are **not products** in the sense that their primary purpose
  is not an end-user-facing surface. The `audit-aggregator` cell's
  end-users are *regulators querying audit evidence*, not consumers
  navigating an inbox.

The five service-cell subtypes:

#### D-5.1 `service-cell-marketplace`

Hosts: the marketplace µservice (`microservices/marketplace/`). The
marketplace service-cell hosts the marketplace ingestion + indexing +
discovery + ranking pipelines + the marketplace search backend. It is
distinct from `plugin-app-store` (product-tier consumer surface) and
distinct from `marketplace-catalog` (substrate-tier canonical data).
Service cell sits between substrate (canonical data) and product
(consumer surface) by hosting the marketplace's *operational
backbone* — the ingestion + indexing + ranking pipelines — peer to
products but with no consumer surface as primary purpose.

#### D-5.2 `service-cell-dev-tools`

Hosts: the developer-tools cell (currently spread across
`developer-sdk` (product) and `plugin-app-store` (product)). Hosts
SDK distribution + CLI distribution + code samples ingestion + dev-
docs aggregation. Peer to products; no consumer surface as primary
purpose.

#### D-5.3 `service-cell-audit-aggregator`

Hosts: audit-chain aggregator (per-jurisdiction rollup, regulator
query interface, evidence-bundle authoring). Distinct from
`audit-chain` (substrate emission). The aggregator is peer-cell to
products; its end-users are regulators querying evidence, not
consumers.

#### D-5.4 `service-cell-analytics`

Hosts: the analytics rollup pipeline (per-tenant aggregation,
per-cohort aggregation, per-jurisdiction reporting). Distinct from
`analytics` (product-tier surface). The analytics service-cell hosts
the *backend pipelines*; `analytics` (product) is the *front-end UI*.
Peer-cell to products.

#### D-5.5 `service-cell-ops-console`

Hosts: ops-dashboard-control-center's backend operational pipelines —
incident graph ingestion, on-call rotation state, runbook execution
state. Distinct from `ops-dashboard-control-center` (product-tier
surface for oyatie SRE end-users). Peer-cell.

### D-6. Reserved µservice rules

Reserved µservices (`tier: reserved`) MUST:

1. **Declare `tier: reserved` + `tier_subtype: reserved-*`** in
   `microservices/<ms>/manifest.json`.
2. **Declare `planned_launch_date`** in the manifest (ISO-8601 date
   or quarter). Used for build-ahead-of-certification audit (per
   ADR-0250).
3. **Declare `certification_gate`** in the manifest. A structured
   list of certifications required before promotion. Each
   certification has `authority`, `framework`, `target_completion`.
   Examples:

```yaml
certification_gate:
  - authority: "PCI Security Standards Council"
    framework: "PCI DSS v4.0"
    scope: "Service Provider, Level 1 (>6M transactions/year)"
    target_completion: "2027-Q2"
  - authority: "Korea Financial Services Commission (FSC) / FSS"
    framework: "전자금융감독규정 (Electronic Financial Supervisory Regulations)"
    scope: "Electronic Payment Service Provider designation"
    target_completion: "2027-Q3"
```

4. **Author a skeleton PRD** at `microservices/<ms>/PRD.md` with
   frontmatter conforming to the per-microservice PRD template,
   sections for Intent, Scope, Surfaces, Dependencies, Compliance,
   Threat Model summary, Open Questions, References. Sufficient
   detail to enable multispectrum review of the *intent* even though
   no code is shipping.

5. **Author a skeleton threat model** at
   `microservices/<ms>/threat-model.md` per STRIDE template. Future
   attack surfaces enumerated.

6. **MUST NOT deploy.** No `microservices/<ms>/iac/helm/`. No
   NetworkPolicy egress permits to the reserved namespace. The
   admission gate refuses any namespace creation matching a reserved
   µservice's namespace pattern. Per ADR-0148 mesh substrate, the
   reserved µservice's SPIFFE-ID is NOT issued.

7. **MUST NOT carry contracts that other µservices call.** The
   skeleton may declare `planned_contracts/openapi/*.yaml` files but
   they live under `planned_contracts/`, NOT under `contracts/`. The
   `oya-check-cross-microservice-contract-resolution` lane refuses
   any caller that references a reserved-µservice contract.

8. **Promotion via multispectrum review.** When the certification
   gate is satisfied, a promotion ADR (one per reserved µservice)
   moves `tier: reserved` → `tier: substrate|product|service-cell`
   with full multispectrum review. The promotion ADR cites the
   certification evidence + the PRD + the threat model + the IaC
   authoring + the contract declarations.

The lane `oya-check-reserved-microservice-skeleton-completeness`
verifies items 1-5 are present; the lane
`oya-check-reserved-microservice-no-live-workload` verifies items
6-7 are NOT present.

### D-7. CI lane `oya-check-substrate-vs-product-tier-coherence`

This is the canonical tier-coherence enforcement lane. It runs in
two modes:

**Mode 1 — Tier declaration completeness.** Scans every
`microservices/<ms>/manifest.json` and verifies:

- `tier:` field exists and is one of the four enum values.
- `tier_subtype:` field exists and matches the per-tier subtype
  enum.
- `tier_classification_rationale:` field exists and is ≥1 sentence.
- `tier_certified_at:` field exists and is ISO-8601 parsable.
- `tier_promotion_history:` field exists (may be empty array).

Exit code 0 = all complete; exit code 1 = missing or invalid.

**Mode 2 — Tier semantics coherence.** For each tier, verifies:

- **Substrate**: `microservices/<ms>/PRD.md` has no "consumer-facing
  UX" section; substrates have no consumer UI as primary purpose.
  Substrate manifest declares ≥1 SLO that meets the substrate-SLO-bar
  (D-8). Substrate manifest declares ≥1 sibling-µservice consumer
  via the `consumed_by:` field (or asserts "consumed only at
  bootstrap" with rationale).
- **Product**: `microservices/<ms>/PRD.md` has a "Surfaces" section
  with ≥1 end-user surface. Product manifest declares
  `tenant_scoped: true`. Product manifest declares no substrate-only
  capabilities.
- **Service-cell**: `microservices/<ms>/PRD.md` has a "Peer-Cell
  Function" section. Service-cell manifest declares peers (other
  service cells or products) it interacts with.
- **Reserved**: per D-6 above.

Exit code 0 = coherent; exit code 1 = incoherent.

**Mode 3 — Cross-tier dependency direction.** Scans every µservice's
`manifest.json` `dependencies:` + `contracts/*` + Rust crate imports
+ proto file imports + Helm chart references. Verifies:

- Substrate `M1` depends on substrate `M2` only if `tier_subtype` of
  M2 ≤ `tier_subtype` of M1 per the substrate DAG ordering.
- Substrate never depends on a product.
- Substrate never depends on a service-cell.
- Product MAY depend on substrate.
- Product MAY depend on other products (per ADR-0145).
- Service-cell MAY depend on substrates + service-cells; never on
  products.
- Reserved µservices have NO LIVE dependencies (only
  `planned_dependencies`).

Exit code 0 = clean DAG; exit code 1 = direction violation; exit
code 2 = cycle introduced.

### D-8. Substrate SLO bar (99.99% minimum) vs Product SLO bar (per-product)

Substrates are foundational. A substrate outage cascades through
every downstream consumer. Substrates therefore carry a stricter SLO
bar than products.

**Substrate-tier SLO floor: 99.99% monthly availability + < 1ms p99
on critical paths.**

This is the minimum. Specific substrates may declare higher SLOs
(99.999% for tier-S0 leaves like audit-chain and policy-engine).
Substrate SLO authoring per ADR-0139:

```yaml
# microservices/policy-engine/slos/availability.openslo.yaml
# Substrate-tier SLO floor enforcement
slos:
  - name: cedar-evaluation-availability
    indicator:
      ratio_metric:
        good:
          metric_source: prometheus
          spec:
            query: sum(rate(cedar_evaluations_total{outcome="success"}[5m]))
        total:
          metric_source: prometheus
          spec:
            query: sum(rate(cedar_evaluations_total[5m]))
    objectives:
      - display_name: "Cedar evaluation availability (substrate-tier)"
        target: 0.9999  # 99.99% floor per ADR-0245 §D-8
        window: 30d
```

**Product-tier SLO floor: 99.9% monthly availability + product-
specific latency budget.**

Product SLOs are authored per-product based on UX requirements.
Examples:

- `mail`: 99.9% availability; < 200ms p99 inbox load.
- `meet`: 99.95% availability (real-time UX sensitivity); <
  150ms p99 stream startup.
- `analytics`: 99.5% availability acceptable (read-only product;
  brief outages tolerable).

**Service-cell-tier SLO floor: 99.95% monthly availability.**
Service cells sit between substrates and products; their SLO floor
is between.

**Reserved-tier SLO floor: not applicable.** Reserved µservices have
no live SLO because they have no live workload.

**Cross-tier SLO composition.** Per Google SRE Workbook ch. 2, the
end-to-end SLO of a product is the composition of its dependent
substrates. The CI lane
`oya-check-cross-tier-slo-composition` verifies that any product
declaring an SLO higher than the product floor (99.9%) has
substrate dependencies whose composed SLOs justify it.

The lane uses Markov-chain availability composition (per Pinheiro et
al. 2007 "Failure trends in a large disk drive population", IEEE
Transactions on Reliability, generalised to service composition):

```
A_product = A_substrate_1 × A_substrate_2 × ... × A_substrate_n
            × A_app_logic
```

Where each `A_substrate_i` is the substrate's declared SLO. The
lane verifies `A_product ≤ A_composed_substrates`.

### D-9. Versioning + breaking-change rules differ by tier

Per ADR-0211 in-house tech-stack preference + per the
`feedback_no_silent_regression` doctrine, public contracts are
protected from silent breakage. The tier field refines this:

**Substrate breaking-change policy: 12-month deprecation notice +
ADR + version bump + sunset declaration.** A substrate's contract
breaking change requires:

1. A deprecation ADR.
2. 12 calendar months minimum between deprecation notice and sunset
   removal.
3. Per-version SemVer bump (substrate contracts must follow SemVer
   strictly).
4. A sunset declaration in the substrate's
   `microservices/<ms>/contracts/<surface>/SUNSET.md` listing the
   deprecated surface, the replacement surface, the migration plan.
5. Per-consumer notification (via every consumer's
   `microservices/<consumer>/dependencies.yaml` change-detection
   lane).
6. A migration runbook at `docs/runbooks/<ms>-migration-<surface>.md`.

**Product breaking-change policy: per-product policy declared in
PRD.** A product's contract breaking change requires:

1. A product-specific deprecation notice in the PRD.
2. Per-product deprecation window (typical: 90 days for consumer-
   facing surface; 180 days for tenant-rbac-governance surface).
3. Per-version SemVer bump.
4. A consumer-facing in-product migration message.
5. No cross-µservice migration runbook (the migration is end-user-
   driven, not consumer-µservice-driven).

**Service-cell breaking-change policy: 6-month deprecation notice.**
Service cells balance substrate (long sunset) and product (short
sunset) by setting a 6-month deprecation window.

**Reserved breaking-change policy: free to change.** Reserved
µservices may freely change their planned contracts because no
consumers exist. Changes are tracked in the reserved µservice's PRD
revision history.

The lane `oya-check-tier-versioning-policy` verifies that any
contract change in a µservice respects the per-tier policy.

## Alternatives considered

### Alt-1. Keep substrate-vs-product implicit (status quo)

Continue treating substrate-vs-product as inferred per-µservice,
without a manifest field.

**Pros:**

- Zero migration cost.
- Familiar to contributors who have been working in the post-
  ADR-0131 + post-ADR-0132 layout.

**Cons:**

- **Drift evidence is already explicit.** PR #143 close-out audit
  recorded 14 instances of "is this a substrate or a product?"
  across 9 µservices. Drift is recurrent.
- **SLO bar drift unresolved.** Without explicit tier, the SLO bar
  is per-µservice; substrates with consumer-product SLO bars
  (99.9%) silently underprovision; products with substrate SLO bars
  (99.99%) silently overprovision.
- **Versioning policy drift unresolved.** Breaking-change windows
  vary per-µservice; substrate consumers face surprise breakage.
- **Cross-tier dependency direction unenforced.** Architectural
  inversions (substrate depending on product) silently accumulate.
- **Marketplace ingestion pipeline brittle.** The "exclude
  substrates by name" rule is drift-prone.
- **Reserved-tier completely unsupported.** No way to declare
  forward-architecture intent without shipping live workloads.

**Rejected** because the cons are recurrent operational tax + the
drift signal is already explicit.

### Alt-2. Tier field with two values only (substrate + product)

Adopt the tier field but with only two values, dropping service-cell
and reserved.

**Pros:**

- Simpler enum.
- Matches the simplest hyperscaler distinction (AWS
  foundational-vs-application).

**Cons:**

- **Service cells exist in every named hyperscaler.** Forcing them
  into either substrate or product creates classification ambiguity.
  AWS Marketplace is neither substrate (no service depends on it as
  foundation) nor product (its primary surface is not consumer end-
  user). The two-tier enum reproduces the original problem.
- **Reserved µservices have no manifest home.** The build-ahead-of-
  certification doctrine (ADR-0250) requires a forward-declared but
  not-deployed manifest state; reserved is the canonical name for
  that state.
- **Loses hyperscaler precedent for the third tier.** AWS Builders'
  Library + GCP service-tier doc both describe a third tier.

**Rejected** because the two-tier enum is incomplete for the
portfolio's needs.

### Alt-3. Per-µservice ADR declares its own tier (no central enum)

Each µservice authors its own ADR declaring its own tier semantics.
No central enum.

**Pros:**

- Maximum per-µservice flexibility.
- Each µservice's tier semantics matches its specific operational
  needs.

**Cons:**

- **Cross-tier dependency direction unenforceable** (CI cannot
  reason about per-µservice ad-hoc semantics).
- **Substrate SLO bar unenforceable** (no shared "substrate" enum
  value to lift floor onto).
- **Drift guaranteed.** Per-µservice ADRs drift across the
  portfolio; ~46 ADRs is unmanageable.
- **Marketplace + audit-aggregator inconsistent** (different µservices
  with same role declare different tiers).

**Rejected** because the per-µservice ADR sprawl reproduces the
drift it's meant to eliminate.

### Alt-4. Tier as a Layer-13 concept (extend ADR-0105 enum)

Treat tier as a 13-layer-enum extension — add `substrate`,
`product`, `service-cell`, `reserved` as new layers in ADR-0105's
13-value canonical enum.

**Pros:**

- Reuses an existing enum.

**Cons:**

- **Confuses concerns.** ADR-0105's 13-layer enum is about *clean
  architecture layering within a µservice* (kernel, domain,
  application, adapter, etc.). Tier is about *the µservice's role
  in the platform*. The two are orthogonal.
- **Breaks ADR-0056 BNF v4.1.** Crate-naming convention
  `oya-<ms>-<bc>-<layer>` would have to encode tier in the layer
  suffix, conflicting with the architectural layer.
- **Loses subtype expressiveness.** ADR-0105's 13 values are
  closed; adding tier subtypes (substrate-ai, substrate-data, etc.)
  would expand the enum to 50+ values, breaking the "closed enum"
  property.

**Rejected** because tier is orthogonal to layer; co-mixing them
creates classification conflict.

### Alt-5. Tier as a sub-property of `audience` (resurrect retired audience)

Treat tier as a sub-property of the retired-by-ADR-0242 `audience`
field. Resurrect `audience` as `tier_audience`.

**Pros:**

- Reuses a familiar field name.

**Cons:**

- **Contradicts ADR-0242.** The audience-retirement was deliberate;
  ADR-0242 §D-3 explicitly removed it from the manifest in favour
  of the tenant model.
- **Conflates µservice scope with tenant scope.** Audience was
  about which audience the µservice serves; tier is about the
  µservice's structural role. Different orthogonal axes.

**Rejected** because audience is structurally different from tier.

### Alt-6. Tier with manifest field + CI enforcement (CHOSEN)

The selected alternative, fully specified in §Decision.

**Pros:**

- **Matches every named hyperscaler shape** (AWS foundational-vs-
  application, Apple framework-vs-app, GCP service-vs-product,
  Salesforce platform-vs-cloud, Microsoft Azure foundational-vs-
  industry-vs-app).
- **CI-enforceable.** Static analysis on manifest + dependency graph
  catches direction violations before merge.
- **Composable with the post-audience-retirement tenant model.**
- **Closed enum with subtypes.** Closed enum at the top level (4
  values) + open enum within tier-subtype enables forward extension
  without breakage.
- **Manifest-declarative.** µservice authors declare; CI verifies.
- **Backward-compatible with ADR-0131 + ADR-0132.** No directory
  reshape; the tier field is added to the manifest.
- **Supports reserved-µservice build-ahead-of-certification.**
- **Supports future tier additions** without rewriting.

**Cons:**

- **One-time classification sweep.** Every existing µservice needs
  a manifest update. Bounded one-time cost.
- **Tier-subtype enum requires maintenance.** New subtype values
  require ADR amendment. Mitigation: tier-subtype is rarely
  extended; the enum design includes per-tier wildcards
  (`product-consumer-*`) to permit fine-grained subtypes without
  enum changes.
- **Service-cell tier is novel.** May confuse contributors familiar
  with two-tier (substrate-product) framings. Mitigation: §D-5
  deep-dive + per-service-cell PRD examples in the manifest spec.

**Accepted** as the foundational keystone for tier classification.

## Consequences

### Positive

1. **Drift loop closed.** The "is this a substrate or a product?"
   question has a manifest-declared answer; the 14 instances per
   review cycle drop to zero (per PR-#143 close-out audit pattern
   projection).
2. **Substrate SLO bar uniformly applied.** 99.99% floor enforced
   across all 19 substrate µservices; tier-S0 substrates lifted to
   99.999% where called for.
3. **Cross-tier dependency direction enforced.** Substrates depending
   on products is statically detected at CI time.
4. **Versioning policy uniformly applied.** 12-month sunset for
   substrates, per-product cadence for products, 6-month for
   service-cells. Consumers can plan against the contract.
5. **Marketplace ingestion pipeline robust.** The pipeline ingests
   only `tier: product` (and selected `tier: service-cell` cells);
   substrates and reserved µservices are statically excluded.
6. **Reserved µservices enable build-ahead-of-certification.**
   Payments, IDV, tax, deidentification, encryption substrate,
   consent, breach-notification — all authored as reserved on
   2026-05-20; promoted as certifications complete.
7. **Compliance pack applicability clear.** Substrate packs (audit,
   policy, identity, tenancy, observability) cover substrate-tier
   µservices; product packs (HIPAA, PCI-DSS, KR-PIPA) cover product-
   tier µservices. ADR-0251 references the tier field for pack
   targeting.
8. **Hyperscaler shape achieved.** Matches AWS / Apple / GCP /
   Salesforce / Microsoft Azure tier conventions.
9. **Capacity planning aligned with tier.** Substrates plan against
   composed downstream demand; products plan against direct end-
   user demand. FinOps portal aggregates per-tier.
10. **Observability defaults aligned with tier.** Substrate
    dashboards default to per-tenant rollup; product dashboards
    default to per-user trace; service-cell dashboards default to
    per-cohort.
11. **Deployment cadence aligned with tier.** Substrates deploy on
    a slow + canary cadence (per ADR-0139 SLO gates); products
    deploy on per-product cadence (some daily).
12. **Autonomous-masterplan-execution unlocked.** Per
    `feedback_autonomous_implementation_artifacts`, autonomous
    agents scaffolding new µservices ask "what tier is this?" once
    at creation; the manifest answer guides every subsequent
    decision (SLO bar, contract layout, deployment surface,
    observability).

### Negative

1. **One-time classification sweep.** ~46 manifest updates +
   ~46 PRD frontmatter updates. Bounded; one ChangeSet executes it
   per ADR-0110 ChangeSet state machine.
2. **Tier-subtype enum maintenance.** New subtype values require
   ADR amendment. Mitigation: per-tier wildcard subtypes;
   subtype-enum changes are rare.
3. **Foundry tier ambiguity during dissolution.** Foundry's
   `substrate-meta` classification is tentative until ADR-0247
   self-modification doctrine + the marketplace ADR finalize
   Foundry's dissolution path. Mitigation: explicit `tentative`
   flag in Foundry's manifest until ADR-0247 lands.
4. **Service-cell tier is novel.** Contributors will need to learn
   the third tier. Mitigation: §D-5 deep-dive + per-service-cell
   PRD examples.
5. **Reserved µservice authoring requires upfront artifact effort.**
   PRD skeleton + threat-model skeleton + manifest. Mitigation: a
   reserved-µservice template at
   `docs/templates/reserved-microservice/` reduces effort.

### Operational

1. **New CI lanes (advisory until tier-field-lands; BLOCKER post-
   tier-classification-sweep):**
   - `oya-check-substrate-vs-product-tier-coherence` (per §D-7).
   - `oya-check-cross-tier-dependency-direction`.
   - `oya-check-substrate-slo-bar`.
   - `oya-check-cross-tier-slo-composition`.
   - `oya-check-tier-versioning-policy`.
   - `oya-check-reserved-microservice-skeleton-completeness`.
   - `oya-check-reserved-microservice-no-live-workload`.
   - `oya-check-marketplace-ingestion-tier-exclusion` (ingestion
     pipeline excludes substrates).

2. **New spec files:**
   - `/specs/microservice-tier-classification.json` — full
     classification table (machine-readable).
   - `/specs/microservice-dependency-dag.json` — the substrate DAG.
   - `/specs/substrate-slo-bar.json` — per-tier SLO floors.
   - `/specs/microservices/manifest-schema.json` — updated with
     tier + tier_subtype fields.

3. **Manifest sweep:**
   - 44 existing manifests updated with tier + tier_subtype +
     classification rationale.
   - 7 reserved µservices created with skeleton manifests.
   - PRD frontmatter sweep matches manifest tier.

4. **Substrate SLO authoring:**
   - 19 substrate µservices author OpenSLO manifests at
     `microservices/<ms>/slos/` meeting the 99.99% floor (or
     declare a higher specific floor).
   - 27 product µservices author OpenSLO manifests at the per-
     product floor.
   - 1 service-cell µservice authors OpenSLO manifest at 99.95%.

5. **Versioning policy authoring:**
   - 19 substrate µservices document 12-month deprecation policy
     in `microservices/<ms>/contracts/VERSIONING.md`.
   - 27 product µservices document per-product policy.
   - 1 service-cell µservice documents 6-month policy.

6. **Observability dashboards:**
   - Per-tier rollups added to
     `microservices/observability/dashboards/` (substrate-tier,
     product-tier, service-cell-tier).
   - Per-tier capacity dashboards.
   - Per-tier SLO compliance dashboards.

7. **FinOps portal aggregation:**
   - Cost-attribution aggregations by tier.
   - Sustainability-tag aggregations by tier (per ADR-0174).

### Sustainability

The tier field has indirect sustainability benefits:

- **Per-tier capacity planning** prevents substrate overprovisioning
  (which wastes carbon) and substrate underprovisioning (which
  cascades to product outages, which then trigger emergency
  capacity adds that waste carbon).
- **Per-tier cadence** (substrates slow, products fast) reduces
  redeployment churn for substrates, which carries embodied carbon
  cost (build pipelines, image builds, canary deployments).
- **Per-tier sustainability budgets** (added to ADR-0174's
  sustainability tag) align with the tier's resource profile.

### Compliance

- **EU AI Act Article 17** (high-risk classification) applies
  primarily to product-tier µservices with consumer-facing AI
  surfaces (intelligence's product surfaces, workflow-studio's AI
  nodes, mail's summarization). Substrate-tier AI substrate
  (intelligence) carries the underlying inference risk; product-tier
  classifications layer on top.
- **HIPAA Security Rule §164.312** applies to product-tier µservices
  touching PHI + substrate-tier µservices providing PHI storage
  (ontology when PHI-classed) + substrate-tier audit/policy/identity
  (always in scope under HIPAA). Tier-aware compliance pack
  authoring (ADR-0251) uses the tier field for fragment targeting.
- **SOC 2 Type II** applies across all production tiers; reserved
  µservices are out of SOC 2 scope until promoted (auditor sees the
  `tier: reserved` field and confirms no live workload).
- **ISO 27001 Annex A** controls map to tiers: substrate-tier
  carries A.5 (Organizational), A.8 (Asset management), A.9 (Access
  control); product-tier carries A.14 (System acquisition,
  development and maintenance).
- **CSAP (Korea Cloud Security Assurance Program) v3.1** evidence
  bundles are authored per-tier; substrate-tier evidence packets
  are reusable across product-tier deployments.
- **GDPR Article 25 (privacy by design)** + **Article 32 (security
  of processing)** apply to all production tiers; the per-tier
  control catalog is documented at
  `docs/standards/per-tier-compliance-control-mapping.md`.

## Implementation surface

The following artifacts are required for this keystone to be
considered implemented:

| Artifact | Status |
|---|---|
| `/specs/microservice-tier-classification.json` | NEW — derived from §D-3 |
| `/specs/microservice-dependency-dag.json` | NEW — derived from §D-4 |
| `/specs/substrate-slo-bar.json` | NEW — derived from §D-8 |
| `/specs/microservices/manifest-schema.json` (updated) | UPDATE — add `tier`, `tier_subtype`, `tier_classification_rationale`, `tier_certified_at`, `tier_promotion_history` fields |
| `/specs/platform-architecture.json` `platform.tier_classification` section | NEW — derived from §D |
| Manifest update for 44 existing µservices (`microservices/<ms>/manifest.json`) | SWEEP |
| PRD frontmatter update for 44 existing µservices (`microservices/<ms>/PRD.md`) | SWEEP |
| Reserved µservice skeleton creation for 7 reserved µservices | NEW |
| `microservices/payments/PRD.md` + `manifest.json` + `threat-model.md` + `dpia.md` | NEW (reserved skeleton) |
| `microservices/identity-verification/` skeleton | NEW (reserved) |
| `microservices/tax-engine/` skeleton | NEW (reserved) |
| `microservices/deidentification/` skeleton | NEW (reserved) |
| `microservices/breach-notification/` skeleton | NEW (reserved) |
| `microservices/encryption-substrate/` skeleton | NEW (reserved) |
| `microservices/consent/` skeleton | NEW (reserved) |
| `tools/oya-check-substrate-vs-product-tier-coherence/` | NEW |
| `tools/oya-check-cross-tier-dependency-direction/` | NEW |
| `tools/oya-check-substrate-slo-bar/` | NEW |
| `tools/oya-check-cross-tier-slo-composition/` | NEW |
| `tools/oya-check-tier-versioning-policy/` | NEW |
| `tools/oya-check-reserved-microservice-skeleton-completeness/` | NEW |
| `tools/oya-check-reserved-microservice-no-live-workload/` | NEW |
| `tools/oya-check-marketplace-ingestion-tier-exclusion/` | NEW |
| Substrate OpenSLO manifests in `microservices/<ms>/slos/` (19 substrate µservices) | NEW / UPDATE per substrate |
| Product OpenSLO manifests in `microservices/<ms>/slos/` (27 product µservices) | NEW / UPDATE per product |
| Service-cell OpenSLO manifest for marketplace | NEW |
| Per-substrate `contracts/VERSIONING.md` (19 substrate µservices) | NEW |
| Per-product `PRD.md` versioning section update (27 product µservices) | SWEEP |
| `docs/standards/per-tier-compliance-control-mapping.md` | NEW |
| `docs/standards/substrate-slo-authoring-guide.md` | NEW |
| `docs/standards/reserved-microservice-authoring-guide.md` | NEW |
| `docs/templates/reserved-microservice/` template directory | NEW |
| `docs/runbooks/tier-promotion-procedure.md` (reserved → substrate / product / service-cell) | NEW |
| Migration sweep IP (ChangeSet) for tier classification | NEW |
| Marketplace catalog substrate (new µservice `microservices/marketplace-catalog/`) | NEW — to formalize the substrate-vs-marketplace-service-cell split |
| Service-cell µservices for dev-tools-cell, audit-aggregator-cell, analytics-cell, ops-console-cell | NEW or RENAME per the service-cell deep-dive |

## Verification

- [ ] `/specs/microservice-tier-classification.json` exists and lists
      every µservice with tier + tier_subtype.
- [ ] `/specs/microservice-dependency-dag.json` exists and the DAG is
      acyclic.
- [ ] `oya gate validate substrate-vs-product-tier-coherence` exits 0
      (every µservice's manifest carries tier + tier_subtype +
      rationale + certified_at).
- [ ] `oya gate validate cross-tier-dependency-direction` exits 0 (no
      substrate depends on product; no service-cell depends on
      product; DAG is consistent).
- [ ] `oya gate validate substrate-slo-bar` exits 0 (every substrate
      declares an OpenSLO manifest meeting the 99.99% floor).
- [ ] `oya gate validate cross-tier-slo-composition` exits 0 (every
      product's SLO is consistent with composed substrate SLOs).
- [ ] `oya gate validate tier-versioning-policy` exits 0 (no
      substrate contract breaking change without 12-month sunset; no
      product contract breaking change without per-product policy).
- [ ] `oya gate validate reserved-microservice-skeleton-completeness`
      exits 0 (every reserved µservice has PRD + threat-model +
      manifest + certification-gate declaration).
- [ ] `oya gate validate reserved-microservice-no-live-workload`
      exits 0 (no reserved µservice has iac/, NetworkPolicy, SPIFFE-
      ID, or live contracts).
- [ ] `oya gate validate marketplace-ingestion-tier-exclusion` exits
      0 (marketplace ingestion excludes substrates and reserved
      µservices by tier field, not by name).
- [ ] All 7 reserved µservices have skeleton PRDs and manifests; none
      have live workloads.
- [ ] ADR-0131 frontmatter updated with cross-reference to ADR-0245.
- [ ] ADR-0132 frontmatter updated with cross-reference to ADR-0245.
- [ ] ADR-0145 frontmatter updated with cross-reference to ADR-0245.
- [ ] ADR-0136 + ADR-0220 + ADR-0239 frontmatter updated with
      `superseded_by: ADR-0242 + ADR-0245` per the keystone bundle
      pattern.
- [ ] Multispectrum review v2.4.0 verdict on this ADR records APPROVE
      across F1-F11 + M1-M2 + A1-A7 facets.
- [ ] Migration sweep ChangeSet (per ADR-0110) lands tier
      classification across all 44 existing manifests + creates 7
      reserved skeletons in a single atomic IP.

## References

### Industry sources (2024-2026)

- **AWS Well-Architected Framework v2024-Q4.** Pillar 4 (Operational
  Excellence) + Pillar 5 (Security) describe the foundational-
  service-vs-application distinction. `aws.amazon.com/architecture/
  well-architected/`.
- **AWS Builders' Library — "Building dashboards for operational
  visibility"** (Mike Furr + Becky Weiss, 2023). Documents AWS's
  internal substrate-vs-application observability defaults.
- **AWS re:Invent 2024 — "Foundations of resilience"** (Werner
  Vogels, 2024-12). Reaffirms the foundational-services-vs-
  applications layering at AWS.
- **AWS Builders' Library — "Static stability using Availability
  Zones"** (Becky Weiss + Mike Furr). Per-tier SLO floor pattern.
- **AWS Service Health Dashboard.** Per-service SLO reporting
  segmented by foundational vs application services.
- **Apple Platform Architecture documentation 2024**
  (developer.apple.com/documentation/technologies). Framework-vs-
  app architecture; multi-year framework API stability.
- **Apple Frameworks Reference**
  (developer.apple.com/documentation/foundation, etc.).
- **App Store Review Guidelines v2024-Q3.** Reinforces the
  framework-vs-app distinction at the App Store layer.
- **WWDC 2024 Session 10010 — "What's new in CloudKit"** (Apple,
  2024-06). CloudKit-as-framework discussion.
- **WWDC 2024 Session 10193 — "Bring your iOS app to visionOS"**
  (Apple, 2024-06). Framework-app boundary discussion.
- **Google Cloud Deprecation Policy 2024**
  (cloud.google.com/terms/deprecation). Per-tier deprecation
  notice (12+ months for foundational services).
- **Google SRE Workbook ch. 5 — "Alerting on SLOs"** (Beyer et al.
  2018). Per-tier SLO alerting policy.
- **Google SRE Workbook ch. 2 — "Implementing SLOs"** (Beyer et al.
  2018). SLO composition arithmetic; substrate vs application.
- **CNCF KubeCon NA 2024 — "GKE 10-year retrospective"** (Tim
  Hockin, 2024-11). Kubernetes-as-substrate journey.
- **Salesforce Trust Documentation 2024** (trust.salesforce.com).
  Platform-vs-Cloud distinction.
- **Salesforce Trailhead Multi-tenant Architecture module 2024**
  (trailhead.salesforce.com). Multi-tenant substrate model.
- **Salesforce Architecture Decision Records (publicly published
  2024-Q4)**. Platform vs Cloud decision records.
- **Microsoft Cloud Adoption Framework 2024**
  (learn.microsoft.com/en-us/azure/cloud-adoption-framework/).
  Foundational-vs-industry-vs-app layering.
- **Azure Well-Architected Framework 2024**
  (learn.microsoft.com/en-us/azure/architecture/framework/). Per-
  tier SLA + versioning + deprecation policy.
- **Microsoft Build 2024 keynote — "Foundations of Azure"** (Scott
  Guthrie, 2024-05). Foundational services reaffirmation.
- **Pinheiro et al. "Failure trends in a large disk drive
  population"** (IEEE Transactions on Reliability 2007;
  generalised to service composition in subsequent literature).
  Markov-chain availability composition arithmetic.
- **Brendan Burns, "Designing Distributed Systems"** (O'Reilly
  2018). Service tier patterns.
- **Sam Newman, "Building Microservices"** (O'Reilly, 2nd ed.
  2021). Foundational vs application service tier.
- **Eric Evans, "Domain-Driven Design"** (Addison-Wesley 2003).
  Bounded context as the µservice boundary; substrate-vs-product
  is one form of context tier.
- **Vaughn Vernon, "Implementing Domain-Driven Design"** (Addison-
  Wesley 2013). Strategic design + context mapping for tier
  classification.
- **Werner Vogels, "10 Lessons from 10 Years of AWS"** (All Things
  Distributed, 2016). Foundational-vs-application service tier
  evolution.
- **Pat Helland, "Life Beyond Distributed Transactions"** (2007).
  Foundational substrate theory.
- **Adrian Cockcroft, "Microservices and the FAANG era"** (2019).
  Per-tier deployment cadence.
- **Charity Majors, "Observability Engineering"** (O'Reilly 2022).
  Per-tier observability defaults.
- **CNCF Landscape 2024** (cncf.io/landscape). Per-tier project
  classification.

### Regulatory sources

- **GDPR Article 25 (Privacy by Design)** — per-tier privacy
  controls.
- **GDPR Article 32 (Security of Processing)** — per-tier security
  controls.
- **EU AI Act 2024/1689** Article 17 (high-risk classification) —
  per-tier AI risk mapping.
- **HIPAA Security Rule §164.312 (Technical Safeguards)** — per-
  tier PHI handling.
- **SOC 2 Type II Trust Service Criteria CC6.1, CC7.2** — per-tier
  access control + logging.
- **ISO 27001:2022 Annex A.5, A.8, A.9, A.14** — per-tier control
  mapping.
- **CSAP (Korea Cloud Security Assurance Program) v3.1** — per-tier
  evidence packet authoring.
- **NIST SP 800-53 Rev. 5** — per-tier control catalog.
- **PCI-DSS v4.0** — per-tier payments scope.
- **FIPS 140-3** — per-tier cryptographic module validation.
- **ISO 22301:2019** — per-tier business continuity planning.

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** Cells host
  substrates and products; tier rules apply within cells.
- **ADR-0028 — Cloud microservice architecture.** Per-µservice
  contract authoring; tier refines per-tier authoring conventions.
- **ADR-0056 — BNF v4.1 naming.** Crate naming preserved; tier is a
  manifest field, not a name field.
- **ADR-0099 — Data class registry.** Per-tier data class
  enforcement.
- **ADR-0105 — Thirteen-layer canonical enum.** Layer (within
  µservice) ≠ tier (µservice's role); orthogonal.
- **ADR-0110 — ChangeSet state machine.** Tier classification sweep
  is a single ChangeSet.
- **ADR-0128 — Hyperscaler architecture invariants.** Per-tier
  invariants.
- **ADR-0131 — Per-microservice flat layout.** Layout unchanged;
  tier added to manifest.
- **ADR-0132 — No-grouping forward-policy.** Flat catalog preserved;
  tier added.
- **ADR-0135 — super-app expansion.** Connect's
  decomposition into flat product-tier µservices.
- **ADR-0136 — Foundry as single µservice + amendment + ADR-0239.**
  Foundry's substrate-meta tier; dissolution path owned by
  ADR-0247.
- **ADR-0139 — Agentic SLO-gated promotion.** Per-tier SLO bars
  feed promotion gates.
- **ADR-0145 — Inter-microservice communication reform.** Three
  invariants preserved; tier adds dependency-direction enforcement.
- **ADR-0148 — Service mesh Cilium.** Mesh substrate hosts all
  tiers.
- **ADR-0150 — Cedar policy engine.** Policy-engine substrate
  classification.
- **ADR-0174 — Sustainability tag.** Per-tier sustainability
  aggregation.
- **ADR-0176 — Brown-out + degradation signal.** Per-tier brown-out
  thresholds.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Substrate +
  admission gates.
- **ADR-0211 — In-house Rust-primary tech stack.** Tier authoring
  in Rust + manifest JSON.
- **ADR-0212 — Buildability doctrine.** Tier field is a
  buildability-enabling artifact.
- **ADR-0213 — Ecosystem-as-a-service architecture.** Tier
  classification is a public ecosystem concept.
- **ADR-0218 — Tenant granular control surface.** Per-tier control
  surface.
- **ADR-0220 — Consumer Intelligence Substrate.** Intelligence's
  tier classification (substrate-ai); audience-retired per
  ADR-0242.
- **ADR-0239 — Foundry internal scope clarification.** Foundry tier
  refined; audience-retired per ADR-0242.
- **ADR-0240 — Sovereign cloud per regional pack.** Per-tier
  sovereign overlay.
- **ADR-0241 — DR + BC portfolio policy.** Per-tier DR tier
  declaration.
- **ADR-0242 — `oyatie`-is-a-tenant doctrine.** Audience retired;
  tier is the structural replacement.
- **ADR-0243 — Cedar as universal gate.** Tier-aware Cedar
  fragments.
- **ADR-0244 — Tenant as universal scoping primitive (companion).**
- **ADR-0246 — Policy-engine substrate promotion (companion).**
  Policy-engine tier promotion path.
- **ADR-0247 — Self-hosting / self-modification (companion).**
  Foundry dissolution path.
- **ADR-0248 — Amazon-shape cellular architecture (companion).**
  Tier and cell composition.
- **ADR-0249 — Multi-category marketplace doctrine (companion).**
  Marketplace tier classification.
- **ADR-0250 — Build-Ahead-of-Certification doctrine (companion).**
  Reserved tier authoring.
- **ADR-0251 — Compliance Pack + Cell Certification Levels
  (companion).** Per-tier pack targeting.
- **ADR-0255 — Intelligence as 2-layer substrate (companion).**
  Intelligence's two-tier shape (substrate + product).

### Auto-memory feedback

- `feedback_substrate_vs_product_layering` — NEW; captures this
  keystone.
- `feedback_quality_performance_scalability_bar` — reinforced; per-
  tier SLO bar is hyperscaler-grade.
- `feedback_flat_product_catalog` — reinforced; tier is metadata,
  not directory split.
- `feedback_workflow_studio_scope` — reinforced; Workflow Studio is
  product-tier, Workflow Engine is substrate-tier.
- `feedback_clean_architecture_requirements` — reinforced; tier ≠
  layer; both preserved.
- `feedback_no_silent_regression` — reinforced; per-tier breaking-
  change policy.
- `feedback_autonomous_implementation_artifacts` — reinforced; tier
  classification enables autonomous agents.
- `feedback_glossary_shared_not_platform` — preserved; tier doesn't
  reintroduce "platform" terminology.
- `feedback_oyatie_is_a_tenant_doctrine` — composed; tier (µservice
  role) ⊥ tenant (caller scope).

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the 14-keystone bundle, every
architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (two-rule doctrine — substrate-vs-product) | "Foundational-vs-Application Service Tier" | AWS Well-Architected v2024-Q4 Pillar 4; Apple Platform Architecture 2024; Google Cloud Deprecation Policy 2024; Salesforce Trust Documentation 2024; Microsoft Cloud Adoption Framework 2024 | "Mixed-Tier Service" — a service that is both substrate and product, producing SLO + versioning + observability drift |
| D-2 (manifest `tier` + `tier_subtype` fields) | "Manifest-Declared Service Tier" | AWS Service Health Dashboard tier classification; CNCF Landscape per-project tier metadata; GCP service tier API | "Inferred Tier" — tier inferred per-µservice rather than declared |
| D-3 (full classification table) | "Per-Service Tier Registration" | AWS Service Health Dashboard registry; GCP service catalog; Apple Framework Index | "Lazy Tier Classification" — services emerge without tier classification |
| D-4 (cross-tier dependency rules) | "Layered Service Tier DAG" | AWS Builders' Library service-layering pattern; GCP service dependency graph; Apple Framework dependency rules | "Inverted Dependency" — substrate depends on product (architectural inversion) |
| D-4.B (substrate DAG ordering) | "Foundational Dependency DAG" | AWS Builders' Library "Static stability"; GCP Borg/Omega layering (Verma et al. 2016); Apple Frameworks Reference dependency layers | "Cyclic Substrate Dependency" — substrate cycle producing chicken-egg bootstrap failure |
| D-5 (service-cell deep-dive) | "Peer-Cell Service Pattern" | AWS Marketplace + AWS Activate peer-cell pattern; Salesforce AppExchange peer-cell; Stripe peer-cell | "Forced Two-Tier" — service cells classified as substrate or product creating ambiguity |
| D-6 (reserved µservice rules) | "Build-Ahead-of-Certification" | AWS pre-launch service pattern; FedRAMP reserved namespace; Apple beta-framework pattern | "Live-Before-Certified" — uncertified service deployed live, missing regulatory gate |
| D-7 (CI lane coherence enforcement) | "Coverage-Required Tier Classification" | AWS Config conformance packs; Google SRE Workbook ch. 4 SLO coverage; Apple Xcode static analysis | "Honour-System Tier" — tier classification not enforced at CI time |
| D-8 (substrate SLO bar 99.99% min) | "Per-Tier SLO Floor" | Google SRE Workbook ch. 2 SLO composition; AWS Well-Architected v2024-Q4 Pillar 4; Microsoft Azure Well-Architected | "Uniform SLO" — single SLO across all tiers, producing substrate underprovisioning and product overprovisioning |
| D-8 (cross-tier SLO composition) | "Markov-Chain Availability Composition" | Pinheiro et al. 2007; Google SRE Workbook ch. 2; AWS Well-Architected Reliability Pillar | "Unverified Composition" — product SLO higher than substrates' composed SLO |
| D-9 (per-tier versioning policy) | "Tier-Aware Deprecation" | Google Cloud Deprecation Policy 2024 (12+ months foundational); AWS deprecation policy; Apple framework SemVer | "Uniform Deprecation Window" — single deprecation window across tiers |

---

## Appendix B: Worked example — adding a new µservice and classifying its tier

To illustrate the doctrine in operation, here is a worked example
walking through the addition of a hypothetical new µservice.

### Scenario

A team proposes a new µservice `microservices/incident-management/`
to host the SRE incident management surface — incident creation,
triage, postmortem authoring, on-call rotation state.

### Step 1 — Identify tier candidates

The author asks: who consumes this µservice?

- Primary consumers: `oyatie.security.incident-response.*` principals
  + `oyatie.platform-ops.sre.*` principals.
- Secondary consumers: ops-dashboard-control-center (product) +
  observability (substrate, for incident telemetry).

The primary consumers are oyatie internal SRE principals interacting
via an end-user surface. This is *product-tier*, *product-internal*
subtype.

### Step 2 — Verify tier classification

Apply §D-1 two-rule check:

- **Rule 1 (substrate is audience-neutral capability-focused).** Does
  incident-management provide an audience-neutral capability consumed
  by *other µservices*? Not primarily. Other µservices may emit
  incident telemetry (substrate emission path), but the consumption
  is via human SRE principals. NOT substrate.
- **Rule 2 (product is tenant-scoped surface-focused).** Does
  incident-management provide a surface for human users within a
  tenant (`oyatie.platform-ops.sre.*`)? Yes. The surface is the
  incident creation + triage + postmortem UI. Tenant-scoped (only
  oyatie SRE principals have surface access). YES product.

Tier classification: `tier: product`, `tier_subtype:
product-internal-incident-management`.

### Step 3 — Author `microservices/incident-management/manifest.json`

```yaml
microservice_id: incident-management
microservice_layer: 13-edge-product  # per ADR-0105
tier: product
tier_subtype: product-internal-incident-management
tier_classification_rationale: |
  Primary consumers are human oyatie.platform-ops.sre.* and
  oyatie.security.incident-response.* principals via a UX (incident
  list, incident detail, triage actions, postmortem editor). Surface
  is tenant-scoped (only oyatie internal principals have access via
  Cedar-permits). No other µservice depends on incident-management's
  contracts for substrate-level capability. Per ADR-0245 §D-1 Rule
  2, this is product-tier. Per §D-2 tier_subtype enum, this is
  product-internal subtype.
tier_certified_at: "2026-MM-DD"
tier_promotion_history: []
dependencies:
  substrates:
    - identity         # for Cedar principal evaluation
    - tenancy          # for tenant scope
    - audit-chain      # for incident-event emission
    - observability    # for incident telemetry ingestion
    - policy-engine    # for Cedar gates on incident actions
    - ontology         # for incident object type projection
    - workflow-engine  # for runbook execution
  products:
    - ops-dashboard-control-center  # incidents surface from here
  service-cells: []
  planned_dependencies: []
tenant_scoped: true
surfaces:
  - name: incident-list-ui
    type: web-ui
    audience: oyatie.platform-ops.sre.*
  - name: incident-detail-ui
    type: web-ui
    audience: oyatie.platform-ops.sre.* + oyatie.security.*
  - name: postmortem-editor-ui
    type: web-ui
    audience: oyatie.platform-ops.sre.* + oyatie.security.*
  - name: on-call-rotation-ui
    type: web-ui
    audience: oyatie.platform-ops.sre.*
```

### Step 4 — Author `microservices/incident-management/PRD.md` frontmatter

```yaml
---
microservice: incident-management
tier: product
tier_subtype: product-internal-incident-management
owners: [council-engineering, ops-sre-reliability]
status: Proposed
date: 2026-MM-DD
---
```

### Step 5 — Author OpenSLO manifest at product SLO floor

Product SLO floor per §D-8 is 99.9% with per-product latency
budget. For an internal SRE product:

```yaml
# microservices/incident-management/slos/availability.openslo.yaml
slos:
  - name: incident-list-ui-availability
    indicator:
      ratio_metric:
        good:
          metric_source: prometheus
          spec:
            query: sum(rate(incident_list_ui_requests_total{status="200"}[5m]))
        total:
          metric_source: prometheus
          spec:
            query: sum(rate(incident_list_ui_requests_total[5m]))
    objectives:
      - target: 0.999  # 99.9% per ADR-0245 §D-8 product floor
        window: 30d
  - name: incident-detail-ui-latency-p99
    indicator:
      ratio_metric:
        good:
          metric_source: prometheus
          spec:
            query: histogram_quantile(0.99, rate(incident_detail_ui_latency_seconds_bucket[5m])) < 0.5
        total:
          metric_source: prometheus
          spec:
            query: 1
    objectives:
      - target: 0.99
        window: 30d
```

### Step 6 — Verify cross-tier dependency direction

Run `oya gate validate cross-tier-dependency-direction` on the new
manifest. Verifies:

- All declared substrate dependencies are tier-substrate. ✓
- All declared product dependencies are tier-product (ops-dashboard-
  control-center is product). ✓ (product-to-product is permitted per
  ADR-0145 + this ADR §D-4.F)
- No reverse direction (no substrate depends on incident-management).
  ✓ (no substrate manifest declares incident-management as a
  dependency)

Result: exit 0.

### Step 7 — Author Cedar fragments

Per ADR-0243, every gate is Cedar-mediated. Cedar fragments for
incident-management actions:

- `microservices/policy-engine/fragments/baseline/incident-management-permits.cedar`
- `microservices/policy-engine/fragments/baseline/incident-management-default-deny.cedar`

### Step 8 — Multispectrum review

The new µservice's introduction is a ChangeSet per ADR-0110.
Multispectrum review v2.4.0 fan-out includes:

- F1 (correctness): does the µservice do what its PRD claims?
- F2 (hyperscaler-fitness): does the tier classification match
  hyperscaler precedent?
- F5 (security): Cedar fragments cover all surfaces?
- F6 (performance): SLO targets justified by capacity model?
- A1 (own-policy-adherence-naming): manifest_id follows BNF v4.1?
- A4 (architecture-adherence): tier respects substrate-vs-product
  doctrine?
- A6 (schema-adherence): manifest schema valid?

Verdict: APPROVE → merge.

### Step 9 — Capacity + deployment cadence inherited from tier

Once merged:

- **Capacity planning** follows product-tier defaults (per-tenant
  capacity model from FinOps portal).
- **Deployment cadence** follows product-tier defaults (faster
  than substrate; ADR-0139 SLO gates apply but with product-tier
  thresholds).
- **Observability defaults** follow product-tier defaults (per-user
  trace + per-cohort aggregation).
- **Versioning policy** follows product-tier defaults (per-product
  deprecation per PRD).
- **Compliance pack applicability** follows product-tier defaults
  (HIPAA-pack and PCI-DSS-pack are out-of-scope for internal
  incident management; SOC 2 + ISO 27001 are in-scope).

### Step 10 — Tier promotion path (if applicable)

If, later, multiple substrates begin to depend on incident-
management's incident-event emissions (which would make it
substrate-ish), a tier promotion ADR proposes moving from `product`
to `substrate-audit-aggregator` (service-cell). This requires:

- A promotion ADR.
- Refactor of incident-management's surfaces (some move to a separate
  product) so the residual is audience-neutral.
- Manifest update of `tier_promotion_history`.

This worked example illustrates that **tier classification is a
single-paragraph manifest decision, verifiable at CI, and the rest of
the µservice's operational defaults (SLO, capacity, cadence,
observability, versioning, compliance) inherit from it**. That is
the operational payoff of the doctrine.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `substrate` (manifest tier value) | N/A (manifest enum) | N/A | Manifest `tier` field value; identifies a µservice as a load-bearing platform substrate consumed by all products. Term from hyperscaler vocabulary (AWS substrate, Stripe substrate); distinct from the ADR-0105 layer enum. |
| `product` (manifest tier value) | N/A (manifest enum) | N/A | Manifest `tier` field value; identifies a user-facing product µservice. |
| `service-cell` (manifest tier value) | N/A (manifest enum) | N/A | Manifest `tier` field value; identifies a peer-to-products service cell (e.g., marketplace). Matches AWS-internal "service cell" vocabulary. |
| `reserved` (manifest tier value) | N/A (manifest enum) | N/A | Manifest `tier` field value; identifies a skeleton-without-deploy µservice awaiting certification before launch. |
| `substrate-meta` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for Foundry's transitional classification during dissolution per ADR-0247; `meta` indicates the µservice authors/modifies the platform itself. |
| `substrate-policy` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for policy-engine; `policy` describes the single-concern substrate capability. |
| `substrate-tenancy` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for tenancy µservice. |
| `substrate-identity` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for identity µservice. |
| `substrate-audit` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for audit-chain µservice. |
| `substrate-observability` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for observability µservice. |
| `substrate-secrets` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for cloud-secrets µservice. |
| `service-cell-marketplace` (tier_subtype) | N/A (manifest enum) | N/A | Tier subtype for marketplace service cell per ADR-0249. |

---

*End of ADR-0245.*
