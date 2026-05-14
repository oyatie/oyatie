//! File-backed eventing outbox adapter.
//!
//! This adapter gives the dev/runtime surface an append-only durability seam for
//! the kernel outbox without pulling a database driver into the kernel layer.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use oya_data_boundary_kernel::{Classified, DataClass};
use oya_eventing_domain::{EventingError, Outbox, OutboxRecord};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileOutboxStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileOutboxStoreError {
    Io(String),
    MalformedRecord,
    OutboxDiverged,
    InvalidOutboxHistory,
}

impl FileOutboxStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Outbox, FileOutboxStoreError> {
        if !self.path.exists() {
            return Ok(Outbox::default());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        let records = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_record)
            .collect::<Result<Vec<_>, _>>()?;
        Outbox::from_records(records).map_err(map_eventing_error)
    }

    pub fn append_outbox(&self, outbox: &Outbox) -> Result<usize, FileOutboxStoreError> {
        let persisted = self.load()?;
        let persisted_records = persisted.records();
        let requested_records = outbox.records();
        if persisted_records.len() > requested_records.len()
            || persisted_records != &requested_records[..persisted_records.len()]
        {
            return Err(FileOutboxStoreError::OutboxDiverged);
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

fn encode_record(record: &OutboxRecord) -> String {
    format!(
        "v1|{}|{}|{}|{}|{}|{}",
        record.sequence,
        encode_str(&record.tenant_id),
        encode_str(&record.topic.value),
        encode_str(&record.idempotency_key.value),
        encode_str(&record.payload_ref.value),
        if record.published { "1" } else { "0" }
    )
}

fn decode_record(line: &str) -> Result<OutboxRecord, FileOutboxStoreError> {
    let mut input = line;
    input = input
        .strip_prefix("v1|")
        .ok_or(FileOutboxStoreError::MalformedRecord)?;
    let sequence = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileOutboxStoreError::MalformedRecord)?;
    let tenant_id = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let topic = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let idempotency_key = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let payload_ref = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let published = match input {
        "0" => false,
        "1" => true,
        _ => return Err(FileOutboxStoreError::MalformedRecord),
    };
    Ok(OutboxRecord {
        sequence,
        tenant_id,
        topic: Classified::new(topic, DataClass::InternalOnly),
        idempotency_key: Classified::new(idempotency_key, DataClass::InternalOnly),
        payload_ref: Classified::new(payload_ref, DataClass::InternalOnly),
        published,
    })
}

fn encode_str(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn take_len_prefixed(input: &mut &str) -> Result<String, FileOutboxStoreError> {
    let Some((len_text, rest)) = input.split_once(':') else {
        return Err(FileOutboxStoreError::MalformedRecord);
    };
    let len = len_text
        .parse::<usize>()
        .map_err(|_| FileOutboxStoreError::MalformedRecord)?;
    if rest.len() < len {
        return Err(FileOutboxStoreError::MalformedRecord);
    }
    let (value, remainder) = rest.split_at(len);
    *input = remainder;
    Ok(value.to_string())
}

fn take_separator(input: &mut &str) -> Result<(), FileOutboxStoreError> {
    if let Some(rest) = input.strip_prefix('|') {
        *input = rest;
        Ok(())
    } else {
        Err(FileOutboxStoreError::MalformedRecord)
    }
}

fn take_until_separator<'a>(input: &mut &'a str) -> Result<&'a str, FileOutboxStoreError> {
    let Some((field, rest)) = input.split_once('|') else {
        return Err(FileOutboxStoreError::MalformedRecord);
    };
    *input = rest;
    Ok(field)
}

fn map_eventing_error(error: EventingError) -> FileOutboxStoreError {
    match error {
        EventingError::InvalidOutboxHistory => FileOutboxStoreError::InvalidOutboxHistory,
        EventingError::EmptyTopic
        | EventingError::EmptyIdempotencyKey
        | EventingError::EmptyPayloadRef
        | EventingError::OutboxRecordNotFound => FileOutboxStoreError::MalformedRecord,
    }
}

fn map_io_error(error: std::io::Error) -> FileOutboxStoreError {
    FileOutboxStoreError::Io(error.to_string())
}
