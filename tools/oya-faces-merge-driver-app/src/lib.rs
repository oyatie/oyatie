//! # oya-faces-merge-driver-app (task #125 v1, ADR-0572)
//!
//! A fail-closed local Git merge driver + post-merge settle for the born-accounting generated
//! faces (`scm-facts.generated.json`, `accounting-registry.generated.json`, and the other
//! producer faces under `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/`).
//!
//! ## Why the faces cannot be three-way-merged (the spine of this design)
//! The faces embed git-HISTORY facts, not just working-tree content: every accounting-registry
//! row carries a `last_touch_commit` SHA and TTL/aging derived from commit timestamps (emitted by
//! the ADR-0515-D3 `oya-cloud-ci-scm-facts-emitter-app` git boundary). During an in-progress local
//! merge/rebase the merge commit SHA does not yet exist, so a mid-merge `%O %A %B` driver CANNOT
//! authoritatively regenerate them. Therefore:
//!
//! 1. The per-file merge driver ([`run_merge_driver`]) is COSMETIC: it takes *theirs* (`%B`) into
//!    `%A` so git records the path as resolved and does not emit conflict markers. This value is
//!    discarded — it is never authoritative. It fails closed (non-zero, `%A` untouched) on any IO
//!    error or if the path is not a control-plane-declared regeneratable face.
//! 2. The AUTHORITATIVE regeneration is the post-merge / post-rewrite SETTLE
//!    ([`run_post_merge_settle`]): after the merge/rebase COMMIT exists, it regenerates ALL faces
//!    from the committed merged tree (the scm-facts emitter now sees the real commit graph),
//!    byte-rediff-verifies them against a fresh producer re-render (drift = fail closed), runs a
//!    determinism self-check, and writes the settled faces. Reuses the doctrine-blessed
//!    [`oya_cloud_ci_freshness_app::settle_regenerated_faces`] engine so the output satisfies the
//!    freshness gate (`GENERATED_FACE_PATHS` byte-parity) and registry-drift byte-for-byte.
//!
//! ## Fail-closed contract (HARD requirement — a wrong face is a false-green vector)
//! On ANY regen failure, non-determinism, missing producer, drift mismatch, or IO error the
//! driver/settle exits NON-ZERO and leaves the conflict in place. It NEVER writes a guessed or
//! partial faces file. The settle writes faces only after every face has byte-matched a fresh
//! re-render. The per-file driver writes `%A` atomically (write-temp + rename), so a crash leaves
//! `%A` byte-untouched (mirrors `oya-friction-ledger-merge-driver-app`).
//!
//! ## Universality (policy-as-data — nothing oyatie-specific hardcoded)
//! The set of regeneratable faces and the producer/emitter targets come from
//! `registry/generated-artifact-control-plane.json` (the declared public product contract), read
//! via [`ControlPlane`]. The driver hardcodes no face path: a repo adopting oya-ci ships its own
//! manifest and gets the same behavior. The control-plane schema is CLOSED (the
//! `cloud-ci-generated-artifact-control-plane` gate rejects unknown fields), so this crate reads
//! ONLY existing manifest fields — it does NOT add `merge_driver`/`shard_key` policy fields.
//!
//! ## Irreducible-glue ledger (ADR-0515 D3 / ADR-0523 item 2)
//! The settle's [`Buck2RegenAdapter`] subprocesses the BUILT face-generation binaries
//! (codemod -> scm-facts emitter -> producer) exactly as the canonical
//! `infra/ci/materialize-cloud-ci-generated-faces.sh` and the freshness gate's
//! `regenerate_faces_with_buck2` / register-crate's `Buck2RegenAdapter` do. The scm-facts emitter
//! is the single sanctioned git boundary; the buck2 build + emitter spawn are the irreducible glue
//! at the graph edge — no other shell.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;

use oya_cloud_ci_freshness_app::{
    FaceSettleMode, assert_non_face_tree_clean, settle_regenerated_faces,
};
use serde_json::Value;

/// The repo-relative control-plane manifest — the universality surface (policy-as-data). The
/// declared regeneratable face paths + the producer/emitter generator targets are read from here.
pub const CONTROL_PLANE_PATH: &str = "registry/generated-artifact-control-plane.json";

/// The merge attribute name the `.gitattributes` glob lines + the git-config driver use.
pub const MERGE_ATTRIBUTE: &str = "oya-faces";

/// The dir the generated faces live in (the producer's `--out-dir`, the freshness gate's
/// `FACES_DIR`, the emitter's default `--out` parent). Used to write the scm-facts snapshot.
const FACES_DIR: &str = "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app";

/// The committed scm-facts snapshot face name (the emitter's `--out`, the producer's `--scm-facts`).
const SCM_FACTS_FACE: &str = "scm-facts.generated.json";

