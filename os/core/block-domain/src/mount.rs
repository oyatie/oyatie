//! Mount specs / statuses and their lifecycle.
//!
//! Mirrors Talos's `block.MountSpec`/`block.MountStatus` resources and the
//! mount controller's job of taking a desired mount and reconciling it to an
//! observed state. The actual `mount(2)`/`umount(2)` syscalls are modelled
//! behind the [`Mounter`] trait with an in-memory [`MemMounter`].

use std::collections::{BTreeMap, BTreeSet};

use crate::filesystem::FilesystemType;
use crate::{BlockError, Result};
use os_cosi_domain::{
    AnyResource, Labels, Metadata, Phase as CosiPhase, Resource, ResourceKind, State, StoreError,
    StoreResult,
};
use os_kernel::ResourceId;

/// Talos block resources live in the COSI `runtime` namespace.
///
/// Source: Talos v1.13.0 `pkg/machinery/resources/block` resource definitions
/// use `block.NamespaceName` as their default namespace.
pub const BLOCK_NAMESPACE: &str = "runtime";

/// Talos v1.13.0 block `MountStatus` resource type.
///
/// Source: `pkg/machinery/resources/block/mount_status.go`.
pub const MOUNT_STATUS_TYPE: &str = "MountStatuses.block.talos.dev";

/// Talos v1.13.0 block `VolumeMountRequest` resource type.
///
/// Source: `pkg/machinery/resources/block/volume_mount_request.go`.
pub const VOLUME_MOUNT_REQUEST_TYPE: &str = "VolumeMountRequests.block.talos.dev";

/// Talos v1.13.0 block `VolumeMountStatus` resource type.
///
/// Source: `pkg/machinery/resources/block/volume_mount_status.go`.
pub const VOLUME_MOUNT_STATUS_TYPE: &str = "VolumeMountStatuses.block.talos.dev";

/// Label used by `MountStatusController` to associate child
/// `VolumeMountStatus` resources with their parent `MountStatus`.
pub const MOUNT_STATUS_ID_LABEL: &str = "mount-status-id";

/// Talos mount-status controller finalizer.
pub const MOUNT_STATUS_CONTROLLER_FINALIZER: &str = "block.MountStatusController";

fn resource_id(id: impl Into<String>) -> Result<ResourceId> {
    ResourceId::new(id.into()).map_err(|err| BlockError::InvalidDevice(err.to_string()))
}

fn metadata(namespace: &str, kind: &str, id: impl Into<String>) -> Result<Metadata> {
    Ok(Metadata::new(namespace, kind, resource_id(id)?))
}

/// Source-compatible id for a volume-mount request/status.
///
/// Talos `NewVolumeMounter` sets `mountID` to `requester + "-" + volumeID`,
/// then uses that id for both `VolumeMountRequest` and `VolumeMountStatus`.
pub fn volume_mount_status_id(requester: &str, volume_id: &str) -> String {
    format!("{requester}-{volume_id}")
}

/// Canonical COSI key for a block `MountStatus` id.
pub fn mount_status_key(id: &str) -> Result<String> {
    Ok(metadata(BLOCK_NAMESPACE, MOUNT_STATUS_TYPE, id)?.key())
}

/// Canonical COSI key for a block `VolumeMountRequest` id.
pub fn volume_mount_request_key(id: &str) -> Result<String> {
    Ok(metadata(BLOCK_NAMESPACE, VOLUME_MOUNT_REQUEST_TYPE, id)?.key())
}

/// Canonical COSI key for a block `VolumeMountStatus` id.
pub fn volume_mount_status_key(id: &str) -> Result<String> {
    Ok(metadata(BLOCK_NAMESPACE, VOLUME_MOUNT_STATUS_TYPE, id)?.key())
}

/// The request sub-spec embedded in Talos's block `MountStatusSpec`.
///
/// Source fields mirror `MountRequestSpec` in Talos v1.13.0:
/// `volumeID`, `parentID`, `readOnly`, `detached`,
/// `disableAccessTime`, `secure`, `requesters`, and `requesterIDs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountRequestSpec {
    /// Volume id.
    pub volume_id: String,
    /// Optional parent mount id.
    pub parent_mount_id: Option<String>,
    /// Request read-only mounting.
    pub read_only: bool,
    /// Request a detached mount.
    pub detached: bool,
    /// Disable atime updates.
    pub disable_access_time: bool,
    /// Apply secure mount settings.
    pub secure: bool,
    /// Human/requesting controller names.
    pub requesters: Vec<String>,
    /// COSI ids for corresponding [`requesters`](Self::requesters).
    pub requester_ids: Vec<String>,
}

