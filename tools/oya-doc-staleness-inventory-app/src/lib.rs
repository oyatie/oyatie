#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fmt;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_PATHS: &[&str] = &["README.md", "AGENTS.md", "CLAUDE.md", "docs", "specs"];
const ROOT_POINTER_DOCS: &[&str] = &["README.md", "AGENTS.md", "CLAUDE.md"];
const DOC_SUFFIXES: &[&str] = &["md", "mdx", "json", "yaml", "yml", "toml", "tsv"];
const LOG_TS_PREFIX: &str = "@@@";
const CLAIM_BOUNDARY: &str = "inventory_only_no_deletion_no_archive_no_live_mutation";

#[derive(Debug, Clone)]
pub struct Args {
    json: bool,
    cutoff_days: u64,
    limit: usize,
    paths: Vec<String>,
    fail_on_missing_git_history: bool,
}

#[derive(Debug, Clone)]
struct StaleRecord {
    path: String,
    last_commit_unix: u64,
    age_days: f64,
    category: String,
}

#[derive(Debug)]
pub enum InventoryError {
    InvalidArgs(String),
    Time(String),
    Git(String),
}

impl fmt::Display for InventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InventoryError::InvalidArgs(message) => {
                write!(formatter, "invalid arguments: {message}")
            }
            InventoryError::Time(message) => write!(formatter, "time error: {message}"),
            InventoryError::Git(message) => write!(formatter, "git error: {message}"),
        }
    }
}

impl std::error::Error for InventoryError {}

pub fn run_from_env() -> Result<String, InventoryError> {
    let args = Args::parse(env::args().skip(1))?;
    run(args)
}

pub fn run(args: Args) -> Result<String, InventoryError> {
    let started = SystemTime::now();
    let now = unix_now()?;
    let files = match tracked_files(&args.paths) {
        Ok(files) => files,
        Err(error) => return Ok(git_unavailable_json(&args, error)),
    };
    let last_updates = match last_commit_unix_batch(&args.paths, &files) {
        Ok(last_updates) => last_updates,
        Err(error) => return Ok(git_unavailable_json(&args, error)),
    };

    let cutoff_seconds = args.cutoff_days.saturating_mul(24 * 60 * 60);
    let mut stale_records = Vec::new();
    let mut fresh_count = 0usize;
    let mut missing_history = Vec::new();
    let mut folders: BTreeMap<String, FolderCounts> = BTreeMap::new();

    for path in &files {
        let top = path.split('/').next().unwrap_or(path).to_owned();
        folders.entry(top).or_default().tracked += 1;
        let Some(ts) = last_updates.get(path).copied() else {
            missing_history.push(path.clone());
            continue;
        };
        let elapsed = now.saturating_sub(ts);
        let age_days = round2(elapsed as f64 / 86_400.0);
        let stale = elapsed > cutoff_seconds;
        if stale {
            if let Some(counts) = folders.get_mut(path.split('/').next().unwrap_or(path)) {
                counts.stale += 1;
            }
            stale_records.push(StaleRecord {
                path: path.clone(),
                last_commit_unix: ts,
                age_days,
                category: classify(path, stale),
            });
        } else {
            fresh_count += 1;
        }
    }

    stale_records.sort_by(|left, right| {
        right
            .age_days
            .partial_cmp(&left.age_days)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.path.cmp(&right.path))
    });

    let runtime_seconds = started
        .elapsed()
        .map(|duration| round3(duration.as_secs_f64()))
        .unwrap_or(0.0);
    let verdict = if missing_history.is_empty() {
        "PASS"
    } else {
        "WARN"
    };
    let output = render_inventory_json(
        &args,
        InventoryRender {
            verdict,
            files: &files,
            fresh_count,
            stale_records: &stale_records,
            missing_history: &missing_history,
            folders: &folders,
            runtime_seconds,
        },
    );
    if args.fail_on_missing_git_history && !missing_history.is_empty() {
        return Err(InventoryError::Git(format!(
            "missing git history for {} tracked files",
            missing_history.len()
        )));
    }
    Ok(output)
}

