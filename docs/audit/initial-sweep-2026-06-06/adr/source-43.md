# ADR Audit — source-43

- **side:** SOURCE (`~/Developer/source`, `jason931225/oyatie`)
- **chunk:** 43
- **range:** slice 295–301 of `ls -1 docs/decisions/ADR-*.md | sort`
- **ADRs reviewed:** ADR-0358, ADR-0359, ADR-0360, ADR-0361, ADR-0362, ADR-0363, ADR-0364 (7)

---

### ADR-0358 — Ideal 0→100 production roadmap (strangler-fig, Bazel rules_rust + oya overlay, define-production-100-first)

- **decision_atom:** Reach hyperscaler-grade production via a strangler-fig migration that specs the ideal target first, builds it alongside the live tree, and cuts over service-by-service behind stable contracts — defining the production-100 exit bar before claiming readiness; the masterplan `ideal_production_roadmap` section is the binding planning authority for the effort.
- **domain:** ci-cd-build (cross-cutting: docs-ssot-masterplan)
- **current_status:** Proposed (front-matter `superseded_by: [ADR-0392, ADR-0408]`).
- **disposition:** AMEND (partial supersession already recorded in front-matter; §2 Bazel build-graph/CI engine is dead vocab, §1/§3/§4 survive). Not a full ARCHIVE because the amendment_note explicitly keeps §1 strangler-fig + §3 define-production-100-first + §4 masterplan-authority in force.
- **proposed_resolution:** RATIFY (with §2 carved out) — the surviving strangler-fig + define-100-first + masterplan-binding doctrine is sound and load-bearing; do not DROP. The §2 Bazel choice is already reversed to Buck2, so ratify the remainder, not the whole.
- **governing:** ADR-0392 (Buck2 canonical build graph) + ADR-0408 (Buck2-driven CI/CD) for §2 only; ADR-0364 reframes §4's masterplan as GENERATED (see tensions).
- **truth_flag:** PARTIAL (§1/§3/§4 TRUE; §2 STALE/WRONG — Bazel `rules_rust` is retired vocab per map §2 row "Bazel/rules_rust → Buck2").
- **in_masterplan:** PARTIAL (the roadmap doctrine binds in; the Bazel toolchain detail must not).
- **tensions:** Internal — §4 asserts "masterplan = the single planning authority … this ADR binds into it" (masterplan-as-authority reading), but ADR-0364 (Accepted, same author cluster, one day later semantically) declares the masterplan a GENERATED projection of the ADR log (ADRs-as-authority reading). This is the OPEN founder question (map §4); flag under both. Also the §2 Bazel/Buck2 reversal lives across three ADRs (0358/0392/0408), a churn chain.
- **hyperscaler_challenge:** ALIGNED on posture, QUESTIONABLE on the original tool pick. Google/AWS/Azure absolutely do strangler-fig + define-exit-bar-first + RBE/affected-targets — that is literally Google TAP/Bazel and Amazon Apollo. But Google itself uses Blaze/Bazel while Meta uses Buck2; the §2 Bazel→Buck2 reversal is a defensible internal call, not a hyperscaler-misalignment. Implication: amend (carve §2), keep the posture.
- **ai_slop:** No. Honest amendment_note, preserves superseded record, cites real research (TAP, Apollo, rings, blue-green).
- **refinement:** Strip §2 to a pointer at ADR-0392/0408; restate §4 to match ADR-0364's generated-masterplan model rather than "masterplan is the planning authority this binds into."
- **consensus_needed:** "Is the masterplan an authored planning authority that ADRs bind INTO (§4 here), or a GENERATED projection OF the ADR log (ADR-0364)? Both are Accepted/in-force and they point opposite directions."

---

### ADR-0359 — Jenkins completely replaces GitHub Actions as the CI orchestrator

