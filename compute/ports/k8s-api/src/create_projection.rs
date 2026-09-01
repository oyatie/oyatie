fn cluster_create_input(
    body: &CloudComputeK8sClusterCreateRequest,
) -> Result<KubernetesClusterCreate, CloudComputeK8sApiError> {
    Ok(KubernetesClusterCreate {
        resource_id: body.resource_id.clone(),
        tenant_id: body.tenant_id.clone(),
        region: body.region.clone(),
        flavor: parse_k8s_flavor(body.flavor.clone())?,
        control_plane_version: body.control_plane_version.clone(),
        control_plane_private: body.control_plane_private,
        node_pools: node_pool_create_inputs(body)?,
        quota: quota_envelope(body.quota),
        residency: parse_api_residency(body.residency.clone())?,
        state: KubernetesClusterState::Creating,
        data_class: parse_api_data_class(body.data_class.clone())?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn node_pool_create_inputs(
    body: &CloudComputeK8sClusterCreateRequest,
) -> Result<Vec<KubernetesNodePoolCreate>, CloudComputeK8sApiError> {
    body.node_pools
        .iter()
        .map(|pool| {
            Ok(KubernetesNodePoolCreate {
                id: pool.id.clone(),
                az: pool.az.clone(),
                cell_id: pool.cell_id.clone(),
                subnet_id: pool.subnet_id.clone(),
                security_groups: security_group_values(body, pool)?,
                flavor: flavor_spec(pool.flavor.clone())?,
                min_nodes: pool.min_nodes,
                max_nodes: pool.max_nodes,
                autoscaling_enabled: pool.autoscaling_enabled,
            })
        })
        .collect()
}

fn security_group_values(
    body: &CloudComputeK8sClusterCreateRequest,
    pool: &CloudComputeK8sNodePoolCreateRequest,
) -> Result<Vec<String>, CloudComputeK8sApiError> {
    let mut values = Vec::with_capacity(pool.security_groups.len());
    for group in &pool.security_groups {
        if group.tenant_id != body.tenant_id
            || group.region != body.region
            || group.subnet_id != pool.subnet_id
        {
            return Err(
                CloudComputeK8sApiError::NodePoolSecurityGroupBindingMismatch {
                    node_pool_id: pool.id.clone(),
                    security_group: group.value.clone(),
                    tenant_id: group.tenant_id.clone(),
                    region: group.region.clone(),
                    subnet_id: group.subnet_id.clone(),
                },
            );
        }
        values.push(group.value.clone());
    }
    Ok(values)
}

fn flavor_spec(
    input: CloudComputeK8sNodePoolFlavorSpec,
) -> Result<ComputeFlavorSpec, CloudComputeK8sApiError> {
    Ok(ComputeFlavorSpec {
        class: parse_node_pool_flavor_class(input.class)?,
        vcpu: input.vcpu,
        memory_gb: input.memory_gb,
        gpu_count: input.gpu_count,
        local_ssd_gb: input.local_ssd_gb,
    })
}

fn quota_envelope(input: CloudComputeK8sQuotaEnvelope) -> ComputeQuotaEnvelope {
    ComputeQuotaEnvelope {
        vcpu_limit: input.vcpu_limit,
        memory_gb_limit: input.memory_gb_limit,
        gpu_limit: input.gpu_limit,
        local_ssd_gb_limit: input.local_ssd_gb_limit,
        current_vcpu: input.current_vcpu,
        current_memory_gb: input.current_memory_gb,
        current_gpu: input.current_gpu,
        current_local_ssd_gb: input.current_local_ssd_gb,
    }
}

fn parse_k8s_flavor(label: String) -> Result<K8sFlavor, CloudComputeK8sApiError> {
    match label.as_str() {
        "standard" => Ok(K8sFlavor::Standard),
        "high_availability" => Ok(K8sFlavor::HighAvailability),
        _ => Err(CloudComputeK8sApiError::InvalidClusterFlavorLabel { flavor: label }),
    }
}

fn k8s_flavor_label(flavor: K8sFlavor) -> &'static str {
    match flavor {
        K8sFlavor::Standard => "standard",
        K8sFlavor::HighAvailability => "high_availability",
    }
}

fn parse_node_pool_flavor_class(label: String) -> Result<InstanceFlavor, CloudComputeK8sApiError> {
    match label.as_str() {
        "general_purpose" => Ok(InstanceFlavor::GeneralPurpose),
        "compute_optimized" => Ok(InstanceFlavor::ComputeOptimized),
        "memory_optimized" => Ok(InstanceFlavor::MemoryOptimized),
        "gpu" => Ok(InstanceFlavor::Gpu),
        _ => Err(CloudComputeK8sApiError::InvalidNodePoolFlavorClassLabel { class: label }),
    }
}

fn node_pool_flavor_class_label(class: InstanceFlavor) -> &'static str {
    match class {
        InstanceFlavor::GeneralPurpose => "general_purpose",
        InstanceFlavor::ComputeOptimized => "compute_optimized",
        InstanceFlavor::MemoryOptimized => "memory_optimized",
        InstanceFlavor::Gpu => "gpu",
    }
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudComputeK8sApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudComputeK8sApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudComputeK8sApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudComputeK8sApiError::InvalidDataClassLabel { data_class: label })
}

