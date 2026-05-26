# Self-hosted Forgejo substrate (T1 — ADR-0363)

The change-coordination substrate per ADR-0363: **git + Jenkins + self-hosted
Forgejo**. This directory is the IaC for the Forgejo forge and its wiring to the
Jenkins CI lane, which together **retire the `enforce_admins`-toggle admin-merge
seam** (the temporary bootstrap mechanism used through PRs #181–#188).

## Why
Branch protection on `dev` requires 15 status contexts, but nothing posted them
— so every merge briefly disabled `enforce_admins` and used `--admin`. T1 makes
the checks *real*: Jenkins posts each context to Forgejo's Commit Status API, and
the combined status gates the merge (auto-merge when green). No admin override.

## Contents
- `forgejo.yaml` — Forgejo (GPLv3+, OSI-clean) Deployment/Service/PVC, rootless
  image, hardened (non-root, dropped caps, seccomp, `INSTALL_LOCK`), SQLite.
- `forgejo-argocd-app.yaml` — ArgoCD Application that reconciles `forgejo.yaml`.
- `jenkins-forgejo-token.secret.template.yaml` — template for the Jenkins
  `forgejo-ci-token` string credential (real token created via kubectl; never
  committed — gitleaks enforces this).

## Commit-status wiring
`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy` posts the 14 CI-produced
required contexts (`pending` → `success`/`failure`) via the Forgejo API using the
`forgejo-ci-token` credential. `oya-pr-review` is posted separately by the
reviewer agent. Context list is kept in sync with `.github/branch-protection.yaml`
by the `oya-governance-protection-context-match` gate.

## Verified (this standup, on the colima k3s farm)
- Forgejo `11.0.14` Running in ns `oya-forge` (1/1, healthz green).
- Admin `oya-admin` created; repo `oya-admin/oyatie` initialised.
- **Commit-status proof**: POSTed `cargo-check=success` to a commit → the
  combined status reads `success` (exactly what branch-protection consults).

## Remaining (this task → its follow-ups)
1. Create the live `forgejo-ci-token` Jenkins credential (kubectl, see template).
2. End-to-end: a real PR through Jenkins posts all 14 → Forgejo auto-merges on green.
3. GitHub → Forgejo repo migration (flip primary; ADR-0247 post-bootstrap) — the
   last, deliberate cutover; GitHub stays the bootstrap host until then.
