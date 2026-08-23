# Contributing

This repository is proprietary ([`LICENSE`](../LICENSE)). Contributions are accepted only from authorized people.

Procedure is [`AGENTS.md`](../AGENTS.md). On the directory you are changing, open `ADR.md`, `PRD.md`, `SPEC.md`, `PLAN.md`.

1. Isolated worktree. Install `.githooks/{pre-commit,pre-push}` into `$(git rev-parse --git-common-dir)/hooks/`.
2. SSH-signed commits.
3. Draft pull request against `origin/dev`.
4. Required context: `presubmit`.
5. Independent reviewer APPROVE, threads resolved, squash. Observation is not APPROVE.

Tests live in the crate they cover, not in a service-root or repo-root `tests/` folder.

- Unit: `#[cfg(test)]` next to the code in `src/`.
- Integration: that crate’s `tests/*.rs` (in-process, public API of one package).
- End-to-end / live: the facade crate’s `tests/e2e/` or the adapter crate’s `tests/live_*.rs`, `#[ignore]`, nextest profile `live`. Core has no IO, so it has no e2e.

Do not hand-edit `*.generated.json`.

Security: [`SECURITY.md`](SECURITY.md). Conduct: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).
