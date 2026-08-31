#[test]
fn vm_create_api_rejects_unverified_or_expired_trusted_verifier_proof_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();

    let unverified_request = request(
        "req-compute-vm-unverified-authz",
        "idem-compute-vm-unverified-authz",
    );
    let mut unverified_proof = authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &unverified_request.authorization.decision_id,
    );
    unverified_proof.verified = false;
    let unverified_verifier = trusted_verifier_with(unverified_proof);
    let unverified_error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        unverified_request,
        &unverified_verifier,
    )
    .expect_err("unverified trusted verifier proof is rejected");
    assert_eq!(unverified_error, authorization_denied());
    assert_eq!(unverified_error.vm_create_status_code(), 403);
    let request = request(
        "req-compute-vm-expired-authz",
        "idem-compute-vm-expired-authz",
    );
    let mut proof = authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &request.authorization.decision_id,
    );
    proof.expires_at_epoch_seconds = VERIFIER_EVALUATION_EPOCH_SECONDS;
    let verifier = trusted_verifier_with(proof);

    let error = create_cloud_compute_vm_from_api_with_verifier(
        &mut catalog,
        &mut ledger,
        request,
        &verifier,
    )
    .expect_err("expired trusted verifier proof is rejected");

    assert_eq!(error, authorization_denied());
    assert_eq!(error.vm_create_status_code(), 403);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request("req-compute-vm-authn", "idem-compute-vm-authn");
    request.principal.principal_id = " ".to_string();

    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(error, CloudComputeVmApiError::EmptyPrincipalId);
    assert_eq!(error.vm_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_api_replays_with_refreshed_authz_and_reordered_security_groups() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-vm-authz-refresh-1",
        "idem-compute-vm-authz-refresh",
    );
    let first = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial VM create succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-vm-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.proof = Some(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_VM_CREATE_SURFACE,
        &retry.authorization.decision_id,
    ));
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.k8s.cluster.create".to_string(),
        CLOUD_COMPUTE_VM_CREATE_SURFACE.to_string(),
    ];
    retry.body.security_groups.reverse();
    let second = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.instances().count(), 1);
}