impl MountRequestSpec {
    /// Build a request for `volume_id`.
    pub fn new(volume_id: impl Into<String>) -> Self {
        MountRequestSpec {
            volume_id: volume_id.into(),
            parent_mount_id: None,
            read_only: false,
            detached: false,
            disable_access_time: false,
            secure: false,
            requesters: Vec::new(),
            requester_ids: Vec::new(),
        }
    }

    /// Builder: set parent mount id.
    pub fn with_parent_mount_id(mut self, parent_mount_id: impl Into<String>) -> Self {
        self.parent_mount_id = Some(parent_mount_id.into());
        self
    }

    /// Builder: set read-only flag.
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Builder: set detached flag.
    pub fn with_detached(mut self, detached: bool) -> Self {
        self.detached = detached;
        self
    }

    /// Builder: set disable-access-time flag.
    pub fn with_disable_access_time(mut self, disable_access_time: bool) -> Self {
        self.disable_access_time = disable_access_time;
        self
    }

    /// Builder: set secure flag.
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Builder: add a requester using Talos's `requester-volumeID` id format.
    pub fn with_requester(mut self, requester: impl Into<String>) -> Self {
        let requester = requester.into();
        let id = volume_mount_status_id(&requester, &self.volume_id);
        self.requesters.push(requester);
        self.requester_ids.push(id);
        self
    }

    /// Builder: add a requester with an explicit COSI request id.
    pub fn with_requester_id(
        mut self,
        requester: impl Into<String>,
        id: impl Into<String>,
    ) -> Self {
        self.requesters.push(requester.into());
        self.requester_ids.push(id.into());
        self
    }

    /// Iterate requester names with their matching COSI ids.
    pub fn requester_pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.requesters
            .iter()
            .map(String::as_str)
            .zip(self.requester_ids.iter().map(String::as_str))
    }

    fn requester_ids_set(&self) -> BTreeSet<String> {
        self.requester_ids.iter().cloned().collect()
    }

    fn fingerprint(&self) -> String {
        let requester_pairs = self
            .requester_pairs()
            .map(|(requester, id)| format!("requester={requester},requester_id={id}"))
            .collect::<Vec<_>>()
            .join("|");
        format!(
            "volume_id={};parent_mount_id={};read_only={};detached={};disable_access_time={};secure={};requesters=[{}]",
            self.volume_id,
            self.parent_mount_id.as_deref().unwrap_or(""),
            self.read_only,
            self.detached,
            self.disable_access_time,
            self.secure,
            requester_pairs
        )
    }
}

/// COSI resource form of Talos's block `MountStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountStatusResource {
    meta: Metadata,
    /// Embedded final mount request spec.
    pub request: MountRequestSpec,
    /// Source device/path.
    pub source: String,
    /// Target path.
    pub target: String,
    /// Filesystem type.
    pub filesystem: FilesystemType,
    /// Mount read-only.
    pub read_only: bool,
    /// Whether project quotas are supported.
    pub project_quota_support: bool,
    /// Detached mount flag.
    pub detached: bool,
}

impl MountStatusResource {
    /// Build a block `MountStatus` COSI resource from the host-safe mount spec.
    pub fn new(id: impl Into<String>, request: MountRequestSpec, spec: MountSpec) -> Result<Self> {
        let meta = metadata(BLOCK_NAMESPACE, MOUNT_STATUS_TYPE, id)?;
        Ok(MountStatusResource {
            meta,
            read_only: spec.flags.is_readonly() || request.read_only,
            detached: request.detached,
            request,
            source: spec.source,
            target: spec.target,
            filesystem: spec.fstype,
            project_quota_support: false,
        })
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Set project quota support.
    pub fn with_project_quota_support(mut self, project_quota_support: bool) -> Self {
        self.project_quota_support = project_quota_support;
        self
    }
}

impl Resource for MountStatusResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        format!(
            "{};source={};target={};filesystem={};read_only={};project_quota_support={};detached={}",
            self.request.fingerprint(),
            self.source,
            self.target,
            self.filesystem.as_str(),
            self.read_only,
            self.project_quota_support,
            self.detached
        )
    }

    fn clone_box(&self) -> AnyResource {
        Box::new(self.clone())
    }
}

/// Spec for Talos's block `VolumeMountRequest`.
///
/// Source fields mirror Talos v1.13.0 `VolumeMountRequestSpec`: `volumeID`,
/// `requester`, `readOnly`, `detached`, `disableAccessTime`, and `secure`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMountRequestSpec {
    /// Volume id.
    pub volume_id: String,
    /// Requesting subsystem/controller.
    pub requester: String,
    /// Request read-only mounting.
    pub read_only: bool,
    /// Request a detached mount.
    pub detached: bool,
    /// Disable atime updates.
    pub disable_access_time: bool,
    /// Apply secure mount settings.
    pub secure: bool,
}

