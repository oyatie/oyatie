# 20 — VERIFY: foundry routing + ADR-hygiene overlaps (backlog ↔ consolidation decision-record)

**Lane:** VERIFY foundry routing + ADR-hygiene overlaps between the two reconciliation bodies.
**Mode:** READ-ONLY. No source or audit file edited; only this artifact written.
**Date:** 2026-06-06.

**The two "bodies" being reconciled (Task #21):**
- **Body-1 = consolidation decision-record (audit side).** `synthesis/decision-record-oyatie-canon.md` (D-INTEL, D11(d), CC-1) + `AMENDMENT-PLAN.md` (lane L2 foundry rename, L3 integrity). All under `/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/`.
- **Body-2 = source platform-readiness backlog.** `/Users/jasonlee/Developer/source/.omx/backlog/platform-readiness-backlog.md` (690 lines). Extracted verbatim by the sibling lane artifact `backlog-reconciliation/10-extract-backlog.md`. Relevant items: **B2** (L76), **B-P0-4** (L138), **register #12** (L539), **§I agent-execution-controller** (L683).

**Citation discipline:** every load-bearing claim re-checked against real files (ADR front-matter, foundry residue grep, the two foundry artifacts, the idea doc). Path+line given.

---

## (a) FOUNDRY ROUTING — do the two framings AGREE or DIFFER?

### The on-disk ADR facts (canonical `source/docs/decisions/`, verified)

| ADR | file | `status:` (front-matter) | what it rules |
|---|---|---|---|
| ADR-0335 | `ADR-0335-foundry-microservice-retired-absorbed-by-intelligence.md` | **Accepted** (L3; body §Status L62; `completed-locally` L771) | foundry-µsvc retired, brand RETIRED, absorbed by intelligence; amends 0136/0138/0220/0239/0247/0255 |
| ADR-0347 | `ADR-0347-foundry-fitness-to-governance-bulk-rename.md` | **Proposed** (L4) | `oya-foundry-fitness-*` → `oya-governance-*`; `enforcement_status: advisory-until-wave-15-zb-bulk-rename-pr-lands` (L87) |
| ADR-0363 | `ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate.md` | **Accepted** (L3) | retire agentic-VCS foundry → git+Forgejo+oya-ci; supersedes 0110/0112/0113; amends 0116 |
| ADR-0247 | `ADR-0247-self-hosting-self-modification-doctrine.md` | **Proposed** (L3); `superseded_by: []` (L27) | self-hosting / self-modification doctrine + autonomy ceiling |

### The two routing framings

**Body-2 (backlog) framing — "foundry→intelligence eradication" (a 2-way collapse):**
- B-P0-4 (L138): "foundry retirement incomplete despite ADR-0335."
- register #12 (L539): "complete foundry->intelligence **eradication**."
- B2 (L76): "rename foundry-* crates, superseded-reference lint."
- Net: the backlog frames it as a single-target sweep to "intelligence" + a binary "eradicate" goal. It treats ADR-0335 as the governing fact and the residue as merely "incomplete."

**Body-1 (decision-record) framing — per-file SENSE-ROUTED, 3-to-4-way, never a blind swap:**
- D-INTEL (`decision-record:86`, founder-RULED): foundry **framework/platform → cloud-intelligence**; **agentic-workflow uses → oya-intelligence**; **foundry-fitness → oya-governance**; **agentic-VCS → RETIRED (0363)**. Explicitly: *"NOT a single 'intelligence' target."*
- Wave-0 CORRECTION (`decision-record:99`): the **Wave-0 831-file rename targets CURRENT reality** — platform → **oya-intelligence** (the 96k-LOC engine + 712 lineage files physically live there today), fitness → **oya-governance**, vcs → **retired**. The engine re-home oya-intelligence→cloud-intelligence is a SEPARATE later migration; doing it in Wave-0 would drift docs ahead of code. So `foundry→cloud-intelligence` is the ENDPOINT; the Wave-0 routing uses the present home.
- CC-1 (`decision-record:150`): "the 800-file foundry rename MUST be per-file sense-routed (intelligence vs governance) with HARD carve-outs (Palantir Foundry, Marlboro-Forge/forgery, the foundry-fitness→governance retirement record) — **never a blind swap**."
- AMENDMENT-PLAN L2.0 (`:136`) freezes the concrete routing rule: `oya-foundry-* / axis-foundry / ai-foundry / agent-runtime / provider / capability / RAG / model / sandbox / supervisor` → **cloud-intelligence/intelligence** (~274 files); `foundry-fitness / council-foundry / governance-foundry / amendment-foundry / Proof-Ladder / fitness-lane / CI-gate` → **governance** (~135 files); remainder = carve-outs/FP/journeys.

### VERDICT: the two framings DIFFER in shape but AGREE on substance — Body-1 is the SUPERSET and the safe one; Body-2's framing is DANGEROUSLY UNDER-SPECIFIED.

- **They are NOT in contradiction on the goal** (foundry brand must go; ADR-0335 governs). The Task's own one-line framing — "platform→oya-intelligence, fitness→oya-governance, vcs→retired" — matches Body-1's **Wave-0** routing exactly. So at the Wave-0 level the Task framing IS the decision-record's framing.
- **They DIFFER on three axes that matter for execution:**
  1. **Target cardinality.** Backlog says "→intelligence" (1 target). Decision-record says 3-to-4 sense-routed targets (cloud-intelligence/oya-intelligence + governance + retired), with the cloud-intelligence vs oya-intelligence split deferred to a later re-home. A literal reading of register #12's "foundry→intelligence eradication" would mis-route the ~135 governance-sense `foundry-fitness` files into intelligence — the exact defect CC-1 + AMENDMENT-PLAN §Failure-1 (`:281-282`) exist to prevent.
  2. **"Eradication" vs "sense-routed rename with carve-outs."** The backlog word "eradicate" is the same false-binary that produced ADR-0363's false claim (see §a-false-green below). A blind `s/foundry/intelligence/` corrupts 43 Palantir-Foundry third-party refs, the Marlboro-Forge fiction, and the retirement record itself.
  3. **Endpoint vs current-home.** Backlog has no notion that "intelligence" is itself moving (oya→cloud); decision-record explicitly times Wave-0 to current reality and defers the cloud re-home (`decision-record:90,99`).
- **Reconciliation:** Body-2 register #12 should be amended to point at Body-1's CC-1 / L2.0 routing rule (sense-routed, carve-outs, Wave-0=current-home) rather than its own "→intelligence eradication" phrasing. They are the same decision; the backlog states it imprecisely.

### False-green CONFIRMED (the thing both bodies flag)

- **ADR-0363 false "eradicated" claim — verbatim, line 35:** *"The Foundry name was eradicated (ADR-0362 + the #181–#184 cutover)…"* — yet the live residue (below) disproves it. AMENDMENT-PLAN L3.4 (`:147`) + L2 gate (`:326`) explicitly schedule "fix 0363's false 'eradicated' claim." Decision-record D11(d) (`:56`) names it.
- **`docs/prds/foundry.md` vs `specs/microservices/foundry.json` divergence — CONFIRMED false-green:**
  - `docs/prds/foundry.md:5` → `status: Accepted`, `doc_status: published` (L17). Still presents as a live, accepted PRD.
  - `specs/microservices/foundry.json` → `"title": "Microservice:Foundry — RETIRED"` (L4), `"spec_id": "MSC-FOUNDRY-RETIRED"` (L7), `"status": "Retired"` (L9), `"retired_at": "2026-05-21"`, `"retired_by_adr": "ADR-0335"` (L12).
  - Two artifacts for the SAME retired service disagree (PRD=Accepted, JSON=Retired). This is the backlog's B-P1-3 "PRD-md vs JSON-spec divergence" (L25) and is a concrete instance of B-P0-4 "foundry retirement incomplete."
- **Cedar `oyatie.foundry.*` principals — CONFIRMED present in live spec/policy substrate (ADR-0247-bootstrap-era namespace):** retired-brand reserved sub-scopes persist in `specs/root-hub-pointers.json`, `specs/tenant-model.json`, `specs/platform-architecture.json`, `specs/master-plan-sequencing.json`, `tools/hooks/_canonical-primitives.md`, `tools/anchor-sweep/inject_anchors.py`, plus `evidence/debate/keystone-bundle-2026-05-20-*` debate records. This is the namespace-level leakage source-30 (`:71`, `:135`) flagged: ADR-0242 hard-codes `oyatie.foundry.*` into the bootstrap; the brand is dead per ADR-0335/0347 → should be `oyatie.intelligence.*` / `oyatie.governance.*`.

### LIVE FOUNDRY RESIDUE COUNT (canonical tree; excludes target/buck-out/_upstream/vendor/.claude-worktrees/.git/_legacy-foundry)

Measured in `/Users/jasonlee/Developer/source`:
- **4,714 files** contain `foundry` (case-insensitive).
- **36,210** total `foundry` string occurrences.
- **780 files** carry the `oya-foundry-*` crate/token prefix.

This is the empirical disproof of ADR-0363's "eradicated" claim and the quantification of B-P0-4 "retirement incomplete." (Note: the AMENDMENT-PLAN's census-of-record is **831 non-ADR files / 43 Palantir carve-out** per `decision-record:107` / A.0-1 — that is the ADR-EXCLUDED, sense-routable subset; the 4,714 here is the unfiltered all-file count incl. ADRs, specs, evidence, and the retained doc shell `microservices/foundry/` 597 files. Both numbers are real; they measure different scopes.)

