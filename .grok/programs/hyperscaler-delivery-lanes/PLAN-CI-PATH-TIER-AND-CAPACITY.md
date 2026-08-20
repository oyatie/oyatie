# Plan: CI path/event tiers + capacity ADR amend — then implement

**Status:** PLAN ONLY — no product ADR or workflow change until this plan is accepted  
**Date:** 2026-08-05  
**Drivers:** multi-PR queue death (`maxRunners: 1`); docs PRs pay full suite; Asterinas-style lessons; user ask for ADR amendment + follow-up fix after planning  
**Related live work:** #1562 (postgres path-filter), #1564 (dual-worker maxRunners=2), poller 1561–1568  

---

## 1. Problem (evidence)

| Symptom | Mechanism |
|---------|-----------|
| Docs-only PRs (#1565/#1566) still schedule firewall + buck2 + affected-set + producer | Single workflow `oya-ci-required` with almost all jobs **always-on** |
| Live-postgres taxes every PR | Path-filter designed only on branch **#1562**, not on `dev` |
| Multi-PR wall-clock | ARC general set `maxRunners: 1` (ADR-0630 + capacity contract); dual-worker unlock is **#1564** (git) + human apply |
| Runs queued with runners “busy” and no `in_progress` | Capacity / assignment / cancel-superseding gap |
| Agents thrash on “CI red” that is queue, not logic | No documented **PR class → expected suite** contract |

**Chesterton:** Always-on admission gates exist so docs cannot silent-break faces, total-accounting, census, or reachability. Path-skip must not open false-green channels.

---

## 2. Authority today (re-query)

| ADR | Status | Relevance |
|-----|--------|-----------|
| **ADR-0515** | **Accepted** | Single blocking context `oya-ci-required`; GitHub Actions sole CI authority; fan-in of registered gates |
| **ADR-0554** | **Proposed** (amended by Accepted **0636**) | Binding affected-set: CONE vs fail-closed FULL **inside** the affected-set job — **not** “skip job”; no skip path / label allowlist for the lane |
| **ADR-0636** | **Accepted** | Bounded baseline reuse; not capacity authority |
| **ADR-0630** | **Proposed** | ARC interim; documents `maxRunners: 1` storage boundary; dual-worker is operational evolution of this record |
| **ADR-0539 / 0541 / 0555** | **Proposed** | Freshness / liveness / accounting — still **live in CI**; path-skip must not empty these on material path classes |
| **ADR-0513** | **Superseded** by 0515 | Do not restore multi-required-context model without new Accepted decision |

**Gap (plan lag):** No Accepted ADR defines **which fan-in legs may be path/event-optional** vs **admission-always**, nor the dual-worker capacity model as law (0630 still Proposed; 1564 is code-first).

---

## 3. Goals / non-goals

### Goals

1. **Document law:** which `oya-ci-required` legs are path-optional / event-optional / schedule-only vs always-on PR.  
2. **Preserve** single required context name `oya-ci-required` (0515).  
3. **Fail-closed** when path relevance is unknown (match #1562: fail-open to *run*, not skip).  
4. **Capacity:** dual-worker general runners as the capacity decision (align git #1564 + apply + ADR text).  
5. **Implement** only after ADR amend (or narrowly scoped new ADR) is Accepted-ready and dual-critic’d.

### Non-goals

- Multiple parallel protected contexts (re-open 0513 multi-producer deadlock class)  
- “Docs PR skips all gates”  
- Warm CAS / RE activation  
- Replacing ARC with hosted runners as permanent law (billing wall remains 0630 context)  
- Hand-edit of `*.generated.json`

---

## 4. Proposed decision shape

### 4.1 Prefer **amendment**, not sprawl

| Decision | Form | Why |
|----------|------|-----|
| **A. Path/event-optional fan-in legs** | **New Accepted ADR-0639** *or* **amend ADR-0515** with a short D-section | 0515 is the singleton; optional legs must be explicit under the same fan-in. Prefer **new ADR amends 0515** so 0515 body stays readable; number = next free Accepted slot after re-query (verify `ADR-INDEX` next id on implement day). |
| **B. Dual-worker capacity** | **Amend ADR-0630** (still Proposed → promote to Accepted if founder ratifies, or keep Proposed with accurate maxRunners=2 dual-worker text) | Capacity is runner substrate; belongs with 0630, not 0515. |
| **C. Do not rewrite ADR-0554** | Optional **notes only** | CONE/FULL remains *within* affected-set job; optional leg skip is orthogonal. |

**Recommended numbering (subject to live index):**  
- **ADR-0639** — *Path- and event-conditional constituents under the single `oya-ci-required` fan-in* (amends 0515).  
- **ADR-0630 amendment** — dual-worker general scale set, anti-affinity, 48Gi-per-node, human apply residual.

### 4.2 Content of ADR-0639 (draft decisions)

1. **D1 — Singleton preserved.** Branch protection still requires only `oya-ci-required`. No second required context for “docs CI.”  
2. **D2 — Leg classes.** Every fan-in job is classified as one of:
   - **A-always (PR+trunk):** admission-critical (e.g. producer-regen, freshness, registry-drift, firewall, generated-output-diff-policy, gate-affected-set, fan-in itself; and **buck2** until a later Accepted carve-out with proof)  
   - **P-path-optional (PR only):** expensive specialty (e.g. live-postgres adapters/facades) — skip=OK on fan-in when path-irrelevant; **trunk always runs**  
   - **E-event-optional:** e.g. cache-writer identity (trusted push only) — already exists  
   - **S-schedule-only:** future heavy axes — not required for PR merge  
3. **D3 — Fail-closed uncertainty.** Unresolvable diff ⇒ **run** the optional leg (not skip).  
4. **D4 — Fan-in algebra.** Optional legs: `success ∨ skipped` green; all A-always legs: `success` only; failed optional leg still reds.  
5. **D5 — No silent deletion.** Path-optional is not gate retirement; job stays registered; metrics should count skip rate.  
6. **D6 — Docs class.** “Docs-only” is **not** a free pass on A-always. Future optional thinning of buck2 for pure `docs/**` requires **separate** Accepted decision + false-green analysis (out of v1).  
7. **D7 — Cancel-in-progress.** Superseded PR heads should cancel prior runs for the same workflow+PR to protect scarce runners (implementation under 0630 or 0639).  

### 4.3 Content of ADR-0630 amendment (draft)

1. Dual-worker general: `maxRunners: 2`, arch selector, required hostname anti-affinity, general workspace path on both workers.  
2. Live-postgres remains `maxRunners: 1` on its node.  
3. Git declarations are apply-source; human apply runbook is mandatory for capacity effect.  
4. Does not authorize warm CAS.

---

## 5. Implementation plan (after ADR text Accepted or founder-approved draft PR)

### Wave 0 — Plan freeze + beads (this file)

- [x] This plan written  
- [ ] User/founder accept plan  
- [ ] Beads: epic child for ADR-0639 + 0630 amend + implement slices  

### Wave 1 — Decision records (docs-only PR)

**Worktree from `origin/dev`.** One PR or two:

| PR | Contents |
|----|----------|
| **PR-ADR-A** | ADR-0639 (or 0515 amend) + ADR index regenerate via sanctioned tool + masterplan bound_adrs if planning_impact |
| **PR-ADR-B** (can stack) | ADR-0630 amend dual-worker + capacity notes |

**Verify:** ADR frontmatter; `marketplace-dev-cli doc adr-index` (or current sanctioned path); no hand-edit of forbidden generated faces; dual-critic; merge.

**Hard stop:** Do not change `oya-ci-required.yml` in Wave 1 except comments pointing at ADR id.

### Wave 2 — Capacity apply (ops + already-open code)

| Step | Owner |
|------|--------|
| Merge **#1564** when green (dual-worker git) | agent poller + dual-critic |
| Human apply `RUNBOOK-scale-runners.md` | ops |
| Confirm ≥2 general runners under load | ops |
| Cancel superseded queued runs | agent/ops |

### Wave 3 — Path-optional postgres (code already largely in #1562)

| Step | Owner |
|------|--------|
| Rebase **#1562** onto post-ADR trunk if needed | agent |
| Ensure workflow comments cite **ADR-0639** | agent |
| Merge #1562; measure skip rate on next docs PR | agent |

### Wave 4 — Cancel-in-progress + queue hygiene

| Step | Owner |
|------|--------|
| Add concurrency group per PR for `oya-ci-required` with cancel-in-progress | agent |
| Document interaction with merge_group / Tide | agent |
| Dual-critic + merge | agent |

### Wave 5 — Optional expansion (only if metrics justify)

Path-optional candidates **after** postgres + dual-worker proven:

- Additional disk-heavy specialty jobs (if any still always-on)  
- **Not** firewall / total-accounting / affected-set / census without new ADR  

### Wave 6 — Optional docs-cone study (separate plan)

Falsifiable experiment: pure `docs/**` + allowlist still runs A-always set; report wall-clock and skip rates; **no** buck2 skip without new Accepted decision.

---

## 6. PR train order (implementation)

```
[Plan accept]
  → PR-ADR-0639 (+ index)
  → PR-ADR-0630-amend (or same stack if tiny)
  → merge #1564 (capacity git) → human apply
  → merge #1562 (postgres path-filter, cite 0639)
  → PR cancel-in-progress
  → (later) specialty path-filters only with metrics
```

Do **not** couple ADR + path-filter + capacity apply in one PR.

---

## 7. Verification / acceptance

| Gate | Pass criteria |
|------|----------------|
| ADR | Frontmatter Accepted (or founder-approved draft with explicit status); projections fresh |
| Singleton | `infra/branch-protection` / SSOT still only `oya-ci-required` |
| Postgres | Docs PR skips live-postgres; trunk still runs both; fail-open on bad diff |
| Capacity | Under dual-PR load, ≥2 general jobs can be `in_progress` concurrently |
| False-green | Planted test or recorded experiment: optional skip cannot hide a real postgres-path failure |
| Accounting | New workflow files still reachability-justified |

---

## 8. Risks

| Risk | Mitigation |
|------|------------|
| Path-filter misses a durable path | Fail-open run; expand prefixes from red incidents |
| Fan-in treats skip as fail | Explicit algebra D4; planted fan-in test |
| Dual-worker disk fill | Anti-affinity + 48Gi/node; keep postgres separate |
| ADR-0630 stays Proposed forever | Founder accept or supersede with Accepted capacity ADR |
| Plan lag again | Re-query ADR-INDEX + 0515/0554/0630 on implement day |

---

## 9. Asterinas mapping (what we take / reject)

| Take | Reject |
|------|--------|
| Path-scoped specialty lanes | Multiple required contexts |
| Schedule heavy axes | Docs skip all core gates |
| Cancel-in-progress | Unlimited free-tier assumption |
| PR dry-run vs push publish | Dropping admission faces for FinOps |

---

## 10. Ask to user (acceptance)

Approve this plan (or amend) on:

1. **ADR-0639 new** (amends 0515) vs **inline amend of 0515 only**  
2. **0630 → Accepted** as part of capacity amend, or remain Proposed with amendment text  
3. **Wave 1 docs-only first** before any more workflow edits beyond open #1562/#1564  

On accept: implement Wave 1 ADR PR in an isolated worktree; continue babysitting open PRs in parallel.
