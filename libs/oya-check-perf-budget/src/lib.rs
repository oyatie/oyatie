//! Perf-budget check (ADR-0062 §"performance budget"; decision-principles.json
//! DP-09 bench-and-stress).
//!
//! Every implementation plan markdown (`.omc/plans/milestones/**/IP-*.md`)
//! MUST include a `## Load test` section that contains at least one concrete
//! performance measurement. Empty placeholder sections and digit-only filler
//! such as "0 things to do" are not enough.
//!
//! Scope of this kernel: pure logic over typed [`ImplementationPlan`] nodes
//! pre-harvested by a runner. No I/O.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImplementationPlan {
    pub path: String,    // data_class: INTERNAL_ONLY
    pub content: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ViolationKind {
    SectionMissing,
    SectionEmpty,
    SectionMissingNumbers,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SectionMissing => write!(f, "no '## Load test' section"),
            Self::SectionEmpty => write!(f, "'## Load test' section is empty"),
            Self::SectionMissingNumbers => {
                write!(
                    f,
                    "'## Load test' section contains no concrete performance measurements"
                )
            }
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
    pub plans_checked: usize,
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
            Self::EmptyPath => write!(f, "implementation plan with empty path"),
            Self::DuplicatePath { path } => write!(f, "duplicate plan path: {path}"),
        }
    }
}

impl std::error::Error for Error {}

const SECTION_HEADING: &str = "## Load test";
const PERFORMANCE_MEASUREMENT_TOKENS: &[&str] = &[
    "p50",
    "p95",
    "p99",
    "p999",
    "latency",
    "throughput",
    "rps",
    "qps",
    "req/sec",
    "requests/sec",
    "ms",
    "seconds",
    "sec",
    "users",
    "concurrent",
    "error budget",
    "burn-rate",
    "k6",
    "locust",
    "vegeta",
];

fn extract_section_body<'a>(content: &'a str, heading: &str) -> Option<&'a str> {
    let iter = content.split('\n');
    let mut start: Option<usize> = None;
    let mut consumed = 0usize;
    for line in iter {
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
    // Section ends at the next `## ` heading or EOF.
    let end = tail.find("\n## ").unwrap_or(tail.len());
    Some(&tail[..end])
}

fn body_has_performance_measurement(body: &str) -> bool {
    let tokens = tokenize_measurement_body(body);
    tokens
        .iter()
        .enumerate()
        .any(|(idx, token)| token_has_digit(token) && has_measurement_context(&tokens, idx))
}

