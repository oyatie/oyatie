# Oyatie

Owned hyperscale cloud, written in Rust, plus first-party apps that consume that cloud as tenant #0.

## Law

On a capability directory or `app/<product>/`, open `ADR.md`, `PRD.md`, `SPEC.md`, `PLAN.md`. Those four files are the law for that path. Session procedure is [`AGENTS.md`](AGENTS.md).

## Merge

Protected pull request against `dev`. Required context: `presubmit`. Independent reviewer APPROVE, threads resolved, then squash. Observation (logs, CI green) is not APPROVE.

```sh
cargo fmt --all --check
cargo nextest run --locked --workspace --profile ci
```

Install `.githooks/{pre-commit,pre-push}` into `$(git rev-parse --git-common-dir)/hooks/`.

## License

Proprietary. See [`LICENSE`](LICENSE).
