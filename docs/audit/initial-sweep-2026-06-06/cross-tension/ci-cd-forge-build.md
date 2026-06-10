# Cross-Tension Register — Theme: CI/CD / Forge / Build

> Contradiction-hunter pass, theme `ci-cd-forge-build`, initial sweep 2026-06-06.
> READ-ONLY synthesis. No audited doc was modified. Binding baseline = the keystone map
> (`_map/canonical-posture-and-supersession-map.md`) and the prior linux register
> (`_prior-wm4gkcey5-linux-register.md`).
> SOURCE = `~/Developer/source` (jason931225/oyatie, 346 ADRs). LINUX = `~/Developer/linux` (26 pilot ADRs).
> Convention: "govern" = the latest/locked decision that wins; resolutions are SURGICAL
> (cross-ref / supersede-edge / clarify), never new policy. Founder-call items are flagged DECISION-NEEDED.

---

## 0. Theme corpus read (file refs)

SOURCE (`docs/decisions/`):
- `ADR-0124-own-merge-queue-webhook-driven.md` — own merge-queue, GitHub-webhook-driven (`status: accepted`, `superseded_by: none`)
- `ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md` — Jenkins+ArgoCD substrate (`Proposed`)
- `ADR-0359-jenkins-completely-replaces-github-actions.md` — Jenkins sole CI (`Superseded`, `superseded_by:[ADR-0511]`)
- `ADR-0361-jenkins-native-cicd-revamp-execution.md` — Jenkins-native execution (`Proposed`)
- `ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate.md` — retire bespoke VCS; Forgejo canonical (`Accepted`, `amended_by:[0510,0513]`)
- `ADR-0374-ci-webhook-gateway-forgejo-jenkins.md` — gateway, posts **Forgejo** commit-status (`Accepted`)
- `ADR-0377-forgejo-board-git-ref-cas-fallback.md` — Forgejo board projection (`Proposed (conditional)`) — DUPLICATE NUMBER with `ADR-0377-kafka-to-pulsar-via-kop.md`
- `ADR-0380-ci-loop-closure-on-talos-jenkins-farm-re-establishment.md` — Jenkins farm on Talos (`Accepted (amendment)`, `superseded_by:[]`)
- `ADR-0387-ci-webhook-gateway-forgejo-to-jenkins-commit-status.md` — gateway, D5 posts **GitHub** commit-status (`Proposed`)
- `ADR-0392-buck2-canonical-build-graph.md` — Buck2 build graph, reverses ADR-0358 §2 (`Proposed`, `supersedes:[ADR-0358]`)
- `ADR-0408-buck2-driven-ci-cd.md` — Buck2 CI engine, reverses ADR-0358 §2 (`Proposed`, `supersedes:[ADR-0358]`)
- `ADR-0510-scm-bespoke-hyperscaler-destination-cutover-trigger.md` — Forgejo transitory; bespoke-VCS destination (`Proposed`, `amends:[ADR-0363]`)
- `ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md` — Argo Workflows destination; supersede 0359 (`Proposed`, `supersedes:[ADR-0359]`)
- `ADR-0513-oya-ci-bespoke-rust-prow-cicd-platform.md` — bespoke-Rust Prow `oya-ci` (`Accepted`, founder-locked; phased-supersedes 0380)
- `ADR-0514-build-ci-cd-pipeline-target-architecture-hyperscaler-remediation.md` — target arch + remediation (`Proposed`)
- `ADR-0173-vendor-lock-in-avoidance-and-stack-ownership.md` — vendor doctrine (`Accepted`; legacy markdown-table header, Forgejo as Tier-II replacement target)

LINUX (`docs/decisions/`, `docs/context/`):
- No forge/CI/merge-queue ADR exists. The pilot's only build-toolchain stance is in context docs:
  `docs/context/engineering-conventions.md:18,72` ("native **Buck2** graph is a wiring lane") and
  `docs/context/cloud-native-stack.md:414` ("immutable **Talos** images"). `ADR-0025-node-os-rust-talos-*`
  competes with the adopted-Talos orchestration posture (out of theme; noted in §8).

---

## 1. THE SUPERSESSION CHAIN (current truth for the theme)

Two parallel chains plus a build-graph reversal. Reconstructed from on-disk front-matter + bodies:

### 1.1 CI-orchestration chain (the long churn)
```
ADR-0050 (GH-Actions automation-first, pre-theme)
  └─ ADR-0124 (own merge-queue, GitHub-webhook-driven)           [accepted; substrate now retired by 0363]
  └─ ADR-0349 (Jenkins AUGMENTS GH-Actions + ArgoCD)             [Proposed]
       └─ ADR-0359 (Jenkins COMPLETELY REPLACES GH-Actions)      [Superseded → 0511]
            └─ ADR-0361 (execute Jenkins-native revamp)          [Proposed; Kyverno/Argo-Rollouts named]
            └─ ADR-0380 (Jenkins farm re-established on Talos)    [Accepted; phase-superseded by 0513]
                 └─ ADR-0513 (oya-ci bespoke-Rust Prow)          [ACCEPTED, founder-locked — destination platform]
       └─ ADR-0408 (Buck2 is the ENGINE Jenkins/Argo invokes)    [Proposed]
       └─ ADR-0511 (Argo Workflows = destination orchestrator;   [Proposed — supersedes 0359]
                    Jenkins = transitory bootstrap)
```
**Net current truth (CI orchestration):** Jenkins is **transitory bootstrap only**. The *named destination* is
**Argo Workflows** (ADR-0511, k8s-native CNCF) — but ADR-0513 (the only **Accepted** node, founder-locked)
declares the destination is a **bespoke-Rust Prow `oya-ci`** platform (`hook`/`plank`→`oya-ci-controller`/`crier`/`tide`/`deck`/`plugins`).
**These two destinations are not yet reconciled in the corpus (see Tension T-1).**

### 1.2 Forge/SCM chain
```
ADR-0173 (vendor doctrine: GitHub Tier-II, replacement target = self-hosted Forgejo)   [Accepted]
  └─ ADR-0363 (retire bespoke oya-vcs/oya-git; git + Forgejo PRs canonical; GitHub=bootstrap-only) [Accepted]
       ├─ ADR-0374 (Forgejo→Jenkins webhook gateway; posts FORGEJO commit-status)       [Accepted]
       ├─ ADR-0377-forgejo-board (Forgejo Issues board + git-ref CAS claims)            [Proposed (conditional)]
       ├─ ADR-0387 (Forgejo→Jenkins gateway; D5 posts GITHUB commit-status)             [Proposed — CONTRADICTS 0374/0363]
       └─ ADR-0510 (Forgejo reframed TRANSITORY; bespoke hyperscaler monorepo-VCS = destination, numeric-trigger cutover) [Proposed — amends 0363]
```
**Net current truth (forge):** canonical host = **self-hosted Forgejo** (transitory per ADR-0510), gate sink =
**Forgejo Commit-Status API**, GitHub = **bootstrap host only**. Long-horizon destination = a **bespoke Rust monorepo-VCS**
(Piper/Sapling/Mononoke-class), cutover gated on numeric thresholds. **This collides head-on with the founder's
GitHub (`jason931225/oyatie`) migration directive (Tension T-4).**

### 1.3 Build-graph reversal
```
ADR-0358 §2 (Bazel rules_rust + crate_universe + Bazel-RBE)   [Proposed — the reversed input]
  ├─ ADR-0392 (Buck2 + buck2-prelude + Reindeer + NativeLink-RBE)  [Proposed — supersedes 0358 §2 build-graph]
  └─ ADR-0408 (Buck2-driven CI: cquery rdeps affected-targets)     [Proposed — supersedes 0358 §2 CI-engine]
```
**Net current truth (build):** **Buck2** (Rust binary) + Reindeer-buckified `third-party/rust/BUCK` +
self-hosted **NativeLink** RBE. Bazel is retired-as-input. The machine specs (`cloud-toolchain-target.json`,
`masterplan.json` P-TOOLCHAIN) still encode Bazel — **declared superseded inputs awaiting regeneration** (Tension T-7).
**LINUX ALIGNS** here (pilot already uses a native Buck2 graph) — no cross-side conflict on build.

---

## 2. TENSION T-1 — Two named CI destinations: Argo Workflows (0511) vs bespoke-Rust `oya-ci` Prow (0513)

