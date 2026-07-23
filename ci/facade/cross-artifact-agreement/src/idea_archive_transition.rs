//! Policy-driven idea-archive transition evaluator.
//!
//! This module evaluates the configured lifecycle policy but does not select or activate a
//! lifecycle state itself. No archive body is retired and no history-only completion is claimed
//! by this code.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Stable validator identity consumed by the cross-artifact gate.
pub const IDEA_ARCHIVE_TRANSITION_VALIDATOR: &str =
    "cloud-ci-cross-artifact-agreement/idea-archive-transition";

const BASELINE_JSON: &str = include_str!("idea-archive-transition-baseline.json");
const BASELINE_SCHEMA_VERSION: u64 = 1;
const BASELINE_ID: &str = "IDEA-ARCHIVE-TRANSITION-2026-07-22-V1";
const BASELINE_COMMIT_OID: &str = "1fa09da22be819b062881eb59252f4dd4c6b550a";
const BASELINE_TREE_OID: &str = "d7b15539396db21b219d68779362850cce9afa8f";
const BASELINE_OBJECT_FORMAT: &str = "sha1";
const BASELINE_SCOPE_ROOT: &str = "docs/ideas/archive";
const BASELINE_MANIFEST_PATH: &str =
    "ci/facade/cross-artifact-agreement/src/idea-archive-transition-baseline.json";
const BASELINE_MANIFEST_SHA256: &str =
    "df46f4ae9eea0c6a59831eb5a47126b6f24475ad499e48a993bda53561ff3d4c";
const PREPARATION_STATE: &str = "open";
const PREPARATION_EXCEPTION_SEMANTICS: &str = "exact-path-and-byte-identity-only";
const PREPARATION_EXCEPTION_EXPANSION: &str = "forbidden";
const PREPARATION_AUTHORITY_STATE: &str = "non-authoritative-transition-inputs";
const REQUIRED_SUCCESSOR_EPOCHS: [&str; 4] = ["E6", "E7", "E9", "E10"];
const HISTORY_ONLY_STATE: &str = "closed";
const HISTORY_ONLY_CLOSURE_EVIDENCE_SET_ID: &str =
    "adr-0388-transient-ideas-history-only-retirement-v1";
// These roots are repository/VCS or derived dependency/build state, not
// candidate-tree content. Every other readable non-symlink path remains in the
// duplicate-body scan so ordinary copies cannot evade strict absence checks.
const CANDIDATE_TREE_EXCLUDED_TOP_LEVEL_DIRS: [&str; 4] =
    [".git", "buck-out", "target", "node_modules"];
const BASELINE_ROWS: [(&str, &str, &str, u64); 3] = [
    (
        "cloud-intelligence-bedrock-on-talos-2026-05-28.md",
        "ffc3aafd802f57d7d6f69a248d90360deecbf9cd",
        "2fad4ac166f3a410a0c7aeaef8632c0fe580f034da48f6be4e2bca642e304eca",
        7_180,
    ),
    (
        "cloud-intelligence-v1-pipeline-2026-05-28.md",
        "4d05288a0b3c8585a478f843b824288ff35faf02",
        "740ae04afc93c41240128c22e3bbf2e1ea84dc63f168ebd974a4f0972a72b2a8",
        16_014,
    ),
    (
        "n-lane-parallel-safety-and-unified-devops-console-2026-05-28.md",
        "820fb46ef556bedaaef22f10e5669791b3143b0d",
        "0a1d134ceb7267e1f8e3e7cc6d16a273da5f36266b53febbc4935b937719589f",
        8_553,
    ),
];

/// Closed lifecycle modes for the idea archive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdeaArchiveMode {
    /// Preserve the pre-E4 archive behavior without imposing a frozen set.
    CurrentTreeArchiveCompatible,
    /// Freeze exactly the evaluator-owned legacy corpus while forbidding growth.
    HistoryOnlyPreparation,
    /// Require the archive directory and all legacy bodies to be absent.
    GitHistoryOnly,
}

/// Validated policy consumed by the evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaArchivePolicy {
    /// Policy schema version. data_class: INTERNAL_ONLY
    pub policy_version: u64,
    /// Closed transition mode. data_class: INTERNAL_ONLY
    pub mode: IdeaArchiveMode,
    transition: Option<IdeaArchiveTransitionBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum IdeaArchiveTransitionBinding {
    Preparation,
    HistoryOnly,
}

