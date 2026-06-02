# Auto-Merge Flow — Forgejo and GitHub after CI

This note describes the P0.0 target contract for PR auto-merge into `dev` on both
Forgejo and the GitHub bootstrap mirror. It is a target/bridge contract, not a
Phase-0 completion claim: live merge authority remains blocked until
`oya-ci-required` is posted by trusted cloud-ci/oya-ci control state and required
on the candidate SHA.

## Invariant

Auto-merge may be armed only when all of the following are true:

1. Branch protection requires exactly the cloud-ci/oya-ci required context
   `oya-ci-required` for the target branch.
2. The PR head SHA is pinned in the merge request (`head_commit_id` on Forgejo,
   `--match-head-commit` on GitHub) so a moved PR head cannot inherit stale
   approval or CI evidence.
3. The merge path performs a mergeability/conflict guard before arming
   auto-merge.
4. The merge method is linear-history compatible; P0.0 fixes `squash` for
   both forges, and script/config attempts to use merge or rebase fail closed.
5. Local `oya verify`, local `oya gate`, Buck2 affected-only output, Cargo, and
   this script's stdout are not protected-branch or Phase-0 exit authority.

## Forgejo of record path

1. **Arm the repo-level gate.** `scripts/ci/arm-auto-merge.sh` converges the
   Forgejo `dev` branch-protection rule:

   ```json
   {
     "branch_name": "dev",
     "enable_status_check": true,
     "status_check_contexts": ["oya-ci-required"]
   }
   ```

2. **Schedule the PR.** The same script can schedule a specific PR for
   auto-merge after checks pass:

   ```bash
   FORGEJO_TOKEN=*** scripts/ci/arm-auto-merge.sh \
     --pr-index 123 \
     --head-commit <expected-pr-head-sha> \
     --merge-method squash
   ```

   It posts to `POST /api/v1/repos/{owner}/{repo}/pulls/{index}/merge` with:

   ```json
   {
     "Do": "squash",
     "merge_when_checks_succeed": true,
     "delete_branch_after_merge": true,
     "head_commit_id": "<expected-pr-head-sha>"
   }
   ```

   `merge_when_checks_succeed` schedules the merge until the required check is
   green. Before the POST, the script refreshes the PR, requires the refreshed
   `head.sha` to exactly match `--head-commit`, and requires `mergeable=true`;
   stale heads, conflicts, and unresolved mergeability fail closed.
   `head_commit_id` is the Forgejo stale-head guard. `delete_branch_after_merge`
   is also fixed to `true` in P0.0 so successful auto-merges do not leave stale
   bootstrap branches behind.

## GitHub bootstrap mirror path

GitHub is the bootstrap mirror, but P0.0 requires it to converge to the same
policy while PRs are still opened there:

1. Repository auto-merge must be enabled.
2. GitHub `dev` branch protection must require `oya-ci-required`; checked-in
   target config lives in `infra/branch-protection/dev.json` and
   `.github/branch-protection.yaml`.
3. `scripts/trigger-next-queue-automerge.sh` refuses to arm GitHub auto-merge if
   live required contexts drift from the checked-in target, if the PR head is not
   GitHub-verified, if the required review check is missing, or if mergeability /
   sequential conflict simulation fails. When `origin` is Forgejo and GitHub is
   the bootstrap mirror, the sequential guard uses `--fetch-remote github-mirror`
   (or `GITHUB_FETCH_REMOTE`) so PR-head fetches come from the GitHub remote and
   merge-conflict evidence is not skipped.
4. The final GitHub arming command is:

   ```bash
   gh pr merge <number> --squash --auto --match-head-commit <expected-pr-head-sha>
   ```

## Conflict guard

`scripts/check-sequential-pr-merge-conflicts.sh` models queued GitHub PRs with
`git merge-tree --write-tree` and fails at the first conflict. Its
`--fetch-remote` option is load-bearing during the Forgejo/GitHub bootstrap
split: GitHub PR refs must be fetched from the GitHub mirror remote, not from a
Forgejo `origin` that intentionally lacks GitHub PR refs. Forgejo Tide uses the
Forgejo `mergeable` state and the same required-status/review gates;
future projected-state batching belongs in cloud-ci/oya-ci Tide, not in local
operator procedure.

## References

- `scripts/ci/arm-auto-merge.sh` — Forgejo branch-protection convergence and
  per-PR auto-merge scheduling.
- `scripts/trigger-next-queue-automerge.sh` — GitHub bootstrap mirror
  auto-merge arming.
- `scripts/check-sequential-pr-merge-conflicts.sh` — merge-conflict simulation.
- `infra/branch-protection/dev.json` — checked-in target required context.
- ADR-0363 / ADR-0513 — Forgejo substrate and cloud-ci/oya-ci Tide ownership.
