#[test]
fn api_surface_status_contracts_are_covered() {
    assert_eq!(
        CLOUD_COMPUTE_FUNCTIONS_INVOKE_SURFACE,
        "cloud.compute.functions.invoke"
    );
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Accepted.code(), 202);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::BadRequest.code(), 400);
    assert_eq!(
        CloudComputeFunctionsInvokeApiStatus::Unauthorized.code(),
        401
    );
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Forbidden.code(), 403);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::NotFound.code(), 404);
    assert_eq!(CloudComputeFunctionsInvokeApiStatus::Conflict.code(), 409);
    assert_eq!(
        CloudComputeFunctionsInvokeApiStatus::UnprocessableEntity.code(),
        422
    );
}

#[test]
fn functions_invoke_api_records_invocation_once_and_replays_same_idempotent_result() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-invoke",
        "idem-functions-invoke-001",
        "fninv_001",
    );

    let first = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request.clone())
        .expect("authorized function invoke succeeds");
    let second = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect("same idempotency fingerprint replays");

    assert_eq!(first, second);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
    assert_eq!(first.metadata.request_id, "req-compute-functions-invoke");
    assert_eq!(first.data.invocation_id, "fninv_001");
    assert_eq!(first.data.tenant_id, "ten_alpha");
    assert_eq!(first.data.function_id, FUNCTION_ID);
    assert_eq!(first.data.region, "region-home");
    assert_eq!(first.data.payload_data_class, "PII_IDENTIFYING");
    assert_eq!(first.data.cold_start_budget_ms, 750);
    assert_eq!(first.data.accepted_at_epoch_seconds, 1_700_100_030);
    assert_eq!(first.data.schema_version, 1);
}

#[test]
fn planned_invoke_entrypoint_fails_closed_without_verifier_and_verified_entrypoint_succeeds() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut legacy_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let request = request(
        "req-compute-functions-invoke-alias",
        "idem-functions-invoke-alias",
        "fninv_alias",
    );

    let legacy_error = invoke(&mut catalog, &mut legacy_ledger, request.clone())
        .expect_err("legacy invoke entrypoint fails closed without compute verifier");
    assert_eq!(
        legacy_error,
        CloudComputeFunctionsApiError::AuthorizationVerifierMissing
    );
    assert_eq!(legacy_error.invoke_status_code(), 403);
    assert!(legacy_ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);

    let authorization_verifier = authorization_verifier_for(&request);
    let mut verified_ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let response = invoke_with_authorization_verifier(
        &mut catalog,
        &mut verified_ledger,
        &authorization_verifier,
        request,
    )
    .expect("stable planned invoke entrypoint succeeds with compute verifier");

    assert_eq!(
        response.metadata.request_id,
        "req-compute-functions-invoke-alias"
    );
    assert_eq!(response.data.invocation_id, "fninv_alias");
    assert_eq!(response.data.function_id, FUNCTION_ID);
    assert_eq!(verified_ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 1);
}
