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

/// A NON-crate capability artifact's wholesale move: a file or directory that travels WITH a
/// capability move but carries no cargo/buck/rust identifiers to rewrite (e.g. promotion-gating
/// SLOs `<cap>/observability/slos/*.openslo.yaml`, sell-catalog records). Paths are
/// repo-relative, forward-slash, no trailing slash — same shape as [`CrateMove`] paths. The
/// engine moves these content-preserving (`git mv` wholesale, no in-file rewrite), so an
/// orphaned SLO stem is co-moved to the live capability stem instead of being stranded at a dead
/// path (the doctrine-fix this enables; see ADR-0139 SLO-home convention + ADR-0563 §C2).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArtifactMove {
    pub old_path: String,
    pub new_path: String,
}

/// A capability move: a total, ordered set of crate moves applied as one atomic unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePlan {
    /// Optional human/registry label for the capability being moved (e.g. `iam`). Carried
    /// only for the emitted mapping + reporting; never feeds a transform.
    pub capability: String,
    pub moves: Vec<CrateMove>,
    /// NON-crate artifacts co-moved with this capability (SLOs, catalog records). Default
    /// EMPTY: a plan without artifacts behaves byte-identically to a pre-ArtifactMove plan
    /// (back-compat no-op — the marketplace plan's 4-field shape has no `artifacts`).
    pub artifacts: Vec<ArtifactMove>,
}

