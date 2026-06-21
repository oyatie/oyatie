//! # oya-cloud-ci-register-crate-app (ADR-0568, G011 born-accounting slice 3b)
//!
//! The ORCHESTRATOR half of `register_crate`: the integration layer that wires the pure
//! [`oya-crate-registrar-kernel`](oya_crate_registrar_kernel)'s typed plan to the on-disk
//! writers ([`oya-crate-registrar-app`](oya_crate_registrar_app)) and the producer's
//! registration bridges ([`oya-cloud-ci-accounting-registry-app`](oya_cloud_ci_accounting_registry_app)).
//!
//! ## Dependency direction (load-bearing — ADR-0131/ADR-0512)
//! This crate lives under `cloud/cloud-ci/gates/` precisely so it MAY depend SAME-LAYER on the
//! cloud-ci producer (`fix_owners`/`fix_reachability`/`allocate_next_adr_id`) and DOWNWARD on the
//! `libs/` kernel + writers. A `libs/` crate may NOT depend on `cloud/cloud-ci/gates/` (forbidden
//! layer inversion), so the orchestration — which needs both halves — cannot live in `libs/`.
//!
//! ## What it does (the integration the pure kernel can't)
//! 1. LOADERS — read the live repo SSOTs into the kernel's input types
//!    ([`CurrentState`](oya_crate_registrar_kernel::CurrentState),
//!    [`CapabilitySet`](oya_crate_registrar_kernel::CapabilitySet)).
//! 2. PLAN — call [`plan_register_crate`](oya_crate_registrar_kernel::plan_register_crate) to get
//!    the ordered, typed [`Edit`](oya_crate_registrar_kernel::Edit) diff (fail-closed on a
//!    [`ValidationError`](oya_crate_registrar_kernel::ValidationError)).
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
//! | `FacesSettle`           | NOT applied here — set `requires_faces_settle = true` |
//!
//! `FacesSettle` deliberately does NOT run materialize: materialize needs buck2/shell (the
//! RegenPort is slice 3c). The orchestrator records the obligation; the caller (or 3c) settles.
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

use oya_cloud_ci_accounting_registry_app::{
    ProducerError, adr_id_from_filename, fix_owners, fix_reachability, load_reachability_registry,
    resolve_owners,
};
use oya_crate_registrar_app::{WriterError, adr_governed_paths, capability_mapping, catalog_yaml,
    workspace_member_glob};
use oya_crate_registrar_kernel::{
    CapabilitySet, CurrentState, Edit, RegisterCrateRequest, RegistrationPlan, ValidationError,
    plan_register_crate,
};
use oya_ci_config_kernel::OyaCiConfig;
use serde_json::Value;

/// The repo-relative closed capability registry — the SSOT for both the closed
/// [`CapabilitySet`] (group slugs) and the existing crate-glob membership.
const CAPABILITY_REGISTRY_PATH: &str = "specs/capability-registry.json";

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
    /// changed and the materialized faces must be re-settled. The orchestrator never runs
    /// materialize (no buck2/shell here); it records the obligation for the caller / slice 3c.
    pub requires_faces_settle: bool,
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
        }
    }
}

impl std::error::Error for RegisterError {}

