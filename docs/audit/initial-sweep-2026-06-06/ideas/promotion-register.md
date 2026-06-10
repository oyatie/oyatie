# Ideas / Plans / Research Promotion Register — initial sweep 2026-06-06

> **IDEAS/PLANS PROMOTION auditor.** READ-ONLY synthesis; the only write is this file.
> Scope: every NON-ADR idea/plan/research doc in SOURCE (`~/Developer/source`, GitHub `jason931225/oyatie`,
> 346 ADRs) + LINUX (`~/Developer/linux`, substrate PILOT, 26 ADRs). Each doc classified:
> **PROMOTE-to-new-ADR** | **FOLD-into-existing-ADR(id)** | **REMOVE/ARCHIVE** (garbage/stale/superseded) |
> **KEEP-as-research**. Founder GOAL: the MASTERPLAN becomes the single source of truth — *"if it is not part
> of the masterplan, it is not needed."* `masterplan-membership` noted per promote/fold item; `authored-vs-generated`
> is an OPEN founder question (do NOT assume — see §0).
>
> Baseline trusted as ground truth: the keystone map (`_map/canonical-posture-and-supersession-map.md`) and the
> prior LINUX register (`cross-tension/_prior-wm4gkcey5-linux-register.md`). Every "already an ADR" claim below was
> grep-verified against `source/docs/decisions/`.

---

## 0. The masterplan-membership lens (binding for every PROMOTE/FOLD row)

`source/docs/MASTERPLAN.md` is an explicit **compatibility projection for humans, NOT the implementation authority**;
canonical authority = `/specs/masterplan.json`. Two SOURCE design docs contradict on HOW decisions reach the masterplan:

- **`planning-ssot-consolidation.md`** → masterplan is **GENERATED from ADR front-matter** (ADRs = immutable authored SSOT).
- **`planning-ssot-drift-prevention.md`** → **masterplan.json IS the one authority**; ADRs **bind into** it (`masterplan_ref`).

Therefore **every PROMOTE/FOLD item is masterplan-relevant under BOTH readings**: under generated-from-ADRs it enters
the masterplan *by becoming an ADR with `planning_impact: true`*; under authored-as-SSOT it enters by being *backfilled
+ bound (`masterplan_ref`)*. The `decision_atom` column gives the one-line decision that must land either way.
**This auditor does not resolve authored-vs-generated; it is flagged on every masterplan-touching row.**

The MASTERPLAN.md body itself carries **two retired-vocabulary leaks** the founder's SSOT goal will have to scrub
(surfaced, not edited): the FD-001 surface list still names **`foundry`** (retired brand → `cloud-intelligence` +
`governance`, ADR-0335/0347) and Development-Order step 5 still says **"Jenkins required checks"** (CI destination is
now Argo Workflows, ADR-0511; Jenkins = transitory bootstrap). These are masterplan-content defects, not idea-docs,
but they are the strongest evidence that the masterplan is **not yet** the faithful SSOT the founder wants.

---

## 1. COUNTS PER CLASS

**Total docs scanned: 35** (20 SOURCE ideas incl. 3 archive · 4 SOURCE plans + 1 impl-plan + 1 M01 INDEX bundle ·
5 amendment ADRs assessed as a side-question · 5 LINUX research + 2 LINUX migration).