/// The repo-relative move-manifest the codemod emits and the emitter consumes (materialize.sh
/// step 1 -> step 2). A pure function of (committed plan + candidate tree).
const MOVE_MANIFEST_PATH: &str = "specs/reorg/move-manifest.generated.json";

/// The reorg move-plan glob dir a MOVE PR commits; the codemod's `--plan` input. A no-move run
/// passes no `--plan` (canonical empty manifest).
const MOVE_PLAN_DIR: &str = "specs/reorg";

/// The buck2 targets the [`Buck2RegenAdapter`] builds (mirroring materialize.sh + the freshness
/// gate's `build_face_tools` + register-crate's `Buck2RegenAdapter`).
/// The scm-facts emitter buck2 target — one of the two settle-capable generator targets (the faces
/// it writes are settle-capable). Public so consumers/tests can reference the canonical target.
pub const EMITTER_TARGET: &str =
    "//cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app";
/// The accounting-registry producer buck2 target — the other settle-capable generator target.
pub const PRODUCER_TARGET: &str =
    "//cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin";
const CODEMOD_TARGET: &str = "//tools/oya-reorg-codemod-app:oya-reorg-codemod";

/// The 6 deterministic producer faces (`(file_name, --face name)`), regenerated + byte-rediffed.
/// The scm-facts snapshot (the emitter's output) is the 7th settled face but is not per-face
/// re-rendered (the emitter writes it directly).
const PRODUCER_FACES: [(&str, &str); 6] = [
    ("accounting-registry.generated.json", "registry"),
    ("ttl-policy.generated.json", "ttl-policy"),
    ("decision-crosswalk.generated.json", "decision-crosswalk"),
    ("enforcement-inventory.generated.json", "enforcement-inventory"),
    ("enforcement-liveness.generated.json", "enforcement-liveness"),
    ("gate-baseline.generated.json", "baseline"),
];

/// The two control-plane merge policies that mark an artifact as controller-regenerated (never a
/// contributor merge surface) — identical to the `cloud-ci-generated-artifact-control-plane` gate's
/// `diff_policy_allowed_generated_edit_paths` predicate, so this driver's face set never drifts from
/// what that gate already considers a regeneratable generated artifact.
const REGENERATABLE_MERGE_POLICIES: [&str; 2] = [
    "never-manual-merge-regenerate-from-source-tree",
    "controller-owned-main-materialization",
];

/// What went wrong, mapped to the merge-driver exit-code contract (mirrors the cargo-lock /
/// friction-ledger precedents). `Conflict` = exit 1 (decline this merge, leave the conflict);
/// everything else = exit 2 (unparseable/IO/usage/regen/drift/determinism failure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacesMergeErrorKind {
    /// The driver declines the merge (e.g. the path is not a declared regeneratable face) — exit 1.
    Conflict,
    /// The control-plane manifest is missing/malformed/has no regeneratable faces — exit 2.
    ControlPlane,
    /// A regen step (buck2 build, codemod, emitter, producer) failed — exit 2.
    Regen,
    /// A regenerated face did not byte-match a fresh re-render, or the determinism self-check
    /// tripped — exit 2. NEVER write a face under this condition.
    Drift,
    /// The non-face tree was not clean before settle, or the settle engine refused — exit 2.
    Settle,
    /// A filesystem read/write failed — exit 2.
    Io,
    /// Wrong argument count / bad usage — exit 2.
    Usage,
}

/// A typed, fail-closed driver/settle error. Display carries a diagnosable message (fail LOUD).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacesMergeError {
    kind: FacesMergeErrorKind,
    message: String,
}

impl FacesMergeError {
    /// Build an error of `kind` with a diagnostic `message`.
    pub fn new(kind: FacesMergeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The error kind (drives the process exit code).
    pub fn kind(&self) -> FacesMergeErrorKind {
        self.kind
    }
}

impl Display for FacesMergeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for FacesMergeError {}

/// The control-plane facts the driver needs, read from the declared manifest (policy-as-data). The
/// regeneratable face paths + the buck2 producer/emitter generator targets are all DATA — nothing
/// is hardcoded, so a repo adopting oya-ci with its own manifest gets the same behavior.
///
/// CRITICAL fail-closed scoping: the manifest's regeneratable set is BROADER than what this local
/// settle can authoritatively produce — it also declares controller-materialized planning
/// projections (`oya-ci-native-controller` runner) and the codemod move-manifest, none of which the
/// freshness settle engine regenerates. The driver must NEVER cosmetically resolve a face the settle
/// cannot then re-derive (that would leave a wrong value with no fail-closed catch). So
/// [`ControlPlane::settle_capable_face_paths`] intersects the declared regeneratable set with the
/// freshness gate's authoritative `generated_face_paths()`, and [`ControlPlane::is_regeneratable_face`]
/// (the driver's decline predicate) + the `.gitattributes` glob are scoped to THAT intersection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlane {
    /// The repo-relative paths of every declared regeneratable generated artifact (sorted/deduped).
    /// This is the full control-plane set (may include controller-materialized projections the local
    /// settle cannot produce); use [`ControlPlane::settle_capable_face_paths`] for the driver scope.
    pub regeneratable_face_paths: BTreeSet<String>,
    /// The set of buck2 `generator_target`s the regeneratable faces declare (sorted/deduped). These
    /// must be canonical `//...` labels (the control-plane gate enforces this for buck2 runners).
    pub generator_targets: BTreeSet<String>,
    /// The subset of `regeneratable_face_paths` whose generator target is the accounting producer or
    /// the scm-facts emitter — the faces the local settle can authoritatively re-derive. The driver's
    /// decline predicate + the `.gitattributes` glob are scoped to THIS set (fail-closed).
    settle_capable_face_paths: BTreeSet<String>,
}

