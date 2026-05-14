use std::process::ExitCode;

// Grounded dual-entrypoint shim: docs/AGENTS.md D12 names `repoctl pre-push`
// as the required local gate, while the greenfield bootstrap keeps the CLI
// runtime in one compatibility crate until the ROADMAP persona split is
// promoted. This binary intentionally shares the same library entrypoint as
// `oya` so routing, usage text, and pre-push behavior cannot drift.
fn main() -> ExitCode {
    oya_dev_cli::run_cli_from_env()
}
