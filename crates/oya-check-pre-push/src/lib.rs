//! Foundry pre-push command contract fitness kernel.
//!
//! Asserts the CLI-surface invariant for the canonical `repoctl pre-push`
//! command: the `repoctl` Rust source (canonical surface) and the
//! supporting documents (Done-Definition, CLI manifest, local hook) all
//! agree that `repoctl pre-push` is the canonical local-developer
//! verification entry point AND that its implementation dispatches
//! natively into the Rust `verify` / `gate run-all` surface — not into
//! a transitional `.sh` subprocess.
//!
//! Naming justification: type `PrePushContractEvidence` and function
//! `validate_pre_push_contract` keep the existing canonical names so the
//! external CLI-surface contract is stable across the .sh-removal
//! transition (per `feedback_no_silent_regression`: the kernel's
//! published API is a fitness-lane contract, and the lane id
//! `oya-foundry-fitness-pre-push` stays the same). The renamed error
//! variant `MissingNativeVerifyDispatchInRepoctlSource` describes the
//! new positive scope — repoctl source must dispatch into the native
//! verify surface — and replaces the legacy
//! `MissingContractCheckInCheckScript` variant whose source-of-truth was
//! the transitional `scripts/check.sh` body. Layer enum: this kernel
//! sits on the `domain` layer (port-in-kernel, ADR-0056); it performs
//! pure I/O-free static parsing of evidence strings handed in by a
//! runner.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Canonical local-developer pre-push CLI surface name. The Done-Definition,
/// local git hook, and aggregator dispatch all spell this command verbatim.
pub const CANONICAL_PRE_PUSH_COMMAND: &str = "repoctl pre-push";

/// Token that must appear in the `repoctl` source as proof that the
/// `pre-push` (non-`--verify-contract`) branch dispatches into the
/// native Rust `verify` / `gate run-all` surface instead of subprocessing
/// a transitional `scripts/check.sh`.
///
/// Naming justification: snake_case Rust identifier matches the public
/// function on the `commands::verify` module (`run`) and stays stable
/// across the .sh-removal sub-IPs (IP-B routes the call through it).
pub const REPOCTL_NATIVE_VERIFY_DISPATCH_TOKEN: &str = "commands::verify::run";

/// Marker that must appear in the CLI manifest to prove the `repoctl`
/// binary is declared.
pub const REPOCTL_BIN_NAME_DECLARATION: &str = "name = \"repoctl\"";

/// Evidence bundle handed to the kernel by a runner (the dev-CLI invocation
/// in `oya-dev-cli` reads files and forwards their text here). The kernel
/// is pure: it performs no I/O of its own (port-in-kernel, ADR-0056).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractEvidence<'a> {
    /// Markdown contents of the Done-Definition checklist; must mention
    /// `CANONICAL_PRE_PUSH_COMMAND` as a required check.
    pub done_definition_doc: &'a str, // data_class: INTERNAL_ONLY
    /// CLI manifest contents (e.g. `crates/oya-dev-cli/Cargo.toml`); must
    /// declare the `repoctl` binary target.
    pub cli_manifest: &'a str, // data_class: INTERNAL_ONLY
    /// `crates/oya-dev-cli/src/commands/repoctl.rs` source; must contain
    /// the native verify dispatch token AND must spell the canonical
    /// command name in its dispatch table.
    pub repoctl_source: &'a str, // data_class: INTERNAL_ONLY
    /// Local git hook script contents (`scripts/hooks/pre-push-repoctl.sh`
    /// during the transition; the in-binary `hook install` surface after
    /// IP-E). Must invoke the full `CANONICAL_PRE_PUSH_COMMAND` (not the
    /// `--verify-contract` short-circuit, which is the contract-fitness
    /// check, not the local-verify entry).
    pub hook_script: &'a str, // data_class: INTERNAL_ONLY
}

