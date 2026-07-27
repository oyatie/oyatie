//! Declarative volume config and observed status with a state machine.
//!
//! Mirrors Talos's `block.VolumeConfig`/`block.VolumeStatus` resources. A
//! volume is a higher-level abstraction than a partition: it describes *what*
//! storage a subsystem needs (a partition matched by label, of a minimum size,
//! optionally encrypted) and the controller drives it through a phase machine
//! from `Waiting` to `Ready`.

use crate::encryption::EncryptionConfig;
use crate::filesystem::FilesystemType;
use crate::mount::BLOCK_NAMESPACE;
use crate::{BlockError, Result};
use os_kernel::{ResourceId, validate_disk_locator_bool_expression};
use os_cosi_domain::{AnyResource, Metadata, Resource, ResourceKind};

/// Talos v1.13.0 block `VolumeStatus` resource type.
///
/// Source: `pkg/machinery/resources/block/volume_status.go`.
pub const VOLUME_STATUS_TYPE: &str = "VolumeStatuses.block.talos.dev";

/// Talos v1.13.0 block `VolumeConfig` resource type.
///
/// Source: `pkg/machinery/resources/block/volume_config.go`.
pub const VOLUME_CONFIG_TYPE: &str = "VolumeConfigs.block.talos.dev";

/// Source `block.WaveSystemDisk` provisioning wave.
///
/// Source: `pkg/machinery/resources/block/volume_config.go`.
pub const WAVE_SYSTEM_DISK: i32 = -1;

/// Source `block.WaveUserVolumes` provisioning wave.
///
/// Source: `pkg/machinery/resources/block/volume_config.go`.
pub const WAVE_USER_VOLUMES: i32 = 0;

fn resource_id(id: impl Into<String>) -> Result<ResourceId> {
    ResourceId::new(id.into()).map_err(|err| BlockError::InvalidDevice(err.to_string()))
}

fn metadata(namespace: &str, kind: &str, id: impl Into<String>) -> Result<Metadata> {
    Ok(Metadata::new(namespace, kind, resource_id(id)?))
}

/// Canonical COSI key for a block `VolumeStatus` id.
pub fn volume_status_key(id: &str) -> Result<String> {
    Ok(metadata(BLOCK_NAMESPACE, VOLUME_STATUS_TYPE, id)?.key())
}

/// Canonical COSI key for a block `VolumeConfig` id.
pub fn volume_config_key(id: &str) -> Result<String> {
    Ok(metadata(BLOCK_NAMESPACE, VOLUME_CONFIG_TYPE, id)?.key())
}

/// The kind of backing storage a volume binds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeType {
    /// A whole disk.
    Disk,
    /// A single partition (matched by label).
    Partition,
    /// A host directory bind-style volume.
    Directory,
    /// An in-memory tmpfs-style volume (no backing device).
    Tmpfs,
    /// An external shared volume source.
    External,
}

impl VolumeType {
    /// Stable source-shaped lowercase spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            VolumeType::Disk => "disk",
            VolumeType::Partition => "partition",
            VolumeType::Directory => "directory",
            VolumeType::Tmpfs => "tmpfs",
            VolumeType::External => "external",
        }
    }

    /// Parse the stable lowercase source spelling.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "disk" => Some(VolumeType::Disk),
            "partition" => Some(VolumeType::Partition),
            "directory" => Some(VolumeType::Directory),
            "tmpfs" => Some(VolumeType::Tmpfs),
            "external" => Some(VolumeType::External),
            _ => None,
        }
    }
}

/// How partition-backed volume resolution chooses among multiple matching
/// discovered partitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionMatchPolicy {
    /// Prefer the smallest matching partition that satisfies the declaration.
    SmallestSufficient,
    /// Preserve discovery order and use the first matching partition.
    FirstMatch,
}

/// Source-shaped `block.VolumeConfig.Provisioning` subset preserved by COSI
/// resources.
///
/// This models the fields Talos controllers write/read through
/// `pkg/machinery/resources/block/volume_config.go::ProvisioningSpec` without
/// requiring privileged provisioning side effects.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VolumeConfigProvisioningSpec {
    /// Source `Provisioning.Wave`.
    pub wave: i32,
    /// Source `Provisioning.DiskSelector.Match`.
    pub disk_selector: Option<String>,
    /// Source `Provisioning.DiskSelector.External`.
    pub external_source: Option<String>,
    /// Source `Provisioning.PartitionSpec.Label`.
    pub label: Option<String>,
    /// Source `Provisioning.PartitionSpec.MinSize`.
    pub min_size: u64,
    /// Source `Provisioning.PartitionSpec.MaxSize`, when absolute.
    pub max_size: Option<u64>,
    /// Source `Provisioning.PartitionSpec.RelativeMaxSize`, when relative.
    pub relative_max_size: Option<u64>,
    /// Source `Provisioning.PartitionSpec.NegativeMaxSize`.
    pub negative_max_size: bool,
    /// Source `Provisioning.PartitionSpec.Grow`.
    pub grow: bool,
    /// Source `Provisioning.PartitionSpec.TypeUUID`.
    pub type_uuid: Option<String>,
    /// Source `Provisioning.FilesystemSpec.Type`.
    pub filesystem: Option<FilesystemType>,
    /// Whether the source resource carried an encryption block.
    pub encryption_configured: bool,
}

/// Source-shaped block `EncryptionSpec` subset preserved by COSI resources.
///
/// Source: `pkg/machinery/resources/block/volume_config.go::EncryptionSpec`
/// and `internal/app/machined/pkg/adapters/block.VolumeConfigSpec.ApplyEncryptionConfig`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VolumeConfigEncryptionSpec {
    /// Source `Encryption.Provider`.
    pub provider: String,
    /// Source `Encryption.Cipher`.
    pub cipher: Option<String>,
    /// Source `Encryption.KeySize`.
    pub key_size: Option<u32>,
    /// Source `Encryption.BlockSize`.
    pub block_size: Option<u64>,
    /// Source `Encryption.PerfOptions`.
    pub perf_options: Vec<String>,
    /// Source encryption keys in declaration order.
    pub keys: Vec<VolumeConfigEncryptionKey>,
}

/// Source block encryption key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeConfigEncryptionKeyType {
    /// Static passphrase key.
    Static,
    /// Node identity derived key.
    NodeId,
    /// KMS endpoint key.
    Kms,
    /// TPM-sealed key.
    Tpm,
}

impl VolumeConfigEncryptionKeyType {
    /// Stable source-shaped spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            VolumeConfigEncryptionKeyType::Static => "static",
            VolumeConfigEncryptionKeyType::NodeId => "nodeID",
            VolumeConfigEncryptionKeyType::Kms => "kms",
            VolumeConfigEncryptionKeyType::Tpm => "tpm",
        }
    }

    /// Parse stable source-shaped spelling.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "static" => Some(Self::Static),
            "nodeID" => Some(Self::NodeId),
            "kms" => Some(Self::Kms),
            "tpm" => Some(Self::Tpm),
            _ => None,
        }
    }
}

/// Source block encryption key subset preserved by COSI resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfigEncryptionKey {
    /// Source LUKS key slot.
    pub slot: u8,
    /// Source key provider type.
    pub key_type: VolumeConfigEncryptionKeyType,
    /// Source `LockToSTATE` bit.
    pub lock_to_state: bool,
    /// Source static passphrase, only for `static` keys.
    pub static_passphrase: Option<String>,
    /// Source KMS endpoint, only for `kms` keys.
    pub kms_endpoint: Option<String>,
    /// Source TPM secure-boot enrollment check, only for `tpm` keys.
    pub tpm_check_secureboot_status_on_enroll: Option<bool>,
    /// Source TPM PCRs, only for `tpm` keys.
    pub tpm_pcrs: Vec<u8>,
    /// Source TPM public-key PCRs, only for `tpm` keys.
    pub tpm_pub_key_pcrs: Vec<u8>,
}