- **decision_atom:** Make a self-hostable CI orchestrator the sole CI surface and fully remove GitHub Actions (whose budget SPOF blocked an entire merge-gate), moving PR gating to externally-reported required status checks — with the destination CI engine now Argo Workflows, not Jenkins.
- **domain:** ci-cd-build
- **current_status:** Superseded (front-matter `superseded_by: [ADR-0511]`).
- **disposition:** ARCHIVE. Cleanly superseded; the durable principle (remove GH-Actions SPOF, self-host CI, gate on external status contexts) survives in the governing chain, but Jenkins-as-orchestrator is retired vocab.
- **proposed_resolution:** NA (status is Superseded, not Proposed). Had it been re-derived today it would be DROP-as-written / KEEP-the-principle.
- **governing:** ADR-0511 (Argo Workflows = destination CI orchestration; Jenkins transitory bootstrap only). Downstream: ADR-0513 (oya-ci Prow-shaped), ADR-0514 (target architecture).
- **truth_flag:** STALE (the "Jenkins is the sole CI orchestrator" decision is dead; the underlying "kill GH-Actions SPOF + self-host" motivation is TRUE and carried forward).
- **in_masterplan:** NO as-written (Jenkins-as-destination is retired-vocab per map §2 row "Jenkins → Argo Workflows"); the self-host/no-GHA principle is YES via ADR-0511.
- **tensions:** Part of the long CI/CD churn chain (map §1.3): 0349→0359→0361→0408→0511→0513→0514. The PR #180 Actions-budget-block is the real, cited trigger. No live tension once read through ADR-0511.
- **hyperscaler_challenge:** ALIGNED in spirit, MISALIGNED on Jenkins specifically. AWS/Google/Azure all run bespoke self-hosted CI and gate on arbitrary status contexts (exactly this), and none would tolerate a third-party budget SPOF on the merge gate. But none would land on Jenkins as the modern destination — they'd pick a k8s-native engine (which is why ADR-0511 moves to Argo Workflows). Implication: archive (already superseded).
- **ai_slop:** No. Concrete operational evidence (PR #180, 37 jobs, "Actions budget" annotation), real Jenkins/branch-protection research.
- **refinement:** None needed — it is correctly Superseded. Ensure the on-disk `amends: ADR-0349` and `superseded_by: ADR-0511` cross-refs stay intact through any re-foundation (ADR-0364 D7).
- **consensus_needed:** None (resolved by chain).

---

### ADR-0360 — CI/CD pipeline optimization program (affected-target, gate-only overlay, warm cache, sharding, signed agent image, speculative merge queue, content-addressed gate caching)

- **decision_atom:** Adopt a seven-part CI optimization program (O1 affected-target precision, O2 governance-only `oya` overlay, O3 warm shared cache, O4 distributed test sharding, O5 pinned/signed agent image, O6 speculative always-green merge queue, O7 content-addressed gate caching), each with a hard correctness rule so optimization never weakens the governance gates.
- **domain:** ci-cd-build (cross-cutting: security-supplychain via O5 signed image)
- **current_status:** Proposed (no `superseded_by`; `amends: ADR-0346`).
- **disposition:** AMEND. The optimization *primitives* are engine-agnostic and survive the Jenkins→Argo-Workflows/Buck2 churn, but several specifics are stale: O3's `sccache` cache and O1's bespoke `cargo metadata` affected-selection are largely subsumed by Buck2's native RBE + affected-target query (ADR-0392), and O6's speculative queue is reassigned to cloud-ci/Tide per ADR-0513 (ADR-0363 §3). So: sound program, stale engine bindings.
- **proposed_resolution:** RATIFY (the seven correctness-ruled optimizations as principles) — they are the hyperscaler-CI bar and remain wanted. Re-bind O1/O3 to Buck2 RBE and O6 to Tide rather than DROP. The correctness rules (full-mirror backstop, un-cacheable-gate-always-runs) are the load-bearing, durable part.
- **governing:** No supersession; partial absorption by ADR-0392 (Buck2 RBE/affected-targets supersede O1/O3 mechanics) and ADR-0513 (Tide owns O6 merge queue). `amends: ADR-0346` (preserves `--ci-required` full mirror) still holds.
- **truth_flag:** PARTIAL (correctness rules + O2/O4/O5/O7 TRUE; O1/O3 mechanics and O6 ownership STALE vs Buck2/Tide).
- **in_masterplan:** PARTIAL (the optimization-program intent + correctness invariants bind in; the sccache/cargo-metadata implementation detail should not, given Buck2).
- **tensions:** O6 speculative merge queue here vs ADR-0363 §3 + ADR-0513 (merge queue owned by cloud-ci/Tide, NOT the VCS substrate, and ADR-0111 projected-state folded into Tide) — consistent in intent, but ownership moved; this ADR's O6 reads as if `oya`/CI owns it. O2 "oya as governance-only overlay" aligns with ADR-0363 §4 (oya retired from CI authority). O1/O3 vs Buck2 (ADR-0392) overlap.
- **hyperscaler_challenge:** ALIGNED. This is a near-verbatim transcription of Google's CI playbook (TAP affected-targets, RBE warm cache, nextest sharding, Not-Rocket-Science speculative queue, content-addressed action caching = Bazel's own model). Google/AWS/Azure would make every one of these calls. The only quibble is reimplementing affected-target selection by hand when Buck2/Bazel give it natively — which ADR-0392 fixes. Implication: amend to ride the build tool, keep the program.
- **ai_slop:** No. Each O has a stated correctness rule and an evidence-blocked claim ("blocked_until_required_evidence_is_green"); cites direct observation (1342-file diff, 10+ min full mirror).
- **refinement:** Re-express O1/O3 as "use Buck2 native affected-targets + RBE remote cache" and O6 as "Tide speculative queue (ADR-0513)"; keep O2/O4/O5/O7 and ALL seven correctness rules verbatim.
- **consensus_needed:** "Now that Buck2 (ADR-0392) provides native affected-targets + RBE remote cache and Tide (ADR-0513) owns the merge queue, is ADR-0360's bespoke O1/O3/O6 machinery still needed, or does it collapse into Buck2+Tide config?"

