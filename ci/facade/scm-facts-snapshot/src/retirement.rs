//! Git/object-derived facts for the history-only retirement validator.
//!
//! This module is part of the repository's single sanctioned Git boundary. It observes
//! candidate, first-parent, and immutable predecessor objects and emits a controller-owned,
//! untracked facts bundle. It never decides PASS, never creates receipts, and never copies a
//! retired body into the generated face.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use ci_artifact_inventory_registry::to_canonical_json;
#[cfg(unix)]
use rustix::fd::OwnedFd;
#[cfg(unix)]
use rustix::fs::{AtFlags, FileType, Mode, OFlags};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(crate) const CONTROL_PLANE_PATH: &str = "registry/history-only-retirement/control-plane.json";
/// Canonical untracked generated-facts path, exposed for the integration contract.
pub const GENERATED_FACTS_PATH: &str =
    "ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json";

const CONTROL_PLANE_SCHEMA: &str =
    "https://docs.oyatie.com/schemas/history-only-retirement-control-plane.schema.json";
const CONTROL_PLANE_NAME: &str = "history-only-retirement-control-plane";
const RECEIPT_ROOT: &str = "evidence/history-only-retirement";
const PROTECTED_BASE_REF: &str = "origin/dev";
static NEXT_ATOMIC_WRITE_ID: AtomicU64 = AtomicU64::new(0);

const MASTERPLAN_EVIDENCE_SET_ID: &str = "masterplan-retired-surfaces-history-only-retirement-v1";
const MASTERPLAN_PREPARATION_ID: &str = "masterplan-retired-surfaces-retirement-preparation";
const MASTERPLAN_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/masterplan-retired-surfaces-preparation.json";
const MASTERPLAN_CLOSURE_ID: &str = "masterplan-retired-surfaces-retirement-closure";
const MASTERPLAN_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/masterplan-retired-surfaces-closure.json";

const ADR_0363_EVIDENCE_SET_ID: &str = "adr-0363-amended-agentic-vcs-history-only-retirement-v1";
const ADR_0363_PREPARATION_ID: &str = "adr-0363-amended-agentic-vcs-retirement-preparation";
const ADR_0363_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0363-amended-agentic-vcs-preparation.json";
const ADR_0363_CLOSURE_ID: &str = "adr-0363-amended-agentic-vcs-retirement-closure";
const ADR_0363_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0363-amended-agentic-vcs-closure.json";

const ADR_0388_EVIDENCE_SET_ID: &str = "adr-0388-transient-ideas-history-only-retirement-v1";
const ADR_0388_PREPARATION_ID: &str = "adr-0388-transient-ideas-retirement-preparation";
const ADR_0388_PREPARATION_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-preparation.json";
const ADR_0388_CLOSURE_ID: &str = "adr-0388-transient-ideas-retirement-closure";
const ADR_0388_CLOSURE_PATH: &str =
    "evidence/history-only-retirement/adr-0388-transient-ideas-closure.json";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetirementControlPlane {
    #[serde(rename = "$schema")]
    schema: String,
    schema_version: u64,
    canonical_name: String,
    planning_state: String,
    dispatch_authorized: bool,
    receipt_root: String,
    predecessor_snapshot: CommitTree,
    entries: Vec<ControlPlaneEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CommitTree {
    commit_oid: String,
    tree_oid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ControlPlaneEntry {
    evidence_set_id: String,
    scope_ref: String,
    scope_type: String,
    selectors: Vec<ControlSelector>,
    preparation_artifact_id: String,
    preparation_receipt_path: String,
    closure_artifact_id: String,
    closure_receipt_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ControlSelector {
    selector_type: String,
    selector: String,
    expected_inputs: Vec<ExpectedInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExpectedInput {
    path: String,
    mode: String,
    predecessor_blob_oid: String,
    sha256: String,
    byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TreeEntry {
    mode: String,
    kind: String,
    oid: String,
    path: String,
}

impl TreeEntry {
    fn is_regular_blob(&self) -> bool {
        self.mode == "100644" && self.kind == "blob"
    }
}

pub(crate) trait RetirementObjectSource {
    fn resolve_commit(&self, revision: &str) -> Result<String, String>;
    fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String>;
    fn first_parent(&self, commit_oid: &str) -> Result<String, String>;
    fn parents(&self, commit_oid: &str) -> Result<Vec<String>, String>;
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String>;
    fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String>;
    fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String>;
    fn commits_touching_path(&self, commit_oid: &str, path: &str) -> Result<Vec<String>, String>;
}

pub(crate) struct GitCliRetirementObjectSource {
    repo_root: PathBuf,
}

impl GitCliRetirementObjectSource {
    pub(crate) fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    fn git(&self, args: &[&str], label: &str) -> Result<Vec<u8>, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(args)
            .output()
            .map_err(|error| format!("{label}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "{label}: git exited {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(output.stdout)
    }
}

impl RetirementObjectSource for GitCliRetirementObjectSource {
    fn resolve_commit(&self, revision: &str) -> Result<String, String> {
        let expression = format!("{revision}^{{commit}}");
        let output = self.git(
            &["rev-parse", "--verify", &expression],
            "resolve retirement commit",
        )?;
        parse_oid_text(&output, "resolved retirement commit")
    }

    fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String> {
        let expression = format!("{commit_oid}^{{tree}}");
        let output = self.git(
            &["rev-parse", "--verify", &expression],
            "resolve retirement tree",
        )?;
        parse_oid_text(&output, "resolved retirement tree")
    }

    fn first_parent(&self, commit_oid: &str) -> Result<String, String> {
        let expression = format!("{commit_oid}^1");
        let output = self.git(
            &["rev-parse", "--verify", &expression],
            "resolve retirement first parent",
        )?;
        parse_oid_text(&output, "resolved retirement first parent")
    }

    fn parents(&self, commit_oid: &str) -> Result<Vec<String>, String> {
        let output = self.git(
            &["rev-list", "--parents", "-n", "1", commit_oid],
            "resolve retirement parents",
        )?;
        let line = String::from_utf8(output)
            .map_err(|error| format!("retirement parents are not UTF-8: {error}"))?;
        let mut fields = line.split_whitespace();
        let commit = fields
            .next()
            .ok_or_else(|| "retirement parents are empty".to_owned())?;
        if commit != commit_oid {
            return Err(
                "retirement parent list does not bind the requested evaluated commit".to_owned(),
            );
        }
        fields
            .map(|parent| {
                validate_oid(parent, "retirement parent")?;
                Ok(parent.to_owned())
            })
            .collect()
    }

    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String> {
        let status = Command::new("git")
            .arg("-C")
            .arg(&self.repo_root)
            .args(["merge-base", "--is-ancestor", ancestor, descendant])
            .status()
            .map_err(|error| format!("retirement ancestry: {error}"))?;
        match status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            code => Err(format!("retirement ancestry: git exited {code:?}")),
        }
    }

    fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String> {
        let output = self.git(
            &["ls-tree", "-rz", "--full-tree", "-r", commit_oid],
            "enumerate retirement tree",
        )?;
        parse_ls_tree(&output)
    }

    fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String> {
        self.git(&["cat-file", "blob", blob_oid], "read retirement blob")
    }

    fn commits_touching_path(&self, commit_oid: &str, path: &str) -> Result<Vec<String>, String> {
        let output = self.git(
            &["rev-list", commit_oid, "--", path],
            "walk retirement receipt history",
        )?;
        String::from_utf8(output)
            .map_err(|error| format!("retirement history is not UTF-8: {error}"))?
            .lines()
            .map(|line| {
                validate_oid(line, "retirement history commit")?;
                Ok(line.to_owned())
            })
            .collect()
    }
}

/// Runtime context accepted by the facts materializer.
///
/// Public solely so the package-local integration target can exercise the
/// real Git boundary without duplicating production materialization logic.
#[derive(Debug, Clone)]
pub struct RetirementMaterializationContext<'a> {
    pub control_plane_path: &'a str,
    pub protected_base_commit: &'a str,
    pub evaluated_commit: &'a str,
    pub scm_event_name: &'a str,
    pub scm_event_ref: &'a str,
    pub subject_commit: &'a str,
}

/// Materialize facts through the sanctioned Git boundary.
///
/// This is public for the package-local integration target; it emits facts and
/// never produces a PASS or dispatch decision.
pub fn emit_history_only_retirement_facts(
    repo_root: &Path,
    context: &RetirementMaterializationContext<'_>,
    output_path: &Path,
) -> Result<(), String> {
    canonical_generated_facts_output_path(repo_root, output_path)?;
    let source = GitCliRetirementObjectSource::new(repo_root.to_path_buf());
    let value = materialize_history_only_retirement_facts(&source, context)?;
    let bytes = to_canonical_json(&value)
        .map_err(|error| format!("serialize history-only retirement facts: {error}"))?;
    write_canonical_retirement_facts(repo_root, bytes.as_bytes())
}

/// A Unix capability bound to the canonical retirement-facts parent directory.
///
/// It owns the opened directory descriptor, so its finalization remains bound
/// to that directory even if a pathname ancestor is replaced after [`Self::open`].
#[cfg(unix)]
pub struct CanonicalRetirementFactsWriter {
    directory: OwnedFd,
}

#[cfg(unix)]
impl CanonicalRetirementFactsWriter {
    /// Open the fixed canonical retirement-facts parent without following symlinks.
    pub fn open(repo_root: &Path) -> Result<Self, String> {
        canonical_generated_facts_output_path(repo_root, Path::new(GENERATED_FACTS_PATH))?;
        Ok(Self {
            directory: open_canonical_retirement_facts_parent(repo_root)?,
        })
    }

    /// Atomically replace only the fixed canonical facts basename through this directory fd.
    pub fn write(&self, bytes: &[u8]) -> Result<(), String> {
        const FINAL_NAME: &str = "history-only-retirement-facts.generated.json";
        ensure_regular_or_absent(&self.directory, FINAL_NAME)?;
        let (temporary_name, temporary) = create_temporary_file(&self.directory)?;

        let result = (|| {
            write_all(&temporary, bytes)?;
            rustix::fs::fsync(&temporary)
                .map_err(|error| format!("sync retirement facts temporary file: {error}"))?;
            rustix::fs::renameat(
                &self.directory,
                &temporary_name,
                &self.directory,
                FINAL_NAME,
            )
            .map_err(|error| format!("replace retirement facts output: {error}"))?;
            rustix::fs::fsync(&self.directory)
                .map_err(|error| format!("sync retirement facts directory: {error}"))
        })();
        if result.is_err() {
            let _ = rustix::fs::unlinkat(&self.directory, &temporary_name, AtFlags::empty());
        }
        result
    }
}

/// Non-Unix placeholder that preserves the public API while failing closed.
#[cfg(not(unix))]
pub struct CanonicalRetirementFactsWriter;

#[cfg(not(unix))]
impl CanonicalRetirementFactsWriter {
    /// The descriptor-relative writer is unavailable on this platform.
    pub fn open(_repo_root: &Path) -> Result<Self, String> {
        Err("canonical retirement facts writer requires Unix dirfd support".to_owned())
    }

    /// The descriptor-relative writer is unavailable on this platform.
    pub fn write(&self, _bytes: &[u8]) -> Result<(), String> {
        Err("canonical retirement facts writer requires Unix dirfd support".to_owned())
    }
}

/// Atomically write the canonical ignored retirement-facts file.
///
/// Public only for the package-local integration target's filesystem defenses.
/// The path is intentionally not caller-controlled: this seam can write only
/// [`GENERATED_FACTS_PATH`], after rerunning the ignore/untracked boundary.
#[cfg(unix)]
pub fn write_canonical_retirement_facts(repo_root: &Path, bytes: &[u8]) -> Result<(), String> {
    CanonicalRetirementFactsWriter::open(repo_root)?.write(bytes)
}

#[cfg(not(unix))]
pub fn write_canonical_retirement_facts(_repo_root: &Path, _bytes: &[u8]) -> Result<(), String> {
    CanonicalRetirementFactsWriter::open(_repo_root)?.write(_bytes)
}

#[cfg(unix)]
fn open_canonical_retirement_facts_parent(repo_root: &Path) -> Result<OwnedFd, String> {
    let mut directory = rustix::fs::openat(
        rustix::fs::CWD,
        repo_root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| format!("open retirement facts repository directory: {error}"))?;
    for component in ["ci", "facade", "scm-facts-snapshot"] {
        directory = open_or_create_directory_at(&directory, component)?;
    }
    Ok(directory)
}

#[cfg(unix)]
fn open_or_create_directory_at(parent: &OwnedFd, name: &str) -> Result<OwnedFd, String> {
    if name.contains('\0') {
        return Err(format!("retirement facts directory contains NUL: {name:?}"));
    }
    match rustix::fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(directory) => Ok(directory),
        Err(error) if error == rustix::io::Errno::NOENT => {
            match rustix::fs::mkdirat(parent, name, Mode::from_bits_retain(0o755)) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(error) => {
                    return Err(format!(
                        "create retirement facts directory {name:?}: {error}"
                    ));
                }
            }
            rustix::fs::openat(
                parent,
                name,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| format!("open retirement facts directory {name:?}: {error}"))
        }
        Err(error) if error == rustix::io::Errno::NOTDIR || error == rustix::io::Errno::LOOP => {
            Err(format!(
                "retirement facts directory {name:?} is not a real directory"
            ))
        }
        Err(error) => Err(format!("open retirement facts directory {name:?}: {error}")),
    }
}

#[cfg(unix)]
fn ensure_regular_or_absent(directory: &OwnedFd, name: &str) -> Result<(), String> {
    match rustix::fs::statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(stat) if !FileType::from_raw_mode(stat.st_mode).is_file() => {
            Err("retirement facts output must be a regular file".to_owned())
        }
        Ok(_) | Err(rustix::io::Errno::NOENT) => Ok(()),
        Err(error) => Err(format!("inspect retirement facts output: {error}")),
    }
}

#[cfg(unix)]
fn create_temporary_file(directory: &OwnedFd) -> Result<(String, OwnedFd), String> {
    for _ in 0..32 {
        let name = format!(
            ".retirement-facts-{}-{}",
            std::process::id(),
            NEXT_ATOMIC_WRITE_ID.fetch_add(1, Ordering::Relaxed)
        );
        match rustix::fs::openat(
            directory,
            &name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC | OFlags::NOFOLLOW,
            Mode::from_bits_retain(0o600),
        ) {
            Ok(file) => return Ok((name, file)),
            Err(rustix::io::Errno::EXIST) => continue,
            Err(error) => return Err(format!("create retirement facts temporary file: {error}")),
        }
    }
    Err("exhausted retirement facts temporary file names".to_owned())
}

#[cfg(unix)]
fn write_all(file: &OwnedFd, mut bytes: &[u8]) -> Result<(), String> {
    while !bytes.is_empty() {
        let written = rustix::io::write(file, bytes)
            .map_err(|error| format!("write retirement facts temporary file: {error}"))?;
        if written == 0 {
            return Err("write retirement facts temporary file made no progress".to_owned());
        }
        bytes = &bytes[written..];
    }
    Ok(())
}

fn canonical_generated_facts_output_path(
    repo_root: &Path,
    output_path: &Path,
) -> Result<(), String> {
    if output_path != Path::new(GENERATED_FACTS_PATH)
        || !output_path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "retirement facts output must be the exact canonical repo-relative generated facts path {GENERATED_FACTS_PATH}"
        ));
    }
    let status = Command::new("git")
        .args(["check-ignore", "--quiet", "--", GENERATED_FACTS_PATH])
        .current_dir(repo_root)
        .status()
        .map_err(|error| format!("check retirement facts output ignore boundary: {error}"))?;
    match status.code() {
        Some(0) => Ok(()),
        Some(1) => Err(format!(
            "retirement facts output {GENERATED_FACTS_PATH} must be ignored and untracked"
        )),
        code => Err(format!(
            "check retirement facts output ignore boundary: git exited {code:?}"
        )),
    }
}

