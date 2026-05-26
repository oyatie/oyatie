---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M01-P10
title: Purpose-Discipline + Orphan-Detection
status: complete
purpose: Every artifact declares purpose; orphans are deletion targets.
---

# M01-P10 — Purpose-Discipline + Orphan-Detection

## Purpose
Per MASTERPLAN §2 Directive 10. No "we might need this later" files.

## Acceptance
- Every doc/JSON/README has `purpose:` frontmatter or top-line declaration.
- Every directory has INDEX.md or README.md declaring purpose.
- `oya-governance-orphan-detection` lane CI-blocks orphan artifacts (no inbound reference + not in known-orphan allowlist).

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | Purpose-frontmatter authoring (audit every existing doc/JSON) | split-required-too-broad-for-single-changeset | [`IP-001-purpose-frontmatter-audit.md`](IP-001-purpose-frontmatter-audit.md) |
| IP-001.1 | Purpose-frontmatter audit (Part 1: Root & Core Docs) | planned | [`IP-001.1-audit-root-core.md`](IP-001.1-audit-root-core.md) |
| IP-001.2 | Purpose-frontmatter audit (Part 2: Milestone Plans) | planned | [`IP-001.2-audit-milestones.md`](IP-001.2-audit-milestones.md) |
| IP-001.3 | Purpose-frontmatter audit (Part 3: JSON Registries/Specs) | planned | [`IP-001.3-audit-json.md`](IP-001.3-audit-json.md) |
| IP-002 | Orphan-detection lane kernel | complete | [`IP-002-orphan-detection-lane.md`](IP-002-orphan-detection-lane.md) |

## Estimated parallelism
2 agents (audit is fan-out-able by directory; lane is single-author).

## Symbols-touched
All `docs/**/*.md`, all `**/*.json`, `crates/oya-governance-orphan-detection-kernel`.

## Agent-handoff
```
icm store -t context-oyatie -c "M01-P10 complete: purpose-frontmatter on every artifact; orphan-detection lane green" -i critical -k "M-CC,P03,purpose,orphan-detection,complete"
```
