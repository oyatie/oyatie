# ADR-0516 Supersession Note

ADR-0516 supersedes this document for interim dev-lane unlock. The current interim path is GitHub/GitHub Actions as a temporary lane-unlocker, with no Jenkins, no Forgejo, and no ArgoCD interim authority. Buck2 remains build/test/check authority; native cutover remains cloud native, Kubernetes-native, and hyperscaler native. Historical details below are retained for provenance and native-cutover comparison only.

# Forge of Record — CI Gating (ADR-0363 / ADR-0513)

Forgejo (self-hosted at `forgejo.oya-forge.svc.cluster.local`) is the target
**gating forge of record** for all PR merges to `dev`. GitHub
(`github.com/jason931225/oyatie`) is the bootstrap mirror while Forgejo reachability
and cloud-ci/oya-ci cutover evidence are still being established. During P0.0,
both forges must converge to the same required-context contract so neither path
can regress to Cargo-named lanes or local `oya` output as merge authority.

This document is a target/bridge contract, not a green claim. GitHub `dev`
required-status contexts were synced to `oya-ci-required` on 2026-06-02, but
Phase-0 remains red until that context is posted from trusted cloud-ci/oya-ci
control state on candidate SHAs and Forgejo reaches the same live contract.

## How gating works

1. The ADR-0516 GitHub/GitHub Actions temporary bridge runs Buck2-owned
   build/test/gate targets from trusted controller/trunk state and posts the
   commit status context `oya-ci-required`.
2. Forgejo branch protection for `dev` requires `oya-ci-required` before a PR can
   merge. `scripts/ci/arm-auto-merge.sh` is the idempotent bridge script for
   converging that branch-protection rule and scheduling per-PR auto-merge.
3. GitHub branch protection mirrors the same required-context list for bootstrap
   PRs. `scripts/trigger-next-queue-automerge.sh` refuses to arm GitHub
   auto-merge if live contexts drift from `infra/branch-protection/dev.json`.
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
| `oya-ci-required` | cloud-ci/oya-ci controller or explicitly classified bridge lane | Only required context allowed to satisfy protected-branch merge authority during P0.0 cutover. |

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
