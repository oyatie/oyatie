//! Foundry retired-vocabulary fitness kernel.
//!
//! Prevents documentation + registry drift back to retired CLI
//! subcommands, retired binaries, retired crates, and other retired
//! vocabulary. Each retirement decision adds a row to
//! `registry/vocabulary/retired.yaml`; this kernel asserts no live
//! document still mentions any retired term, with the row's recorded
//! canonical replacement as the suggested fix.
//!
//! Lane id: `oya-governance-retired-vocabulary`. The lane is
//! the machine-checkable encoding of the user's
//! [[feedback_no_exceptions_canonical]] +
//! [[feedback_no_silent_regression]] directives: once a term is
//! retired, CI refuses any future re-introduction (whether by an
//! agent, a human, or a doc that wasn't swept during the retirement).
//!
//! Naming justification: crate name `check-retired-vocabulary`
//! follows the `check-<lane>` family naming. The kernel is a
//! pure-domain port-in-kernel (ADR-0056); the runner in
//! `oya-dev-cli/src/retired_vocabulary_gate.rs` performs the I/O.
//! Type names use the noun form `RetiredTerm` /
//! `RetiredVocabularyMatch` / `RetiredVocabularyReport` /
//! `RetiredVocabularyError` so callers can grep one prefix.
//!
//! Layer enum: this kernel sits on the `domain` layer (port-in-kernel,
//! ADR-0056); it performs pure I/O-free static scanning of strings
//! handed in by the runner.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// One row in the retired-vocabulary registry. Each retired term has a
/// retirement date, a canonical replacement that should be used
/// instead, and an optional ADR pointer recording the retirement
/// decision.
///
/// `term` is the literal substring the kernel searches for; case is
/// significant. Examples: `"repoctl pre-push"`, `"oya dev check"`,
/// `"scripts/check.sh"`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetiredTerm {
    pub term: String,                  // data_class: INTERNAL_ONLY
    pub retired_at: String,            // data_class: INTERNAL_ONLY (YYYY-MM-DD)
    pub canonical_replacement: String, // data_class: INTERNAL_ONLY
    pub adr: Option<String>,           // data_class: INTERNAL_ONLY (ADR-NNNN or None)
}

/// One scanned document (path + contents). The kernel does no I/O;
/// the runner reads files and constructs these.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScannedDocument<'a> {
    pub path: &'a str,     // data_class: INTERNAL_ONLY
    pub contents: &'a str, // data_class: INTERNAL_ONLY
}

/// One drift hit: a line in some document mentions a retired term.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetiredVocabularyMatch {
    pub document_path: String,         // data_class: INTERNAL_ONLY
    pub line_number: usize,            // data_class: INTERNAL_ONLY (1-indexed)
    pub line_contents: String,         // data_class: INTERNAL_ONLY (trimmed)
    pub term: String,                  // data_class: INTERNAL_ONLY
    pub canonical_replacement: String, // data_class: INTERNAL_ONLY
}

/// Per-run report. `violations.is_empty()` is the green condition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetiredVocabularyReport {
    pub documents_checked: usize,                // data_class: INTERNAL_ONLY
    pub terms_checked: usize,                    // data_class: INTERNAL_ONLY
    pub violations: Vec<RetiredVocabularyMatch>, // data_class: INTERNAL_ONLY
}

/// Validation errors. `ViolationsFound` carries the report so the
/// runner can surface every drift hit at once (not one-at-a-time).
///
/// Naming justification: variants describe the failure positively (no
/// "exception" / "exempt" phrasing per
/// `feedback_no_exceptions_canonical`); the data carried by each
/// variant lets the runner render exact remediation instructions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetiredVocabularyError {
    DuplicateTerm(String),
    EmptyTerm,
    EmptyCanonicalReplacement(String),
    ViolationsFound(RetiredVocabularyReport),
}

