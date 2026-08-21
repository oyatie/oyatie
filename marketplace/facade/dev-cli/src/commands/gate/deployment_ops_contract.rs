//! `oya gate validate deployment-ops-contract` — deployment entrypoint and
//! shell-consolidation guard. Encodes that deployment is OpenTofu-owned,
//! operators invoke tofu against `infra/cloudflare` (no root Makefile),
//! day-2 work routes through ops, and manual SSH troubleshooting is not a
//! valid path.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeploymentOpsContractValidateArgs {
    repo_root: PathBuf,
    contract_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DeploymentOpsContractReport {
    pub(crate) opentofu_roots_checked: usize,
    pub(crate) shell_scripts_tracked: usize,
}

pub(crate) fn run(args: Vec<String>) -> ExitCode {
    match parse_deployment_ops_contract_validate_args(args) {
        Ok(parsed) => match validate_deployment_ops_contract(&parsed) {
            Ok(report) => {
                println!(
                    "deployment-ops-contract validation passed: {} OpenTofu roots, {} shell scripts tracked for Rust migration",
                    report.opentofu_roots_checked, report.shell_scripts_tracked
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
            other => {
                return Err(format!(
                    "deployment-ops-contract: unknown flag {other:?}; allowed: --repo-root, --contract"
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

    let mut errors = Vec::new();

    if repo_root.join("Makefile").is_file() {
        errors.push(
            "root Makefile must not exist; Cloudflare edge commands are tofu -chdir=infra/cloudflare (iac/README.md)"
                .to_string(),
        );
    }
    if contract.pointer("/operator_entrypoints/makefile").is_some() {
        errors.push(
            "operator_entrypoints.makefile is retired; document tofu commands under operator_entrypoints.cloudflare_edge"
                .to_string(),
        );
    }

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

    let fmt_check = string_at(
        &contract,
        &[
            "operator_entrypoints",
            "cloudflare_edge",
            "commands",
            "fmt_check",
        ],
    );
    if fmt_check != Some("tofu -chdir=infra/cloudflare fmt -check -recursive") {
        errors.push(
            "operator_entrypoints.cloudflare_edge.commands.fmt_check must be tofu -chdir=infra/cloudflare fmt -check -recursive"
                .to_string(),
        );
    }

    let allowed = string_array_at(
        &contract,
        &["operator_entrypoints", "allowed_human_commands_after_clone"],
    );
    if allowed.is_empty() {
        errors.push(
            "operator_entrypoints.allowed_human_commands_after_clone must be non-empty".to_string(),
        );
    }
    for command in &allowed {
        if command_is_make(command) {
            errors.push(format!(
                "allowed_human_commands_after_clone still lists Make: {command}"
            ));
        }
    }

    if string_at(&contract, &["bootstrap_contract", "entrypoint"]).is_some_and(command_is_make) {
        errors.push("bootstrap_contract.entrypoint must not be a Make target".to_string());
    }
    if string_at(&contract, &["install_contract", "entrypoint"]).is_some_and(command_is_make) {
        errors.push("install_contract.entrypoint must not be a Make target".to_string());
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

fn command_is_make(command: &str) -> bool {
    let trimmed = command.trim();
    trimmed == "make" || trimmed.starts_with("make ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_commands_are_rejected() {
        assert!(command_is_make("make bootstrap"));
        assert!(command_is_make("make"));
        assert!(!command_is_make(
            "tofu -chdir=infra/cloudflare fmt -check -recursive"
        ));
    }

    #[test]
    fn makefile_flag_is_rejected() {
        let parsed = parse_deployment_ops_contract_validate_args(vec!["--makefile".into()]);
        assert!(
            matches!(parsed, Err(ref message) if message.contains("unknown flag")),
            "{parsed:?}"
        );
    }
}
