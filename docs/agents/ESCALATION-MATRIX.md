---
doc_class: EscalationMatrix
shape: matrix
status: Accepted
authority_tier: 1
length_cap: 60
date: 2026-05-12
purpose: |
  The complete, exhaustive matrix of cases where an autonomous agent MUST halt and emit
  `BLOCKED_ON_HUMAN_ORCHESTRATOR`. Designed per the autonomy directive to be as small as
  possible. Anything not in this matrix is resolved by the agent autonomously via
  AGENT-DECISION-TREE / AGENT-FAILURE-RECOVERY.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-DECISION-TREE.md
  - docs/agents/AGENT-FAILURE-RECOVERY.md
  - docs/agents/HUMAN-OPERATOR-GUIDE.md
doc_status: published
---

# Escalation Matrix

> **3 cases.** Halt only when one matches. Everything else: stay in the autonomous loop. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

|---|---|---|---|---|

## Cases explicitly NOT in the matrix (handled autonomously)

- `cargo` failure → D2 / R5 (silent-failure-hunter + standards).
- Fitness lane red → D4 (lane→standard mapping).
- Claim FK error → R1 (scaffold-claim pattern, ADR-0054).
- New dep license fail → R6 (replace dep).
- Need raw `git`/`gh` → Directive-12 (`direct-tool-invocations` audit row, no halt).
- Need to defer an IP → D8 (release-and-defer, no halt).

If a case looks like a halt but isn't in the matrix, prefer release-and-defer over halt. Council-architecture reviews `cutover-orchestrator-actions` weekly; matrix growth is a council decision, never agent-initiated.
