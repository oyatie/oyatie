---
title: "PHASE-0 — THE FALSE-GREEN FIREWALL PLAN (executable, firewall-first)"
charter: D-SEQUENCE (firewall-first order) · D-CICD (oya-ci/oya-cd bespoke-Rust, adopts Prow+Tekton+Argo patterns) · D-DOCTRINE (maintainable-by-enforcement, total-accounting, robust-not-false)
authority: docs/audit/initial-sweep-2026-06-06/synthesis/decision-record-oyatie-canon.md  D-CICD(L168-169) · D-PURESPLIT(L171-172) · D-DOCTRINE(L177-184) · D-SEQUENCE(L195-201)
date: 2026-06-06
mode: PLAN. Assembled READ-ONLY from the 4 lane artifacts in _phase0/ (10-*.md) + live re-verification of load-bearing facts. No source file was edited by this plan.
scope_repos:
  - /Users/jasonlee/Developer/source                                   (the live monorepo — STEP-0 + Phase-0 mutation target)
  - /Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06 (the decision/plan authority corpus)
legend:
  ENFORCED-NOW = exists on disk + runs in `cargo test` today (real RED/GREEN today)
  ASPIRATIONAL-TOOLING = the BUILD/DECIDE/ACT work this plan schedules (not yet enforcing)
provenance_lanes:
  - _phase0/10-ci-adr-spec.md       (the ONE canonical CI/CD ADR authoring spec + producer contract)
  - _phase0/10-producer-plan.md     (the live oya-ci-required producer: exists-vs-build)
  - _phase0/10-gates-registry.md    (accounting-registry schema + 4 keystone gate specs w/ RED/GREEN)
  - _phase0/10-lane-spine.md        (the execution spine: lanes, order, ralph-loop, checkpoints)
---

# PHASE-0 — THE FALSE-GREEN FIREWALL PLAN

> **Reading order:** §1 goal + the verified façade + firewall-first thesis → §2 STEP-0 clean base →
> §3 the ONE canonical CI/CD ADR (+ producer contract + pattern→Rust table) → §4 the producer build
> (exists-vs-build + go-live RED/GREEN) → §5 accounting-registry + 4 keystone gates (each RED+GREEN +
> buck2 wiring) → §6 the execution spine (lanes, order, bootstrap discipline, sign-off + door:one-way
> + credential points, the rollback trigger, the ralph loop) → §7 EXIT CRITERIA.
> Every step is tagged **ENFORCED-NOW** (real today) or **ASPIRATIONAL-TOOLING** (the BUILD work).
> Every load-bearing claim cites a real path:line, re-verified live 2026-06-06.

---

## §1 — GOAL + THE VERIFIED PROBLEM + THE FIREWALL-FIRST THESIS

### 1.1 Goal
Make oyatie's merge-gate enforcement **REAL** — one trusted producer that posts `oya-ci-required` on the
candidate SHA and genuinely **BLOCKS** a known-bad PR, plus 4 keystone gates over one generated
accounting-registry, each proven by a RED fixture it fails + a GREEN fixture it passes + proof it runs in
CI and blocks. Only when the firewall is real do the Phase-1 canon amendments proceed — so they are
gate-verified and **cannot re-drift** (D-SEQUENCE L196-201, D-DOCTRINE L178).

### 1.2 The verified problem — enforcement is a FAÇADE (0 gates block a merge) — ENFORCED-NOW evidence
The audit verdict is not a doc claim; it is read from the live source tree (re-verified 2026-06-06):

- **The required context names a producer that does not run.** `oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:471` declares `pub const GATE_CONTEXT: &str = "oya-ci-required";` — but a repo-wide grep for the controller across `.github/`, `Jenkinsfile`, `infra/ci/` returns **only `.github/branch-protection.yaml`** (the context *name*, never an invocation). **No live producer posts `oya-ci-required` on a candidate SHA** (lane-spine §0; producer-plan §2.2).
- **Both protection files self-disclaim.** `infra/branch-protection/dev.json:2` and `.github/branch-protection.yaml:2-5` both declare themselves *"not Phase-0 exit authority until a trusted cloud-ci/oya-ci producer is live and applied."* The sole listed required context is `[oya-ci-required]` (`dev.json:9-11`, `branch-protection.yaml:55-56`).
- **Live GitHub actually requires a DIFFERENT, broken set.** `specs/phase0-ci-enforcement-baseline.json:73-84` records the live `dev/protection` set = `[cargo-fmt, cargo-check, cargo-clippy, cargo-nextest, oya-pr-review]` — and `oya-pr-review`'s producer returns **HTTP 501** (dev.json:8). So `oya-ci-required` is required by **nobody live**; verdict (`baseline.json:214`) = `P0.0_RED_blocked_until_cloud_ci_required_context_is_live`.
- **Net:** **0 gates block a merge today.** This IS the mechanism of the drift D-DOCTRINE (L178) names: drift is a faulty-process/enforcement problem, and the enforcement that makes recurrence impossible is the only real fix.
- **Forge-of-record mismatch (the load-bearing producer gap).** Live branch protection is on **GitHub** `jason931225/oyatie` (`baseline.json:82` `gh api repos/jason931225/oyatie/...`; `scripts/branch-protection-apply.sh` default), but the only wired controller poster targets **Forgejo** in-cluster (producer-plan §2.2). The enforcement is both a façade AND pointed at the wrong producer family.

### 1.3 The firewall-first thesis (D-SEQUENCE L196, verbatim canon)
*"You cannot fix the canon on fake enforcement; make enforcement REAL first, then fix the canon THROUGH it."*
The dependency spine (D-SEQUENCE L201): **producer → accounting-registry → gates → amendments → reorg.**
Phase-0 builds the producer + the 4 gates; only then does Phase-1 (the 6→1 CI/CD consolidation, dup-0377
renumber, foundry-completion, etc.) run — each amendment now gate-verified.

### 1.4 The robustness bar (founder; applied to every step in this plan)
No thin/flaky/false enforcement. **Every gate is proven by (a) a RED fixture — a known-bad input it MUST
fail — + (b) a GREEN fixture — a known-good it passes — + (c) proof it runs in CI and BLOCKS** (D-DOCTRINE
L181; D-SEQUENCE L198). Until live receipts exist on real SHAs, every authority surface uses **gap-packet
language only** (`baseline.json:37-47` forbids "Phase 0 complete" / "P0.0 green" / "mechanically enforced" /
"production-ready" until proven).

