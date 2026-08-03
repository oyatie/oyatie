//! File-backed Foundry evidence chain adapter.
//!
//! The adapter persists Foundry evidence records as append-only text records and
//! replays them through the kernel verifier before returning a chain to callers.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use intelligence_evidence_domain::{EvidenceChain, EvidenceKind, EvidenceRecord};
use oya_data_boundary_kernel::{
    Classified, DataClass, OperationalDataClass, PrivacyDataClass, parse_data_class_pascal_label,
    privacy_data_classes_from,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileEvidenceChainStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileEvidenceStoreError {
    Io(String),
    MalformedRecord,
    ChainDiverged,
    InvalidChain,
}

impl FileEvidenceChainStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<EvidenceChain, FileEvidenceStoreError> {
        if !self.path.exists() {
            return Ok(EvidenceChain::default());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        let records = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_record)
            .collect::<Result<Vec<_>, _>>()?;
        let chain = EvidenceChain::from_records(records)
            .map_err(|_| FileEvidenceStoreError::InvalidChain)?;
        if chain.verify() {
            Ok(chain)
        } else {
            Err(FileEvidenceStoreError::InvalidChain)
        }
    }

    pub fn append_chain(&self, chain: &EvidenceChain) -> Result<usize, FileEvidenceStoreError> {
        let persisted = self.load()?;
        let persisted_records = persisted.records();
        let requested_records = chain.records();
        if persisted_records.len() > requested_records.len()
            || persisted_records != &requested_records[..persisted_records.len()]
        {
            return Err(FileEvidenceStoreError::ChainDiverged);
        }

        if requested_records.len() == persisted_records.len() {
            return Ok(0);
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(map_io_error)?;
        for record in &requested_records[persisted_records.len()..] {
            writeln!(file, "{}", encode_record(record)).map_err(map_io_error)?;
        }
        Ok(requested_records.len() - persisted_records.len())
    }
}

fn encode_record(record: &EvidenceRecord) -> String {
    format!(
        "v1|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        encode_str(&record.evidence_id.value),
        encode_str(&record.run_id.value),
        encode_str(record.step_id.value.as_deref().unwrap_or("")),
        if record.step_id.value.is_some() {
            "1"
        } else {
            "0"
        },
        encode_str(&record.tenant_id.value),
        encode_str(&record.capability_id.value),
        encode_kind(record.kind.value),
        encode_str(&encode_fields(&record.fields.value)),
        encode_data_classes(&record.data_classes_touched.value),
        encode_data_class(record.data_class.value),
        encode_str(&record.prev_hash.value),
        encode_str(&record.hash.value),
        record.timestamp_epoch_seconds.value,
        record.schema_version.value,
    )
}

fn decode_record(line: &str) -> Result<EvidenceRecord, FileEvidenceStoreError> {
    let mut input = line;
    input = input
        .strip_prefix("v1|")
        .ok_or(FileEvidenceStoreError::MalformedRecord)?;
    let evidence_id = take_len_prefixed_then_separator(&mut input)?;
    let run_id = take_len_prefixed_then_separator(&mut input)?;
    let step_id_value = take_len_prefixed_then_separator(&mut input)?;
    let step_present = take_until_separator(&mut input)?;
    let step_id = match step_present {
        "0" => None,
        "1" => Some(step_id_value),
        _ => return Err(FileEvidenceStoreError::MalformedRecord),
    };
    let tenant_id = take_len_prefixed_then_separator(&mut input)?;
    let capability_id = take_len_prefixed_then_separator(&mut input)?;
    let kind = decode_kind(take_until_separator(&mut input)?)?;
    let fields = decode_fields(&take_len_prefixed_then_separator(&mut input)?)?;
    let data_classes_touched = decode_data_classes(take_until_separator(&mut input)?)?;
    let data_classes_touched = privacy_data_classes_from(&data_classes_touched)
        .map_err(|_| FileEvidenceStoreError::MalformedRecord)?;
    let data_class = decode_data_class(take_until_separator(&mut input)?)?;
    let prev_hash = take_len_prefixed_then_separator(&mut input)?;
    let hash = take_len_prefixed_then_separator(&mut input)?;
    let timestamp_epoch_seconds = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileEvidenceStoreError::MalformedRecord)?;
    let schema_version = input
        .parse::<u32>()
        .map_err(|_| FileEvidenceStoreError::MalformedRecord)?;

    Ok(EvidenceRecord {
        evidence_id: Classified::new(evidence_id, DataClass::InternalOnly),
        run_id: Classified::new(run_id, DataClass::InternalOnly),
        step_id: Classified::new(step_id, DataClass::InternalOnly),
        tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
        capability_id: Classified::new(capability_id, DataClass::InternalOnly),
        kind: Classified::new(kind, OperationalDataClass::Audit),
        fields: Classified::new(fields, OperationalDataClass::Audit),
        data_classes_touched: Classified::new(data_classes_touched, DataClass::InternalOnly),
        data_class: Classified::new(data_class, DataClass::InternalOnly),
        prev_hash: Classified::new(prev_hash, OperationalDataClass::Audit),
        hash: Classified::new(hash, OperationalDataClass::Audit),
        timestamp_epoch_seconds: Classified::new(
            timestamp_epoch_seconds,
            OperationalDataClass::Audit,
        ),
        schema_version: Classified::new(schema_version, DataClass::InternalOnly),
    })
}

