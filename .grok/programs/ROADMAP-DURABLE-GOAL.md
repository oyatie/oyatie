# Oyatie roadmap — durable goal (operator view)

**Brief (paste/create):** [`briefs/oyatie-roadmap-durable.brief.md`](briefs/oyatie-roadmap-durable.brief.md)  
**Purpose:** Long-horizon, parallelizable, closed-loop implementation of the Oyatie roadmap — not a session tasklist.

## Interview locks (2026-08-05)

| Knob | Choice |
|------|--------|
| Horizon | Full roadmap under **live** ADR/tip authority; **reorg may come first**; reject stale masterplan/ultragoal as sole law |
| Autonomy | **Bun zero-intervention** multi-critic agent loop (secrets/founder-ratify only human_blocked) |
| Parallelism | **Contract-lock → max path-disjoint** lanes |
| Durable state | **mm-goals + beads mirrored fail-closed** |

## Critical insight baked into the brief

Historical masterplan / ultragoal graphs are **provenance and queue signals**. Dispatch authority is:

1. Accepted ADRs on current `origin/dev`  
2. Live tree + open PR heads  
3. Then plans/backlogs if they still match  

**Plan lag** and **stale execute** both block work. Reorg is disposition (move|refactor|rewrite|delete|rebrand), not bulk move from old inventory.

## Goal spine (R0–R10)

```text
R0 authority freeze / stale-plan reconciliation
  → R1 reorg disposition (often first real work)
  → R2 admission fabric
  → R3 CAS ordered (cache-only ≫ RE)
  → R4 k8s port W0
  → R5 FD-001 contract lock
  → R6 FD-001 parallel product surfaces
  → R7 later verticals
  → R8 owned-stack deepening
R9 process intelligence (always-on parallel)
R10 human-blocked security/ops (non-thrash)
```

## Closed loop

```text
re-query → select_next_ready → trial → dual-critic fan-out →
merge → tip-sync → dual-SSOT checkpoint → score/grade/learn →
process_edit harness → repeat
```

## Activate

```bash
.grok/bin/mm-goals create --brief-file .grok/programs/briefs/oyatie-roadmap-durable.brief.md
# then mirror each @goal into bd with external_ref; drive via mm-drive tick
```

### Activation record (2026-08-05)

| Field | Value |
|-------|--------|
| **run_id** | `20260805T230336Z-a5ce047f` |
| **run_dir** | `.grok/mm-runs/20260805T230336Z-a5ce047f/` |
| **goals** | 11 (G001–G011 = R0–R10) |
| **active** | G001 R0 Authority freeze |
| **beads epic** | `oyatie-0vz` |
| **beads children** | `oyatie-0vz.1` … `oyatie-0vz.11` |
| **external_ref pattern** | `mm-goals:20260805T230336Z-a5ce047f:G00N` |
| **dual-ssot map** | `…/dual-ssot-beads.json` + `activation-receipt.json` |
| **drive** | `mm-drive status` + `tick` executed; ledger event `durable_roadmap_activated` |

Next implement step: **R0 authority-reconciliation receipt** (ADR-600+ census already measured on tip `c7f60a9db`: 13 Accepted / 25 Proposed).

See brief for full Shared constraints, DoD, hard stops, and per-goal text.


## Full ADR disposition (R0)

- Audit: ~448 ADRs; see `docs/decisions/_disposition/` after PR lands, or harness evidence `adr-full-disposition-audit.json`.
- **Do not mass-Accept** Proposed (160+ &lt;600, 25 ≥600).
- **Do not treat `status: Accepted` as-is** — live-resolve supersession + `amended_by` (brief §B).
- Safe bulk done separately: status case normalize.
- Queues: plan-lag (10 parents), missing status (26), superseded no successor (5), Proposed admission by class, stale-Accepted hygiene.
