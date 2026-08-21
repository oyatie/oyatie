//! Cloud Region/AZ/Cell taxonomy kernel.
//!
//! This crate owns the `REGION_AZ_CELL` contract named by the architecture
//! documents. It keeps cloud taxonomy public where the control-plane contract is
//! public, keeps placement-sensitive details internal, and delegates residency
//! and cell-binding enforcement to their platform kernels.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::{BTreeMap, BTreeSet};

use cell_routing::{CellBinding, CellBindingCreate, CellError, CellRouter, CellTier};
use network_residency::{
    RegionRef, RegionRefCreate, ResidencyClass, ResidencyError, infer_region_jurisdiction_label,
    residency_class_allows_home_region_label,
};
use data_boundary_kernel::{Classified, DataClass};

const CLOUD_REGION_SCHEMA_VERSION: u32 = 1;
const CLOUD_AZ_SCHEMA_VERSION: u32 = 1;
const CLOUD_CELL_SCHEMA_VERSION: u32 = 1;
const CELL_ID_PREFIX: &str = "cell-";
const REGIONAL_PACK_ID_PREFIX: &str = "oya-pack-";
const HSM_PARTITION_PREFIX: &str = "hsm/";
const TENANT_ID_PREFIX: &str = "ten_";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RegionCode {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct AzCode {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CellId {
    pub value: String, // data_class: PUBLIC
}

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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CloudRegionCatalog {
    regions: BTreeMap<RegionCode, CloudRegion>,
    azs: BTreeMap<AzCode, CloudAz>,
    cells: BTreeMap<CellId, CloudCell>,
}

impl RegionCode {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudRegionError> {
        let value = value.into();
        validate_canonical_code(&value, CloudRegionError::InvalidRegionCode)?;
        Ok(Self { value })
    }
}

impl AzCode {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudRegionError> {
        let value = value.into();
        validate_canonical_code(&value, CloudRegionError::InvalidAzCode)?;
        Ok(Self { value })
    }
}

impl CellId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudRegionError> {
        let value = value.into();
        if !value.starts_with(CELL_ID_PREFIX) {
            return Err(CloudRegionError::InvalidCellId);
        }
        validate_canonical_code(&value, CloudRegionError::InvalidCellId)?;
        Ok(Self { value })
    }
}

impl TenantDensityClass {
    pub const fn cell_tier(self) -> CellTier {
        match self {
            Self::Shared => CellTier::Shared,
            Self::Dedicated => CellTier::Dedicated,
            Self::Sovereign | Self::AirGapped => CellTier::SovereignAirGapped,
            Self::FoundryRuntime => CellTier::FoundryRuntime,
        }
    }
}

impl CellCapacity {
    fn has_required_capacity(self) -> bool {
        self.compute_vcpu > 0 && self.memory_gb > 0 && self.ssd_tb > 0
    }

    fn contains(self, utilization: CellUtilization) -> bool {
        utilization.compute_vcpu_used <= self.compute_vcpu
            && utilization.memory_gb_used <= self.memory_gb
            && utilization.ssd_tb_used <= self.ssd_tb
            && utilization.gpu_count_used <= self.gpu_count
    }

    fn has_general_headroom(self, utilization: CellUtilization) -> bool {
        utilization.compute_vcpu_used < self.compute_vcpu
            && utilization.memory_gb_used < self.memory_gb
            && utilization.ssd_tb_used < self.ssd_tb
    }
}

impl CloudRegion {
    pub fn new(input: CloudRegionCreate) -> Result<Self, CloudRegionError> {
        let code = RegionCode::new(input.code)?;
        validate_non_empty(&input.display_name, CloudRegionError::InvalidDisplayName)?;
        validate_regulatory_packs(&input.regulatory_packs)?;
        if !residency_class_allows_home_region_label(&input.residency_strictness, &code.value) {
            return Err(CloudRegionError::RegionResidencyMismatch);
        }
        let region_ref = RegionRef::new(RegionRefCreate {
            region_id: code.value.clone(),
            jurisdiction: infer_region_jurisdiction_label(&code.value),
            cell_group_ref: format!("cells/{}", code.value),
        })
        .map_err(CloudRegionError::ResidencyReferenceRejected)?;
        Ok(Self {
            code: public(code),
            display_name: public(input.display_name),
            regulatory_packs: public(input.regulatory_packs),
            azs: public(Vec::new()),
            state: public(input.state),
            provider_facing: public(input.provider_facing),
            residency_strictness: public(input.residency_strictness),
            region_ref: internal(region_ref),
            created_at_epoch_seconds: public(input.created_at_epoch_seconds),
            schema_version: public(CLOUD_REGION_SCHEMA_VERSION),
        })
    }
}

impl CloudAz {
    pub fn new(input: CloudAzCreate) -> Result<Self, CloudRegionError> {
        let code = AzCode::new(input.code)?;
        let region_code = RegionCode::new(input.region_code)?;
        validate_az_region(&code, &region_code)?;
        validate_non_empty(&input.physical_ref, CloudRegionError::InvalidPhysicalRef)?;
        validate_power_zones(&input.power_zones)?;
        Ok(Self {
            code: public(code),
            region_code: public(region_code),
            physical_ref: internal(input.physical_ref),
            power_zones: public(input.power_zones),
            cells: public(Vec::new()),
            state: public(input.state),
            created_at_epoch_seconds: public(input.created_at_epoch_seconds),
            schema_version: public(CLOUD_AZ_SCHEMA_VERSION),
        })
    }
}

impl CloudCell {
    pub fn new(input: CloudCellCreate) -> Result<Self, CloudRegionError> {
        let id = CellId::new(input.id)?;
        let region_code = RegionCode::new(input.region_code)?;
        let az_code = AzCode::new(input.az_code)?;
        validate_az_region(&az_code, &region_code)?;
        validate_cell_id_namespace(&id, &az_code)?;
        validate_hsm_partition_ref(&input.hsm_partition_ref, &region_code, &id)?;
        validate_capacity(input.capacity, input.utilization)?;
        validate_allowed_residency(&region_code, &input.allowed_residency)?;
        Ok(Self {
            id: public(id),
            region_code: public(region_code),
            az_code: public(az_code),
            state: public(input.state),
            tenant_density: public(input.tenant_density),
            allowed_residency: public(input.allowed_residency),
            capacity: internal(input.capacity),
            utilization: internal(input.utilization),
            hsm_partition_ref: internal(input.hsm_partition_ref),
            created_at_epoch_seconds: public(input.created_at_epoch_seconds),
            schema_version: public(CLOUD_CELL_SCHEMA_VERSION),
        })
    }

    pub fn allows_residency(&self, residency_class: &ResidencyClass) -> bool {
        self.allowed_residency.value.contains(residency_class)
    }

    pub fn has_route_capacity(&self) -> bool {
        self.capacity
            .value
            .has_general_headroom(self.utilization.value)
    }
}

impl CloudRegionCatalog {
    pub fn register_region(
        &mut self,
        input: CloudRegionCreate,
    ) -> Result<CloudRegion, CloudRegionError> {
        let region = CloudRegion::new(input)?;
        if self.regions.contains_key(&region.code.value) {
            return Err(CloudRegionError::DuplicateRegion);
        }
        self.regions
            .insert(region.code.value.clone(), region.clone());
        Ok(region)
    }

    pub fn register_az(&mut self, input: CloudAzCreate) -> Result<CloudAz, CloudRegionError> {
        let az = CloudAz::new(input)?;
        if self.azs.contains_key(&az.code.value) {
            return Err(CloudRegionError::DuplicateAz);
        }
        let region = self
            .regions
            .get_mut(&az.region_code.value)
            .ok_or(CloudRegionError::UnknownRegion)?;
        region.azs.value.push(az.code.value.clone());
        region.azs.value.sort();
        region.azs.value.dedup();
        self.azs.insert(az.code.value.clone(), az.clone());
        Ok(az)
    }

    pub fn register_cell(&mut self, input: CloudCellCreate) -> Result<CloudCell, CloudRegionError> {
        let cell = CloudCell::new(input)?;
        if self.cells.contains_key(&cell.id.value) {
            return Err(CloudRegionError::DuplicateCell);
        }
        let region = self
            .regions
            .get(&cell.region_code.value)
            .ok_or(CloudRegionError::UnknownRegion)?;
        let az = self
            .azs
            .get_mut(&cell.az_code.value)
            .ok_or(CloudRegionError::UnknownAz)?;
        if az.region_code.value != cell.region_code.value {
            return Err(CloudRegionError::CellAzMismatch);
        }
        for residency_class in &cell.allowed_residency.value {
            if !region_allows_residency(region, residency_class) {
                return Err(CloudRegionError::CellResidencyNotAllowedInRegion);
            }
        }
        az.cells.value.push(cell.id.value.clone());
        az.cells.value.sort();
        az.cells.value.dedup();
        self.cells.insert(cell.id.value.clone(), cell.clone());
        Ok(cell)
    }

    pub fn region(&self, code: &RegionCode) -> Option<&CloudRegion> {
        self.regions.get(code)
    }

    pub fn az(&self, code: &AzCode) -> Option<&CloudAz> {
        self.azs.get(code)
    }

    pub fn cell(&self, id: &CellId) -> Option<&CloudCell> {
        self.cells.get(id)
    }

    pub fn regions(&self) -> impl Iterator<Item = &CloudRegion> {
        self.regions.values()
    }

    pub fn azs_for_region<'a>(
        &'a self,
        region_code: &'a RegionCode,
    ) -> impl Iterator<Item = &'a CloudAz> + 'a {
        self.azs
            .values()
            .filter(move |az| &az.region_code.value == region_code)
    }

    pub fn cells_for_region<'a>(
        &'a self,
        region_code: &'a RegionCode,
    ) -> impl Iterator<Item = &'a CloudCell> + 'a {
        self.cells
            .values()
            .filter(move |cell| &cell.region_code.value == region_code)
    }

    pub fn route_for_tenant(
        &self,
        request: TenantCellRouteRequest,
    ) -> Result<CellBindingCreate, CloudRegionError> {
        validate_tenant_id(&request.tenant_id)?;
        let home_region = RegionCode::new(request.home_region_code)?;
        let region = self
            .regions
            .get(&home_region)
            .ok_or(CloudRegionError::UnknownRegion)?;
        if !region_allows_residency(region, &request.residency_class) {
            return Err(CloudRegionError::RegionResidencyMismatch);
        }
        let cell = self
            .cells
            .values()
            .find(|cell| {
                cell.region_code.value == home_region
                    && cell.state.value == CloudCellState::Active
                    && request
                        .required_density
                        .is_none_or(|density| density == cell.tenant_density.value)
                    && cell.allows_residency(&request.residency_class)
                    && cell.has_route_capacity()
            })
            .ok_or(CloudRegionError::NoCompatibleCell)?;
        self.binding_for_cell(request.tenant_id, request.residency_class, &cell.id.value)
    }

    pub fn binding_for_cell(
        &self,
        tenant_id: String,
        residency_class: ResidencyClass,
        cell_id: &CellId,
    ) -> Result<CellBindingCreate, CloudRegionError> {
        validate_tenant_id(&tenant_id)?;
        let cell = self
            .cells
            .get(cell_id)
            .ok_or(CloudRegionError::UnknownCell)?;
        let region = self
            .regions
            .get(&cell.region_code.value)
            .ok_or(CloudRegionError::UnknownRegion)?;
        if !region_allows_residency(region, &residency_class) {
            return Err(CloudRegionError::RegionResidencyMismatch);
        }
        if !cell.allows_residency(&residency_class) {
            return Err(CloudRegionError::CellResidencyDenied);
        }
        Ok(CellBindingCreate {
            tenant_id,
            region: region.region_ref.value.clone(),
            residency_class,
            az: cell.az_code.value.value.clone(),
            cell_id: cell.id.value.value.clone(),
            tier: cell.tenant_density.value.cell_tier(),
            hsm_partition_ref: cell.hsm_partition_ref.value.clone(),
        })
    }

    pub fn bind_route_for_tenant(
        &self,
        router: &mut CellRouter,
        request: TenantCellRouteRequest,
    ) -> Result<CellBinding, CloudRegionError> {
        let binding = self.route_for_tenant(request)?;
        router
            .bind(binding)
            .map_err(CloudRegionError::CellBindingRejected)
    }
}

