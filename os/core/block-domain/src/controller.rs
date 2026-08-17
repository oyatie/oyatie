//! COSI controllers for Talos block resources.
//!
//! This module ports bounded pieces of
//! `internal/app/machined/pkg/controllers/block` into host-safe COSI adapters.

use std::collections::BTreeMap;

use crate::mount::{
    BLOCK_NAMESPACE, VolumeMountRequestResource, VolumeMountRequestSpec, volume_mount_request_key,
};
use crate::volume::{
    VolumeConfigEncryptionKey, VolumeConfigEncryptionKeyType, VolumeConfigEncryptionSpec,
    VolumeConfigMountSpec, VolumeConfigProvisioningSpec, VolumeConfigResource, VolumeConfigSpec,
    VolumeType, WAVE_SYSTEM_DISK, WAVE_USER_VOLUMES, volume_config_key,
};
use crate::{
    filesystem::FilesystemType,
    layout::{size, type_guid},
};
use os_cosi_domain::{
    Controller, ControllerError, Input, Labels, Output, Phase, ReconcileContext, ReconcileResult,
    ResourceKind, Spec,
};

/// Source name for Talos's block volume-config controller.
///
/// Source: `VolumeConfigController.Name()` returns
/// `block.VolumeConfigController`.
pub const VOLUME_CONFIG_CONTROLLER_NAME: &str = "block.VolumeConfigController";

/// Source root mount point held alive for user volumes.
///
/// Source: `constants.UserVolumeMountPoint`.
pub const USER_VOLUME_MOUNT_POINT: &str = "/var/mnt";

/// Source COSI namespace for active machine config resources.
pub const MACHINE_CONFIG_NAMESPACE: &str = "config";

/// Source COSI type for Talos machine config resources.
pub const MACHINE_CONFIG_TYPE: &str = "MachineConfigs.config.talos.dev";

/// Source id for the active v1alpha1 machine config.
pub const MACHINE_CONFIG_ACTIVE_ID: &str = "v1alpha1";

/// Source runtime namespace used by Talos runtime/block resources.
pub const RUNTIME_NAMESPACE: &str = BLOCK_NAMESPACE;

/// Source COSI type for Talos runtime META-key resources.
pub const META_KEY_TYPE: &str = "MetaKeys.runtime.talos.dev";

/// Source id for `meta.StateEncryptionConfig` as `runtime.MetaKeyTagToID(0x09)`.
pub const STATE_ENCRYPTION_META_KEY_ID: &str = "0x09";

/// Source label key used to identify system block volumes.
///
/// Source: `block.SystemVolumeLabel`.
pub const SYSTEM_VOLUME_LABEL: &str = "talos.dev/system-volume";

/// Source label key used to identify user block volumes.
///
/// Source: `block.UserVolumeLabel`.
pub const USER_VOLUME_LABEL: &str = "talos.dev/user-volume";

/// Source label key used to identify raw block volumes.
///
/// Source: `block.RawVolumeLabel`.
pub const RAW_VOLUME_LABEL: &str = "talos.dev/raw-volume";

/// Source label key used to identify imported existing block volumes.
///
/// Source: `block.ExistingVolumeLabel`.
pub const EXISTING_VOLUME_LABEL: &str = "talos.dev/existing-volume";

/// Source label key used to identify external shared block volumes.
///
/// Source: `block.ExternalVolumeLabel`.
pub const EXTERNAL_VOLUME_LABEL: &str = "talos.dev/external-volume";

/// Source label key used to identify swap block volumes.
///
/// Source: `block.SwapVolumeLabel`.
pub const SWAP_VOLUME_LABEL: &str = "talos.dev/swap-volume";

const VOLUME_OUTPUT_CLEANUP_LABELS: &[&str] = &[
    SYSTEM_VOLUME_LABEL,
    USER_VOLUME_LABEL,
    RAW_VOLUME_LABEL,
    EXISTING_VOLUME_LABEL,
    EXTERNAL_VOLUME_LABEL,
    SWAP_VOLUME_LABEL,
];

/// Source prefix for user volumes.
///
/// Source: `constants.UserVolumePrefix`.
pub const USER_VOLUME_PREFIX: &str = "u-";

/// Source META partition/volume id.
///
/// Source: `constants.MetaPartitionLabel`.
pub const META_VOLUME_ID: &str = "META";

/// Source STATE partition/volume id.
///
/// Source: `constants.StatePartitionLabel`.
pub const STATE_VOLUME_ID: &str = "STATE";

/// Source EPHEMERAL partition/volume id, used to preserve absent-default behavior.
///
/// Source: `constants.EphemeralPartitionLabel`.
pub const EPHEMERAL_VOLUME_ID: &str = "EPHEMERAL";

/// Source EPHEMERAL mount point.
///
/// Source: `constants.EphemeralMountPoint`.
pub const EPHEMERAL_MOUNT_POINT: &str = "/var";

/// Source EPHEMERAL SELinux label.
///
/// Source: `constants.EphemeralSelinuxLabel`.
pub const EPHEMERAL_SELINUX_LABEL: &str = "system_u:object_r:ephemeral_t:s0";

/// Source EPHEMERAL mount mode for config-present branches.
///
/// Source: `GetEphemeralVolumeTransformer` sets `MountSpec.FileMode = 0o755`.
pub const EPHEMERAL_MOUNT_FILE_MODE: u32 = 0o755;

/// Source default EPHEMERAL minimum size from `quirks.PartitionSizes`.
///
/// Source: `EphemeralMinSize()` returns 2 GiB.
pub const EPHEMERAL_DEFAULT_MIN_SIZE: u64 = 2 * 1024 * 1024 * 1024;

/// Source STATE mount point.
///
/// Source: `constants.StateMountPoint`.
pub const STATE_MOUNT_POINT: &str = "/system/state";

/// Source STATE SELinux label.
///
/// Source: `constants.StateSelinuxLabel`.
pub const STATE_SELINUX_LABEL: &str = "system_u:object_r:system_state_t:s0";

/// Source STATE mount mode for the no-config/default branch.
///
/// Source: `manageStateNoConfig` sets `MountSpec.FileMode = 0o700`.
pub const STATE_MOUNT_FILE_MODE: u32 = 0o700;

/// Source `metaMatch()` locator expression for the META partition.
pub const META_DEFAULT_LOCATOR_MATCH: &str = "volume.partition_label == \"META\" && volume.name in [\"\", \"talosmeta\"] && volume.size == 1048576u";

/// Source `labelVolumeMatchAndNonEmpty(STATE)` locator expression.
pub const STATE_DEFAULT_LOCATOR_MATCH: &str =
    "volume.partition_label == \"STATE\" && volume.name != \"\"";

/// Source `noMatch` locator expression used to mark volumes as missing.
///
/// Source: `noMatch = cel.MustExpression(cel.ParseBooleanExpression("false", ...))`.
pub const NO_MATCH_LOCATOR_MATCH: &str = "false";

/// Source `labelVolumeMatch(STATE)` locator expression for config-present STATE.
pub const STATE_CONFIG_PRESENT_LOCATOR_MATCH: &str = "volume.partition_label == \"STATE\"";

/// Source `labelVolumeMatch(EPHEMERAL)` locator expression.
pub const EPHEMERAL_CONFIG_PRESENT_LOCATOR_MATCH: &str = "volume.partition_label == \"EPHEMERAL\"";

/// Source `systemDiskMatch()` disk selector expression.
pub const SYSTEM_DISK_MATCH: &str = "system_disk";

/// Source-shaped runtime mode bits consumed by the block volume-config controller.
///
/// This mirrors the upstream `machinedruntime.Mode` questions the source
/// controller asks (`InContainer()` and `IsAgent()`) without making the block
/// crate depend on the machined crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VolumeConfigRuntimeMode {
    /// Cloud runtime mode.
    Cloud,
    /// Container runtime mode.
    Container,
    /// Metal runtime mode.
    #[default]
    Metal,
    /// Metal agent runtime mode.
    MetalAgent,
}

impl VolumeConfigRuntimeMode {
    /// Source-shaped runtime mode name.
    pub const fn as_str(self) -> &'static str {
        match self {
            VolumeConfigRuntimeMode::Cloud => "cloud",
            VolumeConfigRuntimeMode::Container => "container",
            VolumeConfigRuntimeMode::Metal => "metal",
            VolumeConfigRuntimeMode::MetalAgent => "metal-agent",
        }
    }

    /// Mirror source `Mode.InContainer()`.
    pub const fn in_container(self) -> bool {
        matches!(self, VolumeConfigRuntimeMode::Container)
    }

    /// Mirror source `Mode.IsAgent()`.
    pub const fn is_agent(self) -> bool {
        matches!(self, VolumeConfigRuntimeMode::MetalAgent)
    }
}

/// Host-safe COSI adapter for Talos's block `VolumeConfigController`.
///
/// This bounded slice ports the source declaration surface plus the no-config
/// default reconcile outputs: holding `/var/mnt` alive and publishing the
/// source-shaped META/STATE system `VolumeConfig` resources. Machine-config
/// driven user/system volume synthesis and cleanup of stale resources are
/// intentionally left to later slices.
#[derive(Debug, Clone, Copy, Default)]
pub struct VolumeConfigController {
    runtime_mode: VolumeConfigRuntimeMode,
}

impl VolumeConfigController {
    /// Build a new source-shaped block volume-config controller adapter.
    pub const fn new() -> Self {
        Self::new_for_runtime_mode(VolumeConfigRuntimeMode::Metal)
    }

    /// Build a source-shaped block volume-config controller for Talos container mode.
    ///
    /// Source: `GetStateVolumeTransformer(..., inContainer=true, ...)` and
    /// `GetEphemeralVolumeTransformer(inContainer=true)` project directory
    /// volumes instead of partition-backed system volumes.
    pub const fn new_in_container() -> Self {
        Self::new_for_runtime_mode(VolumeConfigRuntimeMode::Container)
    }

    /// Build a source-shaped block volume-config controller for Talos agent mode.
    ///
    /// Source: `GetSystemVolumeTransformers(..., IsAgent())` flows into
    /// `manageStateNoConfig`, which marks no-config STATE as missing with
    /// `noMatch` while preserving the partition-backed declaration.
    pub const fn new_agent() -> Self {
        Self::new_for_runtime_mode(VolumeConfigRuntimeMode::MetalAgent)
    }

    /// Build a source-shaped block volume-config controller from runtime mode.
    ///
    /// Source Talos carries `V1Alpha1Mode machinedruntime.Mode` on the
    /// controller and passes `InContainer()` / `IsAgent()` into
    /// `GetSystemVolumeTransformers` during each run.
    pub const fn new_for_runtime_mode(runtime_mode: VolumeConfigRuntimeMode) -> Self {
        VolumeConfigController { runtime_mode }
    }
}

impl Controller for VolumeConfigController {
    fn name(&self) -> &str {
        VOLUME_CONFIG_CONTROLLER_NAME
    }

