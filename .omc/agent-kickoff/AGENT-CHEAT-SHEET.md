---
doc_class: AgentCheatSheet
shape: cheat-sheet
status: pending approval
authority_tier: 3
length_cap: 80
purpose: |
  Printable 1-pager for autonomous agents. The 10 commands every agent runs, the 5
  lookups they hit most, and the 3 emergency-halt patterns. Pin to context window.
lift_target: oyatie/docs/agents/AGENT-CHEAT-SHEET.md
canonical_authority: docs/CONSTITUTION.md
related:
  - .omc/agent-kickoff/AGENT-ENTRY-POINT.md
  - .omc/agent-kickoff/AGENT-TOOL-PROTOCOL.md
  - .omc/agent-kickoff/ESCALATION-MATRIX.md
---

# Agent Cheat Sheet

## The 10 commands

```
# Locate work
icm recall-context "M<NN>-P<NN> IP-<NNN>" --limit 5

# Identify self + claim
grit claim --agent <id> --intent "<one-line>" <file::Identifier>
grit worktree --agent <id>

# Persist progress
icm store -t context-oyatie -c "<summary>" -i high -k "<MNN-PNN>,<IP-NNN>"

# Run acceptance evidence (baseline)
cargo nextest run --workspace --all-features --no-fail-fast
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check

# Emit audit chain row
oya-tooling-agent-read audit-emit --event <EVT-id> --payload '<json>'

# Ship
grit done --agent <id>

# Heartbeat (hourly minimum)
grit heartbeat --agent <id>
```

## The 5 lookups

```
icm recall -t decisions-oyatie -k "<area>"          # prior architectural calls
icm recall -t errors-resolved -k "<symbol/area>"    # prior failures
icm recall -t scaffold-locks-oyatie -k "<crate>"    # contested scaffolds
oya-tooling-agent-read pr-view <n>                  # PR state without gh
oya-tooling-agent-read log --range A..B --paths P   # history without git
```

## The 3 emergency-halt patterns

Halt only when [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) matches. Format:

```
icm store -t cutover-orchestrator-actions \
  -c "BLOCKED_ON_HUMAN_ORCHESTRATOR: <case-id>: <one-line>" \
  -i critical -k "halt,<area>"
grit release --agent <id> --reason "BLOCKED_ON_HUMAN_ORCHESTRATOR: <case-id>"
```

Cases:
1. `HALT-01` — autonomy-ceiling uplift required (T1→T2, T2→T3, T3→T4) for the change.
2. `HALT-02` — destructive operation needed on shared ref (force-push to `main`, hard-reset of merged history, schema-class downgrade on regulated field).
3. `HALT-03` — sanctioned-primitive itself unhealthy (grit/icm/oya-tooling-agent-read returns infra error after 2 retries).

For everything else: D1–D9 in [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md), R1–R7 in [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md). Stay in the loop.

## Forbidden (regardless of rationale)

`--no-verify` / hook bypass; `git push --force` to `main`; `gh pr merge` (use `grit done`); editing `~/.claude/` or `/Users/home/Documents/GitHub/claude-code`; inventing new icm topics (use the canonical 8); claiming symbols not in the IP frontmatter.