impl VolumeMountRequestSpec {
    /// Build a volume-mount request spec.
    pub fn new(volume_id: impl Into<String>, requester: impl Into<String>) -> Self {
        VolumeMountRequestSpec {
            volume_id: volume_id.into(),
            requester: requester.into(),
            read_only: false,
            detached: false,
            disable_access_time: false,
            secure: false,
        }
    }

    /// Builder: set read-only flag.
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Builder: set detached flag.
    pub fn with_detached(mut self, detached: bool) -> Self {
        self.detached = detached;
        self
    }

    /// Builder: set disable-access-time flag.
    pub fn with_disable_access_time(mut self, disable_access_time: bool) -> Self {
        self.disable_access_time = disable_access_time;
        self
    }

    /// Builder: set secure flag.
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    fn fingerprint(&self) -> String {
        format!(
            "volume_id={};requester={};read_only={};detached={};disable_access_time={};secure={}",
            self.volume_id,
            self.requester,
            self.read_only,
            self.detached,
            self.disable_access_time,
            self.secure
        )
    }
}

/// COSI resource form of Talos's block `VolumeMountRequest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMountRequestResource {
    meta: Metadata,
    /// Request spec.
    pub spec: VolumeMountRequestSpec,
}

impl VolumeMountRequestResource {
    /// Build a block `VolumeMountRequest` COSI resource.
    pub fn new(id: impl Into<String>, spec: VolumeMountRequestSpec) -> Result<Self> {
        let meta = metadata(BLOCK_NAMESPACE, VOLUME_MOUNT_REQUEST_TYPE, id)?;
        Ok(VolumeMountRequestResource { meta, spec })
    }

    /// Kind descriptor for block `VolumeMountRequest`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(BLOCK_NAMESPACE, VOLUME_MOUNT_REQUEST_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }
}

impl Resource for VolumeMountRequestResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        self.spec.fingerprint()
    }

    fn clone_box(&self) -> AnyResource {
        Box::new(self.clone())
    }
}

/// Spec for Talos's block `VolumeMountStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMountStatusSpec {
    /// Mounted volume id.
    pub volume_id: String,
    /// Requesting subsystem/controller.
    pub requester: String,
    /// Mounted target path.
    pub target: String,
    /// Read-only flag.
    pub read_only: bool,
    /// Detached flag.
    pub detached: bool,
    /// Disable atime updates.
    pub disable_access_time: bool,
    /// Secure mount flag.
    pub secure: bool,
}

impl VolumeMountStatusSpec {
    /// Build a volume mount status spec.
    pub fn new(
        volume_id: impl Into<String>,
        requester: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        VolumeMountStatusSpec {
            volume_id: volume_id.into(),
            requester: requester.into(),
            target: target.into(),
            read_only: false,
            detached: false,
            disable_access_time: false,
            secure: false,
        }
    }

    /// Builder: set read-only flag.
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Builder: set detached flag.
    pub fn with_detached(mut self, detached: bool) -> Self {
        self.detached = detached;
        self
    }

    /// Builder: set disable-access-time flag.
    pub fn with_disable_access_time(mut self, disable_access_time: bool) -> Self {
        self.disable_access_time = disable_access_time;
        self
    }

    /// Builder: set secure flag.
    pub fn with_secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    fn from_mount_status(mount: &MountStatusResource, requester: &str) -> Self {
        VolumeMountStatusSpec::new(&mount.request.volume_id, requester, &mount.target)
            .with_read_only(mount.request.read_only)
            .with_detached(mount.request.detached)
            .with_disable_access_time(mount.request.disable_access_time)
            .with_secure(mount.request.secure)
    }

    fn fingerprint(&self) -> String {
        format!(
            "volume_id={};requester={};target={};read_only={};detached={};disable_access_time={};secure={}",
            self.volume_id,
            self.requester,
            self.target,
            self.read_only,
            self.detached,
            self.disable_access_time,
            self.secure
        )
    }
}

/// COSI resource form of Talos's block `VolumeMountStatus`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumeMountStatusResource {
    meta: Metadata,
    /// Status spec.
    pub spec: VolumeMountStatusSpec,
}

impl VolumeMountStatusResource {
    /// Build a block `VolumeMountStatus` COSI resource.
    pub fn new(id: impl Into<String>, spec: VolumeMountStatusSpec) -> Result<Self> {
        let meta = metadata(BLOCK_NAMESPACE, VOLUME_MOUNT_STATUS_TYPE, id)?;
        Ok(VolumeMountStatusResource { meta, spec })
    }

