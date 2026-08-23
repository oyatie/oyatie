---
doc_class: Onboarding-Reading-Order
doc_id: ONB-KEYSTONE-READING-ORDER
status: Published
date: 2026-05-20
owner_team: council-engineering + axis-devrel + council-architecture
audience:
  - new-hire-engineer
  - intern
  - external-contributor
  - new-agent-persona
companion_doc: /docs/onboarding/intern-day-one.md
related_adrs:
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0246-policy-engine-substrate-promotion
  - ADR-0247-self-hosting-self-modification-doctrine
  - ADR-0248-amazon-shape-cellular-architecture
  - ADR-0249-multi-category-marketplace-doctrine
  - ADR-0250-build-ahead-of-certification-doctrine
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0252-time-coordination-distributed-consistency
  - ADR-0253-network-topology-edge-service-mesh
  - ADR-0254-deployment-model-spectrum
  - ADR-0255-intelligence-as-two-layer-ai-substrate
related_specs:
  - /specs/platform-architecture.json
  - /specs/tenant-model.json
  - /specs/cedar-fragment-schema.json
  - /specs/compliance-pack-schema.json
  - /specs/cell-topology.json
  - /specs/microservice-tier-classification.json
  - /specs/deployment-models.json
keystone_bundle: 2026-05-20-foundational-doctrine
---

# Keystone Bundle Reading Order

> A guided, time-budgeted reading path through the 14 foundational
> keystone ADRs (ADR-0242 through ADR-0255), companion specs, PRDs,
> standards, user stories, and analysis docs. Intended for an engineer
> with programming experience but no prior oyatie context who needs to
> contribute production code within their first two weeks.

---

## 1. Purpose

You are reading this document first because the oyatie corpus is large
(hundreds of ADRs, dozens of specs, multiple product PRDs) and a naive
breadth-first read takes weeks and yields very little working
understanding. This guide tells you exactly which documents to read,
in what order, with what reading goal, and how to self-check that you
understood each one before moving on.

The reading order is organised into nine phases, totalling 21-32 hours
of focused reading. If you read for four hours per day, expect to
finish in one working week; if you read for two hours per day around
hands-on coding, expect two working weeks. Either is fine. The Day-One
runbook (`/docs/onboarding/intern-day-one.md`) only requires Phase 1
to be complete before you start hands-on work; the remaining phases
can interleave with coding.

The goals of this reading path are, in order of priority:

1. **Vocabulary.** You should be able to use the words `tenant`,
   `substrate`, `product`, `cell`, `compliance pack`, `Cedar fragment`,
   and `regional pack` in conversation without ambiguity. The keystone
   bundle re-defines several of these terms relative to common
   industry usage, and you cannot read older ADRs accurately without
   the new definitions.
2. **Mental model.** You should be able to draw, on a whiteboard, the
   four-tier cellular topology (Tier 0 / Tier 1 / Tier 2 / Tier 3 plus
   peer service cells and reserved Tier 4), the substrate-vs-product
   layering, and the policy evaluation flow (call site -> Cedar
   evaluator -> permit/forbid -> audit emission).
3. **Doctrine.** You should be able to answer "why does oyatie have
   a policy-engine substrate µservice and not just a library?" without
   hesitation.
4. **Working knowledge.** You should be able to navigate the repository
   confidently: find the right ADR for a given subject, locate the
   spec that drives a given CI lane, and identify which µservice owns
   a given capability.

This document is itself a Phase-0 read. Allocate 15 minutes for it,
then proceed to Phase 1.

---

## 2. Prerequisites

Before you start Phase 1 you should be comfortable with the following.
If any item is unfamiliar, spend an hour or two on it first; you will
not get value out of the keystone reading without these.

### 2.1 Programming and tooling

- **Rust toolchain.** You can install Rust via rustup, run
  `cargo check`, `cargo test`, and `cargo fmt`. You understand
  workspaces and crates. You do not need to be an expert; intermediate
  is fine.
- **Command line.** You are comfortable in a Unix shell. You know how
  to pipe, redirect, and run background jobs.
- **Git.** You know how to clone, branch, commit, push, rebase, and
  open a pull request. You know what a fast-forward merge is.
- **Containers and Kubernetes basics.** You have run `docker` and
  `kubectl` at least once. You know what a Pod, Deployment, Service,
  and Namespace are. You do not need to know operators or admission
  controllers yet.
- **JSON and YAML.** You can read both fluently and write valid
  documents in each.

### 2.2 Architectural concepts

- **Microservices.** You understand why services are split, what a
  bounded context is in DDD terms, and roughly what a saga is.
- **Authentication vs authorization.** You can articulate the
  difference between proving who you are (authn) and proving what you
  may do (authz).
- **Public-key cryptography.** You understand the difference between
  a signing key and an encryption key, and roughly what a certificate
  authority does.
- **Multi-tenancy.** You have at least heard the terms shared-nothing,
  per-tenant database, and bring-your-own-key.

### 2.3 Concepts you do not need yet

You do not need to know Cedar, MLS, SPIFFE, Cilium, Kyverno, OPA,
HLC clocks, WASM, Tantivy, ClickHouse, or any oyatie-specific tool
before reading. The keystone bundle introduces or defines all of
these in context.

### 2.4 Sources you should keep open

- The CLAUDE.md at the repo root.
- The installed agent-runtime skills/roles that apply to your agent surface.
- The repo `Cargo.toml` workspace manifest so you can see the crate
  list at a glance.
- The `microservices/` directory tree open in a separate terminal so
  you can ground each ADR in the actual code layout.

---

## 3. Phase 1: Foundational Doctrine (4-6 hours)

