//! `oya ops` — Rust-owned deployment helper surface.
//!
//! Root-level deployment helper scripts remain as compatibility shims only.
//! The operational logic lives here so OpenTofu and ops-controller
//! flows are not split across bespoke shell loops.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::command_output::OutputFormat;
use crate::command_process::{process_status_label, replay_process_output};

const DEFAULT_TOFU_BIN: &str = "tofu";
const DEFAULT_OCI_BIN: &str = "oci";
const DEFAULT_INFRA_DIR: &str = "infra/oci";
const DEFAULT_READINESS_DIR: &str = "evidence/oci-readiness";
const DEFAULT_A1_LOG: &str = "evidence/oci-readiness/a1-capacity-retry.log";
const DEFAULT_A1_MARKER: &str = "evidence/oci-readiness/a1-acquired.marker";
const DEFAULT_A1_MAX_ATTEMPTS: u32 = 288;
const DEFAULT_A1_SLEEP_SECS: u64 = 300;

#[derive(Clone, Debug, Eq, PartialEq)]
enum OpsAction {
    OciA1CapacityRetry,
    OciReadinessProbe,
    OnpremBringUp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct A1RetryArgs {
    tofu: PathBuf,
    infra_dir: PathBuf,
    log_path: PathBuf,
    success_marker: PathBuf,
    max_attempts: u32,
    sleep_secs: u64,
    dry_run: bool,
    output_format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReadinessProbeArgs {
    oci: PathBuf,
    out_dir: PathBuf,
    compartment_id: Option<String>,
    dry_run: bool,
    output_format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OnpremBringUpArgs {
    repo_root: PathBuf,
    dry_run: bool,
    output_format: OutputFormat,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandStep {
    program: String,
    args: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReadinessProbe {
    label: &'static str,
    args: Vec<String>,
    required: bool,
}

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut iter = args.into_iter();
    let action = match iter.next().as_deref() {
        Some("oci-a1-capacity-retry") => OpsAction::OciA1CapacityRetry,
        Some("oci-readiness-probe") => OpsAction::OciReadinessProbe,
        Some("onprem-bring-up") => OpsAction::OnpremBringUp,
        _ => {
            eprintln!("{usage}");
            return ExitCode::from(2);
        }
    };

    match action {
        OpsAction::OciA1CapacityRetry => run_a1_retry(iter.collect(), usage),
        OpsAction::OciReadinessProbe => run_readiness_probe(iter.collect(), usage),
        OpsAction::OnpremBringUp => run_onprem_bring_up(iter.collect(), usage),
    }
}

fn run_a1_retry(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_a1_retry_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    if parsed.dry_run {
        render_a1_retry_plan(&parsed);
        return ExitCode::SUCCESS;
    }
    match execute_a1_retry(&parsed) {
        Ok(attempt) => {
            render_a1_retry_success(&parsed, attempt);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("oci A1 capacity retry failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_readiness_probe(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_readiness_probe_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let probes = readiness_probe_plan(parsed.compartment_id.as_deref());
    if parsed.dry_run {
        render_readiness_probe_plan(&parsed, &probes);
        return ExitCode::SUCCESS;
    }
    match execute_readiness_probe(&parsed, &probes) {
        Ok(()) => {
            render_readiness_probe_success(&parsed, &probes);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("oci readiness probe failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_onprem_bring_up(args: Vec<String>, usage: &str) -> ExitCode {
    let parsed = match parse_onprem_bring_up_args(args, usage) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let steps = onprem_bring_up_plan(&parsed);
    if parsed.dry_run {
        render_onprem_bring_up_plan(&parsed, &steps);
        return ExitCode::SUCCESS;
    }
    match execute_steps(&steps, Some(&parsed.repo_root)) {
        Ok(()) => {
            render_onprem_bring_up_success(&parsed, &steps);
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("onprem bring-up failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_a1_retry_args(args: Vec<String>, usage: &str) -> Result<A1RetryArgs, String> {
    let mut parsed = A1RetryArgs {
        tofu: env::var_os("TOFU_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_TOFU_BIN)),
        infra_dir: env::var_os("INFRA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_INFRA_DIR)),
        log_path: env::var_os("A1_RETRY_LOG")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_A1_LOG)),
        success_marker: env::var_os("A1_SUCCESS_MARKER")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_A1_MARKER)),
        max_attempts: parse_env_u32("A1_RETRY_MAX", DEFAULT_A1_MAX_ATTEMPTS),
        sleep_secs: parse_env_u64("A1_RETRY_SLEEP", DEFAULT_A1_SLEEP_SECS),
        dry_run: false,
        output_format: OutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--tofu" | "--tofu-bin" => {
                parsed.tofu = PathBuf::from(next_value(
                    "oya ops oci-a1-capacity-retry",
                    &mut iter,
                    &flag,
                )?)
            }
            "--infra-dir" => {
                parsed.infra_dir = PathBuf::from(next_value(
                    "oya ops oci-a1-capacity-retry",
                    &mut iter,
                    &flag,
                )?)
            }
            "--log" | "--log-path" => {
                parsed.log_path = PathBuf::from(next_value(
                    "oya ops oci-a1-capacity-retry",
                    &mut iter,
                    &flag,
                )?)
            }
            "--success-marker" => {
                parsed.success_marker = PathBuf::from(next_value(
                    "oya ops oci-a1-capacity-retry",
                    &mut iter,
                    &flag,
                )?)
            }
            "--max-attempts" => {
                let value = next_value("oya ops oci-a1-capacity-retry", &mut iter, &flag)?;
                parsed.max_attempts = value.parse::<u32>().map_err(|_| {
                    "oya ops oci-a1-capacity-retry: --max-attempts must be a positive integer"
                        .to_string()
                })?;
                if parsed.max_attempts == 0 {
                    return Err(
                        "oya ops oci-a1-capacity-retry: --max-attempts must be > 0".to_string()
                    );
                }
            }
            "--sleep-secs" => {
                let value = next_value("oya ops oci-a1-capacity-retry", &mut iter, &flag)?;
                parsed.sleep_secs = value.parse::<u64>().map_err(|_| {
                    "oya ops oci-a1-capacity-retry: --sleep-secs must be an integer".to_string()
                })?;
            }
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                let value = next_value("oya ops oci-a1-capacity-retry", &mut iter, &flag)?;
                parsed.output_format = OutputFormat::parse(&value).ok_or_else(|| {
                    "oya ops oci-a1-capacity-retry: --format must be text or json".to_string()
                })?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn parse_readiness_probe_args(
    args: Vec<String>,
    usage: &str,
) -> Result<ReadinessProbeArgs, String> {
    let mut parsed = ReadinessProbeArgs {
        oci: env::var_os("OCI_BIN")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_OCI_BIN)),
        out_dir: env::var_os("OCI_READINESS_OUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_READINESS_DIR)),
        compartment_id: env::var("COMPARTMENT_ID")
            .ok()
            .filter(|value| !value.trim().is_empty()),
        dry_run: false,
        output_format: OutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--oci" | "--oci-bin" => {
                parsed.oci =
                    PathBuf::from(next_value("oya ops oci-readiness-probe", &mut iter, &flag)?)
            }
            "--out-dir" => {
                parsed.out_dir =
                    PathBuf::from(next_value("oya ops oci-readiness-probe", &mut iter, &flag)?)
            }
            "--compartment-id" => {
                parsed.compartment_id =
                    Some(next_value("oya ops oci-readiness-probe", &mut iter, &flag)?)
            }
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                let value = next_value("oya ops oci-readiness-probe", &mut iter, &flag)?;
                parsed.output_format = OutputFormat::parse(&value).ok_or_else(|| {
                    "oya ops oci-readiness-probe: --format must be text or json".to_string()
                })?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn parse_onprem_bring_up_args(args: Vec<String>, usage: &str) -> Result<OnpremBringUpArgs, String> {
    let mut parsed = OnpremBringUpArgs {
        repo_root: env::var_os("OYA_REPO_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
        dry_run: false,
        output_format: OutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                parsed.repo_root =
                    PathBuf::from(next_value("oya ops onprem-bring-up", &mut iter, &flag)?)
            }
            "--dry-run" => parsed.dry_run = true,
            "--format" => {
                let value = next_value("oya ops onprem-bring-up", &mut iter, &flag)?;
                parsed.output_format = OutputFormat::parse(&value).ok_or_else(|| {
                    "oya ops onprem-bring-up: --format must be text or json".to_string()
                })?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn execute_a1_retry(args: &A1RetryArgs) -> Result<u32, String> {
    ensure_parent_dir(&args.log_path)?;
    ensure_parent_dir(&args.success_marker)?;
    for attempt in 1..=args.max_attempts {
        append_log(
            &args.log_path,
            &format!(
                "[{}] attempt {}/{}: tofu -chdir={} apply -var=create_stage0_a1=true",
                epoch_now(),
                attempt,
                args.max_attempts,
                args.infra_dir.display()
            ),
        )?;
        let apply = Command::new(&args.tofu)
            .arg(format!("-chdir={}", args.infra_dir.display()))
            .args([
                "apply",
                "-auto-approve",
                "-no-color",
                "-var=create_stage0_a1=true",
            ])
            .output()
            .map_err(|error| format!("could not execute {}: {error}", args.tofu.display()))?;
        append_process_output(&args.log_path, &apply)?;
        if apply.status.success() && stage0_instance_exists(args)? {
            let ip = stage0_public_ip(args).unwrap_or_else(|_| "unknown".to_string());
            fs::write(
                &args.success_marker,
                format!(
                    "acquired_epoch={}\nattempt={}\npublic_ip={}\n",
                    epoch_now(),
                    attempt,
                    ip.trim()
                ),
            )
            .map_err(|error| {
                format!(
                    "could not write success marker {}: {error}",
                    args.success_marker.display()
                )
            })?;
            append_log(
                &args.log_path,
                &format!(
                    "[{}] SUCCESS: A1 stage-0 acquired on attempt {attempt}",
                    epoch_now()
                ),
            )?;
            return Ok(attempt);
        }

        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&apply.stdout),
            String::from_utf8_lossy(&apply.stderr)
        );
        if combined.contains("Out of host capacity") {
            append_log(
                &args.log_path,
                &format!(
                    "[{}] capacity miss; sleep {}s",
                    epoch_now(),
                    args.sleep_secs
                ),
            )?;
            thread::sleep(Duration::from_secs(args.sleep_secs));
            continue;
        }
        return Err(format!(
            "non-capacity OpenTofu failure on attempt {attempt}: {}",
            process_status_label(&apply.status)
        ));
    }
    Err(format!("exhausted {} attempts", args.max_attempts))
}

fn stage0_instance_exists(args: &A1RetryArgs) -> Result<bool, String> {
    let output = Command::new(&args.tofu)
        .arg(format!("-chdir={}", args.infra_dir.display()))
        .args(["state", "list"])
        .output()
        .map_err(|error| {
            format!(
                "could not execute {} state list: {error}",
                args.tofu.display()
            )
        })?;
    if !output.status.success() {
        return Ok(false);
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == "oci_core_instance.stage0[0]"))
}

fn stage0_public_ip(args: &A1RetryArgs) -> Result<String, String> {
    let output = Command::new(&args.tofu)
        .arg(format!("-chdir={}", args.infra_dir.display()))
        .args(["output", "-raw", "stage0_public_ip"])
        .output()
        .map_err(|error| format!("could not execute {} output: {error}", args.tofu.display()))?;
    if !output.status.success() {
        return Err(process_status_label(&output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn readiness_probe_plan(compartment_id: Option<&str>) -> Vec<ReadinessProbe> {
    let mut probes = vec![
        probe("regions", true, ["iam", "region", "list"]),
        probe(
            "compartments",
            true,
            [
                "iam",
                "compartment",
                "list",
                "--compartment-id-in-subtree",
                "true",
                "--all",
            ],
        ),
    ];
    if let Some(compartment_id) = compartment_id {
        probes.extend([
            probe_owned(
                "kms-vaults",
                true,
                ["kms", "management", "vault", "list", "--compartment-id"],
                compartment_id,
            ),
            probe_owned(
                "api-gateways",
                false,
                ["api-gateway", "gateway", "list", "--compartment-id"],
                compartment_id,
            ),
            probe_owned(
                "oke-clusters",
                false,
                ["ce", "cluster", "list", "--compartment-id"],
                compartment_id,
            ),
            probe_owned(
                "compute-instances",
                false,
                ["compute", "instance", "list", "--compartment-id"],
                compartment_id,
            ),
            probe_owned(
                "vault-secrets",
                false,
                ["vault", "secret", "list", "--compartment-id"],
                compartment_id,
            ),
        ]);
    }
    probes
}

fn execute_readiness_probe(
    args: &ReadinessProbeArgs,
    probes: &[ReadinessProbe],
) -> Result<(), String> {
    fs::create_dir_all(&args.out_dir).map_err(|error| {
        format!(
            "could not create readiness dir {}: {error}",
            args.out_dir.display()
        )
    })?;
    let mut required_failures = Vec::new();
    for probe in probes {
        let output_path = args
            .out_dir
            .join(format!("{}-{}.json", epoch_now(), probe.label));
        let output = Command::new(&args.oci)
            .args(&probe.args)
            .args(["--output", "json"])
            .output()
            .map_err(|error| format!("could not execute {}: {error}", args.oci.display()))?;
        if output.status.success() {
            fs::write(&output_path, &output.stdout).map_err(|error| {
                format!(
                    "could not write readiness output {}: {error}",
                    output_path.display()
                )
            })?;
            continue;
        }
        fs::write(output_path.with_extension("failed.json"), &output.stdout).map_err(|error| {
            format!(
                "could not write failed readiness output {}: {error}",
                output_path.display()
            )
        })?;
        fs::write(output_path.with_extension("err"), &output.stderr).map_err(|error| {
            format!(
                "could not write readiness error {}: {error}",
                output_path.display()
            )
        })?;
        if probe.required {
            required_failures.push(format!(
                "{} failed with {}",
                probe.label,
                process_status_label(&output.status)
            ));
        }
    }
    if required_failures.is_empty() {
        Ok(())
    } else {
        Err(required_failures.join("; "))
    }
}

fn onprem_bring_up_plan(_: &OnpremBringUpArgs) -> Vec<CommandStep> {
    vec![
        step("make", ["bootstrap"]),
        step("make", ["install"]),
        step("make", ["ops"]),
    ]
}

fn execute_steps(steps: &[CommandStep], cwd: Option<&Path>) -> Result<(), String> {
    for step in steps {
        let mut command = Command::new(&step.program);
        command.args(&step.args);
        if let Some(cwd) = cwd {
            command.current_dir(cwd);
        }
        let output = command
            .output()
            .map_err(|error| format!("could not execute {}: {error}", render_step(step)))?;
        replay_process_output(&output)?;
        if !output.status.success() {
            return Err(format!(
                "{} failed with {}",
                render_step(step),
                process_status_label(&output.status)
            ));
        }
    }
    Ok(())
}

fn render_a1_retry_plan(args: &A1RetryArgs) {
    let steps = vec![
        CommandStep {
            program: args.tofu.display().to_string(),
            args: vec![
                format!("-chdir={}", args.infra_dir.display()),
                "apply".into(),
                "-auto-approve".into(),
                "-no-color".into(),
                "-var=create_stage0_a1=true".into(),
            ],
        },
        CommandStep {
            program: args.tofu.display().to_string(),
            args: vec![
                format!("-chdir={}", args.infra_dir.display()),
                "state".into(),
                "list".into(),
            ],
        },
    ];
    match args.output_format {
        OutputFormat::Text => {
            println!("OCI A1 capacity retry Rust runner dry-run");
            println!("authority=opentofu");
            println!("infra_dir={}", args.infra_dir.display());
            println!("max_attempts={}", args.max_attempts);
            println!("sleep_secs={}", args.sleep_secs);
            println!("log={}", args.log_path.display());
            println!("success_marker={}", args.success_marker.display());
            for step in steps {
                println!("step: {}", render_step(&step));
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                json!({
                    "command": "oya ops oci-a1-capacity-retry",
                    "dry_run": true,
                    "authority": "opentofu",
                    "infra_dir": args.infra_dir,
                    "max_attempts": args.max_attempts,
                    "sleep_secs": args.sleep_secs,
                    "log": args.log_path,
                    "success_marker": args.success_marker,
                    "steps": steps_to_json(&steps)
                })
            );
        }
    }
}

fn render_a1_retry_success(args: &A1RetryArgs, attempt: u32) {
    match args.output_format {
        OutputFormat::Text => println!(
            "OCI A1 capacity retry passed: attempt={}, marker={}",
            attempt,
            args.success_marker.display()
        ),
        OutputFormat::Json => println!(
            "{}",
            json!({
                "command": "oya ops oci-a1-capacity-retry",
                "status": "passed",
                "attempt": attempt,
                "success_marker": args.success_marker
            })
        ),
    }
}

fn render_readiness_probe_plan(args: &ReadinessProbeArgs, probes: &[ReadinessProbe]) {
    match args.output_format {
        OutputFormat::Text => {
            println!("OCI readiness probe Rust runner dry-run");
            println!("mutation_authority=none (read-only OCI CLI probes)");
            println!("oci={}", args.oci.display());
            println!("out_dir={}", args.out_dir.display());
            if let Some(compartment_id) = &args.compartment_id {
                println!("compartment_id={compartment_id}");
            } else {
                println!("compartment_id=<absent; compartment-scoped probes skipped>");
            }
            for probe in probes {
                println!(
                    "probe: {} required={} args={}",
                    probe.label,
                    probe.required,
                    probe.args.join(" ")
                );
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                json!({
                    "command": "oya ops oci-readiness-probe",
                    "dry_run": true,
                    "mutation_authority": "none",
                    "oci": args.oci,
                    "out_dir": args.out_dir,
                    "compartment_id": args.compartment_id,
                    "probes": probes_to_json(probes)
                })
            );
        }
    }
}

fn render_readiness_probe_success(args: &ReadinessProbeArgs, probes: &[ReadinessProbe]) {
    match args.output_format {
        OutputFormat::Text => println!(
            "OCI readiness probe passed: {} probes planned, out_dir={}",
            probes.len(),
            args.out_dir.display()
        ),
        OutputFormat::Json => println!(
            "{}",
            json!({
                "command": "oya ops oci-readiness-probe",
                "status": "passed",
                "out_dir": args.out_dir,
                "probes": probes_to_json(probes)
            })
        ),
    }
}

fn render_onprem_bring_up_plan(args: &OnpremBringUpArgs, steps: &[CommandStep]) {
    match args.output_format {
        OutputFormat::Text => {
            println!("On-prem bring-up Rust runner dry-run");
            println!("repo_root={}", args.repo_root.display());
            println!("deployment_authority=OpenTofu");
            println!("manual_ssh_troubleshooting_allowed=false");
            println!("day_2_surface=https://ops.oyatie.com");
            for step in steps {
                println!("step: {}", render_step(step));
            }
        }
        OutputFormat::Json => println!(
            "{}",
            json!({
                "command": "oya ops onprem-bring-up",
                "dry_run": true,
                "repo_root": args.repo_root,
                "deployment_authority": "OpenTofu",
                "manual_ssh_troubleshooting_allowed": false,
                "day_2_surface": "https://ops.oyatie.com",
                "steps": steps_to_json(steps)
            })
        ),
    }
}

fn render_onprem_bring_up_success(args: &OnpremBringUpArgs, steps: &[CommandStep]) {
    match args.output_format {
        OutputFormat::Text => println!(
            "onprem bring-up passed: {} steps via OpenTofu; ops=https://ops.oyatie.com",
            steps.len()
        ),
        OutputFormat::Json => println!(
            "{}",
            json!({
                "command": "oya ops onprem-bring-up",
                "status": "passed",
                "repo_root": args.repo_root,
                "steps": steps_to_json(steps),
                "day_2_surface": "https://ops.oyatie.com"
            })
        ),
    }
}

fn probe<const N: usize>(
    label: &'static str,
    required: bool,
    args: [&'static str; N],
) -> ReadinessProbe {
    ReadinessProbe {
        label,
        required,
        args: args.into_iter().map(str::to_string).collect(),
    }
}

fn probe_owned<const N: usize>(
    label: &'static str,
    required: bool,
    args: [&'static str; N],
    trailing: &str,
) -> ReadinessProbe {
    let mut args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
    args.push(trailing.to_string());
    ReadinessProbe {
        label,
        required,
        args,
    }
}

fn step<const N: usize>(program: &str, args: [&str; N]) -> CommandStep {
    CommandStep {
        program: program.to_string(),
        args: args.into_iter().map(str::to_string).collect(),
    }
}

fn steps_to_json(steps: &[CommandStep]) -> Vec<serde_json::Value> {
    steps
        .iter()
        .map(|step| {
            json!({
                "program": step.program,
                "args": step.args,
                "rendered": render_step(step)
            })
        })
        .collect()
}

fn probes_to_json(probes: &[ReadinessProbe]) -> Vec<serde_json::Value> {
    probes
        .iter()
        .map(|probe| {
            json!({
                "label": probe.label,
                "required": probe.required,
                "args": probe.args
            })
        })
        .collect()
}

fn render_step(step: &CommandStep) -> String {
    if step.args.is_empty() {
        step.program.clone()
    } else {
        format!("{} {}", step.program, step.args.join(" "))
    }
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    Ok(())
}

fn append_log(path: &Path, line: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("could not open log {}: {error}", path.display()))?;
    writeln!(file, "{line}")
        .map_err(|error| format!("could not write log {}: {error}", path.display()))
}

fn append_process_output(path: &Path, output: &std::process::Output) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("could not open log {}: {error}", path.display()))?;
    file.write_all(&output.stdout)
        .and_then(|_| file.write_all(&output.stderr))
        .map_err(|error| {
            format!(
                "could not append process output to {}: {error}",
                path.display()
            )
        })
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_env_u32(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn next_value(
    scope: &str,
    iter: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, String> {
    let value = iter
        .next()
        .ok_or_else(|| format!("{scope}: {flag} requires a value"))?;
    if value.starts_with('-') {
        return Err(format!("{scope}: {flag} requires a value"));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_plan_skips_compartment_scoped_probes_without_compartment() {
        let probes = readiness_probe_plan(None);
        assert_eq!(probes.len(), 2);
        assert!(probes.iter().all(|probe| probe.required));
    }

    #[test]
    fn readiness_plan_adds_compartment_scoped_optional_probes() {
        let probes = readiness_probe_plan(Some("ocid1.compartment.example"));
        assert!(
            probes
                .iter()
                .any(|probe| probe.label == "compute-instances")
        );
        assert!(probes.iter().any(|probe| !probe.required));
    }
}
