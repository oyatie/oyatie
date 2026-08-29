//! Identity, region, billing, observability, and FinOps product surfaces.

use std::collections::BTreeSet;

use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

use crate::error::CloudSurfaceError;
use crate::ids::CloudSurfaceId;
use crate::validate::{internal, public, public_class, validate_nonempty};
use crate::{MAX_STS_TTL_SECONDS, MIN_AZ_SEPARATION_KM, MIN_DAY_ONE_AZ_COUNT, REGION_CODE_PREFIX};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamSurfaceCreate {
    pub cedar_policy_gated: bool,         // data_class: PUBLIC
    pub saml2_federation: bool,           // data_class: PUBLIC
    pub oidc_federation: bool,            // data_class: PUBLIC
    pub sts_ttl_seconds: u32,             // data_class: PUBLIC
    pub privileged_mfa_required: bool,    // data_class: PUBLIC
    pub audit_chain_on_every_authz: bool, // data_class: PUBLIC
    pub data_class: DataClass,            // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IamSurface {
    pub cedar_policy_gated: Classified<bool>, // data_class: PUBLIC
    pub saml2_federation: Classified<bool>,   // data_class: PUBLIC
    pub oidc_federation: Classified<bool>,    // data_class: PUBLIC
    pub sts_ttl_seconds: Classified<u32>,     // data_class: PUBLIC
    pub privileged_mfa_required: Classified<bool>, // data_class: PUBLIC
    pub audit_chain_on_every_authz: Classified<bool>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionsSurfaceCreate {
    pub day_one_region: String,                    // data_class: PUBLIC
    pub az_count: u8,                              // data_class: PUBLIC
    pub min_az_separation_km: u16,                 // data_class: PUBLIC
    pub cell_isolation_unit: bool,                 // data_class: PUBLIC
    pub regional_pack_admission: bool,             // data_class: PUBLIC
    pub dedicated_cells_for_regulated_packs: bool, // data_class: PUBLIC
    pub data_class: DataClass,                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionsSurface {
    pub day_one_region: Classified<String>, // data_class: PUBLIC
    pub az_count: Classified<u8>,           // data_class: PUBLIC
    pub min_az_separation_km: Classified<u16>, // data_class: PUBLIC
    pub cell_isolation_unit: Classified<bool>, // data_class: PUBLIC
    pub regional_pack_admission: Classified<bool>, // data_class: PUBLIC
    pub dedicated_cells_for_regulated_packs: Classified<bool>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingSurfaceCreate {
    pub per_resource_per_tenant: bool, // data_class: PUBLIC
    pub per_region_tax_invoice: bool,  // data_class: PUBLIC
    pub usage_events: bool,            // data_class: PUBLIC
    pub metered_overage: bool,         // data_class: PUBLIC
    pub reservations: bool,            // data_class: PUBLIC
    pub commitments: bool,             // data_class: PUBLIC
    pub credits: bool,                 // data_class: PUBLIC
    pub data_class: DataClass,         // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingSurface {
    pub per_resource_per_tenant: Classified<bool>, // data_class: PUBLIC
    pub per_region_tax_invoice: Classified<bool>,  // data_class: PUBLIC
    pub usage_events: Classified<bool>,            // data_class: PUBLIC
    pub metered_overage: Classified<bool>,         // data_class: PUBLIC
    pub reservations: Classified<bool>,            // data_class: PUBLIC
    pub commitments: Classified<bool>,             // data_class: PUBLIC
    pub credits: Classified<bool>,                 // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>,  // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservabilitySurfaceCreate {
    pub per_tenant_slo_dashboards: bool,         // data_class: PUBLIC
    pub audit_chain_mirror: bool,                // data_class: PUBLIC
    pub tenant_owned_namespace: bool,            // data_class: PUBLIC
    pub cross_tenant_admin_grant_required: bool, // data_class: PUBLIC
    pub data_class: DataClass,                   // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservabilitySurface {
    pub per_tenant_slo_dashboards: Classified<bool>, // data_class: PUBLIC
    pub audit_chain_mirror: Classified<bool>,        // data_class: PUBLIC
    pub tenant_owned_namespace: Classified<bool>,    // data_class: PUBLIC
    pub cross_tenant_admin_grant_required: Classified<bool>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinOpsSurfaceCreate {
    pub per_axis_cost_attribution: bool, // data_class: PUBLIC
    pub per_cell_unit_economics: bool,   // data_class: PUBLIC
    pub reservation_commitment_recommendations: bool, // data_class: PUBLIC
    pub anomaly_detector: bool,          // data_class: PUBLIC
    pub public_cloud_cost_adapter: bool, // data_class: PUBLIC
    pub dcim_cost_adapter: bool,         // data_class: PUBLIC
    pub data_class: DataClass,           // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinOpsSurface {
    pub per_axis_cost_attribution: Classified<bool>, // data_class: PUBLIC
    pub per_cell_unit_economics: Classified<bool>,   // data_class: PUBLIC
    pub reservation_commitment_recommendations: Classified<bool>, // data_class: PUBLIC
    pub anomaly_detector: Classified<bool>,          // data_class: PUBLIC
    pub public_cloud_cost_adapter: Classified<bool>, // data_class: PUBLIC
    pub dcim_cost_adapter: Classified<bool>,         // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>,    // data_class: PUBLIC
}

impl IamSurface {
    pub fn new(input: IamSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_iam_surface(&input)?;
        Ok(Self {
            cedar_policy_gated: public(input.cedar_policy_gated),
            saml2_federation: public(input.saml2_federation),
            oidc_federation: public(input.oidc_federation),
            sts_ttl_seconds: public(input.sts_ttl_seconds),
            privileged_mfa_required: public(input.privileged_mfa_required),
            audit_chain_on_every_authz: public(input.audit_chain_on_every_authz),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl RegionsSurface {
    pub fn new(input: RegionsSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_regions_surface(&input)?;
        Ok(Self {
            day_one_region: public(input.day_one_region),
            az_count: public(input.az_count),
            min_az_separation_km: public(input.min_az_separation_km),
            cell_isolation_unit: public(input.cell_isolation_unit),
            regional_pack_admission: public(input.regional_pack_admission),
            dedicated_cells_for_regulated_packs: public(input.dedicated_cells_for_regulated_packs),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl BillingSurface {
    pub fn new(input: BillingSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_billing_surface(&input)?;
        Ok(Self {
            per_resource_per_tenant: public(input.per_resource_per_tenant),
            per_region_tax_invoice: public(input.per_region_tax_invoice),
            usage_events: public(input.usage_events),
            metered_overage: public(input.metered_overage),
            reservations: public(input.reservations),
            commitments: public(input.commitments),
            credits: public(input.credits),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl ObservabilitySurface {
    pub fn new(input: ObservabilitySurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_observability_surface(&input)?;
        Ok(Self {
            per_tenant_slo_dashboards: public(input.per_tenant_slo_dashboards),
            audit_chain_mirror: public(input.audit_chain_mirror),
            tenant_owned_namespace: public(input.tenant_owned_namespace),
            cross_tenant_admin_grant_required: public(input.cross_tenant_admin_grant_required),
            data_class: public_class(input.data_class)?,
        })
    }
}

impl FinOpsSurface {
    pub fn new(input: FinOpsSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        validate_finops_surface(&input)?;
        Ok(Self {
            per_axis_cost_attribution: public(input.per_axis_cost_attribution),
            per_cell_unit_economics: public(input.per_cell_unit_economics),
            reservation_commitment_recommendations: public(
                input.reservation_commitment_recommendations,
            ),
            anomaly_detector: public(input.anomaly_detector),
            public_cloud_cost_adapter: public(input.public_cloud_cost_adapter),
            dcim_cost_adapter: public(input.dcim_cost_adapter),
            data_class: public_class(input.data_class)?,
        })
    }
}

fn validate_iam_surface(input: &IamSurfaceCreate) -> Result<(), CloudSurfaceError> {
    if input.cedar_policy_gated
        && input.saml2_federation
        && input.oidc_federation
        && input.sts_ttl_seconds > 0
        && input.sts_ttl_seconds <= MAX_STS_TTL_SECONDS
        && input.privileged_mfa_required
        && input.audit_chain_on_every_authz
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::InvalidIamSurface)
    }
}

fn validate_regions_surface(input: &RegionsSurfaceCreate) -> Result<(), CloudSurfaceError> {
    if input.day_one_region.starts_with(REGION_CODE_PREFIX)
        && input.az_count >= MIN_DAY_ONE_AZ_COUNT
        && input.min_az_separation_km >= MIN_AZ_SEPARATION_KM
        && input.cell_isolation_unit
        && input.regional_pack_admission
        && input.dedicated_cells_for_regulated_packs
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::InvalidRegionsSurface)
    }
}

fn validate_billing_surface(input: &BillingSurfaceCreate) -> Result<(), CloudSurfaceError> {
    if input.per_resource_per_tenant
        && input.per_region_tax_invoice
        && input.usage_events
        && input.metered_overage
        && input.reservations
        && input.commitments
        && input.credits
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::InvalidBillingSurface)
    }
}

fn validate_observability_surface(
    input: &ObservabilitySurfaceCreate,
) -> Result<(), CloudSurfaceError> {
    if input.per_tenant_slo_dashboards
        && input.audit_chain_mirror
        && input.tenant_owned_namespace
        && input.cross_tenant_admin_grant_required
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::InvalidObservabilitySurface)
    }
}

fn validate_finops_surface(input: &FinOpsSurfaceCreate) -> Result<(), CloudSurfaceError> {
    if input.per_axis_cost_attribution
        && input.per_cell_unit_economics
        && input.reservation_commitment_recommendations
        && input.anomaly_detector
        && input.public_cloud_cost_adapter
        && input.dcim_cost_adapter
    {
        Ok(())
    } else {
        Err(CloudSurfaceError::InvalidFinOpsSurface)
    }
}
