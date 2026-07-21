//! Doc-axis enforcement kernel (ADR-0388).
//!
//! Validates that the `docs/` tree and related registry artifacts stay within
//! the seven canonical doc axes defined by ADR-0388. Five rules are checked:
//!
//! 1. **ADR status casing** — every `docs/decisions/ADR-*.md` frontmatter
//!    `status:` value is one of the five allowed literals (case-sensitive).
//!    Emits a warning (not an error) unless `strict` mode is enabled, because
//!    the existing corpus of ~295 ADRs has inconsistent historical casing. A
//!    follow-up normalisation sweep will promote this to an error.
//!
//! 2. **No shadow docs** — `docs/ideas/*.md` files whose filename date-stamp
//!    is older than 14 days must be either archived (`docs/ideas/archive/`) or
//!    carry a `superseded_by: ADR-NNNN` frontmatter field linking to a real
//!    existing ADR file.
//!
//! 3. **No docs proliferation** — only the canonical subdirectories are
//!    permitted under `docs/`. Any `.md` file placed outside them is an error.
//!
//! 4. **Catalog/manifest crate-claim consistency** — every crate listed in a
//!    microservice `manifest.json` `bounded_contexts[].crates[]` array must
//!    have a corresponding file under `registry/catalog/<crate>.yaml`.
//!
//! 5. **No retired roadmap authority reads** — active docs and templates must
//!    point at `/specs/masterplan.json#masterplan_v2`, not `docs/ROADMAP.md`.

#![forbid(unsafe_code)]
// ADR-0083 Tier 1 (kernel): no unwrap/expect/panic in non-test code.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A structured validation finding returned by this gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocAxisFinding {
    /// Repo-relative path of the offending file. data_class: INTERNAL_ONLY
    pub path: String,
    /// 1-based line number within the file, if applicable. data_class: INTERNAL_ONLY
    pub line: Option<usize>,
    /// Rule identifier that was violated. data_class: INTERNAL_ONLY
    pub rule_violated: DocAxisRule,
    /// Human-readable suggestion for how to fix the violation. data_class: INTERNAL_ONLY
    pub suggested_fix: String,
    /// Whether this finding blocks the gate (`false` = warning only). data_class: INTERNAL_ONLY
    pub blocking: bool,
}

/// The five doc-axis rule identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocAxisRule {
    /// Rule 1: ADR status field must be one of the five canonical values.
    AdrStatusCasing,
    /// Rule 2: idea-pager older than 14 days without promotion or archival.
    ShadowIdea,
    /// Rule 3: markdown file placed outside a canonical `docs/` subdirectory.
    DocsProliferation,
    /// Rule 4: crate claimed in manifest.json but missing from registry/catalog/.
    CatalogManifestDrift,
    /// Rule 5: active documentation or templates read the retired roadmap.
    RetiredRoadmapAuthority,
}

/// Summary counters returned on a clean (no blocking violations) run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocAxisReport {
    /// `docs/decisions/ADR-*.md` files inspected. data_class: INTERNAL_ONLY
    pub adrs_checked: usize,
    /// `docs/ideas/*.md` files inspected (excluding archive/). data_class: INTERNAL_ONLY
    pub ideas_checked: usize,
    /// Canonical subdirectory check: markdown files inspected under `docs/`. data_class: INTERNAL_ONLY
    pub docs_files_checked: usize,
    /// Microservice manifest files inspected. data_class: INTERNAL_ONLY
    pub manifests_checked: usize,
    /// Non-blocking warnings emitted (e.g. ADR casing in non-strict mode). data_class: INTERNAL_ONLY
    pub warnings: usize,
}

/// Validation result: either a clean report (with zero blocking findings), or
/// a non-empty list of findings where at least one is blocking.
pub type ValidationResult = Result<DocAxisReport, Vec<DocAxisFinding>>;

// ---------------------------------------------------------------------------
// Allowed canonical status values (Rule 1)
// ---------------------------------------------------------------------------

