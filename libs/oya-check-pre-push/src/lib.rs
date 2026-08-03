//! Foundry local-verify (pre-push) command contract fitness kernel.
//!
//! Asserts the CLI-surface invariant for the canonical `oya verify`
//! command: the dev-CLI dispatch source, the Done-Definition, and the
//! local git hook all agree that `oya verify --pre-push` is the
//! canonical pre-push self-verify entry point AND that it dispatches
//! natively into the Rust `verify` handler — not into plain lightweight
//! `oya verify` or a transitional `.sh` subprocess.
//!
//! Naming justification: the crate name `oya-check-pre-push` remains
//! stable across the `repoctl` retirement so the fitness-lane id
//! `oya-governance-pre-push` (registered in
//! `registry/quality/lanes.yaml`, the branch-protection required
//! status check, and the IP-C extracted catalog) stays unchanged
//! per `feedback_no_silent_regression`. The lane semantics (local-side
//! pre-push gate) are preserved; only the canonical *command name*
//! swaps from `repoctl pre-push` to `oya verify --pre-push` because
//! `repoctl` is retired and plain `oya verify` is only the lightweight
//! local gate; the pre-push contract must run freshness, generated-face
//! settle checking, and the Buck2 affected-set before push.
//! Type `PrePushContractEvidence` retains its name to keep the
//! lane-internal API stable. Layer enum: this kernel sits on the
//! `domain` layer (port-in-kernel, ADR-0056); it performs pure
//! I/O-free static parsing of evidence strings handed in by a runner.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Canonical local-developer pre-push / pre-PR CLI surface name. The
/// Done-Definition and local git hook spell this command verbatim,
/// and the dev-CLI top-level dispatch source must route the matching
/// subcommand string to the native `commands::verify::run` handler.
///
/// Plain `oya verify` is intentionally insufficient here: it proves
/// the local gate catalog only. The `--pre-push` flag proves the
/// freshness, generated-face settle, and affected-set slice before a
/// push while protected-branch merge authority remains `oya-ci-required`.
pub const CANONICAL_PRE_PUSH_COMMAND: &str = "oya verify --pre-push";

/// Subcommand-match-arm literal that the dev-CLI dispatch source must
/// contain in its top-level command router, confirming that the
/// `verify` subcommand is wired into the CLI surface.
pub const VERIFY_SUBCOMMAND_MATCH_ARM: &str = "Some(\"verify\")";

/// Token that must appear in the dev-CLI dispatch source as proof
/// that the `verify` subcommand routes to the native
/// `commands::verify::run` handler (which forwards to
/// `gate::run` with the `run-all` arg — the canonical Rust aggregator
/// that replaces the transitional `scripts/check.sh`).
pub const NATIVE_VERIFY_DISPATCH_TOKEN: &str = "commands::verify::run";

/// Evidence bundle handed to the kernel by a runner (the dev-CLI
/// invocation reads files and forwards their text here). The kernel
/// is pure: it performs no I/O of its own (port-in-kernel, ADR-0056).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractEvidence<'a> {
    /// Markdown contents of the Done-Definition checklist; must
    /// mention `CANONICAL_PRE_PUSH_COMMAND` as a required check.
    pub done_definition_doc: &'a str, // data_class: INTERNAL_ONLY
    /// Top-level dev-CLI dispatch source
    /// (`crates/oya-dev-cli/src/lib.rs`); must contain
    /// `VERIFY_SUBCOMMAND_MATCH_ARM` AND `NATIVE_VERIFY_DISPATCH_TOKEN`
    /// so the canonical command is provably wired through to native
    /// Rust dispatch with no `.sh` subprocess interposed.
    pub cli_dispatch_source: &'a str, // data_class: INTERNAL_ONLY
    /// Local git hook script contents (the pre-push hook installed
    /// under `.git/hooks/pre-push`, or its source-of-truth file
    /// during the transitional period). Must invoke the canonical
    /// pre-push command, either by spelling
    /// `CANONICAL_PRE_PUSH_COMMAND` directly or by invoking
    /// `cargo run … -p oya-dev-cli -- verify --pre-push` (the
    /// build-from-source equivalent).
    pub hook_script: &'a str, // data_class: INTERNAL_ONLY
}

