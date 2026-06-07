---
id: ADR-0247
status: Proposed
planning_impact: true
date: 2026-05-20
owners:
  - council-architecture
  - council-product
  - council-security
  - council-privacy
  - ops-sre-reliability
  - ops-compliance
  - axis-workflow-engine
  - axis-intelligence
  - axis-audit-chain
  - axis-policy-engine
  - axis-cell
  - axis-identity
  - axis-tenancy
supersedes:
  - ADR-0136-foundry-as-single-microservice.md
  - ADR-0136-amendment (Foundry internal-only carve-out, 2026-05-18)
amends:
  - ADR-0239-amendment-foundry-internal-scope-clarification-2026-05-18.md
  - ADR-0137-foundry-bounded-contexts.md (BC redistribution authority)
  - ADR-0138-foundry-six-path-deprecation.md (target redistribution)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0112-webhook-driven-foundry-agent-invocation.md
  - ADR-0113-vcs-orchestrator-end-to-end.md
  - ADR-0116-retire-external-agent-coordination-tooling.md
  - ADR-0123-hyperscaler-maturity-claim-gate.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-no-grouping-forward-policy.md
  - ADR-0139-agentic-slo-gated-promotion.md
  - ADR-0144-eu-ai-act-graduated-risk-tier-model.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0174-finops-cost-tagging-and-sustainability.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0192-milvus-vector-database.md
  - ADR-0200-wasmtime-substrate.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0220-consumer-intelligence-substrate.md
  - ADR-0221-agentic-development-pipeline-hardening.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
related_specs:
  - /specs/platform-architecture.json
  - /specs/microservices/workflow-engine.json
  - /specs/microservices/intelligence.json
  - /specs/microservices/audit-chain.json
  - /specs/microservices/policy-engine.json
  - /specs/microservices/cell.json
  - /specs/bootstrap-tier-model.json
  - /specs/self-modification-cedar-fragment-schema.json
  - /specs/dev-tools-cell-workflow-library.json
related_memory:
  - feedback_oyatie_is_a_tenant_doctrine
  - feedback_cedar_as_universal_gate
  - feedback_autonomous_implementation_artifacts
  - feedback_foundry_pipeline_canonical
  - feedback_quality_performance_scalability_bar
  - feedback_no_silent_regression
  - feedback_automate_everything
  - feedback_clean_architecture_requirements
  - feedback_workflow_studio_scope
  - feedback_flat_product_catalog
doc_class: Architecture-Decision-Record
keystone_bundle: 2026-05-20-foundational-doctrine
keystone_position: 6-of-14
purpose: >
  Establish the self-hosting / self-modification doctrine. The oyatie
  tenant's workflow library (formerly named "Foundry") has the
  self-hosting property — it can modify the platform that runs it,
  including itself, under Cedar-gated policy. The Foundry-as-µservice
  framing (ADR-0136 + ADR-0136-amendment + ADR-0239) DISSOLVES; its
  six bounded contexts redistribute to Workflow Engine + Intelligence +
  audit-chain + policy-engine. The internal-CI capability becomes a
  named bundle of workflow definitions + Cedar fragments + eval
  criteria running in `dev-tools-cell-N`. A strict bootstrap minimum
  (Tier 0: hardware + DNS + git host + container registry) is defined
  as external; everything above self-hosts.
enforcement_status: advisory-until-foundry-bc-redistribution-lands
enforced_by:
  - oya gate validate self-modification-permitted
  - oya gate validate bootstrap-tier-coherence
  - oya gate validate foundry-dissolution-complete
  - oya gate validate workflow-version-immutability
  - oya gate validate cedar-self-modification-permit-present
---

# ADR-0247: Self-Hosting / Self-Modification Doctrine

## Status

Proposed — 2026-05-20.

Bundled with the 14-ADR foundational keystone set (ADR-0242 through
ADR-0255 inclusive). Lands as a single multispectrum-reviewed PR
where partial acceptance is rejected because the doctrines are
mutually-reinforcing.

This keystone explicitly **SUPERSEDES**:

- **ADR-0136 (Foundry as a single µservice, 2026-05-18)** — the
  consolidation framing of six BCs into one µservice is replaced
  with a redistribution framing where the six BCs migrate to four
  receiving substrates. The consolidation analysis in ADR-0136 was
  correct about the operational coupling of the prior 6-way split;
  what was incorrect was the assumption that the consolidation
  destination should be a *new µservice* rather than the existing
  Workflow Engine + Intelligence + audit-chain + policy-engine
  surfaces that already cover the same primitives.
- **ADR-0136-amendment (Foundry internal-only carve-out, 2026-05-18)**
  — superseded by ADR-0242 (`oyatie`-is-a-tenant) which retires the
  internal-vs-consumer µservice audience distinction at the keystone
  level.

This keystone **AMENDS**:

- **ADR-0239 (Foundry scope clarification, 2026-05-18)** — the
  internal-only framing is amended in favour of the
  `oyatie`-is-a-tenant model, with the "internal" use cases preserved
  as a named workflow library running under the `oyatie.foundry.*`
  sub-scope per ADR-0242 §D-2.
- **ADR-0137 (Foundry bounded contexts)** — BC redistribution
  authority transfers from `microservices/foundry/` to the receiving
  substrate µservices per §D-1 below.
- **ADR-0138 (Foundry six-path deprecation)** — Strangler migration
  retargets to the redistribution destinations rather than to a
  consolidated foundry path.

Enforcement is `advisory-until-foundry-bc-redistribution-lands`. The
doctrine is accepted in text now; the CI lanes that enforce it move
to BLOCKER status only after:

1. `microservices/workflow-engine/` admits the runtime + supervisor
   BC content (per ADR-0249 companion).
2. `microservices/intelligence/` admits the providers + guardrails +
   eval BC content (per ADR-0255 companion).
3. `microservices/audit-chain/` admits the evidence BC content (per
   ADR-0250 companion).
4. `microservices/policy-engine/` admits the autonomy-tier-gate
   Cedar fragments (per ADR-0246 companion).
5. The named workflow library `dev-tools-cell-N` runs the first
   `pr-review` workflow successfully against a real PR.
6. The first round-trip self-modification cycle completes: a
   workflow publishes a new version of a workflow, runs against
   itself, and emits a sealed audit row attesting the cycle.

## Date

2026-05-20.

## Context

### Prior portfolio state (pre-keystone)

The foundry concept entered the oyatie portfolio through a sequence of
ADRs each addressing a real problem but cumulatively producing a
framing that does not survive the foundational doctrine bundle:

- **ADR-0022 (Autonomy tiers T0–T4, inherited from Bominal).**
  Establishes the autonomy ladder used by all agentic surfaces.
- **ADR-0024 (Foundry eval harness).** Original eval BC framing.
- **ADR-0025 (Foundry runtime consolidation).** Original runtime BC
  framing.
- **ADR-0123 (Hyperscaler maturity claim gate).** Each µservice
  registers an HG-* gate; foundry's HG-FOUNDRY was the consolidated
  6-into-1 claim.
- **ADR-0131 (Per-microservice flat layout).** Layout shape for any
  µservice including foundry.
- **ADR-0132 (No-grouping forward policy).** Foundry was not a suite;
  the 6-way split that preceded ADR-0136 was already non-suite.
- **ADR-0136 (Foundry as a single µservice, 2026-05-18).** Consolidates
  six prior foundry µservices into one `microservices/foundry/` with
  six internal BCs (runtime, supervisor, eval, evidence, guardrails,
  providers). 493 artefacts preserved.
- **ADR-0136-amendment (Foundry internal-only, 2026-05-18).** Codifies
  Foundry as serving "Hermes agentic development toolchain; CI/CD
  orchestration; internal eval substrate; internal evidence
  collection" — explicitly *not* tenant-facing.
- **ADR-0220 (Consumer Intelligence Substrate, 2026-05-18).** Creates
  `microservices/intelligence/` as the *consumer-facing* AI substrate.
- **ADR-0239 (Foundry internal scope clarification, 2026-05-18).**
  Draws the sharp boundary: foundry = INTERNAL; intelligence =
  CONSUMER. Manifest `audience` field per ADR-0221 §M-04.
- **ADR-0221 §M-04 (audience-as-µservice-scope field).** Every
  µservice declares INTERNAL | B2B-tenant | B2C-consumer | DEVELOPER.

The foundry µservice as it stands at the start of this keystone
bundle has:

| Surface | Count | Notes |
|---|---|---|
| Bounded contexts | 6 | runtime, supervisor, eval, evidence, guardrails, providers |
| Top-level PRD + lifecycle docs | 13 | PRD, PHASE-01, threat-model, DPIA, compliance, cost-budget, multi-region, incident-response, capacity-model, failure-modes, sdk-plan, competitor-parity, backfill-replay |
| Per-BC archive (bc-sources/) | 78 | 13 docs × 6 BCs |
| Implementation Plans | 90 | IP-001..IP-090 across BCs + IP-091..IP-097 milvus + IP-WASMTIME-001..004 |
| Catalog records | 135 | crate registry entries |
| Helm subcharts | 6 | one per BC under iac/helm/<bc>/ |
| Kustomize bases + overlays | 6+N | per-BC + per-pack overlays |
| Cedar fragments (foundry-scoped) | 41 | policy/ tree |
| Postgres migrations | 18 | per-BC schemas |
| OpenAPI + AsyncAPI + Proto contracts | 18 | 3 × 6 BCs |
| Runbooks | 36 | per-BC operational |
| Dashboards | 18 | per-BC |
| Capabilities | 18 | per-BC declared capabilities |
| OpenSLO manifests | 4 | foundation slos/ |
| Scorecards | per-IP | progress evidence |
| `oya-foundry-*` crates | 135 | per the catalog records |

That is approximately **493 file-level artefacts** plus 135 crates,
**all framed as serving an internal-only audience**. After ADR-0242
removes the internal-vs-consumer distinction and ADR-0243 makes Cedar
the universal gate, the framing under which foundry exists no longer
holds — the µservice was authored to a model that the keystone bundle
explicitly retires.

### What was correct about ADR-0136

ADR-0136's *consolidation* analysis was sound:

1. The 6-way split duplicated 72 cross-cutting docs (13 docs × 6 BCs
   minus the shared 13 at top-level) that would inevitably drift.
2. Six µservices fanning out to one capability hot path added
   ~5–15ms mTLS round-trip latency without isolation benefit.
3. Six governance lanes ran for one product; six SLO promotion
   runways tracked one operational surface.
4. Hyperscaler shape disagreement: AWS Bedrock, Google Vertex AI
   Agent Builder, Microsoft Azure AI Foundry, Anthropic Console,
   Palantir AIP, and LangSmith all ship ONE product surface with
   internal BCs — not six fan-out µservices.

These observations remain correct. The 6-way split was indeed a
topology error. The consolidation eliminated the error.

### What was incorrect about ADR-0136's choice of destination

The consolidation destination — *a new µservice called foundry* — was
incorrect for reasons that become visible only in the context of the
full keystone bundle:

1. **Workflow Engine already exists** (per ADR-0249 companion) as the
   universal orchestration substrate. Its core primitives are
   durable workflow execution, version pinning, retry + backoff,
   signal handling, child-workflow composition, and queryable
   workflow state. The foundry `runtime` BC's "capability dispatch"
   primitive *is* durable workflow execution at a high level — an
   agentic capability is a parameterised workflow that calls LLM
   tools + waits + collects results + emits evidence. Creating a
   separate `runtime` BC restates Workflow Engine primitives in
   foundry-specific vocabulary.
2. **Intelligence already exists** (per ADR-0255 companion) as the
   AI substrate. Its primitives include model gateway, provider
   adapters, eval harness, guardrails, RAG, and per-tenant context
   isolation. The foundry `providers`, `guardrails`, `eval` BCs are
   the exact same primitives.
3. **audit-chain already exists** (per ADR-0250 companion) as the
   Ed25519+Merkle-sealed substrate. The foundry `evidence` BC is the
   *caller* of audit-chain — not a peer substrate. Per ADR-0145
   invariant 1 ("each caller emits its own seal"), the receiver of
   evidence is audit-chain, not a foundry-internal store.
4. **policy-engine already gates everything** (per ADR-0246 + ADR-0243).
   The foundry `supervisor` BC's autonomy-tier-gate primitive is a
   Cedar evaluation under the policy-engine substrate; the supervisor
   BC's kill-switch primitive is a Cedar fragment activation under
   the policy-engine substrate; the supervisor BC's fleet lifecycle
   primitive is Workflow Engine workflows.

The cumulative observation: **every primitive that the foundry BCs
were authored to provide is already a primitive in one of four
existing substrates.** The foundry µservice is, in effect, a
restatement of those substrate primitives in vocabulary specific to
"internal Hermes pipeline" — which the `oyatie`-is-a-tenant doctrine
retires as a meaningful distinction.

### Why self-hosting matters at hyperscaler scale

The self-hosting property — the ability of a platform to modify itself
under its own policy gates — is the defining maturity test for
platform-class systems. Every named hyperscaler reference has the
property:

- **rustc bootstrap.** The Rust compiler is written in Rust. The
  current `stage1` compiler is built by an earlier `stage0` compiler
  (which is the previous release's `stage2`). New language features
  land in `stage1` and are then promoted to `stage2`. Every release
  goes through a multi-stage bootstrap that proves the compiler can
  compile itself. The rustc dev guide (rust-lang/rustc-dev-guide,
  chapter "Bootstrapping the compiler") describes the process in
  detail. The bootstrap minimum is a stage0 binary; everything above
  self-hosts.
- **Kubernetes self-hosting (kops, kubeadm "self-hosting" design
  proposals, kubeadm Phase 2 design 2017-2019).** kubeadm bootstraps
  the Kubernetes control plane as static pods on the bootstrap node,
  then migrates the control plane into DaemonSets / Deployments
  running on the cluster itself. The cluster runs the cluster.
  Reference: kubernetes/community design-proposals/cluster-lifecycle/
  self-hosting-kubernetes.md.
- **Linux distro cross-compile patterns.** Building a Linux
  distribution from scratch requires a cross-compile bootstrap
  (e.g., Linux From Scratch chapters 5-6 build a temporary toolchain
  using the host's compiler, then chapter 6 rebuilds the entire
  toolchain using itself, chroot'd, with the host's compiler removed
  from the search path). The distro then maintains itself through
  successive rebuilds. Reference: LFS book section "Constructing a
  Temporary System."
- **AWS internal infrastructure managing AWS.** Per Werner Vogels's
  2016 "10 Lessons from 10 Years of AWS" and the 2010-2014 internal-
  AWS-migration documented through re:Invent keynotes (2014, 2019),
  AWS retail infrastructure migrated to AWS. The AWS control plane
  team uses AWS IAM, AWS CloudFormation, AWS CodeDeploy, AWS Config
  to manage AWS itself. Subsequent control-plane deploys go through
  the same CI/CD pipelines that customers use.
- **Stripe's internal CI on Stripe infrastructure.** Stripe's internal
  CI (Sorbet typecheck, Stripe-internal tooling) runs on Stripe-
  internal Kubernetes that itself sits on Stripe's payment-rail
  control plane. Reference: Stripe Engineering blog "Bringing
  Pinglist to GitHub" (2020) + Brandur Leach "Sorbet in Production"
  (2019). Stripe's Sorbet type-checker is built by a Sorbet-typed
  build system.
- **Google Borg → Borg.** Borg manages Borg's own control plane
  services (per Verma et al., "Borg, Omega, and Kubernetes," CACM
  2016). Borg's master, scheduler, and link-shard processes run as
  Borg jobs scheduled by Borg.
- **Nix and Nix flakes.** Nixpkgs is built by a Nix binary; the Nix
  binary is itself a Nixpkgs derivation. `nix flake update` lets
  flakes pin their own input revisions. Reference: NixOS manual
  chapter "Hacking" + flakes RFC (rfcs/0049-flakes.md).
- **Anduril Lattice self-hosting for air-gapped operation.** Lattice
  ships with the ability to update itself in environments without
  external connectivity (per Anduril public marketing materials +
  GovCon contract documents 2023-2024). The bootstrap minimum is
  signed media + a one-time provisioning step.
- **Palantir Apollo deploying Palantir Apollo.** Apollo is Palantir's
  deployment system; it deploys Palantir Foundry instances + Palantir
  Gotham instances + Palantir AIP instances. Apollo also deploys
  Apollo itself. Reference: Palantir's "Continuous Deployment with
  Apollo" technical bulletin (palantir.com/platforms/apollo).
- **Cloudflare Workers deployed by Cloudflare Workers.** Cloudflare's
  internal tooling for Workers deployment runs *on* Workers.
  Reference: Cloudflare Engineering blog "Building Pingora" (2022),
  "Workers deploying Workers" (post-2023 series).

The pattern is unambiguous: **mature platforms self-host.** A platform
that requires an external CI system to deploy itself never reaches
maturity because the external CI system is necessarily a separate
trust boundary, separate compliance surface, separate audit chain,
and separate failure mode. Self-hosting eliminates these.

### What "self-modification" specifically means

Self-modification is a stronger property than self-hosting. The
distinction:

- **Self-hosting:** the platform can rebuild and redeploy itself given
  its own source code as input. Excluded from "self-hosting" is the
  initial bootstrap step (which must be done by something else; see
  §D-4 Tier 0).
- **Self-modification:** the platform can author new versions of its
  own components — including new policies, new workflows, new
  substrate code — *under Cedar-gated policy enforcement* that is
  itself part of the platform. The modifying agent (workflow or human)
  acts as an `oyatie.foundry.*` principal subject to the same gates as
  any tenant.

Self-modification is the property that enables `feedback_autonomous_implementation_artifacts`:
"Implement the masterplan runs without user intervention." For this
to be true, the workflows that implement the masterplan must be able
to modify the platform that runs them — including authoring new
ADRs, publishing new workflow versions, signing new Cedar fragments,
and deploying new substrate code — under deterministic policy gates
that prevent runaway escalation.

### The chicken-and-egg problem for self-modification

A self-modifying system has a foundational chicken-and-egg: the gates
that authorise modification must themselves be modifiable, or the
system locks in its initial gate set forever. But arbitrarily
modifiable gates allow the system to authorise any modification,
defeating the gate's purpose.

The resolution, drawn from PKI root-key ceremony practice (per
ADR-0243 §D-5) and from rustc's stage0 bootstrap:

1. There is a **root trust anchor** (org root key in a tier-0 HSM)
   that signs the initial gate set.
2. The initial gate set permits modification of gates only by
   workflows that themselves have been signed by intermediate keys
   chained to the root.
3. The root key is used only for intermediate-key rotation, not for
   day-to-day gate changes.
4. Root-key compromise has a documented recovery procedure (Shamir-
   shared M-of-N reconstitution by the founding team).

This resolution is canonical across hyperscalers and is preserved by
ADR-0243 (Cedar bootstrap chain of trust) + ADR-0246 (policy-engine
substrate).

### Why now (2026-05-20)

Three forcing functions:

1. **The keystone bundle removes the framing that foundry was
   authored to.** ADR-0242 retires internal-vs-consumer µservice
   audiences. ADR-0243 makes Cedar the universal gate. ADR-0245
   (substrate-vs-product layering) and ADR-0246 (policy-engine
   promotion) define peer substrate µservices. ADR-0249 (Workflow
   Engine as universal orchestrator) and ADR-0250 (audit-chain
   substrate promotion) and ADR-0255 (Intelligence rewrite) define
   the receiving substrates. In that bundle, foundry as a separate
   µservice has no remaining role; its primitives are restatements
   of peer-substrate primitives.
2. **The autonomous masterplan goal requires self-modification
   semantics that the current foundry framing does not articulate.**
   Foundry's six BCs describe a *pipeline*, not a *self-modification
   loop*. The pipeline framing is appropriate for CI/CD that runs on
   an external system; the self-modification loop framing is required
   for CI/CD that runs on its own platform.
3. **Reverse-direction redundancy.** The foundry µservice's 493
   artefacts plus 135 crates duplicate primitives that Workflow
   Engine + Intelligence + audit-chain + policy-engine already
   provide. Maintaining the duplication after the keystone bundle
   would be an unbounded coordination cost (every Workflow Engine
   change requires mirror update in foundry runtime BC; every
   Intelligence change requires mirror update in foundry providers
   BC; etc.).

## Decision

The keystone establishes twelve decision sub-sections, D-1 through
D-12.

### D-1. Foundry-as-µservice dissolves; BCs redistribute to existing substrates

The `microservices/foundry/` µservice is **retired**. Its six bounded
contexts redistribute as follows:

| BC | Receiving substrate | Receiving ADR | What moves |
|---|---|---|---|
| `runtime` | `microservices/workflow-engine/` | ADR-0249 | capability dispatch = durable workflow execution; session state = workflow state; capability registry = workflow registry; runtime pool = worker pool; invocation orchestrator = parent workflow + child workflows |
| `supervisor` | `microservices/workflow-engine/` | ADR-0249 | fleet lifecycle = workflow deployment lifecycle; capability deployment = workflow version publication; autonomy policy enforcement = Cedar fragment + workflow-engine gate; supervision event bus = workflow signals + queries; kill-switch = Cedar emergency-forbid fragment per ADR-0243 §Appendix B |
| `providers` | `microservices/intelligence/` | ADR-0255 | LLM provider router; per-provider adapters (Anthropic API, Anthropic Subscription, OpenAI API, OpenAI Subscription, Gemini API, Gemini Subscription, in-house); OpenBao credential adapter |
| `guardrails` | `microservices/intelligence/` (guardrails BC) | ADR-0255 | prompt classifier; output validator; autonomy-tier gate (Cedar adapter); content-safety rule engine; jailbreak detector; AI-slop detector; classifier model adapter (ONNX) |
| `eval` | `microservices/intelligence/` (eval BC) | ADR-0255 | eval runner kernel/domain/usecase; parity analyzer; replay engine; GPU runner pool; ClickHouse golden-store; S3 adapter |
| `evidence` | `microservices/audit-chain/` | ADR-0250 | capability-invocation recorder = audit-chain caller wrapper; evidence-pack builder = audit-chain pack assembler; regulator export = audit-chain export framework; per ADR-0145 invariant 1, each caller emits its own seal — the dedicated "evidence" BC dissolves into per-caller audit emission |

This redistribution preserves every primitive without preserving the
foundry µservice perimeter. The receiving substrates already cover
the primitives; the redistribution is a *vocabulary alignment*, not
a primitive loss.

The eval BC's location is decided by this ADR: **eval BC moves to
Intelligence** (not a new standalone substrate, not audit-chain).
Rationale: eval is a primitive of the AI substrate (it tests AI
behaviour against golden outputs); placing eval in Intelligence keeps
it co-located with the model gateway + provider adapters it tests,
matching the AWS Bedrock model-evaluations + Vertex AI evaluations +
Azure AI Foundry evaluations + Anthropic Console evaluations
hyperscaler pattern (all four place eval inside the AI product).

### D-2. The oyatie-tenant's CI/dev workflow library replaces "Foundry-as-product"

In place of "Foundry as a µservice", the CI/dev capability becomes a
**named workflow library** — a versioned collection of workflow
definitions + Cedar fragments + eval criteria that runs in the
Workflow Engine under the `oyatie.foundry.*` sub-scope per ADR-0242
§D-2.

Workflows in the library (initial set; per ADR-0249 versioning):

| Workflow ID | Purpose | Principal sub-scope |
|---|---|---|
| `oyatie.foundry.pr-review` | Multispectrum review v2.4.0 fan-out per facet; aggregate verdicts; admit or refuse PR | `oyatie.foundry.reviewer-agent` |
| `oyatie.foundry.multispectrum-review` | Per-facet sub-workflow (F1..F11 + M1..M2 + A1..A7) | `oyatie.foundry.reviewer-agent.facet-<f>` |
| `oyatie.foundry.ci-build-and-test` | cargo build + cargo test + cargo clippy + cargo fmt --check across the workspace | `oyatie.foundry.ci-agent` |
| `oyatie.foundry.merge-queue-fix-loop` | per ADR-0111, rebases queued PRs; runs CI; merges if green | `oyatie.foundry.merge-queue-controller` |
| `oyatie.foundry.adr-drafter` | Drafts ADR text from a brief; emits to docs/decisions/ | `oyatie.foundry.adr-drafter` |
| `oyatie.foundry.eval-runner` | Runs eval sets against current Intelligence configuration; emits parity reports | `oyatie.foundry.eval-runner` |
| `oyatie.foundry.evidence-emitter` | Wraps callers per ADR-0145 invariant 1; emits sealed audit rows | `oyatie.foundry.evidence-emitter` |
| `oyatie.foundry.release-deploy` | cosign-verifies artifacts; canary deploys via per-cell Helm releases; auto-rollback on SLO breach | `oyatie.foundry.release-controller` |
| `oyatie.foundry.dependency-update` | Renovate-style; opens PRs for dep bumps; chains to pr-review | `oyatie.foundry.dependency-bot` |
| `oyatie.foundry.security-scan` | trivy + grype + cosign-verify + cargo-audit; emits findings | `oyatie.foundry.security-scanner` |
| `oyatie.foundry.fragment-author` | Drafts new Cedar fragments per intent declaration; chains to multispectrum review | `oyatie.foundry.fragment-author` |
| `oyatie.foundry.workflow-publisher` | Publishes new workflow versions atomically (per D-7 below) | `oyatie.foundry.workflow-publisher` |
| `oyatie.foundry.substrate-upgrader` | Pulls cosign-attested substrate artifacts; canary-deploys via release-deploy | `oyatie.foundry.substrate-upgrader` |
| `oyatie.foundry.rollback-controller` | SLO-breach-triggered rollback; restores prior version of workflows or substrate | `oyatie.foundry.rollback-controller` |
| `oyatie.foundry.meta-trust-root` | Offline HSM-held trust anchor; witness-signs self-modification Cedar fragments; Shamir-shared 5-of-9 across ≥3 jurisdictions; key material NEVER leaves the HSM | `oyatie.foundry.meta-trust-root` |
| `oyatie.foundry.meta-trust-root-attestor` | Day-to-day automation principal that requests witness signatures from offline meta-trust-root key holders; does NOT hold key material; mirrors AWS KMS GenerateDataKey caller separation (per ADR-0293) | `oyatie.foundry.meta-trust-root-attestor` |
| `oyatie.foundry.bootstrap-runner` | Stage-1 external CI runner identity; bound via SPIFFE workload identity issued by `oyatie.foundry.bootstrap-ca`; ephemeral authority bounded by ≤8h bootstrap window; authority revoked by kill-switch fragment at T+8h regardless of Stage-2 readiness (per ADR-0295) | `oyatie.foundry.bootstrap-runner` |
| `oyatie.foundry.bootstrap-ca` | One-shot offline-rooted CA; issues SPIFFE certificates to bootstrap-runner principals during Stage 1; private key destroyed after Stage-1 completes (per ADR-0295) | `oyatie.foundry.bootstrap-ca` |
| `oyatie.foundry.bootstrap-kill-switch-publisher` | Per-region automation that publishes the kill-switch Cedar fragment at T+8h; bounded authority; does NOT hold trust-chain key material (per ADR-0295) | `oyatie.foundry.bootstrap-kill-switch-publisher` |

The workflows live as files in the **dev-tools workflow library
repository** (subdirectory `workflows/` of
`/specs/dev-tools-cell-workflow-library/`). Each workflow declares
its principal sub-scope, its Cedar permits, its required eval
criteria for promotion, and its rollback workflow.

Workflow source layout (per ADR-0249 conventions):

```
workflows/
  oyatie.foundry.pr-review/
    workflow.yaml           # canonical workflow definition
    versions/
      v3/
        workflow.yaml
        cedar-permits.cedar
        eval-criteria.yaml
        promotion-evidence.json
        signed-by.cosign
      v4-candidate/...
    rollback.yaml           # reference to rollback-controller invocation
```

### D-3. Self-modification mechanics

Workflows in the dev-tools workflow library can perform three classes
of self-modification, all gated by Cedar policy:

**Class 1: publish new workflow versions.** Per Workflow Engine D-7
(workflow-as-Object-Type pattern, per ADR-0249), a workflow is a
versioned Object Type. Publishing a new version is itself an action
`Workflow::Action::PublishWorkflowVersion`. The action is gated by
Cedar fragment `policy-engine/fragments/oyatie-self-modification-
permits.cedar` (see §D-8 below).

```
Sequence:
1. oyatie.foundry.workflow-publisher invokes
   WorkflowEngineApi::publish_version(workflow_id, new_version_yaml).
2. WorkflowEngine calls policy-engine.evaluate(...) with
   action=PublishWorkflowVersion.
3. policy-engine returns Permit (under D-8 conditions) or Forbid.
4. On Permit: WorkflowEngine stores the new version (immutable);
   does NOT make it active.
5. A separate action ActivateWorkflowVersion is required for
   atomic swap (D-7 below).
6. Audit row emitted to oyatie.foundry audit stream.
```

**Class 2: modify Cedar fragments.** Per ADR-0243 fragment lifecycle
§D-2, fragments are authored, reviewed, signed, published, activated.
Class 2 self-modification is the *authorship + publication* phase by
an `oyatie.foundry.fragment-author` workflow.

```
Sequence:
1. oyatie.foundry.fragment-author drafts a fragment per an intent
   declaration (e.g., "permit new action class X under conditions Y").
2. Fragment routes through oyatie.foundry.multispectrum-review per
   ADR-0243 §D-8 facets (F1, F2, F5, F6, F7, A1, A4, A6).
3. On verdict pass: fragment signed by intermediate signing key (per
   ADR-0243 §D-5 chain of trust).
4. Fragment publication action gated by Cedar fragment
   policy-engine/fragments/oyatie-self-modification-permits.cedar.
5. On Permit: fragment published to registry; hot-reloads to
   per-cell evaluators per ADR-0243 §D-10.
6. Audit emitted.
```

**Class 3: upgrade substrate code.** Per ADR-0249 + ADR-0211
(in-house preference), substrate µservices ship as cosign-attested
container artifacts. Class 3 self-modification is the
*artifact-pull + canary-deploy* phase by an
`oyatie.foundry.substrate-upgrader` workflow.

```
Sequence:
1. oyatie.foundry.substrate-upgrader receives a release tag from
   oyatie.foundry.release-controller.
2. Artifact pulled from registry; cosign signature verified against
   the org signing key chain (per ADR-0243 §D-5).
3. Action Substrate::Action::DeploySubstrateVersion gated by Cedar
   fragment policy-engine/fragments/oyatie-self-modification-permits.cedar.
4. On Permit: oyatie.foundry.release-deploy executes canary (10% of
   one cell's replicas; observe SLOs for 30 min; expand to 50% of
   cell; observe for 30 min; expand to 100% of cell; expand cell-
   by-cell across the global cell topology per ADR-0248).
5. SLO breach at any stage triggers oyatie.foundry.rollback-controller
   automatically.
6. Audit row emitted for every stage transition.
```

All three classes share a property: **the gate is itself a Cedar
fragment**, which means the gate can be modified by Class 2 self-
modification. The infinite regress is bounded by the chain of trust
(D-5 above) — Class 2 modification of the gate itself requires
multispectrum review + intermediate-key signing + the meta-permit
fragment that itself must be signed by the org root key (HSM, M-of-N
Shamir-shared).

### D-4. Bootstrap minimum (Tier 0)

The bootstrap minimum is the set of components that are **necessarily
external** because they are upstream of the platform's first
deployment. Tier 0 is small and explicit:

| Component | Why external | Sourcing |
|---|---|---|
| Hardware (cells, cloud accounts) | The platform cannot run on hardware that does not yet exist | Cloud-provider account (AWS / GCP / Azure / on-premises bare metal); bare-metal procurement; cell-class provisioning |
| DNS | The platform's own services need DNS resolution before they can serve their own DNS | External DNS provider (Route 53 / Cloudflare DNS / GCP Cloud DNS initially); migrate to self-hosted authoritative DNS µservice post-bootstrap |
| Git host (initial bootstrap source-of-truth) | The platform's source code must live somewhere before the platform exists | GitHub Enterprise / Gitea / GitHub (interim) initially; migrate to self-hosted git-host µservice post-bootstrap |
| Container registry (initial artifact destination) | Container images must be pushed somewhere before the platform exists | Harbor / ECR / GCR initially; migrate to self-hosted registry µservice (likely Harbor) post-bootstrap |

**Tier 0 explicitly excludes:**

- CI/CD systems (GitHub Actions, CircleCI, Jenkins) — self-hosted by
  the `oyatie.foundry.ci-build-and-test` workflow on Workflow Engine
  post-bootstrap.
- Container build tooling (cloud-vendor build services) — self-hosted
  by `oyatie.foundry.ci-build-and-test`.
- Artifact signing infrastructure — self-hosted by Sigstore-equivalent
  + HSM ceremony per ADR-0243 §D-5.
- Secrets management — self-hosted by `microservices/cloud-secrets/`
  (OpenBao) post step 2 of the bootstrap sequence per ADR-0242 §D-5.
- Observability stack — self-hosted by `microservices/observability/`
  post-bootstrap (per ADR-0130 inheritance).
- Identity provider — self-hosted by `microservices/identity/`
  (Zitadel) per ADR-0242 §D-5 step 3.
- Audit chain — self-hosted by `microservices/audit-chain/` per
  ADR-0242 §D-5 step 6.
- Cedar policy engine — self-hosted by `microservices/policy-engine/`
  per ADR-0246.
- Workflow Engine — self-hosted by `microservices/workflow-engine/`
  per ADR-0249.

**Bootstrap minimum invariant:** anything classified above Tier 0 must
self-host post-bootstrap. The CI lane `oya-check-bootstrap-tier-
coherence` (per §D-11 below) verifies the classification.

### D-5. Bootstrap sequence stages (stage 0 → steady state)

The full bootstrap sequence, from zero hardware to a self-modifying
steady state, has five stages:

**Stage 0 — External preparation (humans + Tier 0 only).**

| Step | Action | Owner |
|---|---|---|
| 0.0 | Cloud-provider account(s) provisioned; root credentials in tier-0 HSM (Shamir-shared M-of-N) | founding team |
| 0.1 | DNS zones registered with external DNS provider; org root domain confirmed | founding team |
| 0.2 | Git host org created (GitHub Enterprise / Gitea / GitHub); initial admin access | founding team |
| 0.3 | Container registry namespace created (Harbor / ECR / GCR); initial push credentials sealed in tier-0 HSM | founding team |
| 0.4 | Org root signing key generated in tier-0 HSM (Ed25519 + cosign); Shamir-shared M-of-N — **M=5, N=9 across ≥3 jurisdictions** for the meta-trust-root key and any other trust-chain anchor (org root, Cedar bootstrap root, compliance-pack publisher root); M=3, N=5 retained only for tenant-scoped operational keys — per ADR-0293 §5.5 Shamir threshold expansion | council-security |
| 0.5 | Bootstrap-replay log file initialised; ingest endpoint deferred until Stage 2 step 2.6 (per ADR-0242 §D-5) | council-security |

**Stage 1 — External CI runs kubeadm + signed-artifact deploy (humans + external Tier 0 CI temporarily).**

Stage 1 operates within a **hard ≤8h bootstrap budget** (T+0h = Step 0.4 key ceremony completion; T+8h = kill-switch activation regardless of Stage-2 readiness). All Stage-1 external CI runners impersonate the `oyatie.foundry.bootstrap-runner` principal via SPIFFE workload identity certificates issued by the one-shot `oyatie.foundry.bootstrap-ca` (per ADR-0295). Every Stage-1 artifact MUST be cosign-attested (sigstore cosign + Rekor) against the org root key chain before deployment. The bootstrap CA private key is destroyed after Step 1.10 completes.

| Step | Action | Owner |
|---|---|---|
| 1.0 | External CI runner impersonates `oyatie.foundry.bootstrap-runner` via SPIFFE SVID issued by `oyatie.foundry.bootstrap-ca` (one-shot offline-rooted CA); SVID bound to runner's hardware attestation; runner identity logged to bootstrap-replay log (Step 0.5). External CI executes `kubeadm init` against bootstrap-cell hardware; produces single-node K8s cluster | founding team |
| 1.1 | External CI runs Cilium CNI install per ADR-0193 inheritance; all artifacts cosign-verified | founding team |
| 1.2 | External CI runs minimal etcd verification | founding team |
| 1.3 | External CI deploys cosign-verified + sigstore-Rekor-attested container images via Helm: `microservices/cloud-secrets/` (OpenBao) — root unseal via Shamir HSM ceremony; attestation reference logged | council-security |
| 1.4 | External CI deploys `microservices/identity/` (Zitadel) — initial admin issued; OIDC client for `oyatie` org created; artifact cosign-attested | founding team |
| 1.5 | External CI deploys `microservices/tenancy/` — migration `0001_create_self_tenant.sql` runs; `oyatie` tenant row exists; artifact cosign-attested | founding team |
| 1.6 | External CI deploys `microservices/policy-engine/` — bootstrap Cedar fragment set signed by org root key loaded; artifact cosign-attested | council-security |
| 1.7 | External CI deploys `microservices/audit-chain/` — provisions `oyatie` tenant audit stream + Ed25519 signing key in OpenBao; artifact cosign-attested | council-security |
| 1.8 | External CI deploys cloud-iac cell provisioning — registers Tier 1 cell, marks bootstrap-class; artifact cosign-attested | founding team |
| 1.9 | External CI deploys `microservices/workflow-engine/` — minimal Workflow Engine; loads bootstrap workflow library; artifact cosign-attested. **Kill-switch Cedar fragment** `microservices/policy-engine/fragments/bootstrap/bootstrap-trust-roots-kill-switch.cedar` pre-loaded with `effective_at = T+8h` (see ADR-0295 §D-4); the `oyatie.foundry.bootstrap-kill-switch-publisher` automation activates this fragment automatically at T+8h, disabling `oyatie.foundry.bootstrap-runner` trust even if Stage-2 is not yet complete | founding team |
| 1.10 | **Out-of-band council-security two-member manual hash verification (per ADR-0295 §D-7):** Two members of council-security independently compute `sha256sum` of every cosign-attested Stage-1 artifact and compare against the Rekor log entries. Both members record their hash-match confirmation in the bootstrap-replay log (Step 0.5) as `StageOneArtifactHashVerification` audit events. Verification MUST complete before Stage 2.0 begins. Only after both confirmations does the transition proceed: external CI exits; "bootstrap-cell-T1" is online with the substrate stack | council-security (2-member quorum) |

**Stage 2 — Bootstrap cell self-hosts (substrate is up; foundry workflows take over).**

| Step | Action | Owner |
|---|---|---|
| 2.0 | First Foundry-equivalent workflow runs: `oyatie.foundry.ci-build-and-test` executes against the platform source repo | workflow-engine on bootstrap cell |
| 2.1 | `oyatie.foundry.ci-build-and-test` produces signed container artifacts for all µservices (including itself); pushes to container registry | workflow-engine on bootstrap cell |
| 2.2 | `oyatie.foundry.release-deploy` deploys Tier 2 control plane cells per ADR-0248 — three cells in three AZs, control plane class | workflow-engine on bootstrap cell |
| 2.3 | Tier 2 control plane cells run substrate stack (identity, tenancy, policy-engine, audit-chain, cell, workflow-engine) | tier-2 cells |
| 2.4 | `oyatie.foundry.release-deploy` deploys Tier 3 data plane cells per ADR-0248 — N cells across regions per the cell topology spec | workflow-engine on tier-2 cells |
| 2.5 | Tier 3 data plane cells host product µservices (mail, drive, calendar, messenger, workflow-studio, intelligence, etc.) | tier-3 cells |
| 2.6 | Bootstrap-replay log file from Stage 0 + 1 ingested into audit-chain on the bootstrap cell + replicated to control plane cells (retroactive audit trail per ADR-0242 §D-5 step 6 rationale) | council-security |

**Stage 3 — Bootstrap cell self-retires.**

| Step | Action | Owner |
|---|---|---|
| 3.0 | `oyatie.foundry.release-deploy` deploys a `dev-tools-cell-prod` (per D-6 below) on Tier 2 control plane | workflow-engine on tier-2 cells |
| 3.1 | dev-tools-cell-prod loads the dev-tools workflow library; runs `oyatie.foundry.ci-build-and-test` to verify identity with bootstrap cell's same library | workflow-engine on dev-tools-cell-prod |
| 3.2 | Bootstrap cell drained; workflows scheduled on bootstrap cell migrate to dev-tools-cell-prod | workflow-engine on tier-2 + dev-tools-cell-prod |
| 3.3 | Bootstrap cell decommissioned per ADR-0248 cellular-architecture retire procedure; hardware released | ops-sre-reliability |
| 3.4 | Audit row emitted: `BootstrapCellRetiredEvidence` signed by tier-2 control plane | audit-chain |

**Stage 4 — Steady state with self-modification.**

| Step | Action | Owner |
|---|---|---|
| 4.0 | dev-tools-cell-prod hosts the dev-tools workflow library; runs PR review, CI, eval, release-deploy, dependency-update, security-scan continuously | workflow-engine on dev-tools-cell-prod |
| 4.1 | dev-tools-cell-dev + dev-tools-cell-staging hosted on separate tier-2 cells for self-modification testing (per D-6) | workflow-engine |
| 4.2 | Self-modification cycles execute autonomously under Cedar gates; per D-3 | workflow-engine + policy-engine |
| 4.3 | Bootstrap is COMPLETE | (steady-state) |

The chicken-and-egg of Stage 1 (external CI deploys platform that
will replace external CI) is resolved by:

- External CI in Stage 1 is **time-bounded** (target: ≤ 8 hours from
  Stage 0.5 to Stage 2.0; cf. AWS bootstrap-time documentation).
- External CI runs only **cosign-attested artifacts signed by the
  org root key**; the artifacts' provenance is therefore in the
  platform's own trust chain.
- The bootstrap-replay log (Stage 0.5, ingested at Stage 2.6) makes
  the Stage 0 + 1 actions retroactively auditable within the
  platform's own audit chain.
- Stage 1's external CI is **shut down explicitly** at Stage 1.10;
  the dev-tools workflow library takes over at Stage 2.0. There is no
  silent retention of external CI as a bypass path.

### D-6. Multiple environments — dev-tools-cell-{dev,staging,prod}

Self-modifying workflows must be tested before they take effect in
production. This requires three environments:

| Cell | Role | Tier-2 / Tier-3 | Self-modification policy |
|---|---|---|---|
| `dev-tools-cell-dev` | Engineers develop new workflows / fragments / substrate versions; arbitrary modifications permitted by an `oyatie.dev.<engineer-id>` principal | Tier-2 dev | Cedar permits broad; no production data; auto-teardown unused workflows after 72h |
| `dev-tools-cell-staging` | Promotion target from -dev; runs full eval set + multispectrum review v2.4.0 against the candidate modifications | Tier-2 staging | Cedar permits narrower; some production-shape data via golden corpora; auto-rollback on SLO breach |
| `dev-tools-cell-prod` | Production self-modification execution; modifications must have passed -staging | Tier-2 prod | Cedar permits production-tight; SLO breaches trigger immediate auto-rollback + SEV-2 alert |

**Promotion gates between environments:**

- `dev` → `staging`: requires `oyatie.foundry.pr-review` verdict
  PASS for the candidate modification; requires
  `oyatie.foundry.eval-runner` parity report within tolerance.
- `staging` → `prod`: requires 24h soak time in `staging` with no
  SLO breach; requires multispectrum review v2.4.0 verdict PASS;
  requires human-approval (≥ 2 approvers from council-architecture
  + council-security) if the modification touches the self-modification
  permits fragment itself.
- `prod` SLO breach: auto-rollback triggered by
  `oyatie.foundry.rollback-controller` within 60s of breach detection;
  emits SEV-2 alert; post-mortem required within 72h.

This three-environment shape mirrors the canonical dev → staging →
prod CD pattern + the Kubernetes self-hosting design proposal's
"bootstrap → cluster → production" tiers + AWS internal infrastructure's
dev / gamma / prod fleet pattern.

### D-7. Workflow Engine substrate support for self-modification

The Workflow Engine substrate (per ADR-0249) MUST support the
following primitives for self-modification to work safely:

| Primitive | Behaviour |
|---|---|
| **Workflow versioning** | Each workflow has a version chain `v1, v2, ..., vN`; each version is immutable once published |
| **Per-instance version pinning** | A running workflow instance is pinned to the version it started on; subsequent version publications do not affect already-running instances |
| **New-instance-on-newer-version semantics** | New workflow instances always start on the currently *active* version |
| **Atomic active-version swap** | `ActivateWorkflowVersion(workflow_id, new_version)` is an atomic action; either fully takes effect or fully fails; gated by Cedar |
| **Drain-old-version semantics** | Optionally, the swap can include a drain window (e.g., 30 days for long-running workflows) where old instances finish on their pinned version before retire |
| **Rollback-via-active-swap** | Rolling back is simply `ActivateWorkflowVersion(workflow_id, vN-1)`; no special "rollback" primitive needed |
| **Per-workflow signal API** | Running instances can be queried + signalled (e.g., "drain", "cancel-graceful", "emergency-cancel") |
| **Workflow execution history audit** | Every state transition emits to audit-chain per ADR-0250 |
| **Workflow-as-Object-Type** | Workflows are first-class Object Types per ADR-0249 D-7; can be enumerated, queried, sub-scoped |

The atomic active-version swap is the **load-bearing** primitive: it
is what allows self-modification to be reversible. If the new version
breaks, the rollback is a single atomic action; the platform does
not enter a half-modified state.

### D-8. Cedar fragment specifically gating self-modification

The fragment that gates self-modification is canonical:

```cedar
// microservices/policy-engine/fragments/oyatie-self-modification-permits.cedar
// SCOPE: baseline
// SIGNED BY: org-baseline-key (intermediate, chained to org root per ADR-0243 §D-5)
// VERSION: v1
// EFFECTIVE_AT: <bootstrap-time>
// SUNSET_AT: null (long-lived; ratified at each annual root-key ceremony)

permit (
  principal in Tenant::"oyatie".sub_scopes("foundry"),
  action in [
    Workflow::Action::PublishWorkflowVersion,
    Workflow::Action::ActivateWorkflowVersion,
    Cedar::Action::PublishFragment,
    Cedar::Action::ActivateFragment,
    Substrate::Action::DeploySubstrateVersion,
    Substrate::Action::RollbackSubstrateVersion
  ],
  resource
)
when {
  principal.is_human_approval_present(min_approvers: 2)
  || (
    principal.is_automated_with_baseline_signed_workflow
    && principal.is_attested_by_meta_trust_root
  )
};

// NOTE (per ADR-0293 §D-4): the conjunctive form above replaces the prior
// single-predicate `is_automated_with_baseline_signed_workflow`. Rationale:
// `oyatie.foundry.workflow-publisher` signs the workflow artifact; the meta-
// trust-root witness independently attests that the self-modification action
// has been sanctioned by the offline HSM quorum. An adversary who compromises
// the workflow-publisher signing key alone cannot satisfy both predicates
// simultaneously, breaking the circular-predicate exploit identified in
// F5-247-01. The `is_attested_by_meta_trust_root` predicate is evaluated by
// policy-engine via the `oya-shared-meta-trust-root` crate's witness-
// verification primitive against the 5-of-9 Shamir-shared meta-trust-root key.

// Companion default-deny per ADR-0243 §D-3
forbid (
  principal,
  action in [
    Workflow::Action::PublishWorkflowVersion,
    Workflow::Action::ActivateWorkflowVersion,
    Cedar::Action::PublishFragment,
    Cedar::Action::ActivateFragment,
    Substrate::Action::DeploySubstrateVersion,
    Substrate::Action::RollbackSubstrateVersion
  ],
  resource
)
unless {
  principal in Tenant::"oyatie".sub_scopes("foundry")
};

// Meta-permit: modifying THIS fragment requires the strongest gate
forbid (
  principal,
  action == Cedar::Action::PublishFragment,
  resource is CedarFragment
)
when {
  resource.fragment_id == "baseline/oyatie-self-modification-permits.cedar"
}
unless {
  principal.is_human_approval_present(min_approvers: 3)
  && principal.is_council_security_approver_present
  && principal.is_council_architecture_approver_present
  && principal.is_signed_with_org_root_key_intermediate
};
```

The fragment encodes three policies:

1. **Permit:** `oyatie.foundry.*` principals may perform the six
   listed self-modification actions, under the condition that either
   ≥ 2 human approvers signed off OR the action is initiated by an
   automated workflow that itself is signed by an intermediate key
   (the "baseline-signed-workflow" condition).
2. **Default-deny:** non-`oyatie.foundry.*` principals are forbidden
   from these actions.
3. **Meta-permit:** modification of this fragment itself requires
   ≥ 3 human approvers including one from council-security and one
   from council-architecture, plus the modification must be signed by
   an intermediate key chained to the org root.

The `is_signed_with_org_root_key_intermediate` predicate is evaluated
by policy-engine via the signing chain check per ADR-0243 §D-5.

### D-9. Foundry artifact migration plan (493 → distributed)

Every artifact under `microservices/foundry/` has a documented
destination. The migration plan:

**Top-level lifecycle documents (13 docs)** redistribute:

| Foundry top-level | Destination |
|---|---|
| `microservices/foundry/PRD.md` | Distributes per-BC: each BC section moves to receiving substrate's PRD as a new section labelled `## Inherited from foundry/<bc>` |
| `microservices/foundry/PHASE-01-FOUNDRY-FOUNDATION.md` | Distributes: runtime + supervisor phases → `microservices/workflow-engine/PHASE-NN-INHERITED-FROM-FOUNDRY.md`; providers + guardrails + eval phases → `microservices/intelligence/PHASE-NN-INHERITED-FROM-FOUNDRY.md`; evidence phase → `microservices/audit-chain/PHASE-NN-INHERITED-FROM-FOUNDRY.md` |
| `microservices/foundry/PHASE-02-FOUNDRY-DATA-SUBSTRATE-ADDENDUM.md` | Milvus addendum moves to `microservices/intelligence/` (Milvus is RAG substrate per ADR-0192) |
| `microservices/foundry/threat-model.md` | Per-BC sections distribute to receiving substrate threat-models |
| `microservices/foundry/dpia.md` | Per-BC sections distribute to receiving substrate DPIAs |
| `microservices/foundry/compliance.md` | Per-BC sections distribute to receiving substrate compliance docs |
| `microservices/foundry/cost-budget.md` | Per-BC cost lines distribute to receiving substrate cost-budgets |
| `microservices/foundry/multi-region.md` | Per-BC sections distribute to receiving substrate multi-region docs |
| `microservices/foundry/incident-response.md` | Per-BC sections distribute to receiving substrate incident-response runbooks |
| `microservices/foundry/capacity-model.md` | Per-BC sections distribute |
| `microservices/foundry/failure-modes.md` | Per-BC sections distribute |
| `microservices/foundry/sdk-plan.md` | Per-BC SDK plans distribute |
| `microservices/foundry/competitor-parity-matrix.md` | Per-BC parity rows distribute |
| `microservices/foundry/backfill-replay.md` | Per-BC strategies distribute |

**bc-sources/ archive (78 docs)** distributes:

| Source | Destination |
|---|---|
| `bc-sources/runtime/*` | `microservices/workflow-engine/inherited/foundry-runtime/*` (preserved verbatim; audit-grade content preservation per ADR-0136 inheritance) |
| `bc-sources/supervisor/*` | `microservices/workflow-engine/inherited/foundry-supervisor/*` |
| `bc-sources/eval/*` | `microservices/intelligence/inherited/foundry-eval/*` |
| `bc-sources/evidence/*` | `microservices/audit-chain/inherited/foundry-evidence/*` |
| `bc-sources/guardrails/*` | `microservices/intelligence/inherited/foundry-guardrails/*` |
| `bc-sources/providers/*` | `microservices/intelligence/inherited/foundry-providers/*` |

**Implementation Plans (90 IPs)** renumber per receiving substrate's
IP series:

| Source | Destination |
|---|---|
| IP-001..IP-015 (runtime) | Renumber as `microservices/workflow-engine/IP-NNN-runtime-<feature>.md`; sequence assigned at end of workflow-engine's current IP series |
| IP-016..IP-030 (supervisor) | Renumber as `microservices/workflow-engine/IP-NNN-supervisor-<feature>.md` |
| IP-031..IP-045 (eval) | Renumber as `microservices/intelligence/IP-NNN-eval-<feature>.md` |
| IP-046..IP-060 (evidence) | Renumber as `microservices/audit-chain/IP-NNN-evidence-<feature>.md` |
| IP-061..IP-075 (guardrails) | Renumber as `microservices/intelligence/IP-NNN-guardrails-<feature>.md` |
| IP-076..IP-090 (providers) | Renumber as `microservices/intelligence/IP-NNN-providers-<feature>.md` |
| IP-091..IP-097 (milvus) | Renumber as `microservices/intelligence/IP-NNN-milvus-<feature>.md` |
| IP-WASMTIME-001..004 | Renumber as `microservices/intelligence/IP-NNN-wasmtime-<feature>.md` (Wasmtime serves Intelligence guardrails sandboxing per ADR-0200) |

A renumbering ledger lives at
`docs/migration-ledgers/2026-05-20-foundry-dissolution-ip-renumbering.md`
so external references can be remapped automatically.

**Catalog records (135 records)** distribute:

| Source pattern | Destination |
|---|---|
| `catalog/oya-foundry-runtime-*` | `microservices/workflow-engine/catalog/` |
| `catalog/oya-foundry-supervisor-*` | `microservices/workflow-engine/catalog/` |
| `catalog/oya-foundry-eval-*` | `microservices/intelligence/catalog/` |
| `catalog/oya-foundry-evidence-*` | `microservices/audit-chain/catalog/` |
| `catalog/oya-foundry-guardrails-*` | `microservices/intelligence/catalog/` |
| `catalog/oya-foundry-providers-*` | `microservices/intelligence/catalog/` |
| `catalog/oya-foundry-adapter-*` | `microservices/intelligence/catalog/` (provider adapters) |
| `catalog/oya-foundry-account-*` | `microservices/intelligence/catalog/` (provider account adapters) |
| `catalog/oya-foundry-autonomy-*` | `microservices/policy-engine/catalog/` (autonomy-tier-gate is Cedar) |
| `catalog/oya-foundry-bypass-*` | `microservices/audit-chain/catalog/` (bypass ledger is audit-chain emission) |
| `catalog/oya-foundry-capability-*` | `microservices/workflow-engine/catalog/` |
| `catalog/oya-foundry-api-*` | Distributes per receiving substrate's API |
| `catalog/oya-foundry-architecture-map-*` | `microservices/observability/catalog/` (architecture map is observability surface) |
| `catalog/oya-foundry-cargo-prefix-*` | `microservices/observability/catalog/` (build introspection) |
| `catalog/oya-foundry-catalog-*` | Catalog-on-catalog metaprogramming; distributes to observability |
| `catalog/oya-foundry-claude-account-* + codex-account-*` | `microservices/intelligence/catalog/` |
| `catalog/oya-foundry-cloud-mutation-*` | `microservices/workflow-engine/catalog/` (mutation = workflow signal) |
| `catalog/oya-foundry-dashboard-*` | `microservices/observability/catalog/` |

(The full per-crate mapping ledger lives at
`docs/migration-ledgers/2026-05-20-foundry-dissolution-crate-distribution.md`;
135 lines, one per crate.)

**Helm subcharts (6 charts)** distribute:

| Source | Destination |
|---|---|
| `iac/helm/runtime/` | `microservices/workflow-engine/iac/helm/runtime/` |
| `iac/helm/supervisor/` | `microservices/workflow-engine/iac/helm/supervisor/` |
| `iac/helm/eval/` | `microservices/intelligence/iac/helm/eval/` |
| `iac/helm/evidence/` | `microservices/audit-chain/iac/helm/evidence/` |
| `iac/helm/guardrails/` | `microservices/intelligence/iac/helm/guardrails/` |
| `iac/helm/providers/` | `microservices/intelligence/iac/helm/providers/` |

**Kustomize bases + overlays (6+N)** distribute:

| Source | Destination |
|---|---|
| `iac/kustomize/base/<bc>/` | `microservices/<receiving-substrate>/iac/kustomize/base/<bc>/` |
| `iac/kustomize/overlays/pack-<pack>/<bc>/` | `microservices/<receiving-substrate>/iac/kustomize/overlays/pack-<pack>/<bc>/` |

**Cedar fragments (41 fragments)** distribute:

| Source pattern | Destination |
|---|---|
| `policy/runtime-*.cedar` | `microservices/policy-engine/fragments/oyatie-workflow-engine-runtime-*.cedar` |
| `policy/supervisor-*.cedar` | `microservices/policy-engine/fragments/oyatie-workflow-engine-supervisor-*.cedar` |
| `policy/eval-*.cedar` | `microservices/policy-engine/fragments/oyatie-intelligence-eval-*.cedar` |
| `policy/evidence-*.cedar` | `microservices/policy-engine/fragments/oyatie-audit-chain-evidence-*.cedar` |
| `policy/guardrails-*.cedar` | `microservices/policy-engine/fragments/oyatie-intelligence-guardrails-*.cedar` |
| `policy/providers-*.cedar` | `microservices/policy-engine/fragments/oyatie-intelligence-providers-*.cedar` |
| `policy/autonomy-tier-*.cedar` | `microservices/policy-engine/fragments/oyatie-autonomy-tier-*.cedar` (cross-cutting) |

**Postgres migrations (18)** distribute per BC schema → receiving
substrate schema.

**OpenAPI + AsyncAPI + Proto contracts (18)** distribute:

| Source | Destination |
|---|---|
| `contracts/openapi/<bc>-*.yaml` | `microservices/<receiving>/contracts/openapi/<bc>-*.yaml` |
| `contracts/asyncapi/<bc>-*.yaml` | `microservices/<receiving>/contracts/asyncapi/<bc>-*.yaml` |
| `contracts/proto/<bc>-*.proto` | `microservices/<receiving>/contracts/proto/<bc>-*.proto` |

**Runbooks (36)** distribute per BC operational concern.

**Dashboards (18)** distribute per BC observability surface.

**Capabilities (18)** distribute as declared workflow capabilities.

**OpenSLO manifests (4)** distribute: foundation SLO becomes per-
receiving-substrate SLOs.

**Scorecards** distribute alongside their IPs.

**135 `oya-foundry-*` crates** rename (per the renumbering ledger):

| Source | Destination |
|---|---|
| `oya-foundry-runtime-*` | `oya-workflow-engine-runtime-*` |
| `oya-foundry-supervisor-*` | `oya-workflow-engine-supervisor-*` |
| `oya-foundry-eval-*` | `oya-intelligence-eval-*` |
| `oya-foundry-evidence-*` | `oya-audit-chain-evidence-*` |
| `oya-foundry-guardrails-*` | `oya-intelligence-guardrails-*` |
| `oya-foundry-providers-*` | `oya-intelligence-providers-*` |
| `oya-foundry-adapter-anthropic-*` | `oya-intelligence-adapter-anthropic-*` |
| `oya-foundry-adapter-openai-*` | `oya-intelligence-adapter-openai-*` |
| `oya-foundry-adapter-gemini-*` | `oya-intelligence-adapter-gemini-*` |
| `oya-foundry-account-*` | `oya-intelligence-account-*` |
| `oya-foundry-api-*` | Per-substrate API crates |
| `oya-foundry-autonomy-*` | `oya-policy-engine-autonomy-*` (autonomy-tier-gate is Cedar) |
| `oya-foundry-bypass-*` | `oya-audit-chain-bypass-*` (bypass ledger is audit emission) |
| `oya-foundry-capability-*` | `oya-workflow-engine-capability-*` |
| `oya-foundry-architecture-map-*` | `oya-observability-architecture-map-*` |
| `oya-foundry-cargo-prefix-*` | `oya-observability-cargo-prefix-*` |
| `oya-foundry-catalog-*` | `oya-observability-catalog-*` |
| `oya-foundry-dashboard-*` | `oya-observability-dashboard-*` |
| `oya-foundry-claude-account-*` | `oya-intelligence-claude-account-*` |
| `oya-foundry-codex-account-*` | `oya-intelligence-codex-account-*` |
| `oya-foundry-cloud-mutation-*` | `oya-workflow-engine-cloud-mutation-*` |

Crate renames execute per ADR-0212 buildability — each rename is a
ChangeSet that updates Cargo.toml + every `use ::` import + every
`[dependencies]` reference. The renumbering ledger drives the
automated sweep.

**`microservices/foundry/` directory:** DELETED post-migration. The
deletion is the last ChangeSet of the migration sequence; verified
by `oya gate validate foundry-dissolution-complete` returning empty.

### D-10. Hermes name retired

The name "Hermes" (used in earlier framings as
"Hermes agentic development pipeline" per ADR-0136-amendment and
ADR-0239) is **retired entirely** from oyatie canonical terminology.

Rationale: "Hermes" was inherited from an external github reference
(NousResearch/hermes-agent) and never became canonical in any oyatie
artifact beyond ADR-0136-amendment + ADR-0239. The keystone bundle
replaces it with explicit canonical names:

| Retired term | Canonical replacement |
|---|---|
| "Hermes" | (dropped — no replacement needed; the concept is now "oyatie.foundry.* workflows in dev-tools-cell-N") |
| "Hermes agentic development pipeline" | "oyatie.foundry workflow library" or "dev-tools workflow library" depending on context |
| "Hermes agent" | "oyatie.foundry.<workflow-id> instance" |
| "Hermes pipeline" | "the dev-tools cell pipeline" |

A glossary-compliance sweep removes "Hermes" references from all
existing artifacts. The sweep targets `docs/`, `microservices/`,
`crates/`, `tools/`, `.github/`, `specs/`. The canonical-glossary
CI lane (per ADR-0221 §M-03) is updated to refuse "Hermes" as a
violation post-sweep.

### D-11. CI lanes affected

The following CI lanes change:

**REMOVE:**

| Lane | Reason |
|---|---|
| `oya-governance-foundry-bc-source-coherence` | bc-sources archive distributes to receiving substrates per D-9; lane no longer applicable |
| `oya-governance-foundry-six-path-zero-usage` | Source paths under `microservices/foundry-<bc>/` were already deprecated by ADR-0138 Strangler; this ADR completes the Strangler by deleting `microservices/foundry/` itself |
| `oya-check-audience-coherence` | Per ADR-0242, audience-as-µservice-scope is retired |
| `oya-governance-foundry-helm-rollup-coherence` (if present) | foundry root Chart.yaml dissolves with the directory |

**ADD:**

| Lane | Purpose | Severity |
|---|---|---|
| `oya-check-self-modification-permitted` | Verifies every self-modifying workflow has an explicit Cedar permit fragment + human-approval requirement; rejects any workflow that performs PublishWorkflowVersion / ActivateWorkflowVersion / PublishFragment / ActivateFragment / DeploySubstrateVersion without the corresponding fragment | BLOCKER post-bootstrap |
| `oya-check-bootstrap-tier-coherence` | Verifies bootstrap-minimum components are external (DNS provider, git host, container registry, hardware) and everything else self-hosts; refuses µservice manifests that declare an external bootstrap dependency outside Tier 0 | BLOCKER post-bootstrap |
| `oya-check-foundry-dissolution-complete` | Verifies `microservices/foundry/` is deleted; verifies no `oya-foundry-*` crate names remain (all renamed); verifies the renumbering ledger is complete (every original IP has a destination row) | BLOCKER post-migration |
| `oya-check-workflow-version-immutability` | Verifies that published workflow versions are immutable (content hash matches at publish-time; no mutation post-publish) | BLOCKER |
| `oya-check-cedar-self-modification-permit-present` | Verifies `policy-engine/fragments/oyatie-self-modification-permits.cedar` exists, is signed by org-baseline-key, and includes the meta-permit clause | BLOCKER |
| `oya-check-hermes-name-retired` | Glossary-compliance sub-check; refuses "Hermes" string in any artifact outside this ADR's history and ADR-0136-amendment's archive | BLOCKER post-glossary-sweep |
| `oya-check-dev-tools-cell-environments` | Verifies dev-tools-cell-dev / -staging / -prod exist with the correct Cedar permit configuration | BLOCKER post-bootstrap |

### D-12. Failure modes — what happens when self-modification breaks the platform

Self-modification carries inherent risk: a bad workflow version or
Cedar fragment or substrate version can break the platform. The
failure-mode catalog and recovery procedure:

**Failure mode 1: bad workflow version activated.**

| Stage | Symptom | Recovery |
|---|---|---|
| Activation | Workflow instances on the new version fail immediately | Auto-rollback triggered by `oyatie.foundry.rollback-controller`; ActivateWorkflowVersion to v(N-1); audit row emitted; SEV-2 alert |
| Soak | SLO breach (e.g., error rate > 1% over 5 min) | Same as Activation |
| Production | Catastrophic (e.g., the workflow that runs rollback is itself broken) | Manual bootstrap-replay runbook per §below |

**Failure mode 2: bad Cedar fragment activated.**

| Stage | Symptom | Recovery |
|---|---|---|
| Activation | New denials cascade across the platform (e.g., a permit fragment incorrectly forbid replaces a permit) | Auto-detected by `oyatie.foundry.rollback-controller` watching audit-chain deny rate; rollback to fragment v(N-1); SEV-2 alert |
| Self-permitting drift | A fragment update unintentionally widens self-modification permits | Detected by `oya-check-cedar-self-modification-permit-present` static analysis at fragment review time (pre-publication); detection at activation time triggers fragment sunset + rollback |
| Lockout | A fragment update makes the policy-engine itself unable to evaluate (e.g., circular dependency in fragment imports) | Per-cell evaluators fall back to last-known-good cached compiled bundle per ADR-0243 §D-11 static stability; manual fragment removal via signed-by-root-key emergency procedure |

**Failure mode 3: bad substrate version deployed.**

| Stage | Symptom | Recovery |
|---|---|---|
| Canary (10%) | SLO breach in canary cell | Canary rolled back automatically; main release blocked; SEV-3 |
| Wider rollout (50%) | SLO breach in expanded cell | Roll back to previous version on the affected cell; pause rollout to other cells; SEV-2 |
| Production (100%) | Catastrophic (cell unable to host the substrate) | Per-cell failover to peer cell per ADR-0241 DR + BC portfolio; tier-2 control plane retains last-known-good substrate image |

**Failure mode 4: the rollback-controller itself is broken.**

| Step | Symptom | Recovery |
|---|---|---|
| Detection | Auto-rollback does not trigger despite SLO breach | Manual SEV-1 escalation; human incident commander invokes emergency-permit Cedar fragment (per ADR-0243 §Appendix B) narrowing platform to a safe-mode subset |
| Manual bootstrap replay | Platform reaches a state where neither auto-rollback nor manual rollback works | `docs/runbooks/oyatie-bootstrap-recovery.md` (per ADR-0242 §Implementation surface); replays from Tier 0 + Stage 0 bootstrap-replay log + cosign-attested known-good artifacts |

**Manual bootstrap-replay runbook** (high-level — full text at
`docs/runbooks/oyatie-bootstrap-recovery.md`):

```
1. Convene M-of-N Shamir shareholders to reconstitute org root key.
2. Verify the last cosign-attested known-good substrate artifact
   set against the root key.
3. Provision a recovery cell from Tier 0 hardware.
4. Re-run Stage 1 of the bootstrap sequence (external CI deploys
   substrate stack with the known-good artifacts).
5. Re-ingest the bootstrap-replay log from the recovery cell into
   audit-chain.
6. Migrate any preserved tenant data from the broken cells to the
   recovery cell.
7. Decommission the broken cells.
8. Authorize a post-incident review; ratify or revoke whatever
   self-modification triggered the failure.
9. Update Cedar fragments + workflow versions to prevent recurrence;
   land as a multispectrum-reviewed change.
```

The recovery procedure has been drilled at least once before BLOCKER
promotion of the doctrine; the drill evidence lives at
`evidence/bootstrap-recovery-drill-2026-XX-XX.json`.

## Alternatives Considered

### Alt-1. Keep `microservices/foundry/` as a separate µservice (status quo from ADR-0136)

Maintain the existing 493-artifact foundry µservice with its six
internal BCs, post-keystone-bundle.

**Pros:**

- Zero migration cost (foundry already exists).
- Familiar mental model for contributors who have internalised
  ADR-0136's consolidation rationale.
- Per-BC owners (axis-foundry-runtime / -supervisor / -eval /
  -evidence / -guardrails / -providers) retain their existing axis
  identity.
- ADR-0136's consolidation analysis (real operational coupling) is
  preserved.

**Cons:**

- **Duplicates primitives.** Workflow Engine + Intelligence +
  audit-chain + policy-engine already cover every primitive that
  foundry BCs provide. Maintaining the duplication is unbounded
  coordination cost.
- **Contradicts ADR-0242.** Foundry-as-internal-only µservice was
  retired by ADR-0242's removal of audience-as-µservice-scope.
- **Contradicts ADR-0249 + ADR-0250 + ADR-0255 + ADR-0246.** Each of
  these companion keystones introduces a substrate that already
  contains foundry-BC-equivalent primitives.
- **Blocks autonomous-masterplan-execution.** The pipeline framing of
  foundry-as-µservice does not articulate self-modification semantics
  required for autonomous masterplan execution.
- **Hermes name fossilises.** Continued foundry framing means
  continued use of "Hermes" terminology with no clear retirement
  path.

**Rejected** because the cons are unbounded coordination cost +
contradiction with every keystone in the bundle.

### Alt-2. Foundry stays but only as a thin façade routing to substrates

Retain `microservices/foundry/` as a thin µservice that exposes the
foundry product surface but internally routes every call to Workflow
Engine, Intelligence, audit-chain, or policy-engine.

**Pros:**

- Preserves the "Foundry" brand for internal users.
- Smaller migration cost than full dissolution.
- Allows gradual sunset of foundry surface as callers migrate.

**Cons:**

- **Façade-as-µservice anti-pattern.** A µservice that only routes is
  not a µservice; it is a thin gateway that should be a library or
  middleware, not a separate deployment unit.
- **Doubles audit emission.** Calls through the façade emit twice
  (once at the façade, once at the substrate) — per ADR-0145
  invariant 1, each caller emits its own seal, which would cause
  duplicate audit rows.
- **Continued operational coupling tax.** Every Workflow Engine
  change requires a façade update; every Intelligence change requires
  a façade update; etc.
- **Doesn't retire Hermes naming.**
- **Hyperscaler anti-pattern.** No hyperscaler maintains a thin
  façade µservice between internal-CI primitives and the substrates
  that provide them.

**Rejected** because the façade pattern duplicates audit emission +
adds operational coupling without isolation benefit.

### Alt-3. Promote foundry to peer substrate µservice (substrate-rank promotion)

Treat foundry as a peer substrate µservice on the same tier as
Workflow Engine, Intelligence, audit-chain, policy-engine.

**Pros:**

- Preserves foundry's existing structure.
- Substrate rank gives it formal peer status with the other four
  receiving substrates.

**Cons:**

- **Defines a substrate whose primitives are subsets of other
  substrates.** Per ADR-0245 (substrate-vs-product layering),
  substrates are defined by their *unique* primitives. Foundry's
  primitives are subsets of Workflow Engine + Intelligence +
  audit-chain + policy-engine primitives. Substrate rank without
  unique primitives is incoherent.
- **No-grouping policy violation.** ADR-0132 forbids new bundle/grouping
  µservices. A substrate that just bundles other substrates'
  primitives is a suite by another name.
- **Worse than Alt-1.** Substrate rank would expand foundry's
  governance surface (per-substrate SLO promotion runway, per-substrate
  capacity model, etc.) without changing the underlying duplication
  problem.

**Rejected** because foundry has no unique primitives that justify
substrate rank.

### Alt-4. Merge Workflow Engine + Intelligence + audit-chain + policy-engine into a single super-substrate

Resolve foundry's status by merging the four receiving substrates
into one super-substrate, eliminating the need for distribution.

**Pros:**

- Eliminates the four-way distribution.
- Single point of self-modification.

**Cons:**

- **Eight-into-one over-consolidation.** Each receiving substrate has
  distinct concerns (orchestration vs AI vs audit vs policy).
  Merging would lose the substrate-vs-product layering benefit per
  ADR-0245.
- **Hyperscaler shape disagreement.** No hyperscaler operates a
  single super-substrate. AWS has IAM (policy), CloudTrail (audit),
  Step Functions (workflow), Bedrock (AI) as separate substrates;
  GCP has IAM Conditions, Audit Logs, Workflows, Vertex AI; Azure
  has Azure Policy, Activity Log, Logic Apps, Azure AI Foundry.
- **Single point of failure.** A super-substrate makes every
  primitive depend on every other primitive's availability.
- **Worse than Alt-1 + Alt-3.**

**Rejected** as over-consolidation that contradicts hyperscaler shape.

### Alt-5. Foundry-BCs-redistribute-to-existing-substrates (CHOSEN)

The selected alternative, fully specified in §Decision.

**Pros:**

- **Eliminates the duplication.** Each primitive lives in exactly one
  substrate; receiving substrates already cover the primitives.
- **Aligns with every keystone in the bundle.** Consistent with
  ADR-0242 (oyatie-is-a-tenant), ADR-0243 (Cedar universal gate),
  ADR-0245 (substrate-vs-product), ADR-0246 (policy-engine),
  ADR-0249 (Workflow Engine), ADR-0250 (audit-chain), ADR-0255
  (Intelligence).
- **Enables self-modification semantics.** Per §D-3, the redistributed
  primitives compose into a clean self-modification loop with Cedar-
  gated mechanics.
- **Retires Hermes naming.**
- **Hyperscaler-shape alignment.** Matches the AWS / GCP / Azure
  pattern of separate workflow + AI + audit + policy substrates with
  internal-CI as a tenant of the platform.
- **Bootstrap minimum explicitly defined.** Tier 0 list of external
  dependencies is precise; everything above self-hosts.
- **Audit-grade content preservation.** Per ADR-0136 rule, zero
  content loss; every artifact has a documented destination per §D-9.

**Cons:**

- **One-time migration cost.** 493 artifacts + 135 crates rename +
  90 IPs renumber + 41 Cedar fragments distribute. Bounded; the
  migration is a single coordinated ChangeSet sequence (the renumbering
  ledger drives an automated sweep).
- **bc-sources/ archive remains as `inherited/foundry-<bc>/` under
  receiving substrates.** Contributors need to know the archive
  location; mitigated by frontmatter pointers in receiving substrate
  PRDs.
- **Per-BC owners reassign axis identity.** axis-foundry-runtime
  becomes axis-workflow-engine-runtime; axis-foundry-eval becomes
  axis-intelligence-eval; etc. One-time reassignment.

**Accepted** as the foundational keystone for self-hosting. The cons
are bounded one-time costs; the pros include alignment with the full
keystone bundle + every named hyperscaler reference + the
autonomous-masterplan-execution goal.

### Alt-6. Defer the decision to a later phase

Accept the keystone bundle without ADR-0247; resolve the foundry
question in a follow-up phase.

**Pros:**

- Smaller initial bundle.
- Time to consider redistribution options more carefully.

**Cons:**

- **ADR-0242 + ADR-0243 contradict foundry-as-internal-only
  framing.** Without ADR-0247, the bundle is internally inconsistent.
- **ADR-0249 + ADR-0250 + ADR-0255 receive primitives that overlap
  foundry BCs.** Without ADR-0247, the receiving substrate ADRs
  cannot describe their primitives cleanly.
- **Delay perpetuates drift.** Every day foundry remains as a
  separate µservice without a redistribution decision, more code +
  docs + IPs accumulate that will need to migrate later.
- **The drift cycle that produced ADR-0220 → ADR-0239 in 12 days
  recurs.** Deferring this decision is exactly the pattern that
  produces unbounded amendment loops.

**Rejected** because the keystone bundle is mutually-reinforcing;
partial acceptance has been explicitly rejected per ADR-0242 §Status.

## Consequences

### Positive

1. **Self-modification semantics codified.** The platform has a
   documented, Cedar-gated, reversible self-modification loop. The
   property required by `feedback_autonomous_implementation_artifacts`
   is satisfied by construction.
2. **Hyperscaler-shape alignment with self-hosting references.**
   Matches rustc bootstrap, Kubernetes self-hosting, Linux distro
   cross-compile, AWS internal infrastructure, Stripe internal CI,
   Google Borg, Nix flakes, Palantir Apollo, Cloudflare Workers.
3. **Duplication eliminated.** Each primitive lives in exactly one
   substrate; receiving substrates already cover them. The 493-
   artifact + 135-crate duplication retires.
4. **Bootstrap minimum precisely defined.** Tier 0 is small (4
   components) + explicit. Everything else self-hosts.
5. **Multi-environment self-modification testing.** dev-tools-cell-
   dev / -staging / -prod give self-modifying workflows the same
   dev → staging → prod promotion shape as any other code.
6. **Audit-grade content preservation.** Every artifact has a
   destination per §D-9; zero content loss.
7. **Hermes name retired.** Inherited-from-external terminology
   replaced with canonical oyatie names.
8. **Foundry workflow library is observable + auditable.** Every
   workflow execution emits to audit-chain per ADR-0250; every
   Cedar evaluation emits per ADR-0243 §D-7.
9. **Closes the drift loop of ADR-0136-amendment → ADR-0239.** The
   internal-vs-consumer framing is removed at the keystone level
   (per ADR-0242) + the foundry-as-µservice framing is replaced
   with workflow library + self-modification doctrine.
10. **Autonomous-masterplan-execution path is unblocked.** Workflows
    in the library can autonomously author + review + publish
    modifications to the platform under deterministic policy gates.
11. **Brand surface unification.** The "Foundry" brand (if retained
    in any external context) refers to the workflow library, not a
    µservice. The brand surface clarifies (workflow library ⊂
    Workflow Engine) rather than fragmenting (separate Foundry
    µservice).
12. **Reduced governance surface.** Six governance lanes for foundry
    BCs collapse into the receiving substrates' existing lanes; no
    HG-FOUNDRY gate (per ADR-0123) needed.

### Negative

1. **One-time migration cost.** 493 artifacts move + 135 crates
   rename + 90 IPs renumber + 41 Cedar fragments distribute + 6 Helm
   subcharts distribute + 18 contracts distribute + 36 runbooks
   distribute + 18 dashboards distribute + 18 capabilities distribute
   + 4 SLOs distribute. Mitigation: renumbering ledger drives
   automated sweep; the migration is a single coordinated ChangeSet
   sequence.
2. **Bootstrap sequence is rigorous.** 5 stages from zero hardware
   to steady state. Each stage audited. Mitigation: bootstrap-replay
   log retroactively audits Stage 0 + 1; full runbook at
   `docs/runbooks/oyatie-bootstrap-recovery.md`.
3. **Self-modification carries inherent risk.** A bad workflow
   version or Cedar fragment can break the platform. Mitigation:
   per-stage canary rollout + auto-rollback + manual bootstrap-replay
   procedure; multi-environment testing (dev → staging → prod);
   immutable workflow versions enable single-action rollback.
4. **The dev-tools workflow library is itself a maintenance surface.**
   Workflows + Cedar fragments + eval criteria + signing must be
   maintained. Mitigation: the library is itself maintained by
   workflows in the library (self-hosting), with the
   `oyatie.foundry.dependency-update` + `oyatie.foundry.workflow-
   publisher` workflows automating most maintenance.
5. **Per-BC owners reassign axis identity.** axis-foundry-runtime →
   axis-workflow-engine-runtime; etc. Mitigation: one-time
   reassignment; council-architecture coordinates within the
   migration ChangeSet.
6. **External references to `microservices/foundry/` paths break.**
   Any external doc / external code / external link to the path
   breaks post-deletion. Mitigation: redirect file at
   `microservices/foundry/REDIRECT.md` for one year sunset, pointing
   to the renumbering ledger; per ADR-0138 Strangler pattern.

### Operational

1. **New CI lanes:**
   - `oya-check-self-modification-permitted` (BLOCKER post-bootstrap)
   - `oya-check-bootstrap-tier-coherence` (BLOCKER post-bootstrap)
   - `oya-check-foundry-dissolution-complete` (BLOCKER post-migration)
   - `oya-check-workflow-version-immutability` (BLOCKER)
   - `oya-check-cedar-self-modification-permit-present` (BLOCKER)
   - `oya-check-hermes-name-retired` (BLOCKER post-glossary-sweep)
   - `oya-check-dev-tools-cell-environments` (BLOCKER post-bootstrap)

2. **Removed CI lanes:**
   - `oya-governance-foundry-bc-source-coherence`
   - `oya-governance-foundry-six-path-zero-usage`
   - `oya-check-audience-coherence`
   - `oya-governance-foundry-helm-rollup-coherence`

3. **New artifacts:**
   - `microservices/policy-engine/fragments/oyatie-self-modification-permits.cedar`
   - `/specs/dev-tools-cell-workflow-library.json`
   - `/specs/bootstrap-tier-model.json`
   - `/specs/self-modification-cedar-fragment-schema.json`
   - `docs/standards/dev-tools-workflow-library-authoring.md`
   - `docs/runbooks/oyatie-bootstrap-recovery.md`
   - `docs/runbooks/self-modification-incident-response.md`
   - `docs/migration-ledgers/2026-05-20-foundry-dissolution-ip-renumbering.md`
   - `docs/migration-ledgers/2026-05-20-foundry-dissolution-crate-distribution.md`
   - `microservices/workflow-engine/inherited/foundry-runtime/`
   - `microservices/workflow-engine/inherited/foundry-supervisor/`
   - `microservices/intelligence/inherited/foundry-providers/`
   - `microservices/intelligence/inherited/foundry-guardrails/`
   - `microservices/intelligence/inherited/foundry-eval/`
   - `microservices/audit-chain/inherited/foundry-evidence/`

4. **Removed artifacts (post-migration):**
   - `microservices/foundry/` directory (in entirety)
   - `oya-foundry-*` crate names (renamed per the distribution ledger)

5. **Observability:**
   - `oyatie.foundry.*` workflows visible in workflow-engine dashboard.
   - Self-modification events visible in audit-chain rollup view.
   - Per-environment dev-tools-cell SLOs in observability stack.
   - Cedar fragment lifecycle visible in policy-engine fragment
     registry browser.

6. **Tooling:**
   - `oya foundry workflow publish` (CLI for workflow authors).
   - `oya foundry fragment publish` (CLI for fragment authors).
   - `oya foundry substrate deploy` (CLI for substrate version
     deployment).
   - `oya foundry rollback` (CLI for manual rollback).
   - `oya foundry bootstrap replay` (CLI for bootstrap-replay
     procedure).

7. **HSM ceremony (annual):** root-key rotation per ADR-0243 §D-5;
   ops-compliance owns runbook. Inherits the existing Cedar ceremony.

### Sustainability

- **Reduced compute footprint.** Eliminating foundry-µservice
  duplication reduces redundant deployments. Estimated savings:
  ~3-5% of the platform's total compute footprint (the foundry's
  per-cell Helm subcharts no longer deploy as a separate µservice).
