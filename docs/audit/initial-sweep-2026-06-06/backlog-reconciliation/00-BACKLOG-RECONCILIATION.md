# 00 — BACKLOG RECONCILIATION (SSOT)

**Role:** RECONCILER. Two execution-PENDING consensus bodies both mutate the SAME source ADR/spec/masterplan id-space and MUST become ONE before any mutation begins.

- **Body-1 (linux side)** = CANON-CONSOLIDATION `decision-record-oyatie-canon.md` (~35 D-rulings) + `UNIFIED-EXECUTION-PLAN.md` (A1–A6 amendment lanes / L1–L11 migration lanes), execution authority = WIP `monorepo-consolidation-migration.md`.
- **Body-2 (source side)** = PLATFORM-READINESS backlog `/Users/jasonlee/Developer/source/.omx/backlog/platform-readiness-backlog.md` (pillars A–Q + decision register #1–#21 + locked 4-phase program).

**Both are EXECUTION-PENDING.** Neither has begun. Body-1 Wave-0 starts only on commit-signing confirm (`decision-record:80`); Body-2 = `EXECUTION_APPROVAL = PENDING (not executed)` + "recommend ONE consolidated re-verify before execution" (`backlog:593`, `:607`). This artifact IS that consolidated re-verify.

**Critical shared fact:** both bodies write the SAME source `docs/decisions/` namespace (0001–0514, currently 346 files / 345 distinct numbers). Body-1's A5 authors NET-NEW ADRs into the free block >0514; Body-2's register #1–#21 authors/amends the same space. Collisions are real, not theoretical. **No two lanes may edit the same ADR.**

---

## (1) UNIFIED RECONCILIATION REGISTER

Tags: **ALIGN** (same intent, no conflict) · **EXTEND** (one body adds detail/enforcement the other lacks) · **CONTRADICT** (genuine opposing claims → founder ruling required, see §2) · **NET-NEW** (only one body has it; no twin).
Both directions covered. Nothing dropped.

### 1A. linux D-rulings → backlog pillar/register

| linux D-ruling | backlog twin | Tag | Note |
|---|---|---|---|
| **D1** masterplan GENERATED-from-ADRs + drift gate | register **#11** cross-artifact SSOT-agreement gate (amend 0365); pillar **P** | **ALIGN/EXTEND** | Both amend 0365. D1 = "generate masterplan from ADRs"; #11 = "all artifacts agree + auto-gate". One 0365 amend serves both. **De-dup O2.** |
| **D2** Forge GitHub-now→bespoke-later; Forgejo dropped | pillar **D** + register **#16** (CI-ADR consolidation), `confirmations:236` (GitHub-now) | **ALIGN** | Source `confirmations` already say GitHub-now/Forgejo-board only as fallback. CC-5 (Forgejo→GitHub) is the doc fix. |
| **D3** oya-ci unified Run+graph reshape of 0513 | register **#3** (Prow buck2-native + retire `oya` CLI) + **#16** (consolidate 7 CI ADRs→0513) + **#7** (Tide merge-queue) | **CONTRADICT (headline) / ALIGN (intent)** | **Tension T-CI.** D3 headline names a "Tekton spine + Argo DAG engine" as product faces; backlog #16 + the founder's OWN `decision-record:78` ruling = bespoke-Rust-Prow-ONLY, Argo-rejected, no Tekton. |
| **D4 / D-META** own-the-data-tier endpoint; vendor-the-bridge ratchet | register **#1** (D1 trinity intelligence substrate); pillar **K** (contender bar); **#19** (bespoke cloud toolchain) | **EXTEND** | D4 = data-tier ownership ladder; backlog frames it as contender-bar evidence + sequencing. No conflict. |
| **D5** oya-identity owned + Zitadel bridge | (no direct register item) CC-8 identity doc fix | **NET-NEW (linux)** | Backlog silent on identity shape. |
| **D6** Cedar permanent contract, own PARC engine | (no direct register item) CC-2 Cedar→PARC (24 sites) | **NET-NEW (linux)** | Backlog uses Cedar in §J autonomy-tiers but does not rule engine ownership. |
| **D7** framekernel + owned-VMM committed endpoint; Talos/Kata/Firecracker bridges | pillar **K** absent-CDN/HSM gaps; structural ruling "toolchain IS the thing" `:412` | **EXTEND** | Backlog adds the honest-claim evidence bar on the substrate D7 commits to. |
| **D8** oya-ci day-0 crown-jewel, builds parallelize | LOCKED-PROGRAM `buck2-builds-first-party = gating prerequisite` `:459`; register **#3** | **ALIGN** | Both: build capability is the bootstrap gate. |
| **D9** maximal vertical scope (incl net-new defense/power-grid) | pillar **K** per-service compliance; register **#10** honest-claim | **EXTEND** | Backlog adds the per-vertical compliance-in-scope discipline D9 needs. |
| **D10** Argo Rollouts + Chaos Mesh vendored bridges | pillar **N** (Chaos-Mesh) register **#8** | **ALIGN** | Both vendor Chaos-Mesh behind ports. ArgoCD/Rollouts = CD only (see T-CI R3). |
| **D11** full data-integrity sweep (KCMVP restore / tautology / dangling / foundry rename) | register **#12** + **#15** (renumber dup-0377 / 0511→0513 / foundry eradication / status enum / regen indexes) | **ALIGN + DE-DUP** | **Heavy overlap.** Both touch dup-0377, 0511 supersession, foundry, status-enum, ADR-INDEX regen. **De-dup set §3.** |
| **D12** namespace `tier`→{autonomy_tier/…/tenant_class} | register **#12** "3-axis status enum"; CC-11 tenant-tier→tenant-class | **ALIGN/EXTEND** | D12 = vocab namespacing of `tier`; #12 = status-label enum. Adjacent vocab hygiene. Adopt backlog's 3-axis enum (decision/maturity/constraint) into Body-1 rather than invent a parallel. |
| **D13** (amended) full 0000+ renumber DEFERRED; amend-in-place+additive | register #4/#12 operate on stable ids → compatible | **ALIGN** | Both now agree: no renumber; stable id-space. |
| **D14** approve Proposed-ledger (ratify ~122 / drop 3 / amend 0352) | register **#12** FIXES; pillar **B** status-integrity | **ALIGN/EXTEND** | Backlog #12 is a subset (dup-0377, 0511) of the linux ledger batch. |
| **D15** closed 16-domain enum + cohesion gate meta-ADR | register **#4** pure-split cohesion (0131/0512) + **#11** | **EXTEND** | **De-dup O4.** D15 cohesion-gate meta-ADR and #11 SSOT-gate must not become two parallel meta-ADRs. |
| **D16** autonomy-ceiling runtime Cedar gate (governance-owned) | pillar **J** autonomy-tiers→Cedar; register **#9** frontend copilot autonomy | **ALIGN** | Backlog UI autonomy-tiers feed the same D16 Cedar gate. |
| **D-MERGE** WIP = execution authority; amendment-plan folds in | register **#16/#18** (merge-conflict elimination, Tide); pillar **M** parallel-dev | **ALIGN/EXTEND** | Backlog **#18** "impossible merge conflict once PR opened" + pillar **M** (5 pillars) EXTEND the WIP §9 per-lane protocol. |
| **D-INTEL / D-INTEL FINAL** two-layer AI; re-home 96k-LOC engine (DEFERRED) | register **#1** trinity intelligence; §I agent-exec-controller `:683` | **EXTEND / SEE T-AEC** | Engine re-home is a Body-1 §4 DEFERRED campaign; backlog adds the trinity consistency topology (see T-D1). §I is a NET-NEW open door (T-AEC). |
| **D-CONFORM** WIP per-lane acceptance + conformance register | pillar **G** enforceable CI/CD gates; register **#3/#14/#20** automation-ratchet | **ALIGN/EXTEND** | **Strong overlap.** Backlog **#20** automation-ratchet + pillar **G** (buck2-native gates that BLOCK merge) ARE the enforcement layer D-CONFORM's conformance gates need. Merge into one gate program. |
| **D-LAYER** cloud=substrate / oya=products dogfood seam | structural ruling **PURE SPLIT** `:214` (exactly oya/+cloud/) + register **#4** + **#17** dogfood-sequencing | **ALIGN** | Identical: oya products run as tenant on cloud substrate; no oya→cloud dep. |
| **D-EXEC** sweep-then-refound via parallel team; pre-Wave-0 gates | LOCKED-PROGRAM TRIPLE SIGN-OFF→/ralplan `:466`; **#16** | **ALIGN/EXTEND** | Both gate on sign-off; backlog adds the 4-phase program frame (see T-SCOPE). |
| **D-INTEL/D-SEQ/D-LANES** parallel-lanes + infra-sovereignty ordered | register **#16/#17** dogfood-need sequencing over 0509/0510; pillar **M** | **EXTEND** | **De-dup O3.** D-SEQ infra-ratchet and #16/#17 dogfood-sequencing both order substrate build-out. |
| **D-SAFETY** one governance-owned safety-gate invariant ADR | (no register twin) | **NET-NEW (linux)** | Backlog has no cross-vertical safety invariant. A5-unique. |
| **D-KR** global-canonical core + KR localization packs | register **#5** packs classify (preserve 0064/0010); structural ruling pack trichotomy `:412/:451` | **ALIGN** | Both: canonical-base / shared pack lib / per-module overlay; cross-pack imports forbidden. |
| **D-DEPTH** owed-depth list bound to per-vertical M0 | (no register twin) pillar **K** compliance gaps adjacent | **NET-NEW (linux)** | Backlog tracks compliance gaps but not the owed-depth ledger mechanism. |
| **D-RECOVER / D-LINEAGE / .Trash recovery** bominal restoration | (no register twin) | **NET-NEW (linux)** | Backlog has no bominal-lineage recovery track. |
| **D-EVENT** Pulsar canonical eventing bridge | register **#12** (Kafka→Pulsar implied) CC-6 data-bridges+Kafka→Pulsar; dup-0377 kafka-to-pulsar (Accepted) | **ALIGN** | ADR-0377-kafka-to-pulsar already Accepted on disk. Doc fix CC-6. |
| **D3 build-engine / WF2 sweep CC-1..13** | pillar **B** doc integrity; register **#12** | **ALIGN** | CC doc fixes land in A6; backlog #12 references same. |

### 1B. backlog pillars/register → linux (reverse direction; catch backlog-only items)

| backlog item | linux twin | Tag | Note |
|---|---|---|---|
| **A** architecture-gaps: A1 module-integration manifest, A2a EntityMutated schema, A2b workflow→AI gRPC .proto, A3 effective-dating-kernel | register **#1/#2** | **NET-NEW (backlog)** | No linux D-ruling covers EntityMutated/gRPC-proto/effective-dating. **In-scope question = T-SCOPE.** |
| **A2 (resolved)** federated peer substrates (ADR-0059) | D-LAYER dogfood seam | **ALIGN** | Both reject direct cross-substrate calls. |
| **A4** Power-of-One upgrade-safety + expand/contract CI | D-CONFORM gates | **EXTEND (backlog)** | Adds an upgrade-safety invariant linux lacks. |
| **B / B-P0-*** status-integrity, false-green, dup-0377, foundry-incomplete | D11/D14, A2 integrity lane | **ALIGN + DE-DUP** | Same mechanical fixes. §3. |
| **C** repo-shape sprawl (services husks, 385 underscore crates BNF) | structure-verify + D-CONFORM BNF gate (0056/0105) | **ALIGN/EXTEND** | Sprawl eradication = `20-verify-structure.md` action (a). |
| **E** workflow-engine durable-execution (Temporal model) + per-lang SDKs | (no linux twin) | **NET-NEW (backlog)** | Engine-design decision; in-scope = T-SCOPE. |
| **F** ontology+workflow+AI co-location synergy | D-INTEL two-layer AI (adjacent) | **EXTEND (backlog)** | Co-location best-practice; feeds T-D1. |
| **G** enforceable CI/CD gates (buck2-native, BLOCK merge) | D-CONFORM conformance gates | **ALIGN/EXTEND** | The enforcement substrate for D-CONFORM. |
| **H/I/J/L** frontend WASM-preload / SSR-islands / UI-UX / multi-platform client | (no linux twin) register #6/#9 | **NET-NEW (backlog)** | Whole frontend/client program absent from linux side. T-SCOPE. |
| **K** honest AWS/Google-contender bar + per-service compliance | D9 vertical scope (adjacent) | **EXTEND (backlog)** | Adds the evidence/claim discipline. Ties register #10/#21. |
| **M** extreme-parallel-dev safe coordination (5 pillars) | D-MERGE/D-LANES + WIP §9 | **ALIGN/EXTEND** | Backlog 5-pillar model is more concrete than D-LANES. Fold into WIP §9. |
| **N/O** verification-resilience bar + testing-taxonomy (all enforced) | D-CONFORM + verify-at-each-step founder rule | **EXTEND (backlog)** | Adds production-readiness-evidence.json + SLO-gated promotion (0130/0139). |
| **P** cross-artifact SSOT agreement | D1 generated-masterplan | **ALIGN** | De-dup O2 (0365). |
| **Q** pure-Rust tooling + language-discipline gate | D-CONFORM (Rust guardrails) register #14 | **EXTEND (backlog)** | Adds the no-new-.sh/.py gate. |
| register **#19** bespoke cloud toolchain services + pipeline isolation | D2/D3 forge+CI | **EXTEND (backlog)** | cloud-scm/cloud-ci/cloud-cd as tenant-facing services + bridge deletion criteria. |
| register **#20** automation-ratchet (forbid new `oya` CLI commands) | D-CONFORM | **ALIGN/EXTEND** | Reinforces buck2-native-only enforcement. |
| register **#21** claim-ceiling / production-readiness-claim contract | verify-at-each-step founder rule; D-EXEC honest-claim | **EXTEND (backlog)** | Regulated-claim contract (no "done/complete/secure/parity" without evidence). Strong fit with founder's evidence-based rule. |
| §I **agent-execution-controller** (capture-only, decision-pending) | D-INTEL adjacent (NOT folded) | **CONTRADICT-RISK → T-AEC** | Revive-vs-decline open founder gate vs ADR-0116/0363 (both Accepted, "deliberately killed adjacent layer"). |

---

## (2) TENSIONS FOR FOUNDER (ruling required)

Each: both positions verbatim · verified live evidence · recommended resolution. **Do not begin any mutation until these are ruled.**

### T-CI — CI product shape: D3 "Prow+Tekton+Argo four faces" vs bespoke-Prow-only / Argo-rejected
- **Position A (linux D3 headline, verbatim, `decision-record:157`):** "oya-ci = ONE Rust-native CI/CD system = two nouns (`Run` + Buck2 build-graph over a CAS core) · **four faces (Prow trustless gate+merge-queue · Tekton typed-task/provenance spine · Argo DAG engine · provenance)**". Task #17 title encodes the same "(Prow+Tekton+Argo)".
- **Position B (source register #16 `backlog:585` + §G `:202` + `confirmations:236`):** "CI = BESPOKE PROW IN RUST (0513 operative, 0511 superseded, `oya` CLI retired from CI authority)"; "consolidate 0349/0359/0361/0408/0511/0513/0514 → ONE canonical CI ADR (destination 0513)"; "NOT Argo Workflows (ADR-0511 overtaken)."
- **Position C — the founder's OWN later linux ruling (`decision-record:78`, 2026-06-06, verbatim):** "**0511 Argo (conflicts with the D3 oya-ci ruling; its ideas already absorbed); only oya-ci 0513 is ratified canon**". So Body-1's founder has ALREADY converged on bespoke-only; the D3 headline + task-#17 title are **stale residue from the same body**.
- **Verified live evidence:** `ADR-0513` = `status: Accepted`, the only Accepted CI node, bespoke-Rust-Prow component shape, **NO `supersedes`/`superseded_by` keys** (only `relates:` — and 0511 is NOT in its relates list). `ADR-0511` = `status: Proposed`, `superseded_by: []` (the intended 0511→0513 link is missing on BOTH ends). `0349`=Proposed, `0359`=Superseded(by 0511), `0361`=Proposed, `0408`=Proposed, `0514`=Proposed. **Tekton has ZERO Accepted ADR support** and is rejected wherever named (per `20-verify-ci.md`).
- **RECOMMENDED RESOLUTION:** Rule **bespoke-Rust-Prow-only** (Positions B + C; A is stale). (R1) Re-label the D3 headline from "four faces (Prow+Tekton+Argo)" to "bespoke-Rust Prow full-component shape over Buck2-graph+CAS; DAG-orchestration / typed-task / provenance are **oya-ci-native capabilities inspired by Argo/Tekton IDEAS, not product adoptions**." (R2) **Drop "Tekton" as a named face entirely.** (R3) Confine Argo to **CD only** (ArgoCD/Argo-Rollouts may remain as vendored bridges per D10; Argo-Workflows rejected for CI). (R4) ONE canonical CI ADR = 0513; 0349/0361 DROP, 0359 already-Superseded, **0511 mark Superseded→0513**, 0408/0514 **amend-in-place** (adopted substrate, per `:78`). (R5) Fix supersession metadata both ends: 0511 → `status: Superseded` / `superseded_by: [ADR-0513]`; add reciprocal `supersedes: [ADR-0511]` to 0513. (R6) Update `source/docs/AGENTS.md` (still Jenkins-sole) to oya-ci destination; Jenkins+GH-Actions scaffold = explicitly-UNRATIFIED bridge, build-first-cutover-later. **Also fix linux task-#17 title.**

### T-SCOPE — Is the backlog 4-phase platform-readiness program IN-SCOPE for THIS consolidation, a sequenced follow-on, or a peer that must merge into the masterplan NOW?
- **Position A (Body-1 framing):** the consolidation is the sweep-then-refound of the ADR/spec/masterplan canon + the WIP sibling monorepo migration (L1–L11). The backlog's program (frontend H/I/J, client L, workflow-engine E, verification N/O, the 4 phases) is NOT in the migration queue.
- **Position B (Body-2, `backlog:459` LOCKED PROGRAM):** "full 4-phase program; runway 24+ months; substrate completion bar = deployed+measured-SLO+DOGFOODED-BY-VERTICAL; enforcement = BUCK2-NATIVE ONLY; TRIPLE SIGN-OFF → /ralplan." This is a standing, signed-off program, not a doc artifact.
- **Verified live evidence:** `backlog:593/607` = `EXECUTION_APPROVAL = PENDING (not executed)` + "recommend ONE consolidated re-verify before execution." `backlog:466` triple-sign-off → "proceed to /ralplan" recorded but plan not started. So the program is **approved-in-principle, not begun** — same execution-pending state as Body-1.
- **RECOMMENDED RESOLUTION:** Three-bucket split, ruled now:
  1. **MERGE-NOW into the one amendment campaign** (mechanical, same id-space, can't be deferred without dual-canon): all register **FIX-class** items (#12 dup-0377, #14 0511→0513, #15 foundry/status-enum/index-regen) + pillar **B** integrity + pillar **G/Q/#20** enforcement gates (they ARE the D-CONFORM enforcement layer) + structural **PURE-SPLIT** sprawl eradication (`20-verify-structure.md` action a) + **#16/#17** CI/dogfood sequencing. These fold into Body-1's A1–A6 + conformance register.
  2. **SEQUENCED FOLLOW-ON ADR program (author the ADRs now into the free block, build later):** register **#1 D1-trinity, #2 effective-dating, #6 multi-platform-client, #9 frontend, #10 honest-claim, #19 bespoke-cloud-toolchain, #21 claim-ceiling** + pillars **E/F/H/I/J/K/L/N/O**. These are NET-NEW decisions, NOT mechanical canon fixes; they belong in the masterplan as a Phase-0..3 program but their CODE does not block the sibling migration.
  3. **PEER-INTO-MASTERPLAN NOW:** register **#13** (masterplan+roadmap add the platform-readiness program Phases 0–3 + pillars A–P) — this is the wiring that makes (2) reachable. Land it as the masterplan-generated-wiring meta-ADR (D1 / O2). **Founder must confirm this bucketing.**

### T-D1 — Consistency topology (net-new, defining call)
- **Position (Body-2 `backlog:404` D1 RECOMMENDATION, verbatim intent):** "ontology = SINGLE transactional system-of-record, ONE write path = ontology typed-actions; workflow = federated durable-execution calling ontology typed-actions IN-TXN + consistency token (own DB = orchestration only); intelligence = federated stateless; async Kafka outbox REMOVED from critical consistency path → SAP anti-pattern eliminated." Acceptance = read-your-writes conformance test on payroll close (`:459`).
- **linux side:** NO D-ruling defines this. D-INTEL covers AI-engine homing, not the write-path/consistency topology.
- **Verified live evidence:** register **#1** = NET-NEW (no matching ADR file on disk, per `20-verify-register-coverage.md`); A2a EntityMutated + A2b workflow→AI gRPC .proto both **absent** from the 1797-line kernel (backlog L344/L175). This is a genuine architecture decision with no canon yet.
- **RECOMMENDED RESOLUTION:** Founder must RULE the D1 topology (single-write-path ontology vs the current Kafka-outbox). Author as ONE net-new ADR-0515+ "consistency-domain / federated-execution" with the read-your-writes payroll-close conformance test as its acceptance gate. **Resolve the D1/D01/D15 naming clash** (linux D-rulings, backlog register-#1 "D1", and linux meta-ADR all collide on "D1"). Recommend: rename backlog register-#1 to a unique ADR id; keep linux D-numbers as decision-record-internal only.

### T-AEC — §I Agent-Execution-Controller: revive-as-narrower vs decline (ADR-0116/0363 tension)
- **Position (Body-2 §I `backlog:683`):** capture-only/decision-pending. Agent execution as ephemeral policy-gated K8s-Job; 3 contracts (work-item.v1 / pod-schedule-plan.v1 / evidence-bundle.v1). "DECISION GATE before any code: ADR-0116 + ADR-0363 deliberately killed adjacent layer — may be missing on purpose; FOUNDER MUST RULE revive-as-narrower-net-new vs decline."
- **Verified live evidence:** `ADR-0116` = `status: accepted` (retire external agent-coordination tooling); `ADR-0363` = `status: Accepted`, `amends: [ADR-0116]` (both "deliberately killed the adjacent coordination layer"). `ADR-0247` (self-mod ceiling) = `status: Proposed`. Per `20-verify-foundry-hygiene.md`: reviving §I does NOT contradict 0116/0363 IF scoped as the explicitly-narrower net-new EXECUTION contract (new flat single-concern service under oya/ or cloud/, NOT folded into cloud-intelligence); it WOULD contradict if revived broadly.
- **RECOMMENDED RESOLUTION:** This is a genuine OPEN DOOR, not a de-dup item. Founder rules one of: **(a) REVIVE narrow** — new flat single-concern service (ADR-0131/0132 shape), author net-new ADR, audit `oya-code` crates/+examples/work-items/ for salvage, promote per `.omx` SoT; or **(b) DECLINE** — record the decline, let the `oya-code` harness lapse. Source distillation `oya-code/docs/source-distillation-cloud-intelligence.md` is now-superseded; everything else in it is already owned by source — NOT worth porting either way.

### T-STRUCT — Pure-split structure vs linux migration homes (minor; compatible-but-incomplete)
- **Position (Body-2 structural ruling `:214`):** canonical top-level = exactly `oya/` (products, runs as tenant) + `cloud/` (platform); everything else = SPRAWL to consolidate; amend 0131/0512; remove legacy `microservices/` after verified-empty.
- **Position (Body-1 migration plan §2):** canonical homes "only `{oya,cloud}/<service>/crates/<crate>` and `libs/<lib>/`" — identical, no third tree.
- **Verified live evidence:** `0131` + `0512` BOTH `status: Accepted` and BOTH already carry the 2026-06-02 pure-split amendment (`0512 amends: [ADR-0131]`). Tree inventory (verified live): `oya/`=87, `cloud/`=25, `services/`=5 **husks** (no real crates), `platforms/`=0 **husk**, `microservices/` **ABSENT at top level**, flat `crates/`=2 (ADR-0512 declares forbidden), `libs/`=168. **No live ADR conflict.**
- **RECOMMENDED RESOLUTION:** **NOT a contradiction — COMPATIBLE but INCOMPLETE.** No re-decision needed. Add three enforcement actions to the unified plan (none are re-decisions): (a) eradicate residual sprawl — delete `services/` (5 husks) + `platforms/` (husk), git-mv flat `crates/` 2 crates into `{oya,cloud}/<svc>/crates/`, gated by 0512 workspace-topology check; (b) repair `source/docs/AGENTS.md §Repository topology` (line ~256) which lists retired `platform/` (singular) and OMITS the real `platforms/`+`microservices/` names; (c) the k8s-merge lane (L6) must verify/finish `cloud/cloud-k8s/` which has no `crates/` subdir yet — do not assume it.

---

## (3) DE-DUP SET — ADRs/files BOTH bodies touch → ONE amendment per target

**Ruling principle (from `20-verify-foundry-hygiene.md`):** **Body-1 (AMENDMENT-PLAN, the execution authority) OWNS the mechanical actions.** Body-2 backlog is a PLANNING INPUT (`.omx` SoT) whose register-#12/#15 hygiene lines **reference, not re-execute**, those fixes. **Never run two sweeps over the same 831-file / 346-ADR id-space.**

| Shared target | Body-1 owner | Body-2 reference | ONE merged action |
|---|---|---|---|
| **dup ADR-0377** (kafka-to-pulsar `Accepted` + forgejo-board `Proposed-conditional`, both live) | A-lane L1.0 renumber-MAP (D11/D14) | register #12/#13 | **Renumber the Proposed forgejo-board-CAS file** to next free id; keep kafka-to-pulsar at 0377. One edit. |
| **ADR-0511 supersession** | CC-4 / A4 (D3 cluster, `:78`) | register #12/#14 | 0511 → `status: Superseded` + `superseded_by: [ADR-0513]`; add `supersedes: [ADR-0511]` to 0513. One reciprocal edit (see T-CI R5). |
| **foundry retirement completion** (3,771 residue files / 201 oya-foundry-named live; 0363:35 falsely claims "eradicated") | A1 per-file 4-way sense-route (D-INTEL/D11(d)/CC-1) | register #12/#15 + B-P0-4 | **Body-1 A1 sense-routed rename owns it** (platform→oya-intelligence current-home, fitness→oya-governance, vcs→retired, HARD carve-out Palantir-43/Marlboro-Forge). Fix 0363:35 false-"eradicated" claim. Re-point backlog #12 AT A1/CC-1 — do not run a second blind "foundry→intelligence" sweep (it would re-commit 0363's false-green and mis-route ~135 governance-sense files). |
| **3-axis status enum** | (none — adopt backlog's) | register #12 / Architect-D5 (decision/maturity/constraint) | **ADOPT the backlog's 3-axis enum into Body-1** rather than invent a parallel. Reconcile with D12 `tier`-namespacing in the same vocab-hygiene lane (A3). |
| **ADR-INDEX.md / decisions.json regen** | Wave-3 regenerate-after-renumber (D1 generated-from-source) | register #12/#15 | ONE regen pass AFTER renumber lands. Index is currently generated-from-stale-source (DRIFT, see §5) — itself register FIX #15. |
| **0131 / 0512** pure-split amendment | (already Accepted + amended on disk) | register #4 | **NO edit** — both already Accepted + pure-split-amended. Backlog #4's amendment is already authored. Only add sprawl-eradication enforcement (T-STRUCT). Remove stale duplicate 0131/0512 copies under `source/.claude/worktrees/**`. |
| **0365 cross-artifact gate** | D1 masterplan-generated-wiring meta-ADR | register #11 / pillar P | **ONE 0365 amend** serves D1 + #11 (de-dup O2). Do not author two propagation meta-ADRs. |
| **0513 oya-ci** | D3 reshape / A5 | register #3/#16/#7 | **ONE 0513 amend** (de-dup O1) — Tide merge-queue (#7/#18), buck2-native enforcement (#3), CI-ADR consolidation destination (#16) all land in the single 0513 reshape. |
| **domain-cohesion vs SSOT-gate meta-ADRs** | D15 cohesion-gate meta-ADR | register #11 / #4 | **Keep distinct single-concern** (founder ruled TWO meta-ADRs at `:78`: domain-cohesion + masterplan-generated-wiring) but de-dupe naming with register #11 so they are not three overlapping meta-ADRs (de-dup O2+O4). |

---

## (4) RECOMMENDED UNIFIED PLAN (how the two bodies become ONE)

**One decision canon — direction:** The source platform-readiness **backlog register FOLDS INTO the linux decision-record / UNIFIED-EXECUTION-PLAN as the source-side input that is now reconciled** — NOT vice-versa. Rationale: (i) Body-1 is the body with the founder-signed one-way consolidation-design-set (`:78`, A.0-2 FROZEN) and the WIP execution authority; (ii) the backlog is explicitly an `.omx` SoT **planning input** until promoted (`backlog:248`), and is itself execution-pending awaiting "ONE consolidated re-verify" (`:607`) — this artifact. So: backlog FIX-class + enforcement items merge into Body-1's A1–A6 + conformance register; backlog NET-NEW ADR program becomes a sequenced Phase-0..3 ADR-authoring track wired into the masterplan via the D1 meta-ADR.

**One amendment campaign (single pass per ADR target):**
- **A1** foundry per-file 4-way sense-route — also absorbs backlog #12/#15/B-P0-4 foundry-completion (re-point backlog at A1; one sweep only). Fix 0363:35 false-"eradicated".
- **A2** integrity sweep (KCMVP/KISA restore, tautology 0006, ALL dangling edges) — absorbs backlog B integrity + the dup-0377 renumber + 0511→0513 supersession + ADR-INDEX/decisions.json regen (#12/#14/#15). Adopt backlog 3-axis status enum.
- **A3** vocab namespacing (`tier`→…/tenant_class; 0163 tiers→stages) — reconciled with the 3-axis status enum.
- **A4** Proposed-ledger resolution + CI-cluster drop (0349/0361 drop, 0359 already-superseded, 0511→0513, 0408/0514 amend-in-place) — implements T-CI R4/R5.
- **A5** NET-NEW/reshaped ADRs into free block >0514: oya-ci 0513 reshape (T-CI: bespoke-Prow-only, de-dup O1 with #3/#7/#16/#18) · safety-gate (D-SAFETY, A5-unique) · KR EmploymentClassification enum (D-KR) · infra-sovereignty ordered+M0 (D-SEQ, de-dup O3 with #16/#17) · domain-cohesion meta-ADR (D15) · masterplan-generated-wiring meta-ADR (D1, de-dup O2 with #11/#13) · data-engine-endpoint ADR (D-INTEL/D4) **PLUS the backlog NET-NEW program ADRs** (T-SCOPE bucket 2): #1 D1-trinity/consistency-topology (T-D1) · #2 effective-dating-kernel · #6 multi-platform-client · #9 frontend(Leptos/SSR-islands/WASM-preload/UI-UX) · #10 honest-claim/§K · #19 bespoke-cloud-toolchain-services · #20 automation-ratchet · #21 claim-ceiling/§Q · pillar-E workflow-engine · §I agent-exec-controller ONLY if T-AEC ruled REVIVE.
- **A6** CC-1..CC-13 doc fixes (incl. CC-6 Kafka→Pulsar/D-EVENT, CC-4 Jenkins→oya-ci, CC-5 Forgejo→GitHub) + source/docs/AGENTS.md repository-topology repair (T-STRUCT b) + AGENTS.md CI-destination (T-CI R6).
- **NEW conformance/enforcement lane** (fold pillar G + Q + register #20 + N/O into D-CONFORM's per-lane gates): buck2-native gates that BLOCK merge, language-discipline gate (no new .sh/.py), production-readiness-evidence.json + SLO-gated promotion, claim-ceiling contract (#21). Loop UNCHANGED, additive to WIP STEP 7.
- **Sprawl-eradication action** (T-STRUCT a): delete `services/`+`platforms/` husks, empty flat `crates/`, gated by 0512 topology check.

**Re-sequenced consolidation order (both execution-pending → unify, THEN start):**
1. **GATE-0 (before any mutation):** Founder rules T-CI, T-SCOPE, T-D1, T-AEC, T-STRUCT (bucketing). Confirm commit-signing (last Wave-0 gate, `:80`). Kernel re-verify (check-tcb/diff-oracle/both-build/assert-talos) + user gates G0–G4.
2. **GATE-BEFORE-START** pre-lanes: 0.4✓/0.5✓/0.7✓ done; 0.6 no_std-inertness over the **full 12-entry kernel exclude** (`:107`) at consolidation-time.
3. **Amendment campaign A1–A6 + conformance lane + sprawl-eradication** — onto the cleaned, stable-id canon, ONE pass per ADR (de-dup set §3).
4. **Migration lanes L1–L11** (WIP §6) onto cleaned canon: L1 office · L2 oyago · L3 oyapy · L4 claude-SDK · L5 codex-SDK(new sibling) · L6 k8s (95→oya-cloud-k8s-*; finish `cloud/cloud-k8s/crates/` per T-STRUCT c) · L7 containerd(44→container-runtime) · L8 DROPPED(no source) · L9 node-os · L10 docs(+13 pilot ADRs) · L11 framekernel(no_std, LAST).
5. **Sequenced follow-on** (T-SCOPE bucket 2): build the Phase-0..3 platform-readiness program against the now-authored ADRs; masterplan wired via D1 meta-ADR (#13).

**NET-NEW ADR authoring vs amend-in-place — flagged:**
- **AMEND-IN-PLACE (target exists):** 0513, 0511, 0359, 0349, 0361, 0408, 0514, 0131, 0512, 0064, 0010, 0365, 0163, 0352, 0006, 0377(both), 0363, 0116(via 0363), the ~122 ratified-ledger ADRs.
- **NET-NEW (author from scratch, free block >0514):** D-SAFETY safety-gate · D-KR EmploymentClassification enum · D-SEQ infra-sovereignty · domain-cohesion meta-ADR · masterplan-generated-wiring meta-ADR · data-engine-endpoint · register #1 consistency-topology · #2 effective-dating · #6 multi-platform-client · #10 honest-claim · #19 bespoke-cloud-toolchain · #20 automation-ratchet · #21 claim-ceiling · pillar-E workflow-engine · §I agent-exec-controller (conditional on T-AEC).
- **FIX-class (mechanical, no new decision):** dup-0377 renumber · 0511→0513 supersession · foundry-eradication completion · 3-axis status enum · ADR-INDEX/decisions.json regen.

---

## (5) VERIFIED-STATE APPENDIX (live facts this artifact rests on; read from disk 2026-06-06)

ADR statuses (`source/docs/decisions/` frontmatter, verbatim):
- **ADR-0513** oya-ci bespoke-Rust-Prow = `status: Accepted`; **NO `supersedes`/`superseded_by` keys** (only `relates: [0380,0111,0116,0374,0363,0392,…]` — **0511 NOT in relates**). Only Accepted CI node.
- **ADR-0511** Argo-Workflows-supersede-Jenkins = `status: Proposed`, `supersedes: [ADR-0359]`, `superseded_by: []` (EMPTY — 0511→0513 link missing both ends), `door: two-way`.
- **ADR-0514** target-arch/hyperscaler-remediation = `status: Proposed`, `supersedes: []`, `superseded_by: []`, `related: [ADR-0513,…]`.
- **ADR-0349** Jenkins+ArgoCD = `Proposed`. **ADR-0359** Jenkins-replaces-GHA = `Superseded` (`superseded_by: [ADR-0511]`). **ADR-0361** Jenkins-native = `Proposed`. **ADR-0408** Buck2-driven-CI = `Proposed` (`supersedes: [ADR-0358]`).
- **ADR-0131** per-microservice-flat-layout = `Accepted`. **ADR-0512** canonical-monorepo-pattern = `Accepted`, `amends: [ADR-0131]`. Both already carry the 2026-06-02 pure-split amendment.
- **ADR-0116** retire-external-agent-coordination = `accepted`. **ADR-0363** retire-agentic-vcs-foundry = `Accepted`, `amends: [ADR-0116]`, `supersedes: [0110,0112,0113]`.
- **ADR-0247** self-hosting/self-modification doctrine = `Proposed`.
- **ADR-0377 DUPLICATE CONFIRMED:** `ADR-0377-kafka-to-pulsar-via-kop.md` = `Accepted` (`supersedes: [ADR-0005]`) **and** `ADR-0377-forgejo-board-git-ref-cas-fallback.md` = `Proposed (conditional)`. Two files, one number.
- **ADR-0365** automated-ADR-lifecycle-and-propagation = `Accepted`.

Tree inventory (`/Users/jasonlee/Developer/source`, live subdir counts):
- `oya/`=**87** (canonical products) · `cloud/`=**25** (canonical platform) · `services/`=**5 husks** · `platforms/`=**0 (husk)** · `microservices/`=**ABSENT at top level** · `packs/`=9 · `regional-packs/`=5 · `libs/`=168 · flat `crates/`=**2** (ADR-0512-forbidden, still present).

ADR id-space:
- **346 ADR files / 345 distinct numbers** (delta = dup-0377).
- Frontmatter status scan: **accepted-family 174 · proposed-family 150 · superseded 15** (plus a handful of non-canonical status strings — itself register-#12 status-enum debt).
- **DRIFT FLAG:** the generated `ADR-INDEX.md` counts do not match the live scan → index is generated-from-stale-source = register FIX #15. Trust the scan.

Foundry residue (canonical tree, excl. target/buck-out/_upstream/vendor/.claude-worktrees/.git/_legacy-foundry):
- **3,771 files contain "foundry"; 201 `oya-foundry-*`-named files** still present.
- `ADR-0363:35` (verbatim): "**The Foundry name was eradicated** … the former `oya-foundry-*` crates were renamed across three namespaces." **FALSE-GREEN CONFIRMED** — the eradicated claim is empirically disproven by 3,771 live residue files.

Execution state (both bodies):
- Body-2 backlog: `EXECUTION_APPROVAL = PENDING (not executed)` (`:593`); "recommend ONE consolidated re-verify before execution" (`:607`); triple-sign-off→"/ralplan" recorded (`:466`) but plan not started.
- Body-1: Wave-0 starts on commit-signing confirm (`decision-record:80`); A.0-2 consolidation-design-set FROZEN door:one-way founder-signed (`:78`).
- **Both EXECUTION-PENDING — neither has begun. Unify (rule §2 tensions, apply §3 de-dup, adopt §4 plan) BEFORE any mutation.**
