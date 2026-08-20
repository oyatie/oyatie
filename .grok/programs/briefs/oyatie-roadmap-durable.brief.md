# Durable goal brief — Oyatie roadmap implementation (closed-loop)

**Artifact class:** durable ultragoal brief for `mm-goals create --brief-file`  
**Not merge authority.** Merge remains dual-critic + `oya-ci-required` (ADR-0515).  
**Orchestration:** `.grok` only — no `gjc` / `omc` / `omx` / `hermes`.  
**Doctrine:** Bun closed-loop (trial → dual adversarial → fan-out → measure → edit process).  
**Authority law:** Accepted ADRs on **live `origin/dev`** beat masterplan, ultragoal snapshots, session backlog, and reorg plans.

---

## Shared constraints (every goal inherits these)

### A. Mission

Implement the **Oyatie production roadmap** end-to-end through small, merge-safe, path-disjoint waves until FD-001 hyperscaler-grade exit is honestly claimable and later verticals are unlocked under live authority — not a short task list, not a single session.

Owned-stack direction (founder policy): **Rust whole stack** — kuberos kernel → cloud-os → cloud-k8s → cloud services → oya products; upstream k8s/Talos only behind stable transitional interfaces (ADR-0510 cluster).

### B. Authority re-query (fail-closed; never skip)

Before **any** implement PR, plan mutation, or reorg card execution:

1. `git fetch origin dev` and record tip SHA.  
2. For every ADR the slice cites, run **live resolution** (below) — **do not treat `status: Accepted` as sufficient**.  
3. Run **contradiction check**: if masterplan / ultragoal / backlog / move-plan disagrees with **resolved live** ADRs or live tree → **stop**. Amend plan or ADR edges; do **not** implement the stale path.  
4. Treat these as **queue signals only** (never sole law):  
   - `specs/master-plan-sequencing.json` / masterplan projections  
   - historical ultragoal / `.omx` / `.gjc` snapshots  
   - `SESSION-BACKLOG.md`, mined cards, chat memory  
5. **Plan lag**, **stale execute**, and **stale Accepted** (superseded-in-substance but still labeled Accepted, or Accepted without reading amends) are separate defects. All block dispatch.

#### Live ADR resolution (mandatory)

`status: Accepted` means “was accepted at some time,” **not** “is the current whole law for this topic.”

```text
resolve_live(adr_id) on origin/dev tip:
  1. Load frontmatter + body.
  2. Reverse index: if any LATER Accepted ADR lists this id under `supersedes:`,
     treat this ADR as superseded → recurse to that successor
     (even if this file still says status: Accepted — that is a D2/D3 hygiene defect).
  3. If status is Superseded / Deprecated:
       follow `superseded_by` (or reverse claimers if field empty) → recurse.
  4. If status is Proposed / Rejected / missing:
       NOT implement authority (Accept or park first).
  5. If status is Accepted after steps 2–3:
       a. Load every `amended_by` peer that is itself live-Accepted;
          apply amendments as later overrides — never bare parent alone
          (e.g. 0562 + 0615 + 0635; 0515 + 0624 + 0639 + fabric cluster).
       b. Record the full resolution path in the PR authority receipt.
  6. If two live peers conflict → STOP; open amend/supersede edge PR; do not pick favorites.
```

**Anti-patterns (forbidden):**

- Cite “ADR-XXXX is Accepted” without resolution path and `amended_by` set.  
- Prefer an older Accepted over a later Accepted that supersedes or amends it.  
- Implement from Superseded text because a plan/ultragoal still quotes it.  
- Treat `depends_on: [Proposed]` as if those Proposed were live law.

### B2. Full ADR disposition (all numbers — not only 600+)

**Corpus size on tip (~448 ADRs).** Every implement slice re-queries **all** ADRs that the slice cites, including **&lt;600**, through **live resolution**.

| Status field | Agent rule |
|--------------|------------|
| **Accepted** | **Candidate only.** Must pass live resolution + `amended_by` peers. May be stale if a later ADR supersedes it. |
| **Proposed** | **Not authority.** Admit (Accept) only with evidence + PR; never mass-Accept. |
| **Superseded** | Historical; follow chain to live successor (often **0515**, **0363**, **0616**). |
| **Missing status** | Block claims until triaged (D1 wave). |

