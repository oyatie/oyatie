//! File-backed secrets adapter.
//!
//! This is a dev/runtime durability seam for SecretProvider metadata. It
//! intentionally does not persist provider bytes or reversible encodings of
//! `SecretMaterial`; production OpenBao/HSM adapters own recoverable material.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use oya_secrets_domain::{SecretError, SecretRef, SecretStatus, SecretVault, SecretVersion};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSecretStore {
    path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSecretMetadata {
    pub tenant_id: String,                     // data_class: INTERNAL_ONLY
    pub capability_id: String,                 // data_class: INTERNAL_ONLY
    pub name: String,                          // data_class: INTERNAL_ONLY
    pub version: u64,                          // data_class: INTERNAL_ONLY
    pub previous_version: Option<u64>,         // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: Option<u64>, // data_class: INTERNAL_ONLY
    pub status: SecretStatus,                  // data_class: INTERNAL_ONLY
    pub fingerprint: String,                   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileSecretStoreError {
    Io(String),
    MalformedRecord,
    SecretHistoryDiverged,
    InvalidSecretHistory,
    SecretMaterialUnavailable,
}

impl FileSecretMetadata {
    pub fn from_secret_version(record: &SecretVersion) -> Self {
        Self {
            tenant_id: record.secret_ref.tenant_id.value.clone(),
            capability_id: record.secret_ref.capability_id.value.clone(),
            name: record.secret_ref.name.value.clone(),
            version: record.version.value,
            previous_version: record.previous_version.value,
            expires_at_epoch_seconds: record.expires_at_epoch_seconds.value,
            status: record.status,
            fingerprint: record.material.fingerprint().to_string(),
        }
    }

    fn secret_ref(&self) -> Result<SecretRef, FileSecretStoreError> {
        SecretRef::new(
            self.tenant_id.clone(),
            self.capability_id.clone(),
            self.name.clone(),
        )
        .map_err(map_secret_error)
    }
}

impl FileSecretStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<SecretVault, FileSecretStoreError> {
        let metadata = self.load_metadata()?;
        if metadata.is_empty() {
            Ok(SecretVault::default())
        } else {
            Err(FileSecretStoreError::SecretMaterialUnavailable)
        }
    }

    pub fn load_metadata(&self) -> Result<Vec<FileSecretMetadata>, FileSecretStoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let contents = fs::read_to_string(&self.path).map_err(map_io_error)?;
        let records = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(decode_metadata_record)
            .collect::<Result<Vec<_>, _>>()?;
        validate_metadata_history(&records)?;
        Ok(records)
    }

    pub fn append_vault(&self, vault: &SecretVault) -> Result<usize, FileSecretStoreError> {
        let persisted_records = self.load_metadata()?;
        let requested_records = metadata_for_records(vault.records());
        if persisted_records.len() > requested_records.len()
            || persisted_records != requested_records[..persisted_records.len()]
        {
            return Err(FileSecretStoreError::SecretHistoryDiverged);
        }
        if persisted_records.len() == requested_records.len() {
            return Ok(0);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(map_io_error)?;
        for record in &requested_records[persisted_records.len()..] {
            writeln!(file, "{}", encode_metadata_record(record)).map_err(map_io_error)?;
        }
        Ok(requested_records.len() - persisted_records.len())
    }

    pub fn matches_vault_metadata(
        &self,
        vault: &SecretVault,
    ) -> Result<bool, FileSecretStoreError> {
        Ok(self.load_metadata()? == metadata_for_records(vault.records()))
    }
}

fn metadata_for_records(records: &[SecretVersion]) -> Vec<FileSecretMetadata> {
    records
        .iter()
        .map(FileSecretMetadata::from_secret_version)
        .collect()
}

fn encode_metadata_record(record: &FileSecretMetadata) -> String {
    format!(
        "v2|{}|{}|{}|{}|{}|{}|{}|{}",
        encode_str(&record.tenant_id),
        encode_str(&record.capability_id),
        encode_str(&record.name),
        record.version,
        encode_option_u64(record.previous_version),
        encode_option_u64(record.expires_at_epoch_seconds),
        encode_status(record.status),
        encode_str(&record.fingerprint)
    )
}

