#[test]
fn functions_invoke_api_ignores_caller_supplied_authorization_claim_fields() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-authz-caller-fields",
        "idem-functions-authz-caller-fields-001",
        "fninv_authz_caller_fields",
    );
    let authorization_verifier = authorization_verifier_for(&request);
    request.authorization.tenant_id = "ten_beta".to_string();
    request.authorization.principal_id = "sp_forged_compute".to_string();
    request.authorization.requested_surface = "cloud.compute.vm.create".to_string();
    request.authorization.valid_until_epoch_seconds = request.body.requested_at_epoch_seconds;
    request.authorization.allowed_surfaces.clear();

    let response = invoke_cloud_compute_function_from_api_with_authorization_verifier(
        &mut catalog,
        &mut ledger,
        &authorization_verifier,
        request,
    )
    .expect("compute verifier state, not caller-supplied authorization claims, controls allow");

    assert_eq!(response.data.invocation_id, "fninv_authz_caller_fields");
    assert_eq!(response.data.function_id, FUNCTION_ID);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_replays_with_refreshed_authz() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-authz-refresh-1",
        "idem-functions-authz-refresh-001",
        "fninv_authz_refresh",
    );
    let first = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial function invoke succeeds");

    let mut retry = request;
    retry.boundary.request_id = "req-compute-functions-authz-refresh-2".to_string();
    retry.authorization.decision_id = "authz_decision_sp_compute_refreshed".to_string();
    retry.authorization.allowed_surfaces = vec![
        "cloud.compute.k8s.cluster.create".to_string(),
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE.to_string(),
    ];
    let second = invoke_with_trusted_verifier(&mut catalog, &mut ledger, retry)
        .expect("refreshed authorization evidence does not change operation fingerprint");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_rejects_reused_idempotency_key_with_new_fingerprint() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-idem",
        "idem-functions-idem-001",
        "fninv_idem",
    );
    invoke_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("initial invoke succeeds");

    let mut drifted = request;
    drifted.body.payload_data_class = "PUBLIC".to_string();
    assert_eq!(
        invoke_with_trusted_verifier(&mut catalog, &mut ledger, drifted),
        Err(CloudComputeFunctionsApiError::IdempotencyKeyReused {
            idempotency_key: "idem-functions-idem-001".to_string(),
        })
    );
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn functions_invoke_api_maps_unknown_inactive_and_duplicate_invocations() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let unknown = invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-compute-functions-unknown",
            "idem-functions-unknown-001",
            "fninv_unknown",
        ),
    )
    .expect_err("unknown function is not found");
    assert_eq!(
        unknown,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::UnknownFunction)
    );
    assert_eq!(unknown.invoke_status_code(), 404);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);

    let mut inactive_catalog = CloudComputeCatalog::default();
    seed_deploying_function(&mut inactive_catalog);
    let mut inactive_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let inactive = invoke_with_trusted_verifier(
        &mut inactive_catalog,
        &mut inactive_ledger,
        request(
            "req-compute-functions-inactive",
            "idem-functions-inactive-001",
            "fninv_inactive_api",
        ),
    )
    .expect_err("deploying functions cannot be invoked");
    assert_eq!(
        inactive,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::FunctionNotActive)
    );
    assert_eq!(inactive.invoke_status_code(), 409);
    assert_eq!(inactive_ledger.len(), 1);
    assert_eq!(inactive_catalog.invocations().count(), 0);

    let mut active_catalog = CloudComputeCatalog::default();
    seed_active_function(&mut active_catalog);
    let mut duplicate_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    invoke_with_trusted_verifier(
        &mut active_catalog,
        &mut duplicate_ledger,
        request(
            "req-compute-functions-duplicate-a",
            "idem-functions-duplicate-a",
            "fninv_duplicate",
        ),
    )
    .expect("first invoke succeeds");
    let duplicate = invoke_with_trusted_verifier(
        &mut active_catalog,
        &mut duplicate_ledger,
        request(
            "req-compute-functions-duplicate-b",
            "idem-functions-duplicate-b",
            "fninv_duplicate",
        ),
    )
    .expect_err("same invocation id with a new idempotency key conflicts");
    assert_eq!(
        duplicate,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::DuplicateInvocation)
    );
    assert_eq!(duplicate.invoke_status_code(), 409);
    assert_eq!(duplicate_ledger.len(), 2);
    assert_eq!(active_catalog.invocations().count(), 1);
}
