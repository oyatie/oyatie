# GitHub (interim) substrate (T1 — ADR-0363)

The change-coordination substrate per ADR-0363: **git + Jenkins + self-hosted
GitHub**. This directory is the IaC for the GitHub forge and its wiring to the
Jenkins CI lane, which together **retire the `enforce_admins`-toggle admin-merge
seam** (the temporary bootstrap mechanism used through PRs #181–#188).

## Why

Branch protection on `dev` requires 15 status contexts, but nothing posted them
— so every merge briefly disabled `enforce_admins` and used `--admin`. T1 makes
the checks *real*: Jenkins posts each context to GitHub's Commit Status API, and
the combined status gates the merge (auto-merge when green). No admin override.

## Contents

- `github.yaml` — GitHub (GPLv3+, OSI-clean) Deployment/Service/PVC, rootless
  image, hardened (non-root, dropped caps, seccomp, `INSTALL_LOCK`), SQLite.
- `github-argocd-app.yaml` — ArgoCD Application that reconciles `github.yaml`.
- `jenkins-github-token.secret.template.yaml` — template for the Jenkins
  `github-ci-token` string credential (real token created via kubectl; never
  committed — gitleaks enforces this).

## Commit-status wiring

`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy` posts the 14 CI-produced
required contexts (`pending` → `success`/`failure`) via the GitHub API using the
`github-ci-token` credential. `oya-pr-review` is posted separately by the
reviewer agent. Context list is kept in sync with `.github/branch-protection.yaml`
by the `oya-governance-protection-context-match` gate.

## Verified (this standup, on the colima k3s farm)

- GitHub `11.0.14` Running in ns `oya-forge` (1/1, healthz green).
- Admin `oya-admin` created; repo `oya-admin/oyatie` initialised.
- **Commit-status proof**: POSTed `cargo-check=success` to a commit → the
  combined status reads `success` (exactly what branch-protection consults).

## 2026-06-01 Lane C operator note

For the weekly oya-ci parallel-lane run, this GitHub substrate remains
GitHub-only for pull requests against `dev`. If a local worktree still has a
GitHub `origin`, add or select the GitHub (interim) remote before pushing; do
not open GitHub PRs or use GitHub merge commands for this lane. Record credential
variable names and redacted transcripts only, never token values or raw
authorization headers. Jenkins remains the bridge until Phase-1 parallel-run
evidence and founder/operator approval authorize a cutover.

Lane D should compare `infra/branch-protection/dev.json` with the Jenkins
reported-context inventory when building the shared evidence packet. Lane C does
not own `infra/ci/**`, so status-context producer changes require a separate
integration-approved scope.

## Remaining (this task → its follow-ups)

1. Create the live `github-ci-token` Jenkins credential (kubectl, see template).
2. End-to-end: a real PR through Jenkins posts all 14 → GitHub auto-merges on green.
3. GitHub → GitHub repo migration (flip primary; ADR-0247 post-bootstrap) — the
   last, deliberate cutover; GitHub stays the bootstrap host until then.
