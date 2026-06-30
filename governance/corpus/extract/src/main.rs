//! # corpus-extract binary (ADR-0580, Phase -1 corpus extractor spike)
//!
//! Deterministic CLI driver for the corpus extractor. Given a capability dir-prefix it:
//! 1. resolves the capability's crates via the workspace-members kernel,
//! 2. lists each crate's GIT-TRACKED `.rs` sources via `git ls-files` (no ambient walk),
//! 3. parses them with `syn` and emits the canonical [`FactSet`](corpus_core::FactSet) JSON,
//! 4. prints the OPAQUE-RATE report to stderr (so stdout stays a clean machine-readable fact set).
//!
//! Usage:
//! ```text
//! corpus-extract <repo_root> <capability_dir_prefix>
//! # e.g. corpus-extract . flags
//! ```
//!
//! The ONLY impurity is reading the committed tree (`git ls-files` + file reads); no clock/rand/net.
//! `git ls-files` is read-only and reports exactly the committed/tracked set, so the run is
//! reproducible from the repository state alone.
//!
//! ADR-0083 Tier-3: no unwrap/expect/panic; `#![forbid(unsafe_code)]`. Errors exit non-zero with a
//! message on stderr.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use corpus_extract::{
    CorpusExtraction, SourceFile, SourceSet, SynAstSource, extract_corpus, module_path_for,
    resolve_capability_crates,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let (repo_root, dir_prefix) = match args.as_slice() {
        [_, repo_root, dir_prefix] => (PathBuf::from(repo_root), dir_prefix.clone()),
        _ => {
            eprintln!("usage: corpus-extract <repo_root> <capability_dir_prefix>");
            return ExitCode::from(2);
        }
    };

    match run(&repo_root, &dir_prefix) {
        Ok(json) => {
            println!("{json}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("corpus-extract: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Resolve, extract, and render. Returns the canonical fact-set JSON (stdout) and prints the opaque
/// report to stderr as a side effect.
fn run(repo_root: &Path, dir_prefix: &str) -> Result<String, String> {
    let crate_dirs = resolve_capability_crates(repo_root, dir_prefix)
        .map_err(|e| format!("resolve capability crates: {e}"))?;
    if crate_dirs.is_empty() {
        return Err(format!(
            "capability prefix '{dir_prefix}' resolved zero workspace member crates"
        ));
    }

    let mut files: Vec<SourceFile> = Vec::new();
    for crate_dir in &crate_dirs {
        let crate_id = crate_cargo_name(repo_root, crate_dir)?;
        for rel in git_tracked_rust_sources(repo_root, crate_dir)? {
            let abs = repo_root.join(&rel);
            let source = std::fs::read_to_string(&abs)
                .map_err(|e| format!("read {}: {e}", abs.display()))?;
            let module_path = module_path_for(crate_dir, &rel);
            files.push(SourceFile {
                crate_id: crate_id.clone(),
                module_path,
                source,
            });
        }
    }

    let set = SourceSet::new(files);
    let extraction: CorpusExtraction =
        extract_corpus(&SynAstSource::new(), &set).map_err(|e| format!("extract corpus: {e}"))?;

    report_to_stderr(dir_prefix, &crate_dirs, &extraction);

    extraction
        .facts
        .canonical_json()
        .map_err(|e| format!("serialize fact set: {e}"))
}

/// Read a crate's de-branded cargo `name` from its `Cargo.toml` (the fact `crate_id`).
fn crate_cargo_name(repo_root: &Path, crate_dir: &str) -> Result<String, String> {
    let manifest = repo_root.join(crate_dir).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest)
        .map_err(|e| format!("read {}: {e}", manifest.display()))?;
    // Minimal, dependency-free `name = "..."` extraction from the `[package]` table. The corpus
    // extractor must not pull a TOML parser for one field; the first `name = "..."` line under
    // `[package]` is the cargo name by Cargo's own manifest grammar.
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package
            && let Some(rest) = trimmed.strip_prefix("name")
            && let Some(value) = rest.trim_start().strip_prefix('=')
        {
            let name = value.trim().trim_matches('"');
            if !name.is_empty() {
                return Ok(name.to_owned());
            }
        }
    }
    Err(format!("no [package].name in {}", manifest.display()))
}

/// List a crate's git-TRACKED `.rs` source files (repo-relative), sorted. Uses `git ls-files`
/// scoped to the crate dir — the committed set only, no ambient/untracked/ignored files.
fn git_tracked_rust_sources(repo_root: &Path, crate_dir: &str) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .arg("--")
        .arg(format!("{crate_dir}/*.rs"))
        .output()
        .map_err(|e| format!("spawn git ls-files: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("git ls-files output not utf-8: {e}"))?;
    let mut files: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    files.sort();
    Ok(files)
}

/// Print the OPAQUE-RATE report to stderr (human-readable; stdout stays the machine fact set).
fn report_to_stderr(dir_prefix: &str, crate_dirs: &[String], extraction: &CorpusExtraction) {
    let report = &extraction.report;
    eprintln!("corpus-extract report for capability '{dir_prefix}':");
    eprintln!("  crates ({}):", crate_dirs.len());
    for dir in crate_dirs {
        eprintln!("    {dir}");
    }
    eprintln!("  clean facts: {}", report.clean_facts);
    eprintln!("  opaque units: {}", report.opaque.len());
    eprintln!("  total units: {}", report.total_units());
    let bps = report.opaque_rate_bps();
    eprintln!("  opaque rate: {}.{:02}% ({bps} bps)", bps / 100, bps % 100);
    eprintln!("  opaque by category:");
    if report.by_category.is_empty() {
        eprintln!("    (none)");
    } else {
        for (category, count) in &report.by_category {
            eprintln!("    {category}: {count}");
        }
    }
    if !report.opaque.is_empty() {
        eprintln!("  opaque detail:");
        for reason in &report.opaque {
            eprintln!("    [{}] {}", reason.category(), reason.detail());
        }
    }
}
