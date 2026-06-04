# Auto-Merge Flow — GitHub Lane Unlocker and Native SCM Cutover

ADR-0516 is the active interim direction: GitHub and GitHub Actions unlock
parallel work while the native Oyatie SCM/CI/CD substrates are built. Retired
external SCM/CI/CD substrates are not interim or permanent authority.

## Invariant

Auto-merge may be armed only when all of the following are true:

1. During the temporary GitHub/GitHub Actions lane unlocker, `dev` requires the
   `github-lane-unlocker-required` aggregate context produced by the trusted
   `github-lane-unlocker-ci-cd` workflow.
2. Native cutover requires `oya-ci-required` posted by trusted Oya CI control
   state on the candidate SHA. The native SCM is the pure-Rust,
   Sapling-inspired Oyatie SCM with GitHub and git-protocol adapters.
3. The PR head SHA is pinned with `--match-head-commit` so moved heads cannot
   inherit stale approval or CI evidence.
4. The merge path performs review, live required-context, signature, and
   sequential merge-conflict guards before arming auto-merge.
5. The merge method is linear-history compatible; the bridge fixes `squash` and
   rejects merge/rebase drift.
6. Local `oya verify`, local `oya gate`, Buck2 affected-only output, Cargo, and
   operator memory are not protected-branch or Phase-0 exit authority.

## GitHub temporary bridge path

1. Repository auto-merge is enabled only for the temporary bridge.
2. GitHub `dev` branch protection requires `github-lane-unlocker-required`;
   checked-in target config lives in `infra/branch-protection/dev.json` and
   `.github/branch-protection.yaml`.
3. `scripts/trigger-next-queue-automerge.sh` refuses to arm GitHub auto-merge
   when live required contexts drift from checked-in target config, when the PR
   head is not GitHub-verified, when the required review check is missing, or
   when sequential merge-conflict simulation fails.
4. If `origin` is not the GitHub mirror, the sequential guard uses
   `--fetch-remote github-mirror` or `GITHUB_FETCH_REMOTE` so `refs/pull/*`
   evidence comes from the GitHub remote.
5. The final arming command is:

   ```bash
   gh pr merge <number> --squash --auto --match-head-commit <expected-pr-head-sha>
   ```

## Native SCM/CI cutover path

The native path keeps the proven Prow/Tide admission shape but owns the seams in
Oyatie Rust services:

- SCM: pure-Rust, Sapling-inspired Oyatie SCM with GitHub and git-protocol
  adapters for publication/interoperability.
- CI: Oya CI posts source-bound `oya-ci-required` evidence from trusted
  controller state after Buck2 build/test/check execution.
- CD: the release conveyor owns progressive delivery, rollback, policy, and
  audit instead of adopting retired external SCM/CI/CD substrates as interim
  control planes.

Until that evidence exists, native cutover remains a target contract rather than
an operational merge authority.

## Conflict guard

`scripts/check-sequential-pr-merge-conflicts.sh` is a compatibility entrypoint
for the Rust implementation in `scripts/check-sequential-pr-merge-conflicts.rs`.
It models queued GitHub PRs with `git merge-tree --write-tree` and fails at the
first conflict. Its `--fetch-remote` option is load-bearing during the temporary
bridge: GitHub PR refs must be fetched from the GitHub mirror remote when the
local `origin` is not GitHub.

Future projected-state batching belongs in Oya CI/Tide, not in local operator
procedure.

## References

- `scripts/trigger-next-queue-automerge.sh` — GitHub temporary bridge
  auto-merge arming.
- `scripts/check-sequential-pr-merge-conflicts.sh` — compatibility entrypoint
  for the Rust merge-conflict simulator.
- `scripts/check-sequential-pr-merge-conflicts.rs` — Rust merge-conflict
  simulator used by Buck2 checks.
- `infra/branch-protection/dev.json` — checked-in temporary required context.
- `specs/github-lane-unlocker-bridge.json` — GitHub bridge and native cutover
  context boundary.
- `specs/retired-external-substrate-registry.json` — tombstone registry for
  retired external SCM/CI/CD substrates.
