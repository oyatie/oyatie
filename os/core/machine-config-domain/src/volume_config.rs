//! Typed parser/accessors for Talos block volume config documents.
//!
//! This module is source-guided by upstream Talos
//! `pkg/machinery/config/types/block/{volume_config,user_volume_config,raw_volume_config,existing_volume_config,external_volume_config,swap_volume_config}.go`:
//! system `VolumeConfig` documents are keyed by `name` and currently cover
//! `STATE`, `EPHEMERAL`, and `IMAGECACHE`; user/raw/external/swap volume config
//! names are 1..=34 ASCII letters/digits/hyphens and derive block partition
//! labels/ids with their source prefixes (`u-`, `r-`, `x-`, `s-`). Existing
//! volumes use the source `e-` id prefix but upstream does not impose the
//! partition-label length bound on their names. Fields the current Rust block
//! declaration seam cannot project losslessly are parsed and preserved here so
//! projection code can reject them explicitly.

use crate::container::Config;
use crate::yaml::{self, Yaml};
use std::collections::{BTreeMap, BTreeSet};
use os_kernel::error::{Error, Result};
use os_kernel::{validate_disk_locator_bool_expression, validate_volume_locator_bool_expression};

/// Canonical Talos system-volume document kind.
pub const VOLUME_CONFIG_KIND: &str = "VolumeConfig";
/// Canonical Talos user-volume document kind.
pub const USER_VOLUME_CONFIG_KIND: &str = "UserVolumeConfig";
/// Canonical Talos raw-volume document kind.
pub const RAW_VOLUME_CONFIG_KIND: &str = "RawVolumeConfig";
/// Canonical Talos existing-volume document kind.
pub const EXISTING_VOLUME_CONFIG_KIND: &str = "ExistingVolumeConfig";
/// Canonical Talos external-volume document kind.
pub const EXTERNAL_VOLUME_CONFIG_KIND: &str = "ExternalVolumeConfig";
/// Canonical Talos swap-volume document kind.
pub const SWAP_VOLUME_CONFIG_KIND: &str = "SwapVolumeConfig";

/// Upstream Talos prefix for user-volume partition labels.
pub const USER_VOLUME_PREFIX: &str = "u-";
/// Upstream Talos prefix for raw-volume partition labels.
pub const RAW_VOLUME_PREFIX: &str = "r-";
/// Upstream Talos prefix for existing-volume resource IDs.
pub const EXISTING_VOLUME_PREFIX: &str = "e-";
/// Upstream Talos prefix for external-volume resource IDs.
pub const EXTERNAL_VOLUME_PREFIX: &str = "x-";
/// Upstream Talos prefix for swap-volume partition labels.
pub const SWAP_VOLUME_PREFIX: &str = "s-";
/// Upstream Talos default minimum size for partition-backed user volumes.
pub const MIN_USER_VOLUME_SIZE: u64 = 100 * 1024 * 1024;
/// Upstream Talos image-cache system volume name/partition label.
///
/// Source: Talos v1.13.0 `constants.ImageCachePartitionLabel`, used by
/// `VolumeConfigV1Alpha1.Validate` and the CRI image-cache controller.
pub const IMAGE_CACHE_VOLUME_NAME: &str = "IMAGECACHE";

/// A parsed numeric-or-relative `maxSize` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeLimit {
    /// An absolute byte count.
    Absolute(u64),
    /// A percentage expression such as `80%`.
    RelativePercent(u64),
    /// A negative byte count, accepted by upstream for tail-reservation style
    /// max-size expressions.
    NegativeBytes(u64),
    /// A negative percentage expression such as `-80%`.
    NegativeRelativePercent(u64),
}

impl SizeLimit {
    /// Return the absolute byte count, if this limit is absolute.
    pub fn absolute(self) -> Option<u64> {
        match self {
            SizeLimit::Absolute(bytes) => Some(bytes),
            SizeLimit::RelativePercent(_)
            | SizeLimit::NegativeBytes(_)
            | SizeLimit::NegativeRelativePercent(_) => None,
        }
    }
}

/// Provisioning fields shared by `VolumeConfig` and `UserVolumeConfig`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProvisioningSpec {
    /// Validated Talos DiskLocator CEL selector
    /// (`provisioning.diskSelector.match`), preserved as source text.
    pub disk_selector: Option<String>,
    /// Whether the partition should grow when possible.
    pub grow: Option<bool>,
    /// Minimum byte size.
    pub min_size: Option<u64>,
    /// Maximum byte/relative/negative size.
    pub max_size: Option<SizeLimit>,
}

impl ProvisioningSpec {
    /// True when no provisioning field was configured.
    pub fn is_zero(&self) -> bool {
        self.disk_selector.is_none()
            && self.grow.is_none()
            && self.min_size.is_none()
            && self.max_size.is_none()
    }

    /// Absolute max-size value, if present and representable as bytes.
    pub fn absolute_max_size(&self) -> Option<u64> {
        self.max_size.and_then(SizeLimit::absolute)
    }

    fn validate(&self, field: &str, required: bool, size_supported: bool) -> Result<()> {
        if let Some(selector) = &self.disk_selector {
            if selector.trim().is_empty() {
                return Err(Error::invalid(format!(
                    "{field}.diskSelector.match must be non-empty"
                )));
            }
            validate_disk_locator_bool_expression(selector).map_err(|err| {
                Error::invalid(format!("{field}.diskSelector.match is invalid: {err}"))
            })?;
        } else if required {
            return Err(Error::invalid(format!(
                "{field}.diskSelector.match is required"
            )));
        }

        if !size_supported {
            if self.min_size.is_some() || self.max_size.is_some() || self.grow.is_some() {
                return Err(Error::invalid(format!(
                    "{field}: minSize, maxSize and grow are not supported for this volume type"
                )));
            }
            return Ok(());
        }

        if let (Some(min), Some(SizeLimit::Absolute(max))) = (self.min_size, self.max_size)
            && max != 0
            && min > max
        {
            return Err(Error::invalid(format!(
                "{field}: minSize {min} is greater than maxSize {max}"
            )));
        }

        if required && self.min_size.unwrap_or(0) == 0 && self.max_size.is_none() {
            return Err(Error::invalid(format!(
                "{field}: minSize or maxSize is required"
            )));
        }

        Ok(())
    }
}

/// Parsed block volume encryption config.
///
/// Source: Talos `pkg/machinery/config/types/block/encryption.go`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionSpec {
    /// Source `encryption.provider`; Talos accepts `luks2`.
    pub provider: String,
    /// Source encryption keys in declaration order.
    pub keys: Vec<EncryptionKeySpec>,
    /// Optional source cipher string.
    pub cipher: Option<String>,
    /// Optional source key size in bits.
    pub key_size: Option<u32>,
    /// Optional source encryption block size.
    pub block_size: Option<u64>,
    /// Optional source LUKS2 perf options.
    pub options: Vec<String>,
}

impl EncryptionSpec {
    fn validate(&self, field: &str) -> Result<()> {
        if self.provider != "luks2" {
            return Err(Error::invalid(format!(
                "{field}.provider has unsupported value {:?}",
                self.provider
            )));
        }
        if self.keys.is_empty() {
            return Err(Error::invalid(format!("{field}.keys is required")));
        }

        let mut slots = BTreeSet::new();
        for key in &self.keys {
            if !slots.insert(key.slot) {
                return Err(Error::invalid(format!(
                    "{field}.keys has duplicate slot {}",
                    key.slot
                )));
            }
        }

        Ok(())
    }
}

/// One source encryption key declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionKeySpec {
    /// Source LUKS key slot.
    pub slot: u8,
    /// Source key provider.
    pub provider: EncryptionKeyProvider,
    /// Source `lockToState`, defaulting to `false`.
    pub lock_to_state: bool,
}

/// Source encryption key provider variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionKeyProvider {
    /// Static passphrase in config.
    Static { passphrase: String },
    /// Node identity derived key.
    NodeId,
    /// KMS endpoint key.
    Kms { endpoint: String },
    /// TPM-sealed key with source defaults.
    Tpm {
        check_secureboot_status_on_enroll: bool,
        pcrs: Vec<u8>,
    },
}

/// Parsed system `VolumeConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfigDoc {
    /// System volume name.
    pub name: String,
    /// Provisioning override.
    pub provisioning: ProvisioningSpec,
    /// Whether an encryption block was present.
    pub encryption_configured: bool,
    /// Parsed source encryption config, if the block is non-empty and valid.
    pub encryption: Option<EncryptionSpec>,
    /// `mount.secure`, if configured.
    pub mount_secure: Option<bool>,
}

impl VolumeConfigDoc {
    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        match self.name.as_str() {
            "STATE" | "EPHEMERAL" | IMAGE_CACHE_VOLUME_NAME => {}
            other => {
                return Err(Error::invalid(format!(
                    "VolumeConfig: only STATE, EPHEMERAL and IMAGECACHE volumes are supported (got {other:?})"
                )));
            }
        }

        if self.name == "STATE" && !self.provisioning.is_zero() {
            return Err(Error::invalid(
                "VolumeConfig: provisioning config is not allowed for the STATE volume",
            ));
        }

        self.provisioning
            .validate("VolumeConfig.provisioning", false, true)
    }
}

/// User volume type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserVolumeType {
    /// Directory-backed user volume.
    Directory,
    /// Whole-disk user volume.
    Disk,
    /// Partition-backed user volume (Talos default).
    Partition,
}

impl UserVolumeType {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "" | "partition" => Ok(Self::Partition),
            "directory" => Ok(Self::Directory),
            "disk" => Ok(Self::Disk),
            other => Err(Error::invalid(format!(
                "UserVolumeConfig: unsupported volumeType {other:?}"
            ))),
        }
    }

    /// Canonical config string.
    pub fn as_str(self) -> &'static str {
        match self {
            UserVolumeType::Directory => "directory",
            UserVolumeType::Disk => "disk",
            UserVolumeType::Partition => "partition",
        }
    }
}

