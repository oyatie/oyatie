use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use oya_check_adr_index::{generate_adr_index, read_adr_decision_records, validate_adr_index};

use crate::command_output::{OutputFormat as DevCheckOutputFormat, json_escape};

pub(super) fn run(args: Vec<String>, usage: &str) -> ExitCode {
    match parse_doc_adr_index_args(args, usage) {
        Ok(args) => run_doc_adr_index(args),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DocAdrIndexArgs {
    decisions_dir: PathBuf,
    index_path: PathBuf,
    machine_path: PathBuf,
    output_format: DevCheckOutputFormat,
    write: bool,
}

fn parse_doc_adr_index_args(args: Vec<String>, usage: &str) -> Result<DocAdrIndexArgs, String> {
    let mut parsed = DocAdrIndexArgs {
        decisions_dir: PathBuf::from("docs/decisions"),
        index_path: PathBuf::from("docs/ADR-INDEX.md"),
        machine_path: PathBuf::from("docs/machine-readable/decisions.json"),
        output_format: DevCheckOutputFormat::Text,
        write: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--write" => parsed.write = true,
            "--decisions-dir" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.decisions_dir = PathBuf::from(value);
            }
            "--index" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.index_path = PathBuf::from(value);
            }
            "--machine" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.machine_path = PathBuf::from(value);
            }
            "--format" => {
                let Some(value) = iter.next() else {
                    return Err(usage.to_owned());
                };
                parsed.output_format =
                    DevCheckOutputFormat::parse(&value).ok_or_else(|| usage.to_owned())?;
            }
            _ => return Err(usage.to_owned()),
        }
    }
    Ok(parsed)
}

fn run_doc_adr_index(args: DocAdrIndexArgs) -> ExitCode {
    match run_doc_adr_index_result(&args) {
        Ok(report) => match args.output_format {
            DevCheckOutputFormat::Text => {
                let mode = if args.write { "wrote" } else { "checked" };
                println!(
                    "ADR index {mode}: {} records, next={}, statuses={}",
                    report.records,
                    report.next_adr,
                    report
                        .status_counts
                        .iter()
                        .map(|(status, count)| format!("{status}:{count}"))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                ExitCode::SUCCESS
            }
            DevCheckOutputFormat::Json => {
                let mode = if args.write { "write" } else { "check" };
                println!(
                    "{{\"command\":\"oya doc adr-index\",\"mode\":\"{}\",\"status\":\"passed\",\"records\":{},\"next_adr\":\"{}\"}}",
                    mode,
                    report.records,
                    json_escape(&report.next_adr)
                );
                ExitCode::SUCCESS
            }
        },
        Err(message) => {
            eprintln!("ADR index validation failed: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run_doc_adr_index_result(
    args: &DocAdrIndexArgs,
) -> Result<oya_check_adr_index::AdrIndexReport, String> {
    let records =
        read_adr_decision_records(&args.decisions_dir).map_err(|error| format!("{error:?}"))?;
    if args.write {
        let artifacts =
            generate_adr_index(records).map_err(|error| format!("ADR index invalid: {error:?}"))?;
        if let Some(parent) = args.index_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "ADR index directory unwritable {}: {error}",
                    parent.display()
                )
            })?;
        }
        if let Some(parent) = args.machine_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "ADR machine mirror directory unwritable {}: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&args.index_path, &artifacts.markdown).map_err(|error| {
            format!(
                "ADR index unwritable {}: {error}",
                args.index_path.display()
            )
        })?;
        fs::write(&args.machine_path, &artifacts.json).map_err(|error| {
            format!(
                "ADR machine mirror unwritable {}: {error}",
                args.machine_path.display()
            )
        })?;
        return Ok(artifacts.report);
    }

    let current_markdown = fs::read_to_string(&args.index_path).map_err(|error| {
        format!(
            "ADR index unreadable {}: {error}",
            args.index_path.display()
        )
    })?;
    let current_json = fs::read_to_string(&args.machine_path).map_err(|error| {
        format!(
            "ADR machine mirror unreadable {}: {error}",
            args.machine_path.display()
        )
    })?;
    validate_adr_index(records, &current_markdown, &current_json)
        .map_err(|error| format!("ADR index drift: {error:?}"))
}
