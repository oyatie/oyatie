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

    let rust_surfaces =
        string_array_at(&contract, &["shell_consolidation", "target_rust_surfaces"])
            .into_iter()
            .collect::<BTreeSet<_>>();
    for surface in [
        "oya gate validate deployment-ops-contract",
        "oya onprem plan",
        "oya onprem install",
        "oya onprem uninstall",
        "oya onprem doctor",
        "oya ops oci-a1-capacity-retry",
        "oya ops oci-readiness-probe",
        "oya ops onprem-bring-up",
    ] {
        if !rust_surfaces.contains(surface) {
            errors.push(format!(
                "shell_consolidation.target_rust_surfaces missing {surface}"
            ));
        }
    }
    for surface in &rust_surfaces {
        if surface.starts_with("future ") {
            errors.push(format!(
                "shell_consolidation.target_rust_surfaces must name active commands, not future placeholders: {surface}"
            ));
        }
    }

    let discovered_scripts = discover_shell_scripts(&repo_root.join("infra/onprem"));
    let tracked_scripts = string_array_at(
        &contract,
        &[
            "shell_consolidation",
            "legacy_shell_inventory",
            "migrate_to_rust",
        ],
    )
    .into_iter()
    .collect::<BTreeSet<_>>();
    for script in &discovered_scripts {
        if !tracked_scripts.contains(script) {
            errors.push(format!(
                "infra/onprem shell script not tracked for Rust migration: {script}"
            ));
        }
    }
    for script in &tracked_scripts {
        if !discovered_scripts.contains(script) {
            errors.push(format!("contract tracks missing shell script: {script}"));
        }
    }
    for (shim, action) in [
        ("infra/onprem/setup.sh", "install"),
        ("infra/onprem/diagnose.sh", "doctor"),
        ("infra/onprem/uninstall-all.sh", "uninstall"),
    ] {
        let path = repo_root.join(shim);
        let text = fs::read_to_string(&path).unwrap_or_default();
        let required = format!("onprem {action}");
        if !text.contains(&required) {
            errors.push(format!(
                "top-level onprem shim {shim} must dispatch to `oya {required}`"
            ));
        }
        for forbidden in [
            "ACTION REQUIRED",
            "operator unseal",
            "setup-cloudflared.sh",
            "tofu -chdir",
            "bash \"$HERE",
            "sudo bash",
            "ssh ",
        ] {
            if text.contains(forbidden) {
                errors.push(format!(
                    "top-level onprem shim {shim} contains forbidden manual fragment {forbidden:?}"
                ));
            }
        }
    }
    for path in collect_files(&repo_root.join("infra/onprem"), &["sh"]) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for forbidden in [
            "ssh session",
            "ssh host",
            "scp ",
            "TO ACTIVATE",
            "TO FULLY DISABLE PASSWORD AUTH",
            "manual SSH troubleshooting",
        ] {
            if text.contains(forbidden) {
                errors.push(format!(
                    "legacy onprem shell contains forbidden manual SSH/troubleshooting guidance {forbidden:?}: {}",
                    slash_path(path.strip_prefix(&repo_root).unwrap_or(&path))
                ));
            }
        }
    }

    for (shim, command) in [
        (
            "scripts/oci-a1-capacity-retry.sh",
            "ops oci-a1-capacity-retry",
        ),
        ("scripts/oci-readiness-probe.sh", "ops oci-readiness-probe"),
        ("scripts/onprem-bring-up.sh", "ops onprem-bring-up"),
    ] {
        let path = repo_root.join(shim);
        let text = fs::read_to_string(&path).unwrap_or_default();
        if !text.contains(command) {
            errors.push(format!(
                "root deployment shim {shim} must dispatch to `oya {command}`"
            ));
        }
        for forbidden in [
            "apt-get",
            "python3 -m venv",
            "setup-oyatie-service.sh",
            "tofu apply",
            "oci iam",
            "sudo bash",
            "ssh ",
        ] {
            if text.contains(forbidden) {
                errors.push(format!(
                    "root deployment shim {shim} contains forbidden hand-rolled fragment {forbidden:?}"
                ));
            }
        }
    }

    if repo_root.join("infra/oci/README-stage0-resize.md").exists() {
        errors.push(
            "manual OCI resize runbook still exists at infra/oci/README-stage0-resize.md"
                .to_string(),
        );
    }

    let active_deploy_files = collect_files(&repo_root.join("infra/oci"), &["tf", "tfvars"]);
    for path in active_deploy_files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for forbidden in [
            "oci compute instance update",
            "out-of-band with `oci`",
            "Subscribers added out-of-band",
            "console-edited",
        ] {
            if text.contains(forbidden) {
                errors.push(format!(
                    "active OpenTofu file contains forbidden manual-drift fragment {forbidden:?}: {}",
                    slash_path(path.strip_prefix(&repo_root).unwrap_or(&path))
                ));
            }
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
        shell_scripts_tracked: tracked_scripts.len(),
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

fn discover_shell_scripts(root: &Path) -> BTreeSet<String> {
    let mut scripts = BTreeSet::new();
    collect_shell_scripts(root, root, &mut scripts);
    scripts
}

fn repo_root_for_onprem(root: &Path) -> &Path {
    root.parent().and_then(Path::parent).unwrap_or(root)
}

fn collect_shell_scripts(root: &Path, current: &Path, scripts: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_shell_scripts(root, &path, scripts);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("sh")
            && let Ok(relative) = path.strip_prefix(repo_root_for_onprem(root))
        {
            scripts.insert(slash_path(relative));
        }
    }
}

fn collect_files(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_inner(root, extensions, &mut files);
    files
}

fn collect_files_inner(current: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_inner(&path, extensions, files);
            continue;
        }
        if let Some(extension) = path.extension().and_then(|extension| extension.to_str())
            && extensions.contains(&extension)
        {
            files.push(path);
        }
    }
}

fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join("/")
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