/// Fail-closed policy grammar errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdeaArchivePolicyError {
    /// The required nested mode is absent.
    MissingMode,
    /// The mode string is outside the closed grammar.
    UnknownMode(String),
    /// Policy version, fields, or transition shape is invalid.
    InvalidTransition(String),
    /// A preparation binding attempts to redefine the immutable baseline.
    BaselineMismatch(String),
}

impl std::fmt::Display for IdeaArchivePolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingMode => formatter.write_str("missing retention_rules.idea_archive.mode"),
            Self::UnknownMode(mode) => write!(formatter, "unknown idea archive mode {mode:?}"),
            Self::InvalidTransition(message) => {
                write!(formatter, "invalid idea archive transition: {message}")
            }
            Self::BaselineMismatch(message) => {
                write!(formatter, "idea archive baseline mismatch: {message}")
            }
        }
    }
}

impl std::error::Error for IdeaArchivePolicyError {}

/// One immutable legacy archive row.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IdeaArchiveBaselineEntry {
    /// Filename relative to the baseline scope root. data_class: INTERNAL_ONLY
    pub path: String,
    /// Historical Git blob object id. data_class: INTERNAL_ONLY
    pub blob_oid: String,
    /// SHA-256 of the exact body bytes. data_class: INTERNAL_ONLY
    pub sha256: String,
    /// Exact body byte length. data_class: INTERNAL_ONLY
    pub byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IdeaArchiveBaselineCapture {
    /// Exact origin/dev commit from which the baseline was captured. data_class: INTERNAL_ONLY
    pub commit_oid: String,
    /// Exact tree object for the capture commit. data_class: INTERNAL_ONLY
    pub tree_oid: String,
    /// Git object hash format used by the capture repository. data_class: INTERNAL_ONLY
    pub object_format: String,
}

/// Evaluator-owned immutable archive transition baseline.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IdeaArchiveBaseline {
    /// Baseline schema version. data_class: INTERNAL_ONLY
    pub schema_version: u64,
    /// Stable baseline identity. data_class: INTERNAL_ONLY
    pub baseline_id: String,
    /// Immutable Git capture identity. data_class: INTERNAL_ONLY
    pub captured_from: IdeaArchiveBaselineCapture,
    /// Repo-relative directory containing the legacy rows. data_class: INTERNAL_ONLY
    pub scope_root: String,
    /// Digest of the canonical baseline manifest payload. data_class: INTERNAL_ONLY
    pub manifest_sha256: String,
    /// Exact ordered legacy rows. data_class: INTERNAL_ONLY
    pub entries: Vec<IdeaArchiveBaselineEntry>,
}

/// Candidate-tree path kind observed without following symlinks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdeaArchivePathKind {
    /// The path is absent.
    Missing,
    /// A regular file.
    RegularFile,
    /// A directory.
    Directory,
    /// A symbolic link.
    Symlink,
    /// Another filesystem node kind.
    Other,
}

/// One observed node under `docs/ideas/archive`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaArchiveObservedNode {
    /// Node kind from symlink-aware metadata. data_class: INTERNAL_ONLY
    pub kind: IdeaArchivePathKind,
    /// SHA-256 for regular-file bytes. data_class: INTERNAL_ONLY
    pub sha256: Option<String>,
    /// Length for regular-file bytes. data_class: INTERNAL_ONLY
    pub byte_length: Option<u64>,
}

/// Protected closure status projected by the E7 evidence-set validator.
///
/// This type does not mint authority. Its caller must inject only the output of
/// the separately protected validator; the filesystem collector never infers a
/// verified closure from candidate files or ambient repository state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IdeaArchiveVerifiedClosureProjection {
    /// Evidence-set ids verified closed by the protected validator.
    /// data_class: INTERNAL_ONLY
    pub evidence_set_ids: BTreeSet<String>,
}

/// Complete candidate observation required by the pure evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaArchiveObservation {
    /// Kind of the archive root itself. data_class: INTERNAL_ONLY
    pub archive_root_kind: IdeaArchivePathKind,
    /// Every descendant node, keyed by repo-relative path. data_class: INTERNAL_ONLY
    pub nodes: BTreeMap<String, IdeaArchiveObservedNode>,
    /// Repo-wide exact-body locations keyed by baseline SHA-256. data_class: INTERNAL_ONLY
    pub exact_body_locations: BTreeMap<String, BTreeSet<String>>,
    /// Explicit protected-validator injection seam. data_class: INTERNAL_ONLY
    pub verified_closure_projection: IdeaArchiveVerifiedClosureProjection,
}

