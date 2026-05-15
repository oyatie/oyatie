//! `oya gate validate active-artifact-contract` — vertical enforcement loop runtime
//! for the v3.0.0 active machine-readable artifact contract per ADR-0069.
//!
//! Closes architect r17/r18/r19 + full-consensus-planner-v3 §5 hops 1-2+5-6:
//! - Reads the artifact-capabilities registry
//! - Resolves HEAD-tracked paths via `git ls-files`
//! - Detects violations: R01 (artifact_path not in HEAD) and R02 (duplicate artifact_id)
//! - Optionally emits evidence bundle with `validation_duration_ms`
//! - Optionally emits ONE graph edge artifact per consensus-v3 amendment #4

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

use crate::json_scan::{extract_json_objects, parse_json_string_field};
use crate::usage;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveArtifactContractValidateArgs {
    registry_path: PathBuf,                 // data_class: INTERNAL_ONLY
    emit_evidence_path: Option<PathBuf>,    // data_class: INTERNAL_ONLY
    emit_graph_edges_path: Option<PathBuf>, // data_class: INTERNAL_ONLY
}

pub(crate) fn parse_active_artifact_contract_validate_args(
    args: Vec<String>,
) -> Result<ActiveArtifactContractValidateArgs, String> {
    let mut parsed = ActiveArtifactContractValidateArgs {
        registry_path: PathBuf::from(
            "registries/cross-cutting/artifact-capabilities-registry.json",
        ),
        emit_evidence_path: None,
        emit_graph_edges_path: None,
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
            "--emit-evidence" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.emit_evidence_path = Some(PathBuf::from(path));
            }
            "--emit-graph-edges" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.emit_graph_edges_path = Some(PathBuf::from(path));
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActiveArtifactContractReport {
    pub rows_seen: usize,                           // data_class: INTERNAL_ONLY
    pub head_tracked_count: usize,                  // data_class: INTERNAL_ONLY
    pub untracked_paths: Vec<String>,               // data_class: INTERNAL_ONLY
    pub duplicate_ids: Vec<String>,                 // data_class: INTERNAL_ONLY
    pub validation_duration_ms: u64,                // data_class: INTERNAL_ONLY
    pub graph_edges: Vec<(String, String, String)>, // data_class: INTERNAL_ONLY (artifact_id, artifact_profile, edge_type)
}

pub(crate) fn validate_active_artifact_contract_gate(
    args: ActiveArtifactContractValidateArgs,
) -> Result<ActiveArtifactContractReport, String> {
    let start = Instant::now();

    let registry_text = fs::read_to_string(&args.registry_path).map_err(|error| {
        format!(
            "active-artifact-contract registry unreadable {}: {error}",
            args.registry_path.display()
        )
    })?;

    // Extract rows[] section. Registry shape is:
    //   { "_meta": {...}, "_self_row": {...}, "rows": [ {row}, {row}, ... ] }
    let rows_section = extract_rows_array(&registry_text);

    let row_objects: Vec<&str> = match rows_section {
        Some(section) => extract_json_objects(section),
        None => {
            return Err(format!(
                "active-artifact-contract registry missing top-level `rows` array in {}",
                args.registry_path.display()
            ));
        }
    };

    let mut rows_seen = 0usize;
    let mut artifact_ids: BTreeMap<String, usize> = BTreeMap::new();
    let mut paths_seen: BTreeMap<String, String> = BTreeMap::new(); // normalized git path → registry artifact_path
    let mut duplicate_ids: Vec<String> = Vec::new();
    let mut graph_edges: Vec<(String, String, String)> = Vec::new();

    for (index, row) in row_objects.iter().enumerate() {
        rows_seen += 1;
        let Some(artifact_id) = parse_json_string_field(row, "artifact_id") else {
            return Err(format!(
                "row index {index} missing artifact_id in {}",
                args.registry_path.display()
            ));
        };
        let Some(artifact_path) = parse_json_string_field(row, "artifact_path") else {
            return Err(format!(
                "row artifact_id={artifact_id} missing artifact_path in {}",
                args.registry_path.display()
            ));
        };
        let artifact_profile = parse_json_string_field(row, "artifact_profile")
            .unwrap_or_else(|| "unspecified".to_string());

        // R02: duplicate artifact_id
        if artifact_ids.insert(artifact_id.clone(), index).is_some() {
            duplicate_ids.push(artifact_id.clone());
        }

        paths_seen.insert(
            normalize_repo_root_artifact_path(&artifact_path),
            artifact_path.clone(),
        );

        // Emit one graph edge per row: artifact_id --declares--> artifact_profile
        graph_edges.push((
            artifact_id.clone(),
            artifact_profile.clone(),
            "declares".to_string(),
        ));
    }

    // Resolve HEAD-tracked paths via `git ls-files`.
    let head_tracked = git_ls_files()?;

    let mut untracked_paths: Vec<String> = Vec::new();
    for (path, registry_path) in &paths_seen {
        if !is_head_tracked_artifact_path(path, &head_tracked) {
            untracked_paths.push(registry_path.clone());
        }
    }

    let validation_duration_ms = start.elapsed().as_millis() as u64;

    let report = ActiveArtifactContractReport {
        rows_seen,
        head_tracked_count: head_tracked.len(),
        untracked_paths: untracked_paths.clone(),
        duplicate_ids: duplicate_ids.clone(),
        validation_duration_ms,
        graph_edges: graph_edges.clone(),
    };

    // Optional evidence emission per consensus-v3 amendment #10 + step 5.
    if let Some(evidence_path) = &args.emit_evidence_path {
        write_evidence_bundle(evidence_path, &report, &args.registry_path)?;
    }

    // Optional graph-edge emission per consensus-v3 amendment #4 + step 6.
    if let Some(edges_path) = &args.emit_graph_edges_path {
        write_graph_edges(edges_path, &graph_edges)?;
    }

    if !untracked_paths.is_empty() || !duplicate_ids.is_empty() {
        let mut message = String::new();
        if !untracked_paths.is_empty() {
            message.push_str(&format!(
                "R01: {} artifact_path(s) not HEAD-tracked: {}",
                untracked_paths.len(),
                untracked_paths.join(", ")
            ));
        }
        if !duplicate_ids.is_empty() {
            if !message.is_empty() {
                message.push_str("; ");
            }
            message.push_str(&format!(
                "R02: {} duplicate artifact_id(s): {}",
                duplicate_ids.len(),
                duplicate_ids.join(", ")
            ));
        }
        return Err(message);
    }

    Ok(report)
}

/// Locate the top-level `rows` array in the registry JSON text.
fn extract_rows_array(text: &str) -> Option<&str> {
    let key_index = text.find("\"rows\"")?;
    let after_key = &text[key_index..];
    let colon_index = after_key.find(':')?;
    let after_colon = &after_key[colon_index + 1..];
    let open_bracket = after_colon.find('[')?;
    Some(&after_colon[open_bracket..])
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

fn normalize_repo_root_artifact_path(path: &str) -> String {
    path.strip_prefix('/').unwrap_or(path).to_string()
}

fn is_head_tracked_artifact_path(path: &str, head_tracked: &BTreeSet<String>) -> bool {
    head_tracked.contains(path)
        || head_tracked
            .iter()
            .any(|tracked| is_child_path(path, tracked))
}

fn is_child_path(parent: &str, child: &str) -> bool {
    let Some(rest) = child.strip_prefix(parent) else {
        return false;
    };
    rest.starts_with('/')
}

fn write_evidence_bundle(
    path: &Path,
    report: &ActiveArtifactContractReport,
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

    let outcome = if report.untracked_paths.is_empty() && report.duplicate_ids.is_empty() {
        "success"
    } else {
        "failure"
    };
    let untracked_json: String = report
        .untracked_paths
        .iter()
        .map(|p| format!("\"{}\"", escape_json(p)))
        .collect::<Vec<_>>()
        .join(",");
    let dup_json: String = report
        .duplicate_ids
        .iter()
        .map(|i| format!("\"{}\"", escape_json(i)))
        .collect::<Vec<_>>()
        .join(",");
    let body = format!(
        "{{\n  \"$schema_ref\": \"/templates/evidence-bundle-template.json\",\n  \"_artifact_id\": \"active-artifact-contract-lane-run\",\n  \"_meta\": {{ \"emitter\": \"oya-dev-cli gate validate active-artifact-contract\", \"registry_path\": \"{}\" }},\n  \"outcome\": \"{}\",\n  \"rows_seen\": {},\n  \"head_tracked_count\": {},\n  \"untracked_paths\": [{}],\n  \"duplicate_ids\": [{}],\n  \"validation_duration_ms\": {},\n  \"graph_edge_count\": {}\n}}\n",
        escape_json(&registry_path.display().to_string()),
        outcome,
        report.rows_seen,
        report.head_tracked_count,
        untracked_json,
        dup_json,
        report.validation_duration_ms,
        report.graph_edges.len()
    );
    fs::write(path, body)
        .map_err(|error| format!("evidence bundle write failed {}: {error}", path.display()))?;
    Ok(())
}

fn write_graph_edges(path: &Path, edges: &[(String, String, String)]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!("graph edges dir unwriteable {}: {error}", parent.display())
        })?;
    }
    let edges_json: String = edges
        .iter()
        .map(|(src, tgt, edge)| {
            format!(
                "    {{ \"source\": \"{}\", \"target\": \"{}\", \"edge_type\": \"{}\" }}",
                escape_json(src),
                escape_json(tgt),
                escape_json(edge)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    let body = format!(
        "{{\n  \"$schema_ref\": \"specs/cross-cutting/knowledge-graph-schema.json\",\n  \"_artifact_id\": \"active-artifact-contract-edges\",\n  \"_meta\": {{ \"emitter\": \"oya-dev-cli gate validate active-artifact-contract\", \"layer\": \"semantic\", \"purpose\": \"Generated graph edges that connect active machine-readable artifacts to their declared schemas, registries, templates, and ledgers.\" }},\n  \"edges\": [\n{}\n  ]\n}}\n",
        edges_json
    );
    fs::write(path, body)
        .map_err(|error| format!("graph edges write failed {}: {error}", path.display()))?;
    Ok(())
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
        let args = parse_active_artifact_contract_validate_args(vec![]).unwrap();
        assert_eq!(
            args.registry_path,
            PathBuf::from("registries/cross-cutting/artifact-capabilities-registry.json")
        );
        assert!(args.emit_evidence_path.is_none());
        assert!(args.emit_graph_edges_path.is_none());
    }

    #[test]
    fn parse_args_full() {
        let args = parse_active_artifact_contract_validate_args(vec![
            "--registry".into(),
            "tests/fixtures/missing-row-registry.json".into(),
            "--emit-evidence".into(),
            "/tmp/evidence.json".into(),
            "--emit-graph-edges".into(),
            "/tmp/edges.json".into(),
        ])
        .unwrap();
        assert_eq!(
            args.registry_path.display().to_string(),
            "tests/fixtures/missing-row-registry.json"
        );
        assert_eq!(
            args.emit_evidence_path.unwrap().display().to_string(),
            "/tmp/evidence.json"
        );
        assert_eq!(
            args.emit_graph_edges_path.unwrap().display().to_string(),
            "/tmp/edges.json"
        );
    }

    #[test]
    fn parse_args_unknown_flag_errors() {
        let result = parse_active_artifact_contract_validate_args(vec!["--bogus".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn extract_rows_array_finds_section() {
        let text = r#"{"_meta": {"x": 1}, "rows": [{"a":1},{"b":2}]}"#;
        let section = extract_rows_array(text).expect("rows present");
        assert!(section.starts_with('['));
    }

    #[test]
    fn extract_rows_array_missing_returns_none() {
        let text = r#"{"foo": "bar"}"#;
        assert!(extract_rows_array(text).is_none());
    }

    #[test]
    fn repo_root_artifact_paths_match_git_ls_files_shape() {
        assert_eq!(
            normalize_repo_root_artifact_path("/specs/cross-cutting/masterplan.json"),
            "specs/cross-cutting/masterplan.json"
        );
        assert_eq!(
            normalize_repo_root_artifact_path("registries/cross-cutting/test-suite-registry.json"),
            "registries/cross-cutting/test-suite-registry.json"
        );
    }

    #[test]
    fn directory_artifact_paths_are_covered_by_tracked_children() {
        let tracked = BTreeSet::from([
            "crates/oya-foundry-settings-template-kernel/Cargo.toml".to_string(),
            "crates/oya-foundry-settings-template-kernel/src/lib.rs".to_string(),
        ]);

        assert!(is_head_tracked_artifact_path(
            "crates/oya-foundry-settings-template-kernel",
            &tracked
        ));
        assert!(!is_head_tracked_artifact_path(
            "crates/oya-foundry-settings-template",
            &tracked
        ));
    }
}