/// Source-shaped `block.VolumeConfig.Mount` subset.
///
/// Source `MountSpec` includes more fields; this generic COSI wrapper preserves
/// the mount labeling, project-quota, target/mode/owner, parent, bind-target,
/// and `secure` fields used by system, image-cache, and user volume mounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfigMountSpec {
    /// Source mount target path.
    pub target_path: String,
    /// Source SELinux label to apply to the mount target, when configured.
    pub selinux_label: Option<String>,
    /// Source XFS project-quota support flag.
    pub project_quota_support: bool,
    /// Source target file/directory mode.
    pub file_mode: u32,
    /// Source mount target owner uid.
    pub uid: u32,
    /// Source mount target owner gid.
    pub gid: u32,
    /// Source secure mount option.
    pub secure: bool,
    /// Source parent mount request id.
    pub parent_id: Option<String>,
    /// Source host bind target path.
    pub bind_target: Option<String>,
}

impl VolumeConfigMountSpec {
    /// Build a source-shaped mount spec subset.
    pub fn new(target_path: impl Into<String>, file_mode: u32, uid: u32, gid: u32) -> Self {
        Self {
            target_path: target_path.into(),
            selinux_label: None,
            project_quota_support: false,
            file_mode,
            uid,
            gid,
            secure: false,
            parent_id: None,
            bind_target: None,
        }
    }

    /// Builder: set the source SELinux mount label.
    pub fn with_selinux_label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.selinux_label = (!label.is_empty()).then_some(label);
        self
    }

    /// Builder: set source XFS project quota support.
    pub fn with_project_quota_support(mut self, enabled: bool) -> Self {
        self.project_quota_support = enabled;
        self
    }

    /// Builder: set the source secure mount option.
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Builder: set source parent mount request id.
    pub fn with_parent_id(mut self, parent_id: impl Into<String>) -> Self {
        let parent_id = parent_id.into();
        self.parent_id = (!parent_id.is_empty()).then_some(parent_id);
        self
    }

    /// Builder: set source host bind target.
    pub fn with_bind_target(mut self, bind_target: impl Into<String>) -> Self {
        let bind_target = bind_target.into();
        self.bind_target = (!bind_target.is_empty()).then_some(bind_target);
        self
    }
}

/// Source-shaped block `VolumeConfig` COSI spec.
///
/// This is intentionally distinct from [`VolumeConfig`], which is the
/// host-safe volume-manager declaration used by local planning. Source
/// `VolumeConfig` has separate locator/provisioning/mount sections, so COSI
/// producers can preserve those fields without overloading the manager model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfigSpec {
    /// COSI resource id / source volume id.
    pub id: String,
    /// Source volume type.
    pub volume_type: VolumeType,
    /// Source `Locator.Match` expression.
    pub locator_match: String,
    /// Source `Locator.DiskMatch` expression.
    pub locator_disk_match: String,
    /// Source provisioning subset.
    pub provisioning: VolumeConfigProvisioningSpec,
    /// Source encryption subset, if configured.
    pub encryption: Option<VolumeConfigEncryptionSpec>,
    /// Source mount subset, if configured.
    pub mount: Option<VolumeConfigMountSpec>,
}

impl VolumeConfigSpec {
    /// Build a source-shaped block `VolumeConfig` spec with zero provisioning
    /// and no mount section.
    pub fn new(
        id: impl Into<String>,
        volume_type: VolumeType,
        locator_match: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            volume_type,
            locator_match: locator_match.into(),
            locator_disk_match: String::new(),
            provisioning: VolumeConfigProvisioningSpec::default(),
            encryption: None,
            mount: None,
        }
    }

    /// Builder: attach a source `Locator.DiskMatch` expression.
    pub fn with_locator_disk_match(mut self, disk_match: impl Into<String>) -> Self {
        self.locator_disk_match = disk_match.into();
        self
    }

    /// Builder: attach provisioning details.
    pub fn with_provisioning(mut self, provisioning: VolumeConfigProvisioningSpec) -> Self {
        self.provisioning = provisioning;
        self
    }

    /// Builder: attach mount details.
    pub fn with_mount(mut self, mount: VolumeConfigMountSpec) -> Self {
        self.mount = Some(mount);
        self
    }

    /// Builder: attach source encryption details.
    pub fn with_encryption(mut self, encryption: VolumeConfigEncryptionSpec) -> Self {
        self.encryption = Some(encryption);
        self.provisioning.encryption_configured = true;
        self
    }
}

/// The lifecycle phase of a volume, mirroring `block.VolumePhase`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumePhase {
    /// Waiting for a matching block device to appear.
    Waiting,
    /// A device matched; the volume is being located/validated.
    Located,
    /// The volume is being provisioned (partitioned/formatted).
    Provisioning,
    /// Encrypted volume is being unlocked / opened.
    Opening,
    /// The volume is formatted, opened and ready to mount.
    Ready,
    /// The volume is torn down and no longer holds backing resources.
    Closed,
    /// The volume entered a terminal failed state.
    Failed,
}

impl VolumePhase {
    /// Stable source-shaped lowercase spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            VolumePhase::Waiting => "waiting",
            VolumePhase::Located => "located",
            VolumePhase::Provisioning => "provisioning",
            VolumePhase::Opening => "opening",
            VolumePhase::Ready => "ready",
            VolumePhase::Closed => "closed",
            VolumePhase::Failed => "failed",
        }
    }

    /// Parse the stable lowercase phase spelling.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "waiting" => Some(VolumePhase::Waiting),
            "located" => Some(VolumePhase::Located),
            "provisioning" => Some(VolumePhase::Provisioning),
            "opening" => Some(VolumePhase::Opening),
            "ready" => Some(VolumePhase::Ready),
            "closed" => Some(VolumePhase::Closed),
            "failed" => Some(VolumePhase::Failed),
            _ => None,
        }
    }

    /// Whether transitioning from `self` to `next` is permitted.
    pub fn can_transition_to(self, next: VolumePhase) -> bool {
        use VolumePhase::*;
        if next == Failed {
            // Any non-terminal phase can fail.
            return !matches!(self, Failed | Closed);
        }
        matches!(
            (self, next),
            (Waiting, Located)
                | (Located, Provisioning)
                | (Located, Opening)
                | (Located, Ready)
                | (Provisioning, Opening)
                | (Provisioning, Ready)
                | (Opening, Ready)
                | (Ready, Waiting) // device removed; re-wait
                | (Waiting, Closed)
                | (Located, Closed)
                | (Provisioning, Closed)
                | (Opening, Closed)
                | (Ready, Closed)
        )
    }

    /// Whether this is a terminal phase.
    pub fn is_terminal(self) -> bool {
        matches!(self, VolumePhase::Closed | VolumePhase::Failed)
    }
}

/// Declarative configuration for a volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfig {
    /// Stable volume identifier, e.g. `EPHEMERAL`.
    pub id: String,
    /// What kind of storage this binds to.
    pub volume_type: VolumeType,
    /// Partition label to match (for [`VolumeType::Partition`]).
    pub match_label: Option<String>,
    /// Talos DiskLocator CEL selector used while provisioning partition-backed
    /// volumes. The current Rust discovery path still matches existing volumes
    /// by label, but the declaration validates and preserves this source field
    /// for the provisioning layer rather than silently dropping it.
    pub disk_selector: Option<String>,
    /// Minimum acceptable size in bytes.
    pub min_size: u64,
    /// Absolute maximum size in bytes, if bounded.
    pub max_size: Option<u64>,
    /// Relative maximum size percentage, if the source configured `maxSize`
    /// as a percentage.
    pub relative_max_size: Option<u64>,
    /// Whether [`max_size`](Self::max_size) or
    /// [`relative_max_size`](Self::relative_max_size) represents space to
    /// leave free on the selected allocatable extent instead of space to
    /// consume. This mirrors Talos `PartitionSpec.NegativeMaxSize`.
    pub negative_max_size: bool,
    /// Whether Talos should grow the volume during provisioning when space is
    /// available. `None` means the source document did not configure the field.
    pub grow: Option<bool>,
    /// The filesystem to format with, if any.
    pub filesystem: Option<FilesystemType>,
    /// Optional encryption configuration.
    pub encryption: Option<EncryptionConfig>,
    /// Partition resolution policy.
    pub partition_match_policy: PartitionMatchPolicy,
}