fn idempotency_key_for(
    boundary: &CloudComputeK8sApiBoundaryContext,
    principal: &CloudComputeK8sApiPrincipal,
    surface: &str,
) -> CloudComputeK8sOperationKey {
    CloudComputeK8sOperationKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn cluster_create_fingerprint_for(
    path_cluster_id: &str,
    input: &KubernetesClusterCreate,
) -> CloudComputeK8sRequestFingerprint {
    let mut pools = input.node_pools.clone();
    pools.sort_by(|left, right| left.id.cmp(&right.id));
    CloudComputeK8sRequestFingerprint {
        canonical: canonical_fields(&[
            ("path.cluster_id", path_cluster_id.to_string()),
            ("body.resource_id", input.resource_id.clone()),
            ("body.tenant_id", input.tenant_id.clone()),
            ("body.region", input.region.clone()),
            ("body.flavor", k8s_flavor_label(input.flavor).to_string()),
            (
                "body.control_plane_version",
                input.control_plane_version.clone(),
            ),
            (
                "body.control_plane_private",
                input.control_plane_private.to_string(),
            ),
            ("body.node_pools", canonical_node_pools(&pools)),
            ("body.quota.vcpu_limit", input.quota.vcpu_limit.to_string()),
            (
                "body.quota.memory_gb_limit",
                input.quota.memory_gb_limit.to_string(),
            ),
            ("body.quota.gpu_limit", input.quota.gpu_limit.to_string()),
            (
                "body.quota.local_ssd_gb_limit",
                input.quota.local_ssd_gb_limit.to_string(),
            ),
            (
                "body.quota.current_vcpu",
                input.quota.current_vcpu.to_string(),
            ),
            (
                "body.quota.current_memory_gb",
                input.quota.current_memory_gb.to_string(),
            ),
            (
                "body.quota.current_gpu",
                input.quota.current_gpu.to_string(),
            ),
            (
                "body.quota.current_local_ssd_gb",
                input.quota.current_local_ssd_gb.to_string(),
            ),
            (
                "body.residency",
                input.residency.label().unwrap_or("per_pack").to_string(),
            ),
            ("body.data_class", input.data_class.label().to_string()),
            (
                "body.created_at_epoch_seconds",
                input.created_at_epoch_seconds.to_string(),
            ),
        ]),
    }
}

fn canonical_node_pools(pools: &[KubernetesNodePoolCreate]) -> String {
    pools
        .iter()
        .map(|pool| {
            let mut security_groups = pool.security_groups.clone();
            security_groups.sort();
            canonical_fields(&[
                ("id", pool.id.clone()),
                ("az", pool.az.clone()),
                ("cell_id", pool.cell_id.clone()),
                ("subnet_id", pool.subnet_id.clone()),
                ("security_groups", canonical_sequence(&security_groups)),
                (
                    "flavor.class",
                    node_pool_flavor_class_label(pool.flavor.class).to_string(),
                ),
                ("flavor.vcpu", pool.flavor.vcpu.to_string()),
                ("flavor.memory_gb", pool.flavor.memory_gb.to_string()),
                ("flavor.gpu_count", pool.flavor.gpu_count.to_string()),
                ("flavor.local_ssd_gb", pool.flavor.local_ssd_gb.to_string()),
                ("min_nodes", pool.min_nodes.to_string()),
                ("max_nodes", pool.max_nodes.to_string()),
                ("autoscaling_enabled", pool.autoscaling_enabled.to_string()),
            ])
        })
        .collect::<Vec<_>>()
        .join("")
}

fn canonical_fields(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{}:{}={}:{}", name.len(), name, value.len(), value))
        .collect::<Vec<_>>()
        .join("")
}

fn canonical_sequence(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("{}:{}", value.len(), value))
        .collect::<Vec<_>>()
        .join("")
}
