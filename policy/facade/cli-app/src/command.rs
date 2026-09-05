use std::sync::Arc;

use policy_authoring_app::{PolicyProject, QualificationError};
use shared_ulid_id_kernel::SeededIdGenerator;

#[derive(Clone, Copy, Debug)]
pub enum CommandOutput {
    Report,
    UnsignedBundle,
}

#[derive(Debug)]
pub enum CommandError {
    Json(serde_json::Error),
    Qualification(QualificationError),
}

/// Offline command core. Deterministic IDs exist only for qualification;
/// serving engines always receive an independently supplied runtime generator.
/// The unsigned output is not accepted by the signed-file reader.
///
/// # Errors
/// Returns closed-schema decoding, qualification, or output encoding failures.
pub fn qualify_json(input: &[u8], output: CommandOutput) -> Result<String, CommandError> {
    let project: PolicyProject = serde_json::from_slice(input).map_err(CommandError::Json)?;
    let prepared = project
        .prepare(Arc::new(SeededIdGenerator::default()))
        .map_err(CommandError::Qualification)?;
    match output {
        CommandOutput::Report => serde_json::to_string_pretty(prepared.report()),
        CommandOutput::UnsignedBundle => serde_json::to_string_pretty(prepared.bundle()),
    }
    .map_err(CommandError::Json)
}