fn region_allows_residency(region: &CloudRegion, residency_class: &ResidencyClass) -> bool {
    residency_class_allows_home_region_label(residency_class, &region.code.value.value)
}

fn validate_canonical_code(value: &str, error: CloudRegionError) -> Result<(), CloudRegionError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed != value
        || trimmed.starts_with('-')
        || trimmed.ends_with('-')
        || trimmed.contains("--")
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(error);
    }
    Ok(())
}

fn validate_az_region(az: &AzCode, region: &RegionCode) -> Result<(), CloudRegionError> {
    if az.value == region.value
        || az
            .value
            .strip_prefix(&region.value)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix.len() > 1)
    {
        Ok(())
    } else {
        Err(CloudRegionError::AzRegionMismatch)
    }
}

fn validate_regulatory_packs(packs: &[String]) -> Result<(), CloudRegionError> {
    validate_non_empty_set(
        packs,
        CloudRegionError::EmptyRegulatoryPackSet,
        CloudRegionError::InvalidRegulatoryPack,
        CloudRegionError::DuplicateRegulatoryPack,
        |pack| pack.starts_with(REGIONAL_PACK_ID_PREFIX),
    )
}

fn validate_power_zones(power_zones: &[String]) -> Result<(), CloudRegionError> {
    validate_non_empty_set(
        power_zones,
        CloudRegionError::EmptyPowerZoneSet,
        CloudRegionError::InvalidPowerZone,
        CloudRegionError::DuplicatePowerZone,
        |_| true,
    )
}

