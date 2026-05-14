use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_check_pre_push::{PrePushContractEvidence, validate_pre_push_contract};

use crate::command_output::{OutputFormat, json_escape};
use crate::command_process::{
    process_status_label, run_check_script_process, run_check_script_status_streaming,
};

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("pre-push") => match parse_pre_push_args(args.collect(), usage) {
            Ok(args) => run_pre_push(args),
            Err(message) => {
                eprintln!("{message}");
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!("{usage}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PrePushArgs {
    check_script_path: PathBuf,
    agents_doc_path: PathBuf,
    cli_manifest_path: PathBuf,
    hook_script_path: PathBuf,
    output_format: OutputFormat,
    verify_contract: bool,
}

fn parse_pre_push_args(args: Vec<String>, usage: &str) -> Result<PrePushArgs, String> {
    let mut parsed = PrePushArgs {
        check_script_path: PathBuf::from("scripts/check.sh"),
        agents_doc_path: PathBuf::from("docs/AGENTS.md"),
        cli_manifest_path: PathBuf::from("crates/oya-dev-cli/Cargo.toml"),
        hook_script_path: PathBuf::from("scripts/hooks/pre-push-repoctl.sh"),
        output_format: OutputFormat::Text,
        verify_contract: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--verify-contract" => parsed.verify_contract = true,
            "--check-script" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.check_script_path = PathBuf::from(value);
            }
            "--agents-doc" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.agents_doc_path = PathBuf::from(value);
            }
            "--cli-manifest" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.cli_manifest_path = PathBuf::from(value);
            }
            "--hook-script" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.hook_script_path = PathBuf::from(value);
            }
            "--format" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output_format =
                    OutputFormat::parse(&value).ok_or_else(|| usage.to_owned())?;
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn run_pre_push(args: PrePushArgs) -> ExitCode {
    if args.verify_contract {
        return run_pre_push_contract_check(args);
    }
    run_pre_push_checks(args)
}

fn run_pre_push_checks(args: PrePushArgs) -> ExitCode {
    if !args.check_script_path.is_file() {
        eprintln!(
            "repoctl pre-push failed: check script not found at {}",
            args.check_script_path.display()
        );
        return ExitCode::FAILURE;
    }

    match args.output_format {
        OutputFormat::Text => match run_check_script_status_streaming(&args.check_script_path) {
            Ok(status) => render_pre_push_text(&args.check_script_path, &status),
            Err(error) => {
                eprintln!(
                    "repoctl pre-push failed: could not run {}: {error}",
                    args.check_script_path.display()
                );
                ExitCode::FAILURE
            }
        },
        OutputFormat::Json => match run_check_script_process(&args.check_script_path) {
            Ok(output) => render_pre_push_json(&args.check_script_path, &output),
            Err(error) => {
                eprintln!(
                    "repoctl pre-push failed: could not run {}: {error}",
                    args.check_script_path.display()
                );
                ExitCode::FAILURE
            }
        },
    }
}

fn render_pre_push_text(check_script_path: &Path, status: &std::process::ExitStatus) -> ExitCode {
    if status.success() {
        println!("repoctl pre-push passed: {}", check_script_path.display());
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "repoctl pre-push failed: {} exited with {}",
            check_script_path.display(),
            process_status_label(status)
        );
        ExitCode::FAILURE
    }
}

fn render_pre_push_json(check_script_path: &Path, output: &std::process::Output) -> ExitCode {
    let status = if output.status.success() {
        "passed"
    } else {
        "failed"
    };
    println!(
        "{{\"command\":\"repoctl pre-push\",\"check_script\":\"{}\",\"status\":\"{}\",\"exit_code\":{},\"stdout\":\"{}\",\"stderr\":\"{}\"}}",
        json_escape(&check_script_path.display().to_string()),
        status,
        output.status.code().unwrap_or(-1),
        json_escape(&String::from_utf8_lossy(&output.stdout)),
        json_escape(&String::from_utf8_lossy(&output.stderr)),
    );
    if output.status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_pre_push_contract_check(args: PrePushArgs) -> ExitCode {
    match validate_pre_push_contract_files(&args) {
        Ok(report) => match args.output_format {
            OutputFormat::Text => {
                println!(
                    "repoctl pre-push contract validation passed: command={}, check-command={}, repoctl-bin=declared, hook=wired",
                    report.canonical_command, report.contract_check_command
                );
                ExitCode::SUCCESS
            }
            OutputFormat::Json => {
                println!(
                    "{{\"command\":\"repoctl pre-push --verify-contract\",\"status\":\"passed\",\"canonical_command\":\"{}\",\"contract_check_command\":\"{}\"}}",
                    json_escape(report.canonical_command),
                    json_escape(report.contract_check_command),
                );
                ExitCode::SUCCESS
            }
        },
        Err(message) => {
            eprintln!("repoctl pre-push contract validation failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn validate_pre_push_contract_files(
    args: &PrePushArgs,
) -> Result<oya_check_pre_push::PrePushContractReport, String> {
    let done_definition_doc = read_file_for_contract("AGENTS doc", &args.agents_doc_path)?;
    let check_script = read_file_for_contract("check script", &args.check_script_path)?;
    let cli_manifest = read_file_for_contract("CLI manifest", &args.cli_manifest_path)?;
    let hook_script = read_file_for_contract("hook script", &args.hook_script_path)?;

    validate_pre_push_contract(PrePushContractEvidence {
        done_definition_doc: &done_definition_doc,
        check_script: &check_script,
        cli_manifest: &cli_manifest,
        hook_script: &hook_script,
    })
    .map_err(|error| error.to_string())
}

fn read_file_for_contract(label: &str, path: &Path) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|error| format!("{label} unreadable {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage() -> &'static str {
        "usage text"
    }

    #[test]
    fn parse_pre_push_defaults_to_text_check_script() {
        let args = parse_pre_push_args(Vec::new(), usage()).expect("parse defaults");

        assert_eq!(args.check_script_path, PathBuf::from("scripts/check.sh"));
        assert_eq!(args.output_format, OutputFormat::Text);
        assert!(!args.verify_contract);
    }

    #[test]
    fn parse_pre_push_accepts_contract_json_paths() {
        let args = parse_pre_push_args(
            vec![
                "--verify-contract".to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
                "--check-script".to_owned(),
                "bin/check".to_owned(),
                "--agents-doc".to_owned(),
                "docs/agents.md".to_owned(),
                "--cli-manifest".to_owned(),
                "cli/Cargo.toml".to_owned(),
                "--hook-script".to_owned(),
                "hooks/pre-push".to_owned(),
            ],
            usage(),
        )
        .expect("parse args");

        assert!(args.verify_contract);
        assert_eq!(args.output_format, OutputFormat::Json);
        assert_eq!(args.check_script_path, PathBuf::from("bin/check"));
        assert_eq!(args.agents_doc_path, PathBuf::from("docs/agents.md"));
        assert_eq!(args.cli_manifest_path, PathBuf::from("cli/Cargo.toml"));
        assert_eq!(args.hook_script_path, PathBuf::from("hooks/pre-push"));
    }

    #[test]
    fn parse_pre_push_rejects_dangling_flag_with_usage() {
        let error = parse_pre_push_args(vec!["--format".to_owned()], usage())
            .expect_err("dangling format flag should fail");

        assert_eq!(error, usage());
    }
}
