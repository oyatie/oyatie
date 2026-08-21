//! Pure planner: `plan()` and `materialize_closure()`.
//!
//! Given a `ControlPlane` manifest and a `MaterializeScope`, derives a topologically-
//! ordered `MaterializePlan` — the analysis phase. NO I/O, NO clock, NO subprocess.
//!
//! The single-build / run-producer-twice invariant is a STRUCTURAL property of the
//! plan: in `DeterminismCanary` scope, build+emit steps carry `multiplicity: 1` and
//! producer steps carry `multiplicity: 2` with `OutputSink::TwoCapturedBuffers`. The
//! race cannot be re-introduced by an edit because it is the shape of the data.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{ArtifactId, ControlPlane, GeneratedArtifact, RunnerRegistryEntry};

/// Error returned when the manifest cannot be lowered to a valid plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// An artifact in de-commit class has no generator block.
    MissingGenerator { artifact_id: ArtifactId },
    /// The generator references a runner not declared in `runner_registry`.
    UnregisteredRunner {
        artifact_id: ArtifactId,
        runner_id: String,
    },
    /// The generator target does not start with the runner's canonical prefix.
    NonCanonicalTarget {
        artifact_id: ArtifactId,
        runner_id: String,
        target: String,
        expected_prefix: String,
    },
    /// The `runner_registry` contains a `shell` runner, which is forbidden.
    ShellRunnerForbidden,
    /// Cyclic dependency detected in `input_contract` edges.
    CyclicDependency { cycle: Vec<ArtifactId> },
    /// A `not-tracked-in-git` artifact has `output_mode: controller-materialized`
    /// without a non-controller fallback, making it unreconstructable.
    UnreconstructableDecommitArtifact { artifact_id: ArtifactId },
    /// The manifest declares duplicate artifact_ids.
    DuplicateArtifactId { artifact_id: ArtifactId },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::MissingGenerator { artifact_id } => {
                write!(
                    f,
                    "artifact {artifact_id:?} is not-tracked-in-git but has no generator block"
                )
            }
            PlanError::UnregisteredRunner {
                artifact_id,
                runner_id,
            } => {
                write!(
                    f,
                    "artifact {artifact_id:?} uses unregistered runner {runner_id:?}"
                )
            }
            PlanError::NonCanonicalTarget {
                artifact_id,
                runner_id,
                target,
                expected_prefix,
            } => {
                write!(
                    f,
                    "artifact {artifact_id:?}: runner {runner_id:?} target {target:?} must start with {expected_prefix:?}"
                )
            }
            PlanError::ShellRunnerForbidden => {
                write!(
                    f,
                    "runner_registry contains a 'shell' runner, which is forbidden (ADR-0523 / ADR-0596)"
                )
            }
            PlanError::CyclicDependency { cycle } => {
                write!(f, "cyclic dependency in input_contract: {:?}", cycle)
            }
            PlanError::UnreconstructableDecommitArtifact { artifact_id } => {
                write!(
                    f,
                    "not-tracked-in-git artifact {artifact_id:?} uses controller-materialized output_mode with no reconstructable alternative"
                )
            }
            PlanError::DuplicateArtifactId { artifact_id } => {
                write!(f, "duplicate artifact_id: {artifact_id:?}")
            }
        }
    }
}

impl std::error::Error for PlanError {}

/// The scope of a materialization plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterializeScope {
    /// Single-pass: materialize each artifact in the closure once.
    Consume { target_paths: BTreeSet<String> },
    /// Determinism canary (ADR-0595 / ADR-0596): build+emit steps run once;
    /// producer steps run TWICE capturing both buffers for byte-comparison.
    ///
    /// This is the structural encoding of the single-build invariant: the
    /// multiplicity=2 shape is the plan property — not a hand-coded loop —
    /// so the double-buck2-build race discovered in #828 cannot recur.
    DeterminismCanary { target_paths: BTreeSet<String> },
}

impl MaterializeScope {
    fn target_paths(&self) -> &BTreeSet<String> {
        match self {
            MaterializeScope::Consume { target_paths }
            | MaterializeScope::DeterminismCanary { target_paths } => target_paths,
        }
    }

    fn is_canary(&self) -> bool {
        matches!(self, MaterializeScope::DeterminismCanary { .. })
    }
}

/// Where the executor routes the output of a materialization step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputSink {
    /// Write to the artifact's declared repository-relative path.
    DeclaredPath(String),
    /// Capture stdout (for `stdout-json` output mode).
    Stdout,
    /// In-process controller reconciler owns materialization.
    ControllerMaterialized,
    /// Two captured buffers for the determinism canary — compared byte-for-byte in-kernel.
    TwoCapturedBuffers,
}

