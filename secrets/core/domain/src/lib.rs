//! Secrets kernel.
//!
//! Pure value objects and in-memory invariants for the SecretProvider substrate.
//! Real OpenBao/HSM integrations belong in adapter/runtime crates; this kernel
//! never performs I/O and never prints secret material.

pub mod lease_lifecycle;
pub mod zeroizing;

pub use lease_lifecycle::{
    DynamicLease, LeaseError, LeaseId, LeasePolicy, LeaseRevocationEvent, LeaseState,
    MAX_LEASE_LIFETIME_SECONDS, MAX_LEASE_TTL_SECONDS, MIN_LEASE_TTL_SECONDS, RevocationReason,
};
pub use zeroizing::{VaultPath, VaultPathError, ZeroizingSecret};

use std::collections::BTreeMap;
use std::fmt;

use oya_data_boundary_kernel::{Classified, DataClass, DataClassification, OperationalDataClass};

const OPENBAO_SECRET_REFERENCE_PREFIX: &str = "openbao:secret/";
const CONFIG_SECRET_REFERENCE_PREFIX: &str = "${";
const CONFIG_SECRET_REFERENCE_SUFFIX: &str = "}";
pub const MAX_SECRET_REFERENCE_CACHE_TTL_SECONDS: u64 = 60;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecretProviderKind {
    OpenBao,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretReference {
    provider: SecretProviderKind,
    tenant_id: String,     // data_class: INTERNAL_ONLY
    region: String,        // data_class: PUBLIC
    cell_id: String,       // data_class: PUBLIC
    vault_path: VaultPath, // data_class: INTERNAL_ONLY
    version_label: String, // data_class: INTERNAL_ONLY
    evidence_ref: String,  // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretBootstrapRequest {
    pub tenant_id: String,                    // data_class: INTERNAL_ONLY
    pub region: String,                       // data_class: PUBLIC
    pub cell_id: String,                      // data_class: PUBLIC
    pub external_secret_store_ready: bool,    // data_class: INTERNAL_ONLY
    pub sealed_bootstrap_channel_ready: bool, // data_class: INTERNAL_ONLY
    pub plaintext_env_present: bool,          // data_class: INTERNAL_ONLY
    pub repo_secret_material_detected: bool,  // data_class: INTERNAL_ONLY
    pub evidence_ref: String,                 // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SecretBootstrapStatus {
    Allowed,
}

impl SecretBootstrapStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretBootstrapReceipt {
    pub tenant_id: String, // data_class: INTERNAL_ONLY
    pub region: String,    // data_class: PUBLIC
    pub cell_id: String,   // data_class: PUBLIC
    pub status: SecretBootstrapStatus,
    pub evidence_ref: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecretBootstrapError {
    InvalidTenantId,
    RegionEmpty,
    CellIdEmpty,
    InvalidVaultPath(VaultPathError),
    PathTenantMismatch,
    VersionLabelEmpty,
    EvidenceRefMissing,
    EvidenceRefLooksLikeSecret,
    ExternalSecretStoreUnavailable,
    SealedBootstrapChannelUnavailable,
    SecretMaterialInBootstrap,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretReferenceUri {
    provider: SecretProviderKind,
    path: String,         // data_class: INTERNAL_ONLY
    version: Option<u64>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecretReferenceUriError {
    MissingOpenBaoSecretPrefix,
    MissingConfigWrapper,
    EmptyPath,
    EmptySegment,
    TraversalSegment,
    InvalidSegmentCharacter,
    InvalidVersion,
    ZeroVersion,
    SecretMaterialLiteral,
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

impl SecretReference {
    pub fn openbao(
        tenant_id: impl Into<String>,
        region: impl Into<String>,
        cell_id: impl Into<String>,
        vault_path: impl Into<String>,
        version_label: impl Into<String>,
        evidence_ref: impl Into<String>,
    ) -> Result<Self, SecretBootstrapError> {
        let tenant_id = tenant_id.into();
        let region = region.into();
        let cell_id = cell_id.into();
        let vault_path = vault_path.into();
        let version_label = version_label.into();
        let evidence_ref = evidence_ref.into();
        validate_cloud_secret_boundary(&tenant_id, &region, &cell_id)?;
        if version_label.trim().is_empty() {
            return Err(SecretBootstrapError::VersionLabelEmpty);
        }
        validate_evidence_ref(&evidence_ref)?;
        let vault_path =
            VaultPath::new(vault_path).map_err(SecretBootstrapError::InvalidVaultPath)?;
        let tenant_root = format!("secret/data/t/{tenant_id}/");
        if !vault_path.as_str().starts_with(&tenant_root) {
            return Err(SecretBootstrapError::PathTenantMismatch);
        }
        Ok(Self {
            provider: SecretProviderKind::OpenBao,
            tenant_id,
            region,
            cell_id,
            vault_path,
            version_label,
            evidence_ref,
        })
    }

    pub const fn provider(&self) -> SecretProviderKind {
        self.provider
    }

    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn cell_id(&self) -> &str {
        &self.cell_id
    }

    pub fn vault_path(&self) -> &VaultPath {
        &self.vault_path
    }

    pub fn version_label(&self) -> &str {
        &self.version_label
    }

    pub fn evidence_ref(&self) -> &str {
        &self.evidence_ref
    }
}

impl SecretReferenceUri {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, SecretReferenceUriError> {
        let input = input.as_ref();
        if looks_like_secret_material(input) {
            return Err(SecretReferenceUriError::SecretMaterialLiteral);
        }
        let reference = input
            .strip_prefix(OPENBAO_SECRET_REFERENCE_PREFIX)
            .ok_or(SecretReferenceUriError::MissingOpenBaoSecretPrefix)?;
        let (path, version) = parse_reference_path_and_version(reference)?;
        validate_reference_path(path)?;
        Ok(Self {
            provider: SecretProviderKind::OpenBao,
            path: path.to_string(),
            version,
        })
    }

    pub fn parse_config_reference(input: impl AsRef<str>) -> Result<Self, SecretReferenceUriError> {
        let input = input.as_ref();
        let Some(without_prefix) = input.strip_prefix(CONFIG_SECRET_REFERENCE_PREFIX) else {
            return Err(SecretReferenceUriError::MissingConfigWrapper);
        };
        let Some(reference) = without_prefix.strip_suffix(CONFIG_SECRET_REFERENCE_SUFFIX) else {
            return Err(SecretReferenceUriError::MissingConfigWrapper);
        };
        Self::parse(reference)
    }

    pub const fn provider(&self) -> SecretProviderKind {
        self.provider
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub const fn version(&self) -> Option<u64> {
        self.version
    }

    pub fn path_segments(&self) -> impl Iterator<Item = &str> {
        self.path.split('/')
    }

    pub fn normalized_uri(&self) -> String {
        match self.version {
            Some(version) => {
                format!("{OPENBAO_SECRET_REFERENCE_PREFIX}{}@v{version}", self.path)
            }
            None => format!("{OPENBAO_SECRET_REFERENCE_PREFIX}{}", self.path),
        }
    }

    pub fn normalized_config_reference(&self) -> String {
        format!(
            "{CONFIG_SECRET_REFERENCE_PREFIX}{}{CONFIG_SECRET_REFERENCE_SUFFIX}",
            self.normalized_uri()
        )
    }
}

impl fmt::Display for SecretReferenceUriError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingOpenBaoSecretPrefix => {
                "secret reference must start with 'openbao:secret/'"
            }
            Self::MissingConfigWrapper => {
                "config secret reference must be wrapped as '${openbao:secret/<path>}'"
            }
            Self::EmptyPath => "secret reference path must not be empty",
            Self::EmptySegment => "secret reference path segments must not be empty",
            Self::TraversalSegment => "secret reference path must not contain '..' traversal",
            Self::InvalidSegmentCharacter => {
                "secret reference path contains a character outside the contract alphabet"
            }
            Self::InvalidVersion => {
                "secret reference version must use the '@v<positive-integer>' suffix"
            }
            Self::ZeroVersion => "secret reference version must be greater than zero",
            Self::SecretMaterialLiteral => {
                "secret reference must not contain raw secret material or credential-shaped text"
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SecretReferenceUriError {}

pub const fn clamp_secret_reference_cache_ttl_seconds(requested_seconds: u64) -> u64 {
    if requested_seconds > MAX_SECRET_REFERENCE_CACHE_TTL_SECONDS {
        MAX_SECRET_REFERENCE_CACHE_TTL_SECONDS
    } else {
        requested_seconds
    }
}

pub fn evaluate_secret_bootstrap(
    request: SecretBootstrapRequest,
) -> Result<SecretBootstrapReceipt, SecretBootstrapError> {
    validate_cloud_secret_boundary(&request.tenant_id, &request.region, &request.cell_id)?;
    validate_evidence_ref(&request.evidence_ref)?;
    if request.plaintext_env_present || request.repo_secret_material_detected {
        return Err(SecretBootstrapError::SecretMaterialInBootstrap);
    }
    if !request.external_secret_store_ready {
        return Err(SecretBootstrapError::ExternalSecretStoreUnavailable);
    }
    if !request.sealed_bootstrap_channel_ready {
        return Err(SecretBootstrapError::SealedBootstrapChannelUnavailable);
    }
    Ok(SecretBootstrapReceipt {
        tenant_id: request.tenant_id,
        region: request.region,
        cell_id: request.cell_id,
        status: SecretBootstrapStatus::Allowed,
        evidence_ref: request.evidence_ref,
    })
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

fn parse_reference_path_and_version(
    reference: &str,
) -> Result<(&str, Option<u64>), SecretReferenceUriError> {
    if reference.is_empty() {
        return Err(SecretReferenceUriError::EmptyPath);
    }
    let Some((path, version_text)) = reference.rsplit_once('@') else {
        return Ok((reference, None));
    };
    if path.is_empty() {
        return Err(SecretReferenceUriError::EmptyPath);
    }
    let Some(digits) = version_text.strip_prefix('v') else {
        return Err(SecretReferenceUriError::InvalidVersion);
    };
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return Err(SecretReferenceUriError::InvalidVersion);
    }
    let version = digits
        .parse::<u64>()
        .map_err(|_| SecretReferenceUriError::InvalidVersion)?;
    if version == 0 {
        return Err(SecretReferenceUriError::ZeroVersion);
    }
    Ok((path, Some(version)))
}

fn validate_reference_path(path: &str) -> Result<(), SecretReferenceUriError> {
    if path.is_empty() {
        return Err(SecretReferenceUriError::EmptyPath);
    }
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(SecretReferenceUriError::EmptySegment);
        }
        if segment == ".." {
            return Err(SecretReferenceUriError::TraversalSegment);
        }
        if !segment.chars().all(is_reference_segment_character) {
            return Err(SecretReferenceUriError::InvalidSegmentCharacter);
        }
    }
    Ok(())
}

fn is_reference_segment_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ':')
}

fn validate_cloud_secret_boundary(
    tenant_id: &str,
    region: &str,
    cell_id: &str,
) -> Result<(), SecretBootstrapError> {
    if !tenant_id.starts_with("ten_") {
        return Err(SecretBootstrapError::InvalidTenantId);
    }
    if region.trim().is_empty() {
        return Err(SecretBootstrapError::RegionEmpty);
    }
    if cell_id.trim().is_empty() {
        return Err(SecretBootstrapError::CellIdEmpty);
    }
    Ok(())
}

fn validate_evidence_ref(evidence_ref: &str) -> Result<(), SecretBootstrapError> {
    if evidence_ref.trim().is_empty() {
        return Err(SecretBootstrapError::EvidenceRefMissing);
    }
    if looks_like_secret_material(evidence_ref) {
        return Err(SecretBootstrapError::EvidenceRefLooksLikeSecret);
    }
    Ok(())
}

fn looks_like_secret_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("sk-")
        || lower.contains("password=")
        || lower.contains("token=")
        || lower.contains("secret_material=")
        || lower.contains("-----begin")
}
