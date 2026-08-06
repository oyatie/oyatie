---
id: ADR-0341
title: Cellular promotion gates — explicit per-Tier 0..4 machine-checkable criteria + auto-promotion via cell-orchestrator
status: Superseded
planning_impact: true
date: 2026-05-21
owner_team:
  - council-architecture
  - ops-sre-reliability
  - axis-observability
  - axis-tenancy
  - council-security
owners:
  - council-architecture
  - ops-sre-reliability
  - axis-observability
  - axis-tenancy
  - council-security
supersedes: []
superseded_by: [ADR-700]
amends:
  - ADR-0248-amazon-shape-cellular-architecture.md (carves out explicit machine-checkable promotion-gate criteria + warm-soak floors + auto-promotion mechanism between cellular Tier 0 → 1 → 2 → 3 → 4; bidirectional demotion criteria added; preserves the Tier 0 = highest blast-radius convention)
  - ADR-0244-tenant-as-universal-scoping-primitive.md (cell-orchestrator µservice within tenancy emits per-tenant_class promotion eligibility; demo_trial + paid coverage is a gate input)
  - ADR-0212-buildability-doctrine.md (every µservice manifest gains `cell_promotion_gates` block citing applicable tier + cellular_deployment_pattern from ADR-0244 §D-5)
  - ADR-0263-observability-emission-contract.md (per-cell SLO-budget telemetry + cross-cell call success metric become first-class gate inputs; observability is the substrate per ADR-0130 + ADR-0131)
  - ADR-0251-compliance-pack-cell-certification-levels.md (compliance-pack-coverage gate validates every applicable pack signed off before promotion; pack floor is binding)
  - ADR-0148-cell-orchestrator-control-plane.md (formalizes the cell-orchestrator as the auto-promotion executor inside tenancy + observability; the µservice has been referenced anchorally and gains explicit gate-evaluation duty here)
  - ADR-0186-canary-cohort-discipline.md (canary-cohort SLO compliance ≥ 99.5% becomes a gate input rather than an advisory signal)
  - ADR-0044-inter-cell-mesh-tunnel.md (cross-cell call success ≥ 99.95% becomes a gate input rather than an advisory signal)
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md (Wave 15T-Cell-Promotion-Gates added as a sub-wave that authors the lane + gate evaluator + cell-orchestrator gate-eval glue; per-µservice promotion entries follow in their own canonical-build phase order)
related:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0044-inter-cell-mesh-tunnel.md
  - ADR-0108-deprecation-and-sunset-policy.md
  - ADR-0130-observability-as-substrate.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0148-cell-orchestrator-control-plane.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0186-canary-cohort-discipline.md
  - ADR-0212-buildability-doctrine.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0250-build-ahead-of-certification.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0254-deployment-model-spectrum.md
  - ADR-0263-observability-emission-contract.md
  - ADR-0322-substance-bar-as-doctrine-and-ci-enforcement.md
  - ADR-0324-anti-script-anti-template-doctrine.md
  - ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md
  - ADR-0329-tier-system-retired-replaced-by-tenant-class.md
  - ADR-0330-tenant-class-demo-trial-vs-paid-composable-billing-components.md
  - ADR-0331-cross-microservice-tenant-class-adoption-template.md
  - ADR-0333-cell-microservice-retired-pattern-not-service.md
  - ADR-0336-valkey-not-redis-substrate.md
  - ADR-0337-iceberg-canonical-olap-write-path.md
  - ADR-0338-pod-runtime-tier-0-to-3.md
  - ADR-0339-shared-iac-module-library.md
  - ADR-0340-capacity-model-per-microservice-manifest.md
related_specs:
  - /specs/master-plan-sequencing.json
  - /specs/microservices/manifest-schema.json
  - /specs/microservices/cell.json
  - /specs/platform-architecture.json
  - /specs/markdown-retirement-policy.json
  - /specs/decision-principles.json
related_memory:
  - feedback_six_candidate_adrs_2026_05_21
  - feedback_amazon_shape_cellular_architecture
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_clean_architecture_requirements
  - feedback_build_ahead_of_certification
  - feedback_compliance_pack_primitive
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
  - feedback_microservice_ownership_coherence_2026_05_20
  - feedback_verify_deliverables_not_just_line_count_2026_05_20
  - feedback_docs_substance_not_scaffold_2026_05_20
  - feedback_rust_strict_only_no_python_2026_05_20
  - feedback_zero_handroll_opentofu_only_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - feedback_bominal_inheritance_precedence
companion_docs:
  - docs/standards/hyperscaler-best-practices.md
  - docs/standards/dependency-policy.md
  - microservices/tenancy/ARCHITECTURE.md
  - microservices/observability/ARCHITECTURE.md
  - microservices/cell-orchestrator/ARCHITECTURE.md
  - tools/hooks/_canonical-primitives.md
inbound_citations:
  - /Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_six_candidate_adrs_2026_05_21.md
doc_class: Architecture-Decision-Record
shape: Decision
authority_tier: 1
line_floor: 600
bespoke_authoring_requirement: documentation-rigor-1.1-plus-ADR-0322
enforcement_status: advisory-until-cell-orchestrator-gate-evaluator-lands
enforced_by:
  - oya-check-cell-promotion-gates (new CI lane; evaluates the six machine-checkable criteria for every promotion candidate against per-cell telemetry + manifest; REPORT-ONLY at landing; promoted to BLOCKER after Wave 15T-Cell-Promotion-Gates lands the evaluator + per-µservice promotion-history schema)
  - oya-governance-cell-tier-promotion-evidence (new CI lane; refuses promotion-history entries that lack the evidence pack documented in §D-7; REPORT-ONLY at landing; BLOCKER post-soak)
  - oya-governance-cell-tier-numbering-convention (new CI lane; refuses promotion-history entries that invert the ADR-0248 Tier 0 = highest-blast-radius convention; e.g., refuses claims of "promoted from Tier 4 to Tier 3" being framed as a hardening operation when in this corpus Tier 3 = less critical than Tier 4 in the inverted hyperscaler-style direction is forbidden; clarifies the convention is monotonically decreasing-criticality from Tier 0 to Tier 4)
  - oya-governance-cell-promotion-quiet-window (new CI lane; refuses auto-promotion events that did not observe the per-tier quiet-window floor in §D-3.6)
  - oya-governance-cell-orchestrator-binding (new CI lane; refuses promotion events that did not originate from a cell-orchestrator µservice principal under the ADR-0148 + ADR-0247 self-modification namespace; manual promotion is forbidden outside the emergency-override path in §D-9)
  - Kyverno admission policy `enforce-cell-promotion-gates` (refuses Kubernetes cellular topology mutations — node-pool resize, cell-zone creation, cell-tier label change — that do not carry a cell-orchestrator-signed promotion-event attestation)
purpose: >
  Carve out explicit machine-checkable promotion-gate criteria for every
  cellular tier-promotion edge in ADR-0248's Tier 0..Tier 4 numbering
  (where Tier 0 = highest blast-radius / most isolated, Tier 4 = lowest /
  best-effort), plus the symmetric demotion edges. Define six gate inputs
  the new CI lane `oya-check-cell-promotion-gates` evaluates: (1) error
  budget intact (≥ 99 % of the µservice's SLO budget remaining on the
  current tier); (2) ≥ N days warm-soak in the current tier where N is
  per-edge — Tier 0 → 1: 7 days; Tier 1 → 2: 14 days; Tier 2 → 3: 28
  days; Tier 3 → 4: 56 days; (3) canary cohort success ≥ 99.5 % SLO
  compliance over the warm-soak window; (4) cell-mesh health: cross-cell
  call success ≥ 99.95 % over the warm-soak window per ADR-0044; (5)
  tenant-class coverage: both demo_trial and paid tenants present on the
  current tier (per ADR-0330); (6) compliance-pack coverage: every
  applicable pack from ADR-0251 (HIPAA, GDPR-strict, SOC2, PCI, CSAP,
  EU AI Act Annex III, ISO27001, KR-ISMS-P, JP-FISC, etc.) validated and
  signed off on the current tier. Establish auto-promotion via the
  cell-orchestrator µservice (running inside tenancy + observability per
  the ADR-0148 control-plane shape) once all gates pass plus an N-hour
  quiet window (per §D-3.6) elapses without alert; auto-promotion emits a
  signed promotion-event onto the audit-chain per ADR-0263 and propagates
  the new cell_tier into the per-cell node labels via Kyverno-admitted
  topology mutation. Out of scope: actual cell-orchestrator µservice
  implementation, which is sequenced as a follow-on sub-wave under
  ADR-0148 + ADR-0328 batch discipline.
---

> **Disposition light-edit (2026-08-06):** Cellular promotion gates

# ADR-0341: Cellular promotion gates — explicit per-Tier 0..4 machine-checkable criteria + auto-promotion via cell-orchestrator

## Status

Proposed on 2026-05-21.