/// The five canonical ADR `status:` values (exact case, per ADR-0388).
pub const ALLOWED_ADR_STATUSES: &[&str] = &[
    "Accepted",
    "Proposed",
    "Superseded",
    "Deprecated",
    "Rejected",
];

// ---------------------------------------------------------------------------
// Allowed `docs/` subdirectories (Rule 3)
// ---------------------------------------------------------------------------

/// Top-level subdirectory names that are canonical under `docs/` (ADR-0388).
pub const CANONICAL_DOCS_SUBDIRS: &[&str] = &[
    "decisions",
    "ideas",
    "conventions",
    "machine-readable",
    "products",
    "site",
];

/// Transitional root-level markdown files that predate ADR-0388. They remain
/// allowed so the gate can ratchet against new proliferation without forcing a
/// broad documentation move in unrelated PRs. New root-level docs stay blocked
/// unless a future ADR deliberately promotes them or moves them under a
/// canonical axis.
pub const LEGACY_DOCS_ROOT_FILES: &[&str] = &[
    "ADR-CONSOLIDATION-PLAN.md",
    "ADR-INDEX.md",
    "ADR-LEGACY-REGRESSION-MAPPING.md",
    "AGENT-INSTRUCTION-SOURCES.md",
    "AGENTS-OPERATING-CONTRACT.md",
    "AGENTS.md",
    "CHANGELOG.md",
    "COMPETITIVE-GAP-ANALYSIS.md",
    "COMPLIANCE-MATRIX.md",
    "CONTRADICTION-LEDGER.md",
    "DESIGN.md",
    "DOC-CATALOG.md",
    "DOC-COVERAGE.md",
    "DOC-UPDATE-PROTOCOL.md",
    "DOCUMENTATION.md",
    "FINOPS-PLAN.md",
    "GLOSSARY.md",
    "GTM-PLAN.md",
    "HIRING-CAPACITY-PLAN.md",
    "INCIDENT-MANAGEMENT.md",
    "INTERNATIONALIZATION.md",
    "LEGAL-IP-LEDGER.md",
    "MASTERPLAN.md",
    "MISTAKES-LEDGER.md",
    "PRD-OYATIE-FROM-SCRATCH-CANONICAL.md",
    "PRD.md",
    "PRIVACY-PROGRAM.md",
    "QA-TEST-STRATEGY.md",
    "RACI-OWNERSHIP.md",
    "README.md",
    "RELEASE-MANAGEMENT.md",
    "RISK-REGISTER.md",
    "ROADMAP.md",
    "RUNBOOKS-INDEX.md",
    "SLO-CATALOG.md",
    "SPEC.md",
    "STANDARDS-AND-TEMPLATES.md",
    "TOOLCHAIN.md",
    "VENDOR-PARTNER-LEDGER.md",
    "bootstrap.md",
];

/// Transitional top-level `docs/` directories that already exist in the
/// repository. They are not new axes; they are an allowlist that lets the gate
/// block new unreviewed directories while a separate documentation-retirement
/// wave moves legacy content under canonical ADR-0388 axes.
pub const LEGACY_DOCS_SUBDIRS: &[&str] = &[
    "advanced-cicd",
    "agents",
    "api",
    "architecture",
    "audits",
    "automation",
    "checklists",
    "customer-success",
    "foundry",
    "governance",
    "governance-lanes",
    "gtm",
    "investor",
    "localization-packs",
    "onboarding",
    "operators",
    "performance-budgets",
    "personas",
    "plans",
    "policies",
    "prds",
    "quality",
    "raw",
    "regional-packs",
    "release",
    "research",
    "runbooks",
    "specs",
    "standards",
    "teams",
    "templates",
    "tutorials",
    "user-journeys",
    "user-stories",
    "wiki",
];

