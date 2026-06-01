//! `oya gate validate adr-lifecycle`.
//!
//! Validates ADR lifecycle invariants over the `docs/decisions/` corpus.
//! Implemented by delegating to `oya_check_adr_index::lifecycle::validate_lifecycle`.
//!
//! # Non-YAML ADR metadata parsing
//!
//! Older ADRs express metadata in one of three inline formats rather than YAML
//! frontmatter:
//!
//! - **Pipe-table**: `| Status | Accepted |` / `| Superseded by | ADR-0105 |`
//! - **List**: `- Status: Accepted` / `- Superseded by: ADR-0105`
//! - **Bold**: `**Status:** Accepted` / `**Superseded by:** ADR-0105`
//!
//! `load_adr_file` detects and parses all three formats, producing a first-class
//! `AdrDecisionRecord` so these ADRs participate in L1/L2/L3 checks. Only files
//! that match none of the three formats are emitted as `AdrParseWarning`.
//!
//! # Corpus membership (FIX 1)
//!
//! `validate_adr_lifecycle_gate` builds a `known_ids` set from **all** ADR-*.md
//! filenames present in the decisions dir and threads it into `validate_lifecycle`.
//! L4 resolves references against this set, so a reference to a table-style ADR
//! is never false-flagged as dangling.
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

// ---------------------------------------------------------------------------
// Inline metadata parsing (non-YAML ADRs)
// ---------------------------------------------------------------------------

/// A parsed key→value pair extracted from one of the three inline-metadata
/// formats used by non-YAML ADRs.
struct InlineKv {
    key: String,
    value: String,
}

/// Extract inline metadata key-value pairs from `contents`.
///
/// Recognises three formats (checked in order per line):
/// 1. Pipe-table row: `| Key | Value |`
/// 2. Bold inline:    `**Key:** Value`  (or `- **Key:** Value`)
/// 3. List item:      `- Key: Value`
///
/// Only lines in the first contiguous block of metadata-looking content are
/// collected (stops at a blank line followed by `##` heading, i.e. the first
/// real section). This prevents false matches from body prose tables.
fn extract_inline_kv(contents: &str) -> Vec<InlineKv> {
    let mut result = Vec::new();
    // Skip the title line (# ADR-NNNN ...) and collect until the first "##"
    // section heading or until a blank line followed by non-metadata content.
    let mut past_title = false;
    let mut blank_streak = 0u32;
    for line in contents.lines() {
        let trimmed = line.trim();
        // Skip the H1 title line.
        if trimmed.starts_with("# ") || trimmed.starts_with("#ADR") {
            past_title = true;
            blank_streak = 0;
            continue;
        }
        // Stop at any H2+ section heading — body has started.
        if trimmed.starts_with("## ") || trimmed.starts_with("### ") {
            break;
        }
        if !past_title {
            continue;
        }
        if trimmed.is_empty() {
            blank_streak += 1;
            // Two consecutive blank lines → body has started.
            if blank_streak >= 2 {
                break;
            }
            continue;
        }
        blank_streak = 0;

        // Format 1: pipe-table row `| Key | Value |`
        if trimmed.starts_with('|') {
            let cells: Vec<&str> = trimmed
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(str::trim)
                .collect();
            if cells.len() >= 2 {
                // Skip separator rows (e.g. `| --- | --- |`).
                if cells[0].chars().all(|c| c == '-' || c == ' ') {
                    continue;
                }
                result.push(InlineKv {
                    key: cells[0].to_string(),
                    value: cells[1..].join("|").trim().to_string(),
                });
            }
            continue;
        }

        // Format 2: bold inline `**Key:** Value` (may be prefixed with `- `).
        let dedashed = trimmed.trim_start_matches('-').trim();
        if dedashed.starts_with("**") {
            if let Some(colon_pos) = dedashed.find(":**") {
                let key = dedashed[2..colon_pos].trim().to_string();
                let value = dedashed[colon_pos + 3..].trim().to_string();
                if !key.is_empty() {
                    result.push(InlineKv { key, value });
                    continue;
                }
            }
        }

        // Format 3: list item `- Key: Value`
        if trimmed.starts_with('-') {
            let rest = trimmed.trim_start_matches('-').trim();
            if let Some(colon_pos) = rest.find(':') {
                let key = rest[..colon_pos].trim().to_string();
                let value = rest[colon_pos + 1..].trim().to_string();
                // Sanity-check: key should look like a metadata key (no spaces
                // beyond two words, no special chars except spaces/hyphens).
                if !key.is_empty() && key.len() <= 30 && !key.contains('(') {
                    result.push(InlineKv { key, value });
                    continue;
                }
            }
        }
    }
    result
}