/// User-volume filesystem type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserVolumeFilesystem {
    /// XFS, the Talos default.
    Xfs,
    /// ext4.
    Ext4,
    /// btrfs, parsed for source parity but not yet projectable to `talos-block`.
    Btrfs,
}

impl UserVolumeFilesystem {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "" | "xfs" => Ok(Self::Xfs),
            "ext4" => Ok(Self::Ext4),
            "btrfs" => Ok(Self::Btrfs),
            other => Err(Error::invalid(format!(
                "UserVolumeConfig: unsupported filesystem type {other:?}"
            ))),
        }
    }

    /// Canonical config string.
    pub fn as_str(self) -> &'static str {
        match self {
            UserVolumeFilesystem::Xfs => "xfs",
            UserVolumeFilesystem::Ext4 => "ext4",
            UserVolumeFilesystem::Btrfs => "btrfs",
        }
    }
}

/// Parsed `filesystem` block for a user volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFilesystemSpec {
    /// Effective filesystem type; defaults to XFS.
    pub filesystem: UserVolumeFilesystem,
    /// Optional project-quota flag.
    pub project_quota_support: Option<bool>,
}

impl Default for UserFilesystemSpec {
    fn default() -> Self {
        UserFilesystemSpec {
            filesystem: UserVolumeFilesystem::Xfs,
            project_quota_support: None,
        }
    }
}

impl UserFilesystemSpec {
    fn validate(&self) -> Result<()> {
        if self.project_quota_support == Some(true) && self.filesystem != UserVolumeFilesystem::Xfs
        {
            return Err(Error::invalid(
                "UserVolumeConfig: projectQuotaSupport is only available for xfs filesystem",
            ));
        }
        Ok(())
    }
}

/// Parsed user-volume mount options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserMountSpec {
    /// `mount.disableAccessTime`, if configured.
    pub disable_access_time: Option<bool>,
    /// `mount.secure`, if configured.
    pub secure: Option<bool>,
}

impl UserMountSpec {
    /// True when no mount option was configured.
    pub fn is_zero(&self) -> bool {
        self.disable_access_time.is_none() && self.secure.is_none()
    }
}

/// Parsed `UserVolumeConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserVolumeConfigDoc {
    /// User-supplied name.
    pub name: String,
    /// Volume kind; defaults to partition.
    pub volume_type: UserVolumeType,
    /// Provisioning parameters.
    pub provisioning: ProvisioningSpec,
    /// Filesystem settings.
    pub filesystem: UserFilesystemSpec,
    /// Whether an encryption block was present.
    pub encryption_configured: bool,
    /// Parsed source encryption config, if the block is non-empty and valid.
    pub encryption: Option<EncryptionSpec>,
    /// Mount settings.
    pub mount: UserMountSpec,
}

impl UserVolumeConfigDoc {
    /// Upstream-derived partition label / block volume ID.
    pub fn volume_id(&self) -> String {
        format!("{USER_VOLUME_PREFIX}{}", self.name)
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        validate_user_volume_name(&self.name)?;
        match self.volume_type {
            UserVolumeType::Directory => {
                if !self.provisioning.is_zero() {
                    return Err(Error::invalid(
                        "UserVolumeConfig: provisioning spec is invalid for volumeType directory",
                    ));
                }
                if self.encryption_configured {
                    return Err(Error::invalid(
                        "UserVolumeConfig: encryption spec is invalid for volumeType directory",
                    ));
                }
                if self.filesystem != UserFilesystemSpec::default() {
                    return Err(Error::invalid(
                        "UserVolumeConfig: filesystem spec is invalid for volumeType directory",
                    ));
                }
                if !self.mount.is_zero() {
                    return Err(Error::invalid(
                        "UserVolumeConfig: mount spec is invalid for volumeType directory",
                    ));
                }
            }
            UserVolumeType::Disk => {
                self.provisioning
                    .validate("UserVolumeConfig.provisioning", true, false)?;
                self.filesystem.validate()?;
            }
            UserVolumeType::Partition => {
                self.provisioning
                    .validate("UserVolumeConfig.provisioning", true, true)?;
                self.filesystem.validate()?;
            }
        }
        Ok(())
    }
}

/// Parsed `RawVolumeConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawVolumeConfigDoc {
    /// User-supplied raw volume name.
    pub name: String,
    /// Provisioning parameters.
    pub provisioning: ProvisioningSpec,
    /// Whether an encryption block was present.
    pub encryption_configured: bool,
    /// Parsed source encryption config, if the block is non-empty and valid.
    pub encryption: Option<EncryptionSpec>,
}

impl RawVolumeConfigDoc {
    /// Upstream-derived raw partition label / block volume ID.
    pub fn volume_id(&self) -> String {
        format!("{RAW_VOLUME_PREFIX}{}", self.name)
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        validate_user_volume_name(&self.name)?;
        self.provisioning
            .validate("RawVolumeConfig.provisioning", true, true)?;
        Ok(())
    }
}

/// Parsed mount options shared by existing and external volume configs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportedMountSpec {
    /// `mount.readOnly`, if configured.
    pub read_only: Option<bool>,
    /// `mount.disableAccessTime`, if configured.
    pub disable_access_time: Option<bool>,
    /// `mount.secure`, if configured. Upstream defaults this to `true`.
    pub secure: Option<bool>,
}

impl ImportedMountSpec {
    /// Source-effective `readOnly` value.
    pub fn read_only_effective(&self) -> bool {
        self.read_only.unwrap_or(false)
    }

    /// Source-effective `disableAccessTime` value.
    pub fn disable_access_time_effective(&self) -> bool {
        self.disable_access_time.unwrap_or(false)
    }

    /// Source-effective `secure` value.
    pub fn secure_effective(&self) -> bool {
        self.secure.unwrap_or(true)
    }
}

/// Parsed `ExistingVolumeConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingVolumeConfigDoc {
    /// User-supplied existing volume name.
    pub name: String,
    /// Source `discovery.volumeSelector.match` expression.
    pub volume_selector: String,
    /// Mount request options.
    pub mount: ImportedMountSpec,
}

impl ExistingVolumeConfigDoc {
    /// Upstream-derived block volume ID.
    pub fn volume_id(&self) -> String {
        format!("{EXISTING_VOLUME_PREFIX}{}", self.name)
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        validate_imported_volume_name(&self.name, "ExistingVolumeConfig", None)?;
        if self.volume_selector.trim().is_empty() {
            return Err(Error::invalid(
                "ExistingVolumeConfig.discovery.volumeSelector.match is required",
            ));
        }
        validate_volume_locator_bool_expression(&self.volume_selector).map_err(|err| {
            Error::invalid(format!(
                "ExistingVolumeConfig.discovery.volumeSelector.match is invalid: {err}"
            ))
        })?;
        Ok(())
    }
}

/// External-volume filesystem types accepted by Talos v1.13.0
/// `ExternalVolumeConfig` validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalVolumeFilesystem {
    /// virtiofs shared filesystem.
    Virtiofs,
}

impl ExternalVolumeFilesystem {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "virtiofs" => Ok(Self::Virtiofs),
            other => Err(Error::invalid(format!(
                "ExternalVolumeConfig: invalid filesystem type {other:?}"
            ))),
        }
    }

    /// Canonical config string.
    pub fn as_str(self) -> &'static str {
        match self {
            ExternalVolumeFilesystem::Virtiofs => "virtiofs",
        }
    }
}

/// Parsed external-volume mount options.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExternalMountSpec {
    /// `mount.readOnly`, if configured.
    pub read_only: Option<bool>,
    /// `mount.disableAccessTime`, if configured.
    pub disable_access_time: Option<bool>,
    /// `mount.secure`, if configured. Upstream defaults this to `true`.
    pub secure: Option<bool>,
    /// Source `mount.virtiofs.tag`.
    pub virtiofs_tag: Option<String>,
}

impl ExternalMountSpec {
    /// Source-effective `readOnly` value.
    pub fn read_only_effective(&self) -> bool {
        self.read_only.unwrap_or(false)
    }

    /// Source-effective `disableAccessTime` value.
    pub fn disable_access_time_effective(&self) -> bool {
        self.disable_access_time.unwrap_or(false)
    }

    /// Source-effective `secure` value.
    pub fn secure_effective(&self) -> bool {
        self.secure.unwrap_or(true)
    }
}

/// Parsed `ExternalVolumeConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalVolumeConfigDoc {
    /// User-supplied external volume name.
    pub name: String,
    /// External filesystem type.
    pub filesystem: ExternalVolumeFilesystem,
    /// Mount request and virtiofs options.
    pub mount: ExternalMountSpec,
}