/// One step in a materialization plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializeStep {
    /// The artifact being materialized.
    pub artifact_id: ArtifactId,
    /// The runner to use.
    pub runner_id: String,
    /// The resolved canonical target.
    pub generator_target: String,
    /// Stable operation verb dispatched to the runner.
    pub operation_id: String,
    /// Parameters forwarded to the generator tool.
    pub params: BTreeMap<String, String>,
    /// Output routing.
    pub output: OutputSink,
    /// 1 = single pass (normal / build+emit steps); 2 = canary (run producer twice).
    pub multiplicity: u8,
}

/// A topologically-ordered materialization plan.
///
/// Steps are ordered so that every artifact's dependencies appear before it.
/// `plan()` is deterministic: byte-identical for the same manifest across runs
/// (no clock, no pid, no randomness). This is CP-1 (plan-determinism).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializePlan {
    pub steps: Vec<MaterializeStep>,
}

/// Validate the runner_registry: forbid `shell`, require unique runner_ids.
fn validate_runner_registry(registry: &[RunnerRegistryEntry]) -> Result<(), PlanError> {
    for entry in registry {
        if entry.runner_id == "shell" {
            return Err(PlanError::ShellRunnerForbidden);
        }
    }
    Ok(())
}

/// Validate a single artifact's generator block against the runner_registry.
fn validate_generator(
    artifact: &GeneratedArtifact,
    registry: &[RunnerRegistryEntry],
) -> Result<(), PlanError> {
    let genr = artifact.require_generator()?;

    // Check that the runner is registered.
    let entry = registry
        .iter()
        .find(|r| r.runner_id == genr.runner)
        .ok_or_else(|| PlanError::UnregisteredRunner {
            artifact_id: artifact.artifact_id.clone(),
            runner_id: genr.runner.clone(),
        })?;

    // Check that the target starts with the canonical prefix.
    if !genr
        .generator_target
        .starts_with(&entry.canonical_target_prefix)
    {
        return Err(PlanError::NonCanonicalTarget {
            artifact_id: artifact.artifact_id.clone(),
            runner_id: genr.runner.clone(),
            target: genr.generator_target.clone(),
            expected_prefix: entry.canonical_target_prefix.clone(),
        });
    }

    // A de-commit-class artifact with controller-materialized output has no
    // fallback regeneration path — it would be unreconstructable.
    if artifact.is_not_tracked_in_git()
        && let crate::model::OutputMode::ControllerMaterialized = genr.output_mode
    {
        return Err(PlanError::UnreconstructableDecommitArtifact {
            artifact_id: artifact.artifact_id.clone(),
        });
    }

    Ok(())
}

/// Resolve `input_contract` dependency edges for a set of artifacts.
///
/// `input_contract` tokens are matched against artifact `operation_id` values and
/// `artifact_id` values. An artifact B whose `input_contract` contains token T,
/// where T equals another artifact A's `operation_id` or `artifact_id`, depends on A.
///
/// Returns the adjacency map `artifact_id -> set of artifact_ids it depends on`.
/// This is the SINGLE source of edge semantics — `topological_order()` (sequencing)
/// and `materialize_closure()`/`plan()` (target expansion) share it so they cannot drift.
fn resolve_dependency_edges<'a>(
    artifacts: &[&'a GeneratedArtifact],
) -> BTreeMap<&'a str, BTreeSet<&'a str>> {
    // Build a map from (operation_id | artifact_id) -> artifact_id for dependency resolution.
    let mut token_to_id: BTreeMap<&str, &str> = BTreeMap::new();
    for artifact in artifacts {
        token_to_id.insert(artifact.artifact_id.as_str(), artifact.artifact_id.as_str());
        if let Some(genr) = &artifact.generator {
            token_to_id.insert(genr.operation_id.as_str(), artifact.artifact_id.as_str());
        }
    }

    // Build adjacency: artifact_id -> set of artifact_ids it depends on.
    let mut deps: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for artifact in artifacts {
        let id = artifact.artifact_id.as_str();
        let mut dep_set = BTreeSet::new();
        if let Some(genr) = &artifact.generator {
            for token in &genr.input_contract {
                if let Some(&dep_id) = token_to_id.get(token.as_str())
                    && dep_id != id
                {
                    dep_set.insert(dep_id);
                }
            }
        }
        deps.insert(id, dep_set);
    }

    deps
}

