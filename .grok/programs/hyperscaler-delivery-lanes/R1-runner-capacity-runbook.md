# R1 — Runner capacity runbook (queue_wait)

**Lane:** `R1-runner-capacity` (L2 ops)  
**Drive:** Human ops / founder only — **agents may draft docs; agents cannot scale your cloud.**  
**Authority:** Ops capacity + interim ARC substrate (ADR-0630 **Proposed**). **Not** merge authority.  
**SSOT board:** [`LANES.md`](./LANES.md) · [`PROGRAM.json`](./PROGRAM.json)  
**CI surface:** `.github/workflows/oya-ci-required.yml` · `docs/ci/forge-of-record.md`

---

## 1. Evidence that queue_wait dominates

**Symptom (queue-bound, not gate-logic-bound):** under multi-PR or multi-lane load, jobs sit in **Queued / Waiting for a runner** while wall-clock elapsed grows, even though some sibling jobs later complete successfully.

**How to confirm (human, Actions UI or `gh`):**

1. Open a recent PR check run for the protected workflow `oya-ci-required` (fan-out → fan-in to the single required context).
2. Inspect **individual matrix / lane jobs** (gate matrix, buck2, firewall, reviewer-evidence, etc.):
   - **Queue wait dominates** when many jobs show long time in *Queued* / *Waiting for a runner*, then short-to-moderate *In progress*, and the **fan-in job `oya-ci-required` stays pending** until stragglers acquire runners.
   - **Compute dominates** when jobs start immediately but burn long *In progress* (CPU/cache/build) — that is **not** R1; see R2 path-filter / materialize-once or R4 local assist.
3. Multi-PR signal: while PR A’s gates are still queued, PR B’s (or other repos/org jobs on the same labels) may complete or steal capacity — classic pool starvation, not a missing gate.
4. Single-runner smell: only one (or few) concurrent jobs ever *Running* for labels in `OYA_CI_RUNNER_LABELS`, despite matrix width and `OYA_CI_MAX_PARALLEL` allowing more — capacity or label-match limit, not fan-in defect (`docs/ci/forge-of-record.md`).

**Acceptance target (PROGRAM):** during multi-PR load, *idle runners > 0 while jobs pending* is rare; P50 queue_wait for `oya-ci-required` jobs drops materially.

**Do not misdiagnose:** red gates, OWNERS/Buck failures, or missing reviewer evidence are **not** queue_wait. Fix those on the owning lane (R5/R7/etc.). R1 only addresses **runner pool + max-parallel alignment**.

---

## 2. Knobs (scale pool first, then parallel)

| Knob | Where | Default / notes |
|------|--------|-----------------|
| **`OYA_CI_RUNNER_LABELS`** | Repo or org **Actions variable** (JSON array) | Workflow fallback: `["ubuntu-latest"]`. Example self-hosted/Talos: `["self-hosted","linux","talos"]`. All jobs in `oya-ci-required.yml` use `runs-on: ${{ fromJson(vars.OYA_CI_RUNNER_LABELS \|\| '["ubuntu-latest"]') }}`. |
| **`OYA_CI_MAX_PARALLEL`** | Repo or org **Actions variable** (JSON number) | Workflow default: **`8`** via `max-parallel: ${{ fromJson(vars.OYA_CI_MAX_PARALLEL \|\| '8') }}` on the gate matrix strategy. Caps how many matrix legs of one PR run concurrently. |
| **ARC / self-hosted scale-set min/max** | Your Actions Runner Controller (or equivalent) scale-set / runner group | **Human cloud ops.** Raise **min** (or max + HPA) so the pool has headroom under multi-PR load. Labels must **match** `OYA_CI_RUNNER_LABELS`. |
| GitHub-hosted concurrency | Org/plan runner quota | If still on `ubuntu-latest`, queue_wait may be org-plan limited; switching labels to self-hosted does not auto-provision machines. |

**Policy order (forge-of-record):**

1. Ensure **real runners** exist and match labels (scale pool / ARC min–max).
2. Only then raise **`OYA_CI_MAX_PARALLEL`** so one PR can use available idle capacity.
3. Never set max-parallel above **real** concurrent capacity (starves other PRs / creates false “parallelism”).

ADR-0630 may describe ARC as an **interim** substrate — scale it for throughput; do **not** treat Proposed ADR-0630 as Accepted north-star law.

---

## 3. Checklist (measure → act → re-measure)

Do this as **human ops**. No cluster mutation from agents.

### A. Measure idle vs queued

- [ ] Pick 1–2 live PRs exercising `oya-ci-required` (multi-lane preferred).
- [ ] Note: count of jobs **Queued** vs **In progress**; time-to-first-runner for a sample gate job.
- [ ] On the runner pool / ARC scale-set: **idle runners** vs **busy** vs **desired replicas**.
- [ ] Record current `OYA_CI_RUNNER_LABELS` and `OYA_CI_MAX_PARALLEL` (Settings → Secrets and variables → Actions → Variables).

**Decision table:**

