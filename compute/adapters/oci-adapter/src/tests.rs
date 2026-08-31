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
        resource_id: "oyatie:cloud:ap-chuncheon-1:ten_alpha:instance:app-1".to_string(),
        tenant_id: "ten_alpha".to_string(),
        region: "ap-chuncheon-1".to_string(),
        az: "ap-chuncheon-1-a".to_string(),
        cell_id: "cell-ap-chuncheon-1-a-001".to_string(),
        flavor: ComputeFlavorSpec {
            class: InstanceFlavor::GeneralPurpose,
            vcpu: 4,
            memory_gb: 16,
            gpu_count: 0,
            local_ssd_gb: 100,
        },
        image: format!("oci://harbor.ap-chuncheon-1.oyatie.io/ten_alpha/app@sha256:{DIGEST}"),
        key_pair: Some("key_prod".to_string()),
        vpc_id: "oyatie:cloud:ap-chuncheon-1:ten_alpha:vpc:prod".to_string(),
        subnet_id: "oyatie:cloud:ap-chuncheon-1:ten_alpha:subnet:prod-a".to_string(),
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

fn adapter() -> OciComputeAdapter {
    OciComputeAdapter::new(
        "https://iaas.ap-chuncheon-1.oci.example",
        "ocid1.compartment.oc1..alpha",
        "ZkLG:AP-CHUNCHEON-1-AD-1",
        "ap-chuncheon-1",
    )
    .unwrap()
}

fn request(adapter: &OciComputeAdapter) -> ComputeProviderVmCreateRequest {
    let instance = instance();
    ComputeProviderVmCreateRequest {
        request_id: "compute-vm-oci-001".to_string(),
        provider_instance_ref: adapter.provider_instance_ref(&instance.resource_id.value.value),
        tenant_id: instance.tenant_id.value.clone(),
        actor: "sp_cloud_provisioner".to_string(),
        idempotency_key: "idem-compute-vm-oci-001".to_string(),
        requested_at_epoch_seconds: 1_700_200_020,
        instance,
    }
}

#[test]
fn oci_compute_adapter_projects_launch_instance_without_live_sdk_or_credentials() {
    let adapter = adapter();
    let request = request(&adapter);
    let command = adapter
        .launch_instance_command(&request)
        .expect("command projection succeeds");

    assert_eq!(adapter.provider_kind(), ComputeProviderKind::OciCompute);
    assert_eq!(command.operation, "LaunchInstance");
    assert_eq!(command.method, "POST");
    assert_eq!(command.path, "/20160918/instances");
    assert!(
        command
            .body_canonical
            .contains("compartment_ref=ocid1.compartment.oc1..alpha")
    );
    assert!(
        command
            .body_canonical
            .contains("availability_domain=ZkLG:AP-CHUNCHEON-1-AD-1")
    );
    assert!(
        command
            .body_canonical
            .contains("flavor_class=general_purpose")
    );
    assert!(
        command
            .provider_evidence_ref
            .starts_with("oci-compute://ocid1.compartment.oc1..alpha/ZkLG:AP-CHUNCHEON-1-AD-1/")
    );
    assert!(!command.body_canonical.contains("OCI_PRIVATE_KEY"));
    assert!(!command.body_canonical.contains("tenancy_ocid"));
}

#[test]
fn oci_compute_adapter_preview_create_vm_does_not_report_provisioning_success() {
    let adapter = adapter();
    let request = request(&adapter);
    let error = adapter
        .create_vm(request)
        .expect_err("preview adapter must not emit production provisioning receipt");

    assert_eq!(
            error,
            ComputeProviderVmError::ProviderRejected {
                provider: ComputeProviderKind::OciCompute,
                reason: "OCI Compute adapter is command-projection preview only; create_vm does not perform production provisioning"
                    .to_string(),
            }
        );
}

#[test]
fn oci_compute_adapter_rejects_mismatched_provider_ref_or_region() {
    let adapter = adapter();
    let mut request = request(&adapter);
    request.provider_instance_ref = "oci-compute://other/ad/i-wrong".to_string();

    let error = adapter
        .launch_instance_command(&request)
        .expect_err("mismatched provider ref rejected");

    assert!(matches!(
        error,
        ComputeProviderVmError::ProviderRejected {
            provider: ComputeProviderKind::OciCompute,
            ..
        }
    ));
}