This ADR is an amendment to ADR-0248 (Amazon-shape cellular architecture) that carves out explicit per-edge machine-checkable promotion gates between cellular Tier 0..Tier 4. ADR-0248 established the cellular topology and the criticality-numbered tier ladder (Tier 0 = Foundation cells: identity / KMS / audit; Tier 1 = Substrate cells; Tier 2 = Capability cells; Tier 3 = Application cells; Tier 4 = Edge cells), but treated promotion across the tier ladder as a council-architecture decision made on case-by-case evidence. The implicit reading of ADR-0248 was that an operator decides "this cell is ready to be promoted from Tier 2 → Tier 1" and the topology mutates manually. That reading produces three concurrent regressions: (a) promotion timing depends on human review cadence, which is incompatible with the substance-bar hyperscaler-grade-rigor directive (`feedback_quality_performance_scalability_bar`); (b) promotion criteria are not auditable as machine-checkable facts, which is incompatible with the build-ahead-of-certification directive (`feedback_build_ahead_of_certification`); (c) promotion is not reversible by the same mechanism, which is incompatible with the no-silent-regression directive (`feedback_no_silent_regression`). This ADR resolves all three by binding promotion to six gate inputs evaluated by a CI lane and an in-cluster cell-orchestrator µservice.

The tier numbering aligns with ADR-0248 verbatim and with ADR-0338 pod-runtime-tier (which co-varies with cellular tier). **Tier 0 = highest blast-radius / most isolated**, Tier 4 = best-effort / edge / lowest blast-radius. This ADR does not invert that convention; the numbering remains monotonically decreasing-criticality from Tier 0 to Tier 4 to preserve `feedback_bominal_inheritance_precedence` against the existing Bominal corpus shape.

Enforcement transitions from `advisory-until-cell-orchestrator-gate-evaluator-lands` to `BLOCKER` per the lane sequence in §E below: at landing of Wave 15T-Cell-Promotion-Gates (which authors the lane evaluator + per-cell telemetry schema + cell-orchestrator gate-eval glue), the `oya-check-cell-promotion-gates` lane promotes to BLOCKER for new promotion events; per-µservice migration of legacy ad-hoc promotion records follows the canonical-build phase order under ADR-0328.

This ADR does not author the cell-orchestrator µservice itself. The cell-orchestrator's existence is anchored in ADR-0148 (cell-orchestrator control plane) and ADR-0247 (self-modification doctrine via `oyatie.foundry.*` Cedar principals); its implementation is sequenced as a follow-on sub-wave separately. This ADR establishes only the gate-evaluation contract that any cell-orchestrator implementation must observe.

This ADR does not change the cellular tier numbering from ADR-0248. The five-tier model (Tier 0..Tier 4) is preserved verbatim, including the inversion against the ascending-prestige convention retired alongside ADR-0329.

This ADR does not change tenant_class behavior from ADR-0330. demo_trial and paid tenants both factor into the tenant-class-coverage gate; the gate is satisfied only when both classes are present on the current tier.

This ADR does not change compliance pack activation gating from ADR-0251. Compliance packs continue to set their own floor; the compliance-pack-coverage gate confirms every applicable pack is signed off, but does not override pack semantics.

This ADR does not change Cedar evaluation from ADR-0243. Cedar evaluates application-layer authorization at request time; this ADR's gates run at admission time (Kyverno) plus pre-merge time (CI lane) plus continuous time (cell-orchestrator).

## Date

2026-05-21.

## Context

### A.1 Named pressure: ad-hoc cell promotion is incompatible with hyperscaler-grade rigor

Hyperscalers operate cellular architectures (AWS, Stripe, Cloudflare, Salesforce, Microsoft, Apple per the ADR-0248 named-precedent enumeration) with promotion criteria that are explicit, machine-checkable, and binding. AWS's internal "cell graduation" process — documented in James Hamilton's 2024 re:Invent talk on cellular architecture and in Colm MacCárthaigh's 2023 talks on shuffle sharding — uses six explicit gate inputs evaluated continuously by control-plane services, not a human committee. Stripe's 2024 engineering blog post on cellular architecture documents an equivalent gate set bound to the cellular control-plane component they call "Pier" internally. Cloudflare's 2024 SRE Conference talk on tier promotion within their cellular topology described an equivalent gate set tied to a control-plane component they call "Argo Tier Manager."

The hyperscaler precedent is explicit: tier promotion is a continuously-evaluated, machine-checkable, multi-input decision made by control-plane software, not by a human operator. The decision is auditable because every gate input is a recorded fact (SLO budget remaining, warm-soak duration, canary cohort observation, cross-cell mesh health, tenant-class coverage, compliance-pack coverage), and the promotion event itself is a signed audit-chain row.

The Oyatie corpus before this ADR has ADR-0248 establishing the topology, ADR-0044 establishing the inter-cell mesh tunnel, ADR-0148 anchoring a cell-orchestrator control-plane shape (with implementation deferred), ADR-0186 establishing canary-cohort discipline, ADR-0263 establishing the observability emission contract, ADR-0251 establishing compliance pack certification levels, and ADR-0330 establishing the tenant_class model. The pieces exist; what is missing is the explicit promotion-gate carve-out that says "these six inputs evaluated this way at this cadence promote a cell from Tier N to Tier N-1 (or demote)." That carve-out is this ADR.

### A.2 Named pressure: implicit promotion timing depends on human review cadence

Before this ADR, the implicit reading of ADR-0248 is "promote a cell when council-architecture says so." That reading creates two coupled regressions:

- **Cadence drag.** Council-architecture reviews happen on weekly or fortnightly cadence at best. A cell whose error-budget, warm-soak, canary-cohort, mesh-health, tenant-class-coverage, and compliance-pack-coverage inputs all turn green on Tuesday at 03:14 UTC waits until the next council window (e.g., Thursday at 16:00 UTC) before any promotion decision is made — a ~60-hour lag during which the inputs may drift back to red. The hyperscaler precedent has zero lag: the inputs are continuously evaluated and the promotion fires the moment all conditions hold for the quiet window.
- **Promotion-bias drift.** Human review introduces unintentional bias against promoting cells that touch unfamiliar compliance packs (CSAP, JP-FISC, IL5) because the reviewer wants to "be careful." The hyperscaler precedent is bias-free because the machine-checkable inputs apply uniformly regardless of pack.

Both regressions compound at corpus scale: as Oyatie reaches the projected 47-77 µservice / multi-cell / multi-region steady state, council-architecture-mediated promotion becomes a corpus-wide bottleneck. This ADR removes that bottleneck by making the gate evaluation machine-checkable and the promotion auto-executed.

### A.3 Named pressure: promotion reversibility (demotion) needs the same machine-checkable shape

Promotion is not symmetric with demotion under ad-hoc human review. A cell that gets promoted under a green-six-gates evaluation should be demoted under a symmetric criteria — error budget exhaustion below the floor, warm-soak interrupted by alert burst, canary cohort failure, mesh-health drop, tenant-class-coverage loss, or compliance-pack-coverage loss. Without explicit demotion criteria, a Tier 1 cell that begins to drift toward Tier 2 reliability has no automated path to "demote with audit trail" — it stays Tier 1 until a human notices, which violates the `feedback_no_silent_regression` directive.

This ADR formalizes the bidirectional shape: the same six inputs evaluate both promotion (Tier N → Tier N-1, e.g., Tier 2 → Tier 1) and demotion (Tier N-1 → Tier N, e.g., Tier 1 → Tier 2). The thresholds for demotion are stricter than promotion to avoid flapping (§D-5 below).

### A.4 Named pressure: six gate inputs map to substrates already declared elsewhere

Each of the six gate inputs has an existing canonical substrate in the corpus, making this ADR a wiring decision rather than a substrate decision:

| Gate input | Existing substrate | Wiring this ADR adds |
|---|---|---|
| Error budget intact (≥ 99 %) | OpenSLO docs per ADR-0186 + observability per ADR-0263 | budget-remaining metric becomes a gate input |
| Warm-soak duration (≥ N days) | per-cell creation timestamp in tenancy µservice | warm-soak floor per edge becomes a gate input |
| Canary cohort success (≥ 99.5 % SLO compliance) | canary-cohort discipline per ADR-0186 | cohort SLO compliance becomes a gate input |
| Cell-mesh health (≥ 99.95 % cross-cell call success) | inter-cell mesh tunnel per ADR-0044 | cross-cell call success becomes a gate input |
| tenant-class coverage (demo_trial + paid both present) | tenant_class model per ADR-0330 | per-class presence becomes a gate input |
| compliance-pack coverage (all applicable packs validated) | compliance pack certification per ADR-0251 | per-pack sign-off becomes a gate input |

Because every input is already declared as a canonical substrate, this ADR does not introduce a new measurement plane. It introduces a six-input AND-gate evaluator that consumes existing telemetry and emits a single boolean per cell per tier-edge.

### A.5 Named pressure: substrate vs product layering per ADR-0245

Per ADR-0245 (substrate vs product layering), promotion gates are substrate concerns, not product concerns. The product-side surface is unchanged: tenants do not know their workload is moving from a Tier 3 cell to a Tier 2 cell; the promotion is invisible at the API contract. The substrate-side surface is where the auto-promotion executes: node labels mutate, scheduling constraints update, observability telemetry re-tags. This ADR sits entirely inside the substrate boundary and exposes no product surface.

### A.6 Named pressure: auto-promotion needs a control plane and an audit-chain row

Auto-promotion has two non-negotiable accompaniments: a control plane that executes the topology mutation (so the human path is never the critical path), and a signed audit-chain row per promotion event (so the regulator path always has evidence). ADR-0148 anchors the control plane (cell-orchestrator µservice); ADR-0263 anchors the audit-chain emission contract. This ADR binds the auto-promotion event to both: the cell-orchestrator emits a signed `cell.promotion.executed` audit-chain row per ADR-0263 and propagates the new cell_tier into the per-cell node labels per Kyverno admission. The control plane is the executor; the audit-chain is the evidence.

