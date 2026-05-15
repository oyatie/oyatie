//! Architecture-map freshness fitness kernel — blocks PRs that touch
//! a source-of-truth registry without regenerating the corresponding
//! architecture map / visualization output.
//!
//! I/O-free. Runners hand the kernel:
//! - the digest of the on-disk architecture-map snapshot
//! - the digest computed from the live workspace state
//! - the set of files changed in the PR
//! - the set of source-of-truth glob roots that drive the map
//!
//! Returns a violation if either (a) the snapshot digest differs from
//! the live digest, or (b) a source-of-truth file changed but no
//! snapshot file was regenerated in the same PR.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessInput<'a> {
    pub snapshot_digest_sha256: &'a str, // data_class: INTERNAL_ONLY
    pub live_digest_sha256: &'a str,     // data_class: INTERNAL_ONLY
    pub snapshot_paths: &'a [String],    // data_class: INTERNAL_ONLY
    pub source_of_truth_roots: &'a [String], // data_class: INTERNAL_ONLY
    pub changed_files: &'a [String],     // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FreshnessViolationKind {
    DigestMismatch,
    SourceChangedButSnapshotNotRegenerated,
}

impl FreshnessViolationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DigestMismatch => "snapshot digest does not match live workspace",
            Self::SourceChangedButSnapshotNotRegenerated => {
                "source-of-truth changed but snapshot not regenerated"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessViolation {
    pub kind: FreshnessViolationKind, // data_class: INTERNAL_ONLY
    pub detail: String,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshnessReport {
    pub violations: Vec<FreshnessViolation>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FreshnessError {
    EmptyDigest { which: &'static str },
    NotSha256Hex { which: &'static str },
    NoSnapshotPaths,
    NoSourceRoots,
}

impl FreshnessError {
    pub fn message(&self) -> String {
        match self {
            Self::EmptyDigest { which } => format!("empty {which} digest"),
            Self::NotSha256Hex { which } => {
                format!("{which} digest is not 64-char lowercase sha256 hex")
            }
            Self::NoSnapshotPaths => "no snapshot paths declared".to_owned(),
            Self::NoSourceRoots => "no source-of-truth roots declared".to_owned(),
        }
    }
}

fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn path_under_root(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{root}/"))
}

pub fn check(input: FreshnessInput<'_>) -> Result<FreshnessReport, FreshnessError> {
    if input.snapshot_digest_sha256.is_empty() {
        return Err(FreshnessError::EmptyDigest { which: "snapshot" });
    }
    if input.live_digest_sha256.is_empty() {
        return Err(FreshnessError::EmptyDigest { which: "live" });
    }
    if !is_sha256_hex(input.snapshot_digest_sha256) {
        return Err(FreshnessError::NotSha256Hex { which: "snapshot" });
    }
    if !is_sha256_hex(input.live_digest_sha256) {
        return Err(FreshnessError::NotSha256Hex { which: "live" });
    }
    if input.snapshot_paths.is_empty() {
        return Err(FreshnessError::NoSnapshotPaths);
    }
    if input.source_of_truth_roots.is_empty() {
        return Err(FreshnessError::NoSourceRoots);
    }

    let mut violations = Vec::new();

    if input.snapshot_digest_sha256 != input.live_digest_sha256 {
        violations.push(FreshnessViolation {
            kind: FreshnessViolationKind::DigestMismatch,
            detail: format!(
                "snapshot={} live={}",
                input.snapshot_digest_sha256, input.live_digest_sha256
            ),
        });
    }

    let snapshot_touched = input
        .changed_files
        .iter()
        .any(|f| input.snapshot_paths.iter().any(|s| s == f));

    let source_touched: Vec<&String> = input
        .changed_files
        .iter()
        .filter(|f| {
            input
                .source_of_truth_roots
                .iter()
                .any(|r| path_under_root(f, r))
        })
        .collect();

    if !source_touched.is_empty() && !snapshot_touched {
        let mut detail = source_touched
            .iter()
            .map(|s| (*s).clone())
            .collect::<Vec<_>>();
        detail.sort();
        violations.push(FreshnessViolation {
            kind: FreshnessViolationKind::SourceChangedButSnapshotNotRegenerated,
            detail: detail.join(","),
        });
    }

    Ok(FreshnessReport { violations })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> String {
        "a".repeat(64)
    }

    fn input<'a>(
        s: &'a str,
        l: &'a str,
        snap: &'a [String],
        roots: &'a [String],
        changed: &'a [String],
    ) -> FreshnessInput<'a> {
        FreshnessInput {
            snapshot_digest_sha256: s,
            live_digest_sha256: l,
            snapshot_paths: snap,
            source_of_truth_roots: roots,
            changed_files: changed,
        }
    }

    #[test]
    fn matching_digests_no_source_change_passes() {
        let snap = vec!["registries/cross-cutting/graph/architecture-map.json".to_owned()];
        let roots = vec!["registries/cross-cutting".to_owned()];
        let changed: Vec<String> = vec![];
        let g = good();
        let r = check(input(&g, &g, &snap, &roots, &changed)).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn digest_mismatch_flagged() {
        let snap = vec!["registries/cross-cutting/graph/architecture-map.json".to_owned()];
        let roots = vec!["registries/cross-cutting".to_owned()];
        let changed: Vec<String> = vec![];
        let s = good();
        let l = "b".repeat(64);
        let r = check(input(&s, &l, &snap, &roots, &changed)).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == FreshnessViolationKind::DigestMismatch)
        );
    }

    #[test]
    fn source_changed_snapshot_not_touched_flagged() {
        let snap = vec!["registries/cross-cutting/graph/architecture-map.json".to_owned()];
        let roots = vec!["registries/cross-cutting".to_owned()];
        let changed = vec!["registries/cross-cutting/microservices.json".to_owned()];
        let g = good();
        let r = check(input(&g, &g, &snap, &roots, &changed)).unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.kind == FreshnessViolationKind::SourceChangedButSnapshotNotRegenerated)
        );
    }

    #[test]
    fn source_and_snapshot_both_touched_passes() {
        let snap = vec!["registries/cross-cutting/graph/architecture-map.json".to_owned()];
        let roots = vec!["registries/cross-cutting".to_owned()];
        let changed = vec![
            "registries/cross-cutting/microservices.json".to_owned(),
            "registries/cross-cutting/graph/architecture-map.json".to_owned(),
        ];
        let g = good();
        let r = check(input(&g, &g, &snap, &roots, &changed)).unwrap();
        assert!(r.violations.is_empty(), "{:?}", r.violations);
    }

    #[test]
    fn unrelated_file_changed_does_not_trigger() {
        let snap = vec!["registries/cross-cutting/graph/architecture-map.json".to_owned()];
        let roots = vec!["registries/cross-cutting".to_owned()];
        let changed = vec!["README.md".to_owned()];
        let g = good();
        let r = check(input(&g, &g, &snap, &roots, &changed)).unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn empty_snapshot_digest_errors() {
        let snap = vec!["s".to_owned()];
        let roots = vec!["r".to_owned()];
        let changed: Vec<String> = vec![];
        let g = good();
        let err = check(input("", &g, &snap, &roots, &changed)).unwrap_err();
        assert!(matches!(
            err,
            FreshnessError::EmptyDigest { which: "snapshot" }
        ));
    }

    #[test]
    fn empty_live_digest_errors() {
        let snap = vec!["s".to_owned()];
        let roots = vec!["r".to_owned()];
        let changed: Vec<String> = vec![];
        let g = good();
        let err = check(input(&g, "", &snap, &roots, &changed)).unwrap_err();
        assert!(matches!(err, FreshnessError::EmptyDigest { which: "live" }));
    }

    #[test]
    fn invalid_hex_errors() {
        let snap = vec!["s".to_owned()];
        let roots = vec!["r".to_owned()];
        let changed: Vec<String> = vec![];
        let bad = "Z".repeat(64);
        let err = check(input(&bad, &bad, &snap, &roots, &changed)).unwrap_err();
        assert!(matches!(err, FreshnessError::NotSha256Hex { .. }));
    }

    #[test]
    fn no_snapshot_paths_errors() {
        let snap: Vec<String> = vec![];
        let roots = vec!["r".to_owned()];
        let changed: Vec<String> = vec![];
        let g = good();
        let err = check(input(&g, &g, &snap, &roots, &changed)).unwrap_err();
        assert!(matches!(err, FreshnessError::NoSnapshotPaths));
    }

    #[test]
    fn no_source_roots_errors() {
        let snap = vec!["s".to_owned()];
        let roots: Vec<String> = vec![];
        let changed: Vec<String> = vec![];
        let g = good();
        let err = check(input(&g, &g, &snap, &roots, &changed)).unwrap_err();
        assert!(matches!(err, FreshnessError::NoSourceRoots));
    }
}
