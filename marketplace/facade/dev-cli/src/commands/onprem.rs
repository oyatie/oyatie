//! `oya onprem` — Rust-owned on-prem orchestration surface.
//!
//! This replaces the hand-written top-level `infra/onprem/*.sh`
//! orchestration with a repo-native command that is safe to call from
//! ops controllers or compatibility shims. Component-level
//! shell remains legacy inventory until the migration waves port it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::json;

use crate::command_output::OutputFormat;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OnpremAction {
    Plan,
    Install,
    Uninstall,
    Doctor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OnpremArgs {
    action: OnpremAction,
    repo_root: PathBuf,
    output_format: OutputFormat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Component {
    id: &'static str,
    phase: &'static str,
    install_script: Option<&'static str>,
    uninstall_script: Option<&'static str>,
    managed_by: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DoctorReport {
    checked_components: usize,
    checked_shims: usize,
    warnings: Vec<String>,
    errors: Vec<String>,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_onprem_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    match parsed.action {
        OnpremAction::Plan => {
            render_plan(parsed.output_format);
            ExitCode::SUCCESS
        }
        OnpremAction::Install => {
            render_install(parsed.output_format);
            ExitCode::SUCCESS
        }
        OnpremAction::Uninstall => {
            render_uninstall(parsed.output_format);
            ExitCode::SUCCESS
        }
        OnpremAction::Doctor => match run_doctor(&parsed.repo_root) {
            Ok(report) => {
                render_doctor(parsed.output_format, &report);
                if report.errors.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(message) => {
                eprintln!("onprem doctor failed: {message}");
                ExitCode::FAILURE
            }
        },
    }
}

fn parse_onprem_args(args: Vec<String>, usage: &str) -> Result<OnpremArgs, String> {
    let mut iter = args.into_iter();
    let action = match iter.next().as_deref() {
        Some("plan") => OnpremAction::Plan,
        Some("install") => OnpremAction::Install,
        Some("uninstall") => OnpremAction::Uninstall,
        Some("doctor") => OnpremAction::Doctor,
        _ => return Err(usage.to_string()),
    };
    let mut parsed = OnpremArgs {
        action,
        repo_root: PathBuf::from("."),
        output_format: OutputFormat::Text,
    };
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "oya onprem: --repo-root requires a value".to_string())?;
                parsed.repo_root = PathBuf::from(value);
            }
            "--format" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "oya onprem: --format requires a value".to_string())?;
                parsed.output_format = OutputFormat::parse(&value)
                    .ok_or_else(|| "oya onprem: --format must be text or json".to_string())?;
            }
            other => {
                return Err(format!(
                    "oya onprem: unknown flag {other:?}; allowed: --repo-root, --format"
                ));
            }
        }
    }
    Ok(parsed)
}