impl ControlPlane {
    /// Read + parse the control-plane manifest at `repo_root`. Fail-closed: a missing/malformed
    /// manifest, or one declaring zero regeneratable faces, is a hard error (never silently treat
    /// the merge as resolvable). The face set / targets are the SAME data the
    /// `cloud-ci-generated-artifact-control-plane` gate validates.
    ///
    /// # Errors
    /// [`FacesMergeError`] with kind [`FacesMergeErrorKind::ControlPlane`] on read/parse failure or
    /// an empty regeneratable face set.
    pub fn load(repo_root: &Path) -> Result<Self, FacesMergeError> {
        let path = repo_root.join(CONTROL_PLANE_PATH);
        let text = std::fs::read_to_string(&path).map_err(|e| {
            FacesMergeError::new(
                FacesMergeErrorKind::ControlPlane,
                format!("read control-plane {}: {e}", path.display()),
            )
        })?;
        let manifest: Value = serde_json::from_str(&text).map_err(|e| {
            FacesMergeError::new(
                FacesMergeErrorKind::ControlPlane,
                format!("parse control-plane {}: {e}", path.display()),
            )
        })?;
        Self::from_manifest(&manifest)
    }

    /// Extract the regeneratable face paths + generator targets from a parsed manifest. Pure (no
    /// IO) so it is unit-testable and so a synthetic manifest (universality test) exercises the
    /// same code path as the real one.
    ///
    /// # Errors
    /// [`FacesMergeError`] with kind [`FacesMergeErrorKind::ControlPlane`] when `artifacts` is
    /// missing/empty or no artifact carries a regeneratable merge policy.
    pub fn from_manifest(manifest: &Value) -> Result<Self, FacesMergeError> {
        let artifacts = manifest
            .get("artifacts")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                FacesMergeError::new(
                    FacesMergeErrorKind::ControlPlane,
                    "control-plane manifest has no `artifacts` array",
                )
            })?;

        let mut regeneratable_face_paths = BTreeSet::new();
        let mut generator_targets = BTreeSet::new();
        let mut settle_capable_face_paths = BTreeSet::new();
        for artifact in artifacts {
            let merge_policy = artifact.get("merge_policy").and_then(Value::as_str);
            let Some(merge_policy) = merge_policy else {
                continue;
            };
            if !REGENERATABLE_MERGE_POLICIES.contains(&merge_policy) {
                continue;
            }
            let Some(path) = artifact.get("path").and_then(Value::as_str) else {
                continue;
            };
            regeneratable_face_paths.insert(path.to_owned());
            // The buck2 generator target — DATA, not hardcoded. Controller-runner artifacts
            // (oya-ci-native-controller) carry a non-buck2 target the local driver cannot build;
            // collect only canonical buck2 `//...` labels (the ones the settle can materialize).
            let target = artifact
                .get("generator")
                .and_then(|g| g.get("generator_target"))
                .and_then(Value::as_str);
            if let Some(target) = target
                && target.starts_with("//")
            {
                generator_targets.insert(target.to_owned());
                // SETTLE-CAPABLE scoping (fail-closed): a face is settle-capable iff its generator
                // target is one the settle's Buck2RegenAdapter actually WRITES — the accounting
                // producer or the scm-facts emitter. The codemod move-manifest (a settle INPUT, not a
                // staged face) and the controller-materialized planning projections are declared
                // regeneratable but the local settle cannot produce them, so they are EXCLUDED — the
                // driver must never cosmetically resolve a face the settle cannot then re-derive.
                if target == PRODUCER_TARGET || target == EMITTER_TARGET {
                    settle_capable_face_paths.insert(path.to_owned());
                }
            }
        }

        if regeneratable_face_paths.is_empty() {
            return Err(FacesMergeError::new(
                FacesMergeErrorKind::ControlPlane,
                "control-plane manifest declares no regeneratable generated faces",
            ));
        }

        Ok(Self {
            regeneratable_face_paths,
            generator_targets,
            settle_capable_face_paths,
        })
    }

    /// The faces this LOCAL settle authoritatively regenerates: the declared regeneratable faces
    /// whose generator target is the accounting producer or the scm-facts emitter (the targets the
    /// settle's `Buck2RegenAdapter` actually writes). Policy-as-data (the targets come from the
    /// manifest) AND fail-closed (controller-materialized projections + the codemod move-manifest are
    /// excluded — the driver must never cosmetically resolve a face the settle cannot re-derive).
    /// Sorted/deduped.
    #[must_use]
    pub fn settle_capable_face_paths(&self) -> BTreeSet<String> {
        self.settle_capable_face_paths.clone()
    }

    /// True iff `path` (repo-relative, forward-slash) is a face this local settle authoritatively
    /// regenerates (a settle-capable face). The per-file driver declines (exit 1) any path that is
    /// not — it must never cosmetically resolve a surface the settle cannot then re-derive.
    #[must_use]
    pub fn is_regeneratable_face(&self, path: &str) -> bool {
        self.settle_capable_face_paths().contains(path)
    }
}