This is the load-bearing phase. The four ADRs in this phase introduce
the doctrines on which every other ADR rests. You cannot skip ahead
without re-doing this work later. Read them in the order given;
they are written assuming each one comes before the next.

### 3.1 ADR-0242 — oyatie is a tenant (45-90 minutes)

File: `/docs/decisions/ADR-0702-identity-authz-live-apex.md`

**Read the frontmatter carefully.** Note that this ADR amends
ADR-0136, ADR-0136-amendment, ADR-0220, ADR-0239, and ADR-0221.
Note that the `keystone_position` field is `1-of-14` and the
`keystone_bundle` value is `2026-05-20-foundational-doctrine`.

**Read the Status and Date sections** in full. You will encounter
the phrase `advisory-until-tenant-bootstrap-lands` repeatedly. This
is the keystone bundle's pattern: doctrines are accepted in text now,
but enforcement promotes to BLOCKER status only after the underlying
substrate µservice lands. This is how oyatie avoids the
"big-bang refactor" trap.

**Read the Context section** to understand the prior portfolio state.
The phrase `audience-of-microservice` is what this ADR is killing.
Before ADR-0242 the portfolio treated `intelligence` as a "consumer
µservice" and `foundry` as an "internal µservice"; ADR-0242 deletes
that distinction.

**Read the Decision section** in full. Pay attention to: reserved
namespaces (`oyatie`, `oya`, `oyat`, `oyati`), audit-stream parity
(no special path for the `oyatie` tenant), DSAR cascade applies to
oyatie itself, FinOps cost-center applies to oyatie itself.

**Skim the Consequences section** for now; you will revisit it after
Phase 8 when you read the spec that operationalises it.

What you will know after reading: you will be able to explain why
`oyatie` is a row in the `tenancy` µservice's tenants table; why
there is no special carve-out path for internal workloads; why every
hot-path code change is exposed to the same compliance machinery as
customer-facing code; and how an engineer's local sandbox tenant
(`oyatie.dev.<engineer-id>`) inherits the doctrine.

### 3.2 ADR-0243 — Cedar as Universal Gate (45-90 minutes)

File: `/docs/decisions/ADR-0700-ci-admission-live-apex.md`

**Read the frontmatter.** Note the `keystone_position: 2-of-14`
and the `amends:` field listing ADR-0150 and ADR-0183. ADR-0150
introduced Cedar as an authorization-only engine; ADR-0183 separated
Cedar (app-layer authz) from Kyverno (admission-layer admission).
ADR-0243 extends Cedar's scope from "authorization" to "every gate".

**Read the Status section** and note that enforcement promotes only
after the policy-engine substrate lands (ADR-0246).

**Read the Context section.** The key insight: prior to this ADR,
policy decisions were scattered across code — feature flags lived in
LaunchDarkly, retention sunsets lived in cron jobs, audit-stream
selection lived in switch statements. Each of those was a place
where business logic and policy logic mixed. ADR-0243 says: pull
all of them out of code and into Cedar fragments.

**Read the Decision section.** Note the gate catalog in §D-3 (the
13 minimum-required gate categories): authorization, tenant scope,
data class, jurisdiction overlay, compliance pack, reserved
namespace, audit emission, cost attribution, feature activation,
rate limit/quota, cross-cell traffic, provider-credential BYOK eligibility (ADR-0255 §D-4), DSAR cascade.

**Read the Migration section** to understand which categories of
in-code policy decisions are being migrated to Cedar (provider
routing, cell routing, tax routing, audit-stream selection, etc.).

What you will know after reading: you will be able to explain what
"code never decides policy; code asks the policy engine and acts on
the answer" means in concrete terms; you will know why feature flags
are not stored in LaunchDarkly; you will be able to identify the
13 gate categories and look up which Cedar fragments cover them.

### 3.3 ADR-0244 — Tenant as Universal Scoping Primitive (45-90 minutes)

File: `/docs/decisions/ADR-0702-identity-authz-live-apex.md`

**Read the frontmatter.** Note `keystone_position: 3-of-14` and the
amends field. This ADR finalises the audience-as-tenant-property
framing that ADR-0242 introduced.

**Read the Decision section** for the dotted-hierarchical sub-scope
convention. Examples: `acme`, `acme.marketing`, `acme.marketing.q4`,
`oyatie.dev.alice`. Note the `max_sub_scope_depth: 4` invariant.
Note that a principal carries exactly one sub-scope; cross-scope
actions require an assume-role flow.

**Read the Inheritance and Rollup section.** This is where the
substrate composes: jurisdiction inherits from parent, audit streams
roll up to parent, cost attribution rolls up to parent. This is what
lets a sandbox tenant `oyatie.dev.alice` automatically inherit the
oyatie compliance posture without per-tenant configuration.

**Read the Reserved Namespaces section.** This is what prevents
a customer from registering `corp` as their org tenant.

**Read the Audience Types section.** The eight audience types
(PLATFORM_OWNER, B2B_TENANT, B2C_CONSUMER, DEVELOPER, SANDBOX,
PREVIEW, PARTNER_AGENCY, RESELLER) are the canonical enum. Audience
is a property of the tenant, not the µservice.

What you will know after reading: you will be able to design a
tenant hierarchy for any product; you will know how to construct
a sub-scope path that respects the depth limit; you will understand
why a B2B tenant's audit streams roll up while their feature flags
inherit; you will be able to spot a misuse of "audience-as-microservice"
in any older ADR or doc.

### 3.4 ADR-0245 — Substrate vs Product Layering (60-120 minutes)

File: `/docs/decisions/ADR-0701-monorepo-capability-live-apex.md`

**Read the frontmatter.** Note `keystone_position: 4-of-14`. Note
the amends list (ADR-0131 layout authority, ADR-0132 forward policy,
ADR-0145 communication reform). Note the related-spec list includes
`microservice-tier-classification.json`.

