use std::collections::BTreeMap;

use cell_location::{AzCode, CellId, CellLocationError, RegionCode};
use cell_routing::CellError;
use data_boundary_kernel::Classified;
use network_residency::{RegionRef, ResidencyClass, ResidencyError};

pub(crate) const CLOUD_REGION_SCHEMA_VERSION: u32 = 1;
pub(crate) const CLOUD_AZ_SCHEMA_VERSION: u32 = 1;
pub(crate) const CLOUD_CELL_SCHEMA_VERSION: u32 = 1;
pub(crate) const REGIONAL_PACK_ID_PREFIX: &str = "pack-";
pub(crate) const HSM_PARTITION_PREFIX: &str = "hsm/";
pub(crate) const TENANT_ID_PREFIX: &str = "ten_";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RegionState {
    Planned,
    Preview,
    Ga,
    Retiring,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AzState {
    Planned,
    Active,
    DrOnly,
    Retiring,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CloudCellState {
    Planned,
    Active,
    DrOnly,
    Draining,
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TenantDensityClass {
    Shared,
    Dedicated,
    Sovereign,
    AirGapped,
    FoundryRuntime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegionCreate {
    pub code: String,                         // data_class: PUBLIC
    pub display_name: String,                 // data_class: PUBLIC
    pub regulatory_packs: Vec<String>,        // data_class: PUBLIC
    pub state: RegionState,                   // data_class: PUBLIC
    pub provider_facing: bool,                // data_class: PUBLIC
    pub residency_strictness: ResidencyClass, // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,        // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudRegion {
    pub code: Classified<RegionCode>,              // data_class: PUBLIC
    pub display_name: Classified<String>,          // data_class: PUBLIC
    pub regulatory_packs: Classified<Vec<String>>, // data_class: PUBLIC
    pub azs: Classified<Vec<AzCode>>,              // data_class: PUBLIC
    pub state: Classified<RegionState>,            // data_class: PUBLIC
    pub provider_facing: Classified<bool>,         // data_class: PUBLIC
    pub residency_strictness: Classified<ResidencyClass>, // data_class: PUBLIC
    pub region_ref: Classified<RegionRef>,         // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAzCreate {
    pub code: String,                  // data_class: PUBLIC
    pub region_code: String,           // data_class: PUBLIC
    pub physical_ref: String,          // data_class: INTERNAL_ONLY
    pub power_zones: Vec<String>,      // data_class: PUBLIC
    pub state: AzState,                // data_class: PUBLIC
    pub created_at_epoch_seconds: u64, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudAz {
    pub code: Classified<AzCode>,                  // data_class: PUBLIC
    pub region_code: Classified<RegionCode>,       // data_class: PUBLIC
    pub physical_ref: Classified<String>,          // data_class: INTERNAL_ONLY
    pub power_zones: Classified<Vec<String>>,      // data_class: PUBLIC
    pub cells: Classified<Vec<CellId>>,            // data_class: PUBLIC
    pub state: Classified<AzState>,                // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,           // data_class: PUBLIC
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellCapacity {
    pub compute_vcpu: u64, // data_class: INTERNAL_ONLY
    pub memory_gb: u64,    // data_class: INTERNAL_ONLY
    pub ssd_tb: u64,       // data_class: INTERNAL_ONLY
    pub gpu_count: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellUtilization {
    pub compute_vcpu_used: u64, // data_class: INTERNAL_ONLY
    pub memory_gb_used: u64,    // data_class: INTERNAL_ONLY
    pub ssd_tb_used: u64,       // data_class: INTERNAL_ONLY
    pub gpu_count_used: u64,    // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCellCreate {
    pub id: String,                             // data_class: PUBLIC
    pub region_code: String,                    // data_class: PUBLIC
    pub az_code: String,                        // data_class: PUBLIC
    pub state: CloudCellState,                  // data_class: PUBLIC
    pub tenant_density: TenantDensityClass,     // data_class: PUBLIC
    pub allowed_residency: Vec<ResidencyClass>, // data_class: PUBLIC
    pub capacity: CellCapacity,                 // data_class: INTERNAL_ONLY
    pub utilization: CellUtilization,           // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: String,              // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: u64,          // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudCell {
    pub id: Classified<CellId>,                         // data_class: PUBLIC
    pub region_code: Classified<RegionCode>,            // data_class: PUBLIC
    pub az_code: Classified<AzCode>,                    // data_class: PUBLIC
    pub state: Classified<CloudCellState>,              // data_class: PUBLIC
    pub tenant_density: Classified<TenantDensityClass>, // data_class: PUBLIC
    pub allowed_residency: Classified<Vec<ResidencyClass>>, // data_class: PUBLIC
    pub capacity: Classified<CellCapacity>,             // data_class: INTERNAL_ONLY
    pub utilization: Classified<CellUtilization>,       // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: Classified<String>,          // data_class: INTERNAL_ONLY
    pub created_at_epoch_seconds: Classified<u64>,      // data_class: PUBLIC
    pub schema_version: Classified<u32>,                // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TenantCellRouteRequest {
    pub tenant_id: String,                            // data_class: INTERNAL_ONLY
    pub home_region_code: String,                     // data_class: INTERNAL_ONLY
    pub residency_class: ResidencyClass,              // data_class: INTERNAL_ONLY
    pub required_density: Option<TenantDensityClass>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudRegionError {
    InvalidRegionCode,
    InvalidAzCode,
    InvalidCellId,
    InvalidDisplayName,
    InvalidRegulatoryPack,
    EmptyRegulatoryPackSet,
    DuplicateRegulatoryPack,
    InvalidPhysicalRef,
    InvalidPowerZone,
    EmptyPowerZoneSet,
    DuplicatePowerZone,
    InvalidHsmPartitionRef,
    InvalidTenantId,
    InvalidCapacity,
    UtilizationExceedsCapacity,
    RegionResidencyMismatch,
    EmptyAllowedResidencySet,
    DuplicateAllowedResidencyClass,
    CellResidencyNotAllowedInRegion,
    CellResidencyDenied,
    DuplicateRegion,
    DuplicateAz,
    DuplicateCell,
    UnknownRegion,
    UnknownAz,
    UnknownCell,
    AzRegionMismatch,
    CellRegionMismatch,
    CellAzMismatch,
    NoCompatibleCell,
    CellBindingRejected(CellError),
    ResidencyReferenceRejected(ResidencyError),
}

impl From<CellLocationError> for CloudRegionError {
    fn from(error: CellLocationError) -> Self {
        match error {
            CellLocationError::InvalidRegionCode => Self::InvalidRegionCode,
            CellLocationError::InvalidAzCode => Self::InvalidAzCode,
            CellLocationError::InvalidCellId => Self::InvalidCellId,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudRegionCatalog {
    pub(crate) regions: BTreeMap<RegionCode, CloudRegion>,
    pub(crate) azs: BTreeMap<AzCode, CloudAz>,
    pub(crate) cells: BTreeMap<CellId, CloudCell>,
}
