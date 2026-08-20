# Authority-verified + mined backlog

**Updated:** 2026-08-05  
**Method:** Mine ultragoal snapshots, masterplan pointers, handoffs, session work; **verify authority against ADRs on `origin/dev`** (and PR-bound ADRs when the task is on that PR).  

**Authority policy (corrected):**

1. **ADRs numbered &lt; 600 remain potentially valid and relevant.** They are not discarded by age.  
2. **Before relying on an ADR &lt; 600**, run an **additional check** for **contradiction** with more recent ADRs (especially Accepted ≥ 600, and Accepted/Proposed peers that claim supersession or amend).  
3. **If contradicted:** do **not** silently ignore either side — open or schedule **amendment** and/or **supersession** of the stale ADR (and update frontmatter/`superseded_by` / `amended_by` edges). Live CI behavior is evidence of practice, not a substitute for fixing the decision record.  
4. **If not contradicted:** the older ADR may still govern (or co-govern) that mechanism; cite it and the check performed.  
5. Prefer **re-query** of status on `origin/dev` (Accepted / Proposed / Superseded) over number alone.  
6. AGENTS + single protected context **`oya-ci-required`** (ADR-0515) remain merge-admission floor unless a later Accepted ADR explicitly supersedes them.  
7. Approved plan SHAs (#1560, k8s W0) are **execution intent**, not automatic merge of any PR.

**Lenses applied to every wave:** Cartesian doubt · Essentialism/YAGNI · Chesterton · Pragmatism · Red team · Systems · Operability · Opportunity cost · Blast-radius · Constant-work · Shared-nothing · FinOps · Telemetry-first · Zero-trust · (RE/CAS: causality before scale).

**Bun waves (lessons):** (1) prep contract first · (2) one representative trial before fan-out · (3) dual split-context review · (4) fail closed on missing evidence · (5) edit process/harness when systematic · (6) do not expand while trial red/incomplete.

---

## 1. Authority matrix (as of `origin/dev@a4a5ace5`)

### 1.0 Contradiction check (required for ADR &lt; 600)

When a task cites an ADR numbered below 600:

| Step | Action |
|------|--------|
| 1 | Read frontmatter: `status`, `superseded_by`, `amended_by`, `related` |
| 2 | Search `docs/decisions/` for later ADRs that name it in supersession/amend/related |
| 3 | Compare claim to live gate behavior on `origin/dev` (CI may still enforce a Proposed ADR) |
| 4 | **No contradiction** → treat as still relevant; document the check |
| 5 | **Contradiction** → stop expansion; file amend/supersede work (decision record debt is first-class) |
| 6 | **Ambiguous** → Socratic: escalate with both ADRs + evidence; do not invent consensus |

**Examples of known supersession chains (non-exhaustive):** historical VCS/ratchet ADRs vs ADR-0363 / ADR-0515 fabric; re-check before any “restore old VCS” work. Gate ADRs 0539/0541/0554/0555 may be **Proposed** yet **live in CI** — that is practice; promote/accept or amend deliberately if program law is needed.

### 1.1 Governing floor (treat as binding unless later Accepted supersession)

| Authority | Status | Use for |
|-----------|--------|---------|
| AGENTS.md + `oya-ci-required` / ADR-**0515** | Accepted | Merge admission singleton |
| ADR-**0636** | Accepted | Bounded cross-run affected baseline reuse (not CAS/RE capacity authority) |
| ADR-**0632** | Accepted | Product protocol / owned-fabric posture |
| ADR-**0635** | Accepted | Face-aware substrate dependency graph |
| ADR-**0613–0616**, **0619**, **0624** | Accepted | De-commit faces / census epoch / harness brand retirement |
| **Any Accepted ADR &lt; 600** that survives §1.0 check | Accepted | Still binding for its scope |
| Approved CAS plan SHA `8833df33…` (#1560) | Consensus plan | CAS/RE sequencing **intent** — not merge of #1558 |
| Approved k8s plan SHA `7010aebc…` (session) | On #1561 branch | W0 only; **0637/0638 not on `origin/dev` yet** |

### 1.2 Live gates with **Proposed** ADR (enforce in CI today; status still provisional)

| Gate / topic | ADR | Note |
|--------------|-----|------|
| Freshness | 0539 Proposed | Live in CI; accept/amend if program needs Accepted law |
| Corpus liveness / unpackaged | 0541 Proposed | Live ratchet; still relevant under §1.0 |
| Affected-set binding | 0554 Proposed | FULL/CONE behavior live |
| Unowned/unreachable | 0555 Proposed | Live on firewall; still the design for #1558 packaging |
| NativeLink CAS slice | 0560 Proposed | **Does not authorize warm reads** until Accepted + evidence |
| Buck2 RE / NativeLink scheduler | 0612 Proposed | **No RE activation** until earned + Accepted/plan gate |
| Many 0600–0631 gates | Proposed | Re-read status; contradiction check before expansion |

Proposed ≠ irrelevant. Proposed ≠ free to ignore when CI fails closed on it.

### 1.3 ADR &lt; 600 — still valid until contradicted

| Do | Don’t |
|----|--------|
| Use pre-600 ADRs for mechanism, gate design, and history | Assume “&lt; 600 ⇒ obsolete” |
| Run §1.0 contradiction check against newer ADRs | Silently prefer a new ADR without recording supersession |
| If conflict: **amend or supersede** the stale record | Leave contradicting Accepted/Proposed pairs unresolved |
| Cite both the older ADR and the check outcome in task prep | Expand CAS/RE from Proposed-only without plan + evidence |
| **Live-resolve** every cited ADR (supersession + `amended_by`) | Treat raw `status: Accepted` as current whole law |
| Prefer later live successor / amend set when chains conflict | Cite bare 0515 / bare 0562 / bare older Accepted alone |

### 1.4 Not yet on trunk

| Item | Where | Implication |
|------|--------|-------------|
| ADR-0637 / 0638 | PR #1561 only | Authority for W0 **pending merge**; don’t implement W0-B on trunk until G001 promotes |
| CAS plan G040–G044 | Ultragoal + #1560 | Ordered **after** G039 terminal |

---

## 2. Ultragoal mining (CAS session snapshot) — authority-checked

Source: `cas-fabric/evidence/ultragoal-goals.snapshot.json` (durable snapshot, re-query before mutation).

| UG id | Title (short) | Status in snapshot | Authority check | Lenses / disposition | Wave |
|-------|---------------|--------------------|-----------------|----------------------|------|
| G038 | Reconcile Stage-1 + CAS/RE baseline | complete | Plan SHA bind | Done | — |
| **G039** | #1558 storage pilot | in_progress | Plan + L2 CI; **not** warm CAS | Keep draft until review; no G040 | **W0 trial** |
| G040 | NativeLink/Buck2/CI migrations | pending | 0560/0612 **Proposed**; plan | Only after G039 terminal | **W2** |
| G041 | Credential gates + cache-only CAS proof | pending | #1541 + plan; zero-trust | **Human security first**; no agent secrets | **W1** (with human) |
| G042 | Production CAS + SCM/CI cutover | pending | Needs Accepted CAS ADR cluster | Do not start | **W3** |
| G043 | Evidence-gated RE decision/canary | pending | 0612 Proposed; #1549 | After CAS proof | **W4** |
| G044 | CAS/RE program audit + completion | pending | Quality gates only | Terminal audit | **W5** |
| G019 | Stage-1 PR train (legacy) | in_progress snapshot | **Steering-blocked/superseded** by G038 notes | Do not revive train | **park** |
| G028 | ARC churn / protected-CI queue capacity | pending | Ops + ADR-0630 Proposed interim | Aligns with **R1 runners** | **W0 ops** |
| G020–G022 | Bun closed-loop / throughput / meta | pending | Process; kit mm-drive | Kit waves; not merge authority | **W0 process** |
| G001–G018, G023–G037 | Masterplan product phases | mixed | Broader portfolio | Out of this CAS/k8s session unless assigned | **portfolio** |

---

## 3. Ultragoal mining (k8s port session)

Source: `.gjc/.../ultragoal/goals.json` (provenance only; no gjc CLI).

| Id | Wave | Status | Authority | Disposition |
|----|------|--------|-----------|-------------|
| G001 | W0-A | active; CI green | 0637/0638 **on PR only** until merge | Human APPROVE → merge → packet |
| G002–G008 | W0-B…H | pending | Sequenced behind G001 | **No start** |
| W1+ | — | unapproved | Plan boundary | **Out of scope** |

---

## 4. Session + handoff items (already tracked) — authority tags

| Item | Lane | Authority | Wave |
|------|------|-----------|------|
| #1558 G003 green draft | R5 | CI live + plan pilot; 0555/0541 **Proposed but gated** | W0 trial |
| #1558 review/merge/G039 proof | R5+R3 | 0515 + plan terminal | W0→W1 |
| #1561 formal review/merge | R7 | 0515; 0637/38 promote on merge | W0 |
| #1559 packet | — | 0515 post-merge | **Done** |
| R1 more runners | R1 | Ops; 0630 Proposed as interim substrate | **W0 ops** |
| R2 pre-merge path-filter / materialize-once | R2 | 0515 floor; workflow edit | **W1 process** |
| R3 post-merge packet automation | R3 | AGENTS post-merge product gate | **W1 process** |
| R4 local YAML recipe | R4 | Pragmatism / CONE | **Done tip** |
| mm-drive / quant kit | R8 | 0619 Accepted (no external harness brand); process | **W0 process** |
| #1541 | R9 | Security; blocks G041 | **W1 human** |

---

## 5. Newly mined / emphasized (from ledgers) with lens verdict

| Mined item | Source | Authority | Lens verdict | Wave / lane |
|------------|--------|-----------|--------------|-------------|
| **G028 runner capacity** | UG snapshot | Ops reality + queue evidence | Pragmatism: highest short-term wall-clock ROI | **W0 / R1** |
| **G040 ordered migrations** | UG | Plan; ADRs Proposed | Don’t fan-out until G039 trial green+promoted | **W2 / R5→** |
| **G041 cache-only proof** | UG | #1541 + plan | Zero-trust: human creds before warm | **W1 / R9→R5** |
| **G043 RE decision** | UG | 0612 Proposed | Causality before scale: after CAS | **W4 / R6** |
| **De-commit face ADRs 613–616** | Accepted 600+ | Accepted | Chesterton: keep face policy; R2 must not weaken | **R2 constraints** |
| **0636 baseline reuse** | Accepted | Accepted | Allows affected-set reuse; not CAS authority | R2/R5 CI behavior |
| **0612 RE ADR** | Proposed | Proposed | Red team: no RE in admission path | R6 only |
| **0560 NativeLink** | Proposed | Proposed | No warm cache flip | G040/G041 |
| **Portfolio G023–G037** | UG | Mixed | Opportunity cost: park unless assigned | portfolio |
| **G019 stage-1 train** | UG in_progress | Superseded by G038 | Essentialism: park | park |
| **Masterplan MPV2 k8s W0** | #1561 | On PR | After G001 | R7 then G002+ |
| **Console G029** | UG | Separate product | Out of monorepo CAS session | external |

---

## 6. Bun-style implementation waves

### Wave 0 — Representative trials + ops unlock (no fan-out)

**Prep contract:** keep drafts; no warm CAS; no RE; no self-approve.

| Track | Action | Done when |
|-------|--------|-----------|
| R5 | #1558 formal review → merge → **promoted** G039 packet | G039 terminal |
| R7 | #1561 formal review → merge → G001 packet + beads | G001 checkpoint |
| R1 | Scale runners / max-parallel to pool | Queue wait collapses |
| R8 | Optional kit D3–D4 only if not blocking trials | evaluate ≥ A |

**Stop expansion while R5/R7 trials incomplete** (Bun: one E2E trial before fan-out).

### Wave 1 — Process + security (parallel, path-disjoint)

| Track | Action | Authority |
|-------|--------|-----------|
| R2 | Pre-merge workflow shape PR (path-filter, materialize-once) | 0515; don’t weaken 613–616 faces |
| R3 | Post-merge packet/writer automation | AGENTS product gate |
| R9 | #1541 human credential close | Security |
| R4 | Keep local assist as default agent habit | Pragmatism |

**Dual review** on R2/R3 workflow PRs (split-context).

### Wave 2 — Ordered CAS/CI migrations (G040)

Only if Wave 0 G039 terminal **and** plan re-query still holds.

- NativeLink/Buck cache behavior per plan slices  
- Still **Proposed** 0560/0612 — require Accepted decision or plan waiver before warm  

### Wave 3–5 — G042 production CAS · G043 RE · G044 audit

Per plan; RE only if evidence-earned; G044 quality-gated complete only.

---

## 7. Critical-lens checklist (every task before implement)

1. **Cartesian:** Evidence on `origin/dev` + live PR, or assumption?  
2. **Authority:** Which ADRs apply (any number)? Status on trunk? **§1.0 contradiction check** if any cited ADR is &lt; 600 or Proposed?  
3. **Contradiction debt:** If newer ADR conflicts with older — is amend/supersede filed, or are we illegally picking one?  
4. **YAGNI:** Smallest trial that falsifies?  
5. **Chesterton:** Why does this gate/face/ADR exist?  
6. **Blast-radius:** Own worktree/PR; no couple R1+R2+R5.  
7. **Zero-trust:** Secrets, warm CAS, RE out of untrusted PR writers.  
8. **Operability:** Who fixes at 3am; packet/rollback?  
9. **FinOps:** Runner scale vs recompute vs RE — pick scarce resource.  
10. **Telemetry:** Queue wait vs gate runtime vs CAS hit rate (later).  
11. **Bun:** Trial green before fan-out; edit process on systematic fail.

---

## 8. Maintenance

- Re-run `git ls-tree origin/dev docs/decisions/ADR-06*` after merges (0637/0638 promotion).  
- Refresh ultragoal statuses from live goals if recovered; snapshots lag.  
- Update `SESSION-BACKLOG.md` status columns when waves advance.  
- Never mark G044 / ultragoal complete from Wave 0 PR green alone.
