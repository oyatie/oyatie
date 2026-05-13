---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P05
title: Cell Architecture + Plane Separation Enforcement
status: stub
purpose: Ship the cell-routing primitive and the plane-separation enforcement that every catalog record declares.
---

# M01-P05 — Cell Architecture + Plane Separation Enforcement

## Purpose
Per ADR-0009 (cell architecture) and ADR-0017 (plane separation). Every surface declares its plane in `registry/catalog/<crate>.yaml: plane:`; cross-plane calls are explicit contracts.

## Acceptance
- `crates/oya-platform-cell-*` cell-routing primitive shipped.
- Plane assignment validated in CI; PRs changing plane class trigger cross-plane review.
- Cell-isolation evidence collected per cell.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Cell-routing primitive (`oya-platform-cell-kernel`) | stub | [`IP-001-cell-routing-primitive.md`](IP-001-cell-routing-primitive.md) |
| IP-002 | Plane separation enforcement lane | stub | [`IP-002-plane-separation-lane.md`](IP-002-plane-separation-lane.md) |

## Estimated parallelism
2 agents; disjoint crate suffix.

## Symbols-touched
`crates/oya-platform-cell-*`, `crates/oya-foundry-fitness-plane-separation-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P05 complete: cell-routing primitive + plane separation lane green" -i critical -k "M01,P05,cell,plane,complete"
```