- **Sustained operability.** The substrate-distributed shape is
  long-term operable; no perpetual duplication tax accrues over
  the platform's lifetime.

### Compliance

- **GDPR Article 22 (automated individual decision-making).** Self-
  modification by automated workflows is a form of automated
  decision-making; per ADR-0243 §D-7, every Cedar decision emits to
  audit-chain with per-decision rationale, providing the auditability
  Article 22 requires.
- **EU AI Act Article 14 (transparency).** Self-modification cycles
  are AI-mediated (the workflows use Intelligence's LLM gateway for
  drafting); the applied-fragments list emitted per audit row
  provides the transparency Article 14 requires.
- **SOC 2 CC8.1 (change management).** Self-modification cycles are
  changes; the immutable-workflow-version + audit-chain emission +
  multispectrum review pattern satisfies CC8.1 change-management
  controls.
- **ISO 27001 A.12.1.2 (Change Management).** Same as SOC 2 CC8.1.
- **NIST SP 800-53 CM-3 (Configuration Change Control).** Cedar
  fragment lifecycle + workflow versioning + substrate deploy
  pipeline satisfy CM-3.
- **HIPAA Security Rule §164.308(a)(8) (evaluation).** Periodic
  technical + non-technical evaluation: the dev-tools workflow
  library's `oyatie.foundry.eval-runner` continuously evaluates
  platform behaviour against compliance fragments + golden corpora.