---

## (b) ADR-HYGIENE — is ADR-0377 still a live duplicate? + the de-dup set

### ADR-0377 duplicate — CONFIRMED still live (two files, two decisions, same number, canonical dir)

On-disk in `source/docs/decisions/` (NOT worktrees):
- `ADR-0377-kafka-to-pulsar-via-kop.md` — `status: Accepted` (L4), `supersedes: [ADR-0005]` (L9). Eventing.
- `ADR-0377-forgejo-board-git-ref-cas-fallback.md` — `status: Proposed (conditional: Accepted only after ADR-0377-D2 and ADR-0377-D3 …)` (L3). Forge board.

Two authoritative ADRs, identical id `ADR-0377`, different domains → **genuine collision, still unresolved.** Keystone map §6.1 (L152-155) and cross-tension `ci-cd-forge-build.md` T-8 (L292-307, disposition L344) both flag it. Agreed resolution in BOTH bodies: **renumber the Proposed forge-board one** (the Accepted kafka-to-pulsar one keeps 0377). Decision-record D11(c) (`:56`) lists the dangling-edge fixes incl. `0377→0397/0436`.

### ADR-0511 supersession status — CONFIRMED

- `ADR-0511-ci-orchestration-argo-workflows-supersede-jenkins.md` — `status: ... supersedes:[ADR-0359]` ; cross-tension confirms `Proposed` and CONTESTED vs the only Accepted/founder-locked node **ADR-0513** (`ci-cd-forge-build.md:105, 346`). Backlog register #12 (L74) + #16 (L78) want "mark ADR-0511 superseded_by ADR-0513"; decision-record/AMENDMENT-PLAN treat it as part of the CI-ADR-sprawl consolidation (CC-4). Same direction.