    fn spec(&self) -> Spec {
        Spec::new()
            .with_input(Input::weak(machine_config_kind()).with_id(MACHINE_CONFIG_ACTIVE_ID))
            .with_input(Input::weak(meta_key_kind()).with_id(STATE_ENCRYPTION_META_KEY_ID))
            .with_input(Input::destroy_ready(VolumeMountRequestResource::kind()))
            .with_input(Input::destroy_ready(VolumeConfigResource::kind()))
            .with_output(Output::shared(VolumeConfigResource::kind()))
            .with_output(Output::shared(VolumeMountRequestResource::kind()))
    }

    fn reconcile(&mut self, ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
        upsert_user_volume_root_mount_request(ctx)?;
        let active_machine_config = active_machine_config(ctx)?;
        let state_encryption_meta = state_encryption_meta(ctx)?;
        let mut existing_outputs = existing_labeled_volume_outputs(ctx);

        upsert_system_volume_configs(
            ctx,
            active_machine_config.as_ref(),
            state_encryption_meta.as_ref(),
            self.runtime_mode.in_container(),
            self.runtime_mode.is_agent(),
            &mut existing_outputs,
        )?;
        if let Some(config) = active_machine_config.as_ref() {
            upsert_user_volume_configs(ctx, config, &mut existing_outputs)?;
            upsert_raw_volume_configs(ctx, config, &mut existing_outputs)?;
            upsert_existing_volume_configs(ctx, config, &mut existing_outputs)?;
            upsert_external_volume_configs(ctx, config, &mut existing_outputs)?;
            upsert_swap_volume_configs(ctx, config, &mut existing_outputs)?;
        }
        cleanup_unused_volume_outputs(ctx, existing_outputs)?;
        Ok(())
    }
}

/// Source kind for the active machine config input.
pub fn machine_config_kind() -> ResourceKind {
    ResourceKind::new(MACHINE_CONFIG_NAMESPACE, MACHINE_CONFIG_TYPE)
}

/// Source kind for runtime META-key resources.
pub fn meta_key_kind() -> ResourceKind {
    ResourceKind::new(RUNTIME_NAMESPACE, META_KEY_TYPE)
}

fn upsert_user_volume_root_mount_request(ctx: &mut ReconcileContext<'_>) -> ReconcileResult {
    let desired = root_user_volume_mount_request().map_err(block_error_to_controller)?;

    upsert_volume_mount_request(ctx, desired, None)
}

fn root_user_volume_mount_request() -> crate::Result<VolumeMountRequestResource> {
    VolumeMountRequestResource::new(
        USER_VOLUME_MOUNT_POINT,
        VolumeMountRequestSpec::new(USER_VOLUME_MOUNT_POINT, VOLUME_CONFIG_CONTROLLER_NAME),
    )
}

fn upsert_system_volume_configs(
    ctx: &mut ReconcileContext<'_>,
    active_machine_config: Option<&os_machine_config_domain::Config>,
    state_encryption_meta: Option<&os_machine_config_domain::EncryptionSpec>,
    in_container: bool,
    is_agent: bool,
    existing_outputs: &mut ExistingVolumeOutputs,
) -> ReconcileResult {
    let mut desired = vec![
        default_meta_volume_config().map_err(block_error_to_controller)?,
        state_volume_config(
            active_machine_config,
            state_encryption_meta,
            in_container,
            is_agent,
        )
        .map_err(block_error_to_controller)?,
    ];

    if let Some(config) = active_machine_config {
        desired.push(
            ephemeral_volume_config(config, in_container).map_err(block_error_to_controller)?,
        );
    }

    for resource in desired {
        upsert_desired_volume_config(ctx, existing_outputs, resource)?;
    }

    Ok(())
}

fn upsert_user_volume_configs(
    ctx: &mut ReconcileContext<'_>,
    active_machine_config: &os_machine_config_domain::Config,
    existing_outputs: &mut ExistingVolumeOutputs,
) -> ReconcileResult {
    let docs =
        os_machine_config_domain::user_volume_configs(active_machine_config).map_err(|err| {
            ControllerError::Failed(format!("invalid user volume config document: {err}"))
        })?;

    for doc in docs {
        let volume = user_volume_config(&doc).map_err(block_error_to_controller)?;
        let volume_id = volume.spec.id.clone();
        if !upsert_desired_volume_config(ctx, existing_outputs, volume)? {
            continue;
        }

        let request =
            user_volume_mount_request(&volume_id, &doc).map_err(block_error_to_controller)?;
        upsert_volume_mount_request(ctx, request, Some(USER_VOLUME_LABEL))?;
    }

    Ok(())
}

fn upsert_raw_volume_configs(
    ctx: &mut ReconcileContext<'_>,
    active_machine_config: &os_machine_config_domain::Config,
    existing_outputs: &mut ExistingVolumeOutputs,
) -> ReconcileResult {
    let docs =
        os_machine_config_domain::raw_volume_configs(active_machine_config).map_err(|err| {
            ControllerError::Failed(format!("invalid raw volume config document: {err}"))
        })?;

    for doc in docs {
        let volume = raw_volume_config(&doc).map_err(block_error_to_controller)?;
        upsert_desired_volume_config(ctx, existing_outputs, volume)?;
    }

    Ok(())
}

fn upsert_existing_volume_configs(
    ctx: &mut ReconcileContext<'_>,
    active_machine_config: &os_machine_config_domain::Config,
    existing_outputs: &mut ExistingVolumeOutputs,
) -> ReconcileResult {
    let docs = os_machine_config_domain::existing_volume_configs(active_machine_config).map_err(
        |err| ControllerError::Failed(format!("invalid existing volume config document: {err}")),
    )?;

    for doc in docs {
        let volume = existing_volume_config(&doc).map_err(block_error_to_controller)?;
        let volume_id = volume.spec.id.clone();
        if !upsert_desired_volume_config(ctx, existing_outputs, volume)? {
            continue;
        }

        let request =
            existing_volume_mount_request(&volume_id, &doc).map_err(block_error_to_controller)?;
        upsert_volume_mount_request(ctx, request, Some(EXISTING_VOLUME_LABEL))?;
    }

    Ok(())
}

fn upsert_external_volume_configs(
    ctx: &mut ReconcileContext<'_>,
    active_machine_config: &os_machine_config_domain::Config,
    existing_outputs: &mut ExistingVolumeOutputs,
) -> ReconcileResult {
    let docs = os_machine_config_domain::external_volume_configs(active_machine_config).map_err(
        |err| ControllerError::Failed(format!("invalid external volume config document: {err}")),
    )?;

    for doc in docs {
        let volume = external_volume_config(&doc).map_err(block_error_to_controller)?;
        let volume_id = volume.spec.id.clone();
        if !upsert_desired_volume_config(ctx, existing_outputs, volume)? {
            continue;
        }

        let request =
            external_volume_mount_request(&volume_id, &doc).map_err(block_error_to_controller)?;
        upsert_volume_mount_request(ctx, request, Some(EXTERNAL_VOLUME_LABEL))?;
    }

    Ok(())
}

fn upsert_swap_volume_configs(
    ctx: &mut ReconcileContext<'_>,
    active_machine_config: &os_machine_config_domain::Config,
    existing_outputs: &mut ExistingVolumeOutputs,
) -> ReconcileResult {
    let docs =
        os_machine_config_domain::swap_volume_configs(active_machine_config).map_err(|err| {
            ControllerError::Failed(format!("invalid swap volume config document: {err}"))
        })?;

    for doc in docs {
        let volume = swap_volume_config(&doc).map_err(block_error_to_controller)?;
        let volume_id = volume.spec.id.clone();
        if !upsert_desired_volume_config(ctx, existing_outputs, volume)? {
            continue;
        }

        let request = swap_volume_mount_request(&volume_id).map_err(block_error_to_controller)?;
        upsert_volume_mount_request(ctx, request, Some(SWAP_VOLUME_LABEL))?;
    }

    Ok(())
}

fn default_meta_volume_config() -> crate::Result<VolumeConfigResource> {
    system_volume_config(VolumeConfigSpec::new(
        META_VOLUME_ID,
        VolumeType::Partition,
        META_DEFAULT_LOCATOR_MATCH,
    ))
}

fn state_volume_config(
    active_machine_config: Option<&os_machine_config_domain::Config>,
    state_encryption_meta: Option<&os_machine_config_domain::EncryptionSpec>,
    in_container: bool,
    is_agent: bool,
) -> crate::Result<VolumeConfigResource> {
    if in_container {
        container_state_volume_config()
    } else if let Some(active_machine_config) = active_machine_config {
        config_present_state_volume_config(active_machine_config)
    } else {
        default_state_volume_config(state_encryption_meta, is_agent)
    }
}

fn container_state_volume_config() -> crate::Result<VolumeConfigResource> {
    system_volume_config(
        VolumeConfigSpec::new(STATE_VOLUME_ID, VolumeType::Directory, "").with_mount(
            VolumeConfigMountSpec::new(STATE_MOUNT_POINT, STATE_MOUNT_FILE_MODE, 0, 0)
                .with_selinux_label(STATE_SELINUX_LABEL)
                .with_secure(true),
        ),
    )
}

fn default_state_volume_config(
    state_encryption_meta: Option<&os_machine_config_domain::EncryptionSpec>,
    is_agent: bool,
) -> crate::Result<VolumeConfigResource> {
    let locator_match = if is_agent {
        NO_MATCH_LOCATOR_MATCH
    } else {
        STATE_DEFAULT_LOCATOR_MATCH
    };
    let mut spec = VolumeConfigSpec::new(STATE_VOLUME_ID, VolumeType::Partition, locator_match)
        .with_mount(
            VolumeConfigMountSpec::new(STATE_MOUNT_POINT, STATE_MOUNT_FILE_MODE, 0, 0)
                .with_selinux_label(STATE_SELINUX_LABEL)
                .with_secure(true),
        );

    if let Some(encryption) = state_encryption_meta {
        spec = spec.with_encryption(convert_encryption_spec(encryption)?);
    }

    system_volume_config(spec)
}

fn config_present_state_volume_config(
    active_machine_config: &os_machine_config_domain::Config,
) -> crate::Result<VolumeConfigResource> {
    let extra_volume_config = os_machine_config_domain::volume_configs(active_machine_config)
        .map_err(|err| {
            crate::BlockError::BadTable(format!("invalid volume config document: {err}"))
        })?
        .into_iter()
        .find(|doc| doc.name == STATE_VOLUME_ID);
    let encryption_config = extra_volume_config
        .as_ref()
        .and_then(|doc| doc.encryption.as_ref())
        .or_else(|| {
            active_machine_config
                .core()
                .machine
                .system_disk_encryption
                .get(STATE_VOLUME_ID)
        });
    let encryption = encryption_config.map(convert_encryption_spec).transpose()?;

    let mut spec = VolumeConfigSpec::new(
        STATE_VOLUME_ID,
        VolumeType::Partition,
        STATE_CONFIG_PRESENT_LOCATOR_MATCH,
    )
    .with_provisioning(VolumeConfigProvisioningSpec {
        wave: WAVE_SYSTEM_DISK,
        disk_selector: Some(SYSTEM_DISK_MATCH.to_string()),
        external_source: None,
        label: Some(STATE_VOLUME_ID.to_string()),
        min_size: size::STATE,
        max_size: Some(size::STATE),
        relative_max_size: None,
        negative_max_size: false,
        grow: false,
        type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
        filesystem: Some(FilesystemType::Xfs),
        encryption_configured: extra_volume_config
            .as_ref()
            .is_some_and(|doc| doc.encryption_configured)
            || encryption_config.is_some(),
    })
    .with_mount(
        VolumeConfigMountSpec::new(STATE_MOUNT_POINT, STATE_MOUNT_FILE_MODE, 0, 0)
            .with_selinux_label(STATE_SELINUX_LABEL)
            .with_secure(true),
    );
    if let Some(encryption) = encryption {
        spec = spec.with_encryption(encryption);
    }

    system_volume_config(spec)
}