impl ExternalVolumeConfigDoc {
    /// Upstream-derived block volume ID.
    pub fn volume_id(&self) -> String {
        format!("{EXTERNAL_VOLUME_PREFIX}{}", self.name)
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        validate_imported_volume_name(&self.name, "ExternalVolumeConfig", Some(34))?;
        match self.filesystem {
            ExternalVolumeFilesystem::Virtiofs => {
                if self.mount.virtiofs_tag.as_deref().is_none_or(str::is_empty) {
                    return Err(Error::invalid(
                        "ExternalVolumeConfig.mount.virtiofs.tag is required",
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Parsed `SwapVolumeConfig` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapVolumeConfigDoc {
    /// User-supplied swap volume name.
    pub name: String,
    /// Provisioning parameters.
    pub provisioning: ProvisioningSpec,
    /// Whether an encryption block was present.
    pub encryption_configured: bool,
    /// Parsed source encryption config, if the block is non-empty and valid.
    pub encryption: Option<EncryptionSpec>,
}

impl SwapVolumeConfigDoc {
    /// Upstream-derived swap partition label / block volume ID.
    pub fn volume_id(&self) -> String {
        format!("{SWAP_VOLUME_PREFIX}{}", self.name)
    }

    /// Validate the document in isolation.
    pub fn validate(&self) -> Result<()> {
        validate_user_volume_name(&self.name)?;
        self.provisioning
            .validate("SwapVolumeConfig.provisioning", true, true)?;
        Ok(())
    }
}

/// Decode and validate one `VolumeConfig` document body.
pub fn decode_volume_config_body(body: &str) -> Result<VolumeConfigDoc> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != VOLUME_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "VolumeConfig: unexpected kind {kind:?}"
        )));
    }

    let name = required_string(&root, "name", "VolumeConfig.name")?.to_string();
    let provisioning = match root.get("provisioning") {
        Some(value) => decode_provisioning(value, "VolumeConfig.provisioning")?,
        None => ProvisioningSpec::default(),
    };
    let mount_secure = match root.get("mount") {
        Some(value) => decode_mount_secure(value, "VolumeConfig.mount")?,
        None => None,
    };
    let encryption_configured = mapping_present(root.get("encryption"), "VolumeConfig.encryption")?;
    let encryption = decode_encryption(root.get("encryption"), "VolumeConfig.encryption")?;

    let doc = VolumeConfigDoc {
        name,
        provisioning,
        encryption_configured,
        encryption,
        mount_secure,
    };
    doc.validate()?;
    Ok(doc)
}

/// Decode and validate one `RawVolumeConfig` document body.
pub fn decode_raw_volume_config_body(body: &str) -> Result<RawVolumeConfigDoc> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != RAW_VOLUME_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "RawVolumeConfig: unexpected kind {kind:?}"
        )));
    }

    let name = required_string(&root, "name", "RawVolumeConfig.name")?.to_string();
    let provisioning = match root.get("provisioning") {
        Some(value) => decode_provisioning(value, "RawVolumeConfig.provisioning")?,
        None => ProvisioningSpec::default(),
    };
    let encryption_configured =
        mapping_present(root.get("encryption"), "RawVolumeConfig.encryption")?;
    let encryption = decode_encryption(root.get("encryption"), "RawVolumeConfig.encryption")?;

    let doc = RawVolumeConfigDoc {
        name,
        provisioning,
        encryption_configured,
        encryption,
    };
    doc.validate()?;
    Ok(doc)
}

/// Decode and validate one `ExistingVolumeConfig` document body.
pub fn decode_existing_volume_config_body(body: &str) -> Result<ExistingVolumeConfigDoc> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != EXISTING_VOLUME_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "ExistingVolumeConfig: unexpected kind {kind:?}"
        )));
    }

    let name = required_string(&root, "name", "ExistingVolumeConfig.name")?.to_string();
    let volume_selector = decode_volume_selector(&root)?;
    let mount = match root.get("mount") {
        Some(value) => decode_imported_mount(value, "ExistingVolumeConfig.mount")?,
        None => ImportedMountSpec::default(),
    };

    let doc = ExistingVolumeConfigDoc {
        name,
        volume_selector,
        mount,
    };
    doc.validate()?;
    Ok(doc)
}

/// Decode and validate one `ExternalVolumeConfig` document body.
pub fn decode_external_volume_config_body(body: &str) -> Result<ExternalVolumeConfigDoc> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != EXTERNAL_VOLUME_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "ExternalVolumeConfig: unexpected kind {kind:?}"
        )));
    }

    let name = required_string(&root, "name", "ExternalVolumeConfig.name")?.to_string();
    let filesystem = ExternalVolumeFilesystem::parse(required_string(
        &root,
        "filesystemType",
        "ExternalVolumeConfig.filesystemType",
    )?)?;
    let mount = match root.get("mount") {
        Some(value) => decode_external_mount(value)?,
        None => ExternalMountSpec::default(),
    };

    let doc = ExternalVolumeConfigDoc {
        name,
        filesystem,
        mount,
    };
    doc.validate()?;
    Ok(doc)
}

/// Decode and validate one `SwapVolumeConfig` document body.
pub fn decode_swap_volume_config_body(body: &str) -> Result<SwapVolumeConfigDoc> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != SWAP_VOLUME_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "SwapVolumeConfig: unexpected kind {kind:?}"
        )));
    }

    let name = required_string(&root, "name", "SwapVolumeConfig.name")?.to_string();
    let provisioning = match root.get("provisioning") {
        Some(value) => decode_provisioning(value, "SwapVolumeConfig.provisioning")?,
        None => ProvisioningSpec::default(),
    };
    let encryption_configured =
        mapping_present(root.get("encryption"), "SwapVolumeConfig.encryption")?;
    let encryption = decode_encryption(root.get("encryption"), "SwapVolumeConfig.encryption")?;

    let doc = SwapVolumeConfigDoc {
        name,
        provisioning,
        encryption_configured,
        encryption,
    };
    doc.validate()?;
    Ok(doc)
}

/// Decode and validate one `UserVolumeConfig` document body.
pub fn decode_user_volume_config_body(body: &str) -> Result<UserVolumeConfigDoc> {
    let root = yaml::parse(body).map_err(|e| Error::parse(e.to_string()))?;
    if let Some(kind) = root.get_str("kind")
        && kind != USER_VOLUME_CONFIG_KIND
    {
        return Err(Error::invalid(format!(
            "UserVolumeConfig: unexpected kind {kind:?}"
        )));
    }

    let name = required_string(&root, "name", "UserVolumeConfig.name")?.to_string();
    let volume_type = match root.get_str("volumeType") {
        Some(raw) => UserVolumeType::parse(raw)?,
        None => UserVolumeType::Partition,
    };
    let provisioning = match root.get("provisioning") {
        Some(value) => decode_provisioning(value, "UserVolumeConfig.provisioning")?,
        None => ProvisioningSpec::default(),
    };
    let filesystem = match root.get("filesystem") {
        Some(value) => decode_filesystem(value)?,
        None => UserFilesystemSpec::default(),
    };
    let encryption_configured =
        mapping_present(root.get("encryption"), "UserVolumeConfig.encryption")?;
    let encryption = decode_encryption(root.get("encryption"), "UserVolumeConfig.encryption")?;
    let mount = match root.get("mount") {
        Some(value) => decode_user_mount(value)?,
        None => UserMountSpec::default(),
    };

    let doc = UserVolumeConfigDoc {
        name,
        volume_type,
        provisioning,
        filesystem,
        encryption_configured,
        encryption,
        mount,
    };
    doc.validate()?;
    Ok(doc)
}

/// Extract all system `VolumeConfig` documents from a loaded config.
pub fn volume_configs(config: &Config) -> Result<Vec<VolumeConfigDoc>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == VOLUME_CONFIG_KIND)
    {
        let parsed = decode_volume_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate VolumeConfig document for volume '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

/// Extract all `RawVolumeConfig` documents from a loaded config.
pub fn raw_volume_configs(config: &Config) -> Result<Vec<RawVolumeConfigDoc>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == RAW_VOLUME_CONFIG_KIND)
    {
        let parsed = decode_raw_volume_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate RawVolumeConfig document for volume '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

/// Extract all `ExistingVolumeConfig` documents from a loaded config.
pub fn existing_volume_configs(config: &Config) -> Result<Vec<ExistingVolumeConfigDoc>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == EXISTING_VOLUME_CONFIG_KIND)
    {
        let parsed = decode_existing_volume_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate ExistingVolumeConfig document for volume '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

/// Extract all `ExternalVolumeConfig` documents from a loaded config.
pub fn external_volume_configs(config: &Config) -> Result<Vec<ExternalVolumeConfigDoc>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == EXTERNAL_VOLUME_CONFIG_KIND)
    {
        let parsed = decode_external_volume_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate ExternalVolumeConfig document for volume '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

/// Extract all `SwapVolumeConfig` documents from a loaded config.
pub fn swap_volume_configs(config: &Config) -> Result<Vec<SwapVolumeConfigDoc>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == SWAP_VOLUME_CONFIG_KIND)
    {
        let parsed = decode_swap_volume_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate SwapVolumeConfig document for volume '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

/// Extract all `UserVolumeConfig` documents from a loaded config.
pub fn user_volume_configs(config: &Config) -> Result<Vec<UserVolumeConfigDoc>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for doc in config
        .documents()
        .iter()
        .filter(|doc| doc.meta.kind == USER_VOLUME_CONFIG_KIND)
    {
        let parsed = decode_user_volume_config_body(&doc.body)?;
        if !seen.insert(parsed.name.clone()) {
            return Err(Error::invalid(format!(
                "duplicate UserVolumeConfig document for volume '{}'",
                parsed.name
            )));
        }
        out.push(parsed);
    }
    Ok(out)
}

fn decode_provisioning(value: &Yaml, field: &str) -> Result<ProvisioningSpec> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    let disk_selector = match value.get("diskSelector") {
        Some(selector) => Some(decode_disk_selector(
            selector,
            &format!("{field}.diskSelector"),
        )?),
        None => None,
    };
    let grow = optional_bool(value.get("grow"), &format!("{field}.grow"))?;
    let min_size = match value.get("minSize") {
        Some(v) => Some(parse_byte_size(
            scalar(v, &format!("{field}.minSize"))?,
            &format!("{field}.minSize"),
        )?),
        None => None,
    };
    let max_size = match value.get("maxSize") {
        Some(v) => Some(parse_size_limit(
            scalar(v, &format!("{field}.maxSize"))?,
            &format!("{field}.maxSize"),
        )?),
        None => None,
    };
    Ok(ProvisioningSpec {
        disk_selector,
        grow,
        min_size,
        max_size,
    })
}

pub(crate) fn decode_encryption(
    value: Option<&Yaml>,
    field: &str,
) -> Result<Option<EncryptionSpec>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(mapping) = value.as_mapping() else {
        return Err(Error::parse(format!("{field} must be a mapping")));
    };
    if mapping.is_empty() {
        return Ok(None);
    }

    let provider = required_string(value, "provider", &format!("{field}.provider"))?.to_string();
    let keys_value = value
        .get("keys")
        .ok_or_else(|| Error::invalid(format!("{field}.keys is required")))?;
    let keys = decode_encryption_keys(keys_value, &format!("{field}.keys"))?;
    let cipher = value.get_str("cipher").map(str::to_string);
    let key_size = optional_u32(value.get("keySize"), &format!("{field}.keySize"))?;
    let block_size = optional_u64(value.get("blockSize"), &format!("{field}.blockSize"))?;
    let options = match value.get("options") {
        Some(options) => decode_string_sequence(options, &format!("{field}.options"))?,
        None => Vec::new(),
    };

    let encryption = EncryptionSpec {
        provider,
        keys,
        cipher,
        key_size,
        block_size,
        options,
    };
    encryption.validate(field)?;
    Ok(Some(encryption))
}