| Idle runners | Jobs queued | Action |
|--------------|-------------|--------|
| **0** (or pool below demand) | Yes | **Scale pool** (ARC min/max / more runners / fix labels). Do **not** raise max-parallel alone. |
| **> 0** | Yes | Labels mismatch **or** max-parallel too low **or** non-matrix jobs serial elsewhere. Fix labels first; if labels OK and pool idle, **raise `OYA_CI_MAX_PARALLEL` only up to real idle headroom**. |
| **> 0** | No | Not queue-bound; stop R1 thrash. Look at compute/path-filter (R2/R4). |

### B. Scale pool **or** raise max-parallel (only if idle > 0)

- [ ] **Scale:** increase ARC/self-hosted scale-set **min** (and **max** if HPA-capped); wait for Ready runners with labels matching `OYA_CI_RUNNER_LABELS`.
- [ ] **Or** if idle already > 0 and matrix is artificially capped: set `OYA_CI_MAX_PARALLEL` to a number **≤** observed stable concurrent runners available to this repo (leave headroom for other PRs).
- [ ] Do **not** delete or skip gates to “make green faster.”

### C. Re-measure wall clock on a PR

- [ ] Re-run or open a fresh push on a representative PR.
- [ ] Compare: queue time for gate jobs, wall clock to green/red `oya-ci-required`, concurrent *Running* count.
- [ ] Done when acceptance in PROGRAM is met (idle while pending is rare; P50 queue_wait down).
- [ ] Update `programs/SESSION-BACKLOG.md` (R1 status) when capacity change is live — **not** when only this runbook exists.

---

## 4. Hard stops

| Stop | Why |
|------|-----|
| **Do not delete gates** | Merge floor is ADR-0515 / `oya-ci-required` fan-in. Throughput ≠ weaker policy. |
| **Do not set max-parallel above real capacity** | Creates contention, noisy neighbors, and hides under-provisioning. |
| **Do not mix with CAS warm identity** | R1 is runner ops only. No CAS credentials, no RE, no cache `rw` flip, no #1541 work in this lane. |
| **Not merge authority** | Green faster still needs independent review + protected context. This runbook and any ops change are **not** promote authority. |
| **No agent cloud scale** | Agents do not mutate ARC/clusters/secrets or set org runner quotas. |
| **No coupling mega-PR** | Do not land “scale runners + rewire CI + activate CAS” in one change (`LANES.md` anti-patterns). |

---

## 5. Cross-references

| Ref | Role |
|-----|------|
| **Ultragoal G028** | ARC churn / protected-CI **queue capacity** — pending; ops + ADR-0630 Proposed interim; aligns with **R1 runners** |
| **`programs/AUTHORITY-AND-MINED-BACKLOG.md`** | Wave **0 ops**: R1 “Scale runners / max-parallel to pool → queue wait collapses”; mined item **G028 runner capacity** = highest short-term wall-clock ROI |
| **`PROGRAM.json` → R1-runner-capacity** | Objective, surfaces, acceptance, hard_stops, ultragoal_refs `G028-measure-arc-churn-and-resolve-protec` |
| **`docs/ci/forge-of-record.md`** | Official knob docs for labels + max-parallel |
| **`.github/workflows/oya-ci-required.yml`** | Live `runs-on` + `max-parallel` defaults |
| **ADR-0515** | Sole protected context `oya-ci-required` (do not replace with local CLI green) |
| **ADR-0630 (Proposed)** | Interim ARC substrate note only — scale without over-claiming Accepted law |

---

## 6. Who acts

| Actor | May |
|-------|-----|
| **Human ops / founder** | Read Actions/ARC metrics; change scale-set min/max; set `OYA_CI_RUNNER_LABELS` / `OYA_CI_MAX_PARALLEL`; re-measure; mark R1 done in backlog when capacity is real. |
| **Agent** | Draft/update this runbook and backlog **status lines** only. Diagnose from public PR check UI text if asked. **Cannot** scale your cloud, rotate runner creds, or claim G028/ultragoal complete. |
| **Nobody on this lane** | Delete gates, warm CAS, set RE, self-approve merges, or raise parallel past fleet size. |

**Unblocks (when done):** R5 CI wall-clock measurement validity and R2 work are less poisoned by queue noise — still separate lanes.

---

## Quick command hints (human)

```bash
# Inspect protected workflow defaults (local read-only)
rg -n 'OYA_CI_MAX_PARALLEL|OYA_CI_RUNNER_LABELS|max-parallel' .github/workflows/oya-ci-required.yml

# Example: list recent workflow runs for a PR (requires gh auth; human)
# gh pr checks <PR> --repo jason931225/oyatie
# gh run view <run-id> --repo jason931225/oyatie
```

Repo variables (UI): **Settings → Secrets and variables → Actions → Variables**  
— `OYA_CI_RUNNER_LABELS` = JSON array string  
— `OYA_CI_MAX_PARALLEL` = JSON number string (e.g. `12`)

---

*Last written for lane R1 docs/ops only. Scale still human.*
