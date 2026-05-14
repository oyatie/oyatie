//! Foundry pre-push command contract fitness kernel.

use std::fmt;

pub const CANONICAL_PRE_PUSH_COMMAND: &str = "repoctl pre-push";
pub const PRE_PUSH_CONTRACT_CHECK_COMMAND: &str =
    "cargo run -p oya-dev-cli --bin repoctl -- pre-push --verify-contract";
pub const REPOCTL_BIN_NAME_DECLARATION: &str = "name = \"repoctl\"";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractEvidence<'a> {
    pub done_definition_doc: &'a str, // data_class: INTERNAL_ONLY
    pub check_script: &'a str,        // data_class: INTERNAL_ONLY
    pub cli_manifest: &'a str,        // data_class: INTERNAL_ONLY
    pub hook_script: &'a str,         // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrePushContractReport {
    pub canonical_command: &'static str, // data_class: INTERNAL_ONLY
    pub contract_check_command: &'static str, // data_class: INTERNAL_ONLY
    pub done_definition_mentions_command: bool, // data_class: INTERNAL_ONLY
    pub repoctl_binary_declared: bool,   // data_class: INTERNAL_ONLY
    pub check_script_wires_contract_check: bool, // data_class: INTERNAL_ONLY
    pub hook_wires_full_command: bool,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrePushContractError {
    MissingDoneDefinitionCommand,
    MissingRepoctlBinary,
    MissingContractCheckInCheckScript,
    RecursivePrePushInCheckScript { line: String },
    MissingHookCommand,
    HookUsesContractCheckInsteadOfFullCommand,
}

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
        .check_script
        .contains(PRE_PUSH_CONTRACT_CHECK_COMMAND)
    {
        return Err(PrePushContractError::MissingContractCheckInCheckScript);
    }
    if let Some(line) = recursive_check_script_line(evidence.check_script) {
        return Err(PrePushContractError::RecursivePrePushInCheckScript { line });
    }
    if !hook_invokes_full_pre_push(evidence.hook_script) {
        return Err(PrePushContractError::MissingHookCommand);
    }
    if evidence.hook_script.contains("--verify-contract") {
        return Err(PrePushContractError::HookUsesContractCheckInsteadOfFullCommand);
    }

    Ok(PrePushContractReport {
        canonical_command: CANONICAL_PRE_PUSH_COMMAND,
        contract_check_command: PRE_PUSH_CONTRACT_CHECK_COMMAND,
        done_definition_mentions_command: true,
        repoctl_binary_declared: true,
        check_script_wires_contract_check: true,
        hook_wires_full_command: true,
    })
}

fn recursive_check_script_line(check_script: &str) -> Option<String> {
    check_script.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.contains("--verify-contract") {
            return None;
        }
        if trimmed.contains(CANONICAL_PRE_PUSH_COMMAND)
            || trimmed.contains("--bin repoctl -- pre-push")
        {
            Some(trimmed.to_string())
        } else {
            None
        }
    })
}

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
            Self::MissingContractCheckInCheckScript => write!(
                formatter,
                "check script does not wire `{PRE_PUSH_CONTRACT_CHECK_COMMAND}`"
            ),
            Self::RecursivePrePushInCheckScript { line } => write!(
                formatter,
                "check script would recurse through full pre-push command: {line}"
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
        assert!(report.done_definition_mentions_command);
        assert!(report.repoctl_binary_declared);
        assert!(report.check_script_wires_contract_check);
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
    fn rejects_unwired_contract_check() {
        let mut evidence = valid_evidence();
        evidence.check_script = "cargo test --workspace\n";

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingContractCheckInCheckScript)
        );
    }

    #[test]
    fn rejects_recursive_full_pre_push_from_check_script() {
        let check_script = format!(
            "{PRE_PUSH_CONTRACT_CHECK_COMMAND}\n\
             cargo run -p oya-dev-cli --bin repoctl -- pre-push\n"
        );
        let evidence = PrePushContractEvidence {
            check_script: &check_script,
            ..valid_evidence()
        };

        assert!(matches!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::RecursivePrePushInCheckScript { .. })
        ));
    }

    #[test]
    fn rejects_hook_that_only_runs_contract_check() {
        let mut evidence = valid_evidence();
        evidence.hook_script = PRE_PUSH_CONTRACT_CHECK_COMMAND;

        assert_eq!(
            validate_pre_push_contract(evidence),
            Err(PrePushContractError::MissingHookCommand)
        );
    }

    fn valid_evidence() -> PrePushContractEvidence<'static> {
        PrePushContractEvidence {
            done_definition_doc: "- [ ] D12 `repoctl pre-push` passes.",
            check_script: PRE_PUSH_CONTRACT_CHECK_COMMAND,
            cli_manifest: "[[bin]]\nname = \"repoctl\"\npath = \"src/main.rs\"\n",
            hook_script: "cargo run -p oya-dev-cli --bin repoctl -- pre-push \"$@\"\n",
        }
    }
}
