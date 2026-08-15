---
doc_class: AgentCompletionProtocol
shape: protocol
status: Accepted
authority_tier: 2
length_cap: 120
date: 2026-05-12
purpose: |
  typed artifact (command output, lane, ledger row). No agent declares completion
  without walking this protocol in order.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-ENTRY-POINT.md
  - docs/agents/AGENT-TOOL-PROTOCOL.md
  - docs/templates/checklists/agent-completion-checklist.md
  - docs/AGENTS.md
doc_status: published
---

# Agent Completion Protocol


## C1 — Re-read the IP `done-criteria`

Open the IP frontmatter `done-criteria:` and `acceptance-test-commands:`. These define DONE for this claim — nothing else.

## C2 — Run acceptance test commands (named in IP frontmatter)

For each command listed in the IP `acceptance-test-commands:` array, run it in the worktree. Default baseline (always-required, per [`docs/AGENTS.md`](../AGENTS.md) §Done-Definition D9–D12):

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo fmt --all --check
```

If the IP frontmatter adds lane-specific commands (e.g. `oya-governance-data-class`, `-image-discipline`), run those too.

## C3 — Paste output evidence

Capture exit code + last 20 lines (or full output if shorter) into a local `evidence.txt` in your worktree. The PR template `## Verification` section consumes this. The `traceability-validator` lane refuses PRs without per-check pass/fail lines.



```
```

Almost always `context-oyatie` at `high`. Add `decisions-oyatie` if you made a non-trivial architectural call.

## C5 — Emit audit-chain `EVT-*` row (from IP frontmatter)

The IP `audit-chain-event:` field names the event ID and payload schema. Emit via:

```
oya-tooling-agent-read audit-emit --event <EVT-id> --payload '<json>'
```

Capture the returned emission ID — this goes into PR body `## Evidence` and is referenced by `oya-governance-audit-emission` lane (D16).

## C6 — Update inventory ledger (if applicable)

Phases under M01-P08 (agentic-pipeline cutover) and any phase whose INDEX `inventory-ledger-required: true` requires an `oya-governance-inventory-tracker` row (CHK-INV). Append the row in this PR.

## C7 — Update CHANGELOG / MISTAKES-LEDGER (if applicable)

- Canonical doc touched → `CHANGELOG.md` row in this PR (D18).
- Mechanical-prevention shipped for a prior failure → `MISTAKES-LEDGER.md` row (D17).

## C8 — Walk the AGENTS.md Done-Definition checklist (D1–D18)

Open [`docs/AGENTS.md`](../AGENTS.md) §Done-Definition. Tick every box. Any unticked box = NOT DONE, loop back. The reviewer-agent verdict for your change class must be captured in PR `## Code Review`.


```
```

Outcomes:
- **Conflict** → see [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md) §R3.
- **Missing-evidence refusal** → loop back to C2–C5; do NOT bypass.

## C10 — Do NOT manually rebase or merge


## C11 — Post-merge: pick the next IP

Re-enter [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) §Step 2. Stay in the same milestone if its parallelism strategy permits and prerequisites hold; otherwise descend a sibling phase or milestone per the dependency graph.
