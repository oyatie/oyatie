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
doc_status: published
---

# Agent Failure Recovery

> One row per failure shape. Apply the recovery; stay in the loop. Halt only on the three rows in [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md). Foundation: ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim).


Cause: the IP names a symbol in a file that does not yet exist (greenfield) or in a re-indexed crate.

**Scaffold-claim pattern (ADR-0054).**

3. In the worktree, create the file skeleton (frontmatter / module declarations / placeholder identifier). Do NOT add business logic yet.

If step 1 surfaces an active scaffold-lock owned by another agent, drop to R2.


Cause: another agent owns the symbol or its scaffold-lock.


4. Meanwhile, pick a sibling IP (deterministic descent: same phase, next IP-NNN whose prerequisites are met). Never force-steal.



Session-less mode:

1. Drop `--session` flag from all calls. Per-agent worktree isolation is sufficient.
3. Continue claim → work → done as normal.


Cause: another agent's merge raced and touched overlapping symbols.

2. `tooling-agent-read log --range <merge-base>..HEAD --paths <your touched paths>` — identify the colliding commit.
5. Re-run acceptance tests (C2). Re-emit any failed lanes.

## R5 — CI fitness lane red after C2 passed locally

Cause: lane runs against a fuller surface than your local checks.

1. `tooling-agent-read pr-view <pr-number>` — read the failed lane name.
2. Cross-reference the lane → standard via [`AGENT-DECISION-TREE.md`](AGENT-DECISION-TREE.md) §D4.
3. Open the named `docs/standards/<file>.md`. Apply the correction in the worktree.
4. If the failure is OUT OF CLAIM SCOPE (e.g. `-cohesion` red on a crate you didn't touch), this is a release-and-defer event, not a halt:
   ```
   ```
5. Pick the next IP per [`AGENT-ENTRY-POINT.md`](AGENT-ENTRY-POINT.md) §Step 2.

## R6 — `cargo deny check` fails (license / vuln / banned-source)

1. Identify the offending crate from the deny output.
2. If a new dep was added by this claim → remove or replace with an LTS-pinned permitted alternative per [`docs/standards/dependency-policy.md`](../standards/dependency-policy.md).
3. If a transitive dep regressed → bump the parent or override.
4. AGPL/GPL/SSPL/BUSL/RSAL are never permitted in product code; SOURCE: pick a permissive alternative.


Cause: typo or speculative new topic.

3. Never invent a new topic mid-claim. Topic taxonomy changes are council-architecture only.

## When recovery itself fails

If two consecutive recovery attempts at the same row fail with the same shape, and the failure isn't a [`ESCALATION-MATRIX.md`](ESCALATION-MATRIX.md) row, prefer release-and-defer (D8 in the decision tree) over halt. Halt is only for the three matrix cases.
