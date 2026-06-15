//! Core data model: the move plan, the per-crate move, the emitted mapping, and the error
//! type. All transforms in the engine are pure functions of these.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

/// A single crate's move: where it is, where it goes, and how its package is renamed.
///
/// Paths are repo-relative, forward-slash, no trailing slash (e.g. `cloud/cloud-iam/crates/
/// oya-cloud-iam-domain` -> `iam/core/identity-domain`). Cargo names are kebab package names
/// (e.g. `oya-cloud-iam-domain` -> `identity-domain`); the snake crate name (`identity_domain`)
/// is derived deterministically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateMove {
    pub old_path: String,
    pub new_path: String,
    pub old_cargo_name: String,
    pub new_cargo_name: String,
}

impl CrateMove {
    /// The snake crate name (Cargo `[lib].name` / Rust `extern crate` ident) for the old name.
    pub fn old_crate_ident(&self) -> String {
        snake(&self.old_cargo_name)
    }

    /// The snake crate name for the new name.
    pub fn new_crate_ident(&self) -> String {
        snake(&self.new_cargo_name)
    }
}

/// A capability move: a total, ordered set of crate moves applied as one atomic unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePlan {
    /// Optional human/registry label for the capability being moved (e.g. `iam`). Carried
    /// only for the emitted mapping + reporting; never feeds a transform.
    pub capability: String,
    pub moves: Vec<CrateMove>,
}

impl MovePlan {
    /// The inverse plan: every move's old/new sides swapped. Applying the inverse to a tree
    /// the forward plan produced restores it byte-identically (reversibility-by-construction).
    pub fn inverse(&self) -> MovePlan {
        MovePlan {
            capability: self.capability.clone(),
            moves: self
                .moves
                .iter()
                .map(|m| CrateMove {
                    old_path: m.new_path.clone(),
                    new_path: m.old_path.clone(),
                    old_cargo_name: m.new_cargo_name.clone(),
                    new_cargo_name: m.old_cargo_name.clone(),
                })
                .collect(),
        }
    }

    /// Map from old repo-relative path -> the [`CrateMove`]. Deterministic order.
    pub fn by_old_path(&self) -> BTreeMap<&str, &CrateMove> {
        self.moves
            .iter()
            .map(|m| (m.old_path.as_str(), m))
            .collect()
    }

    /// Map from old kebab cargo name -> the [`CrateMove`].
    pub fn by_old_cargo_name(&self) -> BTreeMap<&str, &CrateMove> {
        self.moves
            .iter()
            .map(|m| (m.old_cargo_name.as_str(), m))
            .collect()
    }

