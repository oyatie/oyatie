---
doc_class: AgentEntryPoint
shape: anchor
status: Accepted
authority_tier: 1
length_cap: 150
date: 2026-05-12
purpose: |
  The first and only file a fresh autonomous agent reads. Routes any cold-start agent
  end-to-end: identify self → locate work → verify prerequisites → claim → work → store
  → done → emit. Designed so an agent with no prior session context can execute an IP
  without human orchestrator hand-holding.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-DECISION-TREE.md
  - docs/agents/AGENT-TOOL-PROTOCOL.md
  - docs/agents/AGENT-COMPLETION-PROTOCOL.md
  - docs/agents/AGENT-FAILURE-RECOVERY.md
  - docs/agents/AGENT-CHEAT-SHEET.md
  - docs/agents/CROSS-REFERENCE-INDEX.md
  - docs/MASTERPLAN.md
  - docs/AGENTS.md
authority_chain_declaration: |
  docs/CONSTITUTION.md > docs/AGENTS.md > docs/MASTERPLAN.md > this file
doc_status: published
---

# Oyatie Agent Entry Point

> **You are a fresh autonomous agent. Read this page top-to-bottom once. Then start at step 1.**

## Step 1 — Identify yourself


```
```

Never omit `--agent` or `--intent`. Without both, claims are anonymous and ungovernable.

## Step 2 — Locate work (the navigation contract)

Descend the 4-tier hierarchy. Do NOT scan the repo. Do NOT ask a human.

1. **Master plan**: read [`docs/MASTERPLAN.md`](../MASTERPLAN.md) §3 milestone index.
2. **Milestone**: pick one with `status: open` whose dependencies are merged. Read its `INDEX.md`.
3. **Phase**: pick a phase with no open prerequisites (per parent INDEX §Parallelism). Read its `INDEX.md`.

If multiple IPs are valid, prefer the lowest IP-NNN under the lowest P-NN under the lowest M-NN. Deterministic ordering prevents two agents racing for the same symbol.

## Step 3 — Verify prerequisites

Run, in order:

```
```

If any recall surfaces a blocker (active scaffold-lock you don't own, retired-path conflict, unresolved decision), STOP — see [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) §Halt.

Confirm: change class is named (per [`docs/AGENTS.md`](../AGENTS.md) §Pre-flight #1). Authority docs for that class are read.



```
```

Claim failure modes route to [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md). Do NOT retry blindly.

When all symbols are claimed:

```
```


- All edits, builds, tests run inside the worktree. Do not touch the main tree.
- For tool-by-tool conventions see [`AGENT-TOOL-PROTOCOL.md`](AGENT-TOOL-PROTOCOL.md).
- For "what do I do when X breaks" see [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) and [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md).



- ≥20 tool calls without a store → emit a `context-oyatie` progress summary.

## Step 7 — Complete and hand off

Walk [`AGENT-COMPLETION-PROTOCOL.md`](AGENT-COMPLETION-PROTOCOL.md) in order. The final two commands are always:

```
```


## When to halt and escalate

There are exactly **3** halt cases (see [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md)). Anything else, you resolve autonomously via [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) and [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md).

Halt format (only when matrix matches):

```
```

Then exit cleanly. The orchestrator polls `cutover-orchestrator-actions` and unblocks you.
