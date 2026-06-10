# Source ADR Audit — Chunk 16

- **Side:** SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`)
- **Chunk:** 16
- **Slice requested:** lines 106–112 of sorted `docs/decisions/ADR-*.md` → ADR-0131 … ADR-0137
- **ADRs actually reviewed (7):** ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0135, ADR-0136, ADR-0137
- **Auditor posture:** read against keystone map (canonical-posture-and-supersession-map.md). Cross-checked ADR-0335/0347/0362/0512/0138 on disk to settle the foundry-cluster dispositions.

This is a dense, internally-coherent cluster authored mostly in one 2026-05-17/18 session: it lays down oyatie's **repo-structure + governance-discipline doctrine** (flat per-service layout, no-grouping, continuous conformance, honest-enforcement gates) plus the **foundry topology** (6→1 consolidation + BC partition). Two distinct sub-clusters:
- **Structure/governance** (0131, 0132, 0133, 0134, 0135) — mostly TRUE/live, but 0131 and 0132 carry a vocabulary fault-line (`microservices/<ms>/` root + "foundry" brand) that later ADRs already amended/retired.
- **Foundry topology** (0136, 0137) — sound architecture decision, but written entirely in retired "foundry" brand vocabulary and the `microservices/foundry/` path that ADR-0335 + ADR-0131-amendment + ADR-0512 have all moved on from.

---

### ADR-0131 — Per-microservice flat layout (universal)

- **decision_atom:** Every service ships as one self-contained flat folder colocating all its artifacts (PRD, phase specs, IPs, contracts, specs, catalog, runbooks, threat-model, SLOs, IaC, crates, tests, evidence); type-based central folders are reserved only for genuinely cross-cutting items, and aggregation indices become generated views.
- **current_status:** Accepted (2026-05-17), with a 2026-06-02 "pure split" amendment moving the service root from `microservices/<ms>/` to `{oya,cloud}/<service>/` per ADR-0512.
- **disposition:** KEEP (the colocation principle is current and load-bearing) — but flag the residual `microservices/<ms>/` examples throughout the body as AMEND-on-sight provenance, not destination.
- **governing:** n/a (this ADR is itself governing; it partially supersedes ADR-0015's docs/crates split and ADR-0119's per-product spec slice; it is in turn root-amended by ADR-0512).
- **truth_flag:** TRUE (principle) / PARTIAL (the body is half-migrated: the §Decision and §Naming blocks use the new `{oya,cloud}/` root, but the migration tables, DAG, completion-gate, and Open-Questions still speak in `microservices/<ms>/` — stale-by-design but internally inconsistent).
- **in_masterplan:** YES — `planning_impact: true`, `related_specs` includes `/specs/per-microservice-flat-layout.json` and `/specs/masterplan.json`. This is canonical structural authority and is masterplan-bound. Carries proper planning front-matter.
- **tensions:**
  - ADR-0512 (`canonical-monorepo-pattern`) — the root rename is bolted on as a prose amendment rather than a clean supersede edge; `superseded_by:[]` stays empty though the original `microservices/` destination is now legacy. Cross-ref discipline drift (keystone §0.6 pattern).
  - ADR-0132 — sibling enforcement still written against `microservices/<ms>/`; both need the same root-migration sweep.
  - ADR-0136/0137 — built `microservices/foundry/` on this ADR's old root; foundry brand later retired by ADR-0335. The "foundry-runtime/foundry-supervisor" examples in the migration DAG (Tier 1a–1c) are retired-vocabulary leakage.
  - Internal: §Negative says "~15 migration IPs" while §Consequences header says "30 migration IPs" and the final table says "25 migration IPs (was ~63)" — three different counts in one doc (fabricated-precision smell).
- **hyperscaler_challenge:** ALIGNED. AWS/Google(google3)/Microsoft/Stripe genuinely colocate per-service docs+code+contracts+runbooks in a monorepo; the ADR cites the real precedents accurately. The one over-reach is the *mandatory uniform* shape with a BLOCKER gate refusing any out-of-layout artifact — hyperscalers tolerate more per-team variance — but that is a defensible agentic-monorepo choice, not misalignment. Argues KEEP.
- **ai_slop:** Minor — the migration-cost table ("≤30s", "≈1500 files", "≈3h cumulative", "≈1 working day") is fabricated precision with no evidence link; the IP-count contradiction noted above; some redundancy between §Consequences and §Operational. Not disqualifying.
- **refinement:** (1) Do the root-rename as a real supersede/amends edge to ADR-0512 in front-matter, not just prose. (2) Sweep the body once: replace all `microservices/<ms>/` with `{oya,cloud}/<service>/` so provenance lives in one clearly-labeled "legacy paths" callout instead of leaking into every table. (3) Reconcile the three migration-IP counts to one number. (4) Drop or footnote the fabricated wall-time estimates.
- **consensus_needed:** no (principle is settled). The only open item is mechanical (root-rename hygiene), not a founder ruling.

---

### ADR-0132 — No-grouping forward-policy (universal flat catalog)

- **decision_atom:** No new bundle/suite/vertical/industry-named grouping µservice may be created; every new service is one flat single-concern µservice, and customer-facing packaging resolves to concrete services + tenant/RBAC entitlements, never to a deployable grouping boundary.
- **current_status:** Accepted (2026-05-17), amended 2026-05-25 by ADR-0362 (the grandfather clause for existing grouping wrappers is superseded; grouping is now flat-only, presentation-tag only).
- **disposition:** KEEP (forward-policy is current and reinforced by ADR-0362) — AMEND the body to drop the now-superseded grandfather/"out of scope, remain as authored" language and the retired `microservices/<ms>/` root.
- **governing:** ADR-0362 governs the grandfather-clause removal (amends, does not supersede — the forward-policy core survives). This ADR amends ADR-0132's own grandfather exception, not the whole decision.
- **truth_flag:** TRUE (forward-policy) / STALE (the "existing grouping wrappers are out of scope / remain as authored" stance, and the `Foundry`/`Workflow`/`Cloud` examples — "Foundry" is a retired brand per ADR-0335; the allowlist exemptions for `/specs/microservices/tenant-rbac.json` etc. need re-checking against ADR-0362 flat-only).
- **in_masterplan:** YES — `planning_impact: true`, `related_specs: [/specs/per-microservice-flat-layout.json]`. Masterplan-bound structural policy.
- **tensions:**
  - ADR-0362 — explicitly amends this ADR (named in 0132's own Status block); the supersession of the grandfather clause is recorded in prose but front-matter `superseded_by:[]` stays empty. Drift.
  - ADR-0335 — uses "Foundry" as a live grouping-wrapper example; that brand is retired, so the example is stale-vocab.
  - ADR-0131 — declared sibling; same `microservices/<ms>/` → `{oya,cloud}/<service>/` root-rename debt.
  - References `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md` — a memory file whose own filename advertises it as retired; citing a retired memory as live policy basis is a smell.
- **hyperscaler_challenge:** ALIGNED. AWS/Google/Stripe do ship per-surface services with independent SLO/release/scale ownership and treat "suite"/"vertical" as marketing, not architecture. The single-SLO-across-concerns failure mode the ADR cites is a real anti-pattern. Argues KEEP.
- **ai_slop:** Low. The §Positive "independent scaling" paragraph is somewhat padded with per-service scaling-dimension examples for services that may not exist yet (ehr/payments "(future)") — mild aspirational filler. Otherwise crisp.
- **refinement:** (1) Record the ADR-0362 amendment as a real `superseded_by`/`amended_by` edge. (2) Replace retired-brand examples ("Foundry") with current names (intelligence/governance). (3) Migrate `microservices/<ms>/` to `{oya,cloud}/<service>/`. (4) Re-point or inline the retired `feedback_workflow_objectgraph_adapter_layer` reference to ADR-0145.
- **consensus_needed:** no (policy settled; amendments mechanical).

---

### ADR-0133 — Industry-best-practice + hyperscaler-grade conformance program

- **decision_atom:** Run a continuous 6-axis (pipeline, directory, naming, standards, practices, policies) conformance program that audits every repo artifact against named industry baselines plus an agentic-dev-team optimization overlay, enforced by a BLOCKER CI lane with quarterly baseline refresh.
- **current_status:** Accepted (2026-05-17). No supersession.
- **disposition:** KEEP — light AMEND for retired-vocabulary and CI-stack churn (it predates the Jenkins→Argo and `oya-foundry-*`→`oya-governance-*` settlements; some lane/path names are stale).
- **governing:** n/a (governing program ADR).
- **truth_flag:** TRUE (program intent) / PARTIAL (some cited surfaces drifted: lane crate path `microservices/governance/src/crates/oya-check-...`, remediation IPs under `microservices/governance/`, Cedar fragments under `microservices/<ms>/policy/*.cedar` — all on the retired `microservices/` root; the pipeline taxonomy `build→…→deploy` predates the ADR-0392/0408/0511/0513 Buck2+Argo settlement but is generic enough to remain valid).
- **in_masterplan:** PARTIAL — front-matter is missing `planning_impact:` (unlike its siblings 0131/0132), yet it declares BLOCKER CI lanes and `related_specs` including `/specs/hyperscaler-gates.json`. A governing program lane this binding should be `planning_impact: true` and masterplan-bound; the omission is a front-matter gap.
- **tensions:**
  - ADR-0123 (hyperscaler-maturity-claim-gate) + ADR-0135 (aspirational-enforcement) — 0133 declares a BLOCKER lane "exists"; 0135 was authored precisely to stop ADRs claiming active enforcement before the crate/workflow/branch-protection exist. 0133's "BLOCKER on dev" lane claim should be read through 0135's honesty gate (is the `oya-governance-industry-best-practice-conformance` lane actually wired?). Potential aspirational-enforcement violation by 0133 itself.
  - CI-stack churn (keystone §1.3) — 0133's pipeline taxonomy and "GitHub Actions Hardening" baseline predate the GH-Actions→Jenkins→Argo migration; baselines list is fine but the enforcement substrate moved.
  - Retired vocab — `oya-governance-*` is correct (post-rename), good; but `microservices/governance/` paths are on the retired root.
- **hyperscaler_challenge:** QUESTIONABLE (in form, not intent). No hyperscaler runs a single unified "6-axis continuous conformance program" ADR — they decompose into separate, independently-owned scorecards (Google's Readiness/SRE review, AWS Well-Architected reviews, security/compliance pipelines) rather than one mega-gate. The *baselines* are real and well-chosen; the *single-program-single-lane* packaging is more aspirational than how hyperscalers actually operate. Argues AMEND (keep the axes, soften the one-BLOCKER-lane framing toward per-axis lanes — which 0134/0135 already start doing).
- **ai_slop:** Moderate — the §Standards table (17 standards each mapped to a named industry source) and the axis baseline lists are impressively complete but read as an exhaustive citation dump; "optimization is key" user-quote framing is repeated; the agentic-overlay 8-principle list partly restates other ADRs. Citation breadth is a strength but some rows are decorative (no current artifact to audit).
- **refinement:** (1) Add `planning_impact: true` + `masterplan_ref`. (2) Re-point lane/IP/policy paths to `{oya,cloud}/<service>/`. (3) Reconcile the pipeline-stage baseline with the current Buck2+Argo CI canon (ADR-0392/0511/0513). (4) Verify the BLOCKER lane actually exists or downgrade its language per ADR-0135. (5) Consider splitting into per-axis lanes rather than one mega-gate.
- **consensus_needed:** yes — "Is the conformance program one continuous BLOCKER lane (as authored), or N independently-owned per-axis scorecards (how hyperscalers actually run)? And is it masterplan-bound (planning_impact) or advisory doctrine?"

---

### ADR-0134 — Portfolio Hyperscaler Pattern Remediation Backlog

- **decision_atom:** Record a portfolio-wide remediation backlog (LLM circuit-breakers, per-tenant rate limits, provider-degraded shed, golden signals, error-budget burn-rate, plus the Buck2/oya-ci P0+LATER pipeline items) as an explicitly *proposed, non-binding* set of acceptance criteria that become enforceable only in the PR that ships each validator + fixtures + branch-protection wiring.
- **current_status:** Proposed (2026-05-17), version 1.1.0. Deliberately not Accepted.
- **disposition:** KEEP (this is the honest, correct shape — a backlog that refuses to self-certify). Long-term it will SUPERSEDE-by-attrition as each item lands its own ADR/validator, but no archive now.
- **governing:** n/a now; individual items reference ADR-0514 (target architecture) as their eventual home, and ADR-0135 enforces their advisory-until-wired honesty.
- **truth_flag:** TRUE — and notably the *best-behaved* ADR in the chunk: it was explicitly rewritten (v1.1.0) to strip the earlier PR-#135 "accepted/enforced" wording that claimed validators existed when they did not. This is a model honest-status ADR.
- **in_masterplan:** PARTIAL — `related_specs` includes `/specs/masterplan.json` and `/specs/products/workflow-studio.json`, but no `planning_impact:` flag (appropriate, since it is a *proposed backlog*, not a binding decision). Correctly NOT fully masterplan-bound until items land.
- **tensions:**
  - ADR-0514 (target architecture) — many items defer to it ("Proposed in ADR-0514"); if 0514 has since absorbed/renumbered these, several rows are duplicative backlog. Cross-ref-but-not-yet-folded.
  - ADR-0135 — its companion enforcement; 0134 is the canonical *example* of what 0135 protects (advisory claims that must not read as active).
  - `related_specs: /specs/products/workflow-studio.json` — uses the retired `/specs/products/` path slice that ADR-0119/0131 say flattened into per-service specs; stale path.
  - "Foundry needs all-providers-degraded shed" — "Foundry" retired brand; the provider-degraded-shed item now belongs to intelligence (ADR-0335).
- **hyperscaler_challenge:** ALIGNED. Circuit-breakers with bounded retry budgets (≤3), per-tenant token buckets + 429/Retry-After, load-shed to bounded 503, golden signals, and multi-window burn-rate alerting are textbook Google SRE / AWS resilience patterns. A hyperscaler would absolutely make these decisions. The *honest-backlog framing* is itself best practice. Argues KEEP.
- **ai_slop:** Low — this ADR is unusually disciplined about not fabricating enforcement. Mild: the table mixes product-SLO items with deep Buck2/psm/buckify build-internals at very different altitudes in one backlog (the `-DCFG_TARGET_OS_darwin` psm linker detail is oddly specific for a "portfolio" ADR), which muddies its scope.
- **refinement:** (1) Re-point `/specs/products/workflow-studio.json` to the per-service spec path. (2) Replace "Foundry" with intelligence. (3) Reconcile lane (A) product-SLO vs lane (B) build/CI items — arguably they belong in two ADRs (one SRE-resilience backlog, one CI-pipeline backlog under ADR-0514). (4) As items land, mark each row's governing ADR to prevent the backlog drifting stale.
- **consensus_needed:** no (shape is correct; it is explicitly Proposed and self-aware).

---

### ADR-0135 — Aspirational Enforcement Gate

- **decision_atom:** Ship a fail-closed validator (`oya gate validate aspirational-enforcement`) that scans normative docs/specs/registry and blocks any binding "active/required/blocks-merge/shall" enforcement claim that names a repository enforcement surface (check crate, workflow, lane-registry row, branch-protection context) which does not actually exist.
- **current_status:** Accepted (2026-05-17), `enforcement_status: active`, `enforced_by: oya gate validate aspirational-enforcement`. The ADR ships its own validator+crate+workflow+branch-protection+fixtures as one slice.
- **disposition:** KEEP — well-formed, self-consistent, and arguably the keystone meta-gate that keeps the whole corpus honest.
- **governing:** n/a (governing meta-gate).
- **truth_flag:** TRUE — and the rare ADR that *demonstrates its own honesty bar*: it is Accepted only because the enforcement slice (validator crate, CLI entrypoint, fixtures, workflow, branch-protection row, lane catalog) all land together. It explicitly enumerates the named surfaces it treats as known and fails closed on unreadable corpus.
- **in_masterplan:** PARTIAL — strong, machine-checkable front-matter (`enforcement_status`, `enforced_by`, `related_specs` = branch-protection.yaml + lanes.yaml + ci-lanes.md) but no `planning_impact:` flag and no `/specs/masterplan.json` ref. For a gate this load-bearing, masterplan binding is warranted.
- **tensions:**
  - ADR-0133 + ADR-0134 — 0135 is the enforcement backstop for both: it exists *because* 0133-style "BLOCKER lane on dev" claims and the old PR-#135 backlog over-claimed. It explicitly names 0133/0134 as keeping programs advisory until validators land. This is the chunk's strongest internal-consistency thread (and a useful tension probe: does 0133's own BLOCKER-lane claim survive 0135's scan?).
  - References still use `oya-check-*` crate dirs under `crates/` and `.github/workflows/` — consistent with the GitHub-Actions-era surface; under the Argo/oya-ci migration (ADR-0511/0513) the "workflow `name:` under `.github/workflows/`" surface definition will need updating. Mild stale.
  - Branch-protection context naming `oya-governance-*` is correct (post-rename) — good hygiene.
- **hyperscaler_challenge:** ALIGNED (and notably mature). Google/AWS internal tooling does verify that declared required-checks actually exist and gate; "don't let a doc claim a control is enforced when the control isn't wired" is exactly the kind of provenance/attestation discipline (cf. SLSA, policy-as-code attestation) hyperscalers invest in. Argues KEEP — this is a genuinely good, hyperscaler-grade decision.
- **ai_slop:** None of note. Tight, scoped, honest; the §Scope-Boundary explicitly disclaims over-reach.
- **refinement:** (1) Add `planning_impact: true` + masterplan ref. (2) Update the "known surfaces" definition for the post-GitHub-Actions CI substrate (Argo Workflows / oya-ci) so the detector keeps working after the CI migration. (3) Confirm the detector's named-surface list re-derives the `oya-governance-*` (not retired `oya-foundry-*`) prefix.
- **consensus_needed:** no (clean, correct, settled).

---

### ADR-0136 — Foundry as a single µservice (with internal bounded contexts)

- **decision_atom:** A hosted-agent platform is one product/µservice with internal bounded contexts (not N independently-deployed micro-services), because the invocation hot path crosses all the BCs in one shared failure boundary — matching AWS Bedrock / Vertex AI / Azure AI Foundry / Anthropic Console / Palantir AIP shape.
- **current_status:** Accepted (2026-05-18). Front-matter shows `superseded_by:[]`, but ADR-0335 (Accepted, 2026-05-21) **amends** it and the GLOSSARY retires the "foundry" brand.
- **disposition:** AMEND (the 6→1 topology decision is TRUE and survives — ADR-0335 P-5 explicitly says "ADR-0136 + amendment remain active as historical context; the 6→1 consolidation precedent stands"). The vocabulary is dead: rebrand foundry→intelligence and rebase the path off `microservices/foundry/` onto `{oya,cloud}/intelligence/` per ADR-0131-amendment+0512+0335. Not ARCHIVE — the architectural precedent is load-bearing.
- **governing:** ADR-0335 (foundry retired → absorbed by intelligence; amends 0136) for vocabulary/ownership; ADR-0131-amendment + ADR-0512 for the path root. ADR-0137 is its companion (BC enumeration). ADR-0138 carries the strangler for old paths.
- **truth_flag:** PARTIAL — the *decision* (consolidation, single deployment perimeter, single SLO/audit surface, DDD BCs) is TRUE and well-argued; the *brand and paths* are STALE (`microservices/foundry/`, `axis-foundry-*`, `oya-foundry-*`, `HG-FOUNDRY` all retired-vocab). Note keystone §1.3 flags the `Accepted, superseded_by:[]` front-matter as a known stale-drift to watch — but it is correctly *amended-not-superseded*, so KEEP-the-decision/AMEND-the-vocab is right.
- **in_masterplan:** YES (structurally) — `planning_impact: true`, `related_specs` includes per-microservice-flat-layout + hyperscaler-gates + `/specs/microservices/foundry.json`. But the foundry.json spec path itself is retired-brand; masterplan binding needs re-pointing to the intelligence substrate.
- **tensions:**
  - ADR-0335 — retires the foundry brand and absorbs the platform into intelligence; 0136's entire vocabulary (foundry/axis-foundry/oya-foundry) is the single biggest retired-vocab concentration in this chunk.
  - ADR-0131-amendment + ADR-0512 — `microservices/foundry/` root is legacy; all 493-artifact path claims are on the dead root.
  - Internal: cites "ADR-0134: Connect-dissolution Strangler (analogous migration pattern)" in §References — but ADR-0134 in this corpus is the *Portfolio Hyperscaler Remediation Backlog*, NOT a connect-dissolution strangler. **Wrong cross-reference** (the connect/communications dissolution is ADR-0135-originally-0126 per ADR-0132, or ADR-0138). This is a concrete citation error.
  - References the retired `feedback_workflow_objectgraph_adapter_layer.md` memory as live cross-BC rule.
  - The "493 → 493 zero loss" and per-BC artifact counts (98+104+74+71+71+75) are fabricated-precision claims with no verification link beyond ADR-0138's deferred checks.
- **hyperscaler_challenge:** ALIGNED (on the topology). The core claim — hosted-agent platforms ship as ONE product surface with internal BCs, and splitting an operationally-inseparable invocation hot path into 6 network-hopped µservices adds latency without isolation — is exactly right and the Bedrock/Vertex/Foundry/Console/AIP precedents are accurately characterized. A hyperscaler would make this consolidation decision. (The retired *brand* is orthogonal to the topology truth.) Argues AMEND (keep decision, fix vocab/path).
- **ai_slop:** Moderate — heavy fabricated precision (exact artifact counts, "~5–15ms cumulative mTLS", "50ms-budget", "493 artefacts"); the four-alternative analysis (a/b/c/d) is thorough but somewhat performative; the same hyperscaler-shape argument is restated in §Context, §Alternatives, and §Consequences. The reasoning is sound, the numeric specificity is unverified decoration.
- **refinement:** (1) Rebrand foundry→intelligence throughout; rebase paths to `{oya,cloud}/intelligence/`. (2) Fix the ADR-0134 mis-citation (should point to the actual communications/six-path strangler). (3) Re-point `/specs/microservices/foundry.json`. (4) Record the ADR-0335 amendment as an explicit `amended_by: [ADR-0335]` edge (currently `superseded_by:[]`, masking the relationship — the keystone-flagged drift). (5) Inline-or-repoint the retired adapter-layer memory ref.
- **consensus_needed:** yes — "Does the foundry 6→1 consolidation *precedent* (single product perimeter for an operationally-inseparable hot path) survive verbatim into the `intelligence` substrate, or does intelligence's two-layer model (ADR-0255) re-partition it? I.e., is ADR-0136 frozen-historical or live-binding under the intelligence brand?"

---

### ADR-0137 — Foundry bounded contexts

- **decision_atom:** The consolidated foundry/intelligence agent platform contains exactly six closed bounded contexts (runtime, supervisor, eval, evidence, guardrails, providers), each with its own ubiquitous language, contract surface, state, and owner sub-axis, and cross-BC traffic is restricted to typed events + ontology reader-ports + per-BC RPC (no cross-BC kernel/adapter imports or shared state).
- **current_status:** Accepted (2026-05-18). `superseded_by:[]`; same amended-by-ADR-0335 situation as its companion 0136.
- **disposition:** AMEND (sound DDD BC partition + inter-BC dependency rules survive; vocabulary/path/brand are retired exactly as 0136). Companion to 0136; same treatment.
- **governing:** ADR-0335 (brand retirement → intelligence) + ADR-0131-amendment/0512 (path root). ADR-0136 is the parent decision; this ADR enumerates its BCs.
- **truth_flag:** PARTIAL — the BC partition, the inter-BC dependency *table* (event producers/consumers/channels), the forbidden-vs-permitted dependency rules, and the BC growth/merge governance are all TRUE and genuinely useful architecture; the brand/path/`oya-foundry-*` crate-name vocabulary is STALE.
- **in_masterplan:** YES (structurally) — `planning_impact` not explicitly set in front-matter shown, but `related_specs: /specs/microservices/foundry.json` and it is binding internal structure; the foundry.json path is retired-brand and must re-point.
- **tensions:**
  - ADR-0335 — same brand retirement; all six `axis-foundry-*` sub-axes + `oya-foundry-<bc>-*` crate names are retired vocab. Under intelligence, the BCs likely re-home (guardrails ↔ governance? per ADR-0335's "Governance stays separate"). Watch: does "guardrails" BC stay in intelligence or migrate to the separate governance domain?
  - ADR-0137 references the retired `feedback_workflow_objectgraph_adapter_layer.md` (front-matter even annotates it "(retired per ADR-0145)") as the live inter-BC rule basis — citing a self-declared-retired memory.
  - The headline NFRs (dispatch p99 ≤50ms, router p99 ≤5ms, inline-check p99 ≤20ms, etc.) are precise budgets with no benchmark/evidence link — same fabricated-precision pattern as 0136.
  - Several follow-up CI lanes (`oya-governance-foundry-bc-boundary`, `ubiquitous-language-scope`) are declared but "authored in a follow-up IP" — exactly the aspirational-enforcement pattern ADR-0135 guards against (declared-not-shipped). 0137 is careful to label them follow-up/deferred, which keeps it just inside the 0135 line.
- **hyperscaler_challenge:** ALIGNED. Internal BC partition with explicit context maps and forbidden cross-context coupling is standard DDD and matches how Bedrock/Vertex/Foundry expose internal product modules. The six-BC count mapping 1:1 to the prior proven split is reasonable. A hyperscaler would structure an agent platform this way internally. Argues AMEND (vocab only).
- **ai_slop:** Moderate — exhaustive per-BC crate-fan-out enumeration and the a/b/c/d alternatives (3 vs 6 vs 12 BCs) are thorough-but-performative; precise NFR budgets are unverified; restates 0136's hyperscaler-shape argument. The inter-BC event table is the genuinely valuable, non-slop core.
- **refinement:** (1) Rebrand foundry→intelligence; rebase `axis-foundry-*`/`oya-foundry-*`/paths. (2) Re-home the `guardrails` BC explicitly relative to ADR-0335's intelligence-vs-governance split. (3) Re-point `/specs/microservices/foundry.json`. (4) Inline-or-repoint the retired adapter-layer memory to ADR-0145. (5) Record `amended_by: [ADR-0335]`. (6) Either ship or clearly defer the bc-boundary lane to stay clean under ADR-0135.
- **consensus_needed:** yes — "Under the retired-foundry → intelligence absorption (ADR-0335) with Governance kept separate, do all six BCs stay inside `intelligence`, or does `guardrails` (safety/policy/Cedar-adapter) migrate to the separate governance domain? The BC list is declared 'closed, grows only by ADR amendment' — this re-homing is exactly such an amendment."

---

## Chunk notes for synthesis

**Cluster identity.** Two tight sub-clusters authored in one 2026-05-17/18 session:
1. **Structure + governance doctrine** (0131 layout, 0132 no-grouping, 0133 conformance program, 0134 remediation backlog, 0135 aspirational-enforcement gate). This is oyatie's *how-we-keep-the-repo-honest* spine.
2. **Foundry topology** (0136 one-µservice, 0137 six-BCs) — sound architecture, dead brand.

**Dominant pattern A — retired-vocabulary leakage on two axes.** Almost every ADR in the chunk carries one or both of:
- the retired `microservices/<ms>/` service root (superseded by `{oya,cloud}/<service>/` per ADR-0131's own 2026-06-02 amendment + ADR-0512), and
- the retired **"foundry"** brand (→ cloud-intelligence/governance per ADR-0335, founder-confirmed).
0136/0137 are the single densest concentration of foundry-brand vocabulary in the corpus. None of these are *wrong decisions* — they are TRUE decisions wearing dead names. Disposition is overwhelmingly **KEEP-the-atom / AMEND-the-vocab**, never ARCHIVE.

**Dominant pattern B — stale front-matter masking real amend edges (keystone §1.3/§5.6 confirmed in-chunk).** 0131 (`superseded_by:[]` despite ADR-0512 root rename), 0132 (`superseded_by:[]` despite ADR-0362 grandfather-clause supersession), 0136/0137 (`superseded_by:[]` despite ADR-0335 amendment). In every case the *superseding/amending* ADR carries the truth and the *amended* ADR's front-matter is stale. Synthesis should trust the governing ADR. Recommend a corpus-wide `amended_by:` back-edge sweep.

**Dominant pattern C — the honesty triad (0133 → 0134 → 0135) is the chunk's strongest signal.** 0135 (aspirational-enforcement gate) exists specifically to stop 0133-style "BLOCKER lane on dev" over-claims and the old PR-#135 backlog over-claims that 0134 v1.1.0 corrected. This is a genuinely mature, hyperscaler-grade self-policing loop and the cleanest material in the chunk. Useful audit probe: run 0135's own detector against 0133 — 0133's "implemented BLOCKER lane" claim is the most likely in-chunk aspirational-enforcement violation.

**Dominant pattern D — fabricated precision.** 0131 (three conflicting migration-IP counts; invented wall-times), 0136 (exact 493/98+104+74+71+71+75 artifact counts; "~5–15ms mTLS"), 0137 (precise p99 NFR budgets with no benchmark). The reasoning is sound; the numeric specificity is unverified decoration. Low-severity but worth a deslop pass.

**Concrete defects found (not just vocab):**
- **ADR-0136 mis-citation:** §References calls ADR-0134 a "Connect-dissolution Strangler" — but ADR-0134 is the Portfolio Hyperscaler Remediation Backlog. The connect/communications dissolution is ADR-0135-(orig-0126)/ADR-0138. Hard citation error.
- **ADR-0131 internal IP-count contradiction:** "~15" vs "30" vs "25" migration IPs in one document.
- **Retired-memory citations:** 0132/0136/0137 cite `feedback_workflow_objectgraph_adapter_layer.md` whose own filename is annotated "(retired per ADR-0145)" — live policy resting on a retired reference.

**Cross-chunk tensions to carry to synthesis:**
- **Foundry-cluster ownership (0136/0137 ↔ ADR-0335/0255):** the topology survives but re-homes into intelligence; the open question is whether the `guardrails` BC follows intelligence or splits to the separate governance domain. Load-bearing for the intelligence-substrate masterplan entry.
- **Masterplan binding gaps:** 0133/0134/0135 lack `planning_impact: true` despite being binding governance gates (keystone §4 notes only 8.8% ADR binding today). If masterplan-as-authority wins the open founder question, these three must be bound in; if generated-from-ADRs wins, they need clean `planning_impact`/`masterplan_ref` front-matter to be picked up. Flag under BOTH readings.
- **Root-rename hygiene (0131/0132/0133/0136/0137):** every structural ADR in this chunk still speaks `microservices/<ms>/`; the `{oya,cloud}/<service>/` migration (0131-amend/0512) is recorded as prose, not edges. A single coordinated path-rename + edge-sweep would clear the largest mechanical debt in the chunk.
- **0133 conformance-program shape vs hyperscaler reality:** the only chunk decision that is *questionable on the merits* (not just vocab) — one continuous mega-BLOCKER-lane vs N independently-owned per-axis scorecards. Needs a founder ruling on program shape + masterplan binding.
