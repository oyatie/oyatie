//! `oya gate validate planning-ssot-coverage` — planning single-source-of-truth
//! drift gate (Direction A: frontmatter-bound, bidirectional, supersession-aware).
//!
//! Enforces that `specs/masterplan.json` is a faithful planning SSOT:
//!  1. Every ADR with frontmatter `planning_impact: true` is referenced in
//!     `masterplan.json#planning_authority.bound_adrs` — unless its status is
//!     Superseded/Deprecated (supersession-aware; immutable-ADR pattern).
//!  2. Every `planning_authority.bound_adrs` id maps to an existing ADR file.
//!  3. Every `planning_authority.bound_specs` path exists and (for specs/*.json)
//!     has a `root-hub-pointers.json` entry.
//!
//! Default severity is report-only (exit 0, prints findings). `--severity error`
//! makes any finding blocking (exit 1). This supports the report-only -> blocking
//! ratchet recorded in `docs/ideas/planning-ssot-drift-prevention.md`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlanningSsotCoverageArgs {
    pub(crate) repo_root: PathBuf,
    pub(crate) blocking: bool,
}

pub(crate) fn parse_planning_ssot_coverage_args(
    args: Vec<String>,
) -> Result<PlanningSsotCoverageArgs, String> {
    let mut parsed = PlanningSsotCoverageArgs {
        repo_root: PathBuf::from("."),
        blocking: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                parsed.repo_root = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--repo-root requires a value".to_string())?,
                );
            }
            "--severity" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--severity requires a value".to_string())?;
                parsed.blocking = value == "error";
            }
            other => {
                return Err(format!(
                    "planning-ssot-coverage: unknown flag {other:?}; allowed: --repo-root, --severity <warn|error>"
                ));
            }
        }
    }
    Ok(parsed)
}

#[derive(Default, Debug)]
pub(crate) struct PlanningSsotCoverageReport {
    pub(crate) planning_adrs: usize,
    pub(crate) bound_adrs: usize,
    pub(crate) bound_specs: usize,
    pub(crate) findings: Vec<String>,
}

fn read_frontmatter(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn frontmatter_value<'a>(frontmatter: &'a str, key: &str) -> Option<&'a str> {
    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&format!("{key}:")) {
            return Some(value.trim());
        }
    }
    None
}

pub(crate) fn validate_planning_ssot_coverage_gate(
    args: PlanningSsotCoverageArgs,
) -> Result<PlanningSsotCoverageReport, String> {
    let root = &args.repo_root;
    let mut report = PlanningSsotCoverageReport::default();

    // bound_adrs + bound_specs from masterplan.planning_authority
    let masterplan = read_json(&root.join("specs/masterplan.json"), "masterplan")?;
    let authority = masterplan
        .get("planning_authority")
        .and_then(Value::as_object)
        .ok_or_else(|| "masterplan.json lacks planning_authority object".to_string())?;
    let bound_adrs: BTreeSet<String> = authority
        .get("bound_adrs")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    report.bound_adrs = bound_adrs.len();
    let bound_specs: Vec<String> = authority
        .get("bound_specs")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    report.bound_specs = bound_specs.len();

    // root-hub current_paths
    let root_hub = read_json(
        &root.join("specs/root-hub-pointers.json"),
        "root-hub-pointers",
    )?;
    let mut root_hub_paths: BTreeSet<String> = BTreeSet::new();
    if let Some(entries) = root_hub.get("entry_points").and_then(Value::as_object) {
        for entry in entries.values() {
            if let Some(path) = entry.get("current_path").and_then(Value::as_str) {
                root_hub_paths.insert(path.trim_start_matches('/').to_string());
            }
        }
    }

    // scan ADRs for planning_impact + status
    let decisions = root.join("docs/decisions");
    let mut adr_ids_on_disk: BTreeSet<String> = BTreeSet::new();
    if let Ok(read_dir) = fs::read_dir(&decisions) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) if n.starts_with("ADR-") && n.ends_with(".md") => n.to_string(),
                _ => continue,
            };
            let id = name.get(0..8).unwrap_or_default().to_string();
            adr_ids_on_disk.insert(id.clone());
            let text = fs::read_to_string(&path).unwrap_or_default();
            let Some(fm) = read_frontmatter(&text) else {
                continue;
            };
            let planning = frontmatter_value(fm, "planning_impact") == Some("true");
            if !planning {
                continue;
            }
            report.planning_adrs += 1;
            let status = frontmatter_value(fm, "status").unwrap_or("").to_lowercase();
            let retired = status.contains("superseded") || status.contains("deprecated");
            if retired {
                continue;
            }
            if !bound_adrs.contains(&id) {
                report.findings.push(format!(
                    "[ADR_UNBOUND] {id} has planning_impact:true but is not in masterplan.planning_authority.bound_adrs"
                ));
            }
        }
    }

    // bidirectional: every bound_adr exists on disk
    for id in &bound_adrs {
        if !adr_ids_on_disk.contains(id) {
            report.findings.push(format!(
                "[BOUND_ADR_MISSING] masterplan binds {id} but no docs/decisions/{id}-*.md exists"
            ));
        }
    }

    // bound_specs exist + specs/*.json have a root-hub entry
    for spec in &bound_specs {
        if !root.join(spec).exists() {
            report.findings.push(format!(
                "[BOUND_SPEC_MISSING] masterplan binds {spec} but the file does not exist"
            ));
            continue;
        }
        if spec.starts_with("specs/") && spec.ends_with(".json") && !root_hub_paths.contains(spec) {
            report.findings.push(format!(
                "[BOUND_SPEC_NO_ROOT_HUB] {spec} is bound but has no root-hub-pointers entry"
            ));
        }
    }

    Ok(report)
}

fn read_json(path: &Path, label: &str) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("{label} unreadable {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("{label} invalid JSON: {error}"))
}

pub(crate) fn run_planning_ssot_coverage(args: Vec<String>) -> ExitCode {
    let parsed = match parse_planning_ssot_coverage_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let blocking = parsed.blocking;
    match validate_planning_ssot_coverage_gate(parsed) {
        Ok(report) => {
            if report.findings.is_empty() {
                println!(
                    "planning-ssot-coverage validation passed: {} planning_impact ADRs, {} bound_adrs, {} bound_specs, 0 findings",
                    report.planning_adrs, report.bound_adrs, report.bound_specs
                );
                ExitCode::SUCCESS
            } else {
                let mode = if blocking { "blocking" } else { "report-only" };
                eprintln!(
                    "planning-ssot-coverage ({mode}): {} finding(s) [{} planning_impact ADRs, {} bound_adrs, {} bound_specs]",
                    report.findings.len(),
                    report.planning_adrs,
                    report.bound_adrs,
                    report.bound_specs
                );
                for finding in &report.findings {
                    eprintln!("  - {finding}");
                }
                if blocking {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        }
        Err(message) => {
            eprintln!("planning-ssot-coverage validation error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_planning_impact_parsed() {
        let text = "---\nid: ADR-0357\nstatus: Proposed\nplanning_impact: true\n---\n# ADR";
        let fm = read_frontmatter(text).expect("frontmatter");
        assert_eq!(frontmatter_value(fm, "planning_impact"), Some("true"));
        assert_eq!(frontmatter_value(fm, "status"), Some("Proposed"));
    }

    #[test]
    fn no_frontmatter_returns_none() {
        assert!(read_frontmatter("# ADR-0217: no frontmatter").is_none());
    }
}
