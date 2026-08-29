//! SKU fulfillment records, the compute-SKU surface binding, and the
//! phase-coverage rules they must satisfy.

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::error::CloudSurfaceError;
use crate::ids::{CloudSkuId, ProviderRef};
use crate::sku::{ComputeSku, ComputeSkuKind, FulfillmentPhase};
use crate::validate::{internal, prefixed_token, public, public_class, validate_nonempty};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkuFulfillmentCreate {
    pub phase: FulfillmentPhase,    // data_class: PUBLIC
    pub provider_ref: String,       // data_class: INTERNAL_ONLY
    pub capability_summary: String, // data_class: PUBLIC
    pub data_class: DataClass,      // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkuFulfillment {
    pub phase: Classified<FulfillmentPhase>, // data_class: PUBLIC
    pub provider_ref: Classified<ProviderRef>, // data_class: INTERNAL_ONLY
    pub capability_summary: Classified<String>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeSkuSurfaceCreate {
    pub id: String,                              // data_class: PUBLIC
    pub sku: ComputeSku,                         // data_class: PUBLIC
    pub fulfillments: Vec<SkuFulfillmentCreate>, // data_class: INTERNAL_ONLY
    pub data_class: DataClass,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputeSkuSurface {
    pub id: Classified<CloudSkuId>,  // data_class: PUBLIC
    pub sku: Classified<ComputeSku>, // data_class: PUBLIC
    pub fulfillments: Classified<Vec<SkuFulfillment>>, // data_class: INTERNAL_ONLY
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
}

impl SkuFulfillment {
    pub fn new(input: SkuFulfillmentCreate) -> Result<Self, CloudSurfaceError> {
        validate_nonempty(
            &input.capability_summary,
            CloudSurfaceError::InvalidFulfillment,
        )?;
        Ok(Self {
            phase: public(input.phase),
            provider_ref: internal(ProviderRef::new(input.provider_ref)?),
            capability_summary: public(input.capability_summary),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl ComputeSkuSurface {
    pub fn new(input: ComputeSkuSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        let fulfillments = input
            .fulfillments
            .into_iter()
            .map(SkuFulfillment::new)
            .collect::<Result<Vec<_>, _>>()?;
        validate_phase_coverage(&fulfillments)?;
        Ok(Self {
            id: public(CloudSkuId::new(input.id)?),
            sku: public(input.sku),
            fulfillments: internal(fulfillments),
            data_class: public_class(input.data_class)?,
        })
    }
}

pub(crate) fn validate_phase_coverage(
    fulfillments: &[SkuFulfillment],
) -> Result<(), CloudSurfaceError> {
    let mut phases = BTreeSet::new();
    for fulfillment in fulfillments {
        if !phases.insert(fulfillment.phase.value) {
            return Err(CloudSurfaceError::InvalidFulfillment);
        }
    }
    let required = BTreeSet::from([
        FulfillmentPhase::PublicCloudConsumption,
        FulfillmentPhase::HybridColo,
        FulfillmentPhase::OwnedMegaDc,
    ]);
    if phases == required {
        Ok(())
    } else {
        Err(CloudSurfaceError::InvalidFulfillment)
    }
}

pub(crate) fn validate_compute_skus(skus: &[ComputeSkuSurface]) -> Result<(), CloudSurfaceError> {
    let mut ids = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for sku in skus {
        if !ids.insert(sku.id.value.clone()) {
            return Err(CloudSurfaceError::DuplicateComputeSku);
        }
        kinds.insert(sku.sku.value.kind());
    }
    let required = BTreeSet::from([
        ComputeSkuKind::ManagedKubernetes,
        ComputeSkuKind::Functions,
        ComputeSkuKind::VirtualMachine,
        ComputeSkuKind::BareMetalLease,
        ComputeSkuKind::Gpu,
        ComputeSkuKind::EdgeCompute,
    ]);
    if kinds == required {
        Ok(())
    } else {
        Err(CloudSurfaceError::MissingComputeSkuKind)
    }
}
