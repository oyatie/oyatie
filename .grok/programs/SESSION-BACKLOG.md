# Session + handoff backlog (durable)

**Updated:** 2026-08-05 (parallel drive start — RE readiness + reorg mine)  
**Purpose:** Single ledger of directives and tasks from **#1560 CAS handoff**, **k8s W0-A handoff**, and **this Grok session**.  
**Not merge authority.** Ultragoal aggregates remain **not completed** unless quality-gated.

**Authority + mined ultragoal:** `programs/AUTHORITY-AND-MINED-BACKLOG.md`  
(ADR &lt; 600 still valid if no contradicting newer ADR; if contradicted → **amend/supersede**, don’t ignore; Bun waves.)

**Related SSOT**

| Surface | Role |
|---------|------|
| `programs/hyperscaler-delivery-lanes/` | R1–R9 separated lanes |
| `programs/cas-fabric/` | #1560 / G039 / #1559 / #1541 |
| `programs/cas-fabric/evidence/RE-SANDBOX-READINESS.md` | R6/G043 plan-only RE readiness (no activation) |
| `programs/REORG-REBRAND-BACKLOG.md` | Ranked reorg/rebrand/rewrite cards (P8) |
| `programs/MASTER-PARALLEL-DRIVE.md` | Full backlog fan-out sequencing |
| `programs/k8s-port/` | G001 / #1561 |
| `bin/mm-drive status` | Live resolvable vs human_blocked |
| beads `oyatie-7xf` | k8s W0-A |
| kit `tasks/todo.md` | kit phases (secondary) |

---

## Global directives (always on)

| ID | Directive | Source | Status |
|----|-----------|--------|--------|
| D-NO-OMC | No `gjc` / `omc` / `omx` / `hermes` CLIs — ideas only | User + kit | **Active** |
| D-SINGLE-PIPE | One mm-delivery pipeline; no per-lane Rhai product sprawl | User | **Active** |
| D-NOT-MERGE | Merge via `oya-ci-required` + **agent dual-critic** (human GH APPROVE not mandatory) | AGENTS / ADR-0515 + drive.v1 merge_policy | **Active** |
| D-NO-FALSE-UG | Never false-complete ultragoal / G039 / G001 without quality gates | Handoff + user | **Active** |
| D-PRESERVE | Primary `agent/cas-live-proof-20260804` is preservation only | #1560 handoff | **Active** |
| D-LANES | Separate concerns; no coupling runner+CAS+k8s in one PR | This session | **Active** |
| D-CAS-BEFORE-RE | CAS/AC cache-only before RE | Approved plan #1560 | **Active** |
| D-PRE-POST | Pre-merge admission ≠ post-merge packet/writers | This session | **Active** |
| D-SHORT-RUNNERS | Short-term wall-clock: unlock more CI runners when queue-bound | This session | **Active** |

---

## Handoff #1560 (CAS fabric)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| H1560-PLAN | Plan SHA `8833df33…` verified | **Done** | evidence/approved-plan |
| H1560-RECOVERY | Recovery `fd2c9a52…` quarantine only | **Done** (track) | not implement base |
| H1560-PRESERVE | 1739/1780 preserve; 41 exclusions | **Done** (handoff) | re-scan before wipe claims |
| H1560-TALOS-ARCH | iCloud archives + checksums OK | **Done** (handoff) | #1541 still open |
| H1560-1559 | PR #1559 → `a4a5ace5` | **Done** | post-merge green + packet |
| H1560-G038 | Baseline reconciliation | **Done** (prior) | snapshots under evidence |
| H1560-G039 | Representative #1558 pilot | **MERGED** `a1bd1f14a` | post-merge CI + G039 packet DRAFT; not ultragoal-complete |
| H1560-G040+ | Later CAS program lanes | **Blocked** | after G039 terminal → 3A/3B/3C |
| H1560-1534 | Cache-only / CAS activation story | **Open** | after pilot + #1541 → G041 |
| H1560-1541 | Talos credentials | **Open** | R9; blocks warm CAS / G041 / RE |
| H1560-1549 | RE sandbox | **Open / plan-only ready** | R6: `RE-SANDBOX-READINESS.md`; **not authorized** (0612 Proposed); after G041 + measured reopen |
| H1560-NO-COUPLE-1561 | Do not couple #1561 to G039 | **Active** | R7 separate |
| H1560-REORG-MINE | Reorg/rebrand debt cards | **Done (mine)** | `programs/REORG-REBRAND-BACKLOG.md` — execute per card, not this row |

### G039 terminal (either)

1. Pilot promoted (review + squash + promoted proof packet + checkpoint), **or**  
2. Pilot superseded with reviewed rationale  

