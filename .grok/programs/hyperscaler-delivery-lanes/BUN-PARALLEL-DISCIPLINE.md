# Bun rewrite lessons applied to parallel delivery

Canonical process rules for multi-PR / multi-lane delivery.  
Workflows: `.grok/workflows/parallel-delivery-bun.rhai`, `.grok/workflows/open-pr-fleet.rhai`.  
Roles: `.grok/multi-model-roles.json`. Drive: `.grok/harness/drive.v1.json`.

## What "fix the process" means (and does not)

**Means:** edit the **workflow / harness / tool** so the next agent cannot make the same class of mistake without a hard fail, forced step, or explicit waiver.

| Fix class | Examples (do this) | Not this |
|-----------|-------------------|----------|
| **Process / harness** | Fail-closed head check in `preflight-ci-infra`; tip-sync phase in `open-pr-fleet`; trial gate before fan-out; expand `process_edit_triggers`; stop-hook rules | One-off rebase of one PR and stop |
| **Tool** | Receipt writes `ok:false` on wrong tree; merge-check rejects head mismatch | Chat note "remember to use PR worktree" |
| **Workflow** | Post-merge tip-sync job; anti-passive re-poll rule; PREFLIGHT_RULE injected into every PR agent | Manual babysit of one red check |

**Does not mean:**

- Only fixing the immediate bug/symptom on one branch
- Relying on session memory, beads comments, or human reminders
- Hero rebases / mid-run pushes "just this once"
- Claiming WAIT/idle while CI is the only remaining work and no monitor is armed

When a mistake repeats **twice**, promote a harness edit (see `run-grade.v1.json` → `promote_process_edit_after_repeats: 2`).

## Six Bun rules

1. **Prep contract first** — ADR / design / path taxonomy before more expansion (e.g. ADR-0639 Wave1 before more path-filters).
2. **One representative trial before fan-out** — do not open more capacity/path-filter legs while the trial PR is red or unapplied.
3. **Dual split-context review** — agent dual-critic packets for every merge (implementer ≠ critic context).
4. **Fail closed on missing evidence** — no merge without exact-head `oya-ci-required`; no complete packet without promoted SHA.
5. **Edit process when systematic** — if agents keep doing X, change the tool/workflow so X fails closed or is forced correct.
6. **Do not expand while trial red/incomplete** — no W0-B until G001; no 3B open until trial green; no more optional legs until capacity trial path is green.

## Systematic mistakes → required process edits

| Mistake class (observed) | Harness fix (must exist) |
|--------------------------|--------------------------|
| Preflight / merge-check on wrong tree (clean tip, dirty primary, unrelated worktree) | `preflight-ci-infra --pr N` fails if cwd HEAD ≠ PR `headRefOid`; fail receipt written |
| Fleet PR left behind after trunk advances | `open-pr-fleet` post-merge **tip-sync** phase: fetch origin/dev, rebase remaining open PRs |
| Passive WAIT on `waiting_ci` (no re-poll, no tip-sync) | Drive class text + stop-hook: waiting_ci ≠ idle; arm monitor or re-tick; process_edit_triggers include passive wait |
| Fan-out while capacity trial red | `parallel-delivery-bun` expand gate; reorg/capacity jobs only when `trial_ready_to_expand` |
| Mid-run push (cancel-in-progress kills FULL cone) | PREFLIGHT_RULE: batch local reds → one push → wait complete run |
| One-off fix without harness change after repeat | CRITIC / grade require `process_edits` when class repeats |

## Preflight contract (CI-infra paths)

```text
Worktree at PR head SHA
  → .grok/bin/preflight-ci-infra --pr N   # exit 0, ok:true receipt, head match
  → dual-critic re-head
  → ONE push
  → wait for COMPLETE oya-ci-required (never mid-run push)
  → mm-drive merge-check / merge
```

Soft multi-arch (e.g. windows `continue-on-error`) is **non-binding**.

## Post-merge tip-sync (fleet)

After any merge to `dev`:

1. `git fetch origin dev`
2. For each remaining open PR against `dev`: if behind, rebase (or recreate) onto new tip in **that PR's worktree**
3. Re-run dual-critic only if content conflict resolution changed behavior
4. Do not claim "waiting" without re-checking base vs head

## Anti-passive rule

`waiting_ci` allows Stop **only if** a background poller/scheduler is armed **or** the next action is explicitly "re-tick after N minutes".  
Never end a session with open mergeable/resolvable work classified as WAIT without a process note and an armed re-query.