impl Args {
    fn parse<I>(raw_args: I) -> Result<Self, InventoryError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut json = false;
        let mut cutoff_days = 3u64;
        let mut limit = 50usize;
        let mut paths = Vec::new();
        let mut fail_on_missing_git_history = false;
        let mut iter = raw_args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--json" => json = true,
                "--cutoff-days" => {
                    let value = iter.next().ok_or_else(|| {
                        InventoryError::InvalidArgs("--cutoff-days requires a value".to_owned())
                    })?;
                    cutoff_days = value.parse().map_err(|_| {
                        InventoryError::InvalidArgs(format!(
                            "--cutoff-days must be an integer, got {value:?}"
                        ))
                    })?;
                }
                "--limit" => {
                    let value = iter.next().ok_or_else(|| {
                        InventoryError::InvalidArgs("--limit requires a value".to_owned())
                    })?;
                    limit = value.parse().map_err(|_| {
                        InventoryError::InvalidArgs(format!(
                            "--limit must be an integer, got {value:?}"
                        ))
                    })?;
                }
                "--paths" => {
                    paths.extend(iter.by_ref());
                    break;
                }
                "--fail-on-missing-git-history" => fail_on_missing_git_history = true,
                _ => return Err(InventoryError::InvalidArgs(format!("unknown flag {arg:?}"))),
            }
        }
        if paths.is_empty() {
            paths = DEFAULT_PATHS
                .iter()
                .map(|path| (*path).to_owned())
                .collect();
        }
        Ok(Self {
            json,
            cutoff_days,
            limit,
            paths,
            fail_on_missing_git_history,
        })
    }
}

#[derive(Debug, Default)]
struct FolderCounts {
    tracked: usize,
    stale: usize,
}

struct InventoryRender<'a> {
    verdict: &'a str,
    files: &'a [String],
    fresh_count: usize,
    stale_records: &'a [StaleRecord],
    missing_history: &'a [String],
    folders: &'a BTreeMap<String, FolderCounts>,
    runtime_seconds: f64,
}

fn unix_now() -> Result<u64, InventoryError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| InventoryError::Time(error.to_string()))
}

fn tracked_files(paths: &[String]) -> Result<Vec<String>, InventoryError> {
    let mut args = vec!["ls-files".to_owned(), "--".to_owned()];
    args.extend(paths.iter().cloned());
    let output = run_git(&args)?;
    let mut files: Vec<String> = output
        .lines()
        .filter(|line| is_doc_candidate(line))
        .map(ToOwned::to_owned)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    files.sort();
    Ok(files)
}

fn last_commit_unix_batch(
    paths: &[String],
    files: &[String],
) -> Result<HashMap<String, u64>, InventoryError> {
    if files.is_empty() {
        return Ok(HashMap::new());
    }
    let wanted: HashSet<&str> = files.iter().map(String::as_str).collect();
    let mut child = Command::new("git")
        .arg("log")
        .arg(format!("--format={LOG_TS_PREFIX}%ct"))
        .arg("--name-only")
        .arg("--")
        .args(paths)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| InventoryError::Git(error.to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| InventoryError::Git("git log stdout unavailable".to_owned()))?;
    let reader = BufReader::new(stdout);
    let mut current_ts = None;
    let mut seen = HashMap::new();
    for raw_line in reader.lines() {
        let line = raw_line.map_err(|error| InventoryError::Git(error.to_string()))?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(ts) = line.strip_prefix(LOG_TS_PREFIX) {
            current_ts = ts.parse::<u64>().ok();
            continue;
        }
        if let Some(ts) = current_ts {
            if wanted.contains(line) && !seen.contains_key(line) {
                seen.insert(line.to_owned(), ts);
                if seen.len() == files.len() {
                    let _ = child.kill();
                    break;
                }
            }
        }
    }
    let status = child
        .wait()
        .map_err(|error| InventoryError::Git(error.to_string()))?;
    if !status.success() && seen.len() != files.len() {
        return Err(InventoryError::Git(format!(
            "git log exited with status {status}"
        )));
    }
    Ok(seen)
}