fn config_present_ephemeral_volume_config(
    active_machine_config: &os_machine_config_domain::Config,
) -> crate::Result<VolumeConfigResource> {
    let extra_volume_config = os_machine_config_domain::volume_configs(active_machine_config)
        .map_err(|err| {
            crate::BlockError::BadTable(format!("invalid volume config document: {err}"))
        })?
        .into_iter()
        .find(|doc| doc.name == EPHEMERAL_VOLUME_ID);

    let mut provisioning = VolumeConfigProvisioningSpec {
        wave: WAVE_SYSTEM_DISK,
        disk_selector: Some(SYSTEM_DISK_MATCH.to_string()),
        external_source: None,
        label: Some(EPHEMERAL_VOLUME_ID.to_string()),
        min_size: EPHEMERAL_DEFAULT_MIN_SIZE,
        max_size: None,
        relative_max_size: None,
        negative_max_size: false,
        grow: true,
        type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
        filesystem: Some(FilesystemType::Xfs),
        encryption_configured: false,
    };
    let mut mount =
        VolumeConfigMountSpec::new(EPHEMERAL_MOUNT_POINT, EPHEMERAL_MOUNT_FILE_MODE, 0, 0)
            .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
            .with_project_quota_support(
                active_machine_config
                    .core()
                    .machine
                    .features
                    .disk_quota_support_enabled(),
            );

    let encryption_config = extra_volume_config
        .as_ref()
        .and_then(|doc| doc.encryption.as_ref())
        .or_else(|| {
            active_machine_config
                .core()
                .machine
                .system_disk_encryption
                .get(EPHEMERAL_VOLUME_ID)
        });
    let encryption = encryption_config.map(convert_encryption_spec).transpose()?;

    if let Some(doc) = extra_volume_config.as_ref() {
        apply_ephemeral_volume_config_override(&mut provisioning, &mut mount, doc);
    }
    if encryption_config.is_some() {
        provisioning.encryption_configured = true;
    }

    let mut spec = VolumeConfigSpec::new(
        EPHEMERAL_VOLUME_ID,
        VolumeType::Partition,
        EPHEMERAL_CONFIG_PRESENT_LOCATOR_MATCH,
    )
    .with_provisioning(provisioning)
    .with_mount(mount);
    if let Some(encryption) = encryption {
        spec = spec.with_encryption(encryption);
    }

    system_volume_config(spec)
}

fn ephemeral_volume_config(
    active_machine_config: &os_machine_config_domain::Config,
    in_container: bool,
) -> crate::Result<VolumeConfigResource> {
    if in_container {
        container_ephemeral_volume_config()
    } else {
        config_present_ephemeral_volume_config(active_machine_config)
    }
}

fn container_ephemeral_volume_config() -> crate::Result<VolumeConfigResource> {
    system_volume_config(
        VolumeConfigSpec::new(EPHEMERAL_VOLUME_ID, VolumeType::Directory, "").with_mount(
            VolumeConfigMountSpec::new(EPHEMERAL_MOUNT_POINT, EPHEMERAL_MOUNT_FILE_MODE, 0, 0)
                .with_selinux_label(EPHEMERAL_SELINUX_LABEL),
        ),
    )
}

fn user_volume_config(
    doc: &os_machine_config_domain::UserVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    match doc.volume_type {
        os_machine_config_domain::UserVolumeType::Directory => user_directory_volume_config(doc),
        os_machine_config_domain::UserVolumeType::Partition => user_partition_volume_config(doc),
        os_machine_config_domain::UserVolumeType::Disk => user_disk_volume_config(doc),
    }
}

fn user_directory_volume_config(
    doc: &os_machine_config_domain::UserVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    labeled_volume_config(
        VolumeConfigSpec::new(doc.volume_id(), VolumeType::Directory, "")
            .with_mount(user_volume_mount_spec(&doc.name).with_bind_target(&doc.name)),
        USER_VOLUME_LABEL,
    )
}

fn user_partition_volume_config(
    doc: &os_machine_config_domain::UserVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    let volume_id = doc.volume_id();
    let mut provisioning = VolumeConfigProvisioningSpec {
        wave: WAVE_USER_VOLUMES,
        disk_selector: Some(
            doc.provisioning
                .disk_selector
                .clone()
                .unwrap_or_else(|| "false".to_string()),
        ),
        external_source: None,
        label: Some(volume_id.clone()),
        min_size: doc
            .provisioning
            .min_size
            .unwrap_or(os_machine_config_domain::MIN_USER_VOLUME_SIZE),
        max_size: None,
        relative_max_size: None,
        negative_max_size: false,
        grow: doc.provisioning.grow.unwrap_or(false),
        type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
        filesystem: Some(user_filesystem_type(doc.filesystem.filesystem)?),
        encryption_configured: doc.encryption_configured,
    };
    if let Some(max_size) = doc.provisioning.max_size {
        apply_size_limit(&mut provisioning, max_size);
    }
    let locator_match = label_volume_match(&volume_id);

    let mut spec = VolumeConfigSpec::new(volume_id, VolumeType::Partition, locator_match)
        .with_provisioning(provisioning)
        .with_mount(
            user_volume_mount_spec(&doc.name)
                .with_project_quota_support(doc.filesystem.project_quota_support.unwrap_or(false)),
        );
    if let Some(encryption) = doc
        .encryption
        .as_ref()
        .map(convert_encryption_spec)
        .transpose()?
    {
        spec = spec.with_encryption(encryption);
    }

    labeled_volume_config(spec, USER_VOLUME_LABEL)
}

fn user_disk_volume_config(
    doc: &os_machine_config_domain::UserVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    let volume_id = doc.volume_id();
    let disk_match = doc
        .provisioning
        .disk_selector
        .clone()
        .unwrap_or_else(|| "false".to_string());

    let mut spec = VolumeConfigSpec::new(volume_id, VolumeType::Disk, "")
        .with_locator_disk_match(disk_match.clone())
        .with_provisioning(VolumeConfigProvisioningSpec {
            wave: WAVE_USER_VOLUMES,
            disk_selector: Some(disk_match),
            external_source: None,
            label: None,
            min_size: 0,
            max_size: None,
            relative_max_size: None,
            negative_max_size: false,
            grow: true,
            type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
            filesystem: Some(user_filesystem_type(doc.filesystem.filesystem)?),
            encryption_configured: doc.encryption_configured,
        })
        .with_mount(
            user_volume_mount_spec(&doc.name)
                .with_project_quota_support(doc.filesystem.project_quota_support.unwrap_or(false)),
        );
    if let Some(encryption) = doc
        .encryption
        .as_ref()
        .map(convert_encryption_spec)
        .transpose()?
    {
        spec = spec.with_encryption(encryption);
    }

    labeled_volume_config(spec, USER_VOLUME_LABEL)
}

fn raw_volume_config(
    doc: &os_machine_config_domain::RawVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    let volume_id = doc.volume_id();
    let mut provisioning = VolumeConfigProvisioningSpec {
        wave: WAVE_USER_VOLUMES,
        disk_selector: Some(
            doc.provisioning
                .disk_selector
                .clone()
                .unwrap_or_else(|| "false".to_string()),
        ),
        external_source: None,
        label: Some(volume_id.clone()),
        min_size: doc
            .provisioning
            .min_size
            .unwrap_or(os_machine_config_domain::MIN_USER_VOLUME_SIZE),
        max_size: None,
        relative_max_size: None,
        negative_max_size: false,
        grow: doc.provisioning.grow.unwrap_or(false),
        type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
        filesystem: None,
        encryption_configured: doc.encryption_configured,
    };
    if let Some(max_size) = doc.provisioning.max_size {
        apply_size_limit(&mut provisioning, max_size);
    }
    let locator_match = label_volume_match(&volume_id);

    let mut spec = VolumeConfigSpec::new(volume_id, VolumeType::Partition, locator_match)
        .with_provisioning(provisioning);
    if let Some(encryption) = doc
        .encryption
        .as_ref()
        .map(convert_encryption_spec)
        .transpose()?
    {
        spec = spec.with_encryption(encryption);
    }

    labeled_volume_config(spec, RAW_VOLUME_LABEL)
}

fn swap_volume_config(
    doc: &os_machine_config_domain::SwapVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    let volume_id = doc.volume_id();
    let mut provisioning = VolumeConfigProvisioningSpec {
        wave: WAVE_USER_VOLUMES,
        disk_selector: Some(
            doc.provisioning
                .disk_selector
                .clone()
                .unwrap_or_else(|| "false".to_string()),
        ),
        external_source: None,
        label: Some(volume_id.clone()),
        min_size: doc
            .provisioning
            .min_size
            .unwrap_or(os_machine_config_domain::MIN_USER_VOLUME_SIZE),
        max_size: None,
        relative_max_size: None,
        negative_max_size: false,
        grow: doc.provisioning.grow.unwrap_or(false),
        type_uuid: Some(type_guid::LINUX_SWAP.to_string()),
        filesystem: Some(FilesystemType::Swap),
        encryption_configured: doc.encryption_configured,
    };
    if let Some(max_size) = doc.provisioning.max_size {
        apply_size_limit(&mut provisioning, max_size);
    }
    let locator_match = label_volume_match(&volume_id);

    let mut spec = VolumeConfigSpec::new(volume_id, VolumeType::Partition, locator_match)
        .with_provisioning(provisioning);
    if let Some(encryption) = doc
        .encryption
        .as_ref()
        .map(convert_encryption_spec)
        .transpose()?
    {
        spec = spec.with_encryption(encryption);
    }

    labeled_volume_config(spec, SWAP_VOLUME_LABEL)
}

fn existing_volume_config(
    doc: &os_machine_config_domain::ExistingVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    labeled_volume_config(
        VolumeConfigSpec::new(
            doc.volume_id(),
            VolumeType::Partition,
            doc.volume_selector.clone(),
        )
        .with_mount(user_volume_mount_spec(&doc.name)),
        EXISTING_VOLUME_LABEL,
    )
}