/// Decode the JSON string stored in Talos runtime `MetaKey` 0x09 for STATE
/// encryption.
///
/// Source: `volumes.MarshalEncryptionMeta` uses Go `encoding/json` over either
/// current block `EncryptionSpec` or legacy v1alpha1 `EncryptionConfig`, both
/// of which have YAML tags but no JSON tags. The persisted JSON therefore uses
/// Go exported field names (`EncryptionProvider`, `EncryptionKeys`, `KeySlot`,
/// `KeyStatic`, `KeyData`, ...), not the YAML field names used in config
/// documents. This decoder accepts both the source META JSON shape and the YAML
/// aliases used by manually supplied JSON.
pub fn decode_encryption_meta_value(value: &str) -> Result<Option<EncryptionSpec>> {
    let value = value.trim();
    if value.is_empty() || value == "null" {
        return Ok(None);
    }

    let json = parse_json_value(value, "StateEncryptionMeta")?;
    decode_encryption_meta_json(&json, "StateEncryptionMeta")
}

fn decode_encryption_meta_json(value: &JsonValue, field: &str) -> Result<Option<EncryptionSpec>> {
    let object = json_object(value, field)?;
    let provider =
        json_string_alias(object, &["EncryptionProvider", "provider"], field)?.unwrap_or_default();
    let keys_value = json_non_null_alias(object, &["EncryptionKeys", "keys"]);

    if provider.is_empty() && keys_value.is_none() {
        return Ok(None);
    }

    let keys = match keys_value {
        Some(keys) => decode_json_encryption_keys(keys, &format!("{field}.EncryptionKeys"))?,
        None => Vec::new(),
    };
    let cipher = json_string_alias(object, &["EncryptionCipher", "cipher"], field)?
        .filter(|s| !s.is_empty());
    let key_size = json_u64_alias(object, &["EncryptionKeySize", "keySize"], field)?
        .map(|value| {
            u32::try_from(value)
                .map_err(|_| Error::invalid(format!("{field}.EncryptionKeySize is too large")))
        })
        .transpose()?;
    let block_size = json_u64_alias(object, &["EncryptionBlockSize", "blockSize"], field)?;
    let options = match json_non_null_alias(object, &["EncryptionPerfOptions", "options"]) {
        Some(options) => {
            decode_json_string_array(options, &format!("{field}.EncryptionPerfOptions"))?
        }
        None => Vec::new(),
    };

    let encryption = EncryptionSpec {
        provider,
        keys,
        cipher,
        key_size,
        block_size,
        options,
    };
    encryption.validate(field)?;
    Ok(Some(encryption))
}

fn decode_json_encryption_keys(value: &JsonValue, field: &str) -> Result<Vec<EncryptionKeySpec>> {
    let JsonValue::Array(items) = value else {
        return Err(Error::parse(format!("{field} must be an array")));
    };

    items
        .iter()
        .enumerate()
        .map(|(index, item)| decode_json_encryption_key(item, &format!("{field}[{index}]")))
        .collect()
}

fn decode_json_encryption_key(value: &JsonValue, field: &str) -> Result<EncryptionKeySpec> {
    let object = json_object(value, field)?;
    let slot = json_u64_alias(object, &["KeySlot", "slot"], field)?
        .ok_or_else(|| Error::invalid(format!("{field}.KeySlot is required")))?;
    let slot =
        u8::try_from(slot).map_err(|_| Error::invalid(format!("{field}.KeySlot is too large")))?;
    let lock_to_state =
        json_bool_alias(object, &["KeyLockToSTATE", "lockToState"], field)?.unwrap_or(false);

    let provider = if let Some(static_key) = json_non_null_alias(object, &["KeyStatic", "static"]) {
        decode_json_static_encryption_key(static_key, &format!("{field}.KeyStatic"))?
    } else if let Some(node_id) = json_non_null_alias(object, &["KeyNodeID", "nodeID"]) {
        json_object(node_id, &format!("{field}.KeyNodeID"))?;
        EncryptionKeyProvider::NodeId
    } else if let Some(kms) = json_non_null_alias(object, &["KeyKMS", "kms"]) {
        decode_json_kms_encryption_key(kms, &format!("{field}.KeyKMS"))?
    } else if let Some(tpm) = json_non_null_alias(object, &["KeyTPM", "tpm"]) {
        decode_json_tpm_encryption_key(tpm, &format!("{field}.KeyTPM"))?
    } else {
        return Err(Error::invalid(format!(
            "{field}: at least one encryption key type must be specified"
        )));
    };

    Ok(EncryptionKeySpec {
        slot,
        provider,
        lock_to_state,
    })
}

fn decode_json_static_encryption_key(
    value: &JsonValue,
    field: &str,
) -> Result<EncryptionKeyProvider> {
    let object = json_object(value, field)?;
    Ok(EncryptionKeyProvider::Static {
        passphrase: json_string_alias(object, &["KeyData", "passphrase"], field)?
            .ok_or_else(|| Error::invalid(format!("{field}.KeyData is required")))?,
    })
}

fn decode_json_kms_encryption_key(value: &JsonValue, field: &str) -> Result<EncryptionKeyProvider> {
    let object = json_object(value, field)?;
    Ok(EncryptionKeyProvider::Kms {
        endpoint: json_string_alias(object, &["KMSEndpoint", "endpoint"], field)?
            .ok_or_else(|| Error::invalid(format!("{field}.KMSEndpoint is required")))?,
    })
}

fn decode_json_tpm_encryption_key(value: &JsonValue, field: &str) -> Result<EncryptionKeyProvider> {
    let object = json_object(value, field)?;
    let check_secureboot_status_on_enroll = json_bool_alias(
        object,
        &[
            "TPMCheckSecurebootStatusOnEnroll",
            "checkSecurebootStatusOnEnroll",
        ],
        field,
    )?
    .unwrap_or(false);
    let pcrs = match json_non_null_alias(object, &["TPMOptions", "options"]) {
        Some(options) => {
            let options = json_object(options, &format!("{field}.TPMOptions"))?;
            match json_non_null_alias(options, &["PCRs", "pcrs"]) {
                Some(pcrs) => decode_json_u8_array(pcrs, &format!("{field}.TPMOptions.PCRs"))?,
                None => Vec::new(),
            }
        }
        None => vec![7],
    };
    for pcr in &pcrs {
        if *pcr > 23 {
            return Err(Error::invalid(format!(
                "{field}.TPMOptions.PCRs contains out-of-range PCR {pcr}"
            )));
        }
    }

    Ok(EncryptionKeyProvider::Tpm {
        check_secureboot_status_on_enroll,
        pcrs,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(u64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

fn parse_json_value(input: &str, field: &str) -> Result<JsonValue> {
    let mut parser = JsonParser {
        input,
        index: 0,
        field,
    };
    let value = parser.parse_value()?;
    parser.skip_ws();
    if !parser.is_eof() {
        return Err(parser.error("trailing content after JSON value"));
    }
    Ok(value)
}

struct JsonParser<'a> {
    input: &'a str,
    index: usize,
    field: &'a str,
}

impl JsonParser<'_> {
    fn parse_value(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        match self.peek_byte() {
            Some(b'n') => {
                self.expect_literal("null")?;
                Ok(JsonValue::Null)
            }
            Some(b't') => {
                self.expect_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.expect_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'0'..=b'9') => self.parse_number().map(JsonValue::Number),
            Some(_) => Err(self.error("unexpected JSON token")),
            None => Err(self.error("unexpected end of JSON")),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.expect_byte(b'[')?;
        self.skip_ws();
        let mut values = Vec::new();
        if self.consume_byte(b']') {
            return Ok(JsonValue::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_ws();
            if self.consume_byte(b']') {
                break;
            }
            self.expect_byte(b',')?;
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.expect_byte(b'{')?;
        self.skip_ws();
        let mut values = BTreeMap::new();
        if self.consume_byte(b'}') {
            return Ok(JsonValue::Object(values));
        }

        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            values.insert(key, value);
            self.skip_ws();
            if self.consume_byte(b'}') {
                break;
            }
            self.expect_byte(b',')?;
        }

        Ok(JsonValue::Object(values))
    }

    fn parse_string(&mut self) -> Result<String> {
        self.expect_byte(b'"')?;
        let mut out = String::new();
        while !self.is_eof() {
            let byte = self.peek_byte().expect("checked eof");
            match byte {
                b'"' => {
                    self.index += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.index += 1;
                    out.push(self.parse_escape()?);
                }
                0x00..=0x1f => return Err(self.error("control character in JSON string")),
                _ => {
                    let ch = self.next_char()?;
                    out.push(ch);
                }
            }
        }

        Err(self.error("unterminated JSON string"))
    }

    fn parse_escape(&mut self) -> Result<char> {
        let Some(byte) = self.peek_byte() else {
            return Err(self.error("unterminated JSON escape"));
        };
        self.index += 1;
        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{0008}'),
            b'f' => Ok('\u{000c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.parse_unicode_escape(),
            _ => Err(self.error("unsupported JSON escape")),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char> {
        let end = self.index + 4;
        if end > self.input.len() {
            return Err(self.error("short JSON unicode escape"));
        }
        let digits = &self.input[self.index..end];
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(self.error("invalid JSON unicode escape"));
        }
        self.index = end;
        let value = u32::from_str_radix(digits, 16).map_err(|err| {
            Error::parse(format!("{}: invalid unicode escape: {err}", self.field))
        })?;
        char::from_u32(value).ok_or_else(|| self.error("invalid JSON unicode scalar"))
    }

    fn parse_number(&mut self) -> Result<u64> {
        let start = self.index;
        while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
        self.input[start..self.index]
            .parse::<u64>()
            .map_err(|err| Error::parse(format!("{}: invalid JSON number: {err}", self.field)))
    }

    fn next_char(&mut self) -> Result<char> {
        let slice = &self.input[self.index..];
        let Some(ch) = slice.chars().next() else {
            return Err(self.error("unexpected end of JSON string"));
        };
        self.index += ch.len_utf8();
        Ok(ch)
    }

    fn expect_literal(&mut self, literal: &str) -> Result<()> {
        if self.input[self.index..].starts_with(literal) {
            self.index += literal.len();
            Ok(())
        } else {
            Err(self.error("invalid JSON literal"))
        }
    }

    fn expect_byte(&mut self, byte: u8) -> Result<()> {
        self.skip_ws();
        if self.consume_byte(byte) {
            Ok(())
        } else {
            Err(self.error("unexpected JSON delimiter"))
        }
    }

    fn consume_byte(&mut self, byte: u8) -> bool {
        if self.peek_byte() == Some(byte) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.index).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }

    fn is_eof(&self) -> bool {
        self.index >= self.input.len()
    }

    fn error(&self, message: &str) -> Error {
        Error::parse(format!("{} at byte {}: {message}", self.field, self.index))
    }
}

fn json_object<'a>(value: &'a JsonValue, field: &str) -> Result<&'a BTreeMap<String, JsonValue>> {
    match value {
        JsonValue::Object(object) => Ok(object),
        _ => Err(Error::parse(format!("{field} must be a JSON object"))),
    }
}

fn json_non_null_alias<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    aliases: &[&str],
) -> Option<&'a JsonValue> {
    aliases
        .iter()
        .find_map(|alias| object.get(*alias))
        .filter(|value| !matches!(value, JsonValue::Null))
}