### The DE-DUP needed BETWEEN the two bodies (ADR-hygiene overlap)

Both bodies independently list the SAME hygiene fix-set. They must not be executed twice. Overlapping items:

| Hygiene item | Body-2 backlog (platform-readiness) | Body-1 decision-record / AMENDMENT-PLAN | De-dup ruling |
|---|---|---|---|
| **Duplicate ADR-0377** | register #12 (L539) "renumber duplicate ADR-0377"; B-P0-1 (L138) | AMENDMENT-PLAN L1.0 (`:119`) "the duplicate forge-board 0377 → next free id"; L3 dangling-edge fix; cross-tension T-8 | SAME fix. Own it in **Body-1 L1.0 renumber MAP** (it is the id-space authority). Backlog #12 references it, does not re-do it. |
| **ADR-0511 superseded_by 0513** | register #12 (L74), #16 (L78) | cross-tension `ci-cd-forge-build` (KEEP-but-CONTESTED, reconciliation note); CC-4 | SAME fix. Own it in Body-1 CC-4 / CI-consolidation; backlog references. |
| **status enum** | register #12 (L539) "3-axis status enum"; B1 (L64); Architect D5 (3 orthogonal axes decision/maturity/constraint, L153) | decision-record D-DOMAIN-adjacent; status-drift is the false-green root | SAME fix. Backlog's 3-axis design (D5) is the richer spec; Body-1 should ADOPT it rather than invent a parallel enum. |
| **regenerate ADR-INDEX / decisions.json** | register #12 (L539) "regenerate ADR-INDEX/decisions.json from source"; §P cross-artifact (L53) | keystone §6.3 "`decisions.json next_adr` is STALE, must be re-derived"; AMENDMENT-PLAN Wave-3 cohesion wiring | SAME fix. Own the regeneration in **Body-1 Wave-3** (after the renumber MAP + dedup land, else you regenerate a stale graph). Backlog §P provides the cross-artifact-agreement gate that ENFORCES it. |
| **foundry retirement completion** | B-P0-4 (L138); B2 (L76); register #12 "eradication" | CC-1 / L2 sense-routed rename; D-INTEL routing; L3.4 fix-0363 | SAME goal, Body-1 has the SAFE method (see §a). Backlog's "eradication" phrasing must be re-pointed at CC-1's sense-routed-with-carve-outs rule. |

