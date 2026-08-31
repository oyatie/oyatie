#[test]
fn vm_create_api_rejects_required_header_and_tenant_drift_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut empty_request = request(" ", "idem-compute-vm-empty-header");
    assert_eq!(
        create_vm_with_trusted_verifier(&mut catalog, &mut ledger, empty_request.clone()),
        Err(CloudComputeVmApiError::EmptyRequestId)
    );

    empty_request.boundary.request_id = "req-compute-vm-tenant-drift".to_string();
    empty_request.boundary.tenant_id = "ten_other".to_string();
    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, empty_request)
        .expect_err("tenant drift is rejected before idempotency ledger write");

    assert_eq!(error.vm_create_status_code(), 403);
    assert!(matches!(
        error,
        CloudComputeVmApiError::TenantMismatch { .. }
    ));
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_trusted_verifier_surface_mismatch_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request("req-compute-vm-authz", "idem-compute-vm-authz");
    let verifier = trusted_verifier_with(authorization_proof_for(
        "sp_compute",
        "cloud.compute.k8s.cluster.create",
        &request.authorization.decision_id,
    ));

    let error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        request,
        &verifier,
    )
    .expect_err("trusted verifier decision does not allow VM create");

    assert_eq!(error, authorization_denied());
    assert_eq!(error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_legacy_entrypoint_fails_closed_without_verifier_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-vm-missing-verifier",
        "idem-compute-vm-missing-verifier",
    );

    let error = create_cloud_compute_vm_from_api(&mut catalog, &mut ledger, request)
        .expect_err("legacy VM create entrypoint must not trust caller-supplied proof");

    assert_eq!(error, authorization_denied());
    assert_eq!(error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_rejects_trusted_verifier_tenant_principal_and_decision_mismatch() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();

    let tenant_request = request(
        "req-compute-vm-tenant-authz",
        "idem-compute-vm-tenant-authz",
    );
    let mut tenant_proof = authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &tenant_request.authorization.decision_id,
    );
    tenant_proof.tenant_id = "ten_other".to_string();
    let tenant_verifier = trusted_verifier_with(tenant_proof);
    let tenant_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        tenant_request,
        &tenant_verifier,
    )
    .expect_err("trusted verifier proof bound to another tenant is rejected");
    assert_eq!(tenant_error, authorization_denied());

    let principal_request = request(
        "req-compute-vm-principal-authz",
        "idem-compute-vm-principal-authz",
    );
    let principal_verifier = trusted_verifier_with(authorization_proof_for(
        "sp_other",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &principal_request.authorization.decision_id,
    ));
    let principal_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        principal_request,
        &principal_verifier,
    )
    .expect_err("trusted verifier proof bound to another principal is rejected");
    assert_eq!(principal_error, authorization_denied());

    let decision_request = request(
        "req-compute-vm-decision-authz",
        "idem-compute-vm-decision-authz",
    );
    let decision_verifier = trusted_verifier_with(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        "authz_decision_other",
    ));
    let decision_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        decision_request,
        &decision_verifier,
    )
    .expect_err("trusted verifier state is keyed by the requested decision id");
    assert_eq!(decision_error, authorization_denied());

    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}
