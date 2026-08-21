use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_adr_index::{AdrDecisionRecord, generate_adr_index, validate_adr_index};

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
) -> Result<check_adr_index::AdrIndexReport, String> {
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
    let base_decision_ids = paths
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .filter(|name| !name.contains("-amendment-"))
        .filter_map(|name| name.get(0..8).map(str::to_string))
        .collect::<BTreeSet<_>>();
    paths.retain(|path| {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return true;
        };
        if !name.contains("-amendment-") {
            return true;
        }
        name.get(0..8)
            .is_none_or(|id| !base_decision_ids.contains(id))
    });

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
        // Multi-id YAML lists (apex supersedes: [ADR-…, …]) must split into
        // one record element per ADR id — same path as Related.
        supersedes: optional_list_adr_metadata(&metadata, "Supersedes"),
        superseded_by: optional_list_adr_metadata(&metadata, "Superseded-by"),
        related: optional_list_adr_metadata(&metadata, "Related"),
    })
}

fn parse_adr_title(contents: &str, expected_id: &str, path: &Path) -> Result<String, String> {
    let contents = content_after_leading_frontmatter(contents);
    let first_line = contents
        .lines()
        .find(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with("<!--")
        })
        .ok_or_else(|| format!("ADR file empty {}", path.display()))?;
    let title_line = first_line
        .trim()
        .strip_prefix("# ")
        .ok_or_else(|| format!("ADR first line must be an H1 in {}", path.display()))?;
    let Some((id, title)) = split_adr_h1_title(title_line) else {
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

fn split_adr_h1_title(title_line: &str) -> Option<(&str, &str)> {
    match (title_line.find(':'), title_line.find(" — ")) {
        (Some(colon), Some(dash)) if dash < colon => {
            Some((&title_line[..dash], &title_line[dash + " — ".len()..]))
        }
        (Some(colon), _) => Some((&title_line[..colon], &title_line[colon + 1..])),
        (None, Some(dash)) => Some((&title_line[..dash], &title_line[dash + " — ".len()..])),
        (None, None) => None,
    }
}

fn split_leading_frontmatter(contents: &str) -> (Option<&str>, &str) {
    let mut lines = contents.split_inclusive('\n');
    let Some(first_line) = lines.next() else {
        return (None, contents);
    };
    if first_line.trim() != "---" {
        return (None, contents);
    }

    let frontmatter_start = first_line.len();
    let mut offset = first_line.len();
    for line in lines {
        let line_start = offset;
        offset += line.len();
        if line.trim() == "---" {
            return (
                Some(&contents[frontmatter_start..line_start]),
                &contents[offset..],
            );
        }
    }
    (None, contents)
}

fn content_after_leading_frontmatter(contents: &str) -> &str {
    split_leading_frontmatter(contents).1
}

fn parse_adr_metadata(contents: &str) -> BTreeMap<String, String> {
    let (frontmatter, contents) = split_leading_frontmatter(contents);
    let mut metadata = frontmatter
        .map(parse_frontmatter_metadata)
        .unwrap_or_default();
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
        if !trimmed.starts_with("**")
            && let Some((key, value)) = trimmed.split_once(':')
        {
            metadata
                .entry(canonical_adr_metadata_key(key.trim()).into())
                .or_insert_with(|| clean_adr_metadata_value(value));
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("**") else {
            continue;
        };
        let Some((key, value)) = rest.split_once(":**") else {
            continue;
        };
        metadata
            .entry(canonical_adr_metadata_key(key.trim()).into())
            .or_insert_with(|| clean_adr_metadata_value(value));
    }
    for line in contents.lines().take(40) {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            break;
        }
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let cells = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        if cells.len() < 2
            || cells[0].eq_ignore_ascii_case("field")
            || cells[0].chars().all(|character| character == '-')
            || cells[1].chars().all(|character| character == '-')
        {
            continue;
        }
        metadata
            .entry(canonical_adr_metadata_key(cells[0]).into())
            .or_insert_with(|| clean_adr_metadata_value(cells[1]));
    }
    metadata
}

