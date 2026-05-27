---
purpose: Oyatie Runbook — Forgejo Agent-Board Operating Workflow
doc_status: published
---

# Oyatie Runbook — Forgejo Agent-Board Operating Workflow

> **Status:** Operator/agent workflow for Wave 3 disjoint lanes on the Forgejo-backed board.
> **Owner:** Governance tooling maintainers.
> **Last verified:** 2026-05-27 from leader-supplied live Forgejo board evidence.
> **Scope:** Plain Git worktrees, Forgejo issues, exclusive `board/*` labels, Git claim refs, and webhooks. This runbook does not authorize `oya git`, `oya vcs`, GitHub Projects, native Forgejo Projects automation, concurrent `oya gate run-all`, or a bespoke long-running board service.

## Operating model

The agent board is an issue-and-ref workflow for coordinating many disjoint implementation lanes without relying on non-atomic issue assignees or unavailable Projects APIs. Each lane has one local worktree, one deliverable id, one claim ref, and one scoped verification bundle.

Authoritative ownership comes from the Git ref `refs/heads/claims/<deliverable-id>`. Forgejo issue labels and assignees are human-facing projections only. They help operators see backlog, claimed, review, or done states, but they do not grant ownership and must be reconciled back to the claim ref when they drift.

## Lane startup

1. Create or reuse exactly one worktree per lane.
2. Confirm the lane scope before editing:
   - deliverable id;
   - allowed paths;
   - forbidden shared files;
   - targeted verification commands;
   - expected commit and evidence format.
3. Fetch the remote and check for an existing claim ref:

   ```bash
   git fetch origin 'refs/heads/claims/*:refs/remotes/origin/claims/*'
   git ls-remote origin "refs/heads/claims/<deliverable-id>"
   ```

4. If the claim ref already exists, do not edit. Report the current owner from the ref, webhook projection, or issue metadata.
5. If the claim ref is absent, attempt to create it before editing. The create operation is the claim.

## Claim-before-edit rule

Agents must claim by creating `refs/heads/claims/<deliverable-id>` before changing scoped files. A successful claim is exactly one remote ref creation for the deliverable id. A losing agent must stop and report the conflict.

Recommended flow:

```bash
# Use the current lane start commit or an agreed empty marker commit.
git push origin HEAD:refs/heads/claims/<deliverable-id>

# Confirm the remote accepted exactly one claim ref.
git ls-remote origin "refs/heads/claims/<deliverable-id>"
```

If the push fails with a remote ref-lock, reference-exists, or non-fast-forward style conflict, treat it as a lost race. Do not force-push, delete, move, or overwrite another worker's claim ref.

## Board label projection

Use exclusive `board/*` labels as non-authoritative board columns:

- `board/backlog` means the issue is ready but unclaimed.
- `board/claimed` means a winning claim ref exists.
- `board/review` means the lane has a commit and evidence ready for integration review.
- `board/done` means the lane has been integrated or otherwise closed by the leader.
- `board/blocked` means the lane cannot proceed without reassignment, scope change, or external authority.

Rules:

1. Only move `board/*` labels after the authoritative state exists.
2. Claim success may project `board/backlog -> board/claimed`.
3. A failed claim must not project ownership labels.
4. Label updates must be idempotent. Reapplying the same projection should produce no semantic change.
5. Label drift is repaired from claim refs and accepted commits, not the other way around.

## Webhook projection expectations

Forgejo webhooks are projection inputs, not an extra locking service. The board projection consumes:

- issue label events for visible column changes;
- push events for `refs/heads/claims/<deliverable-id>`;
- sender identity from webhook payloads;
- commit metadata from the pushed ref.

Projection handlers must be idempotent. Receiving the same label or push webhook twice should converge to the same board snapshot. Missing webhooks are recovered by polling Git refs and issue labels; they must not require a long-running board daemon to be correct.

## Agent identity

Every lane must preserve two identities:

1. **Token user / Forgejo sender:** the authenticated user that created the claim ref or updated labels.
2. **Commit author:** the Git author recorded in the lane commit.

Both identities should appear in evidence when available. If they differ, the runbook treats the token user as the actor for remote mutation and the commit author as the code/document author. Neither replaces the claim ref as the ownership source of truth.

## Editing and verification discipline

After a successful claim:

1. Edit only the scoped files for the lane.
2. Prefer targeted, affected-only verification over global gates.
3. Do not run concurrent `oya gate run-all` from multiple lanes.
4. Run file-scoped whitespace and doc checks for docs-only lanes.
5. Record skipped checks as not applicable or pre-existing, with a concrete reason.
6. Commit only scoped changes from the lane worktree.

Docs-only lane example:

```bash
git diff --check -- docs/runbooks/<runbook>.md
npx --yes markdownlint-cli2 docs/runbooks/<runbook>.md
```

If markdown tooling is unavailable or not configured, record that explicitly and keep `git diff --check` as the required targeted check.

## Commit and handoff

A lane commit must include:

- a subject that explains why the change exists;
- Lore protocol trailers for constraints, rejected alternatives, confidence, scope risk, directives, tested checks, and not-tested gaps;
- the deliverable id or task id in the body when useful for integration.

After commit:

1. Project the issue to `board/review` if the team has authorized label projection for completed lane work.
2. Send the leader the commit hash, scoped files, and verification evidence.
3. Do not start another lane in the same worktree unless the leader assigns it and a new claim ref is created.

## Prohibited fallbacks

Do not use or reintroduce:

- `oya git`;
- `oya vcs`;
- GitHub Projects;
- native Forgejo Projects REST automation as a required board path;
- issue assignee mutation as an atomic lock;
- concurrent `oya gate run-all` across worker lanes;
- a bespoke long-running board service without a later approved design.

## Recovery cases

### Lost claim race

1. Fetch `refs/heads/claims/<deliverable-id>`.
2. Report the winning ref tip and, when available, webhook sender identity.
3. Leave labels and assignees untouched except for a read-only refresh.
4. Wait for reassignment.

### Label drift

1. Read the claim ref and issue labels.
2. Recompute the expected projection.
3. Apply only the missing exclusive `board/*` transition.
4. Preserve an evidence note that labels were repaired from refs.

### Webhook gap

1. Poll claim refs and issue labels.
2. Rebuild the board snapshot from current Forgejo state.
3. Compare to the last projection snapshot.
4. Emit an idempotent reconciliation diff.

### Shared-file conflict

1. Stop before editing the shared file.
2. Report the path, owning lane, and required decision.
3. Wait for leader reassignment or an explicit scope change.

## Acceptance checklist

- [ ] One worktree exists for the lane.
- [ ] The agent created or verified the winning `refs/heads/claims/<deliverable-id>` before editing.
- [ ] Only scoped files changed.
- [ ] `board/*` labels were treated as projection-only.
- [ ] Webhook sender identity and commit author were recorded when available.
- [ ] Verification was affected-only and did not include concurrent `oya gate run-all`.
- [ ] The commit follows Lore protocol.
- [ ] The leader received commit hash and verification evidence.

## Sources

Leader-supplied live evidence from the 2026-05-27 Wave 3 board implementation run: Forgejo `11.0.14` on `oya-forge`, Projects REST `404`, exclusive-label projection, non-atomic assignee race, atomic claim-ref race, sender-bearing push webhook payloads, and the team constraint to coordinate disjoint lanes through plain Git plus Forgejo issue projections.
