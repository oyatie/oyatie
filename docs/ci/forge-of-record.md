# ADR-0516 Supersession Note

ADR-0516 supersedes this document for interim dev-lane unlock. The current interim path is GitHub/GitHub Actions as a temporary lane-unlocker, with no Jenkins, no Forgejo, and no ArgoCD interim authority. Buck2 remains build/test/check authority; native cutover remains cloud native, Kubernetes-native, and hyperscaler native. Historical details below are retained for provenance and native-cutover comparison only.

# Forge of Record — CI Gating (ADR-0363 / ADR-0513)

Forgejo (self-hosted at `forgejo.oya-forge.svc.cluster.local`) is the target
**gating forge of record** for all PR merges to `dev`. GitHub
(`github.com/jason931225/oyatie`) is the bootstrap mirror while Forgejo reachability
and cloud-ci/oya-ci cutover evidence are still being established. During P0.0,
the temporary GitHub bridge uses `github-lane-unlocker-required` while native
Forgejo/cloud-ci cutover remains `oya-ci-required`. Neither path may regress to
Cargo-named lanes or local `oya` output as merge authority.

This document is a target/bridge contract, not a green claim. During ADR-0516,
GitHub `dev` requires `github-lane-unlocker-required` from the GitHub Actions
aggregate workflow so the manual `oya-ci-required` bridge is not needed. Phase-0
remains red until native `oya-ci-required` is posted from trusted cloud-ci/oya-ci
control state on candidate SHAs and Forgejo reaches that live contract.

## How gating works

1. The ADR-0516 GitHub/GitHub Actions temporary bridge runs Buck2-owned
   build/test/gate targets from trusted controller/trunk state and exposes the
   required check `github-lane-unlocker-required`.
2. Forgejo branch protection for native cutover requires `oya-ci-required` before
   a PR can merge. `scripts/ci/arm-auto-merge.sh` is the idempotent bridge script
   for converging that branch-protection rule and scheduling per-PR auto-merge.
3. GitHub branch protection requires the temporary
   `github-lane-unlocker-required` context for bootstrap PRs.
   `scripts/trigger-next-queue-automerge.sh` refuses to arm GitHub auto-merge if
   live contexts drift from `infra/branch-protection/dev.json`.
4. Auto-merge is armed only with a pinned PR head SHA and fixed squash
   method: Forgejo uses `head_commit_id`; GitHub uses
   `gh pr merge --squash --auto --match-head-commit`.
   Forgejo scheduling refreshes the PR first and requires exact `head.sha` match
   plus `mergeable=true`.
5. Merge-conflict guards must run before auto-merge arming. GitHub uses
   `scripts/check-sequential-pr-merge-conflicts.sh`; if `origin` is the Forgejo
   remote and GitHub is the bootstrap mirror, pass `--fetch-remote github-mirror`
   or set `GITHUB_FETCH_REMOTE` so PR refs are fetched from GitHub. Forgejo Tide
   gates on Forgejo's `mergeable` state until projected-state batching lands in
   cloud-ci/oya-ci.

## Required status context

| Context | Producer | Authority boundary |
|---|---|---|
| `github-lane-unlocker-required` | GitHub Actions `github-lane-unlocker-ci-cd` aggregate workflow | Temporary ADR-0516 GitHub bridge authority; avoids manual `oya-ci-required` statuses while native SCM/CI is unfinished. |
| `oya-ci-required` | cloud-ci/oya-ci controller | Native cutover authority after trusted cloud-ci/oya-ci can post source-bound candidate-SHA evidence. |

Legacy Cargo-named contexts (`cargo-fmt`, `cargo-check`, `cargo-clippy`,
`cargo-nextest`, `cargo-deny`) and local `oya-verify` / `oya-gate` output are
historical or advisory evidence only. They must not be required branch-protection
contexts and must not be described as Phase-0 exit authority.

## Auto-merge after CI

- Forgejo: `scripts/ci/arm-auto-merge.sh --pr-index <n> --head-commit <sha>`
  posts `merge_when_checks_succeed=true`, fixed
  `delete_branch_after_merge=true`, and fixed `Do=squash` to Forgejo's PR merge endpoint.
- GitHub: `scripts/trigger-next-queue-automerge.sh` eventually executes
  `gh pr merge <n> --squash --auto --match-head-commit <sha>` after live
  required-context, review, signature, and conflict checks pass.

## References

- ADR-0363: Forgejo substrate and retirement of bespoke agentic-VCS wrappers.
- ADR-0513: cloud-ci/oya-ci Prow-shaped controller and Tide ownership.
- `infra/branch-protection/dev.json`: machine-readable required context target.
- `.github/branch-protection.yaml`: GitHub temporary bridge target.
- `infra/ci/jenkins/reported-status-contexts.json`: bridge-reported contexts.
- `docs/ci/auto-merge-flow.md`: per-forge auto-merge flow.