- **SOX §404 (internal controls).** For pre-IPO finance, the
  immutable-workflow-version + signing-chain + audit-chain pattern
  provides §404-grade internal control documentation.

## Implementation surface

| Artifact | Status |
|---|---|
| `/specs/dev-tools-cell-workflow-library.json` | NEW — schema for the workflow library |
| `/specs/bootstrap-tier-model.json` | NEW — Tier 0 enumeration + validation |
| `/specs/self-modification-cedar-fragment-schema.json` | NEW — schema for the self-modification permits fragment |
| `microservices/policy-engine/fragments/oyatie-self-modification-permits.cedar` | NEW — the canonical fragment from §D-8 |
| `microservices/workflow-engine/workflows/oyatie.foundry.pr-review/workflow.yaml` | NEW — initial workflow |
| `microservices/workflow-engine/workflows/oyatie.foundry.multispectrum-review/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.ci-build-and-test/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.merge-queue-fix-loop/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.adr-drafter/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.eval-runner/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.evidence-emitter/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.release-deploy/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.dependency-update/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.security-scan/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.fragment-author/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.workflow-publisher/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.substrate-upgrader/workflow.yaml` | NEW |
| `microservices/workflow-engine/workflows/oyatie.foundry.rollback-controller/workflow.yaml` | NEW |
| `microservices/workflow-engine/inherited/foundry-runtime/` (78 docs from bc-sources/runtime) | NEW |
| `microservices/workflow-engine/inherited/foundry-supervisor/` (78 docs from bc-sources/supervisor) | NEW |
| `microservices/intelligence/inherited/foundry-providers/` (78 docs) | NEW |
| `microservices/intelligence/inherited/foundry-guardrails/` (78 docs) | NEW |
| `microservices/intelligence/inherited/foundry-eval/` (78 docs) | NEW |
| `microservices/audit-chain/inherited/foundry-evidence/` (78 docs) | NEW |
| `microservices/workflow-engine/IP-NNN-runtime-*.md` (15 IPs renumbered) | NEW |
| `microservices/workflow-engine/IP-NNN-supervisor-*.md` (15 IPs renumbered) | NEW |
| `microservices/intelligence/IP-NNN-eval-*.md` (15 IPs renumbered) | NEW |
| `microservices/audit-chain/IP-NNN-evidence-*.md` (15 IPs renumbered) | NEW |
| `microservices/intelligence/IP-NNN-guardrails-*.md` (15 IPs renumbered) | NEW |
| `microservices/intelligence/IP-NNN-providers-*.md` (15 IPs renumbered) | NEW |
| `microservices/intelligence/IP-NNN-milvus-*.md` (7 IPs renumbered) | NEW |
| `microservices/intelligence/IP-NNN-wasmtime-*.md` (4 IPs renumbered) | NEW |
| `microservices/workflow-engine/catalog/oya-workflow-engine-runtime-*.yaml` (per ledger) | NEW |
| `microservices/workflow-engine/catalog/oya-workflow-engine-supervisor-*.yaml` (per ledger) | NEW |
| `microservices/intelligence/catalog/oya-intelligence-*.yaml` (per ledger) | NEW |
| `microservices/audit-chain/catalog/oya-audit-chain-evidence-*.yaml` (per ledger) | NEW |
| `microservices/policy-engine/catalog/oya-policy-engine-autonomy-*.yaml` (per ledger) | NEW |
| `microservices/observability/catalog/oya-observability-*.yaml` (per ledger) | NEW |
| `microservices/workflow-engine/iac/helm/runtime/` | NEW (from foundry) |
| `microservices/workflow-engine/iac/helm/supervisor/` | NEW |
| `microservices/intelligence/iac/helm/eval/` | NEW |
| `microservices/intelligence/iac/helm/guardrails/` | NEW |
| `microservices/intelligence/iac/helm/providers/` | NEW |
| `microservices/audit-chain/iac/helm/evidence/` | NEW |
| `microservices/policy-engine/fragments/oyatie-workflow-engine-runtime-*.cedar` | NEW (from foundry policy/) |
| `microservices/policy-engine/fragments/oyatie-workflow-engine-supervisor-*.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-intelligence-eval-*.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-audit-chain-evidence-*.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-intelligence-guardrails-*.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-intelligence-providers-*.cedar` | NEW |
| `microservices/policy-engine/fragments/oyatie-autonomy-tier-*.cedar` | NEW |
| Per-substrate threat-model + DPIA + compliance + multi-region + capacity-model + cost-budget + sdk-plan + competitor-parity-matrix + backfill-replay + failure-modes + incident-response updates | UPDATE |
| `docs/standards/dev-tools-workflow-library-authoring.md` | NEW |
| `docs/standards/self-modification-procedure.md` | NEW |
| `docs/runbooks/oyatie-bootstrap-recovery.md` | NEW |
| `docs/runbooks/self-modification-incident-response.md` | NEW |
| `docs/runbooks/dev-tools-cell-promotion-procedure.md` | NEW |
| `docs/migration-ledgers/2026-05-20-foundry-dissolution-ip-renumbering.md` | NEW |
| `docs/migration-ledgers/2026-05-20-foundry-dissolution-crate-distribution.md` | NEW |
| `docs/migration-ledgers/2026-05-20-foundry-dissolution-cedar-fragment-distribution.md` | NEW |
| `tools/oya-check-self-modification-permitted/` | NEW |
| `tools/oya-check-bootstrap-tier-coherence/` | NEW |
| `tools/oya-check-foundry-dissolution-complete/` | NEW |
| `tools/oya-check-workflow-version-immutability/` | NEW |
| `tools/oya-check-cedar-self-modification-permit-present/` | NEW |
| `tools/oya-check-hermes-name-retired/` | NEW |
| `tools/oya-check-dev-tools-cell-environments/` | NEW |
| Crate renames: 135 `oya-foundry-*` → distributed names | SWEEP |
| Cargo.toml + workspace member updates for 135 renames | SWEEP |
| `microservices/foundry/` directory deletion | SWEEP (final ChangeSet) |
| `microservices/foundry/REDIRECT.md` (1-year sunset pointer) | NEW (temporary) |
| Removal of CI lanes per §D-11 | SWEEP |
| Addition of CI lanes per §D-11 | NEW |
| Glossary update: "Hermes" → retired in `docs/glossary.md` | UPDATE |
| ADR-0136 frontmatter update: `superseded_by: [ADR-0247]` | UPDATE |
| ADR-0136-amendment frontmatter update: `superseded_by: [ADR-0247]` | UPDATE |
| ADR-0137 frontmatter update: `amended_by: [ADR-0247]` | UPDATE |
| ADR-0138 frontmatter update: `amended_by: [ADR-0247]` | UPDATE |
| ADR-0239 frontmatter update: `amended_by: [ADR-0247]` | UPDATE |

