#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use oya_workspace_members_kernel::resolve_member_dirs;

mod rust_toolchain_drift;
pub use rust_toolchain_drift::{evaluate_rust_toolchain_drift, read_pinned_rust_toolchain};

pub const LOCK_REMEDIATION_COMMAND: &str = "cargo metadata >/dev/null";
pub const FACE_REMEDIATION_COMMAND: &str = "buck2 run //cloud/cloud-ci/gates/oya-cloud-ci-freshness-app:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .";
pub const FACE_SETTLE_PROTOCOL: &str = "commit content changes first; faces regenerate from the TRACKED TREE STATE (ADR-0552: committed faces carry no history-derived data, so commit ids never enter them); never mix content and regenerated faces in one commit; then run the materialize command; commit only PR-owned generated face diffs; controller-owned generated faces are materialized by cloud-ci/integration controllers, not contributor PRs; then run oya-cloud-ci-face-settle --verify as the LAST step before EVERY push";
pub const FACE_VERIFY_REMEDIATION_COMMAND: &str = "oya-cloud-ci-face-settle --settle --commit";
pub const FACE_SETTLE_COMMIT_COMMAND: &str =
    "git commit -S -m \"chore: settle generated cloud-ci faces\"";
const FACE_SETTLE_COMMIT_MESSAGE: &str = "chore: settle generated cloud-ci faces";
const FACES_DIR: &str = "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app";
const SCM_FACTS_FACE: &str = "scm-facts.generated.json";
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
const GENERATED_FACE_PATHS: [&str; 7] = [
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/scm-facts.generated.json",
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/accounting-registry.generated.json",
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/ttl-policy.generated.json",
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/decision-crosswalk.generated.json",
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-inventory.generated.json",
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/enforcement-liveness.generated.json",
    "cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json",
];
const EMITTER_TARGET: &str =
    "//cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app";
const PRODUCER_TARGET: &str = "//cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin";
const CODEMOD_TARGET: &str = "//tools/oya-reorg-codemod-app:oya-reorg-codemod";
const MOVE_MANIFEST_FACE: &str = "specs/reorg/move-manifest.generated.json";
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
    materialize_move_manifest(&tools, repo_root)?;
    let scm_facts = repo_root.join(FACES_DIR).join(SCM_FACTS_FACE);
    emit_materialized_scm_facts(&tools, repo_root, &scm_facts)?;
    let mut command = Command::new(&tools.producer);
    command
        .args(["--repo-root"])
        .arg(repo_root)
        .args(["--scm-facts"])
        .arg(&scm_facts);
    add_enforcement_liveness_corpus_args(&mut command, repo_root);
    run_status(
        command.current_dir(repo_root),
        "materialize generated accounting faces",
    )
}

pub fn parse_materialize_generated_faces_args(
    args: Vec<String>,
) -> Result<MaterializeGeneratedFacesArgs, FreshnessError> {
    let mut repo_root = PathBuf::from(".");
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
    Ok(MaterializeGeneratedFacesArgs { repo_root })
}

pub fn materialize_generated_faces_usage() -> &'static str {
    "usage: oya-cloud-ci-materialize-generated-faces [--repo-root <path>]"
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
    faces.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(faces)
}

/// Read the file names of non-PR-owned faces from the generated-artifact control-plane manifest.
/// A declared artifact whose `materialization_mode` is `not-tracked-in-git` OR
/// `main-branch-materialized` AND whose `path` EXACTLY equals one of this gate's canonical
/// generated-face paths ([`GENERATED_FACE_PATHS`]) is returned by its file basename (e.g.
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
        // Scope strictly to this gate's faces by CANONICAL FULL PATH, never basename. Matching on
        // basename would let a candidate-controlled manifest row at a non-canonical path (e.g.
        // `anything/scm-facts.generated.json`) collapse to a committed face's name and silently
        // retire that committed face's byte-parity check. An unrelated de-commit-class artifact
        // elsewhere in the manifest must not change freshness behavior here.
        if GENERATED_FACE_PATHS.contains(&path) {
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
    let regenerated = regenerate_producer_faces(&tools, repo_root, &scm_facts)?;
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
    let first = regenerate_producer_faces(&tools, repo_root, &scm_facts)?;
    let second = regenerate_producer_faces(&tools, repo_root, &scm_facts)?;
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

fn add_enforcement_liveness_corpus_args(command: &mut Command, repo_root: &Path) {
    command
        .args(["--enforcement-liveness-claude-settings"])
        .arg(repo_root.join(ENFORCEMENT_LIVENESS_CLAUDE_SETTINGS))
        .args(["--enforcement-liveness-codex-hooks"])
        .arg(repo_root.join(ENFORCEMENT_LIVENESS_CODEX_HOOKS))
        .args(["--enforcement-liveness-hooks-dir"])
        .arg(repo_root.join(ENFORCEMENT_LIVENESS_HOOKS_DIR));
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
            .arg(scm_facts)
            .args(["--stdout", "--face", face_name]);
        add_enforcement_liveness_corpus_args(&mut command, repo_root);
        let output = run_output(
            command.current_dir(repo_root),
            &format!("regenerate {file_name}"),
        )?;
        regenerated.push((file_name.to_owned(), output));
    }
    regenerated.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(regenerated)
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
}

