use std::collections::BTreeSet;

use cell_region::{AzCode, CellId, RegionCode};
use compute_resource::{K8sFlavor, ResourceId, ResourceKind};
use data_boundary_kernel::{Classified, DataClass, PrivacyDataClass};
use network_domain::SecurityGroupId;
use network_residency::ResidencyClass;

use crate::capacity::ComputeUnits;
use crate::{
    COMPUTE_SCHEMA_VERSION, CloudComputeError, ComputeFlavorSpec, ComputeQuotaEnvelope,
    ControlPlaneVersion, NodePoolId, internal, public, public_metadata_class, region_for,
    resource_id_for, resource_ref_for, security_groups, validate_az_region, validate_cell_az,
    validate_tenant_id,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum KubernetesClusterState {
    Creating,
    Ready,
    Reconciling,
    Draining,
    Deleted,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesNodePoolCreate {
    pub id: String,                   // data_class: INTERNAL_ONLY
    pub az: String,                   // data_class: PUBLIC
    pub cell_id: String,              // data_class: PUBLIC
    pub subnet_id: String,            // data_class: INTERNAL_ONLY
    pub security_groups: Vec<String>, // data_class: INTERNAL_ONLY
    pub flavor: ComputeFlavorSpec,    // data_class: PUBLIC
    pub min_nodes: u32,               // data_class: PUBLIC
    pub max_nodes: u32,               // data_class: PUBLIC
    pub autoscaling_enabled: bool,    // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesNodePool {
    pub id: NodePoolId,                        // data_class: INTERNAL_ONLY
    pub az: AzCode,                            // data_class: PUBLIC
    pub cell_id: CellId,                       // data_class: PUBLIC
    pub subnet_id: ResourceId,                 // data_class: INTERNAL_ONLY
    pub security_groups: Vec<SecurityGroupId>, // data_class: INTERNAL_ONLY
    pub flavor: ComputeFlavorSpec,             // data_class: PUBLIC
    pub min_nodes: u32,                        // data_class: PUBLIC
    pub max_nodes: u32,                        // data_class: PUBLIC
    pub autoscaling_enabled: bool,             // data_class: PUBLIC
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesClusterCreate {
    pub resource_id: String,                       // data_class: INTERNAL_ONLY
    pub tenant_id: String,                         // data_class: INTERNAL_ONLY
    pub region: String,                            // data_class: PUBLIC
    pub flavor: K8sFlavor,                         // data_class: PUBLIC
    pub control_plane_version: String,             // data_class: PUBLIC
    pub control_plane_private: bool,               // data_class: PUBLIC
    pub node_pools: Vec<KubernetesNodePoolCreate>, // data_class: INTERNAL_ONLY
    pub quota: ComputeQuotaEnvelope,               // data_class: INTERNAL_ONLY
    pub residency: ResidencyClass,                 // data_class: INTERNAL_ONLY
    pub state: KubernetesClusterState,             // data_class: PUBLIC
    pub data_class: DataClass,                     // data_class: PUBLIC
    pub created_at_epoch_seconds: u64,             // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KubernetesCluster {
    pub resource_id: Classified<ResourceId>, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,       // data_class: INTERNAL_ONLY
    pub region: Classified<RegionCode>,      // data_class: PUBLIC
    pub flavor: Classified<K8sFlavor>,       // data_class: PUBLIC
    pub control_plane_version: Classified<ControlPlaneVersion>, // data_class: PUBLIC
    pub control_plane_private: Classified<bool>, // data_class: PUBLIC
    pub node_pools: Classified<Vec<KubernetesNodePool>>, // data_class: INTERNAL_ONLY
    pub residency: Classified<ResidencyClass>, // data_class: INTERNAL_ONLY
    pub state: Classified<KubernetesClusterState>, // data_class: PUBLIC
    pub data_class: Classified<PrivacyDataClass>, // data_class: PUBLIC
    pub created_at_epoch_seconds: Classified<u64>, // data_class: INTERNAL_ONLY
    pub schema_version: Classified<u32>,     // data_class: PUBLIC
}
impl KubernetesCluster {
    pub fn new(input: KubernetesClusterCreate) -> Result<Self, CloudComputeError> {
        validate_tenant_id(&input.tenant_id)?;
        if input.state != KubernetesClusterState::Creating {
            return Err(CloudComputeError::InvalidKubernetesState);
        }
        let region = region_for(&input.region, &input.residency)?;
        let resource_id = resource_id_for(
            &input.resource_id,
            &input.tenant_id,
            &region,
            ResourceKind::KubernetesCluster(input.flavor),
        )?;
        let control_plane_version = ControlPlaneVersion::new(input.control_plane_version)?;
        let node_pools = node_pools(&input.tenant_id, &region, input.node_pools)?;
        validate_kubernetes_shape(input.flavor, &node_pools)?;
        let requested = node_pools
            .iter()
            .try_fold(ComputeUnits::default(), |sum, pool| {
                let admitted_nodes = if pool.autoscaling_enabled {
                    pool.max_nodes
                } else {
                    pool.min_nodes
                };
                sum.checked_add(pool.flavor.units().checked_mul(admitted_nodes)?)
            })?;
        input.quota.admit(requested)?;
        Ok(Self {
            resource_id: internal(resource_id),
            tenant_id: internal(input.tenant_id),
            region: public(region),
            flavor: public(input.flavor),
            control_plane_version: public(control_plane_version),
            control_plane_private: public(input.control_plane_private),
            node_pools: internal(node_pools),
            residency: internal(input.residency),
            state: public(input.state),
            data_class: public(public_metadata_class(input.data_class)?),
            created_at_epoch_seconds: internal(input.created_at_epoch_seconds),
            schema_version: public(COMPUTE_SCHEMA_VERSION),
        })
    }
}
fn node_pools(
    tenant_id: &str,
    region: &RegionCode,
    input: Vec<KubernetesNodePoolCreate>,
) -> Result<Vec<KubernetesNodePool>, CloudComputeError> {
    if input.is_empty() {
        return Err(CloudComputeError::InvalidNodePoolShape);
    }
    let mut seen = BTreeSet::new();
    let mut pools = Vec::with_capacity(input.len());
    for pool in input {
        let id = NodePoolId::new(pool.id)?;
        if !seen.insert(id.clone()) {
            return Err(CloudComputeError::DuplicateNodePool);
        }
        let az = AzCode::new(pool.az).map_err(|_| CloudComputeError::InvalidAzCode)?;
        validate_az_region(&az, region)?;
        let cell_id = CellId::new(pool.cell_id).map_err(|_| CloudComputeError::InvalidCellId)?;
        validate_cell_az(&cell_id, &az)?;
        let subnet_id = resource_ref_for(&pool.subnet_id, tenant_id, region, "subnet")?;
        let security_groups = security_groups(pool.security_groups)?;
        let flavor = pool.flavor.validate()?;
        if pool.min_nodes == 0 || pool.max_nodes < pool.min_nodes || pool.max_nodes > 1_000 {
            return Err(CloudComputeError::InvalidNodePoolShape);
        }
        pools.push(KubernetesNodePool {
            id,
            az,
            cell_id,
            subnet_id,
            security_groups,
            flavor,
            min_nodes: pool.min_nodes,
            max_nodes: pool.max_nodes,
            autoscaling_enabled: pool.autoscaling_enabled,
        });
    }
    Ok(pools)
}

fn validate_kubernetes_shape(
    flavor: K8sFlavor,
    node_pools: &[KubernetesNodePool],
) -> Result<(), CloudComputeError> {
    if matches!(flavor, K8sFlavor::HighAvailability) {
        let az_count = node_pools
            .iter()
            .map(|pool| pool.az.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let min_nodes: u32 = node_pools.iter().map(|pool| pool.min_nodes).sum();
        if az_count < 3 || min_nodes < 3 {
            return Err(CloudComputeError::KubernetesHaRequiresThreeAzs);
        }
    }
    Ok(())
}