**Now:** #1558 **MERGED** `a1bd1f14a`; G039 packet **DRAFT** — fill after trunk `oya-ci-required` green on promoted SHA; ultragoal still incomplete.

---

## Handoff k8s W0-A (#1561 / G001)

| ID | Item | Status |
|----|------|--------|
| H1561-SOURCE | ADR-0637/0638 + specs + R-DOC | **Done** |
| H1561-LOCAL-GATES | Focused gates at handoff | **Done** |
| H1561-ARCH-REVIEW | Architect APPROVE (not GH formal) | **Done** |
| H1561-PR | PR #1561 head `1e33500d` | **Done** |
| H1561-CI | oya-ci-required SUCCESS | **Done** |
| H1561-FORMAL-GH | Agent dual-critic APPROVE (human not mandatory) | **Done** on head `c744a2f45` |
| H1561-MERGE | Squash to `dev` | **Open / agent when CI green** |
| H1561-PACKET | Post-merge packet | **Draft only** (`G001-post-merge-packet-DRAFT.md`) |
| H1561-BEADS | Close `oyatie-7xf` | **Open** (after packet) |
| H1561-G001-CP | Ultragoal G001 checkpoint | **Open** |
| H1561-G002+ | W0-B…W0-H | **Sequenced** |
| H1561-W1 | W1+ | **Unapproved** |

---

## This session — kit + drive

| ID | Item | Status |
|----|------|--------|
| S-KIT-PHASE0-1 | Foundation; evaluate ≥ A | **Done** |
| S-KIT-PARALLELISM | parallelism + mm-paths | **Done** |
| S-KIT-3.1 | KPI repeat / mm-learn | **Done** |
| S-KIT-QUANT | mm-quant + mm_bridge + learn-hook | **Done** |
| S-KIT-DRIVE | mm-drive + Stop hook MVP | **Done** (D0–D2) |
| S-KIT-D3 | drive ↔ goals checkpoint-check | **Backlog** |
| S-KIT-D4 | briefs + path-overlap | **Backlog** |
| S-KIT-D5 | scheduler /loop docs | **Backlog** |
| S-KIT-D6 | live Stop-hook trial + hooks-trust | **Backlog** |
| S-KIT-1.2 | Console bootstrap | **Human gate** |
| S-KIT-2.2 | Parallel critic metrics | **Backlog** |
| S-KIT-5.2 | Docs consolidation | **Backlog** |
| S-NO-SPRAWL | Retire parallel-lane Rhai product | **Done** |
| S-R4-TIP | local-prepush-new-yaml tip | **Done** |
| S-LANE-BOARD | hyperscaler-delivery-lanes R1–R9 | **Done** |

---

## This session — delivery / ops

| ID | Item | Status |
|----|------|--------|
| S-1559-PACKET | #1559 completion packet | **Done** |
| S-1558-G002 | Diagnose red gates | **Done** |
| S-1558-G003 | OWNERS+Buck+reachability+rebase | **Done** (code) |
| S-1558-CI | oya-ci-required green | **Done** |
| S-1558-REVIEW | Agent dual-critic + merge | **Done merge** |
| S-1558-G039 | Merge + promoted proof | **MERGED; packet DRAFT awaiting trunk green** |
| S-1561-FIX | CI reds cleared | **Done** |
| S-1561-READY | Ready + DoD + draft packet | **Done** |
| S-1561-MERGE | dual-critic + squash | **Open / agent when CI green** |
| S-1541-STATUS | Status JSON only | **Done** |
| S-R1-RUNNERS | Unlock more CI runners | **Open / human ops** — R1 runbook landed, scale still human |
| S-R2-PREMERGE | Path-filter live-postgres | **#1562 OPEN** rebased `adfad9eaa`; dual-critic APPROVE; waiting CI |
| S-R3-POSTMERGE | Trunk packet automation | **Open** (template landed; automation separate PR) |
| S-R6-RE-READY | RE sandbox plan-only readiness (G043) | **Done (docs)** — `cas-fabric/evidence/RE-SANDBOX-READINESS.md`; implement **blocked** |
| S-REORG-MINE | Mine reorg/rebrand/rewrite debt | **Done (mine)** — `programs/REORG-REBRAND-BACKLOG.md` ranked cards |
| S-PARALLEL-DRIVE | MASTER parallel drive start status | **Active** — see agent-driving + MASTER-PARALLEL-DRIVE.md |

---

## Human-blocked (agent must not thrash)

1. **R1** — runner capacity (ops scale)  
2. **R9 #1541** — security credentials  
3. **Aggregate ultragoal complete** — only after program terminals + quality gates  