---

### ADR-0361 — Execute the Jenkins-native CI/CD revamp (license-vetted supply-chain stack, retire GHA, drop parity gate)

- **decision_atom:** Execute the CI cutover in-code with a license-vetted, OSI-strict, self-hostable supply-chain stack (cargo-deny, Opengrep/Semgrep-CE, gitleaks, cargo-cyclonedx+Syft, Trivy+osv-scanner, cosign+in-toto/SLSA, Kyverno verifyImages, Argo CD/Rollouts) in a mandatory shift-left order, delete the 36 GitHub Actions workflows, drop the parity gate, and repoint the closure gate at the new pipeline.
- **domain:** ci-cd-build (cross-cutting: security-supplychain — the SBOM/vuln/sign/provenance/admission stack is the real durable core)
- **current_status:** Proposed (no `superseded_by`; `amends: ADR-0359`).
- **disposition:** AMEND. The supply-chain tool stack + shift-left order + license-vetting are TRUE and survive every CI-engine change; only the "Jenkins-native, per-microservice `Jenkinsfile`" execution frame is stale (Jenkins is transitory bootstrap per ADR-0511; lanes move to Argo Workflows / oya-ci per ADR-0513). So the *what* survives, the *Jenkins how* is retired.
- **proposed_resolution:** RATIFY (the license-vetted supply-chain stack + shift-left order + forbidden-tool list) — this is the concrete, correct, OSI-strict security pipeline and must not be DROPped. Re-bind execution from `Jenkinsfile`/shared-library to Argo Workflows / oya-ci. The forbidden list (Snyk, Drone, Renovate CE/EE, Semgrep registry rules) is durable license policy.
- **governing:** Execution frame superseded-in-effect by ADR-0511 (Argo Workflows) + ADR-0513 (oya-ci Prow-shaped). `amends: ADR-0359` (itself Superseded by ADR-0511). Inherits the Redis→Valkey OSI-strict precedent (ADR-0336).
- **truth_flag:** PARTIAL (supply-chain stack + license policy TRUE; Jenkinsfile/shared-library/parity-drop execution mechanics STALE).
- **in_masterplan:** PARTIAL (the supply-chain pipeline + license vetting bind in; the Jenkins-specific execution should not).
- **tensions:** Same CI churn chain (0359→0361→0408→0511→0513). "Drop the parity gate" + "delete 36 `.github/workflows`" are consistent with ADR-0359/0511 (kill GHA). The closure-gate repoint (`validate_pr_review_pipeline`) hard-codes Jenkins — that detail is now stale vs oya-ci. ADR-0363 §4 further retires `oya verify`/`oya gate` as protected-branch producers, which this ADR still assumes as the root gate.
- **hyperscaler_challenge:** ALIGNED on the supply-chain stack (this IS the SLSA/in-toto/cosign/admission posture AWS/Google/Azure ship), MISALIGNED on Jenkins as the executor. No hyperscaler would build a greenfield 2026 supply-chain pipeline on Jenkins per-service `Jenkinsfile`s; they'd use a k8s-native engine — exactly the ADR-0511 correction. License-strictness (no Snyk/Drone/Semgrep-registry) is more conservative than typical hyperscaler practice but defensible for an OSI-strict self-hostable product. Implication: amend (swap executor), keep stack.
- **ai_slop:** No. Specific, license-classified tool list with per-tool license citations and an explicit forbidden/allowed split — high-signal, not slop.
- **refinement:** Lift the supply-chain stack + shift-left order + forbidden list out of the Jenkins frame into an engine-agnostic "supply-chain pipeline contract"; point execution at ADR-0511/0513.
- **consensus_needed:** "Carry ADR-0361's license-vetted supply-chain stack + shift-left order forward verbatim, but re-host execution on Argo Workflows/oya-ci instead of Jenkinsfiles — confirm the stack is the durable decision and Jenkins was only the (now-retired) carrier?"