    /// Kind descriptor for block `VolumeMountStatus`.
    pub fn kind() -> ResourceKind {
        ResourceKind::new(BLOCK_NAMESPACE, VOLUME_MOUNT_STATUS_TYPE)
    }

    /// Borrow the COSI metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.meta
    }

    /// Mutably borrow the COSI metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    /// Build a child `VolumeMountStatus` from a parent `MountStatus`.
    pub fn from_mount_status(
        mount: &MountStatusResource,
        requester: &str,
        id: &str,
    ) -> Result<Self> {
        let mut status = VolumeMountStatusResource::new(
            id,
            VolumeMountStatusSpec::from_mount_status(mount, requester),
        )?;
        status
            .metadata_mut()
            .labels_mut()
            .set(MOUNT_STATUS_ID_LABEL, mount.metadata().id().as_str());
        Ok(status)
    }

    fn with_metadata(mut self, meta: Metadata) -> Self {
        self.meta = meta;
        self
    }

    /// Convert a type-erased COSI resource of the same kind back into the
    /// host-safe status fields used by image-cache planning.
    pub fn from_resource(resource: &dyn Resource) -> Option<Self> {
        if resource.resource_kind() != Self::kind() {
            return None;
        }

        let fingerprint = resource.spec_fingerprint();
        let fields = parse_fingerprint_fields(&fingerprint);
        let spec = VolumeMountStatusSpec::new(
            *fields.get("volume_id")?,
            *fields.get("requester")?,
            *fields.get("target")?,
        )
        .with_read_only(fields.get("read_only")?.parse().ok()?)
        .with_detached(fields.get("detached")?.parse().ok()?)
        .with_disable_access_time(fields.get("disable_access_time")?.parse().ok()?)
        .with_secure(fields.get("secure")?.parse().ok()?);

        Some(VolumeMountStatusResource {
            meta: resource.metadata().clone(),
            spec,
        })
    }
}

impl Resource for VolumeMountStatusResource {
    fn metadata(&self) -> &Metadata {
        &self.meta
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.meta
    }

    fn spec_fingerprint(&self) -> String {
        self.spec.fingerprint()
    }

    fn clone_box(&self) -> AnyResource {
        Box::new(self.clone())
    }
}

fn volume_mount_status_kind() -> ResourceKind {
    ResourceKind::new(BLOCK_NAMESPACE, VOLUME_MOUNT_STATUS_TYPE)
}

fn parse_fingerprint_fields(fingerprint: &str) -> BTreeMap<&str, &str> {
    fingerprint
        .split(';')
        .filter_map(|part| part.split_once('='))
        .collect()
}

fn children_for_mount(state: &State, mount_id: &str) -> Vec<AnyResource> {
    let mut selector = Labels::new();
    selector.set(MOUNT_STATUS_ID_LABEL, mount_id);
    state.list(&volume_mount_status_kind(), Some(&selector))
}

fn upsert_volume_mount_status(
    state: &mut State,
    mount: &MountStatusResource,
    requester: &str,
    id: &str,
) -> StoreResult<()> {
    let desired = VolumeMountStatusResource::from_mount_status(mount, requester, id)
        .expect("mount request requester id must be a valid COSI id");
    let key = desired.metadata().key();
    if let Some(existing) = state.get(&key) {
        let expected_version = existing.metadata().version();
        let mut meta = existing.metadata().clone();
        meta.labels_mut()
            .set(MOUNT_STATUS_ID_LABEL, mount.metadata().id().as_str());
        let desired = desired.with_metadata(meta);
        state.update(Box::new(desired), expected_version)?;
    } else {
        state.create(Box::new(desired))?;
    }
    Ok(())
}

