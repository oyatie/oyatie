//! ADR placeholder validator + auto-fixer (Wave-3 Rust port).
//!
//! Naming justification:
//! - Crate id `oya-check-adr-placeholders` — `oya-` brand prefix
//!   (ADR-0017 / MFL-0011), `check` lane class (Layer 1 kernel-tier validator
//!   per ADR-0083), `adr-placeholders` two-word subject.
//! - Library identifier `oya_check_adr_placeholders` — snake_case mirror
//!   (ADR-0105 v4 BNF §2.2).
//!
//! Replaces `scripts/rewrite-adr-placeholders.py` (Wave-3 Python → Rust
//! conversion per FIX-AGENT-E round-2 directive). Detects and optionally
//! rewrites `ADR-XXXX` / `ADR-NNNN` placeholder citations across the
//! docs/specs/microservices/registry corpus.
//!
//! Modes:
//! - `Mode::Validate` — return `Err(Report)` if any placeholder is found.
//! - `Mode::AutoFix`  — rewrite files in-place with canonical replacements.
//!
//! Tier 1 (kernel-tier) per ADR-0083: pure logic over already-loaded
//! [`FileContent`] records; the surrounding binary supplies filesystem IO.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// Order-significant phrase-level substitutions. More specific phrases come
/// first so they win over the generic `ADR-NNNN` → `ADR-####` sweep.
///
/// Mirrors the `CONCRETE` table in the legacy
/// `scripts/rewrite-adr-placeholders.py`. Each entry is `(needle, replacement)`.
pub const CONCRETE_REPLACEMENTS: &[(&str, &str)] = &[
    (
        "ADR-NNNN-personal-mail-key-recovery",
        "registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery",
    ),
    (
        "ADR-NNNN-passphrase-derivation-upgrade",
        "registry/placeholder-debt/adr-follow-ups.yaml#passphrase-derivation-upgrade",
    ),
    (
        "ADR-NNNN-mail-workflow-extraction-default",
        "registry/placeholder-debt/adr-follow-ups.yaml#mail-workflow-extraction-default",
    ),
    (
        "ADR-NNNN-connect-umbrella-retired",
        "registry/placeholder-debt/adr-follow-ups.yaml#connect-umbrella-retirement-marker",
    ),
    (
        "ADR-NNNN-grit-scaffold-claim-pattern.md",
        "registry/placeholder-debt/adr-follow-ups.yaml#grit-scaffold-claim-pattern (superseded by ADR-0116)",
    ),
    (
        "ADR-NNNN-grit-cutover-inventory.md",
        "registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)",
    ),
    (
        "ADR-NNNN-grit-cutover-inventory",
        "registry/placeholder-debt/adr-follow-ups.yaml#grit-cutover-inventory (superseded by ADR-0116)",
    ),
    (
        "ADR-XXXX-four-layer-branch-pipeline.md",
        "registry/placeholder-debt/adr-follow-ups.yaml#four-layer-branch-pipeline (drafting)",
    ),
    (
        "ADR-XXXX-four-layer-branch-pipeline",
        "registry/placeholder-debt/adr-follow-ups.yaml#four-layer-branch-pipeline (drafting)",
    ),
    ("ADR-NNNN-retire-<lane>.md", "ADR-####-retire-<lane>.md"),
    (
        "ADR-NNNN-pack-<pack>-onboarding",
        "ADR-####-pack-<pack>-onboarding",
    ),
    (
        "ADR-NNNN-<pack>-<microservice>-regulatory.md",
        "ADR-####-<pack>-<microservice>-regulatory.md",
    ),
    (
        "ADR-NNNN-kr-<microservice>-regulatory.md",
        "ADR-####-kr-<microservice>-regulatory.md",
    ),
    (
        "ADR-NNNN-microservice-<microservice>.md",
        "ADR-####-microservice-<microservice>.md",
    ),
    ("ADR-NNNN-<kebab-summary>.md", "ADR-####-<kebab-summary>.md"),
    ("ADR-NNNN-<slug>.md", "ADR-####-<slug>.md"),
    ("ADR-NNNN-<slug>", "ADR-####-<slug>"),
    ("ADR-NNNN-*.md", "ADR-####-*.md"),
    ("ADR-FORMS-NNNN", "ADR-FORMS-####"),
    ("ADR-WS-NNNN", "ADR-WS-####"),
    ("ADR-SHEETS-NNNN", "ADR-SHEETS-####"),
];

