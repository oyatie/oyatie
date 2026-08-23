//! Cloud resource aggregate kernel.
//!
//! This crate owns the `CLOUD_RESOURCE_TYPE` contract. A resource is the
//! control-plane consistency boundary for kind, owner, tenant, location,
//! residency, lifecycle state, tags, policy attachments, and metering identity.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use cell_region::{AzCode, CellId, RegionCode};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use network_residency::{ResidencyClass, residency_class_allows_home_region_label};

const RESOURCE_SCHEMA_VERSION: u32 = 1;
const RESOURCE_ID_PREFIX_OWNER: &str = "oyatie";
const RESOURCE_ID_PREFIX_SERVICE: &str = "cloud";
const TENANT_ID_PREFIX: &str = "ten_";
const HUMAN_PRINCIPAL_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";
const POLICY_ID_PREFIX: &str = "pol_";
const RESERVED_TAG_PREFIX: &str = "oyatie:";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResourceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PrincipalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IamPolicyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TagKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TagValue {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeteringTag {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InstanceFlavor {
    GeneralPurpose,
    ComputeOptimized,
    MemoryOptimized,
    Gpu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum K8sFlavor {
    Standard,
    HighAvailability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FunctionRuntime {
    Rust,
    TypeScript,
    Python,
    Wasm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BareMetalFlavor {
    GeneralPurpose,
    StorageOptimized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GpuFlavor {
    Training,
    Inference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum BucketTier {
    Standard,
    InfrequentAccess,
    Archive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VolumeTier {
    GeneralPurposeSsd,
    ProvisionedIopsSsd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilesystemTier {
    Standard,
    ThroughputOptimized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LbProtocol {
    L4,
    L7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum DatabaseEngine {
    Postgres,
    Citus,
    PgVector,
    Valkey,
    Kafka,
    ClickHouse,
    Cassandra,
    Iceberg,
    Milvus,
    Temporal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum QueueEngine {
    Kafka,
    Nats,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ImageKind {
    MachineImage,
    ContainerImage,
    FunctionBundle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ResourceKind {
    ComputeInstance(InstanceFlavor),
    KubernetesCluster(K8sFlavor),
    Function(FunctionRuntime),
    BareMetal(BareMetalFlavor),
    GpuFleet(GpuFlavor),
    Bucket(BucketTier),
    Volume(VolumeTier),
    Filesystem(FilesystemTier),
    ArchiveVault,
    Vpc,
    Subnet,
    LoadBalancer(LbProtocol),
    DnsZone,
    CdnDistribution,
    DirectInterconnect,
    DdosProtection,
    Database(DatabaseEngine),
    QueueOrStream(QueueEngine),
    SearchIndex,
    KmsKey,
    SecretBundle,
    Image(ImageKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ResourceState {
    Pending,
    Running,
    Stopped,
    Terminated,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceCreate {
    pub id: String,                          // data_class: INTERNAL_ONLY
    pub tenant_id: String,                   // data_class: INTERNAL_ONLY
    pub region: String,                      // data_class: PUBLIC
    pub az: Option<String>,                  // data_class: PUBLIC
    pub cell_id: String,                     // data_class: PUBLIC
    pub kind: ResourceKind,                  // data_class: PUBLIC
    pub data_class: DataClass,               // data_class: PUBLIC
    pub owner_principal: String,             // data_class: INTERNAL_ONLY
    pub state: ResourceState,                // data_class: PUBLIC
    pub tags: BTreeMap<String, String>,      // data_class: INTERNAL_ONLY
    pub iam_policy_attachments: Vec<String>, // data_class: INTERNAL_ONLY
    pub metering_tag: String,                // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,           // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: u64,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    pub id: Classified<ResourceId>,     // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>, // data_class: PUBLIC
    pub az: Classified<Option<AzCode>>, // data_class: PUBLIC
    pub cell_id: Classified<CellId>,    // data_class: PUBLIC
    pub kind: Classified<ResourceKind>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub owner_principal: Classified<PrincipalId>, // data_class: INTERNAL_ONLY
    pub state: Classified<ResourceState>, // data_class: PUBLIC
    pub tags: Classified<BTreeMap<TagKey, TagValue>>, // data_class: INTERNAL_ONLY
    pub iam_policy_attachments: Classified<Vec<IamPolicyId>>, // data_class: INTERNAL_ONLY
    pub metering_tag: Classified<MeteringTag>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub updated_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudResourceError {
    InvalidResourceId,
    ResourceIdRegionMismatch,
    ResourceIdTenantMismatch,
    ResourceIdKindMismatch,
    InvalidTenantId,
    InvalidPrincipalId,
    InvalidPolicyId,
    DuplicatePolicyId,
    InvalidTagKey,
    InvalidTagValue,
    InvalidMeteringTag,
    InvalidDataClass,
    InvalidInitialState,
    InvalidStateTransition,
    InvalidTimeOrder,
    AzRequiredForResourceKind,
    AzRegionMismatch,
    CellLocationMismatch,
    ResidencyRegionMismatch,
    DuplicateResource,
    UnknownResource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResourceIdParts {
    region: RegionCode, // data_class: PUBLIC
    tenant_id: String,  // data_class: INTERNAL_ONLY
    kind_label: String, // data_class: PUBLIC
    name: String,       // data_class: INTERNAL_ONLY
}

pub trait ResourceRepo {
    fn create(&mut self, input: ResourceCreate) -> Result<Resource, CloudResourceError>;
    fn get(&self, id: &ResourceId) -> Option<&Resource>;
    fn transition_state(
        &mut self,
        id: &ResourceId,
        next_state: ResourceState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Resource, CloudResourceError>;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResourceRegistry {
    resources: BTreeMap<ResourceId, Resource>,
}

impl ResourceKind {
    pub const fn type_label(self) -> &'static str {
        match self {
            Self::ComputeInstance(_) => "instance",
            Self::KubernetesCluster(_) => "k8s",
            Self::Function(_) => "function",
            Self::BareMetal(_) => "bare-metal",
            Self::GpuFleet(_) => "gpu-fleet",
            Self::Bucket(_) => "bucket",
            Self::Volume(_) => "volume",
            Self::Filesystem(_) => "filesystem",
            Self::ArchiveVault => "archive-vault",
            Self::Vpc => "vpc",
            Self::Subnet => "subnet",
            Self::LoadBalancer(LbProtocol::L4) => "lb-v4",
            Self::LoadBalancer(LbProtocol::L7) => "lb-v7",
            Self::DnsZone => "dns-zone",
            Self::CdnDistribution => "cdn-distribution",
            Self::DirectInterconnect => "direct-interconnect",
            Self::DdosProtection => "ddos-protection",
            Self::Database(_) => "database",
            Self::QueueOrStream(_) => "queue-stream",
            Self::SearchIndex => "search-index",
            Self::KmsKey => "kms-key",
            Self::SecretBundle => "secret-bundle",
            Self::Image(_) => "image",
        }
    }

    pub const fn requires_az(self) -> bool {
        matches!(
            self,
            Self::ComputeInstance(_)
                | Self::KubernetesCluster(_)
                | Self::Function(_)
                | Self::BareMetal(_)
                | Self::GpuFleet(_)
                | Self::Volume(_)
                | Self::Filesystem(_)
                | Self::Subnet
                | Self::LoadBalancer(_)
                | Self::Database(_)
                | Self::QueueOrStream(_)
                | Self::SearchIndex
                | Self::Image(_)
        )
    }
}

impl ResourceState {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Terminated)
    }

    /// Returns the canonical lowercase string label for this state.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Terminated => "terminated",
            Self::Error => "error",
        }
    }

    /// Parses a canonical string label back to a `ResourceState`.
    /// Returns `None` for any unrecognised input (fail-closed).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "stopped" => Some(Self::Stopped),
            "terminated" => Some(Self::Terminated),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Returns `true` iff the resource is actively consuming compute (`Running`).
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` iff the resource is idle but not destroyed (`Stopped`).
    pub const fn is_quiescent(self) -> bool {
        matches!(self, Self::Stopped)
    }

    /// Returns the ordered slice of legal successor states reachable from `self`
    /// in a single transition, including the self-loop.
    ///
    /// This exposes the transition graph defined by the crate-private
    /// `state_transition_allowed` predicate so callers can introspect reachability
    /// without holding a [`Resource`] reference.
    pub const fn allowed_next(self) -> &'static [Self] {
        match self {
            Self::Pending => &[Self::Pending, Self::Running, Self::Error, Self::Terminated],
            Self::Running => &[Self::Running, Self::Stopped, Self::Error, Self::Terminated],
            Self::Stopped => &[Self::Stopped, Self::Running, Self::Error, Self::Terminated],
            Self::Error => &[Self::Error, Self::Terminated],
            Self::Terminated => &[Self::Terminated],
        }
    }

    /// Pre-checks whether a transition from `self` to `next` is legal without
    /// mutating a [`Resource`]. Delegates to the existing transition predicate.
    pub fn can_transition_to(self, next: Self) -> bool {
        state_transition_allowed(self, next)
    }
}

impl ResourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        parse_resource_id(&value)?;
        Ok(Self { value })
    }

    pub fn tenant_id(&self) -> Result<String, CloudResourceError> {
        Ok(self.parts()?.tenant_id)
    }

    pub fn region(&self) -> Result<RegionCode, CloudResourceError> {
        Ok(self.parts()?.region)
    }

    pub fn kind_label(&self) -> Result<String, CloudResourceError> {
        Ok(self.parts()?.kind_label)
    }

    pub fn resource_name(&self) -> Result<String, CloudResourceError> {
        Ok(self.parts()?.name)
    }

    fn parts(&self) -> Result<ResourceIdParts, CloudResourceError> {
        parse_resource_id(&self.value)
    }
}

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if (value.starts_with(HUMAN_PRINCIPAL_PREFIX) && value.len() > HUMAN_PRINCIPAL_PREFIX.len())
            || (value.starts_with(SERVICE_PRINCIPAL_PREFIX)
                && value.len() > SERVICE_PRINCIPAL_PREFIX.len())
        {
            Ok(Self { value })
        } else {
            Err(CloudResourceError::InvalidPrincipalId)
        }
    }
}

impl IamPolicyId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if value.starts_with(POLICY_ID_PREFIX) && value.len() > POLICY_ID_PREFIX.len() {
            Ok(Self { value })
        } else {
            Err(CloudResourceError::InvalidPolicyId)
        }
    }
}

impl TagKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if value.trim().is_empty()
            || value.starts_with(RESERVED_TAG_PREFIX)
            || !value.bytes().all(is_tag_byte)
        {
            return Err(CloudResourceError::InvalidTagKey);
        }
        Ok(Self { value })
    }
}

impl TagValue {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 256 {
            return Err(CloudResourceError::InvalidTagValue);
        }
        Ok(Self { value })
    }
}

impl MeteringTag {
    pub fn new(
        value: impl Into<String>,
        tenant_id: &str,
        kind: ResourceKind,
    ) -> Result<Self, CloudResourceError> {
        let value = value.into();
        let expected = format!("oyatie:metering:{tenant_id}:{}", kind.type_label());
        if value == expected {
            Ok(Self { value })
        } else {
            Err(CloudResourceError::InvalidMeteringTag)
        }
    }
}

impl Resource {
    pub fn new(input: ResourceCreate) -> Result<Self, CloudResourceError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_time_order(
            input.created_at_epoch_seconds,
            input.updated_at_epoch_seconds,
        )?;
        if input.state.is_terminal() {
            return Err(CloudResourceError::InvalidInitialState);
        }
        let id = ResourceId::new(input.id)?;
        let id_parts = id.parts()?;
        let region =
            RegionCode::new(input.region).map_err(|_| CloudResourceError::InvalidResourceId)?;
        let az = input
            .az
            .map(AzCode::new)
            .transpose()
            .map_err(|_| CloudResourceError::AzRegionMismatch)?;
        let cell_id =
            CellId::new(input.cell_id).map_err(|_| CloudResourceError::CellLocationMismatch)?;
        validate_resource_id_matches(&id_parts, &region, &input.tenant_id, input.kind)?;
        validate_az_requirement(input.kind, az.as_ref())?;
        validate_az_region(az.as_ref(), &region)?;
        validate_cell_location(&cell_id, &region, az.as_ref())?;
        if !residency_class_allows_home_region_label(&input.residency, &region.value) {
            return Err(CloudResourceError::ResidencyRegionMismatch);
        }
        let data_class = resource_data_class_from_legacy(input.data_class)?;
        let owner_principal = PrincipalId::new(input.owner_principal)?;
        let tags = typed_tags(input.tags)?;
        let iam_policy_attachments = typed_policy_ids(input.iam_policy_attachments)?;
        let metering_tag = MeteringTag::new(input.metering_tag, &input.tenant_id, input.kind)?;
        Ok(Self {
            id: internal(id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            az: public(az),
            cell_id: public(cell_id),
            kind: public(input.kind),
            data_class: public(data_class),
            owner_principal: internal(owner_principal),
            state: public(input.state),
            tags: internal(tags),
            iam_policy_attachments: internal(iam_policy_attachments),
            metering_tag: internal(metering_tag),
            residency: internal(input.residency),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            updated_at_epoch_seconds: internal(input.updated_at_epoch_seconds),
            schema_version: public(RESOURCE_SCHEMA_VERSION),
        })
    }

    pub fn transition_state(
        &self,
        next_state: ResourceState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Self, CloudResourceError> {
        validate_time_order(
            self.updated_at_epoch_seconds.value,
            updated_at_epoch_seconds,
        )?;
        if !state_transition_allowed(self.state.value, next_state) {
            return Err(CloudResourceError::InvalidStateTransition);
        }
        let mut resource = self.clone();
        resource.state = public(next_state);
        resource.updated_at_epoch_seconds = internal(updated_at_epoch_seconds);
        Ok(resource)
    }
}

impl ResourceRepo for ResourceRegistry {
    fn create(&mut self, input: ResourceCreate) -> Result<Resource, CloudResourceError> {
        let resource = Resource::new(input)?;
        if self.resources.contains_key(&resource.id.value) {
            return Err(CloudResourceError::DuplicateResource);
        }
        self.resources
            .insert(resource.id.value.clone(), resource.clone());
        Ok(resource)
    }

    fn get(&self, id: &ResourceId) -> Option<&Resource> {
        self.resources.get(id)
    }

    fn transition_state(
        &mut self,
        id: &ResourceId,
        next_state: ResourceState,
        updated_at_epoch_seconds: u64,
    ) -> Result<Resource, CloudResourceError> {
        let current = self
            .resources
            .get(id)
            .ok_or(CloudResourceError::UnknownResource)?;
        let updated = current.transition_state(next_state, updated_at_epoch_seconds)?;
        self.resources.insert(id.clone(), updated.clone());
        Ok(updated)
    }
}

impl ResourceRegistry {
    pub fn resources(&self) -> impl Iterator<Item = &Resource> {
        self.resources.values()
    }
}

pub fn resource_data_class_from_legacy(
    data_class: DataClass,
) -> Result<PrivacyDataClass, CloudResourceError> {
    PrivacyDataClass::new(data_class).map_err(|_| CloudResourceError::InvalidDataClass)
}

fn parse_resource_id(value: &str) -> Result<ResourceIdParts, CloudResourceError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 6
        || parts[0] != RESOURCE_ID_PREFIX_OWNER
        || parts[1] != RESOURCE_ID_PREFIX_SERVICE
        || parts.iter().any(|part| part.trim().is_empty())
    {
        return Err(CloudResourceError::InvalidResourceId);
    }
    let region = RegionCode::new(parts[2]).map_err(|_| CloudResourceError::InvalidResourceId)?;
    validate_tenant_id(parts[3])?;
    validate_canonical_segment(parts[4], CloudResourceError::InvalidResourceId)?;
    validate_canonical_segment(parts[5], CloudResourceError::InvalidResourceId)?;
    Ok(ResourceIdParts {
        region,
        tenant_id: parts[3].to_string(),
        kind_label: parts[4].to_string(),
        name: parts[5].to_string(),
    })
}

fn validate_resource_id_matches(
    id_parts: &ResourceIdParts,
    region: &RegionCode,
    tenant_id: &str,
    kind: ResourceKind,
) -> Result<(), CloudResourceError> {
    if &id_parts.region != region {
        return Err(CloudResourceError::ResourceIdRegionMismatch);
    }
    if id_parts.tenant_id != tenant_id {
        return Err(CloudResourceError::ResourceIdTenantMismatch);
    }
    if id_parts.kind_label != kind.type_label() {
        return Err(CloudResourceError::ResourceIdKindMismatch);
    }
    Ok(())
}

fn validate_az_requirement(
    kind: ResourceKind,
    az: Option<&AzCode>,
) -> Result<(), CloudResourceError> {
    if kind.requires_az() && az.is_none() {
        Err(CloudResourceError::AzRequiredForResourceKind)
    } else {
        Ok(())
    }
}

fn validate_az_region(az: Option<&AzCode>, region: &RegionCode) -> Result<(), CloudResourceError> {
    if let Some(az) = az {
        if az.value == region.value
            || az
                .value
                .strip_prefix(&region.value)
                .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
        {
            Ok(())
        } else {
            Err(CloudResourceError::AzRegionMismatch)
        }
    } else {
        Ok(())
    }
}

fn validate_cell_location(
    cell_id: &CellId,
    region: &RegionCode,
    az: Option<&AzCode>,
) -> Result<(), CloudResourceError> {
    let expected_prefix = match az {
        Some(az) => format!("cell-{}-", az.value),
        None => format!("cell-{}-", region.value),
    };
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudResourceError::CellLocationMismatch)
    }
}

fn typed_tags(
    tags: BTreeMap<String, String>,
) -> Result<BTreeMap<TagKey, TagValue>, CloudResourceError> {
    tags.into_iter()
        .map(|(key, value)| Ok((TagKey::new(key)?, TagValue::new(value)?)))
        .collect()
}

fn typed_policy_ids(values: Vec<String>) -> Result<Vec<IamPolicyId>, CloudResourceError> {
    let mut seen = BTreeSet::new();
    let mut typed = Vec::with_capacity(values.len());
    for value in values {
        let policy_id = IamPolicyId::new(value)?;
        if !seen.insert(policy_id.clone()) {
            return Err(CloudResourceError::DuplicatePolicyId);
        }
        typed.push(policy_id);
    }
    Ok(typed)
}

fn state_transition_allowed(current: ResourceState, next: ResourceState) -> bool {
    current == next
        || matches!(
            (current, next),
            (ResourceState::Pending, ResourceState::Running)
                | (ResourceState::Pending, ResourceState::Error)
                | (ResourceState::Pending, ResourceState::Terminated)
                | (ResourceState::Running, ResourceState::Stopped)
                | (ResourceState::Running, ResourceState::Error)
                | (ResourceState::Running, ResourceState::Terminated)
                | (ResourceState::Stopped, ResourceState::Running)
                | (ResourceState::Stopped, ResourceState::Error)
                | (ResourceState::Stopped, ResourceState::Terminated)
                | (ResourceState::Error, ResourceState::Terminated)
        )
}

fn validate_tenant_id(value: &str) -> Result<(), CloudResourceError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudResourceError::InvalidTenantId)
    }
}

fn validate_time_order(start: u64, end: u64) -> Result<(), CloudResourceError> {
    if end >= start {
        Ok(())
    } else {
        Err(CloudResourceError::InvalidTimeOrder)
    }
}

fn validate_canonical_segment(
    value: &str,
    error: CloudResourceError,
) -> Result<(), CloudResourceError> {
    if value.trim().is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(error);
    }
    Ok(())
}

fn is_tag_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':')
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use network_residency::{
        PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    };

    use super::*;

    fn residency_class() -> ResidencyClass {
        ResidencyClass::PerPack(Box::new(
            PerPackResidency::new(PerPackResidencyCreate {
                allowed_primary_regions: vec!["region-alpha1".to_string()],
                allowed_replica_regions: vec!["region-beta1".to_string()],
                forbidden_regions: vec!["region-gamma1".to_string()],
                regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                    regulator_refs: vec!["regulator/cloud-resource".to_string()],
                    evidence_ref: "evidence/residency/cloud-resource".to_string(),
                })
                .expect("regulator overlay fixture is valid"),
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    fn tags() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("cost-center".to_string(), "foundry".to_string()),
            ("env".to_string(), "preview".to_string()),
        ])
    }

    fn compute_resource_create() -> ResourceCreate {
        ResourceCreate {
            id: "oyatie:cloud:region-alpha1:ten_alpha:instance:api-001".to_string(),
            tenant_id: "ten_alpha".to_string(),
            region: "region-alpha1".to_string(),
            az: Some("region-alpha1-a".to_string()),
            cell_id: "cell-region-alpha1-a-001".to_string(),
            kind: ResourceKind::ComputeInstance(InstanceFlavor::GeneralPurpose),
            data_class: DataClass::InternalOnly,
            owner_principal: "sp_foundry".to_string(),
            state: ResourceState::Pending,
            tags: tags(),
            iam_policy_attachments: vec!["pol_cloud_compute_admin".to_string()],
            metering_tag: "oyatie:metering:ten_alpha:instance".to_string(),
            residency: residency_class(),
            created_at_epoch_seconds: 1_700_000_000,
            updated_at_epoch_seconds: 1_700_000_000,
        }
    }

    #[test]
    fn creates_resource_aggregate_with_location_residency_and_metering_identity() {
        let resource = Resource::new(compute_resource_create()).expect("resource should be valid");

        assert_eq!(resource.tenant_id.value, "ten_alpha");
        assert_eq!(resource.region.value.value, "region-alpha1");
        assert_eq!(
            resource.az.value.expect("compute has AZ").value,
            "region-alpha1-a"
        );
        assert_eq!(resource.cell_id.value.value, "cell-region-alpha1-a-001");
        assert_eq!(resource.kind.value.type_label(), "instance");
        assert_eq!(
            resource.metering_tag.value.value,
            "oyatie:metering:ten_alpha:instance"
        );
        assert_eq!(resource.schema_version.value, RESOURCE_SCHEMA_VERSION);
    }

    #[test]
    fn rejects_resource_id_that_disagrees_with_tenant_region_or_kind() {
        let tenant_error = Resource::new(ResourceCreate {
            id: "oyatie:cloud:region-alpha1:ten_other:instance:api-001".to_string(),
            ..compute_resource_create()
        })
        .expect_err("resource id tenant must match resource tenant");
        assert_eq!(tenant_error, CloudResourceError::ResourceIdTenantMismatch);

        let kind_error = Resource::new(ResourceCreate {
            id: "oyatie:cloud:region-alpha1:ten_alpha:bucket:api-001".to_string(),
            ..compute_resource_create()
        })
        .expect_err("resource id kind must match resource kind");
        assert_eq!(kind_error, CloudResourceError::ResourceIdKindMismatch);
    }

    #[test]
    fn rejects_az_scoped_resource_without_az() {
        let error = Resource::new(ResourceCreate {
            az: None,
            cell_id: "cell-region-alpha1-001".to_string(),
            ..compute_resource_create()
        })
        .expect_err("compute instances must declare AZ placement");

        assert_eq!(error, CloudResourceError::AzRequiredForResourceKind);
    }

    #[test]
    fn rejects_location_tuple_drift_between_region_az_and_cell() {
        let az_error = Resource::new(ResourceCreate {
            az: Some("region-gamma1-a".to_string()),
            ..compute_resource_create()
        })
        .expect_err("AZ must belong to region");
        assert_eq!(az_error, CloudResourceError::AzRegionMismatch);

        let cell_error = Resource::new(ResourceCreate {
            cell_id: "cell-region-alpha1-b-001".to_string(),
            ..compute_resource_create()
        })
        .expect_err("cell must belong to AZ namespace");
        assert_eq!(cell_error, CloudResourceError::CellLocationMismatch);
    }

    #[test]
    fn rejects_operational_labels_as_resource_payload_data_class() {
        let error = Resource::new(ResourceCreate {
            data_class: DataClass::Audit,
            ..compute_resource_create()
        })
        .expect_err("resource payload class must be a privacy-program class");

        assert_eq!(error, CloudResourceError::InvalidDataClass);
    }

    #[test]
    fn rejects_reserved_or_empty_tenant_tags_and_duplicate_policy_ids() {
        let tag_error = Resource::new(ResourceCreate {
            tags: BTreeMap::from([("oyatie:internal".to_string(), "no".to_string())]),
            ..compute_resource_create()
        })
        .expect_err("tenant tags cannot use the reserved Oyatie prefix");
        assert_eq!(tag_error, CloudResourceError::InvalidTagKey);

        let policy_error = Resource::new(ResourceCreate {
            iam_policy_attachments: vec![
                "pol_cloud_compute_admin".to_string(),
                "pol_cloud_compute_admin".to_string(),
            ],
            ..compute_resource_create()
        })
        .expect_err("policy attachments must be unique");
        assert_eq!(policy_error, CloudResourceError::DuplicatePolicyId);
    }

    #[test]
    fn registry_rejects_duplicate_resource_id_and_applies_valid_state_transitions() {
        let mut registry = ResourceRegistry::default();
        let resource = registry
            .create(compute_resource_create())
            .expect("first create succeeds");
        assert_eq!(
            registry
                .create(compute_resource_create())
                .expect_err("duplicate resource id denied"),
            CloudResourceError::DuplicateResource
        );

        let running = registry
            .transition_state(&resource.id.value, ResourceState::Running, 1_700_000_010)
            .expect("pending can become running");
        assert_eq!(running.state.value, ResourceState::Running);

        let terminated = registry
            .transition_state(&resource.id.value, ResourceState::Terminated, 1_700_000_020)
            .expect("running can terminate");
        assert_eq!(terminated.state.value, ResourceState::Terminated);

        let error = registry
            .transition_state(&resource.id.value, ResourceState::Running, 1_700_000_030)
            .expect_err("terminated resources are terminal");
        assert_eq!(error, CloudResourceError::InvalidStateTransition);
    }

    #[test]
    fn rejects_residency_region_mismatch_and_wrong_metering_tag() {
        let residency_error = Resource::new(ResourceCreate {
            region: "region-gamma1".to_string(),
            az: Some("region-gamma1-a".to_string()),
            cell_id: "cell-region-gamma1-a-001".to_string(),
            id: "oyatie:cloud:region-gamma1:ten_alpha:instance:api-001".to_string(),
            ..compute_resource_create()
        })
        .expect_err("pack residency cannot move to a forbidden region");
        assert_eq!(residency_error, CloudResourceError::ResidencyRegionMismatch);

        let metering_error = Resource::new(ResourceCreate {
            metering_tag: "oyatie:metering:ten_alpha:bucket".to_string(),
            ..compute_resource_create()
        })
        .expect_err("metering tag must match tenant and kind");
        assert_eq!(metering_error, CloudResourceError::InvalidMeteringTag);
    }

    #[test]
    fn resource_state_as_str_and_parse_round_trip() {
        let variants = [
            ResourceState::Pending,
            ResourceState::Running,
            ResourceState::Stopped,
            ResourceState::Terminated,
            ResourceState::Error,
        ];
        for state in variants {
            assert_eq!(
                ResourceState::parse(state.as_str()),
                Some(state),
                "parse(as_str({state:?})) must round-trip"
            );
        }
    }

    #[test]
    fn resource_state_parse_rejects_unknown_inputs() {
        assert_eq!(ResourceState::parse("bogus"), None);
        assert_eq!(ResourceState::parse(""), None);
        assert_eq!(ResourceState::parse("RUNNING"), None);
        assert_eq!(ResourceState::parse("Pending"), None);
    }

    #[test]
    fn resource_state_classifiers_match_only_correct_variant() {
        assert!(ResourceState::Running.is_active());
        assert!(!ResourceState::Pending.is_active());
        assert!(!ResourceState::Stopped.is_active());
        assert!(!ResourceState::Terminated.is_active());
        assert!(!ResourceState::Error.is_active());

        assert!(ResourceState::Stopped.is_quiescent());
        assert!(!ResourceState::Pending.is_quiescent());
        assert!(!ResourceState::Running.is_quiescent());
        assert!(!ResourceState::Terminated.is_quiescent());
        assert!(!ResourceState::Error.is_quiescent());
    }

    #[test]
    fn terminated_allowed_next_contains_only_self_loop() {
        let nexts = ResourceState::Terminated.allowed_next();
        assert_eq!(nexts, &[ResourceState::Terminated]);
    }

    #[test]
    fn transition_graph_allowed_next_agrees_with_can_transition_to_for_all_pairs() {
        let all_states = [
            ResourceState::Pending,
            ResourceState::Running,
            ResourceState::Stopped,
            ResourceState::Terminated,
            ResourceState::Error,
        ];
        for &from in &all_states {
            let nexts = from.allowed_next();
            for &to in &all_states {
                let via_predicate = from.can_transition_to(to);
                let via_graph = nexts.contains(&to);
                assert_eq!(
                    via_predicate, via_graph,
                    "allowed_next({from:?}) and can_transition_to({from:?}, {to:?}) disagree: \
                     predicate={via_predicate}, graph={via_graph}"
                );
            }
        }
    }
}
