//! `oya gate validate retired-vocabulary` runner.
//!
//! Reads `registry/vocabulary/retired.yaml` and walks the configured
//! corpus roots (default: `docs/`, `registry/`, `templates/`,
//! `crates/`, `tools/`, `scripts/`), passing each file to the
//! [`check_retired_vocabulary`] kernel.
//!
//! Lane id: `oya-governance-retired-vocabulary`. The lane fails
//! fast on any drift hit, surfacing every file:line:term row at once
//! so the fix-up commit covers them in a single sweep.
//!
//! Naming justification: module file is snake_case, no redundant
//! suffix; functions follow the existing
//! `parse_<lane>_validate_args` / `validate_<lane>_gate` naming
//! used by every other gate in this crate.

use std::fs;
use std::path::{Path, PathBuf};

use check_retired_vocabulary::{
    RetiredTerm, RetiredVocabularyError, RetiredVocabularyReport, ScannedDocument,
    validate_retired_vocabulary,
};

use crate::yaml_scan::{clean_yaml_value, parse_yaml_inline_values};

const USAGE: &str = "oya gate validate retired-vocabulary \
                     [--registry <registry/vocabulary/retired.yaml>] \
                     [--corpus-root <path>] (repeatable) \
                     [--exclude-root <path>] (repeatable)";

const DEFAULT_REGISTRY_PATH: &str = "registry/vocabulary/retired.yaml";

// User-facing documentation surfaces. Rust source comments under
// `crates/` and `tools/` are NOT scanned by default — those comments
// legitimately document code by naming what it replaced (e.g. a kernel
// docstring that says "swaps the canonical command constant from
// `repoctl pre-push` to `oya verify`" is historical context, not drift
// back to the retired surface). Callers who want code-comment coverage
// can opt in via `--corpus-root crates --corpus-root tools`.
const DEFAULT_CORPUS_ROOTS: &[&str] = &["docs", "registry", "templates", "scripts", ".github"];

// Historical-record paths: each subtree intentionally names retired
// terms because its purpose IS to record past state. Excluding them
// keeps the lane focused on active drift and avoids CI noise on
// docs whose value is precisely their historical accuracy.
//
// - `evidence/audits/` — audit reports of past retirements.
// - `docs/CHANGELOG.md` — changelog entries reference what was retired.
// - `docs/plans/` — historical implementation plans.
// - `docs/decisions/` — ADRs document decisions at a point in time;
//   amending them in-place breaks ADR doctrine. New decisions
//   reference the canonical replacement.
// - `docs/adr-archive/` — the SAME reasoning as `docs/decisions/`, one
//   directory over. An archived ADR is still a decision record at a
//   point in time (its own frontmatter carries a "HISTORICAL /
//   NON-AUTHORITY" banner, not a license to rewrite it); 5 archived
//   ADRs legitimately cite a retired term while explaining the
//   retirement itself (e.g. "`scripts/check.sh` (descended from
//   legacy boundary-validator ancestry; here-canonical)"). This entry
//   was missing -- `docs/decisions/` was added when ADRs lived in one
//   directory, and the archive split off later without the exclusion
//   following it.
// - `registry/fixuptasks.jsonl` — a structured task LEDGER, not free
//   prose; two of its rows are CLOSED remediation tasks whose `result`
//   field is itself the retirement evidence ("Legacy scripts/check.sh
//   is absent on dev"). Scanning a completed-task record for the term
//   it retired is the same category error `evidence/audits/` exists
//   to prevent, one root over.
const DEFAULT_EXCLUDE_ROOTS: &[&str] = &[
    "evidence/audits",
    "docs/CHANGELOG.md",
    "docs/plans",
    "docs/decisions",
    "docs/adr-archive",
    "registry/fixuptasks.jsonl",
    ".grit",
    ".omc",
    ".omx",
    "target",
    // Substring-grep over Rust source under governance/check/retired-vocabulary/
    // would match the kernel's own test fixtures (which intentionally embed
    // retired terms as string literals to test the kernel itself). The kernel
    // crate is self-validating; excluding it avoids meta-flagging.
    //
    // Both this entry and the runner entry below carried a `crates/` prefix that
    // matched nothing after the crates moved out of it, so neither exclusion was
    // live. They are re-anchored on the real paths here.
    "governance/check/retired-vocabulary",
    // The runner module embeds retired terms in YAML-parser tests + comment
    // examples. Same self-validating reasoning as the kernel crate above.
    "marketplace/facade/dev-cli/src/retired_vocabulary_gate.rs",
    // The registry file IS the canonical record of retired terms; it must
    // spell each one verbatim. Excluding it prevents trivial self-match.
    "registry/vocabulary/retired.yaml",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetiredVocabularyValidateArgs {
    pub registry_path: PathBuf,
    pub corpus_roots: Vec<PathBuf>,
    pub exclude_roots: Vec<PathBuf>,
}

impl Default for RetiredVocabularyValidateArgs {
    fn default() -> Self {
        Self {
            registry_path: PathBuf::from(DEFAULT_REGISTRY_PATH),
            corpus_roots: DEFAULT_CORPUS_ROOTS.iter().map(PathBuf::from).collect(),
            exclude_roots: DEFAULT_EXCLUDE_ROOTS.iter().map(PathBuf::from).collect(),
        }
    }
}

pub(crate) fn parse_retired_vocabulary_validate_args(
    args: Vec<String>,
) -> Result<RetiredVocabularyValidateArgs, String> {
    let mut parsed = RetiredVocabularyValidateArgs::default();
    let mut user_set_corpus_roots = false;
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--registry" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.registry_path = PathBuf::from(value);
            }
            "--corpus-root" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                if !user_set_corpus_roots {
                    parsed.corpus_roots.clear();
                    user_set_corpus_roots = true;
                }
                parsed.corpus_roots.push(PathBuf::from(value));
            }
            "--exclude-root" => {
                let Some(value) = iter.next() else {
                    return Err(USAGE.to_owned());
                };
                parsed.exclude_roots.push(PathBuf::from(value));
            }
            _ => return Err(USAGE.to_owned()),
        }
    }
    Ok(parsed)
}