fn run_git(args: &[String]) -> Result<String, InventoryError> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| InventoryError::Git(error.to_string()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(InventoryError::Git(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

fn is_doc_candidate(path: &str) -> bool {
    if ROOT_POINTER_DOCS.contains(&path) {
        return true;
    }
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| DOC_SUFFIXES.contains(&ext.to_ascii_lowercase().as_str()))
}

fn classify(path: &str, stale: bool) -> String {
    let parts: HashSet<&str> = path.split('/').collect();
    if ROOT_POINTER_DOCS.contains(&path) {
        return if stale {
            "root_pointer_review"
        } else {
            "root_pointer_fresh"
        }
        .to_owned();
    }
    if parts.contains("archive") || parts.contains("archives") || parts.contains("stale-documents")
    {
        return if stale {
            "archive_provenance_review"
        } else {
            "archive_recent"
        }
        .to_owned();
    }
    if path.starts_with("docs/decisions/") || parts.contains("decisions") {
        return if stale {
            "historical_provenance_review"
        } else {
            "historical_recent"
        }
        .to_owned();
    }
    if path.starts_with("specs/") {
        return if stale {
            "machine_readable_spec_review"
        } else {
            "machine_readable_spec_fresh"
        }
        .to_owned();
    }
    let markdown = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "md" | "mdx"));
    if markdown {
        return if stale {
            "active_markdown_review"
        } else {
            "active_markdown_fresh"
        }
        .to_owned();
    }
    if stale {
        "documentation_adjacent_review"
    } else {
        "documentation_adjacent_fresh"
    }
    .to_owned()
}

fn git_unavailable_json(args: &Args, error: InventoryError) -> String {
    let verdict = if args.fail_on_missing_git_history {
        "FAIL"
    } else {
        "SKIP"
    };
    format!(
        "{{\"claim_boundary\":\"{}\",\"error\":\"{}\",\"live_mutation_performed\":false,\"local_static_only\":true,\"reason\":\"git history unavailable\",\"verdict\":\"{}\"}}",
        CLAIM_BOUNDARY,
        json_escape(&error.to_string()),
        verdict
    )
}

fn render_inventory_json(args: &Args, render: InventoryRender<'_>) -> String {
    let stale_limit = render.stale_records.len().min(args.limit);
    let missing_limit = render.missing_history.len().min(args.limit);
    let mut out = String::new();
    out.push('{');
    write_json_string_field(&mut out, "claim_boundary", CLAIM_BOUNDARY, false);
    write_json_number_field(&mut out, "cutoff_days", args.cutoff_days, true);
    out.push_str(",\"folder_summary\":{");
    for (idx, (folder, counts)) in render.folders.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(folder));
        out.push_str("\":{");
        write_json_number_field(&mut out, "stale", counts.stale, false);
        write_json_number_field(&mut out, "tracked", counts.tracked, true);
        out.push('}');
    }
    out.push('}');
    write_json_number_field(&mut out, "fresh_file_count", render.fresh_count, true);
    out.push_str(",\"live_mutation_performed\":false");
    out.push_str(",\"local_static_only\":true");
    write_json_number_field(
        &mut out,
        "missing_history_count",
        render.missing_history.len(),
        true,
    );
    write_json_float_field(&mut out, "runtime_seconds", render.runtime_seconds, true);
    out.push_str(",\"scanned_paths\":[");
    for (idx, path) in args.paths.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(path));
        out.push('"');
    }
    out.push(']');
    write_json_number_field(
        &mut out,
        "stale_candidate_count",
        render.stale_records.len(),
        true,
    );
    out.push_str(",\"stale_candidates\":[");
    for (idx, item) in render.stale_records.iter().take(stale_limit).enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push('{');
        write_json_string_field(
            &mut out,
            "action",
            "audit_update_archive_or_delete_in_separate_evidence_backed_pr",
            false,
        );
        write_json_float_field(&mut out, "age_days", item.age_days, true);
        write_json_string_field(&mut out, "category", &item.category, true);
        write_json_number_field(&mut out, "last_commit_unix", item.last_commit_unix, true);
        write_json_string_field(&mut out, "path", &item.path, true);
        out.push('}');
    }
    out.push(']');
    write_json_number_field(&mut out, "tracked_file_count", render.files.len(), true);
    write_json_number_field(
        &mut out,
        "truncated_stale_candidates",
        render.stale_records.len().saturating_sub(args.limit),
        true,
    );
    write_json_string_field(&mut out, "verdict", render.verdict, true);
    out.push_str(",\"missing_history\":[");
    for (idx, path) in render
        .missing_history
        .iter()
        .take(missing_limit)
        .enumerate()
    {
        if idx > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(path));
        out.push('"');
    }
    out.push(']');
    out.push('}');
    if args.json { out } else { out }
}