| Class | Count | Items |
|---|---:|---|
| **PROMOTE-to-new-ADR** | **2** | agent-execution-controller (decision-pending) · affected-gated-migration-engine ("Sweep") |
| **FOLD-into-existing-ADR** | **9** | best-of-both→0384 · oauth-pool→0384 · agentic-slo-gated-promotion→0139 · buck2-native-ci-gate→0392/0408 · nativelink-remote-cache-first→0514 · oya-ci-bespoke-prow→0513 · build-ci-pipeline-review→0513/0514 · cicd-pipeline-revamp→0511(via 0359) · pipeline-optimization→0408/0514 |
| **REMOVE/ARCHIVE** (stale/superseded/garbage) | **11** | 3 ideas/archive/* (already superseded by 0389/0390/0391) · single-bootstrap-omni-talos (self-superseded by 0375) · hyperscaler-gap-closure-plan (M01 execution residue) · post-cutover-program (T1-T3 absorbed/dated) · rename-plan-2026-05-12 (self-superseded) · rename-plan-v4 (cutover done) · cutover-cross-cutting-amendments (cutover done) · M01-foundation-cc-01-cutover/* (stale, archive per SSOT design) · IP-INTEL-MIGRATE-CANONICAL (execution IP under 0509) |
| **KEEP-as-research** | **8** | hyperscaler-practices-to-adopt (backlog) · planning-ssot-consolidation · planning-ssot-drift-prevention · LINUX: hyperscaler-production-roadmap · beat-or-parity-scorecard · distributed-database-architecture-research · go-to-rust-backlog.json · kernel-consensus.json + round1-lanes.json (raw lane dumps) |
| **PILOT-SCAFFOLD (retire-at-integration, keep-until-then)** | **2** | LINUX migration/source-consolidation-plan.md · source-manifest.md |
| **AMENDMENT-ADRs (side-question: real decisions, NOT noise)** | **5** | 0239 · 0353 · 0354 · 0355 · 0356 |

> Net: only **2 true new ADRs** are owed from this entire idea/plan corpus. The corpus is overwhelmingly
> **already-promoted** (9 fold) or **spent execution residue** (11 remove). That is the healthy signal: source's
> idea→research→ADR pipeline (ADR-0365) mostly worked; the backlog of un-captured truth is small.

---

## 2. MUST-PROMOTE (the only genuinely un-captured decisions)

### P1 — `source/docs/ideas/agent-execution-controller.md` → **PROMOTE-to-new-ADR (decision-pending)**
- **Status:** PR #605 treated as MERGED; canonical, decision-pending (promote-as-narrower vs decline). NOT slop.
- **Why net-new (verified):** grep of `source/docs/decisions/` for `agent-execution|pod-runner|work-item|evidence-bundle`
  returns only unrelated ADRs. `cloud/cloud-intelligence` (ADR-0384/0389/0390/0391) is the **inference/token gateway**;
  `oya/intelligence` (ADR-0255) owns substrate/supervisor/capabilities. **No ADR owns agent *execution*** — "run this
  agent CLI as a K8s Job and hand back sealed evidence." This is the one concept the source corpus does not cover.
- **decision_atom:** *"Adopt a flat single-concern agent-execution controller (work-item.v1 + pod-schedule-plan.v1 +
  evidence-bundle.v1; Cedar-gated claim; K8s-Job/Talos-API split; provider-native CLI lifecycle preserved) as a
  net-new narrower service — OR explicitly decline it as forbidden by ADR-0116/0363 (retire agent-coordination /
  agentic-VCS)."*
- **masterplan-membership:** YES (new substrate service → `planning_impact: true`). **Founder gate first:** the doc
  itself flags the key question — is this layer *wanted* or *retired-on-purpose* (ADR-0116/0363 killed the adjacent
  coordination layer). Promote-as-narrower OR decline; the ADR records whichever. Provenance repo `~/Developer/oya-code`
  has salvageable Rust (`crates/`, `examples/work-items/`) **if** accepted.
- **Cross-tension flag:** declares **Talos machine API + K8s Job + Forgejo-shaped** surfaces — consistent with source
  canon but inherits the **forge fault-line** (founder=GitHub vs canon=Forgejo); the controller must not hard-bind a forge.

### P2 — `source/docs/ideas/affected-gated-migration-engine.md` ("Sweep") → **PROMOTE-to-new-ADR**
- **Why net-new (verified):** grep `affected-gated|sweep engine|auto-quarantine` in `decisions/` → **0 hits.** The
  engine (one reusable Workflow parameterized by transform/unit-discovery/verify-cmd/risk-class: bulk-rollout +
  adversarial-verify gate + parallel worktree lanes + gate-failure auto-triage → auto-merge-on-green) is referenced
  by `oya-ci-bespoke-prow.md` ("the Sweep engine becomes a tide client") but is **not itself** ADR-0111 (merge-queue)
  nor any CI ADR. It is a distinct *mass-transform execution pattern*.
- **decision_atom:** *"Adopt the affected-gated mass-migration engine ('Sweep'): a risk-classed Workflow that fans
  bulk transforms across crates behind the unchanged buck2 affected gate, auto-quarantines failures (KNOWN_FAILING),
  and auto-merges the green majority with no human in the merge loop — feeding, never weakening, the gate."*
- **masterplan-membership:** YES (eng-productivity substrate consumed by every migration; `planning_impact: true`).
  **Caveat:** scope it as a **tide client** (per oya-ci-bespoke-prow / ADR-0513) so it does not re-introduce a bespoke
  speculative merge-queue that ADR-0111/0513 already own. Likely lands as a *narrow* ADR or a deliverable folded under
  ADR-0513 — flag for the founder which (promote-standalone vs fold-into-0513).

> **Everything else that looks like a "new idea" is already an ADR or already dead.** No third promotion is owed.

---

## 3. MUST-REMOVE (highest-confidence garbage / stale / superseded)

### R1 — `source/docs/ideas/archive/*` (3 files) → **REMOVE/ARCHIVE — already superseded, leave frozen**
`cloud-intelligence-bedrock-on-talos` (front-matter `superseded_by: ADR-0389`), `cloud-intelligence-v1-pipeline`
(`superseded_by: ADR-0390`), `n-lane-parallel-safety-and-unified-devops-console` (`superseded_by: ADR-0391`).
All three **self-declare superseded** and the named ADRs **exist on disk** (verified). They are correctly already in
`ideas/archive/`. **Action: none** — keep frozen as historical record; do NOT re-promote, do NOT delete (audit trail).

### R2 — `source/docs/ideas/single-bootstrap-omni-talos.md` → **REMOVE/ARCHIVE — self-superseded**
Top banner: *"SUPERSEDED 2026-05-27 by ADR-0375 (Talos + CAPI + Argo CD)."* The Omni-Managed recommendation is dead;
ADR-0375 exists. Still sitting in the live `ideas/` dir (not archived) — **move to `ideas/archive/`**. Retired-vocab it
carries: `oyaCiLane.groovy` / Jenkins-as-CI (now Argo, 0511), Tier-3 (now tenant-class, 0329).

### R3 — `source/docs/plans/rename-plan-2026-05-12.md` → **REMOVE/ARCHIVE — self-superseded chain**
Front-matter `status: Superseded`, `superseded_by: rename-plan-v2-2026-05-12.md`; the live authority is **v4.1**
(`rename-plan-v4-clean-arch-2026-05-13.md`). Pure historical record; "foundry-fitness umbrella" framing is itself
retired (ADR-0347 foundry-fitness→governance). Archive.

### R4 — `source/docs/plans/rename-plan-v4-clean-arch-2026-05-13.md` (217 KB) → **REMOVE/ARCHIVE — cutover executed**
`status: approved`, `execution: approved-by-user-2026-05-13`. This is the **executed** 140-crate rename cutover plan.
The rename happened (source now has 723 workspace members per the migration manifest); this is a giant spent execution
artifact, not a decision. Its *durable* decisions already live in ADR-0015/0017/0054/0056/0057 (its `related_adrs`).
**Archive** (do not delete — it is the provenance of the BNF). Not masterplan material.

### R5 — `source/docs/plans/cutover-cross-cutting-amendments-2026-05-12.md` + `M01-foundation-cc-01-cutover/*` (8 files)
→ **REMOVE/ARCHIVE — cutover done; SSOT design explicitly says archive this**
`planning-ssot-consolidation.md` §"Keep/Merge/Retire" **names this exact directory**: *"ARCHIVE: `docs/plans/M01-foundation-cc-01-cutover/`
(stale; to `.omc/archive/…`)"* and flags M01-* as superseded 2026-05-19. The 12-directive stack inside cross-cutting-amendments
is **dated** (mentions Foundry-owned visualization, `oyaCiLane`, Jenkins, Redis-exclusion-as-forward-note) — its still-live
directives (distroless, LTS-pins, provider-agnostic adapters) are doctrine already captured elsewhere. **Archive the whole M01 bundle.**

### R6 — `source/docs/ideas/hyperscaler-gap-closure-plan.md` → **REMOVE/ARCHIVE — M01 execution residue**
A 30-gap artifact-authoring checklist for the **M01 observability changeset** (threat-model/dpia/cost-budget/… file
inventory). All assumptions `[x]`-confirmed 2026-05-17. It is execution bookkeeping (which files to author at which
paths), not a decision; its one real decision (SLO-gated promotion) is **ADR-0139**. Uses retired `M01` wave vocab and
references already-superseded suite ADRs. Archive.

### R7 — `source/docs/ideas/post-cutover-program.md` → **REMOVE/ARCHIVE — absorbed + dated**
T1 (stand up Forgejo + Jenkins commit-status), T2 (colocate intelligence crates under ADR-0131/0357), T3 (first
identity service). T1 collides with the **forge fault-line** (Forgejo + Jenkins, both now contested/transitory:
0510/0511) and the founder's GitHub directive; T2 is owned by ADR-0357 + IP-INTEL-MIGRATE; T3 is owned by ADR-0476
(`oya-identity` bespoke). No un-captured decision remains. Archive (its diagnosis — "spec-saturated, code-starved" — is
already doctrine in ADR-0322 substance-bar).

### R8 — `source/docs/implementation-plans/IP-INTEL-MIGRATE-CANONICAL.md` → **REMOVE/ARCHIVE-WHEN-DONE — pure execution IP**
`status: Draft`; an implementation plan *under* canonical **ADR-0509** (hyperscaler single-crate decomposition) +
precedent ADR-0476. IPs are execution artifacts, not decisions — they do not become ADRs and are not masterplan
authority. Keep only until the 121→13 intelligence collapse lands, then archive. (If source adopts the
generated-masterplan model, IPs live as `deliverables` under ADR-0509, not as standalone docs.)

---

## 4. FOLD-INTO-EXISTING-ADR (already-decided; idea is the pre-ADR draft — collapse the reference, don't re-promote)

| Idea doc | Fold into | masterplan? | decision_atom (already captured by the ADR) |
|---|---|---|---|
| `ideas/cloud-intelligence-best-of-both.md` | **ADR-0384** (llm-gateway OAuth pool redesign) | yes | Clean-room Rust multiplexing gateway over pooled subscriptions (gpt-load core + one-api breadth; OpenBao keys; per-agent quota; Valkey-coordinated HA). Idea = the design draft behind 0384. |
| `ideas/cloud-intelligence-oauth-subscription-pool.md` | **ADR-0384** | yes | Per-tenant Cedar-isolated SubscriptionPool + audit/billing/analytics event spine. Same ADR; "forward-pointer to in-flight OAuth ADR" — that ADR is 0384. |
| `ideas/agentic-slo-gated-promotion.md` | **ADR-0139** (agentic-slo-gated-promotion) | yes | Event-driven per-component promotion gated by Google-SRE multi-window multi-burn-rate SLO engine on adopted Grafana/Mimir stack. Idea name == ADR name. Retired vocab inside: "Foundry-native SLO engine" → governance/intelligence; `oya-governance-canary-*` already corrected. |
| `ideas/buck2-native-ci-gate.md` | **ADR-0392** (buck2 canonical build graph) + **ADR-0408** (buck2-driven CI/CD) | yes | Pivot the gate from cargo-interim to buck2-native (BXL affected query; presubmit-affected/postsubmit-full; hermetic toolchains; NativeLink RE). |
| `ideas/nativelink-remote-cache-first.md` | **ADR-0514** (build/CI/CD target arch) — NativeLink named in 0392/0408/0514 | yes | NativeLink CAS+AC cache-only first on Talos (keyed auth, 3-tier split), RE later. Carries a **founder-decided 2026-05-30** block (keyed-not-anon; CAS/scheduler split) — ensure that decision is reflected in the ADR body, else it is the one un-folded atom here. |
| `ideas/oya-ci-bespoke-prow.md` | **ADR-0513** (oya-ci bespoke-Rust Prow) | yes | Bespoke-Rust Prow-shaped CI/CD platform (hook/plank/crier/tide/deck/plugins → Rust) unifying gate + merge-queue + reviewer-approve + ChatOps; supersedes Jenkins (0380) + folds 0111. Idea == ADR. |
| `ideas/build-ci-pipeline-review.md` | **ADR-0513** + **ADR-0514** | yes | Refined target arch: hermetic toolchain, durable post-buckify third-party, trunk-isolated/depth-capped/CAS-backed gate, controller-over-Jenkins. This is the SIMPLIFY review feeding 0513/0514; no new decision beyond them. |
| `ideas/cicd-pipeline-revamp.md` | **ADR-0511** (Argo supersedes Jenkins) via the 0359→0511 chain | yes | "Collapse 36 GH-Actions into Jenkins-native license-clean pipeline." **STALE at the Jenkins layer** — Jenkins-as-destination is retired (0511). Only the *license-clean stage order* (cargo-deny/Opengrep/gitleaks/SBOM/cosign/SLSA/Kyverno→Kubewarden) survives; fold that into the supply-chain ADRs, drop the Jenkins framing. |
| `ideas/pipeline-optimization.md` | **ADR-0408**/**ADR-0514** (affected-precision + cache) | yes | Precise affected-targets, kill double-cargo, trunk-warmed cache, test sharding, merge-queue speculation. Optimization tactics under the buck2/CI ADRs; "tune GH Actions / Jenkins" framing is retired. |

