# Delivery fabric (idea-refined)

## Problem statement

How might we keep multi-agent delivery **continuously productive** so that work discovery, implementation (via mm-*), PR babysitting, and meta-health never idle—without laundering same-family dual-critic or skipping TDD?

## Recommended direction

**Four always-on workflow classes** (minimum concurrent activity), sharing one **task board** SSOT and the **mm-delivery pipeline** for mechanical stages:

| Class | Workflow name | Cadence | Owns |
|-------|---------------|---------|------|
| **W1 Portfolio** | `portfolio-work-manager` | every 15–30m | Discover/claim lanes → `task-board.v1.json` + beads + hindsight |
| **W2 Implement** | `implement-claimed-lane` | every 10–15m | Claim `ready` slice → mm-pipeline path → PR |
| **W3 Babysit** | `pr-babysit-lanes` | every 5–10m | Open PRs (board + fleet) → CI fix loop |
| **W4 Productivity** | `workflow-productivity-watch` | every 5–10m | Assert W1–W3 productive; re-arm; process_edits |

**Invariant:** productivity-watch **fails closed** if fewer than 4 fabric classes have a live/recent run or armed scheduler. Idle chat is not productivity.

## Key assumptions

- [ ] Beads + board dual-write stays consistent (claim on board + `bd update --claim`)
- [ ] Path-disjoint lanes prevent worktree thrash
- [ ] Kit on tree (ideally `dev` via #1575) so mm-pipeline/mm-role exist
- [ ] Cross-model critics when merging (not same-family launder)

## MVP scope

1. Four `.rhai` workflows + task board schema/file  
2. `mm-fabric-status` + drive.v1 fabric section  
3. Schedulers keeping all four armed  
4. Implement path forced through CAPTURE → … → RED → IMPLEMENT (mm-pipeline)

## Not doing (now)

- Nested workflows (Rhai can't launch workflows) — composition is **schedulers + board**  
- Auto-merge without oya-ci-required  
- Auto-promote learn packs without human gate  
- Secrets / #1541 thrash  

## Soft reds & blocks (never silent)

GHA may mark soft platform legs `continue-on-error`, but **process still queues them**.

| Event | Board `source` | Priority | Drain by |
|-------|----------------|----------|----------|
| Check name contains `soft` and FAILURE | `soft_red` | P2 | babysit / implement claim |
| Binding check FAILURE | `ci_red` | P1 | babysit immediately |
| mergeable CONFLICTING / DIRTY / BEHIND / CHANGES_REQUESTED | `block` | P1–P2 | babysit |
| CANCELLED tip thrash | `block` / `ci_cancelled` | P3 | note + re-arm CI |

**Tool:** `.grok/bin/mm-queue-ingest` (run every portfolio + babysit tick).  
**Clear:** check no longer red on tip **or** lane `status=waived` with written reason (human if awry).

Silent ignore of soft reds is a **process defect**.

## Autonomy (human supervises only)

| Gate | Action |
|------|--------|
| Dual-critic **APPROVE** (packet) + `oya-ci-required` SUCCESS + merge-check ok | **`mm-drive merge --pr N`** — no human GH APPROVE |
| Dual-critic non-APPROVE / missing | **resolvable** — fix code/review until APPROVE (not human_blocked) |
| Soft red / block on board | **ready lane** — must be claimed or fixed, not dropped |
| Human | Intervene only if something looks awry |

## Success

- ≥4 fabric classes active (scheduler or run < max_stale_min)  
- Board always has ≥1 `ready` **or** explicit empty after ingest with `queue_stats.soft_red=0`  
- **Zero silent soft reds** — ingest every tick  
- Open PRs not abandoned (babysit re-poll armed)  
- Merge when agent APPROVE + CI green without waiting on human  
- Process_edits when systematic failures  
