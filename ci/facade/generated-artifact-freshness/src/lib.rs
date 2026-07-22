#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use ci_cross_artifact_agreement::{MASTERPLAN_MD_PATH, derive_masterplan_md_projection};
use ci_planning_projection::render_board_sync_projection;
use oya_workspace_members_kernel::resolve_member_dirs;

mod rust_toolchain_drift;
pub use rust_toolchain_drift::{evaluate_rust_toolchain_drift, read_pinned_rust_toolchain};

pub const LOCK_REMEDIATION_COMMAND: &str = "cargo metadata >/dev/null";
pub const FACE_REMEDIATION_COMMAND: &str = "buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .";
const RETIREMENT_CONTROL_PLANE_PATH: &str = "registry/history-only-retirement-control-plane.json";
const RETIREMENT_FACTS_PATH: &str =
    "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json";
pub const FACE_SETTLE_PROTOCOL: &str = "commit content changes first; faces regenerate from the TRACKED TREE STATE (ADR-0552: committed faces carry no history-derived data, so commit ids never enter them); never mix content and regenerated faces in one commit; then run the materialize command; commit only PR-owned generated face diffs; controller-owned generated faces are materialized by cloud-ci/integration controllers, not contributor PRs; then run oya-cloud-ci-face-settle --verify as the LAST step before EVERY push";
pub const FACE_VERIFY_REMEDIATION_COMMAND: &str = "oya-cloud-ci-face-settle --settle --commit";
pub const FACE_SETTLE_COMMIT_COMMAND: &str =
    "git commit -S -m \"chore: settle generated cloud-ci faces\"";
const FACE_SETTLE_COMMIT_MESSAGE: &str = "chore: settle generated cloud-ci faces";
const FACES_DIR: &str = "ci/facade/artifact-inventory-registry";
const SCM_FACTS_FACE: &str = "scm-facts.generated.json";
const ADR_CENSUS_PARENT_RECEIPT_FACE: &str = "adr-census-parent-receipt.generated.json";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS: &str = ".claude/settings.json";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS: &str = ".codex/hooks.json";
const ENFORCEMENT_LIVENESS_HOOKS_DIR: &str = "tools/hooks";
/// The generated-artifact control-plane manifest. Faces whose `materialization_mode` is
/// non-PR-owned are materialized by cloud-ci/controllers, not byte-compared against contributor
/// branch copies.
const CONTROL_PLANE_MANIFEST: &str = "registry/generated-artifact-control-plane.json";
/// Materialization mode marking a declared generated artifact as intentionally de-committed.
const NOT_TRACKED_IN_GIT_MODE: &str = "not-tracked-in-git";
/// Materialization mode marking a declared generated artifact as an integration-branch baseline:
/// committed on the protected branch for merge-base consumers, but not a contributor PR byte-diff.
const MAIN_BRANCH_MATERIALIZED_MODE: &str = "main-branch-materialized";
const MASTERPLAN_PROJECTION_FACE: &str = "masterplan.generated.json";
const MASTERPLAN_PROJECTION_PATH: &str = "docs/machine-readable/masterplan.generated.json";
const BOARD_SYNC_PROJECTION_FACE: &str = "board-sync.generated.json";
const BOARD_SYNC_PROJECTION_PATH: &str = "docs/machine-readable/board-sync.generated.json";
const MASTERPLAN_SOURCE_PATH: &str = "specs/masterplan.json";
const ARCHITECTURE_PRODUCT_GRAPH_FACE: &str = "product-graph.html";
const ARCHITECTURE_PRODUCT_GRAPH_PATH: &str = "docs/architecture/product-graph.html";
const ACTIVE_ARTIFACT_CONTRACT_GRAPH_FACE: &str = "active-artifact-contract-edges.json";
const ACTIVE_ARTIFACT_CONTRACT_GRAPH_PATH: &str =
    "registry/graph/active-artifact-contract-edges.json";
/// PR-owned / face-settle generated paths. Controller-owned generated artifacts that must be
/// materialized on protected branches, such as `product-graph.html`, intentionally stay out of
/// this list so contributor PRs do not acquire a new generated merge surface.
const GENERATED_FACE_PATHS: [&str; 7] = [
    "ci/facade/artifact-inventory-registry/scm-facts.generated.json",
    "ci/facade/artifact-inventory-registry/accounting-registry.generated.json",
    "ci/facade/artifact-inventory-registry/ttl-policy.generated.json",
    "ci/facade/artifact-inventory-registry/decision-crosswalk.generated.json",
    "ci/facade/artifact-inventory-registry/enforcement-inventory.generated.json",
    "ci/facade/artifact-inventory-registry/enforcement-liveness.generated.json",
    "ci/facade/artifact-inventory-registry/gate-baseline.generated.json",
];
/// Controller-owned generated artifacts whose freshness is proven by regeneration/determinism,
/// but whose byte diffs are not staged by `oya-cloud-ci-face-settle` in contributor PRs.
const CONTROLLER_MATERIALIZED_ARTIFACT_PATHS: [&str; 5] = [
    "ci/facade/artifact-inventory-registry/adr-census-parent-receipt.generated.json",
    MASTERPLAN_PROJECTION_PATH,
    BOARD_SYNC_PROJECTION_PATH,
    ARCHITECTURE_PRODUCT_GRAPH_PATH,
    ACTIVE_ARTIFACT_CONTRACT_GRAPH_PATH,
];
const EMITTER_TARGET: &str = "//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot";
const PRODUCER_TARGET: &str =
    "//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin";
const CODEMOD_TARGET: &str = "//tools/oya-reorg-codemod-app:oya-reorg-codemod";
const ARCHITECTURE_GRAPH_GENERATOR_TARGET: &str =
    "//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator";
const MASTERPLAN_GENERATOR_TARGET: &str = "//marketplace/facade/dev-cli:oya";
const ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_TARGET: &str = "//.claude:settings-json";
const ENFORCEMENT_LIVENESS_CODEX_HOOKS_TARGET: &str = "//.codex:hooks-json";
const ENFORCEMENT_LIVENESS_HOOKS_DIR_TARGET: &str = "//tools/hooks:top-level-hook-scripts";
const MOVE_MANIFEST_FACE: &str = "specs/reorg/move-manifest.generated.json";
/// De-committed face: merge-base CONTENT of every `normal-source-merge` hand-curated-ratchet
/// artifact, keyed by declared path. Materialized from the SAME merge-base source worktree this
/// file already checks out for frozen-baseline regeneration (a plain filesystem read — no
/// additional git call), so the generated-output-diff-policy gate can verify a hand-curated-
/// ratchet baseline's plain-modify content diff (shrink-only / move-plan-backed substitution)
/// without itself calling git (gate production code must stay hermetic).
const RATCHET_MERGE_BASE_FACE: &str =
    "ci/facade/generated-artifact-policy/ratchet-merge-base.generated.json";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingCode {
    LockMissingMemberPackage,
    LockStaleMemberVersion,
    LockOrphanPathPackage,
    GeneratedFaceStale,
    RustToolchainDrift,
}

impl FindingCode {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingCode::LockMissingMemberPackage => "lock_missing_member_package",
            FindingCode::LockStaleMemberVersion => "lock_stale_member_version",
            FindingCode::LockOrphanPathPackage => "lock_orphan_path_package",
            FindingCode::GeneratedFaceStale => "generated_face_stale",
            FindingCode::RustToolchainDrift => "rust_toolchain_drift",
        }
    }
}

impl Display for FindingCode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: FindingCode,
    pub key: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    pub findings: Vec<Finding>,
}

impl CheckReport {
    pub fn is_green(&self) -> bool {
        self.findings.is_empty()
    }
}

