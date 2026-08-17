//! Workflow Studio DSL loader domain.
//!
//! Parses `workflow_spec.v1` JSON, validates it through the emitter domain,
//! and returns a deterministic canonical representation for storage/diffing.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use workflow_studio_dsl_emitter::{WorkflowSpec, WorkflowSpecEmitError, emit_canonical_json};

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedWorkflowSpec {
    pub spec: WorkflowSpec,     // data_class: INTERNAL_ONLY
    pub canonical_json: String, // data_class: INTERNAL_ONLY
}

#[derive(Debug)]
pub enum WorkflowSpecLoadError {
    Parse(serde_json::Error),
    Validation(WorkflowSpecEmitError),
    RoundtripMismatch,
}

impl PartialEq for WorkflowSpecLoadError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Parse(left), Self::Parse(right)) => left.to_string() == right.to_string(),
            (Self::Validation(left), Self::Validation(right)) => left == right,
            (Self::RoundtripMismatch, Self::RoundtripMismatch) => true,
            _ => false,
        }
    }
}

impl From<WorkflowSpecEmitError> for WorkflowSpecLoadError {
    fn from(value: WorkflowSpecEmitError) -> Self {
        Self::Validation(value)
    }
}

pub fn load_workflow_spec(json: &str) -> Result<LoadedWorkflowSpec, WorkflowSpecLoadError> {
    let spec: WorkflowSpec = serde_json::from_str(json).map_err(WorkflowSpecLoadError::Parse)?;
    let canonical_json = emit_canonical_json(&spec).map_err(WorkflowSpecLoadError::from)?;
    let reparsed: WorkflowSpec =
        serde_json::from_str(&canonical_json).map_err(WorkflowSpecLoadError::Parse)?;
    if reparsed
        .canonicalized()
        .map_err(WorkflowSpecLoadError::from)?
        != spec.canonicalized().map_err(WorkflowSpecLoadError::from)?
    {
        return Err(WorkflowSpecLoadError::RoundtripMismatch);
    }
    Ok(LoadedWorkflowSpec {
        spec: reparsed,
        canonical_json,
    })
}

pub fn roundtrip_canonical_json(json: &str) -> Result<String, WorkflowSpecLoadError> {
    load_workflow_spec(json).map(|loaded| loaded.canonical_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_canonicalizes_workflow_spec_v1() {
        let input = r#"{
            "schema_version":"workflow_spec.v1",
            "tenant_id":"ten_acme",
            "definition_id":"wfd_onboarding",
            "version":"1.0.0",
            "nodes":[
                {"id":"wfn_start","kind":"http","label":"Start"},
                {"id":"wfn_transform","kind":"transform","label":"Prepare"}
            ],
            "edges":[{"from":"wfn_start","to":"wfn_transform","condition":"ok"}]
        }"#;

        let loaded = load_workflow_spec(input).unwrap();

        assert_eq!(loaded.spec.schema_version, WorkflowSpec::schema_version());
        assert_eq!(loaded.spec.nodes[0].id, "wfn_start");
        assert_eq!(
            roundtrip_canonical_json(&loaded.canonical_json),
            Ok(loaded.canonical_json)
        );
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let input = r#"{"schema_version":"workflow_spec.v0","tenant_id":"ten_acme","definition_id":"wfd_bad","version":"1.0.0","nodes":[{"id":"wfn_start","kind":"http","label":"Start"}],"edges":[]}"#;

        assert_eq!(
            load_workflow_spec(input),
            Err(WorkflowSpecLoadError::Validation(
                WorkflowSpecEmitError::InvalidSchemaVersion
            ))
        );
    }

    #[test]
    fn rejects_unknown_fields_due_to_contract_deny_unknown_fields() {
        let input = r#"{"schema_version":"workflow_spec.v1","tenant_id":"ten_acme","definition_id":"wfd_bad","version":"1.0.0","nodes":[{"id":"wfn_start","kind":"http","label":"Start","surprise":true}],"edges":[]}"#;

        assert!(matches!(
            load_workflow_spec(input),
            Err(WorkflowSpecLoadError::Parse(_))
        ));
    }
}
