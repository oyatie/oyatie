//! File-backed Foundry step ledger adapter.
//!
//! The store persists a replayable step-ledger snapshot while rejecting stale
//! overwrites that would mutate already-persisted immutable step identity.

use std::fs;
use std::path::PathBuf;

use data_boundary_kernel::{
    Classified, DataClass, PrivacyDataClass, parse_data_class_pascal_label,
    privacy_data_classes_from,
};
use intelligence_step_domain::{Step, StepDisposition, StepError, StepKind, StepLedger, StepState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileStepLedgerStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileStepLedgerStoreError {
    Io(String),
    MalformedRecord,
    LedgerDiverged,
    InvalidLedgerHistory,
}

impl FileStepLedgerStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<StepLedger, FileStepLedgerStoreError> {
        if !self.path.exists() {
            return Ok(StepLedger::default());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        let steps = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_step)
            .collect::<Result<Vec<_>, _>>()?;
        StepLedger::from_steps(steps).map_err(map_step_error)
    }

    pub fn save_ledger(&self, ledger: &StepLedger) -> Result<usize, FileStepLedgerStoreError> {
        StepLedger::from_steps(ledger.steps().to_vec()).map_err(map_step_error)?;
        let persisted = self.load()?;
        let changes = count_step_changes(persisted.steps(), ledger.steps())?;
        if changes == 0 {
            return Ok(0);
        }
        let mut contents = String::new();
        for step in ledger.steps() {
            contents.push_str(&encode_step(step));
            contents.push('\n');
        }
        fs::write(&self.path, contents).map_err(map_io_error)?;
        Ok(changes)
    }
}

fn count_step_changes(
    persisted: &[Step],
    requested: &[Step],
) -> Result<usize, FileStepLedgerStoreError> {
    if persisted.len() > requested.len() {
        return Err(FileStepLedgerStoreError::LedgerDiverged);
    }
    let mut changes = requested.len() - persisted.len();
    for (persisted_step, requested_step) in persisted.iter().zip(requested) {
        if persisted_step == requested_step {
            continue;
        }
        if is_allowed_step_transition(persisted_step, requested_step) {
            changes += 1;
        } else {
            return Err(FileStepLedgerStoreError::LedgerDiverged);
        }
    }
    Ok(changes)
}

fn is_allowed_step_transition(persisted: &Step, requested: &Step) -> bool {
    persisted.step_id == requested.step_id
        && persisted.run_id == requested.run_id
        && persisted.sequence == requested.sequence
        && persisted.kind == requested.kind
        && persisted.provider_kind == requested.provider_kind
        && persisted.model_ref == requested.model_ref
        && persisted.input_tokens == requested.input_tokens
        && persisted.output_tokens == requested.output_tokens
        && persisted.data_classes_touched == requested.data_classes_touched
        && persisted.data_class == requested.data_class
        && persisted.started_at_epoch_seconds == requested.started_at_epoch_seconds
        && persisted.schema_version == requested.schema_version
        && persisted.state.value == StepState::Running
        && requested.state.value != StepState::Running
        && persisted.disposition.value.is_none()
        && requested.disposition.value.is_some()
        && persisted.latency_ms.value.is_none()
        && requested.latency_ms.value.is_some()
        && persisted.completed_at_epoch_seconds.value.is_none()
        && requested.completed_at_epoch_seconds.value.is_some()
}

fn encode_step(step: &Step) -> String {
    format!(
        "v1|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        encode_str(&step.step_id.value),
        encode_str(&step.run_id.value),
        step.sequence.value,
        encode_step_kind(step.kind.value),
        encode_str(&step.provider_kind.value),
        encode_str(step.model_ref.value.as_deref().unwrap_or("")),
        if step.model_ref.value.is_some() {
            "1"
        } else {
            "0"
        },
        encode_optional_u32(step.input_tokens.value),
        encode_optional_u32(step.output_tokens.value),
        encode_optional_u32(step.latency_ms.value),
        encode_data_classes(&step.data_classes_touched.value),
        encode_data_class(step.data_class.value),
        encode_step_state(step.state.value),
        encode_step_disposition(step.disposition.value),
        step.started_at_epoch_seconds.value,
        encode_optional_u64(step.completed_at_epoch_seconds.value),
        step.schema_version.value
    )
}

