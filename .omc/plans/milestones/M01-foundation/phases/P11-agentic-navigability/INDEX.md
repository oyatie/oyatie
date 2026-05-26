---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P11
title: Agentic-Dev Optimization (Navigability Lanes)
status: complete
purpose: Predictable naming + machine-readable directory indexes + real grit-claim symbols in every IP.
---

# M01-P11 — Agentic-Navigability

## Purpose
Per MASTERPLAN §2 Directive 10. Fresh agent should navigate the tree without orchestrator hand-holding.

## Acceptance
- `oya-governance-agentic-navigability` lane CI-blocks: missing INDEX.md, missing frontmatter parent-pointer, IP without real `file::Identifier` grit-claim symbols, undeclared purpose.
- Predictable-naming lane: every file matches canonical pattern (no random suffixes / version bumps in filenames).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Agentic-navigability lane kernel + parent-pointer validator | complete | [`IP-001-navigability-lane.md`](IP-001-navigability-lane.md) |
| IP-002 | Predictable-naming convention enforcement | complete | [`IP-002-predictable-naming.md`](IP-002-predictable-naming.md) |

## Estimated parallelism
2 agents.

## Symbols-touched
`crates/oya-governance-agentic-navigability-kernel`, `crates/oya-governance-predictable-naming-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P11 complete: agentic-navigability + predictable-naming lanes green" -i critical -k "M-CC,P04,agentic-navigability,complete"
```
