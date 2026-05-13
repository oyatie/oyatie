---
doc_class: AgentFailureRecovery
shape: recovery-protocol
status: Accepted
authority_tier: 2
length_cap: 120
date: 2026-05-12
purpose: |
  What to do when a sanctioned-primitive operation fails. Every recovery here keeps the
  agent inside the autonomous loop. Halt only when `ESCALATION-MATRIX.md` matches.
canonical_authority: docs/CONSTITUTION.md
foundation: ADR-0053 (sanctioned primitives), ADR-0054 (scaffold-claim)
related:
  - docs/agents/AGENT-DECISION-TREE.md
  - docs/agents/AGENT-TOOL-PROTOCOL.md
  - docs/agents/ESCALATION-MATRIX.md
  - docs/standards/INDEX.md
---

# Agent Failure Recovery

> One row per failure shape. Apply the recovery; stay in the loop. Halt only on the three rows in [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md). Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).

## R1 — `grit claim` FK error (symbol not in symbol-graph yet)

Cause: the IP names a symbol in a file that does not yet exist (greenfield) or in a re-indexed crate.

**Scaffold-claim pattern (ADR-0054).**

1. `icm recall -t scaffold-locks-oyatie -k "<crate-path>"` — confirm no active scaffold-lock.
2. `grit claim --agent <id> --intent "scaffold <crate>" <existing-symbol e.g. crates/oya-platform-X/src/lib.rs::__scaffold>` — anchor on a real symbol.
3. In the worktree, create the file skeleton (frontmatter / module declarations / placeholder identifier). Do NOT add business logic yet.
4. `grit symbols --refresh` — re-indexes.
5. `grit claim --agent <id> --intent "<real intent>" <file::Identifier>` — now succeeds.
6. `icm store -t scaffold-locks-oyatie -c "<crate>: scaffolded by <agent>; real-claim in flight" -i high -k "scaffold,<crate>"`.
7. `grit release --agent <id> --symbol <__scaffold>` once the real symbol is claimed.

If step 1 surfaces an active scaffold-lock owned by another agent, drop to R2.

## R2 — `grit claim` lock held by another agent

Cause: another agent owns the symbol or its scaffold-lock.

`icm-coordination-lock` fallback:

1. `grit watch --symbol <file::Id> --agent <id>` — stream release events.
2. While waiting, emit `icm store -t context-oyatie -c "waiting on <other-agent> for <symbol>; ETA unknown" -i medium`.
3. After 30 min OR if `oya-tooling-agent-read pr-view <linked-PR>` shows their PR stalled with no heartbeat: `icm recall -t context-oyatie -k "<other-agent>"` to see their last checkpoint. If stalled ≥4 h with no heartbeat, council-architecture's GC will reclaim; emit a `context-oyatie` row noting the wait.
4. Meanwhile, pick a sibling IP (deterministic descent: same phase, next IP-NNN whose prerequisites are met). Never force-steal.

## R3 — `grit session` start error (RM-05, 0.3.0 bug)

Cause: known grit 0.3.0 session bug.

Session-less mode:

1. Drop `--session` flag from all calls. Per-agent worktree isolation is sufficient.
2. `icm store -t errors-resolved -c "grit 0.3.0 session bug; switched to session-less" -i high -k "grit,session,RM-05"`.
3. Continue claim → work → done as normal.

## R4 — `grit done` conflict at merge queue

Cause: another agent's merge raced and touched overlapping symbols.

1. `grit watch --queue --agent <id>` — stream merge-queue events.
2. `oya-tooling-agent-read log --range <merge-base>..HEAD --paths <your touched paths>` — identify the colliding commit.
3. `icm recall -t context-oyatie -k "<colliding-symbol>"` — locate the other agent's checkpoints.
4. `grit assign --agent <id> --to <colliding-agent> --symbol <conflict-symbol>` IF the conflict is in a symbol they should own; OTHERWISE rebase the worktree (`grit worktree --refresh --agent <id>` — the grit primitive handles the rebase under the merge-queue contract).
5. Re-run acceptance tests (C2). Re-emit any failed lanes.
6. `grit done --agent <id>` again.
7. If conflict recurs ≥3 times on the same symbol, emit `icm store -t errors-resolved -c "thrash on <symbol>; needs cross-agent coordination" -i high` and switch IPs.

## R5 — CI fitness lane red after C2 passed locally

Cause: lane runs against a fuller surface than your local checks.

1. `oya-tooling-agent-read pr-view <pr-number>` — read the failed lane name.
2. Cross-reference the lane → standard via [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) §D4.
3. Open the named `docs/standards/<file>.md`. Apply the correction in the worktree.
4. If the failure is OUT OF CLAIM SCOPE (e.g. `-cohesion` red on a crate you didn't touch), this is a release-and-defer event, not a halt:
   ```
   icm store -t context-oyatie -c "<lane> red on <crate>; out of claim scope; releasing for council coordination" -i high
   grit release --agent <id> --reason "out-of-claim-scope: <lane>"
   ```
5. Pick the next IP per [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) §Step 2.

## R6 — `cargo deny check` fails (license / vuln / banned-source)

1. Identify the offending crate from the deny output.
2. If a new dep was added by this claim → remove or replace with an LTS-pinned permitted alternative per [`docs/standards/dependency-policy.md`](../standards/dependency-policy.md).
3. If a transitive dep regressed → bump the parent or override.
4. AGPL/GPL/SSPL/BUSL/RSAL are never permitted in product code; SOURCE: pick a permissive alternative.

## R7 — `icm` topic not in canonical list

Cause: typo or speculative new topic.

1. `icm topics` — confirm canonical names.
2. Use the closest canonical match from [`AGENT-ICM-TOPIC-CONVENTIONS.md`](AGENT-ICM-TOPIC-CONVENTIONS.md).
3. Never invent a new topic mid-claim. Topic taxonomy changes are council-architecture only.

## When recovery itself fails

If two consecutive recovery attempts at the same row fail with the same shape, and the failure isn't a [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) row, prefer release-and-defer (D8 in the decision tree) over halt. Halt is only for the three matrix cases.