/// Successful evaluation report. It is evidence of conformance to the selected
/// policy mode only, never evidence that history-only retirement completed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaArchiveTransitionReport {
    /// Mode evaluated. data_class: INTERNAL_ONLY
    pub mode: IdeaArchiveMode,
    /// Exact preparation-only rows skipped as nonauthority. Empty in other modes.
    /// data_class: INTERNAL_ONLY
    pub preparation_nonauthority_paths: BTreeSet<String>,
}

/// Fail-closed baseline/candidate mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdeaArchiveTransitionError {
    /// The immutable baseline or candidate observation drifted.
    BaselineMismatch(String),
}

impl std::fmt::Display for IdeaArchiveTransitionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BaselineMismatch(message) => {
                write!(formatter, "idea archive baseline mismatch: {message}")
            }
        }
    }
}

impl std::error::Error for IdeaArchiveTransitionError {}

/// Read-only candidate-tree collection error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdeaArchiveCollectError(pub String);

impl std::fmt::Display for IdeaArchiveCollectError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for IdeaArchiveCollectError {}

/// Parse the nested three-mode policy grammar.
pub fn parse_idea_archive_policy(
    policy: &Value,
) -> Result<IdeaArchivePolicy, IdeaArchivePolicyError> {
    let Some(idea_archive) = policy
        .get("retention_rules")
        .and_then(Value::as_object)
        .and_then(|rules| rules.get("idea_archive"))
        .and_then(Value::as_object)
    else {
        return Err(IdeaArchivePolicyError::MissingMode);
    };

    let Some(mode_value) = idea_archive.get("mode") else {
        return Err(IdeaArchivePolicyError::MissingMode);
    };
    let mode_text = mode_value.as_str().ok_or_else(|| {
        IdeaArchivePolicyError::InvalidTransition("mode must be a string".to_owned())
    })?;
    let mode = match mode_text {
        "current-tree-archive-compatible" => IdeaArchiveMode::CurrentTreeArchiveCompatible,
        "history-only-preparation" => IdeaArchiveMode::HistoryOnlyPreparation,
        "git-history-only" => IdeaArchiveMode::GitHistoryOnly,
        other => return Err(IdeaArchivePolicyError::UnknownMode(other.to_owned())),
    };

    let allowed_keys = BTreeSet::from(["policy_version", "mode", "transition"]);
    let actual_keys = idea_archive
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_keys != allowed_keys {
        return Err(IdeaArchivePolicyError::InvalidTransition(
            "idea_archive must contain exactly policy_version, mode, and transition".to_owned(),
        ));
    }
    let policy_version = idea_archive
        .get("policy_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            IdeaArchivePolicyError::InvalidTransition("policy_version must be integer 1".to_owned())
        })?;
    if policy_version != 1 {
        return Err(IdeaArchivePolicyError::InvalidTransition(format!(
            "unsupported policy_version {policy_version}"
        )));
    }
    let transition = idea_archive.get("transition").ok_or_else(|| {
        IdeaArchivePolicyError::InvalidTransition("transition is required".to_owned())
    })?;

    let binding = match mode {
        IdeaArchiveMode::CurrentTreeArchiveCompatible => {
            if !transition.is_null() {
                return Err(IdeaArchivePolicyError::InvalidTransition(format!(
                    "{mode_text} requires transition: null"
                )));
            }
            None
        }
        IdeaArchiveMode::HistoryOnlyPreparation => {
            let object = transition.as_object().ok_or_else(|| {
                IdeaArchivePolicyError::InvalidTransition(
                    "history-only-preparation requires a baseline binding object".to_owned(),
                )
            })?;
            let expected_keys = BTreeSet::from([
                "state",
                "baseline_id",
                "manifest_path",
                "sha256",
                "exception_semantics",
                "exception_expansion",
                "authority_state",
                "completion_claim",
                "required_successor_epochs",
            ]);
            let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
            if keys != expected_keys {
                return Err(IdeaArchivePolicyError::InvalidTransition(
                    "preparation transition fields differ from the closed E5 contract".to_owned(),
                ));
            }
            let baseline = immutable_idea_archive_baseline()
                .map_err(|error| IdeaArchivePolicyError::BaselineMismatch(error.to_string()))?;
            validate_baseline_binding(object, &baseline)?;
            require_exact_string(object, "state", PREPARATION_STATE)?;
            require_exact_string(
                object,
                "exception_semantics",
                PREPARATION_EXCEPTION_SEMANTICS,
            )?;
            require_exact_string(
                object,
                "exception_expansion",
                PREPARATION_EXCEPTION_EXPANSION,
            )?;
            require_exact_string(object, "authority_state", PREPARATION_AUTHORITY_STATE)?;
            if object.get("completion_claim") != Some(&Value::Bool(false)) {
                return Err(IdeaArchivePolicyError::InvalidTransition(
                    "preparation completion_claim must be false".to_owned(),
                ));
            }
            let epochs = object
                .get("required_successor_epochs")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    IdeaArchivePolicyError::InvalidTransition(
                        "required_successor_epochs must be an array".to_owned(),
                    )
                })?;
            if epochs.len() != REQUIRED_SUCCESSOR_EPOCHS.len()
                || !epochs
                    .iter()
                    .zip(REQUIRED_SUCCESSOR_EPOCHS)
                    .all(|(actual, expected)| actual.as_str() == Some(expected))
            {
                return Err(IdeaArchivePolicyError::InvalidTransition(
                    "required_successor_epochs must be exactly E6, E7, E9, E10 in order".to_owned(),
                ));
            }
            Some(IdeaArchiveTransitionBinding::Preparation)
        }
        IdeaArchiveMode::GitHistoryOnly => {
            let object = transition.as_object().ok_or_else(|| {
                IdeaArchivePolicyError::InvalidTransition(
                    "git-history-only requires a protected closure-set binding".to_owned(),
                )
            })?;
            let expected_keys = BTreeSet::from([
                "state",
                "baseline_id",
                "manifest_path",
                "sha256",
                "closure_evidence_set_id",
            ]);
            let keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
            if keys != expected_keys {
                return Err(IdeaArchivePolicyError::InvalidTransition(
                    "git-history-only transition fields differ from the closed E10 contract"
                        .to_owned(),
                ));
            }
            let baseline = immutable_idea_archive_baseline()
                .map_err(|error| IdeaArchivePolicyError::BaselineMismatch(error.to_string()))?;
            validate_baseline_binding(object, &baseline)?;
            require_exact_string(object, "state", HISTORY_ONLY_STATE)?;
            if object
                .get("closure_evidence_set_id")
                .and_then(Value::as_str)
                != Some(HISTORY_ONLY_CLOSURE_EVIDENCE_SET_ID)
            {
                return Err(IdeaArchivePolicyError::BaselineMismatch(
                    "git-history-only closure evidence set is not the predeclared protected set"
                        .to_owned(),
                ));
            }
            Some(IdeaArchiveTransitionBinding::HistoryOnly)
        }
    };

    Ok(IdeaArchivePolicy {
        policy_version,
        mode,
        transition: binding,
    })
}

