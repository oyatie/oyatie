//! File-backed Foundry run ledger adapter.
//!
//! The store persists a replayable run-ledger snapshot while rejecting stale
//! overwrites that would mutate already-persisted immutable run identity.

use std::fs;
use std::path::PathBuf;

use intelligence_capability_domain::AutonomyTier;
use intelligence_run_domain::{Run, RunDisposition, RunError, RunLedger, RunState};
use data_boundary_kernel::{
    Classified, DataClass, PrivacyDataClass, parse_data_class_pascal_label,
    privacy_data_classes_from,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileRunLedgerStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileRunLedgerStoreError {
    Io(String),
    MalformedRecord,
    LedgerDiverged,
    InvalidLedgerHistory,
}

impl FileRunLedgerStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<RunLedger, FileRunLedgerStoreError> {
        if !self.path.exists() {
            return Ok(RunLedger::default());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        let runs = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_run)
            .collect::<Result<Vec<_>, _>>()?;
        RunLedger::from_runs(runs).map_err(map_run_error)
    }

    pub fn save_ledger(&self, ledger: &RunLedger) -> Result<usize, FileRunLedgerStoreError> {
        RunLedger::from_runs(ledger.runs().to_vec()).map_err(map_run_error)?;
        let persisted = self.load()?;
        let changes = count_run_changes(persisted.runs(), ledger.runs())?;
        if changes == 0 {
            return Ok(0);
        }
        let mut contents = String::new();
        for run in ledger.runs() {
            contents.push_str(&encode_run(run));
            contents.push('\n');
        }
        fs::write(&self.path, contents).map_err(map_io_error)?;
        Ok(changes)
    }
}

fn count_run_changes(
    persisted: &[Run],
    requested: &[Run],
) -> Result<usize, FileRunLedgerStoreError> {
    if persisted.len() > requested.len() {
        return Err(FileRunLedgerStoreError::LedgerDiverged);
    }
    let mut changes = requested.len() - persisted.len();
    for (persisted_run, requested_run) in persisted.iter().zip(requested) {
        if persisted_run == requested_run {
            continue;
        }
        if is_allowed_run_transition(persisted_run, requested_run) {
            changes += 1;
        } else {
            return Err(FileRunLedgerStoreError::LedgerDiverged);
        }
    }
    Ok(changes)
}

fn is_allowed_run_transition(persisted: &Run, requested: &Run) -> bool {
    persisted.run_id == requested.run_id
        && persisted.tenant_id == requested.tenant_id
        && persisted.capability_id == requested.capability_id
        && persisted.initiator_user_id == requested.initiator_user_id
        && persisted.autonomy_tier_used == requested.autonomy_tier_used
        && persisted.data_classes_touched == requested.data_classes_touched
        && persisted.data_class == requested.data_class
        && persisted.region == requested.region
        && persisted.idempotency_key == requested.idempotency_key
        && persisted.started_at_epoch_seconds == requested.started_at_epoch_seconds
        && persisted.schema_version == requested.schema_version
        && persisted.state.value == RunState::Running
        && requested.state.value != RunState::Running
        && persisted.disposition.value.is_none()
        && requested.disposition.value.is_some()
        && persisted.completed_at_epoch_seconds.value.is_none()
        && requested.completed_at_epoch_seconds.value.is_some()
}

fn encode_run(run: &Run) -> String {
    format!(
        "v1|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        encode_str(&run.run_id.value),
        encode_str(&run.tenant_id.value),
        encode_str(&run.capability_id.value),
        encode_str(&run.initiator_user_id.value),
        encode_autonomy_tier(run.autonomy_tier_used.value),
        encode_data_classes(&run.data_classes_touched.value),
        encode_data_class(run.data_class.value),
        encode_run_state(run.state.value),
        encode_run_disposition(run.disposition.value),
        encode_str(&run.region.value),
        encode_str(&run.idempotency_key.value),
        run.started_at_epoch_seconds.value,
        encode_optional_u64(run.completed_at_epoch_seconds.value),
        run.schema_version.value
    )
}

