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

use corpus_core::{CoveragePolicy, evaluate_coverage};
use corpus_extract::{
    CorpusExtraction, SourceFile, SourceSet, SynAstSource, extract_corpus, module_path_for,
    resolve_capability_crates,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let (repo_root, dir_prefix, ceiling) = match args.as_slice() {
        [_, repo_root, dir_prefix] => (PathBuf::from(repo_root), dir_prefix.clone(), None),
        [_, repo_root, dir_prefix, ceiling] => match ceiling.parse::<usize>() {
            Ok(ceiling) => (PathBuf::from(repo_root), dir_prefix.clone(), Some(ceiling)),
            Err(error) => {
                eprintln!("corpus-extract: unindexed-target ceiling must be an integer: {error}");
                return ExitCode::from(2);
            }
        },
        _ => {
            eprintln!(
                "usage: corpus-extract <repo_root> <capability_dir_prefix> \
                 [unindexed_target_ceiling]"
            );
            return ExitCode::from(2);
        }
    };

    match run(&repo_root, &dir_prefix, ceiling) {
        Ok(Outcome { json, blocked }) => {
            println!("{json}");
            // A ratchet that only ever prints is advisory forever. Exiting non-zero on a BLOCKING
            // finding is what lets a caller (a CI lane, a buck2 action) enforce it without
            // reimplementing the evaluation — and is why the ceiling is an argument rather than a
            // number frozen inside the binary.
            if blocked {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(message) => {
            eprintln!("corpus-extract: {message}");
            ExitCode::FAILURE
        }
    }
}

/// What a run produced: the canonical fact-set JSON, and whether the coverage ratchet blocked.
struct Outcome {
    json: String,
    blocked: bool,
}

/// Resolve, extract, and render. Returns the canonical fact-set JSON (stdout) and prints the opaque
/// report to stderr as a side effect.
fn run(repo_root: &Path, dir_prefix: &str, ceiling: Option<usize>) -> Result<Outcome, String> {
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
                // `git ls-files` already yields REPO-relative paths, which is exactly the File-node
                // container identity the graph requires.
                path: rel,
                module_path,
                source,
            });
        }
    }

    let set = SourceSet::new(files);
    let extraction: CorpusExtraction =
        extract_corpus(&SynAstSource::new(), &set).map_err(|e| format!("extract corpus: {e}"))?;

    report_to_stderr(dir_prefix, &crate_dirs, &extraction);
    let blocked = evaluate_ratchet(&extraction, ceiling);

    let json = extraction
        .facts
        .canonical_json()
        .map_err(|e| format!("serialize fact set: {e}"))?;
    Ok(Outcome { json, blocked })
}

/// Evaluate the edge-coverage ratchet and report every finding. Returns whether any finding blocks.
///
/// The anti-vacuity floor is DERIVED, never asserted: a run that resolved facts but produced zero
/// `Refs` targets means the reference pass collapsed, and its "nothing dangles" result is
/// meaningless rather than perfect. With no ceiling supplied the ratchet is ADVISORY — every
/// dangling target is still reported, but nothing fails — which is how it is born.
fn evaluate_ratchet(extraction: &CorpusExtraction, ceiling: Option<usize>) -> bool {
    let coverage = extraction.graph.coverage();
    let policy = CoveragePolicy {
        baseline_unindexed_targets: ceiling.unwrap_or(coverage.unindexed_targets()),
        min_expected_targets: usize::from(!extraction.facts.is_empty()),
    };
    let findings = evaluate_coverage(&extraction.graph, &policy);
    let blocking: Vec<_> = findings.iter().filter(|finding| finding.blocking).collect();
    eprintln!(
        "  coverage ratchet (ceiling {}{}):",
        policy.baseline_unindexed_targets,
        if ceiling.is_some() {
            ""
        } else {
            ", ADVISORY — no ceiling supplied"
        }
    );
    let mut dangling = 0usize;
    for finding in &findings {
        // Dangling targets are already listed above as the unindexed sample; printing each one
        // twice would bury the codes that decide the verdict. They are COUNTED here so an empty
        // section can never be mistaken for a clean one.
        if finding.code == corpus_core::CODE_EDGE_DANGLING_TARGET {
            dangling += 1;
        } else {
            eprintln!("    [{}] {}", finding.code, finding.detail);
        }
    }
    eprintln!(
        "    [{}] x{dangling}",
        corpus_core::CODE_EDGE_DANGLING_TARGET
    );
    eprintln!(
        "    verdict: {}",
        if blocking.is_empty() {
            "PASS"
        } else {
            "BLOCKED"
        }
    );
    !blocking.is_empty()
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

    // The graph + the COVERAGE RATCHET input. Every number here is counted from the graph that was
    // just built; none is asserted.
    let graph = &extraction.graph;
    let coverage = graph.coverage();
    eprintln!("  graph nodes: {}", graph.nodes().len());
    eprintln!("  graph edges: {}", graph.edges().len());
    eprintln!(
        "  edge-target coverage: {}/{} indexed ({}.{:02}%)",
        coverage.indexed_targets,
        coverage.total_targets,
        coverage.rate_bps() / 100,
        coverage.rate_bps() % 100
    );
    eprintln!("  unindexed edge targets: {}", coverage.unindexed_targets());
    // Print a bounded SAMPLE of the unindexed set, not just its size. A coverage number nobody can
    // audit is an assertion wearing a measurement's clothes: the sample is what lets a reader see
    // whether the miss is real (a genuinely absent definition) or an artifact of the extractor's
    // missing import resolution (`Vec`, `String`, an unqualified intra-module call).
    let unresolved = graph.unresolved_targets();
    for target in unresolved.iter().take(20) {
        eprintln!("    unindexed: {}::{}", target.container, target.path);
    }
    if unresolved.len() > 20 {
        eprintln!("    ... and {} more", unresolved.len() - 20);
    }
    let duplicates = graph.duplicate_containers();
    eprintln!("  byte-identical file families: {}", duplicates.len());
    for (digest, containers) in duplicates.iter().take(10) {
        eprintln!(
            "    {} x{}: {}",
            &digest.as_str()[..8],
            containers.len(),
            containers.join(", ")
        );
    }
}
