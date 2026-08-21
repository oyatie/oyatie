//! # oya-cloud-ci-register-crate-app (ADR-0568, G011 born-accounting slice 3b)
//!
//! The ORCHESTRATOR half of `register_crate`: the integration layer that wires the pure
//! [`oya-crate-registrar-kernel`](crate_registrar_kernel)'s typed plan to the on-disk
//! writers ([`oya-crate-registrar-app`](crate_registrar_app)) and the producer's
//! registration bridges ([`oya-cloud-ci-accounting-registry-app`](ci_artifact_inventory_registry)).
//!
//! ## Dependency direction (load-bearing — ADR-0131/ADR-0512)
//! This crate lives under `ci/facade/` precisely so it MAY depend SAME-LAYER on the
//! cloud-ci producer (`fix_owners`/`fix_reachability`/`allocate_next_adr_id`) and DOWNWARD on the
//! `libs/` kernel + writers. A `libs/` crate may NOT depend on `ci/facade/` (forbidden
//! layer inversion), so the orchestration — which needs both halves — cannot live in `libs/`.
//!
//! ## What it does (the integration the pure kernel can't)
//! 1. LOADERS — read the live repo SSOTs into the kernel's input types
//!    ([`CurrentState`](crate_registrar_kernel::CurrentState),
//!    [`CapabilitySet`](crate_registrar_kernel::CapabilitySet)).
//! 2. PLAN — call [`plan_register_crate`](crate_registrar_kernel::plan_register_crate) to get
//!    the ordered, typed [`Edit`](crate_registrar_kernel::Edit) diff (fail-closed on a
//!    [`ValidationError`](crate_registrar_kernel::ValidationError)).
//! 3. DISPATCH — apply each `Edit` in order to its writer/bridge (the dispatch table below).
//! 4. OUTCOME — record what was applied (and whether faces still need a settle run).
//!
//! ## Dispatch table (Edit -> writer/bridge)
//! | `Edit`                  | applied by                                            |
//! |-------------------------|-------------------------------------------------------|
//! | `OwnersWrite`           | producer `fix_owners` (`<dir>=<owner>`)               |
//! | `WorkspaceMemberGlob`   | writer `workspace_member_glob::apply` (verify/no-op)  |
//! | `CapabilityMapping`     | writer `capability_mapping::apply`                    |
//! | `AdrGovernedPathAppend` | writer `adr_governed_paths::apply` (resolve ADR path) |
//! | `CatalogYaml`           | writer `catalog_yaml::apply`                          |
//! | `ReachabilityEntry`     | producer `fix_reachability` (`<path>=<anchor>`)       |
//! | `FacesSettle`           | `cargo metadata` lock refresh, then face settle via `RegenPort` |
//!
//! `FacesSettle` deliberately does NOT run materialize in [`register_crate`]: materialize needs
//! buck2/shell (the RegenPort is slice 3c). [`register_crate_and_settle`] is the auto-on-birth
//! entrypoint: refresh Cargo.lock with `cargo metadata`, then settle faces from that candidate tree.
//!
//! ## Fail-closed + idempotent
//! Any edit error ABORTS the dispatch and returns [`RegisterError`], reporting what was applied
//! BEFORE the failure so the caller can recover. Re-running on an already-registered crate yields
//! an empty plan (kernel idempotency) → no changes, `requires_faces_settle = false`.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic; `#![forbid(unsafe_code)]`.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use ci_artifact_inventory_registry::{
    ProducerError, adr_id_from_filename, fix_owners, fix_reachability, load_reachability_registry,
    resolve_owners,
};
// REUSE the membership-lint gate's home resolution (NOT a reimplementation). `homes_for` over
// `parse_mapping` is the SINGLE source of "which home(s) does this crate map to?" — the same logic
// the `oya-cloud-ci-capability-membership-app` gate BLOCKS on. Sharing it makes the orchestrator's
// "already capability-mapped?" check drift-proof: it sees ALL four home sources (capability dir-
// prefixes, app_products → meta:app/, meta_directory_absorbs → meta:kernel//os/, and the
// `*`-suffix glob membership) exactly as the gate does, so it never emits a spurious mapping edit
// that would DOUBLE-MAP a crate the gate then flags RED.
use ci_module_membership::{Mapping, homes_for, parse_mapping};
// The slice-3d self-validation subset gates (total-accounting / slo-coverage / catalog-liveness) are
// referenced by their full crate path in `run_self_validation` (no `use` needed) — each gate's pure
// `evaluate_keyed` is driven over the POST-settle faces so the just-registered crate is validated
// against the SAME gate logic CI runs (never a reimplementation). All three are cycle-free (they dep
// only serde_json / a downward libs/ crate, never back to this orchestrator).
use ci_config_kernel::OyaCiConfig;
use crate_registrar_app::{
    WriterError, adr_governed_paths, capability_mapping, catalog_yaml, workspace_member_glob,
};
use crate_registrar_kernel::{
    CapabilitySet, CurrentState, Edit, RegisterCrateRequest, RegistrationPlan, ValidationError,
    plan_register_crate,
};
use serde_json::Value;

/// The repo-relative closed capability registry — the SSOT for both the closed
/// [`CapabilitySet`] (group slugs) and the existing crate-glob membership.
const CAPABILITY_REGISTRY_PATH: &str = "governance/capability-registry.json";

/// The repo-relative dir the generated faces (scm-facts snapshot + producer faces) live beside.
/// Identical to the producer's default `--out-dir` and the freshness gate's `FACES_DIR`.
const FACES_DIR: &str = "ci/facade/artifact-inventory-registry";

/// The committed scm-facts snapshot face name (the emitter's `--out`, the producer's `--scm-facts`).
const SCM_FACTS_FACE: &str = "scm-facts.generated.json";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS: &str = ".claude/settings.json";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS: &str = ".codex/hooks.json";
const ENFORCEMENT_LIVENESS_HOOKS_DIR: &str = "tools/hooks";

/// The repo-relative move-manifest the codemod's `manifest` subcommand emits and the emitter
/// consumes (materialize.sh step 1 → step 2). A pure function of (committed plan + candidate tree).
const MOVE_MANIFEST_PATH: &str = "specs/reorg/move-manifest.generated.json";

/// The buck2 targets the [`Buck2RegenAdapter`] builds (mirroring materialize.sh's single
/// `buck2 build … --show-output`). Target-name match (`parse_show_output_path`) maps each to its
/// built-binary path — the same shape the freshness gate's `build_face_tools` uses.
const EMITTER_TARGET: &str = "//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot";
const PRODUCER_TARGET: &str =
    "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin";
const CODEMOD_TARGET: &str = "//tools/oya-reorg-codemod-app:oya-reorg-codemod";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_TARGET: &str = "//.claude:settings-json";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS_TARGET: &str = "//.codex:hooks-json";
const ENFORCEMENT_LIVENESS_HOOKS_DIR_TARGET: &str = "//tools/hooks:top-level-hook-scripts";

/// The 6 producer faces the producer writes (and the byte-rediff re-renders), as
/// `(file_name, --face name)`. The scm-facts snapshot is written by the emitter (re-rendered via a
/// re-run emitter is overkill — the byte-rediff re-renders only the deterministic producer faces).
const PRODUCER_FACES: [(&str, &str); 6] = [
    ("accounting-registry.generated.json", "registry"),
    ("ttl-policy.generated.json", "ttl-policy"),
    ("decision-crosswalk.generated.json", "decision-crosswalk"),
    (
        "enforcement-inventory.generated.json",
        "enforcement-inventory",
    ),
    (
        "enforcement-liveness.generated.json",
        "enforcement-liveness",
    ),
    ("gate-baseline.generated.json", "baseline"),
];

fn append_enforcement_liveness_corpus_args(
    command: &mut Command,
    corpus: &EnforcementLivenessCorpusPaths,
) {
    command
        .arg("--enforcement-liveness-claude-settings")
        .arg(&corpus.claude_settings)
        .arg("--enforcement-liveness-codex-hooks")
        .arg(&corpus.codex_hooks)
        .arg("--enforcement-liveness-hooks-dir")
        .arg(&corpus.hooks_dir);
}

/// The human remediation command for a settle failure, mirroring the freshness gate's
/// `FACE_REMEDIATION_COMMAND`.
const FACE_REMEDIATION_COMMAND: &str = "buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .";

/// The kind of edit an [`AppliedEdit`] records, mirroring the dispatched [`Edit`] variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppliedEditKind {
    /// An `OwnersWrite` edit was dispatched to the producer's `fix_owners` bridge.
    OwnersWrite,
    /// A `WorkspaceMemberGlob` coverage check was dispatched to the writer (verify-only).
    WorkspaceMemberGlob,
    /// A `CapabilityMapping` edit was dispatched to the writer.
    CapabilityMapping,
    /// An `AdrGovernedPathAppend` edit was dispatched to the writer.
    AdrGovernedPathAppend,
    /// A `CatalogYaml` edit was dispatched to the writer.
    CatalogYaml,
    /// A `ReachabilityEntry` edit was dispatched to the producer's `fix_reachability` bridge.
    ReachabilityEntry,
}

/// One dispatched edit and what it touched. `changed` is `true` when the on-disk SSOT was
/// actually rewritten, `false` when the writer/bridge found it already correct (idempotent no-op).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedEdit {
    /// Which kind of edit was applied.
    pub kind: AppliedEditKind,
    /// The repo-relative path (or dir) the edit primarily targeted.
    pub path: String,
    /// `true` iff the edit changed bytes on disk; `false` for an idempotent no-op.
    pub changed: bool,
}