/// Run the per-file merge driver (`oya-faces-merge-driver driver %O %A %B %P`).
///
/// COSMETIC by design (see the crate docs): copy *theirs* (`%B`) over `%A` ATOMICALLY so git
/// records the face as resolved without conflict markers, then exit 0. The written value is NEVER
/// authoritative — the post-merge settle overwrites it from the committed merged tree. The
/// `repo_root` + `pathname` are used to fail closed: if `pathname` is not a control-plane-declared
/// regeneratable face the driver DECLINES (kind [`FacesMergeErrorKind::Conflict`], exit 1) and
/// leaves `%A` untouched, so git falls back to a normal conflict.
///
/// # Errors
/// - [`FacesMergeErrorKind::Conflict`] (exit 1) if `pathname` is not a declared regeneratable face.
/// - [`FacesMergeErrorKind::ControlPlane`] / [`FacesMergeErrorKind::Io`] (exit 2) on a manifest or
///   IO failure. On ANY error `%A` is left byte-untouched.
pub fn run_merge_driver(
    repo_root: &Path,
    _ancestor: &Path,
    ours: &Path,
    theirs: &Path,
    pathname: &str,
) -> Result<(), FacesMergeError> {
    let control_plane = ControlPlane::load(repo_root)?;
    let normalized = pathname.strip_prefix("./").unwrap_or(pathname);
    if !control_plane.is_regeneratable_face(normalized) {
        // Not a declared regeneratable face — never resolve it cosmetically; decline so git keeps
        // the conflict and a human/queue sees it. (`%A` stays byte-untouched: nothing is written.)
        return Err(FacesMergeError::new(
            FacesMergeErrorKind::Conflict,
            format!(
                "{normalized} is not a control-plane-declared regeneratable face; declining the \
                 cosmetic merge so git keeps the conflict (the post-merge settle is authoritative \
                 only for declared faces)"
            ),
        ));
    }

    // Cosmetic resolve: theirs -> ours, atomically (write temp + rename), so a crash leaves `%A`
    // byte-untouched (the friction-ledger precedent). The settle re-derives the real bytes.
    let theirs_bytes = std::fs::read(theirs).map_err(|e| {
        FacesMergeError::new(
            FacesMergeErrorKind::Io,
            format!("read theirs {}: {e}", theirs.display()),
        )
    })?;
    write_atomic(ours, &theirs_bytes)
}

/// Atomically replace `target` with `contents`: write a sibling temp file then rename over the
/// target. A crash between write and rename leaves `target` byte-untouched (the friction-ledger
/// driver's incident-2 guarantee). The temp lives beside the target so the rename is same-filesystem.
fn write_atomic(target: &Path, contents: &[u8]) -> Result<(), FacesMergeError> {
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("faces-merge-result");
    let temp = target.with_file_name(format!(".{file_name}.oya-faces-merge-tmp-{}", std::process::id()));
    std::fs::write(&temp, contents).map_err(|e| {
        FacesMergeError::new(
            FacesMergeErrorKind::Io,
            format!("write merge temp {}: {e}", temp.display()),
        )
    })?;
    std::fs::rename(&temp, target).map_err(|e| {
        let _ = std::fs::remove_file(&temp);
        FacesMergeError::new(
            FacesMergeErrorKind::Io,
            format!("move merged face into {}: {e}", target.display()),
        )
    })
}

/// The port the settle uses to regenerate the committed faces from the candidate tree. Injectable
/// so unit tests run with NO buck2 ([`FakeRegenPort`] in tests); the one production implementation
/// is [`Buck2RegenAdapter`]. Mirrors the register-crate `RegenPort` precedent.
pub trait RegenPort {
    /// Regenerate the committed generated faces from the merged tree at `repo_root` (scm-facts
    /// snapshot + the producer faces), writing them under [`FACES_DIR`]. Returns the regenerated
    /// faces as `(file_name, bytes)` (the shape `settle_regenerated_faces` consumes), sorted by name.
    ///
    /// # Errors
    /// [`FacesMergeError`] with kind [`FacesMergeErrorKind::Regen`] on any step failure.
    fn regenerate(&self, repo_root: &Path) -> Result<Vec<(String, String)>, FacesMergeError>;