/// Successful contract report. Each boolean records the positive scope
/// that the evidence satisfied; consumers may surface this as JSON
/// evidence for fitness-lane audits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractReport {
    pub canonical_command: &'static str, // data_class: INTERNAL_ONLY
    pub native_verify_dispatch_token: &'static str, // data_class: INTERNAL_ONLY
    pub done_definition_mentions_command: bool, // data_class: INTERNAL_ONLY
    pub repoctl_binary_declared: bool,   // data_class: INTERNAL_ONLY
    pub repoctl_source_dispatches_native_verify: bool, // data_class: INTERNAL_ONLY
    pub hook_wires_full_command: bool,   // data_class: INTERNAL_ONLY
}

/// Errors returned when evidence does not satisfy the contract.
///
/// Naming justification: variants describe the missing positive scope
/// in canonical terms (no "exception" / "exempt" phrasing per
/// `feedback_no_exceptions_canonical`). Each variant is loud and
/// CI-detectable per `feedback_no_silent_regression`. `Display` is
/// implemented manually below to keep this kernel free of external
/// crate dependencies (matches the rest of the `crates/oya-check-*/`
/// kernel family).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrePushContractError {
    MissingDoneDefinitionCommand,
    MissingRepoctlBinary,
    MissingNativeVerifyDispatchInRepoctlSource,
    MissingCanonicalCommandNameInRepoctlSource,
    MissingHookCommand,
    HookUsesContractCheckInsteadOfFullCommand,
}

/// Validate the CLI-surface contract. Pure function: parses the supplied
/// evidence strings and returns either a positive report or a typed
/// error. Performs no I/O; the runner is responsible for reading source
/// files into `PrePushContractEvidence`.
pub fn validate_pre_push_contract(
    evidence: PrePushContractEvidence<'_>,
) -> Result<PrePushContractReport, PrePushContractError> {
    if !evidence
        .done_definition_doc
        .contains(CANONICAL_PRE_PUSH_COMMAND)
    {
        return Err(PrePushContractError::MissingDoneDefinitionCommand);
    }
    if !evidence.cli_manifest.contains(REPOCTL_BIN_NAME_DECLARATION) {
        return Err(PrePushContractError::MissingRepoctlBinary);
    }
    if !evidence
        .repoctl_source
        .contains(REPOCTL_NATIVE_VERIFY_DISPATCH_TOKEN)
    {
        return Err(PrePushContractError::MissingNativeVerifyDispatchInRepoctlSource);
    }
    if !repoctl_source_names_canonical_command(evidence.repoctl_source) {
        return Err(PrePushContractError::MissingCanonicalCommandNameInRepoctlSource);
    }
    if !hook_invokes_full_pre_push(evidence.hook_script) {
        return Err(PrePushContractError::MissingHookCommand);
    }
    if evidence.hook_script.contains("--verify-contract") {
        return Err(PrePushContractError::HookUsesContractCheckInsteadOfFullCommand);
    }

    Ok(PrePushContractReport {
        canonical_command: CANONICAL_PRE_PUSH_COMMAND,
        native_verify_dispatch_token: REPOCTL_NATIVE_VERIFY_DISPATCH_TOKEN,
        done_definition_mentions_command: true,
        repoctl_binary_declared: true,
        repoctl_source_dispatches_native_verify: true,
        hook_wires_full_command: true,
    })
}

/// True iff the repoctl source spells the canonical `repoctl pre-push`
/// command name in its dispatch surface (typically as the `"pre-push"`
/// subcommand match arm). The two literal tokens that satisfy this scope
/// are `"pre-push"` (the subcommand string match) and `repoctl pre-push`
/// (any doc-comment or println that names the canonical command).
///
/// Naming justification: function name is snake_case; the predicate is
/// stated positively (no "missing" / "exception" phrasing).
fn repoctl_source_names_canonical_command(repoctl_source: &str) -> bool {
    repoctl_source.contains("\"pre-push\"") || repoctl_source.contains(CANONICAL_PRE_PUSH_COMMAND)
}

/// True iff the hook script has a non-comment, non-empty line that
/// invokes the full canonical `repoctl pre-push` command (i.e. not the
/// `--verify-contract` short-circuit).
fn hook_invokes_full_pre_push(hook_script: &str) -> bool {
    hook_script.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.contains("--verify-contract")
            && (trimmed.contains(CANONICAL_PRE_PUSH_COMMAND)
                || trimmed.contains("--bin repoctl -- pre-push"))
    })
}