impl Finding {
    fn new(code: FindingCode, key: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code,
            key: key.into(),
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceSettleMode {
    Check,
    Verify,
    Settle,
    SettleAndCommit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceSettleReport {
    pub message: String,
    pub stale_faces: Vec<String>,
    /// Rendered non-face freshness-gate findings (Cargo.lock staleness). Populated by
    /// the Verify mode only, which runs the gate's FULL check; always empty for the
    /// face-scoped Check/Settle modes.
    pub lock_findings: Vec<String>,
    pub staged_faces: Vec<String>,
    pub committed: bool,
}

impl FaceSettleReport {
    pub fn is_success(&self) -> bool {
        self.stale_faces.is_empty() && self.lock_findings.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceSettleArgs {
    pub repo_root: PathBuf,
    pub mode: FaceSettleMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializeGeneratedFacesArgs {
    pub repo_root: PathBuf,
    pub retirement: Option<RetirementMaterializeArgs>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetirementMaterializeArgs {
    pub control_plane_path: String,
    pub facts_out: PathBuf,
    pub protected_base_commit: String,
    pub candidate_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemberPackage {
    pub member_path: String,
    pub name: String,
    pub version: String,
}

impl MemberPackage {
    pub fn new(
        member_path: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            member_path: member_path.into(),
            name: name.into(),
            version: version.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LockPackage {
    pub name: String,
    pub version: String,
    pub is_path_package: bool,
}

impl LockPackage {
    pub fn path(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            is_path_package: true,
        }
    }

    pub fn external(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            is_path_package: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshnessError {
    message: String,
}

impl FreshnessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for FreshnessError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for FreshnessError {}

pub fn evaluate_lock_freshness(
    members: &[MemberPackage],
    lock_packages: &[LockPackage],
) -> Vec<Finding> {
    let mut findings = BTreeSet::new();
    let members_by_name: BTreeMap<&str, &MemberPackage> = members
        .iter()
        .map(|member| (member.name.as_str(), member))
        .collect();
    let path_lock_packages: BTreeMap<&str, &LockPackage> = lock_packages
        .iter()
        .filter(|package| package.is_path_package)
        .map(|package| (package.name.as_str(), package))
        .collect();

    for member in members {
        match path_lock_packages.get(member.name.as_str()) {
            None => {
                findings.insert(Finding::new(
                    FindingCode::LockMissingMemberPackage,
                    &member.member_path,
                    format!(
                        "workspace member `{}` ({}) is absent from Cargo.lock; remediation: {LOCK_REMEDIATION_COMMAND}",
                        member.name, member.member_path
                    ),
                ));
            }
            Some(package) if package.version != member.version => {
                findings.insert(Finding::new(
                    FindingCode::LockStaleMemberVersion,
                    &member.member_path,
                    format!(
                        "workspace member `{}` version {} does not match Cargo.lock version {}; remediation: {LOCK_REMEDIATION_COMMAND}",
                        member.name, member.version, package.version
                    ),
                ));
            }
            Some(_) => {}
        }
    }

    for package in lock_packages
        .iter()
        .filter(|package| package.is_path_package)
    {
        if !members_by_name.contains_key(package.name.as_str()) {
            findings.insert(Finding::new(
                FindingCode::LockOrphanPathPackage,
                &package.name,
                format!(
                    "sourceless Cargo.lock package `{}` {} has no workspace member; remediation: {LOCK_REMEDIATION_COMMAND}",
                    package.name, package.version
                ),
            ));
        }
    }

    findings.into_iter().collect()
}

pub fn check_repo(repo_root: &Path) -> Result<CheckReport, FreshnessError> {
    let decommitted = read_decommitted_face_names(repo_root);
    // Determinism canary: for non-PR-owned faces there is no contributor-branch byte copy to
    // trust, so regenerate the producer faces a SECOND time (from the SAME scm-facts) and require
    // byte stability. A nondeterministic producer must hard-fail here rather than silently green.
    // When every face is PR-owned, take the single-pass path so committed-byte parity pays no
    // extra regeneration cost.
    if decommitted.is_empty() {
        let regenerated_faces = regenerate_faces_with_buck2(repo_root)?;
        return check_repo_with_regenerated_faces(repo_root, regenerated_faces);
    }
    let (first_pass, second_pass) = regenerate_faces_twice_with_buck2(repo_root)?;
    let determinism_findings = evaluate_face_determinism(&first_pass, &second_pass, &decommitted);
    let mut report = check_repo_with_regenerated_faces(repo_root, first_pass)?;
    report.findings.extend(determinism_findings);
    report.findings.sort();
    report.findings.dedup();
    Ok(report)
}

pub fn generated_face_paths() -> Vec<String> {
    GENERATED_FACE_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .collect()
}

pub fn run_face_settle_with_buck2(
    repo_root: &Path,
    mode: FaceSettleMode,
) -> Result<FaceSettleReport, FreshnessError> {
    // Assert tree preconditions BEFORE the buck2 regeneration so a dirty tree fails in
    // milliseconds instead of after a build. Verify certifies the COMMITTED tree (HEAD),
    // so it requires the FULL tracked tree clean (faces included); the other modes keep
    // the existing non-face cleanliness contract.
    match mode {
        FaceSettleMode::Verify => assert_committed_tree_clean(repo_root)?,
        FaceSettleMode::Check | FaceSettleMode::Settle | FaceSettleMode::SettleAndCommit => {
            assert_non_face_tree_clean(repo_root)?;
        }
    }
    let regenerated_faces = regenerate_faces_with_buck2(repo_root)?;
    match mode {
        FaceSettleMode::Check => check_regenerated_faces(repo_root, regenerated_faces),
        FaceSettleMode::Verify => verify_committed_tree(repo_root, regenerated_faces),
        FaceSettleMode::Settle | FaceSettleMode::SettleAndCommit => {
            settle_regenerated_faces(repo_root, regenerated_faces, mode)
        }
    }
}

pub fn materialize_generated_faces_with_buck2(repo_root: &Path) -> Result<(), FreshnessError> {
    let tools = build_materializer_tools(repo_root)?;
    let retirement = effective_retirement_materialization(repo_root, None);
    materialize_generated_faces_with_tools(&tools, repo_root, retirement.as_ref())
}

pub fn materialize_generated_faces_from_args(
    args: &MaterializeGeneratedFacesArgs,
) -> Result<(), FreshnessError> {
    let tools = build_materializer_tools(&args.repo_root)?;
    let retirement =
        effective_retirement_materialization(&args.repo_root, args.retirement.as_ref());
    materialize_generated_faces_with_tools(&tools, &args.repo_root, retirement.as_ref())
}

fn effective_retirement_materialization(
    _repo_root: &Path,
    explicit: Option<&RetirementMaterializeArgs>,
) -> Option<RetirementMaterializeArgs> {
    // The protected base is an admission fact, not a graph heuristic. In particular, HEAD^1 on a
    // normal multi-commit PR is an earlier candidate commit, not the protected branch. The
    // producer workflow supplies event-bound, verified OIDs; any caller without that transport
    // must leave retirement facts absent rather than materializing a misleading dormant receipt.
    explicit.cloned()
}

fn materialize_generated_faces_with_tools(
    tools: &MaterializerTools,
    repo_root: &Path,
    retirement: Option<&RetirementMaterializeArgs>,
) -> Result<(), FreshnessError> {
    materialize_move_manifest(tools, repo_root)?;
    let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
    emit_materialized_scm_facts(tools, repo_root, &scm_facts, retirement)?;
    emit_adr_census_parent_receipt(
        &tools.emitter,
        repo_root,
        &repo_root
            .join(FACES_DIR)
            .join(ADR_CENSUS_PARENT_RECEIPT_FACE),
    )?;
    let mut command = Command::new(&tools.producer);
    command
        .args(["--repo-root"])
        .arg(repo_root)
        .args(["--scm-facts"])
        .arg(&scm_facts);
    append_enforcement_liveness_corpus_args(&mut command, &tools.enforcement_liveness_corpus);
    command.current_dir(repo_root);
    run_status(&mut command, "materialize generated accounting faces")?;
    materialize_active_artifact_contract_graph(tools, repo_root)?;
    materialize_masterplan_projection(tools, repo_root)?;
    materialize_board_sync_projection(repo_root)?;
    materialize_masterplan_md_projection(repo_root)?;
    materialize_architecture_product_graph(tools, repo_root)
}

pub fn parse_materialize_generated_faces_args(
    args: Vec<String>,
) -> Result<MaterializeGeneratedFacesArgs, FreshnessError> {
    let mut repo_root = PathBuf::from(".");
    let mut retirement_control_plane: Option<String> = None;
    let mut retirement_facts_out: Option<PathBuf> = None;
    let mut protected_base_commit: Option<String> = None;
    let mut candidate_commit: Option<String> = None;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err(FreshnessError::new(
                        "materialize generated faces: --repo-root requires a path",
                    ));
                };
                repo_root = PathBuf::from(value);
            }
            "--retirement-control-plane" => {
                let Some(value) = iter.next() else {
                    return Err(FreshnessError::new(
                        "materialize generated faces: --retirement-control-plane requires a repo-relative path",
                    ));
                };
                retirement_control_plane = Some(value);
            }
            "--retirement-facts-out" => {
                let Some(value) = iter.next() else {
                    return Err(FreshnessError::new(
                        "materialize generated faces: --retirement-facts-out requires a path",
                    ));
                };
                retirement_facts_out = Some(PathBuf::from(value));
            }
            "--protected-base-commit" => {
                let Some(value) = iter.next() else {
                    return Err(FreshnessError::new(
                        "materialize generated faces: --protected-base-commit requires a commit oid",
                    ));
                };
                protected_base_commit = Some(value);
            }
            "--candidate-commit" => {
                let Some(value) = iter.next() else {
                    return Err(FreshnessError::new(
                        "materialize generated faces: --candidate-commit requires a commit oid",
                    ));
                };
                candidate_commit = Some(value);
            }
            "--help" | "-h" => {
                return Err(FreshnessError::new(materialize_generated_faces_usage()));
            }
            other => {
                return Err(FreshnessError::new(format!(
                    "materialize generated faces: unknown argument {other:?}; {}",
                    materialize_generated_faces_usage()
                )));
            }
        }
    }
    let retirement = match (
        retirement_control_plane,
        retirement_facts_out,
        protected_base_commit,
        candidate_commit,
    ) {
        (
            Some(control_plane_path),
            Some(facts_out),
            Some(protected_base_commit),
            Some(candidate_commit),
        ) => {
            if control_plane_path != RETIREMENT_CONTROL_PLANE_PATH {
                return Err(FreshnessError::new(
                    "materialize generated faces: retirement control-plane path is not canonical",
                ));
            }
            if facts_out.as_path() != Path::new(RETIREMENT_FACTS_PATH) {
                return Err(FreshnessError::new(
                    "materialize generated faces: retirement facts output path is not canonical",
                ));
            }
            Some(RetirementMaterializeArgs {
                control_plane_path,
                facts_out,
                protected_base_commit,
                candidate_commit,
            })
        }
        (None, None, None, None) => None,
        _ => {
            return Err(FreshnessError::new(
                "materialize generated faces: --retirement-control-plane, \
                 --retirement-facts-out, --protected-base-commit, and --candidate-commit \
                 are all-or-none",
            ));
        }
    };
    Ok(MaterializeGeneratedFacesArgs {
        repo_root,
        retirement,
    })
}

pub fn materialize_generated_faces_usage() -> &'static str {
    "usage: oya-cloud-ci-materialize-generated-faces [--repo-root <path>] \
     [--retirement-control-plane <repo-relative-path> --retirement-facts-out <path> \
     --protected-base-commit <oid> --candidate-commit <oid>]"
}

pub fn parse_face_settle_args(args: Vec<String>) -> Result<FaceSettleArgs, FreshnessError> {
    let mut repo_root = PathBuf::from(".");
    let mut settle = false;
    let mut commit = false;
    let mut verify = false;
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--repo-root" => {
                let Some(value) = iter.next() else {
                    return Err(FreshnessError::new(
                        "face settle: --repo-root requires a path",
                    ));
                };
                repo_root = PathBuf::from(value);
            }
            "--settle" => {
                settle = true;
            }
            "--commit" => {
                commit = true;
            }
            "--verify" => {
                verify = true;
            }
            "--help" | "-h" => {
                return Err(FreshnessError::new(face_settle_usage()));
            }
            other => {
                return Err(FreshnessError::new(format!(
                    "face settle: unknown argument {other:?}; {}",
                    face_settle_usage()
                )));
            }
        }
    }
    if verify && (settle || commit) {
        return Err(FreshnessError::new(
            "face settle: --verify is read-only and cannot be combined with --settle/--commit",
        ));
    }
    if commit && !settle {
        return Err(FreshnessError::new(
            "face settle: --commit requires --settle",
        ));
    }
    let mode = if verify {
        FaceSettleMode::Verify
    } else {
        match (settle, commit) {
            (false, false) => FaceSettleMode::Check,
            (true, false) => FaceSettleMode::Settle,
            (true, true) => FaceSettleMode::SettleAndCommit,
            (false, true) => FaceSettleMode::Check,
        }
    };
    Ok(FaceSettleArgs { repo_root, mode })
}

pub fn face_settle_usage() -> &'static str {
    "usage: oya-cloud-ci-face-settle [--repo-root <path>] [--settle [--commit] | --verify]"
}

pub fn check_regenerated_faces(
    repo_root: &Path,
    regenerated_faces: Vec<(String, String)>,
) -> Result<FaceSettleReport, FreshnessError> {
    assert_non_face_tree_clean(repo_root)?;
    let committed_faces = read_committed_generated_faces(repo_root)?;
    let decommitted = read_decommitted_face_names(repo_root);
    let findings = evaluate_face_freshness(&committed_faces, &regenerated_faces, &decommitted);
    let stale_faces: Vec<String> = findings.iter().map(|finding| finding.key.clone()).collect();
    let message = if stale_faces.is_empty() {
        "generated cloud-ci faces are settled".to_owned()
    } else {
        format!(
            "generated cloud-ci faces are stale:\n{}\n\nRun: {FACE_REMEDIATION_COMMAND}\nProtocol: {FACE_SETTLE_PROTOCOL}",
            bullet_list(&stale_faces)
        )
    };
    Ok(FaceSettleReport {
        message,
        stale_faces,
        lock_findings: Vec::new(),
        staged_faces: Vec::new(),
        committed: false,
    })
}

/// Read-only certification that the COMMITTED tree (HEAD) passes the cloud-ci freshness
/// gate: it runs the gate's OWN full check (`check_repo_with_regenerated_faces`, single
/// owner — generated-face byte parity AND Cargo.lock member parity) against a tree that
/// is asserted byte-identical to HEAD. Exit contract: success only when the gate itself
/// would be green; otherwise the report names each stale face / lock finding and prints
/// the exact remediation command. This function NEVER writes: it only runs read-only git
/// queries and filesystem reads.
///
/// FRIC-1781250000 (historical) + FRIC-1781234047/ADR-0552 (the structural fix): committed
/// scm-facts USED to encode per-path `last_touch_commit`, so any later commit touching a
/// non-generated-class path — and every squash-merge to the base branch — un-settled the
/// faces. Under the v2 stable/volatile split the committed faces are a pure function of the
/// tracked TREE, so only changes that actually alter face-relevant tree content un-settle
/// them. This remains the required LAST step before EVERY push; the ADR-0539 freshness gate
/// remains the canonical backstop.
pub fn verify_committed_tree(
    repo_root: &Path,
    regenerated_faces: Vec<(String, String)>,
) -> Result<FaceSettleReport, FreshnessError> {
    assert_committed_tree_clean(repo_root)?;
    // With the FULL tracked tree clean and no untracked files (asserted above), every
    // tracked path in the working tree is byte-identical to HEAD, so this check IS the
    // freshness-gate check performed on a clean CI checkout of this commit.
    let report = check_repo_with_regenerated_faces(repo_root, regenerated_faces)?;
    let stale_faces: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| finding.code == FindingCode::GeneratedFaceStale)
        .map(|finding| finding.key.clone())
        .collect();
    let lock_findings: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| finding.code != FindingCode::GeneratedFaceStale)
        .map(|finding| format!("{} {}: {}", finding.code, finding.key, finding.detail))
        .collect();
    let message = if stale_faces.is_empty() && lock_findings.is_empty() {
        "face-settle --verify: committed tree is settled at HEAD — generated faces byte-identical and Cargo.lock member-fresh (the cloud-ci freshness gate's own checks)".to_owned()
    } else {
        let mut sections = Vec::new();
        if !stale_faces.is_empty() {
            sections.push(format!(
                "stale generated faces:\n{}\nRemediation: {FACE_VERIFY_REMEDIATION_COMMAND}",
                bullet_list(&stale_faces)
            ));
        }
        if !lock_findings.is_empty() {
            sections.push(format!(
                "stale Cargo.lock state:\n{}\nRemediation: {LOCK_REMEDIATION_COMMAND} (then commit the refreshed Cargo.lock as a content commit)",
                bullet_list(&lock_findings)
            ));
        }
        format!(
            "face-settle --verify: committed tree is STALE at HEAD — pushing now fails the cloud-ci freshness gate:\n{}\n\nProtocol: {FACE_SETTLE_PROTOCOL}",
            sections.join("\n")
        )
    };
    Ok(FaceSettleReport {
        message,
        stale_faces,
        lock_findings,
        staged_faces: Vec::new(),
        committed: false,
    })
}

pub fn settle_regenerated_faces(
    repo_root: &Path,
    regenerated_faces: Vec<(String, String)>,
    mode: FaceSettleMode,
) -> Result<FaceSettleReport, FreshnessError> {
    assert_non_face_tree_clean(repo_root)?;
    let non_pr_owned = read_decommitted_face_names(repo_root);
    write_regenerated_faces(repo_root, &regenerated_faces, &non_pr_owned)?;
    let pr_owned_face_paths = pr_owned_generated_face_paths(&non_pr_owned);
    let changed_faces = tracked_face_changes(repo_root)?;
    let changed_pr_owned_faces: Vec<String> = changed_faces
        .into_iter()
        .filter(|path| pr_owned_face_paths.iter().any(|allowed| allowed == path))
        .collect();
    if changed_pr_owned_faces.is_empty() {
        return Ok(FaceSettleReport {
            message: "generated cloud-ci PR-owned faces are already settled".to_owned(),
            stale_faces: Vec::new(),
            lock_findings: Vec::new(),
            staged_faces: Vec::new(),
            committed: false,
        });
    }

    git_add_face_paths(repo_root, &pr_owned_face_paths)?;
    let staged_faces = staged_paths(repo_root)?;
    assert_staged_paths_are_pr_owned_faces(&staged_faces, &pr_owned_face_paths)?;
    let committed = if mode == FaceSettleMode::SettleAndCommit {
        git_commit_faces(repo_root)?;
        true
    } else {
        false
    };
    let commit_line = if committed {
        "created generated-face settle commit".to_owned()
    } else {
        format!("suggested commit: {FACE_SETTLE_COMMIT_COMMAND}")
    };
    Ok(FaceSettleReport {
        message: format!(
            "staged generated cloud-ci face diffs only:\n{}\n\n{commit_line}\nProtocol: {FACE_SETTLE_PROTOCOL}\nCommand: {FACE_REMEDIATION_COMMAND}",
            bullet_list(&staged_faces)
        ),
        stale_faces: Vec::new(),
        lock_findings: Vec::new(),
        staged_faces,
        committed,
    })
}

pub fn assert_non_face_tree_clean(repo_root: &Path) -> Result<(), FreshnessError> {
    let tracked_changes = tracked_non_face_changes(repo_root)?;
    let untracked_paths = untracked_paths(repo_root)?;
    if tracked_changes.is_empty() && untracked_paths.is_empty() {
        Ok(())
    } else {
        let mut sections = Vec::new();
        if !tracked_changes.is_empty() {
            sections.push(format!(
                "tracked non-face changes:\n{}",
                bullet_list(&tracked_changes)
            ));
        }
        if !untracked_paths.is_empty() {
            sections.push(format!(
                "untracked files alter the tracked-paths universe used by generated faces:\n{}",
                bullet_list(&untracked_paths)
            ));
        }
        Err(FreshnessError::new(format!(
            "non-face tree state must be committed before settling generated faces:\n{}\nProtocol: {FACE_SETTLE_PROTOCOL}",
            sections.join("\n")
        )))
    }
}

