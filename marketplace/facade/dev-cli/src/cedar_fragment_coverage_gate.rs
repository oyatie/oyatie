//! `oya gate validate cedar-fragment-coverage` — runtime for the C01..C04
//! invariants declared in `registry/cedar-fragments.json`. Makes the
//! cedar-fragments-registry non-paper: drift between OpenAPI contracts,
//! bounded-contexts.json, and on-disk `.cedar` files now fails the lane.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use check_cedar_fragment_coverage::{
    CoverageInputs, FragmentRow, FragmentStatus, ValidationReport, Violation, validate,
};

use crate::json_scan::{
    extract_json_array_for_key, extract_json_objects, parse_json_string_array_field,
    parse_json_string_field,
};
use crate::usage;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CedarFragmentCoverageValidateArgs {
    registry_path: PathBuf,
    bounded_contexts_path: PathBuf,
    contracts_dir: PathBuf,
    cedar_dir: PathBuf,
    emit_evidence_path: Option<PathBuf>,
}

pub(crate) fn parse_cedar_fragment_coverage_validate_args(
    args: Vec<String>,
) -> Result<CedarFragmentCoverageValidateArgs, String> {
    let mut parsed = CedarFragmentCoverageValidateArgs {
        registry_path: PathBuf::from("registry/cedar-fragments.json"),
        bounded_contexts_path: PathBuf::from("registry/bounded-contexts.json"),
        contracts_dir: PathBuf::from("contracts"),
        cedar_dir: PathBuf::from(".omc/cedar"),
        emit_evidence_path: None,
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--registry" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.registry_path = PathBuf::from(path);
            }
            "--bounded-contexts" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.bounded_contexts_path = PathBuf::from(path);
            }
            "--contracts-dir" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.contracts_dir = PathBuf::from(path);
            }
            "--cedar-dir" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.cedar_dir = PathBuf::from(path);
            }
            "--emit-evidence" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.emit_evidence_path = Some(PathBuf::from(path));
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CedarFragmentCoverageReport {
    pub report: ValidationReport,
    pub validation_duration_ms: u64,
}

pub(crate) fn validate_cedar_fragment_coverage_gate(
    args: CedarFragmentCoverageValidateArgs,
) -> Result<CedarFragmentCoverageReport, String> {
    let start = Instant::now();

    let registry_rows = read_fragment_registry(&args.registry_path)?;
    let bc_references = read_bc_references(&args.bounded_contexts_path)?;
    let openapi_references = read_openapi_references(&args.contracts_dir)?;
    let head_tracked_paths = git_ls_files()?;
    let cedar_files_on_disk = head_tracked_paths
        .iter()
        .filter(|path| {
            path.starts_with(&format!("{}/", args.cedar_dir.display())) && path.ends_with(".cedar")
        })
        .cloned()
        .collect();

    let inputs = CoverageInputs {
        registry_rows,
        openapi_references,
        bc_references,
        cedar_files_on_disk,
        head_tracked_paths,
    };

    let report = validate(&inputs);
    let validation_duration_ms = start.elapsed().as_millis() as u64;
    let wrapped = CedarFragmentCoverageReport {
        report: report.clone(),
        validation_duration_ms,
    };

    if let Some(evidence_path) = &args.emit_evidence_path {
        write_evidence_bundle(evidence_path, &wrapped, &args.registry_path)?;
    }

    if !report.is_clean() {
        return Err(format_violations(&report.violations));
    }

    Ok(wrapped)
}

fn read_fragment_registry(path: &Path) -> Result<Vec<FragmentRow>, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "cedar-fragments registry unreadable {}: {error}",
            path.display()
        )
    })?;
    let fragments_array = extract_json_array_for_key(&text, "fragments").ok_or_else(|| {
        format!(
            "cedar-fragments registry missing top-level `fragments` array in {}",
            path.display()
        )
    })?;
    let objects = extract_json_objects(fragments_array);
    let mut rows = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        let Some(fragment_id) = parse_json_string_field(object, "fragment_id") else {
            return Err(format!(
                "cedar-fragments row index {index} missing fragment_id in {}",
                path.display()
            ));
        };
        let Some(fragment_path_planned) = parse_json_string_field(object, "fragment_path_planned")
        else {
            return Err(format!(
                "cedar-fragments row fragment_id={fragment_id} missing fragment_path_planned"
            ));
        };
        let status_str = parse_json_string_field(object, "status").ok_or_else(|| {
            format!("cedar-fragments row fragment_id={fragment_id} missing status")
        })?;
        let status = match status_str.as_str() {
            "operational" => FragmentStatus::Operational,
            "planned" => FragmentStatus::Planned,
            "blocked-by-foundation-prerequisite" => FragmentStatus::BlockedByFoundationPrerequisite,
            other => {
                return Err(format!(
                    "cedar-fragments row fragment_id={fragment_id} has unknown status `{other}`"
                ));
            }
        };
        rows.push(FragmentRow {
            fragment_id,
            fragment_path_planned,
            status,
        });
    }
    Ok(rows)
}

