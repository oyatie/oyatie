//! `oya gate validate adr-lifecycle`.
//!
//! Validates ADR lifecycle invariants over the `docs/decisions/` corpus.
//! Implemented by delegating to `oya_check_adr_index::lifecycle::validate_lifecycle`.
//!
//! # Table-style ADR detection
//!
//! Older ADRs (e.g. ADR-0146) express metadata via a markdown TABLE
//! (`| Status | Accepted |`) rather than YAML frontmatter. The frontmatter
//! parser (`read_frontmatter`) returns `None` for these files. Rather than
//! silently passing them (which would let real violations evade L1/L2), this
//! gate detects the table-style pattern and records an explicit
//! `AdrParseWarning` for each, which surfaces as a `Warn`-severity violation
//! in the output.
//!
//! ADR-0083 Tier-3 posture: panic-free — every fallible step returns
//! `Result`/`ExitCode`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::adr_planning_frontmatter::{frontmatter_list, frontmatter_scalar, read_frontmatter};
use oya_check_adr_index::lifecycle::{
    validate_lifecycle, AdrParseWarning, LifecycleRule, LifecycleResult, Severity,
};
use oya_check_adr_index::AdrDecisionRecord;

// ---------------------------------------------------------------------------
// Args
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdrLifecycleArgs {
    pub(crate) decisions_dir: PathBuf,
    /// When true, Warn-severity violations also cause a non-zero exit code.
    pub(crate) strict: bool,
}

pub(crate) fn parse_adr_lifecycle_args(
    args: Vec<String>,
) -> Result<AdrLifecycleArgs, String> {
    let mut parsed = AdrLifecycleArgs {
        decisions_dir: PathBuf::from("docs/decisions"),
        strict: false,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--decisions-dir requires a value".to_string())?,
                );
            }
            "--strict" => {
                parsed.strict = true;
            }
            other => {
                return Err(format!(
                    "adr-lifecycle: unknown flag {other:?}; allowed: --decisions-dir --strict"
                ));
            }
        }
    }
    Ok(parsed)
}

// ---------------------------------------------------------------------------
// ADR file loading
// ---------------------------------------------------------------------------

/// Extract an `ADR-NNNN` id from a filename, or `None`.
fn adr_id_from_filename(name: &str) -> Option<String> {
    if !name.starts_with("ADR-") || !name.ends_with(".md") {
        return None;
    }
    let digits: String = name[4..].chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.len() != 4 {
        return None;
    }
    Some(format!("ADR-{digits}"))
}

/// Detect whether a file's body contains a markdown-table-style metadata block
/// (`| Status | <value> |`) rather than YAML frontmatter.
fn has_table_style_metadata(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim();
        // Look for a row with "Status" as the first cell.
        if !trimmed.starts_with('|') {
            return false;
        }
        let cells: Vec<&str> = trimmed
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        cells.len() >= 2 && cells[0].eq_ignore_ascii_case("status")
    })
}

/// Parse a single ADR file, returning either a record + body or a
/// `AdrParseWarning` for table-style/un-parseable files.
fn load_adr_file(
    path: &Path,
) -> Result<Either<(AdrDecisionRecord, String), AdrParseWarning>, String> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let Some(id) = adr_id_from_filename(file_name) else {
        return Err(format!("Cannot extract ADR id from filename: {}", path.display()));
    };
    let number: u16 = id[4..].parse().map_err(|e| format!("Bad ADR number in {id}: {e}"))?;
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;

    // Try YAML frontmatter first.
    if let Some(fm) = read_frontmatter(&contents) {
        let title = frontmatter_scalar(fm, "title").unwrap_or_else(|| format!("{id} (no title)"));
        let status = frontmatter_scalar(fm, "status").unwrap_or_default();
        let owner = frontmatter_scalar(fm, "owner").unwrap_or_else(|| "unknown".into());
        let date = frontmatter_scalar(fm, "date").unwrap_or_default();
        let supersedes = frontmatter_list(fm, "supersedes");
        let superseded_by = frontmatter_list(fm, "superseded_by");
        let related = frontmatter_list(fm, "related");
        let path_str = format!(
            "decisions/{file_name}"
        );
        let record = AdrDecisionRecord {
            number,
            id,
            title,
            status,
            owner,
            date,
            path: path_str,
            supersedes,
            superseded_by,
            related,
        };
        return Ok(Either::Left((record, contents)));
    }

    // No YAML frontmatter. Check if it's table-style.
    if has_table_style_metadata(&contents) {
        return Ok(Either::Right(AdrParseWarning::table_style(id)));
    }

    // Neither YAML frontmatter nor table-style — emit a generic parse warning.
    Ok(Either::Right(AdrParseWarning {
        adr_id: id.clone(),
        reason: format!(
            "ADR {id} has neither YAML frontmatter (--- ... ---) nor a markdown metadata table; \
             it cannot be lifecycle-checked"
        ),
    }))
}

