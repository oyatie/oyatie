//! Policy contract v2: typed model for the generated-artifact control-plane manifest.
//!
//! This is the schema for `registry/generated-artifact-control-plane.json` at
//! `schema_version: 2`. It is an ADDITIVE extension of v1: every v1 field is
//! preserved; `runner_registry` is promoted from a gate const to manifest data;
//! `generator.input_contract` becomes the DAG edge source; `artifact_class` is
//! typed into the `ArtifactClass` enum.
//!
//! Repo-agnostic: ZERO oyatie paths, names, or targets. All adopter-specific data
//! lives in the JSON manifest.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Opaque artifact identifier (e.g. `"cloud-ci-accounting-registry-face"`).
pub type ArtifactId = String;

/// Materialized bytes — UTF-8 string (generated JSON/TS/Go/etc. is text).
pub type Bytes = String;

/// The materialization mode of a declared artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterializationMode {
    /// Committed to git; freshness = byte-parity to regeneration.
    MergeCandidate,
    /// De-committed: derived on demand, never committed. Integrity = determinism canary.
    /// The one-way door: re-tracking is a hard RED (ADR-0595 / ADR-0596).
    NotTrackedInGit,
    /// Committed on main branch only; candidates regenerate on demand.
    MainBranchMaterialized,
    /// Transitional: committed on branches; controller materializes on main.
    BranchCommittedRegeneratedUntilControllerMaterialization,
    /// Any other mode from v1; round-trips without loss.
    #[serde(untagged)]
    Other(String),
}

impl MaterializationMode {
    /// Returns true when this artifact is de-commit-class (must not be tracked in git).
    pub fn is_not_tracked_in_git(&self) -> bool {
        matches!(self, MaterializationMode::NotTrackedInGit)
    }
}

/// The class of a generated artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactClass {
    /// Pure derivation: a view computed entirely from source inputs.
    PureView,
    /// Frozen reference: anchored at merge-base; guarded by ADR-0596 re-tracking door.
    FrozenReference,
    /// Committed artifact that should remain tracked.
    Committed,
    /// Any other class from v1; round-trips without loss.
    #[serde(untagged)]
    Other(String),
}

/// Output mode declared by the generator block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputMode {
    /// Generator writes JSON to stdout; executor captures it.
    StdoutJson,
    /// Generator writes to a declared filesystem path.
    DeclaredArtifactPathWrite,
    /// Generator is controlled by the native controller reconciler (in-process).
    ControllerMaterialized,
    /// Any other output mode; round-trips without loss.
    #[serde(untagged)]
    Other(String),
}

/// A runner declared in the manifest's `runner_registry`.
///
/// There is NO `shell` runner. Arbitrary subprocess execution is forbidden (MF-2 /
/// ADR-0523 / ADR-0596). A runner not in the registry causes `plan()` to return Err.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Runner {
    /// buck2 build + exec. Target must start with `//`.
    #[serde(rename = "buck2")]
    Buck2 { target: String },
    /// In-process native controller reconciler call. Target prefix: `oya-ci://generated-artifact-controller/`.
    #[serde(rename = "oya-ci-native-controller")]
    NativeController { target: String },
    /// Any future registered runner (e.g. `node-codegen`, `npm://...`).
    Registered { runner_id: String, target: String },
}

/// One entry in `runner_registry` — declares a runner and its canonical target prefix.
///
/// Adopters add entries here to introduce new runners (e.g. `node-codegen`).
/// A runner not declared here is rejected by `plan()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerRegistryEntry {
    /// The runner identifier used in artifact `generator.runner` fields.
    pub runner_id: String,
    /// All targets for this runner must start with this prefix.
    pub canonical_target_prefix: String,
    /// How the executor lowers this runner to an action (informational; not parsed by kernel).
    pub lowering: String,
    /// If this runner requires an ADR-0523 irreducible-glue exception, cite it here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub irreducible_glue_adr: Option<String>,
}

/// The `generator` block for a declared artifact — the sole source of truth for
/// how to regenerate this artifact. In v2 this is LOAD-BEARING: `plan()` reads it
/// to derive the materialization plan and topological order.
///
/// `input_contract` strings are the DAG edge source: an artifact whose
/// `input_contract` references a token produced by another artifact (e.g.
/// `"scm-facts-snapshot"` produced by the `cloud-ci-scm-facts-boundary-snapshot`
/// artifact) will be sequenced AFTER that artifact in the plan. No target strings
/// or hand-authored `needs:` edges — ordering is fully data-derived.
///
/// Note: `scm-facts-snapshot` has NO privileged kernel status. It is an artifact id
/// token like any other. A non-oyatie adopter whose codegen has no scm-facts
/// dependency simply omits it from `input_contract`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Generator {
    /// The runner id. Must match an entry in `runner_registry`.
    pub runner: String,
    /// The canonical target the runner will execute.
    pub generator_target: String,
    /// Stable operation verb the runner dispatches on.
    pub operation_id: String,
    /// Key-value parameters forwarded to the generator tool. All values non-empty.
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    /// DAG edge tokens: strings naming inputs this artifact depends on.
    /// If token T equals another artifact's `operation_id` (or a well-known
    /// canonical token like an artifact_id), that artifact must materialize first.
    #[serde(default)]
    pub input_contract: Vec<String>,
    /// How the generator emits its output.
    pub output_mode: OutputMode,
}

