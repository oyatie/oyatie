use compute_resource::{FunctionRuntime, InstanceFlavor, K8sFlavor};
use data_boundary_kernel::DataClass;
use network_residency::{
    PerPackResidency, PerPackResidencyCreate, RegulatorOverlay, RegulatorOverlayCreate,
    ResidencyClass,
};

use crate::*;

pub(super) const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub(super) fn residency_class() -> ResidencyClass {
    ResidencyClass::PerPack(Box::new(
        PerPackResidency::new(PerPackResidencyCreate {
            allowed_primary_regions: vec!["region-alpha".to_string()],
            allowed_replica_regions: vec!["region-beta".to_string()],
            forbidden_regions: vec!["region-gamma".to_string()],
            regulator_overlay: RegulatorOverlay::new(RegulatorOverlayCreate {
                regulator_refs: vec!["regulator/global-cloud".to_string()],
                evidence_ref: "evidence/residency/global-cloud".to_string(),
            })
            .expect("regulator overlay fixture is valid"),
        })
        .expect("per-pack residency fixture is valid"),
    ))
}

pub(super) fn quota() -> ComputeQuotaEnvelope {
    ComputeQuotaEnvelope {
        vcpu_limit: 128,
        memory_gb_limit: 512,
        gpu_limit: 8,
        local_ssd_gb_limit: 4_096,
        current_vcpu: 4,
        current_memory_gb: 16,
        current_gpu: 0,
        current_local_ssd_gb: 100,
    }
}

pub(super) fn flavor() -> ComputeFlavorSpec {
    ComputeFlavorSpec {
        class: InstanceFlavor::GeneralPurpose,
        vcpu: 4,
        memory_gb: 16,
        gpu_count: 0,
        local_ssd_gb: 100,
    }
}

pub(super) fn image() -> String {
    format!("oci://harbor.region-alpha.oyatie.io/ten_alpha/app@sha256:{DIGEST}")
}

pub(super) fn function_bundle() -> String {
    format!("function://harbor.region-alpha.oyatie.io/ten_alpha/image-resize@sha256:{DIGEST}")
}

pub(super) fn instance_create() -> InstanceCreate {
    InstanceCreate {
        resource_id: "oyatie:cloud:region-alpha:ten_alpha:instance:app-1".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        az: "region-alpha-a".to_string(),
        cell_id: "cell-region-alpha-a-001".to_string(),
        flavor: flavor(),
        image: image(),
        key_pair: Some("key_prod".to_string()),
        vpc_id: "oyatie:cloud:region-alpha:ten_alpha:vpc:prod".to_string(),
        subnet_id: "oyatie:cloud:region-alpha:ten_alpha:subnet:prod-a".to_string(),
        security_groups: vec!["sg_web".to_string()],
        iam_role: Some("role_app".to_string()),
        user_data_uri: Some("userdata/ten_alpha/app-1/cloud-init.yaml".to_string()),
        quota: quota(),
        residency: residency_class(),
        state: InstanceState::Pending,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_100_000,
    }
}

pub(super) fn node_pool(id: &str, az: &str, subnet: &str) -> KubernetesNodePoolCreate {
    KubernetesNodePoolCreate {
        id: id.to_string(),
        az: az.to_string(),
        cell_id: format!("cell-{az}-001"),
        subnet_id: subnet.to_string(),
        security_groups: vec!["sg_web".to_string()],
        flavor: flavor(),
        min_nodes: 1,
        max_nodes: 5,
        autoscaling_enabled: true,
    }
}

pub(super) fn k8s_create() -> KubernetesClusterCreate {
    KubernetesClusterCreate {
        resource_id: "oyatie:cloud:region-alpha:ten_alpha:k8s:prod".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        flavor: K8sFlavor::HighAvailability,
        control_plane_version: "v1.30.2-oyatie.1".to_string(),
        control_plane_private: true,
        node_pools: vec![
            node_pool(
                "np_a",
                "region-alpha-a",
                "oyatie:cloud:region-alpha:ten_alpha:subnet:prod-a",
            ),
            node_pool(
                "np_b",
                "region-alpha-b",
                "oyatie:cloud:region-alpha:ten_alpha:subnet:prod-b",
            ),
            node_pool(
                "np_c",
                "region-alpha-c",
                "oyatie:cloud:region-alpha:ten_alpha:subnet:prod-c",
            ),
        ],
        quota: quota(),
        residency: residency_class(),
        state: KubernetesClusterState::Creating,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_100_010,
    }
}

pub(super) fn function_create() -> FunctionDeploymentCreate {
    FunctionDeploymentCreate {
        resource_id: "oyatie:cloud:region-alpha:ten_alpha:function:image-resize".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "region-alpha".to_string(),
        az: "region-alpha-a".to_string(),
        cell_id: "cell-region-alpha-a-001".to_string(),
        runtime: FunctionRuntime::Wasm,
        name: "image-resize".to_string(),
        bundle: function_bundle(),
        cold_start_budget_ms: 750,
        timeout_ms: 30_000,
        memory_mb: 512,
        max_concurrency: 250,
        allowed_data_classes: vec![DataClass::Public, DataClass::PiiIdentifying],
        residency: residency_class(),
        state: FunctionDeploymentState::Deploying,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_100_020,
    }
}

pub(super) fn invocation(id: &str, data_class: DataClass) -> FunctionInvocationRequest {
    FunctionInvocationRequest {
        invocation_id: id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        function_id: "oyatie:cloud:region-alpha:ten_alpha:function:image-resize".to_string(),
        region: "region-alpha".to_string(),
        payload_data_class: data_class,
        idempotency_key: format!("idem-{id}-0123456789"),
        current_concurrent_invocations: 0,
        requested_at_epoch_seconds: 1_700_100_030,
    }
}

pub(super) fn provider_vm_request() -> ComputeProviderVmCreateRequest {
    let instance = Instance::new(instance_create()).expect("instance contract is valid");
    ComputeProviderVmCreateRequest {
        request_id: "compute-vm-provider-001".to_string(),
        provider_instance_ref: "provider://cell-region-alpha-a-001/app-1".to_string(),
        tenant_id: instance.tenant_id.value.clone(),
        actor: "sp_cloud_provisioner".to_string(),
        idempotency_key: "idem-compute-vm-provider-001".to_string(),
        requested_at_epoch_seconds: 1_700_100_050,
        instance,
    }
}