fn validate_allowed_residency(
    region_code: &RegionCode,
    residency_classes: &[ResidencyClass],
) -> Result<(), CloudRegionError> {
    if residency_classes.is_empty() {
        return Err(CloudRegionError::EmptyAllowedResidencySet);
    }
    let mut seen = BTreeSet::new();
    for residency_class in residency_classes {
        if !seen.insert(residency_class.clone()) {
            return Err(CloudRegionError::DuplicateAllowedResidencyClass);
        }
        if !residency_class_allows_home_region_label(residency_class, &region_code.value) {
            return Err(CloudRegionError::CellResidencyNotAllowedInRegion);
        }
    }
    Ok(())
}

fn validate_capacity(
    capacity: CellCapacity,
    utilization: CellUtilization,
) -> Result<(), CloudRegionError> {
    if !capacity.has_required_capacity() {
        return Err(CloudRegionError::InvalidCapacity);
    }
    if !capacity.contains(utilization) {
        return Err(CloudRegionError::UtilizationExceedsCapacity);
    }
    Ok(())
}

fn validate_cell_id_namespace(cell_id: &CellId, az_code: &AzCode) -> Result<(), CloudRegionError> {
    let expected_prefix = format!("{CELL_ID_PREFIX}{}-", az_code.value);
    if cell_id.value.starts_with(&expected_prefix) {
        Ok(())
    } else {
        Err(CloudRegionError::CellAzMismatch)
    }
}

