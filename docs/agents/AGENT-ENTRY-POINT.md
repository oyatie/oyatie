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
  - docs/agents/AGENT-ICM-TOPIC-CONVENTIONS.md
  - docs/agents/AGENT-CHEAT-SHEET.md
  - docs/agents/CROSS-REFERENCE-INDEX.md
  - docs/MASTERPLAN.md
  - docs/AGENTS.md
authority_chain_declaration: |
  docs/CONSTITUTION.md > docs/AGENTS.md > docs/MASTERPLAN.md > this file
---

# Oyatie Agent Entry Point

> **You are a fresh autonomous agent. Read this page top-to-bottom once. Then start at step 1.**
> Authority: this page operates under [`docs/CONSTITUTION.md`](../CONSTITUTION.md) and the [Oyatie Operating Contract](../AGENTS.md). Sanctioned primitives only: `{grit, icm, oya-tooling-agent-read}` plus Directive-12 documented exceptions. Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## Step 1 — Identify yourself

Generate a stable agent ID (`<harness>-<short-uuid>`, e.g. `claude-3a9f`). All grit/icm operations carry it.

```
grit claim --agent <agent-id> --intent "<one-line work intent>" <file::Identifier>
```

Never omit `--agent` or `--intent`. Without both, claims are anonymous and ungovernable.

## Step 2 — Locate work (the navigation contract)

Descend the 4-tier hierarchy. Do NOT scan the repo. Do NOT ask a human.

1. **Master plan**: read [`docs/MASTERPLAN.md`](../MASTERPLAN.md) §3 milestone index.
2. **Milestone**: pick one with `status: open` whose dependencies are merged. Read its `INDEX.md`.
3. **Phase**: pick a phase with no open prerequisites (per parent INDEX §Parallelism). Read its `INDEX.md`.
4. **Implementation Plan (IP)**: pick an `IP-NNN-*.md` whose `agent-prerequisites` frontmatter is satisfied. The IP frontmatter names: symbols-to-claim, acceptance-test-commands, icm-store-payload, audit-chain `EVT-*` event.

If multiple IPs are valid, prefer the lowest IP-NNN under the lowest P-NN under the lowest M-NN. Deterministic ordering prevents two agents racing for the same symbol.

## Step 3 — Verify prerequisites

Run, in order:

```
icm recall-context "M<NN>-P<NN> IP-<NNN>" --limit 5            # prior decisions
icm recall -t decisions-oyatie -k "<IP slug keywords>"          # ADR pointers
icm recall -t errors-resolved -k "<symbol/area keywords>"       # prior failures
icm recall -t scaffold-locks-oyatie -k "<crate path>"           # active scaffold-claim?
```

If any recall surfaces a blocker (active scaffold-lock you don't own, retired-path conflict, unresolved decision), STOP — see [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) §Halt.

Confirm: change class is named (per [`docs/AGENTS.md`](../AGENTS.md) §Pre-flight #1). Authority docs for that class are read.

## Step 4 — `grit claim` the IP's symbols

For each `file::Identifier` in the IP frontmatter `symbols-to-grit-claim:`:

```
grit claim --agent <agent-id> --intent "<MNN-PNN IP-NNN one-line>" <file::Identifier>
```

Claim failure modes route to [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md). Do NOT retry blindly.

When all symbols are claimed:

```
grit worktree --agent <agent-id>            # auto-prepares .grit/worktrees/<agent-id>/
```

## Step 5 — Work inside `.grit/worktrees/<agent-id>/`

- All edits, builds, tests run inside the worktree. Do not touch the main tree.
- For tool-by-tool conventions see [`AGENT-TOOL-PROTOCOL.md`](AGENT-TOOL-PROTOCOL.md).
- For "what do I do when X breaks" see [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) and [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md).
- Heartbeat hourly: `grit heartbeat --agent <agent-id>`.

## Step 6 — `icm store` at named events

Persist progress at every required milestone (per [`AGENT-ICM-TOPIC-CONVENTIONS.md`](AGENT-ICM-TOPIC-CONVENTIONS.md)):

- Claim succeeded → `icm store -t context-oyatie -c "<IP> claimed; symbols=<n>" -i high`
- Error resolved → `icm store -t errors-resolved -c "<one-line cause + fix>" -i high -k "<area>"`
- Decision made → `icm store -t decisions-oyatie -c "<decision + rationale>" -i high`
- Direct `git`/`gh` invoked (Directive 12) → `icm store -t direct-tool-invocations -c "<rationale>" -i high -k "git,<context>"` BEFORE the call.
- ≥20 tool calls without a store → emit a `context-oyatie` progress summary.

## Step 7 — Complete and hand off

Walk [`AGENT-COMPLETION-PROTOCOL.md`](AGENT-COMPLETION-PROTOCOL.md) in order. The final two commands are always:

```
icm store -t context-oyatie -c "<IP> complete; EVT=<audit-id>" -i high
grit done --agent <agent-id>
```

`grit done` is the only sanctioned merge primitive. Do NOT `git rebase`, `git merge`, or `gh pr merge` yourself.

## When to halt and escalate

There are exactly **3** halt cases (see [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md)). Anything else, you resolve autonomously via [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) and [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md).

Halt format (only when matrix matches):

```
icm store -t cutover-orchestrator-actions -c "BLOCKED_ON_HUMAN_ORCHESTRATOR: <case-id>: <one-line>" -i critical -k "halt,<area>"
grit release --agent <agent-id> --reason "BLOCKED_ON_HUMAN_ORCHESTRATOR: <case-id>"
```

Then exit cleanly. The orchestrator polls `cutover-orchestrator-actions` and unblocks you.
