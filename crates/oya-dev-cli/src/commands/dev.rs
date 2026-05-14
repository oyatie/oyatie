use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::command_output::{OutputFormat, json_escape};
use crate::command_process::{
    process_status_label, replay_process_output, run_check_script_process,
};

pub(crate) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("check") => match parse_dev_check_args(args.collect(), usage) {
            Ok(args) => run_dev_check(args),
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
struct DevCheckArgs {
    check_script_path: PathBuf,
    output_format: OutputFormat,
}

fn parse_dev_check_args(args: Vec<String>, usage: &str) -> Result<DevCheckArgs, String> {
    let mut parsed = DevCheckArgs {
        check_script_path: PathBuf::from("scripts/check.sh"),
        output_format: OutputFormat::Text,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage.to_string());
        };
        match flag.as_str() {
            "--check-script" => parsed.check_script_path = PathBuf::from(value),
            "--format" => {
                parsed.output_format =
                    OutputFormat::parse(&value).ok_or_else(|| usage.to_string())?;
            }
            _ => return Err(usage.to_string()),
        }
    }
    Ok(parsed)
}

fn run_dev_check(args: DevCheckArgs) -> ExitCode {
    if !args.check_script_path.is_file() {
        eprintln!(
            "dev check failed: check script not found at {}",
            args.check_script_path.display()
        );
        return ExitCode::FAILURE;
    }

    let output = match run_check_script_process(&args.check_script_path) {
        Ok(output) => output,
        Err(error) => {
            eprintln!(
                "dev check failed: could not run {}: {error}",
                args.check_script_path.display()
            );
            return ExitCode::FAILURE;
        }
    };

    match args.output_format {
        OutputFormat::Text => render_dev_check_text(&args.check_script_path, &output),
        OutputFormat::Json => render_dev_check_json(&args.check_script_path, &output),
    }
}

fn render_dev_check_text(check_script_path: &Path, output: &std::process::Output) -> ExitCode {
    if let Err(error) = replay_process_output(output) {
        eprintln!("dev check failed: {error}");
        return ExitCode::FAILURE;
    }
    if output.status.success() {
        println!("dev check passed: {}", check_script_path.display());
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "dev check failed: {} exited with {}",
            check_script_path.display(),
            process_status_label(&output.status)
        );
        ExitCode::FAILURE
    }
}

fn render_dev_check_json(check_script_path: &Path, output: &std::process::Output) -> ExitCode {
    let status = if output.status.success() {
        "passed"
    } else {
        "failed"
    };
    println!(
        "{{\"command\":\"oya dev check\",\"check_script\":\"{}\",\"status\":\"{}\",\"exit_code\":{},\"stdout\":\"{}\",\"stderr\":\"{}\"}}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_dev_check_args_accepts_script_and_json_format() {
        let args = parse_dev_check_args(
            vec![
                "--check-script".into(),
                "scripts/custom-check.sh".into(),
                "--format".into(),
                "json".into(),
            ],
            "usage text",
        )
        .expect("dev check args parse");

        assert_eq!(
            args.check_script_path,
            PathBuf::from("scripts/custom-check.sh")
        );
        assert_eq!(args.output_format, OutputFormat::Json);
    }

    #[test]
    fn parse_dev_check_args_returns_usage_for_unknown_format() {
        assert_eq!(
            parse_dev_check_args(vec!["--format".into(), "yaml".into()], "usage text"),
            Err("usage text".to_string())
        );
    }
}