impl MovePlan {
    /// The inverse plan: every move's old/new sides swapped (crate moves AND artifact moves).
    /// Applying the inverse to a tree the forward plan produced restores it byte-identically
    /// (reversibility-by-construction).
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
            artifacts: self
                .artifacts
                .iter()
                .map(|a| ArtifactMove {
                    old_path: a.new_path.clone(),
                    new_path: a.old_path.clone(),
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
    /// * paths are repo-relative + normalized (no `.`/`..`/leading-or-trailing slash);
    /// * source-branded de-brand moves do not keep deprecated leading `oya` / `cloud`
    ///   targets (`old_path` / `old_cargo_name` may carry the legacy brand; `new_path` /
    ///   `new_cargo_name` must not);
    /// * each artifact `old_path`/`new_path` is a normalized repo-relative path AND collides
    ///   with NO other `old_path`/`new_path` ACROSS both `moves` and `artifacts` (no two sources
    ///   map in, no two map out — the cross-service SLO-name collision backstop, fail-closed).
    pub fn validate(&self) -> Result<(), CodemodError> {
        // A plan is empty only when it carries NEITHER crate moves NOR non-crate artifact
        // co-moves. An artifact-ONLY plan (`moves: []`, `artifacts: [...]`) is well-formed:
        // it is the PR-B backfill shape — relocating orphaned SLOs + re-keying catalog records
        // for ALREADY-moved capabilities, where no crate moves in this PR. (PR-A added
        // ArtifactMove but left this guard keyed on `moves` alone; the artifact-only design
        // contract — design §PR-B "artifact-ONLY plans (zero crate moves)" — needs both sides
        // empty to be the no-op the EmptyPlan error is meant to catch.)
        if self.moves.is_empty() && self.artifacts.is_empty() {
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
            if is_deprecated_brand_path_source(&m.old_path)
                && is_deprecated_brand_path_target(&m.new_path)
            {
                return Err(CodemodError::DeprecatedBrandTarget {
                    which: "new_path".to_string(),
                    value: m.new_path.clone(),
                });
            }
            if is_deprecated_brand_name(&m.old_cargo_name)
                && is_deprecated_brand_name(&m.new_cargo_name)
            {
                return Err(CodemodError::DeprecatedBrandTarget {
                    which: "new_cargo_name".to_string(),
                    value: m.new_cargo_name.clone(),
                });
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
        // Artifacts share the old_path/new_path collision spaces with crate moves: an artifact
        // source that equals a crate source (or another artifact source) maps two things in;
        // an artifact target that equals a crate target maps two things out. Both are
        // non-deterministic destinations the engine cannot resolve — reject (the dup-new_path
        // fail-closed is also the cross-service SLO-name-collision backstop, design §PR-B ⚠).
        for a in &self.artifacts {
            for (label, p) in [("old_path", &a.old_path), ("new_path", &a.new_path)] {
                if !is_normalized_rel_path(p) {
                    return Err(CodemodError::BadPath {
                        which: format!("artifact {label}"),
                        path: p.clone(),
                    });
                }
            }
            if is_deprecated_brand_artifact_source(&a.old_path)
                && is_deprecated_brand_artifact_target(&a.new_path)
            {
                return Err(CodemodError::DeprecatedBrandTarget {
                    which: "artifact new_path".to_string(),
                    value: a.new_path.clone(),
                });
            }
            if !old_paths.insert(a.old_path.clone()) {
                return Err(CodemodError::DuplicateKey {
                    kind: "old_path".to_string(),
                    value: a.old_path.clone(),
                });
            }
            if !new_paths.insert(a.new_path.clone()) {
                return Err(CodemodError::DuplicateKey {
                    kind: "new_path".to_string(),
                    value: a.new_path.clone(),
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

    /// Forward committed de-brand policy gate: reject any move target that introduces or keeps
    /// deprecated leading `oya` / `cloud` names. Unlike [`MovePlan::validate`], this is direction-
    /// strict and intentionally independent of whether the old/source side was branded; the
    /// reversible `apply --revert` path still uses the structural validator so rollback can move
    /// back to a legacy source shape.
    pub fn validate_debrand_targets(&self) -> Result<(), CodemodError> {
        for m in &self.moves {
            if is_deprecated_brand_path_target(&m.new_path) {
                return Err(CodemodError::DeprecatedBrandTarget {
                    which: "new_path".to_string(),
                    value: m.new_path.clone(),
                });
            }
            if is_deprecated_brand_name(&m.new_cargo_name) {
                return Err(CodemodError::DeprecatedBrandTarget {
                    which: "new_cargo_name".to_string(),
                    value: m.new_cargo_name.clone(),
                });
            }
        }
        for a in &self.artifacts {
            if is_deprecated_brand_artifact_target(&a.new_path) {
                return Err(CodemodError::DeprecatedBrandTarget {
                    which: "artifact new_path".to_string(),
                    value: a.new_path.clone(),
                });
            }
        }
        Ok(())
    }

    /// The FILE-level move pairs `(old_file_path, new_file_path)` derived deterministically
    /// from the crate-DIR plan, for the rename-aware path-keyed CI baseline relabel (task #64).
    ///
    /// CANDIDATE-SIDE DERIVATION (the BLOCKER-A fix): `tracked_paths` is the candidate
    /// POST-move tree (`git ls-files` AFTER the codemod's wholesale `git mv <old_dir>
    /// <new_dir>`), so `old_path` is GONE from it and only `new_path` is present. We therefore
    /// enumerate strict descendants of each move's `new_path` (which ARE in the candidate tree)
    /// and map each back to its mirror `old_path/<rel>`, emitting `(old_path/rel -> new_path/rel)`.
    /// This is EXACT because `plan.rs` step-7 is a wholesale `git mv <old_dir> <new_dir>`
    /// (longest-first), so every file under the old dir lands at the mirror location under the
    /// new dir — there is no `-M`/`-C`/similarity heuristic. The result has `new_path` PRESENT in
    /// the candidate (the emitter's P3 holds) and `old_path` ABSENT from it (P2 holds, the dir
    /// moved away). (Enumerating `old_path` descendants over the candidate tree — the previous,
    /// structurally-inert form — found ZERO descendants and emitted an empty manifest, so the
    /// relabel could never fire for a real move.)
    ///
    /// A file path EQUAL to `new_path` (no trailing component) is not a crate-dir member and is
    /// skipped; only strict descendants (`new_path/<rel>`) are mapped. The result is sorted +
    /// deduplicated (BTreeMap, keyed by old_path) so the emitted manifest is canonical and
    /// `committed==regenerated` holds byte-for-byte. `MovePlan::validate` guarantees `new_path`
    /// injectivity + no nesting, so a candidate file maps under at most one move.
    pub fn file_level_manifest(&self, tracked_paths: &[String]) -> Vec<(String, String)> {
        let mut pairs: BTreeMap<String, String> = BTreeMap::new();
        for m in &self.moves {
            let new_prefix = format!("{}/", m.new_path);
            for path in tracked_paths {
                if let Some(rel) = path.strip_prefix(&new_prefix) {
                    pairs.insert(format!("{}/{}", m.old_path, rel), path.clone());
                }
            }
        }
        pairs.into_iter().collect()
    }

    /// The FILE-level move pairs for the NON-crate [`ArtifactMove`]s, derived candidate-side the
    /// same way [`file_level_manifest`] derives crate pairs (the post-move tree carries NEW
    /// paths; OLD is gone). For each artifact:
    /// * DIR artifact — enumerate strict descendants of `new_path` present in `tracked_paths`
    ///   and map each back to `old_path/<rel>` (mirrors the crate-dir logic);
    /// * FILE artifact — when `new_path` is present in `tracked_paths` AS AN EXACT path (a
    ///   tracked file, not a dir), emit the single `(old_path, new_path)` pair directly.
    ///
    /// These pairs are MERGED into the manifest `files` list at the call site (alongside
    /// [`file_level_manifest`]) so ADR-0563's path-keyed relabel + the total-accounting follow
    /// co-moved artifacts. Sorted + deduplicated (BTreeMap keyed by old_path); `validate`
    /// guarantees artifact-path injectivity across moves+artifacts, so a candidate path maps
    /// under at most one artifact (and never collides with a crate pair).
    pub fn artifact_file_pairs(&self, tracked_paths: &[String]) -> Vec<(String, String)> {
        let mut pairs: BTreeMap<String, String> = BTreeMap::new();
        for a in &self.artifacts {
            // FILE artifact: new_path is itself a tracked path (exact match) — emit the pair.
            let is_file = tracked_paths.iter().any(|p| p == &a.new_path);
            if is_file {
                pairs.insert(a.old_path.clone(), a.new_path.clone());
                continue;
            }
            // DIR artifact: map each strict NEW-dir descendant back to old_path/<rel>.
            let new_prefix = format!("{}/", a.new_path);
            for path in tracked_paths {
                if let Some(rel) = path.strip_prefix(&new_prefix) {
                    pairs.insert(format!("{}/{}", a.old_path, rel), path.clone());
                }
            }
        }
        pairs.into_iter().collect()
    }

    /// The crate-DIR move pairs `(old_crate_dir, new_crate_dir)` for the existence-only relabel
    /// of the total-accounting / target-parity gates, which key by crate-DIR / `member_path`
    /// (task #64 Section C). Unlike [`file_level_manifest`], these are pure plan pairs (one per
    /// move) — the gate key IS the crate dir, not a file under it — so no tracked-tree
    /// enumeration is needed. Emitted only for moves whose `new_path` actually landed in the
    /// candidate tree (a strict descendant exists), so a crate-DIR pair is never emitted for a
    /// move that did not take effect. Sorted + deduplicated (BTreeMap) for a canonical manifest;
    /// `MovePlan::validate` guarantees `old_path` injectivity.
    pub fn crate_dir_pairs(&self, tracked_paths: &[String]) -> Vec<(String, String)> {
        let mut pairs: BTreeMap<String, String> = BTreeMap::new();
        for m in &self.moves {
            let new_prefix = format!("{}/", m.new_path);
            // Only emit the crate-DIR pair when the move actually landed (some file is under
            // new_path in the candidate tree) — mirrors the file-level candidate-side guard so a
            // no-effect move never emits a pair.
            if tracked_paths.iter().any(|p| p.starts_with(&new_prefix)) {
                pairs.insert(m.old_path.clone(), m.new_path.clone());
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

/// Encode `(capability, file-level pairs, crate-dir pairs, crate-ident pairs)` as the
/// canonical-JSON move-manifest `serde_json::Value` (schema [`REORG_MOVE_MANIFEST_SCHEMA`]):
///
/// ```json
/// {"schema": "...", "capability": "<cap>",
///  "files": [{"old_path": "...", "new_path": "..."}, ...],
///  "crate_dirs": [{"old_path": "...", "new_path": "..."}, ...],
///  "crate_idents": [{"old": "...", "new": "..."}, ...]}
/// ```
///
/// `files` drives the path-keyed brand-residue relabel; `crate_dirs` drives the
/// existence-only total-accounting / target-parity relabel (those gates key by crate-DIR /
/// `member_path`, task #64 Section C); `crate_idents` drives the tier-dep endpoint mapping.
///
/// Pure + deterministic: the caller passes the SORTED outputs of
/// [`MovePlan::file_level_manifest`] + [`MovePlan::crate_dir_pairs`] +
/// [`MovePlan::crate_ident_pairs`], so re-deriving the manifest from the committed plan +
/// candidate tree yields byte-identical bytes (the `committed==regenerated` registry-drift
/// binding). For a NO-MOVE PR the caller passes empty slices, yielding `files: []`/`crate_dirs:
/// []`/`crate_idents: []` — the strict no-op the emitter reads as "no renames" (identity relabel).
pub fn move_manifest_value(
    capability: &str,
    file_pairs: &[(String, String)],
    crate_dir_pairs: &[(String, String)],
    crate_ident_pairs: &[(String, String)],
) -> serde_json::Value {
    serde_json::json!({
        "schema": REORG_MOVE_MANIFEST_SCHEMA,
        "capability": capability,
        "files": file_pairs
            .iter()
            .map(|(old, new)| serde_json::json!({"old_path": old, "new_path": new}))
            .collect::<Vec<_>>(),
        "crate_dirs": crate_dir_pairs
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
    /// A de-brand move target kept or introduced a deprecated leading `oya` / `cloud` brand.
    DeprecatedBrandTarget { which: String, value: String },
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
    /// The owned Cargo.lock rename/canonicalize transform rejected the lockfile (e.g. a
    /// non-canonical block boundary) — fail-closed rather than corrupt the lock.
    LockfileTransform { message: String },
    /// More than one committed `specs/reorg/*-move-plan.json` exists in the candidate tree (#65).
    /// A MOVE PR commits exactly one plan; >1 is a contributor error the manifest materialization
    /// must fail-closed on rather than silently first-winning an arbitrary one.
    MultipleMovePlans { count: usize, paths: Vec<String> },
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
            CodemodError::DeprecatedBrandTarget { which, value } => write!(
                f,
                "{which} remains on deprecated oya/cloud brand target {value:?}; de-brand targets must use capability-first names"
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
            CodemodError::LockfileTransform { message } => {
                write!(f, "Cargo.lock transform failed (fail-closed): {message}")
            }
            CodemodError::MultipleMovePlans { count, paths } => write!(
                f,
                "more than one committed move-plan in specs/reorg/ ({count}); a move PR commits \
                 exactly one (fail-closed): {paths:?}"
            ),
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

fn is_deprecated_brand_name(value: &str) -> bool {
    value.starts_with("oya-") || value.starts_with("cloud-")
}

fn is_deprecated_brand_path_source(path: &str) -> bool {
    path.starts_with("oya/") || path.starts_with("cloud/") || is_deprecated_brand_path_leaf(path)
}

fn is_deprecated_brand_path_target(path: &str) -> bool {
    path.starts_with("oya/") || path.starts_with("cloud/")
}

fn is_deprecated_brand_artifact_source(path: &str) -> bool {
    path.starts_with("oya/")
        || path.starts_with("cloud/")
        || is_deprecated_brand_path_leaf(path)
        || is_deprecated_brand_file_stem(path)
}

fn is_deprecated_brand_artifact_target(path: &str) -> bool {
    path.starts_with("oya/") || path.starts_with("cloud/") || is_deprecated_brand_file_stem(path)
}

fn is_deprecated_brand_path_leaf(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(is_deprecated_brand_name)
}

fn is_deprecated_brand_file_stem(path: &str) -> bool {
    path.rsplit('/').next().is_some_and(|leaf| {
        leaf.strip_suffix(".yaml")
            .or_else(|| leaf.strip_suffix(".yml"))
            .or_else(|| leaf.strip_suffix(".json"))
            .is_some_and(is_deprecated_brand_name)
    })
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
            artifacts: vec![],
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
            artifacts: vec![],
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
            artifacts: vec![],
        };
        assert!(matches!(
            plan.validate(),
            Err(CodemodError::NestedTarget { .. })
        ));
    }

    #[test]
    fn validate_rejects_branded_crate_move_targets() {
        let oya_path_plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-iam/crates/oya-cloud-iam-domain".to_string(),
                new_path: "oya/identity/crates/oya-identity-domain".to_string(),
                old_cargo_name: "oya-cloud-iam-domain".to_string(),
                new_cargo_name: "identity-domain".to_string(),
            }],
            artifacts: vec![],
        };
        assert!(matches!(
            oya_path_plan.validate(),
            Err(CodemodError::DeprecatedBrandTarget { which, value })
                if which == "new_path" && value == "oya/identity/crates/oya-identity-domain"
        ));

        let cloud_name_plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-iam/crates/oya-cloud-iam-domain".to_string(),
                new_path: "iam/core/domain".to_string(),
                old_cargo_name: "oya-cloud-iam-domain".to_string(),
                new_cargo_name: "cloud-iam-domain".to_string(),
            }],
            artifacts: vec![],
        };
        assert!(matches!(
            cloud_name_plan.validate(),
            Err(CodemodError::DeprecatedBrandTarget { which, value })
                if which == "new_cargo_name" && value == "cloud-iam-domain"
        ));

        let introduced_brand_path_plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "iam/core/domain".to_string(),
                new_path: "cloud/iam/core/domain".to_string(),
                old_cargo_name: "identity-domain".to_string(),
                new_cargo_name: "identity-domain".to_string(),
            }],
            artifacts: vec![],
        };
        assert!(matches!(
            introduced_brand_path_plan.validate_debrand_targets(),
            Err(CodemodError::DeprecatedBrandTarget { which, value })
                if which == "new_path" && value == "cloud/iam/core/domain"
        ));

        let introduced_brand_name_plan = MovePlan {
            capability: "iam".to_string(),
            moves: vec![CrateMove {
                old_path: "iam/core/domain".to_string(),
                new_path: "iam/core/domain-v2".to_string(),
                old_cargo_name: "identity-domain".to_string(),
                new_cargo_name: "oya-identity-domain".to_string(),
            }],
            artifacts: vec![],
        };
        assert!(matches!(
            introduced_brand_name_plan.validate_debrand_targets(),
            Err(CodemodError::DeprecatedBrandTarget { which, value })
                if which == "new_cargo_name" && value == "oya-identity-domain"
        ));
    }

    #[test]
    fn validate_rejects_branded_artifact_targets() {
        let plan = MovePlan {
            capability: "calendar".to_string(),
            moves: vec![CrateMove {
                old_path: "oya/calendar/crates/oya-calendar-domain".to_string(),
                new_path: "comms/core/calendar-domain".to_string(),
                old_cargo_name: "oya-calendar-domain".to_string(),
                new_cargo_name: "comms-calendar-domain".to_string(),
            }],
            artifacts: vec![ArtifactMove {
                old_path: "registry/catalog/oya-calendar-domain.yaml".to_string(),
                new_path: "registry/catalog/oya-calendar-domain.yaml".to_string(),
            }],
        };
        assert!(matches!(
            plan.validate(),
            Err(CodemodError::DeprecatedBrandTarget { which, value })
                if which == "artifact new_path" && value == "registry/catalog/oya-calendar-domain.yaml"
        ));

        let introduced_brand_artifact_plan = MovePlan {
            capability: "calendar".to_string(),
            moves: vec![CrateMove {
                old_path: "calendar/core/domain".to_string(),
                new_path: "calendar/core/domain-v2".to_string(),
                old_cargo_name: "calendar-domain".to_string(),
                new_cargo_name: "calendar-domain-v2".to_string(),
            }],
            artifacts: vec![ArtifactMove {
                old_path: "registry/catalog/calendar-domain.yaml".to_string(),
                new_path: "registry/catalog/cloud-calendar-domain.yaml".to_string(),
            }],
        };
        assert!(matches!(
            introduced_brand_artifact_plan.validate_debrand_targets(),
            Err(CodemodError::DeprecatedBrandTarget { which, value })
                if which == "artifact new_path" && value == "registry/catalog/cloud-calendar-domain.yaml"
        ));
    }

    #[test]
    fn validate_allows_debranded_targets_that_keep_cloud_as_a_non_prefix_descriptor() {
        let plan = MovePlan {
            capability: "marketplace".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-marketplace/crates/oya-cloud-marketplace-domain".to_string(),
                new_path: "marketplace/core/cloud-domain".to_string(),
                old_cargo_name: "oya-cloud-marketplace-domain".to_string(),
                new_cargo_name: "marketplace-cloud-domain".to_string(),
            }],
            artifacts: vec![ArtifactMove {
                old_path: "registry/catalog/oya-cloud-marketplace-domain.yaml".to_string(),
                new_path: "registry/catalog/marketplace-cloud-domain.yaml".to_string(),
            }],
        };
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn file_level_manifest_derives_from_candidate_new_descendants_mapping_back_to_old() {
        // The candidate POST-move tree carries the NEW paths; old_path is GONE. The derivation
        // must enumerate strict descendants of NEW_path and map them BACK to old_path/<rel>, so
        // the emitted pairs have new PRESENT in the candidate (P3) and old ABSENT (P2). The
        // previous (inert) form enumerated old_path descendants over this same candidate tree
        // and found ZERO => empty manifest => the relabel could never fire.
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/cloud-observability/crates/oya-cloud-observability-domain".to_string(),
                new_path: "observability/core/aggregate".to_string(),
                old_cargo_name: "oya-cloud-observability-domain".to_string(),
                new_cargo_name: "observability-core-aggregate".to_string(),
            }],
            artifacts: vec![],
        };
        // The candidate tracked set is the POST-move tree: NEW paths present, OLD path absent.
        let tracked = vec![
            "observability/core/aggregate/src/lib.rs".to_string(),
            "observability/core/aggregate/Cargo.toml".to_string(),
            // unrelated NEW file (different crate) — must NOT map
            "observability/core/api/src/lib.rs".to_string(),
            // a file equal to the new dir name prefix but not a strict descendant — must NOT map
            "observability/core/aggregate-extra/x.rs".to_string(),
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
            "only strict NEW-dir descendants map, mapped back to old/<rel>, sorted by old"
        );
    }

    #[test]
    fn file_level_manifest_is_empty_when_new_dir_absent_from_candidate() {
        // Feeding the inert OLD-tree (where new_path is absent) yields NO pairs — the candidate
        // post-move tree is the contract, and a tree with only old paths means the move did not
        // land, so the relabel correctly does not fire.
        let plan = MovePlan {
            capability: "x".to_string(),
            moves: vec![CrateMove {
                old_path: "cloud/a".to_string(),
                new_path: "x/core/a".to_string(),
                old_cargo_name: "oya-a".to_string(),
                new_cargo_name: "x-core-a".to_string(),
            }],
            artifacts: vec![],
        };
        assert!(plan.file_level_manifest(&[]).is_empty(), "no tracked => no pairs");
        // Only the OLD path tracked (move did not land) => still empty (new descendants absent).
        assert!(
            plan.file_level_manifest(&["cloud/a/src/lib.rs".to_string()])
                .is_empty(),
            "old-only tree => no NEW descendants => empty manifest (inert old-enum would over-map)"
        );
    }

    #[test]
    fn crate_dir_pairs_emit_one_pair_per_landed_move() {
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![
                CrateMove {
                    old_path: "cloud/cloud-observability/crates/oya-cloud-observability-domain".to_string(),
                    new_path: "observability/core/aggregate".to_string(),
                    old_cargo_name: "oya-cloud-observability-domain".to_string(),
                    new_cargo_name: "observability-core-aggregate".to_string(),
                },
                // a SECOND move whose new dir is NOT in the candidate tree (did not land) — its
                // crate-DIR pair must NOT be emitted (mirrors the file-level candidate guard).
                CrateMove {
                    old_path: "cloud/cloud-observability/crates/oya-cloud-observability-ghost".to_string(),
                    new_path: "observability/core/ghost".to_string(),
                    old_cargo_name: "oya-cloud-observability-ghost".to_string(),
                    new_cargo_name: "observability-core-ghost".to_string(),
                },
            ],
            artifacts: vec![],
        };
        let tracked = vec!["observability/core/aggregate/src/lib.rs".to_string()];
        assert_eq!(
            plan.crate_dir_pairs(&tracked),
            vec![(
                "cloud/cloud-observability/crates/oya-cloud-observability-domain".to_string(),
                "observability/core/aggregate".to_string()
            )],
            "only the landed move emits a crate-DIR pair"
        );
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
            artifacts: vec![],
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
        let empty = move_manifest_value("", &[], &[], &[]);
        assert_eq!(empty["schema"], REORG_MOVE_MANIFEST_SCHEMA);
        assert_eq!(empty["capability"], "");
        assert_eq!(empty["files"], serde_json::json!([]));
        assert_eq!(empty["crate_dirs"], serde_json::json!([]));
        assert_eq!(empty["crate_idents"], serde_json::json!([]));

        let full = move_manifest_value(
            "observability",
            &[("old/a.rs".to_string(), "new/a.rs".to_string())],
            &[("old".to_string(), "new".to_string())],
            &[("oya-old".to_string(), "new-core".to_string())],
        );
        assert_eq!(full["files"][0]["old_path"], "old/a.rs");
        assert_eq!(full["files"][0]["new_path"], "new/a.rs");
        assert_eq!(full["crate_dirs"][0]["old_path"], "old");
        assert_eq!(full["crate_dirs"][0]["new_path"], "new");
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
            artifacts: vec![],
        };
        let mapping = plan.mapping();
        let row = &mapping.rows[0];
        assert_eq!(row.old_path, "cloud/cloud-iam/crates/oya-iam");
        assert_eq!(row.new_path, "iam/core/iam");
        assert_eq!(row.old_cargo_name, "oya-cloud-iam");
        assert_eq!(row.new_cargo_name, "iam-core");
        assert_eq!(row.buck_label, "//iam/core/iam:iam-core");
    }