fn decode_run(line: &str) -> Result<Run, FileRunLedgerStoreError> {
    let mut input = line;
    input = input
        .strip_prefix("v1|")
        .ok_or(FileRunLedgerStoreError::MalformedRecord)?;
    let run_id = take_len_prefixed_then_separator(&mut input)?;
    let tenant_id = take_len_prefixed_then_separator(&mut input)?;
    let capability_id = take_len_prefixed_then_separator(&mut input)?;
    let initiator_user_id = take_len_prefixed_then_separator(&mut input)?;
    let autonomy_tier_used = decode_autonomy_tier(take_until_separator(&mut input)?)?;
    let data_classes_touched = decode_data_classes(take_until_separator(&mut input)?)?;
    let data_classes_touched = privacy_data_classes_from(&data_classes_touched)
        .map_err(|_| FileRunLedgerStoreError::MalformedRecord)?;
    let data_class = decode_data_class(take_until_separator(&mut input)?)?;
    let state = decode_run_state(take_until_separator(&mut input)?)?;
    let disposition = decode_run_disposition(take_until_separator(&mut input)?)?;
    let region = take_len_prefixed_then_separator(&mut input)?;
    let idempotency_key = take_len_prefixed_then_separator(&mut input)?;
    let started_at_epoch_seconds = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileRunLedgerStoreError::MalformedRecord)?;
    let completed_at_epoch_seconds = decode_optional_u64(take_until_separator(&mut input)?)?;
    let schema_version = input
        .parse::<u32>()
        .map_err(|_| FileRunLedgerStoreError::MalformedRecord)?;

    Ok(Run {
        run_id: Classified::new(run_id, DataClass::InternalOnly),
        tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
        capability_id: Classified::new(capability_id, DataClass::InternalOnly),
        initiator_user_id: Classified::new(initiator_user_id, DataClass::PiiIdentifying),
        autonomy_tier_used: Classified::new(autonomy_tier_used, DataClass::InternalOnly),
        data_classes_touched: Classified::new(data_classes_touched, DataClass::InternalOnly),
        data_class: Classified::new(data_class, DataClass::InternalOnly),
        state: Classified::new(state, DataClass::InternalOnly),
        disposition: Classified::new(disposition, DataClass::InternalOnly),
        region: Classified::new(region, DataClass::InternalOnly),
        idempotency_key: Classified::new(idempotency_key, DataClass::InternalOnly),
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

fn take_len_prefixed_then_separator(input: &mut &str) -> Result<String, FileRunLedgerStoreError> {
    let value = take_len_prefixed(input)?;
    take_separator(input)?;
    Ok(value)
}

fn take_len_prefixed(input: &mut &str) -> Result<String, FileRunLedgerStoreError> {
    let Some((len_text, rest)) = input.split_once(':') else {
        return Err(FileRunLedgerStoreError::MalformedRecord);
    };
    let len = len_text
        .parse::<usize>()
        .map_err(|_| FileRunLedgerStoreError::MalformedRecord)?;
    if rest.len() < len {
        return Err(FileRunLedgerStoreError::MalformedRecord);
    }
    let (value, remainder) = rest.split_at(len);
    *input = remainder;
    Ok(value.to_string())
}

fn take_separator(input: &mut &str) -> Result<(), FileRunLedgerStoreError> {
    if let Some(rest) = input.strip_prefix('|') {
        *input = rest;
        Ok(())
    } else {
        Err(FileRunLedgerStoreError::MalformedRecord)
    }
}

fn take_until_separator<'a>(input: &mut &'a str) -> Result<&'a str, FileRunLedgerStoreError> {
    let Some((field, rest)) = input.split_once('|') else {
        return Err(FileRunLedgerStoreError::MalformedRecord);
    };
    *input = rest;
    Ok(field)
}

fn encode_autonomy_tier(tier: AutonomyTier) -> &'static str {
    match tier {
        AutonomyTier::T1ViewOnly => "T1ViewOnly",
        AutonomyTier::T2Advisory => "T2Advisory",
        AutonomyTier::T3ExecuteWithApproval => "T3ExecuteWithApproval",
        AutonomyTier::T4AutoExecute => "T4AutoExecute",
    }
}

fn decode_autonomy_tier(value: &str) -> Result<AutonomyTier, FileRunLedgerStoreError> {
    match value {
        "T1ViewOnly" => Ok(AutonomyTier::T1ViewOnly),
        "T2Advisory" => Ok(AutonomyTier::T2Advisory),
        "T3ExecuteWithApproval" => Ok(AutonomyTier::T3ExecuteWithApproval),
        "T4AutoExecute" => Ok(AutonomyTier::T4AutoExecute),
        _ => Err(FileRunLedgerStoreError::MalformedRecord),
    }
}

