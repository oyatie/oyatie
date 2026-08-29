//! Compute, storage, and network product surfaces.

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::error::CloudSurfaceError;
use crate::fulfillment::{ComputeSkuSurface, ComputeSkuSurfaceCreate, validate_compute_skus};
use crate::ids::CloudSurfaceId;
use crate::validate::{internal, prefixed_token, public, public_class, validate_nonempty};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum StorageSurfaceKind {
    Object,
    Block,
    File,
    Archive,
    Database,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkSurfaceKind {
    Vpc,
    LoadBalancer,
    Dns,
    Interconnect,
    DdosProtection,
    ServiceMesh,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeSurfaceCreate {
    pub skus: Vec<ComputeSkuSurfaceCreate>, // data_class: PUBLIC
    pub data_class: DataClass,              // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeSurface {
    pub skus: Classified<Vec<ComputeSkuSurface>>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSurfaceCreate {
    pub surfaces: Vec<StorageSurfaceKind>, // data_class: PUBLIC
    pub s3_compatible_object_api: bool,    // data_class: PUBLIC
    pub nvme_block_tiers: bool,            // data_class: PUBLIC
    pub nfs41_smb3_file_api: bool,         // data_class: PUBLIC
    pub cold_archive_tier: bool,           // data_class: PUBLIC
    pub per_cell_key_material: bool,       // data_class: PUBLIC
    pub data_class: DataClass,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageSurface {
    pub surfaces: Classified<Vec<StorageSurfaceKind>>, // data_class: PUBLIC
    pub s3_compatible_object_api: Classified<bool>,    // data_class: PUBLIC
    pub nvme_block_tiers: Classified<bool>,            // data_class: PUBLIC
    pub nfs41_smb3_file_api: Classified<bool>,         // data_class: PUBLIC
    pub cold_archive_tier: Classified<bool>,           // data_class: PUBLIC
    pub per_cell_key_material: Classified<bool>,       // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkSurfaceCreate {
    pub surfaces: Vec<NetworkSurfaceKind>,    // data_class: PUBLIC
    pub per_tenant_per_cell_vpc: bool,        // data_class: PUBLIC
    pub l4_l7_load_balancing: bool,           // data_class: PUBLIC
    pub mtls_termination: bool,               // data_class: PUBLIC
    pub dnssec: bool,                         // data_class: PUBLIC
    pub direct_interconnect_all_phases: bool, // data_class: PUBLIC
    pub regional_line_rate_scrubbing: bool,   // data_class: PUBLIC
    pub data_class: DataClass,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NetworkSurface {
    pub surfaces: Classified<Vec<NetworkSurfaceKind>>, // data_class: PUBLIC
    pub per_tenant_per_cell_vpc: Classified<bool>,     // data_class: PUBLIC
    pub l4_l7_load_balancing: Classified<bool>,        // data_class: PUBLIC
    pub mtls_termination: Classified<bool>,            // data_class: PUBLIC
    pub dnssec: Classified<bool>,                      // data_class: PUBLIC
    pub direct_interconnect_all_phases: Classified<bool>, // data_class: PUBLIC
    pub regional_line_rate_scrubbing: Classified<bool>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>,      // data_class: PUBLIC
}

impl ComputeSurface {
    pub fn new(input: ComputeSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        let skus = input
            .skus
            .into_iter()
            .map(ComputeSkuSurface::new)
            .collect::<Result<Vec<_>, _>>()?;
        validate_compute_skus(&skus)?;
        Ok(Self {
            skus: public(skus),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl StorageSurface {
    pub fn new(input: StorageSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_storage_surface(&input)?;
        Ok(Self {
            surfaces: public(input.surfaces),
            s3_compatible_object_api: public(input.s3_compatible_object_api),
            nvme_block_tiers: public(input.nvme_block_tiers),
            nfs41_smb3_file_api: public(input.nfs41_smb3_file_api),
            cold_archive_tier: public(input.cold_archive_tier),
            per_cell_key_material: public(input.per_cell_key_material),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl NetworkSurface {
    pub fn new(input: NetworkSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_network_surface(&input)?;
        Ok(Self {
            surfaces: public(input.surfaces),
            per_tenant_per_cell_vpc: public(input.per_tenant_per_cell_vpc),
            l4_l7_load_balancing: public(input.l4_l7_load_balancing),
            mtls_termination: public(input.mtls_termination),
            dnssec: public(input.dnssec),
            direct_interconnect_all_phases: public(input.direct_interconnect_all_phases),
            regional_line_rate_scrubbing: public(input.regional_line_rate_scrubbing),
            data_class: public_class(input.data_class)?,
        })
    }
}

fn validate_storage_surface(input: &StorageSurfaceCreate) -> Result<(), CloudSurfaceError> {
    let surfaces = input.surfaces.iter().copied().collect::<BTreeSet<_>>();
    let required = BTreeSet::from([
        StorageSurfaceKind::Object,
        StorageSurfaceKind::Block,
        StorageSurfaceKind::File,
        StorageSurfaceKind::Archive,
        StorageSurfaceKind::Database,
    ]);
    if surfaces == required
        && input.s3_compatible_object_api
        && input.nvme_block_tiers
        && input.nfs41_smb3_file_api
        && input.cold_archive_tier
        && input.per_cell_key_material
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::MissingStorageSurface)
    }
}

fn validate_network_surface(input: &NetworkSurfaceCreate) -> Result<(), CloudSurfaceError> {
    let surfaces = input.surfaces.iter().copied().collect::<BTreeSet<_>>();
    let required = BTreeSet::from([
        NetworkSurfaceKind::Vpc,
        NetworkSurfaceKind::LoadBalancer,
        NetworkSurfaceKind::Dns,
        NetworkSurfaceKind::Interconnect,
        NetworkSurfaceKind::DdosProtection,
        NetworkSurfaceKind::ServiceMesh,
    ]);
    if surfaces == required
        && input.per_tenant_per_cell_vpc
        && input.l4_l7_load_balancing
        && input.mtls_termination
        && input.dnssec
        && input.direct_interconnect_all_phases
        && input.regional_line_rate_scrubbing
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::MissingNetworkSurface)
    }
}
