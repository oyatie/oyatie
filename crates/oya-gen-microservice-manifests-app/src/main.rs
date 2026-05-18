//! Thin CLI wrapper around [`oya_gen_microservice_manifests_app`] kernel.
//!
//! Walks `microservices/<ms>/` for the 32 canonical µservices, builds each
//! manifest in-memory, and writes:
//!   - microservices/<ms>/manifest.json
//!   - specs/microservices/manifests-index.json
//!
//! Flags:
//!  - `--repo-root <path>` (default `.`)
//!  - `--check` — recompute manifests in-memory and exit non-zero if any
//!    on-disk manifest differs (no writes).
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_gen_microservice_manifests_app::{
    MICROSERVICES, ManifestInputs, SourceFile, build_manifest, build_manifests_index,
};

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
                Ok(on_disk) => {
                    if on_disk != text {
                        differs += 1;
                        eprintln!("[diff] {}", path.display());
                    }
                }
                Err(e) => {
                    differs += 1;
                    eprintln!("[missing] {}: {e}", path.display());
                }
            }
        } else if let Err(e) = fs::write(&path, &text) {
            eprintln!("write {}: {e}", path.display());
            return ExitCode::FAILURE;
        } else {
            wrote += 1;
            println!("[ok] {}", path.display());
        }
    }

    if !check {
        let index = build_manifests_index(
            "2026-05-18",
            MICROSERVICES,
        );
        let mut idx = serde_json::to_string_pretty(&index).expect("serialize");
        idx.push('\n');
        let idx_path = repo_root.join("specs/microservices/manifests-index.json");
        if let Err(e) = fs::write(&idx_path, &idx) {
            eprintln!("write {}: {e}", idx_path.display());
            return ExitCode::FAILURE;
        }
        println!(
            "[ok] aggregate index → {} count={}",
            idx_path.display(),
            wrote
        );
    }

    if check {
        if differs == 0 {
            println!(
                "manifests --check: all {} byte-identical",
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

fn load_docs_decisions(repo_root: &Path) -> Result<Vec<String>, String> {
    let dir = repo_root.join("docs/decisions");
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false)
            && let Some(name) = entry.file_name().to_str() {
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