> **Fold note (founder-relevant):** under **generated-from-ADRs**, these idea docs should be *deleted/archived* once
> their ADR carries the full `deliverables` — the idea is dual-source drift risk. Under **authored-as-SSOT**, they stay
> as `explanation/` rationale but must be `masterplan_ref`-bound to their ADR so the drift gate (planning-ssot-coverage,
> currently 8.8% bound) passes. Either way they are **not** independent decisions.

---

## 5. KEEP-AS-RESEARCH (durable input; not a decision; must NOT be promoted as-is)

| Doc | Side | Why keep (not promote) |
|---|---|---|
| `ideas/hyperscaler-practices-to-adopt.md` | SOURCE | A 24-item prioritized **adopt backlog** (two-way-door ADR field, error-budget freeze, COE→gate, flaky-quarantine, DORA, PR-FAQ, Kayenta canary…). Each item is *itself* a future decision that flows through ADR-0365 (research→consensus→ADR). It is a research backlog, not one decision. Several items already became ADRs (0365/0366/0367/0368). KEEP as the live adopt-queue. |
| `ideas/planning-ssot-consolidation.md` | SOURCE | **The authored-vs-generated design (generated side).** Load-bearing for the founder's masterplan-SSOT question. KEEP as the canonical reference for the GENERATED-from-ADR model (KEP precedent, ADR template schema, re-found-from-ADR-0000). This is the doc the founder must rule on — do not collapse it. |
| `ideas/planning-ssot-drift-prevention.md` | SOURCE | **The authored-vs-generated design (authored side):** masterplan.json IS authority; ADRs bind via `masterplan_ref`; 8.8%-bound today. KEEP as the counter-design. The pair (this + consolidation) frames the OPEN founder decision; **a future ADR must pick one** — flag as the single highest-leverage masterplan decision outstanding. |
| `research/hyperscaler-production-roadmap.md` | LINUX | 7-lane Talos/k8s control-plane research synthesis (etcd/iptables/flat-CP ceilings; concrete Rust crate choices). Explicitly *operationalizes* existing operating-system D21/D22 — "no architecture conflict." Pure research; feeds backlog, not a decision. KEEP. |
| `research/beat-or-parity-scorecard.md` | LINUX | Competitive **target tracker** (beat-or-parity vs Linux/Asterinas/Talos) with an unusually honest status header (M1/M2 complete; "Asterinas more mature today"; targets ≠ claims). KEEP as the measurement ledger; not a decision. |
| `research/distributed-database-architecture-research.md` | LINUX | Deep-research synthesis (CockroachDB/Spanner 5-layer, multi-Raft, TrueTime/HLC) feeding `distributed-database-engine-canonical.json` and ADR-0001. Q4/Q6/Q7/Q8 explicitly OPEN/unverified. KEEP as the ADR-0001 evidence base. |
| `research/go-to-rust-backlog.json` (222 KB) | LINUX | Six-lane Go→Rust dependency-graph conversion backlog (k8s + Talos). Prioritized engineering backlog, not a decision. KEEP. |
| `research/kernel-consensus.json` (109 KB) + `round1-lanes.json` (91 KB) | LINUX | Raw multi-lane research dumps (HW CPU-feature consensus; perf/security/efficiency lanes). Working artifacts feeding the roadmap + scorecard. KEEP as raw evidence; **candidates to RETIRE-at-integration** (migration plan §1 marks `round1-lanes.json` RETIRE, `kernel-consensus.json` AMBIGUOUS). Not masterplan material. |