/// Returns `true` when `contents` has any recognisable inline metadata
/// (pipe-table, bold, or list format with a "Status" key).
fn has_inline_metadata(contents: &str) -> bool {
    extract_inline_kv(contents)
        .iter()
        .any(|kv| kv.key.eq_ignore_ascii_case("status"))
}

/// Normalise a raw metadata value cell to a canonical status string.
///
/// Algorithm:
/// 1. Strip strikethrough regions `~~...~~` (they hold the overridden status).
/// 2. Strip markdown bold markers `**`.
/// 3. Find the **first** canonical status word in the remaining text.
/// 4. Include a `(qualifier)` immediately following it (if any).
///
/// Examples:
/// - `~~Accepted~~ **Superseded**` → `"Superseded"`
/// - `Accepted (amendment)` → `"Accepted (amendment)"`
/// - `Proposed (target: Accepted upon PR merge)` → `"Proposed (target: Accepted upon PR merge)"`
/// - `**Superseded**` → `"Superseded"`
fn normalise_status(raw: &str) -> String {
    // Strip strikethrough regions `~~...~~` (they represent the old/overridden status).
    let without_strike = {
        let mut s = raw.to_string();
        while let Some(start) = s.find("~~") {
            if let Some(end) = s[start + 2..].find("~~") {
                s.replace_range(start..start + 2 + end + 2, "");
            } else {
                break;
            }
        }
        s
    };
    // Strip markdown bold `**...**` wrapping (keep content).
    let without_bold = without_strike.replace("**", "");
    let text = without_bold.trim();

    const CANONICAL: &[&str] = &["Proposed", "Accepted", "Superseded", "Rejected", "Deprecated"];

    // Find the FIRST canonical token in the text. After stripping strikethrough
    // sections the remaining text starts with (or contains) the effective status.
    let mut first: Option<(usize, &str)> = None;
    for &canon in CANONICAL {
        if let Some(pos) = text.find(canon) {
            if first.is_none() || pos < first.unwrap().0 {
                first = Some((pos, canon));
            }
        }
    }

    let Some((pos, canon)) = first else {
        // No canonical token found; return trimmed raw value as-is.
        return text.to_string();
    };

    // Include a qualifier `(...)` immediately following the canonical word.
    let after = text[pos + canon.len()..].trim_start();
    if after.starts_with('(') {
        if let Some(qe) = after.find(')') {
            let qualifier = &after[..qe + 1];
            return format!("{canon} {qualifier}");
        }
    }
    canon.to_string()
}

/// Normalise a "null-equivalent" value (lone dash, em-dash, None, N/A, empty)
/// to an empty string.
fn normalise_null(raw: &str) -> String {
    let t = raw.trim();
    if t == "—" || t == "-" || t.eq_ignore_ascii_case("none") || t.eq_ignore_ascii_case("n/a") || t.is_empty() {
        String::new()
    } else {
        t.to_string()
    }
}

/// Extract `ADR-NNNN` ids from a comma-separated cell that may contain
/// markdown link syntax `[ADR-0105](...)`, strikethrough `~~...~~`, and
/// prose descriptions like `ADR-0064 (canonical-base)`.
fn extract_adr_ids_from_cell(raw: &str) -> Vec<String> {
    // Strip strikethrough regions.
    let mut s = raw.to_string();
    while let Some(start) = s.find("~~") {
        if let Some(end) = s[start + 2..].find("~~") {
            s.replace_range(start..start + 2 + end + 2, "");
        } else {
            break;
        }
    }
    // Strip markdown link targets `[text](url)` → keep text.
    // Replace `[ADR-NNNN](...)` with `ADR-NNNN`.
    let mut result = Vec::new();
    // Scan for ADR-NNNN patterns (4 digits).
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"ADR-") {
            let digits: String = s[i + 4..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if digits.len() == 4 {
                result.push(format!("ADR-{digits}"));
                i += 4 + 4;
                continue;
            }
        }
        i += 1;
    }
    result
}

