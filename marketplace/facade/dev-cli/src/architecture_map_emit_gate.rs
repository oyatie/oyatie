//! `oya gate emit architecture-map` — walks the workspace and emits the
//! architecture-map JSON. Backs the visualization-as-code directive.

use std::path::PathBuf;
use std::time::Instant;

use oya_intelligence_architecture_map_app::{MapBuildError, build_map, emit_json};

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
    pub duration_ms: u64,
}

pub(crate) fn emit_architecture_map_gate(
    args: ArchitectureMapEmitArgs,
) -> Result<ArchitectureMapEmitReport, String> {
    let start = Instant::now();
    let map = build_map(&args.workspace_root).map_err(format_build_error)?;
    emit_json(&map, &args.out_path).map_err(format_build_error)?;
    Ok(ArchitectureMapEmitReport {
        node_count: map.node_count(),
        edge_count: map.edge_count(),
        orphan_count: map.orphans().len(),
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
        MapBuildError::WorkspaceMembers(detail) => {
            format!("architecture-map workspace-member resolution failed: {detail}")
        }
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
}