fn json_string_alias(
    object: &BTreeMap<String, JsonValue>,
    aliases: &[&str],
    field: &str,
) -> Result<Option<String>> {
    let Some(value) = json_non_null_alias(object, aliases) else {
        return Ok(None);
    };
    match value {
        JsonValue::String(value) => Ok(Some(value.clone())),
        _ => Err(Error::parse(format!(
            "{field}.{} must be a JSON string",
            aliases[0]
        ))),
    }
}

fn json_u64_alias(
    object: &BTreeMap<String, JsonValue>,
    aliases: &[&str],
    field: &str,
) -> Result<Option<u64>> {
    let Some(value) = json_non_null_alias(object, aliases) else {
        return Ok(None);
    };
    match value {
        JsonValue::Number(value) => Ok(Some(*value)),
        _ => Err(Error::parse(format!(
            "{field}.{} must be a JSON number",
            aliases[0]
        ))),
    }
}

fn json_bool_alias(
    object: &BTreeMap<String, JsonValue>,
    aliases: &[&str],
    field: &str,
) -> Result<Option<bool>> {
    let Some(value) = json_non_null_alias(object, aliases) else {
        return Ok(None);
    };
    match value {
        JsonValue::Bool(value) => Ok(Some(*value)),
        _ => Err(Error::parse(format!(
            "{field}.{} must be a JSON bool",
            aliases[0]
        ))),
    }
}

fn decode_json_string_array(value: &JsonValue, field: &str) -> Result<Vec<String>> {
    let JsonValue::Array(items) = value else {
        return Err(Error::parse(format!("{field} must be an array")));
    };

    items
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            JsonValue::String(value) => Ok(value.clone()),
            _ => Err(Error::parse(format!("{field}[{index}] must be a string"))),
        })
        .collect()
}

fn decode_json_u8_array(value: &JsonValue, field: &str) -> Result<Vec<u8>> {
    let JsonValue::Array(items) = value else {
        return Err(Error::parse(format!("{field} must be an array")));
    };

    items
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            JsonValue::Number(value) => u8::try_from(*value)
                .map_err(|_| Error::invalid(format!("{field}[{index}] is too large"))),
            _ => Err(Error::parse(format!("{field}[{index}] must be a number"))),
        })
        .collect()
}

fn decode_encryption_keys(value: &Yaml, field: &str) -> Result<Vec<EncryptionKeySpec>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(format!("{field} must be a sequence")));
    };
    let mut keys = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        keys.push(decode_encryption_key(item, &format!("{field}[{index}]"))?);
    }
    Ok(keys)
}

fn decode_encryption_key(value: &Yaml, field: &str) -> Result<EncryptionKeySpec> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    let slot = required_u8(value, "slot", &format!("{field}.slot"))?;
    let lock_to_state =
        optional_bool(value.get("lockToState"), &format!("{field}.lockToState"))?.unwrap_or(false);

    let provider = if let Some(static_key) = value.get("static") {
        decode_static_encryption_key(static_key, &format!("{field}.static"))?
    } else if let Some(node_id) = value.get("nodeID") {
        if node_id.as_mapping().is_none() {
            return Err(Error::parse(format!("{field}.nodeID must be a mapping")));
        }
        EncryptionKeyProvider::NodeId
    } else if let Some(kms) = value.get("kms") {
        decode_kms_encryption_key(kms, &format!("{field}.kms"))?
    } else if let Some(tpm) = value.get("tpm") {
        decode_tpm_encryption_key(tpm, &format!("{field}.tpm"))?
    } else {
        return Err(Error::invalid(format!(
            "{field}: at least one encryption key type must be specified"
        )));
    };

    Ok(EncryptionKeySpec {
        slot,
        provider,
        lock_to_state,
    })
}

fn decode_static_encryption_key(value: &Yaml, field: &str) -> Result<EncryptionKeyProvider> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    Ok(EncryptionKeyProvider::Static {
        passphrase: required_string(value, "passphrase", &format!("{field}.passphrase"))?
            .to_string(),
    })
}

fn decode_kms_encryption_key(value: &Yaml, field: &str) -> Result<EncryptionKeyProvider> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    Ok(EncryptionKeyProvider::Kms {
        endpoint: required_string(value, "endpoint", &format!("{field}.endpoint"))?.to_string(),
    })
}

fn decode_tpm_encryption_key(value: &Yaml, field: &str) -> Result<EncryptionKeyProvider> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    let check_secureboot_status_on_enroll = optional_bool(
        value.get("checkSecurebootStatusOnEnroll"),
        &format!("{field}.checkSecurebootStatusOnEnroll"),
    )?
    .unwrap_or(false);
    let pcrs = match value.get("options") {
        Some(options) => {
            if options.as_mapping().is_none() {
                return Err(Error::parse(format!("{field}.options must be a mapping")));
            }
            match options.get("pcrs") {
                Some(pcrs) => decode_u8_sequence(pcrs, &format!("{field}.options.pcrs"))?,
                None => Vec::new(),
            }
        }
        // Source default: `EncryptionKeyTPM.PCRs()` returns SecureBootStatePCR
        // (PCR 7) when no TPM options block is configured.
        None => vec![7],
    };
    for pcr in &pcrs {
        if *pcr > 23 {
            return Err(Error::invalid(format!(
                "{field}.options.pcrs contains out-of-range PCR {pcr}"
            )));
        }
    }
    Ok(EncryptionKeyProvider::Tpm {
        check_secureboot_status_on_enroll,
        pcrs,
    })
}

fn decode_disk_selector(value: &Yaml, field: &str) -> Result<String> {
    if let Some(raw) = value.as_str() {
        if raw.trim().is_empty() {
            return Err(Error::invalid(format!("{field} must be non-empty")));
        }
        validate_disk_locator_bool_expression(raw)
            .map_err(|err| Error::invalid(format!("{field} is invalid: {err}")))?;
        return Ok(raw.to_string());
    }
    let Some(match_expr) = value.get_str("match") else {
        return Err(Error::parse(format!("{field}.match must be a string")));
    };
    if match_expr.trim().is_empty() {
        return Err(Error::invalid(format!("{field}.match must be non-empty")));
    }
    validate_disk_locator_bool_expression(match_expr)
        .map_err(|err| Error::invalid(format!("{field}.match is invalid: {err}")))?;
    Ok(match_expr.to_string())
}

fn decode_mount_secure(value: &Yaml, field: &str) -> Result<Option<bool>> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    optional_bool(value.get("secure"), &format!("{field}.secure"))
}

fn decode_user_mount(value: &Yaml) -> Result<UserMountSpec> {
    if value.as_mapping().is_none() {
        return Err(Error::parse("UserVolumeConfig.mount must be a mapping"));
    }
    Ok(UserMountSpec {
        disable_access_time: optional_bool(
            value.get("disableAccessTime"),
            "UserVolumeConfig.mount.disableAccessTime",
        )?,
        secure: optional_bool(value.get("secure"), "UserVolumeConfig.mount.secure")?,
    })
}

fn decode_imported_mount(value: &Yaml, field: &str) -> Result<ImportedMountSpec> {
    if value.as_mapping().is_none() {
        return Err(Error::parse(format!("{field} must be a mapping")));
    }
    Ok(ImportedMountSpec {
        read_only: optional_bool(value.get("readOnly"), &format!("{field}.readOnly"))?,
        disable_access_time: optional_bool(
            value.get("disableAccessTime"),
            &format!("{field}.disableAccessTime"),
        )?,
        secure: optional_bool(value.get("secure"), &format!("{field}.secure"))?,
    })
}

fn decode_external_mount(value: &Yaml) -> Result<ExternalMountSpec> {
    if value.as_mapping().is_none() {
        return Err(Error::parse("ExternalVolumeConfig.mount must be a mapping"));
    }
    let virtiofs_tag = match value.get("virtiofs") {
        Some(virtiofs) => {
            if virtiofs.as_mapping().is_none() {
                return Err(Error::parse(
                    "ExternalVolumeConfig.mount.virtiofs must be a mapping",
                ));
            }
            Some(
                required_string(virtiofs, "tag", "ExternalVolumeConfig.mount.virtiofs.tag")?
                    .to_string(),
            )
        }
        None => None,
    };

    Ok(ExternalMountSpec {
        read_only: optional_bool(value.get("readOnly"), "ExternalVolumeConfig.mount.readOnly")?,
        disable_access_time: optional_bool(
            value.get("disableAccessTime"),
            "ExternalVolumeConfig.mount.disableAccessTime",
        )?,
        secure: optional_bool(value.get("secure"), "ExternalVolumeConfig.mount.secure")?,
        virtiofs_tag,
    })
}

