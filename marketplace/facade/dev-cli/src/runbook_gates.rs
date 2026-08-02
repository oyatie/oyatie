use std::fs;
use std::path::{Path, PathBuf};

use check_runbook_freshness::{RunbookFreshnessRecord, validate_runbook_freshness};
use check_runbook_index::validate_runbook_index_resolves;

use crate::{current_epoch_days_i64, parse_yyyy_mm_dd_to_epoch_days, slash_path, usage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunbookIndexValidateArgs {
    docs_dir: PathBuf,
}

pub(crate) fn parse_runbook_index_validate_args(
    args: Vec<String>,
) -> Result<RunbookIndexValidateArgs, String> {
    let mut parsed = RunbookIndexValidateArgs {
        docs_dir: PathBuf::from("docs"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(path) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--docs-dir" => parsed.docs_dir = PathBuf::from(path),
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_runbook_index_gate(args: RunbookIndexValidateArgs) -> Result<usize, String> {
    let runbook_index = fs::read_to_string(args.docs_dir.join("RUNBOOKS-INDEX.md"))
        .map_err(|error| format!("RUNBOOKS-INDEX unreadable: {error}"))?;
    let indexed_paths = parse_indexed_runbook_paths(&runbook_index)?;
    let existing_paths = list_existing_runbook_paths(&args.docs_dir.join("runbooks"))?;
    let report = validate_runbook_index_resolves(&indexed_paths, &existing_paths)
        .map_err(|error| format!("indexed runbook path invalid: {error:?}"))?;
    Ok(report.indexed_count)
}

fn parse_indexed_runbook_paths(index: &str) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for line in index.lines() {
        if !line.trim_start().starts_with("- ") {
            continue;
        }
        for candidate in line.split('`').skip(1).step_by(2) {
            if !candidate.ends_with(".md") {
                continue;
            }
            if let Some(runbook_relative_path) = candidate.strip_prefix("runbooks/") {
                paths.push(runbook_relative_path.to_string());
            } else {
                paths.push(candidate.to_string());
            }
        }
    }
    if paths.is_empty() {
        Err("RUNBOOKS-INDEX contains no indexed runbook .md entries".to_string())
    } else {
        Ok(paths)
    }
}

fn list_existing_runbook_paths(runbooks_dir: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    collect_runbook_paths(runbooks_dir, runbooks_dir, &mut paths)?;
    Ok(paths)
}

fn collect_runbook_paths(root: &Path, dir: &Path, paths: &mut Vec<String>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("runbooks directory unreadable: {error}"))?
    {
        let entry =
            entry.map_err(|error| format!("runbook directory entry unreadable: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_runbook_paths(root, &path, paths)?;
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("runbook path outside root: {error}"))?;
        paths.push(slash_path(relative));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunbookFreshnessValidateArgs {
    runbooks_dir: PathBuf,
    today_epoch_days: i64,
}

pub(crate) fn parse_runbook_freshness_validate_args(
    args: Vec<String>,
) -> Result<RunbookFreshnessValidateArgs, String> {
    let mut parsed = RunbookFreshnessValidateArgs {
        runbooks_dir: PathBuf::from("docs/runbooks"),
        today_epoch_days: current_epoch_days_i64()?,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let Some(value) = iter.next() else {
            return Err(usage());
        };
        match flag.as_str() {
            "--runbooks-dir" => parsed.runbooks_dir = PathBuf::from(value),
            "--today" => parsed.today_epoch_days = parse_yyyy_mm_dd_to_epoch_days(&value)?,
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_runbook_freshness_gate(
    args: RunbookFreshnessValidateArgs,
) -> Result<(usize, usize, usize), String> {
    let records = read_runbook_freshness_records(&args.runbooks_dir)?;
    let report = validate_runbook_freshness(records, args.today_epoch_days)
        .map_err(|error| format!("runbook freshness SLA violation: {error:?}"))?;
    Ok((
        report.runbooks_checked,
        report.severity_scoped_runbooks,
        report.unscoped_runbooks,
    ))
}

fn read_runbook_freshness_records(
    runbooks_dir: &Path,
) -> Result<Vec<RunbookFreshnessRecord>, String> {
    let mut relative_paths = Vec::new();
    collect_runbook_paths(runbooks_dir, runbooks_dir, &mut relative_paths)?;
    relative_paths.sort();
    relative_paths
        .into_iter()
        .map(|relative_path| {
            let path = runbooks_dir.join(&relative_path);
            let contents = fs::read_to_string(&path)
                .map_err(|error| format!("runbook unreadable {}: {error}", path.display()))?;
            parse_runbook_freshness_record(relative_path, &contents)
        })
        .collect()
}

fn parse_runbook_freshness_record(
    path: String,
    contents: &str,
) -> Result<RunbookFreshnessRecord, String> {
    let last_verified_epoch_days = match markdown_metadata_value(contents, "Last verified") {
        Some(value) if value.trim().is_empty() => None,
        Some(value) => Some(parse_yyyy_mm_dd_to_epoch_days(&value).map_err(|error| {
            format!("{path}: Last verified must start with YYYY-MM-DD: {error}")
        })?),
        None => None,
    };
    Ok(RunbookFreshnessRecord {
        path,
        status: markdown_metadata_value(contents, "Status"),
        severity_scope: markdown_metadata_value(contents, "Severity scope"),
        last_verified_epoch_days,
    })
}

fn markdown_metadata_value(contents: &str, label: &str) -> Option<String> {
    let bold_marker = format!("**{label}:**");
    let bare_marker = format!("{label}:");
    contents.lines().find_map(|line| {
        let mut normalized = line.trim_start();
        if let Some(after_quote) = normalized.strip_prefix('>') {
            normalized = after_quote.trim_start();
        }
        if let Some(after_list_marker) = normalized.strip_prefix("- ") {
            normalized = after_list_marker.trim_start();
        }
        normalized
            .strip_prefix(&bold_marker)
            .or_else(|| normalized.strip_prefix(&bare_marker))
            .map(|value| value.trim().to_string())
    })
}