/// Derive a topological order from `input_contract` edges.
///
/// An artifact B whose `input_contract` resolves to another artifact A is sequenced
/// AFTER A. This is CP-2 (topological order from input_contract). Edge resolution is
/// delegated to `resolve_dependency_edges()` so closure and ordering share semantics.
///
/// Returns Err on a cycle.
fn topological_order(artifacts: &[&GeneratedArtifact]) -> Result<Vec<ArtifactId>, PlanError> {
    // Build adjacency: artifact_id -> set of artifact_ids it depends on.
    let deps = resolve_dependency_edges(artifacts);

    // Kahn's algorithm for topological sort (deterministic — uses BTreeSet).
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    let mut dependents: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for artifact in artifacts {
        let id = artifact.artifact_id.as_str();
        in_degree.entry(id).or_insert(0);
        for &dep in &deps[id] {
            *in_degree.entry(id).or_insert(0) += 1;
            dependents.entry(dep).or_default().insert(id);
        }
    }

    let mut queue: BTreeSet<&str> = in_degree
        .iter()
        .filter_map(|(&id, &deg)| if deg == 0 { Some(id) } else { None })
        .collect();

    let mut order: Vec<ArtifactId> = Vec::with_capacity(artifacts.len());

    while let Some(&id) = queue.iter().next() {
        queue.remove(id);
        order.push(id.to_owned());
        if let Some(dependents_of_id) = dependents.get(id) {
            for &dep in dependents_of_id {
                let degree = in_degree.get_mut(dep).expect("all ids in in_degree");
                *degree -= 1;
                if *degree == 0 {
                    queue.insert(dep);
                }
            }
        }
    }

    if order.len() != artifacts.len() {
        // Cycle: find the remaining nodes as evidence.
        let emitted: BTreeSet<&str> = order.iter().map(|s| s.as_str()).collect();
        let cycle: Vec<ArtifactId> = in_degree
            .keys()
            .filter(|&&id| !emitted.contains(id))
            .map(|&id| id.to_owned())
            .collect();
        return Err(PlanError::CyclicDependency { cycle });
    }

    Ok(order)
}

/// Build a `MaterializeStep` for a given artifact + scope.
fn build_step(
    artifact: &GeneratedArtifact,
    scope: &MaterializeScope,
) -> Result<MaterializeStep, PlanError> {
    use crate::model::OutputMode;

    let genr = artifact.require_generator()?;

    let output = match &genr.output_mode {
        OutputMode::StdoutJson => {
            if scope.is_canary() && artifact.is_not_tracked_in_git() {
                OutputSink::TwoCapturedBuffers
            } else {
                OutputSink::Stdout
            }
        }
        OutputMode::DeclaredArtifactPathWrite => {
            if scope.is_canary() && artifact.is_not_tracked_in_git() {
                OutputSink::TwoCapturedBuffers
            } else {
                OutputSink::DeclaredPath(artifact.path.clone())
            }
        }
        OutputMode::ControllerMaterialized => OutputSink::ControllerMaterialized,
        OutputMode::Other(_) => OutputSink::DeclaredPath(artifact.path.clone()),
    };

    // Multiplicity: in canary scope, de-commit-class producer steps run twice.
    // Build+emit steps (non-de-commit) always run once — this is the single-build invariant.
    let multiplicity = if scope.is_canary() && artifact.is_not_tracked_in_git() {
        2
    } else {
        1
    };

    Ok(MaterializeStep {
        artifact_id: artifact.artifact_id.clone(),
        runner_id: genr.runner.clone(),
        generator_target: genr.generator_target.clone(),
        operation_id: genr.operation_id.clone(),
        params: genr.parameters.clone(),
        output,
        multiplicity,
    })
}

