//! `oya gate validate masterplan-drift` (ADR-0364 D4).
//!
//! The drift gate is the inspection mechanism that keeps the generated
//! masterplan projection honest. Contributor-owned output paths still use the
//! byte-for-byte `oya gen masterplan --check` path. Controller-materialized
//! outputs declared in the generated-artifact control plane validate that the
//! source ADR projection regenerates successfully, then leave final bytes to the
//! controller named by that manifest (Amazon "mechanisms, not intentions").

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::commands::generate::masterplan::{self, GenMasterplanArgs};

const GENERATED_ARTIFACT_CONTROL_PLANE: &str = "registry/generated-artifact-control-plane.json";
const CONTROLLER_MERGE_POLICY: &str = "never-manual-merge-regenerate-from-source-tree";
const CONTROLLER_RUNNER: &str = "oya-ci-native-controller";
const CONTROLLER_OUTPUT_MODE: &str = "controller-materialized";
const CONTROLLER_TARGET_PREFIX: &str = "oya-ci://generated-artifact-controller/";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MasterplanDriftArgs {
    pub(crate) decisions_dir: PathBuf,
    pub(crate) output: PathBuf,
}

pub(crate) fn parse_masterplan_drift_args(
    args: Vec<String>,
) -> Result<MasterplanDriftArgs, String> {
    let mut parsed = MasterplanDriftArgs {
        decisions_dir: PathBuf::from("docs/decisions"),
        output: PathBuf::from("docs/machine-readable/masterplan.generated.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--decisions-dir" => {
                parsed.decisions_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--decisions-dir requires a value".to_string())?,
                );
            }
            "--masterplan" | "--output" => {
                parsed.output = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--masterplan requires a value".to_string())?,
                );
            }
            other => {
                return Err(format!(
                    "masterplan-drift: unknown flag {other:?}; allowed: --decisions-dir, --masterplan"
                ));
            }
        }
    }
    Ok(parsed)
}

pub(crate) fn run_masterplan_drift(args: Vec<String>) -> ExitCode {
    let parsed = match parse_masterplan_drift_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };

    if is_controller_materialized_output(&parsed.output) {
        return match masterplan::render_projection(&parsed.decisions_dir) {
            Ok((projection, _json)) => {
                println!(
                    "masterplan-drift passed: {} is controller-materialized by {}; source projection regenerated successfully ({} accepted planning_impact ADRs, {} deliverables, {} milestones)",
                    parsed.output.display(),
                    GENERATED_ARTIFACT_CONTROL_PLANE,
                    projection.adr_count,
                    projection.deliverable_count,
                    projection.milestones.len()
                );
                ExitCode::SUCCESS
            }
            Err(message) => {
                eprintln!("masterplan-drift: source projection regeneration failed: {message}");
                ExitCode::FAILURE
            }
        };
    }

    // Delegate to the generator's --check path: regenerate in-memory and diff
    // against the committed projection. The generator owns the diff + messaging.
    masterplan::run(
        vec![
            "--check".to_string(),
            "--decisions-dir".to_string(),
            parsed.decisions_dir.to_string_lossy().into_owned(),
            "--output".to_string(),
            parsed.output.to_string_lossy().into_owned(),
        ],
        "oya gate validate masterplan-drift [--decisions-dir <docs/decisions>] [--masterplan <docs/machine-readable/masterplan.generated.json>]",
    )
}

fn is_controller_materialized_output(output: &Path) -> bool {
    let manifest_text = match std::fs::read_to_string(GENERATED_ARTIFACT_CONTROL_PLANE) {
        Ok(text) => text,
        Err(_) => return false,
    };
    let manifest: serde_json::Value = match serde_json::from_str(&manifest_text) {
        Ok(value) => value,
        Err(_) => return false,
    };
    is_controller_materialized_output_in_manifest(output, &manifest)
}

fn is_controller_materialized_output_in_manifest(
    output: &Path,
    manifest: &serde_json::Value,
) -> bool {
    let normalized_output = normalize_repo_relative_slash_path(output);
    let Some(artifacts) = manifest
        .get("artifacts")
        .or_else(|| manifest.get("generated_artifacts"))
        .and_then(|value| value.as_array())
    else {
        return false;
    };

    artifacts.iter().any(|artifact| {
        let Some(path) = artifact.get("path").and_then(|value| value.as_str()) else {
            return false;
        };
        if path != normalized_output {
            return false;
        }
        if artifact
            .get("merge_policy")
            .and_then(|value| value.as_str())
            != Some(CONTROLLER_MERGE_POLICY)
        {
            return false;
        }

        let generator_output_mode = artifact
            .get("generator")
            .and_then(|generator| generator.get("output_mode"))
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let generator_runner = artifact
            .get("generator")
            .and_then(|generator| generator.get("runner"))
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let generator_target = artifact
            .get("generator")
            .and_then(|generator| generator.get("generator_target"))
            .and_then(|value| value.as_str())
            .unwrap_or_default();

        generator_output_mode == CONTROLLER_OUTPUT_MODE
            && generator_runner == CONTROLLER_RUNNER
            && generator_target.starts_with(CONTROLLER_TARGET_PREFIX)
    })
}