---

## 6. PILOT-SCAFFOLD — `LINUX/docs/migration/*` (keep until integration, then retire)

| Doc | Class | Note |
|---|---|---|
| `migration/source-consolidation-plan.md` | **PILOT-SCAFFOLD (retire-at-integration)** | Self-declares *"Pilot-only scaffold (retired at integration). This plan is NOT itself migrated."* Drives the 6-tree → `~/Developer/source` migration. **Founder-relevant locked decisions (2026-06-06) live here, not yet in any ADR:** (1) GitHub IS canonical now ("forgejo is old, github is new") — this is the founder's forge directive in writing, and it **explicitly says** *"Forgejo references in source/docs/AGENTS.md are stale on this point"* (the forge fault-line, founder side); (2) db-engine home = `cloud/cloud-data`; (3) std-first / no_std-last sequencing; (4) Buck2-whole-graph is the real merge gate (not cargo). These four are **promotable to a migration ADR** if the founder wants them in the masterplan — flagged, not auto-promoted (scaffold doctrine says retire-at-integration). |
| `migration/source-manifest.md` | **PILOT-SCAFFOLD (retire-at-integration)** | Per-tree first-party crate inventory + codename→descriptive rename rules (`oyaoffice-*`→`oya-office-*`, `ctrd_*`→`oya-cloud-container-platform-*`, `talos-*`→`oya-cloud-node-os-*`, `oyago/oyapy`→transpiler-*). Truthing artifact for merge-not-duplicate. Verified consistent with keystone-map's container-platform rename (ADR-0017). Retire at integration. |