    /// The byte-rediff drift + determinism self-check: regenerate a SECOND time and confirm every
    /// producer face byte-matches the first regeneration. A mismatch is non-determinism / drift and
    /// fails closed ([`FacesMergeErrorKind::Drift`]) — never write a face under this condition.
    ///
    /// # Errors
    /// [`FacesMergeError`] with kind [`FacesMergeErrorKind::Drift`] on a byte mismatch, or
    /// [`FacesMergeErrorKind::Regen`] if the second regeneration cannot run.
    fn verify_determinism(
        &self,
        repo_root: &Path,
        first: &[(String, String)],
    ) -> Result<(), FacesMergeError>;
}

/// Run the authoritative post-merge / post-rewrite settle (`oya-faces-merge-driver settle`).
///
/// This is the REAL work (the per-file driver is cosmetic). After the merge/rebase COMMIT exists,
/// regenerate ALL faces from the committed merged tree via `regen`, run the determinism self-check,
/// then settle them through the freshness engine. Fail-closed at every step.
///
/// Order (each step fails closed before the next):
/// 1. [`assert_non_face_tree_clean`] — the non-face tree must be committed (post-merge-commit
///    invariant; the per-file `%A` write was cosmetic and the real bytes come from the committed
///    tree). A dirty non-face tree refuses the settle.
/// 2. `regen.regenerate` — regenerate scm-facts + producer faces from the merged tree.
/// 3. `regen.verify_determinism` — regenerate again + byte-compare (non-determinism = fail closed).
/// 4. [`settle_regenerated_faces`] (`FaceSettleMode::Settle`) — write + stage the faces-only diff
///    (or report already-settled). The freshness engine re-asserts the non-face tree is clean and
///    refuses to stage any non-face path.
///
/// On success returns the sorted list of face file names that were regenerated. On ANY failure
/// returns a [`FacesMergeError`] and NO partial/guessed face is committed (regenerate writes the
/// real faces only after a clean-tree check; the determinism check runs before staging).
///
/// # Errors
/// [`FacesMergeError`] on a dirty tree, a regen failure, a determinism/drift mismatch, or a settle
/// refusal. The exit code maps via [`FacesMergeError::kind`].
pub fn run_post_merge_settle(
    repo_root: &Path,
    regen: &dyn RegenPort,
) -> Result<Vec<String>, FacesMergeError> {
    // Load the control-plane FIRST: a repo with no declared regeneratable faces has nothing to
    // settle, and the manifest must be valid before we touch the tree (fail closed early).
    let _control_plane = ControlPlane::load(repo_root)?;

    // 1. The non-face tree must be committed (post-merge-commit invariant). A dirty non-face tree
    //    means the faces would be regenerated against an uncommitted tree -> refuse.
    assert_non_face_tree_clean(repo_root).map_err(|e| {
        FacesMergeError::new(
            FacesMergeErrorKind::Settle,
            format!(
                "post-merge settle requires a committed non-face tree (the merge/rebase commit \
                 must exist before faces regenerate authoritatively): {e}"
            ),
        )
    })?;

    // 2. Regenerate from the committed merged tree.
    let regenerated = regen.regenerate(repo_root)?;

    // 3. Determinism + drift self-check BEFORE staging — non-determinism never reaches the tree.
    regen.verify_determinism(repo_root, &regenerated)?;

    // 4. Settle through the doctrine-blessed freshness engine (writes + stages faces-only). This
    //    re-asserts the non-face tree is clean and refuses any non-face staged path, so the output
    //    satisfies the freshness gate + registry-drift byte-for-byte.
    let report = settle_regenerated_faces(repo_root, regenerated.clone(), FaceSettleMode::Settle)
        .map_err(|e| {
            FacesMergeError::new(
                FacesMergeErrorKind::Settle,
                format!("settle regenerated faces: {e}"),
            )
        })?;

    if !report.is_success() {
        return Err(FacesMergeError::new(
            FacesMergeErrorKind::Settle,
            format!("settle reported stale faces (fail closed): {}", report.message),
        ));
    }

    let mut faces: Vec<String> = regenerated.into_iter().map(|(name, _)| name).collect();
    faces.sort();
    Ok(faces)
}

