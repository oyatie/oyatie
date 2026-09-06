#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloudComputeK8sClusterCreateIntent {
    pub resource_id: String,
    pub tenant_id: String,
    pub region: String,
    pub flavor: String,
    pub control_plane_version: String,
    pub control_plane_private: bool,
    pub node_pools: Vec<CloudComputeK8sNodePoolIntent>,
    pub residency: String,
    pub data_class: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloudComputeK8sNodePoolIntent {
    pub id: String,
    pub az: String,
    pub cell_id: String,
    pub subnet_id: String,
    pub security_groups: Vec<String>,
    pub flavor: CloudComputeK8sNodePoolFlavorSpec,
    pub min_nodes: u32,
    pub max_nodes: u32,
    pub autoscaling_enabled: bool,
}

pub fn validate_cloud_compute_k8s_create_intent(
    intent: &CloudComputeK8sClusterCreateIntent,
) -> Result<(), CloudComputeK8sApiError> {
    let node_pools = intent
        .node_pools
        .iter()
        .map(|pool| {
            Ok(KubernetesNodePoolCreate {
                id: pool.id.clone(),
                az: pool.az.clone(),
                cell_id: pool.cell_id.clone(),
                subnet_id: pool.subnet_id.clone(),
                security_groups: pool.security_groups.clone(),
                flavor: node_pool_flavor(&pool.flavor)?,
                min_nodes: pool.min_nodes,
                max_nodes: pool.max_nodes,
                autoscaling_enabled: pool.autoscaling_enabled,
            })
        })
        .collect::<Result<Vec<_>, CloudComputeK8sApiError>>()?;
    compute_domain::validate_kubernetes_cluster_intent(compute_domain::KubernetesClusterIntent {
        resource_id: intent.resource_id.clone(),
        tenant_id: intent.tenant_id.clone(),
        region: intent.region.clone(),
        flavor: parse_k8s_flavor(intent.flavor.clone())?,
        control_plane_version: intent.control_plane_version.clone(),
        control_plane_private: intent.control_plane_private,
        node_pools,
        residency: parse_api_residency(intent.residency.clone())?,
        data_class: parse_api_data_class(intent.data_class.clone())?,
    })
    .map_err(CloudComputeK8sApiError::Compute)
}

pub fn cloud_compute_k8s_create_intent_fingerprint(
    intent: &CloudComputeK8sClusterCreateIntent,
) -> Result<String, CloudComputeK8sApiError> {
    validate_cloud_compute_k8s_create_intent(intent)?;
    let mut canonical = intent.clone();
    canonical
        .node_pools
        .sort_by(|left, right| left.id.cmp(&right.id));
    for pool in &mut canonical.node_pools {
        pool.security_groups.sort();
    }
    serde_json::to_string(&("pending_intent", canonical))
        .map_err(|_| CloudComputeK8sApiError::LifecycleRepositoryInvariantViolation)
}

fn node_pool_flavor(
    input: &CloudComputeK8sNodePoolFlavorSpec,
) -> Result<ComputeFlavorSpec, CloudComputeK8sApiError> {
    Ok(ComputeFlavorSpec {
        class: parse_node_pool_flavor_class(input.class.clone())?,
        vcpu: input.vcpu,
        memory_gb: input.memory_gb,
        gpu_count: input.gpu_count,
        local_ssd_gb: input.local_ssd_gb,
    })
}