/// The outcome of [`register_crate`]: the ordered edits applied (with their changed flags) plus
/// whether the generated faces still need a settle run (the `FacesSettle` obligation the
/// orchestrator records but does NOT execute — that is slice 3c's RegenPort).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    /// The edits dispatched, in plan order (one per non-`FacesSettle` edit in the plan).
    pub applied: Vec<AppliedEdit>,
    /// `true` iff the plan included a `FacesSettle` edit — i.e. some SSOT was (or would be)
    /// changed and the materialized faces must be re-settled. [`register_crate`] never runs
    /// materialize (no buck2/shell here); it records the obligation. [`register_crate_and_settle`]
    /// (slice 3c) EXECUTES it via a [`RegenPort`] and records the result in [`Outcome::faces_settled`].
    pub requires_faces_settle: bool,
    /// `Some` iff [`register_crate_and_settle`] actually ran the [`RegenPort`] to settle the faces
    /// (i.e. `requires_faces_settle` was `true`). `None` for the subprocess-free [`register_crate`]
    /// / [`register_crate_detailed`] path, and for a settle run whose plan recorded no obligation.
    pub faces_settled: Option<FacesSettled>,
    /// `true` iff [`register_crate_and_settle`] refreshed Cargo.lock (`cargo metadata`) before
    /// settling faces. This is part of the auto-on-birth contract: a new workspace crate must enter
    /// Cargo.lock before scm-facts / accounting faces are regenerated from the candidate tree.
    pub cargo_lock_refreshed: bool,
    /// `Some` iff [`register_crate_and_settle`] ran [`ValidationMode::MinimalSubset`] self-validation
    /// (the slice-3d fail-closed subset gate check). On a returned [`Outcome`] the
    /// [`SelfValidation::new_findings`] set is ALWAYS empty (a non-empty set fails closed with
    /// [`RegisterError::SelfValidationFailed`] BEFORE this is recorded). `None` for
    /// [`ValidationMode::Skip`] (slice-3c backward compat) and for the subprocess-free paths.
    pub validation: Option<SelfValidation>,
}

/// The result of executing the recorded `FacesSettle` obligation via a [`RegenPort`]: the generated
/// faces re-written from the candidate tree plus the byte-rediff verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FacesSettled {
    /// The generated-face file names re-written by the regen run (the scm-facts snapshot + the
    /// producer faces), sorted for a deterministic record.
    pub faces_written: Vec<String>,
    /// `true` iff every re-written face byte-matched a fresh per-face producer re-render
    /// (`--stdout --face <name>`) — the drift byte-rediff. A mismatch fails closed with
    /// [`RegisterError::DriftDetected`] BEFORE this is set, so on a returned [`Outcome`] this is
    /// always `true`.
    pub drift_clean: bool,
}

/// One self-validation finding scoped to the just-registered crate, normalized across the gates'
/// DIFFERING `Finding` types into a single common shape. Each gate's own `Finding` carries a
/// different field set (total-accounting = `{code,key}`; capability-membership = `{code,key,detail}`;
/// slo-coverage / catalog-liveness = `{code,key}`) — [`run_self_validation`] converts EACH into this
/// common shape separately (it never assumes a shared `Finding` type), keeping `gate` so a refusal
/// names WHICH gate flagged the crate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedFinding {
    /// The gate that emitted the finding (a stable `&'static str` gate id, e.g. the gate's `GATE_ID`).
    pub gate: &'static str,
    /// The gate's bare violation `code`.
    pub code: String,
    /// The gate finding `key` (the offending unit: a registry row path, the crate dir, or the
    /// catalog crate-id) — always scoped UNDER the just-registered crate by [`run_self_validation`].
    pub key: String,
}

/// The result of [`ValidationMode::MinimalSubset`] self-validation: the crate-scoped findings the
/// subset of gates' `evaluate_keyed` emitted for the JUST-registered crate, post-settle.
///
/// Because the crate is newly registered, ANY post-settle finding keyed under the crate's dir/paths
/// is BY CONSTRUCTION new (pre-existing corpus debt is keyed to OTHER paths and is filtered out by
/// the crate scope), so no before/after snapshot is needed — the crate scope IS the "new" filter.
/// A NON-empty set fails closed ([`RegisterError::SelfValidationFailed`]); on a returned
/// [`Outcome`] this set is therefore ALWAYS empty (success).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelfValidation {
    /// The crate-scoped findings (empty on success). A non-empty set never reaches here — it fails
    /// closed with [`RegisterError::SelfValidationFailed`] before the [`Outcome`] is built.
    pub new_findings: BTreeSet<ScopedFinding>,
}

/// A typed refusal from [`register_crate`]. Fail-closed: the orchestrator never leaves a partial,
/// silent result — every failure is a typed error carrying the edits applied before it (in
/// [`RegisterError::applied_before`]) so the caller can recover.
#[derive(Debug)]
pub enum RegisterError {
    /// The kernel refused to plan the request (invalid crate dir/capability/role/owner/ADR/path).
    Plan(ValidationError),
    /// A writer (capability-mapping / ADR governed-paths / catalog / member-glob) failed.
    Writer(WriterError),
    /// A producer bridge (`fix_owners` / `fix_reachability`) failed.
    Producer(ProducerError),
    /// A `WorkspaceMemberGlob` edit named a dir NO `[workspace].members` glob covers. The
    /// orchestrator NEVER synthesizes a glob (a covering glob is a human ADR-0538/ADR-0568 D2
    /// decision); it fails closed so the human adds the glob. The message names the dir + fix.
    MemberGlobUncovered {
        /// The uncovered crate dir.
        dir: String,
    },
    /// The owning ADR id named by an `AdrGovernedPathAppend` edit has no
    /// `docs/decisions/<id>-*.md` file (the ADR must exist before its governed surfaces can be
    /// appended). Names the id + the dir scanned.
    AdrFileNotFound {
        /// The unresolved ADR id.
        adr: String,
        /// The ADR corpus dir scanned (`cfg.justification.adr_dir`).
        adr_dir: String,
    },
    /// A filesystem / git read the orchestrator's loaders need failed.
    Io(String),
    /// Cargo.lock registration failed during auto-on-birth (`cargo metadata`). The subprocess
    /// output is included so a caller can diagnose manifest/lock resolution failures directly.
    CargoLockRefreshFailed(String),
    /// The [`RegenPort`] failed to settle the generated faces (a buck2 build, the codemod manifest,
    /// the scm-facts emitter, or the producer step). Carries the failing step's context plus the
    /// subprocess stdout/stderr so the failure is diagnosable, not opaque (fail LOUD).
    RegenFailed(String),
    /// After a settle run, a generated face did NOT byte-match a fresh per-face producer re-render
    /// (the drift byte-rediff). A mismatch means the just-written face is not the deterministic
    /// function of the candidate tree the producer would emit — fail closed rather than commit a
    /// face the registry-drift gate would then flag RED. Names the drifting face.
    DriftDetected {
        /// The generated-face file name whose on-disk bytes diverged from the fresh re-render.
        face: String,
    },
    /// [`ValidationMode::MinimalSubset`] self-validation found that the JUST-registered crate would
    /// fail a subset gate: running each gate's `evaluate_keyed` over the POST-settle faces emitted
    /// at least one finding keyed UNDER the registered crate. Fail closed rather than report success
    /// for a registration that just authored a crate the gates would then flag RED in CI. Carries the
    /// crate-scoped findings (which gate + code + key) so the refusal is diagnosable.
    SelfValidationFailed {
        /// The crate-scoped findings the subset gates emitted for the just-registered crate.
        findings: BTreeSet<ScopedFinding>,
    },
}