fn validate_baseline_binding(
    object: &serde_json::Map<String, Value>,
    baseline: &IdeaArchiveBaseline,
) -> Result<(), IdeaArchivePolicyError> {
    if object.get("baseline_id").and_then(Value::as_str) != Some(baseline.baseline_id.as_str())
        || object.get("manifest_path").and_then(Value::as_str) != Some(BASELINE_MANIFEST_PATH)
        || object.get("sha256").and_then(Value::as_str) != Some(baseline.manifest_sha256.as_str())
    {
        return Err(IdeaArchivePolicyError::BaselineMismatch(
            "policy transition differs from the evaluator-owned baseline binding".to_owned(),
        ));
    }
    Ok(())
}

fn require_exact_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
) -> Result<(), IdeaArchivePolicyError> {
    if object.get(field).and_then(Value::as_str) != Some(expected) {
        return Err(IdeaArchivePolicyError::InvalidTransition(format!(
            "{field} must be {expected:?}"
        )));
    }
    Ok(())
}

/// Parse and validate the evaluator-owned immutable baseline.
pub fn immutable_idea_archive_baseline() -> Result<IdeaArchiveBaseline, IdeaArchiveTransitionError>
{
    let baseline: IdeaArchiveBaseline = serde_json::from_str(BASELINE_JSON).map_err(|error| {
        IdeaArchiveTransitionError::BaselineMismatch(format!(
            "immutable baseline JSON does not parse: {error}"
        ))
    })?;
    validate_immutable_baseline(&baseline)?;
    Ok(baseline)
}

