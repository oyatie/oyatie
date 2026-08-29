// ADR-0083 Tier 3: integration tests legitimately use `.unwrap()` / `.expect()` / `panic!()`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use application_surface::*;
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};

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
    let missing = CloudSurface::new(surface).expect_err("all six compute families are required");
    assert_eq!(missing, CloudSurfaceError::MissingComputeSkuKind);

    let mut surface = surface_create();
    surface.compute.skus[1].id = "csku_k8s_ha_gp".to_string();
    let duplicate = CloudSurface::new(surface).expect_err("duplicate SKU ids are rejected");
    assert_eq!(duplicate, CloudSurfaceError::DuplicateComputeSku);

    let mut surface = surface_create();
    surface.compute.skus[0].data_class = DataClass::InternalOnly;
    let data_class = CloudSurface::new(surface).expect_err("public SKU metadata must stay public");
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
    let network = CloudSurface::new(surface).expect_err("DNSSEC is part of the network surface");
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
    let billing = CloudSurface::new(surface).expect_err("commitments are part of stable billing");
    assert_eq!(billing, CloudSurfaceError::InvalidBillingSurface);

    let mut surface = surface_create();
    surface.observability.cross_tenant_admin_grant_required = false;
    let observability =
        CloudSurface::new(surface).expect_err("cross-tenant observability requires explicit grant");
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