/// Verify-mode precondition: the FULL tracked tree (faces included) must match HEAD and no
/// untracked files may exist. Unlike `assert_non_face_tree_clean`, dirty FACE paths also
/// refuse — an uncommitted face edit means the committed state cannot be certified (the
/// classic shape: `--settle` ran but the settle commit was forgotten).
pub fn assert_committed_tree_clean(repo_root: &Path) -> Result<(), FreshnessError> {
    let tracked_changes = tracked_non_face_changes(repo_root)?;
    let face_changes = tracked_face_changes(repo_root)?;
    let untracked_paths = untracked_paths(repo_root)?;
    if tracked_changes.is_empty() && face_changes.is_empty() && untracked_paths.is_empty() {
        return Ok(());
    }
    let mut sections = Vec::new();
    if !tracked_changes.is_empty() {
        sections.push(format!(
            "tracked non-face changes:\n{}",
            bullet_list(&tracked_changes)
        ));
    }
    if !face_changes.is_empty() {
        sections.push(format!(
            "uncommitted generated-face changes (did you run --settle without --commit?):\n{}",
            bullet_list(&face_changes)
        ));
    }
    if !untracked_paths.is_empty() {
        sections.push(format!(
            "untracked files:\n{}",
            bullet_list(&untracked_paths)
        ));
    }
    Err(FreshnessError::new(format!(
        "face-settle --verify certifies the COMMITTED tree (HEAD) only; commit or remove these changes first:\n{}\nProtocol: {FACE_SETTLE_PROTOCOL}",
        sections.join("\n")
    )))
}

pub fn check_repo_with_regenerated_faces(
    repo_root: &Path,
    regenerated_faces: Vec<(String, String)>,
) -> Result<CheckReport, FreshnessError> {
    let members = read_member_packages(repo_root)?;
    let lock_text = read_to_string(&repo_root.join("Cargo.lock"))?;
    let lock_packages = parse_lock_packages(&lock_text)?;
    let committed_faces = read_committed_generated_faces(repo_root)?;
    let decommitted = read_decommitted_face_names(repo_root);

    let mut findings = BTreeSet::new();
    findings.extend(evaluate_lock_freshness(&members, &lock_packages));
    findings.extend(evaluate_face_freshness(
        &committed_faces,
        &regenerated_faces,
        &decommitted,
    ));
    findings.extend(evaluate_rust_toolchain_drift(repo_root)?);

    Ok(CheckReport {
        findings: findings.into_iter().collect(),
    })
}

pub fn parse_lock_packages(lock_text: &str) -> Result<Vec<LockPackage>, FreshnessError> {
    let document: toml::Value = toml::from_str(lock_text)
        .map_err(|error| FreshnessError::new(format!("parse Cargo.lock: {error}")))?;
    let packages = document
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| FreshnessError::new("Cargo.lock missing [[package]] array"))?;

    let mut parsed = Vec::with_capacity(packages.len());
    for package in packages {
        let table = package
            .as_table()
            .ok_or_else(|| FreshnessError::new("Cargo.lock package entry is not a table"))?;
        let name = required_string(table, "name", "Cargo.lock package")?;
        let version = required_string(table, "version", "Cargo.lock package")?;
        if table.contains_key("source") {
            parsed.push(LockPackage::external(name, version));
        } else {
            parsed.push(LockPackage::path(name, version));
        }
    }
    Ok(parsed)
}

pub fn parse_member_package_manifest(
    member_path: &str,
    manifest_text: &str,
    workspace_version: &str,
) -> Result<MemberPackage, FreshnessError> {
    let document: toml::Value = toml::from_str(manifest_text)
        .map_err(|error| FreshnessError::new(format!("parse {member_path}/Cargo.toml: {error}")))?;
    let package = document
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| {
            FreshnessError::new(format!("{member_path}/Cargo.toml missing [package]"))
        })?;
    let name = required_string(
        package,
        "name",
        &format!("{member_path}/Cargo.toml [package]"),
    )?;
    let version = package_version(package, workspace_version, member_path)?;
    Ok(MemberPackage::new(member_path, name, version))
}

/// Evaluate generated-face freshness.
///
/// Two predicate classes:
///   - committed-class faces (NOT in `decommitted`): byte parity — committed bytes must equal the
///     buck2-regenerated bytes, and a regeneration that produces a face with no committed copy is
///     stale. This is the unchanged contract for any face still tracked in git.
///   - non-PR-owned faces (in `decommitted`, a legacy parameter name): the face is either
///     de-committed (`not-tracked-in-git`) or integration-controller-owned
///     (`main-branch-materialized`), so contributor PRs do not own byte parity against the local
///     checkout copy. Freshness instead requires the face to regenerate successfully (it must be
///     present in `regenerated`). Determinism (regenerate-twice byte-stability) is enforced
///     separately by [`evaluate_face_determinism`].
pub fn evaluate_face_freshness(
    committed: &[(String, String)],
    regenerated: &[(String, String)],
    decommitted: &BTreeSet<String>,
) -> Vec<Finding> {
    let regenerated_by_name: BTreeMap<&str, &str> = regenerated
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_str()))
        .collect();
    let committed_names: BTreeSet<&str> = committed.iter().map(|(name, _)| name.as_str()).collect();
    let mut findings = BTreeSet::new();

    for (name, committed_bytes) in committed {
        // A non-PR-owned face that lingers on disk is not a contributor merge surface; do not
        // require byte parity against a local copy. It is validated via the regenerated loop below
        // + the determinism canary.
        if decommitted.contains(name) {
            continue;
        }
        match regenerated_by_name.get(name.as_str()) {
            Some(regenerated_bytes) if *regenerated_bytes == committed_bytes => {}
            Some(_) => {
                findings.insert(stale_face_finding(
                    name,
                    "committed bytes differ from regenerated bytes",
                ));
            }
            None => {
                findings.insert(stale_face_finding(
                    name,
                    "regeneration did not produce this committed face",
                ));
            }
        }
    }

    for (name, _) in regenerated {
        if decommitted.contains(name) {
            // Non-PR-owned class: regeneration producing this face is the REQUIRED state, not stale.
            continue;
        }
        if !committed_names.contains(name.as_str()) {
            findings.insert(stale_face_finding(
                name,
                "regeneration produced an uncommitted generated face",
            ));
        }
    }

    // Every declared non-PR-owned face must still regenerate; a producer that silently stops
    // emitting one would otherwise pass (it has no contributor-owned byte parity and no regenerated
    // entry).
    for name in decommitted {
        if !regenerated_by_name.contains_key(name.as_str()) {
            findings.insert(stale_face_finding(
                name,
                "non-PR-owned generated face was not produced by regeneration",
            ));
        }
    }

    findings.into_iter().collect()
}

/// Determinism canary for non-PR-owned faces: regenerating twice must yield byte-identical output.
/// With contributor-branch byte parity removed for these faces, this is the integrity canary that
/// keeps derive-on-demand/controller-materialization sound — a nondeterministic producer must
/// hard-fail here rather than silently green.
pub fn evaluate_face_determinism(
    first: &[(String, String)],
    second: &[(String, String)],
    decommitted: &BTreeSet<String>,
) -> Vec<Finding> {
    let second_by_name: BTreeMap<&str, &str> = second
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_str()))
        .collect();
    let mut findings = BTreeSet::new();
    for (name, first_bytes) in first {
        if !decommitted.contains(name) {
            continue;
        }
        match second_by_name.get(name.as_str()) {
            Some(second_bytes) if *second_bytes == first_bytes => {}
            _ => {
                findings.insert(stale_face_finding(
                    name,
                    "non-PR-owned generated face is not deterministic across regenerations",
                ));
            }
        }
    }
    findings.into_iter().collect()
}

pub fn render_remediation() -> String {
    format!(
        "Remediation:\n  lock: {LOCK_REMEDIATION_COMMAND}\n  faces: {FACE_REMEDIATION_COMMAND}\n  rust-toolchain: align rust-toolchain.toml, Cargo manifests, service manifests, Dockerfiles, workflows, and active standards/spec text to one exact stable patch\n  face settle protocol: {FACE_SETTLE_PROTOCOL}"
    )
}

pub fn render_findings(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "freshness gate passed".to_owned();
    }

    let mut output = String::from("freshness gate failed:\n");
    for finding in findings {
        output.push_str(&format!(
            "- {} {}: {}\n",
            finding.code, finding.key, finding.detail
        ));
    }
    output.push('\n');
    output.push_str(&render_remediation());
    output
}

pub fn read_member_packages(repo_root: &Path) -> Result<Vec<MemberPackage>, FreshnessError> {
    let workspace_version = read_workspace_version(repo_root)?;
    let member_dirs = resolve_member_dirs(repo_root)
        .map_err(|error| FreshnessError::new(format!("resolve workspace members: {error}")))?;
    let mut members = Vec::with_capacity(member_dirs.len());
    for member_dir in member_dirs {
        let manifest = read_to_string(&repo_root.join(&member_dir).join("Cargo.toml"))?;
        members.push(parse_member_package_manifest(
            &member_dir,
            &manifest,
            &workspace_version,
        )?);
    }
    Ok(members)
}

pub fn read_committed_generated_faces(
    repo_root: &Path,
) -> Result<Vec<(String, String)>, FreshnessError> {
    let dir = repo_root.join(FACES_DIR);
    let entries = std::fs::read_dir(&dir)
        .map_err(|error| FreshnessError::new(format!("read {}: {error}", dir.display())))?;
    let mut faces = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            FreshnessError::new(format!("read entry in {}: {error}", dir.display()))
        })?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".generated.json") {
            continue;
        }
        faces.push((name.to_owned(), read_to_string(&path)?));
    }
    let product_graph = repo_root.join(ARCHITECTURE_PRODUCT_GRAPH_PATH);
    if product_graph.exists() {
        faces.push((
            ARCHITECTURE_PRODUCT_GRAPH_FACE.to_owned(),
            read_to_string(&product_graph)?,
        ));
    }
    let masterplan = repo_root.join(MASTERPLAN_PROJECTION_PATH);
    if masterplan.exists() {
        faces.push((
            MASTERPLAN_PROJECTION_FACE.to_owned(),
            read_to_string(&masterplan)?,
        ));
    }
    let board_sync = repo_root.join(BOARD_SYNC_PROJECTION_PATH);
    if board_sync.exists() {
        faces.push((
            BOARD_SYNC_PROJECTION_FACE.to_owned(),
            read_to_string(&board_sync)?,
        ));
    }
    faces.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(faces)
}

/// Read the file names of non-PR-owned faces from the generated-artifact control-plane manifest.
/// A declared artifact whose `materialization_mode` is `not-tracked-in-git` OR
/// `main-branch-materialized` AND whose `path` EXACTLY equals one of this gate's canonical
/// generated-face paths ([`GENERATED_FACE_PATHS`]) or explicitly controller-materialized paths
/// ([`CONTROLLER_MATERIALIZED_ARTIFACT_PATHS`]) is returned by its file basename (e.g.
/// `ttl-policy.generated.json`), matching the keys used everywhere else in this gate. A missing or
/// malformed manifest yields an empty set, so the byte-parity contract is the safe default (no face
/// is silently exempted).
///
/// The match is on the CANONICAL FULL PATH, never the basename: a deceptive manifest row at a
/// non-canonical path that merely shares a basename with a committed face (e.g.
/// `anything/scm-facts.generated.json`) must NOT retire the real committed face's byte-parity
/// check. Keying on basename here would let a candidate-controlled manifest collapse such a row to
/// the committed face's name and silently exempt it.
pub fn read_decommitted_face_names(repo_root: &Path) -> BTreeSet<String> {
    let path = repo_root.join(CONTROL_PLANE_MANIFEST);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return BTreeSet::new();
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&text) else {
        return BTreeSet::new();
    };
    let Some(artifacts) = manifest.get("artifacts").and_then(|value| value.as_array()) else {
        return BTreeSet::new();
    };
    let mut names = BTreeSet::new();
    for artifact in artifacts {
        let mode = artifact
            .get("materialization_mode")
            .and_then(|value| value.as_str());
        if mode != Some(NOT_TRACKED_IN_GIT_MODE) && mode != Some(MAIN_BRANCH_MATERIALIZED_MODE) {
            continue;
        }
        let Some(path) = artifact.get("path").and_then(|value| value.as_str()) else {
            continue;
        };
        // Scope strictly to this gate's materialized artifacts by CANONICAL FULL PATH, never
        // basename. Matching on basename would let a candidate-controlled manifest row at a
        // non-canonical path (e.g. `anything/scm-facts.generated.json`) collapse to a committed
        // face's name and silently retire that committed face's byte-parity check. An unrelated
        // de-commit-class artifact elsewhere in the manifest must not change freshness behavior
        // here.
        if GENERATED_FACE_PATHS.contains(&path)
            || CONTROLLER_MATERIALIZED_ARTIFACT_PATHS.contains(&path)
        {
            names.insert(file_basename(path).to_owned());
        }
    }
    names
}

fn file_basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// A regenerated face set: `(face_file_name, face_bytes)` pairs, sorted by name.
pub type RegeneratedFaces = Vec<(String, String)>;

pub fn regenerate_faces_with_buck2(repo_root: &Path) -> Result<RegeneratedFaces, FreshnessError> {
    let tools = build_face_tools(repo_root)?;
    let scm_facts = temporary_scm_facts_path();
    let cleanup = TempFileCleanup {
        path: scm_facts.clone(),
    };
    // The volatile snapshot is routed to its own temp path too: this regeneration exists
    // only to byte-compare the COMMITTED faces, and `--verify` is contractually read-only —
    // it must not rewrite the checkout's materialized scm-volatile-facts snapshot.
    let volatile_facts = temporary_volatile_facts_path();
    let volatile_cleanup = TempFileCleanup {
        path: volatile_facts.clone(),
    };
    emit_scm_facts(&tools, repo_root, &scm_facts, &volatile_facts)?;
    let regenerated = regenerate_all_faces(&tools, repo_root, &scm_facts)?;
    drop(cleanup);
    drop(volatile_cleanup);
    Ok(regenerated)
}