fn validate_immutable_baseline(
    baseline: &IdeaArchiveBaseline,
) -> Result<(), IdeaArchiveTransitionError> {
    if baseline.schema_version != BASELINE_SCHEMA_VERSION
        || baseline.baseline_id != BASELINE_ID
        || baseline.captured_from.commit_oid != BASELINE_COMMIT_OID
        || baseline.captured_from.tree_oid != BASELINE_TREE_OID
        || baseline.captured_from.object_format != BASELINE_OBJECT_FORMAT
        || baseline.scope_root != BASELINE_SCOPE_ROOT
        || baseline.manifest_sha256 != BASELINE_MANIFEST_SHA256
    {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "immutable baseline identity metadata drifted".to_owned(),
        ));
    }
    if baseline.entries.len() != BASELINE_ROWS.len() {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "immutable baseline row count drifted".to_owned(),
        ));
    }
    for (entry, expected) in baseline.entries.iter().zip(BASELINE_ROWS) {
        if (
            entry.path.as_str(),
            entry.blob_oid.as_str(),
            entry.sha256.as_str(),
            entry.byte_length,
        ) != expected
        {
            return Err(IdeaArchiveTransitionError::BaselineMismatch(format!(
                "immutable baseline row drifted at {}",
                entry.path
            )));
        }
    }
    let digest = hex_sha256(canonical_manifest_payload(baseline).as_bytes());
    if digest != baseline.manifest_sha256 {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "immutable baseline manifest digest is stale".to_owned(),
        ));
    }
    Ok(())
}

fn canonical_manifest_payload(baseline: &IdeaArchiveBaseline) -> String {
    use std::fmt::Write as _;

    let mut payload = String::new();
    let _ = writeln!(payload, "schema_version={}", baseline.schema_version);
    let _ = writeln!(payload, "baseline_id={}", baseline.baseline_id);
    let _ = writeln!(
        payload,
        "captured_from.commit_oid={}",
        baseline.captured_from.commit_oid
    );
    let _ = writeln!(
        payload,
        "captured_from.tree_oid={}",
        baseline.captured_from.tree_oid
    );
    let _ = writeln!(
        payload,
        "captured_from.object_format={}",
        baseline.captured_from.object_format
    );
    let _ = writeln!(payload, "scope_root={}", baseline.scope_root);
    for entry in &baseline.entries {
        let _ = writeln!(
            payload,
            "entry={}\t{}\t{}\t{}",
            entry.path, entry.blob_oid, entry.sha256, entry.byte_length
        );
    }
    payload
}

/// Purely evaluate one parsed policy against a complete observation.
pub fn evaluate_idea_archive_transition(
    policy: &IdeaArchivePolicy,
    observation: &IdeaArchiveObservation,
) -> Result<IdeaArchiveTransitionReport, IdeaArchiveTransitionError> {
    let baseline = immutable_idea_archive_baseline()?;
    match policy.mode {
        IdeaArchiveMode::CurrentTreeArchiveCompatible => {
            // Compatibility preserves the exact readable transition inventory; it never
            // authorizes the archive to grow or treats archive placement as compliance.
            let report = evaluate_preparation(&baseline, observation)?;
            Ok(IdeaArchiveTransitionReport {
                mode: policy.mode,
                preparation_nonauthority_paths: report.preparation_nonauthority_paths,
            })
        }
        IdeaArchiveMode::HistoryOnlyPreparation => {
            if policy.transition != Some(IdeaArchiveTransitionBinding::Preparation) {
                return Err(IdeaArchiveTransitionError::BaselineMismatch(
                    "preparation mode lost its validated baseline binding".to_owned(),
                ));
            }
            evaluate_preparation(&baseline, observation)
        }
        IdeaArchiveMode::GitHistoryOnly => {
            if policy.transition != Some(IdeaArchiveTransitionBinding::HistoryOnly) {
                return Err(IdeaArchiveTransitionError::BaselineMismatch(
                    "git-history-only mode lost its protected closure-set binding".to_owned(),
                ));
            }
            evaluate_history_only(&baseline, observation)
        }
    }
}

