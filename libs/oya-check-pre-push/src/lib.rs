//! Foundry local-verify (pre-push) command contract fitness kernel.
//!
//! Asserts the local preflight invariant after developer-CLI retirement:
//! the Done-Definition, native workflow evidence, and local git hook all
//! agree that Buck2 authority checks are the canonical pre-push / pre-PR
//! verification entry point. GitHub/Prow status publication remains external
//! to this pure kernel.
//!
//! Naming justification: the crate name `oya-check-pre-push` remains
//! stable across the `repoctl` retirement so the fitness-lane id
//! `oya-governance-pre-push` (registered in
//! `registry/quality/lanes.yaml`, the branch-protection required
//! status check, and the IP-C extracted catalog) stays unchanged
//! per `feedback_no_silent_regression`. The lane semantics (local-side
//! pre-push gate) are preserved; the canonical command is now a Buck2
//! authority pair rather than the retired local wrapper.
//! Type `PrePushContractEvidence` retains its name to keep the
//! lane-internal API stable. Layer enum: this kernel sits on the
//! `domain` layer (port-in-kernel, ADR-0056); it performs pure
//! I/O-free static parsing of evidence strings handed in by a runner.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Canonical local-developer pre-push / pre-PR command.
pub const CANONICAL_PRE_PUSH_COMMAND: &str =
    "buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check";

/// Token proving the native workflow keeps the repo hygiene authority target.
pub const VERIFY_SUBCOMMAND_MATCH_ARM: &str = "repo-hygiene-automation-check";

/// Token proving the native workflow keeps the Buck2 authority policy target.
pub const NATIVE_VERIFY_DISPATCH_TOKEN: &str = "buck2-authority-policy-check";

/// Evidence bundle handed to the kernel by a runner (the dev-CLI
/// invocation reads files and forwards their text here). The kernel
/// is pure: it performs no I/O of its own (port-in-kernel, ADR-0056).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractEvidence<'a> {
    /// Markdown contents of the Done-Definition checklist; must
    /// mention `CANONICAL_PRE_PUSH_COMMAND` as a required check.
    pub done_definition_doc: &'a str, // data_class: INTERNAL_ONLY
    /// Native workflow or source text; must contain both authority tokens so
    /// the local preflight cannot drift back to a retired wrapper.
    pub cli_dispatch_source: &'a str, // data_class: INTERNAL_ONLY
    /// Local git hook script contents (the pre-push hook installed
    /// under `.git/hooks/pre-push`, or its source-of-truth file
    /// during the transitional period). Must invoke the canonical Buck2
    /// pre-push / pre-PR command directly.
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
/// invokes the canonical Buck2 local-verify command.
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
                "native workflow source does not contain the repo-hygiene authority token \
                 (missing token `{VERIFY_SUBCOMMAND_MATCH_ARM}`)"
            ),
            Self::MissingNativeVerifyDispatchInCli => write!(
                formatter,
                "native workflow source does not contain the Buck2 authority-policy token \
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

        assert_eq!(
            report.canonical_command,
            "buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check"
        );
        assert_eq!(
            report.native_verify_dispatch_token,
            "buck2-authority-policy-check"
        );
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
    fn rejects_missing_repo_hygiene_authority_token() {
        let mut evidence = valid_evidence();
        evidence.cli_dispatch_source = "buck2 build //:buck2-authority-policy-check\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingVerifySubcommandWiringInCli)
        );
    }

    #[test]
    fn rejects_missing_buck2_authority_policy_token() {
        let mut evidence = valid_evidence();
        evidence.cli_dispatch_source = "buck2 build //:repo-hygiene-automation-check\n";

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
    fn rejects_hook_that_uses_retired_dev_cli_form() {
        let mut evidence = valid_evidence();
        evidence.hook_script =
            "cargo run -q -p oya-dev-cli -- verify --ci-required \"$@\" || exit 1\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingHookCommand)
        );
    }

    fn valid_evidence() -> PrePushContractEvidence<'static> {
        PrePushContractEvidence {
            done_definition_doc: "- [ ] D12 `buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check` passes.",
            cli_dispatch_source: "buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check\n",
            hook_script: "buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check\n",
        }
    }
}
