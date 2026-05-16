---
doc_class: AgentCompletionProtocol
shape: protocol
status: pending approval
authority_tier: 2
length_cap: 120
purpose: |
  Step-by-step the final mile: from "code looks done" to `grit done`. Every step has a
  typed artifact (command output, lane, ledger row). No agent declares completion
  without walking this protocol in order.
lift_target: oyatie/docs/agents/AGENT-COMPLETION-PROTOCOL.md
canonical_authority: docs/CONSTITUTION.md
related:
  - .omc/agent-kickoff/AGENT-ENTRY-POINT.md
  - .omc/agent-kickoff/AGENT-TOOL-PROTOCOL.md
  - .omc/agent-kickoff/AGENT-ICM-TOPIC-CONVENTIONS.md
  - /templates/checklists/agent-completion-checklist.md
  - docs/AGENTS.md
---

# Agent Completion Protocol

> Run these in order. Each step produces a typed artifact (command output, lane row, ledger row). Skipping a step = silent regression. The merge gate refuses `grit done` if D16 (audit emission) or `## Code Review` is missing.

## C1 — Re-read the IP `done-criteria`

Open the IP frontmatter `done-criteria:` and `acceptance-test-commands:`. These define DONE for this claim — nothing else.

## C2 — Run acceptance test commands (named in IP frontmatter)

For each command listed in the IP `acceptance-test-commands:` array, run it in the worktree. Default baseline (always-required, per [`docs/AGENTS.md`](../../docs/AGENTS.md) §Done-Definition D9–D12):

```
cargo nextest run --workspace --all-features --no-fail-fast
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check
repoctl pre-push
```

If the IP frontmatter adds lane-specific commands (e.g. `oya-foundry-fitness-data-class`, `-image-discipline`), run those too.

## C3 — Paste output evidence

Capture exit code + last 20 lines (or full output if shorter) into a local `evidence.txt` in your worktree. The PR template `## Verification` section consumes this. The `traceability-validator` lane refuses PRs without per-check pass/fail lines.

## C4 — `icm store` the named topic + payload

The IP frontmatter declares `icm-store-payload:` with `topic`, `content-template`, `importance`, `keywords`. Emit it verbatim (substitute `<placeholders>`):

```
icm store -t <topic> -c "<content with placeholders filled>" -i <importance> -k "<kw1,kw2>"
```

Almost always `context-oyatie` at `high`. Add `decisions-oyatie` if you made a non-trivial architectural call.

## C5 — Emit audit-chain `EVT-*` row (from IP frontmatter)

The IP `audit-chain-event:` field names the event ID and payload schema. Emit via:

```
oya-tooling-agent-read audit-emit --event <EVT-id> --payload '<json>'
```

Capture the returned emission ID — this goes into PR body `## Evidence` and is referenced by `oya-foundry-fitness-audit-emission` lane (D16).

## C6 — Update inventory ledger (if applicable)

Phases under M-CC-P01 (agentic-pipeline cutover) and any phase whose INDEX `inventory-ledger-required: true` requires an `oya-foundry-fitness-inventory-tracker` row (CHK-INV). Append the row in this PR.

## C7 — Update CHANGELOG / MISTAKES-LEDGER (if applicable)

- Canonical doc touched → `CHANGELOG.md` row in this PR (D18).
- Mechanical-prevention shipped for a prior failure → `MISTAKES-LEDGER.md` row (D17).

## C8 — Walk the AGENTS.md Done-Definition checklist (D1–D18)

Open [`docs/AGENTS.md`](../../docs/AGENTS.md) §Done-Definition. Tick every box. Any unticked box = NOT DONE, loop back. The reviewer-agent verdict for your change class must be captured in PR `## Code Review`.

## C9 — `grit done`

```
grit done --agent <agent-id>
```

Outcomes:
- **OK** → claim merged via merge queue; your worktree is GC'd; emit one final `icm store -t context-oyatie -c "<IP> shipped; EVT=<id>; PR=<n>" -i high`.
- **Conflict** → see [`AGENT-FAILURE-RECOVERY.md`](AGENT-FAILURE-RECOVERY.md) §R3.
- **Missing-evidence refusal** → loop back to C2–C5; do NOT bypass.

## C10 — Do NOT manually rebase or merge

`grit done` is the only sanctioned merge primitive. If `grit done` is unhealthy on your harness, that is a halt case (`HALT-03` in [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md)) — release the claim, emit the halt row, do not work around it with raw `git`. Working around `grit done` invalidates the audit chain.

## C11 — Post-merge: pick the next IP

Re-enter [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) §Step 2. Stay in the same milestone if its parallelism strategy permits and prerequisites hold; otherwise descend a sibling phase or milestone per the dependency graph.