fn encode_fields(fields: &BTreeMap<String, String>) -> String {
    let mut encoded = fields.len().to_string();
    encoded.push('|');
    for (key, value) in fields {
        encoded.push_str(&encode_str(key));
        encoded.push_str(&encode_str(value));
    }
    encoded
}

fn decode_fields(value: &str) -> Result<BTreeMap<String, String>, FileEvidenceStoreError> {
    let Some((count_text, mut rest)) = value.split_once('|') else {
        return Err(FileEvidenceStoreError::MalformedRecord);
    };
    let count = count_text
        .parse::<usize>()
        .map_err(|_| FileEvidenceStoreError::MalformedRecord)?;
    let mut fields = BTreeMap::new();
    for _ in 0..count {
        let key = take_len_prefixed(&mut rest)?;
        let field_value = take_len_prefixed(&mut rest)?;
        fields.insert(key, field_value);
    }
    if !rest.is_empty() {
        return Err(FileEvidenceStoreError::MalformedRecord);
    }
    Ok(fields)
}

fn encode_str(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn take_len_prefixed_then_separator(input: &mut &str) -> Result<String, FileEvidenceStoreError> {
    let value = take_len_prefixed(input)?;
    take_separator(input)?;
    Ok(value)
}

fn take_len_prefixed(input: &mut &str) -> Result<String, FileEvidenceStoreError> {
    let Some((len_text, rest)) = input.split_once(':') else {
        return Err(FileEvidenceStoreError::MalformedRecord);
    };
    let len = len_text
        .parse::<usize>()
        .map_err(|_| FileEvidenceStoreError::MalformedRecord)?;
    if rest.len() < len {
        return Err(FileEvidenceStoreError::MalformedRecord);
    }
    let (value, remainder) = rest.split_at(len);
    *input = remainder;
    Ok(value.to_string())
}

fn take_separator(input: &mut &str) -> Result<(), FileEvidenceStoreError> {
    if let Some(rest) = input.strip_prefix('|') {
        *input = rest;
        Ok(())
    } else {
        Err(FileEvidenceStoreError::MalformedRecord)
    }
}

fn take_until_separator<'a>(input: &mut &'a str) -> Result<&'a str, FileEvidenceStoreError> {
    let Some((field, rest)) = input.split_once('|') else {
        return Err(FileEvidenceStoreError::MalformedRecord);
    };
    *input = rest;
    Ok(field)
}

fn encode_kind(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::CapabilityInvocation => "CapabilityInvocation",
        EvidenceKind::ToolCall => "ToolCall",
        EvidenceKind::ProviderCall => "ProviderCall",
        EvidenceKind::DataFlow => "DataFlow",
        EvidenceKind::AutonomyDecision => "AutonomyDecision",
        EvidenceKind::ConsentCheck => "ConsentCheck",
    }
}

fn decode_kind(value: &str) -> Result<EvidenceKind, FileEvidenceStoreError> {
    match value {
        "CapabilityInvocation" => Ok(EvidenceKind::CapabilityInvocation),
        "ToolCall" => Ok(EvidenceKind::ToolCall),
        "ProviderCall" => Ok(EvidenceKind::ProviderCall),
        "DataFlow" => Ok(EvidenceKind::DataFlow),
        "AutonomyDecision" => Ok(EvidenceKind::AutonomyDecision),
        "ConsentCheck" => Ok(EvidenceKind::ConsentCheck),
        _ => Err(FileEvidenceStoreError::MalformedRecord),
    }
}

fn encode_data_classes(data_classes: &[PrivacyDataClass]) -> String {
    data_classes
        .iter()
        .map(|data_class| encode_data_class(data_class.data_class()))
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_data_classes(value: &str) -> Result<Vec<DataClass>, FileEvidenceStoreError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value.split(',').map(decode_data_class).collect()
}

fn encode_data_class(data_class: DataClass) -> &'static str {
    data_class.pascal_label()
}

fn decode_data_class(value: &str) -> Result<DataClass, FileEvidenceStoreError> {
    parse_data_class_pascal_label(value).ok_or(FileEvidenceStoreError::MalformedRecord)
}

fn map_io_error(error: std::io::Error) -> FileEvidenceStoreError {
    FileEvidenceStoreError::Io(error.to_string())
}
