use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use architecture_graph_generator_app::masterplan_from_generated;
use serde_json::Value;
use tempfile::NamedTempFile;

pub const MASTERPLAN_RESOURCE_ENV: &str = "OYA_ARCH_GRAPH_MASTERPLAN";
const DEFAULT_MASTERPLAN: &str = "docs/machine-readable/masterplan.generated.json";
const DECISIONS_DIR: &str = "docs/decisions";

#[derive(Debug)]
pub struct MasterplanInput {
    path: PathBuf,
    _temporary: Option<NamedTempFile>,
}

impl MasterplanInput {
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[cfg(test)]
    pub fn is_temporary(&self) -> bool {
        self._temporary.is_some()
    }
}

pub fn resolve_masterplan_input(repo_root: &Path) -> Result<MasterplanInput, String> {
    let declared = std::env::var_os(MASTERPLAN_RESOURCE_ENV);
    resolve_masterplan_input_with(repo_root, declared.as_deref())
}

pub fn resolve_masterplan_input_with(
    repo_root: &Path,
    declared: Option<&OsStr>,
) -> Result<MasterplanInput, String> {
    if let Some(declared) = declared {
        if declared.is_empty() {
            return Err(format!("{MASTERPLAN_RESOURCE_ENV} must not be empty"));
        }
        let declared = PathBuf::from(declared);
        let path = if declared.is_absolute() {
            declared
        } else {
            repo_root.join(declared)
        };
        validate_projection(&path, MASTERPLAN_RESOURCE_ENV)?;
        return Ok(MasterplanInput {
            path,
            _temporary: None,
        });
    }

    let canonical = repo_root.join(DEFAULT_MASTERPLAN);
    match fs::symlink_metadata(&canonical) {
        Ok(_) => {
            validate_projection(&canonical, "controller-materialized masterplan projection")?;
            return Ok(MasterplanInput {
                path: canonical,
                _temporary: None,
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "inspect controller-materialized masterplan projection {}: {error}",
                canonical.display()
            ));
        }
    }

    let (_, bytes) = ci_generated_artifact_freshness::render_masterplan_projection_from_decisions(
        &repo_root.join(DECISIONS_DIR),
    )?;
    validate_projection_bytes(Path::new("<temporary-masterplan-projection>"), &bytes)?;
    let mut temporary = tempfile::Builder::new()
        .prefix("oya-architecture-graph-masterplan-")
        .suffix(".generated.json")
        .tempfile()
        .map_err(|error| format!("create temporary masterplan projection: {error}"))?;
    temporary
        .write_all(bytes.as_bytes())
        .map_err(|error| format!("write temporary masterplan projection: {error}"))?;
    let path = temporary.path().to_path_buf();
    Ok(MasterplanInput {
        path,
        _temporary: Some(temporary),
    })
}

fn validate_projection(path: &Path, label: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("{label} {} is unavailable: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{label} {} must be a regular non-symlink file",
            path.display()
        ));
    }
    let bytes = fs::read_to_string(path)
        .map_err(|error| format!("read {label} {}: {error}", path.display()))?;
    validate_projection_bytes(path, &bytes)
}

fn validate_projection_bytes(path: &Path, bytes: &str) -> Result<(), String> {
    let value: Value = serde_json::from_str(bytes)
        .map_err(|error| format!("parse masterplan projection {}: {error}", path.display()))?;
    masterplan_from_generated(&value)
        .map(|_| ())
        .map_err(|error| format!("validate masterplan projection {}: {error}", path.display()))
}