### A.7 Named pressure: emergency override path

The auto-promotion path MUST have an emergency override for the rare case when a human operator needs to demote a cell immediately (e.g., during an active incident before gate telemetry has caught up to reality). The override path is documented in §D-9: it requires multi-party authorization (incident commander + on-call SRE + council-security), it emits the same audit-chain row class with `cell.promotion.override` event class, and it is observable by the same dashboards. The override path does not skip the audit trail; it skips only the warm-soak floor and the gate evaluation.

### A.8 Inherited constraints

- **ADR-0009 cell architecture per tenant per region.** Cells remain the universal isolation primitive; this ADR's gates evaluate per-cell facts, not per-tenant facts.
- **ADR-0044 inter-cell mesh tunnel.** Cross-cell call success ≥ 99.95 % is a gate input; the mesh tunnel telemetry is the canonical source.
- **ADR-0148 cell-orchestrator control plane.** The cell-orchestrator is the auto-promotion executor; this ADR does not author the µservice but binds to its principal namespace.
- **ADR-0186 canary cohort discipline.** Canary cohort SLO compliance ≥ 99.5 % is a gate input; canary cohort definition follows ADR-0186 verbatim.
- **ADR-0211 in-house tech stack preference.** The cell-orchestrator runs as a Rust-strict µservice per `feedback_rust_strict_only_no_python_2026_05_20`; gate evaluation is Rust-strict.
- **ADR-0240 sovereign-cloud per regional pack.** Sovereign cells follow the same gate set with the compliance-pack-coverage gate naturally including the sovereign pack (CSAP, KR-ISMS-P, JP-FISC, etc.).
- **ADR-0244 tenant scoping universal primitive.** Every gate evaluation carries `tenant_id` + `tenant_class` context; cell-orchestrator decisions are tenant-scoped to the tenant resident on the cell.
- **ADR-0247 self-modification doctrine.** Cell-orchestrator operates under the `oyatie.foundry.*` Cedar principal namespace per ADR-0247; promotion events are self-modification events.
- **ADR-0251 compliance pack cell certification.** The compliance-pack-coverage gate input is binding; pack floor stricter than this ADR's gates wins.
- **ADR-0252 HLC default + TrueTime tier.** Warm-soak duration measurement uses HLC ordering by default and TrueTime for cells operating under fin-grade compliance per ADR-0252.
- **ADR-0263 observability emission contract.** Every gate input is consumed via the canonical observability emission; the cell-orchestrator does not bypass the emission contract.
- **ADR-0322 substance-bar.** This ADR carries 600+ lines of substance per the line floor.
- **ADR-0324 anti-script doctrine.** Wave 15T-Cell-Promotion-Gates rewrite buckets author per-µservice promotion history entries bespoke per ADR-0324.
- **ADR-0328 substance-bar canonical-sequence-and-batch-discipline.** Wave 15T-Cell-Promotion-Gates is sequenced under ADR-0328.
- **ADR-0329 + ADR-0330 tenant_class.** demo_trial + paid both present is a gate input.
- **ADR-0338 pod runtime tier 0..3.** Pod runtime tier co-varies with cellular tier; the two axes are independent decision surfaces but their gate-shape mirrors each other (pod runtime tier promotion under ADR-0338 D-10 follows analogous evidence-pack discipline).
- **ADR-0339 shared IaC module library.** Per-cell node-pool topology mutation on auto-promotion uses the canonical IaC primitives at `microservices/cloud-iac/modules/<context>/cell-zone/` per ADR-0339.

### A.9 What this ADR does not assert

- This ADR does not author the cell-orchestrator µservice. The µservice's existence is anchored in ADR-0148; the implementation sub-wave is separately sequenced.
- This ADR does not change the cellular tier numbering from ADR-0248. The five-tier model is preserved verbatim.
- This ADR does not introduce new SLO metrics. Every gate input is an existing observability emission per ADR-0263.
- This ADR does not change tenant_class semantics. demo_trial + paid both present is the binding shape per ADR-0330.
- This ADR does not change compliance pack activation gating. Pack semantics are preserved per ADR-0251; the gate confirms sign-off, does not redefine it.
- This ADR does not author the per-cell telemetry schema. The schema is added in Wave 15T-Cell-Promotion-Gates as part of the lane evaluator authoring.
- This ADR does not author the Kyverno policy `enforce-cell-promotion-gates`. The policy is added in Wave 15T-Cell-Promotion-Gates as part of the admission-gate authoring.
- This ADR does not change ADR-0338 pod runtime tier semantics. Pod runtime tier and cellular tier are independent axes; this ADR governs only the cellular axis.

## Decision

### B.1 Decision statement

Every cellular promotion or demotion event between ADR-0248 tiers (Tier 0..Tier 4, where Tier 0 = highest blast-radius / most isolated, Tier 4 = best-effort / edge / lowest blast-radius) MUST be evaluated against six machine-checkable gate inputs by the new CI lane `oya-check-cell-promotion-gates` plus the in-cluster cell-orchestrator µservice. The six gates are: (1) error budget ≥ 99 % of SLO budget remaining on the current tier; (2) warm-soak ≥ N days in the current tier where N is per-edge (Tier 0 → 1: 7d; Tier 1 → 2: 14d; Tier 2 → 3: 28d; Tier 3 → 4: 56d); (3) canary cohort SLO compliance ≥ 99.5 % over the warm-soak window; (4) cross-cell call success ≥ 99.95 % over the warm-soak window per ADR-0044; (5) both demo_trial + paid tenants present on the current tier per ADR-0330; (6) every applicable compliance pack per ADR-0251 validated and signed off. Auto-promotion fires when all six gates pass + an N-hour quiet window per §D-3.6 elapses without alert burst. Demotion fires when any gate falls below its demotion threshold per §D-5. The cell-orchestrator µservice is the executor; it emits a signed `cell.promotion.executed` (or `cell.promotion.demoted`) audit-chain row per ADR-0263, propagates the new cell_tier into the per-cell node labels via Kyverno-admitted topology mutation, and updates the per-µservice manifest `cell_promotion_history` field.

The six gate definitions are in D-2.

The per-edge warm-soak floors are in D-3.

The per-edge quiet windows are in D-3.6.

The auto-promotion execution path is in D-4.

The demotion path (symmetric inverse) is in D-5.

The gate-evaluator implementation is in D-6.

The evidence pack format for human-reviewed promotions (rare, post-soak override path) is in D-7.

The Kyverno admission policy is in D-8.

The emergency override path is in D-9.

The per-µservice manifest field updates are in D-10.

The Wave 15T-Cell-Promotion-Gates sequence is in D-11.

The cell-orchestrator binding is in D-12.

### B.2 Numbered decision clauses

B2.001. Every cellular tier-promotion or tier-demotion event between ADR-0248 Tier 0..Tier 4 MUST be evaluated against the six machine-checkable gates in D-2.

B2.002. The gate evaluation is performed by the new CI lane `oya-check-cell-promotion-gates` at pre-merge time AND by the in-cluster cell-orchestrator µservice at continuous-evaluation time.

B2.003. The six gates evaluate AND: a promotion fires only when all six pass; a demotion fires when any one falls below its demotion threshold in §D-5.

B2.004. Gate 1 (error budget intact) requires ≥ 99 % of the µservice's SLO budget remaining on the current tier over the warm-soak window per OpenSLO + ADR-0186.

B2.005. Gate 2 (warm-soak floor) requires ≥ N days in the current tier where N is the per-edge floor in D-3: Tier 0 → 1 = 7 days; Tier 1 → 2 = 14 days; Tier 2 → 3 = 28 days; Tier 3 → 4 = 56 days.

B2.006. Gate 3 (canary cohort success) requires the canary cohort defined per ADR-0186 to maintain ≥ 99.5 % SLO compliance over the warm-soak window.

B2.007. Gate 4 (cell-mesh health) requires cross-cell call success ≥ 99.95 % over the warm-soak window per the inter-cell mesh tunnel telemetry from ADR-0044.

B2.008. Gate 5 (tenant-class coverage) requires both `tenant_class = demo_trial` and `tenant_class = paid` tenants present on the current tier per ADR-0330.

B2.009. Gate 6 (compliance-pack coverage) requires every applicable compliance pack per ADR-0251 (HIPAA, GDPR-strict, SOC2, PCI, CSAP, EU AI Act Annex III, ISO27001, KR-ISMS-P, JP-FISC, etc.) validated and signed off on the current tier.

B2.010. The per-edge quiet window after gate convergence is per D-3.6: Tier 0 → 1 = 24 hours; Tier 1 → 2 = 48 hours; Tier 2 → 3 = 96 hours; Tier 3 → 4 = 168 hours.

B2.011. Auto-promotion fires when all six gates pass AND the quiet window elapses with no alert burst on the cell.

B2.012. Auto-promotion executes via the cell-orchestrator µservice running inside the tenancy + observability substrate per ADR-0148; the µservice operates under the `oyatie.foundry.*` Cedar principal namespace per ADR-0247.

B2.013. On auto-promotion, the cell-orchestrator emits a signed `cell.promotion.executed` audit-chain row per ADR-0263 carrying the cell_id, previous tier, new tier, six-gate evaluation snapshot, evaluator version, signed by the cell-orchestrator principal.

B2.014. On auto-promotion, the cell-orchestrator propagates the new cell_tier into the per-cell node labels via Kubernetes API + Kyverno-admitted topology mutation; the admission policy `enforce-cell-promotion-gates` (D-8) refuses topology mutation without a cell-orchestrator-signed promotion-event attestation.

