fn instance_create_input(
    body: &CloudComputeVmCreateRequest,
) -> Result<InstanceCreate, CloudComputeVmApiError> {
    Ok(InstanceCreate {
        resource_id: body.resource_id.clone(),
        tenant_id: body.tenant_id.clone(),
        region: body.region.clone(),
        az: body.az.clone(),
        cell_id: body.cell_id.clone(),
        flavor: flavor_spec(body.flavor.clone())?,
        image: body.image.clone(),
        key_pair: body.key_pair.clone(),
        vpc_id: body.vpc_id.clone(),
        subnet_id: body.subnet_id.clone(),
        security_groups: security_group_values(body)?,
        iam_role: iam_role_value(body)?,
        user_data_uri: body.user_data_uri.clone(),
        quota: quota_envelope(body.quota),
        residency: parse_api_residency(body.residency.clone())?,
        state: InstanceState::Pending,
        data_class: parse_api_data_class(body.data_class.clone())?,
        created_at_epoch_seconds: body.created_at_epoch_seconds,
    })
}

fn security_group_values(
    body: &CloudComputeVmCreateRequest,
) -> Result<Vec<String>, CloudComputeVmApiError> {
    let mut values = Vec::with_capacity(body.security_groups.len());
    for group in &body.security_groups {
        if group.tenant_id != body.tenant_id
            || group.region != body.region
            || group.vpc_id != body.vpc_id
        {
            return Err(CloudComputeVmApiError::SecurityGroupBindingMismatch {
                security_group: group.value.clone(),
                tenant_id: group.tenant_id.clone(),
                region: group.region.clone(),
                vpc_id: group.vpc_id.clone(),
            });
        }
        values.push(group.value.clone());
    }
    Ok(values)
}

fn iam_role_value(
    body: &CloudComputeVmCreateRequest,
) -> Result<Option<String>, CloudComputeVmApiError> {
    let Some(role) = &body.iam_role else {
        return Ok(None);
    };
    if role.tenant_id != body.tenant_id || role.region != body.region || role.vpc_id != body.vpc_id
    {
        return Err(CloudComputeVmApiError::IamRoleBindingMismatch {
            role_id: role.value.clone(),
            tenant_id: role.tenant_id.clone(),
            region: role.region.clone(),
            vpc_id: role.vpc_id.clone(),
        });
    }
    Ok(Some(role.value.clone()))
}

fn flavor_spec(
    input: CloudComputeVmFlavorSpec,
) -> Result<ComputeFlavorSpec, CloudComputeVmApiError> {
    Ok(ComputeFlavorSpec {
        class: parse_flavor_class(input.class)?,
        vcpu: input.vcpu,
        memory_gb: input.memory_gb,
        gpu_count: input.gpu_count,
        local_ssd_gb: input.local_ssd_gb,
    })
}

fn quota_envelope(input: CloudComputeVmQuotaEnvelope) -> ComputeQuotaEnvelope {
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

fn parse_flavor_class(label: String) -> Result<InstanceFlavor, CloudComputeVmApiError> {
    match label.as_str() {
        "general_purpose" => Ok(InstanceFlavor::GeneralPurpose),
        "compute_optimized" => Ok(InstanceFlavor::ComputeOptimized),
        "memory_optimized" => Ok(InstanceFlavor::MemoryOptimized),
        "gpu" => Ok(InstanceFlavor::Gpu),
        _ => Err(CloudComputeVmApiError::InvalidFlavorClassLabel { class: label }),
    }
}

fn flavor_class_label(class: InstanceFlavor) -> &'static str {
    match class {
        InstanceFlavor::GeneralPurpose => "general_purpose",
        InstanceFlavor::ComputeOptimized => "compute_optimized",
        InstanceFlavor::MemoryOptimized => "memory_optimized",
        InstanceFlavor::Gpu => "gpu",
    }
}

fn parse_api_residency(label: String) -> Result<ResidencyClass, CloudComputeVmApiError> {
    parse_residency_class_label(&label)
        .ok_or(CloudComputeVmApiError::InvalidResidencyLabel { residency: label })
}

fn parse_api_data_class(label: String) -> Result<DataClass, CloudComputeVmApiError> {
    parse_data_class_label(&label)
        .ok_or(CloudComputeVmApiError::InvalidDataClassLabel { data_class: label })
}

fn image_kind_label(kind: ImageRefKind) -> &'static str {
    match kind {
        ImageRefKind::Oci => "oci",
        ImageRefKind::Qcow2 => "qcow2",
        ImageRefKind::FunctionBundle => "function_bundle",
    }
}

fn instance_state_label(state: InstanceState) -> &'static str {
    match state {
        InstanceState::Pending => "pending",
        InstanceState::Running => "running",
        InstanceState::Stopping => "stopping",
        InstanceState::Stopped => "stopped",
        InstanceState::Terminated => "terminated",
    }
}

fn idempotency_key_for(
    boundary: &CloudComputeVmApiBoundaryContext,
    principal: &CloudComputeVmApiPrincipal,
    surface: &str,
) -> CloudComputeVmIdempotencyLedgerKey {
    CloudComputeVmIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn vm_create_fingerprint_for(
    path_instance_id: &str,
    input: &InstanceCreate,
) -> CloudComputeVmRequestFingerprint {
    let mut security_groups = input.security_groups.clone();
    security_groups.sort();
    CloudComputeVmRequestFingerprint {
        canonical: canonical_fields(&[
            ("path.instance_id", path_instance_id.to_string()),
            ("body.resource_id", input.resource_id.clone()),
            ("body.tenant_id", input.tenant_id.clone()),
            ("body.region", input.region.clone()),
            ("body.az", input.az.clone()),
            ("body.cell_id", input.cell_id.clone()),
            (
                "body.flavor.class",
                flavor_class_label(input.flavor.class).to_string(),
            ),
            ("body.flavor.vcpu", input.flavor.vcpu.to_string()),
            ("body.flavor.memory_gb", input.flavor.memory_gb.to_string()),
            ("body.flavor.gpu_count", input.flavor.gpu_count.to_string()),
            (
                "body.flavor.local_ssd_gb",
                input.flavor.local_ssd_gb.to_string(),
            ),
            ("body.image", input.image.clone()),
            ("body.key_pair", input.key_pair.clone().unwrap_or_default()),
            ("body.vpc_id", input.vpc_id.clone()),
            ("body.subnet_id", input.subnet_id.clone()),
            ("body.security_groups", canonical_sequence(&security_groups)),
            ("body.iam_role", input.iam_role.clone().unwrap_or_default()),
            (
                "body.user_data_uri",
                input.user_data_uri.clone().unwrap_or_default(),
            ),
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
