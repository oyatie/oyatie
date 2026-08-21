use std::fs;
use std::path::{Path, PathBuf};

use check_banned_primitives_kernel::{
    CommandInvocation, PrimitiveUsage, check_documented_genuine_need, scan_agent_instruction_file,
    scan_command_invocation,
};
use serde_json::Value;

use crate::{path_has_component, slash_path, usage};

const DEFAULT_ROOTS: [&str; 4] = ["AGENTS.md", "CLAUDE.md", "docs", ".omc"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BannedPrimitivesValidateArgs {
    repo_root: PathBuf,
    roots: Vec<PathBuf>,
    command_log_roots: Vec<PathBuf>,
    require_command_log_corpus: bool,
    known_rationales: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BannedPrimitivesGateReport {
    pub files_scanned: usize,
    pub sources_checked: usize,
    pub fences_checked: usize,
    pub command_log_records_checked: usize,
    pub usages_checked: usize,
    pub documented_exceptions: usize,
}

pub(crate) fn parse_banned_primitives_validate_args(
    args: Vec<String>,
) -> Result<BannedPrimitivesValidateArgs, String> {
    let mut parsed = BannedPrimitivesValidateArgs {
        repo_root: PathBuf::from("."),
        roots: DEFAULT_ROOTS.iter().map(PathBuf::from).collect(),
        command_log_roots: Vec::new(),
        require_command_log_corpus: false,
        known_rationales: Vec::new(),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.repo_root = PathBuf::from(value);
            }
            "--root" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.roots.push(PathBuf::from(value));
            }
            "--clear-default-roots" => parsed.roots.clear(),
            "--command-log-root" | "--command-log-corpus" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.command_log_roots.push(PathBuf::from(value));
            }
            "--require-command-log-corpus" => parsed.require_command_log_corpus = true,
            "--known-rationale" => {
                let Some(value) = iter.next() else {
                    return Err(usage());
                };
                parsed.known_rationales.push(value);
            }
            _ => return Err(usage()),
        }
    }
    if parsed.roots.is_empty() {
        return Err("banned-primitives requires at least one --root".to_string());
    }
    Ok(parsed)
}

pub(crate) fn validate_banned_primitives_gate(
    args: BannedPrimitivesValidateArgs,
) -> Result<BannedPrimitivesGateReport, String> {
    let files = collect_files(&args.repo_root, &args.roots)?;
    let mut sources = Vec::new();
    let mut usages = Vec::new();
    for path in &files {
        let contents = fs::read_to_string(path).map_err(|error| {
            format!(
                "banned-primitives source unreadable {}: {error}",
                path.display()
            )
        })?;
        let path_display = display_repo_path(&args.repo_root, path);
        let audit = scan_agent_instruction_file(&path_display, &contents)
            .map_err(|error| format!("banned-primitives scan failed: {error}"))?;
        if audit.source.fence_count > 0 {
            sources.push(audit.source);
            usages.extend(audit.usages);
        }
    }
    let (command_log_records_checked, command_log_usages) =
        scan_command_log_corpora(&args.repo_root, &args.command_log_roots)?;
    if args.require_command_log_corpus && command_log_records_checked == 0 {
        return Err(
            "banned-primitives command-log corpus required but no records were scanned".to_string(),
        );
    }
    if let Some(usage) = command_log_usages.first() {
        return Err(format!(
            "{}:{} uses forbidden command-log primitive {}",
            usage.path,
            usage.line,
            usage.primitive.as_str()
        ));
    }
    usages.extend(command_log_usages);
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    let report = check_documented_genuine_need(&sources, &usages, &args.known_rationales)
        .map_err(|error| error.message())?;

    Ok(BannedPrimitivesGateReport {
        files_scanned: files.len(),
        sources_checked: report.sources_checked,
        fences_checked: report.fences_checked,
        command_log_records_checked,
        usages_checked: report.usages_checked,
        documented_exceptions: report.documented_exceptions,
    })
}