> **Verify-the-recent-LINUX-edits task (wm4gkcey5):** the prior workflow's edits to `source-consolidation-plan.md`
> (container-platform rename on line 46; ADR-count "currently 26" on lines 26/53/83/85) are **CORRECT** and match the
> keystone map's ADR-0017 authority + the live 26-ADR corpus. **Not "plain wrong."** The migration docs are internally
> coherent with the canonical-posture map. No erroneous reconciliation found in this idea/plan/research slice.

---

## 7. SIDE-QUESTION — the "amendment" ADRs (0239, 0353, 0354, 0355, 0356): **REAL decisions, NOT noise**

These are out of strict idea/plan scope (they live in `decisions/`) but the brief asked the question. Verdict:
**they are legitimate `amends:` ADRs, not noise** — each narrows/clarifies a parent decision and is correctly modeled.

- **ADR-0239** `amends: ADR-0136` — Foundry scope = INTERNAL-only; consumer AI = `microservices/intelligence/` (ADR-0220).
  Real boundary decision. **BUT historically anchored to the retired "Foundry" brand** — 0239 is on the supersession path
  ADR-0136→0239→0335 (foundry retired→intelligence). It is a *valid step in a chain that ended in retirement*: keep as
  history, but its live truth is fully absorbed by ADR-0335. Flag: do NOT treat 0239's "Foundry" framing as current.