fn parse_frontmatter_metadata(frontmatter: &str) -> BTreeMap<String, String> {
    let mut raw = BTreeMap::<String, Vec<String>>::new();
    let mut current_key = None::<String>;
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(item) = trimmed.strip_prefix("- ") {
            if let Some(key) = current_key.as_ref() {
                raw.entry(key.clone())
                    .or_default()
                    .extend(clean_yaml_metadata_values(item));
            }
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            current_key = None;
            continue;
        };
        let key = key.trim().to_string();
        current_key = Some(key.clone());
        raw.entry(key)
            .or_default()
            .extend(clean_yaml_metadata_values(value));
    }

    let mut metadata = BTreeMap::new();
    for (key, values) in raw {
        let Some(mapped_key) = frontmatter_metadata_key(&key) else {
            continue;
        };
        let values = values
            .into_iter()
            .map(|value| normalize_frontmatter_metadata_value(mapped_key, &value))
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if !values.is_empty() {
            metadata.insert(mapped_key.into(), values.join(", "));
        }
    }
    metadata
}

fn frontmatter_metadata_key(key: &str) -> Option<&'static str> {
    match key {
        "status" => Some("Status"),
        "date" => Some("Date"),
        "deciders" => Some("Deciders"),
        "owner" | "owner_team" | "decision_owner" => Some("Owner"),
        "owners" => Some("Owners"),
        "supersedes" => Some("Supersedes"),
        "superseded_by" => Some("Superseded-by"),
        "related" | "related_adrs" => Some("Related"),
        _ => None,
    }
}

fn canonical_adr_metadata_key(key: &str) -> &str {
    match key {
        "Superseded by" => "Superseded-by",
        "Related ADRs" => "Related",
        value => value,
    }
}

fn clean_yaml_metadata_values(value: &str) -> Vec<String> {
    let value = value.trim();
    if is_empty_metadata_value(value) || value == "~" || value == "[]" {
        return Vec::new();
    }
    if let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        return inner
            .split(',')
            .map(clean_yaml_scalar)
            .filter(|value| !value.is_empty())
            .collect();
    }
    let value = clean_yaml_scalar(value);
    if value.is_empty() {
        Vec::new()
    } else {
        vec![value]
    }
}

fn clean_yaml_scalar(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn normalize_frontmatter_metadata_value(key: &str, value: &str) -> String {
    if matches!(key, "Supersedes" | "Superseded-by" | "Related")
        && let Some(adr) = extract_adr_id(value)
    {
        return adr;
    }
    value.to_string()
}

fn extract_adr_id(value: &str) -> Option<String> {
    let start = value.find("ADR-")?;
    let candidate = value.get(start..start + 8)?;
    let digits = candidate.strip_prefix("ADR-")?;
    if digits.chars().all(|character| character.is_ascii_digit()) {
        Some(candidate.to_string())
    } else {
        None
    }
}

fn required_adr_metadata(
    metadata: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String, String> {
    let value = metadata.get(key).or_else(|| {
        if key == "Owner" {
            metadata
                .get("Owners")
                .or_else(|| metadata.get("Deciders"))
                .or_else(|| metadata.get("Authors"))
        } else {
            None
        }
    });
    value
        .filter(|value| !is_empty_metadata_value(value.trim()))
        .cloned()
        .ok_or_else(|| format!("ADR metadata {key} missing in {}", path.display()))
}

fn optional_single_adr_metadata(metadata: &BTreeMap<String, String>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !is_empty_metadata_value(value))
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
                .filter(|item| !is_empty_metadata_value(item))
                .map(|item| extract_adr_id(item).unwrap_or_else(|| item.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn clean_adr_metadata_value(value: &str) -> String {
    value.trim().replace('`', "")
}

fn is_empty_metadata_value(value: &str) -> bool {
    let value = value.trim();
    value.is_empty()
        || value == "-"
        || value == "—"
        || value.eq_ignore_ascii_case("none")
        || value.eq_ignore_ascii_case("n/a")
}