| Position | ADR | File / ref | Status |
|---|---|---|---|
| Destination CI orchestrator = **Argo Workflows** (k8s-native CNCF, Argo Events trigger), Jenkins transitory; "a bespoke CI controller is a *future option behind the same evidence bar, not this decision*" | ADR-0511 §1, §Rejected-alternatives | `0511:47-55, 87` | Proposed |
| Destination CI = **bespoke-Rust Prow `oya-ci`** (`oya-ci-controller` = plank, `oya-ci-tide` = merge-queue, `oya-ci-deck`, `oya-ci-plugins`); "adopt the shape, not the code, in pure Rust"; deletes the Jenkins gate path at Phase-1 | ADR-0513 Decision + feature-parity table; ADR-0514 §2 target arch | `0513:66-101`, `0514:97-119` | **Accepted (founder-locked)** |

**True contradiction or reconcilable?** **Reconcilable in principle, but currently UNRECONCILED in the text — and the
two ADRs do not cite each other's destination claim.** ADR-0511 explicitly defers "bespoke CI controller" to a *future*
evidence bar and names Argo Workflows as *the* destination; ADR-0513 (authored one day later, 2026-05-30, and the only
**Accepted/founder-locked** node) builds exactly that bespoke controller *now* as the destination and says nothing about
Argo Workflows. ADR-0514 (`depends_on: 0392,0408`) elaborates the `oya-ci` controller path and never mentions Argo
Workflows either. So the corpus simultaneously asserts "Argo Workflows is the destination, bespoke is deferred" and
"the bespoke Rust platform is the founder-locked destination, cut over at Phase-1."

**Which governs?** **ADR-0513** — it is the only `Accepted`/founder-locked node in the chain; ADR-0511 is `Proposed`.
The plausible true synthesis (not yet written down): **Argo Workflows is a transitory/bridge orchestrator on the road
to the bespoke `oya-ci` Prow platform** — the same "keep-what-works, name-the-destination, migrate-when-green" pattern
ADR-0511 itself invokes for Jenkins. But that demotes ADR-0511's Argo Workflows from "destination" to "second transitory
stage," which ADR-0511 does not say.

**Proposed resolution (surgical):**
- Add a reconciliation note + cross-ref edge between ADR-0511 and ADR-0513 stating the layering explicitly: either
  (a) Argo Workflows is the destination and `oya-ci`/Tide is a governance overlay *invoked by* Argo Workflows, OR
  (b) `oya-ci` Prow is the destination and Argo Workflows is a second transitory bridge after Jenkins. The two ADRs
  must name the same destination.
- Until resolved, downstream/masterplan should treat **ADR-0513 (`oya-ci`) as governing** (it is Accepted) and
  **ADR-0511 (Argo Workflows) as Proposed/contested**.

**DECISION-NEEDED-FROM-FOUNDER:** *Is the destination CI orchestrator (a) Argo Workflows (CNCF, adopt-OSS) with `oya gate`
as the only bespoke piece, or (b) the bespoke-Rust `oya-ci` Prow platform (ADR-0513) that replaces the orchestrator
itself? ADR-0511 and ADR-0513 currently name different destinations; only one can be the masterplan CI node.*

---

## 3. TENSION T-2 — `ci-webhook-gateway` posts to GitHub (ADR-0387) vs Forgejo (ADR-0374): same service, two ADRs, opposite sink

| Position | ADR | File / ref |
|---|---|---|
| Gateway kicks Jenkins; **Jenkins posts the required statuses to the Forgejo Commit-Status API**; "the producer must not certify its own work"; GitHub bootstrap-only per 0363 | ADR-0374 Decision §4, Context | `0374:60-66, 96-100, 142-143` |
| Same gateway, but **D5 posts the 5 contexts to GitHub via `gh api repos/<owner>/<repo>/statuses/<sha>`**; title "Forgejo → Jenkins → **GitHub** Commit-Status Bridge"; binds **ADR-0112** (retired by 0363) and **ADR-0359** (superseded by 0511) as live | ADR-0387 title, D5, §Binding-ADRs | `0387:34-36, 43, 66-78` |