/// Active document and template surfaces that formerly read `docs/ROADMAP.md`.
///
/// Historical ADRs, audits, evidence, and negative fixtures intentionally do
/// not appear here: they may cite the retired surface as provenance.
pub const ACTIVE_ROADMAP_AUTHORITY_PATHS: &[&str] = &[
    "docs/DESIGN.md",
    "docs/DOCUMENTATION.md",
    "docs/PRD.md",
    "docs/automation/product-map-spec.md",
    "docs/automation/roadmap-visualization-spec.md",
    "docs/checklists/release-readiness-checklist.md",
    "docs/prds/hr.md",
    "docs/prds/payroll.md",
    "docs/templates/milestone-index-template.md",
    "docs/templates/prd-template.md",
    "docs/templates/prfaq-template.md",
    "templates/checklists/release-readiness-checklist.md",
    "templates/milestone-index-template.md",
    "templates/prd-template.md",
    "templates/prfaq-template.md",
];

// ---------------------------------------------------------------------------
// Idea-pager promotion window
// ---------------------------------------------------------------------------

/// Number of days an idea-pager may remain without promotion or archival.
pub const IDEA_PROMOTION_DAYS: i64 = 14;

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Validate the doc-axis conventions for the repository rooted at `repo_root`.
///
/// Pass `strict = true` to promote ADR-status-casing findings from warnings to
/// blocking errors (needed once the normalisation sweep lands).
///
/// # Errors
/// Returns `Err(findings)` where at least one finding is blocking. Warnings
/// are surfaced inside the `Ok(report)` path via `report.warnings`.
pub fn validate(repo_root: &Path, strict: bool) -> ValidationResult {
    let mut findings: Vec<DocAxisFinding> = Vec::new();

    let mut report = DocAxisReport {
        adrs_checked: 0,
        ideas_checked: 0,
        docs_files_checked: 0,
        manifests_checked: 0,
        warnings: 0,
    };

    check_adr_status_casing(repo_root, strict, &mut findings, &mut report);
    check_shadow_ideas(repo_root, &mut findings, &mut report);
    check_docs_proliferation(repo_root, &mut findings, &mut report);
    check_retired_roadmap_authority(repo_root, &mut findings);
    check_catalog_manifest_drift(repo_root, &mut findings, &mut report);

    // Count warnings from non-blocking findings.
    report.warnings = findings.iter().filter(|f| !f.blocking).count();

    let blocking: Vec<DocAxisFinding> = findings.into_iter().filter(|f| f.blocking).collect();
    if blocking.is_empty() {
        Ok(report)
    } else {
        Err(blocking)
    }
}

// ---------------------------------------------------------------------------
// Rule 5 — No retired roadmap authority reads
// ---------------------------------------------------------------------------

fn check_retired_roadmap_authority(repo_root: &Path, findings: &mut Vec<DocAxisFinding>) {
    for rel in ACTIVE_ROADMAP_AUTHORITY_PATHS {
        let path = repo_root.join(rel);
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(_) => continue,
        };

        for (index, line) in contents.lines().enumerate() {
            if contains_retired_roadmap_reference(line) {
                findings.push(DocAxisFinding {
                    path: (*rel).to_string(),
                    line: Some(index + 1),
                    rule_violated: DocAxisRule::RetiredRoadmapAuthority,
                    suggested_fix: "Replace the current-authority/read reference with `/specs/masterplan.json#masterplan_v2`; use a narrower masterplan_v2 field only when it truthfully preserves the referenced semantics. Historical ADR, audit, evidence, and negative-fixture citations remain allowed outside this active-path set.".to_string(),
                    blocking: true,
                });
            }
        }
    }
}

fn contains_retired_roadmap_reference(line: &str) -> bool {
    const RETIRED_NAME: &str = "ROADMAP.md";

    line.match_indices(RETIRED_NAME).any(|(start, _)| {
        let before = line[..start].chars().next_back();
        let after = line[start + RETIRED_NAME.len()..].chars().next();
        !before.is_some_and(is_path_name_character) && !after.is_some_and(is_path_name_character)
    })
}

fn is_path_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
}

// ---------------------------------------------------------------------------
// Rule 1 — ADR status casing
// ---------------------------------------------------------------------------