---

## §2 — STEP-0: COMMIT THE WIP TO A CLEAN BASE  ◄ door:one-way + founder sign-off
*Authority: D-SEQUENCE L197 ("Step-0: commit the WIP to a clean base — precondition to ALL mutation"). Lane: 10-lane-spine §A.*
*Tag: ASPIRATIONAL-TOOLING (the git acts are scheduled, gated on founder sign-off; nothing committed by this plan).*

**Live surface (ENFORCED-NOW facts, re-verified):** source on branch **`feat/oya-ci-tide`**; remotes
`origin=http://forgejo.local/oya-admin/oyatie.git` + `github-mirror=https://www.github.com/jason931225/oyatie`;
**79 dirty** entries; WIP plan `.omc/plans/monorepo-consolidation-migration.md` is **untracked**.

### 2.1 Triage the 79 by class (separate verifier pass — NO blind `git add -A`)
| Class | Live examples | Disposition |
|---|---|---|
| Decision/spec canon | `docs/adr-archive/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md (M), `specs/masterplan.json` (MM), `specs/master-plan-sequencing.json` (MM), `registry/quality/lanes.yaml` (MM) | **STAGE as base canon** — the corpus the gates will police; verify each diff is intended first. |
| WIP execution authority | `.omc/plans/monorepo-consolidation-migration.md` (??), `.omc/plans/open-questions.md` (??) | **COMMIT** — D-MERGE made this the execution authority; highest-value untracked file. |
| Phase-0 net-new seeds | `specs/cloud-*.json` (??), `scripts/tests/cloud_*_check.py` (??), `docs/ideas/buck2-native-ci-gate.md` (??), `docs/ideas/agent-execution-controller.md` (??) | **STAGE selectively** — commit the decision-grounded ones, leave runner junk. |
| Tooling/harness churn | `.claude/`, `.codex/`, `.gemini/`, `.omc/state/*.jsonl`, `.omc/sessions/*`, `evidence/multispectrum/*` | **GITIGNORE / leave** — never commit session jsonl (the `git ls-files` accounting discipline). |
| Renames/deletes | `specs/language-discipline-registry.json` (AD), `specs/repo-hygiene-automation.json` (AD), `.omc/state/*.jsonl` (D) | **Resolve intent** before committing (relocated vs dropped). |

### 2.2 Ordered git procedure (concrete)
1. **Stay on `feat/oya-ci-tide`** — do NOT cut a new base branch (it is the in-flight authority-migration branch the WIP loop rebases on `github-mirror/dev`). Phase-0 lanes branch OFF this as `phase0/<lane>` (one-lane-one-squash-PR).
2. **Triage the 79** per §2.1 in a separate verifier pass (founder rule: verify at each step; no phantom dispositions).
3. **Re-confirm signing is provisioned (G3).** `commit.gpgsign` / `user.signingkey` / `gpg.format` / `tag.gpgsign` must be non-empty (they were empty at WIP-authoring time; canon L201/decision-record:47 says now provisioned — re-confirm before committing).
4. **Commit the canon + WIP authority as ONE signed, squash-shaped commit**, linear history; body carries the 5-H2 PR template + DOC-CATALOG/CHANGELOG rows (note path is `docs/DOC-CATALOG.md`, not `docs/CATALOG.md`).
5. **Push `github-mirror` ONLY** — never `origin`/Forgejo (FORGE-EXPLICIT). `gh pr create --repo jason931225/oyatie --base dev`.
6. **door:one-way founder sign-off on the base commit** before any gate lane opens.

**Why door:one-way:** STEP-0 fixes what is "in the canon" at the instant the firewall is built. The gates born
in Phase-0 police *this base*; anything left dirty/untracked is invisible to total-accounting and re-seeds drift.

---

## §3 — THE ONE CANONICAL CI/CD ADR  ◄ door:one-way + founder sign-off
*Authority: D-CICD L169 as REFINED by founder 2026-06-06 — consolidate ADR-0349/0359/0361/0511/0513/0514 → ONE canonical CI/CD ADR (= ADR-0515); **ADR-0408 stays SEPARATE** (Buck2 build; new ADR `depends_on:[0408]`). Build-first-cutover-later. Door: one-way. Lane: 10-ci-adr-spec.*
*Tag: ASPIRATIONAL-TOOLING (the ADR is authored in this lane; it does not yet exist).*

### 3.1 The new ADR (the producer spec)
- **Number:** **`ADR-0515`** — next free above 0514, additive on stable numbering per D13-AMENDED (NOT a re-foundation-series number; the full ADR-0000+ re-foundation is deferred). dup-0377 is an independent mid-space collision (Phase-1 A-INTEGRITY) and does NOT gate this. Handle: `ADR-0515-oya-ci-cd-unified-rust-native-cicd`. *(The `00NN` placeholders in the §3.2 table = `0515`.)*
- **Title:** *"oya-ci / oya-cd — the unified Rust-native CI/CD product (two nouns, four faces, six layers; adopts the patterns of Prow + Tekton + Argo-WF + Argo-CD + Argo-Rollouts in Rust)."*
- **Status:** Accepted (door:one-way; founder-ruled). It is the **ratifying ADR** that reshapes the founder-locked ADR-0513, inheriting 0513's Accepted authority rather than re-opening it.
- **Home (D-CICD L169 + D-PURESPLIT L171-172):** `cloud/cloud-scm` + `cloud/cloud-ci` + `cloud/cloud-cd` — tenant-facing dogfood products under `cloud/` only; **no `oya/`→`cloud/` internal dep** (dogfood purity). Backlog #19.

### 3.2 The 6→1 consolidation / supersession map — 0408 stays SEPARATE (each ADR's fate + exact edge)
| ADR | Status (verified) | Fate | Exact edge to write | Doc-lifecycle verb |
|---|---|---|---|---|
| **0513** oya-ci bespoke-Rust Prow | **Accepted** (founder-locked) | **RESHAPED → ratified.** "clone Prow's 8 components" → two-nouns/four-faces. | new `supersedes:[0513]`; 0513 → `superseded_by:[00NN]`, status→Superseded. | Authority moves day-0; keep on disk (lineage). |
| **0511** Argo-WF orchestrator | **Proposed** (`supersedes:[0359]`, `superseded_by:[]`) | **RESOLVED by rejecting both poles** (not adopt-Argo-wholesale, not clone-all-Prow). Argo DAG/event ideas re-enter as Face C; etcd-CRD substrate dropped. | new `supersedes:[0511]`; 0511 → `superseded_by:[00NN]`. | Drop the ADR debt; archive-eligible after Phase-1. |
| **0514** target-arch + hyperscaler remediation | **Proposed** (`depends_on:[0392,0408]`) | **HONORED + folded as substrate.** D1–D6 re-homed onto the new ADR's Phase-1 deliverable list; CAS-hit>60% kept as a promotion gate. | new `supersedes:[0514]`; new ADR re-states D1–D6 so nothing lost. | Superseded; keep until Phase-1 deliverables land. |
| **0408** Buck2-driven build/CI | **Proposed** (`supersedes:[0358]`) | **STAYS SEPARATE (founder 2026-06-06).** Buck2 build = a distinct bounded context oya-ci USES; NOT collapsed (minimal-blast-radius). | **NOT superseded.** New CI/CD ADR (0515) adds `depends_on:[0408]`; amend 0408's references to cite 0515 for the orchestration it feeds. | Stays Accepted/authoritative for the build graph (with 0392). |
| **0349** Jenkins+ArgoCD substrate | **Proposed** (never ratified) | **SUPERSEDED.** Replaced by oya-ci + oya-cd `DeliveryPlane`. | new `supersedes:[0349]`; 0349 → `superseded_by:[00NN]`. **Resolve `byp_adr_0349_jenkins_argocd_substrate.yaml`.** | Bridge stays OPERATIVE-but-unratified until cutover; doc superseded now. |
| **0359** Jenkins replaces GHA | **Superseded** (`superseded_by:[0511]`; **body still says "Proposed"** — drift) | **Re-point.** | **`superseded_by:[0511]` → `[00NN]`** (0511 is itself superseded; skip to new ADR). Fix the status/body drift. | Already superseded; re-pointed. |
| **0361** Jenkins-native execution | **Proposed** (`amends:[0359]`) | **SUPERSEDED.** License-vetted supply-chain tool stack (cargo-deny/Opengrep/gitleaks/Syft/Trivy/osv/cosign/in-toto-SLSA/Kyverno) RETAINED as layer-e gate-step list. | new `supersedes:[0361]`; 0361 → `superseded_by:[00NN]`. | Bridge promote-pipelines operative until cutover; doc superseded now. |

**The central seam — 0511↔0513 whiplash — FIXED:** verified both have **no edge** yet directly contradict
(0511 L87 rejects "bespoke CI controller now"; 0513 IS it, Accepted one day later). Fix = collapse **both into
the new ADR** (`0511→00NN` AND `0513→00NN`), not a single cross-edge between two superseded docs. The new
ADR's body records the resolution: keep Argo's DAG/event ideas (0511) on the bespoke-Rust controller (0513).

**Build-first-cutover-later (D3/D-LAYER reconciled):** the new ADR is the single AUTHORITY from day-0; the
physical Jenkins/Argo scaffold + 0349/0359/0361 docs stay **operative-but-unratified** as an explicit bridge
until cutover, then retire ("superseded-on-cutover," NOT archived now).

**Phased-relation ADRs (NOT collapsed — `related` + phase-marked):** 0367 trustless-verification (Phase-1
keystone, "signature IS the check"), 0124 own-merge-queue (`supersedes` in-mechanism → graph `conflicts(a,b)`),
0369 stacked-trunk/speculative-merge (Phase-3), 0366 agentic self-enforcing (LAST, Phase-3→5), 0392 Buck2
build-graph (stays authoritative), 0181 cosign promotion-ladder (re-homes onto oya-cd signer).

### 3.3 The `oya-ci-required` PRODUCER CONTRACT (the firewall keystone the ADR must declare)
- **Producer identity:** a **deployed `oya-ci-controller`** (or minimal Rust bridge) — a service, NOT config on the branch it gates (deadlock-proof; kills ADR-0513 failure-mode-2 self-deadlock).
- **Trust model (trustless / trunk-sourced):** gate definitions come from **trusted trunk/controller state**; the PR tree is **DATA-UNDER-TEST ONLY**; a PR cannot weaken its own gate; **no `oya` CLI is merge authority**; **the signature IS the check** (producer ≠ verifier ≠ approver; ADR-0367).
- **What it posts:** context `oya-ci-required` (alias `cloud-ci-required`) on the **candidate PR-head SHA** (40-hex), as a `Phase0CiEnforcementResult` bundle conforming to `specs/phase0-ci-enforcement-result-schema.json` (`candidate_sha`, `producer{context,kind,trusted_control_state,gate_definition_source,candidate_bytes_policy}`, `fixture_results[]`, `observed_verdict`, `provenance`, `claim_boundary{p0_0_green,phase0_complete}`).
- **Robustness:** GREEN `tc-0.0-good-cloud-ci-required-and-isolated` + the 10 RED fixtures it MUST fail (candidate-mutable-producer, candidate-deletes-trusted-target, legacy-oya-cli-authority, missing-required-context, buck2-affected-only-producer, oya-verify/oya-gate-run-all producer, required-context-present-not-required, override-without-ttl-audit, cross-tenant-shared-cache). Then **apply the live ruleset** and re-run T0.0 before `claim_boundary.p0_0_green` may flip.

### 3.4 Component shape — which face reimplements which pattern in Rust (the ADR's normative §)
**Two nouns:** `Run` (Prow ProwJob × Tekton TaskRun × Argo Workflow — one typed content-addressed object;
MVP = single Postgres table behind `RunStore`, **NOT etcd CRDs**) + the **Buck2 build-graph + CAS** (the brain
— `affected-by` / `conflicts(a,b)` / `cache-key` / `provenance-subject` are one query family).

| Face | Adopts the pattern of | Rust reimplementation (what oya OWNS) | Substrate REUSED behind a port | Ownership |
|---|---|---|---|---|
| **A** Ingress + trustless gate + submit queue | **Prow** `hook`+Tide + Uber SubmitQueue | webhook→CloudEvent→one `Run` (HMAC fail-closed); **signature IS the check** (0367); serial Tide merge-queue; graph-exact `conflicts(a,b)` | `ForgeAdapter` (no single-token SPOF) | **OWN-now** |
| **B** Typed-Task executor | **Tekton** typed Task/Step | Rust-typed Task IR; a Step = a REAPI action (typed in Rust, not YAML) | `TaskRuntime` | **OWN-now** (combinators OWN-when-proven) |
| **C** DAG engine / scheduler | **Argo Workflows + Events** | DAG + CloudEvents correlation; **CAS-backed memoization** (not ConfigMaps); **Argo etcd-CRD substrate DROPPED** | `EventBus` (NATS/Kafka), `Scheduler` (K8s as one impl) | OWN-when-proven |
| **D** Out-of-band provenance signer | **Tekton Chains** / SLSA | out-of-band signer → `slsa/v1`, cosign, immutable ledger; **hermeticity-via-cache → SLSA-L3** | `Attestor`/`SigningBackend` (cosign/Fulcio/Rekor) | gate-signature OWN-now → full signer OWN-when-proven |
| **CD (oya-cd)** GitOps + progressive delivery | **Argo CD + Argo Rollouts** | `DeliveryPlane` owns signed-handoff + cosign-verify-on-sync + audit-chain | ArgoCD/Argo-Rollouts = **REUSE-behind-port** (NOT reimplemented MVP) | REUSE-behind-port |
| **brain** (layer d) | Bazel/Buck2 RE + hyperscaler graph | Buck2 graph as queryable object; REAPI v2 + CAS; `cquery rdeps` affected selection | REAPI v2 (NativeLink → owned when CAS-hit>60%) | graph+CAS OWN-now |
| **core** (two nouns) | Prow control-plane × hyperscaler graph | `RunStore` (Postgres single-table → sharded); K8s as a `Scheduler` impl | Postgres (REUSE) | OWN-now → sharded when write-rate-gated |

**Principle (D-CICD L169, verbatim):** *"do what Go does, in a cloud-native manner, but in Rust"* — adopt the
PATTERN, reimplement in Rust; **never run the Go binary**. Substrate that is not the moat (REAPI/CAS, event-bus,
kube-scheduler, cosign wire) is REUSE-behind-port. **Build engine:** oya owns the `BuildSpec`/`BuildStrategy`
contract, vendors BuildKit + Buildah behind the port (scratch/distroless default, SBOM+SLSA day-0; Kaniko
archived). Owned Rust build engine = OWN-when-proven (Phase-5), convergence with ADR-0408's buck2-native OCI.

### 3.5 Reshape of the existing Prow-shaped scaffold
KEEP `oya/ci-webhook-gateway`(=hook→`Run` producer) + `oya/ci-controller`(=plank, OWN the `Run` lifecycle) +
`oya/ci-tide`(=tide, RESHAPE engine to serial-now/speculation-later); SUPERSEDE the frozen ProwJob enum
(presubmit/postsubmit/periodic/batch → scheduling policy over the graph); DEFER deck/plugins/sinker to roadmap.

### 3.6 Founder flags — RESOLVED (founder 2026-06-06)
1. **0408/0514 — RULED:** 0514 **collapses** into the new CI/CD ADR (0515); **0408 stays SEPARATE** (Buck2 build = distinct bounded context; 0515 `depends_on:[0408]`, amend 0408 refs only — minimal-blast-radius). 0514's non-orchestration deliverables (hermetic-toolchain #83, buckify idempotence) re-home onto 0515's Phase-1 deliverable list so nothing is lost; 0408↔0392 build-graph relation untouched.
2. **Product spec in `linux/` — RESOLVED:** 0515 cites the real path `linux/docs/research/bespoke-ci-design-2026-06-06/40-PRODUCT-SPEC.md` now; **relocate the bundle into `source/docs/research/` in Phase-1** so the SSOT lives with the canon it governs.
3. **ADR-0359 status/body drift — RESOLVED:** fix during re-pointing in the A-INTEGRITY status-enum lane (Phase-1).
4. **New-ADR number — RESOLVED:** **ADR-0515** (next free above 0514, additive on stable numbering per D13-AMENDED). dup-0377 is an independent mid-space collision (Phase-1 A-INTEGRITY) and does NOT gate this number.

---

## §4 — THE PRODUCER BUILD (the live `oya-ci-required` producer that BLOCKS)
*Authority: D-SEQUENCE L198 ("stand up the live oya-ci-required producer"). Lane: 10-producer-plan.*

**Headline:** the producer is **~90% built and is the correct Prow-shaped decomposition** — but it **posts to
the WRONG FORGE** (Forgejo) while branch protection reads GitHub, so it is not a live producer of the enforced
gate. The work is **narrow: one adapter + wiring + a live ruleset flip**, not a green-field service.

### 4.1 EXISTS vs BUILD ledger (read from real code)
| Capability | Status | Evidence |
|---|---|---|
| Pure gate state machine → `oya-ci-required` | **ENFORCED-NOW** | kernel `src/lib.rs:471,484` (`GATE_CONTEXT`; total `map_job_to_status`) |
| `/gate-run` SHA-bound (rejects non-40-hex), fail-closed | **ENFORCED-NOW** | app `src/lib.rs:464-483` |
| Trunk-sourced gate Job (PR bytes untrusted; snapshot trusted targets before checkout) | **ENFORCED-NOW** | k8s-adapter `src/lib.rs:108-123` |
| Runner token isolation (FORGEJO_CI_TOKEN never to runner SA) | **ENFORCED-NOW** | k8s-adapter `src/lib.rs:167-175` |
| Forgejo status poster | **ENFORCED-NOW** | forgejo-adapter `src/lib.rs:72-122` |
| Webhook→/gate-run dispatcher (opt-in; **default = jenkins**) | **ENFORCED-NOW** | dispatch `src/dispatch.rs:222`; config `src/config.rs:90` |
| Phase-0 RED/GREEN policy engine + 11-fixture corpus (in `cargo test`) | **ENFORCED-NOW** | kernel `src/lib.rs:285,1441`; `specs/fixtures/phase0-ci-enforcement-baseline/*` |
| Helm chart (controller SA + low-priv runner SA + netpol + ESO) | **ENFORCED-NOW as TEMPLATE** (every value carries `nonClaim: …template-only`) | `iac/k8s/helm/values.yaml:25,32,36,91` |
| Branch-protection apply/verify (check-only, self-disclaimed, legacy-CLI verifier) | **ENFORCED-NOW (check-only)** | `scripts/branch-protection-apply.sh:129`; `dev.json:2,25` |
| **GitHub poster for `oya-ci-required`** | **BUILD** (HTTP shape reusable; wrong context+process today) | gateway github-adapter `src/lib.rs`; `tests/d5_commit_status_post.rs:22` (speaks `cargo-*`, not `oya-ci-required`) |
| **Forge-of-record decision** (GitHub vs Forgejo) | **BUILD/DECIDE** | mismatch: poster=Forgejo vs protection=GitHub (`baseline.json:82`) |
| **Forge-neutral `CommitStatusPoster` seam** | **BUILD** (small refactor of `ForgejoStatusPoster`) | kernel `src/lib.rs:620` |
| **Live branch-protection flip to `["oya-ci-required"]`** | **BUILD/ACT** | live set still `cargo-*`+`oya-pr-review` (`baseline.json:73-84`) |
| **Tide required-context = `oya-ci-required`** | **BUILD/FIX** (default is `oya-ci-gate`) | tide kernel `src/lib.rs:76` |
| **Real deploy + RBAC + ESO sync (drop nonClaims)** | **BUILD/ACT** | `values.yaml:25,32,36,91` |
| **Live GREEN/RED/tamper PR receipts + blocked-merge proof** | **BUILD** | exit-gate fixture `tc-0.12-…` still RED |

### 4.2 Minimal-viable producer that BLOCKS (the BUILD work) — ASPIRATIONAL-TOOLING
1. **DECIDE the forge of record (blocking design decision).** Recommend **Option A — GitHub** (smaller delta: live enforcement, `gh` tooling, fixtures, and baseline already assume GitHub; only a poster adapter + token wiring is net-new). Option B (move protection to Forgejo) re-homes all enforcement.
2. **Build `GitHubCommitStatusPoster` behind a forge-neutral `CommitStatusPoster` seam** (refactor of `ForgejoStatusPoster`, kernel:620). Reuse the proven HTTP shape from the gateway github-adapter (Bearer, `X-GitHub-Api-Version`, `POST /repos/<owner>/<repo>/statuses/<sha>`) but post context `"oya-ci-required"`. Pure kernel + reconcile loop stay unchanged.
3. **Flip live GitHub `dev` required contexts to exactly `["oya-ci-required"]`** (`branch-protection-apply.sh --apply`); **replace the verify step's `oya-dev-cli` dependency** (script:129) with the bespoke policy engine `evaluate_phase0_ci_policy` (so the verifier carries no legacy-CLI authority).
4. **Fix tide:** set `OYA_TIDE_REQUIRED_STATUS_CONTEXT="oya-ci-required"` (default is `"oya-ci-gate"`, tide kernel:76) — else the merge predicate reads a different context than the controller posts.
5. **Deploy for real** (drop the `nonClaim` markers; inject a GitHub crier token to the **controller ONLY, never the runner**; set the gateway to `OYA_CI_DISPATCHER=controller` + `OYA_CI_CONTROLLER_URL`).

### 4.3 Go-live RED/GREEN proof
- **Unit/policy layer — ENFORCED-NOW:** GREEN `tc-0.0-good-cloud-ci-required-and-isolated` + 11 RED fixtures, asserted by `phase0_fixture_corpus_executes_red_green_policy` (kernel:1441) which REQUIRES ≥1 GREEN + ≥1 RED. This is already a real RED/GREEN gate in `cargo test`.
- **Live "blocks a merge" layer — BUILD (ASPIRATIONAL-TOOLING):**
  1. **GREEN PR** → controller posts `oya-ci-required=success` on the GitHub HEAD SHA; assert via `gh api …/commits/<sha>/statuses` that `context==oya-ci-required && state==success` AND the PR is **mergeable**.
  2. **RED PR** (failing trusted-target test) → `=failure`; assert GitHub **refuses the merge** (proof it BLOCKS, not just reports).
  3. **Tamper PR** (edit `infra/ci/buck2-affected-gate.sh` to `exit 0` / delete a trusted target / self-post) → still `failure`/blocked (trunk-sourced + token isolation; live form of `tc-0.0.1a-bad-candidate-mutable-producer`).
  4. **CI-runs-and-blocks proof:** recorded `gh api …/required_status_checks == ["oya-ci-required"]` + a presubmit running the policy test. Until receipts exist on real SHAs, exit-gate fixture `tc-0.12-current-red-p0-0-live-context-missing` correctly stays RED (`p0_0_green:false`).

---

## §5 — THE ACCOUNTING-REGISTRY + THE 4 KEYSTONE GATES
*Authority: D-DOCTRINE L180 (total accounting) + D-SEQUENCE L198 (4 keystone gates). Lane: 10-gates-registry.*
*Tag: the registry + Gates 1/2/3 are **BUILD** (grep=0 hits in source); Gate-4 EXISTS as a seed and is HARDENED.*

### 5.1 The generated accounting-registry — `accounting-registry.generated.json` [BUILD]
**One generated record per `git ls-files` path** (the tracked-truth discipline; worktrees/`target/`/`buck-out/`
are runner-local, accounted by class not row). Gate-2 owns/produces it; Gates 1/3/4 are predicates/views over it.

**11 fields:** `path · unit_class(code|doc|spec|registry|evidence|vendor|build_config|generated|ephemeral|husk)
· owner(OWNERS-derived; null⇒RED) · justification_ref(→ADR/spec/need) · reachable_from[](masterplan|root-hub|
cargo-members|doc-catalog|crosswalk) · ttl{ttl_class,budget_days,action:report|archive|delete,protected}
· last_touch_commit · tracked · verdict(KEEP|ARCHIVE|MERGE|NEEDS-OWNER|RED) · dup_of · _provenance`.

**Invariants:** (1) `committed == regenerated` — a hand-edit to any generated face is itself RED (`registry_drift`),
so drift is structurally impossible; (2) total coverage — `set(rows) == set(git ls-files) − gitignored − ephemeral`;
(3) carve-outs (vendor/generated/ephemeral) live as DATA in the `unit_class`/`ttl-policy` tables, **never as
scanner code** (Linus: the exception lives in the table); (4) producer is a buck2 `rust_binary`, **never an `oya`
CLI** (register #20; retires ADR-0365's `oya gen`/`oya gate` defect).

**Companion generated faces (also `committed==regenerated`):** `decision-crosswalk.generated.json` (Gate-1),
`ttl-policy.generated.json` (Gate-3), `enforcement-inventory.generated.json` (Gate-4).

### 5.2 The 4 gate crates (each: blocking codes · RED fixture · GREEN fixture · self-test)
All are `cloud-ci-*` `rust_test` crates emitting `{verdict, violations[]}`; each test asserts
`assert_eq!(report.violations, fixture.expected_violations)` per fixture. **Bootstrapping honesty:** a gate stays
`automated_advisory_until_p0_0` until its self-test reproduces its live exhibit as RED on the current corpus;
only then may it flip to `automated_blocking_now`.

**GATE-1 `cloud-ci-cross-artifact-agreement` [BUILD]** (§P backlog:522; amends ADR-0365)
Codes: `orphan_decision · unpropagated_decision · status_disagreement · generated_face_drift · dual_decision_collision · supersession_half_edge`.
- **RED** `tc-XA-bad-axes-count-drift` (frozen live exhibit: `catalog.json:12 axes_count:6` vs `contracts.json:9 axes_count:7`); companions `tc-XA-bad-dup-adr-number` (the two ADR-0377 files), `tc-XA-bad-half-supersession` (0511 `superseded_by:[]`).
- **GREEN** `tc-XA-good-decision-all-four-agree` (ADR Accepted + spec + masterplan node + roadmap node + reciprocal supersession).
- **Self-test (born-blocking):** MUST emit `generated_face_drift` (axes 6 vs 7), `dual_decision_collision` (dup-0377), `supersession_half_edge` (0511) as RED before flipping.

**GATE-2 `cloud-ci-total-accounting` [BUILD]** (owns + produces the registry; D-DOCTRINE L180; pillar-G backlog:302)
Codes: `unaccounted · unowned · unjustified · unreachable · no_ttl_class · registry_drift`.
- **RED** `tc-TA-bad-orphan-no-justification` (foundry-residue: a file justified by ADR-0363 which claims it was "eradicated"); companions `…-new-file-no-row`(unaccounted), `…-no-owner`(unowned), `…-unreachable-from-masterplan`(unreachable), `…-hand-edited-registry`(registry_drift).
- **GREEN** `tc-TA-good-fully-accounted` (owner + resolving justification + non-empty reachability + ttl_class) + `tc-TA-good-archive-candidate-reported` (over-TTL orphan REPORTED, not deleted in-gate).
- **Auto-archive = report → `git mv` → `_archive/`, second-verifier-gated, NEVER `rm`** (founder rule: never delete on an unverified verdict).
- **Self-test (born-blocking):** flags the 780 `oya-foundry-*` files as `unjustified` + 57 `oya-governance-*` crates as `unreachable` + broadly `unowned` (live `find -name OWNERS` = 0 tree-wide).

**GATE-3 `cloud-ci-staleness-reaper` [BUILD]** (pillar-G sinker++ backlog:311; linux Task-#14 >48h)
Codes: `stale_over_budget_unreachable · untyped_staleness · reap_without_report`.
- **RED** `tc-SR-bad-stale-unreachable-doc` (>budget AND unreachable scratch ai-slop doc); companions `…-untyped-resource`, `…-reap-without-report`.
- **GREEN** `tc-SR-good-old-but-reachable-adr` (age alone ≠ stale) + `tc-SR-good-protected-not-reaped` (protected class never reaped) + `tc-SR-good-stale-reported-then-archived` (report→`git mv`, no `rm`).
- **Self-test (born-blocking):** reports the `synthesis/_partial-*`/`_verify-*` scratch artifacts (if over-budget AND unreachable) as archive candidates.

**GATE-4 `cloud-ci-automation-ratchet` (EXISTS as seed; HARDEN)** (register #20 backlog:661-668)
EXISTS on disk: `specs/phase0-automation-matrix.json` (`status:seed-contract-not-green`; `gate_contract.id:cloud-ci-automation-ratchet`; 10 required fields; 4-enum) + 4 live fixtures (1 GREEN/3 RED).
**REAL on-disk codes (authoritative — the prior design doc invented different names):**
`enforceable_or_automatable_marked_human_judgment · blocking_invariant_mapped_to_oya_cli · {duplicate_row_id, unknown_classification, missing_or_empty_required_field}` + NET-NEW `advisory_claiming_enforced · ratchet_regression`.
- **RED** `tc-0.16-bad-oya-cli-authority` (EXISTS; row whose `target_gate_or_controller` is an `oya` CLI call) + `tc-AR-bad-advisory-claiming-enforced` (NEW; claims "enforces" with no wired buck2 target).
- **GREEN** `tc-0.16-good-human-judgment-with-retirement-path` (EXISTS).
- **Self-test (born-blocking):** flags 57 `oya-governance-*` crates + `diataxis-doc-class` + `prd-axis-coverage` + ADR-0365's 7 `oya gate`/`oya gen` `verified_by` lines.
> **CORRECTION carried (load-bearing):** implement to the DISK codes (`blocking_invariant_mapped_to_oya_cli`, not `oya_cli_authority`; the triple, not `incomplete_exception`). Do NOT rename the live fixtures.

### 5.3 Buck2-native wiring (no new `oya` CLI) — register #20
- 1 producer `rust_binary` `//cloud/cloud-ci/gates:accounting-registry-producer` (emits registry + 3 companion faces).
- 4 gate `rust_test`s `:cross-artifact-agreement` / `:total-accounting` / `:staleness-reaper` / `:automation-ratchet`, each globbing `specs/fixtures/<gate>/tc-*.json`.
- 1 `:registry-drift` test (re-runs producer in sandbox, byte-diffs the committed registry — a hand-edit to any generated face fails it).
- **G-INTEGRITY track: NO buck2-build-graph dependency** (backlog:341) — operates on specs+filesystem+git, so it **ships in Phase-0 before the build migration** ("the false-green firewall must not wait"). Required cloud-ci context post-P0.0; until the live producer is proven, honest status = `automated_advisory_until_p0_0`.

---

## §6 — THE EXECUTION SPINE
*Authority: D-SEQUENCE L201 (the dependency spine + door:one-way points). Lane: 10-lane-spine.*

### 6.1 The dependency spine (locked order, D-SEQUENCE L201)
```
STEP-0 base commit (signed, founder-signed-off)
  │ door:one-way
  ▼
[P0.ADR] ADR-0515 — the ONE canonical CI/CD ADR (consolidate 0349/0359/0361/0511/0513/0514 → 1; 0408 stays separate, depends_on)   ◄ door:one-way + sign-off
  │ SEQUENTIAL hard chain
  ▼
[P0.PRODUCER / FE-1] wire controller into presubmit; post oya-ci-required on candidate SHA; apply ruleset   ◄ door:one-way + sign-off + CREDENTIALS
  │ SEQUENTIAL hard chain
  ▼
[P0.REGISTRY] accounting-registry.generated.json (buck2 rust_binary, NOT oya CLI)
  │
  ▼  ── then the 4 gates FAN OUT in parallel (predicates over the one shared registry) ──
  ├─ GATE-2 total-accounting (producer of record)
  ├─ GATE-1 cross-artifact-agreement
  ├─ GATE-3 staleness-reaper
  └─ GATE-4 automation-ratchet (polices Gates 1-3; test-time dep, not build dep)
  │
  ▼
[FIREWALL REAL]  oya-ci-required green ⇔ all 4 gates green on the candidate SHA   ◄ door:one-way + sign-off (EXIT gate)
  ▼
UNLOCKS PHASE-1 (A-CI / A-STRUCT / A-FOUNDRY / A-INTEGRITY / A-TASTE / A-IDENTITY — each now gate-verified)
```
**ADR → PRODUCER → REGISTRY are sequential (hard chain); the 4 gates fan out after the registry.** All four are
G-INTEGRITY track (no build-graph dep) so they ship Phase-0 before migration.

### 6.2 The self-proving bootstrap discipline (the heart of Phase-0)
Phase-0 artifacts are built UNDER advisory enforcement (the firewall they will become does not exist yet). The
discipline that prevents a fake-green Phase-0:
1. **Honest status start** — each gate starts `seed-contract-not-green` / `automated_advisory_until_p0_0`; never overstated.
2. **Born-already-blocking self-test** — a gate flips to `automated_blocking_now` ONLY after its RED/GREEN self-test reproduces its **live drift exhibit** as RED on the current corpus.
3. **Producer proven by a known-bad PR** — FE-1 go-live is proven not by assertion but by a deliberate RED PR that the producer MUST post `oya-ci-required=failure` against (captured check_run) + a known-good PR it passes.
4. **Gate-4 polices the others** — the ratchet flags any of Gates 1-3 that claims "enforced" without its self-test reproducing its exhibit (the ratchet polices itself).

### 6.3 Verification gates + door:one-way + founder-sign-off + credential points
| # | Gate | Type | What is verified / why irreversible |
|---|---|---|---|
| 1 | **STEP-0 base commit** | door:one-way + sign-off | fixes what is "in the canon" the firewall will police; everything left dirty is invisible to total-accounting |
| 2 | **The canonical CI/CD ADR** | door:one-way + sign-off | consolidates 7 ADRs into one; supersession is irreversible (build-first-cutover-later) |
| 3 | **FE-1 producer go-live** | door:one-way + sign-off **+ CREDENTIALS** | turning on real enforcement; needs **GitHub admin credential** to apply the ruleset + snapshot live `required_status_checks` |
| 4 | **Each source mutation** | per-PR sign-off | "every source mutation" is door:one-way (canon L201); ADR-0365 `decision-door`: `door:one-way` ADRs cannot auto-merge |
| 5 | **Firewall declared real** | door:one-way + sign-off | only after all 4 gates' self-tests reproduce their live exhibits as RED AND the producer blocks a known-bad PR; the Phase-0 EXIT gate that unlocks Phase-1 |

**Founder-held credentials:** G1 = `github-mirror` push (origin=Forgejo, never push); G3 = signing key (DONE —
re-confirm non-empty); **FE-1 = GitHub admin credential to apply branch protection + snapshot live state.**
Verification is a **separate verifier lane vs the real source** — no self-approval, no phantom findings.

### 6.4 The LOCKED rollback / checkpoint triggers (HARD HALT — backlog to founder, do NOT iterate)
- **CP-BUCK2-LINUX (the locked trigger):** per `docs/ideas/buck2-native-ci-gate.md` P0 — *"prove `buck2 build //... && buck2 test //...` green on Linux in-cluster … NOTHING else proceeds until this is green."* Only darwin is verified; Linux clang/triple fixups (psm/ring/aws-lc/openssl) unproven. If not green on the Linux gate runner → **HARD checkpoint** (infra-correctness blocker, not a code defect; iterating burns serial wall-clock). The G-INTEGRITY 4 gates ship regardless (no build-graph dep); the FULL firewall (build-correctness as a required context) is gated on this.
- **CP-AUTH-FLIP (G0):** `oya-ci-required` flips live mid-Phase-0 → every lane's Done-Definition shifts → HALT, founder decides pivot. (Each lane's STEP-0 re-diffs live protection vs baseline to detect this.)
- **CP-PRODUCER-RED:** FE-1 cannot post a real check_run / ruleset apply fails on credentials → firewall cannot go live → backlog until credentials/producer fixed.
- **CP-GATE-SELFTEST-FAIL:** a gate's self-test does NOT reproduce its live exhibit as RED → the gate is fake-green → it MUST NOT flip to blocking (claim-ceiling #21).

**Rollback mechanics:** squash-merge keeps reverts atomic + history linear; a clean per-lane revert is real ONLY
for cargo-side (non-blocking) failures — a merged lane can redden the next lane's GLOBAL buck2 graph, so after
ANY revert re-run the whole-graph `-check` matrix on rebased dev before resuming. Auto-archive = `git mv` to
`_archive/`, never `rm`, second-verifier-gated.

### 6.5 The ralph-loop structure (Phase-0 specialization of the WIP serial loop)
```
PRECONDITION (once):
  STEP-0 base committed + founder-signed-off
  && signing provisioned (G3 re-confirmed non-empty)
  && live protection baseline snapshot recorded (for G0 drift-detect)
  && founder sign-off on the canonical CI/CD ADR (door:one-way)

PHASE0_QUEUE = [ ADR, PRODUCER(FE-1), REGISTRY, GATE-2, GATE-1, GATE-3, GATE-4 ]
  # ADR→PRODUCER→REGISTRY sequential; the 4 GATES fan out after REGISTRY

while PHASE0_QUEUE not empty:
  lane = pop_front()                  # strict serial single driver; one graph mutation at a time
  STEP 0: re-diff live protection vs baseline -> if flipped to oya-ci-required: HALT (CP-AUTH-FLIP/G0)
  STEP 1: rebase phase0/<lane> on github-mirror/dev
  STEP 2-7: build the artifact (ADR / Rust producer / gate crate); Cargo+Buck2 DUAL build;
            if buck2 first-party build/test RED on Linux: HALT (CP-BUCK2-LINUX)
  STEP 8: SELF-TEST — run the gate's RED/GREEN fixtures; RED MUST reproduce the live exhibit as a block;
          if not: HALT (CP-GATE-SELFTEST-FAIL); do NOT mark enforced
  STEP 9: prove-by-known-bad-PR (PRODUCER lane) — post oya-ci-required=failure on a known-bad SHA
          (captured check_run) + pass a known-good SHA
  STEP 10: multispectrum evidence + 5-H2 PR body + DOC-CATALOG/CHANGELOG rows
  STEP 11: push github-mirror; gh pr create --base dev; signed commits + linear history
  STEP 12: drive github-lane-unlocker-required GREEN + resolve conversations
  STEP 13: door:one-way founder sign-off where required (PRODUCER go-live; else per-PR sign-off)
  STEP 14: SQUASH-merge -> rebase-on-dev + re-run authority gate (keep next lane honest)
  STEP 15: flip lane status seed-contract-not-green -> automated_blocking_now (ONLY after STEP-8 passed)

TERMINATION: all 4 gates automated_blocking_now (self-tests green) && producer proven by known-bad PR
             && oya-ci-required is a REQUIRED context posting on candidate SHAs
             => FIREWALL REAL => door:one-way founder sign-off => UNLOCKS PHASE-1 => /cancel
```
**BUILD-TO-BOTH-GATES (WIP R1):** the live required context during Phase-0 is still `github-lane-unlocker-required`
(Buck2-whole-graph, signatures off); the *target* is `oya-ci-required`+signing. Phase-0 IS the work that makes
`oya-ci-required` real, so lanes build to the live unlocker AND are the producer of the target. **The boulder
never stops:** within a lane a cargo-side red iterates; the CP-* checkpoints are the ONLY legitimate HALTs — they
backlog to founder, they do not iterate.

---

## §7 — EXIT CRITERIA (Phase-0 is DONE only when ALL hold)
Phase-0 unlocks Phase-1 **only** when every one of these is proven — not asserted (D-SEQUENCE L198-201; D-DOCTRINE L181):

1. **The ONE canonical CI/CD ADR (ADR-0515) is authored + Accepted** (door:one-way, founder-signed-off); the 6→1 supersession edges are written with reciprocal links (0408 NOT collapsed — `depends_on` edge only); the 0511↔0513 seam is closed by collapsing both into it.
2. **`oya-ci-required` posts LIVE and BLOCKS a known-bad PR (proven):** a real check_run shows `=failure` on a known-bad candidate SHA on the forge branch protection reads (GitHub), and GitHub **refuses the merge**; a known-good PR posts `=success` and is mergeable; a tamper PR is still blocked (trunk-sourced + token isolation). Recorded `gh api …/required_status_checks == ["oya-ci-required"]`. Exit-gate fixture `tc-0.12-…` flips from RED.
3. **The accounting-registry generates + validates:** `accounting-registry.generated.json` is produced by the buck2 `rust_binary` (not `oya` CLI), total coverage holds, and `committed == regenerated` (the `:registry-drift` test passes).
4. **The 4 keystone gates are wired + RED/GREEN-proven + self-tests pass:** each emits its on-disk violation codes; each RED fixture reproduces its **live drift exhibit** (Gate-1: axes 6 vs 7 + dup-0377 + 0511 half-edge; Gate-2: 780 foundry orphans + 57 unreachable governance crates + OWNERS=0; Gate-3: an over-budget unreachable scratch doc; Gate-4: 57 governance crates + ADR-0365 oya-CLI `verified_by`); each GREEN passes; each is `automated_blocking_now` (flipped only after STEP-8).
5. **No CP-* checkpoint is open** (buck2 Linux green or explicitly backlogged with founder; no auth-flip pending; no fake-green gate).

**Only then** does Phase-1 (A-CI 6→1 consolidation, A-INTEGRITY dup-0377 renumber + status-enum, A-FOUNDRY
ADR-0363 fix, A-STRUCT pure-split, A-TASTE, A-IDENTITY) proceed — each amendment gate-verified, unable to re-drift.

---

## §8 — COVERAGE / WHAT THIS PLAN DID NOT COVER (no silent caps)
- **Assembled from:** the 4 lane artifacts in `_phase0/` (10-ci-adr-spec / 10-producer-plan / 10-gates-registry / 10-lane-spine, all read in full), plus live re-verification of: canon D-CICD(L168-169)/D-PURESPLIT(L171-172)/D-DOCTRINE(L177-184)/D-SEQUENCE(L195-201); source branch=`feat/oya-ci-tide`, 79 dirty, remotes (origin=Forgejo / github-mirror=GitHub); `GATE_CONTEXT="oya-ci-required"` at kernel:471; baseline forge=GitHub `jason931225/oyatie` (baseline:82).
- **This plan is an ASSEMBLY, not a build.** It did NOT author the ADR, build the producer adapter, write the registry producer or any gate crate / fixture JSON, run anything live (`gh`/kube/buck2), or re-query the live GitHub ruleset — all "live" non-source-tree claims are sourced from the checked-in baseline + the lanes, not re-observed this session.
- **Counts carried (not re-derived this session):** 780 `oya-foundry-*` residue / 57 `oya-governance-*` crates / 346-id ADR space — reused from the prior verification lanes; honest foundry figure per canon L201 = `microservices/foundry/` 597-file shell + ~4110 mentions (NOT 201 un-renamed crates — worktree-inflated).
- **Founder flags — all RESOLVED 2026-06-06 (see §3.6):** (1) 0408/0514 → 0514 collapses, **0408 stays SEPARATE** (`depends_on`); (2) product spec cited at its real `linux/` path, relocate to `source/` in Phase-1; (3) ADR-0359 drift → A-INTEGRITY status-enum lane; (4) new-ADR number = **ADR-0515** (dup-0377 is independent). Gate-1/2/3 violation codes are this design's contract (their fixture dirs are [BUILD] — do not yet exist); only Gate-4 codes are reconciled against real on-disk fixtures.
- **Path corrections carried from the lanes:** real files are `/Users/jasonlee/Developer/source/infra/ci/buck2-affected-gate.sh` and `/Users/jasonlee/Developer/source/.github/branch-protection.yaml` (the task-cited `oya/infra/ci/…` and `infra/branch-protection/.../branch-protection.yaml` paths do not exist).

*End PHASE-0 FALSE-GREEN FIREWALL PLAN. Authority: D-SEQUENCE (order) · D-CICD (oya-ci pattern) · D-DOCTRINE (enforcement) · the 4 _phase0 lane artifacts. READ-ONLY assembly; no source mutated.*