## Verification

- [ ] `microservices/foundry/` directory does not exist (deletion ChangeSet committed).
- [ ] No `oya-foundry-*` crate names exist in `crates/` (all 135 renamed per ledger).
- [ ] `microservices/workflow-engine/inherited/foundry-runtime/` + `foundry-supervisor/` exist with full bc-sources content.
- [ ] `microservices/intelligence/inherited/foundry-providers/` + `foundry-guardrails/` + `foundry-eval/` exist with full bc-sources content.
- [ ] `microservices/audit-chain/inherited/foundry-evidence/` exists with full bc-sources content.
- [ ] All 90 original Foundry IPs renumber per `docs/migration-ledgers/2026-05-20-foundry-dissolution-ip-renumbering.md`; ledger validates against destinations on disk.
- [ ] `microservices/policy-engine/fragments/oyatie-self-modification-permits.cedar` exists, is signed by org-baseline-key, parses under Cedar v4.2, passes `cedar-policy-analyzer` formal verification.
- [ ] `oya gate validate self-modification-permitted` exits 0.
- [ ] `oya gate validate bootstrap-tier-coherence` exits 0; lists exactly 4 Tier-0 dependencies.
- [ ] `oya gate validate foundry-dissolution-complete` exits 0; reports zero remaining foundry artifacts.
- [ ] `oya gate validate workflow-version-immutability` exits 0; sample 10 published workflows; all hashes match published content.
- [ ] `oya gate validate cedar-self-modification-permit-present` exits 0.
- [ ] `oya gate validate dev-tools-cell-environments` exits 0; three cells (dev, staging, prod) present with the correct Cedar permit configuration.
- [ ] `grep -r "Hermes" docs/ microservices/ crates/ tools/ specs/` returns only hits in ADR-0136-amendment (historical archive), ADR-0239 (historical archive), this ADR-0247 (retirement), the glossary's "retired terms" section, and the bootstrap-replay log archive.
- [ ] Bootstrap dry-run from zero-hardware to Stage 4 completes within 8 hours on the reference hardware profile.
- [ ] Bootstrap-recovery drill completes per `docs/runbooks/oyatie-bootstrap-recovery.md` within 4 hours; recovery cell hosts the platform at Stage 4 fidelity.
- [ ] First self-modification cycle (workflow publishes new version of itself; runs against own emitted audit row to verify) completes within 5 minutes; emits 4+ audit rows (publish-attempt, Cedar evaluate, publish-commit, activate, post-activation evaluation).
- [ ] First rollback drill (intentionally bad workflow version activated; rollback-controller restores prior version) completes within 60 seconds.
- [ ] Cedar fragment hot-reload p99 across all per-cell evaluators in dev-tools-cell-prod measured < 5s under load.
- [ ] Multispectrum review v2.4.0 verdict PASS achieved for the keystone ADR set including ADR-0247.
- [ ] ADR-0136, ADR-0136-amendment, ADR-0239, ADR-0137, ADR-0138 frontmatter updated with the appropriate `superseded_by` / `amended_by` field pointing to ADR-0247.
- [ ] Audit row emitted: `FoundryDissolutionCompleteEvidence` signed by tier-2 control plane key.