impl VolumeConfig {
    /// A partition-backed volume matched by label.
    pub fn partition(id: impl Into<String>, label: impl Into<String>, min_size: u64) -> Self {
        VolumeConfig {
            id: id.into(),
            volume_type: VolumeType::Partition,
            match_label: Some(label.into()),
            disk_selector: None,
            min_size,
            max_size: None,
            relative_max_size: None,
            negative_max_size: false,
            grow: None,
            filesystem: Some(FilesystemType::Xfs),
            encryption: None,
            partition_match_policy: PartitionMatchPolicy::SmallestSufficient,
        }
    }

    /// A raw, unformatted partition-backed volume matched by label.
    pub fn raw_partition(id: impl Into<String>, label: impl Into<String>, min_size: u64) -> Self {
        let mut config = Self::partition(id, label, min_size);
        config.filesystem = None;
        config.partition_match_policy = PartitionMatchPolicy::FirstMatch;
        config
    }

    /// A whole-disk volume selected by a Talos DiskLocator expression.
    pub fn disk(id: impl Into<String>, disk_selector: impl Into<String>) -> Self {
        VolumeConfig {
            id: id.into(),
            volume_type: VolumeType::Disk,
            match_label: None,
            disk_selector: Some(disk_selector.into()),
            min_size: 0,
            max_size: None,
            relative_max_size: None,
            negative_max_size: false,
            grow: None,
            filesystem: Some(FilesystemType::Xfs),
            encryption: None,
            partition_match_policy: PartitionMatchPolicy::SmallestSufficient,
        }
    }

    /// Builder: attach encryption.
    pub fn encrypted(mut self, enc: EncryptionConfig) -> Self {
        self.encryption = Some(enc);
        self
    }

    /// Whether this volume requires unlocking before it is ready.
    pub fn is_encrypted(&self) -> bool {
        self.encryption.is_some()
    }

    /// Resolve the source `maxSize` representation against the selected
    /// allocatable extent.
    ///
    /// This is source-guided by Talos v1.13.0
    /// `pkg/machinery/resources/block/volume_config.go::PartitionSpec.ResolveMaxSize`:
    /// a relative value is `available * percent / 100`; a negative value means
    /// "leave that much free"; zero means "unbounded" to the partition
    /// allocation path.
    pub fn resolve_max_size(&self, available_bytes: u64) -> Result<u64> {
        let size = if let Some(relative) = self.relative_max_size {
            available_bytes
                .checked_mul(relative)
                .ok_or_else(|| BlockError::Geometry("relative max size overflows".to_string()))?
                / 100
        } else {
            self.max_size.unwrap_or(0)
        };

        if self.negative_max_size {
            if size > available_bytes {
                return Err(BlockError::Geometry(
                    "partition size cannot be negative".to_string(),
                ));
            }
            return Ok(available_bytes - size);
        }

        Ok(size)
    }

    /// Validate the configuration's internal consistency.
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(BlockError::InvalidDevice("empty volume id".to_string()));
        }
        match self.volume_type {
            VolumeType::Partition => {
                if self.match_label.is_none() {
                    return Err(BlockError::BadTable(
                        "partition volume requires a match label".to_string(),
                    ));
                }
            }
            VolumeType::Disk => {
                if self.match_label.is_some() {
                    return Err(BlockError::BadTable(
                        "disk volume cannot match a partition label".to_string(),
                    ));
                }
                if self.disk_selector.is_none() {
                    return Err(BlockError::InvalidDevice(
                        "disk volume requires a disk selector".to_string(),
                    ));
                }
                if self.min_size != 0
                    || self.max_size.is_some()
                    || self.relative_max_size.is_some()
                    || self.negative_max_size
                    || self.grow.is_some()
                {
                    return Err(BlockError::InvalidDevice(
                        "disk volume does not support min size, max size or grow".to_string(),
                    ));
                }
            }
            VolumeType::External => {
                if self.match_label.is_some() || self.disk_selector.is_some() {
                    return Err(BlockError::BadTable(
                        "external volume cannot match a disk or partition selector".to_string(),
                    ));
                }
                if self.min_size != 0
                    || self.max_size.is_some()
                    || self.relative_max_size.is_some()
                    || self.negative_max_size
                    || self.grow.is_some()
                {
                    return Err(BlockError::InvalidDevice(
                        "external volume does not support min size, max size or grow".to_string(),
                    ));
                }
                if self.encryption.is_some() {
                    return Err(BlockError::InvalidDevice(
                        "external volume does not support encryption".to_string(),
                    ));
                }
            }
            VolumeType::Directory | VolumeType::Tmpfs => {}
        }
        if let Some(selector) = self.disk_selector.as_deref() {
            if selector.trim().is_empty() {
                return Err(BlockError::InvalidDevice(
                    "disk selector cannot be empty".to_string(),
                ));
            }
            validate_disk_locator_bool_expression(selector).map_err(|error| {
                BlockError::InvalidDevice(format!("disk selector is invalid: {error}"))
            })?;
        }
        if self.max_size.is_some() && self.relative_max_size.is_some() {
            return Err(BlockError::Geometry(
                "max size cannot be both absolute and relative".to_string(),
            ));
        }
        if let Some(relative) = self.relative_max_size
            && relative > 100
        {
            return Err(BlockError::Geometry(
                "relative max size greater than 100".to_string(),
            ));
        }
        if self.negative_max_size && self.max_size.is_none() && self.relative_max_size.is_none() {
            return Err(BlockError::Geometry(
                "negative max size requires an absolute or relative max size".to_string(),
            ));
        }
        if let Some(max) = self.max_size
            && !self.negative_max_size
            && self.relative_max_size.is_none()
            && max != 0
            && max < self.min_size
        {
            return Err(BlockError::Geometry(
                "max size smaller than min size".to_string(),
            ));
        }
        if let Some(enc) = &self.encryption {
            enc.validate()?;
        }
        Ok(())
    }

    /// Whether a candidate device of `size` bytes with `label` satisfies this
    /// config's matching rules.
    pub fn matches(&self, label: Option<&str>, size: u64) -> bool {
        if size < self.min_size {
            return false;
        }
        match (&self.match_label, label) {
            (Some(want), Some(have)) => want.eq_ignore_ascii_case(have),
            (Some(_), None) => false,
            (None, _) => true,
        }
    }
}

/// COSI resource form of Talos's block `VolumeConfig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeConfigResource {
    meta: Metadata,
    /// Source-shaped volume configuration.
    pub spec: VolumeConfigSpec,
}

impl VolumeConfigResource {
    /// Build a block `VolumeConfig` resource from a source-shaped spec.
    pub fn new(spec: VolumeConfigSpec) -> Result<Self> {
        let meta = metadata(BLOCK_NAMESPACE, VOLUME_CONFIG_TYPE, spec.id.clone())?;
        Ok(Self { meta, spec })
    }