pub(crate) fn materialize_history_only_retirement_facts(
    source: &impl RetirementObjectSource,
    context: &RetirementMaterializationContext<'_>,
) -> Result<Value, String> {
    if context.control_plane_path != CONTROL_PLANE_PATH {
        return Err(format!(
            "retirement control-plane path must be {CONTROL_PLANE_PATH}"
        ));
    }

    for (label, requested) in [
        (
            "requested protected base commit",
            context.protected_base_commit,
        ),
        ("requested evaluated commit", context.evaluated_commit),
        ("requested subject commit", context.subject_commit),
    ] {
        validate_oid(requested, label)?;
    }
    let candidate = source.resolve_commit(context.evaluated_commit)?;
    let head = source.resolve_commit("HEAD")?;
    if candidate != head {
        return Err(format!(
            "retirement candidate {candidate} is not exact HEAD {head}"
        ));
    }
    let protected = source.resolve_commit(context.protected_base_commit)?;
    let subject = source.resolve_commit(context.subject_commit)?;
    if protected != context.protected_base_commit
        || candidate != context.evaluated_commit
        || subject != context.subject_commit
    {
        return Err(
            "requested retirement event identity must equal resolved commit identity".to_owned(),
        );
    }
    validate_event_identity(
        source,
        context.scm_event_name,
        context.scm_event_ref,
        &protected,
        &candidate,
        &subject,
    )?;
    let first_parent = source.first_parent(&candidate)?;
    if protected != first_parent {
        return Err(format!(
            "retirement protected base {protected} is not candidate first parent {first_parent}"
        ));
    }
    if !source.is_ancestor(&protected, &candidate)? {
        return Err("retirement protected base is not an ancestor of candidate".to_owned());
    }
    let protected_tree = source.tree_for_commit(&protected)?;
    let candidate_tree = source.tree_for_commit(&candidate)?;
    let protected_entries = entries_by_path(source.tree_entries(&protected)?)?;
    let candidate_entries = entries_by_path(source.tree_entries(&candidate)?)?;

    let candidate_control_entry = candidate_entries
        .get(CONTROL_PLANE_PATH)
        .ok_or_else(|| "candidate retirement control plane is absent".to_owned())?;
    require_regular(
        candidate_control_entry,
        "candidate retirement control plane",
    )?;
    let candidate_control_bytes = source.read_blob(&candidate_control_entry.oid)?;
    let control_plane: RetirementControlPlane = parse_closed_json(&candidate_control_bytes)?;
    validate_control_plane(&control_plane)?;

    let predecessor = source.resolve_commit(&control_plane.predecessor_snapshot.commit_oid)?;
    if predecessor != control_plane.predecessor_snapshot.commit_oid {
        return Err("retirement predecessor commit is not canonical".to_owned());
    }
    let predecessor_tree = source.tree_for_commit(&predecessor)?;
    if predecessor_tree != control_plane.predecessor_snapshot.tree_oid {
        return Err("retirement predecessor commit/tree binding does not match Git".to_owned());
    }
    if !source.is_ancestor(&predecessor, &protected)? {
        return Err("retirement predecessor is not an ancestor of protected base".to_owned());
    }
    let predecessor_entries = entries_by_path(source.tree_entries(&predecessor)?)?;
    validate_predecessor_inputs(source, &control_plane, &predecessor_entries)?;
    validate_selector_coverage(&control_plane, &predecessor_entries, "predecessor")?;
    validate_selector_coverage(&control_plane, &protected_entries, "protected")?;
    validate_selector_coverage(&control_plane, &candidate_entries, "candidate")?;

    let protected_control = protected_entries.get(CONTROL_PLANE_PATH);
    let bootstrap = protected_control.is_none();
    let protected_control_bytes = match protected_control {
        None => None,
        Some(entry) => {
            require_regular(entry, "protected retirement control plane")?;
            let bytes = source.read_blob(&entry.oid)?;
            if bytes != candidate_control_bytes || entry.oid != candidate_control_entry.oid {
                return Err(
                    "protected and candidate retirement control planes are not immutable-identical"
                        .to_owned(),
                );
            }
            Some(bytes)
        }
    };

    let stages = control_plane
        .entries
        .iter()
        .map(|entry| classify_stage(entry, &protected_entries, &candidate_entries))
        .collect::<Result<Vec<_>, _>>()?;
    let stage = stages
        .first()
        .copied()
        .ok_or_else(|| "retirement control plane has no entries".to_owned())?;
    if stages.iter().any(|candidate| *candidate != stage) {
        return Err(
            "retirement receipt population is not atomic across all three scopes".to_owned(),
        );
    }
    if bootstrap && stage != ReceiptStage::Dormant {
        return Err("retirement bootstrap may not add receipts".to_owned());
    }

    let protected_receipt_inventory = receipt_root_inventory(&protected_entries);
    let candidate_receipt_inventory = receipt_root_inventory(&candidate_entries);
    let expected_receipt_paths = expected_receipt_paths(&control_plane);
    let unexpected_protected_receipt_paths = protected_receipt_inventory
        .difference(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let unexpected_candidate_receipt_paths = candidate_receipt_inventory
        .difference(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();

    let control_plane_entries = control_plane
        .entries
        .iter()
        .map(control_entry_value)
        .collect::<Result<Vec<_>, _>>()?;
    let control_plane_entry_hashes = control_plane
        .entries
        .iter()
        .zip(control_plane_entries.iter())
        .map(|(entry, value)| {
            Ok(json!({
                "scope_ref": entry.scope_ref,
                "sha256": canonical_value_sha256(value)?,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let protected_control_sha = protected_control_bytes.as_deref().map(sha256_digest);
    let candidate_control_sha = sha256_digest(&candidate_control_bytes);
    let mut receipts = Vec::new();
    let mut object_facts = Vec::new();
    let mut scopes = Vec::new();
    let mut all_required_paths = BTreeSet::new();
    let mut protected_preparations = Vec::new();

    if stage != ReceiptStage::Dormant {
        for entry in &control_plane.entries {
            let (receipt_path, receipt_entry) =
                receipt_for_stage(stage, entry, &protected_entries, &candidate_entries)?;
            require_regular(receipt_entry, "retirement receipt")?;
            let receipt_bytes = source.read_blob(&receipt_entry.oid)?;
            let candidate_receipt_sha256 = sha256_digest(&receipt_bytes);
            let receipt: Value = parse_closed_json(&receipt_bytes)?;
            validate_receipt_identity(stage, entry, receipt_path, &receipt)?;
            let artifact_id = required_value_string(receipt.get("artifact_id"), "artifact_id")?;

            let input_facts = entry
                .selectors
                .iter()
                .flat_map(|selector| selector.expected_inputs.iter())
                .map(|input| {
                    all_required_paths.insert(input.path.clone());
                    input_fact(
                        source,
                        input,
                        &predecessor_entries,
                        &protected_entries,
                        &candidate_entries,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            let control_entry = control_entry_value(entry)?;
            let baseline = receipt
                .get("baseline")
                .and_then(Value::as_object)
                .ok_or_else(|| format!("receipt {receipt_path} has no baseline object"))?;
            let baseline_commit =
                required_value_string(baseline.get("commit_oid"), "baseline.commit_oid")?;
            let baseline_tree =
                required_value_string(baseline.get("tree_oid"), "baseline.tree_oid")?;
            validate_oid(baseline_commit, "receipt baseline commit")?;
            validate_oid(baseline_tree, "receipt baseline tree")?;
            require_predecessor_baseline(
                baseline_commit,
                baseline_tree,
                &predecessor,
                &predecessor_tree,
                receipt_path,
            )?;
            receipts.push(json!({
                "receipt_path": receipt_path,
                "artifact_id": artifact_id,
                "scope_ref": entry.scope_ref,
                "receipt_state": stage.as_str(),
                "candidate_receipt_blob_oid": receipt_entry.oid,
                "candidate_receipt_sha256": candidate_receipt_sha256,
                "baseline_commit_oid": baseline_commit,
                "baseline_tree_oid": baseline_tree,
            }));

            let (preparation_path, preparation_blob, predecessor_context) = match stage {
                ReceiptStage::PreparedNew => (
                    Value::Null,
                    Value::Null,
                    json!({
                        "source": "control-plane-predecessor",
                        "commit_oid": predecessor,
                        "tree_oid": predecessor_tree,
                        "receipt_path": Value::Null,
                        "receipt_blob_oid": Value::Null,
                    }),
                ),
                ReceiptStage::ClosureNew => {
                    let (linked_path, linked_blob) = closure_preparation_link(&receipt)?;
                    if linked_path != entry.preparation_receipt_path {
                        return Err(format!(
                            "closure {receipt_path} links unexpected preparation path"
                        ));
                    }
                    let protected_preparation = protected_entries
                        .get(&entry.preparation_receipt_path)
                        .ok_or_else(|| {
                            "closure is missing protected preparation receipt".to_owned()
                        })?;
                    require_regular(protected_preparation, "protected preparation receipt")?;
                    if linked_blob != protected_preparation.oid {
                        return Err(format!(
                            "closure {receipt_path} links unexpected protected preparation blob"
                        ));
                    }
                    let preparation_bytes = source.read_blob(&protected_preparation.oid)?;
                    let preparation: Value = parse_closed_json(&preparation_bytes)?;
                    validate_receipt_identity(
                        ReceiptStage::PreparedNew,
                        entry,
                        &entry.preparation_receipt_path,
                        &preparation,
                    )?;
                    let (commit, tree) = receipt_baseline(&preparation)?;
                    require_predecessor_baseline(
                        &commit,
                        &tree,
                        &predecessor,
                        &predecessor_tree,
                        &entry.preparation_receipt_path,
                    )?;
                    protected_preparations.push(json!({
                        "receipt_path": entry.preparation_receipt_path,
                        "receipt_blob_oid": protected_preparation.oid,
                        "baseline_commit_oid": commit,
                        "baseline_tree_oid": tree,
                    }));
                    (
                        json!(entry.preparation_receipt_path),
                        json!(protected_preparation.oid),
                        json!({
                            "source": "protected-preparation-receipt",
                            "commit_oid": commit,
                            "tree_oid": tree,
                            "receipt_path": entry.preparation_receipt_path,
                            "receipt_blob_oid": protected_preparation.oid,
                        }),
                    )
                }
                ReceiptStage::ClosedCarried => {
                    let (path, blob) = closure_preparation_link(&receipt)?;
                    if path != entry.preparation_receipt_path {
                        return Err(format!(
                            "closure {receipt_path} links unexpected preparation path"
                        ));
                    }
                    let (commit, tree) =
                        find_linked_preparation(source, &protected, path, blob, entry)?;
                    require_predecessor_baseline(
                        &commit,
                        &tree,
                        &predecessor,
                        &predecessor_tree,
                        path,
                    )?;
                    protected_preparations.push(json!({
                        "receipt_path": path,
                        "receipt_blob_oid": blob,
                        "baseline_commit_oid": commit,
                        "baseline_tree_oid": tree,
                    }));
                    (
                        json!(path),
                        json!(blob),
                        json!({
                            "source": "linked-preparation-history",
                            "commit_oid": commit,
                            "tree_oid": tree,
                            "receipt_path": path,
                            "receipt_blob_oid": blob,
                        }),
                    )
                }
                ReceiptStage::Dormant => unreachable!("dormant stage has no receipt facts"),
            };

            let protected_receipt = protected_entries.get(receipt_path);
            let protected_receipt_sha256 = protected_receipt
                .map(|entry| {
                    source
                        .read_blob(&entry.oid)
                        .map(|bytes| sha256_digest(&bytes))
                })
                .transpose()?;
            object_facts.push(json!({
                "artifact_id": artifact_id,
                "receipt_path": receipt_path,
                "protected_base_ref": PROTECTED_BASE_REF,
                "receipt_state": stage.as_str(),
                "scope_ref": entry.scope_ref,
                "scope_type": entry.scope_type,
                "baseline_commit_oid": baseline_commit,
                "baseline_tree_oid": baseline_tree,
                "protected_receipt_blob_oid": protected_receipt.map_or(Value::Null, |entry| json!(entry.oid)),
                "candidate_receipt_blob_oid": receipt_entry.oid,
                "protected_registry_row_sha256": protected_receipt_sha256,
                "candidate_registry_row_sha256": candidate_receipt_sha256,
                "retired_inputs": input_facts,
                "preparation_receipt_path": preparation_path,
                "protected_preparation_receipt_blob_oid": preparation_blob,
                "predecessor_context": predecessor_context,
                "control_plane_entry": control_entry,
                "control_plane_entry_sha256": canonical_value_sha256(&control_entry_value(entry)?)?,
            }));
            scopes.push(coverage_scope(
                entry,
                &predecessor_entries,
                &protected_entries,
                &candidate_entries,
            ));
        }
    }

    receipts.sort_by(|left, right| {
        left.get("receipt_path")
            .and_then(Value::as_str)
            .cmp(&right.get("receipt_path").and_then(Value::as_str))
    });
    object_facts.sort_by(|left, right| {
        left.get("receipt_path")
            .and_then(Value::as_str)
            .cmp(&right.get("receipt_path").and_then(Value::as_str))
    });
    scopes.sort_by(|left, right| {
        left.get("scope_ref")
            .and_then(Value::as_str)
            .cmp(&right.get("scope_ref").and_then(Value::as_str))
    });
    protected_preparations.sort_by(|left, right| {
        left.get("receipt_path")
            .and_then(Value::as_str)
            .cmp(&right.get("receipt_path").and_then(Value::as_str))
    });

    let protected_receipt_paths = protected_receipt_inventory
        .intersection(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let candidate_receipt_paths = candidate_receipt_inventory
        .intersection(&expected_receipt_paths)
        .cloned()
        .collect::<Vec<_>>();
    let carried_receipt_paths = protected_receipt_paths
        .iter()
        .filter(|path| candidate_receipt_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    let new_receipt_paths = candidate_receipt_paths
        .iter()
        .filter(|path| !protected_receipt_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();

    Ok(json!({
        "receipts": receipts,
        "scm_facts": {
            "retirement_receipt_coverage": {
                "protected_base_ref": PROTECTED_BASE_REF,
                "protected_receipt_paths": protected_receipt_paths,
                "candidate_receipt_paths": candidate_receipt_paths,
                "carried_receipt_paths": carried_receipt_paths,
                "new_receipt_paths": new_receipt_paths,
                "scopes": scopes,
                "required_retired_paths": all_required_paths,
            },
            "retirement_receipt_object_facts": object_facts,
            "protected_scm_context": {
                "protected_base_ref": PROTECTED_BASE_REF,
                "protected_base_commit_oid": protected,
                "protected_base_tree_oid": protected_tree,
                "evaluated_commit_oid": candidate,
                "evaluated_tree_oid": candidate_tree,
                "subject_commit_oid": subject,
                "subject_tree_oid": source.tree_for_commit(&subject)?,
                "scm_event_name": context.scm_event_name,
                "subject_relationship": if context.scm_event_name == "pull_request" { "pull-request-head" } else { "evaluated-self" },
                "protected_base_is_ancestor_of_evaluated": true,
                "protected_base_is_evaluated_first_parent": true,
                "subject_is_evaluated_second_parent": context.scm_event_name == "pull_request",
                "predecessor_commit_oid": predecessor,
                "predecessor_tree_oid": predecessor_tree,
                "predecessor_commit_exists": true,
                "predecessor_tree_exists": true,
                "predecessor_commit_tree_bound": true,
                "predecessor_is_ancestor_of_protected_base": true,
                "protected_preparation_receipts": protected_preparations,
            },
            "retirement_control_plane_context": {
                "control_plane_path": CONTROL_PLANE_PATH,
                "receipt_root": RECEIPT_ROOT,
                "bootstrap": bootstrap,
                "lifecycle_state": stage.as_str(),
                "protected_control_plane_blob_oid": protected_control.map_or(Value::Null, |entry| json!(entry.oid)),
                "protected_control_plane_sha256": protected_control_sha,
                "protected_control_plane_byte_count": protected_control_bytes.as_ref().map(|bytes| bytes.len() as u64),
                "candidate_control_plane_blob_oid": candidate_control_entry.oid,
                "candidate_control_plane_sha256": candidate_control_sha,
                "candidate_control_plane_byte_count": candidate_control_bytes.len() as u64,
                "control_plane_entries": control_plane_entries,
                "control_plane_entry_hashes": control_plane_entry_hashes,
                "protected_receipt_root_paths": protected_receipt_inventory,
                "candidate_receipt_root_paths": candidate_receipt_inventory,
                "unexpected_protected_receipt_paths": unexpected_protected_receipt_paths,
                "unexpected_candidate_receipt_paths": unexpected_candidate_receipt_paths,
            },
        }
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiptStage {
    Dormant,
    PreparedNew,
    ClosureNew,
    ClosedCarried,
}

impl ReceiptStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Dormant => "dormant",
            Self::PreparedNew => "prepared-new",
            Self::ClosureNew => "closure-new",
            Self::ClosedCarried => "closed-carried",
        }
    }
}

fn classify_stage(
    entry: &ControlPlaneEntry,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
) -> Result<ReceiptStage, String> {
    let pp = protected.contains_key(&entry.preparation_receipt_path);
    let pc = protected.contains_key(&entry.closure_receipt_path);
    let cp = candidate.contains_key(&entry.preparation_receipt_path);
    let cc = candidate.contains_key(&entry.closure_receipt_path);
    match (pp, pc, cp, cc) {
        (false, false, false, false) => Ok(ReceiptStage::Dormant),
        (false, false, true, false) => Ok(ReceiptStage::PreparedNew),
        (true, false, false, true) => Ok(ReceiptStage::ClosureNew),
        (false, true, false, true) => {
            let protected_entry = protected
                .get(&entry.closure_receipt_path)
                .expect("presence checked");
            let candidate_entry = candidate
                .get(&entry.closure_receipt_path)
                .expect("presence checked");
            if protected_entry.oid != candidate_entry.oid
                || protected_entry.mode != candidate_entry.mode
                || protected_entry.kind != candidate_entry.kind
            {
                return Err(format!(
                    "carried closure {} changed",
                    entry.closure_receipt_path
                ));
            }
            Ok(ReceiptStage::ClosedCarried)
        }
        _ => Err(format!(
            "invalid retirement receipt lifecycle for {}",
            entry.scope_ref
        )),
    }
}

fn receipt_for_stage<'a>(
    stage: ReceiptStage,
    control: &'a ControlPlaneEntry,
    _protected: &'a BTreeMap<String, TreeEntry>,
    candidate: &'a BTreeMap<String, TreeEntry>,
) -> Result<(&'a str, &'a TreeEntry), String> {
    let path = match stage {
        ReceiptStage::PreparedNew => control.preparation_receipt_path.as_str(),
        ReceiptStage::ClosureNew | ReceiptStage::ClosedCarried => {
            control.closure_receipt_path.as_str()
        }
        ReceiptStage::Dormant => return Err("dormant stage has no receipt".to_owned()),
    };
    candidate
        .get(path)
        .map(|entry| (path, entry))
        .ok_or_else(|| format!("candidate receipt {path} is absent"))
}

fn validate_control_plane(control: &RetirementControlPlane) -> Result<(), String> {
    if control.schema != CONTROL_PLANE_SCHEMA
        || control.schema_version != 1
        || control.canonical_name != CONTROL_PLANE_NAME
        || control.planning_state != "HOLD(Planning)"
        || control.dispatch_authorized
        || control.receipt_root != RECEIPT_ROOT
    {
        return Err("retirement control-plane header is not canonical HOLD".to_owned());
    }
    validate_oid(
        &control.predecessor_snapshot.commit_oid,
        "retirement predecessor commit",
    )?;
    validate_oid(
        &control.predecessor_snapshot.tree_oid,
        "retirement predecessor tree",
    )?;
    if control.entries.len() != 3 {
        return Err("retirement control plane must contain exactly three entries".to_owned());
    }
    let expected = [
        fixed_entry(
            "artifact:masterplan",
            "masterplan-retired-surfaces",
            MASTERPLAN_EVIDENCE_SET_ID,
            MASTERPLAN_PREPARATION_ID,
            MASTERPLAN_PREPARATION_PATH,
            MASTERPLAN_CLOSURE_ID,
            MASTERPLAN_CLOSURE_PATH,
        ),
        fixed_entry(
            "ADR-0363",
            "amended-agentic-vcs-retirement",
            ADR_0363_EVIDENCE_SET_ID,
            ADR_0363_PREPARATION_ID,
            ADR_0363_PREPARATION_PATH,
            ADR_0363_CLOSURE_ID,
            ADR_0363_CLOSURE_PATH,
        ),
        fixed_entry(
            "ADR-0388",
            "transient-ideas",
            ADR_0388_EVIDENCE_SET_ID,
            ADR_0388_PREPARATION_ID,
            ADR_0388_PREPARATION_PATH,
            ADR_0388_CLOSURE_ID,
            ADR_0388_CLOSURE_PATH,
        ),
    ];
    for (entry, fixed) in control.entries.iter().zip(expected) {
        if entry.scope_ref != fixed.scope_ref
            || entry.scope_type != fixed.scope_type
            || entry.evidence_set_id != fixed.evidence_set_id
            || entry.preparation_artifact_id != fixed.preparation_artifact_id
            || entry.preparation_receipt_path != fixed.preparation_receipt_path
            || entry.closure_artifact_id != fixed.closure_artifact_id
            || entry.closure_receipt_path != fixed.closure_receipt_path
        {
            return Err(format!(
                "retirement control-plane identity mismatch for {}",
                entry.scope_ref
            ));
        }
        validate_repo_path(&entry.preparation_receipt_path)?;
        validate_repo_path(&entry.closure_receipt_path)?;
    }
    validate_fixed_selectors(control)
}

fn fixed_entry(
    scope_ref: &str,
    scope_type: &str,
    evidence_set_id: &str,
    preparation_artifact_id: &str,
    preparation_receipt_path: &str,
    closure_artifact_id: &str,
    closure_receipt_path: &str,
) -> ControlPlaneEntry {
    ControlPlaneEntry {
        evidence_set_id: evidence_set_id.to_owned(),
        scope_ref: scope_ref.to_owned(),
        scope_type: scope_type.to_owned(),
        selectors: Vec::new(),
        preparation_artifact_id: preparation_artifact_id.to_owned(),
        preparation_receipt_path: preparation_receipt_path.to_owned(),
        closure_artifact_id: closure_artifact_id.to_owned(),
        closure_receipt_path: closure_receipt_path.to_owned(),
    }
}

fn validate_fixed_selectors(control: &RetirementControlPlane) -> Result<(), String> {
    let expected: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::from([
        ("artifact:masterplan", vec![("exact", "docs/ROADMAP.md")]),
        (
            "ADR-0363",
            vec![
                ("exact", ".omc/ultragoal/OWNERS"),
                ("exact", ".omc/ultragoal/TEAMMATE-PREAMBLE.md"),
                ("exact", ".omc/ultragoal/friction-ledger.jsonl"),
                ("exact", ".omc/ultragoal/premise.txt"),
            ],
        ),
        ("ADR-0388", vec![("glob", "docs/ideas/archive/**")]),
    ]);
    for entry in &control.entries {
        let required = expected
            .get(entry.scope_ref.as_str())
            .ok_or_else(|| format!("unknown retirement scope {}", entry.scope_ref))?;
        if entry.selectors.len() != required.len() {
            return Err(format!("selector count mismatch for {}", entry.scope_ref));
        }
        for (selector, (kind, pattern)) in entry.selectors.iter().zip(required) {
            if selector.selector_type != *kind || selector.selector != *pattern {
                return Err(format!("selector mismatch for {}", entry.scope_ref));
            }
            if selector.expected_inputs.is_empty() {
                return Err(format!(
                    "selector has no immutable inputs for {}",
                    entry.scope_ref
                ));
            }
            for input in &selector.expected_inputs {
                validate_repo_path(&input.path)?;
                if input.mode != "100644" {
                    return Err(format!(
                        "immutable retirement input {} must declare mode 100644",
                        input.path
                    ));
                }
                validate_oid(&input.predecessor_blob_oid, "predecessor blob")?;
                validate_sha256(&input.sha256)?;
            }
        }
    }
    let actual_paths = control
        .entries
        .iter()
        .flat_map(|entry| entry.selectors.iter())
        .flat_map(|selector| selector.expected_inputs.iter())
        .map(|input| input.path.as_str())
        .collect::<BTreeSet<_>>();
    let required_paths = BTreeSet::from([
        "docs/ROADMAP.md",
        ".omc/ultragoal/OWNERS",
        ".omc/ultragoal/TEAMMATE-PREAMBLE.md",
        ".omc/ultragoal/friction-ledger.jsonl",
        ".omc/ultragoal/premise.txt",
        "docs/ideas/archive/cloud-intelligence-bedrock-on-talos-2026-05-28.md",
        "docs/ideas/archive/cloud-intelligence-v1-pipeline-2026-05-28.md",
        "docs/ideas/archive/n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md",
    ]);
    if actual_paths != required_paths {
        return Err("retirement immutable input population is not exact".to_owned());
    }
    Ok(())
}

fn validate_predecessor_inputs(
    source: &impl RetirementObjectSource,
    control: &RetirementControlPlane,
    predecessor_entries: &BTreeMap<String, TreeEntry>,
) -> Result<(), String> {
    for input in control
        .entries
        .iter()
        .flat_map(|entry| entry.selectors.iter())
        .flat_map(|selector| selector.expected_inputs.iter())
    {
        let entry = predecessor_entries
            .get(&input.path)
            .ok_or_else(|| format!("retirement predecessor path {} is absent", input.path))?;
        require_regular(entry, "retirement predecessor input")?;
        if entry.mode != input.mode || entry.oid != input.predecessor_blob_oid {
            return Err(format!(
                "retirement predecessor blob mismatch for {}",
                input.path
            ));
        }
        let bytes = source.read_blob(&entry.oid)?;
        if sha256_digest(&bytes) != input.sha256 || bytes.len() as u64 != input.byte_count {
            return Err(format!(
                "retirement predecessor raw-byte binding mismatch for {}",
                input.path
            ));
        }
    }
    Ok(())
}

fn validate_selector_coverage(
    control: &RetirementControlPlane,
    entries: &BTreeMap<String, TreeEntry>,
    tree_role: &str,
) -> Result<(), String> {
    for selector in control.entries.iter().flat_map(|entry| &entry.selectors) {
        let expected = selector
            .expected_inputs
            .iter()
            .map(|input| input.path.as_str())
            .collect::<BTreeSet<_>>();
        for path in entries
            .keys()
            .filter(|path| selector_matches_path(selector, path))
        {
            if !expected.contains(path.as_str()) {
                return Err(format!(
                    "retirement selector coverage rejects unlisted {tree_role} path {path}"
                ));
            }
        }
    }
    Ok(())
}

fn validate_event_identity(
    source: &impl RetirementObjectSource,
    event: &str,
    event_ref: &str,
    protected: &str,
    evaluated: &str,
    subject: &str,
) -> Result<(), String> {
    match event {
        "pull_request" => {
            if !event_ref.starts_with("refs/pull/") {
                return Err("pull_request event ref must be a pull request ref".to_owned());
            }
            let parents = source.parents(evaluated)?;
            if parents != [protected.to_owned(), subject.to_owned()] {
                return Err("pull_request evaluated commit parents must be exactly [protected base, subject]".to_owned());
            }
            if subject == evaluated {
                return Err("pull_request subject must not equal evaluated merge commit".to_owned());
            }
        }
        "push" => {
            if event_ref != "refs/heads/dev" {
                return Err("push event ref must be refs/heads/dev".to_owned());
            }
            if subject != evaluated {
                return Err("push subject must equal evaluated commit".to_owned());
            }
            if source.parents(evaluated)? != [protected.to_owned()] {
                return Err(
                    "push evaluated commit parents must be exactly [protected base]".to_owned(),
                );
            }
        }
        "merge_group" => {
            if !event_ref.starts_with("refs/heads/gh-readonly-queue/dev/") {
                return Err(
                    "merge_group event ref must be refs/heads/gh-readonly-queue/dev/...".to_owned(),
                );
            }
            if subject != evaluated {
                return Err("merge_group subject must equal evaluated commit".to_owned());
            }
        }
        _ => {
            return Err(
                "retirement SCM event must be pull_request, push, or merge_group".to_owned(),
            );
        }
    }
    Ok(())
}

fn selector_matches_path(selector: &ControlSelector, path: &str) -> bool {
    match selector.selector_type.as_str() {
        "exact" => selector.selector == path,
        "glob" => selector.selector.strip_suffix("/**").is_some_and(|prefix| {
            path.starts_with(prefix) && path.as_bytes().get(prefix.len()) == Some(&b'/')
        }),
        _ => false,
    }
}

fn validate_receipt_identity(
    stage: ReceiptStage,
    control: &ControlPlaneEntry,
    receipt_path: &str,
    receipt: &Value,
) -> Result<(), String> {
    let expected_id = match stage {
        ReceiptStage::PreparedNew => &control.preparation_artifact_id,
        ReceiptStage::ClosureNew | ReceiptStage::ClosedCarried => &control.closure_artifact_id,
        ReceiptStage::Dormant => return Err("dormant receipt identity".to_owned()),
    };
    if receipt.get("artifact_id").and_then(Value::as_str) != Some(expected_id)
        || receipt.get("scope_ref").and_then(Value::as_str) != Some(&control.scope_ref)
        || receipt
            .get("authority")
            .and_then(|authority| authority.get("planning_state"))
            .and_then(Value::as_str)
            != Some("HOLD(Planning)")
        || receipt
            .get("authority")
            .and_then(|authority| authority.get("dispatch_authorized"))
            .and_then(Value::as_bool)
            != Some(false)
    {
        return Err(format!(
            "receipt {receipt_path} is not bound to its control-plane identity"
        ));
    }
    if receipt.get("promoted_commit_oid").is_some()
        || receipt.get("postmerge_success").is_some()
        || receipt.get("verdict").is_some()
        || receipt.get("pass").is_some()
    {
        return Err(format!(
            "receipt {receipt_path} exceeds the E7 claim ceiling"
        ));
    }
    Ok(())
}

fn input_fact(
    source: &impl RetirementObjectSource,
    input: &ExpectedInput,
    predecessor: &BTreeMap<String, TreeEntry>,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
) -> Result<Value, String> {
    let predecessor_entry = predecessor
        .get(&input.path)
        .ok_or_else(|| format!("predecessor input {} disappeared", input.path))?;
    let protected_snapshot = path_snapshot(source, protected.get(&input.path))?;
    let candidate_snapshot = path_snapshot(source, candidate.get(&input.path))?;
    let predecessor_bytes = source.read_blob(&predecessor_entry.oid)?;
    let candidate_equivalent_paths =
        equivalent_paths(source, candidate, &input.path, &predecessor_bytes)?;
    let protected_equivalent_paths =
        equivalent_paths(source, protected, &input.path, &predecessor_bytes)?;
    let candidate_new_equivalent_paths = candidate_equivalent_paths
        .iter()
        .filter(|path| !protected_equivalent_paths.contains(path))
        .cloned()
        .collect::<Vec<_>>();
    Ok(json!({
        "path": input.path,
        "mode": input.mode,
        "predecessor_blob_oid": input.predecessor_blob_oid,
        "sha256": input.sha256,
        "byte_count": input.byte_count,
        "predecessor_path_exists": true,
        "predecessor_path_kind": "regular",
        "predecessor_sha256": input.sha256,
        "predecessor_byte_count": input.byte_count,
        "predecessor_mode": input.mode,
        "protected_path_exists": protected_snapshot.exists,
        "protected_path_kind": protected_snapshot.kind,
        "protected_blob_oid": protected_snapshot.blob_oid,
        "protected_sha256": protected_snapshot.sha256,
        "protected_byte_count": protected_snapshot.byte_count,
        "protected_mode": protected_snapshot.mode,
        "candidate_path_exists": candidate_snapshot.exists,
        "candidate_path_kind": candidate_snapshot.kind,
        "candidate_blob_oid": candidate_snapshot.blob_oid,
        "candidate_sha256": candidate_snapshot.sha256,
        "candidate_byte_count": candidate_snapshot.byte_count,
        "candidate_mode": candidate_snapshot.mode,
        "candidate_new_equivalent_paths": candidate_new_equivalent_paths,
        "candidate_equivalent_paths": candidate_equivalent_paths,
    }))
}

#[derive(Debug)]
struct PathSnapshot {
    exists: bool,
    kind: Value,
    blob_oid: Value,
    sha256: Value,
    byte_count: Value,
    mode: Value,
}

fn path_snapshot(
    source: &impl RetirementObjectSource,
    entry: Option<&TreeEntry>,
) -> Result<PathSnapshot, String> {
    let Some(entry) = entry else {
        return Ok(PathSnapshot {
            exists: false,
            kind: Value::Null,
            blob_oid: Value::Null,
            sha256: Value::Null,
            byte_count: Value::Null,
            mode: Value::Null,
        });
    };
    require_regular(entry, "retirement target")?;
    let bytes = source.read_blob(&entry.oid)?;
    Ok(PathSnapshot {
        exists: true,
        kind: json!("regular"),
        blob_oid: json!(entry.oid),
        sha256: json!(sha256_digest(&bytes)),
        byte_count: json!(bytes.len() as u64),
        mode: json!(entry.mode),
    })
}

fn equivalent_paths(
    source: &impl RetirementObjectSource,
    entries: &BTreeMap<String, TreeEntry>,
    original_path: &str,
    expected_bytes: &[u8],
) -> Result<Vec<String>, String> {
    let expected_hash = sha256_digest(expected_bytes);
    let expected_len = expected_bytes.len();
    let mut paths = Vec::new();
    for entry in entries.values().filter(|entry| entry.kind == "blob") {
        if entry.path == original_path {
            continue;
        }
        let bytes = source.read_blob(&entry.oid)?;
        if bytes.len() == expected_len
            && sha256_digest(&bytes) == expected_hash
            && bytes == expected_bytes
        {
            paths.push(entry.path.clone());
        }
    }
    paths.sort();
    Ok(paths)
}

fn coverage_scope(
    entry: &ControlPlaneEntry,
    predecessor: &BTreeMap<String, TreeEntry>,
    protected: &BTreeMap<String, TreeEntry>,
    candidate: &BTreeMap<String, TreeEntry>,
) -> Value {
    let selectors = entry
        .selectors
        .iter()
        .map(|selector| {
            let predecessor_paths = selector
                .expected_inputs
                .iter()
                .filter(|input| predecessor.contains_key(&input.path))
                .map(|input| input.path.clone())
                .collect::<Vec<_>>();
            let protected_paths = selector
                .expected_inputs
                .iter()
                .filter(|input| protected.contains_key(&input.path))
                .map(|input| input.path.clone())
                .collect::<Vec<_>>();
            let candidate_paths = selector
                .expected_inputs
                .iter()
                .filter(|input| candidate.contains_key(&input.path))
                .map(|input| input.path.clone())
                .collect::<Vec<_>>();
            let removed_paths = predecessor_paths
                .iter()
                .filter(|path| !candidate_paths.contains(path))
                .cloned()
                .collect::<Vec<_>>();
            let surviving_paths = predecessor_paths
                .iter()
                .filter(|path| candidate_paths.contains(path))
                .cloned()
                .collect::<Vec<_>>();
            let candidate_only_paths = candidate_paths
                .iter()
                .filter(|path| !predecessor_paths.contains(path))
                .cloned()
                .collect::<Vec<_>>();
            json!({
                "selector_type": selector.selector_type,
                "selector": selector.selector,
                "protected_paths": protected_paths,
                "predecessor_paths": predecessor_paths,
                "candidate_paths": candidate_paths,
                "removed_paths": removed_paths,
                "surviving_paths": surviving_paths,
                "candidate_only_paths": candidate_only_paths,
                "external_assertion": false,
            })
        })
        .collect::<Vec<_>>();
    let required_retired_paths = entry
        .selectors
        .iter()
        .flat_map(|selector| selector.expected_inputs.iter())
        .map(|input| input.path.clone())
        .collect::<BTreeSet<_>>();
    json!({
        "scope_ref": entry.scope_ref,
        "scope_type": entry.scope_type,
        "selectors": selectors,
        "required_retired_paths": required_retired_paths,
    })
}

fn closure_preparation_link(receipt: &Value) -> Result<(&str, &str), String> {
    let preparation = receipt
        .get("protected_preparation")
        .and_then(Value::as_object)
        .ok_or_else(|| "closure receipt has no protected_preparation".to_owned())?;
    let path = required_value_string(
        preparation.get("receipt_path"),
        "protected_preparation.receipt_path",
    )?;
    let blob = required_value_string(
        preparation.get("receipt_blob_oid"),
        "protected_preparation.receipt_blob_oid",
    )?;
    validate_repo_path(path)?;
    validate_oid(blob, "protected preparation blob")?;
    Ok((path, blob))
}

fn receipt_baseline(receipt: &Value) -> Result<(String, String), String> {
    let baseline = receipt
        .get("baseline")
        .and_then(Value::as_object)
        .ok_or_else(|| "preparation receipt has no baseline".to_owned())?;
    let commit = required_value_string(baseline.get("commit_oid"), "baseline.commit_oid")?;
    let tree = required_value_string(baseline.get("tree_oid"), "baseline.tree_oid")?;
    validate_oid(commit, "receipt baseline commit")?;
    validate_oid(tree, "receipt baseline tree")?;
    Ok((commit.to_owned(), tree.to_owned()))
}

fn require_predecessor_baseline(
    commit_oid: &str,
    tree_oid: &str,
    predecessor_commit_oid: &str,
    predecessor_tree_oid: &str,
    receipt_path: &str,
) -> Result<(), String> {
    if commit_oid != predecessor_commit_oid || tree_oid != predecessor_tree_oid {
        return Err(format!(
            "receipt {receipt_path} baseline is not the immutable control-plane predecessor"
        ));
    }
    Ok(())
}

fn find_linked_preparation(
    source: &impl RetirementObjectSource,
    protected_commit: &str,
    path: &str,
    blob_oid: &str,
    control: &ControlPlaneEntry,
) -> Result<(String, String), String> {
    let blob_bytes = source.read_blob(blob_oid)?;
    let preparation: Value = parse_closed_json(&blob_bytes)?;
    validate_receipt_identity(ReceiptStage::PreparedNew, control, path, &preparation)?;
    let baseline = receipt_baseline(&preparation)?;
    for commit in source.commits_touching_path(protected_commit, path)? {
        let entries = entries_by_path(source.tree_entries(&commit)?)?;
        if entries
            .get(path)
            .is_some_and(|entry| entry.oid == blob_oid && entry.is_regular_blob())
        {
            return Ok(baseline);
        }
    }
    Err(format!(
        "linked preparation object {blob_oid} at {path} is not reachable in protected history"
    ))
}

fn control_entry_value(entry: &ControlPlaneEntry) -> Result<Value, String> {
    serde_json::to_value(entry)
        .map_err(|error| format!("serialize retirement control-plane entry: {error}"))
}

fn canonical_value_sha256(value: &Value) -> Result<String, String> {
    let bytes = semantic_canonical_json(value)?;
    Ok(sha256_digest(bytes.as_bytes()))
}

fn semantic_canonical_json(value: &Value) -> Result<String, String> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(value.to_string()),
        Value::String(_) => serde_json::to_string(value)
            .map_err(|error| format!("canonicalize retirement string: {error}")),
        Value::Array(values) => values
            .iter()
            .map(semantic_canonical_json)
            .collect::<Result<Vec<_>, _>>()
            .map(|values| format!("[{}]", values.join(","))),
        Value::Object(values) => {
            let ordered = values.iter().collect::<BTreeMap<_, _>>();
            let fields = ordered
                .into_iter()
                .map(|(key, value)| {
                    Ok(format!(
                        "{}:{}",
                        serde_json::to_string(key)
                            .map_err(|error| format!("canonicalize retirement key: {error}"))?,
                        semantic_canonical_json(value)?
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(format!("{{{}}}", fields.join(",")))
        }
    }
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn entries_by_path(entries: Vec<TreeEntry>) -> Result<BTreeMap<String, TreeEntry>, String> {
    let mut result = BTreeMap::new();
    for entry in entries {
        validate_repo_path(&entry.path)?;
        if result.insert(entry.path.clone(), entry).is_some() {
            return Err("Git tree contains a duplicate path".to_owned());
        }
    }
    Ok(result)
}

fn receipt_root_inventory(entries: &BTreeMap<String, TreeEntry>) -> BTreeSet<String> {
    let prefix = format!("{RECEIPT_ROOT}/");
    entries
        .keys()
        .filter(|path| path.starts_with(&prefix))
        .cloned()
        .collect()
}

fn expected_receipt_paths(control: &RetirementControlPlane) -> BTreeSet<String> {
    control
        .entries
        .iter()
        .flat_map(|entry| {
            [
                entry.preparation_receipt_path.clone(),
                entry.closure_receipt_path.clone(),
            ]
        })
        .collect()
}

fn require_regular(entry: &TreeEntry, label: &str) -> Result<(), String> {
    if entry.is_regular_blob() {
        Ok(())
    } else {
        Err(format!(
            "{label} {} must be exact 100644 blob, found {} {}",
            entry.path, entry.mode, entry.kind
        ))
    }
}

fn parse_ls_tree(bytes: &[u8]) -> Result<Vec<TreeEntry>, String> {
    let mut entries = Vec::new();
    for record in bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| "git ls-tree record has no path separator".to_owned())?;
        let header = std::str::from_utf8(&record[..tab])
            .map_err(|error| format!("git ls-tree header is not UTF-8: {error}"))?;
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|error| format!("git path is not UTF-8: {error}"))?;
        let mut fields = header.split_ascii_whitespace();
        let mode = fields.next().unwrap_or_default();
        let kind = fields.next().unwrap_or_default();
        let oid = fields.next().unwrap_or_default();
        if fields.next().is_some() || mode.is_empty() || kind.is_empty() {
            return Err("git ls-tree record has invalid metadata".to_owned());
        }
        validate_oid(oid, "git tree object")?;
        entries.push(TreeEntry {
            mode: mode.to_owned(),
            kind: kind.to_owned(),
            oid: oid.to_owned(),
            path: path.to_owned(),
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(entries)
}

fn parse_oid_text(bytes: &[u8], label: &str) -> Result<String, String> {
    let value = std::str::from_utf8(bytes)
        .map_err(|error| format!("{label} is not UTF-8: {error}"))?
        .trim();
    validate_oid(value, label)?;
    Ok(value.to_owned())
}

fn validate_oid(value: &str, label: &str) -> Result<(), String> {
    if value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(format!("{label} is not a lowercase SHA-1 object id"))
    }
}

fn validate_sha256(value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err("retirement SHA-256 must use sha256: prefix".to_owned());
    };
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err("retirement SHA-256 is not canonical lowercase hex".to_owned())
    }
}

fn validate_repo_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with("./")
        || path.contains("//")
        || path.split('/').any(|part| part == ".." || part.is_empty())
        || path.contains('\0')
    {
        Err(format!(
            "retirement path {path:?} is not canonical repo-relative"
        ))
    } else {
        Ok(())
    }
}

fn required_value_string<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("retirement {label} is missing"))
}

fn parse_closed_json<T>(bytes: &[u8]) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut duplicate_deserializer = serde_json::Deserializer::from_slice(bytes);
    DuplicateKeyFreeJson::deserialize(&mut duplicate_deserializer)
        .map_err(|error| format!("retirement JSON duplicate-key check: {error}"))?;
    duplicate_deserializer
        .end()
        .map_err(|error| format!("retirement JSON trailing data: {error}"))?;
    serde_json::from_slice(bytes).map_err(|error| format!("retirement JSON parse: {error}"))
}

struct DuplicateKeyFreeJson;

impl<'de> Deserialize<'de> for DuplicateKeyFreeJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DuplicateKeyFreeJsonVisitor)
    }
}

struct DuplicateKeyFreeJsonVisitor;

impl<'de> Visitor<'de> for DuplicateKeyFreeJsonVisitor {
    type Value = DuplicateKeyFreeJson;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DuplicateKeyFreeJson::deserialize(deserializer)
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element::<DuplicateKeyFreeJson>()?.is_some() {}
        Ok(DuplicateKeyFreeJson)
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate object key: {key}")));
            }
            map.next_value::<DuplicateKeyFreeJson>()?;
        }
        Ok(DuplicateKeyFreeJson)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREDECESSOR: &str = "1111111111111111111111111111111111111111";
    const PREDECESSOR_TREE: &str = "2222222222222222222222222222222222222222";
    const PROTECTED: &str = "3333333333333333333333333333333333333333";
    const PROTECTED_TREE: &str = "4444444444444444444444444444444444444444";
    const CANDIDATE: &str = "5555555555555555555555555555555555555555";
    const CANDIDATE_TREE: &str = "6666666666666666666666666666666666666666";
    const OTHER_COMMIT: &str = "7777777777777777777777777777777777777777";
    #[derive(Clone)]
    struct FakeSource {
        commits: BTreeMap<String, String>,
        first_parent: String,
        parents: Vec<String>,
        ancestry: BTreeSet<(String, String)>,
        trees: BTreeMap<String, Vec<TreeEntry>>,
        blobs: BTreeMap<String, Vec<u8>>,
        history: BTreeMap<String, Vec<String>>,
    }

    impl RetirementObjectSource for FakeSource {
        fn resolve_commit(&self, revision: &str) -> Result<String, String> {
            self.commits
                .get(revision)
                .cloned()
                .ok_or_else(|| format!("missing commit {revision}"))
        }
        fn tree_for_commit(&self, commit_oid: &str) -> Result<String, String> {
            self.commits
                .get(&format!("tree:{commit_oid}"))
                .cloned()
                .ok_or_else(|| "missing tree".to_owned())
        }
        fn first_parent(&self, _commit_oid: &str) -> Result<String, String> {
            Ok(self.first_parent.clone())
        }
        fn parents(&self, _commit_oid: &str) -> Result<Vec<String>, String> {
            Ok(self.parents.clone())
        }
        fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool, String> {
            Ok(self
                .ancestry
                .contains(&(ancestor.to_owned(), descendant.to_owned())))
        }
        fn tree_entries(&self, commit_oid: &str) -> Result<Vec<TreeEntry>, String> {
            self.trees
                .get(commit_oid)
                .cloned()
                .ok_or_else(|| format!("missing entries {commit_oid}"))
        }
        fn read_blob(&self, blob_oid: &str) -> Result<Vec<u8>, String> {
            self.blobs
                .get(blob_oid)
                .cloned()
                .ok_or_else(|| format!("missing blob {blob_oid}"))
        }
        fn commits_touching_path(
            &self,
            _commit_oid: &str,
            path: &str,
        ) -> Result<Vec<String>, String> {
            Ok(self.history.get(path).cloned().unwrap_or_default())
        }
    }

    fn oid(byte: u8) -> String {
        format!("{byte:040x}")
    }

    fn input(path: &str, bytes: &[u8], blob_oid: String) -> ExpectedInput {
        ExpectedInput {
            path: path.to_owned(),
            mode: "100644".to_owned(),
            predecessor_blob_oid: blob_oid,
            sha256: sha256_digest(bytes),
            byte_count: bytes.len() as u64,
        }
    }

    fn control_plane() -> RetirementControlPlane {
        let bodies = [
            ("docs/ROADMAP.md", b"roadmap".as_slice()),
            (".omc/ultragoal/OWNERS", b"owners".as_slice()),
            (
                ".omc/ultragoal/TEAMMATE-PREAMBLE.md",
                b"preamble".as_slice(),
            ),
            (".omc/ultragoal/friction-ledger.jsonl", b"ledger".as_slice()),
            (".omc/ultragoal/premise.txt", b"premise".as_slice()),
            (
                "docs/ideas/archive/cloud-intelligence-bedrock-on-talos-2026-05-28.md",
                b"idea-a".as_slice(),
            ),
            (
                "docs/ideas/archive/cloud-intelligence-v1-pipeline-2026-05-28.md",
                b"idea-b".as_slice(),
            ),
            (
                "docs/ideas/archive/n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md",
                b"idea-c".as_slice(),
            ),
        ];
        RetirementControlPlane {
            schema: CONTROL_PLANE_SCHEMA.to_owned(),
            schema_version: 1,
            canonical_name: CONTROL_PLANE_NAME.to_owned(),
            planning_state: "HOLD(Planning)".to_owned(),
            dispatch_authorized: false,
            receipt_root: RECEIPT_ROOT.to_owned(),
            predecessor_snapshot: CommitTree {
                commit_oid: PREDECESSOR.to_owned(),
                tree_oid: PREDECESSOR_TREE.to_owned(),
            },
            entries: vec![
                ControlPlaneEntry {
                    selectors: vec![ControlSelector {
                        selector_type: "exact".to_owned(),
                        selector: "docs/ROADMAP.md".to_owned(),
                        expected_inputs: vec![input(bodies[0].0, bodies[0].1, oid(10))],
                    }],
                    ..fixed_entry(
                        "artifact:masterplan",
                        "masterplan-retired-surfaces",
                        MASTERPLAN_EVIDENCE_SET_ID,
                        MASTERPLAN_PREPARATION_ID,
                        MASTERPLAN_PREPARATION_PATH,
                        MASTERPLAN_CLOSURE_ID,
                        MASTERPLAN_CLOSURE_PATH,
                    )
                },
                ControlPlaneEntry {
                    selectors: bodies[1..5]
                        .iter()
                        .enumerate()
                        .map(|(index, (path, bytes))| ControlSelector {
                            selector_type: "exact".to_owned(),
                            selector: (*path).to_owned(),
                            expected_inputs: vec![input(path, bytes, oid(11 + index as u8))],
                        })
                        .collect(),
                    ..fixed_entry(
                        "ADR-0363",
                        "amended-agentic-vcs-retirement",
                        ADR_0363_EVIDENCE_SET_ID,
                        ADR_0363_PREPARATION_ID,
                        ADR_0363_PREPARATION_PATH,
                        ADR_0363_CLOSURE_ID,
                        ADR_0363_CLOSURE_PATH,
                    )
                },
                ControlPlaneEntry {
                    selectors: vec![ControlSelector {
                        selector_type: "glob".to_owned(),
                        selector: "docs/ideas/archive/**".to_owned(),
                        expected_inputs: bodies[5..]
                            .iter()
                            .enumerate()
                            .map(|(index, (path, bytes))| input(path, bytes, oid(20 + index as u8)))
                            .collect(),
                    }],
                    ..fixed_entry(
                        "ADR-0388",
                        "transient-ideas",
                        ADR_0388_EVIDENCE_SET_ID,
                        ADR_0388_PREPARATION_ID,
                        ADR_0388_PREPARATION_PATH,
                        ADR_0388_CLOSURE_ID,
                        ADR_0388_CLOSURE_PATH,
                    )
                },
            ],
        }
    }

    fn fixture() -> FakeSource {
        let control = control_plane();
        let control_bytes = to_canonical_json(&serde_json::to_value(&control).unwrap())
            .unwrap()
            .into_bytes();
        let control_oid = oid(90);
        let mut predecessor_entries = Vec::new();
        let mut blobs = BTreeMap::from([(control_oid.clone(), control_bytes)]);
        for input in control
            .entries
            .iter()
            .flat_map(|entry| entry.selectors.iter())
            .flat_map(|selector| selector.expected_inputs.iter())
        {
            let bytes = input.path.rsplit('/').next().unwrap().as_bytes().to_vec();
            // Replace the synthetic input bytes with bytes matching its declared digest.
            let declared = match input.path.as_str() {
                "docs/ROADMAP.md" => b"roadmap".to_vec(),
                ".omc/ultragoal/OWNERS" => b"owners".to_vec(),
                ".omc/ultragoal/TEAMMATE-PREAMBLE.md" => b"preamble".to_vec(),
                ".omc/ultragoal/friction-ledger.jsonl" => b"ledger".to_vec(),
                ".omc/ultragoal/premise.txt" => b"premise".to_vec(),
                path if path.contains("bedrock") => b"idea-a".to_vec(),
                path if path.contains("v1-pipeline") => b"idea-b".to_vec(),
                _ => b"idea-c".to_vec(),
            };
            let _ = bytes;
            blobs.insert(input.predecessor_blob_oid.clone(), declared);
            predecessor_entries.push(TreeEntry {
                mode: "100644".to_owned(),
                kind: "blob".to_owned(),
                oid: input.predecessor_blob_oid.clone(),
                path: input.path.clone(),
            });
        }
        let candidate_entries = vec![TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: control_oid,
            path: CONTROL_PLANE_PATH.to_owned(),
        }];
        FakeSource {
            commits: BTreeMap::from([
                ("HEAD".to_owned(), CANDIDATE.to_owned()),
                (CANDIDATE.to_owned(), CANDIDATE.to_owned()),
                (PROTECTED.to_owned(), PROTECTED.to_owned()),
                (PREDECESSOR.to_owned(), PREDECESSOR.to_owned()),
                (format!("tree:{CANDIDATE}"), CANDIDATE_TREE.to_owned()),
                (format!("tree:{PROTECTED}"), PROTECTED_TREE.to_owned()),
                (format!("tree:{PREDECESSOR}"), PREDECESSOR_TREE.to_owned()),
            ]),
            first_parent: PROTECTED.to_owned(),
            parents: vec![PROTECTED.to_owned()],
            ancestry: BTreeSet::from([
                (PROTECTED.to_owned(), CANDIDATE.to_owned()),
                (PREDECESSOR.to_owned(), PROTECTED.to_owned()),
            ]),
            trees: BTreeMap::from([
                (PREDECESSOR.to_owned(), predecessor_entries),
                (PROTECTED.to_owned(), Vec::new()),
                (CANDIDATE.to_owned(), candidate_entries),
            ]),
            blobs,
            history: BTreeMap::new(),
        }
    }

    fn context() -> RetirementMaterializationContext<'static> {
        RetirementMaterializationContext {
            control_plane_path: CONTROL_PLANE_PATH,
            protected_base_commit: PROTECTED,
            evaluated_commit: CANDIDATE,
            scm_event_name: "push",
            scm_event_ref: "refs/heads/dev",
            subject_commit: CANDIDATE,
        }
    }

    fn assert_public_consumer_accepts(source: &FakeSource, facts: &Value) {
        let raw_receipts = facts["receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(
                |metadata| ci_cross_artifact_agreement::RawHistoryOnlyRetirementReceipt {
                    receipt_path: metadata["receipt_path"].as_str().unwrap(),
                    bytes: source
                        .blobs
                        .get(metadata["candidate_receipt_blob_oid"].as_str().unwrap())
                        .unwrap(),
                },
            )
            .collect::<Vec<_>>();
        let control_plane_bytes = source
            .blobs
            .get(
                facts["scm_facts"]["retirement_control_plane_context"]
                    ["candidate_control_plane_blob_oid"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
        let evaluation = ci_cross_artifact_agreement::
            evaluate_and_project_history_only_retirement_facts_with_control_plane(
                facts,
                &raw_receipts,
                control_plane_bytes,
            );
        assert!(
            evaluation.findings.is_empty(),
            "producer/consumer semantic drift: {:?}",
            evaluation.findings
        );
    }

    #[test]
    fn event_identity_rejects_pr_parent_order_extra_parent_and_subject_aliases() {
        let mut source = fixture();
        let mut context = context();
        context.scm_event_name = "pull_request";
        context.scm_event_ref = "refs/pull/123/merge";
        context.subject_commit = PREDECESSOR;
        source.parents = vec![PROTECTED.to_owned(), PREDECESSOR.to_owned()];
        assert!(materialize_history_only_retirement_facts(&source, &context).is_ok());

        for parents in [
            vec![PREDECESSOR.to_owned(), PROTECTED.to_owned()],
            vec![
                PROTECTED.to_owned(),
                PREDECESSOR.to_owned(),
                CANDIDATE.to_owned(),
            ],
            vec![PROTECTED.to_owned(), CANDIDATE.to_owned()],
        ] {
            source.parents = parents;
            assert!(materialize_history_only_retirement_facts(&source, &context).is_err());
        }
    }

    #[test]
    fn event_identity_rejects_nonself_push_and_merge_group_subjects() {
        for event in ["push", "merge_group"] {
            let source = fixture();
            let mut context = context();
            context.scm_event_name = event;
            context.scm_event_ref = if event == "push" {
                "refs/heads/dev"
            } else {
                "refs/heads/gh-readonly-queue/dev/pr-123"
            };
            context.subject_commit = PREDECESSOR;
            assert!(materialize_history_only_retirement_facts(&source, &context).is_err());
        }
    }

    #[test]
    fn event_identity_rejects_push_merge_topology() {
        let mut source = fixture();
        source.parents = vec![PROTECTED.to_owned(), PREDECESSOR.to_owned()];

        let error = materialize_history_only_retirement_facts(&source, &context())
            .expect_err("push must not accept a direct merge topology");
        assert!(
            error.contains("push evaluated commit parents"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn event_identity_rejects_non_dev_push_ref() {
        let mut context = context();
        context.scm_event_ref = "refs/heads/contributor";

        let error = materialize_history_only_retirement_facts(&fixture(), &context)
            .expect_err("pushes outside dev must fail closed");
        assert!(
            error.contains("refs/heads/dev"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn event_identity_rejects_evaluated_commit_away_from_head() {
        let mut source = fixture();
        source
            .commits
            .insert(OTHER_COMMIT.to_owned(), OTHER_COMMIT.to_owned());
        let mut context = context();
        context.evaluated_commit = OTHER_COMMIT;

        let error = materialize_history_only_retirement_facts(&source, &context)
            .expect_err("evaluated commit must resolve to HEAD");
        assert!(
            error.contains("not exact HEAD"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn event_identity_rejects_push_with_provider_first_parent_mismatch() {
        let mut source = fixture();
        source.first_parent = PREDECESSOR.to_owned();

        let error = materialize_history_only_retirement_facts(&source, &context)
            .expect_err("push first parent must equal provider protected SHA");
        assert!(
            error.contains("not candidate first parent"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn merge_group_keeps_evaluated_self_without_contributor_identity() {
        let source = fixture();
        let mut context = context();
        context.scm_event_name = "merge_group";
        context.scm_event_ref = "refs/heads/gh-readonly-queue/dev/pr-123";

        let facts = materialize_history_only_retirement_facts(&source, &context)
            .expect("merge-group evaluated-self topology remains valid");
        assert!(
            !facts.to_string().contains("contributor"),
            "merge-group facts must not invent a contributor field"
        );
    }

    #[test]
    fn event_identity_rejects_merge_group_for_non_dev_target() {
        let source = fixture();
        let mut context = context();
        context.scm_event_name = "merge_group";
        context.scm_event_ref = "refs/heads/gh-readonly-queue/release/pr-123";

        let error = materialize_history_only_retirement_facts(&source, &context)
            .expect_err("merge groups targeting a branch other than dev must fail closed");
        assert!(
            error.contains("refs/heads/gh-readonly-queue/dev/"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn event_identity_rejects_merge_group_dev_prefix_collision() {
        let source = fixture();
        let mut context = context();
        context.scm_event_name = "merge_group";
        context.scm_event_ref = "refs/heads/gh-readonly-queue/devil/pr-123";

        let error = materialize_history_only_retirement_facts(&source, &context)
            .expect_err("merge-group target matching must preserve the dev path separator");
        assert!(
            error.contains("refs/heads/gh-readonly-queue/dev/"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn event_identity_rejects_merge_group_nested_dev_branch_collision() {
        let source = fixture();
        let mut context = context();
        context.scm_event_name = "merge_group";
        context.scm_event_ref = "refs/heads/gh-readonly-queue/dev/release/pr-123";

        let error = materialize_history_only_retirement_facts(&source, &context)
            .expect_err("a merge group for dev/release must not be labeled origin/dev");
        assert!(
            error.contains("protected base ref"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn event_identity_rejects_revision_aliases_and_noncanonical_oids() {
        let mut source = fixture();
        source
            .commits
            .insert("alias".to_owned(), CANDIDATE.to_owned());
        let mut context = context();
        context.evaluated_commit = "alias";
        assert!(materialize_history_only_retirement_facts(&source, &context).is_err());

        let mut context = context();
        context.protected_base_commit = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert!(materialize_history_only_retirement_facts(&source, &context).is_err());
    }

    #[test]
    fn closure_and_closed_carried_identity_mutations_fail_closed() {
        let control = control_plane().entries.remove(0);
        for stage in [ReceiptStage::ClosureNew, ReceiptStage::ClosedCarried] {
            for mutation in [
                "artifact_id",
                "scope_ref",
                "planning_state",
                "dispatch_authorized",
            ] {
                let mut receipt = receipt_value(&control, true, Some(BLOB));
                match mutation {
                    "artifact_id" => receipt["artifact_id"] = json!("wrong"),
                    "scope_ref" => receipt["scope_ref"] = json!("wrong"),
                    "planning_state" => receipt["authority"]["planning_state"] = json!("ACTIVE"),
                    "dispatch_authorized" => {
                        receipt["authority"]["dispatch_authorized"] = json!(true)
                    }
                    _ => unreachable!(),
                }
                assert!(
                    validate_receipt_identity(stage, &control, "receipt.json", &receipt).is_err(),
                    "{stage:?} must reject mutated {mutation}"
                );
            }
        }
    }

    fn control_oid(source: &FakeSource) -> String {
        source
            .trees
            .get(CANDIDATE)
            .unwrap()
            .iter()
            .find(|entry| entry.path == CONTROL_PLANE_PATH)
            .unwrap()
            .oid
            .clone()
    }

    fn add_protected_control_plane(source: &mut FakeSource) {
        let control_oid = control_oid(source);
        source.trees.get_mut(PROTECTED).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: control_oid,
            path: CONTROL_PLANE_PATH.to_owned(),
        });
    }

    fn add_current_bodies(source: &mut FakeSource, commit: &str) {
        let bodies = source.trees.get(PREDECESSOR).unwrap().clone();
        source.trees.get_mut(commit).unwrap().extend(bodies);
    }

    fn receipt_value(
        entry: &ControlPlaneEntry,
        closure: bool,
        preparation_blob: Option<&str>,
    ) -> Value {
        let mut value = json!({
            "artifact_id": if closure { &entry.closure_artifact_id } else { &entry.preparation_artifact_id },
            "scope_ref": entry.scope_ref,
            "authority": {
                "planning_state": "HOLD(Planning)",
                "dispatch_authorized": false,
            },
            "baseline": {
                "commit_oid": PREDECESSOR,
                "tree_oid": PREDECESSOR_TREE,
            },
        });
        if let Some(blob) = preparation_blob {
            value["protected_preparation"] = json!({
                "receipt_path": entry.preparation_receipt_path,
                "receipt_blob_oid": blob,
            });
        }
        value
    }

    fn add_receipt(
        source: &mut FakeSource,
        commit: &str,
        path: &str,
        blob_oid: String,
        receipt: &Value,
    ) {
        source.blobs.insert(
            blob_oid.clone(),
            to_canonical_json(receipt).unwrap().into_bytes(),
        );
        source.trees.get_mut(commit).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: blob_oid,
            path: path.to_owned(),
        });
    }

    #[test]
    fn bootstrap_is_candidate_bound_three_row_empty_and_deterministic() {
        let source = fixture();
        let first = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        let second = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        assert_eq!(first, second);
        assert_eq!(first["receipts"], json!([]));
        assert_eq!(
            first["scm_facts"]["retirement_receipt_object_facts"],
            json!([])
        );
        assert_eq!(
            first["scm_facts"]["retirement_receipt_coverage"]["scopes"],
            json!([])
        );
        assert_eq!(
            first["scm_facts"]["retirement_control_plane_context"]["bootstrap"],
            json!(true)
        );
        assert!(first["scm_facts"]["retirement_control_plane_context"]["protected_control_plane_blob_oid"].is_null());
        assert!(first["scm_facts"]["retirement_control_plane_context"]["candidate_control_plane_blob_oid"].as_str().is_some());
        assert_eq!(
            first["scm_facts"]["retirement_control_plane_context"]["control_plane_entries"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        let rendered = to_canonical_json(&first).unwrap();
        assert!(!rendered.contains("verdict"));
        assert!(!rendered.contains("PASS"));
        assert!(!rendered.contains("roadmap_author"));
    }

    #[test]
    fn bootstrap_rejects_nonempty_receipt_population() {
        let mut source = fixture();
        let receipt_oid = oid(99);
        source
            .blobs
            .insert(receipt_oid.clone(), br#"{"artifact_id":"x"}"#.to_vec());
        source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: receipt_oid,
            path: MASTERPLAN_PREPARATION_PATH.to_owned(),
        });
        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("not atomic") || error.contains("may not add receipts"));
    }

    #[test]
    fn prepared_new_is_atomic_three_receipt_facts_without_projection_or_copies() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        add_current_bodies(&mut source, CANDIDATE);
        for (index, entry) in control_plane().entries.iter().enumerate() {
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.preparation_receipt_path,
                oid(100 + index as u8),
                &receipt_value(entry, false, None),
            );
        }

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        assert_eq!(facts["receipts"].as_array().unwrap().len(), 3);
        assert_eq!(
            facts["scm_facts"]["retirement_control_plane_context"]["lifecycle_state"],
            json!("prepared-new")
        );
        assert_eq!(
            facts["scm_facts"]["retirement_receipt_coverage"]["scopes"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        for fact in facts["scm_facts"]["retirement_receipt_object_facts"]
            .as_array()
            .unwrap()
        {
            assert_eq!(fact["receipt_state"], json!("prepared-new"));
            assert_eq!(
                fact["predecessor_context"]["source"],
                json!("control-plane-predecessor")
            );
            for input in fact["retired_inputs"].as_array().unwrap() {
                assert_eq!(input["mode"], json!("100644"));
                assert_eq!(input["candidate_equivalent_paths"], json!([]));
                assert_eq!(input["candidate_new_equivalent_paths"], json!([]));
            }
        }
        let protected_context = facts["scm_facts"]["protected_scm_context"]
            .as_object()
            .unwrap();
        assert!(!protected_context.contains_key("prepared_receipt_paths"));
        assert!(!protected_context.contains_key("control_plane_entries"));
        let rendered = to_canonical_json(&facts).unwrap();
        assert!(!rendered.contains("closure_projection"));
        assert!(!rendered.contains("verdict"));
    }

    #[test]
    fn prepared_new_contract_regression_validates_public_consumer_without_drift_findings() {
        // Cross-crate contract regression: this remains a producer unit test because the
        // narrow FakeSource seam is private; it deliberately does not exercise Git/filesystem.
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        add_current_bodies(&mut source, CANDIDATE);
        for (index, entry) in control_plane().entries.iter().enumerate() {
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.preparation_receipt_path,
                oid(100 + index as u8),
                &receipt_value(entry, false, None),
            );
        }

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        assert_public_consumer_accepts(&source, &facts);
    }

    #[test]
    fn prepared_new_never_projects_raw_receipt_bodies_or_authority_lookalikes() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        add_current_bodies(&mut source, CANDIDATE);
        for (index, entry) in control_plane().entries.iter().enumerate() {
            let mut receipt = receipt_value(entry, false, None);
            match index {
                0 => receipt["retired_body"] = json!("TOP-SECRET-RETIRED-BODY"),
                1 => receipt["PASS"] = json!(true),
                2 => {
                    receipt["authority"]["roadmap_dispatch_authorized"] = json!(true);
                    receipt["qualified_human_authority"] = json!({"verdict": "PASS"});
                }
                _ => unreachable!(),
            }
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.preparation_receipt_path,
                oid(106 + index as u8),
                &receipt,
            );
        }

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        let expected_keys = BTreeSet::from([
            "artifact_id",
            "baseline_commit_oid",
            "baseline_tree_oid",
            "candidate_receipt_blob_oid",
            "candidate_receipt_sha256",
            "receipt_path",
            "receipt_state",
            "scope_ref",
        ]);
        for receipt in facts["receipts"].as_array().unwrap() {
            let actual_keys: BTreeSet<&str> = receipt
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            assert_eq!(actual_keys, expected_keys);
        }
        let rendered = to_canonical_json(&facts).unwrap();
        for forbidden in [
            "TOP-SECRET-RETIRED-BODY",
            "retired_body",
            "PASS",
            "roadmap_dispatch_authorized",
            "qualified_human_authority",
            "verdict",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "controller facts leaked hostile receipt field {forbidden:?}: {rendered}"
            );
        }
    }

    #[test]
    fn emitter_rejects_arbitrary_and_absolute_generated_facts_paths_before_git() {
        for output_path in [
            Path::new("ci/facade/scm-facts-snapshot/not-canonical.generated.json"),
            Path::new(
                "./ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json",
            ),
            Path::new("/tmp/history-only-retirement-facts.generated.json"),
        ] {
            let error = emit_history_only_retirement_facts(Path::new("."), &context(), output_path)
                .unwrap_err();
            assert!(
                error.contains("exact canonical repo-relative generated facts path"),
                "unexpected error for {output_path:?}: {error}"
            );
        }
    }

    #[test]
    fn prepared_new_rejects_receipt_baseline_other_than_control_plane_predecessor() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        add_current_bodies(&mut source, CANDIDATE);
        for (index, entry) in control_plane().entries.iter().enumerate() {
            let mut receipt = receipt_value(entry, false, None);
            if index == 0 {
                receipt["baseline"] = json!({
                    "commit_oid": PROTECTED,
                    "tree_oid": PROTECTED_TREE,
                });
            }
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.preparation_receipt_path,
                oid(103 + index as u8),
                &receipt,
            );
        }

        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("immutable control-plane predecessor"));
    }

    #[test]
    fn closure_new_binds_each_candidate_closure_to_protected_preparation() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        let control = control_plane();
        for (index, entry) in control.entries.iter().enumerate() {
            let preparation_oid = oid(110 + index as u8);
            add_receipt(
                &mut source,
                PROTECTED,
                &entry.preparation_receipt_path,
                preparation_oid.clone(),
                &receipt_value(entry, false, None),
            );
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.closure_receipt_path,
                oid(120 + index as u8),
                &receipt_value(entry, true, Some(&preparation_oid)),
            );
        }

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        assert_eq!(
            facts["scm_facts"]["retirement_control_plane_context"]["lifecycle_state"],
            json!("closure-new")
        );
        assert_eq!(
            facts["scm_facts"]["protected_scm_context"]["protected_preparation_receipts"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        for fact in facts["scm_facts"]["retirement_receipt_object_facts"]
            .as_array()
            .unwrap()
        {
            assert_eq!(fact["receipt_state"], json!("closure-new"));
            assert_eq!(
                fact["predecessor_context"]["source"],
                json!("protected-preparation-receipt")
            );
            assert!(fact["preparation_receipt_path"].as_str().is_some());
            for input in fact["retired_inputs"].as_array().unwrap() {
                assert_eq!(input["protected_path_exists"], json!(true));
                assert_eq!(input["candidate_path_exists"], json!(false));
            }
        }
        assert_public_consumer_accepts(&source, &facts);
    }

    #[test]
    fn closure_new_rejects_candidate_link_to_wrong_protected_preparation_blob() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        let control = control_plane();
        for (index, entry) in control.entries.iter().enumerate() {
            let preparation_oid = oid(113 + index as u8);
            add_receipt(
                &mut source,
                PROTECTED,
                &entry.preparation_receipt_path,
                preparation_oid.clone(),
                &receipt_value(entry, false, None),
            );
            let linked_oid = if index == 0 {
                oid(119)
            } else {
                preparation_oid
            };
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.closure_receipt_path,
                oid(120 + index as u8),
                &receipt_value(entry, true, Some(&linked_oid)),
            );
        }

        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("links unexpected protected preparation blob"));
    }

    #[test]
    fn closure_new_rejects_protected_preparation_with_different_baseline() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        let control = control_plane();
        for (index, entry) in control.entries.iter().enumerate() {
            let preparation_oid = oid(123 + index as u8);
            let mut preparation = receipt_value(entry, false, None);
            if index == 0 {
                preparation["baseline"] = json!({
                    "commit_oid": PROTECTED,
                    "tree_oid": PROTECTED_TREE,
                });
            }
            add_receipt(
                &mut source,
                PROTECTED,
                &entry.preparation_receipt_path,
                preparation_oid.clone(),
                &preparation,
            );
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.closure_receipt_path,
                oid(126 + index as u8),
                &receipt_value(entry, true, Some(&preparation_oid)),
            );
        }

        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("immutable control-plane predecessor"));
    }

    #[test]
    fn closed_carried_uses_reachable_linked_preparation_history() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        let control = control_plane();
        for (index, entry) in control.entries.iter().enumerate() {
            let preparation_oid = oid(130 + index as u8);
            let closure_oid = oid(140 + index as u8);
            let preparation = receipt_value(entry, false, None);
            source.blobs.insert(
                preparation_oid.clone(),
                to_canonical_json(&preparation).unwrap().into_bytes(),
            );
            let history_commit = oid(150 + index as u8);
            source.history.insert(
                entry.preparation_receipt_path.clone(),
                vec![history_commit.clone()],
            );
            source.trees.insert(
                history_commit,
                vec![TreeEntry {
                    mode: "100644".to_owned(),
                    kind: "blob".to_owned(),
                    oid: preparation_oid.clone(),
                    path: entry.preparation_receipt_path.clone(),
                }],
            );
            let closure = receipt_value(entry, true, Some(&preparation_oid));
            add_receipt(
                &mut source,
                PROTECTED,
                &entry.closure_receipt_path,
                closure_oid.clone(),
                &closure,
            );
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.closure_receipt_path,
                closure_oid,
                &closure,
            );
        }

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        assert_eq!(
            facts["scm_facts"]["retirement_control_plane_context"]["lifecycle_state"],
            json!("closed-carried")
        );
        for fact in facts["scm_facts"]["retirement_receipt_object_facts"]
            .as_array()
            .unwrap()
        {
            assert_eq!(fact["receipt_state"], json!("closed-carried"));
            assert_eq!(
                fact["predecessor_context"]["source"],
                json!("linked-preparation-history")
            );
            for input in fact["retired_inputs"].as_array().unwrap() {
                assert_eq!(input["protected_path_exists"], json!(false));
                assert_eq!(input["candidate_path_exists"], json!(false));
            }
        }
    }

    #[test]
    fn closed_carried_rejects_linked_preparation_with_different_baseline() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        let control = control_plane();
        for (index, entry) in control.entries.iter().enumerate() {
            let preparation_oid = oid(153 + index as u8);
            let closure_oid = oid(156 + index as u8);
            let mut preparation = receipt_value(entry, false, None);
            if index == 0 {
                preparation["baseline"] = json!({
                    "commit_oid": PROTECTED,
                    "tree_oid": PROTECTED_TREE,
                });
            }
            source.blobs.insert(
                preparation_oid.clone(),
                to_canonical_json(&preparation).unwrap().into_bytes(),
            );
            let history_commit = oid(159 + index as u8);
            source.history.insert(
                entry.preparation_receipt_path.clone(),
                vec![history_commit.clone()],
            );
            source.trees.insert(
                history_commit,
                vec![TreeEntry {
                    mode: "100644".to_owned(),
                    kind: "blob".to_owned(),
                    oid: preparation_oid.clone(),
                    path: entry.preparation_receipt_path.clone(),
                }],
            );
            let closure = receipt_value(entry, true, Some(&preparation_oid));
            add_receipt(
                &mut source,
                PROTECTED,
                &entry.closure_receipt_path,
                closure_oid.clone(),
                &closure,
            );
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.closure_receipt_path,
                closure_oid,
                &closure,
            );
        }

        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("immutable control-plane predecessor"));
    }

    #[test]
    fn closure_facts_expose_raw_byte_equivalent_candidate_copy() {
        let mut source = fixture();
        add_protected_control_plane(&mut source);
        add_current_bodies(&mut source, PROTECTED);
        let control = control_plane();
        for (index, entry) in control.entries.iter().enumerate() {
            let preparation_oid = oid(160 + index as u8);
            add_receipt(
                &mut source,
                PROTECTED,
                &entry.preparation_receipt_path,
                preparation_oid.clone(),
                &receipt_value(entry, false, None),
            );
            add_receipt(
                &mut source,
                CANDIDATE,
                &entry.closure_receipt_path,
                oid(170 + index as u8),
                &receipt_value(entry, true, Some(&preparation_oid)),
            );
        }
        let roadmap = control.entries[0].selectors[0].expected_inputs[0].clone();
        source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
            mode: "100755".to_owned(),
            kind: "blob".to_owned(),
            oid: roadmap.predecessor_blob_oid,
            path: "copied/roadmap-body".to_owned(),
        });

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        let roadmap_fact = facts["scm_facts"]["retirement_receipt_object_facts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|fact| fact["scope_ref"] == json!("artifact:masterplan"))
            .and_then(|fact| fact["retired_inputs"].as_array())
            .and_then(|inputs| inputs.first())
            .unwrap();
        assert_eq!(
            roadmap_fact["candidate_equivalent_paths"],
            json!(["copied/roadmap-body"])
        );
        assert_eq!(
            roadmap_fact["candidate_new_equivalent_paths"],
            json!(["copied/roadmap-body"])
        );
    }

    #[test]
    fn unexpected_receipt_root_path_is_explicit_fact_not_silently_ignored() {
        let mut source = fixture();
        let unexpected_oid = oid(180);
        source
            .blobs
            .insert(unexpected_oid.clone(), b"unexpected".to_vec());
        source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: unexpected_oid,
            path: "evidence/history-only-retirement/unexpected.json".to_owned(),
        });

        let facts = materialize_history_only_retirement_facts(&source, &context()).unwrap();
        assert_eq!(
            facts["scm_facts"]["retirement_control_plane_context"]["unexpected_candidate_receipt_paths"],
            json!(["evidence/history-only-retirement/unexpected.json"])
        );
    }

    #[test]
    fn independently_read_protected_control_plane_divergence_fails_closed() {
        let mut source = fixture();
        let protected_oid = oid(181);
        source.blobs.insert(protected_oid.clone(), b"{}".to_vec());
        source.trees.get_mut(PROTECTED).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: protected_oid,
            path: CONTROL_PLANE_PATH.to_owned(),
        });

        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("not immutable-identical"));
    }

    #[test]
    fn rejects_non_first_parent_protected_base() {
        let mut source = fixture();
        source.first_parent = PREDECESSOR.to_owned();
        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("not candidate first parent"));
    }

    #[test]
    fn rejects_mutated_predecessor_raw_bytes() {
        let mut source = fixture();
        source.blobs.insert(oid(10), b"changed".to_vec());
        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("raw-byte binding mismatch"));
    }

    #[test]
    fn rejects_candidate_declared_non_regular_predecessor_mode() {
        let mut source = fixture();
        let control_oid = source
            .trees
            .get(CANDIDATE)
            .unwrap()
            .iter()
            .find(|entry| entry.path == CONTROL_PLANE_PATH)
            .unwrap()
            .oid
            .clone();
        let mut control = control_plane();
        control.entries[0].selectors[0].expected_inputs[0].mode = "100755".to_owned();
        source.blobs.insert(
            control_oid,
            to_canonical_json(&serde_json::to_value(control).unwrap())
                .unwrap()
                .into_bytes(),
        );
        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(error.contains("must declare mode 100644"));
    }

    #[test]
    fn selector_rejects_unexpected_matching_candidate_path() {
        let mut source = fixture();
        source.trees.get_mut(CANDIDATE).unwrap().push(TreeEntry {
            mode: "100644".to_owned(),
            kind: "blob".to_owned(),
            oid: oid(201),
            path: "docs/ideas/archive/unlisted.md".to_owned(),
        });
        source.blobs.insert(oid(201), b"unlisted".to_vec());

        let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
        assert!(
            error.contains("selector coverage"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn selector_rejects_unexpected_matching_predecessor_or_protected_path() {
        for tree in [PREDECESSOR, PROTECTED] {
            let mut source = fixture();
            source.trees.get_mut(tree).unwrap().push(TreeEntry {
                mode: "100644".to_owned(),
                kind: "blob".to_owned(),
                oid: oid(if tree == PREDECESSOR { 202 } else { 203 }),
                path: "docs/ideas/archive/unlisted.md".to_owned(),
            });
            let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
            assert!(
                error.contains("selector coverage"),
                "unexpected error: {error}"
            );
        }
    }

    #[test]
    fn duplicate_json_keys_fail_closed() {
        let error = parse_closed_json::<Value>(br#"{"a":1,"a":2}"#).unwrap_err();
        assert!(error.contains("duplicate object key"));
    }

    #[test]
    fn executable_symlink_and_submodule_targets_fail_closed() {
        for (mode, kind) in [("100755", "blob"), ("120000", "blob"), ("160000", "commit")] {
            let mut source = fixture();
            source.trees.get_mut(PREDECESSOR).unwrap()[0].mode = mode.to_owned();
            source.trees.get_mut(PREDECESSOR).unwrap()[0].kind = kind.to_owned();
            let error = materialize_history_only_retirement_facts(&source, &context()).unwrap_err();
            assert!(error.contains("100644 blob"));
        }
    }

    #[test]
    fn ls_tree_parser_preserves_mode_kind_oid_and_path() {
        let bytes = format!("100755 blob {}\tbin/tool\0", oid(7));
        assert_eq!(
            parse_ls_tree(bytes.as_bytes()).unwrap(),
            vec![TreeEntry {
                mode: "100755".to_owned(),
                kind: "blob".to_owned(),
                oid: oid(7),
                path: "bin/tool".to_owned()
            }]
        );
    }
}
