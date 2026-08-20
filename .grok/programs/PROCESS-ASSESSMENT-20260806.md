# Process assessment — mm-delivery kit vs PR #1574 reality

**Date:** 2026-08-06  
**Scope:** multi-model delivery harness (`.grok/mm-*`) + live orchestration of ADR disposition PR #1574  
**Method:** Cartesian doubt + systems + operability + opportunity cost.  
**Static evaluate (kit shape only):** `mm-evaluate static` → letter **A** (93.33) — **does not measure runtime obedience**.

## Verdict

| Layer | Score (judgment) | Note |
|-------|------------------|------|
| Kit **design** (roles, drive, lenses, dual-critic doctrine) | **B+** | Coherent; Bun-inspired process-edit axiom present |
| Kit **presence on authority trunk** | **F** | Not on `origin/dev`; local untracked + side branch only |
| Kit **use in #1574 session** | **D** | Grok same-family subagents cosplayed dual-critic; `mm-role` unused |
| Delivery **outcome process** | **C** | Root CI causes fixed eventually; thrash + pause + laundered APPROVE |
| Continuous evaluation honesty | **C-** | Static A greened over runtime process failure |

**Headline:** The pipeline was built to force multi-model dual-critic + process edits. Agents (including this orchestrator) **bypassed** it with same-family subagents and generic PR babysit. Assessment without runtime fail-closed is a **false-green**.

## Evidence (know vs assume)

| Claim | Evidence | Class |
|-------|----------|--------|
| `.grok` not on `origin/dev` | `git ls-tree origin/dev -- .grok` → 0; `git ls-files .grok` → 0 | Know |
| Kit exists on side branch | `origin/agent/mm-harness-20260805` has ~74 kit files; tip includes `mm-drive`, `mm-role`, harness | Know |
| Local tree has richer kit | ~161 files untracked; programs/evidence/briefs accreted | Know |
| ADR worktree had no kit | `oyatie-adr-disposition-20260806/.grok` missing | Know |
| Dual-critic #1574 used subagents | Session: `spawn_subagent` Critic A/B; packets under `docs/decisions/_disposition/evidence/` | Know |
| `mm-role CRITIC` routes to Claude Opus | dry-run → `claude -p --model claude-opus-5` | Know |
| Static evaluate A | `mm-evaluate static` 20260806T103504Z | Know |
| CI push thrash | Multiple tip runs cancelled when next push landed before settle | Know |
| Passive wait then scold | User: “why have you paused… why aren't you babysitting” | Know |

## Failure classes observed (new + known)

| ID | Observed on #1574 | Severity | Close-gap (process, not chat) |
|----|-------------------|----------|-------------------------------|
| **F-KIT-NOT-ON-DEV** | Kit never authority substrate | **P0** | Land kit PR to `dev` (exclude `mm-runs/`, `memory/`); gate “kit present on base” |
| **F-SAME-FAMILY-CRITIC-LAUNDER** | Dual APPROVE without distinct providers | **P0** | Packet must declare `critics[].provider` + `independence`; merge-check rejects `same_family` when `require_cross_model_critics` |
| **F-WORKTREE-MISSING-KIT** | Disposition worktree had no `.grok` | **P0** | `mm-bootstrap` into worktree on lane start; PREFLIGHT fail if kit missing when `require_kit=true` |
| **F-CI-PUSH-THRASH** | Consecutive pushes cancelled in-flight `oya-ci-required` | **P1** | Cap pushes while tip CI in_progress; hold doc-only until settle or batch |
| **F-BABYSIT-WITHOUT-MM-DRIVE** | Generic pr-babysit / spawn loop, not `mm-drive tick` | **P1** | Babysit path = `mm-drive status/tick/merge-check`; journal under `mm-runs/` |
| **F-STATIC-EVAL-FALSE-GREEN** | Static A while runtime D | **P1** | `mm-evaluate run` requires drive ledger + dual-critic independence fields |
| **F-GROK-ONLY-WORKFLOW** (known) | Session model for “critics” | **P0** | Default critic path = `mm-role CRITIC` ×2 with lens packs |
| **F-NO-LIVE-CI-POLL** (partially closed) | Monitors exist ad hoc | **P2** | Prefer `mm-drive` re_query; document armed-repoll requirement (already in drive.v1) |

## What worked

1. **Doctrine** was right: not merge authority; oya-ci-required sole context; process-edit over symptom-only.  
2. **Local diagnosis** of empty apex deliverables → board-sync fail closed was correct and fixable.  
3. **mm-role / mm-evaluate bins run** on the developer machine when invoked.  
4. **Side branch** preserved a committable kit snapshot.

## What the pipeline should force (fine-tune target)

```text
PREFLIGHT (kit on tree + origin/dev base)
  → EXECUTOR in worktree (mm-bootstrap if needed)
  → push once for CI-relevant fix
  → waiting_ci + armed monitor (no idle_complete)
  → dual CRITIC via mm-role (distinct providers) + mm-lens-prompt
  → packet independence=cross_model | fail closed
  → oya-ci-required green on tip
  → mm-drive merge-check → merge (policy)
  → mm-packet R3 on promoted SHA
  → mm-score / mm-grade / mm-learn process_edits
```

Anything that short-circuits to “two Grok subagents wrote APPROVE JSON” must be **labeled and non-admit** under merge-check.

## Priority actions (ordered)

1. **Land kit on `dev`** from `agent/mm-harness-20260805` + local process_edits (this assessment wave), ignoring run journals.  
2. **Wire fail-closed dual-critic independence** in `drive.v1.json` + packet schema (done in this process_edit wave).  
3. **Worktree kit bootstrap** rule in PREFLIGHT/safety.  
4. **Push thrash budget** in drive policy.  
5. **Runtime evaluate** after next real PR wave using ledger + packets, not static-only.  
6. Finish #1574 CI under process rules (no further doc thrash while CI in flight).

## Assessment of assessment tools

| Tool | Useful for | Insufficient for |
|------|------------|------------------|
| `mm-evaluate static` | Manifest completeness, lenses, safety docs | Whether agents **used** the kit |
| `mm-role --dry-run` | Provider routing sanity | Live critic quality |
| Dual-critic JSON in PR | Audit trail if honest | Independence unless providers declared |
| GitHub Actions | Product truth (merge) | Process learning |

## Sign-off fields (fill after kit lands)

- [ ] Kit PR merged to `dev` SHA: ________  
- [ ] `mm-evaluate static` on clean worktree from `dev`: letter ≥ B  
- [ ] One PR wave with `mm-role` dual CRITIC + distinct providers  
- [ ] `mm-drive merge-check` rejects same_family packet when cross-model required  
