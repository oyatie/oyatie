//! Statelessness check (ADR-0062 §"sharded state"; decision-principles.json DP-09).
//!
//! Outer-layer crates (`application`, `app`, `worker`, presentation-entry-points
//! `rest|grpc|cli|sdk`) MUST NOT carry module-level mutable state.
//! Mutable globals break horizontal-scale invariants (every replica diverges)
//! and the audit-chain emission rule (state changes that don't flow through a
//! port can't be audited).
//!
//! Scope of this kernel: take a list of typed [`SourceFile`] nodes pre-harvested
//! by a runner (the kernel is I/O-free) and flag occurrences of the three
//! global-mutable patterns:
//!   - `static mut` — always mutable
//!   - `lazy_static! { ... }` macro — global lazy with interior mutability
//!   - `once_cell::sync::Lazy<...>` — same family
//!
//! Comments and string literals are NOT excluded (they're false positives, but
//! cheap to false-positive-fix at PR time — reviewer can suppress one line vs.
//! the lane silently missing real state).
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;
use std::fmt;

/// Execution mode for the statelessness check lane.
///
/// `ReportOnly` prints violations but always returns success (exit 0) so early
/// substrate phases can track drift without blocking CI.  `Blocker` causes the
/// check to return a non-zero exit code when any violation is found; P22 flips
/// the lane to this mode once all known violations are resolved.