    /// Kind descriptor for block `VolumeConfig`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(BLOCK_NAMESPACE, VOLUME_CONFIG_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Convert a type-erased COSI resource of the same kind back into the
    /// source-shaped fields currently preserved by the Rust model.
    pub fn from_resource(resource: &dyn Resource) -> Option<Self> {
        if resource.resource_kind() != Self::kind() {
            return None;
        }

        let fingerprint = resource.spec_fingerprint();
        let fields = parse_fingerprint_fields(&fingerprint);
        let id = fields.get("id")?.to_string();
        let volume_type = VolumeType::parse(fields.get("type")?)?;
        let provisioning = VolumeConfigProvisioningSpec {
            wave: parse_field(&fields, "provisioning_wave")?,
            disk_selector: non_empty_field(&fields, "provisioning_disk_selector"),
            external_source: non_empty_field(&fields, "provisioning_external_source"),
            label: non_empty_field(&fields, "provisioning_label"),
            min_size: parse_field(&fields, "provisioning_min_size")?,
            max_size: non_zero_field(&fields, "provisioning_max_size")?,
            relative_max_size: non_zero_field(&fields, "provisioning_relative_max_size")?,
            negative_max_size: parse_field(&fields, "provisioning_negative_max_size")?,
            grow: parse_field(&fields, "provisioning_grow")?,
            type_uuid: non_empty_field(&fields, "provisioning_type_uuid"),
            filesystem: match fields.get("provisioning_filesystem").copied().unwrap_or("") {
                "" => None,
                value => Some(FilesystemType::parse(value).ok()?),
            },
            encryption_configured: parse_field(&fields, "provisioning_encryption_configured")?,
        };
        let encryption = parse_encryption_spec(&fields)?;

        let mount_target = fields.get("mount_target").copied().unwrap_or("");
        let mount_selinux_label = non_empty_field(&fields, "mount_selinux_label");
        let mount_project_quota_support =
            parse_field(&fields, "mount_project_quota_support").unwrap_or(false);
        let mount_file_mode = parse_field(&fields, "mount_file_mode")?;
        let mount_uid = parse_field(&fields, "mount_uid")?;
        let mount_gid = parse_field(&fields, "mount_gid")?;
        let mount_secure = parse_field(&fields, "mount_secure").unwrap_or(false);
        let mount_parent_id = non_empty_field(&fields, "mount_parent_id");
        let mount_bind_target = non_empty_field(&fields, "mount_bind_target");
        let mount = if mount_target.is_empty()
            && mount_file_mode == 0
            && mount_uid == 0
            && mount_gid == 0
            && mount_selinux_label.is_none()
            && !mount_project_quota_support
            && !mount_secure
            && mount_parent_id.is_none()
            && mount_bind_target.is_none()
        {
            None
        } else {
            Some(
                VolumeConfigMountSpec::new(mount_target, mount_file_mode, mount_uid, mount_gid)
                    .with_selinux_label(mount_selinux_label.unwrap_or_default())
                    .with_project_quota_support(mount_project_quota_support)
                    .with_secure(mount_secure)
                    .with_parent_id(mount_parent_id.unwrap_or_default())
                    .with_bind_target(mount_bind_target.unwrap_or_default()),
            )
        };

        Some(Self {
            meta: resource.metadata().clone(),
            spec: VolumeConfigSpec {
                id,
                volume_type,
                locator_match: fields.get("locator_match")?.to_string(),
                locator_disk_match: fields
                    .get("locator_disk_match")
                    .copied()
                    .unwrap_or("")
                    .to_string(),
                provisioning,
                encryption,
                mount,
            },
        })
    }
}

impl Resource for VolumeConfigResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        let mount = self.spec.mount.as_ref();
        let encryption = self.spec.encryption.as_ref();
        let encryption_fields = encryption
            .map(|encryption| {
                format!(
                    ";encryption_provider={};encryption_cipher={};encryption_key_size={};encryption_block_size={};encryption_options={};encryption_keys={}",
                    encryption.provider,
                    encryption.cipher.as_deref().unwrap_or(""),
                    encryption.key_size.unwrap_or(0),
                    encryption.block_size.unwrap_or(0),
                    encode_encryption_options(encryption),
                    encode_encryption_keys(encryption),
                )
            })
            .unwrap_or_default();
        format!(
            "id={};type={};locator_match={};locator_disk_match={};provisioning_wave={};provisioning_disk_selector={};provisioning_external_source={};provisioning_label={};provisioning_min_size={};provisioning_max_size={};provisioning_relative_max_size={};provisioning_negative_max_size={};provisioning_grow={};provisioning_type_uuid={};provisioning_filesystem={};provisioning_encryption_configured={}{};mount_target={};mount_selinux_label={};mount_project_quota_support={};mount_file_mode={};mount_uid={};mount_gid={};mount_secure={};mount_parent_id={};mount_bind_target={}",
            self.spec.id,
            self.spec.volume_type.as_str(),
            self.spec.locator_match,
            self.spec.locator_disk_match,
            self.spec.provisioning.wave,
            self.spec
                .provisioning
                .disk_selector
                .as_deref()
                .unwrap_or(""),
            self.spec
                .provisioning
                .external_source
                .as_deref()
                .unwrap_or(""),
            self.spec.provisioning.label.as_deref().unwrap_or(""),
            self.spec.provisioning.min_size,
            self.spec.provisioning.max_size.unwrap_or(0),
            self.spec.provisioning.relative_max_size.unwrap_or(0),
            self.spec.provisioning.negative_max_size,
            self.spec.provisioning.grow,
            self.spec.provisioning.type_uuid.as_deref().unwrap_or(""),
            self.spec
                .provisioning
                .filesystem
                .map(FilesystemType::as_str)
                .unwrap_or(""),
            self.spec.provisioning.encryption_configured,
            encryption_fields,
            mount.map(|mount| mount.target_path.as_str()).unwrap_or(""),
            mount
                .and_then(|mount| mount.selinux_label.as_deref())
                .unwrap_or(""),
            mount
                .map(|mount| mount.project_quota_support)
                .unwrap_or(false),
            mount.map(|mount| mount.file_mode).unwrap_or(0),
            mount.map(|mount| mount.uid).unwrap_or(0),
            mount.map(|mount| mount.gid).unwrap_or(0),
            mount.map(|mount| mount.secure).unwrap_or(false),
            mount
                .and_then(|mount| mount.parent_id.as_deref())
                .unwrap_or(""),
            mount
                .and_then(|mount| mount.bind_target.as_deref())
                .unwrap_or("")
        )
    }

    fn clone_box(&self) -> AnyResource {
        Box::new(self.clone())
    }
}

/// Observed status of a volume, driven through the phase machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeStatus {
    /// The config being reconciled.
    pub config: VolumeConfig,
    /// Current phase.
    pub phase: VolumePhase,
    /// The device path the volume was located on, once known.
    pub located_on: Option<String>,
    /// A human-readable reason for the current phase (esp. failures).
    pub reason: Option<String>,
    /// Phase to restore from a retryable failure.
    pub pre_fail_phase: Option<VolumePhase>,
    /// Whether a failed status should be retried by the controller.
    pub retryable: bool,
}

impl VolumeStatus {
    /// New status in [`VolumePhase::Waiting`].
    pub fn new(config: VolumeConfig) -> Self {
        VolumeStatus {
            config,
            phase: VolumePhase::Waiting,
            located_on: None,
            reason: None,
            pre_fail_phase: None,
            retryable: false,
        }
    }

    /// Attempt a phase transition, enforcing the state machine.
    pub fn transition(&mut self, next: VolumePhase) -> Result<()> {
        if !self.phase.can_transition_to(next) {
            return Err(BlockError::BadTransition(format!(
                "{:?} -> {:?}",
                self.phase, next
            )));
        }
        self.phase = next;
        Ok(())
    }