- **ADR-0353/0355/0356** (`status: Proposed`) `amends: ADR-0246 / ADR-0255 / ADR-0257` — "library-first network opt-in"
  clarifications to policy-engine-substrate, intelligence-two-layer, and ontology read-path. Real architectural amendments,
  multi-council owned. **Still `Proposed`** — live decisions awaiting accept, legitimately tracked. Note 0355 still lists
  `axis-foundry` owner + `ADR-0050-event-bus-kafka` (Kafka retired→Pulsar, 0377) in `related` — **stale-vocab leakage** to
  flag, not a reason to kill the ADR.
- **ADR-0354** (`status: Proposed`) `amends: ADR-0253` — HTTP/3 fallback + strict-TLS/ECH/PQC network amendment. Real
  network-security decision, clean. Legitimate.

**Net:** the amendment ADRs are a sound mechanism (`amends:` ≠ supersession; the keystone map already notes amendments
supersede *microservice doc-pairs*, not ADRs). They are **not noise**. The only cleanup they need is retired-vocab
scrubbing (`foundry`/`axis-foundry`/`Kafka`) in their metadata — a lint pass, not a promotion/removal.

---

## 8. CROSS-TENSION FLAGS RAISED BY THIS SLICE (surface, do not resolve)

1. **Forge fault-line, founder side, now in writing.** `migration/source-consolidation-plan.md §0.4` states GitHub IS
   canonical and source's Forgejo refs are "stale" — directly contradicting source canon (Forgejo transitory + bespoke-VCS
   destination, ADR-0363/0510). Two LIVE docs in two repos assert opposite canonical forges. **Surface to founder.**
2. **Jenkins/CI in MASTERPLAN.md is stale.** Dev-order step 5 "Jenkins required checks" contradicts ADR-0511 (Argo).
   If the masterplan is the SSOT, this is a content bug; if generated-from-ADRs, the generator would have caught it →
   evidence the masterplan is currently **hand-authored and drifting** (the exact problem both SSOT designs target).
3. **`foundry` in MASTERPLAN.md FD-001 surface list** contradicts ADR-0335/0347 (brand retired). Same drift signal.
4. **agent-execution-controller vs ADR-0116/0363.** Promoting P1 must clear the founder gate: is agent *execution* a
   net-new narrower concern, or forbidden by the same retirement that killed agent-coordination/agentic-VCS? Decision-pending by design.
5. **Sweep engine vs merge-queue ownership.** P2 must not re-own what ADR-0111/0513 (tide) own — promote-narrow or fold-into-0513.

---
*End of promotion register. Trust the superseding ADR over stale idea framing; treat `foundry`/`Jenkins-as-destination`/
`Kafka`/`tier`/`M01-wave` as retired vocab wherever an idea doc still uses it. Only 2 true new ADRs are owed
(both founder-gated). authored-vs-generated masterplan model remains the single highest-leverage OPEN decision.*
