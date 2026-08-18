//! File-backed eventing outbox adapter.
//!
//! This adapter gives the dev/runtime surface an append-only durability seam for
//! the kernel outbox without pulling a database driver into the kernel layer.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use messaging_domain::{EventingError, Outbox, OutboxRecord};
use oya_data_boundary_kernel::{Classified, DataClass};

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

#[derive(Clone, Debug, Eq, PartialEq)]
enum FileOutboxLedgerEvent {
    Record(OutboxRecord),
    Published {
        tenant_id: String, // data_class: INTERNAL_ONLY
        sequence: u64,     // data_class: INTERNAL_ONLY
    },
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
        let events = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_event)
            .collect::<Result<Vec<_>, _>>()?;
        let records = replay_events(events)?;
        Outbox::from_records(records).map_err(map_eventing_error)
    }

    pub fn append_outbox(&self, outbox: &Outbox) -> Result<usize, FileOutboxStoreError> {
        let persisted = self.load()?;
        let persisted_records = persisted.records();
        let requested_records = outbox.records();
        if persisted_records.len() > requested_records.len()
            || !records_share_append_only_prefix(
                persisted_records,
                &requested_records[..persisted_records.len()],
            )
        {
            return Err(FileOutboxStoreError::OutboxDiverged);
        }

        let mut events_to_append = Vec::new();
        for (persisted, requested) in persisted_records.iter().zip(requested_records) {
            if persisted.published && !requested.published {
                return Err(FileOutboxStoreError::OutboxDiverged);
            }
            if !persisted.published && requested.published {
                events_to_append.push(encode_published_event(requested));
            }
        }

        for record in &requested_records[persisted_records.len()..] {
            let mut record_event = record.clone();
            record_event.published = false;
            events_to_append.push(encode_record(&record_event));
            if record.published {
                events_to_append.push(encode_published_event(record));
            }
        }

        if events_to_append.is_empty() {
            return Ok(0);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(map_io_error)?;
        for event in &events_to_append {
            writeln!(file, "{event}").map_err(map_io_error)?;
        }
        Ok(events_to_append.len())
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

fn encode_published_event(record: &OutboxRecord) -> String {
    format!(
        "v1-published|{}|{}",
        record.sequence,
        encode_str(&record.tenant_id)
    )
}

fn decode_event(line: &str) -> Result<FileOutboxLedgerEvent, FileOutboxStoreError> {
    if line.starts_with("v1|") {
        return decode_record(line).map(FileOutboxLedgerEvent::Record);
    }
    decode_published_event(line)
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

fn decode_published_event(line: &str) -> Result<FileOutboxLedgerEvent, FileOutboxStoreError> {
    let mut input = line;
    input = input
        .strip_prefix("v1-published|")
        .ok_or(FileOutboxStoreError::MalformedRecord)?;
    let sequence = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileOutboxStoreError::MalformedRecord)?;
    let tenant_id = take_len_prefixed(&mut input)?;
    if !input.is_empty() {
        return Err(FileOutboxStoreError::MalformedRecord);
    }
    Ok(FileOutboxLedgerEvent::Published {
        tenant_id,
        sequence,
    })
}

fn replay_events(
    events: Vec<FileOutboxLedgerEvent>,
) -> Result<Vec<OutboxRecord>, FileOutboxStoreError> {
    let mut records = Vec::new();
    for event in events {
        match event {
            FileOutboxLedgerEvent::Record(record) => {
                if record.sequence != records.len() as u64 {
                    return Err(FileOutboxStoreError::InvalidOutboxHistory);
                }
                records.push(record);
            }
            FileOutboxLedgerEvent::Published {
                tenant_id,
                sequence,
            } => {
                let index = usize::try_from(sequence)
                    .map_err(|_| FileOutboxStoreError::InvalidOutboxHistory)?;
                let record = records
                    .get_mut(index)
                    .filter(|record| record.sequence == sequence && record.tenant_id == tenant_id)
                    .ok_or(FileOutboxStoreError::InvalidOutboxHistory)?;
                if record.published {
                    return Err(FileOutboxStoreError::InvalidOutboxHistory);
                }
                record.published = true;
            }
        }
    }
    Ok(records)
}

fn records_share_append_only_prefix(
    persisted_records: &[OutboxRecord],
    requested_records: &[OutboxRecord],
) -> bool {
    persisted_records
        .iter()
        .zip(requested_records)
        .all(|(persisted, requested)| same_record_identity(persisted, requested))
}

fn same_record_identity(persisted: &OutboxRecord, requested: &OutboxRecord) -> bool {
    persisted.sequence == requested.sequence
        && persisted.tenant_id == requested.tenant_id
        && persisted.topic == requested.topic
        && persisted.idempotency_key == requested.idempotency_key
        && persisted.payload_ref == requested.payload_ref
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
    let Some(value) = rest.get(..len) else {
        return Err(FileOutboxStoreError::MalformedRecord);
    };
    let Some(remainder) = rest.get(len..) else {
        return Err(FileOutboxStoreError::MalformedRecord);
    };
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
        | EventingError::EmptyTopicAxis
        | EventingError::EmptyTopicDescription
        | EventingError::InvalidTopicName
        | EventingError::DuplicateTopic
        | EventingError::TopicNotFound
        | EventingError::EmptyIdempotencyKey
        | EventingError::EmptyPayloadRef
        | EventingError::IdempotencyReplayMismatch
        | EventingError::OutboxRecordNotFound => FileOutboxStoreError::MalformedRecord,
    }
}

fn map_io_error(error: std::io::Error) -> FileOutboxStoreError {
    FileOutboxStoreError::Io(error.to_string())
}