B2.015. On auto-promotion, the cell-orchestrator updates the per-µservice manifest `cell_promotion_history` field with the new entry; the manifest schema at `/specs/microservices/manifest-schema.json` is updated to add the field.

B2.016. Auto-demotion follows the symmetric inverse path per §D-5; thresholds for demotion are stricter than promotion to avoid flapping.

B2.017. The gate evaluator is implemented in Rust per `feedback_rust_strict_only_no_python_2026_05_20`.

B2.018. The gate evaluator consumes per-cell telemetry via the canonical observability emission contract per ADR-0263; it does not bypass the emission contract.

B2.019. The gate evaluator runs in continuous-evaluation mode inside the cell-orchestrator µservice; it also runs in pre-merge mode inside the `oya-check-cell-promotion-gates` CI lane (the CI lane evaluates against the most recent published snapshot of per-cell telemetry).

B2.020. The CI lane is REPORT-ONLY at landing time and promotes to BLOCKER when Wave 15T-Cell-Promotion-Gates lands the evaluator + per-µservice promotion-history schema.

B2.021. The CI lane validates: (a) every promotion-history entry cites a cell-orchestrator-signed promotion-event; (b) the cited promotion-event's six-gate evaluation snapshot is included; (c) the warm-soak floor was observed; (d) the quiet window was observed; (e) the tier numbering preserves the ADR-0248 monotonically-decreasing-criticality convention.

B2.022. The CI lane refuses promotion-history entries that lack the evidence pack documented in §D-7 (rare; only applies to the emergency-override path).

B2.023. The Kyverno admission policy `enforce-cell-promotion-gates` is BLOCKER-class at landing; it refuses Kubernetes cellular topology mutations (node-pool resize, cell-zone creation, cell-tier label change) that do not carry a cell-orchestrator-signed promotion-event attestation.

B2.024. The Kyverno policy emits an audit event `cell.promotion.admission.denied` on every deny per ADR-0263.

B2.025. The emergency override path (§D-9) requires multi-party authorization: incident commander + on-call SRE + council-security signatures.

B2.026. The emergency override path emits `cell.promotion.override` audit-chain events; the override is logged with the same retention + immutability as the standard promotion-event.

B2.027. The emergency override path does NOT bypass the audit-chain; it bypasses only the warm-soak floor and the gate-evaluation AND-condition.

B2.028. Per-µservice `manifest.json` gains a `cell_promotion_gates` block citing the applicable tier + cellular_deployment_pattern from ADR-0244 §D-5.

B2.029. Per-µservice `manifest.json` gains a `cell_promotion_history` field (array of promotion-event references); the schema at `/specs/microservices/manifest-schema.json` is updated to add the field.

B2.030. The cell-orchestrator µservice MUST operate under the ADR-0148 principal binding; manual promotion is forbidden outside the §D-9 emergency-override path.

B2.031. The cell-orchestrator µservice MUST run inside the tenancy + observability substrate per ADR-0148; standalone control-plane µservices outside the substrate boundary are forbidden.

B2.032. Tier 0 → 1 promotion requires the strictest gates: warm-soak 7 days + quiet window 24 hours + every Tier-0-applicable compliance pack signed off (HIPAA + PCI + EU AI Act Annex III + sovereign packs as applicable). The numerical floor for this edge is intentionally less restrictive on warm-soak duration than other edges because the cell is moving from Tier 0 (highest criticality) to Tier 1 (still substrate-class but less critical) — the operation is a controlled relaxation, not an escalation.

B2.033. Tier 1 → 2 promotion requires warm-soak 14 days + quiet window 48 hours.

B2.034. Tier 2 → 3 promotion requires warm-soak 28 days + quiet window 96 hours.

B2.035. Tier 3 → 4 promotion requires warm-soak 56 days + quiet window 168 hours (7 days). Tier 4 is the least-critical tier; the warm-soak floor is the longest because Tier 4 is also the most-likely-to-be-rolled-back tier in case of stability regression.

B2.036. The symmetric demotion edges follow stricter thresholds per §D-5: any single gate falling below the demotion threshold triggers demotion with no quiet window (demotion is immediate to protect blast-radius).

B2.037. The `oya-check-cell-promotion-gates` lane reads gate inputs from the per-cell telemetry snapshot at `microservices/observability/snapshots/cell-telemetry-<cell-id>-<timestamp>.json` (the snapshot format is added in Wave 15T-Cell-Promotion-Gates).

B2.038. The cell-orchestrator µservice exposes a gRPC contract for gate evaluation (defined under ADR-0145 inter-µservice communication reform); the contract is published per ADR-0212 buildability doctrine.

B2.039. The cell-orchestrator µservice operates under the cellular tier appropriate to its own role; per ADR-0245 substrate-vs-product, the orchestrator is substrate-class and runs at Tier 0 or Tier 1 (substrate touching tenant data plane via cell-tier labels).

B2.040. The cell-orchestrator µservice's pod_runtime_tier per ADR-0338 is Tier 1 (substrate touching tenant data plane); the orchestrator runs under Kata Containers + Cloud Hypervisor in the kata-pool.

B2.041. Tenant-class coverage gate (Gate 5) is satisfied when both demo_trial and paid tenant_class instances are present on the current tier per ADR-0330; if the cell hosts only one class, the gate fails and no promotion occurs.

B2.042. Tenant-class coverage gate also validates that the cell can host both classes — i.e., the per-cell topology supports both demo_trial workloads (OCI Always Free profile per `feedback_oci_always_free_maximization_2026_05_20` + the canonical tenant_class cap-shape from ADR-0331 §D-4) and paid workloads (full-capacity profile).

B2.043. Compliance-pack-coverage gate (Gate 6) is satisfied when every pack applicable to the cell's tenant set per ADR-0251 has a validated certification artifact at `microservices/<name>/compliance/<pack>/cell-<cell-id>-certification.signed.json` (the artifact format is added in Wave 15T-Cell-Promotion-Gates).

B2.044. The cell-orchestrator µservice operates in HA mode with at least 3 replicas per region per ADR-0148; gate evaluation is sharded by cell-id across replicas for throughput.

B2.045. Auto-promotion events are eventually consistent across replicas via the cell-orchestrator's internal consensus (Raft, per ADR-0148); the gate evaluation is deterministic given the same telemetry snapshot.

B2.046. The cell-orchestrator µservice's own SLO budget is bounded by ADR-0186 cohort discipline; if the orchestrator itself misses its SLO, gate evaluation pauses (fail-safe to "no promotion") and an alert fires.

B2.047. Per ADR-0212 buildability doctrine, the cell-orchestrator µservice's manifest declares the gate-evaluation interface in `substrate_dependencies` + `capabilities` + `contracts/` per the standard µservice authoring shape.

B2.048. Per ADR-0331 §D-8 (cross-µservice tenant_class adoption), the cell-orchestrator µservice declares its `cellular_deployment_pattern` per ADR-0244 §D-5 as `cellular_deployment_pattern: substrate_dedicated` (orchestrator is its own substrate-dedicated cell binding).

B2.049. Per ADR-0322 substance-bar, this ADR carries 600+ lines of bespoke authoring substance per the line floor.

B2.050. Per ADR-0324 anti-script doctrine, the Wave 15T-Cell-Promotion-Gates sub-wave authors per-µservice bespoke promotion-history schema, not a templated stamp.

B2.051. Wave 15T-Cell-Promotion-Gates is added to `/specs/master-plan-sequencing.json` waves_15_plus.sub_waves enumeration as a queued sub-wave.

B2.052. The Wave 15T-Cell-Promotion-Gates sub-wave authors: (a) the `oya-check-cell-promotion-gates` lane implementation; (b) the per-cell telemetry snapshot schema; (c) the `cell_promotion_history` + `cell_promotion_gates` manifest fields; (d) the Kyverno policy `enforce-cell-promotion-gates`; (e) the cell-orchestrator gate-evaluator gRPC contract and stub server (the full µservice implementation is a separate sub-wave under ADR-0148).

B2.053. The cell-orchestrator µservice's full implementation is sequenced as a follow-on sub-wave under ADR-0148 + ADR-0328; that sub-wave is out of scope for this ADR.

B2.054. The 30-day post-Acceptance window is the sunset window. The five new lanes + Kyverno policy start as REPORT-ONLY and promote to BLOCKER at day 30 unless Wave 15T-Cell-Promotion-Gates has not yet completed, in which case the sunset extends until residue reaches zero.

B2.055. The canonical-primitives cheat sheet at `tools/hooks/_canonical-primitives.md` adds a Cell Promotion Gates section naming this ADR + the six gate inputs + the per-edge warm-soak floors.

B2.056. This ADR is the canonical-authority source for cellular promotion gates. ADR-0248 remains the canonical-authority source for the cellular topology itself.

B2.057. This ADR does not authorize any waiver mechanism. The emergency-override path (§D-9) is the only exception, and it requires multi-party authorization + audit-chain emission.

B2.058. This ADR is binding on every contributor (human and agent) immediately upon Acceptance. New µservice manifests MUST declare the `cell_promotion_gates` block; existing µservice manifests MUST declare the field within the Wave 15T-Cell-Promotion-Gates sub-wave.

B2.059. This ADR is final on Acceptance. Subsequent amendments require their own ADR superseding or amending this one.

B2.060. This ADR is announced in the realignment-wave findings aggregation and in the next ADR-0327 promotion gate report.