## Agent-driving now (not human-blocked) — parallel drive start

1. **P0 merge babysit** — #1561 / #1562 when `oya-ci-required` green + dual-critic already APPROVE  
2. **R5/G039** — fill post-merge packet after trunk green (not ultragoal-complete until packet + quality gates)  
3. **R2 #1562** — dual-critic APPROVE; undraft+merge when CI green on rebased head  
4. **W0-B plan** — admission plan only until G001 packet complete  
5. **P6 R6 RE** — **docs only** — readiness landed; **no** `remote_enabled` / scheduler / ARC-as-worker  
6. **P8 reorg/rebrand** — pick path-disjoint cards from `REORG-REBRAND-BACKLOG.md` (W0 first); one move-plan at a time  
7. **P2 CAS 3A** — only after G039 terminal; not before  
8. **Kit D3–D4** — optional, non-merge-authority  

---

## Agent-executable when resolvable

- CI red on #1561 / #1562 → fix **that** worktree only  
- R2/R3 / reorg cards → **new** worktree from `origin/dev`  
- Kit D3–D6 with `MM_DRIVE_KIT=1`  
- R6: update readiness / measurement design only  
- **Never** R6 RE activation or R9 secrets automation  
- **Never** warm CAS without #1541 close + G041

---

## Evidence index

| Path | What |
|------|------|
| `cas-fabric/INHERIT.md`, `PROGRAM.json` | #1560 inherit |
| `cas-fabric/evidence/G039-DIAGNOSIS-1558.md` | G002 |
| `cas-fabric/evidence/1559-post-merge-completion-packet.json` | #1559 **example filled** post-merge packet (COMPLETE) |
| `hyperscaler-delivery-lanes/R3-postmerge-packet-template.md` | **R3** post-merge product-completion packet template (fill only after squash to `origin/dev`; PR-head green ≠ packet) |
| `cas-fabric/evidence/1541-status-20260805.json` | #1541 |
| `cas-fabric/evidence/RE-SANDBOX-READINESS.md` | R6/G043 plan-only; not RE auth |
| `programs/REORG-REBRAND-BACKLOG.md` | Ranked reorg/rebrand cards |
| `programs/MASTER-PARALLEL-DRIVE.md` | Parallel lane fan-out |
| `k8s-port/INHERIT.md`, `G001-post-merge-packet-DRAFT.md` | W0-A draft only (not COMPLETE) |
| `hyperscaler-delivery-lanes/LANES.md` | R1–R9 |
| kit `harness/DRIVE.md`, `bin/mm-drive` | Outer drive |
| kit `memory/tips/local-prepush-new-yaml.md` | R4 |

---

## Ultragoal honesty

| Program | Aggregate complete? |
|---------|---------------------|
| CAS fabric (#1560) | **No** — G039 not terminal |
| k8s W0 | **No** — G001 not checkpointed |
| mm-delivery kit | **No** — continuous improvement |

---

## Maintenance rule

On every material state change (CI, review, merge, new lane work): update **this file** and the relevant `PROGRAM.json`. Chat is not the ledger.

## Autonomous merge policy (session directive)

| Rule | Value |
|------|--------|
| When | Independent formal APPROVE + `oya-ci-required` SUCCESS + not draft + mergeable + no CHANGES_REQUESTED + no unresolved threads |
| How | `mm-drive merge-check --pr N` then `mm-drive merge --pr N` (squash) |
| Self-approve | **Never** counts |
| After merge | R3 packet on **promoted** SHA only |
| #1558 | **MERGED** — G039 packet after trunk green |
| #1561/#1562 | Dual-critic APPROVE on recorded heads; merge when CI green + merge-check ok |


## Update 2026-08-05T16:15Z (public GHA + open-pr-fleet)

- Repo **PUBLIC**; free GitHub-hosted CI proven.
- Workflow **open-pr-fleet** owns open PRs 1566–1573.
- **#1573** head `ed46736f6` (hosted multi-arch + RUSTUP_HOME + gate_registration); dual-critic APPROVE; CI in progress.
- Rebased onto `origin/dev`: 1566, 1567, 1568, 1569, 1570, 1572.
- **#1569** MERGEABLE after ADR index conflict resolve (0630/637/638/639 retained).
- Next: merge #1573 when oya-ci-required green; then fan-out merge remaining.

## Goal loop 2026-08-05T16:22Z
- PUBLIC repo; free GHA.
- #1573 head `0d6e6ec93`: lifecycle doc_status fix for idea one-pager; CI in progress run 31024149019.
- Dual-critic APPROVE re-headed. Soft windows fail non-binding.
- Next: merge #1573 when oya-ci-required green; open-pr-fleet drain.
