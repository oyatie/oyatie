---
purpose: Auto-backfilled purpose for INDEX.md
---

---
doc_class: PhaseIndex
parent: ../../INDEX.md
id: M-CC-P01
title: Agentic-Pipeline Cutover (grit/icm SoT)
status: complete
purpose: Lift `ralplan-oyatie-sst-consolidation.md` 12-phase plan into the milestone tree; foundational for every other agent operation.
---

# M-CC-P01 — Agentic-Pipeline Cutover

## Purpose
Per [`../../../ralplan-oyatie-sst-consolidation.md`](../../../ralplan-oyatie-sst-consolidation.md) (iter-2 mid-consensus). The grit/icm cutover IS this phase. Foundational for every other phase in the masterplan.

## Acceptance
- All ten A-criteria GREEN per [`../../../ralplan-oyatie-sst-consolidation.md`](../../../ralplan-oyatie-sst-consolidation.md) §4.
- ADR-0052 (inventory), ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim) all Accepted.
- Three lanes green on `main`: banned-primitives (revised per Directive 12 — documented genuine need permitted), archive-orphan, authoritative-tracked.


## Foundation-cleared evidence

P01 foundation cleared on 2026-05-14 after IP-009, IP-010, and IP-012 code-review approvals. Standalone P01 lanes are green: banned-primitives, archive-orphan, authoritative-tracked, plus the IP-010 parallel-claim demo regression. Fresh full-workspace closeout now passes `./scripts/check.sh` under Rust 1.95.0 / edition 2024 / rustfmt 2024, including cargo check/clippy, cargo-nextest 1327/1327, repoctl, glossary, quality-lanes, architecture-boundary (165 crates), cargo-deny, and machine-readable JSON parse checks; see `/evidence/agentic-pipeline/ip-012-authoritative-tracked.json`.

## Implementation Plans
| IP | Title | Status | File |
|---|---|---|---|
| IP-001 | ADR-0054 scaffold-claim pattern + RACI human-orchestrator row (P0.5) | complete | [`IP-001-adr-0054-scaffold-claim.md`](IP-001-adr-0054-scaffold-claim.md) |
| IP-002 | Inventory pass + ADR-0052 (P1) | complete | [`IP-002-inventory-adr-0052.md`](IP-002-inventory-adr-0052.md) |
| IP-003 | `oya-tooling-agent-read` helper (P2) | complete | [`IP-003-oya-tooling-agent-read.md`](IP-003-oya-tooling-agent-read.md) |
| IP-004 | Bidirectional PRD citation + portfolio-citation lane (P3) | complete | [`IP-004-bidirectional-prd-cite.md`](IP-004-bidirectional-prd-cite.md) |
| IP-005 | Foundry corpus cross-cite + PHASE-00-SPEC.md (P3.5) | complete | [`IP-005-foundry-corpus-cross-cite.md`](IP-005-foundry-corpus-cross-cite.md) |
| IP-006 | Agent-facing memory rewrite (P4) | complete | [`IP-006-agent-facing-memory.md`](IP-006-agent-facing-memory.md) |
| IP-007 | Hook + skill audit + banned-primitives lane activation (P5) | complete | [`IP-007-hook-skill-audit.md`](IP-007-hook-skill-audit.md) |
| IP-008 | Archive orchestration glue + archive-orphan lane (P6) | complete | [`IP-008-archive-glue.md`](IP-008-archive-glue.md) |
| IP-009 | Delete archived glue from active path (P7) | complete | [`IP-009-delete-active-path.md`](IP-009-delete-active-path.md) |
| IP-010 | Parallel-claim demo runbook (P8) | complete | [`IP-010-parallel-claim-demo.md`](IP-010-parallel-claim-demo.md) |
| IP-011 | File upstream grit session bug (P9) | complete | [`IP-011-upstream-grit-bug.md`](IP-011-upstream-grit-bug.md) |
| IP-012 | Authoritative-tracked repo-walk audit (P10) | complete | [`IP-012-authoritative-tracked-audit.md`](IP-012-authoritative-tracked-audit.md) |

## Estimated parallelism
Per [`../../../ralplan-oyatie-sst-consolidation.md`](../../../ralplan-oyatie-sst-consolidation.md) §2 — phases serialize on merge order but each phase fans out to 2-4 agents internally.

## Symbols-touched
See [`../../../ralplan-oyatie-sst-consolidation.md`](../../../ralplan-oyatie-sst-consolidation.md) §2 per-phase symbol lists.

## Agent-handoff
```
icm store -t context-oyatie -c "M-CC-P01 complete: agentic-pipeline cutover green; banned-primitives + archive-orphan + authoritative-tracked lanes green on main; foundational gate cleared for all other milestones" -i critical -k "M-CC,P01,agentic-pipeline,cutover,complete"
```
