use std::fs;
use std::path::{Path, PathBuf};

use oya_foundry_fitness_banned_primitives_kernel::{
    check_documented_genuine_need, scan_agent_instruction_file,
};

use crate::{path_has_component, slash_path, usage};

const DEFAULT_ROOTS: [&str; 4] = ["AGENTS.md", "CLAUDE.md", "docs", ".omc"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BannedPrimitivesValidateArgs {
    repo_root: PathBuf,
    roots: Vec<PathBuf>,
    known_rationales: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BannedPrimitivesGateReport {
    pub files_scanned: usize,
    pub sources_checked: usize,
    pub fences_checked: usize,
    pub usages_checked: usize,
    pub documented_exceptions: usize,
}

pub(crate) fn parse_banned_primitives_validate_args(
    args: Vec<String>,
) -> Result<BannedPrimitivesValidateArgs, String> {
    let mut parsed = BannedPrimitivesValidateArgs {
        repo_root: PathBuf::from("."),
        roots: DEFAULT_ROOTS.iter().map(PathBuf::from).collect(),
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
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    let report = check_documented_genuine_need(&sources, &usages, &args.known_rationales)
        .map_err(|error| error.message())?;

    Ok(BannedPrimitivesGateReport {
        files_scanned: files.len(),
        sources_checked: report.sources_checked,
        fences_checked: report.fences_checked,
        usages_checked: report.usages_checked,
        documented_exceptions: report.documented_exceptions,
    })
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