    /// Record that the volume was located on `device`, advancing the phase.
    pub fn locate(&mut self, device: impl Into<String>) -> Result<()> {
        self.transition(VolumePhase::Located)?;
        self.located_on = Some(device.into());
        self.reason = None;
        Ok(())
    }

    /// Move the volume to [`VolumePhase::Failed`] with a reason.
    pub fn fail(&mut self, reason: impl Into<String>) -> Result<()> {
        self.pre_fail_phase = Some(self.phase);
        self.retryable = false;
        self.transition(VolumePhase::Failed)?;
        self.reason = Some(reason.into());
        Ok(())
    }

    /// Move the volume to [`VolumePhase::Failed`] with retry metadata.
    pub fn fail_retryable_from(
        &mut self,
        pre_fail_phase: VolumePhase,
        reason: impl Into<String>,
    ) -> Result<()> {
        self.pre_fail_phase = Some(pre_fail_phase);
        self.retryable = true;
        self.transition(VolumePhase::Failed)?;
        self.reason = Some(reason.into());
        Ok(())
    }

    /// Restore the saved pre-failure phase before retrying a retryable error.
    pub fn restore_retry(&mut self) -> bool {
        if self.phase != VolumePhase::Failed || !self.retryable {
            return false;
        }
        self.phase = self.pre_fail_phase.unwrap_or(VolumePhase::Waiting);
        self.pre_fail_phase = None;
        self.retryable = false;
        self.reason = None;
        true
    }

    /// Whether the volume is ready to be mounted.
    pub fn is_ready(&self) -> bool {
        self.phase == VolumePhase::Ready
    }

    /// Drive a located volume to [`VolumePhase::Ready`], stepping through
    /// provisioning and (if encrypted) opening as required.
    pub fn make_ready(&mut self) -> Result<()> {
        if self.phase == VolumePhase::Ready {
            self.reason = None;
            self.pre_fail_phase = None;
            self.retryable = false;
            return Ok(());
        }
        if self.phase == VolumePhase::Waiting {
            return Err(BlockError::BadTransition(
                "volume not yet located".to_string(),
            ));
        }
        if self.phase == VolumePhase::Located {
            self.transition(VolumePhase::Provisioning)?;
        }
        if self.config.is_encrypted() {
            if self.phase == VolumePhase::Provisioning {
                self.transition(VolumePhase::Opening)?;
            }
            self.transition(VolumePhase::Ready)?;
        } else {
            self.transition(VolumePhase::Ready)?;
        }
        self.reason = None;
        self.pre_fail_phase = None;
        self.retryable = false;
        Ok(())
    }

    /// Drive a tearing-down volume to [`VolumePhase::Closed`].
    ///
    /// This mirrors Talos's close automata: close can short-circuit from
    /// not-yet-ready phases, and a retryable close failure first restores the
    /// saved pre-failure phase before the next close attempt.
    pub fn make_closed(&mut self) -> Result<()> {
        if self.phase == VolumePhase::Closed {
            self.reason = None;
            self.pre_fail_phase = None;
            self.retryable = false;
            return Ok(());
        }
        if self.phase == VolumePhase::Failed && self.retryable {
            let _ = self.restore_retry();
        }
        self.transition(VolumePhase::Closed)?;
        self.reason = None;
        self.pre_fail_phase = None;
        self.retryable = false;
        Ok(())
    }
}

/// COSI resource form of Talos's block `VolumeStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeStatusResource {
    meta: Metadata,
    /// Observed volume status.
    pub status: VolumeStatus,
}

impl VolumeStatusResource {
    /// Build a block `VolumeStatus` resource from a host-safe status model.
    pub fn new(status: VolumeStatus) -> Result<Self> {
        let meta = metadata(
            BLOCK_NAMESPACE,
            VOLUME_STATUS_TYPE,
            status.config.id.clone(),
        )?;
        Ok(VolumeStatusResource { meta, status })
    }

    /// Kind descriptor for block `VolumeStatus`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(BLOCK_NAMESPACE, VOLUME_STATUS_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Convert a type-erased COSI resource of the same kind back into the
    /// host-safe status fields used by image-cache planning.
    pub fn from_resource(resource: &dyn Resource) -> Option<Self> {
        if resource.resource_kind() != Self::kind() {
            return None;
        }

        let fingerprint = resource.spec_fingerprint();
        let fields = parse_fingerprint_fields(&fingerprint);
        let volume_id = fields.get("volume_id")?.to_string();
        let phase = VolumePhase::parse(fields.get("phase")?)?;
        let mut status = VolumeStatus::new(VolumeConfig::partition(
            volume_id.clone(),
            volume_id.clone(),
            0,
        ));
        status.phase = phase;
        status.located_on = fields
            .get("located_on")
            .filter(|value| !value.is_empty())
            .map(|value| (*value).to_string());
        status.reason = fields
            .get("reason")
            .filter(|value| !value.is_empty())
            .map(|value| (*value).to_string());
        status.retryable = fields
            .get("retryable")
            .and_then(|value| value.parse().ok())
            .unwrap_or(false);

        Some(VolumeStatusResource {
            meta: resource.metadata().clone(),
            status,
        })
    }
}

impl Resource for VolumeStatusResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "volume_id={};phase={};located_on={};reason={};retryable={}",
            self.status.config.id,
            self.status.phase.as_str(),
            self.status.located_on.as_deref().unwrap_or(""),
            self.status.reason.as_deref().unwrap_or(""),
            self.status.retryable
        )
    }

    fn clone_box(&self) -> AnyResource {
        Box::new(self.clone())
    }
}

fn parse_fingerprint_fields(fingerprint: &str) -> std::collections::BTreeMap<&str, &str> {
    fingerprint
        .split(';')
        .filter_map(|part| part.split_once('='))
        .collect()
}

fn parse_field<T: std::str::FromStr>(
    fields: &std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Option<T> {
    fields.get(key)?.parse().ok()
}

fn non_empty_field(fields: &std::collections::BTreeMap<&str, &str>, key: &str) -> Option<String> {
    fields
        .get(key)
        .filter(|value| !value.is_empty())
        .map(|value| (*value).to_string())
}

fn non_zero_field(
    fields: &std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Option<Option<u64>> {
    let value = parse_field::<u64>(fields, key)?;
    Some((value != 0).then_some(value))
}

fn encode_encryption_options(encryption: &VolumeConfigEncryptionSpec) -> String {
    encryption
        .perf_options
        .iter()
        .map(|option| encode_fingerprint_component(option))
        .collect::<Vec<_>>()
        .join("|")
}

fn encode_encryption_keys(encryption: &VolumeConfigEncryptionSpec) -> String {
    encryption
        .keys
        .iter()
        .map(encode_encryption_key)
        .collect::<Vec<_>>()
        .join("|")
}

fn encode_encryption_key(key: &VolumeConfigEncryptionKey) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        key.slot,
        key.key_type.as_str(),
        key.lock_to_state,
        key.static_passphrase
            .as_deref()
            .map(encode_fingerprint_component)
            .unwrap_or_default(),
        key.kms_endpoint
            .as_deref()
            .map(encode_fingerprint_component)
            .unwrap_or_default(),
        key.tpm_check_secureboot_status_on_enroll
            .map(|value| value.to_string())
            .unwrap_or_default(),
        encode_u8_list(&key.tpm_pcrs),
        encode_u8_list(&key.tpm_pub_key_pcrs)
    )
}

