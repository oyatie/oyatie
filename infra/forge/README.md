# Self-hosted Forgejo substrate (T1 — ADR-0363)

The change-coordination substrate per ADR-0363: **git + required status checks +
self-hosted Forgejo**. This directory is IaC for the Forgejo forge and its wiring
to the CI bridge. Cargo-named contexts are historical and must not be required.

## Why

Branch protection on `dev` requires the target `oya-ci-required` context. During
cutover, Jenkins or another trusted bridge may post that context; Phase-0 exit
target is cloud-ci/oya-ci posting it from trusted controller/trunk state. No admin
override is branch-protection authority.

## Contents

- `forgejo.yaml` — Forgejo Deployment/Service/PVC, rootless image, hardened
  (non-root, dropped caps, seccomp, `INSTALL_LOCK`), SQLite.
- `forgejo-argocd-app.yaml` — ArgoCD Application that reconciles `forgejo.yaml`.
- `jenkins-forgejo-token.secret.template.yaml` — template for the bridge status
  token (real token created via kubectl; never committed).

## Commit-status wiring

`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy` posts `oya-ci-required`
(`pending` → `success`/`failure`) using the Forgejo API. The canonical target list
is `infra/ci/jenkins/reported-status-contexts.json` and `infra/branch-protection/dev.json`.

## Verified locally / target evidence

- Forgejo substrate manifests exist under this directory.
- Target status context is `oya-ci-required`.
- Buck2 authority policy forbids legacy tool-specific status contexts in the active
  branch-protection and CI inventory.

## Remaining

1. Create the live bridge token in the cluster.
2. Cut over the trusted cloud-ci/oya-ci producer for `oya-ci-required`.
3. End-to-end PR: Buck2 authority policy + affected Buck2 build/test pass, status
   posts on the candidate SHA, and the merge queue admits only that green context.