/// Successful contract report. Each boolean records the positive
/// scope that the evidence satisfied; consumers may surface this as
/// JSON evidence for fitness-lane audits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractReport {
    pub canonical_command: &'static str, // data_class: INTERNAL_ONLY
    pub native_verify_dispatch_token: &'static str, // data_class: INTERNAL_ONLY
    pub done_definition_mentions_command: bool, // data_class: INTERNAL_ONLY
    pub verify_subcommand_wired_in_cli: bool, // data_class: INTERNAL_ONLY
    pub cli_dispatches_native_verify: bool, // data_class: INTERNAL_ONLY
    pub hook_wires_full_command: bool,   // data_class: INTERNAL_ONLY
}

/// Errors returned when evidence does not satisfy the contract.
///
/// Naming justification: variants describe the missing positive
/// scope in canonical terms (no "exception" / "exempt" phrasing per
/// `feedback_no_exceptions_canonical`). Each variant is loud and
/// CI-detectable per `feedback_no_silent_regression`. `Display` is
/// implemented manually below to keep this kernel free of external
/// crate dependencies (matches the rest of the `crates/oya-check-*/`
/// kernel family).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrePushContractError {
    MissingDoneDefinitionCommand,
    MissingVerifySubcommandWiringInCli,
    MissingNativeVerifyDispatchInCli,
    MissingHookCommand,
}

/// Validate the CLI-surface contract. Pure function: parses the
/// supplied evidence strings and returns either a positive report or
/// a typed error. Performs no I/O; the runner is responsible for
/// reading source files into `PrePushContractEvidence`.
pub fn validate_pre_push_contract(
    evidence: PrePushContractEvidence<'_>,
) -> Result<PrePushContractReport, PrePushContractError> {
    if !evidence
        .done_definition_doc
        .contains(CANONICAL_PRE_PUSH_COMMAND)
    {
        return Err(PrePushContractError::MissingDoneDefinitionCommand);
    }
    if !evidence
        .cli_dispatch_source
        .contains(VERIFY_SUBCOMMAND_MATCH_ARM)
    {
        return Err(PrePushContractError::MissingVerifySubcommandWiringInCli);
    }
    if !evidence
        .cli_dispatch_source
        .contains(NATIVE_VERIFY_DISPATCH_TOKEN)
    {
        return Err(PrePushContractError::MissingNativeVerifyDispatchInCli);
    }
    if !hook_invokes_full_pre_push(evidence.hook_script) {
        return Err(PrePushContractError::MissingHookCommand);
    }

    Ok(PrePushContractReport {
        canonical_command: CANONICAL_PRE_PUSH_COMMAND,
        native_verify_dispatch_token: NATIVE_VERIFY_DISPATCH_TOKEN,
        done_definition_mentions_command: true,
        verify_subcommand_wired_in_cli: true,
        cli_dispatches_native_verify: true,
        hook_wires_full_command: true,
    })
}

/// True iff the hook script has a non-comment, non-empty line that
/// invokes the canonical local-verify command. Accepts both the
/// installed-binary form (`oya verify --pre-push …`) and the
/// build-from-source form
/// (`cargo run … -p oya-dev-cli -- verify --pre-push …`) so the
/// same hook works in a workspace clone and in a system with `oya` on
/// PATH.
///
/// Naming justification: function name is snake_case; the predicate
/// is stated positively (no "missing" / "exception" phrasing).
fn hook_invokes_full_pre_push(hook_script: &str) -> bool {
    hook_script.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return false;
        }
        trimmed.contains(CANONICAL_PRE_PUSH_COMMAND)
            || trimmed.contains("oya-dev-cli -- verify --pre-push")
            || trimmed.contains("oya-dev-cli --bin oya -- verify --pre-push")
    })
}