fn decode_metadata_record(line: &str) -> Result<FileSecretMetadata, FileSecretStoreError> {
    if line.starts_with("v1|") {
        return Err(FileSecretStoreError::SecretMaterialUnavailable);
    }
    let mut input = line
        .strip_prefix("v2|")
        .ok_or(FileSecretStoreError::MalformedRecord)?;
    let tenant_id = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let capability_id = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let name = take_len_prefixed(&mut input)?;
    take_separator(&mut input)?;
    let version = take_until_separator(&mut input)?
        .parse::<u64>()
        .map_err(|_| FileSecretStoreError::MalformedRecord)?;
    let previous_version = decode_option_u64(take_until_separator(&mut input)?)?;
    let expires_at_epoch_seconds = decode_option_u64(take_until_separator(&mut input)?)?;
    let status = decode_status(take_until_separator(&mut input)?)?;
    let fingerprint = take_len_prefixed(&mut input)?;
    if !input.is_empty() || fingerprint.trim().is_empty() {
        return Err(FileSecretStoreError::MalformedRecord);
    }
    let record = FileSecretMetadata {
        tenant_id,
        capability_id,
        name,
        version,
        previous_version,
        expires_at_epoch_seconds,
        status,
        fingerprint,
    };
    record.secret_ref()?;
    Ok(record)
}

fn validate_metadata_history(records: &[FileSecretMetadata]) -> Result<(), FileSecretStoreError> {
    let mut latest_by_ref: BTreeMap<SecretRef, u64> = BTreeMap::new();
    for record in records {
        if record.version == 0 {
            return Err(FileSecretStoreError::InvalidSecretHistory);
        }
        let secret_ref = record.secret_ref()?;
        match latest_by_ref.get(&secret_ref).copied() {
            Some(latest_version) => {
                if record.previous_version != Some(latest_version)
                    || record.version != latest_version + 1
                {
                    return Err(FileSecretStoreError::InvalidSecretHistory);
                }
            }
            None => {
                if record.version != 1 || record.previous_version.is_some() {
                    return Err(FileSecretStoreError::InvalidSecretHistory);
                }
            }
        }
        latest_by_ref.insert(secret_ref, record.version);
    }
    Ok(())
}

fn encode_str(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn take_len_prefixed(input: &mut &str) -> Result<String, FileSecretStoreError> {
    let Some((len_text, rest)) = input.split_once(':') else {
        return Err(FileSecretStoreError::MalformedRecord);
    };
    let len = len_text
        .parse::<usize>()
        .map_err(|_| FileSecretStoreError::MalformedRecord)?;
    if rest.len() < len || !rest.is_char_boundary(len) {
        return Err(FileSecretStoreError::MalformedRecord);
    }
    let (value, remainder) = rest.split_at(len);
    *input = remainder;
    Ok(value.to_string())
}

fn take_separator(input: &mut &str) -> Result<(), FileSecretStoreError> {
    if let Some(rest) = input.strip_prefix('|') {
        *input = rest;
        Ok(())
    } else {
        Err(FileSecretStoreError::MalformedRecord)
    }
}

fn take_until_separator<'a>(input: &mut &'a str) -> Result<&'a str, FileSecretStoreError> {
    let Some((field, rest)) = input.split_once('|') else {
        return Err(FileSecretStoreError::MalformedRecord);
    };
    *input = rest;
    Ok(field)
}

fn encode_option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn decode_option_u64(value: &str) -> Result<Option<u64>, FileSecretStoreError> {
    if value == "none" {
        Ok(None)
    } else {
        value
            .parse::<u64>()
            .map(Some)
            .map_err(|_| FileSecretStoreError::MalformedRecord)
    }
}

fn encode_status(status: SecretStatus) -> &'static str {
    match status {
        SecretStatus::Active => "active",
        SecretStatus::Revoked => "revoked",
    }
}

fn decode_status(value: &str) -> Result<SecretStatus, FileSecretStoreError> {
    match value {
        "active" => Ok(SecretStatus::Active),
        "revoked" => Ok(SecretStatus::Revoked),
        _ => Err(FileSecretStoreError::MalformedRecord),
    }
}

fn map_secret_error(error: SecretError) -> FileSecretStoreError {
    match error {
        SecretError::InvalidSecretHistory => FileSecretStoreError::InvalidSecretHistory,
        SecretError::InvalidTenantId
        | SecretError::InvalidCapabilityId
        | SecretError::InvalidSecretName
        | SecretError::EmptySecretMaterial
        | SecretError::SecretNotFound
        | SecretError::SecretExpired
        | SecretError::SecretRevoked => FileSecretStoreError::MalformedRecord,
    }
}

fn map_io_error(error: std::io::Error) -> FileSecretStoreError {
    FileSecretStoreError::Io(error.to_string())
}