struct MaterializerTools {
    emitter: PathBuf,
    producer: PathBuf,
    codemod: PathBuf,
}

fn build_face_tools(repo_root: &Path) -> Result<FaceTools, FreshnessError> {
    let output = run_output(
        Command::new("buck2")
            .arg("build")
            .arg(EMITTER_TARGET)
            .arg(PRODUCER_TARGET)
            .arg("--show-output")
            .current_dir(repo_root),
        "buck2 build freshness face tools",
    )?;
    let emitter = parse_show_output_path(repo_root, &output, EMITTER_TARGET)?;
    let producer = parse_show_output_path(repo_root, &output, PRODUCER_TARGET)?;
    Ok(FaceTools { emitter, producer })
}

fn build_materializer_tools(repo_root: &Path) -> Result<MaterializerTools, FreshnessError> {
    let output = run_output(
        Command::new("buck2")
            .arg("build")
            .arg(EMITTER_TARGET)
            .arg(PRODUCER_TARGET)
            .arg(CODEMOD_TARGET)
            .arg("--show-output")
            .current_dir(repo_root),
        "buck2 build generated-face materializer tools",
    )?;
    let emitter = parse_show_output_path(repo_root, &output, EMITTER_TARGET)?;
    let producer = parse_show_output_path(repo_root, &output, PRODUCER_TARGET)?;
    let codemod = parse_show_output_path(repo_root, &output, CODEMOD_TARGET)?;
    Ok(MaterializerTools {
        emitter,
        producer,
        codemod,
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

fn emit_materialized_scm_facts(
    tools: &MaterializerTools,
    repo_root: &Path,
    scm_facts: &Path,
) -> Result<(), FreshnessError> {
    run_status(
        Command::new(&tools.emitter)
            .args(["--repo-root"])
            .arg(repo_root)
            .args(["--out"])
            .arg(scm_facts)
            .arg("--merge-base-baseline")
            .current_dir(repo_root),
        "materialize scm-facts boundary snapshot",
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

struct TempFileCleanup {
    path: PathBuf,
}

impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
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

    #[test]
    fn parse_materialize_generated_faces_args_defaults_to_repo_root_dot() {
        let parsed = parse_materialize_generated_faces_args(Vec::new())
            .expect("empty args should use repository root default");

        assert_eq!(parsed.repo_root, PathBuf::from("."));
    }

    #[test]
    fn parse_materialize_generated_faces_args_accepts_repo_root() {
        let parsed = parse_materialize_generated_faces_args(vec![
            "--repo-root".to_owned(),
            "/tmp/oyatie".to_owned(),
        ])
        .expect("parse explicit repository root");

        assert_eq!(parsed.repo_root, PathBuf::from("/tmp/oyatie"));
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
    fn parse_show_output_path_resolves_materializer_targets() {
        let output = "\
root//cloud/cloud-ci/gates/oya-cloud-ci-scm-facts-emitter-app:oya-cloud-ci-scm-facts-emitter-app buck-out/v2/gen/emitter\n\
root//cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin /tmp/producer\n\
";

        let emitter =
            parse_show_output_path(Path::new("/repo"), output, EMITTER_TARGET).expect("emitter");
        let producer =
            parse_show_output_path(Path::new("/repo"), output, PRODUCER_TARGET).expect("producer");

        assert_eq!(emitter, PathBuf::from("/repo/buck-out/v2/gen/emitter"));
        assert_eq!(producer, PathBuf::from("/tmp/producer"));
    }
}