fn teardown_and_destroy_if_unreferenced(
    state: &mut State,
    key: &str,
    version: u64,
) -> StoreResult<bool> {
    let version = state.teardown(key, version)?;
    let current = state
        .get(key)
        .ok_or_else(|| StoreError::NotFound(key.to_string()))?;
    if current.metadata().can_destroy() {
        state.destroy(key, version)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Reconcile the source-guided child `VolumeMountStatus` lifecycle for a
/// block `MountStatus` resource.
///
/// Mirrors Talos v1.13.0 `MountStatusController`: when the parent is running,
/// attach `block.MountStatusController`, fan out one child status per requester,
/// and tear down stale children; when the parent is tearing down, tear down and
/// destroy every child before removing the parent finalizer.
pub fn reconcile_volume_mount_status_resources(
    state: &mut State,
    mount: &MountStatusResource,
) -> StoreResult<()> {
    let mount_key = mount.metadata().key();
    let stored_mount = state
        .get(&mount_key)
        .ok_or_else(|| StoreError::NotFound(mount_key.clone()))?;
    let mount_id = mount.metadata().id().as_str();

    match stored_mount.metadata().phase() {
        CosiPhase::Running => {
            state.add_finalizer(&mount_key, MOUNT_STATUS_CONTROLLER_FINALIZER)?;
            for (requester, id) in mount.request.requester_pairs() {
                upsert_volume_mount_status(state, mount, requester, id)?;
            }

            let expected_ids = mount.request.requester_ids_set();
            for child in children_for_mount(state, mount_id) {
                let child_id = child.metadata().id().as_str().to_string();
                if expected_ids.contains(&child_id) {
                    continue;
                }
                let key = child.metadata().key();
                let version = child.metadata().version();
                let _ = teardown_and_destroy_if_unreferenced(state, &key, version)?;
            }
        }
        CosiPhase::TearingDown => {
            let mut all_destroyed = true;
            for child in children_for_mount(state, mount_id) {
                let key = child.metadata().key();
                let version = child.metadata().version();
                if !teardown_and_destroy_if_unreferenced(state, &key, version)? {
                    all_destroyed = false;
                }
            }
            if all_destroyed {
                state.remove_finalizer(&mount_key, MOUNT_STATUS_CONTROLLER_FINALIZER)?;
            }
        }
    }

    Ok(())
}

/// Bitflag mount options mirroring the subset of `MS_*` flags Talos sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountFlags(u32);

impl MountFlags {
    /// Mount read-only (`MS_RDONLY`).
    pub const RDONLY: MountFlags = MountFlags(1 << 0);
    /// Do not allow set-uid bits to take effect (`MS_NOSUID`).
    pub const NOSUID: MountFlags = MountFlags(1 << 1);
    /// Do not interpret device files (`MS_NODEV`).
    pub const NODEV: MountFlags = MountFlags(1 << 2);
    /// Do not allow execution (`MS_NOEXEC`).
    pub const NOEXEC: MountFlags = MountFlags(1 << 3);

    /// An empty flag set.
    pub const fn empty() -> MountFlags {
        MountFlags(0)
    }

    /// The raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Union of two flag sets.
    pub const fn union(self, other: MountFlags) -> MountFlags {
        MountFlags(self.0 | other.0)
    }

    /// Whether `other`'s bits are all set in `self`.
    pub const fn contains(self, other: MountFlags) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether the read-only bit is set.
    pub const fn is_readonly(self) -> bool {
        self.contains(MountFlags::RDONLY)
    }
}

/// A desired mount: where a device should be mounted and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountSpec {
    /// Source block device, e.g. `/dev/sda2`.
    pub source: String,
    /// Target mountpoint, e.g. `/var`.
    pub target: String,
    /// Filesystem type.
    pub fstype: FilesystemType,
    /// Mount flags.
    pub flags: MountFlags,
}

impl MountSpec {
    /// Build a mount spec.
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        fstype: FilesystemType,
    ) -> Self {
        MountSpec {
            source: source.into(),
            target: target.into(),
            fstype,
            flags: MountFlags::empty(),
        }
    }

    /// Builder: set the flags.
    pub fn with_flags(mut self, flags: MountFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Validate the spec: non-empty absolute source/target, and a read-only
    /// filesystem must not be requested read-write.
    pub fn validate(&self) -> Result<()> {
        if self.source.is_empty() || self.target.is_empty() {
            return Err(BlockError::InvalidDevice(
                "mount source and target are required".to_string(),
            ));
        }
        if !self.target.starts_with('/') {
            return Err(BlockError::InvalidDevice(
                "mount target must be absolute".to_string(),
            ));
        }
        if self.fstype.is_read_only() && !self.flags.is_readonly() {
            return Err(BlockError::BadTransition(format!(
                "{} must be mounted read-only",
                self.fstype.as_str()
            )));
        }
        Ok(())
    }
}

/// The lifecycle phase of a mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountPhase {
    /// Not yet mounted.
    Unmounted,
    /// Mount in progress.
    Mounting,
    /// Mounted and usable.
    Mounted,
    /// Unmount in progress.
    Unmounting,
}

impl MountPhase {
    /// Whether a transition from `self` to `next` is legal.
    pub fn can_transition_to(self, next: MountPhase) -> bool {
        use MountPhase::*;
        matches!(
            (self, next),
            (Unmounted, Mounting)
                | (Mounting, Mounted)
                | (Mounting, Unmounted)
                | (Mounted, Unmounting)
                | (Unmounting, Unmounted)
                | (Unmounting, Mounted)
        )
    }
}

/// Observed status of a mount, tracking its phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountStatus {
    /// The spec this status reconciles.
    pub spec: MountSpec,
    /// Current phase.
    pub phase: MountPhase,
}

