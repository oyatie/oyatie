---
doc_class: AgentCheatSheet
shape: cheat-sheet
status: Accepted
authority_tier: 3
length_cap: 80
date: 2026-05-12
purpose: |
  Printable 1-pager for autonomous agents. The 10 commands every agent runs, the 5
  lookups they hit most, and the 3 emergency-halt patterns. Pin to context window.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-ENTRY-POINT.md
  - docs/agents/AGENT-TOOL-PROTOCOL.md
  - docs/agents/ESCALATION-MATRIX.md
doc_status: published
---

# Agent Cheat Sheet

## The 10 commands

```
# Locate work

# Identify self + claim

# Persist progress

# Run acceptance evidence (baseline)
cargo nextest run --workspace --all-features --no-fail-fast
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check

# Emit audit chain row
tooling-agent-read audit-emit --event <EVT-id> --payload '<json>'

# Ship

# Heartbeat (hourly minimum)
```

## The 5 lookups

```
tooling-agent-read pr-view <n>                  # PR state without gh
tooling-agent-read log --range A..B --paths P   # history without git
```

## The 3 emergency-halt patterns

Halt only when [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) matches. Format:

```
  -c "BLOCKED_ON_HUMAN_ORCHESTRATOR: <case-id>: <one-line>" \
  -i critical -k "halt,<area>"
```

Cases:
1. `HALT-01` — autonomy-ceiling uplift required (T1→T2, T2→T3, T3→T4) for the change.
2. `HALT-02` — destructive operation needed on shared ref (force-push to `main`, hard-reset of merged history, schema-class downgrade on regulated field).

For everything else: D1–D9 in [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md), R1–R7 in [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md). Stay in the loop.

## Forbidden (regardless of rationale)

