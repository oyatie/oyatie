//! Every endpoint file the gate walks MUST parse as YAML.
//!
//! This is the fixture that makes the silent fallback removable. `extract_openapi_like_mutations`
//! degrades to a hand-rolled line scanner whenever `serde_yaml::from_str` returns `None`, and it
//! does so LOUDLY for the 9 files named `*.openapi.*` and SILENTLY for the other 54 the walker
//! accepts. A silent degrade under-reports endpoints, and under-reporting in this gate produces
//! zero findings — which is indistinguishable from a pass.
//!
//! So the fallback cannot be deleted on the argument that it is never taken; that has to be
//! measured. This test measures it, and fails with the offending paths if any file would take the
//! silent path.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !dir.join(".git").exists() {
        assert!(dir.pop(), "walked past the filesystem root without finding .git");
    }
    dir
}

/// Mirrors `is_public_api_file` in src/lib.rs — kept in sync deliberately, so a change to the
/// walker's admission rule shows up here as a diff rather than as silent drift.
fn is_endpoint_file(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('\\', "/");
    let lower = text.to_ascii_lowercase();
    let named = lower.ends_with(".openapi.yaml")
        || lower.ends_with(".openapi.yml")
        || lower.ends_with(".openapi.json");
    named || lower.contains("/contracts/openapi/") || lower.contains("/registry/openapi/")
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "target" || name == "third-party" || name == "buck-out" {
            continue;
        }
        if path.is_dir() {
            collect(&path, out);
        } else if is_endpoint_file(&path) {
            out.push(path);
        }
    }
}

#[test]
fn every_endpoint_file_parses_as_yaml() {
    let root = repo_root();
    let mut files = Vec::new();
    collect(&root, &mut files);
    assert!(
        !files.is_empty(),
        "found no endpoint files at all — the walker rule or the corpus moved, which is itself \
         the failure this test exists to catch"
    );

    let mut unparseable = Vec::new();
    for path in &files {
        let Ok(raw) = std::fs::read_to_string(path) else {
            unparseable.push((path.clone(), "unreadable".to_string()));
            continue;
        };
        if let Err(error) = serde_yaml::from_str::<serde_yaml::Value>(&raw) {
            unparseable.push((path.clone(), error.to_string()));
        }
    }

    assert!(
        unparseable.is_empty(),
        "{} of {} endpoint files do not parse as YAML. Each one currently takes the SILENT \
         line-scanner fallback in extract_openapi_like_mutations and can under-report endpoints \
         without producing a finding:\n{}",
        unparseable.len(),
        files.len(),
        unparseable
            .iter()
            .map(|(path, error)| format!(
                "  {} — {error}",
                path.strip_prefix(&root).unwrap_or(path).display()
            ))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
