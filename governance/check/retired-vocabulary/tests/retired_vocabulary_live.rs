//! Live-tree caller for the retired-vocabulary fitness gate.
//!
//! The kernel is pure; this walks the REAL corpus and hands it observations as DATA. No frozen
//! violation set is maintained: the kernel's contract is `Ok(report)` iff `violations.is_empty()`
//! -- a hard invariant, not a shrink-only ratchet, matching the crate's own stated philosophy
//! ("once a term is retired, CI refuses any future re-introduction"). If a real hit ever
//! appears, the correct response is to FIX the drift (repoint the mention at the canonical
//! replacement), not to freeze it -- freezing would contradict the doctrine's own reason to
//! exist.
//!
//! Corpus roots and historical exclusions MIRROR
//! `marketplace/facade/dev-cli/src/retired_vocabulary_gate.rs` DEFAULT_CORPUS_ROOTS /
//! DEFAULT_EXCLUDE_ROOTS exactly (that runner's own scope decisions, already reasoned through
//! and comment-justified there) rather than reimplementing the reasoning independently. That
//! runner is `pub(crate)` inside marketplace-dev-cli and not reachable from this crate's tests,
//! so the roots are duplicated as data here; if the dev-cli runner's roots ever change, this
//! test's roots must be re-synced in the same PR or the two silently diverge.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use check_retired_vocabulary::{RetiredTerm, ScannedDocument, validate_retired_vocabulary};

const REGISTRY_PATH: &str = "registry/vocabulary/retired.yaml";

const CORPUS_ROOTS: &[&str] = &["docs", "registry", "templates", "scripts", ".github"];

const EXCLUDE_PATHS: &[&str] = &[
    "evidence/audits",
    "docs/CHANGELOG.md",
    "docs/plans",
    "docs/decisions",
    "docs/adr-archive",
    "registry/fixuptasks.jsonl",
    "registry/vocabulary/retired.yaml",
    ".grit",
    ".omc",
    ".omx",
    "target",
];

fn repo_root() -> PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    for _ in 0..16 {
        if dir.join(REGISTRY_PATH).is_file() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    panic!("failed to locate repo root (the dir holding {REGISTRY_PATH})");
}

/// Hand-parsed rather than a YAML crate dependency: the schema is small, flat, and fixed
/// (four string fields per row, `adr` optionally blank), and this crate carries no YAML
/// dependency today. Each `- term:` line starts a new row; subsequent `key: value` lines
/// (more-indented) belong to it until the next `- term:` or end of file.
fn parse_registry(raw: &str) -> Vec<RetiredTerm> {
    let mut terms = Vec::new();
    let mut current: Option<(String, String, String, Option<String>)> = None;

    fn flush(
        current: Option<(String, String, String, Option<String>)>,
        out: &mut Vec<RetiredTerm>,
    ) {
        if let Some((term, retired_at, canonical_replacement, adr)) = current {
            out.push(RetiredTerm {
                term,
                retired_at,
                canonical_replacement,
                adr,
            });
        }
    }

    fn quoted_value(line: &str) -> String {
        let (_, value) = line.split_once(':').expect("key: value line");
        let value = value.trim();
        value.trim_matches('"').to_owned()
    }

    for line in raw.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- term:") {
            flush(current.take(), &mut terms);
            current = Some((
                rest.trim().trim_matches('"').to_owned(),
                String::new(),
                String::new(),
                None,
            ));
        } else if let Some(current) = current.as_mut() {
            if trimmed.starts_with("retired_at:") {
                current.1 = quoted_value(trimmed);
            } else if trimmed.starts_with("canonical_replacement:") {
                current.2 = quoted_value(trimmed);
            } else if trimmed.starts_with("adr:") {
                let v = quoted_value(trimmed);
                current.3 = if v.is_empty() { None } else { Some(v) };
            }
        }
    }
    flush(current, &mut terms);
    terms
}

fn is_excluded(relative: &str) -> bool {
    EXCLUDE_PATHS
        .iter()
        .any(|excluded| relative == *excluded || relative.starts_with(&format!("{excluded}/")))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("path under root")
            .to_str()
            .expect("utf8 path")
            .replace('\\', "/");
        if is_excluded(&relative) {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, out);
        } else {
            out.push(path);
        }
    }
}

#[test]
fn live_corpus_has_no_retired_vocabulary_drift() {
    let root = repo_root();

    let registry_raw = std::fs::read_to_string(root.join(REGISTRY_PATH))
        .unwrap_or_else(|error| panic!("read {REGISTRY_PATH}: {error}"));
    let terms = parse_registry(&registry_raw);
    assert!(
        terms.len() >= 6,
        "observed {} retired terms, below the floor of 6 -- the registry parse collapsed or \
         the file moved",
        terms.len()
    );

    let mut paths = Vec::new();
    for root_name in CORPUS_ROOTS {
        collect_files(&root, &root.join(root_name), &mut paths);
    }
    paths.sort();

    const MIN_EXPECTED_FILES: usize = 200;
    assert!(
        paths.len() >= MIN_EXPECTED_FILES,
        "observed {} candidate files across {CORPUS_ROOTS:?}, below the floor of \
         {MIN_EXPECTED_FILES} -- the walk collapsed or a scope root moved",
        paths.len()
    );

    let contents: Vec<(PathBuf, String)> = paths
        .iter()
        .filter_map(|path| {
            std::fs::read_to_string(path)
                .ok()
                .map(|c| (path.clone(), c))
        })
        .collect();

    let documents = contents.iter().map(|(path, contents)| ScannedDocument {
        path: path.to_str().expect("utf8 path"),
        contents: contents.as_str(),
    });

    match validate_retired_vocabulary(&terms, documents) {
        Ok(report) => {
            assert!(
                report.documents_checked >= MIN_EXPECTED_FILES,
                "kernel checked {} documents, expected at least {MIN_EXPECTED_FILES}",
                report.documents_checked
            );
        }
        Err(error) => panic!(
            "{error}\n\
             Each hit either names a real drift back to a retired surface (fix it: repoint the \
             mention at the canonical replacement in the same change) or belongs in \
             EXCLUDE_PATHS above as a historical record (and the sibling constant in \
             marketplace/facade/dev-cli/src/retired_vocabulary_gate.rs::DEFAULT_EXCLUDE_ROOTS, \
             in the same change, so the two do not diverge)."
        ),
    }
}

#[test]
fn registry_parses_the_six_known_terms_exactly() {
    let root = repo_root();
    let raw = std::fs::read_to_string(root.join(REGISTRY_PATH)).expect("read registry");
    let terms = parse_registry(&raw);
    let names: Vec<&str> = terms.iter().map(|t| t.term.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "repoctl pre-push",
            "oya dev check",
            "scripts/check.sh",
            "scripts/check-architecture-boundaries.sh",
            "scripts/hooks/pre-push-repoctl.sh",
            ".omc/hooks/grit-claim-intent-gate.sh",
        ],
        "registry term list changed -- update this pinned list in the same change so a \
         silently-added or silently-removed row cannot pass unnoticed"
    );
    for term in &terms {
        assert!(
            !term.canonical_replacement.is_empty(),
            "{} has no canonical_replacement",
            term.term
        );
    }
}
