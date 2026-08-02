//! `oya gate validate pre-push-contract` runner.
//!
//! Reads evidence files (Done-Definition, dev-CLI dispatch source,
//! pre-push hook script) and invokes
//! [`check_pre_push::validate_pre_push_contract`]. The kernel is
//! the canonical authority; this module is the I/O-shaped runner
//! (port-in-kernel, ADR-0056).
//!
//! Lane id: `oya-governance-pre-push`. Asserts that the
//! canonical `oya verify` local-developer command is wired
//! consistently across the three evidence surfaces, with no `.sh`
//! subprocess interposed.
//!
//! Naming justification: module file is snake_case, no redundant
//! suffix; functions follow the existing `parse_<lane>_validate_args`
//! / `validate_<lane>_gate` naming used by every other gate in this
//! crate (see e.g. `active_artifact_contract_gate`,
//! `cedar_fragment_coverage_gate`).

use std::fs;
use std::path::{Path, PathBuf};

use check_pre_push::{
    PrePushContractEvidence, PrePushContractReport, validate_pre_push_contract,
};

const USAGE: &str = "oya gate validate pre-push-contract \
                     [--done-definition <path>] \
                     [--cli-dispatch-source <path>] \
                     [--hook-script <path>]";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PrePushContractValidateArgs {
    pub done_definition_doc_path: PathBuf,
    pub cli_dispatch_source_path: PathBuf,
    pub hook_script_path: PathBuf,
}

impl Default for PrePushContractValidateArgs {
    fn default() -> Self {
        Self {
            done_definition_doc_path: PathBuf::from("docs/checklists/done-definition-checklist.md"),
            cli_dispatch_source_path: PathBuf::from("crates/oya-dev-cli/src/lib.rs"),
            hook_script_path: PathBuf::from("scripts/hooks/pre-push.sh"),
        }
    }
}

pub(crate) fn parse_pre_push_contract_validate_args(
    args: Vec<String>,
) -> Result<PrePushContractValidateArgs, String> {
    let mut parsed = PrePushContractValidateArgs::default();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--done-definition" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.done_definition_doc_path = PathBuf::from(value);
            }
            "--cli-dispatch-source" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.cli_dispatch_source_path = PathBuf::from(value);
            }
            "--hook-script" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.hook_script_path = PathBuf::from(value);
            }
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_pre_push_contract_gate(
    args: PrePushContractValidateArgs,
) -> Result<PrePushContractReport, String> {
    let done_definition_doc =
        read_evidence_file("done-definition", &args.done_definition_doc_path)?;
    let cli_dispatch_source =
        read_evidence_file("cli-dispatch-source", &args.cli_dispatch_source_path)?;
    let hook_script = read_evidence_file("hook-script", &args.hook_script_path)?;

    validate_pre_push_contract(PrePushContractEvidence {
        done_definition_doc: &done_definition_doc,
        cli_dispatch_source: &cli_dispatch_source,
        hook_script: &hook_script,
    })
    .map_err(|error| error.to_string())
}

fn read_evidence_file(label: &str, path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("could not read {label} at {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uses_canonical_defaults() {
        let args = parse_pre_push_contract_validate_args(Vec::new()).expect("no flags is valid");
        assert_eq!(
            args.done_definition_doc_path,
            PathBuf::from("docs/checklists/done-definition-checklist.md")
        );
        assert_eq!(
            args.cli_dispatch_source_path,
            PathBuf::from("crates/oya-dev-cli/src/lib.rs")
        );
        assert_eq!(
            args.hook_script_path,
            PathBuf::from("scripts/hooks/pre-push.sh")
        );
    }

    #[test]
    fn parse_accepts_explicit_paths() {
        let args = parse_pre_push_contract_validate_args(vec![
            "--done-definition".to_string(),
            "fixtures/done.md".to_string(),
            "--cli-dispatch-source".to_string(),
            "fixtures/lib.rs".to_string(),
            "--hook-script".to_string(),
            "fixtures/hook.sh".to_string(),
        ])
        .expect("explicit paths parse");
        assert_eq!(
            args.done_definition_doc_path,
            PathBuf::from("fixtures/done.md")
        );
        assert_eq!(
            args.cli_dispatch_source_path,
            PathBuf::from("fixtures/lib.rs")
        );
        assert_eq!(args.hook_script_path, PathBuf::from("fixtures/hook.sh"));
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let result = parse_pre_push_contract_validate_args(vec!["--unknown".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("pre-push-contract"));
    }

    #[test]
    fn parse_rejects_dangling_value_flag() {
        let result = parse_pre_push_contract_validate_args(vec!["--done-definition".to_string()]);
        assert!(result.is_err());
    }
}