fn tokenize_measurement_body(body: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in body.chars() {
        if ch.is_ascii_alphanumeric() || ch == '/' || ch == '-' {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn token_has_digit(token: &str) -> bool {
    token.chars().any(|c| c.is_ascii_digit())
}

fn has_measurement_context(tokens: &[String], number_idx: usize) -> bool {
    if token_is_measurement(&tokens[number_idx]) {
        return true;
    }

    let start = number_idx.saturating_sub(2);
    let end = (number_idx + 3).min(tokens.len());
    tokens[start..end]
        .iter()
        .enumerate()
        .any(|(offset, token)| start + offset != number_idx && token_is_measurement(token))
}

fn token_is_measurement(token: &str) -> bool {
    PERFORMANCE_MEASUREMENT_TOKENS.contains(&token) || token_has_numeric_measurement_suffix(token)
}

fn token_has_numeric_measurement_suffix(token: &str) -> bool {
    let digit_end = token
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit())
        .last()
        .map(|(idx, ch)| idx + ch.len_utf8());
    let Some(digit_end) = digit_end else {
        return false;
    };
    if digit_end == 0 || digit_end >= token.len() {
        return false;
    }

    matches!(
        &token[digit_end..],
        "ms" | "sec" | "seconds" | "rps" | "qps" | "users"
    )
}

pub fn check(plans: &[ImplementationPlan]) -> Result<Report, Error> {
    let mut seen = BTreeSet::new();
    let mut violations = Vec::new();

    for plan in plans {
        if plan.path.trim().is_empty() {
            return Err(Error::EmptyPath);
        }
        if !seen.insert(plan.path.clone()) {
            return Err(Error::DuplicatePath {
                path: plan.path.clone(),
            });
        }

        match extract_section_body(&plan.content, SECTION_HEADING) {
            None => violations.push(Violation {
                path: plan.path.clone(),
                kind: ViolationKind::SectionMissing,
            }),
            Some(body) => {
                let body_trimmed = body.trim();
                if body_trimmed.is_empty() {
                    violations.push(Violation {
                        path: plan.path.clone(),
                        kind: ViolationKind::SectionEmpty,
                    });
                } else if !body_has_performance_measurement(body_trimmed) {
                    violations.push(Violation {
                        path: plan.path.clone(),
                        kind: ViolationKind::SectionMissingNumbers,
                    });
                }
            }
        }
    }

    Ok(Report {
        plans_checked: plans.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(path: &str, content: &str) -> ImplementationPlan {
        ImplementationPlan {
            path: path.into(),
            content: content.into(),
        }
    }

    #[test]
    fn empty_input_passes() {
        let r = check(&[]).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn section_with_numbers_passes() {
        let r = check(&[ip(
            "IP-001.md",
            "# IP\n## Load test\n\np99 < 250ms at 1000 RPS.\n",
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn missing_section_flagged() {
        let r = check(&[ip("IP-002.md", "# IP\nNo load test section here.\n")]).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionMissing);
    }

    #[test]
    fn empty_section_flagged() {
        let r = check(&[ip("IP-003.md", "# IP\n## Load test\n\n## Next section\n")]).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionEmpty);
    }

    #[test]
    fn section_without_numbers_flagged() {
        let r = check(&[ip(
            "IP-004.md",
            "# IP\n## Load test\n\nTBD — pending decision on tooling.\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionMissingNumbers);
    }

    #[test]
    fn digit_only_placeholder_is_not_a_performance_measurement() {
        let r = check(&[ip(
            "IP-004b.md",
            "# IP\n## Load test\n\n0 things to do before merge.\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionMissingNumbers);
    }

    #[test]
    fn ordinary_words_containing_measurement_substrings_are_not_measurements() {
        for body in [
            "0 items to do before merge.",
            "1 section remains before merge.",
            "2 teams signed off.",
        ] {
            let r = check(&[ip("IP-004c.md", &format!("# IP\n## Load test\n\n{body}\n"))]).unwrap();
            assert_eq!(r.violations.len(), 1, "{body}");
            assert_eq!(
                r.violations[0].kind,
                ViolationKind::SectionMissingNumbers,
                "{body}"
            );
        }
    }

    #[test]
    fn adjacent_metric_patterns_are_measurements() {
        for body in [
            "p99 < 250ms at 1000 rps.",
            "Latency target 250ms.",
            "Load generated with k6 for 60 seconds.",
            "Supports 500 concurrent users.",
        ] {
            let r = check(&[ip("IP-004d.md", &format!("# IP\n## Load test\n\n{body}\n"))]).unwrap();
            assert!(r.violations.is_empty(), "{body}");
        }
    }

    #[test]
    fn section_ends_at_next_h2() {
        let r = check(&[ip(
            "IP-005.md",
            "## Load test\n\nNo numbers here.\n\n## Other\n\np99 < 500ms.\n",
        )])
        .unwrap();
        // The digit-bearing line is in the next section; this section gets flagged.
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::SectionMissingNumbers);
    }

    #[test]
    fn rejects_empty_path() {
        let err = check(&[ip("", "")]).unwrap_err();
        assert_eq!(err, Error::EmptyPath);
    }

    #[test]
    fn rejects_duplicate_path() {
        let err = check(&[ip("a.md", ""), ip("a.md", "")]).unwrap_err();
        assert!(matches!(err, Error::DuplicatePath { .. }));
    }
}