**Plan-lag (must clear in R0 or waive):** Accepted ADRs whose `depends_on`/`amends` still list Proposed IDs — notably parents **0565, 0614, 0616, 0619, 0630, 0635, 0636, 0637, 0638, 0639** citing faces/CAS/RE/enforcement Proposed ADRs.

**Activation hard-blocks until Accept:** **0612** (RE), **0560/0556** (CAS warm path), secrets **#1541** human.

**Disposition SSOT (audit):**  
`docs/decisions/_disposition/` (mechanical case normalize + full queues) and  
`.grok/mm-runs/20260805T230336Z-a5ce047f/evidence/adr-full-disposition-audit.json`.



### B3. ADR end-state (clean live source of truth)

Terminal tree statuses only: **Accepted** | **Superseded** | **Rejected** — **no Proposed**.  
Every former Proposed is decided now (Accept / Reject / Superseded).

After fold receipts: **delete** Superseded and Rejected from the working tree (git history preserves).  
**Blocker:** ADR-0624 census epoch currently binds `docs/decisions` corpus — E4 delete/quarantine only with epoch/selector change.

**Live resolution** still applies until consolidation collapses amends.  
**Consolidate** overlapping Accepted into topic apex ADRs; renumber only with `adr-redirect.v1.json` + repo-wide citation rewrite.

SSOT artifacts: live apex **ADR-0700…0709** under `docs/decisions/`; archive under `docs/adr-archive/`; `docs/decisions/_disposition/END-STATE-POLICY.md`, `adr-redirect.v1.json`, `live-accepted-index.json`.

**R0 DoD includes:** disposition receipt for plan-lag + missing-status + superseded-without-successor queues (not “all Proposed Accepted”).

**Reorg may be the first real wave.** Do not assume product fan-out is next if live topology / dual-home / face policy / capability boundaries are still wrong relative to ADR-0613–0616, 0615/0631/0635, 0619, 0624, 0632–0635.

### C. Closed-loop autonomy (Bun; zero intervention)

