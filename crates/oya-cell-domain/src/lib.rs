//! Cell-routing kernel: immutable per-tenant per-region cell placement.

use std::collections::BTreeMap;

use oya_data_boundary_kernel::{Classified, PrivacyDataClass};
use oya_residency_domain::{RegionRef, ResidencyClass, residency_class_allows_home_region_label};

const CELL_BINDING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum CellTier {
    Shared,
    Pooled,
    Dedicated,
    SovereignAirGapped,
    FoundryRuntime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellBindingCreate {
    pub tenant_id: String,               // data_class: INTERNAL_ONLY
    pub region: RegionRef,               // data_class: INTERNAL_ONLY
    pub residency_class: ResidencyClass, // data_class: INTERNAL_ONLY
    pub az: String,                      // data_class: INTERNAL_ONLY
    pub cell_id: String,                 // data_class: INTERNAL_ONLY
    pub tier: CellTier,                  // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: String,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CellBinding {
    pub tenant_id: String,                           // data_class: INTERNAL_ONLY
    pub region: String,                              // data_class: INTERNAL_ONLY
    pub region_ref: Classified<RegionRef>,           // data_class: INTERNAL_ONLY
    pub residency_class: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub az: Classified<String>,                      // data_class: INTERNAL_ONLY
    pub cell_id: Classified<String>,                 // data_class: INTERNAL_ONLY
    pub tier: Classified<CellTier>,                  // data_class: INTERNAL_ONLY
    pub hsm_partition_ref: Classified<String>,       // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CellError {
    AlreadyBound,
    InvalidTenantId,
    EmptyAz,
    EmptyCell,
    EmptyHsmPartition,
    AzRegionMismatch,
    ResidencyRegionMismatch,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CellRouter {
    bindings: BTreeMap<String, CellBinding>,
}

impl CellRouter {
    pub fn bind(&mut self, input: CellBindingCreate) -> Result<CellBinding, CellError> {
        if self.bindings.contains_key(&input.tenant_id) {
            return Err(CellError::AlreadyBound);
        }
        let binding = CellBinding::new(input)?;
        self.bindings
            .insert(binding.tenant_id.clone(), binding.clone());
        Ok(binding)
    }

    pub fn get(&self, tenant_id: &str) -> Option<&CellBinding> {
        self.bindings.get(tenant_id)
    }
}

impl CellBinding {
    pub fn new(input: CellBindingCreate) -> Result<Self, CellError> {
        validate_tenant_id(&input.tenant_id)?;
        validate_non_empty(&input.az, CellError::EmptyAz)?;
        validate_non_empty(&input.cell_id, CellError::EmptyCell)?;
        validate_non_empty(&input.hsm_partition_ref, CellError::EmptyHsmPartition)?;
        let region_id = input.region.region_id.value.clone();
        validate_az_region(&input.az, &region_id)?;
        if !residency_class_allows_home_region_label(&input.residency_class, &region_id) {
            return Err(CellError::ResidencyRegionMismatch);
        }
        Ok(Self {
            tenant_id: input.tenant_id,
            region: region_id,
            region_ref: internal(input.region),
            residency_class: internal(input.residency_class),
            az: internal(input.az),
            cell_id: internal(input.cell_id),
            tier: internal(input.tier),
            hsm_partition_ref: internal(input.hsm_partition_ref),
            schema_version: internal(CELL_BINDING_SCHEMA_VERSION),
        })
    }
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), CellError> {
    if tenant_id.starts_with("ten_") && tenant_id.len() > 4 {
        Ok(())
    } else {
        Err(CellError::InvalidTenantId)
    }
}

fn validate_az_region(az: &str, region_id: &str) -> Result<(), CellError> {
    if az == region_id
        || az
            .strip_prefix(region_id)
            .is_some_and(|suffix| suffix.starts_with('-'))
    {
        Ok(())
    } else {
        Err(CellError::AzRegionMismatch)
    }
}

fn validate_non_empty(value: &str, error: CellError) -> Result<(), CellError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, internal_data_class())
}

fn internal_data_class() -> PrivacyDataClass {
    // ADR-0083 Tier 1: use the infallible kernel constructor; the previous
    // `.expect()` proved a statically known invariant that the kernel now
    // encodes at the type level.
    PrivacyDataClass::internal_only()
}