fn decode_filesystem(value: &Yaml) -> Result<UserFilesystemSpec> {
    if let Some(raw) = value.as_str() {
        let spec = UserFilesystemSpec {
            filesystem: UserVolumeFilesystem::parse(raw)?,
            project_quota_support: None,
        };
        spec.validate()?;
        return Ok(spec);
    }
    if value.as_mapping().is_none() {
        return Err(Error::parse(
            "UserVolumeConfig.filesystem must be a mapping or string",
        ));
    }
    let filesystem = match value.get_str("type") {
        Some(raw) => UserVolumeFilesystem::parse(raw)?,
        None => UserVolumeFilesystem::Xfs,
    };
    let project_quota_support = optional_bool(
        value.get("projectQuotaSupport"),
        "UserVolumeConfig.filesystem.projectQuotaSupport",
    )?;
    let spec = UserFilesystemSpec {
        filesystem,
        project_quota_support,
    };
    spec.validate()?;
    Ok(spec)
}

fn decode_volume_selector(root: &Yaml) -> Result<String> {
    let Some(discovery) = root.get("discovery") else {
        return Err(Error::invalid(
            "ExistingVolumeConfig.discovery.volumeSelector.match is required",
        ));
    };
    if discovery.as_mapping().is_none() {
        return Err(Error::parse(
            "ExistingVolumeConfig.discovery must be a mapping",
        ));
    }
    let Some(selector) = discovery.get("volumeSelector") else {
        return Err(Error::invalid(
            "ExistingVolumeConfig.discovery.volumeSelector.match is required",
        ));
    };
    if selector.as_mapping().is_none() {
        return Err(Error::parse(
            "ExistingVolumeConfig.discovery.volumeSelector must be a mapping",
        ));
    }
    Ok(required_string(
        selector,
        "match",
        "ExistingVolumeConfig.discovery.volumeSelector.match",
    )?
    .to_string())
}

fn mapping_present(value: Option<&Yaml>, field: &str) -> Result<bool> {
    match value {
        Some(v) if v.as_mapping().is_some() => Ok(true),
        Some(_) => Err(Error::parse(format!("{field} must be a mapping"))),
        None => Ok(false),
    }
}

fn optional_bool(value: Option<&Yaml>, field: &str) -> Result<Option<bool>> {
    let Some(value) = value else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| Error::parse(format!("{field} must be a boolean")))
}

fn optional_u32(value: Option<&Yaml>, field: &str) -> Result<Option<u32>> {
    let Some(value) = value else {
        return Ok(None);
    };
    scalar(value, field)?
        .parse::<u32>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned integer")))
}

fn optional_u64(value: Option<&Yaml>, field: &str) -> Result<Option<u64>> {
    let Some(value) = value else {
        return Ok(None);
    };
    scalar(value, field)?
        .parse::<u64>()
        .map(Some)
        .map_err(|_| Error::parse(format!("{field} must be an unsigned integer")))
}

fn required_u8(root: &Yaml, key: &str, field: &str) -> Result<u8> {
    let raw = required_string(root, key, field)?;
    raw.parse::<u8>()
        .map_err(|_| Error::parse(format!("{field} must be an unsigned 8-bit integer")))
}

fn decode_string_sequence(value: &Yaml, field: &str) -> Result<Vec<String>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(format!("{field} must be a sequence")));
    };
    let mut out = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        out.push(scalar(item, &format!("{field}[{index}]"))?.to_string());
    }
    Ok(out)
}

fn decode_u8_sequence(value: &Yaml, field: &str) -> Result<Vec<u8>> {
    let Some(items) = value.as_sequence() else {
        return Err(Error::parse(format!("{field} must be a sequence")));
    };
    let mut out = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let raw = scalar(item, &format!("{field}[{index}]"))?;
        out.push(raw.parse::<u8>().map_err(|_| {
            Error::parse(format!(
                "{field}[{index}] must be an unsigned 8-bit integer"
            ))
        })?);
    }
    Ok(out)
}

fn required_string<'a>(root: &'a Yaml, key: &str, field: &str) -> Result<&'a str> {
    let Some(raw) = root.get_str(key) else {
        return Err(Error::invalid(format!("{field} is required")));
    };
    if raw.trim().is_empty() {
        return Err(Error::invalid(format!("{field} is required")));
    }
    Ok(raw.trim())
}

fn scalar<'a>(value: &'a Yaml, field: &str) -> Result<&'a str> {
    value
        .as_str()
        .ok_or_else(|| Error::parse(format!("{field} must be a scalar")))
}

fn parse_byte_size(raw: &str, field: &str) -> Result<u64> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Error::invalid(format!("{field} must be non-empty")));
    }
    if raw.starts_with('-') {
        return Err(Error::invalid(format!("{field} cannot be negative")));
    }
    if raw.ends_with('%') {
        return Err(Error::invalid(format!("{field} cannot be relative")));
    }
    parse_unit_size(raw, field)
}

fn parse_size_limit(raw: &str, field: &str) -> Result<SizeLimit> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Error::invalid(format!("{field} must be non-empty")));
    }
    let (negative, raw) = raw
        .strip_prefix('-')
        .map_or((false, raw), |rest| (true, rest.trim()));
    if let Some(percent) = raw.strip_suffix('%') {
        let value = percent.trim().parse::<f64>().map_err(|_| {
            Error::parse(format!("{field} percent value must be an unsigned number"))
        })?;
        if !(0.0..=100.0).contains(&value) {
            return Err(Error::invalid(format!(
                "{field} percent value {value} is outside 0..=100"
            )));
        }
        let value = value as u64;
        return Ok(if negative {
            SizeLimit::NegativeRelativePercent(value)
        } else {
            SizeLimit::RelativePercent(value)
        });
    }
    if negative {
        return Ok(SizeLimit::NegativeBytes(parse_unit_size(raw, field)?));
    }
    Ok(SizeLimit::Absolute(parse_unit_size(raw, field)?))
}

fn parse_unit_size(raw: &str, field: &str) -> Result<u64> {
    let (number, multiplier) = split_size_suffix(raw);
    parse_decimal_size(number, multiplier, field)
}

fn split_size_suffix(raw: &str) -> (&str, u64) {
    const SUFFIXES: &[(&str, u64)] = &[
        ("KiB", 1024),
        ("MiB", 1024 * 1024),
        ("GiB", 1024 * 1024 * 1024),
        ("TiB", 1024_u64.pow(4)),
        ("KB", 1000),
        ("MB", 1000 * 1000),
        ("GB", 1000 * 1000 * 1000),
        ("TB", 1000_u64.pow(4)),
        ("K", 1000),
        ("M", 1000 * 1000),
        ("G", 1000 * 1000 * 1000),
        ("T", 1000_u64.pow(4)),
    ];
    for (suffix, multiplier) in SUFFIXES {
        if let Some(number) = raw.strip_suffix(suffix) {
            return (number.trim(), *multiplier);
        }
    }
    (raw, 1)
}

fn parse_decimal_size(number: &str, multiplier: u64, field: &str) -> Result<u64> {
    if number.is_empty() {
        return Err(Error::parse(format!("{field} must include a number")));
    }
    let Some((whole, frac)) = number.split_once('.') else {
        let n = number
            .parse::<u64>()
            .map_err(|_| Error::parse(format!("{field} must be an unsigned byte size")))?;
        return n
            .checked_mul(multiplier)
            .ok_or_else(|| Error::invalid(format!("{field} is too large")));
    };
    if whole.is_empty() || frac.is_empty() || frac.contains('.') {
        return Err(Error::parse(format!(
            "{field} must be a valid decimal size"
        )));
    }
    let whole = whole
        .parse::<u64>()
        .map_err(|_| Error::parse(format!("{field} must be an unsigned byte size")))?;
    let frac = frac
        .parse::<u64>()
        .map_err(|_| Error::parse(format!("{field} must be an unsigned byte size")))?;
    let scale = 10_u64
        .checked_pow(frac_digits(number)?)
        .ok_or_else(|| Error::invalid(format!("{field} has too many decimal places")))?;
    let base = whole
        .checked_mul(multiplier)
        .ok_or_else(|| Error::invalid(format!("{field} is too large")))?;
    let extra = frac
        .checked_mul(multiplier)
        .ok_or_else(|| Error::invalid(format!("{field} is too large")))?
        / scale;
    base.checked_add(extra)
        .ok_or_else(|| Error::invalid(format!("{field} is too large")))
}

fn frac_digits(number: &str) -> Result<u32> {
    let Some((_, frac)) = number.split_once('.') else {
        return Ok(0);
    };
    u32::try_from(frac.len()).map_err(|_| Error::invalid("too many decimal places"))
}

fn validate_user_volume_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 34 {
        return Err(Error::invalid(format!(
            "UserVolumeConfig: name {name:?} must be between 1 and 34 characters long"
        )));
    }
    if name.chars().any(|c| {
        !(c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    }) {
        return Err(Error::invalid(
            "UserVolumeConfig: name can only contain lowercase and uppercase ASCII letters, digits, and hyphens",
        ));
    }
    Ok(())
}