fn read_bc_references(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "bounded-contexts registry unreadable {}: {error}",
            path.display()
        )
    })?;
    // bounded-contexts.json has shape:
    //   { "bounded_contexts": [ { ..., "cedar_fragments_planned": ["id (qualifier)", ...] } ] }
    // We extract every cedar_fragments_planned[] string, strip the trailing
    // " (...)" qualifier and the ".cedar" suffix, leaving the fragment_id.
    let bc_array = extract_json_array_for_key(&text, "bounded_contexts").unwrap_or("");
    let bcs = extract_json_objects(bc_array);
    let mut references = BTreeSet::new();
    for bc in bcs {
        let Some(planned) = parse_json_string_array_field(bc, "cedar_fragments_planned") else {
            continue;
        };
        for raw in planned {
            references.insert(normalize_fragment_reference(&raw));
        }
    }
    Ok(references)
}

fn normalize_fragment_reference(raw: &str) -> String {
    let without_qualifier = raw.split('(').next().unwrap_or("").trim();
    let stripped = without_qualifier
        .strip_suffix(".cedar")
        .unwrap_or(without_qualifier);
    stripped.trim().to_string()
}

fn read_openapi_references(contracts_dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut references = BTreeSet::new();
    let entries = match fs::read_dir(contracts_dir) {
        Ok(it) => it,
        Err(_) => return Ok(references), // contracts dir optional
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !file_name.ends_with(".openapi.yaml") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("openapi file unreadable {}: {error}", path.display()))?;
        scan_yaml_cedar_fragments(&text, &mut references);
    }
    Ok(references)
}

/// Scan YAML text for fragment-reference arrays. Recognizes both forms:
///   - `cedar_fragments: [...]` (kernel-shape, for Surface.cedar_fragments[])
///   - `x-cedar-fragments: [...]` (OpenAPI extension at route/operation level)
///
/// Each supports inline flow `[a, b, c]` and block list `- item` syntax.
/// Schema-property declarations (where the next non-blank line is `type: array`
/// or similar) are ignored.
fn scan_yaml_cedar_fragments(text: &str, out: &mut BTreeSet<String>) {
    const PREFIXES: [&str; 2] = ["cedar_fragments:", "x-cedar-fragments:"];
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        let Some(rest) = PREFIXES.iter().find_map(|p| trimmed.strip_prefix(p)) else {
            continue;
        };
        let rest_trimmed = rest.trim();
        if let Some(inline) = rest_trimmed
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
        {
            for item in inline.split(',') {
                let token = item.trim().trim_matches('"').trim_matches('\'');
                if !token.is_empty() {
                    out.insert(token.to_string());
                }
            }
            continue;
        }
        // Block-list form: look ahead at indented `- item` lines.
        let base_indent = line.len() - trimmed.len();
        while let Some(peek) = lines.peek() {
            let peek_trim = peek.trim_start();
            let peek_indent = peek.len() - peek_trim.len();
            if peek_indent <= base_indent {
                break;
            }
            if let Some(item) = peek_trim.strip_prefix("- ") {
                let token = item.trim().trim_matches('"').trim_matches('\'');
                if !token.is_empty() && !token.contains(':') {
                    out.insert(token.to_string());
                }
                lines.next();
                continue;
            }
            // Schema-property declarations like `type: array` end the array.
            break;
        }
    }
}

fn git_ls_files() -> Result<BTreeSet<String>, String> {
    let output = Command::new("git")
        .args(["ls-files"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("git ls-files failed to spawn: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(format!("git ls-files exit {}: {stderr}", output.status));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("git ls-files output not UTF-8: {error}"))?;
    Ok(stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.to_string())
        .collect())
}

fn write_evidence_bundle(
    path: &Path,
    wrapped: &CedarFragmentCoverageReport,
    registry_path: &Path,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "evidence bundle dir unwriteable {}: {error}",
                parent.display()
            )
        })?;
    }
    let report = &wrapped.report;
    let outcome = if report.is_clean() {
        "success"
    } else {
        "failure"
    };
    let body = format!(
        "{{\n  \"$schema_ref\": \"/templates/evidence-bundle-template.json\",\n  \"_artifact_id\": \"cedar-fragment-coverage-lane-run\",\n  \"_meta\": {{ \"emitter\": \"oya-dev-cli gate validate cedar-fragment-coverage\", \"registry_path\": \"{}\" }},\n  \"outcome\": \"{}\",\n  \"rows_seen\": {},\n  \"openapi_references_seen\": {},\n  \"bc_references_seen\": {},\n  \"cedar_files_seen\": {},\n  \"violation_count\": {},\n  \"validation_duration_ms\": {}\n}}\n",
        escape_json(&registry_path.display().to_string()),
        outcome,
        report.rows_seen,
        report.openapi_references_seen,
        report.bc_references_seen,
        report.cedar_files_seen,
        report.violations.len(),
        wrapped.validation_duration_ms,
    );
    fs::write(path, body)
        .map_err(|error| format!("evidence bundle write failed {}: {error}", path.display()))?;
    Ok(())
}