**True contradiction or reconcilable?** **True intra-corpus contradiction at the gate-sink level.** ADR-0374 (`Accepted`,
2026-05-26) and ADR-0387 (`Proposed`, 2026-05-28) describe the *same* microservice (`oya-ci-webhook-gateway`) but route
the merge-gating commit-status to *different forges*. ADR-0387's GitHub sink also contradicts ADR-0363 §1 (GitHub is
bootstrap-only, Forgejo is the required-status producer) and ADR-0511 §4 (Forgejo Commit-Status is the sink). ADR-0387
additionally rests on two dead anchors: **ADR-0112** (superseded by 0363) and **ADR-0359** (superseded by 0511).
Note: ADR-0387 also drifts on the signature scheme (ed25519) vs ADR-0374 (HMAC-SHA256) — a second same-service divergence.

**Which governs?** **ADR-0374** (Accepted) + **ADR-0363/0511** (Forgejo-canonical sink) govern. ADR-0387's GitHub-status
sink is the odd one out — it reads like a draft written against the GitHub-bootstrap reality (and accidentally matches
the founder's GitHub directive), but it conflicts with the locked Forgejo posture.

**Proposed resolution (surgical):**
- **AMEND ADR-0387**: repoint D5 from `gh api .../statuses` (GitHub) to the **Forgejo** Commit-Status API to match
  ADR-0374/0363; reconcile the signature scheme with ADR-0374's HMAC-SHA256 (or record why ed25519 supersedes it);
  drop the live citations to retired ADR-0112 (→ 0363) and superseded ADR-0359 (→ 0511). If ADR-0387 is in fact a
  superseded earlier draft of ADR-0374, mark it `Superseded → ADR-0374` instead of amending.
- Because the poster moves to Argo Workflows at the SCM-cutover (ADR-0511 §4), note that the *sink* (Forgejo) is
  stable while only the *poster* changes.

**DECISION-NEEDED-FROM-FOUNDER (forge-coupled):** *Resolved by the T-4 forge ruling — if the founder's GitHub directive
wins, ADR-0387's GitHub sink is actually correct and ADR-0374 must flip; if source's Forgejo canon wins, ADR-0387 must
flip. The two gateway ADRs cannot both stand.*

---

## 4. TENSION T-3 — ADR-0124 own-merge-queue is built on a triple-retired substrate

| Position | ADR | File / ref |
|---|---|---|
| Bespoke webhook-driven merge-queue running as a **GitHub Actions** workflow on **GitHub** webhook events; built on `oya-foundry-vcs-*` kernels (ADR-0111/0112/0113); rejects GitHub's *native* merge queue | ADR-0124 Decision §3, taxonomy | `0124:31-63, 84-104` |
| Merge automation belongs in **cloud-ci/oya-ci Tide** on the **Forgejo** substrate; ADR-0110/0112/0113 retired; `oya vcs`/foundry-VCS crates deleted | ADR-0363 §3, ADR-0513 `oya-ci-tide` | `0363:64-65`, `0513:80` |

**True contradiction or reconcilable?** **Stale-superseded — the *mechanism* is dead, a *requirement* survives.**
ADR-0124's entire substrate is retired: (a) `oya-foundry-vcs-*` crates retired (ADR-0335/0363), (b) ADR-0111/0112/0113
superseded by ADR-0363, (c) GitHub-Actions-as-CI superseded by ADR-0359→0511, (d) GitHub-as-substrate rejected by
ADR-0363 §Rejected. Yet ADR-0124 carries `superseded_by: none` — pure stale-front-matter drift (keystone §6). The
surviving true atom is the **O(N²)-cascade-breaking projected-merge-state + file-overlap-clustering algorithm**, which
ADR-0510 §5 and ADR-0513 explicitly relocate into `oya-ci-tide` (folding ADR-0111).

**Which governs?** **ADR-0363 §3 + ADR-0513** (`oya-ci-tide` owns the merge-queue) govern. ADR-0124 is historical.

**Proposed resolution (surgical):**
- **ARCHIVE ADR-0124**: set `status: Superseded`, `superseded_by: [ADR-0363, ADR-0513]` (front-matter currently says
  `none`). Add a one-line salvage note that the projected-state + file-overlap-cluster algorithm survives into
  `oya-ci-tide` (ADR-0513 / ADR-0111), so the valuable algorithm is not lost on archive.

**No founder call needed** — this is mechanical supersession cleanup.

---

## 5. TENSION T-4 — FORGE THREE-WAY: GitHub (founder directive) vs Forgejo (source canon) vs bespoke-VCS (declared destination)

This is the keystone fault-line (#4) landing on this theme. It touches **every** ADR in §1.2 plus the gateways.

| Position | ADR(s) | File / ref |
|---|---|---|
| **GitHub `jason931225/oyatie`** is the migration target (founder directive; the actual repo the corpus migrates to) | Founder directive (binding context); ADR-0124 GitHub-native; ADR-0387 D5 GitHub status; ADR-0173 lists GitHub as current host | `0173:162-172`, `0124`, `0387` |
| **Self-hosted Forgejo** is canonical host; GitHub bootstrap-only; required-status via Forgejo Commit-Status | ADR-0363 §1, ADR-0374, ADR-0377-board, ADR-0511 §4; ADR-0173 replacement-target=Forgejo | `0363:39, 52-53`, `0173:164-172` |
| **Bespoke hyperscaler monorepo-VCS** (Piper/Sapling/Mononoke-class, Rust) is the *decided destination*; Forgejo merely transitory; cutover on numeric trigger | ADR-0510 §1-3 | `0510:43-62` |

**True contradiction or reconcilable?** **Genuine open conflict — must be surfaced, not resolved (per binding context).**
The founder's GitHub directive conflicts with *even the transitory* layer of source canon: ADR-0363 rejects
"GitHub merge-queue/branch-protection as the substrate" outright (`0363:87`), and ADR-0510 declares the long-horizon
destination is a bespoke VCS that *replaces* the forge entirely. Note the layering nuance that softens it slightly:
ADR-0363/0247 treat GitHub as the **bootstrap host** today — so "we currently push to GitHub `jason931225/oyatie`" and
"Forgejo is canonical destination" can *coexist* as host-now/host-later. The hard conflict is the **gate-sink + merge
substrate**: source routes those through Forgejo/oya-ci, while the founder's directive (and ADR-0124/0387) wires them
to GitHub.

**Which governs?** Undecidable without the founder. Source canon (latest locked: ADR-0363 `Accepted`, ADR-0510/0511
`Proposed`) says Forgejo→bespoke. Founder directive says GitHub. **The latest *source* decision favors Forgejo; the
founder's stated migration favors GitHub.** This is exactly the case the binding context says to flag, not resolve.

**Proposed resolution (surgical, conditional on the founder ruling):**
- No content edit this pass. Record the three positions as a single masterplan "open forge decision" node so the
  downstream backfill does not silently pick one. The dependent edits (ADR-0124 archive target, ADR-0387 sink flip,
  ADR-0374 sink, ADR-0511 §4 sink) all *derive* from this ruling and should be sequenced behind it.

**DECISION-NEEDED-FROM-FOUNDER (the load-bearing one for the whole theme):** *Is the canonical forge **(a)** GitHub
`jason931225/oyatie` (your migration directive — makes ADR-0387's GitHub status-sink correct and ADR-0124's GitHub
merge-queue revivable, and demotes ADR-0363/0510 Forgejo canon to "bootstrap that became permanent"), **(b)** self-hosted
Forgejo with GitHub as mirror/bootstrap (source canon ADR-0363/0374/0511 — GitHub is push-target-now only, gate sink is
Forgejo), or **(c)** Forgejo-transitory → bespoke Rust monorepo-VCS (ADR-0510, full ownership)? Every gateway/merge-queue/
status-sink ADR in this theme resolves off this single ruling.*

---

## 6. TENSION T-5 — Jenkins is named "sole/destination" in 0349/0359/0361/0374/0380/0387 but is now "transitory bootstrap only"

| Position | ADR(s) | File / ref |
|---|---|---|
| Jenkins is the **canonical/sole/destination** CI orchestrator | ADR-0349 (canonical substrate), ADR-0359 (sole), ADR-0361 (Jenkins-native execution), ADR-0374 §Orchestrator-authority ("Decision: Jenkins-as-orchestrator"), ADR-0380 (farm re-established), ADR-0387 (Jenkins runner) | `0359:35`, `0374:188`, `0380` |
| Jenkins is **transitory bootstrap only**; destination is Argo Workflows (0511) / bespoke `oya-ci` (0513); the Jenkins gate path is **deleted at oya-ci Phase-1** | ADR-0511 §1, ADR-0513 Phase-1, ADR-0514 §5 | `0511:47-55`, `0513:91-98`, `0514:146-150` |

**True contradiction or reconcilable?** **Reconcilable — clean supersession already expressed, but with residual
stale framing + one missing edge.** ADR-0359 correctly carries `status: Superseded, superseded_by:[ADR-0511]` (verified
on disk) — that edge is clean. The drift is: (a) **ADR-0380** is `Accepted, superseded_by:[]` but its body says it is the
"phased replacement … by ADR-0513" — **missing the `amended_by:/superseded_by:[ADR-0513]` back-edge**; (b) ADR-0349/0361
remain `Proposed` with Jenkins-canonical framing and no note that Jenkins is now transitory; (c) ADR-0374's
"Decision: Jenkins-as-orchestrator" (`0374:188`) is now overtaken by ADR-0511/0513.

**Which governs?** **ADR-0511 (destination = Argo Workflows) + ADR-0513 (Accepted, destination = oya-ci)** govern;
Jenkins-as-destination is retired everywhere. (The internal Argo-vs-oya-ci destination question is Tension T-1.)

**Proposed resolution (surgical):**
- **AMEND ADR-0380**: add `amended_by: [ADR-0513]` (or `superseded_by` at Phase-1 cutover) to match its own body
  ("phased replacement of ADR-0380's Jenkins+Groovy gate path").
- **AMEND ADR-0349 / ADR-0361**: add a one-line "Jenkins is transitory bootstrap per ADR-0511; destination is
  Argo Workflows / oya-ci (ADR-0513)" header note so their Jenkins-canonical framing is not read as live.
- **AMEND ADR-0374**: annotate the "Jenkins-as-orchestrator" resolution (`0374:168-194`) as overtaken by ADR-0511/0513.

**No founder call needed beyond T-1** — this is supersession-edge hygiene.

---

## 7. TENSION T-6 — Admission-engine drift inside the CI chain: Kyverno (0361) vs Kubewarden (0379)

| Position | ADR | File / ref |
|---|---|---|
| Shift-left stack ends in **`Kyverno verifyImages` admission** | ADR-0361 Decision §2-3 | `0361:36-39` |
| **Kubewarden** is default admission (supersedes ADR-0183's Kyverno; Kyverno demoted to first-class adapter) | ADR-0379 (keystone §1.1, posture §Policy) | keystone map L38, L111 |

**True contradiction or reconcilable?** **Reconcilable stale-ref** — ADR-0361 (2026-05-25) predates ADR-0379's
Kubewarden default. The supply-chain lane's admission step needs a mechanical repoint. (ADR-0380 already names ADR-0379
in its `related`, so the substrate knows about Kubewarden; only ADR-0361's prose is stale.)

**Which governs?** **ADR-0379** (Kubewarden default). Governs ADR-0361's admission step.

**Proposed resolution (surgical):** AMEND ADR-0361 §2-3 admission step `Kyverno verifyImages` → `Kubewarden (default,
ADR-0379); Kyverno acceptable as adapter`. Mechanical; no founder call.

---

## 8. TENSION T-7 — Machine specs still encode Bazel/Jenkins while ADRs reverse them (generated-vs-authored masterplan)

| Position | ADR / artifact | File / ref |
|---|---|---|
| Canonical build = **Buck2 + NativeLink**; CI engine = Buck2 `cquery rdeps`; **the Bazel specs are explicitly "SUPERSEDED INPUTS awaiting a separate generated-artifact update — OUT OF SCOPE for this docs-only PR"** | ADR-0392 §Context, ADR-0408 §Context | `0392:59`, `0408:55` |
| `specs/cloud-toolchain-target.json`, `specs/masterplan.json` P-TOOLCHAIN still say **Bazel `rules_rust` + crate_universe + Bazel-RBE**; MASTERPLAN.md FD-001 promotion path still says **Jenkins required checks** | `cloud-toolchain-target.json`, `masterplan.json`, `MASTERPLAN.md` | keystone map L126 (MASTERPLAN.md "Jenkins required checks (note: stale — CI is now Argo Workflows per ADR-0511)") |

**True contradiction or reconcilable?** **Reconcilable but load-bearing for the OPEN masterplan question.** ADR-0392/0408
deliberately leave the machine specs stale and self-declare them superseded inputs. Under the **generated-from-ADRs**
masterplan reading, regenerating from ADR front-matter would *fix* this automatically (the ADRs carry `supersedes:[ADR-0358]`).
Under the **authored-masterplan-as-authority** reading, MASTERPLAN.md + `cloud-toolchain-target.json` must be hand-edited
to drop Bazel/Jenkins — and until then the human projection is **wrong** (says Bazel+Jenkins, truth is Buck2+Argo/oya-ci).
This is the keystone §4 open question instantiated on concrete, TRUE, currently-unbacked CI/build decisions.

**Which governs?** The **ADRs** (0392/0408/0511/0513) carry current truth; the specs are stale by their own admission.

**Proposed resolution (surgical):** No content edit (this is the masterplan-mechanism question, not a per-ADR fix).
Flag that the CI/build masterplan node must be (re)written from ADR-0392/0408/0511/0513 truth, and that MASTERPLAN.md's
"Jenkins required checks" line + `cloud-toolchain-target.json`'s Bazel entries are stale-pending-regeneration.

**DECISION-NEEDED-FROM-FOUNDER (masterplan mechanism, theme-scoped instance):** *Should the CI/build masterplan node be
GENERATED from the ADR `supersedes:` edges (0392/0408 → drop Bazel; 0511/0513 → drop Jenkins automatically), or AUTHORED
directly into MASTERPLAN.md / `cloud-toolchain-target.json` (requiring a manual Bazel→Buck2 + Jenkins→Argo/oya-ci rewrite
first)? The Bazel/Jenkins residue in the specs is the concrete cost of leaving this open.*

---

## 9. TENSION T-8 — ADR-0377 number collision (forge-board vs kafka-to-pulsar)

| Position | ADR | File / ref |
|---|---|---|
| `ADR-0377-forgejo-board-git-ref-cas-fallback.md` — Forgejo board projection, `Proposed (conditional)` | ADR-0377 (forge) | `0377-forgejo-board:1-3` |
| `ADR-0377-kafka-to-pulsar-via-kop.md` — eventing, `Accepted`, supersedes ADR-0005 | ADR-0377 (eventing) | keystone §6.1 |

**True contradiction or reconcilable?** **Genuine ID collision** (keystone §6.1) — two authoritative ADRs share number
0377 in the same directory. Touches this theme because the forge-board one is in-scope. A generated-from-ADRs masterplan
graph keyed on `ADR-NNNN` would mis-merge them.

**Which governs?** Neither "wins" on merits; one must renumber. The `Accepted` kafka-to-pulsar one has the stronger claim
to keep 0377 (it supersedes ADR-0005 and is referenced by that edge); the `Proposed (conditional)` forge-board one is the
natural renumber candidate.

**Proposed resolution (surgical):** Renumber `ADR-0377-forgejo-board` to the next free number (per ADR-0510's own
`numbering_note` discipline, first free above dev's highest). Repoint its inbound refs. No founder call beyond the
generic ADR-id no-reuse invariant (already flagged corpus-wide).

---

## 10. CROSS-SIDE (LINUX ↔ SOURCE) FINDINGS FOR THIS THEME

**Build toolchain — LINUX ALIGNS with SOURCE (no conflict).** The pilot's `engineering-conventions.md:18,72` already
uses a **native Buck2 graph** ("native Buck2 graph is a wiring lane"), matching source's Buck2 reversal (ADR-0392/0408).
This is the *rare* place the two repos agree — the linux pilot does **not** contradict the source build decision; if
anything it is downstream evidence for it. No tension to log on build.

**Forge / CI / merge-queue — LINUX is SILENT (gap, not conflict).** The pilot has **no** forge, CI-orchestration, or
merge-queue ADR. So there is no linux-vs-source contradiction on the theme's core; the source-internal tensions (T-1..T-9)
stand alone. On merge, the linux pilot inherits whatever forge/CI the founder rules (T-4/T-1).

**Verify the wm4gkcey5 auto-reconciliation (binding-context task) — for this theme: NOT "plain wrong."** The prior
linux register's 16 fixes were all DB/isolation/naming/citation hygiene (Postgres scope, container-platform rename,
proven-bar, dangling ADR refs). **None touched CI/CD/forge/build**, and none introduced a forge/CI claim. The one
build-adjacent edge — the pilot's Buck2 usage — is internally consistent and source-aligned. No theme-relevant
reconciliation error found. (Out-of-theme note: ADR-0025 Rust-Talos competes with source's adopted-Talos orchestration,
but that is the §5.3 isolation/orchestration fault-line, not CI/CD.)

---

## 11. RESULTING DISPOSITION CHANGES (driven by these tensions)

| ADR | Current disposition (front-matter) | Tension-driven change | Driver |
|---|---|---|---|
| **ADR-0124** | `accepted`, `superseded_by: none` | **→ ARCHIVE** (`Superseded → 0363, 0513`); salvage projected-state+file-overlap algorithm into oya-ci-tide | T-3 |
| **ADR-0359** | `Superseded → 0511` (already correct) | **KEEP edge** (no change; confirm Argo-Rollouts/Kyverno body refs stale) | T-5 |
| **ADR-0361** | `Proposed` | **→ AMEND** (Jenkins-transitory note; Kyverno→Kubewarden admission) | T-5, T-6 |
| **ADR-0349** | `Proposed` | **→ AMEND** (Jenkins-transitory note per 0511) | T-5 |
| **ADR-0380** | `Accepted`, `superseded_by: []` | **→ AMEND** (add `amended_by:[0513]`; phase-supersession edge) | T-5 |
| **ADR-0374** | `Accepted` | **→ AMEND** (annotate Jenkins-as-orchestrator overtaken by 0511/0513; confirm Forgejo sink governs over 0387) | T-2, T-5 |
| **ADR-0387** | `Proposed` | **→ AMEND or SUPERSEDE** (flip GitHub status-sink → Forgejo to match 0374/0363; drop dead 0112/0359 anchors; reconcile ed25519 vs HMAC) — *conditional on T-4 forge ruling* | T-2, T-4 |
| **ADR-0377-forgejo-board** | `Proposed (conditional)` | **→ RENUMBER** (resolve 0377 collision with kafka-to-pulsar) | T-8 |
| **ADR-0392 / ADR-0408** | `Proposed`, `supersedes:[0358]` | **KEEP** (govern build truth); flag specs/masterplan regeneration | T-7 |
| **ADR-0511** | `Proposed`, `supersedes:[0359]` | **KEEP but CONTESTED vs 0513** — needs reconciliation note (Argo-vs-oya-ci destination) | T-1 |
| **ADR-0513** | `Accepted` (founder-locked) | **KEEP — governs CI destination**; add cross-ref to 0511 | T-1 |
| **ADR-0510** | `Proposed`, `amends:[0363]` | **KEEP** (forge destination); gated behind T-4 founder forge ruling | T-4 |
| **ADR-0173** | `Accepted` | **→ AMEND (vocab/refresh)** — legacy markdown-table header (structurally unbindable to masterplan); "axis-foundry" retired-brand decider; GitHub-Actions/Forgejo Tier-II entries predate 0359/0511/0510 | T-4, retired-vocab |

---

## 12. FOUNDER QUESTIONS (consolidated, theme `ci-cd-forge-build`)

1. **Forge (the load-bearing one).** Canonical forge = **(a)** GitHub `jason931225/oyatie` (your migration directive),
   **(b)** self-hosted Forgejo + GitHub-as-mirror (source canon 0363/0374/0511), or **(c)** Forgejo-transitory → bespoke
   Rust monorepo-VCS (0510)? Every gateway/merge-queue/status-sink ADR resolves off this. *(T-4; gates T-2.)*
2. **CI destination.** Argo Workflows (0511, adopt-CNCF) **or** the bespoke-Rust `oya-ci` Prow platform (0513, founder-locked)?
   The two name different destinations; only one is the masterplan CI node. *(T-1.)*
3. **Masterplan mechanism for build/CI.** Regenerate the CI/build node FROM ADR `supersedes:` edges (auto-drops
   Bazel via 0392/0408 and Jenkins via 0511/0513), or AUTHOR it directly (requires a manual Bazel→Buck2 + Jenkins→
   Argo/oya-ci rewrite of `cloud-toolchain-target.json` + MASTERPLAN.md, both currently stale)? *(T-7.)*

---
*End of CI/CD/forge/build cross-tension register. Surgical-only resolutions; no new policy authored. Forge three-way
(T-4) and CI destination (T-1) are surfaced, not resolved, per binding context.*
