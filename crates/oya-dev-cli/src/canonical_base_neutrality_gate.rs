//! `oya gate validate canonical-base-neutrality` runner.
//!
//! ADR-0064 requires a mechanical lane that proves canonical base
//! crates do not hardcode jurisdiction-specific identifiers or string
//! literals. This runner intentionally stays dependency-free: it walks
//! Rust source files, tokenizes identifiers and string literals with a
//! small local scanner, and reports every jurisdiction leak at once.
//!
//! The default scan is restricted to likely canonical-base workspace
//! crates (`*-kernel`, `*-domain`, `*-usecase`) and excludes pack,
//! regional, adapter, API, CLI, check, fitness, and fixture/test paths.
//! Callers can override roots with `--root <path>` for targeted proof or
//! failure reproduction. The command is not wired into `run-all` yet; it
//! is a truth surface for FD-001 boundary evidence before it becomes a
//! required hosted context.

use std::fs;
use std::path::{Path, PathBuf};

const USAGE: &str = "oya gate validate canonical-base-neutrality \
                     [--repo-root <.>] \
                     [--root <path>] (repeatable) \
                     [--exclude-root <path>] (repeatable) \
                     [--self-test]";

const JURISDICTION_TOKENS: &[&str] = &["Kr", "Us", "Eu", "Jp", "Sea", "Mena", "Kcmvp"];
const JURISDICTION_CODES: &[&str] = &["kr", "us", "eu", "jp", "sea", "mena", "kcmvp"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalBaseNeutralityValidateArgs {
    pub repo_root: PathBuf,
    pub roots: Vec<PathBuf>,
    pub exclude_roots: Vec<PathBuf>,
    pub self_test: bool,
    use_default_canonical_filter: bool,
}

impl Default for CanonicalBaseNeutralityValidateArgs {
    fn default() -> Self {
        Self {
            repo_root: PathBuf::from("."),
            roots: vec![PathBuf::from("crates")],
            exclude_roots: vec![
                PathBuf::from("target"),
                PathBuf::from(".git"),
                PathBuf::from(".omc"),
                PathBuf::from(".omx"),
            ],
            self_test: false,
            use_default_canonical_filter: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalBaseNeutralityReport {
    pub files_checked: usize,
    pub violations: Vec<CanonicalBaseNeutralityViolation>,
    pub self_test: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CanonicalBaseNeutralityViolation {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub kind: CanonicalBaseNeutralityViolationKind,
    pub token: String,
    pub context: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalBaseNeutralityViolationKind {
    JurisdictionIdentifier,
    JurisdictionStringLiteral,
}

pub(crate) fn parse_canonical_base_neutrality_validate_args(
    args: Vec<String>,
) -> Result<CanonicalBaseNeutralityValidateArgs, String> {
    let mut parsed = CanonicalBaseNeutralityValidateArgs::default();
    let mut user_set_roots = false;
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.repo_root = PathBuf::from(value);
            }
            "--root" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                if !user_set_roots {
                    parsed.roots.clear();
                    parsed.use_default_canonical_filter = false;
                    user_set_roots = true;
                }
                parsed.roots.push(PathBuf::from(value));
            }
            "--exclude-root" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.exclude_roots.push(PathBuf::from(value));
            }
            "--self-test" => {
                parsed.self_test = true;
            }
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_canonical_base_neutrality_gate(
    args: CanonicalBaseNeutralityValidateArgs,
) -> Result<CanonicalBaseNeutralityReport, String> {
    if args.self_test {
        return run_self_test();
    }

    let repo_root = normalize_root(&args.repo_root)?;
    let exclude_roots = args
        .exclude_roots
        .iter()
        .map(|path| normalize_join(&repo_root, path))
        .collect::<Vec<_>>();
    let roots = args
        .roots
        .iter()
        .map(|path| normalize_join(&repo_root, path))
        .collect::<Vec<_>>();

    let mut file_paths: Vec<PathBuf> = Vec::new();
    for root in &roots {
        collect_rust_files(
            root,
            &exclude_roots,
            args.use_default_canonical_filter,
            &mut file_paths,
        )?;
    }
    file_paths.sort();
    file_paths.dedup();

    let mut violations = Vec::new();
    for path in &file_paths {
        let contents = fs::read_to_string(path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let display_path = path
            .strip_prefix(&repo_root)
            .unwrap_or(path)
            .display()
            .to_string();
        violations.extend(scan_document(&display_path, &contents));
    }

    let report = CanonicalBaseNeutralityReport {
        files_checked: file_paths.len(),
        violations,
        self_test: false,
    };
    if report.violations.is_empty() {
        Ok(report)
    } else {
        Err(format_violations(&report))
    }
}

fn run_self_test() -> Result<CanonicalBaseNeutralityReport, String> {
    let clean = r#"
pub struct LeastUsed;
pub struct KmsUseReceipt;
pub struct InvalidUserDataUri;
const BUSINESS: &str = "business";
"#;
    let dirty = r#"
pub struct FinancialKrCredit;
pub enum ResidencyClass { StrictKr }
pub enum HsmValidation { KcmvpFips1403Level3 }
const LOCALE: &str = "ko-KR";
const CERT: &str = "KCMVP validated module";
"#;

    let clean_hits = scan_document("<self-test-clean>", clean);
    if !clean_hits.is_empty() {
        return Err(format!(
            "canonical-base-neutrality self-test false-positive on clean fixture:\n{}",
            format_violation_rows(&clean_hits)
        ));
    }
    let dirty_hits = scan_document("<self-test-dirty>", dirty);
    let saw_identifier = dirty_hits
        .iter()
        .any(|hit| hit.kind == CanonicalBaseNeutralityViolationKind::JurisdictionIdentifier);
    let saw_string = dirty_hits
        .iter()
        .any(|hit| hit.kind == CanonicalBaseNeutralityViolationKind::JurisdictionStringLiteral);
    if !saw_identifier || !saw_string {
        return Err(format!(
            "canonical-base-neutrality self-test failed to detect both identifier and string leaks:\n{}",
            format_violation_rows(&dirty_hits)
        ));
    }

    Ok(CanonicalBaseNeutralityReport {
        files_checked: 2,
        violations: Vec::new(),
        self_test: true,
    })
}

fn collect_rust_files(
    root: &Path,
    exclude_roots: &[PathBuf],
    use_default_canonical_filter: bool,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    if exclude_roots
        .iter()
        .any(|excluded| root.starts_with(excluded))
    {
        return Ok(());
    }
    if root.is_file() {
        if is_rust_source(root)
            && (!use_default_canonical_filter || is_default_canonical_path(root))
        {
            out.push(root.to_path_buf());
        }
        return Ok(());
    }
    let entries = fs::read_dir(root).map_err(|error| {
        format!(
            "could not list canonical-base root {}: {error}",
            root.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "could not read entry under canonical-base root {}: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if exclude_roots
            .iter()
            .any(|excluded| path.starts_with(excluded))
        {
            continue;
        }
        if use_default_canonical_filter && is_default_excluded_component(&path) {
            continue;
        }
        if path.is_dir() {
            collect_rust_files(&path, exclude_roots, use_default_canonical_filter, out)?;
        } else if is_rust_source(&path)
            && (!use_default_canonical_filter || is_default_canonical_path(&path))
        {
            out.push(path);
        }
    }
    Ok(())
}

fn is_rust_source(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("rs")
}

fn is_default_excluded_component(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            "tests" | "benches" | "examples" | "fixtures" | "target"
        )
    })
}

fn is_default_canonical_path(path: &Path) -> bool {
    let components = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let Some(crates_index) = components.iter().position(|part| part == "crates") else {
        return false;
    };
    let Some(crate_name) = components.get(crates_index + 1) else {
        return false;
    };
    let is_base_layer = crate_name.ends_with("-kernel")
        || crate_name.ends_with("-domain")
        || crate_name.ends_with("-usecase");
    if !is_base_layer {
        return false;
    }
    let non_canonical_markers = [
        "regional", "-pack", "pack-", "-adapter", "-api", "-rest", "-cli", "check-", "-fitness",
        "dev-cli", "tooling",
    ];
    !non_canonical_markers
        .iter()
        .any(|marker| crate_name.contains(marker))
}

fn scan_document(path: &str, contents: &str) -> Vec<CanonicalBaseNeutralityViolation> {
    let mut violations = Vec::new();
    for (line_index, line) in contents.lines().enumerate() {
        let mut idx = 0;
        let bytes = line.as_bytes();
        while idx < bytes.len() {
            if idx + 1 < bytes.len() && bytes[idx] == b'/' && bytes[idx + 1] == b'/' {
                break;
            }
            if bytes[idx] == b'"' {
                let (literal, end_idx) = parse_string_literal(line, idx);
                if string_contains_jurisdiction_code(&literal) {
                    violations.push(CanonicalBaseNeutralityViolation {
                        path: path.to_string(),
                        line: line_index + 1,
                        column: idx + 1,
                        kind: CanonicalBaseNeutralityViolationKind::JurisdictionStringLiteral,
                        token: literal,
                        context: line.trim().to_string(),
                    });
                }
                idx = end_idx;
                continue;
            }
            if is_identifier_start(bytes[idx]) {
                let start = idx;
                idx += 1;
                while idx < bytes.len() && is_identifier_continue(bytes[idx]) {
                    idx += 1;
                }
                let token = &line[start..idx];
                if identifier_contains_jurisdiction_component(token) {
                    violations.push(CanonicalBaseNeutralityViolation {
                        path: path.to_string(),
                        line: line_index + 1,
                        column: start + 1,
                        kind: CanonicalBaseNeutralityViolationKind::JurisdictionIdentifier,
                        token: token.to_string(),
                        context: line.trim().to_string(),
                    });
                }
                continue;
            }
            idx += 1;
        }
    }
    violations
}

fn parse_string_literal(line: &str, start_quote: usize) -> (String, usize) {
    let bytes = line.as_bytes();
    let mut idx = start_quote + 1;
    let mut out = String::new();
    let mut escaped = false;
    while idx < bytes.len() {
        let byte = bytes[idx];
        if escaped {
            out.push(byte as char);
            escaped = false;
            idx += 1;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            idx += 1;
            continue;
        }
        if byte == b'"' {
            return (out, idx + 1);
        }
        out.push(byte as char);
        idx += 1;
    }
    (out, idx)
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn identifier_contains_jurisdiction_component(identifier: &str) -> bool {
    for token in JURISDICTION_TOKENS {
        let mut search_from = 0;
        while let Some(offset) = identifier[search_from..].find(token) {
            let start = search_from + offset;
            let end = start + token.len();
            let prev = identifier[..start].chars().next_back();
            let next = identifier[end..].chars().next();
            let prev_boundary = prev.is_none_or(|ch| ch == '_' || ch.is_ascii_lowercase());
            let next_boundary =
                next.is_none_or(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit());
            if prev_boundary && next_boundary {
                return true;
            }
            search_from = end;
        }
    }
    false
}

fn string_contains_jurisdiction_code(literal: &str) -> bool {
    literal
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|segment| !segment.is_empty())
        .any(|segment| {
            let lower = segment.to_ascii_lowercase();
            JURISDICTION_CODES.contains(&lower.as_str())
        })
}

fn normalize_root(path: &Path) -> Result<PathBuf, String> {
    if path.exists() {
        fs::canonicalize(path)
            .map_err(|error| format!("could not canonicalize {}: {error}", path.display()))
    } else {
        Ok(path.to_path_buf())
    }
}

fn normalize_join(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn format_violations(report: &CanonicalBaseNeutralityReport) -> String {
    format!(
        "canonical base neutrality violations: {} files checked, {} jurisdiction leaks\n{}",
        report.files_checked,
        report.violations.len(),
        format_violation_rows(&report.violations)
    )
}

fn format_violation_rows(violations: &[CanonicalBaseNeutralityViolation]) -> String {
    violations
        .iter()
        .map(|violation| {
            let kind = match violation.kind {
                CanonicalBaseNeutralityViolationKind::JurisdictionIdentifier => {
                    "jurisdiction-identifier"
                }
                CanonicalBaseNeutralityViolationKind::JurisdictionStringLiteral => {
                    "jurisdiction-string"
                }
            };
            format!(
                "{}:{}:{}: {} `{}` — {}",
                violation.path,
                violation.line,
                violation.column,
                kind,
                violation.token,
                violation.context
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_scan_catches_camel_case_jurisdiction_components() {
        assert!(identifier_contains_jurisdiction_component(
            "FinancialKrCredit"
        ));
        assert!(identifier_contains_jurisdiction_component("StrictKr"));
        assert!(identifier_contains_jurisdiction_component(
            "KrWithUsFailover"
        ));
        assert!(identifier_contains_jurisdiction_component("SovereignEu"));
        assert!(identifier_contains_jurisdiction_component(
            "KcmvpFips1403Level3"
        ));
    }

    #[test]
    fn identifier_scan_avoids_common_us_false_positives() {
        assert!(!identifier_contains_jurisdiction_component("LeastUsed"));
        assert!(!identifier_contains_jurisdiction_component("KmsUseReceipt"));
        assert!(!identifier_contains_jurisdiction_component(
            "InvalidUserDataUri"
        ));
    }

    #[test]
    fn string_scan_catches_standalone_country_and_locale_codes() {
        assert!(string_contains_jurisdiction_code("strict KR residency"));
        assert!(string_contains_jurisdiction_code("ko-KR"));
        assert!(string_contains_jurisdiction_code("sovereign-kr"));
        assert!(string_contains_jurisdiction_code("KCMVP validated module"));
        assert!(!string_contains_jurisdiction_code("business"));
        assert!(!string_contains_jurisdiction_code("trust"));
    }

    #[test]
    fn document_scan_ignores_line_comments_but_checks_literals_and_identifiers() {
        let hits = scan_document(
            "fixture.rs",
            r#"
// pub struct FinancialKrCredit;
pub struct LeastUsed;
pub struct StrictKr;
const MARKET: &str = "ko-KR";
"#,
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(
            hits[0].kind,
            CanonicalBaseNeutralityViolationKind::JurisdictionIdentifier
        );
        assert_eq!(
            hits[1].kind,
            CanonicalBaseNeutralityViolationKind::JurisdictionStringLiteral
        );
    }

    #[test]
    fn parse_defaults_to_canonical_crate_filter() {
        let args =
            parse_canonical_base_neutrality_validate_args(Vec::new()).expect("defaults parse");
        assert_eq!(args.roots, vec![PathBuf::from("crates")]);
        assert!(args.use_default_canonical_filter);
    }

    #[test]
    fn parse_custom_root_disables_default_crate_filter() {
        let args = parse_canonical_base_neutrality_validate_args(vec![
            "--root".to_string(),
            "crates/oya-cloud-data-kernel/src".to_string(),
        ])
        .expect("custom root parses");
        assert_eq!(
            args.roots,
            vec![PathBuf::from("crates/oya-cloud-data-kernel/src")]
        );
        assert!(!args.use_default_canonical_filter);
    }

    #[test]
    fn self_test_proves_positive_and_negative_fixtures() {
        let report = validate_canonical_base_neutrality_gate(CanonicalBaseNeutralityValidateArgs {
            self_test: true,
            ..CanonicalBaseNeutralityValidateArgs::default()
        })
        .expect("self-test passes");
        assert!(report.self_test);
        assert_eq!(report.files_checked, 2);
    }
}