fn normalize_repo_relative_slash_path(path: &Path) -> String {
    let repo_relative = if path.is_absolute() {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };
    let mut normalized = repo_relative.to_string_lossy().replace('\\', "/");
    while let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    normalized
}

// Keep the parsed args constructible into generator args for any future
// callers that want the typed form rather than the argv reconstruction above.
impl From<MasterplanDriftArgs> for GenMasterplanArgs {
    fn from(args: MasterplanDriftArgs) -> Self {
        GenMasterplanArgs {
            decisions_dir: args.decisions_dir,
            output: args.output,
            write: false,
            check: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let parsed = parse_masterplan_drift_args(vec![]).expect("defaults");
        assert_eq!(parsed.decisions_dir, PathBuf::from("docs/decisions"));
        assert_eq!(
            parsed.output,
            PathBuf::from("docs/machine-readable/masterplan.generated.json")
        );
    }

    #[test]
    fn rejects_unknown_flag() {
        assert!(parse_masterplan_drift_args(vec!["--nope".into()]).is_err());
    }

    #[test]
    fn converts_to_check_generator_args() {
        let drift = parse_masterplan_drift_args(vec![]).expect("defaults");
        let generator: GenMasterplanArgs = drift.into();
        assert!(generator.check);
        assert!(!generator.write);
    }

    #[test]
    fn controller_materialized_manifest_match_is_accepted() {
        let manifest = serde_json::json!({
            "artifacts": [{
                "path": "docs/machine-readable/masterplan.generated.json",
                "merge_policy": CONTROLLER_MERGE_POLICY,
                "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
                "generator": {
                    "runner": "oya-ci-native-controller",
                    "generator_target": "oya-ci://generated-artifact-controller/planning/masterplan",
                    "output_mode": "controller-materialized"
                }
            }]
        });

        assert!(is_controller_materialized_output_in_manifest(
            Path::new("docs/machine-readable/masterplan.generated.json"),
            &manifest
        ));
    }

    #[test]
    fn controller_materialized_manifest_match_normalizes_dot_prefix() {
        let manifest = serde_json::json!({
            "artifacts": [{
                "path": "docs/machine-readable/masterplan.generated.json",
                "merge_policy": CONTROLLER_MERGE_POLICY,
                "generator": {
                    "runner": "oya-ci-native-controller",
                    "generator_target": "oya-ci://generated-artifact-controller/planning/masterplan",
                    "output_mode": "controller-materialized"
                }
            }]
        });

        assert!(is_controller_materialized_output_in_manifest(
            Path::new("./docs/machine-readable/masterplan.generated.json"),
            &manifest
        ));
    }

    #[test]
    fn contributor_owned_manifest_match_still_uses_strict_check() {
        let manifest = serde_json::json!({
            "artifacts": [{
                "path": "docs/machine-readable/masterplan.generated.json",
                "merge_policy": "commit-regenerated-output",
                "generator": {
                    "output_mode": "branch-committed"
                }
            }]
        });

        assert!(!is_controller_materialized_output_in_manifest(
            Path::new("docs/machine-readable/masterplan.generated.json"),
            &manifest
        ));
    }

    #[test]
    fn transitional_materialization_wording_without_output_mode_is_not_controller_proof() {
        let manifest = serde_json::json!({
            "artifacts": [{
                "path": "docs/machine-readable/masterplan.generated.json",
                "merge_policy": CONTROLLER_MERGE_POLICY,
                "materialization_mode": "branch-committed-regenerated-until-controller-materialization",
                "generator": {
                    "runner": "oya-ci-native-controller",
                    "generator_target": "oya-ci://generated-artifact-controller/planning/masterplan",
                    "output_mode": "branch-committed"
                }
            }]
        });

        assert!(!is_controller_materialized_output_in_manifest(
            Path::new("docs/machine-readable/masterplan.generated.json"),
            &manifest
        ));
    }
}