    // --- ArtifactMove (PR-A): NON-crate co-move (SLOs, catalog records) ---

    fn obs_crate_move() -> CrateMove {
        CrateMove {
            old_path: "oya/observability/crates/oya-observability-domain".to_string(),
            new_path: "observability/core/domain".to_string(),
            old_cargo_name: "oya-observability-domain".to_string(),
            new_cargo_name: "observability-domain".to_string(),
        }
    }

    #[test]
    fn validate_rejects_artifact_old_path_colliding_with_a_crate_move() {
        // An artifact whose OLD path equals a crate move's old_path maps two sources in — reject.
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![obs_crate_move()],
            artifacts: vec![ArtifactMove {
                old_path: "oya/observability/crates/oya-observability-domain".to_string(),
                new_path: "observability/observability/slos".to_string(),
            }],
        };
        assert!(
            matches!(
                plan.validate(),
                Err(CodemodError::DuplicateKey { kind, .. }) if kind == "old_path"
            ),
            "artifact old_path colliding with a crate move must fail-closed"
        );
    }

    #[test]
    fn validate_rejects_artifact_new_path_colliding_with_another_artifact() {
        // Two artifacts mapping to the SAME new_path (the cross-service SLO-name collision the
        // design §PR-B ⚠ calls out) maps two things out — reject (dup-new_path backstop).
        let plan = MovePlan {
            capability: "storage".to_string(),
            moves: vec![CrateMove {
                old_path: "oya/drive/crates/oya-drive-domain".to_string(),
                new_path: "storage/core/drive-domain".to_string(),
                old_cargo_name: "oya-drive-domain".to_string(),
                new_cargo_name: "storage-drive-domain".to_string(),
            }],
            artifacts: vec![
                ArtifactMove {
                    old_path: "oya/drive/slos/autosharding-events.openslo.yaml".to_string(),
                    new_path: "storage/observability/slos/autosharding-events.openslo.yaml"
                        .to_string(),
                },
                ArtifactMove {
                    old_path: "oya/imaging/slos/autosharding-events.openslo.yaml".to_string(),
                    new_path: "storage/observability/slos/autosharding-events.openslo.yaml"
                        .to_string(),
                },
            ],
        };
        assert!(
            matches!(
                plan.validate(),
                Err(CodemodError::DuplicateKey { kind, .. }) if kind == "new_path"
            ),
            "two artifacts mapping to the same new_path must fail-closed"
        );
    }

    #[test]
    fn validate_rejects_non_normalized_artifact_path() {
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![obs_crate_move()],
            artifacts: vec![ArtifactMove {
                old_path: "oya/observability/slos".to_string(),
                new_path: "observability/observability/slos/".to_string(), // trailing slash
            }],
        };
        assert!(
            matches!(
                plan.validate(),
                Err(CodemodError::BadPath { which, .. }) if which == "artifact new_path"
            ),
            "a non-normalized artifact path must fail-closed"
        );
    }

    #[test]
    fn validate_accepts_a_plan_with_well_formed_artifacts() {
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![obs_crate_move()],
            artifacts: vec![ArtifactMove {
                old_path: "oya/observability/slos".to_string(),
                new_path: "observability/observability/slos".to_string(),
            }],
        };
        assert!(plan.validate().is_ok(), "a well-formed artifact plan validates");
    }

    #[test]
    fn validate_accepts_an_artifact_only_plan_with_no_crate_moves() {
        // PR-B backfill shape: zero crate moves, artifact co-moves only (relocate orphaned SLOs
        // + re-key catalog records for an ALREADY-moved capability). This MUST validate — the
        // EmptyPlan guard fires only when BOTH sides are empty, not when `moves` alone is.
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![],
            artifacts: vec![ArtifactMove {
                old_path: "oya/observability/slos".to_string(),
                new_path: "observability/observability/slos".to_string(),
            }],
        };
        assert!(
            plan.validate().is_ok(),
            "an artifact-only plan (moves: []) is the PR-B backfill shape and must validate"
        );
    }

    #[test]
    fn validate_rejects_a_plan_empty_on_both_sides() {
        // The EmptyPlan no-op: neither crate moves NOR artifacts — nothing to do, fail-closed.
        let plan = MovePlan {
            capability: "x".to_string(),
            moves: vec![],
            artifacts: vec![],
        };
        assert!(matches!(plan.validate(), Err(CodemodError::EmptyPlan)));
    }

    #[test]
    fn inverse_round_trips_artifacts() {
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![obs_crate_move()],
            artifacts: vec![
                ArtifactMove {
                    old_path: "oya/observability/slos".to_string(),
                    new_path: "observability/observability/slos".to_string(),
                },
                ArtifactMove {
                    old_path: "registry/catalog/oya-observability-domain.yaml".to_string(),
                    new_path: "registry/catalog/observability-domain.yaml".to_string(),
                },
            ],
        };
        let inv = plan.inverse();
        // every artifact side is swapped...
        assert_eq!(inv.artifacts[0].old_path, "observability/observability/slos");
        assert_eq!(inv.artifacts[0].new_path, "oya/observability/slos");
        assert_eq!(
            inv.artifacts[1].old_path,
            "registry/catalog/observability-domain.yaml"
        );
        assert_eq!(
            inv.artifacts[1].new_path,
            "registry/catalog/oya-observability-domain.yaml"
        );
        // ...and inverse-of-inverse restores both moves AND artifacts identically.
        let back = inv.inverse();
        assert_eq!(back.artifacts, plan.artifacts);
        assert_eq!(back.moves, plan.moves);
    }

    #[test]
    fn artifact_file_pairs_dir_enumerates_descendants_and_file_emits_direct_pair() {
        // A DIR artifact (SLO dir) enumerates strict NEW-dir descendants -> old/<rel>; a FILE
        // artifact (catalog record) whose new_path is itself a tracked file emits the direct pair.
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![obs_crate_move()],
            artifacts: vec![
                // DIR artifact
                ArtifactMove {
                    old_path: "oya/observability/slos".to_string(),
                    new_path: "observability/observability/slos".to_string(),
                },
                // FILE artifact (a single catalog record re-keyed/re-homed)
                ArtifactMove {
                    old_path: "registry/catalog/oya-observability-domain.yaml".to_string(),
                    new_path: "registry/catalog/observability-domain.yaml".to_string(),
                },
            ],
        };
        // Candidate POST-move tree: NEW slo descendants + the NEW catalog file are present.
        let tracked = vec![
            "observability/observability/slos/api-availability.openslo.yaml".to_string(),
            "observability/observability/slos/api-latency.openslo.yaml".to_string(),
            "registry/catalog/observability-domain.yaml".to_string(),
            // a NEW crate file (handled by file_level_manifest, NOT artifact_file_pairs) — absent here
            "observability/observability/slos-extra/x.yaml".to_string(), // sibling, not a descendant
        ];
        let pairs = plan.artifact_file_pairs(&tracked);
        assert_eq!(
            pairs,
            vec![
                // dir descendants mapped back to old/<rel>, sorted by old
                (
                    "oya/observability/slos/api-availability.openslo.yaml".to_string(),
                    "observability/observability/slos/api-availability.openslo.yaml".to_string(),
                ),
                (
                    "oya/observability/slos/api-latency.openslo.yaml".to_string(),
                    "observability/observability/slos/api-latency.openslo.yaml".to_string(),
                ),
                // single FILE artifact: direct (old, new) pair
                (
                    "registry/catalog/oya-observability-domain.yaml".to_string(),
                    "registry/catalog/observability-domain.yaml".to_string(),
                ),
            ],
            "dir enumerates descendants (old/<rel>); file emits the direct pair; sibling excluded"
        );
    }

    #[test]
    fn artifact_file_pairs_is_empty_when_no_artifacts() {
        // BACK-COMPAT no-op: a plan with NO artifacts emits ZERO artifact pairs regardless of the
        // candidate tree — so the manifest `files` list is byte-identical to a pre-ArtifactMove
        // plan (the existing committed plans are provably unaffected).
        let plan = MovePlan {
            capability: "observability".to_string(),
            moves: vec![obs_crate_move()],
            artifacts: vec![],
        };
        let tracked = vec![
            "observability/core/domain/src/lib.rs".to_string(),
            "observability/observability/slos/api.openslo.yaml".to_string(),
        ];
        assert!(
            plan.artifact_file_pairs(&tracked).is_empty(),
            "no artifacts => no artifact pairs (back-compat no-op)"
        );
    }
}