fn external_volume_config(
    doc: &os_machine_config_domain::ExternalVolumeConfigDoc,
) -> crate::Result<VolumeConfigResource> {
    let virtiofs_tag = doc.mount.virtiofs_tag.as_deref().ok_or_else(|| {
        crate::BlockError::BadTable("ExternalVolumeConfig missing virtiofs tag".to_string())
    })?;

    labeled_volume_config(
        VolumeConfigSpec::new(doc.volume_id(), VolumeType::External, "")
            .with_provisioning(VolumeConfigProvisioningSpec {
                wave: WAVE_USER_VOLUMES,
                disk_selector: None,
                external_source: Some(virtiofs_tag.to_string()),
                label: None,
                min_size: 0,
                max_size: None,
                relative_max_size: None,
                negative_max_size: false,
                grow: false,
                type_uuid: None,
                filesystem: Some(external_filesystem_type(doc.filesystem)),
                encryption_configured: false,
            })
            .with_mount(user_volume_mount_spec(&doc.name)),
        EXTERNAL_VOLUME_LABEL,
    )
}

fn user_volume_mount_spec(name: &str) -> VolumeConfigMountSpec {
    VolumeConfigMountSpec::new(name, EPHEMERAL_MOUNT_FILE_MODE, 0, 0)
        .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
        .with_parent_id(USER_VOLUME_MOUNT_POINT)
}

fn user_filesystem_type(
    filesystem: os_machine_config_domain::UserVolumeFilesystem,
) -> crate::Result<FilesystemType> {
    match filesystem {
        os_machine_config_domain::UserVolumeFilesystem::Xfs => Ok(FilesystemType::Xfs),
        os_machine_config_domain::UserVolumeFilesystem::Ext4 => Ok(FilesystemType::Ext4),
        os_machine_config_domain::UserVolumeFilesystem::Btrfs => Err(crate::BlockError::BadTable(
            "UserVolumeConfig filesystem btrfs is not supported by talos-block yet".to_string(),
        )),
    }
}

fn external_filesystem_type(
    filesystem: os_machine_config_domain::ExternalVolumeFilesystem,
) -> FilesystemType {
    match filesystem {
        os_machine_config_domain::ExternalVolumeFilesystem::Virtiofs => FilesystemType::Virtiofs,
    }
}

fn convert_encryption_spec(
    encryption: &os_machine_config_domain::EncryptionSpec,
) -> crate::Result<VolumeConfigEncryptionSpec> {
    Ok(VolumeConfigEncryptionSpec {
        provider: encryption.provider.clone(),
        cipher: encryption.cipher.clone(),
        key_size: encryption.key_size,
        block_size: encryption.block_size,
        perf_options: encryption.options.clone(),
        keys: encryption
            .keys
            .iter()
            .map(convert_encryption_key)
            .collect::<crate::Result<Vec<_>>>()?,
    })
}

fn convert_encryption_key(
    key: &os_machine_config_domain::EncryptionKeySpec,
) -> crate::Result<VolumeConfigEncryptionKey> {
    let mut out = VolumeConfigEncryptionKey {
        slot: key.slot,
        key_type: VolumeConfigEncryptionKeyType::NodeId,
        lock_to_state: key.lock_to_state,
        static_passphrase: None,
        kms_endpoint: None,
        tpm_check_secureboot_status_on_enroll: None,
        tpm_pcrs: Vec::new(),
        tpm_pub_key_pcrs: Vec::new(),
    };

    match &key.provider {
        os_machine_config_domain::EncryptionKeyProvider::Static { passphrase } => {
            out.key_type = VolumeConfigEncryptionKeyType::Static;
            out.static_passphrase = Some(passphrase.clone());
        }
        os_machine_config_domain::EncryptionKeyProvider::NodeId => {
            out.key_type = VolumeConfigEncryptionKeyType::NodeId;
        }
        os_machine_config_domain::EncryptionKeyProvider::Kms { endpoint } => {
            out.key_type = VolumeConfigEncryptionKeyType::Kms;
            out.kms_endpoint = Some(endpoint.clone());
        }
        os_machine_config_domain::EncryptionKeyProvider::Tpm {
            check_secureboot_status_on_enroll,
            pcrs,
        } => {
            out.key_type = VolumeConfigEncryptionKeyType::Tpm;
            out.tpm_check_secureboot_status_on_enroll = Some(*check_secureboot_status_on_enroll);
            out.tpm_pcrs = pcrs.clone();
            // Source adapter always returns constants.UKIPCR (PCR 11) for
            // TPM public-key PCRs.
            out.tpm_pub_key_pcrs = vec![11];
        }
    }

    Ok(out)
}

fn user_volume_mount_request(
    volume_id: &str,
    doc: &os_machine_config_domain::UserVolumeConfigDoc,
) -> crate::Result<VolumeMountRequestResource> {
    VolumeMountRequestResource::new(
        volume_id,
        VolumeMountRequestSpec::new(volume_id, VOLUME_CONFIG_CONTROLLER_NAME)
            .with_disable_access_time(doc.mount.disable_access_time.unwrap_or(false))
            .with_secure(doc.mount.secure.unwrap_or(false)),
    )
}

fn existing_volume_mount_request(
    volume_id: &str,
    doc: &os_machine_config_domain::ExistingVolumeConfigDoc,
) -> crate::Result<VolumeMountRequestResource> {
    VolumeMountRequestResource::new(
        volume_id,
        VolumeMountRequestSpec::new(volume_id, VOLUME_CONFIG_CONTROLLER_NAME)
            .with_read_only(doc.mount.read_only_effective())
            .with_disable_access_time(doc.mount.disable_access_time_effective())
            .with_secure(doc.mount.secure_effective()),
    )
}

fn external_volume_mount_request(
    volume_id: &str,
    doc: &os_machine_config_domain::ExternalVolumeConfigDoc,
) -> crate::Result<VolumeMountRequestResource> {
    VolumeMountRequestResource::new(
        volume_id,
        VolumeMountRequestSpec::new(volume_id, VOLUME_CONFIG_CONTROLLER_NAME)
            .with_read_only(doc.mount.read_only_effective())
            .with_disable_access_time(doc.mount.disable_access_time_effective())
            .with_secure(doc.mount.secure_effective()),
    )
}

fn swap_volume_mount_request(volume_id: &str) -> crate::Result<VolumeMountRequestResource> {
    VolumeMountRequestResource::new(
        volume_id,
        VolumeMountRequestSpec::new(volume_id, VOLUME_CONFIG_CONTROLLER_NAME),
    )
}

fn apply_ephemeral_volume_config_override(
    provisioning: &mut VolumeConfigProvisioningSpec,
    mount: &mut VolumeConfigMountSpec,
    doc: &os_machine_config_domain::VolumeConfigDoc,
) {
    if let Some(disk_selector) = &doc.provisioning.disk_selector {
        provisioning.disk_selector = Some(disk_selector.clone());
    }
    if let Some(min_size) = doc.provisioning.min_size {
        provisioning.min_size = min_size;
    }
    if let Some(max_size) = doc.provisioning.max_size {
        apply_size_limit(provisioning, max_size);
    }
    if let Some(grow) = doc.provisioning.grow {
        provisioning.grow = grow;
    }
    provisioning.encryption_configured = doc.encryption_configured;
    if let Some(secure) = doc.mount_secure {
        mount.secure = secure;
    }
}

fn apply_size_limit(
    provisioning: &mut VolumeConfigProvisioningSpec,
    max_size: os_machine_config_domain::SizeLimit,
) {
    match max_size {
        os_machine_config_domain::SizeLimit::Absolute(bytes) => {
            provisioning.max_size = Some(bytes);
            provisioning.relative_max_size = None;
            provisioning.negative_max_size = false;
        }
        os_machine_config_domain::SizeLimit::RelativePercent(percent) => {
            provisioning.max_size = None;
            provisioning.relative_max_size = Some(percent);
            provisioning.negative_max_size = false;
        }
        os_machine_config_domain::SizeLimit::NegativeBytes(bytes) => {
            provisioning.max_size = Some(bytes);
            provisioning.relative_max_size = None;
            provisioning.negative_max_size = true;
        }
        os_machine_config_domain::SizeLimit::NegativeRelativePercent(percent) => {
            provisioning.max_size = None;
            provisioning.relative_max_size = Some(percent);
            provisioning.negative_max_size = true;
        }
    }
}

fn label_volume_match(label: &str) -> String {
    format!("volume.partition_label == \"{label}\"")
}

fn system_volume_config(spec: VolumeConfigSpec) -> crate::Result<VolumeConfigResource> {
    labeled_volume_config(spec, SYSTEM_VOLUME_LABEL)
}

fn labeled_volume_config(
    spec: VolumeConfigSpec,
    label: &str,
) -> crate::Result<VolumeConfigResource> {
    let mut resource = VolumeConfigResource::new(spec)?;
    resource.metadata_mut().labels_mut().set(label, "");
    Ok(resource)
}

#[derive(Debug, Default)]
struct ExistingVolumeOutputs {
    volume_configs_by_id: BTreeMap<String, String>,
    volume_mount_requests_by_id: BTreeMap<String, String>,
}

impl ExistingVolumeOutputs {
    fn mark_desired(&mut self, volume_id: &str) {
        self.volume_configs_by_id.remove(volume_id);
        self.volume_mount_requests_by_id.remove(volume_id);
    }

    fn desired_is_tearing_down(&self, ctx: &ReconcileContext<'_>, volume_id: &str) -> bool {
        [
            &self.volume_configs_by_id,
            &self.volume_mount_requests_by_id,
        ]
        .into_iter()
        .filter_map(|resources_by_id| resources_by_id.get(volume_id))
        .filter_map(|key| ctx.get(key))
        .any(|resource| resource.metadata().phase() == Phase::TearingDown)
    }
}

fn existing_labeled_volume_outputs(ctx: &ReconcileContext<'_>) -> ExistingVolumeOutputs {
    ExistingVolumeOutputs {
        volume_configs_by_id: labeled_output_keys_by_id(ctx, &VolumeConfigResource::kind()),
        volume_mount_requests_by_id: labeled_output_keys_by_id(
            ctx,
            &VolumeMountRequestResource::kind(),
        ),
    }
}

fn labeled_output_keys_by_id(
    ctx: &ReconcileContext<'_>,
    kind: &ResourceKind,
) -> BTreeMap<String, String> {
    let mut resources_by_id = BTreeMap::new();

    for label in VOLUME_OUTPUT_CLEANUP_LABELS {
        let mut selector = Labels::new();
        selector.set(*label, "");

        for resource in ctx.list(kind, Some(&selector)) {
            resources_by_id.insert(
                resource.metadata().id().as_str().to_string(),
                resource.metadata().key(),
            );
        }
    }

    resources_by_id
}

fn upsert_desired_volume_config(
    ctx: &mut ReconcileContext<'_>,
    existing_outputs: &mut ExistingVolumeOutputs,
    desired: VolumeConfigResource,
) -> Result<bool, ControllerError> {
    let volume_id = desired.spec.id.clone();

    if existing_outputs.desired_is_tearing_down(ctx, &volume_id) {
        return Ok(false);
    }

    existing_outputs.mark_desired(&volume_id);
    upsert_volume_config(ctx, desired)?;
    Ok(true)
}

fn cleanup_unused_volume_outputs(
    ctx: &mut ReconcileContext<'_>,
    existing_outputs: ExistingVolumeOutputs,
) -> ReconcileResult {
    for key in existing_outputs.volume_configs_by_id.values() {
        cleanup_unused_volume_output(ctx, key)?;
    }

    for key in existing_outputs.volume_mount_requests_by_id.values() {
        cleanup_unused_volume_output(ctx, key)?;
    }

    Ok(())
}