/// The production [`RegenPort`]: the NATIVE-in-Rust regeneration that subprocesses the BUILT
/// face-generation binaries (codemod -> scm-facts emitter -> producer), mirroring the
/// doctrine-blessed precedents in `oya-cloud-ci-freshness-app::regenerate_faces_with_buck2` and
/// register-crate's `Buck2RegenAdapter`. It NEVER shells to `materialize-…sh`; the buck2 build +
/// emitter spawn are the irreducible glue at the graph edge (ADR-0515 D3 git boundary).
///
/// Unlike the read-only freshness gate (which routes the emitter to a TEMP scm-facts path), this
/// adapter writes the REAL `<FACES_DIR>/scm-facts.generated.json` because settling MUTATES on
/// purpose — the settle engine then stages the faces-only diff.
//
// IRREDUCIBLE-GLUE LEDGER (ADR-0515 D3 git boundary / ADR-0523 item 2): the scm-facts emitter is
// the single sanctioned `git` boundary; the buck2 build + emitter spawn are the only subprocesses.
#[derive(Debug, Clone, Copy, Default)]
pub struct Buck2RegenAdapter;

impl Buck2RegenAdapter {
    /// Build the tools and run codemod(manifest) -> emitter -> producer, returning the regenerated
    /// faces as `(file_name, bytes)`. Shared by `regenerate` and the determinism re-run.
    fn materialize(&self, repo_root: &Path) -> Result<Vec<(String, String)>, FacesMergeError> {
        let tools = build_face_tools(repo_root)?;

        // 1. codemod(manifest): regenerate the committed move-manifest from the committed plan (if
        //    any) + the candidate tree, so the emitter consumes a FRESH copy. ORDER is load-bearing
        //    (materialize.sh step 1). A no-move run passes NO --plan (canonical empty manifest).
        let manifest_out = repo_root.join(MOVE_MANIFEST_PATH);
        let mut codemod = Command::new(&tools.codemod);
        codemod.arg("manifest").args(["--repo-root"]).arg(repo_root);
        if let Some(plan) = first_move_plan(repo_root)? {
            codemod.args(["--plan"]).arg(plan);
        }
        codemod.args(["--out"]).arg(&manifest_out).current_dir(repo_root);
        run_regen_status(&mut codemod, "codemod manifest")?;

        // 2. emitter: write the REAL scm-facts snapshot (+ the frozen gate-baseline the firewall
        //    differences against). --merge-base-baseline mirrors materialize.sh.
        let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
        run_regen_status(
            Command::new(&tools.emitter)
                .args(["--repo-root"])
                .arg(repo_root)
                .args(["--out"])
                .arg(&scm_facts)
                .arg("--merge-base-baseline")
                .current_dir(repo_root),
            "scm-facts emitter",
        )?;

        // 3. producer: regenerate the 6 producer faces from the just-written scm-facts.
        run_regen_status(
            Command::new(&tools.producer)
                .args(["--repo-root"])
                .arg(repo_root)
                .args(["--scm-facts"])
                .arg(&scm_facts)
                .current_dir(repo_root),
            "accounting-registry producer",
        )?;

        // Read the just-written faces back as (file_name, bytes) for the settle engine + the
        // determinism compare. The scm-facts snapshot is the emitter's output; the 6 producer
        // faces are the producer's.
        let mut faces: Vec<(String, String)> = Vec::with_capacity(PRODUCER_FACES.len() + 1);
        faces.push((SCM_FACTS_FACE.to_owned(), read_face(&scm_facts)?));
        for (file_name, _face_name) in PRODUCER_FACES {
            let path = repo_root.join(FACES_DIR).join(file_name);
            faces.push((file_name.to_owned(), read_face(&path)?));
        }
        faces.sort_by(|left, right| left.0.cmp(&right.0));
        Ok(faces)
    }
}

impl RegenPort for Buck2RegenAdapter {
    fn regenerate(&self, repo_root: &Path) -> Result<Vec<(String, String)>, FacesMergeError> {
        self.materialize(repo_root)
    }

