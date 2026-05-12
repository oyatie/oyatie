use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_foundry_adr_index_kernel::{generate_adr_index, validate_adr_index, AdrDecisionRecord};

use crate::command_output::{json_escape, OutputFormat as DevCheckOutputFormat};

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
) -> Result<oya_foundry_adr_index_kernel::AdrIndexReport, String> {
    let records = read_adr_decision_records(&args.decisions_dir)?;
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

fn read_adr_decision_records(decisions_dir: &Path) -> Result<Vec<AdrDecisionRecord>, String> {
    let mut paths = fs::read_dir(decisions_dir)
        .map_err(|error| {
            format!(
                "ADR decisions dir unreadable {}: {error}",
                decisions_dir.display()
            )
        })?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("ADR decisions dir entry unreadable: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("ADR-") && name.ends_with(".md"))
    });
    paths.sort();

    paths
        .iter()
        .map(|path| read_adr_decision_record(path))
        .collect()
}

fn read_adr_decision_record(path: &Path) -> Result<AdrDecisionRecord, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("ADR path is not utf8: {}", path.display()))?;
    let id = file_name
        .get(0..8)
        .ok_or_else(|| format!("ADR filename too short: {file_name}"))?
        .to_string();
    let number = id
        .strip_prefix("ADR-")
        .ok_or_else(|| format!("ADR filename missing ADR prefix: {file_name}"))?
        .parse::<u16>()
        .map_err(|error| format!("ADR number invalid in {file_name}: {error}"))?;
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("ADR file unreadable {}: {error}", path.display()))?;
    let title = parse_adr_title(&contents, &id, path)?;
    let metadata = parse_adr_metadata(&contents);
    let status = required_adr_metadata(&metadata, "Status", path)?;
    let owner = required_adr_metadata(&metadata, "Owner", path)?;
    let date = required_adr_metadata(&metadata, "Date", path)?;

    Ok(AdrDecisionRecord {
        number,
        id,
        title,
        status,
        owner,
        date,
        path: format!("decisions/{file_name}"),
        supersedes: optional_single_adr_metadata(&metadata, "Supersedes"),
        superseded_by: optional_single_adr_metadata(&metadata, "Superseded-by"),
        related: optional_list_adr_metadata(&metadata, "Related"),
    })
}

fn parse_adr_title(contents: &str, expected_id: &str, path: &Path) -> Result<String, String> {
    let first_line = contents
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| format!("ADR file empty {}", path.display()))?;
    let title_line = first_line
        .trim()
        .strip_prefix("# ")
        .ok_or_else(|| format!("ADR first line must be an H1 in {}", path.display()))?;
    let Some((id, title)) = title_line.split_once(':') else {
        return Err(format!(
            "ADR H1 must use '<id>: <title>' in {}",
            path.display()
        ));
    };
    if id.trim() != expected_id {
        return Err(format!(
            "ADR H1 id {} does not match filename id {} in {}",
            id.trim(),
            expected_id,
            path.display()
        ));
    }
    let title = title.trim();
    if title.is_empty() {
        Err(format!("ADR title empty in {}", path.display()))
    } else {
        Ok(title.into())
    }
}

fn parse_adr_metadata(contents: &str) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    for line in contents.lines().take(30) {
        let mut trimmed = line.trim();
        if trimmed == "---" || trimmed.starts_with("## ") {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix('>') {
            trimmed = rest.trim();
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            trimmed = rest.trim();
        }
        let Some(rest) = trimmed.strip_prefix("**") else {
            continue;
        };
        let Some((key, value)) = rest.split_once(":**") else {
            continue;
        };
        metadata.insert(key.trim().into(), clean_adr_metadata_value(value));
    }
    metadata
}

fn required_adr_metadata(
    metadata: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String, String> {
    metadata
        .get(key)
        .filter(|value| !value.trim().is_empty() && value.trim() != "-")
        .cloned()
        .ok_or_else(|| format!("ADR metadata {key} missing in {}", path.display()))
}

fn optional_single_adr_metadata(metadata: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty() && *value != "-")
        .map(|value| vec![value.to_string()])
        .unwrap_or_default()
}

fn optional_list_adr_metadata(metadata: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty() && *item != "-")
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn clean_adr_metadata_value(value: &str) -> String {
    value.trim().replace('`', "")
}
