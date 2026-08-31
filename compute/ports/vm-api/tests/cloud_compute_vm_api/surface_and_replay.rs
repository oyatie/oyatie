#[test]
fn openapi_runtime_binding_contracts_are_covered() {
    assert_eq!(CLOUD_COMPUTE_VM_CREATE_SURFACE, "cloud.compute.vm.create");
    assert_eq!(CloudComputeVmCreateApiStatus::Created.code(), 201);
    assert_eq!(CloudComputeVmCreateApiStatus::BadRequest.code(), 400);
    assert_eq!(CloudComputeVmCreateApiStatus::Unauthorized.code(), 401);
    assert_eq!(CloudComputeVmCreateApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeVmCreateApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeVmCreateApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeVmCreateApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn vm_create_api_creates_instance_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request("req-compute-vm-create", "idem-compute-vm-create");

    let first = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("authorized VM create succeeds");
    let second = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.instances().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-vm-create");
    assert_eq!(first.data.resource_id, INSTANCE_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.az, "region-home-a");
    assert_eq!(first.data.cell_id, "cell-region-home-a-001");
    assert_eq!(first.data.flavor.class, "general_purpose");
    assert_eq!(first.data.flavor.vcpu, 4);
    assert_eq!(first.data.image_kind, "oci");
    assert_eq!(first.data.residency, "strict_home_region");
    assert_eq!(first.data.state, "pending");
    assert_eq!(first.data.data_class, "PUBLIC");
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn vm_create_api_uses_trusted_verifier_and_ignores_caller_supplied_proof_fields() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request(
        "req-compute-vm-ignore-caller-proof",
        "idem-compute-vm-ignore-caller-proof",
    );
    request.authorization.tenant_id = "ten_forged".to_string();
    request.authorization.principal_id = "sp_forged_compute".to_string();
    request.authorization.allowed_surfaces = vec!["cloud.compute.k8s.cluster.create".to_string()];
    let mut forged_proof = authorization_proof_for(
        "sp_attacker",
        "cloud.compute.k8s.cluster.create",
        "authz_decision_attacker",
    );
    forged_proof.tenant_id = "ten_other".to_string();
    forged_proof.expires_at_epoch_seconds = forged_proof.issued_at_epoch_seconds;
    request.authorization.proof = Some(forged_proof);

    let response = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect("trusted verifier state authorizes independently of caller proof fields");

    assert_eq!(response.data.resource_id, INSTANCE_ID);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.instances().count(), 1);
}

#[test]
fn vm_create_api_rejects_path_body_drift_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request("req-compute-vm-drift", "idem-compute-vm-drift");
    request.body.resource_id = "oyatie:cloud:region-home:ten_alpha:instance:other".to_string();

    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("path/body instance drift is rejected");

    assert_eq!(
        error,
        CloudComputeVmApiError::InstanceIdMismatch {
            path_instance_id: INSTANCE_ID.to_string(),
            body_resource_id: "oyatie:cloud:region-home:ten_alpha:instance:other".to_string(),
        }
    );
    assert_eq!(error.vm_create_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}
