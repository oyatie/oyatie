# GitHub ops notes for Lane D — 2026-06-01

## Scope

Lane C note for the oya-ci weekly parallel-lane run. This records current
GitHub substrate discovery and verification handoff inputs without secrets.

## Observed local state

- Current `origin` in this worker worktree points at
  `https://www.github.com/jason931225/oyatie`; workers must add or select the
  GitHub (interim) remote before pushing or opening GitHub pull requests.
- Target branch remains `dev`.
- GitHub token/credential references must be named only; do not paste token
  values, bearer strings, webhook secrets, or raw authorization headers.
- `infra/forge/jenkins-github-token.secret.template.yaml` names the Jenkins
  `github-ci-token` credential template; the live secret is not committed.

## Status-context sources for Lane D

- Required branch-protection contexts are declared in
  `infra/branch-protection/dev.json`.
- Jenkins-reported contexts are declared in
  `infra/ci/jenkins/reported-status-contexts.json` and posted by the Jenkins
  shared library. Lane C does not own `infra/ci/**`, so mismatches should be
  treated as an integration/review input rather than fixed in this lane.
- `infra/forge/README.md` still describes the GitHub/Jenkins wiring and the
  no-secret credential template.

## Verification contract

- Rust lanes must run affected checks with `RUSTUP_TOOLCHAIN=1.96.0`, edition
  2024 formatting, and Buck2 `test`, `build`, `[check]`, and `[clippy.txt]`
  targets where applicable.
- This Lane C documentation slice ran docs-focused checks and a Buck2 smoke
  build only; no tide/controller Rust targets were changed.
- `oya gate run-all` should remain leader/integration-owned to avoid concurrent
  worker-lane gate runs.

## Blockers / risks to surface

- No GitHub remote or live API endpoint was configured in this worker worktree.
- No live token probe was run, so there is no live GitHub reachability proof
  from this worker.
- Current local evidence is sufficient for a no-secret Lane C handoff, not for
  a production mutation or Jenkins cutover.
