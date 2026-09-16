use super::*;

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
