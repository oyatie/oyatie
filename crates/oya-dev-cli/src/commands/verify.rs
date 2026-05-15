//! `oya verify` — local-developer verification entry point.
//!
//! Per user directive 2026-05-15 ("pre-push should really just be part of
//! some other check/validate"), `pre-push` is not a distinct concept; it
//! is the local-side mirror of the verification gate. This subcommand is
//! the canonical fold of the legacy `scripts/hooks/pre-push-repoctl.sh`
//! invocation into the Rust CLI surface.
//!
//! Naming justification: top-level subcommand `verify` (kebab-case);
//! module file `src/commands/verify.rs` (snake_case, no redundant
//! `_command` suffix because it lives under `commands/`); handler
//! `run` (snake_case verb). Conforms to ADR-0105/0106/0107 v4 BNF
//! and the 13-value layer enum at
//! `crates/oya-foundry-fitness-predictable-naming-kernel::ALLOWED_ROLES`.
//!
//! Behaviour: dispatches to `gate run-all`, the canonical pre-merge
//! gate aggregator (Wave 2 replacement for `scripts/check.sh`, audit row
//! B-1 in `evidence/audits/shell-python-replacement-audit-2026-05-15.md`).
//! Optional `--fast` short-circuits the deferred gates list; default is
//! the same lane set the aggregator runs in non-deferred mode.
//!
//! This subcommand is the canonical replacement target for the local
//! `pre-push` git hook (`scripts/hooks/pre-push-repoctl.sh`). The hook
//! script is updated in the same change to invoke
//! `cargo run -q -p oya-dev-cli -- verify`; full deletion of the .sh is
//! tracked under the transitional-.sh-removal sweep (audit row B-12).

use std::process::ExitCode;

use super::gate;

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    // `oya verify` is intentionally a thin alias for `oya gate run-all`.
    // Any positional/flag passthrough is forwarded verbatim so the local
    // pre-push hook can call `oya verify --include-deferred` etc. without
    // a parallel parser. Unknown flags surface via `gate run-all`'s own
    // parser (the canonical error path).
    let mut forwarded = Vec::with_capacity(args.len() + 1);
    forwarded.push("run-all".to_string());
    forwarded.extend(args);
    gate::run(forwarded, usage)
}

#[cfg(test)]
mod tests {
    // Behavioural coverage for `verify` is provided by the underlying
    // `gate::run_all` tests (see `commands/gate/run_all.rs`) because
    // `verify` is a thin, transformation-only alias: it prepends
    // `"run-all"` to the argument vector and delegates to `gate::run`.
    //
    // We intentionally avoid an integration-style smoke test in this
    // module because invoking `gate::run_all` from a unit test would
    // shell out to `cargo run …` for ~38 validate lanes, which is the
    // expensive aggregator path and inappropriate for the kernel-tier
    // test budget (ADR-0083 Tier 1).
}