/// Regenerate the producer faces a SECOND time from the SAME scm-facts snapshot and return both
/// passes (ADR-0595 determinism canary). Building the tools and emitting scm-facts ONCE — then
/// re-running only the producer — isolates producer determinism (the emitter is proven
/// deterministic) and avoids a double `buck2 build` that could otherwise race a mid-run rebuild.
/// The `(first_pass, second_pass)` tuple feeds [`evaluate_face_determinism`].
pub fn regenerate_faces_twice_with_buck2(
    repo_root: &Path,
) -> Result<(RegeneratedFaces, RegeneratedFaces), FreshnessError> {
    let tools = build_face_tools(repo_root)?;
    let scm_facts = temporary_scm_facts_path();
    let cleanup = TempFileCleanup {
        path: scm_facts.clone(),
    };
    let volatile_facts = temporary_volatile_facts_path();
    let volatile_cleanup = TempFileCleanup {
        path: volatile_facts.clone(),
    };
    emit_scm_facts(&tools, repo_root, &scm_facts, &volatile_facts)?;
    let first = regenerate_all_faces(&tools, repo_root, &scm_facts)?;
    let second = regenerate_all_faces(&tools, repo_root, &scm_facts)?;
    drop(cleanup);
    drop(volatile_cleanup);
    Ok((first, second))
}

fn emit_scm_facts(
    tools: &FaceTools,
    repo_root: &Path,
    scm_facts: &Path,
    volatile_facts: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(&tools.emitter)
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--out"])
            .arg(scm_facts)
            .args(["--volatile-out"])
            .arg(volatile_facts)
            .current_dir(repo_root),
        "run scm-facts emitter",
    )
}

fn regenerate_producer_faces(
    tools: &FaceTools,
    repo_root: &Path,
    scm_facts: &Path,
) -> Result<RegeneratedFaces, FreshnessError> {
    let mut regenerated = vec![(SCM_FACTS_FACE.to_owned(), read_to_string(scm_facts)?)];
    for (file_name, face_name) in PRODUCER_FACES {
        let mut command = Command::new(&tools.producer);
        command
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--scm-facts"])
            .arg(scm_facts);
        append_enforcement_liveness_corpus_args(&mut command, &tools.enforcement_liveness_corpus);
        command
            .args(["--stdout", "--face", face_name])
            .current_dir(repo_root);
        let output = run_output(&mut command, &format!("regenerate {file_name}"))?;
        regenerated.push((file_name.to_owned(), output));
    }
    regenerated.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(regenerated)
}

fn append_enforcement_liveness_corpus_args(
    command: &mut Command,
    corpus: &EnforcementLivenessCorpusPaths,
) {
    append_enforcement_liveness_corpus_paths(
        command,
        &corpus.claude_settings,
        &corpus.codex_hooks,
        &corpus.hooks_dir,
    );
}

fn append_enforcement_liveness_corpus_paths(
    command: &mut Command,
    claude_settings: &Path,
    codex_hooks: &Path,
    hooks_dir: &Path,
) {
    command
        .arg("--enforcement-liveness-claude-settings")
        .arg(claude_settings)
        .arg("--enforcement-liveness-codex-hooks")
        .arg(codex_hooks)
        .arg("--enforcement-liveness-hooks-dir")
        .arg(hooks_dir);
}

fn regenerate_all_faces(
    tools: &FaceTools,
    repo_root: &Path,
    scm_facts: &Path,
) -> Result<RegeneratedFaces, FreshnessError> {
    let mut regenerated = regenerate_producer_faces(tools, repo_root, scm_facts)?;
    regenerated.push(regenerate_adr_census_parent_receipt(
        &tools.emitter,
        repo_root,
    )?);
    regenerated.push(regenerate_active_artifact_contract_graph(tools, repo_root)?);
    regenerated.extend(regenerate_architecture_projection_faces(tools, repo_root)?);
    regenerated.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(regenerated)
}

fn regenerate_adr_census_parent_receipt(
    emitter: &Path,
    repo_root: &Path,
) -> Result<(String, String), FreshnessError> {
    let output = temporary_adr_census_parent_receipt_path();
    let cleanup = TempFileCleanup {
        path: output.clone(),
    };
    emit_adr_census_parent_receipt(emitter, repo_root, &output)?;
    let bytes = read_to_string(&output)?;
    drop(cleanup);
    Ok((ADR_CENSUS_PARENT_RECEIPT_FACE.to_owned(), bytes))
}

fn emit_adr_census_parent_receipt(
    emitter: &Path,
    repo_root: &Path,
    output: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(emitter)
            .args(["--repo-root"])
            .arg(repo_root)
            .arg("--adr-census-parent-receipt")
            .arg("--adr-census-parent-receipt-out")
            .arg(output)
            .current_dir(repo_root),
        "materialize fixed historical ADR census receipt",
    )
}

fn regenerate_active_artifact_contract_graph(
    tools: &FaceTools,
    repo_root: &Path,
) -> Result<(String, String), FreshnessError> {
    let output = temporary_active_artifact_contract_graph_path();
    let cleanup = TempFileCleanup {
        path: output.clone(),
    };
    run_status(
        Command::new(&tools.masterplan_generator)
            .args(["gate", "validate", "active-artifact-contract"])
            .arg("--emit-graph-edges")
            .arg(&output)
            .current_dir(repo_root),
        "regenerate active-artifact-contract graph",
    )?;
    let bytes = read_to_string(&output)?;
    drop(cleanup);
    Ok((ACTIVE_ARTIFACT_CONTRACT_GRAPH_FACE.to_owned(), bytes))
}

fn regenerate_architecture_projection_faces(
    tools: &FaceTools,
    repo_root: &Path,
) -> Result<RegeneratedFaces, FreshnessError> {
    let masterplan = temporary_masterplan_path();
    let masterplan_cleanup = TempFileCleanup {
        path: masterplan.clone(),
    };
    let output = temporary_product_graph_path();
    let cleanup = TempFileCleanup {
        path: output.clone(),
    };
    run_status(
        Command::new(&tools.masterplan_generator)
            .args(["gen", "masterplan", "--write", "--output"])
            .arg(&masterplan)
            .current_dir(repo_root),
        "regenerate masterplan projection for architecture product graph",
    )?;
    run_status(
        Command::new(&tools.architecture_graph_generator)
            .arg("--write")
            .args(["--masterplan"])
            .arg(&masterplan)
            .args(["--output"])
            .arg(&output)
            .current_dir(repo_root),
        "regenerate architecture product graph",
    )?;
    let masterplan_bytes = read_to_string(&masterplan)?;
    let masterplan_value = serde_json::from_str(&masterplan_bytes).map_err(|error| {
        FreshnessError::new(format!(
            "parse regenerated masterplan projection {}: {error}",
            masterplan.display()
        ))
    })?;
    let board_sync_bytes = render_board_sync_projection(&masterplan_value)
        .map_err(|error| FreshnessError::new(format!("render board-sync projection: {error}")))?;
    let product_graph_bytes = read_to_string(&output)?;
    drop(cleanup);
    drop(masterplan_cleanup);
    Ok(vec![
        (MASTERPLAN_PROJECTION_FACE.to_owned(), masterplan_bytes),
        (BOARD_SYNC_PROJECTION_FACE.to_owned(), board_sync_bytes),
        (
            ARCHITECTURE_PRODUCT_GRAPH_FACE.to_owned(),
            product_graph_bytes,
        ),
    ])
}

fn write_regenerated_faces(
    repo_root: &Path,
    regenerated_faces: &[(String, String)],
    non_pr_owned: &BTreeSet<String>,
) -> Result<(), FreshnessError> {
    for (name, bytes) in regenerated_faces {
        if non_pr_owned.contains(name) {
            continue;
        }
        let path = generated_face_path_for_name(name)
            .ok_or_else(|| FreshnessError::new(format!("unknown generated face {name:?}")))?;
        let full_path = repo_root.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                FreshnessError::new(format!("create {}: {error}", parent.display()))
            })?;
        }
        std::fs::write(&full_path, bytes)
            .map_err(|error| FreshnessError::new(format!("write {}: {error}", path)))?;
    }
    Ok(())
}

fn generated_face_path_for_name(name: &str) -> Option<&'static str> {
    GENERATED_FACE_PATHS.iter().copied().find(|path| {
        path.rsplit('/')
            .next()
            .map(|file_name| file_name == name)
            .unwrap_or(false)
    })
}

fn pr_owned_generated_face_paths(non_pr_owned: &BTreeSet<String>) -> Vec<String> {
    GENERATED_FACE_PATHS
        .iter()
        .filter(|path| !non_pr_owned.contains(file_basename(path)))
        .map(|path| (*path).to_owned())
        .collect()
}

fn tracked_non_face_changes(repo_root: &Path) -> Result<Vec<String>, FreshnessError> {
    let output = run_output(
        Command::new("git")
            .args(["status", "--porcelain=v1", "--untracked-files=no"])
            .current_dir(repo_root),
        "git status tracked changes",
    )?;
    let mut changes = Vec::new();
    for line in output.lines() {
        let Some(path) = porcelain_status_path(line) else {
            continue;
        };
        if !is_generated_face_path(&path) {
            changes.push(path);
        }
    }
    changes.sort();
    Ok(changes)
}

fn untracked_paths(repo_root: &Path) -> Result<Vec<String>, FreshnessError> {
    let output = run_output(
        Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(repo_root),
        "git list untracked files",
    )?;
    let mut paths: Vec<String> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect();
    paths.sort();
    Ok(paths)
}

fn tracked_face_changes(repo_root: &Path) -> Result<Vec<String>, FreshnessError> {
    let mut command = Command::new("git");
    command
        .args(["status", "--porcelain=v1", "--untracked-files=no", "--"])
        .args(GENERATED_FACE_PATHS)
        .current_dir(repo_root);
    let output = run_output(&mut command, "git status generated face changes")?;
    let mut changes = Vec::new();
    for line in output.lines() {
        if let Some(path) = porcelain_status_path(line) {
            changes.push(path);
        }
    }
    changes.sort();
    Ok(changes)
}

fn staged_paths(repo_root: &Path) -> Result<Vec<String>, FreshnessError> {
    let output = run_output(
        Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(repo_root),
        "git diff staged paths",
    )?;
    Ok(output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect())
}

fn porcelain_status_path(line: &str) -> Option<String> {
    let path = line.get(3..)?.trim();
    let normalized = path
        .rsplit_once(" -> ")
        .map(|(_, new_path)| new_path)
        .unwrap_or(path)
        .trim_matches('"');
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_owned())
    }
}

fn is_generated_face_path(path: &str) -> bool {
    GENERATED_FACE_PATHS.contains(&path)
}

fn git_add_face_paths(repo_root: &Path, paths: &[String]) -> Result<(), FreshnessError> {
    run_status(
        Command::new("git")
            .arg("add")
            .arg("--")
            .args(paths)
            .current_dir(repo_root),
        "git add PR-owned generated face paths",
    )
}

fn git_commit_faces(repo_root: &Path) -> Result<(), FreshnessError> {
    run_status(
        Command::new("git")
            .args(["commit", "-m", FACE_SETTLE_COMMIT_MESSAGE])
            .current_dir(repo_root),
        "git commit generated face settle",
    )
}

fn assert_staged_paths_are_pr_owned_faces(
    paths: &[String],
    allowed_paths: &[String],
) -> Result<(), FreshnessError> {
    if paths.is_empty() {
        return Err(FreshnessError::new(
            "no generated face paths are staged for settle commit",
        ));
    }
    let allowed: BTreeSet<&str> = allowed_paths.iter().map(String::as_str).collect();
    let bad: Vec<String> = paths
        .iter()
        .filter(|path| !allowed.contains(path.as_str()))
        .cloned()
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(FreshnessError::new(format!(
            "settle commit may stage only PR-owned generated face paths, found:\n{}\nProtocol: {FACE_SETTLE_PROTOCOL}",
            bullet_list(&bad)
        )))
    }
}