fn encode_run_state(state: RunState) -> &'static str {
    match state {
        RunState::Running => "Running",
        RunState::Succeeded => "Succeeded",
        RunState::Failed => "Failed",
        RunState::Cancelled => "Cancelled",
        RunState::RejectedAutonomy => "RejectedAutonomy",
        RunState::RejectedClass => "RejectedClass",
        RunState::RejectedBudget => "RejectedBudget",
        RunState::RejectedLicense => "RejectedLicense",
        RunState::RejectedPolicy => "RejectedPolicy",
    }
}

fn decode_run_state(value: &str) -> Result<RunState, FileRunLedgerStoreError> {
    match value {
        "Running" => Ok(RunState::Running),
        "Succeeded" => Ok(RunState::Succeeded),
        "Failed" => Ok(RunState::Failed),
        "Cancelled" => Ok(RunState::Cancelled),
        "RejectedAutonomy" => Ok(RunState::RejectedAutonomy),
        "RejectedClass" => Ok(RunState::RejectedClass),
        "RejectedBudget" => Ok(RunState::RejectedBudget),
        "RejectedLicense" => Ok(RunState::RejectedLicense),
        "RejectedPolicy" => Ok(RunState::RejectedPolicy),
        _ => Err(FileRunLedgerStoreError::MalformedRecord),
    }
}

fn encode_run_disposition(disposition: Option<RunDisposition>) -> &'static str {
    match disposition {
        None => "None",
        Some(RunDisposition::Success) => "Success",
        Some(RunDisposition::FailureClass) => "FailureClass",
        Some(RunDisposition::FailureProvider) => "FailureProvider",
        Some(RunDisposition::FailureTimeout) => "FailureTimeout",
        Some(RunDisposition::FailureBudget) => "FailureBudget",
        Some(RunDisposition::FailureAutonomy) => "FailureAutonomy",
        Some(RunDisposition::FailureLicense) => "FailureLicense",
        Some(RunDisposition::FailureAuthorization) => "FailureAuthorization",
    }
}

fn decode_run_disposition(value: &str) -> Result<Option<RunDisposition>, FileRunLedgerStoreError> {
    match value {
        "None" => Ok(None),
        "Success" => Ok(Some(RunDisposition::Success)),
        "FailureClass" => Ok(Some(RunDisposition::FailureClass)),
        "FailureProvider" => Ok(Some(RunDisposition::FailureProvider)),
        "FailureTimeout" => Ok(Some(RunDisposition::FailureTimeout)),
        "FailureBudget" => Ok(Some(RunDisposition::FailureBudget)),
        "FailureAutonomy" => Ok(Some(RunDisposition::FailureAutonomy)),
        "FailureLicense" => Ok(Some(RunDisposition::FailureLicense)),
        "FailureAuthorization" => Ok(Some(RunDisposition::FailureAuthorization)),
        _ => Err(FileRunLedgerStoreError::MalformedRecord),
    }
}

fn encode_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "None".to_string(), |value| value.to_string())
}

fn decode_optional_u64(value: &str) -> Result<Option<u64>, FileRunLedgerStoreError> {
    if value == "None" {
        return Ok(None);
    }
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| FileRunLedgerStoreError::MalformedRecord)
}

fn encode_data_classes(data_classes: &[PrivacyDataClass]) -> String {
    data_classes
        .iter()
        .map(|data_class| encode_data_class(data_class.data_class()))
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_data_classes(value: &str) -> Result<Vec<DataClass>, FileRunLedgerStoreError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value.split(',').map(decode_data_class).collect()
}

fn encode_data_class(data_class: DataClass) -> &'static str {
    data_class.pascal_label()
}

fn decode_data_class(value: &str) -> Result<DataClass, FileRunLedgerStoreError> {
    parse_data_class_pascal_label(value).ok_or(FileRunLedgerStoreError::MalformedRecord)
}

fn map_run_error(error: RunError) -> FileRunLedgerStoreError {
    match error {
        RunError::InvalidRunHistory => FileRunLedgerStoreError::InvalidLedgerHistory,
        RunError::InvalidTenantId
        | RunError::InvalidCapabilityId
        | RunError::InvalidInitiatorId
        | RunError::MissingDataClasses
        | RunError::InvalidDataClass
        | RunError::EmptyRegion
        | RunError::EmptyIdempotencyKey
        | RunError::RunNotFound
        | RunError::RunNotRunning => FileRunLedgerStoreError::MalformedRecord,
    }
}

fn map_io_error(error: std::io::Error) -> FileRunLedgerStoreError {
    FileRunLedgerStoreError::Io(error.to_string())
}
