---
doc_class: Program-Design-Note
doc_status: published
authority_tier: 3
purpose: |
  Wave4 design note: live presubmit cancel-in-progress (ADR-0639 D6)
  + Tide/merge_group isolation for scarce-runner queue hygiene.
---
# Wave4 design note: cancel-in-progress + Tide / merge_group

**Status:** prep complete (workflow citation + Tide note under docs/programs; dual-critic under evidence/ci) — behavior already live on `dev`  
**Date:** 2026-08-05  
**Bead:** `oyatie-oso.26`  
**Authority:** ADR-0639 **D6** (PR #1569; Accepted on branch, not yet on `dev` trunk)  
**Plan:** `PLAN-CI-PATH-TIER-AND-CAPACITY.md` § Wave 4  
**PR:** [#1570](https://github.com/jason931225/oyatie/pull/1570) (comment/citation + accounting registration)
**Dual-critic:** APPROVE (see `evidence/ci/pr-1570-dual-critic.json`)

---

## 1. Gate for this prep

Homes: design note lives under allowlisted `docs/programs/` (not `.grok/`, which is outside
`allowed_root_dirs` in `root-workspace-hygiene-policy.json`). Dual-critic packets live under
`evidence/ci/` (owned + reachability-covered by the `evidence/` prefix).


Wave4 prep was allowed because **PR #1569 exists** (ADR-0639 on branch).  
Hard rule from dispatcher: do **not** path-filter other jobs; separate from Wave3 postgres (#1562).

---

## 2. What is already live on `origin/dev`

`origin/dev` already implements the functional Wave4 concurrency shape (pre-ADR citation):

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.event.pull_request.number || github.sha }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}
```

| Property | Live value | Why |
|----------|------------|-----|
| PR group key | `workflow + PR number` | Superseded heads share one group |
| Non-PR group key | `workflow + github.sha` | Per-commit isolation (#1509 trunk fix) |
| cancel-in-progress | **true only** for `pull_request` | Drain scarce runners on force-push / new commits |
| cancel-in-progress | **false** for `push` / `merge_group` / `workflow_dispatch` | Preserve full surface-all integration evidence |

Historical fixes that already land this:

- **#886** — cancel superseded PR required runs  
- **#1330** — drop `pull_request` `edited` trigger (same-SHA double-fire cancel thrash)  
- **#1509** — non-PR arm uses `github.sha` not `github.ref` (trunk pending-eviction)

**Wave4 code delta is therefore citation + Tide safety docs**, not a behavior flip of `cancel-in-progress: false → true`.

---

## 3. ADR-0639 D6 (authoritative wording)

From ADR-0639 on `agent/adr-0639-ci-path-tier-20260805`:

> **D6 — Cancel-in-progress.** The workflow MAY use a per-PR concurrency group with `cancel-in-progress: true` so superseded heads do not hold scarce runners. Cancel of a superseded attempt is not a merge-green substitute for a later successful run on the current head.

Related algebra (**D4**): a leg `cancelled` reddens unless it is explicit whole-run supersession; admission still requires **SUCCESS** of `presubmit` on the **current** head SHA.

---

## 4. Tide / merge_group interaction (acceptance docs)

### 4.1 Event matrix

| Event | Typical `github.ref` | Concurrency group | `cancel-in-progress` | Notes |
|-------|----------------------|-------------------|----------------------|-------|
| `pull_request` (opened/reopened/synchronize) | `refs/pull/N/merge` | `presubmit-<PR#>` | **true** | Newer head cancels older in-progress/queued for same PR |
| `push` to `dev` | `refs/heads/dev` | `presubmit-<sha>` | false | Each trunk commit own group (#1509) |
| `merge_group` (`checks_requested`) | merge-queue temp ref | `presubmit-<sha>` | false | Tide validation never cancelled by PR pushes |
| `workflow_dispatch` | dispatch ref | `presubmit-<sha>` | false | Manual re-run isolation |

### 4.2 Why PR push cannot cancel Tide (and vice versa)

On `merge_group`, `github.event.pull_request.number` is **empty**.  
Group expression therefore falls through to `github.sha` (the merge_group head SHA).

On `pull_request`, the first operand is the PR number, so the group is **not** the head SHA.

Therefore:

1. An open-PR `synchronize` run **cannot** cancel an in-flight Tide `merge_group` validation.  
2. A Tide `merge_group` run **cannot** cancel open-PR work for that PR number.  
3. Two different PRs never share a cancel group (different PR numbers).  
4. Two trunk pushes never share a cancel group (different SHAs).

### 4.3 GitHub pending-eviction still applies within a group

Even with `cancel-in-progress: false`, GitHub admits at most one RUNNING + one PENDING per concurrency group and **evicts older PENDING** when a newer run joins the same group. That was the trunk self-cancel class fixed by #1509 (constant `refs/heads/dev` group).

`merge_group` refs are unique per queue entry; combined with sha keys, Tide is not subject to that trunk-burst eviction class.

### 4.4 Merge-green contract

| Outcome on a SHA | Admission effect |
|------------------|------------------|
| Superseded run `cancelled` | No effect on later head |
| Current head `presubmit` SUCCESS | Eligible for merge (with other BP rules) |
| Current head cancelled / failure | Blocks |

Agents must not treat cancelled check-runs as “CI red logic failure” without checking whether a newer head already re-ran.

---

## 5. Scope boundaries (YAGNI)

| In scope Wave4 | Out of scope |
|----------------|--------------|
| Cite ADR-0639 D6 on concurrency block | Path-filters (Wave3 / #1562) |
| Document Tide isolation | Dual-worker apply (Wave2 / #1564 + human) |
| Dual-critic of citation PR | Changing cancel predicate or group keys |
| Beads `oyatie-oso.26` | Warm CAS / RE / multi required contexts |

**Do not** “simplify” group back to `github.ref` — reopens measured trunk pending-eviction.

**Do not** set `cancel-in-progress: true` globally — would cancel in-flight Tide / trunk evidence.

---

## 6. PR train dependency

```
#1569 ADR-0639 (docs)  ── in flight; D6 authority for citation
#1564 dual-worker git  ── capacity (orthogonal)
#1562 postgres path    ── Wave3 (orthogonal; no coupling)
#1570 cancel docs      ── Wave4 citation; safe to land after or with 1569
```

Prefer merge **#1569 before or with #1570** so trunk comments cite an ADR already on `dev`. Draft #1570 may stay draft until #1569 merges if reviewers require trunk ADR presence; behavior is already live either way.

---

## 7. Verification checklist

- [x] Gate: PR 1569 exists → prep allowed  
- [x] `origin/dev` concurrency already cancel-in-progress for PR only  
- [x] Minimal worktree from `origin/dev`: branch `agent/ci-cancel-in-progress-20260805`  
- [x] Workflow change = comments only; jobs/path-filters untouched  
- [x] Draft PR #1570 open  
- [x] Dual-critic APPROVE (evidence on branch; head pin commits after workflow tip)  
- [ ] Merge when #1569 policy + presubmit green on exact head (prefer after #1569)  

---

## 8. Bun / queue hygiene lesson

Scarce runners + always-on A-always legs → superseded PR heads must not occupy the queue.  
Cancel-in-progress is **queue hygiene**, not admission. Capacity still needs dual-worker apply (#1564) and optional P-path legs (#1562) under ADR-0639 D2–D4.
