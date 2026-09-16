use super::*;
use crate::validate::*;

impl CloudSurfaceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudSurfaceError> {
        prefixed_token(
            value.into(),
            SURFACE_ID_PREFIX,
            CloudSurfaceError::InvalidSurfaceId,
        )
        .map(|value| Self { value })
    }
}

impl CloudSkuId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudSurfaceError> {
        prefixed_token(value.into(), SKU_ID_PREFIX, CloudSurfaceError::InvalidSkuId)
            .map(|value| Self { value })
    }
}

impl ProviderRef {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudSurfaceError> {
        prefixed_token(
            value.into(),
            PROVIDER_REF_PREFIX,
            CloudSurfaceError::InvalidProviderRef,
        )
        .map(|value| Self { value })
    }
}

impl ComputeSku {
    pub const fn kind(&self) -> ComputeSkuKind {
        match self {
            Self::ManagedKubernetes { .. } => ComputeSkuKind::ManagedKubernetes,
            Self::Functions { .. } => ComputeSkuKind::Functions,
            Self::VirtualMachine { .. } => ComputeSkuKind::VirtualMachine,
            Self::BareMetalLease { .. } => ComputeSkuKind::BareMetalLease,
            Self::Gpu { .. } => ComputeSkuKind::Gpu,
            Self::EdgeCompute { .. } => ComputeSkuKind::EdgeCompute,
        }
    }
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

impl CloudSurface {
    pub fn new(input: CloudSurfaceCreate) -> Result<Self, CloudSurfaceError> {
        Ok(Self {
            id: public(CloudSurfaceId::new(input.id)?),
            compute: public(ComputeSurface::new(input.compute)?),
            storage: public(StorageSurface::new(input.storage)?),
            network: public(NetworkSurface::new(input.network)?),
            iam: public(IamSurface::new(input.iam)?),
            regions: public(RegionsSurface::new(input.regions)?),
            billing: public(BillingSurface::new(input.billing)?),
            observability: public(ObservabilitySurface::new(input.observability)?),
            finops: public(FinOpsSurface::new(input.finops)?),
            data_class: public_class(input.data_class)?,
            schema_version: public(CLOUD_SURFACE_SCHEMA_VERSION),
        })
    }
}