/// Generic sigils that remain after the phrase-level pass. Matched only on
/// word boundaries to avoid mangling unrelated identifiers.
pub const GENERIC_SIGILS: &[(&str, &str)] = &[("ADR-NNNN", "ADR-####"), ("ADR-XXXX", "ADR-####")];

/// Operating mode of the validator/fixer kernel.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Mode {
    /// Surface placeholders as a [`Report`] error without mutating anything.
    #[default]
    Validate,
    /// Apply canonical replacements in-memory; callers may persist the
    /// resulting [`Rewrite`] back to disk.
    AutoFix,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validate => write!(f, "validate"),
            Self::AutoFix => write!(f, "auto-fix"),
        }
    }
}

/// A file submitted to the kernel for inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileContent {
    pub path: String,
    pub content: String,
}

/// A specific placeholder occurrence the validator surfaced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hit {
    pub path: String,
    pub line: u32,
    pub token: String,
}

/// Pass-mode report (validate mode).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    pub files_checked: usize,
    pub hits: Vec<Hit>,
}

/// Auto-fix output: the rewritten file body + a replacement count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rewrite {
    pub path: String,
    pub content: String,
    pub replacements: usize,
}

/// Top-level kernel error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// At least one placeholder remains. Carries the full report so callers
    /// can render a human-readable failure summary.
    PlaceholdersFound(Report),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlaceholdersFound(report) => {
                write!(
                    f,
                    "ADR placeholder validation failed: {} hits across {} files",
                    report.hits.len(),
                    report.files_checked
                )
            }
        }
    }
}

/// Validate the supplied files for ADR placeholders.
///
/// Returns `Ok(Report)` with `hits.is_empty()` when the corpus is clean.
/// Returns `Err(Error::PlaceholdersFound(...))` otherwise.
pub fn validate(files: &[FileContent]) -> Result<Report, Error> {
    let mut report = Report {
        files_checked: files.len(),
        hits: Vec::new(),
    };
    for file in files {
        scan_file_for_hits(file, &mut report.hits);
    }
    if report.hits.is_empty() {
        Ok(report)
    } else {
        Err(Error::PlaceholdersFound(report))
    }
}

/// Apply the canonical replacements to each file, producing a [`Rewrite`]
/// per file whose content actually changed.
pub fn auto_fix(files: &[FileContent]) -> Vec<Rewrite> {
    let mut rewrites = Vec::new();
    for file in files {
        let (text, n) = apply_replacements(&file.content);
        if n > 0 {
            rewrites.push(Rewrite {
                path: file.path.clone(),
                content: text,
                replacements: n,
            });
        }
    }
    rewrites
}

fn scan_file_for_hits(file: &FileContent, out: &mut Vec<Hit>) {
    for (lineno, line) in file.content.lines().enumerate() {
        for token in ["ADR-NNNN", "ADR-XXXX"] {
            if has_word_boundary_match(line, token) {
                out.push(Hit {
                    path: file.path.clone(),
                    line: (lineno + 1) as u32,
                    token: token.to_string(),
                });
            }
        }
    }
}

fn apply_replacements(input: &str) -> (String, usize) {
    let mut text = input.to_string();
    let mut total = 0usize;
    for (needle, repl) in CONCRETE_REPLACEMENTS {
        if text.contains(needle) {
            total += text.matches(needle).count();
            text = text.replace(needle, repl);
        }
    }
    for (sigil, repl) in GENERIC_SIGILS {
        let (next, n) = replace_word_boundary(&text, sigil, repl);
        total += n;
        text = next;
    }
    (text, total)
}