fn decode_step(line: &str) -> Result<Step, FileStepLedgerStoreError> {
    let mut input = line;
    input = input
        .strip_prefix("v1|")
        .ok_or(FileStepLedgerStoreError::MalformedRecord)?;
    let step_id = take_len_prefixed_then_separator(&mut input)?;
    let run_id = take_len_prefixed_then_separator(&mut input)?;
    let sequence = take_until_separator(&mut input)?
        .parse::<u32>()
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)?;
    let kind = decode_step_kind(take_until_separator(&mut input)?)?;
    let provider_kind = take_len_prefixed_then_separator(&mut input)?;
    let model_ref_value = take_len_prefixed_then_separator(&mut input)?;
    let model_ref = match take_until_separator(&mut input)? {
        "0" => None,
        "1" => Some(model_ref_value),
        _ => return Err(FileStepLedgerStoreError::MalformedRecord),
    };
    let input_tokens = decode_optional_u32(take_until_separator(&mut input)?)?;
    let output_tokens = decode_optional_u32(take_until_separator(&mut input)?)?;
    let latency_ms = decode_optional_u32(take_until_separator(&mut input)?)?;
    let data_classes_touched = decode_data_classes(take_until_separator(&mut input)?)?;
    let data_classes_touched = privacy_data_classes_from(&data_classes_touched)
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)?;
    let data_class = decode_data_class(take_until_separator(&mut input)?)?;
    let state = decode_step_state(take_until_separator(&mut input)?)?;
    let disposition = decode_step_disposition(take_until_separator(&mut input)?)?;
    let started_at_epoch_seconds = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)?;
    let completed_at_epoch_seconds = decode_optional_u64(take_until_separator(&mut input)?)?;
    let schema_version = input
        .parse::<u32>()
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)?;

    Ok(Step {
        step_id: Classified::new(step_id, DataClass::InternalOnly),
        run_id: Classified::new(run_id, DataClass::InternalOnly),
        sequence: Classified::new(sequence, DataClass::InternalOnly),
        kind: Classified::new(kind, DataClass::InternalOnly),
        provider_kind: Classified::new(provider_kind, DataClass::InternalOnly),
        model_ref: Classified::new(model_ref, DataClass::InternalOnly),
        input_tokens: Classified::new(input_tokens, DataClass::BehavioralTenantProduct),
        output_tokens: Classified::new(output_tokens, DataClass::BehavioralTenantProduct),
        latency_ms: Classified::new(latency_ms, DataClass::BehavioralTenantProduct),
        data_classes_touched: Classified::new(data_classes_touched, DataClass::InternalOnly),
        data_class: Classified::new(data_class, DataClass::InternalOnly),
        state: Classified::new(state, DataClass::InternalOnly),
        disposition: Classified::new(disposition, DataClass::InternalOnly),
        started_at_epoch_seconds: Classified::new(
            started_at_epoch_seconds,
            DataClass::InternalOnly,
        ),
        completed_at_epoch_seconds: Classified::new(
            completed_at_epoch_seconds,
            DataClass::InternalOnly,
        ),
        schema_version: Classified::new(schema_version, DataClass::InternalOnly),
    })
}

fn encode_str(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn take_len_prefixed_then_separator(input: &mut &str) -> Result<String, FileStepLedgerStoreError> {
    let value = take_len_prefixed(input)?;
    take_separator(input)?;
    Ok(value)
}

fn take_len_prefixed(input: &mut &str) -> Result<String, FileStepLedgerStoreError> {
    let Some((len_text, rest)) = input.split_once(':') else {
        return Err(FileStepLedgerStoreError::MalformedRecord);
    };
    let len = len_text
        .parse::<usize>()
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)?;
    if rest.len() < len {
        return Err(FileStepLedgerStoreError::MalformedRecord);
    }
    let (value, remainder) = rest.split_at(len);
    *input = remainder;
    Ok(value.to_string())
}

fn take_separator(input: &mut &str) -> Result<(), FileStepLedgerStoreError> {
    if let Some(rest) = input.strip_prefix('|') {
        *input = rest;
        Ok(())
    } else {
        Err(FileStepLedgerStoreError::MalformedRecord)
    }
}

fn take_until_separator<'a>(input: &mut &'a str) -> Result<&'a str, FileStepLedgerStoreError> {
    let Some((field, rest)) = input.split_once('|') else {
        return Err(FileStepLedgerStoreError::MalformedRecord);
    };
    *input = rest;
    Ok(field)
}

fn encode_step_kind(kind: StepKind) -> &'static str {
    match kind {
        StepKind::ToolCall => "ToolCall",
        StepKind::ProviderCall => "ProviderCall",
        StepKind::Reasoning => "Reasoning",
        StepKind::Retrieval => "Retrieval",
        StepKind::Cite => "Cite",
        StepKind::Wait => "Wait",
        StepKind::Branch => "Branch",
    }
}