fn encode_u8_list(values: &[u8]) -> String {
    values
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_encryption_spec(
    fields: &std::collections::BTreeMap<&str, &str>,
) -> Option<Option<VolumeConfigEncryptionSpec>> {
    let provider = fields.get("encryption_provider").copied().unwrap_or("");
    if provider.is_empty() {
        return Some(None);
    }

    let cipher = non_empty_field(fields, "encryption_cipher");
    let key_size = non_zero_field(fields, "encryption_key_size")?.map(|value| value as u32);
    let block_size = non_zero_field(fields, "encryption_block_size")?;
    let perf_options =
        parse_component_list(fields.get("encryption_options").copied().unwrap_or(""));
    let keys = parse_encryption_keys(fields.get("encryption_keys").copied().unwrap_or(""))?;

    Some(Some(VolumeConfigEncryptionSpec {
        provider: provider.to_string(),
        cipher,
        key_size,
        block_size,
        perf_options,
        keys,
    }))
}

fn parse_encryption_keys(raw: &str) -> Option<Vec<VolumeConfigEncryptionKey>> {
    if raw.is_empty() {
        return Some(Vec::new());
    }
    raw.split('|').map(parse_encryption_key).collect()
}

fn parse_encryption_key(raw: &str) -> Option<VolumeConfigEncryptionKey> {
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 8 {
        return None;
    }
    let slot = parts[0].parse::<u8>().ok()?;
    let key_type = VolumeConfigEncryptionKeyType::parse(parts[1])?;
    let lock_to_state = parts[2].parse::<bool>().ok()?;
    let static_passphrase = non_empty_component(parts[3]);
    let kms_endpoint = non_empty_component(parts[4]);
    let tpm_check_secureboot_status_on_enroll = if parts[5].is_empty() {
        None
    } else {
        Some(parts[5].parse::<bool>().ok()?)
    };
    Some(VolumeConfigEncryptionKey {
        slot,
        key_type,
        lock_to_state,
        static_passphrase,
        kms_endpoint,
        tpm_check_secureboot_status_on_enroll,
        tpm_pcrs: parse_u8_list(parts[6])?,
        tpm_pub_key_pcrs: parse_u8_list(parts[7])?,
    })
}

fn parse_component_list(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split('|').map(decode_fingerprint_component).collect()
}

fn non_empty_component(raw: &str) -> Option<String> {
    (!raw.is_empty()).then(|| decode_fingerprint_component(raw))
}

fn parse_u8_list(raw: &str) -> Option<Vec<u8>> {
    if raw.is_empty() {
        return Some(Vec::new());
    }
    raw.split(',')
        .map(|value| value.parse::<u8>().ok())
        .collect()
}

fn encode_fingerprint_component(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '%' => encoded.push_str("%25"),
            ';' => encoded.push_str("%3B"),
            '=' => encoded.push_str("%3D"),
            ':' => encoded.push_str("%3A"),
            '|' => encoded.push_str("%7C"),
            ',' => encoded.push_str("%2C"),
            _ => encoded.push(ch),
        }
    }
    encoded
}