impl std::fmt::Display for RegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterError::Plan(e) => write!(f, "register-crate plan refused: {e:?}"),
            RegisterError::Writer(e) => write!(f, "register-crate writer failed: {e}"),
            RegisterError::Producer(e) => write!(f, "register-crate producer bridge failed: {e}"),
            RegisterError::MemberGlobUncovered { dir } => write!(
                f,
                "register-crate: crate dir {dir:?} is not covered by any [workspace].members \
                 glob. The orchestrator never synthesizes a glob (a covering glob is a human \
                 ADR-0538/ADR-0568 D2 decision). Exact fix: add a members glob in the root \
                 Cargo.toml that covers {dir:?}, then re-run register_crate."
            ),
            RegisterError::AdrFileNotFound { adr, adr_dir } => write!(
                f,
                "register-crate: owning ADR {adr} has no {adr_dir}/{adr}-*.md file — the ADR must \
                 exist before its governed surfaces can be appended"
            ),
            RegisterError::Io(m) => write!(f, "register-crate io: {m}"),
            RegisterError::CargoLockRefreshFailed(m) => {
                write!(f, "register-crate Cargo.lock refresh failed: {m}")
            }
            RegisterError::RegenFailed(m) => {
                write!(f, "register-crate faces-settle regen failed: {m}")
            }
            RegisterError::DriftDetected { face } => write!(
                f,
                "register-crate faces-settle drift: re-written face {face:?} did not byte-match a \
                 fresh producer re-render — the settled face is not the deterministic function of \
                 the candidate tree the producer emits. Fix: re-run the settle from a clean tree; \
                 remediation: {FACE_REMEDIATION_COMMAND}"
            ),
            RegisterError::SelfValidationFailed { findings } => {
                write!(
                    f,
                    "register-crate self-validation refused: the just-registered crate would fail \
                     {} subset gate finding(s) post-settle (fail-closed — registering a crate the \
                     gates flag RED is never a success):",
                    findings.len()
                )?;
                for finding in findings {
                    write!(
                        f,
                        " [{}:{} key={}]",
                        finding.gate, finding.code, finding.key
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RegisterError {}

impl From<WriterError> for RegisterError {
    fn from(e: WriterError) -> Self {
        // The member-glob uncovered case is surfaced as its own fail-closed variant (the human
        // must add a glob) rather than an opaque writer error.
        match e {
            WriterError::WorkspaceMemberUncovered(dir) => {
                RegisterError::MemberGlobUncovered { dir }
            }
            other => RegisterError::Writer(other),
        }
    }
}

impl From<ProducerError> for RegisterError {
    fn from(e: ProducerError) -> Self {
        RegisterError::Producer(e)
    }
}

impl From<ValidationError> for RegisterError {
    fn from(e: ValidationError) -> Self {
        RegisterError::Plan(e)
    }
}

/// Register a new crate's born-accounting: load the live SSOT snapshot, plan the upsert diff via
/// the pure kernel, and dispatch each edit to its writer/bridge in order.
///
/// I/O orchestration the pure kernel cannot do: reads the capability registry, OWNERS resolution,
/// the reachability registry, the ADR corpus, the catalog, and `git ls-files` (for the producer
/// bridges' self-validation). All repo specifics live here; the kernel stays R0-pure.
///
/// Fail-closed: a [`ValidationError`] from the kernel, any writer/bridge error, or an uncovered
/// member glob aborts and returns a [`RegisterError`]. On a dispatch failure the
/// [`Outcome`] of what was applied so far is returned via [`RegisterOutcome::Failed`] semantics
/// (see [`register_crate_detailed`]); this convenience wrapper returns the [`Outcome`] on success.
///
/// Idempotent: re-running on an already-registered crate yields an empty plan → no changes and
/// `requires_faces_settle = false`.
///
/// # Errors
/// [`RegisterError`] on a plan refusal, a writer/bridge failure, an uncovered member glob, a
/// missing ADR file, or a loader IO failure.
pub fn register_crate(
    repo_root: &Path,
    req: &RegisterCrateRequest,
) -> Result<Outcome, RegisterError> {
    match register_crate_detailed(repo_root, req) {
        RegisterOutcome::Done(outcome) => Ok(outcome),
        RegisterOutcome::Failed { error, .. } => Err(error),
    }
}

/// The detailed result of a dispatch: success carries the [`Outcome`]; failure carries the
/// [`RegisterError`] AND the edits applied before it aborted (so the caller can recover / report
/// the partial application). Fail-closed: a partial application never silently looks complete.
#[derive(Debug)]
pub enum RegisterOutcome {
    /// Every edit in the plan applied successfully (or the plan was a no-op).
    Done(Outcome),
    /// A dispatch error aborted the run. `applied` are the edits applied before the failure.
    Failed {
        /// The typed failure.
        error: RegisterError,
        /// The edits applied (in order) before the failure aborted the dispatch.
        applied: Vec<AppliedEdit>,
    },
}

/// Like [`register_crate`] but returns the [`RegisterOutcome`] so a caller can inspect the edits
/// applied BEFORE a dispatch failure (the recovery aid). Plan-time refusals and loader IO failures
/// return `Failed` with an empty `applied` (nothing was written).
#[must_use]
pub fn register_crate_detailed(repo_root: &Path, req: &RegisterCrateRequest) -> RegisterOutcome {
    // Source the oya-ci policy from the REPO's `oya-ci.toml` (not the compiled-in oyatie default),
    // mirroring the producer's loader — a non-oyatie adopter (neutral profile, custom
    // `reachability.registry` / `justification.adr_dir` / `owners.file_name`) gets the right paths
    // for every loader + bridge. Fail-closed on a malformed file (see [`load_config`]).
    let cfg = match load_config(repo_root) {
        Ok(c) => c,
        Err(e) => {
            return RegisterOutcome::Failed {
                error: e,
                applied: Vec::new(),
            };
        }
    };

    // --- LOADERS: live SSOT snapshot the pure kernel consumes ---
    let capabilities = match load_capability_set(repo_root) {
        Ok(c) => c,
        Err(e) => {
            return RegisterOutcome::Failed {
                error: e,
                applied: Vec::new(),
            };
        }
    };
    let tracked_paths = match list_tracked_paths(repo_root) {
        Ok(t) => t,
        Err(e) => {
            return RegisterOutcome::Failed {
                error: e,
                applied: Vec::new(),
            };
        }
    };
    let current = match load_current_state(repo_root, &cfg, req, &tracked_paths) {
        Ok(c) => c,
        Err(e) => {
            return RegisterOutcome::Failed {
                error: e,
                applied: Vec::new(),
            };
        }
    };

    // --- PLAN: the pure kernel computes the ordered upsert diff ---
    let plan: RegistrationPlan = match plan_register_crate(req, &current, &capabilities) {
        Ok(p) => p,
        Err(e) => {
            return RegisterOutcome::Failed {
                error: RegisterError::Plan(e),
                applied: Vec::new(),
            };
        }
    };

    // --- DISPATCH: each edit, in order, to its writer/bridge ---
    let mut applied: Vec<AppliedEdit> = Vec::new();
    let mut requires_faces_settle = false;
    for edit in &plan.edits {
        match dispatch_edit(repo_root, &cfg, &tracked_paths, edit) {
            Ok(Some(applied_edit)) => applied.push(applied_edit),
            // `FacesSettle` is recorded, not executed (RegenPort = slice 3c).
            Ok(None) => requires_faces_settle = true,
            Err(error) => return RegisterOutcome::Failed { error, applied },
        }
    }

    RegisterOutcome::Done(Outcome {
        applied,
        requires_faces_settle,
        faces_settled: None,
        cargo_lock_refreshed: false,
        validation: None,
    })
}

fn cargo_lock_refresh_required(
    repo_root: &Path,
    req: &RegisterCrateRequest,
) -> Result<bool, RegisterError> {
    let package_name = request_package_name(repo_root, req)?;
    Ok(!cargo_lock_contains_package(repo_root, &package_name)?)
}

fn request_package_name(
    repo_root: &Path,
    req: &RegisterCrateRequest,
) -> Result<String, RegisterError> {
    let manifest = repo_root
        .join(req.crate_dir.trim_end_matches('/'))
        .join("Cargo.toml");
    let contents = std::fs::read_to_string(&manifest)
        .map_err(|error| RegisterError::Io(format!("{}: {error}", manifest.display())))?;
    parse_package_table_string(&contents, "name").ok_or_else(|| {
        RegisterError::Io(format!(
            "{}: missing parseable [package] name",
            manifest.display()
        ))
    })
}

fn cargo_lock_contains_package(
    repo_root: &Path,
    package_name: &str,
) -> Result<bool, RegisterError> {
    let lockfile = repo_root.join("Cargo.lock");
    let contents = match std::fs::read_to_string(&lockfile) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(RegisterError::Io(format!(
                "{}: {error}",
                lockfile.display()
            )));
        }
    };

    let mut in_package = false;
    for raw in contents.lines() {
        let trimmed = raw.split('#').next().unwrap_or("").trim();
        if trimmed == "[[package]]" {
            in_package = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_package = false;
            continue;
        }
        if in_package
            && parse_key_value_string(trimmed, "name")
                .as_deref()
                .is_some_and(|name| name == package_name)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parse_package_table_string(contents: &str, key: &str) -> Option<String> {
    let mut in_package = false;
    for raw in contents.lines() {
        let trimmed = raw.split('#').next().unwrap_or("").trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package && let Some(value) = parse_key_value_string(trimmed, key) {
            return Some(value);
        }
    }
    None
}

fn parse_key_value_string(line: &str, expected_key: &str) -> Option<String> {
    let (key, value) = line.split_once('=')?;
    if key.trim() != expected_key {
        return None;
    }
    let value = value.trim().trim_matches('"').trim_matches('\'');
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

// ───────────────────────────── faces-settle (slice 3c) ─────────────────────────────

/// How [`register_crate_and_settle`] self-validates the settled tree.
///
/// [`ValidationMode::Skip`] (slice 3c) applies the SSOT edits, executes the recorded `FacesSettle`
/// obligation via the [`RegenPort`], and runs the byte-rediff drift check — no in-process gate
/// evaluation. [`ValidationMode::MinimalSubset`] (slice 3d) adds FAIL-CLOSED self-validation: AFTER
/// the faces settle, it runs a high-value SUBSET of gates' `evaluate_keyed` over the POST-settle faces
/// and REFUSES success ([`RegisterError::SelfValidationFailed`]) if the just-registered crate would
/// fail any of them (see [`run_self_validation`]). `Skip` stays backward-compatible (no self-validation,
/// [`Outcome::validation`] = `None`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    /// Settle the faces (regen + byte-rediff) with NO in-process gate self-validation.
    Skip,
    /// Settle the faces, THEN run the slice-3d crate-scoped subset gate self-validation: refuse
    /// success ([`RegisterError::SelfValidationFailed`]) if the just-registered crate would fail a
    /// subset gate post-settle (total-accounting + capability-membership always; slo-coverage +
    /// catalog-liveness when a catalog edit was applied).
    MinimalSubset,
}

/// The port [`register_crate_and_settle`] uses to EXECUTE the recorded `FacesSettle` obligation:
/// regenerate the committed generated faces from the candidate tree. Injectable so unit tests run
/// with NO buck2 ([`FakeRegenPort`]); the one production implementation is [`Buck2RegenAdapter`].
///
/// `regenerate` MUST write the REAL faces under [`FACES_DIR`] (the scm-facts snapshot + the 6
/// producer faces) — register-crate is SETTLING (mutating on purpose), the inverse of the freshness
/// gate's read-only temp routing. It returns the sorted file names it wrote so the caller can record
/// them and run the byte-rediff over them.
///
/// `verify_drift` is the byte-rediff: it re-renders each producer face from the candidate tree and
/// byte-compares to the just-written face, failing closed with [`RegisterError::DriftDetected`] on a
/// mismatch. It lives on the port (not a free fn) so it is injectable — a unit test can drive the
/// drift verdict via [`FakeRegenPort`] with NO buck2 (the byte-rediff re-runs the BUILT producer,
/// which a unit test cannot).
///
/// # Errors
/// [`RegisterError::RegenFailed`] on any step failure (build, codemod, emitter, producer), carrying
/// the failing step's context + subprocess output. Never swallow a failure (fail LOUD).
/// [`RegisterError::DriftDetected`] from `verify_drift` on a byte mismatch.
pub trait RegenPort {
    /// Refresh Cargo.lock for the candidate tree (`cargo metadata >/dev/null`) before generated
    /// faces are settled. Auto-on-birth must not let a new workspace crate reach face generation
    /// while absent from Cargo.lock; the refreshed lock becomes part of the same candidate tree the
    /// scm-facts emitter and accounting producer consume.
    fn refresh_cargo_lock(&self, repo_root: &Path) -> Result<(), RegisterError>;

    /// Regenerate the committed generated faces from the candidate tree at `repo_root`. Returns the
    /// sorted generated-face file names written.
    fn regenerate(&self, repo_root: &Path) -> Result<Vec<String>, RegisterError>;

    /// The byte-rediff drift check: re-render each producer face from the candidate tree and
    /// byte-compare to the just-written face on disk. `Ok(())` iff every face byte-matches.
    fn verify_drift(&self, repo_root: &Path) -> Result<(), RegisterError>;

    /// Render a STDOUT-ONLY producer gate-input face (e.g. `slo-coverage` / `catalog-liveness`) as a
    /// `Value`, for slice-3d self-validation. These faces are NOT written to disk — the producer
    /// derives them via private `collect_*` fns and only emits them on `--stdout --face <name>` — so
    /// self-validation cannot read them off the tree; it must drive the BUILT producer. The
    /// production [`Buck2RegenAdapter`] runs `producer --stdout --face <face> --scm-facts <face>` over
    /// the POST-settle scm-facts snapshot; [`FakeRegenPort`] injects a crafted face so unit tests
    /// drive the REAL gate `evaluate_keyed` with NO buck2. The on-disk producer faces
    /// (`accounting-registry.generated.json`) and the live-tree membership scan are read directly by
    /// [`run_self_validation`], so they do NOT route through this method.
    ///
    /// # Errors
    /// [`RegisterError::RegenFailed`] if the producer cannot render the face.
    fn gate_input_face(&self, repo_root: &Path, face: &str) -> Result<Value, RegisterError>;
}

/// The production [`RegenPort`]: invokes the BUILT binaries (codemod → emitter → producer,
/// the load-bearing order) directly from Rust. It never shells through an external materializer
/// bridge. This mirrors the doctrine-blessed precedent in
/// `oya-cloud-ci-freshness-app::regenerate_faces_with_buck2` (lib.rs:726) — `Command::new` at the
/// built-binary edge is the ESTABLISHED pattern, not a new CLI surface.
///
/// The CRITICAL inversion vs the freshness gate: that gate routes the emitter to a TEMP scm-facts
/// path because `--verify` is contractually read-only; this adapter writes the REAL
/// `<FACES_DIR>/scm-facts.generated.json` because settling MUTATES on purpose — a temp path would
/// leave the committed snapshot stale → registry-drift RED in CI.
///
// IRREDUCIBLE-GLUE LEDGER (ADR-0523 item 2 / ADR-0525): git+buck2 at the graph edge
#[derive(Debug, Clone, Copy, Default)]
pub struct Buck2RegenAdapter;

impl RegenPort for Buck2RegenAdapter {
    fn refresh_cargo_lock(&self, repo_root: &Path) -> Result<(), RegisterError> {
        let mut command = Command::new("cargo");
        command.arg("metadata").current_dir(repo_root);
        run_cargo_lock_status(&mut command, "cargo metadata")
    }

    fn regenerate(&self, repo_root: &Path) -> Result<Vec<String>, RegisterError> {
        let tools = build_face_tools(repo_root)?;

        // 1. codemod(manifest): regenerate the de-committed move-manifest (ADR-0614) from the
        //    candidate tree, so the emitter consumes a FRESH copy. The codemod is the authoritative
        //    selector: it excludes landed/PARKED plans, fails closed on multiple active plans, and
        //    emits the canonical empty manifest when none is active. ORDER is load-bearing
        //    (materialize step 1).
        let manifest_out = repo_root.join(MOVE_MANIFEST_PATH);
        let mut codemod = Command::new(&tools.codemod);
        append_manifest_args(&mut codemod, repo_root, &manifest_out);
        codemod.current_dir(repo_root);
        run_settle_status(&mut codemod, "codemod manifest")?;

        // 2. emitter: write the REAL scm-facts snapshot (+ the frozen gate-baseline snapshot the
        //    --merge-base-baseline flag materializes). Default --frozen-base-ref (origin/dev) — the
        //    canonical materialize.sh passes no explicit --frozen-base-ref, so neither do we.
        let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
        run_settle_status(
            Command::new(&tools.emitter)
                .args(["--repo-root"])
                .arg(repo_root)
                .args(["--out"])
                .arg(&scm_facts)
                .arg("--merge-base-baseline")
                .current_dir(repo_root),
            "scm-facts emitter",
        )?;

        // 3. producer: regenerate the 6 producer faces from the just-written scm-facts. The emitter
        //    MUST have succeeded first — a missing scm-facts is a HARD producer error (main.rs:84);
        //    we never reach here on an emitter failure (step 2 propagates as RegenFailed).
        let mut producer_command = Command::new(&tools.producer);
        producer_command
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--scm-facts"])
            .arg(&scm_facts);
        append_enforcement_liveness_corpus_args(
            &mut producer_command,
            &tools.enforcement_liveness_corpus,
        );
        producer_command.current_dir(repo_root);
        run_settle_status(&mut producer_command, "accounting-registry producer")?;

        let mut written: Vec<String> = vec![SCM_FACTS_FACE.to_owned()];
        for (file_name, _face) in PRODUCER_FACES {
            written.push(file_name.to_owned());
        }
        written.sort();
        Ok(written)
    }

    fn verify_drift(&self, repo_root: &Path) -> Result<(), RegisterError> {
        // Re-build the tools (a warm buck2 cache makes the re-build a near-no-op) so the rediff
        // stands alone — it does NOT depend on `regenerate` having stashed the built binaries. The
        // scm-facts snapshot is the emitter's output (no per-face producer re-render), so the rediff
        // covers only the 6 deterministic producer faces — exactly the faces a stale-tree mistake
        // would corrupt. `write_face` and `--stdout` share `to_canonical_json`, so a match is exact.
        let tools = build_face_tools(repo_root)?;
        let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
        for (file_name, face_name) in PRODUCER_FACES {
            let mut command = Command::new(&tools.producer);
            command
                .args(["--repo-root"])
                .arg(repo_root)
                .args(["--scm-facts"])
                .arg(&scm_facts);
            append_enforcement_liveness_corpus_args(
                &mut command,
                &tools.enforcement_liveness_corpus,
            );
            command
                .args(["--stdout", "--face", face_name])
                .current_dir(repo_root);
            let rerender =
                run_settle_output(&mut command, &format!("byte-rediff re-render {file_name}"))?;
            let on_disk_path = repo_root.join(FACES_DIR).join(file_name);
            let on_disk = std::fs::read_to_string(&on_disk_path).map_err(|e| {
                RegisterError::RegenFailed(format!(
                    "read {} for byte-rediff: {e}",
                    on_disk_path.display()
                ))
            })?;
            if on_disk != rerender {
                return Err(RegisterError::DriftDetected {
                    face: file_name.to_owned(),
                });
            }
        }
        Ok(())
    }

    fn gate_input_face(&self, repo_root: &Path, face: &str) -> Result<Value, RegisterError> {
        // Render the stdout-only gate-input face from the POST-settle scm-facts snapshot. The
        // emitter must already have written <FACES_DIR>/scm-facts.generated.json (regenerate runs
        // before self-validation), so the producer reads a fresh declared input — no ambient git.
        let tools = build_face_tools(repo_root)?;
        let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
        let mut command = Command::new(&tools.producer);
        command
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--scm-facts"])
            .arg(&scm_facts);
        append_enforcement_liveness_corpus_args(&mut command, &tools.enforcement_liveness_corpus);
        command
            .args(["--stdout", "--face", face])
            .current_dir(repo_root);
        let rendered = run_settle_output(
            &mut command,
            &format!("self-validation gate-input face {face}"),
        )?;
        serde_json::from_str(&rendered)
            .map_err(|e| RegisterError::RegenFailed(format!("parse gate-input face {face}: {e}")))
    }
}

/// The built face-generation binaries the [`Buck2RegenAdapter`] subprocesses.
struct FaceTools {
    codemod: std::path::PathBuf,
    emitter: std::path::PathBuf,
    producer: std::path::PathBuf,
    enforcement_liveness_corpus: EnforcementLivenessCorpusPaths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnforcementLivenessCorpusPaths {
    claude_settings: std::path::PathBuf,
    codex_hooks: std::path::PathBuf,
    hooks_dir: std::path::PathBuf,
}

/// `buck2 build` the emitter, producer `-bin`, and codemod in ONE invocation with `--show-output`,
/// then parse each output binary path by target-name match — mirroring the freshness gate's
/// `build_face_tools` (lib.rs:941) and materialize.sh's single build call.
///
/// # Errors
/// [`RegisterError::RegenFailed`] if the build fails or `--show-output` omits a target.
fn build_face_tools(repo_root: &Path) -> Result<FaceTools, RegisterError> {
    let output = run_settle_output(
        Command::new("buck2")
            .arg("build")
            .arg(EMITTER_TARGET)
            .arg(PRODUCER_TARGET)
            .arg(CODEMOD_TARGET)
            .arg(ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_TARGET)
            .arg(ENFORCEMENT_LIVENESS_CODEX_HOOKS_TARGET)
            .arg(ENFORCEMENT_LIVENESS_HOOKS_DIR_TARGET)
            .arg("--show-output")
            .current_dir(repo_root),
        "buck2 build face tools",
    )?;
    let emitter = parse_show_output_path(repo_root, &output, EMITTER_TARGET)?;
    let producer = parse_show_output_path(repo_root, &output, PRODUCER_TARGET)?;
    let codemod = parse_show_output_path(repo_root, &output, CODEMOD_TARGET)?;
    let enforcement_liveness_corpus = parse_enforcement_liveness_corpus_paths(repo_root, &output)?;
    Ok(FaceTools {
        codemod,
        emitter,
        producer,
        enforcement_liveness_corpus,
    })
}

/// Parse a single `<target> <path>` line out of `buck2 build --show-output` by target-name match,
/// resolving a relative path against `repo_root` (mirrors freshness `parse_show_output_path`:956).
fn parse_show_output_path(
    repo_root: &Path,
    output: &str,
    target: &str,
) -> Result<std::path::PathBuf, RegisterError> {
    for line in output.lines() {
        let mut parts = line.split_whitespace();
        let seen_target = parts.next().unwrap_or_default();
        let path = parts.next().unwrap_or_default();
        if seen_target.contains(target) && !path.is_empty() {
            let path = std::path::PathBuf::from(path);
            return Ok(if path.is_absolute() {
                path
            } else {
                repo_root.join(path)
            });
        }
    }
    Err(RegisterError::RegenFailed(format!(
        "buck2 --show-output did not include {target}"
    )))
}

fn parse_enforcement_liveness_corpus_paths(
    repo_root: &Path,
    output: &str,
) -> Result<EnforcementLivenessCorpusPaths, RegisterError> {
    let claude_settings_output = parse_show_output_path(
        repo_root,
        output,
        ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_TARGET,
    )?;
    let codex_hooks_output =
        parse_show_output_path(repo_root, output, ENFORCEMENT_LIVENESS_CODEX_HOOKS_TARGET)?;
    let hooks_dir =
        parse_show_output_path(repo_root, output, ENFORCEMENT_LIVENESS_HOOKS_DIR_TARGET)?;
    Ok(EnforcementLivenessCorpusPaths {
        claude_settings: buck_filegroup_file(claude_settings_output, "settings.json"),
        codex_hooks: buck_filegroup_file(codex_hooks_output, "hooks.json"),
        hooks_dir,
    })
}

fn buck_filegroup_file(output: std::path::PathBuf, file_name: &str) -> std::path::PathBuf {
    if output.is_file() {
        output
    } else {
        output.join(file_name)
    }
}

fn append_manifest_args(command: &mut Command, repo_root: &Path, out: &Path) {
    command
        .arg("manifest")
        .arg("--repo-root")
        .arg(repo_root)
        .arg("--out")
        .arg(out);
}

/// Run the Cargo.lock refresh subprocess (`cargo metadata`) for its exit status. This is the
/// auto-on-birth Cargo.lock registration edge: it intentionally captures stdout (like shell
/// `>/dev/null`) and returns a lock-refresh-specific failure rather than a face-regeneration error.
fn run_cargo_lock_status(command: &mut Command, step: &str) -> Result<(), RegisterError> {
    let output = command
        .output()
        .map_err(|e| RegisterError::CargoLockRefreshFailed(format!("{step}: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(cargo_lock_command_failed(step, &output))
    }
}

fn cargo_lock_command_failed(step: &str, output: &std::process::Output) -> RegisterError {
    RegisterError::CargoLockRefreshFailed(format!(
        "{step} failed with status {}:\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

/// Run a settle subprocess for its exit status, folding a non-zero exit (with stdout+stderr) into
/// [`RegisterError::RegenFailed`]. Mirrors freshness `run_status`:1011.
fn run_settle_status(command: &mut Command, step: &str) -> Result<(), RegisterError> {
    let output = command
        .output()
        .map_err(|e| RegisterError::RegenFailed(format!("{step}: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(settle_command_failed(step, &output))
    }
}

/// Run a settle subprocess and capture its stdout (UTF-8), folding failure into
/// [`RegisterError::RegenFailed`]. Mirrors freshness `run_output`:1000.
fn run_settle_output(command: &mut Command, step: &str) -> Result<String, RegisterError> {
    let output = command
        .output()
        .map_err(|e| RegisterError::RegenFailed(format!("{step}: {e}")))?;
    if !output.status.success() {
        return Err(settle_command_failed(step, &output));
    }
    String::from_utf8(output.stdout)
        .map_err(|e| RegisterError::RegenFailed(format!("{step}: stdout was not UTF-8: {e}")))
}

/// Render a failed settle subprocess into a diagnosable [`RegisterError::RegenFailed`] (context +
/// status + stdout + stderr — fail LOUD). Mirrors freshness `command_failed`:1022.
fn settle_command_failed(step: &str, output: &std::process::Output) -> RegisterError {
    RegisterError::RegenFailed(format!(
        "{step} failed with status {}:\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

/// Register a new crate's born-accounting AND settle the generated faces: apply the SSOT edits (the
/// same dispatch [`register_crate_detailed`] runs), then — iff the plan recorded a `FacesSettle`
/// obligation — EXECUTE it via `regen`, run the byte-rediff drift check, and record the result.
///
/// This is the slice-3c "close the loop" entrypoint: [`register_crate`] records but does NOT execute
/// the obligation; this one executes it through an injectable [`RegenPort`] ([`Buck2RegenAdapter`]
/// in production, [`FakeRegenPort`] in unit tests).
///
/// The byte-rediff drift check: for each producer face, re-run the BUILT producer `--stdout --face
/// <name>` and byte-compare to the just-written face on disk (`write_face` and `--stdout` share the
/// same `to_canonical_json`, so a match is exact). A mismatch fails closed with
/// [`RegisterError::DriftDetected`] rather than commit a face the registry-drift gate would flag RED.
///
/// `validate` is [`ValidationMode::Skip`] for the slice-3c "just settle" path (no in-process gate
/// self-validation); [`ValidationMode::MinimalSubset`] (slice 3d) additionally runs the crate-scoped
/// subset gate self-validation over the POST-settle faces and fails closed
/// ([`RegisterError::SelfValidationFailed`]) if the just-registered crate would fail a subset gate.
///
/// Fail-closed: a plan refusal, a writer/bridge failure, a regen failure
/// ([`RegisterError::RegenFailed`]), a drift mismatch ([`RegisterError::DriftDetected`]), or a
/// self-validation finding ([`RegisterError::SelfValidationFailed`]) aborts and returns a
/// [`RegisterError`]. A no-obligation plan (`requires_faces_settle == false`) does NOT call `regen`
/// and leaves [`Outcome::faces_settled`] `None` (and self-validation is skipped — nothing changed).
///
/// # Errors
/// [`RegisterError`] on a plan/writer/bridge failure, a regen failure, a drift mismatch, or a
/// self-validation finding.
pub fn register_crate_and_settle(
    repo_root: &Path,
    req: &RegisterCrateRequest,
    regen: &dyn RegenPort,
    validate: ValidationMode,
) -> Result<Outcome, RegisterError> {
    // Apply the SSOT edits via the existing subprocess-free dispatch (existing tests unbroken).
    let mut outcome = match register_crate_detailed(repo_root, req) {
        RegisterOutcome::Done(outcome) => outcome,
        RegisterOutcome::Failed { error, .. } => return Err(error),
    };

    // Nothing changed AND the crate is already represented in Cargo.lock ⇒ no obligation ⇒ no
    // regen (faces_settled stays None). If a prior attempt applied the SSOT edits but failed during
    // Cargo.lock refresh, the pure plan is now empty; the missing lock entry preserves the settle
    // obligation so a retry cannot falsely report success.
    let lock_refresh_required = cargo_lock_refresh_required(repo_root, req)?;
    if !outcome.requires_faces_settle && !lock_refresh_required {
        return Ok(outcome);
    }
    if lock_refresh_required {
        outcome.requires_faces_settle = true;
    }

    // Auto-on-birth Cargo.lock registration: refresh the lockfile BEFORE regenerating scm-facts and
    // producer faces, so the generated accounting surfaces observe the same candidate tree the
    // contributor will commit. This is the mechanical `cargo metadata >/dev/null` remediation folded
    // into the crate-birth primitive instead of left as a later freshness-gate surprise.
    regen.refresh_cargo_lock(repo_root)?;
    outcome.cargo_lock_refreshed = true;

    // EXECUTE the recorded FacesSettle obligation: regen the committed faces from the candidate tree.
    let faces_written = regen.regenerate(repo_root)?;

    // Byte-rediff drift check: re-render each producer face and byte-compare to the just-written one.
    regen.verify_drift(repo_root)?;

    outcome.faces_settled = Some(FacesSettled {
        faces_written,
        drift_clean: true,
    });

    // Slice 3d: MinimalSubset runs the crate-scoped subset gate self-validation AFTER the faces are
    // settled (so each gate's evaluate_keyed sees the POST-settle faces). Skip stays a no-op (3c
    // backward compat). Fail-closed: a non-empty crate-scoped finding set refuses success.
    if validate == ValidationMode::MinimalSubset {
        let new_findings = run_self_validation(repo_root, req, &outcome, regen)?;
        if !new_findings.is_empty() {
            return Err(RegisterError::SelfValidationFailed {
                findings: new_findings,
            });
        }
        outcome.validation = Some(SelfValidation { new_findings });
    }

    Ok(outcome)
}

// ───────────────────────────── self-validation (slice 3d) ─────────────────────────────

/// The committed total-accounting gate-input face FILE NAME (the producer's `registry` face) — the
/// FIRST entry of [`PRODUCER_FACES`], written under [`FACES_DIR`]. Read directly off the POST-settle
/// tree (no producer re-run needed: it is a committed face), so total-accounting self-validation needs
/// NO buck2 in tests.
const TOTAL_ACCOUNTING_FACE_FILE: &str = PRODUCER_FACES[0].0;

/// The committed capability-membership gate policy (the `--policy` default the gate's main loads).
/// Self-validation re-runs the gate's own `collect` (a live-tree scan) + `evaluate_keyed` over it.
const MEMBERSHIP_POLICY_PATH: &str =
    "ci/facade/module-membership/capability-membership-policy.json";

/// The stdout-only producer `--face` names for the slo-coverage / catalog-liveness gate inputs (NOT
/// committed faces — rendered via [`RegenPort::gate_input_face`]). Run ONLY when a [`CatalogYaml`]
/// edit was applied (a catalog-less crate has no slo/catalog row, so those gates are silent for it).
const SLO_COVERAGE_FACE: &str = "slo-coverage";
const CATALOG_LIVENESS_FACE: &str = "catalog-liveness";

/// Run [`ValidationMode::MinimalSubset`] self-validation: run a high-value SUBSET of gates'
/// `evaluate_keyed` over the POST-settle faces and collect ONLY the findings keyed UNDER the
/// just-registered crate.
///
/// Crate-scoped == "new" (the design's endorsed simplification): because the crate is newly
/// registered, ANY post-settle finding keyed under its dir/paths is BY CONSTRUCTION new — pre-existing
/// corpus debt is keyed to OTHER paths and is filtered out by the crate scope — so NO before/after
/// snapshot is needed (which also sidesteps the fact that the settle already overwrote the pre-edit
/// faces). Each gate's `Finding` type DIFFERS (total-accounting = `{code,key}`; capability-membership
/// = `{code,key,detail}`; slo-coverage / catalog-liveness = `{code,key}`) — each is converted into the
/// common [`ScopedFinding`] separately; no shared `Finding` type is assumed.
///
/// The subset (firewall + the full-set crate-authoring gates are EXCLUDED per the design — firewall
/// needs the merge-base frozen baseline that is CI-tier's authority; registry-drift is already covered
/// by slice 3c's `verify_drift`):
///   - total-accounting  — ALWAYS; on-disk `registry` face; keys = registry row paths under the crate.
///   - capability-membership — ALWAYS; live-tree `collect` + `evaluate_keyed`; key == crate dir.
///   - slo-coverage      — IFF a CatalogYaml edit applied; stdout-only face; key == catalog crate-id.
///   - catalog-liveness  — IFF a CatalogYaml edit applied; stdout-only face; key == catalog crate-id.
///
/// # Errors
/// [`RegisterError::Io`] on a face/policy read or parse failure, or [`RegisterError::RegenFailed`]
/// from [`RegenPort::gate_input_face`] when a stdout-only face cannot be rendered. Fail-closed: a
/// load failure refuses rather than silently passing self-validation.
fn run_self_validation(
    repo_root: &Path,
    req: &RegisterCrateRequest,
    outcome: &Outcome,
    regen: &dyn RegenPort,
) -> Result<BTreeSet<ScopedFinding>, RegisterError> {
    let crate_dir = req.crate_dir.trim_end_matches('/').to_owned();
    let mut findings: BTreeSet<ScopedFinding> = BTreeSet::new();

    // 1. total-accounting — ALWAYS. Read the on-disk `registry` face; keys are registry row paths.
    //    Scope to findings whose key is the crate dir OR a path under it (`<crate_dir>/...`).
    let ta_face_rel = format!("{FACES_DIR}/{TOTAL_ACCOUNTING_FACE_FILE}");
    let ta_face = load_committed_face(repo_root, &ta_face_rel)?;
    for finding in ci_artifact_accountability::evaluate_keyed(&ta_face) {
        if key_under_crate(&finding.key, &crate_dir) {
            findings.insert(ScopedFinding {
                gate: ci_artifact_accountability::GATE_ID,
                code: finding.code,
                key: finding.key,
            });
        }
    }

    // 2. capability-membership — ALWAYS. Re-run the gate's OWN collect (live-tree scan) +
    //    evaluate_keyed over the committed policy. Scope to `key == crate_dir` (the gate keys a
    //    crate finding by its dir).
    let policy = load_committed_face(repo_root, MEMBERSHIP_POLICY_PATH)?;
    let observed = ci_module_membership::collect(repo_root, &policy)
        .map_err(|e| RegisterError::Io(format!("capability-membership collect: {e}")))?;
    for finding in ci_module_membership::evaluate_keyed(&policy, &observed) {
        if finding.key == crate_dir {
            findings.insert(ScopedFinding {
                gate: ci_module_membership::GATE_ID,
                code: finding.code,
                key: finding.key,
            });
        }
    }

    // 3 + 4. slo-coverage / catalog-liveness — ONLY iff a CatalogYaml edit was applied. The faces are
    //    stdout-only (the producer derives them via private collect_* fns), so they are rendered via
    //    the RegenPort. Scope to the crate's catalog crate-id (the catalog leaf), which is the gate
    //    finding key for those gates.
    if catalog_edit_applied(outcome) {
        let catalog_id = catalog_crate_id(&crate_dir);

        let slo_face = regen.gate_input_face(repo_root, SLO_COVERAGE_FACE)?;
        for finding in ci_slo_coverage::evaluate_keyed(&slo_face) {
            if finding.key == catalog_id {
                findings.insert(ScopedFinding {
                    gate: ci_slo_coverage::GATE_ID,
                    code: finding.code,
                    key: finding.key,
                });
            }
        }

        let catalog_face = regen.gate_input_face(repo_root, CATALOG_LIVENESS_FACE)?;
        for finding in ci_service_catalog_parity::evaluate_keyed(&catalog_face) {
            if finding.key == catalog_id {
                findings.insert(ScopedFinding {
                    gate: ci_service_catalog_parity::GATE_ID,
                    code: finding.code,
                    key: finding.key,
                });
            }
        }
    }

    Ok(findings)
}

/// True iff `key` is the crate dir itself or a path strictly under it (`<crate_dir>/...`). The
/// crate-scope filter that makes a post-settle finding "new" by construction (pre-existing debt is
/// keyed to OTHER paths).
fn key_under_crate(key: &str, crate_dir: &str) -> bool {
    key == crate_dir || key.starts_with(&format!("{crate_dir}/"))
}

/// True iff the dispatched plan applied a [`AppliedEditKind::CatalogYaml`] edit — the condition under
/// which the slo-coverage / catalog-liveness gates have a row for this crate (a catalog-less crate
/// has no such row, so those gates are silent for it and must NOT be run).
fn catalog_edit_applied(outcome: &Outcome) -> bool {
    outcome
        .applied
        .iter()
        .any(|a| a.kind == AppliedEditKind::CatalogYaml)
}

/// The catalog crate-id (the catalog leaf) for `crate_dir` — the file stem of the crate's
/// `registry/catalog/<leaf>.yaml`, which is the `crate_id` key the slo-coverage / catalog-liveness
/// gates emit. It is the last path segment of the crate dir (the producer's `file_stem` of the
/// catalog source path), matching the catalog writer's `catalog_path` leaf.
fn catalog_crate_id(crate_dir: &str) -> String {
    crate_dir.rsplit('/').next().unwrap_or(crate_dir).to_owned()
}

/// Read + parse a committed JSON face / policy off the POST-settle tree. Fail-closed
/// ([`RegisterError::Io`]) on a read or parse failure — self-validation never silently passes on a
/// missing face.
fn load_committed_face(repo_root: &Path, rel: &str) -> Result<Value, RegisterError> {
    let abs = repo_root.join(rel);
    let text = std::fs::read_to_string(&abs)
        .map_err(|e| RegisterError::Io(format!("read {rel} for self-validation: {e}")))?;
    serde_json::from_str(&text)
        .map_err(|e| RegisterError::Io(format!("parse {rel} for self-validation: {e}")))
}

/// A test-double [`RegenPort`] so unit tests exercise [`register_crate_and_settle`] with NO buck2.
/// It records every `repo_root` it was asked to settle/verify, optionally simulates a regen failure,
/// and optionally simulates a drift on a named face so a test can drive every settle path
/// deterministically.
///
/// Gated behind `cfg(test)`: it is a test affordance, not a production surface (production uses
/// [`Buck2RegenAdapter`] only).
#[cfg(test)]
pub struct FakeRegenPort {
    /// If `true`, `regenerate` returns [`RegisterError::RegenFailed`] without touching the tree.
    pub fail: bool,
    /// If `true`, `refresh_cargo_lock` returns [`RegisterError::CargoLockRefreshFailed`] before any
    /// face regeneration can run.
    pub fail_lock_refresh: bool,
    /// The face file names `regenerate` claims to have written (returned on success).
    pub faces_written: Vec<String>,
    /// If `Some(face)`, `verify_drift` returns [`RegisterError::DriftDetected`] for that face.
    pub drift_face: Option<String>,
    /// `repo_root`s `refresh_cargo_lock` was called with, in order.
    pub lock_refresh_calls: std::cell::RefCell<Vec<std::path::PathBuf>>,
    /// `repo_root`s `regenerate` was called with, in order (so a test can assert it ran / did not).
    pub regen_calls: std::cell::RefCell<Vec<std::path::PathBuf>>,
    /// `repo_root`s `verify_drift` was called with, in order.
    pub verify_calls: std::cell::RefCell<Vec<std::path::PathBuf>>,
    /// The stdout-only gate-input faces `gate_input_face` returns, keyed by `--face` name (e.g.
    /// `slo-coverage` / `catalog-liveness`). A test seeds CRAFTED faces so the REAL gate
    /// `evaluate_keyed` is driven with NO buck2. A face name not present yields an empty `{"rows":[]}`
    /// (a benign "no rows for this crate" input that emits no crate-keyed finding).
    pub gate_faces: std::collections::BTreeMap<String, Value>,
    /// The `(repo_root, face)` pairs `gate_input_face` was called with, in order (so a test can
    /// assert WHICH stdout-only faces were rendered — e.g. that slo/catalog ran only on a catalog edit).
    pub gate_face_calls: std::cell::RefCell<Vec<(std::path::PathBuf, String)>>,
    /// Coarse ordered event log across lock refresh + face settle steps.
    pub events: std::cell::RefCell<Vec<String>>,
}

#[cfg(test)]
impl Default for FakeRegenPort {
    fn default() -> Self {
        Self {
            fail: false,
            fail_lock_refresh: false,
            faces_written: PRODUCER_FACES
                .iter()
                .map(|(file_name, _)| (*file_name).to_owned())
                .chain(std::iter::once(SCM_FACTS_FACE.to_owned()))
                .collect(),
            drift_face: None,
            lock_refresh_calls: std::cell::RefCell::new(Vec::new()),
            regen_calls: std::cell::RefCell::new(Vec::new()),
            verify_calls: std::cell::RefCell::new(Vec::new()),
            gate_faces: std::collections::BTreeMap::new(),
            gate_face_calls: std::cell::RefCell::new(Vec::new()),
            events: std::cell::RefCell::new(Vec::new()),
        }
    }
}

#[cfg(test)]
impl RegenPort for FakeRegenPort {
    fn refresh_cargo_lock(&self, repo_root: &Path) -> Result<(), RegisterError> {
        self.lock_refresh_calls
            .borrow_mut()
            .push(repo_root.to_path_buf());
        self.events
            .borrow_mut()
            .push("cargo-lock-refresh".to_owned());
        if self.fail_lock_refresh {
            return Err(RegisterError::CargoLockRefreshFailed(
                "fake cargo metadata failure".to_owned(),
            ));
        }
        Ok(())
    }

    fn regenerate(&self, repo_root: &Path) -> Result<Vec<String>, RegisterError> {
        self.regen_calls.borrow_mut().push(repo_root.to_path_buf());
        self.events.borrow_mut().push("faces-regenerate".to_owned());
        if self.fail {
            return Err(RegisterError::RegenFailed("fake regen failure".to_owned()));
        }
        let mut written = self.faces_written.clone();
        written.sort();
        Ok(written)
    }

    fn verify_drift(&self, repo_root: &Path) -> Result<(), RegisterError> {
        self.verify_calls.borrow_mut().push(repo_root.to_path_buf());
        self.events.borrow_mut().push("faces-verify".to_owned());
        match &self.drift_face {
            Some(face) => Err(RegisterError::DriftDetected { face: face.clone() }),
            None => Ok(()),
        }
    }

    fn gate_input_face(&self, repo_root: &Path, face: &str) -> Result<Value, RegisterError> {
        self.gate_face_calls
            .borrow_mut()
            .push((repo_root.to_path_buf(), face.to_owned()));
        Ok(self
            .gate_faces
            .get(face)
            .cloned()
            .unwrap_or_else(|| serde_json::json!({ "rows": [] })))
    }
}

/// Dispatch a single [`Edit`] to its writer/bridge. Returns `Ok(Some(applied))` for a mutating
/// edit, `Ok(None)` for the recorded-only `FacesSettle`, or `Err` (fail-closed) on any failure.
fn dispatch_edit(
    repo_root: &Path,
    cfg: &OyaCiConfig,
    tracked_paths: &[String],
    edit: &Edit,
) -> Result<Option<AppliedEdit>, RegisterError> {
    match edit {
        Edit::OwnersWrite { dir, owner } => {
            // Producer bridge: `<dir>=<owner>`. fix_owners writes <dir>/OWNERS and self-validates.
            let spec = format!("{dir}={owner}");
            fix_owners(repo_root, cfg, tracked_paths, &spec)?;
            // `changed: true` is HONEST here, not an assumption: `fix_owners` REFUSES (returns
            // `ProducerError::Refused`, mapped to `Err` above) when `<dir>/OWNERS` already exists —
            // it "only seeds missing registrations". So an `Ok` return ALWAYS means a new OWNERS
            // file was written: there is no succeed-as-no-op path. (The kernel also only emits this
            // edit when `owners_present == false`, so a re-run plans no OwnersWrite at all.)
            Ok(Some(AppliedEdit {
                kind: AppliedEditKind::OwnersWrite,
                path: format!("{dir}/{}", cfg.owners.file_name),
                changed: true,
            }))
        }
        Edit::WorkspaceMemberGlob { dir } => {
            // Verifier writer: Ok(false)=covered no-op; WorkspaceMemberUncovered=needs a human glob.
            let changed = workspace_member_glob::apply(repo_root, dir)?;
            Ok(Some(AppliedEdit {
                kind: AppliedEditKind::WorkspaceMemberGlob,
                path: workspace_member_glob::MANIFEST_PATH.to_owned(),
                changed,
            }))
        }
        Edit::CapabilityMapping { dir, capability } => {
            let changed = capability_mapping::apply(repo_root, dir, capability)?;
            Ok(Some(AppliedEdit {
                kind: AppliedEditKind::CapabilityMapping,
                path: capability_mapping::REGISTRY_PATH.to_owned(),
                changed,
            }))
        }
        Edit::AdrGovernedPathAppend { adr, paths } => {
            let adr_rel = resolve_adr_path(repo_root, cfg, adr)?;
            let changed = adr_governed_paths::apply(repo_root, &adr_rel, paths)?;
            Ok(Some(AppliedEdit {
                kind: AppliedEditKind::AdrGovernedPathAppend,
                path: adr_rel,
                changed,
            }))
        }
        Edit::CatalogYaml { dir, plane, slo } => {
            // Judgment call 5b: the kernel emits CatalogYaml ONLY when the request carries a catalog
            // spec. No catalog file ⇒ no slo-coverage / catalog-liveness row for this crate ⇒ no
            // gate finding (those gates only evaluate rows that exist), so a catalog-less crate is
            // correctly silent rather than RED.
            let changed = catalog_yaml::apply(repo_root, dir, plane, slo)?;
            Ok(Some(AppliedEdit {
                kind: AppliedEditKind::CatalogYaml,
                path: catalog_yaml::catalog_path(dir),
                changed,
            }))
        }
        Edit::ReachabilityEntry { path } => {
            // Producer bridge: `<prefix>=<anchor>`. The anchor names WHY the tree is reached; for a
            // kernel-emitted non-crate governed path the anchor is the owning registration context.
            let anchor =
                format!("ADR-0568 born-accounting: register-crate reachability entry for {path}");
            let spec = format!("{path}={anchor}");
            fix_reachability(repo_root, cfg, tracked_paths, &spec)?;
            Ok(Some(AppliedEdit {
                kind: AppliedEditKind::ReachabilityEntry,
                path: path.clone(),
                changed: true,
            }))
        }
        // FacesSettle is recorded (requires_faces_settle), never executed here: materialize needs
        // buck2/shell — that is slice 3c's RegenPort.
        Edit::FacesSettle => Ok(None),
    }
}

// ───────────────────────────── loaders (repo I/O) ─────────────────────────────

/// Load the repo's `oya-ci.toml` policy (OYA-CI-CONFORMANCE-FLOOR-PLAN §3.3), mirroring the
/// producer's loader (`oya-cloud-ci-accounting-registry-app::main::load_config`) so the orchestrator
/// honours the SAME profile/paths the producer bridges do. The file is parsed by the CLOSED-schema
/// loader; a malformed file / unknown key is a HARD error (fail LOUD, never silently revert to the
/// oyatie default). Only file-NotFound falls back to the compiled-in bundled default (zero-config =
/// today's first-party posture). This is the universality fix: a non-oyatie repo's custom
/// `reachability.registry` / `justification.adr_dir` / `owners.file_name` are honoured, not the
/// hardcoded oyatie profile that `OyaCiConfig::default()` returned.
///
/// # Errors
/// [`RegisterError::Io`] on a malformed `oya-ci.toml` or a non-NotFound read failure.
fn load_config(repo_root: &Path) -> Result<OyaCiConfig, RegisterError> {
    let path = repo_root.join("oya-ci.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => OyaCiConfig::from_toml_str(&text)
            .map_err(|e| RegisterError::Io(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(OyaCiConfig::bundled_default()),
        Err(e) => Err(RegisterError::Io(format!("{}: {e}", path.display()))),
    }
}

/// Load the closed [`CapabilitySet`] from `governance/capability-registry.json`: the set of capability
/// slugs a human may pass as `req.capability`. Two slug classes, both bare (no `capability:`/`meta:`
/// label prefix — the kernel compares `req.capability` by exact string):
///   1. The WRITER-APPLIABLE slugs — every `membership_lint_coverage.absorbs_current_crate_globs`
///      group's key (`meta_dir` if present, else `capability`). These are the only slugs the writer
///      `capability_mapping::compute` can actually UPSERT a glob into, so a kernel-accepted slug in
///      this class is one a `CapabilityMapping` edit can apply.
///   2. The EXPRESSIBLE META homes — `app_products.meta_dir` (default `app/`) + each
///      `meta_directory_absorbs[].meta_dir` (`kernel/`, `os/`). A crate that genuinely belongs under
///      one of these is DIR-PREFIX mapped (so it is already-mapped and never drives a writer call),
///      but the human must still be able to NAME its home — without these slugs the kernel would
///      reject the only correct choice and force a wrong group. Including them gives a genuinely-
///      unmapped meta crate a valid capability choice.
///
/// # Errors
/// [`RegisterError::Io`] if the registry is unreadable; the parse is fail-closed (a malformed
/// registry yields an empty set, which makes the kernel reject every capability).
fn load_capability_set(repo_root: &Path) -> Result<CapabilitySet, RegisterError> {
    let abs = repo_root.join(CAPABILITY_REGISTRY_PATH);
    let text = std::fs::read_to_string(&abs)
        .map_err(|e| RegisterError::Io(format!("read {CAPABILITY_REGISTRY_PATH}: {e}")))?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|e| RegisterError::Io(format!("parse {CAPABILITY_REGISTRY_PATH}: {e}")))?;
    let mut set = CapabilitySet::new();
    let coverage = root.get("membership_lint_coverage");

    // 1. The writer-appliable slugs (absorbs_current_crate_globs group keys).
    if let Some(groups) = coverage
        .and_then(|m| m.get("absorbs_current_crate_globs"))
        .and_then(Value::as_array)
    {
        for group in groups {
            if let Some(slug) = group_slug(group) {
                set.insert(slug.to_owned());
            }
        }
    }

    // 2. The expressible meta homes: app_products → `app/`, meta_directory_absorbs → `kernel/`/`os/`.
    if let Some(app) = coverage.and_then(|m| m.get("app_products")) {
        let meta = app
            .get("meta_dir")
            .and_then(Value::as_str)
            .unwrap_or("app/");
        set.insert(meta.to_owned());
    }
    if let Some(entries) = coverage
        .and_then(|m| m.get("meta_directory_absorbs"))
        .and_then(Value::as_array)
    {
        for entry in entries {
            if let Some(meta) = entry.get("meta_dir").and_then(Value::as_str) {
                set.insert(meta.to_owned());
            }
        }
    }

    Ok(set)
}

/// The slug a capability-registry membership group is keyed by: its `meta_dir` if present, else
/// its `capability`. Matches the writer's `capability_mapping::group_slug`.
fn group_slug(group: &Value) -> Option<&str> {
    group
        .get("meta_dir")
        .and_then(Value::as_str)
        .or_else(|| group.get("capability").and_then(Value::as_str))
}

/// Parse the live registry into the membership gate's [`Mapping`] (the SAME parse the gate uses —
/// reused, not reimplemented). A malformed registry is a fail-closed [`RegisterError::Io`] (the
/// gate's own `MEM-POLICY-MALFORMED` message is carried through).
fn load_mapping(repo_root: &Path) -> Result<Mapping, RegisterError> {
    let abs = repo_root.join(CAPABILITY_REGISTRY_PATH);
    let text = std::fs::read_to_string(&abs)
        .map_err(|e| RegisterError::Io(format!("read {CAPABILITY_REGISTRY_PATH}: {e}")))?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|e| RegisterError::Io(format!("parse {CAPABILITY_REGISTRY_PATH}: {e}")))?;
    parse_mapping(&root).map_err(|e| RegisterError::Io(format!("{CAPABILITY_REGISTRY_PATH}: {e}")))
}

/// True iff `crate_dir` already maps to at least one capability/meta home, computed by REUSING the
/// membership-lint gate's `homes_for`/`parse_mapping` (NOT a divergent reimplementation). This sees
/// every home source the gate enforces — `capabilities[].absorbs_current_dirs` dir-prefixes,
/// `app_products.current_dirs` (→ `meta:app/`), `meta_directory_absorbs[].current_dirs`
/// (→ `meta:kernel/`/`meta:os/`), and the `*`-suffix `absorbs_current_crate_globs` membership — so
/// `capability_already_mapped(dir) == (homes_for(dir).len() >= 1)`. Sharing the gate's logic is the
/// drift fix: the orchestrator never emits a spurious `CapabilityMapping` edit for a crate the gate
/// considers mapped (under `oya/<app product>`, `cloud/cloud-kernel`, `os`, or a glob),
/// which would otherwise DOUBLE-MAP it and turn the membership gate RED.
fn capability_already_mapped(repo_root: &Path, crate_dir: &str) -> Result<bool, RegisterError> {
    let mapping = load_mapping(repo_root)?;
    let normalized = crate_dir.trim_end_matches('/');
    Ok(!homes_for(&mapping, normalized).is_empty())
}

/// Build the [`CurrentState`] snapshot for `req` by reading the live SSOTs. The plan is the diff
/// of the request against this snapshot, so each loader answers "does this SSOT already carry the
/// registration?".
///
/// `faces_settled` is set to `false` (conservative): the orchestrator never reads the materialized
/// faces' settle status (that needs buck2/shell — slice 3c), and the kernel only consults
/// `faces_settled` to suppress a `FacesSettle` push when NOTHING else changed — but `FacesSettle`
/// is already gated on `changed`, so a `false` here is sound (a no-op plan still emits no settle).
///
/// # Errors
/// [`RegisterError::Io`] on a loader read failure.
fn load_current_state(
    repo_root: &Path,
    cfg: &OyaCiConfig,
    req: &RegisterCrateRequest,
    tracked_paths: &[String],
) -> Result<CurrentState, RegisterError> {
    let dir = req.crate_dir.trim_end_matches('/');

    // OWNERS: present iff a tracked OWNERS file resolves the crate dir to a valid owner. We use the
    // producer's content-aware resolver over the tracked universe ∪ the crate's own paths.
    let owners_present = owners_resolve_dir(repo_root, cfg, tracked_paths, dir);

    // Member glob: covered iff an existing root [workspace].members glob covers the dir.
    let member_glob_covers = member_glob_covers_dir(repo_root, dir)?;

    // Capability: mapped iff the dir is in a crate-glob group OR absorbed by a capability dir.
    let capability_mapped = capability_already_mapped(repo_root, dir)?;

    // ADR governed paths: the verbatim paths already enumerated under the owning ADR's block.
    let adr_governed_paths = load_adr_governed_paths(repo_root, cfg, &req.owning_adr)?;

    // Catalog: present iff registry/catalog/<leaf>.yaml exists (only meaningful when one is required).
    let catalog_present = repo_root.join(catalog_yaml::catalog_path(dir)).is_file();

    // Reachability: the non-crate paths already carried as reachability-registry entries.
    let reachability_entries = load_reachability_entries(repo_root, cfg)?;

    Ok(CurrentState {
        owners_present,
        member_glob_covers,
        capability_mapped,
        adr_governed_paths,
        catalog_present,
        reachability_entries,
        faces_settled: false,
    })
}

/// True iff a schema-valid tracked OWNERS file ownership-resolves at least one path under `dir/`
/// to `OWNERS:<dir>` (the producer's resolution semantics — existence AND valid content). The
/// resolution runs over `tracked_paths` ALONE (the `git ls-files` universe): a just-seeded
/// registration reads as present on a re-run only once its OWNERS file is staged into that tracked
/// universe (the caller `git add`s it between runs); nothing is synthetically added here.
fn owners_resolve_dir(
    repo_root: &Path,
    cfg: &OyaCiConfig,
    tracked_paths: &[String],
    dir: &str,
) -> bool {
    let resolution = resolve_owners(repo_root, tracked_paths, cfg);
    let want = format!("OWNERS:{dir}");
    resolution
        .by_path
        .iter()
        .any(|(path, resolved)| path.starts_with(&format!("{dir}/")) && resolved == &want)
}

/// True iff an existing root `[workspace].members` glob covers `dir`. Resolved purely through the
/// member-glob writer's compute (the single source of glob semantics): `Ok` = covered,
/// `WorkspaceMemberUncovered` = not covered. A manifest-parse failure is surfaced as IO.
fn member_glob_covers_dir(repo_root: &Path, dir: &str) -> Result<bool, RegisterError> {
    let abs = repo_root.join(workspace_member_glob::MANIFEST_PATH);
    let current = std::fs::read_to_string(&abs).map_err(|e| {
        RegisterError::Io(format!(
            "read {}: {e}",
            workspace_member_glob::MANIFEST_PATH
        ))
    })?;
    match workspace_member_glob::compute(&current, dir) {
        Ok(_) => Ok(true),
        Err(WriterError::WorkspaceMemberUncovered(_)) => Ok(false),
        Err(other) => Err(RegisterError::Writer(other)),
    }
}

/// The verbatim governed paths already enumerated under `adr`'s `## Governed surfaces` block, or
/// an empty set when the ADR file or block is absent. Resolves the ADR id to its
/// `docs/decisions/<id>-*.md` path; a missing ADR file yields an empty set (the kernel will plan
/// the append, and the dispatch's [`resolve_adr_path`] then fails closed if it is still missing).
fn load_adr_governed_paths(
    repo_root: &Path,
    cfg: &OyaCiConfig,
    adr: &str,
) -> Result<BTreeSet<String>, RegisterError> {
    let adr_rel = match find_adr_path(repo_root, cfg, adr) {
        Some(p) => p,
        None => return Ok(BTreeSet::new()),
    };
    let abs = repo_root.join(&adr_rel);
    let text = match std::fs::read_to_string(&abs) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(RegisterError::Io(format!("read {adr_rel}: {e}"))),
    };
    Ok(parse_governed_surfaces(&text))
}

/// Parse the verbatim paths inside an ADR markdown's `## Governed surfaces` fenced block. Each
/// non-empty trimmed line inside the first fence after the heading is one path. Mirrors the
/// writer's `existing_block_paths` extraction so the snapshot agrees with what the writer reads.
fn parse_governed_surfaces(text: &str) -> BTreeSet<String> {
    const HEADING: &str = "## Governed surfaces";
    const FENCE: &str = "```";
    let mut paths = BTreeSet::new();
    let mut in_section = false;
    let mut in_fence = false;
    for line in text.lines() {
        if !in_section {
            if line.trim_end() == HEADING {
                in_section = true;
            }
            continue;
        }
        if !in_fence {
            // A new markdown heading ends the section before any fence.
            if line.trim_start().starts_with('#') {
                break;
            }
            if line.trim_start().starts_with(FENCE) {
                in_fence = true;
            }
            continue;
        }
        if line.trim() == FENCE {
            break;
        }
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            paths.insert(trimmed.to_owned());
        }
    }
    paths
}

/// The non-crate paths already registered in the reachability registry (the `prefix` of each
/// entry). Loaded via the producer's fail-loud `load_reachability_registry`.
///
/// # Errors
/// [`RegisterError::Producer`] if the registry is malformed (fail-loud — never a silent empty).
fn load_reachability_entries(
    repo_root: &Path,
    cfg: &OyaCiConfig,
) -> Result<BTreeSet<String>, RegisterError> {
    let registry_abs = repo_root.join(cfg.reachability.registry.as_str());
    let entries = load_reachability_registry(&registry_abs)?;
    Ok(entries.into_iter().map(|e| e.prefix).collect())
}

/// Resolve an ADR id to its `docs/decisions/<id>-*.md` repo-relative path, failing CLOSED if no
/// such file exists (the ADR must exist before its governed surfaces can be appended).
///
/// # Errors
/// [`RegisterError::AdrFileNotFound`] if no `<id>-*.md` file is in `cfg.justification.adr_dir`.
fn resolve_adr_path(
    repo_root: &Path,
    cfg: &OyaCiConfig,
    adr: &str,
) -> Result<String, RegisterError> {
    find_adr_path(repo_root, cfg, adr).ok_or_else(|| RegisterError::AdrFileNotFound {
        adr: adr.to_owned(),
        adr_dir: cfg.justification.adr_dir.clone(),
    })
}

/// The repo-relative `<corpus>/<id>-*.md` path whose filename encodes `adr` (via the
/// producer's `adr_id_from_filename`), or `None` if neither the live ADR dir nor the
/// historical archive has such a file. Live decisions win over archive on id collision.
fn find_adr_path(repo_root: &Path, cfg: &OyaCiConfig, adr: &str) -> Option<String> {
    let mut dirs = vec![cfg.justification.adr_dir.as_str()];
    if cfg.justification.adr_dir != "docs/adr-archive" {
        dirs.push("docs/adr-archive");
    }
    for adr_dir in dirs {
        let abs_dir = repo_root.join(adr_dir);
        let Ok(entries) = std::fs::read_dir(&abs_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if adr_id_from_filename(&name).as_deref() == Some(adr) {
                return Some(format!("{adr_dir}/{name}"));
            }
        }
    }
    None
}

/// The `git ls-files` tracked-paths universe (sorted+deduped), repo-relative — the input the
/// producer bridges' self-validation counts coverage over. The orchestrator IS the I/O layer, so
/// it may shell to git here (the kernel never does).
///
/// # Errors
/// [`RegisterError::Io`] if `git ls-files` cannot run or returns non-zero / non-UTF-8 output.
fn list_tracked_paths(repo_root: &Path) -> Result<Vec<String>, RegisterError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("ls-files")
        .output()
        .map_err(|e| RegisterError::Io(format!("git ls-files: {e}")))?;
    if !output.status.success() {
        return Err(RegisterError::Io(format!(
            "git ls-files exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|e| RegisterError::Io(format!("git ls-files output not UTF-8: {e}")))?;
    let mut paths: Vec<String> = text.lines().map(str::to_owned).collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

#[cfg(test)]
mod tests;