impl fmt::Display for PrePushContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDoneDefinitionCommand => write!(
                formatter,
                "Done-Definition does not require `{CANONICAL_PRE_PUSH_COMMAND}`"
            ),
            Self::MissingVerifySubcommandWiringInCli => write!(
                formatter,
                "dev-CLI dispatch source does not contain the verify subcommand match arm \
                 (missing token `{VERIFY_SUBCOMMAND_MATCH_ARM}`)"
            ),
            Self::MissingNativeVerifyDispatchInCli => write!(
                formatter,
                "dev-CLI dispatch source does not route to the native verify handler \
                 (missing token `{NATIVE_VERIFY_DISPATCH_TOKEN}`)"
            ),
            Self::MissingHookCommand => write!(
                formatter,
                "pre-push hook script does not invoke `{CANONICAL_PRE_PUSH_COMMAND}`"
            ),
        }
    }
}

impl std::error::Error for PrePushContractError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_grounded_pre_push_contract() {
        let report = validate_pre_push_contract(valid_evidence()).expect("contract validates");

        assert_eq!(report.canonical_command, "oya verify --pre-push");
        assert_eq!(report.native_verify_dispatch_token, "commands::verify::run");
        assert!(report.done_definition_mentions_command);
        assert!(report.verify_subcommand_wired_in_cli);
        assert!(report.cli_dispatches_native_verify);
        assert!(report.hook_wires_full_command);
    }

    #[test]
    fn rejects_missing_done_definition_command() {
        let mut evidence = valid_evidence();
        evidence.done_definition_doc = "D12: run something else";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingDoneDefinitionCommand)
        );
    }

    #[test]
    fn rejects_missing_verify_subcommand_wiring() {
        let mut evidence = valid_evidence();
        // The dispatch source routes a different subcommand but does
        // not contain the `Some("verify")` match arm.
        evidence.cli_dispatch_source =
            "Some(\"check\") => commands::check::run(args.collect(), &usage()),\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingVerifySubcommandWiringInCli)
        );
    }

    #[test]
    fn rejects_cli_without_native_verify_dispatch() {
        let mut evidence = valid_evidence();
        // The dispatch source wires the `Some("verify")` match arm
        // but routes to a non-native handler (proves the predicate
        // is independent of the subcommand wiring check).
        evidence.cli_dispatch_source =
            "Some(\"verify\") => some_subprocess_call(args, &usage()),\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingNativeVerifyDispatchInCli)
        );
    }

    #[test]
    fn rejects_hook_that_does_not_invoke_canonical_command() {
        let mut evidence = valid_evidence();
        evidence.hook_script = "cargo run -p oya-dev-cli -- gate run-all\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingHookCommand)
        );
    }

    #[test]
    fn rejects_hook_that_stops_at_plain_verify() {
        let mut evidence = valid_evidence();
        evidence.hook_script = "oya verify \"$@\"\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingHookCommand)
        );
    }

    #[test]
    fn accepts_hook_that_uses_cargo_run_form() {
        // Build-from-source form: the hook in a fresh clone calls
        // `cargo run -p oya-dev-cli -- verify --pre-push` because
        // `oya` is not yet on PATH. Both forms must satisfy the
        // contract.
        let mut evidence = valid_evidence();
        evidence.hook_script =
            "cargo run -q -p oya-dev-cli -- verify --pre-push \"$@\" || exit 1\n";

        let report =
            validate_pre_push_contract(evidence).expect("cargo-run hook satisfies the contract");
        assert!(report.hook_wires_full_command);
    }

    fn valid_evidence() -> PrePushContractEvidence<'static> {
        PrePushContractEvidence {
            done_definition_doc: "- [ ] D12 `oya verify --pre-push` passes before push; `oya-ci-required` remains protected PR authority.",
            cli_dispatch_source: "match args.next().as_deref() {\n    \
                 Some(\"verify\") => commands::verify::run(args.collect(), &usage()),\n}\n",
            hook_script: "oya verify --pre-push \"$@\" || exit 1\n\
                          # canonical: oya verify --pre-push\n",
        }
    }
}