## References

### Industry sources (self-hosting / bootstrap)

- **The Rust Programming Language Compiler — `rust-lang/rustc-dev-guide` chapter "Bootstrapping the compiler"** (rustc.rust-lang.org/rustc-dev-guide/building/bootstrapping.html). Stage 0 / Stage 1 / Stage 2 compiler chain; bootstrap minimum.
- **rust-lang/rust `src/bootstrap/README.md`**. Operational details of rustc self-hosting.
- **Linux From Scratch (LFS) book, sections "Constructing a Temporary System" + "Building the LFS System"** (linuxfromscratch.org/lfs/view/stable/). Cross-compile bootstrap from host toolchain.
- **kubernetes/community design-proposals/cluster-lifecycle/self-hosting-kubernetes.md** (2017-2019 era). Kubernetes self-hosting design discussion.
- **kubernetes/kubeadm Phase 2 self-hosting design** (kubernetes.io/docs/setup/production-environment/tools/kubeadm/self-hosting). kubeadm self-hosted control plane.
- **Sigstore project — Rekor + Cosign + Fulcio** (sigstore.dev). Signed-artifact attestation, used in §D-3 substrate upgrader.
- **NixOS Manual, chapter "Hacking" + `Nix Flakes` RFC 49** (github.com/NixOS/rfcs/blob/master/rfcs/0049-flakes.md). Self-pinning flake inputs.
- **Werner Vogels, "10 Lessons from 10 Years of AWS"** (All Things Distributed, 2016). Amazon's internal-AWS migration.
- **AWS re:Invent 2014 keynote (Andy Jassy) + AWS re:Invent 2019 keynote (Werner Vogels)**. amazon.com runs on AWS.
- **Stripe Engineering blog — "Bringing Pinglist to GitHub"** (stripe.com/blog, 2020). Stripe-internal infrastructure runs on Stripe.
- **Brandur Leach, "Sorbet in Production"** (brandur.org, 2019). Stripe's Sorbet typechecker is built by a Sorbet-typed build system.
- **Verma et al., "Borg, Omega, and Kubernetes"** (CACM 2016, vol. 59 no. 5). Google Borg manages Borg's own control plane.
- **Palantir Technical Bulletin — "Continuous Deployment with Apollo"** (palantir.com/platforms/apollo). Apollo deploys Apollo.
- **Cloudflare Engineering blog — "Building Pingora"** (blog.cloudflare.com/pingora-open-source, 2022). Cloudflare's edge runs on Cloudflare.
- **Cloudflare Workers internal deployment** (blog.cloudflare.com 2023-2024 series). Workers deployed by Workers.
- **Anduril Lattice for air-gapped self-hosting** (anduril.com/lattice; GovCon contract documents 2023-2024). Self-hosting in disconnected environments.
- **Microsoft One Engineering System (1ES)** (microsoft.com/devblogs/one-engineering-system-2014). Microsoft IT runs on Azure; 1ES self-hosts on itself.

