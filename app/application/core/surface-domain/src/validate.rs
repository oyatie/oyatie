use super::*;
use std::collections::BTreeSet;

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

pub(crate) fn validate_storage_surface(
    input: &StorageSurfaceCreate,
) -> Result<(), CloudSurfaceError> {
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

pub(crate) fn validate_network_surface(
    input: &NetworkSurfaceCreate,
) -> Result<(), CloudSurfaceError> {
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

pub(crate) fn validate_iam_surface(input: &IamSurfaceCreate) -> Result<(), CloudSurfaceError> {
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

pub(crate) fn validate_regions_surface(
    input: &RegionsSurfaceCreate,
) -> Result<(), CloudSurfaceError> {
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

pub(crate) fn validate_billing_surface(
    input: &BillingSurfaceCreate,
) -> Result<(), CloudSurfaceError> {
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

pub(crate) fn validate_observability_surface(
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

pub(crate) fn validate_finops_surface(
    input: &FinOpsSurfaceCreate,
) -> Result<(), CloudSurfaceError> {
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

pub(crate) fn validate_nonempty(
    value: &str,
    error: CloudSurfaceError,
) -> Result<(), CloudSurfaceError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

pub(crate) fn prefixed_token(
    value: String,
    prefix: &str,
    error: CloudSurfaceError,
) -> Result<String, CloudSurfaceError> {
    if value.starts_with(prefix)
        && value.len() > prefix.len()
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
    {
        Ok(value)
    } else {
        Err(error)
    }
}

pub(crate) fn public_class(
    data_class: DataClass,
) -> Result<Classified<PrivacyDataClass>, CloudSurfaceError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudSurfaceError::InvalidDataClass)?;
    if class.data_class() == DataClass::Public {
        Ok(public(class))
    } else {
        Err(CloudSurfaceError::InvalidDataClass)
    }
}

pub(crate) fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

pub(crate) fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}