fn bullet_list(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

struct FaceTools {
    emitter: PathBuf,
    producer: PathBuf,
    masterplan_generator: PathBuf,
    architecture_graph_generator: PathBuf,
    enforcement_liveness_corpus: EnforcementLivenessCorpusPaths,
}

struct MaterializerTools {
    emitter: PathBuf,
    producer: PathBuf,
    codemod: PathBuf,
    masterplan_generator: PathBuf,
    architecture_graph_generator: PathBuf,
    enforcement_liveness_corpus: EnforcementLivenessCorpusPaths,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnforcementLivenessCorpusPaths {
    claude_settings: PathBuf,
    codex_hooks: PathBuf,
    hooks_dir: PathBuf,
}

fn build_face_tools(repo_root: &Path) -> Result<FaceTools, FreshnessError> {
    let output = run_output(
        Command::new("buck2")
            .arg("build")
            .arg(EMITTER_TARGET)
            .arg(PRODUCER_TARGET)
            .arg(MASTERPLAN_GENERATOR_TARGET)
            .arg(ARCHITECTURE_GRAPH_GENERATOR_TARGET)
            .arg(ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_TARGET)
            .arg(ENFORCEMENT_LIVENESS_CODEX_HOOKS_TARGET)
            .arg(ENFORCEMENT_LIVENESS_HOOKS_DIR_TARGET)
            .arg("--show-output")
            .current_dir(repo_root),
        "buck2 build freshness face tools",
    )?;
    let emitter = parse_show_output_path(repo_root, &output, EMITTER_TARGET)?;
    let producer = parse_show_output_path(repo_root, &output, PRODUCER_TARGET)?;
    let masterplan_generator =
        parse_show_output_path(repo_root, &output, MASTERPLAN_GENERATOR_TARGET)?;
    let architecture_graph_generator =
        parse_show_output_path(repo_root, &output, ARCHITECTURE_GRAPH_GENERATOR_TARGET)?;
    let enforcement_liveness_corpus = parse_enforcement_liveness_corpus_paths(repo_root, &output)?;
    Ok(FaceTools {
        emitter,
        producer,
        masterplan_generator,
        architecture_graph_generator,
        enforcement_liveness_corpus,
    })
}

fn build_materializer_tools(repo_root: &Path) -> Result<MaterializerTools, FreshnessError> {
    // Absolute repo-root so the derived tool binary paths are spawnable from ANY current_dir:
    // the ADR-0616 merge-base regen runs these tools with cwd set to the merge-base worktree, where
    // a `--repo-root .`-relative `buck-out/...` binary path would not resolve (os error 2 on spawn).
    let repo_root_abs = std::fs::canonicalize(repo_root).map_err(|e| {
        FreshnessError::new(format!(
            "canonicalize repo-root {}: {e}",
            repo_root.display()
        ))
    })?;
    let repo_root = repo_root_abs.as_path();
    let output = run_output(
        Command::new("buck2")
            .arg("build")
            .arg(EMITTER_TARGET)
            .arg(PRODUCER_TARGET)
            .arg(CODEMOD_TARGET)
            .arg(MASTERPLAN_GENERATOR_TARGET)
            .arg(ARCHITECTURE_GRAPH_GENERATOR_TARGET)
            .arg(ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS_TARGET)
            .arg(ENFORCEMENT_LIVENESS_CODEX_HOOKS_TARGET)
            .arg(ENFORCEMENT_LIVENESS_HOOKS_DIR_TARGET)
            .arg("--show-output")
            .current_dir(repo_root),
        "buck2 build generated-face materializer tools",
    )?;
    let emitter = parse_show_output_path(repo_root, &output, EMITTER_TARGET)?;
    let producer = parse_show_output_path(repo_root, &output, PRODUCER_TARGET)?;
    let codemod = parse_show_output_path(repo_root, &output, CODEMOD_TARGET)?;
    let masterplan_generator =
        parse_show_output_path(repo_root, &output, MASTERPLAN_GENERATOR_TARGET)?;
    let architecture_graph_generator =
        parse_show_output_path(repo_root, &output, ARCHITECTURE_GRAPH_GENERATOR_TARGET)?;
    let enforcement_liveness_corpus = parse_enforcement_liveness_corpus_paths(repo_root, &output)?;
    Ok(MaterializerTools {
        emitter,
        producer,
        codemod,
        masterplan_generator,
        architecture_graph_generator,
        enforcement_liveness_corpus,
    })
}

fn materialize_move_manifest(
    tools: &MaterializerTools,
    repo_root: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(&tools.codemod)
            .arg("manifest")
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--out"])
            .arg(repo_root.join(MOVE_MANIFEST_FACE))
            .current_dir(repo_root),
        "materialize reorg move manifest",
    )
}

/// Materialize [`RATCHET_MERGE_BASE_FACE`]: for every `normal-source-merge` artifact declared in
/// the CANDIDATE control-plane manifest, read its content from the ALREADY-MATERIALIZED
/// merge-base source `worktree` (a plain filesystem read — the worktree checkout is the one git
/// boundary this materializer already owns; no additional `Command::new` call). An artifact
/// absent from the merge-base tree (e.g. brand new) is simply omitted from the face — the
/// consuming gate then fails closed on it, never falsely permissive. A missing/malformed
/// control-plane manifest yields an empty face for the SAME reason: the consuming gate treats
/// "no verifiable merge-base content" as fail-closed, so an empty face is the safe default here,
/// mirroring [`read_decommitted_face_names`]'s established lenient-parse convention.
fn materialize_ratchet_merge_base_contents(
    repo_root: &Path,
    worktree: &Path,
) -> Result<(), FreshnessError> {
    let mut contents = serde_json::Map::new();
    if let Ok(text) = std::fs::read_to_string(repo_root.join(CONTROL_PLANE_MANIFEST))
        && let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&text)
        && let Some(artifacts) = manifest.get("artifacts").and_then(|value| value.as_array())
    {
        for artifact in artifacts {
            if artifact
                .get("merge_policy")
                .and_then(|value| value.as_str())
                != Some("normal-source-merge")
            {
                continue;
            }
            let Some(path) = artifact.get("path").and_then(|value| value.as_str()) else {
                continue;
            };
            if let Ok(content) = std::fs::read_to_string(worktree.join(path)) {
                contents.insert(path.to_owned(), serde_json::Value::String(content));
            }
        }
    }
    let face_path = repo_root.join(RATCHET_MERGE_BASE_FACE);
    if let Some(parent) = face_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| FreshnessError::new(format!("mkdir {}: {error}", parent.display())))?;
    }
    let body =
        serde_json::to_string_pretty(&serde_json::Value::Object(contents)).map_err(|error| {
            FreshnessError::new(format!("serialize {RATCHET_MERGE_BASE_FACE}: {error}"))
        })?;
    std::fs::write(&face_path, body)
        .map_err(|error| FreshnessError::new(format!("write {RATCHET_MERGE_BASE_FACE}: {error}")))
}

fn emit_materialized_scm_facts(
    tools: &MaterializerTools,
    repo_root: &Path,
    scm_facts: &Path,
    retirement: Option<&RetirementMaterializeArgs>,
) -> Result<(), FreshnessError> {
    // Phase 1: publish the merge-base sha (the emitter owns it — the single git boundary — and the
    // materializer materializes EXACTLY that source tree, never recomputing it). This same call
    // writes the candidate scm-facts face (--out).
    let merge_base_file = temporary_merge_base_path();
    let merge_base_cleanup = TempFileCleanup {
        path: merge_base_file.clone(),
    };
    let mut candidate_emission = Command::new(&tools.emitter);
    candidate_emission
        .args(["--repo-root"])
        .arg(repo_root)
        .args(["--out"])
        .arg(scm_facts)
        .arg("--merge-base-baseline")
        .args(["--merge-base-out"])
        .arg(&merge_base_file);
    append_retirement_materialization_args(&mut candidate_emission, retirement);
    candidate_emission.current_dir(repo_root);
    run_status(
        &mut candidate_emission,
        "publish merge-base and candidate retirement facts for frozen-baseline regeneration",
    )?;
    let merge_base = read_merge_base(&merge_base_file)?;

    // Phase 2 (ADR-0616): materialize the merge-base SOURCE worktree ONCE, regenerate the frozen
    // baseline over it TWICE (the determinism twin), and hand both to the emitter which produces the
    // AUTHORITATIVE frozen snapshot — the regeneration IS the frozen reference (replacing the retired
    // `git show <merge_base>:<face>` committed-blob read), the determinism canary proves the producer
    // is reproducible, and provenance binds it to the merge-base tree. FAIL-CLOSED throughout.
    let worktree = temporary_worktree_path();
    let worktree_cleanup = WorktreeCleanup {
        repo_root: repo_root.to_path_buf(),
        path: worktree.clone(),
    };
    add_merge_base_worktree(repo_root, &merge_base, &worktree)?;

    let regen_first = regenerate_frozen_baseline_from_merge_base_source(tools, &worktree)?;
    let regen_second = regenerate_frozen_baseline_from_merge_base_source(tools, &worktree)?;

    let regen_face_file = temporary_regen_baseline_path();
    let regen_face_cleanup = TempFileCleanup {
        path: regen_face_file.clone(),
    };
    write_regen_baseline(&regen_face_file, &regen_first)?;
    let regen_verify_file = temporary_regen_baseline_verify_path();
    let regen_verify_cleanup = TempFileCleanup {
        path: regen_verify_file.clone(),
    };
    write_regen_baseline(&regen_verify_file, &regen_second)?;

    // The emitter (the single git boundary — owns the merge-base policy read, the rename-aware
    // relabel, and the provenance tree read) turns the regeneration into the authoritative frozen
    // snapshot: `--regen-baseline-verify` triggers the determinism canary, and
    // `--frozen-provenance-producer` records the analyzer identity in the provenance materials.
    run_status(
        Command::new(&tools.emitter)
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--out"])
            .arg(scm_facts)
            .arg("--merge-base-baseline")
            .args(["--regen-baseline-face"])
            .arg(&regen_face_file)
            .args(["--regen-baseline-verify"])
            .arg(&regen_verify_file)
            .args(["--frozen-provenance-producer", PRODUCER_TARGET])
            .current_dir(repo_root),
        "materialize frozen baseline from merge-base source",
    )?;

    // Reuse the SAME merge-base worktree (still open) to materialize the ratchet-baseline
    // merge-base-content face — a plain filesystem read, not a new git boundary.
    materialize_ratchet_merge_base_contents(repo_root, &worktree)?;

    drop(regen_verify_cleanup);
    drop(regen_face_cleanup);
    drop(worktree_cleanup);
    drop(merge_base_cleanup);
    Ok(())
}

fn append_retirement_materialization_args(
    command: &mut Command,
    retirement: Option<&RetirementMaterializeArgs>,
) {
    if let Some(retirement) = retirement {
        command
            .args(["--retirement-control-plane", &retirement.control_plane_path])
            .args(["--retirement-facts-out"])
            .arg(&retirement.facts_out)
            .args(["--protected-base-commit", &retirement.protected_base_commit])
            .args(["--candidate-commit", &retirement.candidate_commit]);
    }
}

/// Validate + read the merge-base sha the emitter published.
fn read_merge_base(merge_base_file: &Path) -> Result<String, FreshnessError> {
    let merge_base = read_to_string(merge_base_file)?.trim().to_owned();
    if merge_base.len() < 40 || !merge_base.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(FreshnessError::new(format!(
            "frozen-baseline regeneration: merge-base {merge_base:?} is not a revision id"
        )));
    }
    Ok(merge_base)
}

/// Materialize the merge-base SOURCE into an isolated linked worktree. A PHYSICAL checkout (not
/// `git archive`) is required because the emitter needs `.git` for `git ls-files`; `--detach` so a
/// merge-base that is also checked out elsewhere (e.g. HEAD) does not error. Registered in the
/// common `.git`, so a unique path is safe under parallel materialize runs.
fn add_merge_base_worktree(
    repo_root: &Path,
    merge_base: &str,
    worktree: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .args(["worktree", "add", "--detach"])
            .arg(worktree)
            .arg(merge_base),
        "add merge-base source worktree",
    )
}

fn write_regen_baseline(path: &Path, bytes: &str) -> Result<(), FreshnessError> {
    std::fs::write(path, bytes).map_err(|error| {
        FreshnessError::new(format!(
            "write regenerated frozen baseline {}: {error}",
            path.display()
        ))
    })
}

/// ADR-0616: regenerate the frozen reference by running the accounting producer over a materialized
/// merge-base SOURCE `worktree`. THIS is the frozen baseline the firewall compares against — it
/// REPLACES the retired `git show <merge_base>:<face>` committed-blob read. Returns the baseline
/// face JSON (producer stdout). FAIL-CLOSED: any emit/regen failure is a hard error, never a
/// fallback (a de-committed frozen with a fallback would empty-frozen-deadlock — the #828 defect).
///
/// Blob-INDEPENDENT: it runs the producer's `--face baseline`, which PRODUCES the baseline from
/// source (merge-base scm-facts + `oya-ci.toml` + the tracked tree + the enforcement-liveness
/// corpus) and never reads the (de-committed) `gate-baseline.generated.json`. Called TWICE by the
/// materializer over the same worktree for the determinism canary; the producer is deterministic,
/// so both runs agree on the ratchet projection.
fn regenerate_frozen_baseline_from_merge_base_source(
    tools: &MaterializerTools,
    worktree: &Path,
) -> Result<String, FreshnessError> {
    // Merge-base scm-facts (STABLE tracked-paths over the mb tree) via the emitter; volatile facts
    // are routed to a throwaway temp path (this regeneration is read-only w.r.t. the checkout).
    let mb_scm_facts = temporary_scm_facts_path();
    let mb_scm_facts_cleanup = TempFileCleanup {
        path: mb_scm_facts.clone(),
    };
    let mb_volatile = temporary_volatile_facts_path();
    let mb_volatile_cleanup = TempFileCleanup {
        path: mb_volatile.clone(),
    };
    run_status(
        Command::new(&tools.emitter)
            .args(["--repo-root"])
            .arg(worktree)
            .args(["--out"])
            .arg(&mb_scm_facts)
            .args(["--volatile-out"])
            .arg(&mb_volatile)
            .current_dir(worktree),
        "emit merge-base scm-facts",
    )?;

    // Run the producer rooted at the worktree with the mb tree's OWN enforcement-liveness corpus
    // (faithful mb inputs), emitting the baseline face to stdout. Never reads the committed blob.
    let mut producer = Command::new(&tools.producer);
    producer
        .args(["--repo-root"])
        .arg(worktree)
        .args(["--scm-facts"])
        .arg(&mb_scm_facts);
    append_enforcement_liveness_corpus_paths(
        &mut producer,
        &worktree.join(ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS),
        &worktree.join(ENFORCEMENT_LIVENESS_CODEX_HOOKS),
        &worktree.join(ENFORCEMENT_LIVENESS_HOOKS_DIR),
    );
    producer
        .args(["--stdout", "--face", "baseline"])
        .current_dir(worktree);
    let regen_baseline = run_output(
        &mut producer,
        "regenerate frozen baseline from merge-base source",
    )?;

    drop(mb_volatile_cleanup);
    drop(mb_scm_facts_cleanup);
    Ok(regen_baseline)
}

fn materialize_masterplan_projection(
    tools: &MaterializerTools,
    repo_root: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(&tools.masterplan_generator)
            .args(["gen", "masterplan", "--write"])
            .current_dir(repo_root),
        "materialize masterplan projection",
    )
}

fn materialize_active_artifact_contract_graph(
    tools: &MaterializerTools,
    repo_root: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(&tools.masterplan_generator)
            .args(["gate", "validate", "active-artifact-contract"])
            .arg("--emit-graph-edges")
            .arg(repo_root.join(ACTIVE_ARTIFACT_CONTRACT_GRAPH_PATH))
            .current_dir(repo_root),
        "materialize active-artifact-contract graph",
    )
}

fn materialize_board_sync_projection(repo_root: &Path) -> Result<(), FreshnessError> {
    let source_path = repo_root.join(MASTERPLAN_PROJECTION_PATH);
    let source = read_to_string(&source_path)?;
    let masterplan = serde_json::from_str(&source).map_err(|error| {
        FreshnessError::new(format!(
            "parse masterplan projection {}: {error}",
            source_path.display()
        ))
    })?;
    let projection = render_board_sync_projection(&masterplan)
        .map_err(|error| FreshnessError::new(format!("render board-sync projection: {error}")))?;
    let output_path = repo_root.join(BOARD_SYNC_PROJECTION_PATH);
    std::fs::write(&output_path, projection).map_err(|error| {
        FreshnessError::new(format!(
            "write board-sync projection {}: {error}",
            output_path.display()
        ))
    })
}