/// Layer values where statelessness is required (outer ring + entry points).
/// Inner-ring crates (`kernel`, `domain`) are exempt because they typically
/// contain only types + pure logic; if they do violate, they need a separate
/// review anyway. The check is opt-in by layer.
pub const SCOPED_LAYERS: [&str; 7] = ["application", "app", "worker", "rest", "grpc", "cli", "sdk"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    pub crate_id: String, // data_class: INTERNAL_ONLY
    pub layer: String,    // data_class: INTERNAL_ONLY ; one of SCOPED_LAYERS or a non-scoped layer
    pub path: String,     // data_class: INTERNAL_ONLY
    pub content: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ViolationKind {
    StaticMut,
    LazyStaticMacro,
    OnceCellLazy,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticMut => write!(f, "static mut"),
            Self::LazyStaticMacro => write!(f, "lazy_static!"),
            Self::OnceCellLazy => write!(f, "once_cell::sync::Lazy"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub crate_id: String,
    pub path: String,
    pub line: u32,
    pub kind: ViolationKind,
    pub excerpt: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub files_checked: usize,
    pub files_in_scope: usize,
    pub violations: Vec<Violation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    EmptyCrateId { path: String },
    EmptyPath { crate_id: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCrateId { path } => write!(f, "empty crate_id for path {path}"),
            Self::EmptyPath { crate_id } => write!(f, "empty path for crate {crate_id}"),
        }
    }
}

impl std::error::Error for Error {}

fn is_in_scope(layer: &str) -> bool {
    SCOPED_LAYERS.contains(&layer)
}

fn scan_line(line: &str) -> Option<ViolationKind> {
    let trimmed = line.trim_start();
    // Skip line comments. Multi-line comments are not handled — would need a
    // real parser; cheap heuristic is acceptable for v1 per module docstring.
    if trimmed.starts_with("//") {
        return None;
    }
    if trimmed.contains("static mut ") || trimmed.contains("static mut\t") {
        return Some(ViolationKind::StaticMut);
    }
    if trimmed.contains("lazy_static!") {
        return Some(ViolationKind::LazyStaticMacro);
    }
    if trimmed.contains("once_cell::sync::Lazy") {
        return Some(ViolationKind::OnceCellLazy);
    }
    None
}

pub fn check(files: &[SourceFile]) -> Result<Report, Error> {
    let mut seen = BTreeSet::new();
    let mut violations = Vec::new();
    let mut files_in_scope = 0usize;

    for file in files {
        if file.crate_id.trim().is_empty() {
            return Err(Error::EmptyCrateId {
                path: file.path.clone(),
            });
        }
        if file.path.trim().is_empty() {
            return Err(Error::EmptyPath {
                crate_id: file.crate_id.clone(),
            });
        }
        let key = (file.crate_id.clone(), file.path.clone());
        if !seen.insert(key) {
            // Duplicate entry — caller bug; treat as soft skip rather than error
            // since the kernel is I/O-free and shouldn't punish runner double-feeds.
            continue;
        }

        if !is_in_scope(&file.layer) {
            continue;
        }
        files_in_scope += 1;

        for (idx, line) in file.content.lines().enumerate() {
            if let Some(kind) = scan_line(line) {
                violations.push(Violation {
                    crate_id: file.crate_id.clone(),
                    path: file.path.clone(),
                    line: (idx + 1) as u32,
                    kind,
                    excerpt: line.trim().to_string(),
                });
            }
        }
    }

    Ok(Report {
        files_checked: files.len(),
        files_in_scope,
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(crate_id: &str, layer: &str, path: &str, content: &str) -> SourceFile {
        SourceFile {
            crate_id: crate_id.into(),
            layer: layer.into(),
            path: path.into(),
            content: content.into(),
        }
    }

    #[test]
    fn empty_input_passes() {
        let r = check(&[]).unwrap();
        assert!(r.violations.is_empty());
        assert_eq!(r.files_in_scope, 0);
    }

    #[test]
    fn clean_app_layer_passes() {
        let r = check(&[node(
            "x",
            "app",
            "src/main.rs",
            "fn main() {\n    let x = 1;\n}\n",
        )])
        .unwrap();
        assert!(r.violations.is_empty());
        assert_eq!(r.files_in_scope, 1);
    }

    #[test]
    fn flags_static_mut_in_app_layer() {
        let r = check(&[node(
            "x",
            "app",
            "src/main.rs",
            "fn main() {}\nstatic mut COUNTER: usize = 0;\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::StaticMut);
        assert_eq!(r.violations[0].line, 2);
    }

    #[test]
    fn flags_lazy_static_macro_in_worker() {
        let r = check(&[node(
            "x",
            "worker",
            "src/lib.rs",
            "use lazy_static::lazy_static;\nlazy_static! { static ref GLOBAL: Vec<u8> = Vec::new(); }\n",
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::LazyStaticMacro);
    }

    #[test]
    fn flags_once_cell_lazy_in_rest() {
        let r = check(&[node(
            "x",
            "rest",
            "src/lib.rs",
            "static FOO: once_cell::sync::Lazy<u32> = once_cell::sync::Lazy::new(|| 0);\n",
        )])
        .unwrap();
        // Two occurrences on the same line — scan finds it once per line, not per match.
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::OnceCellLazy);
    }

    #[test]
    fn ignores_kernel_layer() {
        let r = check(&[node(
            "x",
            "kernel",
            "src/lib.rs",
            "static mut DANGER: u8 = 0;\n",
        )])
        .unwrap();
        // Out-of-scope layer; no violation even though the pattern is present.
        assert!(r.violations.is_empty());
        assert_eq!(r.files_in_scope, 0);
    }

    #[test]
    fn ignores_domain_layer() {
        let r = check(&[node(
            "x",
            "domain",
            "src/lib.rs",
            "lazy_static! { static ref G: u8 = 0; }\n",
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn skips_comment_lines() {
        let r = check(&[node(
            "x",
            "app",
            "src/main.rs",
            "// static mut FAKE: u8 = 0;\nfn main() {}\n",
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn rejects_empty_crate_id() {
        let err = check(&[node("", "app", "src/main.rs", "")]).unwrap_err();
        assert!(matches!(err, Error::EmptyCrateId { .. }));
    }

    #[test]
    fn rejects_empty_path() {
        let err = check(&[node("x", "app", "", "")]).unwrap_err();
        assert!(matches!(err, Error::EmptyPath { .. }));
    }

    #[test]
    fn line_numbers_are_one_indexed() {
        let r = check(&[node(
            "x",
            "app",
            "src/lib.rs",
            "fn a() {}\nfn b() {}\nstatic mut S: u8 = 0;\n",
        )])
        .unwrap();
        assert_eq!(r.violations[0].line, 3);
    }
}