    /// Validate the plan is internally well-formed and fail-closed on the collision classes
    /// the engine cannot resolve deterministically. Returns the first violation found.
    ///
    /// Checks:
    /// * no duplicate `old_path`, `new_path`, `old_cargo_name`, or `new_cargo_name`
    ///   (a name/target collision the move would create);
    /// * no `new_path` nests inside another move's `new_path` in a way that would shadow it
    ///   (a target-path collision);
    /// * paths are repo-relative + normalized (no `.`/`..`/leading-or-trailing slash).
    pub fn validate(&self) -> Result<(), CodemodError> {
        if self.moves.is_empty() {
            return Err(CodemodError::EmptyPlan);
        }
        let mut old_paths = BTreeSet::new();
        let mut new_paths = BTreeSet::new();
        let mut old_names = BTreeSet::new();
        let mut new_names = BTreeSet::new();
        for m in &self.moves {
            for (label, p) in [("old_path", &m.old_path), ("new_path", &m.new_path)] {
                if !is_normalized_rel_path(p) {
                    return Err(CodemodError::BadPath {
                        which: label.to_string(),
                        path: p.clone(),
                    });
                }
            }
            if !old_paths.insert(m.old_path.clone()) {
                return Err(CodemodError::DuplicateKey {
                    kind: "old_path".to_string(),
                    value: m.old_path.clone(),
                });
            }
            if !new_paths.insert(m.new_path.clone()) {
                return Err(CodemodError::DuplicateKey {
                    kind: "new_path".to_string(),
                    value: m.new_path.clone(),
                });
            }
            if !old_names.insert(m.old_cargo_name.clone()) {
                return Err(CodemodError::DuplicateKey {
                    kind: "old_cargo_name".to_string(),
                    value: m.old_cargo_name.clone(),
                });
            }
            if !new_names.insert(m.new_cargo_name.clone()) {
                return Err(CodemodError::DuplicateKey {
                    kind: "new_cargo_name".to_string(),
                    value: m.new_cargo_name.clone(),
                });
            }
        }
        // A new_path that nests strictly inside another new_path would be moved under a dir
        // the engine also moves — an ambiguous, non-deterministic destination. Reject.
        for a in &new_paths {
            for b in &new_paths {
                if a != b && a.starts_with(&format!("{b}/")) {
                    return Err(CodemodError::NestedTarget {
                        inner: a.clone(),
                        outer: b.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// The FILE-level move pairs `(old_file_path, new_file_path)` derived deterministically
    /// from the crate-DIR plan, for the rename-aware path-keyed CI baseline relabel (task #64).
    ///
    /// For each tracked file path in `tracked_paths` that lies under some move's `old_path`
    /// crate dir, the new path is `<new_path>/<rel>` where `rel` is the file path relative to
    /// `old_path`. This is EXACT because `plan.rs` step-7 performs a wholesale
    /// `git mv <old_dir> <new_dir>` (longest-first), so every file under the old dir lands at
    /// the mirror location under the new dir — there is no `-M`/`-C`/similarity heuristic.
    ///
    /// A file path EQUAL to `old_path` (no trailing component) is not a crate-dir member and is
    /// skipped; only strict descendants (`old_path/<rel>`) are mapped. The result is sorted +
    /// deduplicated (BTreeMap) so the emitted manifest is canonical and `committed==regenerated`
    /// holds byte-for-byte. `MovePlan::validate` guarantees `old_path` injectivity + no nesting,
    /// so a tracked file maps under at most one move (longest matching `old_path` would be
    /// ambiguous only for nested dirs, which validate rejects).
    pub fn file_level_manifest(&self, tracked_paths: &[String]) -> Vec<(String, String)> {
        let mut pairs: BTreeMap<String, String> = BTreeMap::new();
        for m in &self.moves {
            let old_prefix = format!("{}/", m.old_path);
            for path in tracked_paths {
                if let Some(rel) = path.strip_prefix(&old_prefix) {
                    pairs.insert(path.clone(), format!("{}/{}", m.new_path, rel));
                }
            }
        }
        pairs.into_iter().collect()
    }

    /// The crate-IDENT move pairs `(old_cargo_name, new_cargo_name)` for tier-dependency
    /// edge-endpoint mapping (task #64 correction #3). The tier-dep gate keys edges over crate
    /// IDENTS (`<code>|<from-ident> -> <to-idents>`), so the relabel maps endpoints via these
    /// kebab cargo-name pairs, NOT old_path->new_path. Sorted + deduplicated (BTreeMap) for a
    /// canonical manifest; `MovePlan::validate` guarantees `old_cargo_name` injectivity.
    pub fn crate_ident_pairs(&self) -> Vec<(String, String)> {
        let mut pairs: BTreeMap<String, String> = BTreeMap::new();
        for m in &self.moves {
            pairs.insert(m.old_cargo_name.clone(), m.new_cargo_name.clone());
        }
        pairs.into_iter().collect()
    }

    /// Build the emitted [`Mapping`] (one row per crate move) for audit + the inverse.
    pub fn mapping(&self) -> Mapping {
        Mapping {
            capability: self.capability.clone(),
            rows: self
                .moves
                .iter()
                .map(|m| MappingRow {
                    old_path: m.old_path.clone(),
                    new_path: m.new_path.clone(),
                    old_cargo_name: m.old_cargo_name.clone(),
                    new_cargo_name: m.new_cargo_name.clone(),
                    buck_label: format!("//{}:{}", m.new_path, m.new_cargo_name),
                })
                .collect(),
        }
    }
}

/// The canonical move-manifest schema id (task #64). The authoritative, committed,
/// anti-forgery-bound bijection the rename-aware path-keyed CI baseline relabel consumes.
pub const REORG_MOVE_MANIFEST_SCHEMA: &str = "oya-ci/reorg-move-manifest/v1";

/// Encode `(capability, file-level pairs, crate-ident pairs)` as the canonical-JSON
/// move-manifest `serde_json::Value` (schema [`REORG_MOVE_MANIFEST_SCHEMA`]):
///
/// ```json
/// {"schema": "...", "capability": "<cap>",
///  "files": [{"old_path": "...", "new_path": "..."}, ...],
///  "crate_idents": [{"old": "...", "new": "..."}, ...]}
/// ```
///
/// Pure + deterministic: the caller passes the SORTED outputs of
/// [`MovePlan::file_level_manifest`] + [`MovePlan::crate_ident_pairs`], so re-deriving the
/// manifest from the committed plan + candidate tree yields byte-identical bytes (the
/// `committed==regenerated` registry-drift binding). For a NO-MOVE PR the caller passes empty
/// slices, yielding `files: []`/`crate_idents: []` — the strict no-op the emitter reads as
/// "no renames" (identity relabel).
pub fn move_manifest_value(
    capability: &str,
    file_pairs: &[(String, String)],
    crate_ident_pairs: &[(String, String)],
) -> serde_json::Value {
    serde_json::json!({
        "schema": REORG_MOVE_MANIFEST_SCHEMA,
        "capability": capability,
        "files": file_pairs
            .iter()
            .map(|(old, new)| serde_json::json!({"old_path": old, "new_path": new}))
            .collect::<Vec<_>>(),
        "crate_idents": crate_ident_pairs
            .iter()
            .map(|(old, new)| serde_json::json!({"old": old, "new": new}))
            .collect::<Vec<_>>(),
    })
}

/// One audit row of the emitted mapping tuple, per ADR-0562 P0.13 spec
/// `(old_path, new_path, old_cargo_name, new_cargo_name, buck_label)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingRow {
    pub old_path: String,
    pub new_path: String,
    pub old_cargo_name: String,
    pub new_cargo_name: String,
    pub buck_label: String,
}

/// The full mapping for a capability move (invertible audit record).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping {
    pub capability: String,
    pub rows: Vec<MappingRow>,
}

/// Every failure the engine can fail-closed on. The CLI maps any of these to a non-zero
/// exit; no transform proceeds past a returned error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodemodError {
    EmptyPlan,
    BadPath { which: String, path: String },
    DuplicateKey { kind: String, value: String },
    NestedTarget { inner: String, outer: String },
    /// A relative `path=` dep could not be recomputed unambiguously (e.g. it points outside
    /// the repo root, or its target cannot be located post-move).
    AmbiguousPathDep { manifest: String, dep: String, path: String },
    /// The move would re-home a crate onto a path that already exists in the tree.
    TargetExists { path: String },
    /// A moved crate's source path does not exist.
    SourceMissing { path: String },
    /// An IO failure while reading/writing/moving.
    Io { context: String, message: String },
    /// A `Cargo.toml`/`BUCK` could not be parsed.
    Parse { path: String, message: String },
    /// `cargo metadata` did not resolve in the dry-run sandbox (fail-closed).
    CargoUnresolved { message: String },
    /// `buck2 targets //...` did not resolve in the dry-run sandbox (fail-closed).
    BuckUnresolved { message: String },
}

impl fmt::Display for CodemodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodemodError::EmptyPlan => write!(f, "move plan is empty"),
            CodemodError::BadPath { which, path } => {
                write!(f, "{which} is not a normalized repo-relative path: {path:?}")
            }
            CodemodError::DuplicateKey { kind, value } => {
                write!(f, "duplicate {kind} in move plan: {value:?} (collision)")
            }
            CodemodError::NestedTarget { inner, outer } => write!(
                f,
                "target path {inner:?} nests inside another moved target {outer:?} (ambiguous destination)"
            ),
            CodemodError::AmbiguousPathDep {
                manifest,
                dep,
                path,
            } => write!(
                f,
                "ambiguous relative path-dep recompute in {manifest}: dep {dep:?} path {path:?}"
            ),
            CodemodError::TargetExists { path } => {
                write!(f, "move target already exists: {path:?}")
            }
            CodemodError::SourceMissing { path } => {
                write!(f, "move source missing: {path:?}")
            }
            CodemodError::Io { context, message } => write!(f, "io ({context}): {message}"),
            CodemodError::Parse { path, message } => write!(f, "parse {path}: {message}"),
            CodemodError::CargoUnresolved { message } => {
                write!(f, "cargo metadata did not resolve (fail-closed): {message}")
            }
            CodemodError::BuckUnresolved { message } => {
                write!(f, "buck2 targets //... did not resolve (fail-closed): {message}")
            }
        }
    }
}