fn materialize_masterplan_md_projection(repo_root: &Path) -> Result<(), FreshnessError> {
    let source_path = repo_root.join(MASTERPLAN_SOURCE_PATH);
    let source = std::fs::read_to_string(&source_path).map_err(|error| {
        FreshnessError::new(format!(
            "read masterplan source {}: {error}",
            source_path.display()
        ))
    })?;
    let masterplan: serde_json::Value = serde_json::from_str(&source).map_err(|error| {
        FreshnessError::new(format!(
            "parse masterplan source {}: {error}",
            source_path.display()
        ))
    })?;
    let projection = derive_masterplan_md_projection(&masterplan).map_err(|fragment| {
        FreshnessError::new(format!(
            "derive masterplan Markdown projection from {}: missing or invalid {fragment}",
            source_path.display()
        ))
    })?;
    let output_path = repo_root.join(MASTERPLAN_MD_PATH);
    std::fs::write(&output_path, projection).map_err(|error| {
        FreshnessError::new(format!(
            "write masterplan Markdown projection {}: {error}",
            output_path.display()
        ))
    })
}

fn materialize_architecture_product_graph(
    tools: &MaterializerTools,
    repo_root: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(&tools.architecture_graph_generator)
            .arg("--write")
            .current_dir(repo_root),
        "materialize architecture product graph",
    )
}

fn parse_show_output_path(
    repo_root: &Path,
    output: &str,
    target: &str,
) -> Result<PathBuf, FreshnessError> {
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
    Err(FreshnessError::new(format!(
        "buck2 --show-output did not include {target}"
    )))
}

fn parse_enforcement_liveness_corpus_paths(
    repo_root: &Path,
    output: &str,
) -> Result<EnforcementLivenessCorpusPaths, FreshnessError> {
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

fn buck_filegroup_file(output: PathBuf, file_name: &str) -> PathBuf {
    if output.is_file() {
        output
    } else {
        output.join(file_name)
    }
}

fn read_workspace_version(repo_root: &Path) -> Result<String, FreshnessError> {
    let manifest = read_to_string(&repo_root.join("Cargo.toml"))?;
    let document: toml::Value = toml::from_str(&manifest).map_err(|error| {
        FreshnessError::new(format!(
            "parse root Cargo.toml for workspace version: {error}"
        ))
    })?;
    document
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| FreshnessError::new("root Cargo.toml missing [workspace.package].version"))
}

fn read_to_string(path: &Path) -> Result<String, FreshnessError> {
    std::fs::read_to_string(path)
        .map_err(|error| FreshnessError::new(format!("read {}: {error}", path.display())))
}

fn run_output(command: &mut Command, context: &str) -> Result<String, FreshnessError> {
    let output = command
        .output()
        .map_err(|error| FreshnessError::new(format!("{context}: {error}")))?;
    if !output.status.success() {
        return Err(command_failed(context, &output));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| FreshnessError::new(format!("{context}: stdout was not UTF-8: {error}")))
}

fn run_status(command: &mut Command, context: &str) -> Result<(), FreshnessError> {
    let output = command
        .output()
        .map_err(|error| FreshnessError::new(format!("{context}: {error}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_failed(context, &output))
    }
}

fn command_failed(context: &str, output: &std::process::Output) -> FreshnessError {
    FreshnessError::new(format!(
        "{context} failed with status {}:\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn temporary_scm_facts_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-scm-facts-{}-{nanos}.json",
        std::process::id()
    ))
}

fn temporary_volatile_facts_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-scm-volatile-facts-{}-{nanos}.json",
        std::process::id()
    ))
}

fn temporary_masterplan_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-masterplan-{}-{nanos}.generated.json",
        std::process::id()
    ))
}

fn temporary_product_graph_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-product-graph-{}-{nanos}.html",
        std::process::id()
    ))
}

fn temporary_adr_census_parent_receipt_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-adr-census-parent-receipt-{}-{nanos}.generated.json",
        std::process::id()
    ))
}

fn temporary_active_artifact_contract_graph_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-active-artifact-contract-{}-{nanos}.json",
        std::process::id()
    ))
}

/// ADR-0616: the file the emitter publishes the computed merge-base sha to, so the regeneration
/// materializes exactly that source tree without recomputing the merge-base.
fn temporary_merge_base_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-merge-base-{}-{nanos}.txt",
        std::process::id()
    ))
}

/// ADR-0616: the throwaway file the regenerated frozen baseline (from the merge-base source) is
/// written to before the emitter turns it into the authoritative frozen snapshot.
fn temporary_regen_baseline_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-regen-baseline-{}-{nanos}.generated.json",
        std::process::id()
    ))
}

/// ADR-0616: the throwaway file the SECOND (determinism-twin) regeneration is written to, so the
/// emitter can assert the two regenerations project identically (the determinism canary).
fn temporary_regen_baseline_verify_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-regen-baseline-verify-{}-{nanos}.generated.json",
        std::process::id()
    ))
}

/// ADR-0616: the isolated linked worktree the merge-base SOURCE tree is checked out into.
fn temporary_worktree_path() -> PathBuf {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!(
        "oya-ci-freshness-mb-worktree-{}-{nanos}",
        std::process::id()
    ))
}

struct TempFileCleanup {
    path: PathBuf,
}

impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Best-effort removal of the merge-base source worktree (ADR-0616 PR-1): deregister it from the
/// common `.git`, delete the checkout, and prune, so a failed cross-check never leaks worktrees.
struct WorktreeCleanup {
    repo_root: PathBuf,
    path: PathBuf,
}

impl Drop for WorktreeCleanup {
    fn drop(&mut self) {
        let _ = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(["worktree", "remove", "--force"])
            .arg(&self.path)
            .output();
        let _ = std::fs::remove_dir_all(&self.path);
        let _ = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(["worktree", "prune"])
            .output();
    }
}

fn stale_face_finding(name: &str, reason: &str) -> Finding {
    Finding::new(
        FindingCode::GeneratedFaceStale,
        name,
        format!(
            "{reason}; remediation: {FACE_REMEDIATION_COMMAND}; settle protocol: {FACE_SETTLE_PROTOCOL}"
        ),
    )
}

fn package_version(
    package: &toml::map::Map<String, toml::Value>,
    workspace_version: &str,
    member_path: &str,
) -> Result<String, FreshnessError> {
    let Some(version) = package.get("version") else {
        return Err(FreshnessError::new(format!(
            "{member_path}/Cargo.toml [package] missing `version`"
        )));
    };
    if let Some(version) = version.as_str() {
        return Ok(version.to_owned());
    }
    let workspace_inherited = version
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(toml::Value::as_bool)
        == Some(true);
    if workspace_inherited {
        return Ok(workspace_version.to_owned());
    }
    Err(FreshnessError::new(format!(
        "{member_path}/Cargo.toml [package].version must be a string or `version.workspace = true`"
    )))
}

fn required_string(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
    context: &str,
) -> Result<String, FreshnessError> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| FreshnessError::new(format!("{context} missing string `{key}`")))
}