**Read the Decision section, section D-1 (Four Tiers).** The four
tiers are: `substrate` (audience-neutral capability), `product`
(tenant-scoped surface), `service-cell` (dedicated-function peer
cell), and `reserved` (placeholder for a future capability). Memorise
this enum.

**Read D-2 (Dependency Direction Rules).** Products call substrates;
substrates do not call products. Service cells are peer cells with
their own deploy cadence. Reserved µservices declare intent without
shipping workloads (so the registry knows the slot is taken).

**Read D-3 (Classification Authority).** The tier is a manifest
field. CI validates the tier. Tier transitions require a
multispectrum review.

**Read the Examples section.** `messenger` is classified as a
product (product-consumer-messenger); `tenancy`, `identity`,
`policy-engine`, `audit-chain`, `workflow-engine` are substrates;
`marketplace-substrates` is a substrate cluster; `community` is
a product.

What you will know after reading: you will be able to classify any
µservice into one of the four tiers and justify the classification;
you will know why `policy-engine` was promoted out of the `ontology`
µservice (it is a substrate, not an Ontology BC); you will be able
to identify dependency-direction violations when you see them.

### 3.5 Phase 1 self-check

Before moving on, write down (do not look) answers to these:

- Q1: Why does `oyatie` need to be a row in the tenants table?
- Q2: Why is Cedar's scope wider than "authorization"?
- Q3: What does `acme.marketing.q4` decompose to in terms of tenant
  and sub-scopes, and what depth is it?
- Q4: What are the four µservice tiers and what are the dependency
  direction rules between them?

If you cannot answer any of these confidently, re-read the relevant
ADR. The remaining phases assume you can.

---

## 4. Phase 2: Infrastructure (3-5 hours)

The infrastructure phase covers how workloads run, how they talk to
each other on the wire, and how policy is enforced at the cell
boundary.

### 4.1 ADR-0248 — Amazon-shape Cellular Architecture (60-90 minutes)

File: `/docs/decisions/ADR-0700-ci-admission-live-apex.md`

**Read the frontmatter and Status.** This is keystone 7-of-14.

**Read the Decision section, D-1 (Tier Model).** The four-tier
model: Tier 0 (external dependencies), Tier 1 (bootstrap cell),
Tier 2 (control plane cells), Tier 3 (data plane cells). Plus peer
service cells (marketplace, dev-tools, audit-aggregator, analytics).
Plus Tier 4 reserved for post-certification financial-grade and
fulfillment-grade workloads.

**Read D-2 (Cell Internals).** Each cell is a K8s cluster
(everything is K8s except the edge POP layer). Each cell carries
its own copy of the policy-engine evaluator (`per_cell_evaluator`),
its own Valkey cache for fragment hot-reload, and its own DR tier
declaration.

**Read D-3 (Shuffle Sharding).** Two-shard placement of tenants
across data-plane cells; a poison-pill tenant only contaminates
two cells, not all of them. The static stability invariant: each
cell must be sized to operate without the control plane for a
window long enough to tolerate cross-region failure.

**Read D-4 (Sandboxing).** Cloud Hypervisor + Kata Containers for
workload-vs-host isolation; Cilium ambient mesh inside cells for
workload-to-workload identity-aware encryption.

**Read the Acceptance Tests section.** This is the test catalogue
that the cell substrate must pass before promotion.

What you will know after reading: you will be able to draw the
four-tier topology; you will know what shuffle sharding means and
why it is two-shard; you will be able to identify which µservice
runs in which cell tier.

### 4.2 ADR-0253 — Network topology (45-60 minutes)

File: `/docs/decisions/ADR-0708-platform-foundations-live-apex.md`

**Read the frontmatter and Status.** This is keystone 12-of-14.

**Read the Decision section.** Five layers: planetary apex DNS
(Anycast + GeoDNS), edge POPs (Cloudflare Workers today; Pingora
self-hosted by Year 3+), per-cell ingress termination, intra-cell
Cilium ambient mesh, inter-cell mesh.

**Read the Identity section.** Workload identity via SPIFFE/SPIRE.
Every workload has an SVID; every workload-to-workload call is mTLS.

**Read the Crypto section.** TLS 1.3 only, post-quantum hybrid key
exchange, HTTP/3/QUIC client-side.

What you will know after reading: you will be able to trace a
client request from DNS through the edge POP to the cell ingress
to the µservice; you will know what SPIFFE SVID is and why every
workload carries one.

### 4.3 ADR-0254 — Deployment model spectrum (45-60 minutes)

File: `/docs/decisions/ADR-0709-general-live-apex.md`

**Read the frontmatter and Status.** This is keystone 13-of-14.

**Read the Decision section.** The five deployment models:
shared-cloud, dedicated-cloud, hybrid/BYO-cloud, on-prem connected,
on-prem air-gapped. All five ship the same Helm charts, the same
Cedar policy bundles, the same container images. The substrate
beneath the cell varies; the cell contents do not.

**Read the Deployment Control Plane section.** The new
`microservices/deployment-control-plane/` µservice (Palantir Apollo
equivalent) orchestrates upgrades, canary, rollback, and air-gapped
bundle delivery.

What you will know after reading: you will be able to articulate
the five deployment models; you will understand why the same code
shipped to a SaaS cell must also ship to an air-gapped cell with no
behavioural differences.

### 4.4 ADR-0246 — Policy-Engine Substrate Promotion (60-75 minutes)

File: `/docs/decisions/ADR-0701-monorepo-capability-live-apex.md`

**Read the frontmatter and Status.** This is keystone 5-of-14.