fn evaluate_preparation(
    baseline: &IdeaArchiveBaseline,
    observation: &IdeaArchiveObservation,
) -> Result<IdeaArchiveTransitionReport, IdeaArchiveTransitionError> {
    if observation.archive_root_kind != IdeaArchivePathKind::Directory {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "preparation requires the exact archive directory".to_owned(),
        ));
    }

    let expected_paths = baseline
        .entries
        .iter()
        .map(|entry| format!("{}/{}", baseline.scope_root, entry.path))
        .collect::<BTreeSet<_>>();
    let observed_paths = observation.nodes.keys().cloned().collect::<BTreeSet<_>>();
    if observed_paths != expected_paths {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "preparation archive path set differs from the immutable three-row baseline".to_owned(),
        ));
    }

    for entry in &baseline.entries {
        let path = format!("{}/{}", baseline.scope_root, entry.path);
        let Some(node) = observation.nodes.get(&path) else {
            return Err(IdeaArchiveTransitionError::BaselineMismatch(format!(
                "missing baseline path {path}"
            )));
        };
        if node.kind != IdeaArchivePathKind::RegularFile
            || node.sha256.as_deref() != Some(entry.sha256.as_str())
            || node.byte_length != Some(entry.byte_length)
        {
            return Err(IdeaArchiveTransitionError::BaselineMismatch(format!(
                "baseline bytes, digest, length, or path kind drifted at {path}"
            )));
        }
        let expected_locations = BTreeSet::from([path.clone()]);
        if observation.exact_body_locations.get(&entry.sha256) != Some(&expected_locations) {
            return Err(IdeaArchiveTransitionError::BaselineMismatch(format!(
                "baseline body {} is absent, renamed, or duplicated outside {path}",
                entry.sha256
            )));
        }
    }

    let baseline_digests = baseline
        .entries
        .iter()
        .map(|entry| entry.sha256.as_str())
        .collect::<BTreeSet<_>>();
    if observation
        .exact_body_locations
        .keys()
        .any(|digest| !baseline_digests.contains(digest.as_str()))
    {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "observation contains a candidate-defined baseline digest".to_owned(),
        ));
    }

    Ok(IdeaArchiveTransitionReport {
        mode: IdeaArchiveMode::HistoryOnlyPreparation,
        preparation_nonauthority_paths: expected_paths,
    })
}

fn evaluate_history_only(
    baseline: &IdeaArchiveBaseline,
    observation: &IdeaArchiveObservation,
) -> Result<IdeaArchiveTransitionReport, IdeaArchiveTransitionError> {
    if observation.archive_root_kind != IdeaArchivePathKind::Missing
        || !observation.nodes.is_empty()
    {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "git-history-only requires docs/ideas/archive to be absent".to_owned(),
        ));
    }
    for entry in &baseline.entries {
        if observation
            .exact_body_locations
            .get(&entry.sha256)
            .is_some_and(|locations| !locations.is_empty())
        {
            return Err(IdeaArchiveTransitionError::BaselineMismatch(format!(
                "git-history-only body {} remains in the candidate tree",
                entry.sha256
            )));
        }
    }
    let expected_evidence_sets = BTreeSet::from([HISTORY_ONLY_CLOSURE_EVIDENCE_SET_ID.to_owned()]);
    if observation.verified_closure_projection.evidence_set_ids != expected_evidence_sets {
        return Err(IdeaArchiveTransitionError::BaselineMismatch(
            "git-history-only requires the exact protected E7 closure evidence-set projection"
                .to_owned(),
        ));
    }
    Ok(IdeaArchiveTransitionReport {
        mode: IdeaArchiveMode::GitHistoryOnly,
        preparation_nonauthority_paths: BTreeSet::new(),
    })
}

/// Collect the filesystem observation used by the pure evaluator.
///
/// The collector never follows symlinks and hashes only regular files whose
/// byte lengths match an immutable baseline row when checking for repo-wide
/// exact-body duplicates. It excludes only top-level VCS/derived-state roots
/// named by `CANDIDATE_TREE_EXCLUDED_TOP_LEVEL_DIRS`; it scans all other
/// readable repository paths.
pub fn collect_idea_archive_observation(
    repo_root: &Path,
    verified_closure_projection: IdeaArchiveVerifiedClosureProjection,
) -> Result<IdeaArchiveObservation, IdeaArchiveCollectError> {
    let baseline = immutable_idea_archive_baseline()
        .map_err(|error| IdeaArchiveCollectError(error.to_string()))?;
    let archive_root = repo_root.join(&baseline.scope_root);
    let archive_root_kind = path_kind(&archive_root)?;
    let mut nodes = BTreeMap::new();
    if archive_root_kind == IdeaArchivePathKind::Directory {
        collect_archive_nodes(repo_root, &archive_root, &mut nodes)?;
    }

    let by_length = baseline.entries.iter().fold(
        BTreeMap::<u64, BTreeSet<String>>::new(),
        |mut acc, entry| {
            acc.entry(entry.byte_length)
                .or_default()
                .insert(entry.sha256.clone());
            acc
        },
    );
    let mut exact_body_locations = BTreeMap::<String, BTreeSet<String>>::new();
    collect_exact_body_locations(repo_root, repo_root, &by_length, &mut exact_body_locations)?;

    Ok(IdeaArchiveObservation {
        archive_root_kind,
        nodes,
        exact_body_locations,
        verified_closure_projection,
    })
}

