//! Thin CLI wrapper around [`oya_gen_microservice_manifests_app`] kernel.
//!
//! Walks `microservices/<ms>/` for the canonical µservices and builds the
//! source-derived manifest seed in-memory. Existing enriched manifests are
//! validated for source compatibility instead of overwritten; missing manifests
//! are seeded, and the aggregate index is refreshed.
//!
//! Flags:
//!  - `--repo-root <path>` (default `.`)
//!  - `--check` — recompute source-derived manifest fields and exit non-zero if
//!    an on-disk enriched manifest is incompatible (no writes).
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;

use oya_gen_microservice_manifests_app::{
    MICROSERVICES, ManifestInputs, SourceFile, build_manifest, build_manifests_index,
};

const MANIFESTS_INDEX_GENERATED_AT: &str = "2026-05-19";

// New FD-001 services must prove that their source files still match the
// enriched manifest rows for facts this generator derives directly. Legacy
// manifests remain source-compatible on registry/contract identity while their
// historical enrichment drift is migrated lane-by-lane.
const STRICT_SOURCE_FIDELITY_SERVICES: &[&str] = &["ops-dashboard-control-center"];

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

    let decisions = match load_docs_decisions(&repo_root) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let mut differs = 0usize;
    let mut wrote = 0usize;
    let mut validated = 0usize;
    for ms in MICROSERVICES {
        let inputs = match collect_ms_inputs(&repo_root, ms, &decisions) {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("{msg}");
                return ExitCode::FAILURE;
            }
        };
        let manifest = build_manifest(&inputs);
        let mut text = serde_json::to_string_pretty(&manifest).expect("serialize");
        text.push('\n');
        let path = repo_root.join(format!("microservices/{ms}/manifest.json"));
        if check {
            match fs::read_to_string(&path) {
                Ok(on_disk) => match serde_json::from_str::<Value>(&on_disk) {
                    Ok(on_disk_json) => {
                        let problems = manifest_compatibility_problems(&on_disk_json, &manifest);
                        if !problems.is_empty() {
                            differs += 1;
                            eprintln!("[incompatible] {}", path.display());
                            for problem in problems {
                                eprintln!("  - {problem}");
                            }
                        }
                    }
                    Err(e) => {
                        differs += 1;
                        eprintln!("[invalid-json] {}: {e}", path.display());
                    }
                },
                Err(e) => {
                    differs += 1;
                    eprintln!("[missing] {}: {e}", path.display());
                }
            }
        } else if path.exists() {
            match fs::read_to_string(&path) {
                Ok(on_disk) => match serde_json::from_str::<Value>(&on_disk) {
                    Ok(on_disk_json) => {
                        let problems = manifest_compatibility_problems(&on_disk_json, &manifest);
                        if problems.is_empty() {
                            validated += 1;
                            println!("[ok] source-compatible existing {}", path.display());
                        } else {
                            eprintln!("[incompatible] {}", path.display());
                            for problem in problems {
                                eprintln!("  - {problem}");
                            }
                            return ExitCode::FAILURE;
                        }
                    }
                    Err(e) => {
                        eprintln!("[invalid-json] {}: {e}", path.display());
                        return ExitCode::FAILURE;
                    }
                },
                Err(e) => {
                    eprintln!("read {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
        } else if let Err(e) = fs::write(&path, &text) {
            eprintln!("write {}: {e}", path.display());
            return ExitCode::FAILURE;
        } else {
            wrote += 1;
            println!("[ok] seeded missing {}", path.display());
        }
    }

    let index = build_manifests_index(MANIFESTS_INDEX_GENERATED_AT, MICROSERVICES);
    if check {
        let idx_path = repo_root.join("specs/microservices/manifests-index.json");
        match fs::read_to_string(&idx_path) {
            Ok(on_disk) => {
                let mut expected = serde_json::to_string_pretty(&index).expect("serialize");
                expected.push('\n');
                if on_disk != expected {
                    differs += 1;
                    eprintln!("[diff] {}", idx_path.display());
                }
            }
            Err(e) => {
                differs += 1;
                eprintln!("[missing] {}: {e}", idx_path.display());
            }
        }
    } else {
        let mut idx = serde_json::to_string_pretty(&index).expect("serialize");
        idx.push('\n');
        let idx_path = repo_root.join("specs/microservices/manifests-index.json");
        if let Err(e) = fs::write(&idx_path, &idx) {
            eprintln!("write {}: {e}", idx_path.display());
            return ExitCode::FAILURE;
        }
        println!(
            "[ok] aggregate index → {} written={} source_compatible_existing={}",
            idx_path.display(),
            wrote,
            validated
        );
    }

    if check {
        if differs == 0 {
            println!(
                "manifests --check: all {} source-compatible",
                MICROSERVICES.len()
            );
            ExitCode::SUCCESS
        } else {
            eprintln!("manifests --check: {differs} drift");
            ExitCode::FAILURE
        }
    } else {
        ExitCode::SUCCESS
    }
}

fn manifest_compatibility_problems(on_disk: &Value, generated: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    for key in ["schema_version", "microservice", "version"] {
        if on_disk.get(key) != generated.get(key) {
            problems.push(format!("{key} mismatch"));
        }
    }

    // All manifests must remain registered and point at source-discovered
    // contract files. FD-001-era manifests additionally opt into strict
    // source-fidelity checks for capability and SLO facts so check mode cannot
    // false-green tier/risk/query drift while legacy enrichment debt is migrated
    // service by service.
    compare_contract_paths(on_disk, generated, &mut problems);
    if strict_source_fidelity_required(on_disk) {
        compare_rows_by_file(
            "capabilities",
            &["name", "tier", "eu_ai_act_risk_class"],
            on_disk,
            generated,
            &mut problems,
        );
        compare_rows_by_file(
            "slos",
            &["name", "target", "sli"],
            on_disk,
            generated,
            &mut problems,
        );
    }

    problems
}

fn strict_source_fidelity_required(on_disk: &Value) -> bool {
    on_disk
        .get("microservice")
        .and_then(Value::as_str)
        .map(|ms| STRICT_SOURCE_FIDELITY_SERVICES.contains(&ms))
        .unwrap_or(false)
}

fn compare_contract_paths(on_disk: &Value, generated: &Value, problems: &mut Vec<String>) {
    for family in ["openapi", "asyncapi", "proto"] {
        let expected = string_array(&generated["contracts"][family]);
        if expected.is_empty() {
            continue;
        }
        let actual = string_array(&on_disk["contracts"][family]);
        for item in expected {
            if !actual.contains(&item) {
                problems.push(format!("contracts.{family} missing {item}"));
            }
        }
    }
}

fn compare_rows_by_file(
    section: &str,
    fields: &[&str],
    on_disk: &Value,
    generated: &Value,
    problems: &mut Vec<String>,
) {
    let Some(expected_rows) = generated.get(section).and_then(Value::as_array) else {
        return;
    };
    if expected_rows.is_empty() {
        return;
    }
    let Some(actual_rows) = on_disk.get(section).and_then(Value::as_array) else {
        problems.push(format!("{section} missing"));
        return;
    };
    for expected in expected_rows {
        let file = expected
            .get("file")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if file.is_empty() {
            continue;
        }
        let Some(actual) = actual_rows
            .iter()
            .find(|row| row.get("file").and_then(Value::as_str) == Some(file))
        else {
            problems.push(format!("{section} missing row for {file}"));
            continue;
        };
        for field in fields {
            if actual.get(*field) != expected.get(*field) {
                problems.push(format!("{section}.{file}.{field} mismatch"));
            }
        }
    }
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn load_docs_decisions(repo_root: &Path) -> Result<Vec<String>, String> {
    let dir = repo_root.join("docs/decisions");
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false)
            && let Some(name) = entry.file_name().to_str()
        {
            out.push(name.to_string());
        }
    }
    Ok(out)
}

fn collect_ms_inputs(
    repo_root: &Path,
    ms: &str,
    decisions: &[String],
) -> Result<ManifestInputs, String> {
    let dir = repo_root.join(format!("microservices/{ms}"));
    let mut files: Vec<SourceFile> = Vec::new();
    if dir.is_dir() {
        walk(&dir, repo_root, &mut files)?;
    }
    Ok(ManifestInputs {
        microservice: ms.to_string(),
        files,
        docs_decisions_filenames: decisions.to_vec(),
    })
}

fn walk(dir: &Path, repo_root: &Path, out: &mut Vec<SourceFile>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, repo_root, out)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(rel) = path.strip_prefix(repo_root) else {
            continue;
        };
        out.push(SourceFile {
            repo_relative_path: rel.to_string_lossy().to_string(),
            content: text,
        });
    }
    Ok(())
}
