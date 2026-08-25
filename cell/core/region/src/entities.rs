use cell_routing::CellTier;
use data_boundary_kernel::{Classified, DataClass};
use network_residency::{
    RegionRef, RegionRefCreate, ResidencyClass, infer_region_jurisdiction_label,
    residency_class_allows_home_region_label,
};

use crate::model::{
    CLOUD_AZ_SCHEMA_VERSION, CLOUD_CELL_SCHEMA_VERSION, CLOUD_REGION_SCHEMA_VERSION, CellCapacity,
    CellUtilization, CloudAz, CloudAzCreate, CloudCell, CloudCellCreate, CloudRegion,
    CloudRegionCreate, CloudRegionError, TenantDensityClass,
};
use crate::validation::{
    validate_allowed_residency, validate_az_region, validate_capacity, validate_cell_id_namespace,
    validate_hsm_partition_ref, validate_non_empty, validate_power_zones,
    validate_regulatory_packs,
};
use crate::{AzCode, CellId, RegionCode};

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
    pub(crate) fn has_required_capacity(self) -> bool {
        self.compute_vcpu > 0 && self.memory_gb > 0 && self.ssd_tb > 0
    }

    pub(crate) fn contains(self, utilization: CellUtilization) -> bool {
        utilization.compute_vcpu_used <= self.compute_vcpu
            && utilization.memory_gb_used <= self.memory_gb
            && utilization.ssd_tb_used <= self.ssd_tb
            && utilization.gpu_count_used <= self.gpu_count
    }

    pub(crate) fn has_general_headroom(self, utilization: CellUtilization) -> bool {
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

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}
