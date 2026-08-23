# Contributing

This repository is proprietary ([`LICENSE`](../LICENSE)). Contributions are accepted only from authorized people.

Procedure is [`AGENTS.md`](../AGENTS.md). On the directory you are changing, open `ADR.md`, `PRD.md`, `SPEC.md`, `PLAN.md`.

1. Isolated worktree. Install `.githooks/{pre-commit,pre-push}` into `$(git rev-parse --git-common-dir)/hooks/`.
2. SSH-signed commits.
3. Draft pull request against `origin/dev`.
4. Required context: `presubmit`.
5. Independent reviewer APPROVE, threads resolved, squash. Observation is not APPROVE.

Do not hand-edit `*.generated.json`.

Security: [`SECURITY.md`](SECURITY.md). Conduct: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).