impl std::error::Error for CodemodError {}

/// Convert a kebab cargo/package name to the snake crate identifier (`oya-foo-bar` ->
/// `oya_foo_bar`). This mirrors Cargo's own `[lib].name` defaulting and the `extern crate`
/// ident rule: only `-` becomes `_`; all other characters pass through.
pub fn snake(kebab: &str) -> String {
    kebab.replace('-', "_")
}

/// True if `p` is a normalized, repo-relative, forward-slash path with no `.`/`..`
/// components and no leading/trailing slash. Empty is rejected.
pub fn is_normalized_rel_path(p: &str) -> bool {
    if p.is_empty() || p.starts_with('/') || p.ends_with('/') || p.contains('\\') {
        return false;
    }
    p.split('/')
        .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// Recompute a relative `path=` dependency for a manifest that moved from `old_manifest_dir`
/// to `new_manifest_dir`, where the dependency originally pointed (relative to
/// `old_manifest_dir`) at `rel_dep`. The dependency target may ITSELF have moved; the
/// caller supplies `resolve_target` to map an old target-dir (repo-relative) to its new
/// repo-relative location (identity when the target did not move).
///
/// Returns the new relative path string (forward-slash) the moved manifest must use, or
/// `None` when the recompute is ambiguous (the dep resolves outside the repo root) — the
/// caller maps `None` to a fail-closed [`CodemodError::AmbiguousPathDep`].
///
/// This is the pure core of the move-fatal `../../../` recompute. It is a function of three
/// repo-relative dirs only — no filesystem access — so it is fully deterministic.
pub fn recompute_rel_path_dep(
    old_manifest_dir: &str,
    new_manifest_dir: &str,
    rel_dep: &str,
    resolve_target: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    // 1. Resolve the OLD absolute (repo-relative) target the dep currently points at.
    let old_target = join_rel(old_manifest_dir, rel_dep)?;
    // 2. Map it through the move (the target itself may be moving).
    let new_target = resolve_target(&old_target).unwrap_or(old_target);
    // 3. Compute the relative path from the NEW manifest dir to the NEW target.
    Some(rel_path_between(new_manifest_dir, &new_target))
}

/// Join a repo-relative base dir with a relative path that may contain `..`, normalizing the
/// result to a repo-relative path. Returns `None` if it escapes the repo root.
pub fn join_rel(base: &str, rel: &str) -> Option<String> {
    let mut stack: Vec<&str> = if base.is_empty() {
        Vec::new()
    } else {
        base.split('/').filter(|s| !s.is_empty()).collect()
    };
    for seg in rel.split('/') {
        match seg {
            "" | "." => continue,
            ".." => {
                // escaping above the repo root yields None.
                stack.pop()?;
            }
            other => stack.push(other),
        }
    }
    Some(stack.join("/"))
}

/// The relative path (forward-slash, using `..`) from repo-relative dir `from` to
/// repo-relative dir/file `to`. Pure; mirrors `pathdiff` for normalized inputs.
pub fn rel_path_between(from: &str, to: &str) -> String {
    let from_parts: Vec<&str> = from.split('/').filter(|s| !s.is_empty()).collect();
    let to_parts: Vec<&str> = to.split('/').filter(|s| !s.is_empty()).collect();
    let common = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let ups = from_parts.len() - common;
    let mut out: Vec<String> = std::iter::repeat_n("..".to_string(), ups).collect();
    out.extend(to_parts[common..].iter().map(|s| s.to_string()));
    if out.is_empty() {
        ".".to_string()
    } else {
        out.join("/")
    }
}

/// True if `p` exists as a directory on disk.
pub fn dir_exists(root: &Path, rel: &str) -> bool {
    root.join(rel).is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_only_converts_dashes() {
        assert_eq!(snake("oya-cloud-iam-domain"), "oya_cloud_iam_domain");
        assert_eq!(snake("identity-domain"), "identity_domain");
        assert_eq!(snake("already_snake"), "already_snake");
    }

    #[test]
    fn normalized_path_rejects_dotdot_and_slashes() {
        assert!(is_normalized_rel_path("iam/core/identity-domain"));
        assert!(!is_normalized_rel_path("/abs"));
        assert!(!is_normalized_rel_path("trailing/"));
        assert!(!is_normalized_rel_path("a/../b"));
        assert!(!is_normalized_rel_path("a/./b"));
        assert!(!is_normalized_rel_path(""));
        assert!(!is_normalized_rel_path("win\\path"));
    }

    #[test]
    fn join_rel_normalizes_dotdot() {
        assert_eq!(
            join_rel("cloud/cloud-iam/crates/oya-iam", "../oya-domain").as_deref(),
            Some("cloud/cloud-iam/crates/oya-domain")
        );
        assert_eq!(
            join_rel(
                "cloud/cloud-iam/crates/oya-iam",
                "../../../../libs/oya-kernel"
            )
            .as_deref(),
            Some("libs/oya-kernel")
        );
        assert_eq!(join_rel("a", "../../b"), None, "escaping repo root is None");
    }

    #[test]
    fn rel_path_between_is_inverse_of_join() {
        let from = "iam/core";
        let to = "base/oya-kernel";
        let rel = rel_path_between(from, to);
        assert_eq!(rel, "../../base/oya-kernel");
        assert_eq!(join_rel(from, &rel).as_deref(), Some(to));
    }

    #[test]
    fn recompute_handles_moved_manifest_and_unmoved_target() {
        // manifest moves cloud/cloud-iam/crates/a -> iam/core/a; dep at ../b stays at
        // cloud/cloud-iam/crates/b (target did not move).
        let new = recompute_rel_path_dep(
            "cloud/cloud-iam/crates/a",
            "iam/core/a",
            "../b",
            &|_old| None,
        )
        .unwrap();
        // iam/core/a has 3 segments -> 3 `..` to root, then down to the unmoved target.
        assert_eq!(new, "../../../cloud/cloud-iam/crates/b");
    }

    #[test]
    fn recompute_handles_both_manifest_and_target_moving() {
        // both a (manifest) and b (target) move into iam/core.
        let new = recompute_rel_path_dep(
            "cloud/cloud-iam/crates/a",
            "iam/core/a",
            "../b",
            &|old| {
                if old == "cloud/cloud-iam/crates/b" {
                    Some("iam/core/b".to_string())
                } else {
                    None
                }
            },
        )
        .unwrap();
        assert_eq!(new, "../b", "both moved into the same dir -> sibling again");
    }

    #[test]
    fn recompute_deep_path_dep_to_libs() {
        // The move-fatal class: ../../../../libs/oya-kernel from a deep crate.
        let new = recompute_rel_path_dep(
            "cloud/cloud-iam/crates/oya-iam-app",
            "iam/facade/iam-app",
            "../../../../libs/oya-kernel",
            &|_old| None,
        )
        .unwrap();
        // iam/facade/iam-app -> libs/oya-kernel
        assert_eq!(new, "../../../libs/oya-kernel");
        assert_eq!(
            join_rel("iam/facade/iam-app", &new).as_deref(),
            Some("libs/oya-kernel")
        );
    }

    #[test]
    fn inverse_swaps_every_side() {
        let plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-iam/crates/oya-iam".to_string(),
                new_path: "iam/core/iam".to_string(),
                old_cargo_name: "oya-cloud-iam".to_string(),
                new_cargo_name: "iam-core".to_string(),
            }],
        };
        let inv = plan.inverse();
        assert_eq!(inv.moves[0].old_path, "iam/core/iam");
        assert_eq!(inv.moves[0].new_path, "cloud/cloud-iam/crates/oya-iam");
        assert_eq!(inv.moves[0].old_cargo_name, "iam-core");
        assert_eq!(inv.moves[0].new_cargo_name, "oya-cloud-iam");
        // inverse of inverse is identity.
        assert_eq!(inv.inverse().moves, plan.moves);
    }

    #[test]
    fn validate_rejects_duplicate_new_name_collision() {
        let plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![
                CrateMove {
                    old_path: "cloud/a".to_string(),
                    new_path: "iam/core/a".to_string(),
                    old_cargo_name: "oya-a".to_string(),
                    new_cargo_name: "collide".to_string(),
                },
                CrateMove {
                    old_path: "cloud/b".to_string(),
                    new_path: "iam/core/b".to_string(),
                    old_cargo_name: "oya-b".to_string(),
                    new_cargo_name: "collide".to_string(),
                },
            ],
        };
        assert!(matches!(
            plan.validate(),
            Err(CodemodError::DuplicateKey { kind, .. }) if kind == "new_cargo_name"
        ));
    }

    #[test]
    fn validate_rejects_nested_target() {
        let plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![
                CrateMove {
                    old_path: "cloud/a".to_string(),
                    new_path: "iam/core".to_string(),
                    old_cargo_name: "oya-a".to_string(),
                    new_cargo_name: "a".to_string(),
                },
                CrateMove {
                    old_path: "cloud/b".to_string(),
                    new_path: "iam/core/b".to_string(),
                    old_cargo_name: "oya-b".to_string(),
                    new_cargo_name: "b".to_string(),
                },
            ],
        };
        assert!(matches!(
            plan.validate(),
            Err(CodemodError::NestedTarget { .. })
        ));
    }

    #[test]
    fn file_level_manifest_mirrors_dir_move_for_tracked_descendants() {
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-observability/crates/oya-cloud-observability-domain".to_string(),
                new_path: "observability/core/aggregate".to_string(),
                old_cargo_name: "oya-cloud-observability-domain".to_string(),
                new_cargo_name: "observability-core-aggregate".to_string(),
            }],
        };
        let tracked = vec![
            "cloud/cloud-observability/crates/oya-cloud-observability-domain/src/lib.rs".to_string(),
            "cloud/cloud-observability/crates/oya-cloud-observability-domain/Cargo.toml".to_string(),
            // unrelated file (different crate) — must NOT map
            "cloud/cloud-observability/crates/oya-cloud-observability-api/src/lib.rs".to_string(),
            // a file equal to the dir name prefix but not a strict descendant — must NOT map
            "cloud/cloud-observability/crates/oya-cloud-observability-domain-extra/x.rs".to_string(),
        ];
        let pairs = plan.file_level_manifest(&tracked);
        assert_eq!(
            pairs,
            vec![
                (
                    "cloud/cloud-observability/crates/oya-cloud-observability-domain/Cargo.toml".to_string(),
                    "observability/core/aggregate/Cargo.toml".to_string()
                ),
                (
                    "cloud/cloud-observability/crates/oya-cloud-observability-domain/src/lib.rs".to_string(),
                    "observability/core/aggregate/src/lib.rs".to_string()
                ),
            ],
            "only strict descendants map, mirror-located, sorted"
        );
    }

    #[test]
    fn file_level_manifest_is_empty_for_no_moves_or_no_tracked() {
        let plan = MovePlan {
            capability: "x".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/a".to_string(),
                new_path: "x/core/a".to_string(),
                old_cargo_name: "oya-a".to_string(),
                new_cargo_name: "x-core-a".to_string(),
            }],
        };
        assert!(plan.file_level_manifest(&[]).is_empty(), "no tracked => no pairs");
    }

    #[test]
    fn crate_ident_pairs_exposes_cargo_name_bijection() {
        let plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![
                CrateMove {
                    old_path: "cloud/cloud-iam/crates/oya-cloud-iam-domain".to_string(),
                    new_path: "iam/core/identity-domain".to_string(),
                    old_cargo_name: "oya-cloud-iam-domain".to_string(),
                    new_cargo_name: "identity-domain".to_string(),
                },
                CrateMove {
                    old_path: "cloud/cloud-iam/crates/oya-cloud-iam-app".to_string(),
                    new_path: "iam/facade/iam-app".to_string(),
                    old_cargo_name: "oya-cloud-iam-app".to_string(),
                    new_cargo_name: "iam-app".to_string(),
                },
            ],
        };
        assert_eq!(
            plan.crate_ident_pairs(),
            vec![
                ("oya-cloud-iam-app".to_string(), "iam-app".to_string()),
                ("oya-cloud-iam-domain".to_string(), "identity-domain".to_string()),
            ],
            "sorted by old cargo name"
        );
    }

    #[test]
    fn move_manifest_value_is_canonical_and_empty_for_no_move() {
        let empty = move_manifest_value("", &[], &[]);
        assert_eq!(empty["schema"], REORG_MOVE_MANIFEST_SCHEMA);
        assert_eq!(empty["capability"], "");
        assert_eq!(empty["files"], serde_json::json!([]));
        assert_eq!(empty["crate_idents"], serde_json::json!([]));

        let full = move_manifest_value(
            "observability",
            &[("old/a.rs".to_string(), "new/a.rs".to_string())],
            &[("oya-old".to_string(), "new-core".to_string())],
        );
        assert_eq!(full["files"][0]["old_path"], "old/a.rs");
        assert_eq!(full["files"][0]["new_path"], "new/a.rs");
        assert_eq!(full["crate_idents"][0]["old"], "oya-old");
        assert_eq!(full["crate_idents"][0]["new"], "new-core");
    }

    #[test]
    fn mapping_emits_the_five_tuple() {
        let plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-iam/crates/oya-iam".to_string(),
                new_path: "iam/core/iam".to_string(),
                old_cargo_name: "oya-cloud-iam".to_string(),
                new_cargo_name: "iam-core".to_string(),
            }],
        };
        let mapping = plan.mapping();
        let row = &mapping.rows[0];
        assert_eq!(row.old_path, "cloud/cloud-iam/crates/oya-iam");
        assert_eq!(row.new_path, "iam/core/iam");
        assert_eq!(row.old_cargo_name, "oya-cloud-iam");
        assert_eq!(row.new_cargo_name, "iam-core");
        assert_eq!(row.buck_label, "//iam/core/iam:iam-core");
    }
}