/// Parse inline (non-YAML) metadata from `contents` into an `AdrDecisionRecord`.
///
/// Returns `None` if no `status` key is found (i.e. not a recognised format).
fn parse_inline_metadata(
    id: &str,
    number: u16,
    file_name: &str,
    contents: &str,
) -> Option<AdrDecisionRecord> {
    let kvs = extract_inline_kv(contents);
    // Must have at least a status key.
    let status_raw = kvs
        .iter()
        .find(|kv| kv.key.eq_ignore_ascii_case("status"))?
        .value
        .clone();

    let status = normalise_status(&status_raw);

    // Extract title from the H1 heading line.
    let title = contents
        .lines()
        .find(|l| l.trim_start().starts_with("# "))
        .map(|l| {
            l.trim_start()
                .trim_start_matches('#')
                .trim()
                // Strip the `ADR-NNNN —` or `ADR-NNNN:` prefix if present.
                .trim_start_matches(id)
                .trim_start_matches(" —")
                .trim_start_matches(':')
                .trim()
                .to_string()
        })
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| format!("{id} (no title)"));

    let owner = kvs
        .iter()
        .find(|kv| kv.key.eq_ignore_ascii_case("owner") || kv.key.eq_ignore_ascii_case("deciders"))
        .map(|kv| kv.value.clone())
        .unwrap_or_else(|| "unknown".into());

    let date = kvs
        .iter()
        .find(|kv| kv.key.eq_ignore_ascii_case("date"))
        .map(|kv| kv.value.clone())
        .unwrap_or_default();

    // Supersedes: may be a null value or a list of ADR-ids.
    let supersedes = kvs
        .iter()
        .find(|kv| kv.key.eq_ignore_ascii_case("supersedes"))
        .map(|kv| {
            let v = normalise_null(&kv.value);
            if v.is_empty() { vec![] } else { extract_adr_ids_from_cell(&v) }
        })
        .unwrap_or_default();

    // Superseded by: look for "Superseded by" (with space) or "superseded_by".
    let superseded_by = kvs
        .iter()
        .find(|kv| {
            kv.key.eq_ignore_ascii_case("superseded by")
                || kv.key.eq_ignore_ascii_case("superseded_by")
        })
        .map(|kv| {
            let v = normalise_null(&kv.value);
            if v.is_empty() { vec![] } else { extract_adr_ids_from_cell(&v) }
        })
        .unwrap_or_default();

    // Related: comma-separated ADR-id list, possibly with prose.
    let related = kvs
        .iter()
        .find(|kv| kv.key.eq_ignore_ascii_case("related") || kv.key.eq_ignore_ascii_case("related adrs"))
        .map(|kv| {
            let v = normalise_null(&kv.value);
            if v.is_empty() { vec![] } else { extract_adr_ids_from_cell(&v) }
        })
        .unwrap_or_default();

    Some(AdrDecisionRecord {
        number,
        id: id.to_string(),
        title,
        status,
        owner,
        date,
        path: format!("decisions/{file_name}"),
        supersedes,
        superseded_by,
        related,
    })
}

// ---------------------------------------------------------------------------
// ADR file loading
// ---------------------------------------------------------------------------

