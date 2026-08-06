# Delivery fabric (idea-refined)

## Problem statement

How might we keep multi-agent delivery **continuously productive** so that work discovery, implementation (via mm-*), PR babysitting, and meta-health never idle—without laundering same-family dual-critic or skipping TDD?

## Recommended direction

**Four always-on workflow classes** (minimum concurrent activity), sharing one **task board** SSOT and the **mm-delivery pipeline** for mechanical stages:

| Class | Workflow name | Cadence | Owns |
|-------|---------------|---------|------|
| **W1 Portfolio** | `portfolio-work-manager` | every 15–30m | Discover/claim lanes → `task-board.v1.json` + beads + hindsight |
| **W2 Implement** | `implement-claimed-lane` | every 10–15m | Claim `ready` slice → mm-pipeline path → PR |
| **W3 Babysit** | `pr-babysit-lanes` | every 5–10m | **Sole** open-PR babysit owner (single-flight) → CI fix / merge |
| **W4 Productivity** | `workflow-productivity-watch` | every 5–10m | Assert W1–W3 productive; re-arm; process_edits |
| **W5 North-star audit** | `northstar-portfolio-audit` | every 15–30m | **Separate** from W4: audit status/backlog/board vs `NORTH-STAR-SHAPE.md`; enqueue gaps |

**Invariant:** productivity-watch **fails closed** if fewer than **4 core** fabric classes (W1–W4) have a live/recent run or armed scheduler. Idle chat is not productivity.  
**W5** is always-on discovery against north-star debt; it does not replace W4 and does not babysit or implement.

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

## origin/dev awareness (always)

Every portfolio tick and reorg claim MUST know remote trunk:

| Tool | Output |
|------|--------|
| `mm-dev-status --fetch` | `origin/dev` sha, subject, oya-ci on tip, open PRs behind dev, local behind count |

**Defects:** stale execute (implement without re-query tip); PR lag ignored; reorg from dirty primary 628 commits behind.

Board field: `origin_dev: { sha, sha12, at }`.

## Reorg / rewrite / debrand / remove (long-overdue, first-class)

Not optional cleanup. Enqueued by **`mm-reorg-enqueue`** from `REORG-REBRAND-BACKLOG.md` + doctrine.

| Class | Examples on board |
|-------|-------------------|
| refactor | RR-FACE-DECOMMIT, RR-MOVEPLAN-SINGLETON |
| rebrand | RR-BRAND-0619 |
| mixed | RR-DUAL-0615-FOLLOW, RR-LIBS-DISPOSITION |
| move | RR-CAS-3A (blocked until G039 prereq) |
| delete/rewrite | later waves W3–W4 |

W2 implement prefer order: **ci_red → soft_red → reorg W0/W1 ready → beads → console**.  
Every reorg PR: worktree from **current** `origin/dev`, ADR re-query, one concern.

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
- Open PRs not abandoned (W3 single-flight claim per `(pr,head)`; see `BABYSIT-SINGLE-FLIGHT.md`)  
- Implement never multi-polls CI after handoff (`ready_for_babysit`) 
- Merge when agent APPROVE + CI green without waiting on human  
- Process_edits when systematic failures  


## Workflow files

- `.grok/workflows/portfolio-work-manager.rhai`
- `.grok/workflows/implement-claimed-lane.rhai`
- `.grok/workflows/pr-babysit-lanes.rhai`
- `.grok/workflows/workflow-productivity-watch.rhai`

Run: `/workflow portfolio-work-manager` (etc). Schedulers re-arm every 5–15m.