#[cfg(test)]
mod materialize_generated_faces_tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    #[cfg(unix)]
    fn write_executable(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        let file_name = path
            .file_name()
            .expect("executable file name")
            .to_string_lossy();
        let tmp = path.with_file_name(format!(".{file_name}.tmp-{}-{nanos}", std::process::id()));
        std::fs::write(&tmp, body).expect("write temporary executable");
        let mut permissions = std::fs::metadata(&tmp)
            .expect("temporary executable metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&tmp, permissions).expect("chmod temporary executable");
        std::fs::rename(&tmp, path).expect("install executable");
    }

    fn derivable_masterplan() -> serde_json::Value {
        serde_json::json!({
            "masterplan_v2": {
                "canonical_plan_authority": {
                    "path": "/specs/masterplan.json",
                    "live_work_item_id_space": {
                        "id_prefix": "MPV2-",
                        "numeric_width": 4
                    }
                },
                "surface_dispositions": [
                    {
                        "path": "/specs/legacy-plan.json",
                        "disposition": "absorbed"
                    }
                ]
            }
        })
    }

    fn write_masterplan(root: &Path, masterplan: &serde_json::Value) {
        std::fs::create_dir_all(root.join("specs")).expect("create specs dir");
        std::fs::write(
            root.join(MASTERPLAN_SOURCE_PATH),
            serde_json::to_vec(masterplan).expect("serialize masterplan fixture"),
        )
        .expect("write masterplan fixture");
    }

    #[test]
    fn masterplan_markdown_materialization_fails_closed_on_read_error() {
        let root = temp_root("masterplan-markdown-read-error");

        let error = materialize_masterplan_md_projection(&root)
            .expect_err("missing canonical source must fail closed");

        assert!(error.to_string().contains("read masterplan source"));
    }

    #[test]
    fn masterplan_markdown_materialization_fails_closed_on_parse_error() {
        let root = temp_root("masterplan-markdown-parse-error");
        std::fs::create_dir_all(root.join("specs")).expect("create specs dir");
        std::fs::write(root.join(MASTERPLAN_SOURCE_PATH), "{")
            .expect("write malformed masterplan fixture");

        let error = materialize_masterplan_md_projection(&root)
            .expect_err("malformed canonical source must fail closed");

        assert!(error.to_string().contains("parse masterplan source"));
    }

    #[test]
    fn masterplan_markdown_materialization_fails_closed_on_derivation_error() {
        let root = temp_root("masterplan-markdown-derivation-error");
        write_masterplan(&root, &serde_json::json!({}));

        let error = materialize_masterplan_md_projection(&root)
            .expect_err("underivable canonical source must fail closed");

        assert!(
            error
                .to_string()
                .contains("derive masterplan Markdown projection")
        );
        assert!(error.to_string().contains("masterplan_v2"));
    }

    #[test]
    fn masterplan_markdown_materialization_fails_closed_on_write_error() {
        let root = temp_root("masterplan-markdown-write-error");
        write_masterplan(&root, &derivable_masterplan());
        std::fs::create_dir_all(root.join(MASTERPLAN_MD_PATH))
            .expect("create directory at projection path");

        let error = materialize_masterplan_md_projection(&root)
            .expect_err("unwritable projection path must fail closed");

        assert!(
            error
                .to_string()
                .contains("write masterplan Markdown projection")
        );
    }

    #[test]
    fn board_sync_materialization_fails_closed_on_read_error() {
        let root = temp_root("board-sync-read-error");

        let error = materialize_board_sync_projection(&root)
            .expect_err("missing masterplan projection must fail closed");

        assert!(error.to_string().contains("read"));
        assert!(!root.join(BOARD_SYNC_PROJECTION_PATH).exists());
    }

    #[test]
    fn board_sync_materialization_fails_closed_on_parse_error() {
        let root = temp_root("board-sync-parse-error");
        std::fs::create_dir_all(
            root.join(MASTERPLAN_PROJECTION_PATH)
                .parent()
                .expect("masterplan projection parent"),
        )
        .expect("create projection dir");
        std::fs::write(root.join(MASTERPLAN_PROJECTION_PATH), "{")
            .expect("write malformed masterplan projection");

        let error = materialize_board_sync_projection(&root)
            .expect_err("malformed masterplan projection must fail closed");

        assert!(error.to_string().contains("parse masterplan projection"));
        assert!(!root.join(BOARD_SYNC_PROJECTION_PATH).exists());
    }

    #[test]
    fn board_sync_materialization_fails_closed_on_write_error() {
        let root = temp_root("board-sync-write-error");
        std::fs::create_dir_all(
            root.join(MASTERPLAN_PROJECTION_PATH)
                .parent()
                .expect("masterplan projection parent"),
        )
        .expect("create projection dir");
        std::fs::write(
            root.join(MASTERPLAN_PROJECTION_PATH),
            serde_json::to_vec(&serde_json::json!({
                "milestones": [{
                    "milestone": "M-ALPHA",
                    "adrs": [{
                        "deliverables": [{
                            "id": "A-1",
                            "description": "first item",
                            "status": "declared"
                        }]
                    }]
                }]
            }))
            .expect("serialize masterplan projection fixture"),
        )
        .expect("write masterplan projection fixture");
        std::fs::create_dir(root.join(BOARD_SYNC_PROJECTION_PATH))
            .expect("create directory at board projection path");

        let error = materialize_board_sync_projection(&root)
            .expect_err("unwritable board projection path must fail closed");

        assert!(error.to_string().contains("write board-sync projection"));
    }

    #[test]
    fn parse_materialize_generated_faces_args_defaults_to_repo_root_dot() {
        let parsed = parse_materialize_generated_faces_args(Vec::new())
            .expect("empty args should use repository root default");

        assert_eq!(parsed.repo_root, PathBuf::from("."));
        assert_eq!(parsed.retirement, None);
    }

    #[test]
    fn parse_materialize_generated_faces_args_accepts_repo_root() {
        let parsed = parse_materialize_generated_faces_args(vec![
            "--repo-root".to_owned(),
            "/tmp/oyatie".to_owned(),
        ])
        .expect("parse explicit repository root");

        assert_eq!(parsed.repo_root, PathBuf::from("/tmp/oyatie"));
        assert_eq!(parsed.retirement, None);
    }

    #[test]
    fn parse_materialize_generated_faces_args_accepts_exact_retirement_transport() {
        let parsed = parse_materialize_generated_faces_args(vec![
            "--retirement-control-plane".to_owned(),
            "registry/history-only-retirement-control-plane.json".to_owned(),
            "--retirement-facts-out".to_owned(),
            "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json".to_owned(),
            "--protected-base-commit".to_owned(),
            "1111111111111111111111111111111111111111".to_owned(),
            "--candidate-commit".to_owned(),
            "2222222222222222222222222222222222222222".to_owned(),
        ])
        .expect("parse exact retirement transport");

        assert_eq!(
            parsed.retirement,
            Some(RetirementMaterializeArgs {
                control_plane_path: "registry/history-only-retirement-control-plane.json"
                    .to_owned(),
                facts_out: PathBuf::from(
                    "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json",
                ),
                protected_base_commit: "1111111111111111111111111111111111111111".to_owned(),
                candidate_commit: "2222222222222222222222222222222222222222".to_owned(),
            })
        );
    }

    #[test]
    fn parse_materialize_generated_faces_args_rejects_partial_retirement_transport() {
        let error = parse_materialize_generated_faces_args(vec![
            "--retirement-control-plane".to_owned(),
            "registry/history-only-retirement-control-plane.json".to_owned(),
        ])
        .expect_err("partial retirement transport must fail closed");

        assert!(error.to_string().contains("all-or-none"));
    }

    #[test]
    fn retirement_transport_is_appended_only_when_explicit() {
        let retirement = RetirementMaterializeArgs {
            control_plane_path: "registry/history-only-retirement-control-plane.json".to_owned(),
            facts_out: PathBuf::from(
                "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json",
            ),
            protected_base_commit: "1111111111111111111111111111111111111111".to_owned(),
            candidate_commit: "2222222222222222222222222222222222222222".to_owned(),
        };
        let mut candidate = Command::new("emitter");
        append_retirement_materialization_args(&mut candidate, Some(&retirement));
        let candidate_args = candidate
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(candidate_args.contains(&"--retirement-control-plane".to_owned()));
        assert!(candidate_args.contains(&retirement.candidate_commit));

        let mut frozen = Command::new("emitter");
        append_retirement_materialization_args(&mut frozen, None);
        assert_eq!(frozen.get_args().count(), 0);
    }

    #[test]
    fn multi_commit_local_materialization_does_not_infer_head_parent_as_protected_base() {
        let root = temp_root("retirement-auto-materialization");
        std::fs::create_dir_all(root.join("registry")).expect("create registry");
        std::fs::write(root.join(RETIREMENT_CONTROL_PLANE_PATH), "{}")
            .expect("write control-plane marker");

        // A control-plane file may be present in a normal multi-commit contributor checkout.
        // That topology has no event-bound protected base, and must never turn HEAD^1 into one.
        assert_eq!(effective_retirement_materialization(&root, None), None);
    }

    #[test]
    fn parse_materialize_generated_faces_args_rejects_unknown_flag() {
        let error = parse_materialize_generated_faces_args(vec!["--settle".to_owned()])
            .expect_err("materializer must not inherit face-settle modes");

        assert!(error.to_string().contains("unknown argument"));
        assert!(
            error
                .to_string()
                .contains("oya-cloud-ci-materialize-generated-faces")
        );
    }

    #[test]
    fn producer_regeneration_commands_declare_enforcement_liveness_corpus() {
        let mut command = Command::new("/tmp/producer");
        let corpus = EnforcementLivenessCorpusPaths {
            claude_settings: PathBuf::from("/buck/declared/settings-json/settings.json"),
            codex_hooks: PathBuf::from("/buck/declared/hooks-json/hooks.json"),
            hooks_dir: PathBuf::from("/buck/declared/top-level-hook-scripts"),
        };
        append_enforcement_liveness_corpus_args(&mut command, &corpus);

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert!(args.windows(2).any(|pair| {
            pair == [
                "--enforcement-liveness-claude-settings",
                "/buck/declared/settings-json/settings.json",
            ]
        }));
        assert!(args.windows(2).any(|pair| {
            pair == [
                "--enforcement-liveness-codex-hooks",
                "/buck/declared/hooks-json/hooks.json",
            ]
        }));
        assert!(args.windows(2).any(|pair| {
            pair == [
                "--enforcement-liveness-hooks-dir",
                "/buck/declared/top-level-hook-scripts",
            ]
        }));
    }

    #[test]
    fn explicit_corpus_path_appender_preserves_paths() {
        let mut command = Command::new("/tmp/producer");
        append_enforcement_liveness_corpus_paths(
            &mut command,
            Path::new("/repo/.claude/settings.json"),
            Path::new("/repo/.codex/hooks.json"),
            Path::new("/repo/tools/hooks"),
        );

        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert!(args.windows(2).any(|pair| {
            pair == [
                "--enforcement-liveness-claude-settings",
                "/repo/.claude/settings.json",
            ]
        }));
        assert!(args.windows(2).any(|pair| {
            pair == [
                "--enforcement-liveness-codex-hooks",
                "/repo/.codex/hooks.json",
            ]
        }));
        assert!(
            args.windows(2)
                .any(|pair| { pair == ["--enforcement-liveness-hooks-dir", "/repo/tools/hooks",] })
        );
    }

    #[test]
    fn parse_show_output_path_resolves_materializer_targets() {
        let output = "\
root//ci/facade/scm-facts-snapshot:ci-scm-facts-snapshot buck-out/v2/gen/emitter\n\
root//ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin /tmp/producer\n\
root//tools/oya-reorg-codemod-app:oya-reorg-codemod buck-out/v2/gen/codemod\n\
root//marketplace/facade/dev-cli:oya /tmp/oya\n\
root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator buck-out/v2/gen/architecture-graph\n\
root//.claude:settings-json buck-out/v2/gen/.claude/__settings-json__/settings-json\n\
root//.codex:hooks-json buck-out/v2/gen/.codex/__hooks-json__/hooks-json\n\
root//tools/hooks:top-level-hook-scripts buck-out/v2/gen/tools/hooks/__top-level-hook-scripts__/top-level-hook-scripts\n\
";

        let emitter =
            parse_show_output_path(Path::new("/repo"), output, EMITTER_TARGET).expect("emitter");
        let producer =
            parse_show_output_path(Path::new("/repo"), output, PRODUCER_TARGET).expect("producer");
        let codemod =
            parse_show_output_path(Path::new("/repo"), output, CODEMOD_TARGET).expect("codemod");
        let masterplan_generator =
            parse_show_output_path(Path::new("/repo"), output, MASTERPLAN_GENERATOR_TARGET)
                .expect("masterplan generator");
        let architecture_graph_generator = parse_show_output_path(
            Path::new("/repo"),
            output,
            ARCHITECTURE_GRAPH_GENERATOR_TARGET,
        )
        .expect("architecture graph generator");

        assert_eq!(emitter, PathBuf::from("/repo/buck-out/v2/gen/emitter"));
        assert_eq!(producer, PathBuf::from("/tmp/producer"));
        assert_eq!(codemod, PathBuf::from("/repo/buck-out/v2/gen/codemod"));
        assert_eq!(masterplan_generator, PathBuf::from("/tmp/oya"));
        assert_eq!(
            architecture_graph_generator,
            PathBuf::from("/repo/buck-out/v2/gen/architecture-graph")
        );
        let corpus =
            parse_enforcement_liveness_corpus_paths(Path::new("/repo"), output).expect("corpus");
        assert_eq!(
            corpus.claude_settings,
            PathBuf::from(
                "/repo/buck-out/v2/gen/.claude/__settings-json__/settings-json/settings.json"
            )
        );
        assert_eq!(
            corpus.codex_hooks,
            PathBuf::from("/repo/buck-out/v2/gen/.codex/__hooks-json__/hooks-json/hooks.json")
        );
        assert_eq!(
            corpus.hooks_dir,
            PathBuf::from(
                "/repo/buck-out/v2/gen/tools/hooks/__top-level-hook-scripts__/top-level-hook-scripts"
            )
        );
    }

    #[test]
    fn controller_projection_faces_are_not_pr_owned_face_paths() {
        let root = temp_root("oya-product-graph-controller-owned");
        std::fs::create_dir_all(root.join("registry")).expect("create registry dir");
        std::fs::write(
            root.join(CONTROL_PLANE_MANIFEST),
            serde_json::json!({
                "artifacts": [
                    {
                        "path": "ci/facade/artifact-inventory-registry/adr-census-parent-receipt.generated.json",
                        "materialization_mode": NOT_TRACKED_IN_GIT_MODE
                    },
                    {
                        "path": MASTERPLAN_PROJECTION_PATH,
                        "materialization_mode": NOT_TRACKED_IN_GIT_MODE
                    },
                    {
                        "path": BOARD_SYNC_PROJECTION_PATH,
                        "materialization_mode": NOT_TRACKED_IN_GIT_MODE
                    },
                    {
                        "path": ARCHITECTURE_PRODUCT_GRAPH_PATH,
                        "materialization_mode": MAIN_BRANCH_MATERIALIZED_MODE
                    },
                    {
                        "path": "elsewhere/product-graph.html",
                        "materialization_mode": MAIN_BRANCH_MATERIALIZED_MODE
                    },
                    {
                        "path": ACTIVE_ARTIFACT_CONTRACT_GRAPH_PATH,
                        "materialization_mode": NOT_TRACKED_IN_GIT_MODE
                    }
                ]
            })
            .to_string(),
        )
        .expect("write manifest");

        let non_pr_owned = read_decommitted_face_names(&root);
        let generated_paths = generated_face_paths();
        let pr_owned_paths = pr_owned_generated_face_paths(&non_pr_owned);

        assert!(non_pr_owned.contains(ADR_CENSUS_PARENT_RECEIPT_FACE));
        assert!(non_pr_owned.contains(MASTERPLAN_PROJECTION_FACE));
        assert!(non_pr_owned.contains(BOARD_SYNC_PROJECTION_FACE));
        assert!(non_pr_owned.contains(ARCHITECTURE_PRODUCT_GRAPH_FACE));
        assert!(non_pr_owned.contains(ACTIVE_ARTIFACT_CONTRACT_GRAPH_FACE));
        assert!(!generated_paths.contains(&MASTERPLAN_PROJECTION_PATH.to_owned()));
        assert!(
            !generated_paths.contains(&format!("{FACES_DIR}/{ADR_CENSUS_PARENT_RECEIPT_FACE}"))
        );
        assert!(!generated_paths.contains(&ARCHITECTURE_PRODUCT_GRAPH_PATH.to_owned()));
        assert!(!pr_owned_paths.contains(&MASTERPLAN_PROJECTION_PATH.to_owned()));
        assert!(!pr_owned_paths.contains(&ARCHITECTURE_PRODUCT_GRAPH_PATH.to_owned()));
    }

    #[test]
    fn read_committed_generated_faces_includes_architecture_projection_faces() {
        let root = temp_root("oya-committed-faces");
        std::fs::create_dir_all(root.join(FACES_DIR)).expect("create faces dir");
        std::fs::create_dir_all(root.join("docs/architecture")).expect("create docs dir");
        std::fs::create_dir_all(root.join("docs/machine-readable"))
            .expect("create machine-readable dir");
        std::fs::write(root.join(FACES_DIR).join(SCM_FACTS_FACE), "scm\n").expect("write scm face");
        std::fs::write(root.join(ARCHITECTURE_PRODUCT_GRAPH_PATH), "graph\n")
            .expect("write product graph");
        std::fs::write(root.join(MASTERPLAN_PROJECTION_PATH), "masterplan\n")
            .expect("write masterplan projection");
        std::fs::write(root.join(BOARD_SYNC_PROJECTION_PATH), "board\n")
            .expect("write board projection");

        let faces = read_committed_generated_faces(&root).expect("read committed generated faces");

        assert!(faces.contains(&(SCM_FACTS_FACE.to_owned(), "scm\n".to_owned())));
        assert!(faces.contains(&(
            ARCHITECTURE_PRODUCT_GRAPH_FACE.to_owned(),
            "graph\n".to_owned()
        )));
        assert!(faces.contains(&(
            MASTERPLAN_PROJECTION_FACE.to_owned(),
            "masterplan\n".to_owned()
        )));
        assert!(faces.contains(&(BOARD_SYNC_PROJECTION_FACE.to_owned(), "board\n".to_owned())));
    }

    #[cfg(unix)]
    #[test]
    fn regenerated_architecture_product_graph_uses_temporary_output() {
        let root = temp_root("oya-regenerate-product-graph");
        std::fs::create_dir_all(root.join("bin")).expect("create bin dir");
        std::fs::create_dir_all(root.join("docs/architecture")).expect("create docs dir");
        std::fs::create_dir_all(root.join("docs/machine-readable")).expect("create docs data dir");
        std::fs::write(
            root.join(ARCHITECTURE_PRODUCT_GRAPH_PATH),
            "committed graph\n",
        )
        .expect("write committed graph");
        std::fs::write(
            root.join("docs/machine-readable/masterplan.generated.json"),
            "committed masterplan\n",
        )
        .expect("write committed masterplan");
        let masterplan_generator = root.join("bin/oya");
        write_executable(
            &masterplan_generator,
            r#"#!/bin/sh
set -eu
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output) shift; out="$1" ;;
  esac
  shift || true
done
test -n "$out"
mkdir -p "$(dirname "$out")"
printf '{"milestones":[{"milestone":"M-test","adrs":[{"deliverables":[{"id":"D-1","description":"test","status":"declared"}]}]}],"adr_count":0,"deliverable_count":1,"generator":"test"}\n' > "$out"
"#,
        );
        let generator = root.join("bin/architecture-graph");
        write_executable(
            &generator,
            r#"#!/bin/sh
set -eu
out=""
masterplan=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output) shift; out="$1" ;;
    --masterplan) shift; masterplan="$1" ;;
  esac
  shift || true
