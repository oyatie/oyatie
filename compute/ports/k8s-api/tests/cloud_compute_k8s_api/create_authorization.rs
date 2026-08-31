#[test]
fn k8s_create_api_rejects_trusted_verifier_mismatches_before_ledger() {
    for case in [
        "forged",
        "tenant",
        "principal",
        "surface",
        "decision",
        "expired",
        "stale",
    ] {
        let mut catalog = CloudComputeCatalog::default();
        let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
        let request = request(
            &format!("req-compute-k8s-authz-{case}"),
            &format!("idem-compute-k8s-authz-{case}"),
        );
        let mut proof = authorization_proof_for(
            "sp_compute",
            CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
            &request.authorization.decision_id,
        );
        let verifier_key = match case {
            "forged" => {
                proof.verified = false;
                None
            }
            "tenant" => {
                proof.tenant_id = "ten_other".to_string();
                None
            }
            "principal" => {
                proof.principal_id = "sp_other".to_string();
                None
            }
            "surface" => {
                proof.surface = "cloud.compute.vm.create".to_string();
                None
            }
            "decision" => {
                proof.decision_id = "authz_decision_other".to_string();
                Some(request.authorization.decision_id.clone())
            }
            "expired" => {
                proof.expires_at_epoch_seconds = proof.issued_at_epoch_seconds;
                None
            }
            "stale" => {
                proof.expires_at_epoch_seconds = K8S_AUTHZ_EVALUATION_EPOCH_SECONDS;
                None
            }
            _ => unreachable!("test case is exhaustive"),
        };
        let mut verifier =
            CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS);
        if let Some(decision_id) = verifier_key {
            verifier.trust_authorization_proof_for_decision(decision_id, proof);
        } else {
            verifier.trust_authorization_proof(proof);
        }

        let error = create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
            &mut catalog,
            &mut ledger,
            request,
            &verifier,
        )
        .expect_err("trusted verifier mismatch is rejected before mutation");

        assert_eq!(
            error,
            CloudComputeK8sApiError::AuthorizationDenied {
                surface: CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
            }
        );
        assert_eq!(error.cluster_create_status_code(), 403);
        assert!(ledger.is_empty());
        assert_eq!(catalog.kubernetes_clusters().count(), 0);
    }
}

#[test]
fn k8s_create_api_ignores_caller_supplied_authorization_proof() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request(
        "req-compute-k8s-ignore-proof",
        "idem-compute-k8s-ignore-proof",
    );
    request.authorization.tenant_id = "ten_forged".to_string();
    request.authorization.principal_id = "sp_forged_compute".to_string();
    request.authorization.allowed_surfaces.clear();
    request.authorization.proof = Some(CloudComputeK8sApiAuthorizationProof {
        tenant_id: "ten_other".to_string(),
        principal_id: "sp_other".to_string(),
        surface: "cloud.compute.vm.create".to_string(),
        decision_id: "authz_decision_other".to_string(),
        verified: false,
        issued_at_epoch_seconds: 1_700_099_000,
        expires_at_epoch_seconds: 1_700_099_000,
    });
    let verifier = trusted_create_verifier_for(&request);

    let response = create_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        request,
        &verifier,
    )
    .expect("trusted verifier state, not caller proof fields, authorizes create");

    assert_eq!(response.data.resource_id, CLUSTER_ID);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}

#[test]
fn k8s_create_api_separates_missing_authentication_from_denied_authorization() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let mut request = request("req-compute-k8s-authn", "idem-compute-k8s-authn");
    request.principal.principal_id.clear();

    let error = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request)
        .expect_err("missing authenticated principal is an authentication failure");

    assert_eq!(error, CloudComputeK8sApiError::EmptyPrincipalId);
    assert_eq!(error.cluster_create_status_code(), 401);
    assert!(ledger.is_empty());
    assert_eq!(catalog.kubernetes_clusters().count(), 0);
}

#[test]
fn k8s_create_api_replays_with_refreshed_authz_and_reordered_pools() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    let request = request(
        "req-compute-k8s-authz-refresh-1",
        "idem-compute-k8s-authz-refresh",
    );
    let first =
        create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, request.clone())
            .expect("initial cluster create succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-k8s-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.proof = Some(authorization_proof_for(
        "sp_compute",
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE,
        &retry.authorization.decision_id,
    ));
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.vm.create".to_string(),
        CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE.to_string(),
    ];
    retry.body.node_pools.reverse();
    for pool in &mut retry.body.node_pools {
        pool.security_groups.reverse();
    }
    let second = create_cloud_compute_k8s_cluster_from_api(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.kubernetes_clusters().count(), 1);
}
