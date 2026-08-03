//! `oya gate emit architecture-map` — walks the workspace and emits the
//! architecture-map JSON. Backs the visualization-as-code directive.

use std::path::PathBuf;
use std::time::Instant;

use oya_intelligence_architecture_map_app::{MapBuildError, build_artifact, emit_artifact_json};

use crate::usage;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArchitectureMapEmitArgs {
    workspace_root: PathBuf,
    out_path: PathBuf,
}

pub(crate) fn parse_architecture_map_emit_args(
    args: Vec<String>,
) -> Result<ArchitectureMapEmitArgs, String> {
    let mut parsed = ArchitectureMapEmitArgs {
        workspace_root: PathBuf::from("."),
        out_path: PathBuf::from("registry/graph/architecture-map.json"),
    };
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--workspace-root" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.workspace_root = PathBuf::from(path);
            }
            "--out" => {
                let Some(path) = iter.next() else {
                    return Err(usage());
                };
                parsed.out_path = PathBuf::from(path);
            }
            _ => return Err(usage()),
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArchitectureMapEmitReport {
    pub node_count: usize,
    pub edge_count: usize,
    pub orphan_count: usize,
    pub resolved_workspace_crate_count: usize,
    pub represented_workspace_crate_count: usize,
    pub coverage_ratio_basis_points: u32,
    pub duration_ms: u64,
}

pub(crate) fn emit_architecture_map_gate(
    args: ArchitectureMapEmitArgs,
) -> Result<ArchitectureMapEmitReport, String> {
    let start = Instant::now();
    let artifact = build_artifact(&args.workspace_root).map_err(format_build_error)?;
    emit_artifact_json(&artifact, &args.out_path).map_err(format_build_error)?;
    Ok(ArchitectureMapEmitReport {
        node_count: artifact.map.node_count(),
        edge_count: artifact.map.edge_count(),
        orphan_count: artifact.coverage.orphan_crate_ids.len(),
        resolved_workspace_crate_count: artifact.coverage.resolved_workspace_crate_count,
        represented_workspace_crate_count: artifact.coverage.represented_workspace_crate_count,
        coverage_ratio_basis_points: artifact.coverage.coverage_ratio_basis_points,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

fn format_build_error(error: MapBuildError) -> String {
    match error {
        MapBuildError::Io { path, source } => {
            format!(
                "architecture-map I/O failed at {}: {source}",
                path.display()
            )
        }
        MapBuildError::Map(error) => format!("architecture-map build failed: {error:?}"),
        MapBuildError::WorkspaceMembers(error) => {
            format!("architecture-map workspace membership resolution failed: {error}")
        }
        MapBuildError::InvalidExistingArtifact { path, reason } => format!(
            "architecture-map existing artifact is invalid at {}: {reason}",
            path.display()
        ),
        MapBuildError::CountDiscontinuity {
            previous_node_count,
            proposed_node_count,
            minimum_expected_node_count,
        } => format!(
            "architecture-map suspicious node-count discontinuity: previous={previous_node_count}, proposed={proposed_node_count}, minimum={minimum_expected_node_count}"
        ),
        MapBuildError::IncompleteCoverage {
            missing_workspace_crate_ids,
            orphan_crate_ids,
        } => format!(
            "architecture-map incomplete crate coverage: missing={missing_workspace_crate_ids:?}, orphaned={orphan_crate_ids:?}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_defaults() {
        let args = parse_architecture_map_emit_args(vec![]).unwrap();
        assert_eq!(args.workspace_root, PathBuf::from("."));
        assert_eq!(
            args.out_path,
            PathBuf::from("registry/graph/architecture-map.json")
        );
    }

    #[test]
    fn parse_args_overrides() {
        let args = parse_architecture_map_emit_args(vec![
            "--workspace-root".into(),
            "/tmp/ws".into(),
            "--out".into(),
            "/tmp/out.json".into(),
        ])
        .unwrap();
        assert_eq!(args.workspace_root.display().to_string(), "/tmp/ws");
        assert_eq!(args.out_path.display().to_string(), "/tmp/out.json");
    }

    #[test]
    fn parse_args_unknown_flag_errors() {
        let result = parse_architecture_map_emit_args(vec!["--bogus".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn emit_gate_uses_safe_provenance_bearing_artifact_path() {
        let root =
            std::env::temp_dir().join(format!("oya-architecture-map-gate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("crates/a")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .unwrap();
        std::fs::write(
            root.join("crates/a/Cargo.toml"),
            "[package]\nname = \"a\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let out_path = root.join("architecture-map.json");

        let report = emit_architecture_map_gate(ArchitectureMapEmitArgs {
            workspace_root: root.clone(),
            out_path: out_path.clone(),
        })
        .unwrap();

        assert_eq!(report.resolved_workspace_crate_count, 1);
        assert_eq!(report.represented_workspace_crate_count, 1);
        assert_eq!(report.coverage_ratio_basis_points, 10_000);
        let body = std::fs::read_to_string(out_path).unwrap();
        assert!(body.contains("\"producer_version\""));
        assert!(body.contains("\"source_digest_sha256\""));
        let _ = std::fs::remove_dir_all(&root);
    }
}