fn components() -> Vec<Component> {
    vec![
        Component {
            id: "cleanup",
            phase: "00",
            install_script: Some("infra/onprem/cleanup/install.sh"),
            uninstall_script: Some("infra/onprem/cleanup/uninstall.sh"),
            managed_by: "rust-wave-w1",
        },
        Component {
            id: "security",
            phase: "00b",
            install_script: Some("infra/onprem/security/install.sh"),
            uninstall_script: Some("infra/onprem/security/uninstall.sh"),
            managed_by: "rust-wave-w1",
        },
        Component {
            id: "hardening",
            phase: "01",
            install_script: Some("infra/onprem/hardening/install.sh"),
            uninstall_script: Some("infra/onprem/hardening/uninstall.sh"),
            managed_by: "rust-wave-w3",
        },
        Component {
            id: "sanoid",
            phase: "02",
            install_script: Some("infra/onprem/sanoid/install.sh"),
            uninstall_script: Some("infra/onprem/sanoid/uninstall.sh"),
            managed_by: "rust-wave-w1",
        },
        Component {
            id: "reboots",
            phase: "03",
            install_script: Some("infra/onprem/reboots/install.sh"),
            uninstall_script: Some("infra/onprem/reboots/uninstall.sh"),
            managed_by: "rust-wave-w1",
        },
        Component {
            id: "foundry",
            phase: "04",
            install_script: Some("infra/onprem/foundry/install.sh"),
            uninstall_script: Some("infra/onprem/foundry/uninstall.sh"),
            managed_by: "rust-wave-w3",
        },
        Component {
            id: "openbao",
            phase: "05",
            install_script: Some("infra/onprem/openbao/install.sh"),
            uninstall_script: Some("infra/onprem/openbao/uninstall.sh"),
            managed_by: "rust-wave-w3",
        },
        Component {
            id: "podman",
            phase: "06",
            install_script: Some("infra/onprem/podman/install.sh"),
            uninstall_script: Some("infra/onprem/podman/uninstall.sh"),
            managed_by: "rust-wave-w2",
        },
        Component {
            id: "containerd",
            phase: "07",
            install_script: Some("infra/onprem/containerd/install.sh"),
            uninstall_script: Some("infra/onprem/containerd/uninstall.sh"),
            managed_by: "rust-wave-w2",
        },
        Component {
            id: "kubeadm",
            phase: "08",
            install_script: Some("infra/onprem/kubeadm/install.sh"),
            uninstall_script: Some("infra/onprem/kubeadm/uninstall.sh"),
            managed_by: "rust-wave-w2",
        },
        Component {
            id: "istio",
            phase: "09",
            install_script: Some("infra/onprem/istio/install.sh"),
            uninstall_script: Some("infra/onprem/istio/uninstall.sh"),
            managed_by: "rust-wave-w3",
        },
        Component {
            id: "cloudflared",
            phase: "10",
            install_script: Some("infra/onprem/cloudflared/setup-cloudflared.sh"),
            uninstall_script: Some("infra/onprem/cloudflared/uninstall.sh"),
            managed_by: "opentofu-cloudflare-plus-rust-wave-w3",
        },
        Component {
            id: "diagnostics",
            phase: "11",
            install_script: Some("infra/onprem/diagnose.sh"),
            uninstall_script: None,
            managed_by: "rust-now",
        },
    ]
}

fn top_level_shims() -> [(&'static str, &'static str); 3] {
    [
        ("infra/onprem/setup.sh", "install"),
        ("infra/onprem/diagnose.sh", "doctor"),
        ("infra/onprem/uninstall-all.sh", "uninstall"),
    ]
}

fn render_plan(output_format: OutputFormat) {
    let components = components();
    match output_format {
        OutputFormat::Text => {
            println!("Oyatie on-prem plan: Rust orchestrator + OpenTofu/ops authority");
            println!("manual_ssh_troubleshooting_allowed=false");
            println!("day_2_surface=https://ops.oyatie.com");
            println!();
            println!("Install order:");
            for component in &components {
                println!(
                    "  {} {} managed_by={} legacy_script={}",
                    component.phase,
                    component.id,
                    component.managed_by,
                    component.install_script.unwrap_or("-")
                );
            }
            println!();
            println!("Top-level shell orchestration is replaced by `oya onprem` shims.");
            println!("Component scripts are legacy inventory for the Rust migration waves.");
        }
        OutputFormat::Json => {
            let install_order = components
                .iter()
                .map(|component| {
                    json!({
                        "id": component.id,
                        "phase": component.phase,
                        "managed_by": component.managed_by,
                        "legacy_install_script": component.install_script,
                        "legacy_uninstall_script": component.uninstall_script
                    })
                })
                .collect::<Vec<_>>();
            println!(
                "{}",
                json!({
                    "command": "oya onprem plan",
                    "manual_ssh_troubleshooting_allowed": false,
                    "day_2_surface": "https://ops.oyatie.com",
                    "install_order": install_order,
                    "top_level_orchestration": "rust",
                    "component_shell_status": "legacy_inventory_for_migration_waves"
                })
            );
        }
    }
}

