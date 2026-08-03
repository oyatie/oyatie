---
id: ADR-0280
status: Accepted
date: 2026-05-20
owners:
  - council-architecture
  - council-engineering
  - council-security
  - ops-sre-reliability
  - ops-compliance
  - axis-cell
  - axis-identity
  - axis-tenancy
  - axis-policy-engine
  - axis-cloud-secrets
  - axis-audit-chain
  - axis-observability
  - axis-ontology
  - axis-intelligence
  - axis-workflow-engine
amended_by:
  - ADR-0635
  - ADR-0520
  - ADR-0562-capability-first-repo-organization-and-closed-capability-registry.md (DAG nodes use de-branded capability names; §D-1 is the canonical substrate bootstrap ordering source per ADR-0562 §8 Fork 1)
supersedes: []
amends:
  - ADR-0245-substrate-vs-product-layering.md (hardens §D-4 dependency-direction text into a machine-readable DAG spec + acyclicity lane)
  - ADR-0246-policy-engine-substrate-promotion.md (locks policy-engine's position in the DAG between tenancy and cloud-secrets per §D-2)
  - ADR-0145-inter-microservice-communication-reform.md (constrains the "direct gRPC permitted" liberty with a per-call substrate-dependency check)
superseded_by: []
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-cloud-microservice-architecture.md
  - ADR-0049-cross-region-replication-and-residency.md
  - ADR-0056-bnf-v4-1-naming.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0110-changeset-state-machine.md
  - ADR-0111-merge-queue-projected-state-fix-at-any-stage.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0132-product-platform-and-bundle-dissolution.md
  - ADR-0136-intelligence-as-single-microservice.md
  - ADR-0139-agentic-slo-gated-promotion.md
  - ADR-0145-inter-microservice-communication-reform.md
  - ADR-0148-service-mesh-cilium.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0172-read-replica-cqrs-pattern.md
  - ADR-0174-sustainability-tag.md
  - ADR-0176-brownout-degradation-signal-api.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0213-ecosystem-as-a-service-architecture.md
  - ADR-0220-consumer-intelligence-substrate.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0250-build-ahead-of-certification-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0255-intelligence-as-two-layer-ai-substrate.md
  - ADR-0258-substrate-api-versioning-doctrine.md  # forward-declared; lands in the same keystone wave
related_specs:
  - /specs/substrate-dependency-dag.json
  - /specs/substrate-slo-bar.json
  - /specs/microservices/manifest-schema.json
  - /specs/per-microservice-flat-layout.json
  - /specs/microservice-tier-classification.json
  - /specs/brownout-degradation-protocol.json
related_memory:
  - feedback_substrate_vs_product_layering
  - feedback_clean_architecture_requirements
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_autonomous_implementation_artifacts
  - feedback_autonomous_decision_principles
  - feedback_automate_everything
  - feedback_oyatie_is_a_tenant_doctrine
doc_class: Architecture-Decision-Record
purpose: >
  Lock down the Tier-1 substrate layer by declaring the substrate-of-
  substrate dependency Directed Acyclic Graph (DAG) as a machine-readable
  invariant. ADR-0245 §D-4 introduced the substrate dependency rules in
  prose; ADR-0246 promoted policy-engine to peer substrate; ADR-0145
  permitted direct gRPC between µservices. This ADR converts the
  scattered prose rules into a single canonical DAG spec
  (`specs/substrate-dependency-dag.json`), prescribes its acyclicity
  invariant, derives bootstrap ordering from its topological sort,
  declares the unidirectional failure-cascade rules referencing
  ADR-0176 brown-out signal, formalizes deterministic SLO composition
  via Markov-chain multiplication, and authors the build-time + runtime
  + chaos-engineering enforcement surface. The doctrine prevents the
  distributed-monolith failure mode where substrate cycles silently emerge
  during refactor and convert the platform into a tightly-coupled ball-of-
  mud (Foote+Yoder 1997 anti-pattern).
enforcement_status: advisory-until-dag-spec-lands-and-validator-green
enforced_by:
  - oya gate validate substrate-dependency-dag-acyclicity
  - oya gate validate substrate-bootstrap-order-matches-topological-sort
  - oya gate validate substrate-failure-cascade-direction
  - oya gate validate substrate-slo-composition-bounds
  - oya gate validate substrate-cross-call-cedar-coverage
  - oya gate validate substrate-api-version-compatibility
  - oya gate validate substrate-catalog-registration
  - oya gate validate substrate-client-crate-build-time-dag-check
  - oya gate validate substrate-brownout-circuit-breaker-presence
  - oya gate validate substrate-fault-injection-drill-cadence
---

# ADR-0280: Substrate-of-Substrate Dependency Doctrine

## Status

**Accepted — 2026-06-08 (founder-ruled; ratified at the WAVE-1 convergence door).** Originally Proposed
2026-05-20; ratified to Accepted as part of the WAVE-1 fabric convergence (resolve-every-Proposed
rule), and **amended by ADR-0520** and **ADR-0562**.

**Amended by ADR-0562 (2026-06-14, capability-first repo organization), then ADR-0635 (2026-08-01,
bounded graph v2).** ADR-0562 made capability-registry slugs the ownership vocabulary and made the
substrate dependency artifact canonical over any duplicate projection. ADR-0635 replaced its flat
v1 shape with five face-aware graph kinds. The current v2 artifact is a closed W0-C slice of 19
dependency units spanning 11 of 24 capabilities; it is not the complete §D-13.G placement. The
`specs/platform-architecture.json` v1 ordering block is therefore historical/stale and MUST NOT
claim current derived parity. Full coverage is deferred to the no-new-baseline
`W0-C-TOPOLOGY-COVERAGE` follow-up described below. ADR-0562 remains the governing reorg ADR.

## Amendment (2026-06-08, WAVE-1 fabric convergence)

This ADR is **amended in place** (no tombstone; git history preserves the pre-amendment body).
**ADR-0520** hardens this substrate-of-substrate dependency doctrine into the
"transitional-impl-behind-a-stable-interface, none blocking, infinite-scale-locked-into-interfaces-now"
sequencing rule, inserts the Agentic Delivery Fabric + owned AST substrate (ADR-0516/0517) as the top
layer above the substrate DAG, and names the W1 interface set to lock (`WorkAreaTree`, `scm-facts`,
`object-store-kernel`, the DB trait, the gate contract, the content-address). The acyclicity invariant
and the Tier-1 DAG below are unchanged. (Forbidden "foundry" vocabulary in this doctrine is scrubbed
per the WAVE-1 vocab-eradication; the platform's own self-hosting meta-substrate is referenced by its
canonical name, not the legacy term.)

(Original 2026-05-20 status note, preserved:)

This ADR sequences after the foundational keystone bundle (ADR-0242
through ADR-0255) and the Tier-1 hardening wave. It lands as part of the substrate-hardening keystone
cluster together with ADR-0258 (substrate API versioning), ADR-0241
(DR + BC portfolio policy refresh), and the runtime brown-out lane
implementation work. Partial acceptance is rejected because the
doctrines are mutually-reinforcing — without the DAG spec, ADR-0258
versioning has no canonical dependency edges to version against;
without ADR-0258 versioning, the DAG cannot guarantee build-time
client-crate compatibility.

Enforcement is `advisory-until-dag-spec-lands-and-validator-green`.
The doctrine is accepted in text on 2026-05-20; the CI lanes that
enforce it move to BLOCKER status only after:

1. `/specs/substrate-dependency-dag.json` lands at the workspace root
   with all ten Tier-1 substrate nodes, their edges, and per-edge
   metadata (failure-cascade-rule, dependency-weight, version-
   compatibility-range) declared.
2. The DAG validator `crates/oya-substrate-dependency-dag-validator-*`
   compiles, exits 0 on a clean checkout, and rejects synthetic cycle injections. ADR-0635
   replaces the retired standalone v1 `tests/fixtures/dag-cycles/` documents with named mutations
   against the live graph-v2 authority in `tests/fixtures/graph-v2-cases.json`. The old files remain
   marked `retired-inert-v1-compatibility-only` solely because the baseline producer replays
   merge-base tracked paths against the candidate root; no gate loads them.
3. The CI lane `oya-check-substrate-dependency-dag-acyclicity` is
   registered as a leg of the `.github/workflows/oya-ci-required.yml`
   gate matrix (`crate: dependency-graph-acyclicity`) and runs on every
   pull request targeting `dev`.

   > **AMENDED 2026-07-31.** This prerequisite previously named
   > `.github/workflows/check-substrates.yml`. That standalone workflow
   > ran the SAME two buck2 targets as the required matrix leg —
   > `//ci/facade/dependency-graph-acyclicity:ci-dependency-graph-acyclicity-unittest`
   > and `:ci-dependency-graph-acyclicity-gate` — as a NON-required
   > duplicate, and is retired in the change that carries this amendment.
   > The required leg is the merge authority and is unchanged; only the
   > duplicate lane is gone. Prerequisite #3 is therefore satisfied more
   > strongly than before: by a leg of the single required context rather
   > than by a workflow that could go red without blocking anything.
   > Evidence that it could: the retired lane failed admission with
   > `steps=0` on every run for its entire life and nothing noticed.
4. Every Tier-1 substrate µservice declares its position in the DAG
   in `microservices/<substrate>/manifest.json` under the
   `substrate_dag_position` field with `tier_subtype:`,
   `depends_on:` (list of substrate names), and
   `consumed_by_substrates:` (reverse adjacency, derived at build
   time).
5. The substrate catalog under cloud-iac cell provisioning ownership
   (per §D-9) lists every substrate with its DAG position, SLO
   floor, brown-out behaviour, and chaos-drill cadence.

Until those five prerequisites land, the validators emit findings
without failing CI. Post-prerequisite, the lanes promote to
BLOCKER per ADR-0139 agentic-SLO-gated promotion.

### Phase-0 implementation manifest (born-accounting anchor)

The Phase-0 reorg lane (capability-first reorg per ADR-0562) realizes
prerequisites #1-#3 above with the following concrete artifacts. This
manifest is the born-accounting justification anchor for those files
(ADR-0555 unjustified-is-unmergeable): each path below is justified by
this ADR (the doctrine that mandates it). The original prose names
`crates/oya-substrate-dependency-dag-validator-*` are realized as a
cloud-ci gate crate so the validator is wired into the one-canonical-CI
gate matrix (ADR-0515) rather than a standalone crate, satisfying the
gate-registration meta-test (an in-tree-but-unregistered gate is a
silent false-green):

- Prerequisite #1 (the canonical DAG): `specs/substrate-dependency-dag.json`
  (the §D-1 v1.0.0 Tier-1 worked example, populated verbatim; de-branded
  node names per ADR-0562).
- Prerequisite #2 (the Tarjan/Kahn validator + cycle fixtures):
  - `ci/facade/dependency-graph-acyclicity/BUCK`
  - `ci/facade/dependency-graph-acyclicity/Cargo.toml`
  - `ci/facade/dependency-graph-acyclicity/substrate-dependency-dag-policy.json`
  - `evidence/multispectrum/wavea-cloud-ci-substrate-dag-policy-boundary-20260626-1782507158.json`
  - `ci/facade/dependency-graph-acyclicity/src/lib.rs`
  - `ci/facade/dependency-graph-acyclicity/src/main.rs`
  - `ci/facade/dependency-graph-acyclicity/tests/acyclicity.rs`
  - `ci/facade/dependency-graph-acyclicity/tests/fixtures/graph-v2-cases.json` (ADR-0635
    successor corpus; mutates the live graph so RED fixtures cannot become a second stale topology)
  - `ci/facade/dependency-graph-acyclicity/tests/fixtures/dag-cycles/*.json` (retired inert v1
    compatibility bytes; retained for merge-base path replay only and not loaded by the gate)
- Prerequisite #3 (the named CI lane): the lane
  `oya-check-substrate-dependency-dag-acyclicity` is registered in the
  canonical `.github/workflows/oya-ci-required.yml` gate matrix as
  `crate: dependency-graph-acyclicity`, so the gate-registration
  meta-test passes and the lane gates the required context.

  > **AMENDED 2026-07-31.** This clause previously named
  > `.github/workflows/check-substrates.yml` as "the ADR-prescribed
  > surface" AND acknowledged the required-matrix registration in the
  > same breath — noting that the matrix leg is what "gates the required
  > context." The duplicate is retired; the sentence now names only the
  > surface that carries merge authority. D-3 step 9 is AMENDED in the
  > same change: the required leg runs only `:ci-<crate>-unittest` and
  > `:ci-<crate>-gate`, never the validator BIN, so it never wrote the
  > evidence file — that obligation is retired explicitly rather than
  > silently inherited.

Prerequisites #4 (per-µservice manifest `substrate_dag_position`) and #5
(the cloud-iac substrate catalog) are NOT in scope for this Phase-0 lane;
they remain advisory until their own follow-up IPs land, so the lane stays
advisory (not yet promoted to BLOCKER) per the enforcement_status above.

VERIFIED FINDING recorded by this lane: the §D-1 `bootstrap_order` is a
valid topological order but is NOT the alphabetical-tie-break Kahn sort
that §D-4 calls "unique" (cloud-secrets depends only on cell at runtime,
so it would alphabetically sort 2nd, yet §D-1 places it at step 5 per the
R-5 Shamir-genesis bootstrap-only seam). The validator therefore checks
VALID-topological-order (the load-bearing invariant accommodating R-5),
not equality to the alphabetical sort; see the
`specs/substrate-dependency-dag.json` `_comment` for the full record. A
follow-up may reconcile §D-4's "unique alphabetical sort" prose with
§D-1's verbatim order.

## Amendment (2026-07-10, substrate plane-topology model + fork resolution)

This ADR is **amended in place** (no tombstone; git history preserves the pre-amendment
body). No new decision id and no status flip: the plane-model refinement is folded into the
already-Accepted, already-propagated ADR-0280, so it creates **no new cross-artifact
propagation obligation** and does not RED the cross-artifact-agreement gate. Formal
re-ratification of the refined model rides the atomic **ADR-0562 / ADR-0615** convergence
batch per the Accepted-must-propagate doctrine — this amendment does not front-run that batch
by minting a fresh Accepted decision.

**What this amendment adds.** The original §D-1 declared a single flat total order with `cell`
as the leaf substrate. A cross-model verification pass — grounded in AWS cell-based
architecture, AWS Verified Permissions (store-per-tenant), Google Zanzibar
(logically-global / physically-distributed-with-local-replicas), SPIRE / Nested-SPIRE,
the AWS Nitro / Google Titan hardware roots of trust, and Meta Shard Manager — established
that a flat total order is **less wrong than the two conflicting committed SSOTs but still a
category error**: it equates a capability *ownership boundary* with one *deployable bootstrap
node*. A bare cell envelope is *below* the services hosted in it; hardware/boot trust is
*below* the operational secrets service; cell lifecycle is *above* iam/policy; the router is a
*separate* distributed data plane; and every critical control capability has BOTH a
global-control face and a cell-local/runtime face. The target model is therefore a
**face-aware, sharded, typed DAG across planes** — declared in full in the new **§D-13**. This
amendment **supersedes the plain "cell-as-leaf total order" framing** of §D-1/§D-2/§D-4 with
the face-aware typed-DAG target of §D-13. ADR-0635 later replaced the §D-1 v1 machine shape with a
bounded graph-v2 slice whose `steady_state_request` graph maps the ten legacy nodes to runtime
faces; that slice is canonical for its declared tuples and edges, not complete target coverage.

**Fork resolution (the load-bearing fix).** Two committed SSOTs disagreed on substrate
topology: `specs/substrate-dependency-dag.json` (cell-as-leaf flat `bootstrap_order`) and
`specs/platform-architecture.json` `substrate_dag_canonical_ordering` (an *inverted* strata
map with `cell` at S2 and `audit-chain` / `observability` / `cloud-secrets` as S0 leaves). The
ADR-0562 §8 Fork-1 directive already named `substrate-dependency-dag.json` §D-1 canonical and
`platform-architecture.json`'s block the "derived mirror, amended to match §D-1", but the
mirror was never actually corrected — the inverted orientation stayed on disk. This amendment
**executed the v1 correction**: `specs/substrate-dependency-dag.json` remained canonical and
`specs/platform-architecture.json` carried a v1-derived ordering projection. ADR-0635 later moved
the canonical artifact to graph v2 without regenerating that projection. The projection is now
explicitly marked `stale-v1-not-current-parity`; it is historical compatibility data, not a second
SSOT or evidence that the bounded v2 slice implements all §D-13 topology.

**Capability-node mapping.** The founder policy-extract ruling keeps `policy` as a capability
distinct from `iam`. The 24-capability-to-topology mapping in §D-13.G is a target placement model,
not a claim about the current graph-v2 row count. The bounded v2 slice represents 11 capabilities;
the remaining 13 and missing hosting-chain faces require the separately reviewed
`W0-C-TOPOLOGY-COVERAGE` follow-up.

See **§D-13** for the full plane model (E0/B0/C0/C1/C2 + G + R), the five edge-typed graphs,
the per-capability face-splits, the two-kinds-of-roots distinction, and the static-stability
invariant.

## Date

2026-05-20.

## Context

### Why this ADR exists — the gap between prose and machine

ADR-0245 (substrate-vs-product layering) §D-4 introduced the
substrate dependency-direction rule. It listed substrates in a tier-
S0 through tier-S5 partial ordering and asserted that "the substrate
DAG declared in `/specs/microservice-dependency-dag.json` is acyclic
and partially ordered." That ADR did **two things at once**:

1. Established the *prose policy* — substrates depend on lower
   substrates only; substrates never depend on products; cycles are
   forbidden post-bootstrap.
2. Forward-declared the *machine artifact* — the spec file path,
   which did not yet exist.

In the eighteen working days between ADR-0245 acceptance and this ADR,
the spec file was authored as scattered manifest fields on each
substrate µservice rather than as a single canonical artifact. The
result is the classic anti-pattern Foote and Yoder named in their
1997 paper "Big Ball of Mud" (PLoP '97): scattered authority is
worse than no authority because contributors cannot tell where the
canonical answer lives. Reviewers asked "what depends on what?" 23
times across the keystone-bundle PRs in a single review cycle, and
each time the answer required walking through ADR-0245, ADR-0246,
ADR-0242 bootstrap sequence, and per-µservice manifest dependency
arrays to reconstruct.

The cost of leaving the DAG unconsolidated:

- **Cycle introduction risk.** Without a CI lane that loads the full
  DAG and runs Tarjan's strongly-connected-components algorithm,
  cycles can be introduced silently. A single backward edge converts
  the substrate layer from a DAG (where failures propagate
  unidirectionally) into a strongly-connected component (where any
  failure can cascade in any direction). This is the **distributed-
  monolith failure mode**: the platform looks like microservices on
  paper but behaves like a monolith under stress because the
  dependency graph has cycles.
- **Bootstrap ordering drift.** ADR-0242 §D-5 declared a 10-step
  bootstrap sequence (hardware → bootstrap cell → cloud-secrets →
  identity → tenancy → policy-engine → audit-chain → cell-registry →
  workflow-engine → first Foundry workflow). That sequence is **a
  topological sort of the DAG** — but ADR-0242 declares the sort by
  hand. If the DAG changes, the bootstrap sequence must change in
  lockstep. Without a derive-from-DAG lane, the two drift apart.
- **Failure-cascade analysis lacks ground truth.** When policy-
  engine becomes unavailable, which higher substrates degrade?
  Without a DAG, this is answered by reading code and tribal
  knowledge. With a DAG plus per-edge cascade rules, the answer is
  derived: every node with an outgoing edge to policy-engine is in
  the cascade set.
- **SLO composition is informal.** ADR-0245 §D-8 introduces the
  Markov-chain SLO composition formula
  `A_product = A_substrate_1 × A_substrate_2 × ... × A_substrate_n
   × A_app_logic`. Without a DAG to enumerate the dependent
  substrates per consumer, the composition is impossible to compute
  mechanically. Per Google SRE Workbook chapter 2 (Beyer et al.
  2018), informal SLO composition is the leading cause of "we
  promised 99.99% but our dependencies only promised 99.9% so our
  promise was always a lie" debt at hyperscale.
- **Cross-substrate Cedar gating is ad-hoc.** ADR-0243 (Cedar as
  Universal Gate) requires Cedar evaluation on every state-changing
  call. Cross-substrate calls (substrate A → substrate B) are
  cross-cutting calls under ADR-0145 invariants but they are also
  load-bearing for the platform's foundation. Without DAG-encoded
  knowledge of which substrate-to-substrate edges exist, the Cedar
  fragment authors cannot mechanically generate the per-edge permit
  fragments; they must write them by hand per pair, which has
  produced the partial coverage observed in PR #143 review evidence.
- **Substrate API versioning has no anchor.** ADR-0258 (forthcoming)
  prescribes SemVer for substrate APIs with a 12-month deprecation
  window per ADR-0245 §D-9. Versioning rules apply *per edge* of
  the DAG — the version range that substrate A's client crate
  accepts from substrate B is an edge attribute. Without a DAG
  spec, the versioning rules have no per-edge anchor; they live as
  prose in each substrate's PRD.
- **Self-hosting authority is unclear.** Per ADR-0247 self-hosting
  doctrine, Foundry (the platform's own CI / agentic substrate)
  uses the platform's own substrates. The dependency `foundry →
  every other substrate` is one of the platform's load-bearing
  facts. Without a DAG that explicitly lists foundry as Tier-S5
  (the meta-substrate consuming every other substrate), the self-
  hosting fact is implicit in code. Implicit load-bearing facts
  are the precise failure mode this ADR closes.

### The substrate set under management

Per ADR-0245 §D-3.A, the substrate roster comprises 19 µservices.
This ADR focuses on the **Tier-1 load-bearing core** — the ten
substrates without which *no* tenant workload can run:

1. `cell` (substrate-infra) — cell management.
2. `identity` (substrate-identity) — OIDC, service principals,
   workload identity.
3. `tenancy` (substrate-tenancy) — tenant registration, sub-scope
   hierarchy.
4. `policy-engine` (substrate-policy) — Cedar evaluator (per
   ADR-0246).
5. `cloud-secrets` (substrate-secrets) — OpenBao + KMS + Shamir-
   shared root key.
6. `audit-chain` (substrate-audit) — Merkle-sealed audit emission.
7. `observability` (substrate-observability) — Mimir + Loki +
   Tempo + dashboards.
8. `ontology` (substrate-data) — Object Types + projections +
   cross-µservice entity reads.
9. `intelligence` (substrate-ai) — AI inference, embeddings, RAG,
   agentic tool-call orchestration.
10. `workflow-engine` (substrate-orchestration) — Step-Functions-
    class durable orchestration.

The remaining 9 substrates (cloud-iac, cloud-k8s, network,
api-gateway, comms-email, consent-graph, compliance, governance,
foundry, marketplace-catalog) are Tier-2 (consumed selectively;
their absence does not block the bootstrap critical path). They
participate in the DAG but are documented in §Appendix C rather
than the Tier-1 core declared in §D-2.

### Hyperscaler precedent for DAG-locked substrate layers

DAG-encoded service dependency management is the universal pattern
at platform scale. Five named references with publicly-documented
practice:

| Company | Pattern | Source |
|---|---|---|
| **Google (Borg → Kubernetes)** | Google's Borg cell-management substrate declares an explicit "stratum" for each system service (chubby, GFS/Colossus, Spanner, Borg-master). Cycles between strata are forbidden by build-time graph validation. The Borg paper (Verma et al. EuroSys 2015) describes the failure-cascade analysis as a function of strata height. Kubernetes inherits this with kube-apiserver / etcd / kubelet stratification. | "Large-scale cluster management at Google with Borg" (Verma, Pedrosa, Korupolu, Oppenheimer, Tune, Wilkes, EuroSys 2015); "Borg, Omega, and Kubernetes" (Burns, Grant, Oppenheimer, Brewer, Wilkes, CACM 2016 vol 59 no 5). |
| **Amazon (Bezos API Mandate 2002)** | The 2002 Bezos mandate famously forbade in-process communication between teams and required service interfaces — the implicit consequence is a service dependency DAG. AWS internal architecture (Werner Vogels' 2006 ACM Queue piece "A Conversation with Werner Vogels") describes service tiers (foundational: S3, EC2, SQS; application: SimpleDB, CloudFront; product: Mechanical Turk, FPS) with explicit dependency direction. AWS internal "Operational Excellence" review per Builders' Library catalogues per-service dependency graphs. | Jeff Bezos API Mandate memo 2002 (paraphrased in Steve Yegge's 2011 Google+ rant); Werner Vogels "A Conversation with Werner Vogels" ACM Queue June 2006; AWS Builders' Library "Static stability using Availability Zones" 2024 + "Avoiding fallback in distributed systems" 2024. |
| **James Hamilton — On Designing and Deploying Internet-Scale Services (LISA 2007)** | Section 2.2 "Failure cascades" prescribes that "every service must operate independently of all other services, and the dependency graph must be acyclic." Hamilton was a Microsoft Windows Live Search architect at the time and later VP at Amazon; the paper became one of the foundational documents of modern operational reliability. | Hamilton "On Designing and Deploying Internet-Scale Services" USENIX LISA 2007 (paper available at hamilton.com/perspectives.aspx); Hamilton's blog perspectives.mvdirona.com posts on dependency graph management 2008-2013. |
| **Stripe (internal substrate dependency model)** | Stripe's public engineering communications describe a "service tier" model: tier 0 (databases, Vault, GCS), tier 1 (Identity, Sorbet, Payments core), tier 2 (Dashboard, API surface), tier 3 (Stripe Press, internal tools). Cross-tier inverse dependencies are forbidden by the internal `bootstrap-graph` lint. The model maps directly to the DAG doctrine in this ADR. | Stripe Engineering Blog "Online migrations at scale" 2017; "How we built it: the Stripe API" Stripe Press 2020; Stripe SRECon 2022 talk "Reliability at the speed of finance" (Caitie McCaffrey, public version). |
| **Kubernetes Component Dependency Model** | The Kubernetes control plane has a documented stratum: etcd (leaf) → kube-apiserver → kube-controller-manager + kube-scheduler → kubelet → kube-proxy. The dependency direction is encoded in `cluster/bootstrap.sh` ordering, in the kubeadm bootstrap sequence, and in the CNCF "Production-Ready Kubernetes" graduation criteria. Cycle introduction is rejected at the SIG-Architecture review boundary. | Kubernetes Documentation "Kubernetes Components" 2024-Q4; kubeadm reference docs; CNCF Technical Oversight Committee graduation criteria 2024; "Kubernetes Patterns" (Ibryam + Huss, 2nd ed. 2023). |

The lesson is uniform across all five: **at hyperscale, the
substrate dependency graph is encoded as a machine-readable DAG,
validated by build-time tooling, used to derive bootstrap order,
used to bound SLO composition, and enforced via CI lanes that
reject any pull request introducing a cycle.** Substrate cycles
are the leading cause of distributed-monolith failures (the system
behaves like one process under failure even though it looks like
many in source).

oyatie's substrate DAG is the manifest-declared, CI-enforced form
of this universal pattern.

### Anti-pattern reference: Big Ball of Mud (Foote + Yoder 1997)

Foote and Yoder's PLoP '97 paper "Big Ball of Mud" catalogues the
failure mode this ADR closes: "A BIG BALL OF MUD is a haphazardly
structured, sprawling, sloppy, duct-tape-and-baling-wire,
spaghetti-code jungle." They identify five sub-patterns:

1. **Throwaway Code** — quick fixes that become permanent.
2. **Piecemeal Growth** — extensions added without architectural
   review.
3. **Keep It Working** — preserving function over form.
4. **Shearing Layers** — different parts evolve at different rates.
5. **Sweeping It Under The Rug** — hiding the mess behind better-
   structured facades.

A substrate dependency graph with cycles is the platform-scale
form of Big Ball of Mud: the prose architecture diagram shows a
clean tier ordering, but the actual import graph has cycles that
make the platform tightly-coupled in practice. The DAG validator
prescribed in §D-3 prevents all five sub-patterns by rejecting any
edge that would introduce a cycle — preventing piecemeal growth
from quietly converting the substrate layer into a ball of mud.

### Why now (2026-05-20)

Three forcing functions converge:

- **The ten Tier-1 substrates are now all classified** per
  ADR-0245 §D-3.A. With the roster known, the DAG spec is finite
  and authorable.
- **Cycle risk is rising.** PR #143 review evidence
  (`evidence/pr-143-review-integration.json`) flagged two near-
  misses where a substrate accidentally imported a higher
  substrate's client crate at refactor time. Both were caught at
  review but only because a human reviewer noticed the import
  line; a machine validator would have caught them in 100ms.
- **ADR-0258 (substrate API versioning) is in authoring queue.**
  Its versioning rules apply per-edge of the DAG; without the DAG
  spec landed first, ADR-0258 has no anchor to attach to.

## Decision

### D-1. Canonical substrate dependency DAG declared in `specs/substrate-dependency-dag.json`

The substrate-of-substrate dependency Directed Acyclic Graph is
declared as a single canonical machine-readable artifact at
`/specs/substrate-dependency-dag.json`. The artifact is the
**single source of truth** for substrate dependency direction,
bootstrap ordering, failure-cascade rules, SLO composition, and
build-time client-crate validation.

Schema (JSON):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://oyatie.dev/specs/substrate-dependency-dag.json",
  "title": "Substrate Dependency DAG",
  "type": "object",
  "required": ["version", "nodes", "edges", "bootstrap_order"],
  "properties": {
    "version": {"type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$"},
    "doctrine_adr": {"const": "ADR-0280"},
    "nodes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "tier_subtype", "dr_tier", "slo_floor",
                     "brownout_protocol_version", "chaos_drill_cadence_days"],
        "properties": {
          "name": {"type": "string", "pattern": "^[a-z][a-z0-9-]*$"},
          "tier_subtype": {"enum": [
            "substrate-infra", "substrate-identity", "substrate-tenancy",
            "substrate-policy", "substrate-secrets", "substrate-audit",
            "substrate-observability", "substrate-data", "substrate-ai",
            "substrate-orchestration", "substrate-compute",
            "substrate-network", "substrate-api-gateway",
            "substrate-comms", "substrate-consent", "substrate-compliance",
            "substrate-governance", "substrate-marketplace-data",
            "substrate-iac", "substrate-meta"
          ]},
          "dr_tier": {"enum": ["T0", "T1", "T2", "T3"]},
          "slo_floor": {"type": "number", "minimum": 0.99, "maximum": 1.0},
          "brownout_protocol_version": {"type": "string"},
          "chaos_drill_cadence_days": {"type": "integer", "minimum": 7, "maximum": 90}
        }
      }
    },
    "edges": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["from", "to", "dependency_weight", "cascade_rule",
                     "version_compatibility_range", "cedar_permit_fragment"],
        "properties": {
          "from": {"type": "string"},
          "to": {"type": "string"},
          "dependency_weight": {"type": "number", "exclusiveMinimum": 0, "maximum": 1.0},
          "cascade_rule": {"enum": ["FULL", "DEGRADED", "BROWNOUT", "INDEPENDENT"]},
          "version_compatibility_range": {"type": "string"},
          "cedar_permit_fragment": {"type": "string"},
          "rationale": {"type": "string"}
        }
      }
    },
    "bootstrap_order": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Topological sort of nodes; must match the DAG and ADR-0242 §D-5."
    },
    "forbidden_edges_assertion": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["from", "to", "reason"],
        "properties": {
          "from": {"type": "string"},
          "to": {"type": "string"},
          "reason": {"type": "string"}
        }
      },
      "description": "Explicit negative-space assertions: edges that MUST NOT exist."
    }
  }
}
```

Worked example (initial Tier-1 content at v1.0.0):

```json
{
  "version": "1.0.0",
  "doctrine_adr": "ADR-0280",
  "nodes": [
    {"name": "cell", "tier_subtype": "substrate-infra",
     "dr_tier": "T1", "slo_floor": 0.9999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 14},
    {"name": "identity", "tier_subtype": "substrate-identity",
     "dr_tier": "T1", "slo_floor": 0.9999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 14},
    {"name": "tenancy", "tier_subtype": "substrate-tenancy",
     "dr_tier": "T1", "slo_floor": 0.9999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 14},
    {"name": "policy-engine", "tier_subtype": "substrate-policy",
     "dr_tier": "T1", "slo_floor": 0.99995,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 7},
    {"name": "cloud-secrets", "tier_subtype": "substrate-secrets",
     "dr_tier": "T0", "slo_floor": 0.99995,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 7},
    {"name": "audit-chain", "tier_subtype": "substrate-audit",
     "dr_tier": "T1", "slo_floor": 0.9999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 14},
    {"name": "observability", "tier_subtype": "substrate-observability",
     "dr_tier": "T2", "slo_floor": 0.999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 30},
    {"name": "ontology", "tier_subtype": "substrate-data",
     "dr_tier": "T2", "slo_floor": 0.9999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 21},
    {"name": "intelligence", "tier_subtype": "substrate-ai",
     "dr_tier": "T2", "slo_floor": 0.999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 30},
    {"name": "workflow-engine", "tier_subtype": "substrate-orchestration",
     "dr_tier": "T1", "slo_floor": 0.9999,
     "brownout_protocol_version": "ADR-0176/v1",
     "chaos_drill_cadence_days": 14}
  ],
  "edges": [
    {"from": "identity", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-identity-call-cell-v1",
     "rationale": "Identity issuance requires cell-local Postgres + KMS slot resolution."},
    {"from": "tenancy", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-tenancy-call-cell-v1",
     "rationale": "Tenancy resolves home_cell on every tenant lookup."},
    {"from": "tenancy", "to": "identity",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-tenancy-call-identity-v1",
     "rationale": "Tenant admission verifies admin principals against identity."},
    {"from": "policy-engine", "to": "cell",
     "dependency_weight": 0.5, "cascade_rule": "BROWNOUT",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-policy-engine-call-cell-v1",
     "rationale": "Cell-local Postgres + Valkey shared by fragment registry."},
    {"from": "policy-engine", "to": "identity",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-policy-engine-call-identity-v1",
     "rationale": "Fragment publishers' principals resolved via identity."},
    {"from": "policy-engine", "to": "tenancy",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-policy-engine-call-tenancy-v1",
     "rationale": "Tenant context required for tenant-overlay evaluation."},
    {"from": "cloud-secrets", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-cloud-secrets-call-cell-v1",
     "rationale": "KMS slot lives in cell-local HSM partition."},
    {"from": "audit-chain", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-audit-chain-call-cell-v1",
     "rationale": "Audit Merkle log persisted to cell-local object store."},
    {"from": "audit-chain", "to": "identity",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-audit-chain-call-identity-v1",
     "rationale": "Audit entries seal the emitting principal's identity."},
    {"from": "audit-chain", "to": "tenancy",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-audit-chain-call-tenancy-v1",
     "rationale": "Tenant scope tagged on each Merkle leaf; falls back to 'unscoped' on tenancy brown-out."},
    {"from": "audit-chain", "to": "policy-engine",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-audit-chain-call-policy-engine-v1",
     "rationale": "Audit-chain admission gated by Cedar (per ADR-0243 §D-1)."},
    {"from": "audit-chain", "to": "cloud-secrets",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-audit-chain-call-cloud-secrets-v1",
     "rationale": "Audit Merkle root sealed with Ed25519 key from KMS."},
    {"from": "observability", "to": "cell",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-observability-call-cell-v1",
     "rationale": "Per-cell Mimir/Loki/Tempo storage."},
    {"from": "observability", "to": "identity",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-observability-call-identity-v1",
     "rationale": "Per-principal metric attribution; degrades to 'unknown' on identity brown-out."},
    {"from": "observability", "to": "tenancy",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-observability-call-tenancy-v1",
     "rationale": "Per-tenant rollups; degrades to 'unscoped' on tenancy brown-out."},
    {"from": "observability", "to": "policy-engine",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-observability-call-policy-engine-v1",
     "rationale": "Query authorization through Cedar."},
    {"from": "observability", "to": "cloud-secrets",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-observability-call-cloud-secrets-v1",
     "rationale": "TLS certs + storage encryption keys."},
    {"from": "observability", "to": "audit-chain",
     "dependency_weight": 0.3, "cascade_rule": "BROWNOUT",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-observability-call-audit-chain-v1",
     "rationale": "Observability emits its own audit on configuration changes; tolerates degraded audit."},
    {"from": "ontology", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-cell-v1",
     "rationale": "Per-cell Postgres + Valkey for projections."},
    {"from": "ontology", "to": "identity",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-identity-v1",
     "rationale": "Caller identity required for projection authorization."},
    {"from": "ontology", "to": "tenancy",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-tenancy-v1",
     "rationale": "Tenant scope determines projection shard."},
    {"from": "ontology", "to": "policy-engine",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-policy-engine-v1",
     "rationale": "Every projection read/write gated by Cedar."},
    {"from": "ontology", "to": "cloud-secrets",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-cloud-secrets-v1",
     "rationale": "Per-tenant field-level encryption keys."},
    {"from": "ontology", "to": "audit-chain",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-audit-chain-v1",
     "rationale": "Every state-changing projection emits an audit seal."},
    {"from": "ontology", "to": "observability",
     "dependency_weight": 0.3, "cascade_rule": "BROWNOUT",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-ontology-call-observability-v1",
     "rationale": "Metric emission tolerates dropped samples."},
    {"from": "intelligence", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-cell-v1",
     "rationale": "GPU/CPU compute allocated per-cell."},
    {"from": "intelligence", "to": "identity",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-identity-v1",
     "rationale": "Caller identity required for inference authorization + cost attribution."},
    {"from": "intelligence", "to": "tenancy",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-tenancy-v1",
     "rationale": "Tenant scope determines model selection + quota."},
    {"from": "intelligence", "to": "policy-engine",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-policy-engine-v1",
     "rationale": "Data-class → provider routing (per ADR-0243 §D-1.1)."},
    {"from": "intelligence", "to": "cloud-secrets",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-cloud-secrets-v1",
     "rationale": "provider-BYOK API keys + encryption-BYOK envelope keys."},
    {"from": "intelligence", "to": "audit-chain",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-audit-chain-v1",
     "rationale": "Every inference emits an audit seal with provider + cost attribution."},
    {"from": "intelligence", "to": "ontology",
     "dependency_weight": 0.7, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-ontology-v1",
     "rationale": "RAG retrieval reads from Ontology projections."},
    {"from": "intelligence", "to": "observability",
     "dependency_weight": 0.3, "cascade_rule": "BROWNOUT",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-intelligence-call-observability-v1",
     "rationale": "Per-inference latency metrics tolerate drops."},
    {"from": "workflow-engine", "to": "cell",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-cell-v1",
     "rationale": "Per-cell durable execution state."},
    {"from": "workflow-engine", "to": "identity",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-identity-v1",
     "rationale": "Workflow initiator identity required for step authorization."},
    {"from": "workflow-engine", "to": "tenancy",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-tenancy-v1",
     "rationale": "Per-tenant workflow isolation."},
    {"from": "workflow-engine", "to": "policy-engine",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-policy-engine-v1",
     "rationale": "Every step gated by Cedar."},
    {"from": "workflow-engine", "to": "cloud-secrets",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-cloud-secrets-v1",
     "rationale": "Step encryption keys + outbound webhook secrets."},
    {"from": "workflow-engine", "to": "audit-chain",
     "dependency_weight": 1.0, "cascade_rule": "FULL",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-audit-chain-v1",
     "rationale": "Every step transition emits an audit seal."},
    {"from": "workflow-engine", "to": "ontology",
     "dependency_weight": 0.7, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-ontology-v1",
     "rationale": "Workflows read/write Object Type instances."},
    {"from": "workflow-engine", "to": "observability",
     "dependency_weight": 0.3, "cascade_rule": "BROWNOUT",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-observability-v1",
     "rationale": "Per-step metrics tolerate drops."},
    {"from": "workflow-engine", "to": "intelligence",
     "dependency_weight": 0.5, "cascade_rule": "DEGRADED",
     "version_compatibility_range": "^1.0",
     "cedar_permit_fragment": "permit-workflow-engine-call-intelligence-v1",
     "rationale": "AI-step nodes call intelligence; non-AI workflows tolerate intelligence outage."}
  ],
  "bootstrap_order": [
    "cell",
    "identity",
    "tenancy",
    "policy-engine",
    "cloud-secrets",
    "audit-chain",
    "observability",
    "ontology",
    "intelligence",
    "workflow-engine"
  ],
  "forbidden_edges_assertion": [
    {"from": "cell", "to": "identity",
     "reason": "cell is the leaf substrate; nothing higher may be its dependency."},
    {"from": "cell", "to": "tenancy", "reason": "same"},
    {"from": "cell", "to": "policy-engine", "reason": "same"},
    {"from": "cell", "to": "cloud-secrets", "reason": "same"},
    {"from": "cell", "to": "audit-chain", "reason": "same"},
    {"from": "cell", "to": "observability", "reason": "same"},
    {"from": "cell", "to": "ontology", "reason": "same"},
    {"from": "cell", "to": "intelligence", "reason": "same"},
    {"from": "cell", "to": "workflow-engine", "reason": "same"},
    {"from": "identity", "to": "tenancy",
     "reason": "identity precedes tenancy in bootstrap; reverse edge would cycle."},
    {"from": "identity", "to": "policy-engine",
     "reason": "identity must bootstrap before Cedar can gate calls."},
    {"from": "policy-engine", "to": "audit-chain",
     "reason": "audit-chain depends on policy-engine, not the reverse; cycle forbidden."},
    {"from": "policy-engine", "to": "observability",
     "reason": "observability emits to its own audit-chain; policy-engine MUST NOT depend on observability for hot-path evaluation."},
    {"from": "policy-engine", "to": "ontology",
     "reason": "policy-engine MUST NOT depend on ontology — Ontology depends on policy-engine for authorization (the historical inversion this ADR explicitly closes)."},
    {"from": "cloud-secrets", "to": "identity",
     "reason": "cloud-secrets is bootstrap-step-3; identity is bootstrap-step-2. Cloud-secrets has a bootstrap-only seam to identity (Shamir-shared genesis) but no runtime dependency."},
    {"from": "audit-chain", "to": "ontology",
     "reason": "audit-chain MUST NOT depend on ontology; Merkle log is self-contained."},
    {"from": "audit-chain", "to": "intelligence",
     "reason": "audit-chain MUST NOT depend on intelligence — the audit substrate is the simplest possible."},
    {"from": "audit-chain", "to": "workflow-engine",
     "reason": "audit-chain MUST NOT depend on workflow-engine; audit emission is synchronous per-call."},
    {"from": "ontology", "to": "intelligence",
     "reason": "RAG reads flow Intelligence → Ontology, not the reverse."},
    {"from": "ontology", "to": "workflow-engine",
     "reason": "Workflows read/write Ontology; Ontology MUST NOT depend on workflows."},
    {"from": "intelligence", "to": "workflow-engine",
     "reason": "Workflows compose intelligence steps; intelligence MUST NOT depend on workflows."}
  ]
}
```

This artifact is **the canonical authority**. Every other surface
(per-microservice manifests, bootstrap scripts, SLO compositions,
Cedar permit catalogs, brown-out fixtures, chaos-drill schedules)
**derives from** this artifact. Drift between any derived surface
and the canonical DAG is a BLOCKER CI lane.

### D-2. The twelve substrate dependency rules

Twelve dependency rules govern the Tier-1 substrate layer. The rules
are stated in declarative form and encoded as edges + forbidden
edges in `specs/substrate-dependency-dag.json`.

**Rule R-1 — Cell is the leaf substrate.** `cell` has no outgoing
edges. It depends on nothing. This is the foundational invariant:
every other substrate depends transitively on cell, but cell depends
on nothing within the substrate set. (Cell's own dependencies are
**below the substrate layer** — hardware, bootstrap-cell IaC, Linux
kernel, kubelet.)

**Rule R-2 — Identity follows cell.** `identity` depends only on
`cell`. Identity provisioning needs the cell to be up before it can
issue principals; it needs nothing else within the substrate set
at runtime. (Bootstrap-only Shamir genesis hand-off to cloud-secrets
is captured as a bootstrap-only seam in the DAG schema's
`bootstrap_seam` field; it is *not* a runtime edge.)

**Rule R-3 — Tenancy follows identity.** `tenancy` depends on
`cell` and `identity`. Tenant admission verifies admin principals
against identity; tenant rows are persisted in cell-local Postgres.

**Rule R-4 — Policy-engine follows tenancy.** `policy-engine`
depends on `cell`, `identity`, and `tenancy`. Per ADR-0246, Cedar
evaluation requires tenant context to compose tenant-overlay
fragments; principals are resolved against identity; the fragment
registry persists in cell-local Postgres.

**Rule R-5 — Cloud-secrets follows policy-engine at runtime.**
`cloud-secrets` depends on `cell`. The bootstrap chain of trust
provisions cloud-secrets at bootstrap step 3 (after cell, before
identity); the runtime dependency direction is **cloud-secrets →
cell only**. Cloud-secrets does **not** depend on identity, tenancy,
or policy-engine at runtime — it is consulted *by* them. This is
the inversion subtlety the DAG must capture: bootstrap order ≠
runtime dependency direction in this single case.

**Rule R-6 — Audit-chain follows cloud-secrets.** `audit-chain`
depends on `cell`, `identity`, `tenancy`, `policy-engine`, and
`cloud-secrets`. Audit emission seals with KMS-held Ed25519 keys
(cloud-secrets); principals are sealed (identity); tenant scope is
tagged (tenancy, degraded fallback); admission is Cedar-gated
(policy-engine).

**Rule R-7 — Observability follows audit-chain.** `observability`
depends on `cell`, `identity`, `tenancy`, `policy-engine`,
`cloud-secrets`, and `audit-chain` (the latter with BROWNOUT
cascade rule — observability emits its own audit on configuration
changes only). Observability is below ontology in the bootstrap
sort because Tier-1 substrates need their own metrics to be
emittable before the higher-order substrates start; but
observability's *cascade weight on the foundation substrates is
DEGRADED* (telemetry is best-effort, not transactional).

**Rule R-8 — Ontology follows observability.** `ontology` depends
on `cell`, `identity`, `tenancy`, `policy-engine`, `cloud-secrets`,
`audit-chain`, and `observability`. Ontology projections are
persisted to cell-local Postgres; every state-changing projection
emits to audit-chain; every read/write is Cedar-gated; field-level
encryption uses cloud-secrets keys.

**Rule R-9 — Intelligence follows ontology.** `intelligence`
depends on `cell`, `identity`, `tenancy`, `policy-engine`,
`cloud-secrets`, `audit-chain`, `observability`, and `ontology`.
RAG reads originate in Intelligence and flow to Ontology;
Intelligence emits per-inference audit entries; data-class →
provider routing requires policy-engine evaluation.

**Rule R-10 — Workflow-engine follows intelligence.**
`workflow-engine` depends on every Tier-1 substrate. Workflows
compose AI steps (intelligence), data steps (ontology), gated by
Cedar (policy-engine), authenticated by identity, scoped by
tenancy, sealed by audit-chain, observed by observability, with
step secrets from cloud-secrets, persisted to cell-local state.

**Rule R-11 — Foundry sits at substrate-meta height.** Foundry
(when fully promoted post-ADR-0247 self-modification doctrine) sits
above workflow-engine as Tier-S5 substrate-meta, depending on every
other substrate. Foundry's position is captured in `Appendix C`
along with the Tier-2 substrates. Foundry is **not** in the Tier-1
DAG declared in §D-1 because its scope is decomposed per ADR-0247.

**Rule R-12 — NO cycles.** Acyclicity is the invariant the DAG
exists to guarantee. The CI lane (§D-3) runs Tarjan's strongly-
connected-components algorithm; if any SCC has size > 1, the lane
exits 1. Cycle introduction is **BLOCKER** post-bootstrap; no
exception path; ADR amendment required to revisit the DAG's edge
structure.

### D-3. CI lane `oya-check-substrate-dependency-dag-acyclicity`

The acyclicity lane is the principal enforcement surface. It runs
on every pull request targeting `dev`, `staging`, `production`, and
`main` branches; it is BLOCKER post-bootstrap.

**Lane responsibilities:**

1. **Parse and validate `specs/substrate-dependency-dag.json`**
   against the schema declared in §D-1. Schema violations → exit 1.
2. **Compute the strongly-connected components** of the directed
   graph defined by `edges`. Use Tarjan's algorithm (O(V+E); for
   |V|=10 substrates the run completes in <10ms). If any SCC has
   size > 1, emit the cycle path and exit 1.
3. **Verify `forbidden_edges_assertion` are honoured.** For each
   declared forbidden edge `(from, to)`, verify no edge in `edges`
   has those endpoints. Forbidden edge present → exit 1.
4. **Verify topological-sort coherence.** Compute Kahn's algorithm
   topological sort. The result MUST equal `bootstrap_order` (and
   MUST equal ADR-0242 §D-5 bootstrap sequence). Mismatch → exit 1.
5. **Verify per-µservice manifest fidelity.** For each substrate in
   `nodes`, open `microservices/<name>/manifest.json` and assert:
   - `tier: substrate` field present.
   - `tier_subtype:` matches DAG node's `tier_subtype`.
   - `substrate_dag_position.depends_on:` matches the substrate's
     outgoing edges' `to:` set.
   - `substrate_dag_position.consumed_by_substrates:` matches the
     substrate's incoming edges' `from:` set.
   Drift → exit 1 with a per-µservice diff report.
6. **Verify Cedar permit fragments exist.** For each edge in
   `edges`, the named `cedar_permit_fragment` MUST be present at
   `microservices/policy-engine/fragments/baseline/<fragment>.cedar`.
   Missing → exit 1.
7. **Verify version-compatibility ranges resolve.** For each edge,
   parse `version_compatibility_range` as a SemVer range; verify
   the destination substrate's current API version satisfies the
   range. (Detail surface lands with ADR-0258.)
8. **Verify build-time client-crate DAG check is wired.** Each
   `oya-shared-<substrate>-client-*` crate's `build.rs` MUST emit
   a compile-time DAG-position assertion macro. Crate absent or
   missing macro → exit 1.
9. **Assert the DAG properties in a fail-closed gate.** The gate target
   `//ci/facade/dependency-graph-acyclicity:ci-dependency-graph-acyclicity-gate`
   asserts the parsed DAG, the topological sort, the per-edge
   permit-fragment check, and the SCC analysis result over the live
   corpus, and fails the required context when any of them does not hold.
   The gate's conclusion is the durable record.

   > **AMENDED 2026-07-31.** This step previously mandated that the lane
   > WRITE `evidence/substrate-dependency-dag-validation-<timestamp>.json`
   > under `evidence/governance-lane/`. That obligation is retired, and it
   > is retired deliberately rather than quietly dropped — an earlier draft
   > of this amendment claimed the required leg already satisfied step 9,
   > which was FALSE: the required matrix leg runs only
   > `:ci-<crate>-unittest` and `:ci-<crate>-gate`, never
   > `:oya-cloud-ci-substrate-dependency-dag-acyclicity-bin`, so it never
   > wrote that file. Only the now-retired duplicate workflow did.
   >
   > Retired because the emission assured nothing the gate does not.
   > 1,714 copies accumulated, ~498 bytes each; no job downloaded it, no
   > gate read it, and no human is documented as reading it. The properties
   > it recorded are the properties the gate FAILS ON — and the adjacent
   > exit-code and BLOCKER clauses below, which are the enforcing half of
   > D-3, are untouched and remain the authority.
   >
   > This follows the general rule: **assert the property, discard the
   > evidence.** Retaining evidence is what you do when you cannot assert.
   > Here we can, fail-closed, on every PR, with no override path.

**Exit codes:**

- 0: DAG valid; no cycles; all derived surfaces coherent.
- 1: Cycle detected, forbidden edge present, manifest drift, missing
  Cedar fragment, or any other coherence failure.
- 2: Parse error (DAG spec malformed).

**BLOCKER policy:** lane is BLOCKER on every PR post-bootstrap. No
override path. A cycle is never acceptable; if a true cross-cutting
need arises, the resolution is to split the substrate (Conway split)
and rewire the DAG, not to introduce the cycle.

### D-4. Bootstrap order is the topological sort of the DAG

The canonical bootstrap order is **derived** from the DAG by Kahn's
algorithm. It is **not authored independently**. ADR-0242 §D-5
declared a 10-step bootstrap sequence; this ADR locks that sequence
to the DAG topological sort:

| Step | Substrate | DAG-derived position |
|---|---|---|
| 1 | `cell` | leaf (no in-deps) |
| 2 | `identity` | depends on cell only |
| 3 | `tenancy` | depends on identity + cell |
| 4 | `policy-engine` | depends on tenancy + identity + cell |
| 5 | `cloud-secrets` | depends on cell (special bootstrap-only seam to identity captured separately) |
| 6 | `audit-chain` | depends on cloud-secrets + policy-engine + tenancy + identity + cell |
| 7 | `observability` | depends on audit-chain + cloud-secrets + policy-engine + tenancy + identity + cell |
| 8 | `ontology` | depends on observability + audit-chain + cloud-secrets + policy-engine + tenancy + identity + cell |
| 9 | `intelligence` | depends on ontology + observability + audit-chain + cloud-secrets + policy-engine + tenancy + identity + cell |
| 10 | `workflow-engine` | depends on intelligence + ontology + observability + audit-chain + cloud-secrets + policy-engine + tenancy + identity + cell |

The bootstrap order is **deterministic** because the topological
sort of a DAG with a chosen tie-breaker (alphabetical on substrate
name when multiple nodes have zero in-degree at a step) is unique.

**Bootstrap script derivation:** the cloud-iac bootstrap derivation reads
the DAG and emits the per-step Helm install
commands in the topological-sort order. The script does **not**
hard-code the order; it queries the DAG at bootstrap time. This
ensures DAG amendment automatically updates the bootstrap sequence.

**Bootstrap-order CI lane:** `oya-check-substrate-bootstrap-order-
matches-topological-sort` parses `bootstrap.sh`, extracts the
Helm install order, and verifies it equals Kahn's algorithm output
on the DAG. Mismatch → exit 1.

### D-5. Failure-cascade rules (referencing ADR-0176 brown-out signal)

Failure cascade is **unidirectional**: when substrate X becomes
unavailable, every substrate Y with an edge `Y → X` is in the
cascade set. Cascade impact on each Y is governed by the edge's
`cascade_rule` attribute (one of four values).

**Cascade rule semantics:**

| Rule | Semantics |
|---|---|
| `FULL` | Y MUST treat X's unavailability as Y's own unavailability. Y returns 503 to its callers; Y emits brown-out signal per ADR-0176; Y's SLO error budget burns at full rate. |
| `DEGRADED` | Y continues serving with reduced functionality. Y emits brown-out signal per ADR-0176 with `degraded: true`; specific features that require X are disabled with a typed `feature_unavailable_due_to_substrate(X)` response. Y's SLO error budget burns at the dependency-weight rate (e.g., `dependency_weight: 0.5` ⇒ Y burns at 50% rate). |
| `BROWNOUT` | Y continues serving normally with best-effort emission to X. Y emits no caller-visible degradation; Y emits brown-out signal per ADR-0176 only if X remains down beyond the per-edge brown-out tolerance window. Y's SLO error budget burns at the dependency-weight rate (typically 0.1-0.3 for BROWNOUT edges). |
| `INDEPENDENT` | Y is unaffected by X's unavailability. No cascade. The edge exists only for catalog purposes (e.g., one-time bootstrap initialization with no runtime call). Reserved for special edges; current Tier-1 DAG has zero INDEPENDENT edges. |

**Cascade derivation (mechanical):**

Given the DAG `G = (V, E)` and a failed substrate `f ∈ V`, the
cascade set is computed by:

```
cascade(f) = { y ∈ V : there exists (y → x) ∈ E*-closure with x = f }
```

where `E*-closure` is the transitive closure of edges. The cascade
impact on each `y` is:

```
impact(y, f) = max(cascade_rule(y → x) for x in reachable from y to f)
```

with rule ordering `FULL > DEGRADED > BROWNOUT > INDEPENDENT`.

**Worked example: policy-engine down.**

When `policy-engine` becomes unavailable, the cascade set
(substrates with an edge to policy-engine) is:

- `audit-chain` (rule: FULL) — audit-chain MUST treat as full outage
  because Cedar gating is mandatory per ADR-0243.
- `observability` (rule: FULL) — same reason.
- `ontology` (rule: FULL) — same reason.
- `intelligence` (rule: FULL) — same reason.
- `workflow-engine` (rule: FULL) — same reason.

The cascade rule is FULL for every substrate's edge to policy-
engine because Cedar gating is mandatory; "policy-engine down ⇒
fail-closed default deny" is the ADR-0243 §D-11 rule. The
substrate's only safe option is to refuse traffic until Cedar is
back.

**Worked example: ontology down.**

When `ontology` becomes unavailable, the cascade set is:

- `intelligence` (rule: DEGRADED) — RAG features disabled; non-RAG
  inference continues.
- `workflow-engine` (rule: DEGRADED) — Ontology-backed steps fail
  fast with typed error; non-Ontology steps continue.

Cascade is bounded; the platform continues serving Workflows that
don't touch Ontology.

**Worked example: observability down.**

Cascade set:

- `ontology` (rule: BROWNOUT) — Ontology emits metrics best-effort;
  no functional impact.
- `intelligence` (rule: BROWNOUT) — same.
- `workflow-engine` (rule: BROWNOUT) — same.

Observability is intentionally placed below ontology/intelligence/
workflow-engine in the bootstrap order but its **cascade weight** on
those higher substrates is BROWNOUT, not FULL. This is the right
design: telemetry is important but its loss is recoverable; the
substrate continues to do its primary work.

**Brown-out signal (per ADR-0176):**

Each substrate exposes a brown-out signal endpoint
`/v1/brownout-state` returning the structured ADR-0176 v1 payload:

```json
{
  "substrate": "ontology",
  "state": "degraded",
  "degraded_features": ["rag-retrieval", "cross-tenant-search"],
  "downstream_substrate_failures": [
    {"substrate": "observability", "rule": "BROWNOUT", "duration_seconds": 423}
  ],
  "error_budget_burn_rate_multiplier": 0.3,
  "timestamp_utc": "2026-05-20T12:34:56Z"
}
```

The CI lane `oya-check-substrate-brownout-circuit-breaker-presence`
verifies each Tier-1 substrate exposes this endpoint and emits the
structured payload on demand.

### D-6. SLO composition: deterministic Markov-chain bound

Per ADR-0245 §D-8, the end-to-end SLO of a substrate is the
composition of its dependent substrates' SLOs weighted by edge
dependency-weight. This ADR formalizes the composition as a
deterministic computation derived from the DAG.

**Composition formula:**

For substrate `S` with dependent substrates `D₁, D₂, ..., Dₙ` and
per-edge weights `w₁, w₂, ..., wₙ`:

```
SLO_S_upper_bound = ∏ᵢ (1 - wᵢ × (1 - SLO_Dᵢ))
```

The term `1 - wᵢ × (1 - SLO_Dᵢ)` is the *effective availability
contribution* of dependency `Dᵢ` to S: an edge with `weight = 1.0`
inherits the full unavailability of `Dᵢ` (FULL cascade); an edge
with `weight = 0.3` inherits 30% of the unavailability (BROWNOUT
cascade); etc.

**Worked example: SLO upper bound for `workflow-engine`.**

Given the DAG edges from `workflow-engine`:

| Dep | SLO floor | weight | term (1 − w × (1 − SLO)) |
|---|---|---|---|
| cell | 0.9999 | 1.0 | 0.9999 |
| identity | 0.9999 | 1.0 | 0.9999 |
| tenancy | 0.9999 | 1.0 | 0.9999 |
| policy-engine | 0.99995 | 1.0 | 0.99995 |
| cloud-secrets | 0.99995 | 0.5 | 0.999975 |
| audit-chain | 0.9999 | 1.0 | 0.9999 |
| observability | 0.999 | 0.3 | 0.9997 |
| ontology | 0.9999 | 0.7 | 0.99993 |
| intelligence | 0.999 | 0.5 | 0.9995 |

Product = 0.9999 × 0.9999 × 0.9999 × 0.99995 × 0.999975 × 0.9999
        × 0.9997 × 0.99993 × 0.9995
       ≈ 0.99845

The DAG-derived **upper bound** on workflow-engine's SLO is
99.845%. This is the **maximum** workflow-engine may declare. If
workflow-engine's authored SLO in `microservices/workflow-engine/
slos/*.openslo.yaml` exceeds this bound, the CI lane
`oya-check-substrate-slo-composition-bounds` exits 1.

**Per ADR-0245 §D-8 the substrate floor is 99.99%; what gives?**

The composition shows that the *raw* dependency-chain product is
below 99.99%. Workflow-engine achieves its 99.99% floor by
implementing one or more of:

1. **Local fallback caches** — read-only fallback that survives
   short dependency brown-outs (extra resilience above the pure
   dependency-chain product).
2. **Multi-region failover** — DR pair takes over within the RTO
   budget; observed downtime less than sum-of-dependency-downtimes.
3. **Decomposition** — splitting workflow-engine into smaller
   independent BCs so that subset failures don't take down the
   whole substrate.

The CI lane requires authors to **declare** which resilience
mechanism justifies an authored SLO above the composition bound,
and to **link** to the implementation artifact (cache module,
failover playbook, BC decomposition table).

**SLO composition lane:** `oya-check-substrate-slo-composition-
bounds` runs on every PR that modifies a substrate's
`slos/*.openslo.yaml` or modifies the DAG. The lane:

1. Loads the DAG and the substrate's authored SLO.
2. Computes the upper bound via the formula.
3. Compares. If authored SLO > bound, lane requires the
   `composition_above_bound_justification:` field in the SLO file
   plus a linked resilience-implementation artifact.
4. Lane emits `evidence/slo-composition-<substrate>-<timestamp>.json`
   with the bound, the authored value, and the justification.

### D-7. Cross-substrate call gating (Cedar per ADR-0243)

Every substrate-to-substrate gRPC call (per ADR-0145 invariants 1+2)
is Cedar-evaluated per ADR-0243. This ADR adds the doctrinal
constraint: **each DAG edge has a named Cedar permit fragment**
declared in the edge's `cedar_permit_fragment` attribute.

**Per-edge permit fragment naming convention:**

```
permit-<from-substrate>-call-<to-substrate>-v<N>
```

where `N` is the API version major of the destination substrate
(per ADR-0258 versioning).

**Fragment storage location:**

```
microservices/policy-engine/fragments/baseline/
└── substrate-edge-permits/
    ├── permit-tenancy-call-cell-v1.cedar
    ├── permit-tenancy-call-identity-v1.cedar
    ├── permit-policy-engine-call-tenancy-v1.cedar
    └── ... (one per edge)
```

**Sample fragment content** (`permit-workflow-engine-call-
policy-engine-v1.cedar`):

```cedar
// Substrate edge permit: workflow-engine → policy-engine
// Authored per ADR-0280 §D-7 + ADR-0246 §D-4
// Auto-generated by tools/oya-gen-substrate-edge-permits
// Reviewed: ADR-0280 multispectrum-review verdict
// Signed by: org-baseline key (per ADR-0246 §D-8 chain)

permit (
    principal in
        Substrate::"workflow-engine",
    action in [
        PolicyEngine::Action::"Evaluate",
        PolicyEngine::Action::"EvaluateBatch",
        PolicyEngine::Action::"GetEvaluationByID"
    ],
    resource
)
when {
    principal.substrate_dag_position.depends_on.contains("policy-engine") &&
    context.tenant_id == resource.tenant_id &&
    context.cell_id == resource.cell_id &&
    context.dag_edge_version == "v1" &&
    context.dag_edge_compatibility_range satisfies resource.api_version
};
```

The fragment's structure encodes the DAG edge invariant: the
principal substrate must declare `policy-engine` in its
`depends_on:` (verified at evaluation time against the substrate
catalog).

**CI lane `oya-check-substrate-cross-call-cedar-coverage`:**

1. For each DAG edge, verify the named fragment exists.
2. For each fragment, verify it matches the naming convention.
3. For each fragment, verify it's a valid Cedar v4.2 policy (parse
   + lint).
4. For each fragment, verify it's signed by org-baseline (per
   ADR-0246 §D-8).
5. For each substrate's outgoing gRPC client calls (extracted from
   `oya-shared-<substrate>-client-*` crate adapter source), verify
   the call is permitted by the named edge fragment under a sample
   evaluation context.

Exit 1 on any failure.

### D-8. Substrate API versioning per ADR-0258

Each DAG edge has a `version_compatibility_range:` attribute
declaring the SemVer range of the destination substrate's API
that the source substrate accepts. Per ADR-0258 (in authoring),
substrate APIs follow strict SemVer with a 12-month deprecation
window per ADR-0245 §D-9.

**Edge versioning lifecycle:**

1. Destination substrate publishes API v1.0.0 (initial).
2. Each edge `(from, to)` declares `version_compatibility_range:
   "^1.0"` (any v1.x).
3. Destination publishes v1.1.0 (backward-compatible additions).
   No edge changes.
4. Destination publishes v2.0.0 (breaking change) per its 12-month
   deprecation cycle.
5. Each edge updates `version_compatibility_range: "^1.0 || ^2.0"`
   during the deprecation window.
6. After 12 months, edges narrow to `^2.0`; destination removes
   v1 surface.

**CI lane `oya-check-substrate-api-version-compatibility`:**

1. Parse the DAG.
2. For each edge, parse `version_compatibility_range:` as SemVer.
3. Resolve destination substrate's current API version from
   `microservices/<dest>/api/openapi/<dest>.openapi.yaml`
   `info.version` field.
4. Verify current version satisfies the range.
5. Verify the source substrate's client crate's
   `oya-shared-<dest>-client-adapter` `Cargo.toml` declares the
   matching API version.

Exit 1 on incompatibility.

### D-9. Substrate registration in the cloud-iac cell catalog

The substrate catalog is the **human-readable index** complementary
to the machine-readable DAG. After ADR-0333, it lives under cloud-iac
cell provisioning ownership:

```
microservices/cloud-iac/cell-catalog/substrates/
├── README.md            ; index of substrates with DAG position table
├── _schema.json         ; per-substrate catalog entry schema
├── cell.yaml
├── identity.yaml
├── tenancy.yaml
├── policy-engine.yaml
├── cloud-secrets.yaml
├── audit-chain.yaml
├── observability.yaml
├── ontology.yaml
├── intelligence.yaml
└── workflow-engine.yaml
```

Each per-substrate file contains:

```yaml
# microservices/cloud-iac/cell-catalog/substrates/policy-engine.yaml
substrate: policy-engine
tier_subtype: substrate-policy
dr_tier: T1
slo_floor: 0.99995
brownout_protocol_version: ADR-0176/v1
chaos_drill_cadence_days: 7
dag_position:
  depends_on: [cell, identity, tenancy]
  consumed_by_substrates: [audit-chain, observability, ontology, intelligence, workflow-engine]
  bootstrap_step: 4
  cascade_impact_on_dependents:
    audit-chain: FULL
    observability: FULL
    ontology: FULL
    intelligence: FULL
    workflow-engine: FULL
microservice_path: microservices/policy-engine
api_version_current: v1.0.0
api_versions_supported: [v1.0]
authoring_adrs: [ADR-0150, ADR-0246, ADR-0280]
team_axis: axis-policy-engine
on_call_escalation: ops-policy-engine-rotation
```

**Catalog CI lane `oya-check-substrate-catalog-registration`:**

1. Verify every node in the DAG has a corresponding catalog file.
2. Verify catalog content matches the DAG node attributes (no
   drift).
3. Verify catalog content matches the per-microservice manifest
   `substrate_dag_position:` field (no drift).
4. Render the catalog README index from the per-substrate files
   (build-time generation).

Exit 1 on any drift.

### D-10. Build-time DAG validation in `oya-shared-substrate-client-*` crates

Every substrate exposes a shared client crate per ADR-0246 §D-3
naming convention: `oya-shared-<substrate>-client-{kernel, adapter,
sdk}`. The build-time DAG check is implemented via a procedural
macro that runs in the consuming crate's `build.rs`.

**Macro contract:**

The procedural macro `oya-shared-substrate-dag-position-assert`
(crate: `crates/oya-shared-substrate-dag-position-assert/`) reads
`specs/substrate-dependency-dag.json` at compile time and emits a
compile-time assertion verifying the consuming crate's caller
substrate has a DAG edge to the destination substrate.

**Usage pattern:**

```rust
// In oya-shared-policy-engine-client-adapter/build.rs
use oya_shared_substrate_dag_position_assert::assert_dag_edge;

fn main() {
    // Verify that the caller substrate (read from CARGO_PKG_NAME parsing)
    // has a declared DAG edge to "policy-engine". If the caller is not
    // a substrate (e.g., a product or a service-cell crate), the assert
    // is a no-op — only substrate-to-substrate calls go through the DAG.
    assert_dag_edge!(to = "policy-engine");
}
```

The macro:

1. Reads `specs/substrate-dependency-dag.json` (path resolved via
   `CARGO_MANIFEST_DIR` → workspace root).
2. Identifies the caller substrate (parses
   `CARGO_PKG_NAME = "oya-<caller>-..."` for the caller portion).
3. If the caller is one of the substrates in the DAG, verifies an
   edge `(caller, destination)` exists.
4. If the edge does not exist, emits a compile error:

```
error[E0001]: substrate edge missing in DAG
  --> oya-shared-policy-engine-client-adapter/build.rs:6:5
   |
 6 |     assert_dag_edge!(to = "policy-engine");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: caller substrate "intelligence" does not declare an edge to "policy-engine"
   = help: add the edge to specs/substrate-dependency-dag.json (requires DAG amendment ADR)
   = help: or, if the caller is not actually a substrate, refactor to use the µservice tier classification correctly
```

**CI lane `oya-check-substrate-client-crate-build-time-dag-check`:**

1. For each `oya-shared-<substrate>-client-adapter` crate, parse
   `build.rs` and verify it invokes `assert_dag_edge!`.
2. For each crate using `oya-shared-<substrate>-client-*` as a
   dependency, run `cargo check` and verify the build-time assert
   passes.

Exit 1 on missing macro invocation or assert failure.

### D-11. Runtime dependency-down circuit breaker (per ADR-0176 brown-out)

Build-time DAG enforcement prevents *static* dependency-direction
violations. Runtime dependency-down conditions require
**per-edge circuit breakers** that emit the ADR-0176 brown-out
signal and trigger the cascade-rule-prescribed behaviour.

**Circuit breaker structure (Rust example):**

```rust
// crates/oya-shared-policy-engine-client-adapter/src/circuit_breaker.rs
use oya_shared_brownout_signal::{BrownoutSignal, BrownoutState};

pub struct PolicyEngineCircuitBreaker {
    edge_attributes: SubstrateEdgeAttributes,
    state: CircuitState,
    consecutive_failures: AtomicU32,
    last_failure_at: AtomicU64,
}

impl PolicyEngineCircuitBreaker {
    pub async fn evaluate(&self, req: EvaluateRequest) -> Result<Decision, ClientError> {
        match self.state.load() {
            CircuitState::Closed => self.try_call(req).await,
            CircuitState::HalfOpen => self.try_call_with_probe(req).await,
            CircuitState::Open => self.fallback_per_cascade_rule(req).await,
        }
    }

    fn fallback_per_cascade_rule(&self, req: EvaluateRequest) -> Result<Decision, ClientError> {
        match self.edge_attributes.cascade_rule {
            CascadeRule::Full => {
                self.emit_brownout(BrownoutState::Failed);
                Err(ClientError::SubstrateUnavailable {
                    substrate: "policy-engine",
                    fallback: "fail-closed-default-deny",
                })
            }
            CascadeRule::Degraded => {
                self.emit_brownout(BrownoutState::Degraded);
                Ok(Decision::Forbid {
                    reason: format!(
                        "policy-engine degraded; substrate edge weight {} ⇒ default-deny",
                        self.edge_attributes.weight
                    ),
                })
            }
            CascadeRule::Brownout => {
                self.emit_brownout(BrownoutState::Brownout);
                // Brown-out semantics: emit signal but continue best-effort
                Ok(Decision::NotApplicable)  // Caller decides downstream
            }
            CascadeRule::Independent => unreachable!("policy-engine edges are never INDEPENDENT"),
        }
    }
}
```

**CI lane `oya-check-substrate-brownout-circuit-breaker-presence`:**

1. For each substrate-to-substrate client adapter crate, verify a
   circuit breaker module exists.
2. Verify the circuit breaker imports `oya-shared-brownout-signal`
   and emits the ADR-0176 v1 payload on Open / HalfOpen state
   transitions.
3. Verify the circuit breaker's fallback behaviour matches the
   DAG edge's `cascade_rule:` attribute (FULL ⇒ propagate
   unavailability; DEGRADED ⇒ typed forbid; BROWNOUT ⇒ continue).

Exit 1 on missing breaker or mismatched fallback.

### D-12. Per-substrate fault-injection chaos engineering (per ADR-0241 DR drills)

Static + runtime enforcement covers steady-state and detect-able
failure. **Untested failure modes are the leading cause of
production outages at hyperscale** (per Hamilton 2007 LISA paper
§3.4, Beyer et al. 2018 SRE Workbook chapter 14, AWS Builders'
Library "Avoiding fallback in distributed systems" 2024). This
ADR requires **periodic per-substrate fault-injection chaos drills**
that exercise the cascade rules under controlled conditions.

**Per-substrate drill cadence (declared in DAG `nodes` entry):**

| DR tier | Cadence (days) | Drill type |
|---|---|---|
| T0 | 7 | Live (in DR cell), automated rollback |
| T1 | 14 | Live (in DR cell), automated rollback |
| T2 | 21 | Live (in DR cell) or game-day simulation |
| T3 | 30 | Game-day simulation acceptable |

**Drill protocol:**

1. **Schedule:** Per-substrate cron declared in `cadence_days` field;
   drill executor owned by observability cell-health emits the schedule
   to a per-cell Kafka topic `substrate.chaos-drill.schedule.v1`.
2. **Pre-drill announcement:** 24 hours prior, drill executor emits
   `substrate.chaos-drill.announce.v1` event with the substrate, the
   target cell (DR pair), the drill window, the expected cascade set.
3. **Fault injection:** At drill time, drill executor invokes the
   per-substrate fault-injection API (Litmus Chaos / Chaos Mesh
   primitive) to take the substrate down in the DR cell.
4. **Cascade observation:** Per-substrate brown-out signals are
   collected; cascade set is compared against the DAG-derived
   expected cascade set; deviation → SEV-3 page.
5. **Restoration:** At drill window end, fault is removed; substrate
   returns; brown-out signals clear; restoration latency is recorded.
6. **Evidence emission:** Drill emits
   `evidence/dr-drills/substrate-<name>-<timestamp>.json` containing
   the schedule, the cascade observed, the cascade expected, the
   restoration latency, and the SEV-3 pages (if any).

**CI lane `oya-check-substrate-fault-injection-drill-cadence`:**

1. For each substrate, read the most recent drill evidence file.
2. Compute `now - last_drill_at`. If > `cadence_days`, exit 1.
3. Verify each drill evidence file matches the schema and the
   cascade set matches the DAG-derived expected set.

Exit 1 on missed drill or cascade-set deviation.

### D-13. Substrate plane-topology model (E0 / B0 / C0 / C1 / C2 + G + R)

**This section supersedes the "cell-as-leaf total order" framing of §D-1 / §D-2 / §D-4.**
The flat total order was the correct *machine artifact* for one of the platform's dependency
graphs — the C-plane steady-state runtime graph — but it is not the whole topology. The
canonical substrate topology is a **face-aware, sharded, typed DAG across planes**. §D-1's
former v1 machine shape was replaced by ADR-0635's bounded graph-v2 slice; this section defines
the target planes, graphs, face-splits, roots, and invariant that the current closed slice does not
yet cover completely and that a single `bootstrap_order` list cannot encode.

#### D-13.A. Two kinds of roots

The original §D-2 R-1 called `cell` "the leaf substrate; it depends on nothing." That
conflates two distinct notions of "root". The canonical model distinguishes them:

- **Security / integrity roots (EXTERNAL to the 24 capabilities).** Hardware root of trust +
  measured boot; an offline / threshold organisation root key + immutable trust bundle; the
  signed bootstrap manifest + artifact digests + the initial cell identity; and the genesis /
  break-glass authority. These are the AWS Nitro / Google Titan analog and live in plane **E0**
  (external genesis roots). They are NOT substrate µservices and NOT registry capabilities.
- **Liveness / hosting roots.** Bare compute; bootstrap network / DNS / time; local durable
  boot storage; and the minimal runtime that starts Kubernetes and the first service. These
  are the true bottom of the *hosting* order — the **B0** empty cell envelope.

The consequence: operational KMS / SPIRE (`cloud-secrets`) is **not** irreducible. It runs on
the hosting substrate (B0) and is authenticated through IAM. `secrets.root-control` is the sole
in-graph cryptographic authority *service*, but it is NOT the bootstrap leaf; the roots below
it are external (E0) and hosting (B0). The router is not a root at all — it is a data plane
(R).

#### D-13.B. The canonical plane model — four internal planes + external root

```text
E0  external genesis roots: hardware RoT + signed boot/artifacts + org root key/quorum
        + bootstrap compute/network/storage facts
          |
B0  empty cell envelope: {network.bootstrap, compute.bootstrap, storage.bootstrap}
        -> k8s.bootstrap -> cell.envelope
          |
C0  cell-local trust & authz: secrets.cell/intermediate -> iam verifier
        -> tenancy/home-cell snapshot -> policy PDP + local versioned policy/ReBAC store
          |
C1  cell-local reliability & platform: audit append/seal, observability, data, messaging,
        per-cell gateway, local flags
          |
C2  cell workloads: intelligence, workflow, billing metering, marketplace, console APIs,
        compliance enforcement, comms

  in parallel:
G   logically-global, physically-PARTITIONED management plane: secrets root admin,
        cell registry/lifecycle/placement, IAM admin, tenancy directory,
        policy authoring/signing/distribution, fleet k8s/network/compute/storage,
        CI/IaC + catalogs/aggregation
          | (signed, versioned snapshots — one-way)
R   distributed routing data plane: network edge + gateway edge + cell.router — thin,
        cellularized, cached, NO live G-plane dependency (static stability)
```

**Generating rule.** Push everything into the cell that *can* live in the cell; the global
plane (G) may be large, but **cell runtime must never synchronously depend on it**
(static-stability invariant — the data plane survives a control-plane outage). The router (R)
is a data plane, not the control plane, and must itself be cellularized (it is the only layer
that knows all cells). The C-plane bottom-to-top ordering (B0 → C0 → C1 → C2) is the target
ordering. ADR-0635's bounded graph-v2 slice encodes the represented runtime path with
`cell.envelope` as its B0 leaf; it does not yet encode the complete
`network.bootstrap → compute.bootstrap → storage.bootstrap → k8s.bootstrap → cell.envelope`
hosting chain.

#### D-13.C. Five distinct edge-typed graphs

A single `bootstrap_order` cannot encode all substrate relationships because the platform has
**five distinct dependency graphs with different edge types, often pointing in opposite
directions**:

1. **Genesis graph** — first-ever bring-up from E0: external roots → B0 → the first management
   cell (`cell.genesis`, break-glass).
2. **New-cell provisioning graph** — G-plane `cell.lifecycle.cp` creates an empty cell ID, then
   populates C0 → C1 → C2 (depends on iam / policy / tenancy / audit).
3. **Steady-state request graph** — the C-plane runtime dependency DAG. **This is the graph
   the §D-1 machine artifact encodes** (Tarjan-acyclic, Kahn-topo-sortable).
4. **Control-data publication graph** — G publishes signed, versioned snapshots one-way to R
   and C0 (policy bundles, tenant/home-cell directory, placement). Edges point G → cell, but
   they are **asynchronous and cached**, never synchronous runtime edges.
5. **Failure / brownout propagation graph** — the cascade closure of §D-5, honouring ADR-0176
   brown-out.

The §D-1 acyclicity invariant governs graph (3). **Amended by ADR-0635:** graph v2 now gives all
five graph kinds a closed machine shape for the bounded 19-unit W0-C slice, and derives graph (5)
from graph (3). This does not imply full §D-13.G coverage; see §D-13.F–H.

#### D-13.D. Per-capability face-splits (the crux)

The category error is resolved by splitting each conflated capability into its plane-specific
**faces**. The same capability name appears in more than one plane; each face has its own
dependency direction:

- **cell** → `cell.envelope` (B0 empty failure domain — the true hosting leaf) · `cell.genesis`
  (break-glass, first management cell) · `cell.lifecycle.cp` (G; depends on iam / policy /
  tenancy / audit) · `cell.router.dp` (R; signed cached snapshots, no synchronous tenancy /
  iam). **This breaks the alleged cycle**: genesis creates an empty cell ID first; lifecycle
  needs iam / policy only later. It also reconciles the fork — the DAG's "cell is leaf" is
  right about `cell.envelope`; platform-architecture.json's "cell at S2" was right about
  `cell.lifecycle.cp`. Both committed specs were partially correct about *different faces*.
- **iam** → G identity admin · C0 local token / SVID / JWK validation.
- **policy** → G authoring / signing / distribution · C0 local PDP + versioned tenant policy /
  ReBAC store. Standalone ✓, but **NOT a singleton global PDP**.
- **tenancy** → G lifecycle / home-cell directory · C0 signed cached tenant context + routing
  snapshot.
- **secrets** → external-root-backed (E0) G root/intermediate management · C0 cell key
  partition + downstream SPIRE issuer.

**Policy placement (standalone, cell-distributed).** G authoring / signing / distribution +
home-cell authority (tenant policy + ReBAC tuples) + per-cell runtime PDP + last-known-good
snapshot; PEPs at the gateway and at every protected service; the router does routing (plus at
most a cached token-signature check), **NOT full authz**. A stale snapshot must **DENY or route
to the authoritative shard — never silently authorize**. (AWS Verified Permissions =
store-per-tenant; Zanzibar = logically-global but physically-distributed-with-local-replicas,
not one PDP instance.)

#### D-13.E. Static-stability invariant

**Cell runtime never synchronously depends on the G-plane.** Existing sessions and routes
continue on cached, signed, versioned state (policy bundles, tenant directory, placement
snapshots). Only *new* identity / tenant / placement / migration operations may safely stop
when G is unavailable — and even those fail closed (deny or route-to-authoritative), never
fail open. This is the AWS "static stability using Availability Zones" doctrine applied to the
control/data-plane split: the data plane (C-plane runtime + R) survives a full control-plane
(G) outage. The §D-1 `forbidden_edges_assertion` set is the machine-checked floor of this
invariant for the C-plane faces; the G-plane no-synchronous-dependency rule is authored here
and flagged for the DAG v2 face schema.

#### D-13.F. Fork resolution — sole authority and stale-projection disposition

`specs/substrate-dependency-dag.json` is the **sole canonical machine topology artifact**.
ADR-0635 graph v2 makes its `steady_state_request` graph graph 3 of §D-13.C and maps the legacy
`cell` node to the `cell.envelope` (B0) face. Its current closed set is nevertheless a bounded W0-C
slice, not the full §D-13 target.

`specs/platform-architecture.json` `substrate_dag_canonical_ordering` retains a v1 ordering only
for historical compatibility. It is marked `stale-v1-not-current-parity`, carries
`current_parity_claim: false`, and MUST NOT be used as a current derived mirror, bootstrap source,
or proof that all capabilities and hosting roots are represented. A current projection may be
restored only by the `W0-C-TOPOLOGY-COVERAGE` follow-up after the missing topology is ratified.

#### D-13.G. 24-capability plane placement (summary)

The following is the **target placement model**, not current graph-v2 coverage. At target state,
every one of the 24 registry capabilities maps to topology faces across these planes (E0 =
external, B0/C0/C1/C2 = cell planes, G = management, R = router; `M` = management-only, never a
runtime dependency):

`cell`[B0 / G / R] · `iam`[G / C0] · `policy`[G / C0] · `tenancy`[G / C0] ·
`secrets`[E0-backed G / C0] · `audit`[C1 seal + G async aggregation] ·
`observability`[C1 collectors + G async fleet] · `data`[C1 + optional G schema] ·
`storage`[B0 + C1 + G capacity] · `compute`[B0 + C1 + G fleet] · `k8s`[B0 / C0 per-cell + G
fleet] · `network`[B0 + R edge + C0 mesh + G DNS/config] · `gateway`[R edge + C1 ingress /
PEP] · `messaging`[C1 bus / outbox + async cross-cell + optional G schema] ·
`intelligence`[C2 + G registry] · `workflow`[C2 + G catalog] · `iac`[M desired-state — never a
runtime dependency] · `ci`/`delivery-fabric`[M management, isolated runners — never a prod
request dependency] · `billing`[C1 / C2 metering + G async rating / invoice] ·
`marketplace`[G catalog + C2 cache / fulfil] · `console`[G / R shell; APIs route into cells,
no shared tenant datastore] · `compliance`[G authoring + C1 enforcement / evidence] ·
`comms`[C2 delivery shard + G templates] · `flags`[G authoring + C0 / C1 last-known-good
evaluator, no synchronous global lookup].

**Current bounded coverage (ADR-0635/W0-C).** The 19 declared dependency units span only these 11
capabilities: `network`, `cell`, `iam`, `tenancy`, `policy`, `secrets`, `audit`, `observability`,
`data`, `intelligence`, and `workflow`. These 13 registry capabilities are omitted from the current
machine graph: `storage`, `compute`, `k8s`, `gateway`, `messaging`, `ci`, `iac`, `billing`,
`marketplace`, `console`, `compliance`, `comms`, and `flags`. The current B0 chain contains
`network.bootstrap` and `cell.envelope` but omits the target `compute.bootstrap`,
`storage.bootstrap`, and `k8s.bootstrap` faces. No parity or completeness claim is valid across
those omissions.

**`governance` is NOT one of the 24 capabilities** — it is cross-cutting, implemented via
`ci` / `iac` / `policy` / `compliance` / `audit`. A standalone `governance` capability would
make 25; ADR-0615 resolved it decomposes rather than stands alone. The registry's historical
`iam`-owns-`{identity, policy-engine}` grouping is superseded by the founder policy-extract
ruling, with `policy` owning its own target topology faces.

**Mandatory completion disposition.** `W0-C-TOPOLOGY-COVERAGE`, tracked by
[GitHub #1537](https://github.com/jason931225/oyatie/issues/1537), must ratify the exact face tuples
and typed edges for the 13 omitted capabilities and missing B0 hosting chain before extending the
closed graph. It must update the derived failure closure and any architecture projection
atomically, migrate affected consumers, and MUST NOT mint a new frozen baseline.

#### D-13.H. Flagged for schema extension / founder call (not forced in this increment)

This amendment resolves the fork and authors the model as far as the current schemas support.
The following are **deliberately flagged, not forced**, to keep the acyclicity, canonical-json,
and cross-artifact-agreement gates green and to avoid a closed-registry membership change
riding a topology ADR:

1. **Topology coverage.** ADR-0635 lands the face/plane schema and all five graph kinds for the
   bounded 19-unit slice. The remaining 13 capabilities plus the missing B0 hosting-chain faces
   stay deferred to `W0-C-TOPOLOGY-COVERAGE`; their exact tuples and edges are not invented here.
2. **Module-membership and layer-rank consumers.** They remain separate graph-v2 migrations under
   `W0-C-MODULE-MEMBERSHIP` and `W0-C-LAYER-RANKS`.
3. **Current architecture projection.** The v1 projection remains stale until topology coverage is
   ratified and a deterministic graph-v2 projection can replace it atomically.

All three follow-ups preserve the no-new-baseline rule.

#### D-13.I. Reasoning and precedent

The model is chosen for three load-bearing reasons: **blast-radius containment** (a cell is a
bounded failure domain; conflating it with the services it hosts destroys the boundary),
**control-plane / data-plane separation** (the router and the cell runtime must survive a
management-plane outage), and **static stability** (cached, signed, versioned snapshots over
synchronous global lookups). Precedent: AWS cell-based architecture and "Static stability using
Availability Zones" (Builders' Library); AWS Verified Permissions store-per-tenant; Google
Zanzibar's logically-global / physically-distributed-with-local-replicas authorization; SPIRE /
Nested-SPIRE workload identity; AWS Nitro / Google Titan hardware roots of trust; and Meta Shard
Manager's placement / router separation. Each supports the same conclusion the flat total order
missed: **the unit of ownership is not the unit of deployment, and every control capability has
both a global-control face and a cell-local runtime face.**

## Alternatives considered

### Alt-1. Keep substrate dependencies implicit (status quo before this ADR)

Continue declaring the DAG in prose across ADR-0245, ADR-0246,
ADR-0242 bootstrap sequence, and per-µservice manifest fields. No
canonical machine-readable artifact.

**Pros:**
- No new authoring surface; existing ADRs cover most of the rules.
- Reviewers walk the prose ADRs when uncertainty arises.

**Cons:**
- Cycle introduction silently possible; reviewers can miss a single
  backward import line.
- Bootstrap order and DAG drift apart over time (the ADR-0242 list
  vs. the manifest dependencies vs. the actual import graph).
- SLO composition cannot be computed mechanically; remains informal.
- Per-edge Cedar permit fragments authored case-by-case with
  incomplete coverage (per PR #143 evidence).
- Big Ball of Mud (Foote+Yoder 1997) anti-pattern risk: looks like
  microservices, behaves like a monolith under stress.
- Contradicts the **automate-everything** memory
  (feedback_automate_everything): "Anything mechanical (consensus
  loops, sweeps, gates, claim/work/done) must be scripted; never
  manually iterate deterministic work." Acyclicity checking is the
  textbook mechanical task.

**Rejected** — the cost of leaving the DAG implicit is the
distributed-monolith failure mode that this entire keystone wave is
authored to prevent.

### Alt-2. Per-µservice manifest dependency arrays only (no canonical DAG)

Declare each substrate's `depends_on:` field in
`microservices/<name>/manifest.json` and synthesize the DAG at CI
time from the manifests. No canonical `specs/substrate-dependency-
dag.json`.

**Pros:**
- Distributed authorship: each substrate team owns its own
  manifest.
- Lower coordination overhead per change.

**Cons:**
- The DAG is **derived**, not **authoritative**. The forbidden-edge
  assertions cannot be authored at the manifest level (a manifest
  can only declare positive dependencies, not negative-space
  forbidden ones).
- Per-edge attributes (cascade_rule, dependency_weight, Cedar
  permit fragment, version compatibility range) live in
  `manifest.json` schemas, polluting the per-µservice manifest with
  cross-µservice concerns.
- No single review surface for the **whole substrate layer**'s
  topology. Reviewers must read 10 manifests to understand the DAG.
- Bootstrap order derivation still requires a separate authoring
  step (the topological sort tie-breaker has to be declared
  somewhere).
- Schema evolution for per-edge attributes is hard: every change
  touches every substrate's manifest.

**Rejected** — distributed authorship loses the single-source-of-
truth property that makes the DAG load-bearing.

### Alt-3. Layered architecture diagram only (PlantUML)

Author the substrate dependency relationship as a PlantUML or
Mermaid diagram in `docs/architecture/substrate-layers.md`. Visual
inspection used for review. No machine validation.

**Pros:**
- Human-friendly representation.
- Easy to render in PRs and docs.

**Cons:**
- Not machine-validated. Cycles can be drawn into the diagram and
  pass review (it looks fine to the eye).
- Manifest manifests, bootstrap script, and SLO composition cannot
  derive from a diagram — diagrams are presentation layer, not data
  layer.
- Diagrams drift from code over time (the classic "stale
  documentation" problem).

**Rejected** — diagrams are an output, not a source.

### Alt-4. DAG as Rust code (programmatic declaration)

Declare the DAG as a Rust constant in
`crates/oya-substrate-dag/src/lib.rs`. Build-time generate JSON
representation for cross-language consumption.

**Pros:**
- Type-safe authoring.
- Compile-time verification of structure.

**Cons:**
- Non-Rust consumers (Go control-plane components, TypeScript
  Foundry orchestrator helpers, JSON-schema validators for IaC)
  cannot read Rust source directly; they need a generated
  artifact anyway.
- A Rust constant is less amenable to manual editing for spec
  amendments — JSON is universally toolable.
- Worse for non-engineer review (architects, ops leads) than JSON.

**Rejected** — JSON is the lowest-friction canonical surface;
build-time Rust validation is layered on top (see §D-10).

### Alt-5. Use existing ADR-0245 spec path `microservice-dependency-dag.json`

ADR-0245 forward-declared `/specs/microservice-dependency-dag.json`
(note: **microservice**, not **substrate**). Reuse that path
instead of introducing a new spec.

**Pros:**
- One fewer spec file.
- Maintains continuity with ADR-0245's forward declaration.

**Cons:**
- The ADR-0245 spec was scoped to **all 19 substrates** plus all 27
  products plus the service cell plus the 7 reserved µservices —
  far broader than the Tier-1 substrate core this ADR locks down.
- Mixing tiers in one DAG file mixes invariants: cycle prohibition
  is **substrate-strict**, but product-to-product edges are
  permitted per ADR-0145 §D-4.F. A single DAG file would have to
  encode multiple cycle policies.
- The substrate DAG must change far less often than the product
  DAG (products refactor; substrates are tier-1 stable per
  ADR-0245 §D-9 12-month deprecation policy). Mixing them
  conflates change cadence.

**Resolved** — adopt **both** spec files:
- `/specs/substrate-dependency-dag.json` (this ADR) is the
  Tier-1 substrate-strict DAG with full acyclicity invariant.
- `/specs/microservice-dependency-dag.json` (ADR-0245) is the
  broader portfolio DAG including products and service cells with
  the relaxed product-to-product edge policy.

The substrate DAG is a **strict subset** of the microservice DAG:
substrate edges in the broader DAG MUST equal substrate edges in
this DAG. The lane
`oya-check-substrate-dag-subset-of-microservice-dag` verifies the
subset property.

### Alt-6. Graph database (Neo4j / DGraph) as DAG storage

Store the DAG in a graph database; query via Cypher / GraphQL for
analysis.

**Pros:**
- Rich query language for cascade analysis.
- Easy to add new graph-theoretic queries.

**Cons:**
- Adds runtime infrastructure dependency to the build/CI surface.
- The DAG has 10 nodes and ~45 edges; a graph database is wildly
  over-engineered.
- A JSON file checked into git has the auditability + reviewability
  + version-control properties a graph database lacks.

**Rejected** — at the substrate scale (10 nodes), JSON-in-git is
the right substrate; in-memory graph computation in the validator
crate handles all queries in <10ms.

## Consequences

### Positive

1. **Single source of truth for substrate topology.** Reviewers,
   contributors, and agents all read one file:
   `specs/substrate-dependency-dag.json`. Per-µservice manifests,
   bootstrap scripts, SLO compositions, brown-out fixtures, chaos
   drills, and Cedar permit fragments derive from it.
2. **Cycle introduction becomes impossible.** The CI lane runs
   Tarjan's algorithm on every PR; cycles are rejected at the
   admission gate. The distributed-monolith failure mode is closed.
3. **Bootstrap ordering is mechanical.** ADR-0242 §D-5's manually-
   authored 10-step sequence is replaced by Kahn's algorithm
   output on the DAG. No drift; future DAG amendments
   automatically update the bootstrap order.
4. **Failure cascade analysis is mechanical.** When substrate X is
   reported down, the cascade set is computed in O(V+E). On-call
   responders consult a deterministic answer instead of tribal
   knowledge.
5. **SLO composition becomes deterministic.** Substrates' authored
   SLOs are bounded by the Markov-chain composition of their
   dependencies. Over-promised SLOs are caught at PR time, not at
   post-mortem.
6. **Cross-substrate Cedar coverage becomes mechanical.** Per-edge
   permit fragments are generated from the DAG; coverage CI lane
   verifies every edge has its fragment.
7. **Build-time client-crate enforcement.** Substrate-to-substrate
   gRPC client crates carry compile-time DAG-position asserts.
   Refactor mistakes (a substrate accidentally importing a higher
   substrate's client) are caught at `cargo check` time.
8. **Runtime brown-out signal is structured.** Per-edge circuit
   breakers emit the ADR-0176 v1 payload; cross-substrate cascade
   responses are consistent across the fleet.
9. **Chaos drills are scheduled and audited.** Per-substrate cadence
   declared in the DAG; missed drills surface as BLOCKER CI
   findings; observed cascades validate the DAG.
10. **Self-hosting authority is encoded.** Foundry's Tier-S5 meta-
    substrate position (per ADR-0247) is captured as an explicit
    DAG entry with explicit edges to every other substrate. The
    self-modification doctrine has a load-bearing anchor.

### Negative

1. **One more spec file.** `specs/substrate-dependency-dag.json` is
   a new authoring surface. DAG amendments require ADR amendment
   per ADR-0211 no-silent-regression doctrine — a higher friction
   for legitimate dependency additions.
2. **Build-time procedural macro complexity.** The `assert_dag_edge!`
   macro adds compile-time machinery; debug experiences for failed
   asserts must be excellent (compile errors with help suggestions
   per §D-10).
3. **Per-substrate chaos drill cadence is operational load.**
   Tier-0 substrates drill weekly; Tier-1 every two weeks. The
   drill executor service-cell must run continuously and emit
   evidence. (Mitigated by automation: per ADR-0241 the drill
   executor is already on the roadmap.)
4. **DAG amendment requires multispectrum review.** Adding a new
   edge or rewiring an existing edge requires the full
   multispectrum-review process per ADR-0144 governance. This is
   intentional (the DAG is load-bearing) but adds friction.
5. **Substrate split decisions become explicit.** When a substrate
   accumulates a cross-cutting responsibility (e.g., policy-engine
   absorbing both Cedar fragment registry + Cedar evaluator + Cedar
   audit), the team must split (Conway split per ADR-0246 §D-1) or
   accept the increased SLO composition product. No more silent
   accumulation.
6. **Per-edge attribute schema evolution is cross-substrate
   coordination.** Adding a new edge attribute (e.g., a new
   cascade-rule variant) requires updating every edge plus the
   schema plus the validator plus the catalog. (Mitigated by the
   keystone-bundle authoring pattern: cross-cutting schema changes
   land as their own ADR.)

### Operational

1. ADR-0242 §D-5 bootstrap sequence is **derived**, not authored.
   The ADR text is amended to read "see §D-4 of ADR-0280 for the
   bootstrap order derivation."
2. ADR-0245 §D-4 substrate dependency rules are **encoded** in the
   DAG spec. The ADR text remains as prose explanation; the
   machine artifact is the load-bearing surface.
3. ADR-0246 §D-1 policy-engine promotion is encoded as the
   policy-engine node + its edges in the DAG. Future policy-engine
   refactors update the DAG, which propagates to ADR-0246's
   downstream invariants.
4. ADR-0145 §invariants 1-3 remain unchanged; this ADR layers
   substrate-strict cycle prohibition on top of the broader
   ADR-0145 direct-gRPC permission.
5. ADR-0176 brown-out signal is consumed by every per-edge circuit
   breaker; the brown-out signal API is locked to v1 for the
   duration of this DAG's v1.0.0.
6. ADR-0241 DR drill portfolio is the consumer of the chaos drill
   cadence declared in the DAG `nodes` entries.
7. ADR-0247 self-modification doctrine references Foundry's
   Tier-S5 position in the DAG; Foundry is **not** in the v1.0.0
   Tier-1 DAG declared here (it lands in v1.1.0 once Foundry's
   decomposition is final per ADR-0247).
8. ADR-0258 (substrate API versioning) consumes the
   `version_compatibility_range:` edge attribute; the two ADRs are
   reviewed and accepted together.

## Verification

The doctrine's enforcement surface is verifiable end-to-end:

### V-1. Spec landing verification

```bash
test -f specs/substrate-dependency-dag.json
jq -e '.doctrine_adr == "ADR-0280"' specs/substrate-dependency-dag.json
jq -e '.version | test("^\\d+\\.\\d+\\.\\d+$")' specs/substrate-dependency-dag.json
jq -e '.nodes | length >= 10' specs/substrate-dependency-dag.json
```

### V-2. Acyclicity lane verification

```bash
oya gate validate substrate-dependency-dag-acyclicity --strict
# Expect exit 0 + machine-readable evidence at
# evidence/governance-lane/substrate-dependency-dag-validation-<timestamp>.json
```

### V-3. Cycle-injection test

ADR-0635 retires the standalone v1 DAG copies. The graph-v2 corpus names self-loop, two-node,
three-node, and buried six-node SCC mutations and applies them to the live canonical document.

```bash
buck2 test //ci/facade/dependency-graph-acyclicity:ci-dependency-graph-acyclicity-gate
# Expect every named RED mutation to fail closed and the live graph to remain GREEN.
```

### V-4. Bootstrap-order verification

```bash
buck2 test //ci/facade/dependency-graph-acyclicity:ci-dependency-graph-acyclicity-gate
# Verifies the declared order is a valid dependency-first order, an invalid order is RED,
# and deterministic alphabetical Kahn output remains available without requiring the valid
# hand-authored order to equal that tie-break.
```

### V-5. SLO composition bound verification

```bash
for ms in cell identity tenancy policy-engine cloud-secrets audit-chain \
          observability ontology intelligence workflow-engine; do
  oya gate validate substrate-slo-composition-bounds --substrate "$ms"
done
# Each invocation emits evidence/slo-composition-<ms>-<timestamp>.json
# with the bound, the authored value, and any justification.
```

### V-6. Cross-substrate Cedar coverage verification

```bash
oya gate validate substrate-cross-call-cedar-coverage
# Verifies every DAG edge has its named Cedar permit fragment under
# microservices/policy-engine/fragments/baseline/substrate-edge-permits/
# and the fragment compiles + is signed by org-baseline.
```

### V-7. Build-time client-crate DAG check verification

```bash
cargo check --workspace 2>&1 | grep "assert_dag_edge"
# Expect no errors. To verify the macro catches violations, inject a
# synthetic violation:
mv crates/oya-shared-policy-engine-client-adapter/build.rs build.rs.bak
cat > crates/oya-shared-policy-engine-client-adapter/build.rs <<'EOF'
fn main() {
    oya_shared_substrate_dag_position_assert::assert_dag_edge!(to = "intelligence");
}
EOF
cargo check --workspace 2>&1 | grep "substrate edge missing in DAG"
# Expect compile error per §D-10.
mv build.rs.bak crates/oya-shared-policy-engine-client-adapter/build.rs
```

### V-8. Runtime circuit-breaker presence verification

```bash
oya gate validate substrate-brownout-circuit-breaker-presence
# Verifies each substrate client adapter has the per-edge circuit
# breaker module with the cascade-rule-matched fallback behaviour.
```

### V-9. Chaos drill cadence verification

```bash
oya gate validate substrate-fault-injection-drill-cadence
# For each substrate, verifies the most recent drill evidence file
# is within cadence_days of now. Expect exit 0 in steady state.
```

### V-10. Multispectrum review acceptance

The ADR is reviewed per the F1-F9 + M1 + M2 + applicable A1-A7
facets (per the v2.4.0 multispectrum review). Evidence files emit
to `evidence/debate/adr-0280/`.

## References

- **DAG-based service architecture papers.**
  Verma, A., Pedrosa, L., Korupolu, M., Oppenheimer, D., Tune, E.,
  Wilkes, J. "Large-scale cluster management at Google with Borg."
  EuroSys 2015. — The foundational paper on Borg's strata model
  and acyclic dependency invariant.
  Burns, B., Grant, B., Oppenheimer, D., Brewer, E., Wilkes, J.
  "Borg, Omega, and Kubernetes." Communications of the ACM 59(5),
  2016. — How the Borg stratification evolved into Kubernetes.
- **Hamilton's LISA 2007 paper on internet-scale services.**
  Hamilton, J. "On Designing and Deploying Internet-Scale Services."
  USENIX LISA 2007. — Section 2.2 "Failure cascades" prescribes
  that "every service must operate independently of all other
  services, and the dependency graph must be acyclic." Available
  at https://mvdirona.com/jrh/talksAndPapers/JamesRH_Lisa.pdf
  (mirror: mvdirona.com/perspectives.aspx).
- **Bezos API Mandate 2002.**
  Yegge, S. "Stevey's Google Platforms Rant" (2011 Google+ post,
  preserved). — Public paraphrase of the 2002 Amazon directive:
  "All teams will henceforth expose their data and functionality
  through service interfaces. ... There will be no other form of
  interprocess communication allowed." The implicit consequence is
  a service dependency DAG; Amazon's internal "Operational
  Excellence" review per AWS Builders' Library catalogues per-
  service dependency graphs.
- **Werner Vogels — A Conversation with Werner Vogels.**
  Vogels, W. interviewed by Gray, J. ACM Queue, June 2006. — On
  AWS's service tier model (foundational vs. application vs.
  product) and the deliberate cross-tier dependency direction
  rules.
- **Stripe internal substrate dependency model.**
  Stripe Engineering Blog "Online migrations at scale" 2017;
  Stripe Press "How we built it: the Stripe API" 2020;
  Stripe SRECon 2022 "Reliability at the speed of finance"
  (McCaffrey, C.). — Stripe's tier-0 / tier-1 / tier-2 / tier-3
  service-tier model and the internal `bootstrap-graph` lint that
  forbids cross-tier inverse dependencies.
- **Kubernetes dependency management.**
  Kubernetes Documentation "Kubernetes Components" 2024-Q4;
  kubeadm reference docs; "Kubernetes Patterns" Ibryam + Huss
  2nd ed. 2023. — The control-plane stratum: etcd → kube-apiserver
  → kube-controller-manager + kube-scheduler → kubelet → kube-proxy.
- **AWS Builders' Library.**
  "Static stability using Availability Zones" 2024;
  "Avoiding fallback in distributed systems" 2024;
  "Workload isolation using shuffle-sharding" 2024. — AWS's
  publicly-documented per-service dependency-graph practices.
- **Google SRE Workbook.**
  Beyer, B., Murphy, N. R., Rensin, D. K., Kawahara, K., Thorne, S.
  (eds). "The Site Reliability Workbook." O'Reilly 2018. — Chapter
  2 on SLO composition; Chapter 14 on chaos engineering as the
  validation surface for cascade rules.
- **CNCF / Kubernetes governance.**
  CNCF Technical Oversight Committee graduation criteria 2024;
  KubeCon NA 2024 "GKE 10-year retrospective" by Tim Hockin.
- **Foote + Yoder — Big Ball of Mud (1997).**
  Foote, B., Yoder, J. "Big Ball of Mud." Fourth Conference on
  Patterns Languages of Programs (PLoP '97), Allerton Park,
  Monticello, Illinois, September 1997. — The canonical anti-
  pattern reference: scattered authority + piecemeal growth +
  unstructured dependency graphs produce the platform-scale ball
  of mud this ADR exists to prevent. Paper available at
  laputan.org/mud/.
- **Conway's Law.**
  Conway, M. E. "How Do Committees Invent?" Datamation, April 1968.
  — The dependency direction in software mirrors the communication
  direction in the organization. The team-axis ownership per
  substrate (axis-cell, axis-identity, etc.) is the Conway-aligned
  surface; the DAG is the technical encoding.
- **Tarjan's strongly-connected-components algorithm.**
  Tarjan, R. E. "Depth-first search and linear graph algorithms."
  SIAM Journal on Computing 1(2), 1972. — The O(V+E) cycle-
  detection algorithm used by the DAG validator (§D-3).
- **Kahn's topological-sort algorithm.**
  Kahn, A. B. "Topological sorting of large networks."
  Communications of the ACM 5(11), 1962. — The O(V+E) topological
  sort used to derive bootstrap order (§D-4).
- **Markov-chain availability composition.**
  Pinheiro, E., Weber, W.-D., Barroso, L. A. "Failure trends in a
  large disk drive population." IEEE Transactions on Reliability,
  2007. — Generalized to service composition by Google SRE
  Workbook ch. 2; the formula prescribed in §D-6.
- **OpenTelemetry semantic conventions.**
  OpenTelemetry Specification 1.30, semantic conventions for
  service.name + service.instance.id. — The substrate name is the
  primary OTel `service.name` attribute; the DAG node identity is
  the substrate name.
- **Cedar v4.2 language reference.**
  Cedar Engine Working Group, "Cedar Policy Language Specification
  v4.2," 2024-Q4. — Cedar permit fragment naming convention used in
  §D-7.
- **Litmus Chaos / Chaos Mesh.**
  CNCF Litmus Chaos project documentation 2024;
  PingCAP Chaos Mesh project documentation 2024. — The chaos
  primitives used by the drill executor (§D-12).
- **PR #143 review evidence (internal).**
  `evidence/pr-143-review-integration.json`;
  `evidence/pr-143-review-idea-refine.json`;
  `evidence/pr-143-close-out-plan-and-gap-audit-2026-05-18.json`.
  — The near-miss substrate-dependency-inversion findings that
  motivated this ADR's authoring.

## Appendix A — Pattern attribution

Every named external pattern this ADR incorporates is attributed to
its source.

| Pattern | Origin | Where used in this ADR |
|---|---|---|
| Acyclic service dependency graph | Hamilton LISA 2007 §2.2 | §D-1, §D-2 (Rule R-12), §D-3 |
| Topological sort for bootstrap order | Kahn 1962 | §D-4 |
| Strongly-connected components for cycle detection | Tarjan 1972 | §D-3 |
| Markov-chain service availability composition | Pinheiro 2007 / SRE Workbook 2018 ch. 2 | §D-6 |
| Service tier model | Bezos API Mandate 2002 / Vogels ACM Queue 2006 | §D-2 (Rules R-1 through R-11), implicit |
| Borg strata / Kubernetes component stratification | Verma 2015 / Burns 2016 | §D-2 (Rules R-1 through R-11), background |
| Substrate split via Conway split | Conway 1968 + ADR-0246 §D-1 | §Alt-2, §Consequences |
| Big Ball of Mud anti-pattern avoidance | Foote + Yoder 1997 | §Context, §Alt-1, §Consequences |
| Brown-out degradation signal | ADR-0176 v1 | §D-5, §D-11 |
| Chaos engineering periodic drills | SRE Workbook 2018 ch. 14 + ADR-0241 | §D-12 |
| Per-edge cascade rule (FULL/DEGRADED/BROWNOUT/INDEPENDENT) | Original (inspired by AWS Builders' Library "Avoiding fallback" 2024) | §D-5 |
| Build-time procedural macro for DAG assertion | Original (inspired by Rust `static_assertions` crate pattern) | §D-10 |
| Cedar per-edge permit fragments | Original (extends ADR-0243 + ADR-0246) | §D-7 |
| Single-source-of-truth machine-readable spec | feedback_automate_everything memory + general SRE practice | §D-1 |
| 12-month deprecation window for substrate APIs | ADR-0245 §D-9 + ADR-0258 (forthcoming) | §D-8 |
| Substrate catalog as human-readable index | Original (parallel to k8s `kubectl get` output) | §D-9 |
| Fail-closed default-deny on substrate dependency-down | ADR-0243 §D-11 | §D-5, §D-11 |

External patterns are referenced + extended; internal additions are
the per-edge attributes (cascade_rule, dependency_weight, Cedar
permit fragment name, version compatibility range, brown-out
protocol version), the validator crate, the build-time macro, and
the substrate catalog rendering.

## Appendix B — Worked example: adding a new substrate to the DAG safely

This worked example walks through the end-to-end process of adding
a new substrate to the DAG. Suppose the platform needs a new
substrate `feature-flags` (which today is a `product-internal`
µservice per ADR-0245 §D-3.B, but is being promoted to substrate
because per ADR-0243 §D-13 feature flags are Cedar fragments and
the feature-flag service is on the hot path of every gated call).

### Step 1 — Authorize the addition via an ADR amendment

Per the "DAG amendment requires multispectrum review" consequence in
§Consequences, adding a new edge requires an ADR amendment. Author:

```
docs/decisions/ADR-0XXX-feature-flags-substrate-promotion.md
```

The ADR amendment:

- Cites ADR-0245 promotion criteria (the substrate-vs-product
  layering rule).
- Cites ADR-0280 §D-2 for the dependency rules the new substrate
  must satisfy.
- Declares the new substrate's position in the DAG: between which
  existing substrates.
- Declares the new substrate's tier_subtype: in this case
  `substrate-feature-flags` (new subtype; requires §D-2 enum
  expansion ADR amendment).

### Step 2 — Identify the new substrate's edges

For `feature-flags`:

- Outgoing edges (substrates feature-flags depends on):
  - `cell` (cell-local feature-flag state) — weight 1.0, FULL.
  - `identity` (caller identity for flag attribution) — weight 1.0,
    FULL.
  - `tenancy` (tenant-scoped flags) — weight 1.0, FULL.
  - `policy-engine` (per-flag Cedar gating) — weight 1.0, FULL.
  - `cloud-secrets` (flag-bundle encryption) — weight 0.5, DEGRADED.
  - `audit-chain` (flag-change audit) — weight 1.0, FULL.
  - `observability` (per-flag metrics) — weight 0.3, BROWNOUT.

- Incoming edges (substrates that will depend on feature-flags):
  - `intelligence` (LLM provider feature flags) — weight 0.3,
    BROWNOUT.
  - `workflow-engine` (workflow-step feature flags) — weight 0.3,
    BROWNOUT.

### Step 3 — Verify no cycle would be introduced

Simulate the addition: add `feature-flags` node, add the outgoing
edges, add the incoming edges, run Tarjan's algorithm.

Expected position in topological sort: between `policy-engine` and
`cloud-secrets` (because feature-flags depends on policy-engine but
not on cloud-secrets) — wait, actually feature-flags **also**
depends on cloud-secrets. So feature-flags must come after both
policy-engine **and** cloud-secrets. The topological sort places
feature-flags at step 6.5 — between audit-chain (step 6) and
observability (step 7).

Run the simulation:

```bash
cargo run -p oya-substrate-dependency-dag-validator -- simulate \
  --add-node feature-flags \
  --add-edges feature-flags:cell,feature-flags:identity,... \
  --output evidence/dag-simulation-feature-flags.json
```

The validator confirms no cycle would be introduced (no path from
any existing substrate back to feature-flags).

### Step 4 — Update `specs/substrate-dependency-dag.json`

Bump version to 1.1.0 (per ADR-0258 SemVer; minor bump because the
addition is backward-compatible — existing substrates' edges
unchanged).

Add the `feature-flags` node, its outgoing edges, its incoming
edges, and update `bootstrap_order` to insert it at the right
position.

Add forbidden-edge assertions if any:

```json
{"from": "feature-flags", "to": "intelligence",
 "reason": "intelligence consumes feature-flags, not the reverse."},
{"from": "feature-flags", "to": "workflow-engine",
 "reason": "workflow-engine consumes feature-flags, not the reverse."}
```

### Step 5 — Author the µservice scaffold

Per ADR-0131 per-microservice flat layout:

```
microservices/feature-flags/
├── PRD.md
├── manifest.json   ; with tier: substrate, tier_subtype: substrate-feature-flags
├── api/
├── capabilities/
├── iac/
├── migrations/
├── slos/
└── tests/
```

Author the shared client crate family:

```
crates/oya-shared-feature-flags-client-kernel/
crates/oya-shared-feature-flags-client-adapter/   ; with build.rs assert_dag_edge!
crates/oya-shared-feature-flags-client-sdk/
```

### Step 6 — Author the Cedar edge permits

For each new edge, author a permit fragment at:

```
microservices/policy-engine/fragments/baseline/substrate-edge-permits/
├── permit-feature-flags-call-cell-v1.cedar
├── permit-feature-flags-call-identity-v1.cedar
├── permit-feature-flags-call-tenancy-v1.cedar
├── permit-feature-flags-call-policy-engine-v1.cedar
├── permit-feature-flags-call-cloud-secrets-v1.cedar
├── permit-feature-flags-call-audit-chain-v1.cedar
├── permit-feature-flags-call-observability-v1.cedar
├── permit-intelligence-call-feature-flags-v1.cedar
└── permit-workflow-engine-call-feature-flags-v1.cedar
```

Each fragment follows the pattern in §D-7.

### Step 7 — Update existing substrate manifests

Each substrate that **consumes** feature-flags must add an entry to
its `substrate_dag_position.depends_on:` list:

```yaml
# microservices/intelligence/manifest.json
substrate_dag_position:
  depends_on: [cell, identity, tenancy, policy-engine, cloud-secrets,
               audit-chain, observability, ontology, feature-flags]
  consumed_by_substrates: [...]
```

### Step 8 — Update the substrate catalog

Add `microservices/cloud-iac/cell-catalog/substrates/feature-flags.yaml` per
the §D-9 schema. Regenerate the catalog README index.

### Step 9 — Update the bootstrap script

Re-derive the cloud-iac bootstrap output from the
updated DAG. The script reads the DAG and emits the topological
sort; no manual edit needed.

### Step 10 — Schedule the first chaos drill

Per §D-12, the new substrate's `chaos_drill_cadence_days:` is
declared in the DAG node entry. For a T1 substrate, the first
drill is scheduled 14 days after substrate go-live.

### Step 11 — Run the validation gauntlet

```bash
oya gate validate substrate-dependency-dag-acyclicity
oya gate validate substrate-bootstrap-order-matches-topological-sort
oya gate validate substrate-cross-call-cedar-coverage
oya gate validate substrate-catalog-registration
oya gate validate substrate-client-crate-build-time-dag-check
oya gate validate substrate-slo-composition-bounds
oya gate validate substrate-brownout-circuit-breaker-presence
```

All must exit 0 before the PR can land.

### Step 12 — Multispectrum review

Per ADR-0144 governance, the PR is reviewed across F1-F9 + M1 + M2
+ applicable A1-A7 facets. The DAG amendment is one of the highest-
sensitivity changes in the portfolio (per ADR-0245 §D-9 substrate
breaking-change policy), requiring 12 calendar months of
deprecation notice for any subsequent edge removal.

### Step 13 — Land + monitor

The PR lands via the Foundry pipeline (per ADR-0136 / ADR-0247).
The new substrate's brown-out signal is monitored; the first chaos
drill is observed; the SLO composition for upstream consumers is
recomputed to verify the bound still holds.

### Step 14 — Lock in (post-bootstrap)

After 30 days of green operation, the `enforcement_status` field in
the ADR amendment promotes from `advisory` to `BLOCKER`; any
attempt to remove the feature-flags edges requires a deprecation
ADR per ADR-0245 §D-9.

This walk-through demonstrates that adding a new substrate to the
DAG is a **bounded, auditable, mechanical process**. The 14 steps
take 1-2 sprints depending on the µservice's scaffolding scope;
the DAG amendment itself takes ~30 minutes once the edges are
known. The validator catches mistakes (cycles, missing fragments,
missing manifest fields) at step 11, before the multispectrum
review effort is spent.

## Appendix C — Tier-2 substrates and Foundry's Tier-S5 position

This ADR's v1.0.0 DAG covers the Tier-1 load-bearing core (the ten
substrates without which no tenant workload runs). The Tier-2
substrates (consumed selectively; their absence does not block the
bootstrap critical path) are forward-declared here for
completeness; they land in `specs/substrate-dependency-dag.json`
v1.1.0 once each has gone through its own per-substrate scaffolding
IP.

### Tier-2 substrates (9 substrates)

| Substrate | tier_subtype | dr_tier | Notes |
|---|---|---|---|
| `cloud-iac` | substrate-iac | T2 | Helm + Terraform module registry; consumed at deploy time, not runtime. |
| `cloud-k8s` | substrate-compute | T1 | Kubernetes control-plane wrapper; consumed by cell substrate at provisioning time. |
| `network` | substrate-network | T1 | DNS + mesh + NetworkPolicy authoring; consumed at deploy time. |
| `api-gateway` | substrate-api-gateway | T1 | Envoy / Cilium ingress; sits at the edge of products + service-cells. |
| `comms-email` | substrate-comms | T2 | Transactional email; consumed by products' notification flows. |
| `consent-graph` | substrate-consent | T2 | Consent state authoring; consumed by tenancy + compliance. |
| `compliance` | substrate-compliance | T2 | Compliance Pack fragment registry; consumed by tenancy + cell. |
| `governance` | substrate-governance | T2 | ~50 oya-check-* lanes; consumed at CI time, not runtime. |
| `marketplace-catalog` | substrate-marketplace-data | T2 | Canonical product catalog data; consumed by plugin-app-store + marketplace cell. |

### Tier-S5 — Foundry (substrate-meta)

| Substrate | tier_subtype | dr_tier | Notes |
|---|---|---|---|
| `foundry` | substrate-meta | T2 | Per ADR-0247 self-modification doctrine, Foundry's role is decomposed; the substrate-meta tier captures its CI + multispectrum-review + evidence-emission role. Lands in DAG v1.2.0 once decomposition is final. |

The Tier-2 substrates and Foundry will be added to the canonical
DAG via the §Appendix B worked-example process in subsequent
keystone bundles. The doctrine accommodates 30+ substrate nodes;
the validator's Tarjan / Kahn computations remain O(V+E) and
complete in <100ms at portfolio scale.

## Appendix D — Glossary

| Term | Definition |
|---|---|
| Substrate | A µservice with `tier: substrate` per ADR-0245. Audience-neutral capability provider. |
| DAG | Directed Acyclic Graph. Nodes are substrates; edges are dependency relationships. |
| Tier-1 substrate | Load-bearing substrate without which no tenant workload runs. The ten substrates declared in §D-2. |
| Tier-2 substrate | Substrate consumed selectively; absence does not block bootstrap. The nine substrates in Appendix C. |
| Tier-S5 | The substrate-meta tier; substrates that author and modify the platform itself (Foundry). |
| Bootstrap order | The topological sort of the DAG; the sequence in which substrates start at platform genesis. |
| Cascade rule | Per-edge attribute declaring how the source substrate behaves when the destination is unavailable. Four values: FULL, DEGRADED, BROWNOUT, INDEPENDENT. |
| Dependency weight | Per-edge numeric attribute (0..1) used in SLO composition: 1.0 ⇒ destination's unavailability fully inherits; lower ⇒ partial inheritance. |
| SLO composition bound | The Markov-chain product of dependent substrates' SLOs weighted by edge dependency-weight; the upper bound on the source substrate's authored SLO. |
| Brown-out signal | Per ADR-0176 v1 structured payload emitted by a substrate when it enters degraded operation. |
| Chaos drill | Scheduled fault-injection exercise per ADR-0241 + §D-12; takes a substrate down in the DR cell, observes the cascade, restores. |
| Cedar edge permit fragment | Per-edge Cedar policy fragment authoring that the source substrate is permitted to call the destination substrate's actions, scoped by tenant + cell + DAG-position assertion. |
| Substrate catalog | The human-readable index under cloud-iac cell-catalog substrate ownership; complement to the machine-readable DAG. |
| `assert_dag_edge!` | Build-time procedural macro that verifies the calling substrate has a declared DAG edge to the destination. Compile error on missing edge. |
| Big Ball of Mud | Foote + Yoder 1997 anti-pattern: scattered authority + piecemeal growth + unstructured dependency graphs. The failure mode this ADR exists to prevent at substrate scale. |
| Distributed monolith | The platform-scale form of Big Ball of Mud: looks like microservices, behaves like a monolith under stress due to substrate cycles. |