fn cleanup_unused_volume_output(ctx: &mut ReconcileContext<'_>, key: &str) -> ReconcileResult {
    if !ctx.contains(key) {
        return Ok(());
    }

    ctx.teardown(key)?;

    if ctx
        .get(key)
        .is_some_and(|resource| resource.metadata().can_destroy())
    {
        ctx.destroy(key)?;
    }

    Ok(())
}

fn upsert_volume_config(
    ctx: &mut ReconcileContext<'_>,
    desired: VolumeConfigResource,
) -> ReconcileResult {
    let key = volume_config_key(&desired.spec.id).map_err(block_error_to_controller)?;

    if ctx.contains(&key) {
        ctx.modify(&key, |resource| {
            let metadata = resource.metadata().clone();
            let mut updated = desired.clone();
            *updated.metadata_mut() = metadata;
            for label in VOLUME_OUTPUT_CLEANUP_LABELS {
                if desired.metadata().labels().has(label) {
                    updated.metadata_mut().labels_mut().set(*label, "");
                }
            }
            *resource = Box::new(updated);
        })?;
    } else {
        ctx.create(Box::new(desired))?;
    }

    Ok(())
}

fn upsert_volume_mount_request(
    ctx: &mut ReconcileContext<'_>,
    desired: VolumeMountRequestResource,
    label: Option<&str>,
) -> ReconcileResult {
    let key =
        volume_mount_request_key(&desired.spec.volume_id).map_err(block_error_to_controller)?;

    if ctx.contains(&key) {
        ctx.modify(&key, |resource| {
            let metadata = resource.metadata().clone();
            let mut updated = desired.clone();
            *updated.metadata_mut() = metadata;
            if let Some(label) = label {
                updated.metadata_mut().labels_mut().set(label, "");
            }
            *resource = Box::new(updated);
        })?;
    } else {
        let mut desired = desired;
        if let Some(label) = label {
            desired.metadata_mut().labels_mut().set(label, "");
        }
        ctx.create(Box::new(desired))?;
    }

    Ok(())
}

fn active_machine_config(
    ctx: &ReconcileContext<'_>,
) -> Result<Option<os_machine_config_domain::Config>, ControllerError> {
    let key = machine_config_key();
    let Some(resource) = ctx.get(&key) else {
        return Ok(None);
    };

    if resource.resource_kind() != machine_config_kind() {
        return Err(ControllerError::Failed(format!(
            "unexpected active machine config kind {}",
            resource.resource_kind()
        )));
    }

    let contents = machine_config_contents_from_fingerprint(&resource.spec_fingerprint())?;
    let config = os_machine_config_domain::load_from_bytes(&contents).map_err(|err| {
        ControllerError::Failed(format!("error parsing active machine config: {err}"))
    })?;

    Ok(Some(config))
}

fn state_encryption_meta(
    ctx: &ReconcileContext<'_>,
) -> Result<Option<os_machine_config_domain::EncryptionSpec>, ControllerError> {
    let key = state_encryption_meta_key();
    let Some(resource) = ctx.get(&key) else {
        return Ok(None);
    };

    if resource.resource_kind() != meta_key_kind() {
        return Err(ControllerError::Failed(format!(
            "unexpected state encryption meta key kind {}",
            resource.resource_kind()
        )));
    }

    let value = meta_key_value_from_fingerprint(&resource.spec_fingerprint())?;
    os_machine_config_domain::decode_encryption_meta_value(&value).map_err(|err| {
        ControllerError::Failed(format!("error parsing state encryption meta key: {err}"))
    })
}

fn machine_config_key() -> String {
    format!("{MACHINE_CONFIG_NAMESPACE}/{MACHINE_CONFIG_TYPE}/{MACHINE_CONFIG_ACTIVE_ID}")
}

fn state_encryption_meta_key() -> String {
    format!("{RUNTIME_NAMESPACE}/{META_KEY_TYPE}/{STATE_ENCRYPTION_META_KEY_ID}")
}

fn machine_config_contents_from_fingerprint(fingerprint: &str) -> Result<String, ControllerError> {
    let Some(contents_hex) = fingerprint.strip_prefix("contents=") else {
        return Err(ControllerError::Failed(format!(
            "unexpected active machine config fingerprint {fingerprint:?}"
        )));
    };
    let bytes = parse_hex_bytes(contents_hex).map_err(ControllerError::Failed)?;
    String::from_utf8(bytes).map_err(|err| {
        ControllerError::Failed(format!("active machine config is not UTF-8: {err}"))
    })
}

fn meta_key_value_from_fingerprint(fingerprint: &str) -> Result<String, ControllerError> {
    if let Some(value_hex) = fingerprint.strip_prefix("value_hex=") {
        let bytes = parse_hex_bytes(value_hex).map_err(ControllerError::Failed)?;
        return String::from_utf8(bytes).map_err(|err| {
            ControllerError::Failed(format!("state encryption meta key is not UTF-8: {err}"))
        });
    }

    if let Some(value) = fingerprint.strip_prefix("value=") {
        return Ok(value.to_string());
    }

    Err(ControllerError::Failed(format!(
        "unexpected state encryption meta key fingerprint {fingerprint:?}"
    )))
}

fn parse_hex_bytes(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("hex byte string has odd length".to_string());
    }

    (0..s.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&s[index..index + 2], 16)
                .map_err(|err| format!("invalid hex byte at {index}: {err}"))
        })
        .collect()
}

fn block_error_to_controller(error: crate::BlockError) -> ControllerError {
    ControllerError::Failed(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FilesystemType, VolumeConfigMountSpec, VolumeConfigProvisioningSpec, VolumeConfigResource,
        VolumeConfigSpec, VolumeMountRequestResource, VolumeType, WAVE_SYSTEM_DISK,
        layout::{size, type_guid},
        mount::volume_mount_request_key,
        volume::volume_config_key,
    };
    use os_cosi_domain::{Input, InputKind, Metadata, Output, Resource, ResourceKind, Runtime};
    use os_kernel::ResourceId;

    #[test]
    fn volume_config_controller_projects_default_meta_state_configs_without_machine_config() {
        let mut runtime = Runtime::new();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();
        runtime.run().unwrap();

        let meta = volume_config(&runtime, META_VOLUME_ID);
        assert_eq!(
            meta.spec,
            VolumeConfigSpec::new(
                META_VOLUME_ID,
                VolumeType::Partition,
                META_DEFAULT_LOCATOR_MATCH
            )
        );
        assert_eq!(
            meta.spec.provisioning,
            VolumeConfigProvisioningSpec::default()
        );
        assert_eq!(meta.spec.mount, None);
        assert!(
            meta.metadata().labels().has(SYSTEM_VOLUME_LABEL),
            "META is tagged as a source system volume"
        );

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert_eq!(
            state.spec,
            VolumeConfigSpec::new(
                STATE_VOLUME_ID,
                VolumeType::Partition,
                STATE_DEFAULT_LOCATOR_MATCH
            )
            .with_mount(
                VolumeConfigMountSpec::new(STATE_MOUNT_POINT, STATE_MOUNT_FILE_MODE, 0, 0)
                    .with_selinux_label(STATE_SELINUX_LABEL)
                    .with_secure(true)
            )
        );
        assert_eq!(
            state.spec.provisioning,
            VolumeConfigProvisioningSpec::default()
        );
        assert!(
            state.spec.mount.as_ref().expect("STATE mount").secure,
            "source default STATE mount is secure"
        );
        assert!(
            state.metadata().labels().has(SYSTEM_VOLUME_LABEL),
            "STATE is tagged as a source system volume"
        );

        let ephemeral_key = volume_config_key(EPHEMERAL_VOLUME_ID).unwrap();
        assert!(
            runtime.state().get(&ephemeral_key).is_none(),
            "source skips EPHEMERAL when no MachineConfig/machine section exists"
        );

        runtime.run().unwrap();
        let writes_after_converged: usize = runtime.history().iter().map(|r| r.writes).sum();
        assert_eq!(
            writes_after_converged, 0,
            "default system volume projection is idempotent"
        );
    }

    #[test]
    fn volume_config_controller_projects_config_present_state_provisioning_from_machine_config() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert_eq!(
            state.spec.locator_match, "volume.partition_label == \"STATE\"",
            "source config-present STATE uses labelVolumeMatch, not the no-config non-empty locator"
        );
        assert_eq!(
            state.spec.provisioning,
            VolumeConfigProvisioningSpec {
                wave: WAVE_SYSTEM_DISK,
                disk_selector: Some("system_disk".to_string()),
                external_source: None,
                label: Some(STATE_VOLUME_ID.to_string()),
                min_size: size::STATE,
                max_size: Some(size::STATE),
                relative_max_size: None,
                negative_max_size: false,
                grow: false,
                type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
                filesystem: Some(FilesystemType::Xfs),
                encryption_configured: false,
            }
        );
        assert_eq!(
            state.spec.mount,
            Some(
                VolumeConfigMountSpec::new(STATE_MOUNT_POINT, STATE_MOUNT_FILE_MODE, 0, 0)
                    .with_selinux_label(STATE_SELINUX_LABEL)
                    .with_secure(true)
            )
        );
    }

    #[test]
    fn volume_config_controller_projects_config_present_ephemeral_defaults_from_machine_config() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let ephemeral = volume_config(&runtime, EPHEMERAL_VOLUME_ID);
        assert_eq!(
            ephemeral.spec,
            VolumeConfigSpec::new(
                EPHEMERAL_VOLUME_ID,
                VolumeType::Partition,
                EPHEMERAL_CONFIG_PRESENT_LOCATOR_MATCH
            )
            .with_provisioning(VolumeConfigProvisioningSpec {
                wave: WAVE_SYSTEM_DISK,
                disk_selector: Some(SYSTEM_DISK_MATCH.to_string()),
                external_source: None,
                label: Some(EPHEMERAL_VOLUME_ID.to_string()),
                min_size: EPHEMERAL_DEFAULT_MIN_SIZE,
                max_size: None,
                relative_max_size: None,
                negative_max_size: false,
                grow: true,
                type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
                filesystem: Some(FilesystemType::Xfs),
                encryption_configured: false,
            })
            .with_mount(
                VolumeConfigMountSpec::new(EPHEMERAL_MOUNT_POINT, EPHEMERAL_MOUNT_FILE_MODE, 0, 0)
                    .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
            )
        );
        assert!(
            ephemeral.metadata().labels().has(SYSTEM_VOLUME_LABEL),
            "EPHEMERAL is tagged as a source system volume"
        );
    }

    #[test]
    fn volume_config_controller_projects_config_present_ephemeral_volume_config_overrides() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 3221225472
  maxSize: 80%
  grow: false
mount:
  secure: true