**De-dup principle:** Body-1 (the consolidation decision-record + AMENDMENT-PLAN) is the **execution authority** for every mechanical ADR-id / rename / regenerate action — it owns the renumber MAP, the sweep ordering (D11 sweep-before-backfill, RULED), and the carve-outs. Body-2 (backlog) is a **planning input** (`.omx` SoT discipline, Founder Refinement #5, L122) whose hygiene items in register #12 are REFERENCES to the same fixes, not a second execution track. The de-dup is: **fold the backlog's register-#12 hygiene line into the AMENDMENT-PLAN lanes (L1.0 dedup, L3 dangling-edges, Wave-3 regenerate) and the 3-axis status enum (Architect D5) into Body-1's status work — do not run two parallel sweeps over the same ADR id-space** (a double sweep on an 831-file irreversible rename guarantees either a verifier stall or a miscount, AMENDMENT-PLAN A.0 `:20`).

---

## (c) AGENT-EXECUTION-CONTROLLER (§I) decision-gate — does reviving it CONTRADICT ADR-0116 / ADR-0363?

### Statuses (verified)
- **ADR-0116** `ADR-0116-retire-external-agent-coordination-tooling.md` — `status: accepted` (L3), `> Status: Accepted — 2026-05-16` (L10), supersedes ADR-0054. Retires grit/rtk/icm/vox.
- **ADR-0363** — `status: Accepted` (L3). Retires agentic-VCS foundry. **Amends ADR-0116** ("which parked external coordination tooling behind a manual-bootstrap seam — that seam is now removed, not deployed", L24).

### What §I is and what "reviving" means
`backlog-reconciliation/10-extract-backlog.md` §I (L138-148) + source `docs/ideas/agent-execution-controller.md`: §I = **agent EXECUTION as an ephemeral policy-gated unit of work** (run agent CLI as a K8s Job, isolated+audited, sealed `evidence-bundle.v1` with a reproduction command). It is CAPTURE-ONLY / decision-pending (2026-06-05), provenance = the now-superseded `oya-code` harness distillation; flagged as "the one concept source does not already own."

### The decision-gate, verbatim (source `docs/ideas/agent-execution-controller.md:33`)
> "**Is this layer wanted at all, or retired on purpose?** ADR-0116 (retire external agent-coordination tooling) and ADR-0363 (retire agentic VCS foundry) deliberately killed the adjacent coordination layer. This idea must either (a) be reconciled as a *net-new, narrower* concern that those ADRs did not intend to forbid, or (b) be explicitly declined. **Resolve this before any code.** *Decision owner: founder.*"

### VERDICT: reviving §I does NOT automatically contradict 0116/0363 — but it is GATED, founder-owned, and contradicts them IF revived in the broad form they killed.

- 0116 and 0363 are both **Accepted** and they genuinely "killed the adjacent layer": 0116 retired the external coordination tooling (grit/rtk/icm/vox), 0363 retired the agentic-VCS/coordination foundry. So a revival that re-introduces a *broad agent-coordination / agentic-VCS substrate* WOULD contradict the accepted retirements.
- **But §I is scoped to a NARROWER net-new concern** that the idea doc itself argues those ADRs "did not intend to forbid": a single-concern *execution* contract (work-item + pod-runner + evidence-bundle), explicitly distinct from cloud-intelligence (inference egress) and oya-intelligence (substrate/supervisor/guardrails), and — if pursued — a NEW flat single-concern service under oya/ or cloud/ per the pure split, **NOT folded into cloud-intelligence** (`10-extract:144`).
- The idea doc resolves the apparent contradiction the correct way: it does NOT assert revival; it routes to a **founder decision gate** with two clean outcomes — (a) revive-as-narrower-net-new, or (b) decline-and-let-the-oya-code-harness-repo-lapse — and forbids any code until ruled (`agent-execution-controller.md:33`).
- **ADR-0247 link:** §I's autonomy/execution ceiling sits under ADR-0247 (self-hosting/self-modification doctrine, `status: Proposed`) — the decision-record §F sign-off set explicitly lists "ADR-0247 self-mod ceiling" as a door:one-way founder item (AMENDMENT-PLAN `:228`). So §I cannot be revived independently of ratifying/scoping 0247.

**So:** the answer to "does reviving §I contradict 0116/0363?" is — **it would if revived broadly; it does not if revived as the explicitly-narrower net-new execution-contract concern, AND only the founder may make that call (decision gate is unresolved as of 2026-06-05).** This is NOT an ADR-hygiene/de-dup item; it is a genuine open founder decision that must precede any code, and it is correctly captured (not pre-decided) in both bodies.

---

## RETURN SUMMARY (one-screen)

- **Foundry-routing agreement:** Body-1 (decision-record/AMENDMENT-PLAN) and Body-2 (backlog) **AGREE on the goal but DIFFER on shape.** Body-1 = per-file SENSE-ROUTED (platform→oya-intelligence [current home; cloud-intelligence is the deferred endpoint], fitness→oya-governance, vcs→retired) with HARD carve-outs, never a blind swap (CC-1, D-INTEL `:86/:99`, L2.0 `:136`). Body-2 = "foundry→intelligence **eradication**" (register #12 L539, B-P0-4 L138) — a 1-target, binary-"eradicate" phrasing that, taken literally, mis-routes the ~135 governance-sense files and re-commits ADR-0363's false-green. The Task's own "platform→oya-intelligence / fitness→oya-governance / vcs→retired" = Body-1's Wave-0 routing exactly. **De-dup: re-point backlog register-#12 foundry line at Body-1 CC-1 / L2.0.**
- **Residue count (canonical tree, exclusions applied):** **4,714 files / 36,210 occurrences / 780 `oya-foundry-*` files** contain `foundry`. ADR-EXCLUDED sense-routable census-of-record = **831 files / 43 Palantir carve-out** (`decision-record:107`). Empirically disproves ADR-0363's verbatim "**The Foundry name was eradicated**" (`ADR-0363:35`).
- **False-green confirmed:** `docs/prds/foundry.md` = `status: Accepted`/published WHILE `specs/microservices/foundry.json` = `"status":"Retired"` (MSC-FOUNDRY-RETIRED, retired_by_adr ADR-0335). Cedar `oyatie.foundry.*` principals still live in `specs/{root-hub-pointers,tenant-model,platform-architecture,master-plan-sequencing}.json` + tools/ + evidence/debate.
- **ADR-0377 dup confirmed:** TWO live files in canonical `docs/decisions/` — `ADR-0377-kafka-to-pulsar-via-kop.md` (Accepted, supersedes 0005) + `ADR-0377-forgejo-board-git-ref-cas-fallback.md` (Proposed conditional). Both bodies agree: renumber the Proposed forge-board one.
- **0116 / 0363 status:** both **Accepted** (0363 amends 0116). They genuinely retired the adjacent agent-coordination/agentic-VCS layer. Reviving §I agent-execution-controller **does NOT contradict them IF scoped as the explicitly-narrower net-new execution-contract** (work-item/pod-runner/evidence-bundle, new flat service, NOT folded into cloud-intelligence); it WOULD contradict them if revived broadly. **Unresolved founder decision-gate** (`agent-execution-controller.md:33`, owner: founder, "resolve before any code"); tied to ADR-0247 (Proposed) self-mod ceiling. Not a de-dup item — a genuine open door.
- **De-dup needed between the two bodies (ADR-hygiene):** duplicate-0377 renumber, ADR-0511→0513 supersession, 3-axis status enum, regenerate ADR-INDEX/decisions.json, and foundry-retirement completion are listed in BOTH register #12 (backlog) AND the AMENDMENT-PLAN/decision-record. **Ruling: Body-1 (AMENDMENT-PLAN, execution authority) OWNS the mechanical actions** (L1.0 renumber MAP owns 0377 dedup; L3 owns dangling-edges; Wave-3 owns regenerate-after-renumber; CC-4 owns 0511); **Body-2 backlog is a planning input** (`.omx` SoT, Founder Refinement #5) whose register-#12 hygiene line REFERENCES — not re-executes — those fixes. ADOPT the backlog's Architect-D5 3-axis status enum into Body-1's status work rather than inventing a parallel one. **Do not run two sweeps over the same ADR id-space.**