---

### ADR-0362 — Full grouping retirement (flat-only catalog)

- **decision_atom:** Retire ALL product grouping (suite/family/bundle/platform/vertical) as an architecture artifact — flat single-concern microservices are the only architecture unit; existing grouping wrappers are demoted to deprecated tombstones; grouping survives only as a future ADR-gated presentation/catalog tag, never a code/deploy/SLO/binding unit; and the previously-aspirational `no-grouping` gate is implemented for real.
- **domain:** tenancy (cross-cutting: governance-process — turns an aspirational lane into a real enforced gate)
- **current_status:** Accepted (deciders include founder; `amends: ADR-0132`; `superseded_by: []`).
- **disposition:** KEEP. Current, correct, founder-decided, with real enforcement and a verification command. A retirement-vocabulary keystone (map §1.2, §2 "product grouping → flat-only catalog").
- **proposed_resolution:** NA (Accepted).
- **governing:** None (it is the governing ADR; amends ADR-0132, related to the ADR-0238/0237 dissolution-strangler that owns the actual decomposition).
- **truth_flag:** TRUE.
- **in_masterplan:** YES (flat-only catalog + no-grouping gate is canonical posture; also referenced by ADR-0363 §1 and named in the keystone retired-vocab table).
- **tensions:** Minimal. Mild forward-dependency: §3 "no paper µservices" defers real decomposition to ADR-0238 via the ADR-0237 strangler and an "not-yet-authored `tenant-rbac-governance-council` dissolution ADR" — a dangling future-ADR pointer to track, not a contradiction. Consistent with ADR-0131 (flat-layout authority).
- **hyperscaler_challenge:** ALIGNED. "Independent deploy/scale/SLO/blast-radius per service; no wrapper that re-grows a monolith by gravity" is exactly the AWS two-pizza / Google service-per-concern doctrine; the "grouping is presentation-only, never a binding artifact" rule is the correct hyperscaler stance. Google/AWS/Azure would make this call. Implication: keep.
- **ai_slop:** No. Founder-directive-grounded, rejects "manufacture paper µservices" as an honest-claims violation (anti-slop reasoning baked in), real `oya gate` verification.
- **refinement:** Track the "not yet authored `tenant-rbac-governance-council` dissolution ADR" so it doesn't become a permanent dangling reference; otherwise none.
- **consensus_needed:** None.

---

### ADR-0363 — Retire bespoke agentic-VCS; Foundry→Intelligence; `oya` is a governance-gate engine

