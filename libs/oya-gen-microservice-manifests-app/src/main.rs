//! Retired CLI guard for [`oya_gen_microservice_manifests_app`] provenance.
//!
//! The former writer walked `microservices/<ms>/` and refreshed
//! `specs/microservices/manifests-index.json`. That layout is retired: current
//! manifest-index authority uses `oya/` and `cloud/` roots, keeps `anonymous` as
//! a no-standalone community subproduct, and keeps `foundry` as a retired row
//! absorbed by `intelligence`.
//!
//! Flags:
//!  - `--repo-root <path>` (default `.`)
//!  - `--check` — validate that the checked-in index still matches the current
//!    source-authority contract and cannot be rewritten to legacy rows.
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

use oya_gen_microservice_manifests_app::{MICROSERVICES, build_manifests_index};

const MANIFESTS_INDEX_GENERATED_AT: &str = "2026-05-19";
const RETIRED_WRITER_MESSAGE: &str = "oya-gen-microservice-manifests writer is retired/provenance-only; run with --check to validate the current source-authority index guard. It will not write specs/microservices/manifests-index.json.";

fn main() -> ExitCode {
    let mut repo_root = PathBuf::from(".");
    let mut check = false;
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                if let Some(p) = iter.next() {
                    repo_root = PathBuf::from(p);
                } else {
                    eprintln!("--repo-root requires an argument");
                    return ExitCode::from(2);
                }
            }
            "--check" => check = true,
            other => {
                eprintln!("unknown flag: {other}");
                return ExitCode::from(2);
            }
        }
    }

    if !check {
        eprintln!("{RETIRED_WRITER_MESSAGE}");
        return ExitCode::from(2);
    }

    let expected = build_manifests_index(MANIFESTS_INDEX_GENERATED_AT, MICROSERVICES);
    match validate_current_manifest_index(&repo_root, &expected) {
        Ok(summary) => {
            println!(
                "manifests --check: retired writer guard passed; rows={} active_manifest_paths_checked={}",
                summary.rows, summary.active_manifest_paths_checked
            );
            ExitCode::SUCCESS
        }
        Err(problems) => {
            for problem in &problems {
                eprintln!("{problem}");
            }
            eprintln!(
                "manifests --check: retired writer guard failed ({} problems)",
                problems.len()
            );
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CheckSummary {
    rows: usize,
    active_manifest_paths_checked: usize,
}

fn validate_current_manifest_index(
    repo_root: &Path,
    expected: &Value,
) -> Result<CheckSummary, Vec<String>> {
    let mut problems = Vec::new();
    let idx_path = repo_root.join("specs/microservices/manifests-index.json");
    let on_disk_text = match fs::read_to_string(&idx_path) {
        Ok(text) => text,
        Err(e) => {
            return Err(vec![format!("[missing] {}: {e}", idx_path.display())]);
        }
    };
    let on_disk: Value = match serde_json::from_str(&on_disk_text) {
        Ok(value) => value,
        Err(e) => {
            return Err(vec![format!("[invalid-json] {}: {e}", idx_path.display())]);
        }
    };

    if &on_disk != expected {
        problems.push(format!(
            "[diff] {} differs from embedded current-path manifest-index contract",
            idx_path.display()
        ));
    }

    let Some(rows) = on_disk.get("microservices").and_then(Value::as_array) else {
        problems.push("[invalid] microservices must be an array".to_string());
        return Err(problems);
    };
    let expected_count = rows.len();
    if on_disk.get("manifest_count").and_then(Value::as_u64) != Some(expected_count as u64) {
        problems.push(format!(
            "[invalid] manifest_count must equal microservices length {expected_count}"
        ));
    }

    for retired_name in ["cell", "network", "shorts"] {
        if rows
            .iter()
            .any(|row| value_str(row, "name") == Some(retired_name))
        {
            problems.push(format!(
                "[legacy-row] retired `{retired_name}` must not appear in manifests-index"
            ));
        }
    }

    let readiness = &on_disk["readiness_contracts"]["multi_region_disposition"];
    if !readiness.is_object() {
        problems.push("[invalid] readiness_contracts.multi_region_disposition missing".to_string());
    }
    if value_str(readiness, "manifest_field") != Some("multi_region_disposition") {
        problems.push(
            "[invalid] readiness_contracts.multi_region_disposition.manifest_field drifted"
                .to_string(),
        );
    }
    if !value_str(readiness, "coverage_scope_note")
        .unwrap_or_default()
        .contains("current `oya/<service>/manifest.json` and `cloud/<service>/manifest.json` roots")
    {
        problems.push(
            "[invalid] coverage_scope_note must distinguish current oya/cloud roots from legacy microservices roots"
                .to_string(),
        );
    }

    let mut active_manifest_paths_checked = 0usize;
    for row in rows {
        let name = value_str(row, "name").unwrap_or("<missing-name>");
        match name {
            "anonymous" => {
                if row.get("manifest").is_some() {
                    problems.push(
                        "[invalid] anonymous must not declare a standalone manifest".to_string(),
                    );
                }
                if value_str(row, "parent_inventory") != Some("oya/community/manifest.json") {
                    problems.push(
                        "[invalid] anonymous parent_inventory must be oya/community/manifest.json"
                            .to_string(),
                    );
                }
                if value_str(row, "subproduct_of") != Some("community") {
                    problems
                        .push("[invalid] anonymous subproduct_of must be community".to_string());
                }
                continue;
            }
            "foundry" => {
                if value_str(row, "status") != Some("retired") {
                    problems.push("[invalid] foundry row must stay retired".to_string());
                }
                if value_str(row, "absorbed_by") != Some("intelligence") {
                    problems.push("[invalid] foundry absorbed_by must be intelligence".to_string());
                }
                if row.get("do_not_treat_as_active").and_then(Value::as_bool) != Some(true) {
                    problems
                        .push("[invalid] foundry do_not_treat_as_active must be true".to_string());
                }
            }
            _ => {}
        }

        let Some(manifest_path) = value_str(row, "manifest") else {
            problems.push(format!(
                "[invalid] {name} must declare manifest or explicit no-standalone semantics"
            ));
            continue;
        };
        if manifest_path.starts_with("microservices/") {
            problems.push(format!(
                "[legacy-path] {name} points at retired {manifest_path}; expected current oya/ or cloud/ inventory path"
            ));
            continue;
        }
        if !(manifest_path.starts_with("oya/") || manifest_path.starts_with("cloud/")) {
            problems.push(format!(
                "[invalid] {name} manifest path {manifest_path} must be under oya/ or cloud/"
            ));
            continue;
        }

        if name == "foundry" {
            // Retired provenance row intentionally points at the absorbing
            // intelligence manifest; do not require the manifest's own
            // `microservice` field to pretend foundry is still active.
            continue;
        }

        let full_path = repo_root.join(manifest_path);
        let manifest_text = match fs::read_to_string(&full_path) {
            Ok(text) => text,
            Err(e) => {
                problems.push(format!("[missing] {}: {e}", full_path.display()));
                continue;
            }
        };
        let manifest: Value = match serde_json::from_str(&manifest_text) {
            Ok(value) => value,
            Err(e) => {
                problems.push(format!("[invalid-json] {}: {e}", full_path.display()));
                continue;
            }
        };
        if value_str(&manifest, "schema_version") != Some("1.0") {
            problems.push(format!(
                "[invalid] {manifest_path} schema_version must be 1.0"
            ));
        }
        if value_str(&manifest, "microservice") != Some(name) {
            problems.push(format!(
                "[invalid] {manifest_path} microservice must be {name}"
            ));
        }
        if value_str(&manifest, "version") != Some("0.1.0") {
            problems.push(format!("[invalid] {manifest_path} version must be 0.1.0"));
        }
        active_manifest_paths_checked += 1;
    }

    if problems.is_empty() {
        Ok(CheckSummary {
            rows: expected_count,
            active_manifest_paths_checked,
        })
    } else {
        Err(problems)
    }
}

fn value_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}