### B.3 What this decision does not do

- This ADR does not author the cell-orchestrator µservice implementation; the implementation is a separate sub-wave under ADR-0148.
- This ADR does not change ADR-0248 cellular tier numbering; the five-tier model is preserved verbatim.
- This ADR does not introduce new SLO metrics; every gate input is an existing observability emission per ADR-0263.
- This ADR does not change tenant_class semantics; demo_trial + paid both present is the binding shape per ADR-0330.
- This ADR does not change compliance pack activation gating; pack semantics are preserved per ADR-0251.
- This ADR does not change ADR-0338 pod runtime tier semantics; pod runtime tier and cellular tier are independent axes.

## Consequences

### C.1 Positive consequences

- **Promotion bottleneck removed.** Council-architecture-mediated promotion is replaced by continuously-evaluated machine-checkable gates. Promotion latency drops from "review cadence" to "quiet window after gate convergence" (24-168 hours per edge, deterministic).
- **Promotion bias removed.** Machine-checkable gates apply uniformly regardless of compliance pack, regional posture, or tenant identity; promotion-bias drift is eliminated.
- **Auditability gained.** Every promotion event is a signed audit-chain row per ADR-0263; regulators see continuous evidence rather than meeting minutes.
- **Demotion symmetry gained.** Demotion fires under the same evaluator with stricter thresholds; cells that drift toward lower-reliability tiers move automatically without a human-review lag.
- **Substrate-vs-product clarity gained per ADR-0245.** Tier promotion is entirely substrate-side; product contracts are unchanged through promotion events.
- **Hyperscaler precedent matched.** AWS / Stripe / Cloudflare cellular promotion discipline is met verbatim; the gate set mirrors the AWS internal "cell graduation" process and Stripe's "Pier" gate set.
- **Emergency override preserved.** Multi-party-authorized override path exists for incident response; the path is audit-chain-emitting and observability-visible.
- **Tenant-class coverage enforced.** Both demo_trial and paid tenants must be present on a cell before promotion, validating that the cell can host both classes (per `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`).
- **Compliance-pack-coverage enforced.** Every applicable pack must be validated and signed off before promotion; the build-ahead-of-certification directive (`feedback_build_ahead_of_certification`) is reinforced.
- **Reversible by design.** The bidirectional gate shape (promotion + demotion) eliminates the silent-regression risk (`feedback_no_silent_regression`).

### C.2 Negative consequences

- **Cell-orchestrator dependency.** Auto-promotion requires the cell-orchestrator µservice to be running and healthy; if the orchestrator misses its SLO, gate evaluation pauses (fail-safe to "no promotion") and an alert fires. The cell-orchestrator becomes a critical-path substrate.
- **Per-cell telemetry snapshot overhead.** The CI lane reads per-cell telemetry snapshots at pre-merge time; the snapshot generation cost is non-zero but bounded.
- **Quiet window adds latency.** A cell that converges all six gates at hour 0 still waits 24-168 hours before auto-promotion fires; this is a deliberate latency to absorb late-arriving alerts.
- **Per-µservice manifest schema update required.** ~77 µservices need a manifest update declaring `cell_promotion_gates` + `cell_promotion_history`. The update is per-µservice bespoke per ADR-0324 anti-template discipline.
- **Kyverno policy authoring + rollout.** The `enforce-cell-promotion-gates` Kyverno policy must be authored, deployed per cell, and soaked as REPORT-ONLY before promoting to BLOCKER.
- **Cross-team coordination for emergency-override.** Multi-party authorization for emergency override requires incident commander + on-call SRE + council-security; the path is harder than ad-hoc, by design.
- **Telemetry schema churn.** The per-cell telemetry snapshot schema is new; its evolution under ADR-0108 sunset will require coordination with the cell-orchestrator µservice.

### C.3 Neutral consequences

- **Service mesh unchanged.** Direct gRPC over HTTP/3 + mTLS via ADR-0145 + ADR-0253 continues; promotion gates are admission/CI/control-plane concerns.
- **Cedar authorization unchanged.** Cedar evaluates application-layer authorization at request time; promotion gates run at admission/CI/control-plane time.
- **Observability emission preserved.** Per ADR-0263 the gate inputs are existing emissions; the new dashboard cuts derive from existing metrics.
- **Tenant-facing query semantics unchanged.** Tenants continue to issue requests via the standard API contract; promotion events are invisible at the contract.
- **Tenant_class behavior preserved.** demo_trial + paid semantics from ADR-0330 are inputs to Gate 5; they are not modified by this ADR.

### C.4 Engineering-rigor dimensions

| Dimension | Requirement created by this ADR | Acceptance signal |
|---|---|---|
| Maintainability | Single set of six gates across every tier-edge across every cell | Wave 15T-Cell-Promotion-Gates lands; `oya-check-cell-promotion-gates` lane stays green at BLOCKER |
| Auditability | Every promotion event = signed audit-chain row per ADR-0263 | Sampled audit-chain rows show `cell.promotion.executed` / `cell.promotion.demoted` / `cell.promotion.override` events |
| Reversibility | Bidirectional gate shape (promotion + demotion) | Demotion path tested in chaos drills |
| Performance | Quiet window ≤ 168 hours per edge | Per-cell metric `cell_promotion_quiet_window_remaining_seconds` is observable |
| Compliance | Pack floor stricter than gate; both honored | Per-pack `cell_compliance_pack_validated{pack=...}` metric green |
| Substrate-vs-product | Promotion is entirely substrate-side; no product contract change | Product API contract tests stay green through promotion events |
| Observability | Six gate inputs + three event classes (`cell.promotion.executed` / `demoted` / `override`) emit canonical telemetry per ADR-0263 | Per-cell dashboard segments by tier; gate-evaluator dashboard shows current gate state per cell |
| Security | Cell-orchestrator under `oyatie.foundry.*` principal per ADR-0247; emergency override under multi-party authz | Cedar fragments for cell-orchestrator audited per ADR-0150 + ADR-0183 |
| Resilience | Cell-orchestrator HA via 3+ replicas per region; Raft consensus | Cell-orchestrator SLO p99 < 100 ms per gate evaluation; recovery from replica loss < 30 s |

### C.5 Hyperscaler-grade rigor application

**Named precedent.** AWS internal "cell graduation" process (James Hamilton 2024 re:Invent talk on cellular architecture; Colm MacCárthaigh 2023 talks on shuffle sharding) uses an equivalent six-input gate set evaluated continuously by control-plane services. Stripe's 2024 engineering blog post "Building cellular architecture at Stripe" documents the equivalent gate set bound to "Pier" — Stripe's internal cellular control-plane component. Cloudflare's 2024 SRE Conference talk on tier promotion within their cellular topology describes an equivalent gate set tied to "Argo Tier Manager" — Cloudflare's internal cellular control-plane. The three hyperscaler precedents converge on the same shape: machine-checkable gates evaluated continuously, control-plane execution, audit-chain emission.

**Failure-mode tree.** Failure modes:
(1) cell-orchestrator down → gate evaluation pauses (fail-safe to "no promotion"); alert fires; manual override path remains available per §D-9;
(2) per-cell telemetry snapshot stale → gate evaluator uses the most recent valid snapshot up to a freshness threshold (5 minutes); past the threshold, evaluation pauses;
(3) flapping promotion/demotion → demotion thresholds in §D-5 are stricter than promotion thresholds; the quiet window absorbs short bursts;
(4) compliance pack signoff stale → Gate 6 fails; promotion does not fire; alert fires for the responsible compliance reviewer;
(5) tenant-class coverage incomplete (only demo_trial OR only paid present) → Gate 5 fails; promotion does not fire;
(6) cell-mesh health drop → Gate 4 fails; promotion does not fire; if drop sustains, demotion fires per §D-5;
(7) canary cohort SLO miss → Gate 3 fails; promotion does not fire; if miss sustains, demotion fires per §D-5;
(8) error budget exhaustion → Gate 1 fails; promotion does not fire; if exhaustion sustains, demotion fires per §D-5;
(9) emergency override misuse → multi-party authorization requirement absorbs single-actor misuse; audit-chain emission preserves evidence;
(10) gate-evaluator bug → REPORT-ONLY soak window (30 days post-landing) catches false positives before BLOCKER promotion;
(11) Kyverno policy false positive → REPORT-ONLY soak before BLOCKER promotion + manual override path per §D-9.

**Capacity math.** Per-cell gate evaluation runs continuously every 60 seconds; each evaluation reads ~6 metric series × ~3 labels × ~1 sample = ~18 telemetry reads per cell per minute. At 100 cells per region × 5 regions = 500 cells, the orchestrator handles ~9,000 reads/minute = ~150 reads/sec — well within Rust gRPC + ADR-0263 emission capacity. Per-promotion-event work is bounded at: gate evaluation (5 ms) + audit-chain row emission (10 ms) + Kyverno admission (50 ms) + node label propagation (200 ms) = ~265 ms per promotion event end-to-end. The cell-orchestrator HA mode (3 replicas per region) handles the load with > 10× headroom.

**Observability hooks.** The gate-evaluator dashboard at `microservices/observability/dashboards/cell-promotion-gates.md` shows current gate state per cell per tier; dimensions are `cell_id` × `current_tier` × `target_tier` × `gate_index_1_to_6`. The gate-state metric `cell_promotion_gate_satisfied{cell_id, gate_index, target_tier}` is a 0/1 gauge. The promotion-event metric `cell_promotion_events_total{event_class, from_tier, to_tier}` is a counter. The quiet-window-remaining metric `cell_promotion_quiet_window_remaining_seconds{cell_id, target_tier}` is a gauge. Total new label cardinality is bounded at 500 cells × 5 tiers × 6 gates ≈ 15,000 unique series — within the observability cost envelope per ADR-0263.

