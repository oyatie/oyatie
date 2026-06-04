# Self-hosted Forgejo substrate history (T1 — ADR-0363)

ADR-0516 supersedes this directory for interim development-lane unlock. The
current temporary SCM/CI path is GitHub + GitHub Actions; native cutover targets
Oyatie SCM and oya-ci/release-conveyor seams. Files here are historical local
substrate reference only until a separate native-SCM lane rewrites or removes
them. Cargo-named contexts are historical and must not be required.

## Why

During the GitHub bridge, `dev` requires `github-lane-unlocker-required`.
`oya-ci-required` remains the native cutover destination after trusted
controller/trunk evidence exists. No retired external SCM/CI/CD substrate bridge
is interim branch-protection authority.

## Contents

- `forgejo.yaml` — Forgejo Deployment/Service/PVC, rootless image, hardened
  (non-root, dropped caps, seccomp, `INSTALL_LOCK`), SQLite.
- `forgejo-argocd-app.yaml` — historical ArgoCD Application reference retained
  for provenance until the native-SCM cleanup lane removes or rewrites it.

## Commit-status wiring

Historical retired CI-to-local-SCM commit-status wiring is retired. The active bridge
context is recorded in `specs/github-lane-unlocker-bridge.json`,
`.github/branch-protection.yaml`, and `infra/branch-protection/dev.json`.

## Verified locally / target evidence

- Forgejo substrate manifests exist under this directory.
- Temporary target status context is `github-lane-unlocker-required`; native
  cutover target remains `oya-ci-required`.
- Buck2 authority policy forbids legacy tool-specific status contexts in the active
  branch-protection and CI inventory.

## Remaining

1. Remove or rewrite the remaining Forgejo/ArgoCD historical local-substrate
   manifests in dedicated cleanup lanes.
2. Cut over the trusted native oya-ci producer for `oya-ci-required`.
3. End-to-end PR: Buck2 authority policy + affected Buck2 build/test pass, status
   posts on the candidate SHA, and the merge queue admits only that green context.
