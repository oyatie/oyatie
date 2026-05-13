---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P03
title: Purpose-Discipline + Orphan-Detection
status: stub
purpose: Every artifact declares purpose; orphans are deletion targets.
---

# M-CC-P03 — Purpose-Discipline + Orphan-Detection

## Purpose
Per MASTERPLAN §2 Directive 10. No "we might need this later" files.

## Acceptance
- Every doc/JSON/README has `purpose:` frontmatter or top-line declaration.
- Every directory has INDEX.md or README.md declaring purpose.
- `oya-foundry-fitness-orphan-detection` lane CI-blocks orphan artifacts (no inbound reference + not in known-orphan allowlist).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Purpose-frontmatter authoring (audit every existing doc/JSON) | stub | [`IP-001-purpose-frontmatter-audit.md`](IP-001-purpose-frontmatter-audit.md) |
| IP-002 | Orphan-detection lane kernel | stub | [`IP-002-orphan-detection-lane.md`](IP-002-orphan-detection-lane.md) |

## Estimated parallelism
2 agents (audit is fan-out-able by directory; lane is single-author).

## Symbols-touched
All `docs/**/*.md`, all `**/*.json`, `crates/oya-foundry-fitness-orphan-detection-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P03 complete: purpose-frontmatter on every artifact; orphan-detection lane green" -i critical -k "M-CC,P03,purpose,orphan-detection,complete"
```