encryption: {}
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let ephemeral = volume_config(&runtime, EPHEMERAL_VOLUME_ID);
        assert_eq!(
            ephemeral.spec.provisioning,
            VolumeConfigProvisioningSpec {
                wave: WAVE_SYSTEM_DISK,
                disk_selector: Some("disk.transport == \"nvme\"".to_string()),
                external_source: None,
                label: Some(EPHEMERAL_VOLUME_ID.to_string()),
                min_size: 3 * 1024 * 1024 * 1024,
                max_size: None,
                relative_max_size: Some(80),
                negative_max_size: false,
                grow: false,
                type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
                filesystem: Some(FilesystemType::Xfs),
                encryption_configured: true,
            }
        );
        assert_eq!(
            ephemeral.spec.mount,
            Some(
                VolumeConfigMountSpec::new(EPHEMERAL_MOUNT_POINT, EPHEMERAL_MOUNT_FILE_MODE, 0, 0)
                    .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
                    .with_secure(true)
            )
        );
        assert_eq!(
            ephemeral.spec.locator_match,
            EPHEMERAL_CONFIG_PRESENT_LOCATOR_MATCH
        );
    }

    #[test]
    fn volume_config_controller_projects_ephemeral_project_quota_from_machine_features() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
  features:
    diskQuotaSupport: true
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let ephemeral = volume_config(&runtime, EPHEMERAL_VOLUME_ID);
        let mount = ephemeral.spec.mount.as_ref().expect("EPHEMERAL mount");
        assert_eq!(
            mount.selinux_label.as_deref(),
            Some(EPHEMERAL_SELINUX_LABEL)
        );
        assert!(
            mount.project_quota_support,
            "source EPHEMERAL mount follows machine.features.diskQuotaSupport"
        );
    }

    #[test]
    fn volume_config_controller_projects_container_state_as_directory_without_config() {
        let mut runtime = Runtime::new();
        runtime
            .register(Box::new(VolumeConfigController::new_in_container()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "container controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert_eq!(
            state.spec,
            VolumeConfigSpec::new(STATE_VOLUME_ID, VolumeType::Directory, "").with_mount(
                VolumeConfigMountSpec::new(STATE_MOUNT_POINT, STATE_MOUNT_FILE_MODE, 0, 0)
                    .with_selinux_label(STATE_SELINUX_LABEL)
                    .with_secure(true)
            )
        );
        assert!(
            state.metadata().labels().has(SYSTEM_VOLUME_LABEL),
            "container STATE remains tagged as a source system volume"
        );

        let ephemeral_key = volume_config_key(EPHEMERAL_VOLUME_ID).unwrap();
        assert!(
            runtime.state().get(&ephemeral_key).is_none(),
            "source skips container EPHEMERAL until MachineConfig/machine exists"
        );
    }

    #[test]
    fn volume_config_controller_projects_container_ephemeral_as_directory_ignoring_partition_overrides()
     {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
  features:
    diskQuotaSupport: true
---
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 3221225472
  maxSize: 80%
  grow: false
mount:
  secure: true
encryption: {}
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new_in_container()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "container controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let ephemeral = volume_config(&runtime, EPHEMERAL_VOLUME_ID);
        assert_eq!(
            ephemeral.spec,
            VolumeConfigSpec::new(EPHEMERAL_VOLUME_ID, VolumeType::Directory, "").with_mount(
                VolumeConfigMountSpec::new(EPHEMERAL_MOUNT_POINT, EPHEMERAL_MOUNT_FILE_MODE, 0, 0)
                    .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
            )
        );
        let mount = ephemeral.spec.mount.as_ref().expect("container mount");
        assert!(
            !mount.project_quota_support,
            "source container EPHEMERAL branch does not set ProjectQuotaSupport"
        );
        assert!(
            !mount.secure,
            "source container EPHEMERAL branch ignores partition mount.secure override"
        );
    }

    #[test]
    fn volume_config_controller_projects_user_partition_volume_config_and_mount_request() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: local-data
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 150MiB
  maxSize: 2GiB
  grow: true
filesystem:
  type: ext4
mount:
  disableAccessTime: true
  secure: true
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let user = volume_config(&runtime, "u-local-data");
        assert_eq!(
            user.spec,
            VolumeConfigSpec::new(
                "u-local-data",
                VolumeType::Partition,
                "volume.partition_label == \"u-local-data\""
            )
            .with_provisioning(VolumeConfigProvisioningSpec {
                wave: 0,
                disk_selector: Some("disk.transport == \"nvme\"".to_string()),
                external_source: None,
                label: Some("u-local-data".to_string()),
                min_size: 150 * 1024 * 1024,
                max_size: Some(2 * 1024 * 1024 * 1024),
                relative_max_size: None,
                negative_max_size: false,
                grow: true,
                type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
                filesystem: Some(FilesystemType::Ext4),
                encryption_configured: false,
            })
            .with_mount(
                VolumeConfigMountSpec::new("local-data", 0o755, 0, 0)
                    .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
                    .with_parent_id(USER_VOLUME_MOUNT_POINT)
            )
        );
        assert!(
            user.metadata().labels().has("talos.dev/user-volume"),
            "UserVolumeConfig output must carry the source user-volume label"
        );

        let request = volume_mount_request(&runtime, "u-local-data");
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=u-local-data;requester=block.VolumeConfigController;read_only=false;detached=false;disable_access_time=true;secure=true"
        );
        assert!(
            request.metadata().labels().has("talos.dev/user-volume"),
            "user volume mount request must be labeled for source cleanup selection"
        );
    }

    #[test]
    fn volume_config_controller_projects_user_volume_encryption_spec() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: encrypted-data
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 150MiB
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
  cipher: aes-xts-plain64
  keySize: 512
  blockSize: 4096
  options:
    - no_read_workqueue
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let user = volume_config(&runtime, "u-encrypted-data");
        assert!(
            user.spec.provisioning.encryption_configured,
            "non-empty source encryption block should be reflected in provisioning"
        );
        let fingerprint = user.spec_fingerprint();
        assert!(
            fingerprint.contains("encryption_provider=luks2"),
            "source provider should be projected into the COSI fingerprint: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_cipher=aes-xts-plain64"),
            "source cipher should be projected into the COSI fingerprint: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_key_size=512"),
            "source key size should be projected into the COSI fingerprint: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_block_size=4096"),
            "source block size should be projected into the COSI fingerprint: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_options=no_read_workqueue"),
            "source perf options should be projected into the COSI fingerprint: {fingerprint}"
        );
        assert!(
            fingerprint
                .contains("encryption_keys=0:nodeID:false:::::|1:static:true:hunter2::::|2:kms:false::https%3A//kms.example:::|3:tpm:false:::true:1,7:11"),
            "source key slots/providers should be projected in source order: {fingerprint}"
        );
    }

    #[test]
    fn volume_config_controller_projects_legacy_system_disk_encryption_fallbacks() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
  systemDiskEncryption:
    state:
      provider: luks2
      keys:
        - slot: 0
          static:
            passphrase: state-secret
      cipher: aes-xts-plain64
      keySize: 512
      blockSize: 4096
      options:
        - no_read_workqueue
    ephemeral:
      provider: luks2
      keys:
        - slot: 0
          nodeID: {}
        - slot: 1
          kms:
            endpoint: https://kms.example
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert!(
            state.spec.provisioning.encryption_configured,
            "legacy state encryption should set the provisioning compatibility bit"
        );
        let state_fingerprint = state.spec_fingerprint();
        assert!(
            state_fingerprint.contains("encryption_provider=luks2"),
            "legacy state provider should be projected: {state_fingerprint}"
        );
        assert!(
            state_fingerprint.contains("encryption_cipher=aes-xts-plain64"),
            "legacy state cipher should be projected: {state_fingerprint}"
        );
        assert!(
            state_fingerprint.contains("encryption_key_size=512"),
            "legacy state key size should be projected: {state_fingerprint}"
        );
        assert!(
            state_fingerprint.contains("encryption_block_size=4096"),
            "legacy state block size should be projected: {state_fingerprint}"
        );
        assert!(
            state_fingerprint.contains("encryption_options=no_read_workqueue"),
            "legacy state options should be projected: {state_fingerprint}"
        );
        assert!(
            state_fingerprint.contains("encryption_keys=0:static:false:state-secret::::"),
            "legacy state key should be projected: {state_fingerprint}"
        );

        let ephemeral = volume_config(&runtime, EPHEMERAL_VOLUME_ID);
        assert!(
            ephemeral.spec.provisioning.encryption_configured,
            "legacy ephemeral encryption should set the provisioning compatibility bit"
        );
        let ephemeral_fingerprint = ephemeral.spec_fingerprint();
        assert!(
            ephemeral_fingerprint.contains(
                "encryption_keys=0:nodeID:false:::::|1:kms:false::https%3A//kms.example:::"
            ),
            "legacy ephemeral key variants should be projected: {ephemeral_fingerprint}"
        );
    }

    #[test]
    fn volume_config_controller_prefers_volume_config_encryption_over_legacy_fallback() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
  systemDiskEncryption:
    ephemeral:
      provider: luks2
      keys:
        - slot: 0
          nodeID: {}