impl From<WriterError> for RegisterError {
    fn from(e: WriterError) -> Self {
        // The member-glob uncovered case is surfaced as its own fail-closed variant (the human
        // must add a glob) rather than an opaque writer error.
        match e {
            WriterError::WorkspaceMemberUncovered(dir) => RegisterError::MemberGlobUncovered { dir },
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
pub fn register_crate(repo_root: &Path, req: &RegisterCrateRequest) -> Result<Outcome, RegisterError> {
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
    let cfg = OyaCiConfig::default();

    // --- LOADERS: live SSOT snapshot the pure kernel consumes ---
    let capabilities = match load_capability_set(repo_root) {
        Ok(c) => c,
        Err(e) => return RegisterOutcome::Failed { error: e, applied: Vec::new() },
    };
    let tracked_paths = match list_tracked_paths(repo_root) {
        Ok(t) => t,
        Err(e) => return RegisterOutcome::Failed { error: e, applied: Vec::new() },
    };
    let current = match load_current_state(repo_root, &cfg, req, &tracked_paths) {
        Ok(c) => c,
        Err(e) => return RegisterOutcome::Failed { error: e, applied: Vec::new() },
    };

    // --- PLAN: the pure kernel computes the ordered upsert diff ---
    let plan: RegistrationPlan = match plan_register_crate(req, &current, &capabilities) {
        Ok(p) => p,
        Err(e) => {
            return RegisterOutcome::Failed { error: RegisterError::Plan(e), applied: Vec::new() };
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

    RegisterOutcome::Done(Outcome { applied, requires_faces_settle })
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
            let anchor = format!(
                "ADR-0568 born-accounting: register-crate reachability entry for {path}"
            );
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

/// Load the closed [`CapabilitySet`] from `specs/capability-registry.json`: the slug of every
/// `membership_lint_coverage.absorbs_current_crate_globs` group (its `meta_dir` if present, else
/// its `capability`). This is the SAME closed set the writer's `capability_mapping` validates
/// against, so a capability the kernel accepts is one a writer can apply.
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
    if let Some(groups) = root
        .get("membership_lint_coverage")
        .and_then(|m| m.get("absorbs_current_crate_globs"))
        .and_then(Value::as_array)
    {
        for group in groups {
            if let Some(slug) = group_slug(group) {
                set.insert(slug.to_owned());
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

/// True iff `crate_dir` is already listed in ANY `absorbs_current_crate_globs` group's `globs`,
/// OR is absorbed by a capability's `absorbs_current_dirs` path prefix (e.g. a crate under
/// `cloud/cloud-ci/` is absorbed by the `ci` capability's `cloud/cloud-ci` dir entry — exactly
/// the producer's own situation). Either makes the crate already capability-mapped.
fn capability_already_mapped(repo_root: &Path, crate_dir: &str) -> Result<bool, RegisterError> {
    let abs = repo_root.join(CAPABILITY_REGISTRY_PATH);
    let text = std::fs::read_to_string(&abs)
        .map_err(|e| RegisterError::Io(format!("read {CAPABILITY_REGISTRY_PATH}: {e}")))?;
    let root: Value = serde_json::from_str(&text)
        .map_err(|e| RegisterError::Io(format!("parse {CAPABILITY_REGISTRY_PATH}: {e}")))?;
    let normalized = crate_dir.trim_end_matches('/');

    // 1. An explicit crate-glob membership entry (exact dir match in any group's globs).
    if let Some(groups) = root
        .get("membership_lint_coverage")
        .and_then(|m| m.get("absorbs_current_crate_globs"))
        .and_then(Value::as_array)
    {
        for group in groups {
            if let Some(globs) = group.get("globs").and_then(Value::as_array)
                && globs.iter().any(|g| g.as_str() == Some(normalized))
            {
                return Ok(true);
            }
        }
    }

    // 2. A capability dir-prefix absorption (the path is the namespace). A crate under a
    //    capability's `absorbs_current_dirs` entry is mapped by its dir, not a crate-glob.
    if let Some(caps) = root.get("capabilities").and_then(Value::as_array) {
        for cap in caps {
            if let Some(dirs) = cap.get("absorbs_current_dirs").and_then(Value::as_array) {
                for dir in dirs.iter().filter_map(Value::as_str) {
                    let dir = dir.trim_end_matches('/');
                    if normalized == dir || normalized.starts_with(&format!("{dir}/")) {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
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
/// crate's own OWNERS file (if already tracked) is included in the universe so a just-seeded
/// registration reads as present on a re-run.
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
        RegisterError::Io(format!("read {}: {e}", workspace_member_glob::MANIFEST_PATH))
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

/// The repo-relative `docs/decisions/<id>-*.md` path whose filename encodes `adr` (via the
/// producer's `adr_id_from_filename`), or `None` if the ADR corpus has no such file.
fn find_adr_path(repo_root: &Path, cfg: &OyaCiConfig, adr: &str) -> Option<String> {
    let adr_dir = cfg.justification.adr_dir.as_str();
    let abs_dir = repo_root.join(adr_dir);
    let entries = std::fs::read_dir(&abs_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if adr_id_from_filename(&name).as_deref() == Some(adr) {
            return Some(format!("{adr_dir}/{name}"));
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
