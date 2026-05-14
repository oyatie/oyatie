//! File-backed audit ledger adapter.
//!
//! The adapter persists the audit chain as append-only text records and replays
//! them through the kernel verifier before returning a chain to callers.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use oya_audit_chain_domain::{AuditChain, AuditEvent, Plane};
use oya_data_boundary_kernel::{
    DataClass, Purpose, parse_data_class_pascal_label, parse_purpose_pascal_label,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileAuditLedger {
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileAuditLedgerError {
    Io(String),
    MalformedRecord,
    ChainDiverged,
    InvalidChain,
}

impl FileAuditLedger {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<AuditChain, FileAuditLedgerError> {
        if !self.path.exists() {
            return Ok(AuditChain::default());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        let events = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_event)
            .collect::<Result<Vec<_>, _>>()?;
        AuditChain::from_events(events).map_err(|_| FileAuditLedgerError::InvalidChain)
    }

    pub fn append_chain(&self, chain: &AuditChain) -> Result<usize, FileAuditLedgerError> {
        let persisted = self.load()?;
        let persisted_events = persisted.events();
        let requested_events = chain.events();
        if persisted_events.len() > requested_events.len()
            || persisted_events != &requested_events[..persisted_events.len()]
        {
            return Err(FileAuditLedgerError::ChainDiverged);
        }

        if requested_events.len() == persisted_events.len() {
            return Ok(0);
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(map_io_error)?;
        for event in &requested_events[persisted_events.len()..] {
            writeln!(file, "{}", encode_event(event)).map_err(map_io_error)?;
        }
        Ok(requested_events.len() - persisted_events.len())
    }
}

fn encode_event(event: &AuditEvent) -> String {
    format!(
        "v1|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        event.sequence,
        encode_str(&event.tenant_id),
        encode_str(&event.surface),
        encode_plane(event.plane),
        encode_purpose(event.purpose),
        encode_data_classes(&event.data_classes),
        encode_str(&event.decision),
        encode_str(&event.previous_hash),
        encode_str(&event.hash)
    )
}

fn decode_event(line: &str) -> Result<AuditEvent, FileAuditLedgerError> {
    let mut input = line;
    input = input
        .strip_prefix("v1|")
        .ok_or(FileAuditLedgerError::MalformedRecord)?;
    let sequence = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileAuditLedgerError::MalformedRecord)?;
    let tenant_id = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let surface = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let plane = decode_plane(take_until_separator(&mut input)?)?;
    let purpose = decode_purpose(take_until_separator(&mut input)?)?;
    let data_classes = decode_data_classes(take_until_separator(&mut input)?)?;
    let decision = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let previous_hash = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let hash = take_len_prefixed(&mut input)?;
    if !input.is_empty() {
        return Err(FileAuditLedgerError::MalformedRecord);
    }
    Ok(AuditEvent {
        sequence,
        tenant_id,
        surface,
        plane,
        purpose,
        data_classes,
        decision,
        previous_hash,
        hash,
    })
}

fn encode_str(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn take_len_prefixed(input: &mut &str) -> Result<String, FileAuditLedgerError> {
    let Some((len_text, rest)) = input.split_once(':') else {
        return Err(FileAuditLedgerError::MalformedRecord);
    };
    let len = len_text
        .parse::<usize>()
        .map_err(|_| FileAuditLedgerError::MalformedRecord)?;
    if rest.len() < len {
        return Err(FileAuditLedgerError::MalformedRecord);
    }
    let (value, remainder) = rest.split_at(len);
    *input = remainder;
    Ok(value.to_string())
}

fn take_separator(input: &mut &str) -> Result<(), FileAuditLedgerError> {
    if let Some(rest) = input.strip_prefix('|') {
        *input = rest;
        Ok(())
    } else {
        Err(FileAuditLedgerError::MalformedRecord)
    }
}

fn take_until_separator<'a>(input: &mut &'a str) -> Result<&'a str, FileAuditLedgerError> {
    let Some((field, rest)) = input.split_once('|') else {
        return Err(FileAuditLedgerError::MalformedRecord);
    };
    *input = rest;
    Ok(field)
}

fn encode_plane(plane: Plane) -> &'static str {
    match plane {
        Plane::Control => "Control",
        Plane::Data => "Data",
        Plane::Audit => "Audit",
        Plane::Analytics => "Analytics",
    }
}

fn decode_plane(value: &str) -> Result<Plane, FileAuditLedgerError> {
    match value {
        "Control" => Ok(Plane::Control),
        "Data" => Ok(Plane::Data),
        "Audit" => Ok(Plane::Audit),
        "Analytics" => Ok(Plane::Analytics),
        _ => Err(FileAuditLedgerError::MalformedRecord),
    }
}

fn encode_purpose(purpose: Purpose) -> &'static str {
    purpose.pascal_label()
}

fn decode_purpose(value: &str) -> Result<Purpose, FileAuditLedgerError> {
    parse_purpose_pascal_label(value).ok_or(FileAuditLedgerError::MalformedRecord)
}

fn encode_data_classes(data_classes: &[DataClass]) -> String {
    data_classes
        .iter()
        .map(|data_class| encode_data_class(*data_class))
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_data_classes(value: &str) -> Result<Vec<DataClass>, FileAuditLedgerError> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value.split(',').map(decode_data_class).collect()
}

fn encode_data_class(data_class: DataClass) -> &'static str {
    data_class.pascal_label()
}

fn decode_data_class(value: &str) -> Result<DataClass, FileAuditLedgerError> {
    parse_data_class_pascal_label(value).ok_or(FileAuditLedgerError::MalformedRecord)
}

fn map_io_error(error: std::io::Error) -> FileAuditLedgerError {
    FileAuditLedgerError::Io(error.to_string())
}