fn validate_imported_volume_name(name: &str, kind: &str, max_len: Option<usize>) -> Result<()> {
    if name.is_empty() {
        return Err(Error::invalid(format!("{kind}: name is required")));
    }
    if let Some(max_len) = max_len
        && name.len() > max_len
    {
        return Err(Error::invalid(format!(
            "{kind}: name {name:?} must be between 1 and {max_len} characters long"
        )));
    }
    if name.chars().any(|c| {
        !(c.is_ascii_lowercase() || c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
    }) {
        return Err(Error::invalid(format!(
            "{kind}: name can only contain lowercase and uppercase ASCII letters, digits, and hyphens"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::load_from_bytes;

    const BASE: &str = "\
version: v1alpha1
machine:
  type: worker
";

    fn multidoc(doc: &str) -> String {
        format!("{BASE}---\n{doc}")
    }

    #[test]
    fn volume_config_doc_parses_system_override() {
        let doc = "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  minSize: 1073741824
  maxSize: 2147483648
  grow: true
";
        let parsed = decode_volume_config_body(doc).unwrap();
        assert_eq!(parsed.name, "EPHEMERAL");
        assert_eq!(parsed.provisioning.min_size, Some(1_073_741_824));
        assert_eq!(parsed.provisioning.absolute_max_size(), Some(2_147_483_648));
        assert_eq!(parsed.provisioning.grow, Some(true));
        assert!(!parsed.encryption_configured);
    }

    #[test]
    fn volume_config_doc_parses_source_encryption_key_variants() {
        let doc = "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
encryption:
  provider: luks2
  keys:
    - slot: 0
      nodeID: {}
    - slot: 1
      static:
        passphrase: hunter2
      lockToState: true
    - slot: 2
      kms:
        endpoint: https://kms.example
    - slot: 3
      tpm:
        checkSecurebootStatusOnEnroll: true
        options:
          pcrs:
            - 1
            - 7
    - slot: 4
      tpm: {}
  cipher: aes-xts-plain64
  keySize: 512
  blockSize: 4096
  options:
    - no_read_workqueue
";
        let parsed = decode_volume_config_body(doc).unwrap();
        let encryption = parsed.encryption.unwrap();
        assert_eq!(encryption.provider, "luks2");
        assert_eq!(encryption.cipher.as_deref(), Some("aes-xts-plain64"));
        assert_eq!(encryption.key_size, Some(512));
        assert_eq!(encryption.block_size, Some(4096));
        assert_eq!(encryption.options, vec!["no_read_workqueue"]);
        assert_eq!(encryption.keys.len(), 5);
        assert_eq!(encryption.keys[0].provider, EncryptionKeyProvider::NodeId);
        assert!(encryption.keys[1].lock_to_state);
        assert_eq!(
            encryption.keys[1].provider,
            EncryptionKeyProvider::Static {
                passphrase: "hunter2".to_string(),
            }
        );
        assert_eq!(
            encryption.keys[2].provider,
            EncryptionKeyProvider::Kms {
                endpoint: "https://kms.example".to_string(),
            }
        );
        assert_eq!(
            encryption.keys[3].provider,
            EncryptionKeyProvider::Tpm {
                check_secureboot_status_on_enroll: true,
                pcrs: vec![1, 7],
            }
        );
        assert_eq!(
            encryption.keys[4].provider,
            EncryptionKeyProvider::Tpm {
                check_secureboot_status_on_enroll: false,
                pcrs: vec![7],
            }
        );
    }

    #[test]
    fn volume_config_doc_rejects_invalid_source_encryption() {
        let duplicate = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
encryption:
  provider: luks2
  keys:
    - slot: 0
      nodeID: {}
    - slot: 0
      static:
        passphrase: hunter2
",
        )
        .unwrap_err();
        assert!(duplicate.to_string().contains("duplicate slot"));

        let out_of_range_pcr = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
encryption:
  provider: luks2
  keys:
    - slot: 0
      tpm:
        options:
          pcrs:
            - 24
",
        )
        .unwrap_err();
        assert!(out_of_range_pcr.to_string().contains("out-of-range PCR"));
    }

    #[test]
    fn encryption_meta_decodes_source_block_json_shape() {
        let meta = r#"{
            "EncryptionProvider":"luks2",
            "EncryptionKeys":[
                {
                    "KeySlot":0,
                    "KeyStatic":{"KeyData":"state-secret"},
                    "KeyLockToSTATE":true
                },
                {
                    "KeySlot":1,
                    "KeyTPM":{
                        "TPMCheckSecurebootStatusOnEnroll":true,
                        "TPMOptions":{"PCRs":[1,7]}
                    }
                }
            ],
            "EncryptionCipher":"aes-xts-plain64",
            "EncryptionKeySize":512,
            "EncryptionBlockSize":4096,
            "EncryptionPerfOptions":["no_read_workqueue"]
        }"#;

        let encryption = decode_encryption_meta_value(meta).unwrap().unwrap();

        assert_eq!(encryption.provider, "luks2");
        assert_eq!(encryption.cipher.as_deref(), Some("aes-xts-plain64"));
        assert_eq!(encryption.key_size, Some(512));
        assert_eq!(encryption.block_size, Some(4096));
        assert_eq!(encryption.options, vec!["no_read_workqueue"]);
        assert_eq!(encryption.keys.len(), 2);
        assert!(encryption.keys[0].lock_to_state);
        assert_eq!(
            encryption.keys[0].provider,
            EncryptionKeyProvider::Static {
                passphrase: "state-secret".to_string(),
            }
        );
        assert_eq!(
            encryption.keys[1].provider,
            EncryptionKeyProvider::Tpm {
                check_secureboot_status_on_enroll: true,
                pcrs: vec![1, 7],
            }
        );
    }

    #[test]
    fn encryption_meta_decodes_legacy_v1alpha1_json_shape() {
        let meta = r#"{
            "EncryptionProvider":"luks2",
            "EncryptionKeys":[
                {"KeySlot":0,"KeyNodeID":{}},
                {"KeySlot":1,"KeyKMS":{"KMSEndpoint":"https://kms.example"}},
                {"KeySlot":2,"KeyTPM":{}}
            ]
        }"#;

        let encryption = decode_encryption_meta_value(meta).unwrap().unwrap();

        assert_eq!(encryption.keys[0].provider, EncryptionKeyProvider::NodeId);
        assert_eq!(
            encryption.keys[1].provider,
            EncryptionKeyProvider::Kms {
                endpoint: "https://kms.example".to_string(),
            }
        );
        assert_eq!(
            encryption.keys[2].provider,
            EncryptionKeyProvider::Tpm {
                check_secureboot_status_on_enroll: false,
                pcrs: vec![7],
            }
        );
        assert!(!encryption.keys.iter().any(|key| key.lock_to_state));
    }

    #[test]
    fn encryption_meta_null_or_zero_is_absent() {
        assert_eq!(decode_encryption_meta_value("null").unwrap(), None);
        assert_eq!(decode_encryption_meta_value("{}").unwrap(), None);
    }

    #[test]
    fn user_volume_config_docs_extract_multiple_named_documents() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: Data-01
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 536870912
filesystem:
  type: ext4
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: logs
provisioning:
  diskSelector:
    match: disk.transport == \"sata\"
  maxSize: 1GiB
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = user_volume_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "Data-01");
        assert_eq!(docs[0].volume_id(), "u-Data-01");
        assert_eq!(docs[0].filesystem.filesystem, UserVolumeFilesystem::Ext4);
        assert_eq!(docs[1].name, "logs");
        assert_eq!(docs[1].filesystem.filesystem, UserVolumeFilesystem::Xfs);
        assert_eq!(
            docs[1].provisioning.absolute_max_size(),
            Some(1_073_741_824)
        );
    }

    #[test]
    fn raw_volume_config_docs_extract_multiple_named_documents() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: RawVolumeConfig
name: local-data
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: 5GiB
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: fast
provisioning:
  diskSelector:
    match: disk.transport == \"sata\"
  minSize: 1GiB
  maxSize: 2GiB
  grow: true
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = raw_volume_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "local-data");
        assert_eq!(docs[0].volume_id(), "r-local-data");
        assert_eq!(
            docs[0].provisioning.absolute_max_size(),
            Some(5 * 1024 * 1024 * 1024)
        );
        assert_eq!(docs[1].name, "fast");
        assert_eq!(docs[1].volume_id(), "r-fast");
        assert_eq!(docs[1].provisioning.min_size, Some(1_073_741_824));
        assert_eq!(
            docs[1].provisioning.absolute_max_size(),
            Some(2_147_483_648)
        );
        assert_eq!(docs[1].provisioning.grow, Some(true));
    }

    #[test]
    fn swap_volume_config_docs_extract_multiple_named_documents() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: local-swap
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  maxSize: 5GiB
---
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: fast
provisioning:
  diskSelector:
    match: disk.transport == \"sata\"
  minSize: 1GiB
  maxSize: -2GiB
  grow: true
encryption: {}
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = swap_volume_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "local-swap");
        assert_eq!(docs[0].volume_id(), "s-local-swap");
        assert_eq!(
            docs[0].provisioning.absolute_max_size(),
            Some(5 * 1024 * 1024 * 1024)
        );
        assert!(!docs[0].encryption_configured);
        assert_eq!(docs[1].name, "fast");
        assert_eq!(docs[1].volume_id(), "s-fast");
        assert_eq!(docs[1].provisioning.min_size, Some(1_073_741_824));
        assert_eq!(
            docs[1].provisioning.max_size,
            Some(SizeLimit::NegativeBytes(2_147_483_648))
        );
        assert_eq!(docs[1].provisioning.grow, Some(true));
        assert!(docs[1].encryption_configured);
    }

    #[test]
    fn existing_volume_config_docs_extract_selector_and_mount_options() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: imported-data
discovery:
  volumeSelector:
    match: volume.partition_label == \"MY-DATA\" && disk.serial == \"SERIAL123\"
mount:
  readOnly: true
  disableAccessTime: true
---
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: imported-logs
discovery:
  volumeSelector:
    match: volume.name == \"xfs\"
mount:
  secure: false
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = existing_volume_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "imported-data");
        assert_eq!(docs[0].volume_id(), "e-imported-data");
        assert_eq!(
            docs[0].volume_selector,
            "volume.partition_label == \"MY-DATA\" && disk.serial == \"SERIAL123\""
        );
        assert_eq!(docs[0].mount.read_only, Some(true));
        assert_eq!(docs[0].mount.disable_access_time, Some(true));
        assert!(docs[0].mount.secure_effective());
        assert_eq!(docs[1].mount.secure, Some(false));
        assert!(!docs[1].mount.secure_effective());
    }

    #[test]
    fn external_volume_config_docs_extract_virtiofs_and_mount_options() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: ExternalVolumeConfig