fn render_install(output_format: OutputFormat) {
    match output_format {
        OutputFormat::Text => {
            println!("onprem install request accepted by Rust orchestrator");
            println!("normal deployment authority: tofu -chdir=infra/cloudflare apply");
            println!("day-2 host/service changes: https://ops.oyatie.com");
            println!("legacy component shell is not invoked directly by this command");
            println!("run `oya onprem plan --format text` for the component migration order");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                json!({
                    "command": "oya onprem install",
                    "status": "accepted",
                    "normal_deployment_authority": "tofu -chdir=infra/cloudflare apply",
                    "day_2_surface": "https://ops.oyatie.com",
                    "legacy_component_shell_invoked": false
                })
            );
        }
    }
}

fn render_uninstall(output_format: OutputFormat) {
    let mut order = components();
    order.reverse();
    match output_format {
        OutputFormat::Text => {
            println!("onprem uninstall request accepted by Rust orchestrator");
            println!("destructive removal is not executed from compatibility shell");
            println!("route teardown through ops/OpenTofu change control");
            println!("Planned reverse order:");
            for component in order {
                println!("  {} {}", component.phase, component.id);
            }
        }
        OutputFormat::Json => {
            let uninstall_order = order
                .iter()
                .map(|component| {
                    json!({
                        "id": component.id,
                        "phase": component.phase,
                        "legacy_uninstall_script": component.uninstall_script
                    })
                })
                .collect::<Vec<_>>();
            println!(
                "{}",
                json!({
                    "command": "oya onprem uninstall",
                    "status": "accepted",
                    "destructive_execution": false,
                    "teardown_authority": "ops/OpenTofu change control",
                    "uninstall_order": uninstall_order
                })
            );
        }
    }
}

fn run_doctor(repo_root: &Path) -> Result<DoctorReport, String> {
    let repo_root =
        fs::canonicalize(repo_root).map_err(|error| format!("repo root unreadable: {error}"))?;
    let mut report = DoctorReport {
        checked_components: 0,
        checked_shims: 0,
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    for component in components() {
        report.checked_components += 1;
        for script in [component.install_script, component.uninstall_script]
            .into_iter()
            .flatten()
        {
            if !repo_root.join(script).is_file() {
                report
                    .warnings
                    .push(format!("legacy component script absent: {script}"));
            }
        }
    }

    for (shim, action) in top_level_shims() {
        report.checked_shims += 1;
        let path = repo_root.join(shim);
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("top-level shim unreadable {shim}: {error}"))?;
        let required = format!("onprem {action}");
        if !text.contains(&required) {
            report.errors.push(format!(
                "top-level shim {shim} must dispatch to `oya {required}`"
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
                report.errors.push(format!(
                    "top-level shim {shim} contains forbidden fragment {forbidden:?}"
                ));
            }
        }
    }

    Ok(report)
}

fn render_doctor(output_format: OutputFormat, report: &DoctorReport) {
    match output_format {
        OutputFormat::Text => {
            if report.errors.is_empty() {
                println!(
                    "onprem doctor passed: {} components checked, {} Rust shims checked",
                    report.checked_components, report.checked_shims
                );
            } else {
                println!(
                    "onprem doctor failed: {} components checked, {} Rust shims checked",
                    report.checked_components, report.checked_shims
                );
            }
            for warning in &report.warnings {
                println!("warning: {warning}");
            }
            for error in &report.errors {
                println!("error: {error}");
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                json!({
                    "command": "oya onprem doctor",
                    "status": if report.errors.is_empty() { "passed" } else { "failed" },
                    "components_checked": report.checked_components,
                    "rust_shims_checked": report.checked_shims,
                    "warnings": report.warnings,
                    "errors": report.errors
                })
            );
        }
    }
}
