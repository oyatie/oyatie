use crate::*;

use super::fixtures::*;

#[test]
fn creates_vm_instance_with_cell_network_iam_quota_and_digest_image() {
    let instance = Instance::new(instance_create()).expect("instance contract is valid");

    assert_eq!(instance.resource_id.value.kind_label().unwrap(), "instance");
    assert_eq!(instance.az.value.value, "region-alpha-a");
    assert_eq!(instance.cell_id.value.value, "cell-region-alpha-a-001");
    assert_eq!(instance.flavor.value.vcpu, 4);
    assert_eq!(instance.image.value.kind, ImageRefKind::Oci);
    assert_eq!(instance.security_groups.value.len(), 1);
    assert!(instance.iam_role.value.is_some());
    assert_eq!(instance.schema_version.value, COMPUTE_SCHEMA_VERSION);
}

#[test]
fn provider_vm_receipt_requires_non_empty_provider_evidence() {
    let request = provider_vm_request();

    let receipt = ComputeProviderVmReceipt::from_request(
        ComputeProviderKind::AwsEc2,
        request.clone(),
        "aws-req-001",
        "aws-ec2://evidence/req-001",
    )
    .expect("provider receipt keeps neutral VM identity");
    assert_eq!(receipt.provider_kind, ComputeProviderKind::AwsEc2);
    assert_eq!(receipt.tenant_id, "ten_alpha");
    assert_eq!(receipt.region, "region-alpha");
    assert_eq!(receipt.az, "region-alpha-a");

    let mut invalid_idempotency_request = request.clone();
    invalid_idempotency_request.idempotency_key = "short".to_string();
    let invalid_idempotency = ComputeProviderVmReceipt::from_request(
        ComputeProviderKind::AwsEc2,
        invalid_idempotency_request,
        "aws-req-001",
        "aws-ec2://evidence/req-001",
    )
    .expect_err("provider VM idempotency key is bounded before adapter projection");
    assert_eq!(invalid_idempotency, ComputeProviderVmError::InvalidRequest);
    let missing_request_id = ComputeProviderVmReceipt::from_request(
        ComputeProviderKind::AwsEc2,
        request.clone(),
        " ",
        "aws-ec2://evidence/req-001",
    )
    .expect_err("provider request id is required");
    assert_eq!(missing_request_id, ComputeProviderVmError::InvalidRequest);

    let missing_evidence_ref = ComputeProviderVmReceipt::from_request(
        ComputeProviderKind::AwsEc2,
        request,
        "aws-req-001",
        "",
    )
    .expect_err("provider evidence ref is required");
    assert_eq!(missing_evidence_ref, ComputeProviderVmError::InvalidRequest);
}

#[test]
fn rejects_vm_identity_location_quota_image_and_forged_state() {
    let state_error = Instance::new(InstanceCreate {
        state: InstanceState::Running,
        ..instance_create()
    })
    .expect_err("create callers cannot forge runtime state");
    assert_eq!(state_error, CloudComputeError::InvalidInstanceState);

    let quota_error = Instance::new(InstanceCreate {
        quota: ComputeQuotaEnvelope {
            vcpu_limit: 6,
            ..quota()
        },
        ..instance_create()
    })
    .expect_err("cell quota is checked before scheduling");
    assert_eq!(quota_error, CloudComputeError::QuotaExceeded);

    let image_error = Instance::new(InstanceCreate {
        image: "oci://harbor.region-alpha.oyatie.io/ten_alpha/app:latest".to_string(),
        ..instance_create()
    })
    .expect_err("image refs must be digest pinned");
    assert_eq!(image_error, CloudComputeError::InvalidImageRef);

    let cell_error = Instance::new(InstanceCreate {
        cell_id: "cell-region-alpha-b-001".to_string(),
        ..instance_create()
    })
    .expect_err("cell id must stay inside selected AZ");
    assert_eq!(cell_error, CloudComputeError::CellAzMismatch);
}