fn format_violations(violations: &[Violation]) -> String {
    violations
        .iter()
        .map(|v| match v {
            Violation::C01UnknownOpenapiReference { fragment_id } => {
                format!("C01: openapi references unknown fragment `{fragment_id}`")
            }
            Violation::C02UnknownBcReference { fragment_id } => {
                format!("C02: bounded-contexts references unknown fragment `{fragment_id}`")
            }
            Violation::C03OrphanCedarFile { path } => {
                format!("C03: orphan .cedar file (no registry row): {path}")
            }
            Violation::C03CedarFileStatusMismatch {
                fragment_id,
                actual_status,
            } => format!(
                "C03: fragment `{fragment_id}` has .cedar file on disk but status={}",
                actual_status.name()
            ),
            Violation::C04OperationalPathMissing { fragment_id, path } => {
                format!("C04: fragment `{fragment_id}` status=operational but path missing: {path}")
            }
            Violation::C04NonOperationalPathExists {
                fragment_id,
                path,
                status,
            } => format!(
                "C04: fragment `{fragment_id}` status={} but path present in HEAD: {path}",
                status.name()
            ),
            Violation::DuplicateFragmentId { fragment_id } => {
                format!("duplicate fragment_id in registry: `{fragment_id}`")
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_defaults() {
        let args = parse_cedar_fragment_coverage_validate_args(vec![]).unwrap();
        assert_eq!(
            args.registry_path,
            PathBuf::from("registry/cedar-fragments.json")
        );
        assert_eq!(
            args.bounded_contexts_path,
            PathBuf::from("registry/bounded-contexts.json")
        );
        assert_eq!(args.contracts_dir, PathBuf::from("contracts"));
        assert_eq!(args.cedar_dir, PathBuf::from(".omc/cedar"));
    }

    #[test]
    fn parse_args_unknown_flag_errors() {
        let result = parse_cedar_fragment_coverage_validate_args(vec!["--bogus".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn normalize_strips_qualifier_and_suffix() {
        assert_eq!(
            normalize_fragment_reference("ops-internal-public.cedar (M02-P20 inherited)"),
            "ops-internal-public"
        );
        assert_eq!(
            normalize_fragment_reference("ops-tenant-private.cedar (Wave 5)"),
            "ops-tenant-private"
        );
        assert_eq!(normalize_fragment_reference("ops-bare-id"), "ops-bare-id");
    }

    #[test]
    fn scan_yaml_inline_form() {
        let yaml = "routes:\n  cedar_fragments: [foo, bar, baz]\n";
        let mut out = BTreeSet::new();
        scan_yaml_cedar_fragments(yaml, &mut out);
        assert_eq!(out.len(), 3);
        assert!(out.contains("foo"));
        assert!(out.contains("baz"));
    }

    #[test]
    fn scan_yaml_block_form() {
        let yaml = "routes:\n  cedar_fragments:\n    - alpha\n    - beta\n  next: value\n";
        let mut out = BTreeSet::new();
        scan_yaml_cedar_fragments(yaml, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out.contains("alpha"));
        assert!(out.contains("beta"));
    }

    #[test]
    fn scan_yaml_recognizes_openapi_extension_prefix() {
        let yaml = "paths:\n  /workspace:\n    get:\n      x-cedar-fragments: [ops-internal-public, ops-tenant-public]\n";
        let mut out = BTreeSet::new();
        scan_yaml_cedar_fragments(yaml, &mut out);
        assert_eq!(out.len(), 2);
        assert!(out.contains("ops-internal-public"));
        assert!(out.contains("ops-tenant-public"));
    }

    #[test]
    fn scan_yaml_ignores_schema_property_declaration() {
        // The OpenAPI schema declares `cedar_fragments:` as a property TYPE,
        // not as data. The scanner should not treat the `type: array` line as
        // a fragment id.
        let yaml = "schemas:\n  Surface:\n    cedar_fragments:\n      type: array\n      items: { type: string }\n";
        let mut out = BTreeSet::new();
        scan_yaml_cedar_fragments(yaml, &mut out);
        assert!(
            out.is_empty(),
            "schema declaration should not be parsed as data: {out:?}"
        );
    }
}