/// Minimal Either type to avoid a dependency.
enum Either<L, R> {
    Left(L),
    Right(R),
}

// ---------------------------------------------------------------------------
// Gate entry point
// ---------------------------------------------------------------------------

pub(crate) fn run_adr_lifecycle(args: Vec<String>) -> ExitCode {
    let parsed = match parse_adr_lifecycle_args(args) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    match validate_adr_lifecycle_gate(parsed) {
        Ok(result) => {
            // Print parse warnings (table-style ADRs etc.).
            for pw in &result.parse_warnings {
                eprintln!(
                    "adr-lifecycle WARN [{}]: {}",
                    pw.adr_id, pw.reason
                );
            }
            // Print violations.
            for v in &result.violations {
                // Skip parse-warning re-emissions (already printed above).
                if v.rule == LifecycleRule::L1StatusVocab
                    && result
                        .parse_warnings
                        .iter()
                        .any(|pw| pw.adr_id == v.adr_id)
                {
                    continue;
                }
                let prefix = match v.severity {
                    Severity::Error => "FAIL",
                    Severity::Warn => "WARN",
                };
                eprintln!(
                    "adr-lifecycle {} [{}] {}: {}",
                    prefix,
                    v.adr_id,
                    v.rule.as_str(),
                    v.detail
                );
                if let Some(fix) = &v.suggested_fix {
                    eprintln!("  fix: {fix}");
                }
            }
            let clean = result.is_clean();
            println!(
                "adr-lifecycle validation {}: {} ADRs, {} errors, {} warnings, {} table-style",
                if clean { "passed" } else { "failed" },
                result.violations.iter().map(|v| &v.adr_id).collect::<std::collections::BTreeSet<_>>().len()
                    + result.parse_warnings.len(),
                result.summary.total_errors,
                result.summary.total_warnings,
                result.parse_warnings.len(),
            );
            if clean {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(msg) => {
            eprintln!("adr-lifecycle validation error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn validate_adr_lifecycle_gate(
    args: AdrLifecycleArgs,
) -> Result<LifecycleResult, String> {
    let dir = &args.decisions_dir;
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Cannot read decisions dir {}: {e}", dir.display()))?;

    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Dir entry error: {e}"))?;
        let path = entry.path();
        let is_adr = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("ADR-") && n.ends_with(".md"));
        if is_adr {
            paths.push(path);
        }
    }
    paths.sort();

    if paths.is_empty() {
        return Err(format!(
            "No ADR-*.md files found in {}",
            dir.display()
        ));
    }

    let mut records: Vec<AdrDecisionRecord> = Vec::new();
    let mut bodies: BTreeMap<String, String> = BTreeMap::new();
    let mut parse_warnings: Vec<AdrParseWarning> = Vec::new();

    for path in &paths {
        match load_adr_file(path) {
            Ok(Either::Left((record, body))) => {
                bodies.insert(record.id.clone(), body);
                records.push(record);
            }
            Ok(Either::Right(pw)) => {
                parse_warnings.push(pw);
            }
            Err(msg) => {
                // IO errors are hard failures.
                return Err(msg);
            }
        }
    }

    // Build the bodies ref-map for validate_lifecycle.
    let bodies_ref: BTreeMap<String, &str> = bodies
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str()))
        .collect();

    Ok(validate_lifecycle(
        records.iter(),
        &bodies_ref,
        &parse_warnings,
    ))
}
