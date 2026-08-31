#[test]
fn vm_create_api_rejects_foreign_security_group_and_iam_role_proofs_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut group_request = request("req-compute-vm-sg-proof", "idem-compute-vm-sg-proof");
    group_request.body.security_groups[0].tenant_id = "ten_other".to_string();

    let group_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, group_request)
        .expect_err("security group proof must match tenant boundary");

    assert!(matches!(
        group_error,
        CloudComputeVmApiError::SecurityGroupBindingMismatch { .. }
    ));
    assert_eq!(group_error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);

    let mut role_request = request("req-compute-vm-role-proof", "idem-compute-vm-role-proof");
    role_request
        .body
        .iam_role
        .as_mut()
        .expect("role ref exists")
        .vpc_id = "oyatie:cloud:region-home:ten_other:vpc:foreign".to_string();
    let role_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, role_request)
        .expect_err("IAM role proof must match VPC boundary");

    assert!(matches!(
        role_error,
        CloudComputeVmApiError::IamRoleBindingMismatch { .. }
    ));
    assert_eq!(role_error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request("req-compute-vm-idem", "idem-compute-vm-idem");
    create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial VM create succeeds");

    let mut drifted = request;
    drifted.body.flavor.memory_gb = 32;
    assert_eq!(
        create_vm_with_trusted_verifier(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeVmApiError::IdempotencyKeyReused {
            idempotency_key: "idem-compute-vm-idem".to_string(),
        })
    );
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_maps_duplicate_instance_to_conflict() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-compute-vm-dup-1", "idem-compute-vm-dup-1"),
    )
    .expect("first VM create succeeds");

    let error = create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-compute-vm-dup-2", "idem-compute-vm-dup-2"),
    )
    .expect_err("same instance id through new idempotency key is a conflict");

    assert_eq!(
        error,
        CloudComputeVmApiError::Compute(CloudComputeError::DuplicateInstance)
    );
    assert_eq!(error.vm_create_status_code(), 409);
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_maps_quota_residency_and_invalid_image_without_masking() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut quota_request = request("req-compute-vm-quota", "idem-compute-vm-quota");
    quota_request.body.quota.vcpu_limit = 6;
    let quota_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, quota_request)
        .expect_err("tenant quota is enforced");
    assert_eq!(
        quota_error,
        CloudComputeVmApiError::Compute(CloudComputeError::QuotaExceeded)
    );
    assert_eq!(quota_error.vm_create_status_code(), 403);

    let mut residency_request = request("req-compute-vm-residency", "idem-compute-vm-residency");
    residency_request.body.region = "failover-region".to_string();
    residency_request.body.az = "failover-region-a".to_string();
    residency_request.body.cell_id = "cell-failover-region-a-001".to_string();
    residency_request.body.resource_id =
        "oyatie:cloud:failover-region:ten_alpha:instance:app-1".to_string();
    residency_request.path_instance_id = residency_request.body.resource_id.clone();
    residency_request.body.vpc_id = "oyatie:cloud:failover-region:ten_alpha:vpc:prod".to_string();
    residency_request.body.subnet_id =
        "oyatie:cloud:failover-region:ten_alpha:subnet:prod-a".to_string();
    for group in &mut residency_request.body.security_groups {
        group.region = "failover-region".to_string();
        group.vpc_id = "oyatie:cloud:failover-region:ten_alpha:vpc:prod".to_string();
    }
    if let Some(role) = &mut residency_request.body.iam_role {
        role.region = "failover-region".to_string();
        role.vpc_id = "oyatie:cloud:failover-region:ten_alpha:vpc:prod".to_string();
    }
    let residency_error =
        create_vm_with_trusted_verifier(&mut catalog, &mut ledger, residency_request)
            .expect_err("strict home-region residency denies US VM placement");
    assert_eq!(
        residency_error,
        CloudComputeVmApiError::Compute(CloudComputeError::ResidencyRegionMismatch)
    );
    assert_eq!(residency_error.vm_create_status_code(), 403);

    let mut image_request = request("req-compute-vm-image", "idem-compute-vm-image");
    image_request.body.image =
        "oci://harbor.region-home.oyatie.io/ten_alpha/app:latest".to_string();
    let image_error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, image_request)
        .expect_err("image refs must be digest pinned");
    assert_eq!(
        image_error,
        CloudComputeVmApiError::Compute(CloudComputeError::InvalidImageRef)
    );
    assert_eq!(image_error.vm_create_status_code(), 400);
    assert_eq!(catalog.instances().count(), 0);
}
