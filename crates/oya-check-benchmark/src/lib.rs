//! Competitive-benchmark check (ADR-0062 §"competitive benchmarks";
//! decision-principles.json DP-09 bench-and-stress).
//!
//! Every PRD markdown (`docs/products/**/PRD*.md`, `docs/prds/*.md`) MUST
//! include a `## Competitive benchmark` section with at least one named
//! competitor reference. The cheap heuristic is "section body contains either
//! a digit (number) OR a recognized competitor token from the registry."
//! The registry of accepted competitor tokens is passed in by the runner so
//! the kernel stays I/O-free and the recognized-set is owned by docs/standards.
//!
//! Scope: pure logic over [`Prd`] nodes + a competitor-token registry.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prd {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub content: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ViolationKind {
    SectionMissing,
    SectionEmpty,
    SectionUnsubstantiated,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SectionMissing => write!(f, "no '## Competitive benchmark' section"),
            Self::SectionEmpty => write!(f, "'## Competitive benchmark' section is empty"),
            Self::SectionUnsubstantiated => write!(
                f,
                "'## Competitive benchmark' section has no number and no recognized competitor"
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub path: String,
    pub kind: ViolationKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub prds_checked: usize,
    pub violations: Vec<Violation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    EmptyPath,
    DuplicatePath { path: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "PRD with empty path"),
            Self::DuplicatePath { path } => write!(f, "duplicate PRD path: {path}"),
        }
    }
}

impl std::error::Error for Error {}

const SECTION_HEADING: &str = "## Competitive benchmark";

fn extract_section_body<'a>(content: &'a str, heading: &str) -> Option<&'a str> {
    let mut consumed = 0usize;
    let mut start: Option<usize> = None;
    for line in content.split('\n') {
        if line.trim_end() == heading {
            start = Some(consumed + line.len() + 1);
            break;
        }
        consumed += line.len() + 1;
    }
    let start = start?;
    if start >= content.len() {
        return Some("");
    }
    let tail = &content[start..];
    let end = tail.find("\n## ").unwrap_or(tail.len());
    Some(&tail[..end])
}

fn body_has_digit(body: &str) -> bool {
    body.chars().any(|c| c.is_ascii_digit())
}

fn body_has_known_competitor(body: &str, competitors: &[&str]) -> bool {
    let lower = body.to_ascii_lowercase();
    competitors
        .iter()
        .any(|name| !name.is_empty() && lower.contains(&name.to_ascii_lowercase()))
}

/// Validate each PRD against the competitive-benchmark contract.
///
/// `known_competitors` is the registry of accepted competitor tokens (e.g.
/// `["stripe", "linear", "palantir", "n8n"]`). At least one of these must
/// appear in the section body, OR the body must contain a concrete digit
/// (a measured number is its own evidence).
pub fn check(prds: &[Prd], known_competitors: &[&str]) -> Result<Report, Error> {
    let mut seen = BTreeSet::new();
    let mut violations = Vec::new();

    for prd in prds {
        if prd.path.trim().is_empty() {
            return Err(Error::EmptyPath);
        }
        if !seen.insert(prd.path.clone()) {
            return Err(Error::DuplicatePath {
                path: prd.path.clone(),
            });
        }

        match extract_section_body(&prd.content, SECTION_HEADING) {
            None => violations.push(Violation {
                path: prd.path.clone(),
                kind: ViolationKind::SectionMissing,
            }),
            Some(body) => {
                let body_trimmed = body.trim();
                if body_trimmed.is_empty() {
                    violations.push(Violation {
                        path: prd.path.clone(),
                        kind: ViolationKind::SectionEmpty,
                    });
                } else if !body_has_digit(body_trimmed)
                    && !body_has_known_competitor(body_trimmed, known_competitors)
                {
                    violations.push(Violation {
                        path: prd.path.clone(),
                        kind: ViolationKind::SectionUnsubstantiated,
                    });
                }
            }
        }
    }

    Ok(Report {
        prds_checked: prds.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prd(path: &str, content: &str) -> Prd {
        Prd {
            path: path.into(),
            content: content.into(),
        }
    }

    const COMPETITORS: &[&str] = &["stripe", "linear", "palantir", "n8n", "snowflake"];

    #[test]
    fn empty_input_passes() {
        let r = check(&[], COMPETITORS).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn section_with_competitor_passes() {
        let r = check(
            &[prd(
                "docs/products/payments/PRD.md",
                "# Payments\n## Competitive benchmark\n\nWe match Stripe on auth-rate.\n",
            )],
            COMPETITORS,
        )
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn section_with_only_numbers_passes() {
        let r = check(
            &[prd(
                "docs/prds/foundry.md",
                "## Competitive benchmark\n\nTarget: p99 < 300ms.\n",
            )],
            COMPETITORS,
        )
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn missing_section_flagged() {
        let r = check(
            &[prd("docs/prds/x.md", "# X\nNo benchmark section.\n")],
            COMPETITORS,
        )
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionMissing);
    }

    #[test]
    fn empty_section_flagged() {
        let r = check(
            &[prd(
                "docs/prds/y.md",
                "## Competitive benchmark\n\n## Next\n",
            )],
            COMPETITORS,
        )
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionEmpty);
    }

    #[test]
    fn section_without_evidence_flagged() {
        let r = check(
            &[prd(
                "docs/prds/z.md",
                "## Competitive benchmark\n\nTo be researched later.\n",
            )],
            COMPETITORS,
        )
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionUnsubstantiated);
    }

    #[test]
    fn case_insensitive_competitor_match() {
        let r = check(
            &[prd(
                "docs/prds/a.md",
                "## Competitive benchmark\n\nWe outperform PALANTIR foundry on tenant isolation.\n",
            )],
            COMPETITORS,
        )
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn rejects_empty_path() {
        let err = check(&[prd("", "")], COMPETITORS).unwrap_err();
        assert_eq!(err, Error::EmptyPath);
    }

    #[test]
    fn rejects_duplicate_path() {
        let err = check(&[prd("a.md", ""), prd("a.md", "")], COMPETITORS).unwrap_err();
        assert!(matches!(err, Error::DuplicatePath { .. }));
    }
}