fn decode_step_kind(value: &str) -> Result<StepKind, FileStepLedgerStoreError> {
    match value {
        "ToolCall" => Ok(StepKind::ToolCall),
        "ProviderCall" => Ok(StepKind::ProviderCall),
        "Reasoning" => Ok(StepKind::Reasoning),
        "Retrieval" => Ok(StepKind::Retrieval),
        "Cite" => Ok(StepKind::Cite),
        "Wait" => Ok(StepKind::Wait),
        "Branch" => Ok(StepKind::Branch),
        _ => Err(FileStepLedgerStoreError::MalformedRecord),
    }
}

fn encode_step_state(state: StepState) -> &'static str {
    match state {
        StepState::Running => "Running",
        StepState::Succeeded => "Succeeded",
        StepState::Failed => "Failed",
        StepState::Cancelled => "Cancelled",
    }
}

fn decode_step_state(value: &str) -> Result<StepState, FileStepLedgerStoreError> {
    match value {
        "Running" => Ok(StepState::Running),
        "Succeeded" => Ok(StepState::Succeeded),
        "Failed" => Ok(StepState::Failed),
        "Cancelled" => Ok(StepState::Cancelled),
        _ => Err(FileStepLedgerStoreError::MalformedRecord),
    }
}

fn encode_step_disposition(disposition: Option<StepDisposition>) -> &'static str {
    match disposition {
        None => "None",
        Some(StepDisposition::Succeeded) => "Succeeded",
        Some(StepDisposition::FailedProvider) => "FailedProvider",
        Some(StepDisposition::FailedTimeout) => "FailedTimeout",
        Some(StepDisposition::FailedBudget) => "FailedBudget",
        Some(StepDisposition::Cancelled) => "Cancelled",
    }
}

fn decode_step_disposition(
    value: &str,
) -> Result<Option<StepDisposition>, FileStepLedgerStoreError> {
    match value {
        "None" => Ok(None),
        "Succeeded" => Ok(Some(StepDisposition::Succeeded)),
        "FailedProvider" => Ok(Some(StepDisposition::FailedProvider)),
        "FailedTimeout" => Ok(Some(StepDisposition::FailedTimeout)),
        "FailedBudget" => Ok(Some(StepDisposition::FailedBudget)),
        "Cancelled" => Ok(Some(StepDisposition::Cancelled)),
        _ => Err(FileStepLedgerStoreError::MalformedRecord),
    }
}

fn encode_optional_u32(value: Option<u32>) -> String {
    value.map_or_else(|| "None".to_string(), |value| value.to_string())
}

fn decode_optional_u32(value: &str) -> Result<Option<u32>, FileStepLedgerStoreError> {
    if value == "None" {
        return Ok(None);
    }
    value
        .parse::<u32>()
        .map(Some)
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)
}

fn encode_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "None".to_string(), |value| value.to_string())
}

fn decode_optional_u64(value: &str) -> Result<Option<u64>, FileStepLedgerStoreError> {
    if value == "None" {
        return Ok(None);
    }
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| FileStepLedgerStoreError::MalformedRecord)
}

fn encode_data_classes(data_classes: &[PrivacyDataClass]) -> String {
    data_classes
        .iter()
        .map(|data_class| encode_data_class(data_class.data_class()))
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_data_classes(value: &str) -> Result<Vec<DataClass>, FileStepLedgerStoreError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value.split(',').map(decode_data_class).collect()
}

fn encode_data_class(data_class: DataClass) -> &'static str {
    data_class.pascal_label()
}

fn decode_data_class(value: &str) -> Result<DataClass, FileStepLedgerStoreError> {
    parse_data_class_pascal_label(value).ok_or(FileStepLedgerStoreError::MalformedRecord)
}

fn map_step_error(error: StepError) -> FileStepLedgerStoreError {
    match error {
        StepError::InvalidStepHistory => FileStepLedgerStoreError::InvalidLedgerHistory,
        StepError::InvalidRunId
        | StepError::EmptyProviderKind
        | StepError::EmptyModelRef
        | StepError::MissingDataClasses
        | StepError::InvalidDataClass
        | StepError::StepNotFound
        | StepError::StepNotRunning => FileStepLedgerStoreError::MalformedRecord,
    }
}

fn map_io_error(error: std::io::Error) -> FileStepLedgerStoreError {
    FileStepLedgerStoreError::Io(error.to_string())
}
