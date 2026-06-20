use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile::{Builder, TempDir};

use crate::error::{CodexError, Result};

/// Turn-scoped output schema file.
///
/// `TempDir` deletes the directory when it is dropped; see the tempfile docs:
/// <https://docs.rs/tempfile/latest/tempfile/struct.TempDir.html>.
#[derive(Debug)]
pub(crate) struct OutputSchemaFile {
    path: Option<PathBuf>,
    _dir: Option<TempDir>,
}

impl OutputSchemaFile {
    pub(crate) fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

pub(crate) fn create_output_schema_file(schema: Option<&Value>) -> Result<OutputSchemaFile> {
    let Some(schema) = schema else {
        return Ok(OutputSchemaFile {
            path: None,
            _dir: None,
        });
    };

    if !schema.is_object() {
        return Err(CodexError::InvalidOutputSchema);
    }

    let dir = Builder::new().prefix("codex-output-schema-").tempdir()?;
    let path = dir.path().join("schema.json");
    let bytes = serde_json::to_vec(schema)?;
    fs::write(&path, bytes)?;

    Ok(OutputSchemaFile {
        path: Some(path),
        _dir: Some(dir),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn writes_object_schema_to_temp_file() {
        let file = create_output_schema_file(Some(&json!({"type": "object"}))).unwrap();
        let path = file.path().unwrap().to_path_buf();
        assert!(path.exists());
        drop(file);
        assert!(!path.exists());
    }

    #[test]
    fn rejects_non_object_schema() {
        let err = create_output_schema_file(Some(&json!(["not", "object"]))).unwrap_err();
        assert!(matches!(err, CodexError::InvalidOutputSchema));
    }
}