impl MountStatus {
    /// New status in the [`MountPhase::Unmounted`] phase.
    pub fn new(spec: MountSpec) -> Self {
        MountStatus {
            spec,
            phase: MountPhase::Unmounted,
        }
    }

    /// Attempt a phase transition, enforcing the state machine.
    pub fn transition(&mut self, next: MountPhase) -> Result<()> {
        if !self.phase.can_transition_to(next) {
            return Err(BlockError::BadTransition(format!(
                "{:?} -> {:?}",
                self.phase, next
            )));
        }
        self.phase = next;
        Ok(())
    }

    /// Whether the mount is currently active.
    pub fn is_mounted(&self) -> bool {
        self.phase == MountPhase::Mounted
    }
}

/// The syscall boundary for mounting and unmounting.
pub trait Mounter {
    /// Mount according to `spec`.
    fn mount(&mut self, spec: &MountSpec) -> Result<()>;
    /// Unmount `target`.
    fn unmount(&mut self, target: &str) -> Result<()>;
    /// Whether `target` is currently mounted.
    fn is_mounted(&self, target: &str) -> bool;
}

/// An in-memory mount table for tests.
#[derive(Debug, Default)]
pub struct MemMounter {
    table: BTreeMap<String, MountSpec>,
    unmount_failures: BTreeMap<String, String>,
}

impl MemMounter {
    /// A fresh, empty mount table.
    pub fn new() -> Self {
        MemMounter::default()
    }

    /// Number of active mounts.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the table is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Inject a one-shot failure for `target`'s next unmount attempt.
    pub fn fail_next_unmount(&mut self, target: impl Into<String>, reason: impl Into<String>) {
        self.unmount_failures.insert(target.into(), reason.into());
    }
}

impl Mounter for MemMounter {
    fn mount(&mut self, spec: &MountSpec) -> Result<()> {
        spec.validate()?;
        if self.table.contains_key(&spec.target) {
            return Err(BlockError::BadTransition(format!(
                "{} already mounted",
                spec.target
            )));
        }
        self.table.insert(spec.target.clone(), spec.clone());
        Ok(())
    }

    fn unmount(&mut self, target: &str) -> Result<()> {
        if let Some(reason) = self.unmount_failures.remove(target) {
            return Err(BlockError::InvalidDevice(reason));
        }
        if self.table.remove(target).is_none() {
            return Err(BlockError::NotFound(format!("{target} not mounted")));
        }
        Ok(())
    }

    fn is_mounted(&self, target: &str) -> bool {
        self.table.contains_key(target)
    }
}

/// Reconcile a [`MountStatus`] to the [`MountPhase::Mounted`] state, driving the
/// state machine and the underlying [`Mounter`].
pub fn reconcile_mount<M: Mounter>(status: &mut MountStatus, mounter: &mut M) -> Result<()> {
    if status.phase == MountPhase::Mounted {
        return Ok(());
    }
    status.transition(MountPhase::Mounting)?;
    mounter.mount(&status.spec)?;
    status.transition(MountPhase::Mounted)?;
    Ok(())
}

