---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M06-P03
title: Per-Vertical Regulatory Binding
status: stub
purpose: Bind each of the 13 verticals to its KR regulatory pack subset + DPIA.
---

# M06-P03 — Per-Vertical Regulatory Binding

## Purpose
Per [`../../../../../docs/PRIVACY-PROGRAM.md`](../../../../../docs/PRIVACY-PROGRAM.md) §2.5. Each vertical has its own DPIA + per-vertical control evidence.

## Acceptance
- Per-vertical DPIA on file at `regional-packs/kr/dpia/<vertical>.md`.
- Per-vertical regulator binding evidence collected.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | DPIA template + per-vertical authoring (13 verticals) | stub | [`IP-001-dpia-per-vertical.md`](IP-001-dpia-per-vertical.md) |
| IP-002 | Per-vertical regulator binding evidence | stub | [`IP-002-per-vertical-regulator.md`](IP-002-per-vertical-regulator.md) |

## Estimated parallelism
13 agents in parallel (one per vertical), 2-track (DPIA + regulator binding).

## Symbols-touched
`regional-packs/kr/dpia/`, `crates/oya-ops-compliance-per-vertical-binding-app`.

## Agent-handoff
```
icm store -t context-oyatie -c "M06-P03 complete: 13 vertical DPIAs on file; per-vertical regulator binding evidenced" -i critical -k "M06,P03,per-vertical-regulatory,complete"
```