- **decision_atom:** Retire the bespoke agentic-VCS layer (`oya vcs` CLI, `oya git` wrapper, dormant changeset-state-machine/merge-queue/webhook-receiver crates) in favour of plain git + self-hosted Forgejo PRs + Prow-shaped cloud-ci/oya-ci required contexts; absorb the Foundry AI-agent platform into Intelligence while keeping Governance as its own service (layering: a validator cannot live inside what it validates).
- **domain:** forge-vcs (cross-cutting: intelligence-ai / agentic-platform — the Foundry→Intelligence absorption)
- **current_status:** Accepted (`supersedes: [ADR-0110, ADR-0112, ADR-0113]`; `amends: ADR-0116`; `amended_by: [ADR-0510, ADR-0513]`).
- **disposition:** KEEP (amended). It is a live governing keystone (map §1.2/§1.3/§5) — it supersedes three ADRs and is the Foundry-dissolution + oya-CLI-deauthorization anchor. The doc has already absorbed its own amendments inline (the "Amended by ADR-0513" block flips the no-Tide-queue / Jenkins-as-destination readings). So KEEP the body, with the amendment understood as governing the merge-queue + CI-authority sub-points.
- **proposed_resolution:** NA (Accepted).
- **governing:** None over the core retirement (it governs). For its now-stale sub-readings: ADR-0510 (Forgejo reframed transitory; bespoke hyperscaler monorepo-VCS = declared destination) and ADR-0513 (merge queue → Tide; oya-ci Prow-shaped = current CI authority; Jenkins bridge-only). It supersedes ADR-0110/0112/0113.
- **truth_flag:** TRUE for the retirement core (kill bespoke VCS, Foundry→Intelligence, Governance-stays-separate-by-layering, oya = gate engine not CI authority). PARTIAL on substrate finality — "Forgejo is the self-hosted substrate" is now *transitory* per ADR-0510, and the founder's live migration directive is GitHub `jason931225/oyatie` (map §5 three-way fault-line).
- **in_masterplan:** YES (Forge/SCM canonical posture row + Intelligence row + the "oya = governance-gate engine" deauthorization all derive from here, read through ADR-0510/0513).
- **tensions:** THE forge fault-line (map §5 #4): GitHub (founder migration directive) vs Forgejo (this ADR's self-hosted substrate, now transitory per ADR-0510) vs bespoke hyperscaler monorepo-VCS (ADR-0510 declared destination) — a genuine THREE-way unresolved tension. Also: this ADR's original "Forgejo native required-status gating via Jenkins-posted statuses" is superseded by ADR-0513's oya-ci required contexts. Internal layering claim (Governance ⊄ Intelligence) is sound and uncontested.
- **hyperscaler_challenge:** MIXED. "Don't reinvent the wheel; use git as-is; retire 13 dormant never-deployed crates" — QUESTIONABLE-resolved-correctly: hyperscalers (Google Piper/Critique, Meta Sapling/Mononoke) famously DO build bespoke monorepo VCS at scale, so "never build VCS" isn't a hyperscaler axiom — but they build it only at extreme scale, and retiring 0–1-dependent dormant crates pre-scale is the right call (which is exactly why ADR-0510 reframes Forgejo as transitory and names a future bespoke VCS as the destination). Foundry→Intelligence absorption + Governance-as-independent-validator (layering) is ALIGNED with how hyperscalers separate CI/policy from the platform it governs. Implication: keep, but the "own the VCS eventually" question (ADR-0510) is the real hyperscaler-grade tension to surface.
- **ai_slop:** No. Evidence-dense (20 crates, only 2 wired, ~13 with 0–1 dependents), founder deep-interview quotes, explicit rejected-alternatives, staged migration PRs with task IDs, and an honest inline amendment recording its own superseded readings.
- **refinement:** The body is already self-amended; ensure any masterplan projection reads the Forgejo claim as *transitory* (ADR-0510) and merge-queue/CI-authority as Tide/oya-ci (ADR-0513), not the original Jenkins/Forgejo readings.
- **consensus_needed:** "THREE-way forge decision: founder's migration directive is GitHub `jason931225/oyatie`; ADR-0363 canon is self-hosted Forgejo; ADR-0510 reframes Forgejo as transitory and names a bespoke hyperscaler monorepo-VCS as the destination. Which is the masterplan's single forge truth — GitHub-now, Forgejo-interim, or own-the-VCS-eventually?"

---

### ADR-0364 — Generative ADR template; masterplan generated from the ADR log

- **decision_atom:** Make the masterplan a GENERATED projection of the ADR decision log — `oya gen masterplan` reads accepted `planning_impact: true` ADRs (topo-sorted by depends_on/supersedes, grouped by milestone, emitting deliverables) with status DERIVED from gate output (never authored), contracts ratified by-reference, docs reorganized into Diátaxis, and the ~300-ADR log re-founded into a clean immutable ADR-0000+ series — so ADRs are the single authored SSOT and every other planning source is build output guarded by a drift gate.
- **domain:** docs-ssot-masterplan (cross-cutting: governance-process — the ADR-completeness/drift/traceability gates)
- **current_status:** Accepted (`supersedes: []`, `superseded_by: []`; models its own generative front-matter).
- **disposition:** KEEP. This is the keystone authority-model ADR and the literal mandate behind THIS AUDIT ("masterplan GENERATED from the ADR log; ADRs = immutable SSOT" — the task's own framing). Current, Accepted, self-exemplifying.
- **proposed_resolution:** NA (Accepted).
- **governing:** None (it governs the planning-SSOT model). `depends_on: ADR-0363`. Its D7 re-foundation (distill ~300 → ~44 clean ADRs, ADR-0000+ with `consolidates:` provenance) is the mechanism this entire audit feeds.
- **truth_flag:** TRUE (and it is the operative truth-model for the whole audit). One caveat: it is Accepted *doctrine* whose deliverables (`oya gen masterplan`, the 4 gates, the re-foundation) are gate-derived/not-yet-green — TRUE-as-decision, PENDING-as-implementation.
- **in_masterplan:** YES — definitionally; it defines how the masterplan is built. The masterplan's own front-matter (`shape: compatibility_projection`, `canonical_authority: /specs/masterplan.json`) must be read through this ADR.
- **tensions:** THE open founder question (map §4), and ADR-0364 is one of the two poles. Direct conflict surface:
  - **`planning-ssot-consolidation.md` + ADR-0364 = ADRs-generate-masterplan** (this ADR: "`specs/masterplan.json` becomes build output, never hand-authored").
  - **`planning-ssot-drift-prevention.md` = masterplan.json IS the authority; ADRs bind INTO it** (opposite direction), and ADR-0358 §4 ("masterplan = the single planning authority this ADR binds into") sits on that opposite pole.
  - The masterplan front-matter itself says `canonical_authority: /specs/masterplan.json` (authority), which reads against ADR-0364's "masterplan is generated build output." UNRESOLVED — must flag under both readings (map §4 explicit instruction).
- **hyperscaler_challenge:** ALIGNED. "Generate the roadmap from immutable decision records; one canonical source per concern; status derived from CI not authored; contracts ratified by-reference not embedded" is precisely the Kubernetes KEP model (the one validated precedent it cites) + Google's canonical-doc/"docs as build artifact" rule + AWS "mechanisms not intentions" drift gate. Google/AWS/Azure would make this call. The only risk hyperscalers would flag: a 300→44 re-foundation (D7) is a high-blast-radius one-time migration that must not lose the audit trail (the ADR itself rejects in-place renumber for exactly this reason — well-reasoned). Implication: keep.
- **ai_slop:** No. The richest, most self-aware ADR in the chunk — it models its own front-matter schema, cites the KEP precedent specifically as "the one validated instance," and bakes the immutability/drift/anti-spec-saturation rules into the design.
- **consensus_needed:** "Is ADR-0364 the settled answer (masterplan = GENERATED from ADRs; ADRs = immutable authored SSOT) — overriding `planning-ssot-drift-prevention.md`, ADR-0358 §4, and the masterplan front-matter's `canonical_authority: /specs/masterplan.json` (which all read masterplan-as-authority)? The audit's own mandate assumes ADR-0364 wins; that assumption needs an explicit founder ratification because three live artifacts still encode the opposite direction."

---

## Chunk notes

**Shape of the chunk.** Five of seven ADRs are CI/CD-build (0358, 0359, 0360, 0361 — the long CI churn chain — plus the supply-chain stack), one is tenancy/governance (0362 grouping retirement), one is forge-vcs/intelligence (0363), and the last is the docs-ssot-masterplan authority model (0364). The CI cluster is the dominant theme and the most stale.

**Net CI/CD truth (read through the keystone map §1.3).** The four CI ADRs in this chunk are all way-stations on the chain 0349→**0359**(here, Superseded)→**0361**(here, executes 0359)→0408→0511→0513→0514, with 0358/0360 feeding the build-graph + optimization design. Current canonical truth: **Buck2** (build/RBE/affected-targets) + **Argo Workflows** (k8s-native CI orchestration) + **ArgoCD/Argo-Rollouts** (CD) + bespoke-Rust **oya-ci** (Prow-shaped) as target, with **Jenkins = transitory bootstrap only** and **GitHub Actions retired** (the PR #180 budget-SPOF was the trigger). Disposition pattern for the cluster: **ADR-0359 ARCHIVE** (cleanly Superseded by 0511); **0358/0360/0361 AMEND-not-archive** — their durable cores (strangler-fig + define-100-first + masterplan-binding; the 7 correctness-ruled optimizations; the license-vetted supply-chain stack + shift-left order + forbidden-tool list) survive every engine swap, but their Bazel/Jenkins/sccache *carriers* are retired vocab and must be re-bound to Buck2/Argo-Workflows/oya-ci/Tide. The recurring lesson: **separate the decision (kill GHA SPOF, self-host, affected-targets, supply-chain hardening, license-strict) from the carrier (Jenkins, Bazel, sccache) — the decisions are TRUE, the carriers churned.**

**Two clean KEEPs.** ADR-0362 (flat-only grouping retirement) and ADR-0364 (generated-masterplan) are both Accepted, founder-decided, current, and load-bearing keystones — no churn, no stale carrier. ADR-0363 is a KEEP-with-inline-amendment (its Forgejo-substrate + Jenkins-status + no-merge-queue readings were already amended by ADR-0510/0513 inside the doc body).

**The two big surfaced tensions (both already flagged in map §4/§5, do not resolve):**
1. **Masterplan authored-vs-generated (ADR-0364 vs ADR-0358 §4 vs the two `planning-ssot-*.md` ideas vs the masterplan front-matter `canonical_authority`).** ADR-0364 (Accepted) says masterplan is GENERATED build output from immutable ADRs; ADR-0358 §4 (in-force), `planning-ssot-drift-prevention.md`, and `masterplan.json`'s own front-matter say masterplan IS the authority that ADRs bind INTO. Opposite directions, all live. THE OPEN FOUNDER QUESTION — and critically, **this audit's own mandate presupposes the ADR-0364 / generated-from-ADRs answer**, so a founder ratification of ADR-0364 over the masterplan-as-authority artifacts is the single most consequential consensus item in the chunk.
2. **Forge three-way (ADR-0363 → ADR-0510):** GitHub (founder migration directive `jason931225/oyatie`) vs self-hosted Forgejo (ADR-0363, now transitory per ADR-0510) vs bespoke hyperscaler monorepo-VCS (ADR-0510 declared destination). Surface only.

**Re-foundation hook.** ADR-0364 D7 is the mechanism that consumes this entire audit (distill ~300 ADRs → clean ADR-0000+ series). For this chunk, that means: re-author 0362/0364 near-verbatim; re-author 0363 with its amendments folded in; collapse 0358/0360/0361 into the surviving Buck2/Argo/supply-chain decisions with `consolidates:` provenance; archive 0359 frozen (it is already Superseded). No ADR in this chunk is GARBAGE or pure AI-slop — all carry real evidence, citations, and honest supersession/amendment records.

**Vocabulary hygiene checks (clean — no leakage in this chunk's authored decisions, only in superseded/amended carriers):** "Bazel/rules_rust" (0358 §2 — retired→Buck2, already noted), "Jenkins-as-destination" (0359/0361 — retired→Argo Workflows, already superseded/amended), "Foundry" (0363 — correctly treated as RETIRED and absorbed into Intelligence/Governance, used only to describe the thing being retired). No `tier-system`, `M0-M3/MVP`, `Redis`, `Kafka`, `cell-as-service`, or `Backstage` leakage. ADR-0363's `oya-foundry-*` mentions are explicitly describing the rename-away (history), not new work.