impl fmt::Display for PrePushContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDoneDefinitionCommand => write!(
                formatter,
                "Done-Definition does not require `{CANONICAL_PRE_PUSH_COMMAND}`"
            ),
            Self::MissingRepoctlBinary => write!(
                formatter,
                "CLI manifest does not declare the repoctl binary"
            ),
            Self::MissingNativeVerifyDispatchInRepoctlSource => write!(
                formatter,
                "repoctl source does not dispatch into the native verify surface \
                 (missing token `{REPOCTL_NATIVE_VERIFY_DISPATCH_TOKEN}`)"
            ),
            Self::MissingCanonicalCommandNameInRepoctlSource => write!(
                formatter,
                "repoctl source does not name the canonical command \
                 `{CANONICAL_PRE_PUSH_COMMAND}` in its dispatch surface"
            ),
            Self::MissingHookCommand => write!(
                formatter,
                "pre-push hook script does not invoke `{CANONICAL_PRE_PUSH_COMMAND}`"
            ),
            Self::HookUsesContractCheckInsteadOfFullCommand => write!(
                formatter,
                "pre-push hook script must run the full command, not only --verify-contract"
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

        assert_eq!(report.canonical_command, "repoctl pre-push");
        assert_eq!(report.native_verify_dispatch_token, "commands::verify::run");
        assert!(report.done_definition_mentions_command);
        assert!(report.repoctl_binary_declared);
        assert!(report.repoctl_source_dispatches_native_verify);
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
    fn rejects_missing_repoctl_binary_declaration() {
        let mut evidence = valid_evidence();
        evidence.cli_manifest = "[[bin]]\nname = \"oya\"\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingRepoctlBinary)
        );
    }

    #[test]
    fn rejects_repoctl_source_without_native_verify_dispatch() {
        let mut evidence = valid_evidence();
        evidence.repoctl_source = "match args.next().as_deref() {\n    \
            Some(\"pre-push\") => some_subprocess_call(),\n}\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingNativeVerifyDispatchInRepoctlSource)
        );
    }

    #[test]
    fn rejects_repoctl_source_without_canonical_command_name() {
        let mut evidence = valid_evidence();
        // dispatches into native verify but does not name the canonical
        // `pre-push` subcommand — proves the predicate is independent.
        evidence.repoctl_source = "commands::verify::run(args, &usage());\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingCanonicalCommandNameInRepoctlSource)
        );
    }

    #[test]
    fn rejects_hook_that_only_runs_contract_check() {
        let mut evidence = valid_evidence();
        evidence.hook_script =
            "cargo run -p oya-dev-cli --bin repoctl -- pre-push --verify-contract\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingHookCommand)
        );
    }

    #[test]
    fn rejects_hook_that_invokes_only_verify_contract_alongside_other_lines() {
        let mut evidence = valid_evidence();
        // A hook with the `--verify-contract` flag anywhere (even on a
        // line that also names the canonical command) is rejected per
        // the CLI-surface invariant: the local-developer hook runs the
        // full verify pass, not the contract short-circuit.
        evidence.hook_script = "cargo run -p oya-dev-cli --bin repoctl -- \
             pre-push --verify-contract\ncargo run -p oya-dev-cli --bin repoctl -- pre-push\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::HookUsesContractCheckInsteadOfFullCommand)
        );
    }

    fn valid_evidence() -> PrePushContractEvidence<'static> {
        PrePushContractEvidence {
            done_definition_doc: "- [ ] D12 `repoctl pre-push` passes.",
            cli_manifest: "[[bin]]\nname = \"repoctl\"\npath = \"src/main.rs\"\n",
            repoctl_source:
                "match args.next().as_deref() {\n    \
                 Some(\"pre-push\") => commands::verify::run(args.collect(), &usage()),\n}\n",
            hook_script: "cargo run -p oya-dev-cli --bin repoctl -- pre-push \"$@\"\n",
        }
    }
}
