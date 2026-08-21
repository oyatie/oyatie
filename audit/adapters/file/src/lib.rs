//! File-backed audit ledger adapter.
//!
//! The adapter persists the audit chain as append-only text records and replays
//! them through the kernel verifier before returning a chain to callers.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use audit_chain_domain::{
    AuditChain, AuditEvent, Ed25519Signature, Ed25519VerificationKeySet, MerkleRoot, Plane,
    TenantShardId,
};
use data_boundary_kernel::{
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
        self.load_events().and_then(|events| {
            AuditChain::from_events(events).map_err(|_| FileAuditLedgerError::InvalidChain)
        })
    }

    pub fn load_multi_tenant_shards(&self) -> Result<AuditChain, FileAuditLedgerError> {
        self.load_events().and_then(|events| {
            AuditChain::from_multi_tenant_shard_events(events)
                .map_err(|_| FileAuditLedgerError::InvalidChain)
        })
    }

    pub fn load_with_trusted_keys(
        &self,
        trusted_keys: &Ed25519VerificationKeySet,
    ) -> Result<AuditChain, FileAuditLedgerError> {
        self.load_events().and_then(|events| {
            AuditChain::from_signed_events(events, trusted_keys)
                .map_err(|_| FileAuditLedgerError::InvalidChain)
        })
    }

    fn load_events(&self) -> Result<Vec<AuditEvent>, FileAuditLedgerError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_event)
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn append_chain(&self, chain: &AuditChain) -> Result<usize, FileAuditLedgerError> {
        let persisted = if chain.is_multi_tenant_shards() {
            self.load_multi_tenant_shards()?
        } else {
            self.load()?
        };
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
    let (signature_key_id, signature_public_key, signature_hex) = event
        .ed25519_signature
        .as_ref()
        .map(|signature| {
            (
                signature.key_id.as_str(),
                signature.public_key_hex.as_str(),
                signature.signature_hex.as_str(),
            )
        })
        .unwrap_or(("", "", ""));
    format!(
        "v2|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        event.sequence,
        encode_str(event.tenant_shard.as_str()),
        encode_str(&event.tenant_id),
        encode_str(&event.surface),
        encode_plane(event.plane),
        encode_purpose(event.purpose),
        encode_data_classes(&event.data_classes),
        encode_str(&event.decision),
        encode_str(&event.previous_hash),
        encode_str(&event.hash),
        encode_str(event.merkle_root.as_str()),
        encode_str(signature_key_id),
        encode_str(signature_public_key),
        encode_str(signature_hex),
    )
}

fn decode_event(line: &str) -> Result<AuditEvent, FileAuditLedgerError> {
    if line.starts_with("v2|") {
        decode_v2_event(line)
    } else {
        Err(FileAuditLedgerError::MalformedRecord)
    }
}

fn decode_v2_event(line: &str) -> Result<AuditEvent, FileAuditLedgerError> {
    let mut input = line;
    input = input
        .strip_prefix("v2|")
        .ok_or(FileAuditLedgerError::MalformedRecord)?;
    let sequence = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileAuditLedgerError::MalformedRecord)?;
    let tenant_shard = TenantShardId::new(take_len_prefixed(&mut input)?)
        .map_err(|_| FileAuditLedgerError::MalformedRecord)?;
    take_separator(&mut input)?;
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
    take_separator(&mut input)?;
    let merkle_root = MerkleRoot {
        value: take_len_prefixed(&mut input)?,
    };
    take_separator(&mut input)?;
    let signature_key_id = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let signature_public_key = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let signature_hex = take_len_prefixed(&mut input)?;
    if !input.is_empty() {
        return Err(FileAuditLedgerError::MalformedRecord);
    }
    let ed25519_signature = if signature_key_id.is_empty()
        && signature_public_key.is_empty()
        && signature_hex.is_empty()
    {
        None
    } else {
        Some(Ed25519Signature {
            key_id: signature_key_id,
            public_key_hex: signature_public_key,
            signature_hex,
        })
    };
    Ok(AuditEvent {
        sequence,
        tenant_shard,
        tenant_id,
        surface,
        plane,
        purpose,
        data_classes,
        decision,
        previous_hash,
        hash,
        merkle_root,
        ed25519_signature,
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
    let value = rest
        .get(..len)
        .ok_or(FileAuditLedgerError::MalformedRecord)?;
    let remainder = rest
        .get(len..)
        .ok_or(FileAuditLedgerError::MalformedRecord)?;
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