fn validate_hsm_partition_ref(
    value: &str,
    region_code: &RegionCode,
    cell_id: &CellId,
) -> Result<(), CloudRegionError> {
    validate_non_empty(value, CloudRegionError::InvalidHsmPartitionRef)?;
    let expected = format!(
        "{HSM_PARTITION_PREFIX}{}/{}",
        region_code.value, cell_id.value
    );
    if value == expected {
        Ok(())
    } else {
        Err(CloudRegionError::InvalidHsmPartitionRef)
    }
}

fn validate_tenant_id(value: &str) -> Result<(), CloudRegionError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudRegionError::InvalidTenantId)
    }
}

fn validate_non_empty(value: &str, error: CloudRegionError) -> Result<(), CloudRegionError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn validate_non_empty_set(
    values: &[String],
    empty_error: CloudRegionError,
    invalid_error: CloudRegionError,
    duplicate_error: CloudRegionError,
    accepts: impl Fn(&str) -> bool,
) -> Result<(), CloudRegionError> {
    if values.is_empty() {
        return Err(empty_error);
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() || !accepts(value) {
            return Err(invalid_error);
        }
        if !seen.insert(value) {
            return Err(duplicate_error);
        }
    }
    Ok(())
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
                    regulator_refs: vec!["regulator/cloud-region".to_string()],
                    evidence_ref: "evidence/residency/cloud-region".to_string(),
                })
                .expect("regulator overlay fixture is valid"),
            })
            .expect("per-pack residency fixture is valid"),
        ))
    }

    fn region_create() -> CloudRegionCreate {
        CloudRegionCreate {
            code: "region-alpha1".to_string(),
            display_name: "Alpha Region".to_string(),
            regulatory_packs: vec!["oya-pack-alpha".to_string()],
            state: RegionState::Preview,
            provider_facing: true,
            residency_strictness: residency_class(),
            created_at_epoch_seconds: 1_700_000_000,
        }
    }

    fn az_create() -> CloudAzCreate {
        CloudAzCreate {
            code: "region-alpha1-a".to_string(),
            region_code: "region-alpha1".to_string(),
            physical_ref: "dc/region-alpha1/a".to_string(),
            power_zones: vec!["pz-a1".to_string(), "pz-a2".to_string()],
            state: AzState::Active,
            created_at_epoch_seconds: 1_700_000_010,
        }
    }

    fn cell_create() -> CloudCellCreate {
        CloudCellCreate {
            id: "cell-region-alpha1-a-001".to_string(),
            region_code: "region-alpha1".to_string(),
            az_code: "region-alpha1-a".to_string(),
            state: CloudCellState::Active,
            tenant_density: TenantDensityClass::Shared,
            allowed_residency: vec![residency_class()],
            capacity: CellCapacity {
                compute_vcpu: 128,
                memory_gb: 512,
                ssd_tb: 40,
                gpu_count: 0,
            },
            utilization: CellUtilization {
                compute_vcpu_used: 32,
                memory_gb_used: 128,
                ssd_tb_used: 10,
                gpu_count_used: 0,
            },
            hsm_partition_ref: "hsm/region-alpha1/cell-region-alpha1-a-001".to_string(),
            created_at_epoch_seconds: 1_700_000_020,
        }
    }

    fn catalog_with_cell() -> CloudRegionCatalog {
        let mut catalog = CloudRegionCatalog::default();
        catalog
            .register_region(region_create())
            .expect("region fixture registers");
        catalog
            .register_az(az_create())
            .expect("AZ fixture registers");
        catalog
            .register_cell(cell_create())
            .expect("cell fixture registers");
        catalog
    }

    #[test]
    fn registers_region_az_cell_and_produces_platform_cell_binding() {
        let catalog = catalog_with_cell();
        let mut router = CellRouter::default();

        let binding = catalog
            .bind_route_for_tenant(
                &mut router,
                TenantCellRouteRequest {
                    tenant_id: "ten_alpha".to_string(),
                    home_region_code: "region-alpha1".to_string(),
                    residency_class: residency_class(),
                    required_density: Some(TenantDensityClass::Shared),
                },
            )
            .expect("route should satisfy platform cell binding invariants");

        assert_eq!(binding.tenant_id, "ten_alpha");
        assert_eq!(binding.region, "region-alpha1");
        assert_eq!(binding.az.value, "region-alpha1-a");
        assert_eq!(binding.cell_id.value, "cell-region-alpha1-a-001");
        assert_eq!(binding.tier.value, CellTier::Shared);
        assert_eq!(binding.residency_class.value, residency_class());
    }

    #[test]
    fn rejects_duplicate_region_az_and_cell_ids() {
        let mut catalog = CloudRegionCatalog::default();
        catalog
            .register_region(region_create())
            .expect("first region registers");
        assert_eq!(
            catalog
                .register_region(region_create())
                .expect_err("duplicate region denied"),
            CloudRegionError::DuplicateRegion
        );

        catalog
            .register_az(az_create())
            .expect("first AZ registers");
        assert_eq!(
            catalog
                .register_az(az_create())
                .expect_err("duplicate AZ denied"),
            CloudRegionError::DuplicateAz
        );

        catalog
            .register_cell(cell_create())
            .expect("first cell registers");
        assert_eq!(
            catalog
                .register_cell(cell_create())
                .expect_err("duplicate cell denied"),
            CloudRegionError::DuplicateCell
        );
    }

    #[test]
    fn rejects_az_outside_region() {
        let error = CloudAz::new(CloudAzCreate {
            code: "region-gamma1-a".to_string(),
            ..az_create()
        })
        .expect_err("AZ code must sit under its region code");

        assert_eq!(error, CloudRegionError::AzRegionMismatch);
    }

    #[test]
    fn rejects_cell_id_that_does_not_match_az_namespace() {
        let error = CloudCell::new(CloudCellCreate {
            id: "cell-region-alpha1-b-001".to_string(),
            ..cell_create()
        })
        .expect_err("cell ids must be namespaced under their AZ");

        assert_eq!(error, CloudRegionError::CellAzMismatch);
    }

    #[test]
    fn rejects_cell_without_registered_region_or_az() {
        let mut catalog = CloudRegionCatalog::default();
        catalog
            .register_region(region_create())
            .expect("region registers");

        assert_eq!(
            catalog
                .register_cell(cell_create())
                .expect_err("cell must reference a registered AZ"),
            CloudRegionError::UnknownAz
        );
    }

    #[test]
    fn rejects_cell_when_residency_is_not_allowed_in_region() {
        let error = CloudCell::new(CloudCellCreate {
            allowed_residency: vec![residency_class()],
            region_code: "region-gamma1".to_string(),
            az_code: "region-gamma1-a".to_string(),
            id: "cell-region-gamma1-a-001".to_string(),
            hsm_partition_ref: "hsm/region-gamma1/cell-region-gamma1-a-001".to_string(),
            ..cell_create()
        })
        .expect_err("pack residency cannot be advertised in a forbidden region");

        assert_eq!(error, CloudRegionError::CellResidencyNotAllowedInRegion);
    }

    #[test]
    fn route_denies_residency_class_not_allowed_by_cell() {
        let catalog = catalog_with_cell();

        let error = catalog
            .route_for_tenant(TenantCellRouteRequest {
                tenant_id: "ten_global".to_string(),
                home_region_code: "region-alpha1".to_string(),
                residency_class: ResidencyClass::Global,
                required_density: None,
            })
            .expect_err("cell does not advertise pack residency");

        assert_eq!(error, CloudRegionError::NoCompatibleCell);
    }

    #[test]
    fn rejects_cell_capacity_that_is_over_allocated() {
        let error = CloudCell::new(CloudCellCreate {
            utilization: CellUtilization {
                compute_vcpu_used: 129,
                memory_gb_used: 128,
                ssd_tb_used: 10,
                gpu_count_used: 0,
            },
            ..cell_create()
        })
        .expect_err("utilization cannot exceed declared capacity");

        assert_eq!(error, CloudRegionError::UtilizationExceedsCapacity);
    }

    #[test]
    fn direct_cell_binding_revalidates_residency_against_cell_manifest() {
        let catalog = catalog_with_cell();
        let cell_id = CellId::new("cell-region-alpha1-a-001").expect("fixture cell id is valid");

        // Cell manifest declares `allowed_residency: [PerPack(...)]`. Binding requests
        // `Global` — region check passes (any non-empty region code), then cell
        // manifest check denies because Global is not in the cell's allowed list.
        let error = catalog
            .binding_for_cell("ten_global".to_string(), ResidencyClass::Global, &cell_id)
            .expect_err("binding must not ignore the cell residency manifest");

        assert_eq!(error, CloudRegionError::CellResidencyDenied);
    }
}
