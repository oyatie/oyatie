//! Secrets kernel.
//!
//! Pure value objects and in-memory invariants for the SecretProvider substrate.
//! Real OpenBao/HSM integrations belong in adapter/runtime crates; this kernel
//! never performs I/O and never prints secret material.

use std::collections::BTreeMap;
use std::fmt;

use oya_data_boundary_kernel::{Classified, DataClass, DataClassification, OperationalDataClass};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SecretRef {
    pub tenant_id: Classified<String>,
    pub capability_id: Classified<String>,
    pub name: Classified<String>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct SecretMaterial {
    bytes: Classified<Vec<u8>>,
    fingerprint: Classified<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecretStatus {
    Active,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretVersion {
    pub secret_ref: SecretRef,
    pub version: Classified<u64>,
    pub previous_version: Classified<Option<u64>>,
    pub expires_at_epoch_seconds: Classified<Option<u64>>,
    pub status: SecretStatus,
    pub material: SecretMaterial,
}

#[derive(Clone, Eq, PartialEq)]
pub struct SecretLease {
    secret_ref: SecretRef,
    version: Classified<u64>,
    material: SecretMaterial,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecretError {
    InvalidTenantId,
    InvalidCapabilityId,
    InvalidSecretName,
    EmptySecretMaterial,
    SecretNotFound,
    SecretExpired,
    SecretRevoked,
    InvalidSecretHistory,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SecretVault {
    records: Vec<SecretVersion>,
    latest_by_ref: BTreeMap<SecretRef, usize>,
}

impl SecretRef {
    pub fn new(
        tenant_id: String,
        capability_id: String,
        name: String,
    ) -> Result<Self, SecretError> {
        if !tenant_id.starts_with("ten_") {
            return Err(SecretError::InvalidTenantId);
        }
        if !capability_id.starts_with("cap.") {
            return Err(SecretError::InvalidCapabilityId);
        }
        if name.trim().is_empty() || name.contains('/') || name.contains('|') {
            return Err(SecretError::InvalidSecretName);
        }
        Ok(Self {
            tenant_id: Classified::new(tenant_id, DataClass::InternalOnly),
            capability_id: Classified::new(capability_id, DataClass::InternalOnly),
            name: Classified::new(name, OperationalDataClass::Secret),
        })
    }
}

impl SecretMaterial {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SecretError> {
        Self::try_from(bytes)
    }

    pub fn classification(&self) -> DataClassification {
        self.bytes.data_class
    }

    /// Legacy data-use compatibility label for call sites that still traffic
    /// in `DataClass` payloads while field metadata moves to
    /// `DataClassification`. The source of truth remains
    /// [`Self::classification`].
    pub fn legacy_data_class(&self) -> DataClass {
        self.classification().compatibility_data_class()
    }

    #[deprecated(
        note = "use classification for canonical typed access or legacy_data_class for the compatibility projection"
    )]
    pub fn data_class(&self) -> DataClass {
        self.legacy_data_class()
    }

    pub fn expose_for_provider(&self) -> &[u8] {
        &self.bytes.value
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint.value
    }
}

impl TryFrom<Vec<u8>> for SecretMaterial {
    type Error = SecretError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            return Err(SecretError::EmptySecretMaterial);
        }
        let fingerprint = material_fingerprint(&bytes);
        Ok(Self {
            bytes: Classified::new(bytes, OperationalDataClass::Secret),
            fingerprint: Classified::new(fingerprint, DataClass::InternalOnly),
        })
    }
}

impl fmt::Debug for SecretMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretMaterial")
            .field("bytes", &"REDACTED")
            .field("fingerprint", &self.fingerprint.value)
            .finish()
    }
}

impl SecretLease {
    pub fn secret_ref(&self) -> &SecretRef {
        &self.secret_ref
    }

    pub fn version(&self) -> u64 {
        self.version.value
    }

    pub fn expose_for_provider(&self) -> &[u8] {
        self.material.expose_for_provider()
    }
}

impl fmt::Debug for SecretLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretLease")
            .field("secret_ref", &self.secret_ref)
            .field("version", &self.version.value)
            .field("material", &"REDACTED")
            .finish()
    }
}

impl SecretVault {
    pub fn from_records(records: Vec<SecretVersion>) -> Result<Self, SecretError> {
        let mut vault = Self::default();
        for record in records {
            vault.insert_existing(record)?;
        }
        Ok(vault)
    }