/// A declared generated artifact in the control-plane manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    /// Unique identifier within the manifest.
    pub artifact_id: ArtifactId,
    /// Repository-relative path where the artifact lives (or lived, for de-commit class).
    pub path: String,
    /// Classification of this artifact.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_class: Option<ArtifactClass>,
    /// Materialization mode.
    pub materialization_mode: MaterializationMode,
    /// Generator block: promoted to load-bearing in v2.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generator: Option<Generator>,
    /// Preserve all other fields for round-trip fidelity.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl GeneratedArtifact {
    /// True if this artifact is de-commit-class (intentionally not tracked in git).
    pub fn is_not_tracked_in_git(&self) -> bool {
        self.materialization_mode.is_not_tracked_in_git()
    }

    /// Returns the generator block, or an error if absent.
    pub fn require_generator(&self) -> Result<&Generator, crate::plan::PlanError> {
        self.generator
            .as_ref()
            .ok_or_else(|| crate::plan::PlanError::MissingGenerator {
                artifact_id: self.artifact_id.clone(),
            })
    }
}

/// The top-level generated-artifact control-plane manifest (v2).
///
/// Repo-agnostic: zero hardcoded oyatie paths. Adopters supply this manifest; the
/// kernel reads only its typed fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlPlane {
    /// Schema version. v2 adds `runner_registry` + promotes `generator` to load-bearing.
    #[serde(default)]
    pub schema_version: u32,

    /// Declared runner registry. Every runner referenced in `generator.runner` must
    /// appear here. NO `shell` runner is permitted.
    #[serde(default)]
    pub runner_registry: Vec<RunnerRegistryEntry>,

    /// All declared generated artifacts in this repository.
    #[serde(default)]
    pub artifacts: Vec<GeneratedArtifact>,

    /// Preserve all other top-level fields for round-trip fidelity.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl ControlPlane {
    /// Parse a control-plane manifest from JSON text.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Find the runner registry entry for a given runner_id.
    pub fn find_runner(&self, runner_id: &str) -> Option<&RunnerRegistryEntry> {
        self.runner_registry
            .iter()
            .find(|r| r.runner_id == runner_id)
    }

    /// Return all artifact ids in de-commit-class, sorted.
    pub fn decommit_artifact_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self
            .artifacts
            .iter()
            .filter(|a| a.is_not_tracked_in_git())
            .map(|a| a.artifact_id.as_str())
            .collect();
        ids.sort();
        ids
    }

    /// Return all artifact paths in de-commit-class, sorted.
    pub fn decommit_paths(&self) -> Vec<&str> {
        let mut paths: Vec<&str> = self
            .artifacts
            .iter()
            .filter(|a| a.is_not_tracked_in_git())
            .map(|a| a.path.as_str())
            .collect();
        paths.sort();
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let json = r#"{"schema_version":2,"runner_registry":[],"artifacts":[]}"#;
        let cp = ControlPlane::from_json(json).unwrap();
        assert_eq!(cp.schema_version, 2);
        assert!(cp.runner_registry.is_empty());
        assert!(cp.artifacts.is_empty());
    }

    #[test]
    fn materialization_mode_not_tracked_in_git() {
        let json = r#"{"schema_version":2,"runner_registry":[],"artifacts":[
          {"artifact_id":"a","path":"foo/a.generated.json",
           "materialization_mode":"not-tracked-in-git",
           "generator":{"runner":"buck2","generator_target":"//foo:bar",
             "operation_id":"emit-a","output_mode":"stdout-json"}}
        ]}"#;
        let cp = ControlPlane::from_json(json).unwrap();
        assert!(cp.artifacts[0].is_not_tracked_in_git());
        assert_eq!(cp.decommit_paths(), vec!["foo/a.generated.json"]);
    }

    #[test]
    fn round_trip_extra_fields() {
        // v1 fields that the kernel does not know about must round-trip without loss.
        let json = r#"{"schema_version":1,"unknown_field":"preserved","artifacts":[]}"#;
        let cp = ControlPlane::from_json(json).unwrap();
        assert_eq!(
            cp.extra.get("unknown_field").and_then(|v| v.as_str()),
            Some("preserved")
        );
    }
}