pub(crate) fn validate_retired_vocabulary_gate(
    args: RetiredVocabularyValidateArgs,
) -> Result<RetiredVocabularyReport, String> {
    let registry_text = fs::read_to_string(&args.registry_path).map_err(|error| {
        format!(
            "could not read retired-vocabulary registry at {}: {error}",
            args.registry_path.display()
        )
    })?;
    let terms = parse_retired_registry(&registry_text).map_err(|error| {
        format!(
            "could not parse retired-vocabulary registry at {}: {error}",
            args.registry_path.display()
        )
    })?;

    let mut document_paths: Vec<PathBuf> = Vec::new();
    for root in &args.corpus_roots {
        collect_text_files(root, &args.exclude_roots, &mut document_paths)?;
    }
    document_paths.sort();

    let documents: Vec<(PathBuf, String)> = document_paths
        .into_iter()
        .map(|path| {
            let contents = fs::read_to_string(&path).map_err(|error| {
                format!("could not read corpus file {}: {error}", path.display())
            })?;
            Ok((path, contents))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let scanned: Vec<ScannedDocument<'_>> = documents
        .iter()
        .map(|(path, contents)| ScannedDocument {
            path: path.to_str().unwrap_or("<non-utf8-path>"),
            contents: contents.as_str(),
        })
        .collect();

    validate_retired_vocabulary(&terms, scanned).map_err(|error| match error {
        RetiredVocabularyError::ViolationsFound(_) => error.to_string(),
        other => format!("retired-vocabulary registry is malformed: {other}"),
    })
}

fn parse_retired_registry(yaml_text: &str) -> Result<Vec<RetiredTerm>, String> {
    let mut terms: Vec<RetiredTerm> = Vec::new();
    let mut current: Option<RetiredTerm> = None;
    let mut in_retired_array = false;

    for raw_line in yaml_text.lines() {
        let line = raw_line.trim_end();
        let stripped = line.trim_start();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        if stripped == "retired:" {
            in_retired_array = true;
            continue;
        }
        if !in_retired_array {
            continue;
        }
        // Row start: a line like `  - term: "..."` opens a new entry.
        if let Some(rest) = stripped.strip_prefix("- term:") {
            if let Some(prev) = current.take() {
                terms.push(prev);
            }
            current = Some(RetiredTerm {
                term: clean_yaml_value(rest.trim()).to_string(),
                retired_at: String::new(),
                canonical_replacement: String::new(),
                adr: None,
            });
            continue;
        }
        let Some(entry) = current.as_mut() else {
            continue;
        };
        if let Some(rest) = stripped.strip_prefix("retired_at:") {
            entry.retired_at = clean_yaml_value(rest.trim()).to_string();
        } else if let Some(rest) = stripped.strip_prefix("canonical_replacement:") {
            entry.canonical_replacement = clean_yaml_value(rest.trim()).to_string();
        } else if let Some(rest) = stripped.strip_prefix("adr:") {
            let value = clean_yaml_value(rest.trim());
            entry.adr = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        } else if let Some(_rest) = stripped.strip_prefix("term:") {
            // A bare `term:` line after the row was opened indicates a
            // malformed entry — `- term:` opens, plain `term:` is reserved
            // as future syntax. Surface it loud so registry hand-edits
            // don't silently drop rows.
            return Err(format!(
                "unexpected `term:` line outside of row start: `{line}` — \
                 each row must begin with `- term: …`"
            ));
        } else {
            // Permit `_inline_values` syntax for forward compat (currently unused).
            let _ = parse_yaml_inline_values(stripped);
        }
    }
    if let Some(last) = current.take() {
        terms.push(last);
    }
    if terms.is_empty() {
        return Err("retired-vocabulary registry has zero rows".to_string());
    }
    Ok(terms)
}

fn collect_text_files(
    root: &Path,
    exclude_roots: &[PathBuf],
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !root.exists() {
        // A missing corpus root is not an error — some callers pass roots
        // that don't exist in every layout (e.g. fresh clones without
        // `tools/`). Silently skip.
        return Ok(());
    }
    let entries = fs::read_dir(root)
        .map_err(|error| format!("could not list corpus dir {}: {error}", root.display()))?;
    for entry in entries {
        let entry = entry
            .map_err(|error| format!("could not read entry in {}: {error}", root.display()))?;
        let path = entry.path();
        if exclude_roots
            .iter()
            .any(|excluded| path.starts_with(excluded))
        {
            continue;
        }
        if path.is_dir() {
            collect_text_files(&path, exclude_roots, out)?;
        } else if is_text_extension(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_text_extension(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };
    matches!(
        extension,
        "md" | "mdx" | "rs" | "toml" | "yaml" | "yml" | "json" | "tsv" | "csv" | "txt" | "sh"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_registry_extracts_all_rows() {
        let yaml = r#"
retired:
  - term: "repoctl pre-push"
    retired_at: "2026-05-15"
    canonical_replacement: "oya verify"
    adr: ""
  - term: "oya dev check"
    retired_at: "2026-05-15"
    canonical_replacement: "oya verify"
    adr: "ADR-0099"
"#;
        let terms = parse_retired_registry(yaml).expect("parses");
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].term, "repoctl pre-push");
        assert_eq!(terms[0].canonical_replacement, "oya verify");
        assert_eq!(terms[0].adr, None);
        assert_eq!(terms[1].term, "oya dev check");
        assert_eq!(terms[1].adr, Some("ADR-0099".to_string()));
    }

    #[test]
    fn parse_registry_rejects_zero_rows() {
        let yaml = "retired:\n";
        let error = parse_retired_registry(yaml).unwrap_err();
        assert!(error.contains("zero rows"));
    }

    #[test]
    fn parse_uses_canonical_defaults() {
        let args = parse_retired_vocabulary_validate_args(Vec::new()).expect("no flags is valid");
        assert_eq!(
            args.registry_path,
            PathBuf::from("registry/vocabulary/retired.yaml")
        );
        assert!(args.corpus_roots.contains(&PathBuf::from("docs")));
        assert!(
            args.exclude_roots
                .contains(&PathBuf::from("evidence/audits"))
        );
    }

    #[test]
    fn parse_corpus_root_replaces_defaults_on_first_use() {
        let args = parse_retired_vocabulary_validate_args(vec![
            "--corpus-root".to_string(),
            "fixtures/corpus".to_string(),
        ])
        .expect("custom root parses");
        assert_eq!(args.corpus_roots, vec![PathBuf::from("fixtures/corpus")]);
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let result = parse_retired_vocabulary_validate_args(vec!["--unknown".to_string()]);
        assert!(result.is_err());
    }
}
