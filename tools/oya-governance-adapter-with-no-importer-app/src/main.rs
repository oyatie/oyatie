//! Foundry adapter-with-no-importer fitness dev-CLI.
//!
//! Walks `crates/` (and `tools/`) for `Cargo.toml` manifests, extracts the
//! `[package].name` from each, and feeds the records into the kernel
//! [`check`]. Exits with code 0 when the workspace has no adapter crate
//! lacking an importer; non-zero otherwise.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use oya_governance_adapter_with_no_importer_kernel::{
    AdapterImporterReport, WorkspaceCrate, check,
};

const DEFAULT_CRATES_ROOT: &str = "crates";
const DEFAULT_TOOLS_ROOT: &str = "tools";

fn main() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(report) => {
            println!(
                "adapter-with-no-importer ok: adapters_checked={} importers_observed={} violations=0",
                report.adapters_checked, report.importers_observed,
            );
            ExitCode::SUCCESS
        }
        Err(LaneError::Violations(report)) => {
            eprintln!(
                "adapter-with-no-importer FAIL: adapters_checked={} importers_observed={} violations={}",
                report.adapters_checked,
                report.importers_observed,
                report.violations.len(),
            );
            for violation in &report.violations {
                eprintln!(
                    "  - {} expected={} hint={}",
                    violation.adapter_crate, violation.expected_importer_pattern, violation.hint,
                );
            }
            ExitCode::FAILURE
        }
        Err(LaneError::Io(message)) => {
            eprintln!("adapter-with-no-importer error: {message}");
            ExitCode::FAILURE
        }
    }
}

enum LaneError {
    Violations(AdapterImporterReport),
    Io(String),
}

fn run<I>(args: I) -> Result<AdapterImporterReport, LaneError>
where
    I: IntoIterator<Item = String>,
{
    let options = Options::parse(args).map_err(LaneError::Io)?;
    let mut workspace = Vec::new();
    for root in &options.roots {
        let crates = scan_root(root).map_err(LaneError::Io)?;
        workspace.extend(crates);
    }
    workspace.sort_by(|a, b| a.name.cmp(&b.name));
    let report = check(&workspace);
    if report.is_clean() {
        Ok(report)
    } else {
        Err(LaneError::Violations(report))
    }
}

struct Options {
    roots: Vec<PathBuf>,
}

impl Options {
    fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut roots: Vec<PathBuf> = Vec::new();
        let args = args.into_iter().collect::<Vec<_>>();
        let mut index = 0usize;
        while index < args.len() {
            match args[index].as_str() {
                "--root" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| "--root requires a path".to_string())?;
                    roots.push(PathBuf::from(value));
                }
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unexpected argument '{other}'\n{}", usage())),
            }
            index += 1;
        }
        if roots.is_empty() {
            roots.push(PathBuf::from(DEFAULT_CRATES_ROOT));
            roots.push(PathBuf::from(DEFAULT_TOOLS_ROOT));
        }
        Ok(Self { roots })
    }
}

fn usage() -> String {
    "usage: oya-governance-adapter-with-no-importer-app [--root PATH]...".into()
}

fn scan_root(root: &Path) -> Result<Vec<WorkspaceCrate>, String> {
    let mut out = Vec::new();
    if !root.exists() {
        // Tolerate missing optional roots (e.g., a workspace with only `crates/`).
        return Ok(out);
    }
    let entries = fs::read_dir(root)
        .map_err(|error| format!("could not read directory {}: {error}", root.display()))?;
    for entry in entries {
        let entry = entry
            .map_err(|error| format!("directory entry error under {}: {error}", root.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("file_type error: {error}"))?;
        if !file_type.is_dir() {
            continue;
        }
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let contents = fs::read_to_string(&manifest)
            .map_err(|error| format!("could not read {}: {error}", manifest.display()))?;
        let Some(name) = extract_package_name(&contents) else {
            continue;
        };
        out.push(WorkspaceCrate {
            name,
            manifest_path: manifest.to_string_lossy().into_owned(),
        });
    }
    Ok(out)
}

/// Minimal `[package].name` extractor — avoids pulling toml as a dependency
/// for a fitness lane that only needs one scalar field. Accepts the
/// canonical form used by every Cargo manifest in the workspace:
///   `name = "oya-foo-bar"`
fn extract_package_name(manifest: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if let Some(section) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_package = section.trim() == "package";
            continue;
        }
        if !in_package {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(value) = rest.strip_prefix('=') {
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim()
                    .to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_package_name() {
        let manifest = r#"
[package]
name = "oya-governance-adapter-with-no-importer-kernel"
edition.workspace = true
"#;
        assert_eq!(
            extract_package_name(manifest).as_deref(),
            Some("oya-governance-adapter-with-no-importer-kernel")
        );
    }

    #[test]
    fn ignores_name_outside_package_section() {
        let manifest = r#"
[lib]
name = "not_the_package_name"

[package]
name = "real-name"
"#;
        assert_eq!(extract_package_name(manifest).as_deref(), Some("real-name"));
    }

    #[test]
    fn returns_none_for_manifest_without_package_name() {
        assert!(extract_package_name("[workspace]\nmembers = []\n").is_none());
    }
}