---
apiVersion: v1alpha1
kind: VolumeConfig
name: EPHEMERAL
encryption:
  provider: luks2
  keys:
    - slot: 0
      static:
        passphrase: volume-config-secret
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let ephemeral = volume_config(&runtime, EPHEMERAL_VOLUME_ID);
        let fingerprint = ephemeral.spec_fingerprint();
        assert!(
            fingerprint.contains("encryption_keys=0:static:false:volume-config-secret::::"),
            "VolumeConfig encryption should take precedence over legacy fallback: {fingerprint}"
        );
        assert!(
            !fingerprint.contains("nodeID"),
            "legacy fallback must not leak through when VolumeConfig encryption exists: {fingerprint}"
        );
    }

    #[test]
    fn volume_config_controller_projects_no_config_state_encryption_from_meta_key() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(state_encryption_meta_key(
                r#"{
                    "EncryptionProvider":"luks2",
                    "EncryptionKeys":[
                        {
                            "KeySlot":0,
                            "KeyStatic":{"KeyData":"state-meta-secret"},
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
                }"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert!(
            state.spec.provisioning.encryption_configured,
            "source META encryption should set the provisioning compatibility bit"
        );
        assert!(
            state.spec.mount.as_ref().expect("STATE mount").secure,
            "no-config STATE mount remains secure while carrying META encryption"
        );
        let fingerprint = state.spec_fingerprint();
        assert!(
            fingerprint.contains("encryption_provider=luks2"),
            "META provider should be projected into STATE: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_cipher=aes-xts-plain64"),
            "META cipher should be projected into STATE: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_key_size=512"),
            "META key size should be projected into STATE: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_block_size=4096"),
            "META block size should be projected into STATE: {fingerprint}"
        );
        assert!(
            fingerprint.contains("encryption_options=no_read_workqueue"),
            "META perf options should be projected into STATE: {fingerprint}"
        );
        assert!(
            fingerprint.contains(
                "encryption_keys=0:static:true:state-meta-secret::::|1:tpm:false:::true:1,7:11"
            ),
            "META key variants should be projected into STATE: {fingerprint}"
        );

        let ephemeral_key = volume_config_key(EPHEMERAL_VOLUME_ID).unwrap();
        assert!(
            runtime.state().get(&ephemeral_key).is_none(),
            "source still skips EPHEMERAL with no MachineConfig"
        );

        runtime.run().unwrap();
        let writes_after_converged: usize = runtime.history().iter().map(|r| r.writes).sum();
        assert_eq!(
            writes_after_converged, 0,
            "META-backed no-config STATE projection is idempotent"
        );
    }

    #[test]
    fn volume_config_controller_runtime_mode_drives_container_and_agent_state() {
        let mut container_runtime = Runtime::new();
        container_runtime
            .register(Box::new(VolumeConfigController::new_for_runtime_mode(
                VolumeConfigRuntimeMode::Container,
            )))
            .unwrap();
        container_runtime.run().unwrap();
        assert!(
            container_runtime.history().iter().all(|record| record.ok),
            "container runtime-mode controller must reconcile cleanly: {:?}",
            container_runtime.history()
        );
        assert_eq!(
            volume_config(&container_runtime, STATE_VOLUME_ID)
                .spec
                .volume_type,
            VolumeType::Directory,
            "runtime-mode container should drive source inContainer STATE projection"
        );
        assert!(
            volume_config(&container_runtime, STATE_VOLUME_ID)
                .spec
                .mount
                .as_ref()
                .expect("container STATE mount")
                .secure,
            "runtime-mode container should preserve source secure STATE mount"
        );

        let mut agent_runtime = Runtime::new();
        agent_runtime
            .register(Box::new(VolumeConfigController::new_for_runtime_mode(
                VolumeConfigRuntimeMode::MetalAgent,
            )))
            .unwrap();
        agent_runtime.run().unwrap();
        assert!(
            agent_runtime.history().iter().all(|record| record.ok),
            "agent runtime-mode controller must reconcile cleanly: {:?}",
            agent_runtime.history()
        );
        let agent_state = volume_config(&agent_runtime, STATE_VOLUME_ID);
        assert_eq!(agent_state.spec.volume_type, VolumeType::Partition);
        assert_eq!(
            agent_state.spec.locator_match, NO_MATCH_LOCATOR_MATCH,
            "runtime-mode metal-agent should drive source IsAgent noMatch STATE projection"
        );
    }

    #[test]
    fn volume_config_controller_projects_agent_no_config_state_as_missing_partition() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(state_encryption_meta_key(
                r#"{
                    "EncryptionProvider":"luks2",
                    "EncryptionKeys":[
                        {"KeySlot":0,"KeyStatic":{"KeyData":"agent-state-meta-secret"}}
                    ]
                }"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new_agent()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "agent controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert_eq!(
            state.spec.volume_type,
            VolumeType::Partition,
            "source agent no-config STATE remains a partition declaration"
        );
        assert_eq!(
            state.spec.locator_match, NO_MATCH_LOCATOR_MATCH,
            "source manageStateNoConfig marks agent STATE as missing with noMatch"
        );
        assert!(
            state.spec.mount.as_ref().expect("STATE mount").secure,
            "agent no-config STATE preserves the source secure mount"
        );
        assert!(
            state.spec.provisioning.encryption_configured,
            "agent no-config STATE still carries META encryption"
        );
        assert!(
            state
                .spec_fingerprint()
                .contains("encryption_keys=0:static:false:agent-state-meta-secret::::"),
            "agent STATE META encryption should be projected: {}",
            state.spec_fingerprint()
        );

        let ephemeral_key = volume_config_key(EPHEMERAL_VOLUME_ID).unwrap();
        assert!(
            runtime.state().get(&ephemeral_key).is_none(),
            "agent no-config branch still skips EPHEMERAL"
        );
    }

    #[test]
    fn volume_config_controller_ignores_state_meta_encryption_when_machine_config_present() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(state_encryption_meta_key(
                r#"{
                    "EncryptionProvider":"luks2",
                    "EncryptionKeys":[
                        {"KeySlot":0,"KeyStatic":{"KeyData":"state-meta-secret"}}
                    ]
                }"#,
            )))
            .unwrap();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let state = volume_config(&runtime, STATE_VOLUME_ID);
        assert!(
            !state.spec.provisioning.encryption_configured,
            "config-present STATE ignores META and only uses config/legacy encryption"
        );
        let fingerprint = state.spec_fingerprint();
        assert!(
            !fingerprint.contains("state-meta-secret"),
            "no-config META fallback must not leak into config-present STATE: {fingerprint}"
        );
        assert!(
            !fingerprint.contains("encryption_provider=luks2"),
            "config-present STATE with no config encryption should remain unencrypted: {fingerprint}"
        );
    }

    #[test]
    fn volume_config_controller_projects_user_directory_volume_config_and_default_mount_request() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: host-share
volumeType: directory
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile cleanly: {:?}",
            runtime.history()
        );

        let user = volume_config(&runtime, "u-host-share");
        assert_eq!(
            user.spec,
            VolumeConfigSpec::new("u-host-share", VolumeType::Directory, "").with_mount(
                VolumeConfigMountSpec::new("host-share", 0o755, 0, 0)
                    .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
                    .with_parent_id(USER_VOLUME_MOUNT_POINT)
                    .with_bind_target("host-share")
            )
        );
        assert!(
            user.metadata().labels().has("talos.dev/user-volume"),
            "directory UserVolumeConfig output must carry the source user-volume label"
        );

        let request = volume_mount_request(&runtime, "u-host-share");
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=u-host-share;requester=block.VolumeConfigController;read_only=false;detached=false;disable_access_time=false;secure=false"
        );
    }

    #[test]
    fn volume_config_controller_projects_user_disk_volume_config_and_mount_request() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: raw-disk
volumeType: disk
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
filesystem:
  type: ext4
mount:
  disableAccessTime: true
  secure: true
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile disk user volume cleanly: {:?}",
            runtime.history()
        );

        let user = volume_config(&runtime, "u-raw-disk");
        assert_eq!(
            user.spec,
            VolumeConfigSpec::new("u-raw-disk", VolumeType::Disk, "")
                .with_locator_disk_match("disk.transport == \"nvme\"")
                .with_provisioning(VolumeConfigProvisioningSpec {
                    wave: WAVE_USER_VOLUMES,
                    disk_selector: Some("disk.transport == \"nvme\"".to_string()),
                    external_source: None,
                    label: None,
                    min_size: 0,
                    max_size: None,
                    relative_max_size: None,
                    negative_max_size: false,
                    grow: true,
                    type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
                    filesystem: Some(FilesystemType::Ext4),
                    encryption_configured: false,
                })
                .with_mount(
                    VolumeConfigMountSpec::new("raw-disk", 0o755, 0, 0)
                        .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
                        .with_parent_id(USER_VOLUME_MOUNT_POINT)
                )
        );
        assert!(
            user.metadata().labels().has("talos.dev/user-volume"),
            "disk UserVolumeConfig output must carry the source user-volume label"
        );

        let request = volume_mount_request(&runtime, "u-raw-disk");
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=u-raw-disk;requester=block.VolumeConfigController;read_only=false;detached=false;disable_access_time=true;secure=true"
        );
    }

    #[test]
    fn volume_config_controller_projects_swap_volume_config_and_default_mount_request() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: local-swap
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 256MiB
  maxSize: 1GiB
  grow: true
encryption: {}
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile swap volume config cleanly: {:?}",
            runtime.history()
        );

        let swap = volume_config(&runtime, "s-local-swap");
        assert_eq!(
            swap.spec,
            VolumeConfigSpec::new(
                "s-local-swap",
                VolumeType::Partition,
                "volume.partition_label == \"s-local-swap\""
            )
            .with_provisioning(VolumeConfigProvisioningSpec {
                wave: WAVE_USER_VOLUMES,
                disk_selector: Some("disk.transport == \"nvme\"".to_string()),
                external_source: None,
                label: Some("s-local-swap".to_string()),
                min_size: 256 * 1024 * 1024,
                max_size: Some(1024 * 1024 * 1024),
                relative_max_size: None,
                negative_max_size: false,
                grow: true,
                type_uuid: Some(type_guid::LINUX_SWAP.to_string()),
                filesystem: Some(FilesystemType::Swap),
                encryption_configured: true,
            })
        );
        assert!(
            swap.metadata().labels().has("talos.dev/swap-volume"),
            "SwapVolumeConfig output must carry the source swap-volume label"
        );

        let request = volume_mount_request(&runtime, "s-local-swap");
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=s-local-swap;requester=block.VolumeConfigController;read_only=false;detached=false;disable_access_time=false;secure=false"
        );
        assert!(
            request.metadata().labels().has("talos.dev/swap-volume"),
            "swap volume mount request must be labeled for source cleanup selection"
        );
    }

    #[test]
    fn volume_config_controller_projects_raw_volume_config_without_mount_request() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: local-raw
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 150MiB
  maxSize: 2GiB
  grow: true
encryption: {}
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile raw volume config cleanly: {:?}",
            runtime.history()
        );

        let raw = volume_config(&runtime, "r-local-raw");
        assert_eq!(
            raw.spec,
            VolumeConfigSpec::new(
                "r-local-raw",
                VolumeType::Partition,
                "volume.partition_label == \"r-local-raw\""
            )
            .with_provisioning(VolumeConfigProvisioningSpec {
                wave: WAVE_USER_VOLUMES,
                disk_selector: Some("disk.transport == \"nvme\"".to_string()),
                external_source: None,
                label: Some("r-local-raw".to_string()),
                min_size: 150 * 1024 * 1024,
                max_size: Some(2 * 1024 * 1024 * 1024),
                relative_max_size: None,
                negative_max_size: false,
                grow: true,
                type_uuid: Some(type_guid::LINUX_FILESYSTEM.to_string()),
                filesystem: None,
                encryption_configured: true,
            })
        );
        assert!(
            raw.metadata().labels().has("talos.dev/raw-volume"),
            "RawVolumeConfig output must carry the source raw-volume label"
        );

        let request_key = volume_mount_request_key("r-local-raw").unwrap();
        assert!(
            runtime.state().get(&request_key).is_none(),
            "source RawVolumeTransformer uses SkipMountTransform"
        );
    }

    #[test]
    fn volume_config_controller_projects_existing_and_external_volume_configs() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: imported-data
discovery:
  volumeSelector:
    match: volume.partition_label == "MY-DATA" && disk.serial == "SERIAL123"
mount:
  readOnly: true
  disableAccessTime: true