### Industry sources (self-modification / policy / signing)

- **AWS Verified Permissions** (docs.aws.amazon.com/verifiedpermissions). Reference Cedar production deployment, including self-modification of policies under signed chains.
- **AWS Key Management Service (KMS) key hierarchy** (docs.aws.amazon.com/kms/latest/developerguide/concepts.html). Root + intermediate signing chain pattern.
- **Sigstore "Cosign" attestation format**. Used in §D-3 artifact pull.
- **PKI X.509 RFC 5280**. Root + intermediate certificate chain.
- **Certificate Transparency RFC 6962**. Audit-replay of signing-key use.
- **Shamir's Secret Sharing scheme (Shamir 1979, "How to Share a Secret," CACM 22 no. 11)**. M-of-N HSM root-key sharding.
- **NIST SP 800-57 Part 1 (Recommendation for Key Management — Part 1: General)**. Key lifecycle.

### Industry sources (workflow + orchestration)

- **Temporal documentation — "Workflow Versioning"** (docs.temporal.io). Per-instance version pinning + atomic active-version swap pattern.
- **Cadence documentation (Uber, pre-Temporal)**. Workflow versioning ancestor.
- **AWS Step Functions versioning** (docs.aws.amazon.com/step-functions). Atomic state-machine version activation.
- **GCP Workflows versioning** (cloud.google.com/workflows/docs/creating-updating-workflow).
- **Azure Logic Apps "stateful workflow versioning"**.
- **Argo Workflows + Argo CD GitOps** (argoproj.io). Workflow-driven self-modification of cluster state.

### Industry sources (CI/CD self-hosting)

- **GitHub Actions self-hosted runners** documentation. Pattern for migrating off vendor-hosted CI.
- **Drone CI + Drone self-hosted** (drone.io). Self-hosted CI ancestor.
- **Jenkins X (the cloud-native Jenkins reimagining)** documentation. Self-deploying Jenkins X.
- **Buildkite self-hosted agents**. Hybrid model.

### Regulatory sources

- **GDPR Article 22 — Automated individual decision-making**. Self-modification audit-trail requirement.
- **EU AI Act 2024/1689 Article 14 — Transparency obligations**. AI-mediated self-modification transparency.
- **SOC 2 Type II — CC8.1 (Change Management)**. Self-modification = change.
- **ISO 27001:2022 Annex A.12.1.2 — Change Management**.
- **NIST SP 800-53 CM-3 — Configuration Change Control**.
- **NIST SP 800-53 CM-5 — Access Restrictions for Change**.
- **HIPAA Security Rule §164.308(a)(8) — Evaluation**.
- **SOX §404 — Internal Controls**.
- **FRCP 37(e) — Failure to Preserve Electronically Stored Information**. Audit-chain retention through self-modification cycles.

### Academic + practitioner sources