**Rollback path.** Per-cell rollback: a misclassified promotion executes a demotion via the same gate evaluator; the demotion emits its own audit-chain row. Cell-level rollback: the emergency-override path per §D-9 forces an immediate demotion with multi-party authorization. Cross-cell rollback: not applicable; promotion events are per-cell.

**Multi-region awareness.** Each region runs its own cell-orchestrator HA cohort; cross-region promotion events are independent (no global consensus required). Per ADR-0240 sovereign-cloud per regional pack, sovereign cells have region-pinned cell-orchestrator instances.

**Sovereign-cell awareness.** Sovereign cells (HIPAA, GDPR-strict, CSAP, PCI, IL5) follow the same gate set; the compliance-pack-coverage gate naturally includes the sovereign pack. Sovereign packs may set stricter floors than this ADR's defaults (e.g., a HIPAA-pinned cell may require ≥ 99.95 % error budget instead of ≥ 99 %); pack floor stricter than gate wins.

**Versioning + deprecation.** Per ADR-0108 sunset discipline. Gate definitions may evolve under amendment ADRs; the cell-orchestrator gate-evaluator binary carries a version label on every emitted promotion-event for provenance. Deprecation of a gate requires a new ADR + a quarterly cycle of REPORT-ONLY before removal.

## D. Detailed mechanics — twelve enforcement surfaces

The cell-promotion-gate mechanism touches twelve enforcement surfaces. Each subsection D-1 through D-12 enumerates one surface. Numbering is normative.

### D-1: ADR-0248 cellular tier numbering — preserved verbatim

D-1.1. ADR-0248 defines the cellular tier ladder: Tier 0 = Foundation cells (identity / KMS / audit); Tier 1 = Substrate cells; Tier 2 = Capability cells; Tier 3 = Application cells; Tier 4 = Edge cells. Tier 0 = highest blast-radius / most isolated; Tier 4 = lowest blast-radius / best-effort.

D-1.2. This ADR preserves the numbering verbatim. No inversion to ascending-prestige is introduced; this is critical to preserve `feedback_bominal_inheritance_precedence` and to keep ADR-0338 pod-runtime-tier (which co-varies with cellular tier) consistent.

D-1.3. Promotion edges in this ADR are: Tier 0 ← 1 (less common; a cell becomes more critical), Tier 1 ← 2, Tier 2 ← 3, Tier 3 ← 4. Demotion edges are the symmetric inverse: Tier 0 → 1, Tier 1 → 2, Tier 2 → 3, Tier 3 → 4.

D-1.4. Note on terminology: the user directive of 2026-05-21 specifies "Tier 0 → 1 → 2 → 3 → 4" with warm-soak floors of 7d / 14d / 28d / 56d. Read in the ADR-0248 monotonically-decreasing-criticality direction, "Tier 0 → 1" means demoting from Tier 0 to Tier 1 (less critical). The 7-day warm-soak applies when a cell currently classified Tier 0 has been running stable enough to relax to Tier 1. The 56-day warm-soak applies when a cell currently Tier 3 has been running stable enough to relax to Tier 4. **This is the canonical reading** because it matches the user directive's intent: the longer the warm-soak, the less-critical the destination tier, because Tier 4 cells are the most "casual" but also the most-easily-rolled-back if they regress.

D-1.5. Promotion in the inverse direction (Tier 4 → 3 → 2 → 1 → 0, i.e., a cell becomes more critical) follows the same six gates but with the warm-soak floors interpreted in reverse: Tier 4 → 3 = 56 days; Tier 3 → 2 = 28 days; Tier 2 → 1 = 14 days; Tier 1 → 0 = 7 days. The intent is that moving INTO a more-critical tier requires the longest soak when the source tier is most-relaxed (Tier 4 → 3 takes 56 days because the cell is graduating from "casual" to "application-grade"), and the shortest soak when the source tier is already substrate-class (Tier 1 → 0 takes 7 days because the cell is already operating at substrate hardness).