impl fmt::Display for RetiredVocabularyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateTerm(term) => write!(
                formatter,
                "duplicate retired-vocabulary term `{term}` — every retired \
                 term must appear exactly once in the registry"
            ),
            Self::EmptyTerm => write!(
                formatter,
                "empty retired-vocabulary term — every registry row must \
                 declare a non-empty `term`"
            ),
            Self::EmptyCanonicalReplacement(term) => write!(
                formatter,
                "retired-vocabulary term `{term}` has empty \
                 `canonical_replacement` — every row must point at the \
                 canonical replacement surface"
            ),
            Self::ViolationsFound(report) => {
                writeln!(
                    formatter,
                    "{} retired-vocabulary drift hit(s) across {} documents \
                     ({} retired terms checked):",
                    report.violations.len(),
                    report.documents_checked,
                    report.terms_checked
                )?;
                for hit in &report.violations {
                    writeln!(
                        formatter,
                        "  {}:{}: `{}` → use canonical `{}`",
                        hit.document_path, hit.line_number, hit.term, hit.canonical_replacement
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RetiredVocabularyError {}

/// Validate the documents against the retired-term registry.
///
/// Returns `Ok(report)` when no document mentions any retired term;
/// returns `Err(ViolationsFound(report))` when one or more do.
/// `DuplicateTerm` / `EmptyTerm` / `EmptyCanonicalReplacement` errors
/// fire when the registry itself is malformed; those are runner-side
/// input failures, not document drift.
///
/// The kernel performs no I/O. Document strings are scanned line by
/// line; a line matches a term if the line contains the term as a
/// substring (case-sensitive). 1-indexed line numbers match the way
/// editors and `grep` render hits.
pub fn validate_retired_vocabulary<'a, D>(
    terms: &[RetiredTerm],
    documents: D,
) -> Result<RetiredVocabularyReport, RetiredVocabularyError>
where
    D: IntoIterator<Item = ScannedDocument<'a>>,
{
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for row in terms {
        if row.term.is_empty() {
            return Err(RetiredVocabularyError::EmptyTerm);
        }
        if row.canonical_replacement.is_empty() {
            return Err(RetiredVocabularyError::EmptyCanonicalReplacement(
                row.term.clone(),
            ));
        }
        if !seen.insert(row.term.as_str()) {
            return Err(RetiredVocabularyError::DuplicateTerm(row.term.clone()));
        }
    }

    let mut violations: Vec<RetiredVocabularyMatch> = Vec::new();
    let mut documents_checked = 0usize;
    for document in documents {
        documents_checked += 1;
        for (line_index, line) in document.contents.lines().enumerate() {
            for row in terms {
                if line.contains(&row.term) {
                    violations.push(RetiredVocabularyMatch {
                        document_path: document.path.to_string(),
                        line_number: line_index + 1,
                        line_contents: line.trim().to_string(),
                        term: row.term.clone(),
                        canonical_replacement: row.canonical_replacement.clone(),
                    });
                }
            }
        }
    }

    let report = RetiredVocabularyReport {
        documents_checked,
        terms_checked: terms.len(),
        violations,
    };

    if report.violations.is_empty() {
        Ok(report)
    } else {
        Err(RetiredVocabularyError::ViolationsFound(report))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn retired(term: &str, replacement: &str) -> RetiredTerm {
        RetiredTerm {
            term: term.to_string(),
            retired_at: "2026-05-15".to_string(),
            canonical_replacement: replacement.to_string(),
            adr: None,
        }
    }

    #[test]
    fn accepts_documents_with_no_retired_term() {
        let terms = vec![retired("repoctl pre-push", "oya verify")];
        let doc = ScannedDocument {
            path: "docs/clean.md",
            contents: "Run `oya verify` before pushing.\n",
        };
        let report =
            validate_retired_vocabulary(&terms, [doc]).expect("clean document is accepted");
        assert_eq!(report.documents_checked, 1);
        assert_eq!(report.terms_checked, 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn flags_document_that_mentions_retired_term() {
        let terms = vec![retired("repoctl pre-push", "oya verify")];
        let doc = ScannedDocument {
            path: "docs/stale.md",
            contents: "Pre-push: run `repoctl pre-push` first.\n",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        let RetiredVocabularyError::ViolationsFound(report) = error else {
            panic!("expected ViolationsFound, got {error:?}");
        };
        assert_eq!(report.violations.len(), 1);
        let hit = &report.violations[0];
        assert_eq!(hit.document_path, "docs/stale.md");
        assert_eq!(hit.line_number, 1);
        assert_eq!(hit.term, "repoctl pre-push");
        assert_eq!(hit.canonical_replacement, "oya verify");
    }

    #[test]
    fn flags_multiple_terms_on_distinct_lines() {
        let terms = vec![
            retired("repoctl pre-push", "oya verify"),
            retired("oya dev check", "oya verify"),
            retired("scripts/check.sh", "oya gate run-all"),
        ];
        let doc = ScannedDocument {
            path: "docs/triple-stale.md",
            contents: "Line A repoctl pre-push.\n\
                       Line B oya dev check.\n\
                       Line C scripts/check.sh.\n",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        let RetiredVocabularyError::ViolationsFound(report) = error else {
            panic!("expected ViolationsFound, got {error:?}");
        };
        assert_eq!(report.violations.len(), 3);
        assert_eq!(report.documents_checked, 1);
        assert_eq!(report.terms_checked, 3);
    }

    #[test]
    fn rejects_duplicate_term_in_registry() {
        let terms = vec![
            retired("repoctl pre-push", "oya verify"),
            retired("repoctl pre-push", "oya verify"),
        ];
        let doc = ScannedDocument {
            path: "docs/anywhere.md",
            contents: "",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        assert!(matches!(error, RetiredVocabularyError::DuplicateTerm(_)));
    }

    #[test]
    fn rejects_empty_term_in_registry() {
        let terms = vec![RetiredTerm {
            term: String::new(),
            retired_at: "2026-05-15".to_string(),
            canonical_replacement: "oya verify".to_string(),
            adr: None,
        }];
        let doc = ScannedDocument {
            path: "docs/anywhere.md",
            contents: "",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        assert_eq!(error, RetiredVocabularyError::EmptyTerm);
    }

    #[test]
    fn rejects_empty_canonical_replacement() {
        let terms = vec![RetiredTerm {
            term: "repoctl pre-push".to_string(),
            retired_at: "2026-05-15".to_string(),
            canonical_replacement: String::new(),
            adr: None,
        }];
        let doc = ScannedDocument {
            path: "docs/anywhere.md",
            contents: "",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        assert!(matches!(
            error,
            RetiredVocabularyError::EmptyCanonicalReplacement(_)
        ));
    }

    #[test]
    fn reports_one_hit_per_term_per_line() {
        // A line that names the same retired term twice still produces
        // one hit per `(line, term)` pair (substring search is matched
        // at most once per line/term combo), so the runner can render
        // a one-hit-per-line error.
        let terms = vec![retired("repoctl pre-push", "oya verify")];
        let doc = ScannedDocument {
            path: "docs/twice.md",
            contents: "Old: repoctl pre-push, also see repoctl pre-push.\n",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        let RetiredVocabularyError::ViolationsFound(report) = error else {
            panic!("expected ViolationsFound");
        };
        assert_eq!(report.violations.len(), 1);
    }

    #[test]
    fn line_numbers_are_one_indexed_per_editor_convention() {
        let terms = vec![retired("repoctl pre-push", "oya verify")];
        let doc = ScannedDocument {
            path: "docs/multi-line.md",
            contents: "Line 1 clean.\n\
                       Line 2 clean.\n\
                       Line 3 has repoctl pre-push.\n",
        };
        let error = validate_retired_vocabulary(&terms, [doc]).unwrap_err();
        let RetiredVocabularyError::ViolationsFound(report) = error else {
            panic!("expected ViolationsFound");
        };
        assert_eq!(report.violations[0].line_number, 3);
    }
}