**Read the Context section.** This ADR is the operational complement
to ADR-0243 (Cedar as Universal Gate). ADR-0243 says "Cedar is the
gate"; ADR-0246 says "Cedar evaluation runs in its own peer substrate
µservice, not as a BC inside Ontology".

**Read the Decision section.** The Cedar evaluator is its own
µservice. Each cell runs at least three evaluator replicas. The
in-cell cache is Valkey with a 1-second TTL. The static-stability
fallback is a 30-second cache plus default-deny.

**Read the Fragment Lifecycle section.** The eight states:
`authored -> reviewed -> signed -> published -> activated ->
in-force -> sunset -> tombstoned`. Each transition requires
specific multispectrum-review facets.

**Read the Performance Targets section.** p50 0.1ms, p99 1ms,
p999 5ms on the hot path. These are the targets the evaluator
substrate must meet for promotion.

What you will know after reading: you will be able to deploy and
configure a per-cell policy evaluator; you will know what happens
when the upstream policy authority is unavailable (static-stability
fallback with default-deny); you will understand the fragment
signing chain (org root key -> intermediate keys -> fragment).

### 4.5 Phase 2 self-check

- Q5: What is shuffle sharding and why is the shard count two?
- Q6: What is a SPIFFE SVID and why does every workload have one?
- Q7: What are the five deployment models?
- Q8: What is the policy evaluator's static-stability fallback
  behaviour?

---

## 5. Phase 3: Specialty Doctrines (3-4 hours)

The specialty phase covers three load-bearing doctrines that each
deserve their own keystone: self-modification, time/consistency,
and the Intelligence substrate.

### 5.1 ADR-0247 — Self-Hosting / Self-Modification (60-90 minutes)