fn write_json_string_field(out: &mut String, key: &str, value: &str, needs_comma: bool) {
    if needs_comma {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    out.push_str(&json_escape(value));
    out.push('"');
}

fn write_json_number_field<T: fmt::Display>(
    out: &mut String,
    key: &str,
    value: T,
    needs_comma: bool,
) {
    if needs_comma {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&value.to_string());
}

fn write_json_float_field(out: &mut String, key: &str, value: f64, needs_comma: bool) {
    if needs_comma {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    out.push_str(&format!("{value:.3}"));
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_docs_are_candidates() {
        assert!(is_doc_candidate("README.md"));
        assert!(is_doc_candidate("docs/ci/github-actions-lane-unlocker.md"));
        assert!(is_doc_candidate("specs/repo-hygiene-automation.json"));
        assert!(!is_doc_candidate("oya/service/src/lib.rs"));
    }

    #[test]
    fn categories_preserve_cleanup_boundaries() {
        assert_eq!(classify("README.md", true), "root_pointer_review");
        assert_eq!(
            classify(
                "docs/decisions/ADR-0516-github-actions-interim-lane-unlocker.md",
                true
            ),
            "historical_provenance_review"
        );
        assert_eq!(
            classify("docs/archive/stale-documents/old.md", true),
            "archive_provenance_review"
        );
        assert_eq!(
            classify("specs/repo-hygiene-automation.json", true),
            "machine_readable_spec_review"
        );
    }

    #[test]
    fn argument_defaults_keep_inventory_safe() {
        let args = Args::parse(Vec::<String>::new()).expect("parse defaults");
        assert!(args.json == false);
        assert_eq!(args.cutoff_days, 3);
        assert_eq!(args.limit, 50);
        assert!(!args.fail_on_missing_git_history);
        assert_eq!(args.paths, DEFAULT_PATHS);
    }

    #[test]
    fn output_records_non_mutation_claim_boundary() {
        let args = Args {
            json: true,
            cutoff_days: 3,
            limit: 1,
            paths: vec!["docs".to_owned()],
            fail_on_missing_git_history: false,
        };
        let mut folders = BTreeMap::new();
        folders.insert(
            "docs".to_owned(),
            FolderCounts {
                tracked: 1,
                stale: 1,
            },
        );
        let records = vec![StaleRecord {
            path: "docs/old.md".to_owned(),
            last_commit_unix: 1,
            age_days: 4.0,
            category: "active_markdown_review".to_owned(),
        }];
        let output = render_inventory_json(
            &args,
            InventoryRender {
                verdict: "PASS",
                files: &["docs/old.md".to_owned()],
                fresh_count: 0,
                stale_records: &records,
                missing_history: &[],
                folders: &folders,
                runtime_seconds: 0.1,
            },
        );
        assert!(output.contains(
            "\"claim_boundary\":\"inventory_only_no_deletion_no_archive_no_live_mutation\""
        ));
        assert!(output.contains("\"live_mutation_performed\":false"));
        assert!(output.contains("\"stale_candidate_count\":1"));
    }
}