    fn verify_determinism(
        &self,
        repo_root: &Path,
        first: &[(String, String)],
    ) -> Result<(), FacesMergeError> {
        // Re-render every producer face via `--stdout --face <name>` from the just-written scm-facts
        // and byte-compare to the first regeneration. `write_face` and `--stdout` share the same
        // `to_canonical_json`, so a match is exact. A mismatch is non-determinism / drift -> fail
        // closed (never write a face under this condition). The scm-facts snapshot is the emitter's
        // deterministic output (head_time_secs = max last-touch ts, no wall-clock), so it is covered
        // by the producer-face rediff (the faces are a pure function of it).
        let tools = build_face_tools(repo_root)?;
        let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
        let first_by_name: std::collections::BTreeMap<&str, &str> = first
            .iter()
            .map(|(name, bytes)| (name.as_str(), bytes.as_str()))
            .collect();
        for (file_name, face_name) in PRODUCER_FACES {
            let rerender = run_regen_output(
                Command::new(&tools.producer)
                    .args(["--repo-root"])
                    .arg(repo_root)
                    .args(["--scm-facts"])
                    .arg(&scm_facts)
                    .args(["--stdout", "--face", face_name])
                    .current_dir(repo_root),
                &format!("determinism re-render {file_name}"),
            )?;
            match first_by_name.get(file_name) {
                Some(first_bytes) if *first_bytes == rerender => {}
                Some(_) => {
                    return Err(FacesMergeError::new(
                        FacesMergeErrorKind::Drift,
                        format!(
                            "non-deterministic / drifting face {file_name}: a fresh re-render did \
                             not byte-match the first regeneration — refusing to settle (fail \
                             closed; never emit a guessed face)"
                        ),
                    ));
                }
                None => {
                    return Err(FacesMergeError::new(
                        FacesMergeErrorKind::Drift,
                        format!(
                            "face {file_name} was re-rendered but absent from the first \
                             regeneration set — refusing to settle (fail closed)"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// The built face-generation binaries the [`Buck2RegenAdapter`] subprocesses.
struct FaceTools {
    codemod: PathBuf,
    emitter: PathBuf,
    producer: PathBuf,
}

/// `buck2 build` the emitter, producer `-bin`, and codemod in ONE invocation with `--show-output`,
/// then parse each output binary path by target-name match — mirroring the freshness gate's
/// `build_face_tools` + register-crate's `build_face_tools` + materialize.sh's single build call.
fn build_face_tools(repo_root: &Path) -> Result<FaceTools, FacesMergeError> {
    let output = run_regen_output(
        Command::new("buck2")
            .arg("build")
            .arg(EMITTER_TARGET)
            .arg(PRODUCER_TARGET)
            .arg(CODEMOD_TARGET)
            .arg("--show-output")
            .current_dir(repo_root),
        "buck2 build face tools",
    )?;
    let emitter = parse_show_output_path(repo_root, &output, EMITTER_TARGET)?;
    let producer = parse_show_output_path(repo_root, &output, PRODUCER_TARGET)?;
    let codemod = parse_show_output_path(repo_root, &output, CODEMOD_TARGET)?;
    Ok(FaceTools {
        codemod,
        emitter,
        producer,
    })
}

fn parse_show_output_path(
    repo_root: &Path,
    output: &str,
    target: &str,
) -> Result<PathBuf, FacesMergeError> {
    for line in output.lines() {
        let mut parts = line.split_whitespace();
        let seen_target = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();
        if seen_target.contains(target) && !path.is_empty() {
            let path = PathBuf::from(path);
            return Ok(if path.is_absolute() {
                path
            } else {
                repo_root.join(path)
            });
        }
    }
    Err(FacesMergeError::new(
        FacesMergeErrorKind::Regen,
        format!("buck2 --show-output did not include {target}"),
    ))
}

/// The first reorg move-plan under `specs/reorg/*-move-plan.json` (sorted), or `None` for a no-move
/// run. Exactly one plan is expected per move PR (mirrors materialize.sh + register-crate).
fn first_move_plan(repo_root: &Path) -> Result<Option<PathBuf>, FacesMergeError> {
    let dir = repo_root.join(MOVE_PLAN_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(FacesMergeError::new(
                FacesMergeErrorKind::Regen,
                format!("read {}: {e}", dir.display()),
            ));
        }
    };
    let mut plans: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| {
            FacesMergeError::new(
                FacesMergeErrorKind::Regen,
                format!("read entry in {}: {e}", dir.display()),
            )
        })?;
        let path = entry.path();
        let is_plan = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("-move-plan.json"));
        if is_plan {
            plans.push(path);
        }
    }
    plans.sort();
    Ok(plans.into_iter().next())
}

fn read_face(path: &Path) -> Result<String, FacesMergeError> {
    std::fs::read_to_string(path).map_err(|e| {
        FacesMergeError::new(
            FacesMergeErrorKind::Regen,
            format!("read regenerated face {}: {e}", path.display()),
        )
    })
}

fn run_regen_status(command: &mut Command, step: &str) -> Result<(), FacesMergeError> {
    let output = command.output().map_err(|e| {
        FacesMergeError::new(FacesMergeErrorKind::Regen, format!("{step}: {e}"))
    })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(regen_command_failed(step, &output))
    }
}

fn run_regen_output(command: &mut Command, step: &str) -> Result<String, FacesMergeError> {
    let output = command.output().map_err(|e| {
        FacesMergeError::new(FacesMergeErrorKind::Regen, format!("{step}: {e}"))
    })?;
    if !output.status.success() {
        return Err(regen_command_failed(step, &output));
    }
    String::from_utf8(output.stdout).map_err(|e| {
        FacesMergeError::new(
            FacesMergeErrorKind::Regen,
            format!("{step}: stdout was not UTF-8: {e}"),
        )
    })
}

fn regen_command_failed(step: &str, output: &std::process::Output) -> FacesMergeError {
    FacesMergeError::new(
        FacesMergeErrorKind::Regen,
        format!(
            "{step} failed with status {}:\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A minimal control-plane manifest fragment with one producer-target face (settle-capable), one
    /// emitter-target face (settle-capable), one regeneratable-but-controller face (NOT settle-
    /// capable), and one normal-source-merge face (not regeneratable). Exercises the full scoping.
    fn full_manifest() -> Value {
        json!({
            "artifacts": [
                {
                    "artifact_id": "producer-face",
                    "path": "faces/registry.generated.json",
                    "merge_policy": "never-manual-merge-regenerate-from-source-tree",
                    "generator": { "generator_target": PRODUCER_TARGET }
                },
                {
                    "artifact_id": "scm-facts-face",
                    "path": "faces/scm-facts.generated.json",
                    "merge_policy": "controller-owned-main-materialization",
                    "generator": { "generator_target": EMITTER_TARGET }
                },
                {
                    "artifact_id": "controller-projection",
                    "path": "docs/machine-readable/board-sync.generated.json",
                    "merge_policy": "never-manual-merge-regenerate-from-source-tree",
                    "generator": { "generator_target": "oya-ci://generated-artifact-controller/x" }
                },
                {
                    "artifact_id": "codemod-manifest",
                    "path": "specs/reorg/move-manifest.generated.json",
                    "merge_policy": "never-manual-merge-regenerate-from-source-tree",
                    "generator": { "generator_target": "//tools/oya-reorg-codemod-app:oya-reorg-codemod" }
                },
                {
                    "artifact_id": "normal-face",
                    "path": "out/normal.json",
                    "merge_policy": "normal-source-merge",
                    "generator": { "generator_target": "//normal:bin" }
                }
            ]
        })
    }

    #[test]
    fn settle_capable_is_scoped_to_producer_and_emitter_targets() {
        let cp = ControlPlane::from_manifest(&full_manifest()).expect("parse control plane");
        // Producer + emitter faces are settle-capable (the driver may cosmetically resolve them).
        assert!(cp.is_regeneratable_face("faces/registry.generated.json"));
        assert!(cp.is_regeneratable_face("faces/scm-facts.generated.json"));
        // The controller projection + codemod move-manifest are REGENERATABLE in the manifest but NOT
        // settle-capable — the driver MUST decline them (the local settle cannot re-derive them).
        assert!(cp.regeneratable_face_paths.contains("docs/machine-readable/board-sync.generated.json"));
        assert!(!cp.is_regeneratable_face("docs/machine-readable/board-sync.generated.json"));
        assert!(cp.regeneratable_face_paths.contains("specs/reorg/move-manifest.generated.json"));
        assert!(!cp.is_regeneratable_face("specs/reorg/move-manifest.generated.json"));
        // The normal-source-merge face is neither regeneratable nor settle-capable.
        assert!(!cp.is_regeneratable_face("out/normal.json"));
        // The settle-capable set is exactly the two producer/emitter faces.
        let capable = cp.settle_capable_face_paths();
        assert_eq!(capable.len(), 2);
        assert!(capable.contains("faces/registry.generated.json"));
        assert!(capable.contains("faces/scm-facts.generated.json"));
    }

    #[test]
    fn control_plane_fails_closed_on_no_regeneratable_faces() {
        let manifest = json!({
            "artifacts": [
                { "artifact_id": "a", "path": "x.json", "merge_policy": "normal-source-merge" }
            ]
        });
        let err = ControlPlane::from_manifest(&manifest).expect_err("must fail closed");
        assert_eq!(err.kind(), FacesMergeErrorKind::ControlPlane);
    }

    #[test]
    fn control_plane_fails_closed_on_missing_artifacts() {
        let err = ControlPlane::from_manifest(&json!({})).expect_err("must fail closed");
        assert_eq!(err.kind(), FacesMergeErrorKind::ControlPlane);
    }

    #[test]
    fn control_plane_skips_non_canonical_generator_target() {
        // A controller (non-buck2) target like `oya-ci://...` is not a buck2 label the local driver
        // can build — it must not enter `generator_targets` and is not settle-capable.
        let manifest = json!({
            "artifacts": [
                {
                    "artifact_id": "controller-face",
                    "path": "out/c.generated.json",
                    "merge_policy": "never-manual-merge-regenerate-from-source-tree",
                    "generator": { "generator_target": "oya-ci://generated-artifact-controller/x" }
                }
            ]
        });
        let cp = ControlPlane::from_manifest(&manifest).expect("parse control plane");
        // Declared regeneratable, but NOT settle-capable (controller target the local settle skips).
        assert!(cp.regeneratable_face_paths.contains("out/c.generated.json"));
        assert!(!cp.is_regeneratable_face("out/c.generated.json"));
        assert!(cp.generator_targets.is_empty());
    }
}