/// Reconcile a [`MountStatus`] to the [`MountPhase::Unmounted`] state.
///
/// This mirrors Talos's teardown split: the mount controller removes the
/// filesystem mount before any encrypted device-mapper close is eligible.
/// Unmount failures leave the status in [`MountPhase::Unmounting`] so a later
/// reconciliation pass can retry without remounting or losing the target.
pub fn reconcile_unmount<M: Mounter>(status: &mut MountStatus, mounter: &mut M) -> Result<()> {
    match status.phase {
        MountPhase::Unmounted => {
            if mounter.is_mounted(&status.spec.target) {
                mounter.unmount(&status.spec.target)?;
            }
            return Ok(());
        }
        MountPhase::Mounted => status.transition(MountPhase::Unmounting)?,
        MountPhase::Unmounting => {}
        MountPhase::Mounting => {
            return Err(BlockError::BadTransition(
                "cannot unmount while mount is in progress".to_string(),
            ));
        }
    }
    mounter.unmount(&status.spec.target)?;
    status.transition(MountPhase::Unmounted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use os_cosi_domain::{Phase, Resource, State, StoreError};

    fn spec() -> MountSpec {
        MountSpec::new("/dev/sda2", "/var", FilesystemType::Xfs)
    }

    #[test]
    fn flags_union_and_contains() {
        let f = MountFlags::NOSUID.union(MountFlags::NODEV);
        assert!(f.contains(MountFlags::NOSUID));
        assert!(f.contains(MountFlags::NODEV));
        assert!(!f.contains(MountFlags::RDONLY));
        assert!(MountFlags::RDONLY.is_readonly());
    }

    #[test]
    fn spec_validation() {
        assert!(spec().validate().is_ok());
        let bad = MountSpec::new("", "/var", FilesystemType::Xfs);
        assert!(bad.validate().is_err());
        let rel = MountSpec::new("/dev/x", "var", FilesystemType::Xfs);
        assert!(rel.validate().is_err());
        // iso9660 must be read-only.
        let iso = MountSpec::new("/dev/sr0", "/media", FilesystemType::Iso9660);
        assert!(iso.validate().is_err());
        let iso_ro = iso.with_flags(MountFlags::RDONLY);
        assert!(iso_ro.validate().is_ok());
    }

    #[test]
    fn phase_machine_rejects_illegal() {
        let mut st = MountStatus::new(spec());
        assert!(st.transition(MountPhase::Mounted).is_err());
        st.transition(MountPhase::Mounting).unwrap();
        st.transition(MountPhase::Mounted).unwrap();
        assert!(st.is_mounted());
        assert!(st.transition(MountPhase::Mounting).is_err());
        st.transition(MountPhase::Unmounting).unwrap();
        st.transition(MountPhase::Unmounted).unwrap();
    }

    #[test]
    fn mem_mounter_tracks_mounts() {
        let mut m = MemMounter::new();
        assert!(m.is_empty());
        m.mount(&spec()).unwrap();
        assert!(m.is_mounted("/var"));
        assert_eq!(m.len(), 1);
        assert!(m.mount(&spec()).is_err()); // double mount
        m.unmount("/var").unwrap();
        assert!(m.unmount("/var").is_err());
    }

    #[test]
    fn reconcile_drives_to_mounted() {
        let mut st = MountStatus::new(spec());
        let mut m = MemMounter::new();
        reconcile_mount(&mut st, &mut m).unwrap();
        assert!(st.is_mounted());
        assert!(m.is_mounted("/var"));
        // Idempotent.
        reconcile_mount(&mut st, &mut m).unwrap();
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn reconcile_unmount_drives_to_unmounted() {
        let mut st = MountStatus::new(spec());
        let mut m = MemMounter::new();
        reconcile_mount(&mut st, &mut m).unwrap();

        reconcile_unmount(&mut st, &mut m).unwrap();

        assert_eq!(st.phase, MountPhase::Unmounted);
        assert!(!m.is_mounted("/var"));
        reconcile_unmount(&mut st, &mut m).unwrap();
        assert_eq!(st.phase, MountPhase::Unmounted);
    }

    #[test]
    fn reconcile_unmount_failure_is_retryable_without_losing_mount() {
        let mut st = MountStatus::new(spec());
        let mut m = MemMounter::new();
        reconcile_mount(&mut st, &mut m).unwrap();
        m.fail_next_unmount("/var", "target busy");

        let err = reconcile_unmount(&mut st, &mut m).unwrap_err();

        assert!(err.to_string().contains("target busy"));
        assert_eq!(st.phase, MountPhase::Unmounting);
        assert!(m.is_mounted("/var"));

        reconcile_unmount(&mut st, &mut m).unwrap();
        assert_eq!(st.phase, MountPhase::Unmounted);
        assert!(!m.is_mounted("/var"));
    }

    fn kubelet_mount_request() -> MountRequestSpec {
        MountRequestSpec::new("DATA")
            .with_requester("kubelet")
            .with_read_only(true)
            .with_detached(true)
            .with_disable_access_time(true)
            .with_secure(true)
    }

    fn kubelet_mount_status_resource() -> MountStatusResource {
        MountStatusResource::new(
            volume_mount_status_id("kubelet", "DATA"),
            kubelet_mount_request(),
            MountSpec::new(
                "/dev/mapper/luks2-DATA",
                "/var/lib/kubelet",
                FilesystemType::Xfs,
            )
            .with_flags(MountFlags::RDONLY),
        )
        .unwrap()
    }

    #[test]
    fn cosi_mount_status_resource_fingerprint_tracks_mount_spec_and_request() {
        let mount = kubelet_mount_status_resource();

        assert_eq!(
            mount.metadata().key(),
            "runtime/MountStatuses.block.talos.dev/kubelet-DATA"
        );
        let fingerprint = mount.spec_fingerprint();
        assert!(fingerprint.contains("volume_id=DATA"));
        assert!(fingerprint.contains("requester=kubelet"));
        assert!(fingerprint.contains("requester_id=kubelet-DATA"));
        assert!(fingerprint.contains("source=/dev/mapper/luks2-DATA"));
        assert!(fingerprint.contains("target=/var/lib/kubelet"));
        assert!(fingerprint.contains("filesystem=xfs"));
        assert!(fingerprint.contains("read_only=true"));
        assert!(fingerprint.contains("detached=true"));
        assert!(fingerprint.contains("disable_access_time=true"));
        assert!(fingerprint.contains("secure=true"));
    }

    #[test]
    fn cosi_volume_mount_request_resource_matches_talos_kind_and_fields() {
        let request = VolumeMountRequestResource::new(
            "cri.ImageCacheConfigController-IMAGECACHE",
            VolumeMountRequestSpec::new("IMAGECACHE", "cri.ImageCacheConfigController")
                .with_read_only(true)
                .with_detached(true)
                .with_disable_access_time(true)
                .with_secure(true),
        )
        .unwrap();

        assert_eq!(
            request.metadata().key(),
            "runtime/VolumeMountRequests.block.talos.dev/cri.ImageCacheConfigController-IMAGECACHE"
        );
        assert_eq!(
            volume_mount_request_key("cri.ImageCacheConfigController-IMAGECACHE").unwrap(),
            request.metadata().key()
        );
        assert_eq!(
            VolumeMountRequestResource::kind(),
            ResourceKind::new(BLOCK_NAMESPACE, VOLUME_MOUNT_REQUEST_TYPE)
        );
        let fingerprint = request.spec_fingerprint();
        assert!(fingerprint.contains("volume_id=IMAGECACHE"));
        assert!(fingerprint.contains("requester=cri.ImageCacheConfigController"));
        assert!(fingerprint.contains("read_only=true"));
        assert!(fingerprint.contains("detached=true"));
        assert!(fingerprint.contains("disable_access_time=true"));
        assert!(fingerprint.contains("secure=true"));
    }

    #[test]
    fn cosi_volume_mount_status_resource_preserves_requester_target_and_flags() {
        let mount = kubelet_mount_status_resource();
        let status =
            VolumeMountStatusResource::from_mount_status(&mount, "kubelet", "kubelet-DATA")
                .unwrap();

        assert_eq!(
            status.metadata().key(),
            "runtime/VolumeMountStatuses.block.talos.dev/kubelet-DATA"
        );
        assert_eq!(
            status.metadata().labels().get(MOUNT_STATUS_ID_LABEL),
            Some("kubelet-DATA")
        );
        let fingerprint = status.spec_fingerprint();
        assert!(fingerprint.contains("volume_id=DATA"));
        assert!(fingerprint.contains("requester=kubelet"));
        assert!(fingerprint.contains("target=/var/lib/kubelet"));
        assert!(fingerprint.contains("read_only=true"));
        assert!(fingerprint.contains("detached=true"));
        assert!(fingerprint.contains("disable_access_time=true"));
        assert!(fingerprint.contains("secure=true"));
    }

    #[test]
    fn cosi_mount_status_lifecycle_fans_out_and_drains_children_before_parent_finalizer() {
        let mount = kubelet_mount_status_resource();
        let mount_key = mount.metadata().key();
        let child_key = volume_mount_status_key("kubelet-DATA").unwrap();
        let mut state = State::new();
        state.create(Box::new(mount.clone())).unwrap();

        reconcile_volume_mount_status_resources(&mut state, &mount).unwrap();

        let stored_mount = state.get(&mount_key).unwrap();
        assert!(
            stored_mount
                .metadata()
                .finalizers()
                .contains(MOUNT_STATUS_CONTROLLER_FINALIZER)
        );
        assert!(state.contains(&child_key));

        let mount_version = stored_mount.metadata().version();
        state.teardown(&mount_key, mount_version).unwrap();
        state.add_finalizer(&child_key, "kubelet").unwrap();
        reconcile_volume_mount_status_resources(&mut state, &mount).unwrap();

        let child = state.get(&child_key).unwrap();
        assert_eq!(child.metadata().phase(), Phase::TearingDown);
        assert!(child.metadata().finalizers().contains("kubelet"));
        let parent = state.get(&mount_key).unwrap();
        assert!(
            parent
                .metadata()
                .finalizers()
                .contains(MOUNT_STATUS_CONTROLLER_FINALIZER)
        );
        let err = state
            .destroy(&mount_key, parent.metadata().version())
            .unwrap_err();
        assert!(matches!(err, StoreError::StillReferenced(_)));

        state.remove_finalizer(&child_key, "kubelet").unwrap();
        reconcile_volume_mount_status_resources(&mut state, &mount).unwrap();

        assert!(!state.contains(&child_key));
        let parent = state.get(&mount_key).unwrap();
        assert!(
            !parent
                .metadata()
                .finalizers()
                .contains(MOUNT_STATUS_CONTROLLER_FINALIZER)
        );
        state
            .destroy(&mount_key, parent.metadata().version())
            .unwrap();
        assert!(!state.contains(&mount_key));
    }
}