File: `/docs/decisions/ADR-0709-general-live-apex.md

**Read the frontmatter and Status.** This is keystone 6-of-14.

**Read the Decision section.** The Foundry-as-a-µservice framing
(ADR-0136 + amendments) dissolves. Foundry's six bounded contexts
redistribute to Workflow Engine (orchestration), Intelligence
(model invocation, eval, guardrails), audit-chain (CI evidence),
and policy-engine (gate fragments). The internal-CI capability
becomes a named bundle of workflow definitions running in
`dev-tools-cell-N`.

**Read the Bootstrap Tier section.** Tier 0 is external (hardware,
DNS, git host, container registry). Everything above Tier 0 self-
hosts. The platform can rebuild itself from Tier 0 plus a fresh
container registry.

**Read the Self-Modification Cedar section.** Cedar fragments gate
which workflows are permitted to modify which production resources.
Self-modification is permit-by-default for sandbox tenants and
permit-by-explicit-fragment for the oyatie tenant.

What you will know after reading: you will be able to explain why
Foundry is not a µservice but a workflow library; you will know
what "Tier 0 minimum bootstrap" means; you will be able to identify
which workflows belong in `dev-tools-cell` versus a tenant cell.

### 5.2 ADR-0252 — Time, Coordination, Distributed Consistency (60-90 minutes)

File: `/docs/decisions/ADR-0709-general-live-apex.md`

**Read the frontmatter and Status.** This is keystone 11-of-14.

**Read the Decision section.** Hybrid Logical Clocks (HLC) as the
default clock primitive. TrueTime-style atomic-clock-backed clocks
reserved for Tier-4 financial-grade and IL5+ classified cells.
Workflow Engine sagas for cross-microservice coordination (no
distributed locks). Caller-supplied idempotency keys (Stripe
pattern) for retry safety.

**Read the Locks-as-Anti-Pattern section.** Distributed locks are
forbidden outside narrow exceptions (D-5 enumerates them). Code
must not depend on a distributed lock for correctness.

**Read the Leap Second section.** Smear the leap second (Google
approach). Code must not assume monotonicity across a smear window.

**Read the Idempotency-Key Format section.** Caller-supplied,
opaque to the platform, scoped per (tenant, action, hour).

What you will know after reading: you will know what HLC is and
why oyatie uses it; you will know how to add an idempotency key to
a request; you will know why locks are forbidden and what to use
instead (saga compensation).

### 5.3 ADR-0255 — Intelligence as Two-Layer Substrate (60-90 minutes)

File: `/docs/decisions/ADR-0701-monorepo-capability-live-apex.md`

**Read the frontmatter and Status.** This is keystone 14-of-14
(the final ADR in the bundle).

**Read the Decision section.** Intelligence is two layers: (a) an
audience-neutral AI substrate that serves every tenant including
`oyatie` itself, and (b) a consumer brand surface that renders the
"oyatie intelligence" brand UX. The first is a substrate; the
second is a product.

**Read the provider-credential BYOK section (ADR-0255 §D-4).** The substrate owns zero credentials.
Every credential is a SecretReference with an explicit owner. A
tenant can bring its own OpenAI key, its own Anthropic key, its own
fine-tuned model weights, its own embedding provider.

**Read the Multi-Modal section.** Text, image, audio, video, code
are day-one. No staged rollout per modality.

**Read the Foundry-BC-Absorption section.** Foundry's `providers`,
`guardrails`, `eval` BCs land inside Intelligence. The
`embeddings` and `fine-tuning` BCs promote to their own peer
substrate µservices.

What you will know after reading: you will know why Intelligence
is two layers and where the seam is; you will be able to wire a
tenant-supplied OpenAI key into the substrate; you will be able to
classify any AI-related capability as substrate, product, or seam.

### 5.4 Phase 3 self-check

- Q9: Where does Foundry's six-BC dissolution land?
- Q10: Why are distributed locks forbidden?
- Q11: What does provider-credential BYOK mean for Intelligence credentials (ADR-0255 §D-4)?

---

## 6. Phase 4: Compliance + Certification (2-3 hours)

The compliance phase covers how oyatie ships compliance machinery as
a first-class platform feature rather than as a deferred bolt-on.

### 6.1 ADR-0250 — Build-Ahead-of-Certification (60-90 minutes)

File: `/docs/decisions/ADR-0709-general-live-apex.md`

**Read the frontmatter and Status.** This is keystone 9-of-14.

**Read the Decision section.** Every certification-gated capability
is architected and built day-one and launched per-market only after
regulatory clearance lands. Build precedes certification.
Certifications drop on working systems rather than triggering
build-from-zero.

**Read the Three-State Lifecycle section.** Capability states:
`built-not-launched`, `launched-in-jurisdiction-X`,
`sunset-in-jurisdiction-X`. Each state has its own Cedar gate.

**Read the Anti-Bypass section.** A `built-not-launched` capability
must still pass through Cedar; the gate denies all non-oyatie
tenants. This prevents a customer-facing feature from leaking before
launch.

**Read the Examples section.** Apple Pay per-country rollout, Stripe
geographic expansion, AWS regional service-availability — oyatie
follows the same pattern.

What you will know after reading: you will know why a half-built
healthcare capability is acceptable in `built-not-launched` state
but never reaches a customer; you will know how to bring a
capability through the three-state lifecycle.

### 6.2 ADR-0251 — Compliance Pack and Cell Certification Levels (60-90 minutes)

File: `/docs/decisions/ADR-0708-platform-foundations-live-apex.md`

**Read the frontmatter and Status.** This is keystone 10-of-14.

**Read the Decision section.** A Compliance Pack is a first-class
versioned signed bundle that wraps a single regulation (HIPAA, PCI
DSS, FedRAMP, EU GDPR, KR-PIPA, KR-FSS, DoD IL5/6, FERPA, FDA 21
CFR Part 11, EU AI Act, EU NIS2, EU DSA, JP APPI, SG PDPA, AU
Privacy Act, KSA NDMO/SDAIA). Cells declare a SET of certifications
enumerating which packs they can host. Tenants install packs onto
themselves.

**Read the Aggregation Semantics section.** At evaluation time the
policy engine aggregates installed packs; deny-wins. Cross-pack
traffic is Cedar-gated.

**Read the Pack Schema section.** A pack contains: Cedar fragments,
data-class extensions, retention floors, BAA/DPA templates, breach-
notification workflow definitions, evidence-emission rules,
encryption requirements.

What you will know after reading: you will be able to author a
new compliance pack; you will know why a tenant cannot install a
pack onto a cell that does not certify for that pack; you will
understand deny-wins aggregation.

### 6.3 Phase 4 self-check

- Q12: What is the three-state lifecycle for a certification-gated
  capability?
- Q13: What does a Compliance Pack contain?
- Q14: What does deny-wins mean across multiple installed packs?

---

## 7. Phase 5: Product Surfaces (2-3 hours)

The product phase covers the ADR that defines the marketplace plus
the four anchor PRDs that show how products consume the substrates.

### 7.1 ADR-0249 — Multi-Category Marketplace Doctrine (45-60 minutes)

File: `/docs/decisions/ADR-0705-product-protocol-live-apex.md`

**Read the frontmatter and Status.** This is keystone 8-of-14.

**Read the Decision section.** Marketplace is a unified multi-
category commerce surface (Amazon retail + Facebook Marketplace +
Apple App Store + Upwork + Substack). Eight shared substrate
µservices: catalog, inventory, orders, fulfillment, reviews,
discovery, pricing, trust-safety. Four category-specific bounded
contexts: physical-goods, c2c, services, subscriptions.

**Read the Category Rollout section.** Categories ship per
ADR-0250's three-state lifecycle. Plugin-app-store is the first
category to ship (it has no physical-fulfillment dependency).

What you will know after reading: you will be able to map any
marketplace feature to one of the eight substrate µservices; you
will know why oyatie does not have a separate `plugin-app-store`
commerce stack but instead refactors onto the marketplace
substrates.

### 7.2 PRD: Messenger (30-45 minutes)

File: `/microservices/messenger/PRD.md`

**Read sections 1-3.** Messenger is a hero product with two
surfaces: Personal Messenger (B2C) targeting Signal/Telegram/
KakaoTalk/Line/WhatsApp/Discord; Work Messenger (B2B) targeting
Slack/Teams/Naver Works. Both share the same substrate.

**Note the dual-context isolation invariant.** A user's personal
DMs are structurally invisible to any org admin, even when both
contexts share a physical cluster.

**Note the MLS, Matrix, LiveKit choices.** MLS for E2E group keys,
Matrix for federation, LiveKit for huddles.

### 7.3 PRD: Mail (30-45 minutes)

File: `/microservices/mail/PRD.md`

**Read sections 1-3.** Mail is a hero product with two surfaces:
Personal Mail (B2C) targeting Gmail/Outlook/Hey/Superhuman; Work
Mail (B2B) targeting Microsoft 365/Google Workspace. SMTP, IMAP4rev2,
JMAP, REST at the edge.

**Note the feature-matrix table in §3.** Every row is a comparator
column with a `Y` / `P` / `N` rating; the oyatie target column shows
where oyatie matches, partials, or exceeds the comparator.

### 7.4 PRD: Community (30-45 minutes)

File: `/microservices/community/PRD.md`

**Read sections 1-3.** Community is a tenant-scoped product that
provides Discord-shape real-time channels, Reddit-shape interest
aggregation, Stack Overflow-shape voted Q&A, Notion/Confluence-shape
KB articles, GitHub Discussions, Mastodon/Lemmy federation.

**Note the tier classification.** Community is a product, not a
substrate. It calls many substrates and serves none.

### 7.5 PRD: Workplace Integration (30-45 minutes)

File: `/docs/products/workplace-integration/PRD.md`

**Read sections 1-3.** Workplace Integration is a cross-cutting
product layer (not a single µservice) composed from Mail, Messenger,
Calendar, Meet, Drive, Notes, Tasks, Forms, Workflow Studio,
Workflow Engine, HR-reserved, Payroll-reserved, Plugin App Store,
Sites, Recordings, Sheets, Slides, Docs, Comms-Email, Audit-Chain,
Tenancy, Identity, Ontology, Intelligence.

**Note the comparator list.** Microsoft 365, Google Workspace,
Notion, Slack, ServiceNow, Workday, Concur, DocuSign, BambooHR,
Expensify, Greenhouse, Calendly — Workplace Integration competes
with all of these as a single coherent product.

### 7.6 Phase 5 self-check

- Q15: What are the eight marketplace substrate µservices?
- Q16: Why is Community a product and not a substrate?
- Q17: What does the dual-context invariant mean for Messenger?

---

## 8. Phase 6: Standards (3-4 hours)

The standards phase covers cross-cutting standards documents that
constrain how every product is built. These are not ADRs; they
are normative reference docs that ADRs cite.

### 8.1 UX best practices (60-90 minutes)

Search: `docs/standards/ux-*.md` and `docs/standards/design-system-*.md`.

**Read the keyboard-first interaction guidelines.** Every primary
flow must be operable with the keyboard.

**Read the accessibility guidelines.** WCAG 2.2 AA minimum; AAA on
hero products.

**Read the localisation guidelines.** Every user-facing string is
keyed; the canonical-base is English; jurisdictional overlays
provide localised strings. No hard-coded strings.

### 8.2 MLS end-to-end encryption (45-60 minutes)

File: `docs/standards/mls-rfc-9420.md` or similar.

**Understand the MLS group-key agreement model.** Every group has a
key tree; key updates rotate the tree; member-add and member-remove
trigger automatic rekey.

**Understand the device fan-out model.** Each user can have multiple
devices; each device holds its own leaf in the tree.

### 8.3 Voice and video (45-60 minutes)

File: `docs/standards/voice-video-livekit.md` or `docs/standards/webrtc-baseline.md`.

**Understand the LiveKit SFU model.** Each call has a room; each
participant publishes their audio + video to the SFU; the SFU
forwards to subscribers. SRTP + Opus + AV1 at the wire.

**Understand the ICE/STUN/TURN flow.** ICE candidate gathering;
STUN for NAT traversal; TURN for fallback relay.

### 8.4 Emoji and sticker standards (30-45 minutes)

File: `docs/standards/emoji-unicode-15.md` or similar.

**Understand the canonical emoji set.** Unicode 15.1 baseline plus
custom-emoji extension per tenant. Stickers are Lottie + WebP +
AVIF; never proprietary formats.

### 8.5 Phase 6 self-check

- Q18: What WCAG level is required on hero products?
- Q19: What protocol does MLS use for key updates after member
  changes?
- Q20: What codecs are canonical for voice and video?

---

## 9. Phase 7: User Stories (2-3 hours)

The user-story phase covers B2C and B2B compendia. These are
narrative documents showing how a real user flows through the
platform, end-to-end. They are written by council-product and
exist to ground every ADR and PRD in a concrete user journey.

### 9.1 B2C compendium (60-90 minutes)

Search: `docs/products/*/user-stories-b2c-*.md` or
`docs/user-stories/b2c-compendium.md`.

Read at least three stories end-to-end:

- A new consumer signs up for Personal Messenger, adds a friend
  by phone number, sends a message, and the message is MLS-
  encrypted, signed, audit-chained, and delivered through the
  consumer-tenant cell.
- A consumer creates a personal mailbox at `alice@oyatie.app`,
  sets up a custom domain, imports messages from Gmail, and
  configures anti-tracking.
- A consumer browses the marketplace, buys a c2c second-hand
  item, pays via the platform's facilitator merchant flow, and
  receives shipping confirmation.

### 9.2 B2B compendium (60-90 minutes)

Search: `docs/products/*/user-stories-b2b-*.md` or
`docs/user-stories/b2b-compendium.md`.

Read at least three stories end-to-end:

- An Acme tenant admin onboards 200 employees via SCIM,
  configures SAML SSO, installs the HIPAA pack, configures the
  per-region cell pinning, and rolls out Work Messenger.
- An Acme employee receives a Work Mail message containing an
  action card; clicks "Approve"; the click triggers a Workflow
  Engine saga that records the approval to the audit chain,
  notifies the requester in Work Messenger, and updates the
  Ontology object.
- An Acme tenant admin runs a DSAR (data-subject-access-request)
  on an ex-employee; the DSAR cascade pulls data from every
  µservice that holds personal data for that user; the response
  bundle is signed, retention-tagged, and delivered.

### 9.3 Phase 7 self-check

- Self-quiz: pick one story and try to write down which
  substrate µservices it calls, in what order.

---

## 10. Phase 8: Specs (1-2 hours)

The specs phase covers four machine-readable specs that
operationalise the doctrines you read in Phases 1-4. Specs are
JSON; they are the authoritative source for CI lanes, manifest
validators, and scaffolders.

### 10.1 `platform-architecture.json` (30-45 minutes)

File: `/specs/platform-architecture.json`

**Read the `_meta` and `version` fields.**

**Read the `keystone_adr_bundle` array.** This lists the 14 ADRs.

**Read the `platform.tenancy` section.** This operationalises
ADR-0242 and ADR-0244 — reserved namespaces, sub-scope rules,
audience types, the canonical oyatie tenant row, ephemeral tenant
classes.

**Read the `platform.policy` section.** This operationalises
ADR-0243 and ADR-0246 — the Cedar engine version, the in-cell
cache, the fragment lifecycle states, the signing chain, the
minimum-required gates list.

### 10.2 `tenant-model.json` (15-30 minutes)

File: `/specs/tenant-model.json`

Read the schema of a tenant row. Note the fields: tenant_id,
audience_type, parent_tenant_id, jurisdiction_primary,
data_residency_allowed, sovereign_cloud_pack, finops_cost_center,
merchant_status, payout_method, dsar_response_sla_days,
audit_streams, locked.

### 10.3 `cedar-fragment-schema.json` (15-30 minutes)

File: `/specs/cedar-fragment-schema.json`

Read the schema of a fragment. Note the fields: id, scope, owner,
signature, version, depends_on, gate_category, default_decision.

### 10.4 `compliance-pack-schema.json` (15-30 minutes)

File: `/specs/compliance-pack-schema.json`

Read the schema of a pack. Note the fields: pack_id, regulation,
cedar_fragments, data_class_extensions, retention_floors,
breach_notification_workflows, evidence_emission_rules.

### 10.5 Phase 8 self-check

- Find one CI lane that is driven by `platform-architecture.json`.
- Find one CI lane that is driven by `cedar-fragment-schema.json`.

---

## 11. Phase 9: Analysis (1-2 hours)

The analysis phase covers the meta-documents that connect the
keystone bundle to industry patterns and surface known gaps.

### 11.1 Hyperscaler pattern attribution (45-60 minutes)

File: `/docs/architecture/hyperscaler-pattern-attribution.md`

**Read the introductory section.** This document maps each oyatie
doctrine to its industry origin: AWS cell-based architecture, AWS
shuffle sharding, Stripe idempotency keys, Stripe expansion shape,
Google smear leap-second, Apple per-country rollout, Palantir Apollo,
Palantir Ontology.

**Read the per-pattern attribution table.** For each oyatie
doctrine, the corresponding industry pattern is cited.

### 11.2 Keystone bundle audit report (30-45 minutes)

File: `/docs/architecture/keystone-bundle-audit-report.md`

Read the multispectrum review findings. This is the F1..F11 + M1+M2
+ A1..A7 facet review of the 14-ADR bundle. Note any open issues.

### 11.3 Idea-refine deep-dive (15-30 minutes)

File: `/docs/architecture/keystone-bundle-idea-refine-deep-dive.md`

Read the structured divergent-convergent thinking that produced
the bundle. This is the source-of-decisions document; if you ever
need to justify why a decision was made, this is where to look.

---

## 12. Total reading time

Phase | Topic                              | Time (hours)
------|------------------------------------|-------------
0     | This document                      | 0.25
1     | Foundational doctrine              | 4-6
2     | Infrastructure                     | 3-5
3     | Specialty doctrines                | 3-4
4     | Compliance + certification         | 2-3
5     | Product surfaces                   | 2-3
6     | Standards                          | 3-4
7     | User stories                       | 2-3
8     | Specs                              | 1-2
9     | Analysis                           | 1-2
      | **Total**                          | **21-32**

Allocate 4-6 hours per day for one working week, or 2 hours per day
for two weeks. Phase 1 must be complete before you start hands-on
work; the rest can interleave with coding.

---

## 13. Self-check quiz

After completing all nine phases, you should be able to answer all
20 of these questions confidently. If you cannot, re-read the
relevant ADR or spec. Do not look at the answers in §14 until you
have written down your own.

### 13.1 Doctrinal questions

1. Why is `oyatie` a row in the tenants table?
2. What is the difference between a substrate and a product?
3. What are the four µservice tiers?
4. What does "code never decides policy" mean?
5. What is a sub-scope and what is the maximum depth?
6. What are the eight audience types?
7. What are the 13 minimum-required Cedar gate categories?
8. What does the policy evaluator do when it cannot reach the
   upstream policy authority?
9. What does provider-credential BYOK mean for credentials (ADR-0255 §D-4)?
10. What is shuffle sharding and why is it two-shard?

### 13.2 Operational questions

11. What is the three-state lifecycle for a certification-gated
    capability?
12. What is a Compliance Pack, and what does it contain?
13. What is HLC, and why does oyatie use it instead of wall-clock
    timestamps?
14. What is an idempotency key, and who supplies it?
15. What are the five deployment models?
16. What is the Deployment Control Plane µservice, and what is its
    industry analog?
17. What is a SPIFFE SVID, and which workloads have one?
18. What is the difference between Cedar (app-tier authz) and
    Kyverno (admission-tier)?
19. What does the static-stability invariant mean for a cell?
20. What is the Tier-0 bootstrap minimum, and why does it matter?

### 13.3 Quiz answer keys

The answer keys live in `/docs/onboarding/keystone-quiz-answers.md`.
Do not look until you have written your own answers down.

---

## 14. Common confusions

This section lists the top 10 confusing points new readers
report, and clarifies each. If something feels confusing in the
reading, check here first.

### 14.1 "Why is the policy engine its own µservice and not a library?"

Cedar is the universal gate. Every state-changing action consults
it. A library would mean every µservice carries its own copy of
the Cedar evaluator, its own cache, its own fragment loader, its
own signing chain — and a security-critical update would mean
rebuilding every µservice. A peer substrate µservice with per-cell
deployment lets us hot-reload fragments, rotate signing keys, and
update the evaluator without touching the products that consume it.
ADR-0246 details the reasoning.

### 14.2 "Is `oyatie` a tenant or the platform?"

Both. `oyatie` is the platform's name, AND `oyatie` is one tenant
among many on the platform. Every workload — including the workload
that runs the Foundry CI agent — lives under a tenant. The
oyatie company's own workloads live under the `oyatie` tenant,
following the same compliance machinery as any customer tenant.

### 14.3 "What is the difference between a regional pack and a compliance pack?"

A regional pack (ADR-0010) is a geographic+legal jurisdiction
bundle — Korea's regional pack carries KR-PIPA + KR-FSS + KR
sovereign cloud provider preferences. A compliance pack (ADR-0251)
is a single-regulation bundle — HIPAA is a pack, PCI DSS is a pack,
KR-PIPA is a pack. A regional pack can include several compliance
packs; a compliance pack is global and is installed by tenants
across many regions.

### 14.4 "Is Foundry a µservice?"

No, not anymore. ADR-0247 dissolved the Foundry-as-µservice framing
(ADR-0136 + amendments). Foundry's six BCs redistributed across
Workflow Engine, Intelligence, audit-chain, and policy-engine. The
internal-CI capability is now a named bundle of workflow
definitions + Cedar fragments + eval criteria running inside
`dev-tools-cell-N`.

### 14.5 "If audience is a tenant property, why do PRDs still say `audience_modes: [B2C-personal, B2B-work]`?"

Because the µservice serves multiple audiences (e.g., Messenger
serves personal users AND work tenants), and the PRD documents
that. The audience itself is carried on the tenant row, not the
µservice manifest. Older PRDs may carry stale `audience` fields on
the manifest; those are migrating per the keystone bundle.

### 14.6 "Is the marketplace one µservice or eight?"

Eight substrate µservices plus four category-specific BCs. The
substrates: catalog, inventory, orders, fulfillment, reviews,
discovery, pricing, trust-safety. The BCs: physical-goods, c2c,
services, subscriptions. Marketplace is built like a layer cake:
substrates at the bottom, BCs in the middle, surfaces on top.

### 14.7 "Why does ADR-0254 say the same Helm charts ship to SaaS and air-gapped, but the substrate beneath varies?"

Because the cell contents (the K8s manifests, the policy bundles,
the workflow definitions) are the unit that ships, and the cell
contents are identical across deployment models. What varies is
the substrate the cell sits on: a SaaS deployment puts the cell on
oyatie-operated AWS; a BYO-cloud deployment puts it on the tenant's
own AWS account; an on-prem deployment puts it on tenant hardware.
Same cell contents; different cloud substrate.

### 14.8 "If locks are forbidden, how do I do mutual exclusion?"

You do not. You use a saga (per ADR-0222) with compensation;
or an idempotency key (per ADR-0252); or a CRDT for last-writer-wins;
or a single-writer pattern where one designated workload owns a
resource and others queue requests. Distributed locks have a long
list of failure modes (network partition, lock-holder crash, clock
skew) that compromise correctness; ADR-0252's D-5 enumerates the
narrow exceptions where they are tolerated.

### 14.9 "How do I know if my new code is substrate or product?"

Apply the ADR-0245 tests: (a) is it audience-neutral or
audience-specific? (b) does it serve every tenant including
`oyatie`, or only a subset? (c) is its dependency direction
inward (consumed by other code) or outward (consumes other code)?
Substrate = audience-neutral, serves every tenant, consumed by
others, inward dependencies. Product = audience-specific, tenant-
scoped, consumes substrates, outward dependencies.

### 14.10 "What is a 'gate' in the Cedar context?"

A gate is any code path where a decision-with-policy-implication
is evaluated. Authorization (may principal X do action Y on
resource Z?) is one gate. Routing (which provider should we use for
data class D?) is another. Eligibility (is this tenant eligible for
this feature?) is another. The full list is in ADR-0243 §D-3 — 13
gate categories, with one minimum-required permit fragment per
category plus a default-deny.

---

## 15. After the keystone bundle

Once you have completed Phases 0-9 and answered the quiz, you have
the doctrinal foundation. The next reading paths, in priority order:

1. **The 13-layer canonical enum (ADR-0105).** This is the layer
   taxonomy that every crate respects.
2. **The hyperscaler invariants (ADR-0128).** The eight
   architecture invariants every µservice carries.
3. **Inter-microservice communication (ADR-0145).** The wire
   protocol and idempotency model.
4. **The per-µservice flat layout (ADR-0131).** The directory
   structure every µservice carries.
5. **Your axis's recent ADRs.** Filter `docs/decisions/` by your
   axis tag and read the last six months.
6. **Your µservice's PRD.** Whichever µservice you will be working
   on first.

The Day-One runbook (`/docs/onboarding/intern-day-one.md`) covers
the hands-on workflow once you have completed Phase 1 of this
document.

---

## 16. How to use this document with an agent

If you are an LLM agent reading this document as part of an agentic
flow, use this protocol:

1. Read the document in full.
2. Identify which phase you have completed so far (track in your
   working memory).
3. For each phase, after reading, write a 100-word summary of what
   you learned and store it in your notepad.
4. Before claiming completion of any phase, run the self-check
   questions for that phase and write down your answers.
5. Cite the specific ADR section in any decision you make that
   touches a keystone topic.
6. If you encounter a contradiction between an older ADR and a
   keystone ADR, the keystone wins.

The Day-One runbook covers the agentic equivalent of the human
runbook: build the bootstrap µservices, run the local cluster,
deploy a tenant, and submit a PR.

---

## 17. Document maintenance

This document is owned by `council-engineering + axis-devrel +
council-architecture`. When a keystone ADR amends or is superseded,
the relevant Phase section in this document is updated, the version
field is bumped, and a CHANGELOG entry is added below.

CHANGELOG:

- 2026-05-20: Initial publication. Tracks the 14-ADR
  `2026-05-20-foundational-doctrine` bundle.