fn decode_fingerprint_component(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut decoded = String::with_capacity(raw.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &raw[index + 1..index + 3];
            match hex {
                "25" => {
                    decoded.push('%');
                    index += 3;
                    continue;
                }
                "3B" => {
                    decoded.push(';');
                    index += 3;
                    continue;
                }
                "3D" => {
                    decoded.push('=');
                    index += 3;
                    continue;
                }
                "3A" => {
                    decoded.push(':');
                    index += 3;
                    continue;
                }
                "7C" => {
                    decoded.push('|');
                    index += 3;
                    continue;
                }
                "2C" => {
                    decoded.push(',');
                    index += 3;
                    continue;
                }
                _ => {}
            }
        }
        decoded.push(bytes[index] as char);
        index += 1;
    }
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::{EncryptionKey, KeyProvider};

    fn enc() -> EncryptionConfig {
        let mut c = EncryptionConfig::new();
        c.add_key(
            EncryptionKey::new(
                0,
                KeyProvider::Static {
                    passphrase: "p".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        c
    }

    #[test]
    fn config_validation_and_matching() {
        let cfg = VolumeConfig::partition("EPHEMERAL", "EPHEMERAL", 1024);
        assert!(cfg.validate().is_ok());
        assert!(cfg.matches(Some("ephemeral"), 4096));
        assert!(!cfg.matches(Some("STATE"), 4096));
        assert!(!cfg.matches(Some("EPHEMERAL"), 100)); // too small
        assert!(!cfg.matches(None, 4096)); // label required

        let mut bad = cfg.clone();
        bad.max_size = Some(10);
        assert!(bad.validate().is_err());

        let mut relative = cfg.clone();
        relative.relative_max_size = Some(80);
        assert!(relative.validate().is_ok());

        let mut negative = cfg.clone();
        negative.max_size = Some(10);
        negative.negative_max_size = true;
        assert!(negative.validate().is_ok());

        let mut grow = cfg.clone();
        grow.grow = Some(true);
        assert!(grow.validate().is_ok());

        let mut selected = cfg.clone();
        selected.disk_selector = Some("disk.transport == \"nvme\" && !system_disk".to_string());
        assert!(selected.validate().is_ok());

        let mut invalid_selector = cfg;
        invalid_selector.disk_selector = Some("disk.transport ==".to_string());
        assert!(invalid_selector.validate().is_err());
    }

    #[test]
    fn raw_disk_exact_one_config_requires_selector_and_disallows_partition_size_fields() {
        let cfg = VolumeConfig::disk("u-data", "disk.transport == \"nvme\"");
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.volume_type, VolumeType::Disk);
        assert_eq!(cfg.match_label, None);

        let mut missing_selector = cfg.clone();
        missing_selector.disk_selector = None;
        assert!(matches!(
            missing_selector.validate(),
            Err(BlockError::InvalidDevice(message)) if message.contains("requires a disk selector")
        ));

        let mut sized = cfg;
        sized.min_size = 1;
        assert!(matches!(
            sized.validate(),
            Err(BlockError::InvalidDevice(message)) if message.contains("does not support")
        ));
    }

    #[test]
    fn cosi_volume_status_resource_round_trips_image_cache_planning_fields() {
        let mut status = VolumeStatus::new(VolumeConfig::partition("IMAGECACHE", "IMAGECACHE", 0));
        status.phase = VolumePhase::Ready;
        status.located_on = Some("/dev/sda5".to_string());
        let resource = VolumeStatusResource::new(status).unwrap();

        assert_eq!(
            resource.metadata().key(),
            "runtime/VolumeStatuses.block.talos.dev/IMAGECACHE"
        );
        assert_eq!(
            resource.spec_fingerprint(),
            "volume_id=IMAGECACHE;phase=ready;located_on=/dev/sda5;reason=;retryable=false"
        );

        let parsed = VolumeStatusResource::from_resource(&resource).unwrap();
        assert_eq!(parsed.status.config.id, "IMAGECACHE");
        assert_eq!(parsed.status.phase, VolumePhase::Ready);
        assert_eq!(parsed.status.located_on.as_deref(), Some("/dev/sda5"));
    }

    #[test]
    fn cosi_volume_config_resource_preserves_source_image_cache_fields() {
        let spec = VolumeConfigSpec {
            id: "IMAGECACHE".to_string(),
            volume_type: VolumeType::Partition,
            locator_match: "volume.partition_label == \"IMAGECACHE\"".to_string(),
            locator_disk_match: String::new(),
            provisioning: VolumeConfigProvisioningSpec {
                wave: WAVE_SYSTEM_DISK,
                disk_selector: Some("system_disk".to_string()),
                external_source: None,
                label: Some("IMAGECACHE".to_string()),
                min_size: 500 * 1024 * 1024,
                max_size: Some(1024 * 1024 * 1024),
                relative_max_size: None,
                negative_max_size: false,
                grow: false,
                type_uuid: Some(crate::layout::type_guid::LINUX_FILESYSTEM.to_string()),
                filesystem: Some(FilesystemType::Ext4),
                encryption_configured: false,
            },
            encryption: None,
            mount: Some(VolumeConfigMountSpec::new(
                "/system/imagecache/disk",
                0o700,
                0,
                0,
            )),
        };
        let resource = VolumeConfigResource::new(spec.clone()).unwrap();

        assert_eq!(
            volume_config_key("IMAGECACHE").unwrap(),
            "runtime/VolumeConfigs.block.talos.dev/IMAGECACHE"
        );
        assert_eq!(
            resource.metadata().key(),
            volume_config_key("IMAGECACHE").unwrap()
        );
        assert_eq!(
            VolumeConfigResource::kind(),
            ResourceKind::new(BLOCK_NAMESPACE, VOLUME_CONFIG_TYPE)
        );
        assert_eq!(
            resource.spec_fingerprint(),
            "id=IMAGECACHE;type=partition;locator_match=volume.partition_label == \"IMAGECACHE\";locator_disk_match=;provisioning_wave=-1;provisioning_disk_selector=system_disk;provisioning_external_source=;provisioning_label=IMAGECACHE;provisioning_min_size=524288000;provisioning_max_size=1073741824;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=false;provisioning_type_uuid=0fc63daf-8483-4772-8e79-3d69d8477de4;provisioning_filesystem=ext4;provisioning_encryption_configured=false;mount_target=/system/imagecache/disk;mount_selinux_label=;mount_project_quota_support=false;mount_file_mode=448;mount_uid=0;mount_gid=0;mount_secure=false;mount_parent_id=;mount_bind_target="
        );

        let parsed = VolumeConfigResource::from_resource(&resource).unwrap();
        assert_eq!(parsed.spec, spec);
    }

    #[test]
    fn cosi_volume_config_resource_preserves_mount_selinux_project_quota_fields() {
        let spec = VolumeConfigSpec::new(
            "EPHEMERAL",
            VolumeType::Partition,
            "volume.partition_label == \"EPHEMERAL\"",
        )
        .with_mount(
            VolumeConfigMountSpec::new("/var", 0o755, 0, 0)
                .with_selinux_label("system_u:object_r:ephemeral_t:s0")
                .with_project_quota_support(true)
                .with_secure(true),
        );
        let resource = VolumeConfigResource::new(spec.clone()).unwrap();

        assert_eq!(
            resource.spec_fingerprint(),
            "id=EPHEMERAL;type=partition;locator_match=volume.partition_label == \"EPHEMERAL\";locator_disk_match=;provisioning_wave=0;provisioning_disk_selector=;provisioning_external_source=;provisioning_label=;provisioning_min_size=0;provisioning_max_size=0;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=false;provisioning_type_uuid=;provisioning_filesystem=;provisioning_encryption_configured=false;mount_target=/var;mount_selinux_label=system_u:object_r:ephemeral_t:s0;mount_project_quota_support=true;mount_file_mode=493;mount_uid=0;mount_gid=0;mount_secure=true;mount_parent_id=;mount_bind_target="
        );

        let parsed = VolumeConfigResource::from_resource(&resource).unwrap();
        assert_eq!(parsed.spec, spec);
    }

    #[test]
    fn cosi_volume_config_resource_preserves_directory_volume_type() {
        let spec = VolumeConfigSpec::new("STATE", VolumeType::Directory, "").with_mount(
            VolumeConfigMountSpec::new("/system/state", 0o700, 0, 0)
                .with_selinux_label("system_u:object_r:system_state_t:s0"),
        );
        let resource = VolumeConfigResource::new(spec.clone()).unwrap();

        assert_eq!(
            resource.spec_fingerprint(),
            "id=STATE;type=directory;locator_match=;locator_disk_match=;provisioning_wave=0;provisioning_disk_selector=;provisioning_external_source=;provisioning_label=;provisioning_min_size=0;provisioning_max_size=0;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=false;provisioning_type_uuid=;provisioning_filesystem=;provisioning_encryption_configured=false;mount_target=/system/state;mount_selinux_label=system_u:object_r:system_state_t:s0;mount_project_quota_support=false;mount_file_mode=448;mount_uid=0;mount_gid=0;mount_secure=false;mount_parent_id=;mount_bind_target="
        );

        let parsed = VolumeConfigResource::from_resource(&resource).unwrap();
        assert_eq!(parsed.spec, spec);
    }

    #[test]
    fn cosi_volume_config_resource_preserves_user_volume_mount_parent_and_bind_target() {
        let spec = VolumeConfigSpec::new("u-local-data", VolumeType::Directory, "").with_mount(
            VolumeConfigMountSpec::new("local-data", 0o755, 0, 0)
                .with_selinux_label("system_u:object_r:ephemeral_t:s0")
                .with_parent_id("/var/mnt")
                .with_bind_target("local-data"),
        );
        let resource = VolumeConfigResource::new(spec.clone()).unwrap();

        assert_eq!(
            resource.spec_fingerprint(),
            "id=u-local-data;type=directory;locator_match=;locator_disk_match=;provisioning_wave=0;provisioning_disk_selector=;provisioning_external_source=;provisioning_label=;provisioning_min_size=0;provisioning_max_size=0;provisioning_relative_max_size=0;provisioning_negative_max_size=false;provisioning_grow=false;provisioning_type_uuid=;provisioning_filesystem=;provisioning_encryption_configured=false;mount_target=local-data;mount_selinux_label=system_u:object_r:ephemeral_t:s0;mount_project_quota_support=false;mount_file_mode=493;mount_uid=0;mount_gid=0;mount_secure=false;mount_parent_id=/var/mnt;mount_bind_target=local-data"
        );

        let parsed = VolumeConfigResource::from_resource(&resource).unwrap();
        assert_eq!(parsed.spec, spec);
    }

    #[test]
    fn cosi_volume_config_resource_preserves_source_encryption_fields() {
        let spec = VolumeConfigSpec::new(
            "u-encrypted-data",
            VolumeType::Partition,
            "volume.partition_label == \"u-encrypted-data\"",
        )
        .with_provisioning(VolumeConfigProvisioningSpec {
            wave: WAVE_USER_VOLUMES,
            disk_selector: Some("disk.transport == \"nvme\"".to_string()),
            external_source: None,
            label: Some("u-encrypted-data".to_string()),
            min_size: 150 * 1024 * 1024,
            max_size: None,
            relative_max_size: None,
            negative_max_size: false,
            grow: false,
            type_uuid: Some(crate::layout::type_guid::LINUX_FILESYSTEM.to_string()),
            filesystem: Some(FilesystemType::Xfs),
            encryption_configured: true,
        })
        .with_encryption(VolumeConfigEncryptionSpec {
            provider: "luks2".to_string(),
            cipher: Some("aes-xts-plain64".to_string()),
            key_size: Some(512),
            block_size: Some(4096),
            perf_options: vec!["no_read_workqueue".to_string()],
            keys: vec![
                VolumeConfigEncryptionKey {
                    slot: 0,
                    key_type: VolumeConfigEncryptionKeyType::NodeId,
                    lock_to_state: false,
                    static_passphrase: None,
                    kms_endpoint: None,
                    tpm_check_secureboot_status_on_enroll: None,
                    tpm_pcrs: Vec::new(),
                    tpm_pub_key_pcrs: Vec::new(),
                },
                VolumeConfigEncryptionKey {
                    slot: 1,
                    key_type: VolumeConfigEncryptionKeyType::Static,
                    lock_to_state: true,
                    static_passphrase: Some("hunter2".to_string()),
                    kms_endpoint: None,
                    tpm_check_secureboot_status_on_enroll: None,
                    tpm_pcrs: Vec::new(),
                    tpm_pub_key_pcrs: Vec::new(),
                },
                VolumeConfigEncryptionKey {
                    slot: 2,
                    key_type: VolumeConfigEncryptionKeyType::Kms,
                    lock_to_state: false,
                    static_passphrase: None,
                    kms_endpoint: Some("https://kms.example".to_string()),
                    tpm_check_secureboot_status_on_enroll: None,
                    tpm_pcrs: Vec::new(),
                    tpm_pub_key_pcrs: Vec::new(),
                },
                VolumeConfigEncryptionKey {
                    slot: 3,
                    key_type: VolumeConfigEncryptionKeyType::Tpm,
                    lock_to_state: false,
                    static_passphrase: None,
                    kms_endpoint: None,
                    tpm_check_secureboot_status_on_enroll: Some(true),
                    tpm_pcrs: vec![1, 7],
                    tpm_pub_key_pcrs: vec![11],
                },
            ],
        });
        let resource = VolumeConfigResource::new(spec).unwrap();

        assert!(resource.spec_fingerprint().contains(
            "encryption_keys=0:nodeID:false:::::|1:static:true:hunter2::::|2:kms:false::https%3A//kms.example:::|3:tpm:false:::true:1,7:11"
        ));

        let parsed = VolumeConfigResource::from_resource(&resource).unwrap();
        let encryption = parsed.spec.encryption.unwrap();
        assert_eq!(encryption.provider, "luks2");
        assert_eq!(encryption.cipher.as_deref(), Some("aes-xts-plain64"));
        assert_eq!(encryption.key_size, Some(512));
        assert_eq!(encryption.block_size, Some(4096));
        assert_eq!(encryption.perf_options, vec!["no_read_workqueue"]);
        assert_eq!(encryption.keys.len(), 4);
        assert_eq!(
            encryption.keys[0].key_type,
            VolumeConfigEncryptionKeyType::NodeId
        );
        assert_eq!(
            encryption.keys[1].static_passphrase.as_deref(),
            Some("hunter2")
        );
        assert!(encryption.keys[1].lock_to_state);
        assert_eq!(
            encryption.keys[2].kms_endpoint.as_deref(),
            Some("https://kms.example")
        );
        assert_eq!(
            encryption.keys[3].key_type,
            VolumeConfigEncryptionKeyType::Tpm
        );
        assert_eq!(
            encryption.keys[3].tpm_check_secureboot_status_on_enroll,
            Some(true)
        );
        assert_eq!(encryption.keys[3].tpm_pcrs, vec![1, 7]);
        assert_eq!(encryption.keys[3].tpm_pub_key_pcrs, vec![11]);
    }

    #[test]
    fn partition_resolves_relative_and_negative_max_size() {
        let mut relative = VolumeConfig::partition("DATA", "DATA", 1024);
        relative.relative_max_size = Some(80);
        assert_eq!(
            relative.resolve_max_size(10 * 1024 * 1024).unwrap(),
            8 * 1024 * 1024
        );

        let mut negative_bytes = VolumeConfig::partition("DATA", "DATA", 1024);
        negative_bytes.max_size = Some(1024 * 1024);
        negative_bytes.negative_max_size = true;
        assert_eq!(
            negative_bytes.resolve_max_size(10 * 1024 * 1024).unwrap(),
            9 * 1024 * 1024
        );

        let mut negative_relative = VolumeConfig::partition("DATA", "DATA", 1024);
        negative_relative.relative_max_size = Some(10);
        negative_relative.negative_max_size = true;
        assert_eq!(
            negative_relative
                .resolve_max_size(10 * 1024 * 1024)
                .unwrap(),
            9 * 1024 * 1024
        );

        let mut too_negative = VolumeConfig::partition("DATA", "DATA", 1024);
        too_negative.max_size = Some(11 * 1024 * 1024);
        too_negative.negative_max_size = true;
        assert!(matches!(
            too_negative.resolve_max_size(10 * 1024 * 1024),
            Err(BlockError::Geometry(message)) if message.contains("cannot be negative")
        ));
    }

    #[test]
    fn raw_volume_partition_config_is_unformatted_first_match_partition() {
        let cfg = VolumeConfig::raw_partition("r-local-data", "r-local-data", 1024);

        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.volume_type, VolumeType::Partition);
        assert_eq!(cfg.match_label.as_deref(), Some("r-local-data"));
        assert_eq!(cfg.filesystem, None);
        assert_eq!(cfg.partition_match_policy, PartitionMatchPolicy::FirstMatch);
    }

    #[test]
    fn partition_grow_plan_rejects_max_size_smaller_than_min_size() {
        let mut cfg = VolumeConfig::partition("DATA", "DATA", 200);
        cfg.max_size = Some(100);

        let err = cfg.validate();

        assert!(matches!(
            err,
            Err(BlockError::Geometry(message)) if message.contains("max size smaller than min size")
        ));
    }

    #[test]
    fn phase_transitions_enforced() {
        assert!(VolumePhase::Waiting.can_transition_to(VolumePhase::Located));
        assert!(!VolumePhase::Waiting.can_transition_to(VolumePhase::Ready));
        assert!(VolumePhase::Located.can_transition_to(VolumePhase::Failed));
        assert!(!VolumePhase::Failed.can_transition_to(VolumePhase::Failed));
        assert!(VolumePhase::Failed.is_terminal());
        assert!(VolumePhase::Closed.is_terminal());
        assert!(VolumePhase::Ready.can_transition_to(VolumePhase::Closed));
        assert!(!VolumePhase::Closed.can_transition_to(VolumePhase::Ready));
    }

    #[test]
    fn unencrypted_make_ready() {
        let cfg = VolumeConfig::partition("STATE", "STATE", 1024);
        let mut st = VolumeStatus::new(cfg);
        assert!(st.make_ready().is_err()); // not located
        st.locate("/dev/sda2").unwrap();
        st.make_ready().unwrap();
        assert!(st.is_ready());
        assert_eq!(st.located_on.as_deref(), Some("/dev/sda2"));
        st.make_ready().unwrap(); // idempotent
    }

    #[test]
    fn encrypted_make_ready_passes_through_opening() {
        let cfg = VolumeConfig::partition("EPHEMERAL", "EPHEMERAL", 1024).encrypted(enc());
        let mut st = VolumeStatus::new(cfg);
        st.locate("/dev/sda3").unwrap();
        st.make_ready().unwrap();
        assert!(st.is_ready());
    }

    #[test]
    fn failure_is_terminal() {
        let cfg = VolumeConfig::partition("META", "META", 1024);
        let mut st = VolumeStatus::new(cfg);
        st.locate("/dev/sda1").unwrap();
        st.fail("format error").unwrap();
        assert_eq!(st.phase, VolumePhase::Failed);
        assert_eq!(st.reason.as_deref(), Some("format error"));
        assert!(st.make_ready().is_err());
    }

    #[test]
    fn make_closed_short_circuits_from_ready_and_retryable_failure() {
        let cfg = VolumeConfig::partition("STATE", "STATE", 1024);
        let mut st = VolumeStatus::new(cfg);
        st.locate("/dev/sda2").unwrap();
        st.make_ready().unwrap();

        st.make_closed().unwrap();

        assert_eq!(st.phase, VolumePhase::Closed);
        assert!(st.phase.is_terminal());
        st.make_closed().unwrap();
        assert_eq!(st.phase, VolumePhase::Closed);

        let cfg = VolumeConfig::partition("EPHEMERAL", "EPHEMERAL", 1024);
        let mut retry = VolumeStatus::new(cfg);
        retry.locate("/dev/sda3").unwrap();
        retry.make_ready().unwrap();
        retry
            .fail_retryable_from(VolumePhase::Ready, "close interrupted")
            .unwrap();

        retry.make_closed().unwrap();

        assert_eq!(retry.phase, VolumePhase::Closed);
        assert_eq!(retry.pre_fail_phase, None);
        assert!(!retry.retryable);
        assert_eq!(retry.reason, None);
    }
}