fn has_word_boundary_match(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let nlen = needle.len();
    let mut idx = 0;
    while let Some(found) = haystack[idx..].find(needle) {
        let pos = idx + found;
        let before_ok = pos == 0 || !is_word_char(bytes[pos - 1]);
        let after_ok = pos + nlen >= bytes.len() || !is_word_char(bytes[pos + nlen]);
        if before_ok && after_ok {
            return true;
        }
        idx = pos + nlen;
        if idx >= haystack.len() {
            break;
        }
    }
    false
}

fn replace_word_boundary(input: &str, needle: &str, repl: &str) -> (String, usize) {
    let mut out = String::with_capacity(input.len());
    let mut idx = 0;
    let bytes = input.as_bytes();
    let nlen = needle.len();
    let mut count = 0usize;
    while let Some(found) = input[idx..].find(needle) {
        let pos = idx + found;
        let before_ok = pos == 0 || !is_word_char(bytes[pos - 1]);
        let after_ok = pos + nlen >= bytes.len() || !is_word_char(bytes[pos + nlen]);
        out.push_str(&input[idx..pos]);
        if before_ok && after_ok {
            out.push_str(repl);
            count += 1;
        } else {
            out.push_str(&input[pos..pos + nlen]);
        }
        idx = pos + nlen;
    }
    out.push_str(&input[idx..]);
    (out, count)
}

fn is_word_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fc(path: &str, content: &str) -> FileContent {
        FileContent {
            path: path.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn validate_clean_corpus_returns_ok() {
        let files = vec![fc("docs/a.md", "Cites ADR-0123 — concrete reference.")];
        let report = validate(&files).expect("clean corpus passes");
        assert_eq!(report.files_checked, 1);
        assert!(report.hits.is_empty());
    }

    #[test]
    fn validate_detects_adr_nnnn_placeholder() {
        let files = vec![fc("docs/a.md", "todo: file ADR-NNNN-soon")];
        let err = validate(&files).expect_err("placeholder must fail validate mode");
        match err {
            Error::PlaceholdersFound(report) => {
                assert_eq!(report.hits.len(), 1);
                assert_eq!(report.hits[0].token, "ADR-NNNN");
                assert_eq!(report.hits[0].line, 1);
            }
        }
    }

    #[test]
    fn validate_detects_adr_xxxx_placeholder() {
        let files = vec![fc("docs/b.md", "later: ADR-XXXX placeholder here")];
        let err = validate(&files).expect_err("XXXX placeholder must fail");
        let Error::PlaceholdersFound(report) = err;
        assert_eq!(report.hits.len(), 1);
        assert_eq!(report.hits[0].token, "ADR-XXXX");
    }

    #[test]
    fn auto_fix_rewrites_concrete_phrase_first() {
        let files = vec![fc(
            "docs/c.md",
            "follow-up: ADR-NNNN-personal-mail-key-recovery is required",
        )];
        let rewrites = auto_fix(&files);
        assert_eq!(rewrites.len(), 1);
        assert!(
            rewrites[0].content.contains(
                "registry/placeholder-debt/adr-follow-ups.yaml#personal-mail-key-recovery"
            )
        );
        assert!(!rewrites[0].content.contains("ADR-NNNN"));
    }

    #[test]
    fn auto_fix_rewrites_generic_sigil_with_word_boundary() {
        let files = vec![fc("docs/d.md", "shape: ADR-NNNN goes here")];
        let rewrites = auto_fix(&files);
        assert_eq!(rewrites.len(), 1);
        assert!(rewrites[0].content.contains("ADR-####"));
        assert_eq!(rewrites[0].replacements, 1);
    }

    #[test]
    fn auto_fix_leaves_unrelated_identifiers_untouched() {
        let files = vec![fc(
            "docs/e.md",
            "neighbor token ADR-NNNNX should not match the bare sigil",
        )];
        let rewrites = auto_fix(&files);
        // 'ADR-NNNNX' (alpha follows) is not a bare-word match for ADR-NNNN.
        assert!(rewrites.is_empty());
    }

    #[test]
    fn validate_skips_word_boundary_false_match() {
        let files = vec![fc("docs/f.md", "lookalike ADR-NNNN5 not a sigil")];
        let report = validate(&files).expect("non-boundary token tolerated");
        assert!(report.hits.is_empty());
    }
}