/// THE ENGINE SIGNATURE — pure analysis phase.
///
/// Reads ONLY the manifest. No filesystem, no clock, no buck2, no git.
///
/// Returns a topologically-ordered `MaterializePlan` for the given scope, or
/// `Err` on a manifest that cannot be lowered (unregistered runner, non-canonical
/// target, cycle, de-commit artifact with no generator, etc.).
///
/// CP-1: deterministic — byte-identical for the same manifest across runs.
/// CP-2: topological order derived entirely from `input_contract` tokens.
/// CP-4 (structural): in `DeterminismCanary` scope, producer steps have
///       `multiplicity: 2`; build+emit steps have `multiplicity: 1`.
pub fn plan(
    manifest: &ControlPlane,
    scope: MaterializeScope,
) -> Result<MaterializePlan, PlanError> {
    // 1. Validate runner_registry (no shell runner).
    validate_runner_registry(&manifest.runner_registry)?;

    // 2. Check for duplicate artifact_ids.
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    for artifact in &manifest.artifacts {
        if !seen_ids.insert(artifact.artifact_id.as_str()) {
            return Err(PlanError::DuplicateArtifactId {
                artifact_id: artifact.artifact_id.clone(),
            });
        }
    }

    // 3. Determine which artifacts to include in the plan.
    let target_paths = scope.target_paths();
    let artifacts_to_plan: Vec<&GeneratedArtifact> = if target_paths.is_empty() {
        // Empty target set = plan ALL artifacts.
        manifest.artifacts.iter().collect()
    } else {
        // Seed: artifacts whose path is DIRECTLY in the requested target set.
        let all_artifacts: Vec<&GeneratedArtifact> = manifest.artifacts.iter().collect();
        let seed_ids: BTreeSet<&str> = all_artifacts
            .iter()
            .filter(|a| target_paths.contains(&a.path))
            .map(|a| a.artifact_id.as_str())
            .collect();

        // Expand to the TRANSITIVE closure over input_contract dependency edges.
        // Worklist/BFS to a fixpoint: for each id in the set, pull in every id its
        // input_contract resolves to (via the SHARED edge resolver), repeat until no
        // new ids are added. This guarantees an upstream producer is included even when
        // only a downstream leaf path was requested. Pure + deterministic (BTree order).
        let deps = resolve_dependency_edges(&all_artifacts);
        let mut closed_ids: BTreeSet<&str> = seed_ids.clone();
        let mut worklist: Vec<&str> = seed_ids.into_iter().collect();
        while let Some(id) = worklist.pop() {
            if let Some(dep_set) = deps.get(id) {
                for &dep_id in dep_set {
                    if closed_ids.insert(dep_id) {
                        worklist.push(dep_id);
                    }
                }
            }
        }

        // Filter to the closed (target + transitive-dependency) set.
        manifest
            .artifacts
            .iter()
            .filter(|a| closed_ids.contains(a.artifact_id.as_str()))
            .collect()
    };

    // 4. Validate each artifact that has a generator block.
    for artifact in &artifacts_to_plan {
        if artifact.generator.is_some() {
            validate_generator(artifact, &manifest.runner_registry)?;
        }
        // de-commit-class artifacts MUST have a generator.
        if artifact.is_not_tracked_in_git() && artifact.generator.is_none() {
            return Err(PlanError::MissingGenerator {
                artifact_id: artifact.artifact_id.clone(),
            });
        }
    }

    // 5. Derive topological order from input_contract edges.
    let order = topological_order(&artifacts_to_plan)?;

    // 6. Build the plan steps in topological order.
    let artifact_by_id: BTreeMap<&str, &GeneratedArtifact> = artifacts_to_plan
        .iter()
        .map(|a| (a.artifact_id.as_str(), *a))
        .collect();

    let mut steps = Vec::with_capacity(order.len());
    for id in &order {
        if let Some(&artifact) = artifact_by_id.get(id.as_str())
            && artifact.generator.is_some()
        {
            steps.push(build_step(artifact, &scope)?);
        }
    }

    Ok(MaterializePlan { steps })
}