fn collect_archive_nodes(
    repo_root: &Path,
    directory: &Path,
    nodes: &mut BTreeMap<String, IdeaArchiveObservedNode>,
) -> Result<(), IdeaArchiveCollectError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| io_error("read archive directory", directory, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error("read archive entry", directory, error))?;
    entries.sort_by_key(fs::DirEntry::path);
    for entry in entries {
        let path = entry.path();
        let kind = path_kind(&path)?;
        let relative = repo_relative(repo_root, &path)?;
        let (sha256, byte_length) = if kind == IdeaArchivePathKind::RegularFile {
            let bytes =
                fs::read(&path).map_err(|error| io_error("read archive file", &path, error))?;
            let length = u64::try_from(bytes.len()).map_err(|error| {
                IdeaArchiveCollectError(format!(
                    "archive file length does not fit u64 at {}: {error}",
                    path.display()
                ))
            })?;
            (Some(hex_sha256(&bytes)), Some(length))
        } else {
            (None, None)
        };
        nodes.insert(
            relative,
            IdeaArchiveObservedNode {
                kind,
                sha256,
                byte_length,
            },
        );
        if kind == IdeaArchivePathKind::Directory {
            collect_archive_nodes(repo_root, &path, nodes)?;
        }
    }
    Ok(())
}

fn collect_exact_body_locations(
    repo_root: &Path,
    directory: &Path,
    by_length: &BTreeMap<u64, BTreeSet<String>>,
    locations: &mut BTreeMap<String, BTreeSet<String>>,
) -> Result<(), IdeaArchiveCollectError> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| io_error("read candidate directory", directory, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error("read candidate entry", directory, error))?;
    entries.sort_by_key(fs::DirEntry::path);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| io_error("read candidate metadata", &path, error))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if should_skip_directory(repo_root, &path) {
                continue;
            }
            collect_exact_body_locations(repo_root, &path, by_length, locations)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Some(candidate_digests) = by_length.get(&metadata.len()) else {
            continue;
        };
        let bytes =
            fs::read(&path).map_err(|error| io_error("read candidate file", &path, error))?;
        let digest = hex_sha256(&bytes);
        if candidate_digests.contains(&digest) {
            locations
                .entry(digest)
                .or_default()
                .insert(repo_relative(repo_root, &path)?);
        }
    }
    Ok(())
}

fn should_skip_directory(repo_root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(repo_root) else {
        return true;
    };
    let Some(first) = relative.components().next() else {
        return false;
    };
    first
        .as_os_str()
        .to_str()
        .is_some_and(|name| CANDIDATE_TREE_EXCLUDED_TOP_LEVEL_DIRS.contains(&name))
}

fn path_kind(path: &Path) -> Result<IdeaArchivePathKind, IdeaArchiveCollectError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            Ok(if file_type.is_symlink() {
                IdeaArchivePathKind::Symlink
            } else if file_type.is_dir() {
                IdeaArchivePathKind::Directory
            } else if file_type.is_file() {
                IdeaArchivePathKind::RegularFile
            } else {
                IdeaArchivePathKind::Other
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(IdeaArchivePathKind::Missing)
        }
        Err(error) => Err(io_error("read path metadata", path, error)),
    }
}