name: shared-data
filesystemType: virtiofs
mount:
  readOnly: true
  disableAccessTime: true
  virtiofs:
    tag: DataShare
---
apiVersion: v1alpha1
kind: ExternalVolumeConfig
name: shared-logs
filesystemType: virtiofs
mount:
  secure: false
  virtiofs:
    tag: LogsShare
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = external_volume_configs(&container).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].name, "shared-data");
        assert_eq!(docs[0].volume_id(), "x-shared-data");
        assert_eq!(docs[0].filesystem.as_str(), "virtiofs");
        assert_eq!(docs[0].mount.virtiofs_tag.as_deref(), Some("DataShare"));
        assert_eq!(docs[0].mount.read_only, Some(true));
        assert_eq!(docs[0].mount.disable_access_time, Some(true));
        assert!(docs[0].mount.secure_effective());
        assert_eq!(docs[1].mount.secure, Some(false));
        assert_eq!(docs[1].mount.virtiofs_tag.as_deref(), Some("LogsShare"));
    }

    #[test]
    fn existing_and_external_volume_config_docs_reject_missing_required_fields() {
        let missing_selector = decode_existing_volume_config_body(
            "\
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: imported-data
",
        );
        assert!(missing_selector.is_err());

        let malformed_selector = decode_existing_volume_config_body(
            "\
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: imported-data
discovery:
  volumeSelector:
    match: volume.partition_label ==
",
        );
        assert!(malformed_selector.is_err());

        let missing_virtiofs = decode_external_volume_config_body(
            "\
apiVersion: v1alpha1
kind: ExternalVolumeConfig
name: shared-data
filesystemType: virtiofs
",
        );
        assert!(missing_virtiofs.is_err());

        let unsupported_external_type = decode_external_volume_config_body(
            "\
apiVersion: v1alpha1
kind: ExternalVolumeConfig
name: shared-data
filesystemType: xfs
mount:
  virtiofs:
    tag: DataShare
",
        );
        assert!(unsupported_external_type.is_err());
    }

    #[test]
    fn existing_volume_config_load_rejects_user_volume_name_conflict() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: shared
discovery:
  volumeSelector:
    match: volume.partition_label == \"SHARED\"
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: shared
volumeType: directory
",
        );
        let err = load_from_bytes(&cfg).unwrap_err();
        assert!(
            err.to_string().contains("ExistingVolumeConfig")
                && err.to_string().contains("UserVolumeConfig")
        );
    }

    #[test]
    fn swap_volume_config_doc_rejects_missing_selector_or_size() {
        let missing_selector = decode_swap_volume_config_body(
            "\
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: swapdata
provisioning:
  minSize: 1
",
        )
        .unwrap_err();
        assert!(missing_selector.to_string().contains("diskSelector"));

        let missing_size = decode_swap_volume_config_body(
            "\
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: swapdata
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
",
        )
        .unwrap_err();
        assert!(missing_size.to_string().contains("minSize or maxSize"));
    }

    #[test]
    fn swap_volume_config_doc_rejects_invalid_names() {
        for name in ["", "swap_data", "contains space"] {
            let doc = format!(
                "\
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: {name}
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 1
"
            );
            assert!(decode_swap_volume_config_body(&doc).is_err(), "{name:?}");
        }
    }

    #[test]
    fn raw_volume_config_doc_rejects_missing_selector_or_size() {
        let missing_selector = decode_raw_volume_config_body(
            "\
apiVersion: v1alpha1
kind: RawVolumeConfig
name: rawdata
provisioning:
  minSize: 1
",
        )
        .unwrap_err();
        assert!(missing_selector.to_string().contains("diskSelector"));

        let missing_size = decode_raw_volume_config_body(
            "\
apiVersion: v1alpha1
kind: RawVolumeConfig
name: rawdata
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
",
        )
        .unwrap_err();
        assert!(missing_size.to_string().contains("minSize or maxSize"));
    }

    #[test]
    fn raw_volume_config_doc_rejects_invalid_names() {
        for name in ["", "raw_data", "contains space"] {
            let doc = format!(
                "\
apiVersion: v1alpha1
kind: RawVolumeConfig
name: {name}
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 1
"
            );
            assert!(decode_raw_volume_config_body(&doc).is_err(), "{name:?}");
        }
    }

    #[test]
    fn volume_config_doc_rejects_unknown_system_volume() {
        let err = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: DATA
provisioning:
  minSize: 1
",
        )
        .unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn volume_config_doc_rejects_state_provisioning() {
        let err = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: STATE
provisioning:
  minSize: 1
",
        )
        .unwrap_err();
        assert!(err.to_string().contains("STATE"));
    }

    #[test]
    fn user_volume_config_doc_rejects_invalid_names() {
        for name in ["", "data_1", "contains space"] {
            let doc = format!(
                "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: {name}
provisioning:
  minSize: 1
"
            );
            assert!(decode_user_volume_config_body(&doc).is_err(), "{name:?}");
        }
        let long = "a".repeat(35);
        let doc = format!(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: {long}
provisioning:
  minSize: 1
"
        );
        assert!(decode_user_volume_config_body(&doc).is_err());
    }

    #[test]
    fn volume_config_doc_rejects_bad_provisioning() {
        let err = decode_user_volume_config_body(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 200
  maxSize: 100
",
        )
        .unwrap_err();
        assert_eq!(err.kind(), "invalid");
    }

    #[test]
    fn user_volume_config_doc_requires_selector_and_size() {
        let missing_selector = decode_user_volume_config_body(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
provisioning:
  minSize: 1
",
        )
        .unwrap_err();
        assert!(missing_selector.to_string().contains("diskSelector"));

        let missing_size = decode_user_volume_config_body(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
",
        )
        .unwrap_err();
        assert!(missing_size.to_string().contains("minSize or maxSize"));
    }

    #[test]
    fn raw_disk_user_volume_config_doc_accepts_selector_only() {
        let doc = decode_user_volume_config_body(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
volumeType: disk
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
filesystem:
  type: ext4
",
        )
        .unwrap();

        assert_eq!(doc.name, "data");
        assert_eq!(doc.volume_id(), "u-data");
        assert_eq!(doc.volume_type, UserVolumeType::Disk);
        assert_eq!(
            doc.provisioning.disk_selector.as_deref(),
            Some("disk.transport == \"nvme\"")
        );
        assert_eq!(doc.provisioning.min_size, None);
        assert_eq!(doc.provisioning.max_size, None);
        assert_eq!(doc.provisioning.grow, None);
        assert_eq!(doc.filesystem.filesystem, UserVolumeFilesystem::Ext4);
    }

    #[test]
    fn raw_disk_user_volume_config_doc_rejects_partition_size_fields() {
        let err = decode_user_volume_config_body(
            "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
volumeType: disk
provisioning:
  diskSelector:
    match: disk.transport == \"nvme\"
  minSize: 1
",
        )
        .unwrap_err();

        assert!(err.to_string().contains("minSize, maxSize and grow"));
    }

    #[test]
    fn user_volume_config_doc_rejects_invalid_disk_selector_cel() {
        for selector in [
            "disk.transport ==",
            "\"not valid CEL\"",
            "disk.size > 10 * GB",
        ] {
            let doc = format!(
                "\
apiVersion: v1alpha1
kind: UserVolumeConfig
name: data
provisioning:
  diskSelector:
    match: {selector}
  minSize: 1
"
            );
            let err = decode_user_volume_config_body(&doc).unwrap_err();
            assert!(err.to_string().contains("diskSelector"), "{selector}");
        }
    }

    #[test]
    fn volume_config_doc_rejects_duplicate_keys_on_load() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  minSize: 1
---
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  minSize: 2
",
        );
        let err = load_from_bytes(&cfg).unwrap_err();
        assert!(err.to_string().contains("duplicate VolumeConfig"));
    }

    #[test]
    fn volume_config_doc_uses_source_imagecache_partition_label() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  minSize: 1
---
apiVersion: v1alpha1
kind: VolumeConfig
name: IMAGECACHE
provisioning:
  minSize: 2
",
        );
        let container = load_from_bytes(&cfg).unwrap();
        let docs = volume_configs(&container).unwrap();
        assert_eq!(
            docs.iter().map(|doc| doc.name.as_str()).collect::<Vec<_>>(),
            vec!["EPHEMERAL", "IMAGECACHE"]
        );
    }

    #[test]
    fn volume_config_doc_rejects_dashed_image_cache_alias() {
        let cfg = multidoc(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: IMAGE-CACHE
provisioning:
  minSize: 2
",
        );
        let err = load_from_bytes(&cfg).unwrap_err();
        assert!(err.to_string().contains("IMAGECACHE"));
        assert!(err.to_string().contains("IMAGE-CACHE"));
    }

    #[test]
    fn volume_config_doc_parses_relative_and_negative_max_size() {
        let relative = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  maxSize: 80%
",
        )
        .unwrap();
        assert_eq!(
            relative.provisioning.max_size,
            Some(SizeLimit::RelativePercent(80))
        );

        let negative = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  maxSize: -1GiB
",
        )
        .unwrap();
        assert_eq!(
            negative.provisioning.max_size,
            Some(SizeLimit::NegativeBytes(1_073_741_824))
        );

        let negative_relative = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  maxSize: -80%
",
        )
        .unwrap();
        assert_eq!(
            negative_relative.provisioning.max_size,
            Some(SizeLimit::NegativeRelativePercent(80))
        );

        let fractional_relative = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  maxSize: 12.9%
",
        )
        .unwrap();
        assert_eq!(
            fractional_relative.provisioning.max_size,
            Some(SizeLimit::RelativePercent(12))
        );

        let too_large_relative = decode_volume_config_body(
            "\
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  maxSize: 101%
",
        )
        .unwrap_err();
        assert_eq!(too_large_relative.kind(), "invalid");
    }
}