done
test -n "$out"
test -n "$masterplan"
grep -q '"milestones"' "$masterplan"
printf 'fresh graph\n' > "$out"
"#,
        );
        let tools = FaceTools {
            emitter: PathBuf::from("/unused-emitter"),
            producer: PathBuf::from("/unused-producer"),
            masterplan_generator,
            architecture_graph_generator: generator,
            enforcement_liveness_corpus: EnforcementLivenessCorpusPaths {
                claude_settings: root.join("buck/declared/settings.json"),
                codex_hooks: root.join("buck/declared/hooks.json"),
                hooks_dir: root.join("buck/declared/hooks"),
            },
        };

        let regenerated = regenerate_architecture_projection_faces(&tools, &root)
            .expect("regenerate architecture projection faces");

        assert_eq!(
            regenerated,
            vec![
                (
                    MASTERPLAN_PROJECTION_FACE.to_owned(),
                    "{\"milestones\":[{\"milestone\":\"M-test\",\"adrs\":[{\"deliverables\":[{\"id\":\"D-1\",\"description\":\"test\",\"status\":\"declared\"}]}]}],\"adr_count\":0,\"deliverable_count\":1,\"generator\":\"test\"}\n"
                        .to_owned(),
                ),
                (
                    BOARD_SYNC_PROJECTION_FACE.to_owned(),
                    "{\n  \"_generated\": \"GENERATED by `oya gen board-sync` from masterplan deliverables. Do not hand-edit.\",\n  \"github_projection\": {\n    \"exclusive_label_scopes\": [\n      \"state\",\n      \"owner\",\n      \"deliverable\",\n      \"milestone\"\n    ],\n    \"issue_identity\": \"deliverable_id\"\n  },\n  \"issues\": [\n    {\n      \"body\": \"Generated from masterplan deliverable `D-1`.\\n\\ntest\\n\\n<!-- oya-board-sync:D-1 -->\\n\",\n      \"deliverable_id\": \"D-1\",\n      \"labels\": [\n        \"state/declared\",\n        \"owner/unassigned\",\n        \"deliverable/d-1\",\n        \"milestone/m-test\"\n      ],\n      \"title\": \"D-1: test\"\n    }\n  ]\n}\n".to_owned(),
                ),
                (
                ARCHITECTURE_PRODUCT_GRAPH_FACE.to_owned(),
                "fresh graph\n".to_owned()
                ),
            ]
        );
        assert_eq!(
            std::fs::read_to_string(root.join(ARCHITECTURE_PRODUCT_GRAPH_PATH))
                .expect("committed graph"),
            "committed graph\n"
        );
        assert_eq!(
            std::fs::read_to_string(root.join(MASTERPLAN_PROJECTION_PATH))
                .expect("committed masterplan"),
            "committed masterplan\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn materializer_invokes_all_controller_projection_generators() {
        let root = temp_root("oya-materialize-faces");
        std::fs::create_dir_all(root.join("bin")).expect("create bin dir");
        let masterplan = derivable_masterplan();
        write_masterplan(&root, &masterplan);
        let log = root.join("calls.log");
        let log_path = log.display();

        let codemod = root.join("bin/codemod");
        write_executable(
            &codemod,
            &format!(
                r#"#!/bin/sh
set -eu
printf 'codemod %s\n' "$*" >> "{log_path}"
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --out) shift; out="$1" ;;
  esac
  shift || true
done
test -n "$out"
mkdir -p "$(dirname "$out")"
printf '{{"moves":[]}}\n' > "$out"
"#
            ),
        );

        let emitter = root.join("bin/emitter");
        write_executable(
            &emitter,
            &format!(
                r#"#!/bin/sh
set -eu
printf 'emitter %s\n' "$*" >> "{log_path}"
out=""
mbout=""
censusout=""
census=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --out) shift; out="$1" ;;
    --merge-base-out) shift; mbout="$1" ;;
    --adr-census-parent-receipt) census=true ;;
    --adr-census-parent-receipt-out) shift; censusout="$1" ;;
  esac
  shift || true
done
if [ "$census" = true ]; then
  test -n "$censusout"
  mkdir -p "$(dirname "$censusout")"
  printf '{{"fixed":"receipt"}}\n' > "$censusout"
  exit 0
fi
test -n "$out"
mkdir -p "$(dirname "$out")"
printf '{{"facts":[]}}\n' > "$out"
# ADR-0616 PR-1: publish the merge-base sha the cross-check materializes.
if [ -n "$mbout" ]; then git rev-parse HEAD > "$mbout"; fi
"#
            ),
        );

        let producer = root.join("bin/producer");
        write_executable(
            &producer,
            &format!(
                r#"#!/bin/sh
set -eu
printf 'producer %s\n' "$*" >> "{log_path}"
# ADR-0616 PR-1: the frozen-baseline regeneration emits the baseline face to stdout.
face=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --face) shift; face="$1" ;;
  esac
  shift || true
done
if [ "$face" = "baseline" ]; then printf '{{"gates":{{}}}}\n'; fi
"#
            ),
        );

        let masterplan_generator = root.join("bin/oya");
        write_executable(
            &masterplan_generator,
            &format!(
                r#"#!/bin/sh
set -eu
printf 'oya %s\n' "$*" >> "{log_path}"
if [ "$1" = "gen" ]; then
  test "$#" -eq 3
  test "$2" = "masterplan"
  test "$3" = "--write"
  mkdir -p docs/machine-readable
  printf '{{"milestones":[{{"milestone":"M-test","adrs":[{{"deliverables":[{{"id":"D-1","description":"test","status":"declared"}}]}}]}}],"adr_count":0,"deliverable_count":1,"generator":"test"}}\n' > docs/machine-readable/masterplan.generated.json
elif [ "$1" = "gate" ]; then
  test "$2" = "validate"
  test "$3" = "active-artifact-contract"
  shift 3
  test "$1" = "--emit-graph-edges"
  mkdir -p "$(dirname "$2")"
  printf '{{"edges":[]}}\n' > "$2"
else
  exit 2
fi
"#
            ),
        );

        let architecture_graph_generator = root.join("bin/architecture-graph");
        write_executable(
            &architecture_graph_generator,
            &format!(
                r#"#!/bin/sh
set -eu
printf 'architecture %s\n' "$*" >> "{log_path}"
test "$#" -eq 1
test "$1" = "--write"
mkdir -p docs/architecture
printf 'generated dashboard\n' > docs/architecture/product-graph.html
"#
            ),
        );

        let tools = MaterializerTools {
            emitter,
            producer,
            codemod,
            masterplan_generator,
            architecture_graph_generator,
            enforcement_liveness_corpus: EnforcementLivenessCorpusPaths {
                claude_settings: root.join("buck/declared/settings.json"),
                codex_hooks: root.join("buck/declared/hooks.json"),
                hooks_dir: root.join("buck/declared/hooks"),
            },
        };

        // ADR-0616 PR-1: the frozen-baseline regen cross-check materializes a merge-base worktree,
        // so the fixture root must be a real git repo with a committed HEAD (the fake emitter
        // publishes its sha as the merge-base).
        init_git_repo(&root);

        materialize_generated_faces_with_tools(&tools, &root, None)
            .expect("materialize faces and architecture product graph");

        let calls = std::fs::read_to_string(&log).expect("read call log");
        let codemod_pos = calls.find("codemod manifest").expect("codemod call");
        let emitter_pos = calls.find("emitter --repo-root").expect("emitter call");
        let census_pos = calls
            .find("--adr-census-parent-receipt --adr-census-parent-receipt-out")
            .expect("fixed census receipt call");
        let producer_pos = calls.rfind("producer --repo-root").expect("producer call");
        let active_graph_pos = calls
            .find("oya gate validate active-artifact-contract --emit-graph-edges")
            .expect("active-artifact graph generator call");
        let masterplan_pos = calls
            .find("oya gen masterplan --write")
            .expect("masterplan generator call");
        let architecture_pos = calls
            .find("architecture --write")
            .expect("architecture generator call");
        assert!(codemod_pos < emitter_pos);
        assert!(emitter_pos < census_pos);
        assert!(census_pos < producer_pos);
        assert!(producer_pos < active_graph_pos);
        assert!(active_graph_pos < masterplan_pos);
        assert!(masterplan_pos < architecture_pos);
        assert_eq!(
            std::fs::read_to_string(root.join(ACTIVE_ARTIFACT_CONTRACT_GRAPH_PATH))
                .expect("active artifact contract graph materialized"),
            "{\"edges\":[]}\n"
        );
        assert!(calls.contains(&format!(
            "--enforcement-liveness-claude-settings {}",
            root.join("buck/declared/settings.json").display()
        )));
        assert!(calls.contains(&format!(
            "--enforcement-liveness-codex-hooks {}",
            root.join("buck/declared/hooks.json").display()
        )));
        assert!(calls.contains(&format!(
            "--enforcement-liveness-hooks-dir {}",
            root.join("buck/declared/hooks").display()
        )));
        assert!(!calls.contains(&format!(
            "--enforcement-liveness-claude-settings {}",
            root.join(ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS).display()
        )));
        assert_eq!(
            std::fs::read_to_string(root.join(FACES_DIR).join(ADR_CENSUS_PARENT_RECEIPT_FACE))
                .expect("fixed census receipt materialized"),
            "{\"fixed\":\"receipt\"}\n"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("docs/machine-readable/masterplan.generated.json"))
                .expect("masterplan materialized"),
            "{\"milestones\":[{\"milestone\":\"M-test\",\"adrs\":[{\"deliverables\":[{\"id\":\"D-1\",\"description\":\"test\",\"status\":\"declared\"}]}]}],\"adr_count\":0,\"deliverable_count\":1,\"generator\":\"test\"}\n"
        );
        assert!(
            std::fs::read_to_string(root.join(BOARD_SYNC_PROJECTION_PATH))
                .expect("board-sync materialized")
                .contains("\"deliverable_id\": \"D-1\"")
        );
        assert_eq!(
            std::fs::read_to_string(root.join(ci_cross_artifact_agreement::MASTERPLAN_MD_PATH))
                .expect("masterplan Markdown materialized"),
            ci_cross_artifact_agreement::derive_masterplan_md_projection(&masterplan)
                .expect("derive expected masterplan Markdown")
        );
        assert_eq!(
            std::fs::read_to_string(root.join("docs/architecture/product-graph.html"))
                .expect("product graph materialized"),
            "generated dashboard\n"
        );
    }

    #[cfg(unix)]
    fn init_git_repo(root: &Path) {
        let run = |args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .expect("run git");
            assert!(
                output.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        run(&["init", "-q"]);
        run(&["config", "user.name", "Oyatie Test"]);
        run(&["config", "user.email", "oyatie-test@example.com"]);
        run(&["config", "commit.gpgsign", "false"]);
        run(&["add", "-A"]);
        run(&["commit", "-q", "-m", "seed"]);
    }

    /// ADR-0616: a producer that fails to regenerate the frozen baseline from the merge-base source
    /// is a HARD ERROR (fail-closed) — never a silent fallback, which (with the frozen reference
    /// de-committed) would empty-frozen-deadlock the firewall.
    #[cfg(unix)]
    #[test]
    fn frozen_baseline_regen_is_fail_closed_on_producer_failure() {
        let root = temp_root("oya-regen-fail-closed");
        std::fs::create_dir_all(root.join("bin")).expect("create bin dir");
        let emitter = root.join("bin/emitter");
        write_executable(
            &emitter,
            r#"#!/bin/sh
set -eu
out=""
while [ "$#" -gt 0 ]; do case "$1" in --out) shift; out="$1" ;; esac; shift || true; done
test -n "$out"; mkdir -p "$(dirname "$out")"
printf '{"schema":"oya-ci/scm-facts/v2","tracked_paths":[]}\n' > "$out"
"#,
        );
        // The producer FAILS the baseline regeneration.
        let producer = root.join("bin/producer");
        write_executable(
            &producer,
            r#"#!/bin/sh
echo "boom: cannot regenerate the baseline" >&2
exit 1
"#,
        );

        let tools = regen_tools(&root, emitter, producer);
        let error = regenerate_frozen_baseline_from_merge_base_source(&tools, &root)
            .expect_err("a producer regen failure must be a hard error (fail-closed)");
        assert!(
            error
                .to_string()
                .contains("regenerate frozen baseline from merge-base source"),
            "{error}"
        );
    }

    /// ADR-0616 blob-independence: the regeneration reads only SOURCE. The worktree carries NO
    /// `gate-baseline.generated.json`, yet the regeneration still produces a baseline, and the
    /// producer command never references the committed blob path.
    #[cfg(unix)]
    #[test]
    fn frozen_baseline_regen_is_blob_independent() {
        let root = temp_root("oya-regen-blob-independent");
        std::fs::create_dir_all(root.join("bin")).expect("create bin dir");
        let log = root.join("producer.log");
        let log_path = log.display();
        let emitter = root.join("bin/emitter");
        write_executable(
            &emitter,
            r#"#!/bin/sh
set -eu
out=""
while [ "$#" -gt 0 ]; do case "$1" in --out) shift; out="$1" ;; esac; shift || true; done
test -n "$out"; mkdir -p "$(dirname "$out")"
printf '{"schema":"oya-ci/scm-facts/v2","tracked_paths":[]}\n' > "$out"
"#,
        );
        // The producer asserts the committed blob is ABSENT from its source tree, yet still
        // produces a baseline (it PRODUCES `--face baseline`, never reads the committed blob).
        let producer = root.join("bin/producer");
        write_executable(
            &producer,
            &format!(
                r#"#!/bin/sh
set -eu
printf 'producer %s\n' "$*" >> "{log_path}"
face=""
while [ "$#" -gt 0 ]; do case "$1" in --face) shift; face="$1" ;; esac; shift || true; done
test "$face" = "baseline"
test ! -e "ci/facade/artifact-inventory-registry/gate-baseline.generated.json"
printf '{{"gates":{{}}}}\n'
"#
            ),
        );

        let tools = regen_tools(&root, emitter, producer);
        let baseline = regenerate_frozen_baseline_from_merge_base_source(&tools, &root).expect(
            "the regeneration must succeed with the committed blob absent (blob-independent)",
        );
        assert!(
            baseline.contains("gates"),
            "regeneration produced a baseline: {baseline}"
        );

        let calls = std::fs::read_to_string(&log).expect("read producer log");
        assert!(
            calls.contains("--face baseline"),
            "the regeneration must run the producer's baseline face: {calls}"
        );
        assert!(
            !calls.contains("gate-baseline.generated.json"),
            "the regeneration must NEVER reference the committed blob path: {calls}"
        );
    }

    /// Materializer tools with only the emitter + producer wired (the frozen-baseline regeneration
    /// uses only those two); every other tool is an unused placeholder.
    #[cfg(unix)]
    fn regen_tools(root: &Path, emitter: PathBuf, producer: PathBuf) -> MaterializerTools {
        MaterializerTools {
            emitter,
            producer,
            codemod: PathBuf::from("/unused-codemod"),
            masterplan_generator: PathBuf::from("/unused-masterplan"),
            architecture_graph_generator: PathBuf::from("/unused-architecture"),
            enforcement_liveness_corpus: EnforcementLivenessCorpusPaths {
                claude_settings: root.join(".claude/settings.json"),
                codex_hooks: root.join(".codex/hooks.json"),
                hooks_dir: root.join("tools/hooks"),
            },
        }
    }
}