- **Zero human intervention** for plan, implement, dual-critic, preflight, push, merge-check, merge, post-merge packet, tip-sync, process edit, and next-queue selection.  
- **Multi-critic is agents only** (split-context dual-critic minimum; more for high-risk). Human GH APPROVE is **not** required.  
- **Exception class (only):** cryptographic secrets / credential materialization (#1541 class), founder ratification when an ADR requires it, and external account provisioning that agents cannot perform. These are **human_blocked** lanes — agents prepare evidence and stop without thrashing.  
- No passive WAIT: `waiting_ci` requires armed re-poll + re-tick; tip-sync after every merge.

### D. Parallelism

- **Serialize** shared contracts, shared data model, root workspace membership, branch-protection/admission, toolchain required-context, and active move-plan ownership.  
- **After contract lock:** max **path-disjoint** implement lanes (one writer per hot path/file set).  
- **One concern per PR.** Multi-capability reorg is OK as an epic; each PR has one dominant class (move|refactor|rewrite|delete|rebrand).  
- **Trial before fan-out** for each new *class* of work (capacity, path-filter, reorg move, product scaffold).  
- Do not couple: runners + CAS warm + k8s engine + product µservice in one PR.

### E. Durable state (dual SSOT, fail-closed)

| Store | Role |
|-------|------|
| **mm-goals** `.grok/mm-runs/<run_id>/goals.json` + `ledger.jsonl` | Execution SSOT for goal status, checkpoints, evidence refs |
| **beads** (`bd`) | Issue cards, blockers, handoff; must mirror goal IDs / external refs |
| **programs/** | Lane SSOT (`k8s-port`, `cas-fabric`, `hyperscaler-delivery-lanes`, reorg doctrine) |

Checkpoint rule: `mm-goals checkpoint --status complete` **fails closed** unless:

1. quality-gate JSON pass,  
2. beads card closed or explicitly linked with same SHA/packet,  
3. evidence paths exist (PR, merge SHA, dual-critic, `oya-ci-required` exact-head).  

Drift between mm-goals and beads on an active goal → **block further fan-out** until reconciled (process_edit).

### F. Delivery mechanics (every implement slice)

1. Isolated worktree from **current** `origin/dev`.  
2. Authority re-query receipt in PR body or evidence JSON.  
3. Dual-critic APPROVE packet under `.grok/programs/**/evidence/`.  
4. CI-infra paths: `.grok/bin/preflight-ci-infra --pr N` at **exact PR head** (process gate).  
5. One push; never mid-run thrash.  
6. `mm-drive merge-check` → `merge` when ok.  
7. R3 post-merge packet on **promoted** SHA when product-complete is claimed.  
8. Tip-sync remaining open PRs.  
9. `mm-score` / `mm-grade` / `mm-learn` at wave boundaries → **process_edits** into harness (not chat notes).  
10. Never hand-edit `*.generated.json`. Never claim ultragoal aggregate complete from local green alone.

### G. Hard stops (always)

- No warm CAS / `remote_enabled` / RE workers without Accepted authority **and** earned prerequisites (plan + #1541 + proofs).  
- No secrets in git/logs.  
- No second protected admission context.  
- No W1+ k8s corpus until approved.  
- No gjc/omc/omx/hermes as live authority (ADR-0619).  
- No implement from dirty primary / cas-live-proof preserve tree.

### H. Self-assessment loop (continuous, parallel to delivery)

At every consolidation boundary (merge train idle, wave end, or N≥3 systematic failures):

- Measure: verified lead time, queue wait, conflict/rework rate, head_mismatch incidents, passive WAIT incidents, trial-red fan-out, dual-critic defect yield, false-complete attempts, process_edit lag.  
- **Fix process not only bug:** edit `.grok/workflows/*`, `preflight-ci-infra`, `drive.v1.json`, role prompts, grade triggers.  
- Promote process_edit after **2 repeats** of a mistake class.  
- Re-partition path-disjoint queues from measured contention, not gut feel.

### I. Definition of Done (roadmap-level — not session-level)

Roadmap goal is **not** complete until all of:

1. **Authority graph current:** masterplan / sequencing / ultragoal / reorg inventory reconciled to Accepted ADRs on tip (or explicit amend PRs landed).  
2. **Topology honest:** dual-home / face / capability boundary debt for active work is disposed (park|move|refactor|rewrite|delete) with evidence.  
3. **Admission fabric healthy:** hosted `oya-ci-required` sole context; process harness fail-closed; trunk green on tip.  
4. **CAS path:** cache-only proof path either complete under Accepted authority or honestly Option-D terminal without false RE claims.  
5. **K8s port W0:** G001 done; W0-B…H per ready-gates without bulk W1.  
6. **FD-001:** required surfaces (core, messenger, mail, community, infra, ops-dashboard, intelligence, workflow, ontology, canonical-base, korea pack) meet architecture exit bar under live ADRs — full-depth, not MVP cosplay.  
7. Closed-loop metrics show sustained verified throughput without human intervention except true human_blocked classes.  
8. Dual SSOT (mm-goals + beads) consistent; no open false-complete claims.

Partial wave completion is normal; **aggregate roadmap complete** is quality-gated only.

---

## Goal graph (durable; parallelizable)

Execute in dependency order. Within a wave, max path-disjoint parallelism.

@goal: R0 Authority freeze and stale-plan reconciliation
Re-query **entire** ADR corpus on live origin/dev (all numbers, not only 600+): status histogram, Accepted/Proposed/Superseded/missing, plan-lag edges, superseded without successor, status-case debt, and **stale-Accepted** (status Accepted but later supersession/amends apply). **Never treat status=Accepted as-is** — run live resolution (supersession chain + amended_by) for every cited ADR. Re-query floor clusters only via live resolution (0515 chain, 0562+0615+0635, 0613–0619, 0637/0638). Diff master-plan-sequencing, ultragoal snapshots, SESSION-BACKLOG, REORG cards, CAS plan against tip. Produce disposition receipt: keep | amend | supersede | accept | park | waive-design-input-only | mark-stale-accepted. Mechanical-only bulk OK (status case). **Never mass-Accept Proposed.** Open PRs for: (1) mechanical normalize, (2) D1 missing status, (3) D2 plan-lag + stale-Accepted hygiene, (4) D3 superseded_by fill, (5) selective Accept when next slice requires. Hard stop: no product/CAS/RE implement while blocking plan-lag, stale-Accepted, or Proposed-as-authority remains on touched surfaces. Dual SSOT: mm-goals + beads R0. DoD: disposition artifacts + live-resolution rule documented; zero unwaived plan-lag/stale-Accepted on active dispatch set; not “all ADRs Accepted.”

@goal: R1 Reorg disposition wave (may precede all product)
Using REORG-DOCTRINE (move|refactor|rewrite|delete|rebrand|mixed), rank live dual-home, face, capability-boundary, and spent-plan debt from re-queried tip — not from stale inventory alone. Execute path-disjoint reorg PRs one concern each; at most one active move-plan (0614). Prefer topology honesty before new product crates. Parallel with R2 only when path-disjoint. DoD: active reorg cards for current horizon disposed or parked with evidence; no dual-home surprises on next product slice paths.

@goal: R2 Admission and delivery fabric (substrate)
Keep sole protected context oya-ci-required healthy (hosted GHA primary; ARC overflow only). Maintain preflight-at-PR-head, tip-sync, cancel-in-progress, path-optional legs under Accepted 0639, dual-critic merge path, R3 packets. Process harness under .grok is first-class product of this goal. DoD: trunk tip green; process gates prove head_mismatch and mid-run push classes fail closed; open PR queue drainable without hero ops.

@goal: R3 CAS fabric ordered (cache-only before RE)
After R0/R1 gates that touch CAS paths: ordered 3A→3B→3C from promoted heads only; no warm flip; root .buckconfig dark; #1541 human_blocked for secrets. G041 cache-only proof only after #1541 + Accepted warm/CAS authority or plan-scoped waiver. RE (G043/#1549) only after measured reopen + Accepted RE decision (0612 Proposed is not authority). DoD: either earned cache-only path with packets or explicit Option-D terminal without RE cosplay.

@goal: R4 K8s deterministic port W0 (G001–G008)
G001 governance complete (promoted + packet). Flip W0-B ready-gate only on explicit G002 start after re-query. Implement W0-B…H slices per W0-B-ADMISSION-PLAN and ADR-0637/0638: engine under build/port-engine, not ci/*; no bulk W1 corpus; no Go in verify(). Parallel with R3 only path-disjoint. DoD: W0 exit criteria in plan met; W1 still unapproved unless new Accepted ADR.

@goal: R5 FD-001 contract lock
Lock shared API contracts, shared data models, packaging axes (Tenant/RBAC), portable K8s deploy matrix, and evidence/SLO authoring rules for FD-001 required surfaces. Serialize root Cargo/workspace membership and cross-surface contract PRs. Re-query masterplan FD-001 against tip ADRs; amend sequencing docs if lag. DoD: contract-lock packet + green gates; product_fanout_entry_rule satisfied.

@goal: R6 FD-001 parallel product lanes
After R5: max path-disjoint implementation of required surfaces — core, messenger, mail, community, infra, ops-dashboard-control-center, intelligence, workflow, ontology, canonical-base, korea-localization-pack — full-depth hyperscaler exit bar (not MVP). Clean architecture, API-first, independent horizontal scaling, OTel/SLO before promote past dev. One µservice concern per PR; dual-critic + oya-ci-required; R3 packet per product-complete claim. DoD: FD-001 exit claim evidence pack passes architecture_exit_bar under live ADRs.

@goal: R7 Later verticals and platform phases
Only after FD-001 GA gate: Phase-1+ platform/capability/collab/enterprise families per live masterplan authority (re-queried). No sector vertical production-GA claim until FD-001 gate. Parallel path-disjoint within phase; serialize shared contracts. DoD: phase exit gates with packets; no false-complete.

@goal: R8 Owned-stack deepening (kernel → products)
Advance owned Rust stack seams (kuberos/cloud-os/cloud-k8s as authorized by Accepted ADRs) behind stable interfaces; keep upstream transitional. Does not authorize RE or warm CAS by itself. Coordinate with R3/R4. DoD: interface contracts + promotion evidence; no dual-authority CI.

@goal: R9 Closed-loop process intelligence (continuous)
Parallel forever: measure delivery system; dual-critic the harness; land process_edits; keep mm-goals↔beads sync fail-closed; refresh authority census on a cadence. Goals G020–G022 spirit without external harness brand. DoD: never “done”; health SLIs published each wave boundary.

@goal: R10 Human-blocked security and ops (non-thrash)
#1541 credential rotation, production account provisioning, founder ratify classes. Agents prepare runbooks/evidence only; status human_blocked; never invent secrets. DoD: each card closed by human with packet or explicit defer with risk acceptance.

---

## Wave policy (how the loop runs)

```text
loop forever until roadmap DoD:
  1. authority re-query on origin/dev
  2. select_next_ready: path-disjoint, dependency-satisfied, not human_blocked
  3. if new work class: one trial PR → dual-critic → green → only then fan-out
  4. parallel implement agents (bounded by runners + leases)
  5. dual-critic → preflight@head → one push → wait complete CI → merge
  6. tip-sync + R3 packet when required
  7. mm-goals checkpoint + beads close (dual SSOT)
  8. score/grade/learn → process_edits if systematic
  9. never expand while trial red; never implement stale plan
```

### select_next_ready priority (default)

1. Blocking authority contradictions (R0)  
2. Reorg cards that unblock next product/CAS paths (R1)  
3. Red admission fabric / trunk (R2)  
4. Ordered CAS slice if unblocked (R3)  
5. Next k8s W0 slice if gate open (R4)  
6. FD-001 contract then product lanes (R5→R6)  
7. Later phases (R7+)  
8. Always: R9 process improvements that cut measured thrash  

### Parallel lease rules

- One temporal owner set per path prefix per open PR.  
- Hot files (`.github/workflows/oya-ci-required.yml`, root `Cargo.toml`, generated faces materialize scripts): single-writer queue.  
- Reorg move-plan: singleton active plan.  

---

## Activation commands

```bash
# Create durable run from this brief (from repo with .grok harness)
.grok/bin/mm-goals create --brief-file .grok/programs/briefs/oyatie-roadmap-durable.brief.md

# Mirror into beads (fail-closed dual SSOT): one bead per @goal with external_ref=mm-goal-id
# bd create ... --title "R0 Authority freeze..." --description "see brief" 

# Drive
.grok/bin/mm-drive status --json
.grok/bin/mm-drive tick --json

# Wave boundary
.grok/bin/mm-score --run-id <id>
.grok/bin/mm-grade --run-id <id>
.grok/bin/mm-learn from-run --run-id <id>
```

Workflows to prefer (not exclusive):

| Situation | Workflow |
|-----------|----------|
| Authority/plan only | `lens-delivery-plan` |
| Fleet / tip-sync | `open-pr-fleet` |
| Capacity thrash | `parallel-delivery-bun` |
| CAS 3B+ after 3A | `post-3a-implementation-fanout` |
| Bounded CI fix | `implement-ci-substrate` |
| Product/reorg slice | worktree + dual-critic + mm-drive (product Rhai optional later) |

---

## Explicit non-goals

- Completing the roadmap in one PR or one day.  
- Treating historical ultragoal “steering-blocked” items as live dispatch without re-admit.  
- Claiming FD-001 or ultragoal complete from CI green alone.  
- Warm CAS / RE cosplay under Proposed ADRs.  
- Reorg from stale remote or unpropagated supersession.  

---

## Change control for this brief

When Accepted ADRs change disposition:

1. Update this brief in the same PR that lands the ADR edge or immediately after.  
2. Ledger event in mm-goals + beads.  
3. Re-rank R0/R1 before continuing R5+.  

**Version:** 2026-08-05.roadmap-durable.v1  
**Inputs:** user interview (horizon=authority-first/reorg-aware; autonomy=Bun zero-intervention multi-critic; parallelism=contract-lock then max path-disjoint; SSOT=mm-goals+beads fail-closed); REORG-DOCTRINE; master-plan-sequencing FD-001; ADR-0515/0613–0619/0637–0639; BUN-PARALLEL-DISCIPLINE.