- **Reflection in computing — Brian Cantwell Smith, "Procedural Reflection in Programming Languages" (MIT, 1982)**. Foundational reflective-tower work; the platform's self-modification is a reflective tower with Cedar gating reflection access.
- **Eric Brewer, "Towards Robust Distributed Systems" (PODC 2000)**. CAP context.
- **Pat Helland, "Life Beyond Distributed Transactions" (2007)**. Workflow durable execution theoretical foundation.
- **"The Reflective Tower" — Friedman + Wand, "Reification: Reflection without Metaphysics" (LISP 1984)**. Bounded reflection patterns.
- **Eric Evans, *Domain-Driven Design* (Addison-Wesley, 2003)**. Bounded contexts (referenced by ADR-0136 inheritance).
- **Vaughn Vernon, *Implementing Domain-Driven Design* (Addison-Wesley, 2013)**. BC integration patterns.
- **Site Reliability Engineering — Beyer, Jones, Petoff, Murphy, eds. (O'Reilly, 2016)**, chapter 8 "Release Engineering" — self-hosting release engineering pattern.
- **The Site Reliability Workbook — Beyer, Murphy, Rensin, Kawahara, Thorne, eds. (O'Reilly, 2018)**, chapter 4 "Service Level Objectives" — SLO-gated rollout.

### Internal portfolio ADRs

- **ADR-0009 — Cell architecture per-tenant per-region.** dev-tools-cell-N lives on Tier-2 cells.
- **ADR-0028 — Cloud microservice architecture.** Substrate µservice pattern.
- **ADR-0049 — Cross-region replication + residency.** dev-tools-cell-prod replicates across packs.
- **ADR-0105 — Thirteen-layer canonical enum.** Workflow library uses layer-0 (canonical) workflows.
- **ADR-0111 — Merge queue projected state.** `oyatie.foundry.merge-queue-fix-loop` implements the projected state.
- **ADR-0112 — Webhook-driven Foundry agent invocation.** Webhooks become Workflow Engine triggers.
- **ADR-0113 — VCS orchestrator end-to-end.** Reinterpreted as a Workflow Engine + audit-chain composition.
- **ADR-0116 — Retire external agent coordination tooling.** Foundry workflows replace external coordination.
- **ADR-0123 — Hyperscaler maturity claim gate.** HG-FOUNDRY retires; per-substrate HG gates inherit.
- **ADR-0128 — Hyperscaler architecture invariants.** Self-hosting is a hyperscaler invariant.
- **ADR-0131 — Per-microservice flat layout.** Receiving substrates retain flat layout.
- **ADR-0132 — No-grouping forward policy.** Foundry-as-µservice dissolution complies (no new bundle).
- **ADR-0136 — Foundry as single µservice.** Superseded by this ADR.
- **ADR-0136-amendment — Foundry internal-only.** Superseded by ADR-0242 + this ADR.
- **ADR-0137 — Foundry bounded contexts.** Amended; BCs redistribute per §D-1.
- **ADR-0138 — Foundry six-path deprecation.** Amended; Strangler retargets per §D-9.
- **ADR-0139 — Agentic SLO-gated promotion.** Per-environment dev-tools-cell SLO gates.
- **ADR-0144 — EU AI Act graduated-risk tier model.** Self-modification AI surfaces declare tier.
- **ADR-0145 — Inter-microservice communication reform.** Per ADR-0145 invariant 1, each caller emits its own audit seal — drives the evidence BC dissolution.
- **ADR-0150 — Cedar policy engine.** Cedar gates self-modification per §D-8.
- **ADR-0174 — FinOps cost tagging + sustainability.** Self-modification cycles tagged per cost center.
- **ADR-0183 — Cedar app authz + Kyverno admission.** Both gates apply.
- **ADR-0192 — Milvus vector DB.** Moves to Intelligence per §D-9.
- **ADR-0200 — Wasmtime substrate.** Moves to Intelligence per §D-9.
- **ADR-0211 — In-house tech stack preference.** Self-hosted substrate consistent with Class C in-house mandate.
- **ADR-0212 — Buildability doctrine.** Self-modification deliverables are buildability artifacts.
- **ADR-0220 — Consumer Intelligence Substrate.** Inheritance for Intelligence; eval / guardrails / providers move in per §D-1.
- **ADR-0221 — Agentic dev pipeline hardening.** §M-04 audience field removal per ADR-0242 + this ADR.
- **ADR-0239 — Foundry scope clarification.** Amended by this ADR.
- **ADR-0240 — Sovereign cloud per regional pack.** dev-tools-cell-N respects per-pack overlays.
- **ADR-0241 — DR + BC portfolio policy.** dev-tools-cell-prod is T2; bootstrap cell is T0 (per-bootstrap-only).
- **ADR-0242 — `oyatie`-is-a-tenant doctrine** (keystone #1 — companion). `oyatie.foundry.*` sub-scope per §D-2.
- **ADR-0243 — Cedar as universal gate** (keystone #2 — companion). Self-modification gates are Cedar.
- **ADR-0244 — Tenant as universal scoping primitive** (keystone #3 — companion).
- **ADR-0245 — Substrate vs Product layering** (keystone #4 — companion). Substrate definition criteria; foundry's dissolution per substrate-criteria.
- **ADR-0246 — Policy-engine substrate promotion** (keystone #5 — companion). Receives autonomy-tier-gate fragments.
- **ADR-0248 — Amazon-shape cellular architecture** (keystone #7 — companion). Cell topology including dev-tools-cell-N.
- **ADR-0249 — Workflow Engine as universal orchestrator** (keystone — companion). Receives runtime + supervisor BC primitives.
- **ADR-0250 — audit-chain substrate promotion** (keystone — companion). Receives evidence BC primitives.
- **ADR-0251 — Compliance Pack + Cell Certification Levels** (keystone — companion). Dev-tools cells declare certification levels.
- **ADR-0255 — Intelligence substrate rewrite** (keystone — companion). Receives providers + guardrails + eval BC primitives.

### Auto-memory feedback

- `feedback_oyatie_is_a_tenant_doctrine` — applies; `oyatie.foundry.*` sub-scope.
- `feedback_cedar_as_universal_gate` — applies; self-modification gates are Cedar.
- `feedback_autonomous_implementation_artifacts` — NEW reinforcement; self-modification enables autonomous masterplan execution.
- `feedback_foundry_pipeline_canonical` — UPDATED; the canonical pipeline name is now "dev-tools workflow library" + the `oyatie.foundry.*` sub-scope rather than "Foundry pipeline" (the latter retained as informal language).
- `feedback_quality_performance_scalability_bar` — reinforced; matches every named hyperscaler self-hosting reference.
- `feedback_no_silent_regression` — reinforced; immutable workflow versions + signed Cedar fragments prevent silent change.
- `feedback_automate_everything` — reinforced; the dev-tools workflow library automates the platform's own maintenance.
- `feedback_clean_architecture_requirements` — applies; substrate-vs-product layering preserved.
- `feedback_workflow_studio_scope` — reinforced; Workflow Studio (consumer product) and dev-tools workflow library (oyatie tenant) share the same Workflow Engine substrate.
- `feedback_flat_product_catalog` — preserved.

---

## Appendix A: Hyperscaler-pattern attribution matrix

Per the audit pattern established in the keystone bundle, every
architectural decision in this ADR is attributed to a named
hyperscaler pattern + source + anti-pattern avoided.

| Decision section | Hyperscaler pattern (named) | Source citation | Anti-pattern avoided |
|---|---|---|---|
| D-1 (Foundry-as-µservice dissolves; BCs redistribute) | "Substrate Primitive De-duplication" | AWS Bedrock + Step Functions + IAM as separate substrates; GCP Vertex AI + Workflows + IAM Conditions; Azure AI Foundry + Logic Apps + Azure Policy | "Primitive Duplication Across Sibling µservices" — same primitive in multiple µservices drifts over time |
| D-2 (workflow library replaces Foundry-as-product) | "Internal-CI as Tenant-of-Platform" | AWS internal CI as AWS IAM principal; Stripe internal CI as Stripe tenant; Google internal CI as Borg tenant | "Internal-CI as Separate µservice" — bypass paths + drift loops |
| D-3 (self-modification mechanics under Cedar gates) | "Policy-Gated Reflective Tower" | AWS Verified Permissions self-modification; Anthropic Console self-modification under Cedar-equivalent gates | "Unrestricted Reflection" — system can authorise any modification of itself |
| D-4 (bootstrap Tier 0 minimum) | "Audited Bootstrap Replay" | rustc stage0; Kubernetes kubeadm certificate chain; Certificate Transparency log bootstrap | "Untraceable Genesis" — original deployment lacks audit trail |
| D-5 (5-stage bootstrap sequence) | "Multi-Stage Self-Host Bootstrap" | rustc stage0/1/2 chain; LFS Chapter 5/6 cross-compile pattern; kubeadm Phase 1/2 design | "Big-Bang Bootstrap" — single step from zero to steady-state with no audit |
| D-6 (dev/staging/prod self-modification environments) | "Three-Tier CD with Auto-Rollback" | AWS internal dev/gamma/prod fleets; Google canary → fleet rollout; Spinnaker bake-to-prod pipeline | "Single-Environment Self-Modification" — production drift without rehearsal |
| D-7 (workflow versioning + atomic swap) | "Immutable Workflow Version Pinning" | Temporal workflow versioning; AWS Step Functions versioning; GCP Workflows versioning | "Mutable Workflow Drift" — running instances change underfoot |
| D-8 (Cedar fragment gating self-modification) | "Policy-Engine-Gated Self-Modification" | AWS Verified Permissions + KMS chain; Sigstore signed-fragment provenance | "Trust-On-First-Use Self-Modification" — first publisher acquires unbounded modification rights |
| D-9 (artifact migration plan) | "Lossless Substrate Distribution" | rustc stage0/1/2 maintains historical artifact lineage; Nix flakes preserve input provenance | "Lossy Migration" — primitives drop during reorganisation |
| D-10 (Hermes name retirement) | "Inherited-Term Decomission" | Glossary discipline; canonical-glossary enforcement | "Vestigial Terminology Sprawl" — inherited names persist with no canonical meaning |
| D-11 (CI lanes for self-modification) | "Coverage-Required Self-Modification" | Google SRE Workbook ch. 4 (SLO coverage); AWS Config conformance packs | "Untested Self-Modification Surface" — paths discovered missing in production |
| D-12 (failure modes + bootstrap-replay runbook) | "Documented Recovery Procedure" | AWS Builder's Library "Static Stability"; NIST SP 800-34 contingency planning; Google SRE incident-response runbooks | "Tribal Recovery Knowledge" — only one engineer knows how to recover |

---

## Appendix B: Worked example — first PR review workflow running on a self-hosted platform

To illustrate the doctrine concretely, here is a worked example of
the first end-to-end PR review running on a self-hosted oyatie
platform, post-Stage 4.

**Scenario:** An oyatie engineer (principal `oyatie.dev.jasonlee`,
per ADR-0242 §D-8 sandbox tenant) submits PR #200 to the source
repository hosted on the platform's self-hosted git µservice
(post-bootstrap; the git host migrated from external Tier 0 GitHub
Enterprise to the self-hosted git µservice during Stage 3 of the
bootstrap). PR #200 modifies a Cedar fragment in
`microservices/policy-engine/fragments/baseline/` (specifically,
the `oyatie-finance-permits.cedar` fragment to add a new permit for
an expense-approval workflow).

**Step-by-step trace:**

**T+0s — PR submission.** The engineer pushes commits to a feature
branch on the self-hosted git µservice + opens PR #200 via the git
µservice's UI. The git µservice emits a `PullRequestOpened` event
on the platform's internal event bus (via Workflow Engine signal API).

**T+1s — Workflow Engine receives signal.**
`oyatie.foundry.pr-review` workflow instance #15234 starts on
dev-tools-cell-prod. Workflow Engine evaluates Cedar fragment
`oyatie-pr-review-permits.cedar` for the action
`Workflow::Action::StartInstance`:

```cedar
permit (
  principal == "system:workflow-engine-trigger",
  action == Workflow::Action::StartInstance,
  resource == Workflow::"oyatie.foundry.pr-review"
)
when {
  context.trigger_event_class == "PullRequestOpened"
  && context.tenant_id == "oyatie"
};
```

Decision: Permit. Audit row emitted to `oyatie.foundry` stream.

**T+2s — PR review fan-out.** `oyatie.foundry.pr-review` reads the PR
metadata via the git µservice's gRPC API; identifies that PR #200
touches a Cedar fragment in `policy-engine/fragments/baseline/`; this
triggers the *fragment-author review path* per multispectrum review
v2.4.0. The workflow fans out child-workflow instances for facets:

- `oyatie.foundry.multispectrum-review.F1` (correctness)
- `oyatie.foundry.multispectrum-review.F2` (hyperscaler-fitness)
- `oyatie.foundry.multispectrum-review.F5` (security)
- `oyatie.foundry.multispectrum-review.F6` (performance)
- `oyatie.foundry.multispectrum-review.F7` (supply chain)
- `oyatie.foundry.multispectrum-review.A1` (own-policy-adherence-naming)
- `oyatie.foundry.multispectrum-review.A4` (architecture-adherence)
- `oyatie.foundry.multispectrum-review.A6` (schema-adherence)

Each child invokes Intelligence's LLM gateway (per ADR-0255) with a
facet-specific prompt + the PR diff + the contextual reading material
(prior fragments, ADR-0243 fragment lifecycle).

**T+30s — Facet verdicts collected.** Each facet workflow completes
within 30s p99 (the LLM round-trip dominates). Verdicts:

- F1: PASS — fragment intent matches frontmatter declaration.
- F2: PASS — fragment uses canonical Cedar idioms.
- F5: PASS — no privilege escalation introduced; `cedar-policy-analyzer`
  formal verification confirms the new permit cannot widen access.
- F6: PASS — evaluation cost analysis: < 0.1ms additional p99 per
  evaluation.
- F7: PASS — fragment dependencies signed by intermediate keys.
- A1: PASS — fragment_id follows BNF v4.1.
- A4: PASS — respects scope/overlay/pack layering.
- A6: PASS — Cedar entity types match Ontology Object Type definitions.

**T+35s — CI build + test.** In parallel,
`oyatie.foundry.ci-build-and-test` runs against PR #200:
`cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace -- -D warnings`, `cargo fmt --all --check`.
PASS within ~3 minutes.

**T+3m38s — eval-runner.** `oyatie.foundry.eval-runner` runs the
Cedar fragment against the eval corpus
`microservices/policy-engine/eval-corpora/baseline-fragment-eval.yaml`
which contains 100 test cases for the fragment. PASS rate: 100%.

**T+5m — pr-review aggregates.** All child verdicts + CI + eval are
PASS. `oyatie.foundry.pr-review` aggregates: verdict PASS.

**T+5m1s — Admission gate.** Workflow Engine evaluates Cedar fragment
`oyatie-merge-queue-admission.cedar`:

```cedar
permit (
  principal in Tenant::"oyatie".sub_scopes("foundry"),
  action == MergeQueue::Action::Admit,
  resource is PullRequest
)
when {
  resource.multispectrum_review_verdict == "PASS"
  && resource.ci_status == "GREEN"
  && resource.eval_runner_status == "PASS"
};
```

Decision: Permit. PR #200 admitted to the merge queue.

**T+5m3s — Merge queue.** `oyatie.foundry.merge-queue-fix-loop` is
already processing two earlier PRs (#198, #199) per ADR-0111 projected
merge state. PR #200 is enqueued behind them.

**T+12m — PR #198 merges.**

**T+18m — PR #199 merges.**

**T+24m — PR #200's turn.** The merge-queue-fix-loop rebases PR #200
onto the current `dev` HEAD; CI re-runs (the merge-queue-fix-loop
explicitly re-runs CI per ADR-0111); PASS within 3 minutes; PR #200
merges into `dev`.

**T+27m — Fragment publication.** Now that PR #200 is on `dev`,
`oyatie.foundry.fragment-author` workflow detects the new fragment
content in `microservices/policy-engine/fragments/baseline/`; invokes
`Cedar::Action::PublishFragment`:

```cedar
// policy-engine evaluates against oyatie-self-modification-permits.cedar
permit (
  principal in Tenant::"oyatie".sub_scopes("foundry"),
  action == Cedar::Action::PublishFragment,
  resource is CedarFragment
)
when {
  principal.is_automated_with_baseline_signed_workflow
};
```

The `is_automated_with_baseline_signed_workflow` predicate verifies
that the workflow's *workflow definition* is signed by the org-baseline-
key (`oyatie.foundry.fragment-author` workflow v3 is signed; check
passes). Decision: Permit. Fragment published.

**T+27m5s — Hot-reload.** Per ADR-0243 §D-10, the fragment
publication emits `CedarFragmentPublished` on the per-cell
`fragment-reload` topic. All per-cell evaluators receive within 5s;
recompile + atomic swap; new fragment in force across the platform.

**T+30m — First evaluation under new fragment.** An expense-approval
workflow on the finance team's tenant invokes
`oyatie.finance.approver` to approve an expense. Cedar evaluation
now includes the new permit; Permit. Action proceeds.

**T+30m1s — Audit row emitted.** The Cedar evaluation row carries:

```json
{
  "event_class": "CedarEvaluation",
  "evaluation_id": "<uuid>",
  "principal": "oyatie.finance.approver",
  "action": "Finance::Action::ApproveExpense",
  "resource": "FinanceExpense::id/1234",
  "tenant_id": "oyatie",
  "decision": "Permit",
  "applied_fragments": [
    "baseline/oyatie-finance-permits.cedar:v5"
  ],
  "determining_policies": ["oyatie-finance-permits:permit-approve-expense"],
  "evaluation_ms": 0.3,
  "audit_emitted_at": "2026-XX-XX...",
  "audit_stream": "oyatie.finance"
}
```

The `:v5` suffix in the applied fragment reference reflects the new
version just published by PR #200's workflow.

**What this trace demonstrates:**

1. **Self-modification cycle completed end-to-end** without any human
   intervention beyond the engineer's initial PR submission.
2. **Every step was Cedar-gated.** PR admission, CI execution, eval
   execution, merge queue admission, fragment publication, fragment
   activation, the eventual use of the new fragment — every one
   evaluated through policy-engine.
3. **Every step emitted to audit-chain.** A regulator can replay the
   full sequence from PR submission to first use of the new fragment
   from a single audit query.
4. **The self-hosting property is visible.** The git host, CI runner,
   eval runner, merge queue, Cedar evaluator, audit chain, workflow
   engine — all on the platform itself. No external CI involved.
5. **Multispectrum review v2.4.0 ran inline.** Eight facets per
   ADR-0243 §D-8 evaluated the fragment change.
6. **Workflow versioning + atomic swap** at the fragment level
   (publication is atomic) and at the workflow level (the
   `oyatie.foundry.fragment-author` workflow itself was running on a
   pinned version throughout).
7. **Tier 0 was not invoked.** No external service was called; the
   bootstrap minimum was not exercised.

This is the steady-state shape that the doctrine produces. Every
subsequent self-modification cycle follows the same trace, with the
specific workflow / fragment / substrate change varying.

## Naming justification

Per `feedback_naming_justification`: every new name introduced by this ADR carries a one-line BNF v4.1 + ADR-0105 13-layer conformance justification.

| Name | Layer (ADR-0105) | BNF v4.1 segments | Justification |
|---|---|---|---|
| `oya-check-self-modification-permitted` | N/A (check-family) | `check`.`self-modification-permitted` | CI fitness-check per ADR-0105 Amendment 2; verifies every self-modifying workflow has an explicit Cedar permit fragment + human-approval requirement per §D; rejects workflows performing `PublishFragment`/`ActivateFragment`/`DeploySubstrateVersion` without the fragment. |
| `oya-check-bootstrap-tier-coherence` | N/A (check-family) | `check`.`bootstrap-tier-coherence` | CI fitness-check; verifies bootstrap-minimum components are external (Tier 0) and everything else self-hosts; refuses manifests declaring external bootstrap dependencies outside Tier 0. |
| `oya-check-foundry-dissolution-complete` | N/A (check-family) | `check`.`foundry-dissolution-complete` | CI fitness-check; verifies `microservices/foundry/` is deleted and no `oya-foundry-*` crate names remain after the Foundry-dissolution migration per §D. |
| `oya-check-workflow-version-immutability` | N/A (check-family) | `check`.`workflow-version-immutability` | CI fitness-check; verifies published workflow versions are immutable (content hash matches at publish time; no post-publish mutation). |
| `oya-check-cedar-self-modification-permit-present` | N/A (check-family) | `check`.`cedar-self-modification-permit-present` | CI fitness-check; verifies `policy-engine/fragments/oyatie-self-modification-permits.cedar` exists, is signed by org-baseline-key, and includes the meta-permit clause. |
| `oya-check-hermes-name-retired` | N/A (check-family) | `check`.`hermes-name-retired` | CI fitness-check; glossary-compliance sub-check; refuses "Hermes" string in any artifact outside this ADR's history section. |
| `oya-check-dev-tools-cell-environments` | N/A (check-family) | `check`.`dev-tools-cell-environments` | CI fitness-check; verifies dev-tools-cell-dev/-staging/-prod exist with correct Cedar permit configuration. |
| `oyatie.foundry.ci-agent` (principal sub-scope) | N/A (sub-scope path) | N/A | Principal sub-scope path per ADR-0244 §D-2 dotted-path convention; `oyatie` = platform tenant; `foundry` = foundry subsystem; `ci-agent` = CI agent role. Reserved under `oyatie.*` namespace per ADR-0242. |
| `oyatie.foundry.meta-trust-root` | `trust-anchor` | `oyatie`.`foundry.meta-trust-root` | Offline HSM trust anchor for self-modification witness; `trust-anchor` layer per ADR-0105; distinct from `workflow-publisher` to break the F5-247-01 circular-predicate exploit (per ADR-0293). |
| `oyatie.foundry.meta-trust-root-attestor` | `trust-anchor` | `oyatie`.`foundry.meta-trust-root-attestor` | Day-to-day automation requesting witness signatures; `trust-anchor` layer; never holds key material (per ADR-0293). |
| `oyatie.foundry.bootstrap-runner` | `trust-anchor` | `oyatie`.`foundry.bootstrap-runner` | Stage-1 external CI principal bound via SPIFFE; ephemeral; authority revoked at T+8h (per ADR-0295). |
| `oyatie.foundry.bootstrap-ca` | `trust-anchor` | `oyatie`.`foundry.bootstrap-ca` | One-shot offline-rooted CA; key destroyed post Stage-1 (per ADR-0295). |
| `oyatie.foundry.bootstrap-kill-switch-publisher` | `automation` | `oyatie`.`foundry.bootstrap-kill-switch-publisher` | Automation publishing the T+8h kill-switch Cedar fragment; does not hold trust-chain key material (per ADR-0295). |

---

## Change log

- **2026-05-20 (Wave-3-A cross-reference wiring):** Applied four surgical amendments per ADR-0293, ADR-0294, ADR-0295:
  - §D-2: Added principal rows for `oyatie.foundry.meta-trust-root`, `oyatie.foundry.meta-trust-root-attestor`, `oyatie.foundry.bootstrap-runner`, `oyatie.foundry.bootstrap-ca`, `oyatie.foundry.bootstrap-kill-switch-publisher`.
  - §D-4 step 0.4: Changed Shamir parameters from M=3,N=5 to M=5,N=9 across ≥3 jurisdictions for the meta-trust-root key and any other trust-chain anchor; M=3,N=5 retained for tenant-scoped operational keys only.
  - §D-5 Stage 1: Added SPIFFE-pinned runner identity, sigstore cosign attestation requirement for all Stage-1 artifacts, ≤8h hard bootstrap budget, T+8h Cedar kill-switch fragment pre-load per ADR-0295.
  - §D-5 Stage 1.10: Added out-of-band council-security two-member manual hash verification handoff per ADR-0295 §D-7.
  - §D-8: Replaced single-predicate `is_automated_with_baseline_signed_workflow` with conjunctive form `is_automated_with_baseline_signed_workflow && is_attested_by_meta_trust_root` per ADR-0293 §D-4.

*End of ADR-0247.*