    pub fn put(
        &mut self,
        secret_ref: SecretRef,
        material: SecretMaterial,
        expires_at_epoch_seconds: Option<u64>,
    ) -> Result<SecretVersion, SecretError> {
        if self.latest_by_ref.contains_key(&secret_ref) {
            return self.rotate(&secret_ref, material, expires_at_epoch_seconds);
        }
        let version = SecretVersion {
            secret_ref: secret_ref.clone(),
            version: Classified::new(1, DataClass::InternalOnly),
            previous_version: Classified::new(None, DataClass::InternalOnly),
            expires_at_epoch_seconds: Classified::new(
                expires_at_epoch_seconds,
                DataClass::InternalOnly,
            ),
            status: SecretStatus::Active,
            material,
        };
        self.latest_by_ref.insert(secret_ref, self.records.len());
        self.records.push(version.clone());
        Ok(version)
    }

    pub fn rotate(
        &mut self,
        secret_ref: &SecretRef,
        material: SecretMaterial,
        expires_at_epoch_seconds: Option<u64>,
    ) -> Result<SecretVersion, SecretError> {
        let current = self.latest(secret_ref)?.clone();
        let version = SecretVersion {
            secret_ref: secret_ref.clone(),
            version: Classified::new(current.version.value + 1, DataClass::InternalOnly),
            previous_version: Classified::new(Some(current.version.value), DataClass::InternalOnly),
            expires_at_epoch_seconds: Classified::new(
                expires_at_epoch_seconds,
                DataClass::InternalOnly,
            ),
            status: SecretStatus::Active,
            material,
        };
        self.latest_by_ref
            .insert(secret_ref.clone(), self.records.len());
        self.records.push(version.clone());
        Ok(version)
    }

    pub fn revoke(&mut self, secret_ref: &SecretRef) -> Result<SecretVersion, SecretError> {
        let mut revoked = self.latest(secret_ref)?.clone();
        revoked.version = Classified::new(revoked.version.value + 1, DataClass::InternalOnly);
        revoked.previous_version =
            Classified::new(Some(revoked.version.value - 1), DataClass::InternalOnly);
        revoked.status = SecretStatus::Revoked;
        self.latest_by_ref
            .insert(secret_ref.clone(), self.records.len());
        self.records.push(revoked.clone());
        Ok(revoked)
    }

    pub fn get(
        &self,
        secret_ref: &SecretRef,
        now_epoch_seconds: u64,
    ) -> Result<SecretLease, SecretError> {
        let record = self.latest(secret_ref)?;
        if record.status == SecretStatus::Revoked {
            return Err(SecretError::SecretRevoked);
        }
        if record
            .expires_at_epoch_seconds
            .value
            .is_some_and(|expires_at| now_epoch_seconds >= expires_at)
        {
            return Err(SecretError::SecretExpired);
        }
        Ok(SecretLease {
            secret_ref: record.secret_ref.clone(),
            version: record.version.clone(),
            material: record.material.clone(),
        })
    }

    pub fn records(&self) -> &[SecretVersion] {
        &self.records
    }

    fn latest(&self, secret_ref: &SecretRef) -> Result<&SecretVersion, SecretError> {
        self.latest_by_ref
            .get(secret_ref)
            .and_then(|index| self.records.get(*index))
            .ok_or(SecretError::SecretNotFound)
    }

    fn insert_existing(&mut self, record: SecretVersion) -> Result<(), SecretError> {
        if record.version.value == 0 {
            return Err(SecretError::InvalidSecretHistory);
        }
        match self.latest_by_ref.get(&record.secret_ref).copied() {
            Some(index) => {
                let latest = self
                    .records
                    .get(index)
                    .ok_or(SecretError::InvalidSecretHistory)?;
                if record.previous_version.value != Some(latest.version.value)
                    || record.version.value != latest.version.value + 1
                {
                    return Err(SecretError::InvalidSecretHistory);
                }
            }
            None => {
                if record.version.value != 1 || record.previous_version.value.is_some() {
                    return Err(SecretError::InvalidSecretHistory);
                }
            }
        }
        self.latest_by_ref
            .insert(record.secret_ref.clone(), self.records.len());
        self.records.push(record);
        Ok(())
    }
}

fn material_fingerprint(bytes: &[u8]) -> String {
    let mut state = 0xcbf29ce484222325_u64;
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{state:016x}")
}
