//! Cloud phase-invariant product-surface kernel.
//!
//! This crate owns the ADR-0028 `oya-cloud-surface-kernel` contract: customers
//! bind to one Cloud product surface while the fulfillment substrate moves from
//! rented public-cloud capacity to Oyatie-operated colo to Oyatie-owned DCs.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use oya_data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

const CLOUD_SURFACE_SCHEMA_VERSION: u32 = 1;
const SURFACE_ID_PREFIX: &str = "csurf_";
const SKU_ID_PREFIX: &str = "csku_";
const PROVIDER_REF_PREFIX: &str = "provider/";
const REGION_CODE_PREFIX: &str = "region-";
const MIN_DAY_ONE_AZ_COUNT: u8 = 3;
const MIN_AZ_SEPARATION_KM: u16 = 30;
const MAX_STS_TTL_SECONDS: u32 = 3_600;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudSurfaceId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CloudSkuId {
    pub value: String, // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProviderRef {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FulfillmentPhase {
    PublicCloudConsumption,
    HybridColo,
    OwnedMegaDc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeSkuKind {
    ManagedKubernetes,
    Functions,
    VirtualMachine,
    BareMetalLease,
    Gpu,
    EdgeCompute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KubeTier {
    Standard,
    HighAvailability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NodeClass {
    GeneralPurpose,
    ComputeOptimized,
    MemoryOptimized,
    Gpu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FunctionRuntime {
    Rust,
    TypeScript,
    Python,
    Wasm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ColdStartClass {
    Interactive,
    Batch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum VmShape {
    GeneralPurpose,
    ComputeOptimized,
    MemoryOptimized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum IsolationLevel {
    SharedCell,
    DedicatedCell,
    SovereignCell,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RackClass {
    GeneralPurpose,
    StorageOptimized,
    GpuDense,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LeaseTerm {
    Monthly,
    OneYear,
    ThreeYear,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AcceleratorClass {
    Inference,
    Training,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum InterconnectClass {
    Pcie,
    Infiniband,
    EthernetRoce,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PopClass {
    Regional,
    Metro,
    SovereignEdge,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ComputeSku {
    ManagedKubernetes {
        tier: KubeTier,
        node_class: NodeClass,
    }, // data_class: PUBLIC
    Functions {
        runtime: FunctionRuntime,
        cold_start_class: ColdStartClass,
    }, // data_class: PUBLIC
    VirtualMachine {
        shape: VmShape,
        isolation: IsolationLevel,
    }, // data_class: PUBLIC
    BareMetalLease {
        rack_class: RackClass,
        term: LeaseTerm,
    }, // data_class: PUBLIC
    Gpu {
        accelerator: AcceleratorClass,
        interconnect: InterconnectClass,
    }, // data_class: PUBLIC
    EdgeCompute {
        pop_class: PopClass,
        latency_budget_ms: u16,
    }, // data_class: PUBLIC
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudSurfaceCreate {
    pub id: String,                                // data_class: PUBLIC
    pub compute: ComputeSurfaceCreate,             // data_class: PUBLIC
    pub storage: StorageSurfaceCreate,             // data_class: PUBLIC
    pub network: NetworkSurfaceCreate,             // data_class: PUBLIC
    pub iam: IamSurfaceCreate,                     // data_class: PUBLIC
    pub regions: RegionsSurfaceCreate,             // data_class: PUBLIC
    pub billing: BillingSurfaceCreate,             // data_class: PUBLIC
    pub observability: ObservabilitySurfaceCreate, // data_class: PUBLIC
    pub finops: FinOpsSurfaceCreate,               // data_class: PUBLIC
    pub data_class: DataClass,                     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudSurface {
    pub id: Classified<CloudSurfaceId>,      // data_class: PUBLIC
    pub compute: Classified<ComputeSurface>, // data_class: PUBLIC
    pub storage: Classified<StorageSurface>, // data_class: PUBLIC
    pub network: Classified<NetworkSurface>, // data_class: PUBLIC
    pub iam: Classified<IamSurface>,         // data_class: PUBLIC
    pub regions: Classified<RegionsSurface>, // data_class: PUBLIC
    pub billing: Classified<BillingSurface>, // data_class: PUBLIC
    pub observability: Classified<ObservabilitySurface>, // data_class: PUBLIC
    pub finops: Classified<FinOpsSurface>,   // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudSurfaceError {
    InvalidSurfaceId,
    InvalidSkuId,
    InvalidProviderRef,
    InvalidDataClass,
    InvalidFulfillment,
    MissingComputeSkuKind,
    DuplicateComputeSku,
    MissingStorageSurface,
    MissingNetworkSurface,
    InvalidIamSurface,
    InvalidRegionsSurface,
    InvalidBillingSurface,
    InvalidObservabilitySurface,
    InvalidFinOpsSurface,
}

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

fn validate_phase_coverage(fulfillments: &[SkuFulfillment]) -> Result<(), CloudSurfaceError> {
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

fn validate_compute_skus(skus: &[ComputeSkuSurface]) -> Result<(), CloudSurfaceError> {
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

fn validate_nonempty(value: &str, error: CloudSurfaceError) -> Result<(), CloudSurfaceError> {
    if value.trim().is_empty() {
        Err(error)
    } else {
        Ok(())
    }
}

fn prefixed_token(
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

fn public_class(data_class: DataClass) -> Result<Classified<PrivacyDataClass>, CloudSurfaceError> {
    let class =
        PrivacyDataClass::new(data_class).map_err(|_| CloudSurfaceError::InvalidDataClass)?;
    if class.data_class() == DataClass::Public {
        Ok(public(class))
    } else {
        Err(CloudSurfaceError::InvalidDataClass)
    }
}

fn public<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::Public)
}

fn internal<T>(value: T) -> Classified<T> {
    Classified::new(value, DataClass::InternalOnly)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fulfillment(phase: FulfillmentPhase, provider: &str) -> SkuFulfillmentCreate {
        SkuFulfillmentCreate {
            phase,
            provider_ref: provider.to_string(),
            capability_summary: "same public SKU contract, phase-specific provider implementation"
                .to_string(),
            data_class: DataClass::Public,
        }
    }

    fn fulfillments() -> Vec<SkuFulfillmentCreate> {
        vec![
            fulfillment(
                FulfillmentPhase::PublicCloudConsumption,
                "provider/public-cloud/region-alpha1",
            ),
            fulfillment(
                FulfillmentPhase::HybridColo,
                "provider/oyatie-colo/region-alpha1",
            ),
            fulfillment(
                FulfillmentPhase::OwnedMegaDc,
                "provider/oyatie-owned-dc/region-beta1",
            ),
        ]
    }

    fn compute_sku(id: &str, sku: ComputeSku) -> ComputeSkuSurfaceCreate {
        ComputeSkuSurfaceCreate {
            id: id.to_string(),
            sku,
            fulfillments: fulfillments(),
            data_class: DataClass::Public,
        }
    }

    fn compute_surface() -> ComputeSurfaceCreate {
        ComputeSurfaceCreate {
            skus: vec![
                compute_sku(
                    "csku_k8s_ha_gp",
                    ComputeSku::ManagedKubernetes {
                        tier: KubeTier::HighAvailability,
                        node_class: NodeClass::GeneralPurpose,
                    },
                ),
                compute_sku(
                    "csku_fn_rust_interactive",
                    ComputeSku::Functions {
                        runtime: FunctionRuntime::Rust,
                        cold_start_class: ColdStartClass::Interactive,
                    },
                ),
                compute_sku(
                    "csku_vm_gp_shared",
                    ComputeSku::VirtualMachine {
                        shape: VmShape::GeneralPurpose,
                        isolation: IsolationLevel::SharedCell,
                    },
                ),
                compute_sku(
                    "csku_bm_gp_3y",
                    ComputeSku::BareMetalLease {
                        rack_class: RackClass::GeneralPurpose,
                        term: LeaseTerm::ThreeYear,
                    },
                ),
                compute_sku(
                    "csku_gpu_training_roce",
                    ComputeSku::Gpu {
                        accelerator: AcceleratorClass::Training,
                        interconnect: InterconnectClass::EthernetRoce,
                    },
                ),
                compute_sku(
                    "csku_edge_regional_25ms",
                    ComputeSku::EdgeCompute {
                        pop_class: PopClass::Regional,
                        latency_budget_ms: 25,
                    },
                ),
            ],
            data_class: DataClass::Public,
        }
    }

    fn storage_surface() -> StorageSurfaceCreate {
        StorageSurfaceCreate {
            surfaces: vec![
                StorageSurfaceKind::Object,
                StorageSurfaceKind::Block,
                StorageSurfaceKind::File,
                StorageSurfaceKind::Archive,
                StorageSurfaceKind::Database,
            ],
            s3_compatible_object_api: true,
            nvme_block_tiers: true,
            nfs41_smb3_file_api: true,
            cold_archive_tier: true,
            per_cell_key_material: true,
            data_class: DataClass::Public,
        }
    }

    fn network_surface() -> NetworkSurfaceCreate {
        NetworkSurfaceCreate {
            surfaces: vec![
                NetworkSurfaceKind::Vpc,
                NetworkSurfaceKind::LoadBalancer,
                NetworkSurfaceKind::Dns,
                NetworkSurfaceKind::Interconnect,
                NetworkSurfaceKind::DdosProtection,
                NetworkSurfaceKind::ServiceMesh,
            ],
            per_tenant_per_cell_vpc: true,
            l4_l7_load_balancing: true,
            mtls_termination: true,
            dnssec: true,
            direct_interconnect_all_phases: true,
            regional_line_rate_scrubbing: true,
            data_class: DataClass::Public,
        }
    }

    fn iam_surface() -> IamSurfaceCreate {
        IamSurfaceCreate {
            cedar_policy_gated: true,
            saml2_federation: true,
            oidc_federation: true,
            sts_ttl_seconds: 3_600,
            privileged_mfa_required: true,
            audit_chain_on_every_authz: true,
            data_class: DataClass::Public,
        }
    }

    fn regions_surface() -> RegionsSurfaceCreate {
        RegionsSurfaceCreate {
            day_one_region: "region-alpha1".to_string(),
            az_count: 3,
            min_az_separation_km: 30,
            cell_isolation_unit: true,
            regional_pack_admission: true,
            dedicated_cells_for_regulated_packs: true,
            data_class: DataClass::Public,
        }
    }

    fn billing_surface() -> BillingSurfaceCreate {
        BillingSurfaceCreate {
            per_resource_per_tenant: true,
            per_region_tax_invoice: true,
            usage_events: true,
            metered_overage: true,
            reservations: true,
            commitments: true,
            credits: true,
            data_class: DataClass::Public,
        }
    }

    fn observability_surface() -> ObservabilitySurfaceCreate {
        ObservabilitySurfaceCreate {
            per_tenant_slo_dashboards: true,
            audit_chain_mirror: true,
            tenant_owned_namespace: true,
            cross_tenant_admin_grant_required: true,
            data_class: DataClass::Public,
        }
    }

    fn finops_surface() -> FinOpsSurfaceCreate {
        FinOpsSurfaceCreate {
            per_axis_cost_attribution: true,
            per_cell_unit_economics: true,
            reservation_commitment_recommendations: true,
            anomaly_detector: true,
            public_cloud_cost_adapter: true,
            dcim_cost_adapter: true,
            data_class: DataClass::Public,
        }
    }

    fn surface_create() -> CloudSurfaceCreate {
        CloudSurfaceCreate {
            id: "csurf_cloud_v1".to_string(),
            compute: compute_surface(),
            storage: storage_surface(),
            network: network_surface(),
            iam: iam_surface(),
            regions: regions_surface(),
            billing: billing_surface(),
            observability: observability_surface(),
            finops: finops_surface(),
            data_class: DataClass::Public,
        }
    }

    #[test]
    fn accepts_phase_invariant_cloud_surface_with_all_adr0028_families() {
        let surface = CloudSurface::new(surface_create()).expect("surface");
        assert_eq!(surface.id.value.value, "csurf_cloud_v1");
        assert_eq!(surface.compute.value.skus.value.len(), 6);
        for sku in &surface.compute.value.skus.value {
            assert_eq!(sku.fulfillments.value.len(), 3);
        }
        assert_eq!(surface.regions.value.az_count.value, 3);
        assert_eq!(surface.schema_version.value, CLOUD_SURFACE_SCHEMA_VERSION);
    }

    #[test]
    fn rejects_compute_sku_without_all_three_fulfillment_phases() {
        let mut surface = surface_create();
        surface.compute.skus[3].fulfillments.pop();
        let error = CloudSurface::new(surface).expect_err("phase-specific compute SKU rejected");
        assert_eq!(error, CloudSurfaceError::InvalidFulfillment);
    }

    #[test]
    fn rejects_missing_compute_family_duplicate_sku_and_non_public_sku_metadata() {
        let mut surface = surface_create();
        surface.compute.skus.pop();
        let missing =
            CloudSurface::new(surface).expect_err("all six compute families are required");
        assert_eq!(missing, CloudSurfaceError::MissingComputeSkuKind);

        let mut surface = surface_create();
        surface.compute.skus[1].id = "csku_k8s_ha_gp".to_string();
        let duplicate = CloudSurface::new(surface).expect_err("duplicate SKU ids are rejected");
        assert_eq!(duplicate, CloudSurfaceError::DuplicateComputeSku);

        let mut surface = surface_create();
        surface.compute.skus[0].data_class = DataClass::InternalOnly;
        let data_class =
            CloudSurface::new(surface).expect_err("public SKU metadata must stay public");
        assert_eq!(data_class, CloudSurfaceError::InvalidDataClass);
    }

    #[test]
    fn rejects_incomplete_storage_network_or_iam_contracts() {
        let mut surface = surface_create();
        surface
            .storage
            .surfaces
            .retain(|kind| *kind != StorageSurfaceKind::Archive);
        let storage = CloudSurface::new(surface).expect_err("canonical storage surfaces required");
        assert_eq!(storage, CloudSurfaceError::MissingStorageSurface);

        let mut surface = surface_create();
        surface.network.dnssec = false;
        let network =
            CloudSurface::new(surface).expect_err("DNSSEC is part of the network surface");
        assert_eq!(network, CloudSurfaceError::MissingNetworkSurface);

        let mut surface = surface_create();
        surface.iam.sts_ttl_seconds = 3_601;
        let iam = CloudSurface::new(surface).expect_err("STS TTL cannot exceed one hour");
        assert_eq!(iam, CloudSurfaceError::InvalidIamSurface);
    }

    #[test]
    fn rejects_region_billing_observability_and_finops_drift() {
        let mut surface = surface_create();
        surface.regions.min_az_separation_km = 29;
        let regions = CloudSurface::new(surface).expect_err("day-one AZ separation is required");
        assert_eq!(regions, CloudSurfaceError::InvalidRegionsSurface);

        let mut surface = surface_create();
        surface.billing.commitments = false;
        let billing =
            CloudSurface::new(surface).expect_err("commitments are part of stable billing");
        assert_eq!(billing, CloudSurfaceError::InvalidBillingSurface);

        let mut surface = surface_create();
        surface.observability.cross_tenant_admin_grant_required = false;
        let observability = CloudSurface::new(surface)
            .expect_err("cross-tenant observability requires explicit grant");
        assert_eq!(
            observability,
            CloudSurfaceError::InvalidObservabilitySurface
        );

        let mut surface = surface_create();
        surface.finops.dcim_cost_adapter = false;
        let finops =
            CloudSurface::new(surface).expect_err("FinOps must survive colo and owned DC phases");
        assert_eq!(finops, CloudSurfaceError::InvalidFinOpsSurface);
    }
}