/// Parse a single ADR file, returning either a record + body or a
/// `AdrParseWarning` for completely un-parseable files.
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
        let path_str = format!("decisions/{file_name}");
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

    // No YAML frontmatter — try inline metadata (pipe-table, bold, or list).
    if has_inline_metadata(&contents) {
        if let Some(record) = parse_inline_metadata(&id, number, file_name, &contents) {
            return Ok(Either::Left((record, contents)));
        }
    }

    // Neither YAML frontmatter nor recognisable inline metadata.
    Ok(Either::Right(AdrParseWarning {
        adr_id: id.clone(),
        reason: format!(
            "ADR {id} has neither YAML frontmatter (--- ... ---) nor a recognised inline \
             metadata block (pipe-table / bold / list); it cannot be lifecycle-checked"
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
    let strict = parsed.strict;
    match validate_adr_lifecycle_gate(parsed) {
        Ok(result) => {
            // Print parse warnings (completely unparseable ADRs).
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
            let clean = result.is_clean() && (!strict || result.summary.total_warnings == 0);
            let adr_count = result
                .violations
                .iter()
                .map(|v| &v.adr_id)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                + result.parse_warnings.len();
            println!(
                "adr-lifecycle validation {}: {} ADRs, {} errors, {} warnings, {} unparseable",
                if clean { "passed" } else { "failed" },
                adr_count,
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
    use std::collections::BTreeSet;

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

    // Build the filename-derived known_ids set (FIX 1): includes ALL ADR-*.md
    // files regardless of whether their metadata can be parsed.
    let known_ids: BTreeSet<String> = paths
        .iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| adr_id_from_filename(n))
        })
        .collect();

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
        &known_ids,
    ))
}

// ---------------------------------------------------------------------------
// Unit tests for inline-metadata parsing and normalisation
// ---------------------------------------------------------------------------
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // has_inline_metadata / extract_inline_kv
    // -----------------------------------------------------------------------

    #[test]
    fn detects_pipe_table_status() {
        let contents = "# ADR-0146 — Some title\n\n| Field | Value |\n| --- | --- |\n| Status | Accepted |\n";
        assert!(has_inline_metadata(contents));
    }

    #[test]
    fn detects_list_style_status() {
        let contents = "# ADR-0200 — WASM\n\n- Status: Accepted\n- Date: 2026-05-18\n";
        assert!(has_inline_metadata(contents));
    }

    #[test]
    fn detects_bold_style_status() {
        let contents = "# ADR-0130 — Something\n\n**Status:** Accepted\n**Date:** 2026-05-17\n";
        assert!(has_inline_metadata(contents));
    }

    // -----------------------------------------------------------------------
    // normalise_status
    // -----------------------------------------------------------------------

    #[test]
    fn normalise_status_plain_accepted() {
        assert_eq!(normalise_status("Accepted"), "Accepted");
    }

    #[test]
    fn normalise_status_strikethrough_superseded() {
        // `~~Accepted~~ **Superseded**` -> `Superseded`
        assert_eq!(normalise_status("~~Accepted~~ **Superseded**"), "Superseded");
    }

    #[test]
    fn normalise_status_qualified_accepted() {
        assert_eq!(normalise_status("Accepted (amendment)"), "Accepted (amendment)");
    }

    #[test]
    fn normalise_status_qualified_proposed() {
        // Proposed (target: Accepted ...) -> "Proposed (target: Accepted ...)"
        let raw = "Proposed (target: Accepted upon PR merge)";
        let result = normalise_status(raw);
        assert!(result.starts_with("Proposed"), "got: {result}");
    }

    #[test]
    fn normalise_status_bold_stripped() {
        assert_eq!(normalise_status("**Accepted**"), "Accepted");
    }

    // -----------------------------------------------------------------------
    // parse_inline_metadata — pipe-table format (ADR-0146 style)
    // -----------------------------------------------------------------------

    #[test]
    fn pipe_table_parses_to_record_with_superseded_by() {
        let contents = "\
# ADR-0105 — Some decision

| Field | Value |
| --- | --- |
| Status | ~~Accepted~~ **Superseded** |
| Date | 2026-05-18 |
| Deciders | council-architecture |
| Supersedes | — |
| Superseded by | ADR-0200 |
| Related | ADR-0064 (desc), ADR-0083 (other desc) |
";
        let record = parse_inline_metadata("ADR-0105", 105, "ADR-0105-some-decision.md", contents)
            .expect("should parse");
        assert_eq!(record.id, "ADR-0105");
        assert_eq!(record.status, "Superseded");
        assert!(record.superseded_by.contains(&"ADR-0200".to_string()),
            "superseded_by: {:?}", record.superseded_by);
        assert!(record.supersedes.is_empty(), "supersedes should be empty for —");
        assert!(record.related.contains(&"ADR-0064".to_string()),
            "related: {:?}", record.related);
        assert!(record.related.contains(&"ADR-0083".to_string()),
            "related: {:?}", record.related);
    }

    // -----------------------------------------------------------------------
    // parse_inline_metadata — list format (ADR-0200 style)
    // -----------------------------------------------------------------------

    #[test]
    fn list_style_parses_status_and_related() {
        let contents = "\
# ADR-0200 — WASM runtime canonical: Wasmtime

- Status: Accepted
- Date: 2026-05-18
- Deciders: council-architecture
- Supersedes: none
- Superseded by: none
- Related: ADR-0147 (container sandboxing), ADR-0064 (localization)
";
        let record = parse_inline_metadata("ADR-0200", 200, "ADR-0200-wasm.md", contents)
            .expect("should parse");
        assert_eq!(record.status, "Accepted");
        assert!(record.supersedes.is_empty());
        assert!(record.superseded_by.is_empty());
        assert!(record.related.contains(&"ADR-0147".to_string()),
            "related: {:?}", record.related);
    }

    // -----------------------------------------------------------------------
    // parse_inline_metadata — bold format (ADR-0130 style)
    // -----------------------------------------------------------------------

    #[test]
    fn bold_style_parses_status() {
        let contents = "\
# ADR-0130: Deprecate something

**Status:** Accepted
**Date:** 2026-05-17
**Owner:** council-architecture
**Supersedes:** N/A
";
        let record = parse_inline_metadata("ADR-0130", 130, "ADR-0130-foo.md", contents)
            .expect("should parse");
        assert_eq!(record.status, "Accepted");
        assert!(record.supersedes.is_empty(), "N/A should normalise to empty");
    }

    // -----------------------------------------------------------------------
    // extract_adr_ids_from_cell
    // -----------------------------------------------------------------------

    #[test]
    fn extracts_adr_ids_from_prose_cell() {
        let ids = extract_adr_ids_from_cell(
            "ADR-0064 (canonical-base + localization), ADR-0131 (per-µservice flat layout)"
        );
        assert_eq!(ids, vec!["ADR-0064", "ADR-0131"]);
    }

    #[test]
    fn extracts_adr_ids_with_markdown_links() {
        let ids = extract_adr_ids_from_cell("[ADR-0105](...) and ADR-0146");
        assert!(ids.contains(&"ADR-0105".to_string()));
        assert!(ids.contains(&"ADR-0146".to_string()));
    }

    #[test]
    fn null_values_normalise_to_empty() {
        for s in &["—", "-", "none", "None", "N/A", "n/a", ""] {
            assert_eq!(normalise_null(s), "", "expected empty for {s:?}");
        }
    }
}