---
apiVersion: v1alpha1
kind: ExternalVolumeConfig
name: shared-data
filesystemType: virtiofs
mount:
  secure: false
  virtiofs:
    tag: DataShare
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "controller must reconcile existing/external volume configs cleanly: {:?}",
            runtime.history()
        );

        let existing = volume_config(&runtime, "e-imported-data");
        assert_eq!(
            existing.spec,
            VolumeConfigSpec::new(
                "e-imported-data",
                VolumeType::Partition,
                "volume.partition_label == \"MY-DATA\" && disk.serial == \"SERIAL123\""
            )
            .with_mount(
                VolumeConfigMountSpec::new("imported-data", 0o755, 0, 0)
                    .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
                    .with_parent_id(USER_VOLUME_MOUNT_POINT)
            )
        );
        assert!(
            existing
                .metadata()
                .labels()
                .has("talos.dev/existing-volume"),
            "ExistingVolumeConfig output must carry the source existing-volume label"
        );
        let existing_request = volume_mount_request(&runtime, "e-imported-data");
        assert_eq!(
            existing_request.spec_fingerprint(),
            "volume_id=e-imported-data;requester=block.VolumeConfigController;read_only=true;detached=false;disable_access_time=true;secure=true"
        );
        assert!(
            existing_request
                .metadata()
                .labels()
                .has("talos.dev/existing-volume"),
            "existing volume mount request must be labeled for source cleanup selection"
        );

        let external = volume_config(&runtime, "x-shared-data");
        assert_eq!(
            external.spec,
            VolumeConfigSpec::new("x-shared-data", VolumeType::External, "")
                .with_provisioning(VolumeConfigProvisioningSpec {
                    wave: WAVE_USER_VOLUMES,
                    disk_selector: None,
                    external_source: Some("DataShare".to_string()),
                    label: None,
                    min_size: 0,
                    max_size: None,
                    relative_max_size: None,
                    negative_max_size: false,
                    grow: false,
                    type_uuid: None,
                    filesystem: Some(FilesystemType::Virtiofs),
                    encryption_configured: false,
                })
                .with_mount(
                    VolumeConfigMountSpec::new("shared-data", 0o755, 0, 0)
                        .with_selinux_label(EPHEMERAL_SELINUX_LABEL)
                        .with_parent_id(USER_VOLUME_MOUNT_POINT)
                )
        );
        assert!(
            external
                .metadata()
                .labels()
                .has("talos.dev/external-volume"),
            "ExternalVolumeConfig output must carry the source external-volume label"
        );
        let external_request = volume_mount_request(&runtime, "x-shared-data");
        assert_eq!(
            external_request.spec_fingerprint(),
            "volume_id=x-shared-data;requester=block.VolumeConfigController;read_only=false;detached=false;disable_access_time=false;secure=false"
        );
        assert!(
            external_request
                .metadata()
                .labels()
                .has("talos.dev/external-volume"),
            "external volume mount request must be labeled for source cleanup selection"
        );
    }

    #[test]
    fn volume_config_controller_cleans_stale_labeled_volume_outputs() {
        let mut runtime = Runtime::new();
        runtime
            .state_mut()
            .create(Box::new(active_machine_config(
                r#"
version: v1alpha1
machine:
  type: worker
---
apiVersion: v1alpha1
kind: UserVolumeConfig
name: local-data
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 150MiB
  maxSize: 2GiB
  grow: true
filesystem:
  type: ext4
---
apiVersion: v1alpha1
kind: RawVolumeConfig
name: local-raw
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 150MiB
  maxSize: 2GiB
  grow: true
encryption: {}
---
apiVersion: v1alpha1
kind: ExistingVolumeConfig
name: imported-data
discovery:
  volumeSelector:
    match: volume.partition_label == "MY-DATA" && disk.serial == "SERIAL123"
mount:
  readOnly: true
  disableAccessTime: true
---
apiVersion: v1alpha1
kind: ExternalVolumeConfig
name: shared-data
filesystemType: virtiofs
mount:
  secure: false
  virtiofs:
    tag: DataShare
---
apiVersion: v1alpha1
kind: SwapVolumeConfig
name: local-swap
provisioning:
  diskSelector:
    match: disk.transport == "nvme"
  minSize: 256MiB
  maxSize: 1GiB
  grow: true
encryption: {}
"#,
            )))
            .unwrap();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "initial projection must reconcile cleanly: {:?}",
            runtime.history()
        );
        for id in [
            "u-local-data",
            "r-local-raw",
            "e-imported-data",
            "x-shared-data",
            "s-local-swap",
        ] {
            assert_volume_config_present(&runtime, id);
        }
        for id in [
            "u-local-data",
            "e-imported-data",
            "x-shared-data",
            "s-local-swap",
        ] {
            assert_volume_mount_request_present(&runtime, id);
        }

        let machine_config_key = machine_config_key();
        let machine_config_version = runtime
            .state()
            .get(&machine_config_key)
            .expect("active machine config")
            .metadata()
            .version();
        runtime
            .state_mut()
            .update(
                Box::new(active_machine_config(
                    r#"
version: v1alpha1
machine:
  type: worker
"#,
                )),
                machine_config_version,
            )
            .unwrap();

        runtime.run().unwrap();
        assert!(
            runtime.history().iter().all(|record| record.ok),
            "cleanup projection must reconcile cleanly: {:?}",
            runtime.history()
        );

        for id in [
            "u-local-data",
            "r-local-raw",
            "e-imported-data",
            "x-shared-data",
            "s-local-swap",
        ] {
            assert_volume_config_absent(&runtime, id);
        }
        for id in [
            "u-local-data",
            "e-imported-data",
            "x-shared-data",
            "s-local-swap",
        ] {
            assert_volume_mount_request_absent(&runtime, id);
        }

        assert_volume_config_present(&runtime, META_VOLUME_ID);
        assert_volume_config_present(&runtime, STATE_VOLUME_ID);
        assert_volume_config_present(&runtime, EPHEMERAL_VOLUME_ID);
        assert_volume_mount_request_present(&runtime, USER_VOLUME_MOUNT_POINT);
    }

    fn volume_config(runtime: &Runtime, id: &str) -> VolumeConfigResource {
        let key = volume_config_key(id).unwrap();
        let resource = runtime
            .state()
            .get(&key)
            .unwrap_or_else(|| panic!("missing VolumeConfig {id}"));
        VolumeConfigResource::from_resource(resource.as_ref())
            .unwrap_or_else(|| panic!("invalid VolumeConfig resource {id}"))
    }

    fn assert_volume_config_present(runtime: &Runtime, id: &str) {
        let key = volume_config_key(id).unwrap();
        assert!(
            runtime.state().get(&key).is_some(),
            "expected VolumeConfig {id} to exist"
        );
    }

    fn assert_volume_config_absent(runtime: &Runtime, id: &str) {
        let key = volume_config_key(id).unwrap();
        assert!(
            runtime.state().get(&key).is_none(),
            "expected VolumeConfig {id} to be cleaned up"
        );
    }

    fn volume_mount_request(runtime: &Runtime, id: &str) -> Box<dyn Resource> {
        let key = volume_mount_request_key(id).unwrap();
        runtime
            .state()
            .get(&key)
            .unwrap_or_else(|| panic!("missing VolumeMountRequest {id}"))
            .clone_box()
    }

    fn assert_volume_mount_request_present(runtime: &Runtime, id: &str) {
        let key = volume_mount_request_key(id).unwrap();
        assert!(
            runtime.state().get(&key).is_some(),
            "expected VolumeMountRequest {id} to exist"
        );
    }

    fn assert_volume_mount_request_absent(runtime: &Runtime, id: &str) {
        let key = volume_mount_request_key(id).unwrap();
        assert!(
            runtime.state().get(&key).is_none(),
            "expected VolumeMountRequest {id} to be cleaned up"
        );
    }

    #[test]
    fn volume_config_controller_declares_source_inputs_outputs_and_keeps_user_mount_root() {
        let controller = VolumeConfigController::new();

        assert_eq!(controller.name(), VOLUME_CONFIG_CONTROLLER_NAME);

        let spec = controller.spec();
        let expected_inputs = vec![
            Input::weak(ResourceKind::new(
                MACHINE_CONFIG_NAMESPACE,
                MACHINE_CONFIG_TYPE,
            ))
            .with_id(MACHINE_CONFIG_ACTIVE_ID),
            Input::weak(ResourceKind::new(RUNTIME_NAMESPACE, META_KEY_TYPE))
                .with_id(STATE_ENCRYPTION_META_KEY_ID),
            Input::destroy_ready(VolumeMountRequestResource::kind()),
            Input::destroy_ready(VolumeConfigResource::kind()),
        ];
        assert_eq!(spec.inputs(), expected_inputs.as_slice());
        assert_eq!(spec.strong_input_kinds(), Vec::<ResourceKind>::new());
        assert_eq!(
            spec.inputs()[2].strength(),
            InputKind::DestroyReady,
            "VolumeMountRequest input must observe destroy-ready source resources"
        );

        let expected_outputs = vec![
            Output::shared(VolumeConfigResource::kind()),
            Output::shared(VolumeMountRequestResource::kind()),
        ];
        assert_eq!(spec.outputs(), expected_outputs.as_slice());

        let mut runtime = Runtime::new();
        runtime
            .register(Box::new(VolumeConfigController::new()))
            .unwrap();
        runtime.run().unwrap();

        let key = volume_mount_request_key(USER_VOLUME_MOUNT_POINT).unwrap();
        let request = runtime.state().get(&key).expect("root keepalive request");
        assert_eq!(request.resource_kind(), VolumeMountRequestResource::kind());
        assert_eq!(
            request.spec_fingerprint(),
            "volume_id=/var/mnt;requester=block.VolumeConfigController;read_only=false;detached=false;disable_access_time=false;secure=false"
        );

        runtime.run().unwrap();
        let writes_after_converged: usize = runtime.history().iter().map(|r| r.writes).sum();
        assert_eq!(
            writes_after_converged, 0,
            "converged controller is idempotent"
        );
    }

    #[derive(Debug, Clone)]
    struct TestMachineConfigDocument {
        meta: Metadata,
        contents: String,
    }

    impl TestMachineConfigDocument {
        fn new(contents: impl Into<String>) -> Self {
            Self {
                meta: Metadata::new(
                    MACHINE_CONFIG_NAMESPACE,
                    MACHINE_CONFIG_TYPE,
                    ResourceId::new(MACHINE_CONFIG_ACTIVE_ID).unwrap(),
                ),
                contents: contents.into(),
            }
        }
    }

    impl Resource for TestMachineConfigDocument {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }

        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }

        fn spec_fingerprint(&self) -> String {
            format!("contents={}", test_hex_bytes(self.contents.as_bytes()))
        }

        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    fn active_machine_config(contents: impl Into<String>) -> TestMachineConfigDocument {
        TestMachineConfigDocument::new(contents)
    }

    #[derive(Debug, Clone)]
    struct TestStateEncryptionMetaKey {
        meta: Metadata,
        value: String,
    }

    impl TestStateEncryptionMetaKey {
        fn new(value: impl Into<String>) -> Self {
            Self {
                meta: Metadata::new(
                    RUNTIME_NAMESPACE,
                    META_KEY_TYPE,
                    ResourceId::new(STATE_ENCRYPTION_META_KEY_ID).unwrap(),
                ),
                value: value.into(),
            }
        }
    }

    impl Resource for TestStateEncryptionMetaKey {
        fn metadata(&self) -> &Metadata {
            &self.meta
        }

        fn metadata_mut(&mut self) -> &mut Metadata {
            &mut self.meta
        }

        fn spec_fingerprint(&self) -> String {
            format!("value_hex={}", test_hex_bytes(self.value.as_bytes()))
        }

        fn clone_box(&self) -> Box<dyn Resource> {
            Box::new(self.clone())
        }
    }

    fn state_encryption_meta_key(value: impl Into<String>) -> TestStateEncryptionMetaKey {
        TestStateEncryptionMetaKey::new(value)
    }

    fn test_hex_bytes(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