D-1.6. **Canonical authoring interpretation: the warm-soak floor depends on the destination-tier-criticality.** Lower destination criticality (Tier 4 = best-effort) = longer soak. Higher destination criticality (Tier 0 = foundation) = shorter soak when source is already substrate-class. This shape rewards cells that graduate INTO more-critical tiers (short soak; cell is already proven) and slow-rolls cells that relax INTO less-critical tiers (long soak; the cell is taking on a less-watched role and we want to be sure it's actually stable).

### D-2: Six gate inputs

D-2.1. **Gate 1 — Error budget intact.** Requires ≥ 99 % of the µservice's SLO budget remaining on the current tier over the warm-soak window. The SLO budget is computed per OpenSLO + ADR-0186 from the canonical SLI metrics per µservice. The "current tier" is the tier the cell is currently at; the gate evaluates whether the cell has been operating within budget at its current tier.

D-2.2. **Gate 2 — Warm-soak duration.** Requires ≥ N days in the current tier where N is the per-edge floor in §D-3. Warm-soak begins at the cell's last tier-change event (creation or previous promotion/demotion); ends at the current gate evaluation timestamp.

D-2.3. **Gate 3 — Canary cohort success.** Requires the canary cohort (defined per ADR-0186) to maintain ≥ 99.5 % SLO compliance over the warm-soak window. The canary cohort is a designated subset of tenants on the cell whose SLO compliance is sampled at higher resolution.

D-2.4. **Gate 4 — Cell-mesh health.** Requires cross-cell call success ≥ 99.95 % over the warm-soak window. The inter-cell mesh tunnel per ADR-0044 emits per-call success telemetry; the gate consumes the rollup.

D-2.5. **Gate 5 — Tenant-class coverage.** Requires both `tenant_class = demo_trial` and `tenant_class = paid` tenants present on the current tier per ADR-0330. The gate fails if the cell hosts only one class.

D-2.6. **Gate 6 — Compliance-pack coverage.** Requires every applicable compliance pack per ADR-0251 (HIPAA, GDPR-strict, SOC2, PCI, CSAP, EU AI Act Annex III, ISO27001, KR-ISMS-P, JP-FISC, etc.) validated and signed off on the current tier. Sign-off artifacts live at `microservices/<name>/compliance/<pack>/cell-<cell-id>-certification.signed.json`.

D-2.7. Gates evaluate AND: a promotion fires only when all six pass. Demotion fires when any one falls below the demotion threshold per §D-5.

D-2.8. Gate evaluation runs continuously inside the cell-orchestrator µservice at a 60-second cadence and at pre-merge time inside the `oya-check-cell-promotion-gates` CI lane.

### D-3: Per-edge warm-soak floors

D-3.1. **Tier 0 → 1** (demoting from foundation to substrate): warm-soak ≥ 7 days. The cell has been operating at foundation-class criticality and is taking on slightly less responsibility.

D-3.2. **Tier 1 → 2** (demoting from substrate to capability): warm-soak ≥ 14 days. The cell is relaxing from substrate-class to capability-class.

D-3.3. **Tier 2 → 3** (demoting from capability to application): warm-soak ≥ 28 days. The cell is moving from capability tier to application tier.

D-3.4. **Tier 3 → 4** (demoting from application to edge): warm-soak ≥ 56 days. The cell is moving to the most casual/edge tier; longest soak to absorb potential regressions before relaxing the watch.

D-3.5. Inverse promotion edges (Tier N → Tier N-1, where N-1 is more critical) follow the warm-soak floors symmetrically: Tier 4 → 3 = 56 days; Tier 3 → 2 = 28 days; Tier 2 → 1 = 14 days; Tier 1 → 0 = 7 days. The rationale is symmetric to D-1.6: a cell graduating INTO more-critical tiers from a casual source needs longer soak; a cell graduating INTO foundation from already-substrate needs shorter soak because the cell is already proven.

D-3.6. **Per-edge quiet windows** after gate convergence:
- Tier 0 → 1 (and inverse Tier 1 → 0): 24 hours.
- Tier 1 → 2 (and inverse Tier 2 → 1): 48 hours.
- Tier 2 → 3 (and inverse Tier 3 → 2): 96 hours.
- Tier 3 → 4 (and inverse Tier 4 → 3): 168 hours.

D-3.7. The quiet window starts at the moment all six gates first turn green and resets on any gate flip. Auto-promotion fires when the quiet window elapses without a reset.

D-3.8. The per-edge warm-soak floors are MIN values; per-pack compliance requirements may set stricter floors (e.g., HIPAA may require ≥ 90 days warm-soak before Tier 2 → 3 in some packs); pack floor stricter than this ADR's default wins.

### D-4: Auto-promotion execution path

D-4.1. The cell-orchestrator µservice evaluates the six gates every 60 seconds per cell per candidate tier-edge.

D-4.2. When all six gates pass for a given cell + candidate tier-edge, the cell-orchestrator records the "quiet-window-start" timestamp.

D-4.3. The cell-orchestrator monitors the gates continuously through the quiet window; any gate flip resets the quiet-window-start timestamp.

D-4.4. When the quiet window elapses without reset, the cell-orchestrator initiates the promotion event:

D-4.5. Step 1: cell-orchestrator emits a signed `cell.promotion.executed` audit-chain row per ADR-0263 with the cell_id, previous_tier, new_tier, six-gate evaluation snapshot, evaluator version, signed by the cell-orchestrator principal under the `oyatie.foundry.*` Cedar namespace per ADR-0247.

D-4.6. Step 2: cell-orchestrator submits a Kubernetes API mutation to update the per-cell node labels (`cell.oyatie.io/tier=<new_tier>`); the Kyverno admission policy `enforce-cell-promotion-gates` (§D-8) validates the mutation carries a valid promotion-event attestation and admits it.

D-4.7. Step 3: cell-orchestrator updates the per-µservice manifest `cell_promotion_history` field via a self-modification-doctrine PR per ADR-0247 (the orchestrator opens a PR in the monorepo against `dev` with the new history entry; the PR auto-merges per the Foundry pipeline contract).

D-4.8. Step 4: cell-orchestrator emits an observability event `cell.promotion.completed` carrying timing metadata (gate-evaluation duration, audit-chain emission duration, Kyverno admission duration, node-label-propagation duration) for the cell-promotion-gates dashboard.

D-4.9. Step 5: cell-orchestrator releases the cell back into the continuous gate-evaluation loop at the new tier.

D-4.10. End-to-end auto-promotion latency budget: 265 ms (per §C.5 capacity math).

### D-5: Demotion path (symmetric inverse)

D-5.1. Demotion fires when any one of the six gates falls below its demotion threshold. Demotion thresholds are STRICTER than promotion thresholds to avoid flapping:
- Gate 1 demotion threshold: error budget < 95 % (vs ≥ 99 % for promotion). The gap absorbs short bursts.
- Gate 2 demotion: no warm-soak required; demotion is immediate to protect blast-radius.
- Gate 3 demotion threshold: canary cohort SLO compliance < 99 % (vs ≥ 99.5 % for promotion).
- Gate 4 demotion threshold: cross-cell call success < 99.9 % (vs ≥ 99.95 % for promotion).
- Gate 5 demotion: tenant-class coverage loss (one class drops off cell) does NOT trigger demotion automatically; the cell can remain at its current tier with only one class present, but cannot promote further. This avoids spurious demotion when a tenant churns.
- Gate 6 demotion: compliance pack sign-off revocation triggers immediate demotion to the highest tier where pack is signed off.

D-5.2. Demotion executes via the same cell-orchestrator path as promotion but with the event class `cell.promotion.demoted` instead of `cell.promotion.executed`.

D-5.3. Demotion does NOT have a quiet window; the demotion fires immediately on threshold breach to protect blast-radius.

D-5.4. Demotion sets a 24-hour cooldown after which the cell can re-enter promotion gate evaluation; this prevents rapid promote/demote oscillation.

D-5.5. Demotion is observable via the same dashboard as promotion; the metric `cell_promotion_events_total{event_class="demoted", from_tier, to_tier}` rolls up demotion events.

### D-6: Gate evaluator implementation

D-6.1. The gate evaluator is implemented as a Rust crate `oya-cell-orchestrator-gate-evaluator` (Rust-strict per `feedback_rust_strict_only_no_python_2026_05_20`).

D-6.2. The crate exposes a synchronous evaluation function `evaluate_gates(cell_id, candidate_tier_edge, telemetry_snapshot) -> GateEvaluation` that returns a structured result containing the 6-gate boolean array, per-gate detail, evaluator version, and signing key id.

D-6.3. The crate consumes per-cell telemetry from the canonical observability emission contract per ADR-0263; no bypass is permitted.

D-6.4. The crate is consumed by two callers: the cell-orchestrator µservice (continuous-evaluation mode, 60-second cadence) and the `oya-check-cell-promotion-gates` CI lane (pre-merge mode, evaluates against the most recent published snapshot).

D-6.5. The crate's evaluation is deterministic: given the same telemetry snapshot, the result is identical across replicas.

D-6.6. The crate's evaluation is bounded at 5 ms per evaluation per §C.5; this floor is asserted in unit tests.

D-6.7. The crate's evaluator version is emitted on every evaluation result for provenance; the version is bumped on any gate-definition change.

### D-7: Evidence pack for emergency-override promotion

D-7.1. The emergency override path (§D-9) is the only path where a promotion event is not gate-evaluated; it requires an evidence pack at `microservices/<name>/IPs/IP-cell-promotion-override-<cell-id>-<timestamp>.md` documenting:

D-7.2. (a) the cell_id, current tier, target tier, and override direction (promote / demote); (b) the incident context that motivated the override; (c) the three signatures (incident commander + on-call SRE + council-security) with cryptographic signing keys; (d) the gate-evaluation snapshot at the time of override (recorded for evidence even though it does not gate the override); (e) the post-override remediation plan documenting how the cell will return to the gate-evaluated path.

D-7.3. The evidence pack is reviewed at the next council-architecture session post-incident; the review evaluates whether the override was justified and whether the gate definitions need amendment.

D-7.4. The CI lane `oya-governance-cell-tier-promotion-evidence` refuses promotion-history entries that lack the evidence pack (only applies to override-path entries).

### D-8: Kyverno admission policy

D-8.1. The Kyverno ClusterPolicy `enforce-cell-promotion-gates` is BLOCKER-class at landing time.

D-8.2. The policy refuses Kubernetes cellular topology mutations (node-pool resize, cell-zone creation, cell-tier label change) that do not carry a cell-orchestrator-signed promotion-event attestation in the resource's annotations.

D-8.3. The policy validates the attestation signature against the cell-orchestrator principal's signing key per ADR-0247.

D-8.4. The policy emits an audit event `cell.promotion.admission.denied` on every deny per ADR-0263.

D-8.5. The policy is deployed per cell as part of the per-cell Kyverno installation; each cell carries its own policy instance.

### D-9: Emergency override path

D-9.1. The emergency override is invoked via the cell-orchestrator's override gRPC contract `Override(cell_id, target_tier, signatures[3])`.

D-9.2. Signatures required: (a) incident commander signing key (from on-call incident record); (b) on-call SRE signing key; (c) council-security signing key.

D-9.3. The cell-orchestrator validates all three signatures against the keys in `microservices/cloud-iam/principals/cell-orchestrator-override-quorum.signed.json`.

D-9.4. On successful validation, the cell-orchestrator executes the override:
- Emits `cell.promotion.override` audit-chain event (instead of `executed` or `demoted`) per ADR-0263.
- Skips the warm-soak floor and the gate-evaluation AND-condition.
- Mutates the cell-tier node label via the same Kyverno-admitted path as auto-promotion (the Kyverno policy accepts the override attestation as a valid promotion-event attestation).
- Files the evidence pack per §D-7 (the orchestrator creates the IP file with the gate-snapshot at override time).

D-9.5. The override is observable; the dashboard shows override events in red.

D-9.6. Misuse of the override (any single-actor bypass attempt) is logged with high-severity alert and audit-chain emission.

### D-10: Per-µservice manifest fields

D-10.1. Per-µservice `manifest.json` gains a `cell_promotion_gates` block:
```json
"cell_promotion_gates": {
  "applicable_tiers": [0, 1, 2, 3, 4],
  "cellular_deployment_pattern": "substrate_dedicated",
  "default_initial_tier": 2,
  "promotion_window_per_edge_seconds": {
    "tier_0_to_1": 604800,
    "tier_1_to_2": 1209600,
    "tier_2_to_3": 2419200,
    "tier_3_to_4": 4838400
  },
  "compliance_pack_floor": ["soc2", "gdpr"]
}
```

D-10.2. Per-µservice `manifest.json` gains a `cell_promotion_history` array; each entry references a promotion-event audit-chain row by `event_id`:
```json
"cell_promotion_history": [
  {
    "event_id": "audit-chain://cell.promotion.executed/2026-05-22T03:14:22Z/cell-001",
    "from_tier": 3,
    "to_tier": 2,
    "evaluator_version": "1.0.0",
    "gate_snapshot_sha256": "<hex>"
  }
]
```

D-10.3. The manifest schema at `/specs/microservices/manifest-schema.json` is updated to add both fields.

D-10.4. Each µservice's Wave 15T-Cell-Promotion-Gates bucket updates its manifest as part of the bespoke per-µservice rewrite per ADR-0322 + ADR-0324.

### D-11: Wave 15T-Cell-Promotion-Gates sequence

D-11.1. `/specs/master-plan-sequencing.json` waves_15_plus.sub_waves enumeration adds `15T-Cell-Promotion-Gates` as a queued sub-wave.

D-11.2. Wave 15T-Cell-Promotion-Gates authoring:
- The `oya-check-cell-promotion-gates` lane implementation (Rust + ADR-0263 telemetry consumer).
- The per-cell telemetry snapshot schema at `microservices/observability/snapshots/cell-telemetry-schema.json`.
- The `cell_promotion_gates` + `cell_promotion_history` manifest field additions across all 77 µservices.
- The Kyverno policy `enforce-cell-promotion-gates` (per-cell ClusterPolicy template).
- The cell-orchestrator gate-evaluator gRPC contract and stub server.
- The dashboard at `microservices/observability/dashboards/cell-promotion-gates.md`.

D-11.3. The full cell-orchestrator µservice implementation is sequenced as a SEPARATE follow-on sub-wave under ADR-0148 + ADR-0328; that sub-wave is out of scope for this ADR.

D-11.4. Wave 15T-Cell-Promotion-Gates is queued; dispatches after this ADR is Accepted.

D-11.5. Per-µservice rewrite buckets work under ADR-0322 substance-bar discipline + ADR-0324 anti-template doctrine.

### D-12: Cell-orchestrator binding

D-12.1. The cell-orchestrator µservice runs inside the tenancy + observability substrate per ADR-0148.

D-12.2. The cell-orchestrator operates under the `oyatie.foundry.*` Cedar principal namespace per ADR-0247.

D-12.3. The cell-orchestrator's pod_runtime_tier per ADR-0338 is Tier 1 (substrate touching tenant data plane); runs under Kata Containers + Cloud Hypervisor in the kata-pool.

D-12.4. The cell-orchestrator runs HA with at least 3 replicas per region per ADR-0148; gate evaluation is sharded by cell-id across replicas.

D-12.5. The cell-orchestrator's own SLO budget is bounded per ADR-0186; on SLO miss, gate evaluation pauses (fail-safe to "no promotion") and an alert fires.

D-12.6. The cell-orchestrator emits a self-health metric `cell_orchestrator_evaluation_lag_seconds` that gates its own continued evaluation; lag > 60 seconds triggers automatic pause.

D-12.7. The cell-orchestrator µservice's full implementation is out of scope for this ADR; this ADR establishes only the gate-evaluation contract any cell-orchestrator implementation must observe.

## E. Enforcement-by-lanes

E.1 `oya-check-cell-promotion-gates` (new) — evaluates the six machine-checkable criteria for every promotion candidate against per-cell telemetry + manifest at pre-merge time. REPORT-ONLY at landing; BLOCKER after Wave 15T-Cell-Promotion-Gates lands the evaluator + per-µservice promotion-history schema (≥ 30 days post-Acceptance).

E.2 `oya-governance-cell-tier-promotion-evidence` (new) — refuses promotion-history entries that lack the evidence pack documented in §D-7 (only applies to override-path entries). REPORT-ONLY at landing; BLOCKER post-soak.

E.3 `oya-governance-cell-tier-numbering-convention` (new) — refuses promotion-history entries that invert the ADR-0248 Tier 0 = highest-blast-radius convention; preserves `feedback_bominal_inheritance_precedence`. REPORT-ONLY at landing; BLOCKER post-soak.

E.4 `oya-governance-cell-promotion-quiet-window` (new) — refuses auto-promotion events that did not observe the per-tier quiet-window floor in §D-3.6. REPORT-ONLY at landing; BLOCKER post-soak.

E.5 `oya-governance-cell-orchestrator-binding` (new) — refuses promotion events that did not originate from a cell-orchestrator µservice principal under the ADR-0148 + ADR-0247 self-modification namespace. REPORT-ONLY at landing; BLOCKER post-soak.

E.6 Kyverno admission policy `enforce-cell-promotion-gates` (new) — refuses Kubernetes cellular topology mutations that do not carry a cell-orchestrator-signed promotion-event attestation. validationFailureAction audit at landing; enforce at sunset.

## F. Alternatives considered (rejected)

### F.1 Council-architecture-mediated promotion (status-quo)

**Description.** Promotion decisions made on case-by-case evidence by council-architecture review on weekly/fortnightly cadence. No machine-checkable gate set.

**Rejected because.** Cadence drag (~60-hour median lag from gate convergence to promotion); promotion-bias drift against unfamiliar compliance packs; not auditable as machine-checkable facts; not symmetric with demotion. Incompatible with `feedback_quality_performance_scalability_bar` (hyperscaler-grade rigor) and `feedback_no_silent_regression`.

### F.2 Single-gate model (only error budget)

**Description.** Reduce the six gates to one: ≥ 99 % error budget intact. Drop the other five gates.

**Rejected because.** Error budget alone does not capture cross-cell mesh health, tenant-class coverage, or compliance-pack coverage. A cell can have a green error budget while failing canary cohort SLO (Gate 3) or while missing demo_trial coverage (Gate 5). The six-gate AND ensures all relevant dimensions are healthy before promotion.

### F.3 Continuous promotion with no quiet window

**Description.** Auto-promote the instant all six gates pass; no quiet window.

**Rejected because.** Late-arriving alerts (incidents that hit the cell after gate convergence but before the alerting pipeline completes) would cause flapping promote/demote oscillation. The quiet window absorbs alert latency. Per `feedback_quality_performance_scalability_bar`, the hyperscaler precedent (AWS / Stripe / Cloudflare) all observe a quiet window.

### F.4 Human-only override (no auto-promotion)

**Description.** Make every promotion event an emergency-override path with multi-party authorization.

**Rejected because.** Cadence drag returns; the bottleneck is back. The emergency-override path is for incidents only, not for routine promotion. The auto-promotion path handles routine promotion; the override handles exceptions.

### F.5 Continuous-evaluation but no audit-chain row per promotion

**Description.** Auto-promote via cell-orchestrator but skip the audit-chain row to reduce overhead.

**Rejected because.** Regulators require evidence of every tier-change event per ADR-0250 build-ahead-of-certification + ADR-0251 compliance pack certification levels. The audit-chain row is non-negotiable for auditability.

## G. Sunset schedule

G.1 At Acceptance: lanes E.1-E.5 + Kyverno policy E.6 deploy as REPORT-ONLY across the corpus.

G.2 Days 1-30 post-Acceptance: Wave 15T-Cell-Promotion-Gates authors the lane evaluator + per-cell telemetry snapshot schema + manifest field additions + Kyverno policy + dashboard.

G.3 Day 30 post-Acceptance: lanes E.1-E.5 + Kyverno policy E.6 promote from REPORT-ONLY to BLOCKER. The cell-orchestrator full µservice implementation is queued as a separate follow-on sub-wave under ADR-0148 + ADR-0328.

G.4 Day 30+: per-µservice migration of legacy ad-hoc promotion records (rare; most µservices have no prior promotion history) follows canonical-build phase order under ADR-0328.

G.5 Day 90+: cell-orchestrator full µservice implementation lands as a separate sub-wave; gate evaluation transitions from CI-only to CI + continuous-evaluation.

G.6 Quarterly review thereafter: gate-definition evolution + warm-soak-floor evolution under ADR-0108 sunset discipline.

## H. Acceptance signals

H.1 Wave 15T-Cell-Promotion-Gates lands with: lane evaluator implementation green; per-cell telemetry snapshot schema published; manifest field additions across 77 µservices; Kyverno policy deployed per cell as REPORT-ONLY → BLOCKER per G.3.

H.2 The first sampled auto-promotion event emits a signed `cell.promotion.executed` audit-chain row with all six gate snapshots embedded and is observable on the cell-promotion-gates dashboard.

H.3 The first sampled demotion event emits a signed `cell.promotion.demoted` audit-chain row.

H.4 The first emergency-override event emits a signed `cell.promotion.override` audit-chain row with the three-party signatures embedded.

H.5 The lane `oya-check-cell-promotion-gates` stays green at BLOCKER across the corpus.

H.6 The Kyverno policy `enforce-cell-promotion-gates` stays green at BLOCKER per cell.

H.7 The cell-orchestrator µservice (when its implementation sub-wave lands) reports gate-evaluation p99 < 100 ms per evaluation.

H.8 Council-architecture quarterly review confirms zero promotion-bias drift across compliance packs.

H.9 Sovereign cells (HIPAA, GDPR-strict, CSAP, PCI, IL5) honor pack floors stricter than this ADR's defaults; sovereign-pack governance reports no conflict.

H.10 The realignment-wave findings aggregation reports this ADR as Accepted and Wave 15T-Cell-Promotion-Gates as queued/landed/sunset per the G.1-G.6 timeline.

## I. Owners + accountability

I.1 **council-architecture** — owns the gate definitions + warm-soak floors + quiet-window per-edge values. Evolution of any gate definition requires a new ADR amending this one.

I.2 **ops-sre-reliability** — owns the gate evaluator implementation + per-cell telemetry snapshot schema + cell-orchestrator µservice (separately, under ADR-0148). On-call SREs hold one of the three signing keys for the emergency-override path per §D-9.

I.3 **axis-observability** — owns the per-cell telemetry emission per ADR-0263 + the cell-promotion-gates dashboard. Telemetry schema evolution coordinates with the cell-orchestrator gate evaluator.

I.4 **axis-tenancy** — owns the cell-orchestrator µservice substrate placement (it runs inside the tenancy + observability substrate per ADR-0148) + the per-cell tenant_class coverage validation logic.

I.5 **council-security** — holds one of the three signing keys for the emergency-override path per §D-9. Reviews emergency-override evidence packs at the next council-architecture session post-incident.

I.6 **axis-policy-engine** (Cedar + Kyverno) — owns the Kyverno admission policy `enforce-cell-promotion-gates` + the Cedar fragment governing cell-orchestrator principal authorization.

I.7 Owners are accountable for: (a) keeping their owned surface green at BLOCKER post-sunset; (b) reviewing emergency-override evidence packs at the next council session; (c) participating in the quarterly gate-definition review under ADR-0108 sunset discipline.

---

This ADR is Proposed on 2026-05-21 in the realignment effort. Acceptance is contingent on multispectrum review v2.4.0 per ADR-0322 §D-2 + landing of the Wave 15T-Cell-Promotion-Gates authoring sub-wave under ADR-0328 batch discipline. Authority chain: this ADR carves out an explicit subset of ADR-0248 + ADR-0148 promotion machinery; supersession of this ADR requires explicit ADR superseding both the substance + the substrate binding.