fn repo_relative(repo_root: &Path, path: &Path) -> Result<String, IdeaArchiveCollectError> {
    let relative = path.strip_prefix(repo_root).map_err(|error| {
        IdeaArchiveCollectError(format!(
            "path {} escaped repo root {}: {error}",
            path.display(),
            repo_root.display()
        ))
    })?;
    let text = relative.to_str().ok_or_else(|| {
        IdeaArchiveCollectError(format!(
            "candidate path is not valid UTF-8 under {}",
            repo_root.display()
        ))
    })?;
    Ok(text.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn io_error(action: &str, path: &Path, error: std::io::Error) -> IdeaArchiveCollectError {
    IdeaArchiveCollectError(format!("{action} {}: {error}", path.display()))
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create test parent");
        }
        fs::write(path, bytes).expect("write test file");
    }

    #[test]
    fn immutable_baseline_manifest_and_literals_are_self_consistent() {
        let baseline = immutable_idea_archive_baseline().expect("baseline validates");
        assert_eq!(baseline.entries.len(), 3);
        assert_eq!(baseline.manifest_sha256, BASELINE_MANIFEST_SHA256);
    }

    #[test]
    fn skip_policy_is_limited_to_untracked_build_and_vcs_directories() {
        let root = Path::new("/repo");
        assert!(should_skip_directory(root, Path::new("/repo/.git/objects")));
        assert!(should_skip_directory(root, Path::new("/repo/buck-out/v2")));
        assert!(should_skip_directory(root, Path::new("/repo/target/debug")));
        assert!(should_skip_directory(
            root,
            Path::new("/repo/node_modules/package")
        ));
        assert!(!should_skip_directory(
            root,
            Path::new("/repo/.omc/ultragoal")
        ));
        assert!(!should_skip_directory(
            root,
            Path::new("/repo/.omx/context")
        ));
        assert!(!should_skip_directory(root, Path::new("/repo/docs/ideas")));
        assert!(!should_skip_directory(
            root,
            Path::new("/repo/vendor/source")
        ));
    }

    #[test]
    fn collector_records_nested_and_non_regular_archive_nodes() {
        let temp = tempfile::tempdir().expect("temp repo");
        let archive = temp.path().join(BASELINE_SCOPE_ROOT);
        write_file(&archive.join("nested/body.md"), b"candidate body");

        #[cfg(unix)]
        std::os::unix::fs::symlink("nested/body.md", archive.join("body-link.md"))
            .expect("create archive symlink");

        let observation = collect_idea_archive_observation(
            temp.path(),
            IdeaArchiveVerifiedClosureProjection::default(),
        )
        .expect("collect test repository");
        assert_eq!(
            observation.archive_root_kind,
            IdeaArchivePathKind::Directory
        );
        assert_eq!(
            observation.nodes["docs/ideas/archive/nested"].kind,
            IdeaArchivePathKind::Directory
        );
        assert_eq!(
            observation.nodes["docs/ideas/archive/nested/body.md"].kind,
            IdeaArchivePathKind::RegularFile
        );
        #[cfg(unix)]
        assert_eq!(
            observation.nodes["docs/ideas/archive/body-link.md"].kind,
            IdeaArchivePathKind::Symlink
        );
    }

    #[cfg(unix)]
    #[test]
    fn collector_does_not_follow_an_archive_root_symlink() {
        let temp = tempfile::tempdir().expect("temp repo");
        let target = temp.path().join("outside-archive");
        write_file(&target.join("body.md"), b"candidate body");
        let archive = temp.path().join(BASELINE_SCOPE_ROOT);
        fs::create_dir_all(archive.parent().expect("archive parent"))
            .expect("create archive parent");
        std::os::unix::fs::symlink(&target, &archive).expect("create archive root symlink");

        let observation = collect_idea_archive_observation(
            temp.path(),
            IdeaArchiveVerifiedClosureProjection::default(),
        )
        .expect("collect test repository");
        assert_eq!(observation.archive_root_kind, IdeaArchivePathKind::Symlink);
        assert!(observation.nodes.is_empty());
    }

    #[test]
    fn exact_body_scan_reports_all_current_tree_locations_and_skips_build_output() {
        let temp = tempfile::tempdir().expect("temp repo");
        let bytes = b"same candidate body";
        let digest = hex_sha256(bytes);
        write_file(&temp.path().join("docs/ideas/archive/body.md"), bytes);
        write_file(&temp.path().join(".omc/ultragoal/copy.md"), bytes);
        write_file(&temp.path().join("vendor/source/ordinary-copy.md"), bytes);
        write_file(&temp.path().join(".git/objects/ignored-copy"), bytes);
        write_file(&temp.path().join("buck-out/v2/ignored-copy.md"), bytes);
        write_file(&temp.path().join("target/debug/ignored-copy.md"), bytes);
        write_file(
            &temp.path().join("node_modules/package/ignored-copy.md"),
            bytes,
        );
        let by_length = BTreeMap::from([(
            u64::try_from(bytes.len()).expect("test length fits"),
            BTreeSet::from([digest.clone()]),
        )]);
        let mut locations = BTreeMap::new();

        collect_exact_body_locations(temp.path(), temp.path(), &by_length, &mut locations)
            .expect("scan test repository");

        assert_eq!(
            locations[&digest],
            BTreeSet::from([
                ".omc/ultragoal/copy.md".to_owned(),
                "docs/ideas/archive/body.md".to_owned(),
                "vendor/source/ordinary-copy.md".to_owned(),
            ])
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn collector_rejects_non_utf8_candidate_paths() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt as _;

        let temp = tempfile::tempdir().expect("temp repo");
        let archive = temp.path().join(BASELINE_SCOPE_ROOT);
        fs::create_dir_all(&archive).expect("create archive");
        write_file(
            &archive.join(OsString::from_vec(vec![0xff, b'.', b'm', b'd'])),
            b"body",
        );

        let error = collect_idea_archive_observation(
            temp.path(),
            IdeaArchiveVerifiedClosureProjection::default(),
        )
        .expect_err("non-UTF-8 path must fail closed");
        assert!(error.to_string().contains("not valid UTF-8"));
    }
}