fn check_adr_status_casing(
    repo_root: &Path,
    strict: bool,
    findings: &mut Vec<DocAxisFinding>,
    report: &mut DocAxisReport,
) {
    let decisions_dir = repo_root.join("docs").join("decisions");
    let entries = match fs::read_dir(&decisions_dir) {
        Ok(e) => e,
        Err(_) => return, // directory absent — nothing to check
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !file_name.starts_with("ADR-") || !file_name.ends_with(".md") {
            continue;
        }
        report.adrs_checked += 1;
        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = repo_relative(repo_root, &path);
        if let Some((line_num, bad_value)) = find_bad_status(&contents) {
            findings.push(DocAxisFinding {
                path: rel,
                line: Some(line_num),
                rule_violated: DocAxisRule::AdrStatusCasing,
                suggested_fix: format!(
                    "Change `status: {bad_value}` to one of: {}",
                    ALLOWED_ADR_STATUSES.join(", ")
                ),
                blocking: strict,
            });
        }
    }
}

/// Scan `contents` for a `status:` frontmatter line with a non-canonical value.
/// Returns `(1-based line number, bad value string)` on the first violation found.
fn find_bad_status(contents: &str) -> Option<(usize, String)> {
    let mut in_frontmatter = false;
    let mut frontmatter_opened = false;
    for (idx, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if idx == 0 && trimmed == "---" {
            in_frontmatter = true;
            frontmatter_opened = true;
            continue;
        }
        if frontmatter_opened && trimmed == "---" && idx > 0 {
            break; // end of frontmatter
        }
        if !in_frontmatter {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("status:") {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !ALLOWED_ADR_STATUSES.contains(&value) {
                return Some((idx + 1, value.to_string()));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Rule 2 — No shadow ideas
// ---------------------------------------------------------------------------

fn check_shadow_ideas(
    repo_root: &Path,
    findings: &mut Vec<DocAxisFinding>,
    report: &mut DocAxisReport,
) {
    let ideas_dir = repo_root.join("docs").join("ideas");
    let archive_dir = ideas_dir.join("archive");
    let entries = match fs::read_dir(&ideas_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    // Collect the set of ADR filenames that exist under docs/decisions/ for
    // cross-referencing `superseded_by` claims.
    let existing_adrs = collect_existing_adr_ids(repo_root);

    for entry in entries.flatten() {
        let path = entry.path();
        if path.starts_with(&archive_dir) {
            continue;
        }
        if path.is_dir() {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if !file_name.ends_with(".md") {
            continue;
        }
        report.ideas_checked += 1;

        // Parse date from filename pattern `<topic>-YYYY-MM-DD.md`.
        let date_stamp = extract_date_from_idea_filename(&file_name);
        let age_days = date_stamp
            .as_deref()
            .and_then(parse_ymd_to_days_ago)
            .unwrap_or(0);

        if age_days <= IDEA_PROMOTION_DAYS {
            continue; // still within the promotion window
        }

        // Over 14 days: must have superseded_by in frontmatter pointing to a
        // real ADR, OR be moved to archive (archive check is implicit — files
        // in archive/ are skipped above).
        let contents = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = repo_relative(repo_root, &path);

        if let Some(adr_id) = find_superseded_by(&contents) {
            // Verify the cited ADR actually exists.
            if !existing_adrs.contains(&adr_id) {
                findings.push(DocAxisFinding {
                    path: rel,
                    line: None,
                    rule_violated: DocAxisRule::ShadowIdea,
                    suggested_fix: format!(
                        "`superseded_by: {adr_id}` references an ADR that does not exist in docs/decisions/. Create the ADR or fix the reference."
                    ),
                    blocking: true,
                });
            }
            // Otherwise the citation is valid — file is promoted.
        } else {
            findings.push(DocAxisFinding {
                path: rel,
                line: None,
                rule_violated: DocAxisRule::ShadowIdea,
                suggested_fix: format!(
                    "Idea-pager is {age_days} days old (limit: {IDEA_PROMOTION_DAYS}). Promote to an ADR and add `superseded_by: ADR-NNNN`, or move to docs/ideas/archive/."
                ),
                blocking: true,
            });
        }
    }
}

/// Extract the `YYYY-MM-DD` portion from an idea filename like
/// `my-topic-2026-05-28.md`. Returns `None` if no date is present.
fn extract_date_from_idea_filename(file_name: &str) -> Option<String> {
    // Strip `.md` suffix, then look for the trailing `-YYYY-MM-DD` segment.
    let stem = file_name.strip_suffix(".md")?;
    // The last three dash-separated tokens form the date.
    let parts: Vec<&str> = stem.rsplitn(4, '-').collect();
    if parts.len() < 3 {
        return None;
    }
    // rsplitn order: day, month, year, rest
    let day = parts[0];
    let month = parts[1];
    let year = parts[2];
    if year.len() == 4 && month.len() == 2 && day.len() == 2 {
        Some(format!("{year}-{month}-{day}"))
    } else {
        None
    }
}

/// Parse `YYYY-MM-DD` and return how many days ago that date was relative to
/// "today". Uses a simple integer arithmetic approach against a fixed
/// reference — in tests this is anchored to 2026-05-28. Returns `None` if
/// the date cannot be parsed.
///
/// The reference date is read from the `OYA_TODAY` environment variable
/// (format `YYYY-MM-DD`) so tests can override it without `SystemTime`.
fn parse_ymd_to_days_ago(ymd: &str) -> Option<i64> {
    let today_str = std::env::var("OYA_TODAY").unwrap_or_else(|_| today_ymd_from_system());
    days_between(&today_str, ymd)
}

/// Returns today's date in `YYYY-MM-DD` using `SystemTime`.
fn today_ymd_from_system() -> String {
    // Convert UNIX seconds to a calendar date using the proleptic Gregorian
    // calendar. Pure stdlib, no chrono dependency.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    unix_secs_to_ymd(secs)
}

/// Convert a UNIX timestamp (seconds since epoch) to `YYYY-MM-DD`.
fn unix_secs_to_ymd(secs: u64) -> String {
    // Days since epoch.
    let days = secs / 86400;
    civil_date_from_days(days as i64)
}

/// Civil (proleptic Gregorian) date from days-since-unix-epoch (1970-01-01).
/// Algorithm: Fliegel–Van Flandern via Richards (2013).
fn civil_date_from_days(z: i64) -> String {
    // Shift epoch to 1 Mar 0000 for easier arithmetic.
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// Returns the number of days between `today` and `file_date` (both `YYYY-MM-DD`).
/// Positive means the file date is in the past.
fn days_between(today: &str, file_date: &str) -> Option<i64> {
    let t = ymd_to_days(today)?;
    let f = ymd_to_days(file_date)?;
    Some(t - f)
}

/// Convert `YYYY-MM-DD` to days since an arbitrary epoch (Julian Day Number).
fn ymd_to_days(ymd: &str) -> Option<i64> {
    let mut parts = ymd.splitn(3, '-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    // Rata Die (days since 0001-01-01) — simple, enough for age comparisons.
    let adj_y = if m <= 2 { y - 1 } else { y };
    let adj_m = if m <= 2 { m + 12 } else { m };
    Some(365 * adj_y + adj_y / 4 - adj_y / 100 + adj_y / 400 + (153 * adj_m + 8) / 5 + d - 428)
}

/// Look for `superseded_by:` in frontmatter. Returns the ADR id (e.g. `ADR-0388`).
fn find_superseded_by(contents: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut opened = false;
    for (idx, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if idx == 0 && trimmed == "---" {
            in_frontmatter = true;
            opened = true;
            continue;
        }
        if opened && trimmed == "---" && idx > 0 {
            break;
        }
        if !in_frontmatter {
            break;
        }
        if let Some(rest) = trimmed.strip_prefix("superseded_by:") {
            let value = rest.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() && value != "[]" && value != "null" {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Collect all ADR ids (e.g. `ADR-0388`) that have a corresponding file in
/// `docs/decisions/`.
fn collect_existing_adr_ids(repo_root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let decisions_dir = repo_root.join("docs").join("decisions");
    let entries = match fs::read_dir(&decisions_dir) {
        Ok(e) => e,
        Err(_) => return ids,
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        // Accept filenames like ADR-0388-*.md — extract the ADR-NNNN prefix.
        if name.starts_with("ADR-") && name.ends_with(".md") {
            let parts: Vec<&str> = name.splitn(3, '-').collect();
            if parts.len() >= 2 {
                ids.insert(format!("{}-{}", parts[0], parts[1]));
            }
        }
    }
    ids
}

// ---------------------------------------------------------------------------
// Rule 3 — No docs proliferation
// ---------------------------------------------------------------------------

fn check_docs_proliferation(
    repo_root: &Path,
    findings: &mut Vec<DocAxisFinding>,
    report: &mut DocAxisReport,
) {
    let docs_dir = repo_root.join("docs");
    let entries = match fs::read_dir(&docs_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if path.is_file() && name.ends_with(".md") {
            if LEGACY_DOCS_ROOT_FILES.contains(&name.as_str()) {
                continue;
            }
            // A markdown file directly under docs/ (not in any subdir).
            report.docs_files_checked += 1;
            let rel = repo_relative(repo_root, &path);
            findings.push(DocAxisFinding {
                path: rel,
                line: None,
                rule_violated: DocAxisRule::DocsProliferation,
                suggested_fix: format!(
                    "Move `{name}` into a canonical subdirectory: {}",
                    CANONICAL_DOCS_SUBDIRS.join(", ")
                ),
                blocking: true,
            });
        } else if path.is_dir()
            && !CANONICAL_DOCS_SUBDIRS.contains(&name.as_str())
            && !LEGACY_DOCS_SUBDIRS.contains(&name.as_str())
        {
            // A directory that is not one of the canonical subdirectories.
            report.docs_files_checked += 1;
            let rel = repo_relative(repo_root, &path);
            findings.push(DocAxisFinding {
                path: rel,
                line: None,
                rule_violated: DocAxisRule::DocsProliferation,
                suggested_fix: format!(
                    "Directory `docs/{name}/` is not a canonical doc axis. Allowed: {}. To add a new axis, amend ADR-0388.",
                    CANONICAL_DOCS_SUBDIRS.join(", ")
                ),
                blocking: true,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Rule 4 — Catalog/manifest crate-claim consistency
// ---------------------------------------------------------------------------

fn check_catalog_manifest_drift(
    repo_root: &Path,
    findings: &mut Vec<DocAxisFinding>,
    report: &mut DocAxisReport,
) {
    let ms_dir = repo_root.join("microservices");
    let entries = match fs::read_dir(&ms_dir) {
        Ok(e) => e,
        Err(_) => return, // microservices/ absent — nothing to check
    };

    for entry in entries.flatten() {
        let ms_path = entry.path();
        if !ms_path.is_dir() {
            continue;
        }
        let manifest_path = ms_path.join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        report.manifests_checked += 1;

        let contents = match fs::read_to_string(&manifest_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let crates = extract_crates_from_manifest(&contents);
        for crate_name in crates {
            let catalog_file = repo_root
                .join("registry")
                .join("catalog")
                .join(format!("{crate_name}.yaml"));
            if !catalog_file.exists() {
                let rel = repo_relative(repo_root, &manifest_path);
                findings.push(DocAxisFinding {
                    path: rel,
                    line: None,
                    rule_violated: DocAxisRule::CatalogManifestDrift,
                    suggested_fix: format!(
                        "Crate `{crate_name}` is listed in manifest.json but `registry/catalog/{crate_name}.yaml` does not exist. Add the catalog entry or remove the crate from the manifest."
                    ),
                    blocking: true,
                });
            }
        }
    }
}

/// Extract crate names from a manifest.json `bounded_contexts[].crates[]`
/// structure using simple text scanning (no JSON parser dependency).
///
/// Looks for the pattern `"crates": [...]` and extracts each quoted string.
fn extract_crates_from_manifest(contents: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut in_crates_array = false;
    let mut depth = 0i32;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.contains("\"crates\"") && trimmed.contains('[') {
            in_crates_array = true;
            depth = 0;
        }
        if in_crates_array {
            for ch in trimmed.chars() {
                match ch {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth <= 0 {
                            in_crates_array = false;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            // Extract quoted identifiers.
            let mut s = trimmed;
            while let Some(start) = s.find('"') {
                s = &s[start + 1..];
                if let Some(end) = s.find('"') {
                    let token = &s[..end];
                    // Only accept crate-name shaped tokens (starts with oya-).
                    if token.starts_with("oya-") {
                        results.push(token.to_string());
                    }
                    s = &s[end + 1..];
                } else {
                    break;
                }
            }
        }
    }
    results
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the repo-relative path string for a given absolute path.
fn repo_relative(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

// ---------------------------------------------------------------------------
// Unit tests (kernel-level, no external I/O)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_statuses_are_canonical() {
        assert!(ALLOWED_ADR_STATUSES.contains(&"Accepted"));
        assert!(ALLOWED_ADR_STATUSES.contains(&"Proposed"));
        assert!(ALLOWED_ADR_STATUSES.contains(&"Superseded"));
        assert!(ALLOWED_ADR_STATUSES.contains(&"Deprecated"));
        assert!(ALLOWED_ADR_STATUSES.contains(&"Rejected"));
        assert_eq!(ALLOWED_ADR_STATUSES.len(), 5);
    }

    #[test]
    fn find_bad_status_detects_lowercase() {
        let doc = "---\nid: ADR-0001\nstatus: accepted\n---\n# body\n";
        let result = find_bad_status(doc);
        assert!(result.is_some());
        let (line, val) = result.unwrap();
        assert_eq!(val, "accepted");
        assert_eq!(line, 3);
    }

    #[test]
    fn find_bad_status_passes_canonical_values() {
        for status in ALLOWED_ADR_STATUSES {
            let doc = format!("---\nid: ADR-0001\nstatus: {status}\n---\n# body\n");
            assert_eq!(find_bad_status(&doc), None, "status {status} should pass");
        }
    }

    #[test]
    fn extract_date_from_filename_parses_correctly() {
        assert_eq!(
            extract_date_from_idea_filename("my-idea-2026-05-28.md"),
            Some("2026-05-28".to_string())
        );
        assert_eq!(
            extract_date_from_idea_filename("cloud-intelligence-v1-pipeline-2026-05-28.md"),
            Some("2026-05-28".to_string())
        );
        assert_eq!(extract_date_from_idea_filename("no-date.md"), None);
    }

    #[test]
    fn days_between_computes_age() {
        // 2026-05-28 minus 2026-05-14 = 14 days.
        assert_eq!(days_between("2026-05-28", "2026-05-14"), Some(14));
        // Same day = 0.
        assert_eq!(days_between("2026-05-28", "2026-05-28"), Some(0));
        // 15 days ago.
        assert_eq!(days_between("2026-05-28", "2026-05-13"), Some(15));
    }

    #[test]
    fn find_superseded_by_parses_adr_id() {
        let doc = "---\nsuperseded_by: ADR-0388\n---\n# body\n";
        assert_eq!(find_superseded_by(doc), Some("ADR-0388".to_string()));
    }

    #[test]
    fn find_superseded_by_returns_none_when_absent() {
        let doc = "---\nid: ADR-0001\nstatus: Accepted\n---\n# body\n";
        assert_eq!(find_superseded_by(doc), None);
    }

    #[test]
    fn extract_crates_from_manifest_finds_oya_crates() {
        let manifest = r#"{
  "bounded_contexts": [
    {
      "name": "core",
      "crates": ["oya-check-doc-axis", "oya-check-no-grouping"]
    }
  ]
}"#;
        let crates = extract_crates_from_manifest(manifest);
        assert!(crates.contains(&"oya-check-doc-axis".to_string()));
        assert!(crates.contains(&"oya-check-no-grouping".to_string()));
    }

    #[test]
    fn civil_date_from_days_known_values() {
        // 1970-01-01 is day 0.
        assert_eq!(civil_date_from_days(0), "1970-01-01");
        // 2026-05-28: days since epoch.
        // 2026-05-28 = 20602 days after 1970-01-01 (approximate; test confirms round-trip).
        let ymd = "2026-05-28";
        assert!(ymd_to_days(ymd).is_some());
        assert_eq!(days_between(ymd, ymd), Some(0));
    }
}
