use super::*;
use compute_domain::{
    ComputeFlavorSpec, ComputeProviderVmCreateRequest, ComputeProviderVmPort, ComputeQuotaEnvelope,
    InstanceCreate, InstanceState,
};
use compute_resource::InstanceFlavor;
use data_boundary_kernel::DataClass;
use network_residency::ResidencyClass;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn instance() -> Instance {
    Instance::new(InstanceCreate {
        resource_id: "oyatie:cloud:us-east-1:ten_alpha:instance:app-1".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "us-east-1".to_string(),
        az: "us-east-1-a".to_string(),
        cell_id: "cell-us-east-1-a-001".to_string(),
        flavor: ComputeFlavorSpec {
            class: InstanceFlavor::GeneralPurpose,
            vcpu: 4,
            memory_gb: 16,
            gpu_count: 0,
            local_ssd_gb: 100,
        },
        image: format!("oci://harbor.us-east-1.oyatie.io/ten_alpha/app@sha256:{DIGEST}"),
        key_pair: Some("key_prod".to_string()),
        vpc_id: "oyatie:cloud:us-east-1:ten_alpha:vpc:prod".to_string(),
        subnet_id: "oyatie:cloud:us-east-1:ten_alpha:subnet:prod-a".to_string(),
        security_groups: vec!["sg_web".to_string(), "sg_app".to_string()],
        iam_role: Some("role_app".to_string()),
        user_data_uri: Some("userdata/ten_alpha/app-1/cloud-init.yaml".to_string()),
        quota: ComputeQuotaEnvelope {
            vcpu_limit: 64,
            memory_gb_limit: 256,
            gpu_limit: 0,
            local_ssd_gb_limit: 1024,
            current_vcpu: 0,
            current_memory_gb: 0,
            current_gpu: 0,
            current_local_ssd_gb: 0,
        },
        residency: ResidencyClass::Global,
        state: InstanceState::Pending,
        data_class: DataClass::Public,
        created_at_epoch_seconds: 1_700_200_000,
    })
    .unwrap()
}

fn adapter() -> AwsComputeAdapter {
    AwsComputeAdapter::new(
        "https://ec2.us-east-1.amazonaws.example",
        "123456789012",
        "us-east-1",
    )
    .unwrap()
}

fn request(adapter: &AwsComputeAdapter) -> ComputeProviderVmCreateRequest {
    let instance = instance();
    ComputeProviderVmCreateRequest {
        request_id: "compute-vm-aws-001".to_string(),
        provider_instance_ref: adapter.provider_instance_ref(&instance.resource_id.value.value),
        tenant_id: instance.tenant_id.value.clone(),
        actor: "sp_cloud_provisioner".to_string(),
        idempotency_key: "idem-compute-vm-aws-001".to_string(),
        requested_at_epoch_seconds: 1_700_200_020,
        instance,
    }
}

#[test]
fn aws_compute_adapter_projects_run_instances_without_live_sdk_or_credentials() {
    let adapter = adapter();
    let request = request(&adapter);
    let command = adapter
        .create_instance_command(&request)
        .expect("command projection succeeds");

    assert_eq!(adapter.provider_kind(), ComputeProviderKind::AwsEc2);
    assert_eq!(command.operation, "RunInstances");
    assert_eq!(command.method, "POST");
    assert_eq!(command.path, "/");
    assert!(command.body_canonical.contains("account_ref=123456789012"));
    assert!(
        command
            .body_canonical
            .contains("flavor_class=general_purpose")
    );
    assert!(
        command
            .body_canonical
            .contains("security_groups=sg_web,sg_app")
    );
    assert!(
        command
            .provider_evidence_ref
            .starts_with("aws-ec2://123456789012/us-east-1/")
    );
    assert!(!command.body_canonical.contains("AWS_SECRET_ACCESS_KEY"));
    assert!(!command.body_canonical.contains("aws_access_key_id"));
}

#[test]
fn aws_compute_adapter_preview_create_vm_does_not_report_provisioning_success() {
    let adapter = adapter();
    let request = request(&adapter);
    let error = adapter
        .create_vm(request)
        .expect_err("preview adapter must not emit production provisioning receipt");

    assert_eq!(
            error,
            ComputeProviderVmError::ProviderRejected {
                provider: ComputeProviderKind::AwsEc2,
                reason: "AWS EC2 adapter is command-projection preview only; create_vm does not perform production provisioning"
                    .to_string(),
            }
        );
}

#[test]
fn aws_compute_adapter_rejects_mismatched_provider_ref_or_region() {
    let adapter = adapter();
    let mut request = request(&adapter);
    request.provider_instance_ref = "aws-ec2://other/us-east-1/i-wrong".to_string();

    let error = adapter
        .create_instance_command(&request)
        .expect_err("mismatched provider ref rejected");

    assert!(matches!(
        error,
        ComputeProviderVmError::ProviderRejected {
            provider: ComputeProviderKind::AwsEc2,
            ..
        }
    ));
}
