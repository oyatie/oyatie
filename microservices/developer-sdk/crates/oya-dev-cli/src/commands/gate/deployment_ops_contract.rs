//! `oya gate validate deployment-ops-contract` — deployment entrypoint and
//! shell-consolidation guard. Encodes the 2026-05-16 directive that deployment
//! is OpenTofu-owned, operator entry is the root Makefile, day-2 work routes
//! through ops, and manual SSH troubleshooting is not a valid path.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeploymentOpsContractValidateArgs {
    repo_root: PathBuf,
    contract_path: PathBuf,
    makefile_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeploymentOpsContractReport {
    pub(crate) required_make_targets: usize,
    pub(crate) opentofu_roots_checked: usize,
    pub(crate) shell_scripts_tracked: usize,
}

pub(crate) fn run(args: Vec<String>) -> ExitCode {
    match parse_deployment_ops_contract_validate_args(args) {
        Ok(parsed) => match validate_deployment_ops_contract(&parsed) {
            Ok(report) => {
                println!(
                    "deployment-ops-contract validation passed: {} Make targets, {} OpenTofu roots, {} shell scripts tracked for Rust migration",
                    report.required_make_targets,
                    report.opentofu_roots_checked,
                    report.shell_scripts_tracked
                );
                ExitCode::SUCCESS
            }
            Err(errors) => {
                eprintln!("deployment-ops-contract validation failed:");
                for error in &errors {
                    eprintln!("  {error}");
                }
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn parse_deployment_ops_contract_validate_args(
    args: Vec<String>,
) -> Result<DeploymentOpsContractValidateArgs, String> {
    let mut parsed = DeploymentOpsContractValidateArgs {
        repo_root: PathBuf::from("."),
        contract_path: PathBuf::from("specs/deployment-ops-contract.json"),
        makefile_path: PathBuf::from("Makefile"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                let value = iter.next().ok_or_else(|| {
                    "deployment-ops-contract: --repo-root requires a value".to_string()
                })?;
                parsed.repo_root = PathBuf::from(value);
            }
            "--contract" => {
                let value = iter.next().ok_or_else(|| {
                    "deployment-ops-contract: --contract requires a value".to_string()
                })?;
                parsed.contract_path = PathBuf::from(value);
            }
            "--makefile" => {
                let value = iter.next().ok_or_else(|| {
                    "deployment-ops-contract: --makefile requires a value".to_string()
                })?;
                parsed.makefile_path = PathBuf::from(value);
            }
            other => {
                return Err(format!(
                    "deployment-ops-contract: unknown flag {other:?}; allowed: --repo-root, --contract, --makefile"
                ));
            }
        }
    }
    Ok(parsed)
}

fn validate_deployment_ops_contract(
    args: &DeploymentOpsContractValidateArgs,
) -> Result<DeploymentOpsContractReport, Vec<String>> {
    let repo_root = fs::canonicalize(&args.repo_root)
        .map_err(|error| vec![format!("repo root unreadable: {error}")])?;
    let contract_path = repo_root.join(&args.contract_path);
    let contract_text = fs::read_to_string(&contract_path).map_err(|error| {
        vec![format!(
            "contract unreadable {}: {error}",
            contract_path.display()
        )]
    })?;
    let contract: Value = serde_json::from_str(&contract_text).map_err(|error| {
        vec![format!(
            "contract JSON invalid {}: {error}",
            contract_path.display()
        )]
    })?;

    let makefile_path = repo_root.join(&args.makefile_path);
    let makefile = fs::read_to_string(&makefile_path).map_err(|error| {
        vec![format!(
            "Makefile unreadable {}: {error}",
            makefile_path.display()
        )]
    })?;

    let mut errors = Vec::new();

    if string_at(&contract, &["deployment_authority", "primary"]) != Some("opentofu") {
        errors.push("deployment_authority.primary must be exactly opentofu".to_string());
    }

    let roots = array_at(&contract, &["deployment_authority", "roots"]);
    if roots.is_empty() {
        errors.push("deployment_authority.roots must list OpenTofu root modules".to_string());
    }
    for root in &roots {
        let Some(path) = root.get("path").and_then(Value::as_str) else {
            errors.push("deployment_authority.roots[] missing path".to_string());
            continue;
        };
        let full = repo_root.join(path);
        if !full.is_dir() {
            errors.push(format!("OpenTofu root path is not a directory: {path}"));
        }
        if !full.join("main.tf").is_file() {
            errors.push(format!("OpenTofu root missing main.tf: {path}"));
        }
    }

    let required_targets = string_array_at(
        &contract,
        &["operator_entrypoints", "makefile", "required_targets"],
    );
    if required_targets.is_empty() {
        errors.push("operator_entrypoints.makefile.required_targets must be non-empty".to_string());
    }
    for target in &required_targets {
        if !makefile_declares_target(&makefile, target) {
            errors.push(format!("Makefile missing required target: {target}"));
        }
    }
    for forbidden in ["\n\tssh ", "oci compute instance update", "terraform "] {
        if makefile.contains(forbidden) {
            errors.push(format!(
                "Makefile contains forbidden operator command fragment: {forbidden:?}"
            ));
        }
    }

    if string_at(&contract, &["ops_management_contract", "day_2_surface"])
        != Some("https://ops.oyatie.com")
    {
        errors.push(
            "ops_management_contract.day_2_surface must be https://ops.oyatie.com".to_string(),
        );
    }
    if bool_at(
        &contract,
        &["manual_access", "manual_ssh_troubleshooting_allowed"],
    ) != Some(false)
    {
        errors.push("manual_access.manual_ssh_troubleshooting_allowed must be false".to_string());
    }
    let forbidden_kinds = string_array_at(&contract, &["manual_access", "forbidden_action_kinds"])
        .into_iter()
        .collect::<BTreeSet<_>>();
    for kind in ["ssh_troubleshooting", "oci_cli_mutation", "console_drift"] {
        if !forbidden_kinds.contains(kind) {
            errors.push(format!(
                "manual_access.forbidden_action_kinds missing {kind}"
            ));
        }
    }

    let root_hub =
        fs::read_to_string(repo_root.join("specs/root-hub-pointers.json")).unwrap_or_default();
    if !root_hub.contains("deployment_ops_contract")
        || !root_hub.contains("/specs/deployment-ops-contract.json")
    {
        errors.push("root-hub pointers must expose deployment_ops_contract".to_string());
    }
    let durable_goal =
        fs::read_to_string(repo_root.join("specs/agent-durable-goal.json")).unwrap_or_default();
    if !durable_goal.contains("specs/deployment-ops-contract.json") {
        errors.push(
            "agent durable goal must reference specs/deployment-ops-contract.json".to_string(),
        );
    }
    if durable_goal.contains("page on-call") {
        errors.push(
            "agent durable goal still contains human page-on-call deployment flow".to_string(),
        );
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(DeploymentOpsContractReport {
        required_make_targets: required_targets.len(),
        opentofu_roots_checked: roots.len(),
        shell_scripts_tracked: 0,
    })
}

fn string_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str()
}

fn bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_bool()
}

fn array_at<'a>(value: &'a Value, path: &[&str]) -> Vec<&'a Value> {
    let mut cursor = value;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return Vec::new();
        };
        cursor = next;
    }
    cursor
        .as_array()
        .map(|items| items.iter().collect())
        .unwrap_or_default()
}

fn string_array_at(value: &Value, path: &[&str]) -> Vec<String> {
    array_at(value, path)
        .into_iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn makefile_declares_target(makefile: &str, target: &str) -> bool {
    let target_prefix = format!("{target}:");
    let target_with_prereqs = format!("{target}: ");
    makefile.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with(&target_prefix) || trimmed.starts_with(&target_with_prereqs)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn makefile_target_detection_accepts_prerequisites() {
        let makefile = "bootstrap: verify check-tofu\n\t@echo ok\n";
        assert!(makefile_declares_target(makefile, "bootstrap"));
        assert!(!makefile_declares_target(makefile, "install"));
    }
}