fn scan_command_log_corpora(
    repo_root: &Path,
    roots: &[PathBuf],
) -> Result<(usize, Vec<PrimitiveUsage>), String> {
    let files = collect_command_log_files(repo_root, roots)?;
    let mut records_checked = 0usize;
    let mut usages = Vec::new();
    for file in files {
        let contents = fs::read_to_string(&file).map_err(|error| {
            format!(
                "banned-primitives command-log unreadable {}: {error}",
                file.display()
            )
        })?;
        let path_display = display_repo_path(repo_root, &file);
        for (index, line) in contents.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number = (index + 1) as u32;
            let value = serde_json::from_str::<Value>(line).map_err(|error| {
                format!(
                    "banned-primitives command-log malformed {path_display}:{line_number}: {error}"
                )
            })?;
            if !value
                .get("redacted")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                return Err(format!(
                    "banned-primitives command-log record {path_display}:{line_number} must set redacted=true"
                ));
            }
            let command = command_log_record_command(&value).ok_or_else(|| {
                format!(
                    "banned-primitives command-log record {path_display}:{line_number} has no command or tool/args surface"
                )
            })?;
            records_checked += 1;
            let scan = scan_command_invocation(CommandInvocation {
                source: path_display.clone(),
                line: line_number,
                command,
            });
            usages.extend(scan.usages);
        }
    }
    Ok((records_checked, usages))
}

fn command_log_record_command(value: &Value) -> Option<String> {
    if let Some(command) = find_command_field(value) {
        return Some(command);
    }
    let tool = value
        .get("tool")
        .and_then(Value::as_str)
        .and_then(non_empty)?;
    let args = string_array_field(value, "args")?;
    if args.is_empty() {
        return None;
    }
    Some(format!("{tool} {}", args.join(" ")))
}

fn find_command_field(value: &Value) -> Option<String> {
    find_string_field(value, &["command", "cmd"])
}

fn find_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(found) = map.get(*key).and_then(Value::as_str).and_then(non_empty) {
                    return Some(found.to_string());
                }
            }
            for key in ["arguments", "tool_input", "input"] {
                if let Some(found) = map
                    .get(key)
                    .and_then(Value::as_str)
                    .and_then(json_string_command_field)
                {
                    return Some(found);
                }
            }
            map.values()
                .find_map(|nested| find_string_field(nested, keys))
        }
        Value::Array(items) => items
            .iter()
            .find_map(|nested| find_string_field(nested, keys)),
        _ => None,
    }
}

fn json_string_command_field(value: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(value).ok()?;
    find_command_field(&parsed)
}

fn string_array_field(value: &Value, key: &str) -> Option<Vec<String>> {
    value
        .get(key)?
        .as_array()?
        .iter()
        .map(|item| item.as_str().and_then(non_empty).map(ToString::to_string))
        .collect::<Option<Vec<_>>>()
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn collect_command_log_files(repo_root: &Path, roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for root in roots {
        let path = if root.is_absolute() {
            root.clone()
        } else {
            repo_root.join(root)
        };
        if path.is_file() {
            if is_command_log_file(&path) {
                files.push(path);
            } else {
                return Err(format!(
                    "banned-primitives command-log corpus must be .jsonl: {}",
                    path.display()
                ));
            }
        } else if path.is_dir() {
            collect_command_log_dir(&path, &mut files)?;
        } else {
            return Err(format!(
                "banned-primitives command-log path does not exist: {}",
                path.display()
            ));
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_command_log_dir(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("banned-primitives command-log root unreadable: {error}"))?;
    for entry in entries {
        let entry = entry
            .map_err(|error| format!("banned-primitives command-log entry unreadable: {error}"))?;
        let path = entry.path();
        if path_has_component(&path, "target") || path_has_component(&path, ".git") {
            continue;
        }
        if path.is_dir() {
            collect_command_log_dir(&path, files)?;
        } else if path.is_file() && is_command_log_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_command_log_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")
}

fn collect_files(repo_root: &Path, roots: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for root in roots {
        let path = if root.is_absolute() {
            root.clone()
        } else {
            repo_root.join(root)
        };
        if path.is_file() {
            if is_scanned_file(&path) {
                files.push(path);
            }
        } else if path.is_dir() {
            collect_dir(&path, &mut files)?;
        } else {
            return Err(format!(
                "banned-primitives input path does not exist: {}",
                path.display()
            ));
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_dir(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("banned-primitives corpus root unreadable: {error}"))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("banned-primitives corpus entry unreadable: {error}"))?;
        let path = entry.path();
        if path_has_component(&path, "target") || path_has_component(&path, ".git") {
            continue;
        }
        if path.is_dir() {
            collect_dir(&path, files)?;
        } else if path.is_file() && is_scanned_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_scanned_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("md" | "json" | "toml" | "yaml" | "yml")
    )
}

fn display_repo_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(slash_path)
        .unwrap_or_else(|_| slash_path(path))
        .trim_start_matches("./")
        .to_string()
}