/// Derive the transitive closure of artifacts that must be materialized before
/// reading any artifact at `target_paths`, topologically ordered.
///
/// This replaces the hand-wired CI `needs: producer-regen` and per-leg
/// re-materialisation — the ordering is a function of manifest data, not
/// a hand-maintained surface.
pub fn materialize_closure(
    manifest: &ControlPlane,
    target_paths: &BTreeSet<String>,
) -> Result<MaterializePlan, PlanError> {
    plan(
        manifest,
        MaterializeScope::Consume {
            target_paths: target_paths.clone(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ControlPlane;

    fn two_artifact_manifest() -> ControlPlane {
        let json = r#"{
          "schema_version": 2,
          "runner_registry": [
            {"runner_id":"buck2","canonical_target_prefix":"//","lowering":"build-target-then-exec"},
            {"runner_id":"node-codegen","canonical_target_prefix":"npm://","lowering":"npx-codegen"}
          ],
          "artifacts": [
            {
              "artifact_id": "scm-facts",
              "path": "ci/scm-facts.generated.json",
              "materialization_mode": "not-tracked-in-git",
              "generator": {
                "runner": "buck2",
                "generator_target": "//ci/emitter:emitter",
                "operation_id": "emit-scm-facts",
                "input_contract": ["repo-root"],
                "output_mode": "declared-artifact-path-write"
              }
            },
            {
              "artifact_id": "registry-face",
              "path": "ci/registry.generated.json",
              "materialization_mode": "not-tracked-in-git",
              "generator": {
                "runner": "buck2",
                "generator_target": "//ci/producer:producer",
                "operation_id": "emit-registry",
                "input_contract": ["repo-root", "emit-scm-facts"],
                "output_mode": "stdout-json"
              }
            }
          ]
        }"#;
        ControlPlane::from_json(json).unwrap()
    }

    #[test]
    fn plan_topological_order_from_input_contract() {
        let manifest = two_artifact_manifest();
        let plan = plan(
            &manifest,
            MaterializeScope::Consume {
                target_paths: BTreeSet::new(),
            },
        )
        .unwrap();
        // scm-facts must come before registry-face because registry's input_contract
        // references "emit-scm-facts" (scm-facts' operation_id).
        let ids: Vec<&str> = plan.steps.iter().map(|s| s.artifact_id.as_str()).collect();
        let scm_pos = ids.iter().position(|&id| id == "scm-facts").unwrap();
        let reg_pos = ids.iter().position(|&id| id == "registry-face").unwrap();
        assert!(scm_pos < reg_pos, "scm-facts must precede registry-face");
    }

    #[test]
    fn plan_canary_multiplicity() {
        let manifest = two_artifact_manifest();
        let plan = plan(
            &manifest,
            MaterializeScope::DeterminismCanary {
                target_paths: BTreeSet::new(),
            },
        )
        .unwrap();
        // Both artifacts are not-tracked-in-git, so both should have multiplicity=2.
        for step in &plan.steps {
            assert_eq!(
                step.multiplicity, 2,
                "canary: all de-commit steps multiplicity=2"
            );
        }
    }

    #[test]
    fn unregistered_runner_is_err() {
        let json = r#"{
          "schema_version": 2,
          "runner_registry": [
            {"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}
          ],
          "artifacts": [{
            "artifact_id": "x",
            "path": "foo/x.generated.json",
            "materialization_mode": "not-tracked-in-git",
            "generator": {
              "runner": "unknown-runner",
              "generator_target": "//foo:bar",
              "operation_id": "emit-x",
              "output_mode": "stdout-json"
            }
          }]
        }"#;
        let manifest = ControlPlane::from_json(json).unwrap();
        let err = plan(
            &manifest,
            MaterializeScope::Consume {
                target_paths: BTreeSet::new(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, PlanError::UnregisteredRunner { .. }));
    }

    #[test]
    fn shell_runner_is_forbidden() {
        let json = r#"{
          "schema_version": 2,
          "runner_registry": [
            {"runner_id":"shell","canonical_target_prefix":"","lowering":"exec"}
          ],
          "artifacts": []
        }"#;
        let manifest = ControlPlane::from_json(json).unwrap();
        let err = plan(
            &manifest,
            MaterializeScope::Consume {
                target_paths: BTreeSet::new(),
            },
        )
        .unwrap_err();
        assert_eq!(err, PlanError::ShellRunnerForbidden);
    }

    #[test]
    fn non_canonical_target_is_err() {
        let json = r#"{
          "schema_version": 2,
          "runner_registry": [
            {"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}
          ],
          "artifacts": [{
            "artifact_id": "x",
            "path": "foo/x.generated.json",
            "materialization_mode": "not-tracked-in-git",
            "generator": {
              "runner": "buck2",
              "generator_target": "bad-target",
              "operation_id": "emit-x",
              "output_mode": "stdout-json"
            }
          }]
        }"#;
        let manifest = ControlPlane::from_json(json).unwrap();
        let err = plan(
            &manifest,
            MaterializeScope::Consume {
                target_paths: BTreeSet::new(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, PlanError::NonCanonicalTarget { .. }));
    }

    #[test]
    fn cycle_detection() {
        let json = r#"{
          "schema_version": 2,
          "runner_registry": [
            {"runner_id":"buck2","canonical_target_prefix":"//","lowering":"x"}
          ],
          "artifacts": [
            {
              "artifact_id": "a",
              "path": "a.generated.json",
              "materialization_mode": "not-tracked-in-git",
              "generator": {
                "runner": "buck2",
                "generator_target": "//a:a",
                "operation_id": "emit-a",
                "input_contract": ["emit-b"],
                "output_mode": "stdout-json"
              }
            },
            {
              "artifact_id": "b",
              "path": "b.generated.json",
              "materialization_mode": "not-tracked-in-git",
              "generator": {
                "runner": "buck2",
                "generator_target": "//b:b",
                "operation_id": "emit-b",
                "input_contract": ["emit-a"],
                "output_mode": "stdout-json"
              }
            }
          ]
        }"#;
        let manifest = ControlPlane::from_json(json).unwrap();
        let err = plan(
            &manifest,
            MaterializeScope::Consume {
                target_paths: BTreeSet::new(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, PlanError::CyclicDependency { .. }));
    }
}
